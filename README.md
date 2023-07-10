<p align="center">
    <img src="aati.png" alt="Aati Andalusian Calligraphy in ASCII" width="300" />
</p>

# The Aati Package Manager

[![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/hharas/aati/rust.yml?logo=github)](https://github.com/hharas/aati/actions/workflows/rust.yml)
[![GitHub last commit (branch)](https://img.shields.io/github/last-commit/hharas/aati/master)](https://github.com/hharas/aati/commit/HEAD)
[![GitHub issues](https://img.shields.io/github/issues/hharas/aati)](https://github.com/hharas/aati/issues)
[![GitHub top language](https://img.shields.io/github/languages/top/hharas/aati?logo=rust)](https://github.com/hharas/aati/search?l=rust)
[![Libraries.io dependency status for latest release](https://img.shields.io/librariesio/release/cargo/aati)](https://libraries.io/cargo/aati)
[![Crates.io](https://img.shields.io/crates/v/aati)](https://crates.io/crates/aati)
[![GitHub](https://img.shields.io/github/license/hharas/aati?logo=gnu)](https://www.gnu.org/licenses/gpl-3.0.en.html)

**IMPORTANT NOTE: THE AATI PACKAGE MANAGER IS AN INCOMPLETE PROJECT! IT CAN HAVE BREAKING CHANGES AT ANY MOMENT WITHOUT NOTICES! USE AATI AT YOUR OWN RISK!**

Aati is a Cross-platform Package Manager written in Rust. The Aati Package Manager focuses on providing a Simple, Efficient and Performant interface for installing, managing and removing packages. Aati is not made (but can be forked) to be a system-wide package manager like Pacman or APT but rather a user-specific package manager. Aati supports multiple operating systems including Linux, Windows, Android (on Termux) and more.

Aati also has its own Packaging System revolving around [PKGFILEs](https://github.com/hharas/aati/wiki/4.-PKGFILE-Manual), that are files describing the installation and removal process of packages in a custom TOML-like language, which is a concept somewhat similar to the [Arch Build System's PKGBUILDs](https://wiki.archlinux.org/title/PKGBUILD). The Aati Package Manager recognises files ending with `.tar.lz4` as package distributions that can be installed, either from an online package repository or from the local filesystem.

Read the [Wiki](https://github.com/hharas/aati/wiki) for more Information how to install Aati, use it, package applications for it, etc.

# But why?

Simply: It was so fun to develop. I always thought of building my own package manager, so here it goes!

# Contribution

Any contributions to Aati are welcomed. This Project is definitely incomplete and immature, so any development is appreciated.

# License

The Aati Package Manager is licensed under the GNU General Public License V3.0.
