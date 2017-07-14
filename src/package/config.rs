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

impl Hook {
    fn script_name(&self) -> Option<&str> {
        self.script.as_ref().map(String::as_str)
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Hooks {
    pub pre_install: Option<Hook>,
    pub post_install: Option<Hook>,
    pub pre_uninstall: Option<Hook>,
    pub post_uninstall: Option<Hook>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub target: Option<String>,
    pub hooks: Option<Hooks>,
}

macro_rules! hook_field {
    ($hook_expr:expr, $field:ident) => {
        $hook_expr.as_ref().and_then(|ref hooks| hooks.$field.as_ref())
    };
}

impl Config {
    pub fn from_str(toml_str: &str) -> Result<Config> {
        toml::from_str(toml_str).map_err(Error::from)
    }

    pub fn from_file(file: &mut File) -> Result<Config> {
        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)?;
        Config::from_str(toml_str.as_str())
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
        match File::open(path) {
            Ok(ref mut file) => Config::from_file(file),
            Err(_) => Ok(Config::default()),
        }
    }

    pub fn from_dir<P: AsRef<Path>>(directory: P) -> Result<Config> {
        let mut path = PathBuf::new();
        path.push(directory);
        path.push(DEFAULT_FILENAME);
        Config::from_path(path)
    }

    pub fn script_names(&self) -> Vec<&str> {
        let hooks = vec![
            hook_field!(self.hooks, pre_install),
            hook_field!(self.hooks, post_install),
            hook_field!(self.hooks, pre_uninstall),
            hook_field!(self.hooks, post_uninstall),
        ];

        hooks
            .iter()
            .filter_map(|&hook| hook)
            .filter_map(Hook::script_name)
            .collect()
    }
}
