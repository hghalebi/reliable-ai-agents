# Design Principles

## What You Will Learn

This chapter teaches you to:

This chapter gives you the small set of rules that hold the rest of the book
together. Frameworks change. Model providers change. Hosting platforms change.
The principles in this chapter should still help you decide whether a design is
getting safer or merely getting more complicated.

By the end, you should be able to explain why the principles are ordered,
inspect a design decision for the principle it protects or violates, and verify
the production evidence behind the claim. You should be able to look at a proposed agent feature and ask a
plain question: which production promise does this protect? If the answer is
"it sounds modern," the design is not ready yet. If the answer points to a row,
a type, a receipt, a test, a runbook query, or a release gate, the principle has
become engineering.

The production evidence is a principle-to-artifact map that connects durable
state, typed boundaries, ownership, idempotency, observability, approval,
evaluation, release safety, and recovery practice.

## Chapter Thread

Read this chapter as one link in the production chain:

The previous chapter gave us a vocabulary: state, actor, transition, evidence,
and invariant. This chapter turns that vocabulary into judgment. It adds ten
rules for deciding whether a mechanism belongs in the system and what proof
should come with it.

**Builds on:** the system model names state, actors, evidence, and invariants.
**Adds:** ten production rules that survive framework or provider changes.
**Prepares:** an operating-envelope decision for the Postgres-first architecture.
The next chapters will keep returning to these rules. When we later discuss Postgres, Rust types, workers, Rig, approvals, evals, recovery, Kafka, or Temporal, the same question appears in a slightly different coat: what promise is this mechanism protecting?

## Motivation

In production, mechanisms are easy to copy and hard to judge. A team can add leases, retries, approval gates, and evals without understanding which promise each mechanism protects.

Without design principles, the system becomes a checklist of tools instead of a coherent reliability model. This chapter compresses the book into rules that help you decide what to build first, what to reject, and what evidence should prove the design.

The design principles in this chapter explain why the mechanisms belong
together instead of standing as isolated implementation tricks.

## Plain Version

Read this as the simple version:

The simple rule is this: build the agent system by protecting promises in the
right order, not by adding tools at random.

**Simple rule:** build the agent system by protecting promises in the right
order, not by adding tools at random. **Why it matters:** a control is useful
only when it protects an invariant the system already understands. **What to watch:** a retry policy is not useful before idempotency, a dashboard is not
very useful before durable state, and human approval is fragile if it lives only
in a chat thread. The order matters because production bugs are often just
missing prerequisites wearing a nice jacket.

## What You Already Know

Start with these anchors:

- The system model names state, actor, transition, evidence, and invariant.
- A reliable agent must survive crashes, provider changes, retries, and audits.
- A production rule is useful only when it changes a design decision.

You already know the review grammar from the system model: state, actor,
transition, evidence, and invariant. You also know the central pressure of this
book: a reliable agent must survive crashes, provider changes, retries, and
audits.

Now we add a stricter standard. A production rule is useful only if it changes a
design decision. If a principle cannot change code, SQL, tests, runbooks, or
release gates, it is still only a sentence.

This chapter adds: ten principles that help you choose the boring, durable,
typed, observable option when a framework or model feature looks easier.

## Focus Cue

Keep three things in view:

Keep three things in view: state, move, and proof.

**State:** the dependency between a principle and the mechanism that depends on
it. **Move:** the design step that becomes legal only after the earlier invariant
exists. **Proof:** the artifact another engineer can inspect.

If you get lost, return to those three words. They are less glamorous than a new
tool, but they are easier to operate at 03:00.


## Production Artifact

Build or inspect this artifact before moving on:

**Artifact:** a principle review card maps each design rule to one concrete
artifact. The artifact may be a schema constraint, Rust type, lease predicate,
idempotency record, evaluation receipt, approval row, runbook query, or restore
drill.

