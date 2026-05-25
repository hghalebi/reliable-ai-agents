# Appendix B. Glossary and Invariant Index

## How to Use This Appendix

Use this appendix when a term feels familiar but the production consequence is
not yet clear.

The book uses each concept in a specific way:

```text
term -> mental model -> failure prevented -> production evidence
```

That last column matters most. In production, a concept is real only when there
is a row, type, event, metric, test, policy, receipt, or runbook that proves it.
The misconception index below is the recovery path when a term sounds familiar
but the system is still being designed around the wrong idea.

## Core Concepts

| Term | Mental model | Failure it prevents | Production evidence |
| --- | --- | --- | --- |
| Agent job | One durable unit of model-powered work. | Treating a model call as the workflow. | A job row exists before the model runs. |
| API admission boundary | The HTTP edge where raw requests become typed admission decisions and then durable jobs when allowed. | Letting request JSON or client retries become hidden execution behavior. | `api.rs` requires `Idempotency-Key`, resolves duplicate keys to existing durable work before pressure checks, converts request data into domain types, enforces admission policy for new work, delays or rejects overloaded work, and enqueues one job only when allowed. |
| Admission control | A typed intake decision over queue pressure, provider pressure, tenant budget, and priority. | Adding work during overload until retries amplify the incident. | `admission_control.rs` turns pressure and budget evidence into accepted, delayed, or rejected decisions; `admit_agent_job.sql` stores admitted work and decision evidence together; `resolve_existing_agent_job.sql` records duplicate intake separately; `admission_decision_events` stores the operator record. |
| Scheduled job | Work with a due time, lease, retry window, and idempotency identity. | Hiding scheduled work in process memory. | `scheduled_jobs` stores `next_run_at`, attempts, lease owner, and idempotency key; `scheduled_job.rs` validates rows into typestate values. |
| Background job | Durable execution record for work that may outlive one process. | Treating a worker loop as the source of truth. | `background_jobs` stores workflow state, retry state, attempts, deadline policy, and failure class; `background_job.rs` validates the row into typestate values. |
| Agent run | Durable execution record for one model-backed workflow attempt. | Losing prompt/model/trace/lifecycle evidence for an agent execution. | `agent_runs` stores lifecycle status, prompt/model version, trace id, deadline, and finish evidence; `agent_run.rs` validates rows into typestate values. |
| Agent step | One durable phase inside an agent run. | Debugging a failed run as one opaque model call. | `agent_steps` stores step kind, index, status, input/output references, terminal reason, and timeline evidence; `agent_step.rs` validates rows into typestate values. |
| Agent runner | The boundary that performs one model-backed step. | Provider behavior leaking into worker logic. | The worker depends on `AgentRunner`, not a provider client. |
| Durable ledger | The database record of current work and history. | Lost work after process restart. | `agent_jobs` stores state; `agent_job_events` stores transitions. |
| At-least-once execution | Work may run again after failure. | False exactly-once assumptions. | Leases expire and jobs can be recovered. |
| Idempotency key | The identity of one logical request. | Duplicate webhooks becoming duplicate work. | Duplicate enqueue returns the existing job. |
| Lease | Temporary ownership of a job. | Two workers mutating the same row. | Completion, retry, and heartbeat require the owning worker. |
| Heartbeat | Renewal of ownership during long work. | Long jobs being recovered while still active. | `locked_until` is extended only by the lease owner. |
| Deadline | The latest acceptable time for a workflow phase. | Confusing "worker still owns this" with "work is still inside the promise." | `deadline_at`, timeout policy, and breached-deadline query are separate from lease fields. |
| Timeout policy | Typed rule for what to do when a deadline is breached. | Treating all slow work as the same failure. | Timeout action distinguishes retry, cancellation, human escalation, and dead-lettering. |
| Cancellation request | Durable intent to stop work before the stop is applied. | Losing who requested a stop, why it was requested, or whether it was applied. | `cancellation_requests` stores requester, source, mode, reason, status, observed job status, and applied evidence. |
| Cancellation | Explicit stop state with reason. | Killing work without evidence. | Cancelled jobs store a reason and release any lease; cancellation requests prove pending, applied, ignored, or expired intent. |
| Retry disposition | A typed decision about whether a failure should run again. | Retrying permanent failures forever. | Transient failures schedule future work; permanent failures stop. |
| Workflow state | The durable lifecycle position of the job. | Guessing whether work is running, waiting, retrying, or terminal. | `background_jobs.workflow_state` and `agent_runs.lifecycle_status` are queryable. |
| Agent handoff | Durable transfer of responsibility from one named agent to another. | Specialist agents delegating work through untracked chat context. | `agent_handoffs` stores source run, source agent, target agent, reason, payload, idempotency key, status, and target job evidence. |
| Retry state | The durable retry position of the job. | Hidden retry loops and unbounded attempts. | `background_jobs.retry_state`, attempts, next retry time, and failure class are stored. |
| Dead letter | Terminal failed work that needs inspection or root-cause change. | Silent infinite failure loops. | Dead jobs keep reason, attempt count, and event history. |
| Failure history | Append-only evidence for each important failed attempt and retry/dead-letter decision. | Losing earlier failures when `last_error` is overwritten. | `failure_history` stores source, class, message, workflow state, retry state, outcome, attempt budget, retry time, and trace context; `failure_history.rs` validates database rows before use. |
| Event ledger | Append-only explanation of how state changed. | Operators seeing only the final status. | Each important transition records an event. |
| Audit event | Durable evidence about who or what made a decision. | Decisions living only in logs, chat, or model text. | `audit_events` records actor type, actor id, action, subject, evidence data, and timestamp. |
| Operation event | Durable evidence about what happened while the system ran. | Runtime symptoms becoming impossible to correlate with jobs and runs. | `operation_events` records job/run context, event type, severity, message, evidence data, and timestamp. |
| Terminal state | A state that normal workers do not mutate again. | Accidental replay of completed or rejected work. | SQL predicates protect `succeeded`, `dead`, `cancelled`, and rejected actions. |
| Side-effect receipt | Durable proof that an external action was attempted or completed. | Replaying a side effect without evidence. | Side-effect worker writes an idempotency key and receipt. |
| Outbox event | Durable publication request committed with domain state. | Losing a notification between database commit and external publish. | `outbox_events` stores event kind, aggregate id, idempotency key, status, lease, attempts, and payload. |
| Temporal workflow execution ledger | Durable execution history for workflow commands, timers, signals, activities, retries, and cancellation. | Rebuilding complex orchestration badly inside one worker loop. | A Temporal workflow id maps to the Postgres job id, agent run id, activity receipts, approval rows, audit events, and trace id. |
| Workflow id mapping | Stable relationship between a product operation and the workflow execution that drives it. | Operators choosing between Temporal history and Postgres rows during an incident. | `agent_runs.workflow_ref`, workflow search attributes, and runbook reconciliation point to the same product operation. |
| Activity receipt | Durable product evidence written by an activity after it crosses a side-effect or tool boundary. | Treating a successful activity execution as enough business evidence. | Tool-call rows, side-effect receipts, approval rows, and audit events store the product result outside workflow history. |
| Kafka event distribution | Optional fanout layer that carries typed product events to independent consumers. | Turning one ambiguous product event into many inconsistent projections. | A Postgres outbox row maps to Kafka topic, partition, offset, schema version, consumer receipt, operation event, and trace id. |
| Event envelope | Typed wrapper around an event payload that carries identity, schema version, aggregate id, tenant scope, trace id, and policy-relevant metadata. | Publishing raw JSON that consumers cannot validate, authorize, replay, or correlate. | Rust event types and schema compatibility tests reject unknown event kinds, missing identity, missing version, or unsafe payload shape. |
| Topic-partition-offset | Kafka coordinate for one record in the event log. | Debugging event delivery from screenshots or consumer logs. | Publisher evidence stores topic, partition, and offset for each published outbox event. |
| Consumer receipt | Durable proof that a named consumer processed or rejected one event. | Replaying a stream and changing the world twice. | Consumer tables store event id, consumer name, idempotency key, status, attempt count, projection result, and error evidence. |
| Event replay rule | Typed rule for when a stored event may be replayed to a consumer. | Reprocessing old events without schema, authorization, idempotency, or side-effect checks. | Replay runbooks check event schema version, consumer receipt, side-effect receipt, authorization boundary, and projection idempotency before replay. |
| Compensation action | A controlled follow-up side effect that corrects a previous side effect. | Treating rollback as invisible or automatic. | `compensation_actions` stores the original receipt, kind, reason, approval, idempotency key, lease, attempts, and terminal result. |
| Policy gate | Deterministic decision point before risky action. | Model text granting itself permission. | Policy version and approval state are stored before execution. |
| Human approval | Operator decision attached to risky work. | Autonomous destructive action. | Actor, timestamp, reason, and proposal are recorded. |
| Human escalation | Durable ownership record for unsafe autonomous progress. | Deadline, security, compatibility, or review failures disappearing into chat. | `human_escalations` stores target, kind, severity, status, owner, and timestamps. |
| Failure drill | Controlled rehearsal of a failure hypothesis with bounded blast radius and named evidence. | Random breakage being mistaken for reliability testing. | `failure_drill.rs` encodes scenario, hypothesis, injection, rollback, event evidence, worker outcomes, and pass/fail report. |
| Control plane | The services that configure, inspect, and manage production work. | A dashboard, prompt editor, or admin API becoming an accidental dependency of critical execution. | `fault_tolerance_reviews` records control-plane health separately from execution-plane health. |
| Execution plane | The workers, leases, durable state, and receipts that process already-approved production work. | Non-critical control failures stopping work that could safely continue. | `fault_tolerance_readiness.sql` checks whether execution can serve from last-known-good state. |
| Static stability | Continuing, drafting, or pausing from last-known-good approved state when a control dependency fails. | Chasing live configuration during an outage or using unapproved state after failure. | `StaticStabilityMode`, last-known-good prompt/model/policy versions, and fault-tolerance row-conversion tests. |
| Evaluation receipt | Evidence that behavior changed acceptably. | Shipping prompt or model drift blindly. | Dataset version, rubric, prompt/model version, and result are stored. |
| Release gate report | Typed decision that combines release evidence. | Promoting because one eval, dashboard, or chat approval looked good. | `release_gate.rs` combines evaluation receipt, SLO evaluation, compatibility report, version consistency, and approval evidence into promote, canary, or block; `release_gate_runs` and `release_gate_status.sql` make the decision durable and reviewable. |
| Launch packet | Typed first-user exposure decision for one job kind. | Launching from a meeting note, chat promise, or private checklist. | `job_kind_launch_packets`, `job_kind_launch_packet_status.sql`, and `launch_packet.rs` bind readiness, release, failure-drill, rollback, restore, known-gap, owner, reviewer, and review-window evidence. |
| Trust boundary | A line where data loses authority. | Prompt injection or cross-tenant action. | Tool calls are typed, authorized, scoped, and audited. |
| Tool call | A requested action across a boundary. | Treating model intent as executed work. | `tool_calls` records validation, execution, failure, rejection, version, idempotency, and terminal evidence; `tool_call.rs` validates rows into typestate values. |
| Typed tool contract | Rust boundary that names tool input, output, and failure type. | Passing untrusted JSON through the application as if it were domain data. | `ToolInput<T>`, `ToolOutput<T>`, and `TypedTool` keep execution behind validated types. |
| Raw model output | Provider text or JSON before trust has been earned. | Treating model output as authority. | Agent-result output is parsed and validated in `agent_output.rs`; tool-call output is parsed, validated, policy-checked, approved, and only then converted into tool input. |
| Authorization event | Durable decision about whether an actor may request a tool permission for a tenant. | Confusing valid tool shape with permission to execute. | `authorization_events` stores actor, tenant, requested tenant, tool, permission, decision, reason, and policy version. |
| Sandbox event | Durable decision about whether a tool may use a network destination, scratch path, or secret mode. | Prompt injection turning tool execution into arbitrary egress, filesystem access, or secret exposure. | `sandbox_events` stores tool, resource request, decision, reason, and policy version. |
| Tool execution trust gate | Composition boundary that requires model parsing, authorization, sandbox allowance, and approval evidence before execution. | Letting a tool execute because one security check passed while another was skipped. | `tool_execution_gate.rs` produces `TrustedToolExecution` only after all trust-boundary evidence matches the same run and tool. |
| Secret reference | Pointer to a secret, not the secret value. | Secret values entering prompts, logs, or model output. | `SecretRef` redacts debug output and names only the external secret lookup key. |
| Credential lifecycle | Durable metadata about a secret reference, owner, rotation due date, verification, exposure, and revocation status. | Old, leaked, or unowned credentials staying active because the team tracks them in informal notes. | `credential_assets` stores secret references and `credential_rotation_review.sql` exposes due, overdue, stale, exposed, and revoked credential families. |
| SLI | Measurement of one reliability property. | Vague health claims. | Query or metric source computes the indicator reproducibly and exposes good/total event counts. |
| SLO | Target for an SLI over a window. | Alerting without a reliability promise. | Objective, time window, typed measurement conversion, and owner are written down. |
| Error budget | Allowed unreliability before behavior changes. | Shipping through repeated failure. | Burn rate affects release, capacity, or provider decisions. |
| Backpressure | Slowing admission or execution when capacity is constrained. | Retry storms and provider quota collapse. | Queue age, provider errors, and worker concurrency shape intake. |
| Provider usage event | Durable record of one model-provider call. | Capacity and cost decisions based on unverifiable dashboards. | `provider_usage_events` stores status, token counts, cost, latency, tenant, and provider route. |
| Tenant budget | Admission-control limit for provider spend. | One tenant or retry storm consuming unbounded money. | Typed budget decision compares proposed cost against current spend and limit. |
| Runbook | Production code for humans. | Improvised incident response. | Commands answer queue, lease, dead-letter, active-run, retry, approval, tool-failure, receipt, and pause questions. |
| Restore drill | Practiced recovery from backup or provider loss. | Discovering recovery gaps during disaster. | `restore_drill_runs` records RPO, RTO, replay safety, counts, and operator signoff. |
| Replay decision | Typed decision about restored work after inspecting receipts and terminal state. | Blindly replaying external side effects after restore. | `ReplayDecision` and `restore_replay_candidates.sql` separate safe resume, receipt reconciliation, quarantine, and no-replay. |
| Newtype | Rust type that gives domain meaning to a primitive. | Swapping two strings with different meaning. | Store and worker APIs expose named types, not raw strings. |
| Typestate | Compile-time lifecycle guard for construction order. | Building an invalid request. | A builder cannot produce an enqueueable job until required fields exist. |
| DTO boundary | Adapter layer between external shape and domain model. | Provider or database weirdness infecting core logic. | Rows and provider responses convert immediately into domain types. |

