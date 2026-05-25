# Appendix L. Concept Review And Retrieval Practice

## How to Use This Appendix

The goal is active recall, not passive rereading. A production engineer has not
learned a reliability concept until they can close the chapter, reconstruct the
mechanism, apply it to a new failure, and name the evidence that would prove the
system behaved correctly.

Use this appendix after a chapter, after a part, or before a design review. The
review loop is:

```text
recall -> explain -> apply -> evidence
```

Recall asks you to retrieve the idea without looking. Explain asks you to
connect it to the system model. Apply asks you to move the idea into a new
operational situation. Evidence asks you to name the state, event, query, test,
receipt, metric, or runbook output that would prove the idea is real.

## Practice Contract

Before reopening a chapter, write four lines:

```text
recall:
explain:
apply:
evidence:
```

If one line is vague, return to the chapter and find the missing production
mechanism. Do not reward yourself for remembering vocabulary. The useful skill
is to connect a concept to a transition, a failure, and an artifact an operator
could inspect.

## Front Matter Review

| Chapter | Recall | Explain | Apply | Evidence |
| --- | --- | --- | --- | --- |
| System Model And Notation | What are the five parts of the system notation? | Why does every mechanism need state, actor, transition, evidence, and invariant? | Translate one retry or approval into the notation. | A job row, event, policy decision, receipt, or evaluation artifact proves the transition. |
| Design Principles | Which principles are the book's production rules? | Why do later controls depend on earlier principles? | Choose one weak agent design and name the violated principle. | A design review maps the principle to an artifact and failure prevented. |
| Production Scope And Trade-Offs | What is the operating envelope of the Postgres-first ledger? | Why is architecture choice about evidence ownership, not framework preference? | Decide whether a job should be a script, queue job, Postgres ledger, workflow engine workflow, or platform service. | The chosen architecture names durable state, actor authority, transition, evidence, and invariant. |

## Part I Review

| Chapter | Recall | Explain | Apply | Evidence |
| --- | --- | --- | --- | --- |
| 1. The Problem | What makes a model call different from a reliable agent job? | Why must durable state exist before the model runs? | Describe what survives if the process crashes after intake but before execution. | A job row exists before execution, and an intake event explains who created it. |
| 2. The Mental Model | Which boundaries make up the basic agent job system? | Why should the model not own workflow state? | Draw one job moving through state, worker, model, event, and policy boundaries. | The job timeline separates state changes, model output, and policy decisions. |
| 2.5 Guarantees And Failure Semantics | Which guarantees does this system provide, and which does it not provide? | Why is at-least-once execution different from exactly-once side effects? | Write the expected behavior for duplicate intake, provider timeout, crash, and replay. | Failure semantics are written down before code depends on them. |
| 3. The Postgres Ledger | Which fields make the ledger more than a task table? | How do row locks and leases cooperate? | Explain how two workers avoid claiming the same due job. | The claim query uses row locking, lease fields, and state predicates. |
| 4. The Rust Domain Model | Which raw values become domain types? | How do newtypes and enums protect production invariants? | Replace one raw string boundary in your own code with a named type and constructor. | Domain constructors reject invalid persisted or inbound values. |
| 4.5 Typed Composition Lens | What does type-state prevent? | Why is category theory useful here only as a composition lens? | Sketch a request builder that cannot be sent before auth and payload validation. | Invalid lifecycle transitions are not expressible through the public API. |
| 5. The Worker Loop | What are the worker's core steps? | Why does the worker record events around transitions? | Trace a pending job through success and through a transient provider failure. | Each transition leaves an event and a final or rescheduled state. |
| 6. The Rig Boundary | What belongs at the provider boundary? | Why should provider DTOs not leak into the worker core? | Classify timeout, malformed output, and policy refusal as domain outcomes. | Provider responses convert into typed agent results or typed failures. |
| 7. Running The System Locally | What can the deterministic local runner prove? | Why should local tests exercise the same state machine as production? | Run one local job mentally and name the state and event changes. | Local tests exercise the same domain transitions used by production stores. |
| 8. Production Hardening | Which controls turn a demonstration into a service? | Why do idempotency, leases, approval, observability, and credential lifecycle belong together? | Pick one missing hardening control and name the failure it would prevent. | Hardening controls are visible as schema, configuration, tests, and runbooks. |
| 9. Failure Modes | Which failures are dangerous because they hide state? | Why is visible dead-letter state better than a hidden retry loop? | Convert one silent failure in an agent design into durable state and operator evidence. | Dead jobs, terminal failures, and policy stops can be queried. |
| 10. Capstone | What must move together when adding a job kind? | Why are schema, states, commands, invariants, and tests one change? | Plan a new job kind and list the artifacts that must change before production. | The new job kind has typed state, SQL support, worker behavior, tests, and runbooks. |

