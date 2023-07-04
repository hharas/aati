use crate::types::*;
use crate::utils::*;

use ascii::AsciiString;
use colored::Colorize;
use humansize::{format_size, BINARY};
use lz4::Decoder;
use lz4::EncoderBuilder;
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::remove_file;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::{copy, Write};
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use tar::Archive;
use tar::Builder;
use tiny_http::Header;
use tiny_http::{Response, Server};

pub fn get_command(package_name: &str) {
    // Initialise some variables

    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();
    let repo_list = aati_config["sources"]["repos"].as_array().unwrap();
    let mut added_repos: Vec<toml::Value> = Vec::new();

    for repo_info in repo_list {
        added_repos.push(
            get_repo_config(repo_info["name"].as_str().unwrap())
                .unwrap()
                .parse::<toml::Value>()
                .unwrap(),
        );
    }

    let installed_packages = aati_lock["package"].as_array().unwrap();

    if let Some(extracted_package) = extract_package(package_name, &added_repos) {
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
                        if package_version["tag"].as_str().unwrap() == extracted_package[2].clone()
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
                "{}/{}/{}/{}-{}.tar.lz4",
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
                                    .join(format!("{}-{}.tar.lz4", name, version));

                                let mut downloaded_file = match OpenOptions::new()
                                    .create(true)
                                    .read(true)
                                    .write(true)
                                    .open(&download_path)
                                {
                                    Ok(file) => file,
                                    Err(error) => {
                                        println!(
                                            "{}",
                                            format!(
                                                "- FAILED TO CREATE FILE '{}'! ERROR[29]: {}",
                                                &download_path.display(),
                                                error
                                            )
                                            .bright_red()
                                        );

                                        exit(1);
                                    }
                                };

                                // 5. Save the LZ4 compressed package

                                match copy(&mut reader, &mut downloaded_file) {
                                    Ok(_) => {}
                                    Err(error) => {
                                        println!(
                                            "{}",
                                            format!(
                                                "- FAILED TO WRITE INTO DOWNLOADED FILE '{}'! ERROR[30]: {}",
                                                &download_path.display(),
                                                error
                                            )
                                            .bright_red()
                                        );

                                        exit(1);
                                    }
                                }

                                println!("{}", "+ Finished downloading!".bright_green());

                                // 6. Define two readers for the LZ4 compressed package:
                                //   - One for the checksum verification function
                                //   - and another for the LZ4 Decoder

                                let mut checksum_reader = match File::open(&download_path) {
                                    Ok(file) => file,
                                    Err(error) => {
                                        println!(
                                            "{}",
                                            format!(
                                                "- FAILED TO OPEN DOWNLOADED FILE '{}' FOR READING! ERROR[31]: {}",
                                                &download_path.display(),
                                                error
                                            )
                                            .bright_red()
                                        );

                                        exit(1);
                                    }
                                };

                                let lz4_reader = match File::open(&download_path) {
                                    Ok(file) => file,
                                    Err(error) => {
                                        println!(
                                            "{}",
                                            format!(
                                                "- FAILED TO OPEN DOWNLOADED FILE '{}' FOR READING! ERROR[32]: {}",
                                                &download_path.display(),
                                                error
                                            )
                                            .bright_red()
                                        );

                                        exit(1);
                                    }
                                };

                                let mut body = Vec::new();
                                match checksum_reader.read_to_end(&mut body) {
                                    Ok(_) => {}
                                    Err(error) => {
                                        println!(
                                            "{}",
                                            format!(
                                                "- FAILED TO READ DOWNLOADED FILE '{}'! ERROR[33]: {}",
                                                &download_path.display(),
                                                error
                                            )
                                            .bright_red()
                                        );

                                        exit(1);
                                    }
                                }

                                // 7. Verify the SHA256 Checksum of the LZ4 compressed package

                                if verify_checksum(&body, checksum.to_string()) {
                                    println!("{}", "+ Checksums match!".bright_green());

                                    let mut tar_path_buf = temp_dir();
                                    tar_path_buf.push(&format!(
                                        "{}-{}.tar",
                                        extracted_package[1], extracted_package[2]
                                    ));

                                    let mut package_directory = temp_dir();
                                    package_directory.push(&format!(
                                        "{}-{}",
                                        extracted_package[1], extracted_package[2]
                                    ));

                                    let mut tarball = match File::create(&tar_path_buf) {
                                        Ok(file) => file,
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                                    "- FAILED TO CREATE FILE '{}'! ERROR[35]: {}",
                                                    tar_path_buf.display(),
                                                    error
                                                )
                                                .bright_red()
                                            );

                                            exit(1);
                                        }
                                    };

                                    // 8. Decode the LZ4 compressed package, delete it, then save the uncompressed data into the installation directory

                                    let mut decoder = match Decoder::new(lz4_reader) {
                                        Ok(decoder) => decoder,
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                            "- FAILED TO DECODE THE LZ4 COMPRESSED PACKAGE AT '{}'! ERROR[36]: {}",
                                            download_path.display(),
                                            error
                                        )
                                                .bright_red()
                                            );

                                            exit(1);
                                        }
                                    };

                                    match fs::remove_file(&download_path) {
                                        Ok(_) => {}
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                            "- FAILED TO DELETE DOWNLODED FILE '{}'! ERROR[37]: {}",
                                            download_path.display(),
                                            error
                                        )
                                                .bright_red()
                                            );

                                            exit(1);
                                        }
                                    }

                                    match copy(&mut decoder, &mut tarball) {
                                        Ok(_) => {}
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                            "- FAILED TO WRITE INTO FILE '{}'! ERROR[38]: {}",
                                            tar_path_buf.display(),
                                            error
                                        )
                                                .bright_red()
                                            );

                                            exit(1);
                                        }
                                    }

                                    let tarball = File::open(&tar_path_buf).unwrap();

                                    let mut archive = Archive::new(tarball);

                                    match archive.unpack(temp_dir()) {
                                        Ok(_) => {}
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                                    "- FAILED TO EXTRACT TARBALL '{}'! ERROR[89]: {}",
                                                    tar_path_buf.display(),
                                                    error
                                                )
                                                .bright_red()
                                            );

                                            exit(1);
                                        }
                                    }

                                    match remove_file(tar_path_buf) {
                                        Ok(_) => {}
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                                    "- COULD NOT DELETE TEMPORARY PACKAGE TARBALL! ERROR[93]: {}",
                                                    error
                                                )
                                                .as_str()
                                                .bright_red()
                                            );
                                            exit(1);
                                        }
                                    }

                                    let mut pkgfile_path_buf = package_directory.clone();
                                    pkgfile_path_buf.push("PKGFILE");

                                    let pkgfile = match fs::read_to_string(&pkgfile_path_buf) {
                                        Ok(contents) => contents,
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                                    "- FAILED TO READ FILE '{}'! ERROR[90]: {}",
                                                    pkgfile_path_buf.display(),
                                                    error
                                                )
                                                .bright_red()
                                            );

                                            exit(1);
                                        }
                                    };

                                    let (installation_lines, removal_lines) =
                                        parse_pkgfile(&pkgfile);

                                    execute_lines(installation_lines, Some(&package_directory));

                                    match remove_dir_all(package_directory) {
                                        Ok(_) => {}
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                                    "- COULD NOT DELETE TEMPORARY PACKAGE DIRECTORY! ERROR[83]: {}",
                                                    error
                                                )
                                                .as_str()
                                                .bright_red()
                                            );
                                            exit(1);
                                        }
                                    }

                                    println!(
                                        "{}",
                                        "+ Adding Package to the Lockfile...".bright_green()
                                    );

                                    // 9. Add this Package to the Lockfile

                                    let aati_lock_path_buf = get_aati_lock_path_buf();

                                    let lock_file_str =
                                        match fs::read_to_string(&aati_lock_path_buf) {
                                            Ok(contents) => contents,
                                            Err(error) => {
                                                println!(
                                                    "{}",
                                                    format!(
                                                "- FAILED TO READ LOCKFILE AT '{}'! ERROR[39]: {}",
                                                &aati_lock_path_buf.display(),
                                                error
                                            )
                                                    .bright_red()
                                                );

                                                exit(1);
                                            }
                                        };
                                    let mut lock_file: LockFile =
                                        toml::from_str(&lock_file_str).unwrap();

                                    let package = Package {
                                        name,
                                        source: extracted_package[0].to_string(),
                                        version,
                                        removal: removal_lines,
                                    };

                                    lock_file.package.push(package);

                                    let mut file = match OpenOptions::new()
                                        .write(true)
                                        .truncate(true)
                                        .open(&aati_lock_path_buf)
                                    {
                                        Ok(file) => file,
                                        Err(error) => {
                                            println!(
                                                    "{}",
                                                    format!(
                                                "- FAILED TO OPEN LOCKFILE AT '{}' FOR WRITING! ERROR[40]: {}",
                                                &aati_lock_path_buf.display(),
                                                error
                                            )
                                                    .bright_red()
                                                );

                                            exit(1);
                                        }
                                    };

                                    let toml_str = toml::to_string(&lock_file).unwrap();
                                    match file.write_all(toml_str.as_bytes()) {
                                        Ok(_) => {}
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                            "- FAILED TO WRITE INTO LOCKFILE AT '{}'! ERROR[41]: {}",
                                            &aati_lock_path_buf.display(),
                                            error
                                        )
                                                .bright_red()
                                            );

                                            exit(1);
                                        }
                                    }

                                    println!("{}", "+ Installation is complete!".bright_green());
                                } else {
                                    println!(
                                        "{}",
                                        "- Checksums don't match! Installation is aborted"
                                            .bright_red()
                                    );

                                    match fs::remove_file(&download_path) {
                                        Ok(_) => {}
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                                    "- UNABLE DELETE FILE '{}'! ERROR[44]: {}",
                                                    download_path.display(),
                                                    error
                                                )
                                                .bright_red()
                                            );

                                            exit(1);
                                        }
                                    }
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
    } else {
        println!("{}", "- PACKAGE NOT FOUND!".bright_red());
        exit(1);
    }
}

