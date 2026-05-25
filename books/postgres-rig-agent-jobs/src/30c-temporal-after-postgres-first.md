# 30.6 Temporal After Postgres-First

## What You Will Learn

This chapter teaches you to:

- decide when Temporal solves a real workflow-semantics problem instead of adding ceremony;
- explain why workflow history is not the same thing as product audit evidence;
- inspect the places where timers, replay, cancellation, and child workflows strain the custom worker loop;
- map the book's Postgres job ledger, agent runs, tool calls, approvals, and audit events into a Temporal-backed design;
- verify that typed Rust boundaries, idempotency, policy checks, observability, and product evidence survive when a workflow engine enters the system.

The production evidence is a Temporal adoption record. It names the strained
workflow invariant, the old Postgres owner, the new Temporal owner, the
coexistence plan, the rollback path, and the product ledger rows that remain
authoritative.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the Postgres-first system has explicit state, retries, leases, approvals, and runbooks.
- **Adds:** a disciplined way to adopt Temporal when timers, replay, cancellation, and orchestration dominate the custom worker code.
- **Prepares:** migration decisions where the team keeps product evidence in Postgres while using Temporal for durable workflow execution.

## Production Failure

A team moves every agent job into Temporal because long-running workflows sound
important.

The first incident is confusing. The Temporal Web UI shows a workflow retry,
Postgres shows an agent run stuck in `waiting_for_human`, the audit log has no
approval event, and the support team cannot tell whether a risky tool call
executed.

- **What breaks:** the workflow engine became the only visible history, while product evidence stopped being complete.
- **False fix:** tell operators to inspect Temporal instead of fixing the product ledger.
- **Design response:** use Temporal for workflow execution only after mapping workflow history back to typed product state, audit events, approvals, tool receipts, and runbook queries.

## Motivation

In production, Temporal is useful when workflow semantics are the hard part.

The hard part may be many durable timers, child workflows, fan-out and fan-in,
long waits, cancellation, retries across services, or replay after process
death. Those are real problems. The Postgres-first worker can handle many of
them for a long time, but not forever.

The mistake is treating Temporal as a replacement for product modeling.

Without that distinction, Temporal can remember workflow execution while the
product system forgets why a side effect was allowed. Temporal does not
automatically know which
business facts must be audited, which tool call required approval, which tenant
owns the data, or which side effect receipt proves an external action happened
once.

## Plain Version

The simple rule for this chapter is to adopt Temporal only when the actual shape of your workflow becomes significantly harder to manage than your standard job queue. This matters because while Temporal is exceptionally good at owning durable workflow progress, your product ledger must remain the sole owner of product truth. If you find your team constantly struggling to implement long timers, massive numbers of child workflows, complex cancellation rules, cross-service retries, or custom replay logic, and these issues are becoming your primary source of bugs, then it is time to consider a workflow engine.
Read this as the simple version:
- **Simple rule:** name the invariant before trusting the mechanism.
- **Why it matters:** vague reliability claims fail during incidents.
- **What to watch:** the proof must be a row, type, event, receipt, or runbook check.


## What You Already Know

Start by grounding yourself in the concepts you have already mastered. You know that a job table is a durable promise ensuring work survives a process death. You understand that a retry is only safe when the operation possesses strict identity, and that a tool call is a dangerous side effect requiring a receipt. You have learned that human approval is a formal, durable control state rather than just a UI detail, and that audit events constitute formal business evidence rather than mere debug logs. 

This chapter adds one crucial layer to this foundation: a workflow engine can entirely take over the mechanics of execution, but it must never be allowed to erase or obscure your typed product state.
Start with these anchors:

- Durable state is the first production boundary.
- Typed values make production meaning explicit.
- Evidence must survive process death.

This chapter adds: one more production mechanism that can be inspected, tested, and operated.


## Focus Cue