## Misconception Repair Index

Use this table when the system "looks right" but still fails under realistic
production pressure. The left column is the tempting mental shortcut. The
middle column is the corrected model. The right column is the evidence that
proves the corrected model is actually implemented.

| Misconception | Corrected mental model | Evidence that repairs it |
| --- | --- | --- |
| The model call is the workflow. | The durable job is the workflow; the model performs one step inside it. | A job row exists before the model starts, and the event ledger shows the model step as one transition. |
| Multi-agent means agents chat with each other. | Multi-agent production systems need durable responsibility transfer, not hidden conversation state. | Agent handoffs store source run, source agent, target agent, reason, payload, idempotency key, and acceptance or rejection evidence. |
| Cancellation is just killing a process. | Cancellation is durable intent plus an observed state transition. | Cancellation requests record requester/source/mode/reason, and the job transition records applied or ignored outcome. |
| The API can just call the model. | The API admits intent into durable state; workers execute model behavior later. | `postgres-api-server` enqueues typed jobs, while the worker owns leases, retries, provider calls, and terminal evidence. |
| Retries make side effects safe. | Idempotency keys, outbox events, receipts, and approved compensation actions make retries safe; retry alone only repeats uncertainty. | Duplicate intake returns the existing job, outbox events publish durably, side-effect replay checks a stored receipt, and compensations have their own approval and idempotency evidence. |
| Logs are the audit trail. | Logs are useful observations; state rows and events are the authority. | Operators can reconstruct a job from persisted state and event rows without reading process logs. |
| A prompt can enforce policy. | Prompts can request behavior; policy gates outside the model authorize risky work. | Risky proposals store policy version, approval actor, reason, and side-effect receipt. |
| A passing evaluation means the agent is production-ready. | Evaluation is one release signal; operations, security, recovery, and ownership still need evidence. | Prompt/model changes have eval receipts plus SLOs, runbooks, restore drills, and owners. |
| A backup means disaster recovery is solved. | Recovery is the practiced ability to restore, resume, and avoid duplicate side effects. | A restore drill records RPO, RTO, receipt-aware replay decisions, provider continuity, and resume behavior. |
| Temporal replaces the product ledger. | Temporal may own workflow execution, but Postgres still owns product truth, approvals, receipts, and audit evidence. | A Temporal adoption record maps workflow ids to Postgres rows, activity receipts, approval decisions, audit events, trace ids, and rollback criteria. |
| Kafka is the source of truth. | Kafka may distribute typed facts, but product truth stays in Postgres and consumers keep idempotent receipts. | A Kafka adoption record maps outbox events to topic-partition-offsets, schema versions, consumer receipts, replay rules, and authorization boundaries. |
| Provider errors are just strings to log. | Provider failures are boundary events that drive typed retry or stop decisions. | Adapter tests classify malformed output, timeout, rate limit, policy failure, and terminal provider errors. |
| Raw strings are harmless in early versions. | Raw primitives are acceptable inside adapters, but domain boundaries need named types. | Store and worker APIs use newtypes, enums, smart constructors, and conversion tests. |

