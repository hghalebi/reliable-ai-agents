# 30.7 Kafka After Postgres-First

## What You Will Learn

This chapter teaches you to:

This chapter teaches you when Kafka is the right next step after a disciplined
Postgres-first system, and when it is just a very fast way to distribute
confusion.

Kafka is excellent at durable event distribution, fanout, and replay. It is not
automatically a product ledger, an audit log, or a moral upgrade to your
architecture. By the end of the chapter, you should be able to separate product
truth in Postgres from event distribution in Kafka, inspect topic ownership and
partition keys, explain why an event stream is not automatically a product
audit log, and verify that consumer replay cannot duplicate side effects. The
production evidence is a Kafka adoption record that names the strained event
invariant, topic ownership, partitioning key, schema policy, consumer groups,
outbox bridge, replay boundary, rollback path, and Postgres reconciliation
query.

The production evidence is a Kafka adoption record. It names the strained event
invariant, topic ownership, partitioning key, schema policy, consumer groups,
outbox bridge, replay boundary, rollback path, and Postgres reconciliation
query.

## Chapter Thread

Read this chapter as one link in the production chain:

This chapter builds on a system that already has durable jobs, operation events,
audit events, outbox events, and runbooks. That foundation matters. Kafka enters
after the product facts are already named and persisted, not before.

**Builds on:** the system already has durable jobs, operation events, audit
events, outbox events, and runbooks. **Adds:** Kafka as an optional
event-streaming layer for high-volume fanout, replay, and independent consumers.
**Prepares:** systems where many services need the same agent evidence while
Postgres remains the source of product truth.

## Production Failure

A team publishes every agent transition to Kafka and tells downstream services
to trust the stream.

Later, a customer asks why an automated action happened. Postgres has one
state, Kafka has two similar events, a consumer processed an old schema, and no
one can tell which event was the product fact.

- **What breaks:** the event stream became a second product truth without reconciliation rules.
- **False fix:** say "Kafka is the log" while Postgres, audit events, and consumers disagree.
- **Design response:** use Kafka for distribution and replay only after defining typed events, schema compatibility, idempotent consumers, outbox publication, and Postgres reconciliation.

## Motivation

In production, Kafka is useful when many independent systems need durable
streams of events.

Examples include analytics, search indexing, fraud monitoring, billing
projection, operations dashboards, and cross-team event processing. Kafka is
strong when events need durable retention, partitioned scale, consumer groups,
and replay.

The first version of this book does not require Kafka because most teams need
clearer state before they need a streaming platform.

Without a clear product ledger, Kafka can spread ambiguous state faster than
operators can correct it. When Postgres already records agent runs, tool calls, audit events, and outbox
events, Kafka can become the next distribution layer. It should not become a
shortcut around the typed product model.

## Plain Version

Read this as the simple version:

The simple rule is this: add Kafka when event distribution is the bottleneck,
not when the state machine is unclear.

**Simple rule:** add Kafka when event distribution is the bottleneck, not when
the state machine is unclear. **Why it matters:** Kafka can distribute facts to
many consumers, but it can also multiply bad or ambiguous facts with impressive
speed. **What to watch:** event volume, independent consumer count, replay
needs, retention needs, partition-key design, schema evolution, and consumer
idempotency. If those words are still vague in your system, Kafka will not make
them clearer. It will give them better shoes and a louder microphone.

## What You Already Know

Start with these anchors:

- An event is a historical fact, not a wish.
- An outbox event is how Postgres hands facts to another system safely.
- A retry without idempotency creates duplicate work.

Start with the anchors from earlier chapters. An event is a historical fact, not
a wish. An outbox event is how Postgres hands facts to another system safely. A
retry without idempotency creates duplicate work. A trace id lets operators
connect API, worker, model, tool, and event paths. Audit events are business
evidence and may have stricter retention than operational streams.

This chapter adds: Kafka can carry typed facts outward, but the system must
still know where truth begins and how consumers prove safe processing.

## Focus Cue

