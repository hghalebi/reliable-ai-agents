# 4. The Rust Domain Model

## What You Will Learn

This chapter teaches you to:

- explain why meaningful production values deserve Rust types;
- inspect where raw strings, booleans, numbers, vectors, or JSON would cross a domain boundary;
- verify that constructors, enums, and errors reject invalid states early.

The production evidence is a domain model where ids, statuses, retry counts,
prompt versions, model versions, approvals, tool inputs, and receipts have
explicit types.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the ledger contains meaningful values, not anonymous data.
- **Adds:** newtypes, enums, constructors, and explicit domain errors.
- **Prepares:** typed composition and typestate for legal lifecycle movement.

## Production Failure

A release stores `deepseek-chat` where the system expected a worker id.

Both values were plain strings, so the compiler accepted the mistake:

```text
model_version = "worker-a"
locked_by = "deepseek-chat"
```

- **What breaks:** the database contains text, but the application lost
  meaning before the row was written.
- **False fix:** rename variables and ask reviewers to be careful.
- **Design response:** convert raw boundary values into newtypes, enums, and
  validated constructors before core logic can use them.

## Motivation

In production, raw strings and numbers hide category errors. A prompt version can be confused with a model version, a retry count with a max attempt limit, or a tool name with untrusted model text.

Without explicit domain types, the codebase lets invalid architecture compile. This chapter teaches Rust newtypes, enums, and constructors as reliability tools around a probabilistic model core.

## Plain Version

Read this as the simple version:

- **Simple rule:** If a value has production meaning, give it a Rust type instead of passing a raw primitive around.
- **Why it matters:** Types prevent category errors before they become retries, wrong tool calls, or bad audit records.
- **What to watch:** Watch for raw strings, booleans, integers, JSON values, and vectors crossing domain boundaries without validation.

## What You Already Know

Start with these anchors:

- The ledger contains job kinds, states, worker ids, attempts, versions, and retry decisions.
- Those fields have different meanings even when storage uses strings or numbers.
- Invalid domain values should be rejected near the boundary.

This chapter adds: Rust domain types. Newtypes, enums, constructors, and typed
errors make category mistakes harder to write and easier to test.

## Focus Cue

Keep three things in view:

- **State:** typed Rust values that represent job identity, worker ownership, model versions, retry decisions, failures, and results.
- **Move:** raw storage, HTTP, provider, CLI, or environment data becomes a validated domain value at the boundary.
- **Proof:** Newtypes, enums, smart constructors, conversion tests, and typed errors reject invalid or confused values.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a Rust domain module with newtypes, enums, validated constructors, and typed errors.
- **Why it matters:** raw strings and booleans let production concepts get confused at the worst boundary.
- **Done when:** IDs, statuses, retry counts, model versions, prompt versions, and decisions cannot be mixed accidentally.


## Implementation Map

When you transition from reading about these types to actual implementation, rely on this map as your guide. The primary surfaces you will interact with are `src/domain.rs`, along with the fierce row-conversion modules such as `src/agent_run.rs` and `src/tool_call.rs`. The core state transition here is ruthlessly converting raw, untrusted boundary data into strict newtypes, explicit enums, and validated domain objects. The evidence path mathematically guarantees that any invalid IDs, rogue statuses, impossible attempt counts, hallucinated versions, and unapproved decisions are violently rejected long before the core business logic is ever exposed to them.
Use this map when you move from reading to implementation:
- **Primary surface:** the code, schema, chapter artifact, or runbook that owns this concept.
- **State transition:** the named move that changes durable state.
- **Evidence path:** the row, event, receipt, trace, or test that proves the move.


## Operator Question

Before you ship any architectural idea based on this domain model, you must answer one vital operational question: Which raw, primitive value currently in the codebase would instantly become dangerous if its meaning were accidentally confused with another? To answer this, you must explicitly inspect the newtype constructors, the enum variants, the typed errors, the row conversion tests, and the logs of specifically rejected invalid values. You should immediately escalate the design to leadership if any critical domain boundary still lazily accepts a raw string, a naked boolean, a raw integer count, an untyped JSON value, or a generic, unhelpful error string.
Before you ship this idea, answer one operational question:
- **Question:** what production fact changed, and who was allowed to change it?
- **Evidence to inspect:** the durable row, trace, receipt, policy decision, or audit event.
- **Escalate if:** the answer depends on memory, chat, terminal scrollback, or model explanation.


## Runtime Walkthrough

