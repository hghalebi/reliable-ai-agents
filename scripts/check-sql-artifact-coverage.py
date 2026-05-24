#!/usr/bin/env python3
"""Check that SQL artifacts stay tied to Rust, docs, and runbook evidence."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SQL_DIR = ROOT / "examples" / "postgres-rig-agent-jobs" / "sql"
SQL_REGISTRY = ROOT / "examples" / "postgres-rig-agent-jobs" / "src" / "sql.rs"
README = ROOT / "README.md"
BOOK_SRC = ROOT / "books" / "postgres-rig-agent-jobs" / "src"
COMPANION_CODE_PATH = BOOK_SRC / "45-companion-code-reading-path.md"
EVIDENCE_MAP = BOOK_SRC / "37-implementation-evidence-map.md"
TRACEABILITY = BOOK_SRC / "48-production-requirement-traceability.md"


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def sql_file_names() -> list[str]:
    return sorted(path.name for path in SQL_DIR.glob("*.sql"))


def registry_file_names(text: str) -> list[str]:
    return re.findall(r'SqlFileName::new\("([^"]+\.sql)"\)', text)


def include_str_file_names(text: str) -> list[str]:
    return re.findall(r'include_str!\("\.\./sql/([^"]+\.sql)"\)', text)


def docs_text() -> str:
    parts = [README.read_text(encoding="utf-8")]
    parts.extend(path.read_text(encoding="utf-8") for path in sorted(BOOK_SRC.glob("*.md")))
    return "\n".join(parts)


def require_same_inventory(label: str, expected: list[str], actual: list[str]) -> None:
    expected_set = set(expected)
    actual_set = set(actual)
    missing = sorted(expected_set - actual_set)
    extra = sorted(actual_set - expected_set)
    duplicates = sorted(name for name in actual_set if actual.count(name) > 1)

    if missing or extra or duplicates:
        details = []
        if missing:
            details.append(f"missing: {', '.join(missing)}")
        if extra:
            details.append(f"extra: {', '.join(extra)}")
        if duplicates:
            details.append(f"duplicates: {', '.join(duplicates)}")
        fail(f"{label} SQL artifact inventory drift: {'; '.join(details)}")


def main() -> None:
    sql_files = sql_file_names()
    registry = SQL_REGISTRY.read_text(encoding="utf-8")

    if not sql_files:
        fail(f"{SQL_DIR.relative_to(ROOT)} contains no checked SQL files")

    require_same_inventory(
        f"{SQL_REGISTRY.relative_to(ROOT)} include_str",
        sql_files,
        include_str_file_names(registry),
    )
    require_same_inventory(
        f"{SQL_REGISTRY.relative_to(ROOT)} SQL_ARTIFACTS",
        sql_files,
        registry_file_names(registry),
    )

    text = docs_text()
    unmentioned = [name for name in sql_files if name not in text]
    if unmentioned:
        fail(
            "checked SQL files missing from README/mdBook evidence surfaces: "
            + ", ".join(unmentioned)
        )

    required_doc_phrases = {
        COMPANION_CODE_PATH: (
            "sql.rs",
            "SQL_ARTIFACTS",
            "checked SQL registry",
        ),
        EVIDENCE_MAP: (
            "SQL artifact registry",
            "SQL_ARTIFACTS",
            "check-sql-artifact-coverage.py",
        ),
        TRACEABILITY: (
            "Checked SQL artifacts are registered and source-visible.",
            "SQL_ARTIFACTS",
            "check-sql-artifact-coverage.py",
        ),
    }

    for path, phrases in required_doc_phrases.items():
        path_text = path.read_text(encoding="utf-8")
        for phrase in phrases:
            if phrase not in path_text:
                fail(f"{path.relative_to(ROOT)} missing SQL artifact phrase: {phrase}")

    print("SQL artifact coverage check passed")


if __name__ == "__main__":
    main()
