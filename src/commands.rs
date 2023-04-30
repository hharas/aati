use crate::structs;
use crate::utils::*;

use colored::Colorize;
use humansize::{format_size, BINARY};
use lz4::Decoder;
use lz4::EncoderBuilder;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::{copy, Write};
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;
use std::process::exit;

pub fn get_command(package_name: &str) {
    // Initialise some variables

    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();
    let repo_toml: toml::Value = get_repo_config().unwrap().parse().unwrap();
    let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

    let extracted_package = extract_package(&package_name.to_string());

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
                        && available_package["arch"].as_str().unwrap() == get_arch()
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
            aati_config["repo"]["url"].as_str().unwrap(),
            get_arch(),
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
                        "/ Are you sure you want to install {}-{} ({})?",
                        name, version, human_readable_size
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

                            let home_dir =
                                dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");
                            let download_path =
                                home_dir.join(format!("/tmp/{}-{}.lz4", name, version));

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

                            let mut checksum_reader = File::open(download_path.clone()).unwrap();
                            let lz4_reader = File::open(download_path.clone()).unwrap();

                            let mut body = Vec::new();
                            checksum_reader.read_to_end(&mut body).unwrap();

                            // 7. Verify the SHA256 Checksum of the LZ4 compressed package

                            if verify_checksum(&body, checksum.to_string()) {
                                println!("{}", "+ Checksums match!".bright_green());

                                let installation_path_buf =
                                    home_dir.join(format!(".local/bin/{}", name));
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

                                let aati_lock_path_buf = home_dir.join(".config/aati/lock.toml");

                                let lock_file_str =
                                    fs::read_to_string(aati_lock_path_buf.clone()).unwrap();
                                let mut lock_file: structs::LockFile =
                                    toml::from_str(&lock_file_str).unwrap();

                                let package = structs::Package {
                                    name,
                                    source: "default".to_string(),
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

                                println!("{}", "+ Changing Permissions...".bright_green());

                                // 10. Turn it into an executable file, simply: chmod +x ~/.local/bin/<package name>

                                let metadata = fs::metadata(installation_path_buf.clone()).unwrap();
                                let mut permissions = metadata.permissions();
                                permissions.set_mode(0o755);
                                fs::set_permissions(installation_path_buf, permissions).unwrap();

                                println!("{}", "+ Installation is complete!".bright_green());
                            } else {
                                println!(
                                    "{}",
                                    "- Checksums don't match! Installation is aborted".bright_red()
                                );

                                fs::remove_file(download_path).unwrap();
                            }
                        }

                        Err(error) => {
                            println!("{}", format!("- ERROR[1]: {}", error).as_str().bright_red());
                            exit(1);
                        }
                    };
                }
            }

            Err(error) => {
                println!("{}", format!("- ERROR[0]: {}", error).as_str().bright_red());
                exit(1);
            }
        }
    } else {
        println!("{}", "+ Transaction aborted".bright_green());
    }
}

