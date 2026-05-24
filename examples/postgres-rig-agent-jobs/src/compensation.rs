//! Typed compensation-action primitives.
//!
//! Compensation is not a magic rollback. It is a controlled follow-up side
//! effect with approval evidence, idempotency, leases, retries, and terminal
//! state.

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::approval::ApprovalRequestId;
use crate::domain::{
    AttemptCount, DomainError, FailureMessage, IdempotencyKey, MaxAttempts, WorkerId,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompensationError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} cannot be negative, got {value}")]
    NegativeNumber { field: &'static str, value: i64 },
    #[error("{field} is too large, got {value}")]
    NumberOutOfRange { field: &'static str, value: i64 },
    #[error("compensation payload must be a JSON object")]
    PayloadMustBeObject,
    #[error("unknown compensation status: {value}")]
    UnknownStatus { value: UnknownCompensationStatus },
    #[error("{status} compensation action requires approval_request_id and approved_at")]
    MissingApprovalEvidence { status: CompensationActionStatus },
    #[error("executing compensation action requires locked_by and locked_until")]
    MissingExecutionLease,
    #[error("{status} compensation action requires completed_at")]
    MissingCompletionTime { status: CompensationActionStatus },
    #[error("failed compensation action requires last_error")]
    MissingFailureMessage,
    #[error("compensation action has exhausted attempts")]
    AttemptsExhausted,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_compensation_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, CompensationError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(CompensationError::EmptyText { field });
    }
    Ok(value)
}

