# Appendix O. Companion Code Reading Path

## How to Use This Path

This appendix is a guided source-code tour. Use it when the book's concepts are
clear, but you want to see how the companion Rust and SQL project turns them
into executable boundaries.

Read the code in this order:

```text
manifest -> runtime config -> domain types -> compatibility -> HTTP admission
-> SQL ledger -> store boundary -> worker loop -> provider boundary
-> binaries -> validation gate
```

That order mirrors the book's core argument. Do not start with the model
provider. Start with the state and the types that make the provider call safe.

## Reading Contract

For each file, answer four questions:

```text
What invariant does this file protect?
Which chapter teaches the concept?
Which test or check proves it?
What would break if this layer accepted raw, invalid, or ambiguous data?
```

The goal is not to memorize file names. The goal is to connect source code to
runtime behavior: what is persisted, locked, retried, cancelled, observed,
audited, or gated.

## Step 1: Manifest And Feature Boundaries

Start with:

```text
examples/postgres-rig-agent-jobs/Cargo.toml
examples/postgres-rig-agent-jobs/src/config.rs
```

Inspect:

```text
default features:
  deterministic local path

api-server:
  typed Axum admission boundary

postgres-store:
  real Postgres adapter and worker demo

rig-agent:
  real Rig/DeepSeek provider boundary
```

The manifest teaches a production habit: expensive or external boundaries
should be explicit. The default path should prove the state machine without a
database, network, or API key.

Then read `config.rs` before the binaries. It applies the same raw-outside,
typed-inside rule to process configuration:

```text
raw environment -> RuntimeEnv -> PostgresWorkerConfig or PostgresApiServerConfig
raw environment -> RuntimeEnv -> DeepSeekRuntimeConfig
```

`DATABASE_URL` becomes `DatabaseUrl`, `BIND_ADDRESS` becomes
`HttpBindAddress`, and `DEEPSEEK_API_KEY` is checked as present without being
stored or printed. A missing or malformed runtime value should fail before the
system opens Postgres, binds HTTP, or asks Rig to call the provider.

Related chapters: 6, 7, 11, 17.

Validation evidence:

```bash
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features api-server
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features postgres-store
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features rig-agent
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --all-features config
```

## Step 2: Domain Types

Read:

```text
examples/postgres-rig-agent-jobs/src/domain.rs
examples/postgres-rig-agent-jobs/src/admission_control.rs
examples/postgres-rig-agent-jobs/src/scheduled_job.rs
examples/postgres-rig-agent-jobs/src/background_job.rs
examples/postgres-rig-agent-jobs/src/failure_history.rs
examples/postgres-rig-agent-jobs/src/agent_run.rs
examples/postgres-rig-agent-jobs/src/agent_step.rs
examples/postgres-rig-agent-jobs/src/approval.rs
examples/postgres-rig-agent-jobs/src/agent_output.rs
examples/postgres-rig-agent-jobs/src/tool_call.rs
examples/postgres-rig-agent-jobs/src/handoff.rs
examples/postgres-rig-agent-jobs/src/timeouts.rs
examples/postgres-rig-agent-jobs/src/cancellation.rs
examples/postgres-rig-agent-jobs/src/compatibility.rs
examples/postgres-rig-agent-jobs/src/escalation.rs
examples/postgres-rig-agent-jobs/src/failure_drill.rs
examples/postgres-rig-agent-jobs/src/agent_memory.rs
examples/postgres-rig-agent-jobs/src/cost_accounting.rs
examples/postgres-rig-agent-jobs/src/credential_lifecycle.rs
examples/postgres-rig-agent-jobs/src/data_protection.rs
examples/postgres-rig-agent-jobs/src/slo.rs
examples/postgres-rig-agent-jobs/src/security.rs
examples/postgres-rig-agent-jobs/src/sandbox.rs
examples/postgres-rig-agent-jobs/src/audit.rs
examples/postgres-rig-agent-jobs/src/outbox.rs
examples/postgres-rig-agent-jobs/src/compensation.rs
examples/postgres-rig-agent-jobs/src/release_gate.rs
examples/postgres-rig-agent-jobs/src/recovery.rs
examples/postgres-rig-agent-jobs/src/job_kind_readiness.rs
examples/postgres-rig-agent-jobs/src/fault_tolerance.rs
```

This is the most important file to understand before reading the worker. It
names the concepts the rest of the system is allowed to use:

