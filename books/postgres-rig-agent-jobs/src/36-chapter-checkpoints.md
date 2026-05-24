# Appendix F. Chapter Checkpoints

## How to Use These Checkpoints

This appendix turns the book into a sequence of small engineering proofs. Use it
after each chapter, before moving to the next one.

For each chapter, answer three questions:

```text
prerequisite:
  What earlier idea does this chapter depend on?

mental simulation:
  What tiny job, state transition, query, or incident can I run in my head?

production evidence:
  What artifact would prove this idea in a real system?
```

The goal is not to memorize chapter summaries. The goal is to build a habit:
every concept should connect to an invariant, and every invariant should have
evidence.

## Front Matter Checkpoints

| Chapter | Prerequisite to recall | Mental simulation | Production evidence to name |
| --- | --- | --- | --- |
| System Model And Notation | A script becomes a system when state, actors, transitions, evidence, and invariants are visible. | Rewrite one retry as state -> actor -> transition -> evidence -> invariant. | Shared notation for job rows, event timelines, policy decisions, evaluation receipts, and side-effect receipts. |
| Design Principles | A mechanism is useful only when it protects a production promise. | Map one principle to one artifact and one failure it prevents. | Durable-before-intelligent and the remaining principles connected to review questions. |
| Production Scope And Trade-Offs | Architecture choice should follow the invariant that must be preserved. | Decide whether one risky job belongs in a script, queue, Postgres ledger, workflow engine, or larger platform. | Operating envelope, assumptions, alternatives, and evidence contract for the chosen architecture. |

## Part I Checkpoints

| Chapter | Prerequisite to recall | Mental simulation | Production evidence to name |
| --- | --- | --- | --- |
| 1. The Problem | A model call is only one step in a larger workflow. | A worker crashes after the model returns but before the result is saved. | Durable job row, terminal state, event timeline. |
| 2. The Mental Model | State, execution, intelligence, observability, and policy are separate responsibilities. | Follow one job from API intake to event history. | Boundary diagram, state table, event table. |
| 2.5 Guarantees And Failure Semantics | Guarantees must be written before code depends on them. | Ask what happens after a duplicate request, crash, timeout, or replay. | Failure semantics table and tests for each promise. |
| 3. The Postgres Ledger | Durable work must exist before model execution. | Two workers try to claim the same pending row. | `FOR UPDATE SKIP LOCKED`, lease fields, idempotency key. |
| 4. The Rust Domain Model | The worker can only protect concepts the code can name. | Try to pass a raw status string into the transition logic. | Newtypes, enums, validating constructors, transition tests. |
| 4.5 Typed Composition Lens | Illegal states should become hard to express. | Build a request before authentication or payload validation. | Type-state or constructor boundary that blocks invalid usage. |
| 5. The Worker Loop | A job moves through explicit states, not ad hoc branches. | Execute pending -> running -> succeeded and record each event. | Transition function, lease ownership check, event insert. |
| 6. The Rig Boundary | Provider behavior should enter through one adapter. | Convert a provider timeout into a system retry decision. | Typed provider error, fixture, retry classification test. |
| 7. Running The System Locally | Local runs should exercise the same state machine as production. | Run a deterministic fake model through one job. | Passing local command and state/event output. |
| 8. Production Hardening | A demo becomes a service only when failure is visible and bounded. | Submit the same request twice during a provider outage. | Idempotency, retry, lease, observability, and approval evidence. |
| 9. Failure Modes | Hidden failure is worse than visible failure. | Decide whether a malformed payload should retry or stop. | Dead-letter state, reason field, operator query. |
| 10. Capstone | Extensions must preserve the existing invariants. | Add a new job kind without changing the safety model. | New states, tables, commands, invariants, and tests updated together. |

## Part II Checkpoints

| Chapter | Prerequisite to recall | Mental simulation | Production evidence to name |
| --- | --- | --- | --- |
| 11. The Real Postgres Store | Database rows are boundary data, not domain objects. | Load a row with an invalid status value. | Row-to-domain conversion error and schema constraint. |
| 12. Idempotency And Side Effects | Retrying uncertain work is safe only when intent has one durable identity. | The same webhook arrives twice with the same logical request. | Idempotency key, conflict handling, single side-effect path. |
| 13. Leases, Heartbeats, And Cancellation | Running work must have an owner, a lease expiry, a separate execution deadline, and durable stop intent. | A worker keeps heartbeating while the job exceeds its deadline and an operator requests cancellation. | `locked_by`, `locked_until`, heartbeat, recovery query, timeout policy, breached-deadline query, cancellation request row. |
| 14. Retry, Backoff, And Dead Letters | Failure classification comes before retry scheduling. | Compare provider timeout with missing API key. | Retry disposition, attempt cap, dead-letter event. |
| 15. Observability And SLOs | Operators need state, audit events, operation events, runtime health surfaces, metrics, and traces to agree. | Investigate an old pending job from `/metrics` to queue metric, event timeline, audit evidence, and operation evidence. | `/healthz`, `/readyz`, `/metrics`, SLI query, trace id, job events, audit events by run, operation events by job, runbook command. |
| 16. Human Approval And Policy Gates | The model may propose; policy decides permission. | A model recommends sending an external email. | Policy version, risk level, human decision, immutable proposal snapshot. |
| 17. Testing Production Agents | Tests should target boundaries where failures enter. | Break provider response shape and predict the expected failure. | Fixture, contract test, typed error assertion. |
| 18. Deployment And Operations | Deploys happen while old work still exists. | Shut down a worker while the API is still admitting jobs. | API and worker binaries, graceful shutdown, lease release or expiry, runbook query. |
| 19. Running For Years | Time changes prompts, schemas, models, providers, and teams. | Reprocess a one-year-old job after a schema change. | Versioned rows, migration path, retention policy, ownership record. |
| 20. Final Production Blueprint | The system is a set of boundaries with contracts. | Point to the owner of each failure: API intake, lease, provider, handoff, policy, side effect. | Architecture blueprint with API admission tests, failure contracts, and handoff evidence. |
| 20.1 Agent Handoffs And Multi-Agent Coordination | Multi-agent work is responsibility transfer, not conversation. | A triage agent asks a deployment-safety agent to own a rollback check. | Handoff row, source run, target agent, idempotency key, target job evidence, decision reason, and pending-handoffs query. |
| 20.2 Worked Production Scenario | Controls must cooperate in one evidence chain. | Duplicate webhook, timeout, retry, handoff, approval, and side-effect receipt. | Admission event, retry event, handoff row, approval record, receipt, operator review. |

