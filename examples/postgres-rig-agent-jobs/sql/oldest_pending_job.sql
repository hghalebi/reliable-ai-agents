select
  id,
  kind,
  run_at,
  created_at,
  extract(epoch from now() - created_at)::bigint as pending_age_seconds,
  payload_schema_version,
  prompt_version,
  model_route,
  policy_version
from agent_jobs
where status = 'pending'
order by created_at asc
limit 1;
