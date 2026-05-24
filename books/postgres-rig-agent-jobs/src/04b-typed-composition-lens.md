# 4.5 Typed Composition Lens

## What You Will Learn

This chapter teaches you to:

- explain typed composition as a chain of safe transformations;
- inspect whether the output of one step really matches the input of the next;
- verify that typestate and newtypes make illegal workflow transitions hard to express.

The production evidence is a pipeline where raw input becomes validated domain
state, approved tool requests become executable work, and completed states
cannot be retried by accident.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** domain values now have names and invariants.
- **Adds:** safe composition of typed transformations and selected typestate.
- **Prepares:** the worker loop that moves typed jobs through durable states.

## Production Failure

A validated tool request is stored, then later executed through a function that
also accepts raw model output.

One refactor accidentally sends an unvalidated request down the same path.

- **What breaks:** the pipeline did not preserve the difference between raw,
  parsed, validated, authorized, approved, and executed states.
- **False fix:** add comments saying "only call this after validation."
- **Design response:** compose typed transformations so each function accepts
  only the state it is allowed to handle, and use typestate when workflow order
  is safety-critical.

## Motivation

In production, a workflow state determines which operations are legal. A requested tool call should not execute before validation, and a completed job should not be retried as if it were pending.

Without typed composition, pipelines become long chains of hope around raw JSON and booleans. This chapter shows when newtypes, typestate, and simple composition language make illegal transitions harder to express.

## Plain Version

Read this as the simple version:

- **Simple rule:** Treat the agent system as small typed transformations that can be safely composed.
- **Why it matters:** Composition is reliable only when each output clearly matches the next input and failures stay explicit.
- **What to watch:** Watch tool pipelines, validation pipelines, and state transitions for mismatched types or hidden side effects.

## What You Already Know

Start with these anchors:

- Chapter 4 gave important values real Rust names.
- Some objects have a legal order of operations.
- A completed job, approved request, and validated tool input are not interchangeable states.

This chapter adds: typed composition and typestate. You will use the type
system where workflow order matters, while avoiding type machinery where a
simple validated constructor is enough.

## Focus Cue

Keep three things in view:

- **State:** typed transformations, lifecycle states, and error pipelines that can be safely composed.
- **Move:** a value moves through a pipeline only when the previous output type satisfies the next input requirement.
- **Proof:** Newtypes, typestate builders, and `Result` pipelines make illegal composition visible before runtime.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a small typestate pipeline for one lifecycle that has illegal transitions.
- **Why it matters:** compile-time state is useful when runtime checks would protect a dangerous operation too late.
- **Done when:** an unvalidated or unapproved value cannot call the operation reserved for a validated or approved value.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/typed_pipeline.rs`, typestate examples, and lifecycle transition tests.
- **State transition:** move values through legal states by type instead of late runtime flags.
- **Evidence path:** unvalidated, unapproved, or incomplete values cannot call production-only operations.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which illegal transition should the compiler make impossible?
- **Evidence to inspect:** typestate builder, lifecycle enum, transition method, and compile-checked test.
- **Escalate if:** a completed, rejected, unvalidated, or unapproved value can still call a forbidden operation.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a value moves through a lifecycle with illegal transitions.
2. **Action:** encode the legal transition as a type-level or enum-level step.
3. **Persistence:** produce the next typed state only after the precondition is met.
4. **Check:** verify forbidden operations are unavailable or rejected before side effects.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** illegal lifecycle transitions are rejected by type or explicit state rules.
- **Validation path:** inspect typestate examples, transition methods, and lifecycle tests.
- **Stop if:** a value can skip validation, approval, execution order, or terminal-state rules.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, a workflow state determines which operations are legal
rule: Treat the agent system as small typed transformations that can be safely composed
tiny example: typed transformations, lifecycle states, and error pipelines that can be safely composed
artifact: a small typestate pipeline for one lifecycle that has illegal transitions
proof: illegal lifecycle transitions are rejected by type or explicit state rules
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Intuition

Think of a production agent job as a small pipeline:

```text
external request
  -> validated domain values
  -> durable job
  -> leased worker execution
  -> typed agent result
  -> policy decision
  -> optional side effect
