# Appendix Y. First Production Deployment Proof

## How to Use This Appendix

This appendix is a launch proof path, not a cloud recipe.

Use it when the book's ideas are clear and you want to ask one practical
question:

```text
What must be true before this agent job kind touches real users?
```

The answer is not "we deployed a container" or "the model responded once." The
answer is a small evidence packet that proves the serious MVP can admit work,
run work, observe work, stop risky work, and recover work without depending on
memory or luck.

The target architecture is still the minimal production stack:

```text
one API process
one worker process
one Postgres database
one Rig boundary
one readiness gate
one operator control path
```

Do not add a workflow engine, queue service, message bus, or orchestration
platform to pass this appendix. First prove the Postgres-backed system you
already have.

## The Simple Rule

Do not expose a new agent job kind to real users until the team can prove these
five things:

```text
1. intake is durable
2. execution is owned
3. side effects are controlled
4. behavior is evaluated
5. operators can stop, inspect, and recover the system
```

This is the smallest honest production claim. It is not the final maturity
level. It is the line between a demo and a service.

## Deployment Proof Ladder

Use this ladder in order. Do not skip a rung.

| Rung | Question | Evidence |
| --- | --- | --- |
| 1 | Can the API accept a request safely? | HTTP request requires `Idempotency-Key`, parses raw JSON into typed domain values, rejects malformed input, and writes durable work before the model runs. |
| 2 | Can the worker own work safely? | Worker claims a due row with a lease, records an event, heartbeats long work, and only the lease owner can complete, retry, or dead-letter the job. |
| 3 | Can the Rig boundary fail safely? | Provider output is parsed into typed output, malformed output is rejected, `DEEPSEEK_API_KEY` is required only for the live provider gate, and provider failure becomes a typed retry or terminal decision. |
| 4 | Can risky action be stopped? | Policy, approval, sandbox, and side-effect receipt evidence exist before any external action. |
| 5 | Can the team see what happened? | Trace id, operation event, audit event, metrics, job timeline, and runbook query explain one job and the fleet. |
| 6 | Can release and rollback preserve old work? | Schema, prompt, model, tool, policy, and worker versions remain attached to durable rows; rollback or pause plan is written. |
| 7 | Can recovery avoid duplicate side effects? | Restore/replay decision names pending, running, terminal, receipt-backed, and quarantined work. |

The ladder is deliberately boring. Boring means the system can be reviewed
under pressure.

## The First Launch Packet

Before launch, write one job-kind launch packet:

```text
job kind:
risk level:
target maturity level:
owner:
review date:

durable intake proof:
worker ownership proof:
provider boundary proof:
idempotency and side-effect proof:
policy or approval proof:
observability proof:
evaluation proof:
security proof:
rollback or pause plan:
restore and replay note:
known gaps:
launch decision:
```

Keep the packet short. The value is that each line points to evidence. If a
line points only to intention, the launch proof is incomplete.

## Durable Launch Packet

The launch packet should not live only in a document, chat thread, or private
memory.

For first-user exposure, store the packet in Postgres:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql:job_kind_launch_packets}}
```

This table says:

- the job kind, target level, risk class, owner, reviewer, and next review date
  are explicit;
- durable intake, worker ownership, provider boundary, side-effect control,
  approval, observability, evaluation, security, rollback, and restore evidence
  are named;
- approved or launched packets must link to readiness and release evidence;
- high-risk or regulated launch packets must link to a failure drill;
- known gaps are stored as data, not as vague memory.

Use this query during launch review:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/job_kind_launch_packet_status.sql}}
```

The query answers:

```text
Which job kinds are ready for first users?
Which job kinds are blocked by known gaps?
Which launch reviews are overdue?
Which high-risk launches still need failure-drill evidence?
```

