/* بسم الله الرحمن الرحيم

   Aati - Cross-platform Package Manager written in Rust.
   Copyright (C) 2023  Husayn Haras

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
use std::{
    fs::{read_to_string, OpenOptions},
    io::Write,
    process::exit,
};

use crate::{
    commons::{execute_lines, get_aati_lock, get_aati_lock_path_buf, prompt_yn},
    types::LockFile,
};

pub fn command(package_name: &str) {
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

                let lock_file_str = match read_to_string(&aati_lock_path_buf) {
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
                command(package["name"].as_str().unwrap());
            }
        } else {
            println!("{}", "+ Transaction aborted".bright_green());
        }
    } else {
        println!("{}", "+ You have no packages installed!".bright_green());
    }
}
