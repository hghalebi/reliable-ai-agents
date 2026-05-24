select
  cancellation_requests.id,
  cancellation_requests.job_id,
  cancellation_requests.run_id,
  cancellation_requests.requested_by,
  cancellation_requests.source,
  cancellation_requests.mode,
  cancellation_requests.reason,
  cancellation_requests.requested_at,
  cancellation_requests.expires_at,
  extract(epoch from (now() - cancellation_requests.requested_at))::bigint
    as pending_age_seconds,
  scheduled_jobs.status as job_status,
  scheduled_jobs.locked_by,
  scheduled_jobs.locked_until
from cancellation_requests
join scheduled_jobs on scheduled_jobs.id = cancellation_requests.job_id
where cancellation_requests.status = 'requested'
order by cancellation_requests.requested_at asc, cancellation_requests.id asc;
