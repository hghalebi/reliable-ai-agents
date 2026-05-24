//! Typed sandbox policy for tool execution.
//!
//! A model may propose a tool call, but it must not choose arbitrary network
//! targets, filesystem paths, or secret exposure. The sandbox policy turns
//! those resource requests into typed, auditable allow/deny decisions.

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AgentRunId, DomainError, PolicyVersion};
use crate::tool_contract::{ToolContractError, ToolName};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SandboxError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("egress destination must be a logical name, not a URL: {value}")]
    DestinationMustBeLogicalName { value: String },
    #[error("egress destination cannot contain a wildcard: {value}")]
    WildcardDestination { value: String },
    #[error("sandbox path must be relative to scratch space: {value}")]
    AbsoluteSandboxPath { value: String },
    #[error("sandbox path cannot contain parent traversal: {value}")]
    ParentTraversal { value: String },
    #[error("unknown filesystem access: {value}")]
    UnknownFilesystemAccess { value: UnknownFilesystemAccess },
    #[error("unknown secret access: {value}")]
    UnknownSecretAccess { value: UnknownSecretAccess },
    #[error("unknown sandbox decision: {value}")]
    UnknownSandboxDecision { value: UnknownSandboxDecision },
    #[error("filesystem access {access} requires a sandbox path")]
    MissingSandboxPath { access: FilesystemAccessKind },
    #[error("filesystem access none must not include a path")]
    UnexpectedSandboxPath,
    #[error("denied sandbox decision requires a reason")]
    MissingDenyReason,
    #[error("sandbox policy for {expected} cannot evaluate request for {actual}")]
    ToolMismatch {
        expected: ToolName,
        actual: ToolName,
    },
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
    #[error("tool validation failed: {0}")]
    Tool(#[from] ToolContractError),
}

