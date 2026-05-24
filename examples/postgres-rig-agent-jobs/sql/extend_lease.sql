-- $1 id
-- $2 worker_id
-- $3 lease_duration
update agent_jobs
set
  locked_until = now() + $3::interval,
  updated_at = now()
where id = $1::uuid
  and status = 'running'
  and locked_by = $2::text
returning id;
