use std::process;

use clap;
use colored::*;

use super::package;

#[derive(Debug)]
pub enum Error {
    ArgError(clap::Error),
    PackageError(package::Error),
}

impl Error {
    pub fn exit(self) -> ! {
        match self {
            Error::ArgError(error) => error.exit(),
            Error::PackageError(error) => {
                println!("{} {}", "error:".red().bold(), error);
                process::exit(1)
            }
        }
    }
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
