# 20.2 Worked Production Scenario

## What You Will Learn

This chapter teaches you to:

- explain how the book's controls work together on one risky request;
- inspect duplicate intake, typed validation, policy review, Rig output, approval, tool execution, and receipts in one timeline;
- verify that no single control carries the whole reliability story.

The production evidence is a worked scenario where every decision leaves a row,
event, trace field, approval record, or receipt.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** handoffs can transfer responsibility without losing evidence.
- **Adds:** one realistic workflow through intake, retry, approval, side effect, and review.
- **Prepares:** SLO and error-budget reasoning for operating the system.

## Production Failure

Every individual control passed its own test.

Then one real incident combined duplicate intake, provider timeout, handoff,
approval delay, and side-effect execution. The team could not reconstruct the
whole chain from one evidence packet.

- **What breaks:** isolated correctness did not prove composed reliability.
- **False fix:** add one more isolated example for each mechanism.
- **Design response:** trace one realistic workflow end to end, requiring every
  step to name the row, event, receipt, approval, test, or runbook query that
  proves it.

## Motivation

In production, mechanisms are judged together. A duplicate webhook, provider timeout, approval delay, retry, and side-effect receipt can all belong to the same user-visible workflow.

Without an end-to-end scenario, each control can look correct in isolation while the combined system still loses evidence. This chapter follows one risky request through the full production path.

## Plain Version

Read this as the simple version:

- **Simple rule:** A worked scenario proves that the system pieces cooperate under one realistic business workflow.
- **Why it matters:** Reliability is easier to fake in isolated examples than in an end-to-end case with policy, tools, retries, and audit.
- **What to watch:** Watch the scenario from intake through scheduling, model output, validation, approval, side effect, evaluation, and incident review.

## What You Already Know

Start with these anchors:

- You have already seen idempotent admission, leases, retries, typed provider output, handoffs, approval, receipts, and operator evidence.
- Each control solves a narrow problem.
- Real requests need several controls at once.

This chapter adds: one integrated production scenario. You will follow a risky
request from intake through execution and see how each row, event, trace field,
approval, and receipt cooperates.

## Focus Cue

Keep three things in view:

- **State:** one risky request moving through duplicate intake, retry, handoff, approval, receipt, and operator review.
- **Move:** the scenario advances only when the previous control leaves the evidence required by the next one.
- **Proof:** Duplicate intake, retry, handoff, approval, receipt, and operator review form one traceable chain.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a worked evidence packet for one realistic agent workflow from request to audit trail.
- **Why it matters:** a serious book should show how separate controls combine in one production story.
- **Done when:** the scenario names rows, events, approvals, tests, runbook queries, and reviewer questions for the same run.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** the scenario evidence packet, Appendix N, and the companion modules named in each step.
- **State transition:** walk one realistic job through the whole production system.
- **Evidence path:** every step names its row, event, approval, receipt, test, or runbook query.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Can every step in the scenario point to the row, event, receipt, approval, or test that proves it?
- **Evidence to inspect:** scenario evidence packet, timeline, row ids, runbook queries, evaluation result, and reviewer questions.
- **Escalate if:** a narrative step sounds plausible but cannot be audited in the companion system.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** the worked scenario begins with a realistic user request.
2. **Action:** walk the request through admission, ownership, retry, typed result, approval, and audit.
3. **Persistence:** persist the evidence packet for each step.
4. **Check:** verify the scenario can be reviewed from rows, events, tests, and runbooks.

The important word is reviewed. A production scenario is not finished when the
agent returns a useful answer. It is finished when a tired operator, a security
reviewer, or a future engineer can replay the chain without guessing.

Each step asks a small question:

```text
What changed?
Who was allowed to change it?
Where is the evidence?
What would happen if the process crashed now?
```

If those questions have concrete answers, the scenario is becoming a production
workflow. If the answers depend on memory, private chat, or "the agent probably
did the right thing," the system is still a demo.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** each scenario step has row, event, approval, receipt, test, or runbook evidence.
- **Validation path:** inspect the scenario evidence packet and matching companion artifacts.
- **Stop if:** the story contains a production claim that cannot be audited.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, mechanisms are judged together
rule: A worked scenario proves that the system pieces cooperate under one realistic business workflow
tiny example: one risky request moving through duplicate intake, retry, handoff, approval, receipt, and operator review
artifact: a worked evidence packet for one realistic agent workflow from request to audit trail
proof: each scenario step has row, event, approval, receipt, test, or runbook evidence
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Tiny Scenario

A deployment system sends a webhook:

