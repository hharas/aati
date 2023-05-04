use std::fs;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

use colored::Colorize;
use ring::digest;

use crate::structs;

pub fn is_unix() -> bool {
    std::env::consts::FAMILY == "unix"
}

pub fn get_arch() -> String {
    if cfg!(target_arch = "x86_64") {
        "x86-64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "aarch64".to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn check_config_dir() {
    let config_dir = if is_unix() {
        dirs::home_dir().unwrap().join(".config")
    } else {
        PathBuf::from("C:\\Program Files\\Aati")
    };
    let aati_config_dir = if is_unix() {
        dirs::home_dir().unwrap().join(".config/aati")
    } else {
        PathBuf::from("C:\\Program Files\\Aati")
    };
    let repos_dir = if is_unix() {
        dirs::home_dir().unwrap().join(".config/aati/repos")
    } else {
        PathBuf::from("C:\\Program Files\\Aati\\Repositories")
    };

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).unwrap();
    }

    if !aati_config_dir.exists() {
        fs::create_dir_all(&aati_config_dir).unwrap();
    }

    if !repos_dir.exists() {
        fs::create_dir_all(&repos_dir).unwrap();
    }
}

pub fn get_aati_lock() -> Option<String> {
    check_config_dir();

    let aati_lock_path_buf;
    let aati_lock_path;

    if is_unix() {
        let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");
        aati_lock_path_buf = home_dir.join(".config/aati/lock.toml");

        aati_lock_path = aati_lock_path_buf.as_path();
    } else {
        aati_lock_path_buf = PathBuf::from("C:\\Program Files\\Aati\\Lock.toml");

        aati_lock_path = aati_lock_path_buf.as_path();
    }

    if !aati_lock_path.exists() {
        let mut aati_lock_file = File::create(aati_lock_path).unwrap();

        let default_config = "package = []";
        writeln!(aati_lock_file, "{}", default_config).unwrap();

        if is_unix() {
            println!("{}", "+ Make sure to add ~/.local/bin to PATH. You can do this by appending this at the end of our .bashrc file:\n\n    export PATH=\"$HOME/.local/bin:$PATH\"".yellow());
        } else {
            println!(
                "{}",
                "+ Make sure to add C:\\Program Files\\Aati to PATH.".yellow()
            );
        }
    }

    let aati_lock = read_to_string(aati_lock_path).unwrap();

    Some(aati_lock.trim().to_string())
}

pub fn get_repo_config(repo_name: &str) -> Option<String> {
    check_config_dir();

    let repo_config_path_buf;

    if is_unix() {
        let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");

        repo_config_path_buf = home_dir.join(format!(".config/aati/repos/{}.toml", repo_name));
    } else {
        repo_config_path_buf = PathBuf::from(format!(
            "C:\\Program Files\\Aati\\Repositories\\{}.toml",
            repo_name
        ));
    }

    if !repo_config_path_buf.exists() {
        println!(
            "{}",
            "- NO REPO CONFIG FOUND! PLEASE RUN: $ aati repo <repo url>".bright_red()
        );
        exit(1)
    }

    let repo_config = read_to_string(repo_config_path_buf).unwrap();

    Some(repo_config.trim().to_string())
}

pub fn get_aati_config() -> Option<String> {
    check_config_dir();

    let aati_config_path_buf;
    let aati_config_path;

    if is_unix() {
        let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");

        aati_config_path_buf = home_dir.join(".config/aati/rc.toml");

        aati_config_path = Path::new(&aati_config_path_buf);
    } else {
        aati_config_path_buf = PathBuf::from("C:\\Program Files\\Aati\\Config.toml");

        aati_config_path = Path::new(&aati_config_path_buf);
    }

    if !aati_config_path.exists() {
        let mut aati_config_file = File::create(aati_config_path_buf.clone()).unwrap();

        let default_config = "[sources]\nrepos = []";
        writeln!(aati_config_file, "{}", default_config).unwrap();
    }

    let aati_config = read_to_string(aati_config_path).unwrap();

    Some(aati_config.trim().to_string())
}

