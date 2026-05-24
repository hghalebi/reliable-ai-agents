//! Typed boundary for adopting Temporal after the Postgres-first design.
//!
//! This module does not depend on Temporal. It models the evidence bridge the
//! book requires before a team can move a job kind from the local worker loop
//! into a workflow engine.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::approval::ApprovalRequestId;
use crate::audit::{AuditEventId, OperationEventId, TraceContext};
use crate::domain::{AgentRunId, IdempotencyKey, JobKind};
use crate::scheduled_job::ScheduledJobId;
use crate::tool_call::ToolCallId;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TemporalAdoptionError {
    #[error("{field} cannot be empty")]
    EmptyText { field: &'static str },
    #[error("Temporal reconciliation requires at least one activity receipt")]
    MissingActivityReceipts,
}

fn non_empty_temporal_text(
    value: impl Into<String>,
    field: &'static str,
) -> Result<String, TemporalAdoptionError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(TemporalAdoptionError::EmptyText { field });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemporalWorkflowType(String);

impl TemporalWorkflowType {
    pub fn new(value: impl Into<String>) -> Result<Self, TemporalAdoptionError> {
        Ok(Self(non_empty_temporal_text(
            value,
            "temporal_workflow_type",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemporalWorkflowExecutionRef(String);

impl TemporalWorkflowExecutionRef {
    pub fn new(value: impl Into<String>) -> Result<Self, TemporalAdoptionError> {
        Ok(Self(non_empty_temporal_text(
            value,
            "temporal_workflow_execution_ref",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemporalActivityExecutionRef(String);

impl TemporalActivityExecutionRef {
    pub fn new(value: impl Into<String>) -> Result<Self, TemporalAdoptionError> {
        Ok(Self(non_empty_temporal_text(
            value,
            "temporal_activity_execution_ref",
        )?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemporalTaskQueue(String);

impl TemporalTaskQueue {
    pub fn new(value: impl Into<String>) -> Result<Self, TemporalAdoptionError> {
        Ok(Self(non_empty_temporal_text(value, "temporal_task_queue")?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ANCHOR: temporal_bridge
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalWorkflowBridge {
    scheduled_job_id: ScheduledJobId,
    agent_run_id: AgentRunId,
    job_kind: JobKind,
    workflow_type: TemporalWorkflowType,
    workflow_ref: TemporalWorkflowExecutionRef,
    task_queue: TemporalTaskQueue,
    idempotency_key: IdempotencyKey,
    trace: TraceContext,
    started_at: DateTime<Utc>,
}

impl TemporalWorkflowBridge {
    pub fn new(input: TemporalWorkflowBridgeInput) -> Self {
        Self {
            scheduled_job_id: input.scheduled_job_id,
            agent_run_id: input.agent_run_id,
            job_kind: input.job_kind,
            workflow_type: input.workflow_type,
            workflow_ref: input.workflow_ref,
            task_queue: input.task_queue,
            idempotency_key: input.idempotency_key,
            trace: input.trace,
            started_at: input.started_at,
        }
    }

    pub fn scheduled_job_id(&self) -> ScheduledJobId {
        self.scheduled_job_id
    }

    pub fn agent_run_id(&self) -> AgentRunId {
        self.agent_run_id
    }

    pub fn workflow_ref(&self) -> &TemporalWorkflowExecutionRef {
        &self.workflow_ref
    }

    pub fn trace(&self) -> &TraceContext {
        &self.trace
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalWorkflowBridgeInput {
    pub scheduled_job_id: ScheduledJobId,
    pub agent_run_id: AgentRunId,
    pub job_kind: JobKind,
    pub workflow_type: TemporalWorkflowType,
    pub workflow_ref: TemporalWorkflowExecutionRef,
    pub task_queue: TemporalTaskQueue,
    pub idempotency_key: IdempotencyKey,
    pub trace: TraceContext,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalActivityReceipt {
    workflow_ref: TemporalWorkflowExecutionRef,
    activity_ref: TemporalActivityExecutionRef,
    agent_run_id: AgentRunId,
    tool_call_id: ToolCallId,
    idempotency_key: IdempotencyKey,
    operation_event_id: OperationEventId,
    recorded_at: DateTime<Utc>,
}

impl TemporalActivityReceipt {
    pub fn new(input: TemporalActivityReceiptInput) -> Self {
        Self {
            workflow_ref: input.workflow_ref,
            activity_ref: input.activity_ref,
            agent_run_id: input.agent_run_id,
            tool_call_id: input.tool_call_id,
            idempotency_key: input.idempotency_key,
            operation_event_id: input.operation_event_id,
            recorded_at: input.recorded_at,
        }
    }

    pub fn operation_event_id(&self) -> OperationEventId {
        self.operation_event_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalActivityReceiptInput {
    pub workflow_ref: TemporalWorkflowExecutionRef,
    pub activity_ref: TemporalActivityExecutionRef,
    pub agent_run_id: AgentRunId,
    pub tool_call_id: ToolCallId,
    pub idempotency_key: IdempotencyKey,
    pub operation_event_id: OperationEventId,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalActivityReceipts(Vec<TemporalActivityReceipt>);

impl TemporalActivityReceipts {
    pub fn new(receipts: Vec<TemporalActivityReceipt>) -> Result<Self, TemporalAdoptionError> {
        if receipts.is_empty() {
            return Err(TemporalAdoptionError::MissingActivityReceipts);
        }
        Ok(Self(receipts))
    }

    pub fn as_slice(&self) -> &[TemporalActivityReceipt] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalProductEvidence {
    operation_event_id: OperationEventId,
    audit_event_id: AuditEventId,
    approval_request_id: Option<ApprovalRequestId>,
}

impl TemporalProductEvidence {
    pub fn new(
        operation_event_id: OperationEventId,
        audit_event_id: AuditEventId,
        approval_request_id: Option<ApprovalRequestId>,
    ) -> Self {
        Self {
            operation_event_id,
            audit_event_id,
            approval_request_id,
        }
    }

    pub fn audit_event_id(&self) -> AuditEventId {
        self.audit_event_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalReconciliationPacket {
    bridge: TemporalWorkflowBridge,
    activity_receipts: TemporalActivityReceipts,
    product_evidence: TemporalProductEvidence,
}

impl TemporalReconciliationPacket {
    pub fn new(
        bridge: TemporalWorkflowBridge,
        activity_receipts: TemporalActivityReceipts,
        product_evidence: TemporalProductEvidence,
    ) -> Self {
        Self {
            bridge,
            activity_receipts,
            product_evidence,
        }
    }

    pub fn bridge(&self) -> &TemporalWorkflowBridge {
        &self.bridge
    }

    pub fn activity_receipts(&self) -> &[TemporalActivityReceipt] {
        self.activity_receipts.as_slice()
    }

    pub fn product_evidence(&self) -> &TemporalProductEvidence {
        &self.product_evidence
    }
}
// ANCHOR_END: temporal_bridge

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{SpanId, TraceId};
    use uuid::Uuid;

    fn fixed_time() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("valid fixed timestamp")
    }

    fn trace() -> Result<TraceContext, Box<dyn std::error::Error>> {
        let trace_id = TraceId::new("4bf92f3577b34da6a3ce929d0e0e4736")?;
        let span_id = SpanId::new("00f067aa0ba902b7")?;
        Ok(TraceContext::new(trace_id, Some(span_id)))
    }

    fn bridge() -> Result<TemporalWorkflowBridge, Box<dyn std::error::Error>> {
        Ok(TemporalWorkflowBridge::new(TemporalWorkflowBridgeInput {
            scheduled_job_id: ScheduledJobId::from_uuid(Uuid::nil()),
            agent_run_id: AgentRunId::from_uuid(Uuid::nil()),
            job_kind: JobKind::new("kyc_case_preparation")?,
            workflow_type: TemporalWorkflowType::new("prepare_kyc_case")?,
            workflow_ref: TemporalWorkflowExecutionRef::new("kyc-case-00000000")?,
            task_queue: TemporalTaskQueue::new("agent-workflows")?,
            idempotency_key: IdempotencyKey::new("temporal:kyc-case-00000000")?,
            trace: trace()?,
            started_at: fixed_time(),
        }))
    }

    fn activity_receipt() -> Result<TemporalActivityReceipt, Box<dyn std::error::Error>> {
        Ok(TemporalActivityReceipt::new(TemporalActivityReceiptInput {
            workflow_ref: TemporalWorkflowExecutionRef::new("kyc-case-00000000")?,
            activity_ref: TemporalActivityExecutionRef::new("extract-documents:attempt-1")?,
            agent_run_id: AgentRunId::from_uuid(Uuid::nil()),
            tool_call_id: ToolCallId::from_uuid(Uuid::nil()),
            idempotency_key: IdempotencyKey::new("tool:extract-documents:00000000")?,
            operation_event_id: OperationEventId::try_from_i64(11)?,
            recorded_at: fixed_time(),
        }))
    }

    #[test]
    fn rejects_empty_workflow_execution_ref() {
        assert!(matches!(
            TemporalWorkflowExecutionRef::new(" "),
            Err(TemporalAdoptionError::EmptyText {
                field: "temporal_workflow_execution_ref"
            })
        ));
    }

    #[test]
    fn reconciliation_requires_activity_receipts() {
        assert!(matches!(
            TemporalActivityReceipts::new(Vec::new()),
            Err(TemporalAdoptionError::MissingActivityReceipts)
        ));
    }

    #[test]
    fn reconciliation_packet_keeps_workflow_and_product_evidence_together()
    -> Result<(), Box<dyn std::error::Error>> {
        let receipts = TemporalActivityReceipts::new(vec![activity_receipt()?])?;
        let product_evidence = TemporalProductEvidence::new(
            OperationEventId::try_from_i64(12)?,
            AuditEventId::try_from_i64(13)?,
            Some(ApprovalRequestId::from_uuid(Uuid::nil())),
        );

        let packet = TemporalReconciliationPacket::new(bridge()?, receipts, product_evidence);

        assert_eq!(packet.bridge().workflow_ref().as_str(), "kyc-case-00000000");
        assert_eq!(packet.activity_receipts().len(), 1);
        assert_eq!(packet.product_evidence().audit_event_id().get(), 13);
        Ok(())
    }
}
