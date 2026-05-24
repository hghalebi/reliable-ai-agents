//! Typed provider usage and budget controls.
//!
//! Provider cost is operational state. This module keeps token counts, latency,
//! provider status, tenant identity, and cost as domain values after raw
//! database rows cross the persistence boundary.

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{AgentRunId, DomainError, JobKind, ModelRoute, TenantKey};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CostAccountingError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} cannot be negative, got {value}")]
    NegativeNumber { field: &'static str, value: i64 },
    #[error("{field} is too large, got {value}")]
    NumberOutOfRange { field: &'static str, value: i64 },
    #[error("token totals do not agree: prompt={prompt}, completion={completion}, total={total}")]
    TokenTotalMismatch {
        prompt: PromptTokenCount,
        completion: CompletionTokenCount,
        total: TotalTokenCount,
    },
    #[error("cost addition overflowed")]
    CostOverflow,
    #[error("unknown provider call status: {value}")]
    UnknownProviderCallStatus { value: UnknownProviderCallStatus },
    #[error("invalid shared domain value: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_cost_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, CostAccountingError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(CostAccountingError::EmptyText { field });
    }
    Ok(value)
}

fn non_negative_u32(value: i64, field: &'static str) -> Result<u32, CostAccountingError> {
    if value < 0 {
        return Err(CostAccountingError::NegativeNumber { field, value });
    }

    u32::try_from(value).map_err(|_| CostAccountingError::NumberOutOfRange { field, value })
}

