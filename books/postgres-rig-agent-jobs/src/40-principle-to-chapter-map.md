# Appendix J. Principle To Chapter Map

## How to Use This Map

The design principles are useful only if they lead back to concrete mechanisms.
Use this appendix when a principle sounds right but you want to know where the
book turns it into code, SQL, tests, runbooks, or operating practice.

Read each row as:

```text
principle -> primary chapters -> smallest simulation -> production evidence
```

This is the transfer map for the book. It helps you apply the same reasoning
when the implementation is not the companion Rust crate, not Postgres, or not
Rig.

## Engineering Transformation Map

The book should not be read as a sequence of tool chapters. Read it as a
sequence of engineering transformations:

| Naive shape | Production transformation | Primary proof |
| --- | --- | --- |
| `agent.run("do the task")` | controlled operation with identity, actor, permission, version, and evidence | durable job row, trace id, operation event, and audit event |
| raw user text | trusted domain request | parser, validator, newtype, and rejection test |
| raw model output | validated agent intent | schema validation, semantic validation, policy decision, and optional approval |
| chat loop | durable agent run | `agent_runs`, `agent_steps`, model version, prompt version, and terminal state |
| tool call | permissioned side effect | typed tool input/output, authorization event, idempotency key, and receipt |
| retry | idempotent execution | retry state, attempt budget, failure history, and duplicate-suppression proof |
| memory | governed state | scope, source, confidence, retention, freshness, and retrieval authorization |
| logs | operable evidence | traces, metrics, operation events, audit events, and runbook query |

This map is the bridge from recognition to judgment. The reader first
recognizes the failure, then explains the invariant, then builds the artifact,
then judges whether the pattern fits the current production risk.

## Principle Map

| Principle | Primary chapters | Smallest simulation | Production evidence |
| --- | --- | --- | --- |
| Durable before intelligent | 1, 3, 5, 11 | A process crashes after receiving a request but before the model call. | Job row exists before execution; recovery can find unfinished work. |
| Typed before clever | 4, 4.5, 11, 17 | A raw status string enters the worker path. | Domain newtypes, enums, constructors, row conversion tests, and typed errors reject invalid states. |
| Ownership before concurrency | 3, 5, 13, 18, 20.1 | Two workers try to complete the same running job, while one owned job also breaches its deadline and receives a cancellation request. | Lease fields, handoff state, and SQL predicates require the owning worker or accepting target agent before mutation; timeout policy and cancellation requests prove time promises and stop intent separately from ownership. |
| Boundary before policy | 6, 9, 14, 17, 27.5 | A provider timeout, malformed provider result, and model-generated memory candidate arrive in the same release. | Provider adapter and memory row conversion turn external shapes into typed retry, terminal, validation, or memory-eligibility decisions. |
| Idempotent before retried | 3, 8, 12, 14, 16 | The same webhook arrives twice while the provider is slow. | Idempotency key returns existing work; side-effect receipt prevents duplicate action. |
| Evidence before operations | 5, 13, 15, 20.1, 21, 23, 24 | An operator investigates why one job is still pending, overdue, waiting for cancellation, or stuck between specialist agents. | Event timeline, queue metrics, deadline query, handoff query, cancellation-request query, traces, SLO source, and runbook queries agree. |
| Evaluation before behavior release | 19, 25, 27, 30 | A prompt change improves one fixture but fails a golden incident case. | Evaluation receipt ties golden dataset, rubric, reviewer, prompt/model version, and release decision. |
| Approval is state | 10, 16, 20.1, 20.2, 28 | A model proposes sending an external message after specialist review. | Proposal, handoff, risk, policy version, approver, reason, and side-effect receipt are persisted. |
| Release with old work in mind | 18, 19, 25, 29, 30.5 | A worker deploy happens while old jobs use the previous payload shape, or dispatch moves to a new queue while old jobs still exist. | Versioned rows, compatible migrations, coexistence tests, canary criteria, and rollback path preserve old work. |
| Workflow engine after evidence | 30.5, 30.6 | A job kind has many timers, child workflows, approvals, and cancellation paths. | Temporal workflow history maps back to Postgres product rows, activity receipts, approvals, audit events, and trace ids. |
| Event streaming after outbox | 30.5, 30.7 | Many consumers need replayable events from the same agent run. | Outbox rows, typed event envelopes, Kafka topic-partition-offsets, consumer receipts, and replay drills preserve product truth. |
| Recovery must be practiced | 19, 23, 29, 30 | Restore from backup after some side effects already happened. | RPO, RTO, restore drill, replay rules, receipt handling, and operator signoff are recorded. |

