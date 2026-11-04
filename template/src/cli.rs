//! Command-line interface definition.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use toml::Value;

use crate::config::{CliValue, LogLevel};

/// Command-line arguments.
#[derive(Debug, Parser)]
#[command(name = env!("CARGO_PKG_NAME"), version, about)]
pub struct Cli {
    /// Config file to use instead of the automatic lookup.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Log level (overrides `log.level`).
    #[arg(long, global = true, value_enum)]
    pub log_level: Option<LogLevel>,

    /// Command to run; defaults to `greet`.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Top-level commands.
#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Command {
    /// Print a greeting (the example command; replace with your own).
    Greet {
        /// Name to greet; repeat for several (overrides `greeting.names`).
        #[arg(long = "name", value_name = "NAME")]
        names: Vec<String>,
    },
    /// Inspect configuration.
    Config {
        /// What to show.
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

/// `config` subcommands.
#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum ConfigCommand {
    /// Show the effective configuration and where each value came from.
    Show,
    /// List the config file search order and which file was selected.
    Paths,
}

impl Cli {
    /// Config values supplied by flags, for the highest-precedence layer.
    #[must_use]
    pub fn cli_values(&self) -> Vec<CliValue> {
        let mut values = Vec::new();
        if let Some(level) = self.log_level {
            values.push(CliValue {
                key: "log.level".to_owned(),
                flag: "--log-level".to_owned(),
                value: Value::String(level.as_str().to_owned()),
            });
        }
        if let Some(Command::Greet { names }) = &self.command
            && !names.is_empty()
        {
            values.push(CliValue {
                key: "greeting.names".to_owned(),
                flag: "--name".to_owned(),
                value: Value::Array(names.iter().cloned().map(Value::String).collect()),
            });
        }
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory as _;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn no_flags_means_no_cli_values() {
        let cli = Cli::try_parse_from(["app"]).unwrap();
        assert_eq!(cli.cli_values(), Vec::new());
        assert_eq!(cli.command, None);
    }

    #[test]
    fn flags_become_cli_values() {
        let cli = Cli::try_parse_from([
            "app",
            "--log-level",
            "debug",
            "greet",
            "--name",
            "Ada",
            "--name",
            "Linus",
        ])
        .unwrap();
        let values = cli.cli_values();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].key, "log.level");
        assert_eq!(values[0].value, Value::String("debug".into()));
        assert_eq!(values[1].key, "greeting.names");
        assert_eq!(values[1].flag, "--name");
        assert_eq!(
            values[1].value,
            Value::Array(vec![
                Value::String("Ada".into()),
                Value::String("Linus".into())
            ])
        );
    }

    #[test]
    fn global_flags_work_after_subcommands() {
        let cli = Cli::try_parse_from(["app", "config", "show", "--config", "/x.toml"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/x.toml")));
        assert_eq!(
            cli.command,
            Some(Command::Config {
                command: ConfigCommand::Show
            })
        );
    }

    #[test]
    fn greet_without_names_adds_no_value() {
        let cli = Cli::try_parse_from(["app", "greet"]).unwrap();
        assert_eq!(cli.cli_values(), Vec::new());
    }
}
