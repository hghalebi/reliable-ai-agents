create table scheduled_jobs (
  id uuid primary key,
  task_name text not null,
  status text not null check (
    status in ('pending', 'running', 'succeeded', 'failed', 'dead', 'cancelled')
  ),
  payload jsonb not null,
  attempts int not null default 0 check (attempts >= 0),
  max_attempts int not null default 5 check (max_attempts > 0),
  next_run_at timestamptz not null default now(),
  locked_by text,
  locked_until timestamptz,
  last_error text,
  idempotency_key text unique,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (jsonb_typeof(payload) = 'object'),
  check (
    (status = 'running' and locked_by is not null and locked_until is not null)
    or
    (status <> 'running' and locked_by is null and locked_until is null)
  )
);

create index scheduled_jobs_due_idx
  on scheduled_jobs (next_run_at, created_at)
  where status = 'pending';

create index scheduled_jobs_locked_until_idx
  on scheduled_jobs (locked_until)
  where status = 'running';

create table background_jobs (
  id uuid primary key,
  scheduled_job_id uuid not null unique references scheduled_jobs(id) on delete cascade,
  job_kind text not null,
  workflow_state text not null check (
    workflow_state in (
      'queued',
      'leased',
      'executing_agent',
      'waiting_for_human',
      'waiting_for_retry',
      'completed',
      'failed',
      'cancelled'
    )
  ),
  retry_state text not null check (
    retry_state in (
      'not_started',
      'retryable',
      'waiting_for_retry',
      'exhausted',
      'permanent_failure',
      'not_applicable'
    )
  ),
  attempts int not null default 0 check (attempts >= 0),
  max_attempts int not null default 5 check (max_attempts > 0),
  next_retry_at timestamptz,
  execution_deadline_at timestamptz,
  timeout_policy_name text not null default 'standard-agent:v1',
  timeout_action text not null default 'schedule_retry' check (
    timeout_action in ('schedule_retry', 'cancel_job', 'escalate_to_human', 'dead_letter')
  ),
  last_failure_class text,
  last_error text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (attempts <= max_attempts),
  check (updated_at >= created_at),
  check (btrim(job_kind) <> ''),
  check (btrim(timeout_policy_name) <> ''),
  check (last_failure_class is null or btrim(last_failure_class) <> ''),
  check (last_error is null or btrim(last_error) <> ''),
  check (
    (workflow_state in ('leased', 'executing_agent', 'waiting_for_human')
      and execution_deadline_at is not null)
    or
    (workflow_state not in ('leased', 'executing_agent', 'waiting_for_human')
      and execution_deadline_at is null)
  ),
  check (
    (retry_state = 'waiting_for_retry' and next_retry_at is not null)
    or
    (retry_state <> 'waiting_for_retry' and next_retry_at is null)
  ),
  check (
    (workflow_state in ('queued', 'leased', 'executing_agent', 'waiting_for_human')
      and retry_state in ('not_started', 'retryable'))
    or
    (workflow_state = 'waiting_for_retry'
      and retry_state = 'waiting_for_retry')
    or
    (workflow_state in ('completed', 'cancelled')
      and retry_state = 'not_applicable')
    or
    (workflow_state = 'failed'
      and retry_state in ('exhausted', 'permanent_failure'))
  ),
  check (
    (retry_state in ('retryable', 'waiting_for_retry', 'exhausted', 'permanent_failure')
      and last_failure_class is not null
      and last_error is not null
      and (
        retry_state <> 'waiting_for_retry'
        or attempts < max_attempts
      )
      and (
        retry_state <> 'exhausted'
        or attempts >= max_attempts
      ))
    or
    (retry_state = 'not_started'
      and last_failure_class is null
      and last_error is null)
    or
    (workflow_state = 'completed'
      and last_failure_class is null
      and last_error is null)
    or
    (workflow_state = 'cancelled'
      and retry_state = 'not_applicable')
  )
);

create index background_jobs_workflow_state_idx
  on background_jobs (workflow_state, updated_at);

create index background_jobs_retry_state_idx
  on background_jobs (retry_state, next_retry_at);

create table agent_runs (
  id uuid primary key,
  scheduled_job_id uuid references scheduled_jobs(id) on delete set null,
  background_job_id uuid references background_jobs(id) on delete set null,
  agent_name text not null,
  lifecycle_status text not null check (
    lifecycle_status in (
      'planning',
      'running',
      'waiting_for_human',
      'completed',
      'failed',
      'cancelled'
    )
  ),
  prompt_version text not null,
  model_version text not null,
  trace_id text not null check (
    trace_id ~ '^[0-9A-Fa-f]{32}$'
    and trace_id <> repeat('0', 32)
  ),
  deadline_at timestamptz,
  timeout_policy_name text not null default 'standard-agent:v1',
  timeout_action text not null default 'schedule_retry' check (
    timeout_action in ('schedule_retry', 'cancel_job', 'escalate_to_human', 'dead_letter')
  ),
  started_at timestamptz not null default now(),
  finished_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (scheduled_job_id is not null or background_job_id is not null),
  check (btrim(agent_name) <> ''),
  check (btrim(prompt_version) <> ''),
  check (btrim(model_version) <> ''),
  check (btrim(timeout_policy_name) <> ''),
  check (deadline_at is null or deadline_at > started_at),
  check (finished_at is null or finished_at >= started_at),
  check (
    (lifecycle_status in ('completed', 'failed', 'cancelled') and finished_at is not null)
    or
    (lifecycle_status not in ('completed', 'failed', 'cancelled') and finished_at is null)
  )
);

create index agent_runs_job_idx
  on agent_runs (scheduled_job_id, started_at);

create index agent_runs_status_idx
  on agent_runs (lifecycle_status, started_at);

create index agent_runs_trace_idx
  on agent_runs (trace_id, started_at);

