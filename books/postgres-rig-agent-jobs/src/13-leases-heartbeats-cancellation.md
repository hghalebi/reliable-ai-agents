# 13. Leases, Heartbeats, And Cancellation

## What You Will Learn

This chapter teaches you to:

- explain how long-running work stays owned, renewable, and stoppable;
- inspect lease expiry, heartbeat freshness, cancellation requests, and recovery ownership;
- verify that no worker can hold work forever after it dies.

The production evidence is a leased job with heartbeat events, cancellation
state, expiry rules, recovery queries, and clear ownership transfer.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** side effects are separated from repeatable work.
- **Adds:** temporary ownership, renewal, timeout, and durable stop intent.
- **Prepares:** retry and dead-letter decisions that preserve failure evidence.

The previous chapter protected the outside world from duplicate side effects.
This chapter protects the work itself from confused ownership.

Long-running agent jobs are not like short function calls. A worker may hold a
job while it waits for a model, a tool, a human, a network response, or a
database transaction. During that time, the system has to answer a simple
question with production evidence:

```text
Who is allowed to move this job forward right now?
```

The answer cannot live only in process memory. Processes disappear. The answer
must live in durable state, and it must expire.

## Production Failure

A document-analysis job starts a long model call. The worker process dies, but
the database still shows the job as running.

No worker wants to pick it up because ownership never expires.

- **What breaks:** the system cannot distinguish slow work from abandoned work.
- **False fix:** wait for a human to manually reset stuck rows.
- **Design response:** make ownership time-bounded with leases, renew liveness
  with heartbeats, and record cancellation as durable state.

## Motivation

In production, workers die at inconvenient moments: after claiming a job, during a model call, while waiting on a tool, or after writing partial evidence.

Without leases and heartbeats, the system cannot distinguish slow work from abandoned work. Without cancellation state, intentional stopping becomes invisible. This chapter makes ownership, liveness, deadlines, and cancellation explicit.

## Plain Version

Read this as the simple version:

- **Simple rule:** A lease says a worker owns work for a limited time, and a heartbeat proves it is still alive.
- **Why it matters:** Long-running jobs need ownership that can expire, be renewed, be cancelled, and be recovered safely.
- **What to watch:** Watch locked_by, locked_until, heartbeat time, cancellation state, and stale-owner rejection.

## What You Already Know

Start with these anchors:

- Durable jobs can be seen by more than one worker.
- Idempotency protects intake and side effects, not worker ownership.
- Long-running work needs a way to prove the owner is still alive.

This chapter adds: leases, heartbeats, and cancellation. Workers own work only
for a limited time, renew ownership with evidence, and can stop safely when the
system asks.

## Focus Cue

Keep three things in view:

- **State:** leased work, heartbeat time, deadline policy, and durable cancellation request state.
- **Move:** ownership is claimed, extended, ended, recovered, timed out, or cancelled through explicit predicates.
- **Proof:** Ownership predicates, database time, deadlines, cancellation requests, and recovery queries are separate and inspectable.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** lease, heartbeat, timeout, and cancellation queries that preserve ownership over long work.
- **Why it matters:** long-running agents need time-bounded ownership, not permanent locks or invisible waits.
- **Done when:** stale workers are rejected, healthy workers extend leases, and stop intent is recorded before work is halted.

This artifact is a control surface for work that lasts longer than one quick
database transaction.

The lease says who may mutate the job. The heartbeat says the owner is still
alive. The timeout policy says how long the business is willing to wait. The
cancellation record says someone or something asked the system to stop.

Those facts are related, but they are not interchangeable. When they are mixed
together, operators lose the ability to tell whether the system is slow,
abandoned, overdue, or intentionally stopping.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** lease SQL, timeout modules, cancellation modules, and ownership tests.
- **State transition:** keep long work owned only for a bounded time and stop it through recorded intent.
- **Evidence path:** stale owners are rejected and cancellation leaves a durable timeline.

Each implementation surface owns a different invariant.

The lease SQL owns mutation authority. The heartbeat path owns liveness
evidence. The timeout module owns policy for work that takes too long. The
cancellation module owns stop intent and the observed outcome. The tests prove
that these boundaries stay separate under crash, recovery, and stale-worker
conditions.

