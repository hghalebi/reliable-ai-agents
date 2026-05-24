use std::num::NonZeroU32;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} must be positive, got {value}")]
    NonPositiveNumber { field: &'static str, value: i64 },
    #[error("unknown job status: {value}")]
    UnknownJobStatus { value: InvalidJobStatus },
    #[error("unknown job event type: {value}")]
    UnknownJobEventType { value: InvalidJobEventType },
}

fn non_empty_text(value: impl Into<String>, field: &'static str) -> Result<String, DomainError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(DomainError::EmptyText { field });
    }
    Ok(value)
}

// ANCHOR: types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(Uuid);

impl JobId {
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

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentRunId(Uuid);

impl AgentRunId {
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

impl Default for AgentRunId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerId(String);

impl WorkerId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "worker_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantKey(String);

impl TenantKey {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "tenant_key")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobKind(String);

impl JobKind {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "job_kind")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadSchemaVersion(NonZeroU32);

impl PayloadSchemaVersion {
    pub fn new(value: NonZeroU32) -> Self {
        Self(value)
    }

    pub fn try_from_u32(value: u32) -> Result<Self, DomainError> {
        let Some(value) = NonZeroU32::new(value) else {
            return Err(DomainError::NonPositiveNumber {
                field: "payload_schema_version",
                value: i64::from(value),
            });
        };

        Ok(Self(value))
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl Default for PayloadSchemaVersion {
    fn default() -> Self {
        Self(NonZeroU32::MIN)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptVersion(String);

impl PromptVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "prompt_version")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for PromptVersion {
    fn default() -> Self {
        Self("incident-triage:v1".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRoute(String);

impl ModelRoute {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "model_route")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ModelRoute {
    fn default() -> Self {
        Self("deterministic-local:v1".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolVersion(String);

impl ToolVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "tool_version")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ToolVersion {
    fn default() -> Self {
        Self("no-tools:v1".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyVersion(String);

impl PolicyVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "policy_version")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for PolicyVersion {
    fn default() -> Self {
        Self("approval-policy:v1".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerBuildId(String);

impl WorkerBuildId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "worker_build_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for WorkerBuildId {
    fn default() -> Self {
        Self("local-dev".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AgentJobVersions {
    pub payload_schema: PayloadSchemaVersion,
    pub prompt: PromptVersion,
    pub model_route: ModelRoute,
    pub tool: ToolVersion,
    pub policy: PolicyVersion,
    pub worker_build: WorkerBuildId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "idempotency_key")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseUrl(String);

impl DatabaseUrl {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "database_url")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMessage(String);

impl EventMessage {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "event_message")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<FailureMessage> for EventMessage {
    fn from(value: FailureMessage) -> Self {
        Self(value.0)
    }
}

impl From<CancellationReason> for EventMessage {
    fn from(value: CancellationReason) -> Self {
        Self(value.0)
    }
}

impl From<&WorkerId> for EventMessage {
    fn from(value: &WorkerId) -> Self {
        Self(value.0.clone())
    }
}

impl From<&IdempotencyKey> for EventMessage {
    fn from(value: &IdempotencyKey) -> Self {
        Self(value.0.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureMessage(String);

impl FailureMessage {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "failure_message")?))
    }

    pub(crate) fn from_error_text(value: impl Into<String>) -> Self {
        let value = value.into();
        if value.trim().is_empty() {
            return Self("unknown failure".to_string());
        }

        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FailureMessage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancellationReason(String);

impl CancellationReason {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "cancellation_reason")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<CancellationReason> for FailureMessage {
    fn from(value: CancellationReason) -> Self {
        Self(value.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShutdownReason(String);

impl ShutdownReason {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "shutdown_reason")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidJobStatus(String);

impl InvalidJobStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for InvalidJobStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidJobEventType(String);

impl InvalidJobEventType {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for InvalidJobEventType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInstruction(String);

impl AgentInstruction {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "agent_instruction")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSummary(String);

impl AgentSummary {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "agent_summary")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextAction(String);

impl NextAction {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_text(value, "next_action")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalRequirement {
    NotRequired,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Dead,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Dead => "dead",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for JobStatus {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "dead" => Ok(Self::Dead),
            "cancelled" => Ok(Self::Cancelled),
            value => Err(DomainError::UnknownJobStatus {
                value: InvalidJobStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPayload {
    pub instruction: AgentInstruction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentResult {
    pub summary: AgentSummary,
    pub next_action: NextAction,
    pub approval: ApprovalRequirement,
}

impl AgentResult {
    pub fn new(
        summary: AgentSummary,
        next_action: NextAction,
        approval: ApprovalRequirement,
    ) -> Self {
        Self {
            summary,
            next_action,
            approval,
        }
    }
}
// ANCHOR_END: types

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaxAttempts(NonZeroU32);

impl MaxAttempts {
    pub fn new(value: NonZeroU32) -> Self {
        Self(value)
    }

    pub fn try_from_u32(value: u32) -> Result<Self, DomainError> {
        let Some(value) = NonZeroU32::new(value) else {
            return Err(DomainError::NonPositiveNumber {
                field: "max_attempts",
                value: i64::from(value),
            });
        };

        Ok(Self(value))
    }

    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl Default for MaxAttempts {
    fn default() -> Self {
        Self(NonZeroU32::MIN.saturating_add(4))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AttemptCount(u32);

impl AttemptCount {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn try_from_u32(value: u32) -> Result<Self, DomainError> {
        Ok(Self(value))
    }

    pub fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryDelay(Duration);

impl RetryDelay {
    pub fn from_secs(seconds: u64) -> Self {
        Self(Duration::from_secs(seconds))
    }

    pub fn duration(self) -> Duration {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseDuration(Duration);

impl LeaseDuration {
    pub fn from_secs(seconds: u64) -> Self {
        Self(Duration::from_secs(seconds))
    }

    pub fn duration(self) -> Duration {
        self.0
    }
}

impl Default for LeaseDuration {
    fn default() -> Self {
        Self::from_secs(300)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeartbeatInterval(Duration);

impl HeartbeatInterval {
    pub fn from_secs(seconds: u64) -> Result<Self, DomainError> {
        if seconds == 0 {
            return Err(DomainError::NonPositiveNumber {
                field: "heartbeat_interval_seconds",
                value: 0,
            });
        }

        Ok(Self(Duration::from_secs(seconds)))
    }

    pub fn every_minute() -> Self {
        Self(Duration::from_secs(60))
    }

    pub fn duration(self) -> Duration {
        self.0
    }
}

impl Default for HeartbeatInterval {
    fn default() -> Self {
        Self::every_minute()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    base_delay: RetryDelay,
    max_delay: RetryDelay,
}

impl RetryPolicy {
    pub fn new(base_delay: RetryDelay, max_delay: RetryDelay) -> Self {
        Self {
            base_delay,
            max_delay,
        }
    }

    pub fn delay_for_attempt(self, attempt_count: AttemptCount) -> RetryDelay {
        let exponent = attempt_count.get().saturating_sub(1).min(10);
        let base = self.base_delay.duration().as_secs();
        let max = self.max_delay.duration().as_secs();
        RetryDelay::from_secs(base.saturating_mul(2_u64.pow(exponent)).min(max))
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(RetryDelay::from_secs(30), RetryDelay::from_secs(300))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobEventType {
    JobEnqueued,
    DuplicateSuppressed,
    JobPicked,
    AgentStarted,
    AgentSucceeded,
    AgentFailed,
    RetryScheduled,
    JobSucceeded,
    JobDead,
    LeaseExtended,
    JobCancelled,
    ExpiredLeaseRecovered,
}

impl JobEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::JobEnqueued => "job_enqueued",
            Self::DuplicateSuppressed => "duplicate_suppressed",
            Self::JobPicked => "job_picked",
            Self::AgentStarted => "agent_started",
            Self::AgentSucceeded => "agent_succeeded",
            Self::AgentFailed => "agent_failed",
            Self::RetryScheduled => "retry_scheduled",
            Self::JobSucceeded => "job_succeeded",
            Self::JobDead => "job_dead",
            Self::LeaseExtended => "lease_extended",
            Self::JobCancelled => "job_cancelled",
            Self::ExpiredLeaseRecovered => "expired_lease_recovered",
        }
    }
}

impl TryFrom<&str> for JobEventType {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "job_enqueued" => Ok(Self::JobEnqueued),
            "duplicate_suppressed" => Ok(Self::DuplicateSuppressed),
            "job_picked" => Ok(Self::JobPicked),
            "agent_started" => Ok(Self::AgentStarted),
            "agent_succeeded" => Ok(Self::AgentSucceeded),
            "agent_failed" => Ok(Self::AgentFailed),
            "retry_scheduled" => Ok(Self::RetryScheduled),
            "job_succeeded" => Ok(Self::JobSucceeded),
            "job_dead" => Ok(Self::JobDead),
            "lease_extended" => Ok(Self::LeaseExtended),
            "job_cancelled" => Ok(Self::JobCancelled),
            "expired_lease_recovered" => Ok(Self::ExpiredLeaseRecovered),
            value => Err(DomainError::UnknownJobEventType {
                value: InvalidJobEventType::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDisposition {
    Retryable,
    Permanent,
}

impl RetryDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Retryable => "retryable",
            Self::Permanent => "permanent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseExtensionOutcome {
    Extended,
    NotOwnedOrNotRunning,
}

impl LeaseExtensionOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Extended => "extended",
            Self::NotOwnedOrNotRunning => "not_owned_or_not_running",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobTransition {
    Complete,
    RetryOrDead,
}

impl JobTransition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::RetryOrDead => "retry_or_dead",
        }
    }
}

impl std::fmt::Display for JobTransition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionTransitionOutcome {
    Succeeded,
    NotOwnedOrNotRunning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryTransitionOutcome {
    RetryScheduled,
    Dead,
    NotOwnedOrNotRunning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancellationOutcome {
    Cancelled,
    NotCancellable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEvent {
    pub job_id: JobId,
    pub event_type: JobEventType,
    pub message: Option<EventMessage>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentJob {
    pub id: JobId,
    pub kind: JobKind,
    pub idempotency_key: Option<IdempotencyKey>,
    pub versions: AgentJobVersions,
    pub status: JobStatus,
    pub payload: AgentPayload,
    pub result: Option<AgentResult>,
    pub run_at: DateTime<Utc>,
    pub attempt_count: AttemptCount,
    pub max_attempts: MaxAttempts,
    pub locked_by: Option<WorkerId>,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_error: Option<FailureMessage>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentJobSnapshot {
    id: JobId,
    status: JobStatus,
    run_at: DateTime<Utc>,
}

impl AgentJobSnapshot {
    pub fn new(id: JobId, status: JobStatus, run_at: DateTime<Utc>) -> Self {
        Self { id, status, run_at }
    }

    pub fn id(self) -> JobId {
        self.id
    }

    pub fn status(self) -> JobStatus {
        self.status
    }

    pub fn run_at(self) -> DateTime<Utc> {
        self.run_at
    }
}

impl From<&AgentJob> for AgentJobSnapshot {
    fn from(value: &AgentJob) -> Self {
        Self::new(value.id, value.status, value.run_at)
    }
}

impl AgentJob {
    pub fn new(
        kind: JobKind,
        payload: AgentPayload,
        max_attempts: MaxAttempts,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: JobId::new(),
            kind,
            idempotency_key: None,
            versions: AgentJobVersions::default(),
            status: JobStatus::Pending,
            payload,
            result: None,
            run_at: now,
            attempt_count: AttemptCount::zero(),
            max_attempts,
            locked_by: None,
            locked_until: None,
            last_error: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_idempotency_key(mut self, idempotency_key: IdempotencyKey) -> Self {
        self.idempotency_key = Some(idempotency_key);
        self
    }

    pub fn with_versions(mut self, versions: AgentJobVersions) -> Self {
        self.versions = versions;
        self
    }

    pub fn with_run_at(mut self, run_at: DateTime<Utc>) -> Self {
        self.run_at = run_at;
        self
    }

    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.status == JobStatus::Pending && self.run_at <= now
    }

    pub fn lease_to(
        &mut self,
        worker_id: WorkerId,
        locked_until: DateTime<Utc>,
        now: DateTime<Utc>,
    ) {
        self.status = JobStatus::Running;
        self.locked_by = Some(worker_id);
        self.locked_until = Some(locked_until);
        self.attempt_count = self.attempt_count.increment();
        self.updated_at = now;
    }

    pub fn mark_succeeded(
        &mut self,
        worker_id: &WorkerId,
        result: AgentResult,
        now: DateTime<Utc>,
    ) -> CompletionTransitionOutcome {
        if self.status != JobStatus::Running || self.locked_by.as_ref() != Some(worker_id) {
            return CompletionTransitionOutcome::NotOwnedOrNotRunning;
        }

        self.status = JobStatus::Succeeded;
        self.result = Some(result);
        self.locked_by = None;
        self.locked_until = None;
        self.last_error = None;
        self.updated_at = now;
        CompletionTransitionOutcome::Succeeded
    }

    pub fn retry_or_dead(
        &mut self,
        worker_id: &WorkerId,
        error: FailureMessage,
        retry_policy: RetryPolicy,
        retry_disposition: RetryDisposition,
        now: DateTime<Utc>,
    ) -> RetryTransitionOutcome {
        if self.status != JobStatus::Running || self.locked_by.as_ref() != Some(worker_id) {
            return RetryTransitionOutcome::NotOwnedOrNotRunning;
        }

        self.last_error = Some(error);
        self.locked_by = None;
        self.locked_until = None;
        self.updated_at = now;

        if retry_disposition == RetryDisposition::Permanent
            || self.attempt_count.get() >= self.max_attempts.get()
        {
            self.status = JobStatus::Dead;
            return RetryTransitionOutcome::Dead;
        }

        let retry_at = now
            + retry_policy
                .delay_for_attempt(self.attempt_count)
                .duration();
        self.status = JobStatus::Pending;
        self.run_at = retry_at;
        RetryTransitionOutcome::RetryScheduled
    }

    pub fn extend_lease(
        &mut self,
        worker_id: &WorkerId,
        locked_until: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> LeaseExtensionOutcome {
        if self.status != JobStatus::Running || self.locked_by.as_ref() != Some(worker_id) {
            return LeaseExtensionOutcome::NotOwnedOrNotRunning;
        }

        self.locked_until = Some(locked_until);
        self.updated_at = now;
        LeaseExtensionOutcome::Extended
    }

    pub fn cancel(
        &mut self,
        reason: CancellationReason,
        now: DateTime<Utc>,
    ) -> CancellationOutcome {
        if matches!(
            self.status,
            JobStatus::Succeeded | JobStatus::Dead | JobStatus::Cancelled
        ) {
            return CancellationOutcome::NotCancellable;
        }

        self.status = JobStatus::Cancelled;
        self.locked_by = None;
        self.locked_until = None;
        self.last_error = Some(reason.into());
        self.updated_at = now;
        CancellationOutcome::Cancelled
    }

    pub fn recover_expired_lease(&mut self, now: DateTime<Utc>) -> LeaseExtensionOutcome {
        if self.status != JobStatus::Running {
            return LeaseExtensionOutcome::NotOwnedOrNotRunning;
        }

        let Some(locked_until) = self.locked_until else {
            return LeaseExtensionOutcome::NotOwnedOrNotRunning;
        };

        if locked_until >= now {
            return LeaseExtensionOutcome::NotOwnedOrNotRunning;
        }

        self.status = JobStatus::Pending;
        self.locked_by = None;
        self.locked_until = None;
        self.run_at = now;
        self.updated_at = now;
        LeaseExtensionOutcome::Extended
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QueueMetrics {
    pub pending: QueueDepth,
    pub running: QueueDepth,
    pub succeeded: QueueDepth,
    pub failed: QueueDepth,
    pub dead: QueueDepth,
    pub cancelled: QueueDepth,
    pub oldest_pending_age: Option<QueueAge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QueueDepth(usize);

impl QueueDepth {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn try_from_usize(value: usize) -> Result<Self, DomainError> {
        Ok(Self(value))
    }

    pub fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    pub fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueAge(i64);

impl QueueAge {
    pub fn from_seconds(seconds: i64) -> Result<Self, DomainError> {
        if seconds < 0 {
            return Err(DomainError::NonPositiveNumber {
                field: "queue_age_seconds",
                value: seconds,
            });
        }

        Ok(Self(seconds))
    }

    pub fn saturating_from_seconds(seconds: i64) -> Self {
        Self(seconds.max(0))
    }

    pub fn seconds(self) -> i64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_instruction_rejects_empty_text() {
        let error = AgentInstruction::new("  ").expect_err("empty text must fail");
        assert_eq!(
            error,
            DomainError::EmptyText {
                field: "agent_instruction"
            }
        );
    }

    #[test]
    fn retry_policy_caps_exponential_backoff() {
        let policy = RetryPolicy::new(RetryDelay::from_secs(10), RetryDelay::from_secs(60));
        assert_eq!(
            policy
                .delay_for_attempt(AttemptCount::try_from_u32(1).expect("valid attempt count"))
                .duration(),
            Duration::from_secs(10)
        );
        assert_eq!(
            policy
                .delay_for_attempt(AttemptCount::try_from_u32(2).expect("valid attempt count"))
                .duration(),
            Duration::from_secs(20)
        );
        assert_eq!(
            policy
                .delay_for_attempt(AttemptCount::try_from_u32(5).expect("valid attempt count"))
                .duration(),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn job_defaults_include_replay_versions() {
        let now = Utc::now();
        let job = AgentJob::new(
            JobKind::new("incident_triage").expect("valid kind"),
            AgentPayload {
                instruction: AgentInstruction::new("Analyze failed deployment")
                    .expect("valid instruction"),
            },
            MaxAttempts::default(),
            now,
        );

        assert_eq!(job.versions.payload_schema.get(), 1);
        assert_eq!(job.versions.prompt.as_str(), "incident-triage:v1");
        assert_eq!(job.versions.model_route.as_str(), "deterministic-local:v1");
        assert_eq!(job.versions.tool.as_str(), "no-tools:v1");
        assert_eq!(job.versions.policy.as_str(), "approval-policy:v1");
        assert_eq!(job.versions.worker_build.as_str(), "local-dev");
    }

    #[test]
    fn version_fields_reject_empty_text() {
        let error = PromptVersion::new(" ").expect_err("empty prompt version must fail");

        assert_eq!(
            error,
            DomainError::EmptyText {
                field: "prompt_version"
            }
        );
    }
}
