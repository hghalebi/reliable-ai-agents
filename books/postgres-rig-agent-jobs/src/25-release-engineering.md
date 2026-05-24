# 25. Release Engineering For Agents

## What You Will Learn

This chapter teaches you to:

- explain how code, schema, prompt, policy, and model changes roll out safely;
- inspect version metadata, migration compatibility, canaries, evaluation gates, rollback paths, and release receipts;
- verify that a release can coexist with old jobs already in flight.

The production evidence is a release record that controls version skew,
forward-compatible schema changes, prompt/model evaluation, and deployment
receipts.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** incidents reveal what changes must become safer.
- **Adds:** versioned release gates for code, schema, prompt, model, tool, and policy changes.
- **Prepares:** toil, automation, and ownership after repeated operational work appears.

## Production Failure

A Rust deploy passes tests, but the prompt version changes the agent's tool
selection behavior.

Old jobs still hold the previous schema shape, the canary has no behavior eval,
and rollback only covers code.

**What breaks:** release safety covered binaries but not agent behavior.

The deploy pipeline asked whether the Rust binary built and passed tests. It did
not ask whether the new prompt changed tool selection, whether old jobs could
still be decoded, whether the new model route behaved like the old one, or
whether rollback could restore the full behavior packet.

**False fix:** treat prompt and model changes as content edits outside the
release process.

This is how agent systems drift. A code release goes through review, tests, and
rollback planning. A prompt edit goes through a document change. A model route
changes through configuration. A policy changes in a dashboard. Each change may
look small alone, but together they can create a new production behavior that no
release gate ever evaluated.

**Design response:** release code, schema, prompt, model, tool contracts,
policies, evals, canaries, and rollback evidence through one versioned gate.

The release unit is not only the binary. The release unit is the behavior packet:
code version, schema version, prompt version, model route, tool contract,
policy version, evaluator, dataset, canary evidence, and rollback plan.

## Motivation

In production, an agent release can change code, schema, prompt, model, policy, tool contract, or evaluation behavior. Any one of those can break old work.

Without release gates, a green deploy can still produce unsafe behavior. This chapter makes compatibility, evaluation, canaries, error budgets, and rollback part of one release decision.

## Plain Version

Read this as the simple version:

**Simple rule:** Release agent behavior through gates that check code, schema,
prompts, policies, and evaluation evidence together.

An agent release is not just a deploy. It is a controlled change to what the
system may read, decide, call, approve, and persist.

**Why it matters:** A model or prompt change can break production even when Rust
tests still pass.

Rust tests may prove the worker still compiles and the state machine still
rejects impossible transitions. They do not automatically prove the model still
chooses the right tool, the prompt still refuses unsafe work, or old jobs still
survive a schema transition.

**What to watch:** Watch compatibility checks, evaluation results, canary
evidence, rollback path, and human approval for high-risk changes.

## What You Already Know

Start with these anchors:

- Incidents often reveal a contract changed without enough protection.
- Agent systems have many release surfaces: code, schema, prompt, model, tool, policy, and approval.
- Old jobs may depend on old versions.

This chapter adds: release engineering for agent behavior and infrastructure.
You will use compatibility checks, canaries, evaluation gates, migrations,
rollback paths, and receipts.

## Focus Cue

Keep three things in view:

- **State:** versioned code, schema, prompt, model, tool, policy, canary, evaluation, compatibility, and rollback evidence.
- **Move:** a change reaches production only after old work compatibility and behavior evidence are checked.
- **Proof:** Expand-contract migrations, canaries, version fields, release gates, evaluation receipts, and rollback evidence exist.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a release gate that combines evaluation, SLO, compatibility, migration, and approval evidence.
- **Why it matters:** prompt, model, schema, and worker releases can break old work unless evidence blocks unsafe promotion.
- **Done when:** a change cannot promote unless behavior, compatibility, error budget, and human-risk checks are green.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/release_gate.rs`, compatibility checks, eval receipts, migration evidence, and approval records.
- **State transition:** block unsafe behavior, schema, policy, or worker changes before promotion.
- **Evidence path:** release decisions include eval, SLO, compatibility, approval, and rollback evidence.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** What evidence blocks or allows this release to affect real work?
- **Evidence to inspect:** evaluation receipt, SLO budget, compatibility query, migration status, human approval, and rollback plan.
- **Escalate if:** a prompt, model, schema, policy, or worker change ships without gate evidence.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** a change wants to reach production.

The change may be a Rust binary, SQL migration, prompt, model route, tool schema,
policy rule, evaluator, or dataset. Treat all of them as release surfaces because
all of them can change real agent behavior.

**Action:** evaluate behavior, SLO, compatibility, migration, approval, and
rollback evidence.

The gate should ask several questions at once. Did the eval pass for the exact
prompt and model versions? Is the error budget healthy enough to accept release
risk? Can new workers read old rows? Can old workers ignore new fields during the
transition? Does this high-risk change have human approval? Is rollback a real
path, not a sentence in chat?

**Persistence:** persist the release decision and gate result.

The release decision must survive CI logs and team memory. Store the candidate,
versions, evidence, blockers, canary scope, approver, and rollback plan so an
incident review can reconstruct what changed and why it was allowed.

**Check:** verify unsafe versions cannot affect old or high-risk work.

If the gate blocks, the release should not leak into production through a side
channel. That includes config changes, prompt edits, model-route updates, and
manual worker restarts.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** release gates block unsafe prompt, model, schema, policy, or worker changes.
- **Validation path:** inspect evaluation receipt, SLO budget, compatibility query, migration status, approval, and rollback plan.
- **Stop if:** a change can affect real work without gate evidence.

The evidence should answer one release-review question: what exact behavior is
changing, and what proves old work and high-risk work remain safe?

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, an agent release can change code, schema, prompt, model, policy, tool contract, or evaluation behavior
rule: Release agent behavior through gates that check code, schema, prompts, policies, and evaluation evidence together
tiny example: versioned code, schema, prompt, model, tool, policy, canary, evaluation, compatibility, and rollback evidence
artifact: a release gate that combines evaluation, SLO, compatibility, migration, and approval evidence
proof: release gates block unsafe prompt, model, schema, policy, or worker changes
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Every release changes one or more contracts:

```text
schema contract
worker contract
prompt contract
model contract
tool contract
policy contract
approval contract
```

Safe release engineering keeps old work readable while new behavior is being
introduced, measured, and potentially rolled back.

This is the core mental model. A release is not a moment. It is an overlap
period where old rows, old workers, old prompts, new workers, new prompts, and
in-flight jobs may exist at the same time. The release process is the discipline
that keeps that overlap safe.

For normal software, release engineering often focuses on code and schema. For
agent systems, behavior can change without code changing at all. A new prompt can
alter tool choice. A new model can change refusal behavior. A new policy can
change what requires approval. Those changes need the same seriousness as a
binary deploy.

## Expand And Contract Migrations

Use compatible migrations:

```text
expand:
  add nullable/defaulted column or table
  deploy code that writes both old and new shapes

backfill:
  fill old rows safely in batches

contract:
  remove old reads only after all active workers are compatible
```

The companion schema uses defaults for version columns so old local examples
remain runnable. A production migration should still backfill deliberately and
verify row counts.

Think of expand and contract as a courtesy to work already in the system. Old
jobs did not agree to your new schema. They were created under the old contract.
The migration must give both old and new code a period where the ledger remains
readable.

Migration evidence should also be durable. If a migration happens only in a
terminal scrollback, the next incident reviewer cannot tell which phase ran,
which rows were touched, which compatibility query was used, or which rollback
plan was approved.

The companion tracking schema includes a migration-run ledger:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql:schema_migration_runs}}
```

