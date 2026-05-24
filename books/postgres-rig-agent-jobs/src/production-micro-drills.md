# Appendix AA. Production Micro-Drills

## Purpose

Use this appendix when a full lab feels too large.

The drills are small on purpose. Each one asks for one action, one check, and
one proof sentence. The action is short, but the proof is real. A drill should
help a tired reader move from simple language to deployable evidence without
lowering the production standard.

The rule is:

```text
small drill
real artifact
one proof sentence
stop or repair
```

## How To Use A Drill

Pick one drill. Do not do the whole appendix at once.

Use this shape:

```text
drill:
artifact:
action:
check:
proof sentence:
next:
```

The `proof sentence` must name what the artifact proves. If the sentence is
vague, the drill is not finished.

Good proof sentence:

```text
The job is durable because the row exists before the model starts.
```

Weak proof sentence:

```text
The system is reliable.
```

The weak sentence is too large. It does not tell the reader what to inspect.

## Drill Contract

Every micro-drill has four parts:

| Part | Meaning |
| --- | --- |
| Action | The one thing to read, run, write, or inspect. |
| Check | The concrete evidence to verify. |
| Proof | The one sentence that explains what the evidence proves. |
| Stop rule | The moment when the drill is complete. |

Do not add a second concept until the stop rule is true.

This is attention support and production discipline at the same time. It keeps
the reader from building on an unverified type, row, query, event, policy, or
runbook.

## The Drills

