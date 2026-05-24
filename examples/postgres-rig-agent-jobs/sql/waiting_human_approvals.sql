select
  human_approval_requests.id as approval_request_id,
  human_approval_requests.run_id,
  agent_runs.agent_name,
  agent_runs.prompt_version,
  agent_runs.model_version,
  human_approval_requests.requested_by,
  human_approval_requests.requested_at,
  human_approval_requests.expires_at,
  extract(epoch from now() - human_approval_requests.requested_at)::bigint
    as waiting_seconds
from human_approval_requests
join agent_runs
  on agent_runs.id = human_approval_requests.run_id
where human_approval_requests.status = 'requested'
order by human_approval_requests.requested_at asc;
