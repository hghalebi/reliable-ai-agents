# 23. Runbooks For Agent Job Systems

## What You Will Learn

This chapter teaches you to:

- explain what an operator should do when an agent job system is under stress;
- inspect diagnostic commands, expected evidence, decision points, escalation paths, and rollback or pause actions;
- verify that runbooks reduce confusion instead of hiding judgment.

The production evidence is a runbook that names the query to run, the evidence
to collect, the safe action to take, and the owner to notify.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** capacity and SLO signals identify pressure.
- **Adds:** checked diagnostic and control paths for tired operators.
- **Prepares:** incident response and postmortems when an invariant fails.

## Production Failure

At 2 a.m., a queue-age alert fires.

The operator sees five dashboards, three Slack guesses, and no exact first
query. They pause the wrong job kind and hide the evidence needed for the next
person.

- **What breaks:** signals existed, but safe action did not.
- **False fix:** write a vague note that says "check the queue and restart the
  worker."
- **Design response:** build runbooks that name the symptom, diagnostic query,
  expected evidence, safe control, escalation path, and record to update.

## Motivation

In production, incidents are handled by tired humans under time pressure. A good runbook reduces choice, names evidence, and prevents improvised unsafe actions.

Without checked runbooks, operators copy ad hoc queries, miss redaction rules, or replay work without receipts. This chapter treats runbooks as part of the production system interface.

## Plain Version

Read this as the simple version:

- **Simple rule:** A runbook is a checked path for diagnosing and controlling production, not a loose note to future operators.
- **Why it matters:** During an incident, people need exact questions, SQL files, controls, and escalation rules.
- **What to watch:** Watch whether each runbook query maps to a real table, a real symptom, a safe action, and recorded evidence.

## What You Already Know

Start with these anchors:

- Metrics, SLOs, and capacity signals can tell an operator that attention is needed.
- Signals are not the same as safe action.
- Stress makes vague instructions dangerous.

This chapter adds: runbook structure. You will write the shortest safe path
from symptom to evidence to decision to action, with escalation when judgment is
needed.

## Focus Cue

Keep three things in view:

- **State:** operator questions, diagnostic evidence, safe actions, and the reason recorded for each action.
- **Move:** a symptom becomes diagnosis and then action through a runbook path that preserves evidence.
- **Proof:** Checked SQL files, API health checks, diagnostics, pause/resume commands, replay rules, and evidence notes exist.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a checked SQL runbook suite for stuck jobs, retries, approvals, events, quotas, and controls.
- **Why it matters:** operators need exact questions and queries, not vague advice during an incident.
- **Done when:** on-call can find running, stuck, retrying, waiting, denied, and high-risk work from documented commands.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** checked SQL files under `examples/postgres-rig-agent-jobs/sql` and API control endpoints.
- **State transition:** answer operational questions with exact commands and reviewed queries.
- **Evidence path:** on-call can find stuck, retrying, waiting, denied, and overloaded work without guessing.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** What exact command answers the current incident question safely?
- **Evidence to inspect:** checked SQL file, command variables, expected result shape, interpretation rule, and escalation path.
- **Escalate if:** operators improvise queries or actions while tired and under pressure.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** an alert or operator question appears.
2. **Action:** run the checked diagnostic command before taking action.
3. **Persistence:** persist or capture the query result, decision, owner, and action note.
4. **Check:** verify the action follows evidence rather than improvisation.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** an incident question has a checked command and interpretation rule.
- **Validation path:** run or inspect checked SQL files, API control endpoints, and runbook tests.
- **Stop if:** operators must invent queries or actions under pressure.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, incidents are handled by tired humans under time pressure
rule: A runbook is a checked path for diagnosing and controlling production, not a loose note to future operators
tiny example: operator questions, diagnostic evidence, safe actions, and the reason recorded for each action
artifact: a checked SQL runbook suite for stuck jobs, retries, approvals, events, quotas, and controls
proof: an incident question has a checked command and interpretation rule
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

A runbook turns the event ledger into operational decisions:

```text
state query -> what is happening now?
timeline query -> how did one job get here?
control query -> what safe action can the operator take?
evidence note -> what should the team learn afterward?
```

The operator should not need to remember the schema during an incident. The
runbook should provide the shortest safe path from symptom to evidence to
action.

## Tiny Incident

Suppose an alert says:

```text
oldest_pending_age_seconds > 600
```

