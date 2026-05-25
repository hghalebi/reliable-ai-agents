#!/usr/bin/env python3
"""Check local and further-reading links for the Reliable AI Agents book."""

from __future__ import annotations

import os
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BOOK_DIR = ROOT / "books" / "postgres-rig-agent-jobs"
BOOK_SRC = BOOK_DIR / "src"
FURTHER_READING = BOOK_SRC / "31-credible-resources-further-reading.md"
MARKDOWN_FILES = [
    ROOT / "README.md",
    *sorted(BOOK_SRC.glob("*.md")),
]

LINK_RE = re.compile(r"(!?)\[([^\]]*)\]\(([^)]+)\)")
EXTERNAL_LINK_RE = re.compile(r"^- \[([^\]]+)\]\((https://[^)]+)\)")
REQUIRED_EXTERNAL_SOURCES = {
    "https://www.anthropic.com/engineering/building-effective-agents",
    "https://rig.rs/",
    "https://docs.rs/rig-core/latest/index.html",
    "https://api-docs.deepseek.com/",
    "https://arxiv.org/abs/2210.03629",
    "https://arxiv.org/abs/2302.04761",
    "https://arxiv.org/abs/2303.11366",
    "https://arxiv.org/abs/2107.03374",
    "https://arxiv.org/abs/2310.06770",
    "https://www.postgresql.org/docs/current/sql-select.html",
    "https://www.postgresql.org/docs/current/explicit-locking.html",
    "https://www.postgresql.org/docs/current/transaction-iso.html",
    "https://www.oreilly.com/library/view/designing-data-intensive-applications/9781098119058/",
    "https://docs.temporal.io/",
    "https://docs.temporal.io/workflows",
    "https://docs.temporal.io/encyclopedia/event-history",
    "https://kafka.apache.org/intro/",
    "https://kafka.apache.org/documentation/",
    "https://kafka.apache.org/protocol/",
    "https://sre.google/",
    "https://sre.google/sre-book/testing-reliability/",
    "https://principlesofchaos.org/",
    "https://opentelemetry.io/docs/",
    "https://www.w3.org/TR/trace-context/",
    "https://github.com/openai/evals",
    "https://www.cast.org/what-we-do/universal-design-for-learning/",
    "https://www.cdc.gov/adhd/treatment/classroom.html",
    "https://digital.gov/guides/plain-language/principles/short-simple",
    "https://journals.sagepub.com/doi/10.1177/0963721420922183",
    "https://ies.ed.gov/ncee/wwc/PracticeGuide/1",
    "https://www.nature.com/articles/s44159-022-00089-1",
    "https://www.nist.gov/publications/artificial-intelligence-risk-management-framework-ai-rmf-10",
    "https://doi.org/10.6028/NIST.AI.600-1",
    "https://owasp.org/www-project-top-10-for-large-language-model-applications/",
    "https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html",
    "https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final",
    "https://tag-security.cncf.io/community/resources/security-whitepaper/v2/cloud-native-security-whitepaper/",
    "https://www.edpb.europa.eu/sme-data-protection-guide/respect-individuals-rights_en",
    "https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/data-protection-principles/a-guide-to-the-data-protection-principles/storage-limitation/",
    "https://atlas.mitre.org/",
    "https://doc.rust-lang.org/book/ch09-00-error-handling.html",
    "https://rust-lang.github.io/api-guidelines/",
    "https://docs.rs/thiserror",
    "https://rustsec.org/",
}
REQUIRED_APPENDIX_PHRASES = (
    "source of truth",
    "operational artifact",
    "production decision",
)
ALLOWED_EXTERNAL_DOMAINS = {
    "api-docs.deepseek.com",
    "apps.dtic.mil",
    "arxiv.org",
    "atlas.mitre.org",
    "aws.amazon.com",
    "brandur.org",
    "cheatsheetseries.owasp.org",
    "chrisgavin.dev",
    "cmmiinstitute.com",
    "codeascraft.com",
    "csrc.nist.gov",
    "dataintensive.net",
    "datatracker.ietf.org",
    "digital.gov",
    "dl.acm.org",
    "doc.rust-lang.org",
    "docs.aws.amazon.com",
    "docs.langchain.com",
    "docs.rs",
    "docs.rust-embedded.org",
    "docs.temporal.io",
    "doi.org",
    "en.wikipedia.org",
    "engineering.linkedin.com",
    "erikhollnagel.com",
    "fsharpforfunandprofit.com",
    "genai.ovasp.org",
    "github.com",
    "gocardless.com",
    "hamel.dev",
    "how.complexsystems.fail",
    "ico.org.uk",
    "ies.ed.gov",
    "ipc.on.ca",
    "journals.sagepub.com",
    "kafka.apache.org",
    "lamport.azurewebsites.net",
    "learntla.com",
    "lexi-lambda.github.io",
    "martinfowler.com",
    "medium.com",
    "mem0.ai",
    "microservices.io",
    "mitpress.mit.edu",
    "netflixtechblog.com",
    "openai.com",
    "opentelemetry.io",
    "owasp.org",
    "pagerduty.com",
    "planetscale.com",
    "pmg.csail.mit.edu",
    "principlesofchaos.org",
    "python.langchain.com",
    "research.google",
    "response.pagerduty.com",
    "restate.dev",
    "rig.rs",
    "rust-lang.github.io",
    "rustsec.org",
    "sre.google",
    "stripe.com",
    "tag-security.cncf.io",
    "teamtopologies.com",
    "temporal.io",
    "thinkrelevance.com",
    "tokio.rs",
    "udlguidelines.cast.org",
    "waitingforai.com",
    "web.mit.edu",
    "www.anthropic.com",
    "www.cast.org",
    "www.catchpoint.com",
    "www.cdc.gov",
    "www.confluent.io",
    "www.crunchydata.com",
    "www.cs.cornell.edu",
    "www.deeplearning.ai",
    "www.domainlanguage.com",
    "www.edpb.europa.eu",
    "www.enterpriseintegrationpatterns.com",
    "www.enterprisedb.com",
    "www.fema.gov",
    "www.honeycomb.io",
    "www.infoq.com",
    "www.ipc.on.ca",
    "www.milanjovanovic.tech",
    "www.nature.com",
    "www.nist.gov",
    "www.oreilly.com",
    "www.postgresql.org",
    "proptest-rs.github.io",
    "www.rudderstack.com",
    "www.sciencedirect.com",
    "www.usenix.org",
    "www.w3.org",
    "www.wiley.com",
    "www.youtube.com",
}


