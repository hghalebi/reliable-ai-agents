# 14. Retry, Backoff, And Dead Letters

## What You Will Learn

This chapter teaches you to:

- explain which failures deserve another attempt and which failures must stop;
- inspect retry count, max attempts, next run time, error class, and dead-letter state;
- verify that retry policy does not hide terminal bugs or multiply unsafe side effects.

The production evidence is a retry schedule that separates transient provider
or network failures from permanent validation, policy, or domain failures.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** running work has ownership and stop semantics.
- **Adds:** retry as scheduled future work and dead letter as visible terminal state.
- **Prepares:** observability and SLOs for fleets and individual jobs.

The previous chapter made ownership temporary and recoverable. This chapter
decides what happens after an owned attempt fails.
That decision is not "try again" versus "give up." A production retry system has
to answer a more precise question:

```text
Is repeating this work safe, likely to help, and worth the cost?
```

In AI, we have a unique class of failure: **Stochastic Drift**. A model might work 90% of the time but fail on a specific prompt. If we retry with the *exact same* prompt and model, we are just gambling. We might need a **Model Temperature/Variation Retry** strategy where a retry uses slightly different parameters to get past a reasoning bottleneck.

Sometimes the answer is yes. A provider timeout, a network reset, or a temporary
... (omitted) ...
rate limit may succeed later. Sometimes the answer is no. A malformed payload,
missing configuration, denied policy, or impossible state will not become valid
just because the worker waits ten seconds.

Retry policy is where the system admits that not all failures deserve optimism.

## Production Failure

A provider starts returning malformed output for one prompt version.

The worker retries every failure with the same prompt until the queue is full
and the provider bill climbs.

- **What breaks:** permanent failure is treated like transient failure.
- **False fix:** increase `max_attempts` so the job has more chances.
- **Design response:** classify failures, schedule only safe retries, and make
  exhausted or permanent failures visible as dead-letter state.

## Motivation

In production, some failures should run again and some failures should stop forever. Treating them the same creates retry storms, delayed incidents, and confusing customer outcomes.

Without typed retry policy, the worker cannot explain why it is waiting, retrying, or dead-lettering. This chapter turns failure classification into durable scheduling and terminal evidence.

## Plain Version

Read this as the simple version:

- **Simple rule:** Retry only failures that are safe to repeat, and stop when evidence says the work is exhausted.
- **Why it matters:** Blind retries multiply bugs, provider load, cost, and duplicate side effects.
- **What to watch:** Watch error classification, attempt counts, next_run_at, backoff limits, and dead-letter evidence.

## What You Already Know

Start with these anchors:

- A lease tells which worker owns the current attempt.
- A failed attempt still needs a correct next state.
- Some failures are temporary; some are terminal.

This chapter adds: retry classification. You will turn provider timeouts,
network errors, validation failures, and policy denials into explicit retry,
backoff, or dead-letter decisions.

## Focus Cue

Keep three things in view:

- **State:** classified failure attempts, retry budget, next retry time, and terminal dead-letter evidence.
- **Move:** a failure becomes retry, backoff, escalation, or dead-letter only after classification and attempt accounting.
- **Proof:** Attempt counts, failure class, next retry time, backoff, dead reason, and append-only failure history are persisted.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a retry policy that separates transient failure, permanent failure, exhausted attempts, and dead letters.
- **Why it matters:** safe retries require classification, backoff, attempt counts, and terminal evidence.
- **Done when:** each failure updates attempts, next run time, error evidence, and terminal state according to policy.

This artifact is the failure router for the worker.

The input is a failed attempt. The output is a durable next state: scheduled
retry, delayed retry, terminal permanent failure, exhausted dead letter, or
operator inspection. The policy should not hide inside a loop around a provider
call. It should be visible as a typed decision that can be tested, logged,
queried, and explained.

When the policy is explicit, a reader can trace why the system waited, why it
stopped, and what evidence an operator should inspect next.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** retry policy code, scheduled job queries, background job state, and dead-letter tests.
- **State transition:** classify failures before scheduling another attempt or terminal state.
- **Evidence path:** attempt count, next run time, backoff, last error, and terminal reason are inspectable.

