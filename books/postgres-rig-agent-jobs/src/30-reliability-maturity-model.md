# 30. Reliability Maturity Model

## What You Will Learn

This chapter teaches you exactly how to prove what should improve next as your fragile agent system inevitably becomes more important to the business. You will learn to rigorously inspect every single job kind to formally assess its true maturity level, its actual business risk, its glaring missing controls, its explicit owner, and its next required review date. Finally, you will learn how to verify that reliability maturity is measured by cold, undeniable evidence, rather than hopeful ambition or impressive slide decks.

The production evidence for this chapter is a ruthless maturity scorecard that explicitly maps the risk of each job to a target maturity level, meticulously documenting current control gaps, the exact next required upgrade, the accountable owner, and the specific validation command that proves the upgrade works.

## Chapter Thread

This chapter serves as the operational reality-check in your production chain. It builds directly upon the fact that your recovery controls now explicitly show exactly which job kinds are truly, mathematically dependable. Here, we add evidence-backed maturity strictly categorized by job kind and business risk. By enforcing this discipline, this chapter directly prepares you for the advanced scaling paths that securely preserve this evidence even after the Postgres-first limits inevitably appear.

## Production Failure

Imagine a scenario where two different engineering teams both proudly declare their agents to be "production ready."

The first team's agent drafts internal, low-stakes meeting summaries. The second team's agent autonomously updates critical customer billing records. In a naive organization, both teams fill out the exact same basic deployment checklist, even though a failure in the first agent causes a minor annoyance, while a failure in the second triggers a financial compliance disaster.

What breaks here is the concept of operational rigor: maturity was treated as a generic, feel-good slogan instead of a targeted, evidence-based standard matched explicitly to business risk. A tragically false fix is to aggressively demand the heaviest possible controls for every single job, slowing development to a crawl, or conversely, to accept the same lightweight, dangerous controls for all jobs. The correct, relentlessly practical design response is to formally score each job kind by its actual risk, assigning a target maturity level, identifying existing proof, documenting missing controls, naming a specific owner, defining the next upgrade, and setting a firm review date.

## Motivation

In the unforgiving environment of production, vaguely asking "is this agent ready?" is an operationally useless question. A helpful support triage assistant, a highly privileged billing adjustment agent, and an autonomous deployment rollback agent all fundamentally demand entirely different reliability bars.

Without a rigorous maturity model, engineering teams will either overbuild controls for low-risk work or terrifyingly underbuild controls for high-risk work. This chapter meticulously maps actual job risk to concrete technical controls, demanding explicit evidence, identifying gaps, assigning owners, and scheduling the next brutal review.

## Plain Version

The simple rule for this chapter is that "maturity" simply means your system can mathematically prove more of its reliability with significantly less human guesswork. This matters deeply because operational teams absolutely need a clear way to see which controls are actually real, which are merely manual habits, and which are completely, dangerously missing. As you read, you must aggressively watch the required evidence for each maturity level across durable state, strict types, retries, observability, behavior evaluation, security boundaries, and daily operations.

## What You Already Know

Start by anchoring yourself in the hard-won architecture you have spent this entire book building. You have already introduced durable state, strictly typed boundaries, unyielding ownership, strict idempotency, heavy observability, daily operations, formal evaluation, paranoid security, and practiced recovery. You also inherently understand that different job kinds naturally carry vastly different business risks. Finally, you know that reliability work is never truly "done"; it simply needs a clear, concrete *next* upgrade.

This chapter adds the final operational lens: a formal maturity model. You will learn to map every single job kind to its current operational level, its necessary target level, its most dangerous missing control, its accountable owner, its next review date, and the specific validation evidence that proves it is safe.

## Focus Cue

Keep three critical elements fiercely in view as you read. Regarding **State**, recognize that the scorecard tracks one job kind, its target level, current evidence, specific gaps, the owner, the next upgrade, and the review date. Regarding the **Move**, understand that a job kind legally moves to a higher maturity level *only* after the mathematically required evidence for that specific level actually exists in the database or test suite. Finally, regarding **Proof**, remember that each maturity level strictly names its current proof, its glaring gaps, its next mandatory upgrade, its owner, and its review date.

If you ever get lost in the grading criteria, immediately return to state, move, and proof. They form the absolute shortest path from a theoretical maturity score to a concrete production check at 2 AM.

## Production Artifact

Before moving on from this chapter, you must build or rigorously inspect a specific artifact: a formal maturity scorecard categorized by job kind, detailing the exact gap, owner, next required artifact, and review date. This artifact matters intensely because vastly different agent jobs demand vastly different reliability bars, and that maturity must be made brutally explicit before an incident occurs. You will know this is "done" when every single capability possesses undeniable evidence, a strict target level, an accountable owner, and the specific next concrete improvement required to survive.

## Implementation Map

When you transition from reading about maturity to actual implementation, rely on this map as your guide. The primary surfaces you will interact with are your readiness scorecard, the evaluation fixtures in Appendix E, the traceability matrix in Appendix R, and your team's maturity review rows. The core state transition here is formally mapping each specific job kind to the exact reliability level it actually needs to survive its business purpose. The evidence path mathematically guarantees that every gap, owner, target level, next required artifact, and review date are made explicitly, unavoidably visible to the entire organization.

## Operator Question

Before you ship any architectural idea based on this maturity model, you must answer one vital operational question: What specific maturity level does this job kind mathematically require, and exactly what operational evidence is currently missing? To answer this, you must explicitly inspect the job-kind risk, the current level, the target level, the gap, the owner, the next artifact, and the review date. You should immediately escalate the design to leadership if the team lazily applies one single reliability bar to all agent work, or if they cannot definitively name the exact next improvement required to harden the system.

## Runtime Walkthrough

Follow the concept of maturity tracking as a single runtime pass. First, a trigger occurs when a specific job kind is formally reviewed for production maturity. Next, the action requires the engineering team to brutally compare its current, actual controls against the target reliability level dictated by its risk. For persistence, the team must permanently persist the gap, the owner, the next artifact to be built, and the review date. Finally, the check requires verifying that the next scheduled improvement is highly concrete and entirely evidence-based, rather than just a vague aspiration.

## Acceptance Gate

Do not move on until you can produce the minimum required evidence. You must be able to empirically prove that each job kind explicitly possesses a target level, a documented gap, an accountable owner, a next required artifact, and a firm review date. To validate this path, an operator must inspect the readiness scorecard, the maturity rows, Appendix E, and the strict traceability found in Appendix R. Stop the design process immediately if the engineering team cannot definitively name the exact next evidence-backed reliability improvement required for their agent.

## Mental Model

Think of operational maturity exclusively as the number of catastrophic failures the system can gracefully absorb without surprising the human operators.

Level 0 absorbs almost nothing. It is a fragile script. Level 1 gracefully absorbs hard process crashes. Level 2 securely absorbs invalid data and wild provider boundary drift. Level 3 aggressively absorbs immense operational load and live incidents. Level 4 reliably absorbs slow behavior drift and model degradation. Level 5 ultimately absorbs time, rapid team rotation, sudden provider changes, and full-scale disasters.

The maturity level is emphatically not a shiny badge for a README file. It is a brutal, mathematical statement about exactly what kinds of failure the current architectural design can actually handle.

## Level 0: Script

At Level 0, the agent is merely a script or a local notebook. There is absolutely no durable state, no automated retries, and no formal audit trail. Reruns are entirely manual, and terrifying provider errors are visible only if someone happens to be staring at the terminal logs. This level is perfectly fine for initial learning and fast prototyping. It is absolutely, unequivocally not a production system.

## Level 1: Durable Jobs

At Level 1, the agent finally possesses a durable job ledger. The system now formally respects pending, running, succeeded, and failed states. It enforces a strict idempotency key, utilizes active lease ownership, follows a defined retry policy, gracefully handles dead-letter states, and writes an immutable event timeline. At this level, hard crashes finally become safely recoverable. Operators can query the database and definitively answer what actually happened to a job after the process died.

## Level 2: Typed Boundaries

At Level 2, the agent enforces strict domain types and explicit architectural boundaries. The system now demands typed payloads and results, typed error classes, a formal provider adapter, heavily typed tool schemas, rigorous policy results, and explicitly versioned rows. At this level, mathematically invalid states are significantly harder to express in code, and old, historical work can finally be replayed or safely inspected long after the release that created it is gone.

## Level 3: Operated Service

At Level 3, the agent has graduated to SRE-grade operation. The system now actively measures SLIs and SLOs, triggers burn-rate alerts, strictly enforces capacity controls, relies on mechanical backpressure, maintains actionable runbooks, follows a formal incident process, demands explicit release gates, and gracefully handles process shutdown. At this level, the engineering team can confidently run the system under massive load and respond predictably when it inevitably misbehaves at 3 AM.

## Level 4: Behavior-Gated Agent

At Level 4, the agent finally implements strict behavior reliability controls. The system relies on rigid fixture evaluations, continuous historical shadow evaluations, formal prompt and model release receipts, aggressive policy gates, mandatory human approval samples, active drift detection, and post-incident regression cases. At this level, the team can aggressively change prompts, models, and tools with mathematical confidence, rather than simply deploying and hoping for the best.

## Level 5: Long-Horizon Platform

At Level 5, the agent platform is designed to seamlessly support years of uninterrupted operation. The system enforces multi-version compatibility, mandates routine restore drills, maintains a living security threat model, strictly isolates tenants, tightly governs tool capabilities, actively monitors cost governance, relentlessly automates toil, seamlessly handles ownership rotation, and strictly enforces a deprecation policy. At this level, reliability is no longer dependent on one senior engineer's memory. It is permanently built into the system architecture, the tests, the runbooks, and the team's relentless operating rhythm.

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

I call this **Risk-Proportional Engineering**. It stops teams from "Over-Engineering" the easy stuff and "Under-Engineering" the hard stuff. It is perfectly fine to have a Level 1 "Internal Experiment" and a Level 5 "Payment Agent" in the same codebase.

For a low-risk internal summarizer, the next upgrade may be typed output and
... (omitted) ...
visible dead-letter state. If it fails, users wait or receive a draft that can be
regenerated. That is still worth fixing, but it is not the same risk as a
billing agent that can create external financial changes.
For a billing agent, the next upgrade may be side-effect receipts, approval
evidence, and a rollback or compensation path. If that agent repeats an action,
the business may charge a customer twice or issue the wrong credit. The maturity
model forces the team to see that difference before saying both agents are
"ready."

We should also track **Human-AI Collaboration Maturity**. Moving from Level 4 to Level 5 is often about how well the human team and the agent work together over years—how the team learns from agent failures and how the agent context is refined by human judgment.

This is why maturity is per job kind. A product can have one Level 2 job kind,
... (omitted) ...
one Level 4 job kind, and one job kind that should stay at Level 0 because it is
only a research script. That is not inconsistency. It is honest engineering.

> ### 🎓 The Professor's Corner
>
> **The Karate Belt of Code: Maturity means Proof**
>
> Think of maturity levels like **Karate Belts**. You don't get a Black Belt just because you want one! You have to go to the Dojo and prove you know the moves. 
> 
> Level 1 is a White Belt—you know how to stand up (durable state). Level 5 is a Black Belt—you can handle a fight in the dark (disaster recovery)! It makes your progress feel like a personal achievement, not just a checklist.

> ### 🎓 The Professor's Corner
>
> **The Reliability Volume Knob: Not a Switch**
>
> Reliability isn't a simple "On/Off" switch. It’s more like a **Volume Knob**! 
> 
> For a quiet conversation (low-risk job), you can turn the volume down (basic controls). But for a rock concert (high-risk financial action), you have to turn the volume all the way up! The maturity model helps you decide exactly how loud your reliability needs to be.

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

This walk through also reinforces the core of **Distributed Authority**. A "Drafting" agent (Level 4) only produces **Soft State** (a recommendation), while an "Executing" agent (Level 5) produces **Hard State** (a financial transaction). The maturity model correctly forces the "Hard State" agent to carry the heavier burden of proof.

This also makes planning easier. The team does not need to "make the platform
reliable" in one vague push. It can say: the summarizer needs typed output
validation this week; refund drafting needs a new evaluation slice before the
next prompt release; refund execution needs receipt-backed replay before it can
leave human-supervised mode. Each improvement is small enough to build and clear
enough to review.

I call this a **"Growth Mindset"** for architecture. It’s okay to be at Level 1 today, as long as you have a plan to get to Level 2 tomorrow. It takes the **"Imposter Syndrome"** away and replaces it with a clear journey toward expert engineering.

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

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
