//! Pedagogical Production Observability.
//!
//! This module implements a robust, multi-layered telemetry system using `tracing`.
//! It separates human-readable logs (stdout) from machine-readable/audit-ready logs (files).
//!
//! For a comprehensive map of academic and industry references used in this design,
//! see the `REFERENCES.md` file in the project root.

use thiserror::Error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, time::UtcTime},
    prelude::*,
    EnvFilter,
};

#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("failed to initialize tracing: {0}")]
    InitFailed(String),
}

/// Initializes the telemetry system.
///
/// Returns a `WorkerGuard` which MUST be kept alive in `main` to ensure
/// that non-blocking file logs are flushed to disk before the program exits.
///
/// # Layers
/// 1. **Stdout:** Pretty, colored, and human-readable.
/// 2. **File:** Daily rolling structured logs in `logs/app.log`.
pub fn init_telemetry() -> Result<WorkerGuard, ObservabilityError> {
    // 1. Configure the Filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,postgres_rig_agent_jobs=trace"));

    // 2. Configure File Appender (Daily Rolling)
    let file_appender = tracing_appender::rolling::daily("logs", "app.log");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);

    // 3. Configure Time Formatting (ISO 8601)
    let timer = UtcTime::rfc_3339();

    // 4. Initialize Registry with Multiple Layers
    tracing_subscriber::registry()
        .with(env_filter)
        // Layer 1: Stdout (Pretty)
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_timer(timer.clone())
                .pretty(),
        )
        // Layer 2: File (Structured JSON for auditing)
        .with(
            fmt::Layer::new()
                .with_writer(non_blocking_appender)
                .with_timer(timer)
                .json() // True structured logging for machine consumption
                .with_thread_ids(true) // Identify the thread for concurrency debugging
                .with_line_number(true) // Pinpoint the exact line of code
                .with_file(true) // Include the file name for context
                .with_ansi(false), // Files shouldn't have ANSI escape codes
        )
        .try_init()
        .map_err(|e| ObservabilityError::InitFailed(e.to_string()))?;

    Ok(guard)
}

/*
🎓 The Professor's Corner

1. Syntax Spotlight: `impl Layer<S>`
   Rust uses "Static Dispatch" here. When we add layers with `.with()`, the compiler
   builds a specific type at compile time that handles all the logic, ensuring
   zero-runtime overhead for the abstraction.

2. Design Patterns: RAII (Resource Acquisition Is Initialization)
   The `WorkerGuard` is a classic example of RAII. When the guard is dropped at the
   end of `main`, its `Drop` implementation ensures that any remaining logs in the
   non-blocking buffer are flushed to disk. Without this guard, logs would be lost
   if the program exits abruptly.

3. Design Patterns: Builder Pattern
   The `tracing-subscriber` registry uses a builder-style interface. Methods like
   `.with()` and `.json()` allow us to incrementally configure the logging
   pipeline. This is a common Rust idiom for managing complex configurations
   without a massive number of constructor arguments.

4. Rust Concepts: Non-blocking I/O
   Logging to a file can be slow (Disk I/O). By using `tracing_appender::non_blocking`,
   we move the actual "writing to disk" to a separate background thread. This ensures
   that our main application logic never has to "wait" for a log line to be written,
   preserving high performance.
*/
