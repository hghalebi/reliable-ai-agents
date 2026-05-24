# Appendix P. Design Smells And Failure-Mode Index

## How to Use This Appendix

Use this appendix when a chapter feels clear, but you are not sure whether you
would recognize the broken version in production.

The learning loop is:

```text
concept -> design smell -> production symptom -> corrective invariant -> evidence
```

A design smell is not a stylistic preference. It is a local signal that the
system is missing a production boundary. The smell might be a raw string, a
hidden retry loop, an unowned side effect, a dashboard without a query source,
or an approval decision stored only in conversation. The symptom is what an
operator sees later: duplicate work, stuck jobs, unexplained behavior drift,
unsafe replay, or an incident timeline that cannot be reconstructed.
The corrective invariant names what must become true, and the evidence to inspect
names the artifact that proves the correction.

Read one row after a chapter. Cover the corrective invariant and ask what
evidence would prove it. Then uncover the evidence column and compare your
answer.

## Practice Contract

When reviewing a design, write the smell in this form:

```text
I see:
It can fail by:
The invariant should be:
The evidence should be:
The smallest fix is:
```

This format keeps critique useful. It turns "this feels risky" into a precise
engineering claim that can be tested, reviewed, and owned.

## Beginner Traps And Expert Instincts

Use this section when the table feels too abstract.

The goal is to learn what experienced engineers notice first. A beginner often
sees working code. A production reviewer looks for the missing identity,
boundary, owner, transition, or proof.

Read each row as a small diagnostic card:

```text
Beginner trap:
Expert instinct:
Minimum serious fix:
Proof:
```

| Concept | Beginner trap | Expert instinct | Minimum serious fix | Proof |
| --- | --- | --- | --- | --- |
| Raw data | Use `String`, `bool`, or `serde_json::Value` because the demo is short. | If two values mean different things, they deserve different types. A raw string is not a domain model. | Parse raw input at the boundary, then convert it into newtypes, enums, or validated structs. | Constructor and row-conversion tests reject empty, unknown, or impossible values. |
| Retry | Wrap the failing call in a loop. | If an operation can be retried, it must have identity. Retry without identity creates duplicates. | Add an idempotency key, durable operation row, bounded attempt count, and terminal state. | Duplicate input returns the existing job or receipt, and retry history is queryable. |
| Side effect | Let the agent execute the tool as soon as the model proposes it. | A tool call is a side effect, not just a function call. The model may propose; the system must decide. | Parse, validate, authorize, record, approve when needed, execute once, and store a receipt. | Tool-call, approval, operation-event, receipt, and audit rows agree. |
| Worker ownership | Treat a running worker process as proof that it owns the work. | Ownership must survive process crashes and worker restarts. Memory is not ownership. | Store `locked_by`, `locked_until`, heartbeat evidence, and owner-checked completion queries. | Stale workers cannot complete work, and expired leases appear in recovery queries. |
| Observability | Add logs and call the system observable. | Logs are only one signal. Operators need a path from symptom to state, event, trace, metric, and owner. | Emit structured logs, traces, metrics, operation events, audit events, and checked runbook queries. | One trace id reconstructs a job from intake to terminal state. |
| Memory | Store useful facts as text and retrieve them later. | Memory is not truth. It is evidence with scope, source, confidence, retention, and policy. | Store typed memory records with provenance, retention policy, redaction path, and access scope. | Memory rows decode into domain records, and retrieval is policy-checked before prompt use. |
| Scaling | Add Temporal, Kafka, or a queue when the system feels serious. | Infrastructure should receive a named responsibility, not hide an unclear state machine. | First name the invariant that moves, the evidence that remains, coexistence, rollback, and runbook changes. | Old and new evidence sources reconcile for one real agent run. |

These cards are intentionally simple.

They are not lower-rigor versions of the book. They are fast ways to notice the
same production invariants under stress.

## Front Matter Smells

