//! Typed agent handoff primitives.
//!
//! A handoff is not a chat message between agents. It is a durable transfer of
//! responsibility from one named agent to another, with a reason, payload,
//! idempotency key, and terminal decision evidence.

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AgentRunId, DomainError, IdempotencyKey, JobId};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HandoffError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("agent handoff cannot target the same agent")]
    SameAgent,
    #[error("handoff payload must be a JSON object")]
    PayloadMustBeObject,
    #[error("unknown handoff status: {value}")]
    UnknownStatus { value: UnknownHandoffStatus },
    #[error("accepted handoff requires target_job_id and decided_at")]
    MissingAcceptanceEvidence,
    #[error("{status} handoff requires decision_reason and decided_at")]
    MissingTerminalDecisionEvidence { status: HandoffStatus },
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_handoff_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, HandoffError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(HandoffError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownHandoffStatus(String);

impl UnknownHandoffStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownHandoffStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandoffStatus {
    Requested,
    Accepted,
    Rejected,
    Expired,
    Cancelled,
}

impl HandoffStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for HandoffStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for HandoffStatus {
    type Error = HandoffError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "requested" => Ok(Self::Requested),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "expired" => Ok(Self::Expired),
            "cancelled" => Ok(Self::Cancelled),
            value => Err(HandoffError::UnknownStatus {
                value: UnknownHandoffStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandoffId(Uuid);

impl HandoffId {
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

impl Default for HandoffId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentName(String);

impl AgentName {
    pub fn new(value: impl Into<String>) -> Result<Self, HandoffError> {
        Ok(Self(non_empty_handoff_text(value, "agent_name")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffReason(String);

impl HandoffReason {
    pub fn new(value: impl Into<String>) -> Result<Self, HandoffError> {
        Ok(Self(non_empty_handoff_text(value, "handoff_reason")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffDecisionReason(String);

impl HandoffDecisionReason {
    pub fn new(value: impl Into<String>) -> Result<Self, HandoffError> {
        Ok(Self(non_empty_handoff_text(
            value,
            "handoff_decision_reason",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffPayload(Value);

impl HandoffPayload {
    pub fn new(value: Value) -> Result<Self, HandoffError> {
        if !value.is_object() {
            return Err(HandoffError::PayloadMustBeObject);
        }
        Ok(Self(value))
    }

    pub fn as_json(&self) -> &Value {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffEnvelope {
    id: HandoffId,
    source_run_id: AgentRunId,
    from_agent: AgentName,
    to_agent: AgentName,
    reason: HandoffReason,
    payload: HandoffPayload,
    idempotency_key: IdempotencyKey,
    requested_at: DateTime<Utc>,
}

impl HandoffEnvelope {
    pub fn new(
        source_run_id: AgentRunId,
        from_agent: AgentName,
        to_agent: AgentName,
        reason: HandoffReason,
        payload: HandoffPayload,
        idempotency_key: IdempotencyKey,
        requested_at: DateTime<Utc>,
    ) -> Result<Self, HandoffError> {
        if from_agent == to_agent {
            return Err(HandoffError::SameAgent);
        }

        Ok(Self {
            id: HandoffId::new(),
            source_run_id,
            from_agent,
            to_agent,
            reason,
            payload,
            idempotency_key,
            requested_at,
        })
    }

    pub fn id(&self) -> HandoffId {
        self.id
    }

    pub fn source_run_id(&self) -> AgentRunId {
        self.source_run_id
    }

    pub fn from_agent(&self) -> &AgentName {
        &self.from_agent
    }

    pub fn to_agent(&self) -> &AgentName {
        &self.to_agent
    }

    pub fn reason(&self) -> &HandoffReason {
        &self.reason
    }

    pub fn payload(&self) -> &HandoffPayload {
        &self.payload
    }

    pub fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestedHandoff {
    envelope: HandoffEnvelope,
}

impl RequestedHandoff {
    pub fn new(envelope: HandoffEnvelope) -> Self {
        Self { envelope }
    }

    // ANCHOR: handoff_typestate
    pub fn accept(self, target_job_id: JobId, decided_at: DateTime<Utc>) -> AcceptedHandoff {
        AcceptedHandoff {
            envelope: self.envelope,
            target_job_id,
            decided_at,
        }
    }

    pub fn reject(
        self,
        reason: HandoffDecisionReason,
        decided_at: DateTime<Utc>,
    ) -> RejectedHandoff {
        RejectedHandoff {
            envelope: self.envelope,
            reason,
            decided_at,
        }
    }
    // ANCHOR_END: handoff_typestate

    pub fn expire(
        self,
        reason: HandoffDecisionReason,
        decided_at: DateTime<Utc>,
    ) -> ExpiredHandoff {
        ExpiredHandoff {
            envelope: self.envelope,
            reason,
            decided_at,
        }
    }

    pub fn cancel(
        self,
        reason: HandoffDecisionReason,
        decided_at: DateTime<Utc>,
    ) -> CancelledHandoff {
        CancelledHandoff {
            envelope: self.envelope,
            reason,
            decided_at,
        }
    }

    pub fn envelope(&self) -> &HandoffEnvelope {
        &self.envelope
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedHandoff {
    envelope: HandoffEnvelope,
    target_job_id: JobId,
    decided_at: DateTime<Utc>,
}

impl AcceptedHandoff {
    pub fn envelope(&self) -> &HandoffEnvelope {
        &self.envelope
    }

    pub fn target_job_id(&self) -> JobId {
        self.target_job_id
    }

    pub fn decided_at(&self) -> DateTime<Utc> {
        self.decided_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedHandoff {
    envelope: HandoffEnvelope,
    reason: HandoffDecisionReason,
    decided_at: DateTime<Utc>,
}

impl RejectedHandoff {
    pub fn envelope(&self) -> &HandoffEnvelope {
        &self.envelope
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpiredHandoff {
    envelope: HandoffEnvelope,
    reason: HandoffDecisionReason,
    decided_at: DateTime<Utc>,
}

impl ExpiredHandoff {
    pub fn envelope(&self) -> &HandoffEnvelope {
        &self.envelope
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelledHandoff {
    envelope: HandoffEnvelope,
    reason: HandoffDecisionReason,
    decided_at: DateTime<Utc>,
}

impl CancelledHandoff {
    pub fn envelope(&self) -> &HandoffEnvelope {
        &self.envelope
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffRecord {
    Requested(RequestedHandoff),
    Accepted(AcceptedHandoff),
    Rejected(RejectedHandoff),
    Expired(ExpiredHandoff),
    Cancelled(CancelledHandoff),
}

impl HandoffRecord {
    pub fn status(&self) -> HandoffStatus {
        match self {
            Self::Requested(_) => HandoffStatus::Requested,
            Self::Accepted(_) => HandoffStatus::Accepted,
            Self::Rejected(_) => HandoffStatus::Rejected,
            Self::Expired(_) => HandoffStatus::Expired,
            Self::Cancelled(_) => HandoffStatus::Cancelled,
        }
    }
}

// ANCHOR: handoff_row_boundary
#[derive(Debug, Clone, PartialEq)]
pub struct DbAgentHandoffRow {
    pub id: Uuid,
    pub source_run_id: Uuid,
    pub from_agent: String,
    pub to_agent: String,
    pub reason: String,
    pub payload: Value,
    pub status: String,
    pub idempotency_key: String,
    pub target_job_id: Option<Uuid>,
    pub decision_reason: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}

impl TryFrom<DbAgentHandoffRow> for HandoffRecord {
    type Error = HandoffError;

    fn try_from(row: DbAgentHandoffRow) -> Result<Self, Self::Error> {
        let status = HandoffStatus::try_from(row.status.as_str())?;
        let from_agent = AgentName::new(row.from_agent)?;
        let to_agent = AgentName::new(row.to_agent)?;
        let envelope = HandoffEnvelope {
            id: HandoffId::from_uuid(row.id),
            source_run_id: AgentRunId::from_uuid(row.source_run_id),
            from_agent: from_agent.clone(),
            to_agent: to_agent.clone(),
            reason: HandoffReason::new(row.reason)?,
            payload: HandoffPayload::new(row.payload)?,
            idempotency_key: IdempotencyKey::new(row.idempotency_key)?,
            requested_at: row.requested_at,
        };

        if from_agent == to_agent {
            return Err(HandoffError::SameAgent);
        }

        match status {
            HandoffStatus::Requested => Ok(Self::Requested(RequestedHandoff { envelope })),
            HandoffStatus::Accepted => {
                let Some(target_job_id) = row.target_job_id else {
                    return Err(HandoffError::MissingAcceptanceEvidence);
                };
                let Some(decided_at) = row.decided_at else {
                    return Err(HandoffError::MissingAcceptanceEvidence);
                };

                Ok(Self::Accepted(AcceptedHandoff {
                    envelope,
                    target_job_id: JobId::from_uuid(target_job_id),
                    decided_at,
                }))
            }
            HandoffStatus::Rejected => {
                let Some(reason) = row.decision_reason else {
                    return Err(HandoffError::MissingTerminalDecisionEvidence { status });
                };
                let Some(decided_at) = row.decided_at else {
                    return Err(HandoffError::MissingTerminalDecisionEvidence { status });
                };
                let reason = HandoffDecisionReason::new(reason)?;

                Ok(Self::Rejected(RejectedHandoff {
                    envelope,
                    reason,
                    decided_at,
                }))
            }
            HandoffStatus::Expired => {
                let Some(reason) = row.decision_reason else {
                    return Err(HandoffError::MissingTerminalDecisionEvidence { status });
                };
                let Some(decided_at) = row.decided_at else {
                    return Err(HandoffError::MissingTerminalDecisionEvidence { status });
                };

                Ok(Self::Expired(ExpiredHandoff {
                    envelope,
                    reason: HandoffDecisionReason::new(reason)?,
                    decided_at,
                }))
            }
            HandoffStatus::Cancelled => {
                let Some(reason) = row.decision_reason else {
                    return Err(HandoffError::MissingTerminalDecisionEvidence { status });
                };
                let Some(decided_at) = row.decided_at else {
                    return Err(HandoffError::MissingTerminalDecisionEvidence { status });
                };

                Ok(Self::Cancelled(CancelledHandoff {
                    envelope,
                    reason: HandoffDecisionReason::new(reason)?,
                    decided_at,
                }))
            }
        }
    }
}
// ANCHOR_END: handoff_row_boundary

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn envelope() -> HandoffEnvelope {
        HandoffEnvelope::new(
            AgentRunId::new(),
            AgentName::new("triage_agent").expect("valid source agent"),
            AgentName::new("refund_agent").expect("valid target agent"),
            HandoffReason::new("refund decision requires billing specialization")
                .expect("valid reason"),
            HandoffPayload::new(json!({ "case_id": "case-42" })).expect("valid payload"),
            IdempotencyKey::new("handoff:case-42:refund").expect("valid idempotency key"),
            now(),
        )
        .expect("valid handoff")
    }

    fn row(status: &str) -> DbAgentHandoffRow {
        DbAgentHandoffRow {
            id: Uuid::new_v4(),
            source_run_id: Uuid::new_v4(),
            from_agent: "triage_agent".to_string(),
            to_agent: "refund_agent".to_string(),
            reason: "refund decision requires billing specialization".to_string(),
            payload: json!({ "case_id": "case-42" }),
            status: status.to_string(),
            idempotency_key: "handoff:case-42:refund".to_string(),
            target_job_id: None,
            decision_reason: None,
            requested_at: now(),
            decided_at: None,
        }
    }

    #[test]
    fn handoff_rejects_same_source_and_target_agent() {
        let error = HandoffEnvelope::new(
            AgentRunId::new(),
            AgentName::new("triage_agent").expect("valid source agent"),
            AgentName::new("triage_agent").expect("valid target agent"),
            HandoffReason::new("self delegation is a loop").expect("valid reason"),
            HandoffPayload::new(json!({ "case_id": "case-42" })).expect("valid payload"),
            IdempotencyKey::new("handoff:self").expect("valid idempotency key"),
            now(),
        )
        .expect_err("self handoff must fail");

        assert_eq!(error, HandoffError::SameAgent);
    }

    #[test]
    fn handoff_payload_rejects_non_object_json() {
        let error = HandoffPayload::new(json!(["case-42"])).expect_err("invalid payload");

        assert_eq!(error, HandoffError::PayloadMustBeObject);
    }

    #[test]
    fn requested_handoff_can_be_accepted_with_target_job_evidence() {
        let accepted = RequestedHandoff::new(envelope()).accept(JobId::new(), now());

        assert_eq!(accepted.envelope().to_agent().as_str(), "refund_agent");
    }

    #[test]
    fn requested_handoff_can_be_rejected_with_reason() {
        let rejected = RequestedHandoff::new(envelope()).reject(
            HandoffDecisionReason::new("target agent lacks tenant permission")
                .expect("valid decision reason"),
            now(),
        );

        assert_eq!(rejected.envelope().from_agent().as_str(), "triage_agent");
    }

    #[test]
    fn row_conversion_accepts_requested_handoff() {
        let record = HandoffRecord::try_from(row("requested")).expect("valid row");

        assert_eq!(record.status(), HandoffStatus::Requested);
    }

    #[test]
    fn row_conversion_accepts_accepted_handoff_with_target_job() {
        let mut row = row("accepted");
        row.target_job_id = Some(Uuid::new_v4());
        row.decided_at = Some(now());

        let record = HandoffRecord::try_from(row).expect("valid row");

        assert_eq!(record.status(), HandoffStatus::Accepted);
    }

    #[test]
    fn row_conversion_rejects_accepted_handoff_without_target_job() {
        let mut row = row("accepted");
        row.decided_at = Some(now());

        let error = HandoffRecord::try_from(row).expect_err("missing target job");

        assert_eq!(error, HandoffError::MissingAcceptanceEvidence);
    }

    #[test]
    fn row_conversion_rejects_rejected_handoff_without_decision_reason() {
        let mut row = row("rejected");
        row.decided_at = Some(now());

        let error = HandoffRecord::try_from(row).expect_err("missing terminal reason");

        assert_eq!(
            error,
            HandoffError::MissingTerminalDecisionEvidence {
                status: HandoffStatus::Rejected,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_same_agent_handoff() {
        let mut row = row("requested");
        row.to_agent = "triage_agent".to_string();

        let error = HandoffRecord::try_from(row).expect_err("self handoff");

        assert_eq!(error, HandoffError::SameAgent);
    }

    #[test]
    fn row_conversion_rejects_unknown_status() {
        let error = HandoffRecord::try_from(row("delegating")).expect_err("unknown status");

        assert!(matches!(error, HandoffError::UnknownStatus { .. }));
    }
}
