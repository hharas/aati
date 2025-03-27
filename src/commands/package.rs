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
use lz4::EncoderBuilder;
use std::{
    fs::{remove_file, File},
    io::copy,
    path::PathBuf,
    process::exit,
};
use tar::Builder;

pub fn command(mut directory_name: String, quiet: bool) {
    if directory_name.ends_with('/') {
        directory_name.pop();
    }

    let source = PathBuf::from(directory_name);
    let tar_destination = PathBuf::from(format!("{}.tar", source.display()));
    let lz4_destination = PathBuf::from(format!("{}.lz4", tar_destination.display()));

    if !quiet {
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
    }

    let file = match File::create(&tar_destination) {
        Ok(file) => file,
        Err(error) => {
            eprintln!(
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
                eprintln!(
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
            eprintln!(
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
            eprintln!(
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
            eprintln!(
                "{}",
                format!("- UNABLE INITIALISE THE LZ4 ENCODER! ERROR[76]: {}", error).bright_red()
            );

            exit(1);
        }
    };

    if !quiet {
        println!("{}", "+ Writing the compressed buffer...".bright_green());
    }

    let mut tarball = match File::open(&tar_destination) {
        Ok(file) => file,
        Err(error) => {
            eprintln!(
                "{}",
                format!(
                    "- FAILED TO OPEN FILE '{}' FOR READING! ERROR[96]: {}",
                    tar_destination.display(),
                    error
                )
                .bright_red()
            );

            exit(1);
        }
    };

    match copy(&mut tarball, &mut encoder) {
        Ok(_) => {}
        Err(error) => {
            eprintln!(
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
            eprintln!(
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

    match remove_file(&tar_destination) {
        Ok(_) => {}
        Err(error) => {
            eprintln!(
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

    if !quiet {
        println!(
            "{}",
            format!("+ Done packaging! See: {}", lz4_destination.display()).bright_green()
        );
    }
}
