#!/usr/bin/env python3
"""Check panic-free production Rust for the Reliable AI Agents example.

Tests may use `expect` to make fixture failures readable. Production code should
return typed errors instead of hiding fallible paths behind panic-based control
flow.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = ROOT / "examples" / "postgres-rig-agent-jobs" / "src"
CARGO_MANIFEST = ROOT / "examples" / "postgres-rig-agent-jobs" / "Cargo.toml"
LIB_SRC = SRC_DIR / "lib.rs"
WORKER_SRC = SRC_DIR / "worker.rs"
API_SRC = SRC_DIR / "api.rs"
ADMISSION_CONTROL_SRC = SRC_DIR / "admission_control.rs"
AGENT_OUTPUT_SRC = SRC_DIR / "agent_output.rs"
BACKGROUND_JOB_SRC = SRC_DIR / "background_job.rs"
AGENT_RUN_SRC = SRC_DIR / "agent_run.rs"
AGENT_STEP_SRC = SRC_DIR / "agent_step.rs"
RIG_RUNNER_SRC = SRC_DIR / "rig_runner.rs"
COMPATIBILITY_SRC = SRC_DIR / "compatibility.rs"
ESCALATION_SRC = SRC_DIR / "escalation.rs"
FAILURE_DRILL_SRC = SRC_DIR / "failure_drill.rs"
FAILURE_HISTORY_SRC = SRC_DIR / "failure_history.rs"
EVALUATION_SRC = SRC_DIR / "evaluation.rs"
RELEASE_GATE_SRC = SRC_DIR / "release_gate.rs"
TOOL_EXECUTION_GATE_SRC = SRC_DIR / "tool_execution_gate.rs"
TOOL_CALL_SRC = SRC_DIR / "tool_call.rs"
TOOL_CONTRACT_SRC = SRC_DIR / "tool_contract.rs"
SCHEDULED_JOB_SRC = SRC_DIR / "scheduled_job.rs"
POSTGRES_STORE_SRC = SRC_DIR / "postgres_store.rs"
LOGGING_SRC = SRC_DIR / "logging.rs"
SQL_SRC = SRC_DIR / "sql.rs"

FORBIDDEN_PATTERNS = (
    (re.compile(r"\.(unwrap|expect)\s*\("), "panic-style fallible call"),
    (re.compile(r"\b(panic|todo|unimplemented)\s*!\s*\("), "panic macro"),
    (re.compile(r"Result\s*<[^,\n>]+,\s*String\s*>"), "String error type"),
    (
        re.compile(r"Box\s*<\s*dyn\s+(?:std::)?error::Error"),
        "untyped boxed error",
    ),
)


def production_lines(path: Path) -> list[tuple[int, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    for index, line in enumerate(lines):
        if line.strip() == "#[cfg(test)]":
            lines = lines[:index]
            break

    return [(line_number, line) for line_number, line in enumerate(lines, start=1)]


def without_line_comment(line: str) -> str:
    return line.split("//", 1)[0]


def check_worker_observability(failures: list[str]) -> None:
    manifest = CARGO_MANIFEST.read_text(encoding="utf-8")
    worker = WORKER_SRC.read_text(encoding="utf-8")

    if 'tracing = "0.1"' not in manifest:
        failures.append(
            f"{CARGO_MANIFEST.relative_to(ROOT)}: missing explicit tracing dependency"
        )
    if "tracing-subscriber" not in manifest:
        failures.append(
            f"{CARGO_MANIFEST.relative_to(ROOT)}: missing explicit tracing subscriber dependency"
        )

    for phrase in (
        "tracing::info!",
        "tracing::warn!",
        "worker recovered expired job lease",
        "worker heartbeat extended job lease",
        "worker heartbeat rejected because lease is not owned or running",
        "run_once_with_heartbeats",
        "worker lost lease during heartbeat-supervised execution",
        "run_once_controlled",
        "worker drain requested; skipping new job claim",
        "run_bounded_loop",
        "worker bounded loop stopped after cycle limit",
        "agent execution failed and retry was scheduled",
        "agent execution failed and job is dead-lettered",
        "pub fn as_str(self) -> &'static str",
    ):
        if phrase not in worker:
            failures.append(
                f"{WORKER_SRC.relative_to(ROOT)}: missing worker observability phrase: {phrase}"
            )


def check_runtime_tracing_startup(failures: list[str]) -> None:
    logging = LOGGING_SRC.read_text(encoding="utf-8")
    for phrase in (
        "pub const RUST_LOG_ENV",
        "pub const LOG_FORMAT_ENV",
        "pub enum RuntimeLogFormat",
        "pub struct RuntimeTracingConfig",
        "pub enum RuntimeTracingError",
        "EnvFilter::try_new",
        "RuntimeLogFormat::Json",
        "tracing_subscriber::fmt()",
        "try_init()",
    ):
        if phrase not in logging:
            failures.append(
                f"{LOGGING_SRC.relative_to(ROOT)}: missing runtime tracing phrase: {phrase}"
            )

    for binary in (
        "main.rs",
        "bin/deepseek_agent_demo.rs",
        "bin/postgres_worker_demo.rs",
        "bin/postgres_api_server.rs",
    ):
        path = SRC_DIR / binary
        text = path.read_text(encoding="utf-8")
        for phrase in (
            "init_runtime_tracing(RuntimeTracingConfig::from_env()?)?",
            "RuntimeTracingError",
        ):
            if phrase not in text:
                failures.append(
                    f"{path.relative_to(ROOT)}: missing runtime tracing startup phrase: {phrase}"
                )


def check_sql_artifact_registry(failures: list[str]) -> None:
    sql = SQL_SRC.read_text(encoding="utf-8")
    for phrase in (
        "pub struct SqlFileName",
        "pub struct SqlArtifact",
        "pub const SQL_ARTIFACTS",
        "include_str!",
        "sql_artifact_registry_names_each_checked_sql_file_once",
    ):
        if phrase not in sql:
            failures.append(
                f"{SQL_SRC.relative_to(ROOT)}: missing SQL artifact registry phrase: {phrase}"
            )


def check_typed_application_errors(failures: list[str]) -> None:
    manifest = CARGO_MANIFEST.read_text(encoding="utf-8")
    if re.search(r"(?m)^anyhow\s*=", manifest):
        failures.append(
            f"{CARGO_MANIFEST.relative_to(ROOT)}: direct anyhow dependency weakens the typed-error boundary"
        )

    for path in [LIB_SRC, *SRC_DIR.glob("*.rs"), *SRC_DIR.glob("bin/*.rs")]:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        if "anyhow::" in text:
            failures.append(
                f"{path.relative_to(ROOT)}: production runtime should return typed thiserror errors, not anyhow"
            )

    api = API_SRC.read_text(encoding="utf-8")
    for phrase in (
        "pub enum PostgresApiServerError",
        "Bind(std::io::Error)",
        "Serve(std::io::Error)",
    ):
        if phrase not in api:
            failures.append(
                f"{API_SRC.relative_to(ROOT)}: missing typed API server error phrase: {phrase}"
            )

    for binary, error_name in (
        ("main.rs", "LocalDemoError"),
        ("bin/deepseek_agent_demo.rs", "DeepSeekDemoError"),
        ("bin/postgres_worker_demo.rs", "PostgresWorkerDemoError"),
        ("bin/postgres_api_server.rs", "PostgresApiBinaryError"),
    ):
        path = SRC_DIR / binary
        text = path.read_text(encoding="utf-8")
        if error_name not in text:
            failures.append(
                f"{path.relative_to(ROOT)}: missing typed runtime error enum {error_name}"
            )


def check_api_runtime_surfaces(failures: list[str]) -> None:
    api = API_SRC.read_text(encoding="utf-8")

    for phrase in (
        '.route("/healthz", get(healthz))',
        '.route("/readyz", get(readyz::<S>))',
        '.route("/metrics", get(metrics::<S>))',
        "pub trait AgentJobObservabilityStore",
        "pub trait AgentAdmissionStore",
        "ReadinessStatus::Ready",
        "QueueMetricsResponse",
        "AdmissionDecisionKind",
        "IdempotencyOutcome",
        "existing_job_for_idempotency_key",
        "crate::PostgresAgentJobStore::existing_job_for_idempotency_key",
        "admit_agent_job",
        "crate::PostgresAgentJobStore::admit_agent_job",
        "record_admission_decision",
        "crate::PostgresAgentJobStore::record_admission_decision",
        "admission_rejected",
        "readiness_check_failed",
        "api_runtime_surfaces",
    ):
        if phrase not in api:
            failures.append(
                f"{API_SRC.relative_to(ROOT)}: missing API runtime surface phrase: {phrase}"
            )


def check_admission_control_boundary(failures: list[str]) -> None:
    admission_control = ADMISSION_CONTROL_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct AdmissionPolicy",
        "pub struct AdmissionControlInput",
        "pub struct AdmissionSubject",
        "pub struct AdmissionSignals",
        "pub struct AdmissionDecisionEvent",
        "pub enum AdmissionDecision",
        "pub enum AdmissionDecisionKind",
        "pub enum AdmissionReason",
        "pub enum JobPriority",
        "pub enum ProviderQuotaPressure",
        "pub enum QueuePressure",
        "pub enum BudgetAdmissionState",
        "UnknownJobPriority",
        "UnknownProviderQuotaPressure",
        "pub struct MaxPendingDepth",
        "pub struct MaxOldestPendingAge",
        "pub struct AdmissionDelay",
        "TenantBudgetExceeded",
        "ProviderExhausted",
        "QueueSaturated",
        "pub fn evaluate",
        "with_job_id",
        "fn queue_pressure",
    ):
        if phrase not in admission_control:
            failures.append(
                f"{ADMISSION_CONTROL_SRC.relative_to(ROOT)}: missing admission-control phrase: {phrase}"
            )


def check_trace_context_boundary(failures: list[str]) -> None:
    audit = (SRC_DIR / "audit.rs").read_text(encoding="utf-8")

    for phrase in (
        "pub struct TraceId",
        "pub struct SpanId",
        "pub struct TraceContext",
        "is_valid_nonzero_hex_id",
        "InvalidTraceId",
        "InvalidSpanId",
    ):
        if phrase not in audit:
            failures.append(
                f"{(SRC_DIR / 'audit.rs').relative_to(ROOT)}: missing trace-context phrase: {phrase}"
            )


def check_agent_output_boundary(failures: list[str]) -> None:
    agent_output = AGENT_OUTPUT_SRC.read_text(encoding="utf-8")
    rig_runner = RIG_RUNNER_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct RawAgentOutput",
        "pub struct ParsedAgentOutput",
        "pub struct ValidatedAgentOutput",
        "pub struct AgentOutputParseFailure",
        "pub struct UnknownAgentApprovalRequirement",
        "#[serde(deny_unknown_fields)]",
        "pub fn parse_agent_result",
        "UnknownApprovalRequirement",
    ):
        if phrase not in agent_output:
            failures.append(
                f"{AGENT_OUTPUT_SRC.relative_to(ROOT)}: missing agent-output phrase: {phrase}"
            )

    for phrase in (
        "RawAgentOutput::new(provider_output)?.parse_agent_result()?",
        "Return only a strict JSON object",
        "approval_requirement must be required or not_required",
    ):
        if phrase not in rig_runner:
            failures.append(
                f"{RIG_RUNNER_SRC.relative_to(ROOT)}: missing Rig output-boundary phrase: {phrase}"
            )


def check_agent_run_boundary(failures: list[str]) -> None:
    agent_run = AGENT_RUN_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct AgentRun<State>",
        "pub struct AgentRunPlanning",
        "pub struct AgentRunRunning",
        "pub struct AgentRunWaitingForHuman",
        "pub enum AgentRunLifecycleStatus",
        "pub struct AgentModelVersion",
        "pub struct DbAgentRunRow",
        "pub enum DecodedAgentRun",
        "MissingRunOwner",
        "MissingFinishedAt",
        "UnexpectedFinishedAt",
        "FinishedBeforeStarted",
        "fn validate_finished_evidence",
    ):
        if phrase not in agent_run:
            failures.append(
                f"{AGENT_RUN_SRC.relative_to(ROOT)}: missing agent-run phrase: {phrase}"
            )


def check_background_job_boundary(failures: list[str]) -> None:
    background_job = BACKGROUND_JOB_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct BackgroundJob<State>",
        "pub struct BackgroundJobId",
        "pub struct BackgroundQueued",
        "pub struct BackgroundLeased",
        "pub struct BackgroundExecutingAgent",
        "pub struct BackgroundWaitingForHuman",
        "pub struct BackgroundWaitingForRetry",
        "pub enum WorkflowState",
        "pub enum RetryState",
        "pub struct FailureClass",
        "pub struct DbBackgroundJobRow",
        "pub enum DecodedBackgroundJob",
        "MissingExecutionDeadline",
        "UnexpectedExecutionDeadline",
        "MissingNextRetryAt",
        "UnexpectedNextRetryAt",
        "IncompatibleWorkflowAndRetryState",
        "MissingFailureEvidence",
        "AttemptsExceedMaxAttempts",
        "fn validate_background_job_record",
    ):
        if phrase not in background_job:
            failures.append(
                f"{BACKGROUND_JOB_SRC.relative_to(ROOT)}: missing background-job phrase: {phrase}"
            )


def check_agent_step_boundary(failures: list[str]) -> None:
    agent_step = AGENT_STEP_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct AgentStep<State>",
        "pub struct AgentStepPending",
        "pub struct AgentStepRunning",
        "pub struct AgentStepSucceeded",
        "pub enum AgentStepKind",
        "pub enum AgentStepStatus",
        "pub struct AgentStepIndex",
        "pub struct AgentStepRef",
        "pub struct AgentStepTerminalReason",
        "pub struct DbAgentStepRow",
        "pub enum DecodedAgentStep",
        "NegativeStepIndex",
        "MissingOutputRef",
        "MissingTerminalReason",
        "CompletedBeforeStarted",
        "fn validate_step_evidence",
    ):
        if phrase not in agent_step:
            failures.append(
                f"{AGENT_STEP_SRC.relative_to(ROOT)}: missing agent-step phrase: {phrase}"
            )


def check_compatibility_boundary(failures: list[str]) -> None:
    compatibility = COMPATIBILITY_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct WorkerCompatibilityPolicy",
        "pub struct SupportedPayloadSchemaRange",
        "pub enum CompatibilityDecision",
        "pub enum CompatibilityQuarantineReason",
        "PayloadSchemaTooOld",
        "PayloadSchemaTooNew",
        "JobCompatibilityReport",
    ):
        if phrase not in compatibility:
            failures.append(
                f"{COMPATIBILITY_SRC.relative_to(ROOT)}: missing compatibility-boundary phrase: {phrase}"
            )


def check_human_escalation_boundary(failures: list[str]) -> None:
    escalation = ESCALATION_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct HumanEscalationRecord",
        "pub struct DbHumanEscalationRow",
        "pub enum EscalationKind",
        "pub enum EscalationSeverity",
        "pub enum EscalationStatus",
        "MissingEscalationTarget",
        "MissingAcknowledgementEvidence",
        "MissingResolutionEvidence",
    ):
        if phrase not in escalation:
            failures.append(
                f"{ESCALATION_SRC.relative_to(ROOT)}: missing human-escalation phrase: {phrase}"
            )


def check_failure_drill_boundary(failures: list[str]) -> None:
    failure_drill = FAILURE_DRILL_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct FailureDrillPlan",
        "pub struct FailureDrillReport",
        "pub enum FailureDrillScenario",
        "pub enum EvidenceRequirement",
        "pub struct EvidenceRequirements",
        "pub struct ObservedDrillEvidence",
        "pub struct ProviderTimeoutThenRetryDrill",
        "pub struct WorkerCrashAfterLeaseDrill",
        "RecoveryBeforeLeaseExpiry",
        "MissingRequiredEvent",
        "ProviderTimeoutThenRetry",
        "WorkerCrashAfterLease",
        "pub async fn run",
        "fn provider_timeout_plan",
        "fn worker_crash_plan",
    ):
        if phrase not in failure_drill:
            failures.append(
                f"{FAILURE_DRILL_SRC.relative_to(ROOT)}: missing failure-drill phrase: {phrase}"
            )


def check_failure_history_boundary(failures: list[str]) -> None:
    failure_history = FAILURE_HISTORY_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct FailureHistoryRecord",
        "pub struct FailureHistoryId",
        "pub struct DbFailureHistoryRow",
        "pub enum FailureSource",
        "pub enum FailureOutcome",
        "RetryScheduled",
        "DeadLettered",
        "PermanentFailure",
        "EscalatedToHuman",
        "SpanWithoutTrace",
        "MissingNextRetryAt",
        "DeadLetterBeforeMaxAttempts",
        "IncompatibleOutcomeState",
        "fn validate_failure_history",
        "fn validate_outcome_state",
        "fn validate_retry_time",
    ):
        if phrase not in failure_history:
            failures.append(
                f"{FAILURE_HISTORY_SRC.relative_to(ROOT)}: missing failure-history phrase: {phrase}"
            )


def check_evaluation_row_boundary(failures: list[str]) -> None:
    evaluation = EVALUATION_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct EvaluationRun",
        "pub struct EvaluationRunId",
        "pub enum EvaluationRunStatus",
        "pub struct EvaluationScoreBasisPoints",
        "pub struct EvaluationReport",
        "pub struct DbEvaluationRunRow",
        "UnknownEvaluationRunStatus",
        "InvalidEvaluationScoreBasisPoints",
        "ReportMustBeObject",
        "TerminalRunMissingEvidence",
        "ActiveRunHasTerminalEvidence",
        "fn validate_evaluation_run_evidence",
    ):
        if phrase not in evaluation:
            failures.append(
                f"{EVALUATION_SRC.relative_to(ROOT)}: missing evaluation-row phrase: {phrase}"
            )


def check_release_gate_boundary(failures: list[str]) -> None:
    release_gate = RELEASE_GATE_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct ReleaseGate",
        "pub struct ReleaseGateReport",
        "pub enum ReleaseGateDecision",
        "pub enum ReleaseBlocker",
        "ReleaseApprovalEvidence",
        "NoTrafficForFullPromotion",
        "VersionMismatch",
        "ApprovalMissingOrDenied",
        "SloDecision::BudgetExhausted",
        "CompatibilityDecision::Process",
    ):
        if phrase not in release_gate:
            failures.append(
                f"{RELEASE_GATE_SRC.relative_to(ROOT)}: missing release-gate phrase: {phrase}"
            )


def check_tool_execution_gate_boundary(failures: list[str]) -> None:
    tool_execution_gate = TOOL_EXECUTION_GATE_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct ToolExecutionGate",
        "pub struct ToolExecutionGateInput",
        "pub struct TrustedToolExecution",
        "pub enum ToolExecutionApproval",
        "pub enum ToolExecutionGateError",
        "AuthorizationDenied",
        "SandboxDenied",
        "ApprovalMissing",
        "AuthorizationRunMismatch",
        "SandboxRunMismatch",
        "ToolPermission::PauseJobKind",
        "SandboxDecisionKind::Allowed",
        "TrustedToolExecution<PauseJobKindRequest>",
    ):
        if phrase not in tool_execution_gate:
            failures.append(
                f"{TOOL_EXECUTION_GATE_SRC.relative_to(ROOT)}: missing tool-execution-gate phrase: {phrase}"
            )


def check_tool_call_boundary(failures: list[str]) -> None:
    tool_call = TOOL_CALL_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct ToolCall<State>",
        "pub struct ToolCallRequested",
        "pub struct ToolCallValidated",
        "pub struct ToolCallExecuted",
        "pub enum ToolCallStatus",
        "pub struct ToolCallInput",
        "pub struct ToolCallOutput",
        "pub struct ToolCallTerminalReason",
        "pub struct DbToolCallRow",
        "pub enum DecodedToolCall",
        "InputMustBeObject",
        "MissingOutput",
        "MissingTerminalReason",
        "CompletedBeforeStarted",
        "ToolCallInput([redacted])",
        "fn validate_status_evidence",
    ):
        if phrase not in tool_call:
            failures.append(
                f"{TOOL_CALL_SRC.relative_to(ROOT)}: missing tool-call phrase: {phrase}"
            )


def check_tool_contract_boundary(failures: list[str]) -> None:
    tool_contract = TOOL_CONTRACT_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct RawModelOutput",
        "pub struct ParsedToolRequest",
        "pub struct ValidatedToolRequest",
        "pub struct PolicyCheckedToolRequest",
        "pub struct ApprovedToolRequest",
        "pub trait TypedTool",
        "RawToolRequestDto",
        "PauseJobKindInputDto",
        "#[serde(deny_unknown_fields)]",
        "model_output_parse_rejects_unexpected_top_level_tool_fields",
        "model_output_parse_rejects_unexpected_nested_tool_input_fields",
    ):
        if phrase not in tool_contract:
            failures.append(
                f"{TOOL_CONTRACT_SRC.relative_to(ROOT)}: missing tool-contract phrase: {phrase}"
            )

    if tool_contract.count("#[serde(deny_unknown_fields)]") < 2:
        failures.append(
            f"{TOOL_CONTRACT_SRC.relative_to(ROOT)}: model tool-call DTOs must reject unknown top-level and nested fields"
        )


def check_scheduled_job_boundary(failures: list[str]) -> None:
    scheduled_job = SCHEDULED_JOB_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub struct ScheduledJobId",
        "pub struct ScheduledTaskName",
        "pub struct ScheduledJobPayload",
        "pub struct ScheduledJob<State>",
        "pub struct ScheduledPending",
        "pub struct ScheduledRunning",
        "pub enum ScheduledFailureTransition",
        "pub struct DbScheduledJobRow",
        "pub enum DecodedScheduledJob",
        "MissingRunningLease",
        "UnexpectedLease",
        "AttemptsExhausted",
        "ScheduledFailureTransition::Retry",
        "ScheduledFailureTransition::Dead",
    ):
        if phrase not in scheduled_job:
            failures.append(
                f"{SCHEDULED_JOB_SRC.relative_to(ROOT)}: missing scheduled-job phrase: {phrase}"
            )


def check_postgres_store_row_boundary(failures: list[str]) -> None:
    postgres_store = POSTGRES_STORE_SRC.read_text(encoding="utf-8")

    for phrase in (
        "pub enum AgentJobRowInvariant",
        "PayloadMustBeObject",
        "ResultMustBeObject",
        "RunningLeaseMissing",
        "NonRunningLeasePresent",
        "SucceededMissingResult",
        "SucceededWithLastError",
        "NonSucceededResultPresent",
        "fn validate_lifecycle_evidence",
        "InvalidAgentJobRow",
    ):
        if phrase not in postgres_store:
            failures.append(
                f"{POSTGRES_STORE_SRC.relative_to(ROOT)}: missing Postgres row-boundary phrase: {phrase}"
            )


def main() -> int:
    failures: list[str] = []

    for path in sorted(SRC_DIR.rglob("*.rs")):
        for line_number, line in production_lines(path):
            code = without_line_comment(line)
            for pattern, reason in FORBIDDEN_PATTERNS:
                if pattern.search(code):
                    failures.append(
                        f"{path.relative_to(ROOT)}:{line_number}: {reason}: {line.strip()}"
                    )

    check_typed_application_errors(failures)
    check_worker_observability(failures)
    check_api_runtime_surfaces(failures)
    check_admission_control_boundary(failures)
    check_trace_context_boundary(failures)
    check_agent_output_boundary(failures)
    check_background_job_boundary(failures)
    check_agent_run_boundary(failures)
    check_agent_step_boundary(failures)
    check_compatibility_boundary(failures)
    check_runtime_tracing_startup(failures)
    check_sql_artifact_registry(failures)
    check_human_escalation_boundary(failures)
    check_failure_drill_boundary(failures)
    check_failure_history_boundary(failures)
    check_evaluation_row_boundary(failures)
    check_release_gate_boundary(failures)
    check_tool_execution_gate_boundary(failures)
    check_tool_call_boundary(failures)
    check_tool_contract_boundary(failures)
    check_scheduled_job_boundary(failures)
    check_postgres_store_row_boundary(failures)

    if failures:
        print("rust production hygiene check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("rust production hygiene check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
