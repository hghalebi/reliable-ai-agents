-- psql variables: kind, actor, reason
with previous_control as (
  select kind, paused, reason
  from agent_job_kind_controls
  where kind = :'kind'::text
),
upserted_control as (
  insert into agent_job_kind_controls (kind, paused, reason, updated_by)
  values (:'kind'::text, true, :'reason'::text, :'actor'::text)
  on conflict (kind) do update
  set
    paused = true,
    reason = excluded.reason,
    updated_by = excluded.updated_by,
    updated_at = now()
  returning kind, paused, reason, updated_by, updated_at
),
control_event as (
  insert into agent_job_kind_control_events (
    kind,
    action,
    actor,
    reason,
    previous_paused,
    previous_reason,
    new_paused,
    new_reason
  )
  select
    upserted_control.kind,
    'paused',
    :'actor'::text,
    :'reason'::text,
    coalesce(previous_control.paused, false),
    previous_control.reason,
    upserted_control.paused,
    upserted_control.reason
  from upserted_control
  left join previous_control using (kind)
  returning id as control_event_id
)
select
  upserted_control.kind,
  upserted_control.paused,
  upserted_control.reason,
  upserted_control.updated_by,
  upserted_control.updated_at,
  control_event.control_event_id
from upserted_control
cross join control_event;
