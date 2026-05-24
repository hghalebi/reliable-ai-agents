# Appendix U. Operator Control Surface

## How to Use This Appendix

This appendix describes the operator surface for a reliable agent system. It is
not a UI design exercise. It is a control contract.

Use it when you are ready to build a dashboard, admin page, command-line
operator tool, or internal console on top of the Postgres-backed agent system.
The rule is simple:

```text
an operator surface may summarize state,
but every action must write durable evidence.
```

A dashboard that only shows charts is not enough. A console that can mutate
jobs without audit evidence is worse than no console.

## Motivation

Long-running agents need human control. Operators must see which work is
running, stuck, retrying, waiting for approval, denied by policy, over budget,
or unsafe to replay.

Without a clear control surface, teams fall into two bad habits. They either
trust a vague dashboard label, or they edit database rows by hand during an
incident. Both are dangerous. The first hides detail. The second bypasses the
state machine.

The operator surface should sit between those extremes:

```text
summaries for fast orientation
checked queries for evidence
typed controls for mutation
audit events for accountability
```

## Mental Model

An operator surface has three layers:

```text
read model:
  show the current system state from Postgres, metrics, traces, and events

decision model:
  help the operator choose inspect, wait, pause, resume, cancel, approve,
  escalate, retry, reconcile, or quarantine

write model:
  perform only typed, permissioned, audited state transitions
```

The read model may be convenient. The write model must be strict.

## What the First Console Should Show

Start with a small operations console. It should answer the questions operators
actually ask during an incident:

| View | Operator question | Evidence source |
| --- | --- | --- |
| Queue health | Is work piling up? | `/metrics`, `queue_metrics.sql`, `queue_metrics_by_kind.sql`. |
| Oldest pending work | Which job kind is waiting too long? | `oldest_pending_job.sql`. |
| Running work | Which agent runs are active now? | `running_agent_runs.sql`. |
| Schema migration status | Which expand, backfill, or contract phase is planned, running, blocked, failed, or recently passed? | `schema_migration_status.sql`. |
| Failure drill status | Which controlled failure experiment is planned, running, failed, aborted, or missing evidence? | `failure_drill_status.sql`. |
| Fault-tolerance readiness | Which job kinds can keep serving from last-known-good state when control surfaces fail? | `fault_tolerance_readiness.sql`, `failure_drill_status.sql`, `release_gate_status.sql`, and `fault_tolerance_reviews`. |
| Expired leases | Which workers may have crashed? | `expired_leases.sql`. |
| Scheduled retries | Which failures are waiting to retry? | `scheduled_retries.sql`. |
| Waiting approvals | Which risky actions need a human? | `waiting_human_approvals.sql`. |
| Open escalations | Which issues need named ownership? | `open_human_escalations.sql`. |
| Failed tools | Which tool calls failed or were rejected? | `failed_tool_calls.sql`. |
| Security denials | Which authorization or sandbox decisions denied action? | `denied_authorization_events.sql`, `sandbox_policy_violations.sql`. |
| Tenant boundary review | Did any actor request another tenant, and was every cross-tenant request denied? | `tenant_boundary_review.sql`, `denied_authorization_events.sql`, policy version, trace id, operation events, and audit events. |
| Side effects | Which external actions already happened? | `side_effect_receipts_by_run.sql`, `outbox_backlog.sql`. |
| Temporal reconciliation | Do workflow execution history and product evidence agree? | Temporal workflow id, workflow search attributes, `temporal_workflow_reconciliation.sql`, `running_agent_runs.sql`, `audit_events_by_run.sql`, and side-effect receipts. |
| Kafka distribution | Which published events, consumer groups, and replays need attention? | Outbox status, Kafka topic-partition-offset, `kafka_replay_safety_by_event.sql`, consumer receipts, projection lag, replay drill notes, and operation events. |
| Restore replay | Which restored jobs are safe or unsafe to replay? | `restore_replay_candidates.sql`. |
| Cost and quota | Which provider route is consuming budget? | `provider_usage_by_job_kind.sql`. |
| Job-kind lifecycle | Which job kinds can stay active, enter deprecation, become retirement candidates, or remain retirement-blocked? | `job_kind_lifecycle_review.sql`. |
| Storage pressure | Which evidence tables are growing, dead-row-heavy, or missing vacuum/analyze evidence? | `storage_pressure_by_table.sql`. |
| Credential lifecycle review | Which credential kinds are due, overdue, exposed, stale, or recently revoked? | `credential_rotation_review.sql`. |
| Retention review | Which evidence surfaces have old rows that need archive, aggregation, redaction, or policy review? | `retention_review_by_surface.sql`. |
| Data protection review | Which surfaces have open or overdue redaction, erasure, export, or retention-review requests? | `data_protection_review.sql`. |
| Release gate status | Which candidates are blocked, canary-only, or recently promoted, and what evidence justified the decision? | `release_gate_status.sql`. |

