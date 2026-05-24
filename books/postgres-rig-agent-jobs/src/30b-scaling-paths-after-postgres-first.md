# 30.5 Scaling Paths After Postgres-First

## What You Will Learn

This chapter teaches you to:

- explain how a reliable agent system should evolve after the Postgres-first design is no longer enough;
- inspect the pressure that justifies a queue, workflow engine, stream, worker pool, or observability platform;
- verify that added infrastructure preserves the same state machine and evidence contract.

The production evidence is a scaling decision record where each new component
has a reason, owner, invariant, migration path, rollback path, and operator
evidence.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** maturity gaps and operating limits are visible.
- **Adds:** evidence-preserving migration beyond the Postgres-first serious MVP.
- **Prepares:** a reader-owned production roadmap beyond this book.

## Production Failure

The team adds a queue, a workflow engine, and a metrics platform at the same
time because the system "needs to scale."

Afterward, work moves faster, but no one knows which component owns the state
machine, where retries are counted, or how to reconstruct one agent run.

- **What breaks:** scaling erased the evidence contract.
- **False fix:** replace visible Postgres state with new infrastructure before
  naming the strained invariant.
- **Design response:** add infrastructure only after measured pressure appears,
  then migrate state, ownership, traces, runbooks, rollback, and audit evidence
  without losing the original control model.

## Motivation

In production, scaling should follow a strained invariant, not a desire to add
infrastructure.

A queue, workflow engine, collector, console, stream, or worker pool changes who
owns evidence. That is the important point. Scaling is not only about moving
more work. It is about moving responsibility for state, retries, leases,
visibility, replay, side effects, and audit evidence.

Without a migration map, new components can hide the state machine the book
worked to make explicit. The system may become faster while becoming harder to
explain. Operators may lose the SQL query that proved a job was safe to replay.
Developers may lose the domain type that distinguished a retryable failure from
a permanent failure. Reviewers may lose the audit trail that proved a human
approved a risky action.

This chapter shows how to scale while keeping Postgres as the system of record
or reconciliation point until another component can prove it owns the same
invariant with at least the same clarity.

Do not add infrastructure to escape the state machine.

## Plain Version

Read this as the simple version:

**Simple rule:** Add new infrastructure only when the Postgres-first system shows
a real scaling limit. The limit should be visible in metrics, incidents, toil,
runbook pain, replay complexity, ownership boundaries, or SLO misses.

**Why it matters:** Queues, workflow engines, streaming platforms, and
orchestration tools should solve measured problems, not replace unclear design.
If the state machine is vague before migration, more infrastructure usually
makes it harder to find.

**What to watch:** Watch the exact pressure: throughput, isolation, replay
complexity, retention, latency, provider pressure, human-review queues, or
organizational ownership. Different pressure points justify different scaling
paths.

## What You Already Know

Start with these anchors:

- Durable work comes before model calls.
- Typed boundaries come before business logic.
- Leases come before concurrent workers.
- Idempotency comes before retry.
- Evaluation, approval, events, and restore drills remain required.

This chapter adds: scaling without losing the state machine. New infrastructure
may move an invariant, but it must not erase the evidence that made the system
operable.

## Focus Cue

Keep three things in view:

- **State:** old and new responsibility boundaries for state, actors, transitions, evidence, and rollback during scaling.
- **Move:** new infrastructure is introduced through baseline metrics, coexistence, trace correlation, runbook updates, and rollback criteria.
- **Proof:** Baseline metrics, old/new evidence maps, dual-run or coexistence plans, trace correlation, and rollback criteria are recorded.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a scaling migration plan that preserves the state machine while moving ownership to new infrastructure.
- **Why it matters:** extra infrastructure is useful only when it carries forward the invariant and evidence contract.
- **Done when:** the new queue, workflow engine, observability stack, or worker pool has coexistence, rollback, and evidence mapping.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** queue metrics, outbox backlog, version risk SQL, coexistence plan, and rollback plan.
- **State transition:** move a state-machine responsibility to new infrastructure only after preserving evidence.
- **Evidence path:** new infrastructure has ownership, compatibility, rollback, metrics, and evidence mapping.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which invariant moves to the new infrastructure, and how is rollback proven?
- **Evidence to inspect:** baseline metric, migration plan, coexistence rule, ownership map, evidence mapping, and rollback test.
- **Escalate if:** new infrastructure hides the state machine instead of preserving the evidence contract.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** The system approaches a scaling trigger. The trigger may be a hot
claim table, a growing outbox backlog, a workflow whose timers dominate custom
code, a runbook that operators repeat too often, or multiple teams needing
separate ownership.