| Chapter | Design smell | Production symptom | Corrective invariant | Evidence to inspect |
| --- | --- | --- | --- | --- |
| System Model And Notation | A design names components but not state transitions. | Reviewers cannot tell what changed or who was allowed to change it. | Every serious mechanism is expressible as state -> actor -> transition -> evidence -> invariant. | A job row, event, receipt, policy decision, or evaluation record proves the transition. |
| Design Principles | Principles are slogans rather than constraints. | Teams agree with the prose but ship systems that violate it. | Each principle maps to an artifact that prevents one failure class. | Design review rows connect principle, artifact, failure prevented, and owner. |
| Production Scope And Trade-Offs | Architecture is chosen by preference instead of operating envelope. | The system later needs replay, timers, or orchestration that the chosen design never owned. | The architecture states which guarantees it provides and which it deliberately does not. | Scope notes name script, queue framework, Postgres ledger, workflow engine, or platform boundary. |

## Part I Smells

| Chapter | Design smell | Production symptom | Corrective invariant | Evidence to inspect |
| --- | --- | --- | --- | --- |
| 1. The Problem | The model call is treated as the workflow. | A process crash loses work or makes duplicate intake impossible to explain. | Durable work exists before intelligence runs. | The enqueue path writes a job row and intake event before the model step. |
| 2. The Mental Model | Boundaries collapse into one agent loop. | Provider errors, policy decisions, and state transitions blur together during incidents. | State, worker, provider, policy, and evidence boundaries stay separate. | Event timelines distinguish worker actions, model output, policy results, and terminal state. |
| 2.5 Guarantees And Failure Semantics | The system implies exactly-once behavior without naming the guarantee. | Replays duplicate side effects or operators expect impossible crash behavior. | Execution guarantees and side-effect guarantees are stated separately. | Failure semantics cover duplicate intake, provider timeout, worker crash, and replay. |
| 3. The Postgres Ledger | The table stores status but not ownership, history, duplicate identity, or a checked scheduled-job claim path. | Operators see a stuck status but cannot explain who owns the job, whether it is due, or why it changed. | The ledger stores current state, lease ownership, idempotency, retries, due time, and events. | Schema, typestate, and queries expose `agent_jobs`, `scheduled_jobs`, event rows, leases, attempts, next run time, and idempotency key. |
| 4. The Rust Domain Model | Raw primitives cross domain boundaries. | Values with different meanings are swapped, accepted empty, or persisted invalidly. | Meaningful domain values are named, validated, and converted at boundaries. | Newtypes, enums, smart constructors, and conversion tests reject invalid values. |
| 4.5 Typed Composition Lens | Lifecycle order is enforced by convention. | A request can be sent before auth, payload validation, or policy preparation. | Invalid lifecycle transitions are not expressible through the public API. | Typestate builders expose `build` only after required states exist. |
| 5. The Worker Loop | The worker mutates state without writing transition evidence. | A job moves from pending to terminal state with no trustworthy explanation. | Every important transition writes durable evidence. | Worker tests and event queries show pick, start, result, retry, and terminal events. |
| 6. The Rig Boundary | Provider DTOs or raw model text leak into worker logic. | One provider response change or malformed model field forces retry, policy, and worker code changes. | Provider-specific shape is converted once into typed domain outcomes. | The worker depends on `AgentRunner`; `agent_output.rs` validates provider text; provider adapters classify timeout, malformed output, and terminal failures. |
| 7. Running The System Locally | Local runs prove only that one command returns text. | The system appears usable while the state machine remains untested. | Local execution exercises the same lifecycle as production. | Deterministic tests prove enqueue, lease, retry, success, dead-letter, and recovery behavior. |
| 8. Production Hardening | Hardening controls are treated as later polish. | Duplicate intake, lost ownership, unapproved side effects, stale credentials, and invisible failures appear together. | Idempotency, leases, approval, observability, error classification, and credential lifecycle are core controls. | Schema, worker tests, runbook queries, and configuration boundaries show each control. |
| 9. Failure Modes | Failures are handled by logging and continuing. | Jobs vanish into retry loops or fail without operator-readable state. | Failures become classified state transitions. | Dead jobs, retry events, permanent errors, and policy stops can be queried. |
| 10. Capstone | A new job kind changes only one enum or command. | The feature compiles but lacks lifecycle, SQL, tests, or operations. | A new job kind moves schema, state, worker behavior, tests, and runbooks together. | Review artifacts list every changed source, query, test, and operator command. |

