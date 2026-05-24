select
  id,
  actor_type,
  actor_id,
  action,
  subject,
  event_data,
  created_at
from audit_events
where run_id = :'run_id'::uuid
order by created_at asc, id asc;
