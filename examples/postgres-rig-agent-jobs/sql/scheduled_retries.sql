select
  scheduled_jobs.id as scheduled_job_id,
  background_jobs.id as background_job_id,
  scheduled_jobs.task_name,
  background_jobs.job_kind,
  scheduled_jobs.next_run_at,
  background_jobs.retry_state,
  background_jobs.attempts,
  background_jobs.max_attempts,
  background_jobs.last_failure_class,
  coalesce(background_jobs.last_error, scheduled_jobs.last_error) as last_error
from scheduled_jobs
join background_jobs
  on background_jobs.scheduled_job_id = scheduled_jobs.id
where scheduled_jobs.status = 'pending'
  and background_jobs.retry_state = 'waiting_for_retry'
order by scheduled_jobs.next_run_at asc;
