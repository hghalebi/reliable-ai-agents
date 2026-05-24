# 15. Observability And SLOs

## What You Will Learn

This chapter teaches you to:

- explain the difference between logs and real observability;
- inspect trace fields, operation events, metrics, SLOs, and runbook queries for one job lifecycle;
- verify that an operator can reconstruct what happened without guessing from process memory.

The production evidence is a traceable event timeline with job ids, run ids,
attempts, statuses, latency, cost, model version, and SLO signals.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** retry and terminal states leave evidence.
- **Adds:** logs, events, metrics, traces, SLIs, and SLOs as one evidence system.
- **Prepares:** human approval and policy gates for risky actions.

The previous chapters made the system durable. Durability gives you evidence,
but only if the evidence is connected.

An agent run can cross HTTP admission, Postgres rows, worker leases, model calls,
tool calls, retries, human approvals, and side-effect receipts. During an
incident, the operator should not have to remember how those pieces relate. The
system should make the relationship visible.

That is the point of observability in this book: not more text, but better
answers.

## Production Failure

Users report that agent jobs are "slow today."

The team has logs, but no shared trace id, no queue-age metric, no job-kind SLI,
and no query that connects one run to its retries and provider calls.

- **What breaks:** debugging becomes manual archaeology.
- **False fix:** add more text logs inside the worker.
- **Design response:** record correlated traces, metrics, operation events,
  audit events, and SLO measurements that answer the same operational question.

## Motivation

In production, logs alone are not observability. Operators need to answer whether work is flowing, why it is slow, which provider is failing, which job kind is burning budget, and whether behavior is still safe.

Without correlated state, events, metrics, traces, and SLOs, incidents become manual archaeology. This chapter designs observability as evidence, not noise.

## Plain Version

Read this as the simple version:

- **Simple rule:** Observability means you can reconstruct what happened and measure whether users received the promised behavior.
- **Why it matters:** Logs alone are not enough when jobs run across workers, retries, model calls, approvals, and side effects.
- **What to watch:** Watch trace ids, operation events, queue metrics, latency, failure labels, and SLO evidence.

## What You Already Know

Start with these anchors:

- The ledger records jobs, attempts, retries, leases, and terminal states.
- Those records are not only implementation details.
- Operators need evidence faster than they need more log text.

This chapter adds: observability and SLOs. You will connect traces, operation
events, metrics, and SQL diagnostics to the promises users and operators care
about.

## Focus Cue

Keep three things in view:

- **State:** correlated state rows, event rows, audit rows, operation rows, metrics, traces, logs, and SLO measurements.
- **Move:** each important state transition records enough correlated evidence for later reconstruction.
- **Proof:** Trace ids, structured fields, operation events, audit events, metrics, and runbook queries answer the same job question.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a trace, metric, log, audit-event, and operation-event surface for one agent run and the fleet.
- **Why it matters:** observability is the ability to reconstruct behavior, not a larger pile of logs.
- **Done when:** an operator can answer what happened to one run and whether the fleet is inside its reliability target.

The artifact is an evidence system.

One part explains a single transition. Another part explains one job timeline.
Another part explains the fleet. Another part preserves business-significant
decisions. Another part measures whether the system is meeting its promise.

