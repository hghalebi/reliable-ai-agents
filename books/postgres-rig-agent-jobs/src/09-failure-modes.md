# 9. Failure Modes

## What You Will Learn

This chapter teaches you to:

- explain the common ways agent job systems become unsafe;
- inspect each failure for hidden state, duplicate side effects, lease loss, untyped output, or missing evidence;
- verify that failures become visible states instead of silent loops.

The production evidence is a failure-mode map that ties each design smell to a
state, corrective invariant, diagnostic query, and regression test.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the hardening controls have names and evidence.
- **Adds:** a failure taxonomy for lost ownership, sequence, authority, and terminality.
- **Prepares:** the capstone pattern for extending the system without losing invariants.

## Production Failure

Three incidents look unrelated: one duplicate email, one stuck job, and one
unsafe tool proposal.

The team patches each symptom, but all three came from missing identity,
ownership, authority, or terminal state.

- **What breaks:** failures were treated as isolated bugs instead of missing
  invariants.
- **False fix:** add special cases for each incident and hope the next one has
  a familiar shape.
- **Design response:** classify failures by the invariant they lost, then add
  state, evidence, tests, and runbook checks for that class.

## Motivation

In production, failure is not exceptional. Providers time out, tools return bad data, workers crash, policies deny actions, and humans answer late.

Without a failure model, the system either retries blindly or gives up without evidence. This chapter names the common failure modes so each one becomes visible state with a safe next action.

## Plain Version

The simple rule for this chapter is that an architectural failure must become a concrete state the system can explicitly inspect, never a surprise hidden in scattered log lines. This matters deeply because reliable agents require meticulously planned behavior for hard crashes, unpredicted provider errors, bad model output, inherently unsafe tools, and permanently stuck work. As you read, aggressively watch whether each potential failure mode explicitly possesses a defined owner, a safe next action, a strict retry rule, a clear escalation path, and undeniable, durable database evidence.
Read this as the simple version:
- **Simple rule:** name the invariant before trusting the mechanism.
- **Why it matters:** vague reliability claims fail during incidents.
- **What to watch:** the proof must be a row, type, event, receipt, or runbook check.


## What You Already Know

Start by anchoring yourself in the concepts you have already built. You know that the main hardening controls are now visible and enforceable within the architecture. You also know that most severe agent failures are simply missing facts: missing identity, lost ownership, broken sequence, bypassed authority, or undefined terminality. Furthermore, you understand that a hidden failure cannot be safely retried, securely escalated, or mechanically repaired. 

This chapter adds the crucial skill of formal failure-mode diagnosis. You will learn to map any chaotic production symptom directly back to the specific missing invariant and identify the exact database evidence that should have prevented the failure in the first place.
Start with these anchors:

- Durable state is the first production boundary.
- Typed values make production meaning explicit.
- Evidence must survive process death.

This chapter adds: one more production mechanism that can be inspected, tested, and operated.


## Focus Cue

Keep three critical elements in view as you read. Regarding **State**, recognize the known, predictable ways the system can inadvertently lose ownership, identity, evidence, permission, terminality, or behavior quality. Regarding the **Move**, understand that a production symptom only becomes a reliable repair plan after both the missing invariant and the missing evidence are formally named. Finally, regarding **Proof**, remember that the overall system design must mathematically map each failure to a strict corrective invariant and highly inspectable database evidence. 

If you ever get lost in the taxonomy, immediately return to state, move, and proof. They form the absolute shortest path from abstract failure analysis to a concrete production check.
Keep three things in view:
- **State:** the production fact that changes.
- **Move:** the lawful transition from one state to another.
- **Proof:** the evidence an operator can inspect later.


## Production Artifact

Before moving on from this chapter, you must build or rigorously inspect a formal failure-mode table. This artifact maps each architectural design smell directly to a production symptom, a corrective invariant, and a concrete evidence query. This artifact matters intensely because production teams desperately need fast, reliable recognition of broken designs long before vague incidents turn into prolonged, unsolvable mysteries. You will know this record is "done" when every single named failure explicitly possesses a defined detection path, a corrective invariant, and an owner-facing diagnostic question.
Build or inspect this artifact before moving on:
- **Artifact:** the concrete row, type, policy, receipt, or runbook query for this chapter.
- **Why it matters:** learning becomes production skill only when it changes an inspectable artifact.
- **Done when:** another engineer can inspect the artifact and explain the invariant it protects.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** Appendix P, failure drill fixtures, worker tests, and runbook SQL.
- **State transition:** translate design smells into symptoms, corrective invariants, and evidence checks.
- **Evidence path:** a failure is diagnosable from rows, events, traces, and owner questions.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which invariant failed, and what evidence proves the failure mode?
- **Evidence to inspect:** design smell, production symptom, corrective invariant, failing row, and diagnostic query.
- **Escalate if:** the team can name a symptom but not the invariant or evidence that would prevent recurrence.


