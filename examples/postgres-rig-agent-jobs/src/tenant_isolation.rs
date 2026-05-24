//! Typed review of tenant-boundary authorization evidence.
//!
//! Multi-tenant agent systems fail when tenant scope is treated as a prompt
//! instruction or loose string. This module decodes the read-only
//! `tenant_boundary_review.sql` output into domain values so operators can see
//! cross-tenant attempts, denied attempts, approval pressure, and boundary
//! breaches without trusting private notes or raw dashboard strings.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::{DomainError, TenantKey};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TenantIsolationError {
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
    #[error("unknown tenant isolation review status: {value:?}")]
    UnknownReviewStatus {
        value: UnknownTenantIsolationReviewStatus,
    },
    #[error("{field} cannot be negative, got {value}")]
    NegativeCount { field: &'static str, value: i64 },
    #[error("cross-tenant denied and allowed counts must equal cross_tenant_attempts")]
    CrossTenantCountMismatch,
    #[error("latest_decision_at is required for tenant-boundary review rows")]
    MissingLatestDecisionAt,
    #[error("tenant isolation review status {status:?} conflicts with evidence counts")]
    InconsistentReviewStatus { status: TenantIsolationReviewStatus },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTenantIsolationReviewStatus(String);

impl UnknownTenantIsolationReviewStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantIsolationEventCount(u64);

impl TenantIsolationEventCount {
    pub fn try_from_i64(value: i64, field: &'static str) -> Result<Self, TenantIsolationError> {
        let value = u64::try_from(value)
            .map_err(|_| TenantIsolationError::NegativeCount { field, value })?;

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
pub enum TenantIsolationReviewStatus {
    SameTenantAuthorized,
    ApprovalRequired,
    CrossTenantDenied,
    TenantBoundaryBreach,
}

impl TryFrom<&str> for TenantIsolationReviewStatus {
    type Error = TenantIsolationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "same_tenant_authorized" => Ok(Self::SameTenantAuthorized),
            "approval_required" => Ok(Self::ApprovalRequired),
            "cross_tenant_denied" => Ok(Self::CrossTenantDenied),
            "tenant_boundary_breach" => Ok(Self::TenantBoundaryBreach),
            value => Err(TenantIsolationError::UnknownReviewStatus {
                value: UnknownTenantIsolationReviewStatus::new(value),
            }),
        }
    }
}

impl TenantIsolationReviewStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SameTenantAuthorized => "same_tenant_authorized",
            Self::ApprovalRequired => "approval_required",
            Self::CrossTenantDenied => "cross_tenant_denied",
            Self::TenantBoundaryBreach => "tenant_boundary_breach",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantIsolationEvidence {
    authorization_events: TenantIsolationEventCount,
    cross_tenant_attempts: TenantIsolationEventCount,
    cross_tenant_allowed: TenantIsolationEventCount,
    denied_cross_tenant_attempts: TenantIsolationEventCount,
    approvals_required: TenantIsolationEventCount,
    same_tenant_authorized: TenantIsolationEventCount,
}

impl TenantIsolationEvidence {
    pub fn new(
        authorization_events: TenantIsolationEventCount,
        cross_tenant_attempts: TenantIsolationEventCount,
        cross_tenant_allowed: TenantIsolationEventCount,
        denied_cross_tenant_attempts: TenantIsolationEventCount,
        approvals_required: TenantIsolationEventCount,
        same_tenant_authorized: TenantIsolationEventCount,
    ) -> Result<Self, TenantIsolationError> {
        let evidence = Self {
            authorization_events,
            cross_tenant_attempts,
            cross_tenant_allowed,
            denied_cross_tenant_attempts,
            approvals_required,
            same_tenant_authorized,
        };
        evidence.validate_counts()?;
        Ok(evidence)
    }

    pub fn authorization_events(self) -> TenantIsolationEventCount {
        self.authorization_events
    }

    pub fn cross_tenant_attempts(self) -> TenantIsolationEventCount {
        self.cross_tenant_attempts
    }

    pub fn cross_tenant_allowed(self) -> TenantIsolationEventCount {
        self.cross_tenant_allowed
    }

    pub fn denied_cross_tenant_attempts(self) -> TenantIsolationEventCount {
        self.denied_cross_tenant_attempts
    }

    pub fn approvals_required(self) -> TenantIsolationEventCount {
        self.approvals_required
    }

    pub fn same_tenant_authorized(self) -> TenantIsolationEventCount {
        self.same_tenant_authorized
    }

    fn validate_counts(self) -> Result<(), TenantIsolationError> {
        if self.cross_tenant_allowed.get() + self.denied_cross_tenant_attempts.get()
            != self.cross_tenant_attempts.get()
        {
            return Err(TenantIsolationError::CrossTenantCountMismatch);
        }

        Ok(())
    }
}

// ANCHOR: tenant_isolation_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantIsolationReview {
    actor_tenant_key: TenantKey,
    requested_tenant_key: TenantKey,
    evidence: TenantIsolationEvidence,
    latest_decision_at: DateTime<Utc>,
    review_status: TenantIsolationReviewStatus,
}

