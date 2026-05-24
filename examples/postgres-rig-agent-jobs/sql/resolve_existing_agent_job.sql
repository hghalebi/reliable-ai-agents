-- ANCHOR: resolve_existing_agent_job
-- $1 idempotency_key
with existing_job as (
  select id, status, run_at
  from agent_jobs
  where idempotency_key = $1::text
),
duplicate_event_insert as (
  insert into agent_job_events (job_id, event_type, message)
  select existing_job.id, 'duplicate_suppressed', $1::text
  from existing_job
  returning job_id
)
select existing_job.id, existing_job.status, existing_job.run_at
from existing_job
left join duplicate_event_insert on duplicate_event_insert.job_id = existing_job.id
limit 1;
-- ANCHOR_END: resolve_existing_agent_job
