# 17. Testing Production Agents

## What You Will Learn

This chapter teaches you to:

- explain what must be tested before an agent receives production traffic;
- inspect unit tests, SQL tests, boundary tests, behavior evals, live smoke tests, and regression cases;
- verify that tests cover both deterministic software and probabilistic model behavior.

The production evidence is a test matrix tied to Rust types, Postgres
transitions, Rig integration, policy gates, evaluations, and incident
regressions.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** risky actions have deterministic approval gates.
- **Adds:** tests for domain rules, persistence, workers, SQL, providers, and behavior.
- **Prepares:** deployment and operations that keep tested invariants alive.

## Production Failure

A prompt change looks good in a manual demo and ships to production.

The next morning, the agent starts approving malformed tool requests because
the new output shape was never tested against old fixtures.

**What breaks:** a demo replaced repeatable behavior evidence.

The team saw one good answer and treated it as proof. But a production agent is
not only a prompt. It is a typed state machine, a database workflow, a provider
boundary, a tool executor, and a behavior surface. A manual demo usually
exercises only the happy path.

**False fix:** ask reviewers to try a few more examples before each release.

More manual examples are useful for exploration, but they are not a release
gate. They do not reliably cover old prompt fixtures, malformed model output,
schema drift, retry behavior, unsafe tools, cross-tenant access, or recovery
after a worker crash.

**Design response:** map each reliability claim to a deterministic test, SQL
test, provider boundary check, behavior evaluation, or failure drill. The
question is not "Did the demo look good?" The question is "Which repeatable
evidence would fail if this invariant broke?"

## Motivation

In production, a passing demo does not prove an agent is reliable. The model may answer differently tomorrow, the provider may change shape, and a retry path may duplicate a side effect.

Without layered tests, teams only discover agent failures under live traffic. This chapter separates deterministic tests, SQL tests, provider contract tests, behavior evaluations, and failure drills.

## Plain Version

Read this as the simple version:

**Simple rule:** Test the invariant, not just the happy demo.

An invariant is a rule the production system must keep true. A job with an
expired lease can be recovered. A tool call without approval cannot execute. A
duplicate idempotency key cannot create duplicate side effects. These are the
rules you test.

**Why it matters:** Reliable agents need tests for typed boundaries, storage
transitions, worker ownership, tool safety, and behavior regressions. The model
may change its wording. The system rules must still hold.

**What to watch:** Watch for negative tests. Good production tests do not only
prove that the right input works. They prove that bad model output, stale
leases, duplicate side effects, cross-tenant requests, and unsafe approvals are
rejected.

## What You Already Know

Start with these anchors:

- The system has deterministic contracts around state, SQL, idempotency, leases, and policy.
- The model adds probabilistic behavior.
- Production confidence needs both kinds of evidence.

This chapter adds: the testing map. You will connect unit tests, SQL tests,
boundary tests, behavior evaluations, live smoke tests, and regressions to the
invariants they prove.

## Focus Cue

Keep three things in view:

- **State:** test evidence for types, SQL, boundaries, providers, behavior, simulations, and failure drills.
- **Move:** a reliability claim becomes acceptable only after the matching test or validation lane proves it.
- **Proof:** Unit, SQL, feature, integration, live-provider, evaluation, simulation, and failure-drill checks map to explicit risks.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

**Artifact:** a test matrix with unit, SQL, integration, behavior, regression,
and optional live-provider checks.

The matrix is not paperwork. It is the map from production risk to evidence. If
the risk is "bad JSON becomes a tool call", the evidence is a parser and
validation test. If the risk is "two workers own the same job", the evidence is
a lease ownership test. If the risk is "a prompt change silently worsens KYC
classification", the evidence is a versioned behavior evaluation.

**Why it matters:** agent behavior changes need CI-style evidence, not manual
confidence from a good demo.