Read the chapter with that separation in mind. A system that can answer "who
owns this job?" but cannot answer "should this job still be running?" is only
half operable.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Is the work still owned, alive, cancellable, or recoverable?
- **Evidence to inspect:** lease owner, locked_until, heartbeat event, cancellation request, deadline, and recovery query.
- **Escalate if:** the system cannot distinguish slow work, dead worker, expired ownership, and stop intent.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** long-running work is claimed by a worker.
2. **Action:** heartbeat ownership, observe deadlines, and process cancellation intent separately.
3. **Persistence:** persist lease extension, timeout, cancellation, and recovery evidence.
4. **Check:** verify stale ownership and stop intent are distinguishable.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** ownership, heartbeat, timeout, cancellation, and recovery are distinct states.
- **Validation path:** inspect lease SQL, cancellation rows, timeout tests, and recovery queries.
- **Stop if:** the system cannot tell slow work from dead workers, expired ownership, or stop intent.

This gate prevents a common production mistake: using one vague `running` state
to mean too many things.

`running` by itself does not tell you whether the worker is alive, whether the
lease is still valid, whether the job is past its deadline, whether an operator
asked it to stop, or whether a replacement worker is allowed to recover it. If
all of those meanings are compressed into one status, incidents become manual
interpretation exercises.

The acceptance gate says the system must preserve the distinctions before the
next worker, retry policy, or operator command makes a decision.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, workers die at inconvenient moments: after claiming a job, during a model call, while waiting on a tool, or after writing partial
rule: A lease says a worker owns work for a limited time, and a heartbeat proves it is still alive
tiny example: leased work, heartbeat time, deadline policy, and durable cancellation request state
artifact: lease, heartbeat, timeout, and cancellation queries that preserve ownership over long work
proof: ownership, heartbeat, timeout, cancellation, and recovery are distinct states
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

The job status says what phase the work is in. The lease says who currently has
authority to move it forward.

```text
status = running
locked_by = worker-a
locked_until = 10:05
```

Those fields are a temporary contract. They let other workers cooperate without
trusting process memory.

The lease also makes concurrency visible. Without it, two workers may both
believe they are helping. With it, ownership is a row predicate that every
state-changing query can check.

Think of the lease as a badge with an expiration time.

The badge does not mean the worker owns the job forever. It means the worker may
mutate the job until the badge expires, as long as every mutation proves the
badge still matches the row. This is why the owner id appears in heartbeat,
success, retry, and cancellation-aware transitions.

> ### 🎓 The Professor's Corner
>
> **The Game of Tag: Heartbeats as "Touching Base"**
>
> Think of worker ownership like a game of tag. The worker is "It" (owns the job) until the timer runs out. 
> 
> If the worker wants to stay "It" and finish the job, they have to run back and "Touch Base" (the heartbeat) to reset the timer! If they're too slow and the timer runs out, another worker can tag them out and take over. It turns a complex concurrency rule into a simple game we all know!

A stale worker may still be running in the operating-system sense. That does not
matter. If it no longer owns the lease, it no longer owns the job.

## The Lease Model

A lease says:

```text
worker X owns this job until time T
```

While the lease is valid, other workers skip the job. After the lease expires,
another worker can recover it.

Use database time for lease decisions. If worker clocks disagree, `now()` inside
Postgres is the shared authority for pick, heartbeat, and recovery queries.

> ### 🎓 The Professor's Corner
>
> **Monotonic Clocks: The Problem of Different Watches**
>
> Imagine everyone in a race has their own watch, but the watches are all set slightly differently. If one racer's watch is "Fast," they might think the race is over before it actually is! 
> 
> In a distributed system, we call this **Clock Skew**. To be safe, we use one "Master Clock"—the time inside Postgres. This is a **Monotonic Clock**, meaning it always moves forward and everyone agrees on what time it is. It's the only way to make sure leases don't expire too early!

The database is the right authority because the row is the coordination point.
Worker clocks are observations from separate machines. The lease decision must
use the same clock for every competing worker.

This is one of the few places where boring centralization is a feature.

If each worker used its own clock, a fast clock could recover work too early and
a slow clock could hold work too long. Postgres is already the coordination
backbone for the job row, so Postgres time should decide whether the lease is
still valid.

The system is not asking whether the worker feels alive. It is asking whether
the durable coordination record still grants that worker authority.

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/pick_due_job.sql}}
```

Recovery is explicit:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/recover_expired_jobs.sql}}
```

## Tiny Example

At 10:00, `worker-a` claims a job with a five-minute lease. At 10:02, the
machine dies.

Before 10:05:

```text
other workers skip the job
```

After 10:05:

```text
another worker can recover the expired lease and retry the work
```

The system did not need to know why the machine died. The lease translated
process failure into recoverable database state.

Read the tiny case as:

```text
setup: worker-a owns a job until 10:05 and then dies
transition: other workers skip before expiry and recover after expiry
evidence: locked_by, locked_until, heartbeat events, and recovery event show ownership change
invariant: ownership is temporary, observable, and recoverable
```

## Heartbeats

Long operations need lease extension:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/extend_lease.sql}}
```

The worker exposes heartbeat as a first-class operation, not as an ad hoc
query hidden inside business logic:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/worker.rs:heartbeat}}
```

The production worker path also supervises long-running execution with
heartbeats. This is the important operational pattern: the agent future may be
slow, but lease renewal keeps ownership fresh. If renewal fails, the worker
returns a lease-lost error instead of committing a result after another worker
or operator has taken control.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/worker.rs:run_once_with_heartbeats}}
```

The condition matters:

```text
status = running
locked_by = this worker
```

A worker may extend only work it owns. This prevents a stale process from
silently stealing a job back.

Heartbeats should be boring. They should not perform business work, change
payloads, or reinterpret policy. Their job is only to extend the ownership
window while the current worker is still alive.

If a heartbeat fails, the worker should treat that as a serious ownership
signal. Continuing after losing the lease risks writing a result for work that
another worker has already recovered.

A heartbeat is not a progress report.

It does not say the model is correct, the tool is safe, or the job will finish
soon. It says one narrow thing: the current owner is still present and still
asking to keep temporary authority. That narrow meaning is what makes heartbeats
useful. If heartbeats start carrying business state, they become another hidden
path for mutation.

Keep the heartbeat boring, frequent enough to be useful, and protected by the
same ownership predicate as completion.

The same ownership rule must protect terminal transitions. A worker that no
longer owns the lease must not complete the job or schedule its retry:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/mark_succeeded.sql}}
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/retry_or_dead.sql}}
```

This is a small predicate with a large consequence. Without `locked_by =
this_worker`, a stale worker can overwrite the state that a replacement worker,
operator, or cancellation path already changed.

## Deadlines Are Not Leases

A lease answers:

```text
Which worker currently owns this job?
```

A deadline answers:

```text
How long is this job allowed to stay in this phase?
```

Do not confuse them. A worker can still own a job whose user-facing deadline
has been breached. A job can also lose its lease before the business deadline
is reached. These are different production facts, so they deserve different
types and different runbook queries.

The companion crate models timeout policy as its own boundary:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/timeouts.rs:timeout_policy}}
```

A timeout policy names the policy version, the allowed duration, and the action
to take when the deadline is breached. The action is not a boolean. A timed-out
job may be retried, cancelled, escalated to a human, or dead-lettered depending
on the job kind and attempt state.

Raw database rows are converted at the boundary:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/timeouts.rs:timeout_row_boundary}}
```

The row conversion rejects unknown timeout actions, terminal observed states,
negative attempts, and deadlines that are not after `started_at`. This keeps
the worker from treating malformed timing data as production truth.

Operators can inspect breached deadlines directly:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql}}
```

The mental model is:

```text
lease expired:
  ownership may be recovered by another worker

deadline breached:
  the current work has exceeded its allowed time and needs a policy decision
```

In AI products, a timeout often means we should switch to a "Faster/Smaller" model. If a large model is taking too long to think, our **Model Switching** strategy lets us get *some* answer to the user quickly rather than failing entirely.

The distinction matters during real incidents.

A healthy worker can heartbeat forever while doing work that no longer meets the
product promise. That is a deadline problem, not a lease problem. A worker can
also die early while the job still has plenty of business time left. That is a
lease problem, not a timeout policy problem.

When those concepts share one field, the system either recovers too aggressively
or waits too long. Separate fields let the operator make the right diagnosis.

## Cancellation

Cancellation is not deletion. It is also not only a process signal.

In a production agent system, cancellation has two distinct phases:

```text
request cancellation:
  a user, operator, system policy, or timeout policy asks the system to stop

apply cancellation:
  the control plane or worker observes the request and records what happened
```

This distinction matters because a long-running agent may be inside a model
call, a tool call, a human-approval wait, or a deployment drain. Operators need
to know whether cancellation is still pending, was applied, was ignored because
the job was already terminal, or expired before it could be applied.

In distributed systems, this is a **Two-Phase Stop**. You cannot force a remote process to stop immediately (The **Halting Problem**). You can only record the intent and wait for the process to cooperate or for its lease to expire. This "Cooperative Cancellation" is the only reliable way to handle agents that might be "stuck" in a long reasoning loop.

The companion code models that lifecycle with typestate:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/cancellation.rs:cancellation_typestate}}
```

