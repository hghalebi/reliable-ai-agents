#!/usr/bin/env python3
"""Check structural coherence across the Reliable AI Agents mdBook.

The prose, pedagogy, and objective gates check local chapter quality. This
checker protects the book spine: numbered chapters listed in SUMMARY.md should
use matching H1 titles and appear in the main cross-book maps that help readers
move from failure to concept to evidence.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BOOK_SRC = ROOT / "books" / "postgres-rig-agent-jobs" / "src"
SUMMARY = BOOK_SRC / "SUMMARY.md"

SUMMARY_CHAPTER_RE = re.compile(r"^- \[([0-9][^\]]+)\]\(\./([^)]+)\)")

REQUIRED_SECTION_HEADINGS = (
    "## What You Will Learn",
    "## Chapter Thread",
    "## Production Failure",
    "## Plain Version",
    "## Formal Definition",
    "## Testing Strategy",
    "## Observability Strategy",
    "## Operational Checklist",
    "## Summary",
    "## Further Reading and Sources",
)

CROSS_BOOK_MAPS = (
    BOOK_SRC / "36-chapter-checkpoints.md",
    BOOK_SRC / "42-concept-review-retrieval-practice.md",
    BOOK_SRC / "43-concept-dependency-graph.md",
    BOOK_SRC / "chapter-card-pack.md",
    BOOK_SRC / "49-formal-definition-ledger.md",
    BOOK_SRC / "failure-first-learning-map.md",
)


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def numbered_chapters() -> list[tuple[str, Path]]:
    chapters: list[tuple[str, Path]] = []
    for line in SUMMARY.read_text(encoding="utf-8").splitlines():
        match = SUMMARY_CHAPTER_RE.match(line)
        if match:
            title, file_name = match.groups()
            chapters.append((title, BOOK_SRC / file_name))
    return chapters


def main() -> int:
    failures: list[str] = []
    chapters = numbered_chapters()

    if len(chapters) < 30:
        failures.append("SUMMARY.md exposes too few numbered chapters for the production book spine")

    for title, path in chapters:
        if not path.is_file():
            failures.append(f"SUMMARY.md points to missing chapter: {rel(path)}")
            continue

        text = path.read_text(encoding="utf-8")
        first_heading = next(
            (line.strip() for line in text.splitlines() if line.startswith("# ")),
            "",
        )
        expected_heading = f"# {title}"
        if first_heading != expected_heading:
            failures.append(
                f"{rel(path)} H1 `{first_heading}` does not match SUMMARY title `{expected_heading}`"
            )

        missing_sections = [heading for heading in REQUIRED_SECTION_HEADINGS if heading not in text]
        if missing_sections:
            failures.append(
                f"{rel(path)} missing required teaching sections: {', '.join(missing_sections)}"
            )

    for map_path in CROSS_BOOK_MAPS:
        if not map_path.is_file():
            failures.append(f"missing cross-book map: {rel(map_path)}")
            continue

        text = map_path.read_text(encoding="utf-8")
        for title, _ in chapters:
            if title not in text:
                failures.append(f"{rel(map_path)} missing numbered chapter title `{title}`")

    if failures:
        print("book coherence check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("book coherence check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
