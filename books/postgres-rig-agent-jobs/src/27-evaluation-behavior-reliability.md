# 27. Evaluation And Behavior Reliability

## What You Will Learn

This chapter teaches you to:

- explain how to know the agent is useful and safe, not only available;
- inspect golden datasets, behavior evals, regression cases, prompt versions, model versions, and approval outcomes;
- verify that behavior changes pass evidence gates before release.

The production evidence is an evaluation run tied to prompt, model, policy,
tool contracts, expected outputs, failure cases, and release decisions.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** ownership tells who must judge repeated behavior risk.
- **Adds:** evaluation receipts for prompt, model, and behavior changes.
- **Prepares:** typed memory, retrieval, and retention controls.

## Production Failure

The agent is fast, available, and wrong.

I call this the **"Fluent Failure."** An agent can sound perfectly confident and polite while giving an answer that violates every business policy. A new prompt improves tone but makes the agent miss required escalation in
high-risk cases. Infrastructure metrics stay healthy while behavior regresses.

**What breaks:** service reliability was mistaken for behavior reliability.

The system proved that jobs ran, workers stayed alive, and requests completed.
It did not prove that the agent made the right decision for the risky case. For
an agent, "the system responded" is only the first reliability question. The
second question is whether the response deserved to reach the user or trigger a
tool.

**False fix:** ask a human to skim a few example outputs before shipping.

Skimming examples can catch obvious mistakes, but it does not create a repeatable
release gate. The team cannot know which cases were checked, which prompt and
model versions produced them, which failures were accepted, or whether the next
release regressed the same behavior.

**Design response:** treat datasets, rubrics, model versions, prompt versions,
graders, human samples, and promotion decisions as production release evidence.

The evaluation artifact should be as reviewable as a build artifact. It should
answer: what did we test, which behavior version did we test, what failed, who
reviewed the risky samples, and why was promotion allowed or blocked?

We should also respect **Evaluator Asymmetry**. The **Evaluator Model** should usually be "Larger/Smarter" than the **Worker Model**. For example, you might use GPT-4o to grade the outputs of GPT-4o-mini. This ensures the judge has the "Reasoning Headroom" to spot subtle mistakes.

## Motivation

In production, an agent can be available, fast, and still wrong. Traditional service reliability tells you whether the system responded; behavior reliability tells you whether the response was acceptable.

Without evaluation evidence, prompt and model changes become unreviewed behavior releases. This chapter treats datasets, rubrics, graders, and receipts as production controls.

## Plain Version

Read this as the simple version:

**Simple rule:** Evaluation is the CI pipeline for agent behavior.

Unit tests protect deterministic software. Evaluations protect model-dependent
behavior. Both are needed because a production agent is deterministic software
wrapped around uncertain reasoning.

**Why it matters:** A reliable system must detect when prompts, models, tools, or
policies change the agent in unsafe ways.

The dangerous change may look harmless. A prompt may sound more polite while
losing an escalation rule. A model route may become cheaper while producing less
grounded answers. A policy tweak may reduce approvals while allowing a risky
side effect.

**What to watch:** Watch golden datasets, expected decisions, regression
failures, promotion gates, and versioned evaluation records.

## What You Already Know

Start with these anchors:

- The system can now run, recover, deploy, and operate reliably as infrastructure.
- An available agent can still produce bad behavior.
- Prompt and model changes need release evidence.

This chapter adds: behavior reliability. You will test whether the agent is
right enough, safe enough, and stable enough before a prompt, model, tool, or
policy version ships.

## Focus Cue

Keep three things in view:

