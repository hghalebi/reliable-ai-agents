//! Typed job-kind readiness review for maturity claims.
//!
//! A convincing demo is not a production proof. This module decodes the
//! `job_kind_readiness_review.sql` output into domain values so a job kind can
//! be reviewed as demo, prototype, production, or regulated/high-risk work
//! without passing raw maturity labels through the application.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::{DomainError, JobKind};
use crate::release_gate::{ReleaseGateDecision, ReleaseGateError};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JobKindReadinessError {
    #[error("unknown maturity level: {value:?}")]
    UnknownMaturityLevel { value: UnknownReadinessValue },
    #[error("unknown job risk class: {value:?}")]
    UnknownRiskClass { value: UnknownReadinessValue },
    #[error("unknown readiness status: {value:?}")]
    UnknownJobKindReadinessStatus { value: UnknownReadinessValue },
    #[error("{field} cannot be negative, got {value}")]
    NegativeCount { field: &'static str, value: i64 },
    #[error("evidence_required_count must be greater than zero")]
    MissingRequiredEvidence,
    #[error("{field} cannot exceed evidence_required_count")]
    CountExceedsRequiredEvidence { field: &'static str },
    #[error("next_review_at must be after reviewed_at")]
    InvalidReviewWindow,
    #[error("target maturity {target:?} is too low for risk class {risk:?}")]
    TargetLevelTooLow {
        risk: JobRiskClass,
        target: MaturityLevel,
    },
    #[error("readiness status {status:?} conflicts with maturity evidence")]
    InconsistentJobKindReadinessStatus { status: JobKindReadinessStatus },
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    ReleaseGate(#[from] ReleaseGateError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownReadinessValue(String);

impl UnknownReadinessValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaturityLevel {
    Demo,
    Prototype,
    Production,
    RegulatedHighRisk,
}

impl TryFrom<&str> for MaturityLevel {
    type Error = JobKindReadinessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "demo" => Ok(Self::Demo),
            "prototype" => Ok(Self::Prototype),
            "production" => Ok(Self::Production),
            "regulated_high_risk" => Ok(Self::RegulatedHighRisk),
            value => Err(JobKindReadinessError::UnknownMaturityLevel {
                value: UnknownReadinessValue::new(value),
            }),
        }
    }
}

impl MaturityLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Demo => "demo",
            Self::Prototype => "prototype",
            Self::Production => "production",
            Self::RegulatedHighRisk => "regulated_high_risk",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobRiskClass {
    Low,
    Medium,
    High,
    Regulated,
}

impl TryFrom<&str> for JobRiskClass {
    type Error = JobKindReadinessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "regulated" => Ok(Self::Regulated),
            value => Err(JobKindReadinessError::UnknownRiskClass {
                value: UnknownReadinessValue::new(value),
            }),
        }
    }
}

impl JobRiskClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Regulated => "regulated",
        }
    }

    fn minimum_target_level(self) -> MaturityLevel {
        match self {
            Self::Low => MaturityLevel::Demo,
            Self::Medium => MaturityLevel::Prototype,
            Self::High => MaturityLevel::Production,
            Self::Regulated => MaturityLevel::RegulatedHighRisk,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobKindReadinessStatus {
    ReadyForTarget,
    MissingEvidence,
    BlockedByGaps,
    ReviewOverdue,
}

impl TryFrom<&str> for JobKindReadinessStatus {
    type Error = JobKindReadinessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ready_for_target" => Ok(Self::ReadyForTarget),
            "missing_evidence" => Ok(Self::MissingEvidence),
            "blocked_by_gaps" => Ok(Self::BlockedByGaps),
            "review_overdue" => Ok(Self::ReviewOverdue),
            value => Err(JobKindReadinessError::UnknownJobKindReadinessStatus {
                value: UnknownReadinessValue::new(value),
            }),
        }
    }
}

impl JobKindReadinessStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadyForTarget => "ready_for_target",
            Self::MissingEvidence => "missing_evidence",
            Self::BlockedByGaps => "blocked_by_gaps",
            Self::ReviewOverdue => "review_overdue",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadinessEvidenceCount(u64);

