# 2. The Mental Model

## What You Will Learn

This chapter teaches you to:

- explain the four layers of a reliable agent: intelligence, durable state, execution, and control;
- inspect which layer owns a model call, a job row, a worker action, or a policy gate;
- verify that the layers do not hide each other's responsibilities.

The production evidence is a system boundary where Rig handles model/tool
interaction, Postgres remembers work, Rust workers execute transitions, and
policy gates control risk.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** a durable job exists before the model starts.
- **Adds:** the layered mental model for intelligence, reliability, and control.
- **Prepares:** explicit guarantees and failure semantics for those layers.

## Production Failure

A support agent fails to send a reply after a reviewer approved it.

The team opens the incident and sees only one vague statement:

```text
the agent failed
```

- **What breaks:** nobody can tell whether the failure was provider output,
  worker ownership, policy, approval, persistence, or side-effect execution.
- **False fix:** make the prompt more explicit and retry the whole workflow.
- **Design response:** separate intelligence, reliability, execution, control,
  and evidence so each boundary has an owner and proof.

## Motivation

In production, agent systems become fragile when the model, queue, scheduler, trace log, approval logic, and side effects collapse into one mental bucket. When everything is called "the agent," no one knows which part failed.

Without clean boundaries, a provider timeout can look like a policy problem, and a missing approval can look like model hesitation. This chapter separates the system into parts that can be tested, observed, operated, and replaced.

## Plain Version

Read this as the simple version:

- **Simple rule:** Split the system into intelligence, reliability, and product-control layers.
- **Why it matters:** Rig helps the agent think and call tools, but Postgres and Rust make the work recoverable and reviewable.
- **What to watch:** Check which layer owns each decision before changing code, schema, policy, or operator behavior.

## What You Already Know

Start with these anchors:

- Chapter 1 made the durable job the unit of reliability.
- One component should not own intelligence, state, execution, history, and authority.
- Boundaries make responsibility visible.

This chapter adds: the four-layer mental model. Rig handles model interaction,
Postgres remembers work, Rust workers move state, and policy controls risky
actions.

## Focus Cue

Keep three things in view:

- **State:** the separated responsibilities of durable state, worker execution, provider reasoning, policy, and evidence.
- **Move:** one production fact moves through its assigned boundary without another boundary silently taking over.
- **Proof:** State, worker, provider boundary, policy, event timeline, and operator evidence are separate responsibilities.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a boundary map that separates state, worker execution, model reasoning, policy, and evidence.
- **Why it matters:** agent reliability improves when intelligence and authority are not hidden in the same box.
- **Done when:** one run can be traced from request to worker to model boundary to policy decision to durable event.


## Implementation Map

When you transition from reading this mental model to actual implementation, rely on this map as your guide. The primary surfaces you will interact with are `src/worker.rs`, `src/agent.rs`, `src/rig_runner.rs`, `src/security.rs`, and `src/audit.rs`. The core state transition here is the rigorous routing of a single request explicitly through strictly separated state, execution, model, policy, and evidence boundaries. The evidence path mathematically guarantees that absolutely no component silently owns both probabilistic reasoning and explicit business authority.

## Operator Question

Before you ship any architectural idea based on this mental model, you must answer one vital operational question: Exactly which boundary definitively owns this specific decision: state, worker, model, policy, or evidence? To answer this, you must explicitly inspect the agent run row, the worker event log, the raw provider result, the strict policy decision, and the formal audit event. You should immediately escalate the design to leadership if you ever discover that one single component silently owns reasoning, authority, and durable database mutation all together.

## Runtime Walkthrough

Follow the concept of layered boundaries as a single runtime pass. First, a trigger occurs when a new job securely enters the system. Next, the action requires the architecture to route state transitions, execution logic, probabilistic reasoning, policy checks, and evidence generation strictly through their separate, designated boundaries. For persistence, the system must meticulously record every single boundary crossing as a durable row, a discrete event, or a formally typed decision. Finally, the check requires verifying that absolutely no boundary silently usurped both probabilistic reasoning and strict business authority.

