//! Typed lifecycle for durable tool calls.
//!
//! A tool call is a side effect boundary. Postgres stores JSON and status
//! strings, but application code should work with typed lifecycle states,
//! validated payload wrappers, and explicit terminal evidence.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::agent_step::AgentStepId;
use crate::domain::{AgentRunId, DomainError, IdempotencyKey, ToolVersion};
use crate::tool_contract::{ToolContractError, ToolName};

#[derive(Debug, Error, PartialEq)]
pub enum ToolCallError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("tool call input must be a JSON object")]
    InputMustBeObject,
    #[error("tool call output must be a JSON object")]
    OutputMustBeObject,
    #[error("unknown tool call status: {value}")]
    UnknownStatus { value: UnknownToolCallStatus },
    #[error("{status:?} tool call requires started_at")]
    MissingStartedAt { status: ToolCallStatus },
    #[error("{status:?} tool call requires completed_at")]
    MissingCompletedAt { status: ToolCallStatus },
    #[error("{status:?} tool call cannot carry completed_at")]
    UnexpectedCompletedAt { status: ToolCallStatus },
    #[error("{status:?} tool call cannot carry started_at")]
    UnexpectedStartedAt { status: ToolCallStatus },
    #[error("{status:?} tool call requires output")]
    MissingOutput { status: ToolCallStatus },
    #[error("{status:?} tool call cannot carry output")]
    UnexpectedOutput { status: ToolCallStatus },
    #[error("{status:?} tool call requires terminal reason")]
    MissingTerminalReason { status: ToolCallStatus },
    #[error("{status:?} tool call cannot carry terminal reason")]
    UnexpectedTerminalReason { status: ToolCallStatus },
    #[error("started_at cannot be before created_at")]
    StartedBeforeCreated,
    #[error("completed_at cannot be before started_at")]
    CompletedBeforeStarted,
    #[error("completed_at cannot be before created_at")]
    CompletedBeforeCreated,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
    #[error("tool contract validation failed: {0}")]
    ToolContract(#[from] ToolContractError),
}

fn non_empty_tool_call_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, ToolCallError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(ToolCallError::EmptyText { field });
    }

    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownToolCallStatus(String);

impl UnknownToolCallStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownToolCallStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallStatus {
    Requested,
    Validated,
    Executed,
    Failed,
    Rejected,
}

impl ToolCallStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Validated => "validated",
            Self::Executed => "executed",
            Self::Failed => "failed",
            Self::Rejected => "rejected",
        }
    }
}