Follow the concept of typed boundaries as a single runtime pass. First, a trigger occurs when raw, highly suspicious boundary data enters the Rust application from the database or the network. Next, the action requires the system to forcefully parse that data into strict newtypes, enums, and validated constructors. For persistence, the boundary must definitively return either perfectly typed domain values or explicitly typed validation errors. Finally, the check requires verifying that no important business concept is ever carried deeper into the system as a loose, raw primitive.
Follow the concept as one runtime pass:
1. **Trigger:** the system receives work or a reviewer inspects a design.
2. **Action:** the mechanism changes typed state under an explicit owner.
3. **Persistence:** the change leaves durable evidence.
4. **Check:** an operator verifies the invariant from evidence.


## Acceptance Gate

Do not move on until you can produce the minimum required evidence. You must be able to empirically prove that all important domain concepts are strictly typed *before* the business logic ever sees them. To validate this path, an operator must inspect the constructors, enums, typed errors, and the rigorous row-conversion tests. Stop the design process immediately if a meaningful ID, status, version, retry count, policy decision, or external payload is caught crossing a boundary while disguised as a raw primitive.
Do not move on until this minimum evidence exists:
- **Minimum evidence:** the mechanism has one inspectable artifact and one named invariant.
- **Validation path:** run or inspect the smallest check that proves the artifact exists.
- **Stop if:** the proof depends on a unverified note, chat message, or unverified assumption.


## Micro-Lesson

If you need a concise summary before diving into the heavier mechanisms, remember this sequence: The pain arises because, in production, raw strings and numbers perfectly hide disastrous category errors. The guiding rule is that if a value has genuine production meaning, you must give it a strict Rust type instead of casually passing a raw primitive around. A tiny example of this is seeing typed Rust values explicitly represent job identity, worker ownership, model versions, retry decisions, failures, and results.

The resulting artifact is a highly paranoid Rust domain module packed with newtypes, enums, validated constructors, and typed errors. The ultimate proof of success is that important domain concepts are mathematically typed long before the business logic is ever allowed to touch them. Use this five-line version before the heavier mechanism:

```text
pain: the production failure becomes unclear without this concept
rule: name the invariant and the evidence before adding machinery
tiny example: one job changes state under one owner
artifact: one row, type, receipt, policy, or runbook query
proof: another engineer can inspect the artifact and explain the result
```
If the next section feels large, keep only these five lines in view and then return to the detailed proof.


## Worked Walkthrough

Imagine two identifiers arrive from different places.
One is a user id from the HTTP request.
The other is an agent run id from the database.
Both may be UUID-shaped strings at the edge.

If the application passes both around as `String`, the compiler cannot help.
A function can receive the run id where it expected the user id.
The code may still compile, the database query may still run, and the bug may become visible only as a wrong audit trail.

The domain model fixes this by giving meaning to values after they cross the boundary.
Raw text becomes `UserId`.
Raw text becomes `AgentRunId`.
A retry integer becomes `RetryCount`.
A status string becomes an enum.

The point is not to make Rust look clever.
The point is to move production rules from memory into code.
If a value has a role in reliability, authorization, recovery, audit, or cost control, the role deserves a type.

The database row can still use storage-friendly fields.
The HTTP body can still be JSON.
The provider response can still arrive as text.
But inside the system, those raw shapes should be parsed, validated, and converted before business logic sees them.

This is the raw-outside, typed-inside rule.
It is one of the simplest ways to make an agent system less fragile.

The rule is especially important for AI systems because one part of the system
is allowed to generate plausible text. The model may produce a tool name,
action proposal, explanation, recipient, risk label, or JSON object. None of
that text is domain truth just because it looks structured.

The job of the Rust domain model is to force generated text to cross the same
boundary as any other untrusted input. It must be parsed, validated, authorized,
and converted before it can affect state or side effects.

This typed boundary is your best defense against **Prompt Injection**. If a model
is tricked into proposing an "Admin" tool call, the Rust type system acts as
your first wall—rejecting any action that doesn't fit the strictly defined
`ToolName` enum before it can touch your infrastructure.

This is why types are not style in this book. They are part of the safety
system around the model.

> ### 🎓 The Professor's Corner > > **The NewType Pattern: No More "Naked Strings"** > > In many languages, you'd just use a "Naked String" for everything—names, IDs, emails, and phone numbers. But if everything is just text, you can accidentally use your phone number as your bank account balance, and the computer won't stop you! > > The **NewType Pattern** is like putting a specific wrapper around a value.