```text
JobKind
JobState
WorkerId
IdempotencyKey
AdmissionPolicy
AdmissionRequestId
AdmissionControlInput
AdmissionSubject
AdmissionSignals
AdmissionDecision
AdmissionDecisionEvent
JobPriority
ProviderQuotaPressure
QueuePressure
ScheduledJobId
ScheduledTaskName
ScheduledJobPayload
ScheduledJob
ScheduledPending
ScheduledRunning
ScheduledFailureTransition
BackgroundJob
BackgroundJobId
WorkflowState
RetryState
BackgroundQueued
BackgroundExecutingAgent
BackgroundWaitingForHuman
BackgroundWaitingForRetry
FailureClass
FailureHistoryRecord
FailureHistoryId
FailureSource
FailureOutcome
AgentRun
AgentRunLifecycleStatus
AgentRunPlanning
AgentRunRunning
AgentRunWaitingForHuman
AgentModelVersion
AgentStep
AgentStepStatus
AgentStepKind
AgentStepPending
AgentStepRunning
AgentStepSucceeded
AgentStepIndex
AgentStepRef
ToolCall
ToolCallStatus
ToolCallRequested
ToolCallValidated
ToolCallExecuted
ToolCallInput
ToolCallOutput
AgentInstruction
RetryDisposition
PayloadSchemaVersion
PromptVersion
ModelRoute
ToolVersion
PolicyVersion
WorkerBuildId
ApprovalRequest
ApprovalStatus
ApprovalActor
ApprovalReason
RawAgentOutput
ParsedAgentOutput
ValidatedAgentOutput
AgentName
HandoffStatus
HandoffRecord
RequestedHandoff
AcceptedHandoff
TimeoutPolicy
TimeoutAction
RunningJobDeadline
ExecutionDeadline
CompatibilityPolicyName
WorkerCompatibilityPolicy
SupportedPayloadSchemaRange
CompatibilityDecision
CompatibilityQuarantineReason
EscalationKind
EscalationSeverity
EscalationStatus
HumanEscalationRecord
FailureDrillPlan
FailureDrillScenario
EvidenceRequirement
ProviderTimeoutThenRetryDrill
WorkerCrashAfterLeaseDrill
MemoryScope
MemoryKind
MemorySource
MemoryHorizon
MemoryLifecyclePolicy
MemoryConfidence
RetentionPolicy
TenantKey
ProviderName
ProviderCallStatus
PromptTokenCount
CompletionTokenCount
TotalTokenCount
CostMicrosUsd
TenantBudget
SloName
SliName
SloTargetBasisPoints
ObservedGoodEventCount
ObservedTotalEventCount
SloDecision
ActorId
ToolPermission
AuthorizationDecisionKind
AuthorizationPolicy
SecretRef
ToolSandboxPolicy
SandboxDecisionEvent
EgressDestination
SandboxPath
SecretAccessRequest
AuditActorType
AuditEventRecord
OperationSeverity
OperationEventRecord
TraceId
SpanId
TraceContext
OutboxEventKind
OutboxAggregateId
OutboxPayload
OutboxStatus
OutboxEventRecord
CompensationActionStatus
CompensationActionRecord
CompensationEnvelope
RequestedCompensationAction
ApprovedCompensationAction
ExecutingCompensationAction
ReleaseGate
ReleaseGateReport
ReleaseGateDecision
ReleaseBlocker
RestoreDrillName
RecoveryPointObjectiveSeconds
RecoveryTimeObjectiveSeconds
ReplayCandidate
ReplayDecision
SideEffectEvidence
MaturityLevel
JobRiskClass
ReadinessEvidence
JobKindReadinessStatus
JobKindReadinessReview
```

The production idea is simple:

```text
the worker can protect only the concepts the domain model can name
```

