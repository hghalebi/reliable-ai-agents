# 3. The Postgres Ledger

## What You Will Learn

This chapter teaches you to:

- explain why Postgres can be the first durable coordination layer for agent jobs;
- inspect the rows, locks, leases, idempotency records, and events that coordinate workers;
- verify that a crashed process does not erase the system's memory.

The production evidence is SQL that claims work atomically, records attempts,
protects idempotency, and leaves tracking rows an operator can query.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the system has named guarantees that require durable evidence.
- **Adds:** Postgres as the first coordination and evidence layer.
- **Prepares:** Rust domain types for the meanings stored in the ledger.

## Production Failure

Two workers wake up at the same time and both see the same pending job.

Without database-owned claiming, both workers believe they own it:

```text
worker-a sends the tool request
worker-b sends the tool request
the audit trail shows two side effects
```

- **What breaks:** process memory cannot coordinate shared ownership.
- **False fix:** add a sleep, a local mutex, or a bigger worker timeout.
- **Design response:** use Postgres rows, constraints, transactions, and
  `FOR UPDATE SKIP LOCKED` so ownership is atomic and inspectable.

## Motivation

In production, process memory is not a coordination layer. A worker can disappear after claiming work, two workers can race, or a retry can happen hours after the original request.

Without a durable ledger, the system cannot answer who owns a job, how many attempts occurred, whether a side effect already happened, or why work is stuck. This chapter shows how Postgres becomes the first workflow engine by making coordination visible.

## Plain Version

Read this as the simple version:

- **Simple rule:** Postgres is the first durable memory and coordination layer for agent work.
- **Why it matters:** If work only lives in process memory, a crash can erase ownership, attempts, side effects, and history.
- **What to watch:** Check that rows, locks, leases, idempotency keys, and events can explain what happened without reading worker memory.

## What You Already Know

Start with these anchors:

- A reliable agent job must survive process death.
- Guarantees need a durable place to live.
- Workers need a coordination surface they can all see.

This chapter adds: Postgres as the first ledger. Rows, constraints, locks,
indexes, and events become the system's memory before adding any external
workflow engine or queue.

## Focus Cue

Keep three things in view:

- **State:** Postgres rows for current job state, transition history, retries, leases, idempotency, runs, steps, and tool calls.
- **Move:** work is enqueued, claimed, heartbeated, completed, retried, cancelled, or recovered through atomic database operations.
- **Proof:** SQL constraints, row locks, indexes, tracking tables, and event rows prove the state machine.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** the Postgres ledger schema and claim/recovery SQL for jobs, runs, tool calls, and events.
- **Why it matters:** Postgres is the first coordination layer, so ownership and history must be queryable.
- **Done when:** concurrent workers can claim, skip, recover, and audit work using rows rather than process memory.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `sql/001_agent_jobs.sql`, `sql/002_agent_tracking.sql`, and checked query files under `examples/postgres-rig-agent-jobs/sql`.
- **State transition:** claim, track, retry, recover, and audit work through Postgres rows.
- **Evidence path:** operators can answer ownership and history questions with SQL.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Can Postgres answer who owns this work and what happened before now?
- **Evidence to inspect:** job row, lease fields, attempts, run rows, step rows, tool calls, audit events, and operation events.
- **Escalate if:** ownership or history requires inspecting process memory or worker logs alone.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** work is ready for coordination.
2. **Action:** claim, skip, retry, recover, or audit it through SQL predicates.
3. **Persistence:** persist ownership, attempts, run state, tool calls, and events.
4. **Check:** verify another worker or operator can reconstruct the state from Postgres.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** Postgres can answer ownership, progress, retries, and history for one job.
- **Validation path:** run or inspect the checked claim, recovery, metrics, and tracking SQL tests.
- **Stop if:** worker memory or unstructured logs are required to know what happened.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, process memory is not a coordination layer
rule: Postgres is the first durable memory and coordination layer for agent work
tiny example: Postgres rows for current job state, transition history, retries, leases, idempotency, runs, steps, and tool calls
artifact: the Postgres ledger schema and claim/recovery SQL for jobs, runs, tool calls, and events
proof: Postgres can answer ownership, progress, retries, and history for one job
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Tiny Example

Imagine two workers looking for work at the same time.

```text
worker-a asks for one due job
worker-b asks for one due job
job-1 is pending
```

Without row locking, both workers can believe they own `job-1`. With
`FOR UPDATE SKIP LOCKED`, the first transaction locks the row and the second
transaction skips it. The database turns a race into an explicit ownership
decision.

