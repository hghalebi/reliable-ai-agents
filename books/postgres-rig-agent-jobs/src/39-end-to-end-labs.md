# Appendix I. End-to-End Labs

## How to Use These Labs

These labs turn the book from understanding into engineering practice. Each lab
asks you to change a production control and prove the change with evidence.

Use this shape:

```text
change:
  what you modify

invariant:
  what must remain true after the modification

acceptance evidence:
  tests, SQL, runbook output, event timeline, or review artifact that proves it
```

Do not start with the implementation. Start with the invariant. The code is
correct only when the invariant survives retry, crash, replay, release, and
operator review.

## Lab 1: Add A New Job Kind

Motivation:

```text
real systems rarely run one kind of agent job forever
```

Change:

```text
add a new job kind, payload shape, result shape, and validation path
```

Invariant:

```text
the new job kind must use the same durable admission, lease ownership,
transition rules, retry policy, and event timeline as existing jobs
```

Acceptance evidence:

```text
domain constructor rejects invalid payload
duplicate idempotency key returns one logical job
worker records events for the new job kind
readiness scorecard names the target maturity level
chapter checkpoint can name the prerequisite concept
```

Primary chapters: 3, 4, 5, 10, 11, 30.

## Lab 2: Add A Provider Failure Class

Motivation:

```text
provider behavior changes faster than durable workflow invariants should change
```

Change:

```text
add one new provider failure class and map it to retryable, permanent, or
policy-controlled behavior
```

Invariant:

```text
provider-specific errors must become system decisions before they reach worker
state transitions
```

Acceptance evidence:

```text
adapter fixture captures the provider shape
typed error preserves safe context
worker schedules retry only for retryable failure
permanent failure becomes visible dead state
operator query can explain the reason
```

Primary chapters: 6, 9, 14, 17, 23.

## Lab 3: Add A Runbook Query

Motivation:

```text
operators need questions that map to durable evidence
```

Change:

```text
add a new SQL runbook query for a failure mode not already covered
```

Invariant:

```text
the query must answer an operator question without relying on process-local
memory or scattered logs
```

Acceptance evidence:

```text
query source lives under the SQL directory
test asserts the query covers the intended state or event field
runbook text states the operator question
implementation evidence map points to the query
incident response flow explains when to use it
```

Primary chapters: 15, 21, 23, 24.

## Lab 4: Add A Policy Gate

Motivation:

```text
the model can produce a plausible recommendation that should not be allowed to
act automatically
```

Change:

```text
add a policy rule for one risky recommendation path
```

Invariant:

```text
the model may propose an action, but policy and approval decide whether the
action is allowed
```

Acceptance evidence:

```text
typed result separates proposal from authorization
policy decision records rule version and risk level
approval record stores actor, reason, and proposal snapshot when required
denied action has visible event evidence
side effect cannot run without the required authorization
```

Primary chapters: 12, 16, 20.1, 20.2, 27, 28.

## Lab 5: Add A Behavior Evaluation Gate

Motivation:

```text
availability does not prove that the agent's behavior is acceptable
```

Change:

```text
create a small behavior evaluation for one prompt, model, tool, or policy
change
```

Invariant:

```text
behavior-changing releases should leave behind a reviewable evaluation receipt
```

Acceptance evidence:

```text
dataset version is named
rubric is written before scoring
score and reviewer are recorded
release decision links to the evaluation receipt
post-incident regression case can be added to the same surface
```

Primary chapters: 17, 25, 27, 30.

## Lab 6: Run A Restore And Replay Drill

Motivation:

```text
a backup is only useful if restored work can resume without duplicate side
effects
```

Change:

```text
practice restoring state and deciding which jobs are safe to replay
```

Invariant:

```text
restore is successful only when terminal receipts remain protected and pending
or expired work can be resumed safely
```

Acceptance evidence:

```text
RPO and RTO target are written down
restored state inventory is reviewed
terminal side-effect receipts are not executed again
pending and expired jobs have replay decisions
restore result updates the readiness scorecard
```

Primary chapters: 12, 23, 24, 29, 30.

## Lab 7: Add A Typed Memory Retention Rule

Motivation:

```text
memory becomes dangerous when remembered text can silently influence future
actions after its scope, confidence, or retention policy has changed
```

Change:

```text
add one memory kind or retention rule, then update the typed memory record,
database boundary conversion, retrieval filter, and operator query expectations
```

Invariant:

```text
remembered context may influence a future agent run only when scope, kind,
source, confidence, horizon, and retention policy all allow it
```

Acceptance evidence:

```text
Rust constructor rejects invalid confidence or retention combinations
database row conversion rejects unknown memory kind or horizon
retrieval path returns metadata without exposing raw content by default
agent_memory_by_scope.sql answers which records are eligible for a scope
security review names whether approval is required before using the memory
```

Primary chapters: 4, 27.5, 28, 35, 45.

## Lab 8: Add A Controlled Agent Handoff

Motivation:

```text
multi-agent systems fail when responsibility moves through chat instead of
durable state
```

Change:

```text
add one handoff path from a source agent to a target agent, including reason,
payload, idempotency key, decision state, and target job evidence
```

Invariant:

```text
a handoff transfers responsibility exactly once, never silently grants new
authority, and always leaves enough evidence to reconstruct source and target
work
```

Acceptance evidence:

```text
same-agent handoff is rejected
accepted handoff creates or attaches exactly one target job
rejected handoff stores a decision reason without executing the target job
pending_agent_handoffs.sql shows unresolved responsibility transfers
audit event records source run, target agent, actor, reason, and policy version
```

Primary chapters: 12, 16, 20.1, 20.2, 23, 28.

