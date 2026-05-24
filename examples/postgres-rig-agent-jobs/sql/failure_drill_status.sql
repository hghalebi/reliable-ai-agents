select
  drill_name,
  scenario,
  environment,
  status,
  owner,
  scheduled_job_id,
  run_id,
  trace_id,
  required_evidence_count,
  observed_evidence_count,
  round(
    observed_evidence_count::numeric * 100
      / nullif(required_evidence_count::numeric, 0),
    2
  ) as evidence_percent,
  hypothesis,
  blast_radius,
  injection,
  rollback_action,
  decision_reason,
  started_at,
  extract(epoch from now() - started_at)::bigint as age_seconds,
  completed_at,
  operator_signoff
from failure_drill_runs
where status in ('planned', 'running', 'failed', 'aborted')
   or completed_at >= now() - interval '90 days'
order by
  case status
    when 'failed' then 0
    when 'aborted' then 1
    when 'running' then 2
    when 'planned' then 3
    when 'passed' then 4
    else 5
  end,
  started_at desc,
  drill_name asc;
