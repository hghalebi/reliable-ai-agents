# 10. Capstone

## What You Will Learn

This chapter teaches you to:

- explain how to extend the system without weakening the reliability model;
- inspect whether a new job kind changes commands, states, tables, invariants, tests, and runbooks together;
- verify that extension work preserves the same durable and typed boundaries.

The production evidence is a complete feature path where new behavior arrives
with schema changes, Rust types, worker transitions, audit events, and tests.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the core failure modes are visible.
- **Adds:** the serious MVP extension pattern across Rust, SQL, tests, and runbooks.
- **Prepares:** the real Postgres store where the in-memory model meets persisted rows.

## Production Failure

A team adds a new "contact customer" job kind by copying the old job handler
and changing the prompt.

The new feature ships without a new state transition, idempotency key, approval
policy, audit event, or runbook query.

- **What breaks:** extension work bypassed the reliability model.
- **False fix:** accept feature velocity now and promise to add production
  controls later.
- **Design response:** add new capabilities only when commands, states, SQL,
  Rust types, tests, audit events, and operator evidence move together.

## Motivation

In production, extending an agent system is risky because every new feature can weaken an invariant. A new command may need a schema change, typed input, worker transition, runbook query, and test at the same time.

Without an extension discipline, the codebase grows by local patches and loses the reliability model. This chapter asks you to add capability while preserving the whole evidence chain.

## Plain Version

Read this as the simple version:

- **Simple rule:** The serious MVP is one Rust API, one Rust worker, one Postgres database, and Rig at the model boundary.
- **Why it matters:** The pieces matter because they prove the same work can be requested, persisted, executed, observed, and recovered.
- **What to watch:** Watch the full path from user request to job row, agent run, tool call, approval, receipt, and final status.

## What You Already Know

Start with these anchors:

- Part I built the durable agent job.
- The job has typed state, owned execution, model boundaries, visible events, and failure semantics.
- New behavior must move the same artifacts together.

This chapter adds: extension discipline. You will add capability without
weakening the architecture by changing commands, states, tables, invariants,
tests, and runbooks as one design move.

## Focus Cue

Keep three things in view:

- **State:** a proposed extension to the agent job system and all production artifacts it touches.
- **Move:** a feature is accepted only when its new command, state, table, invariant, test, and operator path agree.
- **Proof:** New commands, states, SQL, tests, and runbooks move together.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a feature-change packet that updates schema, types, worker behavior, tests, docs, and runbooks together.
- **Why it matters:** production systems decay when feature work changes only the happy-path code.
- **Done when:** one new capability has durable state, typed boundaries, worker transitions, tests, and operator evidence.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** the companion crate modules touched by one feature extension.
- **State transition:** change schema, domain types, worker transitions, tests, docs, and runbooks together.
- **Evidence path:** one feature has end-to-end evidence instead of a happy-path-only patch.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Did the feature change update every layer that owns the invariant?
- **Evidence to inspect:** schema diff, Rust type, worker transition, test, runbook query, and chapter note.
- **Escalate if:** the feature only changes happy-path code while persistence, operations, or safety lag behind.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a new feature is added to the companion system.
2. **Action:** update schema, domain type, worker path, test, runbook, and chapter evidence together.
3. **Persistence:** persist the feature evidence across code and documentation.
4. **Check:** verify no layer carries a stale version of the invariant.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** one feature update changes every layer that owns the invariant.
- **Validation path:** inspect schema, domain type, worker path, tests, docs, and runbook query together.
- **Stop if:** the feature works only in happy-path code while persistence or operations lag behind.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, extending an agent system is risky because every new feature can weaken an invariant
rule: The serious MVP is one Rust API, one Rust worker, one Postgres database, and Rig at the model boundary
tiny example: a proposed extension to the agent job system and all production artifacts it touches
artifact: a feature-change packet that updates schema, types, worker behavior, tests, docs, and runbooks together
proof: one feature update changes every layer that owns the invariant
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Worked Walkthrough

The capstone is where the earlier pieces stop being isolated topics.
A user request becomes a durable job.
The job becomes an agent run.
The run becomes typed steps, tool calls, approval decisions, side-effect receipts, and audit evidence.

Start with the request.
It crosses the HTTP or CLI boundary as raw input.
The application validates it, assigns typed identifiers, and stores the first durable state.
At this point, losing the process no longer loses the work.

