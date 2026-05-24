//! Typed first-user launch packet for one agent job kind.
//!
//! A launch packet is the durable answer to "may this job kind touch real
//! users?" It ties readiness, release, failure-drill, rollback, recovery, and
//! operator evidence to one reviewed decision instead of leaving launch proof
//! in private notes.

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{DomainError, JobKind};
use crate::job_kind_readiness::{
    JobKindReadinessError, JobKindReadinessStatus, JobRiskClass, MaturityLevel,
};
use crate::release_gate::{ReleaseGateDecision, ReleaseGateError, ReleaseGateRunId};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LaunchPacketError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("unknown launch decision: {value:?}")]
    UnknownLaunchDecision { value: UnknownLaunchPacketValue },
    #[error("unknown launch status: {value:?}")]
    UnknownLaunchStatus { value: UnknownLaunchPacketValue },
    #[error("unknown failure drill status: {value:?}")]
    UnknownFailureDrillStatus { value: UnknownLaunchPacketValue },
    #[error("known_gaps must be a JSON array")]
    KnownGapsMustBeArray,
    #[error("known_gaps entries must be strings")]
    KnownGapMustBeString,
    #[error("known_gap_count cannot be negative, got {value}")]
    NegativeKnownGapCount { value: i64 },
    #[error("known_gap_count {query_count} does not match known_gaps length {actual_count}")]
    KnownGapCountMismatch { query_count: u64, actual_count: u64 },
    #[error("next_review_at must be after reviewed_at")]
    InvalidReviewWindow,
    #[error(
        "approved launch decisions require ready readiness, promoted release, and no known gaps"
    )]
    ApprovedLaunchMissingEvidence,
    #[error("high-risk launch decisions require a passed failure drill")]
    HighRiskLaunchRequiresPassedFailureDrill,
    #[error("launch status {status:?} conflicts with packet evidence")]
    InconsistentLaunchStatus { status: LaunchStatus },
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Readiness(#[from] JobKindReadinessError),
    #[error(transparent)]
    ReleaseGate(#[from] ReleaseGateError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownLaunchPacketValue(String);

impl UnknownLaunchPacketValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobKindLaunchPacketId(Uuid);

impl JobKindLaunchPacketId {
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobKindReadinessReviewId(Uuid);

impl JobKindReadinessReviewId {
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FailureDrillRunId(Uuid);

impl FailureDrillRunId {
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchDecision {
    Draft,
    Blocked,
    ApprovedForFirstUsers,
    Launched,
    Paused,
}

impl TryFrom<&str> for LaunchDecision {
    type Error = LaunchPacketError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "draft" => Ok(Self::Draft),
            "blocked" => Ok(Self::Blocked),
            "approved_for_first_users" => Ok(Self::ApprovedForFirstUsers),
            "launched" => Ok(Self::Launched),
            "paused" => Ok(Self::Paused),
            value => Err(LaunchPacketError::UnknownLaunchDecision {
                value: UnknownLaunchPacketValue::new(value),
            }),
        }
    }
}

impl LaunchDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Blocked => "blocked",
            Self::ApprovedForFirstUsers => "approved_for_first_users",
            Self::Launched => "launched",
            Self::Paused => "paused",
        }
    }

    fn claims_first_user_exposure(self) -> bool {
        matches!(self, Self::ApprovedForFirstUsers | Self::Launched)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchStatus {
    ReadyForFirstUsers,
    MissingEvidence,
    BlockedByGaps,
    ReviewOverdue,
    Paused,
}

impl TryFrom<&str> for LaunchStatus {
    type Error = LaunchPacketError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ready_for_first_users" => Ok(Self::ReadyForFirstUsers),
            "missing_evidence" => Ok(Self::MissingEvidence),
            "blocked_by_gaps" => Ok(Self::BlockedByGaps),
            "review_overdue" => Ok(Self::ReviewOverdue),
            "paused" => Ok(Self::Paused),
            value => Err(LaunchPacketError::UnknownLaunchStatus {
                value: UnknownLaunchPacketValue::new(value),
            }),
        }
    }
}

