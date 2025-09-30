# Changelog

All notable changes to this project will be documented in this file.
Keep in mind that this is only updated when releases are made and the file
is generated automatically from commit messages (and may or may not be lightly
edited).

For a possibly more edited message focused on the binary please see the github
releases.

## [0.2.8] - 2025-09-30

### ⚙️ Other stuff

- Update Cargo.toml dependencies


## [0.2.7] - 2025-09-14

### 🐛 Bug fixes

- Fix incorrect suggestion

### 🩺 Diagnostics & output formatting

- Improve message for no exact match

### ⚙️ Other stuff

- Update to Rust 1.88.0

## [0.2.6] - 2025-06-02

### 🩺 Diagnostics & output formatting

- Improve error report on cache file version mismatch

## [0.2.5] - 2025-05-17

### ⚙️ Other stuff

- Update Cargo.toml dependencies

## [0.2.4] - 2025-04-18

### 🚀 Features

- Support for xz2 compression

## [0.2.3] - 2025-03-26

### 🚀 Features

- Add support for zstd

## [0.2.2] - 2025-03-25

### 🐛 Bug fixes

- Create cache directory if missing

### 🩺 Diagnostics & output formatting

- Reduce amount of debug output from update command

### 🧪 Testing

- Add more unit tests

### ⚙️ Other stuff

- *(docs)* Typo fixes
- Add missing invariant

## [0.2.1] - 2025-02-28

### ⚡ Performance improvements

- Delay UTF-8 until after filtering for PATH when building cache

### 🚜 Refactoring

- Remove unused code

## [0.2.0] - 2025-02-25

### 🚀 Features

- Allow skipping fuzzy matches if exact matches are found
- Add AUR packages
- Add systemd timer
- Switch to zlib-rs (rather than zlib-ng): This has comparable performance,
  but is simpler to build, since it doesn't involve C code.

### 🐛 Bug Fixes

- Use proper `PATH` in systemd service by using a login shell

### 📚 Documentation

- Massively improve README
- Add a CHANGELOG!
