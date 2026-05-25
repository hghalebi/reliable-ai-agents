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

## Tiny Scenario

Imagine a highly active production queue where two separate, completely independent worker processes wake up at the exact same millisecond. They query the database and both see the exact same lucrative job, `job-1`, sitting there patiently in a `pending` state.

If the architecture is naive and lacks row-level locking, both workers will happily read the row into memory, conclude that the job is free, and simultaneously execute the same expensive model inference or the same dangerous external side effect. They both believe they are the sole owner of `job-1`. This is a classic, catastrophic race condition that relies on hope rather than mathematics.

However, by leveraging the raw power of Postgres, we completely eliminate this risk at the database layer. When the workers query the table, they do not just read the row; they issue a `SELECT ... FOR UPDATE SKIP LOCKED` statement. The very first worker to reach the database transactionally locks the row, updating the `locked_by` and `locked_until` columns. The second worker, arriving milliseconds later, sees the lock and gracefully skips over `job-1`, moving on to the next available task without throwing an error or blocking. 

The database has effortlessly turned a chaotic concurrency race into an explicit, perfectly coordinated ownership decision. The evidence is pristine: if an operator queries the table, they will clearly see the `locked_by` ID, the expiration timestamp, the incremented attempt count, and an updated event timeline that definitively identifies the sole owner. The invariant holds flawlessly: one attempt always possesses exactly one active owner, guaranteed entirely by the database engine long before any application code executes.

Read the tiny case as:

