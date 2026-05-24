# Appendix X. Chapter Card Pack

## Purpose

This appendix gives prefilled chapter cards for fast restart.

Use it when attention is low, when you return after a break, or when the book
feels too large. The cards do not replace the chapters. They tell you where to
look next and what proof matters.

Each card has the same shape:

```text
chapter -> concept -> artifact -> proof -> operator question
```

That shape keeps the book simple without making it shallow.

## How To Use The Cards

Pick one row. Do not read the whole appendix at once.

Then do four small moves:

```text
1. Read the concept.
2. Open or inspect the artifact.
3. Name the proof.
4. Answer the operator question.
```

If you can do those four moves, you have a stable restart point.

If you cannot, go back to the chapter's Plain Version, Focus Cue, and
Production Artifact. Do not jump to the hardest code first.

For ADHD, fatigue, or interruption-heavy work, use the card as a seven-minute
restart:

```text
state:
move:
proof:
artifact:
failure prevented:
next:
```

Keep the `next` line concrete. Good examples are `read one test`, `run one
query`, or `inspect one Rust type`.

## Front Matter Cards

| Chapter | Concept | Artifact | Proof | Operator question |
| --- | --- | --- | --- | --- |
| System Model | Every mechanism is state, actor, transition, evidence, and invariant. | Shared notation in the chapter. | A job transition can be explained with all five terms. | What changed, who changed it, and what evidence remains? |
| Design Principles | Reliability principles order the design decisions. | Ten principles mapped to artifacts. | Each principle points to a row, type, query, test, or runbook. | Which principle is missing from this design? |
| Scope And Trade-Offs | Postgres-first is an operating envelope, not a universal answer. | Architecture comparison table. | The chosen stack names its assumptions and exit criteria. | Which missing invariant would justify moving to a workflow engine or queue system? |

## Part I Cards

| Chapter | Concept | Artifact | Proof | Operator question |
| --- | --- | --- | --- | --- |
| 1. The Problem | A model call is not a durable agent job. | `agent_jobs` row before execution. | Work survives process death before the model runs. | Can we prove the request existed before intelligence started? |
| 2. The Mental Model | An agent is a worker with tools, memory, permissions, and durable state. | Boundary diagram for state, worker, model, and policy. | Each responsibility has one owner. | Which layer owns this failure? |
| 2.5 Guarantees And Failure Semantics | The system must name what it promises and what it does not. | Failure-semantics table. | Retry, cancellation, timeout, and replay claims are explicit. | Which guarantee would a user or operator reasonably expect? |
| 3. The Postgres Ledger | Postgres is the first coordination layer. | Job, event, lease, retry, and idempotency tables. | Workers coordinate through rows, locks, leases, and events. | Which query proves what is pending, running, stuck, or dead? |
| 4. The Rust Domain Model | If a value has a role, give it a type. | Newtypes, enums, constructors, and errors. | Invalid domain values are rejected before workflow logic. | Which raw value could create a production category error? |
| 4.5 Typed Composition Lens | Safe systems compose typed transformations. | Pipeline and typestate examples. | A later step receives only the state it is allowed to handle. | Which illegal transition should become hard to express? |
| 5. The Worker Loop | The worker owns state transitions, not vague work. | `Worker::run_once` and job events. | Every move records durable evidence. | Which worker owns this job right now, and until when? |
| 6. The Rig Boundary | Rig helps the agent think and act, but it does not own reliability. | Provider runner and typed result conversion. | Raw provider output becomes validated domain output or typed failure. | Did model output cross policy before becoming trusted state? |
| 7. Running The System Locally | Local is real when it runs the same state machine. | In-memory store and deterministic runner. | Local tests exercise the same lifecycle as the Postgres path. | Which production invariant does this local run prove? |
| 8. Production Hardening | A demo becomes a service when controls become state. | Idempotency, leases, approvals, credential lifecycle, traces, and events. | Each risky behavior has a durable control. | Which ambiguity does this control remove? |
| 9. Failure Modes | Failures should become visible state. | Failure taxonomy and event evidence. | Duplicate, stuck, unsafe, or invisible work has a diagnosis path. | Which failure is currently hidden from operators? |
| 10. Capstone | Extensions must move code, state, tests, and runbooks together. | New job kind or control extension. | The new behavior has type, SQL, test, event, and operational evidence. | What changed, and what proof changed with it? |

## Part II Cards

