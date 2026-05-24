#!/usr/bin/env python3
"""Check explicit coverage of the Reliable AI Agents project objective.

This checker is intentionally higher level than the prose, link, and Rust
hygiene gates. It protects the book against drifting away from the user's core
objective: a serious Rust, Postgres, and Rig production manual for reliable
long-running AI agents.
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
README = ROOT / "README.md"
BOOK_DIR = ROOT / "books" / "postgres-rig-agent-jobs"
BOOK_SRC = BOOK_DIR / "src"
SUMMARY = BOOK_SRC / "SUMMARY.md"
COVER = BOOK_SRC / "cover.md"
PRIVATE_AUTHORING = Path.home() / ".khowlege" / "reliable-ai-agents-private" / "authoring"
STYLE_GUIDE = PRIVATE_AUTHORING / "STYLE_GUIDE.md"
QUALITY = PRIVATE_AUTHORING / "QUALITY.md"
TRACEABILITY = BOOK_SRC / "48-production-requirement-traceability.md"
CASE_STUDIES = BOOK_SRC / "41-production-case-studies.md"
FIRST_PRODUCTION_DEPLOYMENT_PROOF = BOOK_SRC / "first-production-deployment-proof.md"
PRODUCTION_MICRO_DRILLS = BOOK_SRC / "production-micro-drills.md"
PRODUCTION_BUILD_MILESTONES = BOOK_SRC / "production-build-milestones.md"
FAILURE_FIRST_LEARNING_MAP = BOOK_SRC / "failure-first-learning-map.md"
SOURCES = BOOK_SRC / "31-credible-resources-further-reading.md"
GLOSSARY = BOOK_SRC / "32-glossary-invariant-index.md"
PRODUCTION_EVIDENCE_PACKETS = BOOK_SRC / "44-production-evidence-packets.md"
DESIGN_SMELLS = BOOK_SRC / "46-design-smells-failure-mode-index.md"
FORMAL_DEFINITION_LEDGER = BOOK_SRC / "49-formal-definition-ledger.md"
OPERATOR_CONTROL_SURFACE = BOOK_SRC / "51-operator-control-surface.md"
READINESS = ROOT / "scripts" / "check-book-readiness.sh"
PUBLIC_REPO_SURFACE = ROOT / "scripts" / "check-public-repo-surface.py"
PUBLIC_CHAPTER_STRUCTURE = ROOT / "scripts" / "check-public-chapter-structure.py"
DEPENDENCY_POLICY = ROOT / "scripts" / "check-cargo-dependency-policy.py"
POSTGRES_SCHEMA_CONTRACT = ROOT / "scripts" / "check-postgres-schema-contract.py"
BOOK_CODE_CONTRACT = ROOT / "scripts" / "check-book-code-contract.py"
RUST_BOUNDARY_TYPES = ROOT / "scripts" / "check-rust-boundary-types.py"
CRATE = ROOT / "examples" / "postgres-rig-agent-jobs"
CARGO = CRATE / "Cargo.toml"
SRC = CRATE / "src"
SQL = CRATE / "sql"


def normalize(text: str) -> str:
    return " ".join(text.lower().split())


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def rel(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def require_file(path: Path, failures: list[str]) -> None:
    if not path.is_file():
        failures.append(f"missing required objective artifact: {rel(path)}")


def require_phrases(path: Path, objective: str, phrases: tuple[str, ...], failures: list[str]) -> None:
    if not path.is_file():
        failures.append(f"{objective}: missing file {rel(path)}")
        return

    text = read(path)
    inline = normalize(text)
    for phrase in phrases:
        if phrase not in text and normalize(phrase) not in inline:
            failures.append(f"{objective}: {rel(path)} missing phrase: {phrase}")


def require_any(path: Path, objective: str, alternatives: tuple[str, ...], failures: list[str]) -> None:
    if not path.is_file():
        failures.append(f"{objective}: missing file {rel(path)}")
        return

    text = read(path)
    inline = normalize(text)
    if not any(phrase in text or normalize(phrase) in inline for phrase in alternatives):
        failures.append(
            f"{objective}: {rel(path)} missing one of: {', '.join(alternatives)}"
        )


def check_book_shape(failures: list[str]) -> None:
    require_phrases(
        SUMMARY,
        "coherent book structure",
        (
            "# Part I: The Core System",
            "# Part II: Production Engineering",
            "# Part III: Operating The System",
            "# Part IV: World-Class Reliability",
            "30b-scaling-paths-after-postgres-first.md",
            "30c-temporal-after-postgres-first.md",
            "30d-kafka-after-postgres-first.md",
            "48-production-requirement-traceability.md",
            "attention-friendly-production-learning.md",
            "chapter-card-pack.md",
            "first-production-deployment-proof.md",
            "plain-language-production-cards.md",
            "production-micro-drills.md",
            "production-build-milestones.md",
            "failure-first-learning-map.md",
        ),
        failures,
    )


def check_public_draft_reuse_note(failures: list[str]) -> None:
    require_phrases(
        COVER,
        "public draft and reuse note",
        (
            "Draft status:",
            "License:",
            "Individual learning is allowed",
            "Non-commercial and nonprofit",
            "reuse of limited portions is allowed",
            "Commercial reproduction",
            "company team training",
            "business pipeline requires a separate written license",
            "examples are teaching artifacts",
            "not a substitute for testing",
            "own system",
        ),
        failures,
    )

    require_phrases(
        README,
        "public draft reuse note",
        (
            "## License And Reuse",
            "Individual learning is allowed",
            "Non-commercial and nonprofit reuse of limited portions is allowed",
            "Commercial reproduction, consulting use, client delivery, paid training",
            "company team training, internal business enablement",
            "business pipeline requires a separate written license",
            "See [LICENSE.md](LICENSE.md) for the full license terms",
            "each real system still needs its own tests, operational checks, security review, legal review, and evidence before deployment",
        ),
        failures,
    )

    require_phrases(
        ROOT / "LICENSE.md",
        "strict content license",
        (
            "Individual Learning Permission",
            "Non-Commercial And Nonprofit Reuse",
            "Written License Required For Commercial Use",
            "consulting work",
            "company team training",
            "business pipeline or revenue-generating use",
            "No commercial license is granted by this public repository",
        ),
        failures,
    )


def check_architectural_thesis(failures: list[str]) -> None:
    require_phrases(
        README,
        "book identity and default stack",
        (
            "production-grade AI agents on Rust, Rig, and Postgres",
            "long-running production system",
            "Public Repository Boundary",
            "learner-facing source, companion code, SQL, scripts, and CI configuration",
            "python3 scripts/check-public-repo-surface.py",
            "Run The Full Readiness Gate",
            "Run The Local Ephemeral Postgres Gate",
            "Run The Live DeepSeek Gate",
            "first production deployment proof",
        ),
        failures,
    )

    require_phrases(
        BOOK_SRC / "00d-production-scope-trade-offs.md",
        "minimal infrastructure operating envelope",
        (
            "Postgres-first",
            "workflow engine",
            "queue framework",
            "operating envelope",
            "when Postgres-first should stop owning orchestration",
        ),
        failures,
    )

    require_phrases(
        BOOK_SRC / "20-final-production-blueprint.md",
        "serious MVP blueprint",
        (
            "serious MVP architecture: one Rust API, one Rust worker, Postgres, Rig, traces, approvals, and runbooks",
            "API, Postgres, worker, Rig boundary, policy, approval, side effects, and operations each have one clear responsibility",
            "build the serious MVP before buying complexity",
        ),
        failures,
    )

    require_phrases(
        CARGO,
        "minimal Rust dependency stack",
        (
            "tokio",
            "sqlx-postgres",
            "serde",
            "thiserror",
            "tracing",
            "tracing-subscriber",
            "uuid",
            "chrono",
            "axum",
            "rig-core",
        ),
        failures,
    )


def check_postgres_coordination(failures: list[str]) -> None:
    for path in (
        POSTGRES_SCHEMA_CONTRACT,
        SQL / "001_agent_jobs.sql",
        SQL / "002_agent_tracking.sql",
        SQL / "claim_scheduled_jobs.sql",
        SQL / "retry_or_dead.sql",
        SQL / "extend_lease.sql",
        SQL / "audit_events_by_run.sql",
        SQL / "operation_events_by_job.sql",
        SQL / "waiting_human_approvals.sql",
        SQL / "evaluation_receipts_by_version.sql",
        SQL / "release_gate_status.sql",
        SQL / "agent_memory_by_scope.sql",
        SQL / "pending_agent_handoffs.sql",
        SQL / "restore_replay_candidates.sql",
        SQL / "failure_drill_status.sql",
        SQL / "schema_migration_status.sql",
        SQL / "job_kind_lifecycle_review.sql",
        SQL / "job_kind_readiness_review.sql",
        SQL / "storage_pressure_by_table.sql",
        SQL / "retention_review_by_surface.sql",
        SQL / "credential_rotation_review.sql",
        SQL / "data_protection_review.sql",
        SQL / "temporal_workflow_reconciliation.sql",
        SQL / "kafka_replay_safety_by_event.sql",
        BOOK_CODE_CONTRACT,
        RUST_BOUNDARY_TYPES,
    ):
        require_file(path, failures)

    require_phrases(
        SQL / "002_agent_tracking.sql",
        "Postgres production tracking schema",
        (
            "create table scheduled_jobs",
            "create table background_jobs",
            "create table agent_runs",
            "create table agent_steps",
            "create table tool_calls",
            "create table failure_history",
            "create table audit_events",
            "create table operation_events",
            "create table human_approval_requests",
            "create table agent_handoffs",
            "create table evaluation_runs",
            "create table agent_memory_records",
            "create table data_protection_requests",
            "create table outbox_events",
            "create table compensation_actions",
            "create table provider_usage_events",
            "create table authorization_events",
            "create table sandbox_events",
            "create table credential_assets",
            "create table restore_drill_runs",
            "create table failure_drill_runs",
            "create table schema_migration_runs",
            "create table release_gate_runs",
            "create table job_kind_readiness_reviews",
        ),
        failures,
    )

    require_phrases(
        POSTGRES_SCHEMA_CONTRACT,
        "Postgres schema contract checker",
        (
            "Check the Postgres coordination schema promised by the book objective.",
            "scheduled_jobs",
            "background_jobs",
            "agent_runs",
            "agent_steps",
            "tool_calls",
            "audit_events",
            "operation_events",
            "human_approval_requests",
            "evaluation_runs",
            "agent_memory_records",
            "jsonb_typeof(payload) = 'object'",
            "idempotency_key text not null unique",
            "Postgres schema contract check passed",
        ),
        failures,
    )

    require_phrases(
        BOOK_SRC / "03-postgres-ledger.md",
        "Postgres ledger teaching path",
        (
            "Postgres is the first durable memory and coordination layer",
            "FOR UPDATE SKIP LOCKED",
            "scheduled_jobs",
            "background_jobs",
            "agent_runs",
            "tool_calls",
        ),
        failures,
    )

    require_phrases(
        BOOK_CODE_CONTRACT,
        "Book code example contract checker",
        (
            "ignored Rust snippets must not become",
            "RUST_SOURCE_ROOT",
            "SQL_SOURCE_ROOT",
            "fence_options",
            "known_cargo_bins",
            "documented_manifest_paths",
            "documented_bin_names",
            "unknown Cargo binary",
            "ANCHOR: {anchor}",
            "book code contract check passed",
        ),
        failures,
    )

    require_phrases(
        RUST_BOUNDARY_TYPES,
        "Rust raw-boundary type checker",
        (
            "Reject raw primitive leakage",
            "raw outside, typed inside",
            "BOUNDARY_STRUCT_NAME_RE",
            "RAW_DIRECT_PARAM_RE",
            "rust boundary type check passed",
        ),
        failures,
    )

    require_phrases(
        SQL / "pending_agent_handoffs.sql",
        "Postgres pending handoff diagnostics",
        (
            "agent_handoffs",
            "to_agent",
            "pending_handoffs",
            "oldest_requested_at",
            "oldest_pending_age_seconds",
            "status = 'requested'",
        ),
        failures,
    )

    require_phrases(
        SQL / "schema_migration_status.sql",
        "Postgres schema migration evidence query",
        (
            "schema_migration_runs",
            "changed_percent",
            "compatibility_query_name",
            "operator_signoff",
        ),
        failures,
    )

    require_phrases(
        SQL / "failure_drill_status.sql",
        "Postgres failure drill evidence query",
        (
            "failure_drill_runs",
            "evidence_percent",
            "hypothesis",
            "blast_radius",
            "rollback_action",
            "operator_signoff",
        ),
        failures,
    )

    require_phrases(
        SQL / "release_gate_status.sql",
        "Postgres release gate evidence query",
        (
            "release_gate_runs",
            "evaluation_status",
            "slo_decision",
            "compatibility_decision",
            "migration_status",
            "approval_status",
            "blocker_count",
            "canary_percent",
            "rollback_plan",
            "operator_signoff",
        ),
        failures,
    )

    require_phrases(
        SQL / "job_kind_lifecycle_review.sql",
        "Postgres job-kind lifecycle review query",
        (
            "known_job_kinds",
            "recent_provider_calls_30d",
            "latest_release_decision",
            "deprecation_candidate",
            "retirement_candidate",
            "retirement_blocked",
        ),
        failures,
    )

    require_phrases(
        SQL / "job_kind_readiness_review.sql",
        "Postgres job-kind readiness review query",
        (
            "job_kind_readiness_reviews",
            "target_level",
            "current_level",
            "risk_class",
            "evidence_ready_count",
            "evidence_required_count",
            "blocking_gap_count",
            "review_overdue",
            "latest_release_decision",
            "'ready_for_target'",
            "'missing_evidence'",
            "'blocked_by_gaps'",
            "'review_overdue'",
        ),
        failures,
    )

    require_phrases(
        SQL / "storage_pressure_by_table.sql",
        "Postgres long-horizon storage pressure query",
        (
            "pg_stat_user_tables",
            "estimated_dead_row_percent",
            "pg_total_relation_size",
            "last_autovacuum",
        ),
        failures,
    )

    require_phrases(
        SQL / "retention_review_by_surface.sql",
        "Postgres long-horizon retention review query",
        (
            "retention_surfaces",
            "older_than_90_days",
            "older_than_365_days",
            "agent_memory_records",
            "restore_drill_runs",
            "failure_drill_runs",
            "release_gate_runs",
        ),
        failures,
    )

    require_phrases(
        SQL / "credential_rotation_review.sql",
        "Postgres credential rotation review query",
        (
            "credential_assets",
            "credential_kinds",
            "managed_credentials",
            "rotation_due",
            "overdue_rotation",
            "open_exposure_incidents",
            "stale_verification",
            "exposure_incident_open",
            "credential_health_ok",
        ),
        failures,
    )

    require_phrases(
        SQL / "data_protection_review.sql",
        "Postgres data-protection review query",
        (
            "data_protection_requests",
            "data_surfaces",
            "open_requests",
            "overdue_requests",
            "pending_redaction_requests",
            "pending_erasure_requests",
            "privacy_review_overdue",
            "no_open_privacy_work",
        ),
        failures,
    )


def check_rust_and_rig_boundaries(failures: list[str]) -> None:
    for path in (
        SRC / "domain.rs",
        SRC / "typed_pipeline.rs",
        SRC / "agent_output.rs",
        SRC / "tool_contract.rs",
        SRC / "tool_execution_gate.rs",
        SRC / "rig_runner.rs",
        SRC / "postgres_store.rs",
        SRC / "worker.rs",
        SRC / "logging.rs",
        SRC / "approval.rs",
        SRC / "agent_memory.rs",
        SRC / "job_kind_readiness.rs",
        SRC / "credential_lifecycle.rs",
        SRC / "data_protection.rs",
    ):
        require_file(path, failures)

    require_phrases(
        SRC / "logging.rs",
        "runtime tracing startup boundary",
        (
            "pub const RUST_LOG_ENV",
            "pub const LOG_FORMAT_ENV",
            "pub struct RuntimeTracingConfig",
            "pub enum RuntimeLogFormat",
            "pub fn init_runtime_tracing",
            "EnvFilter::try_new",
            "RuntimeLogFormat::Json",
            "try_init()",
        ),
        failures,
    )

    require_phrases(
        SRC / "domain.rs",
        "newtype-based domain model",
        (
            "pub struct JobId",
            "pub struct AgentRunId",
            "pub struct WorkerId",
            "pub struct TenantKey",
            "pub struct PromptVersion",
            "pub struct ModelRoute",
            "pub struct ToolVersion",
            "pub struct PolicyVersion",
            "pub struct IdempotencyKey",
            "pub enum JobStatus",
        ),
        failures,
    )

    require_phrases(
        SRC / "typed_pipeline.rs",
        "typestate construction path",
        (
            "AgentJobBuilder<NeedsKind>",
            "NeedsInstruction",
            "ReadyToEnqueue",
            "with_idempotency_key",
            "typestate_builder_only_builds_after_required_fields_are_present",
        ),
        failures,
    )

    require_phrases(
        SRC / "tool_contract.rs",
        "typed tool contract and model-output pipeline",
        (
            "pub struct RawModelOutput",
            "pub struct ToolInput<T>",
            "pub struct ToolOutput<T>",
            "pub trait TypedTool",
            "#[serde(deny_unknown_fields)]",
            "PolicyCheckedToolRequest",
            "ApprovedToolRequest",
        ),
        failures,
    )

    require_phrases(
        SRC / "rig_runner.rs",
        "Rig DeepSeek boundary",
        (
            "DeepSeekRigAgentRunner",
            "deepseek::Client::from_env()",
            "RawAgentOutput::new(provider_output)?.parse_agent_result()?",
            "Return only a strict JSON object",
        ),
        failures,
    )

    require_phrases(
        SRC / "job_kind_readiness.rs",
        "typed job-kind readiness boundary",
        (
            "pub enum MaturityLevel",
            "pub enum JobRiskClass",
            "pub enum JobKindReadinessStatus",
            "pub struct ReadinessEvidence",
            "pub struct JobKindReadinessReview",
            "pub struct DbJobKindReadinessReviewRow",
            "regulated_high_risk",
            "TargetLevelTooLow",
            "InconsistentJobKindReadinessStatus",
        ),
        failures,
    )

    require_phrases(
        SRC / "launch_packet.rs",
        "typed first-user launch packet boundary",
        (
            "pub enum LaunchDecision",
            "pub enum LaunchStatus",
            "pub enum FailureDrillStatus",
            "pub struct LaunchEvidenceChecklist",
            "pub struct LaunchKnownGaps",
            "pub struct DbJobKindLaunchPacketStatusRow",
            "pub struct JobKindLaunchPacket",
            "ApprovedLaunchMissingEvidence",
            "HighRiskLaunchRequiresPassedFailureDrill",
            "InconsistentLaunchStatus",
            "launch_packet_row_boundary",
        ),
        failures,
    )

    require_phrases(
        BOOK_SRC / "06-rig-boundary.md",
        "Rig is not the reliability layer",
        (
            "Use Rig for model and tool interaction, but keep reliability rules in your application boundary",
            "## Three-Layer Contract",
            "Agent intelligence layer",
            "Reliability layer",
            "Product/control layer",
            "Rig helps the agent think and propose.",
            "Postgres helps the system remember and recover.",
            "Rust makes the boundary explicit.",
            "DeepSeek through `DEEPSEEK_API_KEY`",
            "RawModelOutput",
            "PolicyCheckedToolRequest",
            "ApprovedToolRequest",
            "Rig gives the agent hands, but it should not own the reliability system",
        ),
        failures,
    )


def check_reliability_controls(failures: list[str]) -> None:
    for module, phrases in {
        "worker.rs": (
            "run_once_with_heartbeats",
            "run_bounded_loop",
            "worker drain requested; skipping new job claim",
            "worker heartbeat extended job lease",
        ),
        "admission_control.rs": (
            "AdmissionPolicy",
            "QueuePressure",
            "ProviderQuotaPressure",
            "BudgetAdmissionState",
        ),
        "outbox.rs": ("OutboxEventKind", "PendingOutboxEvent", "PublishedOutboxEvent"),
        "compensation.rs": ("CompensationActionId", "ApprovedCompensationAction", "ExecutingCompensationAction"),
        "cancellation.rs": ("CancellationRequest", "CancellationApplied", "CancellationExpired"),
        "timeouts.rs": ("TimeoutPolicy", "TimeoutAction", "ExecutionDeadline"),
        "handoff.rs": (
            "HandoffEnvelope",
            "RequestedHandoff",
            "AcceptedHandoff",
            "DbAgentHandoffRow",
            "handoff_row_boundary",
            "row_conversion_rejects_same_agent_handoff",
        ),
        "slo.rs": ("SloMeasurement", "SloDecision", "ErrorBudgetEventCount"),
        "evaluation.rs": ("EvaluationReceipt", "PromotionDecision", "EvaluationDatasetVersion"),
        "release_gate.rs": (
            "ReleaseGateReport",
            "ReleaseGateDecision",
            "ReleaseBlocker",
            "DbReleaseGateRunRow",
            "ReleaseGateRunReceipt",
            "release_gate_row_conversion_accepts_valid_promotion_receipt",
        ),
        "job_kind_lifecycle.rs": (
            "JobKindLifecycleReview",
            "JobKindLifecycleRecommendation",
            "DbJobKindLifecycleReviewRow",
            "row_conversion_rejects_retirement_candidate_with_open_work",
        ),
        "fault_tolerance.rs": (
            "FaultToleranceReadinessReview",
            "StaticStabilityMode",
            "DbFaultToleranceReadinessRow",
            "row_conversion_rejects_ready_without_required_redundancy",
        ),
        "recovery.rs": ("RestoreDrillReport", "ReplayDecision", "ReplayCandidate"),
        "security.rs": ("AuthorizationPolicy", "SecretRef", "AuthorizationDecisionKind"),
        "sandbox.rs": ("ToolSandboxPolicy", "SandboxDecisionEvent", "EgressAllowlist"),
    }.items():
        require_phrases(SRC / module, f"reliability control module {module}", phrases, failures)


def check_raw_outside_typed_inside_and_pedagogy(failures: list[str]) -> None:
    require_phrases(
        BOOK_SRC / "overview.md",
        "book-level transformation doctrine",
        (
            "The model may guess. The system must know.",
            "engineering transformations, not around tools",
            "from raw input to trusted domain data",
            "from model text to validated agent intent",
            "from tool call to permissioned side effect",
            "naive demo -> failure -> intuition -> typed model -> minimal implementation -> production hardening -> tests and evals -> operational judgment",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "00-how-to-read-this-book.md",
        "reader capability transformation ladder",
        (
            "Read each chapter as an engineering transformation, not as a tool lesson.",
            "recognition:",
            "understanding:",
            "implementation:",
            "judgment:",
            "The model may guess. The system must know.",
            "| raw user text | typed domain request |",
            "| tool call | permissioned side effect |",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "00c-design-principles.md",
        "reliable agent laws",
        (
            "## Reliable Agent Laws",
            "The model may propose. The system must decide.",
            "Raw model output is not domain truth.",
            "Every side effect needs identity.",
            "A reliable agent is a probabilistic core inside a deterministic shell.",
            "Do not confuse model intention with system permission.",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "40-principle-to-chapter-map.md",
        "engineering transformation map",
        (
            "## Engineering Transformation Map",
            "`agent.run(\"do the task\")`",
            "| raw model output | validated agent intent |",
            "| retry | idempotent execution |",
            "recognizes the failure, then explains the invariant, then builds the artifact",
        ),
        failures,
    )
    require_phrases(
        FAILURE_FIRST_LEARNING_MAP,
        "failure-first production learning map",
        (
            "failure -> false fix -> invariant -> artifact -> proof",
            "agent.run(\"do the task\")",
            "| Chapter | Production failure | False fix | Invariant | Proof artifact |",
            "| 12. Idempotency And Side Effects |",
            "| 16. Human Approval And Policy Gates |",
            "| 27.5 Agent Memory, Retrieval, And Retention |",
            "| 30.5 Scaling Paths After Postgres-First |",
            "| 30.6 Temporal After Postgres-First |",
            "| 30.7 Kafka After Postgres-First |",
            "from \"the agent ran\" to \"the system can prove what happened\"",
            "The model may guess. The system must know.",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "04-rust-domain-model.md",
        "raw outside typed inside teaching",
        (
            "raw strings and numbers hide category errors",
            "newtypes, enums, and constructors",
            "Judgment: Do Not Type Everything",
            "Do not add a newtype for every local variable.",
            "If this value is wrong, can it create the wrong state",
            "Raw data types are forbidden at boundaries",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "04b-typed-composition-lens.md",
        "composition and typestate teaching",
        (
            "Use category theory as a compact way to think, not as decoration",
            "newtypes: default for meaningful boundary values",
            "typestate: selective for construction order and lifecycle gates",
            "category theory: teaching lens for composition, identity, and error flow",
            "Judgment: When Not To Use Typestate",
            "When typestate makes every function generic and nobody can explain the",
            "Do not make the production API speak math",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "attention-friendly-production-learning.md",
        "attention-friendly no-compromise pedagogy",
        (
            "one concept\none artifact\none proof\none pause",
            "plain phrase -> formal term -> production artifact -> proof",
            "work took too long",
            "stop was requested",
            "safe correction after a side effect",
            "practice failure safely",
            "Method Map For Attention-Friendly Rigor",
            "Fifteen-Minute Production Sprint",
            "small step\nreal artifact\nvisible proof\nshort pause",
            "now: the one concept or artifact I am looking at",
            "done: the proof that lets me stop",
            "read: understand one concept",
            "operate: prove one behavior",
            "watch one\ncomplete one\nprove one",
            "Small Action, Fast Feedback",
            "act: do one small action",
            "check: run or inspect one concrete proof",
            "Do not move to a second abstraction until the first check is real.",
            "one hard term\none plain sentence\none artifact\none proof",
            "Simple Language Without Lowering Rigor",
            "The No-Compromise Gate",
            "Run the validation gate",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "plain-language-production-cards.md",
        "simplest accurate language with production evidence",
        (
            "say it small\nmake it exact\nprove it",
            "thing -> move -> evidence -> promise",
            "| Work must exist before the model starts. | Durable agent job |",
            "| Model text is not trusted state. | Raw model output |",
            "| The model cannot grant itself permission. | Trust boundary |",
            "Can this sentence tell me what to inspect?",
            "Can this sentence tell me what evidence proves the claim?",
            "Invariant: every simple sentence about a reliable agent should still point",
        ),
        failures,
    )
    require_phrases(
        PRODUCTION_MICRO_DRILLS,
        "production micro-drills for fast feedback",
        (
            "Appendix AA. Production Micro-Drills",
            "small drill\nreal artifact\none proof sentence\nstop or repair",
            "one action, one check, and one proof sentence",
            "| Durable intake |",
            "| Job claim |",
            "| Timeout policy |",
            "| Cancellation request |",
            "| Tool contract |",
            "| Human escalation |",
            "| Evaluation receipt |",
            "| Release gate |",
            "| Job-kind readiness review |",
            "| Job-kind launch packet |",
            "| Agent handoff |",
            "| Compensation action |",
            "| Cost and capacity guard |",
            "| Security boundary |",
            "| Injection and exfiltration guard |",
            "| Agent memory |",
            "| Credential lifecycle review |",
            "| Restore replay |",
            "| Failure drill |",
            "| Temporal adoption decision |",
            "| Kafka adoption decision |",
            "## Exact Artifact Index",
            "examples/postgres-rig-agent-jobs/sql/enqueue_agent_job.sql",
            "examples/postgres-rig-agent-jobs/src/tool_contract.rs",
            "examples/postgres-rig-agent-jobs/src/escalation.rs",
            "examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql",
            "examples/postgres-rig-agent-jobs/src/timeouts.rs",
            "examples/postgres-rig-agent-jobs/src/cancellation.rs",
            "examples/postgres-rig-agent-jobs/src/handoff.rs",
            "examples/postgres-rig-agent-jobs/src/compensation.rs",
            "examples/postgres-rig-agent-jobs/src/admission_control.rs",
            "examples/postgres-rig-agent-jobs/src/cost_accounting.rs",
            "examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql",
            "examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql",
            "examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql",
            "examples/postgres-rig-agent-jobs/src/tool_execution_gate.rs",
            "examples/postgres-rig-agent-jobs/sql/denied_authorization_events.sql",
            "examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql",
            "examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql",
            "examples/postgres-rig-agent-jobs/sql/compensation_backlog.sql",
            "examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql",
            "examples/postgres-rig-agent-jobs/src/agent_memory.rs",
            "examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql",
            "examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql",
            "examples/postgres-rig-agent-jobs/src/job_kind_readiness.rs",
            "examples/postgres-rig-agent-jobs/sql/job_kind_readiness_review.sql",
            "examples/postgres-rig-agent-jobs/src/launch_packet.rs",
            "examples/postgres-rig-agent-jobs/sql/job_kind_launch_packet_status.sql",
            "examples/postgres-rig-agent-jobs/sql/release_gate_status.sql",
            "examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql",
            "examples/postgres-rig-agent-jobs/src/failure_drill.rs",
            "examples/postgres-rig-agent-jobs/sql/failure_drill_status.sql",
            "examples/postgres-rig-agent-jobs/src/temporal_adoption.rs",
            "examples/postgres-rig-agent-jobs/sql/temporal_workflow_reconciliation.sql",
            "examples/postgres-rig-agent-jobs/src/kafka_adoption.rs",
            "examples/postgres-rig-agent-jobs/sql/kafka_replay_safety_by_event.sql",
            "--features api-server api::tests::api_admission_enqueues_typed_job",
            "check-micro-drill-artifacts.py",
            "RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh",
            "minute 7: stop or write one repair action",
            "durable intake drill -> durable intake proof",
            "timeout policy drill -> deadline proof",
            "cancellation request drill -> durable stop proof",
            "human escalation drill -> named-owner proof",
            "compensation action drill -> corrective side-effect proof",
            "agent handoff drill -> responsibility transfer proof",
            "cost and capacity guard drill -> admission and spend-control proof",
            "release gate drill -> release decision proof",
            "job-kind readiness review drill -> job-kind launch-level proof",
            "job-kind launch packet drill -> first-user launch proof",
            "injection and exfiltration guard drill -> abuse-boundary proof",
            "agent memory drill -> memory influence proof",
            "restore replay drill -> restore and replay note",
            "failure drill -> controlled-failure proof",
            "temporal adoption decision drill -> workflow adoption proof",
            "kafka adoption decision drill -> event-stream adoption proof",
            "Invariant: every drill must end with a real artifact and one proof sentence.",
        ),
        failures,
    )
    require_phrases(
        PRODUCTION_BUILD_MILESTONES,
        "production build milestones for deployable learning",
        (
            "Appendix AB. Production Build Milestones",
            "build -> inspect -> run -> proof",
            "build:\ninspect:\nrun:\nproof:\ndo not move on if:",
            "| Durable intake |",
            "| Typed domain boundary |",
            "| Worker ownership |",
            "| Timeout policy |",
            "| Cancellation request |",
            "| Rig provider boundary |",
            "| Idempotent side effects |",
            "| Compensation action |",
            "| Human approval gate |",
            "| Human escalation |",
            "| Agent handoff |",
            "| Observability and SLOs |",
            "| Cost and capacity guard |",
            "| Evaluation and release gate |",
            "| Job-kind readiness review |",
            "| Job-kind launch packet |",
            "| Security boundary |",
            "| Injection and exfiltration guard |",
            "| Agent memory |",
            "| Credential lifecycle |",
            "| Data protection |",
            "| Recovery and replay |",
            "| Failure drill |",
            "| Operations runbooks |",
            "| Evidence-preserving scale |",
            "| Temporal adoption decision |",
            "| Kafka adoption decision |",
            "RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh",
            "milestone:\nowner:\nartifact:\ncommand:\nproof:\nmissing evidence:\nnext repair:",
            "small step\nreal artifact\nvisible proof\nrepair before expansion",
            "A late job becomes a visible deadline decision instead of an endless wait.",
            "Stopping work has requested, applied, ignored-terminal, or expired evidence.",
            "A correction is approved, idempotent, leased, retryable, and terminal.",
            "A deadline breach, repeated failure, security signal, approval timeout, or compatibility risk names a human owner and timeline.",
            "A simulated or staged failure proves expected state transitions and operator evidence.",
            "Temporal workflow history reconciles with Postgres product rows, activity receipts, approval decisions, audit events, and trace ids.",
            "Kafka replay cannot duplicate side effects because events come from the outbox and consumers write idempotent receipts.",
            "temporal_workflow_reconciliation.sql",
            "kafka_replay_safety_by_event.sql",
            "job_kind_launch_packet_status.sql",
            "A handoff names source run, source agent, target agent, reason, payload, idempotency key, decision, and target job evidence.",
            "New work can be delayed or rejected before cost, quota, or latency turns retries into an incident.",
            "A job kind can be launched only when its readiness evidence matches its risk.",
            "First-user exposure becomes a typed launch packet with readiness, release, failure-drill, rollback, restore, and known-gap evidence.",
            "Prompt injection, tool injection, and data exfiltration attempts stop before trusted tool execution.",
            "Memory can influence prompts only when scope, kind, source, confidence, horizon, retention, and redaction evidence are valid.",
            "Do not add Temporal before workflow execution is the strained invariant.",
            "Do not add Kafka before the outbox event, schema, partition key, consumer receipt, replay rule, and authorization boundary are explicit.",
            "Simple language helps the learner find the proof.",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "00-how-to-read-this-book.md",
        "minimal prior knowledge repair path",
        (
            "## Prerequisite Repair Map",
            "Use repair, not a long detour.",
            "| Rust newtypes and enums |",
            "| Postgres row locks and leases |",
            "| Rig model and tool boundary |",
            "| SRE vocabulary |",
            "| Agent evaluation |",
            "| Agent security |",
            "| Recovery and replay |",
            "repair only the missing concept until you can name one artifact and one proof",
        ),
        failures,
    )
    require_phrases(
        TRACEABILITY,
        "requirement traceability matrix",
        (
            "Minimal production stack: Rust, PostgreSQL, Rig, worker, API when needed.",
            "PostgreSQL is the first durable coordination layer.",
            "Rig is the agent intelligence boundary, not the reliability layer.",
            "SQL migrations and schemas preserve workflow state.",
            "Domain values use explicit Rust types instead of raw primitives.",
            "Typestate is used where lifecycle state controls legal operations.",
            "Raw outside, typed inside.",
            "Model output is parsed and validated before authority is granted.",
            "Every tool has an explicit contract.",
            "Agent memory is typed, scoped, retained, and policy-checked.",
            "the micro-drill and build milestone require a memory practice loop and milestone proof.",
            "Human approval and escalation are durable control surfaces.",
            "the micro-drill and build milestone require human escalation practice and milestone evidence.",
            "Agent handoffs are durable responsibility transfers, not chat messages.",
            "pending_agent_handoffs.sql",
            "Handoff row-conversion tests reject same-agent loops, non-object payloads, unknown statuses, accepted rows without target jobs, and terminal rows without decision evidence",
            "runtime-tracing",
            "structured tracing before work starts",
            "Checked SQL artifacts are registered and source-visible.",
            "SQL_ARTIFACTS",
            "check-sql-artifact-coverage.py",
            "Behavior evaluation is part of release engineering.",
            "Cost, latency, capacity, and provider quotas are operational controls.",
            "the micro-drill and build milestone require admission and cost-capacity practice evidence.",
            "release_gate_runs",
            "release-gate status query",
            "DbReleaseGateRunRow -> ReleaseGateRunReceipt",
            "Job-kind deprecation and retirement are evidence decisions.",
            "job_kind_lifecycle_review.sql",
            "Lifecycle row-conversion tests reject negative counts, unknown recommendations, missing pause reasons, and inconsistent retirement evidence",
            "Job-kind maturity claims are typed readiness evidence.",
            "job_kind_readiness_review.sql",
            "Readiness row-conversion tests reject unknown maturity labels, negative counts, impossible evidence totals, invalid review windows, regulated jobs targeting too-low maturity, and inconsistent readiness statuses",
            "Credential lifecycle work is durable without storing secret values.",
            "credential_rotation_review.sql",
            "Rust row-conversion tests reject unknown credential kinds, unknown review statuses, negative counts, impossible issue totals, and inconsistent lifecycle status",
            "Privacy and data-protection work is durable operational state.",
            "data_protection_review.sql",
            "Rust row-conversion tests reject unknown surfaces, unknown review statuses, negative counts, impossible count totals, and inconsistent privacy status",
            "Timeout, cancellation, and compensation are separate production controls.",
            "the micro-drill and build milestone require timeout, cancellation, and compensation practice loops.",
            "the micro-drill and build milestone require failure-drill practice evidence.",
            "Security boundaries stay outside the model.",
            "failure-drill runbook checks prove controlled experiments leave hypothesis, blast-radius, injection, rollback, evidence, and signoff",
            "Pedagogy supports ADHD and low-working-memory readers without lowering rigor.",
            "chapter-local Chapter Threads with `Builds on`, `Adds`, and `Prepares`",
            "place in the production chain",
            "method map",
            "fifteen-minute production sprint",
            "now / next / done",
            "small action / fast feedback loops",
            "fast feedback check",
            "chapter-local Micro-Lessons after acceptance gates",
            "micro-lesson",
            "plain-language production cards",
            "production micro-drills",
            "production build milestones",
            "timeout policy, cancellation request, Rig provider boundary, idempotent side effects, compensation action",
            "recovery, failure drill, operations, and evidence-preserving scale",
            "seven-minute production drill",
            "build milestone",
            "simplest accurate sentence",
        ),
        failures,
    )


def check_validation_surfaces(failures: list[str]) -> None:
    require_phrases(
        READINESS,
        "readiness gate coverage",
        (
            "python3 scripts/check-public-repo-surface.py",
            "python3 scripts/check-public-chapter-structure.py",
            "python3 scripts/check-book-objective-coverage.py",
            "python3 scripts/check-book-code-contract.py",
            "python3 scripts/check-sql-artifact-coverage.py",
            "python3 scripts/check-micro-drill-artifacts.py",
            "python3 scripts/check-cargo-dependency-policy.py",
            "mdbook test",
            "mdbook build",
            "cargo fmt",
            "cargo test",
            "cargo clippy",
            "cargo audit",
            "RUN_LOCAL_POSTGRES",
            "RUN_LIVE_DEEPSEEK",
            "schema_migration_status.sql",
            "failure_drill_status.sql",
            "release_gate_status.sql",
            "job_kind_lifecycle_review.sql",
            "job_kind_readiness_review.sql",
            "storage_pressure_by_table.sql",
            "credential_rotation_review.sql",
            "retention_review_by_surface.sql",
            "data_protection_review.sql",
        ),
        failures,
    )

    require_phrases(
        BOOK_CODE_CONTRACT,
        "book executable snippet contract",
        (
            "SHELL_PLACEHOLDER_RE",
            "check_bash_block",
            "non-executable placeholder",
            "bash block references missing repository path",
        ),
        failures,
    )

    require_phrases(
        README,
        "operator validation documentation",
        (
            "Run The Full Readiness Gate",
            "check-public-repo-surface.py",
            "check-public-chapter-structure.py",
            "check-cargo-dependency-policy.py",
            "Run The Local Ephemeral Postgres Gate",
            "Run The Live DeepSeek Gate",
            "Run The Local System",
        ),
        failures,
    )

    require_phrases(
        PUBLIC_REPO_SURFACE,
        "public repository surface gate",
        (
            "Check the public book repository surface",
            "REQUIRED_GITIGNORE_PATTERNS",
            "LOCAL_IGNORED_TOP_LEVEL",
            ".idea/",
            "books/*/book/",
            "examples/*/target/",
            "ALLOWED_TOP_LEVEL",
            "FORBIDDEN_DIRECTORY_NAMES",
            "public repository surface check passed",
        ),
        failures,
    )

    require_phrases(
        PUBLIC_CHAPTER_STRUCTURE,
        "public chapter structure gate",
        (
            "SUMMARY_LINK_RE",
            "DEEP_CHAPTERS",
            "MIN_SOURCE_LINKS",
            "DEEP_REQUIRED_HEADINGS",
            "public chapter structure check passed",
        ),
        failures,
    )

    require_phrases(
        DEPENDENCY_POLICY,
        "minimal dependency policy gate",
        (
            "ALLOWED_RUNTIME_DEPENDENCIES",
            "OPTIONAL_RUNTIME_DEPENDENCIES",
            "REQUIRED_FEATURE_ITEMS",
            "FORBIDDEN_DIRECT_DEPENDENCIES",
            "default feature set must stay empty",
            "cargo dependency policy check passed",
        ),
        failures,
    )

    require_any(
        QUALITY,
        "quality standard names objective coverage gate",
        (
            "scripts/check-book-objective-coverage.py",
            "objective coverage",
        ),
        failures,
    )


def check_optional_scaling_reference_surfaces(failures: list[str]) -> None:
    require_phrases(
        GLOSSARY,
        "optional scaling concepts in glossary and invariant index",
        (
            "Temporal workflow execution ledger",
            "Workflow id mapping",
            "Activity receipt",
            "Kafka event distribution",
            "Event envelope",
            "Topic-partition-offset",
            "Consumer receipt",
            "Event replay rule",
            "Temporal replaces the product ledger.",
            "Kafka is the source of truth.",
            "Temporal adoption preserves product truth.",
            "Kafka adoption preserves event truth and replay safety.",
        ),
        failures,
    )

    require_phrases(
        PRODUCTION_EVIDENCE_PACKETS,
        "optional scaling production evidence packets",
        (
            "## Packet 7: Temporal Adoption Packet",
            "strained workflow invariant:",
            "workflow id rule:",
            "activity id rule:",
            "temporal workflow link table:",
            "temporal activity receipt table:",
            "approval and receipt mapping:",
            "Temporal adoption fails the packet when workflow history becomes the only place",
            "## Packet 8: Kafka Adoption Packet",
            "event envelope version:",
            "partition key:",
            "publish receipt table:",
            "consumer receipt table:",
            "replay-safety query:",
            "Kafka adoption fails the packet when Kafka becomes the product source of truth",
        ),
        failures,
    )

    require_phrases(
        DESIGN_SMELLS,
        "optional scaling design smells",
        (
            "| 30.6 Temporal After Postgres-First |",
            "Temporal history is treated as the product audit trail.",
            "| 30.7 Kafka After Postgres-First |",
            "Kafka is added before event ownership, schema, partition key, and consumer idempotency are defined.",
        ),
        failures,
    )

    require_phrases(
        FORMAL_DEFINITION_LEDGER,
        "optional scaling formal definitions",
        (
            "| 30.6 Temporal After Postgres-First |",
            "workflow execution responsibility to a durable workflow engine",
            "| 30.7 Kafka After Postgres-First |",
            "event distribution responsibility to a partitioned log",
        ),
        failures,
    )

    require_phrases(
        OPERATOR_CONTROL_SURFACE,
        "optional scaling operator control surface",
        (
            "| Temporal reconciliation |",
            "| Kafka distribution |",
            "| Do Temporal and Postgres agree for this run? |",
            "| Which Kafka event or consumer is lagging or unsafe to replay? |",
            "temporal_workflow_reconciliation.sql",
            "kafka_replay_safety_by_event.sql",
            "Mark workflow reconciliation needed",
            "Pause event publisher or consumer group",
            "Replay workflow history as product truth",
            "Replay Kafka topics blindly",
            "Temporal workflow history and Kafka offsets reconcile with Postgres product",
        ),
        failures,
    )

    require_phrases(
        BOOK_SRC / "30c-temporal-after-postgres-first.md",
        "Temporal chapter has executable product evidence",
        (
            "## Postgres Evidence Tables",
            "temporal_workflow_links",
            "temporal_activity_receipts",
            "## Reconciliation Query",
            "temporal_workflow_reconciliation.sql",
            "Do Temporal workflow history and Postgres product evidence agree for this workflow?",
        ),
        failures,
    )

    require_phrases(
        BOOK_SRC / "30d-kafka-after-postgres-first.md",
        "Kafka chapter has executable product evidence",
        (
            "## Postgres Evidence Tables",
            "kafka_publish_receipts",
            "kafka_consumer_receipts",
            "## Replay-Safety Query",
            "kafka_replay_safety_by_event.sql",
            "Can this event be replayed without duplicating side effects?",
        ),
        failures,
    )


def check_agent_research_sources(failures: list[str]) -> None:
    require_phrases(
        SOURCES,
        "primary agent research source coverage",
        (
            "## Agent Research and Evaluation Papers",
            "ReAct: Synergizing Reasoning and Acting in Language Models",
            "https://arxiv.org/abs/2210.03629",
            "Toolformer: Language Models Can Teach Themselves to Use Tools",
            "https://arxiv.org/abs/2302.04761",
            "Reflexion: Language Agents with Verbal Reinforcement Learning",
            "https://arxiv.org/abs/2303.11366",
            "Evaluating Large Language Models Trained on Code",
            "https://arxiv.org/abs/2107.03374",
            "SWE-bench: Can Language Models Resolve Real-World GitHub Issues?",
            "https://arxiv.org/abs/2310.06770",
            "If You Are Building Coding Or Tool-Using Agents",
        ),
        failures,
    )

    require_phrases(
        BOOK_SRC / "02-mental-model.md",
        "ReAct wired into mental model chapter",
        ("ReAct: Synergizing Reasoning and Acting in Language Models",),
        failures,
    )
    require_phrases(
        BOOK_SRC / "06-rig-boundary.md",
        "Toolformer and ReAct wired into Rig boundary chapter",
        (
            "Toolformer: Language Models Can Teach Themselves to Use Tools",
            "ReAct: Synergizing Reasoning and Acting in Language Models",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "27-evaluation-behavior-reliability.md",
        "execution and repository evaluation wired into evaluation chapter",
        (
            "Evaluating Large Language Models Trained on Code",
            "SWE-bench: Can Language Models Resolve Real-World GitHub Issues?",
        ),
        failures,
    )
    require_phrases(
        BOOK_SRC / "27b-agent-memory-retrieval-retention.md",
        "Reflexion wired into memory chapter",
        ("Reflexion: Language Agents with Verbal Reinforcement Learning",),
        failures,
    )


def check_first_production_deployment_proof(failures: list[str]) -> None:
    require_phrases(
        FIRST_PRODUCTION_DEPLOYMENT_PROOF,
        "first production deployment proof",
        (
            "This appendix is a launch proof path, not a cloud recipe.",
            "one API process",
            "one worker process",
            "one Postgres database",
            "one Rig boundary",
            "Do not expose a new agent job kind to real users until",
            "durable intake proof:",
            "worker ownership proof:",
            "provider boundary proof:",
            "policy or approval proof:",
            "evaluation proof:",
            "security proof:",
            "rollback or pause plan:",
            "restore and replay note:",
            "job_kind_launch_packets",
            "job_kind_launch_packet_status.sql",
            "DbJobKindLaunchPacketStatusRow",
            "LaunchEvidenceChecklist",
            "approved_for_first_users",
            "RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh",
            "RUN_LIVE_DEEPSEEK=1 ./scripts/check-book-readiness.sh",
        ),
        failures,
    )

    require_phrases(
        TRACEABILITY,
        "traceability includes first production deployment decision",
        (
            "First production exposure is a job-kind evidence decision.",
            "Appendix Y requires a launch packet for durable intake, worker ownership, Rig boundary proof, side-effect control, evaluation, observability, security, rollback, and recovery before real users see a job kind.",
            "`job_kind_launch_packets` and `job_kind_launch_packet_status.sql` turn the launch packet into a typed, queryable first-user decision.",
        ),
        failures,
    )


def check_risk_tiered_examples(failures: list[str]) -> None:
    require_phrases(
        CASE_STUDIES,
        "risk-tiered realistic examples",
        (
            "## Example Maturity Ladder",
            "Ready for which version of the job kind?",
            "| Case study | Demo version | Prototype version | Production version | Regulated/high-risk version |",
            "Incident triage agent",
            "Customer support reply agent",
            "Billing adjustment agent",
            "demo version -> prototype version -> production version -> regulated/high-risk version",
            "## Version Stop Rules",
            "Regulated/high-risk version",
        ),
        failures,
    )

    require_phrases(
        TRACEABILITY,
        "traceability includes realistic example maturity levels",
        (
            "Realistic examples distinguish demo, prototype, production, and regulated/high-risk versions.",
            "The example maturity ladder, release-gate tests, high-risk approval evidence",
            "Can a reader tell whether an example is only a demo, a prototype, a production workflow, or a regulated/high-risk workflow",
        ),
        failures,
    )

    require_phrases(
        QUALITY,
        "quality standard requires realistic example maturity levels",
        (
            "Production examples should distinguish the demo version, prototype version, production version, and regulated/high-risk version.",
        ),
        failures,
    )

    require_phrases(
        STYLE_GUIDE,
        "style guide requires realistic example maturity levels",
        (
            "Each realistic example should name its demo version",
            "regulated/high-risk version",
        ),
        failures,
    )


def main() -> int:
    failures: list[str] = []

    for path in (
        README,
        SUMMARY,
        COVER,
        STYLE_GUIDE,
        QUALITY,
        TRACEABILITY,
        FIRST_PRODUCTION_DEPLOYMENT_PROOF,
        PRODUCTION_MICRO_DRILLS,
        PRODUCTION_BUILD_MILESTONES,
        FAILURE_FIRST_LEARNING_MAP,
        SOURCES,
        READINESS,
        DEPENDENCY_POLICY,
        CARGO,
    ):
        require_file(path, failures)

    check_book_shape(failures)
    check_public_draft_reuse_note(failures)
    check_architectural_thesis(failures)
    check_postgres_coordination(failures)
    check_rust_and_rig_boundaries(failures)
    check_reliability_controls(failures)
    check_raw_outside_typed_inside_and_pedagogy(failures)
    check_validation_surfaces(failures)
    check_optional_scaling_reference_surfaces(failures)
    check_agent_research_sources(failures)
    check_first_production_deployment_proof(failures)
    check_risk_tiered_examples(failures)

    if failures:
        print("book objective coverage check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("book objective coverage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
