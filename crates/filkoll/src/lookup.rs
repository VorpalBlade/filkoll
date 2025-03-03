use crate::interner::ArchivedStringInterner;
use crate::types::ArchivedDataRoot;
use crate::types::ArchivedRecord;
use crate::types::Header;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use compact_str::CompactString;
use memmap2::Mmap;
use std::fs::File;
use zerocopy::FromBytes;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Suggestion {
    pub distance: u8,
    pub repo: CompactString,
    pub package: CompactString,
    pub directory: CompactString,
    pub command: CompactString,
}

impl Suggestion {
    fn from_record(
        record: &ArchivedRecord,
        command: &str,
        interner: &ArchivedStringInterner,
        repo: &str,
        distance: u8,
    ) -> Self {
        Self {
            repo: repo.into(),
            package: interner.get(record.package.0).into(),
            directory: interner.get(record.directory.0).into(),
            command: command.into(),
            distance,
        }
    }
}

/// Suggestions for search results
pub type Suggestions = Vec<Suggestion>;

pub fn lookup(
    edit_distance: u8,
    no_fuzzy_if_exact: bool,
    search_term: &str,
) -> eyre::Result<Suggestions> {
    let files = find_files()?;

    if files.is_empty() {
        return Err(eyre::eyre!(
            "No cache files found (if this is a new install: sudo filkoll update && sudo \
             systemctl enable filkoll.timer)"
        ));
    }

    let results = files
        .iter()
        .map(|file| search_in_file(file, search_term, edit_distance))
        .collect::<Result<Vec<_>, _>>()?;

    let results: Vec<Suggestion> = results.into_iter().flatten().collect();

    if no_fuzzy_if_exact {
        // Filter out any exact matches
        let exact_matches: Vec<Suggestion> = results
            .iter()
            .filter(|suggestion| suggestion.distance == 0)
            .cloned()
            .collect();
        if !exact_matches.is_empty() {
            return Ok(exact_matches);
        }
    }

    Ok(results)
}

/// Locate *.binaries files in the cache
fn find_files() -> eyre::Result<Vec<Utf8PathBuf>> {
    let files = std::fs::read_dir(crate::CACHE_PATH)?;
    let files = files
        .map(|f| f.map(|f| f.path().try_into()))
        .collect::<Result<Result<Vec<_>, _>, _>>()??;
    Ok(files)
}

fn search_in_file(
    file: &Utf8Path,
    search_term: &str,
    max_edit_dist: u8,
) -> eyre::Result<Suggestions> {
    let file = File::options().read(true).open(file)?;
    // SAFETY:
    // * When the file is written it is created anew, not overwritten in place. As
    //   such it cannot change under us.
    let mmap = unsafe { Mmap::map(&file) }?;

    // Read the header at the start of the file
    // Note: The error contains lifetimes to the buffer, so we need to eagerly
    // convert the error to a string.
    let (header, payload) =
        Header::ref_from_prefix(&mmap).map_err(|e| eyre::eyre!("Failed to load header: {e}"))?;

    if !header.is_valid() {
        return Err(eyre::eyre!(
            "Invalid header in cache file (try sudo filkoll update)"
        ));
    }

    // Load archive from payload
    // SAFETY:
    // * The header has already been validated, so we know the payload is from a
    //   matching version
    let data = unsafe { rkyv::api::access_unchecked::<ArchivedDataRoot>(payload) };

    search_data_root(search_term, max_edit_dist, data)
}

/// Search an archived data root.
fn search_data_root(
    search_term: &str,
    max_edit_dist: u8,
    data: &ArchivedDataRoot,
) -> Result<Vec<Suggestion>, eyre::Error> {
    if max_edit_dist == 0 {
        let exact_match = data.binaries.get(search_term);

        if let Some(exact_match) = exact_match {
            let result = exact_match
                .iter()
                .map(|value| {
                    Suggestion::from_record(value, search_term, &data.interner, &data.repository, 0)
                })
                .collect();
            return Ok(result);
        }
        Ok(vec![])
    } else {
        let mut suggestions = Suggestions::new();
        if max_edit_dist > 0 {
            // Fuzzy search
            for (key, value) in data.binaries.iter() {
                let dist = strsim::levenshtein(key, search_term);
                if dist <= max_edit_dist as usize {
                    suggestions.extend(value.iter().map(|value| {
                        Suggestion::from_record(
                            value,
                            key,
                            &data.interner,
                            &data.repository,
                            dist as u8,
                        )
                    }));
                }
            }
        }
        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use crate::lookup::Suggestion;
    use crate::types::ArchivedDataRoot;
    use crate::types::DataRoot;
    use crate::types::DirectoryRef;
    use crate::types::PackageRef;
    use smallvec::SmallVec;

    #[test]
    fn test_search() {
        // Create string interner:
        let mut builder = crate::interner::StringInternerBuilder::new();
        let usrbin = DirectoryRef(builder.intern("/usr/bin"));
        let coreutils = PackageRef(builder.intern("coreutils"));
        let rust = PackageRef(builder.intern("rust"));
        let sl = PackageRef(builder.intern("sl"));
        let mut binaries = crate::types::BinariesData::new();
        binaries.insert(
            "ls".into(),
            SmallVec::from_slice(&[crate::types::Record {
                package: coreutils,
                directory: usrbin,
            }]),
        );
        binaries.insert(
            "sl".into(),
            SmallVec::from_slice(&[crate::types::Record {
                package: sl,
                directory: usrbin,
            }]),
        );
        binaries.insert(
            "cat".into(),
            SmallVec::from_slice(&[crate::types::Record {
                package: coreutils,
                directory: usrbin,
            }]),
        );
        binaries.insert(
            "cargo".into(),
            SmallVec::from_slice(&[crate::types::Record {
                package: rust,
                directory: usrbin,
            }]),
        );
        let data = DataRoot {
            repository: "core".into(),
            interner: builder.into_readonly(),
            binaries,
        };
        // Serialise and deserialise data to get the archived form
        let serialised = rkyv::api::high::to_bytes::<rkyv::rancor::Error>(&data).unwrap();
        let archived =
            rkyv::api::high::access::<ArchivedDataRoot, rkyv::rancor::Error>(&serialised).unwrap();

        // Test exact match
        let results = super::search_data_root("ls", 0, archived).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0],
            Suggestion {
                distance: 0,
                repo: "core".into(),
                package: "coreutils".into(),
                directory: "/usr/bin".into(),
                command: "ls".into()
            }
        );

        // Test fuzzy match
        let mut results = super::search_data_root("ls", 2, archived).unwrap();
        assert_eq!(results.len(), 2);
        results.sort();
        assert_eq!(
            results,
            vec![
                Suggestion {
                    distance: 0,
                    repo: "core".into(),
                    package: "coreutils".into(),
                    directory: "/usr/bin".into(),
                    command: "ls".into()
                },
                Suggestion {
                    distance: 2,
                    repo: "core".into(),
                    package: "sl".into(),
                    directory: "/usr/bin".into(),
                    command: "sl".into()
                }
            ]
        );
    }
}