create table cancellation_requests (
  id uuid primary key,
  job_id uuid not null references scheduled_jobs(id) on delete cascade,
  run_id uuid references agent_runs(id) on delete set null,
  status text not null check (
    status in ('requested', 'applied', 'ignored_terminal', 'expired')
  ),
  requested_by text not null check (btrim(requested_by) <> ''),
  source text not null check (source in ('user', 'operator', 'system', 'policy')),
  mode text not null check (mode in ('graceful', 'immediate', 'after_current_step')),
  reason text not null check (btrim(reason) <> ''),
  requested_at timestamptz not null default now(),
  expires_at timestamptz,
  applied_at timestamptz,
  observed_job_status text check (
    observed_job_status in ('pending', 'running', 'succeeded', 'failed', 'dead', 'cancelled')
  ),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (expires_at is null or expires_at > requested_at),
  check (
    (status = 'requested' and applied_at is null and observed_job_status is null)
    or
    (status = 'applied'
      and applied_at is not null
      and observed_job_status in ('pending', 'running'))
    or
    (status = 'ignored_terminal'
      and applied_at is not null
      and observed_job_status in ('succeeded', 'dead', 'cancelled'))
    or
    (status = 'expired'
      and expires_at is not null
      and applied_at is not null
      and observed_job_status is null)
  )
);

create index cancellation_requests_pending_idx
  on cancellation_requests (requested_at)
  where status = 'requested';

create index cancellation_requests_job_idx
  on cancellation_requests (job_id, requested_at);

create table agent_handoffs (
  id uuid primary key,
  source_run_id uuid not null references agent_runs(id) on delete cascade,
  from_agent text not null check (btrim(from_agent) <> ''),
  to_agent text not null check (btrim(to_agent) <> ''),
  reason text not null check (btrim(reason) <> ''),
  payload jsonb not null check (jsonb_typeof(payload) = 'object'),
  status text not null check (
    status in ('requested', 'accepted', 'rejected', 'expired', 'cancelled')
  ),
  idempotency_key text not null unique check (btrim(idempotency_key) <> ''),
  target_job_id uuid references scheduled_jobs(id) on delete set null,
  decision_reason text,
  requested_at timestamptz not null default now(),
  decided_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (from_agent <> to_agent),
  check (
    (status = 'requested' and target_job_id is null and decision_reason is null and decided_at is null)
    or
    (status = 'accepted' and target_job_id is not null and decided_at is not null)
    or
    (status in ('rejected', 'expired', 'cancelled')
      and target_job_id is null
      and decision_reason is not null
      and decided_at is not null)
  )
);

create index agent_handoffs_source_idx
  on agent_handoffs (source_run_id, requested_at);

create index agent_handoffs_target_status_idx
  on agent_handoffs (to_agent, status, requested_at);

create table agent_steps (
  id uuid primary key,
  run_id uuid not null references agent_runs(id) on delete cascade,
  step_index int not null check (step_index >= 0),
  step_kind text not null check (
    step_kind in ('plan', 'model_call', 'tool_call', 'approval_gate', 'finalize')
  ),
  status text not null check (
    status in ('pending', 'running', 'succeeded', 'failed', 'skipped')
  ),
  input_ref text,
  output_ref text,
  error text,
  started_at timestamptz,
  completed_at timestamptz,
  created_at timestamptz not null default now(),
  check (input_ref is null or btrim(input_ref) <> ''),
  check (output_ref is null or btrim(output_ref) <> ''),
  check (error is null or btrim(error) <> ''),
  check (started_at is null or started_at >= created_at),
  check (completed_at is null or completed_at >= created_at),
  check (completed_at is null or started_at is null or completed_at >= started_at),
  check (
    (status = 'pending'
      and started_at is null
      and completed_at is null
      and output_ref is null
      and error is null)
    or
    (status = 'running'
      and started_at is not null
      and completed_at is null
      and output_ref is null
      and error is null)
    or
    (status = 'succeeded'
      and started_at is not null
      and completed_at is not null
      and output_ref is not null
      and error is null)
    or
    (status = 'failed'
      and started_at is not null
      and completed_at is not null
      and output_ref is null
      and btrim(error) <> '')
    or
    (status = 'skipped'
      and started_at is null
      and completed_at is not null
      and output_ref is null
      and btrim(error) <> '')
  ),
  unique (run_id, step_index)
);

create index agent_steps_run_idx
  on agent_steps (run_id, step_index);

create table tool_calls (
  id uuid primary key,
  run_id uuid not null references agent_runs(id) on delete cascade,
  step_id uuid references agent_steps(id) on delete set null,
  tool_name text not null,
  tool_version text not null,
  status text not null check (
    status in ('requested', 'validated', 'executed', 'failed', 'rejected')
  ),
  idempotency_key text not null unique,
  input jsonb not null,
  output jsonb,
  error text,
  started_at timestamptz,
  completed_at timestamptz,
  created_at timestamptz not null default now(),
  check (btrim(tool_name) <> ''),
  check (btrim(tool_version) <> ''),
  check (btrim(idempotency_key) <> ''),
  check (jsonb_typeof(input) = 'object'),
  check (output is null or jsonb_typeof(output) = 'object'),
  check (started_at is null or started_at >= created_at),
  check (completed_at is null or completed_at >= created_at),
  check (completed_at is null or started_at is null or completed_at >= started_at),
  check (
    (status = 'executed'
      and started_at is not null
      and completed_at is not null
      and output is not null
      and error is null)
    or
    (status = 'failed'
      and started_at is not null
      and completed_at is not null
      and output is null
      and btrim(error) <> '')
    or
    (status = 'rejected'
      and started_at is null
      and completed_at is not null
      and output is null
      and btrim(error) <> '')
    or
    (status in ('requested', 'validated')
      and output is null
      and error is null
      and completed_at is null)
  )
);

create index tool_calls_run_idx
  on tool_calls (run_id, created_at);

create index tool_calls_status_idx
  on tool_calls (status, created_at);