**Action:** Move one state-machine responsibility only after mapping invariants
and evidence. Do not move dispatch, workflow execution, event distribution, and
operator control all at once unless the system is already at a maturity level
where those migrations can be independently proved.

**Persistence:** Persist baseline metrics, coexistence rules, migration notes,
rollback conditions, and ownership evidence. The migration itself is production
state.

**Check:** Verify the new infrastructure preserves the old proof before
replacing it. If Postgres previously proved idempotency, approval, replay safety,
and auditability, the new path must reconcile with those same facts.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** new infrastructure preserves the old state-machine invariant before replacing it.
- **Validation path:** inspect baseline metrics, coexistence plan, rollback test, ownership map, and evidence mapping.
- **Stop if:** the move hides state, weakens rollback, or breaks operator evidence.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, scaling should follow a strained invariant, not a desire to add infrastructure
rule: Add new infrastructure only when the Postgres-first system shows a real scaling limit
tiny example: old and new responsibility boundaries for state, actors, transitions, evidence, and rollback during scaling
artifact: a scaling migration plan that preserves the state machine while moving ownership to new infrastructure
proof: new infrastructure preserves the old state-machine invariant before replacing it
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Tiny Example

A support agent starts with one worker and a Postgres job table. After a year,
the business adds three high-volume job kinds:

```text
support_triage
refund_review
contract_analysis
```

The queue is now large enough that separate worker pools make sense. That does
not mean the system should immediately add every possible platform component.

A disciplined scaling plan might be:

```text
step 1: split workers by job kind
step 2: add per-kind admission and SLOs
step 3: move only high-volume dispatch to a dedicated queue
step 4: keep Postgres as the audit ledger and idempotency source
step 5: evaluate a workflow engine only for workflows whose timers and child
        workflow graph dominate the custom code
```

The key is that each step has a reason and a retained evidence surface.

Read the tiny case as:

```text
setup: growing job kinds strain one worker pool and one claim table
transition: scaling moves only the strained responsibility to a new component
evidence: decision record names reason, owner, invariant, migration path, retained ledger, and rollback
invariant: scaling may move evidence custody, but must not erase the state machine
```

## Mental Model

Scaling is a custody problem:

```text
Which invariant used to live in Postgres and Rust?
Which component will own it after the change?
What evidence proves the new owner behaves correctly?
How do old and new work coexist during rollout?
```

If those questions cannot be answered, the system is not scaling. It is
fragmenting.

## The Core Problem

The first production architecture is intentionally boring. It can still reach
its limit in several ways:

```text
throughput limit:
  the claim table becomes hot, or one worker pool cannot keep up

workflow-shape limit:
  the business needs many timers, child workflows, fan-out/fan-in, or durable
  replay semantics

team-boundary limit:
  multiple teams need independent deployment and ownership around different
  job families

observability limit:
  local logs and SQL diagnostics are no longer enough for fleet-wide
  investigation

governance limit:
  approvals, security reviews, and audit packets need richer review surfaces
```

Each limit points to a different evolution. A throughput problem may need
worker pools or a queue. A workflow-shape problem may need a workflow engine. A
governance problem may need an operations console, not Kafka.

## The Naive Solution

The naive scaling move is to add a tool by category:

```text
queue feels serious -> add a queue
workflow feels serious -> add a workflow engine
operations feel serious -> add a dashboard
scale feels serious -> add Kubernetes
```

This is infrastructure-first design. It often makes a system look mature while
preserving none of the evidence that operators need.

The failure pattern is common:

```text
Postgres had the idempotency key, but the new queue publishes duplicates.
Postgres had event history, but the workflow dashboard is the only place to
see transitions.
The worker had typed errors, but the new orchestrator treats every failure as
a retryable exception.
The old runbook used SQL evidence, but the new system has no equivalent query.
```