## Part II Smells

| Chapter | Design smell | Production symptom | Corrective invariant | Evidence to inspect |
| --- | --- | --- | --- | --- |
| 11. The Real Postgres Store | Database rows are trusted as domain values. | Bad persisted data reaches worker logic and fails far from the boundary. | Row conversion validates before constructing domain objects. | Store conversion tests reject invalid attempts, versions, payloads, and negative counts. |
| 12. Idempotency And Side Effects | Retry is added before duplicate identity, outbox publication, receipts, and compensation. | A duplicate webhook, replay, timeout, crash between commit and publish, or unsafe rollback issues the same external action twice, loses it, or reverses it without evidence. | One logical request maps to one durable job, one durable publication path, one durable side-effect receipt, or one approved compensation action. | Unique idempotency keys, duplicate enqueue behavior, outbox events, side-effect receipts, and compensation actions prove the mapping. |
| 13. Leases, Heartbeats, And Cancellation | Any worker can finish visible work, lease expiry is used as a business timeout, or cancellation is treated as process killing. | Two workers mutate the same job, a cancellation request stays invisible, or a validly leased job violates its time promise. | Lease ownership, deadline policy, and cancellation intent are separate invariants. | SQL predicates and tests require `locked_by` for owner mutations; timeout policy and breached-deadline query prove slow-work handling; cancellation request rows prove stop intent and outcome. |
| 14. Retry, Backoff, And Dead Letters | Retry is a loop around a failing call. | Permanent failures consume capacity, hide defects, and never reach inspection. | Retry is a typed scheduling decision with a terminal stop path. | Retry disposition, next run time, attempt count, and dead-letter reason are stored. |
| 15. Observability And SLOs | Dashboards are built from convenient process logs, or one `/health` endpoint is asked to prove everything. | Alerts fire without a path to the stuck job, failed invariant, decision authority, runtime symptom, dependency readiness, or SLO measurement. | Observability signals agree with durable workflow evidence, and audit evidence stays separate from operation evidence. | `/healthz`, `/readyz`, `/metrics`, traces, logs, event rows, audit events, operation events, and typed SLI measurements point to the same job and state transition. |
| 16. Human Approval And Policy Gates | Approval or escalation lives in chat, tickets, or model text. | A risky action executes without durable authorization evidence, or unsafe autonomous progress has no named human owner. | Approval and escalation are state outside the model. | Proposal, policy version, actor, reason, timestamp, side-effect receipt, escalation severity, assigned owner, and resolution evidence are recorded. |
| 17. Testing Production Agents | Tests mock away provider and persistence contracts. | The unit suite passes while real malformed output or SQL behavior breaks production. | Tests cover pure domain logic, SQL semantics, provider fixtures, behavior evals, and live smoke paths. | Test names and readiness commands map each boundary to evidence. |
| 18. Deployment And Operations | Deploy assumes API and worker processes can change independently without an admission protocol. | Jobs are admitted with a new shape that old workers cannot decode, or running jobs are abandoned during rollout. | API admission, shutdown, lease expiry, versioning, and restart behavior are designed together. | API and worker binaries, worker lifecycle, lease rules, version fields, and runbook commands explain deploy safety. |
| 19. Running For Years | Old work is expected to remain understandable by memory. | A six-month-old job cannot be replayed or explained after prompt, model, policy, or code drift, or a worker claims a row with an unsupported payload schema. | Long-lived jobs retain enough version evidence to explain old behavior and quarantine incompatible execution. | Job rows record schema, prompt, model, policy, worker, and evaluation versions; worker compatibility policy and the compatibility-risk query expose old/new schema risks. |
| 20. Final Production Blueprint | The architecture diagram has arrows but no API admission contract or handoff evidence. | During an incident, every layer or specialist agent can blame another layer. | Each boundary owns specific state, failures, handoffs, and evidence. | Blueprint review maps API intake, store, worker, provider, handoff, policy, side effect, and operator surfaces. |
| 20.1 Agent Handoffs And Multi-Agent Coordination | Agents pass responsibility through conversation text, raw JSON, or memory instead of a durable handoff. | Work waits forever between specialists, duplicate target jobs appear, or no one can prove which agent owns the next step. | A handoff is a typed, idempotent, persisted transfer with target-job or terminal-decision evidence. | `agent_handoffs`, operation events, trace ids, target jobs, decision reasons, and pending-handoff runbook output prove the transfer. |
| 20.2 Worked Production Scenario | The happy path is understood but the evidence chain is not. | A duplicate webhook plus provider timeout cannot be reconstructed after the fact. | Risky work advances only through durable, inspectable transitions. | Scenario timeline links duplicate intake, retry, handoff, approval, receipt, and operator review. |

