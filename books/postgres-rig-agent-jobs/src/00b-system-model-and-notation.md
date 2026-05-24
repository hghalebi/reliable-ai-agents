# System Model And Notation

## What You Will Learn

This chapter teaches you to:

- explain the book's small review grammar: state, actor, transition, evidence, and invariant;
- inspect a job, lease, event, provider call, policy decision, or receipt with that grammar;
- verify that every later reliability claim can be reduced to one clear state change.

The production evidence is a shared notation that makes jobs, leases, events,
provider boundaries, policy decisions, evaluation receipts, and side-effect
receipts reviewable.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the reader can tell a script from a production system.
- **Adds:** a shared grammar for state, actor, transition, evidence, and invariant.
- **Prepares:** design principles that reuse that grammar across the whole book.

## Motivation

In production, reliable systems fail when teams use different words for the same transition. A worker claim, retry decision, approval gate, and restore replay may all look unrelated even though they share the same shape.

Without shared notation, design reviews become subjective and incidents become arguments about vocabulary. This chapter gives the book one small language for naming state, actor, transition, evidence, and invariant.

The rule is simple: some state exists, some actor is allowed to change it, some evidence is recorded, and some invariant must still be true afterward.

## Plain Version

Read this as the simple version:

- **Simple rule:** Use one small grammar for every reliability claim: state, actor, transition, evidence, and invariant.
- **Why it matters:** It stops important words like reliable, durable, and safe from becoming vague slogans.
- **What to watch:** Check that every later design claim can name what changed, who changed it, what proof remains, and what must stay true.

## What You Already Know

Start with these anchors:

- A worker can pick up work.
- A database row can survive process death.
- An operator needs evidence when something breaks.

This chapter adds: one shared sentence for every later mechanism. Some state
exists, an actor changes it, evidence remains, and an invariant must still hold.

## Focus Cue

Keep three things in view:

- **State:** named production states, actors, transitions, evidence records, and invariants for every mechanism in the book.
- **Move:** a mechanism moves from informal idea to production concept only after it can be expressed in the shared notation.
- **Proof:** A reviewer can express each mechanism as state, actor, transition, evidence, and invariant.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a transition ledger that names state, actor, transition, evidence, and invariant for one agent job.
- **Why it matters:** every later control depends on this shared review language.
- **Done when:** a reviewer can point to the changed state, authorized actor, durable evidence, and surviving invariant for one run.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** front-matter notation, Appendix S, and the chapter-local transition grammar.
- **State transition:** turn one vague agent action into state -> actor -> transition -> evidence -> invariant.
- **Evidence path:** job rows, event timelines, policy decisions, evaluation receipts, and side-effect receipts.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Can you explain what changed, who was allowed to change it, and what evidence remains?
- **Evidence to inspect:** transition notation, event timeline, policy decision, and invariant statement.
- **Escalate if:** the chapter claim cannot be reduced to state, actor, transition, evidence, and invariant.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a reader sees an agent action that sounds vague.
2. **Action:** rewrite the action as state, actor, transition, evidence, and invariant.
3. **Persistence:** store the claim as a reviewable transition record or chapter note.
4. **Check:** ask whether the invariant would still be true after crash, retry, or audit.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** one transition can be written as state, actor, transition, evidence, and invariant.
- **Validation path:** compare the chapter notation against Appendix S and one later chapter formal definition.
- **Stop if:** the action still depends on vague words such as handled, processed, or decided without evidence.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, reliable systems fail when teams use different words for the same transition
rule: Use one small grammar for every reliability claim: state, actor, transition, evidence, and invariant
tiny example: named production states, actors, transitions, evidence records, and invariants for every mechanism in the book
artifact: a transition ledger that names state, actor, transition, evidence, and invariant for one agent job
proof: one transition can be written as state, actor, transition, evidence, and invariant
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Worked Walkthrough

Imagine a customer asks an agent to prepare a KYC case summary.
The user request is not yet a trusted business command.
At the boundary, the system gives the request an identity and records it as work.
That record becomes the first state in the diagram.

The worker then claims the job.
This is the move.
It is not a vague "agent is running" event.
It is a state transition from unowned work to leased work, with a worker id and a deadline.

Next, Rig calls the model and tools.
This is the intelligence boundary.
The model may propose a search, a classification, or a draft.
The model does not get to decide that the proposal is allowed.