Keep three things in view:

- **State:** Postgres records product truth; Kafka distributes selected typed events.
- **Move:** an outbox row becomes a Kafka event, and consumers process it idempotently.
- **Proof:** a runbook can match Postgres outbox id, Kafka topic-partition-offset, consumer receipt, operation event, and trace id.

If you get lost, return to state, move, and proof. Then ask whether an event is
product truth, distribution copy, or consumer-derived projection.

## Production Artifact

Build or inspect this artifact before moving on:

Build or inspect a Kafka adoption decision record before moving on. This record
exists because event streaming changes ordering, replay, schema, retention, and
consumer ownership.

**Artifact:** a Kafka adoption decision record. **Why it matters:** event
streaming changes ordering, replay, schema, and consumer ownership. **Done when:** it names topic ownership, partition key, event schema, idempotency key,
outbox bridge, consumer groups, replay policy, retention, redaction, rollback,
and the Postgres reconciliation query. That sounds like a lot because it is.
Kafka is powerful, and powerful machinery deserves labels.

## Implementation Map

Use this map when you move from reading to implementation:

When you move from reading to implementation, the primary surfaces are the
`outbox_events` table in Postgres, the chosen Kafka topic, the consumer group
configuration, the consumer receipt table, and the tracing context.

The core transition is narrow: a committed product event becomes an outbox row,
the publisher writes that row to Kafka, and each consumer records that it has
processed the event idempotently. The evidence path is equally narrow: a
**Primary surface:** the `outbox_events` table in Postgres, the Kafka topic, the
consumer group configuration, the consumer receipt table, and tracing context.
**State transition:** a committed product event becomes an outbox row, a Kafka
record, and then a consumer receipt. **Evidence path:** the Postgres outbox id
maps to a Kafka topic-partition-offset and to consumer receipts. If that chain
breaks, the system may still be moving bytes, but it is no longer proving
product behavior.


## Operator Question

Before you ship this idea, answer one operational question:

Before shipping this architecture, answer one operational question: which
consumers actually need event replay, and what prevents replay from duplicating
real-world side effects?

**Question:** which consumers actually need event replay, and what prevents
replay from duplicating real-world side effects? **Evidence to inspect:** the
outbox row, Kafka topic, partition key, offset, event schema version, consumer
group, consumer receipt, operation event, and trace id. **Escalate if:**
consumers are treating stream replay as inherently safe without durable
idempotency records. Replay is a tool. Without idempotency, it is a photocopier
pointed at your mistakes.


## Runtime Walkthrough

Follow the concept as one runtime pass:

Follow one event through the system. An agent run records a product event in the
same Postgres transaction that changes product state. An outbox publisher reads
the committed row and publishes the typed event to Kafka. After the broker
accepts the record, the publisher writes the topic, partition, offset, and
publish status back to Postgres.

Then each consumer does its own small piece of work. A search index may update a
projection. An analytics service may update aggregate counts. An operations
dashboard may refresh queue metrics. Each consumer records a receipt for the
event and consumer purpose. The receipt is what lets replay remain boring.

**Trigger:** an agent run emits a product event in the product transaction.
**Action:** the publisher writes the typed outbox event to Kafka. **Persistence:**
the publisher records topic, partition, offset, and publish status back to
Postgres. **Check:** each consumer records a durable receipt before claiming
safe processing.


## Acceptance Gate

Do not move on until this minimum evidence exists:

Do not move on until you can trace one product event from its initial Postgres
state change, through the outbox row, to the Kafka offset, and then to the
consumer receipt, projection update, and any required audit evidence.

**Minimum evidence:** one product event can be traced from Postgres state change
to outbox row, Kafka offset, consumer receipt, projection update, and audit
evidence. **Validation path:** publish one event, replay it, restart one
consumer, and prove that no duplicate side effect occurred. **Stop if:** Kafka
publication can happen before the product transaction commits, or if consumer
replay can change the external world twice.


## Micro-Lesson

