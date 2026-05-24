# 29.5 Extreme Fault Tolerance For Agent Systems

## What You Will Learn

This chapter teaches you to:

- explain isolation, redundancy, and static stability in plain language;
- separate the control plane from the execution plane in an agent system;
- design agent workers so one failing part does not stop the whole product;
- review failover drills, release gates, and last-known-good versions before trusting a job kind.

The production evidence is a fault-tolerance review row for each important job
kind. It records worker redundancy, failure-domain isolation, static stability,
failover drill evidence, release-gate evidence, and next review date.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** disaster recovery taught how to restore and replay after a bad event.
- **Adds:** how the system keeps serving while parts fail.
- **Prepares:** the maturity model, where each job kind gets a realistic reliability target.

## Production Failure

The dashboard service goes down during a provider incident.

Workers are still alive. Postgres is still accepting writes. But the workers
ask the dashboard service for the latest policy before every tool call.

Now the agent cannot process jobs, even though the critical execution path
could have continued from the last approved policy.

- **What breaks:** the execution plane depended on the control plane during an outage.
- **False fix:** restart everything and hope the dashboard comes back.
- **Design response:** isolate the execution plane, store last-known-good policy
  and prompt versions durably, run redundant workers, practice failover, and
  release changes progressively.

## Motivation

Extreme fault tolerance sounds advanced. The core idea is simple:

```text
Important work should continue when less-important parts fail.
```

For agent systems, this matters because long-running work may span minutes,
hours, or days. The model provider may slow down. A worker may crash. A
dashboard may be unavailable. A new prompt may be worse than the old prompt.
An availability zone may fail.

The system should not collapse because one non-critical part is sick.

The model may guess. The system must know which parts are allowed to fail
without stopping critical work.

## Plain Version

Read this as the simple version:

- **Isolation:** keep parts separate so one failure does not spread.
- **Redundancy:** keep more than one copy of important workers and data paths.
- **Static stability:** if a control service fails, continue from the last known good state.
- **Progressive delivery:** ship changes slowly so your own mistake does not reach everyone at once.
- **Always be failing over:** practice failover before the emergency.

For this book, the base implementation is still boring:

```text
Rust worker + Rig agent boundary + Postgres evidence + release gates + drills
```

No new infrastructure is required to learn the principle.

## What You Already Know

Start with these anchors:

- A job table is durable work memory.
- A lease says one worker owns work for a limited time.
- A side effect needs identity.
- A release gate blocks unsafe prompt, model, schema, or worker changes.
- A restore drill proves recovery after loss.

This chapter adds the next question:

```text
Which parts can fail while the critical agent job path keeps operating?
```

## Focus Cue

Keep three things in view:

- **State:** last-known-good prompt, model, policy, worker redundancy, failure domain, release gate, and failover drill.
- **Move:** critical work continues only from approved durable state when control services are degraded.
- **Proof:** a fault-tolerance review row shows isolation, redundancy, static stability, and drill evidence.

If the chapter feels large, come back to one sentence:

```text
The execution plane should not need a healthy control plane to finish already-approved work.
```

## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a fault-tolerance review for each production job kind.
- **Why it matters:** without this review, the team cannot tell whether an outage in one part will cascade into agent execution.
- **Done when:** the review names redundant workers, isolated failure domain, static-stability mode, last-known-good versions, release gate, failover drill, owner, and next review date.

## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `fault_tolerance_reviews`, `fault_tolerance_readiness.sql`, and `src/fault_tolerance.rs`.
- **State transition:** a job kind moves from fragile to ready only when redundancy, static stability, failover, and release evidence agree.
- **Evidence path:** Postgres stores the raw review; Rust decodes it into typed readiness evidence.

## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** If the control plane is down, can the execution plane continue from last-known-good state?
- **Evidence to inspect:** `fault_tolerance_reviews`, failover drill status, release gate decision, redundant worker count, and last-known-good versions.
- **Escalate if:** production execution depends on a dashboard, admin API, prompt registry, or policy service that can be unavailable.

## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a job kind is reviewed for fault tolerance.
2. **Action:** compare current redundancy, isolation, static stability, failover drill, and release gate evidence.
3. **Persistence:** write a `fault_tolerance_reviews` row.
4. **Check:** query readiness and decode the row into typed Rust values.

## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** each production job kind has a current fault-tolerance review.
- **Validation path:** run the SQL readiness query and Rust decoder tests.
- **Stop if:** a control-plane outage can stop already-approved execution work without a deliberate pause policy.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: a dashboard outage stops workers that could have kept serving
rule: keep critical execution isolated from non-critical control dependencies
tiny example: workers use last-known-good policy when the admin service is down
artifact: fault_tolerance_reviews plus failover drill and release gate evidence
proof: readiness query returns ready only when redundancy, drills, and gates agree
```

## The Core Problem

Agent systems often start as one process:

```text
API receives request
agent plans
worker runs tools
dashboard edits settings
logs print output
```

This is easy to understand. It is also easy to couple.

As the system grows, the parts have different criticality:

| Part | Example | Critical during active job execution? |
| --- | --- | --- |
| Execution plane | workers, leases, Postgres job state, side-effect receipts | yes |
| Control plane | dashboard, billing admin, prompt editor, release UI | usually no |
| Observation plane | dashboards, analytics, long-term reports | useful, but not always critical |

If the dashboard fails, the worker should not forget how to execute an already
approved policy.

If analytics is delayed, the job ledger should still record state transitions.

If a prompt editor is down, production workers should keep using the last
approved prompt version.

## Naive Implementation

A beginner design often looks like this:

```text
let latest_policy = admin_api.fetch_policy(job_kind).await?;
let latest_prompt = prompt_service.fetch_prompt(job_kind).await?;
let result = agent.run(latest_prompt, latest_policy).await?;
```

This code looks reasonable. It wants fresh configuration.

But it adds a hidden dependency:

```text
worker execution now requires admin API availability
```

That means a control-plane failure can stop the execution plane.

## Why The Naive Solution Fails

The failure is not only the outage.

The failure is the dependency direction.

Critical execution depends on a less-critical service at the worst possible
time.

What can go wrong:

- the dashboard is down, so workers cannot fetch policy;
- the prompt registry has a bad deployment, so running jobs stop;
- a canary prompt becomes worse, but all workers read it immediately;
- one worker zone fails, and no other workers can absorb the work;
- a failover path exists in theory, but nobody has practiced it.

The design lesson:

```text
If a part is not required to finish already-approved work, keep it out of the critical execution path.
```

## The Core Intuition

Extreme fault tolerance is built from three plain ideas.

**Isolation** means failures do not spread freely.

For agents, isolate:

- one job kind from another;
- read-only tools from write tools;
- control-plane services from worker execution;
- canary prompt/model versions from production versions;
- tenants from each other;
- failure domains such as workers, zones, regions, and providers.

**Redundancy** means important parts have copies.

For agents, redundant copies include:

- more than one worker for a production job kind;
- more than one eligible worker failure domain;
- durable Postgres records for state and receipts;
- last-known-good prompt, model, and policy versions;
- restore and failover procedures practiced by more than one operator.

**Static stability** means the system can continue from the last known good
state when a control part fails.

For agents, static stability means:

- workers do not need a live prompt editor to use an approved prompt;
- workers do not need a live policy dashboard to enforce the last approved policy;
- production does not automatically chase every new model route;
- risky tools can switch to draft-only or approval-only mode during uncertainty.

## The Precise Model

Use this vocabulary:

- **Control plane:** services that manage, configure, or observe the system.
- **Execution plane:** services that process production work and write durable state.
- **Failure domain:** the boundary inside which one failure may affect multiple parts.
- **Last-known-good state:** the newest approved state that has already passed release and safety checks.
- **Failover drill:** a practiced switch from a failing part to a healthy copy.
- **Progressive delivery:** shipping changes through lower-risk channels before full production.

The key invariant:

```text
The execution plane may depend on durable approved state.
It should not depend on live control-plane availability for already-approved work.
```

This does not mean the system always acts autonomously.

If the safe response is to pause a risky job kind, pause it. Static stability
can mean "continue", "draft only", or "pause". The point is that the behavior is
deliberate and durable.

## Formal Definition

Extreme fault tolerance for an agent job kind is the practiced ability to keep
critical execution serving, deliberately degrade it, or deliberately pause it
when an isolated part fails.

That definition has five parts.

The **state** is the current review for one job kind: control-plane status,
execution-plane status, last-known-good prompt, model and policy versions,
worker redundancy, failure domain, static-stability mode, release gate, failover
drill, owner, and next review date.

The **actor** is the operator, release owner, or reliability owner who is
allowed to change the review evidence or move the job kind between serving,
degraded, draft-only, and paused modes.

The **transition** is a controlled move from fragile to ready, or from serving to
degraded, only after the evidence supports that move.

The **evidence** is the `fault_tolerance_reviews` row, failover drill status,
release gate decision, worker count, last-known-good versions, and readiness
query output.

The **invariant** is simple: a non-critical control-plane failure must not
silently stop already-approved execution work, and a risky uncertainty must
degrade or pause through an explicit durable policy.

## The Postgres Schema

The review table turns fault tolerance from a slogan into inspectable evidence:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql:fault_tolerance_reviews}}
```