## Dependency Order

The map is ordered for a reason. Later principles assume earlier ones:

```text
durability makes work recoverable
types make state meaningful
ownership makes concurrency safe
boundaries make policy local
idempotency makes retry safe
evidence makes operations possible
evaluation makes behavior release controlled
approval makes risky action accountable
versioning makes long-running work survivable
workflow engines preserve execution only after product evidence is mapped
event streams distribute facts only after the outbox and consumer receipts exist
recovery practice makes the whole claim credible
```

When a production review finds a weakness, move left before moving right. For
example, do not start with SLO dashboards if the system cannot reconstruct a
job from durable state. Do not add retry policy for side effects before
idempotency and receipts exist.

## Transfer Questions

Use these questions when applying the book to another architecture:

| Principle | Transfer question |
| --- | --- |
| Durable before intelligent | Where does work live when the process dies? |
| Typed before clever | Which domain concepts are still raw strings, booleans, or unvalidated JSON? |
| Ownership before concurrency | Who is allowed to mutate running work, and how is that authority checked? |
| Boundary before policy | Where do provider, tool, database, or API shapes become domain decisions? |
| Idempotent before retried | What prevents duplicate intent from becoming duplicate side effect? |
| Evidence before operations | Can an operator answer the incident question without process memory? |
| Evaluation before behavior release | What evidence blocks a bad prompt, model, tool, or policy change? |
| Approval is state | Is human approval durable state or an informal conversation? |
| Release with old work in mind | What happens to jobs created by the previous version? |
| Workflow engine after evidence | Which workflow invariant moves to Temporal, and which product evidence remains in Postgres? |
| Event streaming after outbox | Which typed event is published, which consumers may replay it, and how do receipts prevent duplicate effects? |
| Recovery must be practiced | When was restore and replay last tested end to end? |

The questions should feel uncomfortable when a design is immature. That is the
point. A principle is useful when it exposes a missing mechanism.

## Common Review Failures

| Symptom | Likely missing principle | Next repair |
| --- | --- | --- |
| Queue metrics are green but users report wrong answers. | Evaluation before behavior release | Add behavior fixtures, historical shadow runs, and release receipts. |
| Retried jobs send duplicate messages. | Idempotent before retried | Add idempotency keys and side-effect receipts before widening retries. |
| Operators depend on application logs to explain old jobs. | Evidence before operations | Persist an event timeline and runbook queries. |
| Workers sometimes overwrite each other. | Ownership before concurrency | Make lease ownership a precondition for mutation. |
| A job is still heartbeating but has missed the customer deadline. | Ownership before concurrency | Add deadline policy and a breached-deadline runbook query instead of stretching the lease model. |
| An operator cannot tell whether a stop request was applied. | Evidence before operations | Add durable cancellation requests with applied, ignored-terminal, and expired outcomes. |
| New releases break old pending jobs. | Release with old work in mind | Add versioned payloads and expand-contract migration discipline. |
| A restore test recovers rows but duplicates external action. | Recovery must be practiced | Include receipt-aware replay in the restore drill. |

## Production Contract

This appendix is complete only when each principle can point to:

```text
primary chapters
one mental simulation
one production artifact
one transfer question
one common failure
```

If a principle cannot point to those things, it is not yet teaching production
engineering. It is only a slogan.

## Summary

The front matter gives the principles. The chapters teach the mechanisms. The
appendices test transfer.

Use this map when you feel the book becoming a list of techniques. The deeper
lesson is the dependency chain from durable work to practiced recovery.

## Further Reading and Sources



- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: supports the appendix's operational review habits and production-readiness vocabulary.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: keeps the appendix grounded in durable state, transactions, logs, and evidence histories.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: supports the appendix's emphasis on explicit boundaries, named types, and maintainable interfaces.