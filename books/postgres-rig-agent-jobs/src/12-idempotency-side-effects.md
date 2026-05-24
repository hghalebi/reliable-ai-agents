# 12. Idempotency And Side Effects

## What You Will Learn

This chapter teaches you to:

- explain why retries are dangerous without idempotency;
- inspect the key, receipt, side-effect boundary, and compensation path for one logical request;
- verify that duplicate intake cannot send duplicate emails, tickets, payments, or tool actions.

The production evidence is an idempotency record that maps one logical request
to one durable side-effect path and one inspectable receipt.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** persisted rows are validated before domain logic trusts them.
- **Adds:** safe retry and replay through idempotency keys and receipts.
- **Prepares:** leases, heartbeats, timeouts, and cancellation for long-running ownership.

The previous chapter made the database boundary trustworthy. This chapter uses
that boundary for the first dangerous production problem: repeated work.

Repeated computation is usually acceptable. Re-reading a document, re-running a
classifier, or rebuilding a plan may waste time and money, but it normally does
not change the outside world. A side effect is different. Sending an email,
charging a card, opening a ticket, deleting a record, or calling a deployment API
changes something outside the worker. Once that happens, retry logic becomes a
safety problem, not just a reliability feature.

The goal of this chapter is to make one idea feel obvious: before a system can
retry a side effect, it must be able to recognize the side effect.

## Production Failure

A support agent sends a customer email, then the worker crashes before saving
success.

The retry sees no success record and sends the email again.

- **What breaks:** the side effect has no durable identity or receipt.
- **False fix:** turn off retries for any job that sends email.
- **Design response:** give the logical operation an idempotency key and record
  the side-effect receipt before replay can proceed.

## Motivation

In production, retries are unavoidable and side effects are dangerous. A client can retry after a timeout even when the first request already sent an email, opened a ticket, or called an external API.

Without idempotency, retry logic multiplies bugs. This chapter shows how one logical intent maps to one durable side-effect path and one receipt policy.

## Plain Version

Read this as the simple version:

- **Simple rule:** A retry is safe only when the same intent maps to the same side-effect path.
- **Why it matters:** Without idempotency, every timeout can become a duplicate email, charge, ticket, or external mutation.
- **What to watch:** Watch idempotency keys, receipts, outbox records, and duplicate-request handling before allowing retries.

## What You Already Know

Start with these anchors:

- The durable store can remember that a logical request already exists.
- Retrying work is not safe by itself.
- External side effects need receipts, not hope.

This chapter adds: idempotency for side effects. The same request should point
to the same durable work and the same receipt instead of sending another email,
ticket, payment, or tool action.

## Focus Cue

Keep three things in view:

- **State:** logical request identity, side-effect intent, outbox publication, receipt, and compensation evidence.
- **Move:** duplicate intent resolves to existing work, while approved side effects create receipts before replay is allowed.
- **Proof:** Idempotency keys, duplicate-suppression events, outbox rows, side-effect receipts, and compensation records agree.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** an idempotency and receipt path for every side effect that might be retried.
- **Why it matters:** retries without idempotency multiply bugs because uncertainty becomes duplicate action.
- **Done when:** one logical request maps to one durable job, one side-effect path, and one receipt or reconciliation record.

This artifact is not a single table or helper function.

It is a path through the system. The request enters with a stable identity. The
database either creates work or returns the work already created for that
identity. The worker records whether it is about to cross a side-effect
boundary. The external call produces a receipt when the system can observe one.
Replay reads that evidence before deciding what to do next.

