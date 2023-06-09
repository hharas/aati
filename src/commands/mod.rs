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

use colored::Colorize;
use toml::Value;

use crate::utils::{get_aati_lock, is_installed, prompt_yn};

mod generate;
mod get;
mod info;
mod install;
mod list;
mod package;
mod remove;
mod repo;
mod serve;
mod sync;
mod upgrade;

pub fn get(arguments: &[String]) {
    for package_name in arguments {
        get::command(package_name);
    }
}

pub fn upgrade(arguments: &[String]) {
    if arguments.is_empty() {
        upgrade::command(None);
    } else if arguments.len() == 1 {
        upgrade::command(Some(arguments.first().unwrap()));
    } else {
        for argument in arguments {
            upgrade::command(Some(argument));
        }
    }
}

pub fn remove(arguments: &[String]) {
    let aati_lock: Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    if arguments.len() == 1 {
        if arguments.first().unwrap() == "--all" {
            if !installed_packages.is_empty() {
                if prompt_yn("/ Are you sure you want to remove all of your packages?") {
                    for installed_package in installed_packages {
                        println!(
                            "{}",
                            format!(
                                "+ Removing '{}'...",
                                installed_package["name"].as_str().unwrap()
                            )
                            .bright_green()
                        );
                        remove::command(installed_package);
                    }
                } else {
                    println!("{}", "+ Transaction aborted".bright_green());
                }
            } else {
                println!("{}", "+ No packages to remove".bright_green());
            }
        } else {
            let package_name = arguments.first().unwrap();

            let result = is_installed(package_name);
            let installed = result.0;
            let package_option = result.1;

            if installed {
                if let Some(package) = package_option {
                    if prompt_yn(
                        format!(
                            "/ Are you sure you want to completely remove {}/{}-{}?",
                            package["source"].as_str().unwrap(),
                            package_name,
                            package["version"].as_str().unwrap()
                        )
                        .as_str(),
                    ) {
                        println!(
                            "{}",
                            format!("+ Removing '{}'...", package_name).bright_green()
                        );
                        remove::command(&package);
                    } else {
                        println!("{}", "+ Transaction aborted".bright_green());
                    }
                } else {
                    println!("{}", "- This Package is not installed!".bright_red());
                }
            } else {
                println!("{}", "- This Package is not installed!".bright_red());
            }
        }
    } else if arguments.first().unwrap() == "--force" {
        if arguments.last().unwrap() == "--all" {
            if prompt_yn("/ Are you sure you want to forcefully remove all of your packages?") {
                for installed_package in installed_packages {
                    let package_name = installed_package["name"].as_str().unwrap();

                    println!(
                        "{}",
                        format!("+ Removing '{}'...", package_name).bright_green()
                    );
                    remove::remove_from_lockfile(package_name);
                }
                println!(
                    "{}",
                    "+ Forceful removal finished successfully!".bright_green()
                );
            } else {
                println!("{}", "+ Transaction aborted".bright_green());
            }
        } else {
            let packages = &arguments[1..];
            let mut did_removal = false;

            for package_name in packages {
                let result = is_installed(package_name);
                let installed = result.0;
                let package_option = result.1;

                if installed {
                    if let Some(installed_package) = package_option {
                        println!(
                            "{}",
                            format!(
                                "+ Forcefully removing package ({}/{}-{})...",
                                installed_package["source"].as_str().unwrap(),
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap()
                            )
                            .bright_green()
                        );
                        remove::remove_from_lockfile(package_name);
                        did_removal = true;
                    }
                } else {
                    println!(
                        "{}",
                        format!(
                            "+ Package '{}' ignored due to not being installed",
                            package_name
                        )
                        .yellow()
                    );
                }
            }

            if did_removal {
                println!(
                    "{}",
                    "+ Forceful removal finished successfully!".bright_green()
                );
            }
        }
    } else if arguments.last().unwrap() == "--force" {
        if arguments.first().unwrap() == "--all" {
            if prompt_yn("/ Are you sure you want to forcefully remove all of your packages?") {
                for installed_package in installed_packages {
                    let package_name = installed_package["name"].as_str().unwrap();

                    println!(
                        "{}",
                        format!("+ Forcefully removing '{}'...", package_name).bright_green()
                    );
                    remove::remove_from_lockfile(package_name);
                }
                println!(
                    "{}",
                    "+ Forceful removal finished successfully!".bright_green()
                );
            } else {
                println!("{}", "+ Transaction aborted".bright_green());
            }
        } else {
            let packages = &arguments[..arguments.len() - 1];

            let mut did_removal = false;

            for package_name in packages {
                let result = is_installed(package_name);
                let installed = result.0;
                let package_option = result.1;

                if installed {
                    if let Some(installed_package) = package_option {
                        println!(
                            "{}",
                            format!(
                                "+ Forcefully removing package ({}/{}-{})...",
                                installed_package["source"].as_str().unwrap(),
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap()
                            )
                            .bright_green()
                        );
                        remove::remove_from_lockfile(package_name);
                        did_removal = true;
                    }
                } else {
                    println!(
                        "{}",
                        format!(
                            "+ Package '{}' ignored due to not being installed",
                            package_name
                        )
                        .yellow()
                    );
                }
            }

            if did_removal {
                println!(
                    "{}",
                    "+ Forceful removal finished successfully!".bright_green()
                );
            }
        }
    } else {
        for package_name in arguments {
            let result = is_installed(package_name);
            let installed = result.0;
            let package_option = result.1;

            if installed {
                if let Some(installed_package) = package_option {
                    println!(
                        "{}",
                        format!(
                            "+ Removing package ({}/{}-{})...",
                            installed_package["source"].as_str().unwrap(),
                            installed_package["name"].as_str().unwrap(),
                            installed_package["version"].as_str().unwrap()
                        )
                        .bright_green()
                    );
                    remove::command(&installed_package);
                }
            } else {
                println!(
                    "{}",
                    format!(
                        "+ Package '{}' ignored due to not being installed",
                        package_name
                    )
                    .yellow()
                );
            }
        }
    }
}

pub fn list(choice_option: Option<&str>) {
    list::command(choice_option);
}

pub fn sync() {
    sync::command();
}

pub fn repo(first_argument_option: Option<&str>, second_argument_option: Option<&str>) {
    repo::command(first_argument_option, second_argument_option);
}

pub fn info(text: &str, repo_name: Option<&str>) {
    info::command(text, repo_name);
}

pub fn package(directory_name: String) {
    package::command(directory_name);
}

pub fn install(filename: &str) {
    install::command(filename);
}

pub fn generate() {
    generate::command();
}

pub fn serve(address_option: Option<&str>) {
    serve::command(address_option);
}