As you read, keep three critical concepts in view. Regarding **State**, remember that the core product state remains securely in Postgres, while the transient workflow execution state may move into Temporal. Regarding the **Move**, understand the sequence: a job starts or signals a workflow, the workflow schedules activities, and those activities explicitly write typed product evidence back into the database. Finally, regarding **Proof**, ensure that an operator can flawlessly reconcile a Temporal workflow id, a Postgres job id, an agent run id, a tool call id, an approval id, and an audit event id. 

If you get lost in the details, return to state, move, and proof. Then, ask yourself one clarifying question: which specific history actually answers the product question?
Keep three things in view:
- **State:** the production fact that changes.
- **Move:** the lawful transition from one state to another.
- **Proof:** the evidence an operator can inspect later.


## Production Artifact

Before you consider moving on, you must build or inspect a formal Temporal adoption decision record. This artifact is critical because introducing a workflow engine fundamentally changes the responsibility boundaries within your architecture. You will know this record is complete when it explicitly names the workflow id mapping, the product ledger mapping, the replay rules, activity idempotency constraints, approval behaviors, trace propagation paths, the rollback strategy, and the required runbook updates.
Build or inspect this artifact before moving on:
- **Artifact:** the concrete row, type, policy, receipt, or runbook query for this chapter.
- **Why it matters:** learning becomes production skill only when it changes an inspectable artifact.
- **Done when:** another engineer can inspect the artifact and explain the invariant it protects.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** Temporal workflow id, Postgres job row, agent run row, activity receipt, approval row, and audit event.
- **State transition:** Postgres dispatch creates or signals a workflow; Temporal drives durable execution; activities write product evidence back to Postgres.
- **Evidence path:** every workflow execution can be joined back to product rows and trace ids.

## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which workflow invariant does Temporal now own, and where does the product evidence live?
- **Evidence to inspect:** Temporal workflow id, workflow type, activity result, Postgres job row, agent run row, tool receipt, approval row, audit event, and trace id.
- **Escalate if:** operators must choose between Temporal history and Postgres evidence because they disagree.

## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a Postgres job reaches a workflow-heavy job kind.
2. **Action:** the worker starts or signals a Temporal workflow using a stable workflow id derived from the typed job identity.
3. **Persistence:** the workflow schedules activities, and each activity writes typed product evidence to Postgres.
4. **Check:** runbooks reconcile Temporal status with product status, side-effect receipts, approvals, and audit events.

## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** one agent run can be reconstructed from both Temporal workflow history and Postgres product rows without contradiction.
- **Validation path:** start one workflow, force one retryable activity failure, require one approval, complete one idempotent tool call, and verify both histories agree.
- **Stop if:** Temporal completion can occur without a matching product event, or a product side effect can occur without an idempotent receipt.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: workflow code is becoming the hardest reliability layer
rule: let Temporal own execution mechanics, not product truth
tiny example: one job id maps to one workflow id and one agent run id
artifact: Temporal adoption record with evidence mapping and rollback
proof: workflow history and Postgres rows tell the same story
```

If the next section feels large, keep only these five lines in view.

## Tiny Example

A KYC case-preparation agent waits for documents, runs extraction, asks for
analyst approval, and then prepares a submission packet.

With only Postgres, the worker loop may own each timer and transition:

```text
scheduled_jobs -> claim job -> run extraction -> wait for approval -> resume -> complete
```

With Temporal, the workflow may own the long wait and orchestration:

```text
workflow: prepare_kyc_case
  activity: load_case_context
  activity: extract_documents
  wait: analyst approval signal
  activity: build_submission_packet
  activity: record_completion
