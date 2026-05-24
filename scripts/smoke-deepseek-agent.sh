#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_MANIFEST="$ROOT_DIR/examples/postgres-rig-agent-jobs/Cargo.toml"
TMP_DIR="$(mktemp -d)"
OUTPUT_FILE="$TMP_DIR/deepseek-agent.out"

: "${DEEPSEEK_API_KEY:?DEEPSEEK_API_KEY is required for the DeepSeek agent smoke test}"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

SMOKE_INSTRUCTION="${SMOKE_INSTRUCTION:-Analyze a failed deployment. Return one safe next step and require human approval before any external action.}"

cargo run --quiet \
  --manifest-path "$CRATE_MANIFEST" \
  --features rig-agent \
  --bin deepseek-agent-demo \
  -- "$SMOKE_INSTRUCTION" \
  >"$OUTPUT_FILE"

python3 - "$OUTPUT_FILE" <<'PY'
import json
import re
import sys
from pathlib import Path


output = Path(sys.argv[1]).read_text(encoding="utf-8")

for required in (
    "worker outcome: Succeeded",
    "job status: Succeeded",
    "result:",
    "events:",
    "AgentStarted",
    "AgentSucceeded",
):
    if required not in output:
        raise SystemExit(f"DeepSeek smoke output missing required evidence: {required}")

match = re.search(r"result:\s*(\{.*?\})\s*events:", output, flags=re.DOTALL)
if not match:
    raise SystemExit("DeepSeek smoke output did not contain a JSON result object")

result = json.loads(match.group(1))
if not isinstance(result.get("summary"), str) or not result["summary"].strip():
    raise SystemExit("DeepSeek smoke result.summary must be a non-empty string")
if not isinstance(result.get("next_action"), str) or not result["next_action"].strip():
    raise SystemExit("DeepSeek smoke result.next_action must be a non-empty string")
if result.get("approval") not in {"required", "not_required"}:
    raise SystemExit("DeepSeek smoke result.approval must be required or not_required")
PY

echo "DeepSeek agent smoke passed"