create table failure_history (
  id bigserial primary key,
  scheduled_job_id uuid not null references scheduled_jobs(id) on delete cascade,
  background_job_id uuid references background_jobs(id) on delete set null,
  run_id uuid references agent_runs(id) on delete set null,
  step_id uuid references agent_steps(id) on delete set null,
  tool_call_id uuid references tool_calls(id) on delete set null,
  failure_source text not null check (
    failure_source in (
      'worker',
      'model_provider',
      'model_output',
      'tool',
      'policy',
      'sandbox',
      'timeout',
      'database'
    )
  ),
  failure_class text not null,
  failure_message text not null,
  workflow_state text not null check (
    workflow_state in (
      'queued',
      'leased',
      'executing_agent',
      'waiting_for_human',
      'waiting_for_retry',
      'completed',
      'failed',
      'cancelled'
    )
  ),
  retry_state text not null check (
    retry_state in (
      'not_started',
      'retryable',
      'waiting_for_retry',
      'exhausted',
      'permanent_failure',
      'not_applicable'
    )
  ),
  failure_outcome text not null check (
    failure_outcome in (
      'retry_scheduled',
      'dead_lettered',
      'permanent_failure',
      'escalated_to_human',
      'cancelled'
    )
  ),
  attempt int not null check (attempt >= 0),
  max_attempts int not null check (max_attempts > 0),
  next_retry_at timestamptz,
  trace_id text check (
    trace_id is null
    or (
      trace_id ~ '^[0-9A-Fa-f]{32}$'
      and trace_id <> repeat('0', 32)
    )
  ),
  span_id text check (
    span_id is null
    or (
      span_id ~ '^[0-9A-Fa-f]{16}$'
      and span_id <> repeat('0', 16)
    )
  ),
  occurred_at timestamptz not null,
  recorded_at timestamptz not null default now(),
  check (btrim(failure_class) <> ''),
  check (btrim(failure_message) <> ''),
  check (attempt <= max_attempts),
  check (recorded_at >= occurred_at),
  check (span_id is null or trace_id is not null),
  check (
    (failure_outcome = 'retry_scheduled'
      and workflow_state = 'waiting_for_retry'
      and retry_state = 'waiting_for_retry'
      and next_retry_at is not null
      and next_retry_at > occurred_at)
    or
    (failure_outcome = 'dead_lettered'
      and workflow_state = 'failed'
      and retry_state = 'exhausted'
      and next_retry_at is null
      and attempt >= max_attempts)
    or
    (failure_outcome = 'permanent_failure'
      and workflow_state = 'failed'
      and retry_state = 'permanent_failure'
      and next_retry_at is null)
    or
    (failure_outcome = 'escalated_to_human'
      and workflow_state = 'waiting_for_human'
      and retry_state in ('not_started', 'retryable')
      and next_retry_at is null)
    or
    (failure_outcome = 'cancelled'
      and workflow_state = 'cancelled'
      and retry_state = 'not_applicable'
      and next_retry_at is null)
  )
);

create index failure_history_job_idx
  on failure_history (scheduled_job_id, occurred_at desc, id desc);

create index failure_history_run_idx
  on failure_history (run_id, occurred_at desc, id desc)
  where run_id is not null;

create index failure_history_tool_call_idx
  on failure_history (tool_call_id, occurred_at desc, id desc)
  where tool_call_id is not null;

create index failure_history_class_idx
  on failure_history (failure_class, failure_outcome, occurred_at desc);

create table authorization_events (
  id uuid primary key,
  run_id uuid not null references agent_runs(id) on delete cascade,
  actor_id text not null,
  actor_tenant_key text not null,
  requested_tenant_key text not null,
  tool_name text not null,
  permission text not null check (
    permission in (
      'pause_job_kind',
      'read_tenant_data',
      'write_memory',
      'send_external_message'
    )
  ),
  decision text not null check (
    decision in ('authorized', 'requires_approval', 'denied')
  ),
  reason text,
  policy_version text not null,
  decided_at timestamptz not null default now(),
  check (
    (decision = 'authorized')
    or
    (decision <> 'authorized' and reason is not null)
  )
);

create index authorization_events_run_idx
  on authorization_events (run_id, decided_at);

create index authorization_events_actor_idx
  on authorization_events (actor_id, actor_tenant_key, decided_at desc);

create index authorization_events_denied_idx
  on authorization_events (decided_at desc)
  where decision in ('denied', 'requires_approval');

create table sandbox_events (
  id uuid primary key,
  run_id uuid not null references agent_runs(id) on delete cascade,
  tool_name text not null,
  network_destination text,
  filesystem_access text not null check (
    filesystem_access in ('none', 'read_scratch', 'write_scratch')
  ),
  filesystem_path text,
  secret_access text not null check (
    secret_access in ('none', 'tool_runtime_only', 'model_visible')
  ),
  decision text not null check (
    decision in ('allowed', 'denied')
  ),
  reason text,
  policy_version text not null,
  decided_at timestamptz not null default now(),
  check (
    (decision = 'allowed' and reason is null)
    or
    (decision = 'denied' and reason is not null)
  ),
  check (
    (filesystem_access = 'none' and filesystem_path is null)
    or
    (filesystem_access <> 'none' and filesystem_path is not null)
  ),
  check (
    network_destination is null
    or position('://' in network_destination) = 0
  ),
  check (
    network_destination is null
    or position('*' in network_destination) = 0
  )
);

create index sandbox_events_run_idx
  on sandbox_events (run_id, decided_at);

create index sandbox_events_denied_idx
  on sandbox_events (decided_at desc)
  where decision = 'denied';

create table credential_assets (
  id uuid primary key,
  secret_ref text not null unique,
  credential_kind text not null check (
    credential_kind in (
      'provider_api_key',
      'database_url',
      'operator_token',
      'webhook_secret',
      'service_account_key',
      'ci_secret',
      'encryption_key'
    )
  ),
  owner text not null,
  storage_location text not null check (
    storage_location in (
      'environment',
      'platform_secret',
      'external_secret_manager',
      'database_reference',
      'ci_secret_store'
    )
  ),
  status text not null check (
    status in ('active', 'rotation_due', 'rotating', 'revoked', 'compromised')
  ),
  rotation_interval_days int not null check (
    rotation_interval_days > 0 and rotation_interval_days <= 730
  ),
  last_rotated_at timestamptz,
  next_rotation_due_at timestamptz not null,
  last_verified_at timestamptz,
  exposure_reported_at timestamptz,
  revoked_at timestamptz,
  policy_version text not null,
  evidence jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (btrim(secret_ref) <> ''),
  check (btrim(owner) <> ''),
  check (btrim(policy_version) <> ''),
  check (jsonb_typeof(evidence) = 'object'),
  check (last_rotated_at is null or next_rotation_due_at > last_rotated_at),
  check (last_verified_at is null or last_rotated_at is null or last_verified_at >= last_rotated_at),
  check (revoked_at is null or exposure_reported_at is null or revoked_at >= exposure_reported_at),
  check (updated_at >= created_at),
  check (
    (status = 'compromised' and exposure_reported_at is not null)
    or
    (status <> 'compromised')
  ),
  check (
    (status = 'revoked' and revoked_at is not null)
    or
    (status <> 'revoked')
  )
);