## Runtime Walkthrough

Follow the concept as one runtime pass:

1. **Trigger:** a symptom appears in jobs, traces, or operator reports.
2. **Action:** map the symptom to a design smell and failed invariant.
3. **Persistence:** record the diagnostic evidence and corrective invariant.
4. **Check:** verify the fix changes the boundary that allowed the failure.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** each design smell maps to symptom, corrective invariant, and evidence.
- **Validation path:** check Appendix P, failure drills, runbook SQL, and chapter-specific failure rows.
- **Stop if:** a failure can be named but not diagnosed or prevented by a stronger invariant.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, failure is not exceptional
rule: A failure should become a state the system can inspect, not a surprise hidden in logs
tiny example: known ways the system can lose ownership, identity, evidence, permission, terminality, or behavior quality
artifact: a failure-mode table that maps each design smell to a symptom, invariant, and evidence query
proof: each design smell maps to symptom, corrective invariant, and evidence
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Worked Walkthrough

Failure analysis starts by refusing vague labels.
"The agent failed" is not a useful diagnosis.
It hides the layer that broke and therefore hides the fix.
Ask what kind of failure occurred.
Did the model produce malformed output?
Did the parser reject valid-looking text?
Did the policy gate deny a risky action?
Did the worker lose its lease?
Did the external tool succeed while the local database write failed?

In **Evaluation-Driven Development**, we go a step further. We don't just ask *if* it failed, we ask **"Was the failure expected given the input?"** If a model fails to summarize a 100-page document because of context limits, that's an **Expected Limit** or **Resource Exhaustion**, not a bug. We should add a category for these exhaustion events to our failure taxonomy.

Each answer points to a different repair.
Malformed model output needs parsing and validation.
A lost lease needs ownership checks.
A duplicated side effect needs idempotency and receipts.
A wrong approval decision needs policy evidence and review history.

