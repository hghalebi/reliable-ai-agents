#!/usr/bin/env python3
"""Reject non-learner-facing phrases from the public mdBook surface.

The book may teach engineering decisions. It must not describe the private
authoring conversation, implementation process, or assistant workflow that
produced those decisions.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
README = ROOT / "README.md"
BOOK_DIR = ROOT / "books" / "postgres-rig-agent-jobs"
BOOK_SRC = BOOK_DIR / "src"
BOOK_OUTPUT = BOOK_DIR / "book"

PUBLIC_SOURCE_SURFACES = [README]
PUBLIC_SOURCE_SURFACES.extend(sorted(BOOK_SRC.rglob("*.md")))

PUBLIC_SURFACES = list(PUBLIC_SOURCE_SURFACES)
if BOOK_OUTPUT.is_dir():
    for suffix in ("*.html", "*.js", "*.json"):
        PUBLIC_SURFACES.extend(sorted(BOOK_OUTPUT.rglob(suffix)))

PRIVATE_OR_GENERATED_REPO_PATHS = (
    BOOK_DIR / "STYLE_GUIDE.md",
    BOOK_DIR / "QUALITY.md",
    ROOT / ".private",
    ROOT / "private",
    ROOT / "private-notes",
    ROOT / "author-notes",
    ROOT / "implementation-reports",
    ROOT / "reports",
    ROOT / "scripts" / "__pycache__",
)

@dataclass(frozen=True)
class SurfaceRule:
    label: str
    pattern: re.Pattern[str]
    learner_rewrite: str


def phrase_rule(label: str, phrase: str, learner_rewrite: str) -> SurfaceRule:
    return SurfaceRule(
        label=label,
        pattern=re.compile(re.escape(phrase), re.IGNORECASE),
        learner_rewrite=learner_rewrite,
    )


NON_LEARNER_RULES = (
    phrase_rule(
        "private style guide file",
        "STYLE_GUIDE.md",
        "Move authoring standards to private notes; teach the rule directly in learner prose.",
    ),
    phrase_rule(
        "private quality file",
        "QUALITY.md",
        "Move quality standards to private notes; keep only learner-facing acceptance criteria.",
    ),
    phrase_rule(
        "checker implementation mention",
        "check-book-pedagogy.py",
        "Do not name local checker scripts in the book; name the learner behavior being protected.",
    ),
    phrase_rule(
        "checker implementation mention",
        "check-book-objective-coverage.py",
        "Do not name local checker scripts in the book; name the learner behavior being protected.",
    ),
    phrase_rule(
        "checker implementation mention",
        "check-book-prose-quality.py",
        "Do not name local checker scripts in the book; describe the textbook prose standard.",
    ),
    phrase_rule(
        "checker implementation mention",
        "pedagogy checker",
        "Say what the chapter teaches; keep checker mechanics out of learner text.",
    ),
    phrase_rule(
        "checker implementation mention",
        "objective-coverage gate",
        "Say what the requirement proves; keep gate mechanics out of learner text.",
    ),
    phrase_rule(
        "checker implementation mention",
        "objective-coverage gates",
        "Say what the requirement proves; keep gate mechanics out of learner text.",
    ),
    phrase_rule(
        "authoring process wording",
        "manuscript checks",
        "Replace with a learner-facing validation or review activity.",
    ),
    phrase_rule(
        "authoring process wording",
        "local objective coverage",
        "Replace with the durable production concept being covered.",
    ),
    phrase_rule(
        "authoring process wording",
        "source-hygiene scripts",
        "Replace with a public-safe validation command or omit.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "Goal for Codex",
        "Move assistant instructions to private notes.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "active thread goal",
        "Move assistant instructions to private notes.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "implementation report",
        "Move implementation reports to private notes; keep only learner-facing explanations.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "conversation with the user",
        "Move conversation references to private notes.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "as requested by the user",
        "Rewrite as a direct technical reason.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "Codex should",
        "Move assistant instructions to private notes.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "Codex will",
        "Move assistant instructions to private notes.",
    ),
    phrase_rule(
        "process release-note wording",
        "The current edition",
        "Rewrite as a stable learner-facing statement.",
    ),
    phrase_rule(
        "process release-note wording",
        "Each main chapter now also includes",
        "Rewrite as a direct chapter contract for the reader.",
    ),
    phrase_rule(
        "process release-note wording",
        "Failure-drill examples now include",
        "Rewrite as direct learner-facing failure-drill guidance.",
    ),
    phrase_rule(
        "process release-note wording",
        "This workspace is",
        "Rewrite as a public project or book statement.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "this pass",
        "Rewrite as durable book content, not a change-log note.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "we added",
        "Rewrite as a stable description of what the reader will learn.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "I added",
        "Move implementation notes to private memory.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "private note",
        "Keep private-note references out of learner-facing content.",
    ),
    phrase_rule(
        "assistant workflow wording",
        "authoring workflow",
        "Move authoring workflow notes to private memory.",
    ),
    SurfaceRule(
        label="assistant workflow wording",
        pattern=re.compile(r"\b(?:you|user)\s+(?:asked|requested|wanted)\b", re.IGNORECASE),
        learner_rewrite="Rewrite as the technical motivation, not as a conversation reference.",
    ),
    SurfaceRule(
        label="assistant workflow wording",
        pattern=re.compile(r"\b(?:as|per)\s+your\s+request\b", re.IGNORECASE),
        learner_rewrite="Rewrite as direct textbook prose.",
    ),
    SurfaceRule(
        label="assistant workflow wording",
        pattern=re.compile(r"\b(?:in|during)\s+this\s+(?:turn|session|chat|conversation)\b", re.IGNORECASE),
        learner_rewrite="Move session/process references to private memory.",
    ),
    SurfaceRule(
        label="assistant workflow wording",
        pattern=re.compile(r"\b(?:this|that)\s+(?:task|prompt)\s+(?:asks|asked|requires|required)\b", re.IGNORECASE),
        learner_rewrite="Rewrite as the engineering problem the chapter solves.",
    ),
    SurfaceRule(
        label="assistant workflow wording",
        pattern=re.compile(r"\b(?:assistant|chatgpt|codex)\s+(?:response|prompt|thread|conversation)\b", re.IGNORECASE),
        learner_rewrite="Move assistant-process wording to private notes.",
    ),
)

SOURCE_ONLY_NON_LEARNER_PHRASES = (
    "TODO",
    "todo",
    "placeholder",
)


def line_for_offset(text: str, offset: int) -> tuple[int, str]:
    line_number = text.count("\n", 0, offset) + 1
    line_start = text.rfind("\n", 0, offset) + 1
    line_end = text.find("\n", offset)
    if line_end == -1:
        line_end = len(text)
    return line_number, text[line_start:line_end].strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--report",
        action="store_true",
        help="print contextual findings; exit status is still non-zero when leaks are present",
    )
    args = parser.parse_args()

    failures: list[str] = []

    for path in PRIVATE_OR_GENERATED_REPO_PATHS:
        if path.exists():
            failures.append(
                f"{path.relative_to(ROOT)} must not be present in the public repository surface"
            )

    for path in ROOT.rglob("*.pyc"):
        failures.append(
            f"{path.relative_to(ROOT)} must not be present in the public repository surface"
        )

    for path in PUBLIC_SURFACES:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for rule in NON_LEARNER_RULES:
            for match in rule.pattern.finditer(text):
                line_number, line = line_for_offset(text, match.start())
                failures.append(
                    f"{path.relative_to(ROOT)}:{line_number}: "
                    f"{rule.label}: {match.group(0)!r}. {rule.learner_rewrite}"
                )
                if args.report:
                    failures.append(f"    {line}")

    for path in PUBLIC_SOURCE_SURFACES:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for phrase in SOURCE_ONLY_NON_LEARNER_PHRASES:
            if phrase in text:
                failures.append(
                    f"{path.relative_to(ROOT)} contains unfinished source marker: {phrase}"
                )

    if failures:
        print("public mdBook surface check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("public mdBook surface check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