The corresponding runbook query is:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/schema_migration_status.sql}}
```

It answers:

```text
Which migration phase is planned, running, blocked, failed, or recently passed?
Which target surface and target version are being changed?
Which payload schema range is still compatible?
How many rows were examined and changed?
Which compatibility query and rollback plan justify the migration?
Who signed off terminal migration evidence?
```

This turns migration from "we ran a script" into a reviewable production
transition.

The goal is not paperwork. The goal is to make rollback and incident review
possible. If a new worker dead-letters old jobs after a migration, the team needs
to know which phase ran, which compatibility query passed, and whether the
system can safely return to the previous behavior.

## Tiny Example

You want to add `policy_version` to old jobs.

Unsafe release:

```text
deploy worker that requires policy_version
old rows fail to parse
running jobs dead-letter
```

Safe release:

```text
add column with default
deploy worker that reads old and new rows
backfill deliberately
later require stricter behavior
```

The difference is compatibility during the transition.

The unsafe version assumes the world changed instantly. The safe version accepts
that production is always mixed for a while. Some rows are old. Some workers are
old. Some jobs are already running. Release engineering protects that mixed
period.

Read the tiny case as:

```text
setup: old jobs lack a new policy_version field
transition: expand, backfill, and contract phases keep old work readable
evidence: migration proof, compatibility query, canary, eval receipt, and rollback path exist
invariant: releases must coexist with work already in flight
```

## Worker Version Skew

During deploy, old and new workers may run at the same time.

Design rules:

```text
new workers must read old rows
old workers must ignore unknown noncritical columns
state transitions must remain compatible
prompt/model changes must be versioned
side-effect contracts must not silently change
```

If the new worker cannot safely process old rows, pause the affected job kind
before deploy and resume only after migration.

Version skew is normal, not exceptional. During a rolling deploy, there may be a
moment when worker A understands the old contract and worker B writes the new
contract. The design should make that overlap boring. If it cannot, the rollout
needs a pause, a migration phase, or a narrower canary.

## Canary Workers

Canary one worker before scaling:

```text
route one low-risk job kind
watch provider error rate
watch dead jobs
watch oldest pending age
watch approval decisions
compare cost per job
```

Rollback criteria should be defined before the deploy:

```text
dead job rate exceeds threshold
provider 429s burn error budget
approval bypass count nonzero
secret leak count nonzero
oldest pending age breaches SLO
```

A canary without rollback criteria is only a smaller gamble. Decide in advance
what evidence means "continue," what evidence means "hold," and what evidence
means "roll back." The operator should not invent those rules while the release
is already failing.

## Typed Release Gate

The easy mistake is to treat each release signal as independent:

```text
eval passed
SLO dashboard looks fine
worker can parse the row
someone said yes in chat
```

Those are useful facts, but they are not yet a release decision. A production
release gate turns them into one typed report:

```text
evaluation receipt
  + SLO evaluation
  + worker compatibility report
  + human approval evidence for high-risk changes
  -> release gate report
```

The companion code keeps that decision small and explicit:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/release_gate.rs:release_gate}}
```

The important detail is not the specific enum names. It is the invariant:

```text
no prompt, model, policy, tool, or worker change is promoted from one signal
alone
```

If evals pass but the error budget is exhausted, the gate blocks. If evals pass
but there is no traffic evidence yet, the gate allows only a canary. If the
evaluation receipt and compatibility report refer to different versions, the
gate blocks because the evidence is about different releases.

This is why the gate must bind evidence to versions. "The eval passed" is too
weak. Which prompt version passed? Which model route? Which tool schema? Which
dataset? Which worker build? A release gate should reject evidence that belongs
to a different candidate.

## Durable Release Gate Evidence

The typed gate is the application decision. The database row is the durable
review record.

For a real release, store the gate result in `release_gate_runs`. The row binds
one candidate to the exact prompt, model, tool, policy, worker build,
evaluation receipt, SLO decision, compatibility decision, approval evidence,
blockers, canary percentage, rollback plan, evaluator, and operator signoff.

The runbook query is checked with the rest of the companion SQL:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/release_gate_status.sql}}
```

The Rust boundary keeps the row raw only at the database edge. After that, the
release record becomes a typed receipt:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/release_gate.rs:release_gate_run_row_boundary}}
```

Read the query as a release-review surface:

```text
blocked or canary releases first
recent promotions second
evaluation, SLO, compatibility, migration, and approval evidence in one row
```

The invariant is simple: a release decision should survive the meeting, the CI
job, the deploy, the row conversion boundary, and the incident review.

That durability changes how teams behave. Instead of asking "who approved this?"
during an incident, the team can inspect the row. Instead of asking "what changed
last night?" the team can compare release candidates, gate decisions, and canary
receipts.

## Prompt And Model Releases

