#!/usr/bin/env python3
"""Check public terminology for the Reliable AI Agents book project."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PUBLIC_PATHS = (
    ROOT / "README.md",
    ROOT / "books" / "postgres-rig-agent-jobs" / "book.toml",
    ROOT / "books" / "postgres-rig-agent-jobs" / "src",
    ROOT / "examples" / "postgres-rig-agent-jobs" / "src",
)

FORBIDDEN_TERMS = (
    "Andrew Ng",
    "andrew ng",
    "Durable AI Agent Jobs",
    "Windmill",
    "windmill",
    "mdBook course",
    "the course",
    "this course",
    "used by the course",
    "chat session",
    "marketing copy",
)

CANONICAL_TERMS = (
    "Reliable AI Agents",
    "agent job",
    "job kind",
    "durable state",
    "typed boundaries",
    "provider boundary",
    "event timeline",
    "system model",
    "design principles",
    "principle map",
    "production case studies",
    "concept review",
    "concept dependency graph",
    "production evidence packets",
    "retrieval practice",
    "evidence chain",
    "readiness scorecard",
)


def public_files() -> list[Path]:
    files: list[Path] = []
    for path in PUBLIC_PATHS:
        if path.is_file():
            files.append(path)
        elif path.is_dir():
            files.extend(sorted(path.rglob("*.md")))
            files.extend(sorted(path.rglob("*.rs")))
            files.extend(sorted(path.rglob("*.toml")))
    return sorted(set(files))


def main() -> int:
    failures: list[str] = []
    corpus_parts: list[str] = []

    for path in public_files():
        text = path.read_text(encoding="utf-8")
        corpus_parts.append(text)
        for line_number, line in enumerate(text.splitlines(), start=1):
            for term in FORBIDDEN_TERMS:
                if term in line:
                    failures.append(
                        f"{path.relative_to(ROOT)}:{line_number}: forbidden public term `{term}`"
                    )

    corpus = "\n".join(corpus_parts)
    for term in CANONICAL_TERMS:
        if term not in corpus:
            failures.append(f"canonical term missing from public corpus: `{term}`")

    if failures:
        print("book terminology check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("book terminology check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