## Part III Checkpoints

| Chapter | Prerequisite to recall | Mental simulation | Production evidence to name |
| --- | --- | --- | --- |
| 21. SLIs, SLOs, And Error Budgets | A reliability promise needs a measured source of truth. | Convert job success into an SLI, typed measurement, error-budget decision, and alert. | SLI query, typed SLO row conversion, SLO window, burn-rate alert, owner. |
| 22. Capacity, Backpressure, And Provider Quotas | Overload should be admitted, delayed, or rejected deliberately. | Provider quota drops while the queue is growing. | Concurrency limit, quota signal, backlog metric, provider usage record, budget decision. |
| 23. Runbooks For Agent Job Systems | Operators need questions that map to evidence. | A customer asks why their job is still pending between specialist agents or waiting for cancellation. | Queue health query, oldest pending query, breached-deadline query, pending handoffs query, pending cancellation query, event timeline, audit query, operation query. |
| 24. Incident Response And Postmortems | Incidents are system feedback, not only emergencies. | A bad prompt version increases dead-letter rate. | Timeline, mitigation, postmortem, action item, regression test. |
| 25. Release Engineering For Agents | Old and new code must coexist during change. | Deploy a new result shape while old jobs are running. | Expand-and-contract migration, versioned payload, canary criteria. |
| 26. Toil, Automation, And Ownership | Reliability decays without ownership. | The same manual replay happens every week. | Toil log, automation candidate, owner, review cadence. |

## Part IV Checkpoints

| Chapter | Prerequisite to recall | Mental simulation | Production evidence to name |
| --- | --- | --- | --- |
| 27. Evaluation And Behavior Reliability | Availability does not prove the agent is correct. | Change the model and compare behavior on golden and historical cases. | Golden dataset, dataset version, rubric, score, reviewer, release receipt. |
| 27.5 Agent Memory, Retrieval, And Retention | Memory can influence future runs, so it must be typed and scoped. | A model-generated note tries to become long-term policy memory. | Memory scope, kind, source, confidence, horizon, retention policy, row conversion, redacted content, and memory-by-scope query. |
| 28. Security, Abuse, And Trust Boundaries | Untrusted text must not inherit trusted authority or resource control. | A prompt-injection attempt tries to call a privileged tool and request a new egress destination. | Tool authorization, sandbox event, short-term/long-term memory policy, scoped credentials, credential lifecycle review, audit event, operation evidence, abuse test. |
| 28.5 Data Protection, Retention, And Privacy Operations | Security and memory controls expose which surfaces may contain sensitive data. | A user asks for a memory redaction while tool-call and audit evidence may also contain the same fact. | `data_protection_requests`, `data_protection_review.sql`, `DbDataProtectionReviewRow`, policy version, audit event, operation event, and completion evidence. |
| 28.6 Tenant Isolation And Multi-Tenant Agents | Security controls already separate model intent from authority. | An actor from one tenant asks a tool to read another tenant's memory or case file. | `authorization_events`, `tenant_boundary_review.sql`, `DbTenantIsolationReviewRow`, policy version, denial reason, trace id, operation event, and audit event. |
| 29. Disaster Recovery And Continuity | Backup is not recovery until restore and replay are tested. | Restore from backup while some side effects already happened. | RPO, RTO, restore drill, receipt-aware replay procedure. |
| 29.5 Extreme Fault Tolerance For Agent Systems | Recovery evidence shows what can resume after loss. | The control plane is unavailable while production workers still hold approved work. | `fault_tolerance_reviews`, last-known-good versions, redundant workers, failover drill, release gate, and `fault_tolerance_readiness.sql`. |
| 30. Reliability Maturity Model | Maturity is per job kind and evidence-backed. | Compare a summarizer with a billing agent. | Target level, current evidence, next upgrade, review date. |
| 30.5 Scaling Paths After Postgres-First | Scaling should preserve evidence, not hide the state machine. | Move one high-volume job kind from Postgres dispatch to a dedicated queue while keeping Postgres as the product ledger. | Baseline metrics, migration notes, coexistence test, trace correlation, operation event, runbook update, and rollback criteria. |
| 30.6 Temporal After Postgres-First | A workflow engine is useful only after product evidence is already explicit. | Move one approval-heavy job kind into Temporal while keeping agent runs, approvals, receipts, and audit events in Postgres. | Workflow id mapping, activity receipts, approval signal record, trace correlation, Postgres reconciliation query, and rollback path. |
| 30.7 Kafka After Postgres-First | Event streaming distributes typed facts; it does not create product truth. | Publish one outbox event to Kafka, replay it, and verify that the consumer does not repeat the side effect. | Outbox row, typed event schema, topic-partition-offset, consumer receipt, replay drill, and trace id. |

## Summary

Use these checkpoints as a friction point. If you cannot name the prerequisite,
simulate the mechanism, and point to production evidence, the chapter has not
become engineering judgment yet. Return to the chapter, then write the missing
invariant in your own words.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