The policy boundary checks the proposal.
If the tool is read-only and within tenant scope, the system may continue.
If the tool can change customer data, the system may require approval.

The proof is the part an operator can inspect later.
There should be a job row, a run row, a tool-call row, a policy decision, and an audit or operation event.
The diagram is useful only because each box leaves evidence behind.

This is why the book keeps asking for state, move, and proof.
State tells you what is true now.
Move tells you what changed.
Proof tells you how the next engineer can verify the story without trusting memory, logs alone, or the model's explanation.

## Intuition

Think of the system as a ledger of promises.

Each row says:

```text
this work exists
this worker currently owns it, or nobody does
this attempt is allowed, or it is not
this result is safe to use, or it is not ready yet
this side effect happened, or it did not
```

The model is not the source of truth. The worker is not the source of truth.
The process memory is not the source of truth. The durable state and the event
timeline are the source of truth.

The notation below keeps returning to one question:

```text
What changed, who was allowed to change it, and what evidence remains?
```

## Tiny Example

A small agent job can be written as:

```text
Job(
  id = incident-123,
  kind = incident_triage,
  state = pending,
  attempt = 0
)
```

A worker claim is a state transition:

```text
pending
  -- lease(worker-a, until = 10:05) -->
running
```

The same transition can be written with the evidence it must leave behind:

```text
precondition:
  state = pending and run_at <= now

action:
  set state = running
  set locked_by = worker-a
  set locked_until = 10:05
  increment attempt

evidence:
  event(job_picked, worker-a)

invariant:
  no other worker may finish this attempt before the lease expires
```

That is the core pattern of the book. A concept is complete only when the
reader can name its precondition, action, evidence, and invariant.

Read the tiny case as:

```text
setup: incident-123 exists as pending durable work
transition: worker-a claims the row and moves it to running
evidence: the job row and job_picked event name the owner and lease
invariant: no other worker may complete this attempt before the lease expires
```

## Core Objects

The book uses a small set of words again and again. They are not decoration.
They are the nouns that make production review possible.

An `AgentJob` is one durable unit of model-powered work. Before execution
begins, the system should be able to prove that the work exists. If the process
dies, the job row is the answer to the question, "What was supposed to happen?"

A `JobKind` is the class of work. Incident triage, KYC preparation, document
extraction, and sales follow-up are different kinds of jobs because they carry
different policies, retry rules, approval rules, and readiness expectations.
The kind answers the question, "Which controls apply here?"

A `JobState` is the lifecycle position of the job. Pending, running,
waiting-for-human, succeeded, failed, dead-lettered, and cancelled are not just
labels. They decide what may legally happen next.

A `WorkerId` names the process that currently owns an attempt. A worker does
not own work because it has a local variable in memory. It owns work because
the durable state says so.

A `Lease` is temporary ownership. It says, "This worker is handling the job,
but only until this time." The expiry is what lets another worker recover the
job after a crash without guessing.

An `Attempt` is one try. This matters because a job can fail, wait, retry, and
then succeed later. The attempt lets an operator attach the right failure,
provider response, retry decision, and receipt to the right execution try.

An `Event` is an immutable fact about the timeline. Events are how the system
remembers what happened after the current row has moved on.

A `ProviderBoundary` is the adapter where provider responses become domain
decisions. This is where strange API shapes, malformed model output, provider
timeouts, and usage metadata must be translated into the book's typed language.
Provider weirdness should not leak through the whole system.

A `PolicyDecision` is a recorded allow, deny, or approval-required result. It
separates model intention from system permission. The model may propose a tool
call. The policy decision says whether the system may continue.

An `EvaluationReceipt` proves that a prompt, model, tool, policy, or evaluator
version passed behavior checks before release. When behavior changes, this
receipt helps explain why the version was trusted.

A `SideEffectReceipt` proves that an external action happened once. It is the
record that lets replay avoid sending the same email, charging the same card,
or updating the same CRM field twice.

These terms are boundaries, not naming preferences. If a value answers one of
these production questions, it deserves a domain type, a schema field, a test,
or a runbook query.

Use this compact table as the quick reference after reading the prose:

| Term | Review question |
| --- | --- |
| `AgentJob` | Does the work exist before execution begins? |
| `JobKind` | Which controls apply to this work? |
| `Lease` | When may another worker recover this job? |
| `ProviderBoundary` | Did external weirdness leak into core logic? |
| `EvaluationReceipt` | Why was this behavior version released? |
| `SideEffectReceipt` | Can replay avoid duplicating the action? |