When all of those pieces agree, a timeout becomes understandable. Without them,
the system only knows that something uncertain happened, and uncertainty plus
retry is how duplicates are born.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/outbox.rs`, `src/compensation.rs`, side-effect receipt SQL, and duplicate admission tests.
- **State transition:** map duplicate intent and uncertain side effects to one durable action path.
- **Evidence path:** replay finds the existing job, receipt, outbox event, or reconciliation requirement.

Each surface answers a different question.

The intake path asks whether this logical request already exists. The outbox
asks whether internal state and publication intent were committed together. The
receipt path asks whether the outside world accepted the action. The
compensation path asks what controlled action is allowed when the original side
effect was wrong or only partly successful.

Do not collapse these into one generic "retry table." The distinctions are what
make the system operable. A duplicate request, an unpublished outbox event, a
missing provider receipt, and an approved compensation request are different
states with different safe next moves.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** If this operation is retried, what prevents a duplicate side effect?
- **Evidence to inspect:** idempotency key, existing-job lookup, outbox event, side-effect receipt, and reconciliation row.
- **Escalate if:** an unknown outcome can be retried without finding the original logical action or receipt.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** an action may be retried after an unknown outcome.
2. **Action:** resolve the idempotency key before doing the side effect again.
3. **Persistence:** persist the existing job, outbox event, receipt, or reconciliation state.
4. **Check:** verify duplicate intent cannot create a duplicate logical side effect.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** a retry cannot create a duplicate logical side effect.
- **Validation path:** inspect idempotency lookup, outbox rows, side-effect receipts, and compensation tests.
- **Stop if:** unknown outcomes are retried without finding existing work or receipts.

This gate is about uncertainty.

The most dangerous production moment is not a clean failure. It is an unknown
outcome. The worker sent the request and then timed out. The provider returned a
response but the process crashed before saving it. The HTTP client saw a broken
connection after the remote system already acted. In all of those cases, the
system must not treat "I do not know" as permission to repeat the action.

A reliable agent system pauses at that point and looks for identity, receipt, or
reconciliation evidence. If it cannot find enough evidence, it should stop,
escalate, or require compensation. Guessing is not a retry strategy.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, retries are unavoidable and side effects are dangerous
rule: A retry is safe only when the same intent maps to the same side-effect path
tiny example: logical request identity, side-effect intent, outbox publication, receipt, and compensation evidence
artifact: an idempotency and receipt path for every side effect that might be retried
proof: a retry cannot create a duplicate logical side effect
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Intuition

An idempotency key is a promise:

```text
same logical request -> same durable job
```

It does not mean the operation is free. It means the system can recognize that
it has seen the request before.

Idempotency is not exactly-once execution. The worker may retry. The provider
may receive more than one call. The external system may see more than one
attempt. The guarantee is that all attempts use the same logical key so the
system can collapse duplicates into one durable outcome.

This distinction matters because "exactly once" is the wrong mental model for
most real systems.

The network can fail after the remote system acted. The process can crash after
the database committed. A provider can accept a request and delay its response.
Two workers can race around an expired lease. These are normal failures, not
rare exceptions.

> ### 🎓 The Professor's Corner
>
> **The Two Generals Problem: The Physics of Failure**
>
> Imagine two generals on different hills who need to attack a city at the same time. They send messengers to each other, but the messengers might get captured! 
> 
> General A says: "Attack at dawn?" 
> General B says: "OK!" 
> But then General B worries: "Did A get my OK? If not, he won't attack and I'll be slaughtered!" 
> 
> In computing, the **Two Generals Problem** proves that you can *never* be 100% sure a message was received over an unreliable network. This is why we need **Idempotency**—it's the only way to be safe when we can't be sure!

Idempotency does not make distributed systems magic. It gives repeated attempts a
shared name. With a shared name, the system can ask: Have we already accepted
this request? Have we already published this event? Have we already received a
receipt? Do we need reconciliation instead of another call?

## Tiny Example

A deployment webhook arrives with event id `deploy-742`. The HTTP caller times
out and sends the same webhook again.

Without an idempotency key:

```text
deploy-742 -> job-a
deploy-742 -> job-b
```

With an idempotency key:

```text
deploy-742 -> job-a
deploy-742 -> duplicate_suppressed(job-a)
```

The important point is not that the second request disappears. The important
point is that the second request is explained as a duplicate of known durable
work.

The key must be stable from the caller's point of view. A retry from the same
webhook, browser action, scheduler tick, or upstream message should present the
same identity even if a different worker handles it later. That is why
idempotency belongs at intake, before the system has created multiple pieces of
work.

Read the tiny case as:

```text
setup: the same deployment webhook arrives twice
transition: both deliveries resolve to one job and one side-effect path
evidence: idempotency record, existing job id, and external receipt agree
invariant: a logical request may be retried, but its side effect must be singular
```

> ### 🎓 The Professor's Corner
>
> **The Check-In Counter: The Confirmation Number**
>
> Think of an **Idempotency Key** like a "Confirmation Number" for a flight. You can show up at the check-in counter three different times, but as long as you show them the *same* confirmation number, they only give you *one* seat on the plane! 
> 
> The key is your "Student's Best Friend." It tells the system: "I've already asked for this, don't give me a second seat!"

## Code Path

The domain type is explicit:

```text
IdempotencyKey
```

The in-memory store suppresses duplicates, and the SQL store uses the unique
`idempotency_key` column.

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/enqueue_agent_job.sql}}
```