Read the tiny case as:

```text
setup: worker-a and worker-b can both see one pending row
transition: one transaction locks and claims the row while the other skips it
evidence: locked_by, locked_until, attempt count, and event timeline identify the owner
invariant: one attempt has one active owner
```

## Schema

The schema used by the companion crate is in:

```text
examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql}}
```

## Production Tracking Surface

The minimal ledger above is enough to teach the worker loop, but production
operators eventually ask more precise questions:

```text
Which agent run is active?
Which step is waiting for a tool?
Which tool call produced an external side effect?
Which approval request is waiting?
Which prompt and model version produced this output?
Which evaluation or audit event proves the decision?
```

The production surface below keeps those answers in Postgres. It does not add a
separate workflow engine. It splits the durable evidence into scheduling,
execution, tool, approval, evaluation, audit, operation, and memory records:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql}}
```

The naming is deliberate. `scheduled_jobs` is the generic scheduling table.
`background_jobs` carries workflow state and retry state for work that must
survive process death. `agent_runs` and `agent_steps` describe execution.
`tool_calls` and `side_effect_receipts` separate proposed tool work from proof
that an external action already crossed the boundary. `human_approval_requests`,
`human_escalations`, `evaluation_runs`, `audit_events`, `operation_events`, and
`agent_memory_records` describe the evidence a production reviewer needs later.

The first implementation can keep using `agent_jobs` as the compact runnable
ledger. The production design should still preserve the same separation of
concerns:

```text
schedule work -> run agent -> record steps -> validate tools -> request approval
  -> record side effects -> evaluate behavior -> audit decisions
```

## Generic Scheduled Jobs

The compact `agent_jobs` table is useful for the first runnable example. The
production schema also includes the more general `scheduled_jobs` table because
not every durable unit of work is the same thing as an agent run. Some jobs
send outbox events, some wait for retries, some prepare compensation, and some
start a model-backed workflow later.

The companion crate names that lifecycle directly:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/scheduled_job.rs:scheduled_job_typestate}}
```

The type states encode a simple rule:

```text
pending scheduled job -> can be claimed
running scheduled job -> can succeed, retry, dead-letter, or cancel
terminal scheduled job -> cannot be claimed by the normal worker path
```

The raw database row remains outside the domain. Storage-friendly values such
as text status, integer attempts, optional lease fields, and JSON payload are
validated before the application receives a typed scheduled job:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/scheduled_job.rs:scheduled_job_row_boundary}}
```

The SQL artifacts mirror the same lifecycle:

```text
examples/postgres-rig-agent-jobs/sql/claim_scheduled_jobs.sql
examples/postgres-rig-agent-jobs/sql/complete_scheduled_job.sql
examples/postgres-rig-agent-jobs/sql/fail_or_retry_scheduled_job.sql
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/claim_scheduled_jobs.sql}}
```

The important point is not the exact table name. The important point is that a
scheduled job is durable before work starts, leased while a worker owns it,
and either completed, retried for a future time, or dead-lettered with an
operator-readable error.

## Background Job Workflow State

`scheduled_jobs` is the due-time and lease promise. `background_jobs` is the
workflow ledger for the longer operation behind that promise. It records whether
the work is queued, leased, executing an agent, waiting for a human, waiting for
retry, completed, failed, or cancelled.

That separation matters because production questions are more precise than
"did the worker run?"

```text
Is the job merely due, or is an agent actively executing?
Is the job waiting for a human or waiting for retry?
Has the retry budget been exhausted?
Which timeout policy owned the deadline?
What failure class explains the last transition?
```

The companion crate gives the background workflow its own typestate boundary:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/background_job.rs:background_job_typestate}}
```

The row boundary validates the two dimensions together. `workflow_state`
answers where the work is. `retry_state` answers whether retry is still legal.
A row that says `completed` and `retryable`, or `waiting_for_retry` without a
next retry time and failure evidence, is not ambiguous. It is invalid.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/background_job.rs:background_job_row_boundary}}
```

This is one of the book's central production lessons: retries are not a loop
hidden in code. Retries are durable state with an attempt budget, next retry
time, failure class, and operator-visible reason.

## Agent Run Lifecycle

`scheduled_jobs` answers whether work should run. `agent_runs` answers what the
agent execution is doing right now and what evidence it left behind. This is
the table operators inspect when they ask:

```text
Which agent is running?
Which prompt and model version are involved?
Which trace id ties this run to logs and operation events?
Is the run waiting for human approval?
Did the run finish, fail, or get cancelled?
```

The companion crate gives that tracking row a typed lifecycle:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent_run.rs:agent_run_typestate}}
```

