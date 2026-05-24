# 20.1 Agent Handoffs And Multi-Agent Coordination

## What You Will Learn

This chapter teaches you to:

- explain how multiple agents cooperate without losing ownership;
- inspect a handoff row, permission scope, receiving agent contract, event timeline, and failure path;
- verify that a handoff is durable work transfer, not a loose chat message.

The production evidence is a typed handoff record that connects two agent runs,
preserves auditability, limits permissions, and makes stalled coordination
visible.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the blueprint names which component owns each failure.
- **Adds:** multi-agent coordination as durable responsibility transfer.
- **Prepares:** a worked production scenario that traces all controls together.

## Production Failure

A triage agent asks a deployment agent to check rollback safety through a plain
conversation message.

The triage agent retries the request, the deployment agent starts a second
check, and operators cannot tell which agent owns the next action.

**What breaks:** responsibility moved without durable ownership transfer.

The system allowed work to move through conversation text. That text may be
clear to a human reader, but it does not give the system a single owner, a
single target job, a retry rule, or an operational query. The result is
coordination that looks smart in a demo and becomes ambiguous in production.

**False fix:** make the handoff message more detailed.

A more detailed message can help the target agent understand the request, but
detail is not ownership. It still does not prove whether the target accepted
the work, rejected it, started a job, or received a duplicate request.

**Design response:** represent handoff as typed state with source run, target
agent, reason, idempotency key, decision, target job, and event evidence. The
handoff becomes a durable transfer of responsibility instead of an informal
sentence between agents.

## Motivation

In production, multi-agent systems fail when responsibility becomes informal. A triage agent may hand work to a specialist, but the system still needs to know who owns the next durable action.

Without typed handoff state, agents can duplicate work, lose context, bypass approval, or leave humans unsure which workflow is active. This chapter makes cooperation a ledgered transition, not a chat transcript.

A handoff is the answer. It is not a chat message.

## Plain Version

Read this as the simple version:

**Simple rule:** A handoff is a typed transfer of responsibility with evidence,
not a loose chat between agents.

The source agent may explain why another specialist is needed. The target agent
may accept or reject the work. The system must record that boundary. Without
that record, the team cannot tell whether work moved, duplicated, stalled, or
escaped policy.

**Why it matters:** Multi-agent systems fail when ownership, permissions,
context, and final authority are unclear. More agents do not automatically make
the system more reliable. More agents create more boundaries that need state.

**What to watch:** Watch source agent, target agent, handoff reason, accepted
scope, new job evidence, rejected handoffs, and duplicate handoff behavior.

## What You Already Know

Start with these anchors:

- Reliable work is durable, typed, owned, and observable.
- Risky work needs approval.
- Retries need idempotency.
- Operators need runbook queries.

This chapter adds: agent handoffs as durable state transitions. One agent can
transfer work to another only through a typed handoff record with permissions,
ownership, events, and failure handling.

## Focus Cue

Keep three things in view:

- **State:** handoff request, source run, source agent, target agent, reason, typed payload, decision, and target job evidence.
- **Move:** responsibility moves only through a durable handoff decision and, when accepted, exactly one target work path.
- **Proof:** Source run, source agent, target agent, idempotency key, accepted or rejected decision, and target job evidence are stored.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

**Artifact:** a handoff contract that transfers ownership, context,
idempotency, and evidence between specialist agents.

The contract says what the source agent is asking for, which target agent may
own the next step, which context is being transferred, which idempotency key
collapses duplicates, and which evidence proves the target accepted or rejected
the work.

**Why it matters:** multi-agent systems are safe only when delegation is a state
transition, not an informal message.