When a constructor rejects empty text or an enum replaces a raw status string,
the code is not becoming fancy. It is moving a production invariant to the
boundary. `admission_control.rs` applies that rule before enqueue: queue
pressure, provider quota pressure, tenant budget, and job priority produce a
typed accepted, delayed, or rejected intake decision. `api.rs` then applies
that decision at the HTTP boundary: accepted work is enqueued now, delayed work
is enqueued with a later `run_at`, and rejected work records an admission
decision without creating a job.
`scheduled_job.rs` applies that rule to the generic scheduling
table: raw rows become typed pending, running, succeeded, failed, dead, or
cancelled jobs, and only pending jobs can be claimed into a running lease.
`background_job.rs` applies it to the durable operation behind the schedule:
workflow state and retry state are validated together, active states require
execution deadlines, waiting retries require next-run time and failure
evidence, and terminal states cannot pretend they are still retryable.
`failure_history.rs` keeps failure review from depending on only the latest
error field: raw rows become typed append-only records with source, class,
outcome, attempt budget, next retry time, and trace context.
`agent_run.rs` applies that rule to execution tracking: raw `agent_runs` rows
become planning, running, waiting-for-human, completed, failed, or cancelled
runs, and malformed trace, deadline, owner, or `finished_at` evidence is
rejected before the run can be used by tool, approval, audit, or operation
logic.
`agent_step.rs` applies the same boundary one level deeper: raw `agent_steps`
rows become pending, running, succeeded, failed, or skipped phases, and the
decoder rejects negative indexes, missing output evidence, missing terminal
reasons, active rows with terminal evidence, and invalid timelines.
`agent_output.rs` applies that rule to the provider boundary:
DeepSeek output starts as raw text, is parsed through a strict DTO, rejects
unknown fields, validates `approval_requirement`, and only then becomes
`AgentResult`. `agent_memory.rs` applies the same idea to short-term and long-term
memory: memory records are scoped, sourced, assigned a horizon, retained,
confidence-scored, and converted from raw database JSON before use.
`approval.rs` applies typestate to human
approval so only requested approvals can become approved, rejected, expired, or
cancelled. `handoff.rs` applies the same idea to specialist-agent delegation:
a handoff names the source run, source agent, target agent, reason, payload,
idempotency key, and acceptance or rejection evidence. `timeouts.rs` separates
lease ownership from execution deadlines: a running job can still be owned while
its timeout policy says retry, cancel, escalate, or dead-letter.
`cancellation.rs` separates stop intent from applied job state, so operators can
distinguish requested, applied, ignored-terminal, and expired cancellation.
`compatibility.rs` separates "a row exists" from "this worker can safely parse
and execute that row"; jobs outside the supported payload-schema range become
typed compatibility quarantines instead of accidental execution.
`escalation.rs` separates approval from ownership escalation: when autonomous
progress is unsafe, the record names the target, kind, severity, reason, owner,
and acknowledgement or resolution timeline.
`failure_drill.rs` turns simulation and chaos-testing language into executable
evidence: a drill names its hypothesis, blast radius, failure injection,
rollback action, required events, observed worker outcomes, and final decision.
`cost_accounting.rs` turns provider usage, token counts, latency,
and budget decisions into typed operational evidence. `slo.rs` turns SLI query
rows into typed SLO measurements, validates measurement windows and basis-point
targets, and evaluates whether the error budget is still available.
`logging.rs` turns `RUST_LOG` and `LOG_FORMAT` into a typed runtime tracing
configuration and installs the subscriber used by the API, worker, local demo,
and DeepSeek demo binaries.
`sql.rs` is the checked SQL registry. It embeds every migration, state
transition, SLI query, runbook query, and operator control file through
`include_str!`, then exposes `SQL_ARTIFACTS` so tests and scripts can catch a
new SQL file that is not visible to Rust and the book.
`security.rs` keeps
actor identity, tenant scope, tool permission, authorization decisions, and
secret references outside model text. `sandbox.rs` turns network egress,
scratch filesystem access, and secret exposure into typed allow/deny decisions
so the model cannot choose arbitrary URLs, host paths, or prompt-visible
secrets. `tenant_isolation.rs` decodes `tenant_boundary_review.sql` so
operators can inspect cross-tenant attempts, denied requests, and boundary
breaches without trusting raw dashboard strings. `audit.rs` separates durable decision evidence from runtime operation
evidence: audit rows name actors, actions, and subjects, while operation rows
name job/run context, trace context, severity, and messages.

`outbox.rs` applies the same discipline to publication: committing domain
state and publishing an external event are separated by typed durable state,
lease ownership, attempts, and idempotency keys. `temporal_adoption.rs` and
`kafka_adoption.rs` apply that same discipline to optional scaling paths
without making either platform part of the default stack. The Temporal adapter
names workflow references, activity receipts, product evidence, and
reconciliation packets. The Kafka adapter names topic-partition-offset records,
publish receipts, consumer groups, idempotency keys, and consumer receipts.
Both adapters make one point concrete: optional infrastructure may move
execution or distribution work, but it must not erase product evidence.
`compensation.rs` applies the same discipline to corrective side effects:
compensation starts from an
original receipt, requires approval evidence, claims execution with a lease,
and ends with durable success, failure, or cancellation. `release_gate.rs`
applies the same discipline to promotion: evaluation receipts, SLO decisions,
worker compatibility, version consistency, and approval evidence become one
typed promote/canary/block decision. `recovery.rs` applies the
same discipline to disaster recovery: a restored
database row becomes a typed replay candidate, and replay decisions distinguish
safe resume, receipt reconciliation, missing-receipt quarantine, and terminal
no-replay. `job_kind_readiness.rs` applies the same discipline to maturity
claims: a raw database row with `demo`, `prototype`, `production`, or
`regulated_high_risk` labels becomes a typed `JobKindReadinessReview`, and
impossible claims such as regulated work targeting ordinary production evidence
are rejected at the row boundary.
`fault_tolerance.rs` applies the same boundary discipline to isolation,
redundancy, and static stability: raw control-plane, execution-plane,
last-known-good, drill, release, and worker-count evidence becomes a typed
`FaultToleranceReadinessReview` before operators trust a readiness claim.

