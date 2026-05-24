with known_job_kinds as (
  select kind as job_kind
  from agent_jobs
  union
  select task_name as job_kind
  from scheduled_jobs
  union
  select job_kind
  from background_jobs
  union
  select job_kind
  from provider_usage_events
  union
  select job_kind
  from release_gate_runs
  union
  select kind as job_kind
  from agent_job_kind_controls
),
scheduled_activity as (
  select
    task_name as job_kind,
    count(*) filter (where status in ('pending', 'running')) as pending_or_running_jobs
  from scheduled_jobs
  group by task_name
),
background_activity as (
  select
    job_kind,
    count(*) filter (
      where workflow_state in ('queued', 'leased', 'executing_agent')
    ) as pending_or_running_jobs,
    count(*) filter (where workflow_state = 'waiting_for_retry') as waiting_retry_jobs,
    count(*) filter (where workflow_state = 'waiting_for_human') as waiting_human_jobs
  from background_jobs
  group by job_kind
),
legacy_activity as (
  select
    kind as job_kind,
    count(*) filter (where status in ('pending', 'running')) as pending_or_running_jobs
  from agent_jobs
  group by kind
),
provider_usage as (
  select
    job_kind,
    count(*) filter (where recorded_at >= now() - interval '30 days') as recent_provider_calls_30d
  from provider_usage_events
  group by job_kind
),
latest_release as (
  select distinct on (job_kind)
    job_kind,
    decision as latest_release_decision,
    evaluated_at as latest_release_evaluated_at
  from release_gate_runs
  order by job_kind, evaluated_at desc
),
lifecycle_evidence as (
  select
    known_job_kinds.job_kind,
    coalesce(agent_job_kind_controls.paused, false) as paused,
    agent_job_kind_controls.reason as pause_reason,
    (
      coalesce(scheduled_activity.pending_or_running_jobs, 0)
      + coalesce(background_activity.pending_or_running_jobs, 0)
      + coalesce(legacy_activity.pending_or_running_jobs, 0)
    )::bigint as pending_or_running_jobs,
    coalesce(background_activity.waiting_retry_jobs, 0)::bigint as waiting_retry_jobs,
    coalesce(background_activity.waiting_human_jobs, 0)::bigint as waiting_human_jobs,
    coalesce(provider_usage.recent_provider_calls_30d, 0)::bigint as recent_provider_calls_30d,
    latest_release.latest_release_decision,
    latest_release.latest_release_evaluated_at
  from known_job_kinds
  left join agent_job_kind_controls
    on agent_job_kind_controls.kind = known_job_kinds.job_kind
  left join scheduled_activity
    on scheduled_activity.job_kind = known_job_kinds.job_kind
  left join background_activity
    on background_activity.job_kind = known_job_kinds.job_kind
  left join legacy_activity
    on legacy_activity.job_kind = known_job_kinds.job_kind
  left join provider_usage
    on provider_usage.job_kind = known_job_kinds.job_kind
  left join latest_release
    on latest_release.job_kind = known_job_kinds.job_kind
)
select
  job_kind,
  paused,
  pause_reason,
  pending_or_running_jobs,
  waiting_retry_jobs,
  waiting_human_jobs,
  recent_provider_calls_30d,
  latest_release_decision,
  latest_release_evaluated_at,
  case
    when pending_or_running_jobs > 0
      or waiting_retry_jobs > 0
      or waiting_human_jobs > 0
      then 'retirement_blocked'
    when paused = true
      and recent_provider_calls_30d = 0
      then 'retirement_candidate'
    when recent_provider_calls_30d = 0
      then 'deprecation_candidate'
    else 'active'
  end as lifecycle_recommendation
from lifecycle_evidence
order by
  case
    when pending_or_running_jobs > 0
      or waiting_retry_jobs > 0
      or waiting_human_jobs > 0
      then 0
    when paused = true
      and recent_provider_calls_30d = 0
      then 1
    when recent_provider_calls_30d = 0
      then 2
    else 3
  end,
  job_kind asc;
