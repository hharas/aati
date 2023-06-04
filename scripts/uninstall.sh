#!/bin/bash

set -e

echo "+ Uninstalling Aati Packages..."
echo "+ Running (aati uninstall --all)..."

if ! aati uninstall --all; then
  echo "- Failed to uninstall Aati packages"
  exit 1
fi

echo "+ Deleting Aati's files..."
echo "+ Running (sudo rm -rf ~/.config/aati)..."

if ! sudo rm -rf ~/.config/aati; then
  echo "- Failed to delete Aati's files"
  exit 1
fi

echo "+ Deleting Aati's executable..."
echo "+ Running (sudo rm -f /usr/local/bin/aati)..."

if ! sudo rm -f /usr/local/bin/aati; then
  echo "- Failed to delete Aati's executable"
  exit 1
fi

echo "+ Alhamdulillah! Exiting..."