Related chapters: 4, 4.5, 9, 12, 16, 17, 19, 22, 28, 29, 29.5, 30.6, 30.7, 34.

Validation evidence:

```text
domain unit tests reject invalid instructions and version fields
retry policy tests prove bounded backoff
job default tests prove replay/version metadata exists
scheduled-job tests reject malformed rows, require running lease evidence,
reject leased non-running rows, and prove claim, completion, retry, and dead
transitions
background-job tests reject incompatible workflow and retry states, missing
deadlines, missing retry times, missing failure evidence, exhausted retry
budgets, and prove queued, leased, executing, waiting-for-human, retry, and
terminal transitions
failure-history tests reject unknown sources, unknown outcomes, invalid
attempt budgets, missing or invalid retry times, premature dead letters,
span-without-trace rows, bad timelines, and incompatible workflow/retry states
agent-run tests reject orphan runs, invalid trace ids, invalid deadlines,
terminal rows without finish evidence, active rows with finish evidence, and
prove planning, running, and waiting-for-human transitions
approval tests reject missing decision evidence and unknown statuses
handoff tests reject same-agent loops, non-object payloads, unknown statuses,
accepted rows without target jobs, and terminal rows without decision evidence
timeout tests reject unknown timeout actions, terminal observed states, invalid
deadlines, negative attempts, and prove retry exhaustion changes the timeout
action to dead-letter
memory tests reject unknown memory kinds, unknown memory horizons,
horizon/retention mismatches, invalid confidence, and malformed memory content
cost tests reject token-total mismatch, negative cost, and unknown provider
statuses
SLO tests reject invalid targets, invalid windows, negative counts, and
good-event counts greater than total-event counts
security tests deny cross-tenant requests, require approval for risky granted
tools, reject malformed authorization rows, and redact secret references
sandbox tests deny non-allowlisted destinations, read-only scratch writes,
model-visible secrets, URL/wildcard destinations, path traversal, and malformed
sandbox event rows
audit tests reject unknown actor types, non-object evidence, empty actions,
missing operation subjects, unknown severities, malformed trace ids, malformed
span ids, and negative event ids
failure-drill tests prove provider-timeout retry/success evidence,
worker-crash expired-lease recovery evidence, non-empty evidence requirements,
missing-event rejection, and recovery-after-lease-expiry validation
outbox tests reject non-object payloads, missing publication lease evidence,
unknown statuses, invalid attempts, and prove typed publication transitions
Temporal-adoption tests reject empty workflow references, require activity
receipts before reconciliation, and keep workflow evidence tied to product
audit and operation evidence
Kafka-adoption tests reject empty topics and negative offsets, then prove
consumer receipts reuse the exact published topic-partition-offset and outbox
event identity
compensation tests reject non-object payloads, missing approval evidence,
missing execution leases, failed rows without errors, unknown statuses,
negative attempts, exhausted retries, and prove typed compensation transitions
release-gate tests block failed evals, exhausted budgets, incompatible work,
version mismatch, and missing high-risk approval, while no-traffic evidence
allows only canary
recovery tests prove safe resume, receipt reconciliation, missing-receipt
quarantine, terminal no-replay, and RTO failure behavior
readiness tests reject unknown maturity labels, negative evidence counts,
impossible evidence totals, invalid review windows, too-low regulated targets,
and inconsistent readiness statuses
compatibility tests prove supported jobs can run, too-old and too-new schemas
quarantine with typed reasons, and invalid compatibility ranges are rejected
escalation tests reject missing targets, unknown escalation kinds, missing
acknowledgement or resolution evidence, and invalid owner timelines
agent-output tests reject malformed JSON, unknown approval values, empty
summaries, and unexpected fields
```

## Step 2.5: Typed Agent And Tool Outputs

Read:

```text
examples/postgres-rig-agent-jobs/src/agent_output.rs
examples/postgres-rig-agent-jobs/src/tool_contract.rs
examples/postgres-rig-agent-jobs/src/tool_call.rs
```

`agent_output.rs` shows the raw-outside/typed-inside rule at the provider
result boundary:

```text
RawAgentOutput
  -> ParsedAgentOutput
  -> ValidatedAgentOutput
  -> AgentResult
```

The model may return JSON-shaped text. It does not get to become a persisted
summary, next action, or approval requirement until typed code validates the
shape and values.

`tool_contract.rs` shows the same raw-outside/typed-inside rule at the agent action
boundary:

```text
RawModelOutput
  -> ParsedToolRequest
  -> ValidatedToolRequest
  -> PolicyCheckedToolRequest
  -> ApprovedToolRequest
  -> ToolInput
  -> ToolOutput
```