## Part II Review

| Chapter | Recall | Explain | Apply | Evidence |
| --- | --- | --- | --- | --- |
| 11. The Real Postgres Store | Where do database rows become domain values? | Why are row conversions a production boundary? | Name one invalid persisted value and the error that should stop it. | Row-to-domain conversion fails before invalid data reaches worker logic. |
| 12. Idempotency And Side Effects | What is the identity of one logical request? | Why does retry need idempotency before side effects? | Design an idempotency key for a webhook, message send, or billing action. | Duplicate intake returns the existing job or existing side-effect receipt. |
| 13. Leases, Heartbeats, And Cancellation | What does a lease prove, what does a deadline prove, and what does a cancellation request prove? | Why is cancellation intent separate from job deletion, and why is deadline breach not the same as lease expiry? | Explain how a worker crash becomes recoverable, then explain a still-heartbeating overdue job with a pending cancellation request. | Expired leases are tied to worker ownership; breached deadlines are visible through timeout policy; cancellation requests show requested, applied, ignored, or expired stop intent. |
| 14. Retry, Backoff, And Dead Letters | Which failures retry, and which stop? | Why is retry a scheduling decision rather than a loop around a call? | Classify timeout, invalid payload, missing key, and exhausted attempts. | Retry disposition, next run time, attempt count, and dead-letter reason are stored. |
| 15. Observability And SLOs | Which signals explain one job, the API process, and the fleet? | Why must runtime health, events, metrics, traces, and SLOs agree? | Investigate an old pending job from `/metrics` to event timeline. | `/healthz`, `/readyz`, queue metrics, traces, and event rows point to the same workflow facts. |
| 16. Human Approval And Policy Gates | What is the split between model proposal and policy decision? | Why is approval durable state? | Design the approval record for a risky external action. | The model proposal, policy result, approval decision, and side-effect receipt are separate artifacts. |
| 17. Testing Production Agents | Which boundaries need tests first? | Why are provider fixtures and behavior evaluations different? | Add one regression test for a malformed provider response. | Tests cover domain logic, SQL behavior, provider contracts, and behavior evals. |
| 18. Deployment And Operations | What happens when deployment meets running work? | Why does graceful shutdown matter for leases? | Describe the safe shutdown sequence for a worker that owns a job. | Shutdown, lease expiry, versioning, and restart behavior are observable. |
| 19. Running For Years | Which things drift over long time horizons? | Why do versions make old work explainable? | Replay a one-year-old job mentally and name the versions it needs. | Jobs retain schema, prompt, model, policy, worker, and evaluation versions. |
| 20. Final Production Blueprint | Which boundary owns each failure class? | Why is the blueprint a set of contracts, not only a diagram? | Assign ownership for intake failure, lease expiry, provider timeout, and policy denial. | Every boundary has a responsibility, failure contract, and evidence source. |
| 20.1 Agent Handoffs And Multi-Agent Coordination | What is transferred during a handoff? | Why is multi-agent coordination a state-machine problem, not a chat problem? | Design a handoff from a triage agent to a specialist agent and name the target job evidence. | Handoff row, source run, target agent, idempotency key, decision evidence, trace id, and pending-handoffs query. |
| 20.2 Worked Production Scenario | Which controls cooperate in the worked scenario? | Why does the evidence chain matter more than the happy path? | Rebuild the scenario with a different job kind and keep the same invariants. | Duplicate intake, retry, approval, receipt, and operator review form one evidence chain. |

