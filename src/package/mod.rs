pub mod config;
pub mod error;

use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use std::process::Command;

use ignore::{Walk, WalkBuilder};
use ignore::overrides::{Override, OverrideBuilder};

pub use self::config::{Config, Hook};
pub use self::error::{Error, Result};

use super::Config as ProgramConfig;

#[derive(Debug)]
pub struct Package<'a> {
    path: PathBuf,
    config: Config,
    program_config: &'a ProgramConfig,
}

impl<'a> Package<'a> {
    pub fn new<P: AsRef<Path>>(relative_path: P, program_config: &ProgramConfig) -> Result<Package> {
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

    fn build_walker(&self) -> Result<Walk> {
        let overrides = self.build_overrides()?;
        Ok(WalkBuilder::new(&self.path).hidden(false).git_global(true).overrides(overrides).build())
    }

    fn build_overrides(&self) -> Result<Override> {
        let mut builder = OverrideBuilder::new(&self.path);
        builder.add(config::DEFAULT_FILENAME)?;

        for script_name in self.config.script_names() {
            builder.add(script_name);
        }

        Ok(builder.build()?)
    }

    fn run_hook(&self, hook: &Hook) -> Result<()> {
        if let Some(ref command) = hook.command {
            self.run_command(Command::new("sh").arg("-c").arg(command).current_dir(&self.path))
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
