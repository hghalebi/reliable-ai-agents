//! Typed lifecycle for `agent_steps`.
//!
//! An agent run is easier to operate when every internal step is durable and
//! typed. Postgres stores step status, references, and timestamps. This module
//! turns those raw rows into lifecycle states before worker, audit, or runbook
//! logic depends on them.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::AgentRunId;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AgentStepError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("step index must be non-negative, got {value}")]
    NegativeStepIndex { value: i32 },
    #[error("unknown agent step kind: {value}")]
    UnknownKind { value: UnknownAgentStepKind },
    #[error("unknown agent step status: {value}")]
    UnknownStatus { value: UnknownAgentStepStatus },
    #[error("{status:?} agent step requires started_at")]
    MissingStartedAt { status: AgentStepStatus },
    #[error("{status:?} agent step cannot carry started_at")]
    UnexpectedStartedAt { status: AgentStepStatus },
    #[error("{status:?} agent step requires completed_at")]
    MissingCompletedAt { status: AgentStepStatus },
    #[error("{status:?} agent step cannot carry completed_at")]
    UnexpectedCompletedAt { status: AgentStepStatus },
    #[error("{status:?} agent step requires output_ref")]
    MissingOutputRef { status: AgentStepStatus },
    #[error("{status:?} agent step cannot carry output_ref")]
    UnexpectedOutputRef { status: AgentStepStatus },
    #[error("{status:?} agent step requires terminal reason")]
    MissingTerminalReason { status: AgentStepStatus },
    #[error("{status:?} agent step cannot carry terminal reason")]
    UnexpectedTerminalReason { status: AgentStepStatus },
    #[error("started_at cannot be before created_at")]
    StartedBeforeCreated,
    #[error("completed_at cannot be before started_at")]
    CompletedBeforeStarted,
    #[error("completed_at cannot be before created_at")]
    CompletedBeforeCreated,
}

fn non_empty_agent_step_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, AgentStepError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(AgentStepError::EmptyText { field });
    }

    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAgentStepKind(String);

impl UnknownAgentStepKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownAgentStepKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAgentStepStatus(String);

impl UnknownAgentStepStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownAgentStepStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStepKind {
    Plan,
    ModelCall,
    ToolCall,
    ApprovalGate,
    Finalize,
}

impl AgentStepKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::ModelCall => "model_call",
            Self::ToolCall => "tool_call",
            Self::ApprovalGate => "approval_gate",
            Self::Finalize => "finalize",
        }
    }
}

