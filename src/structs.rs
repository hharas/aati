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
    pub url: String,
}

// lock.toml

#[derive(Debug, Deserialize, Serialize)]
pub struct LockFile {
    pub package: Vec<Package>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub source: String,
    pub version: String,
}
