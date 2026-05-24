# 21. SLIs, SLOs, And Error Budgets

## What You Will Learn

This chapter teaches you to:

- explain which measurements represent user-facing reliability;
- inspect the SQL source for each SLI, the SLO target, budget burn, and alert rule;
- verify that reliability claims come from durable events and user impact, not dashboard decoration.

The production evidence is an SLI/SLO table tied to run states, latency,
freshness, failure rates, budget policy, and diagnostic queries.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** one worked scenario shows the evidence chain end to end.
- **Adds:** measured reliability promises and error-budget decisions.
- **Prepares:** capacity, backpressure, and provider quota controls.

## Production Failure

A support agent is technically online, but half of urgent tickets wait forty
minutes before the first useful action.

The uptime dashboard stays green because the API still responds.

**What breaks:** availability hid user-visible failure.

The system measured whether a server answered. It did not measure whether the
agent made useful progress for the user. From the infrastructure point of view,
the service looked alive. From the customer's point of view, the service failed.

**False fix:** add more logs and call the service healthy because requests
return `200`.

Logs may explain what happened after someone starts investigating. They do not
define the promise the service made to the user. A green dashboard can still be
wrong if it measures the wrong thing.

**Design response:** define SLIs from durable user-impact evidence, set SLOs
for acceptable behavior, and use error budgets to decide when releases or
capacity changes must slow down.

The fix is not a prettier dashboard. The fix is a measurable contract. If urgent
ticket agents must start useful work within two minutes, the system needs a
query that proves whether that happened, a target that defines acceptable
behavior, and a budget rule that changes how the team acts when the promise is
being broken.

## Motivation

In production, reliability must be measured as a promise. For agents, that promise includes progress, latency, availability, cost, safety, and behavior quality.

Without SLIs, SLOs, and error budgets, teams cannot decide when to release, pause, investigate, or invest in reliability work. This chapter turns operational health into measurable production evidence.

## Plain Version

Read this as the simple version:

**Simple rule:** Turn reliability into measured promises and budgeted failure,
not vague uptime language.

A promise like "the agent is reliable" is too soft to operate. A promise like
"99% of incident-triage jobs start within two minutes over thirty days" can be
measured, debated, tested, alerted on, and used to decide whether a release is
safe.

**Why it matters:** Operators need to know when the agent system is healthy
enough to ship and when change must slow down.

Without an error budget, every incident becomes a negotiation. Product wants to
ship. Engineering wants to stabilize. Support sees user pain. The budget gives
the team a shared rule: if the system is spending failure too quickly,
reliability work comes first.

**What to watch:** Watch user-visible SLIs, SLO windows, bad-event counts,
budget burn, and release-gate decisions.

## What You Already Know

Start with these anchors:

- Part II made the system observable through state rows, events, metrics, and runbooks.
- Observability is raw material, not yet a promise.
- Users experience reliability through latency, freshness, correctness, and completion.

This chapter adds: SLIs, SLOs, and error budgets. You will turn durable evidence
into measurable promises and decide what happens when the budget burns too
fast.

## Focus Cue

Keep three things in view:

