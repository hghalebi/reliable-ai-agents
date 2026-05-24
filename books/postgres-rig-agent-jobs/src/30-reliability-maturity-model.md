# 30. Reliability Maturity Model

## What You Will Learn

This chapter teaches you to:

- explain what should improve next as the agent system becomes more important;
- inspect each job kind for maturity level, risk, missing control, owner, and review date;
- verify that reliability maturity is measured by evidence, not ambition.

The production evidence is a maturity scorecard that maps job risk to a target
level, current gaps, next upgrade, owner, and validation command.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** recovery controls show which job kinds are truly dependable.
- **Adds:** evidence-backed maturity by job kind and risk.
- **Prepares:** scaling paths that preserve evidence after Postgres-first limits appear.

## Production Failure

Two teams both say their agents are "production ready."

One agent drafts internal summaries. The other can update billing records. They
have the same checklist, even though their risk is completely different.

- **What breaks:** maturity was treated as a slogan instead of evidence matched
  to risk.
- **False fix:** demand the heaviest controls for every job or accept the same
  lightweight controls for all jobs.
- **Design response:** score each job kind by risk, target maturity, existing
  proof, missing control, owner, next upgrade, and review date.

## Motivation

In production, "is this agent ready?" is too vague. A support triage assistant, billing adjustment agent, and deployment rollback agent need different reliability bars.

Without a maturity model, teams either overbuild low-risk work or underbuild high-risk work. This chapter maps job risk to concrete controls, evidence, gaps, owners, and next reviews.

## Plain Version

Read this as the simple version:

- **Simple rule:** Maturity means the system can prove more of its reliability with less guesswork.
- **Why it matters:** Teams need a way to see which controls are real, which are manual, and which are missing.
- **What to watch:** Watch evidence for each maturity level across state, types, retries, observability, evaluation, security, and operations.

## What You Already Know

Start with these anchors:

- The book has introduced durable state, typed boundaries, ownership, idempotency, observability, operations, evaluation, security, and recovery.
- Different job kinds carry different risks.
- Reliability work needs a next concrete upgrade.

This chapter adds: a maturity model. You will map each job kind to current
level, target level, missing control, owner, review date, and validation
evidence.

## Focus Cue

Keep three things in view:

- **State:** one job kind, target level, current evidence, gap, owner, next upgrade, and review date.
- **Move:** a job kind moves to a higher level only after the required evidence for that level exists.
- **Proof:** Each level names current proof, gap, next upgrade, owner, and review date.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a maturity scorecard by job kind with gap, owner, next artifact, and review date.
- **Why it matters:** different agent jobs need different reliability bars, and maturity should be explicit.
- **Done when:** each capability has evidence, a target level, an owner, and the next concrete improvement.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** readiness scorecard, Appendix E, Appendix R, and maturity review rows.
- **State transition:** map each job kind to the reliability level it actually needs.
- **Evidence path:** gap, owner, target level, next artifact, and review date are explicit.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** What maturity level does this job kind require, and what evidence is missing?
- **Evidence to inspect:** job-kind risk, current level, target level, gap, owner, next artifact, and review date.
- **Escalate if:** the team applies one reliability bar to all agent work or cannot name the next improvement.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a job kind is reviewed for production maturity.
2. **Action:** compare its current controls to the target reliability level.
3. **Persistence:** persist gap, owner, next artifact, and review date.
4. **Check:** verify the next improvement is concrete and evidence-based.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** each job kind has target level, gap, owner, next artifact, and review date.
- **Validation path:** inspect readiness scorecard, maturity rows, Appendix E, and Appendix R traceability.
- **Stop if:** the team cannot name the next evidence-backed reliability improvement.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, "is this agent ready?" is too vague
rule: Maturity means the system can prove more of its reliability with less guesswork
tiny example: one job kind, target level, current evidence, gap, owner, next upgrade, and review date
artifact: a maturity scorecard by job kind with gap, owner, next artifact, and review date
proof: each job kind has target level, gap, owner, next artifact, and review date
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Think of maturity as the number of failures the system can absorb without
surprise:

```text
Level 0 absorbs almost nothing.
Level 1 absorbs process crashes.
Level 2 absorbs invalid data and provider boundary drift.
Level 3 absorbs operational load and incidents.
Level 4 absorbs behavior drift.
Level 5 absorbs time, team rotation, provider change, and disaster.
```

The level is not a badge. It is a statement about what kinds of failure the
current design can handle.

## Level 0: Script

The agent is a script or notebook:

```text
no durable state
no retries
no audit trail
manual reruns
provider errors visible only in logs
```

This is fine for learning. It is not a production system.

## Level 1: Durable Jobs

The agent has a job ledger:

```text
pending/running/succeeded/failed states
idempotency key
lease ownership
retry policy
dead letter state
event timeline
```

At this level, crashes become recoverable. Operators can answer what happened
to a job.

## Level 2: Typed Boundaries

The agent has domain types and explicit boundaries:

```text
typed payloads and results
typed error classes
provider adapter
tool schemas
policy result
versioned rows
```

At this level, invalid states are harder to express, and old work can be
replayed or inspected safely.

## Level 3: Operated Service

The agent has SRE-grade operation:

```text
SLIs and SLOs
burn-rate alerts
capacity controls
backpressure
runbooks
incident process
release gates
graceful shutdown
```

At this level, the team can run the system under load and respond when it
misbehaves.

## Level 4: Behavior-Gated Agent

The agent has behavior reliability controls:

```text
fixture evals
historical shadow evals
prompt and model release receipts
policy gates
human approval samples
drift detection
post-incident regression cases
```

At this level, the team can change prompts, models, and tools without relying
on hope.

## Level 5: Long-Horizon Platform

The agent platform supports years of operation:

```text
multi-version compatibility
restore drills
security threat model
tenant isolation
tool capability governance
cost governance
toil automation
ownership rotation
deprecation policy
```

At this level, reliability is not one engineer's memory. It is built into the
system, the tests, the runbooks, and the team's operating rhythm.

## How to Use the Model

Assign a maturity level per job kind. A low-risk summarizer may only need Level
2. A payment or compliance agent may need Level 5 before it can act
autonomously.

Then choose the next smallest upgrade:

```text
no ledger -> add durable jobs
no versioning -> add replay metadata
no behavior gate -> add fixture and shadow evals
no runbook -> add operator diagnostics
no restore proof -> run a restore drill
```

The important move is to choose the next upgrade from the failure that matters
most for that job kind. Do not start with the most impressive control. Start
with the missing control that would hurt users, operators, or the business if it
failed tomorrow.

For a low-risk internal summarizer, the next upgrade may be typed output and
visible dead-letter state. If it fails, users wait or receive a draft that can be
regenerated. That is still worth fixing, but it is not the same risk as a
billing agent that can create external financial changes.

For a billing agent, the next upgrade may be side-effect receipts, approval
evidence, and a rollback or compensation path. If that agent repeats an action,
the business may charge a customer twice or issue the wrong credit. The maturity
model forces the team to see that difference before saying both agents are
"ready."

This is why maturity is per job kind. A product can have one Level 2 job kind,
one Level 4 job kind, and one job kind that should stay at Level 0 because it is
only a research script. That is not inconsistency. It is honest engineering.

## Tiny Example

A support summarizer that only drafts internal notes may be acceptable at Level
2: durable jobs, typed payloads, and visible failures. A billing agent that can
trigger external actions needs Level 5 controls: approvals, receipts, restore
drills, threat model, versioning, and ownership.

The same platform may host both. The mistake is treating every job kind as if it
has the same risk.

Read the tiny case as:

```text
setup: two job kinds carry different business risk
transition: each job kind gets a current level, target level, and next upgrade
evidence: scorecard, missing control, owner, review date, and validation command justify maturity
invariant: maturity is evidence of risk control, not aesthetic architecture
```

## Worked Walkthrough

Consider three job kinds in the same product.

The first job kind summarizes internal support tickets for a team lead. It does
not write to external systems. It does not send messages to customers. If it
fails, a human can read the original tickets. This job kind still needs durable
state, typed output, visible failure, and basic evaluation, but it may not need
the strongest approval and disaster-recovery controls before the first internal
release.

