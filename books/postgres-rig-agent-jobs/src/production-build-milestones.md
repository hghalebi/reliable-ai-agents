# Appendix AB. Production Build Milestones

## Purpose

Use this appendix when you want the book to become a build plan.

Reading is useful, but reading is not production proof. A reliable AI agent
needs artifacts you can inspect: rows, types, queries, receipts, traces, tests,
release gates, runbooks, and recovery evidence.

This appendix turns the book into a short milestone ladder. Each milestone has
one build target, one inspection target, one command or check, one proof, and
one stop condition.

The rule is:

```text
build -> inspect -> run -> proof
```

If the proof is missing, stop. Repair the artifact before you add more
features.

## How To Use The Milestone Ladder

Do one milestone at a time.

Use this shape:

```text
build:
inspect:
run:
proof:
do not move on if:
```

The language is simple on purpose. The standard is not simple. Each milestone
still points to production evidence.

For ADHD, fatigue, or interruption-heavy work, treat the table as a restart
surface. Pick one row. Open one artifact. Run one check. Write one proof
sentence. Then stop or repair.

Do not treat reading as completion when the production proof is missing.

## Milestone Contract

Every milestone answers five questions:

| Question | Meaning |
| --- | --- |
| Build | What artifact must exist? |
| Inspect | Where can I see the state, boundary, or control? |
| Run | Which command, test, query, or readiness gate gives feedback? |
| Proof | What sentence proves the invariant? |
| Do not move on if | Which missing evidence blocks the next milestone? |

The contract keeps the book honest. If a chapter explains a concept but cannot
point to a build artifact, the concept is not ready for production use.

## The Milestones

