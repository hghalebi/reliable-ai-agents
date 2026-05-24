//! Typed fault-tolerance readiness review for long-running agent systems.
//!
//! The SQL row is a database boundary. It may contain raw text, raw counts, and
//! optional joins. This module turns that evidence into explicit domain values
//! before the application uses it for launch, operations, or incident review.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::{DomainError, JobKind, ModelRoute, PolicyVersion, PromptVersion};
use crate::release_gate::{ReleaseGateDecision, ReleaseGateError};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FaultToleranceError {
    #[error("unknown control plane status: {value:?}")]
    UnknownControlPlaneStatus { value: UnknownFaultToleranceValue },
    #[error("unknown execution plane status: {value:?}")]
    UnknownExecutionPlaneStatus { value: UnknownFaultToleranceValue },
    #[error("unknown static stability mode: {value:?}")]
    UnknownStaticStabilityMode { value: UnknownFaultToleranceValue },
    #[error("unknown progressive delivery channel: {value:?}")]
    UnknownProgressiveDeliveryChannel { value: UnknownFaultToleranceValue },
    #[error("unknown failover drill status: {value:?}")]
    UnknownFailoverDrillStatus { value: UnknownFaultToleranceValue },
    #[error("unknown fault tolerance readiness status: {value:?}")]
    UnknownReadinessStatus { value: UnknownFaultToleranceValue },
    #[error("{field} cannot be negative, got {value}")]
    NegativeCount { field: &'static str, value: i64 },
    #[error("minimum_redundant_workers must be greater than zero")]
    MissingMinimumRedundancy,
    #[error("next_review_at must be after reviewed_at")]
    InvalidReviewWindow,
    #[error("readiness status {status:?} conflicts with fault-tolerance evidence")]
    InconsistentReadinessStatus {
        status: FaultToleranceReadinessStatus,
    },
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    ReleaseGate(#[from] ReleaseGateError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownFaultToleranceValue(String);

impl UnknownFaultToleranceValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlPlaneStatus {
    Healthy,
    Degraded,
    Unavailable,
}

impl TryFrom<&str> for ControlPlaneStatus {
    type Error = FaultToleranceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "healthy" => Ok(Self::Healthy),
            "degraded" => Ok(Self::Degraded),
            "unavailable" => Ok(Self::Unavailable),
            value => Err(FaultToleranceError::UnknownControlPlaneStatus {
                value: UnknownFaultToleranceValue::new(value),
            }),
        }
    }
}

impl ControlPlaneStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPlaneStatus {
    Serving,
    Degraded,
    Paused,
}

impl TryFrom<&str> for ExecutionPlaneStatus {
    type Error = FaultToleranceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "serving" => Ok(Self::Serving),
            "degraded" => Ok(Self::Degraded),
            "paused" => Ok(Self::Paused),
            value => Err(FaultToleranceError::UnknownExecutionPlaneStatus {
                value: UnknownFaultToleranceValue::new(value),
            }),
        }
    }
}

impl ExecutionPlaneStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Serving => "serving",
            Self::Degraded => "degraded",
            Self::Paused => "paused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticStabilityMode {
    Normal,
    LastKnownGood,
    DraftOnly,
    Paused,
}

impl TryFrom<&str> for StaticStabilityMode {
    type Error = FaultToleranceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "normal" => Ok(Self::Normal),
            "last_known_good" => Ok(Self::LastKnownGood),
            "draft_only" => Ok(Self::DraftOnly),
            "paused" => Ok(Self::Paused),
            value => Err(FaultToleranceError::UnknownStaticStabilityMode {
                value: UnknownFaultToleranceValue::new(value),
            }),
        }
    }
}

impl StaticStabilityMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::LastKnownGood => "last_known_good",
            Self::DraftOnly => "draft_only",
            Self::Paused => "paused",
        }
    }

    fn protects_against_control_plane_loss(self) -> bool {
        !matches!(self, Self::Normal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressiveDeliveryChannel {
    Dev,
    Canary,
    Production,
    HighRiskHold,
}

impl TryFrom<&str> for ProgressiveDeliveryChannel {
    type Error = FaultToleranceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "dev" => Ok(Self::Dev),
            "canary" => Ok(Self::Canary),
            "production" => Ok(Self::Production),
            "high_risk_hold" => Ok(Self::HighRiskHold),
            value => Err(FaultToleranceError::UnknownProgressiveDeliveryChannel {
                value: UnknownFaultToleranceValue::new(value),
            }),
        }
    }
}

