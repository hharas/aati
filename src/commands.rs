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

    let extracted_package = extract_package(package_name);

    let mut is_installed = false;
    let mut is_found = false;
    let mut checksum = "";

    for installed_package in installed_packages {
        if installed_package["name"].as_str().unwrap() == extracted_package[0] {
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
            if available_package["name"].as_str().unwrap() == extracted_package[0] {
                for package_version in available_package["versions"].as_array().unwrap() {
                    if package_version["tag"].as_str().unwrap() == extracted_package[1].clone() {
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
        let name = extracted_package[0].clone();
        let version = extracted_package[1].clone();

        let url = format!(
            "{}/{}/{}-{}.lz4",
            repo_toml["repo"]["url"].as_str().unwrap(),
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

                                let package = structs::Package { name, version };

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
        println!("{}", "+ Transaction stopped".bright_green());
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
                if installed_package["version"].as_str() != available_package["current"].as_str() {
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
                        println!("{}", "+ Transaction stopped".bright_green());
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
                    println!("{}", "+ Transaction stopped".bright_green());
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
                    let mut lock_file: structs::LockFile = toml::from_str(&lock_file_str).unwrap();

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
            println!("{}", "+ Transaction stopped".bright_green());
        }
    } else {
        println!(
            "{}",
            "- This Package is already not installed!".bright_red()
        );
        exit(0);
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
                        {
                            if installed_package["version"].as_str().unwrap()
                                != available_package["current"].as_str().unwrap()
                            {
                                println!(
                                    "    {}-{} {}",
                                    installed_package["name"].as_str().unwrap(),
                                    installed_package["version"].as_str().unwrap(),
                                    "[outdated]".bright_red()
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

            println!(
                "{}",
                format!("+ Available Packages ({}):", available_packages.len())
                    .as_str()
                    .bright_green()
            );

            if !available_packages.is_empty() {
                for package in available_packages {
                    println!("    {}:", package["name"].as_str().unwrap());
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
                    {
                        if installed_package["version"].as_str().unwrap()
                            != available_package["current"].as_str().unwrap()
                        {
                            println!(
                                "    {}-{} {}",
                                installed_package["name"].as_str().unwrap(),
                                installed_package["version"].as_str().unwrap(),
                                "[outdated]".bright_red()
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
    let repo_config: toml::Value = get_repo_config()
        .unwrap()
        .parse()
        .expect("- UNABLE TO PARSE ~/.config/aati/repo.toml!");

    let repo_url = repo_config
        .get("repo")
        .and_then(|repo| repo.get("url"))
        .and_then(|url| url.as_str())
        .expect("- UNABLE TO PARSE INFO FROM ~/.config/aati/repo.toml!");

    let requested_url = format!("{}/repo.toml", repo_url);

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

pub fn repo_command(repo_url_option: Option<&str>) {
    if let Some(repo_url) = repo_url_option {
        if repo_url == "init" {
            let repo_url = prompt("* On what URL will this Package Repository be hosted?");
            let repo_maintainer = prompt("* What's the name of its Maintainer?");
            let repo_description = prompt("* What's the Description of the Repository?");

            fs::create_dir_all("aati_repo").unwrap();
            let mut file = File::create("aati_repo/repo.toml").unwrap();

            fs::create_dir_all("aati_repo/dummy-package").unwrap();

            let dummy1_path = PathBuf::from("aati_repo/dummy-package/dummy-package-0.1.0");
            let dummy2_path = PathBuf::from("aati_repo/dummy-package/dummy-package-0.1.1");

            let mut dummy1 = File::create(dummy1_path.clone()).unwrap();
            let mut dummy2 = File::create(dummy2_path.clone()).unwrap();

            dummy1
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.0\"")
                .unwrap();
            dummy2
                .write_all(b"#!/usr/bin/bash\n\necho \"This is Aati Dummy Package 0.1.1\"")
                .unwrap();

            package_command(format!("{}", dummy1_path.display()).as_str());
            package_command(format!("{}", dummy2_path.display()).as_str());

            fs::remove_file(dummy1_path).unwrap();
            fs::remove_file(dummy2_path).unwrap();

            let contents = format!("[repo]\nurl = \"{}\"\nmaintainer = \"{}\"\ndescription = \"{}\"\n\n[index]\npackages = [\n    {{ name = \"dummy-package\", current = \"0.1.1\", versions = [\n        {{ tag = \"0.1.0\", checksum = \"a7d9edd360059777ee8e0d80a6dcf64299b41d7e6a5f720833fcf2bcd7105604\" }},\n        {{ tag = \"0.1.1\", checksum = \"c92d96486db47fc9d4a9fdef0da454afb5a434a933801e0b603269d17f0ad64d\" }},\n    ], author = \"{}\", description = \"Aati Dummy Package. This is a Package created as a template.\", url = \"https://codeberg.org/amad/aati\" }}\n]", repo_url, repo_maintainer, repo_description, repo_maintainer);

            file.write_all(contents.as_bytes()).unwrap();

            println!(
                "{}",
                "+ The Repo is done! Now you can add your packages".bright_green()
            );
        } else {
            println!(
                "{}",
                format!("+ Setting ({}) as the current repository", repo_url)
                    .as_str()
                    .bright_green()
            );

            let requested_url = format!("{}/repo.toml", repo_url);
            println!(
                "{}",
                format!("+ Requesting ({})", requested_url)
                    .as_str()
                    .bright_green()
            );

            match ureq::get(requested_url.as_str()).call() {
                Ok(repo_toml) => {
                    let repo_toml = repo_toml.into_string().unwrap();

                    check_config_dir();

                    let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");
                    let repo_config_path_buf = home_dir.join(".config/aati/repo.toml");

                    let mut repo_config = File::create(repo_config_path_buf)
                        .expect("- UNABLE TO CREATE ~/.config/aati/repo.toml!");

                    println!(
                        "{}",
                        "+ Writing Repo Config to ~/.config/aati/repo.toml".bright_green()
                    );
                    writeln!(repo_config, "{}", repo_toml.as_str()).unwrap();

                    println!("{}", "+ Repo set successfully!".bright_green());
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
        }
    } else {
        let repo_config = get_repo_config().unwrap();

        let repo_toml: toml::Value = repo_config
            .parse()
            .expect("- CAN NOT PARSE ~/.config/aati/repo.toml!");

        let url = repo_toml["repo"]["url"].as_str().unwrap();
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

    let package = repo_toml["index"]["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pkg| pkg["name"] == package_name.into())
        .expect("- PACKAGE NOT FOUND! TRY: $ aati sync");

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
    let url = package["url"].as_str().unwrap();
    let description = package["description"].as_str().unwrap();

    println!(
        "{}\n    Name: {}\n    Author: {}",
        "+ Package Information:".bright_green(),
        name,
        author
    );

    match is_installed {
        true => match is_up_to_date {
            true => println!("    Version: {} {}", version, "[installed]".bright_green()),
            false => println!(
                "    Version: {} {}",
                version,
                format!("[{} is installed]", installed_package_version).bright_red()
            ),
        },
        false => println!("    Version: {}", version),
    };

    println!(
        "    Available Versions: {}\n    URL: {}\n    Description:\n      {}",
        tags.join(", "),
        url,
        description
    );
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
