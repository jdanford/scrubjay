use clap::{App, Arg, ArgMatches, AppSettings, SubCommand};

use super::error::Error;

pub enum Action {
    Install,
    Uninstall,
    Reinstall,
}

pub struct Config {
    pub action: Action,
    pub package_names: Vec<String>,
    pub dry_run: bool,
    pub verbose: bool,
}

pub fn build_app() -> App<'static, 'static> {
    let package_arg = Arg::with_name("packages")
        .value_name("PACKAGE")
        .required(true)
        .min_values(1);

    App::new("Scrubjay")
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(
            "A little tool to keep your config files all nice and orderly",
        )
        .arg(Arg::with_name("dry-run").short("d").long("dry-run").help(
            "Simulates actions without making any changes",
        ))
        .arg(Arg::with_name("verbose").short("v").long("verbose").help(
            "Enables verbose output",
        ))
        .subcommand(
            SubCommand::with_name("install")
                .about("Installs the provided package(s)")
                .arg(package_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("uninstall")
                .about("Uninstalls the provided package(s)")
                .arg(package_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("reinstall")
                .about("Reinstalls the provided package(s)")
                .arg(package_arg.clone()),
        )
}

impl Config {
    pub fn from_args() -> Result<Config, Error> {
        let app = build_app();
        let matches = app.get_matches_safe()?;

        let (action, package_names) = match matches.subcommand() {
            ("install", Some(submatches)) => (Action::Install, package_names(submatches)),
            ("uninstall", Some(submatches)) => (Action::Uninstall, package_names(submatches)),
            ("reinstall", Some(submatches)) => (Action::Reinstall, package_names(submatches)),
            _ => unreachable!(),
        };

        Ok(Config {
            action: action,
            package_names: package_names,
            dry_run: matches.is_present("dry-run"),
            verbose: matches.is_present("verbose"),
        })
    }
}

fn package_names<'a>(matches: &ArgMatches<'a>) -> Vec<String> {
    matches.values_of_lossy("packages").unwrap()
}