## Transition Notation

Most mechanisms in the book can be read as:

```text
StateBefore
  -- actor / action / evidence -->
StateAfter
```

For example:

```text
pending
  -- worker-a / pick_due_job / event(job_picked) -->
running

running
  -- worker-a / provider_timeout / event(retry_scheduled) -->
pending

running
  -- worker-a / permanent_failure / event(job_dead) -->
dead

awaiting_approval
  -- reviewer / approve / event(approval_granted) -->
ready_for_side_effect
```

The transition is valid only when its preconditions hold. A worker that does
not own the lease cannot mark the job succeeded. A policy gate that has not
recorded approval cannot authorize a risky side effect. A replay command that
does not inspect side-effect receipts cannot safely rerun old work.

## Evidence Chain

Production debugging follows the evidence chain:

```text
job row
  -> event timeline
  -> provider decision
  -> policy decision
  -> evaluation receipt
  -> side-effect receipt
  -> operator action
```

Not every job touches every step. A low-risk summarization job may not require
human approval or an external side effect. A billing, compliance, or incident
response job often does.

The important rule is that missing evidence must mean something. It should not
be ambiguous whether an approval was skipped, not required, failed to persist,
or is still waiting.

## Failure Notation

The book uses four failure outcomes:

```text
transient:
  retry later with backoff

permanent:
  stop and make the reason visible

exhausted:
  stop because retry budget is spent

cancelled:
  stop because an operator or policy ended the work
```

This classification is deliberately small. A production system may have more
subtypes, but every subtype should still map to one operational decision:
retry, stop, escalate, or cancel.

## Formal Definition

For this chapter, the precise definition is:

```text
A reliable agent system is a set of named states changed by authorized actors through explicit transitions that leave durable evidence and preserve invariants.
```

In the book's system model, `State` means the named production condition that
the system is currently in. A job is pending, running, waiting for approval,
succeeded, failed, cancelled, or dead-lettered. A tool call is proposed,
authorized, executed, rejected, or failed. A memory record is candidate,
accepted, expired, or quarantined.

`Actor` means the entity allowed to move the state. The actor may be a worker,
an operator, a reviewer, a policy engine, or a release gate. The important
point is not whether the actor is human or software. The important point is
that the actor is named and authorized.

`Transition` means the lawful move from one state to another. A transition is
not just "the worker handled it." It is a specific change, such as pending to
running, running to waiting-for-retry, proposed tool call to authorized tool
call, or waiting-for-human to approved.

`Evidence` means the durable proof that the transition occurred. It may be a
job row, operation event, audit event, policy decision, evaluation receipt,
side-effect receipt, or runbook output. Without evidence, the transition is
only a story.

`Invariant` means the rule that must remain true after the transition. For
example, only the worker that owns the lease may complete the attempt. A risky
tool call cannot run without permission. A replay cannot duplicate a side
effect that already has a receipt.

## What Can Fail

**Design smell:** the common smell is a diagram that names components but not state
transitions. It may show an API, a worker, a database, a model, and a tool, but
it does not say what changed, who was allowed to change it, or what evidence
survived the change.

**Production symptom:** the symptom appears during review or incident response. Reviewers
cannot tell whether the worker owned the job, whether the retry was allowed,
whether approval was required, whether the side effect happened, or whether the
new model version had passed evaluation.

**Corrective invariant:** the invariant is small and strict: every serious mechanism must be
expressible as state -> actor -> transition -> evidence -> invariant. If the
mechanism cannot be written in that form, it is probably hiding state, relying
on logs alone, or trusting the model too much.

**Evidence to inspect:** the evidence is concrete. Look for a job row, operation event,
audit event, policy decision, evaluation record, or side-effect receipt. If the
evidence does not exist, the design is not yet reviewable.


## Production Contract

Use this notation as a review checklist:

```text
Every important mechanism names a precondition.
Every risky transition records evidence.
Every actor has explicit authority.
Every retry decision has a failure class.
Every side effect has an idempotency key and receipt.
Every behavior release has an evaluation receipt.
Every runbook question points back to durable state.
```

If a chapter introduces a mechanism that cannot be expressed this way, the
mechanism is probably hiding state or relying on process memory.

## Progressive Hardening Path