impl LaunchStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadyForFirstUsers => "ready_for_first_users",
            Self::MissingEvidence => "missing_evidence",
            Self::BlockedByGaps => "blocked_by_gaps",
            Self::ReviewOverdue => "review_overdue",
            Self::Paused => "paused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureDrillStatus {
    Planned,
    Running,
    Passed,
    Failed,
    Aborted,
}

impl TryFrom<&str> for FailureDrillStatus {
    type Error = LaunchPacketError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "planned" => Ok(Self::Planned),
            "running" => Ok(Self::Running),
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            "aborted" => Ok(Self::Aborted),
            value => Err(LaunchPacketError::UnknownFailureDrillStatus {
                value: UnknownLaunchPacketValue::new(value),
            }),
        }
    }
}

impl FailureDrillStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Running => "running",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Aborted => "aborted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewFreshness {
    Current,
    Overdue,
}

impl ReviewFreshness {
    fn from_review_overdue(review_overdue: bool) -> Self {
        if review_overdue {
            Self::Overdue
        } else {
            Self::Current
        }
    }

    fn is_overdue(self) -> bool {
        matches!(self, Self::Overdue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchOwner(String);

impl LaunchOwner {
    pub fn new(value: impl Into<String>) -> Result<Self, LaunchPacketError> {
        Ok(Self(non_empty_launch_text(value, "launch_owner")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchReviewer(String);

impl LaunchReviewer {
    pub fn new(value: impl Into<String>) -> Result<Self, LaunchPacketError> {
        Ok(Self(non_empty_launch_text(value, "launch_reviewer")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchEvidenceStatement {
    field: &'static str,
    value: String,
}

impl LaunchEvidenceStatement {
    pub fn new(value: impl Into<String>, field: &'static str) -> Result<Self, LaunchPacketError> {
        Ok(Self {
            field,
            value: non_empty_launch_text(value, field)?,
        })
    }

    pub fn field(&self) -> &'static str {
        self.field
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchEvidenceChecklist {
    durable_intake: LaunchEvidenceStatement,
    worker_ownership: LaunchEvidenceStatement,
    provider_boundary: LaunchEvidenceStatement,
    side_effect_control: LaunchEvidenceStatement,
    policy_or_approval: LaunchEvidenceStatement,
    observability: LaunchEvidenceStatement,
    evaluation: LaunchEvidenceStatement,
    security: LaunchEvidenceStatement,
    rollback_or_pause_plan: LaunchEvidenceStatement,
    restore_and_replay_note: LaunchEvidenceStatement,
}

impl LaunchEvidenceChecklist {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        durable_intake: String,
        worker_ownership: String,
        provider_boundary: String,
        side_effect_control: String,
        policy_or_approval: String,
        observability: String,
        evaluation: String,
        security: String,
        rollback_or_pause_plan: String,
        restore_and_replay_note: String,
    ) -> Result<Self, LaunchPacketError> {
        Ok(Self {
            durable_intake: LaunchEvidenceStatement::new(durable_intake, "durable_intake_proof")?,
            worker_ownership: LaunchEvidenceStatement::new(
                worker_ownership,
                "worker_ownership_proof",
            )?,
            provider_boundary: LaunchEvidenceStatement::new(
                provider_boundary,
                "provider_boundary_proof",
            )?,
            side_effect_control: LaunchEvidenceStatement::new(
                side_effect_control,
                "side_effect_control_proof",
            )?,
            policy_or_approval: LaunchEvidenceStatement::new(
                policy_or_approval,
                "policy_or_approval_proof",
            )?,
            observability: LaunchEvidenceStatement::new(observability, "observability_proof")?,
            evaluation: LaunchEvidenceStatement::new(evaluation, "evaluation_proof")?,
            security: LaunchEvidenceStatement::new(security, "security_proof")?,
            rollback_or_pause_plan: LaunchEvidenceStatement::new(
                rollback_or_pause_plan,
                "rollback_or_pause_plan",
            )?,
            restore_and_replay_note: LaunchEvidenceStatement::new(
                restore_and_replay_note,
                "restore_and_replay_note",
            )?,
        })
    }

    pub fn durable_intake(&self) -> &LaunchEvidenceStatement {
        &self.durable_intake
    }

    pub fn worker_ownership(&self) -> &LaunchEvidenceStatement {
        &self.worker_ownership
    }

    pub fn provider_boundary(&self) -> &LaunchEvidenceStatement {
        &self.provider_boundary
    }

    pub fn side_effect_control(&self) -> &LaunchEvidenceStatement {
        &self.side_effect_control
    }

    pub fn policy_or_approval(&self) -> &LaunchEvidenceStatement {
        &self.policy_or_approval
    }

    pub fn observability(&self) -> &LaunchEvidenceStatement {
        &self.observability
    }

    pub fn evaluation(&self) -> &LaunchEvidenceStatement {
        &self.evaluation
    }

    pub fn security(&self) -> &LaunchEvidenceStatement {
        &self.security
    }

    pub fn rollback_or_pause_plan(&self) -> &LaunchEvidenceStatement {
        &self.rollback_or_pause_plan
    }

    pub fn restore_and_replay_note(&self) -> &LaunchEvidenceStatement {
        &self.restore_and_replay_note
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchKnownGap(String);

impl LaunchKnownGap {
    pub fn new(value: impl Into<String>) -> Result<Self, LaunchPacketError> {
        Ok(Self(non_empty_launch_text(value, "known_gap")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchKnownGapCount(u64);

impl LaunchKnownGapCount {
    pub fn try_from_i64(value: i64) -> Result<Self, LaunchPacketError> {
        let value =
            u64::try_from(value).map_err(|_| LaunchPacketError::NegativeKnownGapCount { value })?;
        Ok(Self(value))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchKnownGaps(Vec<LaunchKnownGap>);

impl LaunchKnownGaps {
    pub fn from_json(value: Value) -> Result<Self, LaunchPacketError> {
        let gaps = value
            .as_array()
            .ok_or(LaunchPacketError::KnownGapsMustBeArray)?
            .iter()
            .map(|entry| {
                let gap = entry
                    .as_str()
                    .ok_or(LaunchPacketError::KnownGapMustBeString)?;
                LaunchKnownGap::new(gap)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(gaps))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn entries(&self) -> &[LaunchKnownGap] {
        &self.0
    }
}

// ANCHOR: launch_packet_row_boundary
#[derive(Debug, Clone, PartialEq)]
pub struct DbJobKindLaunchPacketStatusRow {
    pub id: Uuid,
    pub job_kind: String,
    pub target_level: String,
    pub risk_class: String,
    pub launch_decision: String,
    pub owner: String,
    pub durable_intake_proof: String,
    pub worker_ownership_proof: String,
    pub provider_boundary_proof: String,
    pub side_effect_control_proof: String,
    pub policy_or_approval_proof: String,
    pub observability_proof: String,
    pub evaluation_proof: String,
    pub security_proof: String,
    pub rollback_or_pause_plan: String,
    pub restore_and_replay_note: String,
    pub known_gaps: Value,
    pub known_gap_count: i64,
    pub readiness_review_id: Option<Uuid>,
    pub release_gate_run_id: Option<Uuid>,
    pub failure_drill_run_id: Option<Uuid>,
    pub reviewed_by: String,
    pub reviewed_at: DateTime<Utc>,
    pub next_review_at: DateTime<Utc>,
    pub review_overdue: bool,
    pub readiness_status: Option<String>,
    pub release_decision: Option<String>,
    pub failure_drill_status: Option<String>,
    pub launch_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobKindLaunchPacket {
    id: JobKindLaunchPacketId,
    job_kind: JobKind,
    target_level: MaturityLevel,
    risk_class: JobRiskClass,
    decision: LaunchDecision,
    owner: LaunchOwner,
    evidence: LaunchEvidenceChecklist,
    known_gaps: LaunchKnownGaps,
    readiness_review_id: Option<JobKindReadinessReviewId>,
    release_gate_run_id: Option<ReleaseGateRunId>,
    failure_drill_run_id: Option<FailureDrillRunId>,
    reviewed_by: LaunchReviewer,
    reviewed_at: DateTime<Utc>,
    next_review_at: DateTime<Utc>,
    review_freshness: ReviewFreshness,
    readiness_status: Option<JobKindReadinessStatus>,
    release_decision: Option<ReleaseGateDecision>,
    failure_drill_status: Option<FailureDrillStatus>,
    launch_status: LaunchStatus,
}

impl JobKindLaunchPacket {
    pub fn job_kind(&self) -> &JobKind {
        &self.job_kind
    }

    pub fn target_level(&self) -> MaturityLevel {
        self.target_level
    }

    pub fn risk_class(&self) -> JobRiskClass {
        self.risk_class
    }

    pub fn decision(&self) -> LaunchDecision {
        self.decision
    }

    pub fn status(&self) -> LaunchStatus {
        self.launch_status
    }

    pub fn evidence(&self) -> &LaunchEvidenceChecklist {
        &self.evidence
    }

    pub fn known_gaps(&self) -> &LaunchKnownGaps {
        &self.known_gaps
    }

    pub fn release_decision(&self) -> Option<ReleaseGateDecision> {
        self.release_decision
    }

    pub fn failure_drill_status(&self) -> Option<FailureDrillStatus> {
        self.failure_drill_status
    }
}

impl TryFrom<DbJobKindLaunchPacketStatusRow> for JobKindLaunchPacket {
    type Error = LaunchPacketError;

    fn try_from(row: DbJobKindLaunchPacketStatusRow) -> Result<Self, Self::Error> {
        if row.next_review_at <= row.reviewed_at {
            return Err(LaunchPacketError::InvalidReviewWindow);
        }

        let target_level = MaturityLevel::try_from(row.target_level.as_str())?;
        let risk_class = JobRiskClass::try_from(row.risk_class.as_str())?;
        let decision = LaunchDecision::try_from(row.launch_decision.as_str())?;
        let known_gaps = LaunchKnownGaps::from_json(row.known_gaps)?;
        let known_gap_count = LaunchKnownGapCount::try_from_i64(row.known_gap_count)?;
        let actual_count = known_gaps.len() as u64;
        if known_gap_count.get() != actual_count {
            return Err(LaunchPacketError::KnownGapCountMismatch {
                query_count: known_gap_count.get(),
                actual_count,
            });
        }

        let readiness_status = row
            .readiness_status
            .as_deref()
            .map(JobKindReadinessStatus::try_from)
            .transpose()?;
        let release_decision = row
            .release_decision
            .as_deref()
            .map(ReleaseGateDecision::try_from)
            .transpose()?;
        let failure_drill_status = row
            .failure_drill_status
            .as_deref()
            .map(FailureDrillStatus::try_from)
            .transpose()?;
        let launch_status = LaunchStatus::try_from(row.launch_status.as_str())?;
        let review_freshness = ReviewFreshness::from_review_overdue(row.review_overdue);

        validate_launch_packet_status(LaunchStatusEvidence {
            decision,
            risk_class,
            known_gaps: &known_gaps,
            review_freshness,
            readiness_status,
            release_decision,
            failure_drill_status,
            launch_status,
        })?;

        Ok(Self {
            id: JobKindLaunchPacketId::from_uuid(row.id),
            job_kind: JobKind::new(row.job_kind)?,
            target_level,
            risk_class,
            decision,
            owner: LaunchOwner::new(row.owner)?,
            evidence: LaunchEvidenceChecklist::new(
                row.durable_intake_proof,
                row.worker_ownership_proof,
                row.provider_boundary_proof,
                row.side_effect_control_proof,
                row.policy_or_approval_proof,
                row.observability_proof,
                row.evaluation_proof,
                row.security_proof,
                row.rollback_or_pause_plan,
                row.restore_and_replay_note,
            )?,
            known_gaps,
            readiness_review_id: row
                .readiness_review_id
                .map(JobKindReadinessReviewId::from_uuid),
            release_gate_run_id: row.release_gate_run_id.map(ReleaseGateRunId::from_uuid),
            failure_drill_run_id: row.failure_drill_run_id.map(FailureDrillRunId::from_uuid),
            reviewed_by: LaunchReviewer::new(row.reviewed_by)?,
            reviewed_at: row.reviewed_at,
            next_review_at: row.next_review_at,
            review_freshness,
            readiness_status,
            release_decision,
            failure_drill_status,
            launch_status,
        })
    }
}
// ANCHOR_END: launch_packet_row_boundary

fn non_empty_launch_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, LaunchPacketError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(LaunchPacketError::EmptyText { field });
    }

    Ok(value)
}

#[derive(Debug, Clone, Copy)]
struct LaunchStatusEvidence<'a> {
    decision: LaunchDecision,
    risk_class: JobRiskClass,
    known_gaps: &'a LaunchKnownGaps,
    review_freshness: ReviewFreshness,
    readiness_status: Option<JobKindReadinessStatus>,
    release_decision: Option<ReleaseGateDecision>,
    failure_drill_status: Option<FailureDrillStatus>,
    launch_status: LaunchStatus,
}

fn validate_launch_packet_status(
    evidence: LaunchStatusEvidence<'_>,
) -> Result<(), LaunchPacketError> {
    let decision = evidence.decision;
    let risk_class = evidence.risk_class;
    let known_gaps = evidence.known_gaps;
    let review_freshness = evidence.review_freshness;
    let readiness_status = evidence.readiness_status;
    let release_decision = evidence.release_decision;
    let failure_drill_status = evidence.failure_drill_status;
    let launch_status = evidence.launch_status;

    if decision.claims_first_user_exposure()
        && (!known_gaps.is_empty()
            || readiness_status != Some(JobKindReadinessStatus::ReadyForTarget)
            || release_decision != Some(ReleaseGateDecision::Promote))
    {
        return Err(LaunchPacketError::ApprovedLaunchMissingEvidence);
    }

    if decision.claims_first_user_exposure()
        && high_risk(risk_class)
        && failure_drill_status != Some(FailureDrillStatus::Passed)
    {
        return Err(LaunchPacketError::HighRiskLaunchRequiresPassedFailureDrill);
    }

    let ready_for_first_users = decision.claims_first_user_exposure()
        && known_gaps.is_empty()
        && readiness_status == Some(JobKindReadinessStatus::ReadyForTarget)
        && release_decision == Some(ReleaseGateDecision::Promote)
        && (!high_risk(risk_class) || failure_drill_status == Some(FailureDrillStatus::Passed));

    let valid = match launch_status {
        LaunchStatus::ReviewOverdue => review_freshness.is_overdue(),
        LaunchStatus::Paused => {
            !review_freshness.is_overdue() && decision == LaunchDecision::Paused
        }
        LaunchStatus::BlockedByGaps => {
            !review_freshness.is_overdue()
                && decision != LaunchDecision::Paused
                && (!known_gaps.is_empty() || decision == LaunchDecision::Blocked)
        }
        LaunchStatus::MissingEvidence => {
            !review_freshness.is_overdue()
                && decision != LaunchDecision::Paused
                && decision != LaunchDecision::Blocked
                && known_gaps.is_empty()
                && !ready_for_first_users
        }
        LaunchStatus::ReadyForFirstUsers => !review_freshness.is_overdue() && ready_for_first_users,
    };

    if valid {
        Ok(())
    } else {
        Err(LaunchPacketError::InconsistentLaunchStatus {
            status: launch_status,
        })
    }
}

fn high_risk(risk_class: JobRiskClass) -> bool {
    matches!(risk_class, JobRiskClass::High | JobRiskClass::Regulated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row() -> DbJobKindLaunchPacketStatusRow {
        DbJobKindLaunchPacketStatusRow {
            id: Uuid::new_v4(),
            job_kind: "incident_triage".to_string(),
            target_level: "production".to_string(),
            risk_class: "high".to_string(),
            launch_decision: "approved_for_first_users".to_string(),
            owner: "sre-oncall".to_string(),
            durable_intake_proof: "enqueue_agent_job.sql stores work before provider calls"
                .to_string(),
            worker_ownership_proof: "pick_due_job.sql uses lease owner and SKIP LOCKED".to_string(),
            provider_boundary_proof: "RawAgentOutput parses before AgentResult".to_string(),
            side_effect_control_proof: "tool calls require idempotency and receipts".to_string(),
            policy_or_approval_proof: "approval row exists for risky escalation".to_string(),
            observability_proof: "job_event_timeline.sql reconstructs the run".to_string(),
            evaluation_proof: "release gate references passing eval receipt".to_string(),
            security_proof: "tool execution gate denies unsafe authority".to_string(),
            rollback_or_pause_plan: "pause job kind and restore previous worker".to_string(),
            restore_and_replay_note: "restore_replay_candidates.sql quarantines unsafe replay"
                .to_string(),
            known_gaps: json!([]),
            known_gap_count: 0,
            readiness_review_id: Some(Uuid::new_v4()),
            release_gate_run_id: Some(Uuid::new_v4()),
            failure_drill_run_id: Some(Uuid::new_v4()),
            reviewed_by: "release-owner".to_string(),
            reviewed_at: Utc::now(),
            next_review_at: Utc::now() + chrono::Duration::days(30),
            review_overdue: false,
            readiness_status: Some("ready_for_target".to_string()),
            release_decision: Some("promote".to_string()),
            failure_drill_status: Some("passed".to_string()),
            launch_status: "ready_for_first_users".to_string(),
        }
    }

    #[test]
    fn launch_packet_decodes_ready_first_user_proof() {
        let packet = JobKindLaunchPacket::try_from(row()).expect("valid launch packet");

        assert_eq!(packet.job_kind().as_str(), "incident_triage");
        assert_eq!(packet.target_level(), MaturityLevel::Production);
        assert_eq!(packet.risk_class(), JobRiskClass::High);
        assert_eq!(packet.decision(), LaunchDecision::ApprovedForFirstUsers);
        assert_eq!(packet.status(), LaunchStatus::ReadyForFirstUsers);
        assert!(packet.known_gaps().is_empty());
        assert_eq!(
            packet.release_decision(),
            Some(ReleaseGateDecision::Promote)
        );
        assert_eq!(
            packet.failure_drill_status(),
            Some(FailureDrillStatus::Passed)
        );
    }

    #[test]
    fn approved_launch_rejects_known_gaps() {
        let mut row = row();
        row.known_gaps = json!(["restore drill has not run"]);
        row.known_gap_count = 1;
        row.launch_status = "blocked_by_gaps".to_string();

        let error = JobKindLaunchPacket::try_from(row).expect_err("known gap blocks launch");

        assert_eq!(error, LaunchPacketError::ApprovedLaunchMissingEvidence);
    }

    #[test]
    fn high_risk_launch_requires_passed_failure_drill() {
        let mut row = row();
        row.failure_drill_status = Some("failed".to_string());
        row.launch_status = "missing_evidence".to_string();

        let error = JobKindLaunchPacket::try_from(row).expect_err("failure drill not passed");

        assert_eq!(
            error,
            LaunchPacketError::HighRiskLaunchRequiresPassedFailureDrill
        );
    }

    #[test]
    fn paused_packet_can_have_known_gaps() {
        let mut row = row();
        row.launch_decision = "paused".to_string();
        row.known_gaps = json!(["provider quota incident under review"]);
        row.known_gap_count = 1;
        row.launch_status = "paused".to_string();

        let packet = JobKindLaunchPacket::try_from(row).expect("paused packet remains valid");

        assert_eq!(packet.decision(), LaunchDecision::Paused);
        assert_eq!(packet.status(), LaunchStatus::Paused);
        assert_eq!(packet.known_gaps().len(), 1);
    }

    #[test]
    fn review_overdue_status_requires_overdue_boundary_value() {
        let mut row = row();
        row.launch_decision = "draft".to_string();
        row.readiness_status = None;
        row.release_decision = None;
        row.failure_drill_status = None;
        row.launch_status = "review_overdue".to_string();
        row.review_overdue = false;

        let error = JobKindLaunchPacket::try_from(row).expect_err("not actually overdue");

        assert_eq!(
            error,
            LaunchPacketError::InconsistentLaunchStatus {
                status: LaunchStatus::ReviewOverdue,
            }
        );
    }

    #[test]
    fn known_gap_count_must_match_json_array() {
        let mut row = row();
        row.known_gaps = json!(["missing approval"]);
        row.known_gap_count = 0;
        row.launch_decision = "blocked".to_string();
        row.launch_status = "blocked_by_gaps".to_string();

        let error = JobKindLaunchPacket::try_from(row).expect_err("count mismatch");

        assert_eq!(
            error,
            LaunchPacketError::KnownGapCountMismatch {
                query_count: 0,
                actual_count: 1,
            }
        );
    }
}
