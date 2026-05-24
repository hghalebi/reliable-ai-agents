//! Typed boundary for adopting Kafka after the Postgres-first design.
//!
//! This module does not depend on a Kafka client. It models the event,
//! publication, and consumer-receipt evidence that must exist before Kafka is a
//! safe scaling path for reliable agents.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::audit::TraceContext;
use crate::domain::{IdempotencyKey, PayloadSchemaVersion};
use crate::outbox::{OutboxAggregateId, OutboxEventId, OutboxEventKind};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum KafkaAdoptionError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("Kafka offset cannot be negative: {value}")]
    NegativeOffset { value: i64 },
}

fn non_empty_kafka_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, KafkaAdoptionError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(KafkaAdoptionError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KafkaTopic(String);

impl KafkaTopic {
    pub fn new(value: impl Into<String>) -> Result<Self, KafkaAdoptionError> {
        Ok(Self(non_empty_kafka_text(value, "kafka_topic")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KafkaConsumerGroup(String);

impl KafkaConsumerGroup {
    pub fn new(value: impl Into<String>) -> Result<Self, KafkaAdoptionError> {
        Ok(Self(non_empty_kafka_text(value, "kafka_consumer_group")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KafkaConsumerName(String);

impl KafkaConsumerName {
    pub fn new(value: impl Into<String>) -> Result<Self, KafkaAdoptionError> {
        Ok(Self(non_empty_kafka_text(value, "kafka_consumer_name")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KafkaPartition(u32);

impl KafkaPartition {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KafkaOffset(u64);

impl KafkaOffset {
    pub fn try_from_i64(value: i64) -> Result<Self, KafkaAdoptionError> {
        if value < 0 {
            return Err(KafkaAdoptionError::NegativeOffset { value });
        }
        Ok(Self(value as u64))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

// ANCHOR: kafka_outbox_bridge
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KafkaRecordRef {
    topic: KafkaTopic,
    partition: KafkaPartition,
    offset: KafkaOffset,
}

impl KafkaRecordRef {
    pub fn new(topic: KafkaTopic, partition: KafkaPartition, offset: KafkaOffset) -> Self {
        Self {
            topic,
            partition,
            offset,
        }
    }

    pub fn topic(&self) -> &KafkaTopic {
        &self.topic
    }

    pub fn partition(&self) -> KafkaPartition {
        self.partition
    }

    pub fn offset(&self) -> KafkaOffset {
        self.offset
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KafkaPublishReceipt {
    outbox_event_id: OutboxEventId,
    event_kind: OutboxEventKind,
    aggregate_id: OutboxAggregateId,
    schema_version: PayloadSchemaVersion,
    record_ref: KafkaRecordRef,
    trace: TraceContext,
    published_at: DateTime<Utc>,
}

impl KafkaPublishReceipt {
    pub fn new(input: KafkaPublishReceiptInput) -> Self {
        Self {
            outbox_event_id: input.outbox_event_id,
            event_kind: input.event_kind,
            aggregate_id: input.aggregate_id,
            schema_version: input.schema_version,
            record_ref: input.record_ref,
            trace: input.trace,
            published_at: input.published_at,
        }
    }

    pub fn outbox_event_id(&self) -> OutboxEventId {
        self.outbox_event_id
    }

    pub fn record_ref(&self) -> &KafkaRecordRef {
        &self.record_ref
    }

    pub fn trace(&self) -> &TraceContext {
        &self.trace
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KafkaPublishReceiptInput {
    pub outbox_event_id: OutboxEventId,
    pub event_kind: OutboxEventKind,
    pub aggregate_id: OutboxAggregateId,
    pub schema_version: PayloadSchemaVersion,
    pub record_ref: KafkaRecordRef,
    pub trace: TraceContext,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerProcessingStatus {
    Completed,
    RejectedPoisonEvent,
    FailedRetryable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumerReceipt {
    consumer_group: KafkaConsumerGroup,
    consumer_name: KafkaConsumerName,
    outbox_event_id: OutboxEventId,
    record_ref: KafkaRecordRef,
    idempotency_key: IdempotencyKey,
    status: ConsumerProcessingStatus,
    processed_at: DateTime<Utc>,
}

impl ConsumerReceipt {
    pub fn completed_from_publish(
        publish: &KafkaPublishReceipt,
        consumer_group: KafkaConsumerGroup,
        consumer_name: KafkaConsumerName,
        idempotency_key: IdempotencyKey,
        processed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            consumer_group,
            consumer_name,
            outbox_event_id: publish.outbox_event_id(),
            record_ref: publish.record_ref().clone(),
            idempotency_key,
            status: ConsumerProcessingStatus::Completed,
            processed_at,
        }
    }

    pub fn outbox_event_id(&self) -> OutboxEventId {
        self.outbox_event_id
    }

    pub fn record_ref(&self) -> &KafkaRecordRef {
        &self.record_ref
    }

    pub fn status(&self) -> ConsumerProcessingStatus {
        self.status
    }
}
// ANCHOR_END: kafka_outbox_bridge

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{SpanId, TraceId};
    use std::num::NonZeroU32;

    fn fixed_time() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("valid fixed timestamp")
    }

    fn trace() -> Result<TraceContext, Box<dyn std::error::Error>> {
        let trace_id = TraceId::new("4bf92f3577b34da6a3ce929d0e0e4736")?;
        let span_id = SpanId::new("00f067aa0ba902b7")?;
        Ok(TraceContext::new(trace_id, Some(span_id)))
    }

    fn publish_receipt() -> Result<KafkaPublishReceipt, Box<dyn std::error::Error>> {
        Ok(KafkaPublishReceipt::new(KafkaPublishReceiptInput {
            outbox_event_id: OutboxEventId::new(),
            event_kind: OutboxEventKind::new("agent_case_classified")?,
            aggregate_id: OutboxAggregateId::new("case-7")?,
            schema_version: PayloadSchemaVersion::new(NonZeroU32::MIN),
            record_ref: KafkaRecordRef::new(
                KafkaTopic::new("agent.case.events")?,
                KafkaPartition::new(3),
                KafkaOffset::try_from_i64(42)?,
            ),
            trace: trace()?,
            published_at: fixed_time(),
        }))
    }

    #[test]
    fn rejects_empty_topic() {
        assert!(matches!(
            KafkaTopic::new(" "),
            Err(KafkaAdoptionError::EmptyText {
                field: "kafka_topic"
            })
        ));
    }

    #[test]
    fn rejects_negative_offset() {
        assert!(matches!(
            KafkaOffset::try_from_i64(-1),
            Err(KafkaAdoptionError::NegativeOffset { value: -1 })
        ));
    }

    #[test]
    fn consumer_receipt_reuses_published_record_identity() -> Result<(), Box<dyn std::error::Error>>
    {
        let published = publish_receipt()?;
        let receipt = ConsumerReceipt::completed_from_publish(
            &published,
            KafkaConsumerGroup::new("search-indexer")?,
            KafkaConsumerName::new("search-indexer-1")?,
            IdempotencyKey::new("consumer:search-indexer:agent_case_classified:case-7")?,
            fixed_time(),
        );

        assert_eq!(receipt.outbox_event_id(), published.outbox_event_id());
        assert_eq!(receipt.record_ref().topic().as_str(), "agent.case.events");
        assert_eq!(receipt.record_ref().partition().get(), 3);
        assert_eq!(receipt.record_ref().offset().get(), 42);
        assert_eq!(receipt.status(), ConsumerProcessingStatus::Completed);
        Ok(())
    }
}
