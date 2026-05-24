# 22. Capacity, Backpressure, And Provider Quotas

## What You Will Learn

This chapter teaches you to:

- explain how the system avoids overload when users, workers, or providers run hot;
- inspect admission control, queue depth, fairness, provider quota, worker concurrency, and backpressure state;
- verify that overload becomes controlled waiting or refusal, not hidden collapse.

The production evidence is a capacity policy tied to job admission, worker
limits, provider budgets, queue metrics, and operator controls.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** SLOs expose user-facing reliability pressure.
- **Adds:** admission control, fairness, quotas, and intentional degradation.
- **Prepares:** runbooks that turn signals into safe operator action.

## Production Failure

A provider slows down, retries pile up, and the worker pool accepts every new
request anyway.

Soon old jobs, new jobs, and retries compete until even low-risk work is late.

**What breaks:** the system treated demand as infinite and capacity as
guaranteed.

The queue kept accepting work even though the downstream system could not absorb
it. Every retry became more demand. Every new request joined a line that was
already too long. The system did not fail all at once. It failed by saying "yes"
too often.

**False fix:** add workers without checking provider quota, tenant fairness,
database pressure, or retry amplification.

More workers help only when worker count is the real bottleneck. If the provider
is already rate-limiting, more workers create more rate-limited calls. If the
database is already under write pressure, more workers create more contention.
If one tenant is consuming the queue, more workers can make unfairness faster.

**Design response:** make admission, delay, pause, and refusal explicit
decisions based on queue age, quota, priority, cost, and SLO pressure.

The system needs to decide before enqueue, before retry, and before execution:
can this work be accepted now without making the whole service less reliable?
That decision should be durable evidence, not an instinct hidden inside a route
handler.

## Motivation

In production, more workers do not always mean more capacity. Provider limits, tenant budgets, retry amplification, human review queues, and downstream latency can turn scale into overload.

Without backpressure, the system accepts work it cannot safely finish. This chapter makes admission control a durable decision instead of an after-the-fact apology.

## Plain Version

Read this as the simple version:

**Simple rule:** Backpressure protects the system when demand, database
capacity, or provider quota becomes tight.

Backpressure is the system saying "not now" while it can still say that safely.
It may delay low-priority work, reject bulk work, pause one job kind, lower
concurrency, or route around a strained provider.

**Why it matters:** An agent that accepts infinite work will eventually create
slow, expensive, or failed work.

Reliable systems do not prove strength by accepting everything. They prove
strength by preserving useful service when demand exceeds safe capacity.

**What to watch:** Watch queue depth, oldest pending age, provider limits,
tenant budgets, admission decisions, and paused job kinds.

## What You Already Know

Start with these anchors:

- SLOs say what the system promises.
- Capacity decides whether the system can keep those promises under load.
- Retries can amplify demand when providers or workers are already strained.

This chapter adds: backpressure and quota discipline. You will make overload a
controlled admission, fairness, delay, or refusal decision instead of a hidden
collapse.

## Focus Cue

Keep three things in view:

