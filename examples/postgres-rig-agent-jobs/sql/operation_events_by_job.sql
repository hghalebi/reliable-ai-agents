select
  id,
  run_id,
  trace_id,
  span_id,
  event_type,
  severity,
  message,
  data,
  created_at
from operation_events
where job_id = :'job_id'::uuid
order by created_at asc, id asc;