## Operator Question Index

Use this index when an incident starts with a question instead of a dashboard.
Each answer should point to durable evidence, not private memory.

| Production question | First evidence to inspect | Second evidence to inspect | Stop if |
| --- | --- | --- | --- |
| Which agents are currently running? | `running_agent_runs.sql` shows active run ids, lifecycle status, prompt versions, model versions, trace ids, and owning jobs. | `operation_events_by_job.sql` reconstructs the current run timeline. | A running run has no trace id, prompt version, model version, or owning job. |
| Which jobs are stuck? | `oldest_pending_job.sql`, `expired_leases.sql`, and `running_jobs_past_deadline.sql` separate waiting work, lost ownership, and breached deadlines. | `dead_jobs_by_reason.sql`, `failure_history_by_job.sql`, and `job_event_timeline.sql` explain repeated or terminal failures. | The console collapses stuck, overdue, cancelled, and dead-lettered work into one vague label. |
| Which tool calls failed? | `failed_tool_calls.sql` shows failed, rejected, and policy-blocked tool calls. | `audit_events_by_run.sql` and `operation_events_by_job.sql` show the surrounding run and decision evidence. | The failed call cannot be tied to an agent run, tool name, input snapshot, error, and trace id. |
| Which retries are scheduled? | `scheduled_retries.sql` shows retryable jobs, attempts, next run time, and last error. | `failure_history_by_job.sql` shows whether the retry pattern is new, repeated, or already terminal. | Retry work has no bounded attempt policy or idempotency key. |
| Which human approvals are waiting? | `waiting_human_approvals.sql` shows approval requests, risk class, reviewer scope, and age. | `open_human_escalations.sql` shows approvals that have become ownership or incident problems. | A risky action can proceed without a durable approval decision. |
| Which model version produced this output? | `running_agent_runs.sql`, `evaluation_receipts_by_version.sql`, and `release_gate_status.sql` connect output to model version, prompt version, dataset, and release decision. | `audit_events_by_run.sql` preserves the business-significant result and actor evidence. | A result cannot be tied to model, prompt, policy, evaluator, and release evidence. |
| Which prompt version was used? | `running_agent_runs.sql` shows the prompt version attached to active work; `evaluation_receipts_by_version.sql` shows how that prompt version behaved before release. | `release_gate_status.sql` shows whether the prompt version was promoted, canary-only, or blocked. | Prompt changes can alter behavior without versioned evaluation evidence. |
| Which side effects happened? | `side_effect_receipts_by_run.sql` shows external actions, receipts, idempotency keys, and reconciliation state. | `outbox_backlog.sql` and `audit_events_by_run.sql` distinguish planned publication, completed publication, and business evidence. | Operators must infer side effects from provider logs, emails, terminal scrollback, or model text. |
| Can the execution plane continue if the control plane is unavailable? | `fault_tolerance_readiness.sql` shows control-plane status, execution-plane status, static-stability mode, redundant workers, failover drill status, release gate decision, and readiness status by job kind. | `failure_drill_status.sql`, `release_gate_status.sql`, and last-known-good version fields prove the operating mode is practiced and approved. | A dashboard, prompt editor, admin API, or policy console outage can accidentally stop already-approved production work. |
| Did any tenant boundary fail? | `tenant_boundary_review.sql` shows cross-tenant attempts, allowed cross-tenant decisions, denied attempts, approval-required actions, and latest decision time. | `denied_authorization_events.sql`, `operation_events_by_job.sql`, and `audit_events_by_run.sql` show the surrounding run and policy evidence. | A cross-tenant request is authorized, requires approval without a scoped delegation model, or lacks a traceable denial reason. |
| Do Temporal and Postgres agree for this run? | The Temporal workflow id, workflow status, activity ids, and search attributes should map to the Postgres job id, agent run id, trace id, and lifecycle status. | `audit_events_by_run.sql`, `side_effect_receipts_by_run.sql`, and approval evidence show whether the workflow result has product evidence. | Temporal history is the only place where a business decision can be explained, or Postgres says terminal while Temporal is still active without a reconciliation event. |
| Which Kafka event or consumer is lagging or unsafe to replay? | Outbox status, Kafka topic-partition-offset, consumer group, consumer receipt, and projection lag show where distribution is stuck. | Event envelope schema version, authorization boundary, replay rule, and side-effect receipts show whether replay is safe. | A replay button ignores consumer receipts, schema compatibility, tenant boundaries, or side-effect idempotency. |
| Can we safely replay this step? | `restore_replay_candidates.sql` classifies restored work as safe to resume, needing reconciliation, or unsafe to replay. | `side_effect_receipts_by_run.sql` and `failure_history_by_job.sql` show whether an external action may already have happened. | The replay decision ignores side-effect receipts or idempotency state. |
| Can we prove what happened? | `job_event_timeline.sql`, `operation_events_by_job.sql`, and `audit_events_by_run.sql` form the evidence chain for one job or run. | Trace ids, actor ids, policy decisions, tool calls, receipts, prompt versions, model versions, and release/evaluation evidence complete the review. | The explanation depends on memory, informal notes, provider-only logs, or a chat transcript. |