- **State:** behavior dataset, rubric, evaluator, prompt version, model version, tool version, policy version, and promotion decision.
- **Move:** a behavior change moves toward production only after it produces versioned evaluation evidence.
- **Proof:** Evaluation runs, dataset versions, grader versions, human review samples, and promotion decisions are stored.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a versioned evaluation dataset, rubric, grader result, and behavior-release receipt.
- **Why it matters:** agent reliability includes behavior quality, not only uptime and job completion.
- **Done when:** a model or prompt change has passing evaluation evidence before it changes production behavior.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/evaluation.rs`, golden datasets, rubrics, grader results, and release gate tests.
- **State transition:** measure model behavior before changing production behavior.
- **Evidence path:** prompt/model versions have passing evaluation receipts tied to release decisions.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Can this behavior change ship with versioned evaluation evidence?
- **Evidence to inspect:** dataset version, prompt version, model version, rubric, grader result, failed cases, and release gate.
- **Escalate if:** model behavior changes because a demo looked good rather than because evaluation passed.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** a model, prompt, rubric, or tool behavior changes.

This may happen through code, configuration, provider routing, a prompt edit, a
tool schema change, a policy update, or a new evaluator version. Treat each as a
behavior release surface.

**Action:** run versioned evaluation before promotion.

The evaluation should use the exact prompt, model, tool, policy, dataset, and
grader versions that may reach production. If the evidence belongs to a
different version, it is not release evidence for this change.

**Persistence:** persist dataset, rubric, result, failures, and release receipt.

The receipt is what lets a release gate, runbook, or postmortem reconstruct why
the behavior was allowed. Store failures as well as passes. A failed case is not
embarrassing; it is a future regression test.

**Check:** verify behavior changes do not ship without passing evidence.

If the evaluation is missing, stale, or mismatched to the candidate version, the
release gate should block or restrict the change to a controlled canary.


## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** behavior changes require versioned evaluation evidence.
- **Validation path:** inspect dataset version, rubric, grader result, failed cases, and release-gate record.
- **Stop if:** model or prompt behavior changes because a demo looked acceptable.

The evidence should answer one production question: which evaluation receipt
proves this exact behavior version is safe enough for the job kind it will run?

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, an agent can be available, fast, and still wrong
rule: Evaluation is the CI pipeline for agent behavior
tiny example: behavior dataset, rubric, evaluator, prompt version, model version, tool version, policy version, and promotion decision
artifact: a versioned evaluation dataset, rubric, grader result, and behavior-release receipt
proof: behavior changes require versioned evaluation evidence
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Intuition

Think of the model as a worker whose judgment changes over time. The model
provider can update weights, prompts can change, context can drift, tools can
return different data, and user behavior can move into cases your tests never
covered.

Evaluation is the calibration loop. It answers:

```text
Given this input and this production context,
would this agent still make an acceptable decision today?
```

The word "today" matters. Agent behavior is not fixed forever. Models can
change, prompts can drift, tools can return different data, and user inputs can
move into cases the original demo never covered. Evaluation is how the system
keeps checking behavior after the first impressive demo.

## Tiny Example

Suppose the incident-triage agent receives:

```text
Payment deploy failed.
Error rate is 18%.
Rollback is available.
```

A weak answer is:

```text
Try restarting the database.
```

A reliable answer is:

```text
Open an incident, freeze new deploys, roll back the payment service, and review
the database only if rollback does not reduce the error rate.
```

The system needs a way to distinguish those answers before production users do.

Notice that both answers are fluent. That is why style is not enough. I call this **The Polite Liar Problem**. Students need to know that "Politeness" is just a mask. We are grading the **Action**, not the **Tone**! A "Good Grade" in this book means "Safety," not just "Grammar."

The evaluation must check the operational decision: open the incident, freeze
deploys, roll back the payment service, and only then inspect the database if
rollback does not help.

Read the tiny case as:

```text
setup: a prompt change gives a plausible but unsafe incident recommendation
transition: evaluation compares behavior against expected cases before release
evidence: golden dataset, shadow run, human sample, failure case, and eval receipt decide promotion
invariant: agent behavior changes need CI-style evidence before production exposure
```

> ### 🎓 The Professor's Corner
>
> **The Tuning Fork: The Calibration Loop**
>
> Think of a guitarist tuning their instrument before a show. They don't just "assume" the strings are right because they were right yesterday! 
> 
> **Evaluation** is our **Tuning Fork**. It provides a standard "Note" (the golden dataset) that we use to check if our agent is still in tune. If the agent sounds "flat" (gives a wrong answer), we turn the knobs (adjust the prompt) until it matches the fork again!

## Evaluation Surfaces

Use several evaluation layers because no single test catches every failure:

```text
unit tests: domain invariants and state transitions
fixture evaluations: known prompts with expected properties
golden traces: full job lifecycle examples
shadow runs: new prompt/model against real historical payloads
canary runs: small live traffic slice with tight rollback
post-incident evals: every incident becomes a regression fixture
```

The job ledger already gives you the raw material: payload, versions, result,
events, error class, and timestamps. That is why versioned rows matter. Without
versions, you cannot know whether a bad answer came from the payload schema,
prompt, model route, policy, or worker build.

Each layer catches a different kind of mistake. Unit tests protect typed
invariants. Fixture evaluations protect known behavior. Golden traces protect
end-to-end workflows. Shadow runs show what a new version would have done on old
traffic. Canary runs test the new version under real conditions with a small
blast radius. Post-incident evals make sure the same failure is not forgotten.

## Golden Datasets

A golden dataset is the small set of cases that must keep working across prompt,
model, tool, and policy changes. It is not every historical payload. It is the
curated set of examples that define "we must never regress this behavior."

I recommend **Boundary Case Curation**. You should prioritize examples that are "Right on the Edge" of a policy—for example, a refund request that is exactly $1 over the automatic limit. These are the cases that actually test the agent's reasoning depth and consistency.

For a support agent, the golden dataset might include:

```text
refund request with clear policy evidence
refund request missing evidence
angry user asking for escalation
prompt-injection attempt inside pasted email
duplicate webhook after a slow provider call
```

For an incident-response agent, it might include:

```text
deploy failure with rollback available
database saturation without rollback evidence
payment outage needing incident escalation
stale dashboard data that should trigger abstention
```

The companion crate names this concept explicitly:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/evaluation.rs:golden_dataset}}
```