## Invariant Index

| Invariant | Where it appears | Evidence to look for |
| --- | --- | --- |
| Work is durable before execution. | Chapters 1, 3, 5, 20 | Enqueue writes a job row before the worker runs the agent. |
| Persisted jobs are decoded before use. | Chapters 3, 11 | `postgres_store.rs` rejects rows with corrupted lease evidence, invalid payload/result shape, missing success results, or stale success errors before constructing `AgentJob`. |
| Scheduled work is claimed before execution. | Chapters 3, 5, 23 | `claim_scheduled_jobs.sql` and `ScheduledJob<ScheduledPending>::claim` move due work into a leased running state before execution. |
| Background work owns workflow and retry state. | Chapters 3, 5, 14, 23 | `background_jobs.workflow_state`, `background_jobs.retry_state`, and `BackgroundJob<State>` separate due-time scheduling from durable operation state, retry budgets, failure classes, and timeout policy. |
| Agent execution is tracked as a lifecycle. | Chapters 3, 15, 20, 23 | `agent_runs.lifecycle_status` and `AgentRun<State>` distinguish planning, running, waiting-for-human, completed, failed, and cancelled runs with trace and version evidence. |
| Agent execution is decomposed into durable steps. | Chapters 3, 5, 15, 23 | `agent_steps.status` and `AgentStep<State>` distinguish pending, running, succeeded, failed, and skipped phases with input/output references and terminal reasons. |
| Tool calls are durable side-effect records. | Chapters 6, 12, 23, 28 | `tool_calls.status` and `ToolCall<State>` distinguish requested, validated, executed, failed, and rejected calls; executed calls require output and timeline evidence, while failed or rejected calls require terminal reasons. |
| Raw HTTP input becomes typed work at admission. | Chapters 18, 20 | API tests reject missing idempotency keys and invalid domain values before enqueue. |
| Overload is decided at admission. | Chapter 22 | Admission-control and API tests prove queue pressure, provider pressure, tenant budget, and priority produce accepted, delayed, or rejected outcomes before work is admitted; the SQL ledger stores admitted work and its decision evidence together. |
| The worker owns only leased work. | Chapters 3, 5, 13 | SQL predicates require `locked_by` and unexpired lease ownership. |
| Deadlines are separate from leases. | Chapters 13, 23 | Timeout policy and breached-deadline queries prove time promises independently from worker ownership. |
| Cancellation intent is durable. | Chapters 13, 23 | Cancellation-request rows separate requested, applied, ignored-terminal, and expired outcomes. |
| Duplicate requests map to one logical job. | Chapters 3, 8, 12, 22 | Idempotency key is unique, duplicate enqueue is visible, and duplicate HTTP intake returns existing work before overload rejection. |
| Publication is durable before external delivery. | Chapters 12, 23 | Outbox rows are committed, claimed with leases, retried, and marked published or failed. |
| Unsafe side effects have compensating paths. | Chapters 12, 16, 23 | Compensation actions reference original receipts, require approval before execution, claim with leases, and end in succeeded, failed, or cancelled evidence. |
| Retry is a typed decision. | Chapters 6, 9, 14 | Failures classify into transient, permanent, malformed-output, or policy-controlled paths. |
| Failure attempts are append-only evidence. | Chapters 3, 14, 23, 24 | `failure_history` records each important failed attempt with source, class, outcome, retry budget, retry time, and trace context instead of relying only on the mutable `last_error` field. |
| Events explain every transition. | Chapters 2, 5, 15, 23 | Event timeline covers enqueue, pick, agent start, result, retry, and terminal state. |
| Decisions and operations are auditable. | Chapters 15, 23, 28 | Audit rows explain actor/action/subject decisions; operation rows explain job/run runtime symptoms. |
| Agent handoffs preserve ownership. | Chapters 20, 20.1, 23 | Handoff rows show source agent, target agent, reason, payload, status, target job, and decision evidence. |
| Risky side effects require policy and approval. | Chapters 10, 16, 28 | Proposal, policy version, approval, and receipt exist before action. |
| Memory is typed production data. | Chapters 27.5, 28 | Memory rows show scope, kind, source, confidence, horizon, retention policy, embedding reference, row conversion, and redacted content policy. |
| Behavior releases require evaluation evidence. | Chapters 25, 27, 30 | Prompt/model changes have dataset, rubric, score, and review evidence. |
| Failure drills prove disruption behavior. | Chapters 17, 24, 34 | Drill plans name hypothesis, blast radius, injection, rollback, required evidence, observed event timeline, and decision before a failure experiment is considered passed. |
| Critical execution is isolated from non-critical control surfaces. | Chapter 29.5 | `fault_tolerance_reviews`, `fault_tolerance_readiness.sql`, and `fault_tolerance.rs` require last-known-good versions, redundancy, static-stability mode, failover drill evidence, and release evidence before a job kind is ready. |
| Release promotion combines multiple signals. | Chapters 25, 27, 30 | Release gate reports combine evaluation, SLO, compatibility, version, and approval evidence. |
| Operations are tied to SLOs and runbooks. | Chapters 15, 21, 23, 24 | Alerts point to SLO burn or invariant failure and a concrete diagnostic path. |
| Provider usage is typed evidence. | Chapter 22 | Token totals, cost, latency, status, tenant, and provider route are recorded and validated. |
| Long-running work remains explainable. | Chapters 19, 25, 29 | Version fields, retention policy, restore drill, replay query, and replay decision type exist. |
| Security boundaries are enforced outside the model. | Chapter 28 | Authorization events, secret references, credential lifecycle review, memory policy, and audit logs exist. |
| Tool resources are sandboxed outside the model. | Chapters 23, 28 | Sandbox events prove allowed and denied egress destinations, scratch paths, and secret-access modes. |
| Tool execution requires all trust-boundary evidence. | Chapter 28 | The tool-execution gate composes parsed model output, matching authorization evidence, matching sandbox evidence, and approval evidence before returning trusted input. |
| Risky work is explainable end to end. | Chapter 20.2 | One scenario links request, retry, approval, receipt, and operator review. |
| Scaling preserves evidence. | Chapter 30.5 | Migration notes, baseline metrics, old/new evidence map, trace correlation, operation events, runbook updates, and rollback criteria show that added infrastructure did not hide the state machine. |
| Temporal adoption preserves product truth. | Chapter 30.6 | Workflow id mapping, activity receipts, approval rows, side-effect receipts, audit events, operation events, trace ids, and reconciliation runbooks agree for one agent run. |
| Kafka adoption preserves event truth and replay safety. | Chapter 30.7 | Outbox rows, event envelopes, schema versions, topic-partition-offsets, consumer receipts, replay rules, authorization boundaries, and trace ids agree for one product event. |

## Summary

A glossary for a production book should not only define words. It should teach
what each concept protects. When a reader can translate a term into a concrete
row, type, event, metric, test, policy, or runbook, the concept has become
operational knowledge.

## Further Reading and Sources



- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: supports the appendix's operational review habits and production-readiness vocabulary.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: keeps the appendix grounded in durable state, transactions, logs, and evidence histories.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: supports the appendix's emphasis on explicit boundaries, named types, and maintainable interfaces.