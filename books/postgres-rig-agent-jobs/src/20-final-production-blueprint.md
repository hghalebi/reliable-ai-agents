# 20. Final Production Blueprint

## What You Will Learn

This chapter teaches you to:

- explain the complete production shape of the Postgres-first reliable agent system;
- inspect how API, worker, Rig, Postgres, policy, observability, evaluation, and operations connect;
- verify that every boundary has an owner, invariant, and evidence source.

The production evidence is a blueprint where each subsystem has a typed
contract, durable state, trace context, test coverage, and operator query.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** long-running operation requires versioned evidence.
- **Adds:** the complete serious MVP architecture and failure ownership map.
- **Prepares:** durable handoffs and multi-agent coordination.

## Production Failure

The team has all the ingredients: API, worker, Postgres, Rig, approval, traces,
and tests.

During an incident, no one can say which boundary owns the failed transition.

- **What breaks:** components exist, but responsibility is not allocated. I call this **"Boundary Blame"**—is it a bad prompt, a bad model, or a bad tool? 
- **False fix:** draw a bigger architecture diagram with more arrows.
- **Design response:** make the blueprint a failure-ownership map where every
  boundary has state, transition, invariant, evidence, and operator question. This is what enables **Root Cause Attribution (RCA)** in AI systems.

## Motivation

In production, the final system is not one binary or one framework. It is a set of boundaries that keep promises under failure, change, and audit.

Without a blueprint, the reader may understand individual mechanisms without seeing where each responsibility lives. This chapter composes the API, worker, Postgres ledger, Rig boundary, policy, evaluation, observability, and operations into one production shape.

## Plain Version

Read this as the simple version:

- **Simple rule:** The blueprint is the smallest complete production shape, not the largest possible architecture.
- **Why it matters:** A serious MVP should prove durable state, typed tools, workers, retries, approvals, observability, and operations before scaling out.
- **What to watch:** Watch whether every component has a clear owner, state boundary, failure path, and evidence surface.

## What You Already Know

Start with these anchors:

- Part II introduced durable Postgres state, idempotency, leases, retries, observability, approval, tests, deployment, and versioning.
- Each mechanism protects one production invariant.
- A production system must show how the mechanisms fit together.

This chapter adds: the full blueprint. You will place each subsystem at its
boundary and name its owner, evidence source, test, and operator question.

## Focus Cue

Keep three things in view:

- **State:** the complete set of system boundaries and the production promises each boundary owns.
- **Move:** a boundary is accepted only when it owns a failure contract and evidence surface.
- **Proof:** API, Postgres, worker, Rig boundary, policy, approval, side effects, and operations each have one clear responsibility.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** the serious MVP architecture: one Rust API, one Rust worker, Postgres, Rig, traces, approvals, and runbooks.
- **Why it matters:** the reader needs a deployable whole system before studying advanced scaling paths.
- **Done when:** a request moves from intake to scheduling to agent execution to policy to audit evidence without hidden state.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** API binary, worker binary, Postgres schema, Rig runner, approval gate, audit events, and runbooks.
- **State transition:** assemble the serious MVP from the earlier controls.
- **Evidence path:** one request can be traced through intake, scheduling, execution, policy, and evidence.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Can one real request be traced through the complete serious MVP?
- **Evidence to inspect:** API intake, Postgres job, worker event, Rig result, policy decision, audit event, and runbook query.
- **Escalate if:** a production promise only exists in a diagram, not in the API, worker, database, and evidence trail.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a real request enters the serious MVP.
2. **Action:** move it through API intake, Postgres scheduling, worker execution, Rig, policy, and audit.
3. **Persistence:** persist rows and events at each boundary.
4. **Check:** verify the full path has no hidden state or untyped authority.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** one real request traverses the serious MVP with no hidden state.
- **Validation path:** trace API intake, Postgres job, worker event, Rig result, policy decision, audit event, and runbook query.
- **Stop if:** any production promise exists only in diagrams or prose.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, the final system is not one binary or one framework
rule: The blueprint is the smallest complete production shape, not the largest possible architecture
tiny example: the complete set of system boundaries and the production promises each boundary owns
artifact: the serious MVP architecture: one Rust API, one Rust worker, Postgres, Rig, traces, approvals, and runbooks
proof: one real request traverses the serious MVP with no hidden state
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Worked Walkthrough

