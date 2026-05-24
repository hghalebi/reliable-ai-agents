with terminal_jobs as (
  select
    kind,
    status,
    updated_at
  from agent_jobs
  where status in ('succeeded', 'dead', 'cancelled')
    and updated_at >= now() - interval '30 days'
)
select
  'terminal-jobs-not-dead:v1' as slo_name,
  'terminal_jobs_not_dead' as sli_name,
  kind as job_kind,
  now() - interval '30 days' as window_started_at,
  now() as window_ended_at,
  9900::bigint as target_basis_points,
  count(*) filter (where status <> 'dead')::bigint as good_events,
  count(*)::bigint as total_events,
  count(*) filter (where status = 'dead')::bigint as dead_events
from terminal_jobs
group by kind
order by kind asc;
