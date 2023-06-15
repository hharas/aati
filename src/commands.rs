use crate::structs;
use crate::utils::*;

use colored::Colorize;
use humansize::{format_size, BINARY};
use lz4::Decoder;
use lz4::EncoderBuilder;
use std::collections::HashMap;
use std::fs;
use std::fs::read_to_string;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::{copy, Write};
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

#[cfg(not(target_os = "windows"))]
use std::os::unix::prelude::PermissionsExt;

pub fn get_command(package_name: &str) {
    // Initialise some variables

    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    match extract_package(&package_name.to_string()) {
        Some(extracted_package) => {
            let repo_toml: toml::Value = get_repo_config(extracted_package[0].as_str())
                .unwrap()
                .parse()
                .unwrap();
            let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

            let mut is_installed = false;
            let mut is_found = false;
            let mut checksum = "";

            for installed_package in installed_packages {
                if installed_package["name"].as_str().unwrap() == extracted_package[1] {
                    is_installed = true;
                }
            }

            // 1. Make sure this Package isn't installed already

            if is_installed {
                println!(
                    "{}",
                    "- This Package is already installed! Did you mean: $ aati upgrade <package>"
                        .bright_red()
                );
                exit(0);
            } else {
                for available_package in available_packages {
                    if available_package["name"].as_str().unwrap() == extracted_package[1] {
                        for package_version in available_package["versions"].as_array().unwrap() {
                            if package_version["tag"].as_str().unwrap()
                                == extracted_package[2].clone()
                                && available_package["target"].as_str().unwrap() == get_target()
                            {
                                is_found = true;
                                checksum = package_version["checksum"].as_str().unwrap();
                            }
                        }
                    }
                }
            }

            // 2. Make sure this Package is found in the Repository

            if !is_found {
                println!(
                    "{}",
                    "- This Package is not found on the Repository! Try: $ aati sync".bright_red()
                );
                exit(1);
            }

            if !is_installed && is_found {
                let name = extracted_package[1].clone();
                let version = extracted_package[2].clone();

                let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();

                let url = format!(
                    "{}/{}/{}/{}-{}.lz4",
                    aati_config["sources"]["repos"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|r| r["name"].as_str().unwrap() == extracted_package[0])
                        .unwrap()["url"]
                        .as_str()
                        .unwrap(),
                    get_target(),
                    name,
                    name,
                    version
                );

                match ureq::head(&url).call() {
                    Ok(head_response) => {
                        let content_length = head_response
                            .header("Content-Length")
                            .and_then(|len| len.parse::<u64>().ok())
                            .unwrap_or(0);

                        let human_readable_size = format_size(content_length, BINARY);

                        // 3. Ask the user if he's sure that he wants to install it

                        if prompt_yn(
                            format!(
                                "/ Are you sure you want to install {}/{}-{} ({})?",
                                extracted_package[0], name, version, human_readable_size
                            )
                            .as_str(),
                        ) {
                            println!(
                                "{}",
                                format!("+ Downloading ({})...", url)
                                    .as_str()
                                    .bright_green()
                            );

                            // 4. Download the LZ4 compressed package

                            match ureq::get(url.as_str()).call() {
                                Ok(response) => {
                                    let mut reader = response.into_reader();

                                    let download_path = std::env::temp_dir()
                                        .join(format!("{}-{}.lz4", name, version));

                                    let mut downloaded_file = OpenOptions::new()
                                        .create(true)
                                        .read(true)
                                        .write(true)
                                        .open(download_path.clone())
                                        .unwrap();

                                    // 5. Save the LZ4 compressed package

                                    copy(&mut reader, &mut downloaded_file).unwrap();

                                    println!("{}", "+ Finished downloading!".bright_green());

                                    // 6. Define two readers for the LZ4 compressed package:
                                    //   - One for the checksum verification function
                                    //   - and another for the LZ4 Decoder

                                    let mut checksum_reader =
                                        File::open(download_path.clone()).unwrap();
                                    let lz4_reader = File::open(download_path.clone()).unwrap();

                                    let mut body = Vec::new();
                                    checksum_reader.read_to_end(&mut body).unwrap();

                                    // 7. Verify the SHA256 Checksum of the LZ4 compressed package

                                    if verify_checksum(&body, checksum.to_string()) {
                                        println!("{}", "+ Checksums match!".bright_green());

                                        let installation_path_buf =
                                            get_installation_path_buf(&name);

                                        let mut new_file =
                                            File::create(installation_path_buf.clone()).unwrap();

                                        // 8. Decode the LZ4 compressed package, delete it, then save the uncompressed data into ~/.local/bin/<package name>

                                        let mut decoder = Decoder::new(lz4_reader).unwrap();

                                        fs::remove_file(download_path).unwrap();

                                        copy(&mut decoder, &mut new_file).unwrap();

                                        println!(
                                            "{}",
                                            "+ Adding Package to the Lockfile...".bright_green()
                                        );

                                        // 9. Add this Package to the Lockfile

                                        let aati_lock_path_buf = get_aati_lock_path_buf();

                                        let lock_file_str =
                                            fs::read_to_string(aati_lock_path_buf.clone()).unwrap();
                                        let mut lock_file: structs::LockFile =
                                            toml::from_str(&lock_file_str).unwrap();

                                        let package = structs::Package {
                                            name,
                                            source: extracted_package[0].to_string(),
                                            version,
                                        };

                                        lock_file.package.push(package);

                                        let mut file = OpenOptions::new()
                                            .write(true)
                                            .truncate(true)
                                            .open(aati_lock_path_buf)
                                            .unwrap();

                                        let toml_str = toml::to_string(&lock_file).unwrap();
                                        file.write_all(toml_str.as_bytes()).unwrap();

                                        #[cfg(not(target_os = "windows"))]
                                        {
                                            println!(
                                                "{}",
                                                "+ Changing Permissions...".bright_green()
                                            );

                                            // 10. (non-windows only) Turn it into an executable file, simply: chmod +x ~/.local/bin/<package name>

                                            let metadata =
                                                fs::metadata(installation_path_buf.clone())
                                                    .unwrap();
                                            let mut permissions = metadata.permissions();
                                            permissions.set_mode(0o755);
                                            fs::set_permissions(installation_path_buf, permissions)
                                                .unwrap();
                                        }

                                        println!(
                                            "{}",
                                            "+ Installation is complete!".bright_green()
                                        );
                                    } else {
                                        println!(
                                            "{}",
                                            "- Checksums don't match! Installation is aborted"
                                                .bright_red()
                                        );

                                        fs::remove_file(download_path).unwrap();
                                    }
                                }

                                Err(error) => {
                                    println!(
                                        "{}",
                                        format!("- ERROR[1]: {}", error).as_str().bright_red()
                                    );
                                    exit(1);
                                }
                            };
                        } else {
                            println!("{}", "+ Transaction aborted".bright_green());
                        }
                    }

                    Err(error) => {
                        println!("{}", format!("- ERROR[0]: {}", error).as_str().bright_red());
                        exit(1);
                    }
                }
            }
        }

        None => {
            println!("{}", "- PACKAGE NOT FOUND!".bright_red());
            exit(1);
        }
    }
}

