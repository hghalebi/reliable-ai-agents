# Appendix V. Maintenance Cadence

## How to Use This Appendix

This appendix turns "run for years" into a calendar. Use it when the system is
already deployed or close to deployment and the team needs to know what to
check repeatedly.

The simple rule is:

```text
reliability decays unless somebody checks evidence on a schedule
```

This is not bureaucracy. A cadence is how the team keeps old jobs readable,
retried work safe, memory bounded, behavior evaluated, costs visible, and
restore procedures practiced after the first launch.

## Motivation

Long-running agent systems rarely fail only because one process crashes. They
fail because small assumptions expire. A prompt version becomes stale. A model
provider changes behavior. A runbook no longer matches the schema. A cost limit
is forgotten. A restore drill has not been practiced since the last migration.

If nobody owns those checks, the system can look healthy until the incident
that proves it has been drifting for months.

The maintenance cadence converts reliability into repeated evidence:

```text
daily:
  is the system healthy today?

weekly:
  what operational pattern is getting worse?

monthly:
  which promises, costs, evals, and policies need review?

quarterly:
  can we still restore, replay, rotate ownership, and survive provider change?
```

## Mental Model

Think of maintenance as a loop around the state machine:

```text
observe -> compare -> decide -> record -> improve
```

The loop is small on most days and larger every month or quarter. The important
part is that every pass leaves evidence. A team should be able to say:

```text
we checked this control
we saw this evidence
we accepted this risk or opened this gap
this owner has the next action
```

## Daily Checks

Daily checks should be short. They answer whether the service is safe to keep
running right now.

| Check | Evidence | Action if unhealthy |
| --- | --- | --- |
| Queue age | `/metrics`, `queue_metrics.sql`, `oldest_pending_job.sql`. | Inspect capacity, provider quota, pauses, and worker health. |
| Expired leases | `expired_leases.sql`. | Recover expired work or investigate worker shutdown. |
| Dead letters | `dead_jobs_by_reason.sql`. | Classify new reasons before replay or escalation. |
| Waiting approvals | `waiting_human_approvals.sql`. | Notify owners when risky work is blocked too long. |
| Failed tools | `failed_tool_calls.sql`. | Separate provider/tool failures from model behavior failures. |
| Security denials | `denied_authorization_events.sql`, `sandbox_policy_violations.sql`. | Escalate repeated abuse, prompt injection, or policy mismatch. |
| Cost and quota | `provider_usage_by_job_kind.sql`. | Slow admission, pause job kinds, or adjust provider routing. |
| Storage pressure | `storage_pressure_by_table.sql`. | Investigate fast-growing tables, dead-row pressure, or missing vacuum/analyze evidence before the database becomes the incident. |

The daily habit is not "look at dashboards." The habit is "answer the smallest
set of questions that proves the system can safely continue."

## Weekly Checks

Weekly checks look for trends and recurring operational pain:

| Check | Evidence | Decision |
| --- | --- | --- |
| Retry patterns | `scheduled_retries.sql`, failure-history rows. | Is one failure class growing? |
| Toil | Runbook history and operator notes. | Which repeated task needs automation or ownership? |
| Evaluation drift | Latest eval receipt by prompt/model version. | Did behavior quality move enough to block release? |
| Approval backlog | Approval age and escalation records. | Are humans keeping up with risky decisions? |
| Memory hygiene | `agent_memory_by_scope.sql`. | Are old or low-confidence memories still eligible? |
| Incident follow-up | Postmortem action items and owners. | Which corrective invariant is still missing? |

Weekly review should create a small improvement backlog. If the same item
appears for several weeks, it is not a note. It is a reliability gap.

## Monthly Checks

Monthly checks review the promises that change more slowly:

| Check | Evidence | Decision |
| --- | --- | --- |
| SLO and error budget | SLI rows, SLO reports, incident history. | Should releases slow down or can autonomy expand? |
| Cost envelope | Provider usage, token counts, latency, budget decisions. | Is this job kind still economically safe? |
| Prompt and model versions | Release gate reports, release-gate status query, and eval receipts. | Which old routes can be deprecated? |
| Job-kind lifecycle | `job_kind_lifecycle_review.sql`, pause/resume control events, release-gate status, and provider usage. | Which job kinds are active, deprecation candidates, retirement candidates, or retirement blocked? |
| Policy versions | Approval decisions, denials, exceptions. | Which policy rule needs revision or owner review? |
| Schema compatibility | `version_compatibility_risks.sql`. | Can current workers still understand old work? |
| Migration evidence | `schema_migration_status.sql`, release notes, compatibility query. | Which expand, backfill, or contract phase is still open, blocked, failed, or missing signoff? |
| Runbook accuracy | Runbook commands and incident notes. | Which command, variable, or interpretation rule is stale? |
| Dependency and advisory review | `cargo audit`, release notes, security advisories. | Which dependency or runtime upgrade is needed? |
| Credential lifecycle review | `credential_rotation_review.sql`, owner, policy version, due date, verification evidence, and exposure status. | Which credentials are due, overdue, stale, exposed, or recently revoked? |
| Retention review | `retention_review_by_surface.sql`, memory and audit policy. | Which evidence should be retained, archived, aggregated, redacted, or reviewed by a policy owner? |
| Data-protection review | `data_protection_review.sql`, request owner, policy version, due date, and completion evidence. | Which redaction, erasure, export, or retention-review request is open or overdue? |