```

The important point is not the new tool.

The important point is the retained evidence:

```text
setup: one KYC case needs document extraction, analyst approval, and a final packet
transition: a Postgres job starts a workflow, and workflow activities write product rows
Temporal workflow id -> agent_runs.workflow_ref
Temporal activity id -> tool_calls.external_execution_ref
approval signal -> human_approval_requests decision row
completion -> audit_events and operation_events
evidence: workflow id, activity id, run id, approval id, receipt id, and trace id reconcile
invariant: workflow replay cannot create an unaudited or unauthorized product action
```

Temporal makes the wait and replay easier. Postgres still makes the product
state inspectable.

## Mental Model

Think of Temporal as a workflow execution ledger.

Think of Postgres as the product truth ledger.

They can work together only if the bridge is explicit:

```text
Temporal history answers:
  How did the workflow execute?

Postgres product rows answer:
  What business state changed?
  Which side effects happened?
  Which approvals were granted?
  Which audit evidence exists?
```

When these histories disagree, production has a reconciliation problem.

## The Core Problem

Temporal replay changes how code executes. Workflow code must be deterministic
because replay rebuilds workflow state from recorded events. External I/O must
live in activities, where results can be recorded and reused.

That model fits reliable agents well if the boundary is respected:

```text
workflow code:
  durable control flow, timers, child workflows, signals, cancellation

activity code:
  model calls, tool calls, database writes, API calls, file I/O

product ledger:
  agent runs, tool calls, approvals, receipts, audit events
```

The risk is putting product trust in the wrong place. A Temporal event proves a
workflow step happened. It does not automatically prove that the business
approval, permission check, or side-effect receipt was recorded in the product
ledger.

## The Naive Solution

The naive migration wraps the old agent in one workflow:

```text
workflow:
  result = agent.run("handle this case")
  return result
```

This hides the same production questions as the original demo line:

```text
Which model version ran?
Which prompt version ran?
Which tool was proposed?
Was the tool authorized?
Was approval required?
Which activity performed the side effect?
Can the side effect be retried safely?
Which Postgres row proves it?
```

Temporal did not fix the architecture. It only moved the opaque call into a
durable shell.

## The Production-Grade Concept

Adopt Temporal as an execution adapter behind the same typed control model.

| Existing concept | Temporal mapping | Product evidence that remains |
| --- | --- | --- |
| `ScheduledJobId` | workflow id or signal target | `scheduled_jobs` row with status, idempotency key, attempts, and trace id |
| `AgentRunId` | workflow search attribute or memo | `agent_runs` row with model, prompt, policy, and lifecycle status |
| Tool execution | activity | `tool_calls`, `operation_events`, side-effect receipt, and idempotency record |
| Human approval | signal or update | `human_approval_requests` row with reviewer, decision, policy version, and reason |
| Cancellation | cancellation request and workflow cancellation | `cancellation_requests`, terminal status, and audit event |
| Retry policy | activity retry policy plus domain retry budget | `failure_history`, attempts, next action, and dead-letter reason |

The adapter should make the relationship boring:

```text
Postgres job row starts the workflow.
Workflow schedules activities.
Activities cross external boundaries.
Activities write product evidence.
Workflow completion updates product state.
Runbooks reconcile both histories.
```

## Typed Rust Boundary Sketch

When starting a Temporal migration, you must never pass workflow ids, activity ids, and approval signals around as raw strings. Because these values immediately become formal production evidence, confusing them turns your incident reviews into guesswork. 

The companion crate provided with this book models the bridge directly without adding a massive Temporal runtime dependency. This approach keeps the book's default tech stack small while making the adoption boundary highly executable.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/temporal_adoption.rs:temporal_bridge}}
```

You should read these types as a strict production checklist. The `TemporalWorkflowBridge` must contain exactly one Postgres job reference, one agent run reference, one workflow execution reference, and one trace id. The `TemporalActivityReceipt` must explicitly document one activity, one tool call, one idempotency key, and one operation event. The `TemporalProductEvidence` must define the exact audit and operation evidence that humans will use later to review the action. Finally, the `TemporalReconciliationPacket` serves as the cryptographic proof that the workflow history and the product evidence undeniably belong to the exact same run. 

This is the central production move: the workflow engine is permitted to own timers and replay mechanics, but the core application still adamantly owns the typed evidence contract.