| Chapter | Concept | Artifact | Proof | Operator question |
| --- | --- | --- | --- | --- |
| 11. The Real Postgres Store | Database rows are raw outside and typed inside. | Row conversion layer. | Bad rows fail conversion before domain logic. | Which storage value needs validation before use? |
| 12. Idempotency And Side Effects | Duplicate intent must map to one durable action path. | Idempotency key and side-effect receipt. | A repeated request returns or resumes the same logical work. | Can retry duplicate the external side effect? |
| 13. Leases, Heartbeats, And Cancellation | Ownership and stop intent are separate state. | `locked_by`, `locked_until`, heartbeat, and cancellation rows. | Dead workers lose ownership; live work can be asked to stop. | Is this job stuck, overdue, cancelled, or still owned? |
| 14. Retry, Backoff, And Dead Letters | Retry is a typed decision, not a reflex. | Failure class, attempts, next run time, and terminal state. | Transient failures retry; permanent or exhausted work stops with evidence. | Should this failure run again or become dead-lettered? |
| 15. Observability And SLOs | Observability explains one job and the fleet. | Trace ids, operation events, metrics, and SLO queries. | The same workflow can be reconstructed from multiple signals. | What happened to this job, and is the fleet healthy? |
| 16. Human Approval And Policy Gates | Approval is durable control state. | Approval request, policy decision, and audit event. | Risky actions wait for permission before execution. | Who approved this action, and what exactly did they approve? |
| 17. Testing Production Agents | Evaluation and failure tests are release controls. | Unit, boundary, SQL, live, and behavior tests. | Important failures are reproduced before production traffic. | Which test would catch this agent regression? |
| 18. Deployment And Operations | Deployment must respect work already in flight. | Config, readiness, shutdown, migration, and runbook surface. | The service can change without losing or duplicating jobs. | What happens to running jobs during deploy? |
| 19. Running For Years | Long-running agents need versioned evidence. | Schema, prompt, model, policy, worker, and compatibility versions. | Old work remains parseable, explainable, and recoverable. | Can we understand this job one year later? |
| 20. Final Production Blueprint | The serious MVP has one API, one worker, one database, and explicit controls. | End-to-end architecture map. | Each boundary has ownership, state, failure contract, and evidence. | Which component owns this production decision? |
| 20.1 Agent Handoffs And Multi-Agent Coordination | A handoff transfers responsibility, not permission. | Handoff request and target job evidence. | Accepted handoffs create or attach exactly one target job. | Who owns the next step now? |
| 20.2 Worked Production Scenario | Reliability is the cooperation of many small controls. | Scenario timeline with intake, retry, approval, receipt, and review. | Every risky transition leaves durable evidence. | Which control would have stopped the bad outcome? |

## Part III Cards

| Chapter | Concept | Artifact | Proof | Operator question |
| --- | --- | --- | --- | --- |
| 21. SLIs, SLOs, And Error Budgets | Reliability needs measured promises. | SLI query, SLO target, and error budget. | Service health is evaluated from durable facts. | Which user promise is burning budget? |
| 22. Capacity, Backpressure, And Provider Quotas | Admission control protects users, workers, and providers. | Queue metrics, tenant budget, provider usage, and admission event. | Overload becomes delay or rejection with evidence. | Should we admit, delay, reject, or scale? |
| 23. Runbooks For Agent Job Systems | A runbook is production code for humans. | Checked SQL files and operator commands. | A tired operator can inspect before acting. | What evidence should we read before changing the system? |
| 24. Incident Response And Postmortems | Incidents need evidence, ownership, and corrective action. | Incident timeline and postmortem record. | The next change points to a broken invariant. | What did we learn that changes the system? |
| 25. Release Engineering For Agents | Prompt, model, policy, schema, and code changes need release evidence. | `release_gate_runs` row and `release_gate_status.sql`. | A release is promoted, canaried, or blocked with durable reasons. | Which version produced this behavior, and what evidence allowed it to ship? |
| 26. Toil, Automation, And Ownership | Automation should preserve judgment and accountability. | Ownership map, toil review, and operation event. | Repetitive work is automated without hiding risky decisions. | Who owns this recurring pain, and what proof shows improvement? |

## Part IV Cards