The Rust type is `GoldenEvaluationDataset`: a small wrapper that marks a
versioned dataset as release-critical evidence rather than an ordinary
experiment.

The important point is not the wrapper itself. The important point is the
release invariant:

```text
no behavior version is promoted until the golden dataset has a receipt
```

A broad evaluation suite can grow large and exploratory. The golden dataset
should stay small enough that humans can understand why each case exists and
serious enough that deleting a failing case requires review.

This is a practical discipline. If the golden dataset becomes a dumping ground,
people stop understanding it. If it is too small or too easy, it becomes a
ceremony. The right golden dataset is a compact contract for behavior the system
must not lose.

## Scorecard

For production agents, score behavior on dimensions that map to real risk:

| Dimension | Question |
| --- | --- |
| task correctness | Did the answer solve the requested operational problem? |
| grounding | Did it rely only on available evidence? |
| policy compliance | Did it avoid disallowed actions? |
| escalation | Did it ask for approval when risk required it? |
| abstention | Did it refuse or defer when evidence was insufficient? |
| cost | Did it stay within budget for the job kind? |
| latency | Did it complete inside the user or operator deadline? |
| replay stability | Does the same version produce equivalent decisions? |

This scorecard should be stored as data, not hidden in a spreadsheet. A prompt
release should leave an evaluation receipt just like a deploy leaves a build
artifact.

Scores should map to release decisions. A low grounding score may block a
research agent from summarizing source material. A failed escalation score may
block a support or incident-response agent from production. A cost regression may
force a narrower canary even when task correctness improves.

## Executable Evaluation Gate

The companion crate includes a small typed evaluation gate. It is deliberately
simple: a versioned dataset contains cases, each case defines expected behavior,
the evaluator runs a deterministic `AgentRunner`, and the result becomes an
`EvaluationReceipt`.

```text
EvaluationDataset
  -> BehaviorEvaluator
  -> EvaluationReceipt
  -> PromotionDecision
```

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/evaluation.rs:behavior_eval_gate}}
```

The important idea is not keyword matching. The important idea is the boundary:

```text
prompt/model/tool/policy version
  -> checked behavior cases
  -> receipt
  -> promote or block
