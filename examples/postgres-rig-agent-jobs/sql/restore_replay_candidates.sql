select
  scheduled_jobs.id as job_id,
  scheduled_jobs.task_name as job_kind,
  scheduled_jobs.status as job_status,
  scheduled_jobs.idempotency_key,
  coalesce(count(side_effect_receipts.id), 0)::bigint as side_effect_receipt_count,
  exists (
    select 1
    from tool_calls expected_tool_calls
    join agent_runs expected_runs
      on expected_runs.id = expected_tool_calls.run_id
    where expected_runs.scheduled_job_id = scheduled_jobs.id
      and expected_tool_calls.status in ('requested', 'validated', 'executed')
  ) as side_effect_expected,
  case
    when scheduled_jobs.status in ('succeeded', 'dead', 'cancelled') then 'do_not_replay_terminal_state'
    when count(side_effect_receipts.id) > 0 then 'reconcile_existing_receipt'
    when exists (
      select 1
      from tool_calls expected_tool_calls
      join agent_runs expected_runs
        on expected_runs.id = expected_tool_calls.run_id
      where expected_runs.scheduled_job_id = scheduled_jobs.id
        and expected_tool_calls.status in ('requested', 'validated', 'executed')
    ) then 'quarantine_missing_receipt'
    else 'resume_from_durable_state'
  end as replay_decision
from scheduled_jobs
left join agent_runs
  on agent_runs.scheduled_job_id = scheduled_jobs.id
left join tool_calls
  on tool_calls.run_id = agent_runs.id
left join side_effect_receipts
  on side_effect_receipts.tool_call_id = tool_calls.id
where scheduled_jobs.status in ('pending', 'running', 'failed', 'dead', 'cancelled', 'succeeded')
group by
  scheduled_jobs.id,
  scheduled_jobs.task_name,
  scheduled_jobs.status,
  scheduled_jobs.idempotency_key
order by scheduled_jobs.updated_at asc, scheduled_jobs.id asc;
