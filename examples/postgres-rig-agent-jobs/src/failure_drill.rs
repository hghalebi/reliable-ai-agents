//! Executable failure drills for the book's simulation and chaos-testing path.
//!
//! A failure drill is not random breakage. It is a small experiment with a
//! hypothesis, limited blast radius, explicit injection, rollback action, and
//! durable evidence requirements.

use chrono::{DateTime, Utc};
use std::num::NonZeroU32;
use thiserror::Error;

use crate::agent::{DeterministicAgentRunner, FailingThenSuccessfulRunner, SimulatedFailureCount};
use crate::domain::{
    AgentInstruction, AgentJob, AgentPayload, DomainError, FailureMessage, JobEventType, JobId,
    JobKind, JobStatus, LeaseDuration, MaxAttempts, RetryPolicy, WorkerId,
};
use crate::memory_store::{InMemoryAgentJobStore, InMemoryStoreError};
use crate::worker::{AgentJobStore, Worker, WorkerError, WorkerRunOutcome};

#[derive(Debug, Error)]
pub enum FailureDrillError {
    #[error("domain validation failed: {0}")]
    Domain(#[from] DomainError),
    #[error("store operation failed: {0}")]
    Store(FailureMessage),
    #[error("worker operation failed: {0}")]
    Worker(FailureMessage),
    #[error("job disappeared during drill: {0:?}")]
    MissingJob(JobId),
    #[error("drill evidence requirements cannot be empty")]
    EmptyEvidenceRequirements,
    #[error("missing required event evidence: {0:?}")]
    MissingRequiredEvent(JobEventType),
    #[error("expected final job status {expected:?}, got {actual:?}")]
    UnexpectedFinalStatus {
        expected: JobStatus,
        actual: JobStatus,
    },
    #[error("expected worker outcome {expected:?} was not observed")]
    MissingWorkerOutcome { expected: WorkerOutcomeKind },
    #[error("recovery delay must be longer than the injected lease duration")]
    RecoveryBeforeLeaseExpiry,
}

impl FailureDrillError {
    fn from_store(error: InMemoryStoreError) -> Self {
        Self::Store(FailureMessage::from_error_text(error.to_string()))
    }

