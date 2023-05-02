// بسم الله الرحمن الرحيم

mod commands;
mod constants;
mod structs;
mod utils;

use std::{env, process::exit};

use colored::Colorize;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args.get(1).map(String::as_str) {
            // I got plans to make colored output an option since some terminals might not support coloring that well
            Some("--help") => {
                println!(
                    "{} - Minimal package manager written in Rust",
                    "aati".bright_blue(),
                );
                println!();
                println!("Usage: {} [COMMANDS/OPTIONS]", "aati".bright_blue());
                println!();
                println!("Commands:");
                println!("    get <package>               Download a package from the Repository and install it");
                println!("    upgrade [package]           Upgrade a package or all packages");
                println!("    uninstall <package>         Uninstall a package");
                println!("    remove <package>            Alias of uninstall");
                println!("    list [installed/available]  List installed or available packages");
                println!("    sync                        Update packages index");
                println!("    repo [url://to/repo]/[init] Display repo info, set a default repo or initialise a repo");
                println!("    info <package>              Show a package's info");
                println!("    package <path/to/binary>    Compress a binary into LZ4");
                println!();
                println!("Options:");
                println!("    -V, --version Print version info");
                println!("        --help    Show this help message");
                println!();
                println!(
                    "{}",
                    "Amad Project: https://codeberg.org/amad".bright_white()
                );
            }

            Some("-V") | Some("--version") => {
                println!(
                    "aati v{}\n\n{}",
                    constants::CURRENT_VERSION,
                    "Amad Project: https://codeberg.org/amad".bright_white()
                );
            }

            Some("get") => match args.get(2) {
                Some(package_name) => {
                    commands::get_command(package_name);
                }

                None => {
                    println!("{}", "- No package name?".bright_red());
                    exit(1);
                }
            },

            // Some("upgrade") => {
            //     if let Some(arg) = args.get(2) {
            //         let choice = arg;
            //         commands::upgrade_command(Some(choice));
            //     } else {
            //         commands::upgrade_command(None);
            //     }
            // }
            
            Some("uninstall") | Some("remove") => match args.get(2) {
                Some(package_name) => {
                    commands::uninstall_command(package_name);
                }

                None => {
                    println!("{}", "- No package name?".bright_red());
                    exit(1);
                }
            },

            Some("list") => {
                if let Some(arg) = args.get(2) {
                    let choice = arg;
                    commands::list_command(Some(choice));
                } else {
                    commands::list_command(None);
                }
            }

            Some("sync") => commands::sync_command(),

            Some("repo") => {
                if let Some(arg1) = args.get(2) {
                    if let Some(arg2) = args.get(3) {
                        commands::repo_command(Some(arg1), Some(arg2));
                    } else {
                        commands::repo_command(Some(arg1), None)
                    }
                } else {
                    commands::repo_command(None, None);
                }
            }

            Some("info") => match args.get(2) {
                Some(package_name) => {
                    commands::info_command(package_name, None);
                }

                None => {
                    println!("{}", "- No package name?".bright_red());
                    exit(1);
                }
            },

            Some("package") => match args.get(2) {
                Some(package_name) => {
                    commands::package_command(package_name);
                }

                None => {
                    println!("{}", "- No package name?".bright_red());
                    exit(1);
                }
            },

            _ => {
                println!("{}", "- Unknown command!".bright_red())
            }
        }
    } else {
        println!(
            "{}",
            "+ Try 'aati --help' for more information".bright_green()
        );
    }
}

// تم بحمد الله