Prompts are production code. Model routes are production dependencies.

This does not mean prompts and models are the same as Rust source files. It means
they change production behavior and therefore need versioning, evaluation, and
rollback evidence. A prompt change can alter a tool call. A model change can
alter extraction accuracy. A route change can alter latency, cost, and safety
behavior.

Treat changes as releases:

```text
new prompt_version
new model_route
offline fixture tests
live smoke test
small canary
error-budget watch
rollback path
```

The job row stores the versions so old work remains explainable:

```text
prompt_version
model_route
tool_version
policy_version
worker_build_id
```

Without those versions, a postmortem can only guess. With them, the team can say
which prompt, model, tool contract, policy, and worker build produced a specific
agent action.

## Graceful Shutdown

A worker should:

```text
stop picking new jobs
finish short current jobs
heartbeat if drain is expected
let long jobs recover by lease expiry
record shutdown reason in process logs
```

Do not delete running rows to make deploys clean. The lease model is the cleanup
mechanism.

Graceful shutdown is part of release engineering because deployments happen
while work is active. A worker that stops picking new jobs but lets leases
recover creates a controlled transition. A worker that deletes or rewrites active
state creates evidence loss.

## Formal Definition

For this chapter, the precise definition is:

```text
Release engineering is the compatibility discipline for changing code, schema, prompts, models, tools, and policies while old work remains safe.
```

In the book's system model, **State** means versioned code, schema, prompt,
model, tool, policy, canary, evaluation, compatibility, and rollback evidence.

The **Actor** is the release gate and release owner that can promote, canary,
block, or roll back changes.

The **Transition** is that a change reaches production only after old work
compatibility and behavior evidence are checked.

The **Evidence** is expand-contract migrations, canaries, version fields,
release gates, evaluation receipts, and rollback evidence.

The **Invariant** is that release speed never outruns evidence that old and new
work are safe together.

## What Can Fail

**Design smell:** new code assumes no old jobs exist. That assumption is almost
never true in a durable agent system.

**Production symptom:** old rows, old prompts, or old workers break during
rollout. The release did not fail because the new code was obviously broken. It
failed because the transition between old and new contracts was not safe.

**Corrective invariant:** releases are compatible with work already in the
ledger.

**Evidence to inspect:** expand-contract migrations, version fields, canary
workers, eval receipts, SLO evaluation, compatibility reports, approval
evidence, and a release gate report prove compatibility.


## Production Contract

A release is safe only if old rows remain readable or the affected job kind is
paused before deploy. New behavior needs a version identifier. Canary criteria
must be defined before promotion. Rollback must be possible without deleting
state.

Prompt and model changes must leave evaluation receipts. The release gate should
check evaluation, SLO status, compatibility, version consistency, and approval
evidence together. Side-effect contracts must not silently change.

## Progressive Hardening Path

**Naive version:** new code assumes no old jobs exist. Current workers, prompts,
models, and schemas are treated as the only reality.

**Safer version:** releases are compatible with work already in the ledger.
Release planning now accounts for old jobs, compatible schemas, versioned
behavior, and rollback.

**Production version:** expand-contract migrations, version fields, canary
workers, eval receipts, SLO evaluation, compatibility reports, approval
evidence, and a release gate report prove compatibility. Use the naive version
only to spot the smell. Use the safer version to protect old work. Use the
production version before schema, model, prompt, or policy changes ship.

## Testing Strategy

Test release gates with old and new evidence together:

- **Unit or type test:** prove Rust release-gate logic blocks failed evaluation, exhausted error budget, incompatible work, version mismatch, or missing high-risk approval.
- **Persistence or boundary test:** prove Postgres compatibility, migration
  phase, backfill counts, evaluation, SLO, approval, and version rows can be
  queried for one release candidate.
- **Regression test:** attempt to promote a prompt, model, schema, or policy change without matching evaluation and compatibility receipts; promotion must fail.

## Observability Strategy

Observe releases as gated evidence bundles.

Emit structured `tracing` fields for release candidate, schema version, worker
build, prompt version, model version, gate name, decision, and trace id. These
fields connect runtime behavior back to the release that introduced it.