**Done when:** prompt, model, tool, policy, and worker changes have tests that
catch regressions before release.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** unit tests, SQL tests, behavior evaluation tests, optional live provider smoke, and readiness gate.
- **State transition:** prove types, persistence, workers, provider boundaries, and behavior before release.
- **Evidence path:** a change has targeted tests plus a full readiness command.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which test would fail if this production invariant broke?
- **Evidence to inspect:** unit test, row-conversion test, SQL test, integration path, behavior evaluation, and readiness gate.
- **Escalate if:** a release relies on manual demo confidence instead of repeatable test evidence.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** a behavior, schema, worker, or provider change is proposed.

The change may look small. A prompt changes. A SQL query changes. A retry rule
changes. A provider adapter starts reading a new field. In an agent system,
small changes can cross trust boundaries.

**Action:** select tests that cover the changed invariant and boundary. Domain
types need unit tests. SQL transitions need SQL or row-conversion tests. Worker
ownership needs lifecycle tests. Provider adapters need contract fixtures.
Prompt behavior needs evaluations.

**Persistence:** persist evaluation receipts and CI/readiness results when the
evidence matters for release review. A team should be able to explain which
prompt version, model version, dataset version, and evaluator version supported
the decision.

**Check:** verify the change fails before release if the invariant breaks. A
test suite is useful only when it can stop a bad change before users see it.


## Acceptance Gate

Do not move on until this minimum evidence exists:

**Minimum evidence:** the changed invariant has repeatable test or evaluation
evidence.

**Validation path:** run targeted unit, SQL, integration, behavior, and
readiness checks as appropriate. You do not need every possible test for every
small edit, but you do need the test lane that matches the changed risk.

**Stop if:** release confidence depends on a manual demo or unversioned prompt
trial. A demo can motivate confidence. It cannot replace evidence.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, a passing demo does not prove an agent is reliable
rule: Test the invariant, not just the happy demo
tiny example: test evidence for types, SQL, boundaries, providers, behavior, simulations, and failure drills
artifact: a test matrix with unit, SQL, integration, behavior, regression, and optional live-provider checks
proof: the changed invariant has repeatable test or evaluation evidence
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Test the system in rings.

```text
pure domain tests
store contract tests
worker state-transition tests
provider boundary checks
SQL contract checks
failure injection
operator query checks
live provider smoke tests
```

Each ring answers a different question. Pure domain tests ask whether typed
rules are correct. Store contract tests ask whether database rows decode into
valid domain state. Worker tests ask whether leases, retries, cancellation, and
dead letters move through legal transitions. Provider boundary checks ask
whether real provider-shaped data still parses and validates. Behavior
evaluations ask whether the agent still performs the task well enough.

The inner rings should be fast and deterministic. The outer rings may be slower
because they prove integration with Postgres, provider APIs, or deployment
surfaces. Do not invert that order. A flaky live model test should not be the
only signal that your retry state machine works.

The useful habit is to name the risk first, then choose the ring. If the risk is
a broken Rust invariant, use a unit test. If the risk is provider schema drift,
use a provider fixture or live smoke test. If the risk is lower answer quality,
use an evaluation dataset.

## Tiny Example

Suppose the provider times out once and then succeeds.

A weak test asserts only that the final status is `succeeded`. A production test
also checks the path:

```text
agent_failed
retry_scheduled
job_picked
agent_succeeded
job_succeeded
```

The timeline matters because operators debug paths, not only terminal states.

The same principle applies to tests. A test that checks only the final answer
can miss the production bug. A better test checks the state transitions and
events that would let an operator understand the answer later.

Read the tiny case as:

```text
setup: the provider times out once and succeeds on retry
transition: tests prove the path, not only the final state
evidence: retry event, state transition, deterministic test, smoke test, and behavior eval agree
invariant: production confidence comes from boundary-specific evidence
```

## What The Companion Tests Cover

The executable crate tests are not random coverage. They are executable claims
about the production architecture.

They cover:

```text
newtype validation
retry backoff
idempotent enqueue
lease extension ownership
cancellation
queue metrics
transient retry
permanent dead-letter
expired lease recovery
SQL contract shape
typestate builder construction
model-output parsing and typed tool validation
version metadata validation
operator runbook SQL shape
```

Run the default suite:

```bash
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml
```

Run the Postgres-store compile and unit tests:

```bash
cargo test \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features postgres-store
```

