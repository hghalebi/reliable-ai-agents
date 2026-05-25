# 27.5 Agent Memory, Retrieval, And Retention

## What You Will Learn

This chapter teaches you to:

- explain how an agent remembers useful context without turning memory into hidden state;
- inspect memory scope, source, confidence, retention policy, embedding metadata, access control, and usage events;
- verify that memory can be trusted, expired, audited, and removed.

The production evidence is a typed memory record with provenance, confidence,
retention, retrieval evidence, authorization scope, and deletion behavior.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** behavior changes require versioned evaluation evidence.
- **Adds:** memory as scoped, typed, retained production data.
- **Prepares:** security and trust-boundary controls around tool-using agents.

## Production Failure

An old model-generated note is retrieved as memory and influences a new customer
case.

No one can tell who created the note, whether it was still allowed, whether it
was true, or why this run was permitted to use it.

- **What breaks:** memory became hidden authority.
- **False fix:** store fewer memories or periodically clear the vector index by
  hand.
- **Design response:** make memory typed, scoped, sourced, retained,
  confidence-scored, authorized, retrievable, deletable, and auditable.

## Motivation

In production, memory is one of the easiest agent features to demo and one of the easiest to abuse. Stored context can become stale, private, poisoned, irrelevant, or impossible to justify.

Without typed memory records and retention policy, memory becomes hidden state in the prompt. This chapter makes memory scoped, sourced, bounded, reviewable, and deletable.

## Plain Version

Read this as the simple version:

- **Simple rule:** Memory is typed, scoped, retained evidence, not a bag of strings.
- **Why it matters:** Bad memory design leaks data, retrieves stale facts, and makes agent behavior hard to audit.
- **What to watch:** Watch memory scope, source, confidence, retention policy, embedding metadata, and deletion or expiry evidence.

## What You Already Know

Start with these anchors:

- The model is not the system of record.
- Postgres stores durable state.
- Rust names domain meaning.
- Rig can use retrieved context during a model step.

This chapter adds: typed memory. You will treat memory as governed records with
scope, source, confidence, retention, retrieval evidence, authorization, and
deletion behavior.
The review question is: which remembered facts are allowed to influence which future action?

## Focus Cue

Keep three things in view:

- **State:** memory record scope, kind, source, confidence, retention policy, authority, redaction, and retrieval evidence.
- **Move:** raw context becomes memory only after metadata, retention, authority, and safety constraints are satisfied.
- **Proof:** Memory metadata, redaction policy, retention rules, retrieval authorization, and audit evidence are queryable.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** typed memory records with scope, source, confidence, retention, retrieval, and deletion policy.
- **Why it matters:** memory is unsafe when remembered text becomes authority without provenance or limits.
- **Done when:** retrieval can prove which remembered facts were eligible, why they were used, and when they expire.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/agent_memory.rs`, memory schema, retrieval SQL, retention policy, and memory tests.
- **State transition:** store, retrieve, apply, and delete memory through typed policy.
- **Evidence path:** memory influence is explainable by scope, source, confidence, retention, authorization, and retrieval event.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which remembered fact was allowed to influence this action, and why?
- **Evidence to inspect:** memory scope, source, kind, confidence, retention, authorization, retrieval event, and deletion rule.
- **Escalate if:** vector similarity alone decides authority, retention, or future influence.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** memory is considered for a future action.
2. **Action:** filter by scope, source, confidence, retention, authorization, and relevance.
3. **Persistence:** persist retrieval and retention evidence.
4. **Check:** verify remembered text influences only actions it is allowed to influence.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** memory influence is scoped, authorized, retained, and explainable.
- **Validation path:** inspect memory record, retrieval event, confidence, retention policy, authorization, and deletion rule.
- **Stop if:** similarity search alone decides authority or future influence.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, memory is one of the easiest agent features to demo and one of the easiest to abuse
rule: Memory is typed, scoped, retained evidence, not a bag of strings
tiny example: memory record scope, kind, source, confidence, retention policy, authority, redaction, and retrieval evidence
artifact: typed memory records with scope, source, confidence, retention, retrieval, and deletion policy
proof: memory influence is scoped, authorized, retained, and explainable
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Tiny Example

A support triage agent sees three cases from the same tenant:

```text
case 1: invoices are sent every Friday
case 2: billing contact prefers concise email
case 3: support agent observes that refund approvals require manager review
```

Those facts do not have the same authority.

```text
user preference:
  useful for drafting tone