- **State:** an SLI source, SLO objective, measurement window, error budget, burn signal, and owner decision.
- **Move:** raw operational counts become typed measurements, budget status, alerts, and release or capacity decisions.
- **Proof:** Query sources, typed measurements, windows, targets, burn alerts, owners, and release decisions are explicit.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** SLI, SLO, and error-budget measurement rows tied to agent job outcomes.
- **Why it matters:** reliability targets should change admission, release, and incident behavior.
- **Done when:** bad events, total events, windows, burn, and no-traffic decisions are measurable from durable evidence.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/slo.rs`, SLI SQL, release gate integration, and error-budget tests.
- **State transition:** turn job outcomes into measured reliability decisions.
- **Evidence path:** window, good count, total count, budget remaining, and no-traffic status are valid.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Is the service inside its reliability budget for this job kind?
- **Evidence to inspect:** SLI numerator, denominator, window, bad events, no-traffic state, burn rate, and release decision.
- **Escalate if:** the team talks about reliability without measuring the exact user-visible promise.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** a service-level promise is evaluated for a window.

This is usually a scheduled measurement job, a release gate, or an incident
review. The important point is that the question is concrete: "During this
window, for this job kind, did the service keep the promise?"

**Action:** count good and bad events for the job kind.

Good and bad must be defined before the query runs. A job that starts within the
target may be good for a start-latency SLI. A job that dead-letters may be bad
for a terminal-outcome SLI. An unauthorized side effect is not a slow event; it
is a safety failure.

**Persistence:** persist SLI, SLO, budget, and no-traffic measurements.

The measurement is itself production evidence. It should not live only inside a
dashboard formula. Store the window, target, numerator, denominator, decision,
and no-traffic state so release gates, runbooks, and postmortems can inspect the
same facts.

**Check:** verify release and incident decisions use the budget result.

The budget matters only if it changes behavior. If the error budget is exhausted
and the release process continues as if nothing happened, the SLO is decoration,
not control.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** job-kind reliability decisions use measured budget evidence.
- **Validation path:** inspect SLI rows, SLO window, bad-event counts, no-traffic state, and release-gate tests.
- **Stop if:** availability or behavior release decisions ignore the measured budget.

The evidence should answer three simple questions without a meeting: what was
the promise, how many events broke it, and what decision did the system make?

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, reliability must be measured as a promise
rule: Turn reliability into measured promises and budgeted failure, not vague uptime language
tiny example: an SLI source, SLO objective, measurement window, error budget, burn signal, and owner decision
artifact: SLI, SLO, and error-budget measurement rows tied to agent job outcomes
proof: job-kind reliability decisions use measured budget evidence
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

An SLI is the instrument. An SLO is the promise. An error budget is the amount
of broken promise the team is willing to tolerate before changing priorities.

```text
SLI -> what we measure
SLO -> what we promise
budget -> how much failure is acceptable
alert -> when action is required
```

The important move is turning reliability from opinion into a contract.

Think of a medical monitor. The number on the screen is not the patient's health
itself, but a good measurement tells the doctor when to act. An SLI is the
measurement. The SLO is the threshold that says what good enough means. The error
budget is the margin between normal imperfection and a system that needs
attention now.

> ### 🎓 The Professor's Corner
>
> **Percentiles vs. Averages: The Lying Average**
>
> In distributed systems, the "Average" is a liar! If half your jobs take 1 second and half take 1 hour, the average is 30 minutes. But *none* of your users waited 30 minutes! 
> 
> You need **Percentiles** (like P95 and P99) to see the **Long Tail of Failure**. A P99 of 1 hour tells you that 1% of your users are having a terrible time. The average hides the pain; the percentile exposes it.

Agent systems need this because they can fail in ways a normal uptime check will
... (omitted) ...
miss. The API can respond while jobs are stuck. The model can answer quickly
while using stale memory. A tool can succeed technically while bypassing an
approval rule. Reliable agent operations require measurements that follow user
impact, not only process liveness.

## SLIs

An SLI is the measurement. Start with these:

```text
job_start_latency_seconds:
  picked_at - eligible_run_at

retry_recovery_latency_seconds:
  succeeded_at - first_retryable_failure_at

dead_job_rate:
  dead jobs / total terminal jobs

approval_bypass_count:
  side-effect jobs without required approval

secret_leak_count:
  event or last_error rows matching secret scanners