pub fn upgrade_command(choice: Option<&str>) {
    let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();
    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();

    let repos = aati_config["sources"]["repos"].as_array().unwrap();
    let mut repos_toml: Vec<toml::Value> = Vec::new();

    for repo in repos {
        repos_toml.push(
            get_repo_config(repo["name"].as_str().unwrap())
                .unwrap()
                .parse::<toml::Value>()
                .unwrap(),
        )
    }

    let installed_packages = aati_lock["package"].as_array().unwrap();

    if let Some(package_name) = choice {
        match extract_package(&package_name.to_string()) {
            Some(extracted_package) => {
                let mut is_installed = false;
                let mut is_up_to_date = true;

                for installed_package in installed_packages {
                    if installed_package["name"].as_str().unwrap() == extracted_package[1]
                        && installed_package["source"].as_str().unwrap() == extracted_package[0]
                    {
                        is_installed = true;
                        if installed_package["version"].as_str().unwrap() != extracted_package[2] {
                            is_up_to_date = false;
                        }
                    }
                }

                if is_installed {
                    if !is_up_to_date {
                        uninstall_command(package_name);
                        get_command(package_name);
                    } else {
                        println!("{}", "+ That Package is already up to date!".bright_green());
                        exit(1);
                    }
                } else {
                    println!("{}", "- Package not installed!".bright_red());
                    exit(1);
                }
            }

            None => {
                println!("{}", "- PACKAGE NOT FOUND!".bright_red());
                exit(1);
            }
        }
    } else {
        let mut to_be_upgraded: Vec<&str> = Vec::new();

        println!("{}", "+ Packages to be upgraded:".bright_green());

        if !installed_packages.is_empty() {
            for installed_package in installed_packages {
                if installed_package["source"].as_str().unwrap() != "local" {
                    let available_packages = repos_toml
                        .iter()
                        .find(|r| r["repo"]["name"] == installed_package["source"])
                        .unwrap()["index"]["packages"]
                        .as_array()
                        .unwrap();

                    for available_package in available_packages {
                        if installed_package["name"] == available_package["name"]
                            && available_package["target"].as_str().unwrap() == get_target()
                            && installed_package["version"] != available_package["current"]
                        {
                            to_be_upgraded.push(available_package["name"].as_str().unwrap());

                            println!(
                                "{}   {}/{}-{} -> {}",
                                "+".bright_green(),
                                installed_package["source"].as_str().unwrap(),
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap(),
                                available_package["current"].as_str().unwrap(),
                            );
                        }
                    }
                }
            }

            if !to_be_upgraded.is_empty() {
                if prompt_yn("/ Are you sure you want to continue this Transaction?") {
                    for package in to_be_upgraded {
                        uninstall_command(package);
                        get_command(package);
                    }

                    println!("{}", "+ Finished upgrading!".bright_green());
                } else {
                    println!("{}", "+ Transaction aborted".bright_green());
                }
            } else {
                println!("{}", "+   None!".bright_green());
                println!("{}", "+ It's all up-to-date!".bright_green());
            }
        } else {
            println!("{}", "+   None!".bright_green());
            println!(
                "{}",
                "- You have no packages installed to upgrade!".bright_red()
            );
            exit(1);
        }
    }
}

