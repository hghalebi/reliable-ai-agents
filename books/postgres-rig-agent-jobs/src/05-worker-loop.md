# 5. The Worker Loop

## What You Will Learn

This chapter teaches you to:

- explain how one job moves from pending work to running work to completion or retry;
- inspect the claim, lease, heartbeat, event, and status update for each transition;
- verify that worker progress survives process death and concurrent workers.

The production evidence is a worker loop that records every state change,
preserves lease ownership, and exposes stuck or failed jobs through Postgres.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** jobs are durable and typed.
- **Adds:** owned execution through claim, lease, run, event, and terminal transition.
- **Prepares:** the Rig boundary where probabilistic model behavior enters.

## Production Failure

A worker claims a job, calls a slow provider, and misses its lease deadline.

Another worker recovers the job. Then the first worker finally returns and
tries to mark the old attempt as successful.

- **What breaks:** stale ownership can overwrite the current lifecycle.
- **False fix:** make every lease very long and hope workers finish in time.
- **Design response:** require ownership predicates for heartbeat, retry,
  completion, cancellation, and recovery transitions.

## Motivation

In production, the worker is where reliability becomes real. It claims jobs, calls the agent boundary, records events, handles retries, and decides when work is terminal.

Without a disciplined worker loop, durable rows become stuck rows, retries become hidden loops, and side effects become hard to audit. This chapter turns the worker into a boring, explicit state-transition machine.

## Plain Version

Read this as the simple version:

- **Simple rule:** A worker may only advance the job it currently owns.
- **Why it matters:** The worker loop is where durable rows become real progress, retries, dead letters, or recovery.
- **What to watch:** Watch claim, heartbeat, completion, retry, cancellation, and recovery paths for lease-owner checks.

## What You Already Know

Start by anchoring yourself in the hard-won architecture you have already built. You know that the system finally possesses durable rows and fiercely typed domain values. You also know that a worker is only legally allowed to change the work it explicitly owns. Finally, you understand that every single state transition, no matter how small, absolutely must leave behind queryable evidence.

This chapter adds the engine to the car: the execution loop. The worker will relentlessly claim exactly one job, run exactly one attempt, formally record exactly one outcome, and aggressively keep its ownership highly visible through active leases and events.

## Focus Cue

Keep three critical elements fiercely in view as you read. Regarding **State**, recognize that the worker lifecycle perfectly models one due, leased, running, retrying, terminal, or recovered job. Regarding the **Move**, understand the rigid sequence: the worker claims the work, records the events, runs exactly one agent step, and permanently commits a success, retry, cancellation, or dead-letter state. Finally, regarding **Proof**, remember that the pick, heartbeat, success, retry, cancellation, and recovery transitions physically require proof of ownership and emit explicit events to the database.

If you ever get lost in the loop logic, immediately return to state, move, and proof. They form the absolute shortest path from a theoretical idea to a concrete production check at 2 AM.

## Production Artifact