## Postgres Evidence Tables

Temporal adoption needs a small bridge in the product database.

Do not hide this bridge in workflow payloads. Store it where operators can join
it to jobs, agent runs, tool calls, approvals, operation events, audit events,
and traces.

The companion schema adds two evidence tables:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql:temporal_optional_scaling_tables}}
```

Read the schema as a production promise:

- `temporal_workflow_links` says which product job and agent run a workflow owns.
- `workflow_execution_ref` is unique, because one workflow identity must not point to many product histories.
- `temporal_activity_receipts` says which side-effecting activity wrote product evidence.
- `operation_event_id` ties the activity back to the operator timeline.
- The trace id keeps the workflow, worker, Rig call, tool call, and database row in one debugging path.

The point is not to store Temporal internals in Postgres.

The point is to keep product evidence queryable when execution moves to a
workflow engine.

## Reconciliation Query

The first runbook query should answer a narrow question:

```text
Do Temporal workflow history and Postgres product evidence agree for this workflow?
```

The checked query is intentionally boring:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/temporal_workflow_reconciliation.sql}}
```

Use it during migration, incidents, and rollback decisions.

If this query cannot find the product rows for a workflow, the workflow is not
production-ready. It may have executed, but the system cannot yet prove the
business meaning of that execution.

## Formal Definition

For this chapter, the precise definition is:

```text
Temporal adoption is evidence-preserving migration of workflow execution responsibility from a custom Postgres worker loop to a durable workflow engine while keeping product truth typed, audited, and queryable.
```

In the book's system model:

- **State:** Temporal owns workflow execution state; Postgres owns product state, approval state, tool-call state, receipts, and audit evidence.
- **Actor:** the worker starts or signals workflows; activities perform external work; operators reconcile workflow and product evidence.
- **Transition:** a job kind moves from direct claim-and-run execution to workflow-backed execution through a coexistence and rollback plan.
- **Evidence:** workflow id, run id, job id, activity id, trace id, receipt id, approval id, and audit event id are correlated.
- **Invariant:** adopting Temporal must not create any product action that lacks typed authorization, idempotency, durable receipt, and audit evidence.

## What Can Fail

When adopting Temporal, several critical failure modes can emerge. The most common design smell occurs when Temporal is introduced before the team has explicitly named which specific workflow invariant it is supposed to own. The production symptom of this mistake is that the Temporal workflow history will loudly declare that the work is "completed," but the underlying Postgres product rows cannot actually prove the approval state, the side effect execution, or the receipt state.

The corrective invariant to enforce is that the workflow execution history and the product evidence must flawlessly reconcile for every single important transition. If a failure occurs, the operational evidence you must inspect includes the workflow id mapping, the search attributes, the activity receipts, the Postgres agent run, the tool call, the human approval, the formal audit event, the failure history, and the final runbook output. **Design smell:** the design names a mechanism but not the invariant it protects.

**Production symptom:** operators cannot explain what changed or which evidence proves it. **Corrective invariant:** every important transition must be owned, durable, and reviewable. **Evidence to inspect:** inspect the row, event, receipt, policy decision, trace, or runbook output.


## Production Contract

When integrating these systems, you are drafting a clear production contract. Temporal is fully permitted to own the mechanical orchestration details: the timers, the activity scheduling, the child workflow graph, the signals, the cancellation propagation, and the complex workflow replays. 

However, Postgres must remain the absolute owner of product evidence (unless a later architectural design explicitly replaces it with an equivalent, fully audited product ledger). Postgres retains authority over the agent run identity, the specific model and prompt versions used, tool-call proposals and validations, authorization decisions, human approval decisions, side-effect receipts, formal audit events, and evaluation receipts. The final contract is incredibly simple: workflow history helps the system continue executing, but product evidence is what helps humans trust what continued.

## Minimum Serious Temporal Adoption

