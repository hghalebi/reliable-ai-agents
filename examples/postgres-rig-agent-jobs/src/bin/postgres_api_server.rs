use postgres_rig_agent_jobs::{
    PostgresApiServerConfig, PostgresApiServerError, RuntimeConfigError, RuntimeTracingConfig,
    RuntimeTracingError, init_runtime_tracing, serve_postgres_api,
};

#[tokio::main]
async fn main() -> Result<(), PostgresApiBinaryError> {
    init_runtime_tracing(RuntimeTracingConfig::from_env()?)?;
    let config = PostgresApiServerConfig::from_env()?;

    serve_postgres_api(config.database_url().clone(), config.bind_address()).await?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum PostgresApiBinaryError {
    #[error("runtime tracing setup failed: {0}")]
    Tracing(#[from] RuntimeTracingError),
    #[error("runtime configuration failed: {0}")]
    Config(#[from] RuntimeConfigError),
    #[error("postgres API server failed: {0}")]
    Server(#[from] PostgresApiServerError),
}
