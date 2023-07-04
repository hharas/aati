use colored::Colorize;

use crate::commands::common::{get_repo_config, get_target};

use super::common::{get_aati_config, get_aati_lock};

pub fn command(choice_option: Option<&str>) {
    let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();
    let repos = aati_config["sources"]["repos"].as_array().unwrap();

    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    if let Some(choice) = choice_option {
        if choice.to_ascii_lowercase() == "installed" {
            println!("{}", "+ Installed Packages:".bright_green());

            if !installed_packages.is_empty() {
                for installed_package in installed_packages {
                    if installed_package["source"].as_str().unwrap() != "local" {
                        match get_repo_config(installed_package["source"].as_str().unwrap())
                            .unwrap()
                            .parse::<toml::Value>()
                            .unwrap()["index"]["packages"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|pkg| {
                                pkg["name"] == installed_package["name"]
                                    && pkg["target"].as_str().unwrap() == get_target()
                                    && pkg["current"] != installed_package["version"]
                            }) {
                            Some(_) => {
                                println!(
                                    "{}   {}/{}-{} {}",
                                    "+".bright_green(),
                                    installed_package["source"].as_str().unwrap(),
                                    installed_package["name"].as_str().unwrap(),
                                    installed_package["version"].as_str().unwrap(),
                                    "[outdated]".yellow()
                                );
                            }

                            None => {
                                println!(
                                    "{}   {}/{}-{}",
                                    "+".bright_green(),
                                    installed_package["source"].as_str().unwrap(),
                                    installed_package["name"].as_str().unwrap(),
                                    installed_package["version"].as_str().unwrap()
                                );
                            }
                        }
                    }
                }

                if installed_packages
                    .iter()
                    .any(|pkg| pkg["source"].as_str().unwrap() == "local")
                {
                    println!(
                        "{}",
                        "+ Packages installed from local files:".bright_green()
                    );

                    for installed_package in installed_packages {
                        if installed_package["source"].as_str().unwrap() == "local" {
                            println!(
                                "{}   {}/{}-{}",
                                "+".bright_green(),
                                installed_package["source"].as_str().unwrap(),
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap()
                            );
                        }
                    }
                }
            } else {
                println!("  None! Install Packages using: $ aati get <package>");
            }
        } else if choice.to_ascii_lowercase() == "available" {
            let installed_packages = aati_lock["package"].as_array().unwrap();

            println!("{}", "+ Available Packages:".bright_green());

            if !repos.is_empty() {
                for repo in repos {
                    let repo_name = repo["name"].as_str().unwrap();

                    let repo_toml: toml::Value =
                        get_repo_config(repo_name).unwrap().parse().unwrap();
                    let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                    println!("{}   {}/", "+".bright_green(), repo_name);

                    for package in available_packages {
                        if package["target"].as_str().unwrap() == get_target() {
                            println!("      {}:", package["name"].as_str().unwrap());
                            let versions = package["versions"].as_array().unwrap();

                            let mut reversed_versions = versions.clone();
                            reversed_versions.reverse();

                            for version in reversed_versions {
                                let tag = version["tag"].as_str().unwrap();
                                let is_installed = installed_packages.iter().any(|installed_pkg| {
                                    installed_pkg["name"].as_str().unwrap()
                                        == package["name"].as_str().unwrap()
                                        && installed_pkg["version"].as_str().unwrap() == tag
                                        && installed_pkg["source"].as_str().unwrap() == repo_name
                                });

                                if !is_installed {
                                    println!(
                                        "        {}-{}",
                                        package["name"].as_str().unwrap(),
                                        tag
                                    );
                                } else {
                                    println!(
                                        "        {}-{} {}",
                                        package["name"].as_str().unwrap(),
                                        tag,
                                        "[installed]".bright_green()
                                    );
                                }
                            }
                        }
                    }
                }

                println!("{}", "\n+ Unsupported packages:".yellow());

                for repo in repos {
                    let repo_name = repo["name"].as_str().unwrap();

                    let repo_toml: toml::Value =
                        get_repo_config(repo_name).unwrap().parse().unwrap();
                    let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                    println!("{}   {}/", "+".yellow(), repo_name);

                    for package in available_packages {
                        if package["target"].as_str().unwrap() != get_target() {
                            println!(
                                "      {} ({}):",
                                package["name"].as_str().unwrap(),
                                package["target"].as_str().unwrap()
                            );
                            let versions = package["versions"].as_array().unwrap();

                            let mut reversed_versions = versions.clone();
                            reversed_versions.reverse();

                            for version in reversed_versions {
                                let tag = version["tag"].as_str().unwrap();

                                println!("        {}-{}", package["name"].as_str().unwrap(), tag);
                            }
                        }
                    }
                }
            } else {
                println!("    None!");
            }
        } else {
            println!("{}", format!("- Unknown choice: {}", choice).bright_red());
        }
    } else {
        println!("{}", "+ Installed Packages:".bright_green());

        if !installed_packages.is_empty() {
            for installed_package in installed_packages {
                if installed_package["source"].as_str().unwrap() != "local" {
                    match get_repo_config(installed_package["source"].as_str().unwrap())
                        .unwrap()
                        .parse::<toml::Value>()
                        .unwrap()["index"]["packages"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|pkg| {
                            pkg["name"] == installed_package["name"]
                                && pkg["target"].as_str().unwrap() == get_target()
                                && pkg["current"] != installed_package["version"]
                        }) {
                        Some(_) => {
                            println!(
                                "{}   {}/{}-{} {}",
                                "+".bright_green(),
                                installed_package["source"].as_str().unwrap(),
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap(),
                                "[outdated]".yellow()
                            );
                        }

                        None => {
                            println!(
                                "{}   {}/{}-{}",
                                "+".bright_green(),
                                installed_package["source"].as_str().unwrap(),
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap()
                            );
                        }
                    }
                }
            }

            if installed_packages
                .iter()
                .any(|pkg| pkg["source"].as_str().unwrap() == "local")
            {
                println!(
                    "{}",
                    "+ Packages installed from local files:".bright_green()
                );

                for installed_package in installed_packages {
                    if installed_package["source"].as_str().unwrap() == "local" {
                        println!(
                            "{}   {}/{}-{}",
                            "+".bright_green(),
                            installed_package["source"].as_str().unwrap(),
                            installed_package["name"].as_str().unwrap(),
                            installed_package["version"].as_str().unwrap()
                        );
                    }
                }
            }
        } else {
            println!("  None! Install Packages using: $ aati get <package>");
        }
    }
}
