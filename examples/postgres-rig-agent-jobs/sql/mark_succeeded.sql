-- $1 id
-- $2 result
-- $3 worker_id
update agent_jobs
set
  status = 'succeeded',
  result = $2::jsonb,
  locked_by = null,
  locked_until = null,
  last_error = null,
  updated_at = now()
where id = $1::uuid
  and status = 'running'
  and locked_by = $3::text
returning id;
