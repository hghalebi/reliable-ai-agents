# Reliable AI Agents

By Hamze Ghalebi

This book teaches how to turn unreliable probabilistic workers into governed,
observable, durable production systems.

The central rule is:

```text
The model may guess. The system must know.
```

This is not a book about calling an LLM API. It is a book about the control
system around the model: the typed, durable, permissioned, observable, audited
system that lets an AI agent act for real users without becoming impossible to
operate.

It teaches a production pattern for AI agents that must keep working for months
or years:

```text
Postgres stores durable agent work.
Rust workers execute the work.
Rig calls the language model.
Events explain what happened.
Evaluation catches behavior drift before users do.
Retries turn temporary failure into future work.
Policies keep risky action under human control.
```

The goal is not to build a clever script. The goal is to build a small,
typed, auditable system that keeps its promises when the model is slow, the
provider is down, the worker crashes, the process restarts, the network flakes,
the same webhook arrives twice, or an operator needs to understand what
happened three months later.

The companion code for this book lives at:

```text
examples/postgres-rig-agent-jobs
```

The default executable uses a deterministic local agent so it can be tested
without API keys. A real Rig-backed DeepSeek agent is available behind the
`rig-agent` feature and reads `DEEPSEEK_API_KEY` from the environment. A real
Postgres store is available behind the `postgres-store` feature.

The dependency graph is intentionally narrow. The code aliases `rig-core` as
`rig` and uses `sqlx-core` plus `sqlx-postgres` directly, so production checks
do not inherit unused companion integrations, SQLx macros, or database drivers
from facade crates.

## Reader Contract

This is a systems book. It does not teach agents as a prompt trick. It teaches
the machinery around the model:

```text
durable state
typed boundaries
provider isolation
policy gates
evaluation
observability
operator control
recovery
```

The promise is practical: after each part, you should know which invariant was
added, which failure it handles, and which test or runbook proves it.

The book is organized around engineering transformations, not around tools.
Rust, Postgres, and Rig appear because they implement the transformations:

```text
from raw input to trusted domain data
from model text to validated agent intent
from chat loop to durable agent run
from tool call to permissioned side effect
from retry to idempotent execution
from memory to governed state
from logs to traces, metrics, operation events, and audit evidence
from demo script to production workflow
```

Each serious chapter should move the reader through this ladder:

```text
naive demo -> failure -> intuition -> typed model -> minimal implementation -> production hardening -> tests and evals -> operational judgment
```

By the end, the reader should not merely recognize terms such as idempotency,
typestate, lease, audit event, evaluation, or human approval gate. The reader
should be able to implement the artifact, test the failure mode, inspect the
evidence, and judge when the pattern is too small or too heavy for the system
in front of them.

## What You Will Build

By the end, you will understand how to implement this loop and operate it:

```text
enqueue job
  -> pick due job with a lease
  -> record trace events
  -> run agent
  -> mark success or schedule retry
  -> recover expired jobs after crashes
  -> inspect, replay, cancel, and improve safely
```

The core idea:

```text
Do not build an AI script.
Build a durable job system where one step uses AI.
```

## Book Shape

Start with "How to Read This Book." It gives the chapter-by-chapter contract:
the dependency ladder, the question each chapter answers, and the production
evidence that would prove the concept in a real system.

Then read "System Model And Notation." It gives the book a small technical
language for jobs, states, leases, events, provider boundaries, policy
decisions, evaluation receipts, and side-effect receipts. Later chapters use
that language implicitly whenever they ask what changed, who was allowed to
change it, and what evidence remains.

Then read "Design Principles." It compresses the book's mechanisms into ten
production rules: durable before intelligent, typed before clever, ownership
before concurrency, boundary before policy, idempotent before retried, evidence
before operations, evaluation before behavior release, approval as state,
release with old work in mind, and practiced recovery.

Then read "Production Scope And Trade-Offs." It explains why this book teaches
a Postgres-first ledger, where that design is strong, and when a workflow
engine, queue framework, distributed platform, or simple script is the better
fit. A serious reader should know the operating envelope before copying an
architecture.

Part I builds the first working system. Each chapter starts from the pain the
concept solves, then builds the mental model, then points at executable Rust or
SQL.

