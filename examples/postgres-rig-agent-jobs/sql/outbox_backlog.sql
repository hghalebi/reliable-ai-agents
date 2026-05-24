select
  status,
  event_kind,
  count(*) as events,
  min(next_attempt_at) filter (where status = 'pending') as oldest_due_at,
  extract(epoch from now() - min(next_attempt_at) filter (where status = 'pending'))::bigint
    as oldest_due_age_seconds,
  count(*) filter (
    where status = 'publishing'
      and locked_until < now()
  ) as expired_publication_leases
from outbox_events
group by status, event_kind
order by status asc, event_kind asc;
