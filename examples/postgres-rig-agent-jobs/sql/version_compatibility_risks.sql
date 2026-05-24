-- psql variables: minimum_payload_schema_version, maximum_payload_schema_version
select
  id as job_id,
  kind as job_kind,
  status as job_status,
  payload_schema_version,
  prompt_version,
  model_route,
  tool_version,
  policy_version,
  worker_build_id,
  run_at,
  created_at,
  case
    when payload_schema_version < :'minimum_payload_schema_version'::int
      then 'payload_schema_too_old'
    when payload_schema_version > :'maximum_payload_schema_version'::int
      then 'payload_schema_too_new'
  end as compatibility_risk
from agent_jobs
where status in ('pending', 'running')
  and (
    payload_schema_version < :'minimum_payload_schema_version'::int
    or payload_schema_version > :'maximum_payload_schema_version'::int
  )
order by run_at asc, created_at asc, id asc;