**Why it matters:** principles only help production work when they change
schemas, types, tests, runbooks, or release gates. **Done when:** each principle
has a named control and a question that would catch a real failure. If the
review card cannot make a risky design awkward, it is decorative paperwork.


## Implementation Map

Use this map when you move from reading to implementation:

When you move from reading to implementation, start with the principle list and
the traceability appendices. For each principle, choose one system surface where
the rule becomes visible: a schema, a type, a test, a runbook, or a release
gate. The state transition is the moment the principle stops being advice and
becomes a check.

**Primary surface:** the design-principle list, Appendix J, and Appendix R
traceability rows. **State transition:** turn each principle into a concrete
schema, type, test, runbook, or release gate. **Evidence path:** a reviewer can
name the artifact that proves the principle is not decorative.


## Operator Question

Before you ship this idea, answer one operational question:

Before shipping a design that claims to follow these principles, ask one
operational question: which production artifact proves this principle changed
the design?

**Question:** which production artifact proves this principle changed the
design? **Evidence to inspect:** principle review card, related chapter,
implementation artifact, and reviewer question. **Escalate if:** a principle
sounds correct but cannot point to a schema, type, test, runbook, approval,
receipt, or release gate. Production systems are very polite about slogans; they
fail anyway.


## Runtime Walkthrough

Follow the concept as one runtime pass:

During a design review, a choice appears: perhaps a retry policy, a new tool, or
a model upgrade. The reviewer chooses the principle that should constrain that
choice and asks for the artifact the principle changes. Then the team checks the
important question: would this artifact catch a real production failure?

**Trigger:** a design choice is being reviewed. **Action:** choose the principle
that should constrain the choice. **Persistence:** point to the concrete
artifact the principle changes. **Check:** ask whether the artifact would catch
a real production failure.

If the answer is yes, the principle is alive. If the answer is no, the system is
collecting inspirational posters.


## Acceptance Gate

Do not move on until this minimum evidence exists:

Do not move on until each principle points to a concrete artifact that can be
reviewed. Appendix J and Appendix R provide the traceability path, but the real
standard is simpler: another engineer should be able to find the artifact and
explain the failure it prevents.

**Minimum evidence:** each principle points to a concrete artifact that can be
reviewed. **Validation path:** check Appendix J and Appendix R for
principle-to-artifact traceability. **Stop if:** a principle changes no schema,
type, test, runbook, approval, receipt, or release gate. That is not a
production principle yet. It is a bumper sticker, and production has no respect
for bumper stickers.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

Here is the compact version before the heavier mechanism. Production failures
often happen because teams copy mechanisms without knowing the promises those
mechanisms protect. The rule is to build by protecting promises in the right
order. The tiny example is a duplicate incident request that becomes one job
because durable state, identity, and ownership already exist. The artifact is a
principle review card. The proof is that each principle points to a concrete
artifact another engineer can inspect.

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

## Tiny Scenario

Imagine a scenario where a user, perhaps due to a spotty network connection, accidentally sends the exact same request twice: "Analyze incident inc-123 and suggest the next action." 

In a naive architecture built entirely around simple scripts, this immediately causes a catastrophic failure. The script receives both requests, calls the expensive model twice in parallel, generates two slightly different answers, and likely triggers two separate escalation paths. The customer is confused, the system wasted resources, and the external side effects were duplicated. This happened because the script was focused entirely on model quality rather than system durability.

A reliable system, however, actively applies core design principles to survive this exact scenario. First, it enforces *durability* by writing one job row to the database before any execution begins. Then, it uses *identity* to map both incoming network requests to a single, deterministic idempotency key. When the workers attempt to process the work, *ownership* ensures that only one worker can successfully lease the job.

If the provider times out during the single execution, the *boundary* principle dictates that the timeout is converted into a typed, understood retry decision rather than a raw crash. Throughout this entire process, *evidence* requires the system to record each transition in the event timeline. Before the agent takes any external action based on the model's analysis, *policy* demands human approval.

Finally, when the action is taken, the *receipt* principle ensures the side effect is permanently recorded before the system can ever consider replaying the action.