**Done when:** the receiving agent can prove what it owns, why it owns it, and
whether duplicate handoffs are harmless.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/handoff.rs`, idempotency records, agent run state, and handoff tests.
- **State transition:** transfer work between agents as ownership state, not as informal chat.
- **Evidence path:** handoff request, acceptance, duplicate handling, and specialist ownership are durable.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Who owns the work after this handoff, and can duplicate handoffs be detected?
- **Evidence to inspect:** handoff id, source agent, target agent, idempotency key, target job, acceptance event, and rejection reason.
- **Escalate if:** agent delegation behaves like an untracked chat message rather than an ownership transition.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** one agent needs another specialist to own part of the work.

This should happen because the target agent has a different permission scope,
tool set, domain contract, or review responsibility. If the same agent could do
the work safely, a handoff may be unnecessary complexity.

**Action:** create a handoff as a durable ownership transition. The source does
not merely write a message. It creates a typed request that the target can
accept, reject, expire, or cancel.

**Persistence:** persist source, target, payload, idempotency, acceptance, and
target job evidence. This is what survives process death and retry.

**Check:** verify duplicate or rejected handoffs do not create ambiguous
ownership. A duplicate request should collapse to the same logical transfer, and
a rejected request should not create target work later.


## Acceptance Gate

Do not move on until this minimum evidence exists:

**Minimum evidence:** handoff changes ownership durably and idempotently.

**Validation path:** inspect handoff rows, target job evidence, idempotency key,
and handoff tests. The database should answer who requested the transfer, who
accepted it, and which target job now owns the next step.

**Stop if:** delegation is only a message with ambiguous owner, duplicate
behavior, or missing target evidence. That design will fail during retries,
worker crashes, or incident review.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, multi-agent systems fail when responsibility becomes informal
rule: A handoff is a typed transfer of responsibility with evidence, not a loose chat between agents
tiny example: handoff request, source run, source agent, target agent, reason, typed payload, decision, and target job evidence
artifact: a handoff contract that transfers ownership, context, idempotency, and evidence between specialist agents
proof: handoff changes ownership durably and idempotently
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Tiny Example

An incident-triage agent analyzes a failed deployment and produces this
proposal:

```text
rollback payments-api from 2026.05.23.4 to 2026.05.23.3
```

The triage agent can identify the symptom, but it is not allowed to verify
deployment rollback safety. In AI systems, we also use handoffs for **Reasoning Optimization**. You can't just keep adding more agents to a chat because the context gets too large and the reasoning degrades. Handoffs are the way we **Prune the Context**. By handing off to a specialist, we start a fresh context with only the necessary information.

The system creates a handoff:

```text
from_agent: incident_triage_agent
to_agent: deployment_safety_agent
reason: rollback requires deployment-specific checks
idempotency_key: handoff:inc-7841:deployment-safety
status: requested
```

If the deployment-safety agent accepts, the handoff records a target job:

```text
target_job_id: deployment-safety-job-912
status: accepted
```

If it rejects, the handoff records a decision reason. In both cases, operators
can inspect the transfer. The system never relies on a hidden chat transcript
as the source of truth.

Read the tiny case as:

```text
setup: triage identifies a rollback candidate but lacks deployment authority
transition: triage creates a typed handoff to a deployment-safety agent
evidence: handoff row, target job, permission scope, idempotency key, and event timeline exist
invariant: handoff transfers responsibility without bypassing ownership or permission
```

> ### 🎓 The Professor's Corner
>
> **The Referral: The Medical Clinic**
>
> Think of specialist agents like a **Medical Clinic**. You see the nurse (triage) first, and they decide if you need a specialist. The nurse doesn't just "tell" the doctor you're coming; they write a **Referral** (the handoff record)! 
> 
> The doctor (specialist) reads the referral to understand what the nurse already found. This ensures you don't have to explain your symptoms twice, and it proves that the nurse officially handed you over to the right person.

## Mental Model

Think of a handoff as a custody transfer:

```text
source run
  -> handoff request
  -> target decision
  -> target job when accepted
  -> event and runbook evidence
