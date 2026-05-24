# Appendix AC. Failure-First Learning Map

## Purpose

Use this appendix when you want the book's teaching spine in one place.

The book does not start from tools. It starts from production failures. Each
chapter should help the reader move through this path:

```text
failure -> false fix -> invariant -> artifact -> proof
```

The failure gives the concept a reason to exist. The false fix shows what a
beginner may try first. The invariant names the promise the system must keep.
The artifact points to a row, type, query, test, runbook, policy, event, or
receipt. The proof tells the reader when the concept is real enough to build
on.

The enemy is the hidden magic line:

```text
agent.run("do the task")
```

That line hides user identity, tool authority, prompt and model versions,
memory scope, retry safety, side effects, approval, traceability, and audit
evidence. This appendix shows how the book replaces that line with a controlled
production system one failure at a time.

## How To Use This Map

Before reading a chapter, read its row and ask:

```text
What breaks without this idea?
Which tempting fix is too weak?
What invariant should survive?
Which artifact proves it?
```

After the chapter, answer the same questions without looking. If the artifact
or proof is vague, return to the chapter, Appendix G, Appendix AA, or Appendix
AB before moving on.

## Front Matter

| Chapter | Production failure | False fix | Invariant | Proof artifact |
| --- | --- | --- | --- | --- |
| System Model And Notation | A design review names components but cannot say what changed or who was allowed to change it. | Draw a bigger architecture diagram. | Every serious mechanism is state, actor, transition, evidence, and invariant. | A reviewer can express one job transition in the five-part notation. |
| Design Principles | The team adds controls in a random order and later discovers that retry depends on missing idempotency. | Add every reliability pattern at once. | Later controls depend on earlier invariants. | Appendix J maps each principle to a failure, artifact, and review question. |
| Production Scope And Trade-Offs | The team chooses infrastructure before naming what must be durable, replayable, or observable. | Start with the most powerful workflow platform. | Architecture choice assigns responsibility for evidence. | The operating envelope names when Postgres-first fits and when it should stop owning orchestration. |

## Part I

| Chapter | Production failure | False fix | Invariant | Proof artifact |
| --- | --- | --- | --- | --- |
| 1. The Problem | A process crashes after receiving a request, and no one can prove whether the agent had work to do. | Call the model faster and log the request. | Work exists before intelligence runs. | A durable job row and intake event exist before the model step. |
| 2. The Mental Model | Provider output, policy decisions, tool execution, and worker state blur into one agent loop. | Put more instructions in the prompt. | Model reasoning is separate from system control. | The event timeline separates state changes, model output, and policy decisions. |
| 2.5 Guarantees And Failure Semantics | The team assumes exactly-once behavior and then replays a side effect. | Promise not to crash during side effects. | Execution guarantees and side-effect guarantees are different. | Failure semantics name duplicate intake, timeout, crash, retry, and replay behavior. |
| 3. The Postgres Ledger | Operators see a stuck status but cannot tell who owns the job or what should happen next. | Add a `status` column and more logs. | The database stores work identity, ownership, due time, attempts, and history. | Ledger schema, claim query, lease fields, idempotency key, and event rows agree. |
| 4. The Rust Domain Model | `String` values for tenant, user, tool, prompt version, and model version get confused. | Add comments explaining which string is which. | If a value has production meaning, it gets a type. | Newtypes, enums, constructors, and conversion tests reject invalid values. |
| 4.5 Typed Composition Lens | A request can be sent before authentication, validation, or policy evidence exists. | Add runtime checks everywhere. | Lifecycle order is encoded where it prevents real mistakes. | Typestate builders and `Result` pipelines make invalid composition visible. |
| 5. The Worker Loop | A worker changes job state without evidence, so an incident timeline has a gap. | Add a debug log after the transition. | Every important transition leaves durable evidence. | Worker events show pick, start, success, retry, recovery, cancellation, and terminal state. |
| 6. The Rig Boundary | Malformed model output enters worker logic and looks like trusted domain state. | Tell the model to always return correct JSON. | Provider output becomes typed result or typed failure before policy sees it. | `agent_output.rs`, `tool_contract.rs`, and provider-boundary tests reject malformed output. |
| 7. Running The System Locally | A local demo returns text but proves nothing about retries, leases, or recovery. | Treat one successful run as enough. | Local execution exercises the same state machine as production. | Deterministic local tests prove enqueue, lease, retry, success, and recovery behavior. |
| 8. Production Hardening | Duplicate intake, unapproved action, stale credentials, and invisible failure arrive together. | Add hardening later after the demo works. | Hardening controls are part of the minimum serious system. | Schema, worker tests, config boundaries, approval rows, and runbook queries expose the controls. |
| 9. Failure Modes | Failures disappear into logs or endless retries. | Retry everything and alert later. | Failures become classified state transitions. | Dead jobs, retry decisions, failure history, policy stops, and terminal states are queryable. |
| 10. Capstone | A new job kind compiles but has no lifecycle, SQL, tests, or runbook. | Add one enum variant and call it done. | A production feature moves schema, states, worker behavior, tests, and operations together. | The job-kind review names every source, query, test, and operator command. |

