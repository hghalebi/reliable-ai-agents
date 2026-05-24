//! Typed release gate for prompt, model, policy, and worker changes.
//!
//! A behavior evaluation receipt is necessary but not sufficient for promotion.
//! A release decision should also consider runtime SLO health, worker
//! compatibility, version consistency, and human approval for high-risk work.

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::approval::{ApprovalRequestId, ApprovalStatus};
use crate::compatibility::{CompatibilityDecision, JobCompatibilityReport};
use crate::domain::{
    AgentJobVersions, DomainError, JobKind, ModelRoute, PayloadSchemaVersion, PolicyVersion,
    PromptVersion, ToolVersion, WorkerBuildId,
};
use crate::evaluation::{EvaluationReceipt, EvaluationRunId, PromotionDecision};
use crate::slo::{SloDecision, SloEvaluation};

fn non_empty_release_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, ReleaseGateError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(ReleaseGateError::EmptyText { field });
    }

    Ok(value)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReleaseGateError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("unknown release risk: {value:?}")]
    UnknownRisk { value: UnknownReleaseGateValue },
    #[error("unknown release gate decision: {value:?}")]
    UnknownDecision { value: UnknownReleaseGateValue },
    #[error("unknown release SLO decision: {value:?}")]
    UnknownSloDecision { value: UnknownReleaseGateValue },
    #[error("unknown release compatibility decision: {value:?}")]
    UnknownCompatibilityDecision { value: UnknownReleaseGateValue },
    #[error("unknown release blocker: {value:?}")]
    UnknownBlocker { value: UnknownReleaseGateValue },
    #[error("release blockers must be a JSON array")]
    BlockersMustBeArray,
    #[error("release blocker entries must be strings")]
    BlockerMustBeString,
    #[error("payload schema version must be positive, got {value}")]
    InvalidPayloadSchemaVersion { value: i32 },
    #[error("canary percent must be between 0 and 100, got {value}")]
    InvalidCanaryPercent { value: i32 },
    #[error(
        "release gate row has inconsistent decision evidence: decision={decision:?}, blocker_count={blocker_count}, canary_percent={canary_percent:?}"
    )]
    InvalidDecisionEvidence {
        decision: ReleaseGateDecision,
        blocker_count: usize,
        canary_percent: CanaryPercent,
    },
    #[error("high-risk non-blocking release rows require durable approval evidence")]
    MissingHighRiskApproval,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownReleaseGateValue(String);

impl UnknownReleaseGateValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReleaseCandidateId(Uuid);

impl ReleaseCandidateId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for ReleaseCandidateId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseGateName(String);

