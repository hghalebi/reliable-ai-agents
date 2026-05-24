-- $1 compensation action id
-- $2 compensating worker id
update compensation_actions
set
  status = 'succeeded',
  locked_by = null,
  locked_until = null,
  completed_at = now(),
  updated_at = now()
where id = $1::uuid
  and status = 'executing'
  and locked_by = $2::text
  and locked_until >= now()
returning *;
