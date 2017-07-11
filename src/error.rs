use clap;

use super::package;

#[derive(Debug)]
pub enum Error {
    ArgError(clap::Error),
    PackageError(package::Error),
    InvalidSubcommand(String),
}

impl From<clap::Error> for Error {
    fn from(error: clap::Error) -> Error {
        Error::ArgError(error)
    }
}

impl From<package::Error> for Error {
    fn from(error: package::Error) -> Error {
        Error::PackageError(error)
    }
}
