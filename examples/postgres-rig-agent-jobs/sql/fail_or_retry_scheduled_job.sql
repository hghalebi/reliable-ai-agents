update scheduled_jobs
set
  status = case
    when attempts >= max_attempts then 'dead'
    else 'pending'
  end,
  next_run_at = case
    when attempts >= max_attempts then next_run_at
    else now() + $2::interval
  end,
  locked_by = null,
  locked_until = null,
  last_error = $3::text,
  updated_at = now()
where id = $1::uuid
  and status = 'running'
  and locked_by = $4::text
  and locked_until >= now()
returning *;
