#!/usr/bin/env python3
"""Check the Postgres coordination schema promised by the book objective.

The book teaches Postgres as the first durable coordination layer. This script
keeps that promise concrete by checking the source SQL for the tables, columns,
state constraints, JSON boundary checks, idempotency keys, and indexes that the
chapters and runbooks rely on.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SQL_DIR = ROOT / "examples" / "postgres-rig-agent-jobs" / "sql"
PRIMARY_SCHEMA = SQL_DIR / "001_agent_jobs.sql"
TRACKING_SCHEMA = SQL_DIR / "002_agent_tracking.sql"


def read_schema() -> str:
    return "\n".join(
        (
            PRIMARY_SCHEMA.read_text(encoding="utf-8"),
            TRACKING_SCHEMA.read_text(encoding="utf-8"),
        )
    )


def table_block(schema: str, table_name: str) -> str | None:
    match = re.search(
        rf"create table {re.escape(table_name)} \(\n(?P<body>.*?)\n\);",
        schema,
        flags=re.S,
    )
    if match is None:
        return None
    return match.group("body")


def has_column(block: str, column_name: str) -> bool:
    return re.search(rf"^\s*{re.escape(column_name)}\s+", block, flags=re.M) is not None


def require_table(schema: str, table_name: str, failures: list[str]) -> str:
    block = table_block(schema, table_name)
    if block is None:
        failures.append(f"missing required Postgres coordination table: {table_name}")
        return ""
    return block


def require_columns(
    table_name: str,
    block: str,
    columns: tuple[str, ...],
    failures: list[str],
) -> None:
    for column in columns:
        if not has_column(block, column):
            failures.append(f"{table_name} missing required column: {column}")


def require_fragments(
    table_name: str,
    block: str,
    fragments: tuple[str, ...],
    failures: list[str],
) -> None:
    normalized_block = " ".join(block.split())
    for fragment in fragments:
        if fragment not in block and " ".join(fragment.split()) not in normalized_block:
            failures.append(f"{table_name} missing required schema fragment: {fragment}")


def require_schema_fragments(
    schema: str,
    fragments: tuple[str, ...],
    failures: list[str],
) -> None:
    normalized_schema = " ".join(schema.split())
    for fragment in fragments:
        if fragment not in schema and " ".join(fragment.split()) not in normalized_schema:
            failures.append(f"schema missing required fragment: {fragment}")


def check_table_contracts(schema: str, failures: list[str]) -> None:
    table_columns: dict[str, tuple[str, ...]] = {
        "agent_jobs": (
            "id",
            "kind",
            "payload_schema_version",
            "prompt_version",
            "model_route",
            "tool_version",
            "policy_version",
            "worker_build_id",
            "status",
            "payload",
            "result",
            "run_at",
            "attempt_count",
            "max_attempts",
            "locked_by",
            "locked_until",
            "idempotency_key",
            "last_error",
            "created_at",
            "updated_at",
        ),
        "scheduled_jobs": (
            "id",
            "task_name",
            "status",
            "payload",
            "attempts",
            "max_attempts",
            "next_run_at",
            "locked_by",
            "locked_until",
            "last_error",
            "idempotency_key",
            "created_at",
            "updated_at",
        ),
        "background_jobs": (
            "id",
            "scheduled_job_id",
            "job_kind",
            "workflow_state",
            "retry_state",
            "attempts",
            "max_attempts",
            "next_retry_at",
            "execution_deadline_at",
            "timeout_policy_name",
            "timeout_action",
            "last_failure_class",
            "last_error",
            "created_at",
            "updated_at",
        ),
        "agent_runs": (
            "id",
            "scheduled_job_id",
            "background_job_id",
            "agent_name",
            "lifecycle_status",
            "prompt_version",
            "model_version",
            "trace_id",
            "deadline_at",
            "timeout_policy_name",
            "timeout_action",
            "started_at",
            "finished_at",
            "created_at",
            "updated_at",
        ),
        "agent_steps": (
            "id",
            "run_id",
            "step_index",
            "step_kind",
            "status",
            "input_ref",
            "output_ref",
            "error",
            "started_at",
            "completed_at",
            "created_at",
        ),
        "tool_calls": (
            "id",
            "run_id",
            "step_id",
            "tool_name",
            "tool_version",
            "status",
            "idempotency_key",
            "input",
            "output",
            "error",
            "started_at",
            "completed_at",
            "created_at",
        ),
        "failure_history": (
            "id",
            "scheduled_job_id",
            "background_job_id",
            "run_id",
            "step_id",
            "tool_call_id",
            "failure_source",
            "failure_class",
            "failure_message",
            "workflow_state",
            "retry_state",
            "failure_outcome",
            "attempt",
            "max_attempts",
            "next_retry_at",
            "trace_id",
            "span_id",
            "occurred_at",
            "recorded_at",
        ),
        "human_approval_requests": (
            "id",
            "run_id",
            "status",
            "requested_by",
            "decided_by",
            "reason",
            "requested_at",
            "decided_at",
            "expires_at",
        ),
        "agent_handoffs": (
            "id",
            "source_run_id",
            "from_agent",
            "to_agent",
            "reason",
            "payload",
            "status",
            "idempotency_key",
            "target_job_id",
            "decision_reason",
            "requested_at",
            "decided_at",
            "created_at",
            "updated_at",
        ),
        "audit_events": (
            "id",
            "run_id",
            "actor_type",
            "actor_id",
            "action",
            "subject",
            "event_data",
            "created_at",
        ),
        "operation_events": (
            "id",
            "job_id",
            "run_id",
            "trace_id",
            "span_id",
            "event_type",
            "severity",
            "message",
            "data",
            "created_at",
        ),
        "temporal_workflow_links": (
            "id",
            "scheduled_job_id",
            "agent_run_id",
            "workflow_type",
            "workflow_execution_ref",
            "task_queue",
            "idempotency_key",
            "trace_id",
            "workflow_status",
            "started_at",
            "completed_at",
            "created_at",
            "updated_at",
        ),
        "temporal_activity_receipts": (
            "id",
            "workflow_link_id",
            "activity_execution_ref",
            "tool_call_id",
            "idempotency_key",
            "operation_event_id",
            "recorded_at",
        ),
        "evaluation_runs": (
            "id",
            "run_id",
            "dataset_version",
            "evaluator_version",
            "prompt_version",
            "model_version",
            "tool_version",
            "policy_version",
            "status",
            "score",
            "report",
            "created_at",
            "completed_at",
        ),
        "agent_memory_records": (
            "id",
            "run_id",
            "memory_kind",
            "memory_scope",
            "source",
            "confidence",
            "memory_horizon",
            "retention_policy",
            "content",
            "embedding_ref",
            "created_at",
            "last_used_at",
        ),
        "outbox_events": (
            "id",
            "event_kind",
            "aggregate_id",
            "idempotency_key",
            "payload",
            "status",
            "attempts",
            "max_attempts",
            "next_attempt_at",
            "locked_by",
            "locked_until",
            "last_error",
            "occurred_at",
            "published_at",
            "created_at",
            "updated_at",
        ),
        "kafka_publish_receipts": (
            "id",
            "outbox_event_id",
            "event_kind",
            "aggregate_id",
            "schema_version",
            "topic",
            "partition_id",
            "record_offset",
            "trace_id",
            "published_at",
            "recorded_at",
        ),
        "kafka_consumer_receipts": (
            "id",
            "publish_receipt_id",
            "outbox_event_id",
            "consumer_group",
            "consumer_name",
            "topic",
            "partition_id",
            "record_offset",
            "idempotency_key",
            "status",
            "error",
            "processed_at",
            "recorded_at",
        ),
        "side_effect_receipts": (
            "id",
            "tool_call_id",
            "idempotency_key",
            "external_system",
            "external_correlation_id",
            "effect_kind",
            "receipt",
            "recorded_at",
        ),
        "credential_assets": (
            "id",
            "secret_ref",
            "credential_kind",
            "owner",
            "storage_location",
            "status",
            "rotation_interval_days",
            "last_rotated_at",
            "next_rotation_due_at",
            "last_verified_at",
            "exposure_reported_at",
            "revoked_at",
            "policy_version",
            "evidence",
            "created_at",
            "updated_at",
        ),
        "data_protection_requests": (
            "id",
            "request_kind",
            "surface",
            "subject_ref",
            "status",
            "requested_by",
            "reason",
            "policy_version",
            "evidence",
            "requested_at",
            "due_at",
            "completed_at",
        ),
        "release_gate_runs": (
            "id",
            "candidate_id",
            "gate_name",
            "job_kind",
            "release_reason",
            "risk",
            "decision",
            "prompt_version",
            "model_version",
            "tool_version",
            "policy_version",
            "worker_build_id",
            "payload_schema_version",
            "evaluation_run_id",
            "slo_decision",
            "compatibility_decision",
            "schema_migration_run_id",
            "approval_request_id",
            "blockers",
            "canary_percent",
            "rollback_plan",
            "evaluated_by",
            "operator_signoff",
            "evaluated_at",
            "created_at",
        ),
        "job_kind_readiness_reviews": (
            "id",
            "job_kind",
            "target_level",
            "current_level",
            "risk_class",
            "evidence_ready_count",
            "evidence_required_count",
            "blocking_gap_count",
            "owner",
            "next_change",
            "evidence",
            "reviewed_at",
            "next_review_at",
            "created_at",
            "updated_at",
        ),
        "job_kind_launch_packets": (
            "id",
            "job_kind",
            "target_level",
            "risk_class",
            "launch_decision",
            "owner",
            "durable_intake_proof",
            "worker_ownership_proof",
            "provider_boundary_proof",
            "side_effect_control_proof",
            "policy_or_approval_proof",
            "observability_proof",
            "evaluation_proof",
            "security_proof",
            "rollback_or_pause_plan",
            "restore_and_replay_note",
            "known_gaps",
            "readiness_review_id",
            "release_gate_run_id",
            "failure_drill_run_id",
            "reviewed_by",
            "reviewed_at",
            "next_review_at",
            "created_at",
            "updated_at",
        ),
    }

    for table_name, columns in table_columns.items():
        block = require_table(schema, table_name, failures)
        if block:
            require_columns(table_name, block, columns, failures)


def check_state_and_identity_contracts(schema: str, failures: list[str]) -> None:
    table_fragments: dict[str, tuple[str, ...]] = {
        "scheduled_jobs": (
            "status in ('pending', 'running', 'succeeded', 'failed', 'dead', 'cancelled')",
            "attempts int not null default 0 check (attempts >= 0)",
            "max_attempts int not null default 5 check (max_attempts > 0)",
            "idempotency_key text unique",
            "jsonb_typeof(payload) = 'object'",
            "(status = 'running' and locked_by is not null and locked_until is not null)",
        ),
        "background_jobs": (
            "workflow_state in (",
            "retry_state in (",
            "check (attempts <= max_attempts)",
            "waiting_for_retry",
            "execution_deadline_at is not null",
        ),
        "agent_runs": (
            "lifecycle_status in (",
            "prompt_version text not null",
            "model_version text not null",
            "trace_id ~ '^[0-9A-Fa-f]{32}$'",
            "scheduled_job_id is not null or background_job_id is not null",
        ),
        "agent_steps": (
            "unique (run_id, step_index)",
            "step_kind in ('plan', 'model_call', 'tool_call', 'approval_gate', 'finalize')",
            "status in ('pending', 'running', 'succeeded', 'failed', 'skipped')",
        ),
        "tool_calls": (
            "status in ('requested', 'validated', 'executed', 'failed', 'rejected')",
            "idempotency_key text not null unique",
            "jsonb_typeof(input) = 'object'",
            "output is null or jsonb_typeof(output) = 'object'",
        ),
        "agent_handoffs": (
            "idempotency_key text not null unique",
            "jsonb_typeof(payload) = 'object'",
            "from_agent <> to_agent",
            "status in ('requested', 'accepted', 'rejected', 'expired', 'cancelled')",
        ),
        "audit_events": (
            "actor_type in ('user', 'worker', 'model', 'system')",
            "jsonb_typeof(event_data) = 'object'",
        ),
        "operation_events": (
            "trace_id ~ '^[0-9A-Fa-f]{32}$'",
            "severity in ('debug', 'info', 'warn', 'error')",
            "jsonb_typeof(data) = 'object'",
        ),
        "temporal_workflow_links": (
            "workflow_execution_ref text not null unique",
            "idempotency_key text not null unique",
            "workflow_status in (",
            "trace_id ~ '^[0-9A-Fa-f]{32}$'",
            "completed_at is null or completed_at >= started_at",
        ),
        "temporal_activity_receipts": (
            "idempotency_key text not null unique",
            "unique (workflow_link_id, activity_execution_ref)",
            "operation_event_id bigint not null references operation_events(id)",
        ),
        "evaluation_runs": (
            "dataset_version text not null",
            "evaluator_version text not null",
            "jsonb_typeof(report) = 'object'",
            "score >= 0 and score <= 1",
        ),
        "agent_memory_records": (
            "confidence >= 0 and confidence <= 1",
            "memory_horizon in ('short_term', 'long_term')",
            "jsonb_typeof(content) = 'object'",
            "retention_policy in ('ephemeral', 'session')",
            "retention_policy in ('durable', 'audit')",
        ),
        "side_effect_receipts": (
            "idempotency_key text not null unique",
            "jsonb_typeof(receipt) = 'object'",
        ),
        "outbox_events": (
            "idempotency_key text not null unique",
            "jsonb_typeof(payload) = 'object'",
            "status in ('pending', 'publishing', 'published', 'failed')",
            "(status = 'publishing' and locked_by is not null and locked_until is not null)",
        ),
        "kafka_publish_receipts": (
            "outbox_event_id uuid not null unique",
            "schema_version int not null check (schema_version > 0)",
            "partition_id int not null check (partition_id >= 0)",
            "record_offset bigint not null check (record_offset >= 0)",
            "trace_id ~ '^[0-9A-Fa-f]{32}$'",
            "unique (topic, partition_id, record_offset)",
        ),
        "kafka_consumer_receipts": (
            "idempotency_key text not null unique",
            "status in ('completed', 'rejected_poison_event', 'failed_retryable')",
            "partition_id int not null check (partition_id >= 0)",
            "record_offset bigint not null check (record_offset >= 0)",
            "unique (consumer_group, outbox_event_id)",
        ),
        "credential_assets": (
            "secret_ref text not null unique",
            "jsonb_typeof(evidence) = 'object'",
            "status in ('active', 'rotation_due', 'rotating', 'revoked', 'compromised')",
        ),
        "data_protection_requests": (
            "request_kind in ('redaction', 'erasure', 'export', 'retention_review')",
            "jsonb_typeof(evidence) = 'object'",
            "due_at >= requested_at",
        ),
        "release_gate_runs": (
            "decision in ('promote', 'canary_only', 'block')",
            "jsonb_typeof(blockers) = 'array'",
            "canary_percent >= 0 and canary_percent <= 100",
        ),
        "job_kind_readiness_reviews": (
            "target_level in ('demo', 'prototype', 'production', 'regulated_high_risk')",
            "risk_class in ('low', 'medium', 'high', 'regulated')",
            "evidence_ready_count <= evidence_required_count",
        ),
        "job_kind_launch_packets": (
            "launch_decision in ('draft', 'blocked', 'approved_for_first_users', 'launched', 'paused')",
            "jsonb_typeof(known_gaps) = 'array'",
            "readiness_review_id is not null",
            "release_gate_run_id is not null",
            "failure_drill_run_id is not null",
            "jsonb_array_length(known_gaps) = 0",
        ),
    }

    for table_name, fragments in table_fragments.items():
        block = require_table(schema, table_name, failures)
        if block:
            require_fragments(table_name, block, fragments, failures)


def check_indexes_and_runbook_contracts(schema: str, failures: list[str]) -> None:
    require_schema_fragments(
        schema,
        (
            "create index scheduled_jobs_due_idx",
            "where status = 'pending'",
            "create index scheduled_jobs_locked_until_idx",
            "where status = 'running'",
            "create index background_jobs_retry_state_idx",
            "create index agent_runs_trace_idx",
            "create index agent_steps_run_idx",
            "create index tool_calls_run_idx",
            "create index failure_history_job_idx",
            "create index audit_events_run_idx",
            "create index operation_events_trace_idx",
            "create index human_approval_requests_status_idx",
            "create index agent_memory_records_scope_idx",
            "create index outbox_events_due_idx",
            "create index kafka_publish_receipts_topic_offset_idx",
            "create index kafka_consumer_receipts_group_status_idx",
            "create index temporal_workflow_links_status_idx",
            "create index temporal_activity_receipts_tool_call_idx",
            "create index credential_assets_due_idx",
            "create index data_protection_requests_status_due_idx",
            "create index release_gate_runs_job_kind_idx",
            "create index job_kind_readiness_reviews_due_idx",
            "create index job_kind_launch_packets_due_idx",
        ),
        failures,
    )

    runbook_files = (
        "claim_scheduled_jobs.sql",
        "scheduled_retries.sql",
        "waiting_human_approvals.sql",
        "running_agent_runs.sql",
        "failed_tool_calls.sql",
        "audit_events_by_run.sql",
        "operation_events_by_job.sql",
        "side_effect_receipts_by_run.sql",
        "evaluation_receipts_by_version.sql",
        "agent_memory_by_scope.sql",
        "credential_rotation_review.sql",
        "data_protection_review.sql",
        "restore_replay_candidates.sql",
        "release_gate_status.sql",
        "job_kind_readiness_review.sql",
        "job_kind_launch_packet_status.sql",
        "temporal_workflow_reconciliation.sql",
        "kafka_replay_safety_by_event.sql",
    )
    for runbook_file in runbook_files:
        if not (SQL_DIR / runbook_file).is_file():
            failures.append(f"missing Postgres evidence query: {runbook_file}")


def main() -> int:
    failures: list[str] = []
    schema = read_schema()

    check_table_contracts(schema, failures)
    check_state_and_identity_contracts(schema, failures)
    check_indexes_and_runbook_contracts(schema, failures)

    if failures:
        print("Postgres schema contract check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("Postgres schema contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