create index credential_assets_due_idx
  on credential_assets (next_rotation_due_at, credential_kind)
  where status in ('active', 'rotation_due', 'rotating', 'compromised');

create index credential_assets_status_idx
  on credential_assets (status, next_rotation_due_at);

create index credential_assets_owner_idx
  on credential_assets (owner, credential_kind, updated_at desc);

create table side_effect_receipts (
  id uuid primary key,
  tool_call_id uuid not null references tool_calls(id) on delete cascade,
  idempotency_key text not null unique,
  external_system text not null,
  external_correlation_id text,
  effect_kind text not null,
  receipt jsonb not null,
  recorded_at timestamptz not null default now(),
  check (jsonb_typeof(receipt) = 'object')
);

create index side_effect_receipts_tool_call_idx
  on side_effect_receipts (tool_call_id, recorded_at);

create index side_effect_receipts_external_idx
  on side_effect_receipts (external_system, external_correlation_id);

create table outbox_events (
  id uuid primary key,
  event_kind text not null,
  aggregate_id text not null,
  idempotency_key text not null unique,
  payload jsonb not null,
  status text not null check (
    status in ('pending', 'publishing', 'published', 'failed')
  ),
  attempts int not null default 0 check (attempts >= 0),
  max_attempts int not null default 5 check (max_attempts > 0),
  next_attempt_at timestamptz not null default now(),
  locked_by text,
  locked_until timestamptz,
  last_error text,
  occurred_at timestamptz not null default now(),
  published_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (jsonb_typeof(payload) = 'object'),
  check (
    (status = 'publishing' and locked_by is not null and locked_until is not null)
    or
    (status <> 'publishing' and locked_by is null and locked_until is null)
  ),
  check (
    (status = 'published' and published_at is not null)
    or
    (status <> 'published')
  ),
  check (
    (status = 'failed' and last_error is not null)
    or
    (status <> 'failed')
  )
);

create index outbox_events_due_idx
  on outbox_events (next_attempt_at, occurred_at)
  where status = 'pending';

create index outbox_events_locked_until_idx
  on outbox_events (locked_until)
  where status = 'publishing';

-- ANCHOR: kafka_optional_scaling_tables
create table kafka_publish_receipts (
  id uuid primary key,
  outbox_event_id uuid not null unique references outbox_events(id) on delete cascade,
  event_kind text not null,
  aggregate_id text not null,
  schema_version int not null check (schema_version > 0),
  topic text not null,
  partition_id int not null check (partition_id >= 0),
  record_offset bigint not null check (record_offset >= 0),
  trace_id text not null check (
    trace_id ~ '^[0-9A-Fa-f]{32}$'
    and trace_id <> repeat('0', 32)
  ),
  published_at timestamptz not null,
  recorded_at timestamptz not null default now(),
  check (btrim(event_kind) <> ''),
  check (btrim(aggregate_id) <> ''),
  check (btrim(topic) <> ''),
  unique (topic, partition_id, record_offset)
);

create index kafka_publish_receipts_topic_offset_idx
  on kafka_publish_receipts (topic, partition_id, record_offset);

create index kafka_publish_receipts_trace_idx
  on kafka_publish_receipts (trace_id, published_at desc);

create table kafka_consumer_receipts (
  id uuid primary key,
  publish_receipt_id uuid not null references kafka_publish_receipts(id) on delete cascade,
  outbox_event_id uuid not null references outbox_events(id) on delete cascade,
  consumer_group text not null,
  consumer_name text not null,
  topic text not null,
  partition_id int not null check (partition_id >= 0),
  record_offset bigint not null check (record_offset >= 0),
  idempotency_key text not null unique,
  status text not null check (
    status in ('completed', 'rejected_poison_event', 'failed_retryable')
  ),
  error text,
  processed_at timestamptz not null,
  recorded_at timestamptz not null default now(),
  check (btrim(consumer_group) <> ''),
  check (btrim(consumer_name) <> ''),
  check (btrim(topic) <> ''),
  check (btrim(idempotency_key) <> ''),
  check (
    (status = 'failed_retryable' and error is not null and btrim(error) <> '')
    or
    (status <> 'failed_retryable' and error is null)
  ),
  unique (consumer_group, outbox_event_id)
);

create index kafka_consumer_receipts_group_status_idx
  on kafka_consumer_receipts (consumer_group, status, processed_at desc);

create index kafka_consumer_receipts_record_idx
  on kafka_consumer_receipts (topic, partition_id, record_offset);
-- ANCHOR_END: kafka_optional_scaling_tables

create table audit_events (
  id bigserial primary key,
  run_id uuid references agent_runs(id) on delete set null,
  actor_type text not null check (actor_type in ('user', 'worker', 'model', 'system')),
  actor_id text not null,
  action text not null,
  subject text not null,
  event_data jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  check (jsonb_typeof(event_data) = 'object')
);

create index audit_events_run_idx
  on audit_events (run_id, created_at, id);

create table operation_events (
  id bigserial primary key,
  job_id uuid references scheduled_jobs(id) on delete set null,
  run_id uuid references agent_runs(id) on delete set null,
  trace_id text not null check (
    trace_id ~ '^[0-9A-Fa-f]{32}$'
    and trace_id <> repeat('0', 32)
  ),
  span_id text check (
    span_id is null
    or (
      span_id ~ '^[0-9A-Fa-f]{16}$'
      and span_id <> repeat('0', 16)
    )
  ),
  event_type text not null,
  severity text not null check (severity in ('debug', 'info', 'warn', 'error')),
  message text not null,
  data jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  check (jsonb_typeof(data) = 'object')
);

create index operation_events_job_idx
  on operation_events (job_id, created_at, id);

create index operation_events_run_idx
  on operation_events (run_id, created_at, id);

create index operation_events_trace_idx
  on operation_events (trace_id, created_at, id);