Use this five-line version before the heavier mechanism:

The short version is this: use Kafka when many independent consumers need
durable event distribution and replay. Postgres records product truth. Kafka
distributes selected typed facts. Consumers record receipts. The proof is that
replaying the stream does not duplicate side effects.

```text
pain: many independent consumers need durable event distribution and replay
rule: Postgres records product truth; Kafka distributes selected typed facts
tiny example: outbox event -> Kafka offset -> consumer receipt
artifact: Kafka adoption record with topic, schema, key, replay, and rollback policy
proof: stream replay does not duplicate side effects
```
If the next section feels large, keep only that chain in view. It is the chapter
in miniature.


## Tiny Scenario

Imagine a support triage agent that has just successfully finished classifying an urgent customer case. In a modern architecture, several downstream systems immediately want to know about this classification: the search index wants to update the searchable case status, the analytics team wants to compute aggregate volume counts, the operations dashboard needs fresh queue metrics, and the fraud monitor is scanning for suspicious pattern signals. 

In a naive architecture, the team might simply instruct the agent to publish a JSON blob directly to a Kafka topic named `agent-events` and hope the downstream consumers figure it out. However, if the agent process crashes immediately after publishing to Kafka but *before* committing the case status to the database, the downstream systems will react to a "fact" that never actually happened in the product ledger. This causes data corruption.

Instead, the Postgres-first system strictly enforces a sequence. It writes one canonical product event directly into the database within the exact same transaction that updates the case status: an `agent_case_classified` event containing the `event_id`, `agent_run_id`, `case_id`, `classification`, `model_version`, `trace_id`, and a timestamp. 

Once that transaction safely commits, an outbox publisher process reads that row and transitions it into a Kafka record. The topic is `agent.case.events`, the partition key is strictly the `case_id` (ensuring ordering for a specific case), and the value is the typed event envelope. 

Crucially, the consumers must still prove they are safe. When the search index or the fraud monitor reads this event, they do not just blindly update their internal state. They must explicitly record a `consumer_receipt` in their own database, noting the `event_id`, the `topic`, the `partition`, the `offset`, and an `idempotency_key`. 

Why is this robust design mandatory? Because if a network partition causes the Kafka broker to resend the event, or if an engineer intentionally rewinds the offset to rebuild a broken projection, the invariant holds: replaying the event does not duplicate a business side effect. The consumer simply checks its receipts, sees the `idempotency_key`, and safely ignores the duplicate message. The event stream acts as a powerful mechanism to help many systems react simultaneously, but it entirely relies on the foundation of typed events, idempotent consumers, and undeniable product evidence.

Read the tiny case as:

```text
setup: an agent completes a case classification and writes a product row
transition: the outbox publisher reads the committed event and writes it to Kafka
evidence: a KafkaOffset publish receipt maps the event ID to a topic-partition-offset
invariant: stream replay or consumer restart cannot duplicate external customer actions
```

## Mental Model

Think of Kafka as a shared event highway.

Cars on the highway are events. Lanes are partitions. Drivers are consumers.
The highway moves traffic well, but it does not decide whether the cargo is
true, safe, or allowed.

In this book's system:

```text
Postgres says:
  this product fact was committed

Kafka says:
  this fact was made available to subscribers

Consumer receipt says:
  this subscriber processed this fact for this purpose
```

All three records matter.

## The Core Problem

Kafka gives scale through partitioned logs and consumer groups. That strength
creates design decisions the Postgres-first system may not have needed yet:

```text
partition key:
  what order must be preserved?

schema version:
  how do old consumers read new events?

consumer group:
  who receives each event?

offset:
  where did this consumer stop?

replay:
  what happens when old events are processed again?
```

For agent systems, these questions are not mechanical. A replayed event might
send a notification, update a CRM, refresh memory, or trigger a human review.
Every consumer that can cause a side effect needs its own idempotency rule.

## The Naive Solution

The naive migration publishes raw state changes:

```text
topic: agent-events
value: whatever JSON the worker had
```

