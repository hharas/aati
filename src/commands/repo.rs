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
    fs::{create_dir_all, read_to_string, remove_file, File, OpenOptions},
    io::Write,
    path::PathBuf,
    process::exit,
};
use toml::Value;

use crate::{
    types::{ConfigFile, Repo},
    utils::{
        check_config_dirs, get_aati_config, get_aati_config_path_buf, get_repo_config,
        get_repo_config_path_buf, prompt_yn,
    },
};

pub fn add(repository_url: String) {
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();
    let added_repos = aati_config["sources"]["repos"].as_array().unwrap();

    let mut is_added = false;

    for added_repo in added_repos {
        if added_repo["url"].as_str().unwrap() == repository_url {
            is_added = true;
        }
    }

    if !is_added {
        println!(
            "{}",
            format!("+ Adding ({}) as a package repository", repository_url).bright_green()
        );

        let requested_url = format!("{}/repo.toml", repository_url);
        println!(
            "{}",
            format!("+ Requesting ({})", requested_url).bright_green()
        );

        match ureq::get(requested_url.as_str()).call() {
            Ok(repo_toml) => {
                let repo_toml = repo_toml.into_string().unwrap();

                let repo_value: Value = repo_toml.parse().unwrap();

                let repo_name = repo_value["repo"]["name"].as_str().unwrap();

                for added_repo in added_repos {
                    if added_repo["name"].as_str().unwrap() == repo_name {
                        is_added = true;
                    }
                }

                if !is_added {
                    check_config_dirs();

                    let repo_config_path_buf = get_repo_config_path_buf(repo_name);

                    let mut repo_config = match File::create(&repo_config_path_buf) {
                        Ok(file) => file,
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO CREATE FILE '{}'! ERROR[68]: {}",
                                    &repo_config_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    };

                    println!(
                        "{}",
                        format!(
                            "+ Writing Repo Config to {}",
                            &repo_config_path_buf.display()
                        )
                        .bright_green()
                    );

                    match writeln!(repo_config, "{}", repo_toml) {
                        Ok(_) => {}
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO WRITE INTO REPO CONFIG AT '{}'! ERROR[69]: {}",
                                    repo_config_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    }

                    // Putting it in rc.toml

                    println!("{}", "+ Adding URL to the Config File...".bright_green());

                    let config_file_str = get_aati_config().unwrap();

                    let mut config_file: ConfigFile = toml::from_str(&config_file_str).unwrap();

                    let repo = Repo {
                        name: repo_name.to_string(),
                        url: repository_url.to_string(),
                    };

                    config_file.sources.repos.push(repo);

                    let aati_config_path_buf = get_aati_config_path_buf();

                    let mut file = match OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(&aati_config_path_buf)
                    {
                        Ok(file) => file,
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                "- FAILED TO OPEN CONFIG FILE AT '{}' FOR WRITING! ERROR[70]: {}",
                                &aati_config_path_buf.display(),
                                error
                            )
                                .bright_red()
                            );

                            exit(1);
                        }
                    };

                    let toml_str = toml::to_string(&config_file).unwrap();
                    match file.write_all(toml_str.as_bytes()) {
                        Ok(_) => {}
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO WRITE INTO CONFIG FILE AT '{}'! ERROR[71]: {}",
                                    aati_config_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    }

                    println!(
                        "{}",
                        "+ The Repository is added successfully!".bright_green()
                    );
                } else {
                    println!("{}", "- This Repository is already added!".bright_red());
                }
            }

            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO REQUEST ({})! ERROR[6]: {}",
                        requested_url, error
                    )
                    .bright_red()
                );
                exit(1);
            }
        }
    } else {
        println!("{}", "- This Repository is already added!".bright_red());
    }
}

pub fn remove(repo_name: String, force: bool) {
    let aati_config_path_buf = get_aati_config_path_buf();
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();
    let added_repos = aati_config["sources"]["repos"].as_array().unwrap();

    let mut is_added = false;
    let mut repo: &Value = &Value::from("name = \"dummy-repo\"\nurl = \"http://localhost:8000\"");

    for added_repo in added_repos {
        if added_repo["name"].as_str().unwrap() == repo_name {
            repo = added_repo;
            is_added = true;
        }
    }

    if is_added {
        if force
            || prompt_yn(
                format!(
                    "Are you sure you want to remove '{}' from your added package repositories?",
                    repo_name
                )
                .as_str(),
            )
        {
            println!(
                "{}",
                format!("+ Removing '{}' from the Config File...", repo_name).bright_green()
            );

            let config_file_str = match read_to_string(&aati_config_path_buf) {
                Ok(contents) => contents,
                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- FAILED TO READ CONFIG FILE AT '{}'! ERROR[72]: {}",
                            &aati_config_path_buf.display(),
                            error
                        )
                        .bright_red()
                    );

                    exit(1);
                }
            };
            let mut config_file: ConfigFile = toml::from_str(&config_file_str).unwrap();

            config_file.sources.repos.retain(|r| {
                r.name != repo["name"].as_str().unwrap() && r.url != repo["url"].as_str().unwrap()
            });

            let mut file = match OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&aati_config_path_buf)
            {
                Ok(file) => file,
                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- FAILED TO OPEN CONFIG FILE AT '{}' FOR WRITING! ERROR[73]: {}",
                            &aati_config_path_buf.display(),
                            error
                        )
                        .bright_red()
                    );

                    exit(1);
                }
            };

            let toml_str = toml::to_string_pretty(&config_file).unwrap();
            match file.write_all(toml_str.as_bytes()) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- FAILED TO WRITE INTO CONFIG FILE AT '{}'! ERROR[74]: {}",
                            aati_config_path_buf.display(),
                            error
                        )
                        .bright_red()
                    );

                    exit(1);
                }
            }

            let repo_path_buf = get_repo_config_path_buf(&repo_name);

            println!(
                "{}",
                format!("+ Deleting '{}'...", repo_path_buf.display()).bright_green()
            );

            match remove_file(&repo_path_buf) {
                Ok(_) => {}
                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- FAILED TO DELETE FILE '{}'! ERROR[79]: {}",
                            repo_path_buf.display(),
                            error
                        )
                        .bright_red()
                    );

                    exit(1);
                }
            }

            println!(
                "{}",
                format!(
                    "+ The Repository {} is removed successfully!",
                    repo["name"].as_str().unwrap()
                )
                .bright_green()
            );
        } else {
            println!("{}", "+ Transaction aborted".bright_green());
        }
    } else {
        println!(
            "{}",
            "- This Repo is not added to the Config file!".bright_red()
        );
        exit(1);
    }
}

