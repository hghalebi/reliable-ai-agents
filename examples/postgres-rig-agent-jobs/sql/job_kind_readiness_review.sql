with latest_release as (
  select distinct on (job_kind)
    job_kind,
    decision as latest_release_decision,
    evaluated_at as latest_release_evaluated_at
  from release_gate_runs
  order by job_kind, evaluated_at desc
),
readiness as (
  select
    job_kind_readiness_reviews.job_kind,
    job_kind_readiness_reviews.target_level,
    job_kind_readiness_reviews.current_level,
    job_kind_readiness_reviews.risk_class,
    job_kind_readiness_reviews.evidence_ready_count,
    job_kind_readiness_reviews.evidence_required_count,
    job_kind_readiness_reviews.blocking_gap_count,
    job_kind_readiness_reviews.owner,
    job_kind_readiness_reviews.next_change,
    job_kind_readiness_reviews.reviewed_at,
    job_kind_readiness_reviews.next_review_at,
    job_kind_readiness_reviews.next_review_at < now() as review_overdue,
    latest_release.latest_release_decision,
    latest_release.latest_release_evaluated_at
  from job_kind_readiness_reviews
  left join latest_release
    on latest_release.job_kind = job_kind_readiness_reviews.job_kind
)
select
  job_kind,
  target_level,
  current_level,
  risk_class,
  evidence_ready_count,
  evidence_required_count,
  blocking_gap_count,
  owner,
  next_change,
  reviewed_at,
  next_review_at,
  review_overdue,
  latest_release_decision,
  latest_release_evaluated_at,
  case
    when review_overdue then 'review_overdue'
    when blocking_gap_count > 0 then 'blocked_by_gaps'
    when evidence_ready_count = evidence_required_count
      and (
        current_level = target_level
        or current_level = 'regulated_high_risk'
        or (current_level = 'production' and target_level in ('demo', 'prototype'))
        or (current_level = 'prototype' and target_level = 'demo')
      )
      then 'ready_for_target'
    else 'missing_evidence'
  end as readiness_status
from readiness
order by
  case
    when review_overdue then 0
    when blocking_gap_count > 0 then 1
    when evidence_ready_count < evidence_required_count then 2
    else 3
  end,
  next_review_at asc,
  job_kind asc,
  target_level asc;