```

Category theory gives a useful lens: each arrow transforms one valid state into
another valid state. Rust lets us encode some of those arrows with types.

Start with the engineering idea before the math.

A reliable agent system keeps moving values across boundaries. A raw webhook
becomes a validated request. A validated request becomes a durable job. A
durable job becomes a leased attempt. A model response becomes a parsed result.
A parsed result becomes a policy-checked proposal. An approved proposal becomes
an executable side effect.

Each step should tell the truth about what it requires and what it produces.
If a function executes a tool, its input type should already prove that parsing,
validation, authorization, approval, and idempotency have happened. If those
facts are missing, the function should not be callable through the safe API.

That is typed composition. It is not about clever function chaining. It is
about making the legal path visible and the illegal path hard to express.

## Tiny Example

A duplicate webhook should not become two independent jobs.

As a composition problem, the useful arrow is:

```text
WebhookRequest -> IdempotencyKey -> AgentJobRequest -> durable job row
```

The identity-like behavior is:

```text
same IdempotencyKey -> same logical job
```

The Rust types do not need to mention category theory. They only need to make
the path clear: validate the request, derive the key, build a job request, then
persist it.

Read the tiny case as:

```text
setup: a webhook may be delivered twice
transition: raw input becomes an idempotency key, then a typed job request, then a row
evidence: each transformation has a typed output that the next step accepts
invariant: composition is safe only when adjacent boundaries agree on meaning
```

The key phrase is adjacent boundaries.

The output of one step must be meaningful as the input of the next step.
`WebhookRequest` is not enough to insert work because it may be malformed,
duplicated, unauthorized, or missing tenant context. `AgentJobRequest` is closer
to the domain because construction already collected and validated the fields
the job table needs.

When a pipeline jumps from raw input directly to a side effect, the system has
lost the chance to ask smaller questions. Is this input valid? Is the action
allowed? Is it safe to retry? Has the same logical request appeared before?
Typed composition keeps those questions in order.

## Newtypes

Newtypes are the most important type-safety tool in this book.

```text
String              -> too vague
AgentInstruction    -> model input
IdempotencyKey      -> duplicate-suppression key
WorkerId            -> lease owner
NextAction          -> proposed operator action
FailureMessage      -> safe persisted failure text
CancellationReason  -> explicit operator/system stop reason
LeaseDuration       -> ownership timeout, not arbitrary time
QueueDepth          -> queue count with operational meaning
```

Newtypes are worth using when a value crosses a boundary, has an invariant, or
can be confused with another value of the same primitive type.

The stricter production rule in this book is:

```text
raw primitives may enter only at adapters
domain code receives named types
store and worker traits expose named types
private DTOs convert immediately
```

This is why the Postgres adapter may read raw database `text` privately, but
the worker sees `FailureMessage`, `CancellationReason`, `RetryDisposition`, and
`LeaseDuration`.

Newtypes are the base layer of composition. They let functions announce the
meaning they need, not only the storage shape they accept.

A function that takes `String` tells the caller almost nothing. A function that
takes `IdempotencyKey` says the caller has already derived the duplicate
suppression identity. A function that takes `ApprovedToolRequest` says the
caller has already passed the approval boundary.

Good names reduce the amount of trust hidden between steps.

## Typestate

Typestate is useful when an object has a construction sequence and the compiler
can prevent illegal ordering.

The companion crate includes a small compile-checked builder:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/typed_pipeline.rs}}
```

This prevents code from building an enqueueable job before it has both:

```text
job kind
agent instruction
```

The important detail is where typestate is used. It is good for request
construction. It is not the right primary model for persisted job status,
because persisted status is runtime data loaded from Postgres.

This is the distinction between **Local Transactional State** and **Global
Persisted State**. You can use a Rust type to guarantee that your *local* code
has the chassis before the wheels (Typestate). But you cannot use a Rust type to
guarantee that a row in a *different* database server is still "Pending." The
database is the authority for Global State, and Rust enums are the right tool to
represent that runtime evidence after you've read it.

Once a row comes from the database, validate it into domain types and use an
explicit `JobStatus` enum.

Typestate is strongest when the order is local, stable, and dangerous to skip.

Building a command is local. The compiler can know whether the builder has a
job kind and an instruction. Executing a tool after approval is also a good
candidate when only an approved value should expose `execute`.

Persisted workflow state is different. A database row is runtime evidence, not
compile-time evidence. The compiler cannot know which status a row will have
until the program reads and validates it. In that case, use enums, conversion
errors, and explicit transition methods.

