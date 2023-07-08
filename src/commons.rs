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
use dirs::home_dir;
use ring::digest;
use std::{
    fs::{copy, create_dir_all, read_to_string, remove_dir_all, remove_file, File},
    io::{stdin, stdout, Write},
    path::{Path, PathBuf},
    process::{exit, Command},
};

use crate::globals::VALID_TARGETS;

use super::types::Package;

pub fn is_windows() -> bool {
    std::env::consts::OS == "windows"
}

pub fn get_target() -> String {
    let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);

    if VALID_TARGETS.iter().any(|t| t == &target) {
        target
    } else {
        println!("- Unsupported Instruction Set Architecture / Operating System!");
        exit(1);
    }
}

pub fn check_config_dir() {
    let config_dir = if !is_windows() {
        home_dir().unwrap().join(".config")
    } else {
        home_dir().unwrap().join("Aati\\Binaries")
    };
    let aati_config_dir = if !is_windows() {
        home_dir().unwrap().join(".config/aati")
    } else {
        home_dir().unwrap().join("Aati")
    };
    let repos_dir = if !is_windows() {
        home_dir().unwrap().join(".config/aati/repos")
    } else {
        home_dir().unwrap().join("Aati\\Repositories")
    };

    if !config_dir.exists() {
        match create_dir_all(&config_dir) {
            Ok(_) => {}

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE DIRECTORY '{}'! ERROR[19]: {}",
                        &config_dir.display(),
                        error
                    )
                    .bright_red()
                );
                exit(1);
            }
        }
    }

    if !aati_config_dir.exists() {
        match create_dir_all(&aati_config_dir) {
            Ok(_) => {}

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE DIRECTORY '{}'! ERROR[20]: {}",
                        &aati_config_dir.display(),
                        error
                    )
                    .bright_red()
                );
                exit(1);
            }
        }
    }

    if !repos_dir.exists() {
        match create_dir_all(&repos_dir) {
            Ok(_) => {}

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE DIRECTORY '{}'! ERROR[21]: {}",
                        &repos_dir.display(),
                        error
                    )
                    .bright_red()
                );
                exit(1);
            }
        }
    }
}

pub fn get_aati_lock() -> Option<String> {
    check_config_dir();

    let aati_lock_path_buf;
    let aati_lock_path;

    if !is_windows() {
        aati_lock_path_buf = home_dir().unwrap().join(".config/aati/lock.toml");

        aati_lock_path = aati_lock_path_buf.as_path();
    } else {
        aati_lock_path_buf = home_dir().unwrap().join("Aati\\Lock.toml");

        aati_lock_path = aati_lock_path_buf.as_path();
    }

    if !aati_lock_path.exists() {
        let mut aati_lock_file = match File::create(aati_lock_path) {
            Ok(file) => file,
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE FILE '{}'! ERROR[22]: {}",
                        &aati_lock_path.display(),
                        error
                    )
                    .bright_red()
                );

                exit(1);
            }
        };

        let default_config = "package = []";
        match writeln!(aati_lock_file, "{}", default_config) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO WRITE INTO FILE '{}'! ERROR[24]: {}",
                        &aati_lock_path.display(),
                        error
                    )
                    .bright_red()
                );

                exit(1);
            }
        }

        if !is_windows() {
            println!(
                "{}",
                "+ Make sure to add ~/.local/bin to PATH and ~/.local/lib to LD_LIBRARY_PATH.
  You can do this by appending these two lines at the end of your .bashrc file:
    export PATH=\"$PATH:$HOME/.local/bin\"
    export LD_LIBRARY_PATH=\"$LD_LIBRARY_PATH:$HOME/.local/lib\""
                    .yellow()
            );
        } else {
            println!(
                "{}",
                format!(
                    "+ Make sure to add {}\\Aati\\Binaries to %PATH%.",
                    home_dir().unwrap().display()
                )
                .yellow()
            );
        }
    }

    let aati_lock = match read_to_string(aati_lock_path) {
        Ok(content) => content,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO READ FILE '{}'! ERROR[23]: {}",
                    &aati_lock_path.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    Some(aati_lock.trim().to_string())
}