## Acceptance Gate

Do not move on until you can produce the minimum required evidence. You must be able to prove that state, execution, model reasoning, strict policy, and audit evidence fundamentally operate as completely separate boundaries. To validate this path, an operator must trace one single run perfectly through the job rows, the worker events, the provider output, the specific policy decision, and the final audit event. Stop the design process immediately if one single software component can both magically reason and silently authorize a durable mutation without explicit, separate checks.

## Micro-Lesson

If you need a concise summary before diving into the heavier mechanisms, remember this sequence: The pain arises because, in production, agent systems become incredibly fragile when the model, the queue, the scheduler, the trace log, the approval logic, and the side effects are carelessly collapsed into one giant mental bucket. The guiding rule is to aggressively split the system into dedicated intelligence, reliability, and product-control layers. A tiny example of this is observing the fiercely separated responsibilities of durable state, worker execution, provider reasoning, security policy, and operational evidence. The resulting artifact is a formal boundary map that structurally separates state, worker execution, model reasoning, policy, and evidence. The ultimate proof of success is that state, execution, model reasoning, policy, and evidence are demonstrably, mathematically separate boundaries in the code.

## Intuition

The LLM should absolutely never be the workflow. The LLM is merely one highly volatile, mathematically expensive step safely contained *inside* the workflow.

This is the first major mental shift required in this book.

In a slick conference demo, it is perfectly natural to confidently declare, "the agent did the task." That sentence is rhetorically convenient, but it hides an operational nightmare. Did the model actually decide? Did the worker blindly retry? Did the database actually remember the state change, or did it silently drop it? Did the strict security policy actually allow the action, or did we just bypass it? Did a human ever look at it? Did a rogue tool just execute an irreversible side effect against production?

Real production systems demand that those questions have brutally separate, mechanically verifiable answers. The model is undeniably important, but it is not the entire system. It is simply the deeply uncertain, probabilistic reasoning step securely trapped inside a deterministic control system that is specifically designed to be durable, inspectable, and ruthlessly safe.

To visualize this, imagine the flow of a single request from top to bottom. First, the API or UI securely creates a job. Next, the Postgres `agent_jobs` table stubbornly stores this current state. Then, the Rust worker wakes up, aggressively locks the job, and begins execution. Only then does the Rig agent produce a typed, probabilistic recommendation. Immediately afterward, the Postgres `agent_job_events` table permanently records exactly what happened. Finally, the `agent_jobs.result` column stores the final, validated output.

The fragile user request violently becomes durable state long before the model is ever disturbed from its slumber. The blue-collar worker firmly owns the attempt before it executes a single instruction. The Rig boundary neatly wrestles the chaotic model/provider interaction into a predictable, typed result. The immutable event log records the exact, undeniable history of what just happened. Finally, the terminal state is stored completely separately from the messy story of how the system arrived there.

That separation is not corporate bureaucracy. It is the only known way to make a probabilistic system remotely operable at 3 AM.

## Chatbots, Workflows, And Agents

These words are often mixed together, so separate them early.

```text
chatbot:
  answers inside a conversation

workflow:
  follows a mostly known path

agent:
  chooses among tool-backed actions inside a bounded system
```

A chatbot may be useful without durable state. If it gives a weak answer, the
user can ask again. A workflow may need durable state, but it often does not
need model-directed choice because the next step is known in advance. A
production agent needs both: the model helps decide what to do next, and the
system around the model makes that decision recoverable, auditable, and safe.

In this book, an agent is not a free-floating personality. It is:

```text
a worker with tools, memory, permissions, and durable state
```

This is the shift from "Generative AI" to **Decision Intelligence**. We aren't just generating text; we are making bounded choices that result in real-world actions. That definition matters because it tells you where to put reliability. The
model can propose the next action. The workflow state, policy gate, lease,
idempotency key, and event timeline decide whether that action may happen and
how the system recovers if the process dies halfway through.