These principles are not abstract philosophical ideas; they are the literal, mechanical reasons why the duplicate network request does not become duplicate work, a duplicate escalation, or a duplicate customer impact. 

Read the tiny case as:

```text
setup: a user accidentally sends the exact same incident triage request twice under a spotty network connection
transition: the system maps both requests to a single idempotency key, enqueues a single job row, and leases it to a single worker
evidence: a single durable job row in the database, with locked_by and locked_until set, along with a side-effect receipt
invariant: only one worker can process the unique lease, ensuring no duplicate external action or duplicate model call runs
```

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

Work must exist before a model call begins.

If work exists only in process memory, a restart can erase it. If the model
returns before the result is stored, a crash can make the system forget what
happened. Durable state is the foundation under every later reliability claim.

Production artifact:

```text
agent_jobs row exists before the worker calls the agent
```

## Principle 2: Typed Before Clever

The system can protect only the concepts it can name.

If job kind, state, attempt count, lease duration, model name, policy version,
and failure class are raw strings or booleans at boundaries, the code can
compile while the architecture is ambiguous. Newtypes and enums are not style
decoration. They are a way to make illegal states harder to express.

Production artifact:

```text
domain APIs expose JobKind, JobState, WorkerId, RetryDisposition, and version types
```

## Principle 3: Ownership Before Concurrency

Parallel workers are safe only when ownership is explicit.

A worker should not complete, retry, or heartbeat a job because it knows the
job id. It should do those actions only because it owns the current lease.
Cancellation is a separate control surface: the stop intent should be durable
before the job is stopped. Concurrency without ownership becomes accidental
shared mutation.

Production artifact:

```text
SQL predicates require locked_by and locked_until for running-job transitions
```

## Principle 4: Boundary Before Policy

External systems should enter through adapters, not leak through the core.

Provider errors, model outputs, tool responses, and database rows often have
messy shapes. The core worker should not know those shapes. It should receive
domain decisions: retryable failure, permanent failure, validated result,
approval required, or policy denied.

Production artifact:

```text
provider responses convert to AgentResult or typed failure before worker policy
```

## Principle 5: Idempotent Before Retried

Retry repeats uncertainty. Idempotency makes repetition safe.

Retries are useful only when duplicate intent maps to one durable action path.
Without an idempotency key and side-effect receipt, retry can duplicate emails,
tickets, payments, approvals, or operational commands.

Production artifact:

```text
duplicate enqueue returns the existing job and side-effect replay checks a receipt
```

## Principle 6: Evidence Before Operations

Operators need persisted facts, not guesses.

Logs are useful, but they are not the authority. A serious runbook should be
able to answer questions from durable rows, event timelines, metrics, traces,
receipts, and version records. If the system cannot explain what happened, it
cannot be operated with confidence.

Production artifact:

```text
runbook queries reconstruct queue health, lease state, dead jobs, and event timeline
```

## Principle 7: Evaluation Before Behavior Release

Availability does not prove behavior quality.

An agent can answer quickly and still produce unsafe, stale, or unsupported
recommendations. Prompt, model, tool, retrieval, and policy changes need
behavior evaluation before they become the default path for real work.

Production artifact:

```text
prompt and model versions have evaluation receipts before promotion
```

## Principle 8: Approval Is State, Not Conversation

Human approval is reliable only when it is recorded.

A chat message saying "approved" is not enough. The system needs the proposal,
risk level, policy version, approver, timestamp, reason, and resulting state
transition. Approval should be replayable evidence, not an informal side
conversation.

Production artifact:

```text
risky action waits for an approval record before side-effect execution
```

## Principle 9: Release With Old Work In Mind

Long-running systems always have old work.

When code, schema, prompts, policies, or provider routes change, pending and
running jobs still exist. Release engineering for agents must preserve old
payloads, old versions, old decisions, and safe replay paths.

Production artifact:

```text
schema, prompt, model, policy, and worker versions are stored with the job
```

## Principle 10: Recovery Must Be Practiced

Backup is not recovery.

Recovery means the team can restore data, resume workers, replay safe work,
avoid duplicate side effects, and explain the remaining gaps. The only credible
proof is a restore drill with measured RPO, RTO, and replay behavior.

Production artifact:

```text
restore drill records RPO, RTO, replay rules, receipt handling, and operator signoff
```

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

In the book's system model, **State** is the dependency between a principle and
the production mechanism that depends on it. **Actor** is the engineer,
reviewer, or operator deciding whether a design is ready to move forward.
**Transition** happens when the design advances only after the prerequisite
invariant has been implemented and evidenced.

**Evidence** is not a belief that the team is careful. It is a chapter,
artifact, review question, test, row, receipt, or runbook. **Invariant** means
later mechanisms never pretend to solve a problem whose prerequisite invariant
is still missing.

## What Can Fail

**Design smell:** the most common failure is not disagreement. Almost everyone
agrees with phrases like "make it observable" or "use idempotency." The failure
is that the principle remains a slogan rather than a constraint.

**Production symptom:** a team agrees with the prose and still ships a system
that violates it. **Corrective invariant:** each principle maps to an artifact
that prevents one failure class. **Evidence to inspect:** a design review row,
or its moral equivalent, connecting principle, artifact, failure prevented, and
owner.


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

**Naive version:** a list of principles that everyone likes and nobody can
enforce. It may be useful for a workshop, but it will not stop a duplicate side
effect or a risky model release.

**Safer version:** each principle maps to an artifact that prevents one failure
class. "Idempotent before retried" points to an idempotency key and receipt.
"Typed before clever" points to newtypes, enums, and conversion tests.
"Recovery must be practiced" points to a restore drill.

**Production version:** principle, artifact, failure class, owner, and repair
path are connected in the design review process. At that point the principle has
a job: it can block a shortcut before the shortcut becomes an incident.

## Testing Strategy

You must test the principles as active ordering rules, never as passive slogans. In your unit or type tests, you must represent one principle in Rust as a formal review rule that explicitly rejects a design artifact when a later control appears before its prerequisite invariant. Your persistence or boundary tests must store a Postgres design-review row that durably links the principle, the specific artifact, the failure prevented, the owner, and the evidence source.

Furthermore, your regression tests must encode one actively rejected shortcut—such as attempting a retry before establishing idempotency, or measuring an SLO before implementing durable events—so the review gate structurally fails if the principle ever becomes merely decorative. **Unit:** test the smallest typed transition and the invariant it preserves. **Persistence:** test the database row, query, or receipt that proves the transition survives process death. **Regression:** keep a failing case for the production bug this chapter is designed to prevent.

## Observability Strategy

You must actively observe design principles as reviewable decisions. Emit structured `tracing` fields for the principle id, the specific job kind, the owner, the trace id, the artifact currently under review, and the blocked or accepted decision. You must record a formal operation event whenever a principle blocks a dangerous design shortcut, such as a retry before idempotency or an SLO before durable events. Ultimately, the runbook query you construct should immediately show which principle applied, which evidence was critically missing, and which human owner explicitly accepted the remaining risk.

## Security and Safety Considerations

Principles function as safety rules only when they demonstrably block unsafe shortcuts. You must treat any design proposal as inherently untrusted until it mathematically proves the prerequisite invariant named by the governing principle. Mandatory authorization, secure sandboxing, and strict human approval must be demanded by principle before risky tools, external systems, or human-controlled actions are allowed to execute. Always meticulously redact secrets and user data from review notes while keeping the violated principle, the accountable owner, and the corrective evidence perfectly visible to the team.
Redact secrets, tenant data, prompts, and private payloads while preserving ids, state names, and evidence references for audit.

## Operational Checklist

Before relying on these ten design principles in production, operators must perform a strict review of the system's boundaries.

