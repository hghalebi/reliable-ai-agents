use chrono::Utc;
use postgres_rig_agent_jobs::{
    AgentInstruction, AgentJobBuilder, AgentJobStore, DeepSeekRigAgentRunner,
    DeepSeekRuntimeConfig, DomainError, IdempotencyKey, InMemoryAgentJobStore, InMemoryStoreError,
    JobKind, MaxAttempts, RuntimeConfigError, RuntimeTracingConfig, RuntimeTracingError, Worker,
    WorkerError, WorkerId, init_runtime_tracing,
};

#[tokio::main]
async fn main() -> Result<(), DeepSeekDemoError> {
    init_runtime_tracing(RuntimeTracingConfig::from_env()?)?;
    let _config = DeepSeekRuntimeConfig::from_env()?;
    let instruction = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let instruction = if instruction.trim().is_empty() {
        "Analyze why an agent job worker might be retrying repeatedly and propose one safe next step"
            .to_string()
    } else {
        instruction
    };

    let now = Utc::now();
    let mut store = InMemoryAgentJobStore::new();
    let job = AgentJobBuilder::new(MaxAttempts::default())
        .with_idempotency_key(IdempotencyKey::new("deepseek-demo:incident-triage")?)
        .kind(JobKind::new("deepseek_incident_triage")?)
        .instruction(AgentInstruction::new(instruction)?)
        .build(now);
    let job_id = store.enqueue(job, now).await?;

    let worker = Worker::new(
        WorkerId::new("deepseek-demo-worker")?,
        DeepSeekRigAgentRunner,
    );
    let outcome = worker.run_once(&mut store, now).await?;

    println!("worker outcome: {outcome:?}");
    if let Some(job) = store.job(job_id) {
        println!("job status: {:?}", job.status);
        println!("result: {}", serde_json::to_string_pretty(&job.result)?);
    }

    println!("events:");
    for event in store.events_for(job_id) {
        println!("  {:?}", event.event_type);
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum DeepSeekDemoError {
    #[error("runtime tracing setup failed: {0}")]
    Tracing(#[from] RuntimeTracingError),
    #[error("runtime configuration failed: {0}")]
    Config(#[from] RuntimeConfigError),
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
    #[error("in-memory store failed: {0}")]
    Store(#[from] InMemoryStoreError),
    #[error("worker failed: {0}")]
    Worker(#[from] WorkerError),
    #[error("failed to serialize result JSON: {0}")]
    Json(#[from] serde_json::Error),
}
