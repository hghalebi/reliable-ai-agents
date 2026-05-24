select
  scheduled_jobs.id as job_id,
  agent_runs.id as run_id,
  background_jobs.job_kind,
  agent_runs.lifecycle_status as observed_state,
  agent_runs.timeout_policy_name,
  agent_runs.started_at,
  agent_runs.deadline_at,
  agent_runs.timeout_action,
  background_jobs.attempts,
  background_jobs.max_attempts,
  extract(epoch from (now() - agent_runs.deadline_at))::bigint as overdue_seconds
from agent_runs
join background_jobs
  on background_jobs.id = agent_runs.background_job_id
join scheduled_jobs
  on scheduled_jobs.id = coalesce(agent_runs.scheduled_job_id, background_jobs.scheduled_job_id)
where agent_runs.lifecycle_status in ('running', 'waiting_for_human')
  and agent_runs.deadline_at is not null
  and agent_runs.deadline_at < now()
order by agent_runs.deadline_at asc, agent_runs.started_at asc, agent_runs.id asc;
