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

pub fn is_windows() -> bool {
    std::env::consts::OS == "windows"
}

pub fn get_target() -> String {
    if cfg!(target_arch = "x86_64") {
        if cfg!(target_os = "windows") {
            "x86_64-windows".to_string()
        } else if cfg!(target_os = "linux") {
            "x86_64-linux".to_string()
        } else {
            "x86_64-unknown".to_string()
        }
    } else if cfg!(target_arch = "aarch64") {
        if cfg!(target_os = "windows") {
            "aarch64-windows".to_string()
        } else if cfg!(target_os = "linux") {
            "aarch64-linux".to_string()
        } else {
            "aarch64-unknown".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

pub fn check_config_dir() {
    let config_dir = if !is_windows() {
        dirs::home_dir().unwrap().join(".config")
    } else {
        PathBuf::from("C:\\Program Files\\Aati\\Binaries")
    };
    let aati_config_dir = if !is_windows() {
        dirs::home_dir().unwrap().join(".config/aati")
    } else {
        PathBuf::from("C:\\Program Files\\Aati")
    };
    let repos_dir = if !is_windows() {
        dirs::home_dir().unwrap().join(".config/aati/repos")
    } else {
        PathBuf::from("C:\\Program Files\\Aati\\Repositories")
    };

    if !config_dir.exists() {
        match fs::create_dir_all(&config_dir) {
            Ok(_) => {}

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- UNABLE TO CREATE THE '{}' DIRECTORY! ERROR[19]: {}",
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
        match fs::create_dir_all(&aati_config_dir) {
            Ok(_) => {}

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- UNABLE TO CREATE THE '{}' DIRECTORY! ERROR[20]: {}",
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
        match fs::create_dir_all(&repos_dir) {
            Ok(_) => {}

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- UNABLE TO CREATE THE '{}' DIRECTORY! ERROR[21]: {}",
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
        let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");
        aati_lock_path_buf = home_dir.join(".config/aati/lock.toml");

        aati_lock_path = aati_lock_path_buf.as_path();
    } else {
        aati_lock_path_buf = PathBuf::from("C:\\Program Files\\Aati\\Lock.toml");

        aati_lock_path = aati_lock_path_buf.as_path();
    }

    if !aati_lock_path.exists() {
        let mut aati_lock_file = match File::create(aati_lock_path) {
            Ok(file) => file,
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- UNABLE TO CREATE FILE '{}'! ERROR[22]: {}",
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
                        "- UNABLE TO WRITE INTO FILE '{}'! ERROR[24]: {}",
                        &aati_lock_path.display(),
                        error
                    )
                    .bright_red()
                );

                exit(1);
            }
        }

        if !is_windows() {
            println!("{}", "+ Make sure to add ~/.local/bin to PATH. You can do this by appending this at the end of our .bashrc file:\n\n    export PATH=\"$HOME/.local/bin:$PATH\"".yellow());
        } else {
            println!(
                "{}",
                "+ Make sure to add C:\\Program Files\\Aati\\Binaries to PATH.".yellow()
            );
        }
    }

    let aati_lock = match read_to_string(aati_lock_path) {
        Ok(content) => content,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- UNABLE TO READ FILE '{}'! ERROR[23]: {}",
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
        let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");

        home_dir.join(format!(".config/aati/repos/{}.toml", repo_name))
    } else {
        PathBuf::from(format!(
            "C:\\Program Files\\Aati\\Repositories\\{}.toml",
            repo_name
        ))
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
                    "- UNABLE TO READ FILE '{}'! ERROR[25]: {}",
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
        let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");

        aati_config_path_buf = home_dir.join(".config/aati/rc.toml");

        aati_config_path = Path::new(&aati_config_path_buf);
    } else {
        aati_config_path_buf = PathBuf::from("C:\\Program Files\\Aati\\Config.toml");

        aati_config_path = Path::new(&aati_config_path_buf);
    }

    if !aati_config_path.exists() {
        let mut aati_config_file = match File::create(&aati_config_path_buf) {
            Ok(file) => file,
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- UNABLE TO CREATE FILE '{}'! ERROR[26]: {}",
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
                        "- UNABLE TO WRITE INTO FILE '{}'! ERROR[27]: {}",
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
                    "- UNABLE TO READ FILE '{}'! ERROR[28]: {}",
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
    match io::stdout().flush() {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- UNABLE TO FLUSH THE STANDARD OUTPUT! ERROR[34]: {}",
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
        let mut results: Vec<structs::Package> = Vec::new();

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
                                results.push(structs::Package {
                                    name: available_package["name"].as_str().unwrap().to_string(),
                                    version: available_package["current"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    source: added_repo["repo"]["name"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
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
                                results.push(structs::Package {
                                    name: available_package["name"].as_str().unwrap().to_string(),
                                    version: available_package["current"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    source: added_repo["repo"]["name"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
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
                                        results.push(structs::Package {
                                            name: available_package["name"]
                                                .as_str()
                                                .unwrap()
                                                .to_string(),
                                            version: version.to_string(),
                                            source: added_repo["repo"]["name"]
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
            } else if name == version {
                for added_repo in added_repos {
                    let available_packages = added_repo["index"]["packages"].as_array().unwrap();

                    for available_package in available_packages {
                        if available_package["name"].as_str().unwrap() == text_to_be_extracted
                            && available_package["target"].as_str().unwrap() == get_target()
                        {
                            results.push(structs::Package {
                                name: available_package["name"].as_str().unwrap().to_string(),
                                version: available_package["current"].as_str().unwrap().to_string(),
                                source: added_repo["repo"]["name"].as_str().unwrap().to_string(),
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
                            results.push(structs::Package {
                                name: available_package["name"].as_str().unwrap().to_string(),
                                version: available_package["current"].as_str().unwrap().to_string(),
                                source: added_repo["repo"]["name"].as_str().unwrap().to_string(),
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
                                    results.push(structs::Package {
                                        name: available_package["name"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                        version: version.to_string(),
                                        source: added_repo["repo"]["name"]
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
                            format!("- UNABLE TO PARSE INPUT! ERROR[10]: {}", error).bright_red()
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

pub fn parse_filename(mut filename: &str) -> structs::Package {
    // Example Usage: parse_filename("dummy-package-0.1.0.lz4");

    filename = filename.trim();

    if filename.ends_with(".lz4") {
        let package = if let Some((package, _)) = filename.rsplit_once(".lz4") {
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

#[test]
fn test_parse_filename() {
    let filename1 = "silm-0.3.3.lz4";
    let expected_result1 = structs::Package {
        name: "silm".to_string(),
        source: "local".to_string(),
        version: "0.3.3".to_string(),
    };

    let filename2 = "arsil-server-0.2.1.lz4";
    let expected_result2 = structs::Package {
        name: "arsil-server".to_string(),
        source: "local".to_string(),
        version: "0.2.1".to_string(),
    };

    assert_eq!(parse_filename(filename1), expected_result1);
    assert_eq!(parse_filename(filename2), expected_result2);
}

pub fn get_installation_path_buf(filename: &str) -> PathBuf {
    let home_dir = dirs::home_dir().unwrap();
    if !is_windows() {
        home_dir.join(format!(".local/bin/{}", filename))
    } else {
        PathBuf::from(format!(
            "C:\\Program Files\\Aati\\Binaries\\{}.exe",
            filename
        ))
    }
}

pub fn get_aati_config_path_buf() -> PathBuf {
    if !is_windows() {
        let home_dir = dirs::home_dir().unwrap();
        home_dir.join(".config/aati/rc.toml")
    } else {
        PathBuf::from("C:\\Program Files\\Aati\\Config.toml")
    }
}

pub fn get_aati_lock_path_buf() -> PathBuf {
    if !is_windows() {
        let home_dir = dirs::home_dir().unwrap();
        home_dir.join(".config/aati/lock.toml")
    } else {
        PathBuf::from("C:\\Program Files\\Aati\\Lock.toml")
    }
}

pub fn get_repo_config_path_buf(repo_name: &str) -> PathBuf {
    if !is_windows() {
        let home_dir = dirs::home_dir().unwrap();
        home_dir.join(format!(".config/aati/repos/{}.toml", repo_name))
    } else {
        PathBuf::from(format!(
            "C:\\Program Files\\Aati\\Repositories\\{}.toml",
            repo_name
        ))
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

        let targets = vec![
            "x86_64-linux",
            "aarch64-linux",
            "x86_64-windows",
            "aarch64-windows",
            "x86_64-unknown",
            "aarch64-unknown",
        ];

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
                "<div class=\"installation_guide\"><p>You can install this package by:</p><ol><li>Adding this package repository to Aati by running:<br/><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati repo add {}</code></li><li>Then telling Aati to fetch it for you by running:<br /><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati get {}/{}</code></li></ol>or you can download the version you want of this package below and install it locally by running:<br /><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati install {}-<i>version</i>.lz4</code></div><br />",
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
                    "<tr><td><a href=\"{}/{}/{}/{}-{}.lz4\">{}</a></td><td>{}</td></tr>",
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
