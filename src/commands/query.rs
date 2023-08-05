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

use std::{process::exit, str::FromStr};

use colored::Colorize;
use toml::Value;

use crate::utils::{get_aati_config, get_aati_lock, get_repo_config, is_supported, prompt};

pub fn command(text: &str, repo_name: Option<&str>) {
    // Initialising main variables
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();
    let repos = aati_config["sources"]["repos"].as_array().unwrap();

    let aati_lock: Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    // Some placeholders too
    let mut is_installed = false;
    let mut is_up_to_date = false;
    let mut installed_package_version = "0.0.0";

    if !text.contains('/') {
        let mut results: Vec<Vec<Value>> = Vec::new();

        if let Some(repo_name) = repo_name {
            let repo_toml: Value = get_repo_config(repo_name).unwrap().parse().unwrap();
            let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

            for available_package in available_packages {
                if available_package["name"].as_str().unwrap() == text
                    && is_supported(available_package["target"].as_str().unwrap())
                {
                    results.push(vec![
                        available_package.clone(),
                        Value::from_str(
                            format!(
                                "name = \"{}\"\nurl = \"{}\"",
                                repo_name,
                                repos
                                    .iter()
                                    .find(|r| r["name"].as_str().unwrap() == repo_name)
                                    .unwrap()["url"]
                                    .as_str()
                                    .unwrap()
                            )
                            .as_str(),
                        )
                        .unwrap(),
                    ]);
                }
            }
        } else {
            for repo in repos {
                let repo_name = repo["name"].as_str().unwrap();

                let repo_toml: Value = get_repo_config(repo_name).unwrap().parse().unwrap();
                let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                for available_package in available_packages {
                    if available_package["name"].as_str().unwrap() == text
                        && is_supported(available_package["target"].as_str().unwrap())
                    {
                        results.push(vec![
                            available_package.clone(),
                            Value::from_str(
                                format!(
                                    "name = \"{}\"\nurl = \"{}\"",
                                    repo_name,
                                    repo["url"].as_str().unwrap()
                                )
                                .as_str(),
                            )
                            .unwrap(),
                        ]);
                    }
                }
            }
        }

        if !results.is_empty() {
            if results.len() == 1 {
                let package = results[0][0].clone();
                let repo_name = results[0][1]["name"].as_str().unwrap();
                let repo_url = results[0][1]["url"].as_str().unwrap();

                // Check if it's installed / up-to-date

                for installed_package in installed_packages {
                    if installed_package["name"] == package["name"]
                        && installed_package["source"].as_str().unwrap() == repo_name
                    {
                        installed_package_version = installed_package["version"].as_str().unwrap();

                        is_installed = true;
                        if installed_package["version"] == package["current"] {
                            is_up_to_date = true;
                        }
                    }
                }

                // Display!

                display_package(
                    package,
                    repo_name,
                    repo_url,
                    is_installed,
                    is_up_to_date,
                    installed_package_version,
                );
            } else {
                let conflicts: Vec<_> = results
                    .iter()
                    .enumerate()
                    .map(|(i, value)| {
                        [
                            (i + 1).to_string(),
                            value[0]["name"].as_str().unwrap().to_string(),
                            value[1]["name"].as_str().unwrap().to_string(),
                        ]
                    })
                    .collect();

                println!(
                    "{}",
                    format!(
                        "+ Package '{}' exists with the same name in multiple repositories:",
                        conflicts[0][1]
                    )
                    .yellow()
                );

                for conflict in conflicts.clone() {
                    println!(
                        "{}    ({}) {}/{}",
                        "+".yellow(),
                        conflict[0],
                        conflict[2],
                        conflict[1],
                    );
                }

                let input = prompt("* Enter the number of the package you choose:");

                match input.parse::<usize>() {
                    Ok(response) => {
                        let mut is_valid = false;

                        for conflict in conflicts {
                            if conflict[0] == response.to_string() {
                                is_valid = true;
                            }
                        }

                        if is_valid {
                            let package = results[response - 1][0].clone();
                            let repo_name = results[response - 1][1]["name"].as_str().unwrap();
                            let repo_url = results[response - 1][1]["url"].as_str().unwrap();

                            for installed_package in installed_packages {
                                if installed_package["name"] == package["name"]
                                    && installed_package["source"].as_str().unwrap() == repo_name
                                {
                                    is_installed = true;
                                    if installed_package["version"] == package["current"] {
                                        is_up_to_date = true;
                                        installed_package_version =
                                            installed_package["version"].as_str().unwrap()
                                    }
                                }
                            }

                            // Display!

                            display_package(
                                package,
                                repo_name,
                                repo_url,
                                is_installed,
                                is_up_to_date,
                                installed_package_version,
                            );
                        } else {
                            println!("{}", "- INVALID CHOICE!".bright_red());
                            exit(1);
                        }
                    }

                    Err(error) => {
                        println!(
                            "{}",
                            format!("- FAILED TO PARSE INPUT! ERROR[9]: {}", error).bright_red()
                        );
                        exit(1);
                    }
                }
            }
        } else {
            println!("{}", "- Package not found!".bright_red());
            exit(1);
        }
    } else {
        let (repo_name, text_to_be_extracted) = text.split_once('/').unwrap();

        command(text_to_be_extracted, Some(repo_name));
    }
}

pub fn display_package(
    package: Value,
    repo_name: &str,
    repo_url: &str,
    is_installed: bool,
    is_up_to_date: bool,
    installed_package_version: &str,
) {
    let name = package["name"].as_str().unwrap();
    let version = package["current"].as_str().unwrap();

    let versions = package["versions"].as_array().unwrap();
    let mut tags: Vec<&str> = vec![];
    for version in versions {
        tags.push(version["tag"].as_str().unwrap())
    }
    let author = package["author"].as_str().unwrap();
    let arch = package["target"].as_str().unwrap();
    let url = package["url"].as_str().unwrap();
    let description = package["description"].as_str().unwrap();

    println!(
        "{}\n    Name: {}\n    Author: {}\n    Architecture: {}\n    Repository: {} ({})",
        "+ Package Information:".bright_green(),
        name,
        author,
        arch,
        repo_name,
        repo_url
    );

    match is_installed {
        true => match is_up_to_date {
            true => {
                println!("    Version: {} {}", version, "[installed]".bright_green())
            }
            false => println!(
                "    Version: {} {}",
                version,
                format!("[{} is installed]", installed_package_version).yellow()
            ),
        },
        false => println!("    Version: {}", version),
    };

    println!(
        "    Available Versions:\n      - {}\n    URL: {}\n    Description:\n      {}",
        tags.join("\n      - "),
        url,
        description
    );
}
