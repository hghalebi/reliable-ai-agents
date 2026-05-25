# 24. Incident Response And Postmortems

## What You Will Learn

This chapter teaches you to:

- explain how the team responds, mitigates, learns, and changes the system after failure;
- inspect incident states, timelines, customer impact, mitigation evidence, postmortem actions, and owners;
- verify that incidents produce durable improvements, not blame or vague notes.

The production evidence is an incident record with a timeline, impact, root
cause, corrective invariant, action items, and follow-up checks.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** runbooks provide safe action paths under pressure.
- **Adds:** incident handling as failed-invariant repair.
- **Prepares:** release engineering that reduces repeated incident risk.

## Production Failure

An agent sends an unsafe recommendation after a model change.

The team mitigates quickly, but the timeline lives in chat, the failed invariant
is never named, and the same class of release ships again later.

- **What breaks:** the incident ended without changing the system.
- **False fix:** blame the model and add a reminder to be more careful next
  time.
- **Design response:** record impact, timeline, mitigation, root cause,
  corrective invariant, owners, and verification so the next release cannot
  repeat the same failure quietly.

## Motivation

In production, agent incidents are not only outages. They can be retry storms, unsafe approvals, provider drift, tool misuse, bad memory, or behavior regression.

Without incident discipline, teams fix symptoms and lose the evidence needed to improve the system. This chapter connects triage, mitigation, postmortems, and action items to the durable ledger.

## Plain Version

Read this as the simple version:

- **Simple rule:** Incident response keeps users safe now and improves the system after the facts are known.
- **Why it matters:** Agent incidents often involve partial work, model behavior, external tools, approvals, and unclear ownership.
- **What to watch:** Watch timeline evidence, impact, mitigation, root cause, corrective invariant, and follow-up ownership.

## What You Already Know

Start with these anchors:

- Runbooks handle known operational conditions.
- Incidents begin when an invariant is broken, a promise is missed, or the runbook is incomplete.
- Learning must change the system, not only the meeting notes.

This chapter adds: incident response and postmortems. You will record impact,
timeline, mitigation, root cause, corrective invariant, owner, and follow-up
evidence.

## Focus Cue

Keep three things in view:

- **State:** detected invariant failure, impact, mitigation, preserved evidence, postmortem finding, and follow-up action.
- **Move:** an incident moves from detection to learning without destroying the evidence needed to prevent recurrence.
- **Proof:** Triage facts, mitigation notes, preserved evidence, action items, owners, and regression tests are recorded.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** an incident timeline and postmortem template driven by durable events and corrective invariants.
- **Why it matters:** incident review should improve the system, not only explain what people did.
- **Done when:** each incident names impact, evidence, failed invariant, mitigation, owner, and a tested prevention action.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** failure drill code, audit/event tables, escalation rows, and postmortem template.
- **State transition:** turn incident evidence into a timeline, failed invariant, mitigation, and prevention action.
- **Evidence path:** a postmortem names impact, evidence, owner, corrective invariant, and validation command.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which failed invariant explains the incident, and what tested change prevents recurrence?
- **Evidence to inspect:** incident timeline, impact, evidence links, failed invariant, mitigation, owner, and regression test.
- **Escalate if:** the postmortem records blame or chronology without a system change and validation evidence.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** an incident is declared.
2. **Action:** build the timeline from durable evidence and identify the failed invariant.
3. **Persistence:** persist impact, mitigation, owner, action item, and validation evidence.
4. **Check:** verify the prevention change is tested and assigned.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** incident review identifies failed invariant and tested prevention.
- **Validation path:** inspect timeline evidence, mitigation, owner, action item, and failure-drill tests.
- **Stop if:** postmortem actions are not tied to system changes or validation.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, agent incidents are not only outages
rule: Incident response keeps users safe now and improves the system after the facts are known
tiny example: detected invariant failure, impact, mitigation, preserved evidence, postmortem finding, and follow-up action
artifact: an incident timeline and postmortem template driven by durable events and corrective invariants
proof: incident review identifies failed invariant and tested prevention
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

An incident is a failed invariant with user or operator impact. The response
loop is:

```text
detect -> understand -> reduce harm -> preserve evidence -> learn -> harden
```

The order matters. A mitigation that destroys evidence may make the current
dashboard look better while making recurrence more likely.

For agents, this is more subtle than ordinary uptime. The service may still
respond while producing unsafe recommendations, bypassing approval, or silently
dropping evidence. Treat those as incidents because they violate the reliability
contract even when latency looks fine.

## Incident Lifecycle

Use a repeatable lifecycle:

```text
detection
triage
mitigation
communication
recovery
postmortem
follow-up
```

The lifecycle is useful because it separates different kinds of thinking. In
detection, the team is asking whether a promise has been broken. In triage, the
team is trying to understand the shape of the break. In mitigation, the team is
reducing harm before every cause is fully known. In postmortem work, the team is
turning what it learned into a stronger system.

These phases often overlap in real life, but the distinction still matters. If
the team jumps straight from detection to explanation, it may leave users exposed
while people debate causes. If the team jumps from mitigation to closure, it may
restore the dashboard while preserving the same weak invariant that caused the
incident. A serious agent system needs both urgency and memory.

For long-running agents, the incident lifecycle also protects evidence. A retry
storm, unsafe tool action, or prompt regression may leave partial rows across
jobs, tool calls, approvals, audit events, and provider logs. If responders patch
those rows by hand without recording what changed, the system loses the very
facts needed to learn. The goal is not only to recover. The goal is to recover in
a way that leaves a trustworthy story behind.

## Tiny Incident

Suppose dead jobs spike after a prompt release.

Do not start by replaying all dead jobs. First ask:

```text
which prompt_version changed?
which job kinds died?
which error class is dominant?
did approval behavior change?
can one low-risk job be replayed safely after rollback?
```

The incident is not "dead jobs exist." The incident is that a release changed a
production behavior contract.

The first useful move is to narrow the failed invariant. Did the release break
output shape, provider compatibility, policy routing, evaluation coverage, or
retry classification? Each answer points to a different mitigation.

Read the tiny case as:

```text
setup: dead jobs spike after a prompt release
transition: response preserves evidence, mitigates safely, and turns learning into corrective work
evidence: incident timeline, impact, failed invariant, mitigation, owner, and follow-up action exist
invariant: incidents must improve the system instead of only explaining the past
```

## Worked Walkthrough

Imagine a support-triage agent that classifies customer tickets, drafts a reply,
and sometimes opens a refund request for human approval. The agent has run well
for weeks. On Monday morning, a new prompt version is released. Thirty minutes
later, the dead-letter count rises and the approval queue looks strangely small.
At first, this may look like two separate problems. It is safer to treat it as
one incident until evidence proves otherwise.

The responder starts with impact, not blame. Which tenants are affected? Which
job kind is failing? Did customers receive wrong replies, or did work stop before
side effects happened? The queue metrics show that only `support_refund_review`
jobs are affected. The timeline shows that the first failures began four minutes
after prompt version `support-refund-v18` became active. That does not prove the
prompt caused the incident, but it gives the team a concrete boundary for the
investigation.

Next, the responder checks the failed invariant. The system promised that a
refund request cannot bypass typed validation and human approval. The dead jobs
show malformed model output for the `refund_amount` field. The approval queue is
smaller because invalid requests are rejected before approval creation. This is
good news: the safety boundary held. The incident is still real, because the
agent stopped processing a real workflow, but the failure mode is contained.

Now mitigation can be precise. The team rolls the prompt route back to
`support-refund-v17`, pauses only the affected job kind, and keeps evidence
collection on. It does not delete dead jobs. It does not clear errors. It does
not manually create approval rows from malformed output. After rollback, one
low-risk job is replayed with the same idempotency key and receipt checks. When
that succeeds, the worker concurrency is restored gradually.

The postmortem is where the incident becomes system improvement. The failed
invariant was not "the model made a mistake." Models will keep making mistakes.
The deeper weakness was that the release gate did not include enough malformed
refund examples for the new prompt. The corrective action is therefore a new
evaluation slice, a release-gate blocker for refund schema regressions, and a
runbook query that shows approval creation rate beside dead-letter rate after a
prompt release.

Notice the shape of the learning. The team did not merely say, "Be careful with
refund prompts." It changed the system so the next bad prompt has a harder time
reaching production. That is the difference between a meeting note and an
engineering repair.

## From Chat Evidence To Durable Evidence

During an incident, chat is useful for coordination, but it is a poor system of
record. Chat answers what people were saying. It does not reliably answer which
jobs were affected, which prompt version was active, which worker owned a lease,
which approvals were skipped, which tool calls executed, or which mitigation
changed state.

The durable ledger answers those questions. The incident record should point to
job ids, trace ids, release ids, prompt versions, model routes, audit events, and
operation events. The postmortem can summarize the story in human language, but
the claims should be backed by rows that existed before the meeting. This keeps
the review honest when memory is incomplete or emotions are high.

There is a practical rule here: if a fact changes the incident conclusion, it
should be backed by durable evidence. "The provider was slow" needs latency and
error data. "No side effect happened" needs tool-call receipts or the absence of
executed tool-call rows. "Rollback fixed it" needs a timeline showing the
rollback and the recovery signal. This discipline protects the team from writing
postmortems that feel plausible but do not prove the repair.

## Triage

Start with facts, not guesses:

```text
which SLO is burning?
which job kinds are affected?
when did the first bad event appear?
which provider/model route is involved?
which worker build is processing the work?
are failures retryable, permanent, or policy-related?
```

Use the queue and timeline queries:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql}}
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/job_event_timeline.sql}}
```

## Escalation Evidence

Some incidents should not remain inside the worker loop. They need a durable
human escalation:

```text
deadline breach -> page the on-call owner
repeated permanent failure -> create a ticket for the job owner
security signal -> escalate to the security reviewer
approval timeout -> escalate to the responsible product or operations owner
compatibility risk -> route to the release owner
```

An escalation record should preserve:

```text
escalation kind
severity
target job or run
reason
assigned owner
acknowledgement time
resolution time
```

That record is not a substitute for mitigation. It is the ownership trail that
keeps unresolved risk from disappearing into chat history.

## Mitigation

Good mitigations reduce user harm without destroying evidence:

```text
pause one job kind
reduce worker concurrency
switch provider route
raise backoff for retryable failures
disable risky side-effect execution
recover expired leases
communicate delayed processing
```

Bad mitigations hide the problem:

```text
delete dead jobs
clear last_error
turn off events
drop approval requirements
manually patch status without recording why
```

## Postmortem

A blameless postmortem should answer:

```text
What was the user-visible impact?
Which SLOs burned?
What signals detected it?
Which signals were missing?
Which invariant failed?
Why did the system not recover automatically?
What will prevent recurrence?
```

For agent systems, include model/provider context:

```text
prompt_version
model_route
tool_version
policy_version
worker_build_id
provider error class
approval decision path
```

## Action Items

Good action items change the system:

```text
add an alert
add a constraint
add a typed boundary
add a runbook command
automate a repeated manual step
add a regression test
add a provider fallback
```

Weak action items only ask people to be more careful.

The postmortem should improve the system's ability to answer the same incident
next time. That usually means better evidence, a stricter boundary, or a safer
control surface.

## Formal Definition

For this chapter, the precise definition is:

```text
Incident response is the controlled reduction of harm after an invariant fails; a postmortem turns the failed invariant into stronger system evidence.
```

In the book's system model:

- **State:** detected invariant failure, impact, mitigation, preserved evidence, postmortem finding, and follow-up action.
- **Actor:** incident responders reduce harm, and the team converts lessons into tests, runbooks, constraints, or automation.
- **Transition:** an incident moves from detection to learning without destroying the evidence needed to prevent recurrence.
- **Evidence:** Triage facts, mitigation notes, preserved evidence, action items, owners, and regression tests are recorded.
- **Invariant:** postmortems improve the system rather than only explaining the outage.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Incidents end when service recovers. |
| Production symptom | The same failure recurs because no invariant or owner changed. |
| Corrective invariant | An incident produces timeline evidence, mitigation, root cause, action item, and owner. |
| Evidence to inspect | Postmortems link failed invariant to regression tests, runbook changes, or design changes. |


## Production Contract

Incident response is strong when:

```text
alerts point to a violated SLO or invariant
triage starts from durable state and event timelines
mitigations reduce harm without deleting evidence
postmortems name the failed invariant
action items become tests, constraints, runbooks, or automation
```

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Incidents end when service recovers. | Ending an incident at service recovery loses the failed invariant and the learning opportunity. |
| Safer version | An incident produces timeline evidence, mitigation, root cause, action item, and owner. | The incident record links symptom, impact, timeline, mitigation, root cause, owner, and action item. |
| Production version | Postmortems link failed invariant to regression tests, runbook changes, or design changes. | Postmortems produce regression tests, runbook changes, or design repairs tied to a concrete invariant. |

Use the naive row when recovery is mistaken for completion. Use the safer row to preserve evidence. Use the production row before the same failure returns.

## Testing Strategy

Test incident review as invariant repair:

- **Unit or type test:** prove Rust incident or drill records reject missing severity, owner, mitigation, timeline, root cause, or action-item evidence.
- **Persistence or boundary test:** prove Postgres audit, operation, failure-history, escalation, and timeline rows can reconstruct the incident without relying on memory.
- **Regression test:** take one postmortem action item and require a new negative test or runbook query before the incident can be marked closed.

## Observability Strategy

Observe incidents from symptom to repair:

- Emit structured `tracing` fields for incident id, severity, affected job kind, failed invariant, mitigation owner, action item, and trace id.
- Record an operation event when triage starts, mitigation changes state, evidence is preserved, escalation is assigned, or a postmortem action closes.
- The runbook query should connect customer impact, failed jobs, failure history, mitigations, and follow-up tests in one reviewable timeline.

## Security and Safety Considerations

Incident response should reduce harm without creating new unsafe actions:

- Treat incident notes, provider errors, logs, screenshots, and customer examples as untrusted sensitive evidence until reviewed and minimized.
- authorization, sandboxing, and approval still apply during mitigation, manual replay, compensation, and emergency access.
- Redact personal data, secrets, and raw prompts from postmortems while preserving failed invariant, impact, owner, action item, and evidence links.

## Operational Checklist

Use this checklist before relying on incident response and learning loops in production:

- **State:** Incidents move through detection, triage, mitigation, recovery, postmortem,
  action item, and verification states.
- **Boundary:** Incident notes are linked to durable job, trace, release, policy, and
  receipt evidence before conclusions are trusted.
- **Failure:** The team can separate symptom, cause, impact, mitigation, and follow-up
  without rewriting history.
- **Observability:** Incident timelines connect alerts, runbook query outputs, trace
  ids, job states, deploys, and customer impact.
- **Safety:** Emergency actions preserve approval, audit, redaction, tenant isolation,
  and replay safety.

## Exercises

1. Write a negative test where an incident mitigation replays jobs without checking
   idempotency receipts or approval state. Explain which idempotency key, receipt, or
   state transition prevents duplicate work.
2. Sketch the Postgres evidence: incident_events, affected jobs, release evidence,
   receipt status, and postmortem action rows.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   IncidentState, MitigationAction, ImpactWindow, and PostmortemActionItem types. Then
   name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What makes an incident different from an expected runbook condition?
- Explain: Why should a postmortem name a corrective invariant?
- Apply: Respond to a spike in dead jobs after a prompt release.
- Evidence: Preserve the timeline, impact, failed invariant, mitigation, action item, owner, and follow-up check.

## Summary

Incident response is part of the design. The ledger, events, versions, and runbooks exist so the team can mitigate quickly and learn accurately.

- **Invariant:** an incident timeline separates detection, impact, cause, mitigation, recovery, and follow-up without losing evidence.
- **Evidence:** alerts, runbook outputs, trace ids, job state, releases, approvals, receipts, and postmortem action items link the story.
- **Carry forward:** good incidents produce system changes, not only better memories.

## Changed Understanding

- **Before this chapter:** an incident looked like a failure to fix quickly.
- **After this chapter:** incident response protects users during failure, and postmortems turn evidence into stronger system invariants.
- **Keep:** preserve the incident timeline, user impact, mitigation evidence, and invariant-changing follow-up.

## Further Reading and Sources



- [John Allspaw: Blameless Postmortems](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: (2012). The definitive industry guide to moving from "who failed" to "what failed." it introduces the "Second Story"—the complex systemic conditions that allowed an agent or operator to fail.
- [James Reason: The Swiss Cheese Model](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: A foundational safety science model explaining how accidents occur only when "latent conditions" (holes in the cheese) in multiple layers (code, tests, policy, human review) align.
- [Erik Hollnagel: Safety-II](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: Shifting the focus from why systems fail (Safety-I) to why they usually succeed (Safety-II). It motivates the "Work-as-Done" evidence captured in this chapter's timelines.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: The industry standard for turning incidents into "Corrective Invariants" and ensuring that follow-up actions lead to durable system improvements.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann). Connects postmortem findings to the formal limits of distributed state and the "Metastable Failure" patterns discussed in this chapter.