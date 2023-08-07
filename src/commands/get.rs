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

use std::{
    env::temp_dir,
    fs::{read_to_string, remove_dir_all, remove_file, File, OpenOptions},
    io::{copy, Read, Write},
    process::exit,
};

use crate::{
    types::{LockFile, Package},
    utils::{
        execute_lines, extract_package, get_aati_config, get_aati_lock, get_aati_lock_path_buf,
        get_repo_config, is_supported, is_windows, parse_pkgfile, prompt_yn,
    },
};
use colored::Colorize;
use humansize::{format_size, BINARY};
use lz4::Decoder;
use ring::digest;
use tar::Archive;
use toml::Value;

pub fn command(package_name: &str, force: bool) {
    // Initialise some variables

    let aati_lock: Value = get_aati_lock().unwrap().parse().unwrap();
    let aati_config: Value = get_aati_config().unwrap().parse().unwrap();
    let repo_list = aati_config["sources"]["repos"].as_array().unwrap();
    let mut added_repos: Vec<Value> = Vec::new();

    for repo_info in repo_list {
        added_repos.push(
            get_repo_config(repo_info["name"].as_str().unwrap())
                .unwrap()
                .parse::<Value>()
                .unwrap(),
        );
    }

    let installed_packages = aati_lock["package"].as_array().unwrap();

    if let Some(extracted_package) = extract_package(package_name, &added_repos) {
        let repo_toml: Value = get_repo_config(extracted_package[0].as_str())
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
                format!("+ Package '{}' is already installed!", extracted_package[1]).bright_blue()
            );
            return;
        } else {
            for available_package in available_packages {
                if available_package["name"].as_str().unwrap() == extracted_package[1] {
                    for package_version in available_package["versions"].as_array().unwrap() {
                        if package_version["tag"].as_str().unwrap() == extracted_package[2].clone()
                            && is_supported(available_package["target"].as_str().unwrap())
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
                format!(
                    "- Package '{}' is not found on the Repository! Try: $ aati sync",
                    extracted_package[1]
                )
                .bright_red()
            );
            exit(1);
        }

        if !is_installed && is_found {
            let name = extracted_package[1].clone();
            let version = extracted_package[2].clone();

            let aati_config: Value = get_aati_config().unwrap().parse().unwrap();

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
                extracted_package[3],
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

                    if force
                        || prompt_yn(
                            format!(
                                "/ Are you sure you want to install {}/{}-{} ({})?",
                                extracted_package[0], name, version, human_readable_size
                            )
                            .as_str(),
                        )
                    {
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

                                    match remove_file(&download_path) {
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

                                    let pkgfile = match read_to_string(&pkgfile_path_buf) {
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

                                    let parsed_pkgfile = parse_pkgfile(&pkgfile);

                                    let selected_installation_lines = if is_windows() {
                                        if !parsed_pkgfile.win_installation_lines.is_empty() {
                                            parsed_pkgfile.win_installation_lines.clone()
                                        } else {
                                            parsed_pkgfile.installation_lines.clone()
                                        }
                                    } else {
                                        parsed_pkgfile.installation_lines.clone()
                                    };

                                    if force
                                        || prompt_yn(&format!(
                                            "+ Commands to be ran:\n  {}\n/ Do these commands seem safe to execute?",
                                            selected_installation_lines.join("\n  ")
                                        ))
                                    {

                                        execute_lines(&selected_installation_lines, &parsed_pkgfile.data, Some(&package_directory));

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

                                    let lock_file_str = match read_to_string(&aati_lock_path_buf) {
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
                                        version,
                                        source: extracted_package[0].to_string(),
                                        target: extracted_package[3].to_string(),
                                        pkgfile: parsed_pkgfile.clone()
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
                                    println!("{}", "+ Transaction aborted".bright_green());

                                    match remove_dir_all(&package_directory) {
                                        Ok(_) => println!("{}", "+ Deleted temporary package directory".bright_green()),
                                        Err(error) => {
                                            println!(
                                                "{}",
                                                format!(
                                                    "- FAILED TO DELETE DIRECTORY '{}'! ERROR[86]: {}",
                                                    package_directory.display(),
                                                    error
                                                )
                                                .as_str()
                                                .bright_red()
                                            );
                                            exit(1);
                                        }
                                    }
                                }
                                } else {
                                    println!(
                                        "{}",
                                        "- Checksums don't match! Installation is aborted"
                                            .bright_red()
                                    );

                                    match remove_file(&download_path) {
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

pub fn verify_checksum(body: &[u8], checksum: String) -> bool {
    let hash = digest::digest(&digest::SHA256, body);
    let hex = hex::encode(hash.as_ref());

    hex == checksum
}
