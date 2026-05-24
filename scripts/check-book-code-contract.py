#!/usr/bin/env python3
"""Check that learner-facing code blocks point at executable artifacts.

The book can use short excerpts, but ignored Rust snippets must not become
decorative pseudocode. If a Rust block is marked `ignore`, it must be an mdBook
include from the companion crate. SQL blocks must likewise include checked SQL
artifacts. The normal readiness gate then compiles/tests the crate and checks
the SQL registry.
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BOOK_SRC = ROOT / "books" / "postgres-rig-agent-jobs" / "src"
RUST_SOURCE_ROOT = ROOT / "examples" / "postgres-rig-agent-jobs" / "src"
SQL_SOURCE_ROOT = ROOT / "examples" / "postgres-rig-agent-jobs" / "sql"
CARGO_MANIFEST = ROOT / "examples" / "postgres-rig-agent-jobs" / "Cargo.toml"

FENCE_RE = re.compile(r"^```(?P<info>[^\n`]*)\n(?P<body>.*?)\n```", re.M | re.S)
INCLUDE_RE = re.compile(
    r"^\{\{#include\s+(?P<path>[^\s}:]+)(?::(?P<anchor>[A-Za-z0-9_-]+))?\}\}$"
)
SHELL_PATH_RE = re.compile(r"(?:\./)?(?:scripts|examples|books)/[A-Za-z0-9._/-]+")
SHELL_PLACEHOLDER_RE = re.compile(r"\.\.\.|YOUR_|REPLACE_|<[^>\n]+>")
CARGO_FEATURE_FLAG_RE = re.compile(r"--features(?:=|\s+)")
CARGO_MANIFEST_FLAG_RE = re.compile(r"--manifest-path(?:=|\s+)")
CARGO_BIN_FLAG_RE = re.compile(r"--bin(?:=|\s+)")


def rel(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def fence_language(info: str) -> str:
    if not info.strip():
        return ""
    return info.strip().split(",", 1)[0].strip()


def fence_options(info: str) -> set[str]:
    parts = [part.strip() for part in info.strip().split(",")]
    return set(parts[1:])


def resolve_include(chapter: Path, body: str, failures: list[str]) -> tuple[Path, str | None] | None:
    match = INCLUDE_RE.match(body.strip())
    if match is None:
        failures.append(
            f"{rel(chapter)} contains a code block that is not a single mdBook include"
        )
        return None

    include_path = (chapter.parent / match.group("path")).resolve()
    if not include_path.is_file():
        failures.append(
            f"{rel(chapter)} includes missing file: {match.group('path')}"
        )
        return None

    return include_path, match.group("anchor")


def require_under(path: Path, root: Path, chapter: Path, kind: str, failures: list[str]) -> None:
    try:
        path.relative_to(root.resolve())
    except ValueError:
        failures.append(
            f"{rel(chapter)} includes {kind} outside the checked companion artifacts: {path}"
        )


def require_anchor(path: Path, anchor: str | None, chapter: Path, failures: list[str]) -> None:
    if anchor is None:
        return

    text = path.read_text(encoding="utf-8")
    if f"ANCHOR: {anchor}" not in text:
        failures.append(
            f"{rel(chapter)} includes missing anchor {anchor!r} in {rel(path)}"
        )
    if f"ANCHOR_END: {anchor}" not in text:
        failures.append(
            f"{rel(chapter)} includes missing anchor end {anchor!r} in {rel(path)}"
        )


def check_rust_block(chapter: Path, info: str, body: str, failures: list[str]) -> None:
    options = fence_options(info)
    if "ignore" not in options:
        return

    resolved = resolve_include(chapter, body, failures)
    if resolved is None:
        return

    include_path, anchor = resolved
    require_under(include_path, RUST_SOURCE_ROOT, chapter, "Rust", failures)
    if include_path.suffix != ".rs":
        failures.append(f"{rel(chapter)} includes non-Rust file as Rust: {rel(include_path)}")
    require_anchor(include_path, anchor, chapter, failures)


def check_sql_block(chapter: Path, body: str, failures: list[str]) -> None:
    resolved = resolve_include(chapter, body, failures)
    if resolved is None:
        return

    include_path, anchor = resolved
    require_under(include_path, SQL_SOURCE_ROOT, chapter, "SQL", failures)
    if include_path.suffix != ".sql":
        failures.append(f"{rel(chapter)} includes non-SQL file as SQL: {rel(include_path)}")
    require_anchor(include_path, anchor, chapter, failures)


def check_bash_block(chapter: Path, body: str, failures: list[str]) -> None:
    """Keep shell snippets runnable instead of leaving placeholder commands."""

    for line_number, line in enumerate(body.splitlines(), start=1):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if SHELL_PLACEHOLDER_RE.search(stripped):
            failures.append(
                f"{rel(chapter)} bash block line {line_number} contains a non-executable placeholder: {stripped}"
            )

    for match in SHELL_PATH_RE.finditer(body):
        raw_path = match.group(0)
        candidate = raw_path[2:] if raw_path.startswith("./") else raw_path
        path = ROOT / candidate
        if not path.exists():
            failures.append(
                f"{rel(chapter)} bash block references missing repository path: {raw_path}"
            )


def known_cargo_features() -> set[str]:
    with CARGO_MANIFEST.open("rb") as file:
        manifest = tomllib.load(file)
    return set(manifest.get("features", {}))


def known_cargo_bins() -> set[str]:
    with CARGO_MANIFEST.open("rb") as file:
        manifest = tomllib.load(file)
    return {
        bin_config["name"]
        for bin_config in manifest.get("bin", [])
        if isinstance(bin_config, dict) and isinstance(bin_config.get("name"), str)
    }


def feature_tokens(raw: str) -> list[str]:
    return [
        token
        for token in re.split(r"[,\s]+", raw.strip())
        if token
    ]


def flag_values(line: str, flag_re: re.Pattern[str]) -> list[str]:
    groups: list[str] = []
    for match in flag_re.finditer(line):
        start = match.end()
        if start >= len(line):
            continue

        if line[start] in {"'", '"'}:
            quote = line[start]
            end = line.find(quote, start + 1)
            if end == -1:
                end = len(line)
            groups.append(line[start + 1 : end].strip())
            continue

        end = start
        while end < len(line) and not line[end].isspace() and line[end] not in {"`", "|"}:
            end += 1
        groups.append(line[start:end].rstrip("\\").strip())

    return groups


def documented_feature_groups(line: str) -> list[str]:
    return flag_values(line, CARGO_FEATURE_FLAG_RE)


def documented_manifest_paths(line: str) -> list[str]:
    return flag_values(line, CARGO_MANIFEST_FLAG_RE)


def documented_bin_names(line: str) -> list[str]:
    return flag_values(line, CARGO_BIN_FLAG_RE)


def check_documented_cargo_features(
    path: Path, text: str, available_features: set[str], failures: list[str]
) -> None:
    """Keep learner-facing Cargo commands aligned with Cargo.toml.

    Some commands live inside Markdown tables rather than fenced bash blocks.
    This catches stale feature names before readers copy a command that cannot
    run.
    """

    for line_number, line in enumerate(text.splitlines(), start=1):
        for raw_features in documented_feature_groups(line):
            for feature in feature_tokens(raw_features):
                if feature in available_features:
                    continue
                failures.append(
                    f"{rel(path)}:{line_number}: unknown Cargo feature in learner-facing command: {feature}"
                )


def check_documented_cargo_command_refs(
    path: Path, text: str, available_bins: set[str], failures: list[str]
) -> None:
    """Keep copied Cargo commands pointed at real manifests and binaries."""

    for line_number, line in enumerate(text.splitlines(), start=1):
        for raw_manifest_path in documented_manifest_paths(line):
            if "$" in raw_manifest_path:
                continue
            manifest_path = (ROOT / raw_manifest_path).resolve()
            if not manifest_path.is_file():
                failures.append(
                    f"{rel(path)}:{line_number}: Cargo command references missing manifest path: {raw_manifest_path}"
                )
                continue
            if manifest_path.name != "Cargo.toml":
                failures.append(
                    f"{rel(path)}:{line_number}: Cargo manifest path does not point to Cargo.toml: {raw_manifest_path}"
                )

        for bin_name in documented_bin_names(line):
            if "$" in bin_name:
                continue
            if bin_name in available_bins:
                continue
            failures.append(
                f"{rel(path)}:{line_number}: unknown Cargo binary in learner-facing command: {bin_name}"
            )


def main() -> int:
    failures: list[str] = []
    available_features = known_cargo_features()
    available_bins = known_cargo_bins()

    for chapter in sorted(BOOK_SRC.glob("*.md")):
        text = chapter.read_text(encoding="utf-8")
        check_documented_cargo_features(chapter, text, available_features, failures)
        check_documented_cargo_command_refs(chapter, text, available_bins, failures)
        for match in FENCE_RE.finditer(text):
            info = match.group("info").strip()
            body = match.group("body").strip()
            language = fence_language(info)

            if language == "rust":
                check_rust_block(chapter, info, body, failures)
            elif language == "sql":
                check_sql_block(chapter, body, failures)
            elif language == "bash":
                check_bash_block(chapter, body, failures)

    readme = ROOT / "README.md"
    if readme.is_file():
        readme_text = readme.read_text(encoding="utf-8")
        check_documented_cargo_features(readme, readme_text, available_features, failures)
        check_documented_cargo_command_refs(readme, readme_text, available_bins, failures)

    if failures:
        print("book code contract check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("book code contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
