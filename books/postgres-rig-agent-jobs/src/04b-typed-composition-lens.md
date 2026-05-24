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

Start by anchoring yourself in the solid type theory you have already established. Chapter 4 finally gave your most important, dangerous production values real Rust names. You also intuitively know that certain operational objects have a strictly legal order of operations. Finally, you understand that a completed job, a formally approved request, and a merely validated tool input are profoundly not interchangeable states, even if they share similar data structures.

This chapter adds the final polish: typed composition and typestate. You will learn to violently deploy the type system wherever workflow order matters, while pragmatically avoiding heavy type machinery in places where a simple, validated constructor is perfectly sufficient.

## Focus Cue

Keep three critical elements fiercely in view as you read. Regarding **State**, recognize that explicitly typed transformations, lifecycle states, and error pipelines are the only things that can be safely composed without fear. Regarding the **Move**, understand that a value is only permitted to move through a pipeline when the previous output type flawlessly and mathematically satisfies the next input requirement. Finally, regarding **Proof**, remember that newtypes, typestate builders, and explicit `Result` pipelines are the undeniable proof that illegal composition was caught and destroyed long before runtime.

If you ever get lost in the abstraction, immediately return to state, move, and proof. They form the absolute shortest path from a theoretical typing concept to a concrete production check.

## Production Artifact

Before moving on from this chapter, you must build or rigorously inspect a specific artifact: a small typestate pipeline specifically designed for a single lifecycle that is prone to illegal transitions. This artifact matters intensely because compile-time state tracking is incredibly useful when relying on runtime checks would only manage to protect a dangerous operation far too late. You will know this is "done" when an unvalidated or unapproved value mathematically cannot even attempt to call the operation strictly reserved for a validated or approved value.


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

For this chapter, the formal definition of adoption is unapologetically academic yet intensely practical: Typed composition is the mathematically safe connection of transformations where the output and input types must flawlessly match, actively utilizing typestate whenever the specific lifecycle order of operations matters to system safety.

In the book's overarching system model, the **State** mapping is precise: typed transformations, lifecycle states, and strict error pipelines are explicitly defined so they can be safely composed. The **Actor** interactions are restricted so that the Rust compiler, the smart constructors, and the developers themselves can connect only fully compatible outputs, inputs, and lifecycle states. The core **Transition** dictates that a value moves through a pipeline *only* when the previous step's output type undeniably satisfies the next step's rigorous input requirement. The **Evidence** ensures that newtypes, typestate builders, and explicit `Result` pipelines make any attempt at illegal composition glaringly visible long before runtime. Ultimately, the governing **Invariant** guarantees that this strict composition successfully forces both category errors and illegal lifecycle transitions to become visible, compile-time failures rather than terrifying production execution bugs.

## What Can Fail

When relying on composition, the most dangerous failure mode is blind optimism. The most common design smell occurs when a critical lifecycle order is enforced purely by loose team convention or hopeful comments rather than strict types. The production symptom of this tragedy is that an unvalidated request can suddenly be sent out before formal authorization, payload validation, or necessary policy preparation has actually happened. The corrective invariant to ruthlessly enforce is that invalid, out-of-order lifecycle transitions must be mathematically inexpressible through your public API. If a failure occurs, the operational evidence you must inspect includes the typestate builders themselves; they should only ever expose a `.build()` or `.execute()` method *after* all prerequisite states definitively exist in the type signature.

## Production Contract

When integrating these powerful type concepts, you are drafting a clear, three-part production contract. 

First, use **newtypes** as your absolute default for any meaningful boundary value. Second, use **typestate** selectively, reserving it strictly for construction order and critical lifecycle gates where skipping a step would cause an incident. Third, use **category theory** purely as a conceptual teaching lens for understanding composition, identity, and error flow, but never as an excuse to rename ordinary services to things like `Monad` or `NaturalTransformation`.

Do not attempt to encode your highly volatile, runtime database status as rigid compile-time typestate. Do not lazily expose confusing math vocabulary in your operational, blue-collar APIs. But absolutely do use these types to structurally force the safe path to be the easiest, most obvious path for the next developer who touches your code at 2 AM.

## Judgment: When Not To Use Typestate

Typestate is an incredibly powerful tool because it can literally make illegal transitions mathematically impossible to call. However, that power comes with a severe cognitive cost.

You should aggressively use typestate when the lifecycle order is highly stable and a wrong call can create a dangerous external side effect. Excellent use cases include moving from unvalidated to validated, from unauthorized to authorized, from approval requested to formally approved, from a draft tool request to an executable tool request, or from an incomplete builder to a fully populated command.

You must absolutely not use typestate when the state is primarily messy, unpredictable runtime data. Terrible use cases include raw database rows freshly loaded from Postgres, states that are dynamically configured by external policy at runtime, workflow states that change every other week based on product requirements, complex dashboards, or simple, scalar values that genuinely only need one standard constructor.

For persisted agent work, the pragmatic split is simple: use an enum and strict row conversion for database status, use typestate for command construction when order truly matters, use a dedicated 'approved' type for the final tool execution gate, and use plain enums for operator queries. When your typestate implementation forces every single function in the codebase to become generically tangled and no one on your team can easily explain the signature, your design has become too heavy. Delete it, and confidently return to using a validated constructor, a clear enum, and a strictly defined transition method.

## Progressive Hardening Path

Migrating to typestate and strict composition is a progressive hardening path.

In the naive version, lifecycle order is desperately enforced by team convention and hopeful comments. In this fragile state, convention-only order allows weary developers to easily assemble perfectly valid-looking objects that are actually in completely invalid, dangerous states.

The safer version dramatically improves upon this by ensuring that invalid lifecycle transitions are fundamentally inexpressible through the public API. Here, typed transformations and strategic typestate mechanically force the allowed composition to be glaringly visible directly in the function signatures.