## Part II

| Chapter | Production failure | False fix | Invariant | Proof artifact |
| --- | --- | --- | --- | --- |
| 11. The Real Postgres Store | A malformed row reaches business logic and fails far from the database boundary. | Trust rows because the migration created them. | Raw database values are decoded into domain values at the boundary. | Row conversion rejects invalid attempts, payloads, versions, leases, and terminal evidence. |
| 12. Idempotency And Side Effects | The worker crashes after an email provider accepts a send request, then retry sends it again. | Turn off retries for emails. | One logical side effect has one durable identity and receipt. | Idempotency key, outbox event, side-effect receipt, and replay test agree. |
| 13. Leases, Heartbeats, And Cancellation | Two workers complete the same job, or a user stop request leaves no durable outcome. | Use longer locks or kill the process. | Lease ownership, deadline policy, and cancellation intent are separate controls. | Lease predicates, heartbeat events, timeout query, and cancellation rows are visible. |
| 14. Retry, Backoff, And Dead Letters | Permanent failures consume capacity forever. | Add exponential backoff to every error. | Retry is a typed scheduling decision with a terminal stop path. | Attempt count, failure class, next retry time, failure history, and dead-letter reason are stored. |
| 15. Observability And SLOs | A dashboard says "healthy" while one customer job is stuck and unexplained. | Add more log lines. | Observability reconstructs behavior from correlated evidence. | Trace id, metrics, operation events, audit events, logs, and runbook queries answer the same job question. |
| 16. Human Approval And Policy Gates | The model proposes a risky action and the system treats that as permission. | Add "ask for approval" to the prompt. | Approval and escalation are durable control surfaces outside the model. | Proposal, policy version, approver, reason, escalation owner, timeline, and receipt are stored. |
| 17. Testing Production Agents | Unit tests pass while real provider output changes shape. | Mock the provider until tests are easy. | Tests cover domain logic, SQL semantics, provider contracts, behavior evals, and live smokes. | Test matrix and readiness gate map each boundary to evidence. |
| 18. Deployment And Operations | New workers cannot decode old jobs, or shutdown abandons leased work. | Deploy quickly and restart if needed. | Releases preserve old work, current leases, configuration, and operator controls. | Version fields, health checks, readiness checks, shutdown behavior, and migration evidence exist. |
| 19. Running For Years | A six-month-old job cannot be explained after prompt, model, or policy drift. | Keep old logs forever and hope they are enough. | Long-lived work carries the versions required to explain and resume it. | Schema, prompt, model, policy, worker, evaluation, and retention evidence stay with the work. |
| 20. Final Production Blueprint | During an incident, every layer blames another layer. | Add another orchestration diagram. | Each boundary owns state, transition, failure, and evidence. | API, Postgres, worker, Rig boundary, policy, side effects, and operations have explicit contracts. |
| 20.1 Agent Handoffs And Multi-Agent Coordination | One agent passes responsibility to another through chat text and no one owns the next step. | Put "handoff" in the conversation history. | A handoff is a typed, idempotent, persisted responsibility transfer. | Handoff row, source run, target agent, decision, target job, trace id, and pending-handoff query exist. |
| 20.2 Worked Production Scenario | The happy path works once, but duplicate intake plus timeout cannot be reconstructed. | Retell the scenario in prose. | Independent controls compose into one evidence chain. | Duplicate intake, retry, approval, handoff, receipt, and operator review form one traceable path. |

