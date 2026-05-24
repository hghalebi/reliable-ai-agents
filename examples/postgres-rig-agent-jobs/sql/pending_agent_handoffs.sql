select
  to_agent,
  count(*) as pending_handoffs,
  min(requested_at) as oldest_requested_at,
  extract(epoch from now() - min(requested_at))::bigint as oldest_pending_age_seconds
from agent_handoffs
where status = 'requested'
group by to_agent
order by oldest_pending_age_seconds desc, to_agent asc;