pub fn get_repo_config(repo_name: &str) -> Option<String> {
    check_config_dir();

    let repo_config_path_buf = if !is_windows() {
        home_dir()
            .unwrap()
            .join(format!(".config/aati/repos/{}.toml", repo_name))
    } else {
        home_dir()
            .unwrap()
            .join(format!("Aati\\Repositories\\{}.toml", repo_name))
    };

    if !repo_config_path_buf.exists() {
        println!(
            "{}",
            format!(
                "- NO REPO CONFIG FOUND IN '{}'! PLEASE RUN: $ aati repo <repo url>",
                repo_config_path_buf.display()
            )
            .bright_red()
        );
        exit(1)
    }

    let repo_config = match read_to_string(&repo_config_path_buf) {
        Ok(content) => content,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO READ FILE '{}'! ERROR[25]: {}",
                    repo_config_path_buf.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    Some(repo_config.trim().to_string())
}

pub fn get_aati_config() -> Option<String> {
    check_config_dir();

    let aati_config_path_buf;
    let aati_config_path;

    if !is_windows() {
        aati_config_path_buf = home_dir().unwrap().join(".config/aati/rc.toml");

        aati_config_path = Path::new(&aati_config_path_buf);
    } else {
        aati_config_path_buf = home_dir().unwrap().join("Aati\\Config.toml");

        aati_config_path = Path::new(&aati_config_path_buf);
    }

    if !aati_config_path.exists() {
        let mut aati_config_file = match File::create(&aati_config_path_buf) {
            Ok(file) => file,
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE FILE '{}'! ERROR[26]: {}",
                        &aati_config_path_buf.display(),
                        error
                    )
                    .bright_red()
                );

                exit(1);
            }
        };

        let default_config = "[sources]\nrepos = []";

        // writeln!(aati_config_file, "{}", default_config).unwrap();

        match writeln!(aati_config_file, "{}", default_config) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO WRITE INTO FILE '{}'! ERROR[27]: {}",
                        &aati_config_path_buf.display(),
                        error
                    )
                    .bright_red()
                );

                exit(1);
            }
        }
    }

    // let aati_config = read_to_string(aati_config_path).unwrap();

    let aati_config = match read_to_string(aati_config_path) {
        Ok(content) => content,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO READ FILE '{}'! ERROR[28]: {}",
                    aati_config_path.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    Some(aati_config.trim().to_string())
}

fn flush_output() {
    match stdout().flush() {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO FLUSH THE STANDARD OUTPUT! ERROR[34]: {}",
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    }
}

pub fn prompt(prompt_text: &str) -> String {
    print!("{}", format!("{} ", prompt_text).as_str().bright_blue());
    flush_output();

    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Ok(_) => {}

        Err(error) => {
            println!(
                "{}",
                format!("- DIDN'T RECEIVE VALID INPUT! ERROR[3]: {}", error).bright_red()
            );
            exit(1)
        }
    };

    input.trim().to_string()
}

pub fn prompt_yn(prompt_text: &str) -> bool {
    print!("{}", format!("{} [Y/n] ", prompt_text).as_str().yellow());
    flush_output();

    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Ok(_) => {}

        Err(error) => {
            println!(
                "{}",
                format!("- DIDN'T RECEIVE VALID INPUT! ERROR[4]: {}", error).bright_red()
            );
            exit(1)
        }
    };

    input.trim().is_empty() || input.trim().to_lowercase() == "y"
}