Consumers parse what they can and ignore what they cannot.

This feels fast until production asks:

```text
Which schema version is this?
Can this event be replayed?
Which tenant owns it?
Is the model output validated?
Is the event safe for long retention?
Which consumers are allowed to see it?
What if two events for the same case arrive out of order?
```

Kafka did not create the ambiguity. It amplified it.

## The Production-Grade Concept

Publish typed product events through an outbox.

The product state change remains in Postgres as a state row and operation event.
The outbox row becomes the publish request, but its outbox id, event id, payload
schema version, and publish status remain product evidence. The event type maps
to a topic and schema, while the Rust event enum and compatibility policy still
define what the event means.

Entity identity becomes the partition key. Choose a domain id such as `CaseId`,
`TenantId`, or `AgentRunId`, depending on the ordering the consumers need.
Consumer processing maps to consumer groups and offsets, but product safety
comes from consumer receipts, idempotency keys, and projection state. Replay
uses offset reset or backfill; safety comes from a replay plan, side-effect
guard, and audit note.

The clean bridge is:

```text
product transaction commits state and outbox row
publisher sends outbox row to Kafka
publisher records topic-partition-offset
consumer validates typed event
consumer records receipt before or with side effect
runbook reconciles all records
```

## Typed Rust Boundary Sketch

Kafka adoption invariably fails when events are treated as anonymous, untyped JSON messages. To survive production, an event must possess strict identity, an explicit schema version, logical partitioning, trace context, and verifiable consumer receipts.

The companion crate models this bridge without forcing an immediate dependency on a specific Kafka client. The code is intentionally small and focused: it names the exact evidence that must exist before any Kafka publisher or consumer can be trusted.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/kafka_adoption.rs:kafka_outbox_bridge}}
```

You should read these types as a strict production checklist. The `KafkaPublishReceipt` must contain exactly one outbox event reference, one schema version, one confirmed topic-partition-offset, and one trace id. The `ConsumerReceipt` must document exactly one consumer purpose, one idempotency key, one processed event id, and the final processing status. Finally, the `KafkaRecordRef` must pinpoint the exact place in the stream where the event became available. 

This is the key production move: while Kafka is permitted to distribute the events, the consumers themselves must still bear the burden of recording durable, idempotent processing evidence back into the system of record.

## Postgres Evidence Tables

Kafka adoption needs two kinds of product-side evidence:

A publish receipt proves a committed outbox event reached a specific topic,
partition, and offset. A consumer receipt proves one consumer group processed
that event safely for one purpose.

The companion schema adds both surfaces:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql:kafka_optional_scaling_tables}}
```

Read the schema as a production promise. `kafka_publish_receipts` maps one
`outbox_events` row to one topic-partition-offset. The uniqueness constraint on
topic, partition, and offset prevents two product receipts from claiming the
same stream position. `kafka_consumer_receipts` records one idempotent
processing result per consumer group and event. The `idempotency_key` prevents a
replay from becoming a duplicate business action. Failed retryable consumers
must record an error, so replay work is visible instead of becoming folklore.

Kafka stores the distributed log. Postgres stores the product evidence that
the distributed log was published and consumed safely.

## Replay-Safety Query

The first Kafka runbook query should answer a narrow question:

```text
Can this event be replayed without duplicating side effects?
```

