use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use thiserror::Error;

use crate::admission_control::AdmissionDecisionEvent;
use crate::domain::{
    AgentEvent, AgentJob, AgentJobSnapshot, AgentResult, CancellationOutcome, CancellationReason,
    CompletionTransitionOutcome, EventMessage, FailureMessage, IdempotencyKey, JobEventType, JobId,
    JobStatus, JobTransition, LeaseDuration, LeaseExtensionOutcome, QueueMetrics, RetryDisposition,
    RetryPolicy, RetryTransitionOutcome, WorkerId,
};
use crate::worker::AgentJobStore;

#[derive(Debug, Error)]
pub enum InMemoryStoreError {
    #[error("job not found: {0:?}")]
    JobNotFound(JobId),
    #[error("job transition {transition} was rejected for {job_id:?}")]
    TransitionRejected {
        job_id: JobId,
        transition: JobTransition,
    },
}

#[derive(Debug, Default)]
pub struct InMemoryAgentJobStore {
    jobs: BTreeMap<uuid::Uuid, AgentJob>,
    idempotency_index: BTreeMap<IdempotencyKey, JobId>,
    events: Vec<AgentEvent>,
    admission_events: Vec<AdmissionDecisionEvent>,
}

impl InMemoryAgentJobStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn job(&self, job_id: JobId) -> Option<&AgentJob> {
        self.jobs.get(&job_id.as_uuid())
    }

    pub fn events_for(&self, job_id: JobId) -> Vec<AgentEvent> {
        self.events
            .iter()
            .filter(|event| event.job_id == job_id)
            .cloned()
            .collect()
    }

    pub fn admission_events(&self) -> &[AdmissionDecisionEvent] {
        &self.admission_events
    }

    pub async fn record_admission_decision(
        &mut self,
        event: AdmissionDecisionEvent,
    ) -> Result<(), InMemoryStoreError> {
        self.admission_events.push(event);
        Ok(())
    }

    pub async fn existing_job_for_idempotency_key(
        &mut self,
        idempotency_key: &IdempotencyKey,
        now: DateTime<Utc>,
    ) -> Result<Option<AgentJobSnapshot>, InMemoryStoreError> {
        let Some(existing_job_id) = self.idempotency_index.get(idempotency_key).copied() else {
            return Ok(None);
        };
        let snapshot = self
            .jobs
            .get(&existing_job_id.as_uuid())
            .map(AgentJobSnapshot::from)
            .ok_or(InMemoryStoreError::JobNotFound(existing_job_id))?;

        self.record_event(
            existing_job_id,
            JobEventType::DuplicateSuppressed,
            Some(EventMessage::from(idempotency_key)),
            now,
        )
        .await?;

        Ok(Some(snapshot))
    }

    pub async fn admit_agent_job(
        &mut self,
        job: AgentJob,
        event: AdmissionDecisionEvent,
        now: DateTime<Utc>,
    ) -> Result<JobId, InMemoryStoreError> {
        let job_id = self.enqueue(job, now).await?;
        self.record_admission_decision(event.with_job_id(job_id))
            .await?;
        Ok(job_id)
    }

    pub fn metrics(&self, now: DateTime<Utc>) -> QueueMetrics {
        let mut metrics = QueueMetrics::default();
        let mut oldest_pending: Option<DateTime<Utc>> = None;

        for job in self.jobs.values() {
            match job.status {
                JobStatus::Pending => {
                    metrics.pending = metrics.pending.increment();
                    oldest_pending = Some(
                        oldest_pending
                            .map_or(job.created_at, |existing| existing.min(job.created_at)),
                    );
                }
                JobStatus::Running => metrics.running = metrics.running.increment(),
                JobStatus::Succeeded => metrics.succeeded = metrics.succeeded.increment(),
                JobStatus::Failed => metrics.failed = metrics.failed.increment(),
                JobStatus::Dead => metrics.dead = metrics.dead.increment(),
                JobStatus::Cancelled => metrics.cancelled = metrics.cancelled.increment(),
            }
        }

        metrics.oldest_pending_age = oldest_pending.map(|created_at| {
            crate::domain::QueueAge::saturating_from_seconds((now - created_at).num_seconds())
        });
        metrics
    }
}