| Chapter | Concept | Artifact | Proof | Operator question |
| --- | --- | --- | --- | --- |
| 27. Evaluation And Behavior Reliability | Behavior reliability is tested before promotion. | Golden dataset, rubric, result, and evaluation receipt. | A model or prompt version cannot promote without passing evidence. | Which behavior change is safe to release? |
| 27.5 Agent Memory, Retrieval, And Retention | Memory is production data with scope and authority. | Memory record, retention policy, retrieval decision, and data-protection request when needed. | Retrieval respects source, scope, confidence, retention, permission, and redaction/erasure evidence. | Why is this remembered fact allowed to influence this run, and is there open privacy work? |
| 28. Security, Abuse, And Trust Boundaries | The model is not a security boundary. | Threat model, policy gate, sandbox decision, credential lifecycle review, audit event, and data-protection review. | Tool use, memory, secrets, prompts, credentials, and privacy requests have explicit controls. | What could the agent read, write, send, leak, retain, rotate, or need to erase? |
| 28.5 Data Protection, Retention, And Privacy Operations | Privacy work is durable operational state. | `data_protection_requests` and `data_protection_review.sql`. | Redaction, erasure, export, and retention-review requests are owned, due, policy-versioned, and auditable. | Which evidence surface still has open or overdue privacy work? |
| 28.6 Tenant Isolation And Multi-Tenant Agents | Tenant scope is authorization state, not prompt text. | `authorization_events`, `tenant_boundary_review.sql`, and `DbTenantIsolationReviewRow`. | Cross-tenant attempts are denied, reviewed, and tied to policy evidence before tool execution. | Did any cross-tenant request avoid denial? |
| 29. Disaster Recovery And Continuity | Backup is not recovery until restore is practiced. | Restore drill, replay candidate, receipt check, and signoff. | Recovery meets RPO/RTO without duplicate side effects. | Can we restore and safely replay after data loss? |
| 29.5 Extreme Fault Tolerance For Agent Systems | Critical execution should survive non-critical failures deliberately. | Fault-tolerance review, readiness query, failover drill, release gate, and last-known-good versions. | A job kind is ready only when isolation, redundancy, static stability, drills, and gates agree. | Can already-approved work continue if the control plane fails? |
| 30. Reliability Maturity Model | Reliability grows by evidence, not by slogans. | Maturity scorecard by job kind. | Each level names required controls and missing gaps. | What level is this job kind actually ready for? |
| 30.5 Scaling Paths After Postgres-First | Add infrastructure only after the state machine is explicit. | Evidence-preserving migration map. | Postgres remains the source of truth or hands off with a clear contract. | Which invariant forces the next architecture step? |
| 30.6 Temporal After Postgres-First | Temporal can own workflow execution, not product truth. | Temporal adoption record. | Workflow history reconciles with Postgres product rows, approvals, receipts, and audit events. | Which workflow invariant moves, and which evidence stays in Postgres? |
| 30.7 Kafka After Postgres-First | Kafka can distribute typed events, not raw uncertainty. | Kafka adoption record. | Outbox events, topic-partition-offsets, consumer receipts, replay drills, and traces agree. | Which consumers need replay, and what prevents duplicate side effects? |

## One-Screen Restart

When a chapter feels too big, copy one row into this smaller card:

```text
chapter:
concept:
artifact:
proof:
operator question:
next action:
```

The next action should be concrete:

```text
inspect one Rust type
run one SQL query
read one test
answer one operator question
```

Do not use the card to avoid the hard parts. Use it to find the next hard part.

## Further Reading and Sources

- [CAST: Universal Design for Learning](./31-credible-resources-further-reading.md#learning-design-and-plain-language) is relevant because the card pack gives multiple ways to enter the same technical idea without changing the production target.
- [CDC: ADHD in the Classroom](./31-credible-resources-further-reading.md#learning-design-and-plain-language) is relevant because the cards make expectations, organization, and restart routines explicit for readers with limited attention.
- [Digital.gov: Short and Simple](./31-credible-resources-further-reading.md#learning-design-and-plain-language) is relevant because the cards use short, direct language while preserving the real engineering claim.
- [Paas and van Merrienboer: Cognitive-Load Theory](./31-credible-resources-further-reading.md#learning-design-and-plain-language) is relevant because the cards reduce unnecessary working-memory load before the reader studies deeper mechanisms.
- [IES: Organizing Instruction and Study to Improve Student Learning](./31-credible-resources-further-reading.md#learning-design-and-plain-language) is relevant because the cards connect concrete examples, practice prompts, and evidence checks.
- [Nature Reviews Psychology: Spacing and Retrieval Practice](./31-credible-resources-further-reading.md#learning-design-and-plain-language) is relevant because the cards support repeated recall of concepts, artifacts, and operator questions over time.