The second job kind drafts customer refunds. It does not issue the refund by
itself, but it proposes an amount and explanation. This job kind needs stronger
evidence. It needs prompt and model versions, evaluation receipts for refund
cases, policy checks, human approval, and audit events. The system must prove
that the model proposed a refund and that a person approved the final action.

The third job kind executes approved refunds through an external payment tool.
This is a different risk class. Now retries, idempotency, side-effect receipts,
compensation, security boundaries, incident response, and restore drills become
part of the minimum serious bar. The tool call is not just output. It changes the
world outside the database.

If the team uses one maturity label for all three job kinds, the label becomes
meaningless. If the team says the whole product is Level 4, the support
summarizer may be overburdened while the refund executor may still be
underprotected. The mature decision is narrower: each job kind receives a target
level, current evidence, missing control, owner, next artifact, and review date.

This also makes planning easier. The team does not need to "make the platform
reliable" in one vague push. It can say: the summarizer needs typed output
validation this week; refund drafting needs a new evaluation slice before the
next prompt release; refund execution needs receipt-backed replay before it can
leave human-supervised mode. Each improvement is small enough to build and clear
enough to review.

The maturity model is therefore not bureaucracy. It is a map from risk to the
next concrete engineering move.

## How Maturity Connects The Book

The maturity model is where the earlier chapters stop being separate topics.
Durable state from Part I becomes the Level 1 floor. Typed boundaries and
provider adapters become the Level 2 floor. SLOs, runbooks, incidents, and
release gates become the Level 3 floor. Evaluation, approval, memory governance,
and security become Level 4 and Level 5 requirements when the job kind can
affect real users or external systems.

Read the model from left to right:

```text
first make work durable
then make boundaries typed
then make operation observable
then make behavior evaluated
then make long-term change, recovery, and governance practiced
```

This order prevents a common mistake. Teams sometimes add advanced observability
or orchestration before the basic state machine is clear. That creates more
tools, not more reliability. Maturity should increase the system's ability to
prove and preserve invariants. If a new control does not improve that ability,
it may be noise.

The model also prevents the opposite mistake: staying with the minimum design
after risk has grown. A draft-only assistant can begin with a small control
surface. A tool-using agent that changes customer data cannot. The same
architecture can support both, but the readiness evidence must differ.

## Formal Definition

For this chapter, the precise definition is:

```text
A maturity level is an evidence-backed target for one job kind, not a general label for the whole product.
```

In the book's system model:

- **State:** one job kind, target level, current evidence, gap, owner, next upgrade, and review date.
- **Actor:** the service owner and reviewer assign maturity based on proof, not aspiration.
- **Transition:** a job kind moves to a higher level only after the required evidence for that level exists.
- **Evidence:** Each level names current proof, gap, next upgrade, owner, and review date.
- **Invariant:** maturity is an evidence-backed operating decision for a specific job kind.

In the companion code, this boundary is represented by
`job_kind_readiness_reviews`, `job_kind_readiness_review.sql`, and
`JobKindReadinessReview`. Raw database labels such as `prototype`,
`production`, and `regulated_high_risk` are decoded into `MaturityLevel`,
`JobRiskClass`, `ReadinessEvidence`, and `JobKindReadinessStatus` before the system
trusts the maturity claim.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Maturity labels are aspirational. |
| Production symptom | A risky job is called "production-ready" without the controls its risk requires. |
| Corrective invariant | Maturity is assigned per job kind from evidence. |
| Evidence to inspect | Scorecards list current evidence, gaps, owners, review dates, and next upgrades. |


## Production Contract

Use the maturity model as an operating tool:

```text
assign a target level per job kind
record the current level with evidence
choose one next upgrade
tie promotion to tests, runbooks, evals, or drills
review levels after incidents and major releases
```

If a maturity level cannot be backed by evidence, it is only a label.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Maturity labels are aspirational. | Maturity is assigned to the whole product as a vague confidence label. |
| Safer version | Maturity is assigned per job kind from evidence. | Each job kind gets a maturity target, evidence gap, owner, next upgrade, and review date. |
| Production version | Scorecards list current evidence, gaps, owners, review dates, next upgrades, and typed readiness rows. | The maturity model turns improvement into an auditable backlog tied to concrete controls and rejects impossible claims such as regulated work targeting ordinary production evidence. |

