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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// rc.toml

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub sources: SourcesSection,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SourcesSection {
    pub repos: Vec<Repo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repo {
    pub name: String,
    pub url: String,
}

// lock.toml

#[derive(Debug, Deserialize, Serialize)]
pub struct LockFile {
    pub package: Vec<Package>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub target: String,
    pub source: String,
    pub pkgfile: Pkgfile,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Pkgfile {
    pub data: HashMap<String, String>,
    pub installation_lines: Vec<String>,
    pub win_installation_lines: Vec<String>,
    pub removal_lines: Vec<String>,
    pub win_removal_lines: Vec<String>,
}
