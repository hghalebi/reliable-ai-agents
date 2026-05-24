with credential_kinds(credential_kind) as (
  values
    ('provider_api_key'),
    ('database_url'),
    ('operator_token'),
    ('webhook_secret'),
    ('service_account_key'),
    ('ci_secret'),
    ('encryption_key')
),
credential_rollup as (
  select
    credential_kind,
    count(*) filter (
      where status in ('active', 'rotation_due', 'rotating', 'compromised')
    )::bigint as managed_credentials,
    count(*) filter (
      where status in ('active', 'rotation_due')
        and next_rotation_due_at <= now()
    )::bigint as rotation_due,
    count(*) filter (
      where status in ('active', 'rotation_due', 'rotating', 'compromised')
        and next_rotation_due_at < now()
    )::bigint as overdue_rotation,
    count(*) filter (
      where status = 'compromised'
    )::bigint as open_exposure_incidents,
    count(*) filter (
      where status in ('active', 'rotation_due', 'rotating', 'compromised')
        and (
          last_verified_at is null
          or last_verified_at < now() - interval '30 days'
        )
    )::bigint as stale_verification,
    count(*) filter (
      where status = 'revoked'
        and revoked_at >= now() - interval '30 days'
    )::bigint as revoked_credentials_30d,
    max(last_rotated_at) as latest_rotation_at,
    min(next_rotation_due_at) filter (
      where status in ('active', 'rotation_due', 'rotating', 'compromised')
    ) as next_rotation_due_at
  from credential_assets
  group by credential_kind
)
select
  credential_kinds.credential_kind,
  coalesce(credential_rollup.managed_credentials, 0)::bigint as managed_credentials,
  coalesce(credential_rollup.rotation_due, 0)::bigint as rotation_due,
  coalesce(credential_rollup.overdue_rotation, 0)::bigint as overdue_rotation,
  coalesce(credential_rollup.open_exposure_incidents, 0)::bigint
    as open_exposure_incidents,
  coalesce(credential_rollup.stale_verification, 0)::bigint
    as stale_verification,
  coalesce(credential_rollup.revoked_credentials_30d, 0)::bigint
    as revoked_credentials_30d,
  credential_rollup.latest_rotation_at,
  credential_rollup.next_rotation_due_at,
  case
    when coalesce(credential_rollup.open_exposure_incidents, 0) > 0
      then 'exposure_incident_open'
    when coalesce(credential_rollup.overdue_rotation, 0) > 0
      then 'rotation_overdue'
    when coalesce(credential_rollup.rotation_due, 0) > 0
      then 'rotation_due'
    when coalesce(credential_rollup.stale_verification, 0) > 0
      then 'verification_stale'
    when coalesce(credential_rollup.managed_credentials, 0) = 0
      then 'no_credentials_registered'
    else 'credential_health_ok'
  end as review_status
from credential_kinds
left join credential_rollup using (credential_kind)
order by
  open_exposure_incidents desc,
  overdue_rotation desc,
  rotation_due desc,
  stale_verification desc,
  credential_kinds.credential_kind asc;
