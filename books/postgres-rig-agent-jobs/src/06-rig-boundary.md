# 6. The Rig Boundary

## What You Will Learn

This chapter teaches you to:

- explain where Rig belongs in the system and where it does not belong;
- inspect the provider boundary that turns prompts, tool calls, model output, and provider errors into typed decisions;
- verify that untrusted model output never becomes domain state without validation.

The production evidence is a Rig boundary that keeps model interaction typed,
parses structured output, classifies provider errors, and records tool-call
evidence.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the worker owns the execution state machine.
- **Adds:** a model and tool boundary that cannot own reliability rules.
- **Prepares:** local execution that proves the system shape without live providers.

## Production Failure

The model returns JSON that looks almost right, but one field is missing and
one tool name is not allowed for the tenant.

The naive worker accepts the text because it came from the agent framework.

- **What breaks:** untrusted model output becomes authority.
- **False fix:** make the prompt say "always return valid JSON" more strongly.
- **Design response:** let Rig handle model interaction, then parse, validate,
  policy-check, and record typed decisions before state changes.

> ### 🎓 The Professor's Corner
>
> **Byzantine Faults: The Difference Between "Off" and "Weird"**
>
> In distributed systems, we usually deal with "Fail-Stop" errors: the server is either ON or OFF. But models introduce **Byzantine Faults**. This is when a component is ON but behaves in a "weird" or unpredictable way—like returning valid-looking JSON that actually contains nonsense. 
> 
> Treating the model as a source of Byzantine faults is why we have a Customs Checkpoint. We don't assume the model is "broken" just because it fails to parse; we assume it's an **Untrusted Advisor** whose advice must be verified against our own rules.

## Motivation

In production, provider behavior changes underneath you. A timeout, rate limit, malformed response, or model-output drift should not leak through the entire worker and persistence model.

Without a small Rig boundary, the core system becomes coupled to provider DTOs, HTTP errors, and raw model text. This chapter keeps Rig responsible for model/tool interaction while Rust and Postgres preserve reliability.

## Plain Version

Read this as the simple version:

- **Simple rule:** Use Rig for model and tool interaction, but keep reliability rules in your application boundary.
- **Why it matters:** A model framework should not become the place where persistence, approval, retries, and audit rules disappear.
- **What to watch:** Watch model output, tool input, tool output, and agent decisions as untrusted data until parsed and validated.

## What You Already Know

Start by anchoring yourself in the hard-won architecture you have already built. You know that the blue-collar worker firmly owns the state transitions. You explicitly know that the model provider lives entirely outside your trusted core. Finally, you understand that messy provider DTOs and raw model outputs are strictly boundary data, and absolutely never domain truth.

This chapter adds the critical intelligence checkpoint: the Rig boundary. Rig provides the worker with an elegant, powerful way to ask the model for one single reasoning step, while your uncompromising Rust adapters parse, validate, strictly classify, and durably record the result.

## Focus Cue

Keep three critical elements fiercely in view as you read. Regarding **State**, recognize that raw provider responses and transient provider errors must explicitly exist before they are ever allowed to become typed agent results or formal retry decisions. Regarding the **Move**, understand that provider output is only legally permitted to cross into your domain after surviving rigorous parsing, semantic validation, and strict failure classification. Finally, regarding **Proof**, remember that provider DTOs must never leak into the worker, malformed output must be violently rejected, and every single retry disposition must be heavily typed.

If you ever get lost in the boundary logic, immediately return to state, move, and proof. They form the absolute shortest path from a theoretical idea to a concrete production check at 3 AM.

## Production Artifact