Another way to say it:

```text
Chatbot: useful answer.
Workflow: known path.
Agent: bounded choice.
Production agent: bounded choice with durable evidence and permission.
```

The last line is the one this book cares about. We are not trying to make the
agent sound more autonomous. We are trying to make its autonomy bounded enough
that a real business can trust, debug, and change it.

## Why The Boundaries Matter

Each boundary prevents a different kind of confusion.

```text
Postgres prevents memory loss.
Rust prevents invalid transitions from spreading.
Rig prevents provider details from leaking into the core.
Events prevent the past from being reconstructed from guesses.
Policy prevents model confidence from becoming authority.
```

By separating the "Intelligence Boundary" (Rig) from the "Control Boundary" (Policy), we are implementing a **State Machine Replication** system where the model provides external reasoning input but the system maintains the state. 

The use of an event timeline is a simple form of **Event Sourcing** or an **Audit Log**. If those responsibilities are combined, the failure mode becomes harder to
see. For example, if model text directly decides whether a refund is safe, an
operator later has to inspect a natural-language answer and infer whether a
policy was followed. If policy is a deterministic boundary, the same operator
can inspect the policy version, decision, actor, and reason.

The boundaries also make testing possible. You can test the Rust state machine
with a deterministic local runner. You can test SQL ownership without calling a
provider. You can test policy without asking the model to behave. The model is
important, but it should not be the only thing standing between a bug and a
production side effect.

The boundary split also helps teams work without stepping on each other. A
model engineer can improve prompts and structured outputs. A backend engineer
can harden leases and retries. A security engineer can review permissions and
sandboxing. An operator can inspect traces and runbooks. These people are all
working on one system, but they are not all editing the same magical loop.

That is the reason this chapter comes early. Before you learn schemas, Rust
types, workers, Rig, approvals, or SLOs, you need a map of who owns what.

## Worked Walkthrough

Imagine you are looking at the execution logs of a supposedly simple agent job.

A perfectly healthy event stream might look delightfully boring:

```text
job_enqueued
job_picked
agent_started
agent_succeeded
job_succeeded
```

However, a stream that encountered the brutal reality of production might look slightly more panicked:

```text
job_enqueued
job_picked
agent_started
agent_failed
retry_scheduled
job_picked
agent_started
agent_succeeded
job_succeeded
```

Even this tiny, seemingly trivial stream already involves several fiercely independent owners.

The `job_enqueued` event belongs entirely to the admission and durable state boundary. The `job_picked` event belongs to the ruthless worker ownership boundary. `agent_started` and `agent_succeeded` belong to the deeply uncertain reasoning boundary. `retry_scheduled` belongs to failure classification and operational scheduling. Finally, `job_succeeded` belongs to the formal terminal state transition.

If an eager developer had carelessly compressed all of those distinct facts into one single, useless log line saying `agent finished`, the system would have instantly lost almost all of the critical information required to successfully operate it at scale.

## The Invariant

To survive production, every single job must exist in exactly one, mathematically clear state:

```text
pending
running
succeeded
failed
dead
cancelled
```

The blue-collar worker does not guess. It violently moves the job through explicit, allowed transitions.

The most important word in that rule is "exactly." A job that is somehow simultaneously `running` and `cancelled` has no clear owner and is a disaster waiting to happen. A job that is neither `pending` nor terminal can magically disappear from every single queue query, becoming a ghost in the machine. A job that possesses generated model output but has absolutely no event timeline is impossible to trust because the system cannot explain how it arrived at that conclusion.

This is exactly where reliability starts to become tangible. A formal state is not just a pretty label for an operational dashboard. It is a strict, binding promise about what is legally allowed to happen next.

A `pending` job may be safely claimed. A `running` job may be completed, retried, formally cancelled, or eventually recovered after its lease expires. A `succeeded` job absolutely should not run again. A `dead` job should emphatically not quietly re-enter the active queue without a deliberate, recorded operator decision.

