# The Aati Package Manager
Small package manager written in Rust

## How can I use it?

### As a user:

First off, install Aati by running the `./install.sh` script, or build Aati on your own.

Now, you need to set a default package repository. Although I'm willing to add support for multiple repositories, Aati currently supports single repositories only, and might act weird when you switch repositories.

In order to set a repository, you need to run `aati repo <repo url>`. You can add the [Amad Project Package Repository](https://codeberg.org/amad/repo/raw/branch/main) if you want to try it out. Afterwards, if you run `aati list available`, you will see the available packages in the repo and their versions that you can install. If you wish to install the latest version of a package, you can run:

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

After answering the prompts, you will be left with this folder structure:

```
aati_repo/
    repo.toml
    dummy-package/
        dummy-package-0.1.0.lz4
        dummy-package-0.1.1.lz4
```

- `aati_repo/`: your repository's folder, in it you can initialise a git repository and host it somewhere, like on Codeberg or Gitlab.
- `repo.toml`: the file that contains the data needed to be able to host this package repository.
- `dummy-package/`: a folder that contains LZ4 compressed packages of the default dummy package.

`repo.toml` is the most importamt file. It contains the following template at first:

```toml
[repo]
url = "<url>"
maintainer = "<maintainer>"
description = "<description>"

[index]
packages = [
    { name = "dummy-package", current = "0.1.1", versions = [
        { tag = "0.1.0", checksum = "a7d9edd360059777ee8e0d80a6dcf64299b41d7e6a5f720833fcf2bcd7105604" },
        { tag = "0.1.1", checksum = "c92d96486db47fc9d4a9fdef0da454afb5a434a933801e0b603269d17f0ad64d" },
    ], author = "<maintainer>", description = "Aati Dummy Package. This is a Package created as a template.", url = "https://codeberg.org/amad/aati" }
]
```

Under `[repo]` there's general information about the Repository. What matters most there is the `url` field, since that's what Aati will look for when syncing with your Repository. The `url` field should be a url that doesn't end with a `/` and should be hosting the `aati_repo` directory. If you're hosting this Repository as a Git Repository on Codeberg, the URL should be `https://codeberg.org/<username>/<repo>/raw/branch/<branch>`.

Under `[index]` is where the `packages` array is located. The `packages` array contains an array that consists of the following package scheme:
```toml
{ name = "<package name>", current = "<package's current version>", versions = [
    { tag = "version's tag", checksum = "file's sha256sum" }
], author = "<package author>", description = "<package description>", url = "<package url>" }
```

In order to add a package, you need to perform two actions:
1. Add your Package to the file structure by creating a directory under `aati_repo/` named after your package's name. In it you will be putting your binaries in the following format: `package-name-x.x.x`. Afterwards you will run `aati package package-name-x.x.x` in order to compress your binary using LZ4, then delete the original binary.

2. Add your package to the `repo.toml` file. Although this step will be replaced by adding a main `index.toml` file for this job, however this hasn't been implemented yet. In order to add it to the `repo.toml` file, you need to generate a sha256 hash of your LZ4 compressed package, then add that as a package entry to the `packages` array. If you're adding a new version to your program, then add it to the `versions` array inside your package's object.

If you want to test things out only, then do the following:
1. `$ aati repo init`
2. Make sure to set `http://localhost:8000` as the repository's url.
3. `$ cd aati_repo`
4. `$ python -m http.server`
5. `$ aati repo http://localhost:8000`
6. Now you're all set. Try installing the dummy package.
7. `$ aati get dummy-package`
8. `$ dummy-package`

Here you go! You set up a local Aati repository that you can use to test things out.

# But why?

Simply: It was so fun to develop. I always thought of building my own package manager, so here it goes!

# Contribution

Any contribution to the Aati Package Manager are welcomed. This Project is definitely incomplete and immature, so any development is appreciated.

# License

Aati is licensed under the GNU General Public License V3.0.