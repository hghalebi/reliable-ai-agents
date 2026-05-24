-- $1 id
-- $2 retry_disposition
-- $3 retry_delay
-- $4 error_message
-- $5 worker_id
update agent_jobs
set
  status = case
    when $2::text = 'permanent' or attempt_count >= max_attempts then 'dead'
    else 'pending'
  end,
  run_at = case
    when $2::text = 'permanent' or attempt_count >= max_attempts then run_at
    else now() + $3::interval
  end,
  locked_by = null,
  locked_until = null,
  last_error = $4::text,
  updated_at = now()
where id = $1::uuid
  and status = 'running'
  and locked_by = $5::text
returning id, status, run_at;
