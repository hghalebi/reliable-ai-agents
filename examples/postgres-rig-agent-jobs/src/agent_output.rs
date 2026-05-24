//! Typed boundary for model-produced agent results.
//!
//! A provider response starts as raw text. It is allowed to be untrusted at the
//! edge, but it must become parsed, validated domain data before the worker can
//! persist it as an `AgentResult`.

use serde::Deserialize;
use thiserror::Error;

use crate::domain::{AgentResult, AgentSummary, ApprovalRequirement, DomainError, NextAction};

fn non_empty_agent_output_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, AgentOutputError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(AgentOutputError::EmptyText { field });
    }

    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawAgentOutput(String);

impl RawAgentOutput {
    pub fn new(value: impl Into<String>) -> Result<Self, AgentOutputError> {
        Ok(Self(non_empty_agent_output_text(
            value,
            "raw_agent_output",
        )?))
    }

    // ANCHOR: agent_output_pipeline
    pub fn parse(self) -> Result<ParsedAgentOutput, AgentOutputError> {
        let dto = serde_json::from_str::<RawAgentOutputDto>(&self.0)?;
        Ok(ParsedAgentOutput { dto })
    }

    pub fn parse_agent_result(self) -> Result<AgentResult, AgentOutputError> {
        Ok(self.parse()?.validate()?.into_agent_result())
    }
    // ANCHOR_END: agent_output_pipeline
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAgentOutput {
    dto: RawAgentOutputDto,
}

impl ParsedAgentOutput {
    pub fn validate(self) -> Result<ValidatedAgentOutput, AgentOutputError> {
        let approval = match self.dto.approval_requirement.as_str() {
            "required" => ApprovalRequirement::Required,
            "not_required" => ApprovalRequirement::NotRequired,
            value => {
                return Err(AgentOutputError::UnknownApprovalRequirement {
                    value: UnknownAgentApprovalRequirement::new(value),
                });
            }
        };

        Ok(ValidatedAgentOutput {
            result: AgentResult::new(
                AgentSummary::new(self.dto.summary)?,
                NextAction::new(self.dto.next_action)?,
                approval,
            ),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedAgentOutput {
    result: AgentResult,
}

impl ValidatedAgentOutput {
    pub fn into_agent_result(self) -> AgentResult {
        self.result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentOutputParseFailure(String);

impl AgentOutputParseFailure {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for AgentOutputParseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAgentApprovalRequirement(String);

impl UnknownAgentApprovalRequirement {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownAgentApprovalRequirement {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AgentOutputError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("failed to parse agent output: {0}")]
    Parse(AgentOutputParseFailure),
    #[error("unknown approval requirement from model: {value}")]
    UnknownApprovalRequirement {
        value: UnknownAgentApprovalRequirement,
    },
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

impl From<serde_json::Error> for AgentOutputError {
    fn from(error: serde_json::Error) -> Self {
        Self::Parse(AgentOutputParseFailure::new(error.to_string()))
    }
}

// ANCHOR: agent_output_dto
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAgentOutputDto {
    summary: String,
    next_action: String,
    approval_requirement: String,
}
// ANCHOR_END: agent_output_dto

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_raw_output() -> RawAgentOutput {
        RawAgentOutput::new(
            r#"{
              "summary": "Deployment failure was analyzed with rollback evidence.",
              "next_action": "Request operator approval before rollback.",
              "approval_requirement": "required"
            }"#,
        )
        .expect("fixture should be non-empty")
    }

    #[test]
    fn valid_agent_output_becomes_domain_result() {
        let result = valid_raw_output()
            .parse_agent_result()
            .expect("fixture should validate");

        assert_eq!(
            result.summary.as_str(),
            "Deployment failure was analyzed with rollback evidence."
        );
        assert_eq!(
            result.next_action.as_str(),
            "Request operator approval before rollback."
        );
        assert_eq!(result.approval, ApprovalRequirement::Required);
    }

    #[test]
    fn malformed_json_is_not_domain_output() {
        let error = RawAgentOutput::new("{not-json")
            .expect("fixture should be non-empty")
            .parse()
            .expect_err("malformed JSON must stay outside the domain");

        assert!(matches!(error, AgentOutputError::Parse(_)));
    }

    #[test]
    fn unknown_approval_requirement_is_rejected() {
        let error = RawAgentOutput::new(
            r#"{
              "summary": "Deployment failure was analyzed.",
              "next_action": "Continue.",
              "approval_requirement": "maybe"
            }"#,
        )
        .expect("fixture should be non-empty")
        .parse()
        .expect("fixture should parse")
        .validate()
        .expect_err("unknown approval value must fail validation");

        assert!(matches!(
            error,
            AgentOutputError::UnknownApprovalRequirement { .. }
        ));
    }

    #[test]
    fn empty_summary_is_rejected_by_domain_validation() {
        let error = RawAgentOutput::new(
            r#"{
              "summary": " ",
              "next_action": "Continue.",
              "approval_requirement": "not_required"
            }"#,
        )
        .expect("fixture should be non-empty")
        .parse()
        .expect("fixture should parse")
        .validate()
        .expect_err("empty summary must fail validation");

        assert!(matches!(error, AgentOutputError::Domain(_)));
    }

    #[test]
    fn unexpected_fields_are_rejected() {
        let error = RawAgentOutput::new(
            r#"{
              "summary": "Deployment failure was analyzed.",
              "next_action": "Continue.",
              "approval_requirement": "not_required",
              "execute_rollback_now": true
            }"#,
        )
        .expect("fixture should be non-empty")
        .parse()
        .expect_err("extra model fields must fail closed");

        assert!(matches!(error, AgentOutputError::Parse(_)));
    }
}
