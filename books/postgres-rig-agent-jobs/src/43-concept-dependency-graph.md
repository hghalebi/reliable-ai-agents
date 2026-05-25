# Appendix M. Concept Dependency Graph

## How to Use This Graph

This appendix is the book's concept-by-concept dependency graph. Use it when a
chapter feels locally clear but the whole system still feels large.

The graph has one rule:

```text
prerequisite -> new concept -> mechanism -> production capability
```

Do not read a later concept as an isolated technique. SLOs depend on durable
state. Retry depends on idempotency. Approval depends on a proposal state.
Disaster recovery depends on versioned, replay-safe state. The book is serious
only if the reader can explain those dependencies, not merely list the tools.

## Reading The Dependency Shape

Each row answers four questions:

```text
What must already be true?
What does this chapter add?
Which mechanism makes it real?
What can the system now do safely?
```

If you cannot answer the first question, go backward. If you cannot answer the
last question, the concept has not yet become production judgment.

## Front Matter Dependencies

| Chapter | Prerequisite | New concept | Mechanism | Production capability unlocked |
| --- | --- | --- | --- | --- |
| System Model And Notation | A reader can distinguish a script from a system. | State, actor, transition, evidence, and invariant form one reasoning unit. | Shared notation for jobs, leases, events, policy decisions, evaluation receipts, and side-effect receipts. | Every later chapter can ask what changed, who changed it, what evidence remains, and what invariant survived. |
| Design Principles | The system model exists. | Ten transferable production rules compress the book's mechanisms. | Durable-before-intelligent, typed-before-clever, ownership-before-concurrency, and the remaining principles. | The reader can evaluate designs outside this exact Rust/Rig/Postgres implementation. |
| Production Scope And Trade-Offs | The reader can reason from state, actor, transition, evidence, and invariant. | Architecture choice is an operating-envelope decision. | Compare script, queue framework, Postgres-first ledger, durable workflow engine, and distributed platform by the evidence each one owns. | The reader can choose or reject the book's architecture for explicit production reasons. |

## Part I Dependencies

| Chapter | Prerequisite | New concept | Mechanism | Production capability unlocked |
| --- | --- | --- | --- | --- |
| 1. The Problem | The reader knows that calling a model can produce useful text. | A reliable agent job is durable work with one probabilistic step. | Create a durable job row before model execution. | Work survives crashes, retries, and later audits. |
| 2. The Mental Model | A durable job exists. | State, worker, model, events, and policy are separate boundaries. | The worker loop keeps the model inside the workflow, not around it. | Each component has one responsibility and one failure surface. |
| 2.5 Guarantees And Failure Semantics | Boundaries are separate. | Reliability means explicit guarantees and explicit non-guarantees. | Write failure semantics for duplicate intake, retry, crash, cancellation, and side effects. | Engineers stop depending on promises the system does not make. |
| 3. The Postgres Ledger | The system needs durable state. | Postgres can coordinate work, not only store results. | Tables, constraints, row locks, leases, idempotency keys, and event rows. | Multiple workers can cooperate through durable state instead of memory. |
| 4. The Rust Domain Model | The ledger has meaningful concepts. | Domain values deserve names, constructors, and enums. | Newtypes and semantic enums for job kind, state, worker, instruction, retry, and versions. | Invalid states become harder to express at architecture boundaries. |
| 4.5 Typed Composition Lens | Domain concepts are named. | Type-state and composition can encode lifecycle constraints. | Builders expose only valid next steps. | Some invalid workflows fail at compile time instead of at 03:00. |
| 5. The Worker Loop | Jobs can be stored and typed. | Execution is a sequence of owned transitions. | Pick, lease, start event, agent call, classification, state update, event. | One job can move safely from pending to terminal or scheduled retry. |
| 6. The Rig Boundary | The worker owns execution. | Provider behavior enters through a boundary. | AgentRunner and Rig-backed runner convert provider output and errors into domain outcomes. | Provider changes do not infect the core state machine. |
| 7. Running The System Locally | The core loop exists. | Local execution can prove the state machine without external services. | Deterministic local runner and default tests. | The reader can validate core behavior without network, Postgres, or API keys. |
| 8. Production Hardening | The demo loop works. | Production controls close places where failure can hide. | Idempotency, leases, approval gates, observability, retries, and credential lifecycle discipline. | The system can tolerate duplicate requests, crashes, risky action, and operator investigation. |
| 9. Failure Modes | Hardening controls have names. | Failure classes reveal missing invariants. | Lost ownership, identity, sequence, authority, and terminality lens. | Engineers can choose the right architectural fix instead of adding local conditionals. |
| 10. Capstone | The core invariants are visible. | New features must preserve old invariants. | Add commands, states, tables, events, tests, and runbooks together. | The system can grow without becoming a disconnected collection of scripts. |

## Part II Dependencies

