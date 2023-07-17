/* بسم الله الرحمن الرحيم

   Aati - Cross-platform Package Manager written in Rust.
   Copyright (C) 2023  Husayn Haras <husayn@dnmx.org>

   This program is free software: you can redistribute it and/or modify
   it under the terms of version 3 of the GNU General Public License
   as published by the Free Software Foundation.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use clap::{Arg, ArgAction, Command};
use commands::{changelog, generate, get, install, list, package, query, repo, sync, upgrade};
use version::get_version;

mod commands;
mod globals;
mod types;
mod utils;
mod version;

fn main() {
    let name: &str = "aati";
    let version = get_version();
    let author: &str = "Husayn Haras <husayn@dnmx.org>";
    let about: &str = "Cross-platform package manger written in Rust";
    let after_help: &str = "Copyright (C) 2023  Husayn Haras <husayn@dnmx.org>

This program is free software: you can redistribute it and/or modify
it under the terms of version 3 of the GNU General Public License
as published by the Free Software Foundation.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

User Guide: https://github.com/hharas/aati/wiki/2.-User-Guide
Issue tracker: https://github.com/hharas/aati/issues";

    let matches = Command::new(name)
        .author(author)
        .version(version)
        .about(about)
        .after_help(after_help)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            Command::new("get")
                .short_flag('G')
                .about("Download and install packages from an available repository")
                .arg(
                    Arg::new("packages")
                        .help("package/s to get")
                        .action(ArgAction::Set)
                        .required(true)
                        .num_args(1..),
                ),
            Command::new("install")
                .short_flag('I')
                .about("Install a package from a local .tar.lz4 archive")
                .arg(
                    Arg::new("package")
                        .help("package .tar.lz4 file")
                        .action(ArgAction::Set)
                        .required(true)
                        .num_args(1),
                ),
            Command::new("upgrade")
                .visible_alias("update")
                .short_flag('U')
                .about("Upgrade local packages to their latest versions")
                .arg(
                    Arg::new("packages")
                        .help("package/s to upgrade")
                        .action(ArgAction::Set)
                        .num_args(1..),
                ),
            Command::new("remove")
                .visible_alias("uninstall")
                .short_flag('R')
                .about("Remove an installed package")
                .args([
                    Arg::new("packages")
                        .required(true)
                        .action(ArgAction::Set)
                        .help("package/s to remove")
                        .num_args(1..),
                    Arg::new("all")
                        .long("all")
                        .short('a')
                        .action(ArgAction::SetTrue)
                        .conflicts_with("packages")
                        .help("Remove all packages"),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help("Apply forceful removal"),
                ]),
            Command::new("list")
                .short_flag('L')
                .about("List installed packages")
                .arg(
                    Arg::new("available")
                        .long("available")
                        .short('a')
                        .help("list available packages")
                        .action(ArgAction::SetTrue),
                ),
            Command::new("sync")
                .short_flag('S')
                .about("Sync repository manifests"),
            Command::new("repo")
                .short_flag('P')
                .about("Manage repositories")
                .subcommand_required(true)
                .subcommands([
                    Command::new("add")
                        .short_flag('a')
                        .about("Add a repository")
                        .arg(
                            Arg::new("url")
                                .help("Repository URL")
                                .action(ArgAction::Set)
                                .required(true)
                                .num_args(1),
                        ),
                    Command::new("remove")
                        .short_flag('r')
                        .about("Remove a repository")
                        .arg(
                            Arg::new("name")
                                .help("Repository name")
                                .action(ArgAction::Set)
                                .required(true)
                                .num_args(1),
                        ),
                    Command::new("info")
                        .short_flag('i')
                        .about("Show a repository's metadata")
                        .arg(
                            Arg::new("name")
                                .help("Repository name")
                                .action(ArgAction::Set)
                                .required(true)
                                .num_args(1),
                        ),
                    Command::new("list")
                        .short_flag('l')
                        .about("List added repositories"),
                    Command::new("init")
                        .short_flag('n')
                        .about("Initialise a new repository")
                        .args([
                            Arg::new("name")
                                .long("name")
                                .short('n')
                                .help("Repository name")
                                .action(ArgAction::Set)
                                .required(true)
                                .num_args(1),
                            Arg::new("maintainer")
                                .long("maintainer")
                                .short('m')
                                .help("Repository maintainer's name")
                                .action(ArgAction::Set)
                                .required(true)
                                .num_args(1),
                            Arg::new("description")
                                .long("description")
                                .short('d')
                                .help("Repository description")
                                .action(ArgAction::Set)
                                .required(true)
                                .num_args(1),
                        ]),
                ]),
            Command::new("query")
                .short_flag('Q')
                .about("Query a package's metadata")
                .arg(
                    Arg::new("package")
                        .help("selected package to query")
                        .required(true)
                        .action(ArgAction::Set)
                        .num_args(1),
                ),
            Command::new("changelog")
                .short_flag('C')
                .about("Display a package's changelog")
                .args([
                    Arg::new("package")
                        .help("selected package")
                        .action(ArgAction::Set)
                        .num_args(1),
                    Arg::new("latest")
                        .short('l')
                        .long("latest")
                        .help("show only the latest changes")
                        .action(ArgAction::SetTrue),
                ]),
            Command::new("package")
                .short_flag('K')
                .about("Compress a directory into a .tar.lz4 package archive")
                .arg(
                    Arg::new("directory")
                        .help("path to package directory")
                        .action(ArgAction::Set)
                        .required(true)
                        .num_args(1),
                ),
            Command::new("generate")
                .short_flag('N')
                .about("Generate HTML files for a repository"),
            Command::new("serve")
                .short_flag('E')
                .about("Serve a package web index using a repo.toml")
                .args([
                    Arg::new("host")
                        .long("host")
                        .short('s')
                        .required(true)
                        .action(ArgAction::Set)
                        .help("server host"),
                    Arg::new("port")
                        .long("port")
                        .short('p')
                        .default_value("8887")
                        .action(ArgAction::Set)
                        .help("server port"),
                    Arg::new("repository")
                        .long("repo")
                        .short('r')
                        .required(true)
                        .action(ArgAction::Set)
                        .help("repository url"),
                    Arg::new("url")
                        .long("url")
                        .short('u')
                        .required(true)
                        .action(ArgAction::Set)
                        .help("server url (e.g. http://example.com)"),
                ]),
        ])
        .get_matches();

    match matches.subcommand() {
        Some(("get", get_matches)) => {
            let packages = get_matches.get_many::<String>("packages").unwrap();
            let packages_vec: Vec<String> = packages.map(|s| s.into()).collect::<Vec<_>>();
            for package in packages_vec {
                get::command(&package);
            }
        }
        Some(("install", install_matches)) => {
            let package = install_matches.get_one::<String>("package").unwrap();
            install::command(package);
        }
        Some(("upgrade", upgrade_matches)) => {
            if let Some(packages) = upgrade_matches.get_many::<String>("packages") {
                let packages_vec: Vec<&str> = packages.map(|s| s.as_str()).collect::<Vec<_>>();
                for package in packages_vec {
                    upgrade::command(Some(package));
                }
            } else {
                upgrade::command(None);
            }
        }
        Some(("remove", remove_matches)) => {
            let is_forced = remove_matches.get_flag("force");

            if remove_matches.get_flag("all") {
                commands::remove(None, is_forced);
            } else {
                let packages = remove_matches.get_many::<String>("packages").unwrap();
                let packages_vec: Vec<String> = packages.map(|s| s.to_owned()).collect::<Vec<_>>();

                commands::remove(Some(packages_vec), is_forced)
            }
        }
        Some(("list", list_matches)) => {
            if list_matches.get_flag("available") {
                list::available()
            } else {
                list::installed();
            }
        }
        Some(("sync", _)) => {
            sync::command();
        }
        Some(("repo", repo_matches)) => match repo_matches.subcommand() {
            Some(("add", add_matches)) => {
                let repository_url = add_matches.get_one::<String>("url").unwrap();
                repo::add(repository_url.into());
            }

            Some(("remove", remove_matches)) => {
                let repo_name = remove_matches.get_one::<String>("name").unwrap();
                repo::remove(repo_name.into());
            }

            Some(("info", info_matches)) => {
                let repository_url = info_matches.get_one::<String>("name").unwrap();
                repo::info(repository_url.into());
            }

            Some(("list", _)) => {
                repo::list();
            }

            Some(("init", init_matches)) => {
                let repo_name = init_matches.get_one::<String>("name").unwrap();
                let repo_maintainer = init_matches.get_one::<String>("maintainer").unwrap();
                let repo_description = init_matches.get_one::<String>("description").unwrap();

                repo::init(
                    repo_name.into(),
                    repo_maintainer.into(),
                    repo_description.into(),
                );
            }

            _ => unreachable!(),
        },
        Some(("query", query_matches)) => {
            let package_name = query_matches.get_one::<String>("package").unwrap();
            query::command(package_name, None);
        }
        Some(("changelog", changelog_matches)) => {
            let latest_only = changelog_matches.get_flag("latest");

            if let Some(package_name) = changelog_matches.get_one::<String>("package") {
                changelog(Some(package_name), latest_only);
            } else {
                changelog(None, latest_only);
            }
        }
        Some(("package", package_matches)) => {
            let directory_name = package_matches.get_one::<String>("directory").unwrap();
            package::command(directory_name.into());
        }
        Some(("generate", _)) => {
            generate::command();
        }
        Some(("serve", serve_matches)) => {}

        _ => unreachable!(),
    }
}

// تم بحمد الله
