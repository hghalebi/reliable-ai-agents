# 7. Running The System Locally

## What You Will Learn

This chapter teaches you to:

- explain what a local run can prove before external infrastructure is added;
- inspect the command, logs, rows, events, and tests that exercise the same state machine as production;
- verify that local execution is a reliability rehearsal, not only a demo.

The production evidence is a deterministic local path that runs the API,
worker, SQL schema, Rig boundary, tests, and diagnostics against the same
contracts.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the Rig boundary is isolated from core reliability.
- **Adds:** a local proof path for the state machine and reader setup.
- **Prepares:** production hardening controls for duplicate, slow, risky, and failed work.

## Production Failure

The demo works only when the developer runs one command in one terminal, with
one happy-path prompt and no durable state.

When the system later moves to Postgres and a worker process, the first real
failure is impossible to reproduce locally.

- **What breaks:** local success did not exercise the production shape.
- **False fix:** skip local reliability checks and test only against live
  services.
- **Design response:** make the local run prove the same lifecycle, typed
  boundaries, job states, events, and diagnostics that production will depend
  on.

## Motivation

In production, confidence should start before the first external dependency. If the local state machine does not work deterministically, a live model or database will only make the failure harder to diagnose.

Without a runnable local path, the book becomes theory instead of engineering practice. This chapter shows how to prove the core lifecycle first, then add Postgres and DeepSeek as stronger integration evidence.

## Plain Version

Read this as the simple version:

- **Simple rule:** The local run should exercise the same reliability shape as production, just at smaller scale.
- **Why it matters:** A demo that skips Postgres, workers, events, and health checks teaches the wrong system.
- **What to watch:** Watch startup, migrations, worker execution, health endpoints, and logs for the same evidence operators will need later.

## What You Already Know

Start with these anchors:

- The book now has a durable state machine, worker loop, and Rig boundary.
- Local execution is useful only when it exercises the same contracts as production.
- A demo without evidence does not prove reliability.

This chapter adds: a local proof path. You will run the API, worker, SQL,
tests, and diagnostics as a small rehearsal of the production system.

## Focus Cue

Keep three things in view:

- **State:** a runnable local job lifecycle that can execute without live infrastructure while preserving production semantics.
- **Move:** the same job state machine is exercised through local, Rig-backed, in-memory, and Postgres-backed adapters.
- **Proof:** The local agent, Rig-backed agent, in-memory store, and Postgres store preserve the same job semantics.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a local run path that exercises the same state machine as production without extra infrastructure.
- **Why it matters:** a beginner path should prove the architecture before adding a platform or workflow engine.
- **Done when:** the reader can enqueue work, run the worker, inspect events, and rerun tests on one machine.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** local binaries, deterministic in-memory store tests, and optional DeepSeek smoke script.
- **State transition:** run the same state machine locally before adding production infrastructure.
- **Evidence path:** the reader can enqueue, run, inspect events, and test without hiding reliability logic.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Can the local path prove the same state transition the production path depends on?
- **Evidence to inspect:** local command output, job row, event timeline, deterministic test, and optional provider smoke result.
- **Escalate if:** the local demo exercises a different path from the production worker, store, or provider boundary.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** the reader starts the local companion system.
2. **Action:** run the deterministic path and optional provider smoke path.
3. **Persistence:** inspect rows, events, logs, and test output.
4. **Check:** verify local behavior exercises the same state machine as production.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** the local path exercises the same state machine the production path depends on.
- **Validation path:** run the local demo, deterministic tests, and optional DeepSeek smoke when credentials are available.
- **Stop if:** local success bypasses leases, events, retries, or typed provider boundaries.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, confidence should start before the first external dependency
rule: The local run should exercise the same reliability shape as production, just at smaller scale
tiny example: a runnable local job lifecycle that can execute without live infrastructure while preserving production semantics
artifact: a local run path that exercises the same state machine as production without extra infrastructure
proof: the local path exercises the same state machine the production path depends on
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Treat the local run as a laboratory version of production:

```text
in-memory store    -> stands in for Postgres
local agent runner -> stands in for Rig and DeepSeek
worker loop        -> same transition machine production will use
event list         -> stands in for the durable event ledger
```

