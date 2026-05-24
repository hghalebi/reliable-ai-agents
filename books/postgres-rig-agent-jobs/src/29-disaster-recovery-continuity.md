# 29. Disaster Recovery And Continuity

## What You Will Learn

This chapter teaches you to:

- explain how the system restarts after serious data, provider, or infrastructure loss;
- inspect state inventory, RPO, RTO, backups, restore drills, replay safety, and provider continuity plans;
- verify that recovery is practiced before an emergency.

The production evidence is a recovery packet with backup proof, restore test
results, replay rules, provider fallback choices, and operator sign-off.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** security and data-protection boundaries identify what must survive and what must not leak.
- **Adds:** restore, replay, RPO, RTO, and receipt-safe recovery.
- **Prepares:** a maturity model for choosing the next reliability investment.

## Production Failure

Postgres restores successfully after an outage.

The rows are back, but the team cannot tell which tool calls already reached
external systems, which jobs can replay, and which side effects need
reconciliation.

- **What breaks:** backup success did not prove workflow recovery.
- **False fix:** restart all workers and let retries sort it out.
- **Design response:** practice restore drills with RPO, RTO, receipts,
  replay-safety decisions, quarantined work, provider fallback, and operator
  sign-off.

## Motivation

In production, a bad day will eventually arrive.

A backup may restore rows, but rows alone do not tell you which side effects
already happened or which jobs can safely replay. A restored `agent_jobs` row can
say that work was running. It cannot, by itself, prove whether an external email
was sent, a billing API accepted the request, a CRM update landed, or a human
approval was already used.

Without restore drills, receipt reconciliation, and replay policy, recovery can
duplicate external actions or abandon valid work. Both failures are serious. A
duplicate side effect can charge, email, approve, or mutate something twice. An
abandoned job can leave a user-facing workflow stuck after the database appears
healthy.

This chapter treats continuity as an operational proof, not a backup checkbox.
The proof is simple to state and hard to fake: after restore, the system knows
what can resume, what must reconcile, what must quarantine, and what must never
replay.

## Plain Version

Read this as the simple version:

**Simple rule:** Backups matter only when restore and replay are practiced and
safe. A backup proves that bytes existed somewhere. Recovery proves that the
system can use restored state without repeating unsafe work.

**Why it matters:** Long-running agents can leave partial side effects. The
worker may crash after an external API succeeds but before the local receipt is
stored. A backup may restore the system to a moment before that receipt existed.
Recovery must know what can resume, what must reconcile, and what must stop.

**What to watch:** Watch restore time, replay candidates, side-effect receipts,
quarantined work, provider fallback, RPO, RTO, and continuity drills. Those are
the signals that tell you whether the system can recover safely, not only
whether the database can be restored.

## What You Already Know

Start with these anchors:

- Security and evaluation reduce risk.
- They do not remove the need to recover after data, provider, or infrastructure loss.
- Recovery without evidence becomes guessing.

This chapter adds: disaster recovery and continuity. You will define RPO, RTO,
backup proof, restore drills, replay safety, provider fallback, and operator
sign-off.

## Focus Cue

Keep three things in view:

- **State:** backup inventory, restored rows, receipts, replay candidates, replay decisions, RPO, RTO, provider continuity, and signoff.
- **Move:** restored work resumes only after replay safety and duplicate-side-effect risk are decided explicitly.
- **Proof:** Restore drill rows, replay decisions, receipt checks, provider continuity, paused workers, and operator signoff exist.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a restore and replay plan that classifies work as resumable, reconciled, quarantined, or terminal.
- **Why it matters:** backups are not enough if restored work can repeat side effects or lose audit evidence.
- **Done when:** a restore drill proves RPO, RTO, replay safety, receipt reconciliation, and quarantine behavior.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/recovery.rs`, restore candidate SQL, receipt reconciliation, and restore drill tests.
- **State transition:** restore state and classify replay before resumed work can cause side effects.
- **Evidence path:** RPO, RTO, replay safety, quarantine, and reconciliation are proven in a drill.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** After restore, which work can resume, reconcile, quarantine, or stay terminal?
- **Evidence to inspect:** restore candidate row, side-effect receipt, replay classification, RPO/RTO measurement, and drill result.
- **Escalate if:** restored work might repeat side effects or disappear because replay rules were not practiced.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** A restore or replay decision is needed. This may happen after a
database restore, provider outage, region incident, accidental deletion,
corrupted deployment, or restore drill.

**Action:** Classify each restored job before workers resume. Some jobs are safe
to resume because no side effect was expected yet. Some need receipt
reconciliation because a side effect may already have happened. Some must be
quarantined because the evidence is incomplete. Terminal jobs should stay
terminal.

**Persistence:** Persist the replay classification, receipt evidence, RPO, RTO,
provider continuity decision, and drill result. A recovery decision that exists
only in a meeting or shell history will not help the next operator.

**Check:** Verify that restore does not repeat side effects or lose auditability.
The check should happen before normal workers resume writes or external calls.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** restore classifies work before replay can happen.
- **Validation path:** inspect restore candidates, receipts, replay classification, RPO/RTO evidence, and drill tests.
- **Stop if:** restored work can repeat side effects or lose audit history.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, a bad day will eventually arrive
rule: Backups matter only when restore and replay are practiced and safe
tiny example: backup inventory, restored rows, receipts, replay candidates, replay decisions, RPO, RTO, provider continuity, and signoff
artifact: a restore and replay plan that classifies work as resumable, reconciled, quarantined, or terminal
proof: restore classifies work before replay can happen
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Disaster recovery is a replay problem under stress. After a restore, the system
must decide what can safely continue, what must stay paused, and what must never
be repeated.

```text
restore state -> inspect evidence -> verify compatibility -> resume safely
```

The dangerous moment is not only the outage. It is the first restart after the
outage, when workers may act on restored or partially stale state.

## RPO And RTO

Start with two numbers:

```text
RPO: how much completed work can the business afford to lose?
RTO: how long can the system be unavailable?
```

RPO means recovery point objective. It asks how much completed work the business
can afford to lose. If the RPO is fifteen minutes, the business is saying that a
restore may lose up to fifteen minutes of accepted work. That may be fine for a
research summary. It is usually not fine for billing, approval, or external
messages.

RTO means recovery time objective. It asks how long the system can be
unavailable. A support summarizer may tolerate hours. An incident triage
assistant may need minutes.

For agent jobs, the answer is rarely one number for the whole system. A billing
agent, support summarizer, and internal research agent have different recovery
needs, so the book models recovery objectives by job kind.

For example, `incident_triage` may need near-zero data loss and recovery in
minutes because it is operator-facing and time-sensitive. `billing_action` may
need zero accepted-work loss because money and approvals are involved, even if
the recovery time can be minutes to hours. `research_summary` can often tolerate
hours of data loss and hours to days of recovery time because the work can be
regenerated.

## Durable State Inventory

A restore plan must know which state matters:

```text
agent_jobs
agent_job_events
job kind pause controls
prompt and policy versions
tool versions
evaluation receipts
side-effect receipts
operator approvals
secrets and provider credentials
observability traces
```

The job row alone is not enough. The event ledger explains what happened before
the restore point. Version rows explain which prompt, model, policy, worker, and
schema were active when work ran. Approval rows explain whether risky work had
permission. Side-effect receipts explain what must not be repeated.

The inventory should also name data-protection requests and tenant-boundary
evidence. Recovery must not resurrect data that should remain redacted, and it
must not allow restored work to cross tenant boundaries because an old row was
trusted too quickly.

## Restore Drill

A restore drill should be practiced before production needs it. The drill should
restore a database backup into an isolated environment, then prove that restored
work can be inspected before it acts.

```text
restore database backup into isolated environment
run schema compatibility checks
verify row counts and recent event timelines
start workers in dry-run or paused mode
sample pending, running, dead, and succeeded jobs
confirm side-effect receipt integrity
resume one low-risk job kind
measure time to safe processing
```

The most important step is starting paused. After restore, you want to inspect
state before workers resume writes or external side effects. A worker that starts
too early can turn a successful restore into a duplicate billing action, repeated
email, stale approval, or unsafe replay.

## Tiny Example

A backup is restored from 10:00. At 10:03, before the outage, a side-effect
worker successfully called an external billing API, but the restored database no
longer contains the receipt.

If the system resumes blindly, replay may repeat the billing action. The safe
recovery plan starts paused, checks external receipts or idempotency keys, and
only then resumes the affected job kind.

Read the tiny case as:

```text
setup: restored database predates an external billing side effect
transition: workers pause while receipts and external idempotency are reconciled
evidence: backup proof, receipt inventory, replay decision, restore drill, and operator sign-off exist
invariant: recovery must preserve side-effect truth, not only database availability
```

## Replay Safety

Replay is only safe when side effects are idempotent or separated from model
work.

Safe replay pattern:

```text
agent proposes action
proposal is stored
policy approves
side-effect worker executes with idempotency key
receipt is stored
replay checks receipt before acting again
```

Unsafe replay pattern:

```text
agent calls external API directly
worker crashes before recording result
lease expiry runs the same external action again
```

The second pattern cannot be made reliable by better logging. The architecture
is wrong because the durable receipt is missing at the side-effect boundary.

## Typed Restore And Replay Decisions

The recovery procedure should not live only in a checklist. The companion code
models restore decisions as typed data:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/recovery.rs:replay_decision}}
```

