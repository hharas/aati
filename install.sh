#!/bin/bash

set -xe

cargo build --release

# Uncomment this if you want a lighter executable
# if command -v upx &> /dev/null; then
#   upx --best --lzma target/release/aati
# fi

sudo cp target/release/aati /usr/local/bin/aati
