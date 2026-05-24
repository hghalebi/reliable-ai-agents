select
  tool_name,
  policy_version,
  reason,
  count(*) as denied_events,
  min(decided_at) as first_seen_at,
  max(decided_at) as last_seen_at
from sandbox_events
where decision = 'denied'
group by tool_name, policy_version, reason
order by denied_events desc, last_seen_at desc, tool_name asc;
