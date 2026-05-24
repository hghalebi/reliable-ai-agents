update agent_jobs
set
  status = 'pending',
  locked_by = null,
  locked_until = null,
  run_at = now(),
  updated_at = now()
where status = 'running'
  and locked_until < now()
returning id;