The checked query connects the source outbox row, publish receipt, and consumer
receipts:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/kafka_replay_safety_by_event.sql}}
```

Use it before replay, after a consumer incident, and during rollback.

If this query cannot show the publish receipt and consumer receipts, Kafka may
have delivered data, but the product system cannot yet prove replay safety.

## Formal Definition

For this chapter, the precise definition is:

```text
Kafka adoption is evidence-preserving migration of event distribution responsibility from direct Postgres polling to a partitioned event log while keeping product truth, event schemas, consumer idempotency, and audit evidence explicit.
```

In the book's system model, **State** is split deliberately: Postgres owns
product state and outbox state, Kafka owns distributed event availability, and
consumers own their projection and receipt state. **Actor** means the outbox
publisher writes events, Kafka stores them by topic and partition, and consumer
groups process them independently.

**Transition** begins when a committed product event becomes a published stream
event. It continues when one or more consumers process that event idempotently.
**Evidence** is the correlation between outbox id, event id, topic, partition,
offset, schema version, consumer group, consumer receipt, operation event, and
trace id. **Invariant** is blunt: Kafka replay or duplicate delivery must not
create duplicate business side effects or untraceable state changes.

## What Can Fail

**Design smell:** Kafka is added before event ownership, schema, partition key,
and consumer idempotency are defined. **Production symptom:** replayed events
update a projection twice, trigger duplicate external actions, or expose data to
a consumer that should not receive it.

**Corrective invariant:** every published event has typed schema, stable
identity, allowed audience, replay rule, and consumer receipt. **Evidence to inspect:** the outbox row, event envelope, schema version, topic-partition-offset,
consumer group lag, consumer receipt, trace id, and reconciliation query.

## Production Contract

Kafka may own distribution mechanics:

```text
topic retention
partitioned ordering
consumer group fanout
offset tracking
stream replay
high-volume event delivery
```

Postgres still owns product truth at the source:

```text
agent run lifecycle
tool-call state
approval decision
side-effect receipt
audit event
outbox event identity
schema version
```

Consumers own their derived state:

```text
projection row
consumer receipt
last processed offset
idempotency record
side-effect receipt when the consumer acts externally
```

The contract is: publish facts, not guesses; consume idempotently, not
optimistically.

## Minimum Serious Kafka Adoption

Do not begin by putting every internal transition on a topic.

Pick one event family where Postgres polling is already the strained part of
the system. Good candidates have several of these pressures at once:

```text
many independent consumers
clear replay requirement
high outbox polling load
cross-team event ownership
large retention need
partitioned ordering by one domain id
projection rebuilds after incidents
```

Then make the adoption small enough to prove.

The minimum serious shape is:

The product transaction writes the state change and `outbox_events` row
together. Every event carries `OutboxEventId`, event kind, aggregate id, schema
version, idempotency key, tenant scope, and trace id. One team owns the topic
name, schema policy, retention rule, and allowed consumers.

The partition key is a typed domain id chosen for the ordering the consumer
actually needs. The publisher records topic, partition, offset, schema version,
publish time, and trace id. Each consumer records event id, consumer group,
consumer name, topic, partition, offset, idempotency key, status, and processed
time.

The replay rule states which consumers can replay safely and which side effects
are blocked or guarded. The reconciliation query compares source state, outbox
status, Kafka offset, consumer receipts, projection state, operation events, and
traces.

This is the smallest useful rule:

```text
Kafka may distribute only committed, typed facts, and every consumer must prove replay safety.
```

That rule keeps Kafka useful without turning the stream into an ungoverned
second product database.

## Real Implementation Shape

A real Kafka version would usually split into four pieces:

```text
publisher module:
  claim pending outbox rows, publish with typed key, record topic-partition-offset

event module:
  typed event envelope, schema version, allowed audience, redaction policy

consumer module:
  parse envelope, validate schema, insert consumer receipt, update projection or act

replay module:
  choose offset range, check replay rule, disable unsafe side effects, record proof