fn flush_output() {
    io::stdout().flush().unwrap();
}

pub fn prompt(prompt_text: &str) -> String {
    print!("{}", format!("{} ", prompt_text).as_str().bright_blue());
    flush_output();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
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
    match io::stdin().read_line(&mut input) {
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
pub fn extract_package(text: &String) -> Vec<String> {
    let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();
    let added_repos = aati_config["sources"]["repos"].as_array().unwrap();

    let mut repo_name = "$unprovided$";
    let mut name;
    let mut version;
    let mut text_to_be_extracted = text.as_str();

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
        let mut results: Vec<structs::Package> = Vec::new();

        if !added_repos.is_empty() {
            if repo_name == "$unprovided$" {
                if name == version {
                    for added_repo in added_repos {
                        let repo_str =
                            get_repo_config(added_repo["name"].as_str().unwrap()).unwrap();
                        let repo_toml: toml::Value = repo_str.parse().unwrap();
                        let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                        for available_package in available_packages {
                            if available_package["name"].as_str().unwrap() == text {
                                if available_package["arch"].as_str().unwrap() == get_arch() {
                                    results.push(structs::Package {
                                        name: available_package["name"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        version: available_package["current"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        source: added_repo["name"].as_str().unwrap().to_string(),
                                    })
                                }
                            }
                        }
                    }
                } else if !version.chars().next().unwrap().is_ascii_digit() {
                    name = text_to_be_extracted;

                    for added_repo in added_repos {
                        let repo_str =
                            get_repo_config(added_repo["name"].as_str().unwrap()).unwrap();
                        let repo_toml: toml::Value = repo_str.parse().unwrap();
                        let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                        for available_package in available_packages {
                            if available_package["name"].as_str().unwrap() == text {
                                if available_package["arch"].as_str().unwrap() == get_arch() {
                                    results.push(structs::Package {
                                        name: available_package["name"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        version: available_package["current"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        source: added_repo["name"].as_str().unwrap().to_string(),
                                    })
                                }
                            }
                        }
                    }
                } else {
                    for added_repo in added_repos {
                        let repo_str =
                            get_repo_config(added_repo["name"].as_str().unwrap()).unwrap();
                        let repo_toml: toml::Value = repo_str.parse().unwrap();
                        let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                        for available_package in available_packages {
                            if available_package["name"].as_str().unwrap() == name {
                                for package_version in
                                    available_package["versions"].as_array().unwrap()
                                {
                                    if package_version["tag"].as_str().unwrap() == version {
                                        if available_package["arch"].as_str().unwrap() == get_arch()
                                        {
                                            results.push(structs::Package {
                                                name: available_package["name"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                                version: version.to_string(),
                                                source: added_repo["name"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                            })
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                if name == version {
                    for added_repo in added_repos {
                        let repo_str =
                            get_repo_config(added_repo["name"].as_str().unwrap()).unwrap();
                        let repo_toml: toml::Value = repo_str.parse().unwrap();
                        let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                        for available_package in available_packages {
                            if available_package["name"].as_str().unwrap() == text_to_be_extracted {
                                if available_package["arch"].as_str().unwrap() == get_arch() {
                                    results.push(structs::Package {
                                        name: available_package["name"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        version: available_package["current"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        source: added_repo["name"].as_str().unwrap().to_string(),
                                    })
                                }
                            }
                        }
                    }
                } else if !version.chars().next().unwrap().is_ascii_digit() {
                    name = text_to_be_extracted;

                    for added_repo in added_repos {
                        let repo_str =
                            get_repo_config(added_repo["name"].as_str().unwrap()).unwrap();
                        let repo_toml: toml::Value = repo_str.parse().unwrap();
                        let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                        for available_package in available_packages {
                            if available_package["name"].as_str().unwrap() == text_to_be_extracted {
                                if available_package["arch"].as_str().unwrap() == get_arch() {
                                    results.push(structs::Package {
                                        name: available_package["name"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        version: available_package["current"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        source: added_repo["name"].as_str().unwrap().to_string(),
                                    })
                                }
                            }
                        }
                    }
                } else {
                    for added_repo in added_repos {
                        let repo_str =
                            get_repo_config(added_repo["name"].as_str().unwrap()).unwrap();
                        let repo_toml: toml::Value = repo_str.parse().unwrap();
                        let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                        for available_package in available_packages {
                            if available_package["name"].as_str().unwrap() == name {
                                for package_version in
                                    available_package["versions"].as_array().unwrap()
                                {
                                    if package_version["tag"].as_str().unwrap() == version {
                                        if available_package["arch"].as_str().unwrap() == get_arch()
                                        {
                                            results.push(structs::Package {
                                                name: available_package["name"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                                version: version.to_string(),
                                                source: added_repo["name"]
                                                    .as_str()
                                                    .unwrap()
                                                    .to_string(),
                                            })
                                        }
                                    }
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

                let repo_toml: toml::Value = get_repo_config(repo_name).unwrap().parse().unwrap();
                let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                if name == version {
                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == name {
                            version = available_package["current"].as_str().unwrap();
                        }
                    }
                } else if !version.chars().next().unwrap().is_ascii_digit() {
                    name = text;

                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == text
                            && available_package["arch"].as_str().unwrap() == get_arch()
                        {
                            version = available_package["current"].as_str().unwrap();
                        }
                    }
                }

                let name_string = name.to_string();
                let version_string = version.to_string();

                vec![repo_name.to_string(), name_string, version_string]
            } else {
                if repo_name == "$unprovided$" {
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
                        "+ This Package exists with the same name in multiple repositories:"
                            .yellow()
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

                                vec![
                                    result_package[3].clone(),
                                    result_package[1].clone(),
                                    result_package[2].clone(),
                                ]
                            } else {
                                println!("{}", "- INVALID CHOICE!".bright_red());
                                exit(1);
                            }
                        }

                        Err(error) => {
                            println!(
                                "{}",
                                format!("- UNABLE TO PARSE INPUT! ERROR[10]: {}", error)
                                    .bright_red()
                            );
                            exit(1);
                        }
                    }
                } else {
                    match results.iter().find(|pkg| pkg.source == repo_name) {
                        Some(result_package) => {
                            vec![
                                result_package.source.clone(),
                                result_package.name.clone(),
                                result_package.version.clone(),
                            ]
                        }

                        None => {
                            println!("{}", "- PACKAGE REPOSITORY NOT FOUND!".bright_red());
                            exit(1);
                        }
                    }
                }
            }
        } else {
            println!("{}", "- PACKAGE NOT FOUND!".bright_red());
            exit(1);
        }
    } else {
        println!("{}", "- UNEXPECTED BEHAVIOUR!".bright_red());
        exit(1);
    }
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
    let arch = package["arch"].as_str().unwrap();
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

pub fn parse_filename(mut filename: &str) -> structs::Package {
    // Example Usage: parse_filename("dummy-package-0.1.0.lz4");

    filename = filename.trim();

    if filename.ends_with(".lz4") {
        let (package, _) = filename.rsplit_once(".lz4").unwrap();

        // package's value is now: dummy-package-0.1.0

        let (name, version) = package.rsplit_once("-").unwrap();

        // Now: name = "dummy-package", version = "0.1.0"

        structs::Package {
            name: name.to_string(),
            version: version.to_string(),
            source: "local".to_string(),
        } //         ^^^^^ That's the name of the repo containing locally installed packages.
    } else {
        println!(
            "{}\n  {}",
            "- Unidentified file extension!".bright_red(),
            "Note: Only LZ4 archives are installable.".bright_blue()
        );
        exit(1);
    }
}
