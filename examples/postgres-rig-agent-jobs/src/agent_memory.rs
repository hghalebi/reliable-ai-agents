//! Typed agent-memory records.
//!
//! Memory is production data. This module keeps memory scope, source,
//! confidence, retention, embedding references, and content as explicit domain
//! values after raw database JSON crosses the persistence boundary.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::AgentRunId;

#[derive(Debug, Error, PartialEq)]
pub enum AgentMemoryError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("unknown memory kind: {value}")]
    UnknownKind { value: UnknownMemoryKind },
    #[error("unknown memory source: {value}")]
    UnknownSource { value: UnknownMemorySource },
    #[error("unknown retention policy: {value}")]
    UnknownRetentionPolicy { value: UnknownRetentionPolicy },
    #[error("unknown memory horizon: {value}")]
    UnknownMemoryHorizon { value: UnknownMemoryHorizon },
    #[error("memory horizon {horizon:?} is incompatible with retention policy {retention:?}")]
    HorizonRetentionMismatch {
        horizon: MemoryHorizon,
        retention: RetentionPolicy,
    },
    #[error("memory confidence must be finite")]
    NonFiniteConfidence,
    #[error("memory confidence must be between 0 and 1, got {value}")]
    ConfidenceOutOfRange { value: f64 },
    #[error("memory content must contain a non-empty text field")]
    MissingContentText,
}

