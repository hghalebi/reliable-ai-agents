# Appendix E. Production Readiness Scorecard

## How to Use This Scorecard

Use this scorecard when you need to decide whether a specific agent job kind is
ready for production traffic, wider autonomy, or a higher maturity level. Score
one job kind at a time. A low-risk summarizer and a payment agent should not
share the same readiness claim.

The scorecard is not a substitute for Appendix C's design review or Appendix
D's failure drills. It is the evidence packet that summarizes them.

For every row, write:

```text
target:
current evidence:
gap:
next change:
owner:
review date:
```

If the evidence is only a sentence about intended behavior, mark the row as not
ready. Production readiness means a specific artifact exists and can be
inspected.

## Rating Rules

Use four ratings:

```text
missing:
  no durable evidence exists

partial:
  evidence exists, but it is incomplete, manual, or not tied to the job kind

ready:
  evidence exists, is repeatable, and is owned

not required:
  the job kind does not need this control yet, and the reason is written down
```

Do not average the ratings. A single missing control can block production if it
guards the highest-risk failure for that job kind.

## Job-Kind Scorecard

| Area | Production question | Ready evidence |
| --- | --- | --- |
| Durable work | Does work exist before model execution? | Job row, idempotency key, enqueue event, duplicate-intake test. |
| API admission | Does the HTTP edge validate and type requests before enqueue? | `Idempotency-Key` requirement, request-to-command conversion, domain validation errors, duplicate admission test. |
| State model | Can the job be only in valid states? | Rust enum, database constraint, transition tests, terminal-state predicates. |
| Lease ownership | Can only the lease owner mutate running work? | `locked_by` and `locked_until` predicates, heartbeat test, expired-lease recovery test. |
| Timeout policy | Does slow work have an explicit deadline and action? | Deadline fields, typed timeout policy, breached-deadline runbook query, retry/cancel/escalate/dead-letter action. |
| Cancellation control | Can users, operators, and policies stop work with durable evidence? | Cancellation request row, typed cancellation lifecycle, pending-cancellation query, applied/ignored/expired evidence. |
| Retry policy | Are retries typed and bounded? | Transient/permanent error classes, backoff policy, attempt cap, retry events. |
| Dead letters | Does failed work stop visibly? | Dead state, reason field, event timeline, runbook query. |
| Provider boundary | Is provider behavior isolated from worker logic? | Agent-output parser, adapter fixtures, typed provider errors, malformed-output tests, compatibility tests. |
| Idempotent side effects | Can replay avoid duplicate external action? | Side-effect idempotency key, receipt, replay check, external correlation id. |
| Policy gate | Can risky actions be proposed without being authorized by the model? | Policy version, decision record, risk level, denial path. |
| Human approval | Are human decisions durable and auditable? | Actor, timestamp, reason, proposal snapshot, approval or rejection event. |
| Human escalation | Does unsafe autonomous progress reach a named human owner? | Escalation target, kind, severity, status, assigned owner, acknowledgement, and resolution evidence. |
| Agent handoff | Can specialist agents transfer responsibility without losing ownership? | Source run, source agent, target agent, handoff reason, idempotency key, target job, decision evidence, and pending-handoff query. |
| Observability | Can one job and the fleet be explained separately? | Job event timeline, trace id, `/healthz`, `/readyz`, `/metrics`, queue metrics, oldest pending age by kind. |
| Operator control surface | Can humans inspect, pause, resume, cancel, approve, escalate, or reconcile without bypassing the state machine? | Checked read views, typed command handlers, permission checks, actor evidence, reason fields, and audit or operation events. |
| Audit and operation evidence | Can decisions and runtime symptoms be reconstructed without private memory? | Audit events by run, operation events by job, typed row conversion, actor/action/subject evidence, severity/message evidence. |
| SLOs | Does alerting map to a reliability promise? | SLI query, typed measurement conversion, SLO window, owner, burn-rate alert, runbook link. |
| Capacity | Can overload be controlled before collapse? | Concurrency limit, admission control, quota handling, backpressure signal. |
| Release safety | Can old work survive code, prompt, model, policy, and schema changes? | Versioned rows, worker compatibility policy, compatibility-risk query, migration ledger, schema-migration status query, release-gate status query, canary criteria, rollback path, typed release gate report, and durable release-gate row. |
| Scaling path | Can added infrastructure preserve the same reliability evidence? | Baseline metrics, invariant-to-component map, coexistence test, trace correlation, runbook update, and rollback criteria. |
| Behavior evaluation | Can prompt and model changes be judged before release? | Dataset version, rubric, grader, score, reviewer, eval receipt. |
| Agent memory | Can remembered context influence future runs without becoming hidden authority? | Memory scope, kind, source, confidence, horizon, retention policy, redaction, row conversion, and memory-by-scope query. |
| Security boundary | Can untrusted text cross into tools, memory, tenant data, egress, filesystem, or secrets? | Tool authorization, sandbox events, scoped credentials, memory policy, abuse test, audit log. |
| Tenant isolation | Can one tenant's actor cause a read, write, memory lookup, or side effect for another tenant? | `authorization_events`, `tenant_boundary_review.sql`, typed tenant keys, policy version, denial reason, trace id, and audit event. |
| Credential lifecycle | Can secret references be rotated, verified, revoked, and investigated without storing secret values? | Credential asset rows, owner, policy version, rotation due date, exposure status, credential-rotation review query, and typed row conversion. |
| Disaster recovery | Can restore and resume avoid duplicate side effects? | Backup, restore drill, RPO, RTO, receipt-aware replay procedure. |
| Failure drills | Can controlled failure experiments prove one invariant without becoming random breakage? | Failure-drill ledger, hypothesis, blast radius, injection, rollback action, required evidence, observed evidence, decision reason, and operator signoff. |
| Extreme fault tolerance | Can critical execution keep serving from last-known-good state when non-critical control surfaces fail? | Fault-tolerance review row, control-plane status, execution-plane status, redundant worker count, isolated failure domain, static-stability mode, failover drill, release gate, and next review date. |
| Ownership | Does someone own the next failure? | Service owner, escalation path, incident role, toil review cadence. |
| Maintenance cadence | Does the job kind have daily, weekly, monthly, quarterly, and incident-triggered review evidence? | Operating calendar, owner, evidence source, last review, next review, and missed-review escalation. |

