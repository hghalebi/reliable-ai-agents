//! Runtime tracing setup for the companion binaries.
//!
//! `tracing` calls are only useful in a running process when a subscriber is
//! installed. This module keeps that startup boundary explicit: raw environment
//! variables become typed logging configuration before the API, worker, or
//! provider demo begins runtime work.

use thiserror::Error;
use tracing_subscriber::{EnvFilter, filter::ParseError};

use crate::config::{EnvVarName, ProcessEnv, RuntimeConfigError, RuntimeEnv};

// ANCHOR: runtime_tracing_config
/// Environment variable that controls runtime tracing filters.
pub const RUST_LOG_ENV: EnvVarName = EnvVarName::new("RUST_LOG");
/// Environment variable that selects compact or JSON log output.
pub const LOG_FORMAT_ENV: EnvVarName = EnvVarName::new("LOG_FORMAT");

const DEFAULT_LOG_FILTER: &str = "info,postgres_rig_agent_jobs=info";

#[derive(Debug, Error)]
/// Failure raised while turning raw runtime logging settings into an installed subscriber.
pub enum RuntimeTracingError {
    #[error("runtime configuration failed: {0}")]
    Config(#[from] RuntimeConfigError),
    #[error("environment variable {name} cannot be empty")]
    EmptyEnv { name: EnvVarName },
    #[error("unknown log format in {name}: {value}")]
    UnknownLogFormat {
        name: EnvVarName,
        value: UnknownLogFormat,
    },
    #[error("invalid tracing filter in {name}: {source}")]
    InvalidLogFilter {
        name: EnvVarName,
        source: ParseError,
    },
    #[error("failed to install tracing subscriber: {source}")]
    InstallSubscriber { source: TracingSubscriberInitError },
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Unknown value supplied through `LOG_FORMAT`.
pub struct UnknownLogFormat(String);

impl UnknownLogFormat {
    /// Creates a typed unknown-format value for error reporting.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownLogFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error payload captured when the global tracing subscriber cannot be installed.
pub struct TracingSubscriberInitError(String);

impl TracingSubscriberInitError {
    /// Creates a typed subscriber-installation error payload.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for TracingSubscriberInitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TracingSubscriberInitError {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Runtime log format supported by the companion binaries.
pub enum RuntimeLogFormat {
    /// Human-readable local log output.
    #[default]
    Compact,
    /// Machine-readable JSON output for production log pipelines.
    Json,
}

impl RuntimeLogFormat {
    /// Returns the environment-facing name of this log format.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Json => "json",
        }
    }

    fn parse(value: &str) -> Result<Self, RuntimeTracingError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "compact" => Ok(Self::Compact),
            "json" => Ok(Self::Json),
            value => Err(RuntimeTracingError::UnknownLogFormat {
                name: LOG_FORMAT_ENV,
                value: UnknownLogFormat::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Validated tracing filter directive parsed from `RUST_LOG`.
pub struct LogFilterDirective(String);

impl LogFilterDirective {
    /// Creates a non-empty tracing filter directive.
    pub fn new(value: impl Into<String>) -> Result<Self, RuntimeTracingError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(RuntimeTracingError::EmptyEnv { name: RUST_LOG_ENV });
        }
        Ok(Self(value))
    }

    /// Returns the underlying tracing filter syntax.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Typed runtime tracing configuration for executable startup.
pub struct RuntimeTracingConfig {
    filter: LogFilterDirective,
    format: RuntimeLogFormat,
}

impl RuntimeTracingConfig {
    /// Reads tracing configuration from process environment variables.
    pub fn from_env() -> Result<Self, RuntimeTracingError> {
        Self::from_source(&ProcessEnv)
    }

    /// Reads tracing configuration from an explicit environment source.
    pub fn from_source(source: &impl RuntimeEnv) -> Result<Self, RuntimeTracingError> {
        let filter = match source.get(RUST_LOG_ENV)? {
            Some(value) => LogFilterDirective::new(value)?,
            None => LogFilterDirective::new(DEFAULT_LOG_FILTER)?,
        };

        let format = match source.get(LOG_FORMAT_ENV)? {
            Some(value) if value.trim().is_empty() => {
                return Err(RuntimeTracingError::EmptyEnv {
                    name: LOG_FORMAT_ENV,
                });
            }
            Some(value) => RuntimeLogFormat::parse(&value)?,
            None => RuntimeLogFormat::default(),
        };

        Ok(Self { filter, format })
    }

    /// Returns the validated tracing filter directive.
    pub fn filter(&self) -> &LogFilterDirective {
        &self.filter
    }

    /// Returns the selected runtime log format.
    pub fn format(&self) -> RuntimeLogFormat {
        self.format
    }

    /// Converts the typed filter directive into a `tracing_subscriber` filter.
    pub fn to_env_filter(&self) -> Result<EnvFilter, RuntimeTracingError> {
        EnvFilter::try_new(self.filter.as_str()).map_err(|source| {
            RuntimeTracingError::InvalidLogFilter {
                name: RUST_LOG_ENV,
                source,
            }
        })
    }
}

/// Installs the process-wide tracing subscriber before runtime work begins.
pub fn init_runtime_tracing(config: RuntimeTracingConfig) -> Result<(), RuntimeTracingError> {
    let env_filter = config.to_env_filter()?;

    match config.format {
        RuntimeLogFormat::Compact => tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .compact()
            .try_init()
            .map_err(|source| RuntimeTracingError::InstallSubscriber {
                source: TracingSubscriberInitError::new(source.to_string()),
            })?,
        RuntimeLogFormat::Json => tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .try_init()
            .map_err(|source| RuntimeTracingError::InstallSubscriber {
                source: TracingSubscriberInitError::new(source.to_string()),
            })?,
    }

    Ok(())
}
// ANCHOR_END: runtime_tracing_config

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct StaticEnv(&'static [(&'static str, &'static str)]);

    impl RuntimeEnv for StaticEnv {
        fn get(&self, name: EnvVarName) -> Result<Option<String>, RuntimeConfigError> {
            Ok(self
                .0
                .iter()
                .find(|(key, _)| *key == name.as_str())
                .map(|(_, value)| (*value).to_string()))
        }
    }

    #[test]
    fn tracing_config_uses_production_safe_defaults() {
        let config = RuntimeTracingConfig::from_source(&StaticEnv(&[])).expect("default config");

        assert_eq!(config.filter().as_str(), DEFAULT_LOG_FILTER);
        assert_eq!(config.format(), RuntimeLogFormat::Compact);
    }

    #[test]
    fn tracing_config_accepts_json_format() {
        let config = RuntimeTracingConfig::from_source(&StaticEnv(&[("LOG_FORMAT", "json")]))
            .expect("json config");

        assert_eq!(config.format().as_str(), "json");
    }

    #[test]
    fn tracing_config_rejects_unknown_log_format() {
        let error = RuntimeTracingConfig::from_source(&StaticEnv(&[("LOG_FORMAT", "pretty")]))
            .expect_err("unknown log format rejected");

        assert!(matches!(
            error,
            RuntimeTracingError::UnknownLogFormat { .. }
        ));
    }

    #[test]
    fn tracing_config_rejects_empty_log_filter() {
        let error = RuntimeTracingConfig::from_source(&StaticEnv(&[("RUST_LOG", "  ")]))
            .expect_err("empty log filter rejected");

        assert!(matches!(
            error,
            RuntimeTracingError::EmptyEnv { name } if name == RUST_LOG_ENV
        ));
    }

    #[test]
    fn tracing_config_rejects_invalid_log_filter_syntax() {
        let config = RuntimeTracingConfig::from_source(&StaticEnv(&[("RUST_LOG", "[bad")]))
            .expect("raw filter is loaded before subscriber parsing");

        let result = config.to_env_filter();

        assert!(matches!(
            result,
            Err(RuntimeTracingError::InvalidLogFilter { .. })
        ));
    }
}
