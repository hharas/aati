use colored::Colorize;
use lz4::Decoder;
use std::{
    env::temp_dir,
    fs::{read_to_string, remove_dir_all, remove_file, File, OpenOptions},
    io::{copy, Write},
    path::PathBuf,
    process::exit,
};
use tar::Archive;

use crate::{
    commons::{
        execute_lines, get_aati_lock, get_aati_lock_path_buf, parse_filename, parse_pkgfile,
        prompt_yn,
    },
    types::{LockFile, Package},
};

pub fn command(filename: &str) {
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

                    let pkgfile = match read_to_string(&pkgfile_path_buf) {
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

                    let lock_file_str = match read_to_string(&aati_lock_path_buf) {
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
