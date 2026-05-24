use std::marker::PhantomData;

use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;

use crate::domain::{DomainError, EventMessage, JobKind};

fn non_empty_tool_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, ToolContractError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(ToolContractError::EmptyText { field });
    }

    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawModelOutput(String);

impl RawModelOutput {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolContractError> {
        Ok(Self(non_empty_tool_text(value, "raw_model_output")?))
    }

    // ANCHOR: model_output_pipeline
    pub fn parse_tool_request(self) -> Result<ParsedToolRequest, ToolContractError> {
        let dto = serde_json::from_str::<RawToolRequestDto>(&self.0)?;
        Ok(ParsedToolRequest { dto })
    }
    // ANCHOR_END: model_output_pipeline
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolName(String);

impl ToolName {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolContractError> {
        Ok(Self(non_empty_tool_text(value, "tool_name")?))
    }

    pub fn pause_job_kind() -> Self {
        Self("pause_job_kind".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownToolName(String);

impl UnknownToolName {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownToolName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseFailureMessage(String);

impl ParseFailureMessage {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for ParseFailureMessage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolReason(String);

impl ToolReason {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolContractError> {
        Ok(Self(non_empty_tool_text(value, "tool_reason")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ToolReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolInput<T>(T);

impl<T> ToolInput<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutput<T>(T);

impl<T> ToolOutput<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

// ANCHOR: typed_tool_trait
#[async_trait]
pub trait TypedTool: Send + Sync {
    type Input: Send + Sync + 'static;
    type Output: Send + Sync + 'static;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn call(
        &self,
        input: ToolInput<Self::Input>,
    ) -> Result<ToolOutput<Self::Output>, Self::Error>;
}
// ANCHOR_END: typed_tool_trait

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedToolRequest {
    dto: RawToolRequestDto,
}

impl ParsedToolRequest {
    pub fn validate_pause_job_kind(
        self,
    ) -> Result<ValidatedToolRequest<PauseJobKindRequest>, ToolContractError> {
        let tool_name = ToolName::new(self.dto.tool_name)?;
        if tool_name != ToolName::pause_job_kind() {
            return Err(ToolContractError::UnknownTool {
                tool_name: UnknownToolName::new(tool_name.as_str()),
            });
        }

        Ok(ValidatedToolRequest {
            tool_name,
            input: PauseJobKindRequest {
                job_kind: JobKind::new(self.dto.input.job_kind)?,
                reason: ToolReason::new(self.dto.input.reason)?,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedToolRequest<T> {
    tool_name: ToolName,
    input: T,
}

impl<T> ValidatedToolRequest<T> {
    pub fn check_policy(self, decision: ToolPolicyDecision) -> PolicyCheckedToolRequest<T> {
        PolicyCheckedToolRequest {
            tool_name: self.tool_name,
            input: self.input,
            decision,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolPolicyDecision {
    AutoApproved,
    ApprovalRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HumanApprovalDecision {
    Approved,
    Rejected { reason: ToolReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyCheckedToolRequest<T> {
    tool_name: ToolName,
    input: T,
    decision: ToolPolicyDecision,
}

impl<T> PolicyCheckedToolRequest<T> {
    pub fn record_approval(
        self,
        decision: HumanApprovalDecision,
    ) -> Result<ApprovedToolRequest<T>, ToolContractError> {
        match (self.decision, decision) {
            (_, HumanApprovalDecision::Rejected { reason }) => {
                Err(ToolContractError::ApprovalRejected { reason })
            }
            (ToolPolicyDecision::AutoApproved, HumanApprovalDecision::Approved)
            | (ToolPolicyDecision::ApprovalRequired, HumanApprovalDecision::Approved) => {
                Ok(ApprovedToolRequest {
                    tool_name: self.tool_name,
                    input: self.input,
                    state: PhantomData,
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approved;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedToolRequest<T> {
    tool_name: ToolName,
    input: T,
    state: PhantomData<Approved>,
}

impl<T> ApprovedToolRequest<T> {
    pub fn tool_name(&self) -> &ToolName {
        &self.tool_name
    }

    pub fn into_tool_input(self) -> ToolInput<T> {
        ToolInput::new(self.input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauseJobKindRequest {
    pub job_kind: JobKind,
    pub reason: ToolReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PauseJobKindResult {
    pub event: EventMessage,
}

// ANCHOR: approved_tool_pipeline
pub fn approved_pause_job_kind_request(
    raw_output: RawModelOutput,
    approval: HumanApprovalDecision,
) -> Result<ApprovedToolRequest<PauseJobKindRequest>, ToolContractError> {
    raw_output
        .parse_tool_request()?
        .validate_pause_job_kind()?
        .check_policy(ToolPolicyDecision::ApprovalRequired)
        .record_approval(approval)
}
// ANCHOR_END: approved_tool_pipeline

#[derive(Debug, Clone, Default)]
pub struct DryRunPauseJobKindTool;

// ANCHOR: dry_run_tool
#[async_trait]
impl TypedTool for DryRunPauseJobKindTool {
    type Input = PauseJobKindRequest;
    type Output = PauseJobKindResult;
    type Error = ToolContractError;

    async fn call(
        &self,
        input: ToolInput<Self::Input>,
    ) -> Result<ToolOutput<Self::Output>, Self::Error> {
        let request = input.into_inner();
        let event = EventMessage::new(format!(
            "would pause job kind {} because {}",
            request.job_kind.as_str(),
            request.reason.as_str()
        ))?;

        Ok(ToolOutput::new(PauseJobKindResult { event }))
    }
}
// ANCHOR_END: dry_run_tool

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ToolContractError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("failed to parse model tool request: {0}")]
    Parse(ParseFailureMessage),
    #[error("unknown tool requested by model: {tool_name}")]
    UnknownTool { tool_name: UnknownToolName },
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
    #[error("human approval rejected the tool request: {reason}")]
    ApprovalRejected { reason: ToolReason },
}

impl From<serde_json::Error> for ToolContractError {
    fn from(error: serde_json::Error) -> Self {
        Self::Parse(ParseFailureMessage::new(error.to_string()))
    }
}

// ANCHOR: tool_request_dto
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawToolRequestDto {
    tool_name: String,
    input: PauseJobKindInputDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct PauseJobKindInputDto {
    job_kind: String,
    reason: String,
}
// ANCHOR_END: tool_request_dto

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_pause_request() -> RawModelOutput {
        RawModelOutput::new(
            r#"{
              "tool_name": "pause_job_kind",
              "input": {
                "job_kind": "incident_triage",
                "reason": "provider quota exhausted"
              }
            }"#,
        )
        .expect("test fixture should be a non-empty model output")
    }

    #[tokio::test]
    async fn model_output_pipeline_validates_policy_and_executes_typed_tool() {
        let approved =
            approved_pause_job_kind_request(raw_pause_request(), HumanApprovalDecision::Approved)
                .expect("fixture should be approved");

        assert_eq!(approved.tool_name(), &ToolName::pause_job_kind());

        let output = DryRunPauseJobKindTool
            .call(approved.into_tool_input())
            .await
            .expect("dry-run tool should produce an audit event")
            .into_inner();

        assert_eq!(
            output.event.as_str(),
            "would pause job kind incident_triage because provider quota exhausted"
        );
    }

    #[test]
    fn model_output_parse_rejects_malformed_json() {
        let error = RawModelOutput::new("{not-json")
            .expect("fixture should be non-empty")
            .parse_tool_request()
            .expect_err("malformed JSON must not become a parsed tool request");

        assert!(matches!(error, ToolContractError::Parse(_)));
    }

    #[test]
    fn model_output_validation_rejects_unknown_tool() {
        let raw = RawModelOutput::new(
            r#"{
              "tool_name": "delete_everything",
              "input": {
                "job_kind": "incident_triage",
                "reason": "bad idea"
              }
            }"#,
        )
        .expect("fixture should be non-empty");

        let error = raw
            .parse_tool_request()
            .expect("fixture should parse")
            .validate_pause_job_kind()
            .expect_err("unknown tool must fail validation");

        assert!(matches!(error, ToolContractError::UnknownTool { .. }));
    }

    #[test]
    fn model_output_parse_rejects_unexpected_top_level_tool_fields() {
        let error = RawModelOutput::new(
            r#"{
              "tool_name": "pause_job_kind",
              "input": {
                "job_kind": "incident_triage",
                "reason": "provider quota exhausted"
              },
              "approved": true
            }"#,
        )
        .expect("fixture should be non-empty")
        .parse_tool_request()
        .expect_err("extra model fields must fail closed");

        assert!(matches!(error, ToolContractError::Parse(_)));
    }

    #[test]
    fn model_output_parse_rejects_unexpected_nested_tool_input_fields() {
        let error = RawModelOutput::new(
            r#"{
              "tool_name": "pause_job_kind",
              "input": {
                "job_kind": "incident_triage",
                "reason": "provider quota exhausted",
                "skip_human_approval": true
              }
            }"#,
        )
        .expect("fixture should be non-empty")
        .parse_tool_request()
        .expect_err("extra nested model fields must fail closed");

        assert!(matches!(error, ToolContractError::Parse(_)));
    }

    #[test]
    fn approval_rejection_blocks_tool_input_construction() {
        let error = raw_pause_request()
            .parse_tool_request()
            .expect("fixture should parse")
            .validate_pause_job_kind()
            .expect("fixture should validate")
            .check_policy(ToolPolicyDecision::ApprovalRequired)
            .record_approval(HumanApprovalDecision::Rejected {
                reason: ToolReason::new("operator rejected risky pause")
                    .expect("fixture reason should be valid"),
            })
            .expect_err("rejected approval must block tool execution");

        assert!(matches!(error, ToolContractError::ApprovalRejected { .. }));
    }
}