## Promotion Rule

Promotion is allowed only when the target level has evidence for the controls
that match the job kind's risk.

```text
Level 1 promotion:
  durable work, state model, lease ownership, retry policy, dead letters
  timeout policy for any job with a user-facing or operator-facing deadline

Level 2 promotion:
  provider boundary, typed payloads, versioned rows, adapter tests

Level 3 promotion:
  observability, SLOs, capacity controls, runbooks, release safety
  scaling path evidence when infrastructure changes are part of the promotion

Level 4 promotion:
  behavior evaluation, policy gates, human approval where risk requires it

Level 5 promotion:
  security boundary, tenant isolation review, credential lifecycle review, data-protection review,
  disaster recovery, ownership rotation, deprecation path backed by
  tenant_boundary_review.sql, credential_rotation_review.sql, data_protection_review.sql, and
  job_kind_lifecycle_review.sql
```

If a row is marked `not required`, the scorecard must explain why the job kind's
risk does not need that control yet. For example, an internal summarizer may not
need human approval, but it still needs durable work, typed payloads, provider
boundary handling, and visible failures.

## Evidence Packet

A serious readiness review should leave behind a small packet:

```text
job kind and target maturity level
completed scorecard rows
links to tests and runbook queries
latest failure-drill answers
latest design-review gaps
current production risks
owner and next review date
```

This packet should be easy to review after an incident. If the team cannot
reconstruct why a job kind was considered ready, the readiness decision was not
operational.

## Executable Readiness Boundary

The companion project includes a typed version of this scorecard:

```text
job_kind_readiness_reviews
  -> job_kind_readiness_review.sql
  -> DbJobKindReadinessReviewRow
  -> JobKindReadinessReview
```

Use it to keep maturity labels out of slide decks and inside reviewable
evidence. The database row can store text-friendly values such as
`production`, `regulated_high_risk`, `high`, and `ready_for_target`, but Rust
must decode them into `MaturityLevel`, `JobRiskClass`, `ReadinessEvidence`,
and `JobKindReadinessStatus` before application logic trusts the claim.

The important rule is:

```text
raw maturity label outside
typed readiness decision inside
```

For example, a regulated job kind cannot target ordinary `production` in the
typed boundary. It must target `regulated_high_risk`, and the row conversion
rejects a lower target before the readiness claim can be used.

## Summary

Readiness is not a mood and not a demo. It is a claim backed by evidence. The
scorecard makes that claim concrete: one job kind, one target level, one set of
controls, one owner, and one next review date.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
