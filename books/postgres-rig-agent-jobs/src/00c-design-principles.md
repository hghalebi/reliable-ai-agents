# Design Principles

## What You Will Learn

This chapter teaches you to:

- explain the ten rules that survive a change of framework, provider, database, or hosting platform;
- inspect a design decision and ask which principle it protects or violates;
- verify that each principle maps to a concrete artifact, not a slogan.

The production evidence is a principle-to-artifact map that connects durable
state, typed boundaries, ownership, idempotency, observability, approval,
evaluation, release safety, and recovery practice.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the system model names state, actors, evidence, and invariants.
- **Adds:** ten production rules that survive framework or provider changes.
- **Prepares:** an operating-envelope decision for the Postgres-first architecture.

## Motivation

In production, mechanisms are easy to copy and hard to judge. A team can add leases, retries, approval gates, and evals without understanding which promise each mechanism protects.

Without design principles, the system becomes a checklist of tools instead of a coherent reliability model. This chapter compresses the book into rules that help you decide what to build first, what to reject, and what evidence should prove the design.

The design principles in this chapter explain why the mechanisms belong
together instead of standing as isolated implementation tricks.

## Plain Version

Read this as the simple version:

- **Simple rule:** Build the agent system by protecting promises in the right order, not by adding tools at random.
- **Why it matters:** Controls only help when they protect an invariant the system already understands.
- **What to watch:** Check whether each new feature depends on durable state, typed boundaries, ownership, idempotency, observability, or versioning.

## What You Already Know

Start with these anchors:

- The previous chapter gave the review grammar: state, actor, transition, evidence, invariant.
- A production rule is useful only when it changes a design decision.
- A reliable agent must survive crashes, provider changes, retries, and audits.

This chapter adds: ten principles that help you choose the boring, durable,
typed, observable option when a framework or model feature looks easier.

## Focus Cue

Keep three things in view:

- **State:** the ordered dependency between a principle and the production mechanism that depends on it.
- **Move:** a design advances only when the earlier invariant required by the principle is already implemented and evidenced.
- **Proof:** The principle points to chapters, artifacts, review questions, and common failure repairs.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a principle review card that maps each design rule to one concrete artifact.
- **Why it matters:** principles only help production work when they change schemas, types, tests, runbooks, or release gates.
- **Done when:** each principle has a named control and a question that catches a real failure.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** the design-principle list, Appendix J, and Appendix R traceability rows.
- **State transition:** turn each principle into a concrete schema, type, test, runbook, or release gate.
- **Evidence path:** a reviewer can name the artifact that proves the principle is not decorative.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which production artifact proves this principle changed the design?
- **Evidence to inspect:** principle review card, related chapter, implementation artifact, and reviewer question.
- **Escalate if:** a principle sounds correct but cannot point to a schema, type, test, runbook, or release gate.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a design choice is being reviewed.
2. **Action:** choose the principle that should constrain the choice.
3. **Persistence:** point to the concrete artifact the principle changes.
4. **Check:** ask whether the artifact would catch a real production failure.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** each principle points to a concrete artifact that can be reviewed.
- **Validation path:** check Appendix J and Appendix R for principle-to-artifact traceability.
- **Stop if:** a principle changes no schema, type, test, runbook, approval, or release gate.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, mechanisms are easy to copy and hard to judge
rule: Build the agent system by protecting promises in the right order, not by adding tools at random
tiny example: the ordered dependency between a principle and the production mechanism that depends on it
artifact: a principle review card that maps each design rule to one concrete artifact
proof: each principle points to a concrete artifact that can be reviewed
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Intuition

Imagine two teams building the same incident-triage agent.

The first team asks:

```text
How do we make the model answer better?
```

The second team asks:

```text
How do we make the work durable?
How do we know who owns it?
How do we prevent duplicate action?
How do we know whether the behavior version is safe?
How does an operator recover the system at 03:00?
```

