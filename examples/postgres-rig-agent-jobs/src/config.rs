//! Runtime configuration boundary for the companion binaries.
//!
//! Environment variables are raw process input. This module converts them into
//! typed runtime configuration before any API server, worker, or provider runner
//! starts.

use std::env::VarError;
use std::fmt;

use thiserror::Error;

use crate::domain::{DatabaseUrl, DomainError};

#[cfg(feature = "api-server")]
use crate::api::HttpBindAddress;

// ANCHOR: runtime_config
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnvVarName(&'static str);

impl EnvVarName {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for EnvVarName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

pub const DATABASE_URL_ENV: EnvVarName = EnvVarName::new("DATABASE_URL");
#[cfg(feature = "api-server")]
pub const BIND_ADDRESS_ENV: EnvVarName = EnvVarName::new("BIND_ADDRESS");
#[cfg(feature = "rig-agent")]
pub const DEEPSEEK_API_KEY_ENV: EnvVarName = EnvVarName::new("DEEPSEEK_API_KEY");

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RuntimeConfigError {
    #[error("required environment variable {name} is missing")]
    MissingEnv { name: EnvVarName },
    #[error("environment variable {name} cannot be empty")]
    EmptyEnv { name: EnvVarName },
    #[error("environment variable {name} is not valid Unicode")]
    InvalidUnicode { name: EnvVarName },
    #[error("invalid database URL in {name}: {source}")]
    InvalidDatabaseUrl {
        name: EnvVarName,
        source: DomainError,
    },
    #[cfg(feature = "api-server")]
    #[error("invalid HTTP bind address in {name}: {value}")]
    InvalidBindAddress { name: EnvVarName, value: String },
}

pub trait RuntimeEnv {
    fn get(&self, name: EnvVarName) -> Result<Option<String>, RuntimeConfigError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessEnv;

impl RuntimeEnv for ProcessEnv {
    fn get(&self, name: EnvVarName) -> Result<Option<String>, RuntimeConfigError> {
        match std::env::var(name.as_str()) {
            Ok(value) => Ok(Some(value)),
            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(_)) => Err(RuntimeConfigError::InvalidUnicode { name }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresWorkerConfig {
    database_url: DatabaseUrl,
}

impl PostgresWorkerConfig {
    pub fn from_env() -> Result<Self, RuntimeConfigError> {
        Self::from_source(&ProcessEnv)
    }

    pub fn from_source(source: &impl RuntimeEnv) -> Result<Self, RuntimeConfigError> {
        Ok(Self {
            database_url: parse_database_url(source)?,
        })
    }

    pub fn database_url(&self) -> &DatabaseUrl {
        &self.database_url
    }
}

#[cfg(feature = "api-server")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresApiServerConfig {
    database_url: DatabaseUrl,
    bind_address: HttpBindAddress,
}

#[cfg(feature = "api-server")]
impl PostgresApiServerConfig {
    pub fn from_env() -> Result<Self, RuntimeConfigError> {
        Self::from_source(&ProcessEnv)
    }

    pub fn from_source(source: &impl RuntimeEnv) -> Result<Self, RuntimeConfigError> {
        let bind_address =
            optional_non_empty_or_default(source, BIND_ADDRESS_ENV, "127.0.0.1:3000")?;
        let bind_address = HttpBindAddress::parse(&bind_address).map_err(|_| {
            RuntimeConfigError::InvalidBindAddress {
                name: BIND_ADDRESS_ENV,
                value: bind_address,
            }
        })?;

        Ok(Self {
            database_url: parse_database_url(source)?,
            bind_address,
        })
    }

    pub fn database_url(&self) -> &DatabaseUrl {
        &self.database_url
    }

    pub fn bind_address(&self) -> HttpBindAddress {
        self.bind_address
    }
}

#[cfg(feature = "rig-agent")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeepSeekApiKeyPresent;

#[cfg(feature = "rig-agent")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeepSeekRuntimeConfig {
    _api_key: DeepSeekApiKeyPresent,
}

#[cfg(feature = "rig-agent")]
impl DeepSeekRuntimeConfig {
    pub fn from_env() -> Result<Self, RuntimeConfigError> {
        Self::from_source(&ProcessEnv)
    }

    pub fn from_source(source: &impl RuntimeEnv) -> Result<Self, RuntimeConfigError> {
        required_non_empty(source, DEEPSEEK_API_KEY_ENV)?;
        Ok(Self {
            _api_key: DeepSeekApiKeyPresent,
        })
    }
}

fn parse_database_url(source: &impl RuntimeEnv) -> Result<DatabaseUrl, RuntimeConfigError> {
    let value = required_non_empty(source, DATABASE_URL_ENV)?;
    DatabaseUrl::new(value).map_err(|source| RuntimeConfigError::InvalidDatabaseUrl {
        name: DATABASE_URL_ENV,
        source,
    })
}
// ANCHOR_END: runtime_config

fn required_non_empty(
    source: &impl RuntimeEnv,
    name: EnvVarName,
) -> Result<String, RuntimeConfigError> {
    match source.get(name)? {
        Some(value) if value.trim().is_empty() => Err(RuntimeConfigError::EmptyEnv { name }),
        Some(value) => Ok(value),
        None => Err(RuntimeConfigError::MissingEnv { name }),
    }
}

#[cfg(feature = "api-server")]
fn optional_non_empty_or_default(
    source: &impl RuntimeEnv,
    name: EnvVarName,
    default: &'static str,
) -> Result<String, RuntimeConfigError> {
    match source.get(name)? {
        Some(value) if value.trim().is_empty() => Err(RuntimeConfigError::EmptyEnv { name }),
        Some(value) => Ok(value),
        None => Ok(default.to_string()),
    }
}

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
    fn worker_config_requires_database_url() {
        let env = StaticEnv(&[]);

        let error = PostgresWorkerConfig::from_source(&env).expect_err("missing database URL");

        assert_eq!(
            error,
            RuntimeConfigError::MissingEnv {
                name: DATABASE_URL_ENV,
            }
        );
    }

    #[test]
    fn worker_config_rejects_empty_database_url() {
        let env = StaticEnv(&[("DATABASE_URL", "   ")]);

        let error = PostgresWorkerConfig::from_source(&env).expect_err("empty database URL");

        assert_eq!(
            error,
            RuntimeConfigError::EmptyEnv {
                name: DATABASE_URL_ENV,
            }
        );
    }

    #[test]
    fn worker_config_accepts_database_url() {
        let env = StaticEnv(&[("DATABASE_URL", "postgres://localhost/reliable_agents")]);

        let config = PostgresWorkerConfig::from_source(&env).expect("valid database URL");

        assert_eq!(
            config.database_url().as_str(),
            "postgres://localhost/reliable_agents"
        );
    }

    #[cfg(feature = "api-server")]
    #[test]
    fn api_config_uses_default_bind_address() {
        let env = StaticEnv(&[("DATABASE_URL", "postgres://localhost/reliable_agents")]);

        let config = PostgresApiServerConfig::from_source(&env).expect("valid API config");

        assert_eq!(
            config.bind_address().socket_addr().to_string(),
            "127.0.0.1:3000"
        );
    }

    #[cfg(feature = "api-server")]
    #[test]
    fn api_config_rejects_invalid_bind_address() {
        let env = StaticEnv(&[
            ("DATABASE_URL", "postgres://localhost/reliable_agents"),
            ("BIND_ADDRESS", "not a socket address"),
        ]);

        let error = PostgresApiServerConfig::from_source(&env).expect_err("invalid bind address");

        assert_eq!(
            error,
            RuntimeConfigError::InvalidBindAddress {
                name: BIND_ADDRESS_ENV,
                value: "not a socket address".to_string(),
            }
        );
    }

    #[cfg(feature = "rig-agent")]
    #[test]
    fn deepseek_config_requires_non_empty_api_key() {
        let missing = StaticEnv(&[]);
        let empty = StaticEnv(&[("DEEPSEEK_API_KEY", " ")]);
        let present = StaticEnv(&[("DEEPSEEK_API_KEY", "sk-test")]);

        assert_eq!(
            DeepSeekRuntimeConfig::from_source(&missing).expect_err("missing key"),
            RuntimeConfigError::MissingEnv {
                name: DEEPSEEK_API_KEY_ENV,
            }
        );
        assert_eq!(
            DeepSeekRuntimeConfig::from_source(&empty).expect_err("empty key"),
            RuntimeConfigError::EmptyEnv {
                name: DEEPSEEK_API_KEY_ENV,
            }
        );
        DeepSeekRuntimeConfig::from_source(&present).expect("present key");
    }
}