```

The receipt records the dataset version, evaluator version, prompt/model/tool
versions, case results, pass/fail counts, and promotion decision. That makes an
evaluation a production artifact rather than an informal notebook.

> ### 🎓 The Professor's Corner
>
> **Semantic Assertions: The Taste Test**
>
> A normal test is like checking the "Color" of a soup—it's easy but doesn't tell you much. A **Semantic Assertion** is like a "Taste Test." You can't just look at the code; you have to check the flavor! 
> 
> Because we can't use a simple "Diff" to check model output, we use a smarter model to perform the taste test. It asks: "Does this answer satisfy the business goal?" It turns a fuzzy "Maybe" into a solid "Yes" or "No."

The persistence boundary should store the versions directly on the evaluation
run. Joining to a nearby agent run is useful context, but a behavior release
should still be reviewable from the evaluation receipt itself:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/evaluation.rs:evaluation_run_row_boundary}}
```

The row decoder rejects unknown statuses, non-object reports, out-of-range
scores, terminal evaluations without score and completion time, and active
evaluations that already carry terminal evidence. That is the raw-outside,
typed-inside rule applied to behavior evidence.

This protects the release gate from bad evidence. A malformed evaluation row
should not quietly become a passing receipt. Once evaluation controls promotion,
evaluation data must be treated like production input.

Evaluation is still one input to release promotion, not the whole decision. A
release can pass behavior evals and still be unsafe if the current SLO budget
is exhausted, the worker cannot process the payload schema, or the approval
evidence is missing for a high-risk change. Chapter 25 combines these signals
into a typed release gate.

Operators should also have a checked query for recent behavior evidence:

```bash
psql "$DATABASE_URL" \
  -f examples/postgres-rig-agent-jobs/sql/evaluation_receipts_by_version.sql
```

That query links dataset, evaluator, prompt, model, tool, policy, score, job
kind, run id, and trace id. It is the answer to the production question:
"Which behavior evidence justified this version?"

## Release Gate

A model or prompt change should move through gates:

```text
draft prompt
run fixture evals
run historical shadow evals
review failures
canary small job kind or tenant set
watch behavior SLIs
promote or roll back
```

The important production rule:

```text
No prompt, model, tool, or policy version becomes default without an evaluation
receipt tied to that version.
```

The phrase "tied to that version" is the core rule. A passing eval for prompt
`p1` does not justify prompt `p2`. A passing eval for one model route does not
justify another route. A passing eval before a tool schema change may not justify
the same prompt after the tool changes.

## Common Mistakes

Do not evaluate only happy paths. Reliable agents need adversarial, ambiguous,
stale, partial, and conflicting inputs.

Do not use the model as the only judge of its own output. LLM judges can help,
but high-risk decisions also need deterministic checks, domain assertions, and
human review samples.

Do not delete failed eval cases after the prompt improves. The failure case is
now part of the safety harness.

This is how the system learns. A production mistake should become a regression
case with a clear expected behavior. If the same class of failure returns, the
release gate should catch it before users do.

## Formal Definition

For this chapter, the precise definition is:

```text
Evaluation is the release-control system for probabilistic behavior, tying outputs to datasets, rubrics, prompts, models, tools, and policies.
```

In the book's system model, **State** means behavior dataset, rubric, evaluator,
prompt version, model version, tool version, policy version, and promotion
decision.

The **Actor** is the evaluation runner or human reviewer deciding whether
behavior is acceptable for release.

The **Transition** is that a behavior change moves toward production only after
it produces versioned evaluation evidence.

The **Evidence** is stored evaluation runs, dataset versions, grader versions,
human review samples, and promotion decisions.

The **Invariant** is that agent behavior changes are governed by evidence, not
anecdotes or impressive examples.

## What Can Fail

**Design smell:** availability is treated as behavior quality. The team says the
agent is reliable because jobs complete, even though no one can prove the
decisions are still acceptable.

**Production symptom:** the system is up while answers drift, policy compliance
weakens, or tool choices degrade. Infrastructure health hides behavior
regression.

**Corrective invariant:** behavior releases require evaluation evidence.

**Evidence to inspect:** dataset, rubric, grader, human review sample, and
evaluation receipt bind to prompt/model versions.


## Production Contract

Behavior reliability is credible only when evaluation datasets and evaluator
logic are versioned. Prompt, model, tool, and policy versions must be recorded.
Release gates must require evaluation receipts. High-risk samples must include
human review. Post-incident failures must become regression cases. Eval results
must be stored as production artifacts.

An eval that is not tied to a release decision is useful research. An eval tied
to a versioned gate is production engineering.

