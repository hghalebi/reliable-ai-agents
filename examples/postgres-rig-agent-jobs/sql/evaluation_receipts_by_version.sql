select
  evaluation_runs.id as evaluation_run_id,
  coalesce(background_jobs.job_kind, scheduled_jobs.task_name, 'unlinked') as job_kind,
  evaluation_runs.status,
  round(evaluation_runs.score * 10000)::bigint as score_basis_points,
  evaluation_runs.dataset_version,
  evaluation_runs.evaluator_version,
  evaluation_runs.prompt_version,
  evaluation_runs.model_version,
  evaluation_runs.tool_version,
  evaluation_runs.policy_version,
  agent_runs.id as run_id,
  agent_runs.trace_id,
  evaluation_runs.report ? 'failed_cases' as has_failed_case_details,
  evaluation_runs.created_at,
  evaluation_runs.completed_at,
  extract(epoch from (evaluation_runs.completed_at - evaluation_runs.created_at))::bigint
    as duration_seconds
from evaluation_runs
left join agent_runs on agent_runs.id = evaluation_runs.run_id
left join background_jobs on background_jobs.id = agent_runs.background_job_id
left join scheduled_jobs on scheduled_jobs.id = agent_runs.scheduled_job_id
where evaluation_runs.created_at >= now() - interval '30 days'
order by evaluation_runs.created_at desc, evaluation_runs.id desc
limit 50;