The operator should not immediately restart workers or add replicas. The
runbook asks narrower questions first:

```text
Are workers picking any jobs?
Are leases expired?
Are dead jobs increasing?
Is one job kind paused?
Is the provider rate-limiting requests?
Is one route or tenant burning the token or cost budget?
Is an SLO budget already exhausted for this job kind?
```

That sequence prevents a common failure: increasing worker count while the real
problem is a provider quota or paused job kind.

Read the tiny case as:

```text
setup: oldest_pending_age_seconds exceeds 600
transition: the operator follows questions from symptom to evidence to action
evidence: queue query, worker health, trace id, event timeline, and owner decision guide the runbook
invariant: operational action should be evidence-led under stress
```

## Queue Health

Show current queue health:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/queue_metrics.sql
```

Expected shape:

```text
pending | running | succeeded | failed | dead | cancelled | oldest_pending_age_seconds
```

Decision rule:

```text
oldest_pending_age high + running low -> workers not picking
oldest_pending_age high + running high -> capacity or provider bottleneck
dead increasing -> inspect reasons before replay
expired leases nonzero -> recover or investigate worker shutdown
```

## API Runtime Surface

When the API is part of the incident, check process health, dependency
readiness, and fleet queue evidence separately:

```bash
curl -fsS "$API_BASE_URL/healthz" >/dev/null
curl -fsS "$API_BASE_URL/readyz"
curl -fsS "$API_BASE_URL/metrics"
```

When you are validating the local Postgres-backed API service, use the checked
smoke script:

```bash
DATABASE_URL="$DATABASE_URL" \
  BIND_ADDRESS="127.0.0.1:3000" \
  scripts/smoke-postgres-api.sh
```

The script starts the API server, checks `/healthz`, `/readyz`, `/metrics`,
admits one idempotent job, and verifies that the pending queue count increases.

Use this interpretation:

```text
/healthz:
  the process can answer a local liveness check

/readyz:
  the process can read the queue dependency and return a typed readiness result

/metrics:
  the process can expose queue counts and oldest pending age for operators
```

If `/healthz` passes but `/readyz` fails, restart loops are not the first
question. Inspect database reachability, credentials, migrations, and connection
pool exhaustion. If both pass but `/metrics` shows stale pending work, move to
worker leases, provider quota, and pause-state diagnostics.

## Oldest Pending Job

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/oldest_pending_job.sql
```

Use this to answer:

```text
Which kind is stuck?
How old is it?
Which prompt/model/policy version will process it?
```

## Version Compatibility Risks

Pending work is not automatically safe work. A worker should process only the
payload schema range it understands:

```bash
psql "$DATABASE_URL" \
  -v minimum_payload_schema_version="1" \
  -v maximum_payload_schema_version="1" \
  -f examples/postgres-rig-agent-jobs/sql/version_compatibility_risks.sql
```

Use this before and after releases:

```text
Which pending or running jobs are too old for the current worker?
Which jobs are too new for the current worker?
Which prompt, model, tool, policy, and worker versions are attached?
Should the job be quarantined, migrated, or handled by a compatible worker?
```

## Schema Migration Status

Before promoting a schema, prompt, model, policy, or worker release that
depends on old-row compatibility, inspect the migration ledger:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/schema_migration_status.sql
```

This query answers:

```text
Which expand, backfill, or contract phase is planned, running, blocked, failed, or recently passed?
Which target surface and target version are changing?
Which payload schema versions remain compatible?
How many rows were examined and changed?
Which compatibility query and rollback plan are attached?
Which terminal migration has operator signoff?
```

If the query is empty, that may be fine for a fresh system. It is not fine for
a real migration that is already changing production behavior. Migration work
should leave durable evidence before the release gate treats it as complete.

## Release Gate Status

A release gate is not only a Rust enum. The gate result should also be durable
evidence that an operator can inspect later:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/release_gate_status.sql
```

This query answers:

```text
Which release candidates are blocked, canary-only, or recently promoted?
Which prompt, model, tool, policy, worker, and payload-schema versions are attached?
Which evaluation, SLO, compatibility, migration, and approval evidence was used?
Which blockers remain, what canary percentage was allowed, and who signed off?
What rollback plan is attached to the decision?
```

Do not treat a passing deploy as a release decision. The release gate row is the
reviewable evidence that explains why the system promoted, canaried, or blocked
the change.

## Failure Drill Status

