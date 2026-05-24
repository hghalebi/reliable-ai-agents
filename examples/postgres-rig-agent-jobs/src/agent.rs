use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use thiserror::Error;

use crate::agent_output::AgentOutputError;
use crate::domain::{
    AgentPayload, AgentResult, AgentSummary, ApprovalRequirement, DomainError, FailureMessage,
    NextAction, RetryDisposition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentFailureKind {
    Transient,
    Permanent,
}

impl std::fmt::Display for AgentFailureKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transient => formatter.write_str("transient"),
            Self::Permanent => formatter.write_str("permanent"),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{kind} provider failure: {message}")]
pub struct ProviderFailure {
    kind: AgentFailureKind,
    message: FailureMessage,
}

impl ProviderFailure {
    pub fn transient(message: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self {
            kind: AgentFailureKind::Transient,
            message: FailureMessage::new(message)?,
        })
    }

    pub fn permanent(message: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self {
            kind: AgentFailureKind::Permanent,
            message: FailureMessage::new(message)?,
        })
    }

    pub fn retry_disposition(&self) -> RetryDisposition {
        match self.kind {
            AgentFailureKind::Transient => RetryDisposition::Retryable,
            AgentFailureKind::Permanent => RetryDisposition::Permanent,
        }
    }
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("invalid agent output: {0}")]
    InvalidOutput(#[from] DomainError),
    #[error("invalid model output: {0}")]
    InvalidModelOutput(#[from] AgentOutputError),
    #[error("agent provider failed: {0}")]
    Provider(#[from] ProviderFailure),
}

impl AgentError {
    pub fn retry_disposition(&self) -> RetryDisposition {
        match self {
            Self::InvalidOutput(_) => RetryDisposition::Permanent,
            Self::InvalidModelOutput(_) => RetryDisposition::Permanent,
            Self::Provider(error) => error.retry_disposition(),
        }
    }
}

// ANCHOR: trait
#[async_trait]
pub trait AgentRunner: Send + Sync {
    async fn run_agent(&self, payload: AgentPayload) -> Result<AgentResult, AgentError>;
}
// ANCHOR_END: trait

// ANCHOR: deterministic_runner
#[derive(Debug, Clone, Default)]
pub struct DeterministicAgentRunner;

#[async_trait]
impl AgentRunner for DeterministicAgentRunner {
    async fn run_agent(&self, payload: AgentPayload) -> Result<AgentResult, AgentError> {
        let summary = format!(
            "Analyzed instruction safely: {}",
            payload.instruction.as_str()
        );

        Ok(AgentResult {
            summary: AgentSummary::new(summary)?,
            next_action: NextAction::new("Prepare operator review before external action")?,
            approval: ApprovalRequirement::Required,
        })
    }
}
// ANCHOR_END: deterministic_runner

#[derive(Debug, Clone)]
pub struct FailingThenSuccessfulRunner {
    remaining_failures: Arc<AtomicUsize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimulatedFailureCount(usize);

impl SimulatedFailureCount {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    fn get(self) -> usize {
        self.0
    }
}

impl FailingThenSuccessfulRunner {
    pub fn new(failures_before_success: SimulatedFailureCount) -> Self {
        Self {
            remaining_failures: Arc::new(AtomicUsize::new(failures_before_success.get())),
        }
    }
}

#[async_trait]
impl AgentRunner for FailingThenSuccessfulRunner {
    async fn run_agent(&self, _payload: AgentPayload) -> Result<AgentResult, AgentError> {
        let should_fail = self
            .remaining_failures
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok();

        if should_fail {
            return Err(ProviderFailure::transient("simulated transient failure")?.into());
        }

        Ok(AgentResult::new(
            AgentSummary::new("Recovered after retry")?,
            NextAction::new("Continue with operator review")?,
            ApprovalRequirement::Required,
        ))
    }
}

#[derive(Debug, Clone, Default)]
pub struct PermanentFailureRunner;

#[async_trait]
impl AgentRunner for PermanentFailureRunner {
    async fn run_agent(&self, _payload: AgentPayload) -> Result<AgentResult, AgentError> {
        Err(ProviderFailure::permanent("simulated permanent failure")?.into())
    }
}