-- ANCHOR: temporal_optional_scaling_tables
create table temporal_workflow_links (
  id uuid primary key,
  scheduled_job_id uuid not null references scheduled_jobs(id) on delete cascade,
  agent_run_id uuid not null references agent_runs(id) on delete cascade,
  workflow_type text not null,
  workflow_execution_ref text not null unique,
  task_queue text not null,
  idempotency_key text not null unique,
  trace_id text not null check (
    trace_id ~ '^[0-9A-Fa-f]{32}$'
    and trace_id <> repeat('0', 32)
  ),
  workflow_status text not null check (
    workflow_status in (
      'started',
      'running',
      'completed',
      'failed',
      'cancelled',
      'reconciliation_needed'
    )
  ),
  started_at timestamptz not null,
  completed_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (btrim(workflow_type) <> ''),
  check (btrim(workflow_execution_ref) <> ''),
  check (btrim(task_queue) <> ''),
  check (btrim(idempotency_key) <> ''),
  check (completed_at is null or completed_at >= started_at),
  check (
    (workflow_status in ('completed', 'failed', 'cancelled') and completed_at is not null)
    or
    (workflow_status not in ('completed', 'failed', 'cancelled') and completed_at is null)
  )
);

create index temporal_workflow_links_run_idx
  on temporal_workflow_links (agent_run_id, started_at desc);

create index temporal_workflow_links_trace_idx
  on temporal_workflow_links (trace_id, started_at desc);

create index temporal_workflow_links_status_idx
  on temporal_workflow_links (workflow_status, updated_at desc);

create table temporal_activity_receipts (
  id uuid primary key,
  workflow_link_id uuid not null references temporal_workflow_links(id) on delete cascade,
  activity_execution_ref text not null,
  tool_call_id uuid not null references tool_calls(id) on delete cascade,
  idempotency_key text not null unique,
  operation_event_id bigint not null references operation_events(id) on delete restrict,
  recorded_at timestamptz not null default now(),
  check (btrim(activity_execution_ref) <> ''),
  check (btrim(idempotency_key) <> ''),
  unique (workflow_link_id, activity_execution_ref)
);

create index temporal_activity_receipts_tool_call_idx
  on temporal_activity_receipts (tool_call_id, recorded_at desc);

create index temporal_activity_receipts_operation_event_idx
  on temporal_activity_receipts (operation_event_id);
-- ANCHOR_END: temporal_optional_scaling_tables

create table provider_usage_events (
  id uuid primary key,
  run_id uuid not null references agent_runs(id) on delete cascade,
  tenant_key text not null,
  job_kind text not null,
  provider_name text not null,
  model_route text not null,
  provider_status text not null check (
    provider_status in ('succeeded', 'rate_limited', 'timeout', 'failed')
  ),
  prompt_tokens int not null check (prompt_tokens >= 0),
  completion_tokens int not null check (completion_tokens >= 0),
  total_tokens int not null check (
    total_tokens >= 0 and total_tokens = prompt_tokens + completion_tokens
  ),
  cost_micros_usd bigint not null check (cost_micros_usd >= 0),
  latency_ms bigint not null check (latency_ms >= 0),
  recorded_at timestamptz not null default now()
);

create index provider_usage_events_tenant_time_idx
  on provider_usage_events (tenant_key, recorded_at desc);

create index provider_usage_events_job_kind_time_idx
  on provider_usage_events (job_kind, recorded_at desc);

create index provider_usage_events_status_time_idx
  on provider_usage_events (provider_status, recorded_at desc);

create table human_approval_requests (
  id uuid primary key,
  run_id uuid not null references agent_runs(id) on delete cascade,
  status text not null check (
    status in ('requested', 'approved', 'rejected', 'expired', 'cancelled')
  ),
  requested_by text not null,
  decided_by text,
  reason text,
  requested_at timestamptz not null default now(),
  decided_at timestamptz,
  expires_at timestamptz,
  check (
    (status in ('approved', 'rejected') and decided_by is not null and decided_at is not null)
    or
    (status not in ('approved', 'rejected'))
  )
);

create index human_approval_requests_status_idx
  on human_approval_requests (status, requested_at);

create table human_escalations (
  id uuid primary key,
  job_id uuid references scheduled_jobs(id) on delete set null,
  run_id uuid references agent_runs(id) on delete set null,
  escalation_kind text not null check (
    escalation_kind in (
      'deadline_breach',
      'repeated_failure',
      'security_signal',
      'approval_timeout',
      'compatibility_risk',
      'operator_review'
    )
  ),
  severity text not null check (severity in ('review', 'ticket', 'page')),
  status text not null check (status in ('open', 'acknowledged', 'resolved', 'cancelled')),
  reason text not null check (btrim(reason) <> ''),
  assigned_to text,
  created_at timestamptz not null default now(),
  acknowledged_at timestamptz,
  resolved_at timestamptz,
  check (job_id is not null or run_id is not null),
  check (acknowledged_at is null or acknowledged_at >= created_at),
  check (resolved_at is null or acknowledged_at is not null),
  check (resolved_at is null or resolved_at >= acknowledged_at),
  check (
    (status = 'open' and assigned_to is null and acknowledged_at is null and resolved_at is null)
    or
    (status = 'acknowledged'
      and assigned_to is not null
      and acknowledged_at is not null
      and resolved_at is null)
    or
    (status in ('resolved', 'cancelled')
      and assigned_to is not null
      and acknowledged_at is not null
      and resolved_at is not null)
  )
);

create index human_escalations_open_idx
  on human_escalations (severity, created_at)
  where status in ('open', 'acknowledged');

create index human_escalations_job_idx
  on human_escalations (job_id, created_at);

create table compensation_actions (
  id uuid primary key,
  receipt_id uuid not null references side_effect_receipts(id) on delete restrict,
  compensation_kind text not null check (btrim(compensation_kind) <> ''),
  reason text not null check (btrim(reason) <> ''),
  payload jsonb not null check (jsonb_typeof(payload) = 'object'),
  status text not null check (
    status in ('requested', 'approved', 'executing', 'succeeded', 'failed', 'cancelled')
  ),
  approval_request_id uuid references human_approval_requests(id) on delete set null,
  idempotency_key text not null unique check (btrim(idempotency_key) <> ''),
  attempts int not null default 0 check (attempts >= 0),
  max_attempts int not null default 3 check (max_attempts > 0),
  next_attempt_at timestamptz not null default now(),
  locked_by text,
  locked_until timestamptz,
  last_error text,
  requested_at timestamptz not null default now(),
  approved_at timestamptz,
  completed_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (
    (status in ('approved', 'executing', 'succeeded', 'failed')
      and approval_request_id is not null
      and approved_at is not null)
    or
    (status in ('requested', 'cancelled'))
  ),
  check (
    (status = 'executing' and locked_by is not null and locked_until is not null)
    or
    (status <> 'executing' and locked_by is null and locked_until is null)
  ),
  check (
    (status in ('succeeded', 'failed', 'cancelled') and completed_at is not null)
    or
    (status not in ('succeeded', 'failed', 'cancelled'))
  ),
  check (
    (status = 'failed' and last_error is not null)
    or
    (status <> 'failed')
  )
);