- **State:** queue pressure, provider pressure, tenant budget, worker concurrency, priority, pause state, cost, and token usage.
- **Move:** incoming work is shaped by current capacity evidence before it can amplify overload.
- **Proof:** Queue age, backlog, priority, worker concurrency, provider usage, tenant budgets, and pause state shape intake.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** admission-control and quota rules for queue depth, age, tenant budget, provider limits, and job kind.
- **Why it matters:** backpressure prevents retries and new work from amplifying overload.
- **Done when:** the system can delay, reject, or accept work with a recorded reason before capacity is exhausted.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/admission_control.rs`, cost accounting, provider usage SQL, and queue metric SQL.
- **State transition:** decide whether to accept, delay, or reject work before overload spreads.
- **Evidence path:** tenant budget, queue age, depth, quota, and provider pressure are recorded.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Should the system accept, delay, or reject the next unit of work?
- **Evidence to inspect:** queue depth, oldest pending age, tenant budget, provider quota, job kind, and admission decision reason.
- **Escalate if:** new work enters during overload without a recorded admission decision.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** new work arrives while capacity may be constrained.

This can be a user request, a scheduled job, a retry, a bulk import, or a model
tool action that creates follow-up work. The system should not treat all of
these as equal pressure.

**Action:** evaluate queue, tenant, provider, cost, and job-kind pressure.

The admission decision reads the current pressure before it creates more work.
Queue age tells you whether users are already waiting. Provider quota tells you
whether the model route can absorb another call. Tenant budget tells you whether
one customer is consuming more than their safe share. Priority tells you which
work should survive a constrained period.

**Persistence:** persist accept, delay, or reject reason.

The reason matters later. When a customer asks why work did not start, the
operator should be able to point to a durable admission decision, not guess from
a graph.

**Check:** verify overload is slowed before retries amplify it.

Backpressure is late if it starts after the retry storm. The system should slow
new or low-priority work while it still has enough capacity to protect critical
work.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** admission can accept, delay, or reject with a recorded reason.
- **Validation path:** inspect queue metrics, tenant budget, provider quota, cost accounting, and admission-control tests.
- **Stop if:** new work enters during overload without backpressure evidence.

The evidence should answer one operational question quickly: did the system
accept this work because capacity was healthy, delay it because capacity was
tight, or reject it because accepting it would harm the service?

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, more workers do not always mean more capacity
rule: Backpressure protects the system when demand, database capacity, or provider quota becomes tight
tiny example: queue pressure, provider pressure, tenant budget, worker concurrency, priority, pause state, cost, and token usage
artifact: admission-control and quota rules for queue depth, age, tenant budget, provider limits, and job kind
proof: admission can accept, delay, or reject with a recorded reason
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Capacity is a chain, not a knob:

```text
enqueue rate
worker concurrency
database writes
provider requests
provider tokens
tool latency
human approvals
```

The system is limited by the narrowest part of that chain. Adding workers only
helps when workers are the limiting resource.

This is the mental model to keep. Capacity is not one big pool. It is a chain of
small promises. A job may need a database write, a model request, tokens, a tool
call, and a human reviewer. The safe rate is bounded by the slowest or most
limited part of that chain.

When a team forgets this, it often scales the most visible layer. Workers are
easy to count, so the team adds workers. But if the provider quota is the narrow
part, the extra workers only hit the limit faster. If human review is the narrow
part, the extra workers create a larger approval backlog.

## Capacity Model

For one job kind:

```text
arrival rate: jobs per minute
service time: seconds per job attempt
worker concurrency: attempts in flight
provider quota: requests and tokens per minute
retry multiplier: extra attempts caused by failure
```

The safe worker count is bounded by the smallest downstream capacity:

```text
database write capacity
provider request quota
provider token quota
tool server capacity
approval team throughput
```

For agent systems, capacity also includes risk. A provider can be healthy while
the approval queue is saturated. A database can be healthy while tenant cost
budgets are exhausted. A worker pool can be idle while a high-risk action waits
for a human gate. Treat all of those as capacity signals.

## Tiny Example

If a provider allows 60 requests per minute and each job attempt makes one
provider call, 200 concurrent workers do not create 200 useful calls. They
create a rate-limit problem.

```text
safe capacity <= provider quota / calls per job attempt
```

Retries make this worse. A 20% timeout rate can turn 100 logical jobs into 120
attempts before users see success.

That is why retries and backpressure must be designed together. A retry is not
free. During an outage, retries can turn one provider problem into a queue
problem, then into an SLO problem, then into a cost problem.

Read the tiny case as:

```text
setup: provider quota is lower than worker demand
transition: admission, fairness, and worker concurrency reduce pressure before collapse
evidence: queue depth, provider quota, tenant budget, delayed job, or rejection record explains throttling
invariant: overload must become controlled waiting or refusal, not unbounded retries
```

## Backpressure

Backpressure means the system says "not now" before it collapses.

This is not the same as failure. A controlled delay is often the most reliable
thing the system can do. It protects critical work, keeps provider calls inside
quota, and prevents retries from turning a temporary constraint into a larger
incident.

Use backpressure when:

```text
oldest pending age is rising
provider 429s exceed baseline
dead jobs spike for one kind
expired leases grow
approval queue exceeds human capacity
tenant budget is near quota
```

Backpressure options:

```text
pause a job kind
reject low-priority enqueue requests
increase run_at delay for new low-priority jobs
lower worker concurrency
switch provider route
require manual approval for more actions
```

Each option has a different meaning. Pausing a job kind protects the whole
system from one unsafe workload. Rejecting low-priority enqueue requests protects
capacity for critical work. Increasing `run_at` delay turns immediate demand
into scheduled demand. Lowering concurrency reduces pressure on downstream
systems. Switching provider route can help only if the alternate route has
capacity and compatible behavior.

The example schema includes a job-kind pause table, and the picker excludes
paused kinds:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/pause_job_kind.sql}}
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/pick_due_job.sql}}
```