Raw database values are decoded at the boundary:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/cancellation.rs:cancellation_row_boundary}}
```

That boundary rejects unknown statuses, unknown sources, invalid expiration
times, an `applied` record without observed job status, and an
`ignored_terminal` record that observed non-terminal work. The result is a
typed record that says what happened instead of a loose string that hopes the
operator understands it later.

The direct job-state transition is still small:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/mark_cancelled.sql}}
```

The cancellation-request queue gives operators the missing visibility:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql}}
```

Use cancellation when:

```text
an operator stops unsafe work
a tenant disables the feature
a newer job supersedes old analysis
the input is withdrawn
shutdown requires explicit drain behavior
```

Cancellation is a request for control, not proof that control already happened.

That is why the chapter separates request and application. A user may cancel
while a worker is inside a provider call. An operator may cancel after the job
already completed. A shutdown process may request cancellation and then expire
before every worker observes it. Each case needs evidence.

Deleting the row would hide the story. A durable cancellation record preserves
who asked, why they asked, what state the job was in when the request was
observed, and whether the system applied or ignored the request.

## Formal Definition

For this chapter, the precise definition is:

```text
A lease is temporary mutation authority; a heartbeat renews that authority; cancellation is durable stop intent with an observed outcome.
```

In the book's system model:

- **State:** leased work, heartbeat time, deadline policy, and durable cancellation request state.
- **Actor:** the lease-owning worker renews or completes work; a requester records stop intent; recovery acts after expiry.
- **Transition:** ownership is claimed, extended, ended, recovered, timed out, or cancelled through explicit predicates.
- **Evidence:** Ownership predicates, database time, deadlines, cancellation requests, and recovery queries are separate and inspectable.
- **Invariant:** temporary ownership, customer deadlines, and stop intent remain separate and auditable.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Any worker can finish visible work. |
| Production symptom | Two workers mutate the same job, a stale worker overwrites a recovered job, or a cancellation request stays invisible. |
| Corrective invariant | Only the lease owner can heartbeat, complete, or retry active work; cancellation intent and cancellation application are separate durable records. |
| Evidence to inspect | SQL predicates and tests require `locked_by` for heartbeat, success, and retry; cancellation request rows show pending, applied, ignored, and expired intent. |


## Production Contract

Leases, heartbeats, and cancellation are correct only if:

```text
claim, heartbeat, success, and retry check worker ownership
lease time is based on database time
expired work is recovered by an explicit transition
deadline breaches are evaluated by typed timeout policy, not lease expiry
cancellation intent is durable before active work is stopped
cancellation records a reason and observed outcome instead of deleting the row
long-running work extends the lease before it expires
```

The invariant is ownership for workers and durable intent for operators. A
worker may move only the job it currently owns. An operator may request
cancellation, but the system must still record whether that request was applied,
ignored because the job was already terminal, or expired.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Any worker can finish visible work. | A worker flag or sleep loop cannot prove who owns long-running work or when ownership expires. |
| Safer version | Only the lease owner can heartbeat, complete, or retry active work; cancellation intent and cancellation application are separate durable records. | Leases, heartbeats, deadlines, cancellation requests, and recovery queries separate authority from intent. |
| Production version | SQL predicates and tests require `locked_by` for heartbeat, success, and retry; cancellation request rows show pending, applied, ignored, and expired intent. | Only the lease owner can mutate work, and expired or cancelled work has an inspectable recovery path. |

Use the naive row when ownership is implicit. Use the safer row to name temporary authority. Use the production row before multiple workers or stop requests exist.

The hardening path is a movement from hope to evidence.

The naive version hopes the worker that started the job is still the right worker
to finish it. The safer version gives the worker temporary authority and makes
that authority renewable. The production version makes every important outcome
visible: claimed, extended, expired, recovered, cancelled, ignored, or timed out.

This is what turns a background job table into an operating surface instead of a
graveyard of mysterious `running` rows.

## Testing Strategy

Test temporary authority separately from stop intent:

- **Unit or type test:** prove Rust lease, heartbeat, deadline, timeout, and cancellation types reject invalid timelines and terminal-state mutation.
- **Persistence or boundary test:** prove Postgres predicates require the owning worker and unexpired lease for mutation, while cancellation requests remain durable intent.
- **Regression test:** simulate worker crash, expired lease, and late cancellation; recovery must not let stale ownership overwrite the new state.

Good tests for this chapter should create races on purpose. I call this **"Testing the Bullies."** You want to see if your code can handle a "Bully" worker who tries to steal a job that isn't theirs. It makes your system "Tough" because it can stand up for its own truth even when other workers are acting aggressively or incorrectly.

Let `worker-a` claim a job. Let the lease expire. Let `worker-b` recover it. Then
try to let `worker-a` complete the original work. The correct result is not a
successful late write. It is a rejected stale owner.

Do the same for cancellation. Request cancellation while work is running, after
work is terminal, and after the request expires. The system should record the
observed outcome, not erase the evidence.

## Observability Strategy

Observe temporary authority and stop intent separately:

- Emit structured `tracing` fields for job id, worker id, lease owner, locked until, heartbeat time, cancellation id, deadline, and trace id.
- Record an operation event when a worker claims, extends, loses, recovers, cancels, ignores, or times out a unit of work.
- The runbook query should show whether a mutation was authorized by a live lease or blocked because ownership expired or cancellation was requested.

The operator should never have to infer ownership from silence.

If a worker is healthy, the heartbeat timeline should show it. If a lease
expired, the recovery event should say which worker took over. If a cancellation
was requested, the pending request should be visible even before a worker applies
it. If a stale worker tries to write after losing ownership, that rejection is
useful evidence.

These events turn a long-running agent from a black box into a sequence of
bounded authority decisions.

## Security and Safety Considerations

Temporary ownership does not bypass safety gates:

- Treat leased job payloads, cancellation reasons, timeout observations, and recovery rows as untrusted until validated.
- authorization, sandboxing, and approval still apply after a worker claims a lease and before it executes risky tools or side effects.
- Redact cancellation details and sensitive payloads while preserving lease owner, deadline, cancellation source, and recovery evidence.

A lease proves ownership, not permission.

This distinction is easy to miss. The worker that owns a job may still be
forbidden from executing a risky tool, sending an external message, reading a
tenant's data, or applying a compensation. Lease checks protect concurrency.
Policy and approval checks protect safety.

Keep both layers. A lease without policy is a race-free unsafe action. Policy
without a lease is a safe action that two workers might perform at the same time.

## Operational Checklist

Use this checklist before relying on leases, heartbeats, deadlines, and cancellation in production:

- **State:** Running work has a worker id, lease expiry, heartbeat time, deadline, and
  cancellation state.
- **Boundary:** Only the worker that owns the current lease can heartbeat, complete,
  retry, or release the job.
- **Failure:** Expired leases recover crashed work, missed deadlines trigger timeout
  policy, and cancellation becomes explicit state.
- **Observability:** Runbook queries expose expired leases, stale heartbeats, deadline
  breaches, cancellation requests, and owning worker.
- **Safety:** Cancellation and recovery do not duplicate side effects because receipts
  and idempotency keys are checked first.

Use the checklist whenever you add a worker type.

Every new worker should answer the same questions: How does it claim work? How
does it prove it is still alive? What deadline applies? How is cancellation
requested? What happens if it dies after a side effect? Which query proves a
stale worker cannot write?

If the answers are not visible in SQL, types, tests, and runbooks, the worker is
not ready for long-running production work.

## Exercises

1. Write a negative test where a stale worker tries to complete a job after lease expiry
   while another worker owns the idempotency path. Explain which idempotency key,
   receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: locked_by, locked_until, heartbeat_at, deadline_at,
   cancellation status, and event timeline for one job.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   Lease, WorkerId, Deadline, CancellationRequest, and TimeoutAction types. Then name
   the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What does a lease prove, what does a heartbeat prove, and what does cancellation request?
- Explain: Why should `worker-b` not complete work leased to `worker-a`?
- Apply: Simulate a worker crash and a still-heartbeating overdue job.
- Evidence: Name the lease predicate, heartbeat event, cancellation state, expiry query, and recovery transition.

## Summary

Leases recover crashed workers. Heartbeats prove long operations are still alive. Cancellation makes intentional stopping auditable instead of invisible.

- **Invariant:** only the current lease owner may complete, retry, heartbeat, or release running work.
- **Evidence:** locked_by, locked_until, heartbeat_at, deadline_at, cancellation records, and trace-linked events show ownership over time.
- **Carry forward:** ownership must expire, because processes do.

## Changed Understanding

- **Before this chapter:** a running job looked owned until the worker finished.
- **After this chapter:** a lease is temporary ownership, a heartbeat is proof of life, and cancellation is a durable control request.
- **Keep:** inspect locked_by, locked_until, heartbeat evidence, and cancellation event before declaring ownership.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
