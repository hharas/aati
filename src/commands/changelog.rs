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
use std::{
    env::temp_dir,
    fs::{remove_file, File},
    io::Write,
    process::{exit, Command, Stdio},
};

use toml::Value;

pub fn display(changelog: &Vec<Value>, latest_only: bool) {
    let is_colored = !cfg!(windows) || latest_only;

    let parsed_changelog = format_changelog(changelog, latest_only, is_colored);

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

        if !cfg!(windows) {
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

pub fn format_changelog(versions: &Vec<Value>, latest_only: bool, is_colored: bool) -> String {
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

        if is_colored {
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
        } else {
            match version_table.get("date") {
                Some(date) => match version_table.get("changes") {
                    Some(changes) => {
                        returned_string.push_str(&format!(
                            "{} @ {}\n{}\n\n",
                            tag,
                            date.as_str().unwrap(),
                            changes.as_str().unwrap()
                        ));
                    }

                    None => {
                        returned_string.push_str(&format!(
                            "{} @ {}\n{}\n\n",
                            tag,
                            date.as_str().unwrap(),
                            "Empty"
                        ));
                    }
                },

                None => match version_table.get("changes") {
                    Some(changes) => {
                        returned_string.push_str(&format!(
                            "{}\n{}\n\n",
                            tag,
                            changes.as_str().unwrap()
                        ));
                    }

                    None => {
                        returned_string.push_str(&format!("{}\n{}\n\n", tag, "Empty"));
                    }
                },
            }
        }
    }

    let returned_string = returned_string.trim().into();

    returned_string
}
