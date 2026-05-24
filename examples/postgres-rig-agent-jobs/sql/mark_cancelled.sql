-- $1 id
-- $2 reason
update agent_jobs
set
  status = 'cancelled',
  locked_by = null,
  locked_until = null,
  last_error = $2::text,
  updated_at = now()
where id = $1::uuid
  and status in ('pending', 'running')
returning id;
