# Appendix A. Credible Resources and Further Reading

## How to Use This Appendix

A reliable agent is not built from one clever framework, one database trick, or
one model provider. It is built by connecting several mature engineering
disciplines: durable data systems, typed application code, production
operations, behavior evaluation, security, and governance.
Use this appendix as a guided reading map. Do not read everything at once. Pick
the source that strengthens the next weak part of your system. 

I call this following the **Theory of Constraints**. If your durable state is weak, no amount of AI papers will save you! Start where the pain is greatest.

```text
durable state weak -> read Postgres and data-systems material
... (omitted) ...
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

I also recommend turning any **AI Research Paper** you read into an **Evaluation Dataset**. If you read a new paper about "Reflexion," don't just admire it—turn its findings into a regression test for your own agent!

> ### 🎓 The Professor's Corner
>
> **The Librarian: Engineering as Information Management**
>
> Think of yourself as part-engineer and part-**Librarian**. You don't have to carry every fact in your head, but you *must* know where the answers are kept! 
> 
> A good librarian knows the difference between a "Gossip Magazine" (a social media thread) and a "Reference Book" (official documentation). One is for entertainment; the other is for building systems that last.

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

- [Andrew Ng: Agentic Workflows](https://www.deeplearning.ai/the-batch/how-agents-can-improve-llm-performance/)

  Read this for an industry guide to iterative, step-based agent orchestration
  and why reasoning loops improve performance over single-shot prompts.

- [Michael Wooldridge: Multiagent Systems](https://www.wiley.com/en-us/An+Introduction+to+Multiagent+Systems%2C+2nd+Edition-p-9780470519462)

  Read this definitive textbook on agent coordination, responsibility
  allocation, and the formal foundations of autonomous systems.

- [Wooldridge: Coordination Protocols](https://dl.acm.org/doi/10.5555/543666)

  Read this for academic formalisms for Joint Intentions and verifiable work
  transfer between autonomous actors.

- [Hancock et al.: Trust in Human-Robot Interaction](https://journals.sagepub.com/doi/10.1177/0018720811417254)

  Read this research defining the pillars of competence and integrity in
  delegation, applicable to human-agent trust boundaries.

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

- [AgentBench: A Comprehensive Benchmark for LLM-as-an-Agent](https://arxiv.org/abs/2308.03688)

  Read this for an academic rubric for evaluating multi-dimensional agent
  performance across diverse environments.

We should use benchmarks like **MMLU** or **GSM8K** only as a warning—general benchmarks are for *model* creators, while *agent* builders need **Task-Specific Evaluation** datasets that match their actual business goals.

- [Amodei et al.: Concrete Problems in AI Safety](https://arxiv.org/abs/1606.06565)

  Read this foundational research on safety challenges in autonomous systems,
  including scalable oversight and human-in-the-loop controls.

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

- [Brandur Leach: Transactionally Staged Job Drains](https://brandur.org/job-drains)

  Read this for the foundational engineering post explaining why writing a
  "durable fact" to Postgres is the only way to avoid losing work.

- [Chris Richardson: Transactional Outbox Pattern](https://microservices.io/patterns/data/transactional-outbox.html)

  Read this for the canonical reference for ensuring that state changes and
  side effects are decoupled yet consistent.

- [Pat Helland: Life Beyond Distributed Transactions](https://waitingforai.com/wp-content/uploads/2021/05/helland-life-beyond-distributed-transactions.pdf)

  Read this seminal paper explaining why scale and reliability require
  "entities" managed through messaging rather than distributed locks.

- [Jim Gray: Queues are Databases](https://arxiv.org/abs/cs/0701158)

  Read this classic academic argument for moving queuing logic into the
  database for transactional safety.

- [Craig Ringer: SELECT SKIP LOCKED](https://www.enterprisedb.com/blog/what-is-select-skip-locked-postgresql-9-5)

  Read this definitive technical explanation of high-concurrency task
  claiming in Postgres.

- [RudderStack: Scaling PostgreSQL Queues](https://www.rudderstack.com/blog/postgresql-as-our-main-streaming-engine-and-queuing-system/)

  Read this high-scale case study on using Postgres as an append-only ledger.

- [Gray & Cheriton: Leases (1989)](https://dl.acm.org/doi/10.1145/74850.74870)

  Read this for the academic origin of time-bound resource ownership (leases).

- [Chandra & Toueg: Unreliable Failure Detectors (1996)](https://dl.acm.org/doi/10.1145/224964.224992)

  Read this foundational research on heartbeat-based failure detection in
  distributed systems.

- [Airbnb: Avoiding Double Payments](https://medium.com/airbnb-engineering/avoiding-double-payments-in-a-distributed-system-29c35d03e536)

  Read this case study on using idempotency keys and leases to achieve high
  consistency in production.

- [Garcia-Molina & Salem: Sagas (1987)](https://www.cs.cornell.edu/andru/cs711/2002fa/reading/sagas.pdf)

  Read this academic foundation for managing long-lived transactions via
  compensating actions.

- [Hayashibara: Phi Accrual Failure Detector (2004)](https://dl.acm.org/doi/10.1145/1028174.1028177)

  Read this for advanced probabilistic failure detection algorithms.

- [Martin Fowler: Data Mapper](https://martinfowler.com/eaaCatalog/dataMapper.html)

  Read this for the foundational pattern for the row-to-domain conversion
  layer used in this book.

- [Eric Evans: Repository Pattern](https://www.domainlanguage.com/ddd/)

  Read this to understand the collection-like interface used to manage
  durable jobs in Domain-Driven Design.

- [Jay Kreps: The Log (2013)](https://engineering.linkedin.com/distributed-systems/log-what-every-software-engineer-should-know-about-real-time-datas-unifying)

  Read this for the unifying abstraction for scaling distributed data and
  event systems.

## Reliability and Operations

- [Google SRE books and resources](https://sre.google/)

  Read this for the original SRE book, the SRE Workbook, and Building Secure &
  Reliable Systems. These sources give the operational vocabulary behind SLIs,
  SLOs, error budgets, toil, incidents, and release discipline.

- [Google SRE chapter: Testing for Reliability](https://sre.google/sre-book/testing-reliability/)

  Read this to connect testing strategy to operational confidence rather than
  treating tests as a narrow checklist.

- [Google SRE: Service Level Objectives](https://sre.google/sre-book/service-level-objectives/)

  Read this for the foundational guide to measuring and managing service
  reliability via user-centric objectives.

- [Google SRE: Implementing SLOs](https://sre.google/workbook/implementing-slos/)

  Read this for the practical "recipe" for establishing reliability targets.

- [Google SRE: Burn Rate Alerting](https://sre.google/workbook/alerting-on-slos/)

  Read this for the industry-standard guide to monitoring error budget
  consumption via burn rates.

- [PlanetScale: The principles of extreme fault tolerance](https://planetscale.com/blog/the-principles-of-extreme-fault-tolerance)

  Read this for a concise production explanation of isolation, redundancy, and
  static stability.

- [Principles of Chaos Engineering](https://principlesofchaos.org/)

  Read this for the foundational methodology for resilience testing through
  fault injection.

- [OpenTelemetry documentation](https://opentelemetry.io/docs/)

  Read this for the standard for traces, metrics, and logs in distributed
  systems.

- [W3C Trace Context](https://www.w3.org/TR/trace-context/)

  Read this for the standard `traceparent` header format used to correlate
  logical operations.

- [AWS Builders' Library: Timeouts, Retries, and Backoff with Jitter](https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/)

  Read this for core distributed systems concepts for surviving ordinary
  failures.

- [Marc Brooker: Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)

  Read this for the primary industry reference for avoiding retry storms
  through randomness in backoff.

- [Metcalfe & Boggs: Ethernet Backoff (1976)](https://dl.acm.org/doi/10.1145/360248.360253)

  Read this for the academic origin of the binary exponential backoff
  algorithm.

- [Charity Majors: Observability vs Monitoring](https://www.honeycomb.io/blog/observability-vs-monitoring)

  Read this to understand the shift from static dashboards to event-driven
  debugging and exploring unknown-unknowns.

- [John Little: Queuing Formula (1961)](https://web.mit.edu/e-strategy/www/Little.pdf)

  Read this mathematical basis for modeling concurrency, throughput, and
  latency ($L = \lambda W$).

- [John Nagle: Congestion Control (RFC 896)](https://datatracker.ietf.org/doc/html/rfc896)

  Read this seminal work on preventing system collapse through flow control
  and backpressure.

- [AWS Builders' Library: Load Shedding](https://aws.amazon.com/builders-library/using-load-shedding-to-avoid-overload/)

  Read this for industry patterns for protecting systems from overload by
  refusing work.

- [Richard Cook: How Complex Systems Fail (1998)](https://how.complexsystems.fail/)

  Read this for foundational insights on the role of human operators in
  maintaining safety in hazardous systems.

- [PagerDuty: Incident Response](https://response.pagerduty.com/)

  Read this definitive guide to the industry-standard incident lifecycle and
  responder roles.

- [John Allspaw: Blameless Postmortems](https://codeascraft.com/2012/05/22/blameless-postmortems/)

  Read this for the philosophy of learning from failure by focusing on systemic
  causes rather than blame.

- [James Reason: The Swiss Cheese Model](https://en.wikipedia.org/wiki/Swiss_cheese_model)

  Read this core safety science concept for understanding how layered defenses
  and latent conditions align to cause accidents.

- [Erik Hollnagel: Safety-II](https://erikhollnagel.com/ideas/safety-i-and-safety-ii.html)

  Read this for the shift from failure-centric safety to success-oriented
  resilience engineering.

- [Google SRE: Release Engineering](https://sre.google/sre-book/release-engineering/)

  Read this for the principles of automated, high-velocity, and safe
  deployments.

- [GoCardless: Zero-Downtime Postgres Migrations](https://gocardless.com/blog/zero-downtime-postgres-migrations-the-hard-parts/)

  Read this for best practices for schema changes under load without service
  interruption.

- [Martin Fowler: Expand and Contract Pattern](https://martinfowler.com/bliki/ParallelChange.html)

  Read this for the core pattern for implementing backward-incompatible schema
  and code changes without downtime.

- [Netflix: Automated Canary Analysis (Kayenta)](https://netflixtechblog.com/automated-canary-analysis-at-netflix-with-kayenta-3260bc7acc69)

  Read this for advanced techniques for statistical release validation and
  canary promotion.

- [Google SRE: Eliminating Toil](https://sre.google/sre-book/eliminating-toil/)

  Read this for the industry-standard definition and management strategy for
  manual, repetitive operational work.

- [Lisanne Bainbridge: Ironies of Automation (1983)](https://en.wikipedia.org/wiki/Ironies_of_Automation)

  Read this seminal research on why automating "easy" tasks makes the
  remaining "hard" tasks more difficult for humans.

- [Sheridan & Verplank: Levels of Automation (1978)](https://apps.dtic.mil/sti/citations/ADA054659)

  Read this for the 10-level framework for allocating responsibility between
  humans and software.

- [Lehman: Laws of Software Evolution](https://en.wikipedia.org/wiki/Lehman%27s_laws_of_software_evolution)

  Read this foundational research on the inevitable decline in quality and
  increase in complexity of long-lived software.

- [Crunchy Data: Autovacuum Tuning](https://www.crunchydata.com/blog/postgres-autovacuum-tuning)

  Read this industry guide to managing database bloat and performance over
  long horizons.

- [AWS Builders' Library: Static Stability](https://aws.amazon.com/builders-library/static-stability-using-availability-zones/)

  Read this definitive guide to building systems that survive control-plane
  dependency failures.

- [Castro & Liskov: PBFT (1999)](https://pmg.csail.mit.edu/papers/osdi99.pdf)

  Read this seminal research on consensus in the presence of lying or
  malicious nodes.

- [IBM: MAPE-K Loop](https://en.wikipedia.org/wiki/Autonomic_computing)

  Read this foundational architectural pattern for building self-healing
  autonomous systems.

- [CMMI: Capability Maturity Model](https://cmmiinstitute.com/cmmi)

  Read this foundational framework for measuring and improving organizational
  process maturity.

- [Catchpoint: SRE Maturity Model](https://www.catchpoint.com/blog/sre-maturity-model)

  Read this practical industry rubric for evolving operational reliability
  practices.

## Evaluation and Behavior Reliability

- [OpenAI Evals](https://github.com/openai/evals)

  Read this for concrete patterns around creating and running evaluations for
  LLMs and LLM systems.

- [Zheng et al.: Judging LLM-as-a-Judge (2023)](https://arxiv.org/abs/2306.05685)

  Read this seminal paper formalizing the use of highly capable models as
  automated proxies for human preference.

- [Hamel Husain: Evaluation-Driven Development](https://hamel.dev/blog/posts/evals/)

  Read this practical guide to building domain-specific, binary evaluation
  pipelines for AI products.

- [Cohen's Kappa ($\kappa$)](https://en.wikipedia.org/wiki/Cohen%27s_kappa)

  Read this statistical standard for measuring inter-rater reliability and
  calibration in judge-based systems.

## Learning Design and Plain Language

- [CAST: Universal Design for Learning](https://www.cast.org/what-we-do/universal-design-for-learning/)

  Read this for the pedagogical standard for accessible, multi-modal
  technical documentation.

- [CDC: ADHD in the Classroom](https://www.cdc.gov/adhd/treatment/classroom.html)

  Read this for teaching supports around clear expectations and organization.

- [John Sweller: Cognitive Load Theory and CS Education (2016)](https://dl.acm.org/doi/10.1145/2839509.2844652)

  Read this for the theoretical backing for the "Worked Example Effect" used
  to accelerate technical understanding.

- [Leslie Lamport: The Temporal Logic of Actions (1994)](https://lamport.azurewebsites.net/tla/tla.html)

  Read this formal source for the state-actor-transition-invariant modeling
  grammar used in this book.

- [Hillel Wayne: Learn TLA+](https://learntla.com/)

  Read this for practical applications of formal modeling to distributed
  system design and verification.

- [Skelton & Pais: Team Topologies (2019)](https://teamtopologies.com/)

  Read this to understand how software boundaries should be designed to fit
  within human working-memory limits.

- [Michael Nygard: Architecture Decision Records (2011)](https://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions)

  Read this standard for recording architectural constraints, context, and the
  "Operating Envelope."

- [Digital.gov: Short and Simple](https://digital.gov/guides/plain-language/principles/short-simple)

  Read this for concrete plain-language rules: know the audience, organize for
  the reader, and write short sentences.

- [Paas and van Merrienboer: Cognitive-Load Theory](https://journals.sagepub.com/doi/10.1177/0963721420922183)

  Read this for the research reason behind worked examples and reducing load.

- [IES: Organizing Instruction and Study to Improve Student Learning](https://ies.ed.gov/ncee/wwc/PracticeGuide/1)

  Read this for practice-guide support around spacing and active retrieval.

- [Nature Reviews Psychology: Spacing and Retrieval Practice](https://www.nature.com/articles/s44159-022-00089-1)

  Read this for a research review of spacing and retrieval practice.

## Security, Abuse, and Governance

- [NIST AI Risk Management Framework 1.0](https://www.nist.gov/publications/artificial-intelligence-risk-management-framework-ai-rmf-10)

  Read this for a vendor-neutral risk framework around building trustworthy
  AI systems.

- [NIST: AI Risk Management Framework](https://www.nist.gov/itl/ai-rmf)

  Read this detailed guidance on human oversight, governance, and
  sociotechnical AI safety.

- [A-MemGuard: Agent Memory Defense (2025)](https://mem0.ai/)

  Read this emerging research on detecting and mitigating persistent memory
  poisoning and injection.

- [MINJA: Memory Injection Attacks (2024)](https://arxiv.org/abs/2406.01258)

  Read this foundational research on the high success rate of adversarial
  persistent memory exploits.

- [Morris et al.: Text Embeddings Reveal Text (2024)](https://arxiv.org/abs/2310.06816)

  Read this research proving that 50-70% of original words can be recovered
  from raw vector embeddings.

- [Myers & Liskov: Decentralized IFC (1997)](https://dl.acm.org/doi/10.1145/266635.266669)

  Read this for the academic origin of information-flow labels and trust
  boundaries in secure systems.

- [OWASP: Top 10 for LLM Applications](https://owasp.org/www-project-top-10-for-large-language-model-applications/)

  Read this industry standard for modern agent vulnerabilities: injection,
  leakage, and insecure output handling.

- [OWASP: LLM Top 10 (2025)](https://genai.ovasp.org/)

  Read this for the latest vulnerability list including vector weaknesses.

- [Ann Cavoukian: Privacy by Design](https://www.ipc.on.ca/wp-content/uploads/2013/09/pbd-primer.pdf)

  Read this for the 7 foundational principles for embedding privacy into the
  architecture of automated systems.

- [EDPB: Data Protection by Design Guidelines](https://www.edpb.europa.eu/our-work-tools/our-documents/guidelines/guidelines-42019-article-25-data-protection-design-and_en)

  Read this regulatory standard for implementing automated data retention and
  erasure workflows.

- [AWS: SaaS Tenant Isolation Strategies](https://docs.aws.amazon.com/whitepapers/latest/saas-tenant-isolation-strategies/welcome.html)

  Read this comprehensive guide to silo, pool, and bridge isolation patterns
  for multi-tenant applications.

- [Non-Interference (Security)](https://en.wikipedia.org/wiki/Non-interference_(security))

  Read this formal mathematical goal for secure multi-tenant isolation and
  confidentiality.

- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)

  Read this when designing credential storage, rotation, and auditability.

- [NIST SP 800-57 Part 1 Rev. 5: Recommendation for Key Management](https://csrc.nist.gov/pubs/sp/800/57/pt1/r5/final)

  Read this for key-management vocabulary and policy.

- [CNCF TAG Security: Cloud Native Security Whitepaper v2](https://tag-security.cncf.io/community/resources/security-whitepaper/v2/cloud-native-security-whitepaper/)

  Read this for production guidance on workload identity and rotation.

- [European Data Protection Board: Respect Individuals' Rights](https://www.edpb.europa.eu/sme-data-protection-guide/respect-individuals-rights_en)

  Read this when data-protection requests need to become operational workflow.

- [ICO: Storage Limitation](https://ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/data-protection-principles/a-guide-to-the-data-protection-principles/storage-limitation/)

  Read this for retention-policy judgment and erasure workflows.

- [MITRE ATLAS](https://atlas.mitre.org/)

  Read this to build a threat-informed view of attacks against AI systems.

- [NIST AI RMF Generative AI Profile](https://doi.org/10.6028/NIST.AI.600-1)

  Read this when your agent can generate user-visible content.

## Rust Engineering

- [The Rust Book: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

  Read this for the difference between Result and panic.

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

  Read this when shaping public types and module boundaries.

- [thiserror documentation](https://docs.rs/thiserror)

  Read this for structured, typed error definitions.

- [Tokio: Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown)

  Read this definitive practical guide to cooperative cancellation and RAII-
  based system shutdown.

- [RustSec Advisory Database](https://rustsec.org/)

  Read this for cargo audit and security advisories.

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

## What to Avoid

Avoid using blog posts, vendor claims, benchmark screenshots, or social media
threads as the source of truth for production design. I call these **"Technical Junk Food."** They are tasty and easy to swallow, but they don't help you grow strong systems!

> ### 🎓 The Professor's Corner
>
> **Primary Sources: Talking to the Architect**
>
> Reading the official Postgres or Rust documentation is like "Talking to the Architect" who built the house! 
> 
> Reading a blog post is like "Talking to a neighbor" who saw the building once. The neighbor might give you a good tip, but if you want to know if a wall is safe to move, you always ask the Architect. Official docs are "Brain Food"—they're harder to chew, but they make you much smarter!

They can be useful for
ideas, but production decisions should be backed by primary docs, standards,
tests, audits, and your own operational evidence.

The practical test is simple:

```text
If this source is wrong, what production decision becomes unsafe?
If the answer is serious, verify it against primary documentation or your own
controlled test before encoding it into the system.
```

## Chapter-Specific Resources

### Part I. The Core System
- **00. How to Read This Book:** [John Sweller: Cognitive Load Theory](https://dl.acm.org/doi/10.1145/2839509.2844652) (2016), [CAST: UDL Guidelines](https://udlguidelines.cast.org/).
  Read this because: Standardizes technical learning via worked examples and engagement.
- **00b. System Model and Notation:** [Leslie Lamport: TLA](https://lamport.azurewebsites.net/tla/tla.html) (1994), [Hillel Wayne: Learn TLA+](https://learntla.com/).
  Read this because: Provides the formal source for state-actor-transition-invariant modeling.
- **00c. Design Principles:** [Skelton & Pais: Team Topologies](https://teamtopologies.com/) (2019), [Google SRE Book: The 5 Pillars of Quality](https://sre.google/sre-book/table-of-contents/).
  Read this because: Connects system boundaries to cognitive load and operational quality.
- **00d. Production Scope and Trade-Offs:** [Michael Nygard: Architecture Decision Records (ADR)](https://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions) (2011), [Temporal: Why Workflow Engines?](https://temporal.io/blog/why-is-workflow-orchestration-hard).
  Read this because: Standardizes recording architectural constraints and trade-offs.
- **01. The Problem:** [Anthropic: Building Effective Agents](https://www.anthropic.com/research/building-effective-agents), [Brandur Leach: Postgres Job Drains](https://brandur.org/job-drains).
  Read this because: Distinguishes agents from workflows and defines durable facts in Postgres.
- **02. The Mental Model:** [Gul Agha: Actors—A Model of Concurrent Computation](https://mitpress.mit.edu/9780262010924/actors/) (1986), [Stripe: Scaling Idempotency](https://stripe.com/blog/idempotency).
  Read this because: Formalizes decoupled actors and intent-based scaling.
- **02.5 Guarantees and Failure Semantics:** [Chris Richardson: Transactional Outbox](https://microservices.io/patterns/data/transactional-outbox.html), [Pat Helland: Life Beyond Distributed Transactions](https://waitingforai.com/wp-content/uploads/2021/05/helland-life-beyond-distributed-transactions.pdf).
  Read this because: Explains how to reliably trigger side effects without distributed locks.
- **03. The Postgres Ledger:** [Jim Gray: Queues are Databases](https://arxiv.org/abs/cs/0701158) (1995), [Craig Ringer: SELECT SKIP LOCKED](https://www.enterprisedb.com/blog/what-is-select-skip-locked-postgresql-9-5).
  Read this because: The classic academic and engineering foundation for database-backed queuing.
- **04. The Rust Domain Model:** [Alexis King: Parse, don't validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/) (2019), [Scott Wlaschin: Domain Modeling Made Functional](https://fsharpforfunandprofit.com/books/).
  Read this because: The primary text for using types to make illegal states unrepresentable.
- **04.5 Typed Composition Lens:** [The Embedded Rust Book: Typestate Programming](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html), [Kohei Honda: Session Types](https://dl.acm.org/doi/10.1145/2103736.2103744) (2012).
  Read this because: Enforces lifecycle states and transitions at compile time.
- **05. The Worker Loop:** [Gray & Cheriton: Leases](https://dl.acm.org/doi/10.1145/74850.74870) (1989), [Chandra & Toueg: Unreliable Failure Detectors](https://dl.acm.org/doi/10.1145/224964.224992) (1996).
  Read this because: Foundational research on resource ownership and failure detection.
- **06. The Rig Boundary:** [Toolformer: LM Tool Use](https://arxiv.org/abs/2302.04761) (2023), [ReAct: Reasoning and Acting](https://arxiv.org/abs/2210.03629) (2022).
  Read this because: Academic basis for model-powered API interaction and reasoning loops.
- **07. Running Locally:** [FoundationDB: Deterministic Simulation Testing](https://www.youtube.com/watch?v=4fFDFbi3toc), [Will Wilson: Testing the Untestable](https://www.youtube.com/watch?v=fFSPwJFXVlU).
  Read this because: The gold standard for proving distributed systems via local simulation.
- **08. Production Hardening:** [Brandur Leach: Idempotency Keys](https://brandur.org/idempotency-keys), [AWS Builders' Library: Avoiding Overload](https://aws.amazon.com/builders-library/avoiding-overload-in-distributed-systems-by-putting-the-smaller-service-in-control/).
  Read this because: Core industry reference for protecting API and worker boundaries.
- **09. Failure Modes:** [Eric Brewer: CAP Twelve Years Later](https://www.infoq.com/articles/cap-twelve-years-later-how-the-rules-have-changed/), [Abadi: PACELC Theorem](https://en.wikipedia.org/wiki/PACELC_theorem).
  Read this because: Contextualizes failures within distributed systems theory.
- **10. Capstone:** [Brandur Leach: The Serious MVP](https://brandur.org/serious-mvp).
  Read this because: Manifesto for building database-first reliability shells before feature complexity.

### Part II. Production Engineering
- **11. The Real Postgres Store:** [Martin Fowler: Data Mapper](https://martinfowler.com/eaaCatalog/dataMapper.html), [Repository Pattern](https://www.domainlanguage.com/ddd/).
  Read this because: Formalizes the conversion layer and job collection interface.
- **12. Idempotency and Side Effects:** [Airbnb Engineering: Avoiding Double Payments](https://medium.com/airbnb-engineering/avoiding-double-payments-in-a-distributed-system-29c35d03e536), [Garcia-Molina & Salem: Sagas](https://www.cs.cornell.edu/andru/cs711/2002fa/reading/sagas.pdf).
  Read this because: Industry case studies on high-consistency distributed payments and transactions.
- **13. Leases, Heartbeats, and Cancellation:** [Hayashibara: Phi Accrual Failure Detector](https://dl.acm.org/doi/10.1145/1028174.1028177) (2004), [Tokio: Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown).
  Read this because: Advanced research on network-adaptive liveness detection.
- **14. Retry, Backoff, and Dead Letters:** [Metcalfe & Boggs: Ethernet](https://dl.acm.org/doi/10.1145/360248.360253) (1976), [Marc Brooker: Jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/).
  Read this because: The origin of backoff algorithms and industry guides on synchronization avoidance.
- **15. Observability and SLOs:** [Google: Dapper Paper](https://research.google/pubs/dapper-a-large-scale-distributed-systems-tracing-infrastructure/), [W3C: Trace Context Standard](https://www.w3.org/TR/trace-context/).
  Read this because: Foundational standard for distributed tracing and correlation.
- **16. Human Approval and Policy Gates:** [Amodei et al.: Concrete Problems in AI Safety](https://arxiv.org/abs/1606.06565), [NIST: AI Risk Management Framework](https://www.nist.gov/itl/ai-rmf).
  Read this because: Scholarly research on human-in-the-loop systems and trustworthy AI.
- **17. Testing Production Agents:** [Principles of Chaos Engineering](https://principlesofchaos.org/), [AgentBench (ICLR 2024)](https://arxiv.org/abs/2308.03688).
  Read this because: Industry-standard methodology for resilience testing and agent benchmarking.
- **18. Deployment and Operations:** [Google SRE: Release Engineering](https://sre.google/sre-book/release-engineering/), [GoCardless: Zero-Downtime Postgres Migrations](https://gocardless.com/blog/zero-downtime-postgres-migrations-the-hard-parts/).
  Read this because: Principles of safe, automated rollout and backward compatibility.
- **19. Running for Years:** [Lehman: Laws of Software Evolution](https://en.wikipedia.org/wiki/Lehman%27s_laws_of_software_evolution), [Crunchy Data: Autovacuum Tuning](https://www.crunchydata.com/blog/postgres-autovacuum-tuning).
  Read this because: Foundational research on the inevitable forces governing long-lived systems.
- **20. Final Production Blueprint:** [Michael Wooldridge: Multiagent Systems](https://www.wiley.com/en-us/An+Introduction+to+Multiagent+Systems%2C+2nd+Edition-p-9780470519462), [Andrew Ng: Agentic Workflows](https://www.deeplearning.ai/the-batch/how-agents-can-improve-llm-performance/).
  Read this because: Textbook coordination and iterative agent orchestration guides.
- **20.1 Agent Handoffs and Coordination:** [Wooldridge: Coordination Protocols](https://dl.acm.org/doi/10.5555/543666), [Hancock et al.: Trust in HRI](https://journals.sagepub.com/doi/10.1177/0018720811417254).
  Read this because: Verifiable work transfer and trust pillars in agent delegation.
- **20.2 Worked Production Scenario:** [Pat Helland: Memories, Guesses, and Apologies](https://arxiv.org/abs/2005.02103) (2020).
  Read this because: Architectural logic for uncertainty and compensation in distributed runs.

### Part III. Operating the System
- **21. SLIs, SLOs, and Error Budgets:** [Google SRE: Service Level Objectives](https://sre.google/sre-book/service-level-objectives/), [The SRE Workbook: Implementing SLOs](https://sre.google/workbook/implementing-slos/).
  Read this because: The primary source for defining user-centric reliability targets.
- **22. Capacity, Backpressure, and Provider Quotas:** [John Little: Little's Law](https://web.mit.edu/e-strategy/www/Little.pdf), [John Nagle: Congestion Control](https://datatracker.ietf.org/doc/html/rfc896).
  Read this because: Mathematical basis for concurrency and preventing congestive system collapse.
- **23. Runbooks:** [Richard Cook: How Complex Systems Fail](https://how.complexsystems.fail/), [PagerDuty: Incident Response](https://response.pagerduty.com/).
  Read this because: Critical insights on human operators and incident lifecycles.
- **24. Incident Response and Postmortems:** [John Allspaw: Blameless Postmortems](https://codeascraft.com/2012/05/22/blameless-postmortems/), [James Reason: Swiss Cheese Model](https://en.wikipedia.org/wiki/Swiss_cheese_model).
  Read this because: The industry-standard philosophy for learning from failure without blame.
- **25. Release Engineering:** [Walter F. Tichy: Tools for SCM](https://dl.acm.org/doi/10.5555/54366.54370) (1988), [Netflix: Automated Canary Analysis (Kayenta)](https://netflixtechblog.com/automated-canary-analysis-at-netflix-with-kayenta-3260bc7acc69).
  Read this because: Foundational taxonomy for version control and statistical rollout validation.
- **26. Toil, Automation, and Ownership:** [Google SRE: Eliminating Toil](https://sre.google/sre-book/eliminating-toil/), [Lisanne Bainbridge: Ironies of Automation](https://en.wikipedia.org/wiki/Ironies_of_Automation) (1983).
  Read this because: Defines sustainable operation and pitfalls of over-automation.

### Part IV. World-Class Reliability
- **27. Evaluation and Behavior Reliability:** [Zheng et al.: Judging LLM-as-a-Judge](https://arxiv.org/abs/2306.05685), [Hamel Husain: Evaluation-Driven Development](https://hamel.dev/blog/posts/evals/).
  Read this because: Practical guides to automated judge-based evaluation and binary pipeline grading.
- **27.5 Agent Memory, Retrieval, and Retention:** [MINJA: Memory Injection Attacks](https://arxiv.org/abs/2406.01258), [Morris et al.: Text Embeddings Reveal Text](https://arxiv.org/abs/2310.06816).
  Read this because: Explains persistent memory poisoning and privacy risks in vectors.
- **28. Security, Abuse, and Trust Boundaries:** [Myers & Liskov: Decentralized IFC](https://dl.acm.org/doi/10.1145/266635.266669), [OWASP: Top 10 for LLM Applications](https://genai.ovasp.org/).
  Read this because: Academic origin of labels and industry standard for agent vulnerabilities.
- **28.5 Data Protection and Privacy Operations:** [Ann Cavoukian: Privacy by Design](https://www.ipc.on.ca/wp-content/uploads/2013/09/pbd-primer.pdf), [EDPB: Data Protection by Design Guidelines](https://www.edpb.europa.eu/our-work-tools/our-documents/guidelines/guidelines-42019-article-25-data-protection-design-and_en).
  Read this because: Foundational framework for embedding privacy into system architecture.
- **28.6 Tenant Isolation and Multi-tenancy:** [AWS: SaaS Tenant Isolation Strategies](https://docs.aws.amazon.com/whitepapers/latest/saas-tenant-isolation-strategies/welcome.html), [Non-Interference](https://en.wikipedia.org/wiki/Non-interference_(security)).
  Read this because: Comprehensive guide to isolation patterns and formal security goals.
- **29. Disaster Recovery and Continuity:** [Google SRE: Data Integrity](https://sre.google/sre-book/data-integrity/), [FEMA: Continuity of Operations (COOP)](https://www.fema.gov/pdf/about/org/ncp/coop_brochure.pdf).
  Read this because: Formal relationship between state backups and organizational succession.
- **29.5 Extreme Fault Tolerance:** [AWS Builders' Library: Static Stability](https://aws.amazon.com/builders-library/static-stability-using-availability-zones/), [Castro & Liskov: PBFT](https://pmg.csail.mit.edu/papers/osdi99.pdf) (1999).
  Read this because: Definitive guide to surviving dependency failure and ledger consistency.
- **30. Reliability Maturity Model:** [CMMI: Capability Maturity Model](https://cmmiinstitute.com/cmmi), [Catchpoint: SRE Maturity Model](https://www.catchpoint.com/blog/sre-maturity-model).
  Read this because: Framework for measuring and improving organizational process and reliability.
- **30b. Scaling Paths After Postgres-First:** [Jay Kreps: The Log](https://engineering.linkedin.com/distributed-systems/log-what-every-software-engineer-should-know-about-real-time-datas-unifying).
  Read this because: The unifying abstraction for scaling distributed data.
- **30c. Temporal After Postgres-First:** [Temporal: What is Durable Execution?](https://temporal.io/blog/what-is-durable-execution), [Chris Gavin: Stop Building State Machines in Your Database](https://chrisgavin.dev/posts/temporal-state-machine/).
  Read this because: Criteria for selecting durable workflow orchestration.
- **30d. Kafka After Postgres-First:** [Apache Kafka: Design](https://kafka.apache.org/documentation/#design), [Confluent: Event Streaming for the Business](https://www.confluent.io/blog/event-streaming-for-the-business/).
  Read this because: Log-structured storage principles and event streaming contracts.