## Part III Review

| Chapter | Recall | Explain | Apply | Evidence |
| --- | --- | --- | --- | --- |
| 21. SLIs, SLOs, And Error Budgets | What does each reliability indicator measure? | Why does an SLO need both a durable query source and typed measurement boundary? | Turn job success into an SLI, SLO window, typed budget decision, and alert owner. | The SLO is backed by a repeatable query over durable state or events and validated before it drives operations. |
| 22. Capacity, Backpressure, And Provider Quotas | What limits shape throughput? | Why is overload a policy decision, not only a scaling problem? | Decide whether to admit, delay, reject, or shed one job kind during quota pressure. | Admission, queue depth, worker concurrency, and provider quota evidence agree. |
| 23. Runbooks For Agent Job Systems | What questions should runbooks answer first? | Why are runbooks production code for humans? | Write the first commands for a stuck pending job or pending cancellation incident. | Runbook commands identify queue health, lease ownership, breached deadlines, cancellation requests, dead jobs, and pause state. |
| 24. Incident Response And Postmortems | What is the incident lifecycle? | Why should incidents create system changes instead of only notes? | Turn a bad prompt incident into a timeline, mitigation, and regression item. | The postmortem links timeline evidence to action items and owner follow-up. |
| 25. Release Engineering For Agents | What can change during an agent release? | Why do old and new workers need compatibility? | Plan a prompt and schema rollout while old jobs are still pending. | Release receipts record code, schema, prompt, model, policy, and evaluation versions. |
| 26. Toil, Automation, And Ownership | What separates useful automation from unsafe automation? | Why does reliability decay without ownership? | Pick one recurring manual action and decide whether it should be automated. | Toil budgets, ownership records, automation scope, and rollback paths are explicit. |

## Part IV Review

