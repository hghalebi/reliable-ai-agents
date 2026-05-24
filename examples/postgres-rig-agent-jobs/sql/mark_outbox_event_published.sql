-- $1 outbox event id
-- $2 publisher worker id
update outbox_events
set
  status = 'published',
  locked_by = null,
  locked_until = null,
  published_at = now(),
  updated_at = now()
where id = $1::uuid
  and status = 'publishing'
  and locked_by = $2::text
  and locked_until >= now()
returning *;