Part II turns the system into production engineering material: real Postgres
integration, idempotency, leases, cancellation, error classification,
observability, SLOs, human approval, deployment, and long-horizon maintenance.
It closes with a worked production scenario that follows one risky request
through duplicate intake, retry, approval, side-effect receipt, and operator
review.

Part III treats the same system as an SRE-owned service: SLIs, SLOs, error
budgets, capacity, backpressure, provider quotas, concrete runbooks, incidents,
release safety, toil, automation, and ownership.

Part IV finishes the production picture: behavior evaluation, typed memory,
credential lifecycle review, retention and data-protection review, security and
abuse resistance, disaster recovery, maturity planning, and
evidence-preserving scaling paths for improving an agent system over multiple
years. Temporal and Kafka appear there as optional migrations: Temporal can take
over workflow execution mechanics, and Kafka can take over event distribution,
but neither replaces the typed Postgres product ledger or the Rig boundary.

Appendix A gives a source-backed reading map. It is not a bibliography for
decoration; it tells you which credible sources to consult when the weak part
of your system is durability, operations, evaluation, security, Rust design, or
provider integration.

Appendix B is the concept and misconception index. It maps the book's main
terms to the mental model, the failure each term prevents, and the production
evidence that proves the concept exists in a real system. It also repairs the
most common wrong mental models, such as treating a model call as the workflow,
a retry as safety, or logs as the audit trail.

Appendix C is the design review. It turns the book into a set of production
review questions, failure-injection prompts, and evidence standards for your
own agent system.

Appendix D is the practice surface. It gives failure drills with expected
reasoning so you can test whether the concepts transfer to new incidents,
provider changes, approval gaps, SLO alerts, and restore scenarios.

Appendix E is the readiness scorecard. It turns the book's ideas into an
evidence packet for one job kind at a time: target level, current proof, gaps,
next change, owner, and review date. The companion code backs this with
`job_kind_readiness_reviews`, `job_kind_readiness_review.sql`, and typed Rust
row conversion so maturity claims stay reviewable.

Appendix F is the checkpoint surface. It gives each chapter a prerequisite to
recall, a tiny mechanism to simulate, and a production artifact to name before
the reader moves on.

Appendix G is the implementation evidence map. It connects the book's
production controls to the exact Rust modules, SQL files, tests, and runbook
queries that prove the companion system implements them.

Appendix H is the visual systems appendix. It compresses the architecture,
state machine, lease timeline, retry timeline, approval path, observability
flow, release/versioning flow, and recovery flow into redrawable diagrams.

Appendix I is the lab surface. It asks you to change the system itself: add a
job kind, provider failure class, runbook query, policy gate, behavior
evaluation gate, restore drill, typed memory retention rule, and controlled
agent handoff, provider usage budget guard, timeout/cancellation policy, and
compensation action while preserving the production invariants.

Appendix J is the principle map. It connects each design principle to primary
chapters, a smallest simulation, production evidence, transfer questions, and
common review failures. Use it when the book starts to feel like a list of
techniques rather than a connected engineering model.

Appendix K gives production case studies. It applies the same principles to
incident triage, customer support replies, and billing adjustments, then marks
the demo, prototype, production, and regulated/high-risk versions of each
example so the reader can see how controls become stricter as job risk
increases.

Appendix L is the retrieval-practice surface. It gives each chapter a
Recall/Explain/Apply prompt so the reader can test whether the idea can be
reconstructed without the page open, applied to a new operational situation,
and tied to production evidence.

Appendix M is the concept dependency graph. It maps each chapter from
prerequisite, to new concept, to mechanism, to production capability unlocked,
so the book reads as one cumulative technical argument rather than a sequence
of isolated reliability topics.

Appendix N gives production evidence packets. It turns readiness, release,
incident, evaluation, security, and restore claims into reviewable artifacts
with owners, expiry, gaps, and proof.

Appendix O is the companion code reading path. It shows how to read the Rust
and SQL project from manifest, to domain types, to ledger, to store boundary,
to worker loop, to provider boundary, to binaries, to validation gate.

Appendix P is the design-smell and failure-mode index. It helps the reader
recognize the broken version of each chapter concept: the local shortcut, the
production symptom, the corrective invariant, and the evidence to inspect.

Appendix Q is the reader-role operating path. It gives AI engineers, Rust
engineers, platform and SRE engineers, security and governance reviewers, and
founders or technical leaders a way to inspect the same production model from
their own responsibility while still collecting durable evidence.

