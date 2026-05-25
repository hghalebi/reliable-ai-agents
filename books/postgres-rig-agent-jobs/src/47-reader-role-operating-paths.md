# Appendix Q. Reader Role Operating Paths

## How to Use These Paths

The first read should be sequential. The book is ordered as a dependency ladder:

```text
durable -> typed -> owned -> isolated -> idempotent -> observable -> versioned -> mature
```

After that first pass, different readers need different operating views. An AI
engineer will inspect provider behavior and evaluations first. A Rust engineer
will inspect type boundaries first. A platform or SRE engineer will inspect
queues, leases, SLOs, and runbooks first. A security reviewer will inspect
authority and audit evidence first. A founder or technical leader will inspect
product risk, maturity, ownership, and launch evidence first.

The shared pattern is:

```text
role -> chapters -> evidence -> practice artifact
```

Do not use a role path to skip the production model. Use it to decide which
evidence to inspect first once the model is in place.

## Shared Baseline

Every role starts with the same baseline question:

```text
Can this agent job survive process death, duplicate input, provider failure,
unsafe model output, and later review?
```

If the answer is not backed by artifacts, the system is not ready. The minimum
artifact set is:

| Control | Evidence every role should recognize |
| --- | --- |
| Durable work | `agent_jobs` or `scheduled_jobs` row exists before model execution. |
| Typed boundary | Inbound JSON, provider output, database rows, and tool payloads convert into domain types before core logic. |
| Ownership | Lease fields and owner predicates protect running work. |
| Idempotency | Duplicate intent maps to one durable job, one outbox event, or one receipt path. |
| Approval | Risky model proposals wait for durable policy and human decision evidence. |
| Observability | Trace id, operation event, audit event, metric, and runbook query point to the same transition. |
| Versioning | Prompt, model, policy, schema, tool, worker, and evaluation versions remain attached to old work. |
| Data protection | Redaction, erasure, export, and retention-review work has durable request, policy, completion, and audit evidence. |
| Recovery | Restore and replay rules know which side effects already happened. |

## AI Engineer Path

The AI engineer path focuses on behavior quality without letting model behavior
own the workflow.

| Step | Read | Inspect | Practice artifact |
| --- | --- | --- | --- |
| 1 | Chapters 1, 2, and 6 | Difference between chatbot, workflow, and reliable agent job; Rig as provider/tool interaction, not durability. | Draw the boundary between model proposal, typed parsing, policy, and durable state. |
| 2 | Chapter 17 | Provider fixtures, malformed output tests, regression cases, and simulation boundaries. | Add one fixture where the model emits a plausible but invalid tool request. |
| 3 | Chapter 27 | Golden dataset, rubric, grader, human review sample, and promotion receipt. | Build a behavior evaluation packet tied to prompt and model versions. |
| 4 | Chapters 27.5 and 28.5 | Memory scope, source, confidence, horizon, retention, retrieval eligibility, and data-protection requests. | Decide which memory records can influence a future run, which must stay out of prompt context, and which require privacy workflow evidence. |
| 5 | Chapter 28 | Prompt injection, tool injection, memory poisoning, and execution gate evidence. | Write one abuse case where model text asks for authority it does not have. |

The AI engineer path is production-ready only when behavior changes leave
evaluation receipts and untrusted model output never becomes trusted domain
state without parsing, validation, policy, and approval where risk requires it.

## Rust Engineer Path

The Rust engineer path focuses on making invalid production states hard to
construct.

| Step | Read | Inspect | Practice artifact |
| --- | --- | --- | --- |
| 1 | Chapters 4 and 4.5 | Newtypes, enums, smart constructors, typestate, and typed composition. | Replace one raw domain value with a validated newtype and negative tests. |
| 2 | Chapters 5 and 11 | Worker lifecycle, database row conversion, and store boundary errors. | Add a row conversion test that rejects malformed persisted state before worker logic sees it. |
| 3 | Chapters 12-14 | Idempotency, leases, cancellation, retry, and dead-letter transitions. | Prove a stale worker cannot complete or retry work after lease ownership changes. |
| 4 | Chapters 16, 20.1, and 28 | Approval, handoff, sandbox, and tool-execution typestate boundaries. | Encode one lifecycle so the terminal state cannot execute the same transition twice. |
| 5 | Appendix O | Companion source path from manifest to domain, SQL, store, worker, provider, binaries, and validation. | Trace one invariant from chapter text to Rust type, SQL predicate, test, and runbook query. |

The Rust engineer path is production-ready only when raw database, HTTP, and
provider values are parsed at boundaries and meaningful domain values are named
inside the system.

## Platform Or SRE Engineer Path

The platform or SRE engineer path focuses on whether the service can be run,
debugged, rolled forward, rolled back, and improved without private memory.

| Step | Read | Inspect | Practice artifact |
| --- | --- | --- | --- |
| 1 | Chapters 3, 5, 13, and 14 | Job ledger, worker loop, leases, deadlines, cancellation, retries, and dead letters. | Answer which jobs are stuck, which leases expired, and which retries are scheduled from SQL. |
| 2 | Chapters 15, 21, and 22 | Traces, metrics, logs, operation events, SLIs, SLOs, provider quotas, and backpressure. | Define one alert that names the query, threshold, owner, and first runbook command. |
| 3 | Chapters 23 and 24 | Runbook commands, incident timeline, failed invariant, mitigation, and action item. | Write an incident packet from durable evidence rather than process logs alone. |
| 4 | Chapters 25 and 26 | Release compatibility, old work, toil, automation, ownership, and rollback. | Add a release checklist item that proves old jobs remain interpretable. |
| 5 | Chapters 28.5, 29, and 30.5 | Data-protection requests, restore, replay, RPO, RTO, and evidence-preserving scaling. | Run or review a privacy/recovery/replay packet before adding new infrastructure. |

