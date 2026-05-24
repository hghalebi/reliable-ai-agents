//! Typed timeout and deadline policy.
//!
//! A lease protects ownership. A timeout protects user and operator promises.
//! They are related, but they are not the same production invariant.

use std::num::NonZeroU32;
use std::time::Duration;

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AgentRunId, AttemptCount, DomainError, JobId, JobKind, MaxAttempts};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TimeoutError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} must be positive, got {value}")]
    NonPositiveNumber { field: &'static str, value: i64 },
    #[error("{field} is too large for this example: {value}")]
    NumberOutOfRange { field: &'static str, value: i64 },
    #[error("unknown timeout action: {value}")]
    UnknownAction { value: UnknownTimeoutAction },
    #[error("unknown timeout observed state: {value}")]
    UnknownObservedState { value: UnknownTimeoutObservedState },
    #[error("deadline must be after started_at")]
    DeadlineNotAfterStart,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_timeout_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, TimeoutError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(TimeoutError::EmptyText { field });
    }
    Ok(value)
}

fn non_negative_u32(value: i64, field: &'static str) -> Result<u32, TimeoutError> {
    if value < 0 {
        return Err(TimeoutError::NonPositiveNumber { field, value });
    }

    u32::try_from(value).map_err(|_| TimeoutError::NumberOutOfRange { field, value })
}

