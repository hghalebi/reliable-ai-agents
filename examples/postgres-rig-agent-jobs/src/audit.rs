//! Typed audit and operation evidence.
//!
//! Audit events answer "who or what made a decision?" Operation events answer
//! "what happened while the system was running?" Both are durable evidence, not
//! process-local log lines.

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AgentRunId, DomainError, JobId};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuditError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("audit or operation event data must be a JSON object")]
    DataMustBeObject,
    #[error("unknown audit actor type: {value}")]
    UnknownActorType { value: UnknownAuditActorType },
    #[error("unknown operation severity: {value}")]
    UnknownSeverity { value: UnknownOperationSeverity },
    #[error("invalid trace id: {value}")]
    InvalidTraceId { value: InvalidTraceId },
    #[error("invalid span id: {value}")]
    InvalidSpanId { value: InvalidSpanId },
    #[error("operation event must reference a job, run, or both")]
    MissingOperationSubject,
    #[error("stored event id cannot be negative: {value}")]
    NegativeEventId { value: i64 },
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_audit_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, AuditError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(AuditError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAuditActorType(String);

impl UnknownAuditActorType {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownAuditActorType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditActorType {
    User,
    Worker,
    Model,
    System,
}

impl AuditActorType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Worker => "worker",
            Self::Model => "model",
            Self::System => "system",
        }
    }
}