A failure drill is only finished when its evidence is recorded. Inspect the
drill ledger before treating a simulation, staging chaos experiment, or limited
production game day as proof:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/failure_drill_status.sql
```

This query answers:

```text
Which drills are planned, running, failed, aborted, or recently passed?
Which scenario, environment, and owner are attached?
What hypothesis, blast radius, injection, and rollback action were declared?
How much required evidence was observed?
Which terminal drill has a decision reason and operator signoff?
```

Do not rely on a chaos dashboard alone. The drill row is the reviewable
contract: what did we try, how far could it spread, what evidence did we need,
what did we observe, and who accepted the result?

## Expired Leases

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/expired_leases.sql
```

Expected action:

```text
few rows after deploy -> recovery should clear them
many rows from same worker -> investigate worker health
many rows across workers -> provider/tool latency or database problem
```

## Running Jobs Past Deadline

Expired leases answer an ownership question. Breached deadlines answer a time
policy question:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql
```

This query answers:

```text
Which running or waiting agent runs exceeded their deadline?
Which timeout policy was in force?
Should the system retry, cancel, escalate, or dead-letter?
How long has the run been overdue?
```

Do not treat a breached deadline as the same thing as a crashed worker. The
worker may still hold a valid lease. The question is whether the work has
exceeded the system's promise.

## Pending Cancellation Requests

If an operator or timeout policy requested cancellation, inspect that request
separately from the job status:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql
```

This query answers:

```text
Which cancellation requests are still waiting to be applied?
Who requested the cancellation?
Was the request user, operator, system, or policy driven?
Should the worker stop immediately, gracefully, or after the current step?
What job status will the cancellation path observe?
```

Do not delete a job to stop work. A cancellation request is intent. The applied
or ignored cancellation record is evidence. Operators need both.

## Dead Jobs By Reason

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/dead_jobs_by_reason.sql
```

Do not blindly replay dead jobs. First decide whether the reason is:

```text
fixed configuration
fixed code
bad input
provider outage
policy rejection
```

Only replay after the root cause is removed.

## Event Timeline

For one job id:

```bash
psql "$DATABASE_URL" \
  -v job_id="00000000-0000-0000-0000-000000000000" \
  -f examples/postgres-rig-agent-jobs/sql/job_event_timeline.sql
```

The event timeline should explain every transition from enqueue to terminal
state.

If it does not, the event model is incomplete. A runbook cannot recover
evidence the system never recorded.

## Failure History By Job

For one scheduled job id:

```bash
psql "$DATABASE_URL" \
  -v scheduled_job_id="00000000-0000-0000-0000-000000000000" \
  -f examples/postgres-rig-agent-jobs/sql/failure_history_by_job.sql
```

Use this after the timeline tells you a job retried, failed, escalated, or
dead-lettered:

```text
Which attempt failed?
Was the source the worker, model provider, model output, tool, policy, sandbox,
timeout, or database?
Was the outcome retry, dead-letter, permanent failure, human escalation, or cancellation?
What retry state and workflow state did the system record at the time?
Which trace id and span id connect the failure to telemetry?
```

Do not treat the latest `last_error` as the full incident record. It is the
current summary. Failure history is the repeated-attempt evidence.

## Audit Events By Run

For one agent run id:

```bash
psql "$DATABASE_URL" \
  -v run_id="00000000-0000-0000-0000-000000000000" \
  -f examples/postgres-rig-agent-jobs/sql/audit_events_by_run.sql
```

Use this when the question is about authority:

```text
Which actor or system component made the decision?
Which action was recorded?
Which subject was affected?
Which evidence object supports the decision?
```

An audit event is not a debug log. It is durable decision evidence.

## Operation Events By Job

For one job id:

```bash
psql "$DATABASE_URL" \
  -v job_id="00000000-0000-0000-0000-000000000000" \
  -f examples/postgres-rig-agent-jobs/sql/operation_events_by_job.sql
```

Use this when the question is about runtime behavior:

```text
Which operational events happened for this job?
Which run did they correlate with?
Which trace id and span id connect this row to live telemetry?
Which events were warnings or errors?
Which message should appear in the incident timeline?
```

The audit query answers "who decided?" The operation query answers "what
happened?" Keeping them separate prevents incident review from mixing authority
evidence with runtime symptoms.

## Running Agent Runs

The compact queue tells you whether jobs are running. The tracking surface tells
you which agent runs are active and which workflow state they occupy:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/running_agent_runs.sql
```

