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

use colored::Colorize;
use lz4::Decoder;
use std::{
    collections::HashMap,
    env::temp_dir,
    fs::{read_to_string, remove_dir_all, remove_file, File, OpenOptions},
    io::{copy, Write},
    path::PathBuf,
    process::exit,
};
use tar::Archive;
use toml::Value;

use crate::{
    commands::remove,
    types::{LockFile, Package, Pkgfile},
    utils::{
        execute_lines, get_aati_lock, get_aati_lock_path_buf, get_target, parse_pkgfile, prompt_yn,
    },
};

pub fn command(filename: &str, force: bool, quiet: bool) {
    let filename_path_buf = PathBuf::from(filename);

    let parsed_package = parse_filename(filename_path_buf.file_name().unwrap().to_str().unwrap());
    let source = parsed_package.source.as_str();
    let name = parsed_package.name.as_str();
    let version = parsed_package.version.as_str();

    let aati_lock: Value = get_aati_lock().parse().unwrap();
    let installed_packages = aati_lock["package"].as_array().unwrap();

    if installed_packages
        .iter()
        .any(|pkg| pkg["name"].as_str().unwrap() == name)
    {
        if force || prompt_yn("There's a package with the same name already installed! Do you want to remove the original and proceed?") {
            remove::command(name, force, quiet);
        } else {
            exit(0);
        }
    }

    match File::open(&filename_path_buf) {
        Ok(input_file) => {
            if force
                || prompt_yn(
                    format!(
                        "/ Are you sure you want to locally install {}-{}?",
                        name, version
                    )
                    .as_str(),
                )
            {
                let mut tar_path_buf = temp_dir();
                tar_path_buf.push(format!("{}-{}.tar", name, version));

                let mut package_directory = temp_dir();
                package_directory.push(format!("{}-{}", name, version));

                let mut tarball = match File::create(&tar_path_buf) {
                    Ok(file) => file,
                    Err(error) => {
                        eprintln!(
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
                        eprintln!(
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
                        eprintln!(
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
                        eprintln!(
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
                        eprintln!(
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

                let pkgfile = match read_to_string(&pkgfile_path_buf) {
                    Ok(contents) => contents,
                    Err(error) => {
                        eprintln!(
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

                let parsed_pkgfile = parse_pkgfile(&pkgfile);

                let selected_installation_lines = if cfg!(windows) {
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
                    execute_lines(
                        &selected_installation_lines,
                        &parsed_pkgfile.data,
                        Some(&package_directory),
                        quiet,
                    );

                    match remove_dir_all(package_directory) {
                        Ok(_) => {}
                        Err(error) => {
                            eprintln!(
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

                    if !quiet {
                        println!("{}", "+ Adding Package to the Lockfile...".bright_green());
                    }

                    let aati_lock_path_buf = get_aati_lock_path_buf();

                    let lock_file_str = match read_to_string(&aati_lock_path_buf) {
                        Ok(contents) => contents,
                        Err(error) => {
                            eprintln!(
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
                        name: name.into(),
                        version: version.into(),
                        source: source.into(),
                        target: get_target(),
                        pkgfile: parsed_pkgfile.clone(),
                    };

                    lock_file.package.push(package);

                    let mut file = match OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(&aati_lock_path_buf)
                    {
                        Ok(file) => file,
                        Err(error) => {
                            eprintln!(
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
                            eprintln!(
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

                    if !quiet {
                        println!("{}", "+ Installation is complete!".bright_green());
                    }
                } else {
                    if !quiet {
                        println!("{}", "+ Transaction aborted".bright_green());
                    }

                    match remove_dir_all(&package_directory) {
                        Ok(_) => {
                            if !quiet {
                                println!(
                                    "{}",
                                    "+ Deleted temporary package directory".bright_green()
                                )
                            }
                        }
                        Err(error) => {
                            eprintln!(
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
            } else if !quiet {
                println!("{}", "+ Transaction aborted".bright_green());
            }
        }

        Err(error) => {
            eprintln!("{}", format!("- ERROR[11]: {}", error).bright_red());
            exit(1);
        }
    }
}

pub fn use_pkgfile(
    path_str: &str,
    provided_name: Option<&String>,
    provided_version: Option<&String>,
    force: bool,
    quiet: bool,
) -> Result<(), String> {
    let pkgfile_path_buf = PathBuf::from(path_str);

    if pkgfile_path_buf.exists() {
        match read_to_string(&pkgfile_path_buf) {
            Ok(pkgfile) => {
                let aati_lock: Value = get_aati_lock().parse().unwrap();
                let installed_packages = aati_lock["package"].as_array().unwrap();

                let package_directory = pkgfile_path_buf.parent().unwrap().to_path_buf();

                let parsed_pkgfile = parse_pkgfile(&pkgfile);

                let name = if let Some(name) = parsed_pkgfile.data.get("name") {
                    name
                } else if let Some(name) = provided_name {
                    name
                } else {
                    return Err(
                        "Package name not provided by the PKGFILE nor as a command line argument!"
                            .into(),
                    );
                };

                let version = if let Some(version) = parsed_pkgfile.data.get("version") {
                    version
                } else if let Some(version) = provided_version {
                    version
                } else {
                    return Err("Package version not provided by the PKGFILE nor as a command line argument!".into());
                };

                if installed_packages
                    .iter()
                    .any(|pkg| pkg["name"].as_str().unwrap() == name)
                {
                    if force || prompt_yn("There's a package with the same name already installed! Do you want to remove the original and proceed?") {
                        remove::command(name, force, quiet);
                    } else {
                        return Ok(())
                    }
                }

                let selected_installation_lines = if cfg!(windows) {
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
                    execute_lines(
                        &selected_installation_lines,
                        &parsed_pkgfile.data,
                        Some(&package_directory),
                        quiet,
                    );
                } else {
                    return Ok(());
                }

                if !quiet {
                    println!("{}", "+ Adding Package to the Lockfile...".bright_green());
                }

                let aati_lock_path_buf = get_aati_lock_path_buf();

                let lock_file_str = match read_to_string(&aati_lock_path_buf) {
                    Ok(contents) => contents,
                    Err(error) => {
                        eprintln!(
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
                    name: name.into(),
                    version: version.into(),
                    source: "local".into(),
                    target: get_target(),
                    pkgfile: parsed_pkgfile,
                };

                lock_file.package.push(package);

                let mut file = match OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&aati_lock_path_buf)
                {
                    Ok(file) => file,
                    Err(error) => {
                        return Err(format!(
                            "- FAILED TO OPEN LOCKFILE AT '{}' FOR WRITING! ERROR[80]: {}",
                            &aati_lock_path_buf.display(),
                            error
                        ))
                    }
                };

                let toml_str = toml::to_string(&lock_file).unwrap();
                match file.write_all(toml_str.as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(format!(
                        "- FAILED TO WRITE INTO LOCKFILE AT '{}'! ERROR[2]: {}",
                        &aati_lock_path_buf.display(),
                        error
                    )),
                }
            }

            Err(error) => Err(format!(
                "Failed to read PKGFILE contents at '{}'! ERROR: {}",
                pkgfile_path_buf.display(),
                error
            )),
        }
    } else {
        Err(format!(
            "No PKGFILE found at '{}'",
            pkgfile_path_buf.display()
        ))
    }
}

pub fn parse_filename(mut filename: &str) -> Package {
    // Example Usage: parse_filename("dummy-package-0.1.0.tar.lz4");

    filename = filename.trim();

    if filename.ends_with(".tar.lz4") {
        let package = if let Some((package, _)) = filename.rsplit_once(".tar.lz4") {
            package
        } else {
            eprintln!(
                "{}",
                format!("- FILE '{}' HAS AN INVALID FILENAME!", filename).bright_red()
            );

            exit(1);
        };

        // package's value is now: dummy-package-0.1.0

        let (name, version) = if let Some((name, version)) = package.rsplit_once('-') {
            (name, version)
        } else {
            eprintln!(
                "{}",
                format!(
                    "- FILE '{}' DOESN'T CONTAIN A HYPHEN AS A SEPARATOR!",
                    filename
                )
                .bright_red()
            );

            exit(1);
        };

        // Now: name = "dummy-package", version = "0.1.0"

        Package {
            name: name.into(),
            version: version.into(),
            source: "local".into(),
            target: get_target(),
            pkgfile: Pkgfile {
                data: HashMap::new(),
                installation_lines: vec![],
                win_installation_lines: vec![],
                removal_lines: vec![],
                win_removal_lines: vec![],
            },
        } //         ^^^^^ That's the name of the repo containing locally installed packages.
    } else {
        eprintln!(
            "{}\n{}",
            "- Unidentified file extension!".bright_red(),
            "+ Note: Only .tar.lz4 files are installable.".bright_blue()
        );
        exit(1);
    }
}

#[test]
fn test_parse_filename() {
    let filename1 = "silm-0.3.3.tar.lz4";
    let expected_result1 = Package {
        name: "silm".into(),
        version: "0.3.3".into(),
        source: "local".into(),
        target: get_target(),
        pkgfile: Pkgfile {
            data: HashMap::new(),
            installation_lines: vec![],
            win_installation_lines: vec![],
            removal_lines: vec![],
            win_removal_lines: vec![],
        },
    };

    let filename2 = "arsil-server-0.2.1.tar.lz4";
    let expected_result2 = Package {
        name: "arsil-server".into(),
        version: "0.2.1".into(),
        source: "local".into(),
        target: get_target(),
        pkgfile: Pkgfile {
            data: HashMap::new(),
            installation_lines: vec![],
            win_installation_lines: vec![],
            removal_lines: vec![],
            win_removal_lines: vec![],
        },
    };

    assert_eq!(parse_filename(filename1), expected_result1);
    assert_eq!(parse_filename(filename2), expected_result2);
}
