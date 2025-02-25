# filkoll

[ [lib.rs] ] [ [crates.io] ] [ [AUR] ] [ [AUR (git)] ]

Filkoll is:

* A tool to figure out what package owns a file, with fuzzy matching.
* A building block for a fast command-not-found handler for your shell.
* Swedish for "file check".

Currently this only supports Arch Linux.

This is inspired by [pkgfile](https://wiki.archlinux.org/index.php/Pkgfile), but hyper-optimised for
doing the "command-not-found" handler as fast as possible.

## MSRV (Minimum Supported Rust Version) policy

The MSRV may be bumped as needed. It is guaranteed that this program will at
least build on the current stable Rust release. An MSRV change is not considered
a breaking change and as such may change even in a patch version.

[crates.io]: https://crates.io/crates/filkoll
[lib.rs]: https://lib.rs/crates/filkoll
[AUR]: https://aur.archlinux.org/packages/filkoll
[AUR (git)]: https://aur.archlinux.org/packages/filkoll-git
