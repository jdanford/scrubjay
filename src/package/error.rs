use std::env;
use std::io;
use std::path::{PathBuf, StripPrefixError};
use std::result;

use ignore;
use shellexpand::LookupError;
use toml;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CommandError(String),
    FileDoesNotExistError(PathBuf),
    FileExistsError(PathBuf),
    IoError(io::Error),
    IgnoreError(ignore::Error),
    NotDirectoryError(PathBuf),
    NotSymlinkError(PathBuf),
    PathError(StripPrefixError),
    VarError(env::VarError),
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

impl From<LookupError<env::VarError>> for Error {
    fn from(error: LookupError<env::VarError>) -> Error {
        Error::VarError(error.cause)
    }
}

impl From<StripPrefixError> for Error {
    fn from(error: StripPrefixError) -> Error {
        Error::PathError(error)
    }
}

impl From<env::VarError> for Error {
    fn from(error: env::VarError) -> Error {
        Error::VarError(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Error {
        Error::TomlError(error)
    }
}