If these parts do not share identifiers and meaning, they become separate piles
of data. If they agree on job id, run id, trace id, event type, status, and time
window, they become a map.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/audit.rs`, `src/slo.rs`, worker tracing fields, metrics SQL, and runbook queries.
- **State transition:** record enough telemetry to reconstruct one run and measure the fleet.
- **Evidence path:** trace ids, event rows, SLI rows, and queue metrics explain the same workflow.

Each surface answers a different kind of question.

The worker tracing fields answer what the running process observed. Operation
events answer what happened in the system. Audit events answer who or what made
a business-significant decision. Metrics answer how the fleet is behaving.
SLO measurements answer whether that behavior is still inside an agreed target.
Runbook queries connect those signals to action.

The implementation is successful only when these surfaces can be joined into one
story.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Can an operator reconstruct one run and one fleet symptom from evidence?
- **Evidence to inspect:** trace id, span id, operation events, audit events, queue metrics, SLI rows, and SLO window.
- **Escalate if:** logs contain text but no stable event, metric, trace, or SLO evidence.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a run or fleet symptom needs explanation.
2. **Action:** join trace fields, events, metrics, and SLO measurements.
3. **Persistence:** persist structured telemetry and budget evidence.
4. **Check:** verify one run and one fleet question can be answered without private memory.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** one run and one fleet symptom can be reconstructed from telemetry.
- **Validation path:** inspect trace ids, operation events, audit events, metrics, SLI rows, and SLO tests.
- **Stop if:** operators need private memory or ad hoc log search to explain behavior.

This gate is deliberately practical.

Pick one run. Can you answer how it entered the system, when it was claimed,
which model route it used, which tool calls happened, which retries occurred,
what failed, who approved or blocked risky action, and why it reached its final
state?

Then pick one fleet symptom. Can you answer whether the queue is aging, workers
are failing to claim jobs, a provider is slow, a job kind is burning budget, or
policy is blocking risky work?

If the answer is "search the logs and ask whoever was on-call," the system is
not observable enough.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, logs alone are not observability
rule: Observability means you can reconstruct what happened and measure whether users received the promised behavior
tiny example: correlated state rows, event rows, audit rows, operation rows, metrics, traces, logs, and SLO measurements
artifact: a trace, metric, log, audit-event, and operation-event surface for one agent run and the fleet
proof: one run and one fleet symptom can be reconstructed from telemetry
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Observability has three distances:

```text
close:
  one event explains one transition

middle:
  one job timeline explains one execution

far:
  metrics and SLOs explain the fleet
```

A serious system needs all three. Logs alone are too close. Dashboards alone are
too far. The job ledger connects them.

This distance model helps avoid two common mistakes.

The first mistake is staring only at logs. Logs are close to the process, so they
often show local symptoms but miss fleet shape. The second mistake is staring
only at dashboards. Dashboards show shape, but they often hide the specific job
or decision that explains the shape.

Good observability lets you move between distances: alert, metric, query, trace,
event, row, and then action.

## Three Layers

Use three observability layers:

```text
state:
  current row in agent_jobs

events:
  append-only timeline in agent_job_events

metrics:
  aggregated health for alerts and dashboards
```

State helps dashboards. Events help debugging. Metrics help alerting.

The layers should agree. If the metric says jobs are stuck but the ledger has
no old pending rows, the metric is wrong or measuring a different system. If an
event says a job succeeded but the current row is still running, the state
machine has a bug. Observability is partly about detecting those contradictions
early.

Agreement matters more than volume.

A system can emit thousands of log lines and still be unobservable if those
lines cannot be connected to durable state. A smaller system with consistent
state, event, metric, and trace identifiers is easier to operate because each
signal can confirm or challenge the others.

The goal is not to collect everything. The goal is to preserve enough evidence
to answer the next operational question.

## Tiny Example

If the oldest pending job is 30 minutes old, the metric tells you there is a
problem. It does not tell you why.

The next step is to inspect state and events:

```text
state says: many jobs pending, few running
events say: workers stopped picking after provider rate limits
runbook says: pause low-priority kind and reduce concurrency
```

The metric starts the investigation. The state and event timeline make it
actionable.

Read the tiny case as:

```text
setup: oldest pending age crosses an alert threshold
transition: the operator moves from metric to row, event timeline, trace, and owner action
evidence: metric sample, queue query, operation events, and trace id tell one story
invariant: observability means reconstructing what happened, not collecting more logs
```

## Queue Metrics

The companion SQL exposes a small health surface:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/queue_metrics.sql}}
The medical monitor analogy is perfect. In AI, we often have the **"Vibes-based Monitoring"** problem, where a developer looks at three outputs and says "The model feels slow today." SLIs turn those vibes into **Hard Data**.

Fleet health is a signal; SLO measurement is a contract. For that, the companion
implementation also exposes SLI queries such as:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql}}
```

