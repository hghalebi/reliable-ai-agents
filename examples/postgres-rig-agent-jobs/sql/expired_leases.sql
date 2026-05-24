select
  id,
  kind,
  locked_by,
  locked_until,
  extract(epoch from now() - locked_until)::bigint as expired_by_seconds,
  attempt_count,
  max_attempts,
  worker_build_id
from agent_jobs
where status = 'running'
  and locked_until < now()
order by locked_until asc, created_at asc;
