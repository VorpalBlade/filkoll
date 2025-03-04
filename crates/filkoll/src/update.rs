use crate::arch::Package;
use crate::arch::pacman_conf::PacmanConfig;
use crate::interner::StringInternerBuilder;
use crate::types::BinariesData;
use crate::types::DataRoot;
use crate::types::DirectoryRef;
use crate::types::PackageRef;
use crate::types::Record;
use bstr::ByteSlice;
use bstr::io::BufReadExt;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use compact_str::CompactString;
use compact_str::format_compact;
use eyre::Context;
use flate2::bufread::GzDecoder;
use hashbrown::hash_map::Entry;
use itertools::Itertools;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use smallvec::SmallVec;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::str::FromStr;
use zerocopy::IntoBytes;

pub fn update(pacman_download: bool) -> eyre::Result<()> {
    if pacman_download {
        // Run pacman -Fy
        let _output = std::process::Command::new("pacman").arg("-Fy").status()?;
    }
    let files = find_files()?;
    tracing::trace!("Found {} *.files", files.len());

    // Create pair of regex based on PATH. We do this because parent() on a path is
    // slow
    #[allow(unstable_name_collisions)]
    let prefilter_regex = std::env::var("PATH")
        .wrap_err("Failed to read PATH")?
        .split(':')
        .filter_map(|mut segment| {
            if segment.is_empty() {
                return None;
            }
            if segment.as_bytes()[0] == b'/' {
                segment = &segment[1..];
            }
            if segment.ends_with("/") {
                Some(format_compact!("^{segment}"))
            } else {
                Some(format_compact!("^{segment}/"))
            }
        })
        .intersperse("|".into())
        .collect::<CompactString>();
    let prefilter = regex::bytes::Regex::new(&prefilter_regex)?;

    // There is no nice way to write this other than with chaining intersperse, so
    // allow unstable name collisions.
    #[allow(unstable_name_collisions)]
    let exact_regex = std::env::var("PATH")
        .wrap_err("Failed to read PATH")?
        .split(':')
        .filter_map(|mut segment| {
            if segment.is_empty() {
                return None;
            }
            if segment.as_bytes()[0] == b'/' {
                segment = &segment[1..];
            }
            if segment.ends_with("/") {
                Some(format_compact!("^{}$", &segment[0..segment.len() - 1]))
            } else {
                Some(format_compact!("^{segment}$"))
            }
        })
        .intersperse("|".into())
        .collect::<CompactString>();
    let filter = regex::Regex::new(&exact_regex)?;
    let prefilter = |path: &[u8]| prefilter.is_match(path);
    let filter = |path: &str| filter.is_match(path);

    // Ensure our cache path exists
    std::fs::create_dir_all(crate::CACHE_PATH)?;

    files
        .par_iter()
        .try_for_each(|file| update_file(file, &prefilter, &filter))?;

    Ok(())
}

#[tracing::instrument(skip(pre_filter, filter))]
fn update_file(
    file: &Utf8Path,
    pre_filter: &impl Fn(&[u8]) -> bool,
    filter: &impl Fn(&str) -> bool,
) -> eyre::Result<()> {
    tracing::info!("Processing {:?}", file);
    let data_root = process_files_archive(file, pre_filter, filter)?;
    tracing::info!("Processed {:?}", file);
    let cache_path = Utf8PathBuf::from_str(crate::CACHE_PATH)?;
    let cache_path = cache_path.join(
        file.file_stem()
            .ok_or_else(|| eyre::eyre!("Failed to get file stem"))?,
    );
    let cache_path = cache_path.with_extension("binaries");
    let tmp_path = cache_path.with_extension("binaries_new");
    let file = File::create(&tmp_path)?;

    let mut writer = std::io::BufWriter::new(file);
    // Write file header
    let header = crate::types::Header::default();
    writer.write_all(header.as_bytes())?;

    // Write data
    tracing::info!("Writing {:?}", &tmp_path);
    let writer = rkyv::ser::writer::IoWriter::new(&mut writer);
    rkyv::api::high::to_bytes_in::<_, rkyv::rancor::Error>(&data_root, writer)?;
    tracing::info!("Wrote {:?}", &tmp_path);
    // INVARIANT: Rename in place of old file.
    // This is a safety requirement to allow mmaping the file during lookup.
    std::fs::rename(&tmp_path, &cache_path)?;
    Ok(())
}