Each surface has one job.

The retry policy names the decision. The scheduled-job query persists the next
time the work may run. The background-job state records whether the workflow is
waiting, failed, exhausted, or terminal. The failure-history table preserves the
sequence of attempts so the latest error does not erase the earlier pattern.

Read them together. A retry policy without durable state forgets after a crash.
Durable state without classification retries the wrong failures. A dead-letter
state without history gives operators a corpse with no diagnosis.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Why should this failure run again, stop now, or dead-letter?
- **Evidence to inspect:** failure class, attempts, max attempts, next_run_at, backoff, last_error, and dead-letter reason.
- **Escalate if:** all failures follow the same retry path or terminal states lack enough reason for review.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** an attempt fails.
2. **Action:** classify the failure as transient, permanent, exhausted, or dead-lettered.
3. **Persistence:** persist attempts, next run time, last error, and terminal reason.
4. **Check:** verify retry is safe and terminal work is explainable.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** each failure is classified before retry or terminal state.
- **Validation path:** inspect attempts, next_run_at, last_error, failure class, and dead-letter tests.
- **Stop if:** all failures share one retry behavior or dead letters lack reviewable reason.

This gate protects the system from retry theater.

Retry theater is when the system appears resilient because it keeps trying, but
the attempts do not make recovery more likely. Retrying malformed model output
with the same prompt, retrying a denied policy decision, or retrying work with a
missing secret does not increase reliability. It only delays the moment when a
human sees the real problem.

A useful retry gate asks for evidence before another attempt is scheduled: What
failed? Could it succeed later? Is the operation safe to repeat? How many times
has it already failed? What should an operator see if retry stops?

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, some failures should run again and some failures should stop forever
rule: Retry only failures that are safe to repeat, and stop when evidence says the work is exhausted
tiny example: classified failure attempts, retry budget, next retry time, and terminal dead-letter evidence
artifact: a retry policy that separates transient failure, permanent failure, exhausted attempts, and dead letters
proof: each failure is classified before retry or terminal state
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Every failure needs a classification:

```text
Retryable  -> schedule later with backoff
Permanent  -> stop and make visible
```

The worker uses `RetryDisposition` instead of a boolean so the call site says
what decision is being made.

This is a reliability boundary. A boolean such as `retry: true` hides the
reasoning. A semantic disposition can preserve whether the system saw a
transient provider failure, a permanent validation failure, exhausted attempts,
or a cancellation.

We should also respect **Model-Specific Error Codes**. For example, if a model says "Content Filter Triggered," that is a permanent failure for that input. Retrying it is useless and potentially a safety violation. Our classification must be smart enough to know when to stop.

Classification is the difference between recovery and repetition.

Recovery changes the future. It waits for a condition that might improve: a
provider recovers, a rate limit resets, a transient network path clears, or a
worker lease is reclaimed. Repetition simply does the same thing again with no
new reason to expect success.

The retry policy should force that distinction into code.

## Tiny Example

Two failures may look similar in logs:

```text
provider timeout
missing API key
```

They need opposite behavior. A timeout should become future work. A missing API
key should become visible terminal state until configuration changes.

```text
provider timeout -> retry at now + backoff
missing API key  -> dead with permanent failure reason
```

The classification protects the queue from pretending every error is temporary.

That protection matters during outages. If a provider is down, retryable
failures should spread out through scheduled backoff. If configuration is
wrong, permanent failures should stop quickly and make the missing configuration
visible. Treating both as "try again now" burns money and hides the real
problem.

Read the tiny case as:

```text
setup: two failures appear in logs, but one is transient and one is configuration error
transition: retryable failure gets future work; terminal failure gets visible dead state
evidence: error class, attempts, next_run_at, and dead-letter event explain the decision
invariant: retries are allowed only for failures that may succeed later and are safe to repeat
```

## Backoff

The default policy is exponential with a cap:

```text
attempt 1 -> 30s
attempt 2 -> 60s
attempt 3 -> 120s
...
cap       -> 300s
```

The point is not just to be polite to providers. Backoff protects your own
queue from turning one outage into a CPU and API-cost storm.

Backoff is load control.