The substitute parts are intentionally boring. They let you test the shape of
the system before any network, secret, database, or provider behavior enters
the picture.

This is important because local does not mean fake.

A fake local demo uses a different path from production. It calls a function,
prints text, and skips the state that will matter later. A useful local run does
the opposite. It keeps the same state transitions and replaces only the expensive
or unstable edges.

In this book, the in-memory store is not a new architecture. It is a smaller
implementation of the same contract. The deterministic agent runner is not a
model substitute for product judgment. It is a way to prove that the worker can
handle a stable agent result before the reader adds provider uncertainty.

That distinction keeps debugging simple. If the deterministic local run fails,
the bug is probably in the core state machine. If the deterministic run passes
but the DeepSeek path fails, the bug is probably at the provider boundary,
configuration boundary, or output validation boundary.

This is a form of **Unit Testing for AI Systems**. We "stub" the probabilistic, expensive parts (the model) to test the deterministic, reliable parts (the worker). It ensures our "plumbing" is solid before we turn on the water.

## Tiny Example

The smallest useful run is one job that moves from enqueue to success and
leaves an event trail. That is the example the default command exercises.

Do not treat this as a toy example.

One job is enough to prove the shape of the system if the job crosses the right
boundaries. It must be enqueued with identity. It must be claimed by a worker. It
must call the agent boundary. It must store a result. It must leave events that
explain what happened.

If the first local run skips those steps, adding more jobs will not make the
architecture reliable. Scale only multiplies the shape you already have.

Read the tiny case as:

```text
setup: one local job can move from enqueue to terminal state
transition: the same API, worker, SQL, and Rust contracts run without external infrastructure
evidence: tests, logs, rows, and events match the production state machine
invariant: local proof is useful only when it exercises production-shaped boundaries
```

> ### 🎓 The Professor's Corner
>
> **Rehearsal: Practicing for the Performance**
>
> Think of your local run as a **Rehearsal**. Just as a musician practices a small piece of music in their room before the concert, we "rehearse" our code locally. 
> 
> We don't want to "mess up" in front of the production audience (the real users) or burn through our budget (the model costs) while we're still learning the notes. Rehearsing with a "Boring" local setup builds the muscle memory needed to handle the "Exciting" production deployment later!

## Run

From the repository root:

```bash
cargo run \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --bin postgres-rig-agent-jobs
```

Expected shape:

```text
job status: succeeded
events:
  job_enqueued
  job_picked
  agent_started
  agent_succeeded
  job_succeeded
```

This output proves the minimum useful lifecycle:

```text
durable work exists
one worker picks it
the agent step runs
the result is stored
the event timeline explains the run
```

If one of those facts is missing, the system is not yet ready for real
infrastructure.

Read the output like an operator, not like a tutorial screenshot.

`job_enqueued` proves the system accepted durable work. `job_picked` proves a
worker claimed it. `agent_started` proves the model boundary was reached.
`agent_succeeded` proves the result crossed back as a typed outcome.
`job_succeeded` proves the worker wrote a terminal transition.

That sequence is the smallest useful evidence chain. It is not enough for full
production readiness, but it is enough to catch a surprising number of design
mistakes before the reader adds Postgres, a live model, dashboards, or deploy
automation.

## Run With DeepSeek

When `DEEPSEEK_API_KEY` is present, run the real Rig-backed binary:

```bash
cargo run \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features rig-agent \
  --bin deepseek-agent-demo \
  -- "Analyze a failed deployment and propose one safe next step"
```

This uses the same job loop, store, events, and typed result contract. Only the
agent runner changes.

That distinction matters. A provider integration should replace one boundary,
not rewrite the worker, retry policy, event model, or job states. If changing
providers forces changes across the queue, the provider boundary is leaking.

The companion repository also packages this provider check as a smoke script:

```bash
RUN_LIVE_DEEPSEEK=1 DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" ./scripts/check-book-readiness.sh
```

The local binaries are intentionally thin, but they still return typed
`thiserror` errors. That keeps runtime configuration failures, domain
validation failures, store failures, worker failures, and provider failures
visible as named cases instead of collapsing them into an untyped application
error.

```bash
DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" scripts/smoke-deepseek-agent.sh
```

