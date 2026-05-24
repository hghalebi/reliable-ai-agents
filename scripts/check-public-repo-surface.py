#!/usr/bin/env python3
"""Check the public book repository surface.

The public repo may contain generated folders after a local build, but those
folders must be ignored before publication. This gate rejects private folders
and verifies that generated or local-only artifacts have explicit `.gitignore`
coverage.
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GITIGNORE = ROOT / ".gitignore"

ALLOWED_TOP_LEVEL = {
    ".git",
    ".github",
    ".gitignore",
    "books",
    "examples",
    "README.md",
    "scripts",
}

OPTIONAL_PUBLIC_DOCS = {
    "CITATION.cff",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "LICENSE",
    "LICENSE.md",
    "SECURITY.md",
}

LOCAL_IGNORED_TOP_LEVEL = {
    ".DS_Store",
    ".idea",
}

FORBIDDEN_DIRECTORY_NAMES = {
    ".private",
    "author-notes",
    "implementation-reports",
    "private",
    "private-notes",
    "reports",
}

FORBIDDEN_FILE_SUFFIXES = {
    ".local.md",
    ".pyc",
}

REQUIRED_GITIGNORE_PATTERNS = {
    ".env",
    "*.log",
    "__pycache__/",
    "*.pyc",
    ".idea/",
    ".private/",
    "private/",
    "private-notes/",
    "author-notes/",
    "implementation-reports/",
    "reports/",
    "*.local.md",
    "books/*/book/",
    "examples/*/target/",
    "examples/*/.env",
}


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def main() -> int:
    failures: list[str] = []

    if not GITIGNORE.is_file():
        failures.append("missing .gitignore for public repository hygiene")
        gitignore_patterns: set[str] = set()
    else:
        gitignore_patterns = {
            line.strip()
            for line in GITIGNORE.read_text(encoding="utf-8").splitlines()
            if line.strip() and not line.strip().startswith("#")
        }

    for pattern in sorted(REQUIRED_GITIGNORE_PATTERNS - gitignore_patterns):
        failures.append(f".gitignore missing required public-surface pattern: {pattern}")

    allowed = ALLOWED_TOP_LEVEL | OPTIONAL_PUBLIC_DOCS
    for entry in sorted(ROOT.iterdir()):
        name = entry.name
        if name in allowed:
            continue
        if name in LOCAL_IGNORED_TOP_LEVEL and f"{name}/" in gitignore_patterns:
            continue
        failures.append(f"{rel(entry)} is not part of the public book repository surface")

    for path in sorted(ROOT.rglob("*")):
        if any(part in LOCAL_IGNORED_TOP_LEVEL for part in path.relative_to(ROOT).parts):
            continue
        if path.name in FORBIDDEN_DIRECTORY_NAMES:
            failures.append(f"{rel(path)} is a private/non-learner-facing folder")
        if path.is_file() and path.suffix in FORBIDDEN_FILE_SUFFIXES:
            failures.append(f"{rel(path)} is a local/generated file that must not publish")

    if failures:
        print("public repository surface check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("public repository surface check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
