use anyhow::Context;
use chrono::Utc;
use postgres_rig_agent_jobs::{
    init_telemetry, AgentInstruction, AgentJobBuilder, AgentJobStore, DeterministicAgentRunner,
    IdempotencyKey, InMemoryAgentJobStore, JobKind, MaxAttempts, Worker, WorkerId,
};
use tracing::{info, trace};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Telemetry and Bind Guard
    let _guard = init_telemetry().context("failed to initialize telemetry")?;

    // 2. Startup Log
    info!(
        version = env!("CARGO_PKG_VERSION"),
        timestamp = %Utc::now(),
        "The Observable App is starting up..."
    );

    let now = Utc::now();
    let mut store = InMemoryAgentJobStore::new();

    // 3. Simulate Domain Action with Trace Logging
    trace!("Building agent job for incident triage...");
    let job = AgentJobBuilder::new(MaxAttempts::default())
        .with_idempotency_key(IdempotencyKey::new("local-demo:failed-deployment")?)
        .kind(JobKind::new("incident_triage")?)
        .instruction(AgentInstruction::new("Analyze failed deployment logs")?)
        .build(now);

    info!(job_id = %job.id.as_uuid(), "Enqueuing job into in-memory store");
    let job_id = store.enqueue(job, now).await.context("failed to enqueue job")?;

    let worker_id = WorkerId::new("local-demo-worker")?;
    let worker = Worker::new(
        worker_id.clone(),
        DeterministicAgentRunner,
    );
    
    info!(worker_id = %worker_id.as_str(), "Starting worker execution...");
    let outcome = worker.run_once(&mut store, now).await.context("worker failed")?;

    info!(?outcome, "Worker execution completed");
    if let Some(job) = store.job(job_id) {
        info!(status = ?job.status, "Job processed successfully");
        trace!(result = %serde_json::to_string(&job.result)?, "Detailed job result");
    }

    info!("Demo run finished. All events flushed via RAII guard.");

    Ok(())
}

/*
🎓 The Professor's Corner

1. Rust Concepts: Error Propagation
   By using `anyhow::Result<()>`, we can use the `?` operator to propagate errors up
   to the runtime. The `context()` method allows us to attach high-level meaning
   to low-level errors (e.g., "failed to initialize telemetry").

2. Rust Concepts: Scope and Drop
   The `_guard` variable is crucial. It lives until the very end of `main()`.
   When `main` returns, `_guard` goes out of scope and is "dropped."
   Because it implements the `Drop` trait, it automatically triggers a flush
   of the non-blocking log appender. This is "Pedagogical Production Code"
   in action—safety through structure.
*/
