extern crate colored;
extern crate scrubjay;

use std::env;
use std::path::PathBuf;

use colored::*;
use scrubjay::config::Config;
use scrubjay::package::Package;
use scrubjay::package::Result;

const ERROR: &'static str = "Error:";

fn install_packages(config: &Config) -> Result<()> {
    let package_names = env::args().skip(1);
    for package_name in package_names {
        let package_path = PathBuf::from(package_name);
        let package = Package::new(&package_path, config)?;
        package.install()?;
    }

    Ok(())
}

fn main() {
    let config = Config {
        dry_run: true,
        verbose: true,
    };
    if let Err(err) = install_packages(&config) {
        println!("{} {:?}", ERROR.red().bold(), err);
    }
}