## Part III

| Chapter | Production failure | False fix | Invariant | Proof artifact |
| --- | --- | --- | --- | --- |
| 21. SLIs, SLOs, And Error Budgets | Teams argue about reliability using impressions instead of measured promises. | Create a dashboard without defining the measurement. | Every SLI has a reproducible source and a typed measurement boundary. | SLI query, SLO window, burn alert, owner, and error-budget decision are explicit. |
| 22. Capacity, Backpressure, And Provider Quotas | Provider rate limits, retry storms, queue age, and spend rise together. | Add more workers. | Admission control protects the system before overload multiplies. | Queue depth, oldest pending age, provider quota, tenant budget, token cost, and admission decision agree. |
| 23. Runbooks For Agent Job Systems | Operators copy risky SQL from old notes during an incident. | Write more prose advice. | Runbooks are checked operational artifacts. | Named SQL files, `psql -f` commands, pause/resume controls, and evidence notes are reviewable. |
| 24. Incident Response And Postmortems | Service recovers but the same invariant fails next month. | Close the incident when the alert clears. | Incidents turn failed invariants into system changes. | Timeline, mitigation, root cause, action item, owner, and regression evidence are recorded. |
| 25. Release Engineering For Agents | A prompt or schema release breaks old pending work. | Roll forward and fix rows manually. | Releases preserve compatibility with work already in the ledger. | Expand-contract migration, eval receipt, compatibility report, canary criteria, rollback plan, and release gate exist. |
| 26. Toil, Automation, And Ownership | Automation repeats an unsafe manual action faster than a human would. | Automate every repeated task. | Automation must have ownership, evidence, approval boundaries, and rollback. | Toil budget, owner, runbook, automation scope, approval rule, and rollback evidence exist. |

## Part IV