The second team is doing production engineering. It still cares about model
quality, but it treats the model as one component inside a controlled system.

## Worked Walkthrough

Imagine a scenario where a user, perhaps due to a spotty network connection, accidentally sends the exact same request twice: "Analyze incident inc-123 and suggest the next action." 

In a naive architecture built entirely around simple scripts, this immediately causes a catastrophic failure. The script receives both requests, calls the expensive model twice in parallel, generates two slightly different answers, and likely triggers two separate escalation paths. The customer is confused, the system wasted resources, and the external side effects were duplicated. This happened because the script was focused entirely on model quality rather than system durability.

A reliable system, however, actively applies core design principles to survive this exact scenario. First, it enforces *durability* by writing one job row to the database before any execution begins. Then, it uses *identity* to map both incoming network requests to a single, deterministic idempotency key. When the workers attempt to process the work, *ownership* ensures that only one worker can successfully lease the job. If the provider times out during the single execution, the *boundary* principle dictates that the timeout is converted into a typed, understood retry decision rather than a raw crash. Throughout this entire process, *evidence* requires the system to record each transition in the event timeline. Before the agent takes any external action based on the model's analysis, *policy* demands human approval. Finally, when the action is taken, the *receipt* principle ensures the side effect is permanently recorded before the system can ever consider replaying the action.

These principles are not abstract philosophical ideas; they are the literal, mechanical reasons why the duplicate network request does not become duplicate work, a duplicate escalation, or a duplicate customer impact. 

## Reliable Agent Laws

Use these laws as the book's short constitution. They are not a replacement for
the mechanisms. They are a compact way to remember which mechanism should exist
before an agent is allowed to act.

| Law | Production meaning |
| --- | --- |
| The model may propose. The system must decide. | Model intention is not system permission. |
| Raw model output is not domain truth. | Parse, validate, authorize, and record before business logic trusts it. |
| Every side effect needs identity. | External action needs an idempotency key, operation record, and receipt. |
| Every important state transition must be durable. | If a transition matters after a crash, it belongs in Postgres or another durable system of record. |
| Every autonomous action needs scoped permission. | A tool call should carry tenant, user, policy, and allowed-action evidence. |
| Every human approval must produce evidence. | Approval is a durable decision record, not a message in chat. |
| Every model-dependent behavior must be versioned. | Prompt, model, tool, policy, dataset, and evaluator versions explain behavior over time. |
| Every production agent needs evals before trust. | Behavior release is a tested promotion decision, not confidence in one demo. |
| Memory must have provenance, scope, and freshness. | Retrieved facts are evidence with limits, not permanent truth. |
| If you cannot trace it, you cannot operate it. | Operators need correlated traces, metrics, events, and queryable state. |
| If you cannot audit it, you cannot trust it. | Business-significant facts require durable audit evidence. |
| A reliable agent is a probabilistic core inside a deterministic shell. | The LLM is uncertain; the surrounding system must be explicit. |

The shortest version is:

```text
The model may guess. The system must know.
```

Do not confuse model intention with system permission. A model can propose a
CRM update, email, approval recommendation, or shell command. The system still
must parse it, validate it, authorize it, record it, and decide whether human
approval is required before the side effect happens.

## Principle 1: Durable Before Intelligent

Work must exist securely before a model call ever begins. If work exists exclusively in volatile process memory, a simple service restart can permanently erase it. If the complex model returns a result just milliseconds before the database successfully stores it, a violent crash can make the entire system completely forget what just happened. Durable state is the absolute, unshakeable foundation under every single reliability claim that follows. The primary production artifact proving this is ensuring the `agent_jobs` row decisively exists in the database long before the worker is allowed to call the agent.

## Principle 2: Typed Before Clever