```

Some values come from state rows, some from event timelines, and some from
security scanners. The important rule is that each SLI must have a reproducible
query or measurement pipeline.

Choose SLIs by asking what the user would call failure. If the user is waiting
for an incident response, start latency matters. If the system promises recovery
from transient provider errors, retry recovery latency matters. If the agent can
perform side effects, approval bypasses matter more than server uptime.

We should also measure **Quality SLIs**. For example, "99% of triage jobs result in a valid proposal." If the "Good Event" count drops because models are failing to parse, that's an SLO breach that requires a prompt engineer, not a SRE. These SLIs should be part of our **Continuous Evaluation** pipeline.

In AI, performance is inextricably linked to cost. We should add a **Cost SLI**: for example, "99% of triage jobs cost less than $0.05." If a worker "loops" or "retries" with a larger model, causing a cost spike, our **Unit Cost** metric will catch it before it burns the budget.

This is where agent reliability becomes different from ordinary web reliability.
A web API can often start with request success rate and latency. An agent system
also needs progress, safety, freshness, cost, and behavior signals. A fast agent
that sends an unauthorized email is not reliable. A cheap agent that leaves work
stuck for hours is not reliable. A fluent agent that cannot prove its tool
actions is not reliable.

## Tiny Example

Suppose incident-triage jobs should start quickly:

```text
SLI: picked_at - eligible_run_at
SLO: 99% start within 2 minutes over 30 days
```

If 1,000 jobs were eligible in the window, the budget allows 10 slow starts. The
eleventh slow start is no longer an anecdote. It is budget exhaustion.

This changes the conversation. Without the SLO, the eleventh slow start may sound
like "one more unlucky case." With the SLO, it is evidence that the system has
spent the failure it was allowed to spend for that window.

Read the tiny case as:

```text
setup: incident-triage jobs promise fast starts
transition: eligible and picked timestamps become an SLI over a window
evidence: measurement row, SLO target, error budget, and burn alert are linked
invariant: reliability promises must be measurable from durable production evidence
```

## SLOs

Example SLOs:

```text
99% of incident-triage jobs start within 2 minutes over 30 days
99% of retryable provider failures recover within 30 minutes over 30 days
99.9% of due jobs are not stuck behind expired leases for more than 5 minutes
0 unauthorized side-effect jobs execute without approval
0 secrets are written to event messages or last_error
```

The last two are correctness SLOs. They do not get a relaxed error budget
because the consequence is trust or security damage.

This is a key distinction. Latency SLOs often allow a small amount of badness
because networks, providers, and workers sometimes fail. Correctness and safety
SLOs may need a zero-tolerance target because one bypassed approval can create a
real business or security incident.

Do not copy another team's SLOs blindly. Choose promises that match your product
risk. A research agent may tolerate slower work during a provider outage. A KYC
case-preparation agent must not silently approve a high-risk case. A deployment
assistant must not take a write action after its approval expires.

## Error Budgets

An error budget is the amount of unreliability the service is allowed to spend.

For a 99% 30-day start-latency SLO:

```text
allowed bad jobs = total eligible jobs * 0.01
```

If the service spends the budget quickly, stop feature work that increases risk
and invest in reliability:

```text
slow deploys
provider fallback
queue partitioning
better backpressure
operator automation
```
The budget is not punishment. It is a planning tool. It lets a team move fast
when the system is healthy and slow down when users are already absorbing too
much failure.

> ### 🎓 The Professor's Corner
>
> **The Error Allowance: Candy vs. Movies**
>
> Think of an **Error Budget** like an allowance your parents give you. If you spend it all on candy (unreliable code) on the first day, you can't go to the movies (ship new features) later in the week! 
> 
> The budget makes the concept of "Failure" personal. It teaches you that every bug has a cost, and if you're out of money, you have to stay home and clean your room (fix the reliability) before you can play again!

For reliable agents, this matters because new model behavior
... (omitted) ...
prompts, and new background-job paths all add risk. If the budget is healthy, a
small canary may be reasonable. If the budget is exhausted, the next release
should improve reliability or wait.

## Burn-Rate Alerts

Alert on fast budget burn, not isolated noise.

```text
page:
  2% of the monthly budget burned in 1 hour

ticket:
  10% of the monthly budget burned in 3 days
```

The page means users are being hurt now. The ticket means the trend will become
an incident if ignored.

Burn-rate alerting prevents two common mistakes. The first mistake is paging for
every single bad event, which creates noise and teaches operators to ignore the
system. The second mistake is waiting until the whole monthly budget is gone,
which discovers the problem after the users have already paid the price.

A good alert tells the team both severity and urgency. "Two percent of the
monthly budget burned in one hour" means the system is failing quickly enough to
wake someone. "Ten percent burned in three days" means the team should plan
stability work before the trend becomes an outage.

## Local Query Surface

The companion example exposes the core metrics query:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/queue_metrics.sql}}
```

It also exposes a per-kind query for capacity and tenant fairness dashboards:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql}}
```

Those queries answer health questions. SLO work needs a stricter artifact: a
measurement row that names the SLO, the SLI, the window, the target, the good
event count, and the total event count.

This difference is important. Queue metrics are useful for diagnosis. SLO
measurements are useful for decisions. A queue-depth chart can help explain why
jobs are slow. An SLO measurement says whether the service kept the user-facing
promise during the window.

The companion implementation includes an executable job-start-latency SLI:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql}}
```

