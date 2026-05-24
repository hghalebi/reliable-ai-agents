//! HTTP admission boundary for the production MVP.
//!
//! This module keeps Axum at the edge. Request JSON and headers are raw
//! transport data only until they are converted into typed domain values and a
//! durable `AgentJob`.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::admission_control::{
    AdmissionControlError, AdmissionControlInput, AdmissionDecision, AdmissionDecisionEvent,
    AdmissionDecisionKind, AdmissionDelay, AdmissionPolicy, AdmissionPolicyName, AdmissionReason,
    AdmissionRequestId, AdmissionSignals, AdmissionSubject, BudgetAdmissionState, JobPriority,
    MaxOldestPendingAge, MaxPendingDepth, ProviderQuotaPressure,
};
use crate::cost_accounting::CostMicrosUsd;
use crate::domain::{
    AgentInstruction, AgentJob, AgentJobSnapshot, AgentJobVersions, AgentPayload, DomainError,
    FailureMessage, IdempotencyKey, JobId, JobKind, JobStatus, MaxAttempts, ModelRoute,
    PayloadSchemaVersion, PolicyVersion, PromptVersion, QueueAge, QueueDepth, QueueMetrics,
    TenantKey, ToolVersion, WorkerBuildId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpBindAddress(SocketAddr);

impl HttpBindAddress {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ApiError> {
        let value = value.as_ref();
        value
            .parse()
            .map(Self)
            .map_err(|_| ApiError::InvalidBindAddress(FailureMessage::from_error_text(value)))
    }

    pub fn socket_addr(self) -> SocketAddr {
        self.0
    }
}

#[derive(Debug)]
pub struct ApiState<S> {
    store: Arc<Mutex<S>>,
    admission_policy: AdmissionPolicy,
}

impl<S> Clone for ApiState<S> {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            admission_policy: self.admission_policy.clone(),
        }
    }
}

impl<S> ApiState<S> {
    pub fn new(store: S) -> Result<Self, AdmissionControlError> {
        Ok(Self::new_with_admission_policy(
            store,
            default_admission_policy()?,
        ))
    }

    pub fn new_with_admission_policy(store: S, admission_policy: AdmissionPolicy) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
            admission_policy,
        }
    }
}