Use this before restarting workers. A long-running run may be healthy if its
workflow state is `waiting_for_human`; it may be unhealthy if it is still
`executing_agent` after the provider timeout window.

## Pending Agent Handoffs

If work appears stalled between specialist agents, inspect pending handoffs
before increasing worker count:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql
```

This query answers:

```text
Which target agent has pending handoffs?
How old is the oldest handoff?
Is one specialist agent becoming a bottleneck?
Did a source agent hand off work that no target accepted?
```

A handoff is a responsibility transfer. It should be visible, idempotent, and
accepted before target work starts.

## Agent Memory By Scope

If a run appears to be influenced by remembered context, inspect memory
metadata before looking at raw content:

```bash
psql "$DATABASE_URL" \
  -v memory_scope="tenant:demo" \
  -f examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql
```

This query answers:

```text
Which memory records exist for this scope?
Which memory kinds and sources are eligible?
Are records short-term or long-term?
Which records have embedding references?
When were they created or last used?
```

The query intentionally omits raw memory content. Content inspection should
require a stronger authorization path than metadata diagnosis.

## Scheduled Retries

Retries should be visible future work, not hidden loops:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/scheduled_retries.sql
```

Use this to answer:

```text
Which failures are waiting for retry?
How many attempts remain?
When will the retry run?
Which failure class is driving the retry backlog?
```

## Waiting Human Approvals

Approval is durable state. During an incident, inspect the waiting approval
queue before bypassing policy or widening worker capacity:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/waiting_human_approvals.sql
```

If approvals are old, the bottleneck may be human review capacity rather than
worker capacity.

## Open Human Escalations

Escalation is not approval. Approval asks whether a risky action may proceed.
Escalation asks which human owner must take responsibility because autonomous
progress is no longer safe:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql
```

This query answers:

```text
Which human escalation is open or acknowledged?
What kind of boundary was reached?
Is it a review, ticket, or page?
Who owns it if it has been acknowledged?
Which job/run, prompt, model, and lifecycle state produced it?
```

Do not clear an escalation by editing the job status. A closed escalation needs
owner and resolution evidence.

## Failed Tool Calls

Tool failures are different from model failures. Inspect them separately:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/failed_tool_calls.sql
```

This query helps distinguish bad tool input, permission failures, provider
shape changes, and external service outages.

## Denied Authorization Events

A failed tool call means execution reached the tool boundary. A denied
authorization event means the request stopped earlier, at the permission
boundary:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/denied_authorization_events.sql
```

Use this before changing policy or approving a risky request:

```text
Which actor requested the action?
Which tenant did the actor belong to?
Which tenant was requested?
Which permission was missing or escalated?
Which policy version made the decision?
```

## Sandbox Policy Violations

Authorization can be correct while sandboxing still blocks the tool. Inspect
sandbox denials when a tool is valid and authorized but cannot access a
destination, scratch path, or runtime secret mode:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql
```

This query answers:

```text
Which tools are being denied by sandbox policy?
Which policy version made the denial?
Which resource rule is failing most often?
Did a prompt-injection attempt try to request a new egress destination?
```

Treat repeated sandbox denials as security signals, not only user errors. A
sudden rise in denied egress destinations, scratch-path traversal attempts, or
model-visible secret requests is an abuse investigation path.

## Credential Rotation Review

Secret values should not live in Postgres, logs, prompts, or runbook notes.
Secret references and lifecycle evidence should be queryable:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql
```

This query answers:

```text
Which credential kinds are registered?
Which credentials are due or overdue for rotation?
Which credential family has an open exposure incident?
Which credentials need verification because the evidence is stale?
Which credentials were revoked recently?
```

If `open_exposure_incidents` is nonzero, treat it as a security incident, not
routine maintenance. Pause affected job kinds, rotate the credential through
the external secret store, verify the new runtime path, and preserve the
rotation evidence without copying the secret value into the database.

## Side-Effect Receipts

Before replaying a risky run, inspect existing side-effect receipts:

```bash
psql "$DATABASE_URL" \
  -v run_id="00000000-0000-0000-0000-000000000000" \
  -f examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql
```

If a receipt exists, replay should not execute the external action again. It
should reconcile the stored receipt and external correlation id.

## Evaluation Receipts By Version