| Milestone | Build | Inspect | Run | Proof | Do not move on if |
| --- | --- | --- | --- | --- | --- |
| Durable intake | A request becomes a durable `scheduled_jobs` row before model work starts. | `examples/postgres-rig-agent-jobs/sql/enqueue_agent_job.sql` and `examples/postgres-rig-agent-jobs/src/api.rs`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features api-server api::tests::api_admission_enqueues_typed_job` | The job can survive API process death because Postgres already stores it. | A request can start model work before a job row exists. |
| Typed domain boundary | Raw request data becomes validated domain types before worker logic. | `examples/postgres-rig-agent-jobs/src/domain.rs` and `examples/postgres-rig-agent-jobs/src/scheduled_job.rs`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml domain::tests::agent_instruction_rejects_empty_text` | The worker receives domain values, not raw strings pretending to be architecture. | A meaningful value crosses a boundary as a naked string, number, boolean, or JSON blob. |
| Worker ownership | A worker claims due work with state, owner, and lease evidence. | `examples/postgres-rig-agent-jobs/sql/pick_due_job.sql` and `examples/postgres-rig-agent-jobs/src/worker.rs`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml sql::tests::pick_query_uses_skip_locked_for_worker_cooperation` | One live worker owns the job for a limited time. | Two workers can believe they own the same job without lease evidence. |
| Timeout policy | Running work has a deadline and a typed action when the deadline is breached. | `examples/postgres-rig-agent-jobs/src/timeouts.rs` and `examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml timeouts` | A late job becomes a visible deadline decision instead of an endless wait. | Deadline behavior is hidden inside worker code, lease expiry, or raw status strings. |
| Cancellation request | A user, operator, system, or policy stop request becomes durable state. | `examples/postgres-rig-agent-jobs/src/cancellation.rs` and `examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml cancellation` | Stopping work has requested, applied, ignored-terminal, or expired evidence. | Cancellation deletes evidence, interrupts work silently, or cannot show who requested the stop. |
| Rig provider boundary | Rig handles agent and tool interaction while the reliability layer keeps durable state. | `examples/postgres-rig-agent-jobs/src/rig_runner.rs` and `examples/postgres-rig-agent-jobs/src/tool_contract.rs`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features rig-agent --no-run` | Provider integration compiles behind an explicit feature without becoming the state machine. | Rig output can bypass parsing, validation, policy, or audit evidence. |
| Idempotent side effects | External actions have idempotency keys and receipts. | `examples/postgres-rig-agent-jobs/src/recovery.rs`, `examples/postgres-rig-agent-jobs/src/compensation.rs`, and `examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml recovery::tests::missing_receipt_quarantines_replay` | Replay can avoid sending, charging, publishing, or mutating twice. | A retry can repeat an external side effect without checking an existing receipt. |
| Compensation action | A corrective side effect is requested, approved, claimed, retried, and completed as durable work. | `examples/postgres-rig-agent-jobs/src/compensation.rs`, `examples/postgres-rig-agent-jobs/sql/claim_compensation_actions.sql`, and `examples/postgres-rig-agent-jobs/sql/compensation_backlog.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml compensation` | A correction is approved, idempotent, leased, retryable, and terminal. | Compensation is treated as a magic rollback, informal note, or untracked manual cleanup. |
| Human approval gate | Risky work waits for a durable human decision. | `examples/postgres-rig-agent-jobs/src/approval.rs` and `examples/postgres-rig-agent-jobs/sql/waiting_human_approvals.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml approval::tests::requested_approval_can_be_approved` | The model cannot approve its own risky action. | A tool call with real-world risk can proceed without a stored decision. |
| Human escalation | Unsafe autonomous progress becomes a durable owner handoff. | `examples/postgres-rig-agent-jobs/src/escalation.rs`, `examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql`, and `books/postgres-rig-agent-jobs/src/16-human-approval-policy.md`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml escalation` | A deadline breach, repeated failure, security signal, approval timeout, or compatibility risk names a human owner and timeline. | Escalation lives only in chat, private memory, ticket text, or job status edits. |
| Agent handoff | Work can move from one named agent to another through a durable responsibility-transfer record. | `examples/postgres-rig-agent-jobs/src/handoff.rs`, `examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql`, and `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml handoff` | A handoff names source run, source agent, target agent, reason, payload, idempotency key, decision, and target job evidence. | Agents pass responsibility through chat text, duplicate handoffs create multiple target jobs, or unresolved handoffs are not queryable. |
| Observability and SLOs | Jobs produce trace, operation, audit, and SLO evidence. | `examples/postgres-rig-agent-jobs/sql/operation_events_by_job.sql`, `examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql`, and `examples/postgres-rig-agent-jobs/src/audit.rs`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml audit::tests::operation_row_conversion_rejects_invalid_trace_id` | An operator can reconstruct one job and one fleet symptom from evidence. | Debugging depends on private memory, terminal scrollback, or unstructured logs. |
| Cost and capacity guard | New work is admitted, delayed, or rejected from typed budget, quota, queue, and latency evidence. | `examples/postgres-rig-agent-jobs/src/admission_control.rs`, `examples/postgres-rig-agent-jobs/src/cost_accounting.rs`, `examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql`, `examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql`, and `examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml admission_control` | New work can be delayed or rejected before cost, quota, or latency turns retries into an incident. | Budget, quota, latency, and queue pressure are dashboards only and cannot affect admission or release. |
| Evaluation and release gate | Behavior changes need versioned evaluation and promotion evidence. | `examples/postgres-rig-agent-jobs/src/evaluation.rs`, `examples/postgres-rig-agent-jobs/src/release_gate.rs`, and `examples/postgres-rig-agent-jobs/sql/release_gate_status.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml release_gate::tests::release_gate_promotes_when_all_evidence_is_green` | A prompt, model, tool, or policy change can be accepted or blocked by evidence. | A behavior change can ship because it "looks good" but has no evaluation receipt. |
| Job-kind readiness review | One job kind has a target maturity level, current level, risk class, evidence counts, owner, and next review date. | `examples/postgres-rig-agent-jobs/src/job_kind_readiness.rs`, `examples/postgres-rig-agent-jobs/sql/job_kind_readiness_review.sql`, and `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml job_kind_readiness` | A job kind can be launched only when its readiness evidence matches its risk. | A regulated or high-risk job kind can be called ready with weak evidence or no review owner. |
| Job-kind launch packet | First-user exposure becomes a typed launch packet with readiness, release, failure-drill, rollback, restore, and known-gap evidence. | `examples/postgres-rig-agent-jobs/src/launch_packet.rs`, `examples/postgres-rig-agent-jobs/sql/job_kind_launch_packet_status.sql`, and `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml launch_packet` | First-user exposure becomes a typed launch packet with readiness, release, failure-drill, rollback, restore, and known-gap evidence. | A job kind reaches real users through a meeting note, chat promise, private checklist, or launch decision that cannot be queried. |
| Security boundary | Authority is decided outside the model. | `examples/postgres-rig-agent-jobs/src/security.rs`, `examples/postgres-rig-agent-jobs/src/sandbox.rs`, and `examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml sandbox::tests::policy_denies_non_allowlisted_network_destination` | Untrusted text cannot grant permission, expose secrets, or select unsafe tools. | The model can choose tenant, credential, network destination, filesystem path, or approval authority. |
| Tenant isolation guard | Actor tenant and requested tenant are checked before tool execution. | `examples/postgres-rig-agent-jobs/src/security.rs`, `examples/postgres-rig-agent-jobs/src/tenant_isolation.rs`, and `examples/postgres-rig-agent-jobs/sql/tenant_boundary_review.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml tenant_isolation` | Cross-tenant attempts are denied, queryable, and tied to policy evidence. | Tenant identity is only prompt text, tool JSON, or frontend state. |
| Injection and exfiltration guard | Hostile prompt or tool text remains untrusted until parser, authorization, sandbox, and approval evidence all agree. | `examples/postgres-rig-agent-jobs/src/tool_execution_gate.rs`, `examples/postgres-rig-agent-jobs/src/security.rs`, `examples/postgres-rig-agent-jobs/src/sandbox.rs`, `examples/postgres-rig-agent-jobs/sql/denied_authorization_events.sql`, and `examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml tool_execution_gate` | Prompt injection, tool injection, and data exfiltration attempts stop before trusted tool execution. | Hostile text can choose tenant, egress destination, filesystem path, secret visibility, approval state, or tool authority. |
| Agent memory | Future context is stored as typed, scoped, retained memory metadata instead of raw prompt text. | `examples/postgres-rig-agent-jobs/src/agent_memory.rs`, `examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql`, and `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml agent_memory` | Memory can influence prompts only when scope, kind, source, confidence, horizon, retention, and redaction evidence are valid. | Memory is a bag of strings, embeddings have no owner or retention, or cross-scope retrieval cannot be explained. |
| Credential lifecycle | Secret references have owners, rotation due dates, verification evidence, and exposure status. | `examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql`, `examples/postgres-rig-agent-jobs/src/credential_lifecycle.rs`, and `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml credential_lifecycle` | Operators can see due, overdue, stale, exposed, and recently revoked credentials without storing secret values. | Secret rotation lives in private memory, chat, shell history, or untyped notes. |
| Data protection | Redaction, erasure, export, and retention-review requests are durable operational state. | `examples/postgres-rig-agent-jobs/sql/data_protection_review.sql`, `examples/postgres-rig-agent-jobs/src/data_protection.rs`, and `examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml data_protection` | Operators can see open, overdue, pending redaction, and pending erasure work by evidence surface. | Privacy work lives in informal notes, chat promises, or untyped support queues. |
| Recovery and replay | Restore logic separates safe replay, unsafe replay, quarantine, and terminal work. | `examples/postgres-rig-agent-jobs/src/recovery.rs` and `examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml recovery::tests::restore_drill_passes_when_replay_is_safe_and_rto_met` | After restore, the system knows what can resume and what needs human review. | Backup exists, but no restore drill proves replay safety. |
| Failure drill | Controlled failure rehearsals have a hypothesis, blast radius, injection, rollback, evidence, result, and signoff. | `examples/postgres-rig-agent-jobs/src/failure_drill.rs`, `examples/postgres-rig-agent-jobs/sql/failure_drill_status.sql`, and `books/postgres-rig-agent-jobs/src/34-failure-drills.md`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml failure_drill` | A simulated or staged failure proves expected state transitions and operator evidence. | The team breaks things without a rollback action, evidence requirement, owner, or decision record. |
| Fault-tolerance review | Critical execution has isolation, redundancy, static stability, failover evidence, and release evidence. | `examples/postgres-rig-agent-jobs/src/fault_tolerance.rs`, `examples/postgres-rig-agent-jobs/sql/fault_tolerance_readiness.sql`, and `books/postgres-rig-agent-jobs/src/29b-extreme-fault-tolerance.md`. | `cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml fault_tolerance` | Already-approved work can continue from last-known-good state when non-critical control surfaces fail. | A control-plane outage can stop production execution accidentally, or the team cannot name the last-known-good state. |
| Operations runbooks | Operators have queries for stuck work, old work, failed work, pending decisions, credential lifecycle, data-protection work, and job-kind retirement. | `examples/postgres-rig-agent-jobs/sql/oldest_pending_job.sql`, `examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql`, `examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql`, `examples/postgres-rig-agent-jobs/sql/data_protection_review.sql`, `examples/postgres-rig-agent-jobs/sql/job_kind_lifecycle_review.sql`, and `books/postgres-rig-agent-jobs/src/23-runbooks.md`. | `RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh` | On-call can answer what is running, stuck, waiting, failed, unsafe, credential-exposed, privacy-overdue, or ready for deprecation. | The team cannot answer production questions from durable records. |
| Evidence-preserving scale | Scaling moves responsibility without losing state-machine evidence. | `books/postgres-rig-agent-jobs/src/30b-scaling-paths-after-postgres-first.md`, `examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql`, and `examples/postgres-rig-agent-jobs/sql/outbox_backlog.sql`. | `RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh` | New infrastructure preserves the old invariant and adds a rollback path. | A queue, workflow engine, cache, or worker pool hides the evidence trail. |
| Temporal adoption decision | Workflow execution moves only when timers, replay, child workflows, or cancellation dominate the custom worker code. | `books/postgres-rig-agent-jobs/src/30c-temporal-after-postgres-first.md`, `temporal_workflow_links`, `temporal_activity_receipts`, `agent_runs`, `tool_calls`, `human_approval_requests`, `operation_events`, and `audit_events`. | `python3 scripts/check-public-chapter-structure.py` | Temporal workflow history reconciles with Postgres product rows, activity receipts, approval decisions, audit events, and trace ids. `temporal_workflow_reconciliation.sql` proves the join. | Temporal becomes the only history, or product actions can complete without Postgres evidence. |
| Kafka adoption decision | Event distribution moves only when many independent consumers need durable fanout or replay. | `books/postgres-rig-agent-jobs/src/30d-kafka-after-postgres-first.md`, `outbox_events`, `kafka_publish_receipts`, `kafka_consumer_receipts`, typed event envelopes, and trace propagation. | `python3 scripts/check-public-chapter-structure.py` | Kafka replay cannot duplicate side effects because events come from the outbox and consumers write idempotent receipts. `kafka_replay_safety_by_event.sql` shows publish and replay evidence. | Raw worker JSON is published, consumers lack receipts, or replay can change the world twice. |

## Stop Conditions

Stop and repair when one of these is true:

- You cannot name the state before and after the action.
- You cannot name the actor allowed to change the state.
- You cannot point to the row, type, query, event, receipt, policy, or test.
- You cannot explain what happens after process death.
- You cannot explain whether retry is safe.
- You cannot tell whether a human decision is required.
- You cannot reconstruct what happened from durable evidence.
- You cannot show which version of prompt, model, tool, policy, or dataset was used.
- You cannot run the readiness gate.

These stop conditions are not learning failures. They are production signals.
They tell you where the system still needs a stronger boundary.

## Milestone Packet

Use this packet at the end of each milestone:

```text
milestone:
owner:
artifact:
command:
proof:
missing evidence:
next repair:
```

Good packet:

```text
milestone: Worker ownership
owner: background worker
artifact: pick_due_job.sql
command: cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml sql::tests::pick_query_uses_skip_locked_for_worker_cooperation
proof: one live worker can claim due work without racing another worker
missing evidence: none
next repair: add heartbeat proof before extending the worker pool
```

Weak packet:

```text
milestone: Worker ownership
proof: workers are reliable
```

The weak packet is not enough. It does not name the artifact, command, state,
or invariant.

## From Learning To Production

Use the milestones in this order:

```text
durable intake
typed domain boundary
worker ownership
timeout policy
cancellation request
Rig provider boundary
idempotent side effects
compensation action
human approval gate
human escalation
agent handoff
observability and SLOs
cost and capacity guard
evaluation and release gate
job-kind readiness review
security boundary
injection and exfiltration guard
agent memory
credential lifecycle
data protection
recovery and replay
failure drill
operations runbooks
evidence-preserving scale
Temporal adoption decision
Kafka adoption decision
```

The order matters.

Do not add scale before the state machine is visible. Do not add retries before
idempotency. Do not treat a timeout as a lost lease, a cancellation as deletion,
or compensation as a magic rollback. Do not add autonomy before approval and
security boundaries. Do not leave unsafe autonomous waits in chat, private
memory, ticket text, or job status edits; create durable escalation owner and
timeline evidence. Do not split one workflow across multiple agents until the
handoff row can prove source, target, decision, idempotency, and target job
evidence. Do not add model behavior changes before evaluation and release
gates. Do not call a job kind production-ready before its readiness row proves
the target level, risk class, evidence, owner, and review date. Do not let
hostile prompt or tool text choose tenant, egress, filesystem path, secret
visibility, approval state, or tool authority. Do not let
memory influence future prompts until scope, source, confidence, retention,
redaction, and retrieval evidence are visible. Do not run chaos experiments
without hypothesis, blast radius, rollback, evidence, and signoff. Do not add
years of operation before runbooks and recovery drills. Do not add Temporal
before workflow execution is the strained invariant. Do not add Kafka before the
outbox event, schema, partition key, consumer receipt, replay rule, and
authorization boundary are explicit.

This is the book's main production habit:

```text
small step
real artifact
visible proof
repair before expansion
```

## Summary

The milestone ladder keeps the book practical. It turns each major concept into
a visible production proof.

A serious agent is not ready because it responds well once. It is ready when a
team can prove durable intake, typed boundaries, worker ownership, timeout
policy, cancellation intent, provider boundaries, idempotent side effects,
compensation control, human control, observability, cost and capacity control,
human escalation, evaluation, security, injection and exfiltration resistance,
recovery, controlled failure drills, operations, and scaling evidence.

Simple language helps the learner find the proof. It does not remove the need
for the proof.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