The legal transitions are deliberately small:

```text
planning -> running -> waiting_for_human -> running -> completed
planning -> running -> failed
planning -> running -> cancelled
```

The database row boundary rejects orphan runs, unknown statuses, invalid trace
ids, deadlines before the start time, terminal rows without `finished_at`, and
non-terminal rows with `finished_at` already set.

By including `prompt_version` and `model_version` in this record, we establish
**Lineage**. In Data-Centric AI, this is vital evidence. If an agent begins
behaving poorly, we can trace exactly which "brain" version (model) and which
"instruction set" (prompt) were used. This turns your ledger into a research
tool for A/B testing and continuous improvement.

This is the same raw-outside/typed-inside rule applied to execution tracking.
The SQL table stores operational facts. Rust turns those facts into a lifecycle
that prevents illegal transitions from becoming normal application behavior.

## Agent Step Timeline

`agent_runs` is the execution envelope. `agent_steps` is the timeline inside
that envelope. A run tells an operator that an agent was active. Steps explain
which part of the run was planning, calling a model, waiting at an approval
gate, calling a tool, or finalizing output.

That matters during incidents. "The run failed" is not enough. The useful
questions are more specific:

```text
Which step was running?
Which step produced output?
Which step failed?
Which step was skipped by policy?
Can this step be replayed safely?
```

The companion crate models the step lifecycle directly:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent_step.rs:agent_step_typestate}}
```

The database row boundary keeps storage-friendly strings and references from
leaking into domain code:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent_step.rs:agent_step_row_boundary}}
```

The invariant is simple: pending steps have not started; running steps have
started but not completed; succeeded steps need output evidence; failed and
skipped steps need terminal reasons. This is how a run becomes explainable
without asking the model to remember what happened.

## Failure History

`last_error` answers the current-state question:

```text
What is the latest error attached to this job?
```

It does not answer the incident question:

```text
How did this job fail over time, which attempts were retried, which attempt
crossed into dead-letter state, and which trace explains the failure?
```

For that, the production ledger needs append-only failure history. A failure
history record is not model memory. It is operational evidence. It stores the
source of the failure, the typed failure class, the operator-readable message,
the workflow state, the retry state, the outcome, the attempt budget, optional
next retry time, and trace context:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/failure_history.rs:failure_history_record}}
```

The raw database row is still allowed to use storage-friendly strings and
integers. The application is not allowed to reason over those raw values. The
row boundary converts the database shape into `FailureSource`,
`FailureOutcome`, `WorkflowState`, `RetryState`, `AttemptCount`,
`MaxAttempts`, and typed trace context before worker or runbook logic can use
it:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/failure_history.rs:failure_history_row_boundary}}
```

This gives operators a better answer than "the job is dead." They can see
whether the system retried a provider timeout, stopped on malformed model
output, escalated a policy problem to a human, cancelled a timeout breach, or
dead-lettered work after the attempt budget was exhausted.

The production invariant is:

```text
every important failure attempt gets append-only evidence
```

`last_error` is a convenience field. `failure_history` is the audit trail for
failure behavior.

## Picking Work Safely

The important concurrency primitive is:

```text
FOR UPDATE SKIP LOCKED
```

It means:

```text
If another worker already locked a due job, skip it and find another one.
```

The query must run inside a transaction. The row lock exists only for that
transaction, so the safe shape is:

```text
begin transaction
  select one due row for update skip locked
  update that row to running with locked_by and locked_until
commit
```

Postgres `READ COMMITTED` is enough for this pattern because the lock and update
are one statement. The query does not need serializable isolation to prevent two
workers from claiming the same row; `FOR UPDATE` handles that row-level
coordination.

This is worker cooperation, not consensus. It does not guarantee global
fairness, and a hot queue can still starve old work if your ordering, indexes,
or pause rules are wrong.

The query is stored in:

```text
examples/postgres-rig-agent-jobs/sql/pick_due_job.sql
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/pick_due_job.sql}}
```

## Enqueueing Work Idempotently

Production systems receive duplicate requests. A user double-clicks. A webhook
provider retries. A deploy script times out and runs again.