The worker then claims the job.
The claim is a database transition, not a local promise.
The worker calls the Rig boundary only after it owns the attempt.
Rig helps the system ask the model for a structured recommendation.

The recommendation is still not business truth. It is only a contextual recommendation. We must be wary of **Automation Bias**, where we trust the model's output just because it's fast and looks confident. 

It is parsed, validated, authorized, and sometimes sent to a human approval gate. Only after those checks does the system execute a side effect.

The final result is not just an answer. It is an evidence chain. This evidence is what enables **Post-Deployment Monitoring** and **Iterative Improvement**. An operator can follow the trace id through the job, run, steps, tool call, approval, receipt, and audit event.
That evidence chain is the real capstone of the first production architecture.

## Tiny Example

Suppose the agent proposes:

```text
next_action: "roll back production deployment"
approval: required
```

The capstone extension should not execute that action from the worker that ran
the model. It should create an action proposal, store the policy version, wait
for approval, then execute the approved action through an idempotent
side-effect job.

That small example is the difference between:

```text
model says -> system acts
```

and:

```text
model proposes -> policy validates -> human approves -> side-effect worker acts
```

The second path has more steps because each step has a different owner. The
model owns the recommendation. Policy owns the risk decision. The human owns
approval. The side-effect worker owns execution. The database owns the evidence
that those boundaries were respected.

Read the tiny case as:

```text
setup: the model proposes a rollback that needs approval
transition: the worker creates approval state instead of executing the action
evidence: approval row, policy event, job state, and runbook query show waiting work
invariant: new behavior must add durable state, typed transitions, and operator evidence together
```

## Build These Commands

The command surface should tell the same story as the architecture.

Each command should expose a durable operation, not a hidden shortcut. Enqueue
creates work with identity. The worker advances leased work. `show-job` explains
current state. `show-events` explains history. `replay` must respect
idempotency, policy, and receipts.

If a command cannot explain which durable state it reads or writes, it is not yet
part of the production system. It may still be useful for local debugging, but it
should not become the path operators use during an incident.

```text
agent-jobs enqueue "Analyze failed deployment"
agent-jobs worker
agent-jobs show-job <job-id>
agent-jobs show-events <job-id>
agent-jobs replay <job-id>
```

Read these commands as a minimal operator interface.

> ### 🎓 The Professor's Corner
>
> **The Single Source of Truth: The Teacher and the Student**
>
> Think of the database as the **Teacher** and your command-line tool as the **Student**. The student might have notes (local logs), but if the student says something that contradicts the teacher's gradebook (the database), the student is wrong! 
> 
> The database is the only place where the "Final Answer" lives. If you want to know what's really happening, you always check the Gradebook!

They let a human ask the questions that matter when an agent job is slow,
duplicated, blocked, or unsafe: What work exists? Who is handling it? What
happened already? Can it be replayed safely? Which evidence says so?

## Add These States

New behavior usually requires new lifecycle language.

Do not hide that language in comments, prompt text, or dashboard labels. If the
system can wait for approval, reject an action, execute a side effect, or fail
after execution begins, those facts should appear in durable state and typed Rust
values.

```text
approval_required
approved
rejected
executing_action
action_succeeded
action_failed
```

Each state should answer one operational question.

`approval_required` tells the worker not to execute the side effect yet.
`approved` says the risk gate has passed. `rejected` says the proposal is
terminal. `executing_action` says a side-effect worker owns the dangerous part.
`action_succeeded` and `action_failed` tell replay and operators what happened
after execution began.

## Add These Tables

Tables are not only storage. They are contracts.

If a proposal, approval, or receipt matters to recovery, it deserves a place in
the schema. Storing all of this as a blob inside one job row may feel faster, but
it makes ownership, querying, retention, and audit harder. 

This is the **Normalization of Evidence**. By keeping proposals, approvals, and receipts in separate tables, we can perform cross-cutting audits—like "Show me all approvals granted by User X in the last 24 hours"—that are nearly impossible with JSON blobs.

```text
agent_action_proposals
agent_action_approvals
agent_action_receipts
```

The proposal table records what the agent suggested. The approval table records
who or what accepted or rejected the risk. The receipt table records whether the
external side effect happened. Keeping those facts separate makes the lifecycle
inspectable without asking a model or a worker process to remember.

## Required Invariants

The invariants are the actual capstone.

Commands, states, and tables are only useful if they enforce these rules. Read
each line as something that should be visible in a constructor, SQL constraint,
worker predicate, test, or runbook query.