The final, production-grade version hardens this integration entirely. The team implements typestate builders that physically only expose `.build()`, `.execute()`, or `.approve()` methods *after* the required, typed prerequisite evidence exists. Use the naive row to aggressively spot "convention-based" safety, use the safer row when composition demands type-level enforcement, and rely on the production row when illegal transitions absolutely must be unrepresentable.

## Testing Strategy

You must aggressively test your composition by ensuring illegal ordering is either mathematically impossible or explicitly, violently rejected. In your unit or type tests, you must prove that your Rust typestate builder legitimately only exposes its terminal methods after the required earlier states physically exist in the type signature. Your persistence or boundary tests must unequivocally prove that Postgres lifecycle rows cannot be successfully decoded into a later domain state unless the earlier, prerequisite evidence is fully present in the data. Furthermore, your regression tests must meticulously encode a terrifying scenario where an output is somehow maliciously used before validation or approval; your type pipeline must structurally fail long before any runtime side effects can be triggered.

## Observability Strategy

You must actively observe your composition as a strict sequence of explicitly typed transformations. Emit structured `tracing` fields detailing the specific pipeline step, the exact input type, the resulting output type, the current typestate, the trace id, and the formal validation outcome. You must record a formal operation event the instant a pipeline successfully advances from raw input to parsed value, to a validated value, to a policy-checked value, to an approved value, and finally to an executed result. Ultimately, the runbook query you construct should instantly identify the exact transformation layer where a failing workflow stopped, definitively proving that it halted long before an illegal side effect could ever occur.

## Security and Safety Considerations

Typed composition functions as a genuine security control only when unsafe states absolutely cannot skip mandatory validation steps. You must treat every single pipeline step that crosses a trust boundary—such as an API endpoint, a database read, or an untrusted model output—as fiercely demanding its own distinct type before it can proceed. Authorization, strict sandboxing, and formal approval checks must be represented as unforgeable, un-skippable steps within the composition chain. Always meticulously redact sensitive payload values from your type-level tracing, while ensuring the exact sequence of typed transformations remains perfectly visible for the inevitable security audit.

Treat every single pipeline input as inherently hostile until the previous typed transformation has explicitly produced the required state. Authorization, sandboxing, and approval decisions must be separate, legally enforced transformations *before* an executable tool request or a side-effect receipt can physically exist in the system. Finally, aggressively redact raw model and tool payloads between transformations while flawlessly preserving the typed state names and any associated failure evidence.

## Operational Checklist

Before relying on typed composition and typestate in production, operators must perform a strict review of the system's boundaries.

First, verify the **State** boundary: ensure lifecycle states such as Pending, Running, WaitingForHuman, Approved, and Completed strictly determine which operations physically exist on the object. Second, inspect the **Boundary** transitions themselves: verify that raw model output violently enters a validation pipeline before it can ever become a typed tool request or trigger a state transition. 

Third, rehearse your **Failure** modes: ensure that illegal transitions—such as trying to complete an unapproved run or blindly executing an unvalidated tool call—are either mathematically unrepresentable by the compiler or brutally rejected at runtime. Fourth, validate your **Observability** pipeline: confirm that typed pipeline steps emit trace fields and operation events that perfectly match the names of the state transitions. Finally, verify **Safety**: ensure that any side-effecting step only appears mathematically *after* the parse, validation, authorization, sandboxing, approval, and idempotency checks have successfully completed.

## Exercises

To test your operational mastery, write a decidedly negative test where a `ToolCall<Requested>` is stubbornly attempting to execute before formal validation or approval supplies the required idempotency evidence. You must explicitly explain which idempotency key, receipt, or state transition mathematically prevented the duplicate work. Next, sketch the exact Postgres evidence: explicitly define the rows that prove raw output, validated request, policy decision, approval, execution, and receipt are stored as completely separate evidence. Finally, define or heavily refine the Rust type, enum, constructor, or typestate that precisely represents the transitions between `ToolCall<Requested>`, `ToolCall<Validated>`, `ToolCall<Approved>`, and `ToolCall<Executed>`. Then, meticulously name the runbook question that proves this enforcement mechanism actually works at 3 AM.

## Self-Check

Before you move on, use this quick retrieval drill to solidify your composition understanding. First, recall exactly what operational problem typestate solves that a simple enum alone cannot mathematically solve. Next, be able to clearly explain why applying typed composition must start with boring engineering flow rather than intimidating category-theory words. Then, defensively scan your own codebase and explicitly choose one lifecycle where only the strictly *next* legal operation should be allowed to compile. Finally, explicitly name the state type, the transition method, the invalid call you prevented, and the compile-time or unit test that undeniably proves your safety.

## Summary

Use newtypes broadly for clear domain meaning. Use typestate narrowly and surgically precisely where the lifecycle state must dictate which operations are legally allowed to exist. Use category theory composition language only when it actively clarifies the engineering pipeline, and never when it obscures it.

The core invariant to remember is that raw input, unpredictable model output, and core workflow state must all securely move through strictly typed transformations long before any dangerous side effects can execute. To enforce this, your architecture must visibly rely on compile-time states, rigorous validation steps, unforgeable policy decisions, formal approval records, and concrete receipts forming an undeniable execution path. 

Moving forward, remember the golden rule: strict type discipline is profoundly not about academic cleverness; it is the most effective form of production bug prevention available.

## Changed Understanding

Before reading this chapter, composition likely looked exactly like casually chaining functions together simply because the compiler allowed it. After this chapter, you should understand that genuinely safe composition means every single step rigorously transforms one formally validated type into the next lawful, prerequisite type. Moving forward, keep in mind that you must always read each pipeline arrow as a highly validated transformation between explicitly typed states.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