case fact:
  useful only inside the tenant or case scope

tool observation:
  useful if the tool result was trusted

policy note:
  useful only if written by an operator or controlled system policy
```

A production system should not store them as anonymous strings. It should name
their kind, source, scope, confidence, and retention rule.

Read the tiny case as:

```text
setup: several remembered facts have different source, scope, and authority
transition: retrieval selects candidate context through typed memory policy
evidence: memory record, confidence, retention, authorization, retrieval event, and deletion rule exist
invariant: remembered text may inform reasoning only when policy allows it
```

## Mental Model

Think of memory as a ledger of candidate context:

```text
raw observation
  -> typed memory candidate
  -> policy and retention check
  -> stored memory record
  -> retrieval query
  -> validation before prompt use
```

The word "candidate" matters. A retrieved memory is not automatically trusted.
It is evidence that may help the next run. The next run still needs tool
authorization, sandboxing, approval gates, and policy checks.

## The Core Problem

Memory creates long-distance coupling:

```text
one run writes a memory
another run retrieves it later
the model changes behavior
the operator has to explain why
```

If memory is stored as raw text, the system cannot answer basic questions:

```text
Who created this memory?
Was it produced by a user, tool, model, operator, or system?
Which tenant, case, or run is it scoped to?
Is it short-term or long-term?
When should it expire?
How confident is the system?
Was sensitive content redacted?
Did it come from a trusted tool result or from untrusted model output?
```

Without those answers, memory becomes invisible policy.

## The Naive Solution

The naive design stores memory as a list of strings:

```text
["user likes concise replies", "refunds require approval", "case is urgent"]
```

That looks harmless. It fails because the strings have no authority model.
The first memory may be a user preference. The second may be a policy rule. The
third may be a temporary case observation. Treating them the same creates a
category error.

The production symptom is subtle: the agent appears helpful until it uses the
wrong memory in the wrong context.

## The Production-Grade Concept

Memory should be typed before it can become context.

The companion crate starts by separating short-term and long-term memory:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent_memory.rs:memory_horizon}}
```

Short-term memory is useful inside a run, session, or approval window. It can
usually expire. Long-term memory may affect future work, so it needs stronger
source, retention, review, and deletion controls.

The literal Rust type is `MemoryHorizon`; the point of the type is to make the
retention decision visible before memory can cross into agent context.

The companion crate then models a memory record as production data:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent_memory.rs:memory_record}}
```

Notice the shape. A memory record has scope, kind, source, confidence,
lifecycle policy, optional embedding reference, timestamps, and redacted
content. The important feature is not vector search. The important feature is
meaning.

## Database Boundary

Postgres stores memory in storage-friendly columns and JSON. That is acceptable
at the persistence boundary. It is not acceptable inside domain logic.

The row conversion validates raw database values before a memory can affect the
agent:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent_memory.rs:memory_row_boundary}}
```

This boundary catches unknown memory kinds, unknown horizons,
horizon/retention mismatches, invalid confidence, empty scopes, malformed
content, and empty embedding references.

The invariant is:

```text
raw memory storage outside
typed memory policy inside
```

## Retrieval As A Typed Selection

Retrieval should not mean "find similar text and trust it." Retrieval means
"select candidate memory records that are allowed to be considered for this
run."

A minimal retrieval rule includes:

```text
scope matches the current tenant, user, case, or run
retention policy allows the current use
source is acceptable for the target action
confidence meets the threshold
memory kind is relevant to the task
content is redacted or safe for prompt use
```

Vector similarity can help rank candidates. It cannot decide authority. A
memory retrieved from an embedding index still needs the same typed metadata
checks as a memory retrieved by SQL.

## Operational Query