The queue must not translate those duplicates into duplicate external actions.
The ledger uses an `idempotency_key` so the same logical request maps to the
same durable job:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/enqueue_agent_job.sql}}
```

The mental model is simple:

```text
same idempotency key -> same job id
different idempotency key -> new work
no key -> best-effort enqueue
```

For externally triggered jobs, "no key" is usually a product smell. In production, we enforce this with a **Unique Constraint** on the `idempotency_key` (and often the `tenant_id`). This is the database's way of saying "No" to duplicate intent at the physical storage layer—the ultimate backstop for your system's integrity.

> ### 🎓 The Professor's Corner
>
> **Atomicity: The "All or Nothing" Magic Trick**
>
> In SQL, an **Atomic** operation is like a magic trick: it either happens completely, or it doesn't happen at all. There is no "halfway done" state that another worker can see. 
> 
> When you claim a job, the database ensures that the lock and the ownership update happen together. If your worker crashes exactly at that millisecond, the database rolls the whole thing back as if it never started. This "Atomicity" is the secret to why a hundred workers can look at the same notebook and never fight over the same job!

## Recovering Crashed Work

If a worker dies, it may leave a job in `running`.

The lease fixes this. A different worker can move expired work back to
`pending`:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/recover_expired_jobs.sql}}
```

Leases depend on database time, not worker wall-clock time. The SQL uses
`now()` so all workers share the same clock authority for pick, heartbeat, and
recovery decisions.

## Completing, Retrying, Cancelling

The job table is a state machine. Every query should preserve that state
machine rather than relying on application code alone.

Success is only valid for a job currently owned by a worker:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/mark_succeeded.sql}}
```

Retry distinguishes temporary failure from permanent failure:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/retry_or_dead.sql}}
```

Cancellation is explicit. It releases any lease and records why the work should
not continue:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/mark_cancelled.sql}}
```

## Queue Health

Long-running systems need a simple question answered at all times:

```text
Is work flowing, stuck, dying, or silently accumulating?
```

The metrics query gives dashboards and alerts a stable contract:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/queue_metrics.sql}}
```

Production runbooks also need narrower diagnostic queries:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/oldest_pending_job.sql}}
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/expired_leases.sql}}
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/dead_jobs_by_reason.sql}}
```

To protect the system during an incident, operators can pause a hot job kind:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/pause_job_kind.sql}}
```

The command updates the current control row and writes a control event with the
actor, reason, previous state, and new state. The picker excludes paused job
kinds before claiming work.

## Formal Definition

For this chapter, the precise definition is:

```text
A Postgres ledger is the durable coordination layer that stores current state, transition history, retry state, leases, and idempotency identity.
```

In the book's system model:

- **State:** Postgres rows for current job state, transition history, retries, leases, idempotency, runs, steps, and tool calls.
- **Actor:** API, worker, recovery loop, and operator transitions mutate the ledger through constrained SQL.
- **Transition:** work is enqueued, claimed, heartbeated, completed, retried, cancelled, or recovered through atomic database operations.
- **Evidence:** SQL constraints, row locks, indexes, tracking tables, and event rows prove the state machine.
- **Invariant:** durable state and transition evidence survive process death and concurrent workers.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | The table stores status but not ownership, history, or duplicate identity. |
| Production symptom | Operators see a stuck status but cannot explain who owns the job or why it changed. |
| Corrective invariant | The ledger stores current state, lease ownership, idempotency, retries, and events. |
| Evidence to inspect | Schema and queries expose `agent_jobs`, `agent_job_events`, leases, attempts, and idempotency key. |


## Production Contract

The ledger must preserve these promises:

```text
enqueue is idempotent for external requests
pick and lease happen atomically
success and retry require the owning worker
cancellation requests record durable stop intent before cancellation is applied
expired leases are recoverable by database time
events are append-only explanations
metrics can prove whether work is flowing
```

If a SQL query changes one of these promises, the Rust state machine, tests,
and runbooks must change with it.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | The table stores status but not ownership, history, or duplicate identity. | A status column alone cannot prove who owned work, why it changed, or whether a duplicate was suppressed. |
| Safer version | The ledger stores current state, lease ownership, idempotency, retries, and events. | The ledger couples current state with lease ownership, attempts, idempotency, and append-only events. |
| Production version | Schema and queries expose `agent_jobs`, `agent_job_events`, leases, attempts, and idempotency key. | Postgres constraints and `SKIP LOCKED` queries become the first coordination contract, not just storage. |

Use the naive row when a table only stores status. Use the safer row when the table must coordinate work. Use the production row when multiple workers and operators share authority.

## Testing Strategy

Test the ledger as the coordination contract:

