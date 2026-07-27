use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct ProjectFile {
    pub project: Project,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub programmer: Programmer,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub device: String,
    pub clock_hz: u32,
    pub target_vdd_mv: u16,
}

#[derive(Debug, Deserialize)]
pub struct Build {
    #[serde(default = "default_sources")]
    pub sources: Vec<PathBuf>,
}

impl Default for Build {
    fn default() -> Self {
        Self {
            sources: default_sources(),
        }
    }
}

fn default_sources() -> Vec<PathBuf> {
    vec![PathBuf::from("src/main.c")]
}

#[derive(Debug, Default, Deserialize)]
pub struct Programmer {
    pub port: Option<String>,
}

impl ProjectFile {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("pdk.toml");
        let input = fs::read_to_string(&path).map_err(|source| Error::Read {
            path: path.clone(),
            source,
        })?;
        toml::from_str(&input).map_err(|source| Error::Config { path, source })
    }
}
