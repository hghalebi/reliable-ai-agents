# Appendix A. Credible Resources and Further Reading

## How to Use This Appendix

A reliable agent is not built from one clever framework, one database trick, or
one model provider. It is built by connecting several mature engineering
disciplines: durable data systems, typed application code, production
operations, behavior evaluation, security, and governance.

Use this appendix as a guided reading map. Do not read everything at once. Pick
the source that strengthens the next weak part of your system:

```text
durable state weak -> read Postgres and data-systems material
operations weak -> read SRE and observability material
behavior weak -> read evaluation material
security weak -> read AI risk and LLM security material
Rust boundaries weak -> read Rust error and API design material
provider boundary weak -> read Rig and provider API docs
```

The standard for inclusion is credibility rather than popularity. The list
favors official documentation, standards-body guidance, foundation projects,
maintained API references, and established engineering books. Blog posts and
conference talks can be useful for ideas, but they should not be the source of
truth for production design.

## Reading Discipline

When you read a source, translate it into an operational artifact:

```text
database source -> migration, query invariant, or concurrency test
SRE source -> SLI, SLO, alert, runbook, or postmortem template
security source -> threat model, policy gate, audit log, or abuse test
evaluation source -> dataset, rubric, grader, or release gate
Rust source -> newtype, error type, trait boundary, or compile-time invariant
provider source -> request/response contract or compatibility test
```

This is the main habit. A serious operator does not collect references. A
serious operator converts references into checks, controls, and evidence.

## Agent Architecture

