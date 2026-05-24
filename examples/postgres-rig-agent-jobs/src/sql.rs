//! Checked SQL artifacts used by the companion implementation and runbooks.
//!
//! SQL files are not prose snippets in this book. They are executable artifacts:
//! migrations, worker transitions, SLI queries, operator diagnostics, and
//! audited controls. This module embeds those files into Rust so tests and
//! documentation can refer to the same checked sources.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Name of a checked SQL artifact under `examples/postgres-rig-agent-jobs/sql`.
pub struct SqlFileName(&'static str);

impl SqlFileName {
    /// Creates a SQL file name for a compile-time artifact registry entry.
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    /// Returns the checked SQL file name.
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Compile-time link between a checked SQL file name and its embedded source.
pub struct SqlArtifact {
    file_name: SqlFileName,
    contents: &'static str,
}

impl SqlArtifact {
    /// Creates a SQL artifact entry for the checked SQL registry.
    pub const fn new(file_name: SqlFileName, contents: &'static str) -> Self {
        Self {
            file_name,
            contents,
        }
    }

    /// Returns the file name for this SQL artifact.
    pub const fn file_name(self) -> SqlFileName {
        self.file_name
    }

    /// Returns the embedded SQL source.
    pub const fn contents(self) -> &'static str {
        self.contents
    }
}

pub const SCHEMA: &str = include_str!("../sql/001_agent_jobs.sql");
pub const TRACKING_SCHEMA: &str = include_str!("../sql/002_agent_tracking.sql");
pub const ENQUEUE_AGENT_JOB: &str = include_str!("../sql/enqueue_agent_job.sql");
pub const ADMIT_AGENT_JOB: &str = include_str!("../sql/admit_agent_job.sql");
pub const RESOLVE_EXISTING_AGENT_JOB: &str = include_str!("../sql/resolve_existing_agent_job.sql");
pub const RECORD_ADMISSION_DECISION: &str = include_str!("../sql/record_admission_decision.sql");
pub const PICK_DUE_JOB: &str = include_str!("../sql/pick_due_job.sql");
pub const CLAIM_SCHEDULED_JOBS: &str = include_str!("../sql/claim_scheduled_jobs.sql");
pub const COMPLETE_SCHEDULED_JOB: &str = include_str!("../sql/complete_scheduled_job.sql");
pub const FAIL_OR_RETRY_SCHEDULED_JOB: &str =
    include_str!("../sql/fail_or_retry_scheduled_job.sql");
pub const RECOVER_EXPIRED_JOBS: &str = include_str!("../sql/recover_expired_jobs.sql");
pub const EXTEND_LEASE: &str = include_str!("../sql/extend_lease.sql");
pub const MARK_CANCELLED: &str = include_str!("../sql/mark_cancelled.sql");
pub const MARK_SUCCEEDED: &str = include_str!("../sql/mark_succeeded.sql");
pub const RETRY_OR_DEAD: &str = include_str!("../sql/retry_or_dead.sql");
pub const QUEUE_METRICS: &str = include_str!("../sql/queue_metrics.sql");
pub const QUEUE_METRICS_BY_KIND: &str = include_str!("../sql/queue_metrics_by_kind.sql");
pub const SLI_JOB_START_LATENCY: &str = include_str!("../sql/sli_job_start_latency.sql");
pub const SLI_TERMINAL_JOBS_NOT_DEAD: &str = include_str!("../sql/sli_terminal_jobs_not_dead.sql");
pub const OLDEST_PENDING_JOB: &str = include_str!("../sql/oldest_pending_job.sql");
pub const EXPIRED_LEASES: &str = include_str!("../sql/expired_leases.sql");
pub const DEAD_JOBS_BY_REASON: &str = include_str!("../sql/dead_jobs_by_reason.sql");
pub const JOB_EVENT_TIMELINE: &str = include_str!("../sql/job_event_timeline.sql");
pub const FAILURE_HISTORY_BY_JOB: &str = include_str!("../sql/failure_history_by_job.sql");
pub const VERSION_COMPATIBILITY_RISKS: &str =
    include_str!("../sql/version_compatibility_risks.sql");
pub const SCHEMA_MIGRATION_STATUS: &str = include_str!("../sql/schema_migration_status.sql");
pub const RUNNING_AGENT_RUNS: &str = include_str!("../sql/running_agent_runs.sql");
pub const PENDING_AGENT_HANDOFFS: &str = include_str!("../sql/pending_agent_handoffs.sql");
pub const AGENT_MEMORY_BY_SCOPE: &str = include_str!("../sql/agent_memory_by_scope.sql");
pub const PENDING_CANCELLATION_REQUESTS: &str =
    include_str!("../sql/pending_cancellation_requests.sql");
pub const SCHEDULED_RETRIES: &str = include_str!("../sql/scheduled_retries.sql");
pub const WAITING_HUMAN_APPROVALS: &str = include_str!("../sql/waiting_human_approvals.sql");
pub const OPEN_HUMAN_ESCALATIONS: &str = include_str!("../sql/open_human_escalations.sql");
pub const FAILED_TOOL_CALLS: &str = include_str!("../sql/failed_tool_calls.sql");
pub const SIDE_EFFECT_RECEIPTS_BY_RUN: &str =
    include_str!("../sql/side_effect_receipts_by_run.sql");
pub const EVALUATION_RECEIPTS_BY_VERSION: &str =
    include_str!("../sql/evaluation_receipts_by_version.sql");
pub const RELEASE_GATE_STATUS: &str = include_str!("../sql/release_gate_status.sql");
pub const PROVIDER_USAGE_BY_JOB_KIND: &str = include_str!("../sql/provider_usage_by_job_kind.sql");
pub const JOB_KIND_LIFECYCLE_REVIEW: &str = include_str!("../sql/job_kind_lifecycle_review.sql");
pub const JOB_KIND_READINESS_REVIEW: &str = include_str!("../sql/job_kind_readiness_review.sql");
pub const JOB_KIND_LAUNCH_PACKET_STATUS: &str =
    include_str!("../sql/job_kind_launch_packet_status.sql");
pub const FAULT_TOLERANCE_READINESS: &str = include_str!("../sql/fault_tolerance_readiness.sql");
pub const DENIED_AUTHORIZATION_EVENTS: &str =
    include_str!("../sql/denied_authorization_events.sql");
pub const TENANT_BOUNDARY_REVIEW: &str = include_str!("../sql/tenant_boundary_review.sql");
pub const SANDBOX_POLICY_VIOLATIONS: &str = include_str!("../sql/sandbox_policy_violations.sql");
pub const CREDENTIAL_ROTATION_REVIEW: &str = include_str!("../sql/credential_rotation_review.sql");
pub const AUDIT_EVENTS_BY_RUN: &str = include_str!("../sql/audit_events_by_run.sql");
pub const OPERATION_EVENTS_BY_JOB: &str = include_str!("../sql/operation_events_by_job.sql");
pub const RUNNING_JOBS_PAST_DEADLINE: &str = include_str!("../sql/running_jobs_past_deadline.sql");
pub const RESTORE_REPLAY_CANDIDATES: &str = include_str!("../sql/restore_replay_candidates.sql");
pub const FAILURE_DRILL_STATUS: &str = include_str!("../sql/failure_drill_status.sql");
pub const STORAGE_PRESSURE_BY_TABLE: &str = include_str!("../sql/storage_pressure_by_table.sql");
pub const RETENTION_REVIEW_BY_SURFACE: &str =
    include_str!("../sql/retention_review_by_surface.sql");
pub const DATA_PROTECTION_REVIEW: &str = include_str!("../sql/data_protection_review.sql");
pub const CLAIM_OUTBOX_EVENTS: &str = include_str!("../sql/claim_outbox_events.sql");
pub const MARK_OUTBOX_EVENT_PUBLISHED: &str =
    include_str!("../sql/mark_outbox_event_published.sql");
pub const MARK_OUTBOX_EVENT_FAILED: &str = include_str!("../sql/mark_outbox_event_failed.sql");
pub const OUTBOX_BACKLOG: &str = include_str!("../sql/outbox_backlog.sql");
pub const APPROVE_COMPENSATION_ACTION: &str =
    include_str!("../sql/approve_compensation_action.sql");
pub const CLAIM_COMPENSATION_ACTIONS: &str = include_str!("../sql/claim_compensation_actions.sql");
pub const MARK_COMPENSATION_SUCCEEDED: &str =
    include_str!("../sql/mark_compensation_succeeded.sql");
pub const MARK_COMPENSATION_FAILED: &str = include_str!("../sql/mark_compensation_failed.sql");
pub const COMPENSATION_BACKLOG: &str = include_str!("../sql/compensation_backlog.sql");
pub const PAUSE_JOB_KIND: &str = include_str!("../sql/pause_job_kind.sql");
pub const RESUME_JOB_KIND: &str = include_str!("../sql/resume_job_kind.sql");
pub const TEMPORAL_WORKFLOW_RECONCILIATION: &str =
    include_str!("../sql/temporal_workflow_reconciliation.sql");
pub const KAFKA_REPLAY_SAFETY_BY_EVENT: &str =
    include_str!("../sql/kafka_replay_safety_by_event.sql");

/// Registry of every checked SQL artifact embedded into the companion crate.
pub const SQL_ARTIFACTS: &[SqlArtifact] = &[
    SqlArtifact::new(SqlFileName::new("001_agent_jobs.sql"), SCHEMA),
    SqlArtifact::new(SqlFileName::new("002_agent_tracking.sql"), TRACKING_SCHEMA),
    SqlArtifact::new(SqlFileName::new("enqueue_agent_job.sql"), ENQUEUE_AGENT_JOB),
    SqlArtifact::new(SqlFileName::new("admit_agent_job.sql"), ADMIT_AGENT_JOB),
    SqlArtifact::new(
        SqlFileName::new("resolve_existing_agent_job.sql"),
        RESOLVE_EXISTING_AGENT_JOB,
    ),
    SqlArtifact::new(
        SqlFileName::new("record_admission_decision.sql"),
        RECORD_ADMISSION_DECISION,
    ),
    SqlArtifact::new(SqlFileName::new("pick_due_job.sql"), PICK_DUE_JOB),
    SqlArtifact::new(
        SqlFileName::new("claim_scheduled_jobs.sql"),
        CLAIM_SCHEDULED_JOBS,
    ),
    SqlArtifact::new(
        SqlFileName::new("complete_scheduled_job.sql"),
        COMPLETE_SCHEDULED_JOB,
    ),
    SqlArtifact::new(
        SqlFileName::new("fail_or_retry_scheduled_job.sql"),
        FAIL_OR_RETRY_SCHEDULED_JOB,
    ),
    SqlArtifact::new(
        SqlFileName::new("recover_expired_jobs.sql"),
        RECOVER_EXPIRED_JOBS,
    ),
    SqlArtifact::new(SqlFileName::new("extend_lease.sql"), EXTEND_LEASE),
    SqlArtifact::new(SqlFileName::new("mark_cancelled.sql"), MARK_CANCELLED),
    SqlArtifact::new(SqlFileName::new("mark_succeeded.sql"), MARK_SUCCEEDED),
    SqlArtifact::new(SqlFileName::new("retry_or_dead.sql"), RETRY_OR_DEAD),
    SqlArtifact::new(SqlFileName::new("queue_metrics.sql"), QUEUE_METRICS),
    SqlArtifact::new(
        SqlFileName::new("queue_metrics_by_kind.sql"),
        QUEUE_METRICS_BY_KIND,
    ),
    SqlArtifact::new(
        SqlFileName::new("sli_job_start_latency.sql"),
        SLI_JOB_START_LATENCY,
    ),
    SqlArtifact::new(
        SqlFileName::new("sli_terminal_jobs_not_dead.sql"),
        SLI_TERMINAL_JOBS_NOT_DEAD,
    ),
    SqlArtifact::new(
        SqlFileName::new("oldest_pending_job.sql"),
        OLDEST_PENDING_JOB,
    ),
    SqlArtifact::new(SqlFileName::new("expired_leases.sql"), EXPIRED_LEASES),
    SqlArtifact::new(
        SqlFileName::new("dead_jobs_by_reason.sql"),
        DEAD_JOBS_BY_REASON,
    ),
    SqlArtifact::new(
        SqlFileName::new("job_event_timeline.sql"),
        JOB_EVENT_TIMELINE,
    ),
    SqlArtifact::new(
        SqlFileName::new("failure_history_by_job.sql"),
        FAILURE_HISTORY_BY_JOB,
    ),
    SqlArtifact::new(
        SqlFileName::new("version_compatibility_risks.sql"),
        VERSION_COMPATIBILITY_RISKS,
    ),
    SqlArtifact::new(
        SqlFileName::new("schema_migration_status.sql"),
        SCHEMA_MIGRATION_STATUS,
    ),
    SqlArtifact::new(
        SqlFileName::new("running_agent_runs.sql"),
        RUNNING_AGENT_RUNS,
    ),
    SqlArtifact::new(
        SqlFileName::new("pending_agent_handoffs.sql"),
        PENDING_AGENT_HANDOFFS,
    ),
    SqlArtifact::new(
        SqlFileName::new("agent_memory_by_scope.sql"),
        AGENT_MEMORY_BY_SCOPE,
    ),
    SqlArtifact::new(
        SqlFileName::new("pending_cancellation_requests.sql"),
        PENDING_CANCELLATION_REQUESTS,
    ),
    SqlArtifact::new(SqlFileName::new("scheduled_retries.sql"), SCHEDULED_RETRIES),
    SqlArtifact::new(
        SqlFileName::new("waiting_human_approvals.sql"),
        WAITING_HUMAN_APPROVALS,
    ),
    SqlArtifact::new(
        SqlFileName::new("open_human_escalations.sql"),
        OPEN_HUMAN_ESCALATIONS,
    ),
    SqlArtifact::new(SqlFileName::new("failed_tool_calls.sql"), FAILED_TOOL_CALLS),
    SqlArtifact::new(
        SqlFileName::new("side_effect_receipts_by_run.sql"),
        SIDE_EFFECT_RECEIPTS_BY_RUN,
    ),
    SqlArtifact::new(
        SqlFileName::new("evaluation_receipts_by_version.sql"),
        EVALUATION_RECEIPTS_BY_VERSION,
    ),
    SqlArtifact::new(
        SqlFileName::new("release_gate_status.sql"),
        RELEASE_GATE_STATUS,
    ),
    SqlArtifact::new(
        SqlFileName::new("provider_usage_by_job_kind.sql"),
        PROVIDER_USAGE_BY_JOB_KIND,
    ),
    SqlArtifact::new(
        SqlFileName::new("job_kind_lifecycle_review.sql"),
        JOB_KIND_LIFECYCLE_REVIEW,
    ),
    SqlArtifact::new(
        SqlFileName::new("job_kind_readiness_review.sql"),
        JOB_KIND_READINESS_REVIEW,
    ),
    SqlArtifact::new(
        SqlFileName::new("job_kind_launch_packet_status.sql"),
        JOB_KIND_LAUNCH_PACKET_STATUS,
    ),
    SqlArtifact::new(
        SqlFileName::new("fault_tolerance_readiness.sql"),
        FAULT_TOLERANCE_READINESS,
    ),
    SqlArtifact::new(
        SqlFileName::new("denied_authorization_events.sql"),
        DENIED_AUTHORIZATION_EVENTS,
    ),
    SqlArtifact::new(
        SqlFileName::new("tenant_boundary_review.sql"),
        TENANT_BOUNDARY_REVIEW,
    ),
    SqlArtifact::new(
        SqlFileName::new("sandbox_policy_violations.sql"),
        SANDBOX_POLICY_VIOLATIONS,
    ),
    SqlArtifact::new(
        SqlFileName::new("credential_rotation_review.sql"),
        CREDENTIAL_ROTATION_REVIEW,
    ),
    SqlArtifact::new(
        SqlFileName::new("audit_events_by_run.sql"),
        AUDIT_EVENTS_BY_RUN,
    ),
    SqlArtifact::new(
        SqlFileName::new("operation_events_by_job.sql"),
        OPERATION_EVENTS_BY_JOB,
    ),
    SqlArtifact::new(
        SqlFileName::new("running_jobs_past_deadline.sql"),
        RUNNING_JOBS_PAST_DEADLINE,
    ),
    SqlArtifact::new(
        SqlFileName::new("restore_replay_candidates.sql"),
        RESTORE_REPLAY_CANDIDATES,
    ),
    SqlArtifact::new(
        SqlFileName::new("failure_drill_status.sql"),
        FAILURE_DRILL_STATUS,
    ),
    SqlArtifact::new(
        SqlFileName::new("storage_pressure_by_table.sql"),
        STORAGE_PRESSURE_BY_TABLE,
    ),
    SqlArtifact::new(
        SqlFileName::new("retention_review_by_surface.sql"),
        RETENTION_REVIEW_BY_SURFACE,
    ),
    SqlArtifact::new(
        SqlFileName::new("data_protection_review.sql"),
        DATA_PROTECTION_REVIEW,
    ),
    SqlArtifact::new(
        SqlFileName::new("claim_outbox_events.sql"),
        CLAIM_OUTBOX_EVENTS,
    ),
    SqlArtifact::new(
        SqlFileName::new("mark_outbox_event_published.sql"),
        MARK_OUTBOX_EVENT_PUBLISHED,
    ),
    SqlArtifact::new(
        SqlFileName::new("mark_outbox_event_failed.sql"),
        MARK_OUTBOX_EVENT_FAILED,
    ),
    SqlArtifact::new(SqlFileName::new("outbox_backlog.sql"), OUTBOX_BACKLOG),
    SqlArtifact::new(
        SqlFileName::new("approve_compensation_action.sql"),
        APPROVE_COMPENSATION_ACTION,
    ),
    SqlArtifact::new(
        SqlFileName::new("claim_compensation_actions.sql"),
        CLAIM_COMPENSATION_ACTIONS,
    ),
    SqlArtifact::new(
        SqlFileName::new("mark_compensation_succeeded.sql"),
        MARK_COMPENSATION_SUCCEEDED,
    ),
    SqlArtifact::new(
        SqlFileName::new("mark_compensation_failed.sql"),
        MARK_COMPENSATION_FAILED,
    ),
    SqlArtifact::new(
        SqlFileName::new("compensation_backlog.sql"),
        COMPENSATION_BACKLOG,
    ),
    SqlArtifact::new(SqlFileName::new("pause_job_kind.sql"), PAUSE_JOB_KIND),
    SqlArtifact::new(SqlFileName::new("resume_job_kind.sql"), RESUME_JOB_KIND),
    SqlArtifact::new(
        SqlFileName::new("temporal_workflow_reconciliation.sql"),
        TEMPORAL_WORKFLOW_RECONCILIATION,
    ),
    SqlArtifact::new(
        SqlFileName::new("kafka_replay_safety_by_event.sql"),
        KAFKA_REPLAY_SAFETY_BY_EVENT,
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn sql_artifact_registry_names_each_checked_sql_file_once() {
        assert_eq!(SQL_ARTIFACTS.len(), 67);

        let mut names = HashSet::new();
        for artifact in SQL_ARTIFACTS {
            let file_name = artifact.file_name().as_str();
            assert!(
                file_name.ends_with(".sql"),
                "SQL artifact name should end in .sql: {file_name}"
            );
            assert!(
                names.insert(file_name),
                "SQL artifact registered more than once: {file_name}"
            );
            assert!(
                !artifact.contents().trim().is_empty(),
                "SQL artifact should embed non-empty source: {file_name}"
            );
        }

        for expected in [
            "001_agent_jobs.sql",
            "002_agent_tracking.sql",
            "pick_due_job.sql",
            "retry_or_dead.sql",
            "release_gate_status.sql",
            "job_kind_lifecycle_review.sql",
            "job_kind_readiness_review.sql",
            "job_kind_launch_packet_status.sql",
            "fault_tolerance_readiness.sql",
            "credential_rotation_review.sql",
            "tenant_boundary_review.sql",
            "data_protection_review.sql",
            "pause_job_kind.sql",
            "resume_job_kind.sql",
            "temporal_workflow_reconciliation.sql",
            "kafka_replay_safety_by_event.sql",
        ] {
            assert!(
                names.contains(expected),
                "SQL artifact registry missing {expected}"
            );
        }
    }

    #[test]
    fn pick_query_uses_skip_locked_for_worker_cooperation() {
        assert!(
            PICK_DUE_JOB
                .to_lowercase()
                .contains("for update of agent_jobs skip locked")
        );
        assert!(PICK_DUE_JOB.contains("agent_job_kind_controls"));
        assert!(PICK_DUE_JOB.contains("coalesce(controls.paused, false) = false"));
    }

    #[test]
    fn scheduled_job_queries_preserve_lease_and_retry_invariants() {
        assert!(CLAIM_SCHEDULED_JOBS.contains("from scheduled_jobs"));
        assert!(CLAIM_SCHEDULED_JOBS.contains("status = 'pending'"));
        assert!(CLAIM_SCHEDULED_JOBS.contains("next_run_at <= now()"));
        assert!(CLAIM_SCHEDULED_JOBS.contains("attempts < max_attempts"));
        assert!(
            CLAIM_SCHEDULED_JOBS
                .to_lowercase()
                .contains("for update skip locked")
        );
        assert!(CLAIM_SCHEDULED_JOBS.contains("locked_by = $2::text"));
        assert!(CLAIM_SCHEDULED_JOBS.contains("locked_until = now() + $3::interval"));

        assert!(COMPLETE_SCHEDULED_JOB.contains("status = 'succeeded'"));
        assert!(COMPLETE_SCHEDULED_JOB.contains("status = 'running'"));
        assert!(COMPLETE_SCHEDULED_JOB.contains("locked_by = $2::text"));
        assert!(COMPLETE_SCHEDULED_JOB.contains("locked_until >= now()"));

        assert!(FAIL_OR_RETRY_SCHEDULED_JOB.contains("when attempts >= max_attempts then 'dead'"));
        assert!(FAIL_OR_RETRY_SCHEDULED_JOB.contains("else 'pending'"));
        assert!(FAIL_OR_RETRY_SCHEDULED_JOB.contains("now() + $2::interval"));
        assert!(FAIL_OR_RETRY_SCHEDULED_JOB.contains("locked_by = $4::text"));
        assert!(FAIL_OR_RETRY_SCHEDULED_JOB.contains("locked_until >= now()"));
    }

    #[test]
    fn schema_has_job_state_and_event_ledger() {
        assert!(SCHEMA.contains("create table agent_jobs"));
        assert!(SCHEMA.contains("create table agent_job_events"));
        assert!(SCHEMA.contains("create table agent_job_kind_control_events"));
        assert!(SCHEMA.contains("create table admission_decision_events"));
        assert!(SCHEMA.contains("payload_schema_version int not null"));
        assert!(SCHEMA.contains("prompt_version text not null"));
        assert!(SCHEMA.contains("status in"));
        assert!(SCHEMA.contains("check (jsonb_typeof(payload) = 'object')"));
        assert!(SCHEMA.contains("check (result is null or jsonb_typeof(result) = 'object')"));
        assert!(SCHEMA.contains("status = 'succeeded' and result is not null"));
        assert!(SCHEMA.contains("status <> 'succeeded' and result is null"));
        assert!(SCHEMA.contains("duplicate_suppressed"));
        assert!(SCHEMA.contains("decision in ('accepted', 'delayed', 'rejected')"));
        assert!(SCHEMA.contains("priority in ('interactive', 'standard', 'bulk')"));
        assert!(SCHEMA.contains("budget_state in ('within_budget', 'exceeded')"));
        assert!(SCHEMA.contains("updated_by text not null"));
        assert!(SCHEMA.contains("action text not null check (action in ('paused', 'resumed'))"));
        assert!(SCHEMA.contains("previous_paused boolean not null"));
        assert!(SCHEMA.contains("new_paused boolean not null"));
        assert!(SCHEMA.contains("agent_job_kind_control_events_kind_idx"));
    }

    #[test]
    fn tracking_schema_names_production_agent_evidence_surfaces() {
        for table_name in [
            "scheduled_jobs",
            "background_jobs",
            "agent_runs",
            "cancellation_requests",
            "agent_handoffs",
            "agent_steps",
            "tool_calls",
            "failure_history",
            "authorization_events",
            "sandbox_events",
            "side_effect_receipts",
            "outbox_events",
            "kafka_publish_receipts",
            "kafka_consumer_receipts",
            "audit_events",
            "operation_events",
            "temporal_workflow_links",
            "temporal_activity_receipts",
            "provider_usage_events",
            "human_approval_requests",
            "human_escalations",
            "compensation_actions",
            "evaluation_runs",
            "agent_memory_records",
            "restore_drill_runs",
            "failure_drill_runs",
            "schema_migration_runs",
            "release_gate_runs",
            "job_kind_readiness_reviews",
            "fault_tolerance_reviews",
            "data_protection_requests",
        ] {
            assert!(TRACKING_SCHEMA.contains(&format!("create table {table_name}")));
        }

        assert!(TRACKING_SCHEMA.contains("idempotency_key text not null unique"));
        assert!(TRACKING_SCHEMA.contains("create table temporal_workflow_links"));
        assert!(TRACKING_SCHEMA.contains("workflow_execution_ref text not null unique"));
        assert!(TRACKING_SCHEMA.contains("workflow_status in"));
        assert!(TRACKING_SCHEMA.contains("create table temporal_activity_receipts"));
        assert!(TRACKING_SCHEMA.contains("unique (workflow_link_id, activity_execution_ref)"));
        assert!(TRACKING_SCHEMA.contains("create table kafka_publish_receipts"));
        assert!(TRACKING_SCHEMA.contains("unique (topic, partition_id, record_offset)"));
        assert!(TRACKING_SCHEMA.contains("create table kafka_consumer_receipts"));
        assert!(TRACKING_SCHEMA.contains("unique (consumer_group, outbox_event_id)"));
        assert!(TRACKING_SCHEMA.contains("status = 'running' and locked_by is not null"));
        assert!(TRACKING_SCHEMA.contains("workflow_state in"));
        assert!(TRACKING_SCHEMA.contains("retry_state in"));
        assert!(TRACKING_SCHEMA.contains("attempts <= max_attempts"));
        assert!(TRACKING_SCHEMA.contains("updated_at >= created_at"));
        assert!(TRACKING_SCHEMA.contains("btrim(job_kind) <> ''"));
        assert!(
            TRACKING_SCHEMA
                .contains("last_failure_class is null or btrim(last_failure_class) <> ''")
        );
        assert!(
            TRACKING_SCHEMA
                .contains("workflow_state in ('leased', 'executing_agent', 'waiting_for_human')")
        );
        assert!(TRACKING_SCHEMA.contains("execution_deadline_at is not null"));
        assert!(
            TRACKING_SCHEMA
                .contains("retry_state = 'waiting_for_retry' and next_retry_at is not null")
        );
        assert!(
            TRACKING_SCHEMA
                .contains("retry_state <> 'waiting_for_retry' and next_retry_at is null")
        );
        assert!(TRACKING_SCHEMA.contains("workflow_state = 'failed'"));
        assert!(TRACKING_SCHEMA.contains("retry_state in ('exhausted', 'permanent_failure')"));
        assert!(TRACKING_SCHEMA.contains("last_failure_class is not null"));
        assert!(TRACKING_SCHEMA.contains("execution_deadline_at timestamptz"));
        assert!(TRACKING_SCHEMA.contains("deadline_at timestamptz"));
        assert!(TRACKING_SCHEMA.contains("timeout_policy_name text not null"));
        assert!(TRACKING_SCHEMA.contains(
            "timeout_action in ('schedule_retry', 'cancel_job', 'escalate_to_human', 'dead_letter')"
        ));
        assert!(
            TRACKING_SCHEMA
                .contains("scheduled_job_id is not null or background_job_id is not null")
        );
        assert!(TRACKING_SCHEMA.contains("btrim(agent_name) <> ''"));
        assert!(TRACKING_SCHEMA.contains("btrim(model_version) <> ''"));
        assert!(TRACKING_SCHEMA.contains("deadline_at is null or deadline_at > started_at"));
        assert!(TRACKING_SCHEMA.contains("finished_at is null or finished_at >= started_at"));
        assert!(TRACKING_SCHEMA.contains("lifecycle_status in"));
        assert!(TRACKING_SCHEMA.contains("input_ref is null or btrim(input_ref) <> ''"));
        assert!(TRACKING_SCHEMA.contains("output_ref is null or btrim(output_ref) <> ''"));
        assert!(TRACKING_SCHEMA.contains("error is null or btrim(error) <> ''"));
        assert!(TRACKING_SCHEMA.contains("status = 'succeeded'"));
        assert!(TRACKING_SCHEMA.contains("output_ref is not null"));
        assert!(TRACKING_SCHEMA.contains("status = 'skipped'"));
        assert!(TRACKING_SCHEMA.contains("btrim(tool_name) <> ''"));
        assert!(TRACKING_SCHEMA.contains("btrim(tool_version) <> ''"));
        assert!(TRACKING_SCHEMA.contains("jsonb_typeof(input) = 'object'"));
        assert!(TRACKING_SCHEMA.contains("output is null or jsonb_typeof(output) = 'object'"));
        assert!(TRACKING_SCHEMA.contains("failure_source in"));
        assert!(TRACKING_SCHEMA.contains("failure_outcome in"));
        assert!(TRACKING_SCHEMA.contains("failure_outcome = 'retry_scheduled'"));
        assert!(TRACKING_SCHEMA.contains("failure_outcome = 'dead_lettered'"));
        assert!(TRACKING_SCHEMA.contains("failure_history_job_idx"));
        assert!(TRACKING_SCHEMA.contains("span_id is null or trace_id is not null"));
        assert!(TRACKING_SCHEMA.contains("completed_at is null or started_at is null"));
        assert!(TRACKING_SCHEMA.contains("status in ('requested', 'validated')"));
        assert!(
            TRACKING_SCHEMA
                .contains("status in ('requested', 'applied', 'ignored_terminal', 'expired')")
        );
        assert!(TRACKING_SCHEMA.contains("source in ('user', 'operator', 'system', 'policy')"));
        assert!(
            TRACKING_SCHEMA.contains("mode in ('graceful', 'immediate', 'after_current_step')")
        );
        assert!(TRACKING_SCHEMA.contains("status = 'applied'"));
        assert!(TRACKING_SCHEMA.contains("status = 'ignored_terminal'"));
        assert!(
            TRACKING_SCHEMA.contains(
                "status in ('requested', 'accepted', 'rejected', 'expired', 'cancelled')"
            )
        );
        assert!(TRACKING_SCHEMA.contains("check (from_agent <> to_agent)"));
        assert!(TRACKING_SCHEMA.contains("confidence >= 0 and confidence <= 1"));
        assert!(TRACKING_SCHEMA.contains("memory_horizon in ('short_term', 'long_term')"));
        assert!(TRACKING_SCHEMA.contains("retention_policy in ('ephemeral', 'session')"));
        assert!(TRACKING_SCHEMA.contains("retention_policy in ('durable', 'audit')"));
        assert!(
            TRACKING_SCHEMA
                .contains("request_kind in ('redaction', 'erasure', 'export', 'retention_review')")
        );
        assert!(
            TRACKING_SCHEMA
                .contains("status in ('requested', 'approved', 'applied', 'rejected', 'expired')")
        );
        assert!(TRACKING_SCHEMA.contains("data_protection_requests_status_due_idx"));
        assert!(TRACKING_SCHEMA.contains("data_protection_requests_surface_subject_idx"));
        assert!(TRACKING_SCHEMA.contains("trace_id ~ '^[0-9A-Fa-f]{32}$'"));
        assert!(TRACKING_SCHEMA.contains("span_id ~ '^[0-9A-Fa-f]{16}$'"));
        assert!(TRACKING_SCHEMA.contains("trace_id <> repeat('0', 32)"));
        assert!(TRACKING_SCHEMA.contains("span_id <> repeat('0', 16)"));
        assert!(TRACKING_SCHEMA.contains("operation_events_trace_idx"));
        assert!(TRACKING_SCHEMA.contains("total_tokens = prompt_tokens + completion_tokens"));
        assert!(TRACKING_SCHEMA.contains("cost_micros_usd bigint not null"));
        assert!(
            TRACKING_SCHEMA.contains("decision in ('authorized', 'requires_approval', 'denied')")
        );
        assert!(TRACKING_SCHEMA.contains("permission in"));
        assert!(TRACKING_SCHEMA.contains("filesystem_access in"));
        assert!(
            TRACKING_SCHEMA
                .contains("secret_access in ('none', 'tool_runtime_only', 'model_visible')")
        );
        assert!(TRACKING_SCHEMA.contains("create table credential_assets"));
        assert!(TRACKING_SCHEMA.contains("credential_kind in"));
        assert!(TRACKING_SCHEMA.contains(
            "status in ('active', 'rotation_due', 'rotating', 'revoked', 'compromised')"
        ));
        assert!(TRACKING_SCHEMA.contains("credential_assets_due_idx"));
        assert!(TRACKING_SCHEMA.contains("credential_assets_status_idx"));
        assert!(TRACKING_SCHEMA.contains("decision in ('allowed', 'denied')"));
        assert!(
            TRACKING_SCHEMA.contains("status in ('pending', 'publishing', 'published', 'failed')")
        );
        assert!(TRACKING_SCHEMA.contains(
            "status in ('requested', 'approved', 'executing', 'succeeded', 'failed', 'cancelled')"
        ));
        assert!(TRACKING_SCHEMA.contains("approval_request_id uuid"));
        assert!(TRACKING_SCHEMA.contains("references side_effect_receipts"));
        assert!(TRACKING_SCHEMA.contains("prompt_version text not null"));
        assert!(TRACKING_SCHEMA.contains("model_version text not null"));
        assert!(TRACKING_SCHEMA.contains("tool_version text not null"));
        assert!(TRACKING_SCHEMA.contains("policy_version text not null"));
        assert!(TRACKING_SCHEMA.contains("jsonb_typeof(report) = 'object'"));
        assert!(TRACKING_SCHEMA.contains("evaluation_runs_version_idx"));
        assert!(TRACKING_SCHEMA.contains("idempotency_key text not null unique"));
        assert!(TRACKING_SCHEMA.contains("rpo_seconds bigint not null"));
        assert!(TRACKING_SCHEMA.contains("scenario text not null"));
        assert!(TRACKING_SCHEMA.contains(
            "environment text not null check (environment in ('local', 'staging', 'production'))"
        ));
        assert!(
            TRACKING_SCHEMA.contains("status text not null check (status in ('planned', 'running', 'passed', 'failed', 'aborted'))")
        );
        assert!(TRACKING_SCHEMA.contains("required_evidence_count int not null"));
        assert!(TRACKING_SCHEMA.contains("observed_evidence_count int not null"));
        assert!(TRACKING_SCHEMA.contains("jsonb_typeof(observed_evidence) = 'array'"));
        assert!(TRACKING_SCHEMA.contains(
            "status <> 'passed'\n    or observed_evidence_count >= required_evidence_count"
        ));
        assert!(TRACKING_SCHEMA.contains("failure_drill_runs_status_idx"));
        assert!(TRACKING_SCHEMA.contains("operator_signoff text"));
        assert!(TRACKING_SCHEMA.contains("phase in ('expand', 'backfill', 'contract')"));
        assert!(
            TRACKING_SCHEMA
                .contains("status in ('planned', 'running', 'passed', 'failed', 'blocked')")
        );
        assert!(TRACKING_SCHEMA.contains("compatible_from_payload_schema_version"));
        assert!(TRACKING_SCHEMA.contains("compatible_through_payload_schema_version"));
        assert!(TRACKING_SCHEMA.contains("rows_changed <= rows_examined"));
        assert!(TRACKING_SCHEMA.contains("schema_migration_runs_status_idx"));
        assert!(TRACKING_SCHEMA.contains("candidate_id uuid not null"));
        assert!(TRACKING_SCHEMA.contains("decision in ('promote', 'canary_only', 'block')"));
        assert!(
            TRACKING_SCHEMA
                .contains("slo_decision in ('within_budget', 'no_traffic', 'budget_exhausted')")
        );
        assert!(TRACKING_SCHEMA.contains("compatibility_decision in ('process', 'quarantine')"));
        assert!(TRACKING_SCHEMA.contains("jsonb_typeof(blockers) = 'array'"));
        assert!(TRACKING_SCHEMA.contains("jsonb_array_length(blockers) = 0"));
        assert!(TRACKING_SCHEMA.contains("jsonb_array_length(blockers) > 0"));
        assert!(TRACKING_SCHEMA.contains("unique (candidate_id, gate_name)"));
        assert!(TRACKING_SCHEMA.contains("release_gate_runs_decision_idx"));
        assert!(TRACKING_SCHEMA.contains("create table job_kind_readiness_reviews"));
        assert!(TRACKING_SCHEMA.contains(
            "target_level in ('demo', 'prototype', 'production', 'regulated_high_risk')"
        ));
        assert!(TRACKING_SCHEMA.contains("risk_class in ('low', 'medium', 'high', 'regulated')"));
        assert!(TRACKING_SCHEMA.contains("evidence_ready_count <= evidence_required_count"));
        assert!(TRACKING_SCHEMA.contains("blocking_gap_count <= evidence_required_count"));
        assert!(TRACKING_SCHEMA.contains("job_kind_readiness_reviews_due_idx"));
        assert!(TRACKING_SCHEMA.contains("create table fault_tolerance_reviews"));
        assert!(
            TRACKING_SCHEMA
                .contains("control_plane_status in ('healthy', 'degraded', 'unavailable')")
        );
        assert!(
            TRACKING_SCHEMA.contains("execution_plane_status in ('serving', 'degraded', 'paused')")
        );
        assert!(TRACKING_SCHEMA.contains(
            "static_stability_mode in ('normal', 'last_known_good', 'draft_only', 'paused')"
        ));
        assert!(TRACKING_SCHEMA.contains(
            "progressive_delivery_channel in ('dev', 'canary', 'production', 'high_risk_hold')"
        ));
        assert!(TRACKING_SCHEMA.contains("redundant_worker_count >= 0"));
        assert!(TRACKING_SCHEMA.contains("minimum_redundant_workers > 0"));
        assert!(TRACKING_SCHEMA.contains("references failure_drill_runs"));
        assert!(TRACKING_SCHEMA.contains("references release_gate_runs"));
        assert!(TRACKING_SCHEMA.contains("fault_tolerance_reviews_due_idx"));
        assert!(TRACKING_SCHEMA.contains("fault_tolerance_reviews_status_idx"));
        assert!(TRACKING_SCHEMA.contains("create table human_escalations"));
        assert!(TRACKING_SCHEMA.contains("escalation_kind in"));
        assert!(TRACKING_SCHEMA.contains("severity in ('review', 'ticket', 'page')"));
        assert!(
            TRACKING_SCHEMA.contains("status in ('open', 'acknowledged', 'resolved', 'cancelled')")
        );
        assert!(TRACKING_SCHEMA.contains("check (job_id is not null or run_id is not null)"));
        assert!(TRACKING_SCHEMA.contains("human_escalations_open_idx"));
    }

    #[test]
    fn recovery_query_releases_expired_running_jobs() {
        assert!(RECOVER_EXPIRED_JOBS.contains("status = 'running'"));
        assert!(RECOVER_EXPIRED_JOBS.contains("locked_until < now()"));
        assert!(RECOVER_EXPIRED_JOBS.contains("status = 'pending'"));
    }

    #[test]
    fn fault_tolerance_query_surfaces_static_stability_and_failover_evidence() {
        assert!(FAULT_TOLERANCE_READINESS.contains("from fault_tolerance_reviews"));
        assert!(FAULT_TOLERANCE_READINESS.contains("left join failure_drill_runs"));
        assert!(FAULT_TOLERANCE_READINESS.contains("left join release_gate_runs"));
        assert!(FAULT_TOLERANCE_READINESS.contains("static_stability_mode"));
        assert!(FAULT_TOLERANCE_READINESS.contains("redundant_worker_count"));
        assert!(FAULT_TOLERANCE_READINESS.contains("failover_drill_status"));
        assert!(FAULT_TOLERANCE_READINESS.contains("control_plane_coupled"));
        assert!(FAULT_TOLERANCE_READINESS.contains("needs_redundancy"));
        assert!(FAULT_TOLERANCE_READINESS.contains("release_gate_missing"));
        assert!(FAULT_TOLERANCE_READINESS.contains("release_blocked"));
    }

    #[test]
    fn enqueue_query_suppresses_duplicate_idempotency_keys() {
        assert!(ENQUEUE_AGENT_JOB.contains("on conflict (idempotency_key) do nothing"));
        assert!(ENQUEUE_AGENT_JOB.contains("false as inserted"));
        assert!(ENQUEUE_AGENT_JOB.contains("payload_schema_version"));
        assert!(ENQUEUE_AGENT_JOB.contains("worker_build_id"));
    }

    #[test]
    fn admission_decision_query_preserves_intake_audit_evidence() {
        assert!(ADMIT_AGENT_JOB.contains("with inserted as"));
        assert!(ADMIT_AGENT_JOB.contains("insert into agent_jobs"));
        assert!(ADMIT_AGENT_JOB.contains("insert into agent_job_events"));
        assert!(ADMIT_AGENT_JOB.contains("insert into admission_decision_events"));
        assert!(ADMIT_AGENT_JOB.contains("duplicate_suppressed"));
        assert!(ADMIT_AGENT_JOB.contains("join admission_event_insert"));
        assert!(ADMIT_AGENT_JOB.contains("join agent_event_insert"));
        assert!(RECORD_ADMISSION_DECISION.contains("insert into admission_decision_events"));
        assert!(RECORD_ADMISSION_DECISION.contains("queue_pressure"));
        assert!(RECORD_ADMISSION_DECISION.contains("provider_pressure"));
        assert!(RECORD_ADMISSION_DECISION.contains("projected_cost_micros_usd"));
        assert!(RECORD_ADMISSION_DECISION.contains("remaining_budget_micros_usd"));
        assert!(RECORD_ADMISSION_DECISION.contains("budget_limit_micros_usd"));
        assert!(RECORD_ADMISSION_DECISION.contains("$14::timestamptz"));
    }

    #[test]
    fn existing_idempotency_query_records_duplicate_before_admission_pressure() {
        assert!(RESOLVE_EXISTING_AGENT_JOB.contains("with existing_job as"));
        assert!(RESOLVE_EXISTING_AGENT_JOB.contains("where idempotency_key = $1::text"));
        assert!(RESOLVE_EXISTING_AGENT_JOB.contains("insert into agent_job_events"));
        assert!(RESOLVE_EXISTING_AGENT_JOB.contains("duplicate_suppressed"));
        assert!(RESOLVE_EXISTING_AGENT_JOB.contains("select existing_job.id"));
        assert!(RESOLVE_EXISTING_AGENT_JOB.contains("existing_job.status"));
        assert!(RESOLVE_EXISTING_AGENT_JOB.contains("existing_job.run_at"));
    }

    #[test]
    fn lifecycle_queries_preserve_ownership_and_terminal_state_rules() {
        assert!(EXTEND_LEASE.contains("locked_by = $2::text"));
        assert!(MARK_SUCCEEDED.contains("status = 'running'"));
        assert!(MARK_SUCCEEDED.contains("locked_by = $3::text"));
        assert!(MARK_CANCELLED.contains("status in ('pending', 'running')"));
        assert!(RETRY_OR_DEAD.contains("locked_by = $5::text"));
    }

    #[test]
    fn retry_query_distinguishes_permanent_failure_from_retryable_failure() {
        assert!(RETRY_OR_DEAD.contains("$2::text = 'permanent'"));
        assert!(RETRY_OR_DEAD.contains("attempt_count >= max_attempts"));
        assert!(RETRY_OR_DEAD.contains("now() + $3::interval"));
    }

    #[test]
    fn metrics_query_exposes_queue_health() {
        assert!(QUEUE_METRICS.contains("oldest_pending_age_seconds"));
        assert!(QUEUE_METRICS.contains("count(*) filter"));
    }

    #[test]
    fn sli_queries_expose_budget_measurement_rows() {
        assert!(SLI_JOB_START_LATENCY.contains("'job-start-latency:v1'"));
        assert!(SLI_JOB_START_LATENCY.contains("'job_start_latency_within_120s'"));
        assert!(SLI_JOB_START_LATENCY.contains("9900::bigint as target_basis_points"));
        assert!(SLI_JOB_START_LATENCY.contains("good_events"));
        assert!(SLI_JOB_START_LATENCY.contains("total_events"));
        assert!(SLI_JOB_START_LATENCY.contains("agent_job_events"));
        assert!(SLI_JOB_START_LATENCY.contains("event_type = 'job_picked'"));
        assert!(SLI_TERMINAL_JOBS_NOT_DEAD.contains("'terminal-jobs-not-dead:v1'"));
        assert!(SLI_TERMINAL_JOBS_NOT_DEAD.contains("'terminal_jobs_not_dead'"));
        assert!(
            SLI_TERMINAL_JOBS_NOT_DEAD.contains("status in ('succeeded', 'dead', 'cancelled')")
        );
        assert!(SLI_TERMINAL_JOBS_NOT_DEAD.contains("count(*) filter (where status <> 'dead')"));
    }

    #[test]
    fn runbook_queries_cover_operator_diagnosis_and_control() {
        assert!(OLDEST_PENDING_JOB.contains("pending_age_seconds"));
        assert!(EXPIRED_LEASES.contains("expired_by_seconds"));
        assert!(DEAD_JOBS_BY_REASON.contains("group by kind"));
        assert!(JOB_EVENT_TIMELINE.contains(":'job_id'::uuid"));
        assert!(JOB_EVENT_TIMELINE.contains("order by created_at asc, id asc"));
        assert!(FAILURE_HISTORY_BY_JOB.contains("failure_history"));
        assert!(FAILURE_HISTORY_BY_JOB.contains(":'scheduled_job_id'::uuid"));
        assert!(FAILURE_HISTORY_BY_JOB.contains("failure_outcome"));
        assert!(FAILURE_HISTORY_BY_JOB.contains("order by failure_history.occurred_at desc"));
        assert!(VERSION_COMPATIBILITY_RISKS.contains(":'minimum_payload_schema_version'::int"));
        assert!(VERSION_COMPATIBILITY_RISKS.contains(":'maximum_payload_schema_version'::int"));
        assert!(VERSION_COMPATIBILITY_RISKS.contains("payload_schema_too_old"));
        assert!(VERSION_COMPATIBILITY_RISKS.contains("payload_schema_too_new"));
        assert!(VERSION_COMPATIBILITY_RISKS.contains("status in ('pending', 'running')"));
        assert!(SCHEMA_MIGRATION_STATUS.contains("schema_migration_runs"));
        assert!(SCHEMA_MIGRATION_STATUS.contains("phase"));
        assert!(
            SCHEMA_MIGRATION_STATUS
                .contains("status in ('planned', 'running', 'failed', 'blocked')")
        );
        assert!(SCHEMA_MIGRATION_STATUS.contains("changed_percent"));
        assert!(SCHEMA_MIGRATION_STATUS.contains("compatibility_query_name"));
        assert!(SCHEMA_MIGRATION_STATUS.contains("operator_signoff"));
        assert!(RUNNING_AGENT_RUNS.contains("lifecycle_status in"));
        assert!(RUNNING_AGENT_RUNS.contains("background_jobs.workflow_state"));
        assert!(PENDING_AGENT_HANDOFFS.contains("agent_handoffs"));
        assert!(PENDING_AGENT_HANDOFFS.contains("status = 'requested'"));
        assert!(PENDING_AGENT_HANDOFFS.contains("oldest_pending_age_seconds"));
        assert!(AGENT_MEMORY_BY_SCOPE.contains("agent_memory_records"));
        assert!(AGENT_MEMORY_BY_SCOPE.contains(":'memory_scope'"));
        assert!(AGENT_MEMORY_BY_SCOPE.contains("has_embedding_ref"));
        assert!(!AGENT_MEMORY_BY_SCOPE.contains("content"));
        assert!(PENDING_CANCELLATION_REQUESTS.contains("cancellation_requests"));
        assert!(PENDING_CANCELLATION_REQUESTS.contains("status = 'requested'"));
        assert!(PENDING_CANCELLATION_REQUESTS.contains("pending_age_seconds"));
        assert!(PENDING_CANCELLATION_REQUESTS.contains("scheduled_jobs.status as job_status"));
        assert!(SCHEDULED_RETRIES.contains("retry_state = 'waiting_for_retry'"));
        assert!(WAITING_HUMAN_APPROVALS.contains("status = 'requested'"));
        assert!(OPEN_HUMAN_ESCALATIONS.contains("human_escalations"));
        assert!(OPEN_HUMAN_ESCALATIONS.contains("status in ('open', 'acknowledged')"));
        assert!(OPEN_HUMAN_ESCALATIONS.contains("open_for_seconds"));
        assert!(OPEN_HUMAN_ESCALATIONS.contains("prompt_version"));
        assert!(FAILED_TOOL_CALLS.contains("status in ('failed', 'rejected')"));
        assert!(SIDE_EFFECT_RECEIPTS_BY_RUN.contains(":'run_id'::uuid"));
        assert!(SIDE_EFFECT_RECEIPTS_BY_RUN.contains("side_effect_receipts"));
        assert!(EVALUATION_RECEIPTS_BY_VERSION.contains("evaluation_runs"));
        assert!(EVALUATION_RECEIPTS_BY_VERSION.contains("score_basis_points"));
        assert!(EVALUATION_RECEIPTS_BY_VERSION.contains("prompt_version"));
        assert!(EVALUATION_RECEIPTS_BY_VERSION.contains("model_version"));
        assert!(EVALUATION_RECEIPTS_BY_VERSION.contains("tool_version"));
        assert!(EVALUATION_RECEIPTS_BY_VERSION.contains("policy_version"));
        assert!(EVALUATION_RECEIPTS_BY_VERSION.contains("has_failed_case_details"));
        assert!(EVALUATION_RECEIPTS_BY_VERSION.contains("limit 50"));
        assert!(RELEASE_GATE_STATUS.contains("release_gate_runs"));
        assert!(RELEASE_GATE_STATUS.contains("evaluation_status"));
        assert!(RELEASE_GATE_STATUS.contains("evaluation_score_basis_points"));
        assert!(RELEASE_GATE_STATUS.contains("slo_decision"));
        assert!(RELEASE_GATE_STATUS.contains("compatibility_decision"));
        assert!(RELEASE_GATE_STATUS.contains("migration_status"));
        assert!(RELEASE_GATE_STATUS.contains("approval_status"));
        assert!(RELEASE_GATE_STATUS.contains("blocker_count"));
        assert!(RELEASE_GATE_STATUS.contains("canary_percent"));
        assert!(RELEASE_GATE_STATUS.contains("operator_signoff"));
        assert!(PROVIDER_USAGE_BY_JOB_KIND.contains("provider_usage_events"));
        assert!(PROVIDER_USAGE_BY_JOB_KIND.contains("sum(cost_micros_usd)"));
        assert!(PROVIDER_USAGE_BY_JOB_KIND.contains("percentile_cont(0.95)"));
        assert!(DENIED_AUTHORIZATION_EVENTS.contains("authorization_events"));
        assert!(
            DENIED_AUTHORIZATION_EVENTS.contains("decision in ('denied', 'requires_approval')")
        );
        assert!(DENIED_AUTHORIZATION_EVENTS.contains("policy_version"));
        assert!(SANDBOX_POLICY_VIOLATIONS.contains("sandbox_events"));
        assert!(SANDBOX_POLICY_VIOLATIONS.contains("decision = 'denied'"));
        assert!(SANDBOX_POLICY_VIOLATIONS.contains("policy_version"));
        assert!(CREDENTIAL_ROTATION_REVIEW.contains("credential_assets"));
        assert!(CREDENTIAL_ROTATION_REVIEW.contains("credential_kinds"));
        assert!(CREDENTIAL_ROTATION_REVIEW.contains("open_exposure_incidents"));
        assert!(CREDENTIAL_ROTATION_REVIEW.contains("stale_verification"));
        assert!(CREDENTIAL_ROTATION_REVIEW.contains("'rotation_overdue'"));
        assert!(CREDENTIAL_ROTATION_REVIEW.contains("'credential_health_ok'"));
        assert!(AUDIT_EVENTS_BY_RUN.contains("audit_events"));
        assert!(AUDIT_EVENTS_BY_RUN.contains(":'run_id'::uuid"));
        assert!(AUDIT_EVENTS_BY_RUN.contains("order by created_at asc, id asc"));
        assert!(OPERATION_EVENTS_BY_JOB.contains("operation_events"));
        assert!(OPERATION_EVENTS_BY_JOB.contains(":'job_id'::uuid"));
        assert!(OPERATION_EVENTS_BY_JOB.contains("trace_id"));
        assert!(OPERATION_EVENTS_BY_JOB.contains("span_id"));
        assert!(OPERATION_EVENTS_BY_JOB.contains("severity"));
        assert!(RUNNING_JOBS_PAST_DEADLINE.contains("agent_runs.deadline_at < now()"));
        assert!(RUNNING_JOBS_PAST_DEADLINE.contains("timeout_policy_name"));
        assert!(RUNNING_JOBS_PAST_DEADLINE.contains("timeout_action"));
        assert!(RUNNING_JOBS_PAST_DEADLINE.contains("overdue_seconds"));
        assert!(RESTORE_REPLAY_CANDIDATES.contains("side_effect_receipts"));
        assert!(RESTORE_REPLAY_CANDIDATES.contains("quarantine_missing_receipt"));
        assert!(RESTORE_REPLAY_CANDIDATES.contains("resume_from_durable_state"));
        assert!(FAILURE_DRILL_STATUS.contains("failure_drill_runs"));
        assert!(FAILURE_DRILL_STATUS.contains("evidence_percent"));
        assert!(FAILURE_DRILL_STATUS.contains("hypothesis"));
        assert!(FAILURE_DRILL_STATUS.contains("blast_radius"));
        assert!(FAILURE_DRILL_STATUS.contains("rollback_action"));
        assert!(FAILURE_DRILL_STATUS.contains("operator_signoff"));
        assert!(STORAGE_PRESSURE_BY_TABLE.contains("pg_stat_user_tables"));
        assert!(STORAGE_PRESSURE_BY_TABLE.contains("n_dead_tup"));
        assert!(STORAGE_PRESSURE_BY_TABLE.contains("estimated_dead_row_percent"));
        assert!(STORAGE_PRESSURE_BY_TABLE.contains("pg_total_relation_size"));
        assert!(STORAGE_PRESSURE_BY_TABLE.contains("last_autovacuum"));
        assert!(STORAGE_PRESSURE_BY_TABLE.contains("'agent_memory_records'"));
        assert!(RETENTION_REVIEW_BY_SURFACE.contains("retention_surfaces"));
        assert!(RETENTION_REVIEW_BY_SURFACE.contains("older_than_90_days"));
        assert!(RETENTION_REVIEW_BY_SURFACE.contains("older_than_365_days"));
        assert!(RETENTION_REVIEW_BY_SURFACE.contains("side_effect_receipts"));
        assert!(RETENTION_REVIEW_BY_SURFACE.contains("agent_memory_records"));
        assert!(RETENTION_REVIEW_BY_SURFACE.contains("restore_drill_runs"));
        assert!(RETENTION_REVIEW_BY_SURFACE.contains("failure_drill_runs"));
        assert!(RETENTION_REVIEW_BY_SURFACE.contains("release_gate_runs"));
        assert!(DATA_PROTECTION_REVIEW.contains("data_protection_requests"));
        assert!(DATA_PROTECTION_REVIEW.contains("data_surfaces"));
        assert!(DATA_PROTECTION_REVIEW.contains("pending_redaction_requests"));
        assert!(DATA_PROTECTION_REVIEW.contains("pending_erasure_requests"));
        assert!(DATA_PROTECTION_REVIEW.contains("recently_applied_requests_30d"));
        assert!(DATA_PROTECTION_REVIEW.contains("'privacy_review_overdue'"));
        assert!(DATA_PROTECTION_REVIEW.contains("'no_open_privacy_work'"));
        assert!(CLAIM_OUTBOX_EVENTS.contains("for update of outbox_events skip locked"));
        assert!(CLAIM_OUTBOX_EVENTS.contains("status = 'publishing'"));
        assert!(MARK_OUTBOX_EVENT_PUBLISHED.contains("locked_by = $2::text"));
        assert!(MARK_OUTBOX_EVENT_PUBLISHED.contains("status = 'published'"));
        assert!(MARK_OUTBOX_EVENT_FAILED.contains("attempts >= max_attempts"));
        assert!(MARK_OUTBOX_EVENT_FAILED.contains("then 'failed'"));
        assert!(OUTBOX_BACKLOG.contains("expired_publication_leases"));
        assert!(OUTBOX_BACKLOG.contains("oldest_due_age_seconds"));
        assert!(OUTBOX_BACKLOG.contains(
            "extract(epoch from now() - min(next_attempt_at) filter (where status = 'pending'))"
        ));
        assert!(APPROVE_COMPENSATION_ACTION.contains("human_approval_requests"));
        assert!(APPROVE_COMPENSATION_ACTION.contains("status = 'approved'"));
        assert!(
            CLAIM_COMPENSATION_ACTIONS.contains("for update of compensation_actions skip locked")
        );
        assert!(CLAIM_COMPENSATION_ACTIONS.contains("status = 'executing'"));
        assert!(MARK_COMPENSATION_SUCCEEDED.contains("locked_by = $2::text"));
        assert!(MARK_COMPENSATION_SUCCEEDED.contains("status = 'succeeded'"));
        assert!(MARK_COMPENSATION_FAILED.contains("attempts >= max_attempts"));
        assert!(MARK_COMPENSATION_FAILED.contains("then 'failed'"));
        assert!(COMPENSATION_BACKLOG.contains("expired_execution_leases"));
        assert!(COMPENSATION_BACKLOG.contains("oldest_due_age_seconds"));
        assert!(COMPENSATION_BACKLOG.contains(
            "extract(epoch from now() - min(next_attempt_at) filter (where status = 'approved'))"
        ));
        assert!(QUEUE_METRICS_BY_KIND.contains("group by kind"));
        assert!(SLI_JOB_START_LATENCY.contains("window_started_at"));
        assert!(SLI_TERMINAL_JOBS_NOT_DEAD.contains("window_ended_at"));
        assert!(PAUSE_JOB_KIND.contains(":'kind'::text"));
        assert!(PAUSE_JOB_KIND.contains(":'actor'::text"));
        assert!(PAUSE_JOB_KIND.contains(":'reason'::text"));
        assert!(PAUSE_JOB_KIND.contains("paused = true"));
        assert!(PAUSE_JOB_KIND.contains("insert into agent_job_kind_control_events"));
        assert!(PAUSE_JOB_KIND.contains("'paused'"));
        assert!(PAUSE_JOB_KIND.contains("previous_paused"));
        assert!(PAUSE_JOB_KIND.contains("control_event_id"));
        assert!(RESUME_JOB_KIND.contains(":'kind'::text"));
        assert!(RESUME_JOB_KIND.contains(":'actor'::text"));
        assert!(RESUME_JOB_KIND.contains(":'reason'::text"));
        assert!(RESUME_JOB_KIND.contains("paused = false"));
        assert!(RESUME_JOB_KIND.contains("insert into agent_job_kind_control_events"));
        assert!(RESUME_JOB_KIND.contains("'resumed'"));
        assert!(RESUME_JOB_KIND.contains("previous_paused"));
        assert!(RESUME_JOB_KIND.contains("control_event_id"));
        assert!(JOB_KIND_LIFECYCLE_REVIEW.contains("known_job_kinds"));
        assert!(JOB_KIND_LIFECYCLE_REVIEW.contains("recent_provider_calls_30d"));
        assert!(JOB_KIND_LIFECYCLE_REVIEW.contains("latest_release_decision"));
        assert!(JOB_KIND_LIFECYCLE_REVIEW.contains("'deprecation_candidate'"));
        assert!(JOB_KIND_LIFECYCLE_REVIEW.contains("'retirement_candidate'"));
        assert!(JOB_KIND_LIFECYCLE_REVIEW.contains("'retirement_blocked'"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("job_kind_readiness_reviews"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("target_level"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("current_level"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("risk_class"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("evidence_ready_count"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("evidence_required_count"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("blocking_gap_count"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("review_overdue"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("latest_release_decision"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("'ready_for_target'"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("'missing_evidence'"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("'blocked_by_gaps'"));
        assert!(JOB_KIND_READINESS_REVIEW.contains("'review_overdue'"));
    }
}