/// Locate *.files from pacman sync directory
fn find_files() -> eyre::Result<Vec<Utf8PathBuf>> {
    // Load pacman.conf to find the sync directory
    let mut readable = BufReader::new(File::open("/etc/pacman.conf")?);
    let pacman_config: PacmanConfig = PacmanConfig::new(&mut readable)?;
    let mut sync_dir = Utf8PathBuf::from_str(&pacman_config.db_path)?;
    sync_dir.push("sync");
    // Locate *.files
    let files = std::fs::read_dir(sync_dir)?;
    let files = files
        .filter_map(|entry| -> Option<Utf8PathBuf> {
            let entry = entry.ok()?;
            let path = Utf8PathBuf::from_path_buf(entry.path()).ok()?;
            if path.extension()? == "files" {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Ok(files)
}

fn process_files_archive(
    path: &Utf8Path,
    pre_filter: &impl Fn(&[u8]) -> bool,
    filter: &impl Fn(&str) -> bool,
) -> eyre::Result<DataRoot> {
    let mut interner = StringInternerBuilder::new();
    let binaries = process_files_archive_inner(path, &mut interner, pre_filter, filter)?;
    let interner = interner.into_readonly();
    Ok(DataRoot {
        repository: path
            .file_stem()
            .ok_or_else(|| eyre::eyre!("No file stem"))?
            .to_string(),
        interner,
        binaries,
    })
}

#[derive(Debug, Default)]
struct RecordWorkInProgress {
    package: Option<CompactString>,
    /// File name and containing directory
    files: SmallVec<[(String, DirectoryRef); 2]>,
}

fn process_files_archive_inner(
    path: &Utf8Path,
    interner: &mut StringInternerBuilder,
    pre_filter: &impl Fn(&[u8]) -> bool,
    filter: &impl Fn(&str) -> bool,
) -> eyre::Result<BinariesData> {
    let file = BufReader::new(File::open(path)?);
    let decoder = GzDecoder::new(file);
    if decoder.header().is_none() {
        eyre::bail!(
            "Failed to open {:?} as gzip compressed (did Arch Linux change formats?)",
            path
        );
    }

    let mut archive = tar::Archive::new(decoder);

    let mut package_map: hashbrown::HashMap<Utf8PathBuf, RecordWorkInProgress> =
        hashbrown::HashMap::new();
    // Buffer to reduce allocations
    let mut str_buffer = String::new();

    for entry in archive.entries().wrap_err("Failed to read files archive")? {
        let entry = entry.wrap_err("TAR parsing error (entry)")?;
        let path = entry.path().wrap_err("TAR parsing error (path)")?;
        let path = Utf8Path::from_path(&path).ok_or_else(|| eyre::eyre!("Non-UTF8 path"))?;
        let file_name = path.file_name().ok_or_else(|| eyre::eyre!("No filename"))?;
        let dir_name = path
            .parent()
            .ok_or_else(|| eyre::eyre!("No parent"))?
            .to_owned();

        match file_name {
            "desc" => {
                // Extract package name and version
                let desc = BufReader::new(entry);
                let pkg =
                    Package::from_desc(desc).wrap_err("Failed to parse package description")?;
                let pkg_name = format_compact!("{} {}", pkg.name, pkg.version);
                match package_map.entry(dir_name) {
                    Entry::Occupied(occupied_entry) => {
                        occupied_entry.into_mut().package = Some(pkg_name);
                    }
                    Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(RecordWorkInProgress {
                            package: Some(pkg_name),
                            files: SmallVec::new(),
                        });
                    }
                };
                str_buffer.clear();
            }
            "files" => {
                // Extract file list
                let mut contents = SmallVec::new();
                let mut bufreader = BufReader::new(entry);
                bufreader.for_byte_line(|line: &[u8]| {
                    if line == b"" || line == b"%FILES%" {
                        return Ok(true);
                    }
                    // Do a first approximate filter, as parent() is slow
                    if !pre_filter(line) {
                        return Ok(true);
                    }
                    let line = line.to_str().map_err(std::io::Error::other)?;

                    let path: &Utf8Path = Utf8Path::new(line.trim());
                    let parent = path.parent();
                    let Some(parent) = parent else {
                        tracing::warn!("No parent for {:?}", path);
                        return Ok(true);
                    };
                    // Do an exact check to detect sub-directories
                    if !filter(parent.as_str()) {
                        return Ok(true);
                    }
                    let file_name = path.file_name();
                    let Some(file_name) = file_name else {
                        tracing::warn!("No file name for {:?}", path);
                        return Ok(true);
                    };
                    contents.push((
                        file_name.to_string(),
                        DirectoryRef(interner.intern(parent.as_str())),
                    ));
                    Ok(true)
                })?;
                match package_map.entry(dir_name) {
                    Entry::Occupied(occupied_entry) => {
                        occupied_entry.into_mut().files = contents;
                    }
                    Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(RecordWorkInProgress {
                            package: None,
                            files: contents,
                        });
                    }
                };
            }
            _ => {
                // Ignore other files
            }
        }
    }

    // Now build a btree mapping file name to Vec<Record>. This is needed to get a
    // sorted list in the next step.
    let mut binaries = BinariesData::new();
    for (dir, work) in package_map.into_iter() {
        if work.files.is_empty() {
            continue;
        }
        let package = work
            .package
            .ok_or_else(|| eyre::eyre!("No package for {:?}", dir))?;
        // We delay creating the package ref until here to avoid adding interning for
        // packages without bin files
        let package = PackageRef(interner.intern(package.as_str()));
        for (file, dir) in work.files {
            binaries
                .entry(file)
                .or_insert_with(SmallVec::new)
                .push(Record {
                    package,
                    directory: dir,
                });
        }
    }

    Ok(binaries)
}