## Executable Admission Control

Backpressure is easiest to get wrong when it is hidden inside a route handler.
The decision should be visible enough to test:

```text
queue pressure + provider pressure + tenant budget + priority -> admission decision
```

The companion code models that decision as a typed policy:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/admission_control.rs:admission_policy}}
```

The outcome is not a boolean. It is a production decision:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/admission_control.rs:admission_decision}}
```

The evaluator combines queue metrics, provider pressure, budget state, and job
priority before the system enqueues work or schedules it for later:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/admission_control.rs:admission_control_evaluate}}
```

The HTTP boundary resolves duplicate idempotency keys before admission pressure
is evaluated. Existing work is not new load; it is a lookup that returns the
known durable job and records `duplicate_suppressed`. For new work, the boundary
then applies the policy before enqueue. Accepted work is enqueued immediately,
delayed work is enqueued with a later `run_at`, and rejected work records an
admission decision without creating durable work:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/api.rs:api_admission_enforcement}}
```

For accepted and delayed work, the Postgres path records the job row, the job
event, and the admission decision in one statement. This is the intake
transaction boundary: either the system can prove both the work and the reason
it was admitted, or the write fails.

Duplicate intake has its own single-statement lookup and event record:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/resolve_existing_agent_job.sql:resolve_existing_agent_job}}
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/admit_agent_job.sql:transactional_admission_enqueue}}
```

The decision itself is durable operator evidence, not a log line that can be
lost during an incident:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql:admission_decision_events_table}}
```

This is the difference between "the queue is large" and "the system made a
bounded, explainable intake decision." If the tenant budget is exhausted, the
request is rejected before it becomes durable work. If the provider is near its
quota, bulk work can be delayed while higher-priority work still has a path.
If the queue is saturated, the next run time becomes part of the decision
rather than a hidden sleep in a worker. If an operator asks why a customer job
did not start, the database can answer with request id, tenant, priority,
queue pressure, provider pressure, budget state, decision, reason, and
timestamp.

## Fairness

A single noisy job kind should not starve everything else.

Fairness is not about making every job identical. It is about making sure one
tenant, job kind, retry pattern, or bulk workflow cannot consume the whole
system while more urgent or safer work waits behind it.

The first fairness level is per-kind visibility:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql}}
```

If one kind dominates the queue, split scheduling policy:

```text
separate worker pools
per-kind concurrency limits
per-tenant admission limits
priority bands
separate queues for critical work
```

Do not add these before you can measure the unfairness. Each scheduling rule is
an operational contract.

That last sentence is important. A priority rule is not decoration. It changes
who waits and who moves. A per-tenant limit changes the customer experience. A
separate worker pool protects one class of work but may strand capacity in
another. Add the rule only when the evidence shows which unfairness you are
solving.

## Provider Quotas

Provider limits are part of your system's capacity model.

For an AI agent system, the model provider is often the narrowest part of the
chain. It may limit requests per minute, tokens per minute, concurrent requests,
context length, or spend. Treat those limits as product constraints, not remote
annoyances.

Track:

```text
requests per minute
tokens per minute
429/rate-limit responses
timeout rate
cost per job kind
cost per tenant
```

Retryable provider errors should use backoff. Permanent provider configuration
errors should dead-letter quickly.

Do not retry all provider errors the same way. A temporary timeout may deserve
bounded backoff. A 429 should reduce pressure. An invalid API key or unsupported
model should fail quickly because retrying it only adds noise and cost.

## Typed Usage And Budget Evidence

Provider usage is not just a dashboard number. It decides whether a tenant can
continue to enqueue work, whether a provider route is safe, and whether retries
are amplifying cost.

The raw provider response may contain usage fields as JSON or as provider-
specific names. That is allowed at the provider boundary. Inside the system,
usage becomes typed operational evidence:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/cost_accounting.rs:provider_usage_typed}}
```

