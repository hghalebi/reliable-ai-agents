//! Durable human escalation types.
//!
//! Approval asks whether a risky action is authorized. Escalation asks a
//! different question: which human owner must take responsibility because the
//! system cannot safely continue on its own?

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AgentRunId, JobId};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EscalationError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("unknown escalation kind: {value}")]
    UnknownKind { value: UnknownEscalationKind },
    #[error("unknown escalation severity: {value}")]
    UnknownSeverity { value: UnknownEscalationSeverity },
    #[error("unknown escalation status: {value}")]
    UnknownStatus { value: UnknownEscalationStatus },
    #[error("human escalation must reference a job or run")]
    MissingEscalationTarget,
    #[error("{status} escalation requires assigned_to and acknowledged_at")]
    MissingAcknowledgementEvidence { status: EscalationStatus },
    #[error("{status} escalation requires resolved_at")]
    MissingResolutionEvidence { status: EscalationStatus },
    #[error("acknowledged_at cannot be before created_at")]
    AcknowledgedBeforeCreated,
    #[error("resolved_at cannot be before acknowledged_at")]
    ResolvedBeforeAcknowledged,
}

fn non_empty_escalation_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, EscalationError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(EscalationError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownEscalationKind(String);

impl UnknownEscalationKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownEscalationKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscalationKind {
    DeadlineBreach,
    RepeatedFailure,
    SecuritySignal,
    ApprovalTimeout,
    CompatibilityRisk,
    OperatorReview,
}

impl EscalationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeadlineBreach => "deadline_breach",
            Self::RepeatedFailure => "repeated_failure",
            Self::SecuritySignal => "security_signal",
            Self::ApprovalTimeout => "approval_timeout",
            Self::CompatibilityRisk => "compatibility_risk",
            Self::OperatorReview => "operator_review",
        }
    }
}