Appendix R is the production requirement traceability matrix. It starts from
the book's requirements, such as Postgres-backed scheduling, Rig boundaries,
newtypes, typestate, idempotency, leases, evaluation, security, and recovery,
then points to the teaching chapter, implementation artifact, validation
evidence, and reviewer question for each one.

Appendix S is the formal definition ledger. It compresses each main chapter's
concept into a precise production definition using the book's state, actor,
transition, evidence, and invariant notation, so intuition can become design
review language.

Appendix T is the running evidence thread. It follows one incident-triage
request across identity, timeline, chapter concepts, failure recovery, and
reviewer questions so the book reads as one production story rather than many
separate controls.

Appendix U is the operator control surface. It explains what a dashboard,
admin page, CLI, or internal console may show, what it may control, and which
permission and audit evidence every operator action must leave behind.

Appendix V is the maintenance cadence. It turns long-running reliability into
daily, weekly, monthly, quarterly, and incident-triggered evidence so "run for
years" becomes an operating calendar with owners, review packets, and missed
review escalation.

Appendix W is the attention-friendly learning protocol. It gives restart
points, chapter cards, two-pass reading, code reading, SQL reading, runbook
reading, study-session loops, and a Tiny Production Path for moving from one
durable proof to an agent that can run for years without lowering the
production standard.

Appendix X is the prefilled chapter-card pack. It turns every main chapter
into one small row: concept, artifact, proof, and operator question. Use it
when you need the shortest accurate route back into the book.

Appendix Y is the first production deployment proof. It turns the serious MVP
into a job-kind launch packet: one API process, one worker process, one Postgres database, one Rig boundary, one readiness gate, and one operator control path.
Use it before exposing a new agent job kind to real users.

Appendix Z is the plain-language production card pack. It gives one small
sentence, the exact term, the artifact to inspect, and the proof sentence for
hard terms such as lease, idempotency, release gate, restore drill, newtype,
typestate, and trust boundary.

Appendix AA is the production micro-drill pack. It gives seven-minute drills
for durable intake, typed boundaries, leases, retries, tool contracts,
approvals, receipts, traces, evaluation, release gates, security, recovery,
and maintenance. Each drill ends with one proof sentence tied to an
inspectable artifact.

Appendix AB is the production build milestone ladder. It turns the whole book
into a sequence of build, inspect, run, prove, and stop decisions: durable
intake, typed domain boundary, worker ownership, Rig provider boundary,
idempotent side effects, human approval, observability, evaluation, security,
recovery, operations, and evidence-preserving scale.

Appendix AC is the failure-first learning map. It turns every major chapter
into one production failure, one false shortcut, one surviving invariant, one
artifact, and one proof sentence. Use it when the book starts to feel like many
concepts instead of one sequence of failures made explicit, typed, durable, and
operable.

The teaching style is intuition-first and systems-oriented. Normal teaching
chapters move from problem, to intuition, to tiny example, to concrete
mechanism, to production invariant. A learner should be able to answer not only
"what code do I write?" but also:

```text
What invariant is this preserving?
What failure does this turn into recoverable state?
What should an operator see at 03:00?
What must never be hidden by a retry?
```

Every chapter should also answer four production questions:

```text
What can fail?
What state proves what happened?
Who or what is allowed to retry?
How would an operator know the system is safe?
```

## Validation

The full book and companion implementation are meant to be checked from the
repository root with the readiness gate:

```bash
./scripts/check-book-readiness.sh
```

That single command runs the book-source checks, mdBook test/build, Rust format
and test lanes, feature-specific test lanes, clippy, docs, dependency audit,
and dependency-policy checks.

The default gate does not require network access, Postgres, or model keys. Use
the optional gates only when you intentionally want live infrastructure proof:

```bash
RUN_LOCAL_POSTGRES=1 ./scripts/check-book-readiness.sh
RUN_LIVE_POSTGRES=1 ./scripts/check-book-readiness.sh
RUN_LIVE_DEEPSEEK=1 DEEPSEEK_API_KEY="$DEEPSEEK_API_KEY" ./scripts/check-book-readiness.sh
```

The DeepSeek run is a real provider call and should be run only when
`DEEPSEEK_API_KEY` is available.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
