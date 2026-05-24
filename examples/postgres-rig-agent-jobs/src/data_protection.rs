//! Typed review of privacy, redaction, erasure, and retention work.
//!
//! Privacy work must be durable operational state, not a private spreadsheet or
//! a support-chat promise. This module decodes the read-only
//! `data_protection_review.sql` output into domain values so operators can see
//! which evidence surfaces have open redaction, erasure, export, or retention
//! review work.

use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DataProtectionError {
    #[error("unknown data-protection surface: {value:?}")]
    UnknownSurface { value: UnknownDataProtectionSurface },
    #[error("unknown data-protection review status: {value:?}")]
    UnknownReviewStatus {
        value: UnknownDataProtectionReviewStatus,
    },
    #[error("{field} cannot be negative, got {value}")]
    NegativeCount { field: &'static str, value: i64 },
    #[error("{field} cannot exceed open_requests")]
    CountExceedsOpenRequests { field: &'static str },
    #[error("data-protection review status {status:?} conflicts with request counts")]
    InconsistentReviewStatus { status: DataProtectionReviewStatus },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownDataProtectionSurface(String);

impl UnknownDataProtectionSurface {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownDataProtectionReviewStatus(String);

impl UnknownDataProtectionReviewStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataProtectionRequestCount(u64);

impl DataProtectionRequestCount {
    pub fn try_from_i64(value: i64, field: &'static str) -> Result<Self, DataProtectionError> {
        let value = u64::try_from(value)
            .map_err(|_| DataProtectionError::NegativeCount { field, value })?;

        Ok(Self(value))
    }

    pub fn get(self) -> u64 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataProtectionSurface {
    AgentJobs,
    ScheduledJobs,
    BackgroundJobs,
    AgentRuns,
    ToolCalls,
    AuditEvents,
    OperationEvents,
    ProviderUsageEvents,
    HumanApprovalRequests,
    SideEffectReceipts,
    EvaluationRuns,
    AgentMemoryRecords,
}

impl TryFrom<&str> for DataProtectionSurface {
    type Error = DataProtectionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "agent_jobs" => Ok(Self::AgentJobs),
            "scheduled_jobs" => Ok(Self::ScheduledJobs),
            "background_jobs" => Ok(Self::BackgroundJobs),
            "agent_runs" => Ok(Self::AgentRuns),
            "tool_calls" => Ok(Self::ToolCalls),
            "audit_events" => Ok(Self::AuditEvents),
            "operation_events" => Ok(Self::OperationEvents),
            "provider_usage_events" => Ok(Self::ProviderUsageEvents),
            "human_approval_requests" => Ok(Self::HumanApprovalRequests),
            "side_effect_receipts" => Ok(Self::SideEffectReceipts),
            "evaluation_runs" => Ok(Self::EvaluationRuns),
            "agent_memory_records" => Ok(Self::AgentMemoryRecords),
            value => Err(DataProtectionError::UnknownSurface {
                value: UnknownDataProtectionSurface::new(value),
            }),
        }
    }
}

impl DataProtectionSurface {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AgentJobs => "agent_jobs",
            Self::ScheduledJobs => "scheduled_jobs",
            Self::BackgroundJobs => "background_jobs",
            Self::AgentRuns => "agent_runs",
            Self::ToolCalls => "tool_calls",
            Self::AuditEvents => "audit_events",
            Self::OperationEvents => "operation_events",
            Self::ProviderUsageEvents => "provider_usage_events",
            Self::HumanApprovalRequests => "human_approval_requests",
            Self::SideEffectReceipts => "side_effect_receipts",
            Self::EvaluationRuns => "evaluation_runs",
            Self::AgentMemoryRecords => "agent_memory_records",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataProtectionReviewStatus {
    NoOpenPrivacyWork,
    PrivacyWorkPending,
    RedactionPending,
    ErasurePending,
    PrivacyReviewOverdue,
}

impl TryFrom<&str> for DataProtectionReviewStatus {
    type Error = DataProtectionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "no_open_privacy_work" => Ok(Self::NoOpenPrivacyWork),
            "privacy_work_pending" => Ok(Self::PrivacyWorkPending),
            "redaction_pending" => Ok(Self::RedactionPending),
            "erasure_pending" => Ok(Self::ErasurePending),
            "privacy_review_overdue" => Ok(Self::PrivacyReviewOverdue),
            value => Err(DataProtectionError::UnknownReviewStatus {
                value: UnknownDataProtectionReviewStatus::new(value),
            }),
        }
    }
}

impl DataProtectionReviewStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoOpenPrivacyWork => "no_open_privacy_work",
            Self::PrivacyWorkPending => "privacy_work_pending",
            Self::RedactionPending => "redaction_pending",
            Self::ErasurePending => "erasure_pending",
            Self::PrivacyReviewOverdue => "privacy_review_overdue",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataProtectionEvidence {
    open_requests: DataProtectionRequestCount,
    overdue_requests: DataProtectionRequestCount,
    pending_redaction_requests: DataProtectionRequestCount,
    pending_erasure_requests: DataProtectionRequestCount,
    recently_applied_requests_30d: DataProtectionRequestCount,
}

impl DataProtectionEvidence {
    pub fn new(
        open_requests: DataProtectionRequestCount,
        overdue_requests: DataProtectionRequestCount,
        pending_redaction_requests: DataProtectionRequestCount,
        pending_erasure_requests: DataProtectionRequestCount,
        recently_applied_requests_30d: DataProtectionRequestCount,
    ) -> Result<Self, DataProtectionError> {
        let evidence = Self {
            open_requests,
            overdue_requests,
            pending_redaction_requests,
            pending_erasure_requests,
            recently_applied_requests_30d,
        };
        evidence.validate_counts()?;
        Ok(evidence)
    }

    pub fn open_requests(self) -> DataProtectionRequestCount {
        self.open_requests
    }

    pub fn overdue_requests(self) -> DataProtectionRequestCount {
        self.overdue_requests
    }

    pub fn pending_redaction_requests(self) -> DataProtectionRequestCount {
        self.pending_redaction_requests
    }

    pub fn pending_erasure_requests(self) -> DataProtectionRequestCount {
        self.pending_erasure_requests
    }

    pub fn recently_applied_requests_30d(self) -> DataProtectionRequestCount {
        self.recently_applied_requests_30d
    }

    fn validate_counts(self) -> Result<(), DataProtectionError> {
        let open = self.open_requests.get();
        for (field, count) in [
            ("overdue_requests", self.overdue_requests),
            (
                "pending_redaction_requests",
                self.pending_redaction_requests,
            ),
            ("pending_erasure_requests", self.pending_erasure_requests),
        ] {
            if count.get() > open {
                return Err(DataProtectionError::CountExceedsOpenRequests { field });
            }
        }

        Ok(())
    }
}

// ANCHOR: data_protection_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataProtectionReview {
    surface: DataProtectionSurface,
    evidence: DataProtectionEvidence,
    latest_request_at: Option<DateTime<Utc>>,
    latest_completed_at: Option<DateTime<Utc>>,
    review_status: DataProtectionReviewStatus,
}

impl DataProtectionReview {
    pub fn surface(&self) -> DataProtectionSurface {
        self.surface
    }

    pub fn evidence(&self) -> DataProtectionEvidence {
        self.evidence
    }

    pub fn latest_request_at(&self) -> Option<DateTime<Utc>> {
        self.latest_request_at
    }

    pub fn latest_completed_at(&self) -> Option<DateTime<Utc>> {
        self.latest_completed_at
    }

    pub fn review_status(&self) -> DataProtectionReviewStatus {
        self.review_status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbDataProtectionReviewRow {
    pub surface: String,
    pub open_requests: i64,
    pub overdue_requests: i64,
    pub pending_redaction_requests: i64,
    pub pending_erasure_requests: i64,
    pub recently_applied_requests_30d: i64,
    pub latest_request_at: Option<DateTime<Utc>>,
    pub latest_completed_at: Option<DateTime<Utc>>,
    pub review_status: String,
}

impl TryFrom<DbDataProtectionReviewRow> for DataProtectionReview {
    type Error = DataProtectionError;

    fn try_from(row: DbDataProtectionReviewRow) -> Result<Self, Self::Error> {
        let surface = DataProtectionSurface::try_from(row.surface.as_str())?;
        let evidence = DataProtectionEvidence::new(
            DataProtectionRequestCount::try_from_i64(row.open_requests, "open_requests")?,
            DataProtectionRequestCount::try_from_i64(row.overdue_requests, "overdue_requests")?,
            DataProtectionRequestCount::try_from_i64(
                row.pending_redaction_requests,
                "pending_redaction_requests",
            )?,
            DataProtectionRequestCount::try_from_i64(
                row.pending_erasure_requests,
                "pending_erasure_requests",
            )?,
            DataProtectionRequestCount::try_from_i64(
                row.recently_applied_requests_30d,
                "recently_applied_requests_30d",
            )?,
        )?;
        let review_status = DataProtectionReviewStatus::try_from(row.review_status.as_str())?;

        validate_review_status(review_status, evidence)?;

        Ok(Self {
            surface,
            evidence,
            latest_request_at: row.latest_request_at,
            latest_completed_at: row.latest_completed_at,
            review_status,
        })
    }
}
// ANCHOR_END: data_protection_row_boundary

fn validate_review_status(
    status: DataProtectionReviewStatus,
    evidence: DataProtectionEvidence,
) -> Result<(), DataProtectionError> {
    let valid = match status {
        DataProtectionReviewStatus::PrivacyReviewOverdue => !evidence.overdue_requests().is_zero(),
        DataProtectionReviewStatus::ErasurePending => {
            evidence.overdue_requests().is_zero() && !evidence.pending_erasure_requests().is_zero()
        }
        DataProtectionReviewStatus::RedactionPending => {
            evidence.overdue_requests().is_zero()
                && evidence.pending_erasure_requests().is_zero()
                && !evidence.pending_redaction_requests().is_zero()
        }
        DataProtectionReviewStatus::PrivacyWorkPending => {
            evidence.overdue_requests().is_zero()
                && evidence.pending_erasure_requests().is_zero()
                && evidence.pending_redaction_requests().is_zero()
                && !evidence.open_requests().is_zero()
        }
        DataProtectionReviewStatus::NoOpenPrivacyWork => {
            evidence.open_requests().is_zero()
                && evidence.overdue_requests().is_zero()
                && evidence.pending_erasure_requests().is_zero()
                && evidence.pending_redaction_requests().is_zero()
        }
    };

    if valid {
        Ok(())
    } else {
        Err(DataProtectionError::InconsistentReviewStatus { status })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row() -> DbDataProtectionReviewRow {
        DbDataProtectionReviewRow {
            surface: "agent_memory_records".to_string(),
            open_requests: 0,
            overdue_requests: 0,
            pending_redaction_requests: 0,
            pending_erasure_requests: 0,
            recently_applied_requests_30d: 2,
            latest_request_at: Some(Utc::now()),
            latest_completed_at: Some(Utc::now()),
            review_status: "no_open_privacy_work".to_string(),
        }
    }

    #[test]
    fn no_open_privacy_work_decodes() {
        let review = DataProtectionReview::try_from(row()).expect("valid privacy review");

        assert_eq!(review.surface(), DataProtectionSurface::AgentMemoryRecords);
        assert_eq!(
            review.review_status(),
            DataProtectionReviewStatus::NoOpenPrivacyWork
        );
        assert_eq!(review.evidence().recently_applied_requests_30d().get(), 2);
    }

    #[test]
    fn overdue_privacy_work_decodes() {
        let mut row = row();
        row.open_requests = 3;
        row.overdue_requests = 1;
        row.review_status = "privacy_review_overdue".to_string();

        let review = DataProtectionReview::try_from(row).expect("valid overdue review");

        assert_eq!(
            review.review_status(),
            DataProtectionReviewStatus::PrivacyReviewOverdue
        );
        assert_eq!(review.evidence().open_requests().get(), 3);
        assert_eq!(review.evidence().overdue_requests().get(), 1);
    }

    #[test]
    fn row_conversion_rejects_negative_count() {
        let mut row = row();
        row.open_requests = -1;

        let error = DataProtectionReview::try_from(row).expect_err("negative count rejected");

        assert_eq!(
            error,
            DataProtectionError::NegativeCount {
                field: "open_requests",
                value: -1,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_surface() {
        let mut row = row();
        row.surface = "raw_prompt_dump".to_string();

        let error = DataProtectionReview::try_from(row).expect_err("unknown surface rejected");

        assert_eq!(
            error,
            DataProtectionError::UnknownSurface {
                value: UnknownDataProtectionSurface::new("raw_prompt_dump"),
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_review_status() {
        let mut row = row();
        row.review_status = "ignore_privacy".to_string();

        let error = DataProtectionReview::try_from(row).expect_err("unknown status rejected");

        assert_eq!(
            error,
            DataProtectionError::UnknownReviewStatus {
                value: UnknownDataProtectionReviewStatus::new("ignore_privacy"),
            }
        );
    }

    #[test]
    fn row_conversion_rejects_specific_request_count_above_open_requests() {
        let mut row = row();
        row.open_requests = 1;
        row.pending_erasure_requests = 2;
        row.review_status = "erasure_pending".to_string();

        let error =
            DataProtectionReview::try_from(row).expect_err("erasure count cannot exceed open");

        assert_eq!(
            error,
            DataProtectionError::CountExceedsOpenRequests {
                field: "pending_erasure_requests",
            }
        );
    }

    #[test]
    fn row_conversion_rejects_no_open_status_with_pending_work() {
        let mut row = row();
        row.open_requests = 1;

        let error = DataProtectionReview::try_from(row).expect_err("status must match open work");

        assert_eq!(
            error,
            DataProtectionError::InconsistentReviewStatus {
                status: DataProtectionReviewStatus::NoOpenPrivacyWork,
            }
        );
    }
}
