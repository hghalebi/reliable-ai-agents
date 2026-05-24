with due_jobs as (
  select id
  from scheduled_jobs
  where status = 'pending'
    and next_run_at <= now()
    and attempts < max_attempts
  order by next_run_at asc, created_at asc
  for update skip locked
  limit $1
)
update scheduled_jobs
set
  status = 'running',
  attempts = attempts + 1,
  locked_by = $2::text,
  locked_until = now() + $3::interval,
  updated_at = now()
from due_jobs
where scheduled_jobs.id = due_jobs.id
returning scheduled_jobs.*;