When a prompt, model, tool, or policy version changes, inspect recent behavior
evaluation receipts before promotion:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/evaluation_receipts_by_version.sql
```

This query answers:

```text
Which dataset and evaluator version judged the behavior?
Which prompt, model, tool, and policy versions were evaluated?
Did the evaluation pass or fail?
What score was recorded in basis points?
Which job kind and trace id connect the receipt to production evidence?
Does the report contain failed-case detail?
```

Do not treat a successful deploy as a behavior release. Deployment proves the
code can run. An evaluation receipt proves the behavior was checked for the
version that is about to receive traffic.

## Outbox Backlog

If side effects are delayed but jobs look healthy, inspect the outbox before
restarting workers:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/outbox_backlog.sql
```

This query answers:

```text
Which event kinds are waiting to publish?
How old is the oldest due event?
Are publication leases expired?
Which event kind is failing independently of agent execution?
```

The outbox is operational state. A job can finish while its downstream event is
still pending publication. Treat that as visible backlog, not as a hidden
thread or best-effort callback.

## Compensation Backlog

If a side effect was wrong and the system requested a compensating action,
inspect the compensation backlog before restarting workers or replaying jobs:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/compensation_backlog.sql
```

This query answers:

```text
Which compensations are waiting for approval or execution?
Which compensation kind is oldest?
Are execution leases expired?
Which compensating actions are failing repeatedly?
```

Compensation is a separate operational surface because it is a new side
effect. It should have approval evidence, idempotency, a lease, retry state,
and terminal evidence of success, failure, or cancellation.

## Restore Replay Candidates

After a database restore or serious state-loss incident, start workers paused
and inspect replay candidates before resuming job kinds:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql
```

This query answers:

```text
Which restored jobs can resume from durable state?
Which jobs already have side-effect receipts?
Which jobs expected a side effect but lack a restored receipt?
Which terminal jobs must not replay?
```

Treat `quarantine_missing_receipt` as a stop sign. The operator should inspect
the external system or idempotency record before resuming that job kind.

## Provider Usage And Budget Pressure

When provider errors, queue age, or spend increase, inspect usage by job kind
before changing worker count:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql
```

This query answers:

```text
Which job kind is spending the most?
Which provider route is rate-limiting?
Are latency and cost rising together?
Which route should be paused, throttled, or reviewed?
```

If the query is missing, the system can still spend money, but it cannot prove
why. That is an observability and budget-control gap.

## Job Kind Lifecycle Review

When a prompt route, model route, tool version, or job kind looks old, do not
delete it from memory. First inspect whether live work, retries, approvals, or
recent provider calls still depend on it:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/job_kind_lifecycle_review.sql
```

This query answers:

```text
Which job kinds still have pending, running, retrying, or human-waiting work?
Which paused job kinds have no recent provider usage?
Which job kinds are deprecation candidates but not retirement candidates yet?
Which latest release decision is tied to each job kind?
```

Retirement is not a delete button. It is a control decision. A safe retirement
needs no open work, no waiting retries, no pending human decisions, no recent
provider usage, and a durable pause or replacement path.

## Job Kind Readiness Review

When a team says a job kind is ready, ask which level it is ready for. Then
inspect the durable readiness review:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/job_kind_readiness_review.sql
```

This query answers:

```text
Which job kinds target demo, prototype, production, or regulated/high-risk use?
Which job kinds are below their target level?
Which reviews are overdue?
Which blocking gaps still exist?
Who owns the next change?
Which latest release decision supports the readiness claim?
```

Readiness is not a feeling. A job kind should not move from prototype to
production, or from production to regulated/high-risk, until the row can name
evidence counts, blocking gaps, owner, next change, review date, and latest
release evidence.

## Storage Pressure By Table

When the system has been running for months, inspect the ledger itself before
assuming slow runbooks are caused by the model or worker:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/storage_pressure_by_table.sql
```

This query answers:

```text
Which evidence tables are largest?
Which tables have high estimated dead-row pressure?
When did vacuum or analyze last touch each table?
Which table should be investigated before an incident review becomes slow?
```

This is a read-only query. It is not a command to delete or vacuum blindly.
Use it to decide whether to review indexes, autovacuum settings, archive
policy, or query shape.

## Retention Review By Surface

