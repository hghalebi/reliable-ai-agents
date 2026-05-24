//! Human approval lifecycle types.
//!
//! Approval is durable authority, not conversation. This module models approval
//! requests with typestate so only requested approvals can be decided.

use std::marker::PhantomData;

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::AgentRunId;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ApprovalError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("unknown approval status: {value}")]
    UnknownStatus { value: UnknownApprovalStatus },
    #[error("approval request is missing a reason")]
    MissingReason,
    #[error("{status} approval request requires decided_by and decided_at")]
    MissingDecisionEvidence { status: ApprovalStatus },
}

fn non_empty_approval_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, ApprovalError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(ApprovalError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownApprovalStatus(String);

impl UnknownApprovalStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownApprovalStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalStatus {
    Requested,
    Approved,
    Rejected,
    Expired,
    Cancelled,
}

impl ApprovalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for ApprovalStatus {
    type Error = ApprovalError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "requested" => Ok(Self::Requested),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "expired" => Ok(Self::Expired),
            "cancelled" => Ok(Self::Cancelled),
            value => Err(ApprovalError::UnknownStatus {
                value: UnknownApprovalStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ApprovalRequestId(Uuid);

impl ApprovalRequestId {
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

impl Default for ApprovalRequestId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalActor(String);

impl ApprovalActor {
    pub fn new(value: impl Into<String>) -> Result<Self, ApprovalError> {
        Ok(Self(non_empty_approval_text(value, "approval_actor")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalReason(String);

impl ApprovalReason {
    pub fn new(value: impl Into<String>) -> Result<Self, ApprovalError> {
        Ok(Self(non_empty_approval_text(value, "approval_reason")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApprovalRequestedAt(DateTime<Utc>);

impl ApprovalRequestedAt {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApprovalDecidedAt(DateTime<Utc>);

impl ApprovalDecidedAt {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApprovalExpiresAt(DateTime<Utc>);

impl ApprovalExpiresAt {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Requested;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Approved;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rejected;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Expired;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

// ANCHOR: approval_typestate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequest<State> {
    id: ApprovalRequestId,
    run_id: AgentRunId,
    requested_by: ApprovalActor,
    decided_by: Option<ApprovalActor>,
    reason: ApprovalReason,
    requested_at: ApprovalRequestedAt,
    decided_at: Option<ApprovalDecidedAt>,
    expires_at: Option<ApprovalExpiresAt>,
    state: PhantomData<State>,
}

impl ApprovalRequest<Requested> {
    pub fn request(
        run_id: AgentRunId,
        requested_by: ApprovalActor,
        reason: ApprovalReason,
        requested_at: ApprovalRequestedAt,
        expires_at: Option<ApprovalExpiresAt>,
    ) -> Self {
        Self {
            id: ApprovalRequestId::new(),
            run_id,
            requested_by,
            decided_by: None,
            reason,
            requested_at,
            decided_at: None,
            expires_at,
            state: PhantomData,
        }
    }

    pub fn approve(
        self,
        decided_by: ApprovalActor,
        decided_at: ApprovalDecidedAt,
    ) -> ApprovalRequest<Approved> {
        self.transition(decided_by, decided_at)
    }

    pub fn reject(
        self,
        decided_by: ApprovalActor,
        decided_at: ApprovalDecidedAt,
    ) -> ApprovalRequest<Rejected> {
        self.transition(decided_by, decided_at)
    }

    pub fn expire(self, decided_at: ApprovalDecidedAt) -> ApprovalRequest<Expired> {
        self.transition_system(decided_at)
    }

    pub fn cancel(
        self,
        decided_by: ApprovalActor,
        decided_at: ApprovalDecidedAt,
    ) -> ApprovalRequest<Cancelled> {
        self.transition(decided_by, decided_at)
    }
}
// ANCHOR_END: approval_typestate

impl<State> ApprovalRequest<State> {
    fn transition<Next>(
        self,
        decided_by: ApprovalActor,
        decided_at: ApprovalDecidedAt,
    ) -> ApprovalRequest<Next> {
        ApprovalRequest {
            id: self.id,
            run_id: self.run_id,
            requested_by: self.requested_by,
            decided_by: Some(decided_by),
            reason: self.reason,
            requested_at: self.requested_at,
            decided_at: Some(decided_at),
            expires_at: self.expires_at,
            state: PhantomData,
        }
    }

    fn transition_system<Next>(self, decided_at: ApprovalDecidedAt) -> ApprovalRequest<Next> {
        ApprovalRequest {
            id: self.id,
            run_id: self.run_id,
            requested_by: self.requested_by,
            decided_by: None,
            reason: self.reason,
            requested_at: self.requested_at,
            decided_at: Some(decided_at),
            expires_at: self.expires_at,
            state: PhantomData,
        }
    }

    pub fn id(&self) -> ApprovalRequestId {
        self.id
    }

    pub fn run_id(&self) -> AgentRunId {
        self.run_id
    }

    pub fn requested_by(&self) -> &ApprovalActor {
        &self.requested_by
    }

    pub fn decided_by(&self) -> Option<&ApprovalActor> {
        self.decided_by.as_ref()
    }

    pub fn reason(&self) -> &ApprovalReason {
        &self.reason
    }

    pub fn requested_at(&self) -> ApprovalRequestedAt {
        self.requested_at
    }

    pub fn decided_at(&self) -> Option<ApprovalDecidedAt> {
        self.decided_at
    }

    pub fn expires_at(&self) -> Option<ApprovalExpiresAt> {
        self.expires_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalRecord {
    Requested(ApprovalRequest<Requested>),
    Approved(ApprovalRequest<Approved>),
    Rejected(ApprovalRequest<Rejected>),
    Expired(ApprovalRequest<Expired>),
    Cancelled(ApprovalRequest<Cancelled>),
}

impl ApprovalRecord {
    pub fn status(&self) -> ApprovalStatus {
        match self {
            Self::Requested(_) => ApprovalStatus::Requested,
            Self::Approved(_) => ApprovalStatus::Approved,
            Self::Rejected(_) => ApprovalStatus::Rejected,
            Self::Expired(_) => ApprovalStatus::Expired,
            Self::Cancelled(_) => ApprovalStatus::Cancelled,
        }
    }
}

// ANCHOR: approval_row_boundary
#[derive(Debug, Clone)]
pub struct DbHumanApprovalRequestRow {
    pub id: Uuid,
    pub run_id: Uuid,
    pub status: String,
    pub requested_by: String,
    pub decided_by: Option<String>,
    pub reason: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl TryFrom<DbHumanApprovalRequestRow> for ApprovalRecord {
    type Error = ApprovalError;

    fn try_from(row: DbHumanApprovalRequestRow) -> Result<Self, Self::Error> {
        let DbHumanApprovalRequestRow {
            id,
            run_id,
            status,
            requested_by,
            decided_by,
            reason,
            requested_at,
            decided_at,
            expires_at,
        } = row;

        let status = ApprovalStatus::try_from(status.as_str())?;
        let base = ApprovalRequest::<Requested> {
            id: ApprovalRequestId::from_uuid(id),
            run_id: AgentRunId::from_uuid(run_id),
            requested_by: ApprovalActor::new(requested_by)?,
            decided_by: None,
            reason: ApprovalReason::new(reason.ok_or(ApprovalError::MissingReason)?)?,
            requested_at: ApprovalRequestedAt::new(requested_at),
            decided_at: None,
            expires_at: expires_at.map(ApprovalExpiresAt::new),
            state: PhantomData,
        };

        match status {
            ApprovalStatus::Requested => Ok(ApprovalRecord::Requested(base)),
            ApprovalStatus::Approved => Ok(ApprovalRecord::Approved(decided_request(
                base, status, decided_by, decided_at,
            )?)),
            ApprovalStatus::Rejected => Ok(ApprovalRecord::Rejected(decided_request(
                base, status, decided_by, decided_at,
            )?)),
            ApprovalStatus::Expired => {
                let decided_at = decided_at.ok_or(ApprovalError::MissingDecisionEvidence {
                    status: ApprovalStatus::Expired,
                })?;
                Ok(ApprovalRecord::Expired(
                    base.transition_system(ApprovalDecidedAt::new(decided_at)),
                ))
            }
            ApprovalStatus::Cancelled => Ok(ApprovalRecord::Cancelled(decided_request(
                base, status, decided_by, decided_at,
            )?)),
        }
    }
}
// ANCHOR_END: approval_row_boundary

fn decided_request<Next>(
    base: ApprovalRequest<Requested>,
    status: ApprovalStatus,
    decided_by: Option<String>,
    decided_at: Option<DateTime<Utc>>,
) -> Result<ApprovalRequest<Next>, ApprovalError> {
    let decided_by = decided_by.ok_or(ApprovalError::MissingDecisionEvidence { status })?;
    let decided_at = decided_at.ok_or(ApprovalError::MissingDecisionEvidence { status })?;

    Ok(base.transition(
        ApprovalActor::new(decided_by)?,
        ApprovalDecidedAt::new(decided_at),
    ))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 11, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    fn actor(value: &str) -> ApprovalActor {
        ApprovalActor::new(value).expect("valid actor")
    }

    fn reason(value: &str) -> ApprovalReason {
        ApprovalReason::new(value).expect("valid reason")
    }

    fn requested() -> ApprovalRequest<Requested> {
        ApprovalRequest::request(
            AgentRunId::new(),
            actor("agent-worker"),
            reason("rollback affects production traffic"),
            ApprovalRequestedAt::new(now()),
            None,
        )
    }

    fn row(status: &str) -> DbHumanApprovalRequestRow {
        DbHumanApprovalRequestRow {
            id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            status: status.to_string(),
            requested_by: "agent-worker".to_string(),
            decided_by: Some("operator-a".to_string()),
            reason: Some("rollback affects production traffic".to_string()),
            requested_at: now(),
            decided_at: Some(now()),
            expires_at: None,
        }
    }

    #[test]
    fn requested_approval_can_be_approved() {
        let approved = requested().approve(actor("operator-a"), ApprovalDecidedAt::new(now()));

        assert_eq!(
            approved.decided_by().expect("decider").as_str(),
            "operator-a"
        );
        assert!(approved.decided_at().is_some());
        assert_eq!(
            approved.reason().as_str(),
            "rollback affects production traffic"
        );
    }

    #[test]
    fn requested_approval_can_be_rejected() {
        let rejected = requested().reject(actor("operator-a"), ApprovalDecidedAt::new(now()));

        assert_eq!(
            rejected.decided_by().expect("decider").as_str(),
            "operator-a"
        );
        assert!(rejected.decided_at().is_some());
    }

    #[test]
    fn row_conversion_accepts_approved_request_with_decision_evidence() {
        let record = ApprovalRecord::try_from(row("approved")).expect("valid approval row");

        assert_eq!(record.status(), ApprovalStatus::Approved);
    }

    #[test]
    fn row_conversion_rejects_approved_request_without_decider() {
        let mut row = row("approved");
        row.decided_by = None;

        let error = ApprovalRecord::try_from(row).expect_err("missing decider must fail");

        assert!(matches!(
            error,
            ApprovalError::MissingDecisionEvidence {
                status: ApprovalStatus::Approved
            }
        ));
    }

    #[test]
    fn row_conversion_rejects_missing_reason() {
        let mut row = row("requested");
        row.reason = None;

        let error = ApprovalRecord::try_from(row).expect_err("missing reason must fail");

        assert_eq!(error, ApprovalError::MissingReason);
    }

    #[test]
    fn row_conversion_rejects_unknown_status() {
        let error = ApprovalRecord::try_from(row("waiting")).expect_err("unknown status must fail");

        assert!(matches!(error, ApprovalError::UnknownStatus { .. }));
    }
}