| Chapter | Prerequisite | New concept | Mechanism | Production capability unlocked |
| --- | --- | --- | --- | --- |
| 11. The Real Postgres Store | The in-memory model and SQL shape are understood. | Persistence is a boundary conversion problem. | Rows convert through validated domain types. | Bad persisted data stops at the boundary instead of corrupting worker logic. |
| 12. Idempotency And Side Effects | Duplicate work is understood as a failure mode. | One logical request must map to one durable action path. | Idempotency keys, duplicate suppression, side-effect receipts. | Retry and replay can happen without multiplying external actions. |
| 13. Leases, Heartbeats, And Cancellation | Workers need explicit ownership. | A lease is temporary authority, a deadline is a time promise, and cancellation is durable stop intent plus an observed outcome. | `locked_by`, `locked_until`, heartbeat, timeout policy, breached-deadline query, recovery, cancellation request lifecycle, and cancellation queries. | Crashed, slow, overdue, and intentionally stopped work remains recoverable and inspectable. |
| 14. Retry, Backoff, And Dead Letters | Failure outcomes are typed. | Retry is scheduling; dead-lettering is visible terminal state. | RetryDisposition, persisted attempts, backoff, dead reason, retry query. | Temporary failures become future work, while permanent failures stop safely. |
| 15. Observability And SLOs | State transitions and events exist. | Operators need close, middle, and far evidence. | Job rows, event timelines, audit events, operation events, `/healthz`, `/readyz`, `/metrics`, traces, SLI queries, SLOs, and alerts. | Operators can move from process health to fleet symptoms to one job's decision and runtime evidence chain. |
| 16. Human Approval And Policy Gates | Side effects are separated from reasoning. | Approval is durable authority, not a prompt instruction. | Proposal state, policy version, operator decision, execution check, receipt. | Risky actions can be reviewed, rejected, approved, and replayed without bypassing policy. |
| 17. Testing Production Agents | Deterministic boundaries exist. | Tests should follow the system rings. | Domain tests, store tests, worker transition tests, SQL checks, provider smoke tests, evals. | Reliability can be proved without treating live model behavior as the only oracle. |
| 18. Deployment And Operations | The service has durable work in flight. | Deploys must preserve API admission, state, leases, credential lifecycle, and shutdown behavior. | Separate API and worker binaries, graceful shutdown, migration discipline, config checks, runbook commands. | Code can change while old work remains safe and explainable. |
| 19. Running For Years | Releases and operations are routine. | Long-running systems are compatibility systems. | Versioned rows, retention rules, provider boundary, ownership records. | Old jobs remain parseable, auditable, and replay-aware after the system changes. |
| 20. Final Production Blueprint | The production controls are known. | The architecture is a map of failure ownership. | API admission boundary, boundary ownership table, durable handoff model, and production checklist. | A reader can explain which component or target agent owns each class of failure. |
| 20.1 Agent Handoffs And Multi-Agent Coordination | The blueprint names handoffs as a boundary. | Multi-agent coordination is durable responsibility transfer. | Handoff typestate, row-to-domain validation, idempotency key, target job evidence, and pending-handoffs query. | Specialist agents can cooperate without losing ownership or creating hidden work. |
| 20.2 Worked Production Scenario | The blueprint exists. | Controls cooperate on one risky request. | Duplicate intake, retry, handoff, approval, side-effect receipt, operator review. | The reader can trace one job through the complete evidence chain. |

## Part III Dependencies

| Chapter | Prerequisite | New concept | Mechanism | Production capability unlocked |
| --- | --- | --- | --- | --- |
| 21. SLIs, SLOs, And Error Budgets | Metrics and events are trustworthy. | Reliability becomes an explicit measurement contract. | SLI sources, typed measurement rows, SLO windows, error budgets, burn alerts, ownership. | Teams can decide when to slow releases or invest in reliability from validated evidence. |
| 22. Capacity, Backpressure, And Provider Quotas | Queue health is measurable. | Overload is a policy and fairness problem, not only a scaling problem. | Admission control, worker concurrency, per-kind fairness, provider quota handling. | The system can degrade intentionally instead of collapsing under demand. |
| 23. Runbooks For Agent Job Systems | Operators have durable evidence. | Runbooks turn system knowledge into repeatable human operations. | Queue, lease, deadline, cancellation, dead-letter, audit, operation, handoff, pause, replay, and timeline queries. | On-call engineers can investigate without guessing or reading code first. |
| 24. Incident Response And Postmortems | SLOs, runbooks, and evidence exist. | An incident is a failed invariant with impact. | Detection, triage, mitigation, communication, recovery, postmortem, action items. | Incidents become system improvements, not only status updates. |
| 25. Release Engineering For Agents | Incidents and deployments are understood. | Code, schema, prompt, model, tool, and policy releases have different risks. | Version skew handling, canaries, migrations, receipts, rollback paths. | Teams can release behavior changes without losing old-work safety. |
| 26. Toil, Automation, And Ownership | Runbooks and incidents reveal repeated work. | Toil indicates missing control surfaces or unclear ownership. | Toil metrics, automation candidates, owner roles, handoff rules. | Human attention shifts from repetitive recovery to judgment and product decisions. |

## Part IV Dependencies