Imagine a customer asks the agent to analyze a failed deployment and recommend a
rollback. In a demo, the system might pass the request straight to the model and
show the answer. In the serious MVP, the request enters through a controlled
path.

The API receives raw HTTP data. At that moment, the system knows almost
nothing. The request may be malformed. The idempotency key may be missing. The
tenant may not have permission to run this job kind. The API boundary therefore
does not "run the agent." It validates the request, converts raw fields into
domain types, derives or checks idempotency, and inserts durable work.

Postgres then becomes the memory of intent. The pending job row says that this
work exists, who requested it, which job kind it belongs to, which versions are
in play, and which idempotency key identifies the logical request. If the API
process dies after insertion, the work is still visible. If the same request
arrives again, the idempotency record prevents a second logical job from being
created.

The worker owns execution, but only after it leases the job. That lease is
temporary authority. The worker records that it started work, calls the Rig
boundary, and receives either validated agent output or a typed failure. The
worker does not let raw provider output mutate business state. The provider
boundary turns uncertain model behavior into a decision the rest of the system
can understand.

Policy then decides whether the recommendation can act. A rollback
recommendation is risky, so the system records a proposal and requires approval.
The model may explain why rollback looks useful, but it cannot grant itself
permission. If a human approves, the approval record names the actor, the
proposal, the versioned policy, and the reason. Only then may the side-effect
path execute.

The side-effect path uses its own identity and receipt. If the process crashes
after calling an external system, the system must be able to answer whether the
external action happened. That is why the receipt matters. Without it, replay is
guesswork.

Finally, operations tie the path together. A trace id follows the request. Job
events explain state transitions. Audit events preserve business evidence.
Metrics show fleet health. Runbooks answer what is stuck, what failed, what is
waiting for approval, and what can be replayed safely.

The blueprint is this chain of responsibility. Each boundary accepts one kind of
input, makes one kind of decision, leaves one kind of evidence, and hands work to
the next boundary only after the required invariant holds.

## Why This Is The Serious MVP

This architecture is intentionally small. It has one API service, one worker
process, one Postgres database, and Rig at the agent boundary. It does not begin
with a queue cluster, workflow engine, event-streaming platform, service mesh, or
multi-region deployment.

This is the **Principle of Least Infrastructure**. Postgres is your **Linearizable Log**. Before you move to a specialized orchestrator, you must prove you can operate the state machine on a simple relational store. 

> ### 🎓 The Professor's Corner
>
> **The Single Box Limit: Postgres First**
>
> Think of your system like a new business. You don't start by renting a whole skyscraper (Kafka)! You start with one good shop (Postgres). 
> 
> You should only move to a bigger "Distributed System" when your single shop can no longer keep its promises. Postgres is incredibly strong—it can handle a lot of work before you ever need to buy more complexity.

Small does not mean casual. The serious MVP is serious because the important
failure questions already have answers:

```text
Did the request exist before the model ran?
Can duplicate intake create duplicate work?
Who owns the job right now?
What model and prompt version produced the output?
Was the tool call authorized?
Did a human approve the risky action?
Did the side effect happen once?
Can an operator reconstruct the timeline?
```

If the answer to those questions is "yes, here is the row, event, receipt, or
test," the system is already much closer to production than a larger system that
hides those facts behind more infrastructure.

The blueprint is also a restraint. It tells the team what not to add yet. Do not
add Kafka because agents feel important. Do not add Temporal because the word
"workflow" appears in a design doc. Do not add Kubernetes because the system has
workers. Add infrastructure when a named invariant outgrows the current owner.
Until then, make the Postgres-backed state machine explicit and operable.

## Tiny Example

Trace one risky job through the final system:

