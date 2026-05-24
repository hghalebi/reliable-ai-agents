#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BOOK_DIR="$ROOT_DIR/books/postgres-rig-agent-jobs"
CRATE_MANIFEST="$ROOT_DIR/examples/postgres-rig-agent-jobs/Cargo.toml"
COMPOSE_FILE="$ROOT_DIR/examples/postgres-rig-agent-jobs/docker-compose.postgres.yml"
DATABASE_URL="postgres://rig_agents:rig_agents_dev@localhost:55432/rig_agents"

export PYTHONDONTWRITEBYTECODE=1

cd "$ROOT_DIR"

if [[ "${RUN_LIVE_POSTGRES:-0}" == "1" ]]; then
  docker info >/dev/null 2>&1 || {
    echo "RUN_LIVE_POSTGRES=1 requires a running Docker-compatible daemon" >&2
    exit 1
  }
fi

echo "== source hygiene =="
test -f "$BOOK_DIR/src/assets/reliable-ai-agents-cover.png"
bash -n scripts/smoke-postgres-api.sh
bash -n scripts/smoke-local-postgres.sh
bash -n scripts/smoke-deepseek-agent.sh
bash -n scripts/check-public-mdbook-ci.sh
python3 scripts/check-public-repo-surface.py
python3 scripts/check-pages-workflow.py
python3 scripts/check-public-chapter-structure.py
python3 scripts/check-book-pedagogy.py
python3 scripts/check-book-coherence.py
python3 scripts/check-book-prose-quality.py
python3 scripts/check-book-terminology.py
python3 scripts/check-book-objective-coverage.py
python3 scripts/check-book-links.py
python3 scripts/check-sql-artifact-coverage.py
python3 scripts/check-postgres-schema-contract.py
python3 scripts/check-book-code-contract.py
python3 scripts/check-micro-drill-artifacts.py
python3 scripts/check-cargo-dependency-policy.py
python3 scripts/check-rust-boundary-types.py
python3 scripts/check-rust-production-hygiene.py
rg -n "Durable AI Agent Jobs" README.md "$BOOK_DIR/book.toml" "$BOOK_DIR/src" && {
  echo "old book title still appears in public book surfaces" >&2
  exit 1
} || true
rg -n "Windmill|windmill" README.md "$BOOK_DIR/src" examples/postgres-rig-agent-jobs/src && {
  echo "unrelated Windmill artifact found in book project" >&2
  exit 1
} || true
rg -n "TODO|todo|placeholder" README.md "$BOOK_DIR/src" examples/postgres-rig-agent-jobs/src && {
  echo "unfinished prose or code marker found" >&2
  exit 1
} || true

echo "== mdbook =="
mdbook test "$BOOK_DIR"
mdbook build "$BOOK_DIR"
python3 scripts/write-public-build-info.py "$ROOT_DIR" "$BOOK_DIR/book"
python3 scripts/check-public-mdbook-surface.py

echo "== rust format and tests =="
cargo fmt --manifest-path "$CRATE_MANIFEST" --check
cargo test --manifest-path "$CRATE_MANIFEST"
cargo test --manifest-path "$CRATE_MANIFEST" --no-default-features
cargo test --manifest-path "$CRATE_MANIFEST" --features api-server
cargo test --manifest-path "$CRATE_MANIFEST" --features postgres-store
cargo test --manifest-path "$CRATE_MANIFEST" --features rig-agent
cargo test --manifest-path "$CRATE_MANIFEST" --all-features

echo "== rust lint, docs, and audit =="
cargo clippy --manifest-path "$CRATE_MANIFEST" --all-targets --all-features -- -D warnings
cargo doc --manifest-path "$CRATE_MANIFEST" --all-features --no-deps
cargo audit --file "$ROOT_DIR/examples/postgres-rig-agent-jobs/Cargo.lock" --no-fetch
cargo tree --manifest-path "$CRATE_MANIFEST" --all-features --target all |
  rg "rig-bedrock|rig-lancedb|rig-fastembed|sqlx-mysql|sqlx-sqlite|sqlx-macros|rsa|rustls-webpki 0\\.101|lru v0\\.12" && {
    echo "forbidden inactive dependency surface found" >&2
    exit 1
  } || true

if [[ "${RUN_LOCAL_POSTGRES:-0}" == "1" ]]; then
  echo "== local postgres smoke =="
  scripts/smoke-local-postgres.sh
fi

if [[ "${RUN_LIVE_POSTGRES:-0}" == "1" ]]; then
  echo "== live postgres smoke =="
  docker compose -f "$COMPOSE_FILE" up -d
  cleanup() {
    docker compose -f "$COMPOSE_FILE" down -v
  }
  trap cleanup EXIT

  for _ in {1..30}; do
    if docker compose -f "$COMPOSE_FILE" exec -T postgres pg_isready -U rig_agents -d rig_agents; then
      break
    fi
    sleep 1
  done
  docker compose -f "$COMPOSE_FILE" exec -T postgres pg_isready -U rig_agents -d rig_agents
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql"
  env DATABASE_URL="$DATABASE_URL" cargo run --manifest-path "$CRATE_MANIFEST" \
    --features postgres-store --bin postgres-worker-demo
  env DATABASE_URL="$DATABASE_URL" BIND_ADDRESS="127.0.0.1:31080" \
    scripts/smoke-postgres-api.sh
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/oldest_pending_job.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/expired_leases.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/dead_jobs_by_reason.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v job_id="00000000-0000-0000-0000-000000000000" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/job_event_timeline.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v scheduled_job_id="00000000-0000-0000-0000-000000000000" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/failure_history_by_job.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v minimum_payload_schema_version="1" \
    -v maximum_payload_schema_version="1" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/version_compatibility_risks.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/schema_migration_status.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/failure_drill_status.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/running_agent_runs.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/scheduled_retries.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/waiting_human_approvals.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/failed_tool_calls.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v run_id="00000000-0000-0000-0000-000000000000" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/evaluation_receipts_by_version.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/release_gate_status.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/job_kind_lifecycle_review.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/job_kind_readiness_review.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/job_kind_launch_packet_status.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/fault_tolerance_readiness.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v memory_scope="tenant:demo" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/sli_terminal_jobs_not_dead.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/denied_authorization_events.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v run_id="00000000-0000-0000-0000-000000000000" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/audit_events_by_run.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v job_id="00000000-0000-0000-0000-000000000000" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/operation_events_by_job.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/storage_pressure_by_table.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/retention_review_by_surface.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/data_protection_review.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/outbox_backlog.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/compensation_backlog.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v workflow_execution_ref="kyc-case-00000000" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/temporal_workflow_reconciliation.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v outbox_event_id="00000000-0000-0000-0000-000000000000" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/kafka_replay_safety_by_event.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v kind="incident_triage" \
    -v actor="readiness-smoke" \
    -v reason="operator smoke test" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/pause_job_kind.sql"
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U rig_agents -d rig_agents \
    -v kind="incident_triage" \
    -v actor="readiness-smoke" \
    -v reason="operator smoke resolved" \
    < "$ROOT_DIR/examples/postgres-rig-agent-jobs/sql/resume_job_kind.sql"
fi

if [[ "${RUN_LIVE_DEEPSEEK:-0}" == "1" ]]; then
  echo "== live DeepSeek smoke =="
  : "${DEEPSEEK_API_KEY:?RUN_LIVE_DEEPSEEK=1 requires DEEPSEEK_API_KEY}"
  scripts/smoke-deepseek-agent.sh
fi

echo "book readiness checks passed"