impl TryFrom<&str> for AgentStepKind {
    type Error = AgentStepError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "plan" => Ok(Self::Plan),
            "model_call" => Ok(Self::ModelCall),
            "tool_call" => Ok(Self::ToolCall),
            "approval_gate" => Ok(Self::ApprovalGate),
            "finalize" => Ok(Self::Finalize),
            value => Err(AgentStepError::UnknownKind {
                value: UnknownAgentStepKind::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

impl AgentStepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

impl TryFrom<&str> for AgentStepStatus {
    type Error = AgentStepError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            value => Err(AgentStepError::UnknownStatus {
                value: UnknownAgentStepStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentStepId(Uuid);

impl AgentStepId {
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

impl Default for AgentStepId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AgentStepIndex(u32);

impl AgentStepIndex {
    pub fn try_from_i32(value: i32) -> Result<Self, AgentStepError> {
        let value =
            u32::try_from(value).map_err(|_| AgentStepError::NegativeStepIndex { value })?;
        Ok(Self(value))
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStepRef(String);

impl AgentStepRef {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentStepError> {
        Ok(Self(non_empty_agent_step_text(value, "agent_step_ref")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStepTerminalReason(String);

impl AgentStepTerminalReason {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentStepError> {
        Ok(Self(non_empty_agent_step_text(
            value,
            "agent_step_terminal_reason",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentStepRecord {
    id: AgentStepId,
    run_id: AgentRunId,
    index: AgentStepIndex,
    kind: AgentStepKind,
    status: AgentStepStatus,
    input_ref: Option<AgentStepRef>,
    output_ref: Option<AgentStepRef>,
    terminal_reason: Option<AgentStepTerminalReason>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl AgentStepRecord {
    fn with_status<State>(self, status: AgentStepStatus) -> AgentStep<State> {
        AgentStep {
            record: AgentStepRecord { status, ..self },
            state: PhantomData,
        }
    }
}

// ANCHOR: agent_step_typestate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStepPending;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStepRunning;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStepSucceeded;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStepFailed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStepSkipped;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStep<State> {
    record: AgentStepRecord,
    state: PhantomData<State>,
}

impl AgentStep<AgentStepPending> {
    pub fn pending(
        run_id: AgentRunId,
        index: AgentStepIndex,
        kind: AgentStepKind,
        input_ref: Option<AgentStepRef>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            record: AgentStepRecord {
                id: AgentStepId::new(),
                run_id,
                index,
                kind,
                status: AgentStepStatus::Pending,
                input_ref,
                output_ref: None,
                terminal_reason: None,
                started_at: None,
                completed_at: None,
                created_at,
            },
            state: PhantomData,
        }
    }

    pub fn start(
        self,
        started_at: DateTime<Utc>,
    ) -> Result<AgentStep<AgentStepRunning>, AgentStepError> {
        validate_started_at(self.record.created_at, started_at)?;
        Ok(AgentStep {
            record: AgentStepRecord {
                status: AgentStepStatus::Running,
                started_at: Some(started_at),
                ..self.record
            },
            state: PhantomData,
        })
    }

    pub fn skip(
        self,
        reason: AgentStepTerminalReason,
        completed_at: DateTime<Utc>,
    ) -> Result<AgentStep<AgentStepSkipped>, AgentStepError> {
        validate_completed_after_created(self.record.created_at, completed_at)?;
        Ok(AgentStep {
            record: AgentStepRecord {
                status: AgentStepStatus::Skipped,
                terminal_reason: Some(reason),
                completed_at: Some(completed_at),
                ..self.record
            },
            state: PhantomData,
        })
    }
}

impl AgentStep<AgentStepRunning> {
    pub fn succeed(
        self,
        output_ref: AgentStepRef,
        completed_at: DateTime<Utc>,
    ) -> Result<AgentStep<AgentStepSucceeded>, AgentStepError> {
        validate_completed_after_started(self.record.started_at, completed_at)?;
        Ok(AgentStep {
            record: AgentStepRecord {
                status: AgentStepStatus::Succeeded,
                output_ref: Some(output_ref),
                completed_at: Some(completed_at),
                ..self.record
            },
            state: PhantomData,
        })
    }

    pub fn fail(
        self,
        reason: AgentStepTerminalReason,
        completed_at: DateTime<Utc>,
    ) -> Result<AgentStep<AgentStepFailed>, AgentStepError> {
        validate_completed_after_started(self.record.started_at, completed_at)?;
        Ok(AgentStep {
            record: AgentStepRecord {
                status: AgentStepStatus::Failed,
                terminal_reason: Some(reason),
                completed_at: Some(completed_at),
                ..self.record
            },
            state: PhantomData,
        })
    }
}
// ANCHOR_END: agent_step_typestate

impl<State> AgentStep<State> {
    pub fn id(&self) -> AgentStepId {
        self.record.id
    }

    pub fn run_id(&self) -> AgentRunId {
        self.record.run_id
    }

    pub fn index(&self) -> AgentStepIndex {
        self.record.index
    }

    pub fn kind(&self) -> AgentStepKind {
        self.record.kind
    }

    pub fn status(&self) -> AgentStepStatus {
        self.record.status
    }

    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.record.started_at
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.record.completed_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedAgentStep {
    Pending(AgentStep<AgentStepPending>),
    Running(AgentStep<AgentStepRunning>),
    Succeeded(AgentStep<AgentStepSucceeded>),
    Failed(AgentStep<AgentStepFailed>),
    Skipped(AgentStep<AgentStepSkipped>),
}

// ANCHOR: agent_step_row_boundary
#[derive(Debug, Clone)]
pub struct DbAgentStepRow {
    pub id: Uuid,
    pub run_id: Uuid,
    pub step_index: i32,
    pub step_kind: String,
    pub status: String,
    pub input_ref: Option<String>,
    pub output_ref: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<DbAgentStepRow> for DecodedAgentStep {
    type Error = AgentStepError;

    fn try_from(row: DbAgentStepRow) -> Result<Self, Self::Error> {
        let status = AgentStepStatus::try_from(row.status.as_str())?;
        let record = AgentStepRecord {
            id: AgentStepId::from_uuid(row.id),
            run_id: AgentRunId::from_uuid(row.run_id),
            index: AgentStepIndex::try_from_i32(row.step_index)?,
            kind: AgentStepKind::try_from(row.step_kind.as_str())?,
            status,
            input_ref: row.input_ref.map(AgentStepRef::new).transpose()?,
            output_ref: row.output_ref.map(AgentStepRef::new).transpose()?,
            terminal_reason: row.error.map(AgentStepTerminalReason::new).transpose()?,
            started_at: row.started_at,
            completed_at: row.completed_at,
            created_at: row.created_at,
        };

        validate_step_evidence(&record)?;

        Ok(match status {
            AgentStepStatus::Pending => {
                DecodedAgentStep::Pending(record.with_status(AgentStepStatus::Pending))
            }
            AgentStepStatus::Running => {
                DecodedAgentStep::Running(record.with_status(AgentStepStatus::Running))
            }
            AgentStepStatus::Succeeded => {
                DecodedAgentStep::Succeeded(record.with_status(AgentStepStatus::Succeeded))
            }
            AgentStepStatus::Failed => {
                DecodedAgentStep::Failed(record.with_status(AgentStepStatus::Failed))
            }
            AgentStepStatus::Skipped => {
                DecodedAgentStep::Skipped(record.with_status(AgentStepStatus::Skipped))
            }
        })
    }
}
// ANCHOR_END: agent_step_row_boundary

fn validate_started_at(
    created_at: DateTime<Utc>,
    started_at: DateTime<Utc>,
) -> Result<(), AgentStepError> {
    if started_at < created_at {
        return Err(AgentStepError::StartedBeforeCreated);
    }
    Ok(())
}

fn validate_completed_after_created(
    created_at: DateTime<Utc>,
    completed_at: DateTime<Utc>,
) -> Result<(), AgentStepError> {
    if completed_at < created_at {
        return Err(AgentStepError::CompletedBeforeCreated);
    }
    Ok(())
}

fn validate_completed_after_started(
    started_at: Option<DateTime<Utc>>,
    completed_at: DateTime<Utc>,
) -> Result<(), AgentStepError> {
    let Some(started_at) = started_at else {
        return Err(AgentStepError::MissingStartedAt {
            status: AgentStepStatus::Running,
        });
    };

    if completed_at < started_at {
        return Err(AgentStepError::CompletedBeforeStarted);
    }
    Ok(())
}

fn validate_step_evidence(record: &AgentStepRecord) -> Result<(), AgentStepError> {
    if let Some(started_at) = record.started_at
        && started_at < record.created_at
    {
        return Err(AgentStepError::StartedBeforeCreated);
    }

    if let Some(completed_at) = record.completed_at
        && completed_at < record.created_at
    {
        return Err(AgentStepError::CompletedBeforeCreated);
    }

    if let (Some(started_at), Some(completed_at)) = (record.started_at, record.completed_at)
        && completed_at < started_at
    {
        return Err(AgentStepError::CompletedBeforeStarted);
    }

    match record.status {
        AgentStepStatus::Pending => {
            reject_started(record)?;
            reject_completed(record)?;
            reject_output_ref(record)?;
            reject_terminal_reason(record)?;
        }
        AgentStepStatus::Running => {
            require_started(record)?;
            reject_completed(record)?;
            reject_output_ref(record)?;
            reject_terminal_reason(record)?;
        }
        AgentStepStatus::Succeeded => {
            require_started(record)?;
            require_completed(record)?;
            require_output_ref(record)?;
            reject_terminal_reason(record)?;
        }
        AgentStepStatus::Failed => {
            require_started(record)?;
            require_completed(record)?;
            reject_output_ref(record)?;
            require_terminal_reason(record)?;
        }
        AgentStepStatus::Skipped => {
            reject_started(record)?;
            require_completed(record)?;
            reject_output_ref(record)?;
            require_terminal_reason(record)?;
        }
    }

    Ok(())
}

fn require_started(record: &AgentStepRecord) -> Result<(), AgentStepError> {
    if record.started_at.is_none() {
        return Err(AgentStepError::MissingStartedAt {
            status: record.status,
        });
    }
    Ok(())
}

fn reject_started(record: &AgentStepRecord) -> Result<(), AgentStepError> {
    if record.started_at.is_some() {
        return Err(AgentStepError::UnexpectedStartedAt {
            status: record.status,
        });
    }
    Ok(())
}

fn require_completed(record: &AgentStepRecord) -> Result<(), AgentStepError> {
    if record.completed_at.is_none() {
        return Err(AgentStepError::MissingCompletedAt {
            status: record.status,
        });
    }
    Ok(())
}

fn reject_completed(record: &AgentStepRecord) -> Result<(), AgentStepError> {
    if record.completed_at.is_some() {
        return Err(AgentStepError::UnexpectedCompletedAt {
            status: record.status,
        });
    }
    Ok(())
}

fn require_output_ref(record: &AgentStepRecord) -> Result<(), AgentStepError> {
    if record.output_ref.is_none() {
        return Err(AgentStepError::MissingOutputRef {
            status: record.status,
        });
    }
    Ok(())
}

fn reject_output_ref(record: &AgentStepRecord) -> Result<(), AgentStepError> {
    if record.output_ref.is_some() {
        return Err(AgentStepError::UnexpectedOutputRef {
            status: record.status,
        });
    }
    Ok(())
}

fn require_terminal_reason(record: &AgentStepRecord) -> Result<(), AgentStepError> {
    if record.terminal_reason.is_none() {
        return Err(AgentStepError::MissingTerminalReason {
            status: record.status,
        });
    }
    Ok(())
}

fn reject_terminal_reason(record: &AgentStepRecord) -> Result<(), AgentStepError> {
    if record.terminal_reason.is_some() {
        return Err(AgentStepError::UnexpectedTerminalReason {
            status: record.status,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn step_index() -> AgentStepIndex {
        AgentStepIndex::try_from_i32(1).expect("valid step index")
    }

    fn input_ref() -> AgentStepRef {
        AgentStepRef::new("memory://run/input-1").expect("valid input ref")
    }

    fn output_ref() -> AgentStepRef {
        AgentStepRef::new("memory://run/output-1").expect("valid output ref")
    }

    fn reason() -> AgentStepTerminalReason {
        AgentStepTerminalReason::new("approval gate denied").expect("valid reason")
    }

    fn pending_step() -> AgentStep<AgentStepPending> {
        AgentStep::pending(
            AgentRunId::new(),
            step_index(),
            AgentStepKind::ToolCall,
            Some(input_ref()),
            now(),
        )
    }

    fn row(status: &str) -> DbAgentStepRow {
        DbAgentStepRow {
            id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            step_index: 1,
            step_kind: "tool_call".to_string(),
            status: status.to_string(),
            input_ref: Some("memory://run/input-1".to_string()),
            output_ref: None,
            error: None,
            started_at: None,
            completed_at: None,
            created_at: now(),
        }
    }

    #[test]
    fn pending_step_can_start_and_succeed() {
        let started_at = now() + Duration::seconds(1);
        let completed_at = started_at + Duration::seconds(3);

        let succeeded = pending_step()
            .start(started_at)
            .expect("valid start")
            .succeed(output_ref(), completed_at)
            .expect("valid completion");

        assert_eq!(succeeded.status(), AgentStepStatus::Succeeded);
        assert_eq!(succeeded.completed_at(), Some(completed_at));
    }

    #[test]
    fn pending_step_can_be_skipped_before_running() {
        let skipped = pending_step()
            .skip(reason(), now() + Duration::seconds(1))
            .expect("valid skip");

        assert_eq!(skipped.status(), AgentStepStatus::Skipped);
    }

    #[test]
    fn row_conversion_accepts_running_step() {
        let mut row = row("running");
        row.started_at = Some(now() + Duration::seconds(1));

        let decoded = DecodedAgentStep::try_from(row).expect("valid running row");

        assert!(matches!(decoded, DecodedAgentStep::Running(_)));
    }

    #[test]
    fn row_conversion_accepts_succeeded_step_with_output_ref() {
        let mut row = row("succeeded");
        row.started_at = Some(now() + Duration::seconds(1));
        row.completed_at = Some(now() + Duration::seconds(3));
        row.output_ref = Some("memory://run/output-1".to_string());

        let decoded = DecodedAgentStep::try_from(row).expect("valid succeeded row");

        assert!(matches!(decoded, DecodedAgentStep::Succeeded(_)));
    }

    #[test]
    fn row_conversion_accepts_skipped_step_with_reason() {
        let mut row = row("skipped");
        row.completed_at = Some(now() + Duration::seconds(1));
        row.error = Some("approval gate denied".to_string());

        let decoded = DecodedAgentStep::try_from(row).expect("valid skipped row");

        assert!(matches!(decoded, DecodedAgentStep::Skipped(_)));
    }

    #[test]
    fn row_conversion_rejects_negative_step_index() {
        let mut row = row("pending");
        row.step_index = -1;

        let error = DecodedAgentStep::try_from(row).expect_err("negative index");

        assert!(matches!(error, AgentStepError::NegativeStepIndex { .. }));
    }

    #[test]
    fn row_conversion_rejects_unknown_kind() {
        let mut row = row("pending");
        row.step_kind = "magic".to_string();

        let error = DecodedAgentStep::try_from(row).expect_err("unknown kind");

        assert!(matches!(error, AgentStepError::UnknownKind { .. }));
    }

    #[test]
    fn row_conversion_rejects_unknown_status() {
        let error = DecodedAgentStep::try_from(row("mystery")).expect_err("unknown status");

        assert!(matches!(error, AgentStepError::UnknownStatus { .. }));
    }

    #[test]
    fn row_conversion_rejects_succeeded_without_output_ref() {
        let mut row = row("succeeded");
        row.started_at = Some(now() + Duration::seconds(1));
        row.completed_at = Some(now() + Duration::seconds(3));

        let error = DecodedAgentStep::try_from(row).expect_err("missing output ref");

        assert!(matches!(error, AgentStepError::MissingOutputRef { .. }));
    }

    #[test]
    fn row_conversion_rejects_failed_without_reason() {
        let mut row = row("failed");
        row.started_at = Some(now() + Duration::seconds(1));
        row.completed_at = Some(now() + Duration::seconds(3));

        let error = DecodedAgentStep::try_from(row).expect_err("missing failure reason");

        assert!(matches!(
            error,
            AgentStepError::MissingTerminalReason { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_active_step_with_completed_at() {
        let mut row = row("running");
        row.started_at = Some(now() + Duration::seconds(1));
        row.completed_at = Some(now() + Duration::seconds(2));

        let error = DecodedAgentStep::try_from(row).expect_err("running row cannot be completed");

        assert!(matches!(
            error,
            AgentStepError::UnexpectedCompletedAt { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_completed_before_started() {
        let mut row = row("succeeded");
        row.started_at = Some(now() + Duration::seconds(3));
        row.completed_at = Some(now() + Duration::seconds(1));
        row.output_ref = Some("memory://run/output-1".to_string());

        let error = DecodedAgentStep::try_from(row).expect_err("invalid timeline");

        assert!(matches!(error, AgentStepError::CompletedBeforeStarted));
    }

    #[test]
    fn row_conversion_rejects_empty_output_ref() {
        let mut row = row("succeeded");
        row.started_at = Some(now() + Duration::seconds(1));
        row.completed_at = Some(now() + Duration::seconds(3));
        row.output_ref = Some(" ".to_string());

        let error = DecodedAgentStep::try_from(row).expect_err("empty output ref");

        assert!(matches!(error, AgentStepError::EmptyText { .. }));
    }
}