At the HTTP boundary, a duplicate key is resolved before queue pressure is
measured. This is subtle but important. A retry of already-admitted work should
not be rejected merely because the queue became saturated after the first
request. It should return the existing durable job and record that the duplicate
was suppressed:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/resolve_existing_agent_job.sql:resolve_existing_agent_job}}
```

The event ledger records the duplicate:

```text
duplicate_suppressed
```

That matters operationally. Suppression should be visible; otherwise an
operator cannot tell whether duplicate traffic is normal, malicious, or caused
by a broken client.

Notice the order of operations.

The system resolves duplicate identity before admission pressure. That means a
legitimate retry of existing work can still find its original job even if the
queue is now full. This is a subtle production property. If retries were checked
after admission control, a client could time out, retry correctly, and receive a
false rejection for work that the system already accepted.

Idempotency is part of intake truth. It is not an optimization after the queue
decision.

## Side-Effect Rule

Do not let the model directly perform irreversible actions.

Use two stages:

```text
agent job:
  read context
  reason
  propose action
  require approval when risky

side-effect job:
  run only after policy and approval
  use its own idempotency key
  write an audit event
```

This keeps retries safe. Retrying analysis is usually acceptable. Retrying a
payment, email campaign, deployment rollback, or user deletion is not.

This is what I call the **Reasoning Sandbox**. In AI systems, we often want to "think" multiple times to find the best plan. If the model's thinking step is directly tied to an external API call, we can't iterate on the plan without side effects. This separation allows for **Non-Destructive Retries** of the reasoning step—a huge win for both reliability and cost.

The receipt is the boundary between "we intended to act" and "the world may
have changed." Once a receipt exists, replay must inspect it before attempting
the action again.

The `tool_calls` table is the step before the receipt. It records the durable
claim that a tool was requested, validated, executed, failed, or rejected. The
companion code keeps that lifecycle typed so a completed side effect cannot be
confused with a model proposal:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/tool_call.rs:tool_call_typestate}}
```

The row boundary rejects non-object inputs, active calls with terminal
evidence, executed calls without output, and failed or rejected calls without a
terminal reason:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/tool_call.rs:tool_call_row_boundary}}
```

The model can propose an action. The system must decide whether that action may
change the world.

That separation is the heart of reliable agents. The model output is a proposed
intent. The policy layer decides whether the intent is allowed. The approval
layer decides whether a human must review it. The side-effect worker executes
only after the intent has durable identity and permission evidence. The receipt
then becomes the fact that future retries must respect.

This is why a tool call is not "just a function call." A function call returns to
your process. A side effect may outlive your process.

## Transactional Outbox

There is one more production gap. Sometimes the system must commit internal
state and publish a notification to another worker, broker, webhook, or
integration.

The unsafe version is:

```text
write database row
publish message
hope both happened
```

If the process crashes between the two lines, the database says the action
happened but no publisher knows about it. If the process retries without a
stable identity, the publisher may emit the same side effect twice.

The safer version is the transactional outbox:

```text
same database transaction:
  write domain state
  write outbox event

separate publisher worker:
  claim outbox event with lease
  publish externally with idempotency key
  mark published or schedule retry