create index compensation_actions_status_idx
  on compensation_actions (status, next_attempt_at);

create index compensation_actions_receipt_idx
  on compensation_actions (receipt_id, created_at);

create table evaluation_runs (
  id uuid primary key,
  run_id uuid references agent_runs(id) on delete set null,
  dataset_version text not null,
  evaluator_version text not null,
  prompt_version text not null,
  model_version text not null,
  tool_version text not null,
  policy_version text not null,
  status text not null check (status in ('queued', 'running', 'passed', 'failed')),
  score numeric check (score is null or (score >= 0 and score <= 1)),
  report jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  completed_at timestamptz,
  check (btrim(dataset_version) <> ''),
  check (btrim(evaluator_version) <> ''),
  check (btrim(prompt_version) <> ''),
  check (btrim(model_version) <> ''),
  check (btrim(tool_version) <> ''),
  check (btrim(policy_version) <> ''),
  check (jsonb_typeof(report) = 'object'),
  check (completed_at is null or completed_at >= created_at),
  check (
    (status in ('passed', 'failed') and score is not null and completed_at is not null)
    or
    (status in ('queued', 'running') and score is null and completed_at is null)
  )
);

create index evaluation_runs_run_idx
  on evaluation_runs (run_id, created_at);

create index evaluation_runs_version_idx
  on evaluation_runs (prompt_version, model_version, tool_version, policy_version, created_at desc);

create table agent_memory_records (
  id uuid primary key,
  run_id uuid references agent_runs(id) on delete set null,
  memory_kind text not null,
  memory_scope text not null,
  source text not null,
  confidence numeric not null check (confidence >= 0 and confidence <= 1),
  memory_horizon text not null check (memory_horizon in ('short_term', 'long_term')),
  retention_policy text not null,
  content jsonb not null,
  embedding_ref text,
  created_at timestamptz not null default now(),
  last_used_at timestamptz,
  check (jsonb_typeof(content) = 'object'),
  check (
    (memory_horizon = 'short_term' and retention_policy in ('ephemeral', 'session'))
    or
    (memory_horizon = 'long_term' and retention_policy in ('durable', 'audit'))
  )
);

create index agent_memory_records_scope_idx
  on agent_memory_records (memory_scope, memory_kind, created_at);

-- ANCHOR: data_protection_requests
create table data_protection_requests (
  id uuid primary key,
  request_kind text not null check (
    request_kind in ('redaction', 'erasure', 'export', 'retention_review')
  ),
  surface text not null check (
    surface in (
      'agent_jobs',
      'scheduled_jobs',
      'background_jobs',
      'agent_runs',
      'tool_calls',
      'audit_events',
      'operation_events',
      'provider_usage_events',
      'human_approval_requests',
      'side_effect_receipts',
      'evaluation_runs',
      'agent_memory_records'
    )
  ),
  subject_ref text not null,
  status text not null check (
    status in ('requested', 'approved', 'applied', 'rejected', 'expired')
  ),
  requested_by text not null,
  reason text not null,
  policy_version text not null,
  evidence jsonb not null default '{}'::jsonb,
  requested_at timestamptz not null default now(),
  due_at timestamptz not null,
  completed_at timestamptz,
  check (btrim(subject_ref) <> ''),
  check (btrim(requested_by) <> ''),
  check (btrim(reason) <> ''),
  check (btrim(policy_version) <> ''),
  check (jsonb_typeof(evidence) = 'object'),
  check (due_at >= requested_at),
  check (completed_at is null or completed_at >= requested_at),
  check (
    (status in ('applied', 'rejected', 'expired') and completed_at is not null)
    or
    (status in ('requested', 'approved') and completed_at is null)
  )
);

create index data_protection_requests_status_due_idx
  on data_protection_requests (status, due_at);

create index data_protection_requests_surface_subject_idx
  on data_protection_requests (surface, subject_ref, requested_at desc);
-- ANCHOR_END: data_protection_requests

create table restore_drill_runs (
  id uuid primary key,
  drill_name text not null,
  status text not null check (status in ('planned', 'running', 'passed', 'failed')),
  backup_source text not null,
  restore_target text not null,
  rpo_seconds bigint not null check (rpo_seconds >= 0),
  rto_seconds bigint not null check (rto_seconds >= 0),
  restored_jobs bigint not null check (restored_jobs >= 0),
  restored_events bigint not null check (restored_events >= 0),
  restored_receipts bigint not null check (restored_receipts >= 0),
  replay_safe_jobs bigint not null check (replay_safe_jobs >= 0),
  quarantined_jobs bigint not null check (quarantined_jobs >= 0),
  started_at timestamptz not null,
  completed_at timestamptz,
  operator_signoff text,
  created_at timestamptz not null default now(),
  check (
    (status in ('passed', 'failed') and completed_at is not null and operator_signoff is not null)
    or
    (status not in ('passed', 'failed'))
  )
);

create index restore_drill_runs_status_idx
  on restore_drill_runs (status, started_at desc);

-- ANCHOR: failure_drill_runs
create table failure_drill_runs (
  id uuid primary key,
  drill_name text not null,
  scenario text not null,
  environment text not null check (environment in ('local', 'staging', 'production')),
  status text not null check (status in ('planned', 'running', 'passed', 'failed', 'aborted')),
  owner text not null,
  hypothesis text not null,
  blast_radius text not null,
  injection text not null,
  rollback_action text not null,
  scheduled_job_id uuid references scheduled_jobs(id) on delete set null,
  run_id uuid references agent_runs(id) on delete set null,
  trace_id text check (
    trace_id is null
    or (
      trace_id ~ '^[0-9A-Fa-f]{32}$'
      and trace_id <> repeat('0', 32)
    )
  ),
  required_evidence_count int not null check (required_evidence_count > 0),
  observed_evidence_count int not null default 0 check (observed_evidence_count >= 0),
  observed_evidence jsonb not null default '[]'::jsonb,
  started_at timestamptz not null,
  completed_at timestamptz,
  decision_reason text,
  operator_signoff text,
  created_at timestamptz not null default now(),
  check (btrim(drill_name) <> ''),
  check (btrim(scenario) <> ''),
  check (btrim(owner) <> ''),
  check (btrim(hypothesis) <> ''),
  check (btrim(blast_radius) <> ''),
  check (btrim(injection) <> ''),
  check (btrim(rollback_action) <> ''),
  check (decision_reason is null or btrim(decision_reason) <> ''),
  check (operator_signoff is null or btrim(operator_signoff) <> ''),
  check (jsonb_typeof(observed_evidence) = 'array'),
  check (completed_at is null or completed_at >= started_at),
  check (
    (status in ('passed', 'failed', 'aborted')
      and completed_at is not null
      and decision_reason is not null
      and operator_signoff is not null)
    or
    (status in ('planned', 'running')
      and completed_at is null
      and decision_reason is null
      and operator_signoff is null)
  ),
  check (
    status <> 'passed'
    or observed_evidence_count >= required_evidence_count
  )
);

