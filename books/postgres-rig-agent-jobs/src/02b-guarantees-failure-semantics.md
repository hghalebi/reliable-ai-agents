# 2.5 Guarantees And Failure Semantics

## What You Will Learn

This chapter teaches you to:

- explain what the system promises and what it does not promise;
- inspect each failure case for delivery semantics, replay safety, duplicate handling, and operator evidence;
- verify that readers do not confuse at-least-once execution with exactly-once side effects.

The production evidence is a written guarantee table tied to job states,
idempotency records, retries, dead letters, and receipts.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the layers have separate responsibilities.
- **Adds:** the promises the system does and does not make.
- **Prepares:** a Postgres ledger that can preserve those promises.

## Production Failure

A worker times out after calling a tool.

The team says the job is "reliable," but no one knows whether that means the
tool ran once, might run again, can be cancelled, or needs a human decision.

- **What breaks:** the system made reliability claims without naming the
  guarantee.
- **False fix:** promise "exactly once" in documentation while retries,
  crashes, and external side effects still exist.
- **Design response:** write the actual guarantees and non-guarantees for
  execution, side effects, retries, cancellation, dead letters, and receipts.

## Motivation

In production, reliability claims are dangerous when they are not written down. Teams say a job is reliable, but they may mean at-least-once execution, at-most-once side effects, best-effort cancellation, or something else entirely.

Without explicit guarantees, retry code and operator decisions depend on guesses. This chapter defines what the system promises, what it refuses to promise, and which evidence proves the promise held under failure.

## Plain Version

Read this as the simple version:

- **Simple rule:** Name exactly what the system promises and what happens when that promise cannot be kept.
- **Why it matters:** Long-running agents fail in partial ways, so unclear guarantees become unclear recovery behavior.
- **What to watch:** Watch every retry, timeout, cancellation, and approval path for a precise terminal or recoverable state.

## What You Already Know

Start with these anchors:

- The system is split into model interaction, durable state, worker execution, events, and policy.
- Each layer can make only certain promises.
- Duplicate execution is different from duplicate side effects.

This chapter adds: explicit guarantees and non-guarantees. You will name what
the system promises before code, retries, tests, or operators depend on it.

## Focus Cue

Keep three things in view:

- **State:** the set of promises and non-promises the system is allowed to make under failure.
- **Move:** a behavior becomes a guarantee only after its failure semantics are written into state, tests, and operations.
- **Proof:** Failure semantics are written down before retries, leases, recovery, or side effects depend on them.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a failure-semantics table for success, retryable failure, terminal failure, unknown outcome, and human wait.
- **Why it matters:** operators need to know what the system promises before they trust retries or recovery.
- **Done when:** each outcome maps to a durable state, worker action, retry rule, and evidence record.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** worker outcome types, retry policy types, cancellation types, and failure classification tests.
- **State transition:** convert each observed outcome into a durable next state.
- **Evidence path:** success, retry, permanent failure, cancellation, unknown outcome, and human wait have distinct evidence.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which guarantee applies to this outcome, and what does the system explicitly not promise?
- **Evidence to inspect:** outcome status, retry disposition, side-effect receipt, lease evidence, and failure class.
- **Escalate if:** a retry, replay, or operator action depends on a guarantee that was never written down.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** an attempt ends with success, failure, cancellation, wait, or uncertainty.
2. **Action:** classify the observed outcome before choosing the next action.
3. **Persistence:** persist the outcome state, retry disposition, and evidence.
4. **Check:** verify the system promises no more than the recorded semantics allow.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** each outcome has a named durable state and an explicit non-guarantee.
- **Validation path:** inspect failure-semantics rows, retry disposition, side-effect receipt, and lease evidence.
- **Stop if:** operators must infer retry or replay safety from unstated assumptions.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, reliability claims are dangerous when they are not written down
rule: Name exactly what the system promises and what happens when that promise cannot be kept
tiny example: the set of promises and non-promises the system is allowed to make under failure
artifact: a failure-semantics table for success, retryable failure, terminal failure, unknown outcome, and human wait
proof: each outcome has a named durable state and an explicit non-guarantee
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Worked Walkthrough

Suppose a worker claims a document-processing job.
The product team says the agent should be reliable.
That sentence is too broad to test.

Turn it into guarantees.
The job should have at most one active owner.
The worker should either complete it, retry it, dead-letter it, or release it through lease expiry.
The tool call should not create the same side effect twice.
The final state should be explainable from durable records.

