create table agent_jobs (
  id uuid primary key,
  kind text not null,
  -- ANCHOR: version_columns
  payload_schema_version int not null default 1 check (payload_schema_version > 0),
  prompt_version text not null default 'incident-triage:v1',
  model_route text not null default 'deterministic-local:v1',
  tool_version text not null default 'no-tools:v1',
  policy_version text not null default 'approval-policy:v1',
  worker_build_id text not null default 'local-dev',
  -- ANCHOR_END: version_columns
  status text not null check (
    status in ('pending', 'running', 'succeeded', 'failed', 'dead', 'cancelled')
  ),
  payload jsonb not null,
  result jsonb,
  run_at timestamptz not null default now(),
  attempt_count int not null default 0 check (attempt_count >= 0),
  max_attempts int not null default 5 check (max_attempts > 0),
  locked_by text,
  locked_until timestamptz,
  idempotency_key text unique,
  last_error text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check (btrim(kind) <> ''),
  check (jsonb_typeof(payload) = 'object'),
  check (result is null or jsonb_typeof(result) = 'object'),
  check (btrim(prompt_version) <> ''),
  check (btrim(model_route) <> ''),
  check (btrim(tool_version) <> ''),
  check (btrim(policy_version) <> ''),
  check (btrim(worker_build_id) <> ''),
  check (idempotency_key is null or btrim(idempotency_key) <> ''),
  check (last_error is null or btrim(last_error) <> ''),
  check (
    (status = 'succeeded' and result is not null and last_error is null)
    or
    (status <> 'succeeded' and result is null)
  ),
  check (
    (status = 'running' and locked_by is not null and locked_until is not null)
    or
    (status <> 'running' and locked_by is null and locked_until is null)
  )
);

create table agent_job_kind_controls (
  kind text primary key,
  paused boolean not null default false,
  reason text,
  updated_by text not null default 'system',
  updated_at timestamptz not null default now(),
  check (btrim(updated_by) <> ''),
  check (
    (paused = true and reason is not null)
    or
    (paused = false and reason is null)
  )
);

create table agent_job_kind_control_events (
  id bigserial primary key,
  kind text not null,
  action text not null check (action in ('paused', 'resumed')),
  actor text not null check (btrim(actor) <> ''),
  reason text not null check (btrim(reason) <> ''),
  previous_paused boolean not null,
  previous_reason text,
  new_paused boolean not null,
  new_reason text,
  created_at timestamptz not null default now(),
  check (
    (action = 'paused' and new_paused = true and new_reason is not null)
    or
    (action = 'resumed' and new_paused = false and new_reason is null)
  )
);

create index agent_job_kind_control_events_kind_idx
  on agent_job_kind_control_events (kind, created_at desc, id desc);

create index agent_jobs_due_idx
  on agent_jobs (run_at, created_at)
  where status = 'pending';

create index agent_jobs_expired_running_idx
  on agent_jobs (locked_until)
  where status = 'running';

create index agent_jobs_kind_status_run_at_idx
  on agent_jobs (kind, status, run_at);

create index agent_jobs_dead_reason_idx
  on agent_jobs (kind, updated_at)
  where status = 'dead';

create table agent_job_events (
  id bigserial primary key,
  job_id uuid not null references agent_jobs(id) on delete cascade,
  event_type text not null check (
    event_type in (
      'job_enqueued',
      'duplicate_suppressed',
      'job_picked',
      'agent_started',
      'agent_succeeded',
      'agent_failed',
      'retry_scheduled',
      'job_succeeded',
      'job_dead',
      'lease_extended',
      'job_cancelled',
      'expired_lease_recovered'
    )
  ),
  message text,
  data jsonb,
  created_at timestamptz not null default now()
);

create index agent_job_events_job_id_created_at_idx
  on agent_job_events (job_id, created_at, id);

-- ANCHOR: admission_decision_events_table
create table admission_decision_events (
  id uuid primary key,
  job_id uuid references agent_jobs(id) on delete set null,
  tenant_key text not null,
  job_kind text not null,
  priority text not null check (priority in ('interactive', 'standard', 'bulk')),
  queue_pressure text not null check (
    queue_pressure in ('healthy', 'backlogged', 'saturated')
  ),
  provider_pressure text not null check (
    provider_pressure in ('healthy', 'near_limit', 'exhausted')
  ),
  budget_state text not null check (budget_state in ('within_budget', 'exceeded')),
  projected_cost_micros_usd bigint not null check (projected_cost_micros_usd >= 0),
  remaining_budget_micros_usd bigint check (remaining_budget_micros_usd >= 0),
  budget_limit_micros_usd bigint check (budget_limit_micros_usd >= 0),
  decision text not null check (decision in ('accepted', 'delayed', 'rejected')),
  reason text not null check (
    reason in (
      'within_operating_envelope',
      'queue_backlogged',
      'queue_saturated',
      'provider_near_limit',
      'provider_exhausted',
      'tenant_budget_exceeded'
    )
  ),
  next_run_at timestamptz,
  decided_at timestamptz not null,
  recorded_at timestamptz not null default now(),
  check (btrim(tenant_key) <> ''),
  check (btrim(job_kind) <> ''),
  check (
    (decision in ('accepted', 'delayed') and job_id is not null)
    or
    (decision = 'rejected' and job_id is null)
  ),
  check (
    (decision = 'delayed' and next_run_at is not null and next_run_at > decided_at)
    or
    (decision <> 'delayed' and next_run_at is null)
  ),
  check (
    (budget_state = 'within_budget'
      and remaining_budget_micros_usd is not null
      and budget_limit_micros_usd is null)
    or
    (budget_state = 'exceeded'
      and remaining_budget_micros_usd is null
      and budget_limit_micros_usd is not null)
  )
);
-- ANCHOR_END: admission_decision_events_table

create index admission_decision_events_tenant_time_idx
  on admission_decision_events (tenant_key, decided_at desc);

create index admission_decision_events_decision_time_idx
  on admission_decision_events (decision, decided_at desc);