The monthly review is where a team prevents "it still runs" from becoming "we
no longer know why it runs."

## Quarterly Checks

Quarterly checks prove the system can survive larger change:

| Check | Evidence | Decision |
| --- | --- | --- |
| Restore drill | RPO, RTO, replay decisions, side-effect receipts. | Can the system restore without duplicate external action? |
| Failure drill ledger | `failure_drill_status.sql`, drill owner, evidence count, and signoff. | Did controlled failure experiments prove or disprove their stated hypothesis? |
| Provider continuity | Provider contract, fallback route, smoke result. | Can critical job kinds survive provider degradation? |
| Ownership rotation | Owner map, escalation path, handoff evidence. | Does knowledge survive team change? |
| Security review | Abuse tests, denied actions, sandbox events, credential lifecycle review, and secret review. | Has the threat model changed? |
| Maturity review | Readiness scorecard by job kind. | Which maturity level is justified by evidence? |
| Scaling review | Queue metrics, latency, Postgres load, quota pressure. | Is the Postgres-first architecture still the right control layer? |
| Data retention, credential, and protection review | Memory, payload, event, audit, archive policies, `credential_rotation_review.sql`, and `data_protection_review.sql`. | Are we keeping too much, too little, the wrong evidence, stale credentials, or overdue privacy work? |
| Storage maintenance review | Table size, dead-row pressure, last vacuum/analyze evidence, and index health. | Is the ledger still cheap enough to query during incidents and audits? |

Quarterly review should be concrete. It should produce either proof that the
current operating model is still valid or a named change with an owner.

## Event-Driven Checks

Some reviews should happen when specific events occur:

| Event | Required check |
| --- | --- |
| New model route | Run behavior evals, provider smoke, cost review, and policy review. |
| New tool | Add typed contract tests, sandbox policy, approval rule, and side-effect receipt plan. |
| New job kind | Create readiness scorecard, SLO, runbook, owner, and evidence packet. |
| New external side effect | Add idempotency key, receipt, compensation path, and replay rule. |
| Schema migration | Run compatibility query, schema migration status query, old-row decoding tests, and backfill evidence review. |
| Release candidate | Record `release_gate_runs`, execute `release_gate_status.sql`, check blockers, canary percentage, rollback plan, and signoff before promotion. |
| Failure drill, simulation, or game day | Record `failure_drill_runs`, execute `failure_drill_status.sql`, compare required versus observed evidence, and open corrective work for failed or aborted drills. |
| Security incident | Review authorization, sandbox, memory, audit, and redaction evidence. |
| Major customer workflow change | Update evaluation dataset, runbook, policy, and approval surface. |

Event-driven checks prevent the calendar from becoming the only source of
discipline. Some risks deserve review immediately.

## Cadence Packet

Each maintenance review should leave a small packet:

```text
review kind:
review date:
job kinds covered:
evidence inspected:
decision:
accepted risk:
gap opened:
owner:
next review:
```

Keep the packet short. The value is not the document itself. The value is that
future operators can see what was checked, what was accepted, and what remains
open.

## What Can Fail

| Failure | Why it hurts | Corrective invariant |
| --- | --- | --- |
| Cadence without evidence | Meetings happen but reliability does not improve. | Every review points to rows, tests, runbooks, evals, or receipts. |
| Daily checks are too large | Operators skip them. | Daily checks answer only immediate safety questions. |
| Quarterly checks are too vague | Restore and ownership decay. | Quarterly checks include drills, handoffs, and scorecards. |
| Cost review is optional | Provider usage grows silently. | Cost and quota checks are part of normal reliability review. |
| Eval review is separated from release | Behavior drift reaches users. | Eval receipts and release gates are reviewed together. |
| Retention review is ignored | Memory and audit data become risky or useless. | Retention rules are reviewed by scope and evidence value. |

The cadence should make decay visible before it becomes an incident.

## Summary

Maintenance is the long-running form of reliability. A system that can run for
years needs more than good code at launch. It needs repeated evidence that the
state machine, provider boundary, policy gates, evaluation process, security
controls, recovery path, and owners still work.

The smallest durable rule is:

```text
daily safety, weekly trends, monthly promises, quarterly survival
```

If those reviews leave evidence and owners, the system can improve while it
runs. If they do not, reliability will drift quietly.

## Further Reading and Sources

- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) supports the cadence's focus on SLOs, toil, ownership, incident follow-up, and operations as software engineering.
- [Google SRE chapter: Testing for Reliability](./31-credible-resources-further-reading.md#reliability-and-operations) supports regular failure drills, restore checks, and reliability testing as repeated practice rather than one-time proof.
- [OpenTelemetry documentation](./31-credible-resources-further-reading.md#reliability-and-operations) supports the trace, metric, and log evidence used in daily and weekly reviews.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) supports reviewing durable state, schemas, logs, retention, and long-lived data-system behavior over time.
- [OWASP Top 10 for LLM Applications](./31-credible-resources-further-reading.md#security-abuse-and-governance) supports recurring review of prompt injection, tool abuse, memory poisoning, and data-exfiltration risks.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) supports periodic review of typed boundaries, public contracts, and maintainable Rust APIs as the code evolves.
