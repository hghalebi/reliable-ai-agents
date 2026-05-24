//! Typed behavior-evaluation primitives for agent release gates.
//!
//! This module keeps evaluation data inside the domain model: dataset versions,
//! evaluator versions, expected approval behavior, required output evidence,
//! and promotion decisions are named types rather than loose strings or JSON.

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::agent::{AgentError, AgentRunner};
use crate::domain::{
    AgentJobVersions, AgentPayload, AgentResult, AgentRunId, AgentSummary, ApprovalRequirement,
    DomainError, FailureMessage, ModelRoute, PolicyVersion, PromptVersion, ToolVersion,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvaluationError {
    #[error("evaluation dataset must contain at least one case")]
    EmptyDataset,
    #[error("evaluation case must require at least one output term")]
    EmptyRequiredOutputTerms,
    #[error("unknown evaluation run status: {value}")]
    UnknownEvaluationRunStatus { value: UnknownEvaluationRunStatus },
    #[error("evaluation score basis points must be between 0 and 10000, got {value}")]
    InvalidEvaluationScoreBasisPoints { value: i64 },
    #[error("evaluation report must be a JSON object")]
    ReportMustBeObject,
    #[error("{status:?} evaluation run requires score and completed_at")]
    TerminalRunMissingEvidence { status: EvaluationRunStatus },
    #[error("{status:?} evaluation run must not have score or completed_at")]
    ActiveRunHasTerminalEvidence { status: EvaluationRunStatus },
    #[error("evaluation run completed before it was created")]
    CompletedBeforeCreated,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_evaluation_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, DomainError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(DomainError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationDatasetVersion(String);

impl EvaluationDatasetVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_evaluation_text(
            value,
            "evaluation_dataset_version",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatorVersion(String);

impl EvaluatorVersion {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_evaluation_text(value, "evaluator_version")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownEvaluationRunStatus(String);

impl UnknownEvaluationRunStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownEvaluationRunStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationRunStatus {
    Queued,
    Running,
    Passed,
    Failed,
}

impl EvaluationRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }

    fn is_terminal(self) -> bool {
        matches!(self, Self::Passed | Self::Failed)
    }
}

impl TryFrom<&str> for EvaluationRunStatus {
    type Error = EvaluationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            value => Err(EvaluationError::UnknownEvaluationRunStatus {
                value: UnknownEvaluationRunStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EvaluationRunId(Uuid);

impl EvaluationRunId {
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

impl Default for EvaluationRunId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EvaluationScoreBasisPoints(u16);

impl EvaluationScoreBasisPoints {
    pub fn try_from_i64(value: i64) -> Result<Self, EvaluationError> {
        if !(0..=10_000).contains(&value) {
            return Err(EvaluationError::InvalidEvaluationScoreBasisPoints { value });
        }

        Ok(Self(value as u16))
    }

    pub fn basis_points(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationReport(Value);

impl EvaluationReport {
    pub fn new(value: Value) -> Result<Self, EvaluationError> {
        if !value.is_object() {
            return Err(EvaluationError::ReportMustBeObject);
        }

        Ok(Self(value))
    }

    pub fn as_json(&self) -> &Value {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationRun {
    id: EvaluationRunId,
    run_id: Option<AgentRunId>,
    dataset_version: EvaluationDatasetVersion,
    evaluator_version: EvaluatorVersion,
    versions: AgentJobVersions,
    status: EvaluationRunStatus,
    score: Option<EvaluationScoreBasisPoints>,
    report: EvaluationReport,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

impl EvaluationRun {
    pub fn id(&self) -> EvaluationRunId {
        self.id
    }

    pub fn run_id(&self) -> Option<AgentRunId> {
        self.run_id
    }

    pub fn dataset_version(&self) -> &EvaluationDatasetVersion {
        &self.dataset_version
    }

    pub fn evaluator_version(&self) -> &EvaluatorVersion {
        &self.evaluator_version
    }

    pub fn versions(&self) -> &AgentJobVersions {
        &self.versions
    }

    pub fn status(&self) -> EvaluationRunStatus {
        self.status
    }

    pub fn score(&self) -> Option<EvaluationScoreBasisPoints> {
        self.score
    }

    pub fn report(&self) -> &EvaluationReport {
        &self.report
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }
}

// ANCHOR: evaluation_run_row_boundary
#[derive(Debug, Clone, PartialEq)]
pub struct DbEvaluationRunRow {
    pub id: Uuid,
    pub run_id: Option<Uuid>,
    pub dataset_version: String,
    pub evaluator_version: String,
    pub prompt_version: String,
    pub model_version: String,
    pub tool_version: String,
    pub policy_version: String,
    pub status: String,
    pub score_basis_points: Option<i64>,
    pub report: Value,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl TryFrom<DbEvaluationRunRow> for EvaluationRun {
    type Error = EvaluationError;

    fn try_from(row: DbEvaluationRunRow) -> Result<Self, Self::Error> {
        let status = EvaluationRunStatus::try_from(row.status.as_str())?;
        let score = row
            .score_basis_points
            .map(EvaluationScoreBasisPoints::try_from_i64)
            .transpose()?;

        validate_evaluation_run_evidence(status, score, row.created_at, row.completed_at)?;

        Ok(Self {
            id: EvaluationRunId::from_uuid(row.id),
            run_id: row.run_id.map(AgentRunId::from_uuid),
            dataset_version: EvaluationDatasetVersion::new(row.dataset_version)?,
            evaluator_version: EvaluatorVersion::new(row.evaluator_version)?,
            versions: AgentJobVersions {
                prompt: PromptVersion::new(row.prompt_version)?,
                model_route: ModelRoute::new(row.model_version)?,
                tool: ToolVersion::new(row.tool_version)?,
                policy: PolicyVersion::new(row.policy_version)?,
                ..AgentJobVersions::default()
            },
            status,
            score,
            report: EvaluationReport::new(row.report)?,
            created_at: row.created_at,
            completed_at: row.completed_at,
        })
    }
}

fn validate_evaluation_run_evidence(
    status: EvaluationRunStatus,
    score: Option<EvaluationScoreBasisPoints>,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
) -> Result<(), EvaluationError> {
    if let Some(completed_at) = completed_at
        && completed_at < created_at
    {
        return Err(EvaluationError::CompletedBeforeCreated);
    }

    if status.is_terminal() {
        if score.is_none() || completed_at.is_none() {
            return Err(EvaluationError::TerminalRunMissingEvidence { status });
        }
    } else if score.is_some() || completed_at.is_some() {
        return Err(EvaluationError::ActiveRunHasTerminalEvidence { status });
    }

    Ok(())
}
// ANCHOR_END: evaluation_run_row_boundary

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationCaseId(String);

impl EvaluationCaseId {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_evaluation_text(
            value,
            "evaluation_case_id",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredOutputTerm(String);

impl RequiredOutputTerm {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(non_empty_evaluation_text(
            value,
            "required_output_term",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn match_summary(&self, summary: &AgentSummary) -> TermMatch {
        let summary = summary.as_str().to_ascii_lowercase();
        let required = self.0.to_ascii_lowercase();

        if summary.contains(&required) {
            TermMatch::Present
        } else {
            TermMatch::Missing
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermMatch {
    Present,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredOutputTerms(Vec<RequiredOutputTerm>);

impl RequiredOutputTerms {
    pub fn new(
        values: impl IntoIterator<Item = RequiredOutputTerm>,
    ) -> Result<Self, EvaluationError> {
        let values: Vec<_> = values.into_iter().collect();

        if values.is_empty() {
            return Err(EvaluationError::EmptyRequiredOutputTerms);
        }

        Ok(Self(values))
    }

    pub fn as_slice(&self) -> &[RequiredOutputTerm] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationCase {
    id: EvaluationCaseId,
    payload: AgentPayload,
    required_summary_terms: RequiredOutputTerms,
    expected_approval: ApprovalRequirement,
}

impl EvaluationCase {
    pub fn new(
        id: EvaluationCaseId,
        payload: AgentPayload,
        required_summary_terms: RequiredOutputTerms,
        expected_approval: ApprovalRequirement,
    ) -> Self {
        Self {
            id,
            payload,
            required_summary_terms,
            expected_approval,
        }
    }

    pub fn id(&self) -> &EvaluationCaseId {
        &self.id
    }

    fn payload(&self) -> AgentPayload {
        self.payload.clone()
    }

    fn evaluate_result(&self, result: &AgentResult) -> EvaluationCaseOutcome {
        let mut failures = Vec::new();

        for required_term in self.required_summary_terms.as_slice() {
            if required_term.match_summary(&result.summary) == TermMatch::Missing {
                failures.push(EvaluationFailure::MissingRequiredSummaryTerm {
                    term: required_term.clone(),
                });
            }
        }

        if result.approval != self.expected_approval {
            failures.push(EvaluationFailure::UnexpectedApprovalRequirement {
                expected: self.expected_approval,
                actual: result.approval,
            });
        }

        if failures.is_empty() {
            EvaluationCaseOutcome::Passed
        } else {
            EvaluationCaseOutcome::Failed(EvaluationFailures(failures))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationDataset {
    version: EvaluationDatasetVersion,
    cases: Vec<EvaluationCase>,
}

impl EvaluationDataset {
    pub fn new(
        version: EvaluationDatasetVersion,
        cases: impl IntoIterator<Item = EvaluationCase>,
    ) -> Result<Self, EvaluationError> {
        let cases: Vec<_> = cases.into_iter().collect();

        if cases.is_empty() {
            return Err(EvaluationError::EmptyDataset);
        }

        Ok(Self { version, cases })
    }

    pub fn version(&self) -> &EvaluationDatasetVersion {
        &self.version
    }

    pub fn cases(&self) -> &[EvaluationCase] {
        &self.cases
    }
}

// ANCHOR: golden_dataset
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldenEvaluationDataset(EvaluationDataset);

impl GoldenEvaluationDataset {
    pub fn new(dataset: EvaluationDataset) -> Self {
        Self(dataset)
    }

    pub fn as_dataset(&self) -> &EvaluationDataset {
        &self.0
    }

    pub fn version(&self) -> &EvaluationDatasetVersion {
        self.0.version()
    }
}
// ANCHOR_END: golden_dataset

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorEvaluator {
    version: EvaluatorVersion,
}

// ANCHOR: behavior_eval_gate
impl BehaviorEvaluator {
    pub fn new(version: EvaluatorVersion) -> Self {
        Self { version }
    }

    pub async fn evaluate_runner<R>(
        &self,
        dataset: &EvaluationDataset,
        runner: &R,
        versions: AgentJobVersions,
    ) -> EvaluationReceipt
    where
        R: AgentRunner,
    {
        let mut results = Vec::new();

        for case in dataset.cases() {
            let outcome = match runner.run_agent(case.payload()).await {
                Ok(result) => case.evaluate_result(&result),
                Err(error) => EvaluationCaseOutcome::Failed(EvaluationFailures(vec![
                    EvaluationFailure::from(error),
                ])),
            };

            results.push(EvaluationCaseResult::new(case.id().clone(), outcome));
        }

        EvaluationReceipt::new(
            dataset.version().clone(),
            self.version.clone(),
            versions,
            EvaluationCaseResults(results),
        )
    }

    pub async fn evaluate_golden_dataset<R>(
        &self,
        dataset: &GoldenEvaluationDataset,
        runner: &R,
        versions: AgentJobVersions,
    ) -> EvaluationReceipt
    where
        R: AgentRunner,
    {
        self.evaluate_runner(dataset.as_dataset(), runner, versions)
            .await
    }
}
// ANCHOR_END: behavior_eval_gate

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationCaseResult {
    case_id: EvaluationCaseId,
    outcome: EvaluationCaseOutcome,
}

impl EvaluationCaseResult {
    pub fn new(case_id: EvaluationCaseId, outcome: EvaluationCaseOutcome) -> Self {
        Self { case_id, outcome }
    }

    pub fn case_id(&self) -> &EvaluationCaseId {
        &self.case_id
    }

    pub fn outcome(&self) -> &EvaluationCaseOutcome {
        &self.outcome
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationCaseOutcome {
    Passed,
    Failed(EvaluationFailures),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationFailures(Vec<EvaluationFailure>);

impl EvaluationFailures {
    pub fn as_slice(&self) -> &[EvaluationFailure] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationFailure {
    MissingRequiredSummaryTerm {
        term: RequiredOutputTerm,
    },
    UnexpectedApprovalRequirement {
        expected: ApprovalRequirement,
        actual: ApprovalRequirement,
    },
    AgentFailed {
        message: FailureMessage,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationCaseResults(Vec<EvaluationCaseResult>);

impl EvaluationCaseResults {
    pub fn as_slice(&self) -> &[EvaluationCaseResult] {
        &self.0
    }

    fn passed_count(&self) -> EvaluationCaseCount {
        EvaluationCaseCount::from_usize(
            self.0
                .iter()
                .filter(|result| matches!(result.outcome(), EvaluationCaseOutcome::Passed))
                .count(),
        )
    }

    fn failed_count(&self) -> EvaluationCaseCount {
        EvaluationCaseCount::from_usize(
            self.0
                .iter()
                .filter(|result| matches!(result.outcome(), EvaluationCaseOutcome::Failed(_)))
                .count(),
        )
    }

    fn total_count(&self) -> EvaluationCaseCount {
        EvaluationCaseCount::from_usize(self.0.len())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvaluationCaseCount(usize);

impl EvaluationCaseCount {
    pub fn from_usize(value: usize) -> Self {
        Self(value)
    }

    pub fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionDecision {
    Promote,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationReceipt {
    dataset_version: EvaluationDatasetVersion,
    evaluator_version: EvaluatorVersion,
    versions: AgentJobVersions,
    total_cases: EvaluationCaseCount,
    passed_cases: EvaluationCaseCount,
    failed_cases: EvaluationCaseCount,
    decision: PromotionDecision,
    case_results: EvaluationCaseResults,
}

impl EvaluationReceipt {
    pub fn new(
        dataset_version: EvaluationDatasetVersion,
        evaluator_version: EvaluatorVersion,
        versions: AgentJobVersions,
        case_results: EvaluationCaseResults,
    ) -> Self {
        let failed_cases = case_results.failed_count();
        let decision = if failed_cases.get() == 0 {
            PromotionDecision::Promote
        } else {
            PromotionDecision::Block
        };

        Self {
            dataset_version,
            evaluator_version,
            versions,
            total_cases: case_results.total_count(),
            passed_cases: case_results.passed_count(),
            failed_cases,
            decision,
            case_results,
        }
    }

    pub fn dataset_version(&self) -> &EvaluationDatasetVersion {
        &self.dataset_version
    }

    pub fn evaluator_version(&self) -> &EvaluatorVersion {
        &self.evaluator_version
    }

    pub fn versions(&self) -> &AgentJobVersions {
        &self.versions
    }

    pub fn total_cases(&self) -> EvaluationCaseCount {
        self.total_cases
    }

    pub fn passed_cases(&self) -> EvaluationCaseCount {
        self.passed_cases
    }

    pub fn failed_cases(&self) -> EvaluationCaseCount {
        self.failed_cases
    }

    pub fn decision(&self) -> PromotionDecision {
        self.decision
    }

    pub fn case_results(&self) -> &EvaluationCaseResults {
        &self.case_results
    }
}

impl From<AgentError> for EvaluationFailure {
    fn from(error: AgentError) -> Self {
        Self::AgentFailed {
            message: FailureMessage::from_error_text(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};
    use serde_json::json;

    use crate::agent::{DeterministicAgentRunner, PermanentFailureRunner};
    use crate::domain::{AgentInstruction, ModelRoute, PromptVersion};

    use super::*;

    fn payload(instruction: &str) -> AgentPayload {
        AgentPayload {
            instruction: AgentInstruction::new(instruction).expect("valid instruction"),
        }
    }

    fn terms(values: &[&str]) -> RequiredOutputTerms {
        RequiredOutputTerms::new(
            values
                .iter()
                .map(|value| RequiredOutputTerm::new(*value).expect("valid term")),
        )
        .expect("non-empty terms")
    }

    fn case(id: &str, instruction: &str, required_terms: &[&str]) -> EvaluationCase {
        EvaluationCase::new(
            EvaluationCaseId::new(id).expect("valid case id"),
            payload(instruction),
            terms(required_terms),
            ApprovalRequirement::Required,
        )
    }

    fn dataset(cases: Vec<EvaluationCase>) -> EvaluationDataset {
        EvaluationDataset::new(
            EvaluationDatasetVersion::new("incident-triage-evals:v1").expect("valid dataset"),
            cases,
        )
        .expect("non-empty dataset")
    }

    fn golden_dataset(cases: Vec<EvaluationCase>) -> GoldenEvaluationDataset {
        GoldenEvaluationDataset::new(dataset(cases))
    }

    fn evaluator() -> BehaviorEvaluator {
        BehaviorEvaluator::new(EvaluatorVersion::new("keyword-rubric:v1").expect("valid version"))
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 10, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn evaluation_run_row() -> DbEvaluationRunRow {
        DbEvaluationRunRow {
            id: Uuid::new_v4(),
            run_id: Some(Uuid::new_v4()),
            dataset_version: "incident-triage-evals:v1".to_string(),
            evaluator_version: "keyword-rubric:v1".to_string(),
            prompt_version: "incident-triage:v1".to_string(),
            model_version: "deepseek-chat:v1".to_string(),
            tool_version: "incident-tools:v1".to_string(),
            policy_version: "approval-policy:v1".to_string(),
            status: "passed".to_string(),
            score_basis_points: Some(9_800),
            report: json!({ "failed_cases": [] }),
            created_at: now(),
            completed_at: Some(now() + Duration::seconds(12)),
        }
    }

    #[tokio::test]
    async fn passing_evaluation_allows_promotion() {
        let receipt = evaluator()
            .evaluate_runner(
                &dataset(vec![case(
                    "incident-basic",
                    "Analyze failed deployment",
                    &["analyzed"],
                )]),
                &DeterministicAgentRunner,
                AgentJobVersions::default(),
            )
            .await;

        assert_eq!(receipt.total_cases().get(), 1);
        assert_eq!(receipt.passed_cases().get(), 1);
        assert_eq!(receipt.failed_cases().get(), 0);
        assert_eq!(receipt.decision(), PromotionDecision::Promote);
    }

    #[tokio::test]
    async fn missing_required_term_blocks_promotion() {
        let receipt = evaluator()
            .evaluate_runner(
                &dataset(vec![case(
                    "incident-missing-rollback",
                    "Analyze failed deployment",
                    &["rollback"],
                )]),
                &DeterministicAgentRunner,
                AgentJobVersions {
                    prompt: PromptVersion::new("incident-triage:v2").expect("valid prompt"),
                    model_route: ModelRoute::new("deepseek-chat").expect("valid model"),
                    ..AgentJobVersions::default()
                },
            )
            .await;

        assert_eq!(receipt.failed_cases().get(), 1);
        assert_eq!(receipt.decision(), PromotionDecision::Block);
        assert!(matches!(
            receipt.case_results().as_slice()[0].outcome(),
            EvaluationCaseOutcome::Failed(_)
        ));
        assert_eq!(receipt.versions().prompt.as_str(), "incident-triage:v2");
        assert_eq!(receipt.versions().model_route.as_str(), "deepseek-chat");
    }

    #[tokio::test]
    async fn agent_failure_blocks_promotion() {
        let receipt = evaluator()
            .evaluate_runner(
                &dataset(vec![case(
                    "provider-failure",
                    "Analyze failed deployment",
                    &["analyzed"],
                )]),
                &PermanentFailureRunner,
                AgentJobVersions::default(),
            )
            .await;

        assert_eq!(receipt.failed_cases().get(), 1);
        assert_eq!(receipt.decision(), PromotionDecision::Block);
    }

    #[tokio::test]
    async fn golden_dataset_evaluation_uses_versioned_release_evidence() {
        let golden = golden_dataset(vec![case(
            "incident-golden-rollback",
            "Analyze failed deployment",
            &["analyzed"],
        )]);

        let receipt = evaluator()
            .evaluate_golden_dataset(
                &golden,
                &DeterministicAgentRunner,
                AgentJobVersions::default(),
            )
            .await;

        assert_eq!(golden.version().as_str(), "incident-triage-evals:v1");
        assert_eq!(
            receipt.dataset_version().as_str(),
            golden.version().as_str()
        );
        assert_eq!(receipt.decision(), PromotionDecision::Promote);
    }

    #[test]
    fn dataset_rejects_empty_cases() {
        let error = EvaluationDataset::new(
            EvaluationDatasetVersion::new("empty:v1").expect("valid dataset"),
            Vec::new(),
        )
        .expect_err("empty dataset must fail");

        assert_eq!(error, EvaluationError::EmptyDataset);
    }

    #[test]
    fn db_row_conversion_accepts_valid_evaluation_run_receipt() {
        let run = EvaluationRun::try_from(evaluation_run_row()).expect("valid evaluation row");

        assert_eq!(run.status(), EvaluationRunStatus::Passed);
        assert_eq!(
            run.score()
                .expect("terminal evaluation has score")
                .basis_points(),
            9_800
        );
        assert_eq!(run.dataset_version().as_str(), "incident-triage-evals:v1");
        assert_eq!(run.versions().prompt.as_str(), "incident-triage:v1");
        assert_eq!(run.versions().model_route.as_str(), "deepseek-chat:v1");
        assert_eq!(run.versions().tool.as_str(), "incident-tools:v1");
        assert_eq!(run.versions().policy.as_str(), "approval-policy:v1");
        assert!(run.report().as_json().is_object());
    }

    #[test]
    fn db_row_conversion_rejects_unknown_evaluation_status() {
        let mut row = evaluation_run_row();
        row.status = "almost_passed".to_string();

        let error = EvaluationRun::try_from(row).expect_err("unknown status must fail");

        assert!(matches!(
            error,
            EvaluationError::UnknownEvaluationRunStatus { .. }
        ));
    }

    #[test]
    fn db_row_conversion_rejects_terminal_run_without_score() {
        let mut row = evaluation_run_row();
        row.score_basis_points = None;

        let error = EvaluationRun::try_from(row).expect_err("terminal run needs score");

        assert_eq!(
            error,
            EvaluationError::TerminalRunMissingEvidence {
                status: EvaluationRunStatus::Passed
            }
        );
    }

    #[test]
    fn db_row_conversion_rejects_active_run_with_terminal_evidence() {
        let mut row = evaluation_run_row();
        row.status = "running".to_string();

        let error = EvaluationRun::try_from(row).expect_err("active run cannot have score");

        assert_eq!(
            error,
            EvaluationError::ActiveRunHasTerminalEvidence {
                status: EvaluationRunStatus::Running
            }
        );
    }

    #[test]
    fn db_row_conversion_rejects_invalid_evaluation_score() {
        let mut row = evaluation_run_row();
        row.score_basis_points = Some(10_001);

        let error = EvaluationRun::try_from(row).expect_err("invalid score must fail");

        assert_eq!(
            error,
            EvaluationError::InvalidEvaluationScoreBasisPoints { value: 10_001 }
        );
    }

    #[test]
    fn db_row_conversion_rejects_non_object_report() {
        let mut row = evaluation_run_row();
        row.report = json!("not a report object");

        let error = EvaluationRun::try_from(row).expect_err("report must be object");

        assert_eq!(error, EvaluationError::ReportMustBeObject);
    }

    #[test]
    fn db_row_conversion_rejects_completed_before_created() {
        let mut row = evaluation_run_row();
        row.completed_at = Some(now() - Duration::seconds(1));

        let error = EvaluationRun::try_from(row).expect_err("time order must be valid");

        assert_eq!(error, EvaluationError::CompletedBeforeCreated);
    }
}