```

The companion module `kafka_adoption.rs` is intentionally not a Kafka client
implementation. It is the typed evidence model you should design before adding
the runtime dependency. If the event id, schema, partition key, and consumer
receipt are vague, a real broker will only spread the vagueness faster.

Kafka's own design documentation is a useful warning here: consumers control
their position in a partition log and can re-consume records. That is a power,
not a safety guarantee. Your application still needs idempotent consumers and
durable receipts for any consumer that changes the world.

## When Not To Use Kafka Yet

Do not add Kafka only because events sound mature.

Use the Postgres-first outbox for longer when the pain is still one of these:

| Pressure | Better next step |
| --- | --- |
| Event names are unclear. | Define typed event enums, schema versions, and event ownership first. |
| Consumers do not record receipts. | Add `consumer_receipts` and idempotency keys before introducing replay. |
| The product fact is ambiguous. | Fix the source transaction and audit event before publishing it widely. |
| One dashboard is stale. | Improve the outbox publisher, indexes, or polling interval before adding a stream platform. |
| Teams want access to all events. | Define tenant boundaries, redaction rules, allowed audiences, and retention policy first. |

Kafka is a good answer when event distribution, retention, replay, and
independent consumers are the strained invariant. It is not a good answer when
the system is still publishing raw uncertainty.

The stop rule is simple:

```text
If an event is not safe to replay,
it is not ready to become a long-retention Kafka record.
```

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Publish raw worker JSON to one broad topic and let consumers improvise. | Fast fanout, but schema, authorization, replay, ordering, idempotency, and evidence are undefined. |
| Safer version | Publish typed outbox events after product transaction commit. | Postgres stays the product source, Rust event types carry meaning, Kafka distributes selected facts, and consumers record receipts. |
| Production version | Add schema compatibility policy, partition-key review, consumer idempotency tests, lag SLOs, replay drills, redaction rules, trace propagation, and reconciliation runbooks. | Kafka becomes a reliable distribution layer without becoming an ungoverned second source of truth. |

## Testing Strategy

Unit tests should prove each event type has a stable name, schema version,
partition key, and allowed audience. Rust event constructors should reject
missing ids, unknown event kinds, and unvalidated model-derived fields.

Persistence tests should prove the outbox row is committed with the product
state change and that publication status stores topic, partition, offset, and
publish time without losing the original event id.

Regression tests should replay the same event twice and prove the consumer
does not duplicate a side effect. Another regression test should feed an old
schema version to a new consumer and prove the compatibility rule is explicit.

Postgres tests protect source truth and consumer receipts. Rust tests protect
event typing and consumer behavior. Kafka integration tests, when introduced,
should use a local broker or test container and realistic event fixtures.

## Observability Strategy

Propagate trace id from the original agent run into the outbox event, Kafka
headers or event envelope, consumer processing span, projection update, and any
side-effect receipt.

Use structured `tracing` fields for event id, event type, schema version,
topic, partition, offset, consumer group, consumer name, job id, run id, tenant,
and idempotency key.

Record an operation event for each important bridge:

```text
outbox_event_ready
kafka_event_published
kafka_consumer_started_event
kafka_consumer_completed_event
kafka_consumer_replayed_event
```

Every runbook query should join source state, outbox status, published offset,
consumer lag, consumer receipts, and trace id. A dashboard may show lag, but a
runbook query should prove which business event is delayed.

## Security and Safety Considerations

Treat event payloads, Kafka headers, consumer inputs, model-derived fields, and
replayed records as untrusted until parsed into typed domain events. Do not let
raw model output become a long-retention stream record without validation,
redaction, and policy review.

The authorization rule applies at publication and consumption. Some consumers may be
allowed to receive aggregate operational events but not tenant-sensitive tool
payloads. The sandboxing and approval checks still apply before consumers perform side
effects.

Redact secrets and sensitive payloads from event values, headers, logs, traces,
dead-letter topics, and replay files. Long retention is useful only when the
event is safe to retain.

## Operational Checklist

Before declaring the Kafka migration complete, operators must perform a strict review of the system's boundaries and failure modes. 

First, verify the **State** boundary: ensure everyone agrees exactly which facts remain permanently in Postgres, which events are transiently distributed via Kafka, and which resulting projections belong exclusively to the consumers. Second, inspect the **Boundary** transitions themselves: verify that outbox rows, event envelopes, network headers, and consumer inputs are strictly converted into typed domain events before any processing occurs. 

Third, rehearse your **Failure** modes: document exactly what happens when the system encounters a duplicate publish attempt, a broker outage, a sudden consumer restart, an intentional offset reset, a stream replay, a poisoned event payload, or a schema mismatch. Fourth, validate your **Observability** pipeline: confirm that a single trace id can seamlessly connect the original product state, the outbox row, the Kafka offset, the consumer receipt, the updated projection, and the final operation event. Finally, verify **Safety**: ensure that all previous guarantees regarding authorization, sandboxing, human approval gates, data redaction, data retention, idempotency, and side-effect receipts are strictly preserved for every new consumer attached to the stream.

## Exercises

1. Pick one product event from the companion system and decide whether it
   belongs in Kafka. Name its event id, schema version, partition key, and
   allowed consumers.
2. Design a `ConsumerReceipt` domain type. Explain why `consumer_name`,
   `event_id`, `topic`, `partition`, and `offset` should not be loose strings
   and integers across the architecture. Add a Rust negative test for an
   invalid receipt.
3. Write a replay drill for one consumer. The drill should process the same
   event twice and prove that no external side effect happens twice. Include
   the Postgres receipt row and idempotency key that make the proof durable.

## Self-Check

Before you move on, use this quick retrieval drill to solidify your understanding. First, recall exactly what Kafka owns in this architecture versus what Postgres still permanently owns. Next, be able to clearly explain why a data stream is not automatically equivalent to a compliant audit log. Then, apply this knowledge to your own systems by deciding whether a specific agent event should be a Postgres-only event, a Kafka event, or both. Finally, ensure you can explicitly name the outbox id, event id, topic, partition, offset, consumer receipt, operation event, and trace id that correlate to one single product event.
- Recall: what is the core invariant in this chapter?
- Explain: why does the invariant matter during an incident?
- Apply: use the idea on one real agent job or tool call.
- Evidence: name the artifact that proves the result.


## Summary

Kafka is a powerful, highly scalable event-streaming platform. In reliable agent systems, however, its best first role is strictly distribution: sending selected, typed facts to independent consumers that genuinely require fanout, replay capabilities, and scale. 

The core invariant to remember is that Kafka replay and duplicate delivery must never create duplicate external side effects or untraceable product state changes. To enforce this, your architecture must rely on outbox rows, topic-partition-offsets, strict schema versions, verifiable consumer receipts, operation events, audit events, traces, and documented runbooks to prove safe distribution. 

Moving forward, remember the golden rule: publish only typed facts through an outbox, and never publish raw uncertainty directly into a long-retention stream.

**Invariant:** Kafka replay and duplicate delivery must not create duplicate side effects or untraceable product state.

**Evidence:** outbox rows, topic-partition-offsets, consumer receipts, operation events, audit events, traces, and runbooks prove safe distribution.

## Changed Understanding

Before reading this chapter, Kafka may have looked like simply the next logical queue to adopt after Postgres. After this chapter, you should understand that Kafka is an optional, advanced event-streaming layer meant exclusively for distribution, replay, and fanout, and only *after* product truth is already explicitly modeled in the database. Moving forward, keep in mind that event streaming will multiply whatever you feed it—both good facts and bad ambiguity—so you must strictly publish only typed, validated, and governed events.
- **Before this chapter:** the mechanism may have looked like an implementation detail.
- **After this chapter:** the mechanism is a production contract with evidence.
- **Keep:** name the invariant, evidence, and operator question before relying on it.

## Further Reading and Sources

- [Apache Kafka Introduction](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: it defines Kafka's core event-streaming concepts before the chapter uses topics, producers, consumers, partitions, and retention.
- [Apache Kafka Documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: it is the primary project documentation for Kafka APIs, configuration, operations, and security.
- [Apache Kafka Protocol](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: offsets, partitions, consumer groups, and transactions become evidence fields after Kafka joins the system.
- [Apache Kafka Design](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: it explains partition-level ordering, consumer positions, rewind, replication, and transaction mechanics.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: Kafka adoption is a log, ordering, replay, and consistency design decision.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: event-streaming infrastructure still needs ownership, SLOs, runbooks, incident response, and toil controls.