pub fn upgrade_command(choice: Option<&str>) {
    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let repo_toml: toml::Value = get_repo_config().unwrap().parse().unwrap();

    let installed_packages = aati_lock["package"].as_array().unwrap();
    let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

    if let Some(package_name) = choice {
        if let Some(installed_package) = installed_packages
            .iter()
            .find(|pkg| pkg["name"].as_str().unwrap() == package_name)
        {
            if let Some(available_package) = available_packages
                .iter()
                .find(|pkg| pkg["name"].as_str().unwrap() == package_name)
            {
                if installed_package["version"].as_str().unwrap()
                    != available_package["current"].as_str().unwrap()
                    && available_package["arch"].as_str().unwrap() == get_arch()
                {
                    if prompt_yn(
                        format!(
                            "/ Are you sure you want to upgrade {}-{} -> {}?",
                            installed_package["name"].as_str().unwrap(),
                            installed_package["version"].as_str().unwrap(),
                            available_package["current"].as_str().unwrap()
                        )
                        .as_str(),
                    ) {
                        uninstall_command(package_name);
                        get_command(package_name);
                        println!("{}", "+ Package Upgrade finished!".bright_green());
                    } else {
                        println!("{}", "+ Transaction aborted".bright_green());
                    }
                } else {
                    println!(
                        "{} The Package '{}-{}' is up-to-date!",
                        "+".bright_green(),
                        installed_package["name"].as_str().unwrap(),
                        installed_package["version"].as_str().unwrap()
                    );
                }
            } else {
                println!("{}", format!("- The Package '{}' is not found in the Package Repository! Try syncing the Repository by running:\n    $ aati sync", package_name).as_str().bright_red());
            }
        } else {
            println!("{}", format!("- The Package '{}' isn't even installed! You can install it by running:\n    $ aati get {}", package_name, package_name).as_str().bright_red());
        }
    } else {
        let mut to_be_upgraded: Vec<&str> = Vec::new();

        println!("{}", "+ Packages to be upgraded:".bright_green());

        if !installed_packages.is_empty() {
            for installed_package in installed_packages {
                for available_package in available_packages {
                    if installed_package["name"].as_str().unwrap()
                        == available_package["name"].as_str().unwrap()
                        && installed_package["version"].as_str().unwrap()
                            != available_package["current"].as_str().unwrap()
                        && available_package["arch"].as_str().unwrap() == get_arch()
                    {
                        to_be_upgraded.push(available_package["name"].as_str().unwrap());
                        println!(
                            "{}   {}-{} -> {}",
                            "+".bright_green(),
                            installed_package["name"].as_str().unwrap(),
                            installed_package["version"].as_str().unwrap(),
                            available_package["current"].as_str().unwrap(),
                        );
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
        }
    }
}

pub fn uninstall_command(package_name: &str) {
    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    if package_name != "--all" {
        let mut is_installed = false;
        let mut package: &toml::Value =
            &toml::Value::from("name = \"dummy-package\"\nversion = \"0.1.0\"");

        for installed_package in installed_packages {
            if installed_package["name"].as_str().unwrap() == package_name {
                package = installed_package;
                is_installed = true;
            }
        }

        if is_installed {
            if prompt_yn(
                format!(
                    "/ Are you sure you want to completely uninstall {}-{}?",
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

                let path = dirs::home_dir()
                    .unwrap()
                    .join(format!(".local/bin/{}", package_name));

                match fs::remove_file(path) {
                    Ok(_) => {
                        println!(
                            "{}",
                            "+ Removing package from the Lockfile...".bright_green()
                        );

                        let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");
                        let aati_lock_path_buf = home_dir.join(".config/aati/lock.toml");

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
            println!(
                "{}",
                "- This Package is already not installed!".bright_red()
            );
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
    let aati_lock: toml::Value = get_aati_lock().unwrap().parse().unwrap();
    let repo_toml: toml::Value = get_repo_config().unwrap().parse().unwrap();

    if let Some(choice) = choice_option {
        if choice.to_ascii_lowercase() == "installed" {
            let installed_packages = aati_lock["package"].as_array().unwrap();
            let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

            println!("{}", "+ Installed Packages:".bright_green());

            if !installed_packages.is_empty() {
                for installed_package in installed_packages {
                    for available_package in available_packages {
                        if installed_package["name"].as_str().unwrap()
                            == available_package["name"].as_str().unwrap()
                            && available_package["arch"].as_str().unwrap() == get_arch()
                        {
                            if installed_package["version"].as_str().unwrap()
                                != available_package["current"].as_str().unwrap()
                            {
                                println!(
                                    "    {}-{} {}",
                                    installed_package["name"].as_str().unwrap(),
                                    installed_package["version"].as_str().unwrap(),
                                    "[outdated]".yellow()
                                );
                            } else {
                                println!(
                                    "    {}-{}",
                                    installed_package["name"].as_str().unwrap(),
                                    installed_package["version"].as_str().unwrap()
                                );
                            }
                        }
                    }
                }
            } else {
                println!("  None! Install Packages using: $ aati get <package>");
            }
        } else if choice.to_ascii_lowercase() == "available" {
            let installed_packages = aati_lock["package"].as_array().unwrap();
            let available_packages = repo_toml["index"]["packages"].as_array().unwrap();
            let repo_name = repo_toml["repo"]["name"].as_str().unwrap();

            println!(
                "{}",
                format!(
                    "+ Available Packages ({}):",
                    available_packages
                        .iter()
                        .filter(|pkg| pkg["arch"].as_str().unwrap() == get_arch())
                        .count()
                )
                .as_str()
                .bright_green()
            );

            if !available_packages.is_empty() {
                for package in available_packages {
                    if package["arch"].as_str().unwrap() == get_arch() {
                        println!("    {}/{}:", repo_name, package["name"].as_str().unwrap());
                        let versions = package["versions"].as_array().unwrap();

                        let mut reversed_versions = versions.clone();
                        reversed_versions.reverse();

                        for version in reversed_versions {
                            let tag = version["tag"].as_str().unwrap();
                            let is_installed = installed_packages.iter().any(|installed_pkg| {
                                installed_pkg["name"].as_str().unwrap()
                                    == package["name"].as_str().unwrap()
                                    && installed_pkg["version"].as_str().unwrap() == tag
                            });

                            if !is_installed {
                                println!("      {}-{}", package["name"].as_str().unwrap(), tag);
                            } else {
                                println!(
                                    "      {}-{} {}",
                                    package["name"].as_str().unwrap(),
                                    tag,
                                    "[installed]".bright_green()
                                );
                            }
                        }
                    }
                }

                println!("{}", "\n+ Unsupported packages:".bright_yellow());

                for package in available_packages {
                    if package["arch"].as_str().unwrap() != get_arch() {
                        println!(
                            "    {}/{} ({}):",
                            repo_name,
                            package["name"].as_str().unwrap(),
                            package["arch"].as_str().unwrap()
                        );
                        let versions = package["versions"].as_array().unwrap();

                        let mut reversed_versions = versions.clone();
                        reversed_versions.reverse();

                        for version in reversed_versions {
                            let tag = version["tag"].as_str().unwrap();

                            println!("      {}-{}", package["name"].as_str().unwrap(), tag);
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
        let installed_packages = aati_lock["package"].as_array().unwrap();
        let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

        println!("{}", "+ Installed Packages:".bright_green());

        if !installed_packages.is_empty() {
            for installed_package in installed_packages {
                for available_package in available_packages {
                    if installed_package["name"].as_str().unwrap()
                        == available_package["name"].as_str().unwrap()
                        && available_package["arch"].as_str().unwrap() == get_arch()
                    {
                        if installed_package["version"].as_str().unwrap()
                            != available_package["current"].as_str().unwrap()
                        {
                            println!(
                                "    {}-{} {}",
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap(),
                                "[outdated]".yellow()
                            );
                        } else {
                            println!(
                                "    {}-{}",
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap()
                            );
                        }
                    }
                }
            }
        } else {
            println!("    None! Install Packages using: $ aati get <package>");
        }
    }
}

pub fn sync_command() {
    let aati_config: toml::Value = get_aati_config()
        .unwrap()
        .parse()
        .expect("- UNABLE TO PARSE ~/.config/aati/rc.toml!");

    match aati_config
        .get("repo")
        .and_then(|repo| repo.get("url"))
        .and_then(|url| url.as_str())
    {
        Some(url) => {
            let requested_url = format!("{}/repo.toml", url);

            println!(
                "{}",
                format!("+ Requesting ({})", requested_url)
                    .as_str()
                    .bright_green()
            );

            match ureq::get(requested_url.as_str()).call() {
                Ok(repo_toml) => {
                    let repo_toml = repo_toml.into_string().unwrap();

                    let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");
                    let repo_config_path_buf = home_dir.join(".config/aati/repo.toml");

                    let mut repo_config = File::create(repo_config_path_buf)
                        .expect("- UNABLE TO CREATE ~/.config/aati/repo.toml!");

                    println!(
                        "{}",
                        "+ Writing Repo Config to ~/.config/aati/repo.toml".bright_green()
                    );
                    writeln!(repo_config, "{}", repo_toml.as_str()).unwrap();

                    println!("{}", "+ Done syncing with the Repo!".bright_green());
                }

                Err(error) => {
                    println!(
                        "{}",
                        format!(
                            "- UNABLE TO REQUEST ({})! ERROR[5]: {}",
                            requested_url, error
                        )
                        .as_str()
                        .bright_red()
                    );
                    exit(1);
                }
            }
        }

        None => {
            println!("{}", "- ERROR[8]: UNABLE TO PARSE INFO FROM ~/.config/aati/rc.toml! TRY: aati repo <repo url>".bright_red());
            exit(1);
        }
    }
}

pub fn repo_command(first_argument_option: Option<&str>, second_argument_option: Option<&str>) {
    if let Some(first_argument) = first_argument_option {
        let aati_config: toml::Value = get_aati_config().unwrap().parse().unwrap();
        let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");

        let aati_config_path_buf = home_dir.join(".config/aati/rc.toml");

        if first_argument == "init" {
            let repo_name = prompt("* What will be the Repository's name (i.e. <name>/package)?");
            let repo_maintainer = prompt("* What's the name of its Maintainer?");
            let repo_description = prompt("* What's the Description of the Repository?");

            fs::create_dir_all("aati_repo").unwrap();
            let mut file = File::create("aati_repo/repo.toml").unwrap();

            fs::create_dir_all("aati_repo/x86-64").unwrap();
            fs::create_dir_all("aati_repo/x86-64/dummy-package").unwrap();
            fs::create_dir_all("aati_repo/aarch64").unwrap();
            fs::create_dir_all("aati_repo/aarch64/dummy-package").unwrap();

            let dummy1_path = PathBuf::from("aati_repo/x86-64/dummy-package/dummy-package-0.1.0");
            let dummy2_path = PathBuf::from("aati_repo/x86-64/dummy-package/dummy-package-0.1.1");
            let dummy3_path = PathBuf::from("aati_repo/aarch64/dummy-package/dummy-package-0.1.0");
            let dummy4_path = PathBuf::from("aati_repo/aarch64/dummy-package/dummy-package-0.1.1");

            let mut dummy1 = File::create(dummy1_path.clone()).unwrap();
            let mut dummy2 = File::create(dummy2_path.clone()).unwrap();
            let mut dummy3 = File::create(dummy3_path.clone()).unwrap();
            let mut dummy4 = File::create(dummy4_path.clone()).unwrap();

            dummy1
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.0 for x86-64 machines\"")
                .unwrap();
            dummy2
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.1 for x86-64 machines\"")
                .unwrap();
            dummy3
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.0 for aarch64 machines\"")
                .unwrap();
            dummy4
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.1 for aarch64 machines\"")
                .unwrap();

            package_command(format!("{}", dummy1_path.display()).as_str());
            package_command(format!("{}", dummy2_path.display()).as_str());
            package_command(format!("{}", dummy3_path.display()).as_str());
            package_command(format!("{}", dummy4_path.display()).as_str());

            fs::remove_file(dummy1_path).unwrap();
            fs::remove_file(dummy2_path).unwrap();
            fs::remove_file(dummy3_path).unwrap();
            fs::remove_file(dummy4_path).unwrap();

            let contents = format!("[repo]\nname = \"{}\"\nmaintainer = \"{}\"\ndescription = \"{}\"\n\n[index]\npackages = [\n    {{ name = \"dummy-package\", current = \"0.1.1\", arch = \"x86-64\", versions = [\n    {{ name = \"dummy-package\", current = \"0.1.1\", arch = \"aarch64\", versions = [\n        {{ tag = \"0.1.0\", checksum = \"4237a71f63ef797e4bd5c70561ae85f68e66f84ae985704c14dd53fa9d81d7ac\" }},\n        {{ tag = \"0.1.1\", checksum = \"eda1b669d0bf90fdeb247a1e768a60baf56b9ba008a05c34859960be803d0ac4\" }},\n    ], author = \"{}\", description = \"Aati Dummy Package. This is a Package created as a template.\", url = \"https://codeberg.org/amad/aati\" }},\n        {{ tag = \"0.1.0\", checksum = \"ac5d6d9d495700c3f5880e89b34f56259a888b9ef671a76fc43410a1712acf95\" }},\n        {{ tag = \"0.1.1\", checksum = \"64cc0909fe1a2eaa2f7b211c1cf0250596d2c20b225c0c86507f01db9032913a\" }},\n    ], author = \"{}\", description = \"Aati Dummy Package. This is a Package created as a template.\", url = \"https://codeberg.org/amad/aati\" }}]\n", repo_name, repo_maintainer, repo_description, repo_maintainer, repo_maintainer);

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

                            let repo_config_path_buf =
                                home_dir.join(format!(".config/aati/repos/{}.toml", repo_name));

                            let mut repo_config = File::create(repo_config_path_buf).expect(
                                format!(
                                    "- UNABLE TO CREATE ~/.config/aati/repos/{}.toml!",
                                    repo_name
                                )
                                .as_str(),
                            );

                            println!(
                                "{}",
                                format!(
                                    "+ Writing Repo Config to ~/.config/aati/repos/{}.toml",
                                    repo_name
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

                            let mut file = OpenOptions::new()
                                .write(true)
                                .truncate(true)
                                .open(home_dir.join(".config/aati/rc.toml"))
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
                        println!("{}", format!("+ Removing {} from the Config File...", second_argument).bright_green());
                        
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

                        println!("{}", format!("+ Deleting '{}'...", home_dir.join(format!(".config/aati/repos/{}.toml", second_argument)).display()).bright_green());

                        fs::remove_file(home_dir.join(format!(".config/aati/repos/{}.toml", second_argument))).unwrap();

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
        }
    } else {
        let aati_config = get_aati_config().unwrap();
        let aati_toml: toml::Value = aati_config
            .parse()
            .expect("- CAN NOT PARSE ~/.config/aati/repo.toml!");

        let repo_config = get_repo_config().unwrap();
        let repo_toml: toml::Value = repo_config
            .parse()
            .expect("- CAN NOT PARSE ~/.config/aati/repo.toml!");

        let url = aati_toml["repo"]["url"].as_str().unwrap();
        let maintainer = repo_toml["repo"]["maintainer"].as_str().unwrap();
        let description = repo_toml["repo"]["description"].as_str().unwrap();
        let packages_number = repo_toml["index"]["packages"].as_array().unwrap().len();

        println!(
            "+ Repository Information:\n    URL: {}\n    Maintainer: {}\n    Number of Packages: {}\n    Description:\n      {}",
            url, maintainer, packages_number, description
        );
    }
}

pub fn info_command(package_name: &str) {
    let mut is_installed = false;
    let mut installed_package_version = "0.0.0";
    let mut is_up_to_date = false;

    let repo_config = get_repo_config().unwrap();

    let repo_toml: toml::Value = repo_config
        .parse()
        .expect("- CAN NOT PARSE ~/.config/aati/repo.toml!");

    match repo_toml["index"]["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pkg| pkg["name"] == package_name.into() && pkg["arch"] == get_arch().into())
        //                                                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //                                                      This has a drawback, which is that
        //                                                      if a user runs aati info <package>
        //                                                      he won't receive any info if it's
        //                                                      a package written for another arch
    {
        Some(package) => {
            let aati_lock = get_aati_lock().unwrap();

            let lock_toml: toml::Value = aati_lock
                .parse()
                .expect("- CAN NOT PARSE ~/.config/aati/repo.toml!");

            let installed_packages = lock_toml["package"].as_array().unwrap();

            if !installed_packages.is_empty() {
                for installed_package in installed_packages {
                    if installed_package["name"].as_str().unwrap() == package_name {
                        is_installed = true;
                        installed_package_version = installed_package["version"].as_str().unwrap();
                        if installed_package_version == package["current"].as_str().unwrap() {
                            is_up_to_date = true;
                        }
                    }
                }
            }

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
                "{}\n    Name: {}\n    Author: {}\n    Architecture: {}",
                "+ Package Information:".bright_green(),
                name,
                author,
                arch
            );

            match is_installed {
                true => match is_up_to_date {
                    true => println!("    Version: {} {}", version, "[installed]".bright_green()),
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

        None => {
            println!("{}", "- ERROR[9]: PACKAGE NOT FOUND (AT LEAST FOR YOUR ISA)! TRY: $ aati sync".bright_red());
            exit(1);
        }
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