create index failure_drill_runs_status_idx
  on failure_drill_runs (status, started_at desc);

create index failure_drill_runs_scenario_idx
  on failure_drill_runs (scenario, environment, started_at desc);
-- ANCHOR_END: failure_drill_runs

-- ANCHOR: schema_migration_runs
create table schema_migration_runs (
  id uuid primary key,
  migration_name text not null,
  phase text not null check (phase in ('expand', 'backfill', 'contract')),
  status text not null check (status in ('planned', 'running', 'passed', 'failed', 'blocked')),
  target_surface text not null,
  target_version text not null,
  compatible_from_payload_schema_version int not null check (compatible_from_payload_schema_version > 0),
  compatible_through_payload_schema_version int not null check (compatible_through_payload_schema_version > 0),
  rows_examined bigint not null default 0 check (rows_examined >= 0),
  rows_changed bigint not null default 0 check (rows_changed >= 0),
  compatibility_query_name text not null,
  rollback_plan text not null,
  started_at timestamptz not null,
  completed_at timestamptz,
  operator_signoff text,
  created_at timestamptz not null default now(),
  check (btrim(migration_name) <> ''),
  check (btrim(target_surface) <> ''),
  check (btrim(target_version) <> ''),
  check (btrim(compatibility_query_name) <> ''),
  check (btrim(rollback_plan) <> ''),
  check (operator_signoff is null or btrim(operator_signoff) <> ''),
  check (compatible_from_payload_schema_version <= compatible_through_payload_schema_version),
  check (rows_changed <= rows_examined),
  check (completed_at is null or completed_at >= started_at),
  check (
    (status in ('passed', 'failed', 'blocked')
      and completed_at is not null
      and operator_signoff is not null)
    or
    (status in ('planned', 'running')
      and completed_at is null
      and operator_signoff is null)
  )
);

create index schema_migration_runs_status_idx
  on schema_migration_runs (status, started_at desc);

create index schema_migration_runs_surface_idx
  on schema_migration_runs (target_surface, target_version, started_at desc);
-- ANCHOR_END: schema_migration_runs

-- ANCHOR: release_gate_runs
create table release_gate_runs (
  id uuid primary key,
  candidate_id uuid not null,
  gate_name text not null,
  job_kind text not null,
  release_reason text not null,
  risk text not null check (risk in ('low', 'high')),
  decision text not null check (decision in ('promote', 'canary_only', 'block')),
  prompt_version text not null,
  model_version text not null,
  tool_version text not null,
  policy_version text not null,
  worker_build_id text not null,
  payload_schema_version int not null check (payload_schema_version > 0),
  evaluation_run_id uuid not null references evaluation_runs(id) on delete restrict,
  schema_migration_run_id uuid references schema_migration_runs(id) on delete set null,
  approval_request_id uuid references human_approval_requests(id) on delete set null,
  slo_decision text not null check (
    slo_decision in ('within_budget', 'no_traffic', 'budget_exhausted')
  ),
  compatibility_decision text not null check (
    compatibility_decision in ('process', 'quarantine')
  ),
  blockers jsonb not null default '[]'::jsonb,
  canary_percent int not null default 0 check (canary_percent >= 0 and canary_percent <= 100),
  rollback_plan text not null,
  evaluated_by text not null,
  operator_signoff text not null,
  evaluated_at timestamptz not null default now(),
  created_at timestamptz not null default now(),
  check (btrim(gate_name) <> ''),
  check (btrim(job_kind) <> ''),
  check (btrim(release_reason) <> ''),
  check (btrim(prompt_version) <> ''),
  check (btrim(model_version) <> ''),
  check (btrim(tool_version) <> ''),
  check (btrim(policy_version) <> ''),
  check (btrim(worker_build_id) <> ''),
  check (btrim(rollback_plan) <> ''),
  check (btrim(evaluated_by) <> ''),
  check (btrim(operator_signoff) <> ''),
  check (jsonb_typeof(blockers) = 'array'),
  check (
    (decision = 'promote' and jsonb_array_length(blockers) = 0 and canary_percent = 100)
    or
    (decision = 'canary_only' and jsonb_array_length(blockers) > 0 and canary_percent > 0 and canary_percent < 100)
    or
    (decision = 'block' and jsonb_array_length(blockers) > 0 and canary_percent = 0)
  ),
  check (
    risk = 'low'
    or decision = 'block'
    or approval_request_id is not null
  ),
  unique (candidate_id, gate_name)
);

create index release_gate_runs_decision_idx
  on release_gate_runs (decision, evaluated_at desc);

create index release_gate_runs_job_kind_idx
  on release_gate_runs (job_kind, evaluated_at desc);

create index release_gate_runs_versions_idx
  on release_gate_runs (
    prompt_version,
    model_version,
    tool_version,
    policy_version,
    evaluated_at desc
  );
-- ANCHOR_END: release_gate_runs

