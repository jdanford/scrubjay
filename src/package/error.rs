use std::env;
use std::fmt;
use std::io;
use std::path::{PathBuf, StripPrefixError};
use std::result;

use ignore;
use shellexpand::LookupError;
use toml;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CommandError(String, String),
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::CommandError(ref command, ref message) => {
                write!(f, "`{}` failed: {}", command, message)
            }
            Error::FileDoesNotExistError(ref path) => {
                write!(f, "`{}` does not exist", path.display())
            }
            Error::FileExistsError(ref path) => write!(f, "`{}` already exists", path.display()),
            Error::IoError(ref error) => fmt::Display::fmt(error, f),
            Error::IgnoreError(ref error) => fmt::Display::fmt(error, f),
            Error::NotDirectoryError(ref path) => {
                write!(f, "`{}` is not a directory", path.display())
            }
            Error::NotSymlinkError(ref path) => write!(f, "`{}` is not a symlink", path.display()),
            Error::PathError(ref error) => fmt::Display::fmt(error, f),
            Error::VarError(ref error) => fmt::Display::fmt(error, f),
            Error::TomlError(ref error) => fmt::Display::fmt(error, f),
        }
    }
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