Once these states possess actual mechanical meaning, the rest of the architecture can become wonderfully simpler. Queries can reliably find work. Workers can aggressively reject illegal transitions. Runbooks can ask incredibly precise questions. Tests can definitively prove that impossible transitions remain delightfully impossible.

## Mechanism

The worker loop is the mechanical, beating heart of this model. It follows a clear, incredibly boring step-by-step **recipe** that forcefully removes the "magic" from the execution:

1. **Read** the next due job from the durable notebook (Postgres).
2. **Claim** the job with an aggressive, durable lease.
3. **Record** an `agent_started` event in the immutable timeline.
4. **Call** the expensive reasoning expert (Rig) for exactly one step.
5. **Classify** whether the outcome was a glorious success or a miserable failure.
6. **Write** the next stable, verified state back to the notebook.
7. **Record** a terminal or retry event to firmly close the loop.

The LLM is tightly boxed *inside* the loop, not floating vaguely around it. This architectural reality means a vastly better model, a significantly cheaper provider, or a cleverly rewritten prompt can completely change the internal reasoning step without altering the ownership, retry, approval, and evidence contracts wrapped around it.

This is the immense, practical payoff of adopting this mental model.

You can entirely replace the model provider without rewriting a single line of job ownership code. You can completely change the retry policy without touching the prompt design. You can forcefully add an approval gate without desperately asking the model to please remember compliance rules in natural language. You can massively improve observability without changing the agent's core reasoning behavior.

The system becomes drastically easier to evolve simply because each layer has a much smaller, better-defined job.

## Layer By Layer

The fully realized system has many moving modules, but this mental model relies on five recurring, critical layers:

1. **Product boundary**: This layer strictly accepts user requests, verifies permissions, establishes tenant context, and determines the initial risk level.
2. **Durable state boundary**: This layer stubbornly stores jobs, runs, steps, leases, retry attempts, and terminal status.
3. **Execution boundary**: This layer aggressively claims work, heartbeats ownership, calls the runner, and writes valid transitions.
4. **Intelligence boundary**: This layer uses Rig to securely call the model and external tools, then cleanly converts the chaotic output into strict typed data.
5. **Control and evidence boundary**: This layer rigidly applies policy, demands approval, enforces sandboxing, records receipts, emits events, tracks metrics, and generates the audit trail.

Do not memorize this list as useless architecture trivia. Use it as an aggressive debugging habit. Whenever something fails in production, the very first question must be: Which specific layer owned the failed decision?

If the model returned comically malformed output, interrogate the intelligence boundary. If two workers dangerously processed the exact same job, interrogate durable state and execution. If a destructive tool executed without explicit permission, interrogate the control boundary. If absolutely nobody can explain what just happened, interrogate the evidence boundary.

This habit forcefully prevents incidents from devolving into vague, unactionable complaints about "the agent." It immediately turns them into sharp, specific engineering questions about state, ownership, policy, or evidence.

## Formal Definition

For this chapter, the precise, formal definition of adoption is clear. A production agent is fundamentally a blue-collar worker equipped with tools, memory, permissions, and undeniable evidence, wrapped securely around a highly probabilistic model step.

In the book's overarching system model, the **State** mapping is precise: you must maintain fiercely separated responsibilities between durable state, worker execution, provider reasoning, strict policy, and operational evidence. The **Actor** interactions are restricted so that each boundary owner—the API, the worker, the Rig adapter, the policy gate, or the operator—is only allowed to change the specific fact it is directly responsible for. The core **Transition** dictates that one single production fact moves smoothly through its assigned boundary without another boundary silently or magically taking over. The **Evidence** ensures that the state representation, the worker process, the provider boundary, the policy check, the event timeline, and the operator evidence remain mathematically separate architectural responsibilities. Ultimately, the governing **Invariant** guarantees that intelligence, state, execution, and safety remain sufficiently decoupled to effectively audit and repair at 3 AM.