When a shared dependency fails, every worker may discover the failure at the same
time. If every worker retries at exactly 30 seconds, they will collide again in a **Retry Storm**. To prevent this, we add **Jitter**—a bit of randomness to the backoff time. It's the standard "safety valve" that ensures workers don't all push at once.

Backoff spreads demand over time. The cap matters too. Without a cap, a job can
disappear for so long that operators stop seeing it as active work. With no
delay, the system hot-loops. A production policy chooses a bounded middle.

## Dead Letters

A dead job is not garbage. It is work the system could not complete under its
current rules.

Dead-lettered jobs should keep:

```text
job id
kind
payload
attempt count
last error
event history
timestamps
```

Operators need this data to decide whether to replay, fix configuration, patch
code, or reject the request.

Dead-letter state should feel like a work queue for operators, not a trash
folder. It is the place where the system admits, explicitly, that automation
has reached the edge of its current rules.

Replay should therefore be a controlled operation. A good replay command asks
what changed since the job died: configuration, provider behavior, prompt
version, policy, code, or input data. Without that answer, replay is just
another blind retry.

Dead-letter state should reduce ambiguity.

It should tell the operator whether the job stopped because attempts were
exhausted, the failure was permanent, policy denied the action, input was
invalid, a dependency remained unavailable, or a safety gate blocked progress.
Those cases lead to different next moves.

The dead-letter queue is not a place to hide failures from customers. It is a
place to make automation limits explicit enough for review, repair, and safe
replay.

## Failure History

`last_error` is useful, but it is a snapshot. Reliable retry systems also need
history.

Consider a job that fails five times:

```text
attempt 1 -> provider timeout -> retry scheduled
attempt 2 -> provider timeout -> retry scheduled
attempt 3 -> model output malformed -> permanent failure
```

If the system stores only the latest error, the earlier retry behavior
disappears. During an incident, that lost evidence matters. Operators need to
know whether the system saw one bad input, a repeated provider outage, an
unsafe policy decision, or retry exhaustion.

The production ledger therefore records each important failed attempt in
`failure_history`. Each row names:

```text
failure source
failure class
failure message
workflow state
retry state
failure outcome
attempt and max attempts
next retry time when retry is scheduled
trace id and span id when available
```

In distributed systems, this is your **Vector of Failure**. It allows you to see if a job is "Flapping" (succeeding then failing) or "Stalling" (consistently failing). This is the data you need for **Root Cause Analysis (RCA)**.

This keeps retry analysis honest. A retry is not just "we tried again." It is
an explicit state transition with a cause, a budget, and evidence that can be
queried later.

History also protects future debugging from recency bias.

The last error may be misleading. A job may fail twice because the provider was
down, then fail permanently because the model returned malformed output after
the provider recovered. If the system stores only the final message, the outage
pattern disappears. If it stores every meaningful failed attempt, operators can
see whether they are dealing with a single bad input, a provider incident, a
prompt regression, or retry exhaustion.

For long-running agents, failure history is part of the evidence trail.

## SQL Contract

