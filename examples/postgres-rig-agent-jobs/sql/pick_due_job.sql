with picked as (
  select id
  from agent_jobs
  left join agent_job_kind_controls controls using (kind)
  where status = 'pending'
    and run_at <= now()
    and coalesce(controls.paused, false) = false
  order by run_at asc, created_at asc
  limit 1
  for update of agent_jobs skip locked
)
update agent_jobs
set
  status = 'running',
  locked_by = $1,
  locked_until = now() + $2::interval,
  attempt_count = attempt_count + 1,
  updated_at = now()
where id in (select id from picked)
returning *;