fn non_empty_memory_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, AgentMemoryError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(AgentMemoryError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownMemoryKind(String);

impl UnknownMemoryKind {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownMemoryKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownMemorySource(String);

impl UnknownMemorySource {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownMemorySource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownRetentionPolicy(String);

impl UnknownRetentionPolicy {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownRetentionPolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownMemoryHorizon(String);

impl UnknownMemoryHorizon {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownMemoryHorizon {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    UserPreference,
    CaseFact,
    ToolObservation,
    PolicyNote,
}

impl MemoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UserPreference => "user_preference",
            Self::CaseFact => "case_fact",
            Self::ToolObservation => "tool_observation",
            Self::PolicyNote => "policy_note",
        }
    }
}

impl TryFrom<&str> for MemoryKind {
    type Error = AgentMemoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "user_preference" => Ok(Self::UserPreference),
            "case_fact" => Ok(Self::CaseFact),
            "tool_observation" => Ok(Self::ToolObservation),
            "policy_note" => Ok(Self::PolicyNote),
            value => Err(AgentMemoryError::UnknownKind {
                value: UnknownMemoryKind::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryScope(String);

impl MemoryScope {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentMemoryError> {
        Ok(Self(non_empty_memory_text(value, "memory_scope")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemorySource {
    User,
    Tool,
    Model,
    Operator,
    System,
}

impl MemorySource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Tool => "tool",
            Self::Model => "model",
            Self::Operator => "operator",
            Self::System => "system",
        }
    }
}

impl TryFrom<&str> for MemorySource {
    type Error = AgentMemoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "user" => Ok(Self::User),
            "tool" => Ok(Self::Tool),
            "model" => Ok(Self::Model),
            "operator" => Ok(Self::Operator),
            "system" => Ok(Self::System),
            value => Err(AgentMemoryError::UnknownSource {
                value: UnknownMemorySource::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionPolicy {
    Ephemeral,
    Session,
    Durable,
    Audit,
}

impl RetentionPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ephemeral => "ephemeral",
            Self::Session => "session",
            Self::Durable => "durable",
            Self::Audit => "audit",
        }
    }
}

impl TryFrom<&str> for RetentionPolicy {
    type Error = AgentMemoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ephemeral" => Ok(Self::Ephemeral),
            "session" => Ok(Self::Session),
            "durable" => Ok(Self::Durable),
            "audit" => Ok(Self::Audit),
            value => Err(AgentMemoryError::UnknownRetentionPolicy {
                value: UnknownRetentionPolicy::new(value),
            }),
        }
    }
}

// ANCHOR: memory_horizon
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryHorizon {
    ShortTerm,
    LongTerm,
}

impl MemoryHorizon {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShortTerm => "short_term",
            Self::LongTerm => "long_term",
        }
    }

    pub fn validate_retention(self, retention: RetentionPolicy) -> Result<(), AgentMemoryError> {
        let compatible = match self {
            Self::ShortTerm => matches!(
                retention,
                RetentionPolicy::Ephemeral | RetentionPolicy::Session
            ),
            Self::LongTerm => {
                matches!(retention, RetentionPolicy::Durable | RetentionPolicy::Audit)
            }
        };

        if compatible {
            Ok(())
        } else {
            Err(AgentMemoryError::HorizonRetentionMismatch {
                horizon: self,
                retention,
            })
        }
    }
}

impl TryFrom<&str> for MemoryHorizon {
    type Error = AgentMemoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "short_term" => Ok(Self::ShortTerm),
            "long_term" => Ok(Self::LongTerm),
            value => Err(AgentMemoryError::UnknownMemoryHorizon {
                value: UnknownMemoryHorizon::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryLifecyclePolicy {
    horizon: MemoryHorizon,
    retention: RetentionPolicy,
}

impl MemoryLifecyclePolicy {
    pub fn new(
        horizon: MemoryHorizon,
        retention: RetentionPolicy,
    ) -> Result<Self, AgentMemoryError> {
        horizon.validate_retention(retention)?;
        Ok(Self { horizon, retention })
    }

    pub fn horizon(self) -> MemoryHorizon {
        self.horizon
    }

    pub fn retention(self) -> RetentionPolicy {
        self.retention
    }
}
// ANCHOR_END: memory_horizon

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryContent(String);

impl MemoryContent {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentMemoryError> {
        Ok(Self(non_empty_memory_text(value, "memory_content")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for MemoryContent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("MemoryContent([redacted])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MemoryConfidence(f64);

impl MemoryConfidence {
    pub fn new(value: f64) -> Result<Self, AgentMemoryError> {
        if !value.is_finite() {
            return Err(AgentMemoryError::NonFiniteConfidence);
        }

        if !(0.0..=1.0).contains(&value) {
            return Err(AgentMemoryError::ConfidenceOutOfRange { value });
        }

        Ok(Self(value))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryEmbeddingRef(String);

impl MemoryEmbeddingRef {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentMemoryError> {
        Ok(Self(non_empty_memory_text(value, "memory_embedding_ref")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(Uuid);

impl MemoryId {
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

impl Default for MemoryId {
    fn default() -> Self {
        Self::new()
    }
}

// ANCHOR: memory_record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    id: MemoryId,
    run_id: Option<AgentRunId>,
    scope: MemoryScope,
    kind: MemoryKind,
    source: MemorySource,
    content: MemoryContent,
    confidence: MemoryConfidence,
    lifecycle: MemoryLifecyclePolicy,
    embedding_ref: Option<MemoryEmbeddingRef>,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}

impl MemoryRecord {
    pub fn new(
        scope: MemoryScope,
        kind: MemoryKind,
        source: MemorySource,
        content: MemoryContent,
        confidence: MemoryConfidence,
        lifecycle: MemoryLifecyclePolicy,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: MemoryId::new(),
            run_id: None,
            scope,
            kind,
            source,
            content,
            confidence,
            lifecycle,
            embedding_ref: None,
            created_at,
            last_used_at: None,
        }
    }
}
// ANCHOR_END: memory_record

impl MemoryRecord {
    pub fn id(&self) -> MemoryId {
        self.id
    }

    pub fn run_id(&self) -> Option<AgentRunId> {
        self.run_id
    }

    pub fn scope(&self) -> &MemoryScope {
        &self.scope
    }

    pub fn kind(&self) -> MemoryKind {
        self.kind
    }

    pub fn source(&self) -> MemorySource {
        self.source
    }

    pub fn content(&self) -> &MemoryContent {
        &self.content
    }

    pub fn confidence(&self) -> MemoryConfidence {
        self.confidence
    }

    pub fn horizon(&self) -> MemoryHorizon {
        self.lifecycle.horizon()
    }

    pub fn retention(&self) -> RetentionPolicy {
        self.lifecycle.retention()
    }

    pub fn embedding_ref(&self) -> Option<&MemoryEmbeddingRef> {
        self.embedding_ref.as_ref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn last_used_at(&self) -> Option<DateTime<Utc>> {
        self.last_used_at
    }

    pub fn with_run_id(mut self, run_id: AgentRunId) -> Self {
        self.run_id = Some(run_id);
        self
    }

    pub fn with_embedding_ref(mut self, embedding_ref: MemoryEmbeddingRef) -> Self {
        self.embedding_ref = Some(embedding_ref);
        self
    }

    pub fn mark_used(mut self, used_at: DateTime<Utc>) -> Self {
        self.last_used_at = Some(used_at);
        self
    }
}

#[derive(Debug, Deserialize)]
struct DbMemoryContent {
    text: String,
}

impl TryFrom<serde_json::Value> for MemoryContent {
    type Error = AgentMemoryError;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let content: DbMemoryContent =
            serde_json::from_value(value).map_err(|_| AgentMemoryError::MissingContentText)?;
        MemoryContent::new(content.text).map_err(|_| AgentMemoryError::MissingContentText)
    }
}

impl From<&MemoryContent> for serde_json::Value {
    fn from(value: &MemoryContent) -> Self {
        serde_json::json!({ "text": value.as_str() })
    }
}

// ANCHOR: memory_row_boundary
#[derive(Debug, Clone)]
pub struct DbAgentMemoryRecordRow {
    pub id: Uuid,
    pub run_id: Option<Uuid>,
    pub memory_kind: String,
    pub memory_scope: String,
    pub source: String,
    pub confidence: f64,
    pub memory_horizon: String,
    pub retention_policy: String,
    pub content: serde_json::Value,
    pub embedding_ref: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl TryFrom<DbAgentMemoryRecordRow> for MemoryRecord {
    type Error = AgentMemoryError;

    fn try_from(row: DbAgentMemoryRecordRow) -> Result<Self, Self::Error> {
        let horizon = MemoryHorizon::try_from(row.memory_horizon.as_str())?;
        let retention = RetentionPolicy::try_from(row.retention_policy.as_str())?;
        let lifecycle = MemoryLifecyclePolicy::new(horizon, retention)?;

        Ok(Self {
            id: MemoryId::from_uuid(row.id),
            run_id: row.run_id.map(AgentRunId::from_uuid),
            scope: MemoryScope::new(row.memory_scope)?,
            kind: MemoryKind::try_from(row.memory_kind.as_str())?,
            source: MemorySource::try_from(row.source.as_str())?,
            content: MemoryContent::try_from(row.content)?,
            confidence: MemoryConfidence::new(row.confidence)?,
            lifecycle,
            embedding_ref: row.embedding_ref.map(MemoryEmbeddingRef::new).transpose()?,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
        })
    }
}
// ANCHOR_END: memory_row_boundary

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use serde_json::json;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 10, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    fn row() -> DbAgentMemoryRecordRow {
        DbAgentMemoryRecordRow {
            id: Uuid::new_v4(),
            run_id: Some(Uuid::new_v4()),
            memory_kind: "case_fact".to_string(),
            memory_scope: "tenant:acme/case:42".to_string(),
            source: "tool".to_string(),
            confidence: 0.82,
            memory_horizon: "long_term".to_string(),
            retention_policy: "audit".to_string(),
            content: json!({ "text": "Rollback was available at incident start." }),
            embedding_ref: Some("pgvector:agent_memory_records:42".to_string()),
            created_at: now(),
            last_used_at: None,
        }
    }

    #[test]
    fn memory_record_names_scope_source_confidence_and_retention() {
        let record = MemoryRecord::new(
            MemoryScope::new("tenant:acme/user:7").expect("valid scope"),
            MemoryKind::UserPreference,
            MemorySource::User,
            MemoryContent::new("Prefer concise incident summaries").expect("valid content"),
            MemoryConfidence::new(0.9).expect("valid confidence"),
            MemoryLifecyclePolicy::new(MemoryHorizon::ShortTerm, RetentionPolicy::Session)
                .expect("horizon and retention are compatible"),
            now(),
        )
        .with_embedding_ref(MemoryEmbeddingRef::new("embedding:123").expect("valid embedding"));

        assert_eq!(record.scope().as_str(), "tenant:acme/user:7");
        assert_eq!(record.kind(), MemoryKind::UserPreference);
        assert_eq!(record.source(), MemorySource::User);
        assert_eq!(record.confidence().value(), 0.9);
        assert_eq!(record.horizon(), MemoryHorizon::ShortTerm);
        assert_eq!(record.retention(), RetentionPolicy::Session);
        assert_eq!(
            record.embedding_ref().expect("embedding").as_str(),
            "embedding:123"
        );
    }

    #[test]
    fn memory_content_debug_redacts_value() {
        let content = MemoryContent::new("customer secret").expect("valid content");

        assert_eq!(format!("{content:?}"), "MemoryContent([redacted])");
    }

    #[test]
    fn db_row_conversion_accepts_valid_memory_record() {
        let record = MemoryRecord::try_from(row()).expect("valid memory row");

        assert_eq!(record.kind(), MemoryKind::CaseFact);
        assert_eq!(record.source(), MemorySource::Tool);
        assert_eq!(record.horizon(), MemoryHorizon::LongTerm);
        assert_eq!(record.retention(), RetentionPolicy::Audit);
        assert_eq!(record.confidence().value(), 0.82);
        assert!(record.run_id().is_some());
        assert_eq!(
            record.content().as_str(),
            "Rollback was available at incident start."
        );
    }

    #[test]
    fn db_row_conversion_rejects_unknown_kind() {
        let mut row = row();
        row.memory_kind = "loose_note".to_string();

        let error = MemoryRecord::try_from(row).expect_err("unknown kind must fail");

        assert!(matches!(error, AgentMemoryError::UnknownKind { .. }));
    }

    #[test]
    fn db_row_conversion_rejects_invalid_confidence() {
        let mut row = row();
        row.confidence = 1.2;

        let error = MemoryRecord::try_from(row).expect_err("invalid confidence must fail");

        assert!(matches!(
            error,
            AgentMemoryError::ConfidenceOutOfRange { .. }
        ));
    }

    #[test]
    fn db_row_conversion_rejects_unknown_memory_horizon() {
        let mut row = row();
        row.memory_horizon = "forever_context".to_string();

        let error = MemoryRecord::try_from(row).expect_err("unknown horizon must fail");

        assert!(matches!(
            error,
            AgentMemoryError::UnknownMemoryHorizon { .. }
        ));
    }

    #[test]
    fn db_row_conversion_rejects_horizon_retention_mismatch() {
        let mut row = row();
        row.memory_horizon = "short_term".to_string();
        row.retention_policy = "audit".to_string();

        let error = MemoryRecord::try_from(row).expect_err("mismatched horizon must fail");

        assert!(matches!(
            error,
            AgentMemoryError::HorizonRetentionMismatch { .. }
        ));
    }

    #[test]
    fn db_row_conversion_rejects_missing_content_text() {
        let mut row = row();
        row.content = json!({ "note": "text field missing" });

        let error = MemoryRecord::try_from(row).expect_err("missing content text must fail");

        assert_eq!(error, AgentMemoryError::MissingContentText);
    }
}
