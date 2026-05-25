# 18. Deployment And Operations

## What You Will Learn

This chapter teaches you to:

- explain how a service changes while old work is still running;
- inspect deploy order, schema compatibility, worker shutdown, secret rotation, readiness checks, and runbook commands;
- verify that deployment does not strand leases or make old jobs unreadable.

The production evidence is a release path that preserves compatibility,
observability, secrets, graceful shutdown, and rollback or pause procedures.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the test suite proves the core invariants.
- **Adds:** release, health, migration, shutdown, and runtime operating discipline.
- **Prepares:** long-horizon compatibility for systems that run for years.

## Production Failure

A new worker version deploys while old jobs are still running.

The migration changed the payload shape, and the new worker cannot decode rows
created by the previous version.

**What breaks:** deployment treated running work as if it disappeared between
releases.

The queue did not disappear during the deploy. The database still contained old
payloads, old leases, old prompt versions, old policy versions, and old
operation history. The new binary entered a world that already had work in it.
When that binary could not understand the old world, the deploy became a data
compatibility failure.

**False fix:** drain the queue manually and hope no old rows remain.

Manual draining may be useful during an emergency, but it is not a deployment
strategy. It depends on timing, operator memory, and luck. A new job can arrive
while the drain is happening. A long-running job can still hold a lease. A dead
letter can still need replay next week.

**Design response:** deploy with compatibility checks, typed configuration,
readiness, graceful drain, rollback, and versioned migration evidence. The
release should prove that old work remains readable, new workers start safely,
and operators can pause, inspect, or roll back before state is lost.

## Motivation

In production, deployment changes the system while work is already running. Old jobs, new workers, migrations, secrets, readiness checks, and shutdown behavior all interact.

Without deployment discipline, a release can strand work, corrupt state, break replay, or hide an unsafe prompt change. This chapter treats deployment as part of the reliability design.

## Plain Version

Read this as the simple version:

**Simple rule:** Deploy agent systems as versioned software with health checks,
migrations, rollback paths, and operator visibility.

A deployment is not only "start the new binary." It is a controlled change to
code, schema, prompts, model versions, policies, workers, and runtime secrets.

**Why it matters:** Long-running jobs can outlive a deploy. The job was created
under one version of the system, but it may finish under another. Old and new
code must handle durable state carefully.

**What to watch:** Watch schema versions, prompt versions, model versions,
readiness checks, migration order, worker drain behavior, and rollback
evidence. These are the signals that tell you whether the old system and the
new system can safely overlap.

## What You Already Know

Start with these anchors:

- Tests prove contracts before release.
- Deployment tests whether old and new code can coexist.
- Jobs may still be running while schema, prompts, policies, or workers change.

This chapter adds: operational release discipline. You will preserve leases,
readiness, shutdown, secrets, and compatibility while the service changes under
live work.

## Focus Cue

Keep three things in view:

- **State:** running work, old payloads, schema version, worker version, secrets, health, readiness, and operator controls during deploy.
- **Move:** a deploy proceeds only while old work remains readable, workers shut down safely, and operators keep control.
- **Proof:** Shutdown, pause, replay, version, migration, health, readiness, metrics, and provider smoke evidence exist.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

**Artifact:** a deployment runbook with typed configuration, readiness checks,
worker drain, rollback, and secret rules.

This runbook is part of the system. It tells an operator how to start the new
version, prove it is ready, stop old workers without stealing or deleting work,
rotate credentials, pause unsafe job kinds, and return to a previous version.

**Why it matters:** long-running jobs must survive deploys while new processes
prove they are safe to serve. A release that only works when the queue is empty
is not a reliable release.

**Done when:** startup fails fast on bad config, workers stop claiming before
shutdown, and rollback preserves old work.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** config parsing, API/worker binaries, readiness endpoints, graceful shutdown, and release scripts.
- **State transition:** start only with valid configuration and stop workers without stealing active work.
- **Evidence path:** deploy, rollback, drain, readiness, and secret checks are explicit.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Can this deploy start, drain, roll back, and preserve old work safely?
- **Evidence to inspect:** typed config, readiness result, worker drain log, version compatibility query, and rollback note.
- **Escalate if:** new code serves traffic or claims jobs before config, migrations, compatibility, or drain behavior is proven.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** a new version is about to run beside existing work.

There may be pending jobs, running jobs, dead-lettered jobs waiting for replay,
approval requests waiting for humans, and outbox events waiting to publish. The
deploy must respect all of them.