```text
incident_id: inc-7841
service: payments-api
symptom: error rate increased after deploy
request: analyze the incident and recommend one safe next action
```

The webhook provider times out waiting for your API response, so it sends the
same webhook twice.

The agent eventually recommends:

```text
summary: deploy 2026.05.23.4 correlates with payment failures
next_action: roll back payments-api to 2026.05.23.3
approval: required
```

That recommendation is useful, but it is also risky. The system must not let a
model-generated sentence become a production rollback by itself.

This is what I call **Systemic Alignment**. The agent helps with the "Precision" (identifying the bad deploy), while the system helps with the "Recall" (ensuring the action actually happens once approved). By separating these, we ensure the agent is aligned with our production safety goals.

Read the tiny case as:

```text
setup: a deployment webhook asks for one safe next action
transition: the system admits, leases, reasons, hands off, approves, executes, and records
evidence: every transition leaves a row, event, trace field, approval, or receipt
invariant: serious requests need multiple small controls, not one magical agent loop
```

## Reading The Scenario

This scenario is deliberately small, but it is not a toy.

The request is simple: analyze an incident and recommend one safe next action.
The consequence is serious: the recommended action may roll back a production
service. That makes it a good teaching case because it separates suggestion from
permission.

The model is allowed to help. It may summarize evidence, identify a likely bad
deploy, and propose a rollback. But the model is not allowed to become the
rollback system. The deterministic control layer still owns admission, leases,
retry state, typed validation, policy, approval, execution, and receipts.

Read each step with one mental model:

```text
The model may propose.
The system must decide.
The database must remember.
```

This is the **Student's Creed**. It's the bridge from agent demos to production engineering. A demo often
optimizes for the shortest path from prompt to action. A reliable system
optimizes for the safest path from request to evidence-backed action.

This is also a **Causal History**. In distributed systems, reliability is the ability to prove that **A happened because of B**. Your end-to-end trace is the proof of that causality.

## Step 1: Admission

The API validates the request and derives an idempotency key:

```text
deployment-incident:payments-api:inc-7841
```

The first webhook inserts a job:

```text
status: pending
idempotency_key: deployment-incident:payments-api:inc-7841
payload_schema_version: 1
prompt_version: incident-triage:v7
model_route: deepseek:v4-flash
policy_version: rollback-policy:v3
```

The duplicate webhook returns the same logical job instead of creating another
side-effect path.

This is the first important turn in the scenario. The duplicate webhook is not
treated as a strange edge case. It is treated as normal production behavior.
Networks retry. Providers retry. Users click twice. A reliable system expects
that.

The idempotency key gives the request a stable identity before any expensive or
risky work begins. Once that identity exists, duplicate intake becomes a lookup
problem instead of a duplicate-execution problem. The database constraint makes
the promise enforceable. The API can be wrong, two requests can race, and the
provider can retry; the uniqueness rule still protects the logical operation.

Production evidence:

```text
agent_jobs row exists
duplicate_suppressed event exists
idempotency key is unique for the logical request
```

## Step 2: Ownership

A worker claims the job with a lease:

```text
status: running
locked_by: worker-a
locked_until: 10:05
attempt_count: 1
```

The worker records:

```text
job_picked
agent_started
```

If `worker-b` asks for work at the same time, the locked row is skipped. If
`worker-a` disappears, the lease expiry makes recovery possible.

A lease is not ownership forever. It is ownership for a bounded time. That
boundedness is the reason the system can recover from process death without
letting two workers safely pretend they own the same job.

This is **Optimistic Concurrency Control (OCC)** implemented with a **Pessimistic Lease**. It's the most efficient way to scale worker pools without central coordination bottlenecks.

This is also where the worker stops being an invisible background loop. The
claim, heartbeat, completion, retry, and cancellation paths all become explicit
state transitions. A worker does not simply "do the job." It proves that it
owns the job before changing it.

Production evidence:

```text
pick query uses FOR UPDATE SKIP LOCKED
finish/retry/cancel queries require locked_by = worker-a
expired lease query can recover stale running rows
```

## Step 3: Retry

The first provider call times out.

The Rig boundary classifies the failure:

```text
provider timeout -> transient failure
```

The worker does not spin in a hidden loop. It persists the next state:

```text
status: pending
run_at: now + backoff
last_error: provider timeout
attempt_count: 1
```

Production evidence:

```text
agent_failed event
retry_scheduled event
next run_at is in the future
failure is classified as transient
```

This is the point where many agent systems become unreliable. They hide retry
inside a library call, an SDK helper, or a loop around `agent.run(...)`. That
can make the happy path shorter, but it also hides the operational truth.

