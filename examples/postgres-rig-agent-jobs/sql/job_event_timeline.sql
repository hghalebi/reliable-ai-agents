-- psql variable: job_id
select
  event_type,
  message,
  data,
  created_at
from agent_job_events
where job_id = :'job_id'::uuid
order by created_at asc, id asc;