The platform or SRE engineer path is production-ready only when an on-call
engineer can reconstruct queue health, one job timeline, one risky decision,
and one recovery rule without asking the model what happened.

## Security Or Governance Reviewer Path

The security or governance reviewer path focuses on authority. The model can
recommend. Deterministic controls decide.

| Step | Read | Inspect | Practice artifact |
| --- | --- | --- | --- |
| 1 | Chapters 16, 28, and 28.6 | Approval, authorization, tenant isolation, sandboxing, scoped credentials, audit events, and tool execution gates. | Draw one trust boundary from user text to tool execution and name each decision artifact. |
| 2 | Chapter 27 | Evaluation evidence for protected failures, refusals, and high-risk behavior. | Add a protected-failure case that must not pass promotion. |
| 3 | Chapters 27.5 and 28.5 | Memory retention, scope, confidence, source, poisoning resistance, redaction, erasure, export, and retention-review workflow. | Decide which memory write needs review, redaction, rejection, or durable data-protection request evidence. |
| 4 | Chapter 29 | Restore, replay, side-effect receipts, data-protection evidence, and quarantine rules. | Identify which restored jobs must not replay automatically and which privacy requests must remain provable after restore. |
| 5 | Appendices C, N, and P | Design review, evidence packets, and design smells. | Produce a security and trust packet with owner, expiry, gap, and proof. |

The security path is production-ready only when no untrusted text can grant
permissions, select hidden credentials, choose arbitrary egress, rewrite memory,
or execute risky side effects without durable policy evidence.

## Founder Or Technical Leader Path

The founder or technical leader path focuses on whether the system is honest
about risk, maturity, cost, ownership, and launch readiness.

| Step | Read | Inspect | Practice artifact |
| --- | --- | --- | --- |
| 1 | Production Scope, Chapters 1, 2, and 8 | Operating envelope, serious MVP, and hardening controls. | Decide which job kind is allowed in the Postgres-first design and which is not. |
| 2 | Chapters 20, 20.1, and 20.2 | Final blueprint, specialist handoffs, and worked production scenario. | Explain one customer-visible workflow as state, actor, transition, evidence, and invariant. |
| 3 | Chapters 21, 22, and 27 | SLOs, capacity, provider spend, latency, and behavior evaluation. | Name the launch SLO, cost guard, and behavior gate for one job kind. |
| 4 | Chapters 25, 26, and 30 | Release safety, ownership, toil, and maturity model. | Assign owner, target maturity, current gap, next change, and review date. |
| 5 | Appendices E and N | Readiness scorecard and evidence packets. | Require a launch packet before exposing a risky agent to real users. |

The leadership path is production-ready only when the launch decision is based
on job-kind evidence, not demo confidence. A low-risk summarizer and a
money-moving agent should not have the same maturity target.

## Cross-Role Handoff

Reliable production work moves across roles. Use this handoff format when one
role finishes a review and another role takes ownership:

```text
job kind:
role completing review:
role receiving next review:
invariant checked:
evidence accepted:
gap still open:
owner:
review date:
next artifact:
```

Examples:

| From | To | Handoff question |
| --- | --- | --- |
| AI engineer | Security reviewer | Does the behavior evaluation include protected failures and tool-abuse cases? |
| Rust engineer | SRE engineer | Which type or SQL predicate proves stale workers cannot mutate this state? |
| SRE engineer | Founder or technical leader | Which SLO, cost, or incident evidence changes the launch decision? |
| Security reviewer | Rust engineer | Which trust-boundary rule needs a stronger type, constructor, or row conversion? |
| Founder or technical leader | SRE engineer | Which job kind needs a higher maturity target before more autonomy? |

This keeps review from becoming a meeting. Each role passes evidence and a
next artifact, not vague confidence.

## Production Contract

A role path is useful only if it produces artifacts. Before ending a role-based
review, write:

```text
I inspected:
I accepted:
I rejected:
The evidence is:
The next owner is:
The next artifact is:
```

If a role cannot name evidence, it has not completed its review. If it cannot
name the next owner, the system will rely on memory and enthusiasm, which are
not operational controls.

## Summary

The book has one technical argument, but readers enter it with different
responsibilities. Role paths keep those responsibilities concrete. AI engineers
protect behavior quality. Rust engineers protect type and state boundaries.
Platform and SRE engineers protect operation. Security and governance reviewers
protect authority. Founders and technical leaders protect launch judgment,
ownership, and maturity. The production system is ready only when those paths
meet on the same durable evidence.

## Further Reading and Sources



- [Anthropic: Building Effective Agents](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: supports the role split between model-directed agent behavior and deterministic workflow controls.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: grounds the role paths in durable state, evidence histories, and explicit data-system contracts.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: supports the platform, ownership, SLO, incident, toil, and launch-readiness responsibilities in the paths.
- [NIST AI Risk Management Framework 1.0](./31-credible-resources-further-reading.md#security-abuse-and-governance) Read this because: supports the governance path's focus on accountability, risk, and trustworthy AI evidence.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: supports the Rust path's focus on typed APIs, explicit constructors, and maintainable domain boundaries.