The retry query distinguishes permanent failure from retryable failure:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/retry_or_dead.sql}}
```

The SQL contract matters because retries are state transitions, not local
control flow.

If the worker crashes after deciding to retry but before saving the next run
time, the decision disappears. If it increments attempts without preserving the
failure reason, the retry budget loses meaning. If it marks a job dead without
terminal evidence, operators get a stopped job but no review path.

The query should therefore update attempt accounting, retry schedule, failure
evidence, and terminal state as one coherent transition.

## Formal Definition

For this chapter, the precise definition is:

```text
Retry is a typed decision to schedule future work after a classified transient failure; dead lettering is terminal evidence that retry is no longer useful.
```

In the book's system model:

- **State:** classified failure attempts, retry budget, next retry time, and terminal dead-letter evidence.
- **Actor:** the worker and retry policy classify failures and persist either future work or terminal inspection state.
- **Transition:** a failure becomes retry, backoff, escalation, or dead-letter only after classification and attempt accounting.
- **Evidence:** Attempt counts, failure class, next retry time, backoff, dead reason, and append-only failure history are persisted.
- **Invariant:** retry is bounded, visible, and safe only for failures that remain worth repeating.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Retry is a loop around a failing call. |
| Production symptom | Permanent failures consume capacity, hide defects, and never reach inspection. |
| Corrective invariant | Retry is a typed scheduling decision with a terminal stop path. |
| Evidence to inspect | Retry disposition, next run time, attempt count, dead-letter reason, and append-only failure history are stored. |


## Production Contract

A retry system is safe only if:

```text
every failure has a typed classification
backoff increases delay instead of hot-looping
attempt count is persisted
dead jobs keep payload, error, and timeline evidence
failure attempts are recorded before the latest error is overwritten
operators can inspect and replay only after the root cause changes
```

Retries are a scheduling decision, not a hidden loop around a fallible call.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Retry is a loop around a failing call. | A retry loop repeats failure without knowing whether the failure is transient, permanent, or exhausted. |
| Safer version | Retry is a typed scheduling decision with a terminal stop path. | Retry disposition, backoff, attempt budget, and dead-letter state become typed scheduling decisions. |
| Production version | Retry disposition, next run time, attempt count, dead-letter reason, and append-only failure history are stored. | Operators can query what will retry, when it will retry, why it failed, and when retrying stopped. |

Use the naive row when retry is just a loop. Use the safer row to classify the failure. Use the production row before repeated work can touch providers or side effects.

> ### 🎓 The Professor's Corner
>
> **The Backoff Curve: The Bouncing Ball**
>
> Think of **Backoff** like a bouncing ball. The first time it hits the floor (fails), it bounces back quickly. But as it hits more times, it takes longer and longer to come back up. The system "gets more patient" as it fails more often!
> 
> We also add **Jitter**—which is like the ball hitting an uneven floor. It bounces in slightly different directions so it doesn't always land in the same spot as all the other balls. This keeps our system from getting overwhelmed by a "Storm" of bouncing balls!

The hardening path changes who owns the decision.

In the naive version, the failing call owns the retry loop. In the safer version,
the domain policy owns the decision. In the production version, the database owns
the evidence so a different worker, operator, or deploy can continue from the
same facts.

That is the central production move: retry becomes durable scheduling, not a
sleep inside a function.

## Testing Strategy

Test retry as a typed scheduling decision:

- **Unit or type test:** prove Rust retry policy computes bounded backoff and separates retryable, permanent, exhausted, and dead-letter outcomes.
- **Persistence or boundary test:** prove Postgres updates attempt count, next run time, failure class, failure history, and terminal dead-letter state atomically.
- **Regression test:** replay a failure at the final attempt and verify the job becomes dead-lettered instead of being scheduled forever.

Good retry tests should make **Wrong Optimism** fail. I call this being a **"Professional Pessimist."** A good engineer always assumes the worst will happen and builds a plan for it. It makes being "Defensive" feel like a superpower!

Test that a provider timeout schedules a delayed retry. Test that a malformed
payload does not. Test that the final allowed attempt becomes dead-lettered with
reviewable evidence. Test that failure history records the earlier attempts
before `last_error` changes.

The goal is not to prove the worker can recover from one error. The goal is to
prove the worker knows which errors should not be retried.

## Observability Strategy

Observe retry as a scheduled decision:

- Emit structured `tracing` fields for job id, attempt, max attempts, failure class, retry disposition, next run time, dead-letter reason, and trace id.
- Record an operation event and failure-history row when work is retried, delayed, permanently failed, exhausted, or moved to dead-letter state.
- The runbook query should list which failures will retry, when they retry, why they stopped, and which operator action is needed next.

Operators need to see the retry fleet, not only individual stack traces.

During an incident, the important questions are aggregate and specific at the
same time: How many jobs are retrying? Which failure class dominates? When will
the next retry wave happen? Which jobs have exhausted attempts? Which prompt,
model route, provider, tenant, or tool version appears in the failures?

If retry decisions are observable, operators can slow intake, disable a job
kind, rotate configuration, contact a provider, or stop replay. If retry is a
hidden loop, operators see only noise.

## Security and Safety Considerations

Retries can multiply security mistakes if gates are not durable:

- Treat retry payloads, failure messages, and dead-letter reasons as untrusted until classified and sanitized.
- authorization, sandboxing, and approval evidence must still be valid when a retry runs later, possibly on a different worker version.
- Redact provider and user error details while preserving retry disposition, attempt budget, next run time, and terminal reason.

A retry runs in the future, and the future may have different permissions.

The user may lose access. A tenant may disable the feature. A tool may be
reclassified as high risk. A prompt version may be blocked. A secret may rotate.
The retry policy must not assume that authorization from the original attempt is
still valid.

Retry scheduling preserves the right to try later. It does not preserve the
right to bypass policy later.

## Operational Checklist

Use this checklist before relying on retry policy and dead-letter state in production:

- **State:** Attempts, max attempts, retry reason, next run time, and terminal dead-
  letter reason are durable fields.
- **Boundary:** Provider, tool, policy, validation, and database failures are classified
  before retry scheduling.
- **Failure:** Retryable failures are delayed with bounded backoff; permanent or
  exhausted failures become visible dead letters.
- **Observability:** Scheduled retries, attempt counts, last error, dead-letter reason,
  and trace id are queryable.
- **Safety:** Retries run only when the operation is idempotent or has no external side
  effect to duplicate.

Use the checklist whenever you add a new failure class.

Every new failure class should answer: Is it retryable? Is retry safe? What
delay applies? What evidence is stored? When does it become terminal? What
operator action is expected? If those answers are missing, the system will fall
back to blind optimism.

## Exercises

1. Write a negative test where a permanent validation failure is not retried while a
   transient provider failure schedules a bounded idempotency-safe retry. Explain which
   idempotency key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: attempts, max_attempts, next_run_at, last_error,
   retry_state, and dead_letter_reason for one job.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   RetryPolicy, RetryCount, MaxAttempts, RetryDisposition, and DeadLetterReason types.
   Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which failures are transient, permanent, or exhausted?
- Explain: Why does backoff protect both the system and the provider?
- Apply: Classify a provider timeout and a malformed persisted payload.
- Evidence: Name the error class, attempt count, next run time, dead-letter state, and event timeline.

## Summary

Retries should convert temporary failure into future work. Dead letters should convert permanent or exhausted failure into visible operational state.

- **Invariant:** retry decisions are bounded, classified, idempotency-safe, and stored durably.
- **Evidence:** attempts, max attempts, retry reason, next_run_at, last error, dead-letter reason, and failure history show why the job will or will not run again.
- **Carry forward:** retry policy is a state machine, not optimism.

## Changed Understanding

- **Before this chapter:** failure looked like either try again or give up.
- **After this chapter:** retry policy is bounded recovery, backoff is load control, and dead-letter state is evidence for human inspection.
- **Keep:** inspect attempts, max attempts, next_run_at, last_error, and dead-letter reason as one retry record.

## Further Reading & Credible References

- **[Metcalfe & Boggs: Ethernet—Distributed Packet Switching for Local Computer Networks](https://dl.acm.org/doi/10.1145/360248.360253)** (1976). The seminal academic paper that introduced the **Binary Exponential Backoff** algorithm. It proves why dynamic retransmission delays are necessary to prevent congestive collapse in shared systems.
- **[Marc Brooker (AWS): Exponential Backoff and Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)**. The definitive industry guide to why pure exponential backoff is insufficient. It introduces "Jitter" (randomness) as the critical mechanism to break synchronization and prevent retry storms.
- **[Google SRE Book: Addressing Cascading Failures (Retry Budgets)](https://sre.google/sre-book/addressing-cascading-failures/#id-Vv9I8)**. Formalizes the concept of a "Retry Budget" (e.g., limiting retries to 10% of total traffic) to ensure that failures do not amplify load.
- **[Uber Engineering: Kafka Retry and Dead Letter Topics](https://www.uber.com/blog/reliable-reprocessing-via-kafka-retry-topics/)**. A high-scale case study on the "Multi-tiered Retry" pattern, which prevents head-of-line blocking by moving failed work to dedicated delay topics.
- **[Designing Data-Intensive Applications](https://dataintensive.net/)** (Martin Kleppmann). Connects retry policy to the "Poison Pill" problem and explains why some failures must become terminal to protect the fleet.
