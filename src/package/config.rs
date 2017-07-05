use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use toml;

use super::{Error, Result};

pub const DEFAULT_FILENAME: &'static str = ".scrubjay.toml";

#[derive(Debug, Deserialize)]
pub struct Hook {
    pub command: Option<String>,
    pub script: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Hooks {
    pub pre_install: Option<Hook>,
    pub post_install: Option<Hook>,
    pub pre_uninstall: Option<Hook>,
    pub post_uninstall: Option<Hook>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    target: Option<String>,
    hooks: Hooks,
}

impl Config {
    pub fn from_str(toml_str: &str) -> Result<Config> {
        toml::from_str(toml_str).map_err(Error::from)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut file = File::open(path)?;
        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)?;
        Config::from_str(toml_str.as_str())
    }

    pub fn from_dir<P: AsRef<Path>>(directory: P) -> Result<Config> {
        let mut path = PathBuf::new();
        path.push(directory);
        path.push(DEFAULT_FILENAME);
        Config::from_path(path)
    }
}
