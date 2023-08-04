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
    collections::HashMap,
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::PathBuf,
    process::exit,
};
use toml::Value;

use crate::globals::POSSIBLE_TARGETS;

pub fn command(base_url: &str, repo_url: &str) {
    match read_to_string("repo.toml") {
        Ok(repo_toml) => match repo_toml.parse::<Value>() {
            Ok(repo_config) => {
                let available_packages = repo_config["index"]["packages"].as_array().unwrap();
                let targets = POSSIBLE_TARGETS;

                let mut html_files: HashMap<PathBuf, String> = HashMap::new();

                html_files.insert(
                    PathBuf::from("index.html"),
                    generate_apr_html(&repo_config, "index", None, base_url, repo_url),
                );

                html_files.insert(
                    PathBuf::from("packages.html"),
                    generate_apr_html(&repo_config, "packages", None, base_url, repo_url),
                );

                html_files.insert(
                    PathBuf::from("about.html"),
                    generate_apr_html(&repo_config, "about", None, base_url, repo_url),
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
                                    create_dir_all(format!(
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
                                        base_url,
                                        repo_url,
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
                                base_url,
                                repo_url,
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

pub fn generate_apr_html(
    repo_config: &Value,
    template: &str,
    current_package: Option<&Value>,
    base_url: &str,
    repo_url: &str,
) -> String {
    let base_url = if base_url == "/" {
        ""
    } else if base_url.ends_with('/') {
        base_url.get(..1).unwrap()
    } else {
        base_url
    };

    let repo_name = repo_config["repo"]["name"].as_str().unwrap();
    let repo_description = repo_config["repo"]["description"].as_str().unwrap();
    let repo_maintainer = repo_config["repo"]["maintainer"].as_str().unwrap();
    let available_packages = repo_config["index"]["packages"].as_array().unwrap();

    let mut response = "<!DOCTYPE html><html lang=\"en\">".to_string();

    let mut head = format!("<head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><meta property=\"og:site_name\" content=\"{}\" /><meta property=\"og:type\" content=\"website\" /><meta property=\"twitter:card\" content=\"summary\" /><meta name=\"description\" content=\"{}\"><style>table, th, td {{ border: 1px solid black; border-collapse: collapse; padding: 5px; }} .installation_guide {{ background-color: #f0f0f0; }}</style>", repo_name, repo_description);
    let mut header;

    if template == "index" {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, base_url, base_url, base_url
        );

        head.push_str(&format!("<meta property=\"og:title\" content=\"index\" /><meta property=\"og:url\" content=\"{}\" /><meta property=\"og:description\" content=\"{}\" />", base_url, repo_description));
        head.push_str(&format!("<title>{}</title></head>", repo_name));
        header.push_str(&format!("<p>{}</p>", repo_description));
        header.push_str(&format!("<p>Add this Package Repository in Aati by running:</p><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati repo add {}</code>", repo_url));
    } else if template == "packages" {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, base_url, base_url, base_url
        );

        header.push_str(&format!(
            "<p>Number of packages: <b>{}</b></p>",
            available_packages.len()
        ));

        let targets = POSSIBLE_TARGETS;

        header.push_str("<ul>");
        for target in targets {
            if available_packages
                .iter()
                .any(|package| package["target"].as_str().unwrap() == target)
            {
                header.push_str(&format!(
                    "<li><code style=\"font-size: 0.9rem;\"><a href=\"{}/{}\">{}</a></code><ul>",
                    base_url, target, target
                ));
                for package in available_packages {
                    let package_name = package["name"].as_str().unwrap();
                    let package_version = package["current"].as_str().unwrap();
                    let package_target = package["target"].as_str().unwrap();
                    if target == package_target {
                        header.push_str(&format!(
                            "<li><a href=\"{}/{}/{}/{}.html\"><b>{}</b>-{}</a></li>",
                            base_url,
                            package_target,
                            package_name,
                            package_name,
                            package_name,
                            package_version,
                        ));
                    }
                }
                header.push_str("</ul></li>");
            }
        }
        header.push_str("</ul>");

        head.push_str(&format!("<meta property=\"og:title\" content=\"packages\" /><meta property=\"og:url\" content=\"{}/packages.html\" /><meta property=\"og:description\" content=\"{} packages available to install\" />", base_url, available_packages.len()));
        head.push_str(&format!("<title>packages - {}</title></head>", repo_name));
    } else if template == "about" {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, base_url, base_url, base_url
        );

        header.push_str(&format!(
            "<p>{}</p><p>Number of packages: <b>{}</b></p><p>Maintained by: <b>{}</b></p><hr /><p>Generated using the <a href=\"https://github.com/hharas/aati\">Aati Package Manager</a> as a hosted Aati Package Repository.</p>",
            repo_description,
            available_packages.len(),
            repo_maintainer
        ));

        head.push_str(&format!("<meta property=\"og:title\" content=\"about\" /><meta property=\"og:url\" content=\"{}/about.html\" /><meta property=\"og:description\" content=\"about {}\" />", base_url, repo_name));
        head.push_str(&format!("<title>about - {}</title></head>", repo_name));
    } else if template == "package" {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, base_url, base_url, base_url
        );

        if let Some(package) = current_package {
            let package_name = package["name"].as_str().unwrap();
            let package_version = package["current"].as_str().unwrap();
            let package_target = package["target"].as_str().unwrap();
            let package_versions = package["versions"].as_array().unwrap();
            let package_author = package["author"].as_str().unwrap();
            let package_description = package["description"].as_str().unwrap();
            let package_url = package["url"].as_str().unwrap();

            header.push_str(&format!(
                "<h3>Package: <code style=\"font-size: 1rem;\">{}</code></h3>",
                package_name
            ));

            header.push_str(&format!(
                "<div class=\"installation_guide\"><p>You can install this package by:</p><ol><li>Adding this package repository to Aati by running:<br/><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati repo add {}</code></li><li>Then telling Aati to fetch it for you by running:<br /><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati get {}/{}</code></li></ol>or you can download the version you want of this package below and install it locally by running:<br /><code>&nbsp;&nbsp;&nbsp;&nbsp;$ aati install {}-<i>version</i>.tar.lz4</code></div><br />",
                repo_url,
                repo_name,
                package_name,
                package_name
            ));

            header.push_str(&format!(
                "Made by: <b>{}</b>, targeted for <b><code style=\"font-size: 0.9rem;\">{}</code></b>.",
                package_author, package_target
            ));
            header.push_str(&format!(
                "<p>Description: <b>{}</b></p>",
                package_description
            ));

            header.push_str(&format!(
                "<p>URL: <a href=\"{}\">{}</a></p>",
                package_url, package_url
            ));

            header.push_str(&format!(
                "<p>Current version: <b>{}</b></p>",
                package_version
            ));
            header.push_str("<p>Available versions:</p>");

            header.push_str("<table><tr><th>Version</th><th>Changes</th><th>SHA256 Checksum</th><th>Release date</th></tr>");
            for version in package_versions {
                let version_table = version.as_table().unwrap();
                let tag = version_table.get("tag").unwrap().as_str().unwrap();
                let checksum = version_table.get("checksum").unwrap().as_str().unwrap();

                match version_table.get("date") {
                    Some(date) => match version_table.get("changes") {
                        Some(changes) => {
                            let changes = changes.as_str().unwrap();
                            header.push_str(&format!(
                                "<tr><td><a href=\"{}/{}/{}/{}-{}.tar.lz4\">{}</a></td><td><pre><code>{}</code></pre></td><td>{}</td><td>{}</td></tr>",
                                repo_url, package_target, package_name, package_name, tag, tag, changes, checksum, date.as_str().unwrap()
                            ));
                        }

                        None => {
                            header.push_str(&format!(
                                "<tr><td><a href=\"{}/{}/{}/{}-{}.tar.lz4\">{}</a></td><td><b>Unavailable</b></td><td>{}</td><td>{}</td></tr>",
                                repo_url, package_target, package_name, package_name, tag, tag, checksum, date.as_str().unwrap()
                            ));
                        }
                    },

                    None => match version_table.get("changes") {
                        Some(changes) => {
                            let changes = changes.as_str().unwrap();
                            header.push_str(&format!(
                                "<tr><td><a href=\"{}/{}/{}/{}-{}.tar.lz4\">{}</a></td><td><pre><code>{}</code></pre></td><td>{}</td><td><b>Unavailable</b></td></tr>",
                                repo_url, package_target, package_name, package_name, tag, tag, changes, checksum
                            ));
                        }

                        None => {
                            header.push_str(&format!(
                                "<tr><td><a href=\"{}/{}/{}/{}-{}.tar.lz4\">{}</a></td><td><b>Unavailable</b></td><td>{}</td><td><b>Unavailable</b></td></tr>",
                                repo_url, package_target, package_name, package_name, tag, tag, checksum
                            ));
                        }
                    },
                }
            }
            header.push_str("</table>");

            head.push_str(&format!("<meta property=\"og:title\" content=\"{}/{}\" /><meta property=\"og:url\" content=\"{}/{}/{}.html\" /><meta property=\"og:description\" content=\"{}\" />", repo_name, package_name, base_url, package_target, package_name, package_description));
            head.push_str(&format!(
                "<title>{}/{}</title></head>",
                repo_name, package_name
            ));
        }
    } else {
        header = format!(
            "<body><h3><code>{}</code> - aati package repository</h3><a href=\"{}/index.html\">home</a> - <a href=\"{}/packages.html\">packages</a> - <a href=\"{}/about.html\">about</a><hr />",
            repo_name, base_url, base_url, base_url
        );

        let target = template;

        // Borrow Checker headache, had to do all this
        let mut these_available_packages = available_packages.clone();
        let retained_available_packages: &mut Vec<Value> = these_available_packages.as_mut();
        retained_available_packages.retain(|package| package["target"].as_str().unwrap() == target);

        header.push_str(&format!(
            "<p>Number of <code style=\"font-size: 0.9rem;\">{}</code>-targeted packages: <b>{}</b></p>",
            target,
            retained_available_packages.len()
        ));

        if retained_available_packages
            .iter()
            .any(|package| package["target"].as_str().unwrap() == target)
        {
            header.push_str("<ul>");
            for package in available_packages {
                let package_name = package["name"].as_str().unwrap();
                let package_version = package["current"].as_str().unwrap();
                let package_target = package["target"].as_str().unwrap();
                if target == package_target {
                    header.push_str(&format!(
                        "<li><a href=\"{}/{}/{}/{}.html\"><b>{}</b>-{}</a></li>",
                        base_url,
                        package_target,
                        package_name,
                        package_name,
                        package_name,
                        package_version,
                    ));
                }
            }
            header.push_str("</ul>");
        }

        head.push_str(&format!("<meta property=\"og:title\" content=\"{} packages\" /><meta property=\"og:url\" content=\"{}/{}\" /><meta property=\"og:description\" content=\"{} {} packages available to install\" />", target, base_url, target, retained_available_packages.len(), target));
        head.push_str(&format!(
            "<title>{} packages - {}</title></head>",
            target, repo_name
        ));
    }

    response.push_str(&head);
    response.push_str(&header);
    response.push_str("</body></html>");

    response
}
