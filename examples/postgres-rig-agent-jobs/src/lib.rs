//! Executable examples for the Reliable AI Agents book.
//!
//! The crate defaults to an in-memory store and deterministic agent runner so
//! it can be compiled and tested without external services. Enable the
//! `rig-agent` feature to compile the real Rig-backed runner.

pub mod admission_control;
pub mod agent;
pub mod agent_memory;
pub mod agent_output;
pub mod agent_run;
pub mod agent_step;
#[cfg(feature = "api-server")]
pub mod api;
pub mod approval;
pub mod audit;
pub mod background_job;
pub mod cancellation;
pub mod compatibility;
pub mod compensation;
pub mod config;
pub mod cost_accounting;
pub mod credential_lifecycle;
pub mod data_protection;
pub mod domain;
pub mod escalation;
pub mod evaluation;
pub mod failure_drill;
pub mod failure_history;
pub mod fault_tolerance;
pub mod handoff;
pub mod job_kind_lifecycle;
pub mod job_kind_readiness;
pub mod kafka_adoption;
pub mod launch_packet;
pub mod logging;
pub mod observability;
pub mod memory_store;
pub mod outbox;
#[cfg(feature = "postgres-store")]
pub mod postgres_store;
pub mod recovery;
pub mod release_gate;
#[cfg(feature = "rig-agent")]
pub mod rig_runner;
pub mod sandbox;
pub mod scheduled_job;
pub mod security;
pub mod slo;
pub mod sql;
pub mod temporal_adoption;
pub mod tenant_isolation;
pub mod timeouts;
pub mod tool_call;
pub mod tool_contract;
pub mod tool_execution_gate;
pub mod typed_pipeline;
pub mod worker;