fn non_empty_sandbox_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, SandboxError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(SandboxError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownFilesystemAccess(String);

impl UnknownFilesystemAccess {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownFilesystemAccess {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownSecretAccess(String);

impl UnknownSecretAccess {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownSecretAccess {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownSandboxDecision(String);

impl UnknownSandboxDecision {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownSandboxDecision {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EgressDestination(String);

impl EgressDestination {
    pub fn new(value: impl Into<String>) -> Result<Self, SandboxError> {
        let value = non_empty_sandbox_text(value, "egress_destination")?;
        if value.contains("://") {
            return Err(SandboxError::DestinationMustBeLogicalName { value });
        }
        if value.contains('*') {
            return Err(SandboxError::WildcardDestination { value });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EgressAllowlist(Vec<EgressDestination>);

impl EgressAllowlist {
    pub fn new(destinations: impl IntoIterator<Item = EgressDestination>) -> Self {
        Self(destinations.into_iter().collect())
    }

    fn contains(&self, destination: &EgressDestination) -> bool {
        self.0.iter().any(|allowed| allowed == destination)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxPath(String);

impl SandboxPath {
    pub fn new(value: impl Into<String>) -> Result<Self, SandboxError> {
        let value = non_empty_sandbox_text(value, "sandbox_path")?;
        if value.starts_with('/') {
            return Err(SandboxError::AbsoluteSandboxPath { value });
        }
        if value.split('/').any(|segment| segment == "..") {
            return Err(SandboxError::ParentTraversal { value });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkSandboxPolicy {
    Disabled,
    Allowlisted(EgressAllowlist),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemSandboxPolicy {
    Disabled,
    ScratchReadOnly,
    ScratchReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretSandboxPolicy {
    NoSecrets,
    RuntimeOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestedNetworkAccess {
    None,
    Destination(EgressDestination),
}

impl RequestedNetworkAccess {
    fn destination(&self) -> Option<&EgressDestination> {
        match self {
            Self::None => None,
            Self::Destination(destination) => Some(destination),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemAccessKind {
    None,
    ReadScratch,
    WriteScratch,
}

impl FilesystemAccessKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ReadScratch => "read_scratch",
            Self::WriteScratch => "write_scratch",
        }
    }
}

impl std::fmt::Display for FilesystemAccessKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for FilesystemAccessKind {
    type Error = SandboxError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::None),
            "read_scratch" => Ok(Self::ReadScratch),
            "write_scratch" => Ok(Self::WriteScratch),
            value => Err(SandboxError::UnknownFilesystemAccess {
                value: UnknownFilesystemAccess::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestedFilesystemAccess {
    None,
    ReadScratch(SandboxPath),
    WriteScratch(SandboxPath),
}

impl RequestedFilesystemAccess {
    pub fn kind(&self) -> FilesystemAccessKind {
        match self {
            Self::None => FilesystemAccessKind::None,
            Self::ReadScratch(_) => FilesystemAccessKind::ReadScratch,
            Self::WriteScratch(_) => FilesystemAccessKind::WriteScratch,
        }
    }

    pub fn path(&self) -> Option<&SandboxPath> {
        match self {
            Self::None => None,
            Self::ReadScratch(path) | Self::WriteScratch(path) => Some(path),
        }
    }

    fn from_parts(kind: FilesystemAccessKind, path: Option<String>) -> Result<Self, SandboxError> {
        match (kind, path) {
            (FilesystemAccessKind::None, None) => Ok(Self::None),
            (FilesystemAccessKind::None, Some(_)) => Err(SandboxError::UnexpectedSandboxPath),
            (FilesystemAccessKind::ReadScratch, Some(path)) => {
                Ok(Self::ReadScratch(SandboxPath::new(path)?))
            }
            (FilesystemAccessKind::WriteScratch, Some(path)) => {
                Ok(Self::WriteScratch(SandboxPath::new(path)?))
            }
            (FilesystemAccessKind::ReadScratch | FilesystemAccessKind::WriteScratch, None) => {
                Err(SandboxError::MissingSandboxPath { access: kind })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretAccessRequest {
    None,
    ToolRuntimeOnly,
    ModelVisible,
}

impl SecretAccessRequest {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ToolRuntimeOnly => "tool_runtime_only",
            Self::ModelVisible => "model_visible",
        }
    }
}

impl TryFrom<&str> for SecretAccessRequest {
    type Error = SandboxError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::None),
            "tool_runtime_only" => Ok(Self::ToolRuntimeOnly),
            "model_visible" => Ok(Self::ModelVisible),
            value => Err(SandboxError::UnknownSecretAccess {
                value: UnknownSecretAccess::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxDecisionKind {
    Allowed,
    Denied,
}

impl SandboxDecisionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
        }
    }
}

impl TryFrom<&str> for SandboxDecisionKind {
    type Error = SandboxError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "allowed" => Ok(Self::Allowed),
            "denied" => Ok(Self::Denied),
            value => Err(SandboxError::UnknownSandboxDecision {
                value: UnknownSandboxDecision::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxDenyReason(String);

impl SandboxDenyReason {
    pub fn new(value: impl Into<String>) -> Result<Self, SandboxError> {
        Ok(Self(non_empty_sandbox_text(value, "sandbox_deny_reason")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSandboxRequest {
    run_id: AgentRunId,
    tool_name: ToolName,
    network: RequestedNetworkAccess,
    filesystem: RequestedFilesystemAccess,
    secret_access: SecretAccessRequest,
}

impl ToolSandboxRequest {
    pub fn new(
        run_id: AgentRunId,
        tool_name: ToolName,
        network: RequestedNetworkAccess,
        filesystem: RequestedFilesystemAccess,
        secret_access: SecretAccessRequest,
    ) -> Self {
        Self {
            run_id,
            tool_name,
            network,
            filesystem,
            secret_access,
        }
    }
}

// ANCHOR: sandbox_policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSandboxPolicy {
    tool_name: ToolName,
    policy_version: PolicyVersion,
    network: NetworkSandboxPolicy,
    filesystem: FilesystemSandboxPolicy,
    secrets: SecretSandboxPolicy,
}

impl ToolSandboxPolicy {
    pub fn new(
        tool_name: ToolName,
        policy_version: PolicyVersion,
        network: NetworkSandboxPolicy,
        filesystem: FilesystemSandboxPolicy,
        secrets: SecretSandboxPolicy,
    ) -> Self {
        Self {
            tool_name,
            policy_version,
            network,
            filesystem,
            secrets,
        }
    }
    // ANCHOR_END: sandbox_policy

    // ANCHOR: sandbox_request_evaluation
    pub fn evaluate(
        &self,
        request: ToolSandboxRequest,
        decided_at: DateTime<Utc>,
    ) -> Result<SandboxDecisionEvent, SandboxError> {
        if request.tool_name != self.tool_name {
            return Err(SandboxError::ToolMismatch {
                expected: self.tool_name.clone(),
                actual: request.tool_name,
            });
        }

        let deny_reason = self.network_violation(&request).or_else(|| {
            self.filesystem_violation(&request)
                .or_else(|| self.secret_violation(&request))
        });

        SandboxDecisionEvent::new(
            request,
            self.policy_version.clone(),
            deny_reason,
            decided_at,
        )
    }
    // ANCHOR_END: sandbox_request_evaluation

    fn network_violation(&self, request: &ToolSandboxRequest) -> Option<&'static str> {
        match (&self.network, request.network.destination()) {
            (_, None) => None,
            (NetworkSandboxPolicy::Disabled, Some(_)) => Some("network access disabled for tool"),
            (NetworkSandboxPolicy::Allowlisted(allowlist), Some(destination)) => {
                if allowlist.contains(destination) {
                    None
                } else {
                    Some("egress destination not allowed for tool")
                }
            }
        }
    }

    fn filesystem_violation(&self, request: &ToolSandboxRequest) -> Option<&'static str> {
        match (self.filesystem, &request.filesystem) {
            (_, RequestedFilesystemAccess::None) => None,
            (FilesystemSandboxPolicy::Disabled, _) => Some("filesystem access disabled for tool"),
            (
                FilesystemSandboxPolicy::ScratchReadOnly,
                RequestedFilesystemAccess::WriteScratch(_),
            ) => Some("scratch filesystem is read-only for tool"),
            (
                FilesystemSandboxPolicy::ScratchReadOnly
                | FilesystemSandboxPolicy::ScratchReadWrite,
                RequestedFilesystemAccess::ReadScratch(_)
                | RequestedFilesystemAccess::WriteScratch(_),
            ) => None,
        }
    }

    fn secret_violation(&self, request: &ToolSandboxRequest) -> Option<&'static str> {
        match (self.secrets, request.secret_access) {
            (_, SecretAccessRequest::None) => None,
            (_, SecretAccessRequest::ModelVisible) => {
                Some("secrets must not be visible to the model")
            }
            (SecretSandboxPolicy::NoSecrets, SecretAccessRequest::ToolRuntimeOnly) => {
                Some("tool is not allowed to resolve runtime secrets")
            }
            (SecretSandboxPolicy::RuntimeOnly, SecretAccessRequest::ToolRuntimeOnly) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxDecisionEvent {
    id: SandboxEventId,
    run_id: AgentRunId,
    tool_name: ToolName,
    network: RequestedNetworkAccess,
    filesystem: RequestedFilesystemAccess,
    secret_access: SecretAccessRequest,
    decision: SandboxDecisionKind,
    reason: Option<SandboxDenyReason>,
    policy_version: PolicyVersion,
    decided_at: DateTime<Utc>,
}

impl SandboxDecisionEvent {
    fn new(
        request: ToolSandboxRequest,
        policy_version: PolicyVersion,
        deny_reason: Option<&'static str>,
        decided_at: DateTime<Utc>,
    ) -> Result<Self, SandboxError> {
        let decision = if deny_reason.is_some() {
            SandboxDecisionKind::Denied
        } else {
            SandboxDecisionKind::Allowed
        };

        let reason = deny_reason.map(SandboxDenyReason::new).transpose()?;

        Ok(Self {
            id: SandboxEventId::new(),
            run_id: request.run_id,
            tool_name: request.tool_name,
            network: request.network,
            filesystem: request.filesystem,
            secret_access: request.secret_access,
            decision,
            reason,
            policy_version,
            decided_at,
        })
    }

    pub fn decision(&self) -> SandboxDecisionKind {
        self.decision
    }

    pub fn reason(&self) -> Option<&SandboxDenyReason> {
        self.reason.as_ref()
    }

    pub fn id(&self) -> SandboxEventId {
        self.id
    }

    pub fn run_id(&self) -> AgentRunId {
        self.run_id
    }

    pub fn tool_name(&self) -> &ToolName {
        &self.tool_name
    }

    pub fn network(&self) -> &RequestedNetworkAccess {
        &self.network
    }

    pub fn filesystem(&self) -> &RequestedFilesystemAccess {
        &self.filesystem
    }

    pub fn secret_access(&self) -> SecretAccessRequest {
        self.secret_access
    }

    pub fn policy_version(&self) -> &PolicyVersion {
        &self.policy_version
    }

    pub fn decided_at(&self) -> DateTime<Utc> {
        self.decided_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SandboxEventId(Uuid);

impl SandboxEventId {
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

impl Default for SandboxEventId {
    fn default() -> Self {
        Self::new()
    }
}

// ANCHOR: sandbox_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbSandboxEventRow {
    pub id: Uuid,
    pub run_id: Uuid,
    pub tool_name: String,
    pub network_destination: Option<String>,
    pub filesystem_access: String,
    pub filesystem_path: Option<String>,
    pub secret_access: String,
    pub decision: String,
    pub reason: Option<String>,
    pub policy_version: String,
    pub decided_at: DateTime<Utc>,
}

impl TryFrom<DbSandboxEventRow> for SandboxDecisionEvent {
    type Error = SandboxError;

    fn try_from(row: DbSandboxEventRow) -> Result<Self, Self::Error> {
        let decision = SandboxDecisionKind::try_from(row.decision.as_str())?;
        let reason = row.reason.map(SandboxDenyReason::new).transpose()?;
        if decision == SandboxDecisionKind::Denied && reason.is_none() {
            return Err(SandboxError::MissingDenyReason);
        }

        let network = match row.network_destination {
            Some(destination) => {
                RequestedNetworkAccess::Destination(EgressDestination::new(destination)?)
            }
            None => RequestedNetworkAccess::None,
        };
        let filesystem = RequestedFilesystemAccess::from_parts(
            FilesystemAccessKind::try_from(row.filesystem_access.as_str())?,
            row.filesystem_path,
        )?;

        Ok(Self {
            id: SandboxEventId::from_uuid(row.id),
            run_id: AgentRunId::from_uuid(row.run_id),
            tool_name: ToolName::new(row.tool_name)?,
            network,
            filesystem,
            secret_access: SecretAccessRequest::try_from(row.secret_access.as_str())?,
            decision,
            reason,
            policy_version: PolicyVersion::new(row.policy_version)?,
            decided_at: row.decided_at,
        })
    }
}
// ANCHOR_END: sandbox_row_boundary

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn policy() -> ToolSandboxPolicy {
        ToolSandboxPolicy::new(
            ToolName::new("fetch_customer_profile").expect("valid tool"),
            PolicyVersion::new("sandbox-policy:v1").expect("valid policy version"),
            NetworkSandboxPolicy::Allowlisted(EgressAllowlist::new([EgressDestination::new(
                "crm_api",
            )
            .expect("valid destination")])),
            FilesystemSandboxPolicy::ScratchReadOnly,
            SecretSandboxPolicy::RuntimeOnly,
        )
    }

    fn request(
        network: RequestedNetworkAccess,
        filesystem: RequestedFilesystemAccess,
        secret_access: SecretAccessRequest,
    ) -> ToolSandboxRequest {
        ToolSandboxRequest::new(
            AgentRunId::new(),
            ToolName::new("fetch_customer_profile").expect("valid tool"),
            network,
            filesystem,
            secret_access,
        )
    }

    fn row(decision: &str) -> DbSandboxEventRow {
        DbSandboxEventRow {
            id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            tool_name: "fetch_customer_profile".to_string(),
            network_destination: Some("crm_api".to_string()),
            filesystem_access: "read_scratch".to_string(),
            filesystem_path: Some("case-42/context.json".to_string()),
            secret_access: "tool_runtime_only".to_string(),
            decision: decision.to_string(),
            reason: None,
            policy_version: "sandbox-policy:v1".to_string(),
            decided_at: Utc::now(),
        }
    }

    #[test]
    fn policy_allows_allowlisted_network_scratch_read_and_runtime_secret() {
        let event = policy()
            .evaluate(
                request(
                    RequestedNetworkAccess::Destination(
                        EgressDestination::new("crm_api").expect("valid destination"),
                    ),
                    RequestedFilesystemAccess::ReadScratch(
                        SandboxPath::new("case-42/context.json").expect("valid path"),
                    ),
                    SecretAccessRequest::ToolRuntimeOnly,
                ),
                Utc::now(),
            )
            .expect("sandbox decision");

        assert_eq!(event.decision(), SandboxDecisionKind::Allowed);
        assert!(event.reason().is_none());
    }

    #[test]
    fn policy_denies_non_allowlisted_network_destination() {
        let event = policy()
            .evaluate(
                request(
                    RequestedNetworkAccess::Destination(
                        EgressDestination::new("metadata_service").expect("valid destination"),
                    ),
                    RequestedFilesystemAccess::None,
                    SecretAccessRequest::None,
                ),
                Utc::now(),
            )
            .expect("sandbox decision");

        assert_eq!(event.decision(), SandboxDecisionKind::Denied);
        assert_eq!(
            event.reason().map(SandboxDenyReason::as_str),
            Some("egress destination not allowed for tool")
        );
    }

    #[test]
    fn policy_denies_write_when_scratch_is_read_only() {
        let event = policy()
            .evaluate(
                request(
                    RequestedNetworkAccess::None,
                    RequestedFilesystemAccess::WriteScratch(
                        SandboxPath::new("case-42/output.json").expect("valid path"),
                    ),
                    SecretAccessRequest::None,
                ),
                Utc::now(),
            )
            .expect("sandbox decision");

        assert_eq!(event.decision(), SandboxDecisionKind::Denied);
        assert_eq!(
            event.reason().map(SandboxDenyReason::as_str),
            Some("scratch filesystem is read-only for tool")
        );
    }

    #[test]
    fn policy_denies_model_visible_secret() {
        let event = policy()
            .evaluate(
                request(
                    RequestedNetworkAccess::None,
                    RequestedFilesystemAccess::None,
                    SecretAccessRequest::ModelVisible,
                ),
                Utc::now(),
            )
            .expect("sandbox decision");

        assert_eq!(event.decision(), SandboxDecisionKind::Denied);
        assert_eq!(
            event.reason().map(SandboxDenyReason::as_str),
            Some("secrets must not be visible to the model")
        );
    }

    #[test]
    fn egress_destination_rejects_url_and_wildcard_values() {
        let url_error = EgressDestination::new("https://crm.example.com").expect_err("no URLs");
        assert!(matches!(
            url_error,
            SandboxError::DestinationMustBeLogicalName { .. }
        ));

        let wildcard_error = EgressDestination::new("crm_*").expect_err("no wildcards");
        assert!(matches!(
            wildcard_error,
            SandboxError::WildcardDestination { .. }
        ));
    }

    #[test]
    fn sandbox_path_rejects_absolute_and_parent_traversal() {
        let absolute_error = SandboxPath::new("/etc/passwd").expect_err("no absolute paths");
        assert!(matches!(
            absolute_error,
            SandboxError::AbsoluteSandboxPath { .. }
        ));

        let traversal_error = SandboxPath::new("../secret").expect_err("no traversal");
        assert!(matches!(
            traversal_error,
            SandboxError::ParentTraversal { .. }
        ));
    }

    #[test]
    fn row_conversion_accepts_allowed_event() {
        let event = SandboxDecisionEvent::try_from(row("allowed")).expect("valid row");

        assert_eq!(event.decision(), SandboxDecisionKind::Allowed);
        assert_eq!(event.secret_access(), SecretAccessRequest::ToolRuntimeOnly);
    }

    #[test]
    fn row_conversion_rejects_denied_event_without_reason() {
        let error = SandboxDecisionEvent::try_from(row("denied")).expect_err("missing reason");

        assert_eq!(error, SandboxError::MissingDenyReason);
    }

    #[test]
    fn row_conversion_rejects_unknown_filesystem_access() {
        let mut row = row("allowed");
        row.filesystem_access = "host_read".to_string();

        let error = SandboxDecisionEvent::try_from(row).expect_err("unknown access");

        assert!(matches!(
            error,
            SandboxError::UnknownFilesystemAccess { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_filesystem_access_without_path() {
        let mut row = row("allowed");
        row.filesystem_path = None;

        let error = SandboxDecisionEvent::try_from(row).expect_err("missing path");

        assert!(matches!(error, SandboxError::MissingSandboxPath { .. }));
    }

    #[test]
    fn row_conversion_rejects_unknown_secret_access() {
        let mut row = row("allowed");
        row.secret_access = "prompt_visible".to_string();

        let error = SandboxDecisionEvent::try_from(row).expect_err("unknown secret access");

        assert!(matches!(error, SandboxError::UnknownSecretAccess { .. }));
    }
}
