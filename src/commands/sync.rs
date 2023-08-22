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

use std::{fs::File, io::Write, process::exit};

use crate::utils::{
    check_aati_dirs, get_aati_config, get_aati_config_path_buf, get_repo_config_path_buf,
};

use colored::Colorize;
use toml::Value;

pub fn command(repo_names_option: Option<Vec<String>>, quiet: bool) {
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();

    match aati_config
        .get("sources")
        .and_then(|sources| sources.get("repos"))
        .and_then(|repos| repos.as_array())
    {
        Some(repos) => {
            let mut selected_repos = Vec::new();

            if let Some(repo_names) = repo_names_option {
                for repo_name in repo_names {
                    if let Some(found_repo) = repos
                        .iter()
                        .find(|r| r["name"].as_str().unwrap() == repo_name)
                    {
                        selected_repos.push(found_repo.to_owned());
                    } else if !quiet {
                        println!("{}", format!("+ Repository '{}' ignored due to not being found in the config file", repo_name).yellow());
                    }
                }
            } else {
                selected_repos.extend(repos.to_owned());
            }

            for repo in selected_repos {
                let url = repo["url"].as_str().unwrap();
                let requested_url = format!("{}/repo.toml", url);

                if !quiet {
                    println!(
                        "{}",
                        format!("+ Requesting ({})", requested_url).bright_green()
                    );
                }

                match ureq::get(requested_url.as_str()).call() {
                    Ok(repo_toml) => {
                        let repo_toml = repo_toml.into_string().unwrap();

                        let repo_value: Value = repo_toml.parse().unwrap();

                        let repo_name = repo_value["repo"]["name"].as_str().unwrap();

                        check_aati_dirs();

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

                        if !quiet {
                            println!(
                                "{}",
                                format!(
                                    "+   Writing Repo Config to {}",
                                    repo_config_path_buf.display()
                                )
                                .bright_green()
                            );
                        }

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

                        if !quiet {
                            println!(
                                "{}",
                                format!("+   Synced with ({}) successfully!", url).bright_green()
                            );
                        }
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
