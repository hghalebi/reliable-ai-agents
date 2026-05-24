#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_MANIFEST="$ROOT_DIR/examples/postgres-rig-agent-jobs/Cargo.toml"

require_cmd() {
  local name="$1"
  command -v "$name" >/dev/null 2>&1 || {
    echo "smoke-local-postgres requires '$name' on PATH" >&2
    exit 1
  }
}

select_postgres_bin_dir() {
  local candidate
  local candidates=()

  if [[ -n "${POSTGRES_BIN_DIR:-}" ]]; then
    candidates+=("$POSTGRES_BIN_DIR")
  fi

  if command -v pg_ctl >/dev/null 2>&1; then
    candidates+=("$(dirname "$(command -v pg_ctl)")")
  fi

  candidates+=(
    "/opt/homebrew/opt/postgresql@17/bin"
    "/opt/homebrew/opt/postgresql@16/bin"
    "/opt/homebrew/opt/postgresql@15/bin"
    "/usr/local/opt/postgresql@17/bin"
    "/usr/local/opt/postgresql@16/bin"
    "/usr/local/opt/postgresql@15/bin"
    "/Applications/Postgres.app/Contents/Versions/latest/bin"
    "/Applications/Postgres.app/Contents/Versions/17/bin"
    "/Applications/Postgres.app/Contents/Versions/16/bin"
    "/Applications/Postgres.app/Contents/Versions/15/bin"
  )

  for candidate in "${candidates[@]}"; do
    if [[ -x "$candidate/initdb" \
      && -x "$candidate/pg_ctl" \
      && -x "$candidate/postgres" \
      && -x "$candidate/psql" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  done

  echo "smoke-local-postgres requires a complete Postgres bin directory with initdb, pg_ctl, postgres, and psql" >&2
  echo "Set POSTGRES_BIN_DIR=/path/to/postgres/bin if it is installed outside the common locations." >&2
  exit 1
}

free_port() {
  python3 - <<'PY'
import socket

sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
}

require_cmd python3
require_cmd cargo

POSTGRES_BIN_DIR="$(select_postgres_bin_dir)"
INITDB="$POSTGRES_BIN_DIR/initdb"
PG_CTL="$POSTGRES_BIN_DIR/pg_ctl"
PSQL="$POSTGRES_BIN_DIR/psql"

TMP_DIR="$(mktemp -d)"
PGDATA="$TMP_DIR/data"
POSTGRES_LOG="$TMP_DIR/postgres.log"
WORKER_LOG="$TMP_DIR/postgres-worker-demo.log"
API_LOG="$TMP_DIR/postgres-api-smoke.log"
POSTGRES_PORT="${LOCAL_POSTGRES_PORT:-$(free_port)}"
API_PORT="${LOCAL_POSTGRES_API_PORT:-$(free_port)}"
DATABASE_URL="postgres://postgres@127.0.0.1:${POSTGRES_PORT}/postgres"

cleanup() {
  "$PG_CTL" -D "$PGDATA" -m fast -w stop >/dev/null 2>&1 || true
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

psql_file() {
  "$PSQL" "$DATABASE_URL" -v ON_ERROR_STOP=1 "$@" >/dev/null
}

"$INITDB" -D "$PGDATA" --username=postgres --no-locale --encoding=UTF8 >/dev/null
"$PG_CTL" \
  -D "$PGDATA" \
  -l "$POSTGRES_LOG" \
  -o "-h 127.0.0.1 -p $POSTGRES_PORT" \
  -w start >/dev/null

psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql"

env DATABASE_URL="$DATABASE_URL" cargo run \
  --manifest-path "$CRATE_MANIFEST" \
  --features postgres-store \
  --bin postgres-worker-demo >"$WORKER_LOG"

env \
  DATABASE_URL="$DATABASE_URL" \
  BIND_ADDRESS="127.0.0.1:${API_PORT}" \
  "$ROOT_DIR/scripts/smoke-postgres-api.sh" >"$API_LOG"

psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/oldest_pending_job.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/expired_leases.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/dead_jobs_by_reason.sql"
psql_file \
  -v job_id="00000000-0000-0000-0000-000000000000" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/job_event_timeline.sql"
psql_file \
  -v scheduled_job_id="00000000-0000-0000-0000-000000000000" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/failure_history_by_job.sql"
psql_file \
  -v minimum_payload_schema_version="1" \
  -v maximum_payload_schema_version="1" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/version_compatibility_risks.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/schema_migration_status.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/failure_drill_status.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/running_agent_runs.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/scheduled_retries.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/waiting_human_approvals.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/failed_tool_calls.sql"
psql_file \
  -v run_id="00000000-0000-0000-0000-000000000000" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/evaluation_receipts_by_version.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/release_gate_status.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/job_kind_lifecycle_review.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/job_kind_readiness_review.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/job_kind_launch_packet_status.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/fault_tolerance_readiness.sql"
psql_file \
  -v memory_scope="tenant:demo" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/sli_terminal_jobs_not_dead.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/denied_authorization_events.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql"
psql_file \
  -v run_id="00000000-0000-0000-0000-000000000000" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/audit_events_by_run.sql"
psql_file \
  -v job_id="00000000-0000-0000-0000-000000000000" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/operation_events_by_job.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/storage_pressure_by_table.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/retention_review_by_surface.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/data_protection_review.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/outbox_backlog.sql"
psql_file -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/compensation_backlog.sql"
psql_file \
  -v workflow_execution_ref="kyc-case-00000000" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/temporal_workflow_reconciliation.sql"
psql_file \
  -v outbox_event_id="00000000-0000-0000-0000-000000000000" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/kafka_replay_safety_by_event.sql"
psql_file \
  -v kind="incident_triage" \
  -v actor="readiness-smoke" \
  -v reason="operator smoke test" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/pause_job_kind.sql"
psql_file \
  -v kind="incident_triage" \
  -v actor="readiness-smoke" \
  -v reason="operator smoke resolved" \
  -f "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/resume_job_kind.sql"

control_event_count="$(
  "$PSQL" "$DATABASE_URL" -v ON_ERROR_STOP=1 -tAc "
    select count(*)
    from agent_job_kind_control_events
    where actor = 'readiness-smoke'
      and (
        action = 'paused'
        and previous_paused = false
        and new_paused = true
        or action = 'resumed'
        and previous_paused = true
        and new_paused = false
      );
  "
)"

if [[ "$control_event_count" != "2" ]]; then
  echo "expected two audited pause/resume control events, found $control_event_count" >&2
  exit 1
fi

echo "local Postgres smoke passed on port $POSTGRES_PORT"
