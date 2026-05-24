#!/usr/bin/env python3
"""Check the public chapter structure without private authoring files.

The full local pedagogy gate can use private quality notes. This gate is the
public, repository-contained contract: the published book must keep its
chapter ladder, sources, exercises, summaries, and production-review sections.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BOOK_SRC = ROOT / "books" / "postgres-rig-agent-jobs" / "src"
SUMMARY = BOOK_SRC / "SUMMARY.md"

SUMMARY_LINK_RE = re.compile(r"\[[^\]]+\]\((?:\./)?([^)\s#?]+\.md)(?:#[^)]*)?\)")
SOURCE_LINK_RE = re.compile(
    r"^- \[[^\]]+\]\(\./31-credible-resources-further-reading\.md#[^)]+\)",
    re.MULTILINE,
)

MIN_SOURCE_LINKS = 3
MAX_SOURCE_LINKS = 7

SOURCE_SECTION_EXEMPTIONS = {
    "cover.md",
    "31-credible-resources-further-reading.md",
}

DEEP_CHAPTERS = {
    "00b-system-model-and-notation.md",
    "00c-design-principles.md",
    "00d-production-scope-trade-offs.md",
    "01-problem.md",
    "02-mental-model.md",
    "02b-guarantees-failure-semantics.md",
    "03-postgres-ledger.md",
    "04-rust-domain-model.md",
    "04b-typed-composition-lens.md",
    "05-worker-loop.md",
    "06-rig-boundary.md",
    "07-running-system-locally.md",
    "08-production-hardening.md",
    "09-failure-modes.md",
    "10-capstone.md",
    "11-real-postgres-store.md",
    "12-idempotency-side-effects.md",
    "13-leases-heartbeats-cancellation.md",
    "14-retry-backoff-dead-letters.md",
    "15-observability-slos.md",
    "16-human-approval-policy.md",
    "17-testing-production-agents.md",
    "18-deployment-operations.md",
    "19-running-for-years.md",
    "20-final-production-blueprint.md",
    "20a-agent-handoffs-multi-agent-coordination.md",
    "20b-worked-production-scenario.md",
    "21-slis-slos-error-budgets.md",
    "22-capacity-backpressure-provider-quotas.md",
    "23-runbooks.md",
    "24-incident-response-postmortems.md",
    "25-release-engineering.md",
    "26-toil-automation-ownership.md",
    "27-evaluation-behavior-reliability.md",
    "27b-agent-memory-retrieval-retention.md",
    "28-security-abuse-trust-boundaries.md",
    "28b-data-protection-retention-privacy.md",
    "28c-tenant-isolation-multi-tenant-agents.md",
    "29-disaster-recovery-continuity.md",
    "30-reliability-maturity-model.md",
    "30b-scaling-paths-after-postgres-first.md",
    "30c-temporal-after-postgres-first.md",
    "30d-kafka-after-postgres-first.md",
}

DEEP_REQUIRED_HEADINGS = (
    "What You Will Learn",
    "Chapter Thread",
    "Motivation",
    "Plain Version",
    "What You Already Know",
    "Focus Cue",
    "Production Artifact",
    "Implementation Map",
    "Operator Question",
    "Runtime Walkthrough",
    "Acceptance Gate",
    "Micro-Lesson",
    "Formal Definition",
    "What Can Fail",
    "Production Contract",
    "Progressive Hardening Path",
    "Testing Strategy",
    "Observability Strategy",
    "Security and Safety Considerations",
    "Operational Checklist",
    "Exercises",
    "Self-Check",
    "Summary",
    "Changed Understanding",
    "Further Reading and Sources",
)

TINY_SECTION_HEADINGS = (
    "Tiny Example",
    "Tiny Incident",
    "Tiny Scenario",
)

MENTAL_MODEL_SECTION_HEADINGS = (
    "Mental Model",
    "Intuition",
    "Schema",
    "The Core Types",
    "Execution Guarantee",
    "Correctness Boundary",
    "Final Architecture",
    "Blueprint",
    "Evidence Table",
    "Trust Boundaries",
)

FAILURE_FIRST_DEEP_CHAPTERS = {
    chapter for chapter in DEEP_CHAPTERS if not chapter.startswith("00")
}


def display(path: Path) -> str:
    return str(path.relative_to(ROOT))


def summary_pages() -> list[str]:
    if not SUMMARY.is_file():
        return []
    text = SUMMARY.read_text(encoding="utf-8")
    pages: list[str] = []
    seen: set[str] = set()
    for match in SUMMARY_LINK_RE.finditer(text):
        page = match.group(1)
        if page not in seen:
            seen.add(page)
            pages.append(page)
    return pages


def heading_line(text: str, heading: str) -> int | None:
    pattern = re.compile(rf"^## {re.escape(heading)}\s*$", re.MULTILINE)
    match = pattern.search(text)
    if match is None:
        return None
    return text[: match.start()].count("\n") + 1


def require_heading_order(path: Path, text: str, headings: tuple[str, ...], failures: list[str]) -> None:
    previous_line = 0
    for heading in headings:
        line = heading_line(text, heading)
        if line is None:
            failures.append(f"{display(path)} missing required heading: ## {heading}")
            continue
        if line < previous_line:
            failures.append(f"{display(path)} heading appears out of order: ## {heading}")
        previous_line = line


def check_sources(path: Path, text: str, failures: list[str]) -> None:
    if path.name in SOURCE_SECTION_EXEMPTIONS:
        return

    if heading_line(text, "Further Reading and Sources") is None:
        failures.append(f"{display(path)} missing ## Further Reading and Sources")
        return

    source_count = len(SOURCE_LINK_RE.findall(text.split("## Further Reading and Sources", 1)[1]))
    if not (MIN_SOURCE_LINKS <= source_count <= MAX_SOURCE_LINKS):
        failures.append(
            f"{display(path)} must have {MIN_SOURCE_LINKS}-{MAX_SOURCE_LINKS} appendix source links; found {source_count}"
        )


def check_deep_chapter(path: Path, text: str, failures: list[str]) -> None:
    require_heading_order(path, text, DEEP_REQUIRED_HEADINGS, failures)

    if path.name in FAILURE_FIRST_DEEP_CHAPTERS and heading_line(text, "Production Failure") is None:
        failures.append(f"{display(path)} missing failure-first heading: ## Production Failure")

    if not any(heading_line(text, heading) is not None for heading in TINY_SECTION_HEADINGS):
        failures.append(f"{display(path)} must include a tiny example, incident, or scenario heading")

    if not any(heading_line(text, heading) is not None for heading in MENTAL_MODEL_SECTION_HEADINGS):
        failures.append(f"{display(path)} must include a mental-model, schema, blueprint, evidence, or boundary heading")


def main() -> int:
    failures: list[str] = []
    pages = summary_pages()

    if not pages:
        failures.append("SUMMARY.md does not list public book pages")

    for page in pages:
        path = BOOK_SRC / page
        if not path.is_file():
            failures.append(f"SUMMARY.md references missing page: {page}")
            continue

        text = path.read_text(encoding="utf-8")
        check_sources(path, text, failures)

        if page in DEEP_CHAPTERS:
            check_deep_chapter(path, text, failures)

    missing_deep = DEEP_CHAPTERS - set(pages)
    for page in sorted(missing_deep):
        failures.append(f"SUMMARY.md missing required main teaching chapter: {page}")

    if failures:
        print("public chapter structure check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("public chapter structure check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
