use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::num::NonZeroU32;
use thiserror::Error;

use crate::agent::{AgentError, AgentRunner};
use crate::domain::{
    AgentJob, AgentResult, CancellationOutcome, CancellationReason, EventMessage, FailureMessage,
    HeartbeatInterval, JobEventType, JobId, LeaseDuration, LeaseExtensionOutcome, RetryDisposition,
    RetryPolicy, ShutdownReason, WorkerId,
};

#[async_trait]
pub trait AgentJobStore {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn enqueue(&mut self, job: AgentJob, now: DateTime<Utc>) -> Result<JobId, Self::Error>;

    async fn recover_expired(&mut self, now: DateTime<Utc>) -> Result<Vec<JobId>, Self::Error>;

    async fn pick_due_job(
        &mut self,
        worker_id: WorkerId,
        now: DateTime<Utc>,
        lease_duration: LeaseDuration,
    ) -> Result<Option<AgentJob>, Self::Error>;

    async fn mark_succeeded(
        &mut self,
        job_id: JobId,
        worker_id: WorkerId,
        result: AgentResult,
        now: DateTime<Utc>,
    ) -> Result<(), Self::Error>;

    async fn retry_or_dead(
        &mut self,
        job_id: JobId,
        worker_id: WorkerId,
        error: FailureMessage,
        retry_policy: RetryPolicy,
        retry_disposition: RetryDisposition,
        now: DateTime<Utc>,
    ) -> Result<(), Self::Error>;

    async fn extend_lease(
        &mut self,
        job_id: JobId,
        worker_id: WorkerId,
        now: DateTime<Utc>,
        lease_duration: LeaseDuration,
    ) -> Result<LeaseExtensionOutcome, Self::Error>;

    async fn cancel(
        &mut self,
        job_id: JobId,
        reason: CancellationReason,
        now: DateTime<Utc>,
    ) -> Result<CancellationOutcome, Self::Error>;

    async fn record_event(
        &mut self,
        job_id: JobId,
        event_type: JobEventType,
        message: Option<EventMessage>,
        now: DateTime<Utc>,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("job store failed: {0}")]
    Store(StoreFailure),
    #[error("agent failed: {0}")]
    Agent(#[from] AgentError),
    #[error("worker lost the lease for job {job_id:?} during agent execution: {outcome:?}")]
    LeaseLost {
        job_id: JobId,
        outcome: LeaseExtensionOutcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreFailure(FailureMessage);

impl StoreFailure {
    fn from_error(error: impl std::error::Error) -> Self {
        Self(FailureMessage::from_error_text(error.to_string()))
    }
}

impl std::fmt::Display for StoreFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0.as_str())
    }
}