impl TryFrom<&str> for AuditActorType {
    type Error = AuditError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "user" => Ok(Self::User),
            "worker" => Ok(Self::Worker),
            "model" => Ok(Self::Model),
            "system" => Ok(Self::System),
            value => Err(AuditError::UnknownActorType {
                value: UnknownAuditActorType::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownOperationSeverity(String);

impl UnknownOperationSeverity {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownOperationSeverity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidTraceId(String);

impl InvalidTraceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for InvalidTraceId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidSpanId(String);

impl InvalidSpanId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for InvalidSpanId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationSeverity {
    Debug,
    Info,
    Warn,
    ErrorLevel,
}

impl OperationSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::ErrorLevel => "error",
        }
    }
}

impl TryFrom<&str> for OperationSeverity {
    type Error = AuditError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::ErrorLevel),
            value => Err(AuditError::UnknownSeverity {
                value: UnknownOperationSeverity::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuditEventId(u64);

impl AuditEventId {
    pub fn try_from_i64(value: i64) -> Result<Self, AuditError> {
        if value < 0 {
            return Err(AuditError::NegativeEventId { value });
        }
        Ok(Self(value as u64))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OperationEventId(u64);

impl OperationEventId {
    pub fn try_from_i64(value: i64) -> Result<Self, AuditError> {
        if value < 0 {
            return Err(AuditError::NegativeEventId { value });
        }
        Ok(Self(value as u64))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuditActorId(String);

impl AuditActorId {
    pub fn new(value: impl Into<String>) -> Result<Self, AuditError> {
        Ok(Self(non_empty_audit_text(value, "audit_actor_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditAction(String);

impl AuditAction {
    pub fn new(value: impl Into<String>) -> Result<Self, AuditError> {
        Ok(Self(non_empty_audit_text(value, "audit_action")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditSubject(String);

impl AuditSubject {
    pub fn new(value: impl Into<String>) -> Result<Self, AuditError> {
        Ok(Self(non_empty_audit_text(value, "audit_subject")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationEventType(String);

impl OperationEventType {
    pub fn new(value: impl Into<String>) -> Result<Self, AuditError> {
        Ok(Self(non_empty_audit_text(value, "operation_event_type")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationMessage(String);

impl OperationMessage {
    pub fn new(value: impl Into<String>) -> Result<Self, AuditError> {
        Ok(Self(non_empty_audit_text(value, "operation_message")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct EvidenceData(Value);

impl EvidenceData {
    pub fn new(value: Value) -> Result<Self, AuditError> {
        if !value.is_object() {
            return Err(AuditError::DataMustBeObject);
        }
        Ok(Self(value))
    }

    pub fn empty() -> Self {
        Self(Value::Object(serde_json::Map::new()))
    }

    pub fn as_json(&self) -> &Value {
        &self.0
    }
}

impl std::fmt::Debug for EvidenceData {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("EvidenceData([redacted])")
    }
}

// ANCHOR: trace_context
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraceId(String);

impl TraceId {
    pub fn new(value: impl Into<String>) -> Result<Self, AuditError> {
        let value = value.into();
        if !is_valid_nonzero_hex_id(&value, 32) {
            return Err(AuditError::InvalidTraceId {
                value: InvalidTraceId::new(value),
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpanId(String);

impl SpanId {
    pub fn new(value: impl Into<String>) -> Result<Self, AuditError> {
        let value = value.into();
        if !is_valid_nonzero_hex_id(&value, 16) {
            return Err(AuditError::InvalidSpanId {
                value: InvalidSpanId::new(value),
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    trace_id: TraceId,
    span_id: Option<SpanId>,
}

impl TraceContext {
    pub fn new(trace_id: TraceId, span_id: Option<SpanId>) -> Self {
        Self { trace_id, span_id }
    }

    pub fn trace_id(&self) -> &TraceId {
        &self.trace_id
    }

    pub fn span_id(&self) -> Option<&SpanId> {
        self.span_id.as_ref()
    }
}

fn is_valid_nonzero_hex_id(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value.as_bytes().iter().all(u8::is_ascii_hexdigit)
        && value.as_bytes().iter().any(|byte| *byte != b'0')
}
// ANCHOR_END: trace_context

// ANCHOR: audit_event_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEventRecord {
    pub id: AuditEventId,
    pub run_id: Option<AgentRunId>,
    pub actor_type: AuditActorType,
    pub actor_id: AuditActorId,
    pub action: AuditAction,
    pub subject: AuditSubject,
    pub event_data: EvidenceData,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DbAuditEventRow {
    pub id: i64,
    pub run_id: Option<Uuid>,
    pub actor_type: String,
    pub actor_id: String,
    pub action: String,
    pub subject: String,
    pub event_data: Value,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<DbAuditEventRow> for AuditEventRecord {
    type Error = AuditError;

    fn try_from(row: DbAuditEventRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: AuditEventId::try_from_i64(row.id)?,
            run_id: row.run_id.map(AgentRunId::from_uuid),
            actor_type: AuditActorType::try_from(row.actor_type.as_str())?,
            actor_id: AuditActorId::new(row.actor_id)?,
            action: AuditAction::new(row.action)?,
            subject: AuditSubject::new(row.subject)?,
            event_data: EvidenceData::new(row.event_data)?,
            created_at: row.created_at,
        })
    }
}
// ANCHOR_END: audit_event_boundary

// ANCHOR: operation_event_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationEventRecord {
    pub id: OperationEventId,
    pub job_id: Option<JobId>,
    pub run_id: Option<AgentRunId>,
    pub trace_context: TraceContext,
    pub event_type: OperationEventType,
    pub severity: OperationSeverity,
    pub message: OperationMessage,
    pub data: EvidenceData,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DbOperationEventRow {
    pub id: i64,
    pub job_id: Option<Uuid>,
    pub run_id: Option<Uuid>,
    pub trace_id: String,
    pub span_id: Option<String>,
    pub event_type: String,
    pub severity: String,
    pub message: String,
    pub data: Value,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<DbOperationEventRow> for OperationEventRecord {
    type Error = AuditError;

    fn try_from(row: DbOperationEventRow) -> Result<Self, Self::Error> {
        if row.job_id.is_none() && row.run_id.is_none() {
            return Err(AuditError::MissingOperationSubject);
        }

        Ok(Self {
            id: OperationEventId::try_from_i64(row.id)?,
            job_id: row.job_id.map(JobId::from_uuid),
            run_id: row.run_id.map(AgentRunId::from_uuid),
            trace_context: TraceContext::new(
                TraceId::new(row.trace_id)?,
                row.span_id.map(SpanId::new).transpose()?,
            ),
            event_type: OperationEventType::new(row.event_type)?,
            severity: OperationSeverity::try_from(row.severity.as_str())?,
            message: OperationMessage::new(row.message)?,
            data: EvidenceData::new(row.data)?,
            created_at: row.created_at,
        })
    }
}
// ANCHOR_END: operation_event_boundary

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn audit_row() -> DbAuditEventRow {
        DbAuditEventRow {
            id: 7,
            run_id: Some(Uuid::new_v4()),
            actor_type: "worker".to_string(),
            actor_id: "worker-a".to_string(),
            action: "approval_requested".to_string(),
            subject: "run:incident-42".to_string(),
            event_data: json!({"policy_version": "approval-policy:v3"}),
            created_at: Utc::now(),
        }
    }

    fn operation_row() -> DbOperationEventRow {
        DbOperationEventRow {
            id: 9,
            job_id: Some(Uuid::new_v4()),
            run_id: Some(Uuid::new_v4()),
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
            span_id: Some("00f067aa0ba902b7".to_string()),
            event_type: "lease_extended".to_string(),
            severity: "info".to_string(),
            message: "worker extended lease".to_string(),
            data: json!({"worker_id": "worker-a"}),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn audit_row_conversion_accepts_valid_event() {
        let record = AuditEventRecord::try_from(audit_row()).expect("valid audit row");

        assert_eq!(record.id.get(), 7);
        assert_eq!(record.actor_type, AuditActorType::Worker);
        assert_eq!(record.action.as_str(), "approval_requested");
    }

    #[test]
    fn audit_row_conversion_rejects_unknown_actor_type() {
        let row = DbAuditEventRow {
            actor_type: "administrator".to_string(),
            ..audit_row()
        };

        let error = AuditEventRecord::try_from(row).expect_err("unknown actor type");

        assert!(matches!(error, AuditError::UnknownActorType { .. }));
    }

    #[test]
    fn audit_row_conversion_rejects_non_object_data() {
        let row = DbAuditEventRow {
            event_data: json!("not an object"),
            ..audit_row()
        };

        let error = AuditEventRecord::try_from(row).expect_err("non-object data");

        assert_eq!(error, AuditError::DataMustBeObject);
    }

    #[test]
    fn audit_row_conversion_rejects_empty_action() {
        let row = DbAuditEventRow {
            action: " ".to_string(),
            ..audit_row()
        };

        let error = AuditEventRecord::try_from(row).expect_err("empty action");

        assert_eq!(
            error,
            AuditError::EmptyText {
                field: "audit_action"
            }
        );
    }

    #[test]
    fn operation_row_conversion_accepts_valid_event() {
        let record = OperationEventRecord::try_from(operation_row()).expect("valid operation row");

        assert_eq!(record.id.get(), 9);
        assert_eq!(record.severity, OperationSeverity::Info);
        assert_eq!(record.event_type.as_str(), "lease_extended");
        assert_eq!(
            record.trace_context.trace_id().as_str(),
            "4bf92f3577b34da6a3ce929d0e0e4736"
        );
        assert_eq!(
            record.trace_context.span_id().expect("span id").as_str(),
            "00f067aa0ba902b7"
        );
    }

    #[test]
    fn operation_row_conversion_rejects_missing_subject() {
        let row = DbOperationEventRow {
            job_id: None,
            run_id: None,
            ..operation_row()
        };

        let error = OperationEventRecord::try_from(row).expect_err("missing subject");

        assert_eq!(error, AuditError::MissingOperationSubject);
    }

    #[test]
    fn operation_row_conversion_rejects_unknown_severity() {
        let row = DbOperationEventRow {
            severity: "critical".to_string(),
            ..operation_row()
        };

        let error = OperationEventRecord::try_from(row).expect_err("unknown severity");

        assert!(matches!(error, AuditError::UnknownSeverity { .. }));
    }

    #[test]
    fn operation_row_conversion_rejects_negative_id() {
        let row = DbOperationEventRow {
            id: -1,
            ..operation_row()
        };

        let error = OperationEventRecord::try_from(row).expect_err("negative id");

        assert_eq!(error, AuditError::NegativeEventId { value: -1 });
    }

    #[test]
    fn operation_row_conversion_rejects_invalid_trace_id() {
        let row = DbOperationEventRow {
            trace_id: "not-a-w3c-trace-id".to_string(),
            ..operation_row()
        };

        let error = OperationEventRecord::try_from(row).expect_err("invalid trace id");

        assert!(matches!(error, AuditError::InvalidTraceId { .. }));
    }

    #[test]
    fn operation_row_conversion_rejects_zero_trace_id() {
        let row = DbOperationEventRow {
            trace_id: "00000000000000000000000000000000".to_string(),
            ..operation_row()
        };

        let error = OperationEventRecord::try_from(row).expect_err("zero trace id");

        assert!(matches!(error, AuditError::InvalidTraceId { .. }));
    }

    #[test]
    fn operation_row_conversion_rejects_invalid_span_id() {
        let row = DbOperationEventRow {
            span_id: Some("bad-span".to_string()),
            ..operation_row()
        };

        let error = OperationEventRecord::try_from(row).expect_err("invalid span id");

        assert!(matches!(error, AuditError::InvalidSpanId { .. }));
    }
}