```

The source agent remains accountable for requesting the transfer. The target
agent becomes accountable only after it accepts and a target job exists. The
database stores the boundary between those two responsibilities.

This is the same idea as a lease, but at a higher level. A lease says "this
worker owns this running job for a while." A handoff says "this target agent
has accepted responsibility for the next piece of work."

> ### 🎓 The Professor's Corner
>
> **Atomic Swaps: The Baton in a Relay Race**
>
> A handoff is like a **Relay Race**. The first runner (source agent) has to hand the baton (responsibility) to the next runner (target agent). 
> 
> If the runner just "throws" the baton and hopes the other person catches it, they'll probably drop it! In our system, we use an **Atomic Swap**: the source agent "gives up" the job at the exact same moment the target agent "takes" it. If the handoff isn't recorded in the database, the baton never left the first runner's hand!

Both prevent vague ownership during concurrency.

## The Core Problem

Without a durable handoff, multi-agent systems usually fail in one of four
ways:

1. The source agent assumes the target agent is working, but no target work was
   created.
2. The target agent starts work from incomplete context.
3. Duplicate retries create multiple target jobs.
4. Operators cannot tell which agent owns the next step.
5. **Context Loss:** When agents talk informally, they often forget critical information (like the 'Customer ID'). Your `handoff_row_boundary` ensures the **Data Payload** is structured. This is what we call **Context Engineering**.

These are not model-quality problems. They are state-model problems. A better
prompt cannot prove that responsibility moved exactly once.

This is the key point. Multi-agent coordination is not primarily about making
agents talk. It is about making ownership visible when work crosses a boundary.
The model can propose the transfer, but the system must decide whether the
transfer is allowed and record whether it happened.

## The Naive Solution

The naive implementation passes a message between agents:

```text
triage_agent says:
  "Deployment agent, please verify rollback safety for inc-7841."
```

This is easy to demo. It is unsafe in production because the message has no
durable identity, no typed payload boundary, no accepted or rejected state, no
target job evidence, no idempotency behavior, and no operational query.

If the process crashes after the source agent emits the message, the system
cannot prove whether the target agent received it. If the source retries, the
target may receive duplicate requests. If an operator asks who owns the
rollback check, the answer may live only in text.

## The Production-Grade Concept

A production handoff is a state machine:

```text
requested -> accepted
requested -> rejected
requested -> expired
requested -> cancelled
```

The important rule is not the names of the states. The rule is that each state
has evidence:

```text
requested:
  source run, source agent, target agent, reason, payload, idempotency key

accepted:
  target job, decision time

rejected / expired / cancelled:
  decision reason, decision time
```

The companion implementation models this as typestate:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/handoff.rs:handoff_typestate}}
```

Typestate is useful here because the legal operations depend on lifecycle. A
requested handoff can be accepted or rejected. An accepted handoff cannot be
accepted again. A rejected handoff cannot create target work later. The Rust
API makes that lifecycle harder to misuse.

## Database Boundary

The database stores handoffs as rows because the transfer must survive process
death. The row boundary is deliberately stricter than a generic JSON message.
It validates that source and target agents are different, the payload is an
object, accepted handoffs have target-job evidence, and terminal decisions have
decision evidence:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/handoff.rs:handoff_row_boundary}}
```

This follows the book's boundary rule:

```text
Raw outside.
Typed inside.
```

The database row can use storage-friendly fields. The domain model receives a
validated handoff value. Raw database strings do not become the architecture.

## Operational Query

The first operator question for handoffs is simple:

```text
Which target agents have unresolved handoffs?
```

The companion query answers that from durable state:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql}}
```

This is why a handoff is not merely a design pattern. It is an operational
surface. A runbook can inspect unresolved responsibility transfer without
reading logs, guessing from chat, or asking the model what happened.

## Category-Theory Lens

Start with engineering language: each agent step transforms one typed state into
another typed state. A handoff composes two state machines:

```text
IncidentContext -> TriageDecision
TriageDecision -> HandoffRequest
HandoffRequest -> DeploymentSafetyJob
DeploymentSafetyJob -> ApprovalProposal
```

