with data_surfaces(surface) as (
  values
    ('agent_jobs'),
    ('scheduled_jobs'),
    ('background_jobs'),
    ('agent_runs'),
    ('tool_calls'),
    ('audit_events'),
    ('operation_events'),
    ('provider_usage_events'),
    ('human_approval_requests'),
    ('side_effect_receipts'),
    ('evaluation_runs'),
    ('agent_memory_records')
),
request_rollup as (
  select
    surface,
    count(*) filter (
      where status in ('requested', 'approved')
    )::bigint as open_requests,
    count(*) filter (
      where status in ('requested', 'approved')
        and due_at < now()
    )::bigint as overdue_requests,
    count(*) filter (
      where request_kind = 'redaction'
        and status in ('requested', 'approved')
    )::bigint as pending_redaction_requests,
    count(*) filter (
      where request_kind = 'erasure'
        and status in ('requested', 'approved')
    )::bigint as pending_erasure_requests,
    count(*) filter (
      where status = 'applied'
        and completed_at >= now() - interval '30 days'
    )::bigint as recently_applied_requests_30d,
    max(requested_at) as latest_request_at,
    max(completed_at) as latest_completed_at
  from data_protection_requests
  group by surface
)
select
  data_surfaces.surface,
  coalesce(request_rollup.open_requests, 0)::bigint as open_requests,
  coalesce(request_rollup.overdue_requests, 0)::bigint as overdue_requests,
  coalesce(request_rollup.pending_redaction_requests, 0)::bigint
    as pending_redaction_requests,
  coalesce(request_rollup.pending_erasure_requests, 0)::bigint
    as pending_erasure_requests,
  coalesce(request_rollup.recently_applied_requests_30d, 0)::bigint
    as recently_applied_requests_30d,
  request_rollup.latest_request_at,
  request_rollup.latest_completed_at,
  case
    when coalesce(request_rollup.overdue_requests, 0) > 0
      then 'privacy_review_overdue'
    when coalesce(request_rollup.pending_erasure_requests, 0) > 0
      then 'erasure_pending'
    when coalesce(request_rollup.pending_redaction_requests, 0) > 0
      then 'redaction_pending'
    when coalesce(request_rollup.open_requests, 0) > 0
      then 'privacy_work_pending'
    else 'no_open_privacy_work'
  end as review_status
from data_surfaces
left join request_rollup using (surface)
order by
  overdue_requests desc,
  pending_erasure_requests desc,
  pending_redaction_requests desc,
  open_requests desc,
  data_surfaces.surface asc;
