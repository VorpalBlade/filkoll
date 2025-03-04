#!/usr/bin/env bash

set -ex

rm -rf /test_dir/actual
mkdir /test_dir/actual

compare() {
    local ret=0
    # Remove timestamps
    sed -e 's/^20[^ ]* *//' -e 's/20[-0-9]*[T ][0-9:.]*//' -i /test_dir/actual/${1}_output.txt
    # Compare
    echo "--- Diff of ${1}_output.txt ---"
    if ! diff -Naur /test_dir/expected/${1}_output.txt /test_dir/actual/${1}_output.txt; then
        ret=1
    fi
    return $ret
}

exit_code=0

echo "1: Update"
sudo RUST_LOG=warn "$(type -p filkoll)" update --no-download 2>&1 | tee /test_dir/actual/1_output.txt
compare 1 || exit_code=1

echo "2: Searches"
searches() {
    filkoll binary -e 0 bash
    filkoll binary -e 2 bash
    filkoll binary -ne 1 zsh
    filkoll binary -e 2 no_such_binary_exists
}
searches  2>&1 | tee /test_dir/actual/2_output.txt
compare 2 || exit_code=1

exit $exit_code
