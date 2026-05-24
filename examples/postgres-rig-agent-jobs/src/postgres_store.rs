use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx_core::{query::query, query_scalar::query_scalar, row::Row};
use sqlx_postgres::{PgPool, PgRow, Postgres};
use thiserror::Error;
use uuid::Uuid;

use crate::admission_control::AdmissionDecisionEvent;
use crate::cost_accounting::CostMicrosUsd;
use crate::domain::{
    AgentEvent, AgentJob, AgentJobSnapshot, AgentJobVersions, AgentPayload, AgentResult,
    AttemptCount, CancellationOutcome, CancellationReason, DatabaseUrl, DomainError, EventMessage,
    FailureMessage, IdempotencyKey, JobEventType, JobId, JobKind, JobStatus, JobTransition,
    LeaseDuration, LeaseExtensionOutcome, MaxAttempts, ModelRoute, PayloadSchemaVersion,
    PolicyVersion, PromptVersion, QueueAge, QueueDepth, QueueMetrics, RetryDisposition,
    RetryPolicy, ToolVersion, WorkerBuildId, WorkerId,
};
use crate::worker::AgentJobStore;

#[derive(Debug, Error)]
pub enum PostgresStoreError {
    #[error("postgres query failed: {0}")]
    Database(#[from] sqlx_core::Error),
    #[error("stored row violates domain invariants: {0}")]
    Domain(#[from] DomainError),
    #[error("stored json does not match the expected payload/result shape: {0}")]
    Json(#[from] serde_json::Error),
    #[error("stored {field} cannot be negative, got {value}")]
    NegativeNumber { field: &'static str, value: i64 },
    #[error("stored {field} is too large to fit in memory, got {value}")]
    TooLarge { field: &'static str, value: i64 },
    #[error("stored agent job row violates lifecycle invariant: {invariant}")]
    InvalidAgentJobRow { invariant: AgentJobRowInvariant },
    #[error("job transition {transition} was rejected for {job_id:?}")]
    TransitionRejected {
        job_id: JobId,
        transition: JobTransition,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentJobRowInvariant {
    PayloadMustBeObject,
    ResultMustBeObject,
    RunningLeaseMissing,
    NonRunningLeasePresent,
    SucceededMissingResult,
    SucceededWithLastError,
    NonSucceededResultPresent,
}

impl std::fmt::Display for AgentJobRowInvariant {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PayloadMustBeObject => formatter.write_str("payload must be a JSON object"),
            Self::ResultMustBeObject => formatter.write_str("result must be a JSON object"),
            Self::RunningLeaseMissing => {
                formatter.write_str("running jobs must carry a complete lease")
            }
            Self::NonRunningLeasePresent => {
                formatter.write_str("non-running jobs must not carry a lease")
            }
            Self::SucceededMissingResult => {
                formatter.write_str("succeeded jobs must carry a result")
            }
            Self::SucceededWithLastError => {
                formatter.write_str("succeeded jobs must not carry a last error")
            }
            Self::NonSucceededResultPresent => {
                formatter.write_str("only succeeded jobs may carry a result")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresAgentJobStore {
    pool: PgPool,
}

impl PostgresAgentJobStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(database_url: DatabaseUrl) -> Result<Self, PostgresStoreError> {
        let pool = PgPool::connect(database_url.as_str()).await?;
        Ok(Self::new(pool))
    }

    pub async fn metrics(&self) -> Result<QueueMetrics, PostgresStoreError> {
        let row = query::<Postgres>(crate::sql::QUEUE_METRICS)
            .fetch_one(&self.pool)
            .await?;

        Ok(QueueMetrics {
            pending: queue_depth_from_i64(row.try_get("pending")?, "pending")?,
            running: queue_depth_from_i64(row.try_get("running")?, "running")?,
            succeeded: queue_depth_from_i64(row.try_get("succeeded")?, "succeeded")?,
            failed: queue_depth_from_i64(row.try_get("failed")?, "failed")?,
            dead: queue_depth_from_i64(row.try_get("dead")?, "dead")?,
            cancelled: queue_depth_from_i64(row.try_get("cancelled")?, "cancelled")?,
            oldest_pending_age: row
                .try_get::<Option<i64>, _>("oldest_pending_age_seconds")?
                .map(QueueAge::from_seconds)
                .transpose()?,
        })
    }
}

#[derive(Debug)]
struct EnqueueRow {
    id: Uuid,
    inserted: bool,
}

impl EnqueueRow {
    fn try_from_row(row: PgRow) -> Result<Self, sqlx_core::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            inserted: row.try_get("inserted")?,
        })
    }
}

#[derive(Debug)]
struct AgentJobRow {
    id: Uuid,
    kind: String,
    payload_schema_version: i32,
    prompt_version: String,
    model_route: String,
    tool_version: String,
    policy_version: String,
    worker_build_id: String,
    idempotency_key: Option<String>,
    status: String,
    payload: serde_json::Value,
    result: Option<serde_json::Value>,
    run_at: DateTime<Utc>,
    attempt_count: i32,
    max_attempts: i32,
    locked_by: Option<String>,
    locked_until: Option<DateTime<Utc>>,
    last_error: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl AgentJobRow {
    fn try_from_row(row: PgRow) -> Result<Self, sqlx_core::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            kind: row.try_get("kind")?,
            payload_schema_version: row.try_get("payload_schema_version")?,
            prompt_version: row.try_get("prompt_version")?,
            model_route: row.try_get("model_route")?,
            tool_version: row.try_get("tool_version")?,
            policy_version: row.try_get("policy_version")?,
            worker_build_id: row.try_get("worker_build_id")?,
            idempotency_key: row.try_get("idempotency_key")?,
            status: row.try_get("status")?,
            payload: row.try_get("payload")?,
            result: row.try_get("result")?,
            run_at: row.try_get("run_at")?,
            attempt_count: row.try_get("attempt_count")?,
            max_attempts: row.try_get("max_attempts")?,
            locked_by: row.try_get("locked_by")?,
            locked_until: row.try_get("locked_until")?,
            last_error: row.try_get("last_error")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    fn try_into_job(self) -> Result<AgentJob, PostgresStoreError> {
        if !self.payload.is_object() {
            return Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::PayloadMustBeObject,
            });
        }

        if self
            .result
            .as_ref()
            .is_some_and(|result| !result.is_object())
        {
            return Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::ResultMustBeObject,
            });
        }

        let attempt_count =
            u32::try_from(self.attempt_count).map_err(|_| PostgresStoreError::NegativeNumber {
                field: "attempt_count",
                value: i64::from(self.attempt_count),
            })?;
        let max_attempts =
            u32::try_from(self.max_attempts).map_err(|_| PostgresStoreError::NegativeNumber {
                field: "max_attempts",
                value: i64::from(self.max_attempts),
            })?;
        let payload_schema_version = u32::try_from(self.payload_schema_version).map_err(|_| {
            PostgresStoreError::NegativeNumber {
                field: "payload_schema_version",
                value: i64::from(self.payload_schema_version),
            }
        })?;
        let status = JobStatus::try_from(self.status.as_str())?;

        self.validate_lifecycle_evidence(status)?;

        Ok(AgentJob {
            id: JobId::from_uuid(self.id),
            kind: JobKind::new(self.kind)?,
            idempotency_key: self.idempotency_key.map(IdempotencyKey::new).transpose()?,
            versions: AgentJobVersions {
                payload_schema: PayloadSchemaVersion::try_from_u32(payload_schema_version)?,
                prompt: PromptVersion::new(self.prompt_version)?,
                model_route: ModelRoute::new(self.model_route)?,
                tool: ToolVersion::new(self.tool_version)?,
                policy: PolicyVersion::new(self.policy_version)?,
                worker_build: WorkerBuildId::new(self.worker_build_id)?,
            },
            status,
            payload: serde_json::from_value::<AgentPayload>(self.payload)?,
            result: self.result.map(serde_json::from_value).transpose()?,
            run_at: self.run_at,
            attempt_count: AttemptCount::try_from_u32(attempt_count)?,
            max_attempts: MaxAttempts::try_from_u32(max_attempts)?,
            locked_by: self.locked_by.map(WorkerId::new).transpose()?,
            locked_until: self.locked_until,
            last_error: self.last_error.map(FailureMessage::new).transpose()?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

    fn validate_lifecycle_evidence(&self, status: JobStatus) -> Result<(), PostgresStoreError> {
        let has_worker = self.locked_by.is_some();
        let has_deadline = self.locked_until.is_some();
        let has_complete_lease = has_worker && has_deadline;
        let has_partial_or_unexpected_lease = has_worker || has_deadline;

        if status == JobStatus::Running && !has_complete_lease {
            return Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::RunningLeaseMissing,
            });
        }

        if status != JobStatus::Running && has_partial_or_unexpected_lease {
            return Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::NonRunningLeasePresent,
            });
        }

        if status == JobStatus::Succeeded {
            if self.result.is_none() {
                return Err(PostgresStoreError::InvalidAgentJobRow {
                    invariant: AgentJobRowInvariant::SucceededMissingResult,
                });
            }

            if self.last_error.is_some() {
                return Err(PostgresStoreError::InvalidAgentJobRow {
                    invariant: AgentJobRowInvariant::SucceededWithLastError,
                });
            }
        } else if self.result.is_some() {
            return Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::NonSucceededResultPresent,
            });
        }

