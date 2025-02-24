//! Parse pacman.conf

use compact_str::CompactString;
use eyre::OptionExt;
use eyre::WrapErr;
use std::io::Read;

/// Pacman configuration (or at least the parts we care about)
#[derive(Debug)]
pub struct PacmanConfig {
    pub db_path: CompactString,
}

impl PacmanConfig {
    pub fn new(file: &mut impl Read) -> eyre::Result<Self> {
        let parser = ini::Ini::read_from(file).wrap_err("Failed to open pacman.conf")?;
        let options: &ini::Properties = parser
            .section(Some("options"))
            .ok_or_eyre("Could not find options section in pacman.conf")?;

        Ok(Self {
            db_path: options.get("DBPath").unwrap_or("/var/lib/pacman/").into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pacman_config() {
        let file = indoc::indoc! {"
            [options]
            RootDir = /other
            DBPath = /dbpath
            # comment
            # Cachedir not set
        "};

        let config = PacmanConfig::new(&mut file.as_bytes()).unwrap();
        assert_eq!(config.db_path, "/dbpath");
    }
}