At the Rust boundary, the database row becomes typed launch evidence:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/launch_packet.rs:launch_packet_row_boundary}}
```

The important idea is small:

```text
first-user launch is a typed, queryable decision
```

`job_kind_launch_packets` and `job_kind_launch_packet_status.sql` turn the
launch packet into a production record. `DbJobKindLaunchPacketStatusRow`,
`LaunchEvidenceChecklist`, and the `approved_for_first_users` decision keep the
packet inside the same raw-outside, typed-inside discipline as the rest of the
system.

## Minimum Commands

Run the offline readiness gate first:

```bash
./scripts/check-book-readiness.sh
```

For a deployment candidate, also run the local Postgres path when local
Postgres tools are available:

```bash
RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh
```

Run the live DeepSeek path only when changing the Rig provider boundary, prompt,
model routing, output parsing, or real agent binary:

```bash
RUN_LIVE_DEEPSEEK=1 ./scripts/check-book-readiness.sh
```

These commands answer different questions:

| Command | What it proves | What it does not prove |
| --- | --- | --- |
| `./scripts/check-book-readiness.sh` | Book, source excerpts, Rust tests, feature builds, clippy, docs, audit, and local source-coverage checks. | Live Postgres behavior or live provider behavior. |
| `RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh` | Schema application, real Postgres worker path, API smoke, runbook SQL, and audited pause/resume controls on a temporary local database. | Production capacity, real provider latency, or cloud networking. |
| `RUN_LIVE_DEEPSEEK=1 ./scripts/check-book-readiness.sh` | Real Rig-backed DeepSeek call, typed output parsing, and provider event evidence. | Postgres durability or fleet behavior. |

A serious launch can use more checks. It must not use fewer checks than the
risk of the job kind requires.

## First-User Checklist

Use this checklist before the first real users:

| Control | Minimum proof |
| --- | --- |
| Scope | The job kind is inside the Postgres-first operating envelope from Production Scope. |
| Intake | Duplicate input returns one logical job or one durable receipt path. |
| Worker | Drain, lease, heartbeat, retry, and dead-letter behavior are tested. |
| Rig | Provider output cannot become trusted state without parsing and validation. |
| Policy | Risky tool calls require policy and approval evidence before execution. |
| Side effects | External action has idempotency key, receipt, and compensation or replay rule. |
| Evaluation | Prompt, model, tool, and policy versions have an evaluation receipt or written low-risk exception. |
| Observability | One job can be reconstructed from trace id, events, metrics, and runbook SQL. |
| Security | Tool authority, tenant scope, egress, filesystem, memory, and secrets are checked outside the model. |
| Operations | Owner, escalation path, pause/resume command, rollback note, and next review date exist. |

The launch is blocked if the team cannot name the evidence for any required
control.

## Tiny Example

Suppose the first real job kind is `incident_triage`.

Do not launch because the model gives good advice once. Launch only when the
packet can say:

```text
durable intake proof:
  enqueue writes an agent job before the provider call

worker ownership proof:
  stale worker completion is rejected by lease-owner tests

provider boundary proof:
  malformed DeepSeek output is rejected before AgentResult exists

side-effect proof:
  notification send has an idempotency key and receipt

approval proof:
  risky escalation waits for durable human decision evidence

observability proof:
  job_event_timeline.sql and operation_events_by_job.sql explain the run

evaluation proof:
  release gate links prompt/model/tool/policy versions to an eval receipt

failure drill proof:
  the launch packet names at least one controlled failure drill or explains why
  the first drill is scheduled after launch with owner, blast radius, and rollback
```

The simple sentence is:

```text
incident_triage may launch because the team can prove the job is durable,
owned, typed, observable, evaluated, stoppable, and recoverable.
```

## Common False Launch Proofs

| False proof | Why it is weak | Better proof |
| --- | --- | --- |
| The prompt worked in a chat window. | Chat success does not prove durability, retries, authority, or recovery. | A durable job row, typed output parser, evaluation receipt, and event timeline. |
| The worker ran once locally. | One happy path does not prove lease ownership or retry safety. | Lease-owner tests, expired-lease recovery, retry classification, and dead-letter evidence. |
| The dashboard looks healthy. | A dashboard can hide missing state or unsafe controls. | Runbook SQL plus trace, audit, operation event, and metrics evidence. |
| The model chose the right tool. | Model choice is not authorization. | Policy decision, approval record if needed, sandbox event, and tool-call receipt. |
| Backups exist. | Backup existence does not prove replay safety. | Restore drill with receipt-aware replay and quarantine decisions. |
| A chaos experiment ran. | Random breakage does not prove the desired invariant. | `failure_drill_status.sql` with hypothesis, blast radius, injection, rollback action, required evidence, observed evidence, and signoff. |

False launch proofs are tempting because they are fast. Reliable systems need
proof that survives failure, not proof that the sunny path was pleasant.

## Operator Handoff

Before the first user-facing launch, hand the system to the operator with this
short note:

```text
job kind:
current status:
owner:
first runbook command:
pause command:
rollback or recovery action:
known risks:
when to escalate:
next review date:
```

If the operator cannot pause, inspect, or escalate the job kind, the system is
not ready for users even if the model behavior looks good.

## Summary

The first production deployment is a proof problem. The proof is not that a
binary started. The proof is that a real job kind can enter durable state, be
owned by one worker, pass through a typed Rig boundary, respect policy, leave
observable evidence, survive retry and deploy, and give operators a safe way to
pause, inspect, and recover.

The short rule is:

```text
launch one job kind only when its evidence packet can survive a review
```

## Further Reading and Sources

- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) supports the appendix's emphasis on launch evidence, ownership, runbooks, SLOs, and operational review.
- [Google SRE chapter: Testing for Reliability](./31-credible-resources-further-reading.md#reliability-and-operations) supports treating launch as a reliability proof that includes tests, drills, and expected failure behavior.
- [PostgreSQL transaction isolation documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) supports the durable intake and worker-ownership proof around transactions, locks, and concurrent workers.
- [Rig: Build AI Applications in Rust](./31-credible-resources-further-reading.md#agent-architecture) supports the separation between model/tool interaction and the surrounding reliability system.
- [OWASP Top 10 for LLM Applications](./31-credible-resources-further-reading.md#security-abuse-and-governance) supports the launch checks for prompt injection, tool abuse, data exposure, memory poisoning, and excessive agency.
- [NIST AI Risk Management Framework 1.0](./31-credible-resources-further-reading.md#security-abuse-and-governance) supports launch evidence for accountability, risk ownership, measurement, and governance.
