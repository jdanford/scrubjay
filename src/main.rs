extern crate scrubjay;

use std::env;

use scrubjay::config::Config;
use scrubjay::package::Package;

fn main() {
    let config = Config {};
    let packages = env::args().skip(1).map(|path| Package::new(path.as_str(), &config));
    for package in packages {
        println!("{:?}", package);
    }
}
