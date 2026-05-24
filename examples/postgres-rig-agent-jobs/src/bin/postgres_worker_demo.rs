use chrono::Utc;
use postgres_rig_agent_jobs::{
    AgentInstruction, AgentJobBuilder, AgentJobStore, DeterministicAgentRunner, DomainError,
    IdempotencyKey, JobKind, MaxAttempts, PostgresAgentJobStore, PostgresStoreError,
    PostgresWorkerConfig, RuntimeConfigError, RuntimeTracingConfig, RuntimeTracingError, Worker,
    WorkerControl, WorkerCycleLimit, WorkerError, WorkerId, init_runtime_tracing,
};
use std::num::NonZeroU32;

#[tokio::main]
async fn main() -> Result<(), PostgresWorkerDemoError> {
    init_runtime_tracing(RuntimeTracingConfig::from_env()?)?;
    let config = PostgresWorkerConfig::from_env()?;
    let now = Utc::now();
    let mut store = PostgresAgentJobStore::connect(config.database_url().clone()).await?;

    let job = AgentJobBuilder::new(MaxAttempts::default())
        .with_idempotency_key(IdempotencyKey::new("postgres-demo:failed-deployment")?)
        .kind(JobKind::new("incident_triage")?)
        .instruction(AgentInstruction::new(
            "Analyze failed deployment logs from the Postgres-backed worker",
        )?)
        .build(now);
    let job_id = store.enqueue(job, now).await?;

    let worker = Worker::new(
        WorkerId::new("postgres-demo-worker")?,
        DeterministicAgentRunner,
    );
    let loop_outcome = worker
        .run_bounded_loop(
            &mut store,
            now,
            WorkerControl::accepting_work(),
            WorkerCycleLimit::new(NonZeroU32::MIN),
        )
        .await?;
    let metrics = store.metrics().await?;
    let events = store.events_for(job_id).await?;

    println!("worker loop outcome: {loop_outcome:?}");
    println!("job id: {:?}", job_id.as_uuid());
    println!("metrics: {metrics:?}");
    println!("events:");
    for event in events {
        println!("  {:?}", event.event_type);
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum PostgresWorkerDemoError {
    #[error("runtime tracing setup failed: {0}")]
    Tracing(#[from] RuntimeTracingError),
    #[error("runtime configuration failed: {0}")]
    Config(#[from] RuntimeConfigError),
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
    #[error("postgres store failed: {0}")]
    Store(#[from] PostgresStoreError),
    #[error("worker failed: {0}")]
    Worker(#[from] WorkerError),
}