The practical rule is:

```text
Use typestate for local construction order.
Use enums for persisted runtime state.
Use explicit transition methods for business rules.
```

## Category Theory Lens

Use category theory as a compact way to think, not as decoration.

Useful ideas:

```text
composition:
  small safe transitions combine into a larger workflow

identity:
  duplicate enqueue with the same idempotency key returns the same job

functor-like mapping:
  convert provider DTOs into domain results without changing the outer workflow

monadic/error flow:
  each step may fail, and typed errors decide retry vs dead-letter
```

Avoid this:

```text
renaming ordinary services to Category, Functor, Monad, NaturalTransformation
```

That makes the system harder to operate. Production code should be boring.

The useful idea is simple: safe steps can be connected when the shape and
meaning line up.

In category-theory language, the valid states are objects and the lawful
transformations are arrows. You do not need that vocabulary to write the code,
but the lens helps explain why the book cares so much about matching inputs and
outputs. A pipeline is safe only when each arrow preserves the validity needed
by the next arrow.

Use the math when it clarifies the engineering. Drop it when it becomes a
performance.

## Formal Definition

For this chapter, the precise definition is:

```text
Typed composition is the safe connection of transformations whose output and input types match, with typestate used when lifecycle order matters.
```

In the book's system model:

- **State:** typed transformations, lifecycle states, and error pipelines that can be safely composed.
- **Actor:** the compiler, constructors, and developers connect only compatible outputs, inputs, and lifecycle states.
- **Transition:** a value moves through a pipeline only when the previous output type satisfies the next input requirement.
- **Evidence:** Newtypes, typestate builders, and `Result` pipelines make illegal composition visible before runtime.
- **Invariant:** composition makes category errors and illegal lifecycle transitions visible before production execution.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Lifecycle order is enforced by convention. |
| Production symptom | A request can be sent before auth, payload validation, or policy preparation. |
| Corrective invariant | Invalid lifecycle transitions are not expressible through the public API. |
| Evidence to inspect | Typestate builders expose `build` only after required states exist. |


## Production Contract

Use the three ideas at different strengths:

```text
newtypes: default for meaningful boundary values
typestate: selective for construction order and lifecycle gates
category theory: teaching lens for composition, identity, and error flow
```

Do not encode database runtime status as compile-time typestate. Do not expose
math vocabulary in operational APIs. Do use types to make the safe path the
easy path.

This contract is intentionally pragmatic.

The book does not ask you to turn the whole system into type-level machinery.
It asks you to notice where a wrong order would hurt production. A missing
idempotency key before enqueue is dangerous. A missing approval before a
rollback is dangerous. A missing receipt before replay is dangerous. Those are
good places for stronger types or explicit state transitions.

For low-risk local code, plain functions are fine. For high-risk workflow
boundaries, the type signature should make the preconditions visible.

## Judgment: When Not To Use Typestate

Typestate is powerful because it can make illegal transitions impossible to
call. That power has a cost.

Use typestate when the lifecycle order is stable and a wrong call can create a
dangerous side effect:

```text
unvalidated -> validated
unauthorized -> authorized
approval requested -> approved
draft tool request -> executable tool request
incomplete builder -> complete command
```

Do not use typestate when the state is mostly runtime data:

```text
database rows loaded from Postgres
states configured by policy at runtime
workflow states that change weekly
dashboards and operator filters
simple values that only need one constructor
```

For persisted agent work, the usual split is:

```text
database status -> enum plus row conversion
command construction -> typestate if order matters
tool execution gate -> typestate or explicit approved type
operator query -> plain enum and SQL predicate
```

When typestate makes every function generic and nobody can explain the
signature, the design is too heavy. Return to a validated constructor, an enum,
and a clear transition method.

Category theory follows the same rule. Use it to explain composition,
identity, mapping, and error flow. Do not make the production API speak math
when ordinary engineering names are clearer.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Lifecycle order is enforced by convention. | Convention-only lifecycle order lets callers assemble valid-looking objects in invalid states. |
| Safer version | Invalid lifecycle transitions are not expressible through the public API. | Typed transformations and typestate make the allowed composition visible in the function signatures. |
| Production version | Typestate builders expose `build` only after required states exist. | The code can expose `build`, `execute`, or `approve` only after the required typed evidence exists. |

