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

use ascii::AsciiString;
use colored::Colorize;
use std::{fs::read_to_string, process::exit};
use toml::Value;

use tiny_http::{Header, Response, Server};

use crate::{
    globals::POSSIBLE_TARGETS,
    utils::{generate_apr_html, prompt},
};

pub fn command(address_option: Option<&str>) {
    let address;
    let website_url;

    if let Some(given_address) = address_option {
        address = given_address;
        website_url = prompt("On what URL will this index be hosted (e.g. http://example.com)?");
    } else {
        address = "localhost:8887";
        website_url = "http://localhost:8887".to_string();
    };

    let repo_url = prompt("On what URL is the package repository hosted?");

    match Server::http(address) {
        Ok(server) => match read_to_string("repo.toml") {
            Ok(repo_toml) => match repo_toml.parse::<Value>() {
                Ok(repo_config) => {
                    let packages = repo_config["index"]["packages"].as_array().unwrap();
                    let targets = POSSIBLE_TARGETS;

                    println!(
                        "{}",
                        format!(
                            "+ Listening on port: {}",
                            server.server_addr().to_ip().unwrap().port()
                        )
                        .bright_green()
                    );

                    for request in server.incoming_requests() {
                        let mut html =
                            generate_apr_html(&repo_config, "index", None, &website_url, &repo_url);
                        let mut url = request.url().to_string();

                        print!(
                            "{}",
                            format!("+   {} {}", request.method(), request.url()).bright_green()
                        );

                        url.remove(0);

                        if url.is_empty() || url == "index.html" {
                            html = generate_apr_html(
                                &repo_config,
                                "index",
                                None,
                                &website_url,
                                &repo_url,
                            );
                        } else if url == "about.html" {
                            html = generate_apr_html(
                                &repo_config,
                                "about",
                                None,
                                &website_url,
                                &repo_url,
                            );
                        } else if url == "packages.html" {
                            html = generate_apr_html(
                                &repo_config,
                                "packages",
                                None,
                                &website_url,
                                &repo_url,
                            );
                        } else {
                            let mut html_assigned = false;

                            for target in &targets {
                                if url == format!("{}/index.html", target)
                                    || url == format!("{}/", target)
                                    || url == *target.to_string()
                                {
                                    html_assigned = true;
                                    html = generate_apr_html(
                                        &repo_config,
                                        target,
                                        None,
                                        &website_url,
                                        &repo_url,
                                    );
                                }
                            }

                            if !html_assigned {
                                for target in &targets {
                                    for package in packages {
                                        let package_name = package["name"].as_str().unwrap();
                                        let package_target = package["target"].as_str().unwrap();

                                        if target == &package_target
                                            && url
                                                == format!(
                                                    "{}/{}/{}.html",
                                                    package_target, package_name, package_name
                                                )
                                        {
                                            html = generate_apr_html(
                                                &repo_config,
                                                "package",
                                                Some(package),
                                                &website_url,
                                                &repo_url,
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        let response = Response::from_string(html);

                        let response = response.with_header(Header {
                            field: "Content-Type".parse().unwrap(),
                            value: AsciiString::from_ascii("text/html; charset=utf8").unwrap(),
                        });

                        match request.respond(response) {
                            Ok(_) => {}

                            Err(error) => {
                                println!("{}", format!("ERROR[18]: {}", error).bright_red());
                                exit(1);
                            }
                        }
                    }
                }

                Err(error) => {
                    println!("{}", format!("ERROR[17]: {}", error).bright_red());
                    exit(1);
                }
            },

            Err(error) => {
                println!("{}", format!("ERROR[16]: {}", error).bright_red());
                exit(1);
            }
        },

        Err(error) => {
            println!("{}", format!("ERROR[15]: {}", error).bright_red());
            exit(1);
        }
    }
}