The result is more infrastructure and less accountability.

## The Production-Grade Concept

Use an evidence-preserving migration map.

If the Postgres job table is strained by dispatch throughput, a dedicated queue
may help. But Postgres should still record logical work, idempotency, state, and
receipts unless the queue provides equivalent durable evidence and operational
queryability.

If the Postgres state machine is strained by timers, child workflows,
fan-out/fan-in, and cancellation semantics, a workflow engine may help. But
workflow history must map back to product state, policy decisions, approvals,
tool calls, receipts, and replay rules. Workflow history is useful execution
evidence. It is not automatically product truth.

If the Postgres outbox is strained by event distribution and replay needs, Kafka
or another event stream may help. But product transactions should still create
typed facts first, and every consumer should record idempotent processing
evidence.

If a single worker is strained by different job families, split worker pools by
job kind, tenant, or risk class before changing the source of truth. Ownership,
leases, cancellation, graceful shutdown, and capacity controls should remain
explicit.

If local structured logs are no longer enough, add a central observability
backend. Trace id, job id, run id, tool call id, operation event, and audit event
should still correlate. Observability helps operators navigate evidence; it does
not replace durable evidence.

If SQL runbooks become too repetitive or risky, build an operations console. The
console should wrap reviewed queries, permission checks, audit events, and
explicit control actions. It should not become a hidden bypass around the ledger.

The map is intentionally conservative. It does not say "never add a queue" or
"never use Temporal." It says the new component must inherit the old production
contract.

## Evidence Before Migration

Before moving work out of the Postgres-first path, gather baseline evidence:

```text
current queue depth by job kind
oldest pending age
provider latency and rate-limit frequency
worker saturation
retry and dead-letter rates
approval wait time
restore and replay drill results
incident history
operator pain from runbooks
```

For example, the existing system can already expose queue pressure by job kind:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql}}
```

This query is not a final scaling solution. It is the evidence that tells you
whether scaling is needed and where.

## Migration Patterns

### Pattern 1: Split Worker Pools

Start here when the state model is still simple but throughput differs by job
kind.

```text
support_triage workers:
  high volume, low side-effect risk

refund_review workers:
  lower volume, approval-heavy

contract_analysis workers:
  long-running, provider-expensive
