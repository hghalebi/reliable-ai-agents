-- psql variable: outbox_event_id
select
  outbox.id as outbox_event_id,
  outbox.event_kind,
  outbox.aggregate_id,
  outbox.status as outbox_status,
  publish.topic,
  publish.partition_id,
  publish.record_offset,
  publish.schema_version,
  count(consumer.id) as consumer_receipt_count,
  count(consumer.id) filter (
    where consumer.status = 'completed'
  ) as completed_consumer_count,
  count(consumer.id) filter (
    where consumer.status = 'rejected_poison_event'
  ) as poison_event_rejection_count,
  count(consumer.id) filter (
    where consumer.status = 'failed_retryable'
  ) as retryable_consumer_failure_count,
  max(consumer.processed_at) as last_consumer_processed_at
from outbox_events outbox
left join kafka_publish_receipts publish
  on publish.outbox_event_id = outbox.id
left join kafka_consumer_receipts consumer
  on consumer.outbox_event_id = outbox.id
where outbox.id = :'outbox_event_id'::uuid
group by
  outbox.id,
  outbox.event_kind,
  outbox.aggregate_id,
  outbox.status,
  publish.topic,
  publish.partition_id,
  publish.record_offset,
  publish.schema_version
order by last_consumer_processed_at desc nulls last;
