#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BOOK_DIR="$ROOT_DIR/books/postgres-rig-agent-jobs"

export PYTHONDONTWRITEBYTECODE=1

cd "$ROOT_DIR"

echo "== public mdBook source checks =="
python3 scripts/check-public-repo-surface.py
python3 scripts/check-pages-workflow.py
python3 scripts/check-public-chapter-structure.py
python3 scripts/check-book-terminology.py
python3 scripts/check-book-links.py
python3 scripts/check-book-code-contract.py
python3 scripts/check-rust-boundary-types.py

echo "== public Rust companion tests =="
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --all-features

echo "== public mdBook build =="
mdbook test "$BOOK_DIR"
mdbook build "$BOOK_DIR"
python3 scripts/write-public-build-info.py "$ROOT_DIR" "$BOOK_DIR/book"
find scripts -name __pycache__ -type d -prune -exec rm -rf {} +
find scripts -name '*.pyc' -delete
python3 scripts/check-public-mdbook-surface.py