Your system can safely protect only the specific concepts it can explicitly name. If crucial facts like job kind, current state, attempt count, lease duration, model name, strict policy version, and failure class are passed around as raw strings or booleans at system boundaries, your code will proudly compile while the architecture remains disastrously ambiguous. Rust newtypes and strict enums are profoundly not decorative style choices; they are a mechanical way to make illegal states mathematically much harder to express. The core production artifact requires domain APIs to fiercely expose `JobKind`, `JobState`, `WorkerId`, `RetryDisposition`, and version types instead of primitives.

## Principle 3: Ownership Before Concurrency

Parallel, aggressive workers are safe only when ownership is explicitly and formally declared. A worker should never attempt to complete, blindly retry, or lazily heartbeat a job simply because it happens to know the job id. It should execute those actions only because it currently owns a mathematically valid lease on that exact job. Cancellation represents a completely separate control surface: the intention to stop work must be highly durable *before* the job is actually stopped. Rampant concurrency without strict ownership rapidly degenerates into accidental, disastrous shared mutation. The production artifact for this principle requires explicit SQL predicates enforcing `locked_by` and `locked_until` checks for all running-job transitions.

## Principle 4: Boundary Before Policy

External systems must strictly enter the architecture through dedicated adapters, absolutely never leaking their raw implementations through the core logic. Provider errors, raw model outputs, complex tool responses, and underlying database rows frequently possess incredibly messy shapes. The core worker logic should not know or care about those messy shapes. Instead, it must receive pristine, typed domain decisions: a retryable failure, a permanent failure, a validated result, an approval required flag, or a policy denied status. The key production artifact here proves that provider responses aggressively convert to an `AgentResult` or a typed failure long before the core worker policy evaluates them.

## Principle 5: Idempotent Before Retried

A retry simply repeats uncertainty, whereas idempotency mathematically makes that repetition safe. Automated retries are operationally useful only when a duplicate intention flawlessly maps to exactly one single durable action path. Without a strict idempotency key and a durable side-effect receipt, a casual retry loop can accidentally duplicate sensitive emails, support tickets, financial payments, internal approvals, or highly dangerous operational commands. The production artifact proving this ensures that a duplicate enqueue attempt merely returns the existing job, and any side-effect replay strictly checks a durable receipt before acting.

## Principle 6: Evidence Before Operations

Human operators desperately need persisted facts during an incident, not vague guesses extracted from a Slack channel. While logs are useful context, they are not the definitive authority. A serious, production-grade runbook must be able to answer any critical operational question directly from durable database rows, formal event timelines, metrics, traces, receipts, and strict version records. If the system cannot unequivocally explain exactly what happened, it cannot be operated with confidence at 3 AM. The production artifact here proves that runbook queries can flawlessly reconstruct queue health, current lease state, dead jobs, and the full event timeline.

## Principle 7: Evaluation Before Behavior Release

High system availability absolutely does not prove behavior quality. An agent can answer a query incredibly quickly and still produce unsafe, dangerously stale, or completely unsupported recommendations. Changes to prompts, models, tools, retrieval logic, and policies demand strict behavior evaluation *before* they ever become the default path for real work. The production artifact aggressively requires that specific prompt and model versions definitively possess evaluation receipts before formal promotion is allowed.

## Principle 8: Approval Is State, Not Conversation

Human approval is only reliable when it is formally recorded as database state. A chat message saying "looks good" is simply not enough for an automated system. The system strictly needs the formal proposal, the calculated risk level, the exact policy version applied, the verifiable approver identity, the timestamp, the provided reason, and the resulting state transition. Approval must be replayable, auditable evidence, not an informal side conversation. The production artifact proving this dictates that any risky action must actively wait for a formal approval record to exist in the database before side-effect execution can begin.

## Principle 9: Release With Old Work In Mind

Long-running systems always, inevitably, have old work currently in flight. When core code, database schema, complex prompts, active policies, or provider routes fundamentally change, pending and running jobs still actively exist. Release engineering for agents must therefore meticulously preserve old payloads, old versions, old decisions, and highly safe replay paths for this legacy work. The production artifact requires that the schema, prompt, model, policy, and specific worker versions are explicitly stored tightly alongside the job.