// This function goes hard. Feel free to copy & paste.
pub fn extract_package(text: &str, added_repos: &Vec<toml::Value>) -> Option<Vec<String>> {
    let mut repo_name = "$unprovided$";
    let mut name;
    let mut version;
    let mut text_to_be_extracted = text;

    if !text.contains('/') {
        let (found_name, found_version) = text.rsplit_once('-').unwrap_or((text, text));
        name = found_name;
        version = found_version;
    } else {
        let (source, text_to_be_splitted) = text.split_once('/').unwrap();
        let (mut found_name, mut found_version) = text_to_be_splitted
            .rsplit_once('-')
            .unwrap_or((text_to_be_splitted, text_to_be_splitted));

        if !found_version.chars().next().unwrap().is_ascii_digit() {
            found_name = text_to_be_splitted;
            found_version = text_to_be_splitted;
        }

        name = found_name;
        version = found_version;
        repo_name = source;
        text_to_be_extracted = text_to_be_splitted;
    }

    if !name.is_empty() || !version.is_empty() {
        // Searching for conflicts
        let mut results: Vec<Package> = Vec::new();

        if !added_repos.is_empty() {
            if repo_name == "$unprovided$" {
                if name == version {
                    for added_repo in added_repos {
                        let available_packages =
                            added_repo["index"]["packages"].as_array().unwrap();

                        for available_package in available_packages {
                            if available_package["name"].as_str().unwrap() == text
                                && available_package["target"].as_str().unwrap() == get_target()
                            {
                                results.push(Package {
                                    name: available_package["name"].as_str().unwrap().to_string(),
                                    version: available_package["current"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    source: added_repo["repo"]["name"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    removal: vec!["$uninitialised$".to_string()],
                                })
                            }
                        }
                    }
                } else if !version.chars().next().unwrap().is_ascii_digit() {
                    name = text_to_be_extracted;

                    for added_repo in added_repos {
                        let available_packages =
                            added_repo["index"]["packages"].as_array().unwrap();

                        for available_package in available_packages {
                            if available_package["name"].as_str().unwrap() == text
                                && available_package["target"].as_str().unwrap() == get_target()
                            {
                                results.push(Package {
                                    name: available_package["name"].as_str().unwrap().to_string(),
                                    version: available_package["current"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    source: added_repo["repo"]["name"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    removal: vec!["$uninitialised$".to_string()],
                                })
                            }
                        }
                    }
                } else {
                    for added_repo in added_repos {
                        let available_packages =
                            added_repo["index"]["packages"].as_array().unwrap();

                        for available_package in available_packages {
                            if available_package["name"].as_str().unwrap() == name {
                                for package_version in
                                    available_package["versions"].as_array().unwrap()
                                {
                                    if package_version["tag"].as_str().unwrap() == version
                                        && available_package["target"].as_str().unwrap()
                                            == get_target()
                                    {
                                        results.push(Package {
                                            name: available_package["name"]
                                                .as_str()
                                                .unwrap()
                                                .to_string(),
                                            version: version.to_string(),
                                            source: added_repo["repo"]["name"]
                                                .as_str()
                                                .unwrap()
                                                .to_string(),
                                            removal: vec!["$uninitialised$".to_string()],
                                        })
                                    }
                                }
                            }
                        }
                    }
                }
            } else if name == version {
                for added_repo in added_repos {
                    let available_packages = added_repo["index"]["packages"].as_array().unwrap();

                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == text_to_be_extracted
                            && available_package["target"].as_str().unwrap() == get_target()
                        {
                            results.push(Package {
                                name: available_package["name"].as_str().unwrap().to_string(),
                                version: available_package["current"].as_str().unwrap().to_string(),
                                source: added_repo["repo"]["name"].as_str().unwrap().to_string(),
                                removal: vec!["$uninitialised$".to_string()],
                            })
                        }
                    }
                }
            } else if !version.chars().next().unwrap().is_ascii_digit() {
                name = text_to_be_extracted;

                for added_repo in added_repos {
                    let available_packages = added_repo["index"]["packages"].as_array().unwrap();

                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == text_to_be_extracted
                            && available_package["target"].as_str().unwrap() == get_target()
                        {
                            results.push(Package {
                                name: available_package["name"].as_str().unwrap().to_string(),
                                version: available_package["current"].as_str().unwrap().to_string(),
                                source: added_repo["repo"]["name"].as_str().unwrap().to_string(),
                                removal: vec!["$uninitialised$".to_string()],
                            })
                        }
                    }
                }
            } else {
                for added_repo in added_repos {
                    let available_packages = added_repo["index"]["packages"].as_array().unwrap();

                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == name {
                            for package_version in available_package["versions"].as_array().unwrap()
                            {
                                if package_version["tag"].as_str().unwrap() == version
                                    && available_package["target"].as_str().unwrap() == get_target()
                                {
                                    results.push(Package {
                                        name: available_package["name"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        version: version.to_string(),
                                        source: added_repo["repo"]["name"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        removal: vec!["$uninitialised$".to_string()],
                                    })
                                }
                            }
                        }
                    }
                }
            }
        } else {
            println!(
                "{}",
                "- YOU HAVE NO REPOSITORIES SET! TRY: aati repo add <repo url>".bright_red()
            );
            exit(1);
        }

        // Check for conflicts

        if !results.is_empty() {
            if results.len() == 1 {
                let found_package = &results[0];

                let repo_name = found_package.source.as_str();

                let repo_toml: &toml::Value = added_repos
                    .iter()
                    .find(|repo| repo["repo"]["name"].as_str().unwrap() == repo_name)
                    .unwrap();

                let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                if name == version {
                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == name
                            && available_package["target"].as_str().unwrap() == get_target()
                        {
                            version = available_package["current"].as_str().unwrap();
                        }
                    }
                } else if !version.chars().next().unwrap().is_ascii_digit() {
                    name = text;

                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == text
                            && available_package["target"].as_str().unwrap() == get_target()
                        {
                            version = available_package["current"].as_str().unwrap();
                        }
                    }
                }

                let name_string = name.to_string();
                let version_string = version.to_string();

                Some(vec![repo_name.to_string(), name_string, version_string])
            } else if repo_name == "$unprovided$" {
                let conflicts: Vec<_> = results
                    .iter()
                    .enumerate()
                    .map(|(i, pkg)| {
                        [
                            (i + 1).to_string(),
                            pkg.name.clone(),
                            pkg.version.clone(),
                            pkg.source.clone(),
                        ]
                    })
                    .collect();
                println!(
                    "{}",
                    "+ This Package exists with the same name in multiple repositories:".yellow()
                );
                for conflict in conflicts.clone() {
                    println!(
                        "{}    ({}) {}/{}-{}",
                        "+".yellow(),
                        conflict[0],
                        conflict[3],
                        conflict[1],
                        conflict[2]
                    );
                }
                let input = prompt("* Enter the number of the package you choose:");
                match input.parse::<usize>() {
                    Ok(response) => {
                        let mut is_valid = false;
                        for conflict in conflicts.clone() {
                            if conflict[0] == response.to_string() {
                                is_valid = true;
                            }
                        }
                        if is_valid {
                            let result_package = conflicts[response - 1].clone();
                            Some(vec![
                                result_package[3].clone(),
                                result_package[1].clone(),
                                result_package[2].clone(),
                            ])
                        } else {
                            println!("{}", "- INVALID CHOICE!".bright_red());
                            exit(1);
                        }
                    }
                    Err(error) => {
                        println!(
                            "{}",
                            format!("- FAILED TO PARSE INPUT! ERROR[10]: {}", error).bright_red()
                        );
                        exit(1);
                    }
                }
            } else {
                match results.iter().find(|pkg| pkg.source == repo_name) {
                    Some(result_package) => Some(vec![
                        result_package.source.clone(),
                        result_package.name.clone(),
                        result_package.version.clone(),
                    ]),
                    None => {
                        println!("{}", "- PACKAGE REPOSITORY NOT FOUND!".bright_red());
                        exit(1);
                    }
                }
            }
        } else {
            None
        }
    } else {
        println!("{}", "- UNEXPECTED BEHAVIOUR!".bright_red());
        exit(1);
    }
}

#[test]
fn test_extract_package() {
    let repo_toml = r#"[repo]
name = "testing"
maintainer = "Husayn Haras"
description = "APR made for testing the extract_package() function"

[index]
packages = [
    { name = "testing-package", current = "0.1.0", target = "x86_64-linux", versions = [
        { tag = "0.1.0", checksum = "checksum-placeholder" }
    ], author = "Husayn Haras", description = "Package made to test the extract_package() function", url = "https://github.com/hharas/aati" },
    { name = "calculator", current = "0.1.1", target = "x86_64-linux", versions = [
        { tag = "0.1.0", checksum = "checksum-placeholder" },
        { tag = "0.1.1", checksum = "checksum-placeholder" },
    ], author = "Husayn Haras", description = "Package made to test the extract_package() function", url = "https://github.com/hharas/aati" },
]"#;

    let repo_config: toml::Value = repo_toml.parse().unwrap();
    let added_repos = vec![repo_config];

    assert_eq!(
        extract_package("calculator", &added_repos),
        Some(vec![
            "testing".to_string(),
            "calculator".to_string(),
            "0.1.1".to_string()
        ])
    );

    assert_eq!(
        extract_package("calculator-0.1.0", &added_repos),
        Some(vec![
            "testing".to_string(),
            "calculator".to_string(),
            "0.1.0".to_string()
        ])
    );

    assert_eq!(
        extract_package("calculator-0.1.1", &added_repos),
        Some(vec![
            "testing".to_string(),
            "calculator".to_string(),
            "0.1.1".to_string()
        ])
    );

    assert_eq!(
        extract_package("testing/calculator", &added_repos),
        Some(vec![
            "testing".to_string(),
            "calculator".to_string(),
            "0.1.1".to_string()
        ])
    );

    assert_eq!(
        extract_package("testing/calculator-0.1.0", &added_repos),
        Some(vec![
            "testing".to_string(),
            "calculator".to_string(),
            "0.1.0".to_string()
        ])
    );

    assert_eq!(
        extract_package("testing/calculator-0.1.1", &added_repos),
        Some(vec![
            "testing".to_string(),
            "calculator".to_string(),
            "0.1.1".to_string()
        ])
    );

    assert_eq!(
        extract_package("testing-package", &added_repos),
        Some(vec![
            "testing".to_string(),
            "testing-package".to_string(),
            "0.1.0".to_string()
        ])
    );

    assert_eq!(
        extract_package("testing-package-0.1.0", &added_repos),
        Some(vec![
            "testing".to_string(),
            "testing-package".to_string(),
            "0.1.0".to_string()
        ])
    );

    assert_eq!(
        extract_package("testing/testing-package", &added_repos),
        Some(vec![
            "testing".to_string(),
            "testing-package".to_string(),
            "0.1.0".to_string()
        ])
    );

    assert_eq!(
        extract_package("testing/testing-package-0.1.0", &added_repos),
        Some(vec![
            "testing".to_string(),
            "testing-package".to_string(),
            "0.1.0".to_string()
        ])
    );

    assert_eq!(extract_package("unknown-package", &added_repos), None);
}

pub fn verify_checksum(body: &[u8], checksum: String) -> bool {
    let hash = digest::digest(&digest::SHA256, body);
    let hex = hex::encode(hash.as_ref());

    hex == checksum
}

pub fn display_package(
    package: toml::Value,
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

pub fn parse_filename(mut filename: &str) -> Package {
    // Example Usage: parse_filename("dummy-package-0.1.0.tar.lz4");

    filename = filename.trim();

    if filename.ends_with(".tar.lz4") {
        let package = if let Some((package, _)) = filename.rsplit_once(".tar.lz4") {
            package
        } else {
            println!(
                "{}",
                format!("- FILE '{}' HAS AN INVALID FILENAME!", filename).bright_red()
            );

            exit(1);
        };

        // package's value is now: dummy-package-0.1.0

        let (name, version) = if let Some((name, version)) = package.rsplit_once('-') {
            (name, version)
        } else {
            println!(
                "{}",
                format!(
                    "- FILE '{}' DOESN'T CONTAIN A HYPHEN AS A SEPARATOR!",
                    filename
                )
                .bright_red()
            );

            exit(1);
        };

        // Now: name = "dummy-package", version = "0.1.0"

        Package {
            name: name.to_string(),
            version: version.to_string(),
            source: "local".to_string(),
            removal: vec!["$uninitialised$".to_string()],
        } //         ^^^^^ That's the name of the repo containing locally installed packages.
    } else {
        println!(
            "{}\n{}",
            "- Unidentified file extension!".bright_red(),
            "+ Note: Only .tar.lz4 files are installable.".bright_blue()
        );
        exit(1);
    }
}

#[test]
fn test_parse_filename() {
    let filename1 = "silm-0.3.3.tar.lz4";
    let expected_result1 = Package {
        name: "silm".to_string(),
        source: "local".to_string(),
        version: "0.3.3".to_string(),
        removal: vec!["$uninitialised$".to_string()],
    };

    let filename2 = "arsil-server-0.2.1.tar.lz4";
    let expected_result2 = Package {
        name: "arsil-server".to_string(),
        source: "local".to_string(),
        version: "0.2.1".to_string(),
        removal: vec!["$uninitialised$".to_string()],
    };

    assert_eq!(parse_filename(filename1), expected_result1);
    assert_eq!(parse_filename(filename2), expected_result2);
}

pub fn get_bin_path_buf() -> PathBuf {
    let home_dir = home_dir().unwrap();

    if !is_windows() {
        let local_dir = home_dir.join(".local");
        let bin_dir = home_dir.join(".local/bin");

        if local_dir.exists() {
            match create_dir_all(&local_dir) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- FAILED TO CREATE DIRECTORY '{}'! ERROR[99]: {}",
                            local_dir.display(),
                            error
                        )
                        .bright_red()
                    );
                    exit(1);
                }
            }
        }

        if bin_dir.exists() {
            match create_dir_all(&bin_dir) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- FAILED TO CREATE DIRECTORY '{}'! ERROR[55]: {}",
                            bin_dir.display(),
                            error
                        )
                        .bright_red()
                    );
                    exit(1);
                }
            }
        }

        home_dir.join(".local/bin")
    } else {
        home_dir.join("Aati\\Binaries")
    }
}

