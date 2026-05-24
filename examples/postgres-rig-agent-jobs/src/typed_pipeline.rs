//! Small typestate example used by the Reliable AI Agents book.
//!
//! Persisted job status is runtime data because it comes from Postgres.
//! Request construction is different: the compiler can prevent an incomplete
//! job from being built before it ever reaches the database.

use chrono::{DateTime, Utc};

use crate::domain::{
    AgentInstruction, AgentJob, AgentJobVersions, AgentPayload, IdempotencyKey, JobKind,
    MaxAttempts,
};

#[derive(Debug, Clone, Copy)]
pub struct NeedsKind;

#[derive(Debug, Clone)]
pub struct NeedsInstruction {
    kind: JobKind,
}

#[derive(Debug, Clone)]
pub struct ReadyToEnqueue {
    kind: JobKind,
    payload: AgentPayload,
}

#[derive(Debug, Clone)]
pub struct AgentJobBuilder<State> {
    state: State,
    max_attempts: MaxAttempts,
    idempotency_key: Option<IdempotencyKey>,
    versions: AgentJobVersions,
}

impl AgentJobBuilder<NeedsKind> {
    pub fn new(max_attempts: MaxAttempts) -> Self {
        Self {
            state: NeedsKind,
            max_attempts,
            idempotency_key: None,
            versions: AgentJobVersions::default(),
        }
    }

    pub fn kind(self, kind: JobKind) -> AgentJobBuilder<NeedsInstruction> {
        AgentJobBuilder {
            state: NeedsInstruction { kind },
            max_attempts: self.max_attempts,
            idempotency_key: self.idempotency_key,
            versions: self.versions,
        }
    }
}

impl<State> AgentJobBuilder<State> {
    pub fn with_idempotency_key(mut self, idempotency_key: IdempotencyKey) -> Self {
        self.idempotency_key = Some(idempotency_key);
        self
    }

    pub fn with_versions(mut self, versions: AgentJobVersions) -> Self {
        self.versions = versions;
        self
    }
}

impl AgentJobBuilder<NeedsInstruction> {
    pub fn instruction(self, instruction: AgentInstruction) -> AgentJobBuilder<ReadyToEnqueue> {
        AgentJobBuilder {
            state: ReadyToEnqueue {
                kind: self.state.kind,
                payload: AgentPayload { instruction },
            },
            max_attempts: self.max_attempts,
            idempotency_key: self.idempotency_key,
            versions: self.versions,
        }
    }
}

impl AgentJobBuilder<ReadyToEnqueue> {
    pub fn build(self, now: DateTime<Utc>) -> AgentJob {
        let job = AgentJob::new(self.state.kind, self.state.payload, self.max_attempts, now);

        let job = job.with_versions(self.versions);

        match self.idempotency_key {
            Some(idempotency_key) => job.with_idempotency_key(idempotency_key),
            None => job,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn typestate_builder_only_builds_after_required_fields_are_present() {
        let now = Utc
            .with_ymd_and_hms(2026, 5, 23, 9, 0, 0)
            .single()
            .expect("valid test timestamp");

        let job = AgentJobBuilder::new(MaxAttempts::default())
            .with_idempotency_key(IdempotencyKey::new("incident-123").expect("valid key"))
            .kind(JobKind::new("incident_triage").expect("valid kind"))
            .instruction(
                AgentInstruction::new("Analyze failed deployment").expect("valid instruction"),
            )
            .build(now);

        assert_eq!(job.kind.as_str(), "incident_triage");
        assert!(job.idempotency_key.is_some());
        assert_eq!(job.versions.payload_schema.get(), 1);
    }
}
