# Scrubjay

A little tool to keep your config files all nice and orderly, named after a cool bird that's really good at caching acorns and retrieving them.

<img src="media/scrub-jay-acorn.jpg" alt="Scrub jay carrying acorn" width="400" height="300">

It is inspired by [GNU Stow](https://www.gnu.org/software/stow/) and written in Rust, leaning heavily on [BurntSushi](https://github.com/BurntSushi/)'s handy [ignore](https://github.com/BurntSushi/ripgrep/tree/master/ignore) crate.
At the moment, it has been tested minimally and probably doesn't work on your machine. ✨

## Purpose

It is often desirable to keep configuration files centralized, so that they can be managed with version control or backup software. Instead of symlinking such files manually, you can use Scrubjay as sort of a local package manager, reducing the tedium while also supporting install/uninstall hooks and other nice customizations.

## Installation

The binary name for `scrubjay` is `sj`. Prebuilt binaries are not yet available, and the crate is not yet published, but if you have a new-ish version of Rust then you can clone the project and run `cargo install`.

## Usage

`sj [install|reinstall|uninstall] [FLAGS] <PACKAGE>...`

### Flags
- `-n`/`--dry-run`: Simulates actions without making any changes
- `-f`/`--force`: Allows existing files to be overwritten or deleted
- `-h`/`--help`: Prints help information
- `-v`/`--verbose`: Enables verbose output

## Package configuration

As this tool is local and minimal, a "package" is just a directory tree with some (optional) configuration. At the moment, the only method of configuration is a `.scrubjay.toml` file ([TOML spec](https://github.com/toml-lang/toml)) located at the package root, with the following contents:

### Top-level keys
- `target`: The directory where this package's file will be installed (tildes and environment variables will be expanded)

### Sections
- `hooks.pre_install`
- `hooks.post_install`
- `hooks.pre_uninstall`
- `hooks.post_uninstall`

Each of these sections can specify a `script` (a path relative to the package root) and/or a `command` (a string to be executed with `sh -c`), which will be run at the appropriate point in the install/uninstall process.

## Ignoring files

`.gitignore` and `.ignore` files at the package root or higher will be respected, including any configured global `.gitignore` file, but it might help to include Git-specific file patterns in `~/.ignore`, as they aren't usually present in a `.gitignore` file.
