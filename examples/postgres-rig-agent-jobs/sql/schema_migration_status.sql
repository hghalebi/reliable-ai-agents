select
  migration_name,
  phase,
  status,
  target_surface,
  target_version,
  compatible_from_payload_schema_version,
  compatible_through_payload_schema_version,
  rows_examined,
  rows_changed,
  round((rows_changed::numeric / greatest(rows_examined, 1)) * 100, 2) as changed_percent,
  compatibility_query_name,
  rollback_plan,
  extract(epoch from now() - started_at)::bigint as age_seconds,
  completed_at,
  operator_signoff
from schema_migration_runs
where status in ('planned', 'running', 'failed', 'blocked')
  or completed_at >= now() - interval '30 days'
order by
  case status
    when 'failed' then 1
    when 'blocked' then 2
    when 'running' then 3
    when 'planned' then 4
    else 5
  end,
  started_at desc,
  migration_name asc;
