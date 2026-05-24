select
  count(*) filter (where status = 'pending') as pending,
  count(*) filter (where status = 'running') as running,
  count(*) filter (where status = 'succeeded') as succeeded,
  count(*) filter (where status = 'failed') as failed,
  count(*) filter (where status = 'dead') as dead,
  count(*) filter (where status = 'cancelled') as cancelled,
  extract(epoch from now() - min(created_at) filter (where status = 'pending'))::bigint
    as oldest_pending_age_seconds
from agent_jobs;
