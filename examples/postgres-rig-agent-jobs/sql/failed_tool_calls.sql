select
  tool_calls.id as tool_call_id,
  tool_calls.run_id,
  tool_calls.step_id,
  tool_calls.tool_name,
  tool_calls.tool_version,
  tool_calls.status,
  tool_calls.error,
  tool_calls.created_at
from tool_calls
where tool_calls.status in ('failed', 'rejected')
order by tool_calls.created_at desc
limit 50;
