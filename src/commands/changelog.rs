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

// TODO: Add `changelog` command functionality

use colored::Colorize;
use std::{
    env::temp_dir,
    fs::{remove_file, File},
    io::Write,
    process::{exit, Command, Stdio},
};

use toml::Value;

use crate::utils::is_windows;

pub fn display(changelog: String, latest_only: bool) {
    let parsed_changelog = parse_changelog(&changelog, latest_only);

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

pub fn parse_changelog(changelog: &str, latest_only: bool) -> String {
    let mut returned_string = String::new();

    let changelog_toml: Value = changelog.parse().unwrap();
    let versions = changelog_toml["version"].as_array().unwrap();

    if latest_only {
        let version = versions.first().unwrap();

        let tag = version["tag"].as_str().unwrap();
        let date = version["date"].as_str().unwrap();
        let changes = version["changes"].as_str().unwrap();

        returned_string.push_str(&format!(
            "{} @ {}\n{}",
            tag.bold().blue(),
            date.yellow(),
            changes
        ));

        returned_string
    } else {
        for version in versions {
            let tag = version["tag"].as_str().unwrap();
            let date = version["date"].as_str().unwrap();
            let changes = version["changes"].as_str().unwrap();

            returned_string.push_str(&format!("{} @ {}\n{}\n\n", tag, date, changes));
        }

        let returned_string = returned_string.trim().to_string();

        returned_string
    }
}
