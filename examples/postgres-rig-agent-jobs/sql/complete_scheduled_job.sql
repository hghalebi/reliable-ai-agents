update scheduled_jobs
set
  status = 'succeeded',
  locked_by = null,
  locked_until = null,
  last_error = null,
  updated_at = now()
where id = $1::uuid
  and status = 'running'
  and locked_by = $2::text
  and locked_until >= now()
returning *;
