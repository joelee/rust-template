//! Non-secret configuration: the schema, its defaults, and resolution.
//!
//! Effective configuration is layered, highest precedence first: CLI flags,
//! per-key environment overrides, the selected config file, and built-in
//! defaults. See `docs/configuration.md` for the full rules.

mod resolve;
mod show;

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use resolve::{CliValue, Resolved, candidate_paths, resolve, select_file};
pub use show::{render_paths, render_show};

/// The complete non-secret configuration.
///
/// Every field must have a default so the application runs without a config
/// file, and so every key has a known type for environment overrides. Avoid
/// `Option` fields: a `None` default has no type to parse overrides against.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// Logging settings (`[log]`).
    pub log: LogConfig,
    /// Example application settings (`[greeting]`).
    pub greeting: GreetingConfig,
}

/// Logging settings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct LogConfig {
    /// Minimum level written to stderr.
    pub level: LogLevel,
}

/// Log verbosity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Errors only.
    Error,
    /// Warnings and errors.
    Warn,
    /// Informational messages and above.
    #[default]
    Info,
    /// Debug messages and above.
    Debug,
    /// Everything.
    Trace,
}

impl LogLevel {
    /// The equivalent `tracing` filter.
    #[must_use]
    pub fn as_filter(self) -> tracing::level_filters::LevelFilter {
        use tracing::level_filters::LevelFilter;
        match self {
            Self::Error => LevelFilter::ERROR,
            Self::Warn => LevelFilter::WARN,
            Self::Info => LevelFilter::INFO,
            Self::Debug => LevelFilter::DEBUG,
            Self::Trace => LevelFilter::TRACE,
        }
    }

    /// The lowercase name used in config files and on the command line.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

/// Example application settings. Replace with your own sections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct GreetingConfig {
    /// Greeting word, e.g. `Hello`.
    pub message: String,
    /// How many times to print each greeting.
    pub repeat: u32,
    /// Names to greet.
    pub names: Vec<String>,
}

impl Default for GreetingConfig {
    fn default() -> Self {
        Self {
            message: "Hello".to_owned(),
            repeat: 1,
            names: vec!["world".to_owned()],
        }
    }
}

/// Where a configuration value came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// The built-in default.
    Default,
    /// The selected config file.
    File(PathBuf),
    /// A per-key environment override, named by its variable.
    Env(String),
    /// A command-line flag.
    Cli(String),
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => f.write_str("default"),
            Self::File(path) => write!(f, "file {}", path.display()),
            Self::Env(var) => write!(f, "env {var}"),
            Self::Cli(flag) => write!(f, "cli {flag}"),
        }
    }
}

/// Errors raised while resolving configuration. Each names the source at
/// fault so the user can fix it.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A config file named explicitly does not exist.
    #[error("config file {} (from {origin}) does not exist", path.display())]
    NotFound {
        /// The missing path.
        path: PathBuf,
        /// What named it: `--config` or the `<PREFIX>_CONFIG_FILE` variable.
        origin: String,
    },
    /// The selected config file could not be read.
    #[error("cannot read config file {}: {error}", path.display())]
    Read {
        /// The file.
        path: PathBuf,
        /// The underlying I/O error.
        error: std::io::Error,
    },
    /// The selected config file is not valid TOML.
    #[error("cannot parse config file {}: {message}", path.display())]
    Parse {
        /// The file.
        path: PathBuf,
        /// The parser's message.
        message: String,
    },
    /// The merged configuration does not match the schema.
    #[error("invalid configuration{context}: {message}")]
    Invalid {
        /// Optional ` in <file>` suffix.
        context: String,
        /// The schema error.
        message: String,
    },
    /// An override variable does not correspond to a config key.
    #[error("unknown config override {var}: no such key")]
    UnknownOverride {
        /// The variable name.
        var: String,
    },
    /// An override variable's value cannot be used for its key.
    #[error("invalid value for {var}: {message}")]
    InvalidOverride {
        /// The variable name.
        var: String,
        /// Why the value was rejected.
        message: String,
    },
    /// An override for a non-string key is empty.
    #[error("{var} is empty; only string keys accept an empty value")]
    EmptyOverride {
        /// The variable name.
        var: String,
    },
    /// The built-in defaults could not be serialised (a bug in the schema).
    #[error("internal error: cannot serialise default configuration: {0}")]
    Defaults(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_deterministic() {
        let config = Config::default();
        assert_eq!(config.log.level, LogLevel::Info);
        assert_eq!(config.greeting.message, "Hello");
        assert_eq!(config.greeting.repeat, 1);
        assert_eq!(config.greeting.names, vec!["world".to_owned()]);
    }

    #[test]
    fn log_levels_map_to_filters_and_names() {
        use tracing::level_filters::LevelFilter;
        let cases = [
            (LogLevel::Error, LevelFilter::ERROR, "error"),
            (LogLevel::Warn, LevelFilter::WARN, "warn"),
            (LogLevel::Info, LevelFilter::INFO, "info"),
            (LogLevel::Debug, LevelFilter::DEBUG, "debug"),
            (LogLevel::Trace, LevelFilter::TRACE, "trace"),
        ];
        for (level, filter, name) in cases {
            assert_eq!(level.as_filter(), filter);
            assert_eq!(level.as_str(), name);
        }
    }

    #[test]
    fn sources_display_their_origin() {
        assert_eq!(Source::Default.to_string(), "default");
        assert_eq!(
            Source::File(PathBuf::from("/etc/a.toml")).to_string(),
            "file /etc/a.toml"
        );
        assert_eq!(Source::Env("A__B".into()).to_string(), "env A__B");
        assert_eq!(Source::Cli("--x".into()).to_string(), "cli --x");
    }

    #[test]
    fn errors_name_the_source_at_fault() {
        let err = ConfigError::NotFound {
            path: PathBuf::from("/missing.toml"),
            origin: "--config".into(),
        };
        assert_eq!(
            err.to_string(),
            "config file /missing.toml (from --config) does not exist"
        );
        let err = ConfigError::EmptyOverride { var: "A__B".into() };
        assert_eq!(
            err.to_string(),
            "A__B is empty; only string keys accept an empty value"
        );
        let err = ConfigError::Defaults("boom".into());
        assert!(err.to_string().contains("boom"));
    }
}