Because the output of one step becomes the input of the next, the boundary must
be explicit. 

In category-theory language, these are composable transformations. We can think of the handoff state as a **Monad**—a "Reliability Shell" that wraps the agent logic. It ensures that the "Inside" logic (how the agent thinks) can change without breaking the "Outside" protocol (how work moves). This keeps the whole system **Decoupled** even as you add more specialist agents.

The useful lesson is practical: composition is safe only when the types match
and the side effects are isolated.

The dangerous operation is not drafting the handoff reason. The dangerous
operation is creating target work and changing ownership. That transition needs
idempotency, persistence, and evidence.

## Formal Definition

For this chapter, the precise definition is:

```text
A handoff is durable responsibility transfer from one named agent/run to another, with an explicit reason, payload, decision, and target-work evidence.
```

In the book's system model:

- **State:** handoff request, source run, source agent, target agent, reason, typed payload, decision, and target job evidence.
- **Actor:** the source agent requests transfer, and the target agent or handoff worker accepts, rejects, expires, or cancels it.
- **Transition:** responsibility moves only through a durable handoff decision and, when accepted, exactly one target work path.
- **Evidence:** Source run, source agent, target agent, idempotency key, accepted or rejected decision, and target job evidence are stored.
- **Invariant:** multi-agent cooperation never hides responsibility in conversation context.

## What Can Fail

**Design smell:** agents pass responsibility through conversation text or raw
JSON without a durable handoff state. The request may look structured, but the
system has no ownership transition.

**Production symptom:** work waits forever between specialist agents, duplicate
target jobs appear, or no one can prove who owns the next step. Operators end up
reading logs or asking the model instead of querying the ledger.

**Corrective invariant:** responsibility transfer is a typed, idempotent,
persisted state transition with accepted, rejected, expired, or cancelled
evidence.

**Evidence to inspect:** `agent_handoffs`, target job rows, decision reason,
idempotency key, operation event, trace id, and the pending-handoffs runbook
query.

## Production Contract

A handoff is production-ready only when it preserves a few promises.

The source run is known. The source agent and target agent are different. The
reason is explicit. The payload is validated before domain use. The handoff has
one idempotency key. accepted handoffs create or attach exactly one target job.
Rejected, expired, or cancelled handoffs have decision evidence. Operators can
query unresolved handoffs.

This is a **Distributed Transaction**. You are moving work from one "Domain" to another. If you don't use a ledger, you have no way to ensure **Linearizability**—meaning you might end up with both agents thinking they own the work, or neither.

Rig may help an agent decide that a specialist is needed. Rig does not make the
handoff reliable by itself. PostgreSQL stores the responsibility transfer, Rust
names the legal transitions, and runbooks make unresolved transfers visible.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | The source agent writes a message to another agent in plain text. | Fast to prototype, but Rust has no type for ownership transfer, SQL has no handoff row, and there is no durable evidence for tests, traces, or runbooks. |
| Safer version | The system introduces a handoff record with source agent, target agent, reason, status, and idempotency. | Duplicate transfers collapse to one logical request, and Postgres can answer which handoffs are waiting. |
| Production version | The handoff has typestate, row-to-domain validation, accepted target-job evidence, terminal decision evidence, operation events, trace id propagation, regression tests, and runbook queries. | Responsibility transfer becomes a durable evidence chain instead of an implicit conversation. |

## Testing Strategy

**Unit or type test:** prove the Rust lifecycle. A requested handoff can become
accepted, rejected, expired, or cancelled; an accepted handoff cannot transition
again through the public API. Constructors should reject empty agent names,
same-agent loops, empty reasons, and invalid idempotency keys.

**Persistence or boundary test:** decode representative Postgres rows through
the row boundary. These tests should reject unknown statuses, non-object
payloads, accepted rows without target jobs, and terminal rows without decision
reasons.