First, verify the **State** boundary: ensure each principle definitively maps to a concrete system artifact, such as a job row, a typed boundary, a lease, a receipt, an eval result, or a restore drill. Second, inspect the **Boundary** transitions themselves: verify that principles are actively checked against Rust, Postgres, and Rig boundaries instead of merely remaining as slogans in prose. 

Third, rehearse your **Failure** modes: a design review must be able to explicitly name which principle was skipped before an incident, duplicate side effect, or unsafe release occurred. Fourth, validate your **Observability** pipeline: confirm that operational events and runbook queries clearly show which principle actively protects the current transition. Finally, verify **Safety**: ensure that any principles guarding external side effects still stringently require authorization, sandboxing, approval, redaction, and retention controls.

## Exercises

To test your mastery, pick one scenario and make the principle do real work.
For example, write a negative test where a retry is added before the side effect
has a receipt or idempotency key. Explain which key, receipt, or state
transition prevents duplicate work.

Then sketch the exact Postgres evidence, such as one row or query, proving that
the chosen principle is durable and inspectable. Finally, define or refine a
Rust type, enum, constructor, or typestate value that represents the review rule
and rejects missing evidence links. Finish by naming the runbook question that
would prove the enforcement works during an incident.

1. Name one invalid transition this chapter should prevent and write the evidence that proves it is blocked.
2. Sketch the durable Postgres row, event, receipt, or policy record that would prove the correct behavior.
3. Add or describe one Rust negative test, type, enum, constructor, or typestate state that makes the production rule harder to violate.

## Self-Check

Before you move on, do a short retrieval drill. **Recall:** name three
principles that protect long-running agents from hidden failure. **Explain:**
why is "durable before intelligent" an enforceable design rule rather than a
motivational slogan? **Apply:** pick one principle and describe the exact bug
that appears when code violates it.

**Evidence:** name the row, type, test, receipt, or runbook query that
proves the principle is enforced. If you cannot name the evidence, the principle
is not ready to carry production weight.


## Summary

The mechanisms presented in this book are simply the concrete implementations of a much smaller set of fundamental principles: durable, typed, owned, bounded, idempotent, observable, evaluated, approved, versioned, and restorable.

The core invariant to remember is that a production design is fundamentally not accepted until the specific principle protecting it is undeniably visible in code, SQL, events, tests, or runbooks. To enforce this, your architecture must rely on ensuring each principle explicitly maps to a concrete artifact such as a job row, typed boundary, lease, receipt, evaluation result, or restore drill. 

Moving forward, remember the golden rule: actively use these principles as a strict review checklist whenever a design starts to feel like scattered tool selection instead of rigorous reliability engineering.

**Invariant:** the chapter concept must preserve its named production rule under failure.

**Evidence:** the proof must be visible as a row, event, receipt, trace, policy, test, or runbook query.

## Changed Understanding

Before reading this chapter, reliability may have simply looked like extra, tedious infrastructure bolted on after the model already "works." After this chapter, you should understand that reliability is a strict, upfront design discipline: you must be durable before you are intelligent, typed before you are clever, and observable before you are trusted. Moving forward, keep in mind that you must use this design-principle checklist ruthlessly before accepting any new agent feature.

**Before this chapter:** the mechanism may have looked like an implementation
detail. **After this chapter:** the mechanism is a production contract with
evidence. **Keep:** name the invariant, the evidence, and the operator question
before relying on any new control.


## Further Reading and Sources



- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: (2019). Grounded in Cognitive Load Theory, this book argues that software boundaries should be designed to fit within human working-memory limits—the "Typed Before Clever" and "Pedagogical" mission of this book.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: Provides the industry-standard foundation for the principles of observability, automation, and release safety discussed in this chapter.
- [Anthropic: Building Effective Agents](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: (2025). The industry guide for the "Durable Before Intelligent" principle, emphasizing deterministic scaffolds over model-only autonomy.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann). Connects these principles to broader distributed systems invariants.