```

In this model, the publisher worker should use **At-Least-Once Delivery**. It keeps trying to send the message until it receives a receipt. Because we have an idempotency key, the receiver can safely handle these repeated attempts. It's the only way to ensure the message *eventually* reaches the outside world.

The companion code models the publishing lifecycle with distinct states. A
pending event can be claimed; a claimed event can be published or turned into a
retry or permanent failure:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/outbox.rs:outbox_typestate}}
```

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/outbox.rs:outbox_publish_or_retry}}
```

The row boundary is still raw, because Postgres stores strings and JSON. The
domain boundary immediately validates status, payload shape, attempts, lease
fields, idempotency key, and terminal evidence:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/outbox.rs:outbox_row_boundary}}
```

The publisher uses the same lease discipline as the job worker:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/claim_outbox_events.sql}}
```

The important idea is not that every product needs Kafka on day one. The idea
is that publishing is work, and work needs durable state. Postgres can hold the
first version of that state clearly enough for the reader to inspect it.

The outbox is the bridge between two truths.

Inside Postgres, a transaction can say, "the job advanced and an event should be
published." Outside Postgres, the publisher still has to perform work that may
fail, time out, or be retried. The outbox makes the intention durable before the
publication happens.

That is why the event is first stored as data. A publisher can then claim it,
publish it, record success, or schedule retry. If the publisher crashes, another
publisher can inspect the same durable row. The system no longer depends on one
process surviving the gap between database commit and external publication.

## Compensation Actions

Some failures cannot be made invisible. If an external side effect happened
and later proves wrong, the system needs a compensating action. That action is
not a hidden rollback. It is a new, controlled side effect with its own
approval, idempotency key, lease, retry state, and terminal evidence:

```text
original side effect
  -> durable receipt
  -> compensation request
  -> human approval
  -> compensating worker
  -> compensation receipt or failure evidence
```

This distinction matters in production. A refund, reversal, deletion request,
or rollback can be riskier than the original action. The agent may propose the
compensation, but it should not silently execute it.

The companion code models that lifecycle as typed states. A requested
compensation can be approved:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/compensation.rs:compensation_typestate}}
```

Only an approved compensation can be claimed by a worker, and claiming it
increments the attempt count under the typed max-attempts rule:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/compensation.rs:compensation_execute_or_finish}}
```

The database row is raw at the storage boundary. The conversion layer rejects
unknown statuses, non-object payloads, negative attempts, missing approval
evidence, missing execution leases, and missing terminal failure evidence:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/compensation.rs:compensation_row_boundary}}
```

