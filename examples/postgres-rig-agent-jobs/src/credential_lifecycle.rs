//! Typed credential lifecycle review for long-running agent systems.
//!
//! The database records secret references and rotation evidence, never secret
//! values. This module decodes the read-only `credential_rotation_review.sql`
//! output into domain values so operators can see which credential families
//! need rotation, verification, revocation, or incident response.

use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CredentialLifecycleError {
    #[error("unknown credential kind: {value:?}")]
    UnknownCredentialKind { value: UnknownCredentialKind },
    #[error("unknown credential review status: {value:?}")]
    UnknownReviewStatus {
        value: UnknownCredentialReviewStatus,
    },
    #[error("{field} cannot be negative, got {value}")]
    NegativeCount { field: &'static str, value: i64 },
    #[error("{field} cannot exceed managed_credentials")]
    CountExceedsManagedCredentials { field: &'static str },
    #[error("credential review status {status:?} conflicts with credential counts")]
    InconsistentReviewStatus { status: CredentialReviewStatus },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCredentialKind(String);

impl UnknownCredentialKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCredentialReviewStatus(String);

impl UnknownCredentialReviewStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CredentialCount(u64);

impl CredentialCount {
    pub fn try_from_i64(value: i64, field: &'static str) -> Result<Self, CredentialLifecycleError> {
        let value = u64::try_from(value)
            .map_err(|_| CredentialLifecycleError::NegativeCount { field, value })?;

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
pub enum CredentialKind {
    ProviderApiKey,
    DatabaseUrl,
    OperatorToken,
    WebhookSecret,
    ServiceAccountKey,
    CiSecret,
    EncryptionKey,
}

impl TryFrom<&str> for CredentialKind {
    type Error = CredentialLifecycleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "provider_api_key" => Ok(Self::ProviderApiKey),
            "database_url" => Ok(Self::DatabaseUrl),
            "operator_token" => Ok(Self::OperatorToken),
            "webhook_secret" => Ok(Self::WebhookSecret),
            "service_account_key" => Ok(Self::ServiceAccountKey),
            "ci_secret" => Ok(Self::CiSecret),
            "encryption_key" => Ok(Self::EncryptionKey),
            value => Err(CredentialLifecycleError::UnknownCredentialKind {
                value: UnknownCredentialKind::new(value),
            }),
        }
    }
}

impl CredentialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProviderApiKey => "provider_api_key",
            Self::DatabaseUrl => "database_url",
            Self::OperatorToken => "operator_token",
            Self::WebhookSecret => "webhook_secret",
            Self::ServiceAccountKey => "service_account_key",
            Self::CiSecret => "ci_secret",
            Self::EncryptionKey => "encryption_key",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialReviewStatus {
    CredentialHealthOk,
    VerificationStale,
    RotationDue,
    RotationOverdue,
    ExposureIncidentOpen,
    NoCredentialsRegistered,
}

impl TryFrom<&str> for CredentialReviewStatus {
    type Error = CredentialLifecycleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "credential_health_ok" => Ok(Self::CredentialHealthOk),
            "verification_stale" => Ok(Self::VerificationStale),
            "rotation_due" => Ok(Self::RotationDue),
            "rotation_overdue" => Ok(Self::RotationOverdue),
            "exposure_incident_open" => Ok(Self::ExposureIncidentOpen),
            "no_credentials_registered" => Ok(Self::NoCredentialsRegistered),
            value => Err(CredentialLifecycleError::UnknownReviewStatus {
                value: UnknownCredentialReviewStatus::new(value),
            }),
        }
    }
}

impl CredentialReviewStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CredentialHealthOk => "credential_health_ok",
            Self::VerificationStale => "verification_stale",
            Self::RotationDue => "rotation_due",
            Self::RotationOverdue => "rotation_overdue",
            Self::ExposureIncidentOpen => "exposure_incident_open",
            Self::NoCredentialsRegistered => "no_credentials_registered",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CredentialRotationEvidence {
    managed_credentials: CredentialCount,
    rotation_due: CredentialCount,
    overdue_rotation: CredentialCount,
    open_exposure_incidents: CredentialCount,
    stale_verification: CredentialCount,
    revoked_credentials_30d: CredentialCount,
}

impl CredentialRotationEvidence {
    pub fn new(
        managed_credentials: CredentialCount,
        rotation_due: CredentialCount,
        overdue_rotation: CredentialCount,
        open_exposure_incidents: CredentialCount,
        stale_verification: CredentialCount,
        revoked_credentials_30d: CredentialCount,
    ) -> Result<Self, CredentialLifecycleError> {
        let evidence = Self {
            managed_credentials,
            rotation_due,
            overdue_rotation,
            open_exposure_incidents,
            stale_verification,
            revoked_credentials_30d,
        };
        evidence.validate_counts()?;
        Ok(evidence)
    }

    pub fn managed_credentials(self) -> CredentialCount {
        self.managed_credentials
    }

    pub fn rotation_due(self) -> CredentialCount {
        self.rotation_due
    }

    pub fn overdue_rotation(self) -> CredentialCount {
        self.overdue_rotation
    }

    pub fn open_exposure_incidents(self) -> CredentialCount {
        self.open_exposure_incidents
    }

    pub fn stale_verification(self) -> CredentialCount {
        self.stale_verification
    }

    pub fn revoked_credentials_30d(self) -> CredentialCount {
        self.revoked_credentials_30d
    }

    fn validate_counts(self) -> Result<(), CredentialLifecycleError> {
        let managed = self.managed_credentials.get();
        for (field, count) in [
            ("rotation_due", self.rotation_due),
            ("overdue_rotation", self.overdue_rotation),
            ("open_exposure_incidents", self.open_exposure_incidents),
            ("stale_verification", self.stale_verification),
        ] {
            if count.get() > managed {
                return Err(CredentialLifecycleError::CountExceedsManagedCredentials { field });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialRotationReview {
    credential_kind: CredentialKind,
    evidence: CredentialRotationEvidence,
    latest_rotation_at: Option<DateTime<Utc>>,
    next_rotation_due_at: Option<DateTime<Utc>>,
    review_status: CredentialReviewStatus,
}

impl CredentialRotationReview {
    pub fn credential_kind(&self) -> CredentialKind {
        self.credential_kind
    }

    pub fn evidence(&self) -> CredentialRotationEvidence {
        self.evidence
    }

    pub fn latest_rotation_at(&self) -> Option<DateTime<Utc>> {
        self.latest_rotation_at
    }

    pub fn next_rotation_due_at(&self) -> Option<DateTime<Utc>> {
        self.next_rotation_due_at
    }

    pub fn review_status(&self) -> CredentialReviewStatus {
        self.review_status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbCredentialRotationReviewRow {
    pub credential_kind: String,
    pub managed_credentials: i64,
    pub rotation_due: i64,
    pub overdue_rotation: i64,
    pub open_exposure_incidents: i64,
    pub stale_verification: i64,
    pub revoked_credentials_30d: i64,
    pub latest_rotation_at: Option<DateTime<Utc>>,
    pub next_rotation_due_at: Option<DateTime<Utc>>,
    pub review_status: String,
}

impl TryFrom<DbCredentialRotationReviewRow> for CredentialRotationReview {
    type Error = CredentialLifecycleError;

    fn try_from(row: DbCredentialRotationReviewRow) -> Result<Self, Self::Error> {
        let credential_kind = CredentialKind::try_from(row.credential_kind.as_str())?;
        let evidence = CredentialRotationEvidence::new(
            CredentialCount::try_from_i64(row.managed_credentials, "managed_credentials")?,
            CredentialCount::try_from_i64(row.rotation_due, "rotation_due")?,
            CredentialCount::try_from_i64(row.overdue_rotation, "overdue_rotation")?,
            CredentialCount::try_from_i64(row.open_exposure_incidents, "open_exposure_incidents")?,
            CredentialCount::try_from_i64(row.stale_verification, "stale_verification")?,
            CredentialCount::try_from_i64(row.revoked_credentials_30d, "revoked_credentials_30d")?,
        )?;
        let review_status = CredentialReviewStatus::try_from(row.review_status.as_str())?;

        validate_review_status(review_status, evidence)?;

        Ok(Self {
            credential_kind,
            evidence,
            latest_rotation_at: row.latest_rotation_at,
            next_rotation_due_at: row.next_rotation_due_at,
            review_status,
        })
    }
}

fn validate_review_status(
    status: CredentialReviewStatus,
    evidence: CredentialRotationEvidence,
) -> Result<(), CredentialLifecycleError> {
    let valid = match status {
        CredentialReviewStatus::ExposureIncidentOpen => {
            !evidence.open_exposure_incidents().is_zero()
        }
        CredentialReviewStatus::RotationOverdue => {
            evidence.open_exposure_incidents().is_zero() && !evidence.overdue_rotation().is_zero()
        }
        CredentialReviewStatus::RotationDue => {
            evidence.open_exposure_incidents().is_zero()
                && evidence.overdue_rotation().is_zero()
                && !evidence.rotation_due().is_zero()
        }
        CredentialReviewStatus::VerificationStale => {
            evidence.open_exposure_incidents().is_zero()
                && evidence.overdue_rotation().is_zero()
                && evidence.rotation_due().is_zero()
                && !evidence.stale_verification().is_zero()
        }
        CredentialReviewStatus::NoCredentialsRegistered => {
            evidence.managed_credentials().is_zero()
                && evidence.open_exposure_incidents().is_zero()
                && evidence.overdue_rotation().is_zero()
                && evidence.rotation_due().is_zero()
                && evidence.stale_verification().is_zero()
        }
        CredentialReviewStatus::CredentialHealthOk => {
            !evidence.managed_credentials().is_zero()
                && evidence.open_exposure_incidents().is_zero()
                && evidence.overdue_rotation().is_zero()
                && evidence.rotation_due().is_zero()
                && evidence.stale_verification().is_zero()
        }
    };

    if valid {
        Ok(())
    } else {
        Err(CredentialLifecycleError::InconsistentReviewStatus { status })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row() -> DbCredentialRotationReviewRow {
        DbCredentialRotationReviewRow {
            credential_kind: "provider_api_key".to_string(),
            managed_credentials: 1,
            rotation_due: 0,
            overdue_rotation: 0,
            open_exposure_incidents: 0,
            stale_verification: 0,
            revoked_credentials_30d: 0,
            latest_rotation_at: Some(Utc::now()),
            next_rotation_due_at: Some(Utc::now()),
            review_status: "credential_health_ok".to_string(),
        }
    }

    #[test]
    fn healthy_credential_review_decodes() {
        let review = CredentialRotationReview::try_from(row()).expect("valid credential review");

        assert_eq!(review.credential_kind(), CredentialKind::ProviderApiKey);
        assert_eq!(
            review.review_status(),
            CredentialReviewStatus::CredentialHealthOk
        );
        assert_eq!(review.evidence().managed_credentials().get(), 1);
    }

    #[test]
    fn exposure_incident_review_decodes() {
        let mut row = row();
        row.managed_credentials = 2;
        row.open_exposure_incidents = 1;
        row.review_status = "exposure_incident_open".to_string();

        let review =
            CredentialRotationReview::try_from(row).expect("valid exposure incident review");

        assert_eq!(
            review.review_status(),
            CredentialReviewStatus::ExposureIncidentOpen
        );
        assert_eq!(review.evidence().open_exposure_incidents().get(), 1);
    }

    #[test]
    fn no_credentials_registered_decodes() {
        let mut row = row();
        row.credential_kind = "database_url".to_string();
        row.managed_credentials = 0;
        row.latest_rotation_at = None;
        row.next_rotation_due_at = None;
        row.review_status = "no_credentials_registered".to_string();

        let review =
            CredentialRotationReview::try_from(row).expect("valid empty credential review");

        assert_eq!(review.credential_kind(), CredentialKind::DatabaseUrl);
        assert_eq!(
            review.review_status(),
            CredentialReviewStatus::NoCredentialsRegistered
        );
    }

    #[test]
    fn row_conversion_rejects_negative_count() {
        let mut row = row();
        row.managed_credentials = -1;

        let error = CredentialRotationReview::try_from(row).expect_err("negative count rejected");

        assert_eq!(
            error,
            CredentialLifecycleError::NegativeCount {
                field: "managed_credentials",
                value: -1,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_credential_kind() {
        let mut row = row();
        row.credential_kind = "raw_env_dump".to_string();

        let error = CredentialRotationReview::try_from(row).expect_err("unknown kind rejected");

        assert_eq!(
            error,
            CredentialLifecycleError::UnknownCredentialKind {
                value: UnknownCredentialKind::new("raw_env_dump"),
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_review_status() {
        let mut row = row();
        row.review_status = "ignore_credentials".to_string();

        let error = CredentialRotationReview::try_from(row).expect_err("unknown status rejected");

        assert_eq!(
            error,
            CredentialLifecycleError::UnknownReviewStatus {
                value: UnknownCredentialReviewStatus::new("ignore_credentials"),
            }
        );
    }

    #[test]
    fn row_conversion_rejects_issue_count_above_managed_credentials() {
        let mut row = row();
        row.managed_credentials = 1;
        row.overdue_rotation = 2;
        row.review_status = "rotation_overdue".to_string();

        let error =
            CredentialRotationReview::try_from(row).expect_err("issue count cannot exceed managed");

        assert_eq!(
            error,
            CredentialLifecycleError::CountExceedsManagedCredentials {
                field: "overdue_rotation",
            }
        );
    }

    #[test]
    fn row_conversion_rejects_healthy_status_with_due_rotation() {
        let mut row = row();
        row.rotation_due = 1;

        let error =
            CredentialRotationReview::try_from(row).expect_err("status must match due work");

        assert_eq!(
            error,
            CredentialLifecycleError::InconsistentReviewStatus {
                status: CredentialReviewStatus::CredentialHealthOk,
            }
        );
    }
}