Run the Rig boundary test lane:

```bash
cargo test \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features rig-agent
```

Read each test as a claim about production behavior. For example,
`extend_lease_only_works_for_the_worker_that_owns_the_job` is not just a unit
test. It is the proof that two workers cannot both extend authority over the
same job.

The test name should teach the rule. If a test name sounds like
`test_worker_1`, it hides the invariant. If it says
`completion_requires_the_worker_that_owns_the_lease`, it documents the
production rule in executable form.

## Live Tests

Live provider tests should be explicit, not hidden inside the default suite.

Use them for small compatibility questions:

```text
auth configuration
model name validity
provider schema drift
latency smoke
basic response sanity
```

Keep live tests small and named. When they fail, the failure should say
"provider compatibility changed" or "configuration is broken," not "the whole
state machine may be wrong."

Do not use live provider tests as the only proof of your state machine. They are
too slow, too flaky, and too expensive for that job.

This separation keeps failure diagnosis clean. If a live DeepSeek smoke test
fails, the likely problem is credentials, provider availability, model name
validity, latency, or response shape. It should not cast doubt on whether
Postgres lease recovery works. That invariant belongs in deterministic tests.

## Simulation And Chaos Testing

Simulation and chaos testing answer a different question from unit tests:

```text
What happens when realistic failure arrives in an inconvenient order?
```

Start with simulation. A simulation is a controlled rehearsal in a local or
staging environment. You choose the sequence, run it repeatedly, and inspect
the durable evidence afterward.

For an agent job system, useful simulations include:

```text
provider timeout followed by retry
worker crash after lease acquisition
worker crash after model output but before side effect
database connection loss during readiness check
duplicate webhook during a slow tool call
approval request expiring while the worker waits
restore from backup before workers resume
```

Chaos testing is the next step. A chaos experiment deliberately injects failure
into a running system to test whether the system still preserves its
invariants. It should not be random destruction. A serious chaos experiment has
the same shape as any production change:

```text
hypothesis:
  expired leases recover without duplicate side effects

blast radius:
  one staging worker, one job kind, one tenant

injection:
  terminate the worker after it claims a job

evidence:
  expired lease query, job event timeline, retry/dead-letter event,
  side-effect receipt count, operation events

rollback:
  pause the job kind, restart workers, inspect queue health
```

The safest path is:

```text
unit test -> deterministic simulation -> staging chaos experiment ->
limited production game day
```

Do not begin with production chaos. Do not inject failure without an owner,
rollback path, and observation plan. The point is not to prove that the system
can be damaged. The point is to build confidence that the system preserves
durability, ownership, idempotency, and evidence when damage occurs.

The companion crate makes this idea executable. A failure drill has a typed
plan: scenario, hypothesis, blast radius, injection, rollback action, and
evidence requirements.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/failure_drill.rs:failure_drill_plan}}
```

One deterministic drill injects a transient provider failure, expects a retry,
then expects the next attempt to succeed. This is not a mock of production
behavior. It is a simulation of the state transition the production system must
preserve:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/failure_drill.rs:provider_timeout_drill}}
```

Another drill simulates a worker disappearing after it claims a lease. The
evidence must show expired-lease recovery before success:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/failure_drill.rs:worker_crash_drill}}
```

Executable drills are useful, but production drills also need durable evidence.
If a team runs a staging game day or a limited production drill, the result
should not live only in chat, a CI log, or someone's memory. Record the drill as
state:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql:failure_drill_runs}}
```

The table uses plain fields on purpose. A tired operator should see the
hypothesis, blast radius, injected failure, rollback action, owner, required
evidence, observed evidence count, result, and signoff in one place. A passed
drill must have at least as much observed evidence as required evidence. A
failed or aborted drill is still useful because it records what broke and who
accepted the result.