impl ReleaseGateName {
    pub fn new(value: impl Into<String>) -> Result<Self, ReleaseGateError> {
        Ok(Self(non_empty_release_text(value, "release_gate_name")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseReason(String);

impl ReleaseReason {
    pub fn new(value: impl Into<String>) -> Result<Self, ReleaseGateError> {
        Ok(Self(non_empty_release_text(value, "release_reason")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseRisk {
    Low,
    High,
}

impl TryFrom<&str> for ReleaseRisk {
    type Error = ReleaseGateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "low" => Ok(Self::Low),
            "high" => Ok(Self::High),
            value => Err(ReleaseGateError::UnknownRisk {
                value: UnknownReleaseGateValue::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseApprovalEvidence {
    NotRequired,
    Approved,
    NotApproved { status: ApprovalStatus },
}

impl ReleaseApprovalEvidence {
    fn allows(self, risk: ReleaseRisk) -> bool {
        match (risk, self) {
            (ReleaseRisk::Low, Self::NotRequired | Self::Approved) => true,
            (ReleaseRisk::High, Self::Approved) => true,
            (_, Self::NotApproved { .. } | Self::NotRequired) => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseGateDecision {
    Promote,
    CanaryOnly,
    Block,
}

impl TryFrom<&str> for ReleaseGateDecision {
    type Error = ReleaseGateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "promote" => Ok(Self::Promote),
            "canary_only" => Ok(Self::CanaryOnly),
            "block" => Ok(Self::Block),
            value => Err(ReleaseGateError::UnknownDecision {
                value: UnknownReleaseGateValue::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseBlocker {
    EvaluationFailed,
    ErrorBudgetExhausted,
    NoTrafficForFullPromotion,
    IncompatibleWork,
    VersionMismatch {
        evaluation_versions: Box<AgentJobVersions>,
        compatibility_versions: Box<AgentJobVersions>,
    },
    ApprovalMissingOrDenied {
        risk: ReleaseRisk,
        approval: ReleaseApprovalEvidence,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseBlockers(Vec<ReleaseBlocker>);

impl ReleaseBlockers {
    pub fn as_slice(&self) -> &[ReleaseBlocker] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn only_canary_blocker(&self) -> bool {
        self.0
            .iter()
            .all(|blocker| matches!(blocker, ReleaseBlocker::NoTrafficForFullPromotion))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseGateInput {
    pub candidate_id: ReleaseCandidateId,
    pub gate_name: ReleaseGateName,
    pub reason: ReleaseReason,
    pub risk: ReleaseRisk,
    pub evaluation: EvaluationReceipt,
    pub slo: SloEvaluation,
    pub compatibility: JobCompatibilityReport,
    pub approval: ReleaseApprovalEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseGateReport {
    candidate_id: ReleaseCandidateId,
    gate_name: ReleaseGateName,
    reason: ReleaseReason,
    decision: ReleaseGateDecision,
    blockers: ReleaseBlockers,
}

impl ReleaseGateReport {
    pub fn candidate_id(&self) -> ReleaseCandidateId {
        self.candidate_id
    }

    pub fn gate_name(&self) -> &ReleaseGateName {
        &self.gate_name
    }

    pub fn reason(&self) -> &ReleaseReason {
        &self.reason
    }

    pub fn decision(&self) -> ReleaseGateDecision {
        self.decision
    }

    pub fn blockers(&self) -> &ReleaseBlockers {
        &self.blockers
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReleaseGate;

// ANCHOR: release_gate
impl ReleaseGate {
    pub fn evaluate(input: ReleaseGateInput) -> ReleaseGateReport {
        let mut blockers = Vec::new();

        if input.evaluation.decision() != PromotionDecision::Promote {
            blockers.push(ReleaseBlocker::EvaluationFailed);
        }

        match input.slo.decision {
            SloDecision::WithinBudget => {}
            SloDecision::NoTraffic => blockers.push(ReleaseBlocker::NoTrafficForFullPromotion),
            SloDecision::BudgetExhausted => blockers.push(ReleaseBlocker::ErrorBudgetExhausted),
        }

        if !matches!(
            input.compatibility.decision(),
            CompatibilityDecision::Process
        ) {
            blockers.push(ReleaseBlocker::IncompatibleWork);
        }

        if input.evaluation.versions() != input.compatibility.job_versions() {
            blockers.push(ReleaseBlocker::VersionMismatch {
                evaluation_versions: Box::new(input.evaluation.versions().clone()),
                compatibility_versions: Box::new(input.compatibility.job_versions().clone()),
            });
        }

        if !input.approval.allows(input.risk) {
            blockers.push(ReleaseBlocker::ApprovalMissingOrDenied {
                risk: input.risk,
                approval: input.approval,
            });
        }

        let blockers = ReleaseBlockers(blockers);
        let decision = if blockers.is_empty() {
            ReleaseGateDecision::Promote
        } else if blockers.only_canary_blocker() {
            ReleaseGateDecision::CanaryOnly
        } else {
            ReleaseGateDecision::Block
        };

        ReleaseGateReport {
            candidate_id: input.candidate_id,
            gate_name: input.gate_name,
            reason: input.reason,
            decision,
            blockers,
        }
    }
}
// ANCHOR_END: release_gate

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReleaseGateRunId(Uuid);

impl ReleaseGateRunId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for ReleaseGateRunId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaMigrationRunId(Uuid);

impl SchemaMigrationRunId {
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseCompatibilityDecision {
    Process,
    Quarantine,
}

impl TryFrom<&str> for ReleaseCompatibilityDecision {
    type Error = ReleaseGateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "process" => Ok(Self::Process),
            "quarantine" => Ok(Self::Quarantine),
            value => Err(ReleaseGateError::UnknownCompatibilityDecision {
                value: UnknownReleaseGateValue::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseGateRunBlocker {
    EvaluationFailed,
    ErrorBudgetExhausted,
    NoTrafficForFullPromotion,
    IncompatibleWork,
    VersionMismatch,
    ApprovalMissingOrDenied,
}

impl TryFrom<&str> for ReleaseGateRunBlocker {
    type Error = ReleaseGateError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "evaluation_failed" => Ok(Self::EvaluationFailed),
            "error_budget_exhausted" => Ok(Self::ErrorBudgetExhausted),
            "no_traffic_for_full_promotion" => Ok(Self::NoTrafficForFullPromotion),
            "incompatible_work" => Ok(Self::IncompatibleWork),
            "version_mismatch" => Ok(Self::VersionMismatch),
            "approval_missing_or_denied" => Ok(Self::ApprovalMissingOrDenied),
            value => Err(ReleaseGateError::UnknownBlocker {
                value: UnknownReleaseGateValue::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseGateRunBlockers(Vec<ReleaseGateRunBlocker>);

impl ReleaseGateRunBlockers {
    pub fn from_json(value: Value) -> Result<Self, ReleaseGateError> {
        let blockers = value
            .as_array()
            .ok_or(ReleaseGateError::BlockersMustBeArray)?
            .iter()
            .map(|item| {
                let blocker = item.as_str().ok_or(ReleaseGateError::BlockerMustBeString)?;
                ReleaseGateRunBlocker::try_from(blocker)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self(blockers))
    }

    pub fn as_slice(&self) -> &[ReleaseGateRunBlocker] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanaryPercent(u8);

impl CanaryPercent {
    pub fn try_from_i32(value: i32) -> Result<Self, ReleaseGateError> {
        if !(0..=100).contains(&value) {
            return Err(ReleaseGateError::InvalidCanaryPercent { value });
        }

        Ok(Self(value as u8))
    }

    pub fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseRollbackPlan(String);

impl ReleaseRollbackPlan {
    pub fn new(value: impl Into<String>) -> Result<Self, ReleaseGateError> {
        Ok(Self(non_empty_release_text(value, "rollback_plan")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseEvaluator(String);

impl ReleaseEvaluator {
    pub fn new(value: impl Into<String>) -> Result<Self, ReleaseGateError> {
        Ok(Self(non_empty_release_text(value, "evaluated_by")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseOperatorSignoff(String);

impl ReleaseOperatorSignoff {
    pub fn new(value: impl Into<String>) -> Result<Self, ReleaseGateError> {
        Ok(Self(non_empty_release_text(value, "operator_signoff")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ANCHOR: release_gate_run_row_boundary
#[derive(Debug, Clone, PartialEq)]
pub struct DbReleaseGateRunRow {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub gate_name: String,
    pub job_kind: String,
    pub release_reason: String,
    pub risk: String,
    pub decision: String,
    pub prompt_version: String,
    pub model_version: String,
    pub tool_version: String,
    pub policy_version: String,
    pub worker_build_id: String,
    pub payload_schema_version: i32,
    pub evaluation_run_id: Uuid,
    pub schema_migration_run_id: Option<Uuid>,
    pub approval_request_id: Option<Uuid>,
    pub slo_decision: String,
    pub compatibility_decision: String,
    pub blockers: Value,
    pub canary_percent: i32,
    pub rollback_plan: String,
    pub evaluated_by: String,
    pub operator_signoff: String,
    pub evaluated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseGateRunReceipt {
    id: ReleaseGateRunId,
    candidate_id: ReleaseCandidateId,
    gate_name: ReleaseGateName,
    job_kind: JobKind,
    reason: ReleaseReason,
    risk: ReleaseRisk,
    decision: ReleaseGateDecision,
    versions: AgentJobVersions,
    evaluation_run_id: EvaluationRunId,
    schema_migration_run_id: Option<SchemaMigrationRunId>,
    approval_request_id: Option<ApprovalRequestId>,
    slo_decision: SloDecision,
    compatibility_decision: ReleaseCompatibilityDecision,
    blockers: ReleaseGateRunBlockers,
    canary_percent: CanaryPercent,
    rollback_plan: ReleaseRollbackPlan,
    evaluated_by: ReleaseEvaluator,
    operator_signoff: ReleaseOperatorSignoff,
    evaluated_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl ReleaseGateRunReceipt {
    pub fn id(&self) -> ReleaseGateRunId {
        self.id
    }

    pub fn candidate_id(&self) -> ReleaseCandidateId {
        self.candidate_id
    }

    pub fn gate_name(&self) -> &ReleaseGateName {
        &self.gate_name
    }

    pub fn job_kind(&self) -> &JobKind {
        &self.job_kind
    }

    pub fn reason(&self) -> &ReleaseReason {
        &self.reason
    }

    pub fn decision(&self) -> ReleaseGateDecision {
        self.decision
    }

    pub fn risk(&self) -> ReleaseRisk {
        self.risk
    }

    pub fn versions(&self) -> &AgentJobVersions {
        &self.versions
    }

    pub fn evaluation_run_id(&self) -> EvaluationRunId {
        self.evaluation_run_id
    }

    pub fn schema_migration_run_id(&self) -> Option<SchemaMigrationRunId> {
        self.schema_migration_run_id
    }

    pub fn approval_request_id(&self) -> Option<ApprovalRequestId> {
        self.approval_request_id
    }

    pub fn slo_decision(&self) -> SloDecision {
        self.slo_decision
    }

    pub fn compatibility_decision(&self) -> ReleaseCompatibilityDecision {
        self.compatibility_decision
    }

    pub fn blockers(&self) -> &ReleaseGateRunBlockers {
        &self.blockers
    }

    pub fn canary_percent(&self) -> CanaryPercent {
        self.canary_percent
    }

    pub fn rollback_plan(&self) -> &ReleaseRollbackPlan {
        &self.rollback_plan
    }

    pub fn evaluated_by(&self) -> &ReleaseEvaluator {
        &self.evaluated_by
    }

    pub fn operator_signoff(&self) -> &ReleaseOperatorSignoff {
        &self.operator_signoff
    }

    pub fn evaluated_at(&self) -> DateTime<Utc> {
        self.evaluated_at
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

impl TryFrom<DbReleaseGateRunRow> for ReleaseGateRunReceipt {
    type Error = ReleaseGateError;

    fn try_from(row: DbReleaseGateRunRow) -> Result<Self, Self::Error> {
        let risk = ReleaseRisk::try_from(row.risk.as_str())?;
        let decision = ReleaseGateDecision::try_from(row.decision.as_str())?;
        let blockers = ReleaseGateRunBlockers::from_json(row.blockers)?;
        let canary_percent = CanaryPercent::try_from_i32(row.canary_percent)?;
        let approval_request_id = row.approval_request_id.map(ApprovalRequestId::from_uuid);

        validate_release_gate_run_evidence(
            risk,
            decision,
            &blockers,
            canary_percent,
            approval_request_id,
        )?;

        if row.payload_schema_version <= 0 {
            return Err(ReleaseGateError::InvalidPayloadSchemaVersion {
                value: row.payload_schema_version,
            });
        }

        let payload_schema = PayloadSchemaVersion::try_from_u32(row.payload_schema_version as u32)?;

        Ok(Self {
            id: ReleaseGateRunId::from_uuid(row.id),
            candidate_id: ReleaseCandidateId::from_uuid(row.candidate_id),
            gate_name: ReleaseGateName::new(row.gate_name)?,
            job_kind: JobKind::new(row.job_kind)?,
            reason: ReleaseReason::new(row.release_reason)?,
            risk,
            decision,
            versions: AgentJobVersions {
                payload_schema,
                prompt: PromptVersion::new(row.prompt_version)?,
                model_route: ModelRoute::new(row.model_version)?,
                tool: ToolVersion::new(row.tool_version)?,
                policy: PolicyVersion::new(row.policy_version)?,
                worker_build: WorkerBuildId::new(row.worker_build_id)?,
            },
            evaluation_run_id: EvaluationRunId::from_uuid(row.evaluation_run_id),
            schema_migration_run_id: row
                .schema_migration_run_id
                .map(SchemaMigrationRunId::from_uuid),
            approval_request_id,
            slo_decision: release_slo_decision_from_str(row.slo_decision.as_str())?,
            compatibility_decision: ReleaseCompatibilityDecision::try_from(
                row.compatibility_decision.as_str(),
            )?,
            blockers,
            canary_percent,
            rollback_plan: ReleaseRollbackPlan::new(row.rollback_plan)?,
            evaluated_by: ReleaseEvaluator::new(row.evaluated_by)?,
            operator_signoff: ReleaseOperatorSignoff::new(row.operator_signoff)?,
            evaluated_at: row.evaluated_at,
            created_at: row.created_at,
        })
    }
}

fn release_slo_decision_from_str(value: &str) -> Result<SloDecision, ReleaseGateError> {
    match value {
        "within_budget" => Ok(SloDecision::WithinBudget),
        "no_traffic" => Ok(SloDecision::NoTraffic),
        "budget_exhausted" => Ok(SloDecision::BudgetExhausted),
        value => Err(ReleaseGateError::UnknownSloDecision {
            value: UnknownReleaseGateValue::new(value),
        }),
    }
}

fn validate_release_gate_run_evidence(
    risk: ReleaseRisk,
    decision: ReleaseGateDecision,
    blockers: &ReleaseGateRunBlockers,
    canary_percent: CanaryPercent,
    approval_request_id: Option<ApprovalRequestId>,
) -> Result<(), ReleaseGateError> {
    let evidence_matches_decision = match decision {
        ReleaseGateDecision::Promote => blockers.is_empty() && canary_percent.get() == 100,
        ReleaseGateDecision::CanaryOnly => {
            !blockers.is_empty() && (1..100).contains(&canary_percent.get())
        }
        ReleaseGateDecision::Block => !blockers.is_empty() && canary_percent.get() == 0,
    };

    if !evidence_matches_decision {
        return Err(ReleaseGateError::InvalidDecisionEvidence {
            decision,
            blocker_count: blockers.as_slice().len(),
            canary_percent,
        });
    }

    if risk == ReleaseRisk::High
        && decision != ReleaseGateDecision::Block
        && approval_request_id.is_none()
    {
        return Err(ReleaseGateError::MissingHighRiskApproval);
    }

    Ok(())
}
// ANCHOR_END: release_gate_run_row_boundary

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::agent::DeterministicAgentRunner;
    use crate::compatibility::{
        CompatibilityPolicyName, SupportedPayloadSchemaRange, WorkerCompatibilityPolicy,
    };
    use crate::domain::{
        AgentInstruction, AgentJob, AgentPayload, JobKind, MaxAttempts, PayloadSchemaVersion,
        WorkerBuildId,
    };
    use crate::evaluation::{
        BehaviorEvaluator, EvaluationCase, EvaluationDataset, EvaluationDatasetVersion,
        EvaluatorVersion, RequiredOutputTerm, RequiredOutputTerms,
    };
    use crate::slo::{
        ObservedGoodEventCount, ObservedTotalEventCount, SliName, SloMeasurement, SloName,
        SloTargetBasisPoints, SloWindow,
    };

    fn schema(value: u32) -> PayloadSchemaVersion {
        PayloadSchemaVersion::try_from_u32(value).expect("valid schema")
    }

    fn versions() -> AgentJobVersions {
        AgentJobVersions::default()
    }

    fn job_with_versions(versions: AgentJobVersions) -> AgentJob {
        AgentJob::new(
            JobKind::new("incident_triage").expect("valid kind"),
            AgentPayload {
                instruction: AgentInstruction::new("Analyze failed deployment")
                    .expect("valid instruction"),
            },
            MaxAttempts::default(),
            Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0)
                .single()
                .expect("valid time"),
        )
        .with_versions(versions)
    }

    async fn passing_evaluation(versions: AgentJobVersions) -> EvaluationReceipt {
        evaluation_for_required_term(versions, "analyzed").await
    }

    async fn failing_evaluation(versions: AgentJobVersions) -> EvaluationReceipt {
        evaluation_for_required_term(versions, "rollback").await
    }

    async fn evaluation_for_required_term(
        versions: AgentJobVersions,
        required_term: &str,
    ) -> EvaluationReceipt {
        let dataset = EvaluationDataset::new(
            EvaluationDatasetVersion::new("incident-triage:v1").expect("valid dataset"),
            [EvaluationCase::new(
                crate::evaluation::EvaluationCaseId::new("incident-basic").expect("valid case"),
                AgentPayload {
                    instruction: AgentInstruction::new("Analyze failed deployment")
                        .expect("valid instruction"),
                },
                RequiredOutputTerms::new([
                    RequiredOutputTerm::new(required_term).expect("valid term")
                ])
                .expect("non-empty terms"),
                crate::domain::ApprovalRequirement::Required,
            )],
        )
        .expect("non-empty dataset");

        BehaviorEvaluator::new(EvaluatorVersion::new("keyword:v1").expect("valid evaluator"))
            .evaluate_runner(&dataset, &DeterministicAgentRunner, versions)
            .await
    }

    fn compatibility_report(job: &AgentJob) -> JobCompatibilityReport {
        WorkerCompatibilityPolicy::new(
            CompatibilityPolicyName::new("release-gate:v1").expect("valid policy"),
            WorkerBuildId::new("worker-2026-05-23").expect("valid worker"),
            SupportedPayloadSchemaRange::new(schema(1), schema(1)).expect("valid range"),
        )
        .evaluate(job)
    }

    fn slo_evaluation(good: u64, total: u64) -> SloEvaluation {
        SloMeasurement::new(
            SloName::new("job-start-latency:v1").expect("valid slo"),
            SliName::new("job_start_latency_within_120s").expect("valid sli"),
            Some(JobKind::new("incident_triage").expect("valid kind")),
            SloWindow::new(
                Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0)
                    .single()
                    .expect("valid start"),
                Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0)
                    .single()
                    .expect("valid end"),
            )
            .expect("valid window"),
            SloTargetBasisPoints::new(9_900).expect("valid target"),
            ObservedGoodEventCount::new(good),
            ObservedTotalEventCount::new(total),
        )
        .expect("valid measurement")
        .evaluate()
    }

    async fn release_input() -> ReleaseGateInput {
        let job = job_with_versions(versions());
        ReleaseGateInput {
            candidate_id: ReleaseCandidateId::new(),
            gate_name: ReleaseGateName::new("incident-triage-release").expect("valid gate"),
            reason: ReleaseReason::new("promote prompt and model route").expect("valid reason"),
            risk: ReleaseRisk::High,
            evaluation: passing_evaluation(versions()).await,
            slo: slo_evaluation(991, 1_000),
            compatibility: compatibility_report(&job),
            approval: ReleaseApprovalEvidence::Approved,
        }
    }

    #[tokio::test]
    async fn release_gate_promotes_when_all_evidence_is_green() {
        let report = ReleaseGate::evaluate(release_input().await);

        assert_eq!(report.decision(), ReleaseGateDecision::Promote);
        assert!(report.blockers().is_empty());
    }

    #[tokio::test]
    async fn release_gate_blocks_when_evaluation_failed() {
        let mut input = release_input().await;
        input.evaluation = failing_evaluation(versions()).await;

        let report = ReleaseGate::evaluate(input);

        assert_eq!(report.decision(), ReleaseGateDecision::Block);
        assert!(
            report
                .blockers()
                .as_slice()
                .contains(&ReleaseBlocker::EvaluationFailed)
        );
    }

    #[tokio::test]
    async fn release_gate_canary_only_when_no_traffic_evidence_exists() {
        let mut input = release_input().await;
        input.risk = ReleaseRisk::Low;
        input.approval = ReleaseApprovalEvidence::NotRequired;
        input.slo = slo_evaluation(0, 0);

        let report = ReleaseGate::evaluate(input);

        assert_eq!(report.decision(), ReleaseGateDecision::CanaryOnly);
        assert_eq!(
            report.blockers().as_slice(),
            &[ReleaseBlocker::NoTrafficForFullPromotion]
        );
    }

    #[tokio::test]
    async fn release_gate_blocks_when_error_budget_is_exhausted() {
        let mut input = release_input().await;
        input.slo = slo_evaluation(989, 1_000);

        let report = ReleaseGate::evaluate(input);

        assert_eq!(report.decision(), ReleaseGateDecision::Block);
        assert!(
            report
                .blockers()
                .as_slice()
                .contains(&ReleaseBlocker::ErrorBudgetExhausted)
        );
    }

    #[tokio::test]
    async fn release_gate_blocks_incompatible_work() {
        let mut incompatible_versions = versions();
        incompatible_versions.payload_schema = schema(2);
        let job = job_with_versions(incompatible_versions);
        let mut input = release_input().await;
        input.compatibility = compatibility_report(&job);

        let report = ReleaseGate::evaluate(input);

        assert_eq!(report.decision(), ReleaseGateDecision::Block);
        assert!(
            report
                .blockers()
                .as_slice()
                .contains(&ReleaseBlocker::IncompatibleWork)
        );
    }

    #[tokio::test]
    async fn release_gate_blocks_version_mismatch_between_eval_and_compatibility() {
        let mut mismatched_versions = versions();
        mismatched_versions.prompt =
            crate::domain::PromptVersion::new("incident-triage:v2").expect("valid prompt");
        let mut input = release_input().await;
        input.evaluation = passing_evaluation(mismatched_versions).await;
        let report = ReleaseGate::evaluate(input);

        assert!(
            report
                .blockers()
                .as_slice()
                .iter()
                .any(|blocker| matches!(blocker, ReleaseBlocker::VersionMismatch { .. }))
        );
    }

    #[tokio::test]
    async fn high_risk_release_requires_approved_human_evidence() {
        let mut input = release_input().await;
        input.approval = ReleaseApprovalEvidence::NotApproved {
            status: ApprovalStatus::Requested,
        };

        let report = ReleaseGate::evaluate(input);

        assert_eq!(report.decision(), ReleaseGateDecision::Block);
        assert!(
            report
                .blockers()
                .as_slice()
                .iter()
                .any(|blocker| matches!(blocker, ReleaseBlocker::ApprovalMissingOrDenied { .. }))
        );
    }

    fn release_gate_row(decision: &str) -> DbReleaseGateRunRow {
        let blockers = match decision {
            "promote" => serde_json::json!([]),
            "canary_only" => serde_json::json!(["no_traffic_for_full_promotion"]),
            "block" => serde_json::json!(["evaluation_failed"]),
            _ => serde_json::json!([]),
        };
        let canary_percent = match decision {
            "promote" => 100,
            "canary_only" => 10,
            _ => 0,
        };
        let approval_request_id = match decision {
            "block" => None,
            _ => Some(Uuid::new_v4()),
        };
        let now = Utc
            .with_ymd_and_hms(2026, 5, 24, 9, 0, 0)
            .single()
            .expect("valid time");

        DbReleaseGateRunRow {
            id: Uuid::new_v4(),
            candidate_id: Uuid::new_v4(),
            gate_name: "incident-triage-release".to_string(),
            job_kind: "incident_triage".to_string(),
            release_reason: "promote tested prompt and model route".to_string(),
            risk: "high".to_string(),
            decision: decision.to_string(),
            prompt_version: "incident-triage:v1".to_string(),
            model_version: "deepseek-chat:v1".to_string(),
            tool_version: "ops-tools:v1".to_string(),
            policy_version: "approval-policy:v1".to_string(),
            worker_build_id: "worker-2026-05-24".to_string(),
            payload_schema_version: 1,
            evaluation_run_id: Uuid::new_v4(),
            schema_migration_run_id: Some(Uuid::new_v4()),
            approval_request_id,
            slo_decision: "within_budget".to_string(),
            compatibility_decision: "process".to_string(),
            blockers,
            canary_percent,
            rollback_plan: "restore previous prompt and model route".to_string(),
            evaluated_by: "release-bot".to_string(),
            operator_signoff: "ops-oncall".to_string(),
            evaluated_at: now,
            created_at: now,
        }
    }

    #[test]
    fn release_gate_row_conversion_accepts_valid_promotion_receipt() {
        let receipt = ReleaseGateRunReceipt::try_from(release_gate_row("promote"))
            .expect("valid release gate row");

        assert_eq!(receipt.decision(), ReleaseGateDecision::Promote);
        assert_eq!(receipt.risk(), ReleaseRisk::High);
        assert_eq!(receipt.canary_percent().get(), 100);
        assert!(receipt.blockers().is_empty());
        assert!(receipt.approval_request_id().is_some());
        assert_eq!(receipt.versions().payload_schema.get(), 1);
    }

    #[test]
    fn release_gate_row_conversion_accepts_canary_receipt_with_blocker() {
        let mut row = release_gate_row("canary_only");
        row.risk = "low".to_string();
        row.approval_request_id = None;

        let receipt = ReleaseGateRunReceipt::try_from(row).expect("valid canary row");

        assert_eq!(receipt.decision(), ReleaseGateDecision::CanaryOnly);
        assert_eq!(receipt.canary_percent().get(), 10);
        assert_eq!(
            receipt.blockers().as_slice(),
            &[ReleaseGateRunBlocker::NoTrafficForFullPromotion]
        );
    }

    #[test]
    fn release_gate_row_conversion_rejects_unknown_decision() {
        let mut row = release_gate_row("promote");
        row.decision = "ship_it".to_string();

        let error = ReleaseGateRunReceipt::try_from(row).expect_err("unknown decision rejected");

        assert!(matches!(error, ReleaseGateError::UnknownDecision { .. }));
    }

    #[test]
    fn release_gate_row_conversion_rejects_high_risk_promotion_without_approval() {
        let mut row = release_gate_row("promote");
        row.approval_request_id = None;

        let error = ReleaseGateRunReceipt::try_from(row).expect_err("approval required");

        assert_eq!(error, ReleaseGateError::MissingHighRiskApproval);
    }

    #[test]
    fn release_gate_row_conversion_rejects_decision_evidence_mismatch() {
        let mut row = release_gate_row("promote");
        row.blockers = serde_json::json!(["evaluation_failed"]);

        let error = ReleaseGateRunReceipt::try_from(row).expect_err("mismatched evidence rejected");

        assert!(matches!(
            error,
            ReleaseGateError::InvalidDecisionEvidence {
                decision: ReleaseGateDecision::Promote,
                blocker_count: 1,
                ..
            }
        ));
    }

    #[test]
    fn release_gate_row_conversion_rejects_non_array_blockers() {
        let mut row = release_gate_row("promote");
        row.blockers = serde_json::json!({"kind": "evaluation_failed"});

        let error = ReleaseGateRunReceipt::try_from(row).expect_err("blockers must be array");

        assert_eq!(error, ReleaseGateError::BlockersMustBeArray);
    }

    #[test]
    fn release_gate_row_conversion_rejects_unknown_blocker() {
        let mut row = release_gate_row("block");
        row.blockers = serde_json::json!(["mystery_blocker"]);

        let error = ReleaseGateRunReceipt::try_from(row).expect_err("unknown blocker rejected");

        assert!(matches!(error, ReleaseGateError::UnknownBlocker { .. }));
    }

    #[test]
    fn release_gate_row_conversion_rejects_invalid_payload_schema_version() {
        let mut row = release_gate_row("promote");
        row.payload_schema_version = 0;

        let error = ReleaseGateRunReceipt::try_from(row).expect_err("schema version rejected");

        assert_eq!(
            error,
            ReleaseGateError::InvalidPayloadSchemaVersion { value: 0 }
        );
    }
}