Read this as a production rule. A restored job with no expected side effect can
resume from durable state. A restored job with an existing receipt needs
reconciliation. A restored job that expected a side effect but has no receipt
is quarantined. Terminal work does not replay.

The restore drill then records whether the system met its recovery objective
and whether any replay candidate was unsafe:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/recovery.rs:restore_drill_report}}
```

Raw database rows still appear at the boundary, but they are immediately
converted into typed replay candidates:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/recovery.rs:replay_candidate_row_boundary}}
```

The operator query produces the row shape that feeds that boundary:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql}}
```

This is the same rule as the rest of the book:

```text
raw outside -> typed inside -> controlled side effect
```

After restore, workers should start paused. The first decision is not "run
everything again." The first decision is "which rows have enough evidence to
resume without duplicating a side effect?"

## Provider Continuity

Reliable agents also need provider continuity. Keep provider choice behind the
agent runner boundary:

```text
DeepSeek outage -> route low-risk jobs to fallback model
model behavior drift -> pin old model route for sensitive job kinds
key revoked -> pause affected job kinds and rotate credentials
rate limit -> backpressure instead of retry storm
```

Provider fallback is not free. A fallback model needs its own evaluation
receipt and policy decision. Otherwise failover can trade availability for bad
behavior.

## Formal Definition

For this chapter, the precise definition is:

```text
Disaster recovery is the practiced ability to restore state, classify replay safety, prevent duplicate side effects, and resume work inside RPO and RTO.
```

In the book's system model:

**State:** The state is backup inventory, restored rows, receipts, replay
candidates, replay decisions, RPO, RTO, provider continuity, and signoff.

**Actor:** The recovery operator restores data with workers paused, classifies
replay safety, reconciles receipts, and resumes workers only after the evidence
is good enough.

**Transition:** The transition is the controlled move from restored state to
safe processing. Restored work resumes only after replay safety and
duplicate-side-effect risk are decided explicitly.

**Evidence:** The evidence is restore drill rows, replay decisions, receipt
checks, provider continuity records, paused workers, and operator signoff.

**Invariant:** Recovery restores service without losing evidence or repeating
unsafe external actions.

## What Can Fail

**Design smell:** Backup existence is treated as recovery readiness. The team
can point to snapshots, but it has not proved that restored work can resume
without duplicate side effects.

**Production symptom:** Restore duplicates side effects, loses audit evidence,
resumes workers before replay safety is known, or leaves valid work abandoned
because no one can classify it.

**Corrective invariant:** Recovery is practiced restore, replay, receipt
handling, and controlled resume.

**Evidence to inspect:** `ReplayDecision`, `restore_replay_candidates.sql`, and
restore drill records should show RPO, RTO, side-effect receipts, provider
continuity, quarantines, and operator signoff.


## Production Contract

Disaster recovery is credible only when:

RPO and RTO are defined per job kind. Restore drills start workers paused. State
inventory includes jobs, events, versions, approvals, side-effect receipts,
tenant-boundary evidence, and data-protection evidence.

Replay checks side-effect receipts and idempotency keys before acting again.
Replay decisions are typed before workers resume. Provider fallback has
evaluation and policy approval. Restore time and safe-resume time are measured,
not guessed.

## Progressive Hardening Path

**Naive version:** Backup existence is treated as recovery readiness. Having a
snapshot is treated as proof that the agent can safely resume after disaster.
This is a false comfort.

**Safer version:** Recovery is practiced as restore, replay, receipt handling,
and controlled resume. Restore drills classify replay safety, receipt evidence,
paused work, and reconciliation needs.

**Production version:** `ReplayDecision`, `restore_replay_candidates.sql`, and
restore drill records show RPO, RTO, side-effect receipts, provider continuity,
quarantines, and operator signoff. Continuity is proved by evidence instead of
hope.

Use the naive version as a warning. Use the safer version to practice restore.
Use the production version before promising continuity to users.

## Testing Strategy

Test restore as replay classification, not backup existence:

Unit or type tests should prove Rust replay decisions separate safe resume,
receipt reconciliation, missing-receipt quarantine, terminal no-replay, and RTO
failure.

Persistence or boundary tests should prove Postgres restore replay candidate
queries expose side-effect expectations, restored receipts, job status, and
operator signoff evidence.

Regression tests should restore from an old snapshot with a missing receipt.
Replay must quarantine or reconcile instead of duplicating the side effect.

## Observability Strategy

Observe restore and replay decisions before resuming work:

Emit structured `tracing` fields for restore drill id, job id, snapshot time,
replay decision, receipt count, RPO, RTO, operator signoff, and trace id. Record
an operation event when restored work is classified as safe resume, reconcile,
quarantine, terminal no-replay, or RTO failure.

The runbook query should show which jobs can resume, which need receipt
reconciliation, and which must remain paused after restore. The operator should
not have to infer recovery safety from raw row counts.

## Security and Safety Considerations

Restore can reintroduce old trust mistakes if replay is careless:

Treat restored rows, old receipts, backup metadata, and replay candidates as
untrusted until restore policy classifies them. Restored data is familiar, but it
is still crossing a recovery boundary.

authorization, sandboxing, and approval may need revalidation after restore
before tools or compensating side effects resume. A restored approval may no
longer be valid if policy, tenant scope, data-protection status, or credential
state changed after the snapshot.

Redact restored payloads and backup secrets while preserving snapshot time,
replay decision, receipt evidence, RPO, and RTO evidence. Recovery evidence
should help operators reason without exposing more sensitive data than needed.

## Operational Checklist

Use this checklist before relying on restore, replay, and continuity in production:

**State:** Backups, restore time, replay candidates, receipts, quarantines,
provider continuity, RPO, and RTO are recorded.

**Boundary:** Restored rows are validated through domain conversion before
workers can replay or complete them.

**Failure:** Missing receipts, unknown terminal state, provider outage, and
exceeded RTO become explicit restore failures.

**Observability:** Restore drills report RPO, RTO, replay decisions, quarantined
jobs, receipt reconciliation, and operator signoff.

**Safety:** Replay never repeats side effects without idempotency receipt checks,
approval state, and compensation policy.

## Exercises

1. Write a negative test where a restored job has no side-effect receipt and must be
   quarantined instead of replayed blindly. Explain which idempotency key, receipt, or
   state transition prevents duplicate work.
2. Sketch the Postgres evidence: restore drill, replay candidate, receipt count,
   quarantine reason, and provider continuity rows.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   RestoreDrill, ReplayDecision, ReceiptReconciliation, Rpo, and Rto types. Then name
   the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What do RPO and RTO mean for agent jobs?
- Explain: Why should workers pause before replay after restore?
- Apply: Restore a database backup that predates a billing side effect.
- Evidence: Name the backup proof, side-effect receipt, external idempotency key, replay rule, restore drill, and operator check.

## Summary

Disaster recovery is not only backup and restore. For reliable agents, recovery means restoring enough state, evidence, receipts, and policy context to decide what can safely continue.

Invariant: restored work is replayed, skipped, reconciled, or quarantined
according to explicit receipt and compatibility evidence.

Evidence: backup metadata, RPO, RTO, restore drills, replay candidates, receipt
counts, quarantine reasons, and operator signoff prove continuity.

Carry forward: a backup is not recovery until replay safety is tested.

## Changed Understanding

**Before this chapter:** backup looked like enough disaster recovery.

**After this chapter:** recovery is proven only when restore, replay, RPO, RTO,
and operator decisions are practiced from durable evidence.

**Keep:** prove restore and replay with backups, checkpoints, RPO/RTO targets,
receipt checks, quarantine decisions, and reconciliation queries.

## Further Reading & Credible References

- **[FEMA: Continuity of Operations (COOP) Planning](https://www.fema.gov/pdf/about/org/ncp/coop_brochure.pdf)**. While focused on government, this standard provides the foundational vocabulary for "Business Continuity Planning" (BCP) and the "Orders of Succession" used to manage the operator sign-off in this chapter.
- **[Google SRE Book: Data Integrity—What You Read is What You Wrote](https://sre.google/sre-book/data-integrity/)**. Explains the formal relationship between backups, checksums, and the "Receipt Reconciliation" needed to prevent side-effect duplication during recovery.
- **[AWS Builders' Library: Ensuring Rollback Safety](https://aws.amazon.com/builders-library/ensuring-rollback-safety-during-deployments/)**. Although focused on code, the principles of "Compatibility" and "Evidence" apply directly to the restored database rows discussed in this chapter.
- **[Designing Data-Intensive Applications](https://dataintensive.net/)** (Martin Kleppmann, Chapter 10: Batch Processing). Connects "Fault Tolerance" to the formal ability to resume a multi-step workflow from a durable checkpoint.
