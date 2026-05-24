//! Typed trust boundary for model-proposed tool execution.
//!
//! A model can propose a tool call, but execution requires independent
//! evidence: parsed tool shape, authorization, sandbox allowance, and approval
//! when the permission is risky.

use thiserror::Error;

use crate::domain::AgentRunId;
use crate::sandbox::{
    SandboxDecisionEvent, SandboxDecisionKind, SandboxDenyReason, SandboxEventId,
};
use crate::security::{
    AuthorizationDecisionKind, AuthorizationEvent, AuthorizationEventId, DecisionReason,
    ToolPermission,
};
use crate::tool_contract::{
    ApprovedToolRequest, HumanApprovalDecision, PauseJobKindRequest, RawModelOutput,
    ToolContractError, ToolInput, ToolName, ToolPolicyDecision, ToolReason,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ToolExecutionGateError {
    #[error("tool contract validation failed: {0}")]
    Tool(#[from] ToolContractError),
    #[error("authorization event {event_id:?} belongs to run {actual:?}, expected {expected:?}")]
    AuthorizationRunMismatch {
        event_id: AuthorizationEventId,
        expected: AgentRunId,
        actual: AgentRunId,
    },
    #[error("sandbox event {event_id:?} belongs to run {actual:?}, expected {expected:?}")]
    SandboxRunMismatch {
        event_id: SandboxEventId,
        expected: AgentRunId,
        actual: AgentRunId,
    },
    #[error("authorization event {event_id:?} is for tool {actual}, expected {expected}")]
    AuthorizationToolMismatch {
        event_id: AuthorizationEventId,
        expected: ToolName,
        actual: ToolName,
    },
    #[error("sandbox event {event_id:?} is for tool {actual}, expected {expected}")]
    SandboxToolMismatch {
        event_id: SandboxEventId,
        expected: ToolName,
        actual: ToolName,
    },
    #[error("authorization event {event_id:?} has permission {actual:?}, expected {expected:?}")]
    AuthorizationPermissionMismatch {
        event_id: AuthorizationEventId,
        expected: ToolPermission,
        actual: ToolPermission,
    },
    #[error("authorization event {event_id:?} denied execution")]
    AuthorizationDenied {
        event_id: AuthorizationEventId,
        reason: Option<DecisionReason>,
    },
    #[error("authorization event {event_id:?} requires human approval")]
    ApprovalMissing { event_id: AuthorizationEventId },
    #[error(
        "approval was rejected even though authorization event {event_id:?} did not require approval"
    )]
    UnexpectedApprovalRejection {
        event_id: AuthorizationEventId,
        reason: ToolReason,
    },
    #[error("sandbox event {event_id:?} denied tool resources")]
    SandboxDenied {
        event_id: SandboxEventId,
        reason: Option<SandboxDenyReason>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolExecutionApproval {
    NotRequired,
    Approved,
    Rejected { reason: ToolReason },
}

impl ToolExecutionApproval {
    fn into_human_decision(
        self,
        policy_decision: ToolPolicyDecision,
        event_id: AuthorizationEventId,
    ) -> Result<HumanApprovalDecision, ToolExecutionGateError> {
        match (policy_decision, self) {
            (ToolPolicyDecision::AutoApproved, Self::NotRequired | Self::Approved) => {
                Ok(HumanApprovalDecision::Approved)
            }
            (ToolPolicyDecision::AutoApproved, Self::Rejected { reason }) => {
                Err(ToolExecutionGateError::UnexpectedApprovalRejection { event_id, reason })
            }
            (ToolPolicyDecision::ApprovalRequired, Self::Approved) => {
                Ok(HumanApprovalDecision::Approved)
            }
            (ToolPolicyDecision::ApprovalRequired, Self::Rejected { reason }) => {
                Ok(HumanApprovalDecision::Rejected { reason })
            }
            (ToolPolicyDecision::ApprovalRequired, Self::NotRequired) => {
                Err(ToolExecutionGateError::ApprovalMissing { event_id })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExecutionGateInput {
    run_id: AgentRunId,
    raw_model_output: RawModelOutput,
    authorization_event: AuthorizationEvent,
    sandbox_event: SandboxDecisionEvent,
    approval: ToolExecutionApproval,
}

impl ToolExecutionGateInput {
    pub fn new(
        run_id: AgentRunId,
        raw_model_output: RawModelOutput,
        authorization_event: AuthorizationEvent,
        sandbox_event: SandboxDecisionEvent,
        approval: ToolExecutionApproval,
    ) -> Self {
        Self {
            run_id,
            raw_model_output,
            authorization_event,
            sandbox_event,
            approval,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedToolExecution<T> {
    request: ApprovedToolRequest<T>,
    authorization_event_id: AuthorizationEventId,
    sandbox_event_id: SandboxEventId,
    approval: ToolExecutionApproval,
}

impl<T> TrustedToolExecution<T> {
    pub fn authorization_event_id(&self) -> AuthorizationEventId {
        self.authorization_event_id
    }

    pub fn sandbox_event_id(&self) -> SandboxEventId {
        self.sandbox_event_id
    }

    pub fn approval(&self) -> &ToolExecutionApproval {
        &self.approval
    }

    pub fn into_tool_input(self) -> ToolInput<T> {
        self.request.into_tool_input()
    }
}

pub struct ToolExecutionGate;

impl ToolExecutionGate {
    // ANCHOR: tool_execution_gate
    pub fn approve_pause_job_kind(
        input: ToolExecutionGateInput,
    ) -> Result<TrustedToolExecution<PauseJobKindRequest>, ToolExecutionGateError> {
        let ToolExecutionGateInput {
            run_id,
            raw_model_output,
            authorization_event,
            sandbox_event,
            approval,
        } = input;

        let validated = raw_model_output
            .parse_tool_request()?
            .validate_pause_job_kind()?;
        let expected_tool = ToolName::pause_job_kind();

        Self::ensure_authorization_matches(
            run_id,
            &authorization_event,
            &expected_tool,
            ToolPermission::PauseJobKind,
        )?;
        Self::ensure_sandbox_matches(run_id, &sandbox_event, &expected_tool)?;

        let authorization_event_id = authorization_event.id();
        let sandbox_event_id = sandbox_event.id();
        let policy_decision = Self::policy_decision(&authorization_event)?;
        Self::ensure_sandbox_allowed(&sandbox_event)?;
        let approval_for_tool = approval
            .clone()
            .into_human_decision(policy_decision, authorization_event_id)?;
        let request = validated
            .check_policy(policy_decision)
            .record_approval(approval_for_tool)?;

        Ok(TrustedToolExecution {
            request,
            authorization_event_id,
            sandbox_event_id,
            approval,
        })
    }
    // ANCHOR_END: tool_execution_gate

    fn ensure_authorization_matches(
        run_id: AgentRunId,
        event: &AuthorizationEvent,
        expected_tool: &ToolName,
        expected_permission: ToolPermission,
    ) -> Result<(), ToolExecutionGateError> {
        if event.run_id() != run_id {
            return Err(ToolExecutionGateError::AuthorizationRunMismatch {
                event_id: event.id(),
                expected: run_id,
                actual: event.run_id(),
            });
        }
        if event.tool_name() != expected_tool {
            return Err(ToolExecutionGateError::AuthorizationToolMismatch {
                event_id: event.id(),
                expected: expected_tool.clone(),
                actual: event.tool_name().clone(),
            });
        }
        if event.permission() != expected_permission {
            return Err(ToolExecutionGateError::AuthorizationPermissionMismatch {
                event_id: event.id(),
                expected: expected_permission,
                actual: event.permission(),
            });
        }

        Ok(())
    }

    fn ensure_sandbox_matches(
        run_id: AgentRunId,
        event: &SandboxDecisionEvent,
        expected_tool: &ToolName,
    ) -> Result<(), ToolExecutionGateError> {
        if event.run_id() != run_id {
            return Err(ToolExecutionGateError::SandboxRunMismatch {
                event_id: event.id(),
                expected: run_id,
                actual: event.run_id(),
            });
        }
        if event.tool_name() != expected_tool {
            return Err(ToolExecutionGateError::SandboxToolMismatch {
                event_id: event.id(),
                expected: expected_tool.clone(),
                actual: event.tool_name().clone(),
            });
        }

        Ok(())
    }

    fn policy_decision(
        event: &AuthorizationEvent,
    ) -> Result<ToolPolicyDecision, ToolExecutionGateError> {
        match event.decision() {
            AuthorizationDecisionKind::Authorized => Ok(ToolPolicyDecision::AutoApproved),
            AuthorizationDecisionKind::RequiresApproval => Ok(ToolPolicyDecision::ApprovalRequired),
            AuthorizationDecisionKind::Denied => Err(ToolExecutionGateError::AuthorizationDenied {
                event_id: event.id(),
                reason: event.reason().cloned(),
            }),
        }
    }

    fn ensure_sandbox_allowed(event: &SandboxDecisionEvent) -> Result<(), ToolExecutionGateError> {
        match event.decision() {
            SandboxDecisionKind::Allowed => Ok(()),
            SandboxDecisionKind::Denied => Err(ToolExecutionGateError::SandboxDenied {
                event_id: event.id(),
                reason: event.reason().cloned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::{PolicyVersion, TenantKey};
    use crate::sandbox::{
        FilesystemSandboxPolicy, NetworkSandboxPolicy, RequestedFilesystemAccess,
        RequestedNetworkAccess, SecretAccessRequest, SecretSandboxPolicy, ToolSandboxPolicy,
        ToolSandboxRequest,
    };
    use crate::security::{
        ActorId, AuthorizationPolicy, PermissionGrant, PermissionGrants, ToolAuthorizationRequest,
    };
    use crate::tool_contract::{DryRunPauseJobKindTool, TypedTool};

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
        .expect("test fixture should be non-empty")
    }

    fn actor() -> ActorId {
        ActorId::new("operator-7").expect("valid actor")
    }

    fn tenant(value: &str) -> TenantKey {
        TenantKey::new(value).expect("valid tenant")
    }

    fn policy_version() -> PolicyVersion {
        PolicyVersion::new("security-policy:v1").expect("valid policy version")
    }

    fn authorization_event(run_id: AgentRunId) -> AuthorizationEvent {
        let tenant_key = tenant("tenant-alpha");
        let grants = PermissionGrants::new([PermissionGrant::new(
            actor(),
            tenant_key.clone(),
            ToolPermission::PauseJobKind,
        )]);
        let policy = AuthorizationPolicy::new(grants);
        let request = ToolAuthorizationRequest::new(
            actor(),
            tenant_key.clone(),
            tenant_key,
            ToolName::pause_job_kind(),
            ToolPermission::PauseJobKind,
            policy_version(),
        );

        policy
            .authorize(run_id, request, Utc::now())
            .expect("authorization event")
    }

    fn denied_authorization_event(run_id: AgentRunId) -> AuthorizationEvent {
        let tenant_key = tenant("tenant-alpha");
        let grants = PermissionGrants::new([PermissionGrant::new(
            actor(),
            tenant_key.clone(),
            ToolPermission::PauseJobKind,
        )]);
        let policy = AuthorizationPolicy::new(grants);
        let request = ToolAuthorizationRequest::new(
            actor(),
            tenant_key,
            tenant("tenant-beta"),
            ToolName::pause_job_kind(),
            ToolPermission::PauseJobKind,
            policy_version(),
        );

        policy
            .authorize(run_id, request, Utc::now())
            .expect("authorization event")
    }

    fn sandbox_event(
        run_id: AgentRunId,
        secret_access: SecretAccessRequest,
    ) -> SandboxDecisionEvent {
        let policy = ToolSandboxPolicy::new(
            ToolName::pause_job_kind(),
            policy_version(),
            NetworkSandboxPolicy::Disabled,
            FilesystemSandboxPolicy::ScratchReadOnly,
            SecretSandboxPolicy::RuntimeOnly,
        );
        let request = ToolSandboxRequest::new(
            run_id,
            ToolName::pause_job_kind(),
            RequestedNetworkAccess::None,
            RequestedFilesystemAccess::None,
            secret_access,
        );

        policy
            .evaluate(request, Utc::now())
            .expect("sandbox decision")
    }

    #[tokio::test]
    async fn gate_approves_only_after_parse_authorization_sandbox_and_approval() {
        let run_id = AgentRunId::new();
        let authorization_event = authorization_event(run_id);
        let sandbox_event = sandbox_event(run_id, SecretAccessRequest::None);
        let trusted = ToolExecutionGate::approve_pause_job_kind(ToolExecutionGateInput::new(
            run_id,
            raw_pause_request(),
            authorization_event.clone(),
            sandbox_event.clone(),
            ToolExecutionApproval::Approved,
        ))
        .expect("trusted tool execution");

        assert_eq!(trusted.authorization_event_id(), authorization_event.id());
        assert_eq!(trusted.sandbox_event_id(), sandbox_event.id());

        let output = DryRunPauseJobKindTool
            .call(trusted.into_tool_input())
            .await
            .expect("dry-run execution")
            .into_inner();

        assert_eq!(
            output.event.as_str(),
            "would pause job kind incident_triage because provider quota exhausted"
        );
    }

    #[test]
    fn gate_blocks_denied_authorization() {
        let run_id = AgentRunId::new();
        let error = ToolExecutionGate::approve_pause_job_kind(ToolExecutionGateInput::new(
            run_id,
            raw_pause_request(),
            denied_authorization_event(run_id),
            sandbox_event(run_id, SecretAccessRequest::None),
            ToolExecutionApproval::Approved,
        ))
        .expect_err("denied authorization must block execution");

        assert!(matches!(
            error,
            ToolExecutionGateError::AuthorizationDenied { .. }
        ));
    }

    #[test]
    fn gate_blocks_sandbox_denial() {
        let run_id = AgentRunId::new();
        let error = ToolExecutionGate::approve_pause_job_kind(ToolExecutionGateInput::new(
            run_id,
            raw_pause_request(),
            authorization_event(run_id),
            sandbox_event(run_id, SecretAccessRequest::ModelVisible),
            ToolExecutionApproval::Approved,
        ))
        .expect_err("sandbox denial must block execution");

        assert!(matches!(
            error,
            ToolExecutionGateError::SandboxDenied { .. }
        ));
    }

    #[test]
    fn gate_blocks_mismatched_authorization_run() {
        let run_id = AgentRunId::new();
        let other_run_id = AgentRunId::new();
        let error = ToolExecutionGate::approve_pause_job_kind(ToolExecutionGateInput::new(
            run_id,
            raw_pause_request(),
            authorization_event(other_run_id),
            sandbox_event(run_id, SecretAccessRequest::None),
            ToolExecutionApproval::Approved,
        ))
        .expect_err("authorization evidence must match the run");

        assert!(matches!(
            error,
            ToolExecutionGateError::AuthorizationRunMismatch { .. }
        ));
    }

    #[test]
    fn gate_requires_human_approval_for_risky_permission() {
        let run_id = AgentRunId::new();
        let error = ToolExecutionGate::approve_pause_job_kind(ToolExecutionGateInput::new(
            run_id,
            raw_pause_request(),
            authorization_event(run_id),
            sandbox_event(run_id, SecretAccessRequest::None),
            ToolExecutionApproval::NotRequired,
        ))
        .expect_err("risky permission must require approval evidence");

        assert!(matches!(
            error,
            ToolExecutionGateError::ApprovalMissing { .. }
        ));
    }

    #[test]
    fn gate_rejects_unknown_model_tool_before_execution() {
        let run_id = AgentRunId::new();
        let raw = RawModelOutput::new(
            r#"{
              "tool_name": "export_everything",
              "input": {
                "job_kind": "incident_triage",
                "reason": "unsafe"
              }
            }"#,
        )
        .expect("fixture should be non-empty");
        let error = ToolExecutionGate::approve_pause_job_kind(ToolExecutionGateInput::new(
            run_id,
            raw,
            authorization_event(run_id),
            sandbox_event(run_id, SecretAccessRequest::None),
            ToolExecutionApproval::Approved,
        ))
        .expect_err("unknown tool must not reach authorization-as-execution");

        assert!(matches!(
            error,
            ToolExecutionGateError::Tool(ToolContractError::UnknownTool { .. })
        ));
    }
}
