//! Typed cancellation-request primitives.
//!
//! Cancellation has two phases in a production agent system: a user, operator,
//! or policy requests cancellation; then the control plane or worker applies,
//! ignores, or expires that request with durable evidence.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AgentRunId, CancellationReason, DomainError, JobId, JobStatus};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CancellationError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("unknown cancellation status: {value}")]
    UnknownStatus { value: UnknownCancellationStatus },
    #[error("unknown cancellation source: {value}")]
    UnknownSource { value: UnknownCancellationSource },
    #[error("unknown cancellation mode: {value}")]
    UnknownMode { value: UnknownCancellationMode },
    #[error("cancellation expires_at must be after requested_at")]
    ExpirationNotAfterRequest,
    #[error("requested cancellation must not have applied evidence")]
    RequestedHasAppliedEvidence,
    #[error("{status} cancellation requires applied_at")]
    MissingAppliedAt { status: CancellationStatus },
    #[error("{status} cancellation requires observed_job_status")]
    MissingObservedJobStatus { status: CancellationStatus },
    #[error("applied cancellation can only observe pending or running work, got {status:?}")]
    AppliedToNonCancellableStatus { status: JobStatus },
    #[error("ignored-terminal cancellation requires terminal job status, got {status:?}")]
    IgnoredNonTerminalStatus { status: JobStatus },
    #[error("expired cancellation must not have observed_job_status")]
    ExpiredHasObservedJobStatus,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_cancellation_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, CancellationError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(CancellationError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCancellationStatus(String);

impl UnknownCancellationStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownCancellationStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCancellationSource(String);

impl UnknownCancellationSource {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownCancellationSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCancellationMode(String);

impl UnknownCancellationMode {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownCancellationMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancellationStatus {
    Requested,
    Applied,
    IgnoredTerminal,
    Expired,
}

impl CancellationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Applied => "applied",
            Self::IgnoredTerminal => "ignored_terminal",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CancellationStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for CancellationStatus {
    type Error = CancellationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "requested" => Ok(Self::Requested),
            "applied" => Ok(Self::Applied),
            "ignored_terminal" => Ok(Self::IgnoredTerminal),
            "expired" => Ok(Self::Expired),
            value => Err(CancellationError::UnknownStatus {
                value: UnknownCancellationStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancellationSource {
    User,
    Operator,
    System,
    Policy,
}

impl CancellationSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Operator => "operator",
            Self::System => "system",
            Self::Policy => "policy",
        }
    }
}

impl TryFrom<&str> for CancellationSource {
    type Error = CancellationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "user" => Ok(Self::User),
            "operator" => Ok(Self::Operator),
            "system" => Ok(Self::System),
            "policy" => Ok(Self::Policy),
            value => Err(CancellationError::UnknownSource {
                value: UnknownCancellationSource::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancellationMode {
    Graceful,
    Immediate,
    AfterCurrentStep,
}

impl CancellationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Graceful => "graceful",
            Self::Immediate => "immediate",
            Self::AfterCurrentStep => "after_current_step",
        }
    }
}

impl TryFrom<&str> for CancellationMode {
    type Error = CancellationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "graceful" => Ok(Self::Graceful),
            "immediate" => Ok(Self::Immediate),
            "after_current_step" => Ok(Self::AfterCurrentStep),
            value => Err(CancellationError::UnknownMode {
                value: UnknownCancellationMode::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CancellationRequestId(Uuid);

impl CancellationRequestId {
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

impl Default for CancellationRequestId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancellationActor(String);

impl CancellationActor {
    pub fn new(value: impl Into<String>) -> Result<Self, CancellationError> {
        Ok(Self(non_empty_cancellation_text(
            value,
            "cancellation_actor",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancellationRequestedAt(DateTime<Utc>);

impl CancellationRequestedAt {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancellationAppliedAt(DateTime<Utc>);

impl CancellationAppliedAt {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancellationExpiresAt(DateTime<Utc>);

impl CancellationExpiresAt {
    pub fn new(
        requested_at: CancellationRequestedAt,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, CancellationError> {
        if expires_at <= requested_at.as_datetime() {
            return Err(CancellationError::ExpirationNotAfterRequest);
        }

        Ok(Self(expires_at))
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancellationRequested;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancellationApplied;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancellationIgnoredTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancellationExpired;

// ANCHOR: cancellation_typestate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancellationRequest<State> {
    id: CancellationRequestId,
    job_id: JobId,
    run_id: Option<AgentRunId>,
    requested_by: CancellationActor,
    source: CancellationSource,
    mode: CancellationMode,
    reason: CancellationReason,
    requested_at: CancellationRequestedAt,
    expires_at: Option<CancellationExpiresAt>,
    applied_at: Option<CancellationAppliedAt>,
    observed_job_status: Option<JobStatus>,
    state: PhantomData<State>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancellationRequestDraft {
    pub job_id: JobId,
    pub run_id: Option<AgentRunId>,
    pub requested_by: CancellationActor,
    pub source: CancellationSource,
    pub mode: CancellationMode,
    pub reason: CancellationReason,
    pub requested_at: CancellationRequestedAt,
    pub expires_at: Option<CancellationExpiresAt>,
}

impl CancellationRequest<CancellationRequested> {
    pub fn request(draft: CancellationRequestDraft) -> Self {
        Self {
            id: CancellationRequestId::new(),
            job_id: draft.job_id,
            run_id: draft.run_id,
            requested_by: draft.requested_by,
            source: draft.source,
            mode: draft.mode,
            reason: draft.reason,
            requested_at: draft.requested_at,
            expires_at: draft.expires_at,
            applied_at: None,
            observed_job_status: None,
            state: PhantomData,
        }
    }

    pub fn apply(
        self,
        observed_job_status: JobStatus,
        applied_at: CancellationAppliedAt,
    ) -> Result<CancellationRequest<CancellationApplied>, CancellationError> {
        if !matches!(observed_job_status, JobStatus::Pending | JobStatus::Running) {
            return Err(CancellationError::AppliedToNonCancellableStatus {
                status: observed_job_status,
            });
        }

        Ok(self.transition(applied_at, Some(observed_job_status)))
    }

    pub fn ignore_terminal(
        self,
        observed_job_status: JobStatus,
        applied_at: CancellationAppliedAt,
    ) -> Result<CancellationRequest<CancellationIgnoredTerminal>, CancellationError> {
        if !matches!(
            observed_job_status,
            JobStatus::Succeeded | JobStatus::Dead | JobStatus::Cancelled
        ) {
            return Err(CancellationError::IgnoredNonTerminalStatus {
                status: observed_job_status,
            });
        }

        Ok(self.transition(applied_at, Some(observed_job_status)))
    }

    pub fn expire(
        self,
        applied_at: CancellationAppliedAt,
    ) -> CancellationRequest<CancellationExpired> {
        self.transition(applied_at, None)
    }
}
// ANCHOR_END: cancellation_typestate

impl<State> CancellationRequest<State> {
    fn transition<Next>(
        self,
        applied_at: CancellationAppliedAt,
        observed_job_status: Option<JobStatus>,
    ) -> CancellationRequest<Next> {
        CancellationRequest {
            id: self.id,
            job_id: self.job_id,
            run_id: self.run_id,
            requested_by: self.requested_by,
            source: self.source,
            mode: self.mode,
            reason: self.reason,
            requested_at: self.requested_at,
            expires_at: self.expires_at,
            applied_at: Some(applied_at),
            observed_job_status,
            state: PhantomData,
        }
    }

    pub fn id(&self) -> CancellationRequestId {
        self.id
    }

    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    pub fn run_id(&self) -> Option<AgentRunId> {
        self.run_id
    }

    pub fn requested_by(&self) -> &CancellationActor {
        &self.requested_by
    }

    pub fn source(&self) -> CancellationSource {
        self.source
    }

    pub fn mode(&self) -> CancellationMode {
        self.mode
    }

    pub fn reason(&self) -> &CancellationReason {
        &self.reason
    }

    pub fn requested_at(&self) -> CancellationRequestedAt {
        self.requested_at
    }

    pub fn expires_at(&self) -> Option<CancellationExpiresAt> {
        self.expires_at
    }

    pub fn applied_at(&self) -> Option<CancellationAppliedAt> {
        self.applied_at
    }

    pub fn observed_job_status(&self) -> Option<JobStatus> {
        self.observed_job_status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancellationRecord {
    Requested(CancellationRequest<CancellationRequested>),
    Applied(CancellationRequest<CancellationApplied>),
    IgnoredTerminal(CancellationRequest<CancellationIgnoredTerminal>),
    Expired(CancellationRequest<CancellationExpired>),
}

impl CancellationRecord {
    pub fn status(&self) -> CancellationStatus {
        match self {
            Self::Requested(_) => CancellationStatus::Requested,
            Self::Applied(_) => CancellationStatus::Applied,
            Self::IgnoredTerminal(_) => CancellationStatus::IgnoredTerminal,
            Self::Expired(_) => CancellationStatus::Expired,
        }
    }
}

// ANCHOR: cancellation_row_boundary
#[derive(Debug, Clone)]
pub struct DbCancellationRequestRow {
    pub id: Uuid,
    pub job_id: Uuid,
    pub run_id: Option<Uuid>,
    pub status: String,
    pub requested_by: String,
    pub source: String,
    pub mode: String,
    pub reason: String,
    pub requested_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub observed_job_status: Option<String>,
}

impl TryFrom<DbCancellationRequestRow> for CancellationRecord {
    type Error = CancellationError;

    fn try_from(row: DbCancellationRequestRow) -> Result<Self, Self::Error> {
        let requested_at = CancellationRequestedAt::new(row.requested_at);
        let base = CancellationRequest::<CancellationRequested> {
            id: CancellationRequestId::from_uuid(row.id),
            job_id: JobId::from_uuid(row.job_id),
            run_id: row.run_id.map(AgentRunId::from_uuid),
            requested_by: CancellationActor::new(row.requested_by)?,
            source: CancellationSource::try_from(row.source.as_str())?,
            mode: CancellationMode::try_from(row.mode.as_str())?,
            reason: CancellationReason::new(row.reason)?,
            requested_at,
            expires_at: row
                .expires_at
                .map(|expires_at| CancellationExpiresAt::new(requested_at, expires_at))
                .transpose()?,
            applied_at: None,
            observed_job_status: None,
            state: PhantomData,
        };

        let status = CancellationStatus::try_from(row.status.as_str())?;
        match status {
            CancellationStatus::Requested => {
                if row.applied_at.is_some() || row.observed_job_status.is_some() {
                    return Err(CancellationError::RequestedHasAppliedEvidence);
                }
                Ok(CancellationRecord::Requested(base))
            }
            CancellationStatus::Applied => {
                let applied_at = applied_at(status, row.applied_at)?;
                let observed_status = observed_status(status, row.observed_job_status)?;
                Ok(CancellationRecord::Applied(
                    base.apply(observed_status, applied_at)?,
                ))
            }
            CancellationStatus::IgnoredTerminal => {
                let applied_at = applied_at(status, row.applied_at)?;
                let observed_status = observed_status(status, row.observed_job_status)?;
                Ok(CancellationRecord::IgnoredTerminal(
                    base.ignore_terminal(observed_status, applied_at)?,
                ))
            }
            CancellationStatus::Expired => {
                let applied_at = applied_at(status, row.applied_at)?;
                if row.observed_job_status.is_some() {
                    return Err(CancellationError::ExpiredHasObservedJobStatus);
                }
                Ok(CancellationRecord::Expired(base.expire(applied_at)))
            }
        }
    }
}
// ANCHOR_END: cancellation_row_boundary

fn applied_at(
    status: CancellationStatus,
    applied_at: Option<DateTime<Utc>>,
) -> Result<CancellationAppliedAt, CancellationError> {
    applied_at
        .map(CancellationAppliedAt::new)
        .ok_or(CancellationError::MissingAppliedAt { status })
}

fn observed_status(
    status: CancellationStatus,
    observed_status: Option<String>,
) -> Result<JobStatus, CancellationError> {
    let observed_status =
        observed_status.ok_or(CancellationError::MissingObservedJobStatus { status })?;
    Ok(JobStatus::try_from(observed_status.as_str())?)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    fn actor() -> CancellationActor {
        CancellationActor::new("operator-a").expect("valid cancellation actor")
    }

    fn reason() -> CancellationReason {
        CancellationReason::new("operator stopped unsafe work").expect("valid cancellation reason")
    }

    fn requested() -> CancellationRequest<CancellationRequested> {
        let requested_at = CancellationRequestedAt::new(now());
        CancellationRequest::request(CancellationRequestDraft {
            job_id: JobId::new(),
            run_id: Some(AgentRunId::new()),
            requested_by: actor(),
            source: CancellationSource::Operator,
            mode: CancellationMode::Graceful,
            reason: reason(),
            requested_at,
            expires_at: Some(
                CancellationExpiresAt::new(requested_at, now() + chrono::Duration::minutes(5))
                    .expect("valid expiration"),
            ),
        })
    }

    fn row(status: &str) -> DbCancellationRequestRow {
        DbCancellationRequestRow {
            id: Uuid::new_v4(),
            job_id: Uuid::new_v4(),
            run_id: Some(Uuid::new_v4()),
            status: status.to_string(),
            requested_by: "operator-a".to_string(),
            source: "operator".to_string(),
            mode: "graceful".to_string(),
            reason: "operator stopped unsafe work".to_string(),
            requested_at: now(),
            expires_at: Some(now() + chrono::Duration::minutes(5)),
            applied_at: Some(now() + chrono::Duration::seconds(30)),
            observed_job_status: Some("running".to_string()),
        }
    }

    #[test]
    fn requested_cancellation_can_be_applied_to_running_work() {
        let applied = requested()
            .apply(
                JobStatus::Running,
                CancellationAppliedAt::new(now() + chrono::Duration::seconds(10)),
            )
            .expect("running work can be cancelled");

        assert_eq!(applied.observed_job_status(), Some(JobStatus::Running));
        assert!(applied.applied_at().is_some());
    }

    #[test]
    fn requested_cancellation_cannot_be_applied_to_terminal_work() {
        let error = requested()
            .apply(
                JobStatus::Succeeded,
                CancellationAppliedAt::new(now() + chrono::Duration::seconds(10)),
            )
            .expect_err("terminal work cannot be cancelled");

        assert_eq!(
            error,
            CancellationError::AppliedToNonCancellableStatus {
                status: JobStatus::Succeeded
            }
        );
    }

    #[test]
    fn requested_cancellation_can_be_ignored_for_terminal_work() {
        let ignored = requested()
            .ignore_terminal(
                JobStatus::Succeeded,
                CancellationAppliedAt::new(now() + chrono::Duration::seconds(10)),
            )
            .expect("terminal work can be ignored with evidence");

        assert_eq!(ignored.observed_job_status(), Some(JobStatus::Succeeded));
    }

    #[test]
    fn row_conversion_accepts_requested_without_applied_evidence() {
        let record = CancellationRecord::try_from(DbCancellationRequestRow {
            applied_at: None,
            observed_job_status: None,
            ..row("requested")
        })
        .expect("valid requested row");

        assert_eq!(record.status(), CancellationStatus::Requested);
    }

    #[test]
    fn row_conversion_rejects_applied_without_observed_status() {
        let error = CancellationRecord::try_from(DbCancellationRequestRow {
            observed_job_status: None,
            ..row("applied")
        })
        .expect_err("applied row needs observed status");

        assert_eq!(
            error,
            CancellationError::MissingObservedJobStatus {
                status: CancellationStatus::Applied
            }
        );
    }

    #[test]
    fn row_conversion_rejects_ignored_terminal_with_running_status() {
        let error = CancellationRecord::try_from(row("ignored_terminal"))
            .expect_err("ignored-terminal must observe terminal status");

        assert_eq!(
            error,
            CancellationError::IgnoredNonTerminalStatus {
                status: JobStatus::Running
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_source() {
        let error = CancellationRecord::try_from(DbCancellationRequestRow {
            source: "cron".to_string(),
            ..row("applied")
        })
        .expect_err("unknown source");

        assert!(matches!(error, CancellationError::UnknownSource { .. }));
    }

    #[test]
    fn expires_at_must_be_after_requested_at() {
        let requested_at = CancellationRequestedAt::new(now());
        let error = CancellationExpiresAt::new(requested_at, now())
            .expect_err("expiration must be after request");

        assert_eq!(error, CancellationError::ExpirationNotAfterRequest);
    }
}
