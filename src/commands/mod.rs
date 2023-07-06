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

mod generate;
mod get;
mod info;
mod install;
mod list;
mod package;
mod remove;
mod repo;
mod serve;
mod sync;
mod upgrade;

pub fn get(package_name: &str) {
    get::command(package_name);
}

pub fn upgrade(choice: Option<&str>) {
    upgrade::command(choice);
}

pub fn remove(package_name: &str) {
    remove::command(package_name);
}

pub fn list(choice_option: Option<&str>) {
    list::command(choice_option);
}

pub fn sync() {
    sync::command();
}

pub fn repo(first_argument_option: Option<&str>, second_argument_option: Option<&str>) {
    repo::command(first_argument_option, second_argument_option);
}

pub fn info(text: &str, repo_name: Option<&str>) {
    info::command(text, repo_name);
}

pub fn package(directory_name: String) {
    package::command(directory_name);
}

pub fn install(filename: &str) {
    install::command(filename);
}

pub fn generate() {
    generate::command();
}

pub fn serve(address_option: Option<&str>) {
    serve::command(address_option);
}