We should also measure **Quality SLIs**. For example, "99% of triage jobs result in a valid proposal." If the "Good Event" count drops because models are failing to parse, that's an SLO breach that requires a prompt engineer, not a SRE. These SLIs should be part of our **Continuous Evaluation** pipeline.

In AI, performance is inextricably linked to cost. We should add a **Cost SLI**: for example, "99% of triage jobs cost less than $0.05." If a worker "loops" or "retries" with a larger model, causing a cost spike, our **Cost-per-Reasoning-Step** metric will catch it before it burns the budget.

The query returns a measurement row with a named SLO, a named SLI, a measurement
window, a target in basis points, a good-event count, and a total-event count.
That row is later converted into typed Rust values before it can drive a release
or paging decision.

> ### 🎓 The Professor's Corner
>
> **Observability as Consensus: The Group Agreement**
>
> Think of every worker and database as people in a meeting. Each one is shouting their "View" of the truth! 
> 
> **Observability** is how we reach **Consensus**. We look at all the shouts (logs, metrics, events) and find where they agree. If one person says "Jobs are stuck" (metric) but the teacher's gradebook says "No they aren't" (ledger), we have a gap! Our job as operators is to find the truth in the middle.

Start with these alerts:

```text
oldest_pending_age_seconds too high
dead jobs increasing
running jobs with expired leases
provider transient failures above baseline
no jobs picked while pending jobs exist
```

These alerts are useful because they point to different failure modes.

Old pending work suggests intake is outrunning execution or workers are not
claiming jobs. Dead jobs increasing suggests failures are reaching terminal
state. Expired leases suggest workers are dying or losing ownership. Provider
transient failures suggest dependency trouble. Pending work with no claims
suggests the worker loop, queue predicate, or readiness path is broken.

A good metric does not finish the diagnosis. It tells the operator where to look
next.

## Runtime Health Surfaces

The API should expose three different operational questions:

```text
/healthz  -> is this process alive?
/readyz   -> can this process reach the queue dependency it needs?
/metrics  -> what does the queue look like right now?
```

Do not collapse these into one endpoint. A process can be alive while Postgres
is unavailable. A queue can be reachable while the backlog is dangerous. These
are different facts, so they deserve different surfaces.

The companion API keeps liveness dependency-free and uses the same typed
`QueueMetrics` domain object behind readiness and metrics responses:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/api.rs:api_runtime_surfaces}}
```

The JSON response is an edge DTO. Inside the application, the queue snapshot
has already crossed into typed Rust values. This preserves the chapter's rule:
raw outside, typed inside, and operational evidence at the boundary.

## Traces

For a full production service, propagate a correlation id across:

```text
enqueue request
job row
worker event
model provider call
tool call
side-effect job
operator approval
```

The event table is the durable trace. OpenTelemetry spans are the live trace.
Use both; they answer different questions.

When they disagree, prefer the durable ledger for audit and use the live trace
to explain timing, dependencies, and latency. The ledger is the source of truth
for what the system committed.

This is the **Causality Thread**. We use the **W3C Trace Context** standard to ensure that this thread is not broken when a job moves from a worker to an external model provider and back. We should also record the **Span ID** in our failure history. It allows you to pinpoint *which specific attempt* caused the trace to fail.

> ### 🎓 The Professor's Corner
>
> **The Storyteller's Rule: Connecting the Dots**
>
> Every signal (trace, log, event) should be a sentence in a story. If the sentences don't connect, the story doesn't make sense! 
> 
> If you have a log that says "Something went wrong" but no Trace ID to link it to a job, you have a sentence without a subject. A good storyteller makes sure every clue leads back to the main character (the Job ID). This is how we reconstruct the whole journey!

This is why the book does not require a heavy observability platform on day one.

You can start with durable trace identifiers in Postgres rows and structured
logs. Later, an OpenTelemetry collector can add richer span timing and cross
service visualization. The concept does not change. The trace id is still the
thread that ties one logical operation together.

The important beginner path is to make correlation unavoidable before adding
more infrastructure.

## Trace Context

Trace context is the correlation handle that lets one logical operation stay
visible across HTTP admission, Postgres rows, worker execution, provider calls,
tool calls, and side-effect receipts.

The important distinction is:

```text
trace id:
  the whole logical operation