```text
webhook requests deployment analysis
  -> API derives idempotency key
  -> Postgres stores pending job
  -> worker leases job
  -> Rig returns rollback recommendation
  -> policy marks action approval_required
  -> operator approves
  -> side-effect worker executes with idempotency key
  -> receipt is stored
```

Every arrow either validates input, persists state, preserves ownership, gates
risk, or records evidence. If an arrow does none of those things, it probably
does not belong in the production path.

This is the blueprint's main discipline: each component exists because it owns
a failure boundary. The design is not a diagram of libraries. It is a map of
where promises are kept.

Read the tiny case as:

```text
setup: one risky webhook becomes one production agent job
transition: the request crosses admission, worker, Rig, policy, side-effect, and audit boundaries
evidence: each boundary leaves a row, event, trace field, test, or receipt
invariant: the whole system is reliable only when every boundary owns a clear promise
```

## Blueprint

```text
API / trigger layer
  validates request
  derives idempotency key
  inserts agent job

Postgres ledger
  stores current job state
  stores scheduled_jobs and background_jobs
  stores workflow state and retry state
  stores agent runs, handoffs, steps, tool calls, approvals, receipts, and evaluations
  stores scoped memory records with source, confidence, retention, and content policy
  stores append-only events
  enforces state constraints
  stores prompt/model/tool/policy versions

Rust worker
  recovers expired jobs
  picks due work with SKIP LOCKED
  leases and heartbeats
  calls AgentRunner
  classifies failure
  succeeds, retries, cancels, or dead-letters

Rig runner
  talks to DeepSeek or another provider
  returns typed AgentResult
  maps provider errors into retry decisions

Policy and approval
  keeps risky side effects out of model text
  requires approval where needed

Operations
  metrics
  traces
  SLOs and error budgets
  runbooks
  replay and cancellation
  migration discipline
  incident response and postmortems
```

## Reading The Blueprint

Read this diagram from top to bottom as a chain of trust.

This is the **Hierarchy of Trust**. The API is the **Gatekeeper**, Postgres is the **Memory**, and the Worker is the **Muscle**. By separating these, you prevent a single compromised process from owning the whole system. This is essentially a **Shared-Nothing Architecture** implemented over a shared ledger.

> ### 🎓 The Professor's Corner
>
> **The Blueprint as a Map: Not a Photo**
>
> A map doesn't show you every single tree in the forest; it shows you the roads so you don't get lost! Your blueprint is your map for production. 
> 
> It tells you where the "Town Squares" (the database) and the "Highways" (the worker loops) are. If you follow the map, you'll always know where the evidence is, even in a storm! It's a "Tool for Travel" rather than a "Test to Pass."

The API is trusted to admit work, not to complete work. Postgres is trusted to
remember state, not to interpret model output. The worker is trusted to move
jobs through legal transitions, not to invent business permission. Rig is
trusted to connect to the model provider, not to decide whether a tool action is
safe. Policy and approval are trusted to grant authority, not to hide execution
details. Operations are trusted to explain and control the system under stress,
not to become the only place where truth exists.

This reading keeps the architecture honest. If a component starts owning too
many responsibilities, the evidence path becomes hard to review. If no component
owns a responsibility, incidents become arguments. The blueprint is complete
only when a reviewer can point to each boundary and say: this is what it accepts,
this is what it may change, this is what it records, and this is the invariant it
protects.

## HTTP Admission Boundary

The API service is not where the agent thinks. It is where raw user or webhook
input becomes durable, typed work. That makes it a production boundary, not
just a transport detail.

The companion Axum module is optional so the beginner path stays small, but
the production MVP has a real HTTP shape:

```text
POST /agent-jobs
Idempotency-Key: incident:payments-api:inc-7841

raw JSON body
  -> CreateAgentJobCommand
  -> AgentJob
  -> store.enqueue(...)
```

The request body may contain strings because HTTP is the raw edge. Those
strings do not cross into the worker. The API immediately converts them into
`JobKind`, `AgentInstruction`, `IdempotencyKey`, `MaxAttempts`, and version
newtypes:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/api.rs:api_admission_command}}
```

The route keeps the same invariant as the SQL ledger: no idempotency key, no
durable work; invalid domain values fail before enqueue:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/api.rs:api_router}}
```