| Drill | Action | Check | Proof sentence |
| --- | --- | --- | --- |
| Durable intake | Find where a request becomes a job row. | The job is written before model execution. | Work survives process death because the database already knows it exists. |
| Idempotency key | Find the logical request identity. | A duplicate request maps to existing work or an existing receipt. | Retry can repeat uncertainty without creating a second logical action. |
| Typed input | Find one constructor or row conversion. | Bad raw input is rejected before worker logic. | The worker receives a domain value, not a disguised string. |
| Job claim | Inspect the claim query or worker claim method. | The claim uses state, owner, and lease evidence. | Only one live worker can own the job at a time. |
| Heartbeat | Find the heartbeat update path. | The worker can extend ownership only while it still owns the job. | A dead or stale worker cannot keep work forever. |
| Retry decision | Find the retry classification. | Transient, permanent, and exhausted failures produce different states. | Retry is a typed scheduling decision, not a blind loop. |
| Timeout policy | Find the deadline policy for running work. | A breached deadline produces a typed action without pretending the lease failed. | Late work is handled as a time-promise failure, not as lost ownership. |
| Cancellation request | Find one durable stop request. | Requested, applied, ignored-terminal, and expired cancellation states are separate. | Stopping work leaves an auditable intent and outcome. |
| Dead letter | Find the terminal failure evidence. | The failure reason is queryable after execution stops. | Failed work becomes visible state for operators. |
| Tool contract | Find one typed tool input and output. | Raw model output is parsed and validated before tool execution. | The tool receives a checked request, not trusted model text. |
| Approval gate | Find the approval row or decision type. | Risky work waits for a durable decision. | The model cannot approve its own risky action. |
| Human escalation | Find one open or acknowledged escalation. | Target, kind, severity, reason, owner, status, and timeline evidence are queryable. | Unsafe autonomous progress becomes named human responsibility. |
| Side-effect receipt | Find the receipt for an external action. | Replay checks the receipt before repeating the action. | Recovery can avoid sending, charging, or publishing twice. |
| Compensation action | Find one corrective side-effect record. | Original receipt, approval, idempotency key, lease, attempts, and terminal outcome are recorded. | A correction is controlled work, not an invisible rollback. |
| Agent handoff | Find the handoff row and target job evidence. | Source agent, target agent, reason, payload, idempotency key, decision, and target job are recorded. | Responsibility moves between agents only through durable evidence. |
| Trace id | Find where trace context is stored. | The same run can be followed across API, worker, model, and tool evidence. | Operators can reconstruct one job without private memory. |
| Cost and capacity guard | Find budget, quota, queue, and latency evidence for one job kind. | Budget state, provider quota pressure, queue depth, oldest pending age, token cost, and start-latency evidence are visible. | New work can be delayed or rejected before overload or runaway spend multiplies retries. |
| Evaluation receipt | Find one evaluation result tied to versions. | Prompt, model, tool, policy, dataset, and score are recorded together. | Behavior changes can be reviewed before release. |
| Release gate | Inspect the release-gate status evidence. | Missing evaluation, SLO, compatibility, approval, or rollback proof blocks promotion. | A release decision is evidence-backed, not confidence-backed. |
| Job-kind readiness review | Find the readiness review for one job kind. | Target level, current level, risk class, evidence count, gaps, owner, and next review date agree. | A job kind is ready only when its risk tier has enough evidence. |
| Job-kind launch packet | Find the first-user launch packet for one job kind. | Readiness review, release gate, failure drill, rollback, restore, known gaps, owner, and review date are queryable. | First-user exposure is a typed production decision, not a meeting note. |
| Security boundary | Find one authorization or sandbox decision. | Authority is decided outside the model. | Untrusted text cannot grant itself permission. |
| Injection and exfiltration guard | Trace one hostile instruction through parser, authorization, sandbox, and approval. | Unknown tools, cross-tenant requests, model-visible secrets, unsafe egress, and missing approval stop before execution. | Hostile text stays data; it cannot grant authority or leak protected data. |
| Tenant isolation guard | Inspect one cross-tenant tool request from model output to authorization evidence. | Actor tenant, requested tenant, denied decision, policy version, trace id, and `tenant_boundary_review.sql` agree. | Tenant scope is authorization state, not prompt text. |
| Agent memory | Find one typed memory record and its metadata query. | Scope, kind, source, confidence, horizon, retention, embedding reference, and content redaction are explicit. | Memory can influence future prompts only through typed, scoped, reviewable evidence. |
| Credential lifecycle review | Find the credential ledger and rotation review query. | Secret references have owners, due dates, verification, exposure, and revocation evidence. | Secret values stay outside Postgres while credential lifecycle risk remains visible. |
| Restore replay | Find the restore or replay decision. | Terminal, receipt-backed, unsafe, and replayable jobs are separated. | Recovery resumes only the work that can safely resume. |
| Failure drill | Find one controlled failure rehearsal. | Hypothesis, blast radius, injection, rollback, required evidence, observed evidence, and decision are named. | Failure practice becomes evidence-backed learning, not random breakage. |
| Fault-tolerance review | Find the control-plane and execution-plane readiness row. | Isolation, redundancy, static stability, failover drill, release gate, and last-known-good versions are named. | Critical execution does not depend on live control-plane availability for already-approved work. |
| Temporal adoption decision | Find the workflow adoption bridge. | Workflow id, activity receipt, product evidence, trace id, and rollback rule reconcile. | Temporal can own execution only when product truth remains in typed evidence. |
| Kafka adoption decision | Find the event-stream adoption bridge. | Outbox id, schema version, topic-partition-offset, consumer receipt, and replay rule reconcile. | Kafka can distribute facts only when replay cannot duplicate side effects. |
| Data protection review | Find the privacy request ledger and review query. | Redaction, erasure, export, and retention-review work is open, overdue, or complete with evidence. | Privacy work is durable operational state, not an informal note. |
| Maintenance review | Find one cadence or evidence-packet row. | Owner, review date, evidence, gap, and next action are named. | Years-long reliability has a schedule, not a mood. |

## Exact Artifact Index

Use this table when "find the artifact" is still too much work. It names the
first file or command to inspect.