```text
approval_required jobs cannot execute side effects
approved actions execute through an idempotency key
rejected actions are terminal
action receipts are append-only
replay never bypasses policy
```

These rules turn the extension from "the agent can now do more" into "the agent
can do more without escaping the control system." That is the difference between
adding capability and adding risk.

## Required Tests

The tests should explain the extension as executable judgment.

Each test protects one production claim. Duplicate approval tests protect
idempotency. Rejection tests protect terminality. Crash-before-receipt tests
protect replay safety. Policy-version tests protect auditability. Timeline tests
protect operator understanding.

> ### 🎓 The Professor's Corner
>
> **Design Friction: The Check Engine Light**
>
> If a test feels "hard" or "inconvenient" to write, don't just complain—listen to it! That's your system's **Check Engine Light** blinking. 
> 
> Inconvenience is often a signal that your architecture needs a clearer boundary or a simpler type. A good design should feel "smooth" to test. If you're fighting your code, it's time to pull over and look under the hood!

```text
duplicate action approval does not duplicate execution
rejected action cannot be executed
worker crash before receipt can replay idempotently
policy version is stored with the proposal
operator can view proposal, approval, and receipt timeline
```

If one of these tests feels inconvenient, that is a signal. The inconvenience is
showing where the architecture needs a clearer boundary or a stronger type.

## Extension Review

Review every proposed extension with one question:

```text
Which existing invariant becomes harder to preserve?
```

A replay command makes idempotency harder. A new side-effect worker makes
approval harder. A new provider route makes error classification harder. A new
tenant tier makes fairness harder. The correct response is not to avoid the
feature. The correct response is to make the affected invariant visible in the
schema, types, events, and tests before the feature ships.

## Final Architecture

At the end of Part I, the architecture is small but serious.

It is not a workflow platform. It is not a pile of prompts. It is a disciplined
Postgres-backed Rust system with Rig at the model boundary. Each component has a
limited job, and the limits are what make the system understandable.

```text
Postgres = durable memory
Rust = state transitions
Rig = reasoning
Events = traceability
Policy = safety
Approvals = control
Receipts = audit
```

The important word is not "stack." The important word is "boundary."

Postgres remembers facts that must survive. Rust names the states and legal
transitions. Rig helps the system obtain model reasoning without owning the
reliability layer. Events explain how state changed. Policy and approvals decide
what is allowed. Receipts prove what happened outside the database.

When those boundaries stay clear, the system can grow without becoming a black
box.

## Formal Definition

For this chapter, the precise definition is:

```text
An extension is production-safe only when it adds behavior without weakening existing state, evidence, idempotency, approval, or replay invariants.
```

In the book's system model:

- **State:** a proposed extension to the agent job system and all production artifacts it touches.
- **Actor:** the implementer and reviewer update Rust, SQL, tests, docs, and runbooks as one design change.
- **Transition:** a feature is accepted only when its new command, state, table, invariant, test, and operator path agree.
- **Evidence:** New commands, states, SQL, tests, and runbooks move together.
- **Invariant:** new capability does not weaken existing durability, idempotency, approval, replay, or evidence guarantees.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | A new job kind changes only one enum or command. |
| Production symptom | The feature compiles but lacks lifecycle, SQL, tests, or operations. |
| Corrective invariant | A new job kind moves schema, state, worker behavior, tests, and runbooks together. |
| Evidence to inspect | Review artifacts list every changed source, query, test, and operator command. |


## Production Contract

The capstone is complete only if new features preserve the old invariants:

```text
no side effect without a durable proposal
no proposal execution without policy and approval evidence
no replay path that bypasses idempotency
no new status without SQL, Rust enum, events, tests, and runbook coverage
no operator command that hides the event timeline
```

Adding an endpoint or command is not enough. The production contract is the
combination of schema, type, transition, event, test, and operational evidence.

This is the review standard for the rest of the book.

From here on, every production feature should be judged by the same question:
does the feature preserve the control loop? A feature that works only in the
happy path is not finished. A feature that cannot be inspected by an operator is
not finished. A feature that bypasses approval, idempotency, or replay rules is
not finished.

