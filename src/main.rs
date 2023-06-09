/* بسم الله الرحمن الرحيم

   Aati - Cross-platform Package Manager written in Rust.
   Copyright (C) 2023  Husayn Haras

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

mod commands;
mod globals;
mod types;
mod utils;
mod version;
use std::{env, process::exit};

use colored::Colorize;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args.get(1).map(String::as_str) {
            Some("--help") => {
                println!(
                    "aati - Cross-platform package manager written in Rust

Usage: aati [COMMANDS/OPTIONS]

Commands:
    get <package>                    Download a package from the Repository and install it
    install <path/to/archive>        Install a package from an LZ4 Archive
    upgrade [package]                Upgrade a package or all packages (alias: update)
    remove <package>/<-all>[--force] Remove a package (alias: uninstall)
    list [installed/available]       List installed or available packages
    sync                             Update package index
    repo                             Package Repository Management
      add <url://to/repo>              Add a package repository
      remove <repo name>               Remove a package repository
      list                             List all added package repositories
      info <repo name>                 Show an overview of a repository
      init                             Initialise a new package repository
    info <package>                   Show a package's info
    package <path/to/binary>         Compress a binary into LZ4
    generate                         Generate .html files for a package repository
    serve [host:port]                Host a package web index (default: localhost:8887)

Options:
    -V, --version Print version info
        --help    Show this help message

Copyright (C) 2023  Husayn Haras

This program is free software: you can redistribute it and/or modify
it under the terms of version 3 of the GNU General Public License
as published by the Free Software Foundation.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

User Guide: https://github.com/hharas/aati/wiki/2.-User-Guide
Issue tracker: https://github.com/hharas/aati/issues"
                );
            }

            Some("-V") | Some("--version") => {
                let aati_version = version::VERSION;

                println!("aati version {}", aati_version,);
            }

            Some("get") => match args.get(2) {
                Some(_) => {
                    commands::get(&args[2..]);
                }

                None => {
                    println!("{}", "- No package name?".bright_red());
                    exit(1);
                }
            },

            Some("upgrade") | Some("update") => commands::upgrade(&args[2..]),

            Some("uninstall") | Some("remove") => match args.get(2) {
                Some(_) => {
                    commands::remove(&args[2..]);
                }

                None => {
                    println!("{}", "- No package name?".bright_red());
                    exit(1);
                }
            },

            Some("list") => {
                if let Some(arg) = args.get(2) {
                    let choice = arg;
                    commands::list(Some(choice));
                } else {
                    commands::list(None);
                }
            }

            Some("sync") => commands::sync(),

            Some("repo") => {
                if let Some(arg1) = args.get(2) {
                    if let Some(arg2) = args.get(3) {
                        commands::repo(Some(arg1), Some(arg2));
                    } else {
                        commands::repo(Some(arg1), None)
                    }
                } else {
                    commands::repo(None, None);
                }
            }

            Some("info") => match args.get(2) {
                Some(package_name) => {
                    commands::info(package_name, None);
                }

                None => {
                    println!("{}", "- No package name?".bright_red());
                    exit(1);
                }
            },

            Some("package") => match args.get(2) {
                Some(package_directory) => {
                    commands::package(package_directory.to_string());
                }

                None => {
                    println!("{}", "- No package name?".bright_red());
                    exit(1);
                }
            },

            Some("install") => match args.get(2) {
                Some(package_name) => {
                    commands::install(package_name);
                }

                None => {
                    println!("{}", "- No archive name?".bright_red());
                    exit(1);
                }
            },

            Some("generate") => commands::generate(),

            Some("serve") => match args.get(2) {
                Some(address) => {
                    commands::serve(Some(address));
                }

                None => {
                    commands::serve(None);
                }
            },

            _ => {
                println!("{}", "- Unknown command!".bright_red())
            }
        }
    } else {
        println!("Try 'aati --help' for more information");
    }
}

// تم بحمد الله
