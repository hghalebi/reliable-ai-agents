//! Typed transactional outbox primitives.
//!
//! The outbox is the boundary between committing domain state and publishing a
//! side-effect notification. It keeps publication durable, leased, retryable,
//! and inspectable instead of hiding it in process memory.

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{
    AttemptCount, DomainError, FailureMessage, IdempotencyKey, MaxAttempts, WorkerId,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OutboxError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} cannot be negative, got {value}")]
    NegativeNumber { field: &'static str, value: i64 },
    #[error("{field} is too large, got {value}")]
    NumberOutOfRange { field: &'static str, value: i64 },
    #[error("outbox payload must be a JSON object")]
    PayloadMustBeObject,
    #[error("unknown outbox status: {value}")]
    UnknownStatus { value: UnknownOutboxStatus },
    #[error("publishing outbox event requires locked_by and locked_until")]
    MissingPublishingLease,
    #[error("published outbox event requires published_at")]
    MissingPublishedAt,
    #[error("failed outbox event requires last_error")]
    MissingFailureMessage,
    #[error("outbox event has exhausted publication attempts")]
    AttemptsExhausted,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_outbox_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, OutboxError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(OutboxError::EmptyText { field });
    }
    Ok(value)
}

fn non_negative_u32(value: i64, field: &'static str) -> Result<u32, OutboxError> {
    if value < 0 {
        return Err(OutboxError::NegativeNumber { field, value });
    }

    u32::try_from(value).map_err(|_| OutboxError::NumberOutOfRange { field, value })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownOutboxStatus(String);

impl UnknownOutboxStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownOutboxStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboxStatus {
    Pending,
    Publishing,
    Published,
    Failed,
}

impl OutboxStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Publishing => "publishing",
            Self::Published => "published",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for OutboxStatus {
    type Error = OutboxError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending" => Ok(Self::Pending),
            "publishing" => Ok(Self::Publishing),
            "published" => Ok(Self::Published),
            "failed" => Ok(Self::Failed),
            value => Err(OutboxError::UnknownStatus {
                value: UnknownOutboxStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutboxEventId(Uuid);

impl OutboxEventId {
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

impl Default for OutboxEventId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEventKind(String);

impl OutboxEventKind {
    pub fn new(value: impl Into<String>) -> Result<Self, OutboxError> {
        Ok(Self(non_empty_outbox_text(value, "outbox_event_kind")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxAggregateId(String);

impl OutboxAggregateId {
    pub fn new(value: impl Into<String>) -> Result<Self, OutboxError> {
        Ok(Self(non_empty_outbox_text(value, "outbox_aggregate_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxPayload(Value);

impl OutboxPayload {
    pub fn new(value: Value) -> Result<Self, OutboxError> {
        if !value.is_object() {
            return Err(OutboxError::PayloadMustBeObject);
        }

        Ok(Self(value))
    }

    pub fn as_json(&self) -> &Value {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEnvelope {
    id: OutboxEventId,
    kind: OutboxEventKind,
    aggregate_id: OutboxAggregateId,
    idempotency_key: IdempotencyKey,
    payload: OutboxPayload,
    attempts: AttemptCount,
    max_attempts: MaxAttempts,
    occurred_at: DateTime<Utc>,
}

impl OutboxEnvelope {
    pub fn new(
        kind: OutboxEventKind,
        aggregate_id: OutboxAggregateId,
        idempotency_key: IdempotencyKey,
        payload: OutboxPayload,
        max_attempts: MaxAttempts,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: OutboxEventId::new(),
            kind,
            aggregate_id,
            idempotency_key,
            payload,
            attempts: AttemptCount::zero(),
            max_attempts,
            occurred_at,
        }
    }

    fn with_incremented_attempts(mut self) -> Self {
        self.attempts = self.attempts.increment();
        self
    }

    pub fn id(&self) -> OutboxEventId {
        self.id
    }

    pub fn kind(&self) -> &OutboxEventKind {
        &self.kind
    }

    pub fn aggregate_id(&self) -> &OutboxAggregateId {
        &self.aggregate_id
    }

    pub fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    pub fn payload(&self) -> &OutboxPayload {
        &self.payload
    }

    pub fn attempts(&self) -> AttemptCount {
        self.attempts
    }

    pub fn max_attempts(&self) -> MaxAttempts {
        self.max_attempts
    }

    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingOutboxEvent {
    envelope: OutboxEnvelope,
    next_attempt_at: DateTime<Utc>,
}

impl PendingOutboxEvent {
    pub fn new(envelope: OutboxEnvelope, next_attempt_at: DateTime<Utc>) -> Self {
        Self {
            envelope,
            next_attempt_at,
        }
    }

    // ANCHOR: outbox_typestate
    pub fn claim(
        self,
        locked_by: WorkerId,
        locked_until: DateTime<Utc>,
    ) -> Result<PublishingOutboxEvent, OutboxError> {
        if self.envelope.attempts().get() >= self.envelope.max_attempts().get() {
            return Err(OutboxError::AttemptsExhausted);
        }

        Ok(PublishingOutboxEvent {
            envelope: self.envelope.with_incremented_attempts(),
            locked_by,
            locked_until,
        })
    }
    // ANCHOR_END: outbox_typestate

    pub fn envelope(&self) -> &OutboxEnvelope {
        &self.envelope
    }

    pub fn next_attempt_at(&self) -> DateTime<Utc> {
        self.next_attempt_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishingOutboxEvent {
    envelope: OutboxEnvelope,
    locked_by: WorkerId,
    locked_until: DateTime<Utc>,
}

impl PublishingOutboxEvent {
    // ANCHOR: outbox_publish_or_retry
    pub fn mark_published(self, published_at: DateTime<Utc>) -> PublishedOutboxEvent {
        PublishedOutboxEvent {
            envelope: self.envelope,
            published_at,
        }
    }

    pub fn fail_for_retry(
        self,
        next_attempt_at: DateTime<Utc>,
        _last_error: FailureMessage,
    ) -> Result<PendingOutboxEvent, OutboxError> {
        if self.envelope.attempts().get() >= self.envelope.max_attempts().get() {
            return Err(OutboxError::AttemptsExhausted);
        }

        Ok(PendingOutboxEvent::new(self.envelope, next_attempt_at))
    }

    pub fn fail_permanently(self, last_error: FailureMessage) -> FailedOutboxEvent {
        FailedOutboxEvent {
            envelope: self.envelope,
            last_error,
        }
    }
    // ANCHOR_END: outbox_publish_or_retry

    pub fn envelope(&self) -> &OutboxEnvelope {
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
pub struct PublishedOutboxEvent {
    envelope: OutboxEnvelope,
    published_at: DateTime<Utc>,
}

impl PublishedOutboxEvent {
    pub fn envelope(&self) -> &OutboxEnvelope {
        &self.envelope
    }

    pub fn published_at(&self) -> DateTime<Utc> {
        self.published_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedOutboxEvent {
    envelope: OutboxEnvelope,
    last_error: FailureMessage,
}

impl FailedOutboxEvent {
    pub fn envelope(&self) -> &OutboxEnvelope {
        &self.envelope
    }

    pub fn last_error(&self) -> &FailureMessage {
        &self.last_error
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutboxEventRecord {
    Pending(PendingOutboxEvent),
    Publishing(PublishingOutboxEvent),
    Published(PublishedOutboxEvent),
    Failed(FailedOutboxEvent),
}

impl OutboxEventRecord {
    pub fn status(&self) -> OutboxStatus {
        match self {
            Self::Pending(_) => OutboxStatus::Pending,
            Self::Publishing(_) => OutboxStatus::Publishing,
            Self::Published(_) => OutboxStatus::Published,
            Self::Failed(_) => OutboxStatus::Failed,
        }
    }
}

// ANCHOR: outbox_row_boundary
#[derive(Debug, Clone, PartialEq)]
pub struct DbOutboxEventRow {
    pub id: Uuid,
    pub event_kind: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
    pub payload: Value,
    pub status: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub next_attempt_at: DateTime<Utc>,
    pub locked_by: Option<String>,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

impl TryFrom<DbOutboxEventRow> for OutboxEventRecord {
    type Error = OutboxError;

    fn try_from(row: DbOutboxEventRow) -> Result<Self, Self::Error> {
        let status = OutboxStatus::try_from(row.status.as_str())?;
        let envelope = OutboxEnvelope {
            id: OutboxEventId::from_uuid(row.id),
            kind: OutboxEventKind::new(row.event_kind)?,
            aggregate_id: OutboxAggregateId::new(row.aggregate_id)?,
            idempotency_key: IdempotencyKey::new(row.idempotency_key)?,
            payload: OutboxPayload::new(row.payload)?,
            attempts: AttemptCount::try_from_u32(non_negative_u32(row.attempts, "attempts")?)?,
            max_attempts: MaxAttempts::try_from_u32(non_negative_u32(
                row.max_attempts,
                "max_attempts",
            )?)?,
            occurred_at: row.occurred_at,
        };

        match status {
            OutboxStatus::Pending => Ok(Self::Pending(PendingOutboxEvent::new(
                envelope,
                row.next_attempt_at,
            ))),
            OutboxStatus::Publishing => {
                let Some(locked_by) = row.locked_by else {
                    return Err(OutboxError::MissingPublishingLease);
                };
                let Some(locked_until) = row.locked_until else {
                    return Err(OutboxError::MissingPublishingLease);
                };

                Ok(Self::Publishing(PublishingOutboxEvent {
                    envelope,
                    locked_by: WorkerId::new(locked_by)?,
                    locked_until,
                }))
            }
            OutboxStatus::Published => {
                let Some(published_at) = row.published_at else {
                    return Err(OutboxError::MissingPublishedAt);
                };

                Ok(Self::Published(PublishedOutboxEvent {
                    envelope,
                    published_at,
                }))
            }
            OutboxStatus::Failed => {
                let Some(last_error) = row.last_error else {
                    return Err(OutboxError::MissingFailureMessage);
                };

                Ok(Self::Failed(FailedOutboxEvent {
                    envelope,
                    last_error: FailureMessage::new(last_error)?,
                }))
            }
        }
    }
}
// ANCHOR_END: outbox_row_boundary

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use serde_json::json;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    fn envelope(max_attempts: u32) -> OutboxEnvelope {
        OutboxEnvelope::new(
            OutboxEventKind::new("tool_call.executed").expect("valid kind"),
            OutboxAggregateId::new("tool-call-7").expect("valid aggregate id"),
            IdempotencyKey::new("tool-call-7:executed").expect("valid idempotency key"),
            OutboxPayload::new(json!({ "tool_call_id": "tool-call-7" })).expect("valid payload"),
            MaxAttempts::try_from_u32(max_attempts).expect("valid max attempts"),
            now(),
        )
    }

    fn pending_event(max_attempts: u32) -> PendingOutboxEvent {
        PendingOutboxEvent::new(envelope(max_attempts), now())
    }

    fn row(status: &str) -> DbOutboxEventRow {
        DbOutboxEventRow {
            id: Uuid::new_v4(),
            event_kind: "tool_call.executed".to_string(),
            aggregate_id: "tool-call-7".to_string(),
            idempotency_key: "tool-call-7:executed".to_string(),
            payload: json!({ "tool_call_id": "tool-call-7" }),
            status: status.to_string(),
            attempts: 0,
            max_attempts: 5,
            next_attempt_at: now(),
            locked_by: None,
            locked_until: None,
            last_error: None,
            occurred_at: now(),
            published_at: None,
        }
    }

    #[test]
    fn payload_rejects_non_object_json() {
        let error = OutboxPayload::new(json!(["not", "an", "object"])).expect_err("invalid");

        assert_eq!(error, OutboxError::PayloadMustBeObject);
    }

    #[test]
    fn pending_event_can_be_claimed_and_published() {
        let claimed = pending_event(5)
            .claim(
                WorkerId::new("publisher-a").expect("valid worker"),
                now() + Duration::seconds(30),
            )
            .expect("claim should succeed");

        assert_eq!(claimed.envelope().attempts().get(), 1);
        assert_eq!(claimed.locked_by().as_str(), "publisher-a");

        let published = claimed.mark_published(now());

        assert_eq!(published.envelope().attempts().get(), 1);
    }

    #[test]
    fn exhausted_pending_event_cannot_be_claimed() {
        let pending = pending_event(1)
            .claim(
                WorkerId::new("publisher-a").expect("valid worker"),
                now() + Duration::seconds(30),
            )
            .expect("first claim")
            .fail_for_retry(
                now() + Duration::seconds(60),
                FailureMessage::new("broker timeout").expect("valid error"),
            )
            .expect_err("attempts are exhausted");

        assert_eq!(pending, OutboxError::AttemptsExhausted);
    }

    #[test]
    fn publishing_event_can_fail_permanently() {
        let failed = pending_event(5)
            .claim(
                WorkerId::new("publisher-a").expect("valid worker"),
                now() + Duration::seconds(30),
            )
            .expect("claim should succeed")
            .fail_permanently(FailureMessage::new("invalid event shape").expect("valid error"));

        assert_eq!(failed.last_error().as_str(), "invalid event shape");
    }

    #[test]
    fn row_conversion_accepts_pending_row() {
        let record = OutboxEventRecord::try_from(row("pending")).expect("valid row");

        assert_eq!(record.status(), OutboxStatus::Pending);
    }

    #[test]
    fn row_conversion_accepts_publishing_row_with_lease() {
        let mut row = row("publishing");
        row.locked_by = Some("publisher-a".to_string());
        row.locked_until = Some(now() + Duration::seconds(30));

        let record = OutboxEventRecord::try_from(row).expect("valid row");

        assert_eq!(record.status(), OutboxStatus::Publishing);
    }

    #[test]
    fn row_conversion_rejects_publishing_without_lease() {
        let error = OutboxEventRecord::try_from(row("publishing")).expect_err("missing lease");

        assert_eq!(error, OutboxError::MissingPublishingLease);
    }

    #[test]
    fn row_conversion_rejects_published_without_timestamp() {
        let error =
            OutboxEventRecord::try_from(row("published")).expect_err("missing published_at");

        assert_eq!(error, OutboxError::MissingPublishedAt);
    }

    #[test]
    fn row_conversion_rejects_failed_without_error() {
        let error = OutboxEventRecord::try_from(row("failed")).expect_err("missing last_error");

        assert_eq!(error, OutboxError::MissingFailureMessage);
    }

    #[test]
    fn row_conversion_rejects_negative_attempts() {
        let mut row = row("pending");
        row.attempts = -1;

        let error = OutboxEventRecord::try_from(row).expect_err("negative attempts");

        assert_eq!(
            error,
            OutboxError::NegativeNumber {
                field: "attempts",
                value: -1,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_status() {
        let error = OutboxEventRecord::try_from(row("queued")).expect_err("unknown status");

        assert!(matches!(error, OutboxError::UnknownStatus { .. }));
    }
}