The model may propose a tool request. It does not get to execute a side effect
just because it produced JSON that looks plausible.

`tool_call.rs` shows how the durable `tool_calls` row becomes production
evidence. The model proposal, validation state, execution result, failure
reason, rejection reason, idempotency key, and timestamps are decoded into a
typed lifecycle instead of being passed around as raw strings and JSON.

Related chapters: 6, 16, 28.

Validation evidence:

```text
agent-output tests reject malformed JSON, unknown approval requirements, empty
summaries, and unexpected fields
tool-contract tests reject malformed JSON, unknown tool names, rejected
approvals, and prove an approved typed request can execute the dry-run tool
tool-call tests prove executed calls require output and timeline evidence,
failed or rejected calls require terminal reasons, active calls reject terminal
evidence, and tool payload debug output is redacted
```

## Step 2.75: HTTP Admission Boundary

Read:

```text
examples/postgres-rig-agent-jobs/src/api.rs
```

This file shows the same raw-outside/typed-inside rule at the HTTP boundary:

```text
HTTP JSON + Idempotency-Key header
  -> CreateAgentJobRequest
  -> CreateAgentJobCommand
  -> existing idempotency lookup
  -> admission decision for new work
  -> AgentJob
  -> store.admit_agent_job(...)
```

The API service owns admission, not intelligence. It rejects missing
idempotency keys and invalid domain values before a row is enqueued. It also
returns an existing job for a duplicate idempotency key before queue pressure is
allowed to reject the request. It does not call the model directly, and it does
not let request JSON become worker state.

The same file also owns the API runtime surface:

```text
/healthz:
  liveness without dependency checks

/readyz:
  dependency readiness through AgentJobObservabilityStore

/metrics:
  queue snapshot exposed from typed QueueMetrics
```

The important design detail is that readiness and metrics reuse domain metrics
instead of inventing a parallel HTTP-only health model.

Related chapters: 3, 18, 20.

Validation evidence:

```text
API tests reject missing idempotency keys and invalid domain values, prove
successful admission, prove duplicate idempotency returns the same job, prove an
overloaded duplicate returns existing work before rejection, prove liveness is
dependency-free, and prove readiness and metrics expose queue health
```

## Step 3: SQL Ledger

Read the schema before the queries:

```text
examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql
examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql
```

Then read the transition queries:

```text
examples/postgres-rig-agent-jobs/sql/enqueue_agent_job.sql
examples/postgres-rig-agent-jobs/sql/admit_agent_job.sql
examples/postgres-rig-agent-jobs/sql/resolve_existing_agent_job.sql
examples/postgres-rig-agent-jobs/sql/record_admission_decision.sql
examples/postgres-rig-agent-jobs/sql/claim_scheduled_jobs.sql
examples/postgres-rig-agent-jobs/sql/complete_scheduled_job.sql
examples/postgres-rig-agent-jobs/sql/fail_or_retry_scheduled_job.sql
examples/postgres-rig-agent-jobs/sql/pick_due_job.sql
examples/postgres-rig-agent-jobs/sql/extend_lease.sql
examples/postgres-rig-agent-jobs/sql/mark_succeeded.sql
examples/postgres-rig-agent-jobs/sql/retry_or_dead.sql
examples/postgres-rig-agent-jobs/sql/recover_expired_jobs.sql
examples/postgres-rig-agent-jobs/sql/mark_cancelled.sql
examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql
examples/postgres-rig-agent-jobs/sql/claim_outbox_events.sql
examples/postgres-rig-agent-jobs/sql/mark_outbox_event_published.sql
examples/postgres-rig-agent-jobs/sql/mark_outbox_event_failed.sql
examples/postgres-rig-agent-jobs/sql/approve_compensation_action.sql
examples/postgres-rig-agent-jobs/sql/claim_compensation_actions.sql
examples/postgres-rig-agent-jobs/sql/mark_compensation_succeeded.sql
examples/postgres-rig-agent-jobs/sql/mark_compensation_failed.sql
```

Read each query as a state transition:

```text
precondition:
  which rows may change?

actor:
  which worker or caller is allowed to change them?

transition:
  which state changes?

evidence:
  which event or field proves the change?

invariant:
  what cannot become true accidentally?
```

The key production habit is to look for predicates. A query that updates a
running job without checking `locked_by` is not a small bug. It breaks the
ownership model.

For generic scheduled work, read the claim and failure queries as the minimal
Postgres scheduler:

```text
due scheduled job -> claim with SKIP LOCKED, owner, lease, and attempt budget
running scheduled job + owning worker -> complete
running scheduled job + owning worker + failure -> retry later or dead-letter
```

