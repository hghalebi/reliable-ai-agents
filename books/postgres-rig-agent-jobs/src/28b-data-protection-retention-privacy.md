# 28.5 Data Protection, Retention, And Privacy Operations

## What You Will Learn

This chapter teaches you to:

- explain why redaction, erasure, export, and retention-review work must be durable operational state;
- inspect which agent evidence surfaces may contain user, tenant, or policy-sensitive data;
- verify that privacy work has an owner, due date, policy version, evidence, and terminal state;
- keep privacy operations separate from informal notes, chat promises, and hidden manual cleanup;
- preserve enough audit and replay evidence while reducing or removing data that should not be retained.

The production evidence is a data-protection ledger. It names the evidence
surface, subject reference, request kind, status, owner, policy version, due
date, completion evidence, and review query for every privacy request.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** memory, security, audit events, runbooks, and long-horizon retention are already explicit.
- **Adds:** privacy requests as durable operational work with typed review status.
- **Prepares:** disaster recovery and continuity, where restored data must still respect retention and privacy evidence.

## Production Failure

A support user asks the team to delete sensitive personal information that an
agent stored in long-term memory.

The request is handled in a private chat. One engineer redacts the memory row,
another forgets to check tool-call payloads, and no one records which policy
version justified keeping the audit event.

- **What breaks:** privacy work is real production work, but the system treats it as informal human memory.
- **False fix:** tell the team to be careful when deleting data.
- **Design response:** store data-protection requests as rows, review them by evidence surface, apply redaction or erasure through audited operations, and keep the minimum evidence needed for security, replay, and incident review.

## Motivation

In production, long-running agents create long-lived evidence.

That evidence is useful. It helps operators debug incidents, replay work,
prove approvals, and evaluate model behavior.

It is also risky. Agent runs, tool calls, memory records, provider usage rows,
approval reasons, audit events, and operation events may contain information
that should be redacted, exported, reviewed, or erased.

The engineering question is not:

```text
Should we keep all evidence forever?
```

The better question is:

```text
Which evidence must remain, which data must be minimized, and which privacy
request owns the decision?
```

This is not only a legal or compliance concern. It is a reliability concern.
If privacy work lives outside the system, operators cannot prove what was
requested, what was changed, what was retained, or why.

Without durable data-protection state, the system can break in four visible
ways: a request is forgotten, the wrong surface is changed, audit evidence is
destroyed, or a reviewer cannot prove why retained evidence was still needed.

## Plain Version

Read this as the simple version:

**Simple rule:** Privacy work is production work. A redaction request, an
erasure request, an export request, or a retention review is not less important
because it came from a support conversation. It changes what the system is
allowed to keep, show, replay, or use in future prompts.

**Why it matters:** If that work lives in informal notes, the team may still do
the right thing once. But later, during an incident, audit, customer escalation,
restore drill, or handoff between engineers, the system cannot prove what
happened. It cannot prove which surface was reviewed, which policy version
applied, who owned the decision, whether the request was late, or why a minimal
audit record was kept.

**What to watch:** The simple operational habit is to watch evidence surfaces,
subject references, policy versions, due dates, owners, completed requests, and
overdue reviews. The system should make those facts visible before anyone has to
search chat history.

## What You Already Know

Start with these anchors:

- Memory is governed state, not a bag of strings.
- Audit events are business evidence, not debug logs.
- Security boundaries decide what data the model may see or act on.
- Retention is a long-horizon reliability decision.
- Runbooks should answer production questions from durable evidence.

This chapter adds: privacy requests become state in the same production system.
They do not live in a private message, a spreadsheet, or an engineer's memory.

## Focus Cue

Keep three things in view:

- **State:** data-protection request, evidence surface, subject reference, request kind, status, policy version, due date, and completion evidence.
- **Move:** a privacy request becomes an approved, applied, rejected, or expired operation.
- **Proof:** `data_protection_requests`, `data_protection_review.sql`, typed row conversion, audit events, and operation events show what happened.

If you get lost, return to state, move, and proof. Then ask one question:
which evidence surface still has open privacy work?

## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a data-protection request ledger and review query.
- **Why it matters:** privacy work must be visible to operators before it becomes overdue or inconsistent.
- **Done when:** redaction, erasure, export, and retention-review requests have typed status, due dates, owner evidence, policy version, and review output by surface.

## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `data_protection_requests`, `data_protection_review.sql`, `src/data_protection.rs`, audit events, and operation events.
- **State transition:** requested or approved privacy work becomes applied, rejected, or expired with completion evidence.
- **Evidence path:** a request can be traced from subject reference to evidence surface, policy version, review status, operator action, and audit record.

## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which evidence surfaces have open or overdue redaction, erasure, export, or retention-review work?
- **Evidence to inspect:** `data_protection_review.sql`, `data_protection_requests`, audit events, operation events, policy version, due date, and completion evidence.
- **Escalate if:** a user, tenant, or policy request is tracked only in chat, support notes, a private spreadsheet, or an untyped ticket.

## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** The trigger is a request from a user, tenant, operator, privacy
reviewer, or policy owner. The request may ask for redaction, erasure, export, or
retention review. At this point, the request is still raw work. It is not yet
proof that anything should be deleted, retained, or exported.

**Action:** The first system action is to create a `data_protection_requests`
row. That row names the evidence surface, the subject reference, the owner, the
reason, the policy version, and the due date. This turns a private promise into
visible operational state.

**Persistence:** After that, every status change should go through an audited
operation. Approval, application, rejection, expiry, and completion evidence all
become part of the system history.

**Check:** The operator can then run `data_protection_review.sql` and see open,
overdue, pending-redaction, and pending-erasure work by evidence surface.

This matters because privacy work often touches more than one table. A memory row
may be redacted while a tool-call payload, audit event, or provider transcript
still contains the same sensitive content. The review query does not make the
decision for the operator. It keeps the unresolved work visible.

## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** one data-protection request can be found by evidence surface, subject reference, policy version, status, due date, and completion evidence.
- **Validation path:** create one redaction or erasure request, run the review query, apply or reject the request through an audited operation, and verify the review status changes.
- **Stop if:** privacy work is handled by deleting rows manually, editing JSON in place, or relying on informal notes instead of durable state.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: privacy work disappears into private human memory
rule: privacy requests are durable operational state
tiny example: one memory record needs redaction across memory, tool, and audit surfaces
artifact: data_protection_requests plus data_protection_review.sql
proof: open, overdue, applied, rejected, and expired requests are queryable
```

If the next section feels large, keep only these five lines in view.

## Tiny Example

A user asks the support team to remove a sensitive sentence from agent memory.

setup: the system has memory, tool-call, and audit-event evidence for one
agent run.

The naive system has only the memory row:

```text
agent_memory_records
  content: "Customer has medical condition ..."
```

The production system creates a request:

```text
data_protection_requests
  request_kind: redaction
  surface: agent_memory_records
  subject_ref: memory:mem_123
  status: requested
  policy_version: privacy-policy-2026-05
  due_at: ...