**Regression test:** recreate the production failures: duplicate source retry
with the same idempotency key, target-agent rejection, expired handoff, and a
malformed row from an older migration. The negative test matters because
handoff bugs are often invisible until operators investigate a stuck workflow.

Postgres tests should check the pending-handoffs query and the constraints that
protect the state machine. Rust tests should check that typed transitions and
row conversion agree with the SQL model.

## Observability Strategy

Every handoff should carry the same trace id as the source run when possible.
The system should emit structured `tracing` events when a handoff is requested,
accepted, rejected, expired, or cancelled. Each transition should also create an
operation event so operators can correlate the transfer with the job timeline.

The purpose is not to create more logs. The purpose is to make responsibility
movement visible. When an incident review asks why a specialist agent acted,
the trace, operation event, handoff row, and target job should tell the same
story.

The runbook query should answer at least three questions:

```text
Which target agents have pending handoffs?
How old is the oldest pending handoff?
Which source runs are waiting on unresolved transfers?
```

Logs are useful for local debugging, but the durable handoff row and runbook
query are the authority. If the trace exists but the database has no handoff
row, ownership did not move.

## Security and Safety Considerations

A handoff payload may contain untrusted model output, user content, or retrieved
context. Do not let that payload grant authority. The target agent still needs
authorization checks, sandboxing constraints, and approval gates before risky
tools or external side effects execute.

This distinction matters because the source agent may be trusted for triage but
not trusted for deployment, refund, data deletion, or compliance approval. The
handoff can move responsibility to a specialist. It cannot smuggle the source
agent's permissions into the target boundary.

Redact sensitive payload fields in debug output and operation messages. Store
only the evidence needed for accountability, replay, and policy review. If a
handoff crosses tenant, role, or data-access boundaries, require a deterministic
policy decision before creating target work.

The safety rule is simple:

```text
a handoff transfers responsibility, not permission
```

## Operational Checklist

| Check | Question |
| --- | --- |
| State | Does each handoff have a requested, accepted, rejected, expired, or cancelled state? |
| Boundary | Are raw database and model payloads validated before becoming domain handoff values? |
| Failure | Can duplicate handoff creation, target rejection, expiry, and target-job creation failure be explained? |
| Observability | Can a trace id, operation event, and runbook query show pending and completed transfers? |
| Safety | Do authorization, sandboxing, approval, and redaction still apply after the handoff? |

## Exercises

1. Design an idempotency key for a handoff from a support-triage agent to a
   refund agent. Name the Postgres fields that prevent duplicate target work.
2. Write a Rust typestate sketch for `RequestedHandoff` and `AcceptedHandoff`.
   Add one negative test showing that an accepted handoff cannot be accepted
   again through the public API.
3. Extend the pending-handoffs runbook query for your own system. Include the
   target agent, oldest pending age, trace id, and one negative test for a
   malformed or unauthorized payload.

## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What is the difference between a handoff and an agent chat message?
- Explain: Why does a handoff transfer responsibility, not permission?
- Apply: Which failure does handoff idempotency prevent? Then describe how an accepted handoff creates or attaches exactly one target job.
- Evidence: Name the handoff row, idempotency key, permission scope, target-job evidence, and event that proves transfer.

## Summary

Multi-agent reliability is not created by letting models talk to each other. It is created by making responsibility transfer explicit, durable, typed, and reviewable.

- **Invariant:** a handoff transfers responsibility exactly once and records source, target, reason, payload, idempotency key, and decision.
- **Evidence:** handoff rows, target job links, rejection reasons, pending-handoff queries, trace ids, and audit events prove ownership.
- **Carry forward:** agents may collaborate, but the ledger decides who owns the next step.

## Changed Understanding

- **Before this chapter:** multi-agent coordination looked like agents messaging each other freely.
- **After this chapter:** handoffs are typed work transfers with ownership, permissions, evidence, and failure semantics.
- **Keep:** inspect the handoff record for source owner, target owner, permission, status, and replay rule.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
