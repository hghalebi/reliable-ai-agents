-- $1 id
-- $2 job_id
-- $3 tenant_key
-- $4 job_kind
-- $5 priority
-- $6 queue_pressure
-- $7 provider_pressure
-- $8 budget_state
-- $9 projected_cost_micros_usd
-- $10 remaining_budget_micros_usd
-- $11 budget_limit_micros_usd
-- $12 decision
-- $13 reason
-- $14 next_run_at
-- $15 decided_at
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
values (
  $1::uuid,
  $2::uuid,
  $3::text,
  $4::text,
  $5::text,
  $6::text,
  $7::text,
  $8::text,
  $9::bigint,
  $10::bigint,
  $11::bigint,
  $12::text,
  $13::text,
  $14::timestamptz,
  $15::timestamptz
);