Do not start by inventing a large admin product. Start by turning checked
runbook queries into a clear read surface.

## What the First Console May Control

The first write controls should be boring and few:

| Control | What it does | Required evidence |
| --- | --- | --- |
| Pause job kind | Stop new claims for a risky route or job kind. | Operator actor, reason, timestamp, previous state. |
| Resume job kind | Allow claims again after the condition is resolved. | Operator actor, reason, timestamp, readiness evidence. |
| Record failure drill result | Close a controlled failure experiment as passed, failed, or aborted. | Hypothesis, blast radius, injection, rollback action, required evidence, observed evidence, decision reason, and signoff. |
| Record release gate result | Persist a promote, canary-only, or block decision for a release candidate. | Candidate id, versions, eval receipt, SLO decision, compatibility decision, blockers, canary percentage, rollback plan, evaluator, and signoff. |
| Record first-user launch packet | Persist the decision that one job kind may touch real users. | Job kind, target level, risk class, ten proof statements, readiness review, release gate, failure drill for high risk, known gaps, reviewer, and next review date. |
| Request cancellation | Record durable intent to stop one job or run. | Requester, source, mode, reason, target identity. |
| Approve or reject action | Decide a policy-gated tool call or compensation. | Approver, decision, reason, proposal snapshot. |
| Escalate | Assign named human ownership. | Severity, target, owner, acknowledgement path. |
| Mark reconciliation needed | Prevent blind replay after uncertain side effect. | Receipt state, external correlation id, review owner. |
| Mark workflow reconciliation needed | Prevent product decisions from depending on a workflow history that disagrees with Postgres. | Workflow id, product row id, observed mismatch, owner, reason, and audit event. |
| Pause event publisher or consumer group | Stop event distribution or projection work while preserving product intake. | Actor, reason, topic, consumer group, last safe offset, affected event family, and resume condition. |

These controls are not direct row edits. They are typed commands that call the
same state transitions the worker and runbooks already trust.

For the first concrete control, the companion SQL uses two tables:

```text
agent_job_kind_controls:
  current pause state for the picker

agent_job_kind_control_events:
  append-only evidence for who changed the control, why, and from what state
```

The current state lets workers make a fast claim decision. The event row lets
humans review the decision later.

## Controls That Should Wait

Some controls are tempting but risky. Do not add them until the evidence model
is strong:

| Control | Why it is risky | Safer first version |
| --- | --- | --- |
| Force replay | May duplicate an external side effect. | Require receipt inspection and replay decision. |
| Replay workflow history as product truth | Workflow history may prove execution steps but not business authorization or audit evidence. | Reconcile workflow id, product rows, approvals, receipts, and audit events before any product mutation. |
| Replay Kafka topics blindly | Reprocessing old events may duplicate projections, side effects, notifications, or cross-tenant exposure. | Require event envelope validation, consumer receipts, schema compatibility, authorization checks, and replay rule approval. |
| Delete job | Erases evidence. | Cancel, dead-letter, or quarantine with reason. |
| Edit payload | Breaks audit history. | Create a new job linked to the old one. |
| Override policy silently | Removes accountability. | Require a policy exception record and reviewer. |
| Bulk approve | Turns human review into a rubber stamp. | Batch only low-risk decisions with sampled review evidence. |
| Change prompt/model live | Can alter behavior without release evidence. | Use release gate with evaluation receipt, durable release-gate row, and rollback path. |

