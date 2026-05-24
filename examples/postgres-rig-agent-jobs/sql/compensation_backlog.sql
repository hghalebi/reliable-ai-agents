select
  status,
  compensation_kind,
  count(*) as actions,
  min(next_attempt_at) filter (where status = 'approved') as oldest_due_at,
  extract(epoch from now() - min(next_attempt_at) filter (where status = 'approved'))::bigint
    as oldest_due_age_seconds,
  count(*) filter (
    where status = 'executing'
      and locked_until < now()
  ) as expired_execution_leases
from compensation_actions
group by status, compensation_kind
order by status asc, compensation_kind asc;
