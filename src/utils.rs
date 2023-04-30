use std::fs;
use std::fs::read_to_string;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use colored::Colorize;
use ring::digest;

pub fn get_arch() -> String {
    if cfg!(target_arch = "x86_64") {
        "x86-64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "aarch64".to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn check_config_dir() {
    let config_dir = dirs::home_dir().unwrap().join(".config");
    let aati_config_dir = dirs::home_dir().unwrap().join(".config/aati");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).unwrap();
    }

    if !aati_config_dir.exists() {
        fs::create_dir_all(&aati_config_dir).unwrap();
    }
}

pub fn get_aati_lock() -> Option<String> {
    check_config_dir();

    let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");

    let aati_lock_path_buf = home_dir.join(".config/aati/lock.toml");

    let aati_lock_path = Path::new(&aati_lock_path_buf);

    if !Path::exists(aati_lock_path) {
        let mut aati_lock_file =
            File::create(aati_lock_path).expect("- UNABLE TO CREATE ~/.config/aati/lock.toml");

        let default_config = "package = []";
        writeln!(aati_lock_file, "{}", default_config)
            .expect("- CAN'T WRITE INTO ~/.config/aati/lock.toml");
        println!("  * Make sure to add ~/.local/bin to PATH. You can do this by appending this at the end of our .bashrc file:\n\n    export PATH=\"$HOME/.local/bin:$PATH\"");
    }

    let aati_lock =
        read_to_string(aati_lock_path).expect("UNABLE TO READ ~/.config/aati/lock.toml!");

    Some(aati_lock.trim().to_string())
}

pub fn get_repo_config() -> Option<String> {
    check_config_dir();

    let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");

    let repo_config_path_buf = home_dir.join(".config/aati/repo.toml");

    let repo_config_path = Path::new(&repo_config_path_buf);

    if !Path::exists(repo_config_path) {
        println!(
            "{}",
            "- NO REPO CONFIG FOUND! PLEASE RUN: $ aati repo <repo url>".bright_red()
        );
        exit(1)
    }

    let repo_config =
        read_to_string(repo_config_path).expect("UNABLE TO READ ~/.config/aati/repo.toml!");

    Some(repo_config.trim().to_string())
}

pub fn get_aati_config() -> Option<String> {
    check_config_dir();

    let home_dir = dirs::home_dir().expect("- CAN'T GET USER'S HOME DIRECTORY");

    let aati_config_path_buf = home_dir.join(".config/aati/rc.toml");

    let aati_config_path = Path::new(&aati_config_path_buf);

    if !Path::exists(aati_config_path) {
        let mut aati_config_file = File::create(aati_config_path_buf.clone()).unwrap();

        let default_config = "[sources]\nrepos = []";
        writeln!(aati_config_file, "{}", default_config)
            .expect("- CAN'T WRITE INTO ~/.config/aati/rc.toml");
    }

    let aati_config =
        read_to_string(aati_config_path).expect("UNABLE TO READ ~/.config/aati/repo.toml!");

    Some(aati_config.trim().to_string())
}

fn flush_output() {
    io::stdout().flush().unwrap();
}

pub fn prompt(prompt_text: &str) -> String {
    print!("{}", format!("{} ", prompt_text).as_str().bright_blue());
    flush_output();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {}

        Err(error) => {
            println!(
                "{}",
                format!("- DIDN'T RECEIVE VALID INPUT! ERROR[3]: {}", error).bright_red()
            );
            exit(1)
        }
    };

    input.trim().to_string()
}

pub fn prompt_yn(prompt_text: &str) -> bool {
    print!("{}", format!("{} [Y/n] ", prompt_text).as_str().yellow());
    flush_output();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {}

        Err(error) => {
            println!(
                "{}",
                format!("- DIDN'T RECEIVE VALID INPUT! ERROR[4]: {}", error).bright_red()
            );
            exit(1)
        }
    };

    input.trim().is_empty() || input.trim().to_lowercase() == "y"
}

pub fn extract_package(text: &String) -> Vec<String> {
    if !text.contains('/') {
        let (mut name, mut version) = text.rsplit_once('-').unwrap_or((text, text));

        let repo_toml: toml::Value = get_repo_config().unwrap().parse().unwrap();
        let available_packages = repo_toml["index"]["packages"].as_array().unwrap();

        if name == version {
            for available_package in available_packages {
                if available_package["name"].as_str().unwrap() == name {
                    version = available_package["current"].as_str().unwrap();
                }
            }
        } else if !version.chars().next().unwrap().is_ascii_digit() {
            name = text;

            for available_package in available_packages {
                if available_package["name"].as_str().unwrap() == text
                    && available_package["arch"].as_str().unwrap() == get_arch()
                {
                    version = available_package["current"].as_str().unwrap();
                }
            }
        }

        let name_string = name.to_string();
        let version_string = version.to_string();

        vec!["$unprovided$".to_string(), name_string, version_string]
    } else {
        let (repo_name, text_to_be_extracted) = text.split_once('/').unwrap();

        let result = extract_package(&text_to_be_extracted.to_string());

        vec![repo_name.to_string(), result[1].clone(), result[2].clone()]
    }
}

pub fn verify_checksum(body: &[u8], checksum: String) -> bool {
    let hash = digest::digest(&digest::SHA256, body);
    let hex = hex::encode(hash.as_ref());

    hex == checksum
}