impl TryFrom<&str> for EscalationKind {
    type Error = EscalationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "deadline_breach" => Ok(Self::DeadlineBreach),
            "repeated_failure" => Ok(Self::RepeatedFailure),
            "security_signal" => Ok(Self::SecuritySignal),
            "approval_timeout" => Ok(Self::ApprovalTimeout),
            "compatibility_risk" => Ok(Self::CompatibilityRisk),
            "operator_review" => Ok(Self::OperatorReview),
            value => Err(EscalationError::UnknownKind {
                value: UnknownEscalationKind::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownEscalationSeverity(String);

impl UnknownEscalationSeverity {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownEscalationSeverity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscalationSeverity {
    Review,
    Ticket,
    Page,
}

impl EscalationSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Review => "review",
            Self::Ticket => "ticket",
            Self::Page => "page",
        }
    }
}

impl TryFrom<&str> for EscalationSeverity {
    type Error = EscalationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "review" => Ok(Self::Review),
            "ticket" => Ok(Self::Ticket),
            "page" => Ok(Self::Page),
            value => Err(EscalationError::UnknownSeverity {
                value: UnknownEscalationSeverity::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownEscalationStatus(String);

impl UnknownEscalationStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownEscalationStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscalationStatus {
    Open,
    Acknowledged,
    Resolved,
    Cancelled,
}

impl EscalationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Acknowledged => "acknowledged",
            Self::Resolved => "resolved",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for EscalationStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for EscalationStatus {
    type Error = EscalationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "open" => Ok(Self::Open),
            "acknowledged" => Ok(Self::Acknowledged),
            "resolved" => Ok(Self::Resolved),
            "cancelled" => Ok(Self::Cancelled),
            value => Err(EscalationError::UnknownStatus {
                value: UnknownEscalationStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HumanEscalationId(Uuid);

impl HumanEscalationId {
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

impl Default for HumanEscalationId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscalationReason(String);

impl EscalationReason {
    pub fn new(value: impl Into<String>) -> Result<Self, EscalationError> {
        Ok(Self(non_empty_escalation_text(value, "escalation_reason")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscalationOwner(String);

impl EscalationOwner {
    pub fn new(value: impl Into<String>) -> Result<Self, EscalationError> {
        Ok(Self(non_empty_escalation_text(value, "escalation_owner")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EscalationCreatedAt(DateTime<Utc>);

impl EscalationCreatedAt {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EscalationAcknowledgedAt(DateTime<Utc>);

impl EscalationAcknowledgedAt {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EscalationResolvedAt(DateTime<Utc>);

impl EscalationResolvedAt {
    pub fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EscalationTarget {
    job_id: Option<JobId>,
    run_id: Option<AgentRunId>,
}

impl EscalationTarget {
    pub fn new(job_id: Option<JobId>, run_id: Option<AgentRunId>) -> Result<Self, EscalationError> {
        if job_id.is_none() && run_id.is_none() {
            return Err(EscalationError::MissingEscalationTarget);
        }

        Ok(Self { job_id, run_id })
    }

    pub fn job_id(self) -> Option<JobId> {
        self.job_id
    }

    pub fn run_id(self) -> Option<AgentRunId> {
        self.run_id
    }
}

// ANCHOR: human_escalation_record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanEscalationRecord {
    id: HumanEscalationId,
    target: EscalationTarget,
    kind: EscalationKind,
    severity: EscalationSeverity,
    status: EscalationStatus,
    reason: EscalationReason,
    assigned_to: Option<EscalationOwner>,
    created_at: EscalationCreatedAt,
    acknowledged_at: Option<EscalationAcknowledgedAt>,
    resolved_at: Option<EscalationResolvedAt>,
}

impl HumanEscalationRecord {
    pub fn id(&self) -> HumanEscalationId {
        self.id
    }

    pub fn target(&self) -> EscalationTarget {
        self.target
    }

    pub fn kind(&self) -> EscalationKind {
        self.kind
    }

    pub fn severity(&self) -> EscalationSeverity {
        self.severity
    }

    pub fn status(&self) -> EscalationStatus {
        self.status
    }

    pub fn reason(&self) -> &EscalationReason {
        &self.reason
    }

    pub fn assigned_to(&self) -> Option<&EscalationOwner> {
        self.assigned_to.as_ref()
    }
}
// ANCHOR_END: human_escalation_record

// ANCHOR: human_escalation_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbHumanEscalationRow {
    pub id: Uuid,
    pub job_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub escalation_kind: String,
    pub severity: String,
    pub status: String,
    pub reason: String,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl TryFrom<DbHumanEscalationRow> for HumanEscalationRecord {
    type Error = EscalationError;

    fn try_from(row: DbHumanEscalationRow) -> Result<Self, Self::Error> {
        let status = EscalationStatus::try_from(row.status.as_str())?;
        let target = EscalationTarget::new(
            row.job_id.map(JobId::from_uuid),
            row.run_id.map(AgentRunId::from_uuid),
        )?;
        let assigned_to = row.assigned_to.map(EscalationOwner::new).transpose()?;
        let acknowledged_at = row.acknowledged_at.map(EscalationAcknowledgedAt::new);
        let resolved_at = row.resolved_at.map(EscalationResolvedAt::new);

        if let Some(acknowledged_at) = acknowledged_at
            && acknowledged_at.as_datetime() < row.created_at
        {
            return Err(EscalationError::AcknowledgedBeforeCreated);
        }

        if matches!(
            status,
            EscalationStatus::Acknowledged
                | EscalationStatus::Resolved
                | EscalationStatus::Cancelled
        ) && (assigned_to.is_none() || acknowledged_at.is_none())
        {
            return Err(EscalationError::MissingAcknowledgementEvidence { status });
        }

        if matches!(
            status,
            EscalationStatus::Resolved | EscalationStatus::Cancelled
        ) && resolved_at.is_none()
        {
            return Err(EscalationError::MissingResolutionEvidence { status });
        }

        if let (Some(acknowledged_at), Some(resolved_at)) = (acknowledged_at, resolved_at)
            && resolved_at.as_datetime() < acknowledged_at.as_datetime()
        {
            return Err(EscalationError::ResolvedBeforeAcknowledged);
        }

        Ok(Self {
            id: HumanEscalationId::from_uuid(row.id),
            target,
            kind: EscalationKind::try_from(row.escalation_kind.as_str())?,
            severity: EscalationSeverity::try_from(row.severity.as_str())?,
            status,
            reason: EscalationReason::new(row.reason)?,
            assigned_to,
            created_at: EscalationCreatedAt::new(row.created_at),
            acknowledged_at,
            resolved_at,
        })
    }
}
// ANCHOR_END: human_escalation_row_boundary

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn row(status: &str) -> DbHumanEscalationRow {
        let created_at = now();
        DbHumanEscalationRow {
            id: Uuid::new_v4(),
            job_id: Some(Uuid::new_v4()),
            run_id: Some(Uuid::new_v4()),
            escalation_kind: "deadline_breach".to_string(),
            severity: "page".to_string(),
            status: status.to_string(),
            reason: "agent run exceeded customer-visible deadline".to_string(),
            assigned_to: None,
            created_at,
            acknowledged_at: None,
            resolved_at: None,
        }
    }

    #[test]
    fn row_conversion_accepts_open_escalation() {
        let row = row("open");

        let escalation = HumanEscalationRecord::try_from(row).expect("valid escalation row");

        assert_eq!(escalation.kind(), EscalationKind::DeadlineBreach);
        assert_eq!(escalation.severity(), EscalationSeverity::Page);
        assert_eq!(escalation.status(), EscalationStatus::Open);
        assert_eq!(
            escalation.reason().as_str(),
            "agent run exceeded customer-visible deadline"
        );
        assert!(escalation.target().job_id().is_some());
        assert!(escalation.target().run_id().is_some());
    }

    #[test]
    fn acknowledged_escalation_requires_owner_and_ack_time() {
        let error =
            HumanEscalationRecord::try_from(row("acknowledged")).expect_err("missing ack evidence");

        assert_eq!(
            error,
            EscalationError::MissingAcknowledgementEvidence {
                status: EscalationStatus::Acknowledged,
            },
        );
    }

    #[test]
    fn resolved_escalation_requires_resolution_time() {
        let mut row = row("resolved");
        row.assigned_to = Some("oncall-primary".to_string());
        row.acknowledged_at = Some(row.created_at + Duration::minutes(1));

        let error = HumanEscalationRecord::try_from(row).expect_err("missing resolution evidence");

        assert_eq!(
            error,
            EscalationError::MissingResolutionEvidence {
                status: EscalationStatus::Resolved,
            },
        );
    }

    #[test]
    fn row_conversion_accepts_resolved_escalation_with_owner_timeline() {
        let mut row = row("resolved");
        row.assigned_to = Some("oncall-primary".to_string());
        row.acknowledged_at = Some(row.created_at + Duration::minutes(1));
        row.resolved_at = Some(row.created_at + Duration::minutes(10));

        let escalation = HumanEscalationRecord::try_from(row).expect("valid resolved escalation");

        assert_eq!(escalation.status(), EscalationStatus::Resolved);
        assert_eq!(
            escalation.assigned_to().map(EscalationOwner::as_str),
            Some("oncall-primary"),
        );
    }

    #[test]
    fn row_conversion_rejects_missing_target() {
        let mut row = row("open");
        row.job_id = None;
        row.run_id = None;

        let error = HumanEscalationRecord::try_from(row).expect_err("missing target");

        assert_eq!(error, EscalationError::MissingEscalationTarget);
    }

    #[test]
    fn row_conversion_rejects_unknown_kind() {
        let mut row = row("open");
        row.escalation_kind = "mystery".to_string();

        let error = HumanEscalationRecord::try_from(row).expect_err("unknown kind");

        assert!(matches!(error, EscalationError::UnknownKind { .. }));
    }

    #[test]
    fn row_conversion_rejects_resolution_before_acknowledgement() {
        let mut row = row("resolved");
        row.assigned_to = Some("oncall-primary".to_string());
        row.acknowledged_at = Some(row.created_at + Duration::minutes(10));
        row.resolved_at = Some(row.created_at + Duration::minutes(1));

        let error = HumanEscalationRecord::try_from(row).expect_err("invalid timeline");

        assert_eq!(error, EscalationError::ResolvedBeforeAcknowledged);
    }
}
