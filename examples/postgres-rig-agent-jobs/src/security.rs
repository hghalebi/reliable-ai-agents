//! Typed authorization and secret-reference boundaries.
//!
//! Security policy is deterministic application state. The model may propose a
//! tool call, but permission, tenant scope, and secret handling are decided
//! outside the model and recorded as typed evidence.

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AgentRunId, DomainError, PolicyVersion, TenantKey};
use crate::tool_contract::{ToolContractError, ToolName};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SecurityError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("unknown tool permission: {value}")]
    UnknownPermission { value: UnknownToolPermission },
    #[error("unknown authorization decision: {value}")]
    UnknownDecision { value: UnknownAuthorizationDecision },
    #[error("authorization decision {decision} requires a reason")]
    MissingDecisionReason { decision: AuthorizationDecisionKind },
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
    #[error("tool validation failed: {0}")]
    Tool(#[from] ToolContractError),
}

fn non_empty_security_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, SecurityError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(SecurityError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownToolPermission(String);

impl UnknownToolPermission {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownToolPermission {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAuthorizationDecision(String);

impl UnknownAuthorizationDecision {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownAuthorizationDecision {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolPermission {
    PauseJobKind,
    ReadTenantData,
    WriteMemory,
    SendExternalMessage,
}

impl ToolPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PauseJobKind => "pause_job_kind",
            Self::ReadTenantData => "read_tenant_data",
            Self::WriteMemory => "write_memory",
            Self::SendExternalMessage => "send_external_message",
        }
    }

    fn requires_human_approval(self) -> bool {
        matches!(self, Self::PauseJobKind | Self::SendExternalMessage)
    }
}

impl TryFrom<&str> for ToolPermission {
    type Error = SecurityError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pause_job_kind" => Ok(Self::PauseJobKind),
            "read_tenant_data" => Ok(Self::ReadTenantData),
            "write_memory" => Ok(Self::WriteMemory),
            "send_external_message" => Ok(Self::SendExternalMessage),
            value => Err(SecurityError::UnknownPermission {
                value: UnknownToolPermission::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationDecisionKind {
    Authorized,
    RequiresApproval,
    Denied,
}

impl AuthorizationDecisionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Authorized => "authorized",
            Self::RequiresApproval => "requires_approval",
            Self::Denied => "denied",
        }
    }
}

impl std::fmt::Display for AuthorizationDecisionKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for AuthorizationDecisionKind {
    type Error = SecurityError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "authorized" => Ok(Self::Authorized),
            "requires_approval" => Ok(Self::RequiresApproval),
            "denied" => Ok(Self::Denied),
            value => Err(SecurityError::UnknownDecision {
                value: UnknownAuthorizationDecision::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorId(String);

impl ActorId {
    pub fn new(value: impl Into<String>) -> Result<Self, SecurityError> {
        Ok(Self(non_empty_security_text(value, "actor_id")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionReason(String);

impl DecisionReason {
    pub fn new(value: impl Into<String>) -> Result<Self, SecurityError> {
        Ok(Self(non_empty_security_text(value, "decision_reason")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretRef(String);

impl SecretRef {
    pub fn new(value: impl Into<String>) -> Result<Self, SecurityError> {
        Ok(Self(non_empty_security_text(value, "secret_ref")?))
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretRef {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretRef([redacted])")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionGrant {
    actor_id: ActorId,
    tenant_key: TenantKey,
    permission: ToolPermission,
}

impl PermissionGrant {
    pub fn new(actor_id: ActorId, tenant_key: TenantKey, permission: ToolPermission) -> Self {
        Self {
            actor_id,
            tenant_key,
            permission,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PermissionGrants(Vec<PermissionGrant>);

impl PermissionGrants {
    pub fn new(grants: impl IntoIterator<Item = PermissionGrant>) -> Self {
        Self(grants.into_iter().collect())
    }

    fn allows(&self, request: &ToolAuthorizationRequest) -> bool {
        self.0.iter().any(|grant| {
            grant.actor_id == request.actor_id
                && grant.tenant_key == request.actor_tenant_key
                && grant.permission == request.permission
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolAuthorizationRequest {
    actor_id: ActorId,
    actor_tenant_key: TenantKey,
    requested_tenant_key: TenantKey,
    tool_name: ToolName,
    permission: ToolPermission,
    policy_version: PolicyVersion,
}

impl ToolAuthorizationRequest {
    pub fn new(
        actor_id: ActorId,
        actor_tenant_key: TenantKey,
        requested_tenant_key: TenantKey,
        tool_name: ToolName,
        permission: ToolPermission,
        policy_version: PolicyVersion,
    ) -> Self {
        Self {
            actor_id,
            actor_tenant_key,
            requested_tenant_key,
            tool_name,
            permission,
            policy_version,
        }
    }
}

pub struct AuthorizationPolicy {
    grants: PermissionGrants,
}

// ANCHOR: typed_authorization
impl AuthorizationPolicy {
    pub fn new(grants: PermissionGrants) -> Self {
        Self { grants }
    }

    pub fn authorize(
        &self,
        run_id: AgentRunId,
        request: ToolAuthorizationRequest,
        decided_at: DateTime<Utc>,
    ) -> Result<AuthorizationEvent, SecurityError> {
        let same_tenant = request.actor_tenant_key == request.requested_tenant_key;
        let has_grant = self.grants.allows(&request);
        let decision = if !same_tenant || !has_grant {
            AuthorizationDecisionKind::Denied
        } else if request.permission.requires_human_approval() {
            AuthorizationDecisionKind::RequiresApproval
        } else {
            AuthorizationDecisionKind::Authorized
        };

        let reason = match decision {
            AuthorizationDecisionKind::Authorized => None,
            AuthorizationDecisionKind::RequiresApproval => Some(DecisionReason::new(
                "permission granted but tool requires human approval",
            )?),
            AuthorizationDecisionKind::Denied if !same_tenant => {
                Some(DecisionReason::new("cross-tenant request denied")?)
            }
            AuthorizationDecisionKind::Denied => Some(DecisionReason::new(
                "actor lacks tenant-scoped permission for requested tool",
            )?),
        };

        AuthorizationEvent::new(run_id, request, decision, reason, decided_at)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationEvent {
    id: AuthorizationEventId,
    run_id: AgentRunId,
    actor_id: ActorId,
    actor_tenant_key: TenantKey,
    requested_tenant_key: TenantKey,
    tool_name: ToolName,
    permission: ToolPermission,
    decision: AuthorizationDecisionKind,
    reason: Option<DecisionReason>,
    policy_version: PolicyVersion,
    decided_at: DateTime<Utc>,
}
// ANCHOR_END: typed_authorization

impl AuthorizationEvent {
    fn new(
        run_id: AgentRunId,
        request: ToolAuthorizationRequest,
        decision: AuthorizationDecisionKind,
        reason: Option<DecisionReason>,
        decided_at: DateTime<Utc>,
    ) -> Result<Self, SecurityError> {
        if decision != AuthorizationDecisionKind::Authorized && reason.is_none() {
            return Err(SecurityError::MissingDecisionReason { decision });
        }

        Ok(Self {
            id: AuthorizationEventId::new(),
            run_id,
            actor_id: request.actor_id,
            actor_tenant_key: request.actor_tenant_key,
            requested_tenant_key: request.requested_tenant_key,
            tool_name: request.tool_name,
            permission: request.permission,
            decision,
            reason,
            policy_version: request.policy_version,
            decided_at,
        })
    }

    pub fn decision(&self) -> AuthorizationDecisionKind {
        self.decision
    }

    pub fn id(&self) -> AuthorizationEventId {
        self.id
    }

    pub fn run_id(&self) -> AgentRunId {
        self.run_id
    }

    pub fn reason(&self) -> Option<&DecisionReason> {
        self.reason.as_ref()
    }

    pub fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    pub fn actor_tenant_key(&self) -> &TenantKey {
        &self.actor_tenant_key
    }

    pub fn requested_tenant_key(&self) -> &TenantKey {
        &self.requested_tenant_key
    }

    pub fn tool_name(&self) -> &ToolName {
        &self.tool_name
    }

    pub fn permission(&self) -> ToolPermission {
        self.permission
    }

    pub fn policy_version(&self) -> &PolicyVersion {
        &self.policy_version
    }

    pub fn decided_at(&self) -> DateTime<Utc> {
        self.decided_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuthorizationEventId(Uuid);

impl AuthorizationEventId {
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

impl Default for AuthorizationEventId {
    fn default() -> Self {
        Self::new()
    }
}

// ANCHOR: authorization_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbAuthorizationEventRow {
    pub id: Uuid,
    pub run_id: Uuid,
    pub actor_id: String,
    pub actor_tenant_key: String,
    pub requested_tenant_key: String,
    pub tool_name: String,
    pub permission: String,
    pub decision: String,
    pub reason: Option<String>,
    pub policy_version: String,
    pub decided_at: DateTime<Utc>,
}

impl TryFrom<DbAuthorizationEventRow> for AuthorizationEvent {
    type Error = SecurityError;

    fn try_from(row: DbAuthorizationEventRow) -> Result<Self, Self::Error> {
        let decision = AuthorizationDecisionKind::try_from(row.decision.as_str())?;
        let reason = row.reason.map(DecisionReason::new).transpose()?;

        if decision != AuthorizationDecisionKind::Authorized && reason.is_none() {
            return Err(SecurityError::MissingDecisionReason { decision });
        }

        Ok(Self {
            id: AuthorizationEventId::from_uuid(row.id),
            run_id: AgentRunId::from_uuid(row.run_id),
            actor_id: ActorId::new(row.actor_id)?,
            actor_tenant_key: TenantKey::new(row.actor_tenant_key)?,
            requested_tenant_key: TenantKey::new(row.requested_tenant_key)?,
            tool_name: ToolName::new(row.tool_name)?,
            permission: ToolPermission::try_from(row.permission.as_str())?,
            decision,
            reason,
            policy_version: PolicyVersion::new(row.policy_version)?,
            decided_at: row.decided_at,
        })
    }
}
// ANCHOR_END: authorization_row_boundary

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn actor() -> ActorId {
        ActorId::new("operator-7").expect("valid actor")
    }

    fn tenant() -> TenantKey {
        TenantKey::new("tenant-alpha").expect("valid tenant")
    }

    fn policy_version() -> PolicyVersion {
        PolicyVersion::new("security-policy:v1").expect("valid policy version")
    }

    fn read_request() -> ToolAuthorizationRequest {
        ToolAuthorizationRequest::new(
            actor(),
            tenant(),
            tenant(),
            ToolName::new("read_case_file").expect("valid tool name"),
            ToolPermission::ReadTenantData,
            policy_version(),
        )
    }

    fn row(decision: &str) -> DbAuthorizationEventRow {
        DbAuthorizationEventRow {
            id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            actor_id: "operator-7".to_string(),
            actor_tenant_key: "tenant-alpha".to_string(),
            requested_tenant_key: "tenant-alpha".to_string(),
            tool_name: "read_case_file".to_string(),
            permission: "read_tenant_data".to_string(),
            decision: decision.to_string(),
            reason: None,
            policy_version: "security-policy:v1".to_string(),
            decided_at: Utc::now(),
        }
    }

    #[test]
    fn policy_authorizes_granted_same_tenant_read() {
        let grants = PermissionGrants::new([PermissionGrant::new(
            actor(),
            tenant(),
            ToolPermission::ReadTenantData,
        )]);
        let policy = AuthorizationPolicy::new(grants);

        let event = policy
            .authorize(AgentRunId::new(), read_request(), Utc::now())
            .expect("authorization decision");

        assert_eq!(event.decision(), AuthorizationDecisionKind::Authorized);
        assert!(event.reason().is_none());
    }

    #[test]
    fn policy_denies_cross_tenant_request() {
        let grants = PermissionGrants::new([PermissionGrant::new(
            actor(),
            tenant(),
            ToolPermission::ReadTenantData,
        )]);
        let policy = AuthorizationPolicy::new(grants);
        let request = ToolAuthorizationRequest::new(
            actor(),
            tenant(),
            TenantKey::new("tenant-beta").expect("valid tenant"),
            ToolName::new("read_case_file").expect("valid tool name"),
            ToolPermission::ReadTenantData,
            policy_version(),
        );

        let event = policy
            .authorize(AgentRunId::new(), request, Utc::now())
            .expect("authorization decision");

        assert_eq!(event.decision(), AuthorizationDecisionKind::Denied);
        assert!(event.reason().is_some());
    }

    #[test]
    fn policy_requires_approval_for_risky_granted_tool() {
        let grants = PermissionGrants::new([PermissionGrant::new(
            actor(),
            tenant(),
            ToolPermission::PauseJobKind,
        )]);
        let policy = AuthorizationPolicy::new(grants);
        let request = ToolAuthorizationRequest::new(
            actor(),
            tenant(),
            tenant(),
            ToolName::pause_job_kind(),
            ToolPermission::PauseJobKind,
            policy_version(),
        );

        let event = policy
            .authorize(AgentRunId::new(), request, Utc::now())
            .expect("authorization decision");

        assert_eq!(
            event.decision(),
            AuthorizationDecisionKind::RequiresApproval
        );
        assert!(event.reason().is_some());
    }

    #[test]
    fn row_conversion_accepts_authorized_event() {
        let event = AuthorizationEvent::try_from(row("authorized")).expect("valid row");

        assert_eq!(event.decision(), AuthorizationDecisionKind::Authorized);
        assert_eq!(event.permission(), ToolPermission::ReadTenantData);
    }

    #[test]
    fn row_conversion_rejects_unknown_permission() {
        let mut row = row("authorized");
        row.permission = "read_everything".to_string();

        let error = AuthorizationEvent::try_from(row).expect_err("unknown permission must fail");

        assert!(matches!(error, SecurityError::UnknownPermission { .. }));
    }

    #[test]
    fn row_conversion_rejects_denial_without_reason() {
        let error = AuthorizationEvent::try_from(row("denied")).expect_err("denial needs reason");

        assert_eq!(
            error,
            SecurityError::MissingDecisionReason {
                decision: AuthorizationDecisionKind::Denied
            }
        );
    }

    #[test]
    fn secret_ref_debug_redacts_secret_name() {
        let secret_ref = SecretRef::new("prod/deepseek/api-key").expect("valid secret ref");

        assert_eq!(format!("{secret_ref:?}"), "SecretRef([redacted])");
        assert_eq!(secret_ref.name(), "prod/deepseek/api-key");
    }
}
