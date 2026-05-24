with launch_packets as (
  select
    job_kind_launch_packets.*,
    jsonb_array_length(job_kind_launch_packets.known_gaps) as known_gap_count
  from job_kind_launch_packets
),
readiness as (
  select
    job_kind_readiness_reviews.id,
    job_kind_readiness_reviews.job_kind,
    job_kind_readiness_reviews.target_level,
    job_kind_readiness_reviews.current_level,
    job_kind_readiness_reviews.risk_class,
    job_kind_readiness_reviews.evidence_ready_count,
    job_kind_readiness_reviews.evidence_required_count,
    job_kind_readiness_reviews.blocking_gap_count,
    job_kind_readiness_reviews.next_review_at < now() as readiness_review_overdue,
    case
      when job_kind_readiness_reviews.next_review_at < now() then 'review_overdue'
      when job_kind_readiness_reviews.blocking_gap_count > 0 then 'blocked_by_gaps'
      when job_kind_readiness_reviews.evidence_ready_count = job_kind_readiness_reviews.evidence_required_count
        and (
          job_kind_readiness_reviews.current_level = job_kind_readiness_reviews.target_level
          or job_kind_readiness_reviews.current_level = 'regulated_high_risk'
          or (
            job_kind_readiness_reviews.current_level = 'production'
            and job_kind_readiness_reviews.target_level in ('demo', 'prototype')
          )
          or (
            job_kind_readiness_reviews.current_level = 'prototype'
            and job_kind_readiness_reviews.target_level = 'demo'
          )
        )
        then 'ready_for_target'
      else 'missing_evidence'
    end as readiness_status
  from job_kind_readiness_reviews
)
select
  launch_packets.id,
  launch_packets.job_kind,
  launch_packets.target_level,
  launch_packets.risk_class,
  launch_packets.launch_decision,
  launch_packets.owner,
  launch_packets.durable_intake_proof,
  launch_packets.worker_ownership_proof,
  launch_packets.provider_boundary_proof,
  launch_packets.side_effect_control_proof,
  launch_packets.policy_or_approval_proof,
  launch_packets.observability_proof,
  launch_packets.evaluation_proof,
  launch_packets.security_proof,
  launch_packets.rollback_or_pause_plan,
  launch_packets.restore_and_replay_note,
  launch_packets.known_gaps,
  launch_packets.known_gap_count,
  launch_packets.readiness_review_id,
  launch_packets.release_gate_run_id,
  launch_packets.failure_drill_run_id,
  launch_packets.reviewed_by,
  launch_packets.reviewed_at,
  launch_packets.next_review_at,
  launch_packets.next_review_at < now() as review_overdue,
  readiness.readiness_status,
  release_gate_runs.decision as release_decision,
  failure_drill_runs.status as failure_drill_status,
  case
    when launch_packets.next_review_at < now() then 'review_overdue'
    when launch_packets.launch_decision = 'paused' then 'paused'
    when launch_packets.known_gap_count > 0
      or launch_packets.launch_decision = 'blocked'
      then 'blocked_by_gaps'
    when readiness.readiness_status = 'ready_for_target'
      and release_gate_runs.decision = 'promote'
      and (
        launch_packets.risk_class not in ('high', 'regulated')
        or failure_drill_runs.status = 'passed'
      )
      and launch_packets.launch_decision in ('approved_for_first_users', 'launched')
      then 'ready_for_first_users'
    else 'missing_evidence'
  end as launch_status
from launch_packets
left join readiness
  on readiness.id = launch_packets.readiness_review_id
left join release_gate_runs
  on release_gate_runs.id = launch_packets.release_gate_run_id
left join failure_drill_runs
  on failure_drill_runs.id = launch_packets.failure_drill_run_id
order by
  case
    when launch_packets.next_review_at < now() then 0
    when launch_packets.launch_decision = 'blocked'
      or launch_packets.known_gap_count > 0
      then 1
    when launch_packets.launch_decision = 'paused' then 2
    when readiness.readiness_status = 'ready_for_target'
      and release_gate_runs.decision = 'promote'
      and (
        launch_packets.risk_class not in ('high', 'regulated')
        or failure_drill_runs.status = 'passed'
      )
      and launch_packets.launch_decision in ('approved_for_first_users', 'launched')
      then 4
    else 3
  end,
  launch_packets.next_review_at asc,
  launch_packets.job_kind asc,
  launch_packets.target_level asc;