Before moving on from this chapter, you must build or rigorously inspect a specific artifact: a robust Rig adapter that explicitly converts wild provider output into heavily typed agent decisions and strictly typed errors. This artifact matters intensely because while Rig beautifully gives the agent its model and tool interaction capabilities, the underlying reliability layer must stubbornly retain the sole authority to validate that interaction. You will know this is "done" when malformed output, hallucinated tools, provider failures, and valid decisions all flawlessly become explicit, predictable domain outcomes.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/rig_runner.rs`, `src/agent_output.rs`, `src/tool_contract.rs`, and the `rig-agent` feature path.
- **State transition:** convert model/provider behavior into typed agent decisions and typed failures.
- **Evidence path:** malformed output, unknown tools, provider errors, and valid results stay at the boundary.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Did provider output become a typed domain decision before it affected state?
- **Evidence to inspect:** raw provider response, parse result, validation error or agent result, retry decision, and event record.
- **Escalate if:** model text, provider DTOs, or unvalidated JSON can authorize a tool or mutate durable state.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** the agent needs model-powered reasoning or a tool decision.
2. **Action:** call Rig at the intelligence boundary and parse the provider response.
3. **Persistence:** persist the typed result, validation failure, retry decision, or terminal error.
4. **Check:** verify untrusted model output never becomes authority without validation.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** provider output becomes typed domain output before authority is granted.
- **Validation path:** run agent-output, tool-contract, and rig feature checks.
- **Stop if:** raw provider DTOs, malformed JSON, or model text can mutate state or authorize tools.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, provider behavior changes underneath you
rule: Use Rig for model and tool interaction, but keep reliability rules in your application boundary
tiny example: raw provider responses and provider errors before they become typed agent results or retry decisions
artifact: a Rig adapter that converts provider output into typed agent decisions and typed errors
proof: provider output becomes typed domain output before authority is granted
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

The boundary has two sides:

```text
inside:
  AgentInstruction, AgentResult, RetryDisposition, policy metadata

outside:
  provider client, model route, HTTP errors, rate limits, raw model text
```

The worker should depend on the inside vocabulary. The provider adapter should
translate the outside world into that vocabulary.

This is the same idea as a customs checkpoint.

Outside the checkpoint, the provider can speak in its own shapes: HTTP status
codes, JSON fields, model text, rate-limit messages, tool-call syntax, and error
payloads. Inside the checkpoint, the worker should see only the system's own
language: `AgentResult`, `ProviderFailure`, `RetryDisposition`, approved tool
requests, and audit evidence.

The point is not to distrust Rig. The point is to put Rig in the correct job.
Rig is a strong model and tool interaction layer. It should help the application
ask the model for useful work. It should not silently decide which side effects
are allowed, which failures are retryable, or which facts are durable business
truth.

When this boundary is small, provider changes stay local. If DeepSeek, OpenAI,
or another provider changes an error shape, the adapter changes. The worker loop,
job table, approval gate, and audit model should not have to learn that
provider's private vocabulary.

## Three-Layer Contract

Keep this split visible whenever Rig appears in the code:

| Layer | What it owns | What it must not hide |
| --- | --- | --- |
| Agent intelligence layer | Model calls, prompts, structured output, tool proposals, and provider interaction through Rig. | Durable state, approval decisions, retry policy, idempotency, audit evidence, or operator control. |
| Reliability layer | Postgres rows, worker leases, retries, idempotency keys, tool-call records, operation events, and recovery rules. | Model reasoning, provider-specific DTOs, or prompt wording as authority. |
| Product/control layer | User intent, tenant permissions, human approvals, policy versions, dashboards, incident handling, and business audit evidence. | Hidden side effects inside a model loop or framework callback. |

The practical rule is:

```text
Rig helps the agent think and propose.
Postgres helps the system remember and recover.
Rust makes the boundary explicit.
```

When a tool call is proposed, ask which layer owns each question:

```text
What might be useful?          -> Rig and the agent intelligence layer
Is it allowed?                 -> product/control layer
Can it be retried safely?      -> reliability layer
Can we prove what happened?    -> reliability plus product/control evidence
```

If one function answers all four questions, the boundary is probably too
large. Split it before the model path becomes the control plane.

## Tiny Example

If DeepSeek times out, the worker should not see "reqwest error 28" as an
unstructured string and decide what to do by substring matching.

The boundary should convert it into a decision the system understands:

```text
provider timeout -> transient agent failure -> retry with backoff
missing API key  -> permanent configuration failure -> dead-letter or fail fast
valid response   -> AgentResult -> policy gate
```

This keeps the retry policy stable even if the provider client changes its
internal error shape.

Notice the order of trust.

The provider error is real evidence, but it is not yet a system decision. The
boundary first classifies it. Only after classification can the worker decide
whether to retry, fail permanently, wait for operator action, or record a policy
failure.

This order matters because stringly typed error handling slowly becomes hidden
architecture. A substring such as `timeout` may work today and fail tomorrow.
A typed `ProviderFailure::TransientTimeout` is a contract the rest of the system
can test, observe, and keep stable across provider changes.

Read the tiny case as:

```text
setup: DeepSeek or another provider returns a timeout, malformed output, or tool proposal
transition: the Rig adapter converts provider behavior into typed domain decisions
evidence: provider error class, parsed output, tool-call row, and trace field are recorded
invariant: provider-specific weirdness does not leak into the core state machine
```

## The Trait

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent.rs:trait}}
```

