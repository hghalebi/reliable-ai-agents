with retention_surfaces as (
  select
    'agent_jobs' as surface,
    count(*)::bigint as total_rows,
    count(*) filter (where created_at < now() - interval '90 days')::bigint as older_than_90_days,
    count(*) filter (where created_at < now() - interval '365 days')::bigint as older_than_365_days,
    min(created_at) as oldest_observed_at,
    max(created_at) as newest_observed_at
  from agent_jobs
  union all
  select
    'agent_job_events',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from agent_job_events
  union all
  select
    'scheduled_jobs',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from scheduled_jobs
  union all
  select
    'background_jobs',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from background_jobs
  union all
  select
    'agent_runs',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from agent_runs
  union all
  select
    'tool_calls',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from tool_calls
  union all
  select
    'failure_history',
    count(*)::bigint,
    count(*) filter (where recorded_at < now() - interval '90 days')::bigint,
    count(*) filter (where recorded_at < now() - interval '365 days')::bigint,
    min(recorded_at),
    max(recorded_at)
  from failure_history
  union all
  select
    'audit_events',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from audit_events
  union all
  select
    'operation_events',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from operation_events
  union all
  select
    'provider_usage_events',
    count(*)::bigint,
    count(*) filter (where recorded_at < now() - interval '90 days')::bigint,
    count(*) filter (where recorded_at < now() - interval '365 days')::bigint,
    min(recorded_at),
    max(recorded_at)
  from provider_usage_events
  union all
  select
    'human_approval_requests',
    count(*)::bigint,
    count(*) filter (where requested_at < now() - interval '90 days')::bigint,
    count(*) filter (where requested_at < now() - interval '365 days')::bigint,
    min(requested_at),
    max(requested_at)
  from human_approval_requests
  union all
  select
    'side_effect_receipts',
    count(*)::bigint,
    count(*) filter (where recorded_at < now() - interval '90 days')::bigint,
    count(*) filter (where recorded_at < now() - interval '365 days')::bigint,
    min(recorded_at),
    max(recorded_at)
  from side_effect_receipts
  union all
  select
    'agent_memory_records',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from agent_memory_records
  union all
  select
    'restore_drill_runs',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from restore_drill_runs
  union all
  select
    'failure_drill_runs',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from failure_drill_runs
  union all
  select
    'release_gate_runs',
    count(*)::bigint,
    count(*) filter (where created_at < now() - interval '90 days')::bigint,
    count(*) filter (where created_at < now() - interval '365 days')::bigint,
    min(created_at),
    max(created_at)
  from release_gate_runs
)
select
  surface,
  total_rows,
  older_than_90_days,
  older_than_365_days,
  oldest_observed_at,
  newest_observed_at
from retention_surfaces
order by older_than_365_days desc, older_than_90_days desc, total_rows desc, surface asc;