pub fn get_lib_path_buf() -> PathBuf {
    let home_dir = home_dir().unwrap();

    if !is_windows() {
        let local_dir = home_dir.join(".local");
        let lib_dir = home_dir.join(".local/lib");

        if local_dir.exists() {
            match create_dir_all(&local_dir) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- FAILED TO CREATE DIRECTORY '{}'! ERROR[56]: {}",
                            local_dir.display(),
                            error
                        )
                        .bright_red()
                    );
                    exit(1);
                }
            }
        }

        if lib_dir.exists() {
            match create_dir_all(&lib_dir) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- FAILED TO CREATE DIRECTORY '{}'! ERROR[57]: {}",
                            lib_dir.display(),
                            error
                        )
                        .bright_red()
                    );
                    exit(1);
                }
            }
        }

        home_dir.join(".local/lib")
    } else {
        home_dir.join("Aati\\Binaries")
    }
}

pub fn get_aati_config_path_buf() -> PathBuf {
    let home_dir = home_dir().unwrap();

    if !is_windows() {
        home_dir.join(".config/aati/rc.toml")
    } else {
        home_dir.join("Aati\\Config.toml")
    }
}

pub fn get_aati_lock_path_buf() -> PathBuf {
    let home_dir = home_dir().unwrap();

    if !is_windows() {
        home_dir.join(".config/aati/lock.toml")
    } else {
        home_dir.join("Aati\\Lock.toml")
    }
}

