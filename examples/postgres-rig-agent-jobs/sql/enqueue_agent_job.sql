-- $1 id
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
)
select * from inserted
union all
select agent_jobs.id, false as inserted
from agent_jobs
where $4::text is not null
  and agent_jobs.idempotency_key = $4::text
  and not exists (select 1 from inserted)
limit 1;