def is_external(target: str) -> bool:
    return target.startswith("http://") or target.startswith("https://")


def local_target(path: Path, raw_target: str) -> Path:
    target = raw_target.split("#", 1)[0].split("?", 1)[0]
    target = urllib.parse.unquote(target)
    return (path.parent / target).resolve()


def link_context(lines: list[str], start: int, max_lines: int = 5) -> str:
    return "\n".join(lines[start + 1 : start + 1 + max_lines]).lower()


def check_markdown_links(failures: list[str]) -> None:
    for path in MARKDOWN_FILES:
        if not path.is_file():
            failures.append(f"missing markdown file for link check: {path.relative_to(ROOT)}")
            continue

        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            for _image_marker, _label, raw_target in LINK_RE.findall(line):
                target = raw_target.strip()
                if target.startswith("#"):
                    continue
                if target.startswith("mailto:"):
                    continue
                if is_external(target):
                    parsed = urllib.parse.urlparse(target)
                    if parsed.scheme != "https":
                        failures.append(
                            f"{path.relative_to(ROOT)}:{line_number}: external links must use https: {target}"
                        )
                    if path != FURTHER_READING:
                        failures.append(
                            f"{path.relative_to(ROOT)}:{line_number}: external links belong in Appendix A: {target}"
                        )
                    if parsed.netloc not in ALLOWED_EXTERNAL_DOMAINS:
                        failures.append(
                            f"{path.relative_to(ROOT)}:{line_number}: unclassified external source domain: {parsed.netloc}"
                        )
                    continue

                resolved = local_target(path, target)
                if not resolved.exists():
                    failures.append(
                        f"{path.relative_to(ROOT)}:{line_number}: local link target does not exist: {target}"
                    )


def check_further_reading_shape(failures: list[str]) -> None:
    if not FURTHER_READING.is_file():
        failures.append(
            f"missing further-reading appendix: {FURTHER_READING.relative_to(ROOT)}"
        )
        return

    text = FURTHER_READING.read_text(encoding="utf-8")
    lines = text.splitlines()
    required_sections = (
        "## Agent Architecture",
        "## Agent Research and Evaluation Papers",
        "## Durable Execution and Data Systems",
        "## Reliability and Operations",
        "## Evaluation and Behavior Reliability",
        "## Learning Design and Plain Language",
        "## Security, Abuse, and Governance",
        "## Rust Engineering",
        "## Suggested Reading Paths",
        "## What to Avoid",
    )
    for section in required_sections:
        if section not in text:
            failures.append(
                f"{FURTHER_READING.relative_to(ROOT)} missing required section: {section}"
            )

    lower_text = text.lower()
    for phrase in REQUIRED_APPENDIX_PHRASES:
        if phrase not in lower_text:
            failures.append(
                f"{FURTHER_READING.relative_to(ROOT)} missing required reference-discipline phrase: {phrase}"
            )

    external_entries = []
    for index, line in enumerate(lines):
        match = EXTERNAL_LINK_RE.match(line)
        if not match:
            continue

        title, url = match.groups()
        external_entries.append(url)
        context = link_context(lines, index)
        if "read this" not in context:
            failures.append(
                f"{FURTHER_READING.relative_to(ROOT)}:{index + 1}: source lacks explicit reading rationale: {title}"
            )

    if len(external_entries) < 20:
        failures.append(
            f"{FURTHER_READING.relative_to(ROOT)} has {len(external_entries)} external sources; expected at least 20 credible sources"
        )

    missing_sources = REQUIRED_EXTERNAL_SOURCES - set(external_entries)
    for missing_source in sorted(missing_sources):
        failures.append(
            f"{FURTHER_READING.relative_to(ROOT)} missing required foundational source: {missing_source}"
        )


def check_external_link_health(failures: list[str]) -> None:
    if os.environ.get("RUN_EXTERNAL_LINK_CHECK") != "1":
        return

    for line_number, line in enumerate(FURTHER_READING.read_text(encoding="utf-8").splitlines(), 1):
        match = EXTERNAL_LINK_RE.match(line)
        if not match:
            continue

        title, url = match.groups()
        request = urllib.request.Request(url, method="GET", headers={"User-Agent": "ReliableAIAgentsLinkCheck/1.0"})
        try:
            with urllib.request.urlopen(request, timeout=10) as response:
                if response.status >= 400:
                    failures.append(
                        f"{FURTHER_READING.relative_to(ROOT)}:{line_number}: external link returned HTTP {response.status}: {title}"
                    )
        except Exception as error:  # noqa: BLE001 - report the concrete link failure.
            failures.append(
                f"{FURTHER_READING.relative_to(ROOT)}:{line_number}: external link check failed for {title}: {error}"
            )


def main() -> int:
    failures: list[str] = []
    check_markdown_links(failures)
    check_further_reading_shape(failures)
    check_external_link_health(failures)

    if failures:
        print("book link check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("book link check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