#[async_trait]
impl AgentJobStore for InMemoryAgentJobStore {
    type Error = InMemoryStoreError;

    async fn enqueue(&mut self, job: AgentJob, now: DateTime<Utc>) -> Result<JobId, Self::Error> {
        let job_id = job.id;

        if let Some(idempotency_key) = &job.idempotency_key {
            if let Some(existing_job_id) = self.idempotency_index.get(idempotency_key).copied() {
                self.record_event(
                    existing_job_id,
                    JobEventType::DuplicateSuppressed,
                    Some(EventMessage::from(idempotency_key)),
                    now,
                )
                .await?;
                return Ok(existing_job_id);
            }

            self.idempotency_index
                .insert(idempotency_key.clone(), job_id);
        }

        self.jobs.insert(job_id.as_uuid(), job);
        self.record_event(job_id, JobEventType::JobEnqueued, None, now)
            .await?;
        Ok(job_id)
    }

    async fn recover_expired(&mut self, now: DateTime<Utc>) -> Result<Vec<JobId>, Self::Error> {
        let mut recovered = Vec::new();

        for job in self.jobs.values_mut() {
            if job.recover_expired_lease(now) == LeaseExtensionOutcome::Extended {
                recovered.push(job.id);
            }
        }

        for job_id in &recovered {
            self.record_event(*job_id, JobEventType::ExpiredLeaseRecovered, None, now)
                .await?;
        }

        Ok(recovered)
    }

    async fn pick_due_job(
        &mut self,
        worker_id: WorkerId,
        now: DateTime<Utc>,
        lease_duration: LeaseDuration,
    ) -> Result<Option<AgentJob>, Self::Error> {
        let due_job_id = self
            .jobs
            .values()
            .filter(|job| job.is_due(now))
            .min_by_key(|job| (job.run_at, job.created_at))
            .map(|job| job.id);

        let Some(job_id) = due_job_id else {
            return Ok(None);
        };

        let job = self
            .jobs
            .get_mut(&job_id.as_uuid())
            .ok_or(InMemoryStoreError::JobNotFound(job_id))?;
        job.lease_to(worker_id, now + lease_duration.duration(), now);
        let leased_job = job.clone();

        self.record_event(job_id, JobEventType::JobPicked, None, now)
            .await?;

        Ok(Some(leased_job))
    }

    async fn mark_succeeded(
        &mut self,
        job_id: JobId,
        worker_id: WorkerId,
        result: AgentResult,
        now: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        let job = self
            .jobs
            .get_mut(&job_id.as_uuid())
            .ok_or(InMemoryStoreError::JobNotFound(job_id))?;
        let outcome = job.mark_succeeded(&worker_id, result, now);

        if outcome == CompletionTransitionOutcome::NotOwnedOrNotRunning {
            return Err(InMemoryStoreError::TransitionRejected {
                job_id,
                transition: JobTransition::Complete,
            });
        }

        self.record_event(job_id, JobEventType::JobSucceeded, None, now)
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
        let job = self
            .jobs
            .get_mut(&job_id.as_uuid())
            .ok_or(InMemoryStoreError::JobNotFound(job_id))?;
        let outcome = job.retry_or_dead(
            &worker_id,
            error.clone(),
            retry_policy,
            retry_disposition,
            now,
        );

        match outcome {
            RetryTransitionOutcome::Dead => {
                self.record_event(job_id, JobEventType::JobDead, Some(error.into()), now)
                    .await
            }
            RetryTransitionOutcome::RetryScheduled => {
                self.record_event(
                    job_id,
                    JobEventType::RetryScheduled,
                    Some(error.into()),
                    now,
                )
                .await
            }
            RetryTransitionOutcome::NotOwnedOrNotRunning => {
                Err(InMemoryStoreError::TransitionRejected {
                    job_id,
                    transition: JobTransition::RetryOrDead,
                })
            }
        }
    }

    async fn extend_lease(
        &mut self,
        job_id: JobId,
        worker_id: WorkerId,
        now: DateTime<Utc>,
        lease_duration: LeaseDuration,
    ) -> Result<LeaseExtensionOutcome, Self::Error> {
        let job = self
            .jobs
            .get_mut(&job_id.as_uuid())
            .ok_or(InMemoryStoreError::JobNotFound(job_id))?;
        let outcome = job.extend_lease(&worker_id, now + lease_duration.duration(), now);

        if outcome == LeaseExtensionOutcome::Extended {
            self.record_event(
                job_id,
                JobEventType::LeaseExtended,
                Some(EventMessage::from(&worker_id)),
                now,
            )
            .await?;
        }

        Ok(outcome)
    }