impl ProgressiveDeliveryChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Canary => "canary",
            Self::Production => "production",
            Self::HighRiskHold => "high_risk_hold",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailoverDrillStatus {
    Planned,
    Running,
    Passed,
    Failed,
    Aborted,
}

impl TryFrom<&str> for FailoverDrillStatus {
    type Error = FaultToleranceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "planned" => Ok(Self::Planned),
            "running" => Ok(Self::Running),
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            "aborted" => Ok(Self::Aborted),
            value => Err(FaultToleranceError::UnknownFailoverDrillStatus {
                value: UnknownFaultToleranceValue::new(value),
            }),
        }
    }
}

impl FailoverDrillStatus {
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
pub enum FaultToleranceReadinessStatus {
    Ready,
    NeedsRedundancy,
    StaticStabilityMissing,
    ControlPlaneCoupled,
    FailoverDrillMissing,
    ReleaseGateMissing,
    ReleaseBlocked,
    ReviewOverdue,
}

impl TryFrom<&str> for FaultToleranceReadinessStatus {
    type Error = FaultToleranceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ready" => Ok(Self::Ready),
            "needs_redundancy" => Ok(Self::NeedsRedundancy),
            "static_stability_missing" => Ok(Self::StaticStabilityMissing),
            "control_plane_coupled" => Ok(Self::ControlPlaneCoupled),
            "failover_drill_missing" => Ok(Self::FailoverDrillMissing),
            "release_gate_missing" => Ok(Self::ReleaseGateMissing),
            "release_blocked" => Ok(Self::ReleaseBlocked),
            "review_overdue" => Ok(Self::ReviewOverdue),
            value => Err(FaultToleranceError::UnknownReadinessStatus {
                value: UnknownFaultToleranceValue::new(value),
            }),
        }
    }
}

impl FaultToleranceReadinessStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NeedsRedundancy => "needs_redundancy",
            Self::StaticStabilityMissing => "static_stability_missing",
            Self::ControlPlaneCoupled => "control_plane_coupled",
            Self::FailoverDrillMissing => "failover_drill_missing",
            Self::ReleaseGateMissing => "release_gate_missing",
            Self::ReleaseBlocked => "release_blocked",
            Self::ReviewOverdue => "review_overdue",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerReplicaCount(u64);

