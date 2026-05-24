# Appendix S. Formal Definition Ledger

## Motivation

The main chapters start with operational pain and tiny examples because that is
how engineers build intuition. A serious technical book also needs precise
definitions. This appendix gives those definitions in one place so the reader
can move from story to mechanism to reviewable contract.

The pattern is the same throughout the book:

```text
state -> actor -> transition -> evidence -> invariant
```

A definition is useful only when it tells the reader what state exists, who may
change it, what transition is allowed, what evidence remains, and which
invariant must survive crash, retry, deploy, restore, or audit.

## How to Use This Ledger

Use this appendix after a chapter when the idea feels intuitive but not yet
sharp. Read the chapter row and ask:

```text
Can I name the state?
Can I name the actor allowed to change it?
Can I name the transition and precondition?
Can I name the evidence left behind?
Can I name the invariant that survives failure?
```

If one answer is vague, return to the chapter, Appendix B, Appendix G, or the
companion code reading path. The goal is not to memorize phrasing. The goal is
to make every production concept crisp enough for design review.

## Formal Definition Ledger

| Chapter | Formal definition | Required evidence |
| --- | --- | --- |
| System Model And Notation | A reliable agent system is a set of named states changed by authorized actors through explicit transitions that leave durable evidence and preserve invariants. | A reviewer can express each mechanism as state, actor, transition, evidence, and invariant. |
| Design Principles | A design principle is a reusable ordering rule that prevents a later mechanism from depending on an earlier missing invariant. | The principle points to chapters, artifacts, review questions, and common failure repairs. |
| Production Scope And Trade-Offs | An architecture choice is the assignment of responsibility for durable state, coordination, side effects, recovery, and operator evidence to concrete system components. | The operating envelope names when Postgres-first is sufficient and when another control surface owns the harder problem. |
| 1. The Problem | An agent job is a durable unit of model-powered work whose existence and state are recorded before the model is allowed to run. | A job row exists before execution, and the model call is one transition inside that job. |
| 2. The Mental Model | A production agent is a worker with tools, memory, permissions, and evidence around a probabilistic model step. | State, worker, provider boundary, policy, event timeline, and operator evidence are separate responsibilities. |
| 2.5 Guarantees And Failure Semantics | A guarantee is a bounded promise about what the system will preserve under failure, and a non-guarantee is an assumption the system refuses to hide. | Failure semantics are written down before retries, leases, recovery, or side effects depend on them. |
| 3. The Postgres Ledger | A Postgres ledger is the durable coordination layer that stores current state, transition history, retry state, leases, and idempotency identity. | SQL constraints, row locks, indexes, tracking tables, and event rows prove the state machine. |
| 4. The Rust Domain Model | A domain model is the typed interior representation that gives meaning, validation, and lifecycle rules to values crossing raw boundaries. | Newtypes, enums, smart constructors, conversion tests, and typed errors reject invalid or confused values. |
| 4.5 Typed Composition Lens | Typed composition is the safe connection of transformations whose output and input types match, with typestate used when lifecycle order matters. | Newtypes, typestate builders, and `Result` pipelines make illegal composition visible before runtime. |
| 5. The Worker Loop | A worker loop is a leased transition executor that repeatedly claims due work, records evidence, runs one step, and commits the next durable state. | Pick, heartbeat, success, retry, cancellation, and recovery transitions require ownership and emit events. |
| 6. The Rig Boundary | The Rig boundary is the adapter layer where provider behavior becomes typed agent output or typed failure before worker policy sees it. | Provider DTOs do not leak into the worker, malformed output is rejected, and retry disposition is typed. |
| 7. Running The System Locally | A local run is a deterministic proof that the same state machine works before external infrastructure or live provider behavior is required. | The local agent, Rig-backed agent, in-memory store, and Postgres store preserve the same job semantics. |
| 8. Production Hardening | Production hardening is the addition of controls that turn hidden failure into state, evidence, policy, or operator action. | Idempotency, leases, approval gates, observability, secret handling, and recovery paths are visible in code and runbooks. |
| 9. Failure Modes | A failure mode is a predictable way the system can lose ownership, identity, evidence, permission, or terminality. | The design maps each failure to a corrective invariant and inspectable evidence. |
| 10. Capstone | An extension is production-safe only when it adds behavior without weakening existing state, evidence, idempotency, approval, or replay invariants. | New commands, states, SQL, tests, and runbooks move together. |
| 11. The Real Postgres Store | A Postgres store boundary is the adapter that converts storage-friendly rows into validated domain values and preserves SQL transition contracts. | Row conversion rejects corrupted state before business logic, and SQL predicates preserve ownership. |
| 12. Idempotency And Side Effects | Idempotency is the mapping from one logical intent to one durable action path even when requests, workers, or providers repeat. | Idempotency keys, duplicate-suppression events, outbox rows, side-effect receipts, and compensation records agree. |
| 13. Leases, Heartbeats, And Cancellation | A lease is temporary mutation authority; a heartbeat renews that authority; cancellation is durable stop intent with an observed outcome. | Ownership predicates, database time, deadlines, cancellation requests, and recovery queries are separate and inspectable. |
| 14. Retry, Backoff, And Dead Letters | Retry is a typed decision to schedule future work after a classified transient failure; dead lettering is terminal evidence that retry is no longer useful. | Attempt counts, failure class, next retry time, backoff, dead reason, and append-only failure history are persisted. |
| 15. Observability And SLOs | Observability is the ability to reconstruct behavior from correlated state, events, metrics, traces, and logs without relying on process memory. | Trace ids, structured fields, operation events, audit events, metrics, and runbook queries answer the same job question. |
| 16. Human Approval And Policy Gates | Approval is durable decision state that authorizes or rejects a proposed risky action after deterministic policy has classified the risk. | Proposal, policy version, actor, reason, decision time, and receipt exist before side-effect execution. |
| 17. Testing Production Agents | Production testing is the evidence system that proves state transitions, boundaries, behavior, failure handling, and provider compatibility before release. | Unit, SQL, feature, integration, live-provider, evaluation, simulation, and failure-drill checks map to explicit risks. |
| 18. Deployment And Operations | A deployment is a controlled compatibility transition where old work, current leases, secrets, schema, and operator controls remain valid. | Shutdown, pause, replay, version, migration, health, readiness, metrics, and provider smoke evidence exist. |
| 19. Running For Years | Long-horizon operation is the ability to explain, parse, recover, and safely resume old work after code, schema, model, policy, and provider changes. | Version fields, retention rules, compatibility checks, ownership records, and restore drills are maintained. |
| 20. Final Production Blueprint | A production blueprint is the allocation of every failure boundary to a component that owns the state, transition, evidence, and invariant. | API, Postgres, worker, Rig boundary, policy, approval, side effects, and operations each have one clear responsibility. |
| 20.1 Agent Handoffs And Multi-Agent Coordination | A handoff is durable responsibility transfer from one named agent/run to another, with an explicit reason, payload, decision, and target-work evidence. | Source run, source agent, target agent, idempotency key, accepted or rejected decision, and target job evidence are stored. |
| 20.2 Worked Production Scenario | A worked scenario is an end-to-end proof that independent controls compose without losing evidence before a risky side effect. | Duplicate intake, retry, handoff, approval, receipt, and operator review form one traceable chain. |
| 21. SLIs, SLOs, And Error Budgets | An SLI is a reproducible measurement, an SLO is a promise over that measurement, and an error budget is the allowed failure before behavior changes. | Query sources, typed measurements, windows, targets, burn alerts, owners, and release decisions are explicit. |
| 22. Capacity, Backpressure, And Provider Quotas | Capacity control is the admission and execution policy that keeps work, providers, tenants, costs, and approvals inside safe limits. | Queue age, backlog, priority, worker concurrency, provider usage, tenant budgets, and pause state shape intake. |
| 23. Runbooks For Agent Job Systems | A runbook is an operator transition protocol that reads evidence before action and preserves the reason for any state change. | Checked SQL files, API health checks, diagnostics, pause/resume commands, replay rules, and evidence notes exist. |
| 24. Incident Response And Postmortems | Incident response is the controlled reduction of harm after an invariant fails; a postmortem turns the failed invariant into stronger system evidence. | Triage facts, mitigation notes, preserved evidence, action items, owners, and regression tests are recorded. |
| 25. Release Engineering For Agents | Release engineering is the compatibility discipline for changing code, schema, prompts, models, tools, and policies while old work remains safe. | Expand-contract migrations, canaries, version fields, release gates, evaluation receipts, and rollback evidence exist. |
| 26. Toil, Automation, And Ownership | Ownership is durable responsibility for a job kind; toil is repeated manual work that should become safer automation only when the invariant is clear. | Owners, toil budgets, runbooks, automation boundaries, approval rules, and rotation evidence are maintained. |
| 27. Evaluation And Behavior Reliability | Evaluation is the release-control system for probabilistic behavior, tying outputs to datasets, rubrics, prompts, models, tools, and policies. | Evaluation runs, dataset versions, grader versions, human review samples, and promotion decisions are stored. |
| 27.5 Agent Memory, Retrieval, And Retention | Agent memory is typed production data with scope, kind, source, confidence, retention, and authority rules before it can influence prompts. | Memory metadata, redaction policy, retention rules, retrieval authorization, and audit evidence are queryable. |
| 28. Security, Abuse, And Trust Boundaries | A trust boundary is the line where text, tool input, memory, credentials, or tenant context loses authority until validated outside the model. | Authorization events, sandbox events, secret references, typed tool calls, audit logs, and policy decisions enforce the boundary. |
| 28.5 Data Protection, Retention, And Privacy Operations | Data protection is the operational control that turns redaction, erasure, export, retention, and privacy review into durable, policy-versioned workflow state. | Data-protection request rows, review queries, completion evidence, audit events, operation events, policy versions, and owners are recorded. |
| 28.6 Tenant Isolation And Multi-Tenant Agents | Tenant isolation is the durable control that prevents one tenant's data, memory, tools, side effects, and evidence from being read or changed by an actor that lacks tenant-scoped authorization. | Actor tenant, requested tenant, permission, policy version, authorization event, tenant-boundary review row, trace id, operation event, and audit event agree. |
| 29. Disaster Recovery And Continuity | Disaster recovery is the practiced ability to restore state, classify replay safety, prevent duplicate side effects, and resume work inside RPO and RTO. | Restore drill rows, replay decisions, receipt checks, provider continuity, paused workers, and operator signoff exist. |
| 29.5 Extreme Fault Tolerance For Agent Systems | Extreme fault tolerance is the practiced ability for critical execution to keep serving, degrade, or pause deliberately when isolated parts fail, using redundancy and last-known-good state instead of live fragile dependencies. | Fault-tolerance review rows, redundant worker counts, isolated failure domains, static-stability mode, failover drills, release gates, and typed readiness decoding agree. |
| 30. Reliability Maturity Model | A maturity level is an evidence-backed target for one job kind, not a general label for the whole product. | Each level names current proof, gap, next upgrade, owner, and review date. |
| 30.5 Scaling Paths After Postgres-First | Scaling is the migration or duplication of responsibility to new infrastructure without losing the old state machine or evidence contract. | Baseline metrics, old/new evidence maps, dual-run or coexistence plans, trace correlation, and rollback criteria are recorded. |
| 30.6 Temporal After Postgres-First | Temporal adoption is the evidence-preserving migration of workflow execution responsibility to a durable workflow engine while product truth remains typed, audited, and queryable in Postgres. | Workflow id mapping, activity receipts, product rows, approval decisions, side-effect receipts, audit events, trace ids, reconciliation runbooks, and rollback criteria agree. |
| 30.7 Kafka After Postgres-First | Kafka adoption is the evidence-preserving migration of event distribution responsibility to a partitioned log while product truth, event schemas, consumer idempotency, and audit evidence remain explicit. | Outbox rows, event envelopes, schema versions, topic-partition-offsets, consumer receipts, replay rules, authorization boundaries, projection evidence, and trace ids agree. |