fn default_admission_policy() -> Result<AdmissionPolicy, AdmissionControlError> {
    Ok(AdmissionPolicy::new(
        AdmissionPolicyName::new("api-default-admission:v1")?,
        MaxPendingDepth::new(QueueDepth::try_from_usize(100)?)?,
        MaxOldestPendingAge::new(QueueAge::from_seconds(300)?)?,
        AdmissionDelay::from_seconds(60)?,
    ))
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("missing Idempotency-Key header")]
    MissingIdempotencyKey,
    #[error("Idempotency-Key header must be valid UTF-8")]
    InvalidIdempotencyKeyHeader,
    #[error("request failed domain validation: {0}")]
    Domain(#[from] DomainError),
    #[error("job store failed: {0}")]
    Store(FailureMessage),
    #[error("admission control failed: {0}")]
    Admission(#[from] AdmissionControlError),
    #[error("admission rejected: {reason:?}")]
    AdmissionRejected { reason: AdmissionReason },
    #[error("readiness dependency check failed: {0}")]
    Readiness(FailureMessage),
    #[error("invalid HTTP bind address: {0}")]
    InvalidBindAddress(FailureMessage),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::MissingIdempotencyKey
            | Self::InvalidIdempotencyKeyHeader
            | Self::Domain(_)
            | Self::Admission(_)
            | Self::InvalidBindAddress(_) => StatusCode::BAD_REQUEST,
            Self::AdmissionRejected { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Readiness(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::MissingIdempotencyKey => "missing_idempotency_key",
            Self::InvalidIdempotencyKeyHeader => "invalid_idempotency_key_header",
            Self::Domain(_) => "domain_validation_failed",
            Self::Store(_) => "store_failed",
            Self::Admission(_) => "admission_control_failed",
            Self::AdmissionRejected { .. } => "admission_rejected",
            Self::Readiness(_) => "readiness_check_failed",
            Self::InvalidBindAddress(_) => "invalid_bind_address",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ApiErrorResponse {
            error_code: self.error_code(),
            message: self.to_string(),
        };
        (status, Json(body)).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error_code: &'static str,
    pub message: String,
}

#[async_trait]
pub trait AgentJobObservabilityStore {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn queue_metrics(
        &mut self,
        now: DateTime<Utc>,
    ) -> Result<QueueMetrics, <Self as AgentJobObservabilityStore>::Error>;
}

#[async_trait]
impl AgentJobObservabilityStore for crate::memory_store::InMemoryAgentJobStore {
    type Error = crate::memory_store::InMemoryStoreError;

    async fn queue_metrics(
        &mut self,
        now: DateTime<Utc>,
    ) -> Result<QueueMetrics, <Self as AgentJobObservabilityStore>::Error> {
        Ok(self.metrics(now))
    }
}

#[cfg(feature = "postgres-store")]
#[async_trait]
impl AgentJobObservabilityStore for crate::PostgresAgentJobStore {
    type Error = crate::PostgresStoreError;

    async fn queue_metrics(
        &mut self,
        _now: DateTime<Utc>,
    ) -> Result<QueueMetrics, <Self as AgentJobObservabilityStore>::Error> {
        self.metrics().await
    }
}

#[async_trait]
pub trait AgentAdmissionStore {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn existing_job_for_idempotency_key(
        &mut self,
        idempotency_key: &IdempotencyKey,
        now: DateTime<Utc>,
    ) -> Result<Option<AgentJobSnapshot>, <Self as AgentAdmissionStore>::Error>;

    async fn admit_agent_job(
        &mut self,
        job: AgentJob,
        event: AdmissionDecisionEvent,
        now: DateTime<Utc>,
    ) -> Result<JobId, <Self as AgentAdmissionStore>::Error>;

    async fn record_admission_decision(
        &mut self,
        event: AdmissionDecisionEvent,
    ) -> Result<(), <Self as AgentAdmissionStore>::Error>;
}

#[async_trait]
impl AgentAdmissionStore for crate::memory_store::InMemoryAgentJobStore {
    type Error = crate::memory_store::InMemoryStoreError;

    async fn existing_job_for_idempotency_key(
        &mut self,
        idempotency_key: &IdempotencyKey,
        now: DateTime<Utc>,
    ) -> Result<Option<AgentJobSnapshot>, <Self as AgentAdmissionStore>::Error> {
        crate::memory_store::InMemoryAgentJobStore::existing_job_for_idempotency_key(
            self,
            idempotency_key,
            now,
        )
        .await
    }

    async fn admit_agent_job(
        &mut self,
        job: AgentJob,
        event: AdmissionDecisionEvent,
        now: DateTime<Utc>,
    ) -> Result<JobId, <Self as AgentAdmissionStore>::Error> {
        crate::memory_store::InMemoryAgentJobStore::admit_agent_job(self, job, event, now).await
    }

    async fn record_admission_decision(
        &mut self,
        event: AdmissionDecisionEvent,
    ) -> Result<(), <Self as AgentAdmissionStore>::Error> {
        crate::memory_store::InMemoryAgentJobStore::record_admission_decision(self, event).await
    }
}

#[cfg(feature = "postgres-store")]
#[async_trait]
impl AgentAdmissionStore for crate::PostgresAgentJobStore {
    type Error = crate::PostgresStoreError;

    async fn existing_job_for_idempotency_key(
        &mut self,
        idempotency_key: &IdempotencyKey,
        now: DateTime<Utc>,
    ) -> Result<Option<AgentJobSnapshot>, <Self as AgentAdmissionStore>::Error> {
        crate::PostgresAgentJobStore::existing_job_for_idempotency_key(self, idempotency_key, now)
            .await
    }

    async fn admit_agent_job(
        &mut self,
        job: AgentJob,
        event: AdmissionDecisionEvent,
        now: DateTime<Utc>,
    ) -> Result<JobId, <Self as AgentAdmissionStore>::Error> {
        crate::PostgresAgentJobStore::admit_agent_job(self, job, event, now).await
    }

    async fn record_admission_decision(
        &mut self,
        event: AdmissionDecisionEvent,
    ) -> Result<(), <Self as AgentAdmissionStore>::Error> {
        crate::PostgresAgentJobStore::record_admission_decision(self, event).await
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentJobRequest {
    pub kind: String,
    pub instruction: String,
    pub tenant_key: Option<String>,
    pub priority: Option<String>,
    pub provider_quota_pressure: Option<String>,
    pub max_attempts: Option<u32>,
    pub payload_schema_version: Option<u32>,
    pub prompt_version: Option<String>,
    pub model_route: Option<String>,
    pub tool_version: Option<String>,
    pub policy_version: Option<String>,
    pub worker_build_id: Option<String>,
}

// ANCHOR: api_admission_command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAgentJobCommand {
    tenant_key: TenantKey,
    kind: JobKind,
    instruction: AgentInstruction,
    priority: JobPriority,
    provider_pressure: ProviderQuotaPressure,
    idempotency_key: IdempotencyKey,
    max_attempts: MaxAttempts,
    versions: AgentJobVersions,
}

impl CreateAgentJobCommand {
    pub fn from_http_parts(
        request: CreateAgentJobRequest,
        idempotency_key: IdempotencyKey,
    ) -> Result<Self, ApiError> {
        let max_attempts = match request.max_attempts {
            Some(value) => MaxAttempts::try_from_u32(value)?,
            None => MaxAttempts::default(),
        };

        let payload_schema = match request.payload_schema_version {
            Some(value) => PayloadSchemaVersion::try_from_u32(value)?,
            None => PayloadSchemaVersion::default(),
        };

        let versions = AgentJobVersions {
            payload_schema,
            prompt: optional_version(request.prompt_version, PromptVersion::new)?,
            model_route: optional_version(request.model_route, ModelRoute::new)?,
            tool: optional_version(request.tool_version, ToolVersion::new)?,
            policy: optional_version(request.policy_version, PolicyVersion::new)?,
            worker_build: optional_version(request.worker_build_id, WorkerBuildId::new)?,
        };
        let tenant_key = match request.tenant_key {
            Some(value) => TenantKey::new(value)?,
            None => TenantKey::new("default-tenant")?,
        };
        let priority = match request.priority {
            Some(value) => JobPriority::try_from(value.as_str())?,
            None => JobPriority::Standard,
        };
        let provider_pressure = match request.provider_quota_pressure {
            Some(value) => ProviderQuotaPressure::try_from(value.as_str())?,
            None => ProviderQuotaPressure::Healthy,
        };

        Ok(Self {
            tenant_key,
            kind: JobKind::new(request.kind)?,
            instruction: AgentInstruction::new(request.instruction)?,
            priority,
            provider_pressure,
            idempotency_key,
            max_attempts,
            versions,
        })
    }

    pub fn admission_subject(&self) -> AdmissionSubject {
        AdmissionSubject::new(
            AdmissionRequestId::new(),
            self.tenant_key.clone(),
            self.kind.clone(),
            self.priority,
        )
    }

    pub fn admission_signals(&self, queue_metrics: QueueMetrics) -> AdmissionSignals {
        AdmissionSignals::new(
            queue_metrics,
            self.provider_pressure,
            BudgetAdmissionState::WithinBudget {
                projected: CostMicrosUsd::zero(),
                remaining: CostMicrosUsd::new(10_000_000_000),
            },
        )
    }

    pub fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    pub fn into_job(self, now: DateTime<Utc>, run_at: DateTime<Utc>) -> AgentJob {
        AgentJob::new(
            self.kind,
            AgentPayload {
                instruction: self.instruction,
            },
            self.max_attempts,
            now,
        )
        .with_idempotency_key(self.idempotency_key)
        .with_versions(self.versions)
        .with_run_at(run_at)
    }
}
// ANCHOR_END: api_admission_command

fn optional_version<T>(
    value: Option<String>,
    parse: impl FnOnce(String) -> Result<T, DomainError>,
) -> Result<T, DomainError>
where
    T: Default,
{
    match value {
        Some(value) => parse(value),
        None => Ok(T::default()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateAgentJobResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub accepted_at: DateTime<Utc>,
    pub run_at: DateTime<Utc>,
    pub admission_request_id: Option<Uuid>,
    pub admission_decision: Option<AdmissionDecisionKind>,
    pub admission_reason: Option<AdmissionReason>,
    pub idempotency_outcome: IdempotencyOutcome,
}

impl CreateAgentJobResponse {
    fn newly_admitted(
        job_id: JobId,
        accepted_at: DateTime<Utc>,
        run_at: DateTime<Utc>,
        admission_request_id: AdmissionRequestId,
        admission_decision: AdmissionDecisionKind,
        admission_reason: AdmissionReason,
    ) -> Self {
        Self {
            job_id: job_id.as_uuid(),
            status: JobStatus::Pending,
            accepted_at,
            run_at,
            admission_request_id: Some(admission_request_id.as_uuid()),
            admission_decision: Some(admission_decision),
            admission_reason: Some(admission_reason),
            idempotency_outcome: IdempotencyOutcome::NewlyAdmitted,
        }
    }

    fn duplicate(existing_job: AgentJobSnapshot, accepted_at: DateTime<Utc>) -> Self {
        Self {
            job_id: existing_job.id().as_uuid(),
            status: existing_job.status(),
            accepted_at,
            run_at: existing_job.run_at(),
            admission_request_id: None,
            admission_decision: None,
            admission_reason: None,
            idempotency_outcome: IdempotencyOutcome::DuplicateSuppressed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdempotencyOutcome {
    NewlyAdmitted,
    DuplicateSuppressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessStatus {
    Ready,
}

impl ReadinessStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueMetricsResponse {
    pub pending: usize,
    pub running: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub dead: usize,
    pub cancelled: usize,
    pub oldest_pending_age_seconds: Option<i64>,
}

impl From<QueueMetrics> for QueueMetricsResponse {
    fn from(metrics: QueueMetrics) -> Self {
        Self {
            pending: metrics.pending.get(),
            running: metrics.running.get(),
            succeeded: metrics.succeeded.get(),
            failed: metrics.failed.get(),
            dead: metrics.dead.get(),
            cancelled: metrics.cancelled.get(),
            oldest_pending_age_seconds: metrics.oldest_pending_age.map(|age| age.seconds()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub status: ReadinessStatus,
    pub checked_at: DateTime<Utc>,
    pub queue: QueueMetricsResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub queue: QueueMetricsResponse,
}

// ANCHOR: api_router
pub fn router<S>(store: S) -> Result<Router, ApiError>
where
    S: AgentJobObservabilityStore + AgentAdmissionStore + Send + 'static,
{
    Ok(router_with_admission_policy(
        store,
        default_admission_policy()?,
    ))
}

pub fn router_with_admission_policy<S>(store: S, admission_policy: AdmissionPolicy) -> Router
where
    S: AgentJobObservabilityStore + AgentAdmissionStore + Send + 'static,
{
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz::<S>))
        .route("/metrics", get(metrics::<S>))
        .route("/agent-jobs", post(create_agent_job::<S>))
        .with_state(ApiState::new_with_admission_policy(store, admission_policy))
}

async fn healthz() -> StatusCode {
    StatusCode::NO_CONTENT
}

// ANCHOR: api_runtime_surfaces
async fn readyz<S>(State(state): State<ApiState<S>>) -> Result<Json<ReadinessResponse>, ApiError>
where
    S: AgentJobObservabilityStore + Send + 'static,
{
    let checked_at = Utc::now();
    let mut store = state.store.lock().await;
    let queue = store
        .queue_metrics(checked_at)
        .await
        .map_err(|error| ApiError::Readiness(FailureMessage::from_error_text(error.to_string())))?;

    Ok(Json(ReadinessResponse {
        status: ReadinessStatus::Ready,
        checked_at,
        queue: queue.into(),
    }))
}

async fn metrics<S>(State(state): State<ApiState<S>>) -> Result<Json<MetricsResponse>, ApiError>
where
    S: AgentJobObservabilityStore + Send + 'static,
{
    let now = Utc::now();
    let mut store = state.store.lock().await;
    let queue = store
        .queue_metrics(now)
        .await
        .map_err(|error| ApiError::Readiness(FailureMessage::from_error_text(error.to_string())))?;

    Ok(Json(MetricsResponse {
        queue: queue.into(),
    }))
}
// ANCHOR_END: api_runtime_surfaces

// ANCHOR: api_admission_enforcement
async fn create_agent_job<S>(
    State(state): State<ApiState<S>>,
    headers: HeaderMap,
    Json(request): Json<CreateAgentJobRequest>,
) -> Result<(StatusCode, Json<CreateAgentJobResponse>), ApiError>
where
    S: AgentJobObservabilityStore + AgentAdmissionStore + Send + 'static,
{
    let now = Utc::now();
    let idempotency_key = idempotency_key_from_headers(&headers)?;
    let command = CreateAgentJobCommand::from_http_parts(request, idempotency_key)?;
    let mut store = state.store.lock().await;
    if let Some(existing_job) = store
        .existing_job_for_idempotency_key(command.idempotency_key(), now)
        .await
        .map_err(|error| ApiError::Store(FailureMessage::from_error_text(error.to_string())))?
    {
        return Ok((
            StatusCode::ACCEPTED,
            Json(CreateAgentJobResponse::duplicate(existing_job, now)),
        ));
    }

    let queue_metrics = store
        .queue_metrics(now)
        .await
        .map_err(|error| ApiError::Store(FailureMessage::from_error_text(error.to_string())))?;
    let admission_event = state.admission_policy.evaluate(AdmissionControlInput::new(
        command.admission_subject(),
        command.admission_signals(queue_metrics),
        now,
    ))?;

    if let AdmissionDecision::Rejected { reason } = admission_event.decision() {
        store
            .record_admission_decision(admission_event)
            .await
            .map_err(|error| ApiError::Store(FailureMessage::from_error_text(error.to_string())))?;
        return Err(ApiError::AdmissionRejected { reason });
    }

    let run_at = admission_event.decision().next_run_at().unwrap_or(now);
    let admission_request_id = admission_event.request_id();
    let admission_decision = admission_event.decision().kind();
    let admission_reason = admission_event.decision().reason();
    let job_id = store
        .admit_agent_job(command.into_job(now, run_at), admission_event, now)
        .await
        .map_err(|error| ApiError::Store(FailureMessage::from_error_text(error.to_string())))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(CreateAgentJobResponse::newly_admitted(
            job_id,
            now,
            run_at,
            admission_request_id,
            admission_decision,
            admission_reason,
        )),
    ))
}
// ANCHOR_END: api_admission_enforcement
// ANCHOR_END: api_router

fn idempotency_key_from_headers(headers: &HeaderMap) -> Result<IdempotencyKey, ApiError> {
    let value = headers
        .get(AgentJobAdmission::IDEMPOTENCY_HEADER)
        .ok_or(ApiError::MissingIdempotencyKey)?;
    let value = header_value_as_str(value)?;
    Ok(IdempotencyKey::new(value)?)
}

fn header_value_as_str(value: &HeaderValue) -> Result<&str, ApiError> {
    value
        .to_str()
        .map_err(|_| ApiError::InvalidIdempotencyKeyHeader)
}

#[cfg(feature = "postgres-store")]
pub async fn serve_postgres_api(
    database_url: crate::DatabaseUrl,
    bind_address: HttpBindAddress,
) -> Result<(), PostgresApiServerError>
where
    crate::PostgresAgentJobStore: AgentJobObservabilityStore + AgentAdmissionStore + Send + 'static,
{
    let store = crate::PostgresAgentJobStore::connect(database_url).await?;
    let listener = tokio::net::TcpListener::bind(bind_address.socket_addr())
        .await
        .map_err(PostgresApiServerError::Bind)?;
    axum::serve(listener, router(store)?)
        .await
        .map_err(PostgresApiServerError::Serve)?;
    Ok(())
}

#[cfg(feature = "postgres-store")]
#[derive(Debug, Error)]
pub enum PostgresApiServerError {
    #[error("postgres store failed: {0}")]
    Store(#[from] crate::PostgresStoreError),
    #[error("api router failed: {0}")]
    Router(#[from] ApiError),
    #[error("failed to bind API listener: {0}")]
    Bind(std::io::Error),
    #[error("API server failed while serving requests: {0}")]
    Serve(std::io::Error),
}

pub struct AgentJobAdmission;

impl AgentJobAdmission {
    pub const ROUTE: &'static str = "POST /agent-jobs";
    pub const IDEMPOTENCY_HEADER: &'static str = "Idempotency-Key";
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::header;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use super::*;
    use crate::memory_store::InMemoryAgentJobStore;
    use crate::worker::AgentJobStore;

    fn small_admission_policy(max_pending: usize) -> AdmissionPolicy {
        AdmissionPolicy::new(
            AdmissionPolicyName::new("test-admission:v1").expect("valid policy name"),
            MaxPendingDepth::new(
                QueueDepth::try_from_usize(max_pending).expect("valid max pending"),
            )
            .expect("positive max pending"),
            MaxOldestPendingAge::new(QueueAge::from_seconds(600).expect("valid max age"))
                .expect("positive max age"),
            AdmissionDelay::from_seconds(30).expect("positive delay"),
        )
    }

    fn test_job(now: DateTime<Utc>) -> AgentJob {
        AgentJob::new(
            JobKind::new("incident_triage").expect("valid kind"),
            AgentPayload {
                instruction: AgentInstruction::new("Existing queued work")
                    .expect("valid instruction"),
            },
            MaxAttempts::default(),
            now,
        )
    }

    fn request(body: &'static str) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri("/agent-jobs")
            .header(header::CONTENT_TYPE, "application/json")
            .header("Idempotency-Key", "incident:api:42")
            .body(Body::from(body))
            .expect("valid request")
    }

    fn get_request(path: &'static str) -> Request<Body> {
        Request::builder()
            .method(Method::GET)
            .uri(path)
            .body(Body::empty())
            .expect("valid request")
    }

    async fn response_body(response: Response) -> Vec<u8> {
        to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body")
            .to_vec()
    }

    #[tokio::test]
    async fn api_admission_enqueues_typed_job() {
        let app = router(InMemoryAgentJobStore::new()).expect("default admission policy");

        let response = app
            .oneshot(request(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment"}"#,
            ))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body: CreateAgentJobResponse =
            serde_json::from_slice(&response_body(response).await).expect("json body");

        assert_eq!(body.status, JobStatus::Pending);
        assert_eq!(body.idempotency_outcome, IdempotencyOutcome::NewlyAdmitted);
        assert!(body.admission_request_id.is_some());
        assert_eq!(
            body.admission_decision,
            Some(AdmissionDecisionKind::Accepted)
        );
        assert_eq!(
            body.admission_reason,
            Some(AdmissionReason::WithinOperatingEnvelope)
        );
        assert_eq!(body.run_at, body.accepted_at);
    }

    #[tokio::test]
    async fn health_endpoint_is_liveness_only() {
        let app = router(InMemoryAgentJobStore::new()).expect("default admission policy");

        let response = app
            .oneshot(get_request("/healthz"))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn readiness_endpoint_checks_queue_dependency() {
        let app = router(InMemoryAgentJobStore::new()).expect("default admission policy");

        let response = app
            .oneshot(get_request("/readyz"))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::OK);
        let body: ReadinessResponse =
            serde_json::from_slice(&response_body(response).await).expect("readiness json");
        assert_eq!(body.status, ReadinessStatus::Ready);
        assert_eq!(body.queue.pending, 0);
    }

    #[tokio::test]
    async fn metrics_endpoint_exposes_queue_metrics_after_admission() {
        let app = router(InMemoryAgentJobStore::new()).expect("default admission policy");
        let admission_response = app
            .clone()
            .oneshot(request(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment"}"#,
            ))
            .await
            .expect("admission response");
        assert_eq!(admission_response.status(), StatusCode::ACCEPTED);

        let response = app
            .oneshot(get_request("/metrics"))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::OK);
        let body: MetricsResponse =
            serde_json::from_slice(&response_body(response).await).expect("metrics json");
        assert_eq!(body.queue.pending, 1);
        assert_eq!(body.queue.running, 0);
    }

    #[tokio::test]
    async fn duplicate_idempotency_key_returns_same_job() {
        let app = router(InMemoryAgentJobStore::new()).expect("default admission policy");
        let first = app
            .clone()
            .oneshot(request(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment"}"#,
            ))
            .await
            .expect("first response");
        let second = app
            .oneshot(request(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment"}"#,
            ))
            .await
            .expect("second response");

        let first_body: CreateAgentJobResponse =
            serde_json::from_slice(&response_body(first).await).expect("first json");
        let second_body: CreateAgentJobResponse =
            serde_json::from_slice(&response_body(second).await).expect("second json");

        assert_eq!(first_body.job_id, second_body.job_id);
        assert_eq!(
            second_body.idempotency_outcome,
            IdempotencyOutcome::DuplicateSuppressed
        );
        assert_eq!(second_body.admission_request_id, None);
        assert_eq!(second_body.admission_decision, None);
        assert_eq!(second_body.admission_reason, None);
    }

    #[tokio::test]
    async fn duplicate_idempotency_key_returns_existing_job_before_overload_rejection() {
        let app =
            router_with_admission_policy(InMemoryAgentJobStore::new(), small_admission_policy(1));
        let first = app
            .clone()
            .oneshot(request(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment","priority":"standard"}"#,
            ))
            .await
            .expect("first response");
        assert_eq!(first.status(), StatusCode::ACCEPTED);
        let first_body: CreateAgentJobResponse =
            serde_json::from_slice(&response_body(first).await).expect("first json");

        let second = app
            .oneshot(request(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment","priority":"bulk"}"#,
            ))
            .await
            .expect("second response");

        assert_eq!(second.status(), StatusCode::ACCEPTED);
        let second_body: CreateAgentJobResponse =
            serde_json::from_slice(&response_body(second).await).expect("second json");
        assert_eq!(first_body.job_id, second_body.job_id);
        assert_eq!(
            second_body.idempotency_outcome,
            IdempotencyOutcome::DuplicateSuppressed
        );
        assert_eq!(second_body.admission_request_id, None);
        assert_eq!(second_body.admission_decision, None);
        assert_eq!(second_body.admission_reason, None);
    }

    #[tokio::test]
    async fn missing_idempotency_key_is_rejected_before_enqueue() {
        let app = router(InMemoryAgentJobStore::new()).expect("default admission policy");
        let request = Request::builder()
            .method(Method::POST)
            .uri("/agent-jobs")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment"}"#,
            ))
            .expect("valid request");

        let response = app.oneshot(request).await.expect("route response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&response_body(response).await).expect("json body");
        assert_eq!(body["error_code"], "missing_idempotency_key");
    }

    #[tokio::test]
    async fn invalid_domain_values_are_rejected_at_http_boundary() {
        let app = router(InMemoryAgentJobStore::new()).expect("default admission policy");

        let response = app
            .oneshot(request(r#"{"kind":"incident_triage","instruction":"  "}"#))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&response_body(response).await).expect("json body");
        assert_eq!(body["error_code"], "domain_validation_failed");
    }

    #[tokio::test]
    async fn admission_policy_delays_standard_work_when_queue_is_saturated() {
        let now = Utc::now();
        let mut store = InMemoryAgentJobStore::new();
        store
            .enqueue(test_job(now), now)
            .await
            .expect("seed saturated queue");
        let app = router_with_admission_policy(store, small_admission_policy(1));

        let response = app
            .oneshot(request(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment","priority":"standard"}"#,
            ))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let body: CreateAgentJobResponse =
            serde_json::from_slice(&response_body(response).await).expect("json body");
        assert_eq!(body.idempotency_outcome, IdempotencyOutcome::NewlyAdmitted);
        assert_eq!(
            body.admission_decision,
            Some(AdmissionDecisionKind::Delayed)
        );
        assert_eq!(body.admission_reason, Some(AdmissionReason::QueueSaturated));
        assert!(body.run_at > body.accepted_at);
    }

    #[tokio::test]
    async fn admission_policy_rejects_bulk_work_before_enqueue_when_queue_is_saturated() {
        let now = Utc::now();
        let mut store = InMemoryAgentJobStore::new();
        store
            .enqueue(test_job(now), now)
            .await
            .expect("seed saturated queue");
        let app = router_with_admission_policy(store, small_admission_policy(1));

        let response = app
            .clone()
            .oneshot(request(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment","priority":"bulk"}"#,
            ))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let body: serde_json::Value =
            serde_json::from_slice(&response_body(response).await).expect("json body");
        assert_eq!(body["error_code"], "admission_rejected");

        let metrics_response = app
            .oneshot(get_request("/metrics"))
            .await
            .expect("metrics response");
        let metrics: MetricsResponse =
            serde_json::from_slice(&response_body(metrics_response).await).expect("metrics json");
        assert_eq!(metrics.queue.pending, 1);
    }

    #[tokio::test]
    async fn invalid_priority_is_rejected_before_enqueue() {
        let app = router(InMemoryAgentJobStore::new()).expect("default admission policy");

        let response = app
            .oneshot(request(
                r#"{"kind":"incident_triage","instruction":"Analyze failed deployment","priority":"urgent"}"#,
            ))
            .await
            .expect("route response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&response_body(response).await).expect("json body");
        assert_eq!(body["error_code"], "admission_control_failed");
    }
}