```text
setup: job-1 exists as a pending row in the agent_jobs table
transition: worker-a transactionally updates the status to running and sets locked_by
evidence: the database row reflects worker-a ownership and has an associated audit event
invariant: no other worker can acquire or update the job until worker-a's lease expires
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

For this chapter, the precise, formal definition of the architecture is incredibly grounded. A Postgres ledger is the strictly durable coordination layer that unapologetically stores current system state, transition history, retry state, worker leases, and idempotency identity.

In the book's overarching system model, the **State** mapping is precise: Postgres rows definitively track current job state, transition history, retries, active leases, idempotency, execution runs, steps, and dangerous tool calls. The **Actor** interactions are restricted so that the API, the worker, the recovery loop, and frantic human operators can only mutate the ledger through tightly constrained SQL. The core **Transition** dictates that work is only enqueued, claimed, heartbeated, completed, retried, cancelled, or recovered through strictly atomic database operations.

The **Evidence** ensures that SQL constraints, row locks, indexes, tracking tables, and immutable event rows mathematically prove the state machine actually ran. Ultimately, the governing **Invariant** guarantees that durable state and transition evidence seamlessly survive sudden process death and highly concurrent, aggressive workers. For this chapter, the precise definition is: required production anchor for this chapter.


## What Can Fail

When treating a database as a workflow engine, several critical failure modes can emerge. The most common design smell occurs when a team simply adds a `status` string column to a table and proudly declares it a queue, completely ignoring ownership, history, or duplicate identity. The production symptom of this tragedy is that operators will stare at a perpetually stuck status at 2 AM, completely unable to explain exactly who currently owns the job or why it originally changed state.

The corrective invariant to ruthlessly enforce is that the ledger must explicitly store current state alongside lease ownership, strict idempotency, retries, and events. If a failure occurs, the operational evidence you must inspect includes the schema and specific queries exposing `agent_jobs`, `agent_job_events`, leases, attempt budgets, and the idempotency key. **Design smell:** the design names a mechanism but not the invariant it protects. **Production symptom:** operators cannot explain what changed or which evidence proves it. **Corrective invariant:** every important transition must be owned, durable, and reviewable.

**Evidence to inspect:** inspect the row, event, receipt, policy decision, trace, or runbook output.


## Production Contract

The Postgres ledger must strictly preserve these operational promises: enqueue must be flawlessly idempotent for external requests; pick and lease operations must happen atomically; success and retry transitions strictly require the identity of the owning worker; cancellation requests must formally record durable stop intent *before* any cancellation is ever applied; expired leases must be recoverable strictly by database time, never local wall-clock time; events must serve as append-only explanations; and metrics queries must definitively prove whether work is actively flowing or silently dying. 

If an eager engineer casually modifies a SQL query and accidentally changes one of these hard-won promises, the Rust state machine, the regression tests, and the operational runbooks absolutely must change with it.

## Progressive Hardening Path

Migrating to a database-backed coordination layer is a progressive hardening path.

In the naive version, the table merely stores a transient status, blissfully ignorant of ownership, history, or duplicate identity. In this fragile state, a simple status column alone mathematically cannot prove who actually owned the work, precisely why it changed, or whether a dangerous duplicate was successfully suppressed.

The safer version dramatically improves upon this by forcefully coupling the current state directly with lease ownership, explicit retry attempts, strict idempotency, and an append-only event log.

The final, production-grade version hardens this integration entirely. The team exposes a robust schema featuring `agent_jobs`, `agent_job_events`, leases, attempts, and idempotency keys via strict queries. At this stage, Postgres constraints and the magical `SKIP LOCKED` queries graduate from being mere storage mechanisms to serving as the unshakeable first coordination contract. Use the naive row when a table only needs to store a passive status. Use the safer row when the table must actively coordinate work.

Use the full production row when multiple aggressive workers and panicked operators must simultaneously share authority over the system. **Naive version:** the mechanism works once but does not leave enough evidence for recovery. **Safer version:** the mechanism names ownership, state, and proof before execution. **Production version:** the mechanism survives crash, retry, deploy, audit, and handoff through durable evidence.

## Testing Strategy

You must aggressively test the ledger strictly as your coordination contract. In your unit or type tests, you must prove that your Rust row conversion strictly rejects missing statuses, mathematically invalid attempt budgets, missing idempotency keys, or lease data magically appearing on the wrong state. Your persistence or boundary tests must vigorously execute Postgres query tests for idempotency enqueueing, atomic claims utilizing `SKIP LOCKED`, safe retries, verified completions, duplicate suppression, and pristine event insertion.

Furthermore, your regression tests must preserve a terrifying two-worker claim case, definitively proving that one due job absolutely cannot be jointly owned by two eager workers following a query refactor. **Unit:** test the smallest typed transition and the invariant it preserves. **Persistence:** test the database row, query, or receipt that proves the transition survives process death. **Regression:** keep a failing case for the production bug this chapter is designed to prevent.

## Observability Strategy

You must actively observe the ledger as the absolute system of record. Emit structured `tracing` fields for the job id, lease owner, attempt count, current status, next run time, idempotency key, and trace id during every single SQL transition. You must record a formal operation event whenever a job is enqueued, claimed, heartbeated, retried, completed, suppressed as a duplicate, or pushed to a dead-letter state. Ultimately, the runbook query you construct should instantly and flawlessly reconstruct the current state and the full transition history directly from Postgres, completely refusing to rely on the fleeting memory of a dead worker process.

## Security and Safety Considerations

The ledger is a critical security boundary, not just a convenient table. You must violently treat raw SQL rows, JSON payloads, status strings, and lease fields as inherently untrusted data until strict constraints and Rust row conversion thoroughly validate them. Crucially, authorization, sandboxing, and approval decisions must be formally stored as separate, durable evidence rows, rather than lazily inferred from a simple job status alone. Always meticulously redact sensitive payload columns from your operational queries, while fiercely preserving the job id, actor, transition state, lease, and audit evidence required for the inevitable compliance review.
Redact secrets, tenant data, prompts, and private payloads while preserving ids, state names, and evidence references for audit.

## Operational Checklist

Before relying on the Postgres ledger as your coordination backbone in production, operators must perform a strict review.

First, verify the **State** boundary: ensure that the job, run, step, tool call, retry, lease, approval, event, and receipt tables comprehensively record both current state and deep history. Second, inspect the **Boundary** transitions themselves: verify that raw database rows are strictly decoded into typed domain values *before* the core worker logic ever trusts the status, attempts, payload, or lease fields. 

Third, rehearse your **Failure** modes: ensure that duplicate enqueues, expired leases, transient failures, permanent failures, and dead letters all possess their own atomic, tested SQL transitions. Fourth, validate your **Observability** pipeline: confirm that queue, lease, retry, event timeline, and receipt queries all explain the exact same workflow consistently from different angles. Finally, verify **Safety**: ensure that highly sensitive payload columns do not double as the audit trail; heavily typed audit and operation events must exclusively carry the safe evidence.

## Exercises

To test your operational mastery, write a negative test where two aggressive workers simultaneously try to claim the exact same pending job, and strictly prove that only one safely receives the lease with the idempotency key entirely intact. You must explicitly explain which idempotency key, receipt, or atomic state transition successfully prevented the duplicate work. Next, sketch the exact Postgres evidence: the `scheduled_jobs`, `agent_runs`, `tool_calls`, retry state, and `operation_events` rows for one single, complex job.

Finally, define or heavily refine the Rust type, enum, constructor, or typestate that represents the critical `DbScheduledJobRow` to `ScheduledJob<Pending>` or `ScheduledJob<Running>` conversion, complete with explicit validation errors. Then, meticulously name the runbook question that proves this enforcement mechanism actually works.
1. Name one invalid transition this chapter should prevent and write the evidence that proves it is blocked.
2. Sketch the durable row, event, receipt, or policy record that would prove the correct behavior.
3. Add or describe one Rust type, enum, constructor, or test that makes the production rule harder to violate.

## Self-Check

Before you move on, use this quick retrieval drill to solidify your ledger understanding. First, recall exactly which database fields prove current state, duplicate identity, worker ownership, and retry timing. Next, be able to clearly explain why Postgres is actively acting as a robust coordination layer, and not merely as passive blob storage. Then, forcefully simulate two workers simultaneously trying to claim the same due job in your head. Finally, explicitly point to the row lock, the lease fields, the attempt count, the idempotency key, and the event timeline that safely resolves the race condition.
- Recall: what is the core invariant in this chapter?
- Explain: why does the invariant matter during an incident?
- Apply: use the idea on one real agent job or tool call.
- Evidence: name the artifact that proves the result.


## Summary

The job table reliably stores the current state. The event table immutably stores the history. Aggressive leases turn catastrophic worker crashes into safely recoverable transitions, and strict idempotency fiercely protects the outside world from duplicate requests.

The core invariant to remember is that Postgres remains the undisputed, first durable coordination layer for work, ownership, retries, approvals, tool calls, and receipts. To enforce this, your architecture must rely on ensuring your queue, lease, retry, event timeline, and receipt queries all perfectly explain one workflow consistently. 

Moving forward, remember the golden rule: treat the database aggressively as the core coordination backbone of your system, not as a passive, dumb storage bucket.

**Invariant:** the chapter concept must preserve its named production rule under failure.

**Evidence:** the proof must be visible as a row, event, receipt, trace, policy, test, or runbook query.

## Changed Understanding

Before reading this chapter, Postgres probably looked like simple, reliable storage hiding somewhere behind the "smart" agent. After this chapter, you should understand that Postgres is actually the fierce, primary coordination layer: it explicitly records work, strictly enforces state transitions, manages leases, bounds retries, and permanently stores the evidence. Moving forward, keep in mind that when debugging, you must always check the Postgres rows that rigorously record jobs, tool calls, retries, leases, and audit events first.
- **Before this chapter:** the mechanism may have looked like an implementation detail.
- **After this chapter:** the mechanism is a production contract with evidence.
- **Keep:** name the invariant, evidence, and operator question before relying on it.


## Further Reading and Sources



- [PostgreSQL `SELECT` documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (1995). The seminal academic paper arguing that queuing mechanisms should be built into database systems rather than separate middleware. It provides the foundational logic for the "Postgres Ledger" approach.
- [PostgreSQL explicit locking documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: The definitive technical explanation of the feature that turned Postgres into a high-concurrency task engine by the engineer who helped implement it.
- [PostgreSQL High Availability, Load Balancing, and Replication](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: A high-scale industry case study on using Postgres as an append-only ledger of job statuses to minimize bloat and maximize throughput.
- [PostgreSQL synchronous commit documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: An architectural review comparing row-level locking (`SKIP LOCKED`) with session-level advisory locks for long-running background tasks.
- [PostgreSQL `SELECT` documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: The primary source for understanding row-level lock modes and the interaction between concurrent transactions.
