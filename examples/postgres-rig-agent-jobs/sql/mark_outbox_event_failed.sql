-- $1 outbox event id
-- $2 publisher worker id
-- $3 retry delay
-- $4 failure message
update outbox_events
set
  status = case
    when attempts >= max_attempts then 'failed'
    else 'pending'
  end,
  next_attempt_at = case
    when attempts >= max_attempts then next_attempt_at
    else now() + $3::interval
  end,
  locked_by = null,
  locked_until = null,
  last_error = $4::text,
  updated_at = now()
where id = $1::uuid
  and status = 'publishing'
  and locked_by = $2::text
returning *;