In this book's design, retry is durable. The database records that the provider
timed out, that the failure was classified as transient, and that the next
attempt is scheduled for a later time. This gives the operator a clear answer
to a practical question: "Is this job stuck, failed, or waiting for a planned
retry?"

## Step 4: Typed Result

The second attempt succeeds. The model returns a structured result:

```text
AgentResult {
  summary,
  next_action,
  approval: Required,
}
```

The worker stores the result, but it does not execute the rollback. The result
is a proposal, not authority.

This distinction is one of the main lessons of the book.

Raw model output is not business truth. Even structured output is only a
candidate. The system still parses it, validates the shape, checks the action
against policy, and stores the approval requirement. Only after those checks
does the output become a typed proposal inside the domain.

That discipline is what keeps a useful model from silently becoming an
unauthorized production actor. The model can identify that a rollback looks
reasonable. The system decides whether rollback is permitted, who must approve
it, and how execution will be recorded.

This separation is what allows us to swap a "Creative/Drafting" model for the proposal and a "Logical/Checking" model for the policy check. I call this **Multi-Model Verification**.

Production evidence:

```text
agent_succeeded event
typed result stored
approval requirement preserved
no side-effect receipt exists yet
```

## Step 4.5: Specialist Handoff

The incident-triage agent can identify the likely rollback, but the system may
require a specialist deployment agent to verify rollback safety. That transfer
is not a chat message. It is a durable handoff:

```text
source_run_id: inc-7841-run
from_agent: incident_triage_agent
to_agent: deployment_safety_agent
reason: rollback requires deployment-specific checks
idempotency_key: handoff:inc-7841:deployment-safety
status: requested
```

The target agent accepts the handoff only when it can create or attach a target
job that owns the next piece of work:

```text
status: accepted
target_job_id: deployment-safety-job-912
decided_at: 2026-05-23T10:07:00Z
```

Production evidence:

```text
agent_handoffs row stores source and target agent
handoff reason and payload are durable
accepted handoff records target job evidence
pending_agent_handoffs.sql shows unresolved transfers
```

The handoff is useful because it changes responsibility without losing the
thread. The incident-triage agent should not pretend to know every deployment
safety rule. The deployment-safety agent should not receive an informal summary
with no link to the original run.

The durable handoff connects both sides. It says why responsibility moved, who
accepted it, and which target job now owns the next step. That makes the
handoff operationally inspectable instead of conversationally convenient.

## Step 5: Policy And Approval

The policy layer sees a risky action:

```text
rollback production service -> approval_required
```

It stores an action proposal:

```text
proposal_id: prop-221
job_id: inc-7841-job
policy_version: rollback-policy:v3
requested_action: rollback payments-api to 2026.05.23.3
status: approval_required
```

An operator approves the action with a reason:

```text
approved_by: operator-17
reason: error budget burn is active and rollback target is known-good
```

Production evidence:

```text
proposal exists before approval
approval records actor, reason, and time
policy version is stored
worker cannot execute rejected proposals
```

Approval is not a button at the end of the UI. It is a control surface in the
workflow.

The approval row records what was proposed, which policy version required human
review, who approved it, and why. That evidence matters later. If the rollback
helps, the team can learn which signal justified it. If the rollback causes a
new problem, the team can see whether the approval process had the right
context, policy, and escalation path.

This is why human-in-the-loop should not be treated as a vague safety slogan.
The human decision must become part of the system's durable evidence.

## Step 6: Side-Effect Receipt

The side-effect worker executes the rollback through a separate idempotency key:

```text
rollback:payments-api:inc-7841:prop-221
```

The receipt records what happened:

```text
action_succeeded
provider_request_id: deployctl-98341
target_version: 2026.05.23.3
```

If the side-effect worker crashes after sending the rollback but before writing
the receipt, replay must use the same side-effect idempotency key and inspect
the external system before acting again.

This is the most dangerous boundary in the scenario. Generating a rollback
proposal is reversible. Sending the rollback command is not.

> ### 🎓 The Professor's Corner
>
> **The Distributed Transaction Problem: The Receipt as Proof**
>
> You can't have a single "Magic Transaction" that spans both your database and the external world. If the database commits but the email API fails, or vice versa, you're in trouble! 
> 
> The **Receipt** is your compensating proof. It's the only way to "bridge the gap" between your local notebook and the outside world. Without the receipt, your system is just guessing what happened!

The side-effect receipt turns an external action into local evidence. It does
not make the external system perfectly reliable, and it does not remove the
need for reconciliation. It gives the local system a durable fact to reason
from: this action was attempted with this idempotency key, through this provider
request, for this approved proposal.