Use the naive row when lifecycle order lives in prose. Use the safer row when composition needs types. Use the production row when illegal transitions should be unrepresentable.

## Testing Strategy

Test composition by making illegal ordering impossible or explicit:

- **Unit or type test:** prove the Rust typestate builder exposes `build`, `execute`, or `approve` only after the required earlier states exist.
- **Persistence or boundary test:** prove Postgres lifecycle rows cannot be decoded into a later domain state unless the earlier evidence is present.
- **Regression test:** preserve a case where an output is used before validation or approval; the type pipeline should fail before runtime side effects.

## Observability Strategy

Observe composition as a sequence of typed transformations:

- Emit structured `tracing` fields for pipeline step, input type, output type, typestate state, trace id, and validation outcome.
- Record an operation event when a pipeline advances from raw input to parsed value, validated value, policy-checked value, approved value, or executed result.
- The runbook query should identify the exact transformation where a workflow stopped before an illegal side effect could occur.

## Security and Safety Considerations

Composition is safe only when unsafe states cannot skip validation:

- Treat each pipeline input as untrusted until the previous typed transformation has produced the required state.
- authorization, sandboxing, and approval should be separate transformations before an executable tool request or side effect receipt can exist.
- Redact raw model and tool payloads between transformations while preserving the typed state names and failure evidence.

## Operational Checklist

Use this checklist before relying on typed composition and typestate in production:

- **State:** Lifecycle states such as Pending, Running, WaitingForHuman, Approved, and
  Completed determine which operations exist.
- **Boundary:** Raw model output enters a validation pipeline before it becomes a typed
  tool request or state transition.
- **Failure:** Illegal transitions such as completing an unapproved run or executing an
  unvalidated tool call are unrepresentable or rejected.
- **Observability:** Typed pipeline steps emit trace fields and events that match the
  state transition names.
- **Safety:** Side-effecting steps appear after parse, validation, authorization,
  sandboxing, approval, and idempotency checks.

## Exercises

1. Write a negative test where a `ToolCall<Requested>` is executed before validation
   or approval supplies the required idempotency evidence. Explain which idempotency
   key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: rows that prove raw output, validated request, policy
   decision, approval, execution, and receipt are separate evidence.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   `ToolCall<Requested>`, `ToolCall<Validated>`, `ToolCall<Approved>`, and
   `ToolCall<Executed>`. Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What problem does typestate solve that an enum alone may not solve?
- Explain: Why should typed composition start with engineering flow before category-theory words?
- Apply: Choose one lifecycle where only the next legal operation should compile.
- Evidence: Name the state type, transition method, invalid call, and compile-time or unit test that proves safety.

## Summary

Use newtypes broadly for domain meaning. Use typestate narrowly where lifecycle state determines which operations are legal. Use composition language only when it clarifies safe pipelines.

- **Invariant:** raw input, model output, and workflow state move through typed transformations before side effects execute.
- **Evidence:** compile-time states, validation steps, policy decisions, approval records, and receipts form a visible path.
- **Carry forward:** type discipline is not cleverness; it is production bug prevention.

## Changed Understanding

- **Before this chapter:** composition looked like chaining functions because the code compiles.
- **After this chapter:** safe composition means every step transforms one validated type into the next lawful type.
- **Keep:** read each pipeline arrow as a validated transformation between typed states.

## Further Reading & Credible References

- **[The Embedded Rust Book: Typestate Programming](https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html)**. The primary industry reference for using the type system to enforce valid state transitions (e.g., ensuring a job is leased before it is executed).
- **[Cliff L. Biffle: The Typestate Pattern in Rust](https://cliffle.com/blog/rust-typestate/)**. A definitive explanation of how Rust's ownership and move semantics enable "un-misusable" APIs.
- **[Edwin Brady: Type-Driven Development with Idris](https://www.manning.com/books/type-driven-development-with-idris)**. Although Idris-specific, this research-backed guide explores how types can be used to describe and verify stateful protocols and concurrent communication orders.
- **[Kohei Honda: Session Types and Distributed Computing](https://dl.acm.org/doi/10.1145/2103736.2103744)** (2012). The seminal academic work on "Session Types," which treat distributed protocols as types that can be verified at compile time.