    async fn cancel(
        &mut self,
        job_id: JobId,
        reason: CancellationReason,
        now: DateTime<Utc>,
    ) -> Result<CancellationOutcome, Self::Error> {
        let job = self
            .jobs
            .get_mut(&job_id.as_uuid())
            .ok_or(InMemoryStoreError::JobNotFound(job_id))?;
        let outcome = job.cancel(reason.clone(), now);

        if outcome == CancellationOutcome::Cancelled {
            self.record_event(job_id, JobEventType::JobCancelled, Some(reason.into()), now)
                .await?;
        }

        Ok(outcome)
    }

    async fn record_event(
        &mut self,
        job_id: JobId,
        event_type: JobEventType,
        message: Option<EventMessage>,
        now: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        self.events.push(AgentEvent {
            job_id,
            event_type,
            message,
            created_at: now,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;
    use std::time::Duration;

    use chrono::TimeZone;

    use crate::domain::{
        AgentInstruction, AgentPayload, IdempotencyKey, JobKind, JobStatus, MaxAttempts,
    };

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 9, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    fn test_job(now: DateTime<Utc>) -> AgentJob {
        AgentJob::new(
            JobKind::new("incident_triage").expect("valid kind"),
            AgentPayload {
                instruction: AgentInstruction::new("Analyze failed deployment")
                    .expect("valid instruction"),
            },
            MaxAttempts::new(NonZeroU32::new(3).expect("non-zero attempts")),
            now,
        )
    }

    #[tokio::test]
    async fn enqueue_returns_existing_job_for_duplicate_idempotency_key() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let idempotency_key = IdempotencyKey::new("deploy-incident-2026-05-23").expect("valid key");

        let first_job_id = store
            .enqueue(
                test_job(now).with_idempotency_key(idempotency_key.clone()),
                now,
            )
            .await
            .expect("enqueue first job");
        let second_job_id = store
            .enqueue(test_job(now).with_idempotency_key(idempotency_key), now)
            .await
            .expect("enqueue duplicate job");

        assert_eq!(first_job_id, second_job_id);
        assert_eq!(store.jobs.len(), 1);
        let event_types: Vec<_> = store
            .events_for(first_job_id)
            .into_iter()
            .map(|event| event.event_type)
            .collect();
        assert!(event_types.contains(&JobEventType::DuplicateSuppressed));
    }

    #[tokio::test]
    async fn extend_lease_only_works_for_the_worker_that_owns_the_job() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store
            .enqueue(test_job(now), now)
            .await
            .expect("enqueue job");
        let worker_id = WorkerId::new("worker-a").expect("valid worker");

        let picked = store
            .pick_due_job(worker_id.clone(), now, LeaseDuration::from_secs(30))
            .await
            .expect("pick job");
        assert!(picked.is_some());

        let wrong_worker_extended = store
            .extend_lease(
                job_id,
                WorkerId::new("worker-b").expect("valid worker"),
                now,
                LeaseDuration::from_secs(60),
            )
            .await
            .expect("wrong worker extend");
        let owning_worker_extended = store
            .extend_lease(job_id, worker_id, now, LeaseDuration::from_secs(60))
            .await
            .expect("owning worker extend");

        assert_eq!(
            wrong_worker_extended,
            LeaseExtensionOutcome::NotOwnedOrNotRunning
        );
        assert_eq!(owning_worker_extended, LeaseExtensionOutcome::Extended);
        let event_types: Vec<_> = store
            .events_for(job_id)
            .into_iter()
            .map(|event| event.event_type)
            .collect();
        assert_eq!(
            event_types
                .iter()
                .filter(|event_type| **event_type == JobEventType::LeaseExtended)
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn completion_requires_the_worker_that_owns_the_lease() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store
            .enqueue(test_job(now), now)
            .await
            .expect("enqueue job");
        let owning_worker = WorkerId::new("worker-a").expect("valid worker");

        store
            .pick_due_job(owning_worker.clone(), now, LeaseDuration::from_secs(30))
            .await
            .expect("pick job");
        let result = AgentResult::new(
            crate::domain::AgentSummary::new("incident is understood").expect("valid summary"),
            crate::domain::NextAction::new("notify operator").expect("valid next action"),
            crate::domain::ApprovalRequirement::Required,
        );

        let wrong_worker_result = store
            .mark_succeeded(
                job_id,
                WorkerId::new("worker-b").expect("valid worker"),
                result.clone(),
                now,
            )
            .await;

        assert!(matches!(
            wrong_worker_result,
            Err(InMemoryStoreError::TransitionRejected {
                transition: JobTransition::Complete,
                ..
            })
        ));
        assert_eq!(
            store.job(job_id).expect("stored job").status,
            JobStatus::Running
        );

        store
            .mark_succeeded(job_id, owning_worker, result, now)
            .await
            .expect("owning worker completes");
        assert_eq!(
            store.job(job_id).expect("stored job").status,
            JobStatus::Succeeded
        );
    }

    #[tokio::test]
    async fn retry_requires_the_worker_that_owns_the_lease() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store
            .enqueue(test_job(now), now)
            .await
            .expect("enqueue job");
        let owning_worker = WorkerId::new("worker-a").expect("valid worker");

        store
            .pick_due_job(owning_worker.clone(), now, LeaseDuration::from_secs(30))
            .await
            .expect("pick job");

        let wrong_worker_result = store
            .retry_or_dead(
                job_id,
                WorkerId::new("worker-b").expect("valid worker"),
                FailureMessage::new("provider timeout").expect("valid failure"),
                RetryPolicy::default(),
                RetryDisposition::Retryable,
                now,
            )
            .await;

        assert!(matches!(
            wrong_worker_result,
            Err(InMemoryStoreError::TransitionRejected {
                transition: JobTransition::RetryOrDead,
                ..
            })
        ));
        assert_eq!(
            store.job(job_id).expect("stored job").status,
            JobStatus::Running
        );

        store
            .retry_or_dead(
                job_id,
                owning_worker,
                FailureMessage::new("provider timeout").expect("valid failure"),
                RetryPolicy::default(),
                RetryDisposition::Retryable,
                now,
            )
            .await
            .expect("owning worker retries");
        assert_eq!(
            store.job(job_id).expect("stored job").status,
            JobStatus::Pending
        );
    }

    #[tokio::test]
    async fn cancel_releases_pending_or_running_job() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let job_id = store
            .enqueue(test_job(now), now)
            .await
            .expect("enqueue job");

        let cancelled = store
            .cancel(
                job_id,
                CancellationReason::new("operator stopped unsafe run").expect("valid reason"),
                now,
            )
            .await
            .expect("cancel job");

        assert_eq!(cancelled, CancellationOutcome::Cancelled);
        let job = store.job(job_id).expect("stored job");
        assert_eq!(job.status, JobStatus::Cancelled);
        assert!(job.locked_by.is_none());
        assert!(job.locked_until.is_none());
    }

    #[tokio::test]
    async fn metrics_report_queue_state_and_oldest_pending_age() {
        let now = now();
        let mut store = InMemoryAgentJobStore::new();
        let mut old_job = AgentJob::new(
            JobKind::new("incident_triage").expect("valid kind"),
            AgentPayload {
                instruction: AgentInstruction::new("Old pending job").expect("valid instruction"),
            },
            MaxAttempts::default(),
            now - Duration::from_secs(120),
        );
        old_job.run_at = now + Duration::from_secs(3600);
        store.enqueue(old_job, now).await.expect("enqueue old job");
        let running_job_id = store
            .enqueue(test_job(now), now)
            .await
            .expect("enqueue job");
        store
            .pick_due_job(
                WorkerId::new("worker-a").expect("valid worker"),
                now,
                LeaseDuration::from_secs(30),
            )
            .await
            .expect("pick job");

        let metrics = store.metrics(now);

        assert_eq!(metrics.pending.get(), 1);
        assert_eq!(metrics.running.get(), 1);
        assert_eq!(
            metrics.oldest_pending_age.map(|age| age.seconds()),
            Some(120)
        );
        assert!(store.job(running_job_id).is_some());
    }
}
