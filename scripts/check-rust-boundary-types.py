#!/usr/bin/env python3
"""Reject raw primitive leakage from public Rust domain boundaries.

The book teaches "raw outside, typed inside". Database rows, HTTP DTOs, and
provider DTOs may use storage/transport-friendly raw shapes. Domain structs and
public behavior should expose explicit types instead.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = ROOT / "examples" / "postgres-rig-agent-jobs" / "src"

RAW_FIELD_RE = re.compile(
    r"\b(?:String|bool|usize|u64|i64|f64|serde_json::Value|Value|Vec\s*<|HashMap\s*<)\b"
)
RAW_DIRECT_PARAM_RE = re.compile(
    r":\s*(?:String|&str|bool|usize|u64|i64|f64|serde_json::Value|Value|Vec\s*<|HashMap\s*<)"
)
PUBLIC_STRUCT_RE = re.compile(
    r"pub struct\s+(?P<name>\w+)(?:<[^>{]+>)?\s*\{(?P<body>.*?)\n\}",
    re.S,
)
PUBLIC_FN_RE = re.compile(
    r"pub(?:\s+async)?\s+fn\s+(?P<name>\w+)\s*\((?P<params>[^)]*)\)",
    re.S,
)

BOUNDARY_STRUCT_NAME_RE = re.compile(
    r"^(?:Db|Raw|.*Dto$|.*Request$|.*Response$|.*Config$|.*ErrorResponse$|ApiState$)"
)
CONSTRUCTOR_OR_ACCESSOR_RE = re.compile(
    r"^(?:new|from_|try_from_|saturating_from_|as_|into_|get$|value$|seconds$|zero$|increment$)"
)


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def production_text(path: Path) -> str:
    text = path.read_text(encoding="utf-8")
    return text.split("#[cfg(test)]", 1)[0]


def code_before_comment(line: str) -> str:
    return line.split("//", 1)[0]


def check_public_struct_fields(path: Path, text: str, failures: list[str]) -> None:
    for match in PUBLIC_STRUCT_RE.finditer(text):
        name = match.group("name")
        if BOUNDARY_STRUCT_NAME_RE.match(name):
            continue

        body = match.group("body")
        start_line = text[: match.start()].count("\n") + 1
        for offset, line in enumerate(body.splitlines(), start=1):
            code = code_before_comment(line).strip().rstrip(",")
            if not code.startswith("pub "):
                continue
            if RAW_FIELD_RE.search(code):
                failures.append(
                    f"{rel(path)}:{start_line + offset}: {name} exposes raw public field: {code}"
                )


def check_public_function_parameters(path: Path, text: str, failures: list[str]) -> None:
    for match in PUBLIC_FN_RE.finditer(text):
        name = match.group("name")
        if CONSTRUCTOR_OR_ACCESSOR_RE.match(name):
            continue

        params = " ".join(code_before_comment(match.group("params")).split())
        if not params or params in {"&self", "&mut self", "self"}:
            continue

        if RAW_DIRECT_PARAM_RE.search(params):
            line_number = text[: match.start()].count("\n") + 1
            failures.append(
                f"{rel(path)}:{line_number}: public function {name} accepts raw boundary parameter(s): {params}"
            )


def main() -> int:
    failures: list[str] = []

    for path in sorted(SRC_DIR.rglob("*.rs")):
        text = production_text(path)
        check_public_struct_fields(path, text, failures)
        check_public_function_parameters(path, text, failures)

    if failures:
        print("rust boundary type check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("rust boundary type check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