This is the serious MVP boundary: the API admits intent, Postgres remembers it,
and workers execute it later. The API does not call the model directly, hide
retry loops, or let raw JSON become application architecture.

## Multi-Agent Handoffs

Multi-agent systems fail when "handoff" means one model writes a note and
another model hopefully understands it later. In production, a handoff is a
durable transfer of responsibility:

```text
source agent run
  -> handoff request with reason and typed payload
  -> target agent accepts or rejects
  -> accepted handoff creates a target job
  -> operators can inspect pending handoffs by target agent
```

The companion code models this as state, not conversation:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/handoff.rs:handoff_typestate}}
```

It prevents **Dangling Handoffs** where a request is lost between two agents. Most multi-agent frameworks treat handoffs as "Sending a message." But in production, a handoff is a **Commitment**. If Agent A hands off to Agent B, the system must record that the **Responsibility shifted**.

The database row boundary validates that the source and target agents are
different, the payload is a JSON object, the accepted state has target-job
evidence, and rejected/expired/cancelled states have decision evidence:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/handoff.rs:handoff_row_boundary}}
```

This keeps specialist agents useful without making responsibility disappear.
The triage agent can hand work to the refund agent, compliance agent, or
incident agent, but the system records who handed off, why, with which payload,
and what happened next.

The next chapter expands this handoff boundary before the worked scenario uses
it in a complete incident path.

## Boundary Ownership

Read the blueprint from failure to owner:

```text
duplicate request:
  API Idempotency-Key validation and Postgres idempotency

crashed worker:
  lease expiry and recovery query

provider timeout:
  Rig boundary and retry disposition

risky recommendation:
  policy and approval state

operator investigation:
  event timeline, run state, handoffs, tool calls, receipts, metrics, traces, and runbooks

old job after deploy:
  stored prompt, model, policy, schema, and worker versions
```

If an owner cannot be named, the design is incomplete. If two owners can mutate
the same fact without a shared invariant, the design is unsafe.

## Production Checklist

Before shipping:

```text
schema has constraints and indexes
API rejects missing idempotency keys and invalid domain values
job rows carry replay/version metadata
enqueue is idempotent
workers use leases
expired leases are recovered
long work can heartbeat
permanent failures do not retry forever
dead jobs are visible
events explain every transition
provider errors are classified
secrets are not logged
risky action requires policy and approval
agent handoffs are durable and accepted before target work starts
tests cover state transitions
runbook covers stuck, dead, replay, cancel
health, readiness, and metrics expose process and queue health separately
error budgets define when to slow releases
deploys do not delete running state
```

## Formal Definition

For this chapter, the precise definition is:

```text
A production blueprint is the allocation of every failure boundary to a component that owns the state, transition, evidence, and invariant.
```

In the book's system model:

- **State:** the complete set of system boundaries and the production promises each boundary owns.
- **Actor:** architects and reviewers assign durable state, execution, policy, side effects, and operations to concrete components.
- **Transition:** a boundary is accepted only when it owns a failure contract and evidence surface.
- **Evidence:** API, Postgres, worker, Rig boundary, policy, approval, side effects, and operations each have one clear responsibility.
- **Invariant:** every important promise has one accountable owner and one inspectable proof path.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | The architecture diagram has arrows but no ownership contracts. |
| Production symptom | During an incident, every layer can blame another layer. |
| Corrective invariant | Each boundary owns specific state, failures, and evidence. |
| Evidence to inspect | Blueprint review maps API intake, store, worker, provider, handoff, policy, side effect, and operator surfaces. |


## Production Contract

The full system should be auditable from one question:

```text
For this logical request, what happened, who or what decided it, which version
made that decision, and what evidence proves the side effect was safe?
```

The blueprint must preserve:

```text
durability before execution
typed boundaries before business logic
idempotency before retry
policy before side effect
events before incident review
eval evidence before behavior release
restore drills before long-horizon claims
```

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | The architecture diagram has arrows but no ownership contracts. | An architecture diagram with arrows can hide ownership gaps and unobservable transitions. |
| Safer version | Each boundary owns specific state, failures, and evidence. | Each boundary owns specific state, authority, failure modes, and evidence responsibilities. |
| Production version | Blueprint review maps API intake, store, worker, provider, handoff, policy, side effect, and operator surfaces. | The blueprint lets reviewers trace intake, worker execution, Rig output, approval, side effects, and operations end to end. |

Use the naive row when the diagram looks complete. Use the safer row to assign ownership. Use the production row before the architecture becomes the reference implementation.

## Testing Strategy

Test the blueprint as an ownership map:

- **Unit or type test:** prove Rust boundary traits and domain types prevent API intake, worker state, Rig output, policy decisions, and side effects from collapsing together.
- **Persistence or boundary test:** prove Postgres tables and queries cover every blueprint responsibility: intake, runs, steps, tools, approvals, audit, operations, and recovery.
- **Regression test:** remove one evidence link from the blueprint path and verify the traceability or readiness gate detects the missing owner.

## Observability Strategy

Observe the blueprint by tracing responsibility handoffs:

- Emit structured `tracing` fields for boundary name, owner component, job id, run id, tool call id, approval id, side-effect receipt, and trace id.
- Record an operation event whenever responsibility crosses API, Postgres, worker, Rig boundary, policy, approval, side effect, or operator surface.
- The runbook query should prove every arrow in the blueprint corresponds to a durable state transition and evidence owner.

## Security and Safety Considerations

The blueprint should show where authority is granted and denied:

- Treat every boundary arrow as untrusted until it names validation, ownership, evidence, and the component allowed to make the transition.
- authorization, sandboxing, and approval should appear as first-class control surfaces, not hidden inside the model or worker.
- Redact payload details in diagrams while preserving boundary names, state owners, trace points, and audit evidence.

## Operational Checklist

Use this checklist before relying on the complete reliable-agent architecture in production:

- **State:** API, worker, Postgres ledger, Rig boundary, policy, evaluation,
  observability, and runbooks each own explicit evidence.
- **Boundary:** No layer passes raw model output, database rows, provider DTOs, or user
  input into another layer without conversion.
- **Failure:** The blueprint shows where each failure is retried, blocked, escalated,
  compensated, or made terminal.
- **Observability:** A single job can be reconstructed from intake, run, step, tool,
  policy, approval, receipt, metric, and incident evidence.
- **Safety:** The architecture keeps permissions, approvals, sandboxing, idempotency,
  redaction, and audit outside prompt text.

## Exercises

1. Write a negative test where a risky request passes through the blueprint and cannot
   execute without typed state, policy, approval, and idempotency evidence. Explain
   which idempotency key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: the full evidence chain from agent_jobs through
   agent_runs, tool_calls, approvals, events, and receipts.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   module boundaries that separate domain, store, worker, provider, policy, evaluation,
   and API types. Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What are the main subsystems in the final blueprint?
- Explain: Which invariant each subsystem owns.
- Apply: Trace one logical request through durability, idempotency, ownership, provider classification, approval, observability, and replay safety.
- Evidence: Name the row, event, trace field, test, receipt, and runbook query for each promise.

## Summary

An AI agent in production is a durable state machine wrapped around an unreliable reasoning engine. The blueprint connects API, worker, Postgres, Rig, policy, evaluation, observability, and operations.

- **Invariant:** every boundary has a responsibility, a failure contract, and durable evidence.
- **Evidence:** intake rows, run and step records, tool calls, approvals, receipts, traces, metrics, eval results, and runbooks reconstruct one request end to end.
- **Carry forward:** build the serious MVP before buying complexity.

## Changed Understanding

- **Before this chapter:** production architecture looked like a list of components.
- **After this chapter:** the blueprint is a set of contracts between API, database, worker, Rig, policy, observability, and operators.
- **Keep:** review the blueprint by contracts: API, Postgres, worker, Rig, policy, observability, and operator surfaces.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