The **Naive version** is a component diagram. It may be useful for orientation,
but it can hide the most important production question: who is allowed to
change state? A box labeled "worker" does not prove that the worker owns the
job. A box labeled "database" does not prove that a side effect cannot happen
twice.

The **Safer version** uses the shared grammar. Every serious mechanism is
written as state -> actor -> transition -> evidence -> invariant. This does
not make the system correct by itself, but it exposes missing ownership,
missing proof, and vague words early enough to fix them.

The **Production version** connects the grammar to artifacts. A reviewer can
point to the job row, operation event, receipt, policy decision, evaluation
record, or runbook query that proves the transition. Use this version whenever
a transition must survive crash, retry, deploy, restore, or audit.

## Testing Strategy

Test the notation by forcing every example to name its proof surface.

A **Unit or type test** should model a Rust transition with `StateBefore`,
`Actor`, `Transition`, and `StateAfter`. The test should fail when the
transition omits the invariant it must preserve. This turns the notation into
an executable explanation instead of a paragraph in the book.

A **Persistence or boundary test** should create a tiny Postgres event row for
the same transition and prove the row names the job, actor, evidence, and
timestamp needed for review. This checks that the design survives the database
boundary instead of staying in memory.

A **Regression test** should catch chapters, examples, or runbooks that name a
component but omit the state -> actor -> transition -> evidence -> invariant
chain. This protects the book itself from drifting back into vague diagrams.

## Observability Strategy

Make the notation observable by logging the review grammar itself. Use
structured `tracing` fields for the state name, actor, transition, trace id,
evidence artifact, and invariant being checked.

When a design review, runbook, or chapter example claims that a transition is
production-ready, the system should record an operation event or point to an
existing one. The runbook query should help a reviewer find the row, receipt,
or event that proves the transition. A diagram can explain the system, but it
should not be the only proof that the system behaved correctly.

## Security and Safety Considerations

Treat the notation itself as a safety boundary. Any state-transition claim that
lacks actor and evidence should be treated as untrusted design input until
reviewed.

This does not mean notation replaces authorization. When a transition can
reach a tool, tenant, credential, customer record, or side effect,
authorization, sandboxing, approval, and receipt evidence still need concrete
owners.

Redact sensitive payloads from examples while preserving the state, actor,
transition, evidence, and invariant needed for audit. A safe example hides the
secret, not the control structure.

## Operational Checklist

Use this checklist before relying on the state/actor/transition/evidence/invariant notation in production:

`State` is ready when each important agent transition names the state before,
state after, actor, evidence, and invariant it must preserve.

`Boundary` is ready when raw user, model, provider, and database facts are
translated into the shared notation before design review.

`Failure` is ready when a review can point to the missing evidence whenever a
transition is ambiguous or unsafe to replay.

`Observability` is ready when runbook output and traces use the same state and
evidence names as the chapter notation.

`Safety` is ready when a risky transition cannot proceed on notation alone.
Authorization, approval, and receipt evidence still have to exist.

## Exercises

1. Write a negative test where a transition changes job state without naming the actor or evidence. Explain which idempotency key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: one `operation_events` row that records `state_before`, `state_after`, actor, trace id, and invariant name.
3. Define or refine the Rust type, enum, constructor, or typestate that represents `StateBefore`, `StateAfter`, `Actor`, `EvidenceRef`, and `InvariantName` as separate domain values. Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What are the five words in the review grammar?
- Explain: Why does evidence sit between a transition and an invariant?
- Apply: Take "retry a failed job" and name the state, actor, transition, evidence, and invariant.
- Evidence: Point to the row or event that would prove the retry was allowed.

## Summary

The notation is a small language for production reasoning: state changes, an actor is allowed to change it, evidence is recorded, and an invariant survives the transition.

- **Invariant:** every important agent transition can be reviewed as state, actor, transition, evidence, and invariant.
- **Evidence:** job rows, operation events, policy decisions, evaluation receipts, and side-effect receipts use the same vocabulary.
- **Carry forward:** when a chapter feels complex, translate it back into this chain before adding more machinery.

## Changed Understanding

- **Before this chapter:** an agent looked like one loop that decides and acts.
- **After this chapter:** an agent system is a set of typed actors, states, transitions, evidence records, and invariants.
- **Keep:** prove each agent concept with a state, actor, transition, evidence record, and invariant.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