The important invariant is:

```text
prompt tokens + completion tokens = total tokens
```

If this invariant fails, the system should reject the usage record rather than
silently corrupting cost and capacity data. A budget decision then becomes a
typed result instead of an informal comparison against a dashboard.

The database boundary follows the same raw-outside, typed-inside rule:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/cost_accounting.rs:provider_usage_row_boundary}}
```

The tracking schema stores provider usage as durable evidence:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql}}
```

This query answers the operator question:

```text
which job kinds and provider routes are burning requests, tokens, cost, and latency?
```

Cost control is part of reliability. An agent that stays available by spending
unbounded money is not reliable. It is missing an admission-control invariant.

This is where capacity and finance meet. A system can be technically alive while
burning money at a rate the business cannot tolerate. That is still an
operational failure. Reliable agents need cost budgets because model calls and
tool calls are part of the capacity envelope.

## Formal Definition

For this chapter, the precise definition is:

```text
Capacity control is the admission and execution policy that keeps work, providers, tenants, costs, and approvals inside safe limits.
```

In the book's system model, **State** means queue pressure, provider pressure,
tenant budget, worker concurrency, priority, pause state, cost, and token usage.

The **Actor** is admission control, a worker, or an operator deciding whether to
accept, delay, reject, pause, or resume work.

The **Transition** is that incoming work is shaped by current capacity evidence
before it can amplify overload.

The **Evidence** is queue age, backlog, priority, worker concurrency, provider
usage, tenant budgets, and pause state shaping intake.

The **Invariant** is that pressure causes controlled admission and execution
decisions, not retry storms or unbounded spend.

## What Can Fail

**Design smell:** overload is solved only by adding workers. This assumes the
worker pool is the bottleneck without proving it.

**Production symptom:** provider 429s, retry storms, queue age, cost, and p95
latency rise together. The system is not merely busy. It is amplifying pressure
across several boundaries at once.

**Corrective invariant:** admission, execution, provider pressure, tenant
budget, and usage evidence are controlled together.

**Evidence to inspect:** queue age, per-kind backlog, worker concurrency,
provider usage rows, token totals, cost, latency, and budget decisions agree.


## Production Contract

Capacity control is correct only when admission can slow or reject low-priority
work and worker concurrency is bounded by downstream limits. Provider 429s
should reduce pressure instead of increasing retries. Job-kind and tenant
metrics should expose unfairness before one workload starves the rest of the
system.

The contract also needs durable control. Pause and resume actions must be
recorded and explained. Cost and approval capacity must be part of the model.
Token totals and budget decisions must be typed evidence, not loose dashboard
numbers.

## Progressive Hardening Path

**Naive version:** overload is solved only by adding workers. This may help when
workers are the bottleneck, but during provider pressure it can amplify rate
limits, costs, retries, and tenant harm.

**Safer version:** admission, execution, provider pressure, tenant budget, and
usage evidence are controlled together. Admission control now combines queue
pressure, priority, tenant budget, provider quota, and cost evidence before work
is accepted.

**Production version:** queue age, per-kind backlog, worker concurrency,
provider usage rows, token totals, cost, latency, and budget decisions agree.
The system can delay, reject, pause, or admit work based on queryable pressure
rather than intuition. Use the naive version only to spot the smell. Use the
safer version to measure pressure. Use the production version before overload can
trigger expensive or unsafe behavior.

## Testing Strategy

Test capacity control before overload reaches the provider:

