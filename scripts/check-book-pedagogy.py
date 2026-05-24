#!/usr/bin/env python3
"""Check the Reliable AI Agents book-level pedagogy contract.

The goal is not to mechanically edit prose. The goal is to catch structural
drift that makes chapters read like loose notes instead of a serious technical
book.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
README = ROOT / "README.md"
BOOK_SRC = ROOT / "books" / "postgres-rig-agent-jobs" / "src"
OVERVIEW = BOOK_SRC / "overview.md"
PRIVATE_AUTHORING = Path.home() / ".khowlege" / "reliable-ai-agents-private" / "authoring"
STYLE_GUIDE = PRIVATE_AUTHORING / "STYLE_GUIDE.md"
QUALITY = PRIVATE_AUTHORING / "QUALITY.md"
READINESS_GATE = ROOT / "scripts" / "check-book-readiness.sh"
LEARNING_PATH = BOOK_SRC / "00-how-to-read-this-book.md"
SYSTEM_MODEL = BOOK_SRC / "00b-system-model-and-notation.md"
DESIGN_PRINCIPLES = BOOK_SRC / "00c-design-principles.md"
SCOPE_TRADEOFFS = BOOK_SRC / "00d-production-scope-trade-offs.md"
SUMMARY = BOOK_SRC / "SUMMARY.md"
GLOSSARY = BOOK_SRC / "32-glossary-invariant-index.md"
DESIGN_REVIEW = BOOK_SRC / "33-production-design-review.md"
FAILURE_DRILLS = BOOK_SRC / "34-failure-drills.md"
READINESS_SCORECARD = BOOK_SRC / "35-production-readiness-scorecard.md"
CHAPTER_CHECKPOINTS = BOOK_SRC / "36-chapter-checkpoints.md"
IMPLEMENTATION_EVIDENCE_MAP = BOOK_SRC / "37-implementation-evidence-map.md"
SYSTEM_DIAGRAMS = BOOK_SRC / "38-system-diagrams-timelines.md"
END_TO_END_LABS = BOOK_SRC / "39-end-to-end-labs.md"
PRINCIPLE_MAP = BOOK_SRC / "40-principle-to-chapter-map.md"
CASE_STUDIES = BOOK_SRC / "41-production-case-studies.md"
CONCEPT_REVIEW = BOOK_SRC / "42-concept-review-retrieval-practice.md"
CONCEPT_DEPENDENCY_GRAPH = BOOK_SRC / "43-concept-dependency-graph.md"
PRODUCTION_EVIDENCE_PACKETS = BOOK_SRC / "44-production-evidence-packets.md"
CODE_READING_PATH = BOOK_SRC / "45-companion-code-reading-path.md"
DESIGN_SMELLS = BOOK_SRC / "46-design-smells-failure-mode-index.md"
ROLE_PATHS = BOOK_SRC / "47-reader-role-operating-paths.md"
REQUIREMENT_TRACEABILITY = BOOK_SRC / "48-production-requirement-traceability.md"
FORMAL_DEFINITION_LEDGER = BOOK_SRC / "49-formal-definition-ledger.md"
RUNNING_EVIDENCE_THREAD = BOOK_SRC / "50-running-evidence-thread.md"
OPERATOR_CONTROL_SURFACE = BOOK_SRC / "51-operator-control-surface.md"
MAINTENANCE_CADENCE = BOOK_SRC / "52-maintenance-cadence.md"
ATTENTION_PROTOCOL = BOOK_SRC / "attention-friendly-production-learning.md"
CHAPTER_CARD_PACK = BOOK_SRC / "chapter-card-pack.md"
FIRST_PRODUCTION_DEPLOYMENT_PROOF = BOOK_SRC / "first-production-deployment-proof.md"
PLAIN_LANGUAGE_PRODUCTION_CARDS = BOOK_SRC / "plain-language-production-cards.md"
PRODUCTION_MICRO_DRILLS = BOOK_SRC / "production-micro-drills.md"
PRODUCTION_BUILD_MILESTONES = BOOK_SRC / "production-build-milestones.md"
FAILURE_FIRST_LEARNING_MAP = BOOK_SRC / "failure-first-learning-map.md"
MEMORY_CHAPTER = BOOK_SRC / "27b-agent-memory-retrieval-retention.md"
HANDOFF_CHAPTER = BOOK_SRC / "20a-agent-handoffs-multi-agent-coordination.md"
WORKED_SCENARIO = BOOK_SRC / "20b-worked-production-scenario.md"
SUMMARY_LINK_RE = re.compile(r"\[[^\]]+\]\((?:\./)?([^)\s#?]+\.md)(?:#[^)]*)?\)")
WORD_RE = re.compile(r"\b[\w'-]+\b")
APPENDIX_SOURCE_RE = re.compile(r"^- \[([^\]]+)\]\(https?://[^)]+\)", re.MULTILINE)
CHAPTER_SOURCE_RE = re.compile(
    r"^- \[([^\]]+)\]\(\./31-credible-resources-further-reading\.md#([^)]+)\)(.+)$",
    re.MULTILINE,
)
MIN_DEEP_CHAPTER_WORDS = 500
MIN_PART_OPENER_WORDS = 450
MIN_CHAPTER_SOURCE_COUNT = 3
MAX_CHAPTER_SOURCE_COUNT = 7
MIN_DEEP_LEARNING_GOAL_WORDS = 45
MIN_DEEP_PREREQUISITE_WORDS = 45
MIN_DEEP_PLAIN_VERSION_WORDS = 35
MIN_DEEP_SELF_CHECK_WORDS = 45
MIN_DEEP_TINY_EXAMPLE_WORDS = 60
MIN_DEEP_MOTIVATION_WORDS = 50
MIN_DEEP_SUMMARY_WORDS = 45
MIN_DEEP_CHANGED_UNDERSTANDING_WORDS = 24
MAX_EXPLANATORY_PARAGRAPH_WORDS = 120

PART_OPENERS = {
    "part-i-core-system.md",
    "part-ii-production-engineering.md",
    "part-iii-operating-the-system.md",
    "part-iv-world-class-reliability.md",
}
SOURCE_SECTION_EXEMPTIONS = {
    "SUMMARY.md",
    "cover.md",
    "31-credible-resources-further-reading.md",
}

MENTAL_MODEL_ANCHORS = (
    "intuition",
    "mental model",
    "model",
    "mechanism",
    "schema",
    "example",
    "code path",
    "contract",
    "blueprint",
    "lifecycle",
    "scorecard",
    "pattern",
    "rule",
    "invariant",
    "split",
    "layers",
    "states",
    "capacity",
    "threat model",
    "lease model",
    "guarantee",
    "boundary",
    "run",
    "idempotency",
    "lease",
    "approval",
    "observability",
    "error classification",
    "test",
    "topology",
    "principle",
    "sli",
    "queue health",
    "runbook",
    "readiness",
    "automation",
    "ownership",
    "rpo",
    "rto",
    "restore",
)

PUBLIC_BANNED_PHRASES = (
    "andrew ng",
    "you asked",
    "course",
    "goal for codex",
    "active thread goal",
    "work slice",
    "implementation report",
    "as requested by the user",
    "per your request",
    "user requested",
    "codex should",
    "codex will",
    "conversation with the user",
    "chatgpt response",
)

DEEP_PEDAGOGY_CHAPTERS = {
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

CORE_FAILURE_OPENING_CHAPTERS = {
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
}

PRODUCTION_ENGINEERING_FAILURE_OPENING_CHAPTERS = {
    "11-real-postgres-store.md",
    "12-idempotency-side-effects.md",
    "13-leases-heartbeats-cancellation.md",
    "14-retry-backoff-dead-letters.md",
    "15-observability-slos.md",
    "16-human-approval-policy.md",
}

PRODUCTION_INTEGRATION_FAILURE_OPENING_CHAPTERS = {
    "17-testing-production-agents.md",
    "18-deployment-operations.md",
    "19-running-for-years.md",
    "20-final-production-blueprint.md",
    "20a-agent-handoffs-multi-agent-coordination.md",
    "20b-worked-production-scenario.md",
}

OPERATING_FAILURE_OPENING_CHAPTERS = {
    "21-slis-slos-error-budgets.md",
    "22-capacity-backpressure-provider-quotas.md",
    "23-runbooks.md",
    "24-incident-response-postmortems.md",
    "25-release-engineering.md",
    "26-toil-automation-ownership.md",
}

WORLD_CLASS_FAILURE_OPENING_CHAPTERS = {
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

FAILURE_OPENING_CHAPTERS = (
    CORE_FAILURE_OPENING_CHAPTERS
    | PRODUCTION_ENGINEERING_FAILURE_OPENING_CHAPTERS
    | PRODUCTION_INTEGRATION_FAILURE_OPENING_CHAPTERS
    | OPERATING_FAILURE_OPENING_CHAPTERS
    | WORLD_CLASS_FAILURE_OPENING_CHAPTERS
)

MAIN_SCAFFOLD_CHAPTERS = DEEP_PEDAGOGY_CHAPTERS

EXPECTED_DEEP_SECTION_ORDER = (
    ("what you will learn", "What You Will Learn"),
    ("chapter thread", "Chapter Thread"),
    ("motivation", "Motivation"),
    ("plain version", "Plain Version"),
    ("what you already know", "What You Already Know"),
    ("focus cue", "Focus Cue"),
    ("production artifact", "Production Artifact"),
    ("implementation map", "Implementation Map"),
    ("operator question", "Operator Question"),
    ("runtime walkthrough", "Runtime Walkthrough"),
    ("acceptance gate", "Acceptance Gate"),
    ("micro-lesson", "Micro-Lesson"),
    ("tiny ", "Tiny Example or Tiny Incident"),
    ("formal definition", "Formal Definition"),
    ("what can fail", "What Can Fail"),
    ("production contract", "Production Contract"),
    ("progressive hardening path", "Progressive Hardening Path"),
    ("testing strategy", "Testing Strategy"),
    ("observability strategy", "Observability Strategy"),
    ("security and safety considerations", "Security and Safety Considerations"),
    ("operational checklist", "Operational Checklist"),
    ("exercises", "Exercises"),
    ("self-check", "Self-Check"),
    ("summary", "Summary"),
    ("changed understanding", "Changed Understanding"),
    ("further reading and sources", "Further Reading and Sources"),
)


def authored_chapters() -> list[Path]:
    return sorted(
        path
        for path in BOOK_SRC.glob("[0-9]*.md")
        if not re.match(r"^(3[1-9]|4[0-9])-", path.name)
    )


def headings(text: str) -> list[str]:
    return [
        line.strip().lower().removeprefix("##").strip()
        for line in text.splitlines()
        if line.startswith("## ")
    ]


def contains_summary_heading(chapter_headings: list[str]) -> bool:
    return any(
        heading == "summary" or heading.endswith("summary")
        for heading in chapter_headings
    )


def section_after_heading(text: str, heading: str, stop_headings: tuple[str, ...]) -> str:
    marker = f"## {heading}"
    if marker not in text:
        return ""

    section = text.split(marker, 1)[1]
    for stop_heading in stop_headings:
        stop_marker = f"## {stop_heading}"
        if stop_marker in section:
            section = section.split(stop_marker, 1)[0]
            break
    return section


def contains_mental_model_anchor(chapter_headings: list[str]) -> bool:
    return any(
        anchor in heading
        for heading in chapter_headings
        for anchor in MENTAL_MODEL_ANCHORS
    )


def expected_section_index(chapter_headings: list[str], expected: str) -> int | None:
    for index, heading in enumerate(chapter_headings):
        if expected.endswith(" "):
            if heading.startswith(expected):
                return index
            continue
        if heading == expected:
            return index
    return None


def normalized_inline(text: str) -> str:
    return " ".join(text.split())


def display_path(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def explanatory_paragraphs(text: str) -> list[tuple[int, str]]:
    """Return prose paragraphs outside code, tables, headings, and lists."""

    paragraphs: list[tuple[int, str]] = []
    in_fence = False
    start_line = 0
    buffer: list[str] = []

    def flush() -> None:
        nonlocal start_line, buffer
        if buffer:
            paragraphs.append((start_line, " ".join(buffer)))
            start_line = 0
            buffer = []

    for line_number, line in enumerate(text.splitlines(), start=1):
        if line.startswith("```"):
            in_fence = not in_fence
            flush()
            continue
        if in_fence:
            continue

        stripped = line.strip()
        if (
            not stripped
            or stripped.startswith("#")
            or stripped.startswith("|")
            or stripped.startswith("- ")
            or re.match(r"^\d+\.", stripped)
        ):
            flush()
            continue

        if not buffer:
            start_line = line_number
        buffer.append(stripped)

    flush()
    return paragraphs


def check_required_file(path: Path, failures: list[str]) -> None:
    if not path.is_file():
        failures.append(f"missing required book control file: {display_path(path)}")


def check_quality_standard(failures: list[str]) -> None:
    if not QUALITY.is_file():
        failures.append(f"missing quality standard: {display_path(QUALITY)}")
        return

    text = QUALITY.read_text(encoding="utf-8")
    required_phrases = (
        "scripts/check-rust-production-hygiene.py",
        "scripts/check-book-terminology.py",
        "scripts/check-book-links.py",
        "canonical technical",
        "Appendix P",
        "design smell",
        "production symptom",
        "corrective invariant",
        "Appendix X should keep a prefilled chapter-card pack",
        "one concept, one artifact, one proof, and one",
        "attention support, not a shortcut around",
        "external sources concentrated in the credible-resources",
        "explicit reading rationale",
        "RUN_EXTERNAL_LINK_CHECK=1 python3 scripts/check-book-links.py",
        "state -> actor -> transition -> evidence -> invariant",
        "Main teaching chapters should start with `What You Will Learn`",
        "small learning contract",
        "what the reader will explain, what they will inspect, what they will verify",
        "Learning goals must use simple action language.",
        "Main teaching chapters must include a `Chapter Thread` after the learning",
        "Read this chapter as one link in the production",
        "`Builds on`, `Adds`, and `Prepares`",
        "one connected production argument",
        "Main teaching chapters must also use `What You Already Know` as a prerequisite",
        "Start with these anchors:",
        "This chapter adds:",
        "Main teaching chapters must include a `Focus Cue` after the prerequisite",
        "Keep three things in view:",
        "`Move`",
        "`Proof`",
        "Main teaching chapters must include a `Production Artifact` after the focus",
        "Build or inspect this artifact before moving on:",
        "`Artifact`, `Why it matters`, and `Done when`",
        "deployable row, type, query, test, runbook, policy record, or evidence",
        "Main teaching chapters must include an `Implementation Map` after the",
        "Use this map when you move from",
        "`Primary surface`, `State transition`,",
        "Rust modules",
        "Main teaching chapters must include an `Operator Question` after the",
        "Before you ship this idea, answer",
        "`Question`, `Evidence to inspect`,",
        "production decision",
        "Main teaching chapters must include a `Runtime Walkthrough` after the operator",
        "Follow the concept as one runtime pass:",
        "`Trigger`, `Action`, `Persistence`, and `Check`",
        "mechanism move from input to durable evidence",
        "Main teaching chapters must include an `Acceptance Gate` after the runtime",
        "Do not move on until this minimum evidence",
        "`Minimum evidence`, `Validation path`, and `Stop if`",
        "deployable proof",
        "Main teaching chapters must include a `Micro-Lesson` after the acceptance gate",
        "Use this five-line",
        "`pain`, `rule`, `tiny",
        "simplest accurate version while still pointing to production",
        "Main teaching chapters must end with a `Self-Check` active-recall drill",
        "Recall:",
        "Explain:",
        "Apply:",
        "Evidence:",
        "Main teaching chapters must include a `Changed Understanding` section after",
        "Before this chapter",
        "After this chapter",
        "reader names the mental-model shift",
        "Motivation sections must start from production pain, name what breaks without the concept, and explain why the mechanism belongs in a reliable agent system.",
        "Main teaching chapters must include a `Plain Version` after the motivation",
        "Read this as the simple version:",
        "`Simple rule`, `Why it matters`, and `What to watch`",
        "simplest accurate version before prerequisite recall",
        "Main teaching chapters must also make the failure path explicit.",
        "Main teaching chapters must include a local `Formal Definition` section",
        "chapter-specific rather than generic placeholders",
        "Main teaching chapters must include an operational checklist and exercises.",
        "Operational Checklist sections must be chapter-specific rather than generic state/boundary/failure scaffolding.",
        "Exercises sections must be chapter-specific rather than generic idempotency/Postgres/Rust/negative-test prompts.",
        "Summary sections must name the chapter-specific mental model, invariant, and evidence to inspect.",
        "Changed Understanding sections must be chapter-specific rather than a generic",
        "which production artifact keeps that new judgment",
        "Main teaching chapters must also include separate testing, observability, and",
        "Testing Strategy sections must be chapter-specific rather than generic",
        "three-level test scaffolding",
        "Observability Strategy sections must be chapter-specific rather than generic",
        "trace/event/runbook scaffolding",
        "Security and Safety Considerations sections must be chapter-specific rather",
        "generic raw-outside typed-inside scaffolding",
        "Privacy and data-protection requests should be durable evidence-backed work:",
        "redaction, erasure, export, and retention-review requests need owner",
        "Credential lifecycle work should be durable evidence-backed work without",
        "secret references need owner, rotation due date",
        "simple language without lowering the technical bar",
        "low working-memory load",
        "Use small action, fast feedback when practice begins",
        "small action",
        "run or inspect one concrete check",
        "attention support and a production habit",
        "Use production micro-drills when a full lab is too large.",
        "one action, one check, one proof sentence, and one stop rule",
        "Use `now / next / done` when a section becomes large.",
        "`Now` names the one",
        "`Next` names one concrete action.",
        "`Done` names the production evidence",
        "Keep read, build, and operate modes separate",
        "Reading means explaining one concept.",
        "Building means changing one artifact.",
        "Operating means proving one behavior.",
        "Tiny examples must be mentally executable.",
        "`transition:`",
        "`invariant:`",
        "Do not simplify away Rust, Postgres, Rig, idempotency, leases,",
        "Main teaching chapters must teach major code and reliability concepts",
        "problem, tiny example, mechanism",
        "What Can",
        "Progressive Hardening Path rows must be",
        "chapter-specific rather than reusable rationale scaffolding",
        "durable before intelligent",
        "typed before clever",
        "recovery must be practiced",
        "operating envelope",
        "The formal definition ledger must keep intuition connected to precision.",
        "state, actor, transition",
        "panic-style fallible calls",
        "String",
        "untyped boxed errors",
        "stub panic macros",
        "worker lifecycle observable with structured `tracing` events",
        "rejects inline ignored Rust snippets",
        "come from `{{#include ...}}` source excerpts",
        "rejects inline SQL fences",
        "examples/postgres-rig-agent-jobs/sql",
        "Runbook `psql` commands are also checked",
        "inline `-c` query strings",
        "Postgres API server through `/healthz`, `/readyz`, `/metrics`, and admission",
        "operator diagnostics plus pause/resume controls",
        "Each chapter should have a local `Further Reading and Sources` section.",
        "name three to seven sources most relevant to that chapter",
        "must map to an external Appendix A source entry",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{display_path(QUALITY)} missing required quality phrase: {phrase}"
            )


def check_style_guide(failures: list[str]) -> None:
    if not STYLE_GUIDE.is_file():
        failures.append(f"missing style guide: {display_path(STYLE_GUIDE)}")
        return

    text = STYLE_GUIDE.read_text(encoding="utf-8")
    required_phrases = (
        "## Book Progression",
        "## System Model",
        "## Design Principles",
        "## Core Editorial Doctrine",
        "## Terminology Canon",
        "## Attention-Friendly Teaching",
        "The book teaches engineering transformations, not library features.",
        "from raw input to trusted domain data",
        "from model text to validated agent intent",
        "from chat loop to durable agent run",
        "from tool call to permissioned side effect",
        "from retry to idempotent execution",
        "from memory to governed state",
        "from logs to traces, metrics, operation events, and audit evidence",
        "from demo script to production workflow",
        "The model may guess. The system must know.",
        "recognition -> understanding -> implementation -> judgment",
        "naive demo -> failure -> intuition -> typed model -> minimal implementation -> production hardening -> tests and evals -> operational judgment",
        "Start chapters from a concrete failure or pain",
        "Keep one major idea per section.",
        "End serious chapters with a changed-understanding summary.",
        "Tests are executable explanations.",
        "durable -> typed -> owned -> isolated -> idempotent -> observable -> versioned -> mature",
        "state -> actor -> transition -> evidence -> invariant",
        "durable before intelligent",
        "typed before clever",
        "recovery must be practiced",
        "Use simple language without lowering the technical bar.",
        "Keep paragraphs short enough for low working-memory load.",
        "ending with a `setup`, `transition`, `evidence`, and `invariant` simulation",
        "Tiny examples should be mentally executable.",
        "Start each teaching chapter with a small learning contract: explain,",
        "Chapter Thread",
        "Do not simplify away Rust, Postgres, Rig, idempotency, leases",
        "The book should be attention-friendly, not technically lighter.",
        "The first page of a chapter should reduce choice.",
        "Use evidence-backed attention supports throughout the book:",
        "Use the micro-lesson rule before dense Rust, SQL, policy, or operations",
        "Use prerequisite repair instead of prerequisite detours",
        "Use the attention restart protocol",
        "Use small action, fast feedback when practice begins",
        "do one small action",
        "run or inspect one concrete check",
        "Use production micro-drills when a full lab is too large.",
        "Use the `now / next / done` rail",
        "`done` names",
        "Keep feedback tied to production evidence",
        "Let readers choose one mode at a time: read to explain one concept",
        "Use faded practice when examples become exercises",
        "Use a plain-language term ladder",
        "plain phrase -> formal term -> production artifact -> proof",
        "artifact and proof must keep",
        "one concept, one artifact, one proof, one pause",
        "Externalize working memory with chapter cards",
        "Give separate code, SQL, and runbook reading protocols",
        "Use a one-new-term rule in dense sections.",
        "Prefer worked examples before open-ended exercises",
        "Use visible checklists and cadence tables",
        "The simplest acceptable explanation is the one that still lets a serious",
        "book, not course",
        "agent job, not generic task",
        "event timeline, not process log as authority",
        "Do not introduce a later production control before the earlier invariant",
        "Include practice surfaces that ask readers to apply concepts before reading",
        "Use chapter checkpoints to force prerequisite recall, mental simulation, and",
        "Use readiness scorecards only when each rating points to concrete evidence",
        "Use implementation evidence maps to connect production controls to exact",
        "Use system diagrams and timelines when a reader must hold multiple production",
        "Use end-to-end labs to move readers from recognition to implementation",
        "Use principle maps to connect transferable rules to chapters",
        "Use production case studies to show how the same invariants become stricter",
        "Use concept review prompts to force recall",
        "Use concept dependency graphs to show what each chapter assumes",
        "Use reader-role paths to help AI engineers",
        "Use production requirement traceability to connect each major book promise",
        "Use formal definition ledgers to compress each main chapter concept",
        "Use running evidence threads to connect separate controls into one production story",
        "Use operator control surfaces to separate read views from typed",
        "Use maintenance cadence appendices to turn long-horizon reliability into daily",
        "durable data-protection work with a queryable review surface",
        "Credential lifecycle work should use secret references",
        "Use production evidence packets to turn readiness",
        "Use companion code reading paths to connect the reader",
        "Use design-smell indexes to teach the reader",
        "Use the chapter-card pack to turn each main chapter into a one-screen",
        "concept, artifact, proof, and operator question",
        "A main teaching chapter should not be only a marker scaffold.",
        "Part openers should be real orientation chapters.",
        "Scope and trade-off chapters should name the operating envelope",
        "Main teaching chapters should activate prior knowledge inside the chapter",
        "Use `What You Already Know` as a prerequisite bridge",
        "list the concrete prior",
        "then use `This chapter adds:`",
        "Use `Focus Cue` as an attention rail",
        "Keep three things in view:",
        "`State`, `Move`, and `Proof`",
        "low-working-memory readers",
        "Use `Production Artifact` as the builder's target.",
        "Build or inspect this artifact before moving on:",
        "name `Artifact`, `Why it",
        "chapter tied to deployable evidence",
        "Use `Implementation Map` as the bridge from concept to code.",
        "Use this map when you move from reading to implementation:",
        "`Primary surface`, `State transition`, and `Evidence path`",
        "Rust, SQL, tests, runbooks, or operational surfaces",
        "Use `Operator Question` as the bridge from implementation to operations.",
        "Before you ship this idea, answer one operational question:",
        "`Question`, `Evidence to inspect`, and `Escalate if`",
        "decision an engineer can make under pressure",
        "Use `Runtime Walkthrough` as the step-by-step execution bridge.",
        "Follow the concept as one runtime pass:",
        "`Trigger`, `Action`,",
        "state move before reading heavier code",
        "Use `Acceptance Gate` as the minimum evidence threshold.",
        "Do not move on until this minimum evidence exists:",
        "`Minimum evidence`, `Validation path`, and `Stop if`",
        "understanding as production readiness",
        "Use `Micro-Lesson` before every dense mechanism.",
        "should appear after the acceptance gate",
        "Name the pain.",
        "Ask for one proof.",
        "Use `Self-Check` as an active-recall drill",
        "recall, explain, apply, and point to evidence",
        "Changed Understanding",
        "before/after mental-model shift",
        "Use the chapter-card pack as the fastest restart surface.",
        "one concept, one artifact, one proof, and one operator",
        "rebuild every dependency in their head",
        "Use the plain-language production cards when a term is the blocker.",
        "one small sentence, one exact",
        "term, one artifact, and one proof sentence",
        "Main teaching chapters should include a `What You Will Learn` section",
        "reader will explain, inspect, and verify",
        "do not reuse generic \"by the end of",
        "Use `Chapter Thread` as the local continuity bridge.",
        "Read this chapter as one link in the production chain:",
        "name `Builds on`",
        "disconnected",
        "Motivation sections should start from production pain, name what breaks without the concept, and explain why the chapter's mechanism belongs in a reliable agent system.",
        "Use `Plain Version` as the simplest accurate bridge.",
        "Read this as the simple version:",
        "`Simple rule`, `Why it matters`, and `What to watch`",
        "simple language before prerequisite recall",
        "Main teaching chapters should include a `What Can Fail` section",
        "Main teaching chapters should include a `Formal Definition` section",
        "The bullets should be chapter-specific, not reusable placeholders.",
        "Main teaching chapters should include a `Progressive Hardening Path`",
        "The table's `What changes`",
        "chapter-specific; do not reuse generic",
        "Main teaching chapters should include a `Testing Strategy`, `Observability",
        "should not reuse generic unit/persistence/regression",
        "specific Rust type, Postgres row or query",
        "should not reuse generic trace/event/runbook",
        "specific structured `tracing` fields",
        "should not reuse generic",
        "chapter's specific",
        "redaction rule",
        "Main teaching chapters should include an `Operational Checklist` section",
        "Operational Checklist sections should be chapter-specific rather than generic state/boundary/failure scaffolding.",
        "Main teaching chapters should include an `Exercises` section",
        "Exercises sections should be chapter-specific rather than generic idempotency/Postgres/Rust/negative-test prompts.",
        "Summary sections should name the chapter-specific mental model, invariant, and evidence to inspect.",
        "They should leave the reader with the smallest durable idea that can guide a production review.",
        "Changed Understanding sections should make the reader's before/after shift explicit.",
        "chapter closes with a remembered judgment",
        "Ignored Rust fences are allowed only for source-backed `{{#include ...}}`",
        "SQL fences must come from `{{#include ...}}` snippets under the companion",
        "Runbook `psql` commands should execute checked SQL files with `-f`",
        "Production Rust shown or included by the book should return typed errors",
        "External references belong in Appendix A",
        "External URLs belong in Appendix A",
        "Each teaching chapter should still end with `Further Reading and Sources`",
        "three to seven relevant Appendix A sources",
        "Each source should have an explicit reading rationale",
        "external Appendix A source entry",
        "Local links, image references,",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{display_path(STYLE_GUIDE)} missing required style-guide phrase: {phrase}"
            )


def check_readiness_gate(failures: list[str]) -> None:
    if not READINESS_GATE.is_file():
        failures.append(f"missing readiness gate: {READINESS_GATE.relative_to(ROOT)}")
        return

    text = READINESS_GATE.read_text(encoding="utf-8")
    required_phrases = (
        "RUN_LIVE_POSTGRES",
        "RUN_LOCAL_POSTGRES",
        "RUN_LIVE_DEEPSEEK",
        "check-book-links.py",
        "requires a running Docker-compatible daemon",
        "smoke-local-postgres.sh",
        "smoke-deepseek-agent.sh",
        "RUN_LIVE_DEEPSEEK=1 requires DEEPSEEK_API_KEY",
        "docker compose",
        "postgres-worker-demo",
        "002_agent_tracking.sql",
        "queue_metrics_by_kind.sql",
        "oldest_pending_job.sql",
        "expired_leases.sql",
        "dead_jobs_by_reason.sql",
        "job_event_timeline.sql",
        "version_compatibility_risks.sql",
        "schema_migration_status.sql",
        "failure_drill_status.sql",
        "running_agent_runs.sql",
        "pending_agent_handoffs.sql",
        "scheduled_retries.sql",
        "waiting_human_approvals.sql",
        "open_human_escalations.sql",
        "failed_tool_calls.sql",
        "side_effect_receipts_by_run.sql",
        "evaluation_receipts_by_version.sql",
        "provider_usage_by_job_kind.sql",
        "job_kind_lifecycle_review.sql",
        "agent_memory_by_scope.sql",
        "denied_authorization_events.sql",
        "sandbox_policy_violations.sql",
        "credential_rotation_review.sql",
        "restore_replay_candidates.sql",
        "storage_pressure_by_table.sql",
        "retention_review_by_surface.sql",
        "outbox_backlog.sql",
        "compensation_backlog.sql",
        "pause_job_kind.sql",
        "resume_job_kind.sql",
        "-v job_id=",
        "-v minimum_payload_schema_version=",
        "-v maximum_payload_schema_version=",
        "-v run_id=",
        "-v memory_scope=",
        "-v kind=",
        "-v reason=",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{READINESS_GATE.relative_to(ROOT)} missing required live-gate phrase: {phrase}"
            )


def check_public_entry_points(failures: list[str]) -> None:
    required = {
        README: (
            "46-design-smells-failure-mode-index.md",
            "48-production-requirement-traceability.md",
            "49-formal-definition-ledger.md",
            "50-running-evidence-thread.md",
            "51-operator-control-surface.md",
            "52-maintenance-cadence.md",
            "first-production-deployment-proof.md",
            "plain-language-production-cards.md",
            "production-micro-drills.md",
            "production-build-milestones.md",
            "failure-first-learning-map.md",
            "runtime environment variables at startup",
            "RUN_LIVE_DEEPSEEK",
            "scripts/smoke-deepseek-agent.sh",
            "design-smell",
            "formal definition ledger",
            "running evidence thread",
            "operator control surface",
            "maintenance cadence",
            "first production deployment proof",
            "chapter-card pack",
            "production micro-drills",
            "production build milestones",
            "failure-first learning map",
        ),
        OVERVIEW: (
            "Appendix P is the design-smell and failure-mode index.",
            "Appendix S is the formal definition ledger.",
            "Appendix T is the running evidence thread.",
            "Appendix U is the operator control surface.",
            "Appendix V is the maintenance cadence.",
            "Appendix W is the attention-friendly learning protocol.",
            "Appendix X is the prefilled chapter-card pack.",
            "Appendix Y is the first production deployment proof.",
            "Appendix Z is the plain-language production card pack.",
            "Appendix AA is the production micro-drill pack.",
            "Appendix AB is the production build milestone ladder.",
            "Appendix AC is the failure-first learning map.",
            "local shortcut",
            "production symptom",
            "corrective invariant",
            "evidence to inspect",
            "production failure, one false shortcut, one surviving invariant",
            "state, actor,",
            "one production story",
            "permission and audit evidence",
            "daily, weekly, monthly, quarterly",
            "concept, artifact, proof, and operator question",
            "one API process, one worker process, one Postgres",
            "seven-minute drills",
            "build, inspect, run, prove",
        ),
    }

    for path, phrases in required.items():
        if not path.is_file():
            failures.append(f"missing public entry point: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        for phrase in phrases:
            if phrase not in text:
                failures.append(
                    f"{path.relative_to(ROOT)} missing public entry-point phrase: {phrase}"
                )


def check_source_backed_code_policy(failures: list[str]) -> None:
    include_re = re.compile(r"\{\{#include\s+([^}:]+)(?::[^}]*)?\}\}")
    rust_src_dir = (ROOT / "examples" / "postgres-rig-agent-jobs" / "src").resolve()
    sql_dir = (ROOT / "examples" / "postgres-rig-agent-jobs" / "sql").resolve()

    for path in sorted(BOOK_SRC.glob("*.md")):
        lines = path.read_text(encoding="utf-8").splitlines()
        in_fence = False
        fence_start = 0
        fence_lang = ""
        block_lines: list[str] = []

        for line_number, line in enumerate(lines, start=1):
            if line.startswith("```"):
                if not in_fence:
                    in_fence = True
                    fence_start = line_number
                    fence_lang = line.strip()[3:].strip()
                    block_lines = []
                    continue

                if fence_lang.startswith("rust") and "ignore" in fence_lang.split(","):
                    block = "\n".join(block_lines)
                    include_matches = list(include_re.finditer(block))
                    if not include_matches:
                        failures.append(
                            f"{path.relative_to(ROOT)}:{fence_start}: ignored Rust fences must include checked source via {{#include ...}}"
                        )
                    for include_match in include_matches:
                        include_path = include_match.group(1)
                        resolved = (path.parent / include_path).resolve()
                        if not resolved.is_file():
                            failures.append(
                                f"{path.relative_to(ROOT)}:{fence_start}: included Rust source does not exist: {include_path}"
                            )
                        try:
                            resolved.relative_to(rust_src_dir)
                        except ValueError:
                            failures.append(
                                f"{path.relative_to(ROOT)}:{fence_start}: ignored Rust include must come from the checked companion crate src: {include_path}"
                            )

                if fence_lang == "sql":
                    block = "\n".join(block_lines)
                    include_matches = list(include_re.finditer(block))
                    if not include_matches:
                        failures.append(
                            f"{path.relative_to(ROOT)}:{fence_start}: SQL fences must include checked companion SQL via {{#include ...}}"
                        )
                    for include_match in include_matches:
                        include_path = include_match.group(1)
                        resolved = (path.parent / include_path).resolve()
                        if not resolved.is_file():
                            failures.append(
                                f"{path.relative_to(ROOT)}:{fence_start}: included SQL source does not exist: {include_path}"
                            )
                        try:
                            resolved.relative_to(sql_dir)
                        except ValueError:
                            failures.append(
                                f"{path.relative_to(ROOT)}:{fence_start}: SQL include must come from the checked companion SQL directory: {include_path}"
                            )

                if fence_lang == "bash":
                    block = "\n".join(block_lines)
                    if "psql" in block:
                        if re.search(r"(^|\s)-c\s+[\"']", block):
                            failures.append(
                                f"{path.relative_to(ROOT)}:{fence_start}: runbook psql commands must use checked SQL files with -f, not inline -c query strings"
                            )
                        if "examples/postgres-rig-agent-jobs/sql/" not in block:
                            failures.append(
                                f"{path.relative_to(ROOT)}:{fence_start}: psql commands must point at checked companion SQL files"
                            )
                        for sql_path in re.findall(r"-f\s+([^\s\\]+\.sql)", block):
                            resolved = (ROOT / sql_path).resolve()
                            if not resolved.is_file():
                                failures.append(
                                    f"{path.relative_to(ROOT)}:{fence_start}: psql command references missing SQL file: {sql_path}"
                                )
                            try:
                                resolved.relative_to(sql_dir)
                            except ValueError:
                                failures.append(
                                    f"{path.relative_to(ROOT)}:{fence_start}: psql command must use companion SQL file: {sql_path}"
                                )

                in_fence = False
                fence_start = 0
                fence_lang = ""
                block_lines = []
                continue

            if in_fence:
                block_lines.append(line)


def check_summary_source_closure(failures: list[str]) -> None:
    if not SUMMARY.is_file():
        failures.append(f"missing mdBook summary: {SUMMARY.relative_to(ROOT)}")
        return

    summary_text = SUMMARY.read_text(encoding="utf-8")
    linked_paths = {
        (BOOK_SRC / match.group(1)).resolve()
        for match in SUMMARY_LINK_RE.finditer(summary_text)
    }
    source_paths = {
        path.resolve()
        for path in BOOK_SRC.glob("*.md")
        if path.name != "SUMMARY.md"
    }

    if not linked_paths:
        failures.append(f"{SUMMARY.relative_to(ROOT)} does not link any markdown source files")

    for missing in sorted(path for path in linked_paths if not path.is_file()):
        failures.append(
            f"{SUMMARY.relative_to(ROOT)} links missing source file: {missing.relative_to(ROOT)}"
        )

    for unlisted in sorted(source_paths - linked_paths):
        failures.append(
            f"{unlisted.relative_to(ROOT)} exists but is not reachable from {SUMMARY.relative_to(ROOT)}"
        )


def check_chapter_source_sections(failures: list[str]) -> None:
    if not SUMMARY.is_file():
        failures.append(f"missing mdBook summary: {SUMMARY.relative_to(ROOT)}")
        return

    appendix_text = (BOOK_SRC / "31-credible-resources-further-reading.md").read_text(
        encoding="utf-8"
    )
    appendix_source_names = set(APPENDIX_SOURCE_RE.findall(appendix_text))
    appendix_anchors = set()
    for line in appendix_text.splitlines():
        if not line.startswith("## "):
            continue
        anchor = re.sub(r"[^a-z0-9 -]", "", line.removeprefix("## ").lower())
        appendix_anchors.add(anchor.strip().replace(" ", "-"))

    summary_text = SUMMARY.read_text(encoding="utf-8")
    linked_paths = sorted(
        (BOOK_SRC / match.group(1)).resolve()
        for match in SUMMARY_LINK_RE.finditer(summary_text)
    )

    for path in linked_paths:
        if path.name in SOURCE_SECTION_EXEMPTIONS:
            continue
        if not path.is_file():
            continue

        text = path.read_text(encoding="utf-8")
        display = path.relative_to(ROOT)
        if "## Further Reading and Sources" not in text:
            failures.append(
                f"{display} must include a '## Further Reading and Sources' section"
            )
            continue

        section = text.split("## Further Reading and Sources", 1)[1]
        source_refs = list(CHAPTER_SOURCE_RE.finditer(section))
        if not MIN_CHAPTER_SOURCE_COUNT <= len(source_refs) <= MAX_CHAPTER_SOURCE_COUNT:
            failures.append(
                f"{display} source section must cite between {MIN_CHAPTER_SOURCE_COUNT} and {MAX_CHAPTER_SOURCE_COUNT} Appendix A sources"
            )
        for source_ref in source_refs:
            source_name = source_ref.group(1)
            anchor = source_ref.group(2)
            rationale = source_ref.group(3).strip()
            if source_name not in appendix_source_names:
                failures.append(
                    f"{display} source '{source_name}' must match an external Appendix A source entry"
                )
            if anchor not in appendix_anchors:
                failures.append(
                    f"{display} source '{source_name}' points to missing Appendix A anchor: {anchor}"
                )
            if len(WORD_RE.findall(rationale)) < 8 or not rationale.endswith("."):
                failures.append(
                    f"{display} source '{source_name}' must include a sentence explaining why it is relevant"
                )


def check_learning_path(failures: list[str]) -> None:
    if not LEARNING_PATH.is_file():
        failures.append(
            f"missing reader-facing learning path: {LEARNING_PATH.relative_to(ROOT)}"
        )
        return

    text = LEARNING_PATH.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "chapter thread -> pain -> plain version -> micro-lesson -> intuition -> tiny example -> mechanism -> what can fail -> invariant -> progressive hardening -> testing -> observability -> safety -> checklist -> exercises -> summary",
        "## If Focus Is Hard Today",
        "This book is attention-friendly, not technically lighter.",
        "simple language without lowering the technical bar",
        "read the chapter in this short loop",
        "Read the Plain Version.",
        "Read the Chapter Thread.",
        "Read the Focus Cue.",
        "Read the Production Artifact.",
        "Read the Implementation Map.",
        "Read the Operator Question.",
        "Read the Runtime Walkthrough.",
        "Read the Acceptance Gate.",
        "Read the Micro-Lesson.",
        "What state is changing?",
        "What move changes it?",
        "What proof remains after the move?",
        "simple rule, why it matters, and what to watch",
        "what row, type, query, runbook",
        "where the chapter becomes executable",
        "what question an engineer or on-call operator",
        "trigger to action to persistence to check",
        "minimum evidence, validation path, and stop condition",
        "Every main chapter also has a `Micro-Lesson` after the acceptance gate.",
        "pain\nrule\ntiny example\nartifact\nproof",
        "Use the `Chapter Thread` when the book starts to feel like many separate",
        "Builds on\nAdds\nPrepares",
        "Do not skip the production contract or operational checklist.",
        "does not simplify away Rust, Postgres, Rig, idempotency, leases, evaluation, security, or operations",
        "## Prerequisite Repair Map",
        "Use repair, not a long detour.",
        "| Rust newtypes and enums |",
        "| Postgres row locks and leases |",
        "| Rig model and tool boundary |",
        "| SRE vocabulary |",
        "| Agent evaluation |",
        "| Agent security |",
        "| Recovery and replay |",
        "Raw provider output becomes parsed domain output or typed failure.",
        "Restored work resumes, reconciles, or quarantines without guessing.",
        "## Concept Ladder",
        "The chapters are ordered as a dependency ladder.",
        "| 1 | Durable work before intelligence |",
        "| 2 | Typed boundaries before business logic |",
        "| 3 | Ownership before concurrency |",
        "| 4 | Provider isolation before failure policy |",
        "| 5 | Idempotency before side effects |",
        "| 6 | Evidence before operations |",
        "| 7 | Versioning before long-horizon operation |",
        "| 8 | Evaluation, memory, security, and recovery before maturity |",
        "durable -> typed -> owned -> isolated -> idempotent -> observable -> versioned -> mature",
        "## Reader Role Paths",
        "| AI engineer |",
        "| Rust engineer |",
        "| Platform or SRE engineer |",
        "| Security or governance reviewer |",
        "| Founder or product owner |",
        "## Chapter Contracts",
        "Before Part I, read \"System Model And Notation.\"",
        "Then read \"Design Principles.\"",
        "Then read \"Production Scope And Trade-Offs.\"",
        "| System Model And Notation |",
        "| Design Principles |",
        "| Production Scope And Trade-Offs |",
        "when a workflow engine, queue framework, distributed platform, or simple script is the better control",
        "The invariant I need:",
        "The evidence that would prove it:",
        "The smallest test or runbook query I can add:",
        "| 20.1 Agent Handoffs And Multi-Agent Coordination |",
        "| 20.2 Worked Production Scenario |",
        "| 27.5 Agent Memory, Retrieval, And Retention |",
        "| 30.5 Scaling Paths After Postgres-First |",
        "Use Appendix B when a term feels clear but the design still feels fragile.",
        "Appendix C as a production design review",
        "Use Appendix D when you want practice.",
        "Use Appendix E when you need a readiness decision.",
        "Use Appendix F after each chapter.",
        "Use Appendix G when you want to connect an idea to the companion implementation.",
        "Use Appendix H when the moving parts feel scattered.",
        "Use Appendix I when you are ready to change the system itself.",
        "Use Appendix J when you want to transfer the design principles to another",
        "Use Appendix K when you want to see the same system model across different",
        "Use Appendix L after chapters or parts for retrieval practice.",
        "Use Appendix M when the dependency structure feels fuzzy.",
        "Use Appendix N when you need to prove readiness or safety to another engineer.",
        "Use Appendix O when you are ready to read the companion Rust and SQL code",
        "Use Appendix P when a concept feels clear but you want to recognize the broken",
        "Use Appendix Q after the first sequential read when you want a role-specific",
        "Use Appendix R when you want to audit the book against its own production",
        "Use Appendix S when an idea feels intuitive but needs a precise production",
        "Use Appendix T when the book feels like many separate controls.",
        "one incident-triage request across identity",
        "Use Appendix U when you are ready to build a dashboard",
        "typed, permissioned, audited",
        "Use Appendix V when long-running reliability needs a calendar.",
        "daily, weekly, monthly, quarterly",
        "Use Appendix W when focus is the main bottleneck.",
        "one concept\none artifact\none proof\none pause",
        "now: the one concept or artifact I am looking at",
        "next: the one action I will take",
        "done: the proof that lets me stop",
        "## Plain-Language Term Ladder",
        "plain phrase -> formal term -> production artifact -> proof",
        "| work that survives a crash | durable job |",
        "| temporary ownership | lease |",
        "| doing it again safely | idempotency |",
        "| the model asks to act | typed tool request |",
        "| ask a person before risk | human approval gate |",
        "| show what happened | event timeline |",
        "Use Appendix X when you need a fast restart card.",
        "one concept, one artifact, one proof, and one operator question",
        "re-enter the book without rebuilding the whole context",
        "Use Appendix Z when a production term feels too large.",
        "artifact to inspect, and the proof sentence",
        "restore drill, newtype, typestate, and trust boundary",
        "Use Appendix AC when you want the failure-first spine of the book.",
        "production failure, to false shortcut, to invariant",
        "state, actor, transition",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{LEARNING_PATH.relative_to(ROOT)} missing required learning-path phrase: {phrase}"
            )

    for chapter in range(1, 31):
        chapter_marker = f"| {chapter}."
        if chapter_marker not in text:
            failures.append(
                f"{LEARNING_PATH.relative_to(ROOT)} missing chapter contract for chapter {chapter}"
            )

    if SUMMARY.is_file():
        summary = SUMMARY.read_text(encoding="utf-8")
        if "00-how-to-read-this-book.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 00-how-to-read-this-book.md"
            )
        if "00b-system-model-and-notation.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 00b-system-model-and-notation.md"
            )
        if "00c-design-principles.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 00c-design-principles.md"
            )
        if "00d-production-scope-trade-offs.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 00d-production-scope-trade-offs.md"
            )
        for opener in sorted(PART_OPENERS):
            if opener not in summary:
                failures.append(
                    f"{SUMMARY.relative_to(ROOT)} must include part opener: {opener}"
                )
        if "32-glossary-invariant-index.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 32-glossary-invariant-index.md"
            )
        if "33-production-design-review.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 33-production-design-review.md"
            )
        if "34-failure-drills.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 34-failure-drills.md"
            )
        if "35-production-readiness-scorecard.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 35-production-readiness-scorecard.md"
            )
        if "36-chapter-checkpoints.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 36-chapter-checkpoints.md"
            )
        if "37-implementation-evidence-map.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 37-implementation-evidence-map.md"
            )
        if "38-system-diagrams-timelines.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 38-system-diagrams-timelines.md"
            )
        if "39-end-to-end-labs.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 39-end-to-end-labs.md"
            )
        if "40-principle-to-chapter-map.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 40-principle-to-chapter-map.md"
            )
        if "41-production-case-studies.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 41-production-case-studies.md"
            )
        if "42-concept-review-retrieval-practice.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 42-concept-review-retrieval-practice.md"
            )
        if "43-concept-dependency-graph.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 43-concept-dependency-graph.md"
            )
        if "44-production-evidence-packets.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 44-production-evidence-packets.md"
            )
        if "45-companion-code-reading-path.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 45-companion-code-reading-path.md"
            )
        if "46-design-smells-failure-mode-index.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 46-design-smells-failure-mode-index.md"
            )
        if "47-reader-role-operating-paths.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 47-reader-role-operating-paths.md"
            )
        if "48-production-requirement-traceability.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 48-production-requirement-traceability.md"
            )
        if "49-formal-definition-ledger.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 49-formal-definition-ledger.md"
            )
        if "50-running-evidence-thread.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 50-running-evidence-thread.md"
            )
        if "51-operator-control-surface.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 51-operator-control-surface.md"
            )
        if "52-maintenance-cadence.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 52-maintenance-cadence.md"
            )
        if "attention-friendly-production-learning.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include attention-friendly-production-learning.md"
            )
        if "chapter-card-pack.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include chapter-card-pack.md"
            )
        if "first-production-deployment-proof.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include first-production-deployment-proof.md"
            )
        if "plain-language-production-cards.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include plain-language-production-cards.md"
            )
        if "production-micro-drills.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include production-micro-drills.md"
            )
        if "production-build-milestones.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include production-build-milestones.md"
            )
        if "failure-first-learning-map.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include failure-first-learning-map.md"
            )
        if "20b-worked-production-scenario.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 20b-worked-production-scenario.md"
            )
        if "20a-agent-handoffs-multi-agent-coordination.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 20a-agent-handoffs-multi-agent-coordination.md"
            )
        if "30b-scaling-paths-after-postgres-first.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 30b-scaling-paths-after-postgres-first.md"
            )
        if "27b-agent-memory-retrieval-retention.md" not in summary:
            failures.append(
                f"{SUMMARY.relative_to(ROOT)} must include 27b-agent-memory-retrieval-retention.md"
            )
    else:
        failures.append(f"missing mdBook summary: {SUMMARY.relative_to(ROOT)}")


def check_part_openers(failures: list[str]) -> None:
    required_headings = {
        "motivation",
        "what you already know",
        "what this part adds",
        "running example",
        "exit criteria",
        "summary",
    }

    for opener in sorted(PART_OPENERS):
        path = BOOK_SRC / opener
        if not path.is_file():
            failures.append(f"missing part opener: {path.relative_to(ROOT)}")
            continue

        text = path.read_text(encoding="utf-8")
        chapter_headings = set(headings(text))
        word_count = len(WORD_RE.findall(text))
        if word_count < MIN_PART_OPENER_WORDS:
            failures.append(
                f"{path.relative_to(ROOT)} has {word_count} words; part openers need at least {MIN_PART_OPENER_WORDS} words of orientation"
            )

        for heading in sorted(required_headings):
            if heading not in chapter_headings:
                failures.append(
                    f"{path.relative_to(ROOT)} missing required part-opener heading: {heading}"
                )


def check_system_model(failures: list[str]) -> None:
    if not SYSTEM_MODEL.is_file():
        failures.append(
            f"missing system model and notation chapter: {SYSTEM_MODEL.relative_to(ROOT)}"
        )
        return

    text = SYSTEM_MODEL.read_text(encoding="utf-8")
    required_phrases = (
        "some state exists",
        "some actor is allowed to change it",
        "some evidence is recorded",
        "some invariant must still be true afterward",
        "What changed, who was allowed to change it, and what evidence remains?",
        "precondition:",
        "action:",
        "evidence:",
        "invariant:",
        "| `AgentJob` |",
        "| `JobKind` |",
        "| `Lease` |",
        "| `ProviderBoundary` |",
        "| `EvaluationReceipt` |",
        "| `SideEffectReceipt` |",
        "StateBefore",
        "StateAfter",
        "job row",
        "event timeline",
        "provider decision",
        "policy decision",
        "evaluation receipt",
        "side-effect receipt",
        "transient:",
        "permanent:",
        "exhausted:",
        "cancelled:",
        "state -> actor -> transition -> evidence -> invariant",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{SYSTEM_MODEL.relative_to(ROOT)} missing required system-model phrase: {phrase}"
            )


def check_design_principles(failures: list[str]) -> None:
    if not DESIGN_PRINCIPLES.is_file():
        failures.append(
            f"missing design principles chapter: {DESIGN_PRINCIPLES.relative_to(ROOT)}"
        )
        return

    text = DESIGN_PRINCIPLES.read_text(encoding="utf-8")
    required_phrases = (
        "The design principles in this chapter explain why the mechanisms belong",
        "## Principle 1: Durable Before Intelligent",
        "## Principle 2: Typed Before Clever",
        "## Principle 3: Ownership Before Concurrency",
        "## Principle 4: Boundary Before Policy",
        "## Principle 5: Idempotent Before Retried",
        "## Principle 6: Evidence Before Operations",
        "## Principle 7: Evaluation Before Behavior Release",
        "## Principle 8: Approval Is State, Not Conversation",
        "## Principle 9: Release With Old Work In Mind",
        "## Principle 10: Recovery Must Be Practiced",
        "agent_jobs row exists before the worker calls the agent",
        "domain APIs expose JobKind, JobState, WorkerId, RetryDisposition, and version types",
        "SQL predicates require locked_by and locked_until",
        "provider responses convert to AgentResult or typed failure",
        "duplicate enqueue returns the existing job and side-effect replay checks a receipt",
        "runbook queries reconstruct queue health, lease state, dead jobs, and event timeline",
        "prompt and model versions have evaluation receipts before promotion",
        "risky action waits for an approval record",
        "schema, prompt, model, policy, and worker versions are stored with the job",
        "restore drill records RPO, RTO, replay rules, receipt handling, and operator signoff",
        "durable, typed, owned, bounded, idempotent, observable, evaluated, approved,",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{DESIGN_PRINCIPLES.relative_to(ROOT)} missing required design-principle phrase: {phrase}"
            )


def check_scope_tradeoffs(failures: list[str]) -> None:
    if not SCOPE_TRADEOFFS.is_file():
        failures.append(
            f"missing production scope and trade-offs chapter: {SCOPE_TRADEOFFS.relative_to(ROOT)}"
        )
        return

    text = SCOPE_TRADEOFFS.read_text(encoding="utf-8")
    required_phrases = (
        "## What You Already Know",
        "## Where Postgres-First Fits",
        "## When To Choose A Workflow Engine",
        "## When To Use A Queue Or Job Framework",
        "## What This Book Does Not Try To Solve",
        "script:",
        "queue framework:",
        "Postgres-first ledger:",
        "durable workflow engine:",
        "distributed platform:",
        "If workflow semantics are the hardest part, evaluate a workflow engine.",
        "The danger is mistaking a queue for a reliability model.",
        "Architecture choice is serious only when it names the contract:",
        "Which missing invariant would make you move up the ladder?",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{SCOPE_TRADEOFFS.relative_to(ROOT)} missing required scope/trade-off phrase: {phrase}"
            )


def check_objective_bridge_terms(failures: list[str]) -> None:
    required_by_file = {
        BOOK_SRC / "02-mental-model.md": (
            "## Chatbots, Workflows, And Agents",
            "chatbot:",
            "workflow:",
            "agent:",
            "a worker with tools, memory, permissions, and durable state",
        ),
        BOOK_SRC / "06-rig-boundary.md": (
            "## Tool Calling Is A Boundary, Not Magic",
            "model proposes a tool call",
            "Rig gives the agent a clean way to work with models and tools.",
        ),
        BOOK_SRC / "27-evaluation-behavior-reliability.md": (
            "## Golden Datasets",
            "GoldenEvaluationDataset",
            "no behavior version is promoted until the golden dataset has a receipt",
        ),
        BOOK_SRC / "28-security-abuse-trust-boundaries.md": (
            "## Short-Term And Long-Term Memory",
            "short-term memory:",
            "long-term memory:",
            "MemoryHorizon",
        ),
        BOOK_SRC / "20a-agent-handoffs-multi-agent-coordination.md": (
            "## The Production-Grade Concept",
            "handoff is a state machine",
            "source run, source agent, target agent, reason, payload, idempotency key",
            "accepted handoffs create or attach exactly one target job",
            "pending_agent_handoffs.sql",
        ),
        BOOK_SRC / "30b-scaling-paths-after-postgres-first.md": (
            "## Migration Patterns",
            "Do not add infrastructure to escape the state machine.",
            "evidence-preserving migration map",
            "Postgres can remain the system of record",
            "queue_metrics_by_kind.sql",
        ),
        BOOK_SRC / "27b-agent-memory-retrieval-retention.md": (
            "## Retrieval As A Typed Selection",
            "memory is production data, not prompt decoration",
            "which remembered facts are allowed to influence which future action?",
            "MemoryHorizon",
            "agent_memory_by_scope.sql",
        ),
    }

    for path, phrases in required_by_file.items():
        if not path.is_file():
            failures.append(f"missing objective bridge file: {path.relative_to(ROOT)}")
            continue

        text = path.read_text(encoding="utf-8")
        for phrase in phrases:
            if phrase not in text:
                failures.append(
                    f"{path.relative_to(ROOT)} missing objective bridge phrase: {phrase}"
                )


def check_observability_trace_context(failures: list[str]) -> None:
    required_by_file = {
        BOOK_SRC / "15-observability-slos.md": (
            "## Trace Context",
            "trace id:",
            "span id:",
            "TraceContext",
            "trace identifiers are operational correlation data, not arbitrary strings",
            "W3C Trace Context",
        ),
        BOOK_SRC / "23-runbooks.md": (
            "Which trace id and span id connect this row to live telemetry?",
        ),
        BOOK_SRC / "37-implementation-evidence-map.md": (
            "operation events with trace/span ids",
            "malformed trace ids",
        ),
        BOOK_SRC / "45-companion-code-reading-path.md": (
            "TraceId",
            "SpanId",
            "TraceContext",
        ),
    }

    for path, phrases in required_by_file.items():
        if not path.is_file():
            failures.append(f"missing observability trace file: {path.relative_to(ROOT)}")
            continue

        text = path.read_text(encoding="utf-8")
        for phrase in phrases:
            if phrase not in text:
                failures.append(
                    f"{path.relative_to(ROOT)} missing trace-context phrase: {phrase}"
                )


def check_long_horizon_compatibility(failures: list[str]) -> None:
    required_by_file = {
        BOOK_SRC / "19-running-for-years.md": (
            "## Worker Compatibility",
            "WorkerCompatibilityPolicy",
            "AgentJob + WorkerCompatibilityPolicy -> Process | Quarantine(reason)",
            "version_compatibility_risks.sql",
            "storage_pressure_by_table.sql",
            "credential_rotation_review.sql",
            "retention_review_by_surface.sql",
            "data_protection_review.sql",
            "job_kind_lifecycle_review.sql",
            "Which evidence surfaces have open privacy work?",
            "Which credentials have open exposure incidents?",
            "Which evidence surfaces have rows older than 365 days?",
            "Which job kinds still have pending, running, retrying, or human-waiting work?",
            "workers check supported schema ranges before execution",
        ),
        BOOK_SRC / "23-runbooks.md": (
            "## Version Compatibility Risks",
            "minimum_payload_schema_version",
            "maximum_payload_schema_version",
            "Should the job be quarantined, migrated, or handled by a compatible worker?",
            "## Schema Migration Status",
            "schema_migration_status.sql",
            "Which expand, backfill, or contract phase is planned, running, blocked, failed, or recently passed?",
            "## Failure Drill Status",
            "failure_drill_status.sql",
            "## Storage Pressure By Table",
            "storage_pressure_by_table.sql",
            "## Retention Review By Surface",
            "retention_review_by_surface.sql",
            "## Credential Rotation Review",
            "credential_rotation_review.sql",
            "## Data Protection Review",
            "data_protection_review.sql",
            "## Job Kind Lifecycle Review",
            "job_kind_lifecycle_review.sql",
            "Which job kinds are deprecation candidates but not retirement candidates yet?",
        ),
        BOOK_SRC / "25-release-engineering.md": (
            "schema_migration_runs",
            "schema_migration_status.sql",
            "Which migration phase is planned, running, blocked, failed, or recently passed?",
            "How many rows were examined and changed?",
        ),
        BOOK_SRC / "52-maintenance-cadence.md": (
            "Storage pressure",
            "storage_pressure_by_table.sql",
            "Retention review",
            "retention_review_by_surface.sql",
            "Data-protection review",
            "data_protection_review.sql",
            "Credential lifecycle review",
            "credential_rotation_review.sql",
            "Job-kind lifecycle",
            "job_kind_lifecycle_review.sql",
            "Storage maintenance review",
        ),
        BOOK_SRC / "35-production-readiness-scorecard.md": (
            "worker compatibility policy",
            "compatibility-risk query",
            "migration ledger",
            "schema-migration status query",
        ),
        BOOK_SRC / "37-implementation-evidence-map.md": (
            "Worker compatibility boundary",
            "version_compatibility_risks.sql",
            "Schema migration status",
            "schema_migration_status.sql",
            "Failure drill status",
            "failure_drill_status.sql",
            "Storage pressure by table",
            "Retention review by surface",
            "Credential rotation review",
            "credential_rotation_review.sql",
            "Data protection review",
            "data_protection_review.sql",
            "Job kind lifecycle review",
            "job_kind_lifecycle_review.sql",
            "too-old and too-new payload schemas are quarantined",
        ),
        BOOK_SRC / "51-operator-control-surface.md": (
            "## Operator Question Index",
            "Use this index when an incident starts with a question instead of a dashboard.",
            "Which agents are currently running?",
            "Which jobs are stuck?",
            "Which tool calls failed?",
            "Which retries are scheduled?",
            "Which human approvals are waiting?",
            "Which model version produced this output?",
            "Which prompt version was used?",
            "Which side effects happened?",
            "Can we safely replay this step?",
            "Can we prove what happened?",
            "running_agent_runs.sql",
            "operation_events_by_job.sql",
            "oldest_pending_job.sql",
            "expired_leases.sql",
            "running_jobs_past_deadline.sql",
            "dead_jobs_by_reason.sql",
            "failure_history_by_job.sql",
            "job_event_timeline.sql",
            "failed_tool_calls.sql",
            "scheduled_retries.sql",
            "waiting_human_approvals.sql",
            "open_human_escalations.sql",
            "evaluation_receipts_by_version.sql",
            "release_gate_status.sql",
            "side_effect_receipts_by_run.sql",
            "outbox_backlog.sql",
            "restore_replay_candidates.sql",
            "audit_events_by_run.sql",
            "schema_migration_status.sql",
            "job_kind_lifecycle_review.sql",
            "storage_pressure_by_table.sql",
            "retention_review_by_surface.sql",
            "credential_rotation_review.sql",
            "data_protection_review.sql",
            "failure_drill_status.sql",
            "long-horizon views include job-kind lifecycle, storage pressure, retention",
            "data-protection review",
        ),
        BOOK_SRC / "45-companion-code-reading-path.md": (
            "examples/postgres-rig-agent-jobs/src/compatibility.rs",
            "Which pending or running jobs require a different worker version before",
            "Which job kinds are active, deprecation candidates, retirement candidates, or retirement blocked?",
            "Which evidence surfaces have overdue redaction, erasure, export, or retention-review work?",
            "Which credential kinds are due, overdue, exposed, stale, or recently revoked?",
            "compatibility tests prove supported jobs can run",
            "schema_migration_status.sql",
            "Which expand, backfill, or contract migration phase is still open",
            "failure_drill_status.sql",
        ),
        BOOK_SRC / "46-design-smells-failure-mode-index.md": (
            "unsupported payload schema",
            "worker compatibility policy and the compatibility-risk query",
        ),
    }

    for path, phrases in required_by_file.items():
        if not path.is_file():
            failures.append(f"missing compatibility file: {path.relative_to(ROOT)}")
            continue

        text = path.read_text(encoding="utf-8")
        for phrase in phrases:
            if phrase not in text:
                failures.append(
                    f"{path.relative_to(ROOT)} missing compatibility phrase: {phrase}"
                )


def check_human_escalation_boundary(failures: list[str]) -> None:
    required_by_file = {
        BOOK_SRC / "16-human-approval-policy.md": (
            "## Escalation Is Not Approval",
            "approval answers: may this risky action proceed?",
            "escalation answers: which human owner must take responsibility now?",
            "HumanEscalationRecord",
        ),
        BOOK_SRC / "23-runbooks.md": (
            "## Open Human Escalations",
            "open_human_escalations.sql",
            "Which human escalation is open or acknowledged?",
        ),
        BOOK_SRC / "24-incident-response-postmortems.md": (
            "## Escalation Evidence",
            "escalation kind",
            "assigned owner",
        ),
        BOOK_SRC / "32-glossary-invariant-index.md": (
            "| Human escalation |",
            "`human_escalations` stores target, kind, severity, status, owner, and timestamps.",
        ),
        BOOK_SRC / "37-implementation-evidence-map.md": (
            "Human escalation boundary",
            "open_human_escalations.sql",
        ),
        BOOK_SRC / "45-companion-code-reading-path.md": (
            "examples/postgres-rig-agent-jobs/src/escalation.rs",
            "Which human escalation is open or acknowledged?",
        ),
    }

    for path, phrases in required_by_file.items():
        if not path.is_file():
            failures.append(f"missing human-escalation file: {path.relative_to(ROOT)}")
            continue

        text = path.read_text(encoding="utf-8")
        for phrase in phrases:
            if phrase not in text:
                failures.append(
                    f"{path.relative_to(ROOT)} missing human-escalation phrase: {phrase}"
                )


def check_glossary(failures: list[str]) -> None:
    if not GLOSSARY.is_file():
        failures.append(f"missing glossary and invariant index: {GLOSSARY.relative_to(ROOT)}")
        return

    text = GLOSSARY.read_text(encoding="utf-8")
    required_phrases = (
        "term -> mental model -> failure prevented -> production evidence",
        "## Core Concepts",
        "## Misconception Repair Index",
        "## Invariant Index",
        "| Agent job |",
        "| Idempotency key |",
        "| Lease |",
        "| Evaluation receipt |",
        "| Trust boundary |",
        "| The model call is the workflow. |",
        "| Retries make side effects safe. |",
        "| Logs are the audit trail. |",
        "| A prompt can enforce policy. |",
        "| A backup means disaster recovery is solved. |",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{GLOSSARY.relative_to(ROOT)} missing required glossary phrase: {phrase}"
            )


def check_worked_scenario(failures: list[str]) -> None:
    if not WORKED_SCENARIO.is_file():
        failures.append(f"missing worked production scenario: {WORKED_SCENARIO.relative_to(ROOT)}")
        return

    text = WORKED_SCENARIO.read_text(encoding="utf-8")
    required_phrases = (
        "duplicate webhook",
        "provider timeout",
        "approval",
        "side-effect receipt",
        "## Evidence Table",
        "Every risky transition has durable evidence before the next irreversible action.",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{WORKED_SCENARIO.relative_to(ROOT)} missing required scenario phrase: {phrase}"
            )


def check_handoff_chapter(failures: list[str]) -> None:
    if not HANDOFF_CHAPTER.is_file():
        failures.append(f"missing agent handoff chapter: {HANDOFF_CHAPTER.relative_to(ROOT)}")
        return

    text = HANDOFF_CHAPTER.read_text(encoding="utf-8")
    required_phrases = (
        "A handoff is the answer. It is not a chat message.",
        "a handoff transfers responsibility, not permission",
        "HandoffRequest -> DeploymentSafetyJob",
        "accepted handoffs create or attach exactly one target job",
        "pending_agent_handoffs.sql",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{HANDOFF_CHAPTER.relative_to(ROOT)} missing required handoff-chapter phrase: {phrase}"
            )


def check_memory_chapter(failures: list[str]) -> None:
    if not MEMORY_CHAPTER.is_file():
        failures.append(f"missing agent memory chapter: {MEMORY_CHAPTER.relative_to(ROOT)}")
        return

    text = MEMORY_CHAPTER.read_text(encoding="utf-8")
    required_phrases = (
        "memory is production data, not prompt decoration",
        "which remembered facts are allowed to influence which future action?",
        "Retrieval should not mean \"find similar text and trust it.\"",
        "raw memory storage outside",
        "typed memory policy inside",
        "agent_memory_by_scope.sql",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{MEMORY_CHAPTER.relative_to(ROOT)} missing required memory-chapter phrase: {phrase}"
            )


def check_design_review(failures: list[str]) -> None:
    if not DESIGN_REVIEW.is_file():
        failures.append(f"missing production design review: {DESIGN_REVIEW.relative_to(ROOT)}")
        return

    text = DESIGN_REVIEW.read_text(encoding="utf-8")
    required_phrases = (
        "## Design Review Questions",
        "## Failure Injection Prompts",
        "## Evidence Review",
        "## Final Readiness Bar",
        "durable before execution",
        "idempotent before retry",
        "eval before behavior release",
        "restore drill before long-horizon claim",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{DESIGN_REVIEW.relative_to(ROOT)} missing required design-review phrase: {phrase}"
            )


def check_failure_drills(failures: list[str]) -> None:
    if not FAILURE_DRILLS.is_file():
        failures.append(f"missing failure drills appendix: {FAILURE_DRILLS.relative_to(ROOT)}")
        return

    text = FAILURE_DRILLS.read_text(encoding="utf-8")
    required_phrases = (
        "## How to Use These Drills",
        "## Practice Contract",
        "## Durable Drill Evidence",
        "failure_drill_status.sql",
        "## Drill 1: Duplicate Webhook During Provider Latency",
        "## Drill 2: Worker Crash After Model Output",
        "## Drill 3: Permanent Provider Shape Change",
        "## Drill 4: Risky Recommendation Without Approval",
        "## Drill 5: SLO Alert Without Job Evidence",
        "## Drill 6: Restore From a Fifteen-Minute-Old Backup",
        "## Drill 9: Staging Chaos Experiment Kills A Worker",
        "## Drill 10: Tool Injection Tries To Exfiltrate Tenant Data",
        "chaos experiment",
        "Tool Injection",
        "Exfiltrate",
        "Expected reasoning:",
        "state transition",
        "event evidence",
        "retry or stop decision",
        "operator question",
        "next engineering change",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{FAILURE_DRILLS.relative_to(ROOT)} missing required failure-drill phrase: {phrase}"
            )


def check_readiness_scorecard(failures: list[str]) -> None:
    if not READINESS_SCORECARD.is_file():
        failures.append(
            f"missing production readiness scorecard: {READINESS_SCORECARD.relative_to(ROOT)}"
        )
        return

    text = READINESS_SCORECARD.read_text(encoding="utf-8")
    required_phrases = (
        "## How to Use This Scorecard",
        "## Rating Rules",
        "## Job-Kind Scorecard",
        "## Promotion Rule",
        "## Evidence Packet",
        "target:",
        "current evidence:",
        "gap:",
        "next change:",
        "owner:",
        "review date:",
        "missing:",
        "partial:",
        "ready:",
        "not required:",
        "| Durable work |",
        "| Provider boundary |",
        "| Idempotent side effects |",
        "| Behavior evaluation |",
        "| Agent memory |",
        "| Disaster recovery |",
        "| Scaling path |",
        "job kind and target maturity level",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{READINESS_SCORECARD.relative_to(ROOT)} missing required readiness-scorecard phrase: {phrase}"
            )


def check_chapter_checkpoints(failures: list[str]) -> None:
    if not CHAPTER_CHECKPOINTS.is_file():
        failures.append(
            f"missing chapter checkpoints appendix: {CHAPTER_CHECKPOINTS.relative_to(ROOT)}"
        )
        return

    text = CHAPTER_CHECKPOINTS.read_text(encoding="utf-8")
    required_phrases = (
        "## How to Use These Checkpoints",
        "## Front Matter Checkpoints",
        "## Part I Checkpoints",
        "## Part II Checkpoints",
        "## Part III Checkpoints",
        "## Part IV Checkpoints",
        "prerequisite:",
        "mental simulation:",
        "production evidence:",
        "| Chapter | Prerequisite to recall | Mental simulation | Production evidence to name |",
        "| Production Scope And Trade-Offs |",
        "| 20.1 Agent Handoffs And Multi-Agent Coordination |",
        "| 20.2 Worked Production Scenario |",
        "| 27.5 Agent Memory, Retrieval, And Retention |",
        "| 30.5 Scaling Paths After Postgres-First |",
        "If you cannot name the prerequisite",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{CHAPTER_CHECKPOINTS.relative_to(ROOT)} missing required checkpoint phrase: {phrase}"
            )

    for chapter in range(1, 31):
        chapter_marker = f"| {chapter}."
        if chapter_marker not in text:
            failures.append(
                f"{CHAPTER_CHECKPOINTS.relative_to(ROOT)} missing checkpoint for chapter {chapter}"
            )


def check_implementation_evidence_map(failures: list[str]) -> None:
    if not IMPLEMENTATION_EVIDENCE_MAP.is_file():
        failures.append(
            f"missing implementation evidence map: {IMPLEMENTATION_EVIDENCE_MAP.relative_to(ROOT)}"
        )
        return

    text = IMPLEMENTATION_EVIDENCE_MAP.read_text(encoding="utf-8")
    required_phrases = (
        "## How to Use This Map",
        "concept -> implementation artifact -> validation evidence -> operator proof",
        "## Core Domain Evidence",
        "## SQL And Persistence Evidence",
        "## Operational Evidence",
        "## Feature And Readiness Evidence",
        "| Production control | Source artifact | What to inspect | Validation evidence |",
        "| Typed domain boundary |",
        "| Durable job ledger |",
        "| Queue health |",
        "| Real Postgres path |",
        "| Reference discipline |",
        "| Runtime configuration boundary |",
        "scripts/smoke-deepseek-agent.sh",
        "RUN_LIVE_DEEPSEEK=1 ./scripts/check-book-readiness.sh",
        "the companion implementation proves the claim",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{IMPLEMENTATION_EVIDENCE_MAP.relative_to(ROOT)} missing required evidence-map phrase: {phrase}"
            )

    required_paths = (
        "examples/postgres-rig-agent-jobs/Cargo.toml",
        "examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql",
        "examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql",
        "examples/postgres-rig-agent-jobs/sql/dead_jobs_by_reason.sql",
        "examples/postgres-rig-agent-jobs/sql/admit_agent_job.sql",
        "examples/postgres-rig-agent-jobs/sql/resolve_existing_agent_job.sql",
        "examples/postgres-rig-agent-jobs/sql/enqueue_agent_job.sql",
        "examples/postgres-rig-agent-jobs/sql/record_admission_decision.sql",
        "examples/postgres-rig-agent-jobs/sql/expired_leases.sql",
        "examples/postgres-rig-agent-jobs/sql/extend_lease.sql",
        "examples/postgres-rig-agent-jobs/sql/job_event_timeline.sql",
        "examples/postgres-rig-agent-jobs/sql/failure_history_by_job.sql",
        "examples/postgres-rig-agent-jobs/sql/version_compatibility_risks.sql",
        "examples/postgres-rig-agent-jobs/sql/running_agent_runs.sql",
        "examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql",
        "examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql",
        "examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql",
        "examples/postgres-rig-agent-jobs/sql/scheduled_retries.sql",
        "examples/postgres-rig-agent-jobs/sql/waiting_human_approvals.sql",
        "examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql",
        "examples/postgres-rig-agent-jobs/sql/failed_tool_calls.sql",
        "examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql",
        "examples/postgres-rig-agent-jobs/sql/evaluation_receipts_by_version.sql",
        "examples/postgres-rig-agent-jobs/sql/release_gate_status.sql",
        "examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql",
        "examples/postgres-rig-agent-jobs/sql/denied_authorization_events.sql",
        "examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql",
        "examples/postgres-rig-agent-jobs/sql/audit_events_by_run.sql",
        "examples/postgres-rig-agent-jobs/sql/operation_events_by_job.sql",
        "examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql",
        "examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql",
        "examples/postgres-rig-agent-jobs/sql/claim_outbox_events.sql",
        "examples/postgres-rig-agent-jobs/sql/claim_scheduled_jobs.sql",
        "examples/postgres-rig-agent-jobs/sql/complete_scheduled_job.sql",
        "examples/postgres-rig-agent-jobs/sql/mark_outbox_event_published.sql",
        "examples/postgres-rig-agent-jobs/sql/mark_outbox_event_failed.sql",
        "examples/postgres-rig-agent-jobs/sql/fail_or_retry_scheduled_job.sql",
        "examples/postgres-rig-agent-jobs/sql/outbox_backlog.sql",
        "examples/postgres-rig-agent-jobs/sql/approve_compensation_action.sql",
        "examples/postgres-rig-agent-jobs/sql/claim_compensation_actions.sql",
        "examples/postgres-rig-agent-jobs/sql/mark_compensation_succeeded.sql",
        "examples/postgres-rig-agent-jobs/sql/mark_compensation_failed.sql",
        "examples/postgres-rig-agent-jobs/sql/compensation_backlog.sql",
        "examples/postgres-rig-agent-jobs/sql/mark_cancelled.sql",
        "examples/postgres-rig-agent-jobs/sql/mark_succeeded.sql",
        "examples/postgres-rig-agent-jobs/sql/oldest_pending_job.sql",
        "examples/postgres-rig-agent-jobs/sql/pause_job_kind.sql",
        "examples/postgres-rig-agent-jobs/sql/pick_due_job.sql",
        "examples/postgres-rig-agent-jobs/sql/queue_metrics.sql",
        "examples/postgres-rig-agent-jobs/sql/queue_metrics_by_kind.sql",
        "examples/postgres-rig-agent-jobs/sql/recover_expired_jobs.sql",
        "examples/postgres-rig-agent-jobs/sql/resume_job_kind.sql",
        "examples/postgres-rig-agent-jobs/sql/retry_or_dead.sql",
        "examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql",
        "examples/postgres-rig-agent-jobs/sql/sli_terminal_jobs_not_dead.sql",
        "examples/postgres-rig-agent-jobs/src/agent.rs",
        "examples/postgres-rig-agent-jobs/src/admission_control.rs",
        "examples/postgres-rig-agent-jobs/src/agent_memory.rs",
        "examples/postgres-rig-agent-jobs/src/agent_output.rs",
        "examples/postgres-rig-agent-jobs/src/background_job.rs",
        "examples/postgres-rig-agent-jobs/src/agent_run.rs",
        "examples/postgres-rig-agent-jobs/src/agent_step.rs",
        "examples/postgres-rig-agent-jobs/src/audit.rs",
        "examples/postgres-rig-agent-jobs/src/api.rs",
        "examples/postgres-rig-agent-jobs/src/approval.rs",
        "examples/postgres-rig-agent-jobs/src/bin/deepseek_agent_demo.rs",
        "examples/postgres-rig-agent-jobs/src/bin/postgres_api_server.rs",
        "examples/postgres-rig-agent-jobs/src/bin/postgres_worker_demo.rs",
        "examples/postgres-rig-agent-jobs/src/cancellation.rs",
        "examples/postgres-rig-agent-jobs/src/config.rs",
        "examples/postgres-rig-agent-jobs/src/compatibility.rs",
        "examples/postgres-rig-agent-jobs/src/cost_accounting.rs",
        "examples/postgres-rig-agent-jobs/src/domain.rs",
        "examples/postgres-rig-agent-jobs/src/escalation.rs",
        "examples/postgres-rig-agent-jobs/src/evaluation.rs",
        "examples/postgres-rig-agent-jobs/src/failure_drill.rs",
        "examples/postgres-rig-agent-jobs/src/failure_history.rs",
        "examples/postgres-rig-agent-jobs/src/handoff.rs",
        "examples/postgres-rig-agent-jobs/src/memory_store.rs",
        "examples/postgres-rig-agent-jobs/src/compensation.rs",
        "examples/postgres-rig-agent-jobs/src/outbox.rs",
        "examples/postgres-rig-agent-jobs/src/postgres_store.rs",
        "examples/postgres-rig-agent-jobs/src/recovery.rs",
        "examples/postgres-rig-agent-jobs/src/release_gate.rs",
        "examples/postgres-rig-agent-jobs/src/rig_runner.rs",
        "examples/postgres-rig-agent-jobs/src/sandbox.rs",
        "examples/postgres-rig-agent-jobs/src/scheduled_job.rs",
        "examples/postgres-rig-agent-jobs/src/security.rs",
        "examples/postgres-rig-agent-jobs/src/slo.rs",
        "examples/postgres-rig-agent-jobs/src/timeouts.rs",
        "examples/postgres-rig-agent-jobs/src/tool_call.rs",
        "examples/postgres-rig-agent-jobs/src/tool_contract.rs",
        "examples/postgres-rig-agent-jobs/src/tool_execution_gate.rs",
        "examples/postgres-rig-agent-jobs/src/typed_pipeline.rs",
        "examples/postgres-rig-agent-jobs/src/worker.rs",
        "scripts/smoke-postgres-api.sh",
        "scripts/smoke-local-postgres.sh",
        "scripts/smoke-deepseek-agent.sh",
        "scripts/check-book-links.py",
        "scripts/check-rust-production-hygiene.py",
    )
    for path in required_paths:
        if path not in text:
            failures.append(
                f"{IMPLEMENTATION_EVIDENCE_MAP.relative_to(ROOT)} missing implementation path: {path}"
            )
        if not (ROOT / path).is_file():
            failures.append(f"implementation evidence path does not exist: {path}")


def check_system_diagrams(failures: list[str]) -> None:
    if not SYSTEM_DIAGRAMS.is_file():
        failures.append(
            f"missing system diagrams appendix: {SYSTEM_DIAGRAMS.relative_to(ROOT)}"
        )
        return

    text = SYSTEM_DIAGRAMS.read_text(encoding="utf-8")
    required_phrases = (
        "## How to Use These Diagrams",
        "component -> state -> transition -> evidence -> operator question",
        "## The Whole System",
        "## Job State Machine",
        "## Lease And Heartbeat Timeline",
        "## Retry And Dead-Letter Timeline",
        "## Approval And Side-Effect Path",
        "## Observability Evidence Flow",
        "## Release And Versioning Flow",
        "## Recovery And Replay Flow",
        "the model never owns the workflow; durable state owns the workflow",
        "only valid transitions are expressible, and terminal states stay terminal",
        "restore is successful only when replay is safe and side effects are not duplicated",
        "Primary chapters:",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{SYSTEM_DIAGRAMS.relative_to(ROOT)} missing required diagram phrase: {phrase}"
            )

    if text.count("Core invariant:") < 8:
        failures.append(
            f"{SYSTEM_DIAGRAMS.relative_to(ROOT)} must name a core invariant for each system diagram"
        )


def check_end_to_end_labs(failures: list[str]) -> None:
    if not END_TO_END_LABS.is_file():
        failures.append(
            f"missing end-to-end labs appendix: {END_TO_END_LABS.relative_to(ROOT)}"
        )
        return

    text = END_TO_END_LABS.read_text(encoding="utf-8")
    required_phrases = (
        "## How to Use These Labs",
        "change:",
        "invariant:",
        "acceptance evidence:",
        "## Lab 1: Add A New Job Kind",
        "## Lab 2: Add A Provider Failure Class",
        "## Lab 3: Add A Runbook Query",
        "## Lab 4: Add A Policy Gate",
        "## Lab 5: Add A Behavior Evaluation Gate",
        "## Lab 6: Run A Restore And Replay Drill",
        "## Lab 7: Add A Typed Memory Retention Rule",
        "## Lab 8: Add A Controlled Agent Handoff",
        "## Lab 9: Add A Provider Usage Budget Guard",
        "## Lab 10: Add A Timeout And Cancellation Policy",
        "## Lab 11: Add A Compensation Action",
        "## Lab 12: Write A Temporal Adoption Record",
        "## Lab 13: Write A Kafka Adoption Record",
        "## Final Lab Review",
        "What invariant did I protect?",
        "Which test or runbook proves the change?",
        "agent_memory_by_scope.sql answers which records are eligible",
        "accepted handoff creates or attaches exactly one target job",
        "provider_usage_by_job_kind.sql shows requests, tokens, cost, and p95 latency",
        "running_jobs_past_deadline.sql shows overdue work with timeout action",
        "compensation_backlog.sql shows due work, expired leases, and oldest pending action",
        "TemporalWorkflowExecutionRef rejects empty workflow references",
        "ConsumerReceipt proves one consumer processed one event once for one purpose",
        "The labs are the bridge from reading to operating.",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{END_TO_END_LABS.relative_to(ROOT)} missing required lab phrase: {phrase}"
            )

    if text.count("Acceptance evidence:") < 13:
        failures.append(
            f"{END_TO_END_LABS.relative_to(ROOT)} must include acceptance evidence for each lab"
        )

    if text.count("Primary chapters:") < 13:
        failures.append(
            f"{END_TO_END_LABS.relative_to(ROOT)} must connect each lab to primary chapters"
        )


def check_principle_map(failures: list[str]) -> None:
    if not PRINCIPLE_MAP.is_file():
        failures.append(
            f"missing principle-to-chapter map: {PRINCIPLE_MAP.relative_to(ROOT)}"
        )
        return

    text = PRINCIPLE_MAP.read_text(encoding="utf-8")
    required_phrases = (
        "principle -> primary chapters -> smallest simulation -> production evidence",
        "## Principle Map",
        "## Dependency Order",
        "## Transfer Questions",
        "## Common Review Failures",
        "| Principle | Primary chapters | Smallest simulation | Production evidence |",
        "| Durable before intelligent |",
        "| Typed before clever |",
        "| Ownership before concurrency |",
        "| Boundary before policy |",
        "| Idempotent before retried |",
        "| Evidence before operations |",
        "| Evaluation before behavior release |",
        "| Approval is state |",
        "| Release with old work in mind |",
        "| Recovery must be practiced |",
        "Where does work live when the process dies?",
        "What prevents duplicate intent from becoming duplicate side effect?",
        "When was restore and replay last tested end to end?",
        "This appendix is complete only when each principle can point to:",
        "one transfer question",
        "one common failure",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{PRINCIPLE_MAP.relative_to(ROOT)} missing required principle-map phrase: {phrase}"
            )

    if text.count("| Durable before intelligent |") != 2:
        failures.append(
            f"{PRINCIPLE_MAP.relative_to(ROOT)} should map durable-before-intelligent in the principle and transfer tables"
        )

    if text.count("Likely missing principle") < 1:
        failures.append(
            f"{PRINCIPLE_MAP.relative_to(ROOT)} must include a common-review-failure table"
        )


def check_case_studies(failures: list[str]) -> None:
    if not CASE_STUDIES.is_file():
        failures.append(
            f"missing production case studies appendix: {CASE_STUDIES.relative_to(ROOT)}"
        )
        return

    text = CASE_STUDIES.read_text(encoding="utf-8")
    required_phrases = (
        "Which invariant stays the same, and which controls become stricter",
        "## Case Study 1: Incident Triage Agent",
        "## Case Study 2: Customer Support Reply Agent",
        "## Case Study 3: Billing Adjustment Agent",
        "## Cross-Case Comparison",
        "| Control | What it protects |",
        "duplicate intake event",
        "retrieval snapshot",
        "financial ledger snapshot id",
        "unreviewed model text never becomes a customer-visible reply",
        "replay can never issue a second credit for the same approved adjustment",
        "The controls become stricter as the job kind becomes riskier.",
        "job kind",
        "risk level",
        "side-effect receipt",
        "evaluation receipt",
        "restore/replay behavior",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{CASE_STUDIES.relative_to(ROOT)} missing required case-study phrase: {phrase}"
            )

    if text.count("## Tiny Incident") != 3:
        failures.append(
            f"{CASE_STUDIES.relative_to(ROOT)} must include a tiny incident for each case study"
        )

    if text.count("## System Model") != 3:
        failures.append(
            f"{CASE_STUDIES.relative_to(ROOT)} must include a system model for each case study"
        )

    if text.count("## Production Evidence") != 3:
        failures.append(
            f"{CASE_STUDIES.relative_to(ROOT)} must include production evidence for each case study"
        )

    if text.count("## What To Notice") != 3:
        failures.append(
            f"{CASE_STUDIES.relative_to(ROOT)} must include a what-to-notice section for each case study"
        )


def check_concept_review(failures: list[str]) -> None:
    if not CONCEPT_REVIEW.is_file():
        failures.append(
            f"missing concept review appendix: {CONCEPT_REVIEW.relative_to(ROOT)}"
        )
        return

    text = CONCEPT_REVIEW.read_text(encoding="utf-8")
    required_phrases = (
        "recall -> explain -> apply -> evidence",
        "The goal is active recall, not passive rereading.",
        "## Practice Contract",
        "## Front Matter Review",
        "## Part I Review",
        "## Part II Review",
        "## Part III Review",
        "## Part IV Review",
        "## What Changed In Your Mind",
        "## Worked Review Example",
        "The goal is simple: name how your mental model changed.",
        "| Chapter | Before | After | Evidence habit |",
        "A durable job is the unit of reliable work.",
        "Security boundaries live outside the model.",
        "name the failure",
        "simulate the smallest transition",
        "point to production evidence",
        "describe the next test or runbook query",
        "| Production Scope And Trade-Offs |",
        "Retrieval practice turns reading into engineering judgment.",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{CONCEPT_REVIEW.relative_to(ROOT)} missing required concept-review phrase: {phrase}"
            )

    if text.count("| Chapter | Recall | Explain | Apply | Evidence |") != 5:
        failures.append(
            f"{CONCEPT_REVIEW.relative_to(ROOT)} must include one recall/explain/apply/evidence table for front matter and each part"
        )

    for chapter in range(1, 31):
        chapter_marker = f"| {chapter}."
        if chapter_marker not in text:
            failures.append(
                f"{CONCEPT_REVIEW.relative_to(ROOT)} missing retrieval-practice prompt for chapter {chapter}"
            )

    if "| 20.2 Worked Production Scenario |" not in text:
        failures.append(
            f"{CONCEPT_REVIEW.relative_to(ROOT)} missing retrieval-practice prompt for the worked production scenario"
        )
    if "| 20.1 Agent Handoffs And Multi-Agent Coordination |" not in text:
        failures.append(
            f"{CONCEPT_REVIEW.relative_to(ROOT)} missing retrieval-practice prompt for agent handoffs"
        )
    if "| 30.5 Scaling Paths After Postgres-First |" not in text:
        failures.append(
            f"{CONCEPT_REVIEW.relative_to(ROOT)} missing retrieval-practice prompt for scaling paths"
        )
    if "| 27.5 Agent Memory, Retrieval, And Retention |" not in text:
        failures.append(
            f"{CONCEPT_REVIEW.relative_to(ROOT)} missing retrieval-practice prompt for agent memory"
        )


def check_concept_dependency_graph(failures: list[str]) -> None:
    if not CONCEPT_DEPENDENCY_GRAPH.is_file():
        failures.append(
            f"missing concept dependency graph: {CONCEPT_DEPENDENCY_GRAPH.relative_to(ROOT)}"
        )
        return

    text = CONCEPT_DEPENDENCY_GRAPH.read_text(encoding="utf-8")
    required_phrases = (
        "prerequisite -> new concept -> mechanism -> production capability",
        "## Front Matter Dependencies",
        "## Part I Dependencies",
        "## Part II Dependencies",
        "## Part III Dependencies",
        "## Part IV Dependencies",
        "## Cross-Cutting Dependency Checks",
        "| Chapter | Prerequisite | New concept | Mechanism | Production capability unlocked |",
        "Does the work exist before the model call?",
        "Can duplicate intent become duplicate side effect?",
        "Can restore and replay happen without guessing about side effects?",
        "The dependency graph is the spine of the book:",
        "| Production Scope And Trade-Offs |",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{CONCEPT_DEPENDENCY_GRAPH.relative_to(ROOT)} missing required dependency-graph phrase: {phrase}"
            )

    if text.count("| Chapter | Prerequisite | New concept | Mechanism | Production capability unlocked |") != 5:
        failures.append(
            f"{CONCEPT_DEPENDENCY_GRAPH.relative_to(ROOT)} must include dependency tables for front matter and all four parts"
        )

    for chapter in range(1, 31):
        chapter_marker = f"| {chapter}."
        if chapter_marker not in text:
            failures.append(
                f"{CONCEPT_DEPENDENCY_GRAPH.relative_to(ROOT)} missing dependency row for chapter {chapter}"
            )

    if "| 20.2 Worked Production Scenario |" not in text:
        failures.append(
            f"{CONCEPT_DEPENDENCY_GRAPH.relative_to(ROOT)} missing dependency row for the worked production scenario"
        )
    if "| 20.1 Agent Handoffs And Multi-Agent Coordination |" not in text:
        failures.append(
            f"{CONCEPT_DEPENDENCY_GRAPH.relative_to(ROOT)} missing dependency row for agent handoffs"
        )
    if "| 30.5 Scaling Paths After Postgres-First |" not in text:
        failures.append(
            f"{CONCEPT_DEPENDENCY_GRAPH.relative_to(ROOT)} missing dependency row for scaling paths"
        )
    if "| 27.5 Agent Memory, Retrieval, And Retention |" not in text:
        failures.append(
            f"{CONCEPT_DEPENDENCY_GRAPH.relative_to(ROOT)} missing dependency row for agent memory"
        )


def check_production_evidence_packets(failures: list[str]) -> None:
    if not PRODUCTION_EVIDENCE_PACKETS.is_file():
        failures.append(
            f"missing production evidence packets appendix: {PRODUCTION_EVIDENCE_PACKETS.relative_to(ROOT)}"
        )
        return

    text = PRODUCTION_EVIDENCE_PACKETS.read_text(encoding="utf-8")
    required_phrases = (
        "## How to Use These Packets",
        "## Packet 1: Job-Kind Launch Packet",
        "## Packet 2: Release Packet",
        "## Packet 3: Incident Packet",
        "## Packet 4: Behavior Evaluation Packet",
        "## Packet 5: Security And Trust Packet",
        "## Packet 6: Restore And Replay Packet",
        "## Final Packet Review",
        "claim:",
        "evidence:",
        "owner:",
        "expiry:",
        "gap:",
        "no durable work -> no launch",
        "old job rows:",
        "failed invariant:",
        "dataset version:",
        "smallest authority needed",
        "Backup is not recovery.",
        "Production rigor means claims are reviewable.",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{PRODUCTION_EVIDENCE_PACKETS.relative_to(ROOT)} missing required evidence-packet phrase: {phrase}"
            )

    required_packet_names = (
        "Job-Kind Launch Packet",
        "Release Packet",
        "Incident Packet",
        "Behavior Evaluation Packet",
        "Security And Trust Packet",
        "Restore And Replay Packet",
    )
    for packet_name in required_packet_names:
        if packet_name not in text:
            failures.append(
                f"{PRODUCTION_EVIDENCE_PACKETS.relative_to(ROOT)} missing packet: {packet_name}"
            )

    if text.count("| Claim | Evidence artifact | Owner | Expiry or review trigger |") != 1:
        failures.append(
            f"{PRODUCTION_EVIDENCE_PACKETS.relative_to(ROOT)} must include the launch evidence table"
        )


def check_code_reading_path(failures: list[str]) -> None:
    if not CODE_READING_PATH.is_file():
        failures.append(
            f"missing companion code reading path appendix: {CODE_READING_PATH.relative_to(ROOT)}"
        )
        return

    text = CODE_READING_PATH.read_text(encoding="utf-8")
    required_phrases = (
        "## How to Use This Path",
        "manifest -> runtime config -> domain types -> compatibility -> HTTP admission",
        "## Step 1: Manifest And Feature Boundaries",
        "## Step 2: Domain Types",
        "## Step 3: SQL Ledger",
        "## Step 4: Store Boundary",
        "## Step 5: Worker Loop",
        "## Step 6: Agent And Provider Boundary",
        "## Step 7: Behavior Evaluation Gate",
        "## Step 8: Operator Queries",
        "## Step 9: Binaries",
        "## Step 10: Validation Gate",
        "What invariant does this file protect?",
        "Which chapter teaches the concept?",
        "Which test or check proves it?",
        "database DTO -> validated domain value -> worker logic",
        "scripts/smoke-deepseek-agent.sh checks the live Rig provider boundary when credentials are supplied",
        "RUN_LIVE_DEEPSEEK=1 ./scripts/check-book-readiness.sh",
        "If a production rule has no source artifact and no validation evidence, it is",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{CODE_READING_PATH.relative_to(ROOT)} missing required code-reading phrase: {phrase}"
            )

    required_paths = (
        "examples/postgres-rig-agent-jobs/Cargo.toml",
        "examples/postgres-rig-agent-jobs/src/config.rs",
        "examples/postgres-rig-agent-jobs/src/domain.rs",
        "examples/postgres-rig-agent-jobs/src/admission_control.rs",
        "examples/postgres-rig-agent-jobs/src/background_job.rs",
        "examples/postgres-rig-agent-jobs/src/failure_drill.rs",
        "examples/postgres-rig-agent-jobs/src/failure_history.rs",
        "examples/postgres-rig-agent-jobs/src/agent_run.rs",
        "examples/postgres-rig-agent-jobs/src/agent_step.rs",
        "examples/postgres-rig-agent-jobs/src/agent_output.rs",
        "examples/postgres-rig-agent-jobs/src/api.rs",
        "examples/postgres-rig-agent-jobs/src/tool_call.rs",
        "examples/postgres-rig-agent-jobs/src/tool_execution_gate.rs",
        "examples/postgres-rig-agent-jobs/src/scheduled_job.rs",
        "examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql",
        "examples/postgres-rig-agent-jobs/sql/admit_agent_job.sql",
        "examples/postgres-rig-agent-jobs/sql/resolve_existing_agent_job.sql",
        "examples/postgres-rig-agent-jobs/sql/record_admission_decision.sql",
        "examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql",
        "examples/postgres-rig-agent-jobs/sql/claim_scheduled_jobs.sql",
        "examples/postgres-rig-agent-jobs/sql/complete_scheduled_job.sql",
        "examples/postgres-rig-agent-jobs/sql/fail_or_retry_scheduled_job.sql",
        "examples/postgres-rig-agent-jobs/sql/running_agent_runs.sql",
        "examples/postgres-rig-agent-jobs/sql/pending_agent_handoffs.sql",
        "examples/postgres-rig-agent-jobs/sql/pending_cancellation_requests.sql",
        "examples/postgres-rig-agent-jobs/sql/scheduled_retries.sql",
        "examples/postgres-rig-agent-jobs/sql/waiting_human_approvals.sql",
        "examples/postgres-rig-agent-jobs/sql/open_human_escalations.sql",
        "examples/postgres-rig-agent-jobs/sql/failed_tool_calls.sql",
        "examples/postgres-rig-agent-jobs/sql/failure_history_by_job.sql",
        "examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql",
        "examples/postgres-rig-agent-jobs/sql/evaluation_receipts_by_version.sql",
        "examples/postgres-rig-agent-jobs/sql/release_gate_status.sql",
        "examples/postgres-rig-agent-jobs/sql/provider_usage_by_job_kind.sql",
        "examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql",
        "examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql",
        "examples/postgres-rig-agent-jobs/sql/sli_terminal_jobs_not_dead.sql",
        "examples/postgres-rig-agent-jobs/sql/denied_authorization_events.sql",
        "examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql",
        "examples/postgres-rig-agent-jobs/sql/audit_events_by_run.sql",
        "examples/postgres-rig-agent-jobs/sql/operation_events_by_job.sql",
        "examples/postgres-rig-agent-jobs/sql/running_jobs_past_deadline.sql",
        "examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql",
        "examples/postgres-rig-agent-jobs/sql/outbox_backlog.sql",
        "examples/postgres-rig-agent-jobs/sql/compensation_backlog.sql",
        "examples/postgres-rig-agent-jobs/src/memory_store.rs",
        "examples/postgres-rig-agent-jobs/src/audit.rs",
        "examples/postgres-rig-agent-jobs/src/background_job.rs",
        "examples/postgres-rig-agent-jobs/src/handoff.rs",
        "examples/postgres-rig-agent-jobs/src/cancellation.rs",
        "examples/postgres-rig-agent-jobs/src/escalation.rs",
        "examples/postgres-rig-agent-jobs/src/failure_drill.rs",
        "examples/postgres-rig-agent-jobs/src/compensation.rs",
        "examples/postgres-rig-agent-jobs/src/outbox.rs",
        "examples/postgres-rig-agent-jobs/src/postgres_store.rs",
        "examples/postgres-rig-agent-jobs/src/recovery.rs",
        "examples/postgres-rig-agent-jobs/src/release_gate.rs",
        "examples/postgres-rig-agent-jobs/src/sandbox.rs",
        "examples/postgres-rig-agent-jobs/src/slo.rs",
        "examples/postgres-rig-agent-jobs/src/timeouts.rs",
        "examples/postgres-rig-agent-jobs/src/tool_contract.rs",
        "examples/postgres-rig-agent-jobs/src/worker.rs",
        "examples/postgres-rig-agent-jobs/src/agent.rs",
        "examples/postgres-rig-agent-jobs/src/rig_runner.rs",
        "examples/postgres-rig-agent-jobs/src/bin/postgres_api_server.rs",
        "examples/postgres-rig-agent-jobs/src/bin/deepseek_agent_demo.rs",
        "examples/postgres-rig-agent-jobs/src/bin/postgres_worker_demo.rs",
        "scripts/check-book-readiness.sh",
        "scripts/smoke-local-postgres.sh",
        "scripts/smoke-deepseek-agent.sh",
    )
    for path in required_paths:
        if path not in text:
            failures.append(
                f"{CODE_READING_PATH.relative_to(ROOT)} missing code path: {path}"
            )
        if not (ROOT / path).is_file():
            failures.append(f"companion code reading path references missing file: {path}")


def check_runtime_config_boundary(failures: list[str]) -> None:
    config_path = ROOT / "examples/postgres-rig-agent-jobs/src/config.rs"
    if not config_path.is_file():
        failures.append(f"missing runtime config boundary: {config_path.relative_to(ROOT)}")
        return

    config_text = config_path.read_text(encoding="utf-8")
    required_config_phrases = (
        "pub struct EnvVarName",
        "pub const DATABASE_URL_ENV",
        "pub const BIND_ADDRESS_ENV",
        "pub const DEEPSEEK_API_KEY_ENV",
        "pub enum RuntimeConfigError",
        "pub trait RuntimeEnv",
        "pub struct ProcessEnv",
        "pub struct PostgresWorkerConfig",
        "pub struct PostgresApiServerConfig",
        "pub struct DeepSeekRuntimeConfig",
        "DeepSeekApiKeyPresent",
        "RuntimeConfigError::InvalidUnicode",
        "DatabaseUrl::new",
        "HttpBindAddress::parse",
        "required_non_empty",
        "optional_non_empty_or_default",
    )
    for phrase in required_config_phrases:
        if phrase not in config_text:
            failures.append(
                f"{config_path.relative_to(ROOT)} missing runtime-config phrase: {phrase}"
            )

    binary_requirements = {
        ROOT / "examples/postgres-rig-agent-jobs/src/bin/postgres_api_server.rs": (
            "PostgresApiServerConfig::from_env",
        ),
        ROOT / "examples/postgres-rig-agent-jobs/src/bin/postgres_worker_demo.rs": (
            "PostgresWorkerConfig::from_env",
        ),
        ROOT / "examples/postgres-rig-agent-jobs/src/bin/deepseek_agent_demo.rs": (
            "DeepSeekRuntimeConfig::from_env",
        ),
    }
    for path, phrases in binary_requirements.items():
        if not path.is_file():
            failures.append(f"missing runtime binary: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        for phrase in phrases:
            if phrase not in text:
                failures.append(
                    f"{path.relative_to(ROOT)} must use typed runtime config: {phrase}"
                )

    documentation_requirements = {
        BOOK_SRC / "18-deployment-operations.md": (
            "process strings become typed runtime configuration",
            "DeepSeek key is checked for presence without storing or printing",
        ),
        IMPLEMENTATION_EVIDENCE_MAP: (
            "| Runtime configuration boundary |",
            "examples/postgres-rig-agent-jobs/src/config.rs",
        ),
        CODE_READING_PATH: (
            "raw environment -> RuntimeEnv -> PostgresWorkerConfig or PostgresApiServerConfig",
            "raw environment -> RuntimeEnv -> DeepSeekRuntimeConfig",
        ),
        REQUIREMENT_TRACEABILITY: (
            "| Runtime configuration is typed before startup. |",
        ),
    }
    for path, phrases in documentation_requirements.items():
        if not path.is_file():
            failures.append(f"missing runtime config documentation: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        for phrase in phrases:
            if phrase not in text:
                failures.append(
                    f"{path.relative_to(ROOT)} missing runtime-config documentation phrase: {phrase}"
                )


def check_live_deepseek_smoke(failures: list[str]) -> None:
    script = ROOT / "scripts/smoke-deepseek-agent.sh"
    if not script.is_file():
        failures.append(f"missing DeepSeek smoke script: {script.relative_to(ROOT)}")
        return

    text = script.read_text(encoding="utf-8")
    required_script_phrases = (
        "DEEPSEEK_API_KEY is required for the DeepSeek agent smoke test",
        "--features rig-agent",
        "--bin deepseek-agent-demo",
        "worker outcome: Succeeded",
        "job status: Succeeded",
        "AgentStarted",
        "AgentSucceeded",
        "result.summary must be a non-empty string",
        "result.next_action must be a non-empty string",
        "result.approval must be required or not_required",
    )
    for phrase in required_script_phrases:
        if phrase not in text:
            failures.append(
                f"{script.relative_to(ROOT)} missing DeepSeek smoke phrase: {phrase}"
            )

    documentation_requirements = {
        README: (
            'RUN_LIVE_DEEPSEEK=1 DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY"',
            "scripts/smoke-deepseek-agent.sh",
        ),
        BOOK_SRC / "07-running-system-locally.md": (
            "scripts/smoke-deepseek-agent.sh",
            "model output becomes a typed result",
        ),
        BOOK_SRC / "17-testing-production-agents.md": (
            "optional DeepSeek smoke gate",
            "real Rig provider boundary still produces parseable typed output",
        ),
        BOOK_SRC / "18-deployment-operations.md": (
            "scripts/smoke-deepseek-agent.sh",
            "agent-start and agent-success event evidence",
        ),
        REQUIREMENT_TRACEABILITY: (
            "| Live provider behavior is checked only at the provider boundary. |",
        ),
    }
    for path, phrases in documentation_requirements.items():
        if not path.is_file():
            failures.append(f"missing DeepSeek smoke documentation: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        inline_text = normalized_inline(text)
        for phrase in phrases:
            if phrase not in text and normalized_inline(phrase) not in inline_text:
                failures.append(
                    f"{path.relative_to(ROOT)} missing DeepSeek smoke documentation phrase: {phrase}"
                )


def check_design_smells(failures: list[str]) -> None:
    if not DESIGN_SMELLS.is_file():
        failures.append(
            f"missing design-smell index: {DESIGN_SMELLS.relative_to(ROOT)}"
        )
        return

    text = DESIGN_SMELLS.read_text(encoding="utf-8")
    required_phrases = (
        "concept -> design smell -> production symptom -> corrective invariant -> evidence",
        "## Practice Contract",
        "## Beginner Traps And Expert Instincts",
        "## Front Matter Smells",
        "## Part I Smells",
        "## Part II Smells",
        "## Part III Smells",
        "## Part IV Smells",
        "## Production Contract",
        "I see:",
        "It can fail by:",
        "The invariant should be:",
        "The evidence should be:",
        "The smallest fix is:",
        "Beginner trap:",
        "Expert instinct:",
        "Minimum serious fix:",
        "If an operation can be retried, it must have identity.",
        "A raw string is not a domain model.",
        "Memory is not truth.",
        "local shortcut",
        "production symptom",
        "corrective invariant",
        "evidence to inspect",
        "Concepts become durable when readers can recognize their broken forms.",
    )
    for phrase in required_phrases:
        if phrase not in text:
            failures.append(
                f"{DESIGN_SMELLS.relative_to(ROOT)} missing required design-smell phrase: {phrase}"
            )

    if text.count("| Chapter | Design smell | Production symptom | Corrective invariant | Evidence to inspect |") != 5:
        failures.append(
            f"{DESIGN_SMELLS.relative_to(ROOT)} must include design-smell tables for front matter and all four parts"
        )

    required_rows = (
        "| System Model And Notation |",
        "| Design Principles |",
        "| Production Scope And Trade-Offs |",
        "| 20.1 Agent Handoffs And Multi-Agent Coordination |",
        "| 20.2 Worked Production Scenario |",
        "| 27.5 Agent Memory, Retrieval, And Retention |",
        "| 30.5 Scaling Paths After Postgres-First |",
    )
    for row in required_rows:
        if row not in text:
            failures.append(
                f"{DESIGN_SMELLS.relative_to(ROOT)} missing design-smell row: {row}"
            )

    for chapter in range(1, 31):
        chapter_marker = f"| {chapter}."
        if chapter_marker not in text:
            failures.append(
                f"{DESIGN_SMELLS.relative_to(ROOT)} missing design-smell row for chapter {chapter}"
            )


def check_reader_role_paths(failures: list[str]) -> None:
    if not ROLE_PATHS.is_file():
        failures.append(
            f"missing reader-role operating paths appendix: {ROLE_PATHS.relative_to(ROOT)}"
        )
        return

    text = ROLE_PATHS.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "role -> chapters -> evidence -> practice artifact",
        "## Shared Baseline",
        "## AI Engineer Path",
        "## Rust Engineer Path",
        "## Platform Or SRE Engineer Path",
        "## Security Or Governance Reviewer Path",
        "## Founder Or Technical Leader Path",
        "## Cross-Role Handoff",
        "## Production Contract",
        "Can this agent job survive process death, duplicate input, provider failure,",
        "The first read should be sequential.",
        "Do not use a role path to skip the production model.",
        "behavior changes leave evaluation receipts",
        "raw database, HTTP, and provider values are parsed at boundaries",
        "an on-call engineer can reconstruct queue health",
        "no untrusted text can grant permissions",
        "launch decision is based on job-kind evidence",
        "Each role passes evidence and a next artifact",
        "I inspected:",
        "I accepted:",
        "I rejected:",
        "The next owner is:",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{ROLE_PATHS.relative_to(ROOT)} missing required reader-role phrase: {phrase}"
            )

    required_roles = (
        "ai engineer",
        "Rust engineer",
        "platform or sre engineer",
        "security or governance reviewer",
        "founder or technical leader",
    )
    for role in required_roles:
        if role.lower() not in text.lower():
            failures.append(
                f"{ROLE_PATHS.relative_to(ROOT)} missing reader role: {role}"
            )

    if text.count("| Step | Read | Inspect | Practice artifact |") != 5:
        failures.append(
            f"{ROLE_PATHS.relative_to(ROOT)} must include one operating table for each reader role"
        )

    if text.count("| From | To | Handoff question |") != 1:
        failures.append(
            f"{ROLE_PATHS.relative_to(ROOT)} must include a cross-role handoff table"
        )


def check_requirement_traceability(failures: list[str]) -> None:
    if not REQUIREMENT_TRACEABILITY.is_file():
        failures.append(
            f"missing production requirement traceability appendix: {REQUIREMENT_TRACEABILITY.relative_to(ROOT)}"
        )
        return

    text = REQUIREMENT_TRACEABILITY.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "teaching chapter -> implementation artifact -> validation evidence -> reviewer question",
        "## Traceability Matrix",
        "## Requirement Review Protocol",
        "## No-Evidence No-Claim Rule",
        "Minimal production stack: Rust, PostgreSQL, Rig, worker, API when needed.",
        "PostgreSQL is the first durable coordination layer.",
        "Rig is the agent intelligence boundary, not the reliability layer.",
        "Live provider behavior is checked only at the provider boundary.",
        "SQL migrations and schemas preserve workflow state.",
        "Scheduling and retry state are database-backed.",
        "Idempotency is present before retry and side effects.",
        "Domain values use explicit Rust types instead of raw primitives.",
        "Typestate is used where lifecycle state controls legal operations.",
        "Raw outside, typed inside.",
        "Runtime configuration is typed before startup.",
        "Model output is parsed and validated before authority is granted.",
        "Every tool has an explicit contract.",
        "Agent memory is typed, scoped, retained, and policy-checked.",
        "Human approval and escalation are durable control surfaces.",
        "First production exposure is a job-kind evidence decision.",
        "Observability uses traces, metrics, logs, audit events, and operation events.",
        "Behavior evaluation is part of release engineering.",
        "Security boundaries stay outside the model.",
        "Credential lifecycle work is durable without storing secret values.",
        "credential_rotation_review.sql",
        "Privacy and data-protection work is durable operational state.",
        "data_protection_review.sql",
        "Timeout, cancellation, and compensation are separate production controls.",
        "Disaster recovery is restore plus replay safety, not backup existence.",
        "Scaling paths preserve evidence rather than hiding the state machine.",
        "Pedagogy is progressive: failure, intuition, typed model, implementation, production hardening, tests, operations, sources.",
        "Pedagogy supports ADHD and low-working-memory readers without lowering rigor.",
        "attention-friendly-production-learning.md",
        "chapter-card-pack.md",
        "first-production-deployment-proof.md",
        "production-build-milestones.md",
        "prefilled chapter cards",
        "plain-language term ladder",
        "production build milestones",
        "Code, SQL, and Cargo commands are checked artifacts, not decorative snippets.",
        "requirement:",
        "chapter evidence:",
        "implementation evidence:",
        "validation evidence:",
        "operator evidence:",
        "next artifact:",
        "the chapter points to a checked Rust type",
        "the chapter points to a checked SQL file",
        "the chapter points to a runbook query",
        "the chapter points to a test or readiness gate",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{REQUIREMENT_TRACEABILITY.relative_to(ROOT)} missing required traceability phrase: {phrase}"
            )

    if text.count("| Production requirement | Where the book teaches it | Implementation evidence | Validation evidence | Reviewer question |") != 1:
        failures.append(
            f"{REQUIREMENT_TRACEABILITY.relative_to(ROOT)} must include the production requirement traceability table"
        )

    if text.count("| Minimal production stack:") != 1:
        failures.append(
            f"{REQUIREMENT_TRACEABILITY.relative_to(ROOT)} must include exactly one minimal-stack requirement row"
        )


def check_attention_protocol(failures: list[str]) -> None:
    if not ATTENTION_PROTOCOL.is_file():
        failures.append(
            f"missing attention-friendly learning appendix: {ATTENTION_PROTOCOL.relative_to(ROOT)}"
        )
        return

    text = ATTENTION_PROTOCOL.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "one concept\none artifact\none proof\none pause",
        "## Now, Next, Done",
        "now:",
        "next:",
        "done:",
        "the proof that lets me stop",
        "The `done` line must name",
        "Avoid vague next actions:",
        "Use concrete next actions:",
        "## Small Action, Fast Feedback",
        "act:",
        "check:",
        "explain:",
        "repair:",
        "Fast feedback helps focus",
        "The `check` line must point to production evidence",
        "Do not move to a second abstraction until the first check is real.",
        "## The Micro-Lesson",
        "pain:",
        "rule:",
        "tiny example:",
        "## Method Map For Attention-Friendly Rigor",
        "Universal design",
        "Clear expectation",
        "Cognitive-load control",
        "Worked example with fading",
        "Retrieval practice",
        "Spaced review",
        "Plain language",
        "small step\nreal artifact\nvisible proof\nshort pause",
        "## Fifteen-Minute Production Sprint",
        "prepare:",
        "inspect:",
        "prove:",
        "capture:",
        "stop:",
        "What state is changing?",
        "What move changes it?",
        "What proof remains after the move?",
        "## The Chapter Card",
        "failure prevented:",
        "operator question:",
        "## The Two-Pass Method",
        "Pass 1 builds orientation:",
        "Pass 2 builds production skill:",
        "## Faded Practice Loop",
        "watch one\ncomplete one\nprove one",
        "The hints fade, but the production standard",
        "## Simple Language Without Lowering Rigor",
        "Use the one-new-term rule in dense sections:",
        "one hard term\none plain sentence\none artifact\none proof",
        "## Plain-Language Term Ladder",
        "plain phrase -> formal term -> production artifact -> proof",
        "| work that must not disappear | durable job |",
        "| repeat without double action | idempotency |",
        "| model-proposed action | typed tool request |",
        "If you cannot name the artifact or proof",
        "## The No-Compromise Gate",
        "Can I name the state?",
        "Can I name one test, query, or runbook that would catch a regression?",
        "## Code Reading Protocol",
        "Find the domain type.",
        "Find the test that rejects the bad case.",
        "## SQL Reading Protocol",
        "Constraint that rejects an impossible row.",
        "## Runbook Reading Protocol",
        "What question does this command answer?",
        "## Spaced Review",
        "Appendix L for retrieval practice",
        "Appendix F for chapter checkpoints",
        "Appendix V for maintenance cadence",
        "## Study Session Template",
        "Stop with a next-action note.",
        "## Distraction Parking Lot",
        "later:",
        "## Visible Wins",
        "A visible win is inspectable.",
        "## Tiny Production Path",
        "Durable intake",
        "`release_gate_runs` and `release_gate_status.sql`",
        "Do not move to the next step until the proof sentence is real.",
        "## Choose One Mode",
        "read: understand one concept",
        "build: change one artifact",
        "operate: prove one behavior",
        "read done:",
        "build done:",
        "operate done:",
        "## Production Builder Template",
        "Add a negative test.",
        "Run the validation gate.",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{ATTENTION_PROTOCOL.relative_to(ROOT)} missing required attention-protocol phrase: {phrase}"
            )

    if "## Further Reading and Sources" not in text:
        failures.append(
            f"{ATTENTION_PROTOCOL.relative_to(ROOT)} must include a '## Further Reading and Sources' section"
        )


def check_chapter_card_pack(failures: list[str]) -> None:
    if not CHAPTER_CARD_PACK.is_file():
        failures.append(
            f"missing chapter-card pack appendix: {CHAPTER_CARD_PACK.relative_to(ROOT)}"
        )
        return

    text = CHAPTER_CARD_PACK.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "chapter -> concept -> artifact -> proof -> operator question",
        "Appendix X. Chapter Card Pack",
        "## How To Use The Cards",
        "Read the concept.",
        "Open or inspect the artifact.",
        "Name the proof.",
        "Answer the operator question.",
        "## Front Matter Cards",
        "## Part I Cards",
        "## Part II Cards",
        "## Part III Cards",
        "## Part IV Cards",
        "## One-Screen Restart",
        "chapter:\nconcept:\nartifact:\nproof:\noperator question:\nnext action:",
        "inspect one Rust type",
        "run one SQL query",
        "read one test",
        "answer one operator question",
        "Do not use the card to avoid the hard parts.",
        "## Further Reading and Sources",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{CHAPTER_CARD_PACK.relative_to(ROOT)} missing required chapter-card phrase: {phrase}"
            )

    required_rows = (
        "| System Model |",
        "| Design Principles |",
        "| Scope And Trade-Offs |",
        "| 1. The Problem |",
        "| 2. The Mental Model |",
        "| 2.5 Guarantees And Failure Semantics |",
        "| 3. The Postgres Ledger |",
        "| 4. The Rust Domain Model |",
        "| 4.5 Typed Composition Lens |",
        "| 5. The Worker Loop |",
        "| 6. The Rig Boundary |",
        "| 7. Running The System Locally |",
        "| 8. Production Hardening |",
        "| 9. Failure Modes |",
        "| 10. Capstone |",
        "| 11. The Real Postgres Store |",
        "| 12. Idempotency And Side Effects |",
        "| 13. Leases, Heartbeats, And Cancellation |",
        "| 14. Retry, Backoff, And Dead Letters |",
        "| 15. Observability And SLOs |",
        "| 16. Human Approval And Policy Gates |",
        "| 17. Testing Production Agents |",
        "| 18. Deployment And Operations |",
        "| 19. Running For Years |",
        "| 20. Final Production Blueprint |",
        "| 20.1 Agent Handoffs And Multi-Agent Coordination |",
        "| 20.2 Worked Production Scenario |",
        "| 21. SLIs, SLOs, And Error Budgets |",
        "| 22. Capacity, Backpressure, And Provider Quotas |",
        "| 23. Runbooks For Agent Job Systems |",
        "| 24. Incident Response And Postmortems |",
        "| 25. Release Engineering For Agents |",
        "| 26. Toil, Automation, And Ownership |",
        "| 27. Evaluation And Behavior Reliability |",
        "| 27.5 Agent Memory, Retrieval, And Retention |",
        "| 28. Security, Abuse, And Trust Boundaries |",
        "| 29. Disaster Recovery And Continuity |",
        "| 30. Reliability Maturity Model |",
        "| 30.5 Scaling Paths After Postgres-First |",
    )
    for row in required_rows:
        if row not in text:
            failures.append(
                f"{CHAPTER_CARD_PACK.relative_to(ROOT)} missing chapter-card row: {row}"
            )


def check_plain_language_production_cards(failures: list[str]) -> None:
    if not PLAIN_LANGUAGE_PRODUCTION_CARDS.is_file():
        failures.append(
            f"missing plain-language production cards appendix: {PLAIN_LANGUAGE_PRODUCTION_CARDS.relative_to(ROOT)}"
        )
        return

    text = PLAIN_LANGUAGE_PRODUCTION_CARDS.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "Appendix Z. Plain-Language Production Cards",
        "say it small\nmake it exact\nprove it",
        "thing -> move -> evidence -> promise",
        "The system handles reliability.",
        "| Work must exist before the model starts. | Durable agent job |",
        "| One worker owns the job only for a while. | Lease |",
        "| Doing work again must not mean doing the external action twice. | Idempotency |",
        "| Model text is not trusted state. | Raw model output |",
        "| A tool call is a side effect boundary. | Typed tool request |",
        "| A person decision is state, not chat. | Human approval gate |",
        "| Reliability needs a measured promise. | SLI and SLO |",
        "| Behavior changes need evidence before release. | Evaluation receipt |",
        "| A release should stop when evidence is missing. | Release gate |",
        "`release_gate_runs` row and `release_gate_status.sql`",
        "| Backup is not recovery until restore is practiced. | Restore drill |",
        "| Two strings with different meanings need different types. | Newtype |",
        "| Some moves should be impossible to call too early. | Typestate |",
        "| The model cannot grant itself permission. | Trust boundary |",
        "| Scaling must preserve the proof trail. | Evidence-preserving migration |",
        "term:\nsmall sentence:\nartifact:\nproof:\nnext action:",
        "Can this sentence tell me what to inspect?",
        "Can this sentence tell me what evidence proves the claim?",
        "Appendix O",
        "Invariant: every simple sentence about a reliable agent should still point",
        "## Further Reading and Sources",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{PLAIN_LANGUAGE_PRODUCTION_CARDS.relative_to(ROOT)} missing required plain-language card phrase: {phrase}"
            )


def check_production_micro_drills(failures: list[str]) -> None:
    if not PRODUCTION_MICRO_DRILLS.is_file():
        failures.append(
            f"missing production micro-drills appendix: {PRODUCTION_MICRO_DRILLS.relative_to(ROOT)}"
        )
        return

    text = PRODUCTION_MICRO_DRILLS.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "Appendix AA. Production Micro-Drills",
        "small drill\nreal artifact\none proof sentence\nstop or repair",
        "drill:\nartifact:\naction:\ncheck:\nproof sentence:\nnext:",
        "The `proof sentence` must name what the artifact proves.",
        "## Drill Contract",
        "| Action | The one thing to read, run, write, or inspect. |",
        "| Stop rule | The moment when the drill is complete. |",
        "Do not add a second concept until the stop rule is true.",
        "## The Drills",
        "| Durable intake |",
        "| Idempotency key |",
        "| Typed input |",
        "| Job claim |",
        "| Heartbeat |",
        "| Retry decision |",
        "| Dead letter |",
        "| Tool contract |",
        "| Approval gate |",
        "| Side-effect receipt |",
        "| Trace id |",
        "| Evaluation receipt |",
        "| Release gate |",
        "| Security boundary |",
        "| Credential lifecycle review |",
        "| Restore replay |",
        "| Temporal adoption decision |",
        "| Kafka adoption decision |",
        "| Data protection review |",
        "| Maintenance review |",
        "## Exact Artifact Index",
        "Use this table when \"find the artifact\" is still too much work.",
        "examples/postgres-rig-agent-jobs/sql/enqueue_agent_job.sql",
        "examples/postgres-rig-agent-jobs/sql/resolve_existing_agent_job.sql",
        "examples/postgres-rig-agent-jobs/sql/pick_due_job.sql",
        "examples/postgres-rig-agent-jobs/sql/extend_lease.sql",
        "examples/postgres-rig-agent-jobs/src/tool_contract.rs",
        "examples/postgres-rig-agent-jobs/sql/side_effect_receipts_by_run.sql",
        "examples/postgres-rig-agent-jobs/sql/evaluation_receipts_by_version.sql",
        "examples/postgres-rig-agent-jobs/sql/release_gate_status.sql",
        "examples/postgres-rig-agent-jobs/sql/sandbox_policy_violations.sql",
        "examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql",
        "examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql",
        "examples/postgres-rig-agent-jobs/src/temporal_adoption.rs",
        "examples/postgres-rig-agent-jobs/src/kafka_adoption.rs",
        "examples/postgres-rig-agent-jobs/sql/data_protection_review.sql",
        "examples/postgres-rig-agent-jobs/sql/job_kind_lifecycle_review.sql",
        "examples/postgres-rig-agent-jobs/sql/storage_pressure_by_table.sql",
        "cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml",
        "--features api-server api::tests::api_admission_enqueues_typed_job",
        "scripts/check-micro-drill-artifacts.py",
        "each focused Cargo test command selects at least one real test",
        "RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh",
        "## Seven-Minute Drill",
        "minute 1: pick one drill",
        "minute 7: stop or write one repair action",
        "## When A Drill Fails",
        "missing artifact:",
        "missing proof:",
        "likely risk:",
        "small repair:",
        "validation:",
        "## From Drill To Deployment",
        "durable intake drill -> durable intake proof",
        "job claim drill -> worker ownership proof",
        "tool contract drill -> provider boundary proof",
        "restore replay drill -> restore and replay note",
        "temporal adoption decision drill -> workflow adoption proof",
        "kafka adoption decision drill -> event-stream adoption proof",
        "Invariant: every drill must end with a real artifact and one proof sentence.",
        "## Further Reading and Sources",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{PRODUCTION_MICRO_DRILLS.relative_to(ROOT)} missing required micro-drill phrase: {phrase}"
            )


def check_production_build_milestones(failures: list[str]) -> None:
    if not PRODUCTION_BUILD_MILESTONES.is_file():
        failures.append(
            f"missing production build milestones appendix: {PRODUCTION_BUILD_MILESTONES.relative_to(ROOT)}"
        )
        return

    text = PRODUCTION_BUILD_MILESTONES.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "Appendix AB. Production Build Milestones",
        "build -> inspect -> run -> proof",
        "build:\ninspect:\nrun:\nproof:\ndo not move on if:",
        "Do not treat reading as completion when the production proof is missing.",
        "## Milestone Contract",
        "| Build | What artifact must exist? |",
        "| Do not move on if | Which missing evidence blocks the next milestone? |",
        "## The Milestones",
        "| Durable intake |",
        "| Typed domain boundary |",
        "| Worker ownership |",
        "| Rig provider boundary |",
        "| Idempotent side effects |",
        "| Human approval gate |",
        "| Observability and SLOs |",
        "| Evaluation and release gate |",
        "| Security boundary |",
        "| Credential lifecycle |",
        "| Data protection |",
        "| Recovery and replay |",
        "| Operations runbooks |",
        "| Evidence-preserving scale |",
        "examples/postgres-rig-agent-jobs/sql/enqueue_agent_job.sql",
        "examples/postgres-rig-agent-jobs/src/domain.rs",
        "examples/postgres-rig-agent-jobs/sql/pick_due_job.sql",
        "examples/postgres-rig-agent-jobs/src/rig_runner.rs",
        "examples/postgres-rig-agent-jobs/src/tool_contract.rs",
        "examples/postgres-rig-agent-jobs/src/approval.rs",
        "examples/postgres-rig-agent-jobs/sql/sli_job_start_latency.sql",
        "examples/postgres-rig-agent-jobs/src/evaluation.rs",
        "examples/postgres-rig-agent-jobs/src/release_gate.rs",
        "examples/postgres-rig-agent-jobs/src/security.rs",
        "examples/postgres-rig-agent-jobs/src/sandbox.rs",
        "examples/postgres-rig-agent-jobs/src/credential_lifecycle.rs",
        "examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql",
        "examples/postgres-rig-agent-jobs/src/data_protection.rs",
        "examples/postgres-rig-agent-jobs/sql/data_protection_review.sql",
        "examples/postgres-rig-agent-jobs/src/recovery.rs",
        "examples/postgres-rig-agent-jobs/sql/oldest_pending_job.sql",
        "examples/postgres-rig-agent-jobs/sql/release_gate_status.sql",
        "examples/postgres-rig-agent-jobs/sql/restore_replay_candidates.sql",
        "RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh",
        "## Stop Conditions",
        "These stop conditions are not learning failures. They are production signals.",
        "milestone:\nowner:\nartifact:\ncommand:\nproof:\nmissing evidence:\nnext repair:",
        "## From Learning To Production",
        "small step\nreal artifact\nvisible proof\nrepair before expansion",
        "Simple language helps the learner find the proof.",
        "## Further Reading and Sources",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{PRODUCTION_BUILD_MILESTONES.relative_to(ROOT)} missing required build-milestone phrase: {phrase}"
            )


def check_failure_first_learning_map(failures: list[str]) -> None:
    if not FAILURE_FIRST_LEARNING_MAP.is_file():
        failures.append(
            f"missing failure-first learning map appendix: {FAILURE_FIRST_LEARNING_MAP.relative_to(ROOT)}"
        )
        return

    text = FAILURE_FIRST_LEARNING_MAP.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "Appendix AC. Failure-First Learning Map",
        "failure -> false fix -> invariant -> artifact -> proof",
        "agent.run(\"do the task\")",
        "## Front Matter",
        "## Part I",
        "## Part II",
        "## Part III",
        "## Part IV",
        "| Chapter | Production failure | False fix | Invariant | Proof artifact |",
        "| 1. The Problem |",
        "| 6. The Rig Boundary |",
        "| 12. Idempotency And Side Effects |",
        "| 16. Human Approval And Policy Gates |",
        "| 20.1 Agent Handoffs And Multi-Agent Coordination |",
        "| 27.5 Agent Memory, Retrieval, And Retention |",
        "| 30.5 Scaling Paths After Postgres-First |",
        "The model may guess. The system must know.",
        "one concrete failure\none false shortcut\none surviving invariant\none production artifact\none proof sentence",
        "from \"the agent ran\" to \"the system can prove what happened\"",
        "## Further Reading and Sources",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{FAILURE_FIRST_LEARNING_MAP.relative_to(ROOT)} missing required failure-first phrase: {phrase}"
            )


def check_core_failure_openings(failures: list[str]) -> None:
    required_phrases = (
        "## Production Failure",
        "**What breaks:**",
        "**False fix:**",
        "**Design response:**",
    )

    for chapter_name in sorted(FAILURE_OPENING_CHAPTERS):
        path = BOOK_SRC / chapter_name
        if not path.is_file():
            failures.append(
                f"missing failure-opening chapter: {path.relative_to(ROOT)}"
            )
            continue

        text = path.read_text(encoding="utf-8")
        chapter_headings = headings(text)
        display = path.relative_to(ROOT)

        production_failure_index = expected_section_index(
            chapter_headings, "production failure"
        )
        chapter_thread_index = expected_section_index(chapter_headings, "chapter thread")
        motivation_index = expected_section_index(chapter_headings, "motivation")

        if production_failure_index is None:
            failures.append(f"{display} must include a '## Production Failure' section")
            continue

        if (
            chapter_thread_index is not None
            and motivation_index is not None
            and not (chapter_thread_index < production_failure_index < motivation_index)
        ):
            failures.append(
                f"{display} Production Failure section must appear after Chapter Thread and before Motivation"
            )

        failure_section = section_after_heading(
            text,
            "Production Failure",
            (
                "Motivation",
                "Plain Version",
                "What You Already Know",
            ),
        )

        for phrase in required_phrases:
            if phrase not in text and phrase not in failure_section:
                failures.append(
                    f"{display} Production Failure section missing required phrase: {phrase}"
                )


def check_first_production_deployment_proof(failures: list[str]) -> None:
    if not FIRST_PRODUCTION_DEPLOYMENT_PROOF.is_file():
        failures.append(
            f"missing first production deployment proof appendix: {FIRST_PRODUCTION_DEPLOYMENT_PROOF.relative_to(ROOT)}"
        )
        return

    text = FIRST_PRODUCTION_DEPLOYMENT_PROOF.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "Appendix Y. First Production Deployment Proof",
        "This appendix is a launch proof path, not a cloud recipe.",
        "one API process",
        "one worker process",
        "one Postgres database",
        "one Rig boundary",
        "Do not expose a new agent job kind to real users until",
        "durable intake proof:",
        "worker ownership proof:",
        "provider boundary proof:",
        "policy or approval proof:",
        "observability proof:",
        "evaluation proof:",
        "security proof:",
        "rollback or pause plan:",
        "restore and replay note:",
        "RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh",
        "RUN_LIVE_DEEPSEEK=1 ./scripts/check-book-readiness.sh",
        "DEEPSEEK_API_KEY",
        "job-kind launch packet",
        "Operator Handoff",
        "pause command:",
        "next review date:",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{FIRST_PRODUCTION_DEPLOYMENT_PROOF.relative_to(ROOT)} missing required first-deployment phrase: {phrase}"
            )

    if "## Further Reading and Sources" not in text:
        failures.append(
            f"{FIRST_PRODUCTION_DEPLOYMENT_PROOF.relative_to(ROOT)} must include a '## Further Reading and Sources' section"
        )


def check_formal_definition_ledger(failures: list[str]) -> None:
    if not FORMAL_DEFINITION_LEDGER.is_file():
        failures.append(
            f"missing formal definition ledger appendix: {FORMAL_DEFINITION_LEDGER.relative_to(ROOT)}"
        )
        return

    text = FORMAL_DEFINITION_LEDGER.read_text(encoding="utf-8")
    inline_text = normalized_inline(text)
    required_phrases = (
        "## Formal Definition Ledger",
        "state -> actor -> transition -> evidence -> invariant",
        "Can I name the state?",
        "Can I name the actor allowed to change it?",
        "Can I name the transition and precondition?",
        "Can I name the evidence left behind?",
        "Can I name the invariant that survives failure?",
        "| Chapter | Formal definition | Required evidence |",
        "A reliable agent system is a set of named states changed by authorized actors",
        "A Postgres ledger is the durable coordination layer",
        "The Rig boundary is the adapter layer where provider behavior becomes typed agent output",
        "A handoff is durable responsibility transfer",
        "A trust boundary is the line where text, tool input, memory, credentials, or tenant context loses authority",
        "Scaling is the migration or duplication of responsibility",
        "defined concept",
        "allowed state",
        "authorized actor",
        "legal transition",
        "durable evidence",
        "surviving invariant",
        "reliable when its concepts have definitions and those definitions have",
    )
    for phrase in required_phrases:
        if phrase not in text and normalized_inline(phrase) not in inline_text:
            failures.append(
                f"{FORMAL_DEFINITION_LEDGER.relative_to(ROOT)} missing required formal-definition phrase: {phrase}"
            )

    required_rows = (
        "| System Model And Notation |",
        "| Design Principles |",
        "| Production Scope And Trade-Offs |",
        "| 1. The Problem |",
        "| 2. The Mental Model |",
        "| 2.5 Guarantees And Failure Semantics |",
        "| 3. The Postgres Ledger |",
        "| 4. The Rust Domain Model |",
        "| 4.5 Typed Composition Lens |",
        "| 5. The Worker Loop |",
        "| 6. The Rig Boundary |",
        "| 7. Running The System Locally |",
        "| 8. Production Hardening |",
        "| 9. Failure Modes |",
        "| 10. Capstone |",
        "| 11. The Real Postgres Store |",
        "| 12. Idempotency And Side Effects |",
        "| 13. Leases, Heartbeats, And Cancellation |",
        "| 14. Retry, Backoff, And Dead Letters |",
        "| 15. Observability And SLOs |",
        "| 16. Human Approval And Policy Gates |",
        "| 17. Testing Production Agents |",
        "| 18. Deployment And Operations |",
        "| 19. Running For Years |",
        "| 20. Final Production Blueprint |",
        "| 20.1 Agent Handoffs And Multi-Agent Coordination |",
        "| 20.2 Worked Production Scenario |",
        "| 21. SLIs, SLOs, And Error Budgets |",
        "| 22. Capacity, Backpressure, And Provider Quotas |",
        "| 23. Runbooks For Agent Job Systems |",
        "| 24. Incident Response And Postmortems |",
        "| 25. Release Engineering For Agents |",
        "| 26. Toil, Automation, And Ownership |",
        "| 27. Evaluation And Behavior Reliability |",
        "| 27.5 Agent Memory, Retrieval, And Retention |",
        "| 28. Security, Abuse, And Trust Boundaries |",
        "| 29. Disaster Recovery And Continuity |",
        "| 30. Reliability Maturity Model |",
        "| 30.5 Scaling Paths After Postgres-First |",
    )
    for row in required_rows:
        if row not in text:
            failures.append(
                f"{FORMAL_DEFINITION_LEDGER.relative_to(ROOT)} missing formal-definition row: {row}"
            )


def check_public_banned_phrases(failures: list[str]) -> None:
    public_paths = [README]
    public_paths.extend(sorted(BOOK_SRC.glob("*.md")))

    for path in public_paths:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8").lower()
        for phrase in PUBLIC_BANNED_PHRASES:
            pattern = re.escape(phrase)
            if phrase[0].isalnum():
                pattern = r"\b" + pattern
            if phrase[-1].isalnum():
                pattern = pattern + r"\b"
            if re.search(pattern, text):
                failures.append(
                    f"{path.relative_to(ROOT)} contains non-learner-facing or private-work phrase: {phrase}"
                )


def check_public_publication_surface(failures: list[str]) -> None:
    public_paths = [README]
    public_paths.extend(sorted(BOOK_SRC.glob("*.md")))

    authoring_leak_phrases = (
        "STYLE_GUIDE.md",
        "QUALITY.md",
        "check-book-pedagogy.py",
        "check-book-objective-coverage.py",
        "pedagogy checker",
        "objective-coverage gate",
        "objective-coverage gates",
        "manuscript checks",
        "local objective coverage",
        "source-hygiene scripts",
        "Book-wide pedagogy is governed by",
        "The current edition",
        "Each main chapter now also includes",
        "Failure-drill examples now include",
        "This workspace is",
    )

    for path in public_paths:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        for phrase in authoring_leak_phrases:
            if phrase in text:
                failures.append(
                    f"{path.relative_to(ROOT)} contains non-learner-facing publication phrase: {phrase}"
                )


def check_validation_command_consistency(failures: list[str]) -> None:
    public_paths = [README]
    public_paths.extend(sorted(BOOK_SRC.glob("*.md")))

    stale_rig_check = re.compile(r"cargo\s+check[^\n]*--features\s+rig-agent")
    for path in public_paths:
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        if stale_rig_check.search(text):
            failures.append(
                f"{path.relative_to(ROOT)} uses stale Rig validation command; use cargo test --features rig-agent"
            )

    required_validation_phrases = {
        BOOK_SRC / "overview.md": (
            "./scripts/check-book-readiness.sh",
            'RUN_LIVE_DEEPSEEK=1 DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY"',
        ),
        BOOK_SRC / "06-rig-boundary.md": (
            "cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features rig-agent",
        ),
        BOOK_SRC / "45-companion-code-reading-path.md": (
            "cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features rig-agent",
        ),
    }
    for path, phrases in required_validation_phrases.items():
        if not path.is_file():
            failures.append(f"missing validation command file: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8")
        for phrase in phrases:
            if phrase not in text:
                failures.append(
                    f"{path.relative_to(ROOT)} missing validation command phrase: {phrase}"
                )


def check_explanatory_paragraph_lengths(failures: list[str]) -> None:
    for chapter_name in sorted(DEEP_PEDAGOGY_CHAPTERS):
        path = BOOK_SRC / chapter_name
        if not path.is_file():
            failures.append(f"missing deep pedagogy chapter: {path.relative_to(ROOT)}")
            continue

        text = path.read_text(encoding="utf-8")
        for line_number, paragraph in explanatory_paragraphs(text):
            word_count = len(WORD_RE.findall(paragraph))
            if word_count > MAX_EXPLANATORY_PARAGRAPH_WORDS:
                failures.append(
                    f"{path.relative_to(ROOT)}:{line_number}: explanatory paragraph has {word_count} words; split it for low working-memory load"
                )


def check_chapter(path: Path, failures: list[str]) -> None:
    text = path.read_text(encoding="utf-8")
    chapter_headings = headings(text)
    display = path.relative_to(ROOT)

    if "motivation" not in chapter_headings:
        failures.append(f"{display} must include a '## Motivation' section")

    if not contains_mental_model_anchor(chapter_headings):
        failures.append(
            f"{display} must include a mental-model, example, mechanism, or invariant section"
        )

    if not contains_summary_heading(chapter_headings):
        failures.append(f"{display} must end with a summary section")

    if path.name in DEEP_PEDAGOGY_CHAPTERS:
        word_count = len(WORD_RE.findall(text))
        if word_count < MIN_DEEP_CHAPTER_WORDS:
            failures.append(
                f"{display} has {word_count} words; main teaching chapters need at least {MIN_DEEP_CHAPTER_WORDS} words of substantive explanation"
            )

        previous_index: int | None = None
        previous_name = ""
        for expected_heading, display_name in EXPECTED_DEEP_SECTION_ORDER:
            current_index = expected_section_index(chapter_headings, expected_heading)
            if current_index is None:
                continue
            if previous_index is not None and current_index <= previous_index:
                failures.append(
                    f"{display} section order drift: '{display_name}' must appear after '{previous_name}'"
                )
            previous_index = current_index
            previous_name = display_name

        if "production contract" not in chapter_headings:
            failures.append(f"{display} must include a '## Production Contract' section")

        if "formal definition" not in chapter_headings:
            failures.append(f"{display} must include a '## Formal Definition' section")
        else:
            formal_section = section_after_heading(
                text,
                "Formal Definition",
                (
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
                    "Further Reading and Sources",
                ),
            )
            for phrase in (
                "For this chapter, the precise definition is:",
                "State",
                "Actor",
                "Transition",
                "Evidence",
                "Invariant",
            ):
                if phrase not in formal_section:
                    failures.append(
                        f"{display} Formal Definition section must include '{phrase}'"
                    )
            for generic_phrase in (
                "the chapter's named production concept",
                "the component, worker, operator, or reviewer",
                "the chapter's mechanism changes",
                "the definition remains true",
            ):
                if generic_phrase in formal_section:
                    failures.append(
                        f"{display} Formal Definition section contains generic placeholder phrase: {generic_phrase}"
                    )

        if "what can fail" not in chapter_headings:
            failures.append(f"{display} must include a '## What Can Fail' section")
        else:
            failure_section = text.split("## What Can Fail", 1)[1]
            for next_heading in (
                "## Production Contract",
                "## Progressive Hardening Path",
                "## Testing Strategy",
                "## Observability Strategy",
                "## Security and Safety Considerations",
                "## Operational Checklist",
                "## Exercises",
                "## Self-Check",
                "## Summary",
            ):
                if next_heading in failure_section:
                    failure_section = failure_section.split(next_heading, 1)[0]
                    break
            for phrase in (
                "Design smell",
                "Production symptom",
                "Corrective invariant",
                "Evidence to inspect",
            ):
                if phrase not in failure_section:
                    failures.append(
                        f"{display} '## What Can Fail' section missing failure scaffold phrase: {phrase}"
                    )

        if "progressive hardening path" not in chapter_headings:
            failures.append(f"{display} must include a '## Progressive Hardening Path' section")
        else:
            progressive_section = section_after_heading(
                text,
                "Progressive Hardening Path",
                (
                    "Testing Strategy",
                    "Observability Strategy",
                    "Security and Safety Considerations",
                    "Operational Checklist",
                    "Exercises",
                    "Self-Check",
                    "Summary",
                    "Further Reading and Sources",
                ),
            )
            for phrase in ("Naive version", "Safer version", "Production version"):
                if phrase not in progressive_section:
                    failures.append(
                        f"{display} Progressive Hardening Path must include '{phrase}'"
                    )
            for generic_phrase in (
                "Fast to write, but the important production concept is still implicit and easy to lose during failure.",
                "The design names the state, type, boundary, or policy that prevents the category error.",
                "The invariant becomes durable evidence that tests, traces, runbooks, and operators can inspect.",
                "Use the naive version to recognize the beginner mistake.",
            ):
                if generic_phrase in progressive_section:
                    failures.append(
                        f"{display} Progressive Hardening Path contains generic rationale scaffolding: {generic_phrase}"
                    )
            progressive_section_lower = progressive_section.lower()
            evidence_anchors = (
                "rust",
                "sql",
                "postgres",
                "typed",
                "type",
                "durable",
                "evidence",
                "tests",
                "test",
                "traces",
                "trace",
                "runbooks",
                "runbook",
                "operator",
                "row",
                "receipt",
                "policy",
                "query",
                "review",
            )
            if sum(1 for phrase in evidence_anchors if phrase in progressive_section_lower) < 3:
                failures.append(
                    f"{display} Progressive Hardening Path must connect hardening to concrete Rust, SQL, evidence, test, trace, runbook, operator, row, receipt, policy, query, or review anchors"
                )

        if "testing strategy" not in chapter_headings:
            failures.append(f"{display} must include a '## Testing Strategy' section")
        else:
            testing_section = section_after_heading(
                text,
                "Testing Strategy",
                (
                    "Observability Strategy",
                    "Security and Safety Considerations",
                    "Operational Checklist",
                    "Exercises",
                    "Self-Check",
                    "Summary",
                    "Further Reading and Sources",
                ),
            )
            for phrase in ("Unit", "Persistence", "Regression", "Rust", "Postgres"):
                if phrase not in testing_section:
                    failures.append(
                        f"{display} Testing Strategy must include '{phrase}'"
                    )
            for generic_phrase in (
                "Test this chapter's invariant at three levels:",
                "prove the relevant Rust type, enum, typestate transition, or policy object rejects an invalid state.",
                "prove the Postgres row, SQL transition, provider adapter, or DTO conversion preserves the durable evidence.",
                "encode the failure from `What Can Fail` so a future change cannot reintroduce the design smell silently.",
            ):
                if generic_phrase in testing_section:
                    failures.append(
                        f"{display} Testing Strategy contains generic test scaffolding: {generic_phrase}"
                    )

        if "observability strategy" not in chapter_headings:
            failures.append(f"{display} must include a '## Observability Strategy' section")
        else:
            observability_section = section_after_heading(
                text,
                "Observability Strategy",
                (
                    "Security and Safety Considerations",
                    "Operational Checklist",
                    "Exercises",
                    "Self-Check",
                    "Summary",
                    "Further Reading and Sources",
                ),
            )
            for phrase in ("trace id", "structured `tracing`", "operation event", "runbook query"):
                if phrase not in observability_section:
                    failures.append(
                        f"{display} Observability Strategy must include '{phrase}'"
                    )
            for generic_phrase in (
                "Observe this chapter's invariant through durable evidence, not process memory:",
                "Include the relevant job id, run id, tool call id, trace id, attempt, status, and worker or actor in structured `tracing` fields.",
                "Record an operation event, audit event, or runbook query result that lets an operator reconstruct the state transition.",
                "Prefer event timelines and targeted SQL diagnostics over unstructured log searching when proving what happened.",
            ):
                if generic_phrase in observability_section:
                    failures.append(
                        f"{display} Observability Strategy contains generic observability scaffolding: {generic_phrase}"
                    )

        if "security and safety considerations" not in chapter_headings:
            failures.append(
                f"{display} must include a '## Security and Safety Considerations' section"
            )
        else:
            security_section = section_after_heading(
                text,
                "Security and Safety Considerations",
                (
                    "Operational Checklist",
                    "Exercises",
                    "Self-Check",
                    "Summary",
                    "Further Reading and Sources",
                ),
            )
            for phrase in ("untrusted", "authorization", "sandboxing", "approval", "Redact"):
                if phrase not in security_section:
                    failures.append(
                        f"{display} Security and Safety Considerations must include '{phrase}'"
                    )
            for generic_phrase in (
                "Apply the raw-outside, typed-inside rule before trusting this chapter's mechanism:",
                "Treat user input, model output, provider responses, database rows, and tool results as untrusted until parsed and validated.",
                "Require authorization, sandboxing, approval, or a documented no-side-effect decision before crossing an external side-effect boundary.",
                "Redact secrets and sensitive payloads from logs while keeping enough typed evidence for audit, incident response, and replay decisions.",
            ):
                if generic_phrase in security_section:
                    failures.append(
                        f"{display} Security and Safety Considerations contains generic security scaffolding: {generic_phrase}"
                    )

        if "operational checklist" not in chapter_headings:
            failures.append(f"{display} must include a '## Operational Checklist' section")
        else:
            checklist_section = section_after_heading(
                text,
                "Operational Checklist",
                ("Exercises", "Self-Check", "Summary", "Further Reading and Sources"),
            )
            for phrase in ("State", "Boundary", "Failure", "Observability", "Safety"):
                if phrase not in checklist_section:
                    failures.append(
                        f"{display} Operational Checklist must include '{phrase}'"
                    )
            for generic_phrase in (
                "Before applying this chapter's idea in production, verify:",
                "the concept has an explicit owner, lifecycle state, and durable evidence record.",
                "raw input from users, models, providers, or database rows is converted into typed domain values before business logic.",
                "at least one expected failure path records enough context for retry, escalation, or safe refusal.",
                "the related run, job, tool call, approval, or incident can be found by trace id and status.",
                "side effects are protected by idempotency, authorization, approval, or a documented no-side-effect decision.",
            ):
                if generic_phrase in checklist_section:
                    failures.append(
                        f"{display} Operational Checklist contains generic checklist scaffolding: {generic_phrase}"
                    )

        if "exercises" not in chapter_headings:
            failures.append(f"{display} must include a '## Exercises' section")
        else:
            exercises_section = section_after_heading(
                text,
                "Exercises",
                ("Self-Check", "Summary", "Further Reading and Sources"),
            )
            for marker in ("1.", "2.", "3."):
                if marker not in exercises_section:
                    failures.append(
                        f"{display} Exercises section must include exercise '{marker}'"
                    )
            for phrase in ("idempotency", "Postgres", "Rust", "negative test"):
                if phrase not in exercises_section:
                    failures.append(
                        f"{display} Exercises section must connect practice to '{phrase}'"
                    )
            for generic_phrase in (
                "Identify the first operation in this chapter that would be unsafe to repeat blindly.",
                "Sketch the minimal Postgres row or Rust type that would prove the production evidence named at the start of the chapter.",
                "Add one negative test, runbook query, or incident question that would fail if this chapter's invariant were broken.",
            ):
                if generic_phrase in exercises_section:
                    failures.append(
                        f"{display} Exercises section contains generic exercise scaffolding: {generic_phrase}"
                    )

        if not any(heading.startswith("tiny ") for heading in chapter_headings):
            failures.append(f"{display} must include a tiny example or tiny incident section")
        else:
            tiny_match = re.search(r"## Tiny[^\n]*\n\n(.*?)(?=\n## )", text, re.S)
            if tiny_match is None:
                failures.append(
                    f"{display} must include a readable tiny example or tiny incident section"
                )
            else:
                tiny_section = tiny_match.group(1)
                tiny_word_count = len(WORD_RE.findall(tiny_section))
                if tiny_word_count < MIN_DEEP_TINY_EXAMPLE_WORDS:
                    failures.append(
                        f"{display} tiny example section has {tiny_word_count} words; main teaching tiny examples need at least {MIN_DEEP_TINY_EXAMPLE_WORDS} words"
                    )
                for phrase in ("setup:", "transition:", "evidence:", "invariant:"):
                    if phrase not in tiny_section:
                        failures.append(
                            f"{display} tiny example section must include '{phrase}'"
                        )

    if path.name in MAIN_SCAFFOLD_CHAPTERS:
        if "what you will learn" not in chapter_headings:
            failures.append(f"{display} must include a '## What You Will Learn' section")
        else:
            learning_section = section_after_heading(
                text,
                "What You Will Learn",
                ("Motivation",),
            )
            learning_word_count = len(WORD_RE.findall(learning_section))
            if learning_word_count < MIN_DEEP_LEARNING_GOAL_WORDS:
                failures.append(
                    f"{display} What You Will Learn section has {learning_word_count} words; main teaching learning goals need at least {MIN_DEEP_LEARNING_GOAL_WORDS} words"
                )
            for phrase in (
                "This chapter teaches you to:",
                "explain",
                "inspect",
                "verify",
                "production evidence",
            ):
                if phrase not in learning_section:
                    failures.append(
                        f"{display} What You Will Learn section must include '{phrase}'"
                    )
            for generic_phrase in (
                "By the end of this chapter, you should be able to answer:",
                "You should also be able to name the production evidence:",
            ):
                if generic_phrase in learning_section:
                    failures.append(
                        f"{display} What You Will Learn section contains generic learning-goal scaffolding: {generic_phrase}"
                    )
        if "chapter thread" not in chapter_headings:
            failures.append(f"{display} must include a '## Chapter Thread' section")
        else:
            thread_section = section_after_heading(
                text,
                "Chapter Thread",
                (
                    "Motivation",
                    "Plain Version",
                    "What You Already Know",
                    "Focus Cue",
                    "Production Artifact",
                ),
            )
            for phrase in (
                "Read this chapter as one link in the production chain:",
                "**Builds on:**",
                "**Adds:**",
                "**Prepares:**",
            ):
                if phrase not in thread_section:
                    failures.append(
                        f"{display} Chapter Thread section must include '{phrase}'"
                    )
        if "what you already know" not in chapter_headings:
            failures.append(f"{display} must include a '## What You Already Know' section")
        else:
            prerequisite_section = section_after_heading(
                text,
                "What You Already Know",
                (
                    "Focus Cue",
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            prerequisite_word_count = len(WORD_RE.findall(prerequisite_section))
            if prerequisite_word_count < MIN_DEEP_PREREQUISITE_WORDS:
                failures.append(
                    f"{display} What You Already Know section has {prerequisite_word_count} words; main teaching prerequisite bridges need at least {MIN_DEEP_PREREQUISITE_WORDS} words"
                )
            for phrase in ("Start with these anchors:", "This chapter adds:"):
                if phrase not in prerequisite_section:
                    failures.append(
                        f"{display} What You Already Know section must include '{phrase}'"
                    )
            if prerequisite_section.count("\n- ") < 3:
                failures.append(
                    f"{display} What You Already Know section must list at least three prior anchors"
                )
        if "plain version" not in chapter_headings:
            failures.append(f"{display} must include a '## Plain Version' section")
        else:
            plain_section = section_after_heading(
                text,
                "Plain Version",
                (
                    "What You Already Know",
                    "Focus Cue",
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            plain_word_count = len(WORD_RE.findall(plain_section))
            if plain_word_count < MIN_DEEP_PLAIN_VERSION_WORDS:
                failures.append(
                    f"{display} Plain Version section has {plain_word_count} words; main teaching plain-language bridges need at least {MIN_DEEP_PLAIN_VERSION_WORDS} words"
                )
            for phrase in (
                "Read this as the simple version:",
                "**Simple rule:**",
                "**Why it matters:**",
                "**What to watch:**",
            ):
                if phrase not in plain_section:
                    failures.append(
                        f"{display} Plain Version section must include '{phrase}'"
                    )
        if "focus cue" not in chapter_headings:
            failures.append(f"{display} must include a '## Focus Cue' section")
        else:
            focus_section = section_after_heading(
                text,
                "Focus Cue",
                (
                    "Production Artifact",
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            for phrase in (
                "Keep three things in view:",
                "**State:**",
                "**Move:**",
                "**Proof:**",
                "state, move, and proof",
            ):
                if phrase not in focus_section:
                    failures.append(
                        f"{display} Focus Cue section must include '{phrase}'"
                    )
        if "production artifact" not in chapter_headings:
            failures.append(f"{display} must include a '## Production Artifact' section")
        else:
            artifact_section = section_after_heading(
                text,
                "Production Artifact",
                (
                    "Implementation Map",
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            for phrase in (
                "Build or inspect this artifact before moving on:",
                "**Artifact:**",
                "**Why it matters:**",
                "**Done when:**",
            ):
                if phrase not in artifact_section:
                    failures.append(
                        f"{display} Production Artifact section must include '{phrase}'"
                    )
        if "implementation map" not in chapter_headings:
            failures.append(f"{display} must include a '## Implementation Map' section")
        else:
            implementation_section = section_after_heading(
                text,
                "Implementation Map",
                (
                    "Operator Question",
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            for phrase in (
                "Use this map when you move from reading to implementation:",
                "**Primary surface:**",
                "**State transition:**",
                "**Evidence path:**",
            ):
                if phrase not in implementation_section:
                    failures.append(
                        f"{display} Implementation Map section must include '{phrase}'"
                    )
        if "operator question" not in chapter_headings:
            failures.append(f"{display} must include a '## Operator Question' section")
        else:
            operator_section = section_after_heading(
                text,
                "Operator Question",
                (
                    "Runtime Walkthrough",
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            for phrase in (
                "Before you ship this idea, answer one operational question:",
                "**Question:**",
                "**Evidence to inspect:**",
                "**Escalate if:**",
            ):
                if phrase not in operator_section:
                    failures.append(
                        f"{display} Operator Question section must include '{phrase}'"
                    )
        if "runtime walkthrough" not in chapter_headings:
            failures.append(f"{display} must include a '## Runtime Walkthrough' section")
        else:
            runtime_section = section_after_heading(
                text,
                "Runtime Walkthrough",
                (
                    "Acceptance Gate",
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            for phrase in (
                "Follow the concept as one runtime pass:",
                "**Trigger:**",
                "**Action:**",
                "**Persistence:**",
                "**Check:**",
            ):
                if phrase not in runtime_section:
                    failures.append(
                        f"{display} Runtime Walkthrough section must include '{phrase}'"
                    )
        if "acceptance gate" not in chapter_headings:
            failures.append(f"{display} must include a '## Acceptance Gate' section")
        else:
            acceptance_section = section_after_heading(
                text,
                "Acceptance Gate",
                (
                    "Micro-Lesson",
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            for phrase in (
                "Do not move on until this minimum evidence exists:",
                "**Minimum evidence:**",
                "**Validation path:**",
                "**Stop if:**",
            ):
                if phrase not in acceptance_section:
                    failures.append(
                        f"{display} Acceptance Gate section must include '{phrase}'"
                    )
        if "micro-lesson" not in chapter_headings:
            failures.append(f"{display} must include a '## Micro-Lesson' section")
        else:
            micro_lesson_section = section_after_heading(
                text,
                "Micro-Lesson",
                (
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            for phrase in (
                "Use this five-line version before the heavier mechanism:",
                "pain:",
                "rule:",
                "tiny example:",
                "artifact:",
                "proof:",
                "If the next section feels large",
            ):
                if phrase not in micro_lesson_section:
                    failures.append(
                        f"{display} Micro-Lesson section must include '{phrase}'"
                    )
        if "self-check" not in chapter_headings:
            failures.append(f"{display} must include a '## Self-Check' section")
        else:
            self_check_section = section_after_heading(
                text,
                "Self-Check",
                ("Summary", "Further Reading and Sources"),
            )
            self_check_word_count = len(WORD_RE.findall(self_check_section))
            if self_check_word_count < MIN_DEEP_SELF_CHECK_WORDS:
                failures.append(
                    f"{display} Self-Check section has {self_check_word_count} words; main teaching self-checks need at least {MIN_DEEP_SELF_CHECK_WORDS} words"
                )
            for phrase in ("Recall:", "Explain:", "Apply:", "Evidence:"):
                if phrase not in self_check_section:
                    failures.append(
                        f"{display} Self-Check section must include '{phrase}'"
                    )
        if "motivation" in chapter_headings:
            motivation_section = section_after_heading(
                text,
                "Motivation",
                (
                    "What You Already Know",
                    "Mental Model",
                    "Intuition",
                    "Tiny Example",
                    "Tiny Incident",
                    "Mechanism",
                    "Formal Definition",
                    "What Can Fail",
                    "Production Contract",
                ),
            )
            motivation_word_count = len(WORD_RE.findall(motivation_section))
            if motivation_word_count < MIN_DEEP_MOTIVATION_WORDS:
                failures.append(
                    f"{display} Motivation section has {motivation_word_count} words; main teaching motivations need at least {MIN_DEEP_MOTIVATION_WORDS} words"
                )
            if "In production" not in motivation_section:
                failures.append(
                    f"{display} Motivation section must start from production pain"
                )
            if "Without" not in motivation_section:
                failures.append(
                    f"{display} Motivation section must name what breaks without the concept"
                )
        if contains_summary_heading(chapter_headings):
            summary_section = section_after_heading(
                text,
                "Summary",
                ("Changed Understanding", "Further Reading and Sources"),
            )
            summary_word_count = len(WORD_RE.findall(summary_section))
            if summary_word_count < MIN_DEEP_SUMMARY_WORDS:
                failures.append(
                    f"{display} Summary section has {summary_word_count} words; main teaching summaries need at least {MIN_DEEP_SUMMARY_WORDS} words"
                )
            for phrase in ("Invariant", "Evidence"):
                if phrase not in summary_section:
                    failures.append(
                        f"{display} Summary section must name '{phrase}'"
                    )
        if "changed understanding" not in chapter_headings:
            failures.append(f"{display} must include a '## Changed Understanding' section")
        else:
            changed_section = section_after_heading(
                text,
                "Changed Understanding",
                ("Further Reading and Sources",),
            )
            changed_word_count = len(WORD_RE.findall(changed_section))
            if changed_word_count < MIN_DEEP_CHANGED_UNDERSTANDING_WORDS:
                failures.append(
                    f"{display} Changed Understanding section has {changed_word_count} words; main teaching chapters need a real before/after mental-model shift"
                )
            for phrase in (
                "**Before this chapter:**",
                "**After this chapter:**",
                "**Keep:**",
            ):
                if phrase not in changed_section:
                    failures.append(
                        f"{display} Changed Understanding section must include '{phrase}'"
                    )

    if "{{#include" not in text and path.name not in {
        "00-how-to-read-this-book.md",
        "00b-system-model-and-notation.md",
        "00c-design-principles.md",
        "00d-production-scope-trade-offs.md",
        "01-problem.md",
        "02-mental-model.md",
        "02b-guarantees-failure-semantics.md",
        "04b-typed-composition-lens.md",
        "07-running-system-locally.md",
        "08-production-hardening.md",
        "09-failure-modes.md",
        "10-capstone.md",
        "12-idempotency-side-effects.md",
        "13-leases-heartbeats-cancellation.md",
        "14-retry-backoff-dead-letters.md",
        "15-observability-slos.md",
        "17-testing-production-agents.md",
        "18-deployment-operations.md",
        "19-running-for-years.md",
        "20-final-production-blueprint.md",
        "20b-worked-production-scenario.md",
        "21-slis-slos-error-budgets.md",
        "22-capacity-backpressure-provider-quotas.md",
        "23-runbooks.md",
        "24-incident-response-postmortems.md",
        "25-release-engineering.md",
        "26-toil-automation-ownership.md",
        "28-security-abuse-trust-boundaries.md",
        "29-disaster-recovery-continuity.md",
        "30-reliability-maturity-model.md",
        "30c-temporal-after-postgres-first.md",
        "30d-kafka-after-postgres-first.md",
        "50-running-evidence-thread.md",
        "51-operator-control-surface.md",
        "52-maintenance-cadence.md",
    }:
        failures.append(
            f"{display} should include checked source snippets or be listed as a prose-only chapter"
        )


def main() -> int:
    failures: list[str] = []

    check_required_file(STYLE_GUIDE, failures)
    check_required_file(QUALITY, failures)
    check_quality_standard(failures)
    check_style_guide(failures)
    check_readiness_gate(failures)
    check_public_entry_points(failures)
    check_source_backed_code_policy(failures)
    check_summary_source_closure(failures)
    check_chapter_source_sections(failures)
    check_learning_path(failures)
    check_part_openers(failures)
    check_system_model(failures)
    check_design_principles(failures)
    check_scope_tradeoffs(failures)
    check_objective_bridge_terms(failures)
    check_observability_trace_context(failures)
    check_long_horizon_compatibility(failures)
    check_human_escalation_boundary(failures)
    check_glossary(failures)
    check_memory_chapter(failures)
    check_handoff_chapter(failures)
    check_worked_scenario(failures)
    check_design_review(failures)
    check_failure_drills(failures)
    check_readiness_scorecard(failures)
    check_chapter_checkpoints(failures)
    check_implementation_evidence_map(failures)
    check_system_diagrams(failures)
    check_end_to_end_labs(failures)
    check_principle_map(failures)
    check_case_studies(failures)
    check_concept_review(failures)
    check_concept_dependency_graph(failures)
    check_production_evidence_packets(failures)
    check_code_reading_path(failures)
    check_runtime_config_boundary(failures)
    check_live_deepseek_smoke(failures)
    check_design_smells(failures)
    check_reader_role_paths(failures)
    check_requirement_traceability(failures)
    check_attention_protocol(failures)
    check_chapter_card_pack(failures)
    check_plain_language_production_cards(failures)
    check_production_micro_drills(failures)
    check_production_build_milestones(failures)
    check_failure_first_learning_map(failures)
    check_core_failure_openings(failures)
    check_first_production_deployment_proof(failures)
    check_formal_definition_ledger(failures)
    check_validation_command_consistency(failures)
    check_public_publication_surface(failures)
    check_public_banned_phrases(failures)
    check_explanatory_paragraph_lengths(failures)

    for path in authored_chapters():
        check_chapter(path, failures)

    if failures:
        print("book pedagogy check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("book pedagogy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