When audit, memory, event, or receipt tables keep growing, inspect retention
pressure by evidence surface:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/retention_review_by_surface.sql
```

This query answers:

```text
Which surfaces have rows older than 90 days?
Which surfaces have rows older than 365 days?
Which rows are audit evidence, replay evidence, memory, cost evidence, or restore evidence?
Which surfaces need archive, aggregation, redaction, or policy-owner review?
```

Retention is not "delete old rows." Retention is a policy decision about which
evidence must remain queryable, which evidence can be aggregated, which data
must be redacted, and which records must be preserved for replay or audit.

## Data Protection Review

When a user, tenant, policy owner, or regulator asks for redaction, erasure,
export, or retention review, do not track the request in chat. Inspect the
durable request ledger:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/data_protection_review.sql
```

This query answers:

```text
Which evidence surfaces have open data-protection requests?
Which requests are overdue?
Which surfaces need redaction or erasure action before more work proceeds?
Which surfaces recently completed privacy work?
```

Use it before changing retention policy or replay behavior. Redaction and
erasure are side effects on the evidence backbone, so they need policy version,
audit evidence, and a receipt-like completion record.

## SLI Measurement Rows

When an alert is tied to an SLO, inspect the measurement row behind the alert
instead of trusting a dashboard label:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql
```

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/sli_terminal_jobs_not_dead.sql
```

These queries answer:

```text
Which SLO and SLI are being measured?
Which job kind and window are affected?
What target is being used?
How many good events and total events exist?
Is this no traffic, within budget, or budget exhausted after typed evaluation?
```

The last question belongs to the typed domain layer. The SQL returns raw
measurement rows. The Rust `SloMeasurement` boundary validates them before they
drive paging, release pauses, or escalation decisions.

## Pause And Resume A Job Kind

Pause a hot or unsafe job kind:

```bash
psql "$DATABASE_URL" \
  -v kind="incident_triage" \
  -v actor="oncall@example.com" \
  -v reason="provider quota exhausted" \
  -f examples/postgres-rig-agent-jobs/sql/pause_job_kind.sql
```

Resume it:

```bash
psql "$DATABASE_URL" \
  -v kind="incident_triage" \
  -v actor="oncall@example.com" \
  -v reason="provider quota recovered and retries are stable" \
  -f examples/postgres-rig-agent-jobs/sql/resume_job_kind.sql
```

Both commands update the current control row and write an
`agent_job_kind_control_events` row. The event records actor, reason, previous
state, new state, and event id, so pause and resume decisions can be reviewed
after the incident.

## What Not To Do

Do not:

```text
delete running rows during deploy
replay dead jobs before fixing the reason
increase worker count during provider rate limiting
store emergency secrets in payloads or events
turn off approval gates to clear a queue
```

## Formal Definition

For this chapter, the precise definition is:

```text
A runbook is an operator transition protocol that reads evidence before action and preserves the reason for any state change.
```

In the book's system model:

- **State:** operator questions, diagnostic evidence, safe actions, and the reason recorded for each action.
- **Actor:** the on-call operator follows checked commands and changes state only after reading current evidence.
- **Transition:** a symptom becomes diagnosis and then action through a runbook path that preserves evidence.
- **Evidence:** Checked SQL files, API health checks, diagnostics, pause/resume commands, replay rules, and evidence notes exist.
- **Invariant:** incident response remains deliberate and auditable under stress.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Runbooks are prose advice without checked commands. |
| Production symptom | Operators improvise during incidents and copy risky SQL from notes. |
| Corrective invariant | Runbooks execute named, reviewed diagnostic and control artifacts. |
| Evidence to inspect | `psql -f` commands point to checked SQL files for health, timelines, audit events, operation events, breached deadlines, handoff backlog, sandbox violations, side-effect receipts, evaluation receipts, outbox backlog, compensation backlog, provider usage, restore replay, pause, and resume. |


## Production Contract

Every runbook action should satisfy three conditions:

```text
it reads current state before changing state
it records or preserves the reason for the action
it avoids making side effects less safe under pressure
```

An emergency runbook that bypasses approval, deletes rows, or hides the reason
for replay is not an operations tool. It is another failure mode.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Runbooks are prose advice without checked commands. | A runbook that starts with a command can mutate state before anyone understands the evidence. |
| Safer version | Runbooks execute named, reviewed diagnostic and control artifacts. | Runbooks read diagnostic SQL first and make operator actions conditional on observed state. |
| Production version | `psql -f` commands point to checked SQL files for health, timelines, audit events, operation events, breached deadlines, handoff backlog, sandbox violations, side-effect receipts, evaluation receipts, outbox backlog, compensation backlog, provider usage, restore replay, pause, and resume. | Every pause, resume, replay, approval, behavior release, or recovery action has a checked query and evidence note. |

