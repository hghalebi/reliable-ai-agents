#!/usr/bin/env python3
"""Check that production micro-drills point to real artifacts and tests."""

from __future__ import annotations

import re
import shlex
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MICRO_DRILLS = ROOT / "books" / "postgres-rig-agent-jobs" / "src" / "production-micro-drills.md"
READINESS = ROOT / "scripts" / "check-book-readiness.sh"

EXPECTED_DRILLS = (
    "Durable intake",
    "Idempotency key",
    "Typed input",
    "Job claim",
    "Heartbeat",
    "Retry decision",
    "Timeout policy",
    "Cancellation request",
    "Dead letter",
    "Tool contract",
    "Approval gate",
    "Human escalation",
    "Side-effect receipt",
    "Compensation action",
    "Agent handoff",
    "Trace id",
    "Cost and capacity guard",
    "Evaluation receipt",
    "Release gate",
    "Job-kind readiness review",
    "Job-kind launch packet",
    "Security boundary",
    "Injection and exfiltration guard",
    "Agent memory",
    "Credential lifecycle review",
    "Restore replay",
    "Failure drill",
    "Fault-tolerance review",
    "Temporal adoption decision",
    "Kafka adoption decision",
    "Data protection review",
    "Maintenance review",
)


@dataclass(frozen=True)
class DrillArtifactRow:
    drill: str
    artifacts: tuple[str, ...]
    fast_check: str


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def section(text: str, heading: str, next_heading: str) -> str:
    start_marker = f"## {heading}"
    end_marker = f"## {next_heading}"
    if start_marker not in text:
        fail(f"{MICRO_DRILLS.relative_to(ROOT)} missing section: {start_marker}")
    body = text.split(start_marker, 1)[1]
    if end_marker not in body:
        fail(f"{MICRO_DRILLS.relative_to(ROOT)} missing section after {start_marker}: {end_marker}")
    return body.split(end_marker, 1)[0]


def code_spans(text: str) -> tuple[str, ...]:
    return tuple(re.findall(r"`([^`]+)`", text))


def parse_artifact_index(text: str) -> list[DrillArtifactRow]:
    table = section(text, "Exact Artifact Index", "Seven-Minute Drill")
    rows: list[DrillArtifactRow] = []

    for line in table.splitlines():
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        if stripped.startswith("| Drill ") or stripped.startswith("| ---"):
            continue

        parts = [part.strip() for part in stripped.strip("|").split("|")]
        if len(parts) != 3:
            fail(f"malformed micro-drill artifact row: {line}")

        drill, start_here, fast_check = parts
        artifacts = tuple(
            span
            for span in code_spans(start_here)
            if span.startswith("examples/") or span.startswith("books/")
        )
        checks = code_spans(fast_check)
        if len(checks) != 1:
            fail(f"{drill}: fast check cell must contain exactly one command")

        rows.append(DrillArtifactRow(drill=drill, artifacts=artifacts, fast_check=checks[0]))

    return rows


def check_drill_inventory(rows: list[DrillArtifactRow]) -> None:
    found = [row.drill for row in rows]
    expected = list(EXPECTED_DRILLS)
    missing = sorted(set(expected) - set(found))
    extra = sorted(set(found) - set(expected))
    duplicates = sorted(name for name in set(found) if found.count(name) > 1)

    if found != expected or missing or extra or duplicates:
        details = []
        if found != expected:
            details.append("drill order changed")
        if missing:
            details.append(f"missing: {', '.join(missing)}")
        if extra:
            details.append(f"extra: {', '.join(extra)}")
        if duplicates:
            details.append(f"duplicates: {', '.join(duplicates)}")
        fail(f"micro-drill artifact index inventory drift: {'; '.join(details)}")


def check_artifacts_exist(rows: list[DrillArtifactRow]) -> None:
    for row in rows:
        if not row.artifacts:
            fail(f"{row.drill}: artifact index row must name at least one source artifact")

        for artifact in row.artifacts:
            path = ROOT / artifact
            if not path.is_file():
                fail(f"{row.drill}: referenced artifact does not exist: {artifact}")


def run_cargo_test(command: str) -> None:
    args = shlex.split(command)
    if args[:2] != ["cargo", "test"]:
        fail(f"unsupported micro-drill cargo command: {command}")

    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=120,
    )
    output = proc.stdout
    selected_tests = [
        int(match.group(1))
        for match in re.finditer(r"running\s+([0-9]+)\s+tests?", output)
    ]

    if proc.returncode != 0:
        print(output, file=sys.stderr)
        fail(f"micro-drill cargo command failed: {command}")

    if not selected_tests or max(selected_tests) == 0:
        print(output, file=sys.stderr)
        fail(f"micro-drill cargo command selected no tests: {command}")


def check_fast_checks(rows: list[DrillArtifactRow]) -> None:
    for row in rows:
        command = row.fast_check
        if command.startswith("cargo test "):
            run_cargo_test(command)
            continue

        if command == "RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh":
            if not READINESS.is_file():
                fail("local Postgres readiness command points to a missing script")
            continue

        fail(f"{row.drill}: unsupported fast-check command: {command}")


def main() -> None:
    if not MICRO_DRILLS.is_file():
        fail(f"missing micro-drill appendix: {MICRO_DRILLS.relative_to(ROOT)}")

    text = MICRO_DRILLS.read_text(encoding="utf-8")
    rows = parse_artifact_index(text)
    check_drill_inventory(rows)
    check_artifacts_exist(rows)
    check_fast_checks(rows)

    print("micro-drill artifact coverage check passed")


if __name__ == "__main__":
    main()
