#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_MANIFEST="$ROOT_DIR/examples/postgres-rig-agent-jobs/Cargo.toml"
SERVER_BIN="$ROOT_DIR/examples/postgres-rig-agent-jobs/target/debug/postgres-api-server"

: "${DATABASE_URL:?DATABASE_URL is required for the Postgres API smoke test}"

BIND_ADDRESS="${BIND_ADDRESS:-127.0.0.1:31080}"
API_BASE_URL="${API_BASE_URL:-http://$BIND_ADDRESS}"
SMOKE_IDEMPOTENCY_KEY="${SMOKE_IDEMPOTENCY_KEY:-api-smoke:$(date -u +%Y%m%dT%H%M%SZ):$$}"
TMP_DIR="$(mktemp -d)"
SERVER_LOG="$TMP_DIR/postgres-api-server.log"
api_pid=""

cleanup() {
  local status="$1"
  if [[ "$status" != "0" ]] && [[ -f "$SERVER_LOG" ]]; then
    echo "postgres API server log:" >&2
    sed -n '1,200p' "$SERVER_LOG" >&2
  fi

  if [[ -n "$api_pid" ]] && kill -0 "$api_pid" >/dev/null 2>&1; then
    kill "$api_pid" >/dev/null 2>&1 || true
    wait "$api_pid" >/dev/null 2>&1 || true
  fi
  rm -rf "$TMP_DIR"
}
trap 'status=$?; cleanup "$status"; exit "$status"' EXIT

cargo build \
  --manifest-path "$CRATE_MANIFEST" \
  --features api-server,postgres-store \
  --bin postgres-api-server

env DATABASE_URL="$DATABASE_URL" BIND_ADDRESS="$BIND_ADDRESS" \
  "$SERVER_BIN" >"$SERVER_LOG" 2>&1 &
api_pid="$!"

for attempt in {1..30}; do
  if ! kill -0 "$api_pid" >/dev/null 2>&1; then
    echo "postgres API server exited before becoming healthy" >&2
    sed -n '1,160p' "$SERVER_LOG" >&2
    exit 1
  fi

  if curl -fsS "$API_BASE_URL/healthz" >/dev/null 2>&1; then
    break
  fi

  if [[ "$attempt" == "30" ]]; then
    echo "postgres API server did not answer /healthz at $API_BASE_URL" >&2
    sed -n '1,160p' "$SERVER_LOG" >&2
    exit 1
  fi

  sleep 1
done

curl -fsS "$API_BASE_URL/healthz" >/dev/null
curl -fsS "$API_BASE_URL/readyz" >"$TMP_DIR/readyz.json"
curl -fsS "$API_BASE_URL/metrics" >"$TMP_DIR/metrics-before.json"
curl -fsS \
  -X POST "$API_BASE_URL/agent-jobs" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: $SMOKE_IDEMPOTENCY_KEY" \
  --data '{"kind":"incident_triage","instruction":"Smoke test the Postgres API runtime surface"}' \
  >"$TMP_DIR/admission.json"
curl -fsS "$API_BASE_URL/metrics" >"$TMP_DIR/metrics-after.json"

python3 - \
  "$TMP_DIR/readyz.json" \
  "$TMP_DIR/metrics-before.json" \
  "$TMP_DIR/admission.json" \
  "$TMP_DIR/metrics-after.json" <<'PY'
import json
import sys


def load_json(path: str) -> dict:
    with open(path, "r", encoding="utf-8") as handle:
        value = json.load(handle)
    if not isinstance(value, dict):
        raise SystemExit(f"{path} did not contain a JSON object")
    return value


def queue_from(value: dict, source: str) -> dict:
    queue = value.get("queue")
    if not isinstance(queue, dict):
        raise SystemExit(f"{source} missing queue object")
    for key in (
        "pending",
        "running",
        "succeeded",
        "failed",
        "dead",
        "cancelled",
    ):
        count = queue.get(key)
        if not isinstance(count, int) or count < 0:
            raise SystemExit(f"{source} queue.{key} must be a non-negative integer")
    age = queue.get("oldest_pending_age_seconds")
    if age is not None and (not isinstance(age, int) or age < 0):
        raise SystemExit(f"{source} queue.oldest_pending_age_seconds must be null or non-negative")
    return queue


readyz = load_json(sys.argv[1])
metrics_before = load_json(sys.argv[2])
admission = load_json(sys.argv[3])
metrics_after = load_json(sys.argv[4])

if readyz.get("status") != "ready":
    raise SystemExit("/readyz did not report ready")

before_queue = queue_from(metrics_before, "/metrics before admission")
after_queue = queue_from(metrics_after, "/metrics after admission")
queue_from(readyz, "/readyz")

if admission.get("status") != "pending":
    raise SystemExit("admitted job did not return pending status")
if not isinstance(admission.get("job_id"), str) or not admission["job_id"]:
    raise SystemExit("admitted job did not return a job_id")

if after_queue["pending"] < before_queue["pending"] + 1:
    raise SystemExit("pending queue count did not increase after API admission")
PY

echo "postgres API smoke passed: $API_BASE_URL"
