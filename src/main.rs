/* بسم الله الرحمن الرحيم

   Aati - Cross-platform Package Manager written in Rust.
   Copyright (C) 2023  Husayn Haras <haras@disroot.org>

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

use colored::Colorize;
use config::{ISSUE_TRACKER_URL, USER_GUIDE_URL};
use std::io::stdout;

use clap::{Arg, ArgAction, Command, ValueHint};
use clap_complete::Shell;
use commands::{
    changelog, generate, get, install, list, package, query, repo, serve, sync, upgrade,
};
use utils::get_target;
use version::get_version;

mod commands;
mod config;
mod types;
mod utils;
mod version;

fn main() {
    let after_help = format!(
        "User Guide: {}
Issue tracker: {}",
        USER_GUIDE_URL, ISSUE_TRACKER_URL
    );

    let long_version = format!(
        "{} ({})
Copyright (C) 2023  Husayn Haras <haras@disroot.org>

This program is free software: you can redistribute it and/or modify
it under the terms of version 3 of the GNU General Public License
as published by the Free Software Foundation.",
        get_version(),
        get_target()
    );

    let mut cli = Command::new("aati")
        .about("Cross-platform package manger written in Rust")
        .long_version(long_version)
        .after_help(after_help)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            Command::new("get")
                .short_flag('G')
                .about("Download and install packages from an available repository")
                .args([
                    Arg::new("packages")
                        .help("Package(s) to get")
                        .action(ArgAction::Set)
                        .required(true)
                        .num_args(1..),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help("Agree to all prompts"),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("Show the least output possible"),
                ]),
            Command::new("install")
                .short_flag('I')
                .about("Install a package from the local filesystem")
                .args([
                    Arg::new("package")
                        .help("Package .tar.lz4 filepath")
                        .action(ArgAction::Set)
                        .required_unless_present("pkgfile")
                        .conflicts_with("pkgfile")
                        .num_args(1)
                        .value_hint(ValueHint::FilePath),
                    Arg::new("pkgfile")
                        .long("use-pkgfile")
                        .short('p')
                        .help("Path to a pkgfile")
                        .action(ArgAction::Set)
                        .required_unless_present("package")
                        .conflicts_with("package")
                        .num_args(1)
                        .value_hint(ValueHint::FilePath),
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("Name of pkgfile-installed package")
                        .action(ArgAction::Set)
                        .conflicts_with("package")
                        .num_args(1),
                    Arg::new("version")
                        .long("version")
                        .short('v')
                        .help("Version of pkgfile-installed package")
                        .action(ArgAction::Set)
                        .conflicts_with("package")
                        .num_args(1),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help("Agree to all prompts"),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("Show the least output possible"),
                ]),
            Command::new("upgrade")
                .visible_alias("update")
                .short_flag('U')
                .about("Upgrade local packages to their latest versions")
                .args([
                    Arg::new("packages")
                        .help("Package(s) to upgrade")
                        .action(ArgAction::Set)
                        .num_args(1..),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help("Agree to all prompts"),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("Show the least output possible"),
                ]),
            Command::new("remove")
                .visible_alias("uninstall")
                .short_flag('R')
                .about("Remove an installed package")
                .args([
                    Arg::new("packages")
                        .required(true)
                        .action(ArgAction::Set)
                        .help("Package(s) to remove")
                        .num_args(1..),
                    Arg::new("all")
                        .long("all")
                        .short('a')
                        .action(ArgAction::SetTrue)
                        .conflicts_with("packages")
                        .help("Remove all packages"),
                    Arg::new("lock")
                        .long("lock")
                        .short('l')
                        .action(ArgAction::SetTrue)
                        .help("Remove from lockfile"),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help("Agree to all prompts"),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("Show the least output possible"),
                ]),
            Command::new("list")
                .short_flag('L')
                .about("List installed packages")
                .arg(
                    Arg::new("available")
                        .long("available")
                        .short('a')
                        .help("List available packages")
                        .action(ArgAction::SetTrue),
                ),
            Command::new("sync")
                .short_flag('S')
                .about("Sync repository manifests")
                .args([
                    Arg::new("repos")
                        .action(ArgAction::Set)
                        .required(false)
                        .num_args(1..)
                        .help("Selected repositories to sync"),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("Show the least output possible"),
                ]),
            Command::new("repo")
                .short_flag('P')
                .about("Manage repositories")
                .subcommand_required(true)
                .subcommands([
                    Command::new("add")
                        .short_flag('a')
                        .about("Add a repository")
                        .args([
                            Arg::new("urls")
                                .help("Repository URL(s)")
                                .action(ArgAction::Set)
                                .required(true)
                                .num_args(1..)
                                .value_hint(ValueHint::Url),
                            Arg::new("quiet")
                                .long("quiet")
                                .short('q')
                                .action(ArgAction::SetTrue)
                                .help("Show the least output possible"),
                        ]),
                    Command::new("remove")
                        .short_flag('r')
                        .about("Remove a repository")
                        .args([
                            Arg::new("names")
                                .help("Repository name(s)")
                                .action(ArgAction::Set)
                                .required(true)
                                .num_args(1..),
                            Arg::new("force")
                                .long("force")
                                .short('f')
                                .action(ArgAction::SetTrue)
                                .help("Agree to all prompts"),
                            Arg::new("all")
                                .long("all")
                                .short('a')
                                .action(ArgAction::SetTrue)
                                .conflicts_with("names")
                                .help("Remove all repositories"),
                            Arg::new("quiet")
                                .long("quiet")
                                .short('q')
                                .action(ArgAction::SetTrue)
                                .help("Show the least output possible"),
                        ]),
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
                            Arg::new("directory")
                                .long("output-directory")
                                .short('o')
                                .action(ArgAction::Set)
                                .num_args(1)
                                .default_value(".")
                                .value_hint(ValueHint::DirPath)
                                .help("Directory under which the repository is created"),
                            Arg::new("quiet")
                                .long("quiet")
                                .short('q')
                                .action(ArgAction::SetTrue)
                                .help("Show the least output possible"),
                        ]),
                ]),
            Command::new("query")
                .short_flag('Q')
                .about("Query a package's metadata")
                .arg(
                    Arg::new("package")
                        .help("Selected package to query")
                        .required(true)
                        .action(ArgAction::Set)
                        .num_args(1),
                ),
            Command::new("changelog")
                .short_flag('C')
                .about("Display a package's changelog")
                .args([
                    Arg::new("package")
                        .help("Selected package")
                        .action(ArgAction::Set)
                        .num_args(1),
                    Arg::new("latest")
                        .short('l')
                        .long("latest")
                        .help("Show only the latest changes")
                        .action(ArgAction::SetTrue),
                ]),
            Command::new("package")
                .short_flag('K')
                .about("Compress a directory into a .tar.lz4 package archive")
                .args([
                    Arg::new("directory")
                        .help("Path to package directory")
                        .action(ArgAction::Set)
                        .required(true)
                        .num_args(1)
                        .value_hint(ValueHint::DirPath),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("Show the least output possible"),
                ]),
            Command::new("generate")
                .short_flag('N')
                .about("Generate HTML files for a repository")
                .args([
                    Arg::new("url")
                        .long("url")
                        .short('u')
                        .default_value("/")
                        .action(ArgAction::Set)
                        .help("Base url")
                        .value_hint(ValueHint::Url),
                    Arg::new("repo")
                        .long("repository")
                        .short('r')
                        .required(true)
                        .action(ArgAction::Set)
                        .help("Repository url")
                        .value_hint(ValueHint::Url),
                    Arg::new("manifest")
                        .long("manifest-path")
                        .short('m')
                        .default_value("repo.toml")
                        .action(ArgAction::Set)
                        .num_args(1)
                        .help("Path to the repo.toml file")
                        .value_hint(ValueHint::FilePath),
                    Arg::new("directory")
                        .long("output-directory")
                        .short('o')
                        .num_args(1)
                        .default_value(".")
                        .action(ArgAction::Set)
                        .help("Directory under which HTML files will be written")
                        .value_hint(ValueHint::DirPath),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("Show the least output possible"),
                ]),
            Command::new("serve")
                .short_flag('E')
                .about("Serve a package web index using a repo.toml")
                .args([
                    Arg::new("host")
                        .long("host")
                        .short('s')
                        .default_value("localhost")
                        .action(ArgAction::Set)
                        .help("Server host")
                        .value_hint(ValueHint::Hostname),
                    Arg::new("port")
                        .long("port")
                        .short('p')
                        .default_value("8887")
                        .action(ArgAction::Set)
                        .help("Server port"),
                    Arg::new("url")
                        .long("url")
                        .short('u')
                        .default_value("/")
                        .action(ArgAction::Set)
                        .help("Base url")
                        .value_hint(ValueHint::Url),
                    Arg::new("repo")
                        .long("repository")
                        .short('r')
                        .required(true)
                        .action(ArgAction::Set)
                        .help("Repository url")
                        .value_hint(ValueHint::Url),
                    Arg::new("manifest")
                        .long("manifest-path")
                        .short('m')
                        .default_value("repo.toml")
                        .action(ArgAction::Set)
                        .help("Path to the repo.toml file")
                        .value_hint(ValueHint::FilePath),
                ]),
            Command::new("completions")
                .short_flag('O')
                .about("Generate tab-completion scripts for your shell")
                .arg(
                    Arg::new("shell")
                        .action(ArgAction::Set)
                        .value_parser(["bash", "zsh", "fish", "elvish", "powershell"])
                        .required(true)
                        .num_args(1),
                ),
        ]);

    match cli.clone().get_matches().subcommand() {
        Some(("get", get_matches)) => {
            let force = get_matches.get_flag("force");
            let quiet = get_matches.get_flag("quiet");

            let packages = get_matches.get_many::<String>("packages").unwrap();
            let packages_vec: Vec<String> = packages.map(|s| s.into()).collect::<Vec<_>>();
            for package in packages_vec {
                get::command(&package, force, quiet);
            }
        }
        Some(("install", install_matches)) => {
            let force = install_matches.get_flag("force");
            let quiet = install_matches.get_flag("quiet");

            if let Some(package) = install_matches.get_one::<String>("package") {
                install::command(package, force, quiet);
            } else {
                let pkgfile = install_matches.get_one::<String>("pkgfile").unwrap();
                let name_option = install_matches.get_one::<String>("name");
                let version_option = install_matches.get_one::<String>("version");

                match install::use_pkgfile(pkgfile, name_option, version_option, force, quiet) {
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("{}", format!("- {}", error).bright_red());
                    }
                }
            }
        }
        Some(("upgrade", upgrade_matches)) => {
            let force = upgrade_matches.get_flag("force");
            let quiet = upgrade_matches.get_flag("quiet");

            if let Some(packages) = upgrade_matches.get_many::<String>("packages") {
                let packages_vec: Vec<&str> = packages.map(|s| s.as_str()).collect::<Vec<_>>();
                for package in packages_vec {
                    upgrade::command(Some(package), force, quiet);
                }
            } else {
                upgrade::command(None, force, quiet);
            }
        }
        Some(("remove", remove_matches)) => {
            let lock_flag = remove_matches.get_flag("lock");
            let force_flag = remove_matches.get_flag("force");
            let quiet_flag = remove_matches.get_flag("quiet");

            if remove_matches.get_flag("all") {
                commands::remove(None, lock_flag, force_flag, quiet_flag);
            } else {
                let packages = remove_matches.get_many::<String>("packages").unwrap();
                let packages_vec: Vec<String> = packages.map(|s| s.to_owned()).collect::<Vec<_>>();

                commands::remove(Some(packages_vec), lock_flag, force_flag, quiet_flag);
            }
        }
        Some(("list", list_matches)) => {
            if list_matches.get_flag("available") {
                list::available()
            } else {
                list::installed();
            }
        }
        Some(("sync", sync_matches)) => {
            let quiet = sync_matches.get_flag("quiet");

            if let Some(repos) = sync_matches.get_many::<String>("repos") {
                let repos_vec = repos.map(|r| r.to_owned()).collect::<Vec<_>>();
                sync::command(Some(repos_vec), quiet);
            } else {
                sync::command(None, quiet);
            }
        }
        Some(("repo", repo_matches)) => match repo_matches.subcommand() {
            Some(("add", add_matches)) => {
                let quiet = add_matches.get_flag("quiet");

                let repository_urls = add_matches.get_many::<String>("urls").unwrap();
                let repository_urls_vec: Vec<String> =
                    repository_urls.map(|s| s.into()).collect::<Vec<_>>();

                for repository_url in repository_urls_vec {
                    repo::add(repository_url, quiet);
                }
            }

            Some(("remove", remove_matches)) => {
                let force = remove_matches.get_flag("force");
                let quiet = remove_matches.get_flag("quiet");

                if remove_matches.get_flag("all") {
                    repo::remove(None, force, quiet)
                } else {
                    let repository_names = remove_matches.get_many::<String>("names").unwrap();
                    let repository_names_vec: Vec<String> =
                        repository_names.map(|s| s.into()).collect::<Vec<_>>();

                    for repository_name in repository_names_vec {
                        repo::remove(Some(repository_name), force, quiet);
                    }
                }
            }

            Some(("info", info_matches)) => {
                let repository_url = info_matches.get_one::<String>("name").unwrap();
                repo::info(repository_url.into());
            }

            Some(("list", _)) => {
                repo::list();
            }

            Some(("init", init_matches)) => {
                let quiet = init_matches.get_flag("quiet");

                let repo_name = init_matches.get_one::<String>("name").unwrap();
                let repo_maintainer = init_matches.get_one::<String>("maintainer").unwrap();
                let repo_description = init_matches.get_one::<String>("description").unwrap();
                let repo_directory = init_matches.get_one::<String>("directory").unwrap();

                repo::init(
                    repo_name.into(),
                    repo_maintainer.into(),
                    repo_description.into(),
                    repo_directory.into(),
                    quiet,
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
            let quiet = package_matches.get_flag("quiet");

            let directory_name = package_matches.get_one::<String>("directory").unwrap();
            package::command(directory_name.into(), quiet);
        }
        Some(("generate", generate_matches)) => {
            let quiet = generate_matches.get_flag("quiet");

            let website_url = generate_matches.get_one::<String>("url").unwrap();
            let repo_url = generate_matches.get_one::<String>("repo").unwrap();
            let manifest_path = generate_matches.get_one::<String>("manifest").unwrap();
            let output_directory = generate_matches.get_one::<String>("directory").unwrap();

            generate::command(
                website_url,
                repo_url,
                manifest_path.into(),
                output_directory.into(),
                quiet,
            );
        }
        Some(("serve", serve_matches)) => {
            let host = serve_matches.get_one::<String>("host").unwrap();
            let port = serve_matches.get_one::<String>("port").unwrap();
            let base_url = serve_matches.get_one::<String>("url").unwrap();
            let repo_url = serve_matches.get_one::<String>("repo").unwrap();
            let manifest_path = serve_matches.get_one::<String>("manifest").unwrap();

            serve::command(host, port, base_url, repo_url, manifest_path.into());
        }
        Some(("completions", completions_matches)) => {
            let shell = completions_matches.get_one::<String>("shell").unwrap();

            match shell.as_str() {
                "bash" => clap_complete::generate(Shell::Bash, &mut cli, "aati", &mut stdout()),
                "zsh" => clap_complete::generate(Shell::Zsh, &mut cli, "aati", &mut stdout()),
                "fish" => clap_complete::generate(Shell::Fish, &mut cli, "aati", &mut stdout()),
                "elvish" => clap_complete::generate(Shell::Elvish, &mut cli, "aati", &mut stdout()),
                "powershell" => {
                    clap_complete::generate(Shell::PowerShell, &mut cli, "aati", &mut stdout())
                }
                _ => unreachable!(),
            }
        }

        _ => unreachable!(),
    }
}

// تم بحمد الله
