# 8. Production Hardening

## What You Will Learn

This chapter teaches you to:

- explain which controls turn a working demo into a service real users can depend on;
- inspect where idempotency, leases, approvals, observability, secrets, and deployment rules attach to the job lifecycle;
- verify that every risky transition has a guard and evidence.

The production evidence is a hardened job system with durable state, typed
boundaries, retry rules, approval gates, traceable events, and secret handling.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the local system can run the core loop.
- **Adds:** the first hardening controls: idempotency, leases, approvals, observability, retries, and secrets discipline.
- **Prepares:** failure-mode analysis that identifies missing invariants.

## Production Failure

The local agent works for one user, then production traffic adds duplicate
requests, slow tools, secret handling, risky actions, and unknown failures.

The happy path still works, but every edge now needs evidence.

- **What breaks:** a working demo was treated as a hardened service.
- **False fix:** add a few broad `try/catch` blocks, more logs, and a manual
  restart habit.
- **Design response:** attach idempotency, leases, approvals, retries,
  observability, secret discipline, and operator controls to the state
  transitions that can hurt users.

## Motivation

In production, demos fail at the edges: duplicate requests, crashes, risky actions, unknown errors, secrets, and missing operator evidence. Hardening is the work of closing those gaps.

Without hardening controls, a system can appear successful until the first retry storm, incident, or audit. This chapter maps the controls that turn a local worker into a service that can survive real users.

## Plain Version

Read this as the simple version:

- **Simple rule:** Hardening means turning assumptions into explicit controls and evidence.
- **Why it matters:** Agent systems fail when timeouts, permissions, limits, and error paths are left as informal expectations.
- **What to watch:** Watch every external call, side effect, approval, and retry for a visible guardrail and a recorded outcome.

## What You Already Know

Start by anchoring yourself in the hard-won mechanics you have already established. You know that your local job lifecycle can successfully run and be thoroughly tested in isolation. You know that moving to production immediately introduces duplicate intake requests, brutal process crashes, risky external actions, terrifying unknown failures, and highly sensitive secrets. Finally, you understand that a control is only actually useful if it strictly attaches to a mathematical state transition, rather than just floating in a log message.

This chapter adds the crucial first hardening layer. By deliberately applying idempotency, explicit leases, strict approvals, heavy observability, and paranoid secret handling, you turn a fragile local system into a serious, production-grade service shape.

## Focus Cue

Keep three critical elements fiercely in view as you read. Regarding **State**, recognize the specific controls that actively turn hidden, silent failures into durable state, policy decisions, metrics, traces, or actionable runbook alerts. Regarding the **Move**, understand that a cute demo behavior only legally becomes production behavior *after* its specific failure path is formally persisted, gated, observed, or repeatedly rehearsed. Finally, regarding **Proof**, remember that idempotency keys, leases, approval gates, observability pipelines, secret handling, and recovery paths must be glaringly visible in both your code and your operational runbooks.

If you ever get lost in the list of features, immediately return to state, move, and proof. They form the absolute shortest path from a theoretical hardening idea to a concrete production check at 2 AM.

## Production Artifact

Before moving on from this chapter, you must build or rigorously inspect a specific artifact: a comprehensive hardening checklist explicitly wired to idempotency, leases, approvals, secrets, traces, and failure visibility. This artifact matters intensely because a demo only truly becomes a service when heavily repeated work, risky actions, and sudden crashes have mathematically enforced controls. You will know this is "done" when the system automatically rejects unsafe requests, meticulously records key transitions, and loudly exposes stuck or risky work long before launch day.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** idempotency, lease, approval, config, secret, tracing, and audit modules.
- **State transition:** add controls that turn a demo path into a bounded production service.
- **Evidence path:** unsafe repetition, risky action, missing secret, and stuck work have explicit rejection or evidence.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which control prevents this demo path from becoming an unsafe production path?
- **Evidence to inspect:** idempotency record, lease, approval request, secret config, trace id, and failure event.
- **Escalate if:** repeating, crashing, or approving the path changes behavior without durable evidence.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a demo path is about to serve real users.
2. **Action:** add idempotency, leases, approval, secrets, tracing, and failure visibility.
3. **Persistence:** persist the control evidence for each risky boundary.
4. **Check:** verify repeat, crash, overload, and risky-action paths are bounded.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** repeat, crash, risky-action, secret, and stuck-work paths have controls.
- **Validation path:** inspect idempotency, lease, approval, config, tracing, and audit evidence.
- **Stop if:** the path is safe only when users behave once, providers respond, and workers never crash.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, demos fail at the edges: duplicate requests, crashes, risky actions, unknown errors, secrets, and missing operator evidence
rule: Hardening means turning assumptions into explicit controls and evidence
tiny example: the controls that turn hidden failure into durable state, policy decisions, metrics, traces, or runbook actions
artifact: a hardening checklist wired to idempotency, leases, approval, secrets, traces, and failure visibility
proof: repeat, crash, risky-action, secret, and stuck-work paths have controls
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Worked Walkthrough