```

Keep the same Postgres ledger. Add admission limits, queue metrics, and SLOs by
job kind. This is the least disruptive scaling move because it changes worker
topology before changing the source of truth.

### Pattern 2: Add A Dedicated Dispatch Queue

Use this when dispatch throughput, not workflow semantics, is the bottleneck.
Postgres can remain the system of record:

```text
API writes logical job and idempotency key to Postgres
outbox publishes dispatch message
queue wakes workers
worker checks Postgres state before executing
worker writes terminal state, events, and receipts back to Postgres
```

The queue accelerates dispatch. It does not become the audit log unless it can
serve that role with comparable durability, retention, and queryability.

### Pattern 3: Adopt A Workflow Engine

Use this when workflow semantics dominate the custom code:

```text
many timers
complex child workflows
fan-out/fan-in
long sleeps
cross-service orchestration
deterministic replay needs
standardized cancellation across teams
```

Do not move to a workflow engine because the word "workflow" appears in a
diagram. Move when the engine owns a hard invariant better than your current
code.

The product ledger still matters. Agent runs, tool calls, approvals, side-effect
receipts, evaluation receipts, and audit events are product evidence. A workflow
history is not automatically a product audit trail.

### Pattern 4: Add Event Streaming

Use this when event distribution, retention, and replay become the hard part.

Good pressure looks like this:

```text
many independent consumers need the same product facts
analytics, search, billing, and operations need replayable event streams
outbox polling becomes the bottleneck
consumer teams need independent deployment
one event family has a clear partition key and schema policy
```

Do not add Kafka because events sound mature. Add it only after the source
facts are already typed, committed, and published through an outbox.

The safe migration path is:

```text
product transaction writes state and outbox row
publisher sends typed outbox event to Kafka
publisher records topic, partition, offset, and trace id
consumer parses typed event and records an idempotent receipt
runbook proves replay cannot duplicate a side effect
```

Kafka can distribute facts. It should not decide which facts are true.

### Pattern 5: Add Central Observability

Use this when local logs and SQL runbooks are insufficient for fleet-level
investigation.

The migration is not:

```text
delete events because traces exist
```

The migration is:

```text
keep durable events for audit and replay
propagate trace context across API, worker, provider, tools, and side effects
export traces, metrics, and logs to a collector
teach runbooks to move between trace view and database evidence
```

Observability is a navigation system. It does not replace the ledger.

### Pattern 6: Build An Operations Console

Use this when repeated SQL runbook operations need safer human control
surfaces.

The console should wrap reviewed operations:

```text
pause job kind
resume job kind
approve request
reject request
quarantine replay candidate
acknowledge incident
record postmortem action
```

Every console action should write an audit event. The console improves
ergonomics; it does not weaken authorization or remove the evidence trail.

## Formal Definition

For this chapter, the precise definition is:

```text
Scaling is the migration or duplication of responsibility to new infrastructure without losing the old state machine or evidence contract.
```

In the book's system model:

**State:** The state is the old and new responsibility boundary for state,
actors, transitions, evidence, and rollback during scaling.

**Actor:** The platform team migrates or duplicates responsibility while
operators compare old and new evidence.

**Transition:** The transition introduces new infrastructure through baseline
metrics, coexistence, trace correlation, runbook updates, and rollback criteria.

**Evidence:** The evidence is baseline metrics, old/new evidence maps, dual-run
or coexistence plans, trace correlation, operation events, and rollback criteria.

**Invariant:** Scaling adds capacity or capability without hiding the state
machine the reader already understands.

## What Can Fail

**Design smell:** Infrastructure is added before naming the invariant it will
own. The team knows the tool name, but it cannot say which control is strained.

**Production symptom:** The scaled system has more components but weaker
idempotency, auditability, replay safety, rollback, or operator evidence.

**Corrective invariant:** Each scaling step preserves or strengthens a named
control from the Postgres-first system.

**Evidence to inspect:** Inspect migration notes, baseline metrics, dual-write
or coexistence plans, release gates, runbook updates, trace id correlation,
operation events, and rollback criteria.

## Production Contract

Treat every scaling change as a migration of responsibility:

```text
state:
  Which durable fact moves or is duplicated?

actor:
  Which process, queue, workflow, console, or operator can change it?

transition:
  Which operation performs the change?

evidence:
  Which old and new artifacts prove the same fact?

invariant:
  What must remain true through rollout, crash, retry, replay, and rollback?