```

Then the operator asks two more questions:

```text
Did the same data appear in tool_calls?
Did audit_events keep only the minimum evidence needed to prove the action?
```

transition: the requested redaction becomes approved, applied, rejected, or
expired through the same audited operation path as other risky work.

evidence: the operator can inspect `data_protection_requests`,
`data_protection_review.sql`, the policy version, the operation event, and the
audit event.

invariant: no privacy request is complete until the affected evidence surface
and the completion proof agree.

The request does not mean "delete everything blindly." It means the system has
a durable workflow for deciding what must be redacted, erased, exported,
retained, or justified.

## Mental Model

Think of privacy operations as a work queue over evidence surfaces.

Each request says:

```text
this subject
on this evidence surface
under this policy version
needs this privacy action
by this due date
with this completion proof
```

The subject reference should be an internal id or stable reference. Do not
store raw personal data in the request row just to find the row later.

## The Core Problem

Agent systems preserve many histories:

```text
conversation history
agent run history
tool execution history
operation history
audit history
memory history
evaluation history
```

Those histories have different purposes. They also have different retention
and privacy risks.

The common mistake is using one rule for all of them:

```text
keep everything forever
```

or:

```text
delete everything when asked
```

Both are too blunt. A reliable system needs surface-specific policy.
Tool-call payloads may be redacted. Audit events may keep a minimal factual
record. Memory content may be removed from prompt influence. Evaluation
fixtures may need a de-identified copy. The system should record the decision,
not bury it.

## The Naive Solution

The naive implementation is a support runbook like this:

```text
Find the user's rows.
Delete the sensitive rows.
Tell the user it is done.
```

This hides hard questions:

```text
Which surfaces were searched?
Which policy version applied?
Was anything retained for audit or security?
Was model memory removed from retrieval?
Was the request completed before the due date?
Who approved the action?
Can we prove the answer later?
```

If the system cannot answer those questions, deletion may be fast but the
operation is not reliable.

## The Production-Grade Concept

Treat privacy requests as typed, durable, auditable work.

The request kind says what kind of work is being asked for. Redaction, erasure,
export, and retention review are different operations. A redaction may remove
sensitive content while preserving a minimal event identity. An erasure may
remove a record entirely when policy allows it. An export may require careful
packaging and authorization. A retention review may decide that evidence must
stay, but only with a documented reason.

The evidence surface says where the work applies. In an agent system, sensitive
data can appear in memory records, tool-call payloads, audit events, operation
events, provider usage rows, evaluation fixtures, traces, and support-facing
views. A request that does not name the surface is hard to operate because no one
knows where to look.

The subject reference identifies the affected data without creating a new leak.
Use an internal id, stable reference, scoped hash, or other controlled pointer.
Do not copy raw personal data into the request row merely to make the request
searchable.

The policy version explains the rule in force when the decision was made. This
matters months later, when the policy has changed and someone needs to understand
why the team retained a minimal audit event or erased a payload.

The status gives privacy work a lifecycle. A request can be requested, approved,
applied, rejected, or expired. Completion evidence records what actually
happened. At the database boundary, that evidence may be stored as a JSON object.
Inside the application, it must be parsed and treated as domain evidence, not as
a bag of trusted data.

The central invariant is:

```text
Privacy work must be queryable before, during, and after it is applied.
```

## Postgres Schema

The companion schema stores privacy work in `data_protection_requests`:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/002_agent_tracking.sql:data_protection_requests}}
```

Read the schema as a production promise:

The request kind is constrained to actions the system understands. The surface
is constrained to known evidence surfaces. The subject reference, requester,
reason, and policy version cannot be empty, because an empty value would make the
request impossible to operate later.

The evidence field must be a JSON object at the database boundary. That gives
operators room to store structured completion evidence without making raw JSON
the application architecture. Due dates and completion dates must make
chronological sense. Terminal statuses require a completion time. Active statuses
must not pretend to be complete.

This is raw outside, typed inside. SQL stores the row. Rust decides whether the
row is safe to treat as domain evidence.

## Review Query

The first operator query answers:

```text
Which surfaces have open privacy work?
```

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/data_protection_review.sql}}
```

This query does not redact or erase data. It tells the operator where work is
waiting and which work is overdue.

That distinction matters. Review is read-only. Redaction, erasure, export, and
retention decisions should be separate audited actions.

## Typed Rust Boundary

The query output is still raw database data until Rust decodes it.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/data_protection.rs:data_protection_row_boundary}}
```

The row boundary rejects unknown evidence surfaces and unknown review statuses.
It also rejects negative counts, pending redaction or erasure counts greater than
open requests, and review statuses that contradict the evidence counts.

This is the compiler and constructor doing production work. A dashboard should
not have to guess whether `privacy_review_overdue` is consistent with the row.

## Formal Definition

For this chapter, the precise definition is:

```text
Data protection for reliable agents is the durable workflow that tracks,
reviews, applies, rejects, or expires privacy-related requests across agent
evidence surfaces without losing auditability, replay safety, or policy
accountability.
```