## Lab 9: Add A Provider Usage Budget Guard

Motivation:

```text
an agent can stay technically available while silently burning tokens, latency,
and money faster than the product can tolerate
```

Change:

```text
add one provider usage or tenant budget rule, then connect it to typed usage
records, admission decisions, runbook evidence, and release review
```

Invariant:

```text
provider usage is accepted as production evidence only when token totals agree,
cost and latency are non-negative, tenant budget state is explicit, and the
admission or release decision can explain the tradeoff
```

Acceptance evidence:

```text
Rust usage type rejects mismatched token totals and negative cost or latency
provider usage row conversion rejects malformed database values
tenant budget decision allows or blocks projected spend deterministically
provider_usage_by_job_kind.sql shows requests, tokens, cost, and p95 latency
release review names whether cost, latency, SLO, and evaluation evidence agree
```

Primary chapters: 21, 22, 23, 25, 35, 45.

## Lab 10: Add A Timeout And Cancellation Policy

Motivation:

```text
long-running work needs a time promise and a stop path, not only a temporary
worker lease
```

Change:

```text
add one timeout policy for a job kind, then connect deadline fields,
cancellation requests, breached-deadline runbooks, and worker behavior
```

Invariant:

```text
lease expiry controls worker ownership, deadline breach controls business
time policy, and cancellation records durable stop intent before active work
is stopped or ignored
```

Acceptance evidence:

```text
Rust timeout policy rejects invalid duration, terminal observed state, and
unknown timeout action
database row conversion rejects malformed deadlines and negative attempts
running_jobs_past_deadline.sql shows overdue work with timeout action
pending_cancellation_requests.sql shows requested, applied, ignored, or expired intent
worker tests prove stale workers cannot complete or retry work after ownership changes
```

Primary chapters: 5, 13, 23, 24, 29, 45.

## Lab 11: Add A Compensation Action

Motivation:

```text
a real side effect cannot always be erased; correction is often another
controlled side effect with its own risk
```

Change:

```text
add one compensation kind for a side effect, including original receipt,
approval evidence, idempotency key, lease ownership, retry policy, and terminal
evidence
```

Invariant:

```text
compensation is never an invisible rollback; it is approved, idempotent,
leased, retried, audited, and observable as its own production action
```

Acceptance evidence:

```text
Rust compensation type rejects non-object payloads and missing approval evidence
approved compensation can be claimed only with a lease and remaining attempts
mark_compensation_succeeded.sql requires the owning worker before completion
mark_compensation_failed.sql records retry or terminal failure evidence
compensation_backlog.sql shows due work, expired leases, and oldest pending action
```

Primary chapters: 12, 16, 23, 24, 29, 45.

## Lab 12: Write A Temporal Adoption Record

Motivation:

```text
a workflow engine should take over only a named execution invariant, not the
whole product truth model
```

Change:

```text
choose one job kind that strains the Postgres worker loop, then write the
Temporal adoption record before adding any Temporal runtime dependency
```

Invariant:

```text
Temporal may own durable workflow execution only when Postgres still owns
product evidence, approvals, side-effect receipts, audit events, and operator
reconciliation
```

Acceptance evidence:

```text
TemporalWorkflowExecutionRef rejects empty workflow references
TemporalActivityReceipts requires at least one activity receipt before reconciliation
the adoption record names the old Postgres owner and new Temporal owner
temporal_workflow_links records the workflow execution reference for one agent run
temporal_activity_receipts records side-effecting activity evidence
temporal_workflow_reconciliation.sql answers whether workflow and product histories agree
approval signals map back to human_approval_requests rows
one runbook question reconciles workflow history with agent runs, tool calls,
operation events, audit events, receipts, and trace id
```

Primary chapters: 13, 16, 20, 23, 30.5, 30.6, 45.

## Lab 13: Write A Kafka Adoption Record

Motivation:

```text
an event stream should distribute committed typed facts, not spread raw worker
JSON faster than operators can understand it
```

Change:

```text
choose one outbox event family that needs fanout or replay, then write the
Kafka adoption record before adding a Kafka runtime dependency
```

Invariant:

```text
Kafka may own event distribution only when Postgres still owns the source
fact and every consumer can prove idempotent replay with a durable receipt
```

Acceptance evidence:

```text
KafkaTopic rejects empty topic names
KafkaOffset rejects negative offsets from broker or database rows
the adoption record names event kind, schema version, partition key, allowed
consumers, redaction rule, and replay rule
KafkaPublishReceipt records the outbox event and topic-partition-offset
ConsumerReceipt proves one consumer processed one event once for one purpose
kafka_publish_receipts records the committed outbox event's stream position
kafka_consumer_receipts records idempotent processing per consumer group
kafka_replay_safety_by_event.sql answers whether replay can be attempted safely
```

Primary chapters: 12, 20, 23, 30.5, 30.7, 45.

## Final Lab Review

After any lab, write a short review:

```text
What invariant did I protect?
Which file, query, or type changed?
Which test or runbook proves the change?
Which operator question became easier to answer?
Which risk remains?
```

If you cannot answer those five questions, the lab is not finished. The goal is
not to make the code compile. The goal is to make the system easier to reason
about during failure.

## Summary

The labs are the bridge from reading to operating. A reliable agent system is
not learned by recognizing terms. It is learned by changing one control at a
time, preserving the invariant, and leaving behind evidence that another
engineer can inspect.

## Further Reading and Sources

- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) supports the appendix's operational review habits and production-readiness vocabulary.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) keeps the appendix grounded in durable state, transactions, logs, and evidence histories.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) supports the appendix's emphasis on explicit boundaries, named types, and maintainable interfaces.