## Principle 10: Recovery Must Be Practiced

Having a backup script is emphatically not the same thing as having recovery capability. True recovery means the operations team can actually restore data, safely resume workers, effectively replay safe work, aggressively avoid duplicate side effects, and clearly explain any remaining gaps to stakeholders. The only credible proof of this capability is a formal restore drill with measured Recovery Point Objective (RPO), Recovery Time Objective (RTO), and perfectly verified replay behavior. The production artifact is the documented restore drill that meticulously records RPO, RTO, replay rules, receipt handling, and formal operator signoff.

## How The Principles Compose

The principles are ordered because each one depends on earlier ones:

```text
durable state
  -> typed concepts
  -> lease ownership
  -> provider boundary
  -> idempotent retry
  -> operational evidence
  -> behavior evaluation
  -> approval state
  -> versioned release
  -> practiced recovery
```

If a system is weak on an early principle, later controls become weaker than
they look. An SLO without durable state measures an unreliable source. A retry
without idempotency repeats risk. A restore drill without side-effect receipts
can recover data while duplicating external action.

## Formal Definition

For this chapter, the precise definition is:

```text
A design principle is a reusable ordering rule that prevents a later mechanism from depending on an earlier missing invariant.
```

In the book's system model:

- **State:** the ordered dependency between a principle and the production mechanism that depends on it.
- **Actor:** the engineer, reviewer, or operator deciding whether a design is ready to move to the next control.
- **Transition:** a design advances only when the earlier invariant required by the principle is already implemented and evidenced.
- **Evidence:** The principle points to chapters, artifacts, review questions, and common failure repairs.
- **Invariant:** later mechanisms never pretend to solve a problem whose prerequisite invariant is still missing.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Principles are slogans rather than constraints. |
| Production symptom | Teams agree with the prose but ship systems that violate it. |
| Corrective invariant | Each principle maps to an artifact that prevents one failure class. |
| Evidence to inspect | Design review rows connect principle, artifact, failure prevented, and owner. |


## Production Contract

Use these principles as a design review gate:

```text
Can every unit of work survive process restart?
Can the type system name every domain boundary?
Can only the lease owner mutate running work?
Can provider behavior be classified before retry policy sees it?
Can duplicate intent be detected before duplicate side effects happen?
Can operators reconstruct the evidence chain without process memory?
Can behavior changes prove evaluation before promotion?
Can risky action wait for recorded approval?
Can old jobs survive new releases?
Can the team restore and replay without duplicating action?
```

A design that cannot answer these questions is not necessarily bad. It is
early. The job of this book is to show how to move it toward evidence-backed
reliability.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Principles are slogans rather than constraints. | A principle list becomes slogans if it does not order real engineering decisions. |
| Safer version | Each principle maps to an artifact that prevents one failure class. | Each principle explains which earlier invariant must exist before a later control can be trusted. |
| Production version | Design review rows connect principle, artifact, failure prevented, and owner. | Design reviews can point from principle to artifact, failure class, owner, and repair path. |

Use the naive row to spot decorative principles. Use the safer row to check ordering. Use the production row when the principle must drive review questions and corrective action.

## Testing Strategy

You must test the principles as active ordering rules, never as passive slogans. In your unit or type tests, you must represent one principle in Rust as a formal review rule that explicitly rejects a design artifact when a later control appears before its prerequisite invariant. Your persistence or boundary tests must store a Postgres design-review row that durably links the principle, the specific artifact, the failure prevented, the owner, and the evidence source. Furthermore, your regression tests must encode one actively rejected shortcut—such as attempting a retry before establishing idempotency, or measuring an SLO before implementing durable events—so the review gate structurally fails if the principle ever becomes merely decorative.

## Observability Strategy

