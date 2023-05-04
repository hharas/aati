<p align="center">
    <img src="art/aati.png" alt="Aati Andalusian Calligraphy in ASCII" width="300" />
</p>

# The Aati Package Manager
Minimal package manager written in Rust

## How can I use it?

### As a user:

First off, install Aati by running the `./install.sh` script, or build Aati on your own.  
Aati relies on Aati Package Repositories (APRs), and so you need to add one in order to you use it. In order to set a repository, you need to run `aati repo add <repo url>`. You can add the [Amad Project Package Repository](https://codeberg.org/amad/repo/raw/branch/stable) if you want to try it out. Afterwards, if you run `aati list available`, you will see the available packages in the repo and their versions that you can install. If you wish to install the latest version of a package, you can run:

```bash
aati get <package name>
```

or:

```bash
aati get <name>-<version>
```

if you wish to install a specific version. If you wish to uninstall a package, you should run `aati remove <name>`. In order to stay in sync with the online repository, you should run `aati sync`, and in case a package got flagged as outdated, you should run `aati upgrade` or `aati upgrade <package>` to upgrade your packages.

### As a repository maintainer:

After installing aati, you must initialise an Aati Package Repository. You can do that by running:

```bash
aati repo init
```

After answering the prompts, you will be left with this tree structure:

```
aati_repo/
    repo.toml
    aarch64-unix/
        dummy-package/
            dummy-package-0.1.0.lz4
            dummy-package-0.1.1.lz4
    x86-64-unix/
        dummy-package/
            dummy-package-0.1.0.lz4
            dummy-package-0.1.1.lz4
```

- `aati_repo/`: your repository's folder, in it you can initialise a git repository and host it somewhere, like on Codeberg or Gitlab.
- `repo.toml`: the file that contains the data needed to be able to host this package repository.
- `x86-64-unix`: where amd64 Unix packages are located. Windows packages, for example, would be under a directory named `x86-64-windows`.
- `aarch64-unix`: where ARMv8 Unix packages are located.
- `dummy-package/`: a folder that contains LZ4 compressed packages of the default dummy package.

`repo.toml` is the most important file. It contains the following template at first:

```toml
[repo]
name = "<name>"
maintainer = "<maintainer>"
description = "<description>"

[index]
packages = [
    { name = "dummy-package", current = "0.1.1", arch = "aarch64", versions = [
        { tag = "0.1.0", checksum = "4237a71f63ef797e4bd5c70561ae85f68e66f84ae985704c14dd53fa9d81d7ac" },
        { tag = "0.1.1", checksum = "eda1b669d0bf90fdeb247a1e768a60baf56b9ba008a05c34859960be803d0ac4" },
    ], author = "<maintainer>", description = "Aati Dummy Package. This is a Package created as a template.", url = "https://codeberg.org/amad/aati" },
    { name = "dummy-package", current = "0.1.1", arch = "x86-64", versions = [
        { tag = "0.1.0", checksum = "ac5d6d9d495700c3f5880e89b34f56259a888b9ef671a76fc43410a1712acf95" },
        { tag = "0.1.1", checksum = "64cc0909fe1a2eaa2f7b211c1cf0250596d2c20b225c0c86507f01db9032913a" },
    ], author = "<maintainer>", description = "Aati Dummy Package. This is a Package created as a template.", url = "https://codeberg.org/amad/aati" }
]
```

Under `[repo]` there's general information about the Repository. Under `[index]` is where the `packages` array is located. The `packages` array contains an array that consists of the following package scheme:
```toml
{ name = "<package name>", current = "<package's current version>", target = "<target architecture>-<target family>", versions = [
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

# But why?

Simply: It was so fun to develop. I always thought of building my own package manager, so here it goes!

# Contribution

Any contributions to the Aati Package Manager are welcomed. This Project is definitely incomplete and immature, so any development is appreciated.

# License

Aati is licensed under the GNU General Public License V3.0.
