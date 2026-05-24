//! Typed compatibility checks for long-running agent jobs.
//!
//! A worker should not blindly process every durable row it can claim. Old work
//! may have been written with an older payload schema, while future work may
//! already require a parser this binary does not understand. This module keeps
//! that long-horizon release contract explicit and testable.

use thiserror::Error;

use crate::domain::{
    AgentJob, AgentJobVersions, DomainError, JobId, JobKind, PayloadSchemaVersion, WorkerBuildId,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompatibilityError {
    #[error("minimum payload schema {minimum:?} is newer than maximum supported {maximum:?}")]
    InvalidPayloadSchemaRange {
        minimum: PayloadSchemaVersion,
        maximum: PayloadSchemaVersion,
    },
    #[error(transparent)]
    Domain(#[from] DomainError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityPolicyName(String);

impl CompatibilityPolicyName {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::EmptyText {
                field: "compatibility_policy_name",
            });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportedPayloadSchemaRange {
    minimum: PayloadSchemaVersion,
    maximum: PayloadSchemaVersion,
}

impl SupportedPayloadSchemaRange {
    pub fn new(
        minimum: PayloadSchemaVersion,
        maximum: PayloadSchemaVersion,
    ) -> Result<Self, CompatibilityError> {
        if minimum.get() > maximum.get() {
            return Err(CompatibilityError::InvalidPayloadSchemaRange { minimum, maximum });
        }

        Ok(Self { minimum, maximum })
    }

    pub fn minimum(self) -> PayloadSchemaVersion {
        self.minimum
    }

    pub fn maximum(self) -> PayloadSchemaVersion {
        self.maximum
    }

    pub fn classify(self, actual: PayloadSchemaVersion) -> PayloadSchemaCompatibility {
        if actual.get() < self.minimum.get() {
            PayloadSchemaCompatibility::TooOld {
                actual,
                minimum: self.minimum,
            }
        } else if actual.get() > self.maximum.get() {
            PayloadSchemaCompatibility::TooNew {
                actual,
                maximum: self.maximum,
            }
        } else {
            PayloadSchemaCompatibility::Supported
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadSchemaCompatibility {
    Supported,
    TooOld {
        actual: PayloadSchemaVersion,
        minimum: PayloadSchemaVersion,
    },
    TooNew {
        actual: PayloadSchemaVersion,
        maximum: PayloadSchemaVersion,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityDecision {
    Process,
    Quarantine(CompatibilityQuarantine),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityQuarantine {
    reason: CompatibilityQuarantineReason,
}

impl CompatibilityQuarantine {
    pub fn new(reason: CompatibilityQuarantineReason) -> Self {
        Self { reason }
    }

    pub fn reason(&self) -> &CompatibilityQuarantineReason {
        &self.reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityQuarantineReason {
    PayloadSchemaTooOld {
        actual: PayloadSchemaVersion,
        minimum: PayloadSchemaVersion,
    },
    PayloadSchemaTooNew {
        actual: PayloadSchemaVersion,
        maximum: PayloadSchemaVersion,
    },
}

// ANCHOR: compatibility_policy
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerCompatibilityPolicy {
    name: CompatibilityPolicyName,
    worker_build: WorkerBuildId,
    payload_schemas: SupportedPayloadSchemaRange,
}

impl WorkerCompatibilityPolicy {
    pub fn new(
        name: CompatibilityPolicyName,
        worker_build: WorkerBuildId,
        payload_schemas: SupportedPayloadSchemaRange,
    ) -> Self {
        Self {
            name,
            worker_build,
            payload_schemas,
        }
    }

    pub fn evaluate(&self, job: &AgentJob) -> JobCompatibilityReport {
        let decision = match self.payload_schemas.classify(job.versions.payload_schema) {
            PayloadSchemaCompatibility::Supported => CompatibilityDecision::Process,
            PayloadSchemaCompatibility::TooOld { actual, minimum } => {
                CompatibilityDecision::Quarantine(CompatibilityQuarantine::new(
                    CompatibilityQuarantineReason::PayloadSchemaTooOld { actual, minimum },
                ))
            }
            PayloadSchemaCompatibility::TooNew { actual, maximum } => {
                CompatibilityDecision::Quarantine(CompatibilityQuarantine::new(
                    CompatibilityQuarantineReason::PayloadSchemaTooNew { actual, maximum },
                ))
            }
        };

        JobCompatibilityReport {
            policy_name: self.name.clone(),
            worker_build: self.worker_build.clone(),
            job_id: job.id,
            job_kind: job.kind.clone(),
            job_versions: job.versions.clone(),
            decision,
        }
    }
}
// ANCHOR_END: compatibility_policy

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobCompatibilityReport {
    policy_name: CompatibilityPolicyName,
    worker_build: WorkerBuildId,
    job_id: JobId,
    job_kind: JobKind,
    job_versions: AgentJobVersions,
    decision: CompatibilityDecision,
}

impl JobCompatibilityReport {
    pub fn policy_name(&self) -> &CompatibilityPolicyName {
        &self.policy_name
    }

    pub fn worker_build(&self) -> &WorkerBuildId {
        &self.worker_build
    }

    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    pub fn job_kind(&self) -> &JobKind {
        &self.job_kind
    }

    pub fn job_versions(&self) -> &AgentJobVersions {
        &self.job_versions
    }

    pub fn decision(&self) -> &CompatibilityDecision {
        &self.decision
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::domain::{AgentInstruction, AgentPayload, MaxAttempts, ModelRoute, PromptVersion};

    fn schema(value: u32) -> PayloadSchemaVersion {
        PayloadSchemaVersion::try_from_u32(value).expect("valid payload schema version")
    }

    fn policy(minimum: u32, maximum: u32) -> Result<WorkerCompatibilityPolicy, CompatibilityError> {
        Ok(WorkerCompatibilityPolicy::new(
            CompatibilityPolicyName::new("incident-worker-compatibility:v1")?,
            WorkerBuildId::new("worker:2026.05.23")?,
            SupportedPayloadSchemaRange::new(schema(minimum), schema(maximum))?,
        ))
    }

    fn job_with_schema(payload_schema: u32) -> AgentJob {
        AgentJob::new(
            JobKind::new("incident_triage").expect("valid job kind"),
            AgentPayload {
                instruction: AgentInstruction::new("investigate failed deploy")
                    .expect("valid instruction"),
            },
            MaxAttempts::default(),
            Utc::now(),
        )
        .with_versions(AgentJobVersions {
            payload_schema: schema(payload_schema),
            prompt: PromptVersion::new("incident-triage:v2").expect("valid prompt version"),
            model_route: ModelRoute::new("deepseek-chat:v1").expect("valid model route"),
            ..AgentJobVersions::default()
        })
    }

    #[test]
    fn supported_payload_schema_can_be_processed() {
        let policy = policy(1, 3).expect("valid policy");
        let job = job_with_schema(2);

        let report = policy.evaluate(&job);

        assert_eq!(
            report.policy_name().as_str(),
            "incident-worker-compatibility:v1"
        );
        assert_eq!(report.worker_build().as_str(), "worker:2026.05.23");
        assert_eq!(report.job_id(), job.id);
        assert_eq!(report.job_kind(), &job.kind);
        assert_eq!(report.job_versions().payload_schema.get(), 2);
        assert_eq!(report.decision(), &CompatibilityDecision::Process);
    }

    #[test]
    fn too_old_payload_schema_is_quarantined() {
        let policy = policy(2, 4).expect("valid policy");
        let job = job_with_schema(1);

        let report = policy.evaluate(&job);

        assert_eq!(
            report.decision(),
            &CompatibilityDecision::Quarantine(CompatibilityQuarantine::new(
                CompatibilityQuarantineReason::PayloadSchemaTooOld {
                    actual: schema(1),
                    minimum: schema(2),
                },
            )),
        );
    }

    #[test]
    fn too_new_payload_schema_is_quarantined() {
        let policy = policy(1, 3).expect("valid policy");
        let job = job_with_schema(4);

        let report = policy.evaluate(&job);

        assert_eq!(
            report.decision(),
            &CompatibilityDecision::Quarantine(CompatibilityQuarantine::new(
                CompatibilityQuarantineReason::PayloadSchemaTooNew {
                    actual: schema(4),
                    maximum: schema(3),
                },
            )),
        );
    }

    #[test]
    fn invalid_schema_range_is_rejected() {
        let error =
            SupportedPayloadSchemaRange::new(schema(5), schema(4)).expect_err("invalid range");

        assert_eq!(
            error,
            CompatibilityError::InvalidPayloadSchemaRange {
                minimum: schema(5),
                maximum: schema(4),
            },
        );
    }

    #[test]
    fn empty_policy_name_is_rejected() {
        let error = CompatibilityPolicyName::new(" ").expect_err("empty policy name");

        assert_eq!(
            error,
            DomainError::EmptyText {
                field: "compatibility_policy_name",
            },
        );
    }
}