The runbook query is the small operational view:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/failure_drill_status.sql}}
```

Use it to ask:

```text
Which failure drills are planned, running, failed, or aborted?
Which drill touched staging or production?
Did the observed evidence satisfy the required evidence?
Who owns the rollback action and terminal signoff?
```

The companion readiness gate already has the ingredients for these
experiments: deterministic worker tests, SQL runbook queries, the Postgres
worker demo, API runtime smoke, the optional live Postgres gate, and an
optional DeepSeek smoke gate. The live DeepSeek gate is not the main oracle for
reliability; it is a contract check that the real Rig provider boundary still
produces parseable typed output when credentials are intentionally supplied.
The next step in a real product would be to run those controls under deliberate
worker, provider, database, and approval failures.

## Formal Definition

For this chapter, the precise definition is:

```text
Production testing is the evidence system that proves state transitions, boundaries, behavior, failure handling, and provider compatibility before release.
```

In the book's system model:

- **State:** test evidence for types, SQL, boundaries, providers, behavior, simulations, and failure drills.
- **Actor:** developers, CI, reviewers, and release gates create and inspect evidence before production release.
- **Transition:** a reliability claim becomes acceptable only after the matching test or validation lane proves it.
- **Evidence:** Unit, SQL, feature, integration, live-provider, evaluation, simulation, and failure-drill checks map to explicit risks.
- **Invariant:** production behavior is released from evidence, not from confidence in a demo.

## What Can Fail

**Design smell:** tests mock away provider and persistence contracts. The suite
looks fast and green, but it no longer proves the boundaries that break in
production.

**Production symptom:** the unit suite passes while real malformed output or
SQL behavior breaks production. This is common when tests validate the service
layer but never exercise row conversion, SQL constraints, provider-shaped
fixtures, or denied authorization paths.

**Corrective invariant:** tests cover pure domain logic, SQL semantics,
provider fixtures, behavior evals, and live smoke paths. Each layer keeps its
own promise.

**Evidence to inspect:** test names and readiness commands map each boundary to
evidence. A release reviewer should be able to ask, "What proves this?" and get
a concrete test, evaluation receipt, or drill result.


## Production Contract

The testing contract is a set of matched responsibilities.

Deterministic tests prove the state machine. SQL tests prove the database
contract. Feature checks prove optional boundaries compile. Live tests prove
external API compatibility. Evals prove behavior quality. Simulation proves
expected failure sequences. Chaos experiments prove the operating surface under
controlled disruption. Typed drill reports connect hypotheses to event
evidence.

No single ring replaces the others. Together they prevent a demo from being
mistaken for a dependable service.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Tests mock away provider and persistence contracts. | A passing happy-path test says little about provider drift, malformed outputs, retries, or unsafe tools. |
| Safer version | Tests cover pure domain logic, SQL semantics, provider fixtures, behavior evals, and live smoke paths. | Tests are selected by boundary: domain, SQL, worker, API, provider adapter, evaluation, and failure drill. |
| Production version | Test names and readiness commands map each boundary to evidence. | A release can be blocked by evidence gaps instead of by vague distrust of the agent. |

Use the naive row when tests only show the demo works. Use the safer row to test the boundary. Use the production row before behavior changes are released.

## Testing Strategy

Test the test strategy itself by mapping checks to risks:

**Unit or type test:** prove Rust evaluation, release-gate, and failure-drill
objects reject missing dataset, rubric, owner, or expected evidence. This keeps
test evidence typed instead of becoming another bag of strings.

**Persistence or boundary test:** prove Postgres fixtures and SQL queries cover
the state, failure, runbook, and evaluation evidence named by the chapter. The
database is part of the reliability contract, so the database needs direct
tests.

**Regression test:** add a behavior change without an evaluation receipt or
failure fixture and verify the release gate blocks promotion. This turns a
previous failure into a permanent guardrail.

## Observability Strategy

Observe test evidence as release evidence:

Emit structured `tracing` fields for test layer, dataset version, evaluator
version, release candidate, job kind, trace id, and pass or fail decision. These
fields connect validation to the release that used it.

Record an operation event when a behavior evaluation, failure drill,
live-provider smoke, SQL check, or release gate blocks promotion. A blocked
release is not noise. It is evidence that the control system did its job.

The runbook query should show which validation artifact proves or disproves
readiness for a prompt, model, tool, or policy change. During an incident, this
lets the team ask whether the failure escaped existing tests or whether the
release ignored failing evidence.

## Security and Safety Considerations

Tests should include abuse and boundary failures, not only happy paths:

Treat fixtures, golden outputs, simulated provider responses, and grader output
as untrusted until schema and semantic validation pass. A test fixture can
accidentally train the system to trust shapes that production must reject.

authorization, sandboxing, and approval tests should cover denied tools,
missing approval, cross-tenant access, and unsafe side effects. These are not
edge cases. They are the safety boundary.

Redact dataset secrets and private examples while preserving case id, expected
policy, observed outcome, and evaluation receipt. The team needs enough
evidence to debug and audit without turning test data into a privacy leak.

## Operational Checklist

Use this checklist before relying on testing agents as systems, not prompts in production:

- **State:** Tests cover domain constructors, SQL transitions, worker lifecycle,
  provider boundary, evaluation, and runbook evidence.
- **Boundary:** Model/provider fixtures preserve real contract shapes while domain tests
  avoid trusting raw JSON.
- **Failure:** A regression test catches duplicate side effects, invalid row conversion,
  bad retry, and unsafe model output before release.
- **Observability:** Test output names the evidence surface that would prove the
  behavior in production.
- **Safety:** Tests include authorization denial, sandbox denial, approval requirement,
  redaction, and replay safety cases.

## Exercises

1. Write a negative test where a tool output skips validation and is persisted without
   idempotency or policy evidence. Explain which idempotency key, receipt, or state
   transition prevents duplicate work.
2. Sketch the Postgres evidence: a fixture database state and query assertion for one
   lifecycle transition and one runbook query.
3. Define or refine the Rust type, enum, constructor, or typestate that represents unit,
   integration, property, and contract test helpers around typed domain values. Then
   name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which test layers cover deterministic software, and which cover model behavior?
- Explain: Why can a live provider smoke test not replace SQL and domain tests?
- Apply: Test a provider timeout followed by success.
- Evidence: Name the unit test, SQL test, integration smoke, behavior eval, and regression case.

## Summary

Test deterministic behavior by default and live providers deliberately.

The state machine should stay correct even when model behavior changes.
**Invariant:** tests cover typed domain rules, SQL transitions, worker lifecycle,
provider contracts, behavior evaluation, security controls, and runbook
evidence.

**Evidence:** unit tests, row-conversion tests, integration tests, contract
fixtures, eval receipts, and failure drills block regressions.

**Carry forward:** evaluation is the CI/CD pipeline for agent behavior.

## Changed Understanding

- **Before this chapter:** agent testing looked like checking whether one prompt gives a good answer.
- **After this chapter:** production agent testing combines unit tests, integration tests, contract tests, evals, and failure drills.
- **Keep:** require every important behavior to have a unit, persistence, regression, or evaluation proof.

## Further Reading & Credible References

- **[Principles of Chaos Engineering](https://principlesofchaos.org/)**. The foundational industry manifesto (pioneered by Netflix) for building resilience by deliberately injecting failure to test system invariants—the core logic behind the "Failure Drills" in this chapter.
- **[OpenAI: Contextual Evals and the Evals API](https://github.com/openai/evals)**. The primary reference for moving from manual prompt testing to repeatable "Behavior Evaluations" using task-specific datasets and "LLM-as-a-Judge" grading patterns.
- **[The proptest book: Property-Based Testing in Rust](https://proptest-rs.github.io/proptest/intro.html)**. Explains how to use random input generation to find deep boundary bugs (e.g., in lease expiry or retry logic) that manual unit tests often miss.
- **[AgentBench: A Comprehensive Benchmark for LLMs as Agents](https://arxiv.org/abs/2308.03688)** (ICLR 2024). The academic foundation for evaluating agents across diverse environments (OS, Database, Web), providing the rubric for the "middle ring" of testing discussed in this chapter.
- **[FoundationDB: Testing the Untestable (Simulation)](https://www.youtube.com/watch?v=4fFDFbi3toc)**. Re-visiting this seminal industry talk to ground the chapter's "Deterministic Local Run" in the rigorous simulation practices of world-class distributed databases.
