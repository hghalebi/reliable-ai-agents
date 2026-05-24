-- psql variable: scheduled_job_id
select
  failure_history.id,
  failure_history.scheduled_job_id,
  failure_history.background_job_id,
  failure_history.run_id,
  failure_history.step_id,
  failure_history.tool_call_id,
  failure_history.failure_source,
  failure_history.failure_class,
  failure_history.failure_message,
  failure_history.workflow_state,
  failure_history.retry_state,
  failure_history.failure_outcome,
  failure_history.attempt,
  failure_history.max_attempts,
  failure_history.next_retry_at,
  failure_history.trace_id,
  failure_history.span_id,
  failure_history.occurred_at,
  failure_history.recorded_at
from failure_history
where failure_history.scheduled_job_id = :'scheduled_job_id'::uuid
order by failure_history.occurred_at desc, failure_history.id desc;