impl WorkerReplicaCount {
    pub fn try_from_i64(value: i64, field: &'static str) -> Result<Self, FaultToleranceError> {
        let value = u64::try_from(value)
            .map_err(|_| FaultToleranceError::NegativeCount { field, value })?;

        Ok(Self(value))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimumWorkerReplicaCount(u64);

impl MinimumWorkerReplicaCount {
    pub fn try_from_i64(value: i64) -> Result<Self, FaultToleranceError> {
        let value = u64::try_from(value).map_err(|_| FaultToleranceError::NegativeCount {
            field: "minimum_redundant_workers",
            value,
        })?;

        if value == 0 {
            return Err(FaultToleranceError::MissingMinimumRedundancy);
        }

        Ok(Self(value))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentJobCount(u64);

impl AgentJobCount {
    pub fn try_from_i64(value: i64, field: &'static str) -> Result<Self, FaultToleranceError> {
        let value = u64::try_from(value)
            .map_err(|_| FaultToleranceError::NegativeCount { field, value })?;

        Ok(Self(value))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureDomain(String);

impl FailureDomain {
    pub fn new(value: impl Into<String>) -> Result<Self, FaultToleranceError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::EmptyText {
                field: "isolated_failure_domain",
            }
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultToleranceOwner(String);

impl FaultToleranceOwner {
    pub fn new(value: impl Into<String>) -> Result<Self, FaultToleranceError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::EmptyText { field: "owner" }.into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ANCHOR: fault_tolerance_review
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultToleranceReadinessReview {
    job_kind: JobKind,
    control_plane_status: ControlPlaneStatus,
    execution_plane_status: ExecutionPlaneStatus,
    last_known_good_policy_version: PolicyVersion,
    last_known_good_prompt_version: PromptVersion,
    last_known_good_model_version: ModelRoute,
    redundant_worker_count: WorkerReplicaCount,
    minimum_redundant_workers: MinimumWorkerReplicaCount,
    isolated_failure_domain: FailureDomain,
    static_stability_mode: StaticStabilityMode,
    progressive_delivery_channel: ProgressiveDeliveryChannel,
    failover_drill_status: Option<FailoverDrillStatus>,
    latest_release_decision: Option<ReleaseGateDecision>,
    active_jobs: AgentJobCount,
    running_jobs: AgentJobCount,
    dead_jobs: AgentJobCount,
    reviewed_at: DateTime<Utc>,
    next_review_at: DateTime<Utc>,
    review_overdue: bool,
    readiness_status: FaultToleranceReadinessStatus,
}

impl FaultToleranceReadinessReview {
    pub fn job_kind(&self) -> &JobKind {
        &self.job_kind
    }

    pub fn readiness_status(&self) -> FaultToleranceReadinessStatus {
        self.readiness_status
    }

    pub fn static_stability_mode(&self) -> StaticStabilityMode {
        self.static_stability_mode
    }

    pub fn control_plane_status(&self) -> ControlPlaneStatus {
        self.control_plane_status
    }

    pub fn execution_plane_status(&self) -> ExecutionPlaneStatus {
        self.execution_plane_status
    }

    pub fn redundant_worker_count(&self) -> WorkerReplicaCount {
        self.redundant_worker_count
    }

    pub fn minimum_redundant_workers(&self) -> MinimumWorkerReplicaCount {
        self.minimum_redundant_workers
    }

    pub fn failover_drill_status(&self) -> Option<FailoverDrillStatus> {
        self.failover_drill_status
    }

    pub fn latest_release_decision(&self) -> Option<ReleaseGateDecision> {
        self.latest_release_decision
    }

    pub fn is_ready_for_production_execution(&self) -> bool {
        self.readiness_status == FaultToleranceReadinessStatus::Ready
    }

    pub fn can_continue_with_control_plane_unavailable(&self) -> bool {
        self.control_plane_status == ControlPlaneStatus::Unavailable
            && self.execution_plane_status == ExecutionPlaneStatus::Serving
            && self
                .static_stability_mode
                .protects_against_control_plane_loss()
    }
}
// ANCHOR_END: fault_tolerance_review

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbFaultToleranceReadinessRow {
    pub job_kind: String,
    pub control_plane_status: String,
    pub execution_plane_status: String,
    pub last_known_good_policy_version: String,
    pub last_known_good_prompt_version: String,
    pub last_known_good_model_version: String,
    pub redundant_worker_count: i64,
    pub minimum_redundant_workers: i64,
    pub isolated_failure_domain: String,
    pub static_stability_mode: String,
    pub progressive_delivery_channel: String,
    pub failover_drill_status: Option<String>,
    pub latest_release_decision: Option<String>,
    pub active_jobs: i64,
    pub running_jobs: i64,
    pub dead_jobs: i64,
    pub reviewed_at: DateTime<Utc>,
    pub next_review_at: DateTime<Utc>,
    pub review_overdue: bool,
    pub readiness_status: String,
}

impl TryFrom<DbFaultToleranceReadinessRow> for FaultToleranceReadinessReview {
    type Error = FaultToleranceError;

    fn try_from(row: DbFaultToleranceReadinessRow) -> Result<Self, Self::Error> {
        let control_plane_status = ControlPlaneStatus::try_from(row.control_plane_status.as_str())?;
        let execution_plane_status =
            ExecutionPlaneStatus::try_from(row.execution_plane_status.as_str())?;
        let static_stability_mode =
            StaticStabilityMode::try_from(row.static_stability_mode.as_str())?;
        let progressive_delivery_channel =
            ProgressiveDeliveryChannel::try_from(row.progressive_delivery_channel.as_str())?;
        let failover_drill_status = row
            .failover_drill_status
            .as_deref()
            .map(FailoverDrillStatus::try_from)
            .transpose()?;
        let latest_release_decision = row
            .latest_release_decision
            .as_deref()
            .map(ReleaseGateDecision::try_from)
            .transpose()?;
        let readiness_status =
            FaultToleranceReadinessStatus::try_from(row.readiness_status.as_str())?;

        let review = Self {
            job_kind: JobKind::new(row.job_kind)?,
            control_plane_status,
            execution_plane_status,
            last_known_good_policy_version: PolicyVersion::new(row.last_known_good_policy_version)?,
            last_known_good_prompt_version: PromptVersion::new(row.last_known_good_prompt_version)?,
            last_known_good_model_version: ModelRoute::new(row.last_known_good_model_version)?,
            redundant_worker_count: WorkerReplicaCount::try_from_i64(
                row.redundant_worker_count,
                "redundant_worker_count",
            )?,
            minimum_redundant_workers: MinimumWorkerReplicaCount::try_from_i64(
                row.minimum_redundant_workers,
            )?,
            isolated_failure_domain: FailureDomain::new(row.isolated_failure_domain)?,
            static_stability_mode,
            progressive_delivery_channel,
            failover_drill_status,
            latest_release_decision,
            active_jobs: AgentJobCount::try_from_i64(row.active_jobs, "active_jobs")?,
            running_jobs: AgentJobCount::try_from_i64(row.running_jobs, "running_jobs")?,
            dead_jobs: AgentJobCount::try_from_i64(row.dead_jobs, "dead_jobs")?,
            reviewed_at: row.reviewed_at,
            next_review_at: row.next_review_at,
            review_overdue: row.review_overdue,
            readiness_status,
        };

        validate_fault_tolerance_readiness(&review)?;

        Ok(review)
    }
}

fn validate_fault_tolerance_readiness(
    review: &FaultToleranceReadinessReview,
) -> Result<(), FaultToleranceError> {
    if review.next_review_at <= review.reviewed_at {
        return Err(FaultToleranceError::InvalidReviewWindow);
    }

    let inconsistent = |status| FaultToleranceError::InconsistentReadinessStatus { status };
    let status = review.readiness_status;

    if review.review_overdue && status == FaultToleranceReadinessStatus::Ready {
        return Err(inconsistent(status));
    }

    if review.redundant_worker_count.get() < review.minimum_redundant_workers.get()
        && status == FaultToleranceReadinessStatus::Ready
    {
        return Err(inconsistent(status));
    }

    if review.control_plane_status == ControlPlaneStatus::Unavailable
        && review.execution_plane_status == ExecutionPlaneStatus::Serving
        && !review
            .static_stability_mode
            .protects_against_control_plane_loss()
        && status == FaultToleranceReadinessStatus::Ready
    {
        return Err(inconsistent(status));
    }

    if matches!(
        review.control_plane_status,
        ControlPlaneStatus::Degraded | ControlPlaneStatus::Unavailable
    ) && !review
        .static_stability_mode
        .protects_against_control_plane_loss()
        && status == FaultToleranceReadinessStatus::Ready
    {
        return Err(inconsistent(status));
    }

    if review.failover_drill_status != Some(FailoverDrillStatus::Passed)
        && status == FaultToleranceReadinessStatus::Ready
    {
        return Err(inconsistent(status));
    }

    if review.progressive_delivery_channel == ProgressiveDeliveryChannel::Production
        && review.latest_release_decision.is_none()
        && status == FaultToleranceReadinessStatus::Ready
    {
        return Err(inconsistent(status));
    }

    if review.latest_release_decision == Some(ReleaseGateDecision::Block)
        && status == FaultToleranceReadinessStatus::Ready
    {
        return Err(inconsistent(status));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn ready_row() -> DbFaultToleranceReadinessRow {
        let reviewed_at = Utc::now();
        DbFaultToleranceReadinessRow {
            job_kind: "kyc_case_preparation".to_string(),
            control_plane_status: "healthy".to_string(),
            execution_plane_status: "serving".to_string(),
            last_known_good_policy_version: "approval-policy:v3".to_string(),
            last_known_good_prompt_version: "kyc-case:v12".to_string(),
            last_known_good_model_version: "deepseek-chat:v1".to_string(),
            redundant_worker_count: 3,
            minimum_redundant_workers: 2,
            isolated_failure_domain: "worker-az-eu-west-1a".to_string(),
            static_stability_mode: "last_known_good".to_string(),
            progressive_delivery_channel: "production".to_string(),
            failover_drill_status: Some("passed".to_string()),
            latest_release_decision: Some("promote".to_string()),
            active_jobs: 5,
            running_jobs: 2,
            dead_jobs: 0,
            reviewed_at,
            next_review_at: reviewed_at + Duration::days(7),
            review_overdue: false,
            readiness_status: "ready".to_string(),
        }
    }

    #[test]
    fn ready_review_decodes_when_redundant_static_and_drilled() {
        let review = FaultToleranceReadinessReview::try_from(ready_row())
            .expect("valid fault-tolerance row should decode");

        assert_eq!(
            review.readiness_status(),
            FaultToleranceReadinessStatus::Ready
        );
        assert!(review.is_ready_for_production_execution());
        assert_eq!(review.redundant_worker_count().get(), 3);
        assert_eq!(review.minimum_redundant_workers().get(), 2);
        assert_eq!(
            review.failover_drill_status(),
            Some(FailoverDrillStatus::Passed)
        );
    }

    #[test]
    fn row_conversion_rejects_negative_worker_count() {
        let mut row = ready_row();
        row.redundant_worker_count = -1;

        let err = FaultToleranceReadinessReview::try_from(row)
            .expect_err("negative worker count should be rejected");

        assert!(matches!(
            err,
            FaultToleranceError::NegativeCount {
                field: "redundant_worker_count",
                value: -1
            }
        ));
    }

    #[test]
    fn row_conversion_rejects_ready_without_required_redundancy() {
        let mut row = ready_row();
        row.redundant_worker_count = 1;

        let err = FaultToleranceReadinessReview::try_from(row)
            .expect_err("ready status should require enough redundant workers");

        assert!(matches!(
            err,
            FaultToleranceError::InconsistentReadinessStatus {
                status: FaultToleranceReadinessStatus::Ready
            }
        ));
    }

    #[test]
    fn row_conversion_rejects_ready_when_control_plane_unavailable_without_static_stability() {
        let mut row = ready_row();
        row.control_plane_status = "unavailable".to_string();
        row.static_stability_mode = "normal".to_string();

        let err = FaultToleranceReadinessReview::try_from(row)
            .expect_err("ready status should require static stability during control-plane loss");

        assert!(matches!(
            err,
            FaultToleranceError::InconsistentReadinessStatus {
                status: FaultToleranceReadinessStatus::Ready
            }
        ));
    }

    #[test]
    fn control_plane_outage_with_last_known_good_can_keep_execution_serving() {
        let mut row = ready_row();
        row.control_plane_status = "unavailable".to_string();
        row.static_stability_mode = "last_known_good".to_string();
        row.readiness_status = "static_stability_missing".to_string();

        let review = FaultToleranceReadinessReview::try_from(row)
            .expect("last-known-good evidence should decode during outage");

        assert!(review.can_continue_with_control_plane_unavailable());
    }

    #[test]
    fn row_conversion_rejects_unknown_static_stability_mode() {
        let mut row = ready_row();
        row.static_stability_mode = "hope".to_string();

        let err = FaultToleranceReadinessReview::try_from(row)
            .expect_err("unknown static stability mode should be rejected");

        assert!(matches!(
            err,
            FaultToleranceError::UnknownStaticStabilityMode { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_invalid_review_window() {
        let mut row = ready_row();
        row.next_review_at = row.reviewed_at;

        let err = FaultToleranceReadinessReview::try_from(row)
            .expect_err("review window should move forward");

        assert_eq!(err, FaultToleranceError::InvalidReviewWindow);
    }

    #[test]
    fn row_conversion_rejects_ready_without_passed_failover_drill() {
        let mut row = ready_row();
        row.failover_drill_status = Some("failed".to_string());

        let err = FaultToleranceReadinessReview::try_from(row)
            .expect_err("ready status should require a passed failover drill");

        assert!(matches!(
            err,
            FaultToleranceError::InconsistentReadinessStatus {
                status: FaultToleranceReadinessStatus::Ready
            }
        ));
    }

    #[test]
    fn row_conversion_rejects_ready_without_release_gate_for_production_channel() {
        let mut row = ready_row();
        row.latest_release_decision = None;

        let err = FaultToleranceReadinessReview::try_from(row)
            .expect_err("ready production status should require release-gate evidence");

        assert!(matches!(
            err,
            FaultToleranceError::InconsistentReadinessStatus {
                status: FaultToleranceReadinessStatus::Ready
            }
        ));
    }

    #[test]
    fn row_conversion_rejects_ready_when_release_gate_blocks() {
        let mut row = ready_row();
        row.latest_release_decision = Some("block".to_string());

        let err = FaultToleranceReadinessReview::try_from(row)
            .expect_err("ready status should reject blocked release evidence");

        assert!(matches!(
            err,
            FaultToleranceError::InconsistentReadinessStatus {
                status: FaultToleranceReadinessStatus::Ready
            }
        ));
    }
}
