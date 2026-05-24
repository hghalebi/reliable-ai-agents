#!/usr/bin/env python3
"""Write public build metadata into the generated mdBook artifact."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path


def git_value(root: Path, *args: str) -> str | None:
    try:
        value = subprocess.check_output(
            ("git", *args),
            cwd=root,
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        return None
    return value or None


def main() -> int:
    if len(sys.argv) != 3:
        print(
            "usage: write-public-build-info.py <repo-root> <book-output-dir>",
            file=sys.stderr,
        )
        return 2

    root = Path(sys.argv[1]).resolve()
    book_output = Path(sys.argv[2]).resolve()
    book_output.mkdir(parents=True, exist_ok=True)

    revision = os.environ.get("GITHUB_SHA") or git_value(root, "rev-parse", "HEAD")
    ref = os.environ.get("GITHUB_REF_NAME") or git_value(root, "branch", "--show-current")

    build_info = {
        "artifact": "mdbook",
        "book": "Reliable AI Agents",
        "generated_at": datetime.now(UTC).isoformat(timespec="seconds"),
        "github_run_id": os.environ.get("GITHUB_RUN_ID"),
        "public_ci": "scripts/check-public-mdbook-ci.sh",
        "source": "books/postgres-rig-agent-jobs",
        "source_ref": ref,
        "source_revision": revision,
    }

    target = book_output / "build-info.json"
    target.write_text(json.dumps(build_info, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote public build metadata: {target.relative_to(root)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
