-- $1 compensation action id
-- $2 compensating worker id
-- $3 retry delay
-- $4 failure message
update compensation_actions
set
  status = case
    when attempts >= max_attempts then 'failed'
    else 'approved'
  end,
  next_attempt_at = case
    when attempts >= max_attempts then next_attempt_at
    else now() + $3::interval
  end,
  locked_by = null,
  locked_until = null,
  last_error = $4::text,
  completed_at = case
    when attempts >= max_attempts then now()
    else null
  end,
  updated_at = now()
where id = $1::uuid
  and status = 'executing'
  and locked_by = $2::text
returning *;
