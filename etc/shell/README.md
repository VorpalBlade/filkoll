# Shell hooks

This directory contains shell integration for `filkoll` to act as the
command-not-found handler. Source this into your shell to use it. E.g.
if these are installed in the default path for the AUR package:

```sh
# zsh
source /usr/doc/filkoll/command-not-found.zsh

# bash
source /usr/doc/filkoll/command-not-found.bash

# fish
source /usr/doc/filkoll/command-not-found.fish
```

## Credits

These scripts are *heavily* based on [pkgfile](https://github.com/falconindy/pkgfile).
