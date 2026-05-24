select
  release_gate_runs.candidate_id,
  release_gate_runs.gate_name,
  release_gate_runs.job_kind,
  release_gate_runs.risk,
  release_gate_runs.decision,
  release_gate_runs.release_reason,
  release_gate_runs.prompt_version,
  release_gate_runs.model_version,
  release_gate_runs.tool_version,
  release_gate_runs.policy_version,
  release_gate_runs.worker_build_id,
  release_gate_runs.payload_schema_version,
  evaluation_runs.status as evaluation_status,
  round(evaluation_runs.score * 10000)::int as evaluation_score_basis_points,
  release_gate_runs.slo_decision,
  release_gate_runs.compatibility_decision,
  schema_migration_runs.phase as migration_phase,
  schema_migration_runs.status as migration_status,
  human_approval_requests.status as approval_status,
  jsonb_array_length(release_gate_runs.blockers) as blocker_count,
  release_gate_runs.blockers,
  release_gate_runs.canary_percent,
  release_gate_runs.rollback_plan,
  release_gate_runs.evaluated_at,
  release_gate_runs.evaluated_by,
  release_gate_runs.operator_signoff
from release_gate_runs
left join evaluation_runs
  on evaluation_runs.id = release_gate_runs.evaluation_run_id
left join schema_migration_runs
  on schema_migration_runs.id = release_gate_runs.schema_migration_run_id
left join human_approval_requests
  on human_approval_requests.id = release_gate_runs.approval_request_id
where
  release_gate_runs.decision <> 'promote'
  or release_gate_runs.evaluated_at >= now() - interval '30 days'
order by
  case release_gate_runs.decision
    when 'block' then 0
    when 'canary_only' then 1
    else 2
  end,
  release_gate_runs.evaluated_at desc
limit 50;