impl ReadinessEvidenceCount {
    pub fn try_from_i64(value: i64, field: &'static str) -> Result<Self, JobKindReadinessError> {
        let value = u64::try_from(value)
            .map_err(|_| JobKindReadinessError::NegativeCount { field, value })?;

        Ok(Self(value))
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessOwner(String);

impl ReadinessOwner {
    pub fn new(value: impl Into<String>) -> Result<Self, JobKindReadinessError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::EmptyText {
                field: "readiness_owner",
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
pub struct NextReadinessChange(String);

impl NextReadinessChange {
    pub fn new(value: impl Into<String>) -> Result<Self, JobKindReadinessError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::EmptyText {
                field: "next_readiness_change",
            }
            .into());
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadinessEvidence {
    ready_count: ReadinessEvidenceCount,
    required_count: ReadinessEvidenceCount,
    blocking_gap_count: ReadinessEvidenceCount,
}

impl ReadinessEvidence {
    pub fn new(
        ready_count: ReadinessEvidenceCount,
        required_count: ReadinessEvidenceCount,
        blocking_gap_count: ReadinessEvidenceCount,
    ) -> Result<Self, JobKindReadinessError> {
        if required_count.is_zero() {
            return Err(JobKindReadinessError::MissingRequiredEvidence);
        }
        for (field, count) in [
            ("evidence_ready_count", ready_count),
            ("blocking_gap_count", blocking_gap_count),
        ] {
            if count.get() > required_count.get() {
                return Err(JobKindReadinessError::CountExceedsRequiredEvidence { field });
            }
        }

        Ok(Self {
            ready_count,
            required_count,
            blocking_gap_count,
        })
    }

    pub fn ready_count(self) -> ReadinessEvidenceCount {
        self.ready_count
    }

    pub fn required_count(self) -> ReadinessEvidenceCount {
        self.required_count
    }

    pub fn blocking_gap_count(self) -> ReadinessEvidenceCount {
        self.blocking_gap_count
    }

    fn complete(self) -> bool {
        self.ready_count.get() == self.required_count.get() && self.blocking_gap_count.is_zero()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobKindReadinessReview {
    job_kind: JobKind,
    target_level: MaturityLevel,
    current_level: MaturityLevel,
    risk_class: JobRiskClass,
    evidence: ReadinessEvidence,
    owner: ReadinessOwner,
    next_change: NextReadinessChange,
    reviewed_at: DateTime<Utc>,
    next_review_at: DateTime<Utc>,
    review_overdue: bool,
    latest_release_decision: Option<ReleaseGateDecision>,
    readiness_status: JobKindReadinessStatus,
}

impl JobKindReadinessReview {
    pub fn job_kind(&self) -> &JobKind {
        &self.job_kind
    }

    pub fn target_level(&self) -> MaturityLevel {
        self.target_level
    }

    pub fn current_level(&self) -> MaturityLevel {
        self.current_level
    }

    pub fn risk_class(&self) -> JobRiskClass {
        self.risk_class
    }

    pub fn evidence(&self) -> ReadinessEvidence {
        self.evidence
    }

    pub fn owner(&self) -> &ReadinessOwner {
        &self.owner
    }

    pub fn next_change(&self) -> &NextReadinessChange {
        &self.next_change
    }

    pub fn reviewed_at(&self) -> DateTime<Utc> {
        self.reviewed_at
    }

    pub fn next_review_at(&self) -> DateTime<Utc> {
        self.next_review_at
    }

    pub fn review_overdue(&self) -> bool {
        self.review_overdue
    }

    pub fn latest_release_decision(&self) -> Option<ReleaseGateDecision> {
        self.latest_release_decision
    }

    pub fn readiness_status(&self) -> JobKindReadinessStatus {
        self.readiness_status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbJobKindReadinessReviewRow {
    pub job_kind: String,
    pub target_level: String,
    pub current_level: String,
    pub risk_class: String,
    pub evidence_ready_count: i64,
    pub evidence_required_count: i64,
    pub blocking_gap_count: i64,
    pub owner: String,
    pub next_change: String,
    pub reviewed_at: DateTime<Utc>,
    pub next_review_at: DateTime<Utc>,
    pub review_overdue: bool,
    pub latest_release_decision: Option<String>,
    pub readiness_status: String,
}

impl TryFrom<DbJobKindReadinessReviewRow> for JobKindReadinessReview {
    type Error = JobKindReadinessError;

    fn try_from(row: DbJobKindReadinessReviewRow) -> Result<Self, Self::Error> {
        let job_kind = JobKind::new(row.job_kind)?;
        let target_level = MaturityLevel::try_from(row.target_level.as_str())?;
        let current_level = MaturityLevel::try_from(row.current_level.as_str())?;
        let risk_class = JobRiskClass::try_from(row.risk_class.as_str())?;
        let evidence = ReadinessEvidence::new(
            ReadinessEvidenceCount::try_from_i64(row.evidence_ready_count, "evidence_ready_count")?,
            ReadinessEvidenceCount::try_from_i64(
                row.evidence_required_count,
                "evidence_required_count",
            )?,
            ReadinessEvidenceCount::try_from_i64(row.blocking_gap_count, "blocking_gap_count")?,
        )?;
        let owner = ReadinessOwner::new(row.owner)?;
        let next_change = NextReadinessChange::new(row.next_change)?;
        let latest_release_decision = row
            .latest_release_decision
            .as_deref()
            .map(ReleaseGateDecision::try_from)
            .transpose()?;
        let readiness_status = JobKindReadinessStatus::try_from(row.readiness_status.as_str())?;

        if row.next_review_at <= row.reviewed_at {
            return Err(JobKindReadinessError::InvalidReviewWindow);
        }

        validate_target_level(risk_class, target_level)?;
        validate_readiness_status(
            readiness_status,
            row.review_overdue,
            current_level,
            target_level,
            evidence,
        )?;

        Ok(Self {
            job_kind,
            target_level,
            current_level,
            risk_class,
            evidence,
            owner,
            next_change,
            reviewed_at: row.reviewed_at,
            next_review_at: row.next_review_at,
            review_overdue: row.review_overdue,
            latest_release_decision,
            readiness_status,
        })
    }
}

fn validate_target_level(
    risk_class: JobRiskClass,
    target_level: MaturityLevel,
) -> Result<(), JobKindReadinessError> {
    if target_level < risk_class.minimum_target_level() {
        Err(JobKindReadinessError::TargetLevelTooLow {
            risk: risk_class,
            target: target_level,
        })
    } else {
        Ok(())
    }
}

fn validate_readiness_status(
    status: JobKindReadinessStatus,
    review_overdue: bool,
    current_level: MaturityLevel,
    target_level: MaturityLevel,
    evidence: ReadinessEvidence,
) -> Result<(), JobKindReadinessError> {
    let meets_target = current_level >= target_level && evidence.complete();
    let valid = match status {
        JobKindReadinessStatus::ReviewOverdue => review_overdue,
        JobKindReadinessStatus::BlockedByGaps => {
            !review_overdue && !evidence.blocking_gap_count.is_zero()
        }
        JobKindReadinessStatus::MissingEvidence => {
            !review_overdue && evidence.blocking_gap_count.is_zero() && !meets_target
        }
        JobKindReadinessStatus::ReadyForTarget => !review_overdue && meets_target,
    };

    if valid {
        Ok(())
    } else {
        Err(JobKindReadinessError::InconsistentJobKindReadinessStatus { status })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row() -> DbJobKindReadinessReviewRow {
        DbJobKindReadinessReviewRow {
            job_kind: "incident_triage".to_string(),
            target_level: "production".to_string(),
            current_level: "production".to_string(),
            risk_class: "high".to_string(),
            evidence_ready_count: 12,
            evidence_required_count: 12,
            blocking_gap_count: 0,
            owner: "sre-oncall".to_string(),
            next_change: "run quarterly restore drill".to_string(),
            reviewed_at: Utc::now(),
            next_review_at: Utc::now() + chrono::Duration::days(30),
            review_overdue: false,
            latest_release_decision: Some("promote".to_string()),
            readiness_status: "ready_for_target".to_string(),
        }
    }

    #[test]
    fn production_readiness_review_decodes() {
        let review = JobKindReadinessReview::try_from(row()).expect("valid readiness review");

        assert_eq!(review.job_kind().as_str(), "incident_triage");
        assert_eq!(review.target_level(), MaturityLevel::Production);
        assert_eq!(review.current_level(), MaturityLevel::Production);
        assert_eq!(review.risk_class(), JobRiskClass::High);
        assert_eq!(review.evidence().ready_count().get(), 12);
        assert_eq!(
            review.readiness_status(),
            JobKindReadinessStatus::ReadyForTarget
        );
        assert_eq!(
            review.latest_release_decision(),
            Some(ReleaseGateDecision::Promote)
        );
    }

    #[test]
    fn regulated_risk_requires_regulated_high_risk_target() {
        let mut row = row();
        row.risk_class = "regulated".to_string();
        row.target_level = "production".to_string();
        row.current_level = "production".to_string();

        let error = JobKindReadinessReview::try_from(row).expect_err("regulated target too low");

        assert_eq!(
            error,
            JobKindReadinessError::TargetLevelTooLow {
                risk: JobRiskClass::Regulated,
                target: MaturityLevel::Production,
            }
        );
    }

    #[test]
    fn missing_evidence_decodes_when_gap_count_is_zero() {
        let mut row = row();
        row.current_level = "prototype".to_string();
        row.evidence_ready_count = 8;
        row.readiness_status = "missing_evidence".to_string();

        let review = JobKindReadinessReview::try_from(row).expect("valid missing evidence review");

        assert_eq!(
            review.readiness_status(),
            JobKindReadinessStatus::MissingEvidence
        );
        assert_eq!(review.current_level(), MaturityLevel::Prototype);
    }

    #[test]
    fn blocked_by_gaps_requires_gap_evidence() {
        let mut row = row();
        row.blocking_gap_count = 2;
        row.evidence_ready_count = 10;
        row.readiness_status = "blocked_by_gaps".to_string();

        let review = JobKindReadinessReview::try_from(row).expect("valid blocked review");

        assert_eq!(
            review.readiness_status(),
            JobKindReadinessStatus::BlockedByGaps
        );
        assert_eq!(review.evidence().blocking_gap_count().get(), 2);
    }

    #[test]
    fn ready_status_rejects_incomplete_evidence() {
        let mut row = row();
        row.evidence_ready_count = 11;

        let error =
            JobKindReadinessReview::try_from(row).expect_err("incomplete evidence rejected");

        assert_eq!(
            error,
            JobKindReadinessError::InconsistentJobKindReadinessStatus {
                status: JobKindReadinessStatus::ReadyForTarget,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_maturity_level() {
        let mut row = row();
        row.target_level = "enterprise_magic".to_string();

        let error = JobKindReadinessReview::try_from(row).expect_err("unknown level rejected");

        assert!(matches!(
            error,
            JobKindReadinessError::UnknownMaturityLevel { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_negative_count() {
        let mut row = row();
        row.blocking_gap_count = -1;

        let error = JobKindReadinessReview::try_from(row).expect_err("negative count rejected");

        assert_eq!(
            error,
            JobKindReadinessError::NegativeCount {
                field: "blocking_gap_count",
                value: -1,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_ready_count_above_required() {
        let mut row = row();
        row.evidence_ready_count = 13;

        let error = JobKindReadinessReview::try_from(row).expect_err("count mismatch rejected");

        assert_eq!(
            error,
            JobKindReadinessError::CountExceedsRequiredEvidence {
                field: "evidence_ready_count",
            }
        );
    }

    #[test]
    fn row_conversion_rejects_invalid_review_window() {
        let mut row = row();
        row.next_review_at = row.reviewed_at;

        let error = JobKindReadinessReview::try_from(row).expect_err("review window rejected");

        assert_eq!(error, JobKindReadinessError::InvalidReviewWindow);
    }
}
