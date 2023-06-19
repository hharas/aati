#!/bin/bash

set -xe

aati uninstall --all

sudo rm -rf ~/.config/aati

sudo rm -f /usr/local/bin/aati
