-- psql variable: workflow_execution_ref
select
  link.workflow_execution_ref,
  link.workflow_type,
  link.task_queue,
  link.workflow_status,
  job.status as scheduled_job_status,
  run.lifecycle_status as agent_run_status,
  count(distinct activity.id) as activity_receipt_count,
  count(distinct approval.id) filter (
    where approval.status in ('approved', 'rejected')
  ) as decided_approval_count,
  count(distinct audit.id) as audit_event_count,
  count(distinct operation.id) as activity_operation_event_count,
  max(link.updated_at) as workflow_link_updated_at
from temporal_workflow_links link
join scheduled_jobs job
  on job.id = link.scheduled_job_id
join agent_runs run
  on run.id = link.agent_run_id
left join temporal_activity_receipts activity
  on activity.workflow_link_id = link.id
left join operation_events operation
  on operation.id = activity.operation_event_id
left join human_approval_requests approval
  on approval.run_id = run.id
left join audit_events audit
  on audit.run_id = run.id
where link.workflow_execution_ref = :'workflow_execution_ref'
group by
  link.workflow_execution_ref,
  link.workflow_type,
  link.task_queue,
  link.workflow_status,
  job.status,
  run.lifecycle_status
order by workflow_link_updated_at desc;
