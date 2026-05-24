-- $1 publisher worker id
-- $2 lease duration
-- $3 batch size
with picked as (
  select id
  from outbox_events
  where status = 'pending'
    and next_attempt_at <= now()
  order by next_attempt_at asc, occurred_at asc, id asc
  limit $3::int
  for update of outbox_events skip locked
)
update outbox_events
set
  status = 'publishing',
  attempts = attempts + 1,
  locked_by = $1::text,
  locked_until = now() + $2::interval,
  updated_at = now()
where id in (select id from picked)
returning *;
