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
    IgnoreError(ignore::Error),
    IoError(io::Error),
    NotDirectoryError(PathBuf),
    NotSymlinkError(PathBuf),
    PathError(StripPrefixError),
    TomlError(toml::de::Error),
    VarError(env::VarError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::CommandError(ref command, ref message) => {
                write!(fmt, "`{}` failed: {}", command, message)
            }
            Error::FileDoesNotExistError(ref path) => {
                write!(fmt, "`{}` does not exist", path.display())
            }
            Error::FileExistsError(ref path) => write!(fmt, "`{}` already exists", path.display()),
            Error::IgnoreError(ref error) => fmt::Display::fmt(error, fmt),
            Error::IoError(ref error) => fmt::Display::fmt(error, fmt),
            Error::NotDirectoryError(ref path) => {
                write!(fmt, "`{}` is not a directory", path.display())
            }
            Error::NotSymlinkError(ref path) => {
                write!(fmt, "`{}` is not a symlink", path.display())
            }
            Error::PathError(ref error) => fmt::Display::fmt(error, fmt),
            Error::TomlError(ref error) => fmt::Display::fmt(error, fmt),
            Error::VarError(ref error) => fmt::Display::fmt(error, fmt),
        }
    }
}

impl From<ignore::Error> for Error {
    fn from(error: ignore::Error) -> Error {
        Error::IgnoreError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IoError(error)
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

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Error {
        Error::TomlError(error)
    }
}

impl From<env::VarError> for Error {
    fn from(error: env::VarError) -> Error {
        Error::VarError(error)
    }
}