Related chapters: 3, 5, 12, 13, 14.

Validation evidence:

```text
SQL tests assert idempotency, scheduled-job claims, row locking, ownership
predicates, recovery, terminal-state rules, and retry/dead-letter separation;
the live Postgres gate also applies the production tracking surface.
```

## Step 4: Store Boundary

Read the trait and the in-memory implementation:

```text
examples/postgres-rig-agent-jobs/src/lib.rs
examples/postgres-rig-agent-jobs/src/memory_store.rs
```

Then read the real adapter:

```text
examples/postgres-rig-agent-jobs/src/postgres_store.rs
```

The in-memory store proves behavior quickly. The Postgres store proves that
database rows can cross into domain values safely.

The boundary rule is:

```text
database DTO -> validated domain value -> worker logic
```

Do not invert it. If database rows leak directly into worker decisions, the
domain model stops protecting the system.

Related chapters: 7, 11, 17.

Validation evidence:

```text
memory-store tests prove behavior without infrastructure
postgres-store tests reject negative counts, negative attempts, invalid schema
versions, and invalid nested payloads
```

## Step 5: Worker Loop

Read:

```text
examples/postgres-rig-agent-jobs/src/worker.rs
```

This is where the book's state machine becomes executable:

```text
recover expired work
pick due work
record start
call agent runner
mark success
schedule retry
dead-letter permanent or exhausted failure
cancel when requested
```

The worker should feel almost boring by this point. If the domain model and SQL
ledger are strong, the worker is mostly a sequence of explicit transitions.
It also emits structured `tracing` events at the same lifecycle boundaries, so
the runtime signal and the durable event ledger describe the same job.

Related chapters: 5, 13, 14, 17, 18.

Validation evidence:

```text
worker and store tests cover success, transient retry, permanent failure,
exhausted attempts, rejected non-owner success/retry, expired lease recovery,
terminal behavior, and stable observability labels
```

## Step 6: Agent And Provider Boundary

Read the deterministic runner first:

```text
examples/postgres-rig-agent-jobs/src/agent.rs
```

Then read the real Rig-backed path:

```text
examples/postgres-rig-agent-jobs/src/rig_runner.rs
examples/postgres-rig-agent-jobs/src/bin/deepseek_agent_demo.rs
```

Then read the trust boundary that turns a model-proposed tool call into an
execution-safe value:

```text
examples/postgres-rig-agent-jobs/src/tool_contract.rs
examples/postgres-rig-agent-jobs/src/tool_call.rs
examples/postgres-rig-agent-jobs/src/security.rs
examples/postgres-rig-agent-jobs/src/sandbox.rs
examples/postgres-rig-agent-jobs/src/tool_execution_gate.rs
```

The deterministic runner exists so the system can prove reliability without a
live model. The Rig runner exists so the same boundary can call DeepSeek when
the `rig-agent` feature and `DEEPSEEK_API_KEY` are available.

The important split is:

```text
provider response -> AgentResult or typed failure
raw model tool output + auth + sandbox + approval -> TrustedToolExecution
```

The provider should not decide retry policy, approval, or side-effect safety.
It should cross the provider boundary as a domain outcome. The Rig runner now
asks the provider for strict JSON, then passes the provider text through
`RawAgentOutput` before the worker can persist an `AgentResult`.

The tool path follows the same rule. `tool_execution_gate.rs` will not produce
a `TrustedToolExecution<PauseJobKindRequest>` unless the model output parses,
the authorization event belongs to the same run and permission, the sandbox
event allows the requested resources, and human approval exists when the
permission is risky.

Related chapters: 6, 17, 27, 28.

Validation evidence:

```text
default tests do not require network access
rig-agent feature check compiles the real provider boundary
agent-output tests prove malformed provider text is permanent invalid output,
not domain state
tool-execution gate tests prove tool calls require parser, authorization,
sandbox, and approval evidence before execution
tool-contract tests prove extra model fields such as approved or skip_human_approval
fail before typed tool input construction
live DeepSeek run is optional and credential-gated
```

## Step 7: Behavior Evaluation Gate

Read:

```text
examples/postgres-rig-agent-jobs/src/evaluation.rs
examples/postgres-rig-agent-jobs/src/release_gate.rs
examples/postgres-rig-agent-jobs/sql/release_gate_status.sql
```

This file turns behavior evaluation into an executable release gate:

```text
dataset version
  -> golden dataset
  -> typed evaluation cases
  -> evaluator version
  -> evaluation receipt
  -> promote or block
```

The important production habit is to keep behavior evidence tied to the exact
prompt, model, tool, policy, and worker versions being promoted.

Then read `release_gate.rs`. It combines the behavior receipt with operational
evidence:

