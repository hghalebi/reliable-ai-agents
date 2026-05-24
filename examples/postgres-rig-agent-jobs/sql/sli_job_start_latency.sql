with first_pick as (
  select
    job_id,
    min(created_at) as picked_at
  from agent_job_events
  where event_type = 'job_picked'
  group by job_id
),
eligible_jobs as (
  select
    agent_jobs.id,
    agent_jobs.kind,
    agent_jobs.run_at,
    first_pick.picked_at
  from agent_jobs
  left join first_pick
    on first_pick.job_id = agent_jobs.id
  where agent_jobs.run_at >= now() - interval '30 days'
    and agent_jobs.run_at <= now()
)
select
  'job-start-latency:v1' as slo_name,
  'job_start_latency_within_120s' as sli_name,
  kind as job_kind,
  now() - interval '30 days' as window_started_at,
  now() as window_ended_at,
  9900::bigint as target_basis_points,
  count(*) filter (
    where picked_at is not null
      and picked_at <= run_at + interval '120 seconds'
  )::bigint as good_events,
  count(*)::bigint as total_events,
  ceil(
    percentile_cont(0.95) within group (
      order by extract(epoch from picked_at - run_at)
    ) filter (where picked_at is not null)
  )::bigint as p95_start_latency_seconds
from eligible_jobs
group by kind
order by kind asc;