Before moving on from this chapter, you must build or rigorously inspect a specific artifact: a bulletproof worker loop that strictly claims one job, meticulously records events, runs one single attempt, and commits exactly one outcome. This artifact matters intensely because passive, durable rows only ever become reliable, useful work when a disciplined worker actively advances them through explicit transitions. You will know this is "done" when the success, retry, cancellation, heartbeat, dead-letter, and recovery paths all mathematically demand undeniable lease ownership evidence before writing to the database.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/worker.rs`, `src/memory_store.rs`, `src/postgres_store.rs`, and worker lifecycle tests.
- **State transition:** claim one job, run one attempt, classify one outcome, and persist the next state.
- **Evidence path:** lease ownership is required for heartbeat, retry, completion, cancellation, and recovery.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Can this worker mutate the job it is about to finish, retry, or heartbeat?
- **Evidence to inspect:** locked_by, locked_until, attempt number, worker id, operation event, and store predicate.
- **Escalate if:** a stale or different worker can change state after ownership changed or expired.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a due job is visible to workers.
2. **Action:** one worker claims it, runs one attempt, and classifies the result.
3. **Persistence:** persist events, status, attempts, lease state, and next action.
4. **Check:** verify only the lease owner can heartbeat, complete, retry, or cancel.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** only the lease owner can advance the job lifecycle.
- **Validation path:** run worker, store, and SQL tests that cover claim, heartbeat, success, retry, cancellation, and recovery.
- **Stop if:** a stale worker can complete, retry, cancel, or heartbeat after ownership changed.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, the worker is where reliability becomes real
rule: A worker may only advance the job it currently owns
tiny example: one due, leased, running, retrying, terminal, or recovered job in the worker lifecycle
artifact: a worker loop that claims one job, records events, runs one attempt, and commits one outcome
proof: only the lease owner can advance the job lifecycle
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Worked Walkthrough

Picture the worker as a careful operator, not as a while-loop with an LLM call inside it.
The worker asks Postgres for one due job.
Postgres either grants a lease or returns nothing.
That first step decides ownership.

After the worker owns the job, it records that the attempt has started.
Only then should it call the agent runner.
If the model call fails, the worker does not guess what happened.
It classifies the failure and writes the next state.

A transient provider timeout may schedule a retry.
A policy denial may stop the job without retrying.
A malformed model output may become a validation failure.
A lost lease may force the worker to stop writing success because another worker may recover the job later.

The loop is reliable because every branch leaves durable evidence.
The important behavior is not "keep polling."
The important behavior is claim, execute, record, recover, and refuse unsafe writes.

This "refusal" is known in distributed systems as **Fencing**. By checking that
the worker still owns the lease (via ownership predicates in your SQL) before
writing any update, you ensure that a stale worker can't accidentally "fence
out" a newer worker that has already taken over the job.

This is why sleeps, local mutexes, and optimistic print statements are not enough.
They do not create shared truth.
The worker loop becomes production-grade only when Postgres owns coordination, Rust owns legal transitions, and the operator can inspect the result.

## Mental Model

The worker is a small state-transition machine:

```text
claim ownership
  -> create evidence
  -> perform one agent step
  -> classify the outcome
  -> persist the next state
```

It should not hide work in memory. If the process dies, Postgres should still
explain what was owned, what was attempted, and what can happen next.

This is the main difference between a worker and a background script.

A background script usually says, "I am running this task." A production worker
says something more precise: "I own this job until this lease expires, I am on
this attempt, and every important move I make will be visible after I die."

That precision makes recovery possible. Another worker does not need to guess
what the first worker was doing. It reads the job row, the lease, the attempt
count, and the event timeline. The database becomes shared memory between
processes that may never meet.

## Tiny Example

For one due job, a healthy run looks like:

```text
pending job
  -> worker leases it
  -> event: job_picked
  -> event: agent_started
  -> agent returns AgentResult
  -> event: agent_succeeded
  -> status: succeeded
```

If the provider times out, the same path becomes:

```text
pending job
  -> worker leases it
  -> event: agent_started
  -> transient failure
  -> status: pending
  -> run_at: now + backoff
```

The failure is not hidden in a local retry loop. It becomes scheduled state.

Read the tiny case as:

```text
setup: one due job is ready for execution
transition: the worker claims, runs, records, and either completes or schedules retry
evidence: lease fields, operation events, status, and next_run_at show progress
invariant: the worker never advances work it does not own
```

The timeout path is not a weaker version of the success path. It is a
first-class path with its own state.

That matters because production failures are normal. Providers time out.
Workers restart. Deployments drain. Tenants exhaust budgets. Operators cancel
jobs. A reliable worker does not hide these cases behind one generic error. It
classifies each outcome and writes the next state.

The state tells the next actor what to do. A pending job with `run_at` in the
future should wait. A dead job should not retry without human action. A running
job with an expired lease can be recovered. A succeeded job should not run
again.

## Mechanism

The companion crate implements the worker against a trait. The default store is
in-memory so the code runs anywhere, but the methods map directly to the SQL
queries in the previous chapter.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/worker.rs:run_once}}
```

For long-running work, the worker uses the supervised variant. The key idea is
simple: run the agent future and the heartbeat loop together. If the heartbeat
is rejected, the worker stops before it can write a stale success or retry.

Heartbeating "protects your reasoning investment." Model providers can be slow
(often 30-60 seconds for deep reasoning). If a worker doesn't heartbeat, the
system might assume it's dead and retry the expensive model call elsewhere.
Heartbeats ensure the system stays patient while the expert is thinking.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/worker.rs:run_once_with_heartbeats}}
```

The same worker boundary is also where deploy drain is enforced. A draining
worker must not claim fresh work:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/worker.rs:run_once_controlled}}
```

