//! Typed admission control for queue pressure and provider quota backpressure.
//!
//! Admission is a production decision, not a hidden `if` statement in the HTTP
//! handler. The API may receive raw JSON, but the system should decide whether
//! to accept, delay, or reject work using typed queue, budget, priority, and
//! provider-pressure evidence.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::cost_accounting::{BudgetDecision, CostMicrosUsd};
use crate::domain::{DomainError, JobId, JobKind, QueueAge, QueueDepth, QueueMetrics, TenantKey};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdmissionControlError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} must be positive, got {value}")]
    NonPositiveNumber { field: &'static str, value: i64 },
    #[error("unknown job priority: {value}")]
    UnknownJobPriority { value: UnknownAdmissionValue },
    #[error("unknown provider quota pressure: {value}")]
    UnknownProviderQuotaPressure { value: UnknownAdmissionValue },
    #[error("admission delay moved outside the supported timestamp range")]
    DelayOverflow,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_admission_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, AdmissionControlError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(AdmissionControlError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAdmissionValue(String);

impl UnknownAdmissionValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl std::fmt::Display for UnknownAdmissionValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AdmissionRequestId(Uuid);

impl AdmissionRequestId {
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

impl Default for AdmissionRequestId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionPolicyName(String);

impl AdmissionPolicyName {
    pub fn new(value: impl Into<String>) -> Result<Self, AdmissionControlError> {
        Ok(Self(non_empty_admission_text(
            value,
            "admission_policy_name",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxPendingDepth(QueueDepth);

impl MaxPendingDepth {
    pub fn new(value: QueueDepth) -> Result<Self, AdmissionControlError> {
        if value.get() == 0 {
            return Err(AdmissionControlError::NonPositiveNumber {
                field: "max_pending_depth",
                value: 0,
            });
        }
        Ok(Self(value))
    }

    fn get(self) -> usize {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxOldestPendingAge(QueueAge);

impl MaxOldestPendingAge {
    pub fn new(value: QueueAge) -> Result<Self, AdmissionControlError> {
        if value.seconds() == 0 {
            return Err(AdmissionControlError::NonPositiveNumber {
                field: "max_oldest_pending_age_seconds",
                value: 0,
            });
        }
        Ok(Self(value))
    }

    fn seconds(self) -> i64 {
        self.0.seconds()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionDelay(Duration);

impl AdmissionDelay {
    pub fn from_seconds(seconds: i64) -> Result<Self, AdmissionControlError> {
        if seconds <= 0 {
            return Err(AdmissionControlError::NonPositiveNumber {
                field: "admission_delay_seconds",
                value: seconds,
            });
        }
        Ok(Self(Duration::seconds(seconds)))
    }

    fn scaled(self, priority: JobPriority) -> Self {
        Self(self.0 * priority.delay_multiplier())
    }

    fn after(self, requested_at: DateTime<Utc>) -> Result<DateTime<Utc>, AdmissionControlError> {
        requested_at
            .checked_add_signed(self.0)
            .ok_or(AdmissionControlError::DelayOverflow)
    }

    pub fn seconds(self) -> i64 {
        self.0.num_seconds()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    Interactive,
    Standard,
    Bulk,
}

impl JobPriority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::Standard => "standard",
            Self::Bulk => "bulk",
        }
    }

    fn delay_multiplier(self) -> i32 {
        match self {
            Self::Interactive => 1,
            Self::Standard => 2,
            Self::Bulk => 4,
        }
    }
}

impl TryFrom<&str> for JobPriority {
    type Error = AdmissionControlError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "interactive" => Ok(Self::Interactive),
            "standard" => Ok(Self::Standard),
            "bulk" => Ok(Self::Bulk),
            value => Err(AdmissionControlError::UnknownJobPriority {
                value: UnknownAdmissionValue::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderQuotaPressure {
    Healthy,
    NearLimit,
    Exhausted,
}

impl ProviderQuotaPressure {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::NearLimit => "near_limit",
            Self::Exhausted => "exhausted",
        }
    }
}

impl TryFrom<&str> for ProviderQuotaPressure {
    type Error = AdmissionControlError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "healthy" => Ok(Self::Healthy),
            "near_limit" => Ok(Self::NearLimit),
            "exhausted" => Ok(Self::Exhausted),
            value => Err(AdmissionControlError::UnknownProviderQuotaPressure {
                value: UnknownAdmissionValue::new(value),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueuePressure {
    Healthy,
    Backlogged,
    Saturated,
}

impl QueuePressure {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Backlogged => "backlogged",
            Self::Saturated => "saturated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionReason {
    WithinOperatingEnvelope,
    QueueBacklogged,
    QueueSaturated,
    ProviderNearLimit,
    ProviderExhausted,
    TenantBudgetExceeded,
}

impl AdmissionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WithinOperatingEnvelope => "within_operating_envelope",
            Self::QueueBacklogged => "queue_backlogged",
            Self::QueueSaturated => "queue_saturated",
            Self::ProviderNearLimit => "provider_near_limit",
            Self::ProviderExhausted => "provider_exhausted",
            Self::TenantBudgetExceeded => "tenant_budget_exceeded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionDecisionKind {
    Accepted,
    Delayed,
    Rejected,
}

impl AdmissionDecisionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Delayed => "delayed",
            Self::Rejected => "rejected",
        }
    }
}

// ANCHOR: admission_decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionDecision {
    Accepted {
        reason: AdmissionReason,
    },
    Delayed {
        reason: AdmissionReason,
        next_run_at: DateTime<Utc>,
    },
    Rejected {
        reason: AdmissionReason,
    },
}

impl AdmissionDecision {
    pub fn kind(self) -> AdmissionDecisionKind {
        match self {
            Self::Accepted { .. } => AdmissionDecisionKind::Accepted,
            Self::Delayed { .. } => AdmissionDecisionKind::Delayed,
            Self::Rejected { .. } => AdmissionDecisionKind::Rejected,
        }
    }

    pub fn reason(self) -> AdmissionReason {
        match self {
            Self::Accepted { reason }
            | Self::Delayed { reason, .. }
            | Self::Rejected { reason } => reason,
        }
    }

    pub fn next_run_at(self) -> Option<DateTime<Utc>> {
        match self {
            Self::Delayed { next_run_at, .. } => Some(next_run_at),
            Self::Accepted { .. } | Self::Rejected { .. } => None,
        }
    }
}
// ANCHOR_END: admission_decision

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetAdmissionState {
    WithinBudget {
        projected: CostMicrosUsd,
        remaining: CostMicrosUsd,
    },
    Exceeded {
        projected: CostMicrosUsd,
        limit: CostMicrosUsd,
    },
}

impl From<BudgetDecision> for BudgetAdmissionState {
    fn from(value: BudgetDecision) -> Self {
        match value {
            BudgetDecision::Allowed {
                projected,
                remaining,
            } => Self::WithinBudget {
                projected,
                remaining,
            },
            BudgetDecision::Exceeded { projected, limit } => Self::Exceeded { projected, limit },
        }
    }
}

impl BudgetAdmissionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WithinBudget { .. } => "within_budget",
            Self::Exceeded { .. } => "exceeded",
        }
    }

    pub fn projected(self) -> CostMicrosUsd {
        match self {
            Self::WithinBudget { projected, .. } | Self::Exceeded { projected, .. } => projected,
        }
    }

    pub fn remaining(self) -> Option<CostMicrosUsd> {
        match self {
            Self::WithinBudget { remaining, .. } => Some(remaining),
            Self::Exceeded { .. } => None,
        }
    }

    pub fn limit(self) -> Option<CostMicrosUsd> {
        match self {
            Self::Exceeded { limit, .. } => Some(limit),
            Self::WithinBudget { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionSubject {
    request_id: AdmissionRequestId,
    tenant_key: TenantKey,
    job_kind: JobKind,
    priority: JobPriority,
}

impl AdmissionSubject {
    pub fn new(
        request_id: AdmissionRequestId,
        tenant_key: TenantKey,
        job_kind: JobKind,
        priority: JobPriority,
    ) -> Self {
        Self {
            request_id,
            tenant_key,
            job_kind,
            priority,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionSignals {
    queue_metrics: QueueMetrics,
    provider_pressure: ProviderQuotaPressure,
    budget: BudgetAdmissionState,
}

impl AdmissionSignals {
    pub fn new(
        queue_metrics: QueueMetrics,
        provider_pressure: ProviderQuotaPressure,
        budget: BudgetAdmissionState,
    ) -> Self {
        Self {
            queue_metrics,
            provider_pressure,
            budget,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionControlInput {
    subject: AdmissionSubject,
    signals: AdmissionSignals,
    requested_at: DateTime<Utc>,
}

impl AdmissionControlInput {
    pub fn new(
        subject: AdmissionSubject,
        signals: AdmissionSignals,
        requested_at: DateTime<Utc>,
    ) -> Self {
        Self {
            subject,
            signals,
            requested_at,
        }
    }
}

// ANCHOR: admission_policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionPolicy {
    name: AdmissionPolicyName,
    max_pending: MaxPendingDepth,
    max_oldest_pending_age: MaxOldestPendingAge,
    delay: AdmissionDelay,
}

impl AdmissionPolicy {
    pub fn new(
        name: AdmissionPolicyName,
        max_pending: MaxPendingDepth,
        max_oldest_pending_age: MaxOldestPendingAge,
        delay: AdmissionDelay,
    ) -> Self {
        Self {
            name,
            max_pending,
            max_oldest_pending_age,
            delay,
        }
    }

    pub fn name(&self) -> &AdmissionPolicyName {
        &self.name
    }
}
// ANCHOR_END: admission_policy

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionDecisionEvent {
    request_id: AdmissionRequestId,
    job_id: Option<JobId>,
    tenant_key: TenantKey,
    job_kind: JobKind,
    priority: JobPriority,
    queue_pressure: QueuePressure,
    provider_pressure: ProviderQuotaPressure,
    budget: BudgetAdmissionState,
    decision: AdmissionDecision,
    decided_at: DateTime<Utc>,
}

impl AdmissionDecisionEvent {
    pub fn with_job_id(mut self, job_id: JobId) -> Self {
        self.job_id = Some(job_id);
        self
    }

    pub fn request_id(&self) -> AdmissionRequestId {
        self.request_id
    }

    pub fn job_id(&self) -> Option<JobId> {
        self.job_id
    }

    pub fn tenant_key(&self) -> &TenantKey {
        &self.tenant_key
    }

    pub fn job_kind(&self) -> &JobKind {
        &self.job_kind
    }

    pub fn priority(&self) -> JobPriority {
        self.priority
    }

    pub fn queue_pressure(&self) -> QueuePressure {
        self.queue_pressure
    }

    pub fn provider_pressure(&self) -> ProviderQuotaPressure {
        self.provider_pressure
    }

    pub fn budget(&self) -> BudgetAdmissionState {
        self.budget
    }

    pub fn decision(&self) -> AdmissionDecision {
        self.decision
    }

    pub fn decided_at(&self) -> DateTime<Utc> {
        self.decided_at
    }
}

// ANCHOR: admission_control_evaluate
impl AdmissionPolicy {
    pub fn evaluate(
        &self,
        input: AdmissionControlInput,
    ) -> Result<AdmissionDecisionEvent, AdmissionControlError> {
        let queue_pressure = self.queue_pressure(&input.signals.queue_metrics);
        let decision = self.decide(
            input.subject.priority,
            queue_pressure,
            input.signals.provider_pressure,
            input.signals.budget,
            input.requested_at,
        )?;

        Ok(AdmissionDecisionEvent {
            request_id: input.subject.request_id,
            job_id: None,
            tenant_key: input.subject.tenant_key,
            job_kind: input.subject.job_kind,
            priority: input.subject.priority,
            queue_pressure,
            provider_pressure: input.signals.provider_pressure,
            budget: input.signals.budget,
            decision,
            decided_at: input.requested_at,
        })
    }

    fn decide(
        &self,
        priority: JobPriority,
        queue_pressure: QueuePressure,
        provider_pressure: ProviderQuotaPressure,
        budget: BudgetAdmissionState,
        requested_at: DateTime<Utc>,
    ) -> Result<AdmissionDecision, AdmissionControlError> {
        if matches!(budget, BudgetAdmissionState::Exceeded { .. }) {
            return Ok(AdmissionDecision::Rejected {
                reason: AdmissionReason::TenantBudgetExceeded,
            });
        }

        match (priority, queue_pressure, provider_pressure) {
            (JobPriority::Bulk, QueuePressure::Saturated, _)
            | (JobPriority::Bulk, _, ProviderQuotaPressure::Exhausted) => {
                Ok(AdmissionDecision::Rejected {
                    reason: dominant_reason(queue_pressure, provider_pressure),
                })
            }
            (_, QueuePressure::Saturated, _) | (_, _, ProviderQuotaPressure::Exhausted) => {
                Ok(AdmissionDecision::Delayed {
                    reason: dominant_reason(queue_pressure, provider_pressure),
                    next_run_at: self.delay.scaled(priority).after(requested_at)?,
                })
            }
            (JobPriority::Bulk, QueuePressure::Backlogged, _)
            | (JobPriority::Bulk, _, ProviderQuotaPressure::NearLimit) => {
                Ok(AdmissionDecision::Delayed {
                    reason: dominant_reason(queue_pressure, provider_pressure),
                    next_run_at: self.delay.scaled(priority).after(requested_at)?,
                })
            }
            (_, QueuePressure::Backlogged, _) | (_, _, ProviderQuotaPressure::NearLimit) => {
                Ok(AdmissionDecision::Delayed {
                    reason: dominant_reason(queue_pressure, provider_pressure),
                    next_run_at: self.delay.after(requested_at)?,
                })
            }
            (_, QueuePressure::Healthy, ProviderQuotaPressure::Healthy) => {
                Ok(AdmissionDecision::Accepted {
                    reason: AdmissionReason::WithinOperatingEnvelope,
                })
            }
        }
    }

    fn queue_pressure(&self, metrics: &QueueMetrics) -> QueuePressure {
        let pending = metrics.pending.get();
        let oldest_pending_age = metrics
            .oldest_pending_age
            .map(QueueAge::seconds)
            .unwrap_or_default();

        if pending >= self.max_pending.get()
            || oldest_pending_age >= self.max_oldest_pending_age.seconds()
        {
            return QueuePressure::Saturated;
        }

        let backlog_depth = self.max_pending.get().div_ceil(2);
        let backlog_age_seconds = (self.max_oldest_pending_age.seconds() / 2).max(1);

        if pending >= backlog_depth || oldest_pending_age >= backlog_age_seconds {
            QueuePressure::Backlogged
        } else {
            QueuePressure::Healthy
        }
    }
}
// ANCHOR_END: admission_control_evaluate

fn dominant_reason(
    queue_pressure: QueuePressure,
    provider_pressure: ProviderQuotaPressure,
) -> AdmissionReason {
    match provider_pressure {
        ProviderQuotaPressure::Exhausted => AdmissionReason::ProviderExhausted,
        ProviderQuotaPressure::NearLimit => AdmissionReason::ProviderNearLimit,
        ProviderQuotaPressure::Healthy => match queue_pressure {
            QueuePressure::Saturated => AdmissionReason::QueueSaturated,
            QueuePressure::Backlogged => AdmissionReason::QueueBacklogged,
            QueuePressure::Healthy => AdmissionReason::WithinOperatingEnvelope,
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 12, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn policy() -> AdmissionPolicy {
        AdmissionPolicy::new(
            AdmissionPolicyName::new("default-admission:v1").expect("valid policy name"),
            MaxPendingDepth::new(QueueDepth::try_from_usize(10).expect("valid depth"))
                .expect("positive depth"),
            MaxOldestPendingAge::new(QueueAge::from_seconds(120).expect("valid age"))
                .expect("positive age"),
            AdmissionDelay::from_seconds(30).expect("positive delay"),
        )
    }

    fn metrics(pending: usize, oldest_pending_age_seconds: Option<i64>) -> QueueMetrics {
        QueueMetrics {
            pending: QueueDepth::try_from_usize(pending).expect("valid pending"),
            oldest_pending_age: oldest_pending_age_seconds
                .map(|seconds| QueueAge::from_seconds(seconds).expect("valid age")),
            ..QueueMetrics::default()
        }
    }

    fn budget() -> BudgetAdmissionState {
        BudgetAdmissionState::WithinBudget {
            projected: CostMicrosUsd::new(700),
            remaining: CostMicrosUsd::new(300),
        }
    }

    fn input(
        priority: JobPriority,
        queue_metrics: QueueMetrics,
        provider_pressure: ProviderQuotaPressure,
        budget: BudgetAdmissionState,
    ) -> AdmissionControlInput {
        AdmissionControlInput::new(
            AdmissionSubject::new(
                AdmissionRequestId::new(),
                TenantKey::new("tenant-alpha").expect("valid tenant"),
                JobKind::new("incident_triage").expect("valid job kind"),
                priority,
            ),
            AdmissionSignals::new(queue_metrics, provider_pressure, budget),
            now(),
        )
    }

    #[test]
    fn healthy_standard_work_is_accepted() {
        let event = policy()
            .evaluate(input(
                JobPriority::Standard,
                metrics(2, Some(10)),
                ProviderQuotaPressure::Healthy,
                budget(),
            ))
            .expect("admission decision");

        assert_eq!(event.queue_pressure(), QueuePressure::Healthy);
        assert_eq!(event.provider_pressure(), ProviderQuotaPressure::Healthy);
        assert_eq!(event.decision().kind(), AdmissionDecisionKind::Accepted);
        assert_eq!(
            event.decision().reason(),
            AdmissionReason::WithinOperatingEnvelope
        );
        assert!(event.decision().next_run_at().is_none());
    }

    #[test]
    fn tenant_budget_exhaustion_rejects_before_enqueue() {
        let event = policy()
            .evaluate(input(
                JobPriority::Interactive,
                metrics(0, None),
                ProviderQuotaPressure::Healthy,
                BudgetAdmissionState::Exceeded {
                    projected: CostMicrosUsd::new(1_100),
                    limit: CostMicrosUsd::new(1_000),
                },
            ))
            .expect("admission decision");

        assert_eq!(event.decision().kind(), AdmissionDecisionKind::Rejected);
        assert_eq!(
            event.decision().reason(),
            AdmissionReason::TenantBudgetExceeded
        );
    }

    #[test]
    fn saturated_queue_delays_standard_work() {
        let event = policy()
            .evaluate(input(
                JobPriority::Standard,
                metrics(10, Some(30)),
                ProviderQuotaPressure::Healthy,
                budget(),
            ))
            .expect("admission decision");

        assert_eq!(event.queue_pressure(), QueuePressure::Saturated);
        assert_eq!(event.decision().kind(), AdmissionDecisionKind::Delayed);
        assert_eq!(event.decision().reason(), AdmissionReason::QueueSaturated);
        assert_eq!(
            event.decision().next_run_at(),
            Some(now() + Duration::seconds(60))
        );
    }

    #[test]
    fn saturated_queue_rejects_bulk_work() {
        let event = policy()
            .evaluate(input(
                JobPriority::Bulk,
                metrics(10, Some(30)),
                ProviderQuotaPressure::Healthy,
                budget(),
            ))
            .expect("admission decision");

        assert_eq!(event.decision().kind(), AdmissionDecisionKind::Rejected);
        assert_eq!(event.decision().reason(), AdmissionReason::QueueSaturated);
    }

    #[test]
    fn provider_near_limit_delays_bulk_work() {
        let event = policy()
            .evaluate(input(
                JobPriority::Bulk,
                metrics(1, None),
                ProviderQuotaPressure::NearLimit,
                budget(),
            ))
            .expect("admission decision");

        assert_eq!(event.provider_pressure(), ProviderQuotaPressure::NearLimit);
        assert_eq!(event.decision().kind(), AdmissionDecisionKind::Delayed);
        assert_eq!(
            event.decision().next_run_at(),
            Some(now() + Duration::seconds(120))
        );
    }

    #[test]
    fn provider_exhaustion_rejects_bulk_work() {
        let event = policy()
            .evaluate(input(
                JobPriority::Bulk,
                metrics(0, None),
                ProviderQuotaPressure::Exhausted,
                budget(),
            ))
            .expect("admission decision");

        assert_eq!(event.decision().kind(), AdmissionDecisionKind::Rejected);
        assert_eq!(
            event.decision().reason(),
            AdmissionReason::ProviderExhausted
        );
    }

    #[test]
    fn oldest_pending_age_can_saturate_queue_without_large_depth() {
        let event = policy()
            .evaluate(input(
                JobPriority::Interactive,
                metrics(1, Some(120)),
                ProviderQuotaPressure::Healthy,
                budget(),
            ))
            .expect("admission decision");

        assert_eq!(event.queue_pressure(), QueuePressure::Saturated);
        assert_eq!(event.decision().kind(), AdmissionDecisionKind::Delayed);
    }

    #[test]
    fn policy_rejects_non_positive_operating_limits() {
        let depth_error =
            MaxPendingDepth::new(QueueDepth::zero()).expect_err("zero depth should fail");
        let age_error = MaxOldestPendingAge::new(QueueAge::from_seconds(0).expect("zero age"))
            .expect_err("zero age should fail");
        let delay_error = AdmissionDelay::from_seconds(0).expect_err("zero delay should fail");

        assert!(matches!(
            depth_error,
            AdmissionControlError::NonPositiveNumber {
                field: "max_pending_depth",
                value: 0
            }
        ));
        assert!(matches!(
            age_error,
            AdmissionControlError::NonPositiveNumber {
                field: "max_oldest_pending_age_seconds",
                value: 0
            }
        ));
        assert!(matches!(
            delay_error,
            AdmissionControlError::NonPositiveNumber {
                field: "admission_delay_seconds",
                value: 0
            }
        ));
    }

    #[test]
    fn minimum_positive_limits_do_not_backpressure_empty_queues() {
        let tiny_policy = AdmissionPolicy::new(
            AdmissionPolicyName::new("tiny-policy:v1").expect("valid policy name"),
            MaxPendingDepth::new(QueueDepth::try_from_usize(1).expect("valid depth"))
                .expect("positive depth"),
            MaxOldestPendingAge::new(QueueAge::from_seconds(1).expect("valid age"))
                .expect("positive age"),
            AdmissionDelay::from_seconds(1).expect("positive delay"),
        );

        let event = tiny_policy
            .evaluate(input(
                JobPriority::Interactive,
                metrics(0, None),
                ProviderQuotaPressure::Healthy,
                budget(),
            ))
            .expect("admission decision");

        assert_eq!(event.queue_pressure(), QueuePressure::Healthy);
        assert_eq!(event.decision().kind(), AdmissionDecisionKind::Accepted);
    }
}