If the process crashes at the worst possible moment, the replay path does not
ask the model what to do again. It checks the receipt and the external system.
That is the difference between recovery and repetition.

Production evidence:

```text
side-effect key is stable
receipt is append-only
replay path checks existing receipt and external action state
```

## Step 7: Operator Review

At 03:00, the operator should be able to answer:

```text
What request created this job?
Was the duplicate webhook suppressed?
Which worker owned the attempts?
Why did the first attempt retry?
Which prompt, model, and policy versions were used?
Did any specialist handoff happen, and which target job accepted it?
Who approved the rollback?
Which receipt proves the rollback happened?
```

The answer should come from state rows, handoff rows, event timelines,
approvals, receipts, metrics, and runbook queries. It should not depend on
memory or a private chat thread.

> ### 🎓 The Professor's Corner
>
> **The Relay Race of Proof: Passing the Packet**
>
> Think of this whole scenario as a **Relay Race**. Each step—Admission, Ownership, Handoff, Approval—is a runner passing a "Proof Packet" to the next runner. 
> 
> If a runner tries to start without a packet (the previous evidence), the race stops! This ensures that no one is "Cheating" and everyone has the proof they need to do their job safely.

This final review is the real exam for the scenario.

If the operator can reconstruct the request, duplicate suppression, attempts,
timeout, typed result, handoff, approval, execution, and receipt from durable
evidence, the workflow is operable. If the operator needs to ask who remembers
what happened, the system did not produce enough evidence.

The goal is not to create paperwork. The goal is to make the system explainable
under pressure.

## Evidence Table

| Question | Evidence |
| --- | --- |
| Did the request become durable work? | `agent_jobs` row with idempotency key. |
| Was duplicate intake safe? | Duplicate suppression event and same job id. |
| Who owned execution? | Lease fields and worker-scoped transition predicates. |
| Why did retry happen? | Failure classification and retry event. |
| Did specialist transfer lose ownership? | `agent_handoffs` row with source agent, target agent, reason, idempotency key, status, and target job. |
| Why was action not automatic? | `approval: required` and policy proposal. |
| Who approved the action? | Approval row with actor and reason. |
| Was the side effect idempotent? | Side-effect key and receipt. |
| Can the team learn later? | Event timeline, versions, metrics, and incident notes. |

## What The Scenario Teaches

The individual mechanisms in this book are intentionally small. Idempotency
prevents duplicate logical work. Leases prevent concurrent ownership. Retry
state makes failure visible. Typed outputs keep generated text away from
business authority. Handoffs move responsibility without losing provenance.
Approval gates control risky autonomy. Receipts make side effects replay-aware.

The scenario matters because production incidents combine these mechanisms.
A duplicate webhook does not wait politely for the provider timeout to finish.
An approval delay does not remove the need for lease recovery. A successful
rollback does not erase the need to prove which model, prompt, policy, and
human decision led to it.

This is why the book treats reliable agents as systems, not prompts. The agent
is useful because it can reason over messy context. The system is reliable
because each risky transition has a typed boundary, durable state, and
reviewable evidence.

## Formal Definition

For this chapter, the precise definition is:

```text
A worked scenario is an end-to-end proof that independent controls compose without losing evidence before a risky side effect.
```

In the book's system model:

- **State:** one risky request moving through duplicate intake, retry, handoff, approval, receipt, and operator review.
- **Actor:** API, worker, provider boundary, policy gate, human approver, side-effect worker, and operator each own one step.
- **Transition:** the scenario advances only when the previous control leaves the evidence required by the next one.
- **Evidence:** Duplicate intake, retry, handoff, approval, receipt, and operator review form one traceable chain.
- **Invariant:** no irreversible action happens before the durable evidence chain proves it is safe.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | The happy path is understood but the evidence chain is not. |
| Production symptom | A duplicate webhook plus provider timeout cannot be reconstructed after the fact. |
| Corrective invariant | Risky work advances only through durable, inspectable transitions. |
| Evidence to inspect | Scenario timeline links duplicate intake, retry, handoff, approval, receipt, and operator review. |


## Production Contract

The whole scenario preserves one contract:

```text
Every risky transition has durable evidence before the next irreversible action.
```

That means:

```text
request before job
job before model call
failure before retry
handoff before target-agent work
proposal before approval
approval before side effect
receipt before replay confidence
timeline before incident learning
```