pub fn get_repo_config_path_buf(repo_name: &str) -> PathBuf {
    let home_dir = home_dir().unwrap();

    if !is_windows() {
        home_dir.join(format!(".config/aati/repos/{}.toml", repo_name))
    } else {
        home_dir.join(format!("Aati\\Repositories\\{}.toml", repo_name))
    }
}

pub fn generate_apr_html(
    repo_config: &toml::Value,
    template: &str,
    current_package: Option<&toml::Value>,
    website_url: &str,
    repo_url: &str,
) -> String {
    let repo_name = repo_config["repo"]["name"].as_str().unwrap();
    let repo_description = repo_config["repo"]["description"].as_str().unwrap();
    let repo_maintainer = repo_config["repo"]["maintainer"].as_str().unwrap();
    let available_packages = repo_config["index"]["packages"].as_array().unwrap();

    let mut response = "<!DOCTYPE html><html lang=\"en\">".to_string();

    let mut head = "<head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><style>table, th, td { border: 1px solid black; border-collapse: collapse; } .installation_guide { background-color: #f0f0f0; }</style>"
        .to_string();

    let mut header;

    if template == "index" {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, website_url, website_url, website_url
        );

        head.push_str(&format!("<title>{}</title></head>", repo_name));
        header.push_str(&format!("<p>{}</p>", repo_description));
        header.push_str(&format!("<p>Add this Package Repository in Aati by running:</p><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati repo add {}</code>", repo_url));
    } else if template == "packages" {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, website_url, website_url, website_url
        );

        header.push_str(&format!(
            "<p>Number of packages: <b>{}</b></p>",
            available_packages.len()
        ));

        let targets = VALID_TARGETS;

        header.push_str("<ul>");
        for target in targets {
            if available_packages
                .iter()
                .any(|package| package["target"].as_str().unwrap() == target)
            {
                header.push_str(&format!(
                    "<li><code style=\"font-size: 0.9rem;\"><a href=\"{}/{}\">{}</a></code><ul>",
                    website_url, target, target
                ));
                for package in available_packages {
                    let package_name = package["name"].as_str().unwrap();
                    let package_version = package["current"].as_str().unwrap();
                    let package_target = package["target"].as_str().unwrap();
                    if target == package_target {
                        header.push_str(&format!(
                            "<li><a href=\"{}/{}/{}/{}.html\"><b>{}</b>-{}</a></li>",
                            website_url,
                            package_target,
                            package_name,
                            package_name,
                            package_name,
                            package_version,
                        ));
                    }
                }
                header.push_str("</ul></li>");
            }
        }
        header.push_str("</ul>");

        head.push_str(&format!("<title>packages - {}</title></head>", repo_name));
    } else if template == "about" {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, website_url, website_url, website_url
        );

        header.push_str(&format!(
            "<p>{}</p><p>Number of packages: <b>{}</b></p><p>Maintained by: <b>{}</b></p><hr /><p>Generated using the <a href=\"https://github.com/hharas/aati\">Aati Package Manager</a> as a hosted Aati Package Repository.</p>",
            repo_description,
            available_packages.len(),
            repo_maintainer
        ));

        head.push_str(&format!("<title>about - {}</title></head>", repo_name));
    } else if template == "package" {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, website_url, website_url, website_url
        );

        if let Some(package) = current_package {
            let package_name = package["name"].as_str().unwrap();
            let package_version = package["current"].as_str().unwrap();
            let package_target = package["target"].as_str().unwrap();
            let package_versions = package["versions"].as_array().unwrap();
            let package_author = package["author"].as_str().unwrap();
            let package_description = package["description"].as_str().unwrap();
            let package_url = package["url"].as_str().unwrap();

            header.push_str(&format!(
                "<h3>Package: <code style=\"font-size: 1rem;\">{}</code></h3>",
                package_name
            ));

            header.push_str(&format!(
                "<div class=\"installation_guide\"><p>You can install this package by:</p><ol><li>Adding this package repository to Aati by running:<br/><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati repo add {}</code></li><li>Then telling Aati to fetch it for you by running:<br /><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati get {}/{}</code></li></ol>or you can download the version you want of this package below and install it locally by running:<br /><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati install {}-<i>version</i>.tar.lz4</code></div><br />",
                repo_url,
                repo_name,
                package_name,
                package_name
            ));

            header.push_str(&format!(
                "Made by: <b>{}</b>, targeted for <b><code style=\"font-size: 0.9rem;\">{}</code></b>.",
                package_author, package_target
            ));
            header.push_str(&format!(
                "<p>Description: <b>{}</b></p>",
                package_description
            ));

            header.push_str(&format!(
                "<p>URL: <a href=\"{}\">{}</a></p>",
                package_url, package_url
            ));

            header.push_str(&format!(
                "<p>Current version: <b>{}</b></p>",
                package_version
            ));
            header.push_str("<p>Available versions:</p>");

            header.push_str("<table><tr><th>Version</th><th>SHA256 Checksum</th></tr>");
            for version in package_versions {
                let tag = version["tag"].as_str().unwrap();
                let checksum = version["checksum"].as_str().unwrap();

                header.push_str(&format!(
                    "<tr><td><a href=\"{}/{}/{}/{}-{}.tar.lz4\">{}</a></td><td>{}</td></tr>",
                    repo_url, package_target, package_name, package_name, tag, tag, checksum
                ));
            }
            header.push_str("</table>");

            head.push_str(&format!(
                "<title>{}/{}</title></head>",
                repo_name, package_name
            ));
        }
    } else {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, website_url, website_url, website_url
        );

        let target = template;

        // Borrow Checker headache, had to do all this
        let mut these_available_packages = available_packages.clone();
        let retained_available_packages: &mut Vec<toml::Value> = these_available_packages.as_mut();
        retained_available_packages.retain(|package| package["target"].as_str().unwrap() == target);

        header.push_str(&format!(
            "<p>Number of <code style=\"font-size: 0.9rem;\">{}</code>-targeted packages: <b>{}</b></p>",
            target,
            retained_available_packages.len()
        ));

        if retained_available_packages
            .iter()
            .any(|package| package["target"].as_str().unwrap() == target)
        {
            header.push_str("<ul>");
            for package in available_packages {
                let package_name = package["name"].as_str().unwrap();
                let package_version = package["current"].as_str().unwrap();
                let package_target = package["target"].as_str().unwrap();
                if target == package_target {
                    header.push_str(&format!(
                        "<li><a href=\"{}/{}/{}/{}.html\"><b>{}</b>-{}</a></li>",
                        website_url,
                        package_target,
                        package_name,
                        package_name,
                        package_name,
                        package_version,
                    ));
                }
            }
            header.push_str("</ul>");
        }

        head.push_str(&format!(
            "<title>{} packages - {}</title></head>",
            target, repo_name
        ));
    }

    response.push_str(&head);
    response.push_str(&header);
    response.push_str("</body></html>");

    response
}

