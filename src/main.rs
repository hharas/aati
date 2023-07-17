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

#![allow(unused)]

use clap::{Arg, ArgAction, Command};
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
                ])
                .args([
                    Arg::new("list")
                        .short('l')
                        .long("list")
                        .help("List added repositories")
                        .conflicts_with("init")
                        .action(ArgAction::SetTrue),
                    Arg::new("init")
                        .short('n')
                        .long("init")
                        .conflicts_with("list")
                        .help("Initialise a new repository")
                        .action(ArgAction::SetTrue),
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
            let packages_vec: Vec<String> = packages.map(|s| s.to_owned()).collect::<Vec<_>>();
            commands::get(&packages_vec);
        }
        Some(("install", install_matches)) => {
            let package = install_matches.get_one::<String>("package").unwrap();
            commands::install(package);
        }
        Some(("upgrade", upgrade_matches)) => {
            if let Some(packages) = upgrade_matches.get_many::<String>("packages") {
                let packages_vec: Vec<&str> = packages.map(|s| s.as_str()).collect::<Vec<_>>();
                commands::upgrade(Some(packages_vec));
            } else {
                commands::upgrade(None);
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
            commands::list(list_matches.get_flag("available"));
        }
        Some(("sync", sync_matches)) => {}
        Some(("repo", repo_matches)) => {}
        Some(("query", query_matches)) => {}
        Some(("changelog", changelog_matches)) => {}
        Some(("package", package_matches)) => {}
        Some(("generate", generate_matches)) => {}
        Some(("serve", serve_matches)) => {}

        _ => unreachable!(),
    }
}

// تم بحمد الله