```text
EvaluationReceipt
  + SloEvaluation
  + JobCompatibilityReport
  + ReleaseApprovalEvidence
  -> ReleaseGateReport
```

This keeps release promotion from becoming a loose checklist. A passed eval is
not enough if the SLO budget is exhausted, the worker cannot process the row,
the versions do not match, or a high-risk release lacks approval.

Then read `release_gate_status.sql`. It shows the durable side of the same
decision:

```text
release candidate
  -> versions
  -> evaluation, SLO, compatibility, migration, and approval evidence
  -> blockers, canary percentage, rollback plan, and signoff
```

Related chapters: 17, 25, 27, 30.

Validation evidence:

```text
evaluation tests prove passing cases promote, golden datasets produce versioned
release evidence, missing required behavior blocks promotion, provider failures
block promotion, and empty datasets are rejected
release-gate tests prove promotion, canary-only, and block decisions are based
on typed evidence rather than one dashboard or one eval result
release-gate status SQL proves operators can inspect blocked, canary-only, and
recently promoted release candidates after the gate runs
```

## Step 8: Operator Queries

Read:

```text
examples/postgres-rig-agent-jobs/sql/queue_metrics.sql
examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql
examples/postgres-rig-agent-jobs/sql/oldest_pending_job.sql
examples/postgres-rig-agent-jobs/sql/expired_leases.sql
examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql
examples/postgres-rig-agent-jobs/sql/dead_jobs_by_reason.sql
examples/postgres-rig-agent-jobs/sql/job_event_timeline.sql
examples/postgres-rig-agent-jobs/sql/failure_history_by_job.sql
examples/postgres-rig-agent-jobs/sql/running_agent_runs.sql
examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql
examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql
examples/postgres-rig-agent-jobs/sql/scheduled_retries.sql
examples/postgres-rig-agent-jobs/sql/waiting_human_approvals.sql
examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql
examples/postgres-rig-agent-jobs/sql/failed_tool_calls.sql
examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql
examples/postgres-rig-agent-jobs/sql/evaluation_receipts_by_version.sql
examples/postgres-rig-agent-jobs/sql/version_compatibility_risks.sql
examples/postgres-rig-agent-jobs/sql/schema_migration_status.sql
examples/postgres-rig-agent-jobs/sql/failure_drill_status.sql
examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql
examples/postgres-rig-agent-jobs/sql/job_kind_readiness_review.sql
examples/postgres-rig-agent-jobs/sql/fault_tolerance_readiness.sql
examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql
examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql
examples/postgres-rig-agent-jobs/sql/sli_terminal_jobs_not_dead.sql
examples/postgres-rig-agent-jobs/sql/denied_authorization_events.sql
examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql
examples/postgres-rig-agent-jobs/sql/audit_events_by_run.sql
examples/postgres-rig-agent-jobs/sql/operation_events_by_job.sql
examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql
examples/postgres-rig-agent-jobs/sql/storage_pressure_by_table.sql
examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql
examples/postgres-rig-agent-jobs/sql/retention_review_by_surface.sql
examples/postgres-rig-agent-jobs/sql/data_protection_review.sql
examples/postgres-rig-agent-jobs/sql/outbox_backlog.sql
examples/postgres-rig-agent-jobs/sql/compensation_backlog.sql
examples/postgres-rig-agent-jobs/sql/pause_job_kind.sql
examples/postgres-rig-agent-jobs/sql/resume_job_kind.sql
```

These files are not reporting extras. They are the operational face of the
state machine.

Each query should answer one operator question:

```text
Is work flowing?
Which kind is stuck?
Which leases expired?
Which running jobs breached their timeout policy?
Why are jobs dead?
What happened to this one job?
Which failed attempts, retry outcomes, and trace ids explain this job?
Which agent runs are active?
Which target agents have pending handoffs?
Which cancellation requests are pending, who requested them, and what mode applies?
Which retries are scheduled?
Which approvals are waiting?
Which human escalation is open or acknowledged?
Which tool calls failed?
Which side effects already have receipts?
Which dataset, evaluator, prompt, model, tool, and policy versions justify
behavior release?
Which pending or running jobs require a different worker version before
execution?
Which expand, backfill, or contract migration phase is still open, blocked,
failed, or recently passed?
Which failure drill is planned, running, failed, aborted, or missing evidence?
Which outbox events are waiting or stuck under expired publication leases?
Which compensation actions are waiting, leased, or stuck under expired leases?
Which job kinds are burning provider tokens, cost, latency, or rate limits?
Which job kinds are demo, prototype, production, or regulated/high-risk, and
which evidence gap blocks the next target?
Which scoped memory records are eligible without exposing raw content?
Which SLO measurement rows show budget exhaustion or missing traffic?
Which authorization decisions were denied or escalated?
Which sandbox policies are denying egress, filesystem, or secret-access requests?
Which actors and actions produced audit evidence for this run?
Which operation events explain this job's runtime behavior?
Which restored jobs are safe to resume, reconcile, quarantine, or leave terminal?
Which tables are growing, dead-row-heavy, or missing vacuum/analyze evidence?
Which credential kinds are due, overdue, exposed, stale, or recently revoked?
Which evidence surfaces have old rows that need retention policy review?
Which evidence surfaces have overdue redaction, erasure, export, or retention-review work?
Is a job kind paused, and why?
Which job kinds are active, deprecation candidates, retirement candidates, or retirement blocked?
```