When adopting Temporal, do not start by moving every single job kind over all at once. Instead, identify one specific job kind where the Postgres worker loop is already being actively strained by complex workflow mechanics. Good candidates for this initial migration are jobs that exhibit several distinct pressures simultaneously: they have exceptionally long wait states, they rely on many durable timers, they require complex child workflows, they depend heavily on external approval signals, they need cross-service cancellation capabilities, they require frequent activity retries that must generate stable receipts, or they suffer from an operator need to manually replay specific execution paths.

Once you have selected a candidate, make the scope of adoption small enough to rigorously inspect. The minimum serious shape for this adoption requires explicit definitions. The workflow identity must be a stable `WorkflowExecutionRef` derived directly from a typed job identity, never a random string. The job mapping must ensure that the original `scheduled_jobs` row safely retains the product job id, the idempotency key, the trace id, and a formal migration marker. The agent run mapping requires that `agent_runs` either stores or can easily derive the workflow reference used by your operators. 

Crucially, the activity boundary must be strictly enforced: every model call, tool call, database write, or external API call must happen exclusively within an activity, never within the deterministic workflow code itself. Each of these side-effecting activities must then write a formal activity receipt back to Postgres containing the activity reference, tool call id, idempotency key, operation event, and trace id.

While a workflow signal may technically resume a pause, the actual approval signal must live as a decision row in `human_approval_requests` alongside the reviewer identity, reason, policy version, and audit event. Finally, you must have a reconciliation runbook capable of comparing the workflow status with all product evidence, and a clear rollback plan so new work can return to the pure Postgres path while in-flight workflows drain safely.

This detailed shape enforces the smallest useful rule for the architecture: Temporal may advance execution *only* through activities that explicitly leave product evidence behind. That rule is what keeps Temporal a useful tool without allowing it to become an invisible, secondary business system.

## Real Implementation Shape

A production-ready Temporal implementation usually splits into four distinct architectural pieces. First, the workflow module is strictly responsible for deterministic control flow, timers, child workflows, signals, and cancellation logic. Second, the activity module handles all non-deterministic actions, such as Rig calls, tool execution, Postgres writes, provider calls, and file I/O. Third, the bridge module enforces the typed mapping between systems, converting a `ScheduledJobId` to a `WorkflowExecutionRef`, mapping an `ActivityExecutionRef` to a `ToolCallId`, and ensuring an approval signal translates into a formal approval row. Finally, the reconciliation module provides the tooling to join workflow history with Postgres rows, traces, and audit events.

The companion module `temporal_adoption.rs` provided in this book is intentionally not a complete Temporal SDK implementation. It is the typed bridge you must design *before* adding the runtime dependency. If those boundary types are vague, a real workflow engine will only make that ambiguity more durable. When you do implement the Temporal Rust SDK, always remember the golden rule from their documentation: workflow code is constantly replayed and must be strictly deterministic. You must use activities for all non-deterministic work, and you must make those activities rigorously idempotent at the product layer.

## When Not To Use Temporal Yet

Do not add Temporal to your architecture simply because writing a workflow sounds like an important, enterprise-grade milestone. You should continue to rely on the simpler, Postgres-first worker loop for as long as possible if your primary architectural pain points are actually symptoms of underlying ambiguity rather than mechanical orchestration limits.

For example, if your overall state machine is unclear, you must first explicitly name the states, transitions, owners, and evidence rows in Postgres. If your retries are occasionally duplicating work, you must add idempotency keys, side-effect receipts, and strict regression tests *before* adding a powerful workflow engine that will just retry those bugs faster. If operators currently cannot fully explain the history of a single agent run, you must first improve the `agent_runs`, `tool_calls`, `operation_events`, `audit_events`, trace propagation, and runbooks.

If human approvals are currently inconsistent or getting lost, you must fix the approval state machine and ensure audit evidence is recorded before you move the "wait" state into a Temporal signal. Finally, if you just have one worker that is acting slowly, you should measure queue latency, split the worker pools, or tune the claim query before attempting a massive orchestration migration.

Temporal is an excellent answer when the pure mechanics of workflow execution—the timers, the pauses, the complex retries—are the specific invariant under strain. It is a strictly incorrect answer when your underlying product model is still vague.

The fundamental rule for stopping a premature migration is simple: If you cannot conclusively prove the execution of your workflow with concrete Postgres evidence today, Temporal will simply give you a much more durable version of that exact same ambiguity tomorrow.

## Progressive Hardening Path

Migrating to a workflow engine is a progressive hardening path, not a single deployment step.

In the naive version of adoption, teams often put the entire agent loop inside a workflow and mistakenly treat Temporal's internal history as sufficient evidence. In this state, execution gracefully resumes after a failure, but actual product state, approvals, idempotency records, and audit evidence remain dangerously implicit or scattered. 

The safer version improves upon this by strictly starting workflows from typed Postgres jobs and forcing all activities to write formal product rows. Here, the workflow engine successfully owns the timers and retries, while Rust domain types and SQL rows continue to enforce strict business boundaries. 

The final, production-grade version hardens this integration entirely. The team adds strict workflow id mapping, specific search attributes, fully idempotent activities, formalized approval signals, end-to-end trace propagation, reconciliation runbooks, explicit rollback criteria, and routine incident drills. At this stage, operators can confidently debug the exact same agent run simultaneously from Temporal history, Postgres rows, traces, and audit events, completely retaining the Postgres-first reliability model while gaining Temporal's execution power.
**Naive version:** the mechanism works once but does not leave enough evidence for recovery.
**Safer version:** the mechanism names ownership, state, and proof before execution.
**Production version:** the mechanism survives crash, retry, deploy, audit, and handoff through durable evidence.

## Testing Strategy

You must test your Temporal integration by verifying both execution progress and evidence preservation. In your unit or type tests, you must model the workflow bridge and verify that conversion fails if any required reference (workflow ref, job ref, trace id) is missing. Your persistence tests must simulate activity executions and confirm they write proper idempotency records and side-effect receipts to Postgres.

Finally, your regression tests must execute a mock workflow that triggers transient errors, verifying that activity retries correctly reuse receipts and do not duplicate external action. **Unit:** test the smallest typed transition and the invariant it preserves. **Persistence:** test the database row, query, or receipt that proves the transition survives process death. **Regression:** keep a failing case for the production bug this chapter is designed to prevent. **Rust:** use the companion crate to make invalid state hard to represent.

## Observability Strategy

You must observe your Temporal workflows by unifying workflow execution traces with database transactions. Emit structured `tracing` fields for the workflow ref, activity ref, job id, and trace id across both systems. You must record operation events when a workflow is started, signaled, or completed, and when activities write receipts. Ultimately, the runbook query must reconcile Temporal workflow status with Postgres rows and trace contexts to prove that no execution is undocumented.

## Security and Safety Considerations

Temporal adoption introduces a new execution surface that must not bypass security policies. You must treat workflow payloads and activity arguments as untrusted boundaries, validating them using typed Rust models before processing. Crucially, authorization, sandboxing, and human approval must be enforced inside activity execution, leaving durable decision records in Postgres. Always redact credentials and sensitive data from activity inputs while keeping the workflow ref, owner, and trace id visible for audit.
Redact secrets, tenant data, prompts, and private payloads while preserving ids, state names, and evidence references for audit.

## Operational Checklist

Before declaring the Temporal migration complete, operators must perform a strict review of the system's boundaries and failure modes.

First, verify the **State** boundary: ensure everyone agrees exactly which workflow execution state lives in Temporal, and which durable product state remains in Postgres. Second, inspect the **Boundary** transitions themselves: verify that workflow payloads, signals, activity inputs, and activity results are strictly converted into typed domain values before they are used to change business logic. 

