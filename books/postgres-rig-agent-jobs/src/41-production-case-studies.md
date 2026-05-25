# Appendix K. Production Case Studies

## How to Use These Case Studies

The main chapters teach mechanisms one concept at a time. The case studies
show how those mechanisms change shape across different risk profiles.

Read each case study with the same question:

```text
Which invariant stays the same, and which controls become stricter because the
job kind is riskier?
```

The system model does not change. The design principles do not change. What
changes is the strength of the evidence, approval, evaluation, and recovery
required for the job kind.

## Case Study 1: Incident Triage Agent

## Motivation

An incident-triage agent reads deployment failures, summarizes the likely
cause, and recommends the next operational action. The work is time-sensitive,
but the agent should not perform destructive action by itself.

## Tiny Incident

```text
payment deploy failed
error rate: 18%
rollback available
database CPU elevated
same incident webhook delivered twice
provider times out on first attempt
```

## System Model

```text
state:
  incident_triage job is pending

actor:
  worker owns the lease

transition:
  pending -> running -> pending(retry) -> running -> awaiting_approval

evidence:
  duplicate intake event
  provider timeout event
  retry_scheduled event
  model recommendation
  policy decision

invariant:
  no rollback recommendation becomes action without approval
```

## Controls

| Control | What it protects |
| --- | --- |
| Idempotency key | Duplicate incident webhooks map to one job. |
| Lease ownership | One worker owns the active attempt. |
| Retry classification | Provider timeout becomes scheduled future work. |
| Policy gate | Rollback recommendation is separated from rollback execution. |
| Event timeline | Operators can reconstruct what happened during the incident. |

## Production Evidence

The evidence packet should contain:

```text
job row before model call
one idempotency key for the incident
retry event for provider timeout
recommendation result with prompt/model/policy version
approval state before any rollback command
operator-facing event timeline
```

## What To Notice

The agent is allowed to compress evidence and suggest action. It is not allowed
to own the incident. Operations still own rollback, escalation, and customer
communication.

## Case Study 2: Customer Support Reply Agent

## Motivation

A support-reply agent drafts customer responses from ticket history, product
state, and policy documents. The agent can improve latency and consistency, but
it touches trust, privacy, and brand risk.

## Tiny Incident

```text
customer asks about a failed refund
ticket includes partial payment data
policy document changed yesterday
retrieval returns one stale article
agent drafts an overconfident reply
```

## System Model

```text
state:
  support_reply job is pending

actor:
  worker can draft; reviewer can approve

transition:
  pending -> running -> draft_ready -> approved -> sent

evidence:
  retrieval snapshot
  policy version
  draft text
  reviewer decision
  side-effect receipt for sent message

invariant:
  unreviewed model text never becomes a customer-visible reply
```

## Controls

| Control | What it protects |
| --- | --- |
| Retrieval snapshot | The draft can be audited against the evidence the agent saw. |
| Policy version | The reply can be tied to the rule set active at generation time. |
| Human approval | Customer-visible text has an accountable reviewer. |
| Side-effect receipt | Replay does not send duplicate replies. |
| Behavior evaluation | Prompt changes are tested against historical tickets before release. |

## Production Evidence

The evidence packet should contain:

```text
ticket payload schema version
retrieval document ids and versions
prompt/model/tool/policy versions
draft result and risk classification
approval actor, timestamp, and reason
message-send idempotency key and receipt
post-incident eval fixture for bad replies
```

## What To Notice

The support agent is not mainly a queue problem. It is a boundary and evidence
problem. The risky transition is not drafting text; the risky transition is
making text visible to the customer.

## Case Study 3: Billing Adjustment Agent

## Motivation

A billing-adjustment agent investigates failed charges, identifies likely
refund or credit actions, and prepares an adjustment request. This is a
high-risk job kind because the side effect touches money.

## Tiny Incident

```text
customer was double charged
ledger shows one settled charge and one pending charge
agent suggests issuing a credit
operator approves the wrong amount
worker crashes after calling billing API
```

## System Model

```text
state:
  billing_adjustment job is awaiting approval

actor:
  reviewer approves; side-effect worker executes

transition:
  awaiting_approval -> ready_for_side_effect -> side_effect_recorded -> succeeded

evidence:
  ledger snapshot
  proposed adjustment
  approval record
  billing API idempotency key
  side-effect receipt

invariant:
  replay can never issue a second credit for the same approved adjustment
```

## Controls

| Control | What it protects |
| --- | --- |
| Ledger snapshot | The adjustment decision is tied to financial evidence. |
| Dual control | High-risk approval can require two distinct actors. |
| Idempotency key | Billing API calls collapse duplicate execution attempts. |
| Side-effect receipt | Crash recovery can determine whether the credit happened. |
| Restore drill | Backup recovery does not duplicate money movement. |

## Production Evidence

The evidence packet should contain:

```text
financial ledger snapshot id
adjustment amount and currency as typed values
policy version and risk level
approval actor set and separation-of-duty evidence
billing API idempotency key
side-effect receipt with provider response id
restore drill proving receipt-aware replay
```

## What To Notice

This job kind needs stronger controls than incident triage or support drafts.
The model may help interpret evidence, but the production system must own
money movement, approval, idempotency, replay, and audit.

## Cross-Case Comparison

| Question | Incident triage | Support reply | Billing adjustment |
| --- | --- | --- | --- |
| Is the model output directly user-visible? | No, it informs operators. | Yes, after review. | No, it proposes financial action. |
| Is human approval required? | Required for action. | Required for customer-visible reply. | Required, often with stronger separation of duties. |
| Is a side-effect receipt required? | Required if the system executes rollback or notification. | Required when sending a reply. | Always required for billing action. |
| Is behavior evaluation required? | Required before prompt/model release. | Required with customer-history fixtures. | Required with financial-risk fixtures and human review. |
| Is restore drill evidence required? | Required for long-running operations. | Required if replies or ticket state can replay. | Required before production use. |

## Example Maturity Ladder

Do not ask only:

```text
Is this agent ready?
```

Ask:

```text
Ready for which version of the job kind?
```

The same idea can be a demo, a prototype, a production workflow, or a
regulated/high-risk workflow. The words matter because each version needs a
different evidence standard.

| Case study | Demo version | Prototype version | Production version | Regulated/high-risk version |
| --- | --- | --- | --- | --- |
| Incident triage agent | Reads a pasted incident note and drafts a summary. | Reads one incident webhook, stores a job row, and returns a recommendation to an operator. | Deduplicates webhooks, uses leases and retries, records trace/audit events, requires approval before action, and appears in runbooks. | Requires stronger change-control evidence, rollback approval, incident commander ownership, failure drills, and post-incident review before any automated action path is enabled. |
| Customer support reply agent | Drafts one reply from a pasted ticket. | Reads one ticket and one policy source, stores a draft, and asks a reviewer to approve it. | Tracks retrieval snapshot, policy version, prompt/model/tool versions, human approval, send receipt, evaluation fixture, and privacy controls. | Requires stricter privacy review, redaction or erasure handling, protected evaluation cases, reviewer separation for sensitive tickets, and audit-ready evidence for customer-visible text. |
| Billing adjustment agent | Explains a possible refund from a small example ledger. | Creates a draft adjustment request for an internal reviewer. | Stores ledger snapshot, typed amount/currency, policy version, approval evidence, idempotency key, side-effect receipt, and restore/replay proof. | Requires separation of duties, dual approval, finance/compliance policy evidence, immutable audit trail, reconciliation, and restore drills that prove replay cannot move money twice. |

Read the ladder in this order:

```text
demo version -> prototype version -> production version -> regulated/high-risk version
```

The demo proves the idea can work once. The prototype proves the team can build
the path. The production version proves users, operators, and failures are
covered. The regulated/high-risk version proves the system can survive audit,
misuse, replay, and human accountability requirements.

The ladder is not an excuse to ship weak systems. It is a way to name the
current evidence level honestly.

## Version Stop Rules

| Version | Stop if this is missing |
| --- | --- |
| Demo version | The reader cannot explain what the agent is allowed to do. |
| Prototype version | The request is not durable before model work starts. |
| Production version | The workflow has no lease, retry, idempotency, approval, receipt, evaluation, observability, security, or recovery evidence where the risk requires it. |
| Regulated/high-risk version | The workflow cannot name owners, policy version, audit evidence, separation of duties, incident procedure, and replay-safe recovery proof. |

## Production Contract

A case study is production-ready only when it can name:

```text
job kind
risk level
durable state
lease owner
provider boundary
policy boundary
approval record
side-effect receipt
evaluation receipt
runbook question
restore/replay behavior
```

If a proposed case study cannot name those artifacts, the design is still
conceptual. That is acceptable during exploration, but not enough for a
long-running production agent.

## Summary

The same principles apply to every case:

```text
durable, typed, owned, bounded, idempotent, observable, evaluated, approved,
versioned, recoverable
```

The controls become stricter as the job kind becomes riskier. A serious agent
platform should make that escalation explicit instead of treating all model
calls as the same kind of work.

## Further Reading and Sources



- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: supports the appendix's operational review habits and production-readiness vocabulary.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: keeps the appendix grounded in durable state, transactions, logs, and evidence histories.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: supports the appendix's emphasis on explicit boundaries, named types, and maintainable interfaces.
- [NIST AI Risk Management Framework 1.0](./31-credible-resources-further-reading.md#security-abuse-and-governance) Read this because: supports the ladder's distinction between ordinary production evidence and stronger high-risk governance evidence.
- [OWASP Top 10 for LLM Applications](./31-credible-resources-further-reading.md#security-abuse-and-governance) Read this because: supports the appendix's treatment of model-enabled actions as trust-boundary and abuse-risk decisions.