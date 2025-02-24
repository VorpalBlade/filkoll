use compact_str::CompactString;
use std::io::BufRead;

pub(crate) mod pacman_conf;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Package {
    pub name: CompactString,
    pub version: CompactString,
}

impl Package {
    pub fn from_desc(mut readable: impl BufRead) -> eyre::Result<Self> {
        let mut name: Option<CompactString> = None;
        let mut version: Option<CompactString> = None;

        let mut line = String::new();
        while readable.read_line(&mut line)? > 0 {
            if line == "%NAME%\n" {
                line.clear();
                readable.read_line(&mut line)?;
                name = Some(line.trim_end().into());
            } else if line == "%VERSION%\n" {
                line.clear();
                readable.read_line(&mut line)?;
                version = Some(line.trim_end().into());
            }
            if name.is_some() && version.is_some() {
                break;
            }
            line.clear();
        }

        Ok(Self {
            name: name.ok_or_else(|| eyre::eyre!("No name"))?,
            version: version.ok_or_else(|| eyre::eyre!("No version"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse() {
        let input = indoc::indoc! {"
            %NAME%
            library-subpackage

            %VERSION%
            1.2.3-4

            %BASE%
            library-base

            %DESC%
            Some library

            %URL%
            https://example.com

            %ARCH%
            x86_64

            %BUILDDATE%
            1234567890

            %INSTALLDATE%
            9876543210

            %PACKAGER%
            Some dude <dude@example.com>

            %SIZE%
            123456

            %REASON%
            1

            %LICENSE%
            Apache

            %VALIDATION%
            pgp

            %DEPENDS%
            gcc-libs
            glibc
            somelib=1.2.3
            some-other-lib.so=4.5.6
            linux-api-headers>=4.10

            %PROVIDES%
            libfoo.so=1.2.3
            "};

        let desc = Package::from_desc(input.as_bytes()).unwrap();

        assert_eq!(
            desc,
            Package {
                name: "library-subpackage".into(),
                version: "1.2.3-4".into(),
            }
        );
    }
}
