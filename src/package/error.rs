use std::io;
use std::path::{PathBuf, StripPrefixError};
use std::result;

use ignore;
use shellexpand;
use toml;

pub type Result<T> = result::Result<T, Error>;

type ShellLookupError = shellexpand::LookupError<String>;

#[derive(Debug)]
pub enum Error {
    CommandError(String),
    IoError(io::Error),
    IgnoreError(ignore::Error),
    NotDirectoryError(PathBuf),
    PathError(StripPrefixError),
    ShellLookupError(ShellLookupError),
    TomlError(toml::de::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IoError(error)
    }
}

impl From<ignore::Error> for Error {
    fn from(error: ignore::Error) -> Error {
        Error::IgnoreError(error)
    }
}

impl From<StripPrefixError> for Error {
    fn from(error: StripPrefixError) -> Error {
        Error::PathError(error)
    }
}

impl From<ShellLookupError> for Error {
    fn from(error: ShellLookupError) -> Error {
        Error::ShellLookupError(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Error {
        Error::TomlError(error)
    }
}