| Chapter | Recall | Explain | Apply | Evidence |
| --- | --- | --- | --- | --- |
| 27. Evaluation And Behavior Reliability | Which behavior dimensions matter beyond availability? | Why is an evaluation receipt a production artifact? | Design a fixture set for a prompt or model change in a risky job kind. | Evaluation receipts bind behavior results to prompt, model, policy, and tool versions. |
| 27.5 Agent Memory, Retrieval, And Retention | Which metadata makes memory production data? | Why is retrieved memory a candidate context rather than trusted authority? | Design a memory record for a tool observation and decide whether it can become long-term memory or needs redaction/erasure review. | Memory scope, kind, source, confidence, horizon, retention, embedding reference, row conversion, redaction, data-protection request, and memory-by-scope query. |
| 28. Security, Abuse, And Trust Boundaries | Where do instructions lose authority? | Why must security boundaries live outside the model? | Map one prompt-injection path from input to tool authorization and one credential exposure path from detection to rotation evidence. | Tool contracts, policy checks, auth decisions, credential lifecycle review, audit events, and data-protection review show boundary enforcement. |
| 28.5 Data Protection, Retention, And Privacy Operations | Which evidence surfaces can contain sensitive data? | Why is privacy work an operational workflow instead of informal discussion? | Design a redaction request that checks memory, tool calls, audit evidence, and operation events. | Data-protection request rows, review query, policy version, completion evidence, audit event, and operation event agree. |
| 28.6 Tenant Isolation And Multi-Tenant Agents | Which facts define tenant authority? | Why is tenant scope an authorization boundary, not prompt text? | Trace a cross-tenant memory read attempt from request to denial evidence. | Actor tenant, requested tenant, permission, policy version, authorization event, trace id, audit event, and operation event agree. |
| 29. Disaster Recovery And Continuity | What is the difference between backup and recovery? | Why must replay know about side-effect receipts? | Describe restore and replay after a backup with some completed side effects. | Restore drills record RPO, RTO, replay rules, receipt handling, and operator signoff. |
| 29.5 Extreme Fault Tolerance For Agent Systems | Which parts may fail while approved work keeps serving? | Why should execution not depend on live control-plane availability? | Decide whether a job kind should continue, degrade to draft-only, or pause during a control-plane outage. | Fault-tolerance review, last-known-good versions, redundancy, failover drill, release gate, and readiness query agree. |
| 30. Reliability Maturity Model | How does maturity differ by job kind? | Why should maturity be evidence-backed rather than aspirational? | Score a summarizer and a billing agent at different maturity levels. | Each maturity level points to concrete controls, gaps, owners, and review dates. |
| 30.5 Scaling Paths After Postgres-First | What should trigger a move beyond the Postgres-first design? | Why is scaling a responsibility migration rather than a tool choice? | Decide whether one strained job kind needs worker pools, a queue, a workflow engine, an observability collector, or no infrastructure change. | Baseline metrics, old/new evidence map, coexistence plan, operation events, trace id correlation, runbook update, and rollback criteria. |
| 30.6 Temporal After Postgres-First | What should Temporal own, and what should Postgres still own? | Why is workflow history not automatically product audit evidence? | Decide whether one approval-heavy job kind needs Temporal or a simpler Postgres state transition. | Workflow id, activity receipt, approval row, agent run row, audit event, reconciliation query, and trace id agree. |
| 30.7 Kafka After Postgres-First | What should Kafka own, and what should Postgres still own? | Why is an event stream not automatically a source of product truth? | Decide whether one agent event should stay Postgres-only, publish to Kafka, or feed both paths through an outbox. | Outbox row, typed event schema, topic-partition-offset, consumer receipt, replay drill, and trace id agree. |

## What Changed In Your Mind

Use this section after a chapter summary.

The goal is simple: name how your mental model changed. If the "after" sentence
does not change how you design, test, or operate an agent, reread the chapter's
failure story and evidence path.