- **Unit or type test:** prove Rust row conversion rejects missing status, invalid attempt budgets, missing idempotency keys, or lease data on the wrong state.
- **Persistence or boundary test:** run Postgres query tests for enqueue, claim with `SKIP LOCKED`, retry, completion, duplicate suppression, and event insertion.
- **Regression test:** preserve a two-worker claim case so one due job cannot be owned by two workers after a query refactor.

## Observability Strategy

Observe the ledger as the system of record:

- Emit structured `tracing` fields for job id, lease owner, attempt, status, next run time, idempotency key, and trace id during each SQL transition.
- Record an operation event whenever enqueue, claim, heartbeat, retry, completion, duplicate suppression, or dead-letter state changes.
- The runbook query should reconstruct current state and transition history from Postgres without relying on worker process memory.

## Security and Safety Considerations

The ledger is a security boundary, not just a table:

- Treat raw SQL rows, JSON payloads, status strings, and lease fields as untrusted until constraints and row conversion validate them.
- authorization, sandboxing, and approval decisions should be stored as separate evidence rows rather than inferred from job status alone.
- Redact sensitive payload columns from operational queries while preserving job id, actor, transition, lease, and audit evidence.

## Operational Checklist

Use this checklist before relying on the Postgres ledger as coordination backbone in production:

- **State:** Job, run, step, tool call, retry, lease, approval, event, and receipt
  tables record current state and history.
- **Boundary:** Database rows are decoded into typed domain values before worker logic
  trusts status, attempts, payload, or lease fields.
- **Failure:** Duplicate enqueue, expired lease, transient failure, permanent failure,
  and dead letter all have atomic SQL transitions.
- **Observability:** Queue, lease, retry, event timeline, and receipt queries explain
  the same workflow from different angles.
- **Safety:** Sensitive payload columns are not the audit trail; typed audit and
  operation events carry safe evidence.

## Exercises

1. Write a negative test where two workers try to claim the same pending job and only
   one receives the lease with the idempotency key intact. Explain which idempotency
   key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: the scheduled_jobs, agent_runs, tool_calls, retry
   state, and operation_events rows for one job.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   `DbScheduledJobRow` to `ScheduledJob<Pending>` or `ScheduledJob<Running>`
   conversion with validation errors. Then name the runbook question that proves it
   works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which fields prove state, duplicate identity, worker ownership, and retry timing?
- Explain: Why is Postgres acting as a coordination layer, not only storage?
- Apply: Simulate two workers trying to claim the same due job.
- Evidence: Point to the row lock, lease fields, attempts, idempotency key, and event timeline.

## Summary

The job table stores current state. The event table stores history. Leases turn worker crashes into recoverable transitions, and idempotency protects the outside world from duplicate requests.

- **Invariant:** Postgres remains the first durable coordination layer for work, ownership, retries, approvals, tool calls, and receipts.
- **Evidence:** queue, lease, retry, event timeline, and receipt queries explain one workflow consistently.
- **Carry forward:** treat the database as the coordination backbone, not as passive storage.

## Changed Understanding

- **Before this chapter:** Postgres looked like storage behind the agent.
- **After this chapter:** Postgres is the first coordination layer: it records work, state transitions, leases, retries, and evidence.
- **Keep:** check the Postgres rows that record jobs, tool calls, retries, leases, and audit events.

## Further Reading & Credible References

- **[Jim Gray: Queues are Databases](https://arxiv.org/abs/cs/0701158)** (1995). The seminal academic paper arguing that queuing mechanisms should be built into database systems rather than separate middleware. It provides the foundational logic for the "Postgres Ledger" approach.
- **[Craig Ringer (2ndQuadrant): What is SELECT SKIP LOCKED?](https://www.enterprisedb.com/blog/what-is-select-skip-locked-postgresql-9-5)**. The definitive technical explanation of the feature that turned Postgres into a high-concurrency task engine by the engineer who helped implement it.
- **[RudderStack: Lessons from Scaling PostgreSQL Queues to 100k Events Per Second](https://www.rudderstack.com/blog/postgresql-as-our-main-streaming-engine-and-queuing-system/)**. A high-scale industry case study on using Postgres as an append-only ledger of job statuses to minimize bloat and maximize throughput.
- **[Brandur Leach: Postgres Job Queues & Advisory Locks](https://brandur.org/postgres-queues)**. An architectural review comparing row-level locking (`SKIP LOCKED`) with session-level advisory locks for long-running background tasks.
- **[PostgreSQL Documentation: The `FOR UPDATE` clause](https://www.postgresql.org/docs/current/sql-select.html#SQL-FOR-UPDATE-SHARE)**. The primary source for understanding row-level lock modes and the interaction between concurrent transactions.
