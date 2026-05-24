//! Typed lifecycle for the `background_jobs` table.
//!
//! A scheduled job says when work should be attempted. A background job says
//! where the durable workflow currently is: queued, leased, executing an agent,
//! waiting for a human, waiting for retry, or terminal.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AttemptCount, DomainError, FailureMessage, JobKind, MaxAttempts};
use crate::scheduled_job::ScheduledJobId;
use crate::timeouts::{ExecutionDeadline, TimeoutAction, TimeoutError, TimeoutPolicyName};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BackgroundJobError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} cannot be negative, got {value}")]
    NegativeNumber { field: &'static str, value: i64 },
    #[error("{field} is too large, got {value}")]
    NumberOutOfRange { field: &'static str, value: i64 },
    #[error("{field} must be positive, got {value}")]
    NonPositiveNumber { field: &'static str, value: i64 },
    #[error("unknown workflow state: {value}")]
    UnknownWorkflowState { value: UnknownWorkflowState },
    #[error("unknown retry state: {value}")]
    UnknownRetryState { value: UnknownRetryState },
    #[error("attempts cannot exceed max_attempts")]
    AttemptsExceedMaxAttempts,
    #[error("updated_at cannot be before created_at")]
    UpdatedBeforeCreated,
    #[error("active workflow state {state:?} requires execution_deadline_at")]
    MissingExecutionDeadline { state: WorkflowState },
    #[error("inactive workflow state {state:?} must not carry execution_deadline_at")]
    UnexpectedExecutionDeadline { state: WorkflowState },
    #[error("waiting_for_retry retry state requires next_retry_at")]
    MissingNextRetryAt,
    #[error("retry state {state:?} must not carry next_retry_at")]
    UnexpectedNextRetryAt { state: RetryState },
    #[error("workflow state {workflow_state:?} is incompatible with retry state {retry_state:?}")]
    IncompatibleWorkflowAndRetryState {
        workflow_state: WorkflowState,
        retry_state: RetryState,
    },
    #[error("state {state:?} requires last_error and last_failure_class")]
    MissingFailureEvidence { state: WorkflowState },
    #[error("state {state:?} must not carry failure evidence")]
    UnexpectedFailureEvidence { state: WorkflowState },
    #[error("retry state exhausted requires attempts to be greater than or equal to max_attempts")]
    ExhaustedBeforeMaxAttempts,
    #[error("retry state waiting_for_retry requires attempts to be less than max_attempts")]
    WaitingRetryAfterMaxAttempts,
    #[error("timeout validation failed: {0}")]
    Timeout(#[from] TimeoutError),
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_background_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, BackgroundJobError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(BackgroundJobError::EmptyText { field });
    }
    Ok(value)
}

fn non_negative_u32(value: i64, field: &'static str) -> Result<u32, BackgroundJobError> {
    if value < 0 {
        return Err(BackgroundJobError::NegativeNumber { field, value });
    }

    u32::try_from(value).map_err(|_| BackgroundJobError::NumberOutOfRange { field, value })
}

fn positive_u32(value: i64, field: &'static str) -> Result<u32, BackgroundJobError> {
    if value <= 0 {
        return Err(BackgroundJobError::NonPositiveNumber { field, value });
    }

    non_negative_u32(value, field)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BackgroundJobId(Uuid);

impl BackgroundJobId {
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

impl Default for BackgroundJobId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownWorkflowState(String);

impl UnknownWorkflowState {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownWorkflowState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownRetryState(String);

impl UnknownRetryState {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownRetryState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowState {
    Queued,
    Leased,
    ExecutingAgent,
    WaitingForHuman,
    WaitingForRetry,
    Completed,
    Failed,
    Cancelled,
}

impl WorkflowState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Leased => "leased",
            Self::ExecutingAgent => "executing_agent",
            Self::WaitingForHuman => "waiting_for_human",
            Self::WaitingForRetry => "waiting_for_retry",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn is_active(self) -> bool {
        matches!(
            self,
            Self::Leased | Self::ExecutingAgent | Self::WaitingForHuman
        )
    }
}

impl TryFrom<&str> for WorkflowState {
    type Error = BackgroundJobError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "queued" => Ok(Self::Queued),
            "leased" => Ok(Self::Leased),
            "executing_agent" => Ok(Self::ExecutingAgent),
            "waiting_for_human" => Ok(Self::WaitingForHuman),
            "waiting_for_retry" => Ok(Self::WaitingForRetry),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            value => Err(BackgroundJobError::UnknownWorkflowState {
                value: UnknownWorkflowState::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryState {
    NotStarted,
    Retryable,
    WaitingForRetry,
    Exhausted,
    PermanentFailure,
    NotApplicable,
}

impl RetryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::Retryable => "retryable",
            Self::WaitingForRetry => "waiting_for_retry",
            Self::Exhausted => "exhausted",
            Self::PermanentFailure => "permanent_failure",
            Self::NotApplicable => "not_applicable",
        }
    }
}

impl TryFrom<&str> for RetryState {
    type Error = BackgroundJobError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "not_started" => Ok(Self::NotStarted),
            "retryable" => Ok(Self::Retryable),
            "waiting_for_retry" => Ok(Self::WaitingForRetry),
            "exhausted" => Ok(Self::Exhausted),
            "permanent_failure" => Ok(Self::PermanentFailure),
            "not_applicable" => Ok(Self::NotApplicable),
            value => Err(BackgroundJobError::UnknownRetryState {
                value: UnknownRetryState::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureClass(String);

impl FailureClass {
    pub fn new(value: impl Into<String>) -> Result<Self, BackgroundJobError> {
        Ok(Self(non_empty_background_text(value, "failure_class")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BackgroundJobRecord {
    id: BackgroundJobId,
    scheduled_job_id: ScheduledJobId,
    job_kind: JobKind,
    retry_state: RetryState,
    attempts: AttemptCount,
    max_attempts: MaxAttempts,
    next_retry_at: Option<DateTime<Utc>>,
    execution_deadline_at: Option<ExecutionDeadline>,
    timeout_policy_name: TimeoutPolicyName,
    timeout_action: TimeoutAction,
    last_failure_class: Option<FailureClass>,
    last_error: Option<FailureMessage>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// ANCHOR: background_job_typestate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundQueued;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundLeased;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundExecutingAgent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundWaitingForHuman;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundWaitingForRetry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundCompleted;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundFailed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundCancelled;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundJob<State> {
    record: BackgroundJobRecord,
    state: PhantomData<State>,
}

impl BackgroundJob<BackgroundQueued> {
    pub fn lease(
        mut self,
        execution_deadline: ExecutionDeadline,
        updated_at: DateTime<Utc>,
    ) -> BackgroundJob<BackgroundLeased> {
        self.record.execution_deadline_at = Some(execution_deadline);
        self.record.updated_at = updated_at;

        BackgroundJob {
            record: self.record,
            state: PhantomData,
        }
    }
}

impl BackgroundJob<BackgroundLeased> {
    pub fn start_agent(
        mut self,
        updated_at: DateTime<Utc>,
    ) -> BackgroundJob<BackgroundExecutingAgent> {
        self.record.updated_at = updated_at;

        BackgroundJob {
            record: self.record,
            state: PhantomData,
        }
    }
}

impl BackgroundJob<BackgroundExecutingAgent> {
    pub fn wait_for_human(
        mut self,
        updated_at: DateTime<Utc>,
    ) -> BackgroundJob<BackgroundWaitingForHuman> {
        self.record.updated_at = updated_at;

        BackgroundJob {
            record: self.record,
            state: PhantomData,
        }
    }

    pub fn complete(mut self, updated_at: DateTime<Utc>) -> BackgroundJob<BackgroundCompleted> {
        self.record.retry_state = RetryState::NotApplicable;
        self.record.execution_deadline_at = None;
        self.record.next_retry_at = None;
        self.record.last_failure_class = None;
        self.record.last_error = None;
        self.record.updated_at = updated_at;

        BackgroundJob {
            record: self.record,
            state: PhantomData,
        }
    }

    pub fn fail_for_retry(
        mut self,
        failure_class: FailureClass,
        error: FailureMessage,
        next_retry_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<BackgroundJob<BackgroundWaitingForRetry>, BackgroundJobError> {
        if self.record.attempts.get() >= self.record.max_attempts.get() {
            return Err(BackgroundJobError::WaitingRetryAfterMaxAttempts);
        }

        self.record.retry_state = RetryState::WaitingForRetry;
        self.record.execution_deadline_at = None;
        self.record.next_retry_at = Some(next_retry_at);
        self.record.last_failure_class = Some(failure_class);
        self.record.last_error = Some(error);
        self.record.updated_at = updated_at;

        Ok(BackgroundJob {
            record: self.record,
            state: PhantomData,
        })
    }

    pub fn fail_permanently(
        mut self,
        failure_class: FailureClass,
        error: FailureMessage,
        updated_at: DateTime<Utc>,
    ) -> BackgroundJob<BackgroundFailed> {
        self.record.retry_state = RetryState::PermanentFailure;
        self.record.execution_deadline_at = None;
        self.record.next_retry_at = None;
        self.record.last_failure_class = Some(failure_class);
        self.record.last_error = Some(error);
        self.record.updated_at = updated_at;

        BackgroundJob {
            record: self.record,
            state: PhantomData,
        }
    }

    pub fn cancel(
        mut self,
        reason: FailureMessage,
        updated_at: DateTime<Utc>,
    ) -> BackgroundJob<BackgroundCancelled> {
        self.record.retry_state = RetryState::NotApplicable;
        self.record.execution_deadline_at = None;
        self.record.next_retry_at = None;
        self.record.last_error = Some(reason);
        self.record.updated_at = updated_at;

        BackgroundJob {
            record: self.record,
            state: PhantomData,
        }
    }
}

impl BackgroundJob<BackgroundWaitingForHuman> {
    pub fn resume(mut self, updated_at: DateTime<Utc>) -> BackgroundJob<BackgroundExecutingAgent> {
        self.record.updated_at = updated_at;

        BackgroundJob {
            record: self.record,
            state: PhantomData,
        }
    }
}

impl BackgroundJob<BackgroundWaitingForRetry> {
    pub fn requeue(mut self, updated_at: DateTime<Utc>) -> BackgroundJob<BackgroundQueued> {
        self.record.retry_state = RetryState::Retryable;
        self.record.next_retry_at = None;
        self.record.updated_at = updated_at;

        BackgroundJob {
            record: self.record,
            state: PhantomData,
        }
    }
}
// ANCHOR_END: background_job_typestate

impl<State> BackgroundJob<State> {
    pub fn id(&self) -> BackgroundJobId {
        self.record.id
    }

    pub fn scheduled_job_id(&self) -> ScheduledJobId {
        self.record.scheduled_job_id
    }

    pub fn job_kind(&self) -> &JobKind {
        &self.record.job_kind
    }

    pub fn retry_state(&self) -> RetryState {
        self.record.retry_state
    }

    pub fn attempts(&self) -> AttemptCount {
        self.record.attempts
    }

    pub fn max_attempts(&self) -> MaxAttempts {
        self.record.max_attempts
    }

    pub fn next_retry_at(&self) -> Option<DateTime<Utc>> {
        self.record.next_retry_at
    }

    pub fn execution_deadline_at(&self) -> Option<ExecutionDeadline> {
        self.record.execution_deadline_at
    }

    pub fn timeout_policy_name(&self) -> &TimeoutPolicyName {
        &self.record.timeout_policy_name
    }

    pub fn timeout_action(&self) -> TimeoutAction {
        self.record.timeout_action
    }

    pub fn last_failure_class(&self) -> Option<&FailureClass> {
        self.record.last_failure_class.as_ref()
    }

    pub fn last_error(&self) -> Option<&FailureMessage> {
        self.record.last_error.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.record.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.record.updated_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedBackgroundJob {
    Queued(BackgroundJob<BackgroundQueued>),
    Leased(BackgroundJob<BackgroundLeased>),
    ExecutingAgent(BackgroundJob<BackgroundExecutingAgent>),
    WaitingForHuman(BackgroundJob<BackgroundWaitingForHuman>),
    WaitingForRetry(BackgroundJob<BackgroundWaitingForRetry>),
    Completed(BackgroundJob<BackgroundCompleted>),
    Failed(BackgroundJob<BackgroundFailed>),
    Cancelled(BackgroundJob<BackgroundCancelled>),
}

impl DecodedBackgroundJob {
    pub fn workflow_state(&self) -> WorkflowState {
        match self {
            Self::Queued(_) => WorkflowState::Queued,
            Self::Leased(_) => WorkflowState::Leased,
            Self::ExecutingAgent(_) => WorkflowState::ExecutingAgent,
            Self::WaitingForHuman(_) => WorkflowState::WaitingForHuman,
            Self::WaitingForRetry(_) => WorkflowState::WaitingForRetry,
            Self::Completed(_) => WorkflowState::Completed,
            Self::Failed(_) => WorkflowState::Failed,
            Self::Cancelled(_) => WorkflowState::Cancelled,
        }
    }
}

// ANCHOR: background_job_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbBackgroundJobRow {
    pub id: Uuid,
    pub scheduled_job_id: Uuid,
    pub job_kind: String,
    pub workflow_state: String,
    pub retry_state: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub execution_deadline_at: Option<DateTime<Utc>>,
    pub timeout_policy_name: String,
    pub timeout_action: String,
    pub last_failure_class: Option<String>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<DbBackgroundJobRow> for DecodedBackgroundJob {
    type Error = BackgroundJobError;

    fn try_from(row: DbBackgroundJobRow) -> Result<Self, Self::Error> {
        let workflow_state = WorkflowState::try_from(row.workflow_state.as_str())?;
        let retry_state = RetryState::try_from(row.retry_state.as_str())?;
        let attempts = AttemptCount::try_from_u32(non_negative_u32(row.attempts, "attempts")?)?;
        let max_attempts =
            MaxAttempts::try_from_u32(positive_u32(row.max_attempts, "max_attempts")?)?;

        let record = BackgroundJobRecord {
            id: BackgroundJobId::from_uuid(row.id),
            scheduled_job_id: ScheduledJobId::from_uuid(row.scheduled_job_id),
            job_kind: JobKind::new(row.job_kind)?,
            retry_state,
            attempts,
            max_attempts,
            next_retry_at: row.next_retry_at,
            execution_deadline_at: row
                .execution_deadline_at
                .map(|deadline_at| ExecutionDeadline::new(row.created_at, deadline_at))
                .transpose()?,
            timeout_policy_name: TimeoutPolicyName::new(row.timeout_policy_name)?,
            timeout_action: TimeoutAction::try_from(row.timeout_action.as_str())?,
            last_failure_class: row.last_failure_class.map(FailureClass::new).transpose()?,
            last_error: row.last_error.map(FailureMessage::new).transpose()?,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };

        validate_background_job_record(workflow_state, &record)?;

        Ok(match workflow_state {
            WorkflowState::Queued => DecodedBackgroundJob::Queued(BackgroundJob {
                record,
                state: PhantomData,
            }),
            WorkflowState::Leased => DecodedBackgroundJob::Leased(BackgroundJob {
                record,
                state: PhantomData,
            }),
            WorkflowState::ExecutingAgent => DecodedBackgroundJob::ExecutingAgent(BackgroundJob {
                record,
                state: PhantomData,
            }),
            WorkflowState::WaitingForHuman => {
                DecodedBackgroundJob::WaitingForHuman(BackgroundJob {
                    record,
                    state: PhantomData,
                })
            }
            WorkflowState::WaitingForRetry => {
                DecodedBackgroundJob::WaitingForRetry(BackgroundJob {
                    record,
                    state: PhantomData,
                })
            }
            WorkflowState::Completed => DecodedBackgroundJob::Completed(BackgroundJob {
                record,
                state: PhantomData,
            }),
            WorkflowState::Failed => DecodedBackgroundJob::Failed(BackgroundJob {
                record,
                state: PhantomData,
            }),
            WorkflowState::Cancelled => DecodedBackgroundJob::Cancelled(BackgroundJob {
                record,
                state: PhantomData,
            }),
        })
    }
}
// ANCHOR_END: background_job_row_boundary

fn validate_background_job_record(
    workflow_state: WorkflowState,
    record: &BackgroundJobRecord,
) -> Result<(), BackgroundJobError> {
    if record.updated_at < record.created_at {
        return Err(BackgroundJobError::UpdatedBeforeCreated);
    }

    if record.attempts.get() > record.max_attempts.get() {
        return Err(BackgroundJobError::AttemptsExceedMaxAttempts);
    }

    validate_workflow_retry_pair(workflow_state, record.retry_state)?;
    validate_execution_deadline(workflow_state, record)?;
    validate_retry_timing(record)?;
    validate_failure_evidence(workflow_state, record)?;
    validate_attempt_budget(record)
}

fn validate_workflow_retry_pair(
    workflow_state: WorkflowState,
    retry_state: RetryState,
) -> Result<(), BackgroundJobError> {
    let compatible = match workflow_state {
        WorkflowState::Queued
        | WorkflowState::Leased
        | WorkflowState::ExecutingAgent
        | WorkflowState::WaitingForHuman => {
            matches!(retry_state, RetryState::NotStarted | RetryState::Retryable)
        }
        WorkflowState::WaitingForRetry => retry_state == RetryState::WaitingForRetry,
        WorkflowState::Completed | WorkflowState::Cancelled => {
            retry_state == RetryState::NotApplicable
        }
        WorkflowState::Failed => matches!(
            retry_state,
            RetryState::Exhausted | RetryState::PermanentFailure
        ),
    };

    if compatible {
        Ok(())
    } else {
        Err(BackgroundJobError::IncompatibleWorkflowAndRetryState {
            workflow_state,
            retry_state,
        })
    }
}

fn validate_execution_deadline(
    workflow_state: WorkflowState,
    record: &BackgroundJobRecord,
) -> Result<(), BackgroundJobError> {
    match (
        workflow_state.is_active(),
        record.execution_deadline_at.is_some(),
    ) {
        (true, true) | (false, false) => Ok(()),
        (true, false) => Err(BackgroundJobError::MissingExecutionDeadline {
            state: workflow_state,
        }),
        (false, true) => Err(BackgroundJobError::UnexpectedExecutionDeadline {
            state: workflow_state,
        }),
    }
}

fn validate_retry_timing(record: &BackgroundJobRecord) -> Result<(), BackgroundJobError> {
    match (record.retry_state, record.next_retry_at.is_some()) {
        (RetryState::WaitingForRetry, true) => Ok(()),
        (RetryState::WaitingForRetry, false) => Err(BackgroundJobError::MissingNextRetryAt),
        (_, false) => Ok(()),
        (state, true) => Err(BackgroundJobError::UnexpectedNextRetryAt { state }),
    }
}

fn validate_failure_evidence(
    workflow_state: WorkflowState,
    record: &BackgroundJobRecord,
) -> Result<(), BackgroundJobError> {
    let has_failure_evidence = record.last_failure_class.is_some() || record.last_error.is_some();
    let has_complete_failure_evidence =
        record.last_failure_class.is_some() && record.last_error.is_some();

    if matches!(
        record.retry_state,
        RetryState::Retryable
            | RetryState::WaitingForRetry
            | RetryState::Exhausted
            | RetryState::PermanentFailure
    ) && !has_complete_failure_evidence
    {
        return Err(BackgroundJobError::MissingFailureEvidence {
            state: workflow_state,
        });
    }

    if (record.retry_state == RetryState::NotStarted || workflow_state == WorkflowState::Completed)
        && has_failure_evidence
    {
        return Err(BackgroundJobError::UnexpectedFailureEvidence {
            state: workflow_state,
        });
    }

    Ok(())
}

fn validate_attempt_budget(record: &BackgroundJobRecord) -> Result<(), BackgroundJobError> {
    if record.retry_state == RetryState::WaitingForRetry
        && record.attempts.get() >= record.max_attempts.get()
    {
        return Err(BackgroundJobError::WaitingRetryAfterMaxAttempts);
    }

    if record.retry_state == RetryState::Exhausted
        && record.attempts.get() < record.max_attempts.get()
    {
        return Err(BackgroundJobError::ExhaustedBeforeMaxAttempts);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn deadline_from(started_at: DateTime<Utc>) -> ExecutionDeadline {
        ExecutionDeadline::new(started_at, started_at + Duration::minutes(15))
            .expect("deadline after start")
    }

    fn row(workflow_state: &str, retry_state: &str) -> DbBackgroundJobRow {
        let now = now();
        DbBackgroundJobRow {
            id: Uuid::new_v4(),
            scheduled_job_id: Uuid::new_v4(),
            job_kind: "incident_triage".to_string(),
            workflow_state: workflow_state.to_string(),
            retry_state: retry_state.to_string(),
            attempts: 0,
            max_attempts: 3,
            next_retry_at: None,
            execution_deadline_at: None,
            timeout_policy_name: "standard-agent:v1".to_string(),
            timeout_action: "schedule_retry".to_string(),
            last_failure_class: None,
            last_error: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn queued_job() -> BackgroundJob<BackgroundQueued> {
        match DecodedBackgroundJob::try_from(row("queued", "not_started"))
            .expect("valid queued row")
        {
            DecodedBackgroundJob::Queued(job) => job,
            other => panic!("expected queued job, got {other:?}"),
        }
    }

    fn active_row(workflow_state: &str) -> DbBackgroundJobRow {
        let mut row = row(workflow_state, "not_started");
        row.execution_deadline_at = Some(now() + Duration::minutes(15));
        row
    }

    fn failed_retry_row() -> DbBackgroundJobRow {
        let mut row = row("waiting_for_retry", "waiting_for_retry");
        row.attempts = 1;
        row.next_retry_at = Some(now() + Duration::minutes(2));
        row.last_failure_class = Some("provider_timeout".to_string());
        row.last_error = Some("deepseek timeout".to_string());
        row
    }

    #[test]
    fn queued_job_can_be_leased_and_started() {
        let started_at = now();
        let leased = queued_job().lease(deadline_from(started_at), started_at);
        let executing = leased.start_agent(now());

        assert!(executing.execution_deadline_at().is_some());
        assert_eq!(executing.retry_state(), RetryState::NotStarted);
    }

    #[test]
    fn executing_job_can_wait_for_human_and_resume() {
        let executing = queued_job()
            .lease(deadline_from(now()), now())
            .start_agent(now());
        let waiting = executing.wait_for_human(now());
        let resumed = waiting.resume(now());

        assert!(resumed.execution_deadline_at().is_some());
    }

    #[test]
    fn executing_job_can_schedule_retry_with_failure_evidence() {
        let executing = queued_job()
            .lease(deadline_from(now()), now())
            .start_agent(now());
        let retry = executing
            .fail_for_retry(
                FailureClass::new("provider_timeout").expect("valid class"),
                FailureMessage::new("deepseek timeout").expect("valid failure"),
                now() + Duration::minutes(2),
                now(),
            )
            .expect("attempts remain");

        assert_eq!(retry.retry_state(), RetryState::WaitingForRetry);
        assert!(retry.next_retry_at().is_some());
        assert!(retry.last_failure_class().is_some());
        assert!(retry.last_error().is_some());
    }

    #[test]
    fn waiting_retry_can_requeue_without_losing_failure_history() {
        let retry =
            match DecodedBackgroundJob::try_from(failed_retry_row()).expect("valid retry row") {
                DecodedBackgroundJob::WaitingForRetry(job) => job,
                other => panic!("expected waiting retry, got {other:?}"),
            };

        let queued = retry.requeue(now());

        assert_eq!(queued.retry_state(), RetryState::Retryable);
        assert!(queued.next_retry_at().is_none());
        assert!(queued.last_error().is_some());
    }

    #[test]
    fn executing_job_can_complete_and_clear_failure_evidence() {
        let executing = queued_job()
            .lease(deadline_from(now()), now())
            .start_agent(now());
        let completed = executing.complete(now());

        assert_eq!(completed.retry_state(), RetryState::NotApplicable);
        assert!(completed.execution_deadline_at().is_none());
        assert!(completed.last_error().is_none());
    }

    #[test]
    fn row_conversion_accepts_active_states_with_deadlines() {
        for workflow_state in ["leased", "executing_agent", "waiting_for_human"] {
            let decoded = DecodedBackgroundJob::try_from(active_row(workflow_state))
                .expect("active row should decode");

            assert_eq!(decoded.workflow_state().as_str(), workflow_state);
        }
    }

    #[test]
    fn row_conversion_accepts_waiting_for_retry_with_failure_evidence() {
        let decoded =
            DecodedBackgroundJob::try_from(failed_retry_row()).expect("retry row should decode");

        assert_eq!(decoded.workflow_state(), WorkflowState::WaitingForRetry);
    }

    #[test]
    fn row_conversion_accepts_failed_exhausted_with_failure_evidence() {
        let mut row = row("failed", "exhausted");
        row.attempts = 3;
        row.max_attempts = 3;
        row.last_failure_class = Some("provider_timeout".to_string());
        row.last_error = Some("deepseek timeout".to_string());

        let decoded = DecodedBackgroundJob::try_from(row).expect("failed row should decode");

        assert_eq!(decoded.workflow_state(), WorkflowState::Failed);
    }

    #[test]
    fn row_conversion_rejects_unknown_workflow_state() {
        let error = DecodedBackgroundJob::try_from(row("paused", "not_started"))
            .expect_err("unknown workflow state must fail");

        assert!(matches!(
            error,
            BackgroundJobError::UnknownWorkflowState { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_unknown_retry_state() {
        let error = DecodedBackgroundJob::try_from(row("queued", "maybe_retry"))
            .expect_err("unknown retry state must fail");

        assert!(matches!(
            error,
            BackgroundJobError::UnknownRetryState { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_active_state_without_deadline() {
        let error = DecodedBackgroundJob::try_from(row("executing_agent", "not_started"))
            .expect_err("active row needs deadline");

        assert_eq!(
            error,
            BackgroundJobError::MissingExecutionDeadline {
                state: WorkflowState::ExecutingAgent
            }
        );
    }

    #[test]
    fn row_conversion_rejects_queued_state_with_deadline() {
        let mut row = row("queued", "not_started");
        row.execution_deadline_at = Some(now() + Duration::minutes(15));
        let error = DecodedBackgroundJob::try_from(row).expect_err("queued row cannot be leased");

        assert_eq!(
            error,
            BackgroundJobError::UnexpectedExecutionDeadline {
                state: WorkflowState::Queued
            }
        );
    }

    #[test]
    fn row_conversion_rejects_mismatched_workflow_and_retry_state() {
        let error = DecodedBackgroundJob::try_from(row("completed", "retryable"))
            .expect_err("completed row cannot be retryable");

        assert!(matches!(
            error,
            BackgroundJobError::IncompatibleWorkflowAndRetryState { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_waiting_for_retry_without_next_retry_at() {
        let mut row = failed_retry_row();
        row.next_retry_at = None;
        let error =
            DecodedBackgroundJob::try_from(row).expect_err("retry row needs next retry time");

        assert_eq!(error, BackgroundJobError::MissingNextRetryAt);
    }

    #[test]
    fn row_conversion_rejects_non_retry_state_with_next_retry_at() {
        let mut row = row("queued", "not_started");
        row.next_retry_at = Some(now() + Duration::minutes(2));
        let error =
            DecodedBackgroundJob::try_from(row).expect_err("queued row cannot be scheduled retry");

        assert_eq!(
            error,
            BackgroundJobError::UnexpectedNextRetryAt {
                state: RetryState::NotStarted
            }
        );
    }

    #[test]
    fn row_conversion_rejects_waiting_for_retry_without_failure_evidence() {
        let mut row = failed_retry_row();
        row.last_error = None;
        let error =
            DecodedBackgroundJob::try_from(row).expect_err("retry row needs failure evidence");

        assert_eq!(
            error,
            BackgroundJobError::MissingFailureEvidence {
                state: WorkflowState::WaitingForRetry
            }
        );
    }

    #[test]
    fn row_conversion_rejects_completed_with_failure_evidence() {
        let mut row = row("completed", "not_applicable");
        row.last_error = Some("old failure".to_string());
        let error = DecodedBackgroundJob::try_from(row)
            .expect_err("completed row should clear failure evidence");

        assert_eq!(
            error,
            BackgroundJobError::UnexpectedFailureEvidence {
                state: WorkflowState::Completed
            }
        );
    }

    #[test]
    fn row_conversion_rejects_retryable_without_failure_evidence() {
        let error = DecodedBackgroundJob::try_from(row("queued", "retryable"))
            .expect_err("retryable row needs previous failure evidence");

        assert_eq!(
            error,
            BackgroundJobError::MissingFailureEvidence {
                state: WorkflowState::Queued
            }
        );
    }

    #[test]
    fn row_conversion_rejects_negative_attempts() {
        let mut row = row("queued", "not_started");
        row.attempts = -1;
        let error = DecodedBackgroundJob::try_from(row).expect_err("attempts must be positive");

        assert!(matches!(error, BackgroundJobError::NegativeNumber { .. }));
    }

    #[test]
    fn row_conversion_rejects_attempts_over_max_attempts() {
        let mut row = row("queued", "not_started");
        row.attempts = 4;
        row.max_attempts = 3;
        let error = DecodedBackgroundJob::try_from(row).expect_err("attempts cannot exceed max");

        assert_eq!(error, BackgroundJobError::AttemptsExceedMaxAttempts);
    }

    #[test]
    fn row_conversion_rejects_exhausted_before_max_attempts() {
        let mut row = row("failed", "exhausted");
        row.attempts = 1;
        row.max_attempts = 3;
        row.last_failure_class = Some("provider_timeout".to_string());
        row.last_error = Some("deepseek timeout".to_string());
        let error =
            DecodedBackgroundJob::try_from(row).expect_err("exhausted state needs max attempts");

        assert_eq!(error, BackgroundJobError::ExhaustedBeforeMaxAttempts);
    }
}
