# filkoll

[ [lib.rs] ] [ [crates.io] ] [ [AUR] ] [ [AUR (git)] ]

Filkoll is:

* A tool to figure out what package owns a file, with fuzzy matching.
* A building block for a fast command-not-found handler for your shell.
* Swedish for "file check".

Currently this only supports Arch Linux.

This is inspired by [pkgfile](https://wiki.archlinux.org/index.php/Pkgfile),
but hyper-optimised for doing the "command-not-found" handler as fast as
possible.

## Installation

Install using the [AUR] package. At this point in time, this is the *only*
supported way to install.

Then to enable the nightly update job:

```shell
sudo systemctl enable --now filkoll-update.timer
```

To search you can use `filkoll binary <mycommand>`:

```console
❯ filkoll binary bash
core/bash 5.2.037-1                      /usr/bin/bash    (exact match)
core/bash 5.2.037-1                      /usr/bin/rbash
core/dash 0.5.12-1                       /usr/bin/dash
extra/avahi 1:0.8+r194+g3f79789-3        /usr/bin/bssh
extra/bass 1.2-14                        /usr/bin/bass
extra/reaver 1.6.6-1                     /usr/bin/wash
```

The results are sorted by how close the match is, with the closest results
at the top. See `filkoll --help`, `filkoll binary --help` etc for more
information on available flags.

To use `filkoll` as a command-not-found handler, you can add the following
to your shell configuration:

```shell
# Select one of these depending on which shell you use:
source /usr/share/doc/filkoll/command-not-found.bash
source /usr/share/doc/filkoll/command-not-found.fish
source /usr/share/doc/filkoll/command-not-found.zsh
```

## Benchmarks

filkoll is much faster than pkgfile (and this is using an unreleased version
of pkgfile that has some improvements).

```console
❯ hyperfine --warmup 5 -N "filkoll/target/release/filkoll binary uv" "pkgfile/build/pkgfile -b uv"
Benchmark 1: filkoll/target/release/filkoll binary uv
  Time (mean ± σ):       5.2 ms ±   1.6 ms    [User: 3.3 ms, System: 1.5 ms]
  Range (min … max):     2.9 ms …  10.1 ms    865 runs
 
Benchmark 2: pkgfile/build/pkgfile -b uv
  Time (mean ± σ):     239.0 ms ±  25.6 ms    [User: 1310.1 ms, System: 54.7 ms]
  Range (min … max):   217.2 ms … 309.5 ms    12 runs
 
Summary
  filkoll/target/release/filkoll binary uv ran
   46.32 ± 15.48 times faster than pkgfile/build/pkgfile -b uv
```

`filkoll` is fast enough that it will feel instant to the user in other words. `pkgfile` is not.

## MSRV (Minimum Supported Rust Version) policy

The MSRV may be bumped as needed. It is guaranteed that this program will at
least build on the current stable Rust release. An MSRV change is not considered
a breaking change and as such may change even in a patch version.

[crates.io]: https://crates.io/crates/filkoll
[lib.rs]: https://lib.rs/crates/filkoll
[AUR]: https://aur.archlinux.org/packages/filkoll
[AUR (git)]: https://aur.archlinux.org/packages/filkoll-git