pub fn uninstall_command(package_name: &str) {
    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    if package_name != "--all" {
        let mut is_installed = false;
        let mut package: &toml::Value = &toml::Value::from(
            "name = \"dummy-package\"\nsource = \"$unprovided$\"\nversion = \"0.1.0\"",
        );

        for installed_package in installed_packages {
            if installed_package["name"].as_str().unwrap() == package_name {
                package = installed_package;
                is_installed = true;
            }
        }

        if is_installed {
            if prompt_yn(
                format!(
                    "/ Are you sure you want to completely uninstall {}/{}-{}?",
                    package["source"].as_str().unwrap(),
                    package_name,
                    package["version"].as_str().unwrap()
                )
                .as_str(),
            ) {
                println!(
                    "{}",
                    format!("+ Deleting '{}' binary...", package_name)
                        .as_str()
                        .bright_green()
                );

                let path = get_installation_path_buf(package_name);

                match fs::remove_file(path) {
                    Ok(_) => {
                        println!(
                            "{}",
                            "+ Removing package from the Lockfile...".bright_green()
                        );

                        let aati_lock_path_buf = get_aati_lock_path_buf();

                        let lock_file_str = fs::read_to_string(aati_lock_path_buf.clone()).unwrap();
                        let mut lock_file: structs::LockFile =
                            toml::from_str(&lock_file_str).unwrap();

                        lock_file
                            .package
                            .retain(|package| package.name != package_name);

                        let mut file = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(&aati_lock_path_buf)
                            .unwrap();

                        let toml_str = toml::to_string_pretty(&lock_file).unwrap();
                        file.write_all(toml_str.as_bytes()).unwrap();

                        println!(
                            "{}",
                            "+ Uninstallation finished successfully!".bright_green()
                        );
                    }

                    Err(error) => {
                        println!(
                            "{}",
                            format!(
                                "- COULD NOT DELETE {}'S BINARY! ERROR[2]: {}",
                                package_name, error
                            )
                            .as_str()
                            .bright_red()
                        );
                        exit(1);
                    }
                }
            } else {
                println!("{}", "+ Transaction aborted".bright_green());
            }
        } else {
            println!("{}", "- This Package is not installed!".bright_red());
            exit(0);
        }
    } else if !installed_packages.is_empty() {
        if prompt_yn("Are you sure you want to uninstall all of your packages?") {
            for package in installed_packages {
                uninstall_command(package["name"].as_str().unwrap());
            }
        } else {
            println!("{}", "+ Transaction aborted".bright_green());
        }
    } else {
        println!("{}", "+ You have no packages installed!".bright_green());
    }
}

