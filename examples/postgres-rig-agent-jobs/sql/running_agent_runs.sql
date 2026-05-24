select
  agent_runs.id as run_id,
  agent_runs.scheduled_job_id,
  agent_runs.background_job_id,
  background_jobs.workflow_state,
  agent_runs.agent_name,
  agent_runs.lifecycle_status,
  agent_runs.prompt_version,
  agent_runs.model_version,
  agent_runs.trace_id,
  extract(epoch from now() - agent_runs.started_at)::bigint as running_for_seconds
from agent_runs
left join background_jobs
  on background_jobs.id = agent_runs.background_job_id
where agent_runs.lifecycle_status in ('planning', 'running', 'waiting_for_human')
order by agent_runs.started_at asc;