Use the naive row when maturity is a label. Use the safer row to scope it to a job kind. Use the production row before leadership treats reliability as complete.

## Testing Strategy

Test maturity as evidence for one job kind:

- **Unit or type test:** prove Rust maturity or scorecard objects reject levels without current proof, gap, next change, owner, and review date.
- **Persistence or boundary test:** prove Postgres or evidence-packet rows can attach maturity claims to a specific job kind and validation artifact.
- **Regression test:** attempt to mark the whole product mature without per-job evidence; the scorecard should force a narrower, reviewable claim.

## Observability Strategy

Observe maturity as per-job evidence, not product confidence:

- Emit structured `tracing` fields for job kind, maturity level, evidence packet, owner, gap, next change, review date, and trace id.
- Record an operation event when a job kind moves between maturity levels or when a review blocks the claimed level.
- The runbook query should show the current proof, missing proof, owner, and next artifact for the specific job kind under review.

## Security and Safety Considerations

Maturity claims must include safety evidence, not only availability evidence:

- Treat maturity scores, evidence packets, owner claims, and review notes as untrusted until linked to concrete controls.
- authorization, sandboxing, and approval evidence should be required for higher maturity levels on risky job kinds.
- Redact sensitive review material while preserving job kind, maturity level, gap, owner, next change, and safety evidence.

## Operational Checklist

Use this checklist before relying on maturity levels by job risk in production:

- **State:** Each job kind has target maturity, current controls, evidence gaps, owner,
  next action, and review date.
- **Boundary:** Maturity ratings depend on concrete Rust, Postgres, Rig, evaluation,
  security, and operations evidence.
- **Failure:** The model exposes gaps such as no restore drill, no eval receipt, no
  approval gate, or unsafe retry policy.
- **Observability:** A readiness scorecard connects each rating to metrics, tests,
  runbook output, incidents, and release evidence.
- **Safety:** Higher-risk job kinds require stronger authorization, approval, sandbox,
  retention, and replay controls.

## Exercises

1. Write a negative test where a regulated job kind is scored mature without evaluation
   receipt, restore drill, approval state, or idempotency evidence. Explain which
   idempotency key, receipt, or state transition prevents duplicate work.
2. Inspect the Postgres evidence: `job_kind_readiness_reviews` rows linked to
   job kind, risk class, target level, current level, evidence counts, owner,
   next change, and next review.
3. Extend the Rust `JobKindReadinessReview` type with one additional evidence
   field. Then update `job_kind_readiness_review.sql`, row conversion tests,
   and the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What reliability layers has the book added?
- Explain: Why maturity depends on job risk rather than team ambition?
- Apply: Assign one job kind a current level, target level, and smallest next upgrade.
- Evidence: Name the missing control, owner, review date, validation command, and production artifact that proves the upgrade.

## Summary

Reliable AI agents are not one feature. They are a ladder of engineering capabilities that become stricter as user, business, and safety risk increase.

- **Invariant:** each job kind has a target maturity level backed by current evidence, gaps, owners, and next review dates.
- **Evidence:** readiness scorecards, control evidence, eval receipts, restore drills, security reviews, runbook outputs, and incident history justify the level.
- **Carry forward:** maturity is a decision about risk, not a badge.

## Changed Understanding

- **Before this chapter:** maturity looked like adding more controls and process.
- **After this chapter:** maturity means the system can prove, practice, and improve its reliability invariants at each operating level.
- **Keep:** score maturity only from evidence: tests, runbooks, incidents, ownership, drills, and measured outcomes.

## Further Reading and Sources

- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) gives the operational frame for SLIs, SLOs, error budgets, toil, incidents, and release discipline.
- [OpenTelemetry documentation](./31-credible-resources-further-reading.md#reliability-and-operations) supports the chapter's treatment of traces, metrics, logs, and cross-boundary evidence.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) connects operational evidence back to durable state, transactions, and event histories.
