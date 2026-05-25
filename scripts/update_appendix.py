import os
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "books/postgres-rig-agent-jobs/src/31-credible-resources-further-reading.md"
with open(PATH, 'r') as f:
    content = f.read()

# Remove the existing suggested reading paths and what to avoid to insert our new stuff
parts = re.split(r"(## Suggested Reading Paths)", content)
header = parts[0]
footer = "## Suggested Reading Paths" + parts[2]

new_stuff = """
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
- **20. Final Production Blueprint:** [Michael Wooldridge: Multiagent Systems](https://www.wiley.com/en-us/An+Introduction+to+Multiagent+Systems%2C+2nd+Edition-p-9780470519462), [DeepLearning.AI: Agentic Workflows](https://www.deeplearning.ai/the-batch/how-agents-can-improve-llm-performance/).
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

"""

final_content = header + new_stuff + footer
with open(PATH, 'w') as f:
    f.write(final_content)