| Chapter | Before | After | Evidence habit |
| --- | --- | --- | --- |
| System Model And Notation | A system diagram looked like boxes and arrows. | A system diagram is state, actor, transition, evidence, and invariant. | Ask which evidence proves the transition. |
| Design Principles | Reliability sounded like a list of good ideas. | Reliability principles are ordered design constraints. | Map each principle to a row, type, query, test, or runbook. |
| Production Scope And Trade-Offs | Architecture choice looked like tool preference. | Architecture choice is about evidence ownership and operating envelope. | Name the invariant that would justify adding infrastructure. |
| 1. The Problem | A model call looked like the agent. | A durable job is the unit of reliable work. | Look for the row that exists before the model runs. |
| 2. The Mental Model | The model looked like the system owner. | The model is one worker input inside a deterministic control system. | Separate state, worker, model, policy, and product control. |
| 2.5 Guarantees | "Reliable" sounded like one broad promise. | Every guarantee has a named scope and failure semantics. | Write what retry, replay, timeout, and cancellation actually promise. |
| 3. The Postgres Ledger | A job table looked like storage. | The ledger is the first coordination layer. | Query pending, running, stuck, retried, and dead work. |
| 4. The Rust Domain Model | Strings and booleans looked harmless. | Domain values need types when confusion can break production. | Find the constructor that rejects invalid meaning. |
| 4.5 Typed Composition | Type-state looked like clever Rust. | Type-state is a way to make illegal workflow moves hard to express. | Ask which transition should not compile. |
| 5. The Worker Loop | A worker looked like code that does work. | A worker is the actor that moves durable state. | Inspect the event before and after each transition. |
| 6. The Rig Boundary | Rig looked like the reliability layer. | Rig helps the agent think and act; the application owns trust and recovery. | Check where raw provider output becomes typed domain output. |
| 7. Running The System Locally | Local demos looked separate from production. | Local runs should exercise the same state machine. | Use local tests to prove a production invariant. |
| 8. Production Hardening | Hardening looked like extra features. | Hardening is making controls durable, typed, observable, and testable. | Name the ambiguity each control removes. |
| 9. Failure Modes | Failures looked like exceptions. | Failures are states the system must expose and recover from. | Convert silent failure into a row, event, query, and owner. |
| 10. Capstone | Extending the system looked like adding code. | A production extension changes type, SQL, worker behavior, tests, and runbooks together. | Review the whole evidence chain before calling the extension done. |
| 11. The Real Postgres Store | Database rows looked trustworthy after loading. | Database rows are raw boundary data until converted into domain values. | Test that impossible rows fail conversion. |
| 12. Idempotency And Side Effects | Retry looked like running work again. | Retry is safe only when one intent maps to one durable side-effect path. | Look for the idempotency key and receipt. |
| 13. Leases, Heartbeats, And Cancellation | Ownership, deadlines, and cancellation looked like one status. | Ownership, time promises, and stop intent are separate facts. | Query owner, lease expiry, breached deadline, and cancellation request separately. |
| 14. Retry, Backoff, And Dead Letters | Retry looked automatic. | Retry is a bounded scheduling decision based on failure class and attempt budget. | Inspect attempts, next run time, failure history, and terminal reason. |
| 15. Observability And SLOs | Logs looked like observability. | Observability is the ability to reconstruct one run and judge the fleet. | Correlate trace id, events, metrics, and SLO source rows. |
| 16. Human Approval And Policy Gates | Human review looked like a UI step. | Approval is durable control state with actor, scope, reason, and evidence. | Prove who approved what before the side effect happened. |
| 17. Testing Production Agents | Tests looked like code correctness checks. | Evals, boundary tests, SQL tests, and failure drills are release controls. | Ask which test blocks the unsafe behavior. |
| 18. Deployment And Operations | Deployment looked like replacing a process. | Deployment changes a running state machine with work in flight. | Prove shutdown, leases, migrations, and readiness preserve work. |
| 19. Running For Years | Long-running meant the process stays up. | Long-running means old work remains versioned, parseable, and recoverable. | Keep schema, prompt, model, policy, worker, and eval versions. |
| 20. Final Blueprint | The MVP looked like an app plus an agent. | The serious MVP is API, worker, database, Rig boundary, and explicit controls. | Assign every failure class to an owner and evidence source. |
| 20.1 Agent Handoffs And Multi-Agent Coordination | Multi-agent looked like agents chatting. | A handoff transfers responsibility through durable state. | Prove source, target, decision, and target job evidence. |
| 20.2 Worked Production Scenario | Reliability looked like many separate controls. | Reliability is the cooperation of small controls across one evidence chain. | Rebuild the timeline from intake to review. |
| 21. SLIs, SLOs, And Error Budgets | Reliability looked subjective. | Reliability is measured against explicit promises. | Tie each SLI to a durable query and typed measurement. |
| 22. Capacity And Quotas | Overload looked like a scaling problem. | Overload is an admission-control and fairness decision. | Decide admit, delay, reject, shed, or scale from evidence. |
| 23. Runbooks For Agent Job Systems | Runbooks looked like notes. | Runbooks are production code for humans under stress. | Prefer checked queries and stop rules over memory. |
| 24. Incidents And Postmortems | Incidents looked like bad days to document. | Incidents reveal broken invariants that must change the system. | Link timeline evidence to corrective action and owner. |
| 25. Release Engineering For Agents | Release looked like shipping code. | Agent release includes prompt, model, policy, schema, worker, and eval evidence. | Require a release receipt before promotion. |
| 26. Toil And Ownership | Automation looked like removing manual work. | Automation should reduce toil without hiding judgment or ownership. | Track owner, repeated pain, automation scope, and rollback. |
| 27. Evaluation And Behavior Reliability | Model quality looked like a feeling. | Behavior reliability is measured before promotion. | Bind results to dataset, rubric, prompt, model, tool, and policy versions. |
| 27.5 Agent Memory, Retrieval, And Retention | Memory looked like useful retrieved text. | Memory is governed data with scope, source, confidence, retention, and authority. | Prove why a remembered fact may influence this run. |
| 28. Security And Trust Boundaries | The model looked able to decide safety. | Security boundaries live outside the model. | Trace every read, write, send, retain, rotate, or erase decision. |
| 28.5 Data Protection, Retention, And Privacy Operations | Privacy work looked like a private support note. | Privacy work is durable, due-dated, policy-versioned operational state. | Prove which surfaces were reviewed and what completion evidence exists. |
| 28.6 Tenant Isolation And Multi-Tenant Agents | Tenant scope looked like a prompt instruction. | Tenant scope is authorization state checked before tools or memory are used. | Prove actor tenant, requested tenant, policy version, and denial evidence. |
| 29. Disaster Recovery And Continuity | Backup looked like recovery. | Recovery is restore plus safe replay under side-effect receipts. | Practice RPO, RTO, receipt checks, and operator signoff. |
| 29.5 Extreme Fault Tolerance For Agent Systems | Fault tolerance looked like restarting everything. | Critical execution should be isolated, redundant, and able to use last-known-good state. | Prove readiness through review rows, drills, gates, and typed decoding. |
| 30. Reliability Maturity Model | Maturity looked like a label. | Maturity is evidence by job kind and risk class. | Score controls, gaps, owners, and review dates. |
| 30.5 Scaling Paths After Postgres-First | Scaling looked like adding tools. | Scaling is moving responsibility without losing evidence. | Compare old and new state, ownership, traces, runbooks, and rollback. |
| 30.6 Temporal After Postgres-First | Temporal looked like a replacement for the job table. | Temporal is optional workflow execution history that must reconcile with product evidence. | Compare workflow id, activity receipt, product rows, approvals, audit events, and trace id. |
| 30.7 Kafka After Postgres-First | Kafka looked like the next queue. | Kafka is optional event distribution that must start from typed outbox facts and end in idempotent consumer receipts. | Compare outbox id, event schema, topic-partition-offset, consumer receipt, replay behavior, and trace id. |