| Drill | Start here | Fast check |
| --- | --- | --- |
| Durable intake | `examples/postgres-rig-agent-jobs/sql/enqueue_agent_job.sql`, `examples/postgres-rig-agent-jobs/src/api.rs`, `examples/postgres-rig-agent-jobs/src/postgres_store.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features api-server api::tests::api_admission_enqueues_typed_job` |
| Idempotency key | `examples/postgres-rig-agent-jobs/sql/resolve_existing_agent_job.sql`, `examples/postgres-rig-agent-jobs/src/api.rs`, `examples/postgres-rig-agent-jobs/src/memory_store.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features api-server duplicate_idempotency_key_returns_same_job` |
| Typed input | `examples/postgres-rig-agent-jobs/src/domain.rs`, `examples/postgres-rig-agent-jobs/src/postgres_store.rs`, `examples/postgres-rig-agent-jobs/src/scheduled_job.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml domain::tests::agent_instruction_rejects_empty_text` |
| Job claim | `examples/postgres-rig-agent-jobs/sql/pick_due_job.sql`, `examples/postgres-rig-agent-jobs/src/worker.rs`, `examples/postgres-rig-agent-jobs/src/scheduled_job.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml sql::tests::pick_query_uses_skip_locked_for_worker_cooperation` |
| Heartbeat | `examples/postgres-rig-agent-jobs/sql/extend_lease.sql`, `examples/postgres-rig-agent-jobs/src/worker.rs`, `examples/postgres-rig-agent-jobs/src/memory_store.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml worker::tests::worker_heartbeat_extends_owned_lease_and_records_event` |
| Retry decision | `examples/postgres-rig-agent-jobs/sql/retry_or_dead.sql`, `examples/postgres-rig-agent-jobs/sql/fail_or_retry_scheduled_job.sql`, `examples/postgres-rig-agent-jobs/src/scheduled_job.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml worker::tests::worker_schedules_retry_after_transient_failure` |
| Timeout policy | `examples/postgres-rig-agent-jobs/src/timeouts.rs`, `examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql`, `books/postgres-rig-agent-jobs/src/13-leases-heartbeats-cancellation.md` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml timeouts` |
| Cancellation request | `examples/postgres-rig-agent-jobs/src/cancellation.rs`, `examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql`, `books/postgres-rig-agent-jobs/src/13-leases-heartbeats-cancellation.md` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml cancellation` |
| Dead letter | `examples/postgres-rig-agent-jobs/sql/dead_jobs_by_reason.sql`, `examples/postgres-rig-agent-jobs/src/failure_history.rs`, `examples/postgres-rig-agent-jobs/src/worker.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml worker::tests::worker_marks_job_dead_after_attempts_are_exhausted` |
| Tool contract | `examples/postgres-rig-agent-jobs/src/tool_contract.rs`, `examples/postgres-rig-agent-jobs/src/tool_execution_gate.rs`, `examples/postgres-rig-agent-jobs/src/tool_call.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml tool_contract::tests::model_output_pipeline_validates_policy_and_executes_typed_tool` |
| Approval gate | `examples/postgres-rig-agent-jobs/src/approval.rs`, `examples/postgres-rig-agent-jobs/sql/waiting_human_approvals.sql`, `examples/postgres-rig-agent-jobs/src/tool_execution_gate.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml approval::tests::requested_approval_can_be_approved` |
| Human escalation | `examples/postgres-rig-agent-jobs/src/escalation.rs`, `examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql`, `books/postgres-rig-agent-jobs/src/16-human-approval-policy.md` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml escalation` |
| Side-effect receipt | `examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql`, `examples/postgres-rig-agent-jobs/src/compensation.rs`, `examples/postgres-rig-agent-jobs/src/recovery.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml recovery::tests::missing_receipt_quarantines_replay` |
| Compensation action | `examples/postgres-rig-agent-jobs/src/compensation.rs`, `examples/postgres-rig-agent-jobs/sql/claim_compensation_actions.sql`, `examples/postgres-rig-agent-jobs/sql/compensation_backlog.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml compensation` |
| Agent handoff | `examples/postgres-rig-agent-jobs/src/handoff.rs`, `examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql`, `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml handoff` |
| Trace id | `examples/postgres-rig-agent-jobs/src/audit.rs`, `examples/postgres-rig-agent-jobs/sql/operation_events_by_job.sql`, `examples/postgres-rig-agent-jobs/sql/audit_events_by_run.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml audit::tests::operation_row_conversion_rejects_invalid_trace_id` |
| Cost and capacity guard | `examples/postgres-rig-agent-jobs/src/admission_control.rs`, `examples/postgres-rig-agent-jobs/src/cost_accounting.rs`, `examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql`, `examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql`, `examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml admission_control` |
| Evaluation receipt | `examples/postgres-rig-agent-jobs/src/evaluation.rs`, `examples/postgres-rig-agent-jobs/sql/evaluation_receipts_by_version.sql`, `examples/postgres-rig-agent-jobs/src/release_gate.rs` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml evaluation::tests::passing_evaluation_allows_promotion` |
| Release gate | `examples/postgres-rig-agent-jobs/src/release_gate.rs`, `examples/postgres-rig-agent-jobs/sql/release_gate_status.sql`, `books/postgres-rig-agent-jobs/src/first-production-deployment-proof.md` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml release_gate::tests::release_gate_promotes_when_all_evidence_is_green` |
| Job-kind readiness review | `examples/postgres-rig-agent-jobs/src/job_kind_readiness.rs`, `examples/postgres-rig-agent-jobs/sql/job_kind_readiness_review.sql`, `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml job_kind_readiness` |
| Job-kind launch packet | `examples/postgres-rig-agent-jobs/src/launch_packet.rs`, `examples/postgres-rig-agent-jobs/sql/job_kind_launch_packet_status.sql`, `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml launch_packet` |
| Security boundary | `examples/postgres-rig-agent-jobs/src/security.rs`, `examples/postgres-rig-agent-jobs/src/sandbox.rs`, `examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml sandbox::tests::policy_denies_non_allowlisted_network_destination` |
| Injection and exfiltration guard | `examples/postgres-rig-agent-jobs/src/tool_execution_gate.rs`, `examples/postgres-rig-agent-jobs/src/security.rs`, `examples/postgres-rig-agent-jobs/src/sandbox.rs`, `examples/postgres-rig-agent-jobs/sql/denied_authorization_events.sql`, `examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml tool_execution_gate` |
| Agent memory | `examples/postgres-rig-agent-jobs/src/agent_memory.rs`, `examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql`, `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml agent_memory` |
| Credential lifecycle review | `examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql`, `examples/postgres-rig-agent-jobs/src/credential_lifecycle.rs`, `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml credential_lifecycle` |
| Restore replay | `examples/postgres-rig-agent-jobs/src/recovery.rs`, `examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql`, `examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml recovery::tests::restore_drill_passes_when_replay_is_safe_and_rto_met` |
| Failure drill | `examples/postgres-rig-agent-jobs/src/failure_drill.rs`, `examples/postgres-rig-agent-jobs/sql/failure_drill_status.sql`, `books/postgres-rig-agent-jobs/src/34-failure-drills.md` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml failure_drill` |
| Fault-tolerance review | `examples/postgres-rig-agent-jobs/src/fault_tolerance.rs`, `examples/postgres-rig-agent-jobs/sql/fault_tolerance_readiness.sql`, `books/postgres-rig-agent-jobs/src/29b-extreme-fault-tolerance.md` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml fault_tolerance` |
| Temporal adoption decision | `examples/postgres-rig-agent-jobs/src/temporal_adoption.rs`, `examples/postgres-rig-agent-jobs/sql/temporal_workflow_reconciliation.sql`, `books/postgres-rig-agent-jobs/src/30c-temporal-after-postgres-first.md`, `books/postgres-rig-agent-jobs/src/44-production-evidence-packets.md` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml temporal_adoption` |
| Kafka adoption decision | `examples/postgres-rig-agent-jobs/src/kafka_adoption.rs`, `examples/postgres-rig-agent-jobs/sql/kafka_replay_safety_by_event.sql`, `books/postgres-rig-agent-jobs/src/30d-kafka-after-postgres-first.md`, `books/postgres-rig-agent-jobs/src/44-production-evidence-packets.md` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml kafka_adoption` |
| Data protection review | `examples/postgres-rig-agent-jobs/sql/data_protection_review.sql`, `examples/postgres-rig-agent-jobs/src/data_protection.rs`, `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql` | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml data_protection` |
| Maintenance review | `books/postgres-rig-agent-jobs/src/52-maintenance-cadence.md`, `examples/postgres-rig-agent-jobs/sql/job_kind_lifecycle_review.sql`, `examples/postgres-rig-agent-jobs/sql/storage_pressure_by_table.sql`, `examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql`, `examples/postgres-rig-agent-jobs/sql/retention_review_by_surface.sql`, `examples/postgres-rig-agent-jobs/sql/data_protection_review.sql` | `RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh` |

