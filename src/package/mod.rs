pub mod config;
pub mod error;

use std::fs;
use std::os::unix::fs::symlink;
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
const INDENT: &'static str = "‣ ";

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
                "Installing {} to {}...",
                self.path.display(),
                target_root.display(),
            );
        } else {
            println!(
                "Installing {}...",
                self.path.display(),
            );
        }

        self.try_run_hook(self.config.hooks.pre_install.as_ref())?;

        for link_result in self.links()? {
            let link = link_result?;
            self.create_link(&link)?;
        }

        self.try_run_hook(self.config.hooks.post_install.as_ref())?;

        println!(
            "Installed {}",
            self.path.display(),
        );

        Ok(())
    }

    pub fn uninstall(&self) -> Result<()> {
        if self.config.target.is_some() {
            let target_root = self.target_root()?;
            println!(
                "Uninstalling {} from {}...",
                self.path.display(),
                target_root.display(),
            );
        } else {
            println!(
                "Uninstalling {}...",
                self.path.display(),
            );
        }

        self.try_run_hook(self.config.hooks.pre_uninstall.as_ref())?;

        for link_result in self.links()? {
            let link = link_result?;
            self.remove_link(&link)?;
        }

        self.try_run_hook(self.config.hooks.post_uninstall.as_ref())?;

        println!(
            "Uninstalled {}",
            self.path.display(),
        );

        Ok(())
    }

    pub fn reinstall(&self) -> Result<()> {
        if self.config.target.is_some() {
            let target_root = self.target_root()?;
            println!(
                "Reinstalling {} to {}...",
                self.path.display(),
                target_root.display(),
            );
        } else {
            println!(
                "Reinstalling {}...",
                self.path.display(),
            );
        }

        self.try_run_hook(self.config.hooks.pre_uninstall.as_ref())?;

        for link_result in self.links()? {
            let link = link_result?;
            self.remove_link(&link)?;
        }

        self.try_run_hook(self.config.hooks.post_uninstall.as_ref())?;
        self.try_run_hook(self.config.hooks.pre_install.as_ref())?;

        for link_result in self.links()? {
            let link = link_result?;
            self.create_link(&link)?;
        }

        self.try_run_hook(self.config.hooks.post_install.as_ref())?;

        println!(
            "Reinstalled {}",
            self.path.display(),
        );
        Ok(())
    }

    fn create_link(&self, link: &Link) -> Result<()> {
        if !self.program_config.dry_run {
            if link.target_path.exists() {
                return Err(Error::FileExistsError(link.target_path.clone()));
            }

            let source_path = link.entry.path();
            symlink(source_path, &link.target_path)?;
        }

        println!("{}Created {}", INDENT, link.target_path.display());
        Ok(())
    }

    fn remove_link(&self, link: &Link) -> Result<()> {
        if !self.program_config.dry_run {
            if !is_symlink(&link.target_path)? {
                return Err(Error::NotSymlinkError(link.target_path.clone()));
            } else if link.target_path.is_dir() {
                fs::remove_dir_all(&link.target_path)?;
            } else {
                fs::remove_file(&link.target_path)?;
            }
        }

        println!("{}Removed {}", INDENT, link.target_path.display());
        Ok(())
    }

    fn try_run_hook(&self, hook_option: Option<&Hook>) -> Result<()> {
        if let Some(hook) = hook_option {
            self.run_hook(hook)?;
        }

        Ok(())
    }

    fn run_hook(&self, hook: &Hook) -> Result<()> {
        if let Some(ref command_str) = hook.command {
            println!("{}Running command {}", INDENT, command_str);

            if !self.program_config.dry_run {
                self.run_command(Command::new("sh").arg("-c").arg(command_str).current_dir(
                    &self.path,
                ))
            } else {
                Ok(())
            }
        } else if let Some(ref script) = hook.script {
            let script_path = self.path.join(script);
            println!("{}Running script {}", INDENT, script_path.display());

            if !self.program_config.dry_run {
                self.run_command(Command::new(script_path).current_dir(&self.path))
            } else {
                Ok(())
            }
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
}

fn add_ignore_glob(builder: &mut OverrideBuilder, glob: &str) -> Result<()> {
    let inverted_glob = format!("!{}", glob);
    builder.add(&inverted_glob)?;
    Ok(())
}

fn is_symlink<P: AsRef<Path>>(path: P) -> Result<bool> {
    let metadata = fs::symlink_metadata(path)?;
    Ok(metadata.file_type().is_symlink())
}