## Part III Smells

| Chapter | Design smell | Production symptom | Corrective invariant | Evidence to inspect |
| --- | --- | --- | --- | --- |
| 21. SLIs, SLOs, And Error Budgets | Reliability is described as "healthy" or "fast" without a query and typed measurement boundary. | Teams debate incidents using impressions or dashboard-only math instead of measured promises. | Every SLI has a durable or reproducible measurement source and impossible measurements are rejected before they drive operations. | SLI queries, typed SLO row conversion, SLO windows, burn-rate alerts, and owners are written down. |
| 22. Capacity, Backpressure, And Provider Quotas | Overload is solved only by adding workers. | Provider 429s, retry storms, queue age, and spend rise together. | Admission, execution, provider pressure, and budget pressure are controlled together. | Queue age, per-kind backlog, worker concurrency, provider usage rows, and budget decisions agree. |
| 23. Runbooks | Runbooks are prose advice without checked commands. | Operators improvise during incidents and copy risky SQL from notes. | Runbooks execute named, reviewed diagnostic and control artifacts. | `psql -f` commands point to checked SQL files for health, timelines, breached deadlines, cancellation requests, audit events, operation events, handoff backlog, outbox backlog, compensation backlog, restore replay, pause, and resume. |
| 24. Incident Response And Postmortems | Incidents end when service recovers. | The same failure recurs because no invariant or owner changed. | An incident produces timeline evidence, mitigation, root cause, action item, and owner. | Postmortems link failed invariant to regression tests, runbook changes, or design changes. |
| 25. Release Engineering | New code assumes no old jobs exist, or one green signal is treated as release approval. | Old rows, old prompts, old workers, exhausted SLO budgets, or missing approval evidence break during rollout. | Releases are compatible with work already in the ledger and promotion combines behavior, SLO, compatibility, version, and approval evidence. | Expand-contract migrations, version fields, canary workers, eval receipts, compatibility reports, SLO evaluations, approval evidence, and release gate reports prove compatibility. |
| 26. Toil, Automation, And Ownership | Automation is added without ownership or rollback. | A script repeats unsafe action faster than a human would. | Automation preserves evidence, has an owner, and has a stop path. | Toil budgets, automation scope, approval boundaries, rollback commands, and owner records exist. |

## Part IV Smells

