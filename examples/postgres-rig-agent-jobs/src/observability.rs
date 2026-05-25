//! Production observability setup for the companion example.
//!
//! The book keeps the runtime dependency surface small, so this module uses
//! `tracing-subscriber` directly and writes structured logs to stdout. A real
//! deployment can route stdout to an OpenTelemetry collector, log pipeline, or
//! platform-native logging backend without adding another crate to the core
//! teaching example.

use thiserror::Error;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::UtcTime},
    prelude::*,
};

#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("failed to initialize tracing: {0}")]
    InitFailed(String),
}

/// Initializes structured telemetry for the example binary.
pub fn init_telemetry() -> Result<(), ObservabilityError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,postgres_rig_agent_jobs=trace"));

    let timer = UtcTime::rfc_3339();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_timer(timer)
                .json()
                .with_thread_ids(true)
                .with_line_number(true)
                .with_file(true)
                .with_ansi(false),
        )
        .try_init()
        .map_err(|e| ObservabilityError::InitFailed(e.to_string()))?;

    Ok(())
}
