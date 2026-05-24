# 11. The Real Postgres Store

## What You Will Learn

This chapter teaches you to:

- explain where the in-memory model meets real database rows;
- inspect the conversion layer between raw storage fields and typed domain objects;
- verify that database rows cannot leak invalid status, attempts, timestamps, or payloads into business logic.

The production evidence is a Postgres store with migrations, constraints,
row-to-domain conversions, transactions, and tests for invalid stored data.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the serious MVP names the required boundaries.
- **Adds:** validated row-to-domain conversion for real persistence.
- **Prepares:** idempotent side-effect control on top of durable storage.

Part I gave you the shape of the agent system while the state still lived in
memory. That was useful because it kept the first state machine small enough to
see.

This chapter crosses the line into production reality. The state now lives in a
database that can outlive the process, the deploy, and the person who originally
wrote the job. That is the point of Postgres. It is also the danger. Durable
state is useful only if the application can trust what it reads back.

So the central question changes from "Can the worker move a job forward?" to
"Can the worker prove that the row it read is a valid job before it moves
anything forward?"

## Production Failure

An operator repairs a stuck job row during an incident and leaves it with
`status = running`, `locked_by = null`, and a stale attempt count.

The next worker reads the row and treats it as valid work.

- **What breaks:** storage shape leaks directly into domain logic.
- **False fix:** trust every row because it came from Postgres.
- **Design response:** decode raw rows through validated conversion before
  worker logic can act on them.

## Motivation

In production, a database row is not automatically valid domain state. Rows can be old, partial, inconsistent, manually repaired, or produced by an earlier version of the system.

Without a store boundary, invalid rows leak into worker logic and turn persistence problems into runtime surprises. This chapter makes row conversion a reliability boundary, not a mechanical mapping step.

## Plain Version

Read this as the simple version:

- **Simple rule:** The database boundary must convert storage rows into validated domain objects.
- **Why it matters:** Raw database values are convenient for storage but unsafe as the shape of application logic.
- **What to watch:** Watch row decoding, status checks, timestamp consistency, payload validation, and domain conversion errors.

## What You Already Know

Start with these anchors:

- Part I used an in-memory store to make the state machine easy to see.
- Production needs rows, migrations, constraints, and transactions.
- Database fields are raw until decoded into domain types.

This chapter adds: the real Postgres store. You will keep storage-friendly row
shapes at the boundary and convert them into validated domain objects before
business logic runs.

## Focus Cue

Keep three things in view:

- **State:** storage-friendly database rows at the edge and validated domain objects inside the application.
- **Move:** raw rows become domain state only after conversion validates lifecycle evidence, payload shape, versions, leases, and results.
- **Proof:** Row conversion rejects corrupted state before business logic, and SQL predicates preserve ownership.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a database row conversion layer between raw SQL rows and typed domain models.
- **Why it matters:** Postgres stores practical values, but application logic should not inherit raw storage shapes.
- **Done when:** invalid rows fail at the boundary and valid rows become typed jobs, runs, steps, calls, and events.

The artifact is small in code but large in meaning.

A database row uses shapes that are convenient for storage: `text`, `integer`,
`timestamptz`, and `jsonb`. A worker needs shapes that carry meaning:
`JobStatus`, `AttemptCount`, `WorkerId`, `LeaseUntil`, `AgentPayload`, and
`TraceId`. The conversion layer is where storage becomes meaning.

This conversion layer acts as our **Anti-Corruption Layer**. In large systems, the database is often shared or accessed by different versions of the code. By forcing every row through a `TryFrom` conversion, we ensure that **Schema Drift** or **Manual Edits** don't poison our application's reasoning. It's the **Validation Boundary** that protects our **State Machine Invariants**.

