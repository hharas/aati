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

use std::process::exit;

use colored::Colorize;
use toml::Value;

use crate::utils::{
    extract_package, get_aati_config, get_aati_lock, get_repo_config, is_supported, prompt_yn,
};

use super::{get, remove};

pub fn command(choice: Option<&str>, force: bool, quiet: bool) {
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();
    let repo_list = aati_config["sources"]["repos"].as_array().unwrap();
    let mut added_repos: Vec<Value> = Vec::new();

    for repo_info in repo_list {
        added_repos.push(
            get_repo_config(repo_info["name"].as_str().unwrap())
                .parse::<Value>()
                .unwrap(),
        );
    }

    let aati_lock: Value = get_aati_lock().parse().unwrap();

    let repos = aati_config["sources"]["repos"].as_array().unwrap();
    let mut repos_toml: Vec<Value> = Vec::new();

    for repo in repos {
        repos_toml.push(
            get_repo_config(repo["name"].as_str().unwrap())
                .parse::<Value>()
                .unwrap(),
        )
    }

    let installed_packages = aati_lock["package"].as_array().unwrap();

    if let Some(package_name) = choice {
        match extract_package(package_name, &added_repos) {
            Some(extracted_package) => {
                let mut is_installed = false;
                let mut is_up_to_date = true;

                for installed_package in installed_packages {
                    if installed_package["name"].as_str().unwrap() == extracted_package[1]
                        && installed_package["source"].as_str().unwrap() == extracted_package[0]
                    {
                        is_installed = true;
                        if installed_package["version"].as_str().unwrap() != extracted_package[2] {
                            is_up_to_date = false;
                        }
                    }
                }

                if is_installed {
                    if !is_up_to_date {
                        remove(Some(vec![package_name.into()]), false, true, quiet);
                        get::command(package_name, true, quiet);
                    } else {
                        if !quiet {
                            println!("{}", "+ That Package is already up to date!".bright_blue());
                        }
                        exit(1);
                    }
                } else {
                    println!("{}", "- Package not installed!".bright_red());
                    exit(1);
                }
            }

            None => {
                println!("{}", "- PACKAGE NOT FOUND!".bright_red());
                exit(1);
            }
        }
    } else {
        let mut to_be_upgraded: Vec<&str> = Vec::new();

        if !quiet {
            println!("{}", "+ Packages to be upgraded:".bright_green());
        }

        if !installed_packages.is_empty() {
            for installed_package in installed_packages {
                if installed_package["source"].as_str().unwrap() != "local" {
                    let available_packages = repos_toml
                        .iter()
                        .find(|r| r["repo"]["name"] == installed_package["source"])
                        .unwrap()["index"]["packages"]
                        .as_array()
                        .unwrap();

                    for available_package in available_packages {
                        if installed_package["name"] == available_package["name"]
                            && is_supported(available_package["target"].as_str().unwrap())
                            && installed_package["version"].as_str().unwrap()
                                != available_package["versions"]
                                    .as_array()
                                    .unwrap()
                                    .first()
                                    .unwrap()["tag"]
                                    .as_str()
                                    .unwrap()
                        {
                            to_be_upgraded.push(available_package["name"].as_str().unwrap());

                            if !quiet {
                                println!(
                                    "{}   {}/{}-{} -> {}",
                                    "+".bright_green(),
                                    installed_package["source"].as_str().unwrap(),
                                    installed_package["name"].as_str().unwrap(),
                                    installed_package["version"].as_str().unwrap(),
                                    available_package["versions"]
                                        .as_array()
                                        .unwrap()
                                        .first()
                                        .unwrap()["tag"]
                                        .as_str()
                                        .unwrap(),
                                );
                            }
                        }
                    }
                }
            }

            if !to_be_upgraded.is_empty() {
                if force || prompt_yn("/ Are you sure you want to continue this Transaction?") {
                    for package in to_be_upgraded {
                        remove(Some(vec![package.into()]), false, true, quiet);
                        get::command(package, true, quiet);
                    }

                    if !quiet {
                        println!("{}", "+ Finished upgrading!".bright_green());
                    }
                } else if !quiet {
                    println!("{}", "+ Transaction aborted".bright_green());
                }
            } else if !quiet {
                println!("{}", "+   None!".bright_green());
                println!("{}", "+ It's all up-to-date!".bright_green());
            }
        } else if !quiet {
            println!("{}", "+   None!".bright_green());
            println!(
                "{}",
                "+ You have no packages installed to upgrade!".bright_green()
            );
        }
    }
}
