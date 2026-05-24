# Reliable AI Agents

Reliable AI Agents is an mdBook and companion Rust project for building
production-grade AI agents on Rust, Rig, and Postgres.

The book treats an agent as a long-running production system, not a demo. The
core system uses Postgres for durable work, Rust for explicit boundaries, Rig
for model interaction, structured events for evidence, and small operational
gates for release, recovery, security, and evaluation.

## Project Layout

- `books/postgres-rig-agent-jobs`: mdBook source. Generated HTML is written to
  `books/postgres-rig-agent-jobs/book` by local and CI builds and is ignored.
- `examples/postgres-rig-agent-jobs`: executable Rust companion project.
  Generated Cargo artifacts stay under `examples/postgres-rig-agent-jobs/target`
  and are ignored.
- `scripts`: local validation, smoke, and readiness commands.

## Public Repository Boundary

The repository should contain learner-facing source, companion code, SQL,
scripts, and CI configuration. It should not contain local build output,
private authoring notes, private delivery notes, secrets, or machine-local
state.

The public-surface gate checks that `.gitignore` protects generated and
private/local paths such as:

- `books/*/book/`
- `examples/*/target/`
- `.env`
- `examples/*/.env`
- `.private/`, `private/`, `private-notes/`, `author-notes/`,
  `implementation-reports/`, and `reports/`
- `.idea/`, `__pycache__/`, `*.pyc`, `*.log`, and `*.local.md`

Run `python3 scripts/check-public-repo-surface.py` before publishing or
preparing a public push.

## Public Reading Path

Start here:

- `books/postgres-rig-agent-jobs/src/00-how-to-read-this-book.md`
- `books/postgres-rig-agent-jobs/src/00b-system-model-and-notation.md`
- `books/postgres-rig-agent-jobs/src/00c-design-principles.md`
- `books/postgres-rig-agent-jobs/src/00d-production-scope-trade-offs.md`

Use these appendices when you need a shorter path back into the system:

- `books/postgres-rig-agent-jobs/src/46-design-smells-failure-mode-index.md`
- `books/postgres-rig-agent-jobs/src/48-production-requirement-traceability.md`
- `books/postgres-rig-agent-jobs/src/49-formal-definition-ledger.md`
- `books/postgres-rig-agent-jobs/src/50-running-evidence-thread.md`
- `books/postgres-rig-agent-jobs/src/51-operator-control-surface.md`
- `books/postgres-rig-agent-jobs/src/52-maintenance-cadence.md`
- `books/postgres-rig-agent-jobs/src/first-production-deployment-proof.md`
- `books/postgres-rig-agent-jobs/src/plain-language-production-cards.md`
- `books/postgres-rig-agent-jobs/src/production-micro-drills.md`
- `books/postgres-rig-agent-jobs/src/production-build-milestones.md`
- `books/postgres-rig-agent-jobs/src/failure-first-learning-map.md`

The design-smell index helps readers recognize broken production designs. The
formal definition ledger turns each concept into state, actor, transition,
evidence, and invariant language. The running evidence thread follows one
agent job across the system. The operator control surface explains dashboards,
CLIs, and internal consoles. The maintenance cadence turns long-running
reliability into daily, weekly, monthly, quarterly, and incident-triggered
reviews.

The first production deployment proof turns one job kind into a launch packet.
The companion schema stores that packet in `job_kind_launch_packets`, and
`job_kind_launch_packet_status.sql` makes the first-user decision queryable.
The production build milestones turn the whole book into a build, inspect,
run, prove, and stop ladder.

Temporal and Kafka are covered as optional scaling chapters after the
Postgres-first system is explicit. Temporal is introduced for workflow
execution pressure, and Kafka is introduced for event distribution and replay
pressure. Neither is required for the beginner path.

For attention-heavy or interruption-heavy learning, use the chapter-card pack,
plain-language production cards, production micro-drills, production build
milestones, concept review appendix, and failure-first learning map. They give a
small public learning surface without lowering the production standard.

## License And Reuse

This is an active draft. Individual learning is allowed.

Non-commercial and nonprofit reuse of limited portions is allowed when you
clearly cite the book title, the author, the source page, and the relevant
sources.

Commercial reproduction, consulting use, client delivery, paid training,
company team training, internal business enablement, model training, retrieval
systems, or any use inside a business pipeline requires a separate written
license from Hamze Ghalebi. No commercial license is granted by this public
repository.

See [LICENSE.md](LICENSE.md) for the full license terms.

Keep the production context with reused material. The examples teach design
discipline, but each real system still needs its own tests, operational checks,
security review, legal review, and evidence before deployment.

## Build The Book

