select
  job_kind,
  provider_name,
  model_route,
  count(*) as provider_calls,
  count(*) filter (where provider_status = 'succeeded') as succeeded_calls,
  count(*) filter (where provider_status = 'rate_limited') as rate_limited_calls,
  count(*) filter (where provider_status = 'timeout') as timed_out_calls,
  count(*) filter (where provider_status = 'failed') as failed_calls,
  sum(prompt_tokens) as prompt_tokens,
  sum(completion_tokens) as completion_tokens,
  sum(total_tokens) as total_tokens,
  sum(cost_micros_usd) as cost_micros_usd,
  percentile_cont(0.95) within group (order by latency_ms) as p95_latency_ms
from provider_usage_events
where recorded_at >= now() - interval '24 hours'
group by job_kind, provider_name, model_route
order by cost_micros_usd desc, provider_calls desc, job_kind asc;
