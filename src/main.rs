extern crate scrubjay;

use std::env;

use scrubjay::package::Package;

fn main() {
    let packages = env::args().skip(1).map(|path| Package::new(path.as_str()));
    for package in packages {
        println!("{:?}", package);
    }
}