Operators need to inspect memory metadata without dumping sensitive content.
The companion query returns memory records for one scope and deliberately omits
the raw `content` column:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/agent_memory_by_scope.sql}}
```

This query answers:

```text
Which memory records exist for this scope?
Which kinds and sources are influencing the agent?
Are they short-term or long-term?
Which records have embedding references?
When were they created or last used?
```

The omission of content is intentional. Debugging memory should start from
metadata. Content inspection should require a stronger authorization path.

## Category-Theory Lens

Start from the engineering idea: validation turns unsafe data into a safer
type.

```text
RawModelOutput -> MemoryCandidate -> MemoryRecord
MemoryRecord -> RetrievedContextCandidate -> PromptContext
```

Each arrow is a transformation with a precondition. In category-theory
language, these are composable morphisms. The useful lesson is practical:
composition is safe only when the output type of one step is the input type the
next step is allowed to trust.

The dangerous jump is:

```text
RawText -> PromptContext
```

That jump skips source, scope, confidence, retention, authorization, and
redaction. It is why raw text memory becomes production risk.

## Formal Definition

For this chapter, the precise definition is:

```text
Agent memory is typed production data with scope, kind, source, confidence, retention, and authority rules before it can influence prompts.
```

In the book's system model:

- **State:** memory record scope, kind, source, confidence, retention policy, authority, redaction, and retrieval evidence.
- **Actor:** memory writers, retrieval code, policy checks, and operators decide what can be stored, used, or expired.
- **Transition:** raw context becomes memory only after metadata, retention, authority, and safety constraints are satisfied.
- **Evidence:** Memory metadata, redaction policy, retention rules, retrieval authorization, and audit evidence are queryable.
- **Invariant:** memory remains typed, bounded, inspectable production data rather than hidden prompt state.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Memory is stored as raw strings or embeddings without source, scope, confidence, horizon, retention, or review policy. |
| Production symptom | A poisoned, stale, cross-tenant, or model-guessed memory silently changes future behavior. |
| Corrective invariant | A memory must be typed, scoped, sourced, retained, confidence-scored, redacted, and policy-checked before it can influence a run. |
| Evidence to inspect | `agent_memory_records`, row conversion tests, retention constraints, embedding reference, trace id, operation event, audit event, and the memory-by-scope runbook query. |

## Production Contract

Agent memory is production-ready only when it preserves these promises:

```text
memory scope is explicit
memory kind is explicit
source is explicit
confidence is bounded
short-term and long-term memory cannot share incompatible retention
raw content is redacted from debug output
retrieval checks authority before prompt use
operators can inspect metadata without exposing private content
deletion and retention rules are known
```

Rig may consume retrieved context. Rig does not make that context trustworthy.
The trust decision belongs to the application boundary.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Store raw strings or embeddings and append similar memories to the prompt. | Fast to prototype, but Rust has no memory authority model, SQL has weak metadata, and there is little durable evidence for tests, traces, or runbooks. |
| Safer version | Store memory records with scope, kind, source, confidence, horizon, retention, and redacted content. | The system can distinguish user preferences, case facts, tool observations, policy notes, short-term context, and long-term memory. |
| Production version | Retrieval becomes a typed policy decision backed by Postgres metadata, row conversion, retention constraints, operation events, trace id correlation, negative tests, audit controls, and runbook queries. | Memory becomes durable evidence rather than hidden prompt state. |

## Testing Strategy

Unit tests should prove the Rust constructors and enums reject invalid memory
states. Test empty scopes, unknown kinds, unknown sources, invalid confidence,
short-term memory with durable retention, long-term memory with ephemeral
retention, and debug output for redacted content.

Persistence tests should decode representative Postgres rows through
`DbAgentMemoryRecordRow -> MemoryRecord`. These tests should reject malformed
JSON content, invalid retention policy, unknown horizon, invalid embedding
reference, and out-of-range confidence.

Regression tests should recreate the failure modes: cross-tenant retrieval,
model-generated policy memory, stale long-term memory, missing source, and a
memory candidate used before authorization. The negative test should prove that
unsafe memory cannot become prompt context.

Postgres tests should prove retention constraints and metadata queries. Rust
tests should prove retrieval policy and row conversion agree with the SQL model.

## Observability Strategy

Memory reads and writes should carry a trace id, run id when available, memory
scope, memory kind, source, horizon, retention policy, and confidence bucket.
Use structured `tracing` fields for those values, but do not log raw content.

Record an operation event when memory is written, rejected, retrieved, ignored,
expired, or deleted. Record an audit event when a human, policy, or privileged
tool changes long-term memory. The runbook query should expose memory metadata
by scope without exposing sensitive text.

The operational question is:

```text
Which memory records were eligible to influence this run, and why?
```

If that cannot be answered without searching raw prompts, memory is still
hidden state.

## Security and Safety Considerations

Treat user text, model output, retrieved documents, tool observations, and
database rows as untrusted until validated. Memory writes need authorization.
Memory retrieval needs tenant and scope checks. Tools that write long-term
memory may need sandboxing, approval, or review before the memory becomes
eligible for future runs.

Redact raw content from logs, debug output, traces, and runbook output. Store
only what the product needs. Do not store secrets, credentials, access tokens,
private tool outputs, or sensitive user text simply because future prompts may
benefit.

When memory content needs redaction, erasure, export, or policy review, create
a durable `data_protection_requests` row and inspect
`data_protection_review.sql`. Do not treat memory deletion as a local cleanup
task. It changes the evidence backbone and must leave policy-versioned proof.

The safety rule is:

```text
memory can inform reasoning, but it cannot grant permission
```

## Operational Checklist

| Check | Question |
| --- | --- |
| State | Does each memory have scope, kind, source, confidence, horizon, retention, and timestamps? |
| Boundary | Are raw JSON and database rows converted into typed memory before retrieval or prompt use? |
| Failure | Can poisoning, staleness, cross-tenant retrieval, bad retention, and unsafe long-term writes be explained and stopped? |
| Observability | Can a trace id, operation event, audit event, and runbook query show which memories were eligible? |
| Safety | Do authorization, sandboxing, approval, redaction, deletion, and retention controls still apply after retrieval? |

## Exercises

1. Design an idempotency key for a memory write produced by a tool observation.
   Name the Postgres fields that prevent the same observation from becoming
   multiple long-term memories.
2. Write a Rust `MemoryCandidate` policy sketch. It should reject model-sourced
   policy notes unless a human approval or system authorization event exists.
3. Extend the memory-by-scope runbook query for your own product. Include a
   negative test for cross-tenant retrieval and keep raw content out of the
   output.

## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Why is memory more than a list of strings?
- Explain: Why is vector similarity not an authorization decision?
- Apply: Ask which remembered facts are allowed to influence which future action.
- Evidence: Name the memory scope, source, confidence, retention policy, retrieval event, authorization check, and deletion record.

## Summary

Agent memory is useful only when it is bounded. In production, memory is typed data with a source, scope, kind, confidence, retention policy, and review path.

- **Invariant:** remembered information may influence future work only when memory policy says it is eligible for that scope and horizon.
- **Evidence:** memory records, retrieval decisions, retention metadata, confidence values, access events, and deletion or expiry records show why a fact was used.
- **Carry forward:** memory is production data, not prompt decoration.

## Changed Understanding

- **Before this chapter:** memory looked like useful context stored for later prompts.
- **After this chapter:** memory is governed evidence with scope, provenance, confidence, retention, and permission boundaries.
- **Keep:** inspect memory scope, source, confidence, retention policy, retrieval reason, and deletion path.

## Further Reading and Sources



- [A-MemGuard: Agent Memory Defense](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: (2025). Emerging research on detecting and mitigating memory poisoning. It highlights why LLM-based detectors often miss 66% of malicious entries, reinforcing the need for the "Provenance and Confidence" fields used in this chapter.
- [Morris et al.: Text Embeddings Reveal Text](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: (ACL 2024). Academic research on "Embedding Inversion." It proves that 50-70% of original words can be recovered from raw vectors, justifying the chapter's rule that vector similarity is not a security boundary.
- [MINJA: Memory Injection Attacks](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: (2024). Foundational research on the 95% success rate of persistent memory poisoning attacks, providing the threat model for the "Memory Write Policy" implemented here.
- [Reflexion: Language Agents with Verbal Reinforcement Learning](./31-credible-resources-further-reading.md#agent-research-and-evaluation-papers) Read this because: (2023). Shows how reflective memory improves agent performance, while this chapter adds the production-critical "Horizon" and "Retention" controls needed to scale it safely.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann). Connects memory retrieval to the formal semantics of "Derived Data" and the maintenance of materialized views (like vector indices).