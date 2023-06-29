<p align="center">
    <img src="aati.png" alt="Aati Andalusian Calligraphy in ASCII" width="300" />
</p>

# The Aati Package Manager

Minimal package manager written in Rust

## How can I use it?

### As a user

First off, install Aati by running the `./install.sh` script, or build Aati on your own (if you wish to install Aati on Windows, see [this section](#installation-guide-for-windows-users)).  
Aati relies on Aati Package Repositories (APRs), and so you need to add one in order to you use it. In order to set a repository, you need to run `aati repo add <repo url>`. You can add the [Amad Project Package Repository](https://amad.codeberg.page) if you want to try it out. Afterwards, if you run `aati list available`, you will see the available packages in the repo and their versions that you can install. If you wish to install the latest version of a package, you can run:

```bash
aati get <package name>
```

or:

```bash
aati get <name>-<version>
```

if you wish to install a specific version. If you wish to uninstall a package, you should run `aati remove <name>`. In order to stay in sync with the online repository, you should run `aati sync`, and in case a package got flagged as outdated, you should run `aati upgrade` or `aati upgrade <package>` to upgrade your packages.

### As a repository maintainer

After installing aati, you must initialise an Aati Package Repository. You can do that by running:

```bash
aati repo init
```

After answering the prompts, you will be left with this tree structure:

```
aati_repo/
    repo.toml
    aarch64-linux/
        dummy-package/
            dummy-package-0.1.0.lz4
            dummy-package-0.1.1.lz4
    x86_64-linux/
        dummy-package/
            dummy-package-0.1.0.lz4
            dummy-package-0.1.1.lz4
```

- `aati_repo/`: your repository's directory, in it you can initialise a git repository and host it somewhere, like on Codeberg or GitLab.
- `repo.toml`: the file that contains the data needed to be able to host this package repository.
- `x86_64-linux`: where amd64 linux packages are located. Windows packages, for example, would be under a directory named `x86_64-windows`.
- `aarch64-linux`: where ARMv8 linux packages are located.
- `dummy-package/`: a directory that contains LZ4 compressed packages of the default dummy package.

`repo.toml` is the most important file. It contains the following template at first:

```toml
[repo]
name = "<name>"
maintainer = "<maintainer>"
description = "<description>"

[index]
packages = [
    { name = "dummy-package", current = "0.1.1", target = "aarch64-linux", versions = [
        { tag = "0.1.0", checksum = "fd54f3db9f9b001d836654dec8b50a3f76f9003e5b86afc9fb0e2ef42c98a935" },
        { tag = "0.1.1", checksum = "41a5dbe93c5641969374a2c369d486168d28fa6e5049730770f72a64c83afd61" },
    ], author = "<maintainer>", description = "Aati Dummy Package. This is a Package created as a template.", url = "https://codeberg.org/amad/aati" },
    { name = "dummy-package", current = "0.1.1", target = "x86_64-linux", versions = [
        { tag = "0.1.0", checksum = "11b3cb26f62469bd04ce1175e9593ae9d1a02920c4e3bd69f3ac4fbde6dc856f" },
        { tag = "0.1.1", checksum = "c8e6b84c85602b774c15c1efefdd9be11c739d73f541f3a92193cf10054a11a0" },
    ], author = "<maintainer>", description = "Aati Dummy Package. This is a Package created as a template.", url = "https://codeberg.org/amad/aati" }
]
```

Under `[repo]` there's general information about the Repository. Under `[index]` is where the `packages` array is located. The `packages` array contains an array that consists of the following package scheme:

```toml
{ name = "<package name>", current = "<package's current version>", target = "<target architecture>-<target os>", versions = [
    { tag = "version's tag", checksum = "file's sha256sum" }
], author = "<package author>", description = "<package description>", url = "<package url>" }
```

In order to add a package, you need to perform two actions:

1. Add your Package to the file structure by creating a directory under `aati_repo/<target>/` named after your package's name. In it you will be putting your binaries in the following format: `package-name-x.x.x`. Afterwards you will run `aati package package-name-x.x.x` in order to compress your binary using LZ4, then delete the original binary.

2. Add your package to the `repo.toml` file. Although this step will be replaced by adding a main `index.toml` file for this job, however this hasn't been implemented yet. In order to add it to the `repo.toml` file, you need to generate a sha256 hash of your LZ4 compressed package, then add that as a package entry to the `packages` array. If you're adding a new version to your program, then add it to the `versions` array inside your package's object.

If you want to test things out only, then do the following:

1. `$ aati repo init`
2. `$ cd aati_repo`
3. `$ python -m http.server`
4. `$ aati repo add http://localhost:8000`
5. Now you're all set. Try installing the dummy package.
6. `$ aati get dummy-package`
7. `$ dummy-package`

Here you go! You set up a local Aati repository that you can use to test things out.

### Installation Guide for Windows Users

Since Batch Code and PowerShell Scripts suck (I had a serious struggle writing installation scripts using them) I decided to write an Installation Guide myself, so here we go!

1. Clone the Repository & cd into it:

```batch
git clone https://codeberg.org/amad/aati.git && cd aati
```

2. Build Aati from source with the `--release` profile:

```batch
cargo build --release
```

3. Make a Directory named `Aati\` under `C:\Program Files\`, another directory named `Binaries\` under `Aati\` and copy the released executable into it:

```batch
mkdir "C:\Program Files\Aati" && mkdir "C:\Program Files\Aati\Binaries" && copy target/release/aati.exe "C:\Program Files\Aati\Binaries\aati.exe"
```

4. Add `C:\Program Files\Aati\Binaries` to `%PATH%`, since that's where all of your installed packages (including `aati.exe` itself) will be located.

5. Open your Terminal as Administrator and run any Aati command you wish.

# But why?

Simply: It was so fun to develop. I always thought of building my own package manager, so here it goes!

# Contribution

Any contributions to the Aati Package Manager are welcomed. This Project is definitely incomplete and immature, so any development is appreciated.

# License

Aati is licensed under the GNU General Public License V3.0.