The prototype works once.
That is not the same as production readiness.
Production begins when the same workflow must survive bad timing, bad inputs, repeated requests, deploys, and tired humans debugging an incident.

Take a document-processing agent.
In the demo, it reads a document, asks the model for a summary, and stores the result.
In production, each of those verbs hides a boundary.
The document may be missing, too large, private, or from another tenant.
The model may return malformed output.
The worker may crash after saving the summary but before marking the job complete.

Hardening means naming those failure points before they happen.
The input gets validation.
The job gets durable state.
The tool call gets an idempotency key.
The model output gets parsed and checked.
The operation gets a trace id.
The business decision gets audit evidence.

None of this is decorative infrastructure.
It is how a team changes a demo into a system that can be trusted with real users.
The model is still useful, but the surrounding software now decides what is allowed, what is remembered, and what can be recovered.

This is what I call **Behavioral Hardening**. We aren't just making the code "not crash"; we are making the agent's behavior reliable. For example, if the model's confidence in an answer is low, the system should treat that as a "risky action" and trigger an approval gate. We frame the **Approval Gate** as a safety control for **Model Uncertainty**, not just a human-in-the-loop feature.

## Mental Model

Production hardening is not the act of adding many features. It is the act of
closing the places where failure can hide:

```text
duplicate request -> idempotency
crashed worker -> lease and recovery
risky action -> approval gate
unknown failure -> event and metric
temporary outage -> classified retry
secret exposure -> redaction and secret storage
```

Each hardening control turns an ambiguous failure into state that the system can
inspect, retry, pause, or escalate.

The order matters. Add durability and idempotency before scaling workers. Add
leases before increasing concurrency. Add approval before connecting risky
tools. Add observability before depending on SLOs. Hardening is a dependency
ladder, not a bag of optional features.

## Tiny Example

Imagine the same incident-triage request arrives twice because the API caller
times out and retries:

```text
request 1: idempotency_key = deploy-742
request 2: idempotency_key = deploy-742
```

A script sees two calls. A production ledger sees one logical job. That is the
difference between a retryable client error and duplicated operational work.

Read the tiny case as:

```text
setup: the same caller retries one incident-triage request
transition: idempotency resolves both arrivals to the same durable job
evidence: one job row, one idempotency key, one event timeline, and no duplicate side effect
invariant: retrying the caller must not multiply work or external action
```

## Idempotency

Every externally triggered job should have an idempotency key:

```text
same request -> same job
```

This protects against duplicate API submissions and retry storms.

The key idea is simple: the client may repeat the request, but the system should
recognize the work.

Without idempotency, the API cannot distinguish a distinct business request from
a retry of the same request after a timeout. For an agent that only drafts text,
that may look harmless. For an agent that sends messages, updates cases, pauses
jobs, or calls business tools, duplicate work can become duplicate side effects.

The database should enforce this rule with a durable key, not with a local cache
or a best-effort check in memory. A unique idempotency record gives the system a
stable answer after restarts, retries, deploys, and concurrent requests.

The operator should be able to inspect one fact: both arrivals resolved to the
same logical operation.

## Leases And Heartbeats

A worker should never own a job forever just because it once started it.

```text
lease active -> current worker owns the job
lease extended -> long operation is still alive
lease expired -> another worker may recover the job
```

Heartbeats are not a performance feature. They are the proof that a long run is
still alive.

A lease is temporary authority.

That temporary authority is what lets multiple workers cooperate without
trusting each other blindly. One worker owns the job for a bounded time. If it
keeps heartbeating, the system extends that ownership. If it disappears, another
worker can recover the job after the lease expires.

This matters for long-running agents because waiting is normal. The worker may
wait on a model call, a tool call, a retrieval step, or a human approval. During
that wait, the system needs evidence that the worker is still alive and still
authorized to write the next transition.

The dangerous case is not only a crashed worker. It is also a stale worker that
returns late and tries to write success after another worker has recovered the
job. Lease checks prevent that stale write.

## Approval Gates

The agent should propose risky actions, not execute them.

```text
AgentResult {
  summary,
  next_action,
  approval
}
```

Then a separate approved action worker performs side effects.

An approval gate is a control surface.

It turns "the model thinks this is a good action" into "the system has permission
to perform this action." Those are different claims. A model can draft an email,
recommend a CRM update, or suggest pausing a workflow. The product still decides
whether the action is allowed, whether a human must approve it, and what evidence
must be recorded.

This is a formal **Happens-Before** relationship. An approval must *happen-before* the execution, and the evidence of that relationship must be stored in the ledger. Without this durable proof, the execution has no authority.