-- ANCHOR: job_kind_readiness_reviews
create table job_kind_readiness_reviews (
  id uuid primary key,
  job_kind text not null,
  target_level text not null check (
    target_level in ('demo', 'prototype', 'production', 'regulated_high_risk')
  ),
  current_level text not null check (
    current_level in ('demo', 'prototype', 'production', 'regulated_high_risk')
  ),
  risk_class text not null check (risk_class in ('low', 'medium', 'high', 'regulated')),
  evidence_ready_count int not null check (evidence_ready_count >= 0),
  evidence_required_count int not null check (evidence_required_count > 0),
  blocking_gap_count int not null default 0 check (blocking_gap_count >= 0),
  owner text not null,
  next_change text not null,
  evidence jsonb not null default '{}'::jsonb,
  reviewed_at timestamptz not null default now(),
  next_review_at timestamptz not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (btrim(job_kind) <> ''),
  check (btrim(owner) <> ''),
  check (btrim(next_change) <> ''),
  check (jsonb_typeof(evidence) = 'object'),
  check (evidence_ready_count <= evidence_required_count),
  check (blocking_gap_count <= evidence_required_count),
  check (next_review_at > reviewed_at),
  check (
    risk_class <> 'medium'
    or target_level in ('prototype', 'production', 'regulated_high_risk')
  ),
  check (
    risk_class <> 'high'
    or target_level in ('production', 'regulated_high_risk')
  ),
  check (
    risk_class <> 'regulated'
    or target_level = 'regulated_high_risk'
  ),
  unique (job_kind, target_level)
);

create index job_kind_readiness_reviews_due_idx
  on job_kind_readiness_reviews (next_review_at, job_kind);

create index job_kind_readiness_reviews_risk_idx
  on job_kind_readiness_reviews (risk_class, target_level, job_kind);
-- ANCHOR_END: job_kind_readiness_reviews

-- ANCHOR: job_kind_launch_packets
create table job_kind_launch_packets (
  id uuid primary key,
  job_kind text not null,
  target_level text not null check (
    target_level in ('demo', 'prototype', 'production', 'regulated_high_risk')
  ),
  risk_class text not null check (risk_class in ('low', 'medium', 'high', 'regulated')),
  launch_decision text not null check (
    launch_decision in ('draft', 'blocked', 'approved_for_first_users', 'launched', 'paused')
  ),
  owner text not null,
  durable_intake_proof text not null,
  worker_ownership_proof text not null,
  provider_boundary_proof text not null,
  side_effect_control_proof text not null,
  policy_or_approval_proof text not null,
  observability_proof text not null,
  evaluation_proof text not null,
  security_proof text not null,
  rollback_or_pause_plan text not null,
  restore_and_replay_note text not null,
  known_gaps jsonb not null default '[]'::jsonb,
  readiness_review_id uuid references job_kind_readiness_reviews(id) on delete restrict,
  release_gate_run_id uuid references release_gate_runs(id) on delete restrict,
  failure_drill_run_id uuid references failure_drill_runs(id) on delete set null,
  reviewed_by text not null,
  reviewed_at timestamptz not null default now(),
  next_review_at timestamptz not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (btrim(job_kind) <> ''),
  check (btrim(owner) <> ''),
  check (btrim(durable_intake_proof) <> ''),
  check (btrim(worker_ownership_proof) <> ''),
  check (btrim(provider_boundary_proof) <> ''),
  check (btrim(side_effect_control_proof) <> ''),
  check (btrim(policy_or_approval_proof) <> ''),
  check (btrim(observability_proof) <> ''),
  check (btrim(evaluation_proof) <> ''),
  check (btrim(security_proof) <> ''),
  check (btrim(rollback_or_pause_plan) <> ''),
  check (btrim(restore_and_replay_note) <> ''),
  check (btrim(reviewed_by) <> ''),
  check (jsonb_typeof(known_gaps) = 'array'),
  check (next_review_at > reviewed_at),
  check (
    risk_class <> 'medium'
    or target_level in ('prototype', 'production', 'regulated_high_risk')
  ),
  check (
    risk_class <> 'high'
    or target_level in ('production', 'regulated_high_risk')
  ),
  check (
    risk_class <> 'regulated'
    or target_level = 'regulated_high_risk'
  ),
  check (
    launch_decision not in ('approved_for_first_users', 'launched')
    or (
      readiness_review_id is not null
      and release_gate_run_id is not null
      and jsonb_array_length(known_gaps) = 0
    )
  ),
  check (
    risk_class not in ('high', 'regulated')
    or launch_decision not in ('approved_for_first_users', 'launched')
    or failure_drill_run_id is not null
  ),
  unique (job_kind, target_level, reviewed_at)
);

create index job_kind_launch_packets_due_idx
  on job_kind_launch_packets (next_review_at, job_kind);

create index job_kind_launch_packets_decision_idx
  on job_kind_launch_packets (launch_decision, risk_class, job_kind);
-- ANCHOR_END: job_kind_launch_packets

-- ANCHOR: fault_tolerance_reviews
create table fault_tolerance_reviews (
  id uuid primary key,
  job_kind text not null,
  control_plane_status text not null check (
    control_plane_status in ('healthy', 'degraded', 'unavailable')
  ),
  execution_plane_status text not null check (
    execution_plane_status in ('serving', 'degraded', 'paused')
  ),
  last_known_good_policy_version text not null,
  last_known_good_prompt_version text not null,
  last_known_good_model_version text not null,
  redundant_worker_count int not null check (redundant_worker_count >= 0),
  minimum_redundant_workers int not null check (minimum_redundant_workers > 0),
  isolated_failure_domain text not null,
  static_stability_mode text not null check (
    static_stability_mode in ('normal', 'last_known_good', 'draft_only', 'paused')
  ),
  progressive_delivery_channel text not null check (
    progressive_delivery_channel in ('dev', 'canary', 'production', 'high_risk_hold')
  ),
  failover_drill_run_id uuid references failure_drill_runs(id) on delete set null,
  release_gate_run_id uuid references release_gate_runs(id) on delete set null,
  owner text not null,
  reviewed_at timestamptz not null default now(),
  next_review_at timestamptz not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (btrim(job_kind) <> ''),
  check (btrim(last_known_good_policy_version) <> ''),
  check (btrim(last_known_good_prompt_version) <> ''),
  check (btrim(last_known_good_model_version) <> ''),
  check (btrim(isolated_failure_domain) <> ''),
  check (btrim(owner) <> ''),
  check (next_review_at > reviewed_at),
  unique (job_kind, reviewed_at)
);

create index fault_tolerance_reviews_due_idx
  on fault_tolerance_reviews (next_review_at, job_kind);

create index fault_tolerance_reviews_status_idx
  on fault_tolerance_reviews (control_plane_status, execution_plane_status, job_kind);
-- ANCHOR_END: fault_tolerance_reviews