impl TryFrom<&str> for ToolCallStatus {
    type Error = ToolCallError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "requested" => Ok(Self::Requested),
            "validated" => Ok(Self::Validated),
            "executed" => Ok(Self::Executed),
            "failed" => Ok(Self::Failed),
            "rejected" => Ok(Self::Rejected),
            value => Err(ToolCallError::UnknownStatus {
                value: UnknownToolCallStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToolCallId(Uuid);

impl ToolCallId {
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

impl Default for ToolCallId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq)]
pub struct ToolCallInput(Value);

impl ToolCallInput {
    pub fn new(value: Value) -> Result<Self, ToolCallError> {
        if !value.is_object() {
            return Err(ToolCallError::InputMustBeObject);
        }

        Ok(Self(value))
    }
}

impl std::fmt::Debug for ToolCallInput {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ToolCallInput([redacted])")
    }
}

#[derive(Clone, PartialEq)]
pub struct ToolCallOutput(Value);

impl ToolCallOutput {
    pub fn new(value: Value) -> Result<Self, ToolCallError> {
        if !value.is_object() {
            return Err(ToolCallError::OutputMustBeObject);
        }

        Ok(Self(value))
    }
}

impl std::fmt::Debug for ToolCallOutput {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ToolCallOutput([redacted])")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallTerminalReason(String);

impl ToolCallTerminalReason {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolCallError> {
        Ok(Self(non_empty_tool_call_text(
            value,
            "tool_call_terminal_reason",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ToolCallRecord {
    id: ToolCallId,
    run_id: AgentRunId,
    step_id: Option<AgentStepId>,
    tool_name: ToolName,
    tool_version: ToolVersion,
    status: ToolCallStatus,
    idempotency_key: IdempotencyKey,
    input: ToolCallInput,
    output: Option<ToolCallOutput>,
    terminal_reason: Option<ToolCallTerminalReason>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl ToolCallRecord {
    fn with_status<State>(self, status: ToolCallStatus) -> ToolCall<State> {
        ToolCall {
            record: ToolCallRecord { status, ..self },
            state: PhantomData,
        }
    }
}

// ANCHOR: tool_call_typestate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallRequested;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallValidated;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallExecuted;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallFailed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallRejected;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall<State> {
    record: ToolCallRecord,
    state: PhantomData<State>,
}

impl ToolCall<ToolCallRequested> {
    pub fn request(
        run_id: AgentRunId,
        tool_name: ToolName,
        tool_version: ToolVersion,
        idempotency_key: IdempotencyKey,
        input: ToolCallInput,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            record: ToolCallRecord {
                id: ToolCallId::new(),
                run_id,
                step_id: None,
                tool_name,
                tool_version,
                status: ToolCallStatus::Requested,
                idempotency_key,
                input,
                output: None,
                terminal_reason: None,
                started_at: None,
                completed_at: None,
                created_at,
            },
            state: PhantomData,
        }
    }

    pub fn validate(self) -> ToolCall<ToolCallValidated> {
        self.record.with_status(ToolCallStatus::Validated)
    }

    pub fn reject(
        self,
        reason: ToolCallTerminalReason,
        completed_at: DateTime<Utc>,
    ) -> Result<ToolCall<ToolCallRejected>, ToolCallError> {
        reject_record(self.record, reason, completed_at)
    }
}

impl ToolCall<ToolCallValidated> {
    pub fn execute(
        self,
        output: ToolCallOutput,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    ) -> Result<ToolCall<ToolCallExecuted>, ToolCallError> {
        validate_execution_timeline(self.record.created_at, started_at, completed_at)?;

        Ok(ToolCall {
            record: ToolCallRecord {
                status: ToolCallStatus::Executed,
                output: Some(output),
                started_at: Some(started_at),
                completed_at: Some(completed_at),
                ..self.record
            },
            state: PhantomData,
        })
    }

    pub fn fail(
        self,
        reason: ToolCallTerminalReason,
        started_at: DateTime<Utc>,
        completed_at: DateTime<Utc>,
    ) -> Result<ToolCall<ToolCallFailed>, ToolCallError> {
        validate_execution_timeline(self.record.created_at, started_at, completed_at)?;

        Ok(ToolCall {
            record: ToolCallRecord {
                status: ToolCallStatus::Failed,
                terminal_reason: Some(reason),
                started_at: Some(started_at),
                completed_at: Some(completed_at),
                ..self.record
            },
            state: PhantomData,
        })
    }

    pub fn reject(
        self,
        reason: ToolCallTerminalReason,
        completed_at: DateTime<Utc>,
    ) -> Result<ToolCall<ToolCallRejected>, ToolCallError> {
        reject_record(self.record, reason, completed_at)
    }
}
// ANCHOR_END: tool_call_typestate

impl<State> ToolCall<State> {
    pub fn id(&self) -> ToolCallId {
        self.record.id
    }

    pub fn run_id(&self) -> AgentRunId {
        self.record.run_id
    }

    pub fn step_id(&self) -> Option<AgentStepId> {
        self.record.step_id
    }

    pub fn tool_name(&self) -> &ToolName {
        &self.record.tool_name
    }

    pub fn tool_version(&self) -> &ToolVersion {
        &self.record.tool_version
    }

    pub fn status(&self) -> ToolCallStatus {
        self.record.status
    }

    pub fn idempotency_key(&self) -> &IdempotencyKey {
        &self.record.idempotency_key
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.record.created_at
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.record.completed_at
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecodedToolCall {
    Requested(ToolCall<ToolCallRequested>),
    Validated(ToolCall<ToolCallValidated>),
    Executed(ToolCall<ToolCallExecuted>),
    Failed(ToolCall<ToolCallFailed>),
    Rejected(ToolCall<ToolCallRejected>),
}

// ANCHOR: tool_call_row_boundary
#[derive(Debug, Clone)]
pub struct DbToolCallRow {
    pub id: Uuid,
    pub run_id: Uuid,
    pub step_id: Option<Uuid>,
    pub tool_name: String,
    pub tool_version: String,
    pub status: String,
    pub idempotency_key: String,
    pub input: Value,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<DbToolCallRow> for DecodedToolCall {
    type Error = ToolCallError;

    fn try_from(row: DbToolCallRow) -> Result<Self, Self::Error> {
        let status = ToolCallStatus::try_from(row.status.as_str())?;
        let record = ToolCallRecord {
            id: ToolCallId::from_uuid(row.id),
            run_id: AgentRunId::from_uuid(row.run_id),
            step_id: row.step_id.map(AgentStepId::from_uuid),
            tool_name: ToolName::new(row.tool_name)?,
            tool_version: ToolVersion::new(row.tool_version)?,
            status,
            idempotency_key: IdempotencyKey::new(row.idempotency_key)?,
            input: ToolCallInput::new(row.input)?,
            output: row.output.map(ToolCallOutput::new).transpose()?,
            terminal_reason: row.error.map(ToolCallTerminalReason::new).transpose()?,
            started_at: row.started_at,
            completed_at: row.completed_at,
            created_at: row.created_at,
        };

        validate_status_evidence(&record)?;

        Ok(match status {
            ToolCallStatus::Requested => {
                DecodedToolCall::Requested(record.with_status(ToolCallStatus::Requested))
            }
            ToolCallStatus::Validated => {
                DecodedToolCall::Validated(record.with_status(ToolCallStatus::Validated))
            }
            ToolCallStatus::Executed => {
                DecodedToolCall::Executed(record.with_status(ToolCallStatus::Executed))
            }
            ToolCallStatus::Failed => {
                DecodedToolCall::Failed(record.with_status(ToolCallStatus::Failed))
            }
            ToolCallStatus::Rejected => {
                DecodedToolCall::Rejected(record.with_status(ToolCallStatus::Rejected))
            }
        })
    }
}
// ANCHOR_END: tool_call_row_boundary

fn reject_record(
    record: ToolCallRecord,
    reason: ToolCallTerminalReason,
    completed_at: DateTime<Utc>,
) -> Result<ToolCall<ToolCallRejected>, ToolCallError> {
    if completed_at < record.created_at {
        return Err(ToolCallError::CompletedBeforeCreated);
    }

    Ok(ToolCall {
        record: ToolCallRecord {
            status: ToolCallStatus::Rejected,
            terminal_reason: Some(reason),
            completed_at: Some(completed_at),
            ..record
        },
        state: PhantomData,
    })
}

fn validate_execution_timeline(
    created_at: DateTime<Utc>,
    started_at: DateTime<Utc>,
    completed_at: DateTime<Utc>,
) -> Result<(), ToolCallError> {
    if started_at < created_at {
        return Err(ToolCallError::StartedBeforeCreated);
    }

    if completed_at < started_at {
        return Err(ToolCallError::CompletedBeforeStarted);
    }

    Ok(())
}

fn validate_status_evidence(record: &ToolCallRecord) -> Result<(), ToolCallError> {
    if let Some(started_at) = record.started_at
        && started_at < record.created_at
    {
        return Err(ToolCallError::StartedBeforeCreated);
    }

    if let Some(completed_at) = record.completed_at
        && completed_at < record.created_at
    {
        return Err(ToolCallError::CompletedBeforeCreated);
    }

    if let (Some(started_at), Some(completed_at)) = (record.started_at, record.completed_at)
        && completed_at < started_at
    {
        return Err(ToolCallError::CompletedBeforeStarted);
    }

    match record.status {
        ToolCallStatus::Requested | ToolCallStatus::Validated => {
            reject_active_evidence(record)?;
        }
        ToolCallStatus::Executed => {
            require_started(record)?;
            require_completed(record)?;
            require_output(record)?;
            reject_terminal_reason(record)?;
        }
        ToolCallStatus::Failed => {
            require_started(record)?;
            require_completed(record)?;
            reject_output(record)?;
            require_terminal_reason(record)?;
        }
        ToolCallStatus::Rejected => {
            require_completed(record)?;
            reject_output(record)?;
            require_terminal_reason(record)?;
            if record.started_at.is_some() {
                return Err(ToolCallError::UnexpectedStartedAt {
                    status: record.status,
                });
            }
        }
    }

    Ok(())
}

fn reject_active_evidence(record: &ToolCallRecord) -> Result<(), ToolCallError> {
    if record.completed_at.is_some() {
        return Err(ToolCallError::UnexpectedCompletedAt {
            status: record.status,
        });
    }
    reject_output(record)?;
    reject_terminal_reason(record)
}

fn require_started(record: &ToolCallRecord) -> Result<(), ToolCallError> {
    if record.started_at.is_none() {
        return Err(ToolCallError::MissingStartedAt {
            status: record.status,
        });
    }
    Ok(())
}

fn require_completed(record: &ToolCallRecord) -> Result<(), ToolCallError> {
    if record.completed_at.is_none() {
        return Err(ToolCallError::MissingCompletedAt {
            status: record.status,
        });
    }
    Ok(())
}

fn require_output(record: &ToolCallRecord) -> Result<(), ToolCallError> {
    if record.output.is_none() {
        return Err(ToolCallError::MissingOutput {
            status: record.status,
        });
    }
    Ok(())
}

fn reject_output(record: &ToolCallRecord) -> Result<(), ToolCallError> {
    if record.output.is_some() {
        return Err(ToolCallError::UnexpectedOutput {
            status: record.status,
        });
    }
    Ok(())
}

fn require_terminal_reason(record: &ToolCallRecord) -> Result<(), ToolCallError> {
    if record.terminal_reason.is_none() {
        return Err(ToolCallError::MissingTerminalReason {
            status: record.status,
        });
    }
    Ok(())
}

fn reject_terminal_reason(record: &ToolCallRecord) -> Result<(), ToolCallError> {
    if record.terminal_reason.is_some() {
        return Err(ToolCallError::UnexpectedTerminalReason {
            status: record.status,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};
    use serde_json::json;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn run_id() -> AgentRunId {
        AgentRunId::new()
    }

    fn tool_name() -> ToolName {
        ToolName::pause_job_kind()
    }

    fn tool_version() -> ToolVersion {
        ToolVersion::new("ops-tools:v1").expect("valid tool version")
    }

    fn idempotency_key() -> IdempotencyKey {
        IdempotencyKey::new("run-7:pause-job-kind").expect("valid idempotency key")
    }

    fn input() -> ToolCallInput {
        ToolCallInput::new(json!({
            "job_kind": "incident_triage",
            "reason": "provider quota exhausted"
        }))
        .expect("object input")
    }

    fn output() -> ToolCallOutput {
        ToolCallOutput::new(json!({
            "event": "job kind paused"
        }))
        .expect("object output")
    }

    fn reason() -> ToolCallTerminalReason {
        ToolCallTerminalReason::new("policy denied execution").expect("valid reason")
    }

    fn row(status: &str) -> DbToolCallRow {
        DbToolCallRow {
            id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            step_id: Some(Uuid::new_v4()),
            tool_name: "pause_job_kind".to_string(),
            tool_version: "ops-tools:v1".to_string(),
            status: status.to_string(),
            idempotency_key: "run-7:pause-job-kind".to_string(),
            input: json!({
                "job_kind": "incident_triage",
                "reason": "provider quota exhausted"
            }),
            output: None,
            error: None,
            started_at: None,
            completed_at: None,
            created_at: now(),
        }
    }

    #[test]
    fn requested_tool_call_can_validate_and_execute() {
        let started_at = now() + Duration::seconds(1);
        let completed_at = started_at + Duration::seconds(2);
        let executed = ToolCall::request(
            run_id(),
            tool_name(),
            tool_version(),
            idempotency_key(),
            input(),
            now(),
        )
        .validate()
        .execute(output(), started_at, completed_at)
        .expect("valid execution timeline");

        assert_eq!(executed.status(), ToolCallStatus::Executed);
        assert_eq!(executed.completed_at(), Some(completed_at));
    }

    #[test]
    fn requested_tool_call_can_be_rejected_before_execution() {
        let rejected = ToolCall::request(
            run_id(),
            tool_name(),
            tool_version(),
            idempotency_key(),
            input(),
            now(),
        )
        .reject(reason(), now() + Duration::seconds(1))
        .expect("valid rejection timeline");

        assert_eq!(rejected.status(), ToolCallStatus::Rejected);
    }

    #[test]
    fn row_conversion_accepts_executed_call_with_output_and_timeline() {
        let mut row = row("executed");
        row.started_at = Some(now() + Duration::seconds(1));
        row.completed_at = Some(now() + Duration::seconds(3));
        row.output = Some(json!({ "event": "job kind paused" }));

        let decoded = DecodedToolCall::try_from(row).expect("valid executed row");

        assert!(matches!(decoded, DecodedToolCall::Executed(_)));
    }

    #[test]
    fn row_conversion_accepts_failed_call_with_reason() {
        let mut row = row("failed");
        row.started_at = Some(now() + Duration::seconds(1));
        row.completed_at = Some(now() + Duration::seconds(3));
        row.error = Some("provider timeout".to_string());

        let decoded = DecodedToolCall::try_from(row).expect("valid failed row");

        assert!(matches!(decoded, DecodedToolCall::Failed(_)));
    }

    #[test]
    fn row_conversion_rejects_unknown_status() {
        let error = DecodedToolCall::try_from(row("mystery")).expect_err("unknown status");

        assert!(matches!(error, ToolCallError::UnknownStatus { .. }));
    }

    #[test]
    fn row_conversion_rejects_non_object_input() {
        let mut row = row("requested");
        row.input = json!("not an object");

        let error = DecodedToolCall::try_from(row).expect_err("invalid input shape");

        assert!(matches!(error, ToolCallError::InputMustBeObject));
    }

    #[test]
    fn row_conversion_rejects_executed_without_output() {
        let mut row = row("executed");
        row.started_at = Some(now() + Duration::seconds(1));
        row.completed_at = Some(now() + Duration::seconds(3));

        let error = DecodedToolCall::try_from(row).expect_err("executed row needs output");

        assert!(matches!(error, ToolCallError::MissingOutput { .. }));
    }

    #[test]
    fn row_conversion_rejects_failed_without_reason() {
        let mut row = row("failed");
        row.started_at = Some(now() + Duration::seconds(1));
        row.completed_at = Some(now() + Duration::seconds(3));

        let error = DecodedToolCall::try_from(row).expect_err("failed row needs reason");

        assert!(matches!(error, ToolCallError::MissingTerminalReason { .. }));
    }

    #[test]
    fn row_conversion_rejects_active_call_with_completed_at() {
        let mut row = row("validated");
        row.completed_at = Some(now() + Duration::seconds(1));

        let error = DecodedToolCall::try_from(row).expect_err("active row cannot be completed");

        assert!(matches!(error, ToolCallError::UnexpectedCompletedAt { .. }));
    }

    #[test]
    fn row_conversion_rejects_completed_before_started() {
        let mut row = row("executed");
        row.started_at = Some(now() + Duration::seconds(3));
        row.completed_at = Some(now() + Duration::seconds(1));
        row.output = Some(json!({ "event": "job kind paused" }));

        let error = DecodedToolCall::try_from(row).expect_err("invalid timeline");

        assert!(matches!(error, ToolCallError::CompletedBeforeStarted));
    }

    #[test]
    fn row_conversion_rejects_empty_tool_version() {
        let mut row = row("requested");
        row.tool_version = " ".to_string();

        let error = DecodedToolCall::try_from(row).expect_err("tool version is domain data");

        assert!(matches!(error, ToolCallError::Domain(_)));
    }

    #[test]
    fn tool_call_payload_debug_redacts_value() {
        assert_eq!(format!("{:?}", input()), "ToolCallInput([redacted])");
        assert_eq!(format!("{:?}", output()), "ToolCallOutput([redacted])");
    }
}
