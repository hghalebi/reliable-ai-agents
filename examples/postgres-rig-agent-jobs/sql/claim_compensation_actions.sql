-- $1 compensating worker id
-- $2 lease duration
-- $3 batch size
with picked as (
  select id
  from compensation_actions
  where status = 'approved'
    and next_attempt_at <= now()
    and attempts < max_attempts
  order by next_attempt_at asc, requested_at asc, id asc
  limit $3::int
  for update of compensation_actions skip locked
)
update compensation_actions
set
  status = 'executing',
  attempts = attempts + 1,
  locked_by = $1::text,
  locked_until = now() + $2::interval,
  updated_at = now()
where id in (select id from picked)
returning *;
