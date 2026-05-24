//! Typed SLI and SLO evaluation.
//!
//! SQL can measure reliability from durable rows, but the application should not
//! treat those raw counts as domain truth until the window, target, and event
//! counts have been validated.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domain::{DomainError, JobKind};

const BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SloError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} cannot be negative, got {value}")]
    NegativeNumber { field: &'static str, value: i64 },
    #[error("{field} is too large, got {value}")]
    NumberOutOfRange { field: &'static str, value: i64 },
    #[error("SLO target must be between 0 and 10000 basis points, got {value}")]
    InvalidTargetBasisPoints { value: i64 },
    #[error("SLO window must end after it starts")]
    InvalidWindow,
    #[error("good events cannot exceed total events: good={good:?}, total={total:?}")]
    GoodEventsExceedTotal {
        good: ObservedGoodEventCount,
        total: ObservedTotalEventCount,
    },
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_slo_text(value: impl Into<String>, field: &'static str) -> Result<String, SloError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(SloError::EmptyText { field });
    }
    Ok(value)
}

fn non_negative_u64(value: i64, field: &'static str) -> Result<u64, SloError> {
    if value < 0 {
        return Err(SloError::NegativeNumber { field, value });
    }

    u64::try_from(value).map_err(|_| SloError::NumberOutOfRange { field, value })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SloName(String);

impl SloName {
    pub fn new(value: impl Into<String>) -> Result<Self, SloError> {
        Ok(Self(non_empty_slo_text(value, "slo_name")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SliName(String);

impl SliName {
    pub fn new(value: impl Into<String>) -> Result<Self, SloError> {
        Ok(Self(non_empty_slo_text(value, "sli_name")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SloWindow {
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
}

impl SloWindow {
    pub fn new(started_at: DateTime<Utc>, ended_at: DateTime<Utc>) -> Result<Self, SloError> {
        if ended_at <= started_at {
            return Err(SloError::InvalidWindow);
        }

        Ok(Self {
            started_at,
            ended_at,
        })
    }

    pub fn started_at(self) -> DateTime<Utc> {
        self.started_at
    }

    pub fn ended_at(self) -> DateTime<Utc> {
        self.ended_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SloTargetBasisPoints(u16);

impl SloTargetBasisPoints {
    pub fn new(value: u16) -> Result<Self, SloError> {
        if value > BASIS_POINT_DENOMINATOR as u16 {
            return Err(SloError::InvalidTargetBasisPoints {
                value: i64::from(value),
            });
        }

        Ok(Self(value))
    }

    pub fn try_from_i64(value: i64) -> Result<Self, SloError> {
        if !(0..=BASIS_POINT_DENOMINATOR as i64).contains(&value) {
            return Err(SloError::InvalidTargetBasisPoints { value });
        }

        let value = u16::try_from(value).map_err(|_| SloError::NumberOutOfRange {
            field: "target_basis_points",
            value,
        })?;

        Self::new(value)
    }

    pub fn basis_points(self) -> u16 {
        self.0
    }

    pub fn allowed_error_basis_points(self) -> u16 {
        BASIS_POINT_DENOMINATOR as u16 - self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObservedGoodEventCount(u64);

impl ObservedGoodEventCount {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn try_from_i64(value: i64) -> Result<Self, SloError> {
        Ok(Self(non_negative_u64(value, "good_events")?))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObservedTotalEventCount(u64);

impl ObservedTotalEventCount {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn try_from_i64(value: i64) -> Result<Self, SloError> {
        Ok(Self(non_negative_u64(value, "total_events")?))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObservedBadEventCount(u64);

impl ObservedBadEventCount {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ErrorBudgetEventCount(u64);

impl ErrorBudgetEventCount {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RemainingErrorBudgetEventCount(u64);

impl RemainingErrorBudgetEventCount {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SliAttainmentBasisPoints(u16);

impl SliAttainmentBasisPoints {
    pub fn basis_points(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SloDecision {
    NoTraffic,
    WithinBudget,
    BudgetExhausted,
}

impl SloDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoTraffic => "no_traffic",
            Self::WithinBudget => "within_budget",
            Self::BudgetExhausted => "budget_exhausted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SloMeasurement {
    pub slo_name: SloName,
    pub sli_name: SliName,
    pub job_kind: Option<JobKind>,
    pub window: SloWindow,
    pub target: SloTargetBasisPoints,
    pub good_events: ObservedGoodEventCount,
    pub total_events: ObservedTotalEventCount,
}

impl SloMeasurement {
    pub fn new(
        slo_name: SloName,
        sli_name: SliName,
        job_kind: Option<JobKind>,
        window: SloWindow,
        target: SloTargetBasisPoints,
        good_events: ObservedGoodEventCount,
        total_events: ObservedTotalEventCount,
    ) -> Result<Self, SloError> {
        if good_events.get() > total_events.get() {
            return Err(SloError::GoodEventsExceedTotal {
                good: good_events,
                total: total_events,
            });
        }

        Ok(Self {
            slo_name,
            sli_name,
            job_kind,
            window,
            target,
            good_events,
            total_events,
        })
    }

    pub fn evaluate(&self) -> SloEvaluation {
        let total = self.total_events.get();
        let good = self.good_events.get();
        let bad = total.saturating_sub(good);
        let allowed_bad = total.saturating_mul(u64::from(self.target.allowed_error_basis_points()))
            / BASIS_POINT_DENOMINATOR;
        let remaining = allowed_bad.saturating_sub(bad);
        let attainment = good
            .saturating_mul(BASIS_POINT_DENOMINATOR)
            .checked_div(total)
            .unwrap_or(0);
        let attainment = attainment.min(BASIS_POINT_DENOMINATOR) as u16;

        let decision = if total == 0 {
            SloDecision::NoTraffic
        } else if bad <= allowed_bad {
            SloDecision::WithinBudget
        } else {
            SloDecision::BudgetExhausted
        };

        SloEvaluation {
            decision,
            bad_events: ObservedBadEventCount(bad),
            allowed_bad_events: ErrorBudgetEventCount(allowed_bad),
            remaining_error_budget_events: RemainingErrorBudgetEventCount(remaining),
            attainment: SliAttainmentBasisPoints(attainment),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SloEvaluation {
    pub decision: SloDecision,
    pub bad_events: ObservedBadEventCount,
    pub allowed_bad_events: ErrorBudgetEventCount,
    pub remaining_error_budget_events: RemainingErrorBudgetEventCount,
    pub attainment: SliAttainmentBasisPoints,
}

// ANCHOR: slo_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbSloMeasurementRow {
    pub slo_name: String,
    pub sli_name: String,
    pub job_kind: Option<String>,
    pub window_started_at: DateTime<Utc>,
    pub window_ended_at: DateTime<Utc>,
    pub target_basis_points: i64,
    pub good_events: i64,
    pub total_events: i64,
}

impl TryFrom<DbSloMeasurementRow> for SloMeasurement {
    type Error = SloError;

    fn try_from(row: DbSloMeasurementRow) -> Result<Self, Self::Error> {
        let job_kind = row.job_kind.map(JobKind::new).transpose()?;

        Self::new(
            SloName::new(row.slo_name)?,
            SliName::new(row.sli_name)?,
            job_kind,
            SloWindow::new(row.window_started_at, row.window_ended_at)?,
            SloTargetBasisPoints::try_from_i64(row.target_basis_points)?,
            ObservedGoodEventCount::try_from_i64(row.good_events)?,
            ObservedTotalEventCount::try_from_i64(row.total_events)?,
        )
    }
}
// ANCHOR_END: slo_row_boundary

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn window() -> SloWindow {
        SloWindow::new(
            Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0)
                .single()
                .expect("start"),
            Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0)
                .single()
                .expect("end"),
        )
        .expect("valid window")
    }

    fn measurement(good: u64, total: u64) -> SloMeasurement {
        SloMeasurement::new(
            SloName::new("job-start-latency:v1").expect("slo name"),
            SliName::new("job_start_latency_within_120s").expect("sli name"),
            Some(JobKind::new("incident_triage").expect("job kind")),
            window(),
            SloTargetBasisPoints::new(9_900).expect("target"),
            ObservedGoodEventCount::new(good),
            ObservedTotalEventCount::new(total),
        )
        .expect("valid measurement")
    }

    #[test]
    fn evaluates_measurement_with_remaining_budget() {
        let evaluation = measurement(991, 1_000).evaluate();

        assert_eq!(evaluation.decision, SloDecision::WithinBudget);
        assert_eq!(evaluation.bad_events.get(), 9);
        assert_eq!(evaluation.allowed_bad_events.get(), 10);
        assert_eq!(evaluation.remaining_error_budget_events.get(), 1);
        assert_eq!(evaluation.attainment.basis_points(), 9_910);
    }

    #[test]
    fn exhausts_budget_when_bad_events_exceed_allowed_budget() {
        let evaluation = measurement(989, 1_000).evaluate();

        assert_eq!(evaluation.decision, SloDecision::BudgetExhausted);
        assert_eq!(evaluation.bad_events.get(), 11);
        assert_eq!(evaluation.allowed_bad_events.get(), 10);
        assert_eq!(evaluation.remaining_error_budget_events.get(), 0);
    }

    #[test]
    fn no_traffic_is_a_distinct_operational_decision() {
        let evaluation = measurement(0, 0).evaluate();

        assert_eq!(evaluation.decision, SloDecision::NoTraffic);
        assert_eq!(evaluation.bad_events.get(), 0);
        assert_eq!(evaluation.allowed_bad_events.get(), 0);
    }

    #[test]
    fn rejects_good_events_greater_than_total_events() {
        let error = SloMeasurement::new(
            SloName::new("job-start-latency:v1").expect("slo name"),
            SliName::new("job_start_latency_within_120s").expect("sli name"),
            None,
            window(),
            SloTargetBasisPoints::new(9_900).expect("target"),
            ObservedGoodEventCount::new(11),
            ObservedTotalEventCount::new(10),
        )
        .expect_err("good events cannot exceed total events");

        assert!(matches!(error, SloError::GoodEventsExceedTotal { .. }));
    }

    #[test]
    fn rejects_invalid_raw_database_row_values() {
        let row = DbSloMeasurementRow {
            slo_name: "job-start-latency:v1".to_string(),
            sli_name: "job_start_latency_within_120s".to_string(),
            job_kind: Some("incident_triage".to_string()),
            window_started_at: window().started_at(),
            window_ended_at: window().ended_at(),
            target_basis_points: 10_001,
            good_events: 1,
            total_events: 1,
        };

        let error = SloMeasurement::try_from(row).expect_err("invalid target");

        assert_eq!(error, SloError::InvalidTargetBasisPoints { value: 10_001 });
    }

    #[test]
    fn rejects_invalid_windows_from_database_rows() {
        let row = DbSloMeasurementRow {
            slo_name: "job-start-latency:v1".to_string(),
            sli_name: "job_start_latency_within_120s".to_string(),
            job_kind: None,
            window_started_at: window().ended_at(),
            window_ended_at: window().started_at(),
            target_basis_points: 9_900,
            good_events: 1,
            total_events: 1,
        };

        let error = SloMeasurement::try_from(row).expect_err("invalid window");

        assert_eq!(error, SloError::InvalidWindow);
    }
}