Each guarantee names a failure class.
Lease ownership protects against two workers running the same attempt.
Retry limits protect against infinite work.
Idempotency protects against duplicated external effects.
Audit evidence protects the business from mystery decisions.

Now read a failure through that lens.
If the worker crashes after a provider call, the guarantee is not "nothing bad happens."
The guarantee is more precise: the recovery path can distinguish no side effect, side effect with receipt, and side effect without local confirmation.
Those are different states, and the system must treat them differently.

This is why reliable systems do not promise magic.
They promise named behavior under named failure.
Once the guarantee is named, Rust types, Postgres constraints, tests, traces, and runbooks can all point at the same invariant.

## Why Guarantees Must Be Smaller Than Marketing

Reliable systems do not begin with big promises. They begin with small promises
that can survive a bad day.

For example, "the agent will process every job exactly once" sounds comforting.
It is also usually false. A worker can crash after a model call. A network can
timeout after the provider accepted a request. A process can die after sending
an email but before writing the receipt. In those moments, the system cannot
honestly know that nothing happened.

A better promise is smaller:

```text
The job will not be lost after it is committed.
A stuck running job can be recovered after its lease expires.
A retry may repeat computation.
A risky side effect must have its own idempotency key and receipt.
An unknown outcome must be reconciled before replay.
```

These promises are less glamorous, but they are useful. They tell the worker
what to do, the database what to enforce, the tests what to simulate, and the
operator what evidence to inspect.

This is the standard for the rest of the book. A guarantee is only real when
the system can prove it during failure.

## Tiny Example

Suppose the worker calls the model and receives a good answer, then crashes
before writing the result.

The durable state still says:

```text
status: running
locked_by: worker-a
locked_until: 10:05
result: null
```

At 10:06, another worker can recover the job. The model call may happen again.
That is not a bug in the queue. It is the cost of at-least-once execution.

The safe design moves dangerous external actions behind their own idempotency
keys and receipts, so a repeated model call does not automatically become a
repeated side effect.

Read the tiny case as:

```text
setup: the provider returned an answer but the worker crashed before storing it
transition: the expired lease makes the attempt recoverable
evidence: running state, missing result, locked_until, and retry event agree
invariant: repeated computation is allowed only when side effects are protected separately
```

Notice the wording: repeated computation is allowed.

That may feel uncomfortable at first. Many engineers want to prevent repeated
work entirely. But in distributed systems, preventing repeated computation is
often harder than making repeated computation safe. The system can call the
model again if the previous result was never stored. What it must not do is
repeat a dangerous external action without idempotency and reconciliation.

This is why the book separates computation from side effects. Asking a model
for a recommendation twice may cost money and time. Sending the same email,
charging the same card, changing the same permission, or rolling back the same
service twice can damage the product. Those two risks need different
guarantees.

## Execution Guarantee

The worker loop is an at-least-once executor.

```text
pending -> running -> succeeded
pending -> running -> pending
pending -> running -> dead
```

If a worker crashes while the job is `running`, another worker can recover it
after the lease expires. That recovery is why work is not lost. It is also why a
model call or tool call might run more than once.

The system guarantees durable state transitions, not exactly-once execution.

At-least-once execution means the system prefers repeating work over losing
work. That is the right default for many background jobs, but it is not free.
Every operation inside the worker must be designed with repetition in mind.

The practical rule is:

```text
Anything inside an at-least-once worker may happen again unless another
boundary proves it already happened.
```

That boundary might be a unique idempotency key, a terminal job state, a tool
receipt, an outbox event, or a reconciliation query against an external system.
Without such evidence, retry is only a guess.

## Side-Effect Guarantee

Exactly-once side effects are not provided by the queue.

The safe promise is weaker and more honest:

```text
same logical request -> same durable job
same approved action -> same side-effect idempotency key
```

If the side-effect worker sends an email, charges a card, changes permissions,
or rolls back a deployment, that external operation needs its own idempotency
key and audit receipt.

This is the most important non-guarantee in the chapter.

The queue can make work durable. It cannot make the outside world transactional
with your database. Once a tool call crosses into an email provider, payment
processor, CRM, deployment system, or ticketing system, the local transaction
cannot simply roll it back.

So the system changes the shape of the problem. It gives the side effect a
stable identity. It records that identity before or during execution. It stores
the receipt when the external system confirms the action. If the process
crashes between those moments, replay uses the same identity and reconciles
with the external system instead of blindly acting again.

## Lease Guarantee

A lease says:

```text
worker X owns this job until time T
```

It does not say:

```text
worker X owns this job forever
worker X is still alive
no duplicate model call can happen
```

The lease is a failure-recovery contract. If the holder disappears, time turns
the stuck `running` row back into recoverable work.

A lease is deliberately temporary.

That temporary nature is what makes it safe. If a worker owned a job forever,
a crash would strand the job. If no worker had to prove ownership, two workers
could mutate the same job at once. The lease sits between those two failures:
one worker owns the job for now, and the system has a recovery path when "for
now" expires.

This is why leases and heartbeats belong in the reliability layer, not in the
prompt. The model cannot promise that a process is alive. The database can
store when ownership expires.

## Failure Matrix

| Failure point | Durable evidence | Recovery behavior | Duplicate risk |
| --- | --- | --- | --- |
| Before enqueue commits | no job row | client may retry enqueue | no durable job existed |
| After enqueue commits | `pending` job row | worker can pick it | duplicate request suppressed by key |
| After pick commits | `running` row with lease | lease expiry recovers it | agent may run again |
| After model call before result write | `running` row | lease expiry recovers it | model call may repeat |
| After retry write | `pending` row with `run_at` | worker retries later | retry is intentional |
| After success write | `succeeded` row and result | no retry | side effects must already be separate |
| After side effect before receipt | side-effect worker owns recovery | replay must use idempotency key | external action may be retried |

Read the matrix from left to right.

The first column names the moment where the crash or timeout happens. The
second column names what the system can still prove. The third column names the
allowed recovery behavior. The fourth column names the duplicate risk that
remains.

This table is not only documentation. It is a design checklist. If a new job
kind cannot fill in these four columns, its failure semantics are not ready for
production.

## Correctness Boundary

Postgres protects job state. Rust protects domain meaning. Rig is behind an
agent boundary. Policy gates protect side effects.

Do not move one boundary's responsibility into another boundary:

```text
bad: prompt says "do not do risky things"
good: policy code decides whether risky action requires approval

bad: retry loop hopes duplicate side effects do not happen
good: side-effect job has its own idempotency key

bad: event log is the only proof of job status
good: job row is the current state, event log explains how it got there
```

The reason for these boundaries is simple: each layer can only guarantee what
it controls.

Postgres can guarantee committed rows, constraints, and transactional updates.
Rust can guarantee typed construction and legal state transitions inside the
program. Rig can provide a clean model/tool boundary, but it cannot make model
behavior deterministic. Policy code can authorize an action, but it cannot
prove that an external provider executed it. Receipts and reconciliation fill
that gap.

When a design asks one layer to guarantee another layer's behavior, the system
becomes fragile. A prompt cannot enforce authorization. A log line cannot own
current state. A retry loop cannot prove an email was not already sent.

## Formal Definition

For this chapter, the precise definition is:

```text
A guarantee is a bounded promise about what the system will preserve under failure, and a non-guarantee is an assumption the system refuses to hide.
```

In the book's system model:

- **State:** the set of promises and non-promises the system is allowed to make under failure.
- **Actor:** the system designer records the semantics, and workers, stores, and runbooks implement only those semantics.
- **Transition:** a behavior becomes a guarantee only after its failure semantics are written into state, tests, and operations.
- **Evidence:** Failure semantics are written down before retries, leases, recovery, or side effects depend on them.
- **Invariant:** the system does not imply stronger reliability, such as exactly-once execution, than it can prove.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | The system implies exactly-once behavior without naming the guarantee. |
| Production symptom | Replays duplicate side effects or operators expect impossible crash behavior. |
| Corrective invariant | Execution guarantees and side-effect guarantees are stated separately. |
| Evidence to inspect | Failure semantics cover duplicate intake, provider timeout, worker crash, and replay. |


## Production Contract

Write the guarantee in the system, not only in prose:

```text
job enqueue commits before work starts
lease expiry is the recovery mechanism
retry may repeat model work
side effects require separate idempotency
events explain but do not authorize state
```

This contract should appear in tests, SQL constraints, worker transitions, and
runbooks. If a later feature assumes exactly-once execution, it is depending on
a guarantee this system does not provide.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | The system implies exactly-once behavior without naming the guarantee. | Implied guarantees make teams believe retries provide safety they never actually implemented. |
| Safer version | Execution guarantees and side-effect guarantees are stated separately. | Separate execution, delivery, and side-effect guarantees expose where duplicates and gaps can occur. |
| Production version | Failure semantics cover duplicate intake, provider timeout, worker crash, and replay. | Runbooks and tests can reason about timeout, crash, duplicate, replay, and dead-letter semantics without folklore. |

Use the naive row whenever a guarantee is assumed. Use the safer row to split the promise. Use the production row before any retry or replay path becomes automatic.

## Testing Strategy

Test every guarantee by naming the failure it does and does not cover:

- **Unit or type test:** model Rust guarantee labels for execution, delivery, and side effects; reject a policy that claims exactly-once side effects without a receipt.
- **Persistence or boundary test:** prove Postgres rows can represent duplicate intake, provider timeout, worker crash, retry scheduling, and dead-letter terminality separately.
- **Regression test:** replay a timed-out provider call and prove the system schedules or stops according to the written guarantee instead of silently assuming success.

## Observability Strategy

Observe guarantees as bounded promises:

- Emit structured `tracing` fields for job id, run id, trace id, guarantee type, delivery attempt, side-effect receipt, and failure classification.
- Record an operation event when duplicate intake, provider timeout, worker crash, retry scheduling, or dead-letter terminality occurs.
- The runbook query should distinguish what was retried, what was deduplicated, and what was never guaranteed in the first place.

## Security and Safety Considerations

Guarantees must not imply safety that the system cannot enforce:

- Treat retry, replay, and timeout outcomes as untrusted until the side-effect guarantee and receipt state are checked.
- authorization, sandboxing, and approval evidence must survive duplicate delivery and worker crash, not only the first execution attempt.
- Redact provider and user payload details from failure records while keeping guarantee type, attempt, decision, and receipt evidence.

## Operational Checklist

Use this checklist before relying on explicit guarantees and non-guarantees in production:

- **State:** Each job kind declares delivery, retry, ordering, side-effect, and
  cancellation semantics before workers depend on them.
- **Boundary:** Provider promises, database guarantees, and application guarantees are
  not mixed into one vague reliability claim.
- **Failure:** A failed run records whether the system promised retry, refusal,
  compensation, cancellation, or no guarantee.
- **Observability:** Runbook queries expose guarantee violations as state, not as
  arguments about logs.
- **Safety:** Guarantees for side effects require idempotency receipts and approval
  gates before repeated execution is allowed.

## Exercises

1. Write a negative test where a cancelled job is retried even though the job kind
   promised no retry after cancellation. Explain which idempotency key, receipt, or
   state transition prevents duplicate work.
2. Sketch the Postgres evidence: a guarantee policy row linked to job kind, retry state,
   cancellation state, and receipt expectations.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   GuaranteePolicy, DeliverySemantics, and SideEffectSemantics enums. Then name the
   runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which guarantee protects repeated computation, and which protects external side effects?
- Explain: Why is at-least-once execution not the same as exactly-once side effects?
- Apply: Classify a provider timeout followed by retry and an email send followed by crash.
- Evidence: Name the retry state, idempotency record, and side-effect receipt that prove the promise.

## Summary

The honest production model is at-least-once execution with idempotent boundaries. That is strong enough to run for years when the failure semantics are explicit.

- **Invariant:** every job kind states what the system does and does not guarantee before workers depend on it.
- **Evidence:** retry, cancellation, ordering, side-effect, and terminal-state records match the guarantee policy.
- **Carry forward:** never let a vague reliability promise stand in for a written failure contract.

## Changed Understanding

- **Before this chapter:** success and failure looked like simple return values from a function.
- **After this chapter:** every useful guarantee needs a named state, failure semantic, and recovery rule.
- **Keep:** attach each guarantee to a failure semantic, owner, and recovery evidence path.

## Further Reading & Credible References

- **[Chris Richardson: The Transactional Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)**. The canonical reference for ensuring that state changes and side effects (like agent tool calls) are decoupled yet consistent.
- **[Milan Jovanović: Exactly-once is impossible, but Idempotency is not](https://www.milanjovanovic.tech/blog/idempotent-consumer-pattern-in-dotnet)**. A clear, high-signal explanation of why distributed systems must embrace "at-least-once" delivery and solve for safety via idempotency.
- **[Pat Helland: Life Beyond Distributed Transactions—An Apostate's Opinion](https://waitingforai.com/wp-content/uploads/2021/05/helland-life-beyond-distributed-transactions.pdf)** (2007). A seminal paper explaining why scale and reliability require "entities" (agent jobs) that are managed through messaging rather than distributed locks.
- **[Designing Data-Intensive Applications](https://dataintensive.net/)** (Martin Kleppmann, Chapter 7: Transactions). Provides the formal vocabulary for isolation, atomicity, and the "Exactly-once" myth in distributed systems.
