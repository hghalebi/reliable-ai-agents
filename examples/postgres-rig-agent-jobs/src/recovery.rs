//! Typed restore-drill and replay-safety primitives.
//!
//! Backup existence is not recovery. Recovery starts paused, inspects durable
//! state, and decides which restored work can resume without duplicating
//! external side effects.

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::{DomainError, IdempotencyKey, JobId, JobKind, JobStatus};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RecoveryError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("{field} cannot be negative, got {value}")]
    NegativeNumber { field: &'static str, value: i64 },
    #[error("restore drill completed before it started")]
    CompletedBeforeStarted,
    #[error("restore drill must include at least one replay decision")]
    EmptyReplayDecisions,
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
}

fn non_empty_recovery_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, RecoveryError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(RecoveryError::EmptyText { field });
    }
    Ok(value)
}

fn non_negative_u64(value: i64, field: &'static str) -> Result<u64, RecoveryError> {
    if value < 0 {
        return Err(RecoveryError::NegativeNumber { field, value });
    }

    u64::try_from(value).map_err(|_| RecoveryError::NegativeNumber { field, value })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RestoreDrillId(Uuid);

impl RestoreDrillId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for RestoreDrillId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreDrillName(String);

impl RestoreDrillName {
    pub fn new(value: impl Into<String>) -> Result<Self, RecoveryError> {
        Ok(Self(non_empty_recovery_text(value, "restore_drill_name")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupSource(String);

impl BackupSource {
    pub fn new(value: impl Into<String>) -> Result<Self, RecoveryError> {
        Ok(Self(non_empty_recovery_text(value, "backup_source")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreTarget(String);

impl RestoreTarget {
    pub fn new(value: impl Into<String>) -> Result<Self, RecoveryError> {
        Ok(Self(non_empty_recovery_text(value, "restore_target")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryPointObjectiveSeconds(u64);

impl RecoveryPointObjectiveSeconds {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn seconds(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryTimeObjectiveSeconds(u64);

impl RecoveryTimeObjectiveSeconds {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn seconds(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffectExpectation {
    NotExpected,
    Expected,
}

impl SideEffectExpectation {
    fn from_bool(value: bool) -> Self {
        if value {
            Self::Expected
        } else {
            Self::NotExpected
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffectEvidence {
    NotExpected,
    ReceiptPresent,
    ReceiptMissing,
}

impl SideEffectEvidence {
    fn from_expectation_and_receipt_count(
        expectation: SideEffectExpectation,
        receipt_count: RestoredReceiptCount,
    ) -> Self {
        match (expectation, receipt_count.get()) {
            (SideEffectExpectation::NotExpected, _) => Self::NotExpected,
            (SideEffectExpectation::Expected, 0) => Self::ReceiptMissing,
            (SideEffectExpectation::Expected, _) => Self::ReceiptPresent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestoredReceiptCount(u64);

impl RestoredReceiptCount {
    pub fn try_from_i64(value: i64) -> Result<Self, RecoveryError> {
        Ok(Self(non_negative_u64(value, "side_effect_receipt_count")?))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCandidate {
    job_id: JobId,
    job_kind: JobKind,
    status: JobStatus,
    idempotency_key: Option<IdempotencyKey>,
    side_effect_evidence: SideEffectEvidence,
}

impl ReplayCandidate {
    pub fn new(
        job_id: JobId,
        job_kind: JobKind,
        status: JobStatus,
        idempotency_key: Option<IdempotencyKey>,
        side_effect_evidence: SideEffectEvidence,
    ) -> Self {
        Self {
            job_id,
            job_kind,
            status,
            idempotency_key,
            side_effect_evidence,
        }
    }

    // ANCHOR: replay_decision
    pub fn decide(&self) -> ReplayDecision {
        match (self.status, self.side_effect_evidence) {
            (JobStatus::Succeeded | JobStatus::Dead | JobStatus::Cancelled, _) => {
                ReplayDecision::DoNotReplayTerminalState
            }
            (_, SideEffectEvidence::ReceiptPresent) => ReplayDecision::ReconcileExistingReceipt,
            (_, SideEffectEvidence::ReceiptMissing) => ReplayDecision::QuarantineMissingReceipt,
            (JobStatus::Pending | JobStatus::Running | JobStatus::Failed, _) => {
                ReplayDecision::ResumeFromDurableState
            }
        }
    }
    // ANCHOR_END: replay_decision

    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    pub fn job_kind(&self) -> &JobKind {
        &self.job_kind
    }

    pub fn status(&self) -> JobStatus {
        self.status
    }

    pub fn idempotency_key(&self) -> Option<&IdempotencyKey> {
        self.idempotency_key.as_ref()
    }

    pub fn side_effect_evidence(&self) -> SideEffectEvidence {
        self.side_effect_evidence
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayDecision {
    ResumeFromDurableState,
    ReconcileExistingReceipt,
    QuarantineMissingReceipt,
    DoNotReplayTerminalState,
}

impl ReplayDecision {
    pub fn is_safe_to_auto_resume(self) -> bool {
        matches!(self, Self::ResumeFromDurableState)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayDecisions(Vec<ReplayDecision>);

impl ReplayDecisions {
    pub fn new(values: impl IntoIterator<Item = ReplayDecision>) -> Result<Self, RecoveryError> {
        let values: Vec<_> = values.into_iter().collect();

        if values.is_empty() {
            return Err(RecoveryError::EmptyReplayDecisions);
        }

        Ok(Self(values))
    }

    pub fn as_slice(&self) -> &[ReplayDecision] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreDrillOutcome {
    Passed,
    Failed,
}

pub struct RestoreDrillInput {
    pub name: RestoreDrillName,
    pub backup_source: BackupSource,
    pub restore_target: RestoreTarget,
    pub rpo: RecoveryPointObjectiveSeconds,
    pub rto: RecoveryTimeObjectiveSeconds,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub replay_decisions: ReplayDecisions,
}

// ANCHOR: restore_drill_report
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreDrillReport {
    id: RestoreDrillId,
    name: RestoreDrillName,
    backup_source: BackupSource,
    restore_target: RestoreTarget,
    rpo: RecoveryPointObjectiveSeconds,
    rto: RecoveryTimeObjectiveSeconds,
    started_at: DateTime<Utc>,
    completed_at: DateTime<Utc>,
    replay_decisions: ReplayDecisions,
}

impl RestoreDrillReport {
    pub fn record(input: RestoreDrillInput) -> Result<Self, RecoveryError> {
        if input.completed_at < input.started_at {
            return Err(RecoveryError::CompletedBeforeStarted);
        }

        Ok(Self {
            id: RestoreDrillId::new(),
            name: input.name,
            backup_source: input.backup_source,
            restore_target: input.restore_target,
            rpo: input.rpo,
            rto: input.rto,
            started_at: input.started_at,
            completed_at: input.completed_at,
            replay_decisions: input.replay_decisions,
        })
    }

    pub fn outcome(&self) -> RestoreDrillOutcome {
        let restore_time_exceeded = self.measured_restore_seconds() > self.rto.seconds();
        let unsafe_replay = self
            .replay_decisions
            .as_slice()
            .contains(&ReplayDecision::QuarantineMissingReceipt);

        if restore_time_exceeded || unsafe_replay {
            RestoreDrillOutcome::Failed
        } else {
            RestoreDrillOutcome::Passed
        }
    }

    pub fn measured_restore_seconds(&self) -> u64 {
        (self.completed_at - self.started_at).num_seconds().max(0) as u64
    }
}
// ANCHOR_END: restore_drill_report

impl RestoreDrillReport {
    pub fn id(&self) -> RestoreDrillId {
        self.id
    }

    pub fn name(&self) -> &RestoreDrillName {
        &self.name
    }

    pub fn backup_source(&self) -> &BackupSource {
        &self.backup_source
    }

    pub fn restore_target(&self) -> &RestoreTarget {
        &self.restore_target
    }

    pub fn rpo(&self) -> RecoveryPointObjectiveSeconds {
        self.rpo
    }

    pub fn replay_decisions(&self) -> &ReplayDecisions {
        &self.replay_decisions
    }
}

// ANCHOR: replay_candidate_row_boundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbRestoreReplayCandidateRow {
    pub job_id: Uuid,
    pub job_kind: String,
    pub job_status: String,
    pub idempotency_key: Option<String>,
    pub side_effect_expected: bool,
    pub side_effect_receipt_count: i64,
}

impl TryFrom<DbRestoreReplayCandidateRow> for ReplayCandidate {
    type Error = RecoveryError;

    fn try_from(row: DbRestoreReplayCandidateRow) -> Result<Self, Self::Error> {
        let receipt_count = RestoredReceiptCount::try_from_i64(row.side_effect_receipt_count)?;
        let expectation = SideEffectExpectation::from_bool(row.side_effect_expected);
        let idempotency_key = row.idempotency_key.map(IdempotencyKey::new).transpose()?;

        Ok(Self::new(
            JobId::from_uuid(row.job_id),
            JobKind::new(row.job_kind)?,
            JobStatus::try_from(row.job_status.as_str())?,
            idempotency_key,
            SideEffectEvidence::from_expectation_and_receipt_count(expectation, receipt_count),
        ))
    }
}
// ANCHOR_END: replay_candidate_row_boundary

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn job_kind() -> JobKind {
        JobKind::new("billing_action").expect("valid job kind")
    }

    fn candidate(status: JobStatus, evidence: SideEffectEvidence) -> ReplayCandidate {
        ReplayCandidate::new(JobId::new(), job_kind(), status, None, evidence)
    }

    fn drill_input(replay_decisions: ReplayDecisions, elapsed_seconds: i64) -> RestoreDrillInput {
        let started_at = Utc::now();
        RestoreDrillInput {
            name: RestoreDrillName::new("monthly restore drill").expect("valid name"),
            backup_source: BackupSource::new("pgbackrest://prod/latest").expect("valid source"),
            restore_target: RestoreTarget::new("isolated-staging").expect("valid target"),
            rpo: RecoveryPointObjectiveSeconds::new(300),
            rto: RecoveryTimeObjectiveSeconds::new(900),
            started_at,
            completed_at: started_at + Duration::seconds(elapsed_seconds),
            replay_decisions,
        }
    }

    fn row() -> DbRestoreReplayCandidateRow {
        DbRestoreReplayCandidateRow {
            job_id: Uuid::new_v4(),
            job_kind: "billing_action".to_string(),
            job_status: "pending".to_string(),
            idempotency_key: Some("billing:tenant-a:adjustment-7".to_string()),
            side_effect_expected: true,
            side_effect_receipt_count: 1,
        }
    }

    #[test]
    fn pending_without_side_effect_can_resume() {
        let decision = candidate(JobStatus::Pending, SideEffectEvidence::NotExpected).decide();

        assert_eq!(decision, ReplayDecision::ResumeFromDurableState);
        assert!(decision.is_safe_to_auto_resume());
    }

    #[test]
    fn missing_receipt_quarantines_replay() {
        let decision = candidate(JobStatus::Pending, SideEffectEvidence::ReceiptMissing).decide();

        assert_eq!(decision, ReplayDecision::QuarantineMissingReceipt);
        assert!(!decision.is_safe_to_auto_resume());
    }

    #[test]
    fn restored_receipt_requires_reconciliation() {
        let decision = candidate(JobStatus::Running, SideEffectEvidence::ReceiptPresent).decide();

        assert_eq!(decision, ReplayDecision::ReconcileExistingReceipt);
        assert!(!decision.is_safe_to_auto_resume());
    }

    #[test]
    fn terminal_job_does_not_replay() {
        let decision = candidate(JobStatus::Succeeded, SideEffectEvidence::NotExpected).decide();

        assert_eq!(decision, ReplayDecision::DoNotReplayTerminalState);
    }

    #[test]
    fn restore_drill_passes_when_replay_is_safe_and_rto_met() {
        let decisions =
            ReplayDecisions::new([ReplayDecision::ResumeFromDurableState]).expect("decisions");
        let report = RestoreDrillReport::record(drill_input(decisions, 120)).expect("report");

        assert_eq!(report.outcome(), RestoreDrillOutcome::Passed);
        assert_eq!(report.measured_restore_seconds(), 120);
    }

    #[test]
    fn restore_drill_fails_when_any_replay_is_quarantined() {
        let decisions =
            ReplayDecisions::new([ReplayDecision::QuarantineMissingReceipt]).expect("decisions");
        let report = RestoreDrillReport::record(drill_input(decisions, 120)).expect("report");

        assert_eq!(report.outcome(), RestoreDrillOutcome::Failed);
    }

    #[test]
    fn restore_drill_fails_when_rto_is_exceeded() {
        let decisions =
            ReplayDecisions::new([ReplayDecision::ResumeFromDurableState]).expect("decisions");
        let report = RestoreDrillReport::record(drill_input(decisions, 1_200)).expect("report");

        assert_eq!(report.outcome(), RestoreDrillOutcome::Failed);
    }

    #[test]
    fn row_conversion_accepts_receipt_backed_candidate() {
        let candidate = ReplayCandidate::try_from(row()).expect("valid row");

        assert_eq!(candidate.status(), JobStatus::Pending);
        assert_eq!(
            candidate.side_effect_evidence(),
            SideEffectEvidence::ReceiptPresent
        );
        assert_eq!(candidate.decide(), ReplayDecision::ReconcileExistingReceipt);
    }

    #[test]
    fn row_conversion_rejects_negative_receipt_count() {
        let mut row = row();
        row.side_effect_receipt_count = -1;

        let error = ReplayCandidate::try_from(row).expect_err("negative count must fail");

        assert_eq!(
            error,
            RecoveryError::NegativeNumber {
                field: "side_effect_receipt_count",
                value: -1,
            }
        );
    }

    #[test]
    fn row_conversion_rejects_unknown_job_status() {
        let mut row = row();
        row.job_status = "restoring".to_string();

        let error = ReplayCandidate::try_from(row).expect_err("unknown status must fail");

        assert!(matches!(error, RecoveryError::Domain(_)));
    }
}
