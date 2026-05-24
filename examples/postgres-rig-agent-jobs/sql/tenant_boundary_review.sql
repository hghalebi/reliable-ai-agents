with tenant_authorization_window as (
  select
    authorization_events.actor_tenant_key,
    authorization_events.requested_tenant_key,
    count(*) as authorization_events,
    count(*) filter (
      where authorization_events.actor_tenant_key <> authorization_events.requested_tenant_key
    ) as cross_tenant_attempts,
    count(*) filter (
      where authorization_events.actor_tenant_key <> authorization_events.requested_tenant_key
        and authorization_events.decision <> 'denied'
    ) as cross_tenant_allowed,
    count(*) filter (
      where authorization_events.actor_tenant_key <> authorization_events.requested_tenant_key
        and authorization_events.decision = 'denied'
    ) as denied_cross_tenant_attempts,
    count(*) filter (
      where authorization_events.decision = 'requires_approval'
    ) as approvals_required,
    count(*) filter (
      where authorization_events.actor_tenant_key = authorization_events.requested_tenant_key
        and authorization_events.decision = 'authorized'
    ) as same_tenant_authorized,
    max(authorization_events.decided_at) as latest_decision_at
  from authorization_events
  where authorization_events.decided_at >= now() - interval '7 days'
  group by
    authorization_events.actor_tenant_key,
    authorization_events.requested_tenant_key
)
select
  tenant_authorization_window.actor_tenant_key,
  tenant_authorization_window.requested_tenant_key,
  tenant_authorization_window.authorization_events,
  tenant_authorization_window.cross_tenant_attempts,
  tenant_authorization_window.cross_tenant_allowed,
  tenant_authorization_window.denied_cross_tenant_attempts,
  tenant_authorization_window.approvals_required,
  tenant_authorization_window.same_tenant_authorized,
  tenant_authorization_window.latest_decision_at,
  case
    when tenant_authorization_window.cross_tenant_allowed > 0 then 'tenant_boundary_breach'
    when tenant_authorization_window.cross_tenant_attempts > 0 then 'cross_tenant_denied'
    when tenant_authorization_window.approvals_required > 0 then 'approval_required'
    else 'same_tenant_authorized'
  end as review_status
from tenant_authorization_window
order by
  tenant_authorization_window.cross_tenant_allowed desc,
  tenant_authorization_window.cross_tenant_attempts desc,
  tenant_authorization_window.approvals_required desc,
  tenant_authorization_window.latest_decision_at desc,
  tenant_authorization_window.actor_tenant_key,
  tenant_authorization_window.requested_tenant_key;