The script verifies that the real Rig-backed worker succeeds, that model
output becomes a typed result, and that the event timeline records agent start
and success evidence. Keep this path optional. The local deterministic runner
is still the default teaching path because most reliability checks should not
depend on network access or provider credentials.

Optional does not mean unimportant.

The DeepSeek path proves a different thing from the deterministic path. The
deterministic path proves the reliability shell. The DeepSeek path proves that a
real provider can enter through the same shell without changing the worker
contract.

Keep those questions separate. When they are mixed together, every failure
becomes ambiguous. Was the retry wrong? Was the API key missing? Did the model
return malformed JSON? Did the worker lose its lease? Separate local lanes let
you answer one question at a time.

This approach is known as **Staged Evaluation**. You don't jump straight to production; you move through increasing levels of fidelity. First deterministic, then small provider calls, then full-scale traffic. It's the only way to avoid burning your budget on unverified code.

## Test

```bash
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml
```

The tests cover:

```text
successful execution
idempotent duplicate suppression
retry after temporary failure
permanent failure without retry storms
dead job after attempts are exhausted
expired lease recovery
lease extension by the owning worker
cancellation
queue metrics
SQL contract checks for SKIP LOCKED and lease recovery
```

Read those tests as executable claims. They are not only checking functions;
they are checking the promises a production operator will depend on.

For example, `expired lease recovery` is not just a test case. It is a statement
about what happens when a worker dies or pauses too long. `idempotent duplicate
suppression` is not just a convenience. It is the reason a retry does not create
two user-visible actions. `permanent failure without retry storms` is not just
error handling. It is how the system protects operators from noisy, useless
loops.

Good tests teach the reader what must remain true. When a future change breaks
one of these tests, the failure should point to a production invariant, not only
to a line of code.

## Check The Optional Production Features

The real Rig runner is tested separately:

```bash
cargo test \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features rig-agent
```

The real Postgres store is also tested separately:

```bash
cargo test \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features postgres-store
```

This split is deliberate. Most readers should be able to run the core book
without secrets or infrastructure. Production engineers can then turn on the
exact boundary they want to inspect.

Think of these feature lanes as zoom levels.

The default lane proves the core lifecycle. The Rig lane adds a live model
boundary. The Postgres lane adds the real coordination layer. The temporary local
Postgres smoke adds migration, SQL, API, worker, and runbook evidence.

Each lane should add one kind of uncertainty. That keeps failures readable. If
the Postgres lane fails after the default lane passes, inspect SQL, migrations,
connection configuration, and row conversion. If the Rig lane fails after the
default lane passes, inspect provider configuration, output parsing, and retry
classification.

If local Postgres binaries are installed, run the temporary database smoke:

```bash
RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh
```

That path creates a fresh local Postgres cluster, applies the schemas, runs the
Postgres-backed worker, smokes the API server, executes runbook SQL, verifies
audited pause/resume control events, and removes the cluster. It is stronger
than a pure unit test and lighter than requiring Docker.

This is what we call an **Ephemeral Environment**. It proves the **Migration Schema** and **SQL Predicates** are correct without the baggage of a long-lived development database. It's the best way to prevent "Works on my machine" from becoming "Broke in the Cloud."

## Formal Definition

For this chapter, the precise definition is:

```text
A local run is a deterministic proof that the same state machine works before external infrastructure or live provider behavior is required.
```

In the book's system model:

- **State:** a runnable local job lifecycle that can execute without live infrastructure while preserving production semantics.
- **Actor:** the developer or readiness gate runs deterministic binaries and optional provider-backed variants.
- **Transition:** the same job state machine is exercised through local, Rig-backed, in-memory, and Postgres-backed adapters.
- **Evidence:** The local agent, Rig-backed agent, in-memory store, and Postgres store preserve the same job semantics.
- **Invariant:** changing adapters must not change enqueue, lease, retry, success, dead-letter, or event semantics.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Local runs prove only that one command returns text. |
| Production symptom | The system appears usable while the state machine remains untested. |
| Corrective invariant | Local execution exercises the same lifecycle as production. |
| Evidence to inspect | Deterministic tests prove enqueue, lease, retry, success, dead-letter, and recovery behavior. |


## Production Contract

The local run is acceptable only if it preserves this contract:

```text
The same worker state machine runs with a local agent, a Rig-backed agent,
an in-memory store, and a Postgres store.
```

The adapters may change. The job semantics should not.

This contract is the bridge from laptop to production.

On a laptop, the system may use one process, one deterministic runner, and one
small store. In production, it may use separate API and worker processes, a real
Postgres database, a live provider, and stronger observability. Those changes
should increase evidence and operational strength. They should not change what
it means to enqueue, claim, retry, succeed, cancel, or dead-letter a job.

If local and production disagree about those words, the local run is teaching the
wrong system.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Local runs prove only that one command returns text. | A one-command demo can pass while skipping leases, retries, event history, and recovery semantics. |
| Safer version | Local execution exercises the same lifecycle as production. | Local execution exercises the same state machine with deterministic stores before live dependencies enter. |
| Production version | Deterministic tests prove enqueue, lease, retry, success, dead-letter, and recovery behavior. | The reader can run the companion code and see enqueue, lease, success, retry, dead-letter, and recovery behavior. |

Use the naive row when local success proves only a prompt. Use the safer row to test the mechanism. Use the production row before claiming the example teaches reliability.

## Testing Strategy

Test local execution against the same semantics as production:

- **Unit or type test:** prove the Rust in-memory store and deterministic agent exercise enqueue, lease, success, retry, dead-letter, and recovery transitions.
- **Persistence or boundary test:** prove the Postgres-backed path uses the same job states and SQL transition rules as the local proof path.
- **Regression test:** run a local worker crash or provider failure fixture and verify the result matches the production retry and recovery contract.

## Observability Strategy

Observe local runs with the same fields used in production:

- Emit structured `tracing` fields for local store kind, job id, worker id, attempt, status, trace id, and deterministic agent outcome.
- Record an operation event in the local proof path for enqueue, claim, success, retry, dead-letter, and recovery transitions.
- The runbook query or local diagnostic should answer the same job-state questions before and after switching to Postgres or a live provider.

## Security and Safety Considerations

Local demos should exercise the same safety model as production:

- Treat local fixtures, CLI input, fake agent output, and seed rows as untrusted unless they pass the same constructors as production data.
- authorization, sandboxing, and approval can be no-op only when the local job has a documented no-side-effect decision.
- Redact local `.env` values and provider keys while preserving enough run output to prove the safety boundary was exercised.

## Operational Checklist

Use this checklist before relying on local proof before external infrastructure in production:

- **State:** The local run exercises the same job states, retries, leases, and receipts
  the production path relies on.
- **Boundary:** Demo inputs and environment variables are parsed into typed config
  before the worker or DeepSeek path starts.
- **Failure:** A missing API key, invalid database URL, broken migration, or failed
  model call stops with explicit evidence.
- **Observability:** Local logs, test output, and runbook queries expose job id, trace
  id, status, attempts, and result.
- **Safety:** The local demo must not bypass idempotency, approval, sandbox, or secret
  redaction just because it is local.

## Exercises

1. Write a negative test where a local run is interrupted after enqueue and rerun
   without creating a duplicate idempotency path. Explain which idempotency key,
   receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: the local database rows created by one deterministic
   run and one failed run.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   WorkerConfig, DeepSeekConfig, and LocalRunResult validation types. Then name the
   runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which production contracts can the local path prove without external infrastructure?
- Explain: Why is local not fake when it runs the same state machine?
- Apply: Before using DeepSeek, decide what deterministic tests should already pass.
- Evidence: Name the command, SQL row, event, Rust test, and diagnostic output that prove the lifecycle.

## Summary

The runnable local system proves the core mechanics before adding real infrastructure. Local does not mean casual; it means the same state machine is easier to inspect.

- **Invariant:** the deterministic local path and optional DeepSeek path exercise the same typed job lifecycle.
- **Evidence:** local test output, job rows, operation events, trace ids, and config validation show enqueue, claim, run, retry, and completion.
- **Carry forward:** prove the boring mechanics locally before trusting a live provider.

## Changed Understanding

- **Before this chapter:** local execution looked like a quick demo command.
- **After this chapter:** local execution is the first proof that the durable job, worker, database, and evidence path fit together.
- **Keep:** run the local system until one job leaves durable evidence before and after worker execution.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