Use the naive row when instructions are action-first. Use the safer row to read before writing. Use the production row before giving operators control of agent state.

## Testing Strategy

Test runbooks as checked operator protocols:

- **Unit or type test:** prove Rust or script-level argument validation rejects missing job ids, schema ranges, memory scopes, and pause/resume reasons.
- **Persistence or boundary test:** execute the checked Postgres SQL files against the schema so diagnostics, pause, resume, replay, and escalation queries remain valid.
- **Regression test:** forbid inline ad hoc `psql -c` diagnostics in the book; runbooks must keep using reviewed SQL files with explicit variables.

## Observability Strategy

Observe operator actions as controlled transitions:

- Emit structured `tracing` fields for runbook name, operator id, job id, SQL file, variables, action, reason, result, and trace id.
- Record an operation event before and after diagnostic reads, pause, resume, replay, escalation, approval, or recovery actions.
- The runbook query should be the checked SQL file itself, so the operator can repeat the diagnostic without inventing an ad hoc command.

## Security and Safety Considerations

Runbooks are powerful write paths and need safety controls:

- Treat operator input, psql variables, incident notes, and pasted diagnostics as untrusted until constrained by checked SQL files and typed commands.
- authorization, sandboxing, and approval should govern pause, resume, replay, compensation, and any destructive or externally visible runbook action.
- Redact secrets and customer payloads from incident notes while preserving operator id, SQL file, variables, reason, and result evidence.

## Operational Checklist

Use this checklist before relying on operator runbooks as executable evidence paths in production:

- **State:** Each runbook question maps to a checked SQL file, expected evidence, and
  safe operator action.
- **Boundary:** Operators inspect named query outputs instead of copying raw SQL or
  trusting unstructured logs.
- **Failure:** A runbook can diagnose stuck jobs, expired leases, dead letters, retries,
  approvals, failed tools, and receipts.
- **Observability:** Every command returns state that can be tied to job id, run id,
  trace id, owner, and current action.
- **Safety:** Control runbooks such as pause, resume, replay, or compensation require
  authorization and redacted output.

## Exercises

1. Write a negative test where a stuck job cannot be diagnosed because the runbook query
   lacks trace id, worker id, or idempotency evidence. Explain which idempotency key,
   receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: one checked SQL runbook file for queue, lease, retry,
   approval, receipt, and pause/resume questions.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   RunbookQuestion, DiagnosticQuery, OperatorAction, and PauseDecision types. Then name
   the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What is the safe path from symptom to action?
- Explain: Why should a runbook ask evidence questions before changing worker count?
- Apply: Write the first three questions for `oldest_pending_age_seconds > 600`.
- Evidence: Each question should point to a SQL file, durable row, trace id, or owner decision.

## Summary

The runbook is ready when a tired operator can inspect, pause, explain, and recover the system without inventing a new query during an incident.

- **Invariant:** every operational question maps to a checked command, safe query, expected evidence, and allowed action.
- **Evidence:** runbook SQL files, pause/resume controls, trace ids, job ids, lease state, retry rows, approval queues, receipts, and incident notes answer the question.
- **Carry forward:** runbooks are part of the system interface.

## Changed Understanding

- **Before this chapter:** a runbook looked like notes for when something goes wrong.
- **After this chapter:** a runbook is an executable operator path from symptom to evidence to safe action.
- **Keep:** run the checked SQL or command path that turns an alert into a safe operator action.

## Further Reading and Sources



- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: (1998). A foundational paper for SRE and Resilience Engineering. It identifies why "Human Operators" are the adaptable element that keeps inherently hazardous systems (like agents) safe, provided they have the right evidence.
- [Google SRE: Managing Incidents](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: (Chapter 14). Provides the formal Incident Command System (ICS) framework used to separate strategy (Incident Commander) from execution (Operations), which the runbook structure in this chapter supports.
- [PagerDuty: Incident Response](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: The industry standard for the on-call lifecycle (Detect, Triage, Diagnose, Remediate). It provides the practical context for the "Mitigation First" rule used in these runbooks.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: Explains the cognitive science behind "Slip" vs "Mistake" and why runbooks must be designed to reduce choices during high-stress periods.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann). Connects runbook queries to the "Observability vs. Monitoring" distinction and the formal requirements for a reviewable event ledger.