pub fn upgrade_command(choice: Option<&str>) {
    let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();
    let repo_list = aati_config["sources"]["repos"].as_array().unwrap();
    let mut added_repos: Vec<toml::Value> = Vec::new();

    for repo_info in repo_list {
        added_repos.push(
            get_repo_config(repo_info["name"].as_str().unwrap())
                .unwrap()
                .parse::<toml::Value>()
                .unwrap(),
        );
    }

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
        match extract_package(package_name, &added_repos) {
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
                "+ You have no packages installed to upgrade!".bright_green()
            );
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
                let aati_lock_path_buf = get_aati_lock_path_buf();

                let lock_file_str = match fs::read_to_string(&aati_lock_path_buf) {
                    Ok(contents) => contents,
                    Err(error) => {
                        println!(
                            "{}",
                            format!(
                                "- FAILED TO READ LOCKFILE AT '{}'! ERROR[45]: {}",
                                &aati_lock_path_buf.display(),
                                error
                            )
                            .bright_red()
                        );

                        exit(1);
                    }
                };
                let mut lock_file: LockFile = toml::from_str(&lock_file_str).unwrap();

                println!("{}", "+ Executing removal commands...".bright_green());

                let found_package = match lock_file
                    .package
                    .iter()
                    .find(|pkg| pkg.name == package_name)
                {
                    Some(found_package) => found_package,
                    None => {
                        println!("{}", "- PACKAGE NOT FOUND IN THE LOCKFILE!".bright_red());
                        exit(1);
                    }
                };

                execute_lines(found_package.removal.clone(), None);

                println!(
                    "{}",
                    "+ Removing package from the Lockfile...".bright_green()
                );

                lock_file
                    .package
                    .retain(|package| package.name != package_name);

                let mut file = match OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&aati_lock_path_buf)
                {
                    Ok(file) => file,
                    Err(error) => {
                        println!(
                            "{}",
                            format!(
                                "- FAILED TO OPEN LOCKFILE AT '{}' FOR WRITING! ERROR[46]: {}",
                                &aati_lock_path_buf.display(),
                                error
                            )
                            .bright_red()
                        );

                        exit(1);
                    }
                };

                let toml_str = toml::to_string_pretty(&lock_file).unwrap();
                match file.write_all(toml_str.as_bytes()) {
                    Ok(_) => {}
                    Err(error) => {
                        println!(
                            "{}",
                            format!(
                                "- FAILED TO WRITE INTO LOCKFILE AT'{}'! ERROR[47]: {}",
                                &aati_lock_path_buf.display(),
                                error
                            )
                            .bright_red()
                        );

                        exit(1);
                    }
                }

                println!(
                    "{}",
                    "+ Uninstallation finished successfully!".bright_green()
                );
            } else {
                println!("{}", "+ Transaction aborted".bright_green());
            }
        } else {
            println!("{}", "- This Package is not installed!".bright_red());
            exit(0);
        }
    } else if !installed_packages.is_empty() {
        if prompt_yn("/ Are you sure you want to uninstall all of your packages?") {
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

pub fn repo_command(first_argument_option: Option<&str>, second_argument_option: Option<&str>) {
    if let Some(first_argument) = first_argument_option {
        let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();

        let aati_config_path_buf = get_aati_config_path_buf();

        if first_argument == "init" {
            let repo_name = prompt("* What will be the Repository's name (i.e. <name>/package)?");
            let repo_maintainer = prompt("* What's the name of its Maintainer?");
            let repo_description = prompt("* What's the Description of the Repository?");

            let repo_dir = PathBuf::from("aati_repo");
            let x86_64_linux_dir = PathBuf::from("aati_repo/x86_64-linux");
            let aarch64_dir = PathBuf::from("aati_repo/aarch64-linux");
            let x86_64_windows_dir = PathBuf::from("aati_repo/x86_64-windows");

            let repo_toml_path_buf = PathBuf::from("aati_repo/repo.toml");

            match fs::create_dir_all(&repo_dir) {
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

            match fs::create_dir_all(&x86_64_linux_dir) {
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

            match fs::create_dir_all(&x86_64_windows_dir) {
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

            match fs::create_dir_all(&aarch64_dir) {
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
#       {{ tag = \"0.1.0\", checksum = \"sha256-sum-here\" }},
#       {{ tag = \"0.1.1\", checksum = \"sha256-sum-here\" }},
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

                            let mut config_file: ConfigFile =
                                toml::from_str(&config_file_str).unwrap();

                            let repo = Repo {
                                name: repo_name.to_string(),
                                url: second_argument.to_string(),
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
                            match fs::read_to_string(&aati_config_path_buf) {
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
                        let mut config_file: ConfigFile =
                            toml::from_str(&config_file_str).unwrap();

                        config_file.sources.repos.retain(|r| {
                            r.name != repo["name"].as_str().unwrap()
                                && r.url != repo["url"].as_str().unwrap()
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

                        let repo_path_buf = get_repo_config_path_buf(second_argument);

                        println!("{}", format!("+ Deleting '{}'...", repo_path_buf.display()).bright_green());

                        match fs::remove_file(&repo_path_buf) {
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
                            format!("- FAILED TO PARSE INPUT! ERROR[9]: {}", error).bright_red()
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

pub fn package_command(mut directory_name: String) {
    if directory_name.ends_with('/') {
        directory_name.pop();
    }

    let source = PathBuf::from(directory_name);
    let tar_destination = PathBuf::from(format!("{}.tar", source.display()));
    let lz4_destination = PathBuf::from(format!("{}.lz4", tar_destination.display()));

    println!(
        "{}",
        format!("+ Packaging '{}'...", source.display()).bright_green()
    );

    println!(
        "{}",
        format!(
            "+ Adding folder contents to a tarball '{}'...",
            &tar_destination.display()
        )
        .as_str()
        .bright_green()
    );

    let file = match File::create(&tar_destination) {
        Ok(file) => file,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO CREATE NEW FILE '{}'! ERROR[7]: {}",
                    tar_destination.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    let mut builder = Builder::new(file);
    builder.mode(tar::HeaderMode::Deterministic);
    match builder.append_dir_all(
        source.to_str().unwrap(),
        format!("./{}", source.to_str().unwrap()),
    ) {
        Ok(_) => match builder.finish() {
            Ok(_) => {}
            Err(error) => {
                println!(
                    "{}",
                    format!(
                        "- FAILED TO CREATE '{}' TARBALL! ERROR[102]: {}",
                        tar_destination.display(),
                        error
                    )
                    .bright_red()
                );

                exit(1);
            }
        },
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO APPEND '{}' DIRECTORY TO THE TARBALL! ERROR[101]: {}",
                    source.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    }

    let output_file = match File::create(&lz4_destination) {
        Ok(file) => file,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO CREATE FILE '{}'! ERROR[75]: {}",
                    &lz4_destination.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    let mut encoder = match EncoderBuilder::new().level(16).build(output_file) {
        Ok(encoder) => encoder,
        Err(error) => {
            println!(
                "{}",
                format!("- UNABLE INITIALISE THE LZ4 ENCODER! ERROR[76]: {}", error).bright_red()
            );

            exit(1);
        }
    };

    println!("{}", "+ Writing the compressed buffer...".bright_green());

    let mut tarball = match File::open(&tar_destination) {
        Ok(file) => file,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO OPEN FILE '{}' FOR READING! ERROR[103]: {}",
                    tar_destination.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    match io::copy(&mut tarball, &mut encoder) {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO WRITE DATA INTO THE LZ4 ENCODER! ERROR[77]: {}",
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    }

    match encoder.finish().1 {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO COMPRESS FINE '{}' USING LZ4! ERROR[78]: {}",
                    source.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    }

    match fs::remove_file(&tar_destination) {
        Ok(_) => {}
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO DELETE FILE {}! ERROR[54]: {}",
                    tar_destination.display(),
                    error
                )
                .as_str()
                .bright_red()
            );
            exit(1);
        }
    }

    println!(
        "{}",
        format!("+ Done packaging! See: {}", lz4_destination.display()).bright_green()
    );
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
        match File::open(&filename_path_buf) {
            Ok(input_file) => {
                if prompt_yn(
                    format!(
                        "/ Are you sure you want to locally install {}-{}?",
                        name, version
                    )
                    .as_str(),
                ) {
                    let mut tar_path_buf = temp_dir();
                    tar_path_buf.push(&format!("{}-{}.tar", name, version));

                    let mut package_directory = temp_dir();
                    package_directory.push(&format!("{}-{}", name, version));

                    let mut tarball = match File::create(&tar_path_buf) {
                        Ok(file) => file,
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO CREATE FILE '{}'! ERROR[94]: {}",
                                    tar_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    };

                    let mut decoder = match Decoder::new(input_file) {
                        Ok(decoder) => decoder,
                        Err(error) => {
                            println!(
                                                "{}",
                                                format!(
                                            "- FAILED TO DECODE THE LZ4 COMPRESSED PACKAGE AT '{}'! ERROR[95]: {}",
                                            filename_path_buf.display(),
                                            error
                                        )
                                                .bright_red()
                                            );

                            exit(1);
                        }
                    };

                    match copy(&mut decoder, &mut tarball) {
                        Ok(_) => {}
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO WRITE INTO FILE '{}'! ERROR[97]: {}",
                                    tar_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    }

                    let tarball = File::open(&tar_path_buf).unwrap();

                    let mut archive = Archive::new(tarball);

                    match archive.unpack(temp_dir()) {
                        Ok(_) => {}
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO EXTRACT TARBALL '{}'! ERROR[81]: {}",
                                    tar_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    }

                    match remove_file(tar_path_buf) {
                        Ok(_) => {}
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- COULD NOT DELETE TEMPORARY PACKAGE TARBALL! ERROR[84]: {}",
                                    error
                                )
                                .as_str()
                                .bright_red()
                            );
                            exit(1);
                        }
                    }

                    let mut pkgfile_path_buf = package_directory.clone();
                    pkgfile_path_buf.push("PKGFILE");

                    let pkgfile = match fs::read_to_string(&pkgfile_path_buf) {
                        Ok(contents) => contents,
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO READ FILE '{}'! ERROR[82]: {}",
                                    pkgfile_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    };

                    let (installation_lines, removal_lines) = parse_pkgfile(&pkgfile);

                    execute_lines(installation_lines, Some(&package_directory));

                    match remove_dir_all(package_directory) {
                        Ok(_) => {}
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- COULD NOT DELETE TEMPORARY PACKAGE DIRECTORY! ERROR[85]: {}",
                                    error
                                )
                                .as_str()
                                .bright_red()
                            );
                            exit(1);
                        }
                    }

                    println!("{}", "+ Adding Package to the Lockfile...".bright_green());

                    let aati_lock_path_buf = get_aati_lock_path_buf();

                    let lock_file_str = match fs::read_to_string(&aati_lock_path_buf) {
                        Ok(contents) => contents,
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO READ LOCKFILE AT '{}'! ERROR[98]: {}",
                                    &aati_lock_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    };
                    let mut lock_file: LockFile = toml::from_str(&lock_file_str).unwrap();

                    let package = Package {
                        name: name.to_string(),
                        source: source.to_string(),
                        version: version.to_string(),
                        removal: removal_lines,
                    };

                    lock_file.package.push(package);

                    let mut file = match OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(&aati_lock_path_buf)
                    {
                        Ok(file) => file,
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO OPEN LOCKFILE AT '{}' FOR WRITING! ERROR[80]: {}",
                                    &aati_lock_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
                    };

                    let toml_str = toml::to_string(&lock_file).unwrap();
                    match file.write_all(toml_str.as_bytes()) {
                        Ok(_) => {}
                        Err(error) => {
                            println!(
                                "{}",
                                format!(
                                    "- FAILED TO WRITE INTO LOCKFILE AT '{}'! ERROR[2]: {}",
                                    &aati_lock_path_buf.display(),
                                    error
                                )
                                .bright_red()
                            );

                            exit(1);
                        }
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
                                    fs::create_dir_all(format!(
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

pub fn serve_command(address_option: Option<&str>) {
    let address;
    let website_url;
    let repo_url;

    if let Some(given_address) = address_option {
        address = given_address;
        website_url = prompt("On what URL will this index be hosted (e.g. http://example.com)?");
        repo_url = prompt("On what URL is the package repository hosted?");
    } else {
        address = "localhost:8887";
        website_url = "http://localhost:8887".to_string();
        repo_url = "http://localhost:8887".to_string();
    };

    match Server::http(address) {
        Ok(server) => match read_to_string("repo.toml") {
            Ok(repo_toml) => match repo_toml.parse::<toml::Value>() {
                Ok(repo_config) => {
                    let packages = repo_config["index"]["packages"].as_array().unwrap();
                    let targets = vec![
                        "x86_64-linux",
                        "aarch64-linux",
                        "x86_64-windows",
                        "aarch64-windows",
                    ];

                    println!(
                        "{}",
                        format!(
                            "+ Listening on port: {}",
                            server.server_addr().to_ip().unwrap().port()
                        )
                        .bright_green()
                    );

                    for request in server.incoming_requests() {
                        let mut html =
                            generate_apr_html(&repo_config, "index", None, &website_url, &repo_url);
                        let mut url = request.url().to_string();

                        print!(
                            "{}",
                            format!("+   {} {}", request.method(), request.url()).bright_green()
                        );

                        url.remove(0);

                        if url.is_empty() || url == "index.html" {
                            html = generate_apr_html(
                                &repo_config,
                                "index",
                                None,
                                &website_url,
                                &repo_url,
                            );
                        } else if url == "about.html" {
                            html = generate_apr_html(
                                &repo_config,
                                "about",
                                None,
                                &website_url,
                                &repo_url,
                            );
                        } else if url == "packages.html" {
                            html = generate_apr_html(
                                &repo_config,
                                "packages",
                                None,
                                &website_url,
                                &repo_url,
                            );
                        } else {
                            let mut html_assigned = false;

                            for target in &targets {
                                if url == format!("{}/index.html", target)
                                    || url == format!("{}/", target)
                                    || url == *target.to_string()
                                {
                                    html_assigned = true;
                                    html = generate_apr_html(
                                        &repo_config,
                                        target,
                                        None,
                                        &website_url,
                                        &repo_url,
                                    );
                                }
                            }

                            if !html_assigned {
                                for target in &targets {
                                    for package in packages {
                                        let package_name = package["name"].as_str().unwrap();
                                        let package_target = package["target"].as_str().unwrap();

                                        if target == &package_target
                                            && url
                                                == format!(
                                                    "{}/{}/{}.html",
                                                    package_target, package_name, package_name
                                                )
                                        {
                                            html = generate_apr_html(
                                                &repo_config,
                                                "package",
                                                Some(package),
                                                &website_url,
                                                &repo_url,
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        let response = Response::from_string(html);

                        let response = response.with_header(Header {
                            field: "Content-Type".parse().unwrap(),
                            value: AsciiString::from_ascii("text/html; charset=utf8").unwrap(),
                        });

                        match request.respond(response) {
                            Ok(_) => {}

                            Err(error) => {
                                println!("{}", format!("ERROR[18]: {}", error).bright_red());
                                exit(1);
                            }
                        }
                    }
                }

                Err(error) => {
                    println!("{}", format!("ERROR[17]: {}", error).bright_red());
                    exit(1);
                }
            },

            Err(error) => {
                println!("{}", format!("ERROR[16]: {}", error).bright_red());
                exit(1);
            }
        },

        Err(error) => {
            println!("{}", format!("ERROR[15]: {}", error).bright_red());
            exit(1);
        }
    }
}
