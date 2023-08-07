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

use colored::Colorize;
use std::{
    fs::{read_to_string, OpenOptions},
    io::Write,
    process::exit,
};

use crate::{
    types::LockFile,
    utils::{execute_lines, get_aati_lock_path_buf, prompt_yn},
};

pub fn command(package_name: &str, force: bool) {
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
    let lock_file: LockFile = toml::from_str(&lock_file_str).unwrap();

    println!("{}", "+ Executing removal commands...".bright_green());

    let found_package = match lock_file
        .package
        .iter()
        .find(|pkg| pkg.name == package_name)
    {
        Some(found_package) => found_package,
        None => {
            println!(
                "{}",
                format!("- Package '{}' not found in the Lockfile!", package_name).bright_red()
            );

            exit(1);
        }
    };

    if force
        || prompt_yn(&format!(
            "+ Commands to be ran:\n  {}\n/ Do these commands seem safe to execute?",
            found_package.pkgfile.removal_lines.join("\n  ")
        ))
    {
        execute_lines(
            found_package.pkgfile.removal_lines.clone(),
            found_package.pkgfile.data.clone(),
            None,
        );

        println!(
            "{}",
            "+ Removing package from the Lockfile...".bright_green()
        );

        remove_from_lockfile(package_name);

        println!("{}", "+ Removal finished successfully!".bright_green());
    } else {
        println!("{}", "+ Transaction aborted".bright_green());
    }
}

pub fn remove_from_lockfile(package_name: &str) {
    let aati_lock_path_buf = get_aati_lock_path_buf();
    let lock_file_str = match read_to_string(&aati_lock_path_buf) {
        Ok(contents) => contents,
        Err(error) => {
            println!(
                "{}",
                format!(
                    "- FAILED TO READ LOCKFILE AT '{}'! ERROR[57]: {}",
                    &aati_lock_path_buf.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };
    let mut lock_file: LockFile = toml::from_str(&lock_file_str).unwrap();

    lock_file.package.retain(|pkg| pkg.name != package_name);

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
}
