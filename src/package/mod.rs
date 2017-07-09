pub mod config;
pub mod error;

use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use std::process::Command;

use ignore::{DirEntry, Walk, WalkBuilder};
use ignore::overrides::{Override, OverrideBuilder};
use shellexpand;

pub use self::config::{Config, Hook};
pub use self::error::{Error, Result};

use super::Config as ProgramConfig;

const DEFAULT_TARGET: &'static str = "~";
const IGNORE_FILENAME: &'static str = ".ignore";

pub struct Link {
    pub entry: DirEntry,
    pub target_path: PathBuf,
}

pub struct Links<'a> {
    package: &'a Package<'a>,
    walker: Walk,
}

impl<'a> Links<'a> {
    fn new(package: &'a Package) -> Result<Links<'a>> {
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
            Some(Err(err)) => Some(Err(Error::IgnoreError(err))),
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct Package<'a> {
    path: PathBuf,
    config: Config,
    program_config: &'a ProgramConfig,
}

impl<'a> Package<'a> {
    pub fn new<P: AsRef<Path>>(
        relative_path: P,
        program_config: &ProgramConfig,
    ) -> Result<Package> {
        let path = canonicalize(relative_path)?;
        if !path.is_dir() {
            return Err(Error::NotDirectoryError(path));
        }

        let config = Config::from_dir(&path)?;

        Ok(Package {
            path: path,
            config: config,
            program_config: program_config,
        })
    }

    pub fn links(&'a self) -> Result<Links<'a>> {
        Links::new(&self)
    }

    pub fn print_links(&self) -> Result<()> {
        for link_result in self.links()? {
            let link = link_result?;
            let source_path = link.entry.path();
            println!(
                "{} => {}",
                source_path.to_string_lossy(),
                link.target_path.to_string_lossy()
            );
        }

        Ok(())
    }

    fn target_root(&self) -> Result<PathBuf> {
        let path_str = self.config.target.as_ref().map_or(
            DEFAULT_TARGET,
            String::as_str,
        );
        let full_path_str = shellexpand::full(path_str)?.into_owned();
        Ok(PathBuf::from(full_path_str))
    }

    fn target_path(&self, source_path: &Path) -> Result<PathBuf> {
        let relative_path = source_path.strip_prefix(&self.path)?;
        let target_root = self.target_root()?;
        Ok(target_root.join(relative_path))
    }

    fn build_walker(&self) -> Result<Walk> {
        let overrides = self.build_overrides()?;
        Ok(
            WalkBuilder::new(&self.path)
                .hidden(false)
                .git_global(true)
                .overrides(overrides)
                .build(),
        )
    }

    fn build_overrides(&self) -> Result<Override> {
        let mut builder = OverrideBuilder::new(&self.path);
        add_ignore_glob(&mut builder, IGNORE_FILENAME)?;
        add_ignore_glob(&mut builder, config::DEFAULT_FILENAME)?;

        for script_name in self.config.script_names() {
            add_ignore_glob(&mut builder, script_name)?;
        }

        Ok(builder.build()?)
    }

    #[allow(dead_code)]
    fn run_hook(&self, hook: &Hook) -> Result<()> {
        if let Some(ref command_str) = hook.command {
            self.run_command(Command::new("sh").arg("-c").arg(command_str).current_dir(
                &self.path,
            ))
        } else if let Some(ref script) = hook.script {
            let script_path = self.path.join(script);
            self.run_command(Command::new(script_path).current_dir(&self.path))
        } else {
            Ok(())
        }
    }

    fn run_command(&self, command: &mut Command) -> Result<()> {
        let output = command.output()?;
        if output.status.success() {
            Ok(())
        } else {
            let message = String::from_utf8_lossy(&output.stderr).into_owned();
            Err(Error::CommandError(message))
        }
    }
}

fn add_ignore_glob(builder: &mut OverrideBuilder, glob: &str) -> Result<()> {
    let inverted_glob = format!("!{}", glob);
    let _ = builder.add(&inverted_glob)?;
    Ok(())
}