pub fn list_command(choice_option: Option<&str>) {
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

pub fn sync_command() {
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

                        let mut repo_config = File::create(repo_config_path_buf.clone())
                            .unwrap_or_else(|_| {
                                panic!("- UNABLE TO CREATE {}!", repo_config_path_buf.display())
                            });

                        println!(
                            "{}",
                            format!(
                                "+   Writing Repo Config to {}",
                                repo_config_path_buf.display()
                            )
                            .bright_green()
                        );

                        writeln!(repo_config, "{}", repo_toml).unwrap();

                        println!(
                            "{}",
                            format!("+   Synced with ({}) successfully!", url).bright_green()
                        );
                    }

                    Err(error) => {
                        println!(
                            "{}",
                            format!(
                                "- UNABLE TO REQUEST ({})! ERROR[5]: {}",
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
                    "- ERROR[8]: UNABLE TO PARSE INFO FROM {}! TRY: aati repo <repo url>",
                    aati_config_path_buf.display()
                )
                .bright_red()
            );
            exit(1);
        }
    }
}

pub fn repo_command(first_argument_option: Option<&str>, second_argument_option: Option<&str>) {
    if let Some(first_argument) = first_argument_option {
        let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();

        let aati_config_path_buf = get_aati_config_path_buf();

        if first_argument == "init" {
            let repo_name = prompt("* What will be the Repository's name (i.e. <name>/package)?");
            let repo_maintainer = prompt("* What's the name of its Maintainer?");
            let repo_description = prompt("* What's the Description of the Repository?");

            fs::create_dir_all("aati_repo").unwrap();
            let mut file = File::create("aati_repo/repo.toml").unwrap();

            fs::create_dir_all("aati_repo/x86-64-linux").unwrap();
            fs::create_dir_all("aati_repo/x86-64-linux/dummy-package").unwrap();
            fs::create_dir_all("aati_repo/aarch64-linux").unwrap();
            fs::create_dir_all("aati_repo/aarch64-linux/dummy-package").unwrap();

            let dummy1_path =
                PathBuf::from("aati_repo/x86-64-linux/dummy-package/dummy-package-0.1.0");
            let dummy2_path =
                PathBuf::from("aati_repo/x86-64-linux/dummy-package/dummy-package-0.1.1");
            let dummy3_path =
                PathBuf::from("aati_repo/aarch64-linux/dummy-package/dummy-package-0.1.0");
            let dummy4_path =
                PathBuf::from("aati_repo/aarch64-linux/dummy-package/dummy-package-0.1.1");

            let mut dummy1 = File::create(dummy1_path.clone()).unwrap();
            let mut dummy2 = File::create(dummy2_path.clone()).unwrap();
            let mut dummy3 = File::create(dummy3_path.clone()).unwrap();
            let mut dummy4 = File::create(dummy4_path.clone()).unwrap();

            dummy1
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.0 for x86-64 linux machines\"")
                .unwrap();
            dummy2
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.1 for x86-64 linux machines\"")
                .unwrap();
            dummy3
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.0 for aarch64 linux machines\"")
                .unwrap();
            dummy4
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.1 for aarch64 linux machines\"")
                .unwrap();

            package_command(format!("{}", dummy1_path.display()).as_str());
            package_command(format!("{}", dummy2_path.display()).as_str());
            package_command(format!("{}", dummy3_path.display()).as_str());
            package_command(format!("{}", dummy4_path.display()).as_str());

            fs::remove_file(dummy1_path).unwrap();
            fs::remove_file(dummy2_path).unwrap();
            fs::remove_file(dummy3_path).unwrap();
            fs::remove_file(dummy4_path).unwrap();

            let contents = format!("[repo]
name = \"{}\"
maintainer = \"{}\"
description = \"{}\"

[index]
packages = [
    {{ name = \"dummy-package\", current = \"0.1.1\", target = \"aarch64-linux\", versions = [
        {{ tag = \"0.1.0\", checksum = \"fd54f3db9f9b001d836654dec8b50a3f76f9003e5b86afc9fb0e2ef42c98a935\" }},
        {{ tag = \"0.1.1\", checksum = \"41a5dbe93c5641969374a2c369d486168d28fa6e5049730770f72a64c83afd61\" }},
    ], author = \"{}\", description = \"Aati Dummy Package. This is a Package created as a template.\", url = \"https://github.com/hharas/aati\" }},
    {{ name = \"dummy-package\", current = \"0.1.1\", target = \"x86-64-linux\", versions = [
        {{ tag = \"0.1.0\", checksum = \"f9a604403a4838e5e9ac64db85ac6dc6f08c0d27889a151ab3d349bc84b9c881\" }},
        {{ tag = \"0.1.1\", checksum = \"7b191ce2d53733d5b02d8740c9975346c33287ab74d7c7c7831df43aefdfddfc\" }},
    ], author = \"{}\", description = \"Aati Dummy Package. This is a Package created as a template.\", url = \"https://github.com/hharas/aati\" }},
]
", repo_name, repo_maintainer, repo_description, repo_maintainer, repo_maintainer);

            file.write_all(contents.as_bytes()).unwrap();

            println!(
                "{}",
                "+ The Repo is done! Now you can add your packages".bright_green()
            );
        } else if first_argument == "list" {
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
        } else if first_argument == "add" {
            if let Some(second_argument) = second_argument_option {
                let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();
                let added_repos = aati_config["sources"]["repos"].as_array().unwrap();

                let mut is_added = false;

                for added_repo in added_repos {
                    if added_repo["url"].as_str().unwrap() == second_argument {
                        is_added = true;
                    }
                }

                if !is_added {
                    println!(
                        "{}",
                        format!("+ Adding ({}) as a package repository", second_argument)
                            .bright_green()
                    );

                    let requested_url = format!("{}/repo.toml", second_argument);
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

                            let mut repo_config =
                                File::create(repo_config_path_buf.clone()).unwrap();

                            println!(
                                "{}",
                                format!(
                                    "+ Writing Repo Config to {}",
                                    repo_config_path_buf.display()
                                )
                                .bright_green()
                            );

                            writeln!(repo_config, "{}", repo_toml).unwrap();

                            // Putting it in rc.toml

                            println!("{}", "+ Adding URL to the Config File...".bright_green());

                            let config_file_str = get_aati_config().unwrap();

                            let mut config_file: structs::ConfigFile =
                                toml::from_str(&config_file_str).unwrap();

                            let repo = structs::Repo {
                                name: repo_name.to_string(),
                                url: second_argument.to_string(),
                            };

                            config_file.sources.repos.push(repo);

                            let aati_config_path_buf = get_aati_config_path_buf();

                            let mut file = OpenOptions::new()
                                .write(true)
                                .truncate(true)
                                .open(aati_config_path_buf)
                                .unwrap();

                            let toml_str = toml::to_string(&config_file).unwrap();
                            file.write_all(toml_str.as_bytes()).unwrap();

                            println!(
                                "{}",
                                "+ The Repository is added successfully!".bright_green()
                            );
                        }

                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- UNABLE TO REQUEST ({})! ERROR[6]: {}",
                                    requested_url, error
                                )
                                .bright_red()
                            );
                            exit(1);
                        }
                    }
                } else {
                    println!("{}", "- This Repository is already added!".bright_red());
                    exit(1);
                }
            } else {
                println!("{}", "- No Repository URL is given!".bright_red());
                exit(1)
            }
        } else if first_argument == "remove" {
            if let Some(second_argument) = second_argument_option {
                let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();
                let added_repos = aati_config["sources"]["repos"].as_array().unwrap();

                let mut is_added = false;
                let mut repo: &toml::Value =
                    &toml::Value::from("name = \"dummy-repo\"\nurl = \"http://localhost:8000\"");

                for added_repo in added_repos {
                    if added_repo["name"].as_str().unwrap() == second_argument {
                        repo = added_repo;
                        is_added = true;
                    }
                }

                if is_added {
                    if prompt_yn(format!("Are you sure you want to remove '{}' from your added package repositories?", second_argument).as_str()) {
                        println!("{}", format!("+ Removing '{}' from the Config File...", second_argument).bright_green());

                        let config_file_str =
                            fs::read_to_string(aati_config_path_buf.clone()).unwrap();
                        let mut config_file: structs::ConfigFile =
                            toml::from_str(&config_file_str).unwrap();

                        config_file.sources.repos.retain(|r| {
                            r.name != repo["name"].as_str().unwrap()
                                && r.url != repo["url"].as_str().unwrap()
                        });

                        let mut file = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(&aati_config_path_buf)
                            .unwrap();

                        let toml_str = toml::to_string_pretty(&config_file).unwrap();
                        file.write_all(toml_str.as_bytes()).unwrap();

                        let repo_path_buf = get_repo_config_path_buf(second_argument);

                        println!("{}", format!("+ Deleting '{}'...", repo_path_buf.display()).bright_green());

                        fs::remove_file(repo_path_buf).unwrap();

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
            } else {
                println!("{}", "- No repo name?".bright_red());
            }
        } else if first_argument == "info" {
            if let Some(second_argument) = second_argument_option {
                let aati_config = get_aati_config().unwrap();
                let aati_toml: toml::Value = aati_config.parse().unwrap();

                let repos = aati_toml["sources"]["repos"].as_array().unwrap();

                let repo_config = get_repo_config(second_argument).unwrap();
                let repo_toml: toml::Value = repo_config.parse().unwrap();

                let url = repos
                    .iter()
                    .find(|r| r["name"].as_str().unwrap() == second_argument)
                    .unwrap()["url"]
                    .as_str()
                    .unwrap();

                let maintainer = repo_toml["repo"]["maintainer"].as_str().unwrap();
                let description = repo_toml["repo"]["description"].as_str().unwrap();
                let packages_number = repo_toml["index"]["packages"].as_array().unwrap().len();

                println!(
                    "{}\n    Name: {}\n    URL: {}\n    Maintainer: {}\n    Number of Packages: {}\n    Description:\n      {}",
                    "+ Repository Information:".bright_green(),
                    second_argument, url, maintainer, packages_number, description
                );
            } else {
                println!("{}", "- No repository name?".bright_red());
                exit(1);
            }
        } else {
            println!("{}", "- Unknown Argument!".bright_red());
            exit(1);
        }
    } else {
    }
}

pub fn info_command(text: &str, repo_name: Option<&str>) {
    // Initialising main variables
    let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();
    let repos = aati_config["sources"]["repos"].as_array().unwrap();

    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    // Some placeholders too
    let mut is_installed = false;
    let mut is_up_to_date = false;
    let mut installed_package_version = "0.0.0";

    if !text.contains('/') {
        let mut results: Vec<Vec<toml::Value>> = Vec::new();

        if let Some(repo_name) = repo_name {
            let repo_toml: toml::Value = get_repo_config(repo_name).unwrap().parse().unwrap();
            let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

            for available_package in available_packages {
                if available_package["name"].as_str().unwrap() == text
                    && available_package["target"].as_str().unwrap() == get_target()
                {
                    results.push(vec![
                        available_package.clone(),
                        toml::Value::from_str(
                            format!(
                                "name = \"{}\"\nurl = \"{}\"",
                                repo_name,
                                repos
                                    .iter()
                                    .find(|r| r["name"].as_str().unwrap() == repo_name)
                                    .unwrap()["url"]
                                    .as_str()
                                    .unwrap()
                            )
                            .as_str(),
                        )
                        .unwrap(),
                    ]);
                }
            }
        } else {
            for repo in repos {
                let repo_name = repo["name"].as_str().unwrap();

                let repo_toml: toml::Value = get_repo_config(repo_name).unwrap().parse().unwrap();
                let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

                for available_package in available_packages {
                    if available_package["name"].as_str().unwrap() == text
                        && available_package["target"].as_str().unwrap() == get_target()
                    {
                        results.push(vec![
                            available_package.clone(),
                            toml::Value::from_str(
                                format!(
                                    "name = \"{}\"\nurl = \"{}\"",
                                    repo_name,
                                    repo["url"].as_str().unwrap()
                                )
                                .as_str(),
                            )
                            .unwrap(),
                        ]);
                    }
                }
            }
        }

        if !results.is_empty() {
            if results.len() == 1 {
                let package = results[0][0].clone();
                let repo_name = results[0][1]["name"].as_str().unwrap();
                let repo_url = results[0][1]["url"].as_str().unwrap();

                // Check if it's installed / up-to-date

                for installed_package in installed_packages {
                    if installed_package["name"] == package["name"]
                        && installed_package["source"].as_str().unwrap() == repo_name
                    {
                        installed_package_version = installed_package["version"].as_str().unwrap();

                        is_installed = true;
                        if installed_package["version"] == package["current"] {
                            is_up_to_date = true;
                        }
                    }
                }

                // Display!

                display_package(
                    package,
                    repo_name,
                    repo_url,
                    is_installed,
                    is_up_to_date,
                    installed_package_version,
                );
            } else {
                let conflicts: Vec<_> = results
                    .iter()
                    .enumerate()
                    .map(|(i, value)| {
                        [
                            (i + 1).to_string(),
                            value[0]["name"].as_str().unwrap().to_string(),
                            value[1]["name"].as_str().unwrap().to_string(),
                        ]
                    })
                    .collect();

                println!(
                    "{}",
                    "+ This Package exists with the same name in multiple repositories:".yellow()
                );

                for conflict in conflicts.clone() {
                    println!(
                        "{}    ({}) {}/{}",
                        "+".yellow(),
                        conflict[0],
                        conflict[2],
                        conflict[1],
                    );
                }

                let input = prompt("* Enter the number of the package you choose:");

                match input.parse::<usize>() {
                    Ok(response) => {
                        let mut is_valid = false;

                        for conflict in conflicts {
                            if conflict[0] == response.to_string() {
                                is_valid = true;
                            }
                        }

                        if is_valid {
                            let package = results[response - 1][0].clone();
                            let repo_name = results[response - 1][1]["name"].as_str().unwrap();
                            let repo_url = results[response - 1][1]["url"].as_str().unwrap();

                            for installed_package in installed_packages {
                                if installed_package["name"] == package["name"]
                                    && installed_package["source"].as_str().unwrap() == repo_name
                                {
                                    is_installed = true;
                                    if installed_package["version"] == package["current"] {
                                        is_up_to_date = true;
                                        installed_package_version =
                                            installed_package["version"].as_str().unwrap()
                                    }
                                }
                            }

                            // Display!

                            display_package(
                                package,
                                repo_name,
                                repo_url,
                                is_installed,
                                is_up_to_date,
                                installed_package_version,
                            );
                        } else {
                            println!("{}", "- INVALID CHOICE!".bright_red());
                            exit(1);
                        }
                    }

                    Err(error) => {
                        println!(
                            "{}",
                            format!("- UNABLE TO PARSE INPUT! ERROR[9]: {}", error).bright_red()
                        );
                        exit(1);
                    }
                }
            }
        } else {
            println!("{}", "- Package not found!".bright_red());
            exit(1);
        }
    } else {
        let (repo_name, text_to_be_extracted) = text.split_once('/').unwrap();

        info_command(text_to_be_extracted, Some(repo_name));
    }
}

pub fn package_command(filename: &str) {
    let source = PathBuf::from(filename);
    let destination = PathBuf::from(format!("{}.lz4", filename));

    match File::open(source) {
        Ok(mut input_file) => {
            println!(
                "{}",
                format!("+ Packaging the '{}' binary...", filename).bright_green()
            );
            let output_file = File::create(destination.clone()).unwrap();

            let mut encoder = EncoderBuilder::new().level(16).build(output_file).unwrap();

            println!("{}", "+ Writing the compressed buffer...".bright_green());

            io::copy(&mut input_file, &mut encoder).unwrap();

            let (_output, _result) = encoder.finish();

            println!(
                "{}",
                format!("+ Done packaging! See: {}", destination.display()).bright_green()
            );
        }

        Err(error) => {
            println!("{}", format!("- ERROR[7]: {}", error).bright_red());
            exit(1);
        }
    }
}

pub fn install_command(filename: &str) {
    let filename_path_buf = PathBuf::from(filename);

    let parsed_package = parse_filename(filename_path_buf.file_name().unwrap().to_str().unwrap());
    let source = parsed_package.source.as_str();
    let name = parsed_package.name.as_str();
    let version = parsed_package.version.as_str();

    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    if !installed_packages
        .iter()
        .any(|pkg| pkg["name"].as_str().unwrap() == name)
    {
        match File::open(filename_path_buf) {
            Ok(input_file) => {
                if prompt_yn(
                    format!(
                        "/ Are you sure you want to locally install {}-{}?",
                        name, version
                    )
                    .as_str(),
                ) {
                    println!("{}", "+ Decoding LZ4...".bright_green());

                    let installation_path_buf = get_installation_path_buf(name);

                    let mut new_file = File::create(installation_path_buf.clone()).unwrap();

                    let mut decoder = Decoder::new(input_file).unwrap();

                    println!("{}", "+ Copying package executable...".bright_green());

                    copy(&mut decoder, &mut new_file).unwrap();

                    println!("{}", "+ Adding package to the Lockfile...".bright_green());

                    let aati_lock_path_buf = get_aati_lock_path_buf();

                    let lock_file_str = fs::read_to_string(aati_lock_path_buf.clone()).unwrap();
                    let mut lock_file: structs::LockFile = toml::from_str(&lock_file_str).unwrap();

                    let package = structs::Package {
                        name: name.to_string(),
                        source: source.to_string(),
                        version: version.to_string(),
                    };

                    lock_file.package.push(package);

                    let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(aati_lock_path_buf)
                        .unwrap();

                    let toml_str = toml::to_string(&lock_file).unwrap();
                    file.write_all(toml_str.as_bytes()).unwrap();

                    #[cfg(not(target_os = "windows"))]
                    {
                        println!("{}", "+ Changing Permissions...".bright_green());

                        let metadata = fs::metadata(installation_path_buf.clone()).unwrap();
                        let mut permissions = metadata.permissions();
                        permissions.set_mode(0o755);
                        fs::set_permissions(installation_path_buf, permissions).unwrap();
                    }

                    println!("{}", "+ Installation is complete!".bright_green());
                } else {
                    println!("{}", "+ Transaction aborted".bright_green());
                }
            }

            Err(error) => {
                println!("{}", format!("- ERROR[11]: {}", error).bright_red());
                exit(1);
            }
        }
    } else {
        println!(
            "{}",
            "- A Package with the same name is already installed!".bright_red()
        );
        exit(1);
    }
}

pub fn generate_command() {
    match read_to_string("repo.toml") {
        Ok(repo_toml) => match repo_toml.parse::<toml::Value>() {
            Ok(repo_config) => {
                let website_url =
                    prompt("On what URL will this index be hosted (e.g. http://example.com)?");
                let repo_url = prompt("On what URL is the package repository hosted?");

                let available_packages = repo_config["index"]["packages"].as_array().unwrap();
                let packages_folder = PathBuf::from("packages");

                let mut html_files: HashMap<PathBuf, String> = HashMap::new();

                html_files.insert(
                    PathBuf::from("index.html"),
                    generate_apr_html(repo_config.clone(), "index", None, &website_url, &repo_url),
                );

                html_files.insert(
                    PathBuf::from("packages.html"),
                    generate_apr_html(
                        repo_config.clone(),
                        "packages",
                        None,
                        &website_url,
                        &repo_url,
                    ),
                );

                html_files.insert(
                    PathBuf::from("about.html"),
                    generate_apr_html(repo_config.clone(), "about", None, &website_url, &repo_url),
                );

                if !available_packages.is_empty() {
                    if !packages_folder.exists() {
                        fs::create_dir_all("packages").unwrap();
                        fs::create_dir_all("packages/x86-64-linux").unwrap();
                        fs::create_dir_all("packages/x86-64-windows").unwrap();
                        fs::create_dir_all("packages/aarch64-linux").unwrap();
                        fs::create_dir_all("packages/aarch64-windows").unwrap();
                    }

                    for package in available_packages {
                        html_files.insert(
                            PathBuf::from(format!(
                                "packages/{}/{}.html",
                                package["target"].as_str().unwrap(),
                                package["name"].as_str().unwrap(),
                            )),
                            generate_apr_html(
                                repo_config.clone(),
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
                            println!("{}", format!("ERROR[14]: {}", error).bright_red());
                            exit(1);
                        }
                    };

                    file.write_all(filehtml.as_bytes()).unwrap();
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
