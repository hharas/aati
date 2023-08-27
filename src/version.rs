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

use toml::Value;

// Get aati changelog & version from an external file
// instead of hardcoding it
const CHANGELOG: &str = include_str!("../Changelog.toml");

// More accurately should be called get_tag() but it is what it is
pub fn get_version() -> String {
    let changelog_toml: Value = CHANGELOG.parse().unwrap();
    let versions = changelog_toml["version"].as_array().unwrap();
    let version = versions.first().unwrap();
    let tag = version["tag"].as_str().unwrap();
    tag.into()
}

// Parse the Changelog.toml file in the project's root directory
pub fn get_versions() -> Vec<Value> {
    let changelog_toml: Value = CHANGELOG.parse().unwrap();
    let versions = changelog_toml["version"].as_array().unwrap();
    versions.to_owned()
}
