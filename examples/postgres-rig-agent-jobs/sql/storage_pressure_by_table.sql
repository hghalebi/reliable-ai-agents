select
  stat.relname as table_name,
  stat.n_live_tup::bigint as estimated_live_rows,
  stat.n_dead_tup::bigint as estimated_dead_rows,
  round(
    (
      stat.n_dead_tup::numeric
      / greatest(stat.n_live_tup + stat.n_dead_tup, 1)
    ) * 100,
    2
  ) as estimated_dead_row_percent,
  pg_total_relation_size(
    format('%I.%I', stat.schemaname, stat.relname)::regclass
  ) as total_bytes,
  pg_size_pretty(
    pg_total_relation_size(
      format('%I.%I', stat.schemaname, stat.relname)::regclass
    )
  ) as total_size,
  stat.last_vacuum,
  stat.last_autovacuum,
  stat.last_analyze,
  stat.last_autoanalyze
from pg_stat_user_tables stat
where stat.schemaname = 'public'
  and stat.relname in (
    'agent_jobs',
    'agent_job_events',
    'admission_decision_events',
    'scheduled_jobs',
    'background_jobs',
    'agent_runs',
    'agent_steps',
    'tool_calls',
    'failure_history',
    'authorization_events',
    'sandbox_events',
    'side_effect_receipts',
    'outbox_events',
    'audit_events',
    'operation_events',
    'provider_usage_events',
    'human_approval_requests',
    'human_escalations',
    'compensation_actions',
    'evaluation_runs',
    'agent_memory_records',
    'restore_drill_runs'
  )
order by total_bytes desc, estimated_dead_row_percent desc, table_name asc;