Third, rehearse your **Failure** modes: document exactly what happens when the system encounters an activity retry, a workflow replay, a cancellation, a duplicate signal, a worker crash, or a required rollback to the old Postgres path. Fourth, validate your **Observability** pipeline: confirm that a single trace id can seamlessly connect the Temporal workflow history, the Postgres product rows, operation events, and the final audit events. Finally, verify **Safety**: ensure that all previous guarantees regarding authorization, sandboxing, human approval gates, data redaction, idempotency, and durable receipts are strictly preserved for every single activity that possesses the capability to change the outside world.

## Exercises

1. Pick one job kind from Chapter 20 and decide whether its timers, child workflows,
   cancellation, or replay needs justify Temporal. Name the invariant that
   moves.
2. Design a typed mapping from `ScheduledJobId` to `WorkflowExecutionRef`.
   Explain why both ids should not be raw strings, and write a Rust negative test
   that rejects an empty workflow reference.
3. Write a reconciliation query plan that compares one workflow id with
   `agent_runs`, `tool_calls`, `human_approval_requests`, `operation_events`,
   and `audit_events` in Postgres. Include the idempotency key that prevents a
   retried activity from duplicating a side effect.

## Self-Check

Before you move on, use this quick retrieval drill to solidify your understanding. First, recall exactly what Temporal should own in this architecture versus what Postgres must still permanently own. Next, be able to clearly explain why a workflow's internal execution history is not automatically equivalent to a compliant product audit trail. Then, apply this knowledge to your own systems by deciding whether a specific long-running approval workflow needs Temporal or if it can be handled by a simpler Postgres job state. Finally, ensure you can explicitly name the workflow id, job id, run id, approval id, receipt id, audit event id, and trace id that correlate to one single execution run.
- Recall: what is the core invariant in this chapter?
- Explain: why does the invariant matter during an incident?
- Apply: use the idea on one real agent job or tool call.
- Evidence: name the artifact that proves the result.


## Summary

Temporal is a serious, highly capable tool for durable workflow execution. It is incredibly valuable when the pure mechanics of workflow coordination become the hardest part of your system: specifically timers, replay, child workflows, cancellation logic, and cross-service orchestration.

The core invariant to remember is that workflow execution can safely move to Temporal only if your underlying product truth remains typed, durable, fully auditable, and easily reconcilable. To enforce this, your architecture must rely on workflow id mapping, explicit activity receipts, Postgres rows, operation events, audit events, traces, and documented runbooks to prove the migration is safe. 

Moving forward, remember the golden rule: never replace a visible, queryable product ledger with an opaque, albeit durable, execution shell.

**Invariant:** the chapter concept must preserve its named production rule under failure.

**Evidence:** the proof must be visible as a row, event, receipt, trace, policy, test, or runbook query.

## Changed Understanding

Before reading this chapter, Temporal may have simply looked like a much bigger, more powerful version of a standard job table. After this chapter, you should understand that Temporal is an optional execution engine whose internal history must always be rigorously mapped back to concrete product evidence. Moving forward, keep in mind that you should adopt Temporal only when complex workflow semantics truly dominate your engineering time, not when your underlying product state is still vague and undefined.
- **Before this chapter:** the mechanism may have looked like an implementation detail.
- **After this chapter:** the mechanism is a production contract with evidence.
- **Keep:** name the invariant, evidence, and operator question before relying on it.


## Further Reading and Sources

- [Temporal Workflows](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: is relevant because workflow definitions, executions, commands, events, replay, and activities are the concepts this chapter maps to the book's product ledger.
- [Temporal Event History](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: is relevant because the chapter depends on understanding workflow history as execution evidence, not automatic business audit evidence.
- [Temporal Activities](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: is relevant because model calls, tool calls, API calls, and database writes must live behind activity boundaries rather than deterministic workflow logic.
- [Temporal Rust SDK Workflows](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: is relevant because Rust readers need to see where Temporal's Rust workflow surface begins and where the book's typed product evidence remains separate.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: is relevant because the migration is a data-system responsibility change around logs, state, and consistency.
