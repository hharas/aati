#!/bin/bash

set -e

echo "+ Checking for repository updates..."
echo "+ Running (git pull origin master --no-rebase)"
git pull origin master --no-rebase

echo "+ Building Aati..."
echo "+ Running (cargo build --release)..."
cargo build --release

if [ $? -ne 0 ]; then
  echo "- Failed to build Aati."
  exit 1
fi

echo "+ Building finished!"

# Uncomment this if you want a lighter executable
# if command -v upx &> /dev/null; then
#   echo "+ Compressing Aati's Executable..."
#   echo "+ Running ($ upx --best --lzma target/release/aati)..."
#   upx --best --lzma target/release/aati
# fi

echo "+ Copying Aati to the /usr/local/bin/ directory..."
echo "+ Running (cp ./target/release/aati /usr/local/bin/aati)..."
sudo cp ./target/release/aati /usr/local/bin/aati

if [ $? -ne 0 ]; then
  echo "- Failed to copy Aati to /usr/local/bin/."
  exit 1
fi

echo "+ Done copying!"

echo "+ Alhamdulillah! Exiting... "