The worker claim query keeps compensation execution cooperative and durable:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/claim_compensation_actions.sql}}
```

The production lesson is: a compensation is also a side effect. It must be
approved, idempotent, leased, retried, audited, and observable.

Compensation is not time travel.

If the agent sent the wrong email, the system cannot unsend it. If it created the
wrong ticket, the system can close or annotate it, but the original action still
happened. If it charged the wrong amount, the refund is another business event,
not an eraser.

This is why compensation needs the same discipline as the original action. It
needs its own identity, policy, approval, receipt, and audit trail. The system
should be honest about history: the original side effect happened, and a later
controlled action repaired or mitigated it. This teaches us that **Architecture has Social Consequences**.

## Common Mistake

Do not generate a random idempotency key inside the worker. That defeats the
purpose. The key must come from the logical request:

```text
tenant id + external event id
tenant id + user action id
deployment id + incident id + action kind
```

A random key identifies an attempt. Idempotency needs to identify intent.

If every worker attempt creates a new key, the system has no way to recognize a
retry. It will faithfully protect each attempt from duplicating itself while
still allowing the original user action to duplicate across attempts. That is a
beautifully typed bug.

Derive the key from stable business facts. The right key should be the same when
the same logical request arrives tomorrow, on another worker, after a process
crash, or through a replay command.

## Formal Definition

For this chapter, the precise definition is:

```text
Idempotency is the mapping from one logical intent to one durable action path even when requests, workers, or providers repeat.
```

In the book's system model:

- **State:** logical request identity, side-effect intent, outbox publication, receipt, and compensation evidence.
- **Actor:** the API derives idempotency keys, and side-effect or publisher workers execute only through durable identities.
- **Transition:** duplicate intent resolves to existing work, while approved side effects create receipts before replay is allowed.
- **Evidence:** Idempotency keys, duplicate-suppression events, outbox rows, side-effect receipts, and compensation records agree.
- **Invariant:** retries may repeat computation but must not multiply external side effects.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Retry is added before duplicate identity and receipts. |
| Production symptom | A duplicate webhook, replay, or timeout issues the same external action twice. |
| Corrective invariant | One logical request maps to one durable job, one durable publication path, one durable side-effect receipt, or one approved compensation action. |
| Evidence to inspect | Unique idempotency keys, duplicate enqueue behavior, side-effect receipts, outbox publication rows, and compensation action rows prove the mapping. |


## Production Contract

Idempotency is correct only when these facts are true:

```text
the key is derived before enqueue
the key represents the logical request, not a worker attempt
duplicates return or reference the original job
suppression is recorded as an event
side-effect workers use their own idempotency keys and receipts
outbox events are claimed with leases before publication
outbox publication success or retry is recorded durably
compensation actions require approval before execution
compensation success or failure is recorded durably
```

If the key is generated after the worker starts, it is too late. The duplicate
has already become a separate piece of work.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Retry is added before duplicate identity and receipts. | Retrying before duplicate identity turns transient failure into repeated external action. |
| Safer version | One logical request maps to one durable job, one durable publication path, one durable side-effect receipt, or one approved compensation action. | Idempotency keys, outbox rows, receipts, and compensation records make one logical intent explicit. |
| Production version | Unique idempotency keys, duplicate enqueue behavior, side-effect receipts, outbox publication rows, and compensation action rows prove the mapping. | Replay can prove whether to suppress, resume, publish, compensate, or stop without guessing. |

Use the naive row when retry comes first. Use the safer row when an operation needs one identity. Use the production row before an agent can send, write, bill, or notify.

The hardening path changes the question the system can answer.

The naive version asks, "Did this attempt fail?" The safer version asks, "Which
logical operation is this attempt part of?" The production version asks, "Given
all evidence we have, what is the only safe next move?"

That final question is what operators need during incidents. They do not want a
worker to be brave. They want the system to prove whether it should suppress,
resume, publish, reconcile, compensate, or stop.

## Testing Strategy

Test side effects through duplicate identity and receipts:

- **Unit or type test:** prove Rust idempotency keys, outbox events, side-effect receipts, and compensation actions reject empty or mismatched identities.
- **Persistence or boundary test:** prove Postgres unique keys, duplicate lookup, outbox claim, receipt lookup, and compensation rows preserve one logical action path.
- **Regression test:** submit the same request twice and replay after a simulated crash; the system must return existing work or receipt instead of creating a second side effect.

The best tests in this chapter simulate uncomfortable timing.

Submit the same request twice. Crash after enqueue. Crash after the provider
accepts the request but before the receipt is stored. Let an outbox publisher
claim an event and then lose its lease. Ask replay what it will do. These tests
are not theatrical. They are the production cases that turn "retry" from a
helpful tool into a duplicate-action bug.

Each test should assert the safe decision, not only the final status.

## Observability Strategy

Observe duplicate identity before external action:

- Emit structured `tracing` fields for idempotency key, job id, outbox event id, receipt id, compensation id, trace id, and replay decision.
- Record an operation event when duplicate intake is suppressed, outbox publication is claimed, a side-effect receipt is found, or compensation is requested.
- The runbook query should answer whether retry will resume, suppress, publish, reconcile, compensate, or stop before repeating a side effect.

Good observability makes duplicate prevention visible.

If the system silently suppresses duplicates, operators cannot tell the
difference between healthy retries, a client stuck in a loop, a webhook provider
redelivering messages, or an abuse pattern. Suppression is a production event.
It should have a count, a trace, a key, and a related job or receipt.

The same is true for replay. A replay command should explain its decision before
it acts. "Found existing receipt; no external call will be made" is different
from "no receipt found; reconciliation required."

## Security and Safety Considerations

Side effects are the most dangerous boundary in the system:

- Treat replayed requests, outbox payloads, receipts, and compensation inputs as untrusted until idempotency and receipt checks pass.
- authorization, sandboxing, and approval must be rechecked or referenced before retrying publication, executing compensation, or sending external messages.
- Redact external receipt payloads while preserving idempotency key, outbox event id, receipt id, and replay decision evidence.

Idempotency is not a substitute for authorization.

A stable key can prevent duplicates, but it cannot decide whether the action was
allowed. A duplicate request from the wrong tenant, a replay after permission was
revoked, or a compensation proposal that touches sensitive data still needs the
policy and approval boundary.

Keep the evidence separate: identity proves sameness, authorization proves
permission, approval proves human control when risk requires it, and receipts
prove what happened outside the system.

## Operational Checklist

Use this checklist before relying on idempotent side effects in production:

- **State:** One logical request maps to one idempotency key, one durable job path, and
  one side-effect receipt.
- **Boundary:** External API responses and duplicate requests become receipt checks
  before any repeated side effect runs.
- **Failure:** Timeout after send, duplicate webhook, retry, and replay all resolve
  through receipt evidence.
- **Observability:** Operators can query idempotency key, side-effect target, receipt
  id, attempt, and replay decision.
- **Safety:** No email, payment, write, or destructive tool runs without authorization,
  approval if needed, and receipt policy.

Use the checklist on every new tool that can mutate the world.

Read-only tools can often be retried with simpler controls. Write tools cannot.
If a tool sends, creates, updates, deletes, deploys, charges, refunds, notifies,
or grants access, it needs an idempotency story before it becomes available to an
agent.

The rule is intentionally boring: no stable identity, no autonomous side effect.

## Exercises

1. Write a negative test where a network timeout happens after the external side effect
   and retry must return the existing idempotency receipt. Explain which idempotency
   key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: idempotency_records and side_effect_receipts rows
   linked to one logical request.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   IdempotencyKey, SideEffectReceipt, ExternalOperationId, and ReplayDecision types.
   Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What is the difference between retrying work and repeating a side effect?
- Explain: Why do external actions need receipts?
- Apply: Use "send email" or "issue refund" and name the logical idempotency key.
- Evidence: Point to the existing job, idempotency record, side-effect receipt, and replay decision that prevent duplication.

## Summary

Idempotency separates "I saw the same request twice" from "there are two pieces of work." It is the safety rail for retries, replay, and uncertain external calls.

- **Invariant:** one logical intent maps to one durable side-effect path and one receipt policy.
- **Evidence:** idempotency records, external operation ids, side-effect receipts, retry events, and replay decisions agree.
- **Carry forward:** retries without idempotency are bug multipliers.

## Changed Understanding

- **Before this chapter:** a retry looked like running the same action again.
- **After this chapter:** a retry is safe only when the side effect has stable identity and the database can recognize previous execution.
- **Keep:** inspect the idempotency key, unique constraint, side-effect receipt, and replay decision together.

## Further Reading & Credible References

- **[Pat Helland: Life Beyond Distributed Transactions—An Apostate's Opinion](https://waitingforai.com/wp-content/uploads/2021/05/helland-life-beyond-distributed-transactions.pdf)** (2007). The seminal paper explaining why scale and reliability require "at-least-once" messaging combined with idempotent entities rather than distributed locks.
- **[Airbnb Engineering: Avoiding Double Payments in a Distributed System](https://medium.com/airbnb-engineering/avoiding-double-payments-in-a-distributed-system-29c35d03e536)**. A detailed technical case study of **Orpheus**, Airbnb's idempotency library, which achieves "five nines" consistency using the exact key and lease patterns described in this chapter.
- **[Garcia-Molina & Salem: Sagas](https://www.cs.cornell.edu/andru/cs711/2002fa/reading/sagas.pdf)** (1987). The academic foundation for the "Compensation Actions" introduced in this chapter. It formalizes how to manage long-lived transactions by breaking them into smaller steps with explicit rollback actions.
- **[Chris Richardson: The Transactional Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)**. The definitive guide to solving the "Dual-Write Problem" where internal state and external notifications must be kept consistent.
- **[Designing Data-Intensive Applications](https://dataintensive.net/)** (Martin Kleppmann). Chapter 11 explains why "exactly-once" is physically impossible in asynchronous networks and why idempotency is the only robust alternative.