- [Anthropic: Building Effective Agents](https://www.anthropic.com/engineering/building-effective-agents)

  Read this for the distinction between workflows and agents, for the argument
  that agent complexity should be earned, and for the emphasis on transparent
  tool interfaces. In this book's language, it helps you decide whether a job
  should be a deterministic workflow step or a model-directed loop.

- [Rig: Build AI Applications in Rust](https://rig.rs/)

  Read this for the Rust-native agent/provider abstraction used by the
  companion crate. Use it to understand the shape of the system before reading
  lower-level API details.

- [Rig API reference on docs.rs](https://docs.rs/rig-core/latest/index.html)

  Read this when you need exact trait, provider, agent, completion, embedding,
  or vector-store API details. This is the correct source for compile-time API
  questions.

- [DeepSeek API docs](https://api-docs.deepseek.com/)

  Read this before changing model names, base URLs, request fields, or provider
  configuration. Provider API details drift; docs should be checked before a
  production release.

## Agent Research and Evaluation Papers

- [ReAct: Synergizing Reasoning and Acting in Language Models](https://arxiv.org/abs/2210.03629)

  Read this for the research pattern behind interleaving model reasoning with
  actions against an environment. In this book, ReAct is useful as a mental
  model for why reasoning traces and tool actions must be separated from durable
  state, policy, audit, and replay rules.

- [Toolformer: Language Models Can Teach Themselves to Use Tools](https://arxiv.org/abs/2302.04761)

  Read this for the tool-use research lineage: when to call an API, what
  arguments to pass, and how to incorporate the result. In production, this
  motivates typed tool contracts, validation, receipts, and policy gates around
  model-proposed tool use.

- [Reflexion: Language Agents with Verbal Reinforcement Learning](https://arxiv.org/abs/2303.11366)

  Read this for feedback-driven agent improvement and reflective memory. In
  this book, the production lesson is not "let the model remember anything"; it
  is that feedback and memory need typed scope, retention, confidence, and
  review controls before they influence future action.

- [Evaluating Large Language Models Trained on Code](https://arxiv.org/abs/2107.03374)

  Read this for HumanEval and execution-based evaluation. It is relevant beyond
  coding agents because it shows why evaluating outputs by executable behavior
  is stronger than judging fluent text alone.

- [SWE-bench: Can Language Models Resolve Real-World GitHub Issues?](https://arxiv.org/abs/2310.06770)

  Read this for realistic, repository-grounded evaluation of code agents. It is
  useful when designing regression datasets, issue-derived evaluation cases,
  and behavior gates for agent systems that modify real artifacts.

## Durable Execution and Data Systems

- [PostgreSQL `SELECT` documentation](https://www.postgresql.org/docs/current/sql-select.html)

  Read this for `FOR UPDATE`, `NOWAIT`, and `SKIP LOCKED`. These are the
  primitives behind cooperative worker leasing in this book.

- [PostgreSQL explicit locking documentation](https://www.postgresql.org/docs/current/explicit-locking.html)

  Read this when you need to reason about row locks, table locks, and how
  concurrent workers interact.

- [PostgreSQL transaction isolation documentation](https://www.postgresql.org/docs/current/transaction-iso.html)

  Read this before changing claim, lease, retry, or recovery queries. The queue
  is only as correct as its transaction boundaries.

- [PostgreSQL row-level security documentation](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)

  Read this when tenant isolation needs database-enforced row visibility in
  addition to application-level authorization, audit events, and runbook
  evidence.

- [PostgreSQL High Availability, Load Balancing, and Replication](https://www.postgresql.org/docs/current/high-availability.html)

  Read this for the database-level vocabulary behind replication, failover,
  standby servers, and high-availability architecture.

- [PostgreSQL synchronous commit documentation](https://www.postgresql.org/docs/current/runtime-config-wal.html#GUC-SYNCHRONOUS-COMMIT)

  Read this when you need to understand when an acknowledged transaction waits
  for WAL durability and how that changes failover confidence.

- [Designing Data-Intensive Applications](https://www.oreilly.com/library/view/designing-data-intensive-applications/9781098119058/)

  Read this second edition to deepen the mental model for durability,
  transactions, replication, consistency, logs, and long-lived data systems. It
  is the broader systems lens behind this book's smaller Postgres-backed design.

- [Temporal documentation](https://docs.temporal.io/)

  Read this as the contrasting durable-execution platform. It is useful even if
  you choose Postgres first, because it clarifies what a workflow engine gives
  you and what you must build yourself: history, replay, timers, retries,
  cancellation, workers, and workflow state.

- [Temporal Workflows](https://docs.temporal.io/workflows)

  Read this before moving a Postgres-backed job kind into a workflow engine. It
  explains workflow definitions, workflow executions, event history, replay,
  determinism, and the role of activities for external I/O.

- [Temporal Event History](https://docs.temporal.io/encyclopedia/event-history)

  Read this when designing reconciliation between workflow execution history
  and product evidence. It explains how commands become persisted events and
  how replay recreates workflow state after failure.

- [Temporal Activities](https://docs.temporal.io/activities)

  Read this before putting model calls, database writes, tool execution, or
  external API calls behind a workflow. It explains why activity code is where
  non-deterministic work belongs and why activities should be idempotent.

- [Temporal Rust SDK Workflows](https://docs.temporal.io/develop/rust/workflows)

  Read this when adapting the book's Rust examples to Temporal's Rust SDK. It
  shows the Rust workflow surface while the book's chapter keeps product
  evidence and domain types outside workflow-engine magic.

- [Apache Kafka Introduction](https://kafka.apache.org/intro/)

  Read this before adding event streaming to an agent system. It explains
  Kafka's core concepts: events, topics, producers, consumers, partitions,
  retention, and the ordering guarantee within a topic partition.

- [Apache Kafka Documentation](https://kafka.apache.org/documentation/)

  Read this as the primary project documentation for Kafka APIs,
  configuration, design, operations, security, Connect, and Streams when a
  Postgres outbox needs to feed a real streaming platform.

- [Apache Kafka Protocol](https://kafka.apache.org/protocol/)

  Read this when you need precise operational vocabulary around topics,
  partitions, offsets, consumer groups, and transactions. These terms become
  evidence fields once Kafka joins a production agent system.

- [Apache Kafka Design](https://kafka.apache.org/43/design/design/)

  Read this before relying on Kafka for ordering, consumer groups, replay, or
  transactional processing. It explains partition-level ordering, consumer
  positions, rewind/re-consumption, replication, and transaction mechanics.

## Reliability and Operations

- [Google SRE books and resources](https://sre.google/)

  Read this for the original SRE book, the SRE Workbook, and Building Secure &
  Reliable Systems. These sources give the operational vocabulary behind SLIs,
  SLOs, error budgets, toil, incidents, and release discipline.

- [Google SRE book introduction](https://sre.google/sre-book/introduction/)

  Read this to understand the core idea: operations treated as a software
  engineering problem. That idea is central to long-running agent systems,
  because reliability comes from designed mechanisms rather than heroics.

- [Google SRE chapter: Testing for Reliability](https://sre.google/sre-book/testing-reliability/)

  Read this before designing failure-injection, load, or disaster-recovery
  tests. It connects testing strategy to operational confidence rather than
  treating tests as a narrow pre-release checklist.

- [PlanetScale: The principles of extreme fault tolerance](https://planetscale.com/blog/the-principles-of-extreme-fault-tolerance)

  Read this for a concise production explanation of isolation, redundancy,
  static stability, control-plane/data-plane separation, progressive delivery,
  and repeated failover practice.

- [Principles of Chaos Engineering](https://principlesofchaos.org/)

  Read this when you want to turn expected failure modes into controlled
  experiments with a hypothesis, blast-radius limit, observation plan, and
  rollback path.

- [OpenTelemetry documentation](https://opentelemetry.io/docs/)

  Read this before designing traces, metrics, and logs for a production agent
  service. Agent systems need observability because their failures often unfold
  across model calls, tool calls, retries, and policy checks.

- [OpenTelemetry signals](https://opentelemetry.io/docs/concepts/signals/)

  Read this to separate traces, metrics, logs, baggage, events, and profiles.
  The distinction matters: a trace explains one job, a metric explains a fleet,
  and a log explains a point event.

- [W3C Trace Context](https://www.w3.org/TR/trace-context/)

  Read this before persisting or propagating trace identifiers across HTTP,
  workers, queues, and tool calls. It defines the trace-context format that
  lets independently instrumented systems correlate one logical operation.

## Evaluation and Behavior Reliability

- [OpenAI Evals](https://github.com/openai/evals)

  Read this for concrete patterns around creating and running evaluations for
  LLMs and LLM systems.

- [OpenAI: How evals drive the next chapter in AI for businesses](https://openai.com/index/evals-drive-next-chapter-of-ai/)

  Read this for practical evaluation habits: task-specific evals, rubrics,
  graders, and human review where risk requires it.

- [LangSmith evaluation docs](https://docs.langchain.com/langsmith/evaluation)

  Read this for a productized view of datasets, experiments, evaluators, and
  production trace evaluation. You do not need to adopt the product to learn
  the evaluation workflow shape.

## Learning Design and Plain Language

- [CAST: Universal Design for Learning](https://www.cast.org/what-we-do/universal-design-for-learning/)

  Read this for the book's commitment to UDL Guidelines 3.0 and multiple ways
  of entering the same technical idea: engagement, representation, action, and
  expression.

- [CDC: ADHD in the Classroom](https://www.cdc.gov/adhd/treatment/classroom.html)

  Read this for evidence-backed teaching supports around clear expectations,
  organization, routines, quick feedback, and accommodations for learners with
  ADHD.

- [Digital.gov: Short and Simple](https://digital.gov/guides/plain-language/principles/short-simple)

  Read this for concrete plain-language rules: know the audience, organize for
  the reader, avoid jargon, use clear words, and write short sentences.

- [Paas and van Merrienboer: Cognitive-Load Theory](https://journals.sagepub.com/doi/10.1177/0963721420922183)

  Read this for the working-memory reason behind worked examples, guidance
  fading, split-attention avoidance, and reducing unnecessary learner load.

- [IES: Organizing Instruction and Study to Improve Student Learning](https://ies.ed.gov/ncee/wwc/PracticeGuide/1)

  Read this for practice-guide support around spacing, active retrieval,
  worked examples, and connecting concrete and abstract representations.

- [Nature Reviews Psychology: Spacing and Retrieval Practice](https://www.nature.com/articles/s44159-022-00089-1)

  Read this for a research review of spacing and retrieval practice as durable
  learning strategies across domains.

## Security, Abuse, and Governance

- [NIST AI Risk Management Framework 1.0](https://www.nist.gov/publications/artificial-intelligence-risk-management-framework-ai-rmf-10)

  Read this for a vendor-neutral risk framework around trustworthy AI systems.

- [NIST AI RMF resources](https://www.nist.gov/itl/ai-risk-management-framework)

  Read this for the playbook, roadmap, crosswalks, and related implementation
  resources.

- [NIST AI RMF Generative AI Profile](https://doi.org/10.6028/NIST.AI.600-1)

  Read this when your agent can generate user-visible content, execute
  external actions, or influence business decisions. It maps generative-AI
  risks to concrete governance, measurement, and management actions.

- [OWASP Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/)

  Read this before giving an agent tools, memory, retrieved context, or
  externally supplied instructions.

- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)

  Read this when designing credential storage, rotation, auditability, access
  control, and incident response. It is the source behind the book's rule that
  Postgres may store secret references and lifecycle evidence, but not secret
  values.

- [NIST SP 800-57 Part 1 Rev. 5: Recommendation for Key Management](https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final)

  Read this for key-management vocabulary around key lifetimes, compromise,
  backup, recovery, and policy. It is useful when agent systems depend on API
  keys, encryption keys, service-account keys, or signing material.

- [CNCF TAG Security: Cloud Native Security Whitepaper v2](https://tag-security.cncf.io/community/resources/security-whitepaper/v2/cloud-native-security-whitepaper/)

  Read this for production security guidance on workload identity, credential
  management, short-lived secrets, rotation, revocation, and incident response.
  You do not need Kubernetes in the main path to learn from its credential
  lifecycle model.

- [OWASP Top 10:2025](https://owasp.org/Top10/2025/)

  Read this for the broader application security baseline: access control,
  misconfiguration, supply chain, injection, logging, and exceptional-condition
  handling.

- [European Data Protection Board: Respect Individuals' Rights](https://www.edpb.europa.eu/sme-data-protection-guide/respect-individuals-rights_en)

  Read this when data-protection requests need to become operational workflow:
  access, rectification, erasure, portability, objection, and accountability
  should be tracked as durable work, not as informal support notes.

- [ICO: Storage Limitation](https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/data-protection-principles/a-guide-to-the-data-protection-principles/storage-limitation/)

  Read this for retention-policy judgment: personal data should be kept only as
  long as necessary, reviewed against documented periods, and erased or
  anonymized unless there is a clear justification to keep it.

- [MITRE ATLAS](https://atlas.mitre.org/)

  Read this to build a threat-informed view of attacks against AI-enabled
  systems. Use it as a bridge between application security, AI-specific abuse,
  red-team exercises, and operational monitoring.

## Rust Engineering

- [The Rust Book: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

  Read this for the difference between recoverable errors, unrecoverable bugs,
  `Result`, and panic.

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

  Read this when shaping public types, constructors, naming, traits, and module
  boundaries. In this book, it is especially relevant to newtypes, builders,
  and provider boundaries.

- [thiserror documentation](https://docs.rs/thiserror)

  Read this for structured, typed error definitions without hand-written
  boilerplate.

- [Axum documentation](https://docs.rs/axum/latest/axum/)

  Read this before changing the optional HTTP admission service. Axum is only
  used at the system edge in this book, where request JSON and headers are
  converted into typed durable jobs.

- [RustSec Advisory Database](https://rustsec.org/)

  Read this for `cargo audit`, security advisories, and dependency audit
  practice.

## Suggested Reading Paths

The paths below are deliberately short. Each path gives you enough theory to
improve the system, then sends you back to code, tests, and operations.

### If You Are New to Reliable Agents

1. Anthropic: Building Effective Agents
2. ReAct and Toolformer
3. Rig docs
4. This book's Chapters 1-7
5. OpenAI Evals
6. OWASP Top 10 for LLM Applications

### If You Are a Backend or Platform Engineer

1. Designing Data-Intensive Applications
2. PostgreSQL locking, transaction isolation, and `SELECT` docs
3. Google SRE book and workbook
4. OpenTelemetry docs
5. This book's Chapters 11-26

### If You Own Security or Governance

1. NIST AI RMF 1.0
2. NIST AI RMF Generative AI Profile
3. OWASP Top 10 for LLM Applications
4. MITRE ATLAS
5. This book's Chapters 27-30

### If You Are Building Coding Or Tool-Using Agents

1. ReAct for the reasoning/action loop
2. Toolformer for tool-call research lineage
3. HumanEval for executable behavior evaluation
4. SWE-bench for repository-grounded regression cases
5. This book's Chapters 6, 12, 17, 27, and 28

### If You Are Implementing the Companion Rust System

1. The Rust Book: Error Handling
2. Rust API Guidelines
3. thiserror docs
4. Axum docs for the optional API edge
5. RustSec
6. Rig API reference

## Chapter-Specific Resources

### 00. How to Read This Book
- **[John Sweller: Cognitive Load Theory and CS Education](https://dl.acm.org/doi/10.1145/2839509.2844652)** (2016). Theoretical backing for the "Worked Example Effect" used in the book's learning loop.
- **[CAST: UDL Guidelines](https://udlguidelines.cast.org/)**. The pedagogical standard for accessible and multi-modal technical documentation.

### 00b. System Model and Notation
- **[Leslie Lamport: The Temporal Logic of Actions (TLA)](https://lamport.azurewebsites.net/tla/tla.html)** (1994). The formal source for state-actor-transition-invariant modeling.
- **[Hillel Wayne: Learn TLA+](https://learntla.com/)**. Practical applications of formal modeling to distributed system design.

### 00c. Design Principles
- **[Skelton & Pais: Team Topologies](https://teamtopologies.com/)** (2019). Connects system boundaries to cognitive load limits.
- **[Google SRE Book: The 5 Pillars of Quality](https://sre.google/sre-book/table-of-contents/)**. Industry-standard principles for observability, automation, and release safety.

### 00d. Production Scope and Trade-Offs
- **[Michael Nygard: Architecture Decision Records (ADR)](https://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions)** (2011). The standard for recording architectural constraints and trade-offs.
- **[Temporal: Why Workflow Engines?](https://temporal.io/blog/why-is-workflow-orchestration-hard)**. Explains the mechanical limits that motivate the move beyond a single-database design.

### 1. The Problem
- **[Anthropic: Building Effective Agents](https://www.anthropic.com/research/building-effective-agents)** (2025). Distinguishes between simple LLM workflows and autonomous agentic behavior.
- **[Brandur Leach: Transactionally Staged Job Drains in Postgres](https://brandur.org/job-drains)** (2017). The foundational post for the "Postgres Ledger" pattern used in this book.
- **[Temporal: What is Durable Execution?](https://temporal.io/blog/what-is-durable-execution)**. Compares "Fail-over" (Durable Execution) vs "Fail-fast" (RPC) models.
- **[AWS Builders' Library: Timeouts, Retries, and Backoff with Jitter](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/)**. Core distributed systems concepts for surviving ordinary failures.

### 2. The Mental Model
- **[Gul Agha: Actors—A Model of Concurrent Computation in Distributed Systems](https://mitpress.mit.edu/9780262010924/actors/)** (1986). The mathematical foundation for decoupled, asynchronous workers (actors) in a distributed environment.
- **[Stripe Engineering: Scaling Idempotency](https://stripe.com/blog/idempotency)**. A practical industry reference for separating intent from execution using stable idempotency keys.

### 2.5 Guarantees and Failure Semantics
- **[Chris Richardson: The Transactional Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)**. Explains how to reliably trigger external side effects (like agent tools) without distributed transactions.
- **[Pat Helland: Life Beyond Distributed Transactions—An Apostate's Opinion](https://waitingforai.com/wp-content/uploads/2021/05/helland-life-beyond-distributed-transactions.pdf)** (2007). Seminal paper on managing data in distributed systems via "entities" and messaging.
- **[Milan Jovanović: Exactly-once is impossible, but Idempotency is not](https://www.milanjovanovic.tech/blog/idempotent-consumer-pattern-in-dotnet)**. Deep-dive into failure semantics and why "at-least-once" is the robust engineering choice.

### 3. The Postgres Ledger
- **[Jim Gray: Queues are Databases](https://arxiv.org/abs/cs/0701158)** (1995). The classic academic argument for moving queuing logic into the database for transactional safety.
- **[Craig Ringer (2ndQuadrant): What is SELECT SKIP LOCKED?](https://www.enterprisedb.com/blog/what-is-select-skip-locked-postgresql-9-5)**. The engineering context behind the feature that powers the cooperative worker model in this book.
- **[RudderStack: Scaling PostgreSQL Queues to 100k Events/sec](https://www.rudderstack.com/blog/postgresql-as-our-main-streaming-engine-and-queuing-system/)**. Demonstrates the scalability of the ledger pattern in production environments.

### 4. The Rust Domain Model
- **[Alexis King: Parse, don't validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)** (2019). The foundational text for type-driven design.
- **[Scott Wlaschin: Domain Modeling Made Functional](https://fsharpforfunandprofit.com/books/)**. A definitive guide to making illegal states unrepresentable through types.

### 4.5 Typed Composition Lens
- **[The Embedded Rust Book: Typestate Programming](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html)**. Explains how to enforce lifecycle states and transitions at compile time.
- **[Kohei Honda: Session Types and Distributed Computing](https://dl.acm.org/doi/10.1145/2103736.2103744)** (2012). The seminal paper on verifying distributed interaction protocols through type systems.

### 5. The Worker Loop
- **[Gray & Cheriton: Leases—An Efficient Mechanism for Distributed Consistency](https://dl.acm.org/doi/10.1145/74850.74870)** (1989). The academic origin of time-bound resource ownership (leases).
- **[Chandra & Toueg: Unreliable Failure Detectors](https://dl.acm.org/doi/10.1145/224964.224992)** (1996). Foundational paper on heartbeat-based failure detection in distributed systems.
- **[Enterprise Integration Patterns: Competing Consumers](https://www.enterpriseintegrationpatterns.com/patterns/messaging/CompetingConsumers.html)**. Definitive guide to scaling message consumption through worker pools.

### 6. The Rig Boundary
- **[Schick et al.: Toolformer—LM Tool Use](https://arxiv.org/abs/2302.04761)** (2023). Academic basis for model-powered API interaction.
- **[Yao et al.: ReAct—Reasoning and Acting](https://arxiv.org/abs/2210.03629)** (2022). The research motivating reasoning/action loops at the intelligence boundary.

### 7. Running The System Locally
- **[FoundationDB: Deterministic Simulation Testing](https://www.youtube.com/watch?v=4fFDFbi3toc)**. The gold standard for proving distributed systems via local deterministic runs.
- **[Will Wilson: Testing the Untestable](https://www.youtube.com/watch?v=fFSPwJFXVlU)**. Practical techniques for deterministic simulation and concurrency testing.

### 8. Production Hardening
- **[Brandur Leach: Idempotency Keys](https://brandur.org/idempotency-keys)**. The canonical guide to protecting API boundaries from duplicate work.
- **[AWS Builders' Library: Avoiding Overload](https://aws.amazon.com/builders-library/avoiding-overload-in-distributed-systems-by-putting-the-smaller-service-in-control/)**. Explains the mechanical protections for background worker systems.

### 9. Failure Modes
- **[Eric Brewer: CAP Twelve Years Later](https://www.infoq.com/articles/cap-twelve-years-later-how-the-rules-have-changed/)**. Contextualizes agent job failures (ownership vs. availability) within distributed systems theory.
- **[Abadi: PACELC Theorem](https://en.wikipedia.org/wiki/PACELC_theorem)**. Describes the trade-offs between latency and consistency during failure recovery.

### 10. Capstone
- **[Brandur Leach: The Serious MVP](https://brandur.org/serious-mvp)**. A manifesto for building robust, database-first systems before adding feature complexity.

### 11. The Real Postgres Store
- **[Martin Fowler: Data Mapper Pattern](https://martinfowler.com/eaaCatalog/dataMapper.html)** (2002). Explains the architectural reason for the row-to-domain conversion layer.
- **[Eric Evans: Repository Pattern](https://www.domainlanguage.com/ddd/)** (2003). Standardizes the collection-like interface used to manage durable jobs.
- **[SQLx docs.rs](https://docs.rs/sqlx/latest/sqlx/)**. Documentation for the compile-time verified SQL toolkit used in the implementation.

### 12. Idempotency and Side Effects
- **[Airbnb Engineering: Avoiding Double Payments](https://medium.com/airbnb-engineering/avoiding-double-payments-in-a-distributed-system-29c35d03e536)**. A world-class case study on using idempotency keys and leases to achieve high consistency.
- **[Garcia-Molina & Salem: Sagas](https://www.cs.cornell.edu/andru/cs711/2002fa/reading/sagas.pdf)** (1987). The academic origin of compensating transactions.
- **[Pat Helland: Life Beyond Distributed Transactions](https://waitingforai.com/wp-content/uploads/2021/05/helland-life-beyond-distributed-transactions.pdf)** (2007). Seminal paper on reliability through messaging and idempotency.

### 13. Leases, Heartbeats, and Cancellation
- **[Hayashibara et al.: The Phi Accrual Failure Detector](https://dl.acm.org/doi/10.1145/1028174.1028177)** (2004). Advanced research on failure detection under network uncertainty.
- **[Tokio: Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown)**. The industry-standard guide to cooperative cancellation in Rust.
- **[Martin Fowler: Pattern—Heartbeat](https://martinfowler.com/articles/patterns-of-distributed-systems/heartbeat.html)**. Explains the use of liveness signals for session management.

### 14. Retry, Backoff, and Dead Letters
- **[Metcalfe & Boggs: Ethernet](https://dl.acm.org/doi/10.1145/360248.360253)** (1976). The origin of the exponential backoff algorithm.
- **[Marc Brooker: Exponential Backoff and Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)**. The primary industry reference for avoiding retry storms through randomness.
- **[Google SRE: Retry Budgets](https://sre.google/sre-book/addressing-cascading-failures/#id-Vv9I8)**. Architectural strategy for limiting load amplification during failures.

### 15. Observability and SLOs
- **[Google: Dapper Tracing Paper](https://research.google/pubs/dapper-a-large-scale-distributed-systems-tracing-infrastructure/)** (2010). The academic foundation for distributed tracing and context propagation.
- **[W3C: Trace Context Standard](https://www.w3.org/TR/trace-context/)**. The industry standard for the `traceparent` header used in this book.
- **[Google SRE: SLOs](https://sre.google/sre-book/service-level-objectives/)**. Best practices for defining and measuring service-level objectives.
- **[Charity Majors: Observability vs Monitoring](https://www.honeycomb.io/blog/observability-vs-monitoring)**. Explains the shift from static dashboards to event-driven debugging.

### 16. Human Approval and Policy Gates
- **[Amodei et al.: Concrete Problems in AI Safety](https://arxiv.org/abs/1606.06565)** (2016). Research on human-in-the-loop systems for scalable oversight.
- **[NIST: AI Risk Management Framework](https://www.nist.gov/itl/ai-rmf)**. Detailed guidance on human oversight and sociotechnical AI safety.
- **[OWASP: LLM02 Insecure Output Handling](https://genai.ovasp.org/llm-02-insecure-output-handling/)**. Industry reference for the risks of trusting model-generated instructions.

### 17. Testing Production Agents
- **[Principles of Chaos Engineering](https://principlesofchaos.org/)**. Industry-standard methodology for resilience testing through fault injection.
- **[The proptest book](https://proptest-rs.github.io/proptest/intro.html)**. Documentation for property-based testing in Rust.
- **[AgentBench (ICLR 2024)](https://arxiv.org/abs/2308.03688)**. Academic rubric for evaluating multi-dimensional agent performance.
- **[OpenAI Evals](https://github.com/openai/evals)**. Framework for repeatable model-behavior testing and grading.

### 18. Deployment and Operations
- **[Google SRE: Release Engineering](https://sre.google/sre-book/release-engineering/)**. Principles of automated, high-velocity, and safe deployments.
- **[AWS Builders' Library: Rollback Safety](https://aws.amazon.com/builders-library/ensuring-rollback-safety-during-deployments/)**. Strategies for ensuring breaking changes can be safely reversed.
- **[GoCardless: Zero-Downtime Postgres Migrations](https://gocardless.com/blog/zero-downtime-postgres-migrations-the-hard-parts/)**. Best practices for schema changes under load.
- **[NIST: Key Management Guidelines](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)**. Formal lifecycle for managing secrets and credentials.

### 19. Running for Years
- **[Lehman: Laws of Software Evolution](https://en.wikipedia.org/wiki/Lehman%27s_laws_of_software_evolution)**. Foundational research on the inevitable forces governing long-lived systems.
- **[Crunchy Data: Autovacuum Tuning](https://www.crunchydata.com/blog/postgres-autovacuum-tuning)**. Industry guide to managing database bloat and performance over time.
- **[EDPB: Data Protection by Design](https://www.edpb.europa.eu/our-work-tools/our-documents/guidelines/guidelines-42019-article-25-data-protection-design-and_en)**. The standard for automated data retention and erasure workflows.

### 20. Final Production Blueprint
- **[Michael Wooldridge: Multiagent Systems](https://www.wiley.com/en-us/An+Introduction+to+Multiagent+Systems%2C+2nd+Edition-p-9780470519462)**. The definitive textbook on agent coordination and responsibility allocation.
- **[Andrew Ng: Agentic Workflows](https://www.deeplearning.ai/the-batch/how-agents-can-improve-llm-performance/)**. Industry guide to iterative, step-based agent orchestration.

### 20.1 Agent Handoffs and Coordination
- **[Wooldridge: Coordination Protocols](https://dl.acm.org/doi/10.5555/543666)**. Academic formalisms for Joint Intentions and verifiable work transfer.
- **[Hancock et al.: Trust in HRI](https://journals.sagepub.com/doi/10.1177/0018720811417254)**. Research defining the pillars of competence and integrity in agent delegation.

### 20.2 Worked Production Scenario
- **[Pat Helland: Memories, Guesses, and Apologies](https://arxiv.org/abs/2005.02103)** (2020). Architectural logic for dealing with uncertainty and compensation in distributed agent workflows.

### 21. SLIs, SLOs, and Error Budgets
- **[Google SRE: Service Level Objectives](https://sre.google/sre-book/service-level-objectives/)**. The foundational guide to measuring and managing service reliability.
- **[The SRE Workbook: Implementing SLOs](https://sre.google/workbook/implementing-slos/)**. Practical step-by-step recipe for defining user-centric reliability targets.
- **[Google SRE: Burn Rate Alerting](https://sre.google/workbook/alerting-on-slos/)**. Advanced strategy for monitoring error budget consumption.

### 22. Capacity, Backpressure, and Provider Quotas
- **[John Little: A Proof for the Queuing Formula](https://web.mit.edu/e-strategy/www/Little.pdf)** (1961). The mathematical basis for modeling concurrency and throughput.
- **[John Nagle: Congestion Control in IP/TCP (RFC 896)](https://datatracker.ietf.org/doc/html/rfc896)** (1984). Seminal work on preventing system collapse through flow control.
- **[AWS Builders' Library: Load Shedding](https://aws.amazon.com/builders-library/using-load-shedding-to-avoid-overload/)**. Industry patterns for protecting systems from overload by refusing work.

### 23. Runbooks
- **[Richard Cook: How Complex Systems Fail](https://how.complexsystems.fail/)** (1998). Foundational insights on the role of human operators in maintaining safety in hazardous systems.
- **[Google SRE: Managing Incidents](https://sre.google/sre-book/managing-incidents/)**. The Incident Command System (ICS) framework for structured outage management.
- **[PagerDuty: Incident Response](https://response.pagerduty.com/)**. Definitive guide to the industry-standard incident lifecycle and responder roles.

### 24. Incident Response and Postmortems
- **[John Allspaw: Blameless Postmortems](https://codeascraft.com/2012/05/22/blameless-postmortems/)** (2012). The industry-standard philosophy for learning from failure without punishment.
- **[James Reason: The Swiss Cheese Model](https://en.wikipedia.org/wiki/Swiss_cheese_model)**. A core safety science concept for understanding layered defenses and latent conditions.
- **[Erik Hollnagel: Safety-II](https://erikhollnagel.com/ideas/safety-i-and-safety-ii.html)**. Shifting from failure-centric safety to success-oriented resilience engineering.

### 25. Release Engineering for Agents
- **[Walter F. Tichy: Tools for SCM](https://dl.acm.org/doi/10.5555/54366.54370)** (1988). Foundational taxonomy for version control and configuration management.
- **[Martin Fowler: The Expand and Contract Pattern](https://martinfowler.com/bliki/ParallelChange.html)**. Core pattern for zero-downtime schema and code transitions.
- **[Netflix: Automated Canary Analysis (Kayenta)](https://netflixtechblog.com/automated-canary-analysis-at-netflix-with-kayenta-3260bc7acc69)**. Advanced techniques for statistical release validation.

### 26. Toil, Automation, and Ownership
- **[Google SRE: Eliminating Toil](https://sre.google/sre-book/eliminating-toil/)**. The industry-standard definition and management strategy for manual repetitive work.
- **[Lisanne Bainbridge: Ironies of Automation](https://en.wikipedia.org/wiki/Ironies_of_Automation)** (1983). Seminal research on the pitfalls of over-automation and the human-out-of-the-loop problem.
- **[Sheridan & Verplank: Levels of Automation](https://apps.dtic.mil/sti/citations/ADA054659)** (1978). The 10-level framework for allocating responsibility between humans and software.

### 27. Evaluation and Behavior Reliability
- **[Zheng et al.: Judging LLM-as-a-Judge](https://arxiv.org/abs/2306.05685)** (2023). Formalizes the use of highly capable models as automated proxies for human preference.
- **[Hamel Husain: Evaluation-Driven Development](https://hamel.dev/blog/posts/evals/)**. Practical guide to building domain-specific, binary evaluation pipelines for LLM products.
- **[Cohen's Kappa ($\kappa$)](https://en.wikipedia.org/wiki/Cohen%27s_kappa)**. The statistical standard for measuring inter-rater reliability and calibration in evaluation systems.

### 27.5 Agent Memory, Retrieval, and Retention
- **[A-MemGuard: Agent Memory Defense](https://mem0.ai/)** (2025). Emerging research on mitigating persistent memory poisoning.
- **[MINJA: Memory Injection Attacks](https://arxiv.org/abs/2406.01258)** (2024). Foundational research on the high success rate of agent memory exploits.
- **[Morris et al.: Text Embeddings Reveal Text](https://arxiv.org/abs/2310.06816)** (ACL 2024). Proves the privacy risks inherent in raw vector embeddings.

### 28. Security, Abuse, and Trust Boundaries
- **[Myers & Liskov: Decentralized IFC](https://dl.acm.org/doi/10.1145/266635.266669)** (1997). The academic origin of labels and trust boundaries in secure systems.
- **[OWASP: Top 10 for LLM Applications](https://genai.ovasp.org/)**. The industry standard for modern agent vulnerabilities.
- **[MITRE ATLAS](https://atlas.mitre.org/)**. Knowledge base of adversarial tactics for AI systems.

### 28.5 Data Protection and Privacy Operations
- **[Ann Cavoukian: Privacy by Design](https://www.ipc.on.ca/wp-content/uploads/2013/09/pbd-primer.pdf)**. The 7 foundational principles for embedding privacy into system architecture.
- **[EDPB: Data Protection by Design Guidelines](https://www.edpb.europa.eu/our-work-tools/our-documents/guidelines/guidelines-42019-article-25-data-protection-design-and_en)**. Regulatory standards for automated privacy workflows.

### 28.6 Tenant Isolation and Multi-tenancy
- **[AWS: SaaS Tenant Isolation Strategies](https://docs.aws.amazon.com/whitepapers/latest/saas-tenant-isolation-strategies/welcome.html)**. Comprehensive guide to silo, pool, and bridge isolation patterns.
- **[Non-Interference](https://en.wikipedia.org/wiki/Non-interference_(security))**. The formal mathematical goal for secure multi-tenant isolation.

### 29. Disaster Recovery and Continuity
- **[Google SRE: Data Integrity](https://sre.google/sre-book/data-integrity/)**. Explains the formal relationship between backups and receipt reconciliation.
- **[AWS Builders' Library: Ensuring Rollback Safety](https://aws.amazon.com/builders-library/ensuring-rollback-safety-during-deployments/)**. Principles of state and code compatibility during recovery.
- **[FEMA: Continuity of Operations (COOP)](https://www.fema.gov/pdf/about/org/ncp/coop_brochure.pdf)**. The administrative foundation for RPO, RTO, and succession.

### 29.5 Extreme Fault Tolerance
- **[AWS Builders' Library: Static Stability](https://aws.amazon.com/builders-library/static-stability-using-availability-zones/)**. Definitive guide to systems that survive control-plane failures.
- **[Castro & Liskov: PBFT](https://pmg.csail.mit.edu/papers/osdi99.pdf)** (1999). Seminal research on Byzantine consensus and ledger-based consistency.
- **[IBM: MAPE-K Loop](https://en.wikipedia.org/wiki/Autonomic_computing)**. Foundational pattern for autonomous, self-healing software systems.

## What to Avoid

Avoid using blog posts, vendor claims, benchmark screenshots, or social media
threads as the source of truth for production design. They can be useful for
ideas, but production decisions should be backed by primary docs, standards,
tests, audits, and your own operational evidence.

The practical test is simple:

```text
If this source is wrong, what production decision becomes unsafe?
If the answer is serious, verify it against primary documentation or your own
controlled test before encoding it into the system.
```