The operator surface should reduce panic, not create a faster way to bypass the
system.

## Permission Model

Operator actions should be scoped. A useful first permission model is:

| Role | Allowed actions |
| --- | --- |
| Viewer | Read queue, run, approval, incident, and cost evidence. |
| On-call operator | Pause, resume, request cancellation, escalate, and annotate incidents. |
| Reviewer | Approve or reject policy-gated tool calls within assigned scope. |
| Security reviewer | Inspect denied authorization, sandbox, memory, and audit events. |
| Release owner | Promote, canary, block, or roll back a release candidate. |
| Administrator | Manage operator roles, but not bypass audit evidence. |

Avoid one global `admin` permission for all actions. The model can start
simple, but it should still separate read, operational control, approval,
security review, release, and role management.

## Audit Contract

Every control action should write an operation or audit event with:

```text
actor id
actor role
target kind
target id
action
reason
trace id or request id
previous state
new state
timestamp
```

The reason matters. An operator action without a reason is hard to review after
an incident, and a production agent system should expect later review.

## Tiny Incident

An alert fires:

```text
oldest_pending_age_seconds: 900
job_kind: incident_triage
provider route: deepseek
```

The console should not offer "retry all" as the first action. It should guide
the operator through evidence:

```text
1. Queue health shows one job kind is old.
2. Provider usage shows rate-limit errors.
3. Scheduled retries are increasing.
4. No expired leases exist.
5. Pause job kind is available with required reason.
6. Escalation is available if user-visible impact is high.
```

Read the tiny case as:

```text
setup: one job kind has stale pending work and provider pressure
transition: operator inspects evidence before pausing or escalating
evidence: queue, provider usage, retries, and operation event explain the action
invariant: an operator control must be evidence-led and auditable
```

## Console Readiness Checklist

Before exposing a control surface, verify:

```text
read:
  every view maps to a checked SQL file, API response, trace, metric, or event

maintenance:
  long-horizon views include job-kind lifecycle, storage pressure, retention
  review, credential lifecycle review, data-protection review, and failure
  drill status, not only current queue state

write:
  every action calls a typed command instead of editing rows directly

identity:
  every action records actor, role, target, reason, and timestamp

permissions:
  risky actions require explicit role scope and, when needed, human approval

redaction:
  model text, memory content, secrets, credentials, and personal data are not
  exposed by default

replay:
  any replay-like action checks receipts and replay decision state first

scaling:
  Temporal workflow history and Kafka offsets reconcile with Postgres product
  state, audit events, side-effect receipts, and trace ids before operators
  trust the summary

release:
  prompt, model, policy, schema, and worker changes go through release evidence

incident:
  the console can export or link the evidence packet used in a postmortem
```

If the console cannot satisfy this checklist, keep the runbook as the primary
control surface and improve the evidence model first.

## Summary

An operator surface is the product/control layer of a reliable agent system. It
should help humans inspect, decide, and act without bypassing the state machine.

The smallest safe console is not the one with the most buttons. It is the one
where every button maps to a typed transition, a permission check, and an audit
event.

## Further Reading and Sources

- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) supports the appendix's emphasis on operator action, incident evidence, ownership, toil, and safe production controls.
- [OpenTelemetry documentation](./31-credible-resources-further-reading.md#reliability-and-operations) supports separating traces, metrics, logs, and events when building the read side of the operator surface.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) supports the durable-state and audit-history expectations behind console actions.
- [OWASP Top 10:2025](./31-credible-resources-further-reading.md#security-abuse-and-governance) supports the permission, access-control, logging, exceptional-condition, and misconfiguration concerns in an operator console.
- [NIST AI Risk Management Framework 1.0](./31-credible-resources-further-reading.md#security-abuse-and-governance) supports the governance expectation that risky AI-assisted actions remain accountable and reviewable.