impl std::error::Error for StoreFailure {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerRunOutcome {
    NoDueJob,
    DrainRequested,
    Succeeded(JobId),
    RetryScheduled(JobId),
    Dead(JobId),
}

impl WorkerRunOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoDueJob => "no_due_job",
            Self::DrainRequested => "drain_requested",
            Self::Succeeded(_) => "succeeded",
            Self::RetryScheduled(_) => "retry_scheduled",
            Self::Dead(_) => "dead",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerCycleLimit(NonZeroU32);

impl WorkerCycleLimit {
    pub fn new(value: NonZeroU32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl Default for WorkerCycleLimit {
    fn default() -> Self {
        Self(NonZeroU32::MIN)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletedWorkerCycles(u32);

impl CompletedWorkerCycles {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerLoopOutcome {
    Idle {
        completed_cycles: CompletedWorkerCycles,
    },
    Draining {
        completed_cycles: CompletedWorkerCycles,
    },
    CycleLimitReached {
        completed_cycles: CompletedWorkerCycles,
    },
}

impl WorkerLoopOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle { .. } => "idle",
            Self::Draining { .. } => "draining",
            Self::CycleLimitReached { .. } => "cycle_limit_reached",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerControl {
    AcceptingWork,
    Draining { reason: ShutdownReason },
}

impl WorkerControl {
    pub fn accepting_work() -> Self {
        Self::AcceptingWork
    }

    pub fn draining(reason: ShutdownReason) -> Self {
        Self::Draining { reason }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AcceptingWork => "accepting_work",
            Self::Draining { .. } => "draining",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Worker<R> {
    worker_id: WorkerId,
    runner: R,
    lease_duration: LeaseDuration,
    heartbeat_interval: HeartbeatInterval,
    retry_policy: RetryPolicy,
}

impl<R> Worker<R> {
    pub fn new(worker_id: WorkerId, runner: R) -> Self {
        Self {
            worker_id,
            runner,
            lease_duration: LeaseDuration::default(),
            heartbeat_interval: HeartbeatInterval::default(),
            retry_policy: RetryPolicy::default(),
        }
    }

    pub fn with_lease_duration(mut self, lease_duration: LeaseDuration) -> Self {
        self.lease_duration = lease_duration;
        self
    }

    pub fn with_heartbeat_interval(mut self, heartbeat_interval: HeartbeatInterval) -> Self {
        self.heartbeat_interval = heartbeat_interval;
        self
    }

    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    // ANCHOR: heartbeat
    pub async fn heartbeat<S>(
        &self,
        store: &mut S,
        job_id: JobId,
        now: DateTime<Utc>,
    ) -> Result<LeaseExtensionOutcome, WorkerError>
    where
        S: AgentJobStore,
    {
        let outcome = store
            .extend_lease(job_id, self.worker_id.clone(), now, self.lease_duration)
            .await
            .map_err(|error| WorkerError::Store(StoreFailure::from_error(error)))?;

        match outcome {
            LeaseExtensionOutcome::Extended => tracing::info!(
                worker_id = %self.worker_id.as_str(),
                job_id = %job_id.as_uuid(),
                outcome = %outcome.as_str(),
                event_type = %JobEventType::LeaseExtended.as_str(),
                "worker heartbeat extended job lease"
            ),
            LeaseExtensionOutcome::NotOwnedOrNotRunning => tracing::warn!(
                worker_id = %self.worker_id.as_str(),
                job_id = %job_id.as_uuid(),
                outcome = %outcome.as_str(),
                "worker heartbeat rejected because lease is not owned or running"
            ),
        }

        Ok(outcome)
    }
    // ANCHOR_END: heartbeat
}

impl<R> Worker<R>
where
    R: AgentRunner,
{
    // ANCHOR: run_once
    pub async fn run_once<S>(
        &self,
        store: &mut S,
        now: DateTime<Utc>,
    ) -> Result<WorkerRunOutcome, WorkerError>
    where
        S: AgentJobStore,
    {
        let Some(job) = self.claim_and_start_job(store, now).await? else {
            return Ok(WorkerRunOutcome::NoDueJob);
        };

        tracing::info!(
            worker_id = %self.worker_id.as_str(),
            job_id = %job.id.as_uuid(),
            job_kind = %job.kind.as_str(),
            "agent execution running without heartbeat supervisor"
        );

        let result = self.runner.run_agent(job.payload.clone()).await;
        self.finish_job_attempt(store, job, result, now).await
    }
    // ANCHOR_END: run_once

    // ANCHOR: run_once_with_heartbeats
    pub async fn run_once_with_heartbeats<S>(
        &self,
        store: &mut S,
        now: DateTime<Utc>,
    ) -> Result<WorkerRunOutcome, WorkerError>
    where
        S: AgentJobStore + Clone,
    {
        let Some(job) = self.claim_and_start_job(store, now).await? else {
            return Ok(WorkerRunOutcome::NoDueJob);
        };

        let mut heartbeat_store = store.clone();
        let mut heartbeat_timer = tokio::time::interval(self.heartbeat_interval.duration());
        heartbeat_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let agent_future = self.runner.run_agent(job.payload.clone());
        tokio::pin!(agent_future);

        loop {
            tokio::select! {
                biased;

                result = &mut agent_future => {
                    return self.finish_job_attempt(store, job, result, now).await;
                }
                _ = heartbeat_timer.tick() => {
                    let heartbeat_at = Utc::now();
                    let outcome = self
                        .heartbeat(&mut heartbeat_store, job.id, heartbeat_at)
                        .await?;

                    if outcome != LeaseExtensionOutcome::Extended {
                        tracing::warn!(
                            worker_id = %self.worker_id.as_str(),
                            job_id = %job.id.as_uuid(),
                            outcome = %outcome.as_str(),
                            "worker lost lease during heartbeat-supervised execution"
                        );
                        return Err(WorkerError::LeaseLost {
                            job_id: job.id,
                            outcome,
                        });
                    }
                }
            }
        }
    }
    // ANCHOR_END: run_once_with_heartbeats

    // ANCHOR: run_once_controlled
    pub async fn run_once_controlled<S>(
        &self,
        store: &mut S,
        now: DateTime<Utc>,
        control: WorkerControl,
    ) -> Result<WorkerRunOutcome, WorkerError>
    where
        S: AgentJobStore + Clone,
    {
        match control {
            WorkerControl::AcceptingWork => self.run_once_with_heartbeats(store, now).await,
            WorkerControl::Draining { reason } => {
                let outcome = WorkerRunOutcome::DrainRequested;
                tracing::warn!(
                    worker_id = %self.worker_id.as_str(),
                    shutdown_reason = %reason.as_str(),
                    control_state = %WorkerControl::draining(reason.clone()).as_str(),
                    outcome = %outcome.as_str(),
                    "worker drain requested; skipping new job claim"
                );
                Ok(outcome)
            }
        }
    }
    // ANCHOR_END: run_once_controlled

    // ANCHOR: run_bounded_loop
    pub async fn run_bounded_loop<S>(
        &self,
        store: &mut S,
        started_at: DateTime<Utc>,
        control: WorkerControl,
        cycle_limit: WorkerCycleLimit,
    ) -> Result<WorkerLoopOutcome, WorkerError>
    where
        S: AgentJobStore + Clone,
    {
        let mut completed_cycles = CompletedWorkerCycles::zero();

        for _ in 0..cycle_limit.get() {
            let outcome = self
                .run_once_controlled(store, started_at, control.clone())
                .await?;

            match outcome {
                WorkerRunOutcome::Succeeded(_)
                | WorkerRunOutcome::RetryScheduled(_)
                | WorkerRunOutcome::Dead(_) => {
                    completed_cycles = completed_cycles.increment();
                }
                WorkerRunOutcome::NoDueJob => {
                    let loop_outcome = WorkerLoopOutcome::Idle { completed_cycles };
                    tracing::info!(
                        worker_id = %self.worker_id.as_str(),
                        completed_cycles = completed_cycles.get(),
                        outcome = %loop_outcome.as_str(),
                        "worker bounded loop stopped because queue is idle"
                    );
                    return Ok(loop_outcome);
                }
                WorkerRunOutcome::DrainRequested => {
                    let loop_outcome = WorkerLoopOutcome::Draining { completed_cycles };
                    tracing::warn!(
                        worker_id = %self.worker_id.as_str(),
                        completed_cycles = completed_cycles.get(),
                        outcome = %loop_outcome.as_str(),
                        "worker bounded loop stopped because drain was requested"
                    );
                    return Ok(loop_outcome);
                }
            }
        }

        let loop_outcome = WorkerLoopOutcome::CycleLimitReached { completed_cycles };
        tracing::info!(
            worker_id = %self.worker_id.as_str(),
            completed_cycles = completed_cycles.get(),
            cycle_limit = cycle_limit.get(),
            outcome = %loop_outcome.as_str(),
            "worker bounded loop stopped after cycle limit"
        );
        Ok(loop_outcome)
    }
    // ANCHOR_END: run_bounded_loop

    async fn claim_and_start_job<S>(
        &self,
        store: &mut S,
        now: DateTime<Utc>,
    ) -> Result<Option<AgentJob>, WorkerError>
    where
        S: AgentJobStore,
    {
        tracing::info!(
            worker_id = %self.worker_id.as_str(),
            "worker scan started"
        );

        let recovered_jobs = store
            .recover_expired(now)
            .await
            .map_err(|error| WorkerError::Store(StoreFailure::from_error(error)))?;

        for recovered_job_id in &recovered_jobs {
            tracing::warn!(
                worker_id = %self.worker_id.as_str(),
                job_id = %recovered_job_id.as_uuid(),
                event_type = %JobEventType::ExpiredLeaseRecovered.as_str(),
                "worker recovered expired job lease"
            );
        }

        let Some(job) = store
            .pick_due_job(self.worker_id.clone(), now, self.lease_duration)
            .await
            .map_err(|error| WorkerError::Store(StoreFailure::from_error(error)))?
        else {
            tracing::info!(
                worker_id = %self.worker_id.as_str(),
                outcome = %WorkerRunOutcome::NoDueJob.as_str(),
                recovered_jobs = recovered_jobs.len(),
                "worker scan finished without due work"
            );
            return Ok(None);
        };

        tracing::info!(
            worker_id = %self.worker_id.as_str(),
            job_id = %job.id.as_uuid(),
            job_kind = %job.kind.as_str(),
            job_status = %job.status.as_str(),
            attempt_count = job.attempt_count.get(),
            max_attempts = job.max_attempts.get(),
            event_type = %JobEventType::JobPicked.as_str(),
            "worker picked due job"
        );

        store
            .record_event(job.id, JobEventType::AgentStarted, None, now)
            .await
            .map_err(|error| WorkerError::Store(StoreFailure::from_error(error)))?;

        tracing::info!(
            worker_id = %self.worker_id.as_str(),
            job_id = %job.id.as_uuid(),
            job_kind = %job.kind.as_str(),
            event_type = %JobEventType::AgentStarted.as_str(),
            "agent execution started"
        );

        Ok(Some(job))
    }

    async fn finish_job_attempt<S>(
        &self,
        store: &mut S,
        job: AgentJob,
        result: Result<AgentResult, AgentError>,
        now: DateTime<Utc>,
    ) -> Result<WorkerRunOutcome, WorkerError>
    where
        S: AgentJobStore,
    {
        // ANCHOR: worker_observability
        match result {
            Ok(result) => {
                store
                    .record_event(job.id, JobEventType::AgentSucceeded, None, now)
                    .await
                    .map_err(|error| WorkerError::Store(StoreFailure::from_error(error)))?;
                store
                    .mark_succeeded(job.id, self.worker_id.clone(), result, now)
                    .await
                    .map_err(|error| WorkerError::Store(StoreFailure::from_error(error)))?;
                let outcome = WorkerRunOutcome::Succeeded(job.id);
                tracing::info!(
                    worker_id = %self.worker_id.as_str(),
                    job_id = %job.id.as_uuid(),
                    job_kind = %job.kind.as_str(),
                    outcome = %outcome.as_str(),
                    event_type = %JobEventType::AgentSucceeded.as_str(),
                    "agent execution succeeded"
                );
                Ok(outcome)
            }
            Err(error) => {
                let retry_disposition = error.retry_disposition();
                let exhausted = retry_disposition == RetryDisposition::Permanent
                    || job.attempt_count.get() >= job.max_attempts.get();
                let failure_message = FailureMessage::from_error_text(error.to_string());
                store
                    .record_event(
                        job.id,
                        JobEventType::AgentFailed,
                        Some(failure_message.clone().into()),
                        now,
                    )
                    .await
                    .map_err(|error| WorkerError::Store(StoreFailure::from_error(error)))?;
                store
                    .retry_or_dead(
                        job.id,
                        self.worker_id.clone(),
                        failure_message,
                        self.retry_policy,
                        retry_disposition,
                        now,
                    )
                    .await
                    .map_err(|error| WorkerError::Store(StoreFailure::from_error(error)))?;

                if exhausted {
                    let outcome = WorkerRunOutcome::Dead(job.id);
                    tracing::warn!(
                        worker_id = %self.worker_id.as_str(),
                        job_id = %job.id.as_uuid(),
                        job_kind = %job.kind.as_str(),
                        outcome = %outcome.as_str(),
                        retry_disposition = %retry_disposition.as_str(),
                        attempt_count = job.attempt_count.get(),
                        max_attempts = job.max_attempts.get(),
                        event_type = %JobEventType::JobDead.as_str(),
                        "agent execution failed and job is dead-lettered"
                    );
                    Ok(outcome)
                } else {
                    let outcome = WorkerRunOutcome::RetryScheduled(job.id);
                    tracing::warn!(
                        worker_id = %self.worker_id.as_str(),
                        job_id = %job.id.as_uuid(),
                        job_kind = %job.kind.as_str(),
                        outcome = %outcome.as_str(),
                        retry_disposition = %retry_disposition.as_str(),
                        attempt_count = job.attempt_count.get(),
                        max_attempts = job.max_attempts.get(),
                        event_type = %JobEventType::RetryScheduled.as_str(),
                        "agent execution failed and retry was scheduled"
                    );
                    Ok(outcome)
                }
            }
        }
        // ANCHOR_END: worker_observability
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;
    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;
    use chrono::TimeZone;
    use tokio::sync::Mutex;

    use crate::agent::{
        DeterministicAgentRunner, FailingThenSuccessfulRunner, PermanentFailureRunner,
        SimulatedFailureCount,
    };
    use crate::domain::{
        AgentEvent, AgentInstruction, AgentPayload, HeartbeatInterval, JobKind, JobStatus,
        LeaseDuration, MaxAttempts, RetryDelay, ShutdownReason,
    };
    use crate::memory_store::{InMemoryAgentJobStore, InMemoryStoreError};

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 9, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    fn test_job(max_attempts: u32, now: DateTime<Utc>) -> AgentJob {
        AgentJob::new(
            JobKind::new("incident_triage").expect("valid kind"),
            AgentPayload {
                instruction: AgentInstruction::new("Analyze failed deployment")
                    .expect("valid instruction"),
            },
            MaxAttempts::new(NonZeroU32::new(max_attempts).expect("non-zero attempts")),
            now,
        )
    }

    #[derive(Debug, Clone)]
    struct DelayedAgentRunner {
        duration: Duration,
    }

    impl DelayedAgentRunner {
        fn new(duration: Duration) -> Self {
            Self { duration }
        }
    }

    #[async_trait]
    impl AgentRunner for DelayedAgentRunner {
        async fn run_agent(&self, payload: AgentPayload) -> Result<AgentResult, AgentError> {
            tokio::time::sleep(self.duration).await;
            DeterministicAgentRunner.run_agent(payload).await
        }
    }

    #[derive(Debug, Clone)]
    struct SharedInMemoryAgentJobStore {
        inner: Arc<Mutex<InMemoryAgentJobStore>>,
    }

    impl SharedInMemoryAgentJobStore {
        fn new() -> Self {
            Self {
                inner: Arc::new(Mutex::new(InMemoryAgentJobStore::new())),
            }
        }

        async fn job(&self, job_id: JobId) -> Option<AgentJob> {
            self.inner.lock().await.job(job_id).cloned()
        }

        async fn events_for(&self, job_id: JobId) -> Vec<AgentEvent> {
            self.inner.lock().await.events_for(job_id)
        }
    }

    #[async_trait]
    impl AgentJobStore for SharedInMemoryAgentJobStore {
        type Error = InMemoryStoreError;

        async fn enqueue(
            &mut self,
            job: AgentJob,
            now: DateTime<Utc>,
        ) -> Result<JobId, Self::Error> {
            self.inner.lock().await.enqueue(job, now).await
        }

        async fn recover_expired(&mut self, now: DateTime<Utc>) -> Result<Vec<JobId>, Self::Error> {
            self.inner.lock().await.recover_expired(now).await
        }

        async fn pick_due_job(
            &mut self,
            worker_id: WorkerId,
            now: DateTime<Utc>,
            lease_duration: LeaseDuration,
        ) -> Result<Option<AgentJob>, Self::Error> {
            self.inner
                .lock()
                .await
                .pick_due_job(worker_id, now, lease_duration)
                .await
        }

        async fn mark_succeeded(
            &mut self,
            job_id: JobId,
            worker_id: WorkerId,
            result: AgentResult,
            now: DateTime<Utc>,
        ) -> Result<(), Self::Error> {
            self.inner
                .lock()
                .await
                .mark_succeeded(job_id, worker_id, result, now)
                .await
        }

        async fn retry_or_dead(
            &mut self,
            job_id: JobId,
            worker_id: WorkerId,
            error: FailureMessage,
            retry_policy: RetryPolicy,
            retry_disposition: RetryDisposition,
            now: DateTime<Utc>,
        ) -> Result<(), Self::Error> {
            self.inner
                .lock()
                .await
                .retry_or_dead(
                    job_id,
                    worker_id,
                    error,
                    retry_policy,
                    retry_disposition,
                    now,
                )
                .await
        }

        async fn extend_lease(
            &mut self,
            job_id: JobId,
            worker_id: WorkerId,
            now: DateTime<Utc>,
            lease_duration: LeaseDuration,
        ) -> Result<LeaseExtensionOutcome, Self::Error> {
            self.inner
                .lock()
                .await
                .extend_lease(job_id, worker_id, now, lease_duration)
                .await
        }

        async fn cancel(
            &mut self,
            job_id: JobId,
            reason: CancellationReason,
            now: DateTime<Utc>,
        ) -> Result<CancellationOutcome, Self::Error> {
            self.inner.lock().await.cancel(job_id, reason, now).await
        }

        async fn record_event(
            &mut self,
            job_id: JobId,
            event_type: JobEventType,
            message: Option<EventMessage>,
            now: DateTime<Utc>,
        ) -> Result<(), Self::Error> {
            self.inner
                .lock()
                .await
                .record_event(job_id, event_type, message, now)
                .await
        }
    }

    #[test]
    fn worker_outcome_exposes_stable_observability_label() {
        let job_id = JobId::new();

        assert_eq!(WorkerRunOutcome::NoDueJob.as_str(), "no_due_job");
        assert_eq!(WorkerRunOutcome::DrainRequested.as_str(), "drain_requested");
        assert_eq!(WorkerRunOutcome::Succeeded(job_id).as_str(), "succeeded");
        assert_eq!(
            WorkerRunOutcome::RetryScheduled(job_id).as_str(),
            "retry_scheduled"
        );
        assert_eq!(WorkerRunOutcome::Dead(job_id).as_str(), "dead");
    }

    #[test]
    fn worker_loop_outcome_exposes_stable_observability_label() {
        let cycles = CompletedWorkerCycles::zero();

        assert_eq!(
            WorkerLoopOutcome::Idle {
                completed_cycles: cycles
            }
            .as_str(),
            "idle"
        );
        assert_eq!(
            WorkerLoopOutcome::Draining {
                completed_cycles: cycles
            }
            .as_str(),
            "draining"
        );
        assert_eq!(
            WorkerLoopOutcome::CycleLimitReached {
                completed_cycles: cycles
            }
            .as_str(),
            "cycle_limit_reached"
        );
    }

    #[tokio::test]
    async fn worker_marks_successful_job_succeeded() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            DeterministicAgentRunner,
        );

        let outcome = worker.run_once(&mut store, now).await.expect("worker run");

        assert_eq!(outcome, WorkerRunOutcome::Succeeded(job_id));
        let job = store.job(job_id).expect("stored job");
        assert_eq!(job.status, JobStatus::Succeeded);
        assert!(job.result.is_some());
    }

    #[tokio::test]
    async fn worker_schedules_retry_after_transient_failure() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            FailingThenSuccessfulRunner::new(SimulatedFailureCount::new(1)),
        )
        .with_retry_policy(RetryPolicy::new(
            RetryDelay::from_secs(10),
            RetryDelay::from_secs(60),
        ));

        let outcome = worker.run_once(&mut store, now).await.expect("worker run");

        assert_eq!(outcome, WorkerRunOutcome::RetryScheduled(job_id));
        let job = store.job(job_id).expect("stored job");
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.attempt_count.get(), 1);
        assert!(job.run_at > now);
    }

    #[tokio::test]
    async fn worker_marks_job_dead_after_attempts_are_exhausted() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(1, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            FailingThenSuccessfulRunner::new(SimulatedFailureCount::new(1)),
        );

        let outcome = worker.run_once(&mut store, now).await.expect("worker run");

        assert_eq!(outcome, WorkerRunOutcome::Dead(job_id));
        let job = store.job(job_id).expect("stored job");
        assert_eq!(job.status, JobStatus::Dead);
    }

    #[tokio::test]
    async fn worker_marks_permanent_failure_dead_without_retrying() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            PermanentFailureRunner,
        );

        let outcome = worker.run_once(&mut store, now).await.expect("worker run");

        assert_eq!(outcome, WorkerRunOutcome::Dead(job_id));
        let job = store.job(job_id).expect("stored job");
        assert_eq!(job.status, JobStatus::Dead);
        assert_eq!(job.attempt_count.get(), 1);
    }