span id:
  one operation inside that trace
```

For example, a support triage job might have one trace id from enqueue to
terminal state, with separate spans for request admission, job claim, model
call, CRM lookup, approval wait, and email draft publication.

The companion crate keeps this compatible with W3C-style trace identifiers
without requiring an OpenTelemetry collector in the beginner path:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/audit.rs:trace_context}}
```

The domain type is `TraceContext`: it pairs a required `TraceId` with an
optional `SpanId` before an operation event can become trusted evidence.

The database stores trace ids on agent runs and operation events. The Rust row
boundary rejects malformed or all-zero trace/span ids before the values become
operator evidence. That keeps this production rule concrete:

```text
trace identifiers are operational correlation data, not arbitrary strings
```

This is also why trace context belongs in durable rows as well as live spans. A
trace backend can expire or be unavailable during an incident; the database
still needs enough correlation evidence for the operator to reconstruct the
workflow.

The companion binaries install a tracing subscriber at startup. Raw environment
variables become a typed runtime tracing configuration before the worker, API,
or provider demo emits events:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/logging.rs:runtime_tracing_config}}
```

The important production rule is small:

```text
emitting structured events is not enough
the process must install a subscriber before work begins
```

`RUST_LOG` controls the filter. `LOG_FORMAT=compact` is the local default, and
`LOG_FORMAT=json` is available for log pipelines that expect machine-readable
events.

## Structured Worker Events

The worker should emit structured telemetry at the same boundaries where it
records durable events. This gives operators two views of the same transition:

```text
durable event:
  what the system committed

structured trace/log event:
  what the running process observed at that moment
```

The companion worker emits fields such as `worker_id`, `job_id`, `job_kind`,
`attempt_count`, `max_attempts`, `retry_disposition`, `event_type`, and
`outcome`.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/worker.rs:worker_observability}}
```

Notice what is not logged: the raw prompt, provider response, API key, or
unvalidated tool payload. Observability should explain production behavior
without widening the security boundary.

The useful invariant is:

```text
for every important worker transition, there is both durable evidence and
structured runtime evidence
```

## Audit And Operation Events

Do not collapse every event into one vague log stream. A reliable agent system
needs two durable evidence classes:

```text
audit event:
  who or what made a decision, about which subject, with what evidence?

operation event:
  what happened while the system was running, for which job or run, and how
  severe was it, with which trace context?
```

The distinction matters during incidents. If a tool call was blocked, the
operator needs audit evidence for the decision and operation evidence for the
runtime symptom. Logs may help explain timing, but durable rows should still be
able to prove what happened.

The companion implementation keeps raw database shapes at the boundary and
converts them into typed records:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/audit.rs:audit_event_boundary}}
```

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/audit.rs:operation_event_boundary}}
```

Notice the production rule:

```text
database row -> validated audit or operation record -> operator evidence
```

Unknown actor types, unknown severities, empty actions, missing subjects, and
non-object evidence are rejected before they enter the domain model. That is
observability as a boundary, not a best-effort print statement.

> ### 🎓 The Professor's Corner
>
> **The Black Box Recorder: More than "Error"**
>
> You don't want to find the flight recorder (the logs) and have it just say "Something went wrong." You want it to tell you the altitude, the speed, and what the pilot was doing! 
> 
> This is why we need **Audit and Operation Events**. They are the dials and gauges of our system. They tell us not just *that* we failed, but *what* we were thinking when we did.

The difference is simple but important.