Do not treat this as boilerplate. In a long-running agent system, the row
converter is often the first component that sees corruption after a bad
migration, a manual repair, an old worker version, or a partial incident
response. If it accepts impossible state, every later layer has to defend itself.
If it rejects impossible state early, the system fails in the right place.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/postgres_store.rs`, migration SQL, row conversion tests, and live Postgres gate.
- **State transition:** turn storage-friendly rows into validated domain state inside a transaction boundary.
- **Evidence path:** bad rows fail at decode and good rows preserve job lifecycle invariants.

The important files play different roles.

The migration defines what Postgres is allowed to store. The row type describes
what the adapter can read. The `TryFrom` conversion decides whether those raw
values are coherent enough to become domain state. The store methods execute
transitions with SQL predicates so workers cannot complete work they do not own
or mutate terminal rows.

> ### 🎓 The Professor's Corner
>
> **Data Integrity: The Shared Whiteboard**
>
> Think of the database as a "Giant Shared Whiteboard." Everyone in the office can write on it! But if someone writes nonsense—like a job that is "Succeeded" but has no result—the worker shouldn't try to read it and get confused. 
> 
> Instead, the worker should just "point at the whiteboard and say No!" This is what **Data Integrity** means: we only let facts that make sense into our "Worker's Club." It's the secret to a happy, bug-free life!

Read those pieces together. A constraint without a decoder is not enough,
because old rows and manual repairs still exist. A decoder without SQL
predicates is not enough, because two workers can race. A store without tests is
not enough, because the most dangerous bugs appear only when state is
inconsistent.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Did a raw database row become typed domain state only after validation?
- **Evidence to inspect:** database row type, TryFrom conversion, domain value, validation error, and transaction boundary.
- **Escalate if:** invalid storage values can leak into worker logic or provider calls.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** the worker reads or writes real database rows.
2. **Action:** convert rows through a boundary layer before domain logic sees them.
3. **Persistence:** persist state changes inside transaction and lease predicates.
4. **Check:** verify invalid rows fail early and valid rows preserve lifecycle rules.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** raw database rows cannot leak into core domain logic.
- **Validation path:** run row-conversion, store, SQL, and optional live Postgres checks.
- **Stop if:** invalid rows can be decoded into running work or valid rows lose lifecycle evidence.

This gate is intentionally stricter than "the worker runs against Postgres."

Running once proves the happy path. A store boundary must prove the unhappy
paths. It should reject a running row with no lease owner, a succeeded row with
no result, a failed row with no error evidence, a payload that is not an object,
and a row whose attempt count no longer makes sense. Those are not exotic edge
cases. They are the exact kinds of shapes that appear after crashes, bad
backfills, interrupted migrations, and emergency database edits.

The acceptance gate exists to keep those shapes from becoming invisible
production risk.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, a database row is not automatically valid domain state
rule: The database boundary must convert storage rows into validated domain objects
tiny example: storage-friendly database rows at the edge and validated domain objects inside the application
artifact: a database row conversion layer between raw SQL rows and typed domain models
proof: raw database rows cannot leak into core domain logic
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

The Rust worker talks to a trait:

```text
AgentJobStore
```

Two implementations can satisfy the same contract:

```text
InMemoryAgentJobStore       -> fast local learning
PostgresAgentJobStore       -> durable production boundary
```

The worker should not care which store is underneath. That separation is the
reason the state machine can be tested without infrastructure and then moved to
real storage.

## Tiny Example

In memory, a job can be a Rust struct. In Postgres, the same job is a row with
text, timestamps, integers, and JSON.

The boundary must recover meaning:

```text
row.status = 'running' -> JobStatus::Running
row.locked_by = 'worker-a' -> WorkerId
row.payload -> AgentPayload
row.payload_schema_version = 1 -> PayloadSchemaVersion
```

If that conversion fails, the correct behavior is not to guess. The adapter
should surface the problem before the worker mutates the job.

Read the tiny case as:

```text
setup: Postgres stores raw text, integers, timestamps, and JSON
transition: row conversion decodes raw fields into typed domain values
evidence: `TryFrom` errors, constraints, and tests reject invalid stored state
invariant: database convenience types do not become application architecture
```

## Compile The Store

The real store is behind the `postgres-store` feature:

```bash
cargo test \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features postgres-store
```

The store uses the same SQL contracts from the ledger chapter.

The example depends on `sqlx-core` and `sqlx-postgres` rather than the broad
`sqlx` facade. That keeps the lockfile aligned with the Postgres-only system
being taught and prevents unused MySQL, SQLite, and macro dependencies from
appearing in security audits.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/postgres_store.rs}}
```

## Apply The Schema

For a local database, the example includes a Postgres compose file:

```bash
docker compose \
  -f examples/postgres-rig-agent-jobs/docker-compose.postgres.yml \
  up -d
```

Then set:

```bash
export DATABASE_URL='postgres://rig_agents:rig_agents_dev@localhost:55432/rig_agents'
```

