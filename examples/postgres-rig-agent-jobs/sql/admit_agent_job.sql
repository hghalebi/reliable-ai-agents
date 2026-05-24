-- ANCHOR: transactional_admission_enqueue
-- $1 job_id
-- $2 kind
-- $3 payload
-- $4 idempotency_key
-- $5 run_at
-- $6 max_attempts
-- $7 payload_schema_version
-- $8 prompt_version
-- $9 model_route
-- $10 tool_version
-- $11 policy_version
-- $12 worker_build_id
-- $13 admission_request_id
-- $14 tenant_key
-- $15 admission_job_kind
-- $16 priority
-- $17 queue_pressure
-- $18 provider_pressure
-- $19 budget_state
-- $20 projected_cost_micros_usd
-- $21 remaining_budget_micros_usd
-- $22 budget_limit_micros_usd
-- $23 decision
-- $24 reason
-- $25 admission_next_run_at
-- $26 decided_at
with inserted as (
  insert into agent_jobs (
    id,
    kind,
    payload_schema_version,
    prompt_version,
    model_route,
    tool_version,
    policy_version,
    worker_build_id,
    status,
    payload,
    idempotency_key,
    run_at,
    max_attempts
  )
  values (
    $1::uuid,
    $2::text,
    $7::int,
    $8::text,
    $9::text,
    $10::text,
    $11::text,
    $12::text,
    'pending',
    $3::jsonb,
    $4::text,
    $5::timestamptz,
    $6::int
  )
  on conflict (idempotency_key) do nothing
  returning id, true as inserted
),
selected_job as (
  select inserted.id, inserted.inserted
  from inserted
  union all
  select agent_jobs.id, false as inserted
  from agent_jobs
  where $4::text is not null
    and agent_jobs.idempotency_key = $4::text
    and not exists (select 1 from inserted)
  limit 1
),
agent_event_insert as (
  insert into agent_job_events (job_id, event_type, message)
  select
    selected_job.id,
    case
      when selected_job.inserted then 'job_enqueued'
      else 'duplicate_suppressed'
    end,
    $4::text
  from selected_job
  returning job_id
),
admission_event_insert as (
  insert into admission_decision_events (
    id,
    job_id,
    tenant_key,
    job_kind,
    priority,
    queue_pressure,
    provider_pressure,
    budget_state,
    projected_cost_micros_usd,
    remaining_budget_micros_usd,
    budget_limit_micros_usd,
    decision,
    reason,
    next_run_at,
    decided_at
  )
  select
    $13::uuid,
    selected_job.id,
    $14::text,
    $15::text,
    $16::text,
    $17::text,
    $18::text,
    $19::text,
    $20::bigint,
    $21::bigint,
    $22::bigint,
    $23::text,
    $24::text,
    $25::timestamptz,
    $26::timestamptz
  from selected_job
  returning job_id
)
select selected_job.id
from selected_job
join agent_event_insert on agent_event_insert.job_id = selected_job.id
join admission_event_insert on admission_event_insert.job_id = selected_job.id
limit 1;
-- ANCHOR_END: transactional_admission_enqueue
