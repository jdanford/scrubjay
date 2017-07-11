extern crate scrubjay;

use std::path::PathBuf;

use scrubjay::config::{Action, Config};
use scrubjay::error::Error;
use scrubjay::package::Package;

fn main_safe() -> Result<(), Error> {
    let config = Config::from_args()?;

    for package_name in config.package_names.iter() {
        let package_path = PathBuf::from(package_name);
        let package = Package::new(&package_path, &config)?;

        match config.action {
            Action::Install => package.install()?,
            Action::Uninstall => package.uninstall()?,
            Action::Reinstall => package.reinstall()?,
        };
    }

    Ok(())
}

fn main() {
    if let Err(error) = main_safe() {
        error.exit();
    }
}