    #[tokio::test]
    async fn worker_recovers_expired_running_job() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            FailingThenSuccessfulRunner::new(SimulatedFailureCount::new(1)),
        )
        .with_lease_duration(LeaseDuration::from_secs(1));

        let picked = store
            .pick_due_job(
                WorkerId::new("stale-worker").expect("valid worker"),
                now,
                LeaseDuration::from_secs(1),
            )
            .await
            .expect("pick due job");
        assert!(picked.is_some());

        let later = now + Duration::from_secs(2);
        let outcome = worker
            .run_once(&mut store, later)
            .await
            .expect("worker run");

        assert_eq!(outcome, WorkerRunOutcome::RetryScheduled(job_id));
        let event_types: Vec<_> = store
            .events_for(job_id)
            .into_iter()
            .map(|event| event.event_type)
            .collect();
        assert!(event_types.contains(&JobEventType::ExpiredLeaseRecovered));
    }

    #[tokio::test]
    async fn worker_heartbeat_extends_owned_lease_and_records_event() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker_id = WorkerId::new("worker-a").expect("valid worker");
        let worker = Worker::new(worker_id.clone(), DeterministicAgentRunner)
            .with_lease_duration(LeaseDuration::from_secs(60));

        let picked = store
            .pick_due_job(worker_id, now, LeaseDuration::from_secs(10))
            .await
            .expect("pick job");
        assert!(picked.is_some());

        let heartbeat_at = now + Duration::from_secs(5);
        let outcome = worker
            .heartbeat(&mut store, job_id, heartbeat_at)
            .await
            .expect("heartbeat");

        assert_eq!(outcome, LeaseExtensionOutcome::Extended);
        assert_eq!(
            store.job(job_id).expect("stored job").locked_until,
            Some(heartbeat_at + Duration::from_secs(60))
        );
        let lease_extended_events = store
            .events_for(job_id)
            .into_iter()
            .filter(|event| event.event_type == JobEventType::LeaseExtended)
            .count();
        assert_eq!(lease_extended_events, 1);
    }

    #[tokio::test]
    async fn worker_heartbeat_rejects_stale_or_wrong_worker() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let owning_worker = WorkerId::new("worker-a").expect("valid worker");
        let stale_worker = Worker::new(
            WorkerId::new("worker-b").expect("valid worker"),
            DeterministicAgentRunner,
        );

        store
            .pick_due_job(owning_worker.clone(), now, LeaseDuration::from_secs(30))
            .await
            .expect("pick job");

        let outcome = stale_worker
            .heartbeat(&mut store, job_id, now + Duration::from_secs(5))
            .await
            .expect("heartbeat");

        assert_eq!(outcome, LeaseExtensionOutcome::NotOwnedOrNotRunning);
        assert_eq!(
            store.job(job_id).expect("stored job").locked_by.as_ref(),
            Some(&owning_worker)
        );
        assert!(
            !store
                .events_for(job_id)
                .iter()
                .any(|event| event.event_type == JobEventType::LeaseExtended)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn worker_run_once_with_heartbeats_extends_lease_during_long_agent_execution() {
        let now = now();
        let mut store = SharedInMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            DelayedAgentRunner::new(Duration::from_secs(5)),
        )
        .with_lease_duration(LeaseDuration::from_secs(20))
        .with_heartbeat_interval(HeartbeatInterval::from_secs(1).expect("valid interval"));

        let mut run_store = store.clone();
        let handle =
            tokio::spawn(async move { worker.run_once_with_heartbeats(&mut run_store, now).await });

        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(2)).await;
        tokio::task::yield_now().await;

        let heartbeat_events = store
            .events_for(job_id)
            .await
            .into_iter()
            .filter(|event| event.event_type == JobEventType::LeaseExtended)
            .count();
        assert!(
            heartbeat_events > 0,
            "long-running execution should renew the lease before completion"
        );

        tokio::time::advance(Duration::from_secs(5)).await;
        let outcome = handle
            .await
            .expect("worker task joined")
            .expect("worker run");

        assert_eq!(outcome, WorkerRunOutcome::Succeeded(job_id));
        assert_eq!(
            store.job(job_id).await.expect("stored job").status,
            JobStatus::Succeeded
        );
    }

    #[tokio::test(start_paused = true)]
    async fn worker_run_once_with_heartbeats_stops_when_lease_is_lost() {
        let now = now();
        let mut store = SharedInMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            DelayedAgentRunner::new(Duration::from_secs(10)),
        )
        .with_lease_duration(LeaseDuration::from_secs(20))
        .with_heartbeat_interval(HeartbeatInterval::from_secs(1).expect("valid interval"));

        let mut run_store = store.clone();
        let handle =
            tokio::spawn(async move { worker.run_once_with_heartbeats(&mut run_store, now).await });

        for _ in 0..3 {
            tokio::task::yield_now().await;
        }
        assert_eq!(
            store.job(job_id).await.expect("stored job").status,
            JobStatus::Running
        );

        store
            .cancel(
                job_id,
                CancellationReason::new("operator cancelled unsafe long work")
                    .expect("valid reason"),
                now,
            )
            .await
            .expect("cancel running job");

        tokio::time::advance(Duration::from_secs(2)).await;
        tokio::task::yield_now().await;

        let error = handle
            .await
            .expect("worker task joined")
            .expect_err("lost lease stops the worker");
        assert!(matches!(
            error,
            WorkerError::LeaseLost {
                job_id: lost_job_id,
                outcome: LeaseExtensionOutcome::NotOwnedOrNotRunning,
            } if lost_job_id == job_id
        ));
        assert_eq!(
            store.job(job_id).await.expect("stored job").status,
            JobStatus::Cancelled
        );
        assert!(
            !store
                .events_for(job_id)
                .await
                .iter()
                .any(|event| event.event_type == JobEventType::JobSucceeded)
        );
    }

    #[tokio::test]
    async fn worker_controlled_run_skips_claiming_due_work_while_draining() {
        let now = now();
        let mut store = SharedInMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            DeterministicAgentRunner,
        );

        let outcome = worker
            .run_once_controlled(
                &mut store,
                now,
                WorkerControl::draining(
                    ShutdownReason::new("deploy drain before shutdown").expect("valid reason"),
                ),
            )
            .await
            .expect("controlled worker run");

        assert_eq!(outcome, WorkerRunOutcome::DrainRequested);
        assert_eq!(
            store.job(job_id).await.expect("stored job").status,
            JobStatus::Pending
        );
        assert!(
            !store
                .events_for(job_id)
                .await
                .iter()
                .any(|event| event.event_type == JobEventType::JobPicked)
        );
    }

    #[tokio::test]
    async fn worker_bounded_loop_processes_due_jobs_until_idle() {
        let now = now();
        let mut store = SharedInMemoryAgentJobStore::new();
        let first_job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let second_job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            DeterministicAgentRunner,
        );

        let outcome = worker
            .run_bounded_loop(
                &mut store,
                now,
                WorkerControl::accepting_work(),
                WorkerCycleLimit::new(NonZeroU32::new(3).expect("non-zero cycle limit")),
            )
            .await
            .expect("bounded worker loop");

        assert_eq!(
            outcome,
            WorkerLoopOutcome::Idle {
                completed_cycles: CompletedWorkerCycles(2)
            }
        );
        assert_eq!(
            store.job(first_job_id).await.expect("first job").status,
            JobStatus::Succeeded
        );
        assert_eq!(
            store.job(second_job_id).await.expect("second job").status,
            JobStatus::Succeeded
        );
    }

    #[tokio::test]
    async fn worker_bounded_loop_stops_at_cycle_limit_before_claiming_extra_work() {
        let now = now();
        let mut store = SharedInMemoryAgentJobStore::new();
        let first_job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let second_job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            DeterministicAgentRunner,
        );

        let outcome = worker
            .run_bounded_loop(
                &mut store,
                now,
                WorkerControl::accepting_work(),
                WorkerCycleLimit::new(NonZeroU32::MIN),
            )
            .await
            .expect("bounded worker loop");

        assert_eq!(
            outcome,
            WorkerLoopOutcome::CycleLimitReached {
                completed_cycles: CompletedWorkerCycles(1)
            }
        );
        let statuses = [
            store.job(first_job_id).await.expect("first job").status,
            store.job(second_job_id).await.expect("second job").status,
        ];
        assert_eq!(
            statuses
                .iter()
                .filter(|status| **status == JobStatus::Succeeded)
                .count(),
            1
        );
        assert_eq!(
            statuses
                .iter()
                .filter(|status| **status == JobStatus::Pending)
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn worker_bounded_loop_honors_drain_before_first_claim() {
        let now = now();
        let mut store = SharedInMemoryAgentJobStore::new();
        let job_id = store.enqueue(test_job(3, now), now).await.expect("enqueue");
        let worker = Worker::new(
            WorkerId::new("worker-a").expect("valid worker"),
            DeterministicAgentRunner,
        );

        let outcome = worker
            .run_bounded_loop(
                &mut store,
                now,
                WorkerControl::draining(
                    ShutdownReason::new("operator requested deploy drain").expect("valid reason"),
                ),
                WorkerCycleLimit::new(NonZeroU32::new(3).expect("non-zero cycle limit")),
            )
            .await
            .expect("bounded worker loop");

        assert_eq!(
            outcome,
            WorkerLoopOutcome::Draining {
                completed_cycles: CompletedWorkerCycles::zero()
            }
        );
        assert_eq!(
            store.job(job_id).await.expect("stored job").status,
            JobStatus::Pending
        );
    }
}
