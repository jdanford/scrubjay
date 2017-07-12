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
    let packages_arg = Arg::with_name("packages")
        .value_name("PACKAGE")
        .required(true)
        .min_values(1);

    let dry_run_arg = Arg::with_name("dry-run").short("d").long("dry-run").help(
        "Simulates actions without making any changes",
    );

    let verbose_arg = Arg::with_name("verbose").short("v").long("verbose").help(
        "Enables verbose output",
    );

    App::new("Scrubjay")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("install")
                .about("Installs the provided package(s)")
                .arg(packages_arg.clone())
                .arg(dry_run_arg.clone())
                .arg(verbose_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("uninstall")
                .about("Uninstalls the provided package(s)")
                .arg(packages_arg.clone())
                .arg(dry_run_arg.clone())
                .arg(verbose_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("reinstall")
                .about("Reinstalls the provided package(s)")
                .arg(packages_arg.clone())
                .arg(dry_run_arg.clone())
                .arg(verbose_arg.clone()),
        )
}

impl Config {
    pub fn from_args() -> Result<Config, Error> {
        let app = build_app();
        let matches = app.get_matches_safe()?;
        let (action, submatches) = match matches.subcommand() {
            ("install", Some(submatches)) => (Action::Install, submatches),
            ("uninstall", Some(submatches)) => (Action::Uninstall, submatches),
            ("reinstall", Some(submatches)) => (Action::Reinstall, submatches),
            _ => unreachable!(),
        };

        Ok(Config {
            action: action,
            package_names: package_names(submatches),
            dry_run: submatches.is_present("dry-run"),
            verbose: submatches.is_present("verbose"),
        })
    }
}

fn package_names<'a>(matches: &ArgMatches<'a>) -> Vec<String> {
    matches.values_of_lossy("packages").unwrap()
}
