# Appendix H. System Diagrams And Timelines

## How to Use These Diagrams

This appendix gives the book's main mechanisms as small systems diagrams. Use it
when the prose is clear but the moving parts still feel separate.

Each diagram is deliberately plain text. The goal is not decoration. The goal is
to make the production invariant visible:

```text
component -> state -> transition -> evidence -> operator question
```

If you cannot redraw a diagram from memory, return to the chapter named below
the diagram and rebuild the idea from the tiny example.

## The Whole System

```text
API / scheduler / webhook
  creates one logical request
          |
          v
Postgres agent_jobs
  stores durable state, payload, versions, lease, attempts
          |
          v
Rust worker
  owns one leased transition at a time
          |
          v
Rig boundary
  performs one model-backed reasoning step
          |
          v
Policy and approval
  decide whether the recommendation may become action
          |
          v
Side-effect boundary
  records idempotency key and external receipt
          |
          v
Postgres agent_job_events
  explains what happened for operators, tests, and incidents
```

Core invariant:

```text
the model never owns the workflow; durable state owns the workflow
```

Primary chapters: 1, 2, 3, 5, 6, 12, 16, 20.

## Job State Machine

```text
pending
  | pick due job with lease
  v
running
  | model succeeds and result is valid
  v
succeeded

running
  | transient failure and attempts remain
  v
pending
  | run_at moves into the future
  v
running

running
  | permanent failure or attempts exhausted
  v
dead

pending or running
  | cancellation request is applied before terminal state
  v
cancelled

running
  | worker stops heartbeating
  v
pending
  | recovery releases expired lease
```

Core invariant:

```text
only valid transitions are expressible, and terminal states stay terminal
```

Primary chapters: 2.5, 4, 5, 11, 13, 14.

## Cancellation Request Timeline

```text
t0
  user, operator, system, or timeout policy requests cancellation
  cancellation_requests.status = requested

t1
  worker or control plane observes the request
  target job is still pending or running

t2
  job moves to cancelled
  cancellation_requests.status = applied

alternative t2
  target job is already succeeded, dead, or cancelled
  cancellation_requests.status = ignored_terminal

alternative t2
  request expires before application
  cancellation_requests.status = expired
```

Core invariant:

```text
stop intent is durable before work is stopped, ignored, or expired
```

Primary chapters: 13, 18, 23, 24.

## Lease And Heartbeat Timeline

```text
t0
  worker-a claims job-1
  locked_by = worker-a
  locked_until = t0 + lease

t1
  worker-a heartbeats
  locked_until = t1 + lease

t2
  worker-a crashes
  no more heartbeats

t3
  locked_until is in the past
  recovery query makes job-1 pending again

t4
  worker-b claims job-1
  worker-b records the next event
```

Core invariant:

```text
running work has an owner, but ownership expires when the owner disappears
```

Primary chapters: 3, 5, 13, 18, 23.

## Deadline And Timeout Timeline

```text
t0
  run starts
  deadline_at = t0 + timeout policy duration

t1
  worker heartbeats successfully
  lease remains valid

t2
  deadline_at is in the past
  job may still be leased by the worker

t3
  runbook query detects overdue work
  timeout policy chooses retry, cancel, escalation, or dead-letter
```

Core invariant:

```text
lease ownership and deadline policy are separate facts
```

Primary chapters: 13, 14, 15, 21, 23.

## Retry And Dead-Letter Timeline

```text
attempt 1
  provider timeout
  retryable
  run_at = now + 30s

attempt 2
  provider timeout
  retryable
  run_at = now + 60s

attempt 3
  provider timeout
  retryable
  run_at = now + 120s

attempt N
  attempts exhausted
  dead
  reason and event timeline remain inspectable
```

Permanent failures take a shorter path:

```text
missing API key -> permanent -> dead -> operator fixes configuration
```

Core invariant:

```text
retry is a typed scheduling decision, not a hidden loop
```

Primary chapters: 6, 9, 14, 17, 24.

## Approval And Side-Effect Path

```text
model output
  proposes action
      |
      v
typed result
  validates shape and meaning
      |
      v
policy decision
  allow, deny, or require human approval
      |
      v
approval record
  stores actor, reason, policy version, and proposal snapshot
      |
      v
side-effect execution
  uses idempotency key and stores external receipt

escalation path
  records target, kind, severity, owner, acknowledgement, and resolution
```

Core invariant:

```text
the model may propose risk, but policy, approval, and escalation own risk
```

Primary chapters: 12, 16, 20.1, 20.2, 27, 28.

## Observability Evidence Flow

```text
job row
  current truth
      |
      v
job events
  historical explanation
      |
      v
metrics
  fleet-level pressure and reliability
      |
      v
traces
  request and provider timing
      |
      v
runbooks
  operator questions tied to evidence
      |
      v
postmortems
  fixes and regression tests
```

Core invariant:

```text
operators should not infer state from scattered logs when durable evidence exists
```

Primary chapters: 15, 21, 23, 24, 26.

## Release And Versioning Flow

```text
old job row
  payload_schema_version = 1
  prompt_version = 2026-05-01
  model_version = deepseek-chat
  policy_version = safety-v1

new deployment
  reads old rows
  writes new rows
  checks worker compatibility range
  preserves old interpretation
  canary checks behavior and operations

version compatibility query
  finds jobs too old or too new for this worker
  routes them to migration, quarantine, or compatible worker

future incident
  event timeline plus versions explain what code, prompt, model, and policy ran
```

Core invariant:

```text
long-running systems must remember enough version context to explain old work
and reject incompatible execution
```

Primary chapters: 18, 19, 25, 27, 29, 30.

## Recovery And Replay Flow

```text
backup restored
      |
      v
jobs and events inspected
      |
      v
terminal receipts protected from duplication
      |
      v
pending and expired work selected for replay
      |
      v
provider and side-effect boundaries checked
      |
      v
operator records restore result against RPO and RTO
```

Core invariant:

```text
restore is successful only when replay is safe and side effects are not duplicated
```

Primary chapters: 12, 23, 24, 29, 30.

## Summary

The diagrams all say the same thing from different distances. Reliable agents
are not made reliable by the model. They are made reliable by durable state,
typed transitions, explicit ownership, bounded retries, evidence, approval,
versioning, and recovery practice.

## Further Reading and Sources



- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: supports the appendix's operational review habits and production-readiness vocabulary.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: keeps the appendix grounded in durable state, transactions, logs, and evidence histories.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: supports the appendix's emphasis on explicit boundaries, named types, and maintainable interfaces.