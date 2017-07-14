#[macro_use]
mod config;
mod error;
mod links;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use colored::*;
use ignore::{Walk, WalkBuilder};
use ignore::overrides::{Override, OverrideBuilder};
use shellexpand;

pub use self::config::{Config, Hook};
pub use self::error::{Error, Result};
pub use self::links::{Link, Links};

use super::Config as ProgramConfig;

const DEFAULT_TARGET: &'static str = "~";
const IGNORE_FILENAME: &'static str = ".ignore";
const INDENT: &'static str = "‣ ";

pub struct Package<'a> {
    path: PathBuf,
    config: Config,
    program_config: &'a ProgramConfig,
}

macro_rules! maybe_run_hook {
    ($self_expr:expr, $hook_expr:expr, $field:ident) => {
        if let Some(ref hook) = hook_field!($hook_expr, $field) {
            $self_expr.run_hook(hook)?;
        }
    }
}

impl<'a> Package<'a> {
    pub fn new(relative_path: &Path, program_config: &'a ProgramConfig) -> Result<Package<'a>> {
        if !relative_path.exists() {
            return Err(Error::FileDoesNotExistError(relative_path.into()));
        }

        let path = fs::canonicalize(relative_path)?;
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

    pub fn install(&self) -> Result<()> {
        if self.config.target.is_some() {
            let target_root = self.target_root()?;
            println!(
                "{} {} {} {}{}",
                "Installing".green(),
                self.path_str(&self.path),
                "to".green(),
                self.path_str(&target_root),
                "...".green(),
            );
        } else {
            println!(
                "{} {}{}",
                "Installing".green(),
                self.path_str(&self.path),
                "...".green(),
            );
        }

        maybe_run_hook!(self, self.config.hooks, pre_install);

        for link_result in self.links()? {
            let link = link_result?;
            self.create_link(&link)?;
        }

        maybe_run_hook!(self, self.config.hooks, post_install);

        println!(
            "{} {}",
            "Installed".green(),
            self.path_str(&self.path),
        );
        Ok(())
    }

    pub fn uninstall(&self) -> Result<()> {
        if self.config.target.is_some() {
            let target_root = self.target_root()?;
            println!(
                "{} {} {} {}{}",
                "Uninstalling".green(),
                self.path_str(&self.path),
                "from".green(),
                self.path_str(&target_root),
                "...".green(),
            );
        } else {
            println!(
                "{} {}{}",
                "Uninstalling".green(),
                self.path_str(&self.path),
                "...".green(),
            );
        }

        maybe_run_hook!(self, self.config.hooks, pre_uninstall);

        for link_result in self.links()? {
            let link = link_result?;
            self.remove_link(&link)?;
        }

        maybe_run_hook!(self, self.config.hooks, post_uninstall);

        println!(
            "{} {}",
            "Uninstalled".green(),
            self.path_str(&self.path),
        );
        Ok(())
    }

    pub fn reinstall(&self) -> Result<()> {
        if self.config.target.is_some() {
            let target_root = self.target_root()?;
            println!(
                "{} {} {} {}{}",
                "Reinstalling".green(),
                self.path_str(&self.path),
                "to".green(),
                self.path_str(&target_root),
                "...".green(),
            );
        } else {
            println!(
                "{} {}{}",
                "Reinstalling".green(),
                self.path_str(&self.path),
                "...".green(),
            );
        }

        maybe_run_hook!(self, self.config.hooks, pre_uninstall);

        for link_result in self.links()? {
            let link = link_result?;
            self.remove_link(&link)?;
        }

        maybe_run_hook!(self, self.config.hooks, post_uninstall);

        maybe_run_hook!(self, self.config.hooks, pre_install);

        for link_result in self.links()? {
            let link = link_result?;
            self.create_link(&link)?;
        }

        maybe_run_hook!(self, self.config.hooks, post_install);

        println!(
            "{} {}",
            "Reinstalled".green(),
            self.path_str(&self.path),
        );
        Ok(())
    }

    fn create_link(&self, link: &Link) -> Result<()> {
        if !self.program_config.dry_run {
            if link.target_path.exists() {
                if self.program_config.force {
                    remove_path(&link.target_path)?;
                } else {
                    return Err(Error::FileExistsError(link.target_path.clone()));
                }
            }

            let source_path = link.entry.path();
            symlink(source_path, &link.target_path)?;
        }

        if self.program_config.verbose {
            println!(
                "{}{} {}",
                INDENT,
                "Created".cyan(),
                self.path_str(&link.target_path)
            );
        }

        Ok(())
    }

    fn remove_link(&self, link: &Link) -> Result<()> {
        if !self.program_config.dry_run {
            if !self.program_config.force && !is_symlink(&link.target_path)? {
                return Err(Error::NotSymlinkError(link.target_path.clone()));
            }

            remove_path(&link.target_path)?;
        }

        if self.program_config.verbose {
            println!(
                "{}{} {}",
                INDENT,
                "Removed".red(),
                self.path_str(&link.target_path)
            );
        }

        Ok(())
    }

    fn run_hook(&self, hook: &Hook) -> Result<()> {
        if let Some(ref command_str) = hook.command {
            self.run_command_str(command_str)
        } else if let Some(ref script_name) = hook.script {
            self.run_script_name(script_name)
        } else {
            Ok(())
        }
    }

    fn run_command_str(&self, command_str: &str) -> Result<()> {
        if self.program_config.verbose {
            println!(
                "{}{} `{}`{}",
                INDENT,
                "Running command".magenta(),
                command_str,
                "...".magenta()
            );
        }

        if !self.program_config.dry_run {
            self.run_command(
                command_str,
                Command::new("sh")
                    .arg("-c")
                    .arg(command_str)
                    .current_dir(&self.path),
            )?;
        }

        Ok(())
    }

    fn run_command(&self, command_str: &str, command: &mut Command) -> Result<()> {
        let output = command.output()?;
        if output.status.success() {
            Ok(())
        } else {
            let message = String::from_utf8_lossy(&output.stderr).into_owned();
            Err(Error::CommandError(command_str.to_owned(), message))
        }
    }

    fn run_script_name(&self, script_name: &str) -> Result<()> {
        let script_path = self.path.join(script_name);

        if self.program_config.verbose {
            println!(
                "{}{} {}{}",
                INDENT,
                "Running script".magenta(),
                self.path_str(&script_path),
                "...".magenta()
            );
        }

        if !self.program_config.dry_run {
            self.run_command(
                script_name,
                Command::new(script_path).current_dir(&self.path),
            )?;
        }

        Ok(())
    }

    pub fn links(&'a self) -> Result<Links<'a>> {
        Links::new(&self)
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
                .max_depth(Some(1))
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

    fn path_str(&'a self, absolute_path: &'a Path) -> ColoredString {
        let path = if absolute_path == self.path {
            absolute_path
        } else if let Ok(relative_path) = absolute_path.strip_prefix(&self.path) {
            relative_path
        } else {
            absolute_path
        };

        let raw_str = path.to_string_lossy();
        let safe_str = if let Some(_) = raw_str.find(char::is_whitespace) {
            format!("`{}`", raw_str)
        } else {
            raw_str.to_string()
        };

        safe_str.bold()
    }
}

fn add_ignore_glob(builder: &mut OverrideBuilder, glob: &str) -> Result<()> {
    let inverted_glob = format!("!{}", glob);
    builder.add(&inverted_glob)?;
    Ok(())
}

fn is_symlink(path: &Path) -> Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    Ok(metadata.file_type().is_symlink())
}

fn remove_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        Ok(fs::remove_dir_all(path)?)
    } else {
        Ok(fs::remove_file(path)?)
    }
}