If any link is missing, the system may still work on a good day, but it cannot
be trusted during recovery, audit, or incident response.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | The happy path is understood but the evidence chain is not. | Understanding the happy path does not prove the controls compose during duplicate input or provider failure. |
| Safer version | Risky work advances only through durable, inspectable transitions. | The scenario advances only through durable, inspectable transitions with clear preconditions. |
| Production version | Scenario timeline links duplicate intake, retry, handoff, approval, receipt, and operator review. | Duplicate intake, retry, handoff, approval, receipt, and review form one traceable evidence chain. |

Use the naive row when a scenario is only narrative. Use the safer row to require preconditions. Use the production row when the scenario must teach recovery reasoning.

## Testing Strategy

Test the scenario as an end-to-end evidence chain:

- **Unit or type test:** prove Rust components can represent duplicate intake, transient provider failure, handoff, approval, side-effect receipt, and operator review separately.
- **Persistence or boundary test:** prove Postgres rows can reconstruct the scenario timeline from job events, run steps, tool calls, approvals, and receipts.
- **Regression test:** break one link, such as missing approval or missing receipt, and verify replay or execution stops at the correct point.

## Observability Strategy

Observe the scenario as one continuous evidence chain:

- Emit structured `tracing` fields for scenario step, job id, run id, handoff id, approval id, receipt id, attempt, and trace id.
- Record an operation event for duplicate intake, retry scheduling, handoff acceptance, approval decision, side-effect execution, and operator review.
- The runbook query should reconstruct the scenario timeline and show which control stopped or allowed each risky transition.

## Security and Safety Considerations

The worked scenario should fail safely at each risky step:

- Treat duplicate webhooks, model output, handoff payloads, approval notes, and side-effect receipts as untrusted until each boundary validates them.
- Check authorization, sandboxing, and approval before the scenario executes the irreversible action.
- Redact customer and provider details from the scenario while preserving job ids, handoff id, approval id, receipt id, and replay evidence.

## Operational Checklist

Use this checklist before relying on one end-to-end risky request in production:

- **State:** Duplicate intake, provider retry, approval wait, tool execution, receipt,
  and operator review form one durable timeline.
- **Boundary:** Webhook payload, model output, policy decision, human decision, and tool
  result each cross a typed boundary.
- **Failure:** The scenario proves duplicate input, timeout, approval delay, and retry
  do not lose ownership or duplicate side effects.
- **Observability:** The evidence table, trace id, events, and runbook queries
  reconstruct the whole story.
- **Safety:** The risky action remains blocked until authorization, sandboxing,
  approval, idempotency, and receipt rules pass.

## Exercises

1. Write a negative test where the duplicate webhook arrives during provider timeout and
   still maps to one idempotency path and one receipt. Explain which idempotency key,
   receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: the scenario evidence table rows for intake, retry,
   approval, execution, receipt, and operator review.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   ScenarioStep, EvidencePacket, ApprovalGate, and SideEffectReceipt types. Then name
   the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which controls appear in the worked scenario?
- Explain: Why no single control carries the whole reliability story.
- Apply: Reconstruct the scenario as a timeline of state transitions.
- Evidence: For each line, name the durable row, event, trace field, approval record, or receipt that proves it happened.

## Summary

Reliable agent systems are built from small mechanisms, but they are judged by the end-to-end path. Duplicate intake, timeout, approval, retry, execution, and receipt must still form one story.

- **Invariant:** every risky transition has durable evidence before the next irreversible action.
- **Evidence:** the scenario table links job, run, tool, policy, approval, retry, receipt, trace, and operator review records.
- **Carry forward:** production reliability is visible only when the whole path can be replayed mentally and operationally.

## Changed Understanding

- **Before this chapter:** a scenario looked like a story about a successful agent run.
- **After this chapter:** a worked scenario is a traceable chain of states, decisions, side effects, approvals, and evidence.
- **Keep:** trace the scenario through each durable state, decision, approval, side effect, and receipt.

## Further Reading and Sources



- [Pat Helland: Memories, Guesses, and Apologies](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: (2020). A foundational paper for the "Worked Scenario" in this chapter. It explains how to build reliable systems where agents (memories) propose actions based on uncertain information (guesses) and how to handle failures (apologies) via compensation.
- [Google SRE chapter: Testing for Reliability](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: Supports the chapter's goal of end-to-end scenario tracing to prove that individual controls (idempotency, leases, retries) compose correctly under live traffic.
- [Anthropic: Building Effective Agents](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: Industry guidance for the multi-step "Orchestration" workflow traced in this scenario, emphasizing the separation of reasoning from tool execution.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann, Chapter 11: Stream Processing). Provides the formal vocabulary for the "Timeline of Transitions" reconstructed by the operator at the end of the scenario.