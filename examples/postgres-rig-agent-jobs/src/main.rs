use chrono::Utc;
use postgres_rig_agent_jobs::{
    AgentInstruction, AgentJobBuilder, AgentJobStore, DeterministicAgentRunner, IdempotencyKey,
    InMemoryAgentJobStore, InMemoryStoreError, JobKind, MaxAttempts, RuntimeTracingConfig,
    RuntimeTracingError, Worker, WorkerError, WorkerId, init_runtime_tracing,
};
use thiserror::Error;
use tracing::{info, trace};

#[derive(Debug, Error)]
enum LocalDemoError {
    #[error("failed to initialize telemetry: {0}")]
    Tracing(#[from] RuntimeTracingError),

    #[error("invalid domain value: {0}")]
    Domain(#[from] postgres_rig_agent_jobs::DomainError),

    #[error("failed to enqueue job: {0}")]
    Enqueue(#[from] InMemoryStoreError),

    #[error("worker failed: {0}")]
    Worker(#[from] WorkerError),

    #[error("failed to serialize job result: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[tokio::main]
async fn main() -> Result<(), LocalDemoError> {
    init_runtime_tracing(RuntimeTracingConfig::from_env()?)?;

    info!(
        version = env!("CARGO_PKG_VERSION"),
        timestamp = %Utc::now(),
        "observable companion app is starting"
    );

    let now = Utc::now();
    let mut store = InMemoryAgentJobStore::new();

    trace!("building incident-triage job");
    let job = AgentJobBuilder::new(MaxAttempts::default())
        .with_idempotency_key(IdempotencyKey::new("local-demo:failed-deployment")?)
        .kind(JobKind::new("incident_triage")?)
        .instruction(AgentInstruction::new("Analyze failed deployment logs")?)
        .build(now);

    info!(job_id = %job.id.as_uuid(), "Enqueuing job into in-memory store");
    let job_id = store.enqueue(job, now).await?;

    let worker_id = WorkerId::new("local-demo-worker")?;
    let worker = Worker::new(worker_id.clone(), DeterministicAgentRunner);

    info!(worker_id = %worker_id.as_str(), "Starting worker execution...");
    let outcome = worker.run_once(&mut store, now).await?;

    info!(?outcome, "Worker execution completed");
    if let Some(job) = store.job(job_id) {
        info!(status = ?job.status, "Job processed successfully");
        trace!(result = %serde_json::to_string(&job.result)?, "Detailed job result");
    }

    info!("demo run finished");

    Ok(())
}