This separation keeps autonomy scoped. Low-risk actions may proceed under policy.
High-risk actions may wait for a reviewer. Regulated actions may require
additional evidence. The approval gate gives the system a place to express those
differences without burying them in prompt wording.

The approval record should say who or what approved the action, which policy
version applied, what was approved, and which side effect was later executed.

## Observability

Keep both:

```text
current state in agent_jobs
append-only events in agent_job_events
```

Current state is for dashboards. Events are for debugging and audits.

> ### 🎓 The Professor's Corner
>
> **Logging vs. Eventing: The Shout vs. The Diary**
>
> Think of a **Log** as a "Shout" into the wind. Someone might hear it, but once it's said, it's gone quickly. If you're not listening at that exact second, you miss it!
> 
> An **Event** is like a "Diary Entry." It's a permanent record that tells the story of what happened. In production, you don't just want to "shout" that an error happened; you want a "diary" that records every step of the journey so you can read it later when things go wrong!

These two views answer different questions.

The current row answers, "What should the system do next?" It helps workers find
pending jobs, operators find stuck work, and dashboards show current health. The
event timeline answers, "How did we get here?" It helps engineers debug
incidents, auditors reconstruct decisions, and product teams understand whether
the agent behaved as designed.

Logs alone are not enough. A log line may be dropped, unstructured, or hard to
correlate. Production observability needs stable identifiers: job id, trace id,
worker id, idempotency key, approval id, model version, prompt version, and
failure class.

If an operator cannot connect the API request, job row, worker attempt, tool
call, model result, and final state, the system is observable only in name.

## Error Classification

A retry loop without classification is dangerous:

```text
retryable: timeout, rate limit, provider unavailable
permanent: invalid payload, missing secret, policy rejection
```

The worker should not keep retrying a job that can never succeed without human
or configuration change.

The word "error" is too broad for production decisions.

A timeout is different from invalid input. A provider outage is different from a
policy rejection. A missing API key is different from malformed model output. If
the worker treats all of them as "try again later," it creates noise, burns
budget, hides real configuration problems, and can delay human response.

In AI, we also have **Reasoning Failure**. If the model refuses to answer a safe prompt or produces nonsensical results repeatedly, retrying won't help. This needs prompt engineering or model switching, not a simple retry loop.

Classifying errors turns failure into a decision. Transient failures can retry
with backoff. Permanent failures can stop quickly. Operator-action failures can
escalate. Policy failures can become review evidence. Unknown failures can be
dead-lettered until a human understands the pattern.

The goal is not to predict every failure perfectly. The goal is to avoid one
anonymous failure bucket that drives every operational response.

## Secrets

Do not store API keys in job payloads or events.

Use environment variables or a secret manager. If a secret-like value enters an
error message, redact it before writing `last_error`.

> ### 🎓 The Professor's Corner
>
> **Redaction: The Black Marker Rule**
>
> Think of **Redaction** like using a "Black Marker" on a sensitive document. You still have the paper, but the bad guys can't see the password! 
> 
> In our system, if an error message contains a secret API key, we "draw a black box" over it before saving it to the database. This lets the operator see *where* the error happened without seeing the *secret* that caused it. It's the secret to keeping your system safe while still being able to fix it!

Secrets become harder to protect once they enter durable evidence.

Job payloads, events, traces, and audit records are designed to survive. That is
good for reliability and dangerous for credentials. If an API key is written into
an event payload, it may be copied into backups, exported into support tooling,
indexed by logs, or shown in a dashboard.

The safer pattern is to store references and decisions, not secret values. A job
may record that a required credential was missing, expired, or denied. It should
not record the credential itself. Debug output should preserve enough information
to diagnose the failure without giving future readers access to the secret.

This is also a model-safety rule. The model should not see credentials unless a
tool boundary explicitly requires a scoped, redacted, or delegated access path.

## Formal Definition

For this chapter, the precise definition is:

```text
Production hardening is the addition of controls that turn hidden failure into state, evidence, policy, or operator action.
```

In the book's system model:

- **State:** the controls that turn hidden failure into durable state, policy decisions, metrics, traces, or runbook actions.
- **Actor:** the engineer adds controls and the operator inspects the evidence when the system is stressed.
- **Transition:** a demo behavior becomes production behavior only after its failure path is persisted, gated, observed, or rehearsed.
- **Evidence:** Idempotency, leases, approval gates, observability, secret handling, and recovery paths are visible in code and runbooks.
- **Invariant:** important failure cannot disappear into process memory, model prose, or ad hoc operator judgment.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Hardening controls are treated as later polish. |
| Production symptom | Duplicate intake, lost ownership, unapproved side effects, and invisible failures appear together. |
| Corrective invariant | Idempotency, leases, approval, observability, error classification, and secrets are core controls. |
| Evidence to inspect | Schema, worker tests, runbook queries, and configuration boundaries show each control. |


