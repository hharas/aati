use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub source: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LockFile {
    pub package: Vec<Package>,
}