impl TenantIsolationReview {
    pub fn actor_tenant_key(&self) -> &TenantKey {
        &self.actor_tenant_key
    }

    pub fn requested_tenant_key(&self) -> &TenantKey {
        &self.requested_tenant_key
    }

    pub fn evidence(&self) -> TenantIsolationEvidence {
        self.evidence
    }

    pub fn latest_decision_at(&self) -> DateTime<Utc> {
        self.latest_decision_at
    }

    pub fn review_status(&self) -> TenantIsolationReviewStatus {
        self.review_status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbTenantIsolationReviewRow {
    pub actor_tenant_key: String,
    pub requested_tenant_key: String,
    pub authorization_events: i64,
    pub cross_tenant_attempts: i64,
    pub cross_tenant_allowed: i64,
    pub denied_cross_tenant_attempts: i64,
    pub approvals_required: i64,
    pub same_tenant_authorized: i64,
    pub latest_decision_at: Option<DateTime<Utc>>,
    pub review_status: String,
}

impl TryFrom<DbTenantIsolationReviewRow> for TenantIsolationReview {
    type Error = TenantIsolationError;

    fn try_from(row: DbTenantIsolationReviewRow) -> Result<Self, Self::Error> {
        let evidence = TenantIsolationEvidence::new(
            TenantIsolationEventCount::try_from_i64(
                row.authorization_events,
                "authorization_events",
            )?,
            TenantIsolationEventCount::try_from_i64(
                row.cross_tenant_attempts,
                "cross_tenant_attempts",
            )?,
            TenantIsolationEventCount::try_from_i64(
                row.cross_tenant_allowed,
                "cross_tenant_allowed",
            )?,
            TenantIsolationEventCount::try_from_i64(
                row.denied_cross_tenant_attempts,
                "denied_cross_tenant_attempts",
            )?,
            TenantIsolationEventCount::try_from_i64(row.approvals_required, "approvals_required")?,
            TenantIsolationEventCount::try_from_i64(
                row.same_tenant_authorized,
                "same_tenant_authorized",
            )?,
        )?;
        let review_status = TenantIsolationReviewStatus::try_from(row.review_status.as_str())?;

        validate_review_status(review_status, evidence)?;

        Ok(Self {
            actor_tenant_key: TenantKey::new(row.actor_tenant_key)?,
            requested_tenant_key: TenantKey::new(row.requested_tenant_key)?,
            evidence,
            latest_decision_at: row
                .latest_decision_at
                .ok_or(TenantIsolationError::MissingLatestDecisionAt)?,
            review_status,
        })
    }
}
// ANCHOR_END: tenant_isolation_row_boundary

fn validate_review_status(
    status: TenantIsolationReviewStatus,
    evidence: TenantIsolationEvidence,
) -> Result<(), TenantIsolationError> {
    let valid = match status {
        TenantIsolationReviewStatus::TenantBoundaryBreach => {
            !evidence.cross_tenant_allowed().is_zero()
        }
        TenantIsolationReviewStatus::CrossTenantDenied => {
            evidence.cross_tenant_allowed().is_zero() && !evidence.cross_tenant_attempts().is_zero()
        }
        TenantIsolationReviewStatus::ApprovalRequired => {
            evidence.cross_tenant_attempts().is_zero() && !evidence.approvals_required().is_zero()
        }
        TenantIsolationReviewStatus::SameTenantAuthorized => {
            evidence.cross_tenant_attempts().is_zero()
                && evidence.approvals_required().is_zero()
                && !evidence.same_tenant_authorized().is_zero()
        }
    };

    if valid {
        Ok(())
    } else {
        Err(TenantIsolationError::InconsistentReviewStatus { status })
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn row() -> DbTenantIsolationReviewRow {
        DbTenantIsolationReviewRow {
            actor_tenant_key: "tenant-alpha".to_string(),
            requested_tenant_key: "tenant-alpha".to_string(),
            authorization_events: 3,
            cross_tenant_attempts: 0,
            cross_tenant_allowed: 0,
            denied_cross_tenant_attempts: 0,
            approvals_required: 0,
            same_tenant_authorized: 3,
            latest_decision_at: Some(Utc::now()),
            review_status: "same_tenant_authorized".to_string(),
        }
    }

    #[test]
    fn same_tenant_authorized_decodes() {
        let review = TenantIsolationReview::try_from(row()).expect("valid review");

        assert_eq!(
            review.review_status(),
            TenantIsolationReviewStatus::SameTenantAuthorized
        );
        assert_eq!(review.evidence().same_tenant_authorized().get(), 3);
    }

    #[test]
    fn cross_tenant_denied_decodes() {
        let mut row = row();
        row.requested_tenant_key = "tenant-beta".to_string();
        row.authorization_events = 2;
        row.cross_tenant_attempts = 2;
        row.denied_cross_tenant_attempts = 2;
        row.same_tenant_authorized = 0;
        row.review_status = "cross_tenant_denied".to_string();

        let review = TenantIsolationReview::try_from(row).expect("valid review");

        assert_eq!(
            review.review_status(),
            TenantIsolationReviewStatus::CrossTenantDenied
        );
        assert_eq!(review.evidence().denied_cross_tenant_attempts().get(), 2);
    }

    #[test]
    fn tenant_boundary_breach_decodes() {
        let mut row = row();
        row.requested_tenant_key = "tenant-beta".to_string();
        row.authorization_events = 1;
        row.cross_tenant_attempts = 1;
        row.cross_tenant_allowed = 1;
        row.same_tenant_authorized = 0;
        row.review_status = "tenant_boundary_breach".to_string();

        let review = TenantIsolationReview::try_from(row).expect("valid review");

        assert_eq!(
            review.review_status(),
            TenantIsolationReviewStatus::TenantBoundaryBreach
        );
        assert_eq!(review.evidence().cross_tenant_allowed().get(), 1);
    }

    #[test]
    fn row_conversion_rejects_unknown_review_status() {
        let mut row = row();
        row.review_status = "maybe_isolated".to_string();

        let error = TenantIsolationReview::try_from(row).expect_err("unknown status must fail");

        assert!(matches!(
            error,
            TenantIsolationError::UnknownReviewStatus { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_negative_count() {
        let mut row = row();
        row.authorization_events = -1;

        let error = TenantIsolationReview::try_from(row).expect_err("negative count must fail");

        assert_eq!(
            error,
            TenantIsolationError::NegativeCount {
                field: "authorization_events",
                value: -1
            }
        );
    }

    #[test]
    fn row_conversion_rejects_cross_tenant_count_mismatch() {
        let mut row = row();
        row.requested_tenant_key = "tenant-beta".to_string();
        row.authorization_events = 2;
        row.cross_tenant_attempts = 2;
        row.denied_cross_tenant_attempts = 1;
        row.same_tenant_authorized = 0;
        row.review_status = "cross_tenant_denied".to_string();

        let error = TenantIsolationReview::try_from(row).expect_err("mismatch must fail");

        assert_eq!(error, TenantIsolationError::CrossTenantCountMismatch);
    }

    #[test]
    fn row_conversion_rejects_missing_latest_decision() {
        let mut row = row();
        row.latest_decision_at = None;

        let error = TenantIsolationReview::try_from(row).expect_err("timestamp must exist");

        assert_eq!(error, TenantIsolationError::MissingLatestDecisionAt);
    }

    #[test]
    fn row_conversion_rejects_inconsistent_review_status() {
        let mut row = row();
        row.review_status = "tenant_boundary_breach".to_string();

        let error = TenantIsolationReview::try_from(row).expect_err("inconsistent status fails");

        assert_eq!(
            error,
            TenantIsolationError::InconsistentReviewStatus {
                status: TenantIsolationReviewStatus::TenantBoundaryBreach
            }
        );
    }
}