In the book's system model:

**State:** The state is the data-protection request: evidence surface, subject reference,
request kind, status, due date, completion evidence, and policy version.

**Actor:** The actor may be a user, tenant, operator, policy owner, privacy reviewer, or
automated review job. The actor matters because privacy work changes data access,
future prompts, audit trails, and sometimes legal obligations.

**Transition:** The transition is the controlled move from requested work to approved, applied,
rejected, or expired work. That transition should happen through audited
operations, not through manual SQL edits.

**Evidence:** The evidence is the request row, review query, audit event,
operation event, policy version, and completion evidence.

**Invariant:** The invariant is that privacy work is never only an informal note,
and sensitive data is not retained, redacted, erased, or exported without durable
evidence.

## What Can Fail

**Design smell:** The common design smell is privacy work handled in support chat, private
spreadsheets, or manual SQL edits. This usually starts with good intent. The team
wants to respond quickly. But speed without state creates a later evidence gap.

**Production symptom:** The production symptom appears when a user, auditor, incident commander, or
operator asks what was removed. The team may remember doing something, but it
cannot prove which surfaces were reviewed or why some audit evidence was
retained.

**Corrective invariant:** The corrective invariant is that redaction, erasure,
export, and retention-review requests have durable lifecycle state and audited
completion evidence.

**Evidence to inspect:** The evidence to inspect is `data_protection_requests`,
`data_protection_review.sql`, audit events, operation events, policy version,
due date, and typed row-conversion tests.

## Production Contract

The contract is:

```text
privacy request -> typed request row -> reviewed surface -> audited action -> completion evidence
```

Do not let privacy work bypass the same standards used elsewhere in the book.

The request should name:

```text
request kind
evidence surface
subject reference
requester or operator
reason
policy version
due date
status
completion evidence
```

The action should preserve:

```text
minimum audit evidence
side-effect and replay safety
operator accountability
policy explanation
traceability to the request
```

## Progressive Hardening Path

**Naive version:** The naive version is manual cleanup after a support message. Engineers delete or
edit rows directly. The response may be fast, but there is no durable proof of
scope, policy, due date, or completion.

**Safer version:** The safer version stores privacy requests in Postgres and reviews open work by
surface. Operators can see overdue, redaction, erasure, export, and
retention-review work before it disappears into private human memory.

**Production version:** The production version applies privacy actions through audited operations. It
records policy version, trace id, completion evidence, retention review, and
row-conversion tests. At that point, privacy work becomes a reliable production
workflow.

## Testing Strategy

Unit and Rust type tests should prove that unknown surfaces, unknown statuses,
negative counts, and impossible count totals fail row conversion. This is the
first line of defense against a misleading dashboard or operator report.

Persistence or Postgres boundary tests should prove that
`data_protection_review.sql` exposes open, overdue, pending-redaction, and
pending-erasure work by surface. The query is an operator tool, so the test
should protect the operator question it answers.

Regression tests should simulate a redaction request for memory content and
prove the request remains visible until an audited completion action changes its
status. An operational drill should create one overdue request in a staging
database, run the review query, apply a safe correction, and record the audit
event.

Tests should not prove that deletion happened by accident. They should prove
that the privacy operation followed the request lifecycle.

## Observability Strategy

Emit structured `tracing` fields for request id, request kind, evidence
surface, subject reference class, policy version, status, due date, actor, and
trace id.

Record operation events for:

```text
data_protection_requested
data_protection_approved
data_protection_applied
data_protection_rejected
data_protection_expired
```

Use metrics for open requests, overdue requests, pending redactions, pending
erasures, and applied requests in the last 30 days. Keep audit events separate
from operational logs.

The first runbook query is `data_protection_review.sql`. It should answer the
on-call question before anyone searches an informal note: which surfaces have open
or overdue privacy work?

## Security and Safety Considerations

Treat privacy requests as untrusted input until authorization, requester
identity, policy version, and evidence surface have been checked.

Do not put raw sensitive content in the privacy request just to identify what
needs work. Use internal ids, stable references, scoped hashes, or metadata
that lets an authorized operator find the data without creating new leakage.

