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
use std::{
    env::temp_dir,
    fs::{remove_file, File},
    io::Write,
    process::{exit, Command, Stdio},
};

use toml::Value;

use crate::utils::{extract_package, get_aati_config, get_repo_config, is_windows};

pub fn get_package_versions(package_name: &str) -> Option<Vec<Value>> {
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();
    let repo_list = aati_config["sources"]["repos"].as_array().unwrap();
    let mut added_repos: Vec<Value> = Vec::new();
    let mut versions: Vec<Value> = Vec::new();

    for repo_info in repo_list {
        added_repos.push(
            get_repo_config(repo_info["name"].as_str().unwrap())
                .unwrap()
                .parse::<Value>()
                .unwrap(),
        );
    }

    match extract_package(package_name, &added_repos) {
        Some(package_vec) => {
            let repo_name = &package_vec[0];
            let package_name = &package_vec[1];

            let repo = added_repos
                .iter()
                .find(|r| r["repo"]["name"].as_str().unwrap() == repo_name)
                .unwrap();
            let available_packages = repo["index"]["packages"].as_array().unwrap();

            for package in available_packages {
                if package["name"].as_str().unwrap() == package_name {
                    versions = package["versions"].as_array().unwrap().to_owned();
                }
            }

            Some(versions)
        }
        None => None,
    }
}

pub fn display(changelog: &Vec<Value>, latest_only: bool) {
    let parsed_changelog = format_changelog(changelog, latest_only);

    if !latest_only {
        let mut temp_changelog_path = temp_dir();
        temp_changelog_path.push("aati-changelog.txt");

        let mut temp_changelog = match File::create(&temp_changelog_path) {
            Ok(temp_changelog) => temp_changelog,
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE FILE '{}'! ERROR[59]: {}",
                        &temp_changelog_path.display(),
                        error
                    )
                    .bright_red()
                );
                exit(1);
            }
        };

        match temp_changelog.write_all(parsed_changelog.as_bytes()) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO WRITE INTO FILE '{}'! ERROR[103]: {}",
                        &temp_changelog_path.display(),
                        error
                    )
                    .bright_red()
                );
                match remove_file(&temp_changelog_path) {
                    Ok(_) => {}
                    Err(error) => {
                        println!(
                            "{}",
                            format!(
                                "- FAILED TO DELETE FILE '{}'! ERROR[103]: {}",
                                &temp_changelog_path.display(),
                                error
                            )
                            .bright_red()
                        );
                        exit(1);
                    }
                }
                exit(1);
            }
        }

        if !is_windows() {
            Command::new("less")
                .arg("-rf")
                .arg(temp_changelog_path.as_os_str())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .unwrap();
        } else {
            Command::new("notepad")
                .arg(temp_changelog_path.as_os_str())
                .output()
                .unwrap();
        }

        match remove_file(&temp_changelog_path) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO DELETE FILE '{}'! ERROR[102]: {}",
                        &temp_changelog_path.display(),
                        error
                    )
                    .bright_red()
                );
                exit(1);
            }
        }
    } else {
        println!("{}", parsed_changelog);
    }
}

pub fn format_changelog(versions: &Vec<Value>, latest_only: bool) -> String {
    let mut returned_string = String::new();
    let mut selected_versions: Vec<&Value> = Vec::new();

    if latest_only {
        selected_versions.push(versions.first().unwrap());
    } else {
        selected_versions.extend(versions);
    }

    for version in selected_versions {
        let version_table = version.as_table().unwrap();

        let tag = version_table.get("tag").unwrap().as_str().unwrap();

        match version_table.get("date") {
            Some(date) => match version_table.get("changes") {
                Some(changes) => {
                    returned_string.push_str(&format!(
                        "{} @ {}\n{}\n\n",
                        tag.bold().blue(),
                        date.as_str().unwrap().yellow(),
                        changes.as_str().unwrap()
                    ));
                }

                None => {
                    returned_string.push_str(&format!(
                        "{} @ {}\n{}\n\n",
                        tag.bold().blue(),
                        date.as_str().unwrap().yellow(),
                        "Empty".bright_red()
                    ));
                }
            },

            None => match version_table.get("changes") {
                Some(changes) => {
                    returned_string.push_str(&format!(
                        "{}\n{}\n\n",
                        tag.bold().blue(),
                        changes.as_str().unwrap()
                    ));
                }

                None => {
                    returned_string.push_str(&format!(
                        "{}\n{}\n\n",
                        tag.bold().blue(),
                        "Empty".bright_red()
                    ));
                }
            },
        }
    }

    let returned_string = returned_string.trim().to_string();

    returned_string
}
