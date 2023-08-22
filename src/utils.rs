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
use dirs::home_dir;
use std::{
    collections::HashMap,
    fs::{copy, create_dir_all, read_to_string, remove_file, File},
    io::{stdin, stdout, Write},
    path::PathBuf,
    process::{exit, Command, Stdio},
};
use toml::Value;

use super::types::Package;
use crate::{
    globals::{
        AATI_DIRNAME, BIN_DIRNAME, CONFIG_FILENAME, LIB_DIRNAME, LOCK_FILENAME, REPOS_DIRNAME,
    },
    types::Pkgfile,
};

pub fn is_windows() -> bool {
    std::env::consts::OS == "windows"
}

pub fn get_target() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

pub fn is_supported(target: &str) -> bool {
    target == get_target() || target == "any"
}

pub fn check_aati_dirs() {
    let home_dir = home_dir().unwrap();

    let aati_dir = home_dir.join(AATI_DIRNAME);
    let repos_dir = aati_dir.join(REPOS_DIRNAME);
    let bin_dir = aati_dir.join(BIN_DIRNAME);
    let lib_dir = aati_dir.join(LIB_DIRNAME);

    if !aati_dir.exists() {
        match create_dir_all(&aati_dir) {
            Ok(_) => {}

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE DIRECTORY '{}'! ERROR[20]: {}",
                        &aati_dir.display(),
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

    if !bin_dir.exists() {
        match create_dir_all(&bin_dir) {
            Ok(_) => {}

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE DIRECTORY '{}'! ERROR[22]: {}",
                        &bin_dir.display(),
                        error
                    )
                    .bright_red()
                );
                exit(1);
            }
        }
    }

    if !lib_dir.exists() {
        match create_dir_all(&lib_dir) {
            Ok(_) => {}

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE DIRECTORY '{}'! ERROR[23]: {}",
                        &lib_dir.display(),
                        error
                    )
                    .bright_red()
                );
                exit(1);
            }
        }
    }
}

pub fn get_bin_path_buf() -> PathBuf {
    check_aati_dirs();

    let home_dir = home_dir().unwrap();

    home_dir.join(AATI_DIRNAME).join(BIN_DIRNAME)
}

pub fn get_lib_path_buf() -> PathBuf {
    check_aati_dirs();

    let home_dir = home_dir().unwrap();

    home_dir.join(AATI_DIRNAME).join(LIB_DIRNAME)
}

pub fn get_aati_config_path_buf() -> PathBuf {
    check_aati_dirs();

    let home_dir = home_dir().unwrap();

    home_dir.join(AATI_DIRNAME).join(CONFIG_FILENAME)
}

pub fn get_aati_lock_path_buf() -> PathBuf {
    check_aati_dirs();

    let home_dir = home_dir().unwrap();

    home_dir.join(AATI_DIRNAME).join(LOCK_FILENAME)
}

pub fn get_repo_config_path_buf(repo_name: &str) -> PathBuf {
    check_aati_dirs();

    let home_dir = home_dir().unwrap();

    home_dir
        .join(AATI_DIRNAME)
        .join(REPOS_DIRNAME)
        .join(format!("{}.toml", repo_name))
}

pub fn get_aati_lock() -> String {
    let aati_lock_path_buf = get_aati_lock_path_buf();

    if !aati_lock_path_buf.exists() {
        let mut aati_lock_file = match File::create(&aati_lock_path_buf) {
            Ok(file) => file,
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE FILE '{}'! ERROR[22]: {}",
                        &aati_lock_path_buf.display(),
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
                        &aati_lock_path_buf.display(),
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
                format!(
                    "+ Make sure to add '{}' to PATH and '{}' to LD_LIBRARY_PATH.
  You can do this by appending these two lines at the end of your .bashrc file:
    export PATH=\"$PATH:$HOME/.aati/bin\"
    export LD_LIBRARY_PATH=\"$LD_LIBRARY_PATH:$HOME/.aati/lib\"",
                    get_bin_path_buf().display(),
                    get_lib_path_buf().display()
                )
                .yellow()
            );
        } else {
            println!(
                "{}",
                format!(
                    "+ Make sure to add '{}' and '{}' to %PATH%.",
                    get_bin_path_buf().display(),
                    get_lib_path_buf().display()
                )
                .yellow()
            );
        }
    }

    let aati_lock = match read_to_string(&aati_lock_path_buf) {
        Ok(content) => content,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO READ FILE '{}'! ERROR[23]: {}",
                    aati_lock_path_buf.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    aati_lock.trim().into()
}