## Progressive Hardening Path

**Naive version:** availability is treated as behavior quality. Availability or
a good demo is mistaken for stable agent behavior.

**Safer version:** behavior releases require evaluation evidence. Golden
datasets, rubrics, evaluator versions, and human review samples become release
evidence.

**Production version:** dataset, rubric, grader, human review sample, and
evaluation receipt bind to prompt/model versions. Prompt and model promotion
depends on evaluation receipts tied to behavior, risk, version, release gate,
and runbook query evidence. Use the naive version only to spot anecdotal quality.
Use the safer version to evaluate behavior. Use the production version before
changing prompts, tools, policies, or models.

## Testing Strategy

Test behavior changes as release-controlled artifacts:

- **Unit or type test:** prove Rust golden datasets, evaluation cases, evaluator versions, and promotion decisions reject empty cases and missing required behavior.
- **Persistence or boundary test:** prove Postgres evaluation runs and receipts bind dataset, prompt version, model version, tool version, policy version, rubric, score, terminal evidence, and outcome.
- **Regression test:** change a prompt or model route without a fresh evaluation receipt; the release gate should block behavior promotion.

## Observability Strategy

Observe behavior quality as release evidence.

Emit structured `tracing` fields for dataset version, evaluator version, prompt
version, model version, tool version, case id, outcome, and trace id. These
fields connect one behavior result to the exact version packet that produced it.

Record an operation event when evaluation starts, a case fails, human review
samples are recorded, or behavior promotion is allowed or blocked. A blocked
behavior release is useful evidence, not a nuisance.

The `evaluation_receipts_by_version.sql` runbook query should link a production
behavior version to the evaluation receipt that justified releasing it.

## Security and Safety Considerations

Evaluation data and graders are part of the trust boundary.

Treat datasets, labels, grader output, model comparisons, and human review
samples as untrusted until validated and versioned. A poisoned dataset can teach
the release gate to accept bad behavior.

authorization, sandboxing, and approval are needed when evaluation can expose
private data, execute tools, or promote high-risk behavior. Redact private
examples and sensitive expected outputs while preserving case id, dataset
version, rubric, outcome, and promotion evidence.

## Operational Checklist

Use this checklist before relying on evaluation as behavior release evidence in
production.

**State:** Prompt, model, tool, policy, dataset, rubric, grader, and release
decision are versioned and linked.

**Boundary:** Raw model outputs become parsed evaluation cases and scored results
before they influence promotion.

**Failure:** A bad behavior change is blocked even if availability checks and
unit tests pass.

**Observability:** Evaluation receipts expose dataset id, case id, prompt
version, model version, score, failure reason, and release gate.

**Safety:** High-risk behavior changes require human review and cannot hide
unsafe outputs in aggregate scores.

## Exercises

1. Write a negative test where a new model passes health checks but fails a golden
   dataset case and must not promote despite idempotency-safe deployment. Explain which
   idempotency key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: evaluation_runs, evaluation_cases, grader results,
   behavior release receipt, and policy version evidence.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   GoldenEvaluationDataset, EvaluationRun, RubricScore, and BehaviorReleaseDecision
   types. Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Why can an available agent still be behaviorally unreliable?
- Explain: Why is evaluation the release gate for prompt and model behavior?
- Apply: Promote one prompt change for a high-risk job kind.
- Evidence: Name the golden dataset, shadow run, human review sample, failure case, and evaluation receipt.

## Summary

System reliability tells you whether the agent ran. Behavior reliability tells
you whether the agent deserved to run.

- **Invariant:** behavior changes are promoted only when versioned evaluation evidence supports the release decision.
- **Evidence:** golden datasets, case results, rubric scores, model and prompt versions, grader outputs, review samples, and release receipts explain the decision.
- **Carry forward:** a green queue does not prove a good agent; evaluation evidence does.

## Changed Understanding

- **Before this chapter:** evaluation looked like an offline quality score.
- **After this chapter:** evaluation is the CI/CD pipeline for model-dependent behavior and must block unsafe prompt, model, or policy changes.
- **Keep:** inspect the golden dataset, evaluator version, pass gate, regression failure, and deployment decision.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