## What Can Fail

When dealing with a layered architecture, several critical failure modes can emerge. The most common design smell occurs when these pristine boundaries simply collapse into one giant, tangled agent loop. The production symptom of this tragedy is that provider errors, policy decisions, and state transitions all blur together during an active incident, making debugging impossible. The corrective invariant to ruthlessly enforce is that the state, worker, provider, policy, and evidence boundaries must stay mathematically separate at all times. If a failure occurs, the operational evidence you must inspect includes the event timelines; they must be able to cleanly distinguish between worker actions, model output, policy results, and terminal state.

## Production Contract

Each boundary in this architecture must own exactly one responsibility. Postgres owns the durable state. Rust owns valid, typed transitions. Rig owns exactly one provider-backed reasoning step. Events own the historical explanation. Policy owns the strict permission to act. 

Do not ever let these responsibilities blur. When raw model text is permitted to magically decide policy, the system becomes impossible to securely audit. When transient events are mistakenly used to replace current state, recovery becomes deeply ambiguous. When messy provider DTOs leak deeply into the worker, retry behavior becomes provider-specific instead of system-specific. The contract is uncompromising: boundaries make responsibility visible, and visible responsibility is the only way to operate a system.

## Progressive Hardening Path

Migrating to a layered mental model is a progressive hardening path.

In the naive version, boundaries collapse entirely into one massive agent loop. In this chaotic state, collapsing everything into a single script ensures that mundane network failures look exactly like mysterious, complex model hallucinations.

The safer version dramatically improves upon this by ensuring the state, worker, provider, policy, and evidence boundaries stay structurally separate. Here, separate boundaries allow each layer to excel at exactly one job: remembering things, enforcing permissions, making provider calls, transitioning worker state, or recording evidence.

The final, production-grade version hardens this integration entirely. The team ensures that event timelines definitively distinguish between worker actions, model output, policy results, and terminal state. At this stage, the event timeline clearly shows whether a bad outcome originated from flawed planning, a denied policy, a botched tool execution, hallucinated provider output, or a failed recovery attempt. Use the naive row to rapidly identify agent-loop blur, use the safer row to start separating responsibilities, and rely on the production row when a reviewer must quickly locate the exact failing boundary at 3 AM.

## Testing Strategy

You must aggressively test the boundary split directly in your code. In your unit or type tests, you must prove that your Rust interfaces keep worker state, provider output, policy decisions, memory, and tool execution as fundamentally separate types or traits. Your persistence or boundary tests must explicitly prove that Postgres event rows can definitively distinguish between a worker action, model output, a policy result, a tool call, and the terminal outcome for a single run. Furthermore, your regression tests must encode a "collapsed-agent-loop" case—for instance, where provider text somehow attempts to update durable state directly without a worker transition—and ensure the boundary test violently rejects it.

## Observability Strategy

You must actively observe each boundary as a completely separate operational responsibility. Emit structured `tracing` fields for the job id, run id, boundary name, actor, trace id, status, and the specific provider or policy version. You must record a formal operation event for all worker transitions, model-output conversions, policy decisions, tool executions, and terminal state changes. Ultimately, the runbook query you construct should instantly reveal whether a failure originated from bad planning, corrupted provider output, a restrictive policy, a broken tool execution, lost memory, or a botched recovery attempt.

## Security and Safety Considerations

Boundary separation is not just good architecture; it is a critical security control. You must treat model text, retrieved memory, tool output, provider responses, and database rows as inherently untrusted data whenever they cross from one layer into another. Crucially, authorization, sandboxing, and approval must live entirely outside the model loop, ensuring the agent can freely propose actions but never receives the authority to execute them by default. Always meticulously redact sensitive context at each boundary while preserving exactly enough typed evidence to definitively explain which specific layer allowed or denied the transition.

## Operational Checklist

Before relying on the boundary split between state, worker, model, events, and policy in production, operators must perform a strict review.

First, verify the **State** boundary: ensure each component fiercely owns exactly one part of the agent lifecycle and cannot silently take over another boundary. Second, inspect the **Boundary** transitions themselves: verify that Rig handles model/tool interaction exclusively, while Rust and Postgres completely own durable execution, policy, and evidence. 

Third, rehearse your **Failure** modes: ensure a standard debugging path can instantly identify whether a bug belongs to fuzzy model reasoning, rigid worker execution, strict policy, durable state, or the evidence log. Fourth, validate your **Observability** pipeline: confirm that events and traces explicitly show which boundary made each decision, and which boundary merely carried the data forward. Finally, verify **Safety**: ensure that the model never, ever grants itself permission; policy, approval, sandboxing, and side-effect receipts must stay entirely outside of the prompt text.

## Exercises

To test your mastery, write a negative test where a hallucinating model response attempts to directly update Postgres state or skip an idempotency check without going through a formal worker transition. You must explicitly explain which idempotency key, receipt, or state transition effectively blocked the duplicate work. Next, sketch the exact Postgres evidence: one event timeline that beautifully separates intake, planning, tool proposal, policy decision, execution, and receipt. Finally, define or heavily refine the Rust type, enum, constructor, or typestate that represents cleanly separated `AgentBoundary`, `WorkerBoundary`, `PolicyBoundary`, and `EvidenceBoundary` types or modules. Then, meticulously name the runbook question that proves this enforcement actually works.

## Self-Check

Before you move on, use this quick retrieval drill to solidify your layered understanding. First, recall exactly which layers officially own intelligence, durable state, execution, policy, and evidence. Next, be able to clearly explain why lazily merging those layers into a single prompt text makes failures catastrophically harder to debug. Then, apply this knowledge by correctly placing a tool call failure into its exact layer and assigning it to the correct owner. Finally, explicitly name the specific row, event, trace field, or policy record that should definitively prove the boundary behaved exactly as intended.

## Summary

The clean, inviolable boundary in this architecture is that Postgres tracks state, Rust executes transitions, Rig provides model/tool interaction, policy aggressively controls risk, and events stubbornly preserve the story.

The core invariant to remember is that no layer is ever permitted to silently take responsibility from another layer. To enforce this, your architecture must rely on ensuring a job timeline explicitly shows which boundary successfully handled intake, planning, tool proposal, policy, execution, receipt, and review. 

Moving forward, remember the golden rule: when aggressively debugging an agent incident at 3 AM, the very first question you must ask is exactly which boundary owned the failed decision.

## Changed Understanding

Before reading this chapter, an agent probably looked like a magical, incredibly smart conversational loop. After this chapter, you should understand that a production agent is essentially just a highly probabilistic worker trapped inside a rigidly deterministic control system. Moving forward, keep in mind that you must always fiercely separate model reasoning from product control, reliability control, and durable evidence.

## Further Reading & Credible References

- **[Gul Agha: Actors—A Model of Concurrent Computation in Distributed Systems](https://mitpress.mit.edu/9780262010924/actors/)** (1986). The academic foundation for the "Agent as Worker" model. It formalizes why actors (workers) must be decoupled in time and space, communicating only through messages (jobs) to ensure safety and scalability.
- **[Stripe Engineering: Scaling Idempotency](https://stripe.com/blog/idempotency)**. A practical industry reference for how to build high-availability systems where "Intelligence" (the user request) is separated from "Execution" (the payment) via stable idempotency keys.
- **[Anthropic: Building Effective Agents](https://www.anthropic.com/research/building-effective-agents)** (2025). Distinguishes between workflows and agents, helping identify which boundary should own the reasoning step.
- **[ReAct: Synergizing Reasoning and Acting in Language Models](https://arxiv.org/abs/2210.03629)** (2022). Research into interleaving reasoning and action, providing the intuition for the Rig/Worker boundary.