**Action:** validate config, readiness, migrations, drain behavior, and rollback
path. Do this before the new process claims work or receives user traffic.

**Persistence:** persist version and compatibility evidence. The system should
record which release, schema version, prompt version, model version, and policy
version were active when work moved.

**Check:** verify old jobs remain safe while new workers start. A healthy deploy
does not reinterpret old rows silently. It either reads them correctly or
quarantines them with evidence.


## Acceptance Gate

Do not move on until this minimum evidence exists:

**Minimum evidence:** deploy, drain, readiness, rollback, and config rules
protect old work.

**Validation path:** inspect config tests, readiness endpoints, worker shutdown
behavior, compatibility SQL, and release notes. These checks should prove both
sides of the transition: the new version can start, and the old durable work is
still meaningful.

**Stop if:** new code can claim jobs before config, migrations, compatibility,
or rollback are proven. Claiming work is authority. Do not grant that authority
until the process has proved it can operate safely.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, deployment changes the system while work is already running
rule: Deploy agent systems as versioned software with health checks, migrations, rollback paths, and operator visibility
tiny example: running work, old payloads, schema version, worker version, secrets, health, readiness, and operator controls during deploy
artifact: a deployment runbook with typed configuration, readiness checks, worker drain, rollback, and secret rules
proof: deploy, drain, readiness, rollback, and config rules protect old work
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Treat deployment as a compatibility problem:

```text
old rows must still decode
old events must still explain history
old prompt/model/policy versions must still be meaningful
running leases must survive worker replacement
operators must still have a safe pause and recovery path
```

The unit of safety is not "the binary started." The unit of safety is "old and
new work can coexist without corrupting state."

> ### 🎓 The Professor's Corner
>
> **The Rolling Upgrade: Changing Tires on the Highway**
>
> A **Rolling Upgrade** is like changing the tires on a car while it's still driving down the highway! You replace one tire at a time (one worker) so the car never has to stop. 
> 
> In our system, the "New Code" and "Old Code" have to drive together for a few minutes. If they don't agree on where the road is (the database schema), the car will crash! This is why compatibility is our most important rule.

This mental model changes how you read every deploy step. A migration is not
only a schema change; it is a promise about old rows. A readiness endpoint is
not only a liveness check; it is a promise that dependencies and migrations are
usable. A worker shutdown is not only process termination; it is a promise that
leased work will either finish or become recoverable.

## Deployment Topology

Start simple. The first production topology does not need a large platform:

```text
Postgres
one API process that validates requests and enqueues jobs
one or more worker processes
one metrics/dashboard surface
one operator runbook
```

Scale workers horizontally only after the queue model is correct.
`FOR UPDATE SKIP LOCKED` gives worker cooperation, not unlimited throughput.
Capacity is bounded by Postgres, provider quotas, tool servers, and human
approval throughput.

This topology is intentionally boring. The API owns admission. The worker owns
durable execution. Postgres owns coordination and history. The operator surface
owns diagnosis and control. 

> ### 🎓 The Professor's Corner
>
> **The Boring Bridge: Postgres as the Connector**
>
> Think of Postgres as a "Boring Bridge" that connects the old worker to the new worker. They don't need to talk to each other; they don't even need to know each other exists! 
> 
> They only need to talk to the same database. As long as they both follow the rules of the bridge, the work can cross safely from the old version to the new one without anyone falling off.

If those boundaries are unclear, adding more
infrastructure will make the deploy harder to reason about, not safer.

## Tiny Example

A deploy starts while `worker-a` owns a long-running job:

```text
job_id: incident-123
status: running
locked_by: worker-a
locked_until: 10:05
worker_build_id: 2026-05-23.a
```

The safe deploy does not delete the row or invent a new status. It lets the
worker finish, extend the lease, or shut down and allow recovery after
`locked_until`. The database contract is the bridge between old and new
processes.

In AI, we should treat this `worker_build_id` as part of the **AI Lineage**. If a worker produces a "weird" result, I need to know if it was the "Morning Build" or the "Afternoon Build." This is the data we need for **Release Regression Analysis**.

Read the tiny case as:

```text
setup: a worker owns a long-running job during deploy
transition: shutdown stops new claims while durable state remains recoverable
evidence: readiness state, lease expiry, worker build id, and runbook command show safe rollout
invariant: deployment must not strand old work or make old rows unreadable
```

## Graceful Shutdown

On shutdown:

```text
stop picking new jobs
finish current short job when possible
extend lease if shutdown drain is expected
otherwise allow lease expiry and recovery
record shutdown reason in process logs
```