Apply the schema:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql
```

In a production deployment, this should be run by your migration system, not by
an ad hoc shell command.

Run the Postgres-backed worker binary:

```bash
cargo run \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features postgres-store \
  --bin postgres-worker-demo
```

The full readiness gate can run the same path when Docker is available:

```bash
RUN_LIVE_POSTGRES=1 ./scripts/check-book-readiness.sh
```

If Docker is not available but local Postgres binaries are installed, use the
ephemeral local database smoke:

```bash
RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh
```

That live gate also executes the operator runbook SQL files for queue health,
oldest pending work, expired leases, dead jobs, event timelines, and pause or
resume controls. Production SQL should be validated as an operator artifact,
not only as text included in the book.

## Boundary Rule

The Postgres store converts raw database rows into domain types immediately:

```text
text status        -> JobStatus
text kind          -> JobKind
jsonb payload      -> AgentPayload
jsonb result       -> AgentResult
text worker id     -> WorkerId
text idempotency   -> IdempotencyKey
int payload version -> PayloadSchemaVersion
text prompt version -> PromptVersion
text model route    -> ModelRoute
text policy version -> PolicyVersion
```

That conversion is where corrupted rows or migration drift should fail loudly.
Do not let database DTOs leak into the worker.

The word "immediately" matters.

If a raw row travels through two or three layers before validation, every layer
must remember that the row might be malformed. That is how invalid states spread.
The safer pattern is simple: raw outside, typed inside. The Postgres adapter is
outside. The worker, policy code, Rig boundary, and tool execution code are
inside.

Once the conversion succeeds, the rest of the application can use the type
system as evidence. A `RunningJob` has lease evidence. A `SucceededJob` has a
result. A `RetryableFailure` has failure evidence and a next retry time. The
compiler cannot prove the database was always correct, but it can help ensure
that only validated database state enters the domain.

This is where we must also respect the **Master Clock**. In distributed systems, clocks are unreliable. The database's `timestamptz` is our Master Clock. The application should always use the database's time for leases and `finished_at` to avoid "Clock Skew" issues between different worker nodes. We call this **Monotonic Progress**.

The adapter also checks lifecycle evidence that the database normally enforces:

```text
running row     -> locked_by and locked_until must both exist
non-running row -> locked_by and locked_until must both be absent
succeeded row   -> result must exist and last_error must be absent
other row        -> result must be absent
payload/result   -> JSON must be an object before domain decoding
```

This duplication is intentional. The database constraint protects stored state.
The Rust conversion protects the application if a migration, manual repair, or
future adapter accidentally reads a malformed row. The same invariant is
checked at both sides of the boundary because corrupt durable state should be
detected before the worker makes another transition.

## Versioned Rows

A row that may be replayed next year must explain the context in which it was
created:

```text
payload_schema_version
prompt_version
model_route
tool_version
policy_version
worker_build_id
```

These fields are not decoration. They answer incident and replay questions:

```text
Which prompt produced this result?
Was this before or after the policy change?
Can the current worker still parse this payload?
Should replay use the old model route or the current default?
```

The companion schema stores these fields directly on `agent_jobs`, and the
Postgres adapter converts them into typed Rust values before the worker sees the
row.

Version fields are what make old work understandable.

Without them, a replay becomes guesswork. The current prompt may not be the
prompt that created the original plan. The current model route may not behave
like the old one. The current policy may reject an action that an older policy
allowed, or allow an action that the older system should not have attempted.

For reliable agents, replay is not "run the same prompt again." Replay is
"resume or reconcile a specific recorded operation under explicit historical
evidence." Versioned rows provide that evidence. This is the foundation of **Reproducible AI**.

## Formal Definition

For this chapter, the precise definition is:

```text
A Postgres store boundary is the adapter that converts storage-friendly rows into validated domain values and preserves SQL transition contracts.
```

In the book's system model:

- **State:** storage-friendly database rows at the edge and validated domain objects inside the application.
- **Actor:** the Postgres store adapter decodes rows and executes SQL transitions for the worker.
- **Transition:** raw rows become domain state only after conversion validates lifecycle evidence, payload shape, versions, leases, and results.
- **Evidence:** Row conversion rejects corrupted state before business logic, and SQL predicates preserve ownership.
- **Invariant:** database representation never leaks unchecked into worker logic.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Database rows are trusted as domain values. |
| Production symptom | Bad persisted data reaches worker logic and fails far from the boundary. |
| Corrective invariant | Row conversion validates lifecycle evidence before constructing domain objects. |
| Evidence to inspect | Store conversion tests reject invalid attempts, versions, payload shape, result shape, lease drift, missing success results, stale success errors, and negative counts. |


## Production Contract

The Postgres store is correct only when:

```text
all database rows validate before entering domain logic
SQL state transitions preserve lease ownership and terminal states
version fields are present on replayable work
adapter errors preserve useful context without leaking secrets
the worker depends on the store trait, not on database DTOs
```

The database is allowed to store generic shapes. The application boundary is
not allowed to keep them generic.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Database rows are trusted as domain values. | Trusting database rows as domain values lets corrupted state leak into business logic. |
| Safer version | Row conversion validates lifecycle evidence before constructing domain objects. | Row conversion validates status, attempts, lease evidence, payload shape, result shape, and timelines. |
| Production version | Store conversion tests reject invalid attempts, versions, payload shape, result shape, lease drift, missing success results, stale success errors, and negative counts. | The store boundary rejects malformed persistence before worker, API, or agent logic can act on it. |

Use the naive row when SQL shape is treated as domain truth. Use the safer row to parse storage into meaning. Use the production row when corrupted rows must fail closed.

The hardening path is not about adding ceremony.

The naive version is attractive because it is short. The query returns a row,
and the worker acts on it. That is fine for a demo. It is dangerous when rows
can be written by old code, future code, migrations, operators, tests, or
incident-recovery scripts.

The safer version adds a named boundary. The production version makes the
boundary executable: the tests show which malformed rows are rejected, and the
SQL predicates show which state transitions are legal under concurrency.

## Testing Strategy

Test row decoding as a trust boundary:

- **Unit or type test:** prove Rust store conversion rejects negative attempts, missing lease evidence, invalid payload schema versions, non-object payloads, and impossible success rows.
- **Persistence or boundary test:** prove Postgres transitions preserve ownership, terminal-state rules, result shape, failure evidence, and queue metrics.
- **Regression test:** load a corrupted row fixture that used to pass through the store; decoding must now fail before worker logic runs.

Good tests for a store boundary look slightly uncomfortable.

They construct rows that should never exist. That is the point. Production
systems eventually see states that "should never happen." A worker may crash
between updates. A migration may backfill one column but not another. An
operator may repair a status and forget the matching timestamp. A previous
version may have accepted a value the current code now rejects.

The test suite should teach the reader that a malformed row is not a surprise
inside the worker. It is expected evidence at the boundary, and the boundary has
a typed answer.

> ### 🎓 The Professor's Corner
>
> **Negative Testing: The Fire Drill**
>
> It’s not enough to know how your code works when everything is perfect! You have to know how it fails when someone (maybe you!) makes a mistake. 
> 
> A **Negative Test** is like a "Fire Drill." You intentionally create a "bad" database row—like a success row without a result—just to see if your code correctly "points at it and says No." It builds confidence that your system can handle the "Bad Days" as well as the "Good Days."

## Observability Strategy

Observe the store boundary where rows become domain values:

- Emit structured `tracing` fields for SQL file or query name, row id, decoded state, lease owner, trace id, and conversion result.
- Record an operation event when row conversion rejects malformed attempts, payloads, results, leases, timelines, or impossible terminal states.
- The runbook query should show whether a failure came from SQL transition rules or domain decoding before worker logic acted.

Store observability should help an operator answer a concrete question:

```text
Did the worker fail because the database refused a transition,
or because the row could not become valid domain state?
```

Those are different incidents. A refused transition often means another worker
won the race, a lease expired, or the job reached a terminal state first. A
decode failure often means corrupted durable state, migration drift, incompatible
schema version, or unsafe manual repair.

Logs alone are not enough. The row id, trace id, query name, conversion error,
and event timeline must line up so an operator can reconstruct what happened
without guessing.

## Security and Safety Considerations

The Postgres store must fail closed on malformed state:

- Treat every database row, JSON payload, lease column, result object, and status value as untrusted until decoded into domain types.
- authorization, sandboxing, and approval rows should be joined or checked explicitly rather than assumed from adjacent job state.
- Redact raw payload and result bodies from store errors while preserving row id, decode failure, status, lease, and trace evidence.

Database trust is a security boundary too.

An agent job row can contain user content, model output, tool arguments,
approval decisions, tenant identifiers, and side-effect receipts. If the store
logs raw payloads during a decode failure, the error path may leak the exact data
the safety system was supposed to protect. If the store assumes approval because
a nearby job status says `approved`, a corrupted row can bypass the control
surface.

The safe pattern is narrow: decode only the fields needed, validate them into
domain types, keep sensitive bodies out of logs, and require explicit evidence
for permissions, approvals, and receipts.

## Operational Checklist

Use this checklist before relying on the store boundary between SQL rows and domain values in production:

- **State:** Database rows become domain jobs only after status, attempts, payload,
  lease, and result invariants are validated.
- **Boundary:** Db row structs do not leak into worker, policy, provider, or tool logic.
- **Failure:** Invalid database state fails close with a decode error instead of
  becoming undefined worker behavior.
- **Observability:** Decode failures include safe row identity, status, trace id, and
  reason without logging raw secrets.
- **Safety:** The store preserves redaction, tenant boundaries, approval evidence, and
  receipt links during conversion.

Use the checklist during code review, not only after an incident.

The fastest way to weaken this chapter's design is to add one shortcut: pass a
`DbAgentJobRow` into a worker helper, let a raw `String` status drive a branch,
or log a full JSON payload when conversion fails. Each shortcut feels local.
Together they dissolve the boundary.

Review the store as the doorway into the domain. Anything that crosses that
doorway should already have a name, a type, and a reason to be trusted.

## Exercises

1. Write a negative test where a row says succeeded but has no result or still has a
   lease and must not become a valid domain job. Explain which idempotency key, receipt,
   or state transition prevents duplicate work.
2. Sketch the Postgres evidence: a DbScheduledJobRow fixture with valid and invalid
   status, attempts, payload, lease, and result values.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   `TryFrom<DbScheduledJobRow> for ScheduledJob<State>` and `JobDecodeError`
   variants. Then name the runbook question that proves it works.

Do the exercises with production repair in mind.

Imagine you are debugging a system at 2 a.m. The question is not whether the
happy path can decode. The question is whether the system gives you a clean,
typed, inspectable reason when durable state is wrong.

## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which types live at the database boundary and which live in the domain?
- Explain: Why should a malformed row not become a best-effort `AgentJob`?
- Apply: Decode a row with invalid attempts or status and decide where it fails.
- Evidence: Name the row type, `TryFrom` conversion, typed error, constraint, and regression test.

## Summary

The in-memory store proves the algorithm. The Postgres store proves the production boundary. Both implement the same transition model, but the database boundary must validate real rows.

- **Invariant:** raw database state does not become domain state until conversion proves it is coherent.
- **Evidence:** row conversion tests reject invalid status, attempts, lease, payload, result, and failure combinations.
- **Carry forward:** the store is a validator and transition engine, not only a query wrapper.

## Changed Understanding

- **Before this chapter:** the database looked like an implementation detail behind the worker.
- **After this chapter:** the store is a contract boundary where raw rows become validated domain state and atomic transitions.
- **Keep:** validate every database row before it becomes domain state or drives a transition.

## Further Reading & Credible References

- **[Martin Fowler: Patterns of Enterprise Application Architecture (Data Mapper)](https://martinfowler.com/eaaCatalog/dataMapper.html)** (2002). The foundational pattern for the "conversion layer" described in this chapter. It explains how to move data between objects and a database while keeping them independent.
- **[Eric Evans: Domain-Driven Design (The Repository Pattern)](https://www.domainlanguage.com/ddd/)** (2003). Chapter 6 formalizes the `AgentJobStore` trait as a Repository—a mechanism that encapsulates storage and retrieval to emulate an in-memory collection of jobs.
- **[SQLx Documentation: Compiling with Type Safety](https://docs.rs/sqlx/latest/sqlx/index.html#type-safety)**. The practical reference for the library used in this chapter to execute SQL and decode rows into Rust structs.
- **[Postgres Connection Pooling: PgBouncer vs. PgCat](https://github.com/postgresml/pgcat)**. An industry review of connection multiplexing, explaining the "Layered Pooling" strategy required when scaling the worker pools introduced in this chapter.