Do not let the model decide whether data should be retained or erased. The
model may help find candidate surfaces, but policy, authorization, reviewer
identity, and audited state decide the action.

Use sandboxing for any automated scanner that inspects raw payloads, memory, or
provider transcripts. A scanner should find candidate data; it should not gain
permission to export, erase, or modify production evidence by itself.

Require approval for high-impact erasure, export, or retention exceptions.
Redact sensitive values from logs and traces while keeping stable ids, policy
versions, and completion receipts.

Do not erase audit evidence blindly. Some systems need a minimal factual record
that an action happened, who approved it, and why. The safer pattern is often:

```text
redact content
retain minimal event identity
record policy reason
link to completion evidence
```

Jurisdiction-specific data-protection rules vary. The engineering pattern in
this chapter is not legal advice. It is the control surface a serious system
needs so legal, compliance, security, and product reviewers have evidence to
work with.

## Operational Checklist

Use the checklist as a short design review, not as paperwork.

First, inspect **State**. Which privacy requests are requested, approved, applied,
rejected, expired, or overdue?

Next, inspect the **Boundary**. Which request fields are raw database values, and
where are they converted into domain types?

Then inspect **Failure** behavior. What happens if a request is overdue, partially
applied, applied to one surface but missed another, or conflicts with audit
retention?

Inspect **Observability** next. One trace should connect request creation, review,
action, audit event, operation event, and completion evidence.

Finally, inspect **Safety**. The action should minimize sensitive data without
destroying the evidence needed for replay, security, incident review, and
accountability.

## Exercises

1. Pick one evidence surface from the companion schema and decide whether it can contain personal, tenant-sensitive, or policy-sensitive data. Name the retention and redaction risk.
2. Design a data-protection request for a tool-call payload that must be redacted but still needs a minimal audit trail. Name the request kind, surface, subject reference, idempotency key, policy version, and completion evidence.
3. Write a negative Rust test plan for `DbDataProtectionReviewRow` where `pending_erasure_requests` is greater than `open_requests`. Explain which production mistake the negative test prevents.
4. Sketch the Postgres row and status transition for an export request that is approved, completed, and later inspected during an incident review.

## Self-Check

Use this quick retrieval drill before moving on:

Recall: Why is privacy work operational state?

Explain: Why can deletion without evidence make the system less trustworthy?

Apply: Decide whether one request should redact, erase, export, or trigger
retention review.

Evidence: Name the request row, review query, policy version, audit event,
operation event, and completion evidence.

## Summary

Data protection is part of reliable agent operations. A long-running agent
system must know which evidence it keeps, which content it redacts, which data
it erases, which exports it owes, and which retention decisions need review.

Invariant: privacy work is durable, typed, reviewable, and audited.

Evidence: `data_protection_requests`, `data_protection_review.sql`,
row-conversion tests, policy versions, operation events, and audit events prove
the workflow.

Carry this forward: never move privacy work into informal notes just because the
data is sensitive.

## Changed Understanding

**Before this chapter:** privacy may have looked like cleanup after the agent
stored something sensitive.

**After this chapter:** privacy should look like a production workflow over
evidence surfaces, with typed state and auditable completion.

**Keep:** minimize sensitive data without destroying the evidence needed to
operate, audit, and recover the system.

## Further Reading and Sources



- [Ann Cavoukian: Privacy by Design](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: The globally recognized standard for embedding privacy into the architecture of automated systems. It provides the "Positive-Sum" philosophy used to balance auditability with data minimization.
- [EDPB: Data Protection by Design](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: The regulatory and technical standard for implementing the durable data-protection workqueue described in this chapter.
- [ICO: Storage Limitation](./31-credible-resources-further-reading.md#security-abuse-and-governance) Read this because: Practical industry guidance on how to preserve enough evidence for system reliability while redacting or erasing what is no longer needed.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann, Chapter 11: Stream Processing). Explains the formal mechanics of "Compacting" and "Deleting" data in distributed ledgers while maintaining a verifiable history.