#!/usr/bin/env python3
"""Check that the public GitHub Pages workflow builds the mdBook artifact."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github" / "workflows" / "mdbook-pages.yml"
PUBLIC_CI = ROOT / "scripts" / "check-public-mdbook-ci.sh"

REQUIRED_PHRASES = (
    "actions/checkout@v6",
    "actions/configure-pages@v6",
    "cargo install mdbook --version 0.5.2 --locked",
    "bash scripts/check-public-mdbook-ci.sh",
    "examples/postgres-rig-agent-jobs/**",
    "scripts/check-pages-workflow.py",
    "scripts/check-public-chapter-structure.py",
    "scripts/check-public-repo-surface.py",
    "scripts/check-rust-boundary-types.py",
    "scripts/write-public-build-info.py",
    "actions/upload-pages-artifact@v5",
    "path: books/postgres-rig-agent-jobs/book",
    "actions/deploy-pages@v5",
    "pages: write",
    "id-token: write",
    "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true",
    "github-pages",
)

PUBLIC_CI_REQUIRED_PHRASES = (
    "python3 scripts/check-public-repo-surface.py",
    "python3 scripts/check-public-chapter-structure.py",
    "python3 scripts/check-book-code-contract.py",
    "python3 scripts/check-rust-boundary-types.py",
    "cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --all-features",
    "mdbook test",
    "mdbook build",
    "python3 scripts/check-public-mdbook-surface.py",
)


def main() -> int:
    if not WORKFLOW.is_file():
        print(
            f"missing public GitHub Pages workflow: {WORKFLOW.relative_to(ROOT)}",
            file=sys.stderr,
        )
        return 1
    if not PUBLIC_CI.is_file():
        print(
            f"missing public mdBook CI script: {PUBLIC_CI.relative_to(ROOT)}",
            file=sys.stderr,
        )
        return 1

    text = WORKFLOW.read_text(encoding="utf-8")
    failures = [
        f"{WORKFLOW.relative_to(ROOT)} missing required workflow phrase: {phrase}"
        for phrase in REQUIRED_PHRASES
        if phrase not in text
    ]
    public_ci_text = PUBLIC_CI.read_text(encoding="utf-8")
    failures.extend(
        f"{PUBLIC_CI.relative_to(ROOT)} missing required public CI phrase: {phrase}"
        for phrase in PUBLIC_CI_REQUIRED_PHRASES
        if phrase not in public_ci_text
    )

    if failures:
        print("public GitHub Pages workflow check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("public GitHub Pages workflow check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