pub fn get_repo_config(repo_name: &str) -> String {
    let repo_config_path_buf = get_repo_config_path_buf(repo_name);

    if !repo_config_path_buf.exists() {
        println!(
            "{}",
            format!(
                "- Could not find repository manifest at '{}'! Try: $ aati repo add <repo url>",
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

    repo_config.trim().into()
}

pub fn get_aati_config() -> Option<String> {
    let aati_config_path_buf = get_aati_config_path_buf();

    if !aati_config_path_buf.exists() {
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

    let aati_config = match read_to_string(&aati_config_path_buf) {
        Ok(content) => content,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO READ FILE '{}'! ERROR[28]: {}",
                    aati_config_path_buf.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    Some(aati_config.trim().into())
}

pub fn prompt(prompt_text: &str) -> String {
    print!("{}", format!("{} ", prompt_text).as_str().bright_blue());
    stdout().flush().unwrap();

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

    input.trim().into()
}

pub fn prompt_yn(prompt_text: &str) -> bool {
    print!("{}", format!("{} [Y/n] ", prompt_text).as_str().yellow());
    stdout().flush().unwrap();

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
pub fn extract_package(text: &str, added_repos: &Vec<Value>) -> Option<Vec<String>> {
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
                                && is_supported(available_package["target"].as_str().unwrap())
                            {
                                results.push(Package {
                                    name: available_package["name"].as_str().unwrap().into(),
                                    version: available_package["versions"]
                                        .as_array()
                                        .unwrap()
                                        .first()
                                        .unwrap()["tag"]
                                        .as_str()
                                        .unwrap()
                                        .into(),
                                    source: added_repo["repo"]["name"].as_str().unwrap().into(),
                                    target: available_package["target"].as_str().unwrap().into(),
                                    pkgfile: Pkgfile {
                                        data: HashMap::new(),
                                        installation_lines: vec![],
                                        win_installation_lines: vec![],
                                        removal_lines: vec![],
                                        win_removal_lines: vec![],
                                    },
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
                                && is_supported(available_package["target"].as_str().unwrap())
                            {
                                results.push(Package {
                                    name: available_package["name"].as_str().unwrap().into(),
                                    version: available_package["versions"]
                                        .as_array()
                                        .unwrap()
                                        .first()
                                        .unwrap()["tag"]
                                        .as_str()
                                        .unwrap()
                                        .into(),
                                    source: added_repo["repo"]["name"].as_str().unwrap().into(),
                                    target: available_package["target"].as_str().unwrap().into(),
                                    pkgfile: Pkgfile {
                                        data: HashMap::new(),
                                        installation_lines: vec![],
                                        win_installation_lines: vec![],
                                        removal_lines: vec![],
                                        win_removal_lines: vec![],
                                    },
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
                                        && is_supported(
                                            available_package["target"].as_str().unwrap(),
                                        )
                                    {
                                        results.push(Package {
                                            name: available_package["name"]
                                                .as_str()
                                                .unwrap()
                                                .into(),
                                            version: version.into(),
                                            source: added_repo["repo"]["name"]
                                                .as_str()
                                                .unwrap()
                                                .into(),
                                            target: available_package["target"]
                                                .as_str()
                                                .unwrap()
                                                .into(),
                                            pkgfile: Pkgfile {
                                                data: HashMap::new(),
                                                installation_lines: vec![],
                                                win_installation_lines: vec![],
                                                removal_lines: vec![],
                                                win_removal_lines: vec![],
                                            },
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
                            && is_supported(available_package["target"].as_str().unwrap())
                        {
                            results.push(Package {
                                name: available_package["name"].as_str().unwrap().into(),
                                version: available_package["versions"]
                                    .as_array()
                                    .unwrap()
                                    .first()
                                    .unwrap()["tag"]
                                    .as_str()
                                    .unwrap()
                                    .into(),
                                source: added_repo["repo"]["name"].as_str().unwrap().into(),
                                target: available_package["target"].as_str().unwrap().into(),
                                pkgfile: Pkgfile {
                                    data: HashMap::new(),
                                    installation_lines: vec![],
                                    win_installation_lines: vec![],
                                    removal_lines: vec![],
                                    win_removal_lines: vec![],
                                },
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
                            && is_supported(available_package["target"].as_str().unwrap())
                        {
                            results.push(Package {
                                name: available_package["name"].as_str().unwrap().into(),
                                version: available_package["versions"]
                                    .as_array()
                                    .unwrap()
                                    .first()
                                    .unwrap()["tag"]
                                    .as_str()
                                    .unwrap()
                                    .into(),
                                source: added_repo["repo"]["name"].as_str().unwrap().into(),
                                target: available_package["target"].as_str().unwrap().into(),
                                pkgfile: Pkgfile {
                                    data: HashMap::new(),
                                    installation_lines: vec![],
                                    win_installation_lines: vec![],
                                    removal_lines: vec![],
                                    win_removal_lines: vec![],
                                },
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
                                    && is_supported(available_package["target"].as_str().unwrap())
                                {
                                    results.push(Package {
                                        name: available_package["name"].as_str().unwrap().into(),
                                        version: version.into(),
                                        source: added_repo["repo"]["name"].as_str().unwrap().into(),
                                        target: available_package["target"]
                                            .as_str()
                                            .unwrap()
                                            .into(),
                                        pkgfile: Pkgfile {
                                            data: HashMap::new(),
                                            installation_lines: vec![],
                                            win_installation_lines: vec![],
                                            removal_lines: vec![],
                                            win_removal_lines: vec![],
                                        },
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

                let repo_toml: &Value = added_repos
                    .iter()
                    .find(|repo| repo["repo"]["name"].as_str().unwrap() == repo_name)
                    .unwrap();

                let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                if name == version {
                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == name
                            && is_supported(available_package["target"].as_str().unwrap())
                        {
                            version = available_package["versions"]
                                .as_array()
                                .unwrap()
                                .first()
                                .unwrap()["tag"]
                                .as_str()
                                .unwrap();
                        }
                    }
                } else if !version.chars().next().unwrap().is_ascii_digit() {
                    name = text;

                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == text
                            && is_supported(available_package["target"].as_str().unwrap())
                        {
                            version = available_package["versions"]
                                .as_array()
                                .unwrap()
                                .first()
                                .unwrap()["tag"]
                                .as_str()
                                .unwrap();
                        }
                    }
                }

                Some(vec![
                    repo_name.into(),
                    name.into(),
                    version.into(),
                    found_package.target.clone(),
                ])
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
                            pkg.target.clone(),
                        ]
                    })
                    .collect();
                println!(
                    "{}",
                    format!(
                        "+ Package '{}' exists with the same name in multiple repositories:",
                        name
                    )
                    .yellow()
                );
                for conflict in conflicts.clone() {
                    println!(
                        "{}    ({}) {}/{}-{}-{}",
                        "+".yellow(),
                        conflict[0],
                        conflict[3],
                        conflict[1],
                        conflict[2],
                        conflict[4]
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
                                result_package[4].clone(),
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
                        result_package.target.clone(),
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
    let repo_toml = format!("[repo]
name = \"testing\"
maintainer = \"Husayn Haras\"
description = \"APR made for testing the extract_package() function\"

[index]
packages = [
    {{ name = \"testing-package\", target = \"any\", versions = [
        {{ tag = \"0.1.0\", checksum = \"checksum-placeholder\" }}
    ], author = \"Husayn Haras\", description = \"Package made to test the extract_package() function\", url = \"https://sr.ht/~haras/aati\" }},
    {{ name = \"calculator\", target = \"{}\", versions = [
        {{ tag = \"0.1.1\", checksum = \"checksum-placeholder\" }},
        {{ tag = \"0.1.0\", checksum = \"checksum-placeholder\" }},
    ], author = \"Husayn Haras\", description = \"Package made to test the extract_package() function\", url = \"https://sr.ht/~haras/aati\" }},
]", get_target());

    let repo_config: Value = repo_toml.parse().unwrap();
    let added_repos = vec![repo_config];

    assert_eq!(
        extract_package("calculator", &added_repos),
        Some(vec![
            "testing".into(),
            "calculator".into(),
            "0.1.1".into(),
            get_target().into()
        ])
    );

    assert_eq!(
        extract_package("calculator-0.1.0", &added_repos),
        Some(vec![
            "testing".into(),
            "calculator".into(),
            "0.1.0".into(),
            get_target().into()
        ])
    );

    assert_eq!(
        extract_package("calculator-0.1.1", &added_repos),
        Some(vec![
            "testing".into(),
            "calculator".into(),
            "0.1.1".into(),
            get_target().into()
        ])
    );

    assert_eq!(
        extract_package("testing/calculator", &added_repos),
        Some(vec![
            "testing".into(),
            "calculator".into(),
            "0.1.1".into(),
            get_target().into()
        ])
    );

    assert_eq!(
        extract_package("testing/calculator-0.1.0", &added_repos),
        Some(vec![
            "testing".into(),
            "calculator".into(),
            "0.1.0".into(),
            get_target().into()
        ])
    );

    assert_eq!(
        extract_package("testing/calculator-0.1.1", &added_repos),
        Some(vec![
            "testing".into(),
            "calculator".into(),
            "0.1.1".into(),
            get_target().into()
        ])
    );

    assert_eq!(
        extract_package("testing-package", &added_repos),
        Some(vec![
            "testing".into(),
            "testing-package".into(),
            "0.1.0".into(),
            "any".into()
        ])
    );

    assert_eq!(
        extract_package("testing-package-0.1.0", &added_repos),
        Some(vec![
            "testing".into(),
            "testing-package".into(),
            "0.1.0".into(),
            "any".into()
        ])
    );

    assert_eq!(
        extract_package("testing/testing-package", &added_repos),
        Some(vec![
            "testing".into(),
            "testing-package".into(),
            "0.1.0".into(),
            "any".into()
        ])
    );

    assert_eq!(
        extract_package("testing/testing-package-0.1.0", &added_repos),
        Some(vec![
            "testing".into(),
            "testing-package".into(),
            "0.1.0".into(),
            "any".into()
        ])
    );

    assert_eq!(extract_package("unknown-package", &added_repos), None);
}

pub fn make_executable(_installation_path_buf: &PathBuf) {
    #[cfg(not(target_os = "windows"))]
    {
        use std::{
            fs::{metadata, set_permissions},
            os::unix::prelude::PermissionsExt,
        };

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

pub fn parse_pkgfile(pkgfile: &str) -> Pkgfile {
    let mut meta_lines: Vec<String> = Vec::new();
    let mut installation_lines = Vec::new();
    let mut win_installation_lines = Vec::new();
    let mut removal_lines = Vec::new();
    let mut win_removal_lines = Vec::new();

    let mut data: HashMap<String, String> = HashMap::new();

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

        if current_section == "[data]" {
            meta_lines.push(trimmed_line.into());
        } else if current_section == "[installation]" {
            installation_lines.push(trimmed_line.into());
        } else if current_section == "[win-installation]" {
            win_installation_lines.push(trimmed_line.into());
        } else if current_section == "[removal]" {
            removal_lines.push(trimmed_line.into());
        } else if current_section == "[win-removal]" {
            win_removal_lines.push(trimmed_line.into());
        }
    }

    for line in meta_lines {
        let tokens: Vec<&str> = line.split_whitespace().collect();

        data.insert(tokens[0].into(), line[tokens[0].len() + 1..].into());
    }

    Pkgfile {
        data,
        installation_lines,
        win_installation_lines,
        removal_lines,
        win_removal_lines,
    }
}

pub fn execute_lines(
    lines: &Vec<String>,
    data: &HashMap<String, String>,
    package_directory_path_buf: Option<&PathBuf>,
    quiet: bool,
) {
    for line in lines {
        let mut line = line
            .replace("$bin_dir", get_bin_path_buf().to_str().unwrap())
            .replace("$lib_dir", get_lib_path_buf().to_str().unwrap())
            .replace("$home_dir", home_dir().unwrap().to_str().unwrap());

        for (key, value) in data {
            line = line.replace(&format!("${}", key), value);
        }

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
                let mut command = if !is_windows() {
                    Command::new("sh")
                } else {
                    Command::new("cmd.exe")
                };

                if !is_windows() {
                    command.arg("-c")
                } else {
                    command.arg("/C")
                };

                let mut line_copy = line.clone();

                command.arg(line_copy.split_off(7)).stderr(Stdio::inherit());

                if !quiet {
                    command.stdout(Stdio::inherit());
                }

                if let Some(package_directory_path_buf) = package_directory_path_buf {
                    command.current_dir(package_directory_path_buf);
                }

                match command.output() {
                    Ok(output) => output,
                    Err(error) => {
                        println!(
                            "{}",
                            format!(
                                "- FAILED RUNNING COMMAND: '{}'! GIVEN ERROR: {}",
                                line, error
                            )
                            .bright_red()
                        );
                        exit(1);
                    }
                };
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
}

pub fn get_package_versions(package_name: &str) -> Option<Vec<Value>> {
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();
    let repo_list = aati_config["sources"]["repos"].as_array().unwrap();
    let mut added_repos: Vec<Value> = Vec::new();
    let mut versions: Vec<Value> = Vec::new();

    for repo_info in repo_list {
        added_repos.push(
            get_repo_config(repo_info["name"].as_str().unwrap())
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
