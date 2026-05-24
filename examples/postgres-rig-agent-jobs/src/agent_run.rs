//! Typed lifecycle for the `agent_runs` table.
//!
//! An agent run is the durable execution record for one model-backed workflow
//! attempt. The database stores text status and timestamps; this module turns
//! those raw row values into typed lifecycle states before other modules attach
//! steps, tool calls, approvals, or audit evidence.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::audit::{AuditError, TraceId};
use crate::background_job::BackgroundJobId;
use crate::domain::{AgentRunId, DomainError, PromptVersion};
use crate::scheduled_job::ScheduledJobId;
use crate::timeouts::{ExecutionDeadline, TimeoutAction, TimeoutError, TimeoutPolicyName};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AgentRunError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("unknown agent run lifecycle status: {value}")]
    UnknownLifecycleStatus {
        value: UnknownAgentRunLifecycleStatus,
    },
    #[error("agent run must reference a scheduled job, background job, or both")]
    MissingRunOwner,
    #[error("terminal agent run with status {status:?} requires finished_at")]
    MissingFinishedAt { status: AgentRunLifecycleStatus },
    #[error("non-terminal agent run with status {status:?} must not have finished_at")]
    UnexpectedFinishedAt { status: AgentRunLifecycleStatus },
    #[error("finished_at must be greater than or equal to started_at")]
    FinishedBeforeStarted,
    #[error("audit validation failed: {0}")]
    Audit(#[from] AuditError),
    #[error("timeout validation failed: {0}")]
    Timeout(#[from] TimeoutError),
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_agent_run_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, AgentRunError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(AgentRunError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAgentRunLifecycleStatus(String);

impl UnknownAgentRunLifecycleStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownAgentRunLifecycleStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunLifecycleStatus {
    Planning,
    Running,
    WaitingForHuman,
    Completed,
    Failed,
    Cancelled,
}

impl AgentRunLifecycleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Running => "running",
            Self::WaitingForHuman => "waiting_for_human",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

impl TryFrom<&str> for AgentRunLifecycleStatus {
    type Error = AgentRunError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "planning" => Ok(Self::Planning),
            "running" => Ok(Self::Running),
            "waiting_for_human" => Ok(Self::WaitingForHuman),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            value => Err(AgentRunError::UnknownLifecycleStatus {
                value: UnknownAgentRunLifecycleStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunName(String);

impl AgentRunName {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentRunError> {
        Ok(Self(non_empty_agent_run_text(value, "agent_name")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentModelVersion(String);

impl AgentModelVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentRunError> {
        Ok(Self(non_empty_agent_run_text(value, "model_version")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentRunRecord {
    id: AgentRunId,
    scheduled_job_id: Option<ScheduledJobId>,
    background_job_id: Option<BackgroundJobId>,
    agent_name: AgentRunName,
    prompt_version: PromptVersion,
    model_version: AgentModelVersion,
    trace_id: TraceId,
    deadline: Option<ExecutionDeadline>,
    timeout_policy_name: TimeoutPolicyName,
    timeout_action: TimeoutAction,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// ANCHOR: agent_run_typestate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunPlanning;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunRunning;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunWaitingForHuman;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunCompleted;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunFailed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunCancelled;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRun<State> {
    record: AgentRunRecord,
    state: PhantomData<State>,
}

impl AgentRun<AgentRunPlanning> {
    pub fn start(mut self, updated_at: DateTime<Utc>) -> AgentRun<AgentRunRunning> {
        self.record.updated_at = updated_at;
        AgentRun {
            record: self.record,
            state: PhantomData,
        }
    }
}

impl AgentRun<AgentRunRunning> {
    pub fn wait_for_human(
        mut self,
        updated_at: DateTime<Utc>,
    ) -> AgentRun<AgentRunWaitingForHuman> {
        self.record.updated_at = updated_at;
        AgentRun {
            record: self.record,
            state: PhantomData,
        }
    }

    pub fn complete(
        mut self,
        finished_at: DateTime<Utc>,
    ) -> Result<AgentRun<AgentRunCompleted>, AgentRunError> {
        validate_finished_at(self.record.started_at, finished_at)?;
        self.record.finished_at = Some(finished_at);
        self.record.updated_at = finished_at;
        Ok(AgentRun {
            record: self.record,
            state: PhantomData,
        })
    }

    pub fn fail(
        mut self,
        finished_at: DateTime<Utc>,
    ) -> Result<AgentRun<AgentRunFailed>, AgentRunError> {
        validate_finished_at(self.record.started_at, finished_at)?;
        self.record.finished_at = Some(finished_at);
        self.record.updated_at = finished_at;
        Ok(AgentRun {
            record: self.record,
            state: PhantomData,
        })
    }

    pub fn cancel(
        mut self,
        finished_at: DateTime<Utc>,
    ) -> Result<AgentRun<AgentRunCancelled>, AgentRunError> {
        validate_finished_at(self.record.started_at, finished_at)?;
        self.record.finished_at = Some(finished_at);
        self.record.updated_at = finished_at;
        Ok(AgentRun {
            record: self.record,
            state: PhantomData,
        })
    }
}

impl AgentRun<AgentRunWaitingForHuman> {
    pub fn resume(mut self, updated_at: DateTime<Utc>) -> AgentRun<AgentRunRunning> {
        self.record.updated_at = updated_at;
        AgentRun {
            record: self.record,
            state: PhantomData,
        }
    }

    pub fn complete(
        mut self,
        finished_at: DateTime<Utc>,
    ) -> Result<AgentRun<AgentRunCompleted>, AgentRunError> {
        validate_finished_at(self.record.started_at, finished_at)?;
        self.record.finished_at = Some(finished_at);
        self.record.updated_at = finished_at;
        Ok(AgentRun {
            record: self.record,
            state: PhantomData,
        })
    }

    pub fn fail(
        mut self,
        finished_at: DateTime<Utc>,
    ) -> Result<AgentRun<AgentRunFailed>, AgentRunError> {
        validate_finished_at(self.record.started_at, finished_at)?;
        self.record.finished_at = Some(finished_at);
        self.record.updated_at = finished_at;
        Ok(AgentRun {
            record: self.record,
            state: PhantomData,
        })
    }

    pub fn cancel(
        mut self,
        finished_at: DateTime<Utc>,
    ) -> Result<AgentRun<AgentRunCancelled>, AgentRunError> {
        validate_finished_at(self.record.started_at, finished_at)?;
        self.record.finished_at = Some(finished_at);
        self.record.updated_at = finished_at;
        Ok(AgentRun {
            record: self.record,
            state: PhantomData,
        })
    }
}
// ANCHOR_END: agent_run_typestate

impl<State> AgentRun<State> {
    pub fn id(&self) -> AgentRunId {
        self.record.id
    }

    pub fn scheduled_job_id(&self) -> Option<ScheduledJobId> {
        self.record.scheduled_job_id
    }

    pub fn background_job_id(&self) -> Option<BackgroundJobId> {
        self.record.background_job_id
    }

    pub fn agent_name(&self) -> &AgentRunName {
        &self.record.agent_name
    }

    pub fn prompt_version(&self) -> &PromptVersion {
        &self.record.prompt_version
    }

    pub fn model_version(&self) -> &AgentModelVersion {
        &self.record.model_version
    }

    pub fn trace_id(&self) -> &TraceId {
        &self.record.trace_id
    }

    pub fn deadline(&self) -> Option<ExecutionDeadline> {
        self.record.deadline
    }

    pub fn timeout_policy_name(&self) -> &TimeoutPolicyName {
        &self.record.timeout_policy_name
    }

    pub fn timeout_action(&self) -> TimeoutAction {
        self.record.timeout_action
    }

    pub fn started_at(&self) -> DateTime<Utc> {
        self.record.started_at
    }

    pub fn finished_at(&self) -> Option<DateTime<Utc>> {
        self.record.finished_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.record.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.record.updated_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedAgentRun {
    Planning(AgentRun<AgentRunPlanning>),
    Running(AgentRun<AgentRunRunning>),
    WaitingForHuman(AgentRun<AgentRunWaitingForHuman>),
    Completed(AgentRun<AgentRunCompleted>),
    Failed(AgentRun<AgentRunFailed>),
    Cancelled(AgentRun<AgentRunCancelled>),
}

// ANCHOR: agent_run_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbAgentRunRow {
    pub id: Uuid,
    pub scheduled_job_id: Option<Uuid>,
    pub background_job_id: Option<Uuid>,
    pub agent_name: String,
    pub lifecycle_status: String,
    pub prompt_version: String,
    pub model_version: String,
    pub trace_id: String,
    pub deadline_at: Option<DateTime<Utc>>,
    pub timeout_policy_name: String,
    pub timeout_action: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<DbAgentRunRow> for DecodedAgentRun {
    type Error = AgentRunError;

    fn try_from(row: DbAgentRunRow) -> Result<Self, Self::Error> {
        let status = AgentRunLifecycleStatus::try_from(row.lifecycle_status.as_str())?;
        let record = AgentRunRecord::try_from_row(row, status)?;

        Ok(match status {
            AgentRunLifecycleStatus::Planning => Self::Planning(AgentRun {
                record,
                state: PhantomData,
            }),
            AgentRunLifecycleStatus::Running => Self::Running(AgentRun {
                record,
                state: PhantomData,
            }),
            AgentRunLifecycleStatus::WaitingForHuman => Self::WaitingForHuman(AgentRun {
                record,
                state: PhantomData,
            }),
            AgentRunLifecycleStatus::Completed => Self::Completed(AgentRun {
                record,
                state: PhantomData,
            }),
            AgentRunLifecycleStatus::Failed => Self::Failed(AgentRun {
                record,
                state: PhantomData,
            }),
            AgentRunLifecycleStatus::Cancelled => Self::Cancelled(AgentRun {
                record,
                state: PhantomData,
            }),
        })
    }
}
// ANCHOR_END: agent_run_row_boundary

impl AgentRunRecord {
    fn try_from_row(
        row: DbAgentRunRow,
        status: AgentRunLifecycleStatus,
    ) -> Result<Self, AgentRunError> {
        if row.scheduled_job_id.is_none() && row.background_job_id.is_none() {
            return Err(AgentRunError::MissingRunOwner);
        }

        validate_finished_evidence(status, row.started_at, row.finished_at)?;

        let deadline = row
            .deadline_at
            .map(|deadline_at| ExecutionDeadline::new(row.started_at, deadline_at))
            .transpose()?;

        Ok(Self {
            id: AgentRunId::from_uuid(row.id),
            scheduled_job_id: row.scheduled_job_id.map(ScheduledJobId::from_uuid),
            background_job_id: row.background_job_id.map(BackgroundJobId::from_uuid),
            agent_name: AgentRunName::new(row.agent_name)?,
            prompt_version: PromptVersion::new(row.prompt_version)?,
            model_version: AgentModelVersion::new(row.model_version)?,
            trace_id: TraceId::new(row.trace_id)?,
            deadline,
            timeout_policy_name: TimeoutPolicyName::new(row.timeout_policy_name)?,
            timeout_action: TimeoutAction::try_from(row.timeout_action.as_str())?,
            started_at: row.started_at,
            finished_at: row.finished_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn validate_finished_evidence(
    status: AgentRunLifecycleStatus,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
) -> Result<(), AgentRunError> {
    match (status.is_terminal(), finished_at) {
        (true, Some(finished_at)) => validate_finished_at(started_at, finished_at),
        (true, None) => Err(AgentRunError::MissingFinishedAt { status }),
        (false, Some(_)) => Err(AgentRunError::UnexpectedFinishedAt { status }),
        (false, None) => Ok(()),
    }
}

fn validate_finished_at(
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
) -> Result<(), AgentRunError> {
    if finished_at < started_at {
        return Err(AgentRunError::FinishedBeforeStarted);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn row_with_status(status: &str) -> DbAgentRunRow {
        let started_at = Utc::now();
        DbAgentRunRow {
            id: Uuid::new_v4(),
            scheduled_job_id: Some(Uuid::new_v4()),
            background_job_id: Some(Uuid::new_v4()),
            agent_name: "incident-triage-agent".to_string(),
            lifecycle_status: status.to_string(),
            prompt_version: "incident-triage:v1".to_string(),
            model_version: "deepseek-v4-flash:2026-05".to_string(),
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
            deadline_at: Some(started_at + Duration::minutes(5)),
            timeout_policy_name: "standard-agent:v1".to_string(),
            timeout_action: "schedule_retry".to_string(),
            started_at,
            finished_at: None,
            created_at: started_at,
            updated_at: started_at,
        }
    }

    #[test]
    fn row_conversion_accepts_running_run_with_owner_and_trace() {
        let decoded = DecodedAgentRun::try_from(row_with_status("running")).expect("valid row");

        let DecodedAgentRun::Running(run) = decoded else {
            panic!("expected running run");
        };

        assert_eq!(run.agent_name().as_str(), "incident-triage-agent");
        assert_eq!(run.trace_id().as_str(), "4bf92f3577b34da6a3ce929d0e0e4736");
    }

    #[test]
    fn planning_run_can_start_and_complete_after_running() {
        let decoded = DecodedAgentRun::try_from(row_with_status("planning")).expect("valid row");
        let DecodedAgentRun::Planning(planning) = decoded else {
            panic!("expected planning run");
        };

        let running = planning.start(Utc::now());
        let completed = running
            .complete(Utc::now() + Duration::seconds(1))
            .expect("completion after start");

        assert!(completed.finished_at().is_some());
    }

    #[test]
    fn running_run_can_wait_for_human_and_resume() {
        let decoded = DecodedAgentRun::try_from(row_with_status("running")).expect("valid row");
        let DecodedAgentRun::Running(running) = decoded else {
            panic!("expected running run");
        };

        let waiting = running.wait_for_human(Utc::now());
        let resumed = waiting.resume(Utc::now());

        assert!(resumed.finished_at().is_none());
    }

    #[test]
    fn row_conversion_accepts_terminal_run_with_finished_at() {
        let mut row = row_with_status("completed");
        row.finished_at = Some(row.started_at + Duration::seconds(10));

        let decoded = DecodedAgentRun::try_from(row).expect("valid terminal row");

        assert!(matches!(decoded, DecodedAgentRun::Completed(_)));
    }

    #[test]
    fn row_conversion_rejects_unknown_status() {
        let row = row_with_status("paused_in_model");

        assert!(matches!(
            DecodedAgentRun::try_from(row),
            Err(AgentRunError::UnknownLifecycleStatus { .. })
        ));
    }

    #[test]
    fn row_conversion_rejects_orphan_run() {
        let mut row = row_with_status("running");
        row.scheduled_job_id = None;
        row.background_job_id = None;

        assert!(matches!(
            DecodedAgentRun::try_from(row),
            Err(AgentRunError::MissingRunOwner)
        ));
    }

    #[test]
    fn row_conversion_rejects_terminal_without_finished_at() {
        let row = row_with_status("failed");

        assert!(matches!(
            DecodedAgentRun::try_from(row),
            Err(AgentRunError::MissingFinishedAt {
                status: AgentRunLifecycleStatus::Failed
            })
        ));
    }

    #[test]
    fn row_conversion_rejects_non_terminal_with_finished_at() {
        let mut row = row_with_status("waiting_for_human");
        row.finished_at = Some(row.started_at + Duration::seconds(10));

        assert!(matches!(
            DecodedAgentRun::try_from(row),
            Err(AgentRunError::UnexpectedFinishedAt {
                status: AgentRunLifecycleStatus::WaitingForHuman
            })
        ));
    }

    #[test]
    fn row_conversion_rejects_finished_before_started() {
        let mut row = row_with_status("cancelled");
        row.finished_at = Some(row.started_at - Duration::seconds(1));

        assert!(matches!(
            DecodedAgentRun::try_from(row),
            Err(AgentRunError::FinishedBeforeStarted)
        ));
    }

    #[test]
    fn row_conversion_rejects_deadline_before_started() {
        let mut row = row_with_status("running");
        row.deadline_at = Some(row.started_at - Duration::seconds(1));

        assert!(matches!(
            DecodedAgentRun::try_from(row),
            Err(AgentRunError::Timeout(TimeoutError::DeadlineNotAfterStart))
        ));
    }

    #[test]
    fn row_conversion_rejects_invalid_trace_id() {
        let mut row = row_with_status("running");
        row.trace_id = "not-a-trace".to_string();

        assert!(matches!(
            DecodedAgentRun::try_from(row),
            Err(AgentRunError::Audit(AuditError::InvalidTraceId { .. }))
        ));
    }

    #[test]
    fn row_conversion_rejects_empty_model_version() {
        let mut row = row_with_status("running");
        row.model_version = "  ".to_string();

        assert!(matches!(
            DecodedAgentRun::try_from(row),
            Err(AgentRunError::EmptyText {
                field: "model_version"
            })
        ));
    }
}