```bash
mdbook build books/postgres-rig-agent-jobs
```

## Test The Book

```bash
mdbook test books/postgres-rig-agent-jobs
```

## Check The Public mdBook Path

```bash
bash scripts/check-public-mdbook-ci.sh
```

This is the public Pages path. It checks the public repository surface, public
terminology, links, code-example include contracts, Rust domain-boundary
hygiene, chapter structure, and source shape. It also runs the companion Rust
tests, tests the mdBook examples, rebuilds the generated HTML, and rejects
non-learner-facing phrases from the rendered book.

## Test The Rust Example

```bash
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --all-features
```

## Run Static Checks

```bash
python3 scripts/check-public-repo-surface.py
python3 scripts/check-public-chapter-structure.py
python3 scripts/check-postgres-schema-contract.py
python3 scripts/check-book-code-contract.py
python3 scripts/check-cargo-dependency-policy.py
python3 scripts/check-rust-boundary-types.py
cargo clippy --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --all-targets --all-features -- -D warnings
```

## Run The Full Readiness Gate

```bash
./scripts/check-book-readiness.sh
```

This is the local proof command for the book and companion implementation. It
checks source hygiene, mdBook build/test, Rust formatting, feature-specific
tests, clippy, docs, audit, and dependency policy.

## Publish With GitHub Pages

The workflow in `.github/workflows/mdbook-pages.yml` builds the mdBook on
`main` when the book, companion examples, public checks, or workflow change. It
runs the public mdBook path above, uploads `books/postgres-rig-agent-jobs/book`,
and deploys that artifact to GitHub Pages. Generated HTML remains ignored
locally.

After deployment, check the public `build-info.json` next to the book. It records
the source revision, GitHub run id, generation time, and public CI path used to
produce the artifact.

## Run The Local Ephemeral Postgres Gate

```bash
RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh
```

This path uses local `initdb`, `pg_ctl`, and `psql` instead of Docker. It
creates a temporary database, applies both schemas, runs the Postgres-backed
worker, smokes the API server, executes runbook SQL files, verifies audited
pause/resume control events, and removes the temporary database.

## Run The Live Postgres Gate

```bash
RUN_LIVE_POSTGRES=1 ./scripts/check-book-readiness.sh
```

This starts the companion Postgres service with Docker, applies the schema,
runs the Postgres-backed worker, smokes the Postgres API server through
`/healthz`, `/readyz`, `/metrics`, and admission, then executes operator
runbook queries against the live database.

## Run The Live DeepSeek Gate

```bash
RUN_LIVE_DEEPSEEK=1 DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" ./scripts/check-book-readiness.sh
```

This runs the real Rig-backed DeepSeek binary, verifies the worker succeeds,
parses the typed result, and checks that the event timeline includes agent
start and success evidence. Normal readiness checks stay offline.

## Run The Local System

```bash
cargo run \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --bin postgres-rig-agent-jobs
```

## Optional DeepSeek Run

The live Rig example reads the model API key from the environment. The binary
checks that `DEEPSEEK_API_KEY` is present before starting the provider path,
but does not store or print the secret.

```bash
DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" cargo run \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features rig-agent \
  --bin deepseek-agent-demo
```

To run the packaged provider smoke directly:

```bash
DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" scripts/smoke-deepseek-agent.sh
```

## Optional Postgres Run

```bash
docker compose \
  -f examples/postgres-rig-agent-jobs/docker-compose.postgres.yml \
  up -d
```

Then apply the schema and run the Postgres API and worker binaries documented
in the book.

The Postgres binaries parse runtime environment variables at startup:
`DATABASE_URL` becomes a typed `DatabaseUrl`, and `BIND_ADDRESS` becomes a
typed HTTP bind address before the server or worker begins runtime work.
`RUST_LOG` and `LOG_FORMAT` become typed tracing configuration before any
worker, API, or provider demo emits runtime events. `LOG_FORMAT=compact` is the
default; `LOG_FORMAT=json` is available for production log pipelines.

```bash
DATABASE_URL="$DATABASE_URL" \
  BIND_ADDRESS="127.0.0.1:3000" \
  RUST_LOG="info,postgres_rig_agent_jobs=info" \
  LOG_FORMAT="json" \
  cargo run \
  --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml \
  --features api-server,postgres-store \
  --bin postgres-api-server
```

To smoke the API surface against an already migrated Postgres database:

```bash
DATABASE_URL="$DATABASE_URL" \
  BIND_ADDRESS="127.0.0.1:3000" \
  scripts/smoke-postgres-api.sh
```

To run the same Postgres smoke without Docker, use:

```bash
scripts/smoke-local-postgres.sh
```