pub fn make_executable(_installation_path_buf: &PathBuf) {
    #[cfg(not(target_os = "windows"))]
    {
        use std::{
            fs::{metadata, set_permissions},
            os::unix::prelude::PermissionsExt,
        };

        println!("{}", "+ Changing Permissions...".bright_green());

        // 10. (non-windows only) Turn it into an executable file, simply: chmod +x ~/.local/bin/<package name>

        let metadata = match metadata(_installation_path_buf) {
            Ok(metadata) => metadata,
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO GET METADATA OF FILE '{}'! ERROR[42]: {}",
                        &_installation_path_buf.display(),
                        error
                    )
                    .bright_red()
                );

                exit(1);
            }
        };

        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        match set_permissions(_installation_path_buf, permissions) {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO SET PERMISSIONS OF FILE '{}'! ERROR[43]: {}",
                        &_installation_path_buf.display(),
                        error
                    )
                    .bright_red()
                );

                exit(1);
            }
        }
    }
}

pub fn parse_pkgfile(pkgfile: &str) -> (Vec<String>, Vec<String>) {
    let mut installation_lines = Vec::new();
    let mut removal_lines = Vec::new();
    let mut current_section = "";

    for line in pkgfile.lines() {
        let trimmed_line = line.trim();

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }

        if trimmed_line.starts_with('[') && trimmed_line.ends_with(']') {
            current_section = trimmed_line;
            continue;
        }

        if current_section == "[installation]" {
            installation_lines.push(trimmed_line.to_string());
        } else if current_section == "[removal]" {
            removal_lines.push(trimmed_line.to_string());
        }
    }

    (installation_lines, removal_lines)
}