You must actively observe design principles as reviewable decisions. Emit structured `tracing` fields for the principle id, the specific job kind, the owner, the trace id, the artifact currently under review, and the blocked or accepted decision. You must record a formal operation event whenever a principle blocks a dangerous design shortcut, such as a retry before idempotency or an SLO before durable events. Ultimately, the runbook query you construct should immediately show which principle applied, which evidence was critically missing, and which human owner explicitly accepted the remaining risk.

## Security and Safety Considerations

Principles function as safety rules only when they demonstrably block unsafe shortcuts. You must treat any design proposal as inherently untrusted until it mathematically proves the prerequisite invariant named by the governing principle. Mandatory authorization, secure sandboxing, and strict human approval must be demanded by principle before risky tools, external systems, or human-controlled actions are allowed to execute. Always meticulously redact secrets and user data from review notes while keeping the violated principle, the accountable owner, and the corrective evidence perfectly visible to the team.

## Operational Checklist

Before relying on these ten design principles in production, operators must perform a strict review of the system's boundaries.

First, verify the **State** boundary: ensure each principle definitively maps to a concrete system artifact, such as a job row, a typed boundary, a lease, a receipt, an eval result, or a restore drill. Second, inspect the **Boundary** transitions themselves: verify that principles are actively checked against Rust, Postgres, and Rig boundaries instead of merely remaining as slogans in prose. 

Third, rehearse your **Failure** modes: a design review must be able to explicitly name which principle was skipped before an incident, duplicate side effect, or unsafe release occurred. Fourth, validate your **Observability** pipeline: confirm that operational events and runbook queries clearly show which principle actively protects the current transition. Finally, verify **Safety**: ensure that any principles guarding external side effects still stringently require authorization, sandboxing, approval, redaction, and retention controls.

## Exercises

To test your mastery, pick a scenario and write a negative test where a retry is accidentally added before the side effect possesses a receipt or an idempotency key. You must explicitly explain which idempotency key, receipt, or state transition prevents the duplicate work from executing. Next, sketch the exact Postgres evidence—one row or query—that proves each chosen principle has durable, inspectable evidence. Finally, define or heavily refine the Rust type, enum, constructor, or typestate that represents a Principle enum or a review checklist type that automatically rejects principles lacking evidence links. Then, meticulously name the runbook question that proves this enforcement works.

## Self-Check

Before you move on, use this quick retrieval drill to solidify your understanding. First, recall and name three specific principles that actively protect long-running agents from hidden failure. Next, be able to clearly explain why "durable before intelligent" is an enforceable design rule, not merely a motivational slogan. Then, apply this knowledge by picking one principle and explicitly describing the exact bug that inevitably appears when it is violated in code. Finally, ensure you can explicitly name the row, type, test, receipt, or runbook query that would unconditionally prove the principle is enforced.

## Summary

The mechanisms presented in this book are simply the concrete implementations of a much smaller set of fundamental principles: durable, typed, owned, bounded, idempotent, observable, evaluated, approved, versioned, and restorable.

The core invariant to remember is that a production design is fundamentally not accepted until the specific principle protecting it is undeniably visible in code, SQL, events, tests, or runbooks. To enforce this, your architecture must rely on ensuring each principle explicitly maps to a concrete artifact such as a job row, typed boundary, lease, receipt, evaluation result, or restore drill. 

Moving forward, remember the golden rule: actively use these principles as a strict review checklist whenever a design starts to feel like scattered tool selection instead of rigorous reliability engineering.

## Changed Understanding

Before reading this chapter, reliability may have simply looked like extra, tedious infrastructure bolted on after the model already "works." After this chapter, you should understand that reliability is a strict, upfront design discipline: you must be durable before you are intelligent, typed before you are clever, and observable before you are trusted. Moving forward, keep in mind that you must use this design-principle checklist ruthlessly before accepting any new agent feature.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
g forward, keep in mind that you must use this design-principle checklist ruthlessly before accepting any new agent feature.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