> ### 🎓 The Professor's Corner
>
> **The Five Whys: The Detective Game**
>
> When something breaks, don't just fix the surface! Play the "Five Whys" game.
> 1. Why did the email send twice? (The worker retried)
> 2. Why did it retry? (It thought it failed)
> 3. Why did it think it failed? (The database write timed out)
> 4. Why did that matter? (There was no receipt from the email API)
> 5. Why was there no receipt? (**Aha!** The code didn't save it first!)
> 
> Keep asking "Why?" until you reach the **Lost Fact**. That's the real bug!

The goal is not to memorize a list of failures.
The goal is to learn how to translate a production symptom into an architectural weakness.
Once the weakness is named, the fix becomes testable.

## Mental Model
This is also why the book separates logs, metrics, traces, operation events, and audit events.
Each evidence type answers a different question.
Together they let the operator reconstruct what happened without asking the model to explain itself after the fact.

## Mental Model

Every failure should answer two questions:

```text
Can this job safely run again?
What evidence will tell an operator what happened?
```

If the answer to either question is unclear, the system needs a stronger state
model, boundary, or event.

This is the diagnostic habit to build.

Do not start by asking, "Which line of code failed?" Start by asking, "Which fact
did the system lose?" A lost fact is usually more important than the immediate
exception. The exception tells you where the pain appeared. The lost fact tells
you why the system could not recover cleanly.

For example, a duplicate email is not only an email bug. It may be a lost
identity bug because the system could not recognize a repeated logical request.
A stuck job is not only a worker bug. It may be a lost ownership bug because no
worker can prove whether it still owns the work. A tool that runs without review
is not only a policy bug. It may be a lost authority bug because the system let a
proposal become an action.

The repair should restore the missing fact in a durable place.

## Failure Classification Lens

Use this lens before adding a fix:

```text
lost ownership:
  no worker can prove whether it may continue

lost identity:
  duplicate requests cannot be collapsed

lost sequence:
  the final state exists but the path is unknowable

lost authority:
  the model or worker acted without a policy decision

lost terminality:
  the system keeps retrying work that cannot succeed
```

This lens keeps the fix architectural. If the failure is lost ownership, a
conditional in the worker is not enough; the lease model needs to be correct.
If the failure is lost identity, adding more logs is not enough; the enqueue
boundary needs idempotency.

The lens also protects the team from scattered patches.

Scattered patches usually match the symptom. They say, "If this provider returns
this message, do that." A failure-mode repair matches the invariant. It says,
"Every externally triggered side effect must have identity" or "Every worker
mutation must prove current ownership."

The second kind of fix is slower to design, but it changes the shape of future
bugs. The next provider, tenant, worker, or tool must pass through the same
boundary. That is how one incident improves the system instead of adding another
special case.

## Common Mistakes

No lease:

```text
worker crashes -> job stays running forever
```

No idempotency:

```text
client retry -> duplicate job -> duplicate action
```

No events:

```text
failure happened -> nobody knows where
```

Agent performs side effects directly:

```text
model mistake -> email/payment/deployment already happened
```

Infinite retry:

```text
bad input -> job burns resources forever
```

Each mistake has the same shape: the system lost a fact that production needs.
The missing fact might be ownership, identity, history, permission, or terminal
failure.

In distributed systems, this is how you protect the **System Capacity** from being consumed by "Zombie Jobs." If we don't classify the failure, the system will never reach a terminal state, and poisoned inputs will eventually drown out all legitimate work.

The useful question is not "How do we make this error go away?"

The useful question is "What fact should have made this error safe?" If the
worker crashes, the lease should make ownership recoverable. If the client
retries, the idempotency key should make duplication harmless. If the model asks
for a risky tool, the policy and approval records should make authority explicit.
If a job can never succeed, the terminal state should stop the retry loop.

> ### 🎓 The Professor's Corner
>
> **The Dead-Letter Office: The Post Office Analogy**
>
> Think of a **Dead-Letter Office** (the DLO) just like a real post office. If a letter can't be delivered, they don't just throw it away! They put it in a special bin for a human to look at later. 
> 
> In our system, if a job fails in a way we don't understand, we put it in the "Dead-Letter" state. It stays in our notebook as durable evidence, waiting for an operator to act as the final arbiter. This ensures we never lose a message, even when we're confused!

This is the difference between debugging a process and improving a system.
Process debugging fixes the current run. System debugging changes the invariant
so the next run is safer.

## Tiny Example

A billing action job calls an external payment API and then the worker crashes
before writing a result.

If the model or worker performed the side effect directly, retry is ambiguous:

```text
Did the payment happen?
Should the worker call the API again?
Where is the receipt?
```

The safe design separates the proposal from the side effect and stores a
receipt at the side-effect boundary. Then replay can ask a concrete question:

```text
Does a receipt already exist for this idempotency key?
```

That question is the difference between recovery and guessing. A production
system should prefer slower recovery with evidence over fast recovery that
might duplicate an external action.

Read the tiny case as:

```text
setup: an external side effect may have happened before the worker crashed
transition: the system must stop, inspect receipts, and choose replay or reconciliation
evidence: missing or present side-effect receipt changes the recovery path
invariant: ambiguous side effects must not be retried blindly
```

## Better Pattern

```text
agent proposes
system records
policy decides
human approves when needed
separate worker executes
events explain every transition
```

The better pattern is slower than a one-file script because it has more
checkpoints. Those checkpoints are the reason it can recover without guessing.

Each checkpoint answers a specific recovery question.

The recorded proposal answers, "What did the model suggest?" The policy decision
answers, "Was the action allowed?" The approval record answers, "Who or what
accepted the risk?" The side-effect receipt answers, "Did the external action
happen?" The event timeline answers, "In what order did these facts appear?"

When those questions have durable answers, recovery can be cautious and precise.
When they do not, the operator is forced to choose between retrying blindly,
giving up, or investigating by hand.

## Formal Definition

For this chapter, the precise definition is:

```text
A failure mode is a predictable way the system can lose ownership, identity, evidence, permission, or terminality.
```

In the book's system model:

- **State:** known ways the system can lose ownership, identity, evidence, permission, terminality, or behavior quality.
- **Actor:** workers, reviewers, and operators classify symptoms against failure modes and choose corrective invariants.
- **Transition:** a production symptom becomes a repair plan only after the missing invariant and evidence are named.
- **Evidence:** The design maps each failure to a corrective invariant and inspectable evidence.
- **Invariant:** failure analysis leads to stronger state and boundaries rather than scattered conditionals.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | Failures are handled by logging and continuing. |
| Production symptom | Jobs vanish into retry loops or fail without operator-readable state. |
| Corrective invariant | Failures become classified state transitions. |
| Evidence to inspect | Dead jobs, retry events, permanent errors, and policy stops can be queried. |


## Production Contract

For every failure mode in this chapter, the system should preserve one of these
facts:

```text
ownership:
  who may act on the job right now

identity:
  which logical request this work represents

timeline:
  how the job reached the current state

permission:
  whether side effects are allowed

terminality:
  whether retry is useful or harmful
```

Treat this list as a quick design review.

For any new feature, ask whether a failure can erase one of these facts. If the
feature calls an external API, identity and timeline matter. If it runs in a
worker, ownership matters. If it changes a customer-visible system, permission
matters. If it can fail repeatedly, terminality matters.

The right answer is rarely "add more logging." Logging can help, but only after
the system has a durable fact to log. The state model, database constraint,
event, receipt, or approval record is the thing that makes the failure
operable.

In a reliable agent system, failure is allowed. Ambiguous failure is the enemy.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Failures are handled by logging and continuing. | Logging and continuing hides whether the system lost identity, ownership, evidence, permission, or terminality. |
| Safer version | Failures become classified state transitions. | Classified failure modes map each symptom to a state transition and corrective invariant. |
| Production version | Dead jobs, retry events, permanent errors, and policy stops can be queried. | Dead-letter queries, retry events, permanent errors, and policy stops give operators an inspectable failure map. |

Use the naive row when failure is just an error string. Use the safer row to classify the broken invariant. Use the production row before operators need to act under pressure.

## Testing Strategy

Test failure classification instead of error-string logging:

- **Unit or type test:** prove Rust failure enums separate transient, permanent, exhausted, cancelled, policy-denied, and malformed-provider cases.
- **Persistence or boundary test:** prove Postgres failure-history rows keep class, source, outcome, attempt budget, retry time, and trace context.
- **Regression test:** replay a known failure mode and verify it becomes the documented state transition rather than a generic logged error.

## Observability Strategy

Observe failures by class and corrective invariant:

- Emit structured `tracing` fields for failure source, failure class, job id, run id, attempt, trace id, retry state, and terminal outcome.
- Record an operation event and failure-history row whenever a transient, permanent, exhausted, cancelled, denied, or malformed-provider failure occurs.
- The runbook query should connect the production symptom to the broken invariant and the state transition that repaired or stopped it.

## Security and Safety Considerations

Failure handling must not become an unsafe fallback path:

- Treat failure payloads, retry reasons, provider messages, and operator notes as untrusted data until classified and sanitized.
- authorization, sandboxing, and approval must still apply during retries, manual recovery, compensation, and incident mitigation.
- Redact sensitive error details while preserving failure source, class, attempt, outcome, and corrective invariant evidence.

## Operational Checklist

Use this checklist before relying on visible failure instead of hidden loops in production:

- **State:** Each failure mode maps to retryable, permanent, exhausted, cancelled,
  escalated, or compensated state.
- **Boundary:** Provider errors, model errors, database errors, policy denials, and tool
  failures are classified before retry logic runs.
- **Failure:** The system avoids silent retries, swallowed errors, duplicate side
  effects, and unowned stuck work.
- **Observability:** Failure history, dead-letter rows, operation events, and trace ids
  reconstruct the failure path.
- **Safety:** Unsafe failures stop or escalate instead of letting the model invent
  recovery actions.

## Exercises

1. Write a negative test where a provider timeout is retried with idempotency while a
   policy denial becomes terminal without tool execution. Explain which idempotency key,
   receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: failure_history and dead_jobs_by_reason evidence for
   one transient and one permanent failure.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   FailureKind, RetryDisposition, DeadLetterReason, and EscalationReason types. Then
   name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which missing facts cause most agent job failures?
- Explain: Why is a hidden loop more dangerous than a visible failed state?
- Apply: Classify a crash after an external API call but before a receipt is written.
- Evidence: Name the missing fact, corrective invariant, row or receipt, and regression test that should expose the bug.

## Summary

A production agent should fail visibly, retry intentionally, and stop safely when the error is no longer recoverable.

- **Invariant:** every failure becomes retryable, permanent, exhausted, cancelled, escalated, compensated, or terminal state.
- **Evidence:** failure history, dead-letter rows, retry schedules, cancellation records, trace ids, and operator queries reconstruct the failed path.
- **Carry forward:** hidden retries and swallowed errors are not resilience; they are lost evidence.

## Changed Understanding

- **Before this chapter:** failures looked like exceptional cases around the normal design.
- **After this chapter:** failures are design inputs; each one should point to an invariant the system can enforce or observe.
- **Keep:** record each failure as design smell, symptom, corrective invariant, and evidence to inspect.

## Further Reading and Sources



- [Eric Brewer: CAP Twelve Years Later](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: The academic foundation for why "Lost Ownership" (Consistency) and "Lost Terminality" (Availability) are fundamental trade-offs in distributed systems.
- [Google SRE book introduction](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: The industry standard for turning symptoms into "Corrective Invariants"—the exact process described in this chapter's taxonomy.
- [Abadi: PACELC Theorem](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: Formal research on how latency (during heartbeats) and consistency (during leases) interact during failure modes.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann). Provides the vocabulary for "Split-Brain" and "Silent Corruption" failure modes. latency (during heartbeats) and consistency (during leases) interact during failure modes.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann). Provides the vocabulary for "Split-Brain" and "Silent Corruption" failure modes.