pub fn info(repo_name: String) {
    let aati_config = get_aati_config().unwrap();
    let aati_toml: Value = aati_config.parse().unwrap();

    let repos = aati_toml["sources"]["repos"].as_array().unwrap();

    let repo_config = get_repo_config(&repo_name).unwrap();
    let repo_toml: Value = repo_config.parse().unwrap();

    let url = repos
        .iter()
        .find(|r| r["name"].as_str().unwrap() == repo_name)
        .unwrap()["url"]
        .as_str()
        .unwrap();

    let maintainer = repo_toml["repo"]["maintainer"].as_str().unwrap();
    let description = repo_toml["repo"]["description"].as_str().unwrap();
    let packages_number = repo_toml["index"]["packages"].as_array().unwrap().len();

    println!(
                    "{}\n    Name: {}\n    URL: {}\n    Maintainer: {}\n    Number of Packages: {}\n    Description:\n      {}",
                    "+ Repository Information:".bright_green(),
                    repo_name, url, maintainer, packages_number, description
                );
}

pub fn list() {
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();
    let repos = aati_config["sources"]["repos"].as_array().unwrap();

    if !repos.is_empty() {
        println!("{}", "+ Currently set package repositories:".bright_green());
        for repo in repos {
            println!(
                "{}   {} ({})",
                "+".bright_green(),
                repo["name"].as_str().unwrap(),
                repo["url"].as_str().unwrap(),
            );
        }
    } else {
        println!("{}", "+ You have no repos set!".yellow());
    }
}

pub fn init(repo_name: String, repo_maintainer: String, repo_description: String) {
    let repo_dir = PathBuf::from("aati_repo");
    let x86_64_linux_dir = PathBuf::from("aati_repo/x86_64-linux");
    let aarch64_dir = PathBuf::from("aati_repo/aarch64-linux");
    let x86_64_windows_dir = PathBuf::from("aati_repo/x86_64-windows");

    let repo_toml_path_buf = PathBuf::from("aati_repo/repo.toml");

    match create_dir_all(&repo_dir) {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO CREATE DIRECTORY '{}'! ERROR[49]: {}",
                    &repo_dir.display(),
                    error
                )
                .bright_red()
            );
            exit(1);
        }
    }

    let mut repo_toml = match File::create(&repo_toml_path_buf) {
        Ok(file) => file,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO CREATE FILE '{}'! ERROR[50]: {}",
                    &repo_toml_path_buf.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    match create_dir_all(&x86_64_linux_dir) {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO CREATE DIRECTORY '{}'! ERROR[51]: {}",
                    &x86_64_linux_dir.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    }

    match create_dir_all(&x86_64_windows_dir) {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO CREATE DIRECTORY '{}'! ERROR[52]: {}",
                    &x86_64_windows_dir.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    }

    match create_dir_all(&aarch64_dir) {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO CREATE DIRECTORY '{}'! ERROR[53]: {}",
                    &aarch64_dir.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    }

    let contents = format!("[repo]
name = \"{}\"
maintainer = \"{}\"
description = \"{}\"

[index]
packages = [
#   {{ name = \"package-name-here\", current = \"0.1.1\", target = \"x86_64-linux\", versions = [
#       {{ tag = \"0.1.1\", checksum = \"sha256-sum-here\" }},
#       {{ tag = \"0.1.0\", checksum = \"sha256-sum-here\" }},
#   ], author = \"{}\", description = \"Package description here.\", url = \"https://github.com/hharas/aati\" }},
]
", repo_name, repo_maintainer, repo_description, repo_maintainer);

    match repo_toml.write_all(contents.as_bytes()) {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO WRITE INTO FILE '{}'! ERROR[67]: {}",
                    repo_toml_path_buf.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    }

    println!(
        "{}",
        "+ The Repo is made! Now you can add your packages".bright_green()
    );
}