An operation event helps operate the system: a job was claimed, a lease was
extended, a provider call failed, a retry was scheduled, a queue crossed a
threshold. An audit event helps prove business meaning: a policy denied an
action, a human approved a risky tool call, an agent proposed an update, a tenant
boundary blocked access.

Both are evidence. They answer different questions, so they should not be
merged into one generic table just because both have timestamps.

## SLOs

Example SLOs:

```text
99% of incident-triage jobs start within 2 minutes
99% of retryable provider failures recover within 30 minutes
0 unauthorized side-effect jobs execute without approval
0 secrets appear in event messages or last_error
```

The last two are correctness SLOs, not latency SLOs.

In an SRE-owned system, each SLO also needs:

```text
SLI query or measurement source
measurement window
error budget
burn-rate alert
owner for budget exhaustion
```

Chapter 21 expands this into a full SRE contract.

An SLO is not a dashboard goal.

It is a promise written in a form the system can measure. The promise should
start from user or operator experience, not from whatever metric is easiest to
graph. "The API process is up" is less meaningful than "99% of high-priority
triage jobs start within two minutes." "The model returned JSON" is less
meaningful than "unauthorized side effects never execute."

For agents, some SLOs measure speed. Some measure safety. Some measure recovery.
All of them need a reproducible measurement source.

## Formal Definition

For this chapter, the precise definition is:

```text
Observability is the ability to reconstruct behavior from correlated state, events, metrics, traces, and logs without relying on process memory.
```

In the book's system model:

- **State:** correlated state rows, event rows, audit rows, operation rows, metrics, traces, logs, and SLO measurements.
- **Actor:** application code emits structured evidence, and operators query it to explain behavior.
- **Transition:** each important state transition records enough correlated evidence for later reconstruction.
- **Evidence:** Trace ids, structured fields, operation events, audit events, metrics, and runbook queries answer the same job question.
- **Invariant:** operators can answer what happened without relying on live process memory or unstructured log archaeology.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Dashboards are built from convenient process logs. |
| Production symptom | Alerts fire without a path to the stuck job or failed invariant. |
| Corrective invariant | Observability signals agree with durable workflow evidence. |
| Evidence to inspect | Metrics, traces, logs, and event rows point to the same job and state transition. |


## Production Contract

Observability is adequate only when:

```text
every important state transition emits an event
audit and operation rows separate decisions from runtime symptoms
every important worker transition emits structured tracing fields
queue health can be measured without reading application logs
one job can be traced from enqueue to terminal state
trace ids and span ids are validated before becoming operation evidence
SLOs map to reproducible queries or scanners
SLO measurements are validated before they drive operational decisions
alerts point to a runbook action
```

If an operator must infer the job lifecycle from scattered logs, the system is
not yet observable.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Dashboards are built from convenient process logs. | Dashboards built from convenient logs cannot prove what happened to one durable job. |
| Safer version | Observability signals agree with durable workflow evidence. | Traces, metrics, logs, audit events, and operation events share job and trace identity. |
| Production version | Metrics, traces, logs, and event rows point to the same job and state transition. | On-call can reconstruct a single run and fleet-level SLO symptoms from correlated evidence. |

Use the naive row when observability means log search. Use the safer row to correlate signals. Use the production row before on-call must debug old work.

The hardening path changes observability from output to evidence.

The naive version emits text because something happened. The safer version gives
the text identity. The production version makes the identity durable, measurable,
and connected to an operator action.

This is the standard: every important signal should help answer either "what
happened to this run?" or "is the fleet still meeting its promise?"

## Testing Strategy

Test observability by reconstructing facts from evidence:

- **Unit or type test:** prove Rust trace ids, span ids, SLI measurements, SLO windows, and operation events reject malformed or impossible values.
- **Persistence or boundary test:** prove Postgres audit events, operation events, queue metrics, and SLI queries correlate to the same job or run.
- **Regression test:** remove or corrupt a trace id in a fixture and verify the observability test fails because on-call could no longer reconstruct the transition.

Observability tests should feel like operator drills.

Start with a symptom. Then require the test to find the job, event, trace, metric,
and SLO evidence that explains it. A test that only checks that a log line was
emitted is too weak. A stronger test proves the emitted evidence can be used to
reconstruct the workflow.

The test should fail if correlation breaks.

## Observability Strategy

Observe observability itself through correlated signals:

- Emit structured `tracing` fields for trace id, span id, job id, run id, SLI name, SLO window, status, and operation event id.
- Record an operation event whenever metrics, traces, logs, audit events, and SLO measurements are linked to the same state transition.
- The runbook query should let on-call move from fleet symptom to one job timeline without relying on unstructured log search.

There is a second-order question here: can you tell whether your observability is
working?

If an alert fires but the runbook cannot find matching rows, the observability
system has a gap. If traces exist but do not include job ids, the trace is hard
to connect to durable state. If audit events exist but do not include actor and
subject, they are weak evidence. Treat those as production bugs.

## Security and Safety Considerations

Observability must explain behavior without leaking data:

- Treat logs, metrics labels, trace attributes, and event payloads as untrusted disclosure surfaces until reviewed for sensitive content.
- authorization, sandboxing, and approval decisions should be observable as outcomes, but not by exposing secrets, prompts, or tenant payloads.
- Redact or bucket high-cardinality and sensitive fields while preserving trace id, SLI name, SLO window, decision, and evidence links.

The safest observability systems are useful without being nosy.

Operators usually need to know which job, tenant, tool, model route, status,
decision, and error class were involved. They usually do not need raw prompts,
full model outputs, API keys, customer secrets, or unredacted documents in logs.

Preserve correlation and decision evidence. Redact content that would widen the
blast radius of a logging or tracing leak.

## Operational Checklist

Use this checklist before relying on observability and SLO evidence in production:

- **State:** Metrics, traces, logs, operation events, and SLO measurements point to the
  same job lifecycle.
- **Boundary:** Telemetry values are typed correlation evidence, not raw strings copied
  through the system.
- **Failure:** An operator can distinguish queue backlog, provider latency, worker
  crash, policy block, and bad behavior.
- **Observability:** Trace id, span id, job id, run id, status, attempts, latency, cost,
  and SLO window are connected.
- **Safety:** Observability redacts payloads and secrets while preserving enough
  evidence for audit and incident response.

Use the checklist before adding a new worker, tool, or model route.

Every new production path should answer: What trace id follows it? Which
operation events does it emit? Which audit events record business decisions?
Which metrics show fleet health? Which SLO might it affect? Which runbook query
turns an alert into action?

If a new feature cannot answer those questions, it is not ready for unattended
production use.

## Exercises

1. Write a negative test where a failed tool call has logs but no operation event or
   trace link and therefore cannot satisfy the SLO evidence contract. Explain which
   idempotency key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: operation_events, queue metrics, SLI measurement rows,
   and job timeline for one incident.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   TraceContext, SliMeasurement, SloWindow, ErrorBudget, and ObservabilityLabel types.
   Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What is the difference between logging and observability?
- Explain: Why should an SLO start from user-facing reliability, not available metrics?
- Apply: Investigate an alert for oldest pending job age.
- Evidence: Name one metric, one state query, one trace id, one operation event, and one owner action.

## Summary

Observability is not logging more. It is designing state, events, metrics, traces, and SLOs so they all explain the same workflow from different distances.

- **Invariant:** an operator can reconstruct what happened without reading raw payloads or guessing from process memory.
- **Evidence:** trace context, operation events, queue metrics, latency and cost measurements, SLO windows, and runbook output correlate by job and run.
- **Carry forward:** if evidence cannot answer an incident question, the system is not observable yet.

## Changed Understanding

- **Before this chapter:** logs looked like enough operational visibility.
- **After this chapter:** operators need traces, metrics, logs, audit events, and SLOs that answer production questions from evidence.
- **Keep:** answer an operator question with trace id, metric, log field, audit event, and SLO evidence.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
