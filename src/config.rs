use clap::{App, Arg, ArgMatches, AppSettings, SubCommand};

use super::error::Error;

pub enum Action {
    Install,
    Uninstall,
    Reinstall,
}

const ARG_DRY_RUN: &'static str = "dry-run";
const ARG_FORCE: &'static str = "force";
const ARG_PACKAGES: &'static str = "packages";
const ARG_VERBOSE: &'static str = "verbose";

pub struct Config {
    pub action: Action,
    pub package_names: Vec<String>,
    pub dry_run: bool,
    pub force: bool,
    pub verbose: bool,
}

pub fn build_app() -> App<'static, 'static> {
    let packages_arg = Arg::with_name(ARG_PACKAGES)
        .value_name("PACKAGE")
        .required(true)
        .min_values(1);

    let dry_run_arg = Arg::with_name(ARG_DRY_RUN)
        .long(ARG_DRY_RUN)
        .short("n")
        .help("Simulates actions without making any changes");

    let force_arg = Arg::with_name(ARG_FORCE).long(ARG_FORCE).short("f").help(
        "Allows existing files to be overwritten or deleted",
    );

    let verbose_arg = Arg::with_name(ARG_VERBOSE)
        .long(ARG_VERBOSE)
        .short("v")
        .help("Enables verbose output");

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
                .arg(force_arg.clone())
                .arg(verbose_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("uninstall")
                .about("Uninstalls the provided package(s)")
                .arg(packages_arg.clone())
                .arg(dry_run_arg.clone())
                .arg(force_arg.clone())
                .arg(verbose_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("reinstall")
                .about("Reinstalls the provided package(s)")
                .arg(packages_arg.clone())
                .arg(dry_run_arg.clone())
                .arg(force_arg.clone())
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
            dry_run: submatches.is_present(ARG_DRY_RUN),
            force: submatches.is_present(ARG_FORCE),
            verbose: submatches.is_present(ARG_VERBOSE),
        })
    }
}

fn package_names<'a>(matches: &ArgMatches<'a>) -> Vec<String> {
    matches.values_of_lossy(ARG_PACKAGES).expect(
        "Argument specification is inconsistent",
    )
}