Do not delete running rows during deploy. Let the lease model do its work.

By heartbeating one last time or voluntarily releasing the lease during shutdown, you reduce **Recovery Latency**. I call this **Lease Relinquishment**.

The worker exposes the drain gate as a typed control state. When the process is
draining, the worker returns before `pick_due_job`, so due work stays pending for
another healthy worker or a later restart:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/worker.rs:run_once_controlled}}
```

The process-level loop is also bounded. This matters during deploys because a
worker should make progress, report why it stopped, and return control to the
supervisor instead of hiding inside an unbounded local loop:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/worker.rs:run_bounded_loop}}
```

This is intentionally small. The important production rule is not a framework
feature. It is the invariant:

```text
draining worker -> no new claims
already-owned short job -> finish if safe
already-owned long job -> heartbeat or let lease expire for recovery
```

## Secrets

Use environment variables or a secret manager for:

```text
DATABASE_URL
DEEPSEEK_API_KEY
RUST_LOG
LOG_FORMAT
provider-specific API keys
operator auth credentials
```

Never store secrets in job payloads, event messages, or `last_error`.

The companion code treats environment variables as an adapter boundary. Raw
process strings become typed runtime configuration before the API server,
Postgres worker, or DeepSeek runner starts:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/config.rs:runtime_config}}
```

Runtime logging follows the same boundary. `RUST_LOG` and `LOG_FORMAT` are raw
process strings until they pass through the tracing config:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/logging.rs:runtime_tracing_config}}
```

The deploy should fail early if the log filter or format is invalid, because a
worker that silently drops structured events is harder to operate during
incidents.

This is the same rule used for HTTP requests, database rows, provider output,
and tool input:

```text
raw outside
typed inside
```

The DeepSeek key is checked for presence without storing or printing the secret
value. The database URL becomes a `DatabaseUrl`. The optional bind address
becomes an `HttpBindAddress` or the process fails before serving traffic.
Configuration failure should be boring, early, and explicit.

## Credential Lifecycle

Secrets do not become safe because they are hidden in an environment variable.
A long-running agent system also needs to know which credential exists, who
owns it, when it was last rotated, when it is due again, and whether exposure
or revocation work is open.

The companion schema stores credential lifecycle metadata in
`credential_assets`. It stores a `secret_ref`, not the secret value. The value
lives in the runtime environment, platform secret store, or external secret
manager. Postgres stores the operational proof:

```text
secret reference
credential kind
owner
storage location
status
last rotation
next rotation due date
verification evidence
exposure or revocation evidence
policy version
```

Operators review the lifecycle with the checked query:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql
```

This query answers:

```text
Which credential families have active references?
Which credentials are due or overdue for rotation?
Which credentials have open exposure incidents?
Which credentials have not been verified recently?
Which credentials were revoked in the last 30 days?
```

The important boundary is simple:

```text
secret value outside Postgres
secret reference and lifecycle evidence inside Postgres
```

This lets the system stay auditable without turning the database into a secret
store.

## Runbook Commands

The first runbook should include:

```text
show pending/running/dead counts
show oldest pending job
show expired leases
show events for one job id
cancel one job
replay one dead job after fixing root cause
pause one job kind or tenant
```

The companion SQL directory includes executable query contracts for these
operator actions. Chapter 23 turns them into a concrete on-call runbook.

## API And Worker Processes

The production MVP has two application processes:

```text
postgres-api-server:
  accepts HTTP requests
  requires Idempotency-Key
  converts request JSON into typed domain values
  enqueues durable work

postgres-worker-demo:
  claims due work
  calls the agent boundary
  writes terminal, retry, or dead-letter evidence
```

Keep these responsibilities separate. 

> ### 🎓 The Professor's Corner
>
> **The Receptionist and the Chef: API vs. Worker**
>
> I call this split **Process Isolation**. The API is the "Receptionist" who takes the order and writes it down in the notebook. The Worker is the "Chef" who actually cooks the meal in the back! 
> 
> If the receptionist tries to cook, the lobby gets crowded and the orders get lost! By keeping them separate, the receptionist can keep taking orders even if the chef is busy with a big, slow meal.

If the API calls the model directly, a
client timeout can become an ambiguous execution. If the worker accepts raw
HTTP JSON, the raw-outside/typed-inside boundary has moved too far into the
system.

Production run shape:

```bash
DATABASE_URL="$DATABASE_URL" \
  BIND_ADDRESS="127.0.0.1:3000" \
  cargo run --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features api-server,postgres-store \
  --bin postgres-api-server
