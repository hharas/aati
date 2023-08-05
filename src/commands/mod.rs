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

use colored::Colorize;
use toml::Value;

use crate::{
    utils::{extract_package, get_aati_config, get_aati_lock, get_repo_config, prompt_yn},
    version::get_versions,
};

use self::changelog::get_package_versions;

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
pub fn remove(packages_option: Option<Vec<String>>, lock: bool, force: bool) {
    let aati_lock: Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    if let Some(packages) = packages_option {
        if lock {
            // $ aati remove --lock package1 package2 package3...
            let packages = &packages[1..];
            let mut did_removal = false;

            for package_name in packages {
                let result = is_installed(package_name);

                if let Some(installed_package) = result {
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
                    remove::remove_from_lockfile(package_name);
                    did_removal = true;
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
                    "+ Removal from Lockfile finished successfully!".bright_green()
                );
            }
        } else {
            // $ aati remove package1 package2 package3...
            let package_name = packages.first().unwrap();

            let result = is_installed(package_name);

            if let Some(package) = result {
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
                    println!(
                        "{}",
                        format!("+ Removing '{}'...", package_name).bright_green()
                    );
                    remove::command(&package, force);
                } else {
                    println!("{}", "+ Transaction aborted".bright_green());
                }
            } else {
                println!(
                    "{}",
                    format!("- Package '{}' is not installed!", package_name).bright_red()
                );
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

                println!(
                    "{}",
                    format!("+ Removing '{}'...", package_name).bright_green()
                );
                remove::remove_from_lockfile(package_name);
            }
            println!(
                "{}",
                "+ Removal from Lockfile finished successfully!".bright_green()
            );
        } else {
            println!("{}", "+ Transaction aborted".bright_green());
        }
    } else {
        // $ aati remove --all
        if !installed_packages.is_empty() {
            if force || prompt_yn("/ Are you sure you want to remove all of your packages?") {
                for installed_package in installed_packages {
                    println!(
                        "{}",
                        format!(
                            "+ Removing '{}'...",
                            installed_package["name"].as_str().unwrap()
                        )
                        .bright_green()
                    );
                    remove::command(installed_package, force);
                }
            } else {
                println!("{}", "+ Transaction aborted".bright_green());
            }
        } else {
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
    let aati_lock: Value = get_aati_lock().unwrap().parse().unwrap();
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();
    let repo_list = aati_config["sources"]["repos"].as_array().unwrap();
    let mut package_option = None;
    let mut added_repos: Vec<Value> = Vec::new();

    for repo_info in repo_list {
        added_repos.push(
            get_repo_config(repo_info["name"].as_str().unwrap())
                .unwrap()
                .parse::<Value>()
                .unwrap(),
        );
    }

    let installed_packages = aati_lock["package"].as_array().unwrap();

    if let Some(extracted_package) = extract_package(package_name, &added_repos) {
        for installed_package in installed_packages {
            if installed_package["name"].as_str().unwrap() == extracted_package[1] {
                package_option = Some(installed_package.clone());
            }
        }
    }

    package_option
}