The capstone does not ask for a large system. It asks for a complete slice.
Complete means the slice can fail, recover, and explain itself.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | A new job kind changes only one enum or command. | Adding a job kind in one enum leaves schema, tests, operations, and review evidence behind. |
| Safer version | A new job kind moves schema, state, worker behavior, tests, and runbooks together. | A new behavior is treated as a cross-layer change that must preserve existing invariants. |
| Production version | Review artifacts list every changed source, query, test, and operator command. | The review artifact proves the new job kind has state, SQL, tests, runbooks, and operator evidence. |

Use the naive row when extension means code only. Use the safer row to carry the invariant across layers. Use the production row before adding a new real user workflow.

## Testing Strategy

Test a new job kind as a cross-layer change:

- **Unit or type test:** prove Rust compiles the new job kind through domain construction, worker routing, agent execution, failure classification, and observability label selection.
- **Persistence or boundary test:** prove Postgres schema, SQL queries, and runbook diagnostics can enqueue, claim, retry, complete, and inspect the new job kind.
- **Regression test:** preserve a review case where a new job kind lacks tests, SQL, or runbook evidence; the capstone checklist should block the change.

## Observability Strategy

Observe a new job kind across every layer it touches:

- Emit structured `tracing` fields for new job kind, command name, worker route, SQL transition, trace id, test artifact, and runbook diagnostic.
- Record an operation event when the new job kind is admitted, claimed, executed, retried, completed, or blocked by missing evidence.
- The runbook query should prove the extension appears in queue health, failure history, event timeline, and operator diagnostics.

## Security and Safety Considerations

A new job kind must bring its safety boundary with it:

- Treat new commands, payload fields, tool contracts, and SQL rows as untrusted until the full cross-layer path validates them.
- authorization, sandboxing, and approval should be reviewed for the new job kind before adding worker routing or side effects.
- Redact new payload examples where needed while preserving job kind, state, runbook query, and negative-test evidence.

## Operational Checklist

Use this checklist before relying on extending the system without weakening invariants in production:

- **State:** A new feature changes domain type, SQL state, worker transition, tests, and
  runbook evidence together.
- **Boundary:** New commands or tools enter through validated API/provider/database
  conversion layers.
- **Failure:** The extension has a negative path for duplicate input, invalid state,
  provider failure, and unsafe side effect.
- **Observability:** The new behavior appears in trace fields, operation events,
  metrics, and at least one runbook query.
- **Safety:** New side effects are blocked until idempotency, authorization, approval,
  sandboxing, and redaction are defined.

## Exercises

1. Write a negative test where the capstone feature is invoked twice and preserves one
   idempotency path with one receipt. Explain which idempotency key, receipt, or state
   transition prevents duplicate work.
2. Sketch the Postgres evidence: the schema row, migration, event, and runbook query
   required by the extension.
3. Define or refine the Rust type, enum, constructor, or typestate that represents the
   new domain type and transition function that make invalid extension states impossible
   or explicit. Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What artifacts must move together when adding a new state or job behavior?
- Explain: Why is adding only an enum variant an incomplete change?
- Apply: Extend the system with `approval_required` and list the required layers.
- Evidence: Name the SQL, Rust type, event, test, runbook query, and readiness evidence that must change.

## Summary

The production pattern is simple: make the job durable before it starts, make every transition explicit, make the side effect idempotent, and make the result observable.

- **Invariant:** a new capability changes domain types, SQL state, worker transitions, tests, and runbook evidence together.
- **Evidence:** the capstone feature has a row, type, state transition, negative test, trace field, and operator question.
- **Carry forward:** extend the system by preserving invariants, not by adding isolated features.

## Changed Understanding

- **Before this chapter:** the first ten chapters looked like separate mechanisms.
- **After this chapter:** they form one control loop: durable intake, typed domain model, worker execution, Rig boundary, and production evidence.
- **Keep:** follow one request from intake row through worker, Rig boundary, side effect, and audit evidence.

## Further Reading & Credible References

- **[Gunnar Hillert: Minimum Viable Architecture](https://www.infoq.com/articles/minimum-viable-architecture/)**. Explains why a "Serious MVP" must include the cross-layer artifacts (schema, types, tests) described in this capstone, rather than just code.
- **[Brandur Leach: The Serious MVP](https://brandur.org/serious-mvp)**. A foundational blog post advocating for building the "reliability shell" (Postgres, workers, idempotency) before the features, exactly as Part I of this book has demonstrated.
- **[Designing Data-Intensive Applications](https://dataintensive.net/)** (Martin Kleppmann). The "Capstone" text for modern software engineering, grounding the end-to-end evidence chain in the formal theory of distributed data systems.