fn non_negative_u64(value: i64, field: &'static str) -> Result<u64, CostAccountingError> {
    if value < 0 {
        return Err(CostAccountingError::NegativeNumber { field, value });
    }

    u64::try_from(value).map_err(|_| CostAccountingError::NegativeNumber { field, value })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownProviderCallStatus(String);

impl UnknownProviderCallStatus {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownProviderCallStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderCallStatus {
    Succeeded,
    RateLimited,
    Timeout,
    Failed,
}

impl ProviderCallStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::RateLimited => "rate_limited",
            Self::Timeout => "timeout",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for ProviderCallStatus {
    type Error = CostAccountingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "succeeded" => Ok(Self::Succeeded),
            "rate_limited" => Ok(Self::RateLimited),
            "timeout" => Ok(Self::Timeout),
            "failed" => Ok(Self::Failed),
            value => Err(CostAccountingError::UnknownProviderCallStatus {
                value: UnknownProviderCallStatus::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProviderUsageEventId(Uuid);

impl ProviderUsageEventId {
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

impl Default for ProviderUsageEventId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderName(String);

impl ProviderName {
    pub fn new(value: impl Into<String>) -> Result<Self, CostAccountingError> {
        Ok(Self(non_empty_cost_text(value, "provider_name")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromptTokenCount(u32);

impl PromptTokenCount {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn try_from_i64(value: i64) -> Result<Self, CostAccountingError> {
        Ok(Self(non_negative_u32(value, "prompt_tokens")?))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for PromptTokenCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompletionTokenCount(u32);

impl CompletionTokenCount {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn try_from_i64(value: i64) -> Result<Self, CostAccountingError> {
        Ok(Self(non_negative_u32(value, "completion_tokens")?))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for CompletionTokenCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotalTokenCount(u32);

impl TotalTokenCount {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn try_from_i64(value: i64) -> Result<Self, CostAccountingError> {
        Ok(Self(non_negative_u32(value, "total_tokens")?))
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for TotalTokenCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CostMicrosUsd(u64);

impl CostMicrosUsd {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn try_from_i64(value: i64) -> Result<Self, CostAccountingError> {
        Ok(Self(non_negative_u64(value, "cost_micros_usd")?))
    }

    pub fn checked_add(self, other: Self) -> Result<Self, CostAccountingError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(CostAccountingError::CostOverflow)
    }

    pub fn micros(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyMillis(u64);

impl LatencyMillis {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn try_from_i64(value: i64) -> Result<Self, CostAccountingError> {
        Ok(Self(non_negative_u64(value, "latency_ms")?))
    }

    pub fn millis(self) -> u64 {
        self.0
    }
}

pub struct ProviderUsageEventInput {
    pub run_id: AgentRunId,
    pub tenant_key: TenantKey,
    pub job_kind: JobKind,
    pub provider_name: ProviderName,
    pub model_route: ModelRoute,
    pub status: ProviderCallStatus,
    pub prompt_tokens: PromptTokenCount,
    pub completion_tokens: CompletionTokenCount,
    pub total_tokens: TotalTokenCount,
    pub cost: CostMicrosUsd,
    pub latency: LatencyMillis,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderUsageEvent {
    id: ProviderUsageEventId,
    run_id: AgentRunId,
    tenant_key: TenantKey,
    job_kind: JobKind,
    provider_name: ProviderName,
    model_route: ModelRoute,
    status: ProviderCallStatus,
    prompt_tokens: PromptTokenCount,
    completion_tokens: CompletionTokenCount,
    total_tokens: TotalTokenCount,
    cost: CostMicrosUsd,
    latency: LatencyMillis,
    recorded_at: DateTime<Utc>,
}

// ANCHOR: provider_usage_typed
impl ProviderUsageEvent {
    pub fn record(input: ProviderUsageEventInput) -> Result<Self, CostAccountingError> {
        validate_token_total(
            input.prompt_tokens,
            input.completion_tokens,
            input.total_tokens,
        )?;

        Ok(Self {
            id: ProviderUsageEventId::new(),
            run_id: input.run_id,
            tenant_key: input.tenant_key,
            job_kind: input.job_kind,
            provider_name: input.provider_name,
            model_route: input.model_route,
            status: input.status,
            prompt_tokens: input.prompt_tokens,
            completion_tokens: input.completion_tokens,
            total_tokens: input.total_tokens,
            cost: input.cost,
            latency: input.latency,
            recorded_at: input.recorded_at,
        })
    }

    pub fn cost(&self) -> CostMicrosUsd {
        self.cost
    }

    pub fn id(&self) -> ProviderUsageEventId {
        self.id
    }

    pub fn run_id(&self) -> AgentRunId {
        self.run_id
    }

    pub fn tenant_key(&self) -> &TenantKey {
        &self.tenant_key
    }

    pub fn job_kind(&self) -> &JobKind {
        &self.job_kind
    }

    pub fn provider_name(&self) -> &ProviderName {
        &self.provider_name
    }

    pub fn model_route(&self) -> &ModelRoute {
        &self.model_route
    }

    pub fn status(&self) -> ProviderCallStatus {
        self.status
    }

    pub fn prompt_tokens(&self) -> PromptTokenCount {
        self.prompt_tokens
    }

    pub fn completion_tokens(&self) -> CompletionTokenCount {
        self.completion_tokens
    }

    pub fn total_tokens(&self) -> TotalTokenCount {
        self.total_tokens
    }

    pub fn latency(&self) -> LatencyMillis {
        self.latency
    }

    pub fn recorded_at(&self) -> DateTime<Utc> {
        self.recorded_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantBudget {
    spent: CostMicrosUsd,
    limit: CostMicrosUsd,
}

impl TenantBudget {
    pub fn new(spent: CostMicrosUsd, limit: CostMicrosUsd) -> Self {
        Self { spent, limit }
    }

    pub fn decide(
        self,
        proposed_event: &ProviderUsageEvent,
    ) -> Result<BudgetDecision, CostAccountingError> {
        let projected = self.spent.checked_add(proposed_event.cost())?;

        if projected > self.limit {
            Ok(BudgetDecision::Exceeded {
                projected,
                limit: self.limit,
            })
        } else {
            Ok(BudgetDecision::Allowed {
                projected,
                remaining: CostMicrosUsd::new(self.limit.micros() - projected.micros()),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetDecision {
    Allowed {
        projected: CostMicrosUsd,
        remaining: CostMicrosUsd,
    },
    Exceeded {
        projected: CostMicrosUsd,
        limit: CostMicrosUsd,
    },
}
// ANCHOR_END: provider_usage_typed

// ANCHOR: provider_usage_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbProviderUsageEventRow {
    pub id: Uuid,
    pub run_id: Uuid,
    pub tenant_key: String,
    pub job_kind: String,
    pub provider_name: String,
    pub model_route: String,
    pub provider_status: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cost_micros_usd: i64,
    pub latency_ms: i64,
    pub recorded_at: DateTime<Utc>,
}

impl TryFrom<DbProviderUsageEventRow> for ProviderUsageEvent {
    type Error = CostAccountingError;

    fn try_from(row: DbProviderUsageEventRow) -> Result<Self, Self::Error> {
        let event = Self {
            id: ProviderUsageEventId::from_uuid(row.id),
            run_id: AgentRunId::from_uuid(row.run_id),
            tenant_key: TenantKey::new(row.tenant_key)?,
            job_kind: JobKind::new(row.job_kind)?,
            provider_name: ProviderName::new(row.provider_name)?,
            model_route: ModelRoute::new(row.model_route)?,
            status: ProviderCallStatus::try_from(row.provider_status.as_str())?,
            prompt_tokens: PromptTokenCount::try_from_i64(row.prompt_tokens)?,
            completion_tokens: CompletionTokenCount::try_from_i64(row.completion_tokens)?,
            total_tokens: TotalTokenCount::try_from_i64(row.total_tokens)?,
            cost: CostMicrosUsd::try_from_i64(row.cost_micros_usd)?,
            latency: LatencyMillis::try_from_i64(row.latency_ms)?,
            recorded_at: row.recorded_at,
        };

        validate_token_total(
            event.prompt_tokens,
            event.completion_tokens,
            event.total_tokens,
        )?;
        Ok(event)
    }
}
// ANCHOR_END: provider_usage_row_boundary

fn validate_token_total(
    prompt: PromptTokenCount,
    completion: CompletionTokenCount,
    total: TotalTokenCount,
) -> Result<(), CostAccountingError> {
    if prompt.get().saturating_add(completion.get()) != total.get() {
        return Err(CostAccountingError::TokenTotalMismatch {
            prompt,
            completion,
            total,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn usage_input(cost: CostMicrosUsd) -> ProviderUsageEventInput {
        ProviderUsageEventInput {
            run_id: AgentRunId::new(),
            tenant_key: TenantKey::new("tenant-alpha").expect("valid tenant"),
            job_kind: JobKind::new("incident_triage").expect("valid job kind"),
            provider_name: ProviderName::new("deepseek").expect("valid provider"),
            model_route: ModelRoute::new("deepseek-chat").expect("valid route"),
            status: ProviderCallStatus::Succeeded,
            prompt_tokens: PromptTokenCount::new(100),
            completion_tokens: CompletionTokenCount::new(40),
            total_tokens: TotalTokenCount::new(140),
            cost,
            latency: LatencyMillis::new(850),
            recorded_at: Utc::now(),
        }
    }

    fn row() -> DbProviderUsageEventRow {
        DbProviderUsageEventRow {
            id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            tenant_key: "tenant-alpha".to_string(),
            job_kind: "incident_triage".to_string(),
            provider_name: "deepseek".to_string(),
            model_route: "deepseek-chat".to_string(),
            provider_status: "succeeded".to_string(),
            prompt_tokens: 100,
            completion_tokens: 40,
            total_tokens: 140,
            cost_micros_usd: 250,
            latency_ms: 850,
            recorded_at: Utc::now(),
        }
    }

    #[test]
    fn usage_event_rejects_mismatched_token_total() {
        let mut input = usage_input(CostMicrosUsd::new(250));
        input.total_tokens = TotalTokenCount::new(139);

        let error = ProviderUsageEvent::record(input).expect_err("mismatch must fail");

        assert!(matches!(
            error,
            CostAccountingError::TokenTotalMismatch { .. }
        ));
    }

    #[test]
    fn tenant_budget_allows_spend_under_limit() {
        let event = ProviderUsageEvent::record(usage_input(CostMicrosUsd::new(250)))
            .expect("valid usage event");
        let budget = TenantBudget::new(CostMicrosUsd::new(500), CostMicrosUsd::new(1_000));

        let decision = budget.decide(&event).expect("budget decision");

        assert_eq!(
            decision,
            BudgetDecision::Allowed {
                projected: CostMicrosUsd::new(750),
                remaining: CostMicrosUsd::new(250),
            }
        );
    }

    #[test]
    fn tenant_budget_blocks_spend_over_limit() {
        let event = ProviderUsageEvent::record(usage_input(CostMicrosUsd::new(600)))
            .expect("valid usage event");
        let budget = TenantBudget::new(CostMicrosUsd::new(500), CostMicrosUsd::new(1_000));

        let decision = budget.decide(&event).expect("budget decision");

        assert_eq!(
            decision,
            BudgetDecision::Exceeded {
                projected: CostMicrosUsd::new(1_100),
                limit: CostMicrosUsd::new(1_000),
            }
        );
    }

    #[test]
    fn row_conversion_accepts_valid_provider_usage() {
        let event = ProviderUsageEvent::try_from(row()).expect("valid row");

        assert_eq!(event.status(), ProviderCallStatus::Succeeded);
        assert_eq!(event.total_tokens(), TotalTokenCount::new(140));
        assert_eq!(event.cost(), CostMicrosUsd::new(250));
    }

    #[test]
    fn row_conversion_rejects_negative_cost() {
        let mut row = row();
        row.cost_micros_usd = -1;

        let error = ProviderUsageEvent::try_from(row).expect_err("negative cost must fail");

        assert_eq!(
            error,
            CostAccountingError::NegativeNumber {
                field: "cost_micros_usd",
                value: -1,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_status() {
        let mut row = row();
        row.provider_status = "partial".to_string();

        let error = ProviderUsageEvent::try_from(row).expect_err("unknown status must fail");

        assert!(matches!(
            error,
            CostAccountingError::UnknownProviderCallStatus { .. }
        ));
    }

    #[test]
    fn row_conversion_rejects_mismatched_token_total() {
        let mut row = row();
        row.total_tokens = 141;

        let error = ProviderUsageEvent::try_from(row).expect_err("mismatch must fail");

        assert!(matches!(
            error,
            CostAccountingError::TokenTotalMismatch { .. }
        ));
    }
}
