use colored::Colorize;
use std::{
    collections::HashMap,
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::PathBuf,
    process::exit,
};

use super::common::{generate_apr_html, prompt};

pub fn command() {
    match read_to_string("repo.toml") {
        Ok(repo_toml) => match repo_toml.parse::<toml::Value>() {
            Ok(repo_config) => {
                let website_url =
                    prompt("On what URL will this index be hosted (e.g. http://example.com)?");
                let repo_url = prompt("On what URL is the package repository hosted?");

                let available_packages = repo_config["index"]["packages"].as_array().unwrap();
                let targets = vec![
                    "x86_64-linux",
                    "aarch64-linux",
                    "x86_64-windows",
                    "aarch64-windows",
                ];

                let mut html_files: HashMap<PathBuf, String> = HashMap::new();

                html_files.insert(
                    PathBuf::from("index.html"),
                    generate_apr_html(&repo_config, "index", None, &website_url, &repo_url),
                );

                html_files.insert(
                    PathBuf::from("packages.html"),
                    generate_apr_html(&repo_config, "packages", None, &website_url, &repo_url),
                );

                html_files.insert(
                    PathBuf::from("about.html"),
                    generate_apr_html(&repo_config, "about", None, &website_url, &repo_url),
                );

                if !available_packages.is_empty() {
                    for package in available_packages {
                        for target in &targets {
                            if available_packages
                                .iter()
                                .any(|pkg| &pkg["target"].as_str().unwrap() == target)
                            {
                                let target_directory = PathBuf::from(target);

                                if !target_directory.exists() {
                                    create_dir_all(format!(
                                        "{}/{}",
                                        target_directory.display(),
                                        package["name"].as_str().unwrap()
                                    ))
                                    .unwrap();
                                }

                                html_files.insert(
                                    PathBuf::from(format!("{}/index.html", target,)),
                                    generate_apr_html(
                                        &repo_config,
                                        target,
                                        None,
                                        &website_url,
                                        &repo_url,
                                    ),
                                );
                            }
                        }

                        html_files.insert(
                            PathBuf::from(format!(
                                "{}/{}/{}.html",
                                package["target"].as_str().unwrap(),
                                package["name"].as_str().unwrap(),
                                package["name"].as_str().unwrap(),
                            )),
                            generate_apr_html(
                                &repo_config,
                                "package",
                                Some(package),
                                &website_url,
                                &repo_url,
                            ),
                        );
                    }
                }

                for (filepath, filehtml) in html_files {
                    let mut file = match File::create(&filepath) {
                        Ok(file) => file,
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO CREATE FILE '{}'! ERROR[14]: {}",
                                    filepath.display(),
                                    error
                                )
                                .bright_red()
                            );
                            exit(1);
                        }
                    };

                    match file.write_all(filehtml.as_bytes()) {
                        Ok(_) => {}
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO WRITE INTO FILE '{}'! ERROR[87]: {}",
                                    filepath.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    }

                    println!(
                        "{}",
                        format!("+ Written {}", filepath.display()).bright_green()
                    );
                }
            }

            Err(error) => {
                println!("{}", format!("ERROR[12]: {}", error).bright_red());
                exit(1);
            }
        },

        Err(error) => {
            println!("{}", format!("ERROR[13]: {}", error).bright_red());
            exit(1);
        }
    }
}