## Worked Review Example

Use this format when a concept feels familiar but still slippery:

```text
concept:
  idempotency before retry

recall:
  retry repeats uncertainty; idempotency prevents repeated intent from
  becoming repeated external action.

explain:
  the idempotency key maps one logical request to one durable job or one
  durable side-effect receipt. Retry may repeat execution, but it must not
  create a second logical effect.

apply:
  a duplicate webhook arrives while the first provider call is still slow.
  The system should return or observe the existing job instead of enqueuing
  a second independent action.

evidence:
  unique idempotency key, existing job returned, duplicate intake event,
  side-effect receipt checked before replay, and a runbook query that shows
  one logical request.
```

The important move is not the wording. The important move is connecting memory
to the smallest mechanism and then to production proof.

## Production Contract

A concept is learned when you can:

```text
name the failure
name the mechanism
simulate the smallest transition
point to production evidence
describe the next test or runbook query
```

If you cannot do those five things, the concept is still prose. Return to the
chapter, find the mechanism, and rehearse it against a concrete failure.

## Summary

Retrieval practice turns reading into engineering judgment. It trains the
reader to move from a production failure to a named invariant, from the
invariant to a mechanism, and from the mechanism to evidence an operator can
trust.

## Further Reading and Sources



- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: supports the appendix's operational review habits and production-readiness vocabulary.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: keeps the appendix grounded in durable state, transactions, logs, and evidence histories.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: supports the appendix's emphasis on explicit boundaries, named types, and maintainable interfaces.