#!/usr/bin/env bash

set -ex

test_dir="$1"
bin_dir="$2"

podman run --rm \
    --mount type=bind,src=$test_dir,target=/test_dir \
    --mount type=bind,src=$bin_dir,target=/bin_dir \
    filkoll_test_img /test_runner_inner.sh
