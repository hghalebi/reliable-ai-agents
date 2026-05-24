select
  kind,
  count(*) filter (where status = 'pending') as pending,
  count(*) filter (where status = 'running') as running,
  count(*) filter (where status = 'dead') as dead,
  extract(epoch from now() - min(created_at) filter (where status = 'pending'))::bigint
    as oldest_pending_age_seconds,
  extract(epoch from now() - min(locked_until) filter (
    where status = 'running'
      and locked_until < now()
  ))::bigint as most_expired_lease_age_seconds
from agent_jobs
group by kind
order by pending desc, running desc, dead desc, kind asc;