```

The scaling path is acceptable only when old and new evidence can be reconciled.

## Progressive Hardening Path

**Naive version:** Add a queue, workflow engine, stream, console, or
orchestration platform because scale feels scary. This is fast to announce, but
Rust and SQL boundaries may be bypassed, durable evidence may fragment, and
tests, traces, and runbooks may stop matching production behavior.

**Safer version:** Add one component only after baseline metrics show which
invariant is strained. The design keeps Postgres as the product ledger until the
replacement evidence is stronger and explicitly reviewed.

**Production version:** Migration notes map old state to new state, dual-run or
coexistence is tested, operation events and trace id propagation correlate both
paths, and runbooks explain rollback. The system scales without losing durable
evidence, idempotency, approval, replay safety, or operator trust.

## Testing Strategy

Unit tests should prove the migration policy. A scaling decision object should
reject a proposal that lacks a named invariant, owner, evidence source, rollback
criteria, or release gate. This can be ordinary Rust domain logic: the goal is
to make architecture review more explicit.

Persistence tests should prove that Postgres still records the product evidence
needed for incident review. If dispatch moves to a queue, the Postgres job row,
idempotency key, outbox event, state transition, and receipt should still agree.

Regression tests should encode the failures from this chapter: duplicate queue
messages do not create duplicate side effects, workflow retries do not bypass
approval, operations-console actions write audit events, and trace-only evidence
does not replace product events.

Postgres tests should continue to prove queue metrics, pending work, retries,
receipts, and audit evidence during coexistence. Rust tests should prove that
the old and new execution adapters classify success, transient failure,
permanent failure, cancellation, and approval-required outcomes in the same
domain language.

## Observability Strategy

Every scaling step should preserve the trace id across old and new components.
Use structured `tracing` fields for job id, run id, worker id, queue message id,
workflow id when present, tool call id, tenant, job kind, and release version.

Record an operation event whenever responsibility crosses a new component
boundary:

```text
postgres_job_dispatched_to_queue
queue_message_claimed
workflow_started_for_job
workflow_completed_for_job
ops_console_pause_requested
```

Each runbook query should state whether it reads the old path, the new path, or
both. During migration, runbooks should prefer reconciliation queries over
single-surface dashboards.

## Security and Safety Considerations

Scaling often expands the attack surface. New queues, workflow engines,
dashboards, collectors, and consoles introduce new authorization boundaries,
credential paths, network routes, and operational permissions.

Treat all cross-component payloads as untrusted until validated. Keep
sandboxing and approval controls outside the model even when orchestration
moves. Redact secrets from dispatch messages, workflow histories, traces, logs,
and console events. Do not place raw model output in long-retention workflow or
queue histories unless the retention and access-control implications are
intentional.

The safety principle is:

```text
scaling should reduce operational risk, not spread privileged data through more
places
```

## Operational Checklist

Use the checklist as an architecture review before each scaling change.

**State:** Which durable facts move, and where can old and new state be
reconciled?

**Boundary:** Which raw queue, workflow, console, or collector payloads are
converted back into typed domain values?

**Failure:** What happens on duplicate messages, partial migration, provider
timeout, worker crash, workflow cancellation, replay, and rollback?

**Observability:** Do trace id propagation, operation events, metrics, and
runbook query outputs correlate across old and new paths?

**Safety:** Are authorization, sandboxing, approval, redaction, audit, and
retention controls preserved or strengthened?

## Exercises

1. Pick one job kind and decide whether the next scaling step should be a
   worker pool, dedicated queue, workflow engine, operations console, or no
   infrastructure change. Name the idempotency invariant and the Postgres
   evidence that must survive.
2. Write a Rust `ScalingDecision` type with fields for invariant, owner,
   evidence source, rollback condition, and release gate. Add a negative test
   that rejects a decision without durable evidence.
3. Design a coexistence runbook query for a queue migration. It should compare
   Postgres job state with queue dispatch state and include a negative test for
   duplicate dispatch.

## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which invariants must survive any scaling move?
- Explain: Why new infrastructure must not erase the state machine.
- Apply: Evaluate a proposal to add a queue, workflow engine, stream, or worker pool.
- Evidence: Name the strained invariant, smallest adequate change, evidence that moves, evidence that stays in Postgres, and rollback path.

## Summary

Scaling a reliable agent system is not a tool shopping exercise. It is a
responsibility migration. The Postgres-first design teaches which invariant each
later component must preserve.

Invariant: adding a queue, workflow engine, collector, console, or worker pool
preserves or improves the evidence contract.

Evidence: migration maps, coexistence queries, rollback conditions, trace
propagation, duplicate-dispatch tests, and Postgres reconciliation prove the
move.

Carry forward: before adding infrastructure, name the invariant that is strained
and the evidence that must survive.

## Changed Understanding

**Before this chapter:** scaling looked like choosing bigger infrastructure
early.

**After this chapter:** scaling is justified when explicit Postgres-first limits
appear and each added system preserves the same state, identity, and evidence
contracts.

**Keep:** add infrastructure only with a migration map, coexistence proof,
rollback condition, and preserved evidence contract.

## Further Reading and Sources

- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) is relevant because scaling changes are really changes to durability, logs, transactions, and ownership of state.
- [Temporal documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) is relevant because workflow engines are the main optional evolution path when timers, replay, and orchestration dominate the system.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) is relevant because scaling decisions should be driven by SLOs, operational pain, incidents, toil, and explicit ownership.
- [OpenTelemetry documentation](./31-credible-resources-further-reading.md#reliability-and-operations) is relevant because central observability must preserve trace, metric, and log correlation across old and new components.
- [PostgreSQL transaction isolation documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) is relevant because coexistence and migration plans still depend on correct transaction boundaries in the product ledger.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) is relevant because architecture migrations should still expose clear Rust boundaries, typed states, and reviewable public contracts.
