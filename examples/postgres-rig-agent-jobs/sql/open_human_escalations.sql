select
  human_escalations.id as escalation_id,
  human_escalations.job_id,
  human_escalations.run_id,
  human_escalations.escalation_kind,
  human_escalations.severity,
  human_escalations.status,
  human_escalations.reason,
  human_escalations.assigned_to,
  human_escalations.created_at,
  human_escalations.acknowledged_at,
  extract(epoch from now() - human_escalations.created_at)::bigint as open_for_seconds,
  scheduled_jobs.task_name as job_kind,
  scheduled_jobs.status as job_status,
  agent_runs.agent_name,
  agent_runs.lifecycle_status as run_status,
  agent_runs.prompt_version,
  agent_runs.model_version
from human_escalations
left join scheduled_jobs
  on scheduled_jobs.id = human_escalations.job_id
left join agent_runs
  on agent_runs.id = human_escalations.run_id
where human_escalations.status in ('open', 'acknowledged')
order by
  case human_escalations.severity
    when 'page' then 0
    when 'ticket' then 1
    else 2
  end,
  human_escalations.created_at asc,
  human_escalations.id asc;
