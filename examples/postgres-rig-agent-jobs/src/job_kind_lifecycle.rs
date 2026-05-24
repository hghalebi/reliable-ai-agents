//! Typed review of job-kind deprecation and retirement evidence.
//!
//! Long-running agent systems need a safe way to remove old job kinds, prompt
//! routes, model routes, and tool contracts. Retirement is not a delete
//! button. It is an evidence decision: no active work, no waiting retries, no
//! pending approvals, no recent provider usage, and a durable pause or
//! deprecation path.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::{DomainError, JobKind};
use crate::release_gate::{ReleaseGateDecision, ReleaseGateError};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JobKindLifecycleError {
    #[error("unknown job-kind lifecycle recommendation: {value:?}")]
    UnknownRecommendation {
        value: UnknownJobKindLifecycleRecommendation,
    },
    #[error("{field} cannot be negative, got {value}")]
    NegativeCount { field: &'static str, value: i64 },
    #[error("paused job kind requires a pause reason")]
    MissingPauseReason,
    #[error("active job kind cannot carry a pause reason")]
    UnexpectedPauseReason,
    #[error("lifecycle recommendation {recommendation:?} conflicts with review evidence")]
    InconsistentRecommendation {
        recommendation: JobKindLifecycleRecommendation,
    },
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    ReleaseGate(#[from] ReleaseGateError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownJobKindLifecycleRecommendation(String);

impl UnknownJobKindLifecycleRecommendation {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleEvidenceCount(u64);

impl LifecycleEvidenceCount {
    pub fn try_from_i64(value: i64, field: &'static str) -> Result<Self, JobKindLifecycleError> {
        let value = u64::try_from(value)
            .map_err(|_| JobKindLifecycleError::NegativeCount { field, value })?;

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
pub struct JobKindPauseReason(String);

impl JobKindPauseReason {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::EmptyText {
                field: "job_kind_pause_reason",
            });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobKindControlState {
    Active,
    Paused { reason: JobKindPauseReason },
}

impl JobKindControlState {
    fn from_row(paused: bool, reason: Option<String>) -> Result<Self, JobKindLifecycleError> {
        match (paused, reason) {
            (true, Some(reason)) => Ok(Self::Paused {
                reason: JobKindPauseReason::new(reason)?,
            }),
            (true, None) => Err(JobKindLifecycleError::MissingPauseReason),
            (false, Some(_)) => Err(JobKindLifecycleError::UnexpectedPauseReason),
            (false, None) => Ok(Self::Active),
        }
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobKindLifecycleRecommendation {
    Active,
    DeprecationCandidate,
    RetirementCandidate,
    RetirementBlocked,
}

impl TryFrom<&str> for JobKindLifecycleRecommendation {
    type Error = JobKindLifecycleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "deprecation_candidate" => Ok(Self::DeprecationCandidate),
            "retirement_candidate" => Ok(Self::RetirementCandidate),
            "retirement_blocked" => Ok(Self::RetirementBlocked),
            value => Err(JobKindLifecycleError::UnknownRecommendation {
                value: UnknownJobKindLifecycleRecommendation::new(value),
            }),
        }
    }
}

impl JobKindLifecycleRecommendation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::DeprecationCandidate => "deprecation_candidate",
            Self::RetirementCandidate => "retirement_candidate",
            Self::RetirementBlocked => "retirement_blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobKindLifecycleEvidence {
    pending_or_running_jobs: LifecycleEvidenceCount,
    waiting_retry_jobs: LifecycleEvidenceCount,
    waiting_human_jobs: LifecycleEvidenceCount,
    recent_provider_calls_30d: LifecycleEvidenceCount,
}

impl JobKindLifecycleEvidence {
    pub fn new(
        pending_or_running_jobs: LifecycleEvidenceCount,
        waiting_retry_jobs: LifecycleEvidenceCount,
        waiting_human_jobs: LifecycleEvidenceCount,
        recent_provider_calls_30d: LifecycleEvidenceCount,
    ) -> Self {
        Self {
            pending_or_running_jobs,
            waiting_retry_jobs,
            waiting_human_jobs,
            recent_provider_calls_30d,
        }
    }

    pub fn pending_or_running_jobs(self) -> LifecycleEvidenceCount {
        self.pending_or_running_jobs
    }

    pub fn waiting_retry_jobs(self) -> LifecycleEvidenceCount {
        self.waiting_retry_jobs
    }

    pub fn waiting_human_jobs(self) -> LifecycleEvidenceCount {
        self.waiting_human_jobs
    }

    pub fn recent_provider_calls_30d(self) -> LifecycleEvidenceCount {
        self.recent_provider_calls_30d
    }

    fn has_open_work(self) -> bool {
        !self.pending_or_running_jobs.is_zero()
            || !self.waiting_retry_jobs.is_zero()
            || !self.waiting_human_jobs.is_zero()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobKindLifecycleReview {
    job_kind: JobKind,
    control_state: JobKindControlState,
    evidence: JobKindLifecycleEvidence,
    latest_release_decision: Option<ReleaseGateDecision>,
    latest_release_evaluated_at: Option<DateTime<Utc>>,
    recommendation: JobKindLifecycleRecommendation,
}

impl JobKindLifecycleReview {
    pub fn job_kind(&self) -> &JobKind {
        &self.job_kind
    }

    pub fn control_state(&self) -> &JobKindControlState {
        &self.control_state
    }

    pub fn evidence(&self) -> JobKindLifecycleEvidence {
        self.evidence
    }

    pub fn latest_release_decision(&self) -> Option<ReleaseGateDecision> {
        self.latest_release_decision
    }

    pub fn latest_release_evaluated_at(&self) -> Option<DateTime<Utc>> {
        self.latest_release_evaluated_at
    }

    pub fn recommendation(&self) -> JobKindLifecycleRecommendation {
        self.recommendation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbJobKindLifecycleReviewRow {
    pub job_kind: String,
    pub paused: bool,
    pub pause_reason: Option<String>,
    pub pending_or_running_jobs: i64,
    pub waiting_retry_jobs: i64,
    pub waiting_human_jobs: i64,
    pub recent_provider_calls_30d: i64,
    pub latest_release_decision: Option<String>,
    pub latest_release_evaluated_at: Option<DateTime<Utc>>,
    pub lifecycle_recommendation: String,
}

impl TryFrom<DbJobKindLifecycleReviewRow> for JobKindLifecycleReview {
    type Error = JobKindLifecycleError;

    fn try_from(row: DbJobKindLifecycleReviewRow) -> Result<Self, Self::Error> {
        let job_kind = JobKind::new(row.job_kind)?;
        let control_state = JobKindControlState::from_row(row.paused, row.pause_reason)?;
        let evidence = JobKindLifecycleEvidence::new(
            LifecycleEvidenceCount::try_from_i64(
                row.pending_or_running_jobs,
                "pending_or_running_jobs",
            )?,
            LifecycleEvidenceCount::try_from_i64(row.waiting_retry_jobs, "waiting_retry_jobs")?,
            LifecycleEvidenceCount::try_from_i64(row.waiting_human_jobs, "waiting_human_jobs")?,
            LifecycleEvidenceCount::try_from_i64(
                row.recent_provider_calls_30d,
                "recent_provider_calls_30d",
            )?,
        );
        let latest_release_decision = row
            .latest_release_decision
            .as_deref()
            .map(ReleaseGateDecision::try_from)
            .transpose()?;
        let recommendation =
            JobKindLifecycleRecommendation::try_from(row.lifecycle_recommendation.as_str())?;

        validate_recommendation(recommendation, &control_state, evidence)?;

        Ok(Self {
            job_kind,
            control_state,
            evidence,
            latest_release_decision,
            latest_release_evaluated_at: row.latest_release_evaluated_at,
            recommendation,
        })
    }
}

fn validate_recommendation(
    recommendation: JobKindLifecycleRecommendation,
    control_state: &JobKindControlState,
    evidence: JobKindLifecycleEvidence,
) -> Result<(), JobKindLifecycleError> {
    let has_open_work = evidence.has_open_work();
    let has_recent_provider_usage = !evidence.recent_provider_calls_30d().is_zero();
    let paused = control_state.is_paused();

    let valid = match recommendation {
        JobKindLifecycleRecommendation::RetirementBlocked => has_open_work,
        JobKindLifecycleRecommendation::RetirementCandidate => {
            paused && !has_open_work && !has_recent_provider_usage
        }
        JobKindLifecycleRecommendation::DeprecationCandidate => {
            !paused && !has_open_work && !has_recent_provider_usage
        }
        JobKindLifecycleRecommendation::Active => {
            !paused && !has_open_work && has_recent_provider_usage
        }
    };

    if valid {
        Ok(())
    } else {
        Err(JobKindLifecycleError::InconsistentRecommendation { recommendation })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row() -> DbJobKindLifecycleReviewRow {
        DbJobKindLifecycleReviewRow {
            job_kind: "incident_triage".to_string(),
            paused: false,
            pause_reason: None,
            pending_or_running_jobs: 0,
            waiting_retry_jobs: 0,
            waiting_human_jobs: 0,
            recent_provider_calls_30d: 7,
            latest_release_decision: Some("promote".to_string()),
            latest_release_evaluated_at: Some(Utc::now()),
            lifecycle_recommendation: "active".to_string(),
        }
    }

    #[test]
    fn active_recent_usage_decodes() {
        let review = JobKindLifecycleReview::try_from(row()).expect("valid lifecycle review");

        assert_eq!(review.job_kind().as_str(), "incident_triage");
        assert_eq!(
            review.recommendation(),
            JobKindLifecycleRecommendation::Active
        );
        assert_eq!(
            review.latest_release_decision(),
            Some(ReleaseGateDecision::Promote)
        );
        assert_eq!(review.evidence().recent_provider_calls_30d().get(), 7);
    }

    #[test]
    fn paused_without_usage_is_retirement_candidate() {
        let mut row = row();
        row.paused = true;
        row.pause_reason = Some("superseded by incident_triage_v2".to_string());
        row.recent_provider_calls_30d = 0;
        row.lifecycle_recommendation = "retirement_candidate".to_string();

        let review = JobKindLifecycleReview::try_from(row).expect("valid retirement candidate");

        assert_eq!(
            review.recommendation(),
            JobKindLifecycleRecommendation::RetirementCandidate
        );
        assert!(review.control_state().is_paused());
    }

    #[test]
    fn row_conversion_rejects_negative_count() {
        let mut row = row();
        row.recent_provider_calls_30d = -1;

        let error = JobKindLifecycleReview::try_from(row).expect_err("negative count rejected");

        assert_eq!(
            error,
            JobKindLifecycleError::NegativeCount {
                field: "recent_provider_calls_30d",
                value: -1,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_recommendation() {
        let mut row = row();
        row.lifecycle_recommendation = "delete_now".to_string();

        let error = JobKindLifecycleReview::try_from(row).expect_err("unknown state rejected");

        assert_eq!(
            error,
            JobKindLifecycleError::UnknownRecommendation {
                value: UnknownJobKindLifecycleRecommendation::new("delete_now"),
            }
        );
    }

    #[test]
    fn row_conversion_rejects_paused_without_reason() {
        let mut row = row();
        row.paused = true;
        row.recent_provider_calls_30d = 0;
        row.lifecycle_recommendation = "retirement_candidate".to_string();

        let error = JobKindLifecycleReview::try_from(row).expect_err("pause reason required");

        assert_eq!(error, JobKindLifecycleError::MissingPauseReason);
    }

    #[test]
    fn row_conversion_rejects_retirement_candidate_with_open_work() {
        let mut row = row();
        row.paused = true;
        row.pause_reason = Some("superseded".to_string());
        row.pending_or_running_jobs = 1;
        row.recent_provider_calls_30d = 0;
        row.lifecycle_recommendation = "retirement_candidate".to_string();

        let error = JobKindLifecycleReview::try_from(row).expect_err("open work blocks retirement");

        assert_eq!(
            error,
            JobKindLifecycleError::InconsistentRecommendation {
                recommendation: JobKindLifecycleRecommendation::RetirementCandidate,
            }
        );
    }
}