pub fn execute_lines(lines: Vec<String>, package_directory_path_buf: Option<&PathBuf>) {
    if prompt_yn(&format!(
        "+ Commands to be ran:\n  {}\n/ Do these commands seem safe to execute?",
        lines.join("\n  ")
    )) {
        for line in lines {
            let line = line
                .replace("$bin_dir", get_bin_path_buf().to_str().unwrap())
                .replace("$lib_dir", get_lib_path_buf().to_str().unwrap())
                .replace("$home_dir", home_dir().unwrap().to_str().unwrap());

            let tokens: Vec<&str> = line.split_whitespace().collect();

            match tokens[0] {
                "install" => {
                    if let Some(ref package_directory_path_buf) = package_directory_path_buf {
                        let mut source_path_buf = PathBuf::from(package_directory_path_buf);
                        source_path_buf.push(tokens[1]);

                        let destination = tokens[2..].join(" ");
                        let destination_path_buf = PathBuf::from(destination);

                        match copy(source_path_buf, &destination_path_buf) {
                            Ok(_) => {}
                            Err(error) => {
                                println!(
                                    "{}",
                                    format!(
                                        "- FAILED TO WRITE INTO FILE '{}'! ERROR[91]: {}",
                                        &destination_path_buf.display(),
                                        error
                                    )
                                    .bright_red()
                                );

                                exit(1);
                            }
                        }

                        make_executable(&destination_path_buf);
                    } else {
                        println!(
                            "{}",
                            format!("+ Command '{}' was ignored due to ", line).yellow()
                        );
                    }
                }

                "copy" => {
                    if let Some(ref package_directory_path_buf) = package_directory_path_buf {
                        let mut source_path_buf = PathBuf::from(package_directory_path_buf);
                        source_path_buf.push(tokens[1]);

                        let destination = tokens[2..].join(" ");
                        let destination_path_buf = PathBuf::from(destination);

                        match copy(source_path_buf, &destination_path_buf) {
                            Ok(_) => {}
                            Err(error) => {
                                println!(
                                    "{}",
                                    format!(
                                        "- FAILED TO WRITE INTO FILE '{}'! ERROR[100]: {}",
                                        &destination_path_buf.display(),
                                        error
                                    )
                                    .bright_red()
                                );

                                exit(1);
                            }
                        }
                    } else {
                        println!(
                            "{}",
                            format!("+ Command '{}' was ignored due to ", line).yellow()
                        );
                    }
                }

                "delete" => {
                    let path = &tokens[1..].join(" ");

                    match remove_file(path) {
                        Ok(_) => {}
                        Err(error) => {
                            println!(
                                "{}",
                                format!("- FAILED TO DELETE FILE {}! ERROR[92]: {}", path, error)
                                    .as_str()
                                    .bright_red()
                            );
                            exit(1);
                        }
                    }
                }

                "system" => {
                    let mut command = Command::new(tokens[1]);
                    command.args(tokens[2..].to_vec());

                    if let Some(package_directory_path_buf) = package_directory_path_buf {
                        command.current_dir(package_directory_path_buf);
                    }

                    let output = match command.output() {
                        Ok(output) => output,
                        Err(error) => {
                            println!(
                                "{}",
                                format!("- FAILED RUNNING COMMAND: '{}'! ERROR: {}", line, error)
                                    .as_str()
                                    .bright_red()
                            );
                            exit(1);
                        }
                    };

                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        println!("{}", stdout);
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        println!("{}", stderr);
                    }
                }

                _ => {
                    println!(
                        "{}",
                        format!("- INVALID PKGFILE COMMAND '{}'!", line).bright_red()
                    );

                    exit(1);
                }
            }
        }
    } else {
        println!("{}", "+ Transaction aborted".bright_green());

        if let Some(package_directory_path_buf) = package_directory_path_buf {
            match remove_dir_all(package_directory_path_buf) {
                Ok(_) => println!("{}", "+ Deleted temporary package directory".bright_green()),
                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- FAILED TO DELETE DIRECTORY '{}'! ERROR[86]: {}",
                            package_directory_path_buf.display(),
                            error
                        )
                        .as_str()
                        .bright_red()
                    );
                    exit(1);
                }
            }
        }

        exit(0);
    }
}