This trait is intentionally small.

The worker does not need to know whether the answer came from a deterministic
test runner, DeepSeek through Rig, or a future provider. It only needs one
operation: run one typed agent step and return one typed outcome.

That shape gives tests and production the same seam without turning the book
into a mock-heavy design. Local tests use a deterministic runner because the
goal is to test worker behavior, not the internet. The Rig runner implements the
same contract when the reader wants to exercise the real model boundary.

The important production lesson is that `AgentRunner` is not an abstraction for
fashion. It is the narrow point where uncertain model behavior becomes a value
the reliable system can reason about.

## Deterministic Local Runner

The default executable uses this runner. It is deliberately simple so tests are
stable.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent.rs:deterministic_runner}}
```

The deterministic runner is not pretending to be an LLM.

It is a teaching tool for the control system. It lets you prove the worker can
claim work, call the agent boundary, record success, schedule retry, and preserve
state without paying the cost and randomness of a live provider call.

That separation is healthy. Use deterministic tests to prove the reliability
shell. Use provider tests and evaluations to study model behavior. If every test
needs a live model call, the system becomes slow, expensive, flaky, and hard to
debug.

## Real Rig DeepSeek Runner

The real Rig integration lives behind the `rig-agent` feature and uses
DeepSeek through `DEEPSEEK_API_KEY`:

> ### 🎓 The Professor's Corner
>
> **Feature Flags: The "Light Switch" for AI**
>
> Think of a **Feature Flag** (like `--features rig-agent`) as a light switch in your room. You can keep the expensive, heavy AI parts "off" most of the time to save money and test faster. You only flip the switch when you're ready for the real thing!
> 
> This is a "pro-tip" for building on a budget. It allows you to develop the whole "plumbing" of your system using cheap, deterministic tools and only call the "Reasoning Experts" (the LLMs) when the infrastructure is already proven.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/rig_runner.rs}}
```

The manifest aliases the `rig-core` package as `rig`. That gives the code the
same `rig::...` teaching shape while avoiding unused companion crates such as
Bedrock, LanceDB, and FastEmbed in the production audit surface.

Test that boundary with the same feature lane used by the readiness gate:

```bash
cargo test --manifest-path examples/postgres-rig-agent-jobs/Cargo.toml --features rig-agent
```

Run it only when `DEEPSEEK_API_KEY` is set and you intentionally want a
provider call.

The feature flag is a production habit as much as a build detail.

Most reliability tests should not depend on a live provider. They should run in
CI, on a laptop, and during refactors without external credentials. The provider
lane is still important, but it answers a narrower question: does the real Rig
adapter still satisfy the same boundary contract?

This is how the book keeps a clean separation between system correctness and
model-provider availability. A provider outage should not prevent you from
testing lease ownership, idempotency, row conversion, or retry classification.

## What Belongs At This Boundary

The Rig boundary should translate between two worlds:

```text
inside the system: typed job payloads, typed results, retry decisions
outside the system: provider clients, model names, network errors, raw text
```

That boundary should classify failures:

```text
missing API key          -> permanent failure
rate limit / timeout     -> transient failure
malformed agent output   -> permanent failure unless the prompt can be repaired
provider outage          -> transient failure
unsafe requested action  -> policy result, not direct execution
```

This is one of the most important production habits. A retry loop is only safe
when the code knows what should be retried.

Failure classification is where provider engineering meets product safety.

If the provider is down, retrying later may be the right move. If the API key is
missing, retrying every minute only creates noise. If the model produces invalid
tool JSON, the system may need a repair prompt, a dead-letter state, or human
review. If the tool request is unsafe, the correct result is not "provider
failure" at all. It is a policy decision.

These distinctions let operators understand the system. They also protect users.
The model may be uncertain, but the operational response should be explicit.

## Structured Output

The example stores an `AgentResult`:

```text
summary
next_action
approval
```

That shape is intentionally modest. A production agent should return something
small enough to validate. If the model returns a paragraph of prose and the
worker must infer whether it is safe to proceed, the system has moved policy
into ambiguous text.

The real Rig runner therefore asks DeepSeek for one strict JSON object:

```text
{
  "summary": "...",
  "next_action": "...",
  "approval_requirement": "required"
}
```

That object is still not trusted. It is only a transport shape at the provider
edge. The companion crate parses it through a narrow DTO that rejects unknown
fields:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent_output.rs:agent_output_dto}}
```

Then the provider text must cross the parsing and validation boundary before it
can become an `AgentResult`:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/agent_output.rs:agent_output_pipeline}}
```

The safer pattern is:

```text
model proposes -> domain validates -> policy gates -> side-effect worker acts
```

This is the moment where many demo agents become unsafe.

The model can produce JSON that looks structured, but structure is not the same
as truth. A field can be missing. A value can be outside policy. A tool name can
be invented. A model can include extra fields that sound authoritative, such as
`approved` or `skip_review`, even though the model has no authority to approve
anything.

Strict parsing catches shape problems. Domain validation catches meaning
problems. Policy checks catch permission problems. Human approval catches risk
problems. Only after those stages should a side-effect worker act.

This is the difference between **Syntactic Correctness** (is the JSON valid?) and **System Safety** (does the action make sense?). Even if the model returns a perfect JSON object, the *meaning* must be checked against our business invariants before we grant it authority.

## Tool Calling Is A Boundary, Not Magic

Tool calling is the moment where model text tries to become system action.
That is why this book treats it as a production boundary instead of a framework
feature.

The naive mental model is:

```text
model chooses a tool -> tool runs
```

The production mental model is:

```text
model proposes a tool call
  -> parser checks shape
  -> domain validates meaning
  -> policy checks permission
  -> approval checks risk
  -> idempotency checks side-effect safety
  -> tool executes
  -> receipt and audit event are recorded
```

Rig gives the agent a clean way to work with models and tools. It does not
remove the need for typed tool contracts, approval state, idempotency, or audit
evidence. A provider can help produce a tool-call proposal; your application
still owns the decision to execute it.

Keep that word "proposal" precise.

A proposal can be useful. It can save work. It can route attention to the right
tool. But a proposal is not permission, not execution, and not proof. The system
must transform it into an authorized operation before anything changes outside
the model conversation.

This is why tool calls belong near the strongest boundaries in the system. They
are where generated text tries to move money, send email, update records, call
APIs, or expose data. Treating that point as a normal function call hides the
danger exactly where the design needs the most clarity.

## Typed Tool-Call Output Pipeline

The companion crate also includes the same pattern for tool calls. Tool-calling
output enters as `RawModelOutput`, then moves through parsing, domain
validation, policy checking, and human approval before it can become a
`ToolInput`.

The transport shape is strict. Extra model fields are rejected before validation
so text such as `approved: true` or `skip_human_approval: true` cannot smuggle
authority into the typed tool request:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/tool_contract.rs:tool_request_dto}}
```

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/tool_contract.rs:approved_tool_pipeline}}
```

The important detail is the direction of trust:

```text
RawModelOutput
  -> ParsedToolRequest
  -> ValidatedToolRequest<PauseJobKindRequest>
  -> PolicyCheckedToolRequest<PauseJobKindRequest>
  -> ApprovedToolRequest<PauseJobKindRequest>
  -> ToolInput<PauseJobKindRequest>
```

Raw JSON is allowed at the provider boundary. It is not allowed to become a
tool call, approval, audit event, or side effect until typed code validates it.
This is how the book's rule becomes executable:

```text
Raw outside. Typed inside.
```

The rule is simple, but it is not cosmetic.

Raw JSON is useful for transport because providers and tools need flexible wire
formats. Inside the application, the same flexibility becomes risk. If a
`serde_json::Value` can travel through the system unchanged, any later function
has to rediscover whether it is parsed, validated, authorized, and safe.

Typed values carry that history in their name. `ParsedToolRequest` says shape
checking happened. `PolicyCheckedToolRequest` says permission checking happened.
`ApprovedToolRequest` says the required human or policy gate has passed. The
type tells the next function what it may assume.

This chain from `RawModelOutput` to `ApprovedToolRequest` is a beautiful example of a **Capability-Based Security** model. You aren't just giving the model "admin rights"; you're giving it the ability to *propose* an action, which only becomes a *capability* after the validation and approval stages.

## Durable Tool-Call Records

The previous pipeline answers one question:

```text
May this model-proposed tool call execute?
```

Production also needs a second question:

```text
What durable evidence proves the tool call lifecycle?
```

That evidence belongs in `tool_calls`. A tool call can be requested, validated,
executed, failed, or rejected. Those are not labels for a dashboard. They are
state-machine facts that determine which evidence must exist.

The companion code models the legal transitions explicitly:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/tool_call.rs:tool_call_typestate}}
```

The database row is still raw because Postgres stores strings, timestamps, and
JSON. The row boundary immediately converts that storage shape into typed
domain evidence:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/tool_call.rs:tool_call_row_boundary}}
```

This is where the book's side-effect rule becomes concrete. An executed tool
call must have started, completed, and produced an object output. A failed or
rejected tool call must have a terminal reason. A requested or validated tool
call cannot silently carry terminal evidence. The model is allowed to propose;
the system is responsible for proving what happened.

## Formal Definition

For this chapter, the precise definition is:

```text
The Rig boundary is the adapter layer where provider behavior becomes typed agent output or typed failure before worker policy sees it.
```

In the book's system model:

- **State:** raw provider responses and provider errors before they become typed agent results or retry decisions.
- **Actor:** the Rig-backed adapter parses, validates, classifies, and redacts provider behavior before the worker sees it.
- **Transition:** provider output crosses into the domain only after parsing, semantic validation, and failure classification.
- **Evidence:** Provider DTOs do not leak into the worker, malformed output is rejected, and retry disposition is typed.
- **Invariant:** provider-specific shapes and model uncertainty do not leak into worker state or safety policy.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Provider DTOs leak into worker logic. |
| Production symptom | One provider response change forces retry, policy, and worker code changes. |
| Corrective invariant | Provider-specific shape is converted once into typed domain outcomes. |
| Evidence to inspect | The worker depends on `AgentRunner`; `agent_output.rs` validates model output; provider adapters classify timeout, malformed output, and terminal failures. |


## Production Contract

Provider adapters must:

```text
hide provider DTOs from worker logic
map provider failures into typed retry decisions
validate structured output before persisting it as domain result
avoid logging secrets, prompts with sensitive data, or raw credentials
record prompt/model/provider versions for long-horizon audit
```

The boundary is successful when swapping a provider changes the adapter, not
the job state machine.

That sentence is the operational test.

If a provider migration forces edits across worker leasing, retry policy,
approval logic, audit events, and tool execution, the boundary is too wide. The
provider should be replaceable at the intelligence edge. The reliability layer
should keep its own vocabulary and invariants.

