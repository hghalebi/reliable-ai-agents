select
  kind,
  coalesce(last_error, 'unknown failure') as last_error,
  count(*) as dead_count,
  min(updated_at) as first_seen_at,
  max(updated_at) as last_seen_at
from agent_jobs
where status = 'dead'
group by kind, coalesce(last_error, 'unknown failure')
order by dead_count desc, last_seen_at desc;