The table is not a replacement for the full chapter. It is a starting point
for fast feedback. Once the proof sentence is clear, return to the chapter,
lab, or launch packet.

The artifact index is checked by `scripts/check-micro-drill-artifacts.py`. The
checker verifies that each listed source file exists and that each focused
Cargo test command selects at least one real test.

## Seven-Minute Drill

When focus is low, shrink any drill into seven minutes:

```text
minute 1: pick one drill
minute 2: open the artifact
minute 3: name the state
minute 4: name the move
minute 5: name the proof
minute 6: write the proof sentence
minute 7: stop or write one repair action
```

If the repair action is bigger than one sentence, split the drill.

## When A Drill Fails

A failed drill is useful. It tells you where the production proof is weak.

Use this repair loop:

```text
missing artifact:
missing proof:
likely risk:
small repair:
validation:
```

Example:

```text
missing artifact: side-effect receipt
missing proof: replay cannot prove whether the email was sent
likely risk: retry may send the email twice
small repair: add a receipt row before allowing replay
validation: add a replay test that refuses work without receipt evidence
```

Do not call the drill complete because the idea sounds right. Complete means
the proof can be inspected.

## From Drill To Deployment

Before first production exposure, connect micro-drills to Appendix Y:

```text
durable intake drill -> durable intake proof
job claim drill -> worker ownership proof
timeout policy drill -> deadline proof
cancellation request drill -> durable stop proof
tool contract drill -> provider boundary proof
approval gate drill -> policy or approval proof
human escalation drill -> named-owner proof
compensation action drill -> corrective side-effect proof
agent handoff drill -> responsibility transfer proof
trace id drill -> observability proof
cost and capacity guard drill -> admission and spend-control proof
evaluation receipt drill -> evaluation proof
release gate drill -> release decision proof
job-kind readiness review drill -> job-kind launch-level proof
job-kind launch packet drill -> first-user launch proof
security boundary drill -> security proof
injection and exfiltration guard drill -> abuse-boundary proof
agent memory drill -> memory influence proof
restore replay drill -> restore and replay note
failure drill -> controlled-failure proof
temporal adoption decision drill -> workflow adoption proof
kafka adoption decision drill -> event-stream adoption proof
```

The drill is the small practice loop. The launch packet is the production
decision. They should point to the same evidence.

## Summary

Micro-drills make hard production knowledge easier to start, not easier to
fake.

Invariant: every drill must end with a real artifact and one proof sentence.

Evidence: the artifact can be a Rust type, SQL row, query, event, test,
receipt, policy record, runbook command, evaluation result, release gate, or
readiness command.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
