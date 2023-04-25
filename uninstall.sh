#!/bin/bash

set -e

echo "+ Uninstalling Aati..."
echo "+ Running (# rm /usr/local/bin/aati)"
sudo rm /usr/local/bin/aati

if [ $? -ne 0 ]; then
  echo "- Failed to uninstall Aati from /usr/local/bin/"
  exit 1
fi

echo "+ Alhamdulillah! Exiting..."