pub use admission_control::{
    AdmissionControlError, AdmissionControlInput, AdmissionDecision, AdmissionDecisionEvent,
    AdmissionDecisionKind, AdmissionDelay, AdmissionPolicy, AdmissionPolicyName, AdmissionReason,
    AdmissionRequestId, AdmissionSignals, AdmissionSubject, BudgetAdmissionState, JobPriority,
    MaxOldestPendingAge, MaxPendingDepth, ProviderQuotaPressure, QueuePressure,
    UnknownAdmissionValue,
};
pub use agent::{
    AgentFailureKind, AgentRunner, DeterministicAgentRunner, FailingThenSuccessfulRunner,
    PermanentFailureRunner, ProviderFailure, SimulatedFailureCount,
};
pub use agent_memory::{
    AgentMemoryError, DbAgentMemoryRecordRow, MemoryConfidence, MemoryContent, MemoryEmbeddingRef,
    MemoryHorizon, MemoryId, MemoryKind, MemoryLifecyclePolicy, MemoryRecord, MemoryScope,
    MemorySource, RetentionPolicy, UnknownMemoryHorizon, UnknownMemoryKind, UnknownMemorySource,
    UnknownRetentionPolicy,
};
pub use agent_output::{
    AgentOutputError, AgentOutputParseFailure, ParsedAgentOutput, RawAgentOutput,
    UnknownAgentApprovalRequirement, ValidatedAgentOutput,
};
pub use agent_run::{
    AgentModelVersion, AgentRun, AgentRunCancelled, AgentRunCompleted, AgentRunError,
    AgentRunFailed, AgentRunLifecycleStatus, AgentRunName, AgentRunPlanning, AgentRunRunning,
    AgentRunWaitingForHuman, DbAgentRunRow, DecodedAgentRun, UnknownAgentRunLifecycleStatus,
};
pub use agent_step::{
    AgentStep, AgentStepError, AgentStepFailed, AgentStepId, AgentStepIndex, AgentStepKind,
    AgentStepPending, AgentStepRef, AgentStepRunning, AgentStepSkipped, AgentStepStatus,
    AgentStepSucceeded, AgentStepTerminalReason, DbAgentStepRow, DecodedAgentStep,
    UnknownAgentStepKind, UnknownAgentStepStatus,
};
#[cfg(all(feature = "api-server", feature = "postgres-store"))]
pub use api::PostgresApiServerError;
#[cfg(all(feature = "api-server", feature = "postgres-store"))]
pub use api::serve_postgres_api;
#[cfg(feature = "api-server")]
pub use api::{
    AgentAdmissionStore, AgentJobAdmission, AgentJobObservabilityStore, ApiError, ApiState,
    CreateAgentJobCommand, CreateAgentJobRequest, CreateAgentJobResponse, HttpBindAddress,
    IdempotencyOutcome, MetricsResponse, QueueMetricsResponse, ReadinessResponse, ReadinessStatus,
    router_with_admission_policy,
};
pub use approval::{
    ApprovalActor, ApprovalDecidedAt, ApprovalError, ApprovalExpiresAt, ApprovalReason,
    ApprovalRecord, ApprovalRequest, ApprovalRequestId, ApprovalRequestedAt, ApprovalStatus,
    Approved, Cancelled, DbHumanApprovalRequestRow, Expired, Rejected, Requested,
    UnknownApprovalStatus,
};
pub use audit::{
    AuditAction, AuditActorId, AuditActorType, AuditError, AuditEventId, AuditEventRecord,
    AuditSubject, DbAuditEventRow, DbOperationEventRow, EvidenceData, OperationEventId,
    OperationEventRecord, OperationEventType, OperationMessage, OperationSeverity, SpanId,
    TraceContext, TraceId, UnknownAuditActorType, UnknownOperationSeverity,
};
pub use background_job::{
    BackgroundCancelled, BackgroundCompleted, BackgroundExecutingAgent, BackgroundFailed,
    BackgroundJob, BackgroundJobError, BackgroundJobId, BackgroundLeased, BackgroundQueued,
    BackgroundWaitingForHuman, BackgroundWaitingForRetry, DbBackgroundJobRow, DecodedBackgroundJob,
    FailureClass, RetryState, UnknownRetryState, UnknownWorkflowState, WorkflowState,
};
pub use cancellation::{
    CancellationActor, CancellationApplied, CancellationAppliedAt, CancellationError,
    CancellationExpired, CancellationExpiresAt, CancellationIgnoredTerminal, CancellationMode,
    CancellationRecord, CancellationRequest, CancellationRequestDraft, CancellationRequestId,
    CancellationRequested, CancellationRequestedAt, CancellationSource, CancellationStatus,
    DbCancellationRequestRow, UnknownCancellationMode, UnknownCancellationSource,
    UnknownCancellationStatus,
};
pub use compatibility::{
    CompatibilityDecision, CompatibilityError, CompatibilityPolicyName, CompatibilityQuarantine,
    CompatibilityQuarantineReason, JobCompatibilityReport, PayloadSchemaCompatibility,
    SupportedPayloadSchemaRange, WorkerCompatibilityPolicy,
};
pub use compensation::{
    ApprovedCompensationAction, CancelledCompensationAction, CompensationActionId,
    CompensationActionRecord, CompensationActionStatus, CompensationEnvelope, CompensationError,
    CompensationKind, CompensationPayload, CompensationReason, DbCompensationActionRow,
    ExecutingCompensationAction, FailedCompensationAction, RequestedCompensationAction,
    SideEffectReceiptId, SucceededCompensationAction, UnknownCompensationStatus,
};
#[cfg(feature = "api-server")]
pub use config::{BIND_ADDRESS_ENV, PostgresApiServerConfig};
pub use config::{
    DATABASE_URL_ENV, EnvVarName, PostgresWorkerConfig, ProcessEnv, RuntimeConfigError, RuntimeEnv,
};
#[cfg(feature = "rig-agent")]
pub use config::{DeepSeekApiKeyPresent, DeepSeekRuntimeConfig};
pub use cost_accounting::{
    BudgetDecision, CompletionTokenCount, CostAccountingError, CostMicrosUsd,
    DbProviderUsageEventRow, LatencyMillis, PromptTokenCount, ProviderCallStatus, ProviderName,
    ProviderUsageEvent, ProviderUsageEventId, ProviderUsageEventInput, TenantBudget,
    TotalTokenCount, UnknownProviderCallStatus,
};
pub use data_protection::{
    DataProtectionError, DataProtectionEvidence, DataProtectionRequestCount, DataProtectionReview,
    DataProtectionReviewStatus, DataProtectionSurface, DbDataProtectionReviewRow,
    UnknownDataProtectionReviewStatus, UnknownDataProtectionSurface,
};
pub use domain::{
    AgentEvent, AgentInstruction, AgentJob, AgentJobSnapshot, AgentJobVersions, AgentPayload,
    AgentResult, AgentRunId, AgentSummary, ApprovalRequirement, AttemptCount, CancellationOutcome,
    CancellationReason, CompletionTransitionOutcome, DatabaseUrl, DomainError, EventMessage,
    FailureMessage, HeartbeatInterval, IdempotencyKey, JobEventType, JobId, JobKind, JobStatus,
    JobTransition, LeaseDuration, LeaseExtensionOutcome, MaxAttempts, ModelRoute, NextAction,
    PayloadSchemaVersion, PolicyVersion, PromptVersion, QueueAge, QueueDepth, QueueMetrics,
    RetryDelay, RetryDisposition, RetryPolicy, RetryTransitionOutcome, ShutdownReason, TenantKey,
    ToolVersion, WorkerBuildId, WorkerId,
};
pub use escalation::{
    DbHumanEscalationRow, EscalationAcknowledgedAt, EscalationCreatedAt, EscalationError,
    EscalationKind, EscalationOwner, EscalationReason, EscalationResolvedAt, EscalationSeverity,
    EscalationStatus, EscalationTarget, HumanEscalationId, HumanEscalationRecord,
    UnknownEscalationKind, UnknownEscalationSeverity, UnknownEscalationStatus,
};
pub use evaluation::{
    BehaviorEvaluator, DbEvaluationRunRow, EvaluationCase, EvaluationCaseCount, EvaluationCaseId,
    EvaluationCaseOutcome, EvaluationCaseResult, EvaluationCaseResults, EvaluationDataset,
    EvaluationDatasetVersion, EvaluationError, EvaluationFailure, EvaluationFailures,
    EvaluationReceipt, EvaluationReport, EvaluationRun, EvaluationRunId, EvaluationRunStatus,
    EvaluationScoreBasisPoints, EvaluatorVersion, GoldenEvaluationDataset, PromotionDecision,
    RequiredOutputTerm, RequiredOutputTerms, TermMatch, UnknownEvaluationRunStatus,
};
pub use failure_drill::{
    EvidenceRequirement, EvidenceRequirements, FailureDrillBlastRadius, FailureDrillDecision,
    FailureDrillError, FailureDrillHypothesis, FailureDrillPlan, FailureDrillReport,
    FailureDrillScenario, FailureInjection, ObservedDrillEvidence, ObservedEventTimeline,
    ObservedWorkerOutcomes, ProviderTimeoutThenRetryDrill, RollbackAction,
    WorkerCrashAfterLeaseDrill, WorkerOutcomeKind,
};
pub use failure_history::{
    DbFailureHistoryRow, FailureHistoryError, FailureHistoryId, FailureHistoryRecord,
    FailureOutcome, FailureSource, UnknownFailureOutcome, UnknownFailureSource,
};
pub use fault_tolerance::{
    AgentJobCount, ControlPlaneStatus, DbFaultToleranceReadinessRow, ExecutionPlaneStatus,
    FailoverDrillStatus, FailureDomain, FaultToleranceError, FaultToleranceOwner,
    FaultToleranceReadinessReview, FaultToleranceReadinessStatus, MinimumWorkerReplicaCount,
    ProgressiveDeliveryChannel, StaticStabilityMode, UnknownFaultToleranceValue,
    WorkerReplicaCount,
};
pub use handoff::{
    AcceptedHandoff, AgentName, CancelledHandoff, DbAgentHandoffRow, ExpiredHandoff,
    HandoffDecisionReason, HandoffEnvelope, HandoffError, HandoffId, HandoffPayload, HandoffReason,
    HandoffRecord, HandoffStatus, RejectedHandoff, RequestedHandoff, UnknownHandoffStatus,
};
pub use job_kind_lifecycle::{
    DbJobKindLifecycleReviewRow, JobKindControlState, JobKindLifecycleError,
    JobKindLifecycleEvidence, JobKindLifecycleRecommendation, JobKindLifecycleReview,
    JobKindPauseReason, LifecycleEvidenceCount, UnknownJobKindLifecycleRecommendation,
};
pub use job_kind_readiness::{
    DbJobKindReadinessReviewRow, JobKindReadinessError, JobKindReadinessReview,
    JobKindReadinessStatus, JobRiskClass, MaturityLevel, NextReadinessChange, ReadinessEvidence,
    ReadinessEvidenceCount, ReadinessOwner, UnknownReadinessValue,
};
pub use kafka_adoption::{
    ConsumerProcessingStatus, ConsumerReceipt, KafkaAdoptionError, KafkaConsumerGroup,
    KafkaConsumerName, KafkaOffset, KafkaPartition, KafkaPublishReceipt, KafkaPublishReceiptInput,
    KafkaRecordRef, KafkaTopic,
};
pub use launch_packet::{
    DbJobKindLaunchPacketStatusRow, FailureDrillRunId, FailureDrillStatus, JobKindLaunchPacket,
    JobKindLaunchPacketId, JobKindReadinessReviewId, LaunchDecision, LaunchEvidenceChecklist,
    LaunchEvidenceStatement, LaunchKnownGap, LaunchKnownGapCount, LaunchKnownGaps, LaunchOwner,
    LaunchPacketError, LaunchReviewer, LaunchStatus, ReviewFreshness, UnknownLaunchPacketValue,
};
pub use logging::{
    LOG_FORMAT_ENV, LogFilterDirective, RUST_LOG_ENV, RuntimeLogFormat, RuntimeTracingConfig,
    RuntimeTracingError, TracingSubscriberInitError, UnknownLogFormat, init_runtime_tracing,
};
pub use observability::{init_telemetry, ObservabilityError};
pub use memory_store::{InMemoryAgentJobStore, InMemoryStoreError};
pub use outbox::{
    DbOutboxEventRow, FailedOutboxEvent, OutboxAggregateId, OutboxEnvelope, OutboxError,
    OutboxEventId, OutboxEventKind, OutboxEventRecord, OutboxPayload, OutboxStatus,
    PendingOutboxEvent, PublishedOutboxEvent, PublishingOutboxEvent, UnknownOutboxStatus,
};
#[cfg(feature = "postgres-store")]
pub use postgres_store::{PostgresAgentJobStore, PostgresStoreError};
pub use recovery::{
    BackupSource, DbRestoreReplayCandidateRow, RecoveryError, RecoveryPointObjectiveSeconds,
    RecoveryTimeObjectiveSeconds, ReplayCandidate, ReplayDecision, ReplayDecisions, RestoreDrillId,
    RestoreDrillInput, RestoreDrillName, RestoreDrillOutcome, RestoreDrillReport, RestoreTarget,
    RestoredReceiptCount, SideEffectEvidence, SideEffectExpectation,
};
pub use release_gate::{
    ReleaseApprovalEvidence, ReleaseBlocker, ReleaseBlockers, ReleaseCandidateId, ReleaseGate,
    ReleaseGateDecision, ReleaseGateError, ReleaseGateInput, ReleaseGateName, ReleaseGateReport,
    ReleaseReason, ReleaseRisk,
};
#[cfg(feature = "rig-agent")]
pub use rig_runner::DeepSeekRigAgentRunner;
pub use sandbox::{
    DbSandboxEventRow, EgressAllowlist, EgressDestination, FilesystemAccessKind,
    FilesystemSandboxPolicy, NetworkSandboxPolicy, RequestedFilesystemAccess,
    RequestedNetworkAccess, SandboxDecisionEvent, SandboxDecisionKind, SandboxDenyReason,
    SandboxError, SandboxEventId, SandboxPath, SecretAccessRequest, SecretSandboxPolicy,
    ToolSandboxPolicy, ToolSandboxRequest, UnknownFilesystemAccess, UnknownSandboxDecision,
    UnknownSecretAccess,
};
pub use scheduled_job::{
    DbScheduledJobRow, DecodedScheduledJob, ScheduledCancelled, ScheduledDead, ScheduledFailed,
    ScheduledFailureTransition, ScheduledJob, ScheduledJobError, ScheduledJobId,
    ScheduledJobPayload, ScheduledPending, ScheduledRunning, ScheduledSucceeded, ScheduledTaskName,
};
pub use security::{
    ActorId, AuthorizationDecisionKind, AuthorizationEvent, AuthorizationEventId,
    AuthorizationPolicy, DbAuthorizationEventRow, DecisionReason, PermissionGrant,
    PermissionGrants, SecretRef, SecurityError, ToolAuthorizationRequest, ToolPermission,
    UnknownAuthorizationDecision, UnknownToolPermission,
};
pub use slo::{
    DbSloMeasurementRow, ErrorBudgetEventCount, ObservedBadEventCount, ObservedGoodEventCount,
    ObservedTotalEventCount, RemainingErrorBudgetEventCount, SliAttainmentBasisPoints, SliName,
    SloDecision, SloError, SloEvaluation, SloMeasurement, SloName, SloTargetBasisPoints, SloWindow,
};
pub use temporal_adoption::{
    TemporalActivityExecutionRef, TemporalActivityReceipt, TemporalActivityReceiptInput,
    TemporalActivityReceipts, TemporalAdoptionError, TemporalProductEvidence,
    TemporalReconciliationPacket, TemporalTaskQueue, TemporalWorkflowBridge,
    TemporalWorkflowBridgeInput, TemporalWorkflowExecutionRef, TemporalWorkflowType,
};
pub use tenant_isolation::{
    DbTenantIsolationReviewRow, TenantIsolationError, TenantIsolationEventCount,
    TenantIsolationEvidence, TenantIsolationReview, TenantIsolationReviewStatus,
    UnknownTenantIsolationReviewStatus,
};
pub use timeouts::{
    DbRunningJobDeadlineRow, ExecutionDeadline, RunningJobDeadline, TimeoutAction, TimeoutDecision,
    TimeoutDuration, TimeoutError, TimeoutObservedState, TimeoutPolicy, TimeoutPolicyName,
    UnknownTimeoutAction, UnknownTimeoutObservedState,
};
pub use tool_call::{
    DbToolCallRow, DecodedToolCall, ToolCall, ToolCallError, ToolCallExecuted, ToolCallFailed,
    ToolCallId, ToolCallInput, ToolCallOutput, ToolCallRejected, ToolCallRequested, ToolCallStatus,
    ToolCallTerminalReason, ToolCallValidated, UnknownToolCallStatus,
};
pub use tool_contract::{
    ApprovedToolRequest, DryRunPauseJobKindTool, HumanApprovalDecision, ParseFailureMessage,
    PauseJobKindRequest, PauseJobKindResult, PolicyCheckedToolRequest, RawModelOutput,
    ToolContractError, ToolInput, ToolName, ToolOutput, ToolPolicyDecision, ToolReason, TypedTool,
    UnknownToolName, ValidatedToolRequest,
};
pub use tool_execution_gate::{
    ToolExecutionApproval, ToolExecutionGate, ToolExecutionGateError, ToolExecutionGateInput,
    TrustedToolExecution,
};
pub use typed_pipeline::{AgentJobBuilder, NeedsInstruction, NeedsKind, ReadyToEnqueue};
pub use worker::{
    AgentJobStore, CompletedWorkerCycles, StoreFailure, Worker, WorkerControl, WorkerCycleLimit,
    WorkerError, WorkerLoopOutcome, WorkerRunOutcome,
};