By creating a `WorkerId` type, we tell Rust: "This isn't just a string; it's a very specific kind of label." Now, if you try to put a phone number where a `WorkerId` belongs, the compiler will tap you on the shoulder and say, "Hey! That doesn't fit here!" It takes the stress away and lets you focus on the logic.

## The Core Types

The code below is included from the executable example crate:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/domain.rs:types}}
```

## Why This Matters

This is better than passing `String` everywhere:

```text
fn run_agent(instruction: String) -> String
```

The typed version tells the reader what each value means:

```text
AgentInstruction -> AgentResult
```

The constructor also rejects empty instructions before the worker starts.

That small rejection matters.

If an empty instruction reaches the model boundary, the failure may appear as a
bad response, a confusing evaluation result, or an operator question later in
the run. If the constructor rejects it at intake, the failure has a clear owner:
the boundary refused invalid work.

Good domain types shorten the distance between cause and evidence. They make
the system fail while the mistake is still local.

## Tiny Example

With raw strings, this mistaken assignment has no type-level signal:

```text
let worker_id = "deepseek-chat".to_string();
let model_name = "worker-a".to_string();
```

Both values are `String`, so the compiler cannot tell that the model route and
lease owner were swapped.

With named types, the mistake becomes visible at the boundary:

```text
let worker_id = WorkerId::new("worker-a")?;
let model_route = ModelRoute::new("deepseek-chat")?;
```

The goal is not aesthetic wrapping. The goal is to make operationally dangerous
mix-ups harder to express.

Read the tiny case as:

```text
setup: model name and worker id are both stored as text at the edge
transition: boundary conversion turns them into different domain types
evidence: constructors and negative tests reject swapped or empty values
invariant: two values with different roles cannot be silently confused
```

The mistake in this example is not exotic. It is the kind of bug that appears
when systems grow quickly. A field is copied from one struct to another. A test
fixture uses the wrong string. A migration maps a column into the wrong DTO
field. A model route and a worker id both look like short labels, so a review
does not catch the swap.

Newtypes do not remove every bug. They remove an entire class of silent swaps.
The compiler becomes a reviewer for meaning, not only for syntax.

## The Production Rule

In a production agent system, a raw string is rarely just a string. It may be a
provider model name, a worker identity, a policy decision, a user instruction,
a cancellation reason, or an idempotency key. If all of those are plain
`String`, a caller can accidentally pass the wrong value to the wrong place and
the compiler cannot help.

The companion crate uses small newtypes for values that cross boundaries:

```text
IdempotencyKey  -> protects external side effects
WorkerId        -> owns leases and heartbeats
AgentInstruction -> enters the model boundary
AgentResult     -> leaves the model boundary
PromptVersion   -> explains which prompt produced old work
ModelRoute      -> explains which model family processed old work
```

The lesson is not "wrap everything." The lesson is:

```text
If confusion would cause production damage, give the value a type.
```

This rule keeps the book from becoming type ceremony.

The type exists because a wrong value would hurt reliability, security,
operability, or auditability. `PromptVersion` tells an operator why old behavior
changed. `ModelRoute` explains which provider path was used. `IdempotencyKey`
prevents duplicate logical work. `WorkerId` protects lease ownership.

These are not wrappers around data. They are names for production
responsibilities.

## Judgment: Do Not Type Everything

Newtypes are not a decoration layer.

Use a newtype when the value has a role that can break production:

```text
crosses a boundary
has a validation rule
affects workflow state
affects security or tenant scope
appears in audit or operation evidence
can be confused with another value of the same primitive type
```

Do not add a newtype for every local variable.

These values can stay raw when they are private, short-lived, and mechanical:

```text
loop index
temporary string buffer
local counter
test-only fixture label
formatting helper
```

The judgment question is:

```text
If this value is wrong, can it create the wrong state, wrong side effect,
wrong tenant boundary, wrong audit evidence, or wrong recovery decision?
```

If yes, give it a type. If no, keep the code simple.

This judgment matters because over-typing can make a codebase harder to learn.
The goal is not to prove that every byte has a custom wrapper. The goal is to
protect meaning where meaning affects production behavior.

Use types where they reduce ambiguity. Avoid them where they only add ceremony.
That balance is what makes type-driven design useful for teams, not just for
individual Rust enthusiasts.

## Raw Data Types Are Forbidden At Boundaries

For the architecture described in this book, raw domain primitives are strictly forbidden from crossing architectural boundaries. 

Do not expose a function signature that takes a raw job id, a raw error string, and a raw boolean flag for permanency. Instead, expose a signature that strictly demands a typed `JobId`, a typed `FailureMessage`, and a typed `RetryDisposition`. 

Raw strings, unstructured JSON values, generic booleans, simple integers, and database text are permitted *only* deep inside private adapters that immediately validate and mathematically convert them. A database row DTO must become a typed domain model. A command-line argument must become an `AgentInstruction`. An environment variable must become a `DatabaseUrl` or a strict provider client config. Raw provider text must become an `AgentSummary`. Loose release metadata must become a `PromptVersion`, `ModelRoute`, or `PolicyVersion`.

This rule is unforgiving because this system is explicitly designed to run for years. A raw `String` innocently called `message` eventually, inevitably becomes a secret leak, a mathematically malformed policy decision, or a completely unsearchable operational trace at 3 AM.

The most important word here is boundaries. Raw values are tolerated at the absolute edge of the system simply because the outside world speaks in messy HTTP bodies, flat database rows, loose environment variables, unpredictable provider responses, and raw command-line arguments. The operational nightmare begins only when those raw values are allowed to keep traveling inward. At the boundary, the system must make a brutal, immediate decision: either aggressively accept and convert the data into a strict domain type, or violently reject it with a typed error. There must never be a third path where the raw value quietly sneaks in and becomes part of your business logic.

## State Is A Contract

The Rust `JobStatus` enum must perfectly mirror the Postgres database check constraint. That aggressive symmetry is not a coincidence; it is an architectural mandate. A worker should never be able to magically invent a new status locally that the database cannot physically store, and the database should never accept a rogue status that the Rust domain cannot possibly understand.

The exact same uncompromising idea applies to event types. Events like `job_enqueued`, `duplicate_suppressed`, `job_picked`, `agent_started`, `agent_succeeded`, `agent_failed`, `retry_scheduled`, `job_succeeded`, `job_dead`, `lease_extended`, `job_cancelled`, and `expired_lease_recovered` are not just strings; they are the formal operational vocabulary of the entire system. When an incident inevitably happens, the event stream must read like a crisp, undeniable timeline, not like a desperate pile of randomly generated log strings.

State names also actively teach the rest of the engineering team exactly how the system works. If an enum explicitly lists variants like `Pending`, `Running`, `Succeeded`, `Dead`, and `Cancelled`, then code review can effectively ask whether each transition is actually legal. Conversely, if the database lazily stores arbitrary status text, every single operational query becomes a terrified guess about spelling and meaning. The same exact idea applies to business decisions. A simple boolean called `approved` mathematically cannot explain the difference between `approved`, `rejected`, `expired`, `withdrawn`, or `requires_more_context`. An explicit enum can.

## Formal Definition

For this chapter, the precise definition is:

```text
A domain model is the typed interior representation that gives meaning, validation, and lifecycle rules to values crossing raw boundaries.
```

In the book's system model:

- **State:** typed Rust values that represent job identity, worker ownership, model versions, retry decisions, failures, and results.
- **Actor:** constructors, conversion layers, and domain APIs accept or reject values before business logic receives them.
- **Transition:** raw storage, HTTP, provider, CLI, or environment data becomes a validated domain value at the boundary.
- **Evidence:** Newtypes, enums, smart constructors, conversion tests, and typed errors reject invalid or confused values.
- **Invariant:** values with different meanings cannot silently masquerade as the same raw primitive inside the system.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Raw primitives cross domain boundaries. |
| Production symptom | Values with different meanings are swapped, accepted empty, or persisted invalidly. |
| Corrective invariant | Meaningful domain values are named, validated, and converted at boundaries. |
| Evidence to inspect | Newtypes, enums, smart constructors, and conversion tests reject invalid values. |

## Production Contract

Domain code should receive named values:

```text
validated DTO -> domain type -> worker/store trait
```

Raw database text, provider JSON, command-line strings, and environment
variables are allowed at adapters only long enough to validate them. Secrets
must have redacted debug output. Status-like values must be enums or
constrained database values, not loose strings.


## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Raw primitives cross domain boundaries. | Raw primitives let unrelated production values be accidentally swapped while still compiling. |
| Safer version | Meaningful domain values are named, validated, and converted at boundaries. | Newtypes, enums, constructors, and typed errors move meaning from comments into the compiler boundary. |
| Production version | Newtypes, enums, smart constructors, and conversion tests reject invalid values. | Invalid IDs, versions, retry counts, and provider outputs fail at construction or conversion before core logic runs. |

Use the naive row to recognize primitive obsession. Use the safer row to name production meaning. Use the production row when invalid data must be rejected before it becomes state.

## Testing Strategy

Test the domain model where raw values enter:

- **Unit or type test:** prove Rust newtypes reject empty identifiers, invalid versions, out-of-range retry counts, malformed trace ids, and confused domain concepts.
- **Persistence or boundary test:** prove Postgres row conversion turns storage-friendly strings and integers into typed domain values before business logic sees them.
- **Regression test:** encode a category error, such as passing a model version where a prompt version belongs, so the compiler or constructor blocks it.

## Observability Strategy

Observe typed boundaries by logging conversion outcomes, not raw payloads:

- Emit structured `tracing` fields for boundary name, domain type, validation result, trace id, job id when available, and redaction decision.
- Record an operation event when raw HTTP, provider, CLI, environment, or database values fail conversion into typed domain values.
- The runbook query should show which typed boundary rejected invalid data without exposing secrets or raw model content.

## Security and Safety Considerations

Typed domain values are the first line of defense against confused authority:

- Treat raw strings, integers, booleans, JSON, and provider DTOs as untrusted until smart constructors or conversions create domain types.
- authorization, sandboxing, and approval should use explicit types so tenant scope, permission, tool name, and decision cannot be confused.
- Redact secret-bearing newtypes in `Debug` output while preserving safe identifiers, versions, and validation errors.

## Operational Checklist

Use this checklist before relying on newtype-based domain modeling in production:

- **State:** Important concepts such as job id, model version, prompt version, retry
  count, and tool name have distinct types.
- **Boundary:** Raw strings, integers, booleans, JSON, and database rows are converted
  at the edge before domain logic runs.
- **Failure:** A category error such as mixing PromptVersion and ModelVersion becomes a
  compile or constructor error.
- **Observability:** Typed values expose safe labels for traces and events without
  leaking secrets or raw payloads.
- **Safety:** Secret and user-controlled types use validating constructors and redacted
  Debug implementations.

## Exercises

1. Write a negative test where a raw string model version is accidentally passed where a
   prompt version or idempotency key is required. Explain which idempotency key,
   receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: a row conversion test that rejects invalid retry
   counts, unknown status strings, and non-object payloads.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   AgentId, AgentRunId, ToolName, PromptVersion, ModelVersion, RetryCount, and
   ApprovalDecision. Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Name three raw values that become safer as newtypes or enums.
- Explain: Why are `PromptVersion` and `ModelVersion` different concepts even if both store text?
- Apply: Pick one raw field and write the invariant its constructor should enforce.
- Evidence: Name the Rust type, validation error, boundary conversion, and negative test that prove the invariant.

## Summary

The database may store generic values, but the Rust boundary should recover meaning immediately. Types are the first safety system around a probabilistic core.

- **Invariant:** important domain concepts are not passed around as raw strings, booleans, integers, maps, or unvalidated JSON.
- **Evidence:** constructors, enums, conversion tests, redacted Debug implementations, and row decoders reject invalid states.
- **Carry forward:** if two values mean different things in production, they deserve different types.

## Changed Understanding

- **Before this chapter:** strings and JSON looked acceptable once they came from trusted code.
- **After this chapter:** domain types carry meaning; raw data enters at boundaries and typed values move through the system.
- **Keep:** look for newtypes, enums, constructors, and row-to-domain conversions at every boundary.

## Further Reading and Sources



- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: (2019). The foundational text for type-driven design. It explains why a system is safer when it transforms unstructured input (like a database string) into a structured type (like a `WorkerId`) rather than just checking a boolean property.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: While written for F#, this book is the industry standard for using Algebraic Data Types (ADTs) to make illegal states unrepresentable—a core principle of the Rust domain model in this chapter.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: The official community standard for when and how to wrap primitives in domain-specific types.
- [thiserror documentation](./31-credible-resources-further-reading.md#rust-engineering) Read this because: The practical reference for defining the custom error enums that power the validated constructors used in this chapter. main model in this chapter.
- [Rust API Guidelines](./31-credible-resources-further-reading.md#rust-engineering) Read this because: The official community standard for when and how to wrap primitives in domain-specific types.
- [thiserror documentation](./31-credible-resources-further-reading.md#rust-engineering) Read this because: The practical reference for defining the custom error enums that power the validated constructors used in this chapter. in this chapter.