It also includes a terminal-job SLI. This one measures whether terminal jobs
ended in acceptable terminal states rather than dead-letter:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/sli_terminal_jobs_not_dead.sql}}
```

The important detail is not the exact query. The important detail is the shape
of the evidence:

```text
slo_name
sli_name
job_kind
window_started_at
window_ended_at
target_basis_points
good_events
total_events
```

That shape lets a worker, dashboard, or release gate evaluate the same SLO
without relying on dashboard-only math.

When the evidence has this shape, the release gate and the incident runbook stop
depending on a screenshot. They can use the same measurement row. That makes the
system easier to operate during stress, when informal interpretation is most
likely to drift.

## Typed Budget Evaluation

Raw SQL counts are still raw data. After they cross the database boundary, the
application converts them into domain values and rejects impossible
measurements:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/slo.rs:slo_row_boundary}}
```

This conversion protects several production invariants:

```text
SLO and SLI names are not empty
the window ends after it starts
the target is represented as basis points, not floating-point folklore
event counts cannot be negative
good events cannot exceed total events
job kind is parsed into the normal domain type
```

Once the measurement is typed, the budget calculation is ordinary deterministic
software:

```text
bad events = total events - good events
allowed bad events = total events * allowed_error_basis_points / 10000
decision = no traffic | within budget | budget exhausted
```

This is the same design principle used throughout the book:

```text
raw outside, typed inside
```

An SLO query is allowed to use storage-friendly fields. The domain layer is not.
If a database row says that 11 events were good out of 10 total events, the
answer is not "show a weird dashboard." The answer is "reject the measurement
before it controls production behavior."

This is the same reason the book uses typed jobs, typed tool calls, and typed
approvals. Once a value controls production behavior, it is no longer "just a
number." It is a decision input. Decision inputs need validation, names, units,
and failure behavior.

## Formal Definition

For this chapter, the precise definition is:

```text
An SLI is a reproducible measurement, an SLO is a promise over that measurement, and an error budget is the allowed failure before behavior changes.
```

In the book's system model, **State** means the SLI source, SLO objective,
measurement window, error budget, burn signal, and owner decision.

The **Actor** is the SRE, service owner, or release owner who interprets the
measurement and changes behavior when the budget demands it.

The **Transition** is from raw operational counts to typed measurements, budget
status, alerts, and release or capacity decisions.

The **Evidence** is explicit: query sources, typed measurements, windows,
targets, burn alerts, owners, and release decisions can be inspected later.

The **Invariant** is that reliability promises are measurable and have
consequences when they are breached.

## What Can Fail

**Design smell:** reliability is described as "healthy" or "fast" without a
query. The team may agree with the words, but no one can point to the evidence
that proves them. Adjectives do not operate systems.

**Production symptom:** teams debate incidents using impressions instead of
measured promises. A dashboard may show queue depth, CPU, and request latency,
but none of those may represent the user-facing promise.

The third failure is unsafe budget math. Raw counts may be stale, impossible, or
measured over the wrong window. If those values bypass typed validation, they can
open a release gate when the service should be frozen.

**Corrective invariant:** every SLI has a durable or reproducible measurement
source.

**Evidence to inspect:** every SLO decision can be traced to validated
measurement rows, windows, burn-rate alerts, and owners.


## Production Contract

An SLO is usable only when the SLI has a reproducible source, raw SLI rows are
validated into typed measurements, and the measurement window is defined. The
budget must be calculated from real traffic, including an explicit no-traffic
state, because an empty window is not the same as perfect reliability.

The contract also needs action. Alerts should distinguish page-worthy burn from
ticket-worthy drift. Budget exhaustion must change release or capacity
decisions. Correctness SLOs must be treated as safety contracts, not as soft
preferences.

## Progressive Hardening Path

**Naive version:** reliability is described as "healthy" or "fast" without a query.
This feels useful in conversation, but it cannot guide release decisions or
incident response. No one knows whether a bad week is inside the normal budget or
evidence that the system is getting worse.

**Safer version:** every SLI has a durable or reproducible measurement source.
Now each promise has a queryable source, window, target, and ownership model.
This is enough to start learning from production behavior.

**Production version:** SLI queries, typed measurement conversion,
SLO windows, burn-rate alerts, and owners. At this point error budgets can block
releases, trigger runbooks, and expose reliability tradeoffs with concrete
numbers. Use the naive version only to spot the smell. Use the safer version to
measure. Use the production version before SLOs affect release or incident
policy.

## Testing Strategy

Test reliability numbers as typed measurements:

- **Unit or type test:** prove Rust SLI and SLO types reject invalid windows, impossible good/total counts, and targets outside the allowed basis-point range.
- **Persistence or boundary test:** prove Postgres SLI queries produce reproducible measurement rows with window, target, good events, and total events.
- **Regression test:** feed an impossible measurement, such as good events greater than total events, and require the release or alert path to fail closed.

## Observability Strategy

Observe SLOs from reproducible measurement rows.

Emit structured `tracing` fields for SLI name, SLO name, measurement window,
target basis points, good events, total events, budget state, and trace id. Those
fields let an operator connect the alert to the measurement job that produced
it.

Record an operation event when an SLO burn, no-traffic decision, exhausted
budget, or release-blocking reliability result is computed. The event is the
bridge between measurement and action.

The runbook query should reproduce the SLI numerator, denominator, window, and
owner before anyone debates reliability from anecdotes.

I call this the **"Check Your Math"** rule. If you have 11 apples and give away 10, you can't have -1 left! If your monitoring math produces impossible numbers, it must "Fail Closed" and stop the release. Defensive engineering applies even to the gauges on your dashboard.

## Security and Safety Considerations

Reliability measurements can leak or distort safety signals.

Treat SLI event rows, metric labels, and alert annotations as untrusted until
they are validated for correctness and disclosure risk. A metric label can leak a
tenant name. An alert annotation can expose a prompt fragment. A raw error field
can carry a secret.

authorization, sandboxing, and approval outcomes may contribute to safety SLIs,
but they should not expose raw policy payloads or secrets. Redact tenant and
prompt data from SLO evidence while preserving window, target, good events, total
events, and budget decision.

## Operational Checklist

Use this checklist before relying on SLIs, SLOs, and error budgets in production.

**State:** Each SLI measurement has a window, numerator, denominator, target,
budget, and decision state.

**Boundary:** Raw metrics become typed SLI measurements before they block
releases or trigger incidents.

**Failure:** Bad windows, no traffic, stale data, and budget exhaustion become
explicit operational states.

**Observability:** Burn alerts, SLO reports, job metrics, provider latency, and
failure reasons share traceable sources.

**Safety:** SLO decisions do not expose sensitive payloads and do not override
policy, approval, or security gates.

## Exercises

1. Write a negative test where a release proceeds despite exhausted error budget and
   missing idempotency evidence for failed jobs. Explain which idempotency key, receipt,
   or state transition prevents duplicate work.
2. Sketch the Postgres evidence: SLI measurement rows linked to job status, latency,
   retry, dead-letter, and provider usage queries.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   SliName, SliWindow, SloTarget, ErrorBudget, and ReleaseBudgetDecision types. Then
   name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What is an SLI, an SLO, and an error budget?
- Explain: Why does a reliability promise need a measurement source?
- Apply: Define "jobs start quickly" as an SLI with a window and target.
- Evidence: Name the SQL measurement, budget calculation, burn alert, and action when the budget burns too fast.

## Summary

SRE work starts when reliability is measured as a contract. For agent systems, latency SLOs and correctness SLOs belong together because a fast unsafe agent is not reliable.

- **Invariant:** each SLI and SLO decision is tied to durable measurement data and a clear operator action.
- **Evidence:** SLI windows, good and bad event counts, error budgets, burn alerts, eval-linked correctness signals, and release gates show service health.
- **Carry forward:** measure what the user and the business need, not only what is easy to count.

## Changed Understanding

- **Before this chapter:** reliability looked like a vague promise that the agent works well.
- **After this chapter:** reliability becomes operable when user-visible behavior is expressed as SLIs, SLOs, and error budgets.
- **Keep:** tie each reliability claim to one SLI query, one SLO target, and one error-budget decision.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
