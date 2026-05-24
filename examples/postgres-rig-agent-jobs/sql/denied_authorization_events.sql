select
  authorization_events.id as authorization_event_id,
  authorization_events.run_id,
  authorization_events.actor_id,
  authorization_events.actor_tenant_key,
  authorization_events.requested_tenant_key,
  authorization_events.tool_name,
  authorization_events.permission,
  authorization_events.decision,
  authorization_events.reason,
  authorization_events.policy_version,
  authorization_events.decided_at
from authorization_events
where authorization_events.decision in ('denied', 'requires_approval')
order by authorization_events.decided_at desc, authorization_events.id desc
limit 100;