| Chapter | Prerequisite | New concept | Mechanism | Production capability unlocked |
| --- | --- | --- | --- | --- |
| 27. Evaluation And Behavior Reliability | Prompt, model, policy, and tool versions are recorded. | Availability does not prove behavior quality. | Evaluation fixtures, behavior dimensions, evaluation receipts, promotion gates. | Model or prompt changes can be promoted with evidence instead of vibes. |
| 27.5 Agent Memory, Retrieval, And Retention | Behavior evidence and typed boundaries exist. | Memory is typed production data, not prompt decoration. | Memory horizon, retention policy, source, scope, confidence, embedding reference, row conversion, redaction, and metadata runbook query. | Agents can use remembered context without letting stale, poisoned, or cross-tenant text become hidden authority. |
| 28. Security, Abuse, And Trust Boundaries | Tools, policy, memory, sandboxing, and users are separate boundaries. | Instructions have authority only inside a trust boundary. | Threat models, tool contracts, memory controls, authorization, sandbox events, audit events. | Prompt injection, tool abuse, SSRF, and data leakage become design concerns, not surprises. |
| 28.5 Data Protection, Retention, And Privacy Operations | Evidence surfaces and trust boundaries are visible. | Privacy requests are operational workflow. | Data-protection request ledger, review query, typed row boundary, policy version, completion evidence, audit event, and operation event. | Redaction, erasure, export, and retention-review requests can be operated without informal notes or destructive guessing. |
| 28.6 Tenant Isolation And Multi-Tenant Agents | Security authorization is outside the model. | Tenant scope is an authorization boundary. | Actor tenant, requested tenant, permission, policy version, authorization event, tenant-boundary review query, typed row boundary, trace id, audit event. | Multi-tenant agents can deny and review cross-tenant attempts before tool execution. |
| 29. Disaster Recovery And Continuity | Versioning, receipts, and runbooks exist. | Backup is not recovery; replay must be safe. | RPO, RTO, state inventory, restore drill, replay rules, side-effect receipt handling. | The system can restart after serious loss without duplicating external effects. |
| 29.5 Extreme Fault Tolerance For Agent Systems | Recovery and release evidence exist. | Critical execution should be isolated from non-critical control-plane failures. | `fault_tolerance_reviews`, last-known-good versions, worker redundancy, static-stability mode, failover drill, release gate, and typed readiness decoding. | The system can keep already-approved work serving, degrade to draft-only mode, or pause deliberately when control surfaces fail. |
| 30. Reliability Maturity Model | All controls are visible. | Maturity is evidence-backed and job-kind specific. | Maturity levels, readiness scorecard, gaps, owners, review dates. | Teams can choose the next reliability upgrade instead of claiming vague robustness. |
| 30.5 Scaling Paths After Postgres-First | Maturity gaps and operating limits are visible. | Scaling is evidence-preserving responsibility migration. | Baseline metrics, invariant-to-component map, coexistence plan, runbook updates, trace correlation, operation events, and rollback criteria. | Teams can evolve to queues, workflow engines, worker pools, collectors, or consoles without losing the reliability model. |
| 30.6 Temporal After Postgres-First | The scaling map names workflow engines as optional. | Temporal is an execution ledger, not automatic product truth. | Workflow id mapping, activity receipts, approval signals, Postgres reconciliation, trace correlation, and rollback criteria. | Teams can adopt durable workflow execution while keeping product evidence typed, audited, and queryable. |
| 30.7 Kafka After Postgres-First | The scaling map names event streams as optional. | Kafka is event distribution, not automatic audit truth. | Outbox bridge, typed event schema, partition-key policy, topic-partition-offset evidence, consumer receipts, and replay drills. | Teams can add fanout and replay without creating duplicate side effects or a second ungoverned source of truth. |

## Cross-Cutting Dependency Checks

Use these checks during review:

```text
Durability check:
  Does the work exist before the model call?

Type check:
  Does the boundary name the domain concept?

Ownership check:
  Can only the current owner move the job?

Idempotency check:
  Can duplicate intent become duplicate side effect?

Evidence check:
  Can an operator reconstruct the timeline without process memory?

Policy check:
  Can risky action happen without deterministic authority?

Version check:
  Can old work be interpreted after release, prompt, model, or schema changes?

Recovery check:
  Can restore and replay happen without guessing about side effects?
```

These checks are intentionally repetitive. Repetition is how the book turns a
large system into a small set of engineering moves.

## Summary

The dependency graph is the spine of the book:

```text
durable work
  -> explicit boundaries
  -> owned execution
  -> provider isolation
  -> idempotent retry
  -> observable operation
  -> versioned change
  -> evaluated, secure, recoverable maturity
```

When a design feels confusing, find the first missing dependency. The right fix
usually belongs there, not at the symptom.

## Further Reading and Sources



- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: supports the appendix's operational review habits and production-readiness vocabulary.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: keeps the appendix grounded in durable state, transactions, logs, and evidence histories.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: supports the appendix's emphasis on explicit boundaries, named types, and maintainable interfaces.