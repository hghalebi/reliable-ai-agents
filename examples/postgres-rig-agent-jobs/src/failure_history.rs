//! Typed failure-history records.
//!
//! `last_error` is useful for the current state of a job. It is not enough for
//! incident review. This module models the append-only failure history that
//! lets operators see repeated attempts, retry decisions, failure classes, and
//! trace context over time.

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::agent_step::AgentStepId;
use crate::audit::{AuditError, SpanId, TraceContext, TraceId};
use crate::background_job::{
    BackgroundJobError, BackgroundJobId, FailureClass, RetryState, WorkflowState,
};
use crate::domain::{AgentRunId, AttemptCount, DomainError, FailureMessage, MaxAttempts};
use crate::scheduled_job::ScheduledJobId;
use crate::tool_call::ToolCallId;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FailureHistoryError {
    #[error("{field} cannot be negative, got {value}")]
    NegativeNumber { field: &'static str, value: i64 },
    #[error("{field} is too large, got {value}")]
    NumberOutOfRange { field: &'static str, value: i64 },
    #[error("{field} must be positive, got {value}")]
    NonPositiveNumber { field: &'static str, value: i64 },
    #[error("unknown failure source: {value}")]
    UnknownSource { value: UnknownFailureSource },
    #[error("unknown failure outcome: {value}")]
    UnknownOutcome { value: UnknownFailureOutcome },
    #[error("attempts cannot exceed max_attempts")]
    AttemptsExceedMaxAttempts,
    #[error("recorded_at cannot be before occurred_at")]
    RecordedBeforeOccurred,
    #[error("span_id cannot exist without trace_id")]
    SpanWithoutTrace,
    #[error("retry_scheduled outcome requires next_retry_at")]
    MissingNextRetryAt,
    #[error("next_retry_at must be after occurred_at")]
    NextRetryNotAfterFailure,
    #[error("non-retry outcome {outcome:?} must not carry next_retry_at")]
    UnexpectedNextRetryAt { outcome: FailureOutcome },
    #[error("dead_lettered outcome requires attempts to be greater than or equal to max_attempts")]
    DeadLetterBeforeMaxAttempts,
    #[error(
        "outcome {outcome:?} is incompatible with workflow state {workflow_state:?} and retry state {retry_state:?}"
    )]
    IncompatibleOutcomeState {
        outcome: FailureOutcome,
        workflow_state: WorkflowState,
        retry_state: RetryState,
    },
    #[error("audit validation failed: {0}")]
    Audit(#[from] AuditError),
    #[error("background job validation failed: {0}")]
    BackgroundJob(#[from] BackgroundJobError),
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_negative_u32(value: i64, field: &'static str) -> Result<u32, FailureHistoryError> {
    if value < 0 {
        return Err(FailureHistoryError::NegativeNumber { field, value });
    }

    u32::try_from(value).map_err(|_| FailureHistoryError::NumberOutOfRange { field, value })
}

fn positive_u32(value: i64, field: &'static str) -> Result<u32, FailureHistoryError> {
    if value <= 0 {
        return Err(FailureHistoryError::NonPositiveNumber { field, value });
    }

    non_negative_u32(value, field)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FailureHistoryId(i64);

impl FailureHistoryId {
    pub fn try_from_i64(value: i64) -> Result<Self, FailureHistoryError> {
        if value <= 0 {
            return Err(FailureHistoryError::NonPositiveNumber {
                field: "failure_history_id",
                value,
            });
        }

        Ok(Self(value))
    }

    pub fn get(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownFailureSource(String);

impl UnknownFailureSource {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownFailureSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownFailureOutcome(String);

impl UnknownFailureOutcome {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownFailureOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureSource {
    Worker,
    ModelProvider,
    ModelOutput,
    Tool,
    Policy,
    Sandbox,
    Timeout,
    Database,
}

impl FailureSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Worker => "worker",
            Self::ModelProvider => "model_provider",
            Self::ModelOutput => "model_output",
            Self::Tool => "tool",
            Self::Policy => "policy",
            Self::Sandbox => "sandbox",
            Self::Timeout => "timeout",
            Self::Database => "database",
        }
    }
}

impl TryFrom<&str> for FailureSource {
    type Error = FailureHistoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "worker" => Ok(Self::Worker),
            "model_provider" => Ok(Self::ModelProvider),
            "model_output" => Ok(Self::ModelOutput),
            "tool" => Ok(Self::Tool),
            "policy" => Ok(Self::Policy),
            "sandbox" => Ok(Self::Sandbox),
            "timeout" => Ok(Self::Timeout),
            "database" => Ok(Self::Database),
            value => Err(FailureHistoryError::UnknownSource {
                value: UnknownFailureSource::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureOutcome {
    RetryScheduled,
    DeadLettered,
    PermanentFailure,
    EscalatedToHuman,
    Cancelled,
}

impl FailureOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RetryScheduled => "retry_scheduled",
            Self::DeadLettered => "dead_lettered",
            Self::PermanentFailure => "permanent_failure",
            Self::EscalatedToHuman => "escalated_to_human",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for FailureOutcome {
    type Error = FailureHistoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "retry_scheduled" => Ok(Self::RetryScheduled),
            "dead_lettered" => Ok(Self::DeadLettered),
            "permanent_failure" => Ok(Self::PermanentFailure),
            "escalated_to_human" => Ok(Self::EscalatedToHuman),
            "cancelled" => Ok(Self::Cancelled),
            value => Err(FailureHistoryError::UnknownOutcome {
                value: UnknownFailureOutcome::new(value),
            }),
        }
    }
}

// ANCHOR: failure_history_record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureHistoryRecord {
    id: FailureHistoryId,
    scheduled_job_id: ScheduledJobId,
    background_job_id: Option<BackgroundJobId>,
    run_id: Option<AgentRunId>,
    step_id: Option<AgentStepId>,
    tool_call_id: Option<ToolCallId>,
    source: FailureSource,
    failure_class: FailureClass,
    failure_message: FailureMessage,
    workflow_state: WorkflowState,
    retry_state: RetryState,
    outcome: FailureOutcome,
    attempt: AttemptCount,
    max_attempts: MaxAttempts,
    next_retry_at: Option<DateTime<Utc>>,
    trace_context: Option<TraceContext>,
    occurred_at: DateTime<Utc>,
    recorded_at: DateTime<Utc>,
}
// ANCHOR_END: failure_history_record

impl FailureHistoryRecord {
    pub fn id(&self) -> FailureHistoryId {
        self.id
    }

    pub fn scheduled_job_id(&self) -> ScheduledJobId {
        self.scheduled_job_id
    }

    pub fn source(&self) -> FailureSource {
        self.source
    }

    pub fn failure_class(&self) -> &FailureClass {
        &self.failure_class
    }

    pub fn outcome(&self) -> FailureOutcome {
        self.outcome
    }

    pub fn attempt(&self) -> AttemptCount {
        self.attempt
    }

    pub fn max_attempts(&self) -> MaxAttempts {
        self.max_attempts
    }

    pub fn next_retry_at(&self) -> Option<DateTime<Utc>> {
        self.next_retry_at
    }

    pub fn trace_context(&self) -> Option<&TraceContext> {
        self.trace_context.as_ref()
    }
}

// ANCHOR: failure_history_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbFailureHistoryRow {
    pub id: i64,
    pub scheduled_job_id: Uuid,
    pub background_job_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub step_id: Option<Uuid>,
    pub tool_call_id: Option<Uuid>,
    pub failure_source: String,
    pub failure_class: String,
    pub failure_message: String,
    pub workflow_state: String,
    pub retry_state: String,
    pub failure_outcome: String,
    pub attempt: i64,
    pub max_attempts: i64,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
}

impl TryFrom<DbFailureHistoryRow> for FailureHistoryRecord {
    type Error = FailureHistoryError;

    fn try_from(row: DbFailureHistoryRow) -> Result<Self, Self::Error> {
        let attempt = AttemptCount::try_from_u32(non_negative_u32(row.attempt, "attempt")?)?;
        let max_attempts =
            MaxAttempts::try_from_u32(positive_u32(row.max_attempts, "max_attempts")?)?;
        let source = FailureSource::try_from(row.failure_source.as_str())?;
        let workflow_state = WorkflowState::try_from(row.workflow_state.as_str())?;
        let retry_state = RetryState::try_from(row.retry_state.as_str())?;
        let outcome = FailureOutcome::try_from(row.failure_outcome.as_str())?;
        let trace_context = decode_trace_context(row.trace_id, row.span_id)?;

        let record = Self {
            id: FailureHistoryId::try_from_i64(row.id)?,
            scheduled_job_id: ScheduledJobId::from_uuid(row.scheduled_job_id),
            background_job_id: row.background_job_id.map(BackgroundJobId::from_uuid),
            run_id: row.run_id.map(AgentRunId::from_uuid),
            step_id: row.step_id.map(AgentStepId::from_uuid),
            tool_call_id: row.tool_call_id.map(ToolCallId::from_uuid),
            source,
            failure_class: FailureClass::new(row.failure_class)?,
            failure_message: FailureMessage::new(row.failure_message)?,
            workflow_state,
            retry_state,
            outcome,
            attempt,
            max_attempts,
            next_retry_at: row.next_retry_at,
            trace_context,
            occurred_at: row.occurred_at,
            recorded_at: row.recorded_at,
        };

        validate_failure_history(&record)?;
        Ok(record)
    }
}
// ANCHOR_END: failure_history_row_boundary

fn decode_trace_context(
    trace_id: Option<String>,
    span_id: Option<String>,
) -> Result<Option<TraceContext>, FailureHistoryError> {
    match (trace_id, span_id) {
        (Some(trace_id), span_id) => Ok(Some(TraceContext::new(
            TraceId::new(trace_id)?,
            span_id.map(SpanId::new).transpose()?,
        ))),
        (None, None) => Ok(None),
        (None, Some(_)) => Err(FailureHistoryError::SpanWithoutTrace),
    }
}

fn validate_failure_history(record: &FailureHistoryRecord) -> Result<(), FailureHistoryError> {
    if record.recorded_at < record.occurred_at {
        return Err(FailureHistoryError::RecordedBeforeOccurred);
    }

    if record.attempt.get() > record.max_attempts.get() {
        return Err(FailureHistoryError::AttemptsExceedMaxAttempts);
    }

    validate_outcome_state(record)?;
    validate_retry_time(record)
}

fn validate_outcome_state(record: &FailureHistoryRecord) -> Result<(), FailureHistoryError> {
    let compatible = match record.outcome {
        FailureOutcome::RetryScheduled => {
            record.workflow_state == WorkflowState::WaitingForRetry
                && record.retry_state == RetryState::WaitingForRetry
        }
        FailureOutcome::DeadLettered => {
            record.workflow_state == WorkflowState::Failed
                && record.retry_state == RetryState::Exhausted
        }
        FailureOutcome::PermanentFailure => {
            record.workflow_state == WorkflowState::Failed
                && record.retry_state == RetryState::PermanentFailure
        }
        FailureOutcome::EscalatedToHuman => {
            record.workflow_state == WorkflowState::WaitingForHuman
                && matches!(
                    record.retry_state,
                    RetryState::NotStarted | RetryState::Retryable
                )
        }
        FailureOutcome::Cancelled => {
            record.workflow_state == WorkflowState::Cancelled
                && record.retry_state == RetryState::NotApplicable
        }
    };

    if !compatible {
        return Err(FailureHistoryError::IncompatibleOutcomeState {
            outcome: record.outcome,
            workflow_state: record.workflow_state,
            retry_state: record.retry_state,
        });
    }

    if record.outcome == FailureOutcome::DeadLettered
        && record.attempt.get() < record.max_attempts.get()
    {
        return Err(FailureHistoryError::DeadLetterBeforeMaxAttempts);
    }

    Ok(())
}

fn validate_retry_time(record: &FailureHistoryRecord) -> Result<(), FailureHistoryError> {
    match (record.outcome, record.next_retry_at) {
        (FailureOutcome::RetryScheduled, Some(next_retry_at)) => {
            if next_retry_at <= record.occurred_at {
                Err(FailureHistoryError::NextRetryNotAfterFailure)
            } else {
                Ok(())
            }
        }
        (FailureOutcome::RetryScheduled, None) => Err(FailureHistoryError::MissingNextRetryAt),
        (outcome, Some(_)) => Err(FailureHistoryError::UnexpectedNextRetryAt { outcome }),
        (_, None) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn row(outcome: &str) -> DbFailureHistoryRow {
        let occurred_at = now();
        DbFailureHistoryRow {
            id: 1,
            scheduled_job_id: Uuid::new_v4(),
            background_job_id: Some(Uuid::new_v4()),
            run_id: Some(Uuid::new_v4()),
            step_id: None,
            tool_call_id: None,
            failure_source: "model_provider".to_string(),
            failure_class: "provider_timeout".to_string(),
            failure_message: "deepseek timeout".to_string(),
            workflow_state: "waiting_for_retry".to_string(),
            retry_state: "waiting_for_retry".to_string(),
            failure_outcome: outcome.to_string(),
            attempt: 1,
            max_attempts: 3,
            next_retry_at: Some(occurred_at + Duration::minutes(2)),
            trace_id: Some("11111111111111111111111111111111".to_string()),
            span_id: Some("2222222222222222".to_string()),
            occurred_at,
            recorded_at: occurred_at + Duration::seconds(1),
        }
    }

    #[test]
    fn row_conversion_accepts_retry_scheduled_history() {
        let record =
            FailureHistoryRecord::try_from(row("retry_scheduled")).expect("valid failure row");

        assert_eq!(record.source(), FailureSource::ModelProvider);
        assert_eq!(record.outcome(), FailureOutcome::RetryScheduled);
        assert_eq!(record.failure_class().as_str(), "provider_timeout");
        assert!(record.next_retry_at().is_some());
        assert!(record.trace_context().is_some());
    }

    #[test]
    fn row_conversion_accepts_dead_letter_history_at_attempt_limit() {
        let mut row = row("dead_lettered");
        row.workflow_state = "failed".to_string();
        row.retry_state = "exhausted".to_string();
        row.attempt = 3;
        row.max_attempts = 3;
        row.next_retry_at = None;

        let record = FailureHistoryRecord::try_from(row).expect("valid dead-letter row");

        assert_eq!(record.outcome(), FailureOutcome::DeadLettered);
    }

    #[test]
    fn row_conversion_accepts_permanent_failure_before_attempt_limit() {
        let mut row = row("permanent_failure");
        row.workflow_state = "failed".to_string();
        row.retry_state = "permanent_failure".to_string();
        row.next_retry_at = None;

        let record = FailureHistoryRecord::try_from(row).expect("valid permanent failure row");

        assert_eq!(record.outcome(), FailureOutcome::PermanentFailure);
    }

    #[test]
    fn row_conversion_accepts_human_escalation_history() {
        let mut row = row("escalated_to_human");
        row.failure_source = "policy".to_string();
        row.workflow_state = "waiting_for_human".to_string();
        row.retry_state = "retryable".to_string();
        row.next_retry_at = None;

        let record = FailureHistoryRecord::try_from(row).expect("valid escalation row");

        assert_eq!(record.outcome(), FailureOutcome::EscalatedToHuman);
    }

    #[test]
    fn row_conversion_rejects_unknown_source() {
        let mut row = row("retry_scheduled");
        row.failure_source = "cosmic_ray".to_string();

        let error = FailureHistoryRecord::try_from(row).expect_err("unknown source must fail");

        assert!(matches!(error, FailureHistoryError::UnknownSource { .. }));
    }

    #[test]
    fn row_conversion_rejects_unknown_outcome() {
        let error =
            FailureHistoryRecord::try_from(row("maybe_retry")).expect_err("unknown outcome fails");

        assert!(matches!(error, FailureHistoryError::UnknownOutcome { .. }));
    }

    #[test]
    fn row_conversion_rejects_negative_attempt() {
        let mut row = row("retry_scheduled");
        row.attempt = -1;

        let error = FailureHistoryRecord::try_from(row).expect_err("attempt must be positive");

        assert!(matches!(error, FailureHistoryError::NegativeNumber { .. }));
    }

    #[test]
    fn row_conversion_rejects_attempts_over_max_attempts() {
        let mut row = row("retry_scheduled");
        row.attempt = 4;
        row.max_attempts = 3;

        let error = FailureHistoryRecord::try_from(row).expect_err("attempt cannot exceed max");

        assert_eq!(error, FailureHistoryError::AttemptsExceedMaxAttempts);
    }

    #[test]
    fn row_conversion_rejects_recorded_before_occurred() {
        let mut row = row("retry_scheduled");
        row.recorded_at = row.occurred_at - Duration::seconds(1);

        let error = FailureHistoryRecord::try_from(row).expect_err("timeline must be valid");

        assert_eq!(error, FailureHistoryError::RecordedBeforeOccurred);
    }

    #[test]
    fn row_conversion_rejects_span_without_trace() {
        let mut row = row("retry_scheduled");
        row.trace_id = None;

        let error = FailureHistoryRecord::try_from(row).expect_err("span needs trace id");

        assert_eq!(error, FailureHistoryError::SpanWithoutTrace);
    }

    #[test]
    fn row_conversion_rejects_missing_next_retry_for_retry_outcome() {
        let mut row = row("retry_scheduled");
        row.next_retry_at = None;

        let error = FailureHistoryRecord::try_from(row).expect_err("retry needs next time");

        assert_eq!(error, FailureHistoryError::MissingNextRetryAt);
    }

    #[test]
    fn row_conversion_rejects_next_retry_before_failure() {
        let mut row = row("retry_scheduled");
        row.next_retry_at = Some(row.occurred_at);

        let error = FailureHistoryRecord::try_from(row).expect_err("retry must be in future");

        assert_eq!(error, FailureHistoryError::NextRetryNotAfterFailure);
    }

    #[test]
    fn row_conversion_rejects_next_retry_for_non_retry_outcome() {
        let mut row = row("permanent_failure");
        row.workflow_state = "failed".to_string();
        row.retry_state = "permanent_failure".to_string();

        let error = FailureHistoryRecord::try_from(row)
            .expect_err("permanent failure cannot carry next retry");

        assert_eq!(
            error,
            FailureHistoryError::UnexpectedNextRetryAt {
                outcome: FailureOutcome::PermanentFailure
            }
        );
    }

    #[test]
    fn row_conversion_rejects_dead_letter_before_attempt_limit() {
        let mut row = row("dead_lettered");
        row.workflow_state = "failed".to_string();
        row.retry_state = "exhausted".to_string();
        row.next_retry_at = None;

        let error = FailureHistoryRecord::try_from(row)
            .expect_err("dead-letter requires attempt exhaustion");

        assert_eq!(error, FailureHistoryError::DeadLetterBeforeMaxAttempts);
    }

    #[test]
    fn row_conversion_rejects_incompatible_outcome_state() {
        let mut row = row("retry_scheduled");
        row.workflow_state = "failed".to_string();
        row.retry_state = "permanent_failure".to_string();

        let error =
            FailureHistoryRecord::try_from(row).expect_err("outcome must match state evidence");

        assert!(matches!(
            error,
            FailureHistoryError::IncompatibleOutcomeState { .. }
        ));
    }
}