        Ok(())
    }
}

#[derive(Debug)]
struct AgentEventRow {
    job_id: Uuid,
    event_type: String,
    message: Option<String>,
    created_at: DateTime<Utc>,
}

impl AgentEventRow {
    fn try_from_row(row: PgRow) -> Result<Self, sqlx_core::Error> {
        Ok(Self {
            job_id: row.try_get("job_id")?,
            event_type: row.try_get("event_type")?,
            message: row.try_get("message")?,
            created_at: row.try_get("created_at")?,
        })
    }

    fn try_into_event(self) -> Result<AgentEvent, PostgresStoreError> {
        Ok(AgentEvent {
            job_id: JobId::from_uuid(self.job_id),
            event_type: JobEventType::try_from(self.event_type.as_str())?,
            message: self.message.map(EventMessage::new).transpose()?,
            created_at: self.created_at,
        })
    }
}

#[async_trait]
impl AgentJobStore for PostgresAgentJobStore {
    type Error = PostgresStoreError;

    async fn enqueue(&mut self, job: AgentJob, now: DateTime<Utc>) -> Result<JobId, Self::Error> {
        let payload = serde_json::to_value(&job.payload)?;
        let idempotency_key = job.idempotency_key.as_ref().map(IdempotencyKey::as_str);
        let row = query::<Postgres>(crate::sql::ENQUEUE_AGENT_JOB)
            .bind(job.id.as_uuid())
            .bind(job.kind.as_str())
            .bind(payload)
            .bind(idempotency_key)
            .bind(job.run_at)
            .bind(i32::try_from(job.max_attempts.get()).map_err(|_| {
                PostgresStoreError::TooLarge {
                    field: "max_attempts",
                    value: i64::from(job.max_attempts.get()),
                }
            })?)
            .bind(
                i32::try_from(job.versions.payload_schema.get()).map_err(|_| {
                    PostgresStoreError::TooLarge {
                        field: "payload_schema_version",
                        value: i64::from(job.versions.payload_schema.get()),
                    }
                })?,
            )
            .bind(job.versions.prompt.as_str())
            .bind(job.versions.model_route.as_str())
            .bind(job.versions.tool.as_str())
            .bind(job.versions.policy.as_str())
            .bind(job.versions.worker_build.as_str())
            .fetch_one(&self.pool)
            .await?;
        let row = EnqueueRow::try_from_row(row)?;

        let job_id = JobId::from_uuid(row.id);
        let event_type = if row.inserted {
            JobEventType::JobEnqueued
        } else {
            JobEventType::DuplicateSuppressed
        };
        self.record_event(
            job_id,
            event_type,
            job.idempotency_key.as_ref().map(EventMessage::from),
            now,
        )
        .await?;

        Ok(job_id)
    }

    async fn recover_expired(&mut self, now: DateTime<Utc>) -> Result<Vec<JobId>, Self::Error> {
        let rows = query::<Postgres>(crate::sql::RECOVER_EXPIRED_JOBS)
            .fetch_all(&self.pool)
            .await?;
        let job_ids = rows
            .into_iter()
            .map(|row| row.try_get::<Uuid, _>("id").map(JobId::from_uuid))
            .collect::<Result<Vec<_>, _>>()?;

        for job_id in &job_ids {
            self.record_event(*job_id, JobEventType::ExpiredLeaseRecovered, None, now)
                .await?;
        }

        Ok(job_ids)
    }

    async fn pick_due_job(
        &mut self,
        worker_id: WorkerId,
        now: DateTime<Utc>,
        lease_duration: LeaseDuration,
    ) -> Result<Option<AgentJob>, Self::Error> {
        let row = query::<Postgres>(crate::sql::PICK_DUE_JOB)
            .bind(worker_id.as_str())
            .bind(interval_literal(lease_duration.duration()))
            .fetch_optional(&self.pool)
            .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let job = AgentJobRow::try_from_row(row)?.try_into_job()?;
        self.record_event(job.id, JobEventType::JobPicked, None, now)
            .await?;
        Ok(Some(job))
    }

    async fn mark_succeeded(
        &mut self,
        job_id: JobId,
        worker_id: WorkerId,
        result: AgentResult,
        now: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        let query_result = query::<Postgres>(crate::sql::MARK_SUCCEEDED)
            .bind(job_id.as_uuid())
            .bind(serde_json::to_value(result)?)
            .bind(worker_id.as_str())
            .execute(&self.pool)
            .await?;

        if query_result.rows_affected() == 0 {
            return Err(PostgresStoreError::TransitionRejected {
                job_id,
                transition: JobTransition::Complete,
            });
        }

        self.record_event(job_id, JobEventType::JobSucceeded, None, now)
            .await?;
        Ok(())
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
        let attempt_count = query_scalar::<Postgres, i32>(
            "select attempt_count from agent_jobs where id = $1::uuid and locked_by = $2::text and status = 'running'",
        )
        .bind(job_id.as_uuid())
        .bind(worker_id.as_str())
        .fetch_optional(&self.pool)
        .await?;
        let Some(attempt_count) = attempt_count else {
            return Err(PostgresStoreError::TransitionRejected {
                job_id,
                transition: JobTransition::RetryOrDead,
            });
        };
        let attempt_count =
            u32::try_from(attempt_count).map_err(|_| PostgresStoreError::NegativeNumber {
                field: "attempt_count",
                value: i64::from(attempt_count),
            })?;

        let row = query::<Postgres>(crate::sql::RETRY_OR_DEAD)
            .bind(job_id.as_uuid())
            .bind(retry_disposition.as_str())
            .bind(interval_literal(
                retry_policy
                    .delay_for_attempt(AttemptCount::try_from_u32(attempt_count)?)
                    .duration(),
            ))
            .bind(error.as_str())
            .bind(worker_id.as_str())
            .fetch_optional(&self.pool)
            .await?;
        let Some(row) = row else {
            return Err(PostgresStoreError::TransitionRejected {
                job_id,
                transition: JobTransition::RetryOrDead,
            });
        };
        let status = JobStatus::try_from(row.try_get::<String, _>("status")?.as_str())?;
        let event_type = if status == JobStatus::Dead {
            JobEventType::JobDead
        } else {
            JobEventType::RetryScheduled
        };
        self.record_event(job_id, event_type, Some(error.into()), now)
            .await?;
        Ok(())
    }

    async fn extend_lease(
        &mut self,
        job_id: JobId,
        worker_id: WorkerId,
        _now: DateTime<Utc>,
        lease_duration: LeaseDuration,
    ) -> Result<LeaseExtensionOutcome, Self::Error> {
        let result = query::<Postgres>(crate::sql::EXTEND_LEASE)
            .bind(job_id.as_uuid())
            .bind(worker_id.as_str())
            .bind(interval_literal(lease_duration.duration()))
            .execute(&self.pool)
            .await?;

        if result.rows_affected() > 0 {
            self.record_event(
                job_id,
                JobEventType::LeaseExtended,
                Some(EventMessage::from(&worker_id)),
                Utc::now(),
            )
            .await?;
            return Ok(LeaseExtensionOutcome::Extended);
        }

        Ok(LeaseExtensionOutcome::NotOwnedOrNotRunning)
    }

    async fn cancel(
        &mut self,
        job_id: JobId,
        reason: CancellationReason,
        now: DateTime<Utc>,
    ) -> Result<CancellationOutcome, Self::Error> {
        let result = query::<Postgres>(crate::sql::MARK_CANCELLED)
            .bind(job_id.as_uuid())
            .bind(reason.as_str())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() > 0 {
            self.record_event(job_id, JobEventType::JobCancelled, Some(reason.into()), now)
                .await?;
            return Ok(CancellationOutcome::Cancelled);
        }

        Ok(CancellationOutcome::NotCancellable)
    }

    async fn record_event(
        &mut self,
        job_id: JobId,
        event_type: JobEventType,
        message: Option<EventMessage>,
        _now: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        query::<Postgres>(
            "insert into agent_job_events (job_id, event_type, message) values ($1, $2, $3)",
        )
        .bind(job_id.as_uuid())
        .bind(event_type.as_str())
        .bind(message.as_ref().map(EventMessage::as_str))
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

impl PostgresAgentJobStore {
    pub async fn existing_job_for_idempotency_key(
        &self,
        idempotency_key: &IdempotencyKey,
        _now: DateTime<Utc>,
    ) -> Result<Option<AgentJobSnapshot>, PostgresStoreError> {
        let row = query::<Postgres>(crate::sql::RESOLVE_EXISTING_AGENT_JOB)
            .bind(idempotency_key.as_str())
            .fetch_optional(&self.pool)
            .await?;
        let Some(row) = row else {
            return Ok(None);
        };

        let job_id = JobId::from_uuid(row.try_get("id")?);
        let status_text: String = row.try_get("status")?;
        let status = JobStatus::try_from(status_text.as_str())?;
        let run_at = row.try_get("run_at")?;
        Ok(Some(AgentJobSnapshot::new(job_id, status, run_at)))
    }

    pub async fn admit_agent_job(
        &self,
        job: AgentJob,
        event: AdmissionDecisionEvent,
        _now: DateTime<Utc>,
    ) -> Result<JobId, PostgresStoreError> {
        let payload = serde_json::to_value(&job.payload)?;
        let idempotency_key = job.idempotency_key.as_ref().map(IdempotencyKey::as_str);
        let budget = event.budget();
        let projected_cost = cost_micros_to_i64(budget.projected(), "projected_cost_micros_usd")?;
        let remaining_budget = budget
            .remaining()
            .map(|cost| cost_micros_to_i64(cost, "remaining_budget_micros_usd"))
            .transpose()?;
        let budget_limit = budget
            .limit()
            .map(|cost| cost_micros_to_i64(cost, "budget_limit_micros_usd"))
            .transpose()?;

        let row = query::<Postgres>(crate::sql::ADMIT_AGENT_JOB)
            .bind(job.id.as_uuid())
            .bind(job.kind.as_str())
            .bind(payload)
            .bind(idempotency_key)
            .bind(job.run_at)
            .bind(i32::try_from(job.max_attempts.get()).map_err(|_| {
                PostgresStoreError::TooLarge {
                    field: "max_attempts",
                    value: i64::from(job.max_attempts.get()),
                }
            })?)
            .bind(
                i32::try_from(job.versions.payload_schema.get()).map_err(|_| {
                    PostgresStoreError::TooLarge {
                        field: "payload_schema_version",
                        value: i64::from(job.versions.payload_schema.get()),
                    }
                })?,
            )
            .bind(job.versions.prompt.as_str())
            .bind(job.versions.model_route.as_str())
            .bind(job.versions.tool.as_str())
            .bind(job.versions.policy.as_str())
            .bind(job.versions.worker_build.as_str())
            .bind(event.request_id().as_uuid())
            .bind(event.tenant_key().as_str())
            .bind(event.job_kind().as_str())
            .bind(event.priority().as_str())
            .bind(event.queue_pressure().as_str())
            .bind(event.provider_pressure().as_str())
            .bind(budget.as_str())
            .bind(projected_cost)
            .bind(remaining_budget)
            .bind(budget_limit)
            .bind(event.decision().kind().as_str())
            .bind(event.decision().reason().as_str())
            .bind(event.decision().next_run_at())
            .bind(event.decided_at())
            .fetch_one(&self.pool)
            .await?;

        Ok(JobId::from_uuid(row.try_get("id")?))
    }

    pub async fn record_admission_decision(
        &self,
        event: AdmissionDecisionEvent,
    ) -> Result<(), PostgresStoreError> {
        let budget = event.budget();
        let projected_cost = cost_micros_to_i64(budget.projected(), "projected_cost_micros_usd")?;
        let remaining_budget = budget
            .remaining()
            .map(|cost| cost_micros_to_i64(cost, "remaining_budget_micros_usd"))
            .transpose()?;
        let budget_limit = budget
            .limit()
            .map(|cost| cost_micros_to_i64(cost, "budget_limit_micros_usd"))
            .transpose()?;

        query::<Postgres>(crate::sql::RECORD_ADMISSION_DECISION)
            .bind(event.request_id().as_uuid())
            .bind(event.job_id().map(JobId::as_uuid))
            .bind(event.tenant_key().as_str())
            .bind(event.job_kind().as_str())
            .bind(event.priority().as_str())
            .bind(event.queue_pressure().as_str())
            .bind(event.provider_pressure().as_str())
            .bind(budget.as_str())
            .bind(projected_cost)
            .bind(remaining_budget)
            .bind(budget_limit)
            .bind(event.decision().kind().as_str())
            .bind(event.decision().reason().as_str())
            .bind(event.decision().next_run_at())
            .bind(event.decided_at())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn events_for(&self, job_id: JobId) -> Result<Vec<AgentEvent>, PostgresStoreError> {
        let rows = query::<Postgres>(
            "select job_id, event_type, message, created_at
             from agent_job_events
             where job_id = $1
             order by created_at asc, id asc",
        )
        .bind(job_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            events.push(AgentEventRow::try_from_row(row)?.try_into_event()?);
        }
        Ok(events)
    }
}

fn interval_literal(duration: Duration) -> String {
    format!("{} seconds", duration.as_secs())
}

fn queue_depth_from_i64(value: i64, field: &'static str) -> Result<QueueDepth, PostgresStoreError> {
    if value < 0 {
        return Err(PostgresStoreError::NegativeNumber { field, value });
    }

    let value =
        usize::try_from(value).map_err(|_| PostgresStoreError::TooLarge { field, value })?;
    Ok(QueueDepth::try_from_usize(value)?)
}

fn cost_micros_to_i64(
    value: CostMicrosUsd,
    field: &'static str,
) -> Result<i64, PostgresStoreError> {
    i64::try_from(value.micros()).map_err(|_| PostgresStoreError::TooLarge {
        field,
        value: i64::MAX,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_literal_uses_seconds_for_postgres_interval_casts() {
        assert_eq!(interval_literal(Duration::from_secs(90)), "90 seconds");
    }

    #[test]
    fn queue_metric_conversion_rejects_negative_counts() {
        assert!(matches!(
            queue_depth_from_i64(-1, "pending"),
            Err(PostgresStoreError::NegativeNumber {
                field: "pending",
                value: -1
            })
        ));
    }

    fn valid_pending_row() -> AgentJobRow {
        let now = Utc::now();
        AgentJobRow {
            id: Uuid::new_v4(),
            kind: "incident_triage".to_string(),
            payload_schema_version: 1,
            prompt_version: "incident-triage:v1".to_string(),
            model_route: "deterministic-local:v1".to_string(),
            tool_version: "no-tools:v1".to_string(),
            policy_version: "approval-policy:v1".to_string(),
            worker_build_id: "test-build".to_string(),
            idempotency_key: Some("incident-1".to_string()),
            status: "pending".to_string(),
            payload: serde_json::json!({
                "instruction": "Analyze failed deployment"
            }),
            result: None,
            run_at: now,
            attempt_count: 0,
            max_attempts: 3,
            locked_by: None,
            locked_until: None,
            last_error: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn row_conversion_rejects_negative_attempt_count() {
        let row = AgentJobRow {
            id: Uuid::new_v4(),
            kind: "incident_triage".to_string(),
            payload_schema_version: 1,
            prompt_version: "incident-triage:v1".to_string(),
            model_route: "deterministic-local:v1".to_string(),
            tool_version: "no-tools:v1".to_string(),
            policy_version: "approval-policy:v1".to_string(),
            worker_build_id: "test-build".to_string(),
            idempotency_key: None,
            status: "pending".to_string(),
            payload: serde_json::json!({
                "instruction": "Analyze failed deployment"
            }),
            result: None,
            run_at: Utc::now(),
            attempt_count: -1,
            max_attempts: 3,
            locked_by: None,
            locked_until: None,
            last_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(matches!(
            row.try_into_job(),
            Err(PostgresStoreError::NegativeNumber {
                field: "attempt_count",
                value: -1
            })
        ));
    }

    #[test]
    fn row_conversion_validates_nested_domain_payload() {
        let row = AgentJobRow {
            id: Uuid::new_v4(),
            kind: "incident_triage".to_string(),
            payload_schema_version: 1,
            prompt_version: "incident-triage:v1".to_string(),
            model_route: "deterministic-local:v1".to_string(),
            tool_version: "no-tools:v1".to_string(),
            policy_version: "approval-policy:v1".to_string(),
            worker_build_id: "test-build".to_string(),
            idempotency_key: Some("incident-1".to_string()),
            status: "succeeded".to_string(),
            payload: serde_json::json!({
                "instruction": "Analyze failed deployment"
            }),
            result: Some(serde_json::json!({
                "summary": "Safe summary",
                "next_action": "Review",
                "approval": "required"
            })),
            run_at: Utc::now(),
            attempt_count: 1,
            max_attempts: 3,
            locked_by: None,
            locked_until: None,
            last_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let job = row.try_into_job().expect("valid row");

        assert_eq!(job.kind.as_str(), "incident_triage");
        assert!(job.idempotency_key.is_some());
        assert!(job.result.is_some());
        assert_eq!(job.versions.worker_build.as_str(), "test-build");
    }

    #[test]
    fn row_conversion_rejects_invalid_payload_schema_version() {
        let row = AgentJobRow {
            id: Uuid::new_v4(),
            kind: "incident_triage".to_string(),
            payload_schema_version: 0,
            prompt_version: "incident-triage:v1".to_string(),
            model_route: "deterministic-local:v1".to_string(),
            tool_version: "no-tools:v1".to_string(),
            policy_version: "approval-policy:v1".to_string(),
            worker_build_id: "test-build".to_string(),
            idempotency_key: None,
            status: "pending".to_string(),
            payload: serde_json::json!({
                "instruction": "Analyze failed deployment"
            }),
            result: None,
            run_at: Utc::now(),
            attempt_count: 0,
            max_attempts: 3,
            locked_by: None,
            locked_until: None,
            last_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(matches!(
            row.try_into_job(),
            Err(PostgresStoreError::Domain(DomainError::NonPositiveNumber {
                field: "payload_schema_version",
                value: 0
            }))
        ));
    }

    #[test]
    fn row_conversion_rejects_running_without_complete_lease() {
        let row = AgentJobRow {
            status: "running".to_string(),
            locked_by: Some("worker-a".to_string()),
            locked_until: None,
            ..valid_pending_row()
        };

        assert!(matches!(
            row.try_into_job(),
            Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::RunningLeaseMissing
            })
        ));
    }

    #[test]
    fn row_conversion_rejects_non_running_with_lease() {
        let row = AgentJobRow {
            locked_by: Some("worker-a".to_string()),
            locked_until: Some(Utc::now()),
            ..valid_pending_row()
        };

        assert!(matches!(
            row.try_into_job(),
            Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::NonRunningLeasePresent
            })
        ));
    }

    #[test]
    fn row_conversion_rejects_succeeded_without_result() {
        let row = AgentJobRow {
            status: "succeeded".to_string(),
            result: None,
            ..valid_pending_row()
        };

        assert!(matches!(
            row.try_into_job(),
            Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::SucceededMissingResult
            })
        ));
    }

    #[test]
    fn row_conversion_rejects_succeeded_with_last_error() {
        let row = AgentJobRow {
            status: "succeeded".to_string(),
            result: Some(serde_json::json!({
                "summary": "Safe summary",
                "next_action": "Review",
                "approval": "required"
            })),
            last_error: Some("old failure leaked into succeeded row".to_string()),
            ..valid_pending_row()
        };

        assert!(matches!(
            row.try_into_job(),
            Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::SucceededWithLastError
            })
        ));
    }

    #[test]
    fn row_conversion_rejects_non_succeeded_with_result() {
        let row = AgentJobRow {
            result: Some(serde_json::json!({
                "summary": "Safe summary",
                "next_action": "Review",
                "approval": "required"
            })),
            ..valid_pending_row()
        };

        assert!(matches!(
            row.try_into_job(),
            Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::NonSucceededResultPresent
            })
        ));
    }

    #[test]
    fn row_conversion_rejects_non_object_payload() {
        let row = AgentJobRow {
            payload: serde_json::json!(["not", "an", "object"]),
            ..valid_pending_row()
        };

        assert!(matches!(
            row.try_into_job(),
            Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::PayloadMustBeObject
            })
        ));
    }

    #[test]
    fn row_conversion_rejects_non_object_result() {
        let row = AgentJobRow {
            status: "succeeded".to_string(),
            result: Some(serde_json::json!(["not", "an", "object"])),
            ..valid_pending_row()
        };

        assert!(matches!(
            row.try_into_job(),
            Err(PostgresStoreError::InvalidAgentJobRow {
                invariant: AgentJobRowInvariant::ResultMustBeObject
            })
        ));
    }
}