Read the schema as a contract:

- every reviewed job kind names its control-plane and execution-plane status;
- every reviewed job kind records last-known-good versions;
- redundancy is a count, not a feeling;
- static stability is explicit;
- release and failover evidence can be joined;
- a next review date prevents stale confidence.

The table does not hide weak evidence. It records it so the readiness query can
show the gap.

## The Readiness Query

The query asks a production question:

```text
Which job kinds are ready, and which ones are fragile?
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/fault_tolerance_readiness.sql}}
```

Notice the order.

The query checks review freshness first. Then it checks control-plane coupling,
static stability, worker redundancy, failover drill evidence, and release gate
evidence.

This is intentional. A stale review is not trustworthy, even if the old row
looked good.

## The Typed Rust Model

The database row is raw because SQL rows cross a boundary.

Inside Rust, the row becomes typed evidence:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/fault_tolerance.rs:fault_tolerance_review}}
```

This model protects meaning:

- `ControlPlaneStatus` is not a random string;
- `StaticStabilityMode` is not a comment;
- `WorkerReplicaCount` cannot be negative;
- `MinimumWorkerReplicaCount` cannot be zero;
- `FaultToleranceReadinessStatus` cannot silently invent a new state.

Raw outside. Typed inside.

## Production Example

Imagine a KYC case-preparation agent.

It can read documents, extract facts, draft a case summary, and prepare a
review checklist. It must not approve the case.

A serious fault-tolerance review might say:

| Field | Value |
| --- | --- |
| job kind | `kyc_case_preparation` |
| control plane | `degraded` |
| execution plane | `serving` |
| static stability | `last_known_good` |
| redundant workers | `3` |
| minimum workers | `2` |
| failure domain | `worker-az-eu-west-1a` |
| delivery channel | `production` |
| failover drill | `passed` |
| release gate | `promote` |

This means:

```text
The control surface is not fully healthy.
But production execution can continue from approved durable state.
There are enough workers.
The failover path has been practiced.
The release gate did not block the current version.
```

A weaker review might say:

```text
control plane: unavailable
execution plane: serving
static stability: normal
redundant workers: 1
minimum workers: 2
```

That is not ready. The worker path is coupled to a failed control plane and has
too little redundancy.

## Failure Modes

Use these as design requirements:

| Failure | Architectural weakness | Design response |
| --- | --- | --- |
| Dashboard down stops workers | execution depends on control plane | use durable last-known-good policy |
| One worker host dies | no redundancy | run multiple workers in isolated domains |
| New prompt breaks all jobs | no progressive delivery | dev -> canary -> production release gate |
| Failover fails during incident | failover was never practiced | schedule failover drills |
| Analytics outage blocks writes | observability path is in critical path | keep audit writes durable and dashboards optional |
| Provider outage causes unsafe tool retries | side effects lack identity | require idempotency and receipts |
| Region fails | no regional continuity plan | define RPO/RTO and optional regional failover path |
| Team deployment causes outage | change reached everyone at once | use release channels and rollback gates |

## Testing Strategy

Test this at three levels.

**Unit tests:** decode raw rows into typed Rust values.

Good test names explain the rule:

```text
row_conversion_rejects_ready_without_required_redundancy
row_conversion_rejects_ready_when_control_plane_unavailable_without_static_stability
```

**SQL tests:** verify the readiness query exposes the right status for weak
evidence.

**Drills:** practice the real failure.

Examples:

- stop one worker and confirm another claims work;
- mark a control service unavailable and confirm last-known-good mode;
- block a release gate and confirm production does not promote;
- run a failover drill and record the result.

## Observability Strategy

Fault tolerance needs more than logs.

Track these signals:

| Signal | Question |
| --- | --- |
| `control_plane_status` | Can operators change or inspect the system? |
| `execution_plane_status` | Can workers still process approved work? |
| redundant worker count | Can one worker fail without stopping the job kind? |
| failover drill status | Has the team practiced the recovery path? |
| release gate decision | Is the current version safe to promote? |
| active/running/dead jobs | Is work moving or stuck? |
| review overdue | Is the evidence stale? |

The important split:

```text
logs describe events
metrics show trends
traces follow one operation
fault-tolerance reviews prove the architecture can absorb failure
```

## Security And Safety Considerations

Static stability is not permission to ignore security.

Last-known-good state must be:

- approved by the release process;
- tied to prompt, model, policy, and schema versions;
- scoped by tenant and job kind when needed;
- revoked when a policy is unsafe;
- auditable after the incident.

If a security boundary is uncertain, the safe static mode may be `draft_only`
or `paused`, not `last_known_good`.

The rule:

```text
Continue only with state that was already safe to use.
```

## Production Judgment

Postgres-first fault tolerance is strong when:

- job volume is moderate;
- state transitions and review evidence must stay visible;
- the team wants minimal infrastructure;
- workers can be scaled horizontally;
- control-plane failures should not stop already-approved execution.

It is weaker when:

- a whole region must fail over automatically with near-zero interruption;
- queue throughput exceeds what the Postgres design can comfortably handle;
- many independent consumers need long replay windows;
- workflows need complex cross-service compensation and visual orchestration.

At that point, you may add a dedicated queue, Kafka, Temporal, or a regional
failover architecture. But first make the state machine and evidence explicit.

## Operational Checklist

Before calling a job kind production-ready, verify:

- the execution plane can finish already-approved work without a live control plane;
- last-known-good prompt, model, and policy versions are durable;
- at least the minimum worker redundancy exists;
- workers are spread across meaningful failure domains;
- release gates block unsafe changes;
- failover drills are scheduled and recorded;
- risky job kinds can move to `draft_only` or `paused`;
- operators can answer which job kinds are fragile right now.

## Exercises

1. Pick a job kind from your system. List its control-plane dependencies and execution-plane dependencies separately.
2. Design a `last_known_good` policy for a support-triage agent. Which actions can continue, which become draft-only, and which pause?
3. Extend the readiness query with a `regional_failover_missing` status. What evidence would you need to store?
4. Write a test that rejects a `ready` status when the release gate decision is `block`.
5. Run a tabletop drill: the prompt registry is down for one hour. What keeps working?

## Summary

Before this chapter, fault tolerance may have sounded like a cloud architecture
luxury.

After this chapter, it should feel like a set of practical questions:

```text
What is isolated?
What is redundant?
What last-known-good state can continue?
What failover path has been practiced?
What release path prevents our own mistakes from reaching everyone?
```

For reliable AI agents, extreme fault tolerance is not about exotic
infrastructure first. It is about disciplined dependency direction, durable
approved state, practiced failover, and evidence that operators can inspect.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
