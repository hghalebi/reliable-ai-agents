#!/usr/bin/env python3
"""Check the companion crate's narrow dependency policy.

Reliable AI Agents teaches a deliberately small production stack. This script
keeps that promise executable: broad facade crates and accidental runtime
dependencies should be an explicit design decision, not drift in Cargo.toml.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
CARGO_MANIFEST = ROOT / "examples" / "postgres-rig-agent-jobs" / "Cargo.toml"

ALLOWED_RUNTIME_DEPENDENCIES = {
    "async-trait",
    "axum",
    "chrono",
    "rig",
    "serde",
    "serde_json",
    "sqlx-core",
    "sqlx-postgres",
    "thiserror",
    "tokio",
    "tracing",
    "tracing-subscriber",
    "uuid",
}

ALLOWED_DEV_DEPENDENCIES = {
    "tower",
}

OPTIONAL_RUNTIME_DEPENDENCIES = {
    "axum",
    "rig",
    "sqlx-core",
    "sqlx-postgres",
}

REQUIRED_FEATURE_ITEMS = {
    "api-server": {"dep:axum", "tokio/net", "tokio/sync"},
    "rig-agent": {"dep:rig"},
    "postgres-store": {"dep:sqlx-core", "dep:sqlx-postgres"},
}

FORBIDDEN_DIRECT_DEPENDENCIES = {
    "anyhow",
    "sqlx",
    "sqlx-macros",
    "sqlx-mysql",
    "sqlx-sqlite",
    "reqwest",
    "openai",
    "redis",
    "rdkafka",
    "lapin",
    "temporal-sdk",
}


def dependency_names(section: dict[str, Any]) -> set[str]:
    return set(section.keys())


def is_optional(spec: Any) -> bool:
    return isinstance(spec, dict) and spec.get("optional") is True


def main() -> int:
    failures: list[str] = []

    if not CARGO_MANIFEST.is_file():
        print(f"missing Cargo manifest: {CARGO_MANIFEST.relative_to(ROOT)}", file=sys.stderr)
        return 1

    manifest = tomllib.loads(CARGO_MANIFEST.read_text(encoding="utf-8"))
    package = manifest.get("package", {})
    dependencies = manifest.get("dependencies", {})
    dev_dependencies = manifest.get("dev-dependencies", {})
    features = manifest.get("features", {})

    if package.get("publish") is not False:
        failures.append("companion crate must stay publish = false")

    runtime_names = dependency_names(dependencies)
    dev_names = dependency_names(dev_dependencies)

    unexpected_runtime = runtime_names - ALLOWED_RUNTIME_DEPENDENCIES
    if unexpected_runtime:
        failures.append(
            "unexpected runtime dependencies: " + ", ".join(sorted(unexpected_runtime))
        )

    unexpected_dev = dev_names - ALLOWED_DEV_DEPENDENCIES
    if unexpected_dev:
        failures.append(
            "unexpected dev dependencies: " + ", ".join(sorted(unexpected_dev))
        )

    forbidden_present = (runtime_names | dev_names) & FORBIDDEN_DIRECT_DEPENDENCIES
    if forbidden_present:
        failures.append(
            "forbidden direct dependencies present: "
            + ", ".join(sorted(forbidden_present))
        )

    default_features = set(features.get("default", []))
    if default_features:
        failures.append(
            "default feature set must stay empty; found: "
            + ", ".join(sorted(default_features))
        )

    for dep_name in sorted(OPTIONAL_RUNTIME_DEPENDENCIES):
        if dep_name not in dependencies:
            failures.append(f"optional runtime dependency missing: {dep_name}")
            continue
        if not is_optional(dependencies[dep_name]):
            failures.append(f"{dep_name} must stay optional and feature-gated")

    for feature_name, required_items in REQUIRED_FEATURE_ITEMS.items():
        actual_items = set(features.get(feature_name, []))
        missing = required_items - actual_items
        if missing:
            failures.append(
                f"feature {feature_name} missing required items: "
                + ", ".join(sorted(missing))
            )

    for feature_name in REQUIRED_FEATURE_ITEMS:
        if feature_name not in features:
            failures.append(f"required feature missing: {feature_name}")

    if failures:
        print("cargo dependency policy check failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("cargo dependency policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
