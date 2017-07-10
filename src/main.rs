extern crate scrubjay;

use std::env;

use scrubjay::config::Config;
use scrubjay::package::Package;
use scrubjay::package::error::Result;

fn install_packages(config: &Config) -> Result<()> {
    let package_names = env::args().skip(1);
    for package_name in package_names {
        let package = Package::new(package_name, config)?;
        package.install()?;
    }

    Ok(())
}

fn main() {
    let config = Config { dry_run: true };
    if let Err(err) = install_packages(&config) {
        println!("Error: {:?}", err);
    }
}