| Chapter | Design smell | Production symptom | Corrective invariant | Evidence to inspect |
| --- | --- | --- | --- | --- |
| 27. Evaluation And Behavior Reliability | Availability is treated as behavior quality. | The system is up while answers drift, policy compliance weakens, or tool choices degrade. | Behavior releases require evaluation evidence. | Dataset, rubric, grader, human review sample, and evaluation receipt bind to prompt/model versions. |
| 27.5 Agent Memory, Retrieval, And Retention | Memory is stored as raw strings or embeddings without source, scope, confidence, horizon, retention, or review policy. | A poisoned, stale, cross-tenant, or model-guessed memory silently changes future behavior. | Memory is typed, scoped, sourced, retained, confidence-scored, redacted, and policy-checked before prompt use. | `agent_memory_records`, row conversion tests, retention constraints, embedding references, redacted debug output, operation events, audit events, and memory-by-scope query. |
| 28. Security, Abuse, And Trust Boundaries | Untrusted text is allowed to carry authority, or execution proceeds after only one security check passed. | Prompt injection changes tool behavior, memory access, network egress, filesystem paths, secret exposure, approval state, credential lifecycle, or cross-tenant action. | Authority comes from typed policy, authorization, sandbox, approval, credential lifecycle, and execution-gate decisions, not model text, and every decision leaves audit evidence. | Tool contracts, authorization events, sandbox events, tool-execution gate tests, secret references, credential lifecycle review, memory policy, audit events, and operation events enforce boundaries. |
| 28.5 Data Protection, Retention, And Privacy Operations | Redaction, erasure, export, or retention review is tracked in informal notes or manual SQL edits. | A user or regulator asks what happened, but the team cannot prove which surfaces were reviewed, which data was changed, or why some evidence was retained. | Privacy requests are durable, typed, due-dated, policy-versioned, and completed through audited operations. | `data_protection_requests`, `data_protection_review.sql`, `DbDataProtectionReviewRow`, policy version, audit event, operation event, and completion evidence prove the workflow. |
| 28.6 Tenant Isolation And Multi-Tenant Agents | Tenant id is passed through prompts, tool JSON, or frontend state as an ordinary string with no authorization evidence. | A memory lookup, document read, CRM update, or external message uses the wrong tenant scope. | Actor tenant and requested tenant are separate typed facts checked by deterministic policy before tool execution. | `authorization_events`, `tenant_boundary_review.sql`, `DbTenantIsolationReviewRow`, denial reason, policy version, trace id, audit event, and operation event prove the boundary. |
| 29. Disaster Recovery And Continuity | Backup existence is treated as recovery readiness. | Restore duplicates side effects or resumes workers before replay safety is known. | Recovery is practiced restore, replay, receipt handling, and controlled resume. | `ReplayDecision`, `restore_replay_candidates.sql`, and restore drill records prove RPO, RTO, side-effect receipts, provider continuity, and operator signoff. |
| 29.5 Extreme Fault Tolerance For Agent Systems | The execution plane reads live control-plane state during critical work. | A dashboard, prompt editor, or admin API outage stops already-approved production jobs. | Critical execution is isolated, redundant, and statically stable from last-known-good state. | `fault_tolerance_reviews`, `fault_tolerance_readiness.sql`, `DbFaultToleranceReadinessRow`, failover drill status, release gate status, and last-known-good versions prove the claim. |
| 30. Reliability Maturity Model | Maturity labels are aspirational. | A risky job is called "production-ready" without the controls its risk requires. | Maturity is assigned per job kind from evidence. | Scorecards list current evidence, gaps, owners, review dates, and next upgrades. |
| 30.5 Scaling Paths After Postgres-First | Infrastructure is added before naming the invariant it will own. | The scaled system has more components but weaker idempotency, auditability, replay safety, or operator evidence. | Each scaling step preserves or strengthens a named control from the Postgres-first system. | Migration notes, baseline metrics, coexistence tests, trace correlation, operation events, runbook updates, and rollback criteria prove the change. |
| 30.6 Temporal After Postgres-First | Temporal history is treated as the product audit trail. | Operators can see a workflow retry but cannot prove which approval, tool receipt, policy version, or audit event authorized the business action. | Temporal may own workflow execution mechanics; Postgres keeps product truth, receipts, approvals, and audit evidence. | A Temporal adoption packet maps workflow ids, activity receipts, Postgres rows, approvals, audit events, trace ids, reconciliation runbooks, and rollback criteria. |
| 30.7 Kafka After Postgres-First | Kafka is added before event ownership, schema, partition key, and consumer idempotency are defined. | Replays or duplicate deliveries update projections twice, leak unsafe payloads, or create a second source of truth that disagrees with Postgres. | Kafka distributes selected typed facts from the outbox; consumers record idempotent receipts and replay rules preserve side-effect safety. | A Kafka adoption packet maps outbox event, event envelope, schema version, topic-partition-offset, consumer receipt, authorization boundary, replay rule, and trace id. |

## Production Contract

Use this appendix during design review and incident review. A smell is actionable
only when it names:

```text
the local shortcut
the production symptom
the invariant being violated
the evidence that should exist
the smallest change that makes recurrence harder
```

If a row feels too abstract, return to the primary chapter and find the query,
type, event, metric, receipt, policy, or test that makes the invariant concrete.

## Summary

Concepts become durable when readers can recognize their broken forms. This
appendix turns each chapter into a diagnostic habit: notice the smell, predict
the production symptom, restore the invariant, and inspect evidence instead of
arguing from confidence.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
