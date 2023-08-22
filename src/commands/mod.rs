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
use toml::Value;

use crate::{
    utils::{get_aati_lock, get_package_versions, prompt_yn},
    version::get_versions,
};

mod changelog;
pub mod generate;
pub mod get;
pub mod install;
pub mod list;
pub mod package;
pub mod query;
mod remove;
pub mod repo;
pub mod serve;
pub mod sync;
pub mod upgrade;

// Either a Some() of a Vec of Strings or a None which will be treated as --all
pub fn remove(packages_option: Option<Vec<String>>, lock: bool, force: bool, quiet: bool) {
    let aati_lock: Value = get_aati_lock().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    if let Some(packages) = packages_option {
        if lock {
            // $ aati remove --lock package1 package2 package3...
            let mut did_removal = false;

            for package_name in packages {
                if let Some(installed_package) = is_installed(&package_name) {
                    if !quiet {
                        println!(
                            "{}",
                            format!(
                                "+ Removing package ({}/{}-{}) from Lockfile...",
                                installed_package["source"].as_str().unwrap(),
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap()
                            )
                            .bright_green()
                        );
                    }

                    remove::remove_from_lockfile(&package_name);
                    did_removal = true;
                } else if !quiet {
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

            if did_removal && !quiet {
                println!(
                    "{}",
                    "+ Removal from Lockfile finished successfully!".bright_green()
                );
            }
        } else {
            // $ aati remove package1 package2 package3...
            for package_name in packages {
                if let Some(package) = is_installed(&package_name) {
                    if force
                        || prompt_yn(
                            format!(
                                "/ Are you sure you want to completely remove {}/{}-{}?",
                                package["source"].as_str().unwrap(),
                                package_name,
                                package["version"].as_str().unwrap()
                            )
                            .as_str(),
                        )
                    {
                        if !quiet {
                            println!(
                                "{}",
                                format!("+ Removing '{}'...", package_name).bright_green()
                            );
                        }
                        remove::command(package["name"].as_str().unwrap(), force, quiet);
                    } else if !quiet {
                        println!("{}", "+ Transaction aborted".bright_green());
                    }
                } else {
                    println!(
                        "{}",
                        format!("- Package '{}' is not installed!", package_name).bright_red()
                    );
                }
            }
        }
    } else if lock {
        // $ aati remove --lock --all
        if force
            || prompt_yn(
                "/ Are you sure you want to remove all of your packages from the Lockfile?",
            )
        {
            for installed_package in installed_packages {
                let package_name = installed_package["name"].as_str().unwrap();

                if !quiet {
                    println!(
                        "{}",
                        format!("+ Removing '{}'...", package_name).bright_green()
                    );
                }

                remove::remove_from_lockfile(package_name);
            }
            if !quiet {
                println!(
                    "{}",
                    "+ Removal from Lockfile finished successfully!".bright_green()
                );
            }
        } else if !quiet {
            println!("{}", "+ Transaction aborted".bright_green());
        }
    } else {
        // $ aati remove --all
        if !installed_packages.is_empty() {
            if force || prompt_yn("/ Are you sure you want to remove all of your packages?") {
                for installed_package in installed_packages {
                    if !quiet {
                        println!(
                            "{}",
                            format!(
                                "+ Removing '{}'...",
                                installed_package["name"].as_str().unwrap()
                            )
                            .bright_green()
                        );
                    }

                    remove::command(installed_package["name"].as_str().unwrap(), force, quiet);
                }
            } else if !quiet {
                println!("{}", "+ Transaction aborted".bright_green());
            }
        } else if !quiet {
            println!("{}", "+ No packages to remove".bright_green());
        }
    }
}

pub fn changelog(package_name_option: Option<&str>, latest_only: bool) {
    if let Some(package_name) = package_name_option {
        match get_package_versions(package_name) {
            Some(versions) => {
                changelog::display(&versions, latest_only);
            }

            None => {
                println!(
                    "{}",
                    "- Package not found in the added repositories!".bright_red()
                );
            }
        }
    } else {
        changelog::display(&get_versions(), latest_only);
    }
}

fn is_installed(package_name: &str) -> Option<Value> {
    let aati_lock: Value = get_aati_lock().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    let mut package_option = None;

    for installed_package in installed_packages {
        if installed_package["name"].as_str().unwrap() == package_name {
            package_option = Some(installed_package.clone());
        }
    }

    package_option
}