This does not mean provider changes are free. Model behavior may change, evals
may fail, prompts may need revision, and cost or latency may shift. But those are
controlled changes. They should move through evaluation and release gates, not
through accidental leakage of provider DTOs into the core state machine.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Provider DTOs leak into worker logic. | Provider DTO leakage lets model quirks shape retry, policy, and persistence logic throughout the system. |
| Safer version | Provider-specific shape is converted once into typed domain outcomes. | Rig handles model and tool interaction while the application converts provider output once into typed outcomes. |
| Production version | The worker depends on `AgentRunner`; `agent_output.rs` validates model output; provider adapters classify timeout, malformed output, and terminal failures. | Malformed model output, provider timeout, and domain result parsing are isolated at the adapter boundary. |

Use the naive row when Rig becomes the whole architecture. Use the safer row when provider shape needs containment. Use the production row before model output can affect durable state.

## Testing Strategy

Test Rig integration as an adapter boundary:

- **Unit or type test:** prove Rust provider output parsing rejects malformed JSON, unknown approval requirements, unexpected tool fields, and provider-shaped data before creating domain results.
- **Persistence or boundary test:** prove Postgres persists only typed agent results, typed failures, and version evidence after the Rig boundary returns.
- **Regression test:** keep a fixture where model text proposes an unknown tool or malformed result; the worker should classify the failure rather than persist raw provider DTOs.

## Observability Strategy

Observe Rig as an adapter, not the source of durable truth:

- Emit structured `tracing` fields for provider name, model version, run id, job id, trace id, output-parse status, and retry classification.
- Record an operation event when provider text becomes typed agent output, malformed output, timeout, or terminal provider failure.
- The runbook query should show the typed outcome that crossed the Rig boundary without leaking provider DTOs into worker policy.

## Security and Safety Considerations

Rig output is model-produced input, not trusted domain authority:

- Treat provider text, tool proposals, structured output, and model reasoning as untrusted until strict parsing, validation, and policy checking pass.
- authorization, sandboxing, and approval should happen after Rig proposes an action and before the application executes it.
- Redact prompts, completions, and provider errors where needed while preserving model version, parse result, retry classification, and trace evidence.

## Operational Checklist

Use this checklist before relying on Rig as the model and tool interaction layer in production:

- **State:** A job reaches Rig only after durable state exists and returns through typed
  result or typed provider failure.
- **Boundary:** Provider request, provider response, model output, and tool proposal
  stay at the Rig boundary until converted.
- **Failure:** Provider timeout, malformed output, policy rejection, and unsupported
  tool call become stable worker decisions.
- **Observability:** Model name, prompt version, provider status, run id, tool proposal
  id, and trace id appear in structured events.
- **Safety:** Rig proposes actions; authorization, sandboxing, approval, and idempotency
  receipts decide whether actions execute.

## Exercises

1. Write a negative test where DeepSeek returns malformed tool JSON or an extra
   `approved` field, and the worker
   records a typed failure without losing the job idempotency key. Explain which
   idempotency key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: agent_runs, tool_calls, provider failure, and
   operation_events rows for one model/tool attempt.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   RawModelOutput, ParsedModelOutput, ValidatedToolRequest, and ProviderFailure types.
   Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: What belongs inside Rig, and what belongs in the reliability layer?
- Explain: Why should provider DTOs and raw model output stop at the boundary?
- Apply: Imagine DeepSeek changes an error response shape. Which boundary should absorb it?
- Evidence: Name the adapter, typed error, retry classification, tool-call record, and trace field that should stay stable.

## Summary

Rig gives the agent hands, but it should not own the reliability system. The worker depends on a small agent boundary and receives typed outcomes.

- **Invariant:** provider behavior and model output enter the system only through validated boundary types.
- **Evidence:** agent runs, provider failures, parsed outputs, tool-call rows, policy decisions, and trace fields show what Rig proposed and what the system accepted.
- **Carry forward:** keep model intelligence replaceable and keep reliability outside the prompt.

## Changed Understanding

- **Before this chapter:** Rig looked like the whole agent system.
- **After this chapter:** Rig gives the model/tool interaction layer; the reliability layer still owns state, policy, idempotency, and audit evidence.
- **Keep:** verify that Rig receives typed, authorized tool contracts rather than raw application state.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