fn non_negative_u32(value: i64, field: &'static str) -> Result<u32, CompensationError> {
    if value < 0 {
        return Err(CompensationError::NegativeNumber { field, value });
    }

    u32::try_from(value).map_err(|_| CompensationError::NumberOutOfRange { field, value })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownCompensationStatus(String);

impl UnknownCompensationStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownCompensationStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompensationActionStatus {
    Requested,
    Approved,
    Executing,
    Succeeded,
    Failed,
    Cancelled,
}

impl CompensationActionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Approved => "approved",
            Self::Executing => "executing",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CompensationActionStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for CompensationActionStatus {
    type Error = CompensationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "requested" => Ok(Self::Requested),
            "approved" => Ok(Self::Approved),
            "executing" => Ok(Self::Executing),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            value => Err(CompensationError::UnknownStatus {
                value: UnknownCompensationStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompensationActionId(Uuid);

impl CompensationActionId {
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

impl Default for CompensationActionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SideEffectReceiptId(Uuid);

impl SideEffectReceiptId {
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompensationKind(String);

impl CompensationKind {
    pub fn new(value: impl Into<String>) -> Result<Self, CompensationError> {
        Ok(Self(non_empty_compensation_text(
            value,
            "compensation_kind",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompensationReason(String);

impl CompensationReason {
    pub fn new(value: impl Into<String>) -> Result<Self, CompensationError> {
        Ok(Self(non_empty_compensation_text(
            value,
            "compensation_reason",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompensationPayload(Value);

impl CompensationPayload {
    pub fn new(value: Value) -> Result<Self, CompensationError> {
        if !value.is_object() {
            return Err(CompensationError::PayloadMustBeObject);
        }

        Ok(Self(value))
    }

    pub fn as_json(&self) -> &Value {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompensationEnvelope {
    id: CompensationActionId,
    receipt_id: SideEffectReceiptId,
    kind: CompensationKind,
    reason: CompensationReason,
    payload: CompensationPayload,
    idempotency_key: IdempotencyKey,
    attempts: AttemptCount,
    max_attempts: MaxAttempts,
    requested_at: DateTime<Utc>,
}

impl CompensationEnvelope {
    pub fn new(
        receipt_id: SideEffectReceiptId,
        kind: CompensationKind,
        reason: CompensationReason,
        payload: CompensationPayload,
        idempotency_key: IdempotencyKey,
        max_attempts: MaxAttempts,
        requested_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: CompensationActionId::new(),
            receipt_id,
            kind,
            reason,
            payload,
            idempotency_key,
            attempts: AttemptCount::zero(),
            max_attempts,
            requested_at,
        }
    }

    fn with_incremented_attempts(mut self) -> Self {
        self.attempts = self.attempts.increment();
        self
    }

    pub fn id(&self) -> CompensationActionId {
        self.id
    }

    pub fn receipt_id(&self) -> SideEffectReceiptId {
        self.receipt_id
    }

    pub fn kind(&self) -> &CompensationKind {
        &self.kind
    }

    pub fn reason(&self) -> &CompensationReason {
        &self.reason
    }

    pub fn payload(&self) -> &CompensationPayload {
        &self.payload
    }

    pub fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    pub fn attempts(&self) -> AttemptCount {
        self.attempts
    }

    pub fn max_attempts(&self) -> MaxAttempts {
        self.max_attempts
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestedCompensationAction {
    envelope: CompensationEnvelope,
}

impl RequestedCompensationAction {
    pub fn new(envelope: CompensationEnvelope) -> Self {
        Self { envelope }
    }

    // ANCHOR: compensation_typestate
    pub fn approve(
        self,
        approval_request_id: ApprovalRequestId,
        approved_at: DateTime<Utc>,
        next_attempt_at: DateTime<Utc>,
    ) -> ApprovedCompensationAction {
        ApprovedCompensationAction {
            envelope: self.envelope,
            approval_request_id,
            approved_at,
            next_attempt_at,
        }
    }
    // ANCHOR_END: compensation_typestate

    pub fn cancel(self, completed_at: DateTime<Utc>) -> CancelledCompensationAction {
        CancelledCompensationAction {
            envelope: self.envelope,
            completed_at,
        }
    }

    pub fn envelope(&self) -> &CompensationEnvelope {
        &self.envelope
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedCompensationAction {
    envelope: CompensationEnvelope,
    approval_request_id: ApprovalRequestId,
    approved_at: DateTime<Utc>,
    next_attempt_at: DateTime<Utc>,
}

impl ApprovedCompensationAction {
    // ANCHOR: compensation_execute_or_finish
    pub fn claim(
        self,
        locked_by: WorkerId,
        locked_until: DateTime<Utc>,
    ) -> Result<ExecutingCompensationAction, CompensationError> {
        if self.envelope.attempts().get() >= self.envelope.max_attempts().get() {
            return Err(CompensationError::AttemptsExhausted);
        }

        Ok(ExecutingCompensationAction {
            envelope: self.envelope.with_incremented_attempts(),
            approval_request_id: self.approval_request_id,
            approved_at: self.approved_at,
            locked_by,
            locked_until,
        })
    }
    // ANCHOR_END: compensation_execute_or_finish

    pub fn cancel(self, completed_at: DateTime<Utc>) -> CancelledCompensationAction {
        CancelledCompensationAction {
            envelope: self.envelope,
            completed_at,
        }
    }

    pub fn envelope(&self) -> &CompensationEnvelope {
        &self.envelope
    }

    pub fn approval_request_id(&self) -> ApprovalRequestId {
        self.approval_request_id
    }

    pub fn approved_at(&self) -> DateTime<Utc> {
        self.approved_at
    }

    pub fn next_attempt_at(&self) -> DateTime<Utc> {
        self.next_attempt_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutingCompensationAction {
    envelope: CompensationEnvelope,
    approval_request_id: ApprovalRequestId,
    approved_at: DateTime<Utc>,
    locked_by: WorkerId,
    locked_until: DateTime<Utc>,
}

impl ExecutingCompensationAction {
    pub fn mark_succeeded(self, completed_at: DateTime<Utc>) -> SucceededCompensationAction {
        SucceededCompensationAction {
            envelope: self.envelope,
            approval_request_id: self.approval_request_id,
            approved_at: self.approved_at,
            completed_at,
        }
    }

    pub fn fail_for_retry(
        self,
        next_attempt_at: DateTime<Utc>,
        _last_error: FailureMessage,
    ) -> Result<ApprovedCompensationAction, CompensationError> {
        if self.envelope.attempts().get() >= self.envelope.max_attempts().get() {
            return Err(CompensationError::AttemptsExhausted);
        }

        Ok(ApprovedCompensationAction {
            envelope: self.envelope,
            approval_request_id: self.approval_request_id,
            approved_at: self.approved_at,
            next_attempt_at,
        })
    }

    pub fn fail_permanently(self, last_error: FailureMessage) -> FailedCompensationAction {
        FailedCompensationAction {
            envelope: self.envelope,
            approval_request_id: self.approval_request_id,
            approved_at: self.approved_at,
            last_error,
        }
    }

    pub fn envelope(&self) -> &CompensationEnvelope {
        &self.envelope
    }

    pub fn locked_by(&self) -> &WorkerId {
        &self.locked_by
    }

    pub fn locked_until(&self) -> DateTime<Utc> {
        self.locked_until
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SucceededCompensationAction {
    envelope: CompensationEnvelope,
    approval_request_id: ApprovalRequestId,
    approved_at: DateTime<Utc>,
    completed_at: DateTime<Utc>,
}

impl SucceededCompensationAction {
    pub fn envelope(&self) -> &CompensationEnvelope {
        &self.envelope
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedCompensationAction {
    envelope: CompensationEnvelope,
    approval_request_id: ApprovalRequestId,
    approved_at: DateTime<Utc>,
    last_error: FailureMessage,
}

impl FailedCompensationAction {
    pub fn envelope(&self) -> &CompensationEnvelope {
        &self.envelope
    }

    pub fn last_error(&self) -> &FailureMessage {
        &self.last_error
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelledCompensationAction {
    envelope: CompensationEnvelope,
    completed_at: DateTime<Utc>,
}

impl CancelledCompensationAction {
    pub fn envelope(&self) -> &CompensationEnvelope {
        &self.envelope
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompensationActionRecord {
    Requested(RequestedCompensationAction),
    Approved(ApprovedCompensationAction),
    Executing(ExecutingCompensationAction),
    Succeeded(SucceededCompensationAction),
    Failed(FailedCompensationAction),
    Cancelled(CancelledCompensationAction),
}

impl CompensationActionRecord {
    pub fn status(&self) -> CompensationActionStatus {
        match self {
            Self::Requested(_) => CompensationActionStatus::Requested,
            Self::Approved(_) => CompensationActionStatus::Approved,
            Self::Executing(_) => CompensationActionStatus::Executing,
            Self::Succeeded(_) => CompensationActionStatus::Succeeded,
            Self::Failed(_) => CompensationActionStatus::Failed,
            Self::Cancelled(_) => CompensationActionStatus::Cancelled,
        }
    }
}

// ANCHOR: compensation_row_boundary
#[derive(Debug, Clone, PartialEq)]
pub struct DbCompensationActionRow {
    pub id: Uuid,
    pub receipt_id: Uuid,
    pub compensation_kind: String,
    pub reason: String,
    pub payload: Value,
    pub status: String,
    pub approval_request_id: Option<Uuid>,
    pub idempotency_key: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub next_attempt_at: DateTime<Utc>,
    pub locked_by: Option<String>,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl TryFrom<DbCompensationActionRow> for CompensationActionRecord {
    type Error = CompensationError;

    fn try_from(row: DbCompensationActionRow) -> Result<Self, Self::Error> {
        let status = CompensationActionStatus::try_from(row.status.as_str())?;
        let envelope = CompensationEnvelope {
            id: CompensationActionId::from_uuid(row.id),
            receipt_id: SideEffectReceiptId::from_uuid(row.receipt_id),
            kind: CompensationKind::new(row.compensation_kind)?,
            reason: CompensationReason::new(row.reason)?,
            payload: CompensationPayload::new(row.payload)?,
            idempotency_key: IdempotencyKey::new(row.idempotency_key)?,
            attempts: AttemptCount::try_from_u32(non_negative_u32(row.attempts, "attempts")?)?,
            max_attempts: MaxAttempts::try_from_u32(non_negative_u32(
                row.max_attempts,
                "max_attempts",
            )?)?,
            requested_at: row.requested_at,
        };

        match status {
            CompensationActionStatus::Requested => {
                Ok(Self::Requested(RequestedCompensationAction { envelope }))
            }
            CompensationActionStatus::Approved => {
                let (approval_request_id, approved_at) =
                    approval_evidence(row.approval_request_id, row.approved_at, status)?;
                Ok(Self::Approved(ApprovedCompensationAction {
                    envelope,
                    approval_request_id,
                    approved_at,
                    next_attempt_at: row.next_attempt_at,
                }))
            }
            CompensationActionStatus::Executing => {
                let (approval_request_id, approved_at) =
                    approval_evidence(row.approval_request_id, row.approved_at, status)?;
                let Some(locked_by) = row.locked_by else {
                    return Err(CompensationError::MissingExecutionLease);
                };
                let Some(locked_until) = row.locked_until else {
                    return Err(CompensationError::MissingExecutionLease);
                };

                Ok(Self::Executing(ExecutingCompensationAction {
                    envelope,
                    approval_request_id,
                    approved_at,
                    locked_by: WorkerId::new(locked_by)?,
                    locked_until,
                }))
            }
            CompensationActionStatus::Succeeded => {
                let (approval_request_id, approved_at) =
                    approval_evidence(row.approval_request_id, row.approved_at, status)?;
                let Some(completed_at) = row.completed_at else {
                    return Err(CompensationError::MissingCompletionTime { status });
                };

                Ok(Self::Succeeded(SucceededCompensationAction {
                    envelope,
                    approval_request_id,
                    approved_at,
                    completed_at,
                }))
            }
            CompensationActionStatus::Failed => {
                let (approval_request_id, approved_at) =
                    approval_evidence(row.approval_request_id, row.approved_at, status)?;
                let Some(_completed_at) = row.completed_at else {
                    return Err(CompensationError::MissingCompletionTime { status });
                };
                let Some(last_error) = row.last_error else {
                    return Err(CompensationError::MissingFailureMessage);
                };

                Ok(Self::Failed(FailedCompensationAction {
                    envelope,
                    approval_request_id,
                    approved_at,
                    last_error: FailureMessage::new(last_error)?,
                }))
            }
            CompensationActionStatus::Cancelled => {
                let Some(completed_at) = row.completed_at else {
                    return Err(CompensationError::MissingCompletionTime { status });
                };

                Ok(Self::Cancelled(CancelledCompensationAction {
                    envelope,
                    completed_at,
                }))
            }
        }
    }
}
// ANCHOR_END: compensation_row_boundary

fn approval_evidence(
    approval_request_id: Option<Uuid>,
    approved_at: Option<DateTime<Utc>>,
    status: CompensationActionStatus,
) -> Result<(ApprovalRequestId, DateTime<Utc>), CompensationError> {
    let Some(approval_request_id) = approval_request_id else {
        return Err(CompensationError::MissingApprovalEvidence { status });
    };
    let Some(approved_at) = approved_at else {
        return Err(CompensationError::MissingApprovalEvidence { status });
    };

    Ok((
        ApprovalRequestId::from_uuid(approval_request_id),
        approved_at,
    ))
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use serde_json::json;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn approval_id() -> ApprovalRequestId {
        ApprovalRequestId::from_uuid(Uuid::new_v4())
    }

    fn envelope(max_attempts: u32) -> CompensationEnvelope {
        CompensationEnvelope::new(
            SideEffectReceiptId::from_uuid(Uuid::new_v4()),
            CompensationKind::new("reverse_credit").expect("valid kind"),
            CompensationReason::new("credit was applied to the wrong account").expect("valid"),
            CompensationPayload::new(json!({ "credit_id": "credit-7" })).expect("valid payload"),
            IdempotencyKey::new("credit-7:reverse").expect("valid idempotency key"),
            MaxAttempts::try_from_u32(max_attempts).expect("valid max attempts"),
            now(),
        )
    }

    fn requested(max_attempts: u32) -> RequestedCompensationAction {
        RequestedCompensationAction::new(envelope(max_attempts))
    }

    fn row(status: &str) -> DbCompensationActionRow {
        DbCompensationActionRow {
            id: Uuid::new_v4(),
            receipt_id: Uuid::new_v4(),
            compensation_kind: "reverse_credit".to_string(),
            reason: "credit was applied to the wrong account".to_string(),
            payload: json!({ "credit_id": "credit-7" }),
            status: status.to_string(),
            approval_request_id: None,
            idempotency_key: "credit-7:reverse".to_string(),
            attempts: 0,
            max_attempts: 3,
            next_attempt_at: now(),
            locked_by: None,
            locked_until: None,
            last_error: None,
            requested_at: now(),
            approved_at: None,
            completed_at: None,
        }
    }

    #[test]
    fn payload_rejects_non_object_json() {
        let error =
            CompensationPayload::new(json!(["not", "an", "object"])).expect_err("invalid payload");

        assert_eq!(error, CompensationError::PayloadMustBeObject);
    }

    #[test]
    fn requested_compensation_can_be_approved_claimed_and_succeeded() {
        let executing = requested(3)
            .approve(approval_id(), now(), now())
            .claim(
                WorkerId::new("compensator-a").expect("valid worker"),
                now() + Duration::seconds(30),
            )
            .expect("claim should succeed");

        assert_eq!(executing.envelope().attempts().get(), 1);
        assert_eq!(executing.locked_by().as_str(), "compensator-a");

        let succeeded = executing.mark_succeeded(now());

        assert_eq!(succeeded.envelope().attempts().get(), 1);
    }

    #[test]
    fn executing_compensation_can_return_to_approved_for_retry() {
        let approved = requested(3)
            .approve(approval_id(), now(), now())
            .claim(
                WorkerId::new("compensator-a").expect("valid worker"),
                now() + Duration::seconds(30),
            )
            .expect("claim should succeed")
            .fail_for_retry(
                now() + Duration::seconds(60),
                FailureMessage::new("provider timeout").expect("valid error"),
            )
            .expect("retry should be allowed");

        assert_eq!(approved.envelope().attempts().get(), 1);
    }

    #[test]
    fn exhausted_compensation_attempt_cannot_retry() {
        let error = requested(1)
            .approve(approval_id(), now(), now())
            .claim(
                WorkerId::new("compensator-a").expect("valid worker"),
                now() + Duration::seconds(30),
            )
            .expect("claim should succeed")
            .fail_for_retry(
                now() + Duration::seconds(60),
                FailureMessage::new("provider timeout").expect("valid error"),
            )
            .expect_err("attempts are exhausted");

        assert_eq!(error, CompensationError::AttemptsExhausted);
    }

    #[test]
    fn row_conversion_accepts_requested_row() {
        let record = CompensationActionRecord::try_from(row("requested")).expect("valid row");

        assert_eq!(record.status(), CompensationActionStatus::Requested);
    }

    #[test]
    fn row_conversion_accepts_executing_row_with_approval_and_lease() {
        let mut row = row("executing");
        row.approval_request_id = Some(Uuid::new_v4());
        row.approved_at = Some(now());
        row.locked_by = Some("compensator-a".to_string());
        row.locked_until = Some(now() + Duration::seconds(30));

        let record = CompensationActionRecord::try_from(row).expect("valid row");

        assert_eq!(record.status(), CompensationActionStatus::Executing);
    }

    #[test]
    fn row_conversion_rejects_approved_without_approval_evidence() {
        let error = CompensationActionRecord::try_from(row("approved")).expect_err("invalid row");

        assert!(matches!(
            error,
            CompensationError::MissingApprovalEvidence { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_executing_without_lease() {
        let mut row = row("executing");
        row.approval_request_id = Some(Uuid::new_v4());
        row.approved_at = Some(now());

        let error = CompensationActionRecord::try_from(row).expect_err("invalid row");

        assert_eq!(error, CompensationError::MissingExecutionLease);
    }

    #[test]
    fn row_conversion_rejects_failed_without_error() {
        let mut row = row("failed");
        row.approval_request_id = Some(Uuid::new_v4());
        row.approved_at = Some(now());
        row.completed_at = Some(now());

        let error = CompensationActionRecord::try_from(row).expect_err("invalid row");

        assert_eq!(error, CompensationError::MissingFailureMessage);
    }

    #[test]
    fn row_conversion_rejects_negative_attempts() {
        let mut row = row("requested");
        row.attempts = -1;

        let error = CompensationActionRecord::try_from(row).expect_err("invalid row");

        assert_eq!(
            error,
            CompensationError::NegativeNumber {
                field: "attempts",
                value: -1,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_status() {
        let error = CompensationActionRecord::try_from(row("waiting")).expect_err("invalid row");

        assert!(matches!(error, CompensationError::UnknownStatus { .. }));
    }
}
