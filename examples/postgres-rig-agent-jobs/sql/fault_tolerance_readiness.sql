with job_counts as (
  select
    kind as job_kind,
    count(*) filter (where status in ('pending', 'running')) as active_jobs,
    count(*) filter (where status = 'running') as running_jobs,
    count(*) filter (where status = 'dead') as dead_jobs
  from agent_jobs
  group by kind
)
select
  ftr.job_kind,
  ftr.control_plane_status,
  ftr.execution_plane_status,
  ftr.last_known_good_policy_version,
  ftr.last_known_good_prompt_version,
  ftr.last_known_good_model_version,
  ftr.redundant_worker_count,
  ftr.minimum_redundant_workers,
  ftr.isolated_failure_domain,
  ftr.static_stability_mode,
  ftr.progressive_delivery_channel,
  failover.status as failover_drill_status,
  release.decision as latest_release_decision,
  coalesce(job_counts.active_jobs, 0) as active_jobs,
  coalesce(job_counts.running_jobs, 0) as running_jobs,
  coalesce(job_counts.dead_jobs, 0) as dead_jobs,
  ftr.reviewed_at,
  ftr.next_review_at,
  ftr.next_review_at < now() as review_overdue,
  case
    when ftr.next_review_at < now() then 'review_overdue'
    when ftr.control_plane_status = 'unavailable'
      and ftr.execution_plane_status = 'serving'
      and ftr.static_stability_mode = 'normal'
      then 'control_plane_coupled'
    when ftr.control_plane_status in ('degraded', 'unavailable')
      and ftr.static_stability_mode = 'normal'
      then 'static_stability_missing'
    when ftr.redundant_worker_count < ftr.minimum_redundant_workers
      then 'needs_redundancy'
    when failover.status is distinct from 'passed'
      then 'failover_drill_missing'
    when ftr.progressive_delivery_channel = 'production'
      and release.id is null
      then 'release_gate_missing'
    when release.decision = 'block'
      then 'release_blocked'
    else 'ready'
  end as readiness_status
from fault_tolerance_reviews ftr
left join failure_drill_runs failover
  on failover.id = ftr.failover_drill_run_id
left join release_gate_runs release
  on release.id = ftr.release_gate_run_id
left join job_counts
  on job_counts.job_kind = ftr.job_kind
order by
  case
    when ftr.next_review_at < now() then 0
    when ftr.control_plane_status in ('degraded', 'unavailable') then 1
    when ftr.redundant_worker_count < ftr.minimum_redundant_workers then 2
    when failover.status is distinct from 'passed' then 3
    when release.decision = 'block' then 4
    else 5
  end,
  ftr.next_review_at,
  ftr.job_kind;
