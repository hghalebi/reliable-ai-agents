#!/usr/bin/env python3
"""Classify outline-heavy chapters in the Reliable AI Agents book.

This check is intentionally narrower than the full pedagogy gate. The existing
pedagogy gate proves that each chapter has the required teaching structure. This
check asks a different question: does the chapter give the reader enough
connected prose to feel like a textbook instead of an outline?
"""

from __future__ import annotations

import ast
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BOOK_SRC = ROOT / "books" / "postgres-rig-agent-jobs" / "src"
PEDAGOGY_CHECK = ROOT / "scripts" / "check-book-pedagogy.py"

OUTLINE_LINE_RE = re.compile(r"^\s*(?:[-*+] |\d+\. |\|)")
WORD_RE = re.compile(r"\b[\w'-]+\b")

# First editorial ratchet. These are high-value chapters where outline-like
# copy most weakens the reader's first mental model.
TEXTBOOK_PROSE_PRIORITY_CHAPTERS = {
    "00b-system-model-and-notation.md",
    "01-problem.md",
    "02b-guarantees-failure-semantics.md",
    "04-rust-domain-model.md",
    "05-worker-loop.md",
    "08-production-hardening.md",
    "09-failure-modes.md",
    "10-capstone.md",
    "26-toil-automation-ownership.md",
}

MAX_PRIORITY_OUTLINE_RATIO = 0.50
MIN_PRIORITY_PROSE_RATIO = 0.50
MIN_PRIORITY_WORKED_WALKTHROUGH_WORDS = 180


def deep_chapters() -> set[str]:
    tree = ast.parse(PEDAGOGY_CHECK.read_text(encoding="utf-8"))
    for node in ast.walk(tree):
        if not isinstance(node, ast.Assign):
            continue
        for target in node.targets:
            if isinstance(target, ast.Name) and target.id == "DEEP_PEDAGOGY_CHAPTERS":
                return set(ast.literal_eval(node.value))
    raise RuntimeError("could not load DEEP_PEDAGOGY_CHAPTERS")


def section_after_heading(text: str, heading: str) -> str:
    marker = f"## {heading}"
    if marker not in text:
        return ""
    section = text.split(marker, 1)[1]
    if "\n## " in section:
        section = section.split("\n## ", 1)[0]
    return section


def line_profile(text: str) -> tuple[int, int, int]:
    outline_lines = 0
    prose_lines = 0
    total_lines = 0
    in_code = False

    for raw_line in text.splitlines():
        line = raw_line.strip()
        if line.startswith("```"):
            in_code = not in_code
            continue
        if in_code or line.startswith("{{#include"):
            continue
        if not line or line.startswith("#"):
            continue

        total_lines += 1
        if OUTLINE_LINE_RE.match(line):
            outline_lines += 1
        else:
            prose_lines += 1

    return outline_lines, prose_lines, total_lines


def classify(outline_ratio: float, prose_ratio: float) -> str:
    if outline_ratio <= 0.36 and prose_ratio >= 0.49:
        return "book-quality"
    if outline_ratio <= 0.50 and prose_ratio >= 0.50:
        return "mixed"
    return "outline-heavy"


def main() -> int:
    failures: list[str] = []
    rows: list[tuple[str, str, float, float]] = []

    for chapter_name in sorted(deep_chapters()):
        path = BOOK_SRC / chapter_name
        if not path.is_file():
            failures.append(f"missing deep chapter: {path.relative_to(ROOT)}")
            continue

        text = path.read_text(encoding="utf-8")
        outline_lines, prose_lines, total_lines = line_profile(text)
        if total_lines == 0:
            failures.append(f"{path.relative_to(ROOT)} has no prose content")
            continue

        outline_ratio = outline_lines / total_lines
        prose_ratio = prose_lines / total_lines
        rows.append((chapter_name, classify(outline_ratio, prose_ratio), outline_ratio, prose_ratio))

        if chapter_name in TEXTBOOK_PROSE_PRIORITY_CHAPTERS:
            if outline_ratio > MAX_PRIORITY_OUTLINE_RATIO:
                failures.append(
                    f"{path.relative_to(ROOT)} is still outline-heavy for the priority rewrite pass: "
                    f"outline ratio {outline_ratio:.2f} > {MAX_PRIORITY_OUTLINE_RATIO:.2f}"
                )
            if prose_ratio < MIN_PRIORITY_PROSE_RATIO:
                failures.append(
                    f"{path.relative_to(ROOT)} needs more connected textbook prose: "
                    f"prose ratio {prose_ratio:.2f} < {MIN_PRIORITY_PROSE_RATIO:.2f}"
                )

            walkthrough = section_after_heading(text, "Worked Walkthrough")
            walkthrough_words = len(WORD_RE.findall(walkthrough))
            if walkthrough_words < MIN_PRIORITY_WORKED_WALKTHROUGH_WORDS:
                failures.append(
                    f"{path.relative_to(ROOT)} needs a substantive '## Worked Walkthrough' section "
                    f"with at least {MIN_PRIORITY_WORKED_WALKTHROUGH_WORDS} words"
                )

    print("chapter prose quality profile:")
    for chapter_name, status, outline_ratio, prose_ratio in rows:
        print(
            f"- {chapter_name}: {status} "
            f"(outline={outline_ratio:.2f}, prose={prose_ratio:.2f})"
        )

    if failures:
        print("\nprose quality check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("book prose quality check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