A production process usually repeats this cycle. The companion crate keeps that
loop bounded and typed so it cannot become an invisible infinite retry loop:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/worker.rs:run_bounded_loop}}
```

Read these code paths as four responsibilities.

First, the worker claims work. Claiming is not just selection; it is the moment
where the database grants temporary ownership. Second, the worker records
evidence before and after the agent boundary. Third, the worker classifies the
outcome into success, retry, cancellation, permanent failure, or lost ownership.
Fourth, the worker writes the next durable state only if it still has the right
to do so.

That last phrase is the heart of the chapter: only if it still has the right to
do so.

> ### 🎓 The Professor's Corner
>
> **Leases: The Library Book Rule**
>
> A **Lease** is just a time-limited contract for ownership. Think of it like renting a book from the library. You own the right to read it as long as you return or renew it by the due date. 
> 
> If you forget to renew (heartbeat), the library assumes you're done or that you've lost the book, and they'll give it to someone else! In our system, if a worker "forgets" to renew its lease, the database gives the job to another worker. It’s a simple rule that keeps the library—and your production system—moving.

## Why The Worker Records Events

A final result is not enough.

When an AI job fails, you want to know where:

```text
job picked?
agent started?
provider failed?
retry scheduled?
max attempts exhausted?
```

That is why the worker records events before and after important transitions.

Events are not a replacement for current state. They are the explanation of
current state.

The job row answers, "What is true now?" The event stream answers, "How did it
become true?" A production system needs both. Current state lets workers find
work efficiently. Events let operators reconstruct failures, prove ownership,
and learn from incidents.

When an agent behaves badly, the event stream should narrow the search. Did the
job never start? Did the provider fail? Did validation reject the model output?
Did the worker schedule retry? Did the job exhaust attempts? Each event removes
guesswork.

## The Hidden Invariant

The worker owns only the job it leased. That sounds obvious, but it protects
the whole system:

```text
worker-a leases job-1
worker-b must skip job-1
worker-a may extend job-1
worker-b may not extend job-1
another worker may recover job-1 only after the lease expires
```

This is why the store trait includes `extend_lease` and `cancel`, not only
`pick_due_job`. Production work is not always quick. A long-running agent may
need to heartbeat while it waits on model output, tools, retrieval, or human
approval.

A heartbeat is a worker saying, "I am still here, and I still own this."

If the heartbeat succeeds, the worker can continue. If the heartbeat fails, the
worker should assume ownership is gone. It must not write success just because
the model eventually returned. The system should prefer a safe retry or
operator-visible recovery over a stale write from a process that no longer owns
the job.

This is one of the places where boring state checks protect user-visible
correctness.

## Retry Is A Decision, Not A Reflex

The worker does not treat all failures the same:

```text
transient provider failure -> retry later
permanent input/config failure -> dead-letter now
```

This prevents a missing API key, invalid payload, or policy rejection from
creating an infinite retry storm.

The worker trait uses named values for these decisions:

```text
LeaseDuration
FailureMessage
RetryDisposition
CancellationReason
LeaseExtensionOutcome
CancellationOutcome
```

That is not ceremony. It prevents raw booleans and strings from becoming hidden
contracts.

Retry is dangerous when it is automatic and anonymous.

A transient timeout, a malformed payload, a policy denial, and a missing secret
all look like "failure" if the worker only sees an error string. They require
**Failure Classification**. In AI, a malformed JSON output is often a "Permanent"
bug for that specific prompt; retrying it 5 times won't fix it. Classification
lets the system say: "This is a reasoning bug, stop retrying, and escalate to a
human." This saves money, reduces latency, and keeps the operator informed.

Typed outcomes force the worker to say which failure happened before it chooses
the next state.

## Formal Definition

For this chapter, the precise definition is:

```text
A worker loop is a leased transition executor that repeatedly claims due work, records evidence, runs one step, and commits the next durable state.
```

In the book's system model:

- **State:** one due, leased, running, retrying, terminal, or recovered job in the worker lifecycle.
- **Actor:** the worker that owns the lease performs mutation; recovery and cancellation paths act through explicit preconditions.
- **Transition:** the worker claims work, records events, runs one agent step, and commits success, retry, cancellation, or dead-letter state.
- **Evidence:** Pick, heartbeat, success, retry, cancellation, and recovery transitions require ownership and emit events.
- **Invariant:** only authorized lifecycle transitions change durable work, and each transition leaves evidence.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | The worker mutates state without writing transition evidence. |
| Production symptom | A job moves from pending to terminal state with no trustworthy explanation. |
| Corrective invariant | Every important transition writes durable evidence. |
| Evidence to inspect | Worker tests and event queries show pick, start, result, retry, and terminal events. |


## Production Contract

The worker loop must preserve these rules:

```text
only the lease owner can finish, retry, or heartbeat a job
cancellation intent is recorded before the job is stopped
every important transition records an event
transient and permanent failures are classified before retry
long work heartbeats before lease expiry
shutdown stops claiming new work before terminating
```

These rules belong in the store trait, SQL predicates, tests, and runbooks. A
worker that only "keeps trying" is not a reliable worker.

The contract also protects future maintainers.

When someone adds a new worker feature, they should be able to ask: does this
path require current ownership? Does it record an event? Does it classify the
failure? Does it leave the job in a state another worker or operator can
understand?

If the answer is unclear, the feature is not ready.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | The worker mutates state without writing transition evidence. | A worker that mutates rows without evidence leaves operators guessing after crash, timeout, or retry. |
| Safer version | Every important transition writes durable evidence. | Each claim, heartbeat, result, retry, and terminal transition becomes an owned state change. |
| Production version | Worker tests and event queries show pick, start, result, retry, and terminal events. | Tests and queries can reconstruct the worker loop from durable events instead of live process memory. |

Use the naive row when work disappears into a process. Use the safer row when ownership matters. Use the production row before long-running jobs depend on worker recovery.

## Testing Strategy

Test the worker loop as leased state transition, not a process loop:

- **Unit or type test:** prove Rust worker outcomes distinguish success, retry, permanent failure, exhausted attempts, cancellation, and recovered expired leases.
- **Persistence or boundary test:** prove Postgres transitions require the current lease owner for heartbeat, completion, retry, and cancellation-related mutation.
- **Regression test:** simulate a stale worker completing work after lease expiry; the store must reject the mutation and preserve recoverable state.

## Observability Strategy

Observe worker authority at every lifecycle transition:

- Emit structured `tracing` fields for worker id, job id, lease owner, locked until, attempt, status, outcome label, and trace id.
- Record an operation event for claim, start, heartbeat, success, transient retry, permanent failure, dead-letter, cancellation, and expired-lease recovery.
- The runbook query should prove whether the worker that changed state still owned the lease at the time of mutation.

## Security and Safety Considerations

A worker should never turn lease ownership into broad authority:

- Treat claimed work, model output, and stored payloads as untrusted even after the worker owns the lease.
- authorization, sandboxing, and approval must still gate risky tools and side effects inside the leased execution step.
- Redact sensitive job payloads from worker logs while preserving worker id, lease owner, attempt, failure class, and trace evidence.

## Operational Checklist

Use this checklist before relying on the worker lifecycle in production:

- **State:** A worker claims one pending job, owns it through a lease, records attempts,
  and moves it to a terminal or retry state.
- **Boundary:** The worker trusts only typed jobs claimed through Postgres, not
  arbitrary in-memory tasks or model text.
- **Failure:** Worker crash, timeout, cancellation, transient provider failure, and
  permanent failure each map to a durable transition.
- **Observability:** Pick, start, success, retry, dead-letter, cancellation, and
  recovery events share the same job id and trace id.
- **Safety:** The worker executes side effects only through idempotent tools with
  policy, approval, and receipt evidence.

## Exercises

1. Write a negative test where a worker loses its lease before completion and another
   worker recovers the job without duplicating the side effect. Explain which
   idempotency key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: the job row before claim, after claim, after heartbeat,
   after retry, and after terminal completion.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   WorkerId, Lease, WorkerOutcome, RetryDisposition, and JobTransition types. Then name
   the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What are the worker loop's main transitions?
- Explain: Why is the worker a state-transition owner instead of a loop around a model call?
- Apply: Trace one timeout from running work to retry or dead letter.
- Evidence: Name the lease, operation event, stored state, next run time, and runbook query that prove the transition.

## Summary

The worker is not an AI abstraction. It is a reliable state-transition machine that happens to call an agent for one step.

- **Invariant:** every worker transition is owned, atomic, observable, and recoverable.
- **Evidence:** pick, lease, heartbeat, success, retry, dead-letter, cancellation, and recovery events share the same job id and trace id.
- **Carry forward:** a worker loop is production-ready only when a crash leaves enough evidence for another worker or operator to continue safely.

## Changed Understanding

- **Before this chapter:** a worker looked like a loop that polls and executes jobs.
- **After this chapter:** a worker is a lease holder that claims, heartbeats, transitions, records, and releases durable work.
- **Keep:** inspect the claim, heartbeat, complete, fail, and release transitions for one job.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