Record an operation event when evaluation, compatibility, SLO, approval, canary,
rollback, or promotion gates pass or block. A blocked release is important
evidence, not noise.

The runbook query should show which old jobs remain compatible, which migration
phase is active, which backfill evidence exists, and which evidence allowed the
release to progress.

## Security and Safety Considerations

Release gates are security boundaries for behavior and schema change.

Treat release metadata, migration payloads, eval outputs, and compatibility
reports as untrusted until verified by typed gates. A malformed release record
should not promote production behavior.

authorization, sandboxing, and approval policies must be included in release
evidence when the change affects tools, tenants, or external actions. Redact
release secrets and provider credentials while preserving candidate id, versions,
gate decisions, blockers, and rollback evidence.

## Operational Checklist

Use this checklist before relying on safe release of code, schema, prompt, model,
and policy changes in production.

**State:** A release has versions, compatibility checks, evaluation receipts,
canary state, release-gate row, rollback plan, and promotion decision.

**Boundary:** Prompt, model, policy, schema, and worker changes are versioned
inputs to a gate, not hidden config drift.

**Failure:** The gate blocks incompatible old work, failed evals, exhausted error
budget, missing approval, and unsafe migration.

**Observability:** Release events connect version ids, eval results,
compatibility query, migration status, release-gate status query, backfill
counts, SLO budget, canary metrics, and rollback status.

**Safety:** High-risk releases require authorization, human approval, sandbox
compatibility, redaction review, and replay safety.

## Exercises

1. Write a negative test where a prompt/model release tries to promote without matching
   evaluation receipt and idempotency-safe rollback evidence. Explain which idempotency
   key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: release_receipts, eval results, compatibility-risk
   rows, canary events, and rollback evidence.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   ReleaseCandidate, PromptVersion, ModelVersion, PolicyVersion, and ReleaseGateDecision
   types. Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which surfaces can change in an agent release?
- Explain: Why prompt and model changes need release evidence like code and schema changes do.
- Apply: Add `policy_version` to old jobs using expand, backfill, and contract phases.
- Evidence: Name the migration proof, compatibility query, canary result, evaluation receipt, and rollback path.

## Summary

Safe release engineering keeps old rows readable, new behavior measurable, and rollback possible. That applies equally to code, schema, prompts, models, and policies.

- **Invariant:** no behavior version is promoted until compatibility, evaluation, SLO, migration, approval, and rollback evidence are acceptable for the job kind.
- **Evidence:** release receipts, eval results, compatibility-risk queries, canary metrics, error budget state, and rollback plans prove readiness.
- **Carry forward:** release gates protect users from both software bugs and behavior regressions.

## Changed Understanding

- **Before this chapter:** release looked like deploying changed code.
- **After this chapter:** agent release engineering versions and gates code, schema, prompts, models, policies, evals, and rollback evidence.
- **Keep:** version code, schema, prompt, model, policy, evaluator, dataset, and rollback plan as one release packet.

## Further Reading & Credible References

- **[Walter F. Tichy: Tools for Software Configuration Management](https://dl.acm.org/doi/10.5555/54366.54370)** (1988). The foundational academic paper for version and configuration control. It formalizes the concepts of "Source vs. Derived Objects" and "Immutability" used to manage the behavior packets in this chapter.
- **[Martin Fowler: The Expand and Contract Pattern](https://martinfowler.com/bliki/ParallelChange.html)**. The definitive practical guide to "Parallel Change"—implementing backward-incompatible schema and worker changes without downtime.
- **[Netflix Technology Blog: Automated Canary Analysis at Netflix with Kayenta](https://netflixtechblog.com/automated-canary-analysis-at-netflix-with-kayenta-3260bc7acc69)**. Explains the statistical approach to comparing "Baseline" and "Canary" metrics to ensure release safety.
- **[Anthropic: Prompt Engineering Best Practices](https://docs.anthropic.com/en/docs/build-with-claude/prompt-engineering/overview)**. Industry guidance for treating prompts as versioned code artifacts, including the use of "Golden Datasets" for regression testing.
- **[Designing Data-Intensive Applications](https://dataintensive.net/)** (Martin Kleppmann, Chapter 4: Encoding and Evolution). The primary reference for managing version skew and the formal limits of forward/backward compatibility.