## Reading the Columns

The "formal definition" column should read like a contract, not like a slogan.
Each row names the object being defined and the condition under which it is
valid. The "required evidence" column names the proof that would let a reviewer
say the concept is implemented rather than merely described.

This distinction matters most when reviewing agent systems. A team can say
"we have retries" while retrying non-idempotent side effects. It can say "we
have observability" while depending on log search to reconstruct old work. The
definition is the standard; the evidence is the proof.

## Production Contract

Every core concept in the book should be reducible to:

```text
defined concept
allowed state
authorized actor
legal transition
durable evidence
surviving invariant
```

If a concept cannot be reduced this way, it is not yet ready to carry
production responsibility. It may still be a useful idea, but it needs a
smaller definition before it becomes architecture.

## Summary

The main chapters teach from intuition to mechanism. This appendix compresses
each mechanism into a definition that can be used in design review, code
review, incident response, and readiness assessment.

A production agent is not reliable because its prose sounds confident. It is
reliable when its concepts have definitions and those definitions have
evidence.

## Further Reading and Sources

- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) supports the appendix's focus on durable state, transactions, logs, and recoverable evidence.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) grounds the definitions in operational review, incident response, and reliability practice.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) supports the typed-boundary definitions used for newtypes, error contracts, and maintainable Rust interfaces.
- [PostgreSQL `SELECT` documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) anchors the ledger and worker-claim definitions in the row-locking behavior used by the companion system.