```

The API service should usually bind privately behind your ingress or internal
network first. Public exposure adds authentication, rate limits, abuse
controls, and tenant authorization; it does not change the admission invariant.

## Release Smoke Surface

After a deploy, smoke the API with separate liveness, readiness, and queue
evidence checks:

```bash
curl -fsS "$API_BASE_URL/healthz" >/dev/null
curl -fsS "$API_BASE_URL/readyz"
curl -fsS "$API_BASE_URL/metrics"
```

The companion repository packages the same check as an executable smoke script:

```bash
DATABASE_URL="$DATABASE_URL" \
  BIND_ADDRESS="127.0.0.1:3000" \
  scripts/smoke-postgres-api.sh
```

When changing the real Rig boundary, run the provider smoke separately:

```bash
DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" scripts/smoke-deepseek-agent.sh
```

This check is intentionally credential-gated. It proves that the real
DeepSeek-backed runner can complete one worker pass, produce a typed result,
and leave agent-start and agent-success event evidence. It should not replace
the deterministic worker tests or the Postgres smoke path.

Read the results separately:

```text
healthz passed, readyz failed:
  process is alive, but the queue dependency is not usable

readyz passed, metrics show old pending work:
  process can reach the queue, but workers or downstream dependencies may be stuck

metrics show dead jobs increasing:
  do not declare the deploy healthy until the failure reason is understood
