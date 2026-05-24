//! Typed lifecycle for the `scheduled_jobs` table.
//!
//! A scheduled job is the generic durable promise that work will survive
//! process death. The worker may only execute it after claiming a lease, and
//! failures become either a future retry or a terminal dead-letter state.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    AttemptCount, DomainError, FailureMessage, IdempotencyKey, JobStatus, MaxAttempts, WorkerId,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScheduledJobError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} cannot be negative, got {value}")]
    NegativeNumber { field: &'static str, value: i64 },
    #[error("{field} is too large, got {value}")]
    NumberOutOfRange { field: &'static str, value: i64 },
    #[error("{field} must be positive, got {value}")]
    NonPositiveNumber { field: &'static str, value: i64 },
    #[error("scheduled job payload must be a JSON object")]
    PayloadMustBeObject,
    #[error("running scheduled job requires locked_by and locked_until")]
    MissingRunningLease,
    #[error("non-running scheduled job with status {status:?} must not have lease fields")]
    UnexpectedLease { status: JobStatus },
    #[error("scheduled job has exhausted attempts")]
    AttemptsExhausted,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_scheduled_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, ScheduledJobError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(ScheduledJobError::EmptyText { field });
    }
    Ok(value)
}

fn non_negative_u32(value: i64, field: &'static str) -> Result<u32, ScheduledJobError> {
    if value < 0 {
        return Err(ScheduledJobError::NegativeNumber { field, value });
    }

    u32::try_from(value).map_err(|_| ScheduledJobError::NumberOutOfRange { field, value })
}

fn positive_u32(value: i64, field: &'static str) -> Result<u32, ScheduledJobError> {
    if value <= 0 {
        return Err(ScheduledJobError::NonPositiveNumber { field, value });
    }

    non_negative_u32(value, field)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScheduledJobId(Uuid);

impl ScheduledJobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for ScheduledJobId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledTaskName(String);

impl ScheduledTaskName {
    pub fn new(value: impl Into<String>) -> Result<Self, ScheduledJobError> {
        Ok(Self(non_empty_scheduled_text(
            value,
            "scheduled_task_name",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledJobPayload(Value);

impl ScheduledJobPayload {
    pub fn new(value: Value) -> Result<Self, ScheduledJobError> {
        if !value.is_object() {
            return Err(ScheduledJobError::PayloadMustBeObject);
        }

        Ok(Self(value))
    }

    pub fn as_json(&self) -> &Value {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScheduledJobRecord {
    id: ScheduledJobId,
    task_name: ScheduledTaskName,
    payload: ScheduledJobPayload,
    attempts: AttemptCount,
    max_attempts: MaxAttempts,
    next_run_at: DateTime<Utc>,
    locked_by: Option<WorkerId>,
    locked_until: Option<DateTime<Utc>>,
    last_error: Option<FailureMessage>,
    idempotency_key: Option<IdempotencyKey>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// ANCHOR: scheduled_job_typestate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledPending;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledRunning;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledSucceeded;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledFailed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledDead;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledCancelled;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledJob<State> {
    record: ScheduledJobRecord,
    state: PhantomData<State>,
}

impl ScheduledJob<ScheduledPending> {
    pub fn claim(
        mut self,
        worker_id: WorkerId,
        locked_until: DateTime<Utc>,
        claimed_at: DateTime<Utc>,
    ) -> Result<ScheduledJob<ScheduledRunning>, ScheduledJobError> {
        if self.record.attempts.get() >= self.record.max_attempts.get() {
            return Err(ScheduledJobError::AttemptsExhausted);
        }

        self.record.attempts = self.record.attempts.increment();
        self.record.locked_by = Some(worker_id);
        self.record.locked_until = Some(locked_until);
        self.record.updated_at = claimed_at;

        Ok(ScheduledJob {
            record: self.record,
            state: PhantomData,
        })
    }
}

impl ScheduledJob<ScheduledRunning> {
    pub fn mark_succeeded(
        mut self,
        completed_at: DateTime<Utc>,
    ) -> ScheduledJob<ScheduledSucceeded> {
        self.record.locked_by = None;
        self.record.locked_until = None;
        self.record.last_error = None;
        self.record.updated_at = completed_at;

        ScheduledJob {
            record: self.record,
            state: PhantomData,
        }
    }

    pub fn fail(
        mut self,
        failure: FailureMessage,
        next_run_at: DateTime<Utc>,
        failed_at: DateTime<Utc>,
    ) -> ScheduledFailureTransition {
        self.record.locked_by = None;
        self.record.locked_until = None;
        self.record.last_error = Some(failure);
        self.record.updated_at = failed_at;

        if self.record.attempts.get() >= self.record.max_attempts.get() {
            ScheduledFailureTransition::Dead(ScheduledJob {
                record: self.record,
                state: PhantomData,
            })
        } else {
            self.record.next_run_at = next_run_at;
            ScheduledFailureTransition::Retry(ScheduledJob {
                record: self.record,
                state: PhantomData,
            })
        }
    }

    pub fn cancel(
        mut self,
        reason: FailureMessage,
        cancelled_at: DateTime<Utc>,
    ) -> ScheduledJob<ScheduledCancelled> {
        self.record.locked_by = None;
        self.record.locked_until = None;
        self.record.last_error = Some(reason);
        self.record.updated_at = cancelled_at;

        ScheduledJob {
            record: self.record,
            state: PhantomData,
        }
    }
}
// ANCHOR_END: scheduled_job_typestate

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduledFailureTransition {
    Retry(ScheduledJob<ScheduledPending>),
    Dead(ScheduledJob<ScheduledDead>),
}

impl<State> ScheduledJob<State> {
    pub fn id(&self) -> ScheduledJobId {
        self.record.id
    }

    pub fn task_name(&self) -> &ScheduledTaskName {
        &self.record.task_name
    }

    pub fn payload(&self) -> &ScheduledJobPayload {
        &self.record.payload
    }

    pub fn attempts(&self) -> AttemptCount {
        self.record.attempts
    }

    pub fn max_attempts(&self) -> MaxAttempts {
        self.record.max_attempts
    }

    pub fn next_run_at(&self) -> DateTime<Utc> {
        self.record.next_run_at
    }

    pub fn locked_by(&self) -> Option<&WorkerId> {
        self.record.locked_by.as_ref()
    }

    pub fn locked_until(&self) -> Option<DateTime<Utc>> {
        self.record.locked_until
    }

    pub fn last_error(&self) -> Option<&FailureMessage> {
        self.record.last_error.as_ref()
    }

    pub fn idempotency_key(&self) -> Option<&IdempotencyKey> {
        self.record.idempotency_key.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.record.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.record.updated_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedScheduledJob {
    Pending(ScheduledJob<ScheduledPending>),
    Running(ScheduledJob<ScheduledRunning>),
    Succeeded(ScheduledJob<ScheduledSucceeded>),
    Failed(ScheduledJob<ScheduledFailed>),
    Dead(ScheduledJob<ScheduledDead>),
    Cancelled(ScheduledJob<ScheduledCancelled>),
}

impl DecodedScheduledJob {
    pub fn status(&self) -> JobStatus {
        match self {
            Self::Pending(_) => JobStatus::Pending,
            Self::Running(_) => JobStatus::Running,
            Self::Succeeded(_) => JobStatus::Succeeded,
            Self::Failed(_) => JobStatus::Failed,
            Self::Dead(_) => JobStatus::Dead,
            Self::Cancelled(_) => JobStatus::Cancelled,
        }
    }
}

// ANCHOR: scheduled_job_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbScheduledJobRow {
    pub id: Uuid,
    pub task_name: String,
    pub status: String,
    pub payload: Value,
    pub attempts: i64,
    pub max_attempts: i64,
    pub next_run_at: DateTime<Utc>,
    pub locked_by: Option<String>,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<DbScheduledJobRow> for DecodedScheduledJob {
    type Error = ScheduledJobError;

    fn try_from(row: DbScheduledJobRow) -> Result<Self, Self::Error> {
        let status = JobStatus::try_from(row.status.as_str())?;
        let attempts = AttemptCount::try_from_u32(non_negative_u32(row.attempts, "attempts")?)?;
        let max_attempts =
            MaxAttempts::try_from_u32(positive_u32(row.max_attempts, "max_attempts")?)?;

        let locked_by = row.locked_by.map(WorkerId::new).transpose()?;
        let locked_until = row.locked_until;
        match (status, locked_by.is_some(), locked_until.is_some()) {
            (JobStatus::Running, true, true) => {}
            (JobStatus::Running, _, _) => return Err(ScheduledJobError::MissingRunningLease),
            (_, false, false) => {}
            _ => return Err(ScheduledJobError::UnexpectedLease { status }),
        }

        let record = ScheduledJobRecord {
            id: ScheduledJobId::from_uuid(row.id),
            task_name: ScheduledTaskName::new(row.task_name)?,
            payload: ScheduledJobPayload::new(row.payload)?,
            attempts,
            max_attempts,
            next_run_at: row.next_run_at,
            locked_by,
            locked_until,
            last_error: row.last_error.map(FailureMessage::new).transpose()?,
            idempotency_key: row.idempotency_key.map(IdempotencyKey::new).transpose()?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        Ok(match status {
            JobStatus::Pending => DecodedScheduledJob::Pending(ScheduledJob {
                record,
                state: PhantomData,
            }),
            JobStatus::Running => DecodedScheduledJob::Running(ScheduledJob {
                record,
                state: PhantomData,
            }),
            JobStatus::Succeeded => DecodedScheduledJob::Succeeded(ScheduledJob {
                record,
                state: PhantomData,
            }),
            JobStatus::Failed => DecodedScheduledJob::Failed(ScheduledJob {
                record,
                state: PhantomData,
            }),
            JobStatus::Dead => DecodedScheduledJob::Dead(ScheduledJob {
                record,
                state: PhantomData,
            }),
            JobStatus::Cancelled => DecodedScheduledJob::Cancelled(ScheduledJob {
                record,
                state: PhantomData,
            }),
        })
    }
}
// ANCHOR_END: scheduled_job_row_boundary

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use serde_json::json;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn row(status: &str) -> DbScheduledJobRow {
        let now = now();
        DbScheduledJobRow {
            id: Uuid::new_v4(),
            task_name: "incident_triage".to_string(),
            status: status.to_string(),
            payload: json!({ "instruction": "triage incident" }),
            attempts: 0,
            max_attempts: 3,
            next_run_at: now,
            locked_by: None,
            locked_until: None,
            last_error: None,
            idempotency_key: Some("tenant-a:incident-1".to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    fn pending_job() -> ScheduledJob<ScheduledPending> {
        match DecodedScheduledJob::try_from(row("pending")).expect("valid pending row") {
            DecodedScheduledJob::Pending(job) => job,
            other => panic!("expected pending job, got {other:?}"),
        }
    }

    #[test]
    fn pending_job_can_be_claimed_with_a_lease() {
        let claimed_at = now();
        let locked_until = claimed_at + Duration::minutes(5);
        let running = pending_job()
            .claim(
                WorkerId::new("worker-a").expect("valid worker"),
                locked_until,
                claimed_at,
            )
            .expect("claim should succeed");

        assert_eq!(running.attempts().get(), 1);
        assert_eq!(running.locked_by().map(WorkerId::as_str), Some("worker-a"));
        assert_eq!(running.locked_until(), Some(locked_until));
    }

    #[test]
    fn running_job_can_complete_and_clear_lease() {
        let claimed_at = now();
        let running = pending_job()
            .claim(
                WorkerId::new("worker-a").expect("valid worker"),
                claimed_at + Duration::minutes(5),
                claimed_at,
            )
            .expect("claim should succeed");
        let succeeded = running.mark_succeeded(claimed_at + Duration::seconds(10));

        assert!(succeeded.locked_by().is_none());
        assert!(succeeded.locked_until().is_none());
        assert!(succeeded.last_error().is_none());
    }

    #[test]
    fn running_failure_schedules_retry_until_attempts_are_exhausted() {
        let claimed_at = now();
        let running = pending_job()
            .claim(
                WorkerId::new("worker-a").expect("valid worker"),
                claimed_at + Duration::minutes(5),
                claimed_at,
            )
            .expect("claim should succeed");
        let retry_at = claimed_at + Duration::minutes(1);
        let transition = running.fail(
            FailureMessage::new("provider timeout").expect("valid failure"),
            retry_at,
            claimed_at + Duration::seconds(2),
        );

        let ScheduledFailureTransition::Retry(retry) = transition else {
            panic!("first failed attempt should retry");
        };
        assert_eq!(retry.next_run_at(), retry_at);
        assert!(retry.locked_by().is_none());
        assert!(retry.last_error().is_some());
    }

    #[test]
    fn exhausted_running_failure_becomes_dead() {
        let mut exhausted = row("running");
        exhausted.attempts = 3;
        exhausted.max_attempts = 3;
        exhausted.locked_by = Some("worker-a".to_string());
        exhausted.locked_until = Some(now() + Duration::minutes(5));
        let running = match DecodedScheduledJob::try_from(exhausted).expect("valid running row") {
            DecodedScheduledJob::Running(job) => job,
            other => panic!("expected running job, got {other:?}"),
        };

        let transition = running.fail(
            FailureMessage::new("max attempts reached").expect("valid failure"),
            now() + Duration::minutes(1),
            now(),
        );

        assert!(matches!(transition, ScheduledFailureTransition::Dead(_)));
    }

    #[test]
    fn row_conversion_accepts_running_row_with_lease() {
        let mut row = row("running");
        row.locked_by = Some("worker-a".to_string());
        row.locked_until = Some(now() + Duration::minutes(5));

        let decoded = DecodedScheduledJob::try_from(row).expect("valid running row");

        assert_eq!(decoded.status(), JobStatus::Running);
    }

    #[test]
    fn row_conversion_rejects_running_without_lease() {
        let error = DecodedScheduledJob::try_from(row("running"))
            .expect_err("running row must include lease evidence");

        assert_eq!(error, ScheduledJobError::MissingRunningLease);
    }

    #[test]
    fn row_conversion_rejects_non_running_with_lease() {
        let mut row = row("pending");
        row.locked_by = Some("worker-a".to_string());
        row.locked_until = Some(now() + Duration::minutes(5));
        let error = DecodedScheduledJob::try_from(row).expect_err("pending row must not be leased");

        assert!(matches!(error, ScheduledJobError::UnexpectedLease { .. }));
    }

    #[test]
    fn row_conversion_rejects_negative_attempts() {
        let mut row = row("pending");
        row.attempts = -1;
        let error = DecodedScheduledJob::try_from(row).expect_err("attempts must be non-negative");

        assert!(matches!(error, ScheduledJobError::NegativeNumber { .. }));
    }

    #[test]
    fn row_conversion_rejects_unknown_status() {
        let error =
            DecodedScheduledJob::try_from(row("paused")).expect_err("unknown status must fail");

        assert!(matches!(error, ScheduledJobError::Domain(_)));
    }

    #[test]
    fn row_conversion_rejects_non_object_payload() {
        let mut row = row("pending");
        row.payload = json!("not an object");
        let error = DecodedScheduledJob::try_from(row).expect_err("payload must be object");

        assert_eq!(error, ScheduledJobError::PayloadMustBeObject);
    }
}