- **Unit or type test:** prove Rust admission policy handles queue pressure, provider quota, tenant budget, priority, and admission delay without raw numeric confusion.
- **Persistence or boundary test:** prove Postgres queue metrics, provider usage, cost rows, and admission-decision rows agree about the pressure signal.
- **Regression test:** simulate a saturated queue or exhausted provider quota; low-priority work should delay or reject before enqueueing more unsafe load.

## Observability Strategy

Observe capacity pressure before it becomes an incident.

Emit structured `tracing` fields for job kind, tenant, priority, queue depth,
oldest pending age, provider route, quota pressure, budget state, and trace id.
Those fields connect one request to the capacity signal that shaped it.

Record an operation event when admission accepts, delays, rejects, or pauses work
because of queue, provider, cost, or tenant pressure. The event is what turns
backpressure from an invisible slowdown into an explainable production decision.

The runbook query should explain why a request was admitted or rejected and
which capacity signal controlled the decision.

## Security and Safety Considerations

Capacity controls must not become unfair or unsafe bypasses.

Treat tenant budget rows, provider quota signals, priority labels, and admission
payloads as untrusted until validated and scoped. A forged priority label should
not jump the line. A malformed provider usage row should not open the gate to
more work.

authorization, sandboxing, and approval still apply to admitted work;
backpressure is not permission to skip safety gates. Redact tenant-sensitive
cost and usage details while preserving quota pressure, budget decision,
priority, and admission evidence.

## Operational Checklist

Use this checklist before relying on capacity, backpressure, and provider quotas
in production.

**State:** Admission decisions account for queue depth, oldest pending age,
tenant budget, provider quota, and worker capacity.

**Boundary:** Provider quota responses and tenant budget rows become typed
admission decisions before enqueue.

**Failure:** Overload delays or rejects work intentionally instead of creating
retry storms and hidden provider debt.

**Observability:** Queue health, admission decision, quota usage, cost, tenant,
and job kind are visible in metrics and events.

**Safety:** Backpressure preserves fairness, redaction, authorization, and
approval requirements even under load.

## Exercises

1. Write a negative test where provider quota is exhausted and bulk work is rejected
   without losing the intake idempotency evidence. Explain which idempotency key,
   receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: admission_events, provider_usage, tenant_budget, queue
   metrics, and oldest_pending_job query output.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   AdmissionDecision, ProviderQuota, TenantBudget, QueuePressure, and JobPriority types.
   Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which limits can overload: users, queue, workers, provider, or tenant budget?
- Explain: Why adding workers can make provider 429s worse.
- Apply: Choose an admission control and worker control for rising queue depth and provider errors.
- Evidence: Name queue depth, oldest age, provider quota, tenant budget, and the delay or rejection record.

## Summary

Capacity planning is not just worker count. It is the relationship between
arrival rate, service time, downstream quotas, retry amplification, tenant
budgets, and human approval throughput.

- **Invariant:** admission decisions protect the system from overload while preserving fairness and evidence.
- **Evidence:** queue metrics, oldest pending age, provider usage, tenant budgets, admission events, cost rows, and trace ids explain accepted, delayed, or rejected work.
- **Carry forward:** backpressure is a product decision made durable.

## Changed Understanding

- **Before this chapter:** capacity looked like adding more workers when jobs pile up.
- **After this chapter:** capacity is controlled flow across database queues, workers, model providers, quotas, budgets, and backpressure.
- **Keep:** inspect backlog, worker saturation, provider quota, cost budget, and admission-control state together.

## Further Reading and Sources

- [PostgreSQL `SELECT` documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) explains the row-selection and locking primitives behind job claiming and queue inspection.
- [PostgreSQL explicit locking documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) is the source to use when reasoning about worker leases, concurrent claims, and lock contention.
- [PostgreSQL transaction isolation documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) sharpens the chapter's treatment of atomic retries, status transitions, and recovery queries.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) connects the local Postgres design to broader principles for durable, auditable systems.
- [DeepSeek API docs](./31-credible-resources-further-reading.md#agent-architecture) is the provider source to check before changing model routes, usage fields, or request limits.
- [OpenTelemetry documentation](./31-credible-resources-further-reading.md#reliability-and-operations) supports the chapter's separation of durable usage evidence from live metrics and traces.