```

This is the deployment version of the book's larger pattern: an operational
surface is useful only when it points to the next evidence source.

## Formal Definition

For this chapter, the precise definition is:

```text
A deployment is a controlled compatibility transition where old work, current leases, secrets, schema, and operator controls remain valid.
```

In the book's system model:

- **State:** running work, old payloads, schema version, worker version, secrets, health, readiness, and operator controls during deploy.
- **Actor:** the release operator, API process, worker process, and readiness gates coordinate compatibility transitions.
- **Transition:** a deploy proceeds only while old work remains readable, workers shut down safely, and operators keep control.
- **Evidence:** Shutdown, pause, replay, version, migration, health, readiness, metrics, and provider smoke evidence exist.
- **Invariant:** changing the service does not strand, corrupt, or silently reinterpret existing work.

## What Can Fail

**Design smell:** deploy assumes workers can stop at any point without a
protocol. This usually means shutdown has not been designed. The process may
die while holding a lease, after a model call, before recording a receipt, or
while an approval is pending.

**Production symptom:** running jobs are abandoned, duplicated, or completed by
stale code. Operators see old leases, unexplained retries, dead letters, or
side effects without matching release evidence.

**Corrective invariant:** shutdown, lease expiry, versioning, and restart
behavior are designed. The worker knows when to stop claiming, the database
knows when a lease expires, and old payload versions are either readable or
quarantined.

**Evidence to inspect:** worker lifecycle, lease rules, version fields, and
runbook commands explain deploy safety. You should be able to prove why a
running job was allowed to finish, recover, retry, or stop.


## Production Contract

A deployment is safe only if a few promises remain true.

No running work is deleted. Old payload versions are still understood or
explicitly quarantined. API and worker binaries can run from the same schema
contract. Workers stop picking before they exit. Leases provide recovery after
process loss. Operators can pause, inspect, cancel, and replay. Secrets are
supplied by runtime configuration, not payloads.

These promises are deliberately plain. A reliable deployment does not need to
sound sophisticated. It needs to preserve work, preserve evidence, and preserve
operator control while the system changes.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Deploy assumes workers can stop at any point without protocol. | Deployment becomes a restart event that ignores in-flight leases, old schemas, and provider credentials. |
| Safer version | Shutdown, lease expiry, versioning, and restart behavior are designed. | Operations define startup, shutdown, readiness, migration, secrets, pause, and rollback behavior. |
| Production version | Worker lifecycle, lease rules, version fields, and runbook commands explain deploy safety. | A deploy preserves old work, proves new compatibility, and gives operators controls before traffic changes. |

Use the naive row when deployment is just shipping code. Use the safer row to protect active work. Use the production row before a release touches real queues.

## Testing Strategy

Test deployment as compatibility with work already in flight:

**Unit or type test:** prove Rust config, shutdown, compatibility, and
release-gate types reject missing env vars, unsupported schemas, and unsafe
promotion evidence. Bad configuration should fail before traffic or workers
start.

**Persistence or boundary test:** prove Postgres readiness, pause/resume,
version compatibility, and queue diagnostics work before and after a migration.
The database is the coordination backbone, so migration safety must be tested
against database evidence.

**Regression test:** simulate deploy while jobs are running. Old leases and old
payload versions must remain explainable or be quarantined safely. This is the
test that would have caught the production failure at the start of the chapter.

## Observability Strategy

Observe deployments as compatibility transitions:

Emit structured `tracing` fields for release version, worker build, schema
version, prompt version, model version, job id, trace id, and shutdown phase.
These fields connect a production event to the release that caused or handled
it.

Record an operation event when readiness changes, workers drain, job kinds
pause or resume, migrations apply, or compatibility quarantine happens. Those
events tell the story of the rollout after the terminal scrollback is gone.

The runbook query should reveal whether old work remains parseable and whether
current traffic is served by compatible code. During an incident, the operator
should not have to guess which version touched a job.

## Security and Safety Considerations

Deployment changes can accidentally widen trust boundaries:

Treat environment variables, migration inputs, old payloads, and release
metadata as untrusted until startup and compatibility checks validate them. A
bad environment value is still untrusted input, even if it came from your own
platform.

authorization, sandboxing, and approval policies must remain compatible while
old and new workers coexist. A deploy must not create a temporary window where
old work can bypass a new safety boundary or new workers can misread old policy
state.

Redact deployment secrets and provider keys while preserving release version,
worker build, schema version, and rollback evidence. Operators need evidence,
not secret values.

## Operational Checklist

Use this checklist before relying on deployment while work is running in production:

- **State:** Old and new workers can read existing jobs, respect leases, drain work, and
  record release evidence.
- **Boundary:** Runtime environment variables, migrations, secrets, and provider
  configuration are validated before startup.
- **Failure:** A deploy can roll back, pause admission, drain workers, or quarantine
  incompatible jobs without losing state.
- **Observability:** Release id, worker version, schema version, prompt version,
  compatibility query, and readiness endpoint are visible.
- **Safety:** Deploys preserve secret redaction, approval policy, sandbox policy, and
  receipt replay rules.

## Exercises

1. Write a negative test where a new worker sees an old payload schema and quarantines
   it instead of corrupting idempotency or receipt evidence. Explain which idempotency
   key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: migration evidence, version_compatibility_risks output,
   and release operation_events for one rollout.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   RuntimeConfig, WorkerCompatibilityPolicy, ReleaseGateDecision, and ShutdownMode
   types. Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What must remain true while old and new code coexist?
- Explain: Why does graceful shutdown matter for leased work?
- Apply: A deploy starts while a worker owns a long-running job. Decide what the worker should stop doing.
- Evidence: Name the durable state, lease expiry rule, readiness signal, secret handling, and rollback or pause command.

## Summary

Production is the system plus its operating surface. If an on-call engineer cannot inspect, cancel, replay, and explain work during deploys, the service is not ready.

**Invariant:** deploys preserve old work, readable state, leases, secrets,
readiness checks, and rollback paths.

**Evidence:** migration results, version compatibility queries, release events,
readiness endpoints, worker shutdown logs, and runbook commands prove the
rollout.

**Carry forward:** a release is safe only when old jobs still have a valid path.

## Changed Understanding

- **Before this chapter:** deployment looked like shipping the service binary.
- **After this chapter:** deployment is a controlled change to code, schema, workers, prompts, models, policies, and operational gates.
- **Keep:** check the release gate across code, schema, prompt, model, policy, eval, and rollback evidence.

## Further Reading and Sources



- [Google SRE: Release Engineering](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: The definitive guide to building fast, repeatable, and automated deployment pipelines using hermetic builds and immutable artifacts.
- [AWS Builders' Library: Avoiding Overload](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: Explains the "One-Box" and "AZ-by-AZ" rollout strategies used to limit blast radius during production changes.
- [GoCardless: Zero-Downtime Postgres Migrations](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: A practical deep-dive into the "Expand and Contract" pattern and the importance of `lock_timeout` when changing schema under live traffic.
- [NIST SP 800-57 Part 1 Rev. 5: Recommendation for Key Management](./31-credible-resources-further-reading.md#security-abuse-and-governance) Read this because: Provides the formal vocabulary and lifecycle (generation, usage, rotation, destruction) for the secrets and credentials introduced in this chapter.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann, Chapter 10: Batch Processing). Connects worker drain behavior and "Lease Expiry" to the formal requirements of fault-tolerant offline systems.