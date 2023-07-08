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

Cross-platform package manager written in Rust

## Installation

For Linux users, just install the Rust Programming Language on your System, then use cargo to directly install Aati by running `cargo install aati`. If you're on Termux, run `pkg install rust` and then do the same thing. For Windows users, see [this dedicated Section](#installation-guide-for-windows-users).

## How can I use it?

### As a user

Aati relies on Aati Package Repositories (APRs), and so you need to add one in order to use it. In order to add a repository, you need to run `aati repo add <repo url>`. You can add the [Amad Project Package Repository](https://amad.codeberg.page) if you want to try Aati out. Afterwards, if you run `aati list available`, you will see the available packages in the repo and their versions that you can install. If you wish to install the latest version of a package, you can run:

```bash
aati get <package name>
```

or:

```bash
aati get <name>-<version>
```

if you wish to install a specific version. If you wish to uninstall a package, you can run `aati remove <name>`. In order to stay in sync with the online repository, you can run `aati sync`, and in case a package got flagged as outdated, you can run `aati upgrade` or `aati upgrade <package>` to upgrade your package/s.

### As a repository maintainer

After installing aati, you can initialise an Aati Package Repository. You can do that by running:

```bash
aati repo init
```

After answering the prompts, you'll be left with this tree structure:

```
aati_repo/
    repo.toml
    aarch64-linux/
    x86_64-linux/
    x86_64-windows/
```

- `aati_repo/`: is your repository's directory, in it you can initialise a git repository and host it somewhere, like on Codeberg or GitLab.
- `repo.toml`: is the file that contains the data needed to be able to host this package repository.
- `x86_64-linux`: is where amd64 linux packages are located.
- `x86_64-windows`: is where amd64 windows packages are located.
- `aarch64-linux`: is where ARMv8 linux packages are located.

`repo.toml` is the most important file. It contains the following template at first:

```toml
[repo]
name = "<name>"
maintainer = "<maintainer>"
description = "<description>"

[index]
packages = [
#   { name = "package-name-here", current = "0.1.1", target = "x86_64-linux", versions = [
#       { tag = "0.1.0", checksum = "sha256-sum-here" },
#       { tag = "0.1.1", checksum = "sha256-sum-here" },
#   ], author = "<maintainer>", description = "Package description here.", url = "https://github.com/hharas/aati" },
]
```

Under `[repo]` there's general information about the Repository. Under `[index]` is where the `packages` array is located. The `packages` array contains an array that consists of the following package scheme:

```toml
{ name = "<package name>", current = "<package's current version>", target = "<target architecture>-<target os>", versions = [
    { tag = "version's tag", checksum = "file's sha256sum" }
], author = "<package author>", description = "<package description>", url = "<package url>" }
```

In order to add a package, you need to perform two actions:

1. Add your Package to the file structure by creating a directory under `aati_repo/<target>/` named after your package's name. In it you will be making a folder named in the following format: `package-name-x.x.x`. In it you will make a PKGFILE that tells Aati how to install and uninstall your package. See the PKGFILE Manual [here](#pkgfile-manual). Afterwards you will run `aati package package-name-x.x.x` in order to archive your folder into a tarball then compress it using LZ4.

2. Add your package to the `repo.toml` file. In order to add it to the `repo.toml` file, you need to generate a SHA256 hash of your LZ4 compressed package, then add that as a package entry to the `packages` array. If you're adding a new version to your program, then add it to the `versions` array inside your package's object and update the `current` field to the latest version.

### Installation Guide for Windows Users

Since Batch Code and PowerShell Scripts suck (I had a serious struggle writing installation scripts using them) I decided to write an Installation Guide myself, so here we go!

1. Clone the Repository & cd into it:

```batch
git clone https://github.com/hharas/aati.git && cd aati
```

2. Build Aati from source with the `--release` profile:

```batch
cargo build --release
```

3. Make a Directory named `Aati\` under your user home directory, and under it another directory named `Binaries\` and then copy the released executable into it:

```batch
mkdir "C:\Users\<username>\Aati" && mkdir "C:\Users\<username>\Aati\Binaries" && copy target\release\aati.exe "C:\Users\<username>\Aati\Binaries\aati.exe"
```

4. Add `C:\Users\<username>\Aati\Binaries` to your user-specific `%PATH%` system variable, since that's where all of your installed packages (including `aati.exe` itself) will be located.

5. Open your Terminal and run any Aati command you wish.

### PKGFILE Manual

The PKGFILE is the essence of Aati's packaging system, as it describes to Aati how to install and uninstall software through it. The PKGFILE is separated into two TOML-like looking fields, `[installation]` and `[removal]`.

```
[installation]
...

[removal]
...
```

When Aati installs a package, it looks for the commands under the `[installation]` field. Commands under the `[installation]` field are ran knowing the current working directory, that is somewhere under the operating system's temporary directory, unlike commands under the `[removal]` field, which are ran without context at all. We'll get to this later in more detail.

Now, you may be wondering: "What are those commands that Aati can execute from the PKGFILE?"  
Well, those commands currently are:
- `install`: copies a file to a destination and (if it's a UNIX operating system) makes it an executable, e.g. `install program $bin_dir/program`
- `copy`: copies a file to a destination only, e.g. `copy lib.so $lib_dir/lib.so`
- `system`: invokes a system command, `sh -c` on UNIX and `cmd /C` on Windows, e.g. `system wget https://picsum.photos/400 -O image.jpeg` will be
- `delete`: deletes a file from the system, e.g. `delete $bin_dir/program`

The PKGFILE also recognises global variables, which are currently limited to:
- `$bin_dir`: directory for binary executables, `~/.local/bin` for UNIX and `C:\Users\<username>\Aati\Binaries` for Windows.
- `$lib_dir`: directory for DLLs, `~/.local/lib` for UNIX and `C:\Users\<username>\Aati\Binaries` for Windows.
- `$home_dir`: user home directory, `~`/`$HOME` for UNIX and `C:\Users\<username>` for Windows.

An Example of a PKGFILE summing up all of what we said above can be:
```bash
[installation]
# Aati executes this section first while acknowledging the current
# working directory, this is why we can easily refer to files
# here using relative paths (i.e. application, library.so, image.jpeg)
install application $bin_dir/application
copy library.so $lib_dir/lib.so
system wget https://picsum.photos/400 -O image.jpeg
copy image.jpeg $home_dir/image.jpeg

[removal]
# Here there's no current working directory involved
# so all the commands in this section can't make use of it
delete $bin_dir/application
delete $lib_dir/lib.so
delete $home_dir/image.jpeg
```

# But why?

Simply: It was so fun to develop. I always thought of building my own package manager, so here it goes!

# Contribution

Any contributions to Aati are welcomed. This Project is definitely incomplete and immature, so any development is appreciated.

# License

The Aati Package Manager is licensed under the GNU General Public License V3.0.
