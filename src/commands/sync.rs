use std::{fs::File, io::Write, process::exit};

use crate::commands::common::{
    check_config_dir, get_aati_config_path_buf, get_repo_config_path_buf,
};

use super::common::get_aati_config;
use colored::Colorize;

pub fn command() {
    let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();

    match aati_config
        .get("sources")
        .and_then(|sources| sources.get("repos"))
        .and_then(|repos| repos.as_array())
    {
        Some(repos) => {
            for repo in repos {
                let url = repo["url"].as_str().unwrap();
                let requested_url = format!("{}/repo.toml", url);

                println!(
                    "{}",
                    format!("+ Requesting ({})", requested_url).bright_green()
                );

                match ureq::get(requested_url.as_str()).call() {
                    Ok(repo_toml) => {
                        let repo_toml = repo_toml.into_string().unwrap();

                        let repo_value: toml::Value = repo_toml.parse().unwrap();

                        let repo_name = repo_value["repo"]["name"].as_str().unwrap();

                        check_config_dir();

                        let repo_config_path_buf = get_repo_config_path_buf(repo_name);

                        let mut repo_config = match File::create(&repo_config_path_buf) {
                            Ok(file) => file,
                            Err(error) => {
                                println!(
                                    "{}",
                                    format!(
                                        "- FAILED TO CREATE FILE '{}'! ERROR[88]: {}",
                                        repo_config_path_buf.display(),
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
                                "+   Writing Repo Config to {}",
                                repo_config_path_buf.display()
                            )
                            .bright_green()
                        );

                        match writeln!(repo_config, "{}", repo_toml) {
                            Ok(_) => {}
                            Err(error) => {
                                println!(
                                    "{}",
                                    format!(
                                        "- FAILED TO WRITE INTO REPO CONFIG AT '{}'! ERROR[48]: {}",
                                        &repo_config_path_buf.display(),
                                        error
                                    )
                                    .bright_red()
                                );

                                exit(1);
                            }
                        }

                        println!(
                            "{}",
                            format!("+   Synced with ({}) successfully!", url).bright_green()
                        );
                    }

                    Err(error) => {
                        println!(
                            "{}",
                            format!(
                                "- FAILED TO REQUEST ({})! ERROR[5]: {}",
                                requested_url, error
                            )
                            .bright_red()
                        );
                        exit(1);
                    }
                }
            }

            println!("{}", "+ Done syncing!".bright_green());
        }

        None => {
            let aati_config_path_buf = get_aati_config_path_buf();
            println!(
                "{}",
                format!(
                    "- ERROR[8]: FAILED TO PARSE INFO FROM {}! TRY: aati repo <repo url>",
                    aati_config_path_buf.display()
                )
                .bright_red()
            );
            exit(1);
        }
    }
}