    fn from_worker(error: WorkerError) -> Self {
        Self::Worker(FailureMessage::from_error_text(error.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureDrillScenario {
    ProviderTimeoutThenRetry,
    WorkerCrashAfterLease,
}

impl FailureDrillScenario {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProviderTimeoutThenRetry => "provider_timeout_then_retry",
            Self::WorkerCrashAfterLease => "worker_crash_after_lease",
        }
    }
}

fn non_empty_drill_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, FailureDrillError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(DomainError::EmptyText { field }.into());
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureDrillHypothesis(String);

impl FailureDrillHypothesis {
    pub fn new(value: impl Into<String>) -> Result<Self, FailureDrillError> {
        Ok(Self(non_empty_drill_text(
            value,
            "failure_drill_hypothesis",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureDrillBlastRadius(String);

impl FailureDrillBlastRadius {
    pub fn new(value: impl Into<String>) -> Result<Self, FailureDrillError> {
        Ok(Self(non_empty_drill_text(
            value,
            "failure_drill_blast_radius",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureInjection(String);

impl FailureInjection {
    pub fn new(value: impl Into<String>) -> Result<Self, FailureDrillError> {
        Ok(Self(non_empty_drill_text(value, "failure_injection")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackAction(String);

impl RollbackAction {
    pub fn new(value: impl Into<String>) -> Result<Self, FailureDrillError> {
        Ok(Self(non_empty_drill_text(value, "rollback_action")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerOutcomeKind {
    NoDueJob,
    DrainRequested,
    Succeeded,
    RetryScheduled,
    Dead,
}

impl WorkerOutcomeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoDueJob => "no_due_job",
            Self::DrainRequested => "drain_requested",
            Self::Succeeded => "succeeded",
            Self::RetryScheduled => "retry_scheduled",
            Self::Dead => "dead",
        }
    }
}

impl From<WorkerRunOutcome> for WorkerOutcomeKind {
    fn from(value: WorkerRunOutcome) -> Self {
        match value {
            WorkerRunOutcome::NoDueJob => Self::NoDueJob,
            WorkerRunOutcome::DrainRequested => Self::DrainRequested,
            WorkerRunOutcome::Succeeded(_) => Self::Succeeded,
            WorkerRunOutcome::RetryScheduled(_) => Self::RetryScheduled,
            WorkerRunOutcome::Dead(_) => Self::Dead,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureDrillDecision {
    Passed,
    Failed,
}

impl FailureDrillDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceRequirement {
    Event(JobEventType),
    FinalStatus(JobStatus),
    WorkerOutcome(WorkerOutcomeKind),
}

// ANCHOR: failure_drill_plan
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureDrillPlan {
    scenario: FailureDrillScenario,
    hypothesis: FailureDrillHypothesis,
    blast_radius: FailureDrillBlastRadius,
    injection: FailureInjection,
    rollback: RollbackAction,
    evidence: EvidenceRequirements,
}
// ANCHOR_END: failure_drill_plan

impl FailureDrillPlan {
    pub fn new(
        scenario: FailureDrillScenario,
        hypothesis: FailureDrillHypothesis,
        blast_radius: FailureDrillBlastRadius,
        injection: FailureInjection,
        rollback: RollbackAction,
        evidence: EvidenceRequirements,
    ) -> Self {
        Self {
            scenario,
            hypothesis,
            blast_radius,
            injection,
            rollback,
            evidence,
        }
    }

    pub fn scenario(&self) -> FailureDrillScenario {
        self.scenario
    }

    pub fn hypothesis(&self) -> &FailureDrillHypothesis {
        &self.hypothesis
    }

    pub fn blast_radius(&self) -> &FailureDrillBlastRadius {
        &self.blast_radius
    }

    pub fn injection(&self) -> &FailureInjection {
        &self.injection
    }

    pub fn rollback(&self) -> &RollbackAction {
        &self.rollback
    }

    pub fn evidence(&self) -> &EvidenceRequirements {
        &self.evidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRequirements(Vec<EvidenceRequirement>);

impl EvidenceRequirements {
    pub fn new(
        requirements: impl IntoIterator<Item = EvidenceRequirement>,
    ) -> Result<Self, FailureDrillError> {
        let requirements: Vec<_> = requirements.into_iter().collect();
        if requirements.is_empty() {
            return Err(FailureDrillError::EmptyEvidenceRequirements);
        }
        Ok(Self(requirements))
    }

    pub fn iter(&self) -> impl Iterator<Item = &EvidenceRequirement> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedEventTimeline(Vec<JobEventType>);

impl ObservedEventTimeline {
    pub fn new(events: impl IntoIterator<Item = JobEventType>) -> Self {
        Self(events.into_iter().collect())
    }

    pub fn contains(&self, event_type: JobEventType) -> bool {
        self.0.contains(&event_type)
    }

    pub fn iter(&self) -> impl Iterator<Item = &JobEventType> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedWorkerOutcomes(Vec<WorkerOutcomeKind>);

impl ObservedWorkerOutcomes {
    pub fn new(outcomes: impl IntoIterator<Item = WorkerOutcomeKind>) -> Self {
        Self(outcomes.into_iter().collect())
    }

    pub fn contains(&self, outcome: WorkerOutcomeKind) -> bool {
        self.0.contains(&outcome)
    }

    pub fn iter(&self) -> impl Iterator<Item = &WorkerOutcomeKind> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedDrillEvidence {
    final_status: JobStatus,
    timeline: ObservedEventTimeline,
    worker_outcomes: ObservedWorkerOutcomes,
}

impl ObservedDrillEvidence {
    pub fn new(
        final_status: JobStatus,
        timeline: ObservedEventTimeline,
        worker_outcomes: ObservedWorkerOutcomes,
    ) -> Self {
        Self {
            final_status,
            timeline,
            worker_outcomes,
        }
    }

    pub fn final_status(&self) -> JobStatus {
        self.final_status
    }

    pub fn timeline(&self) -> &ObservedEventTimeline {
        &self.timeline
    }

    pub fn worker_outcomes(&self) -> &ObservedWorkerOutcomes {
        &self.worker_outcomes
    }

    pub fn verify(&self, requirements: &EvidenceRequirements) -> Result<(), FailureDrillError> {
        for requirement in requirements.iter() {
            match requirement {
                EvidenceRequirement::Event(event_type) if !self.timeline.contains(*event_type) => {
                    return Err(FailureDrillError::MissingRequiredEvent(*event_type));
                }
                EvidenceRequirement::FinalStatus(expected) if self.final_status != *expected => {
                    return Err(FailureDrillError::UnexpectedFinalStatus {
                        expected: *expected,
                        actual: self.final_status,
                    });
                }
                EvidenceRequirement::WorkerOutcome(expected)
                    if !self.worker_outcomes.contains(*expected) =>
                {
                    return Err(FailureDrillError::MissingWorkerOutcome {
                        expected: *expected,
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureDrillReport {
    plan: FailureDrillPlan,
    job_id: JobId,
    evidence: ObservedDrillEvidence,
    decision: FailureDrillDecision,
}

impl FailureDrillReport {
    pub fn new(
        plan: FailureDrillPlan,
        job_id: JobId,
        evidence: ObservedDrillEvidence,
    ) -> Result<Self, FailureDrillError> {
        evidence.verify(plan.evidence())?;
        Ok(Self {
            plan,
            job_id,
            evidence,
            decision: FailureDrillDecision::Passed,
        })
    }

    pub fn plan(&self) -> &FailureDrillPlan {
        &self.plan
    }

    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    pub fn evidence(&self) -> &ObservedDrillEvidence {
        &self.evidence
    }

    pub fn decision(&self) -> FailureDrillDecision {
        self.decision
    }
}

// ANCHOR: provider_timeout_drill
#[derive(Debug, Clone)]
pub struct ProviderTimeoutThenRetryDrill {
    worker_id: WorkerId,
    started_at: DateTime<Utc>,
    retry_policy: RetryPolicy,
}

impl ProviderTimeoutThenRetryDrill {
    pub fn new(worker_id: WorkerId, started_at: DateTime<Utc>, retry_policy: RetryPolicy) -> Self {
        Self {
            worker_id,
            started_at,
            retry_policy,
        }
    }

    pub async fn run(&self) -> Result<FailureDrillReport, FailureDrillError> {
        let mut store = InMemoryAgentJobStore::new();
        let job = drill_job(self.started_at)?;
        let job_id = store
            .enqueue(job, self.started_at)
            .await
            .map_err(FailureDrillError::from_store)?;

        let worker = Worker::new(
            self.worker_id.clone(),
            FailingThenSuccessfulRunner::new(SimulatedFailureCount::new(1)),
        )
        .with_retry_policy(self.retry_policy);

        let first_outcome = worker
            .run_once(&mut store, self.started_at)
            .await
            .map_err(FailureDrillError::from_worker)?;

        let retry_at = store
            .job(job_id)
            .ok_or(FailureDrillError::MissingJob(job_id))?
            .run_at;

        let second_outcome = worker
            .run_once(&mut store, retry_at)
            .await
            .map_err(FailureDrillError::from_worker)?;

        build_report(
            provider_timeout_plan()?,
            job_id,
            &store,
            [first_outcome.into(), second_outcome.into()],
        )
    }
}
// ANCHOR_END: provider_timeout_drill

// ANCHOR: worker_crash_drill
#[derive(Debug, Clone)]
pub struct WorkerCrashAfterLeaseDrill {
    crashed_worker_id: WorkerId,
    recovering_worker_id: WorkerId,
    started_at: DateTime<Utc>,
    lease_duration: LeaseDuration,
    recovery_delay: LeaseDuration,
}

impl WorkerCrashAfterLeaseDrill {
    pub fn new(
        crashed_worker_id: WorkerId,
        recovering_worker_id: WorkerId,
        started_at: DateTime<Utc>,
        lease_duration: LeaseDuration,
        recovery_delay: LeaseDuration,
    ) -> Result<Self, FailureDrillError> {
        if recovery_delay.duration() <= lease_duration.duration() {
            return Err(FailureDrillError::RecoveryBeforeLeaseExpiry);
        }

        Ok(Self {
            crashed_worker_id,
            recovering_worker_id,
            started_at,
            lease_duration,
            recovery_delay,
        })
    }

    pub async fn run(&self) -> Result<FailureDrillReport, FailureDrillError> {
        let mut store = InMemoryAgentJobStore::new();
        let job = drill_job(self.started_at)?;
        let job_id = store
            .enqueue(job, self.started_at)
            .await
            .map_err(FailureDrillError::from_store)?;

        store
            .pick_due_job(
                self.crashed_worker_id.clone(),
                self.started_at,
                self.lease_duration,
            )
            .await
            .map_err(FailureDrillError::from_store)?
            .ok_or(FailureDrillError::MissingJob(job_id))?;

        let recovery_time = self.started_at + self.recovery_delay.duration();
        let worker = Worker::new(self.recovering_worker_id.clone(), DeterministicAgentRunner)
            .with_lease_duration(self.lease_duration);

        let outcome = worker
            .run_once(&mut store, recovery_time)
            .await
            .map_err(FailureDrillError::from_worker)?;

        build_report(worker_crash_plan()?, job_id, &store, [outcome.into()])
    }
}
// ANCHOR_END: worker_crash_drill

fn build_report(
    plan: FailureDrillPlan,
    job_id: JobId,
    store: &InMemoryAgentJobStore,
    worker_outcomes: impl IntoIterator<Item = WorkerOutcomeKind>,
) -> Result<FailureDrillReport, FailureDrillError> {
    let job = store
        .job(job_id)
        .ok_or(FailureDrillError::MissingJob(job_id))?;
    let timeline = ObservedEventTimeline::new(
        store
            .events_for(job_id)
            .into_iter()
            .map(|event| event.event_type),
    );
    let evidence = ObservedDrillEvidence::new(
        job.status,
        timeline,
        ObservedWorkerOutcomes::new(worker_outcomes),
    );
    FailureDrillReport::new(plan, job_id, evidence)
}

fn drill_job(now: DateTime<Utc>) -> Result<AgentJob, FailureDrillError> {
    Ok(AgentJob::new(
        JobKind::new("incident_triage")?,
        AgentPayload {
            instruction: AgentInstruction::new("Investigate failed deployment")?,
        },
        MaxAttempts::new(NonZeroU32::new(3).ok_or(DomainError::NonPositiveNumber {
            field: "max_attempts",
            value: 0,
        })?),
        now,
    ))
}

fn provider_timeout_plan() -> Result<FailureDrillPlan, FailureDrillError> {
    Ok(FailureDrillPlan::new(
        FailureDrillScenario::ProviderTimeoutThenRetry,
        FailureDrillHypothesis::new(
            "a transient provider failure becomes scheduled future work and later succeeds",
        )?,
        FailureDrillBlastRadius::new("one deterministic in-memory job and one worker")?,
        FailureInjection::new("runner returns one transient provider failure before success")?,
        RollbackAction::new("stop the drill and inspect retry events before replaying")?,
        EvidenceRequirements::new([
            EvidenceRequirement::WorkerOutcome(WorkerOutcomeKind::RetryScheduled),
            EvidenceRequirement::WorkerOutcome(WorkerOutcomeKind::Succeeded),
            EvidenceRequirement::Event(JobEventType::AgentFailed),
            EvidenceRequirement::Event(JobEventType::RetryScheduled),
            EvidenceRequirement::Event(JobEventType::AgentSucceeded),
            EvidenceRequirement::Event(JobEventType::JobSucceeded),
            EvidenceRequirement::FinalStatus(JobStatus::Succeeded),
        ])?,
    ))
}

fn worker_crash_plan() -> Result<FailureDrillPlan, FailureDrillError> {
    Ok(FailureDrillPlan::new(
        FailureDrillScenario::WorkerCrashAfterLease,
        FailureDrillHypothesis::new(
            "an expired leased job recovers without losing durable ownership evidence",
        )?,
        FailureDrillBlastRadius::new("one deterministic in-memory job and two worker identities")?,
        FailureInjection::new("first worker claims a job and disappears until its lease expires")?,
        RollbackAction::new("pause the job kind and inspect queue health if recovery fails")?,
        EvidenceRequirements::new([
            EvidenceRequirement::WorkerOutcome(WorkerOutcomeKind::Succeeded),
            EvidenceRequirement::Event(JobEventType::ExpiredLeaseRecovered),
            EvidenceRequirement::Event(JobEventType::JobPicked),
            EvidenceRequirement::Event(JobEventType::AgentStarted),
            EvidenceRequirement::Event(JobEventType::AgentSucceeded),
            EvidenceRequirement::Event(JobEventType::JobSucceeded),
            EvidenceRequirement::FinalStatus(JobStatus::Succeeded),
        ])?,
    ))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use crate::domain::RetryDelay;

    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 23, 9, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    fn worker_id(value: &str) -> WorkerId {
        WorkerId::new(value).expect("valid worker id")
    }

    #[tokio::test]
    async fn provider_timeout_drill_records_retry_and_success_evidence() {
        let drill = ProviderTimeoutThenRetryDrill::new(
            worker_id("worker-a"),
            now(),
            RetryPolicy::new(RetryDelay::from_secs(10), RetryDelay::from_secs(60)),
        );

        let report = drill.run().await.expect("drill passes");

        assert_eq!(
            report.plan().scenario(),
            FailureDrillScenario::ProviderTimeoutThenRetry
        );
        assert_eq!(report.decision(), FailureDrillDecision::Passed);
        assert_eq!(report.evidence().final_status(), JobStatus::Succeeded);
        assert!(
            report
                .evidence()
                .worker_outcomes()
                .contains(WorkerOutcomeKind::RetryScheduled)
        );
        assert!(
            report
                .evidence()
                .timeline()
                .contains(JobEventType::RetryScheduled)
        );
    }

    #[tokio::test]
    async fn worker_crash_drill_records_expired_lease_recovery() {
        let drill = WorkerCrashAfterLeaseDrill::new(
            worker_id("crashed-worker"),
            worker_id("recovery-worker"),
            now(),
            LeaseDuration::from_secs(1),
            LeaseDuration::from_secs(2),
        )
        .expect("valid drill");

        let report = drill.run().await.expect("drill passes");

        assert_eq!(
            report.plan().scenario(),
            FailureDrillScenario::WorkerCrashAfterLease
        );
        assert_eq!(report.evidence().final_status(), JobStatus::Succeeded);
        assert!(
            report
                .evidence()
                .timeline()
                .contains(JobEventType::ExpiredLeaseRecovered)
        );
    }

    #[test]
    fn evidence_requirements_reject_empty_drills() {
        let error = EvidenceRequirements::new([]).expect_err("empty evidence is invalid");

        assert!(matches!(
            error,
            FailureDrillError::EmptyEvidenceRequirements
        ));
    }

    #[test]
    fn evidence_verification_rejects_missing_event() {
        let requirements =
            EvidenceRequirements::new([EvidenceRequirement::Event(JobEventType::JobDead)])
                .expect("valid requirements");
        let evidence = ObservedDrillEvidence::new(
            JobStatus::Succeeded,
            ObservedEventTimeline::new([JobEventType::JobSucceeded]),
            ObservedWorkerOutcomes::new([WorkerOutcomeKind::Succeeded]),
        );

        let error = evidence
            .verify(&requirements)
            .expect_err("missing event must fail");

        assert!(matches!(
            error,
            FailureDrillError::MissingRequiredEvent(JobEventType::JobDead)
        ));
    }

    #[test]
    fn worker_crash_drill_requires_recovery_after_lease_expiry() {
        let error = WorkerCrashAfterLeaseDrill::new(
            worker_id("crashed-worker"),
            worker_id("recovery-worker"),
            now(),
            LeaseDuration::from_secs(5),
            LeaseDuration::from_secs(5),
        )
        .expect_err("recovery must happen after lease expiry");

        assert!(matches!(
            error,
            FailureDrillError::RecoveryBeforeLeaseExpiry
        ));
    }
}
