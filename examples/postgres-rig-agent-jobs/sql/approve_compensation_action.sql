-- $1 compensation action id
-- $2 approved human approval request id
update compensation_actions
set
  status = 'approved',
  approval_request_id = $2::uuid,
  approved_at = now(),
  next_attempt_at = now(),
  updated_at = now()
where id = $1::uuid
  and status = 'requested'
  and exists (
    select 1
    from human_approval_requests
    where id = $2::uuid
      and status = 'approved'
  )
returning *;