Related chapters: 15, 21, 22, 23, 24, 26, 29.

Validation evidence:

```text
runbook SQL tests assert diagnostic and control coverage
implementation evidence map links each query to an operator proof
```

## Step 9: Binaries

Read:

```text
examples/postgres-rig-agent-jobs/src/main.rs
examples/postgres-rig-agent-jobs/src/bin/postgres_api_server.rs
examples/postgres-rig-agent-jobs/src/bin/postgres_worker_demo.rs
examples/postgres-rig-agent-jobs/src/bin/deepseek_agent_demo.rs
```

The binaries show how the same system model is entered from different runtime
surfaces:

```text
local deterministic run:
  no external services

Postgres API server:
  real typed HTTP admission boundary

Postgres worker demo:
  real durable store with bounded loop execution, heartbeat supervision, and typed drain control

DeepSeek demo:
  real provider boundary
```

The binaries should stay thin. If the API binary starts owning business rules,
the architecture is drifting away from the admission and domain layers. If the
worker binary starts parsing HTTP-shaped data, the raw boundary has leaked too
far inward.

The binaries should also keep typed runtime errors. A runnable example may be
small, but configuration failure, domain validation failure, store failure,
worker failure, provider failure, listener bind failure, and JSON serialization
failure should still be named error cases. That is why the companion crate does
not use a direct `anyhow` dependency for its runtime surfaces.

Related chapters: 7, 11, 18.

Validation evidence:

```text
readiness gate builds the feature-specific binaries
production hygiene rejects direct anyhow usage in companion runtime code
scripts/smoke-postgres-api.sh checks the live Postgres API surface
scripts/smoke-local-postgres.sh checks the Postgres worker, API, runbook SQL, and audited pause/resume controls against an ephemeral local database
scripts/smoke-deepseek-agent.sh checks the live Rig provider boundary when credentials are supplied
mdBook includes the commands needed to run each path
```

## Step 10: Validation Gate

End with the project-level checks:

```text
scripts/check-rust-production-hygiene.py
scripts/smoke-postgres-api.sh
scripts/smoke-local-postgres.sh
scripts/smoke-deepseek-agent.sh
scripts/check-book-readiness.sh
```

The validation gate is part of the public learning contract. It proves that
the book, source excerpts, Rust hygiene, mdBook build, tests, clippy, docs,
and dependency audit still agree.

The final command is:

```bash
./scripts/check-book-readiness.sh
```

When changing the real provider boundary and `DEEPSEEK_API_KEY` is available,
run the optional live provider gate:

```bash
RUN_LIVE_DEEPSEEK=1 ./scripts/check-book-readiness.sh
```

When changing SQL or the durable store and local Postgres binaries are
available, run the ephemeral local Postgres gate:

```bash
RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh
```

Run it before treating the implementation or manuscript as finished.

## Production Contract

The companion code is credible only when each layer has one job:

```text
manifest:
  feature boundaries are explicit

domain:
  invalid values are rejected early

SQL:
  state transitions preserve ownership and terminal rules

store:
  persistence data converts into domain data before worker logic

worker:
  job lifecycle is explicit and event-backed

provider:
  external model behavior becomes domain outcome

release gate:
  release_gate_runs converts through DbReleaseGateRunRow into ReleaseGateRunReceipt

runbook queries:
  operators can inspect state without private memory

validation:
  book claims, code behavior, and production hygiene are checked together
```

If a production rule has no source artifact and no validation evidence, it is
not implemented. It is only prose.

## Self-Check

Choose one chapter and trace it into source:

```text
chapter:
invariant:
source file:
test or query:
operator evidence:
```

If you cannot fill in all five lines, use Appendix G to find the artifact, then
return to this code path and read the source in order.

## Summary

The companion implementation is not a separate demo. It is the executable
version of the book's argument:

```text
durable state first
typed boundaries next
owned transitions after that
provider behavior behind a boundary
operator evidence at the end
```

Read the code in that order. The system will make more sense, and the
production invariants will be easier to see.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