## Production Contract

After hardening, every serious failure should land in one of four places:

```text
state:
  the current job status tells operators what can happen next

event:
  the timeline explains how the job reached that state

metric:
  dashboards and alerts show whether this is isolated or systemic

policy:
  risky action is stopped until a human or rule approves it
```

If a failure lands only in process memory or only in provider text, it is still
not production-grade.

This contract keeps hardening practical.

You do not need a separate platform for every failure. You need each important
failure to become inspectable in one of these places. If the current state is
clear, workers can continue safely. If the event history is clear, humans can
reconstruct what happened. If the metric is clear, the team can see whether the
problem is isolated or systemic. If the policy result is clear, risky actions do
not depend on model confidence alone.

Hardening fails when a problem lives nowhere durable. A crash known only to a
dead process, a denial known only to a prompt, or a duplicate known only to a
client timeout cannot support reliable operation.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Hardening controls are treated as later polish. | Treating hardening as later polish lets unsafe side effects and weak evidence enter the first design. |
| Safer version | Idempotency, leases, approval, observability, error classification, and secrets are core controls. | Idempotency, leases, approvals, secrets, observability, and recovery become first-version controls. |
| Production version | Schema, worker tests, runbook queries, and configuration boundaries show each control. | Every hardening mechanism points to code, SQL, tests, and runbook evidence before deployment. |

Use the naive row when controls are deferred. Use the safer row to put safety into the MVP. Use the production row when a launch review asks for proof.

## Testing Strategy

Test each hardening control as a first-version invariant:

- **Unit or type test:** prove Rust types reject missing idempotency, invalid leases, unclassified failures, unsafe approval state, and malformed secret references.
- **Persistence or boundary test:** prove Postgres rows and queries expose idempotency, lease ownership, approval, operation events, and recovery evidence together.
- **Regression test:** preserve one shortcut for each control, such as retry without idempotency or side effect without receipt, and require the gate to fail.

## Observability Strategy

Observe each hardening control as evidence, not a checklist item:

- Emit structured `tracing` fields for job id, control name, idempotency key, lease owner, approval id, trace id, and failure class.
- Record an operation event when idempotency, lease ownership, approval, secret validation, error classification, or recovery control accepts or blocks work.
- The runbook query should show which hardening control prevented an unsafe retry, stale mutation, unapproved action, or unobservable failure.

## Security and Safety Considerations

Hardening is the act of turning hidden trust into explicit gates:

- Treat every newly introduced control as untrusted until tests show it blocks the unsafe path it was added for.
- authorization, sandboxing, approval, idempotency, lease checks, and secret handling should be visible before the system calls external tools.
- Redact secrets and sensitive payloads from hardening evidence while preserving which control allowed, denied, retried, or escalated work.

## Operational Checklist

Use this checklist before relying on the controls that turn a demo into a service in production:

- **State:** Every hardening control has an owner, state, evidence surface, and release
  gate.
- **Boundary:** Demo shortcuts are removed at API, database, provider, tool, policy, and
  operator boundaries.
- **Failure:** A missing control produces a visible failed admission, blocked release,
  escalation, or dead-letter state.
- **Observability:** Each control emits trace fields, operation events, metrics, or
  runbook rows that operators can inspect.
- **Safety:** Risky actions require idempotency, authorization, approval, sandboxing,
  and redacted audit evidence.

## Exercises

1. Write a negative test where a production job kind is admitted without the idempotency
   key, approval policy, or observability evidence it requires. Explain which
   idempotency key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: a hardening checklist row or evidence packet linked to
   job kind, owner, control, and status.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   ProductionControl, HardeningStatus, and ReleaseGateDecision enums. Then name the
   runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Name the first hardening controls added to the demo lifecycle.
- Explain: Which ambiguity does each control remove?
- Apply: Choose duplicate intake, worker crash, risky action, or secret exposure and pick the needed control.
- Evidence: Name the idempotency key, lease, approval record, trace, or redacted configuration that proves hardening.

## Summary

The production system is not bigger because it is fancy. It is bigger because it preserves invariants under duplicate requests, crashes, bad outputs, unsafe tools, and operator stress.

- **Invariant:** each hardening control has an owner, evidence surface, test, and release gate.
- **Evidence:** idempotency records, leases, approval rows, audit events, traces, metrics, and secrets rules all point to the same job path.
- **Carry forward:** hardening is the act of making assumptions explicit before traffic finds them.

## Changed Understanding

- **Before this chapter:** hardening looked like adding checks after the happy path.
- **After this chapter:** hardening means turning each known failure mode into a constraint, state transition, test, or runbook check.
- **Keep:** turn every named hardening claim into a constraint, test, runbook query, or operation event.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
