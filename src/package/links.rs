use std::path::PathBuf;

use ignore::{DirEntry, Walk};

use super::{Error, Result};
use super::Package;

pub struct Link {
    pub entry: DirEntry,
    pub target_path: PathBuf,
}

pub struct Links<'a> {
    package: &'a Package<'a>,
    walker: Walk,
}

impl<'a> Links<'a> {
    pub fn new(package: &'a Package) -> Result<Links<'a>> {
        let mut walker = package.build_walker()?;
        walker.next().unwrap()?;

        Ok(Links {
            package: package,
            walker: walker,
        })
    }
}

impl<'a> Iterator for Links<'a> {
    type Item = Result<Link>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.walker.next() {
            Some(Ok(entry)) => {
                if entry.path() == &self.package.path {
                    self.next()
                } else {
                    let link_result = self.package.target_path(entry.path()).map(|target_path| {
                        Link {
                            entry: entry,
                            target_path: target_path,
                        }
                    });
                    Some(link_result)
                }
            }
            Some(Err(error)) => Some(Err(Error::IgnoreError(error))),
            None => None,
        }
    }
}