| Chapter | Production failure | False fix | Invariant | Proof artifact |
| --- | --- | --- | --- | --- |
| 27. Evaluation And Behavior Reliability | The service is up, but behavior drifts and unsafe answers pass through. | Trust the latest prompt because it looked better in a demo. | Behavior changes need versioned evaluation before release. | Dataset, evaluator, score, prompt version, model version, policy version, and promotion decision are stored. |
| 27.5 Agent Memory, Retrieval, And Retention | A stale or cross-tenant memory silently changes future behavior. | Store useful facts as strings and retrieve by similarity. | Memory is typed, scoped, sourced, retained, confidence-scored, and policy-checked. | Memory metadata, redaction policy, retention rule, retrieval authorization, and memory-by-scope query exist. |
| 28. Security, Abuse, And Trust Boundaries | Prompt injection causes unsafe tool use or data exposure. | Tell the model to ignore hostile instructions. | Authority comes from policy, authorization, sandbox, approval, and credential boundaries outside the model. | Authorization events, sandbox events, tool-execution gate tests, credential review, audit events, and operation events enforce the boundary. |
| 28.5 Data Protection, Retention, And Privacy Operations | A privacy request is handled in chat and only one evidence surface is changed. | Ask engineers to remember which rows were cleaned up. | Redaction, erasure, export, and retention-review requests are durable operational state. | `data_protection_requests`, `data_protection_review.sql`, policy version, completion evidence, operation event, and audit event exist. |
| 28.6 Tenant Isolation And Multi-Tenant Agents | An agent reads or acts on another tenant's data because tenant scope was just a string. | Add a prompt instruction saying never cross tenants. | Tenant scope is authorization state, not prompt text. | `authorization_events`, `tenant_boundary_review.sql`, typed tenant keys, denial reason, policy version, trace id, operation event, and audit event exist. |
| 29. Disaster Recovery And Continuity | Restore succeeds but replay duplicates external side effects. | Keep backups and assume restore is recovery. | Recovery is practiced restore plus replay safety. | Restore drill, replay decision, side-effect receipt check, RPO, RTO, and operator signoff exist. |
| 29.5 Extreme Fault Tolerance For Agent Systems | A control-plane outage stops workers that could have continued already-approved work. | Restart every service and hope the dashboard returns. | Critical execution is isolated, redundant, and able to use last-known-good state deliberately. | `fault_tolerance_reviews`, `fault_tolerance_readiness.sql`, failover drill status, release gate decision, and typed readiness row exist. |
| 30. Reliability Maturity Model | A high-risk job is called production-ready with weak evidence. | Apply one maturity label to the whole product. | Maturity is assigned per job kind from risk and evidence. | Readiness scorecard names target level, current proof, gaps, owner, and review date. |
| 30.5 Scaling Paths After Postgres-First | New infrastructure hides the state machine that made the system operable. | Add Kafka, Temporal, Redis, or Kubernetes because the system feels serious. | Scaling moves responsibility without losing evidence. | Baseline metrics, old/new evidence map, coexistence plan, trace correlation, runbook update, and rollback criteria are recorded. |
| 30.6 Temporal After Postgres-First | Workflow history says complete, but product rows cannot prove approval or side-effect receipts. | Treat Temporal history as the product audit trail. | Workflow execution history and product evidence must reconcile. | Workflow id, activity receipt, approval row, Postgres agent run, audit event, and trace id agree. |
| 30.7 Kafka After Postgres-First | Replayed stream events update projections twice or trigger duplicate side effects. | Publish raw worker JSON and let consumers handle it. | Every streamed fact has typed schema, stable identity, replay rule, and consumer receipt. | Outbox row, topic-partition-offset, schema version, consumer receipt, operation event, and trace id agree. |

## Design Review Drill

Use this when reviewing your own agent design:

```text
failure:
false fix:
invariant:
artifact:
proof:
next repair:
```

Good example:

```text
failure: worker crashes after the email provider accepts the request
false fix: disable retries for email work
invariant: one logical send has one idempotency key and one receipt
artifact: side-effect receipt plus replay decision
proof: replay refuses to send without checking the receipt
next repair: add a regression test for receipt-backed replay
```

Weak example:

```text
failure: email retry bug
proof: durable operation events and trace ids record the send attempt
```

The weak example does not name the invariant or durable artifact. It cannot
guide implementation.

## Production Contract

This appendix is complete only when the reader can turn each major concept
into:

```text
one concrete failure
one false shortcut
one surviving invariant
one production artifact
one proof sentence
```

If a chapter cannot do that, it may still contain useful information, but it is
not yet teaching production judgment.

## Summary

Failure-first teaching keeps the book serious. It prevents the reader from
memorizing vocabulary before they understand why the mechanism exists.

The transformation is the point:

```text
from "the agent ran" to "the system can prove what happened"
```

The model may guess. The system must know. This appendix makes that sentence
practical: every chapter starts from what can break, then builds the durable,
typed, observable proof that prevents the same class of failure from remaining
invisible.

## Further Reading and Sources

- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) supports the appendix's focus on turning failures into durable state, transactions, logs, and replayable evidence.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) supports the failure-first habit because SRE practice starts from user-visible reliability problems and operational evidence.
- [Google SRE chapter: Testing for Reliability](./31-credible-resources-further-reading.md#reliability-and-operations) supports the appendix's insistence that each failure maps to a test, drill, or readiness gate.
- [OWASP Top 10 for LLM Applications](./31-credible-resources-further-reading.md#security-abuse-and-governance) supports the security rows by naming agent-specific failures around prompt injection, tool use, memory, and data exposure.
- [IES: Organizing Instruction and Study to Improve Student Learning](./31-credible-resources-further-reading.md#learning-design-and-plain-language) supports the map's active recall pattern: concrete problem, explanation, application, and evidence.