fn positive_non_zero_u32(value: i64, field: &'static str) -> Result<NonZeroU32, TimeoutError> {
    if value <= 0 {
        return Err(TimeoutError::NonPositiveNumber { field, value });
    }

    let value =
        u32::try_from(value).map_err(|_| TimeoutError::NumberOutOfRange { field, value })?;

    NonZeroU32::new(value).ok_or(TimeoutError::NonPositiveNumber {
        field,
        value: i64::from(value),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTimeoutAction(String);

impl UnknownTimeoutAction {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownTimeoutAction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTimeoutObservedState(String);

impl UnknownTimeoutObservedState {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownTimeoutObservedState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeoutPolicyName(String);

impl TimeoutPolicyName {
    pub fn new(value: impl Into<String>) -> Result<Self, TimeoutError> {
        Ok(Self(non_empty_timeout_text(value, "timeout_policy_name")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutDuration(Duration);

impl TimeoutDuration {
    pub fn from_secs(seconds: u64) -> Result<Self, TimeoutError> {
        if seconds == 0 {
            return Err(TimeoutError::NonPositiveNumber {
                field: "timeout_seconds",
                value: 0,
            });
        }

        Ok(Self(Duration::from_secs(seconds)))
    }

    pub fn duration(self) -> Duration {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutAction {
    ScheduleRetry,
    CancelJob,
    EscalateToHuman,
    DeadLetter,
}

impl TimeoutAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ScheduleRetry => "schedule_retry",
            Self::CancelJob => "cancel_job",
            Self::EscalateToHuman => "escalate_to_human",
            Self::DeadLetter => "dead_letter",
        }
    }
}

impl TryFrom<&str> for TimeoutAction {
    type Error = TimeoutError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "schedule_retry" => Ok(Self::ScheduleRetry),
            "cancel_job" => Ok(Self::CancelJob),
            "escalate_to_human" => Ok(Self::EscalateToHuman),
            "dead_letter" => Ok(Self::DeadLetter),
            value => Err(TimeoutError::UnknownAction {
                value: UnknownTimeoutAction::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutObservedState {
    Running,
    WaitingForHuman,
}

impl TimeoutObservedState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::WaitingForHuman => "waiting_for_human",
        }
    }
}

impl TryFrom<&str> for TimeoutObservedState {
    type Error = TimeoutError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "running" => Ok(Self::Running),
            "waiting_for_human" => Ok(Self::WaitingForHuman),
            value => Err(TimeoutError::UnknownObservedState {
                value: UnknownTimeoutObservedState::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionDeadline(DateTime<Utc>);

impl ExecutionDeadline {
    pub fn new(
        started_at: DateTime<Utc>,
        deadline_at: DateTime<Utc>,
    ) -> Result<Self, TimeoutError> {
        if deadline_at <= started_at {
            return Err(TimeoutError::DeadlineNotAfterStart);
        }

        Ok(Self(deadline_at))
    }

    pub fn at(self) -> DateTime<Utc> {
        self.0
    }

    pub fn is_breached_at(self, now: DateTime<Utc>) -> bool {
        now > self.0
    }
}

// ANCHOR: timeout_policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeoutPolicy {
    name: TimeoutPolicyName,
    duration: TimeoutDuration,
    action: TimeoutAction,
}

impl TimeoutPolicy {
    pub fn new(name: TimeoutPolicyName, duration: TimeoutDuration, action: TimeoutAction) -> Self {
        Self {
            name,
            duration,
            action,
        }
    }

    pub fn deadline_for(&self, started_at: DateTime<Utc>) -> ExecutionDeadline {
        ExecutionDeadline(started_at + self.duration.duration())
    }

    pub fn action(&self) -> TimeoutAction {
        self.action
    }

    pub fn name(&self) -> &TimeoutPolicyName {
        &self.name
    }
}
// ANCHOR_END: timeout_policy

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutDecision {
    WithinDeadline,
    TimedOut { action: TimeoutAction },
}

// ANCHOR: timeout_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningJobDeadline {
    pub job_id: JobId,
    pub run_id: Option<AgentRunId>,
    pub job_kind: JobKind,
    pub observed_state: TimeoutObservedState,
    pub policy_name: TimeoutPolicyName,
    pub started_at: DateTime<Utc>,
    pub deadline: ExecutionDeadline,
    pub configured_action: TimeoutAction,
    pub attempts: AttemptCount,
    pub max_attempts: MaxAttempts,
}

impl RunningJobDeadline {
    pub fn evaluate(&self, now: DateTime<Utc>) -> TimeoutDecision {
        if !self.deadline.is_breached_at(now) {
            return TimeoutDecision::WithinDeadline;
        }

        TimeoutDecision::TimedOut {
            action: self.effective_action(),
        }
    }

    pub fn effective_action(&self) -> TimeoutAction {
        if self.configured_action == TimeoutAction::ScheduleRetry
            && self.attempts.get() >= self.max_attempts.get()
        {
            TimeoutAction::DeadLetter
        } else {
            self.configured_action
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbRunningJobDeadlineRow {
    pub job_id: Uuid,
    pub run_id: Option<Uuid>,
    pub job_kind: String,
    pub observed_state: String,
    pub timeout_policy_name: String,
    pub started_at: DateTime<Utc>,
    pub deadline_at: DateTime<Utc>,
    pub timeout_action: String,
    pub attempts: i64,
    pub max_attempts: i64,
}

impl TryFrom<DbRunningJobDeadlineRow> for RunningJobDeadline {
    type Error = TimeoutError;

    fn try_from(row: DbRunningJobDeadlineRow) -> Result<Self, Self::Error> {
        let attempts = AttemptCount::try_from_u32(non_negative_u32(row.attempts, "attempts")?)?;
        let max_attempts =
            MaxAttempts::new(positive_non_zero_u32(row.max_attempts, "max_attempts")?);

        Ok(Self {
            job_id: JobId::from_uuid(row.job_id),
            run_id: row.run_id.map(AgentRunId::from_uuid),
            job_kind: JobKind::new(row.job_kind)?,
            observed_state: TimeoutObservedState::try_from(row.observed_state.as_str())?,
            policy_name: TimeoutPolicyName::new(row.timeout_policy_name)?,
            started_at: row.started_at,
            deadline: ExecutionDeadline::new(row.started_at, row.deadline_at)?,
            configured_action: TimeoutAction::try_from(row.timeout_action.as_str())?,
            attempts,
            max_attempts,
        })
    }
}
// ANCHOR_END: timeout_row_boundary

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn started_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 9, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    fn row() -> DbRunningJobDeadlineRow {
        let started_at = started_at();

        DbRunningJobDeadlineRow {
            job_id: Uuid::new_v4(),
            run_id: Some(Uuid::new_v4()),
            job_kind: "incident_triage".to_string(),
            observed_state: "running".to_string(),
            timeout_policy_name: "standard-agent:v1".to_string(),
            started_at,
            deadline_at: started_at + Duration::from_secs(60),
            timeout_action: "schedule_retry".to_string(),
            attempts: 1,
            max_attempts: 3,
        }
    }

    #[test]
    fn timeout_policy_computes_deadline_from_start_time() {
        let policy = TimeoutPolicy::new(
            TimeoutPolicyName::new("standard-agent:v1").expect("valid policy name"),
            TimeoutDuration::from_secs(120).expect("valid duration"),
            TimeoutAction::ScheduleRetry,
        );

        assert_eq!(
            policy.deadline_for(started_at()).at(),
            started_at() + Duration::from_secs(120)
        );
        assert_eq!(policy.name().as_str(), "standard-agent:v1");
        assert_eq!(policy.action(), TimeoutAction::ScheduleRetry);
    }

    #[test]
    fn running_job_deadline_stays_within_deadline_before_expiry() {
        let record = RunningJobDeadline::try_from(row()).expect("valid row");

        assert_eq!(
            record.evaluate(started_at() + Duration::from_secs(30)),
            TimeoutDecision::WithinDeadline
        );
    }

    #[test]
    fn running_job_deadline_times_out_after_expiry() {
        let record = RunningJobDeadline::try_from(row()).expect("valid row");

        assert_eq!(
            record.evaluate(started_at() + Duration::from_secs(61)),
            TimeoutDecision::TimedOut {
                action: TimeoutAction::ScheduleRetry
            }
        );
    }

    #[test]
    fn exhausted_retry_timeout_becomes_dead_letter_action() {
        let row = DbRunningJobDeadlineRow {
            attempts: 3,
            max_attempts: 3,
            ..row()
        };
        let record = RunningJobDeadline::try_from(row).expect("valid row");

        assert_eq!(record.effective_action(), TimeoutAction::DeadLetter);
    }

    #[test]
    fn row_conversion_rejects_unknown_timeout_action() {
        let row = DbRunningJobDeadlineRow {
            timeout_action: "ignore".to_string(),
            ..row()
        };

        let error = RunningJobDeadline::try_from(row).expect_err("unknown action");

        assert!(matches!(error, TimeoutError::UnknownAction { .. }));
    }

    #[test]
    fn row_conversion_rejects_terminal_observed_state() {
        let row = DbRunningJobDeadlineRow {
            observed_state: "completed".to_string(),
            ..row()
        };

        let error = RunningJobDeadline::try_from(row).expect_err("terminal state");

        assert!(matches!(error, TimeoutError::UnknownObservedState { .. }));
    }

    #[test]
    fn row_conversion_rejects_invalid_deadline() {
        let started_at = started_at();
        let row = DbRunningJobDeadlineRow {
            started_at,
            deadline_at: started_at,
            ..row()
        };

        let error = RunningJobDeadline::try_from(row).expect_err("invalid deadline");

        assert_eq!(error, TimeoutError::DeadlineNotAfterStart);
    }

    #[test]
    fn row_conversion_rejects_negative_attempts() {
        let row = DbRunningJobDeadlineRow {
            attempts: -1,
            ..row()
        };

        let error = RunningJobDeadline::try_from(row).expect_err("negative attempts");

        assert_eq!(
            error,
            TimeoutError::NonPositiveNumber {
                field: "attempts",
                value: -1
            }
        );
    }

    #[test]
    fn timeout_duration_rejects_zero_seconds() {
        let error = TimeoutDuration::from_secs(0).expect_err("zero timeout");

        assert_eq!(
            error,
            TimeoutError::NonPositiveNumber {
                field: "timeout_seconds",
                value: 0
            }
        );
    }
}
