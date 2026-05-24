# Appendix W. Attention-Friendly Production Learning

## Purpose

This appendix gives a simple reading and building protocol for readers whose
attention is limited by ADHD, fatigue, stress, context switching, or a hard day.

It does not make the book easier in the weak sense. The production standard
stays the same. A reliable agent still needs durable state, typed boundaries,
leases, retries, idempotency, evaluation, security, and operations.

The goal is different: reduce the effort needed to restart, orient, and choose
the next useful action.

Use this appendix when the main chapters feel too large.

## Method Map For Attention-Friendly Rigor

This book uses simple learning methods for hard production material. The
methods are not decoration. Each one has to help the reader build or inspect a
real control.

| Learning method | Plain meaning | How the book uses it | Production proof |
| --- | --- | --- | --- |
| Universal design | Give more than one way into the same idea. | Each chapter offers a plain version, focus cue, artifact, walkthrough, checklist, and exercises. | The same concept can be reached from prose, code, SQL, runbook, or test. |
| Clear expectation | Say what the session is trying to prove. | `now`, `next`, and `done` keep one target visible. | The `done` line names an artifact or validation result. |
| Cognitive-load control | Do not ask working memory to hold everything at once. | One hard term is paired with one plain sentence, one artifact, and one proof. | A reader can inspect the row, type, query, event, or test before adding the next concept. |
| Worked example with fading | Show one, complete one, then prove one without hints. | The book moves from tiny examples to production examples to exercises. | The final exercise names the invariant, failure, and regression check without a filled-in answer. |
| Retrieval practice | Close the page and recall the mechanism. | Appendix L asks for recall, explanation, application, and evidence. | The reader can reconstruct the mechanism from memory and point to production evidence. |
| Spaced review | Return later before the idea disappears. | Checkpoints, chapter cards, and maintenance cadence create scheduled review points. | The reader can still name the state, move, proof, and owner after time has passed. |
| Plain language | Use small words before formal words. | Appendix Z gives the small sentence, exact term, artifact, and proof. | The simple sentence still tells the reader what can fail and what to inspect. |

For an ADHD learner, the important move is not to make the chapter shorter. The
important move is to make the next action visible and small.

For a production engineer, the important move is not to feel fluent. The
important move is to leave evidence.

So the shared rule is:

```text
small step
real artifact
visible proof
short pause
```

## Fifteen-Minute Production Sprint

Use this sprint when starting feels hard. The sprint is short, but the proof is
real.

```text
prepare: choose one chapter section or one artifact
inspect: read or open one type, row, query, event, test, or runbook
prove: say what failure this artifact prevents
capture: write the next action outside your head
stop: pause before adding a second concept
```

Example:

```text
prepare: Chapter 13, leases
inspect: pick_due_job.sql
prove: two workers cannot claim the same due job row
capture: next: find the stale-owner rejection test
stop: pause
```

Do not make the sprint bigger because it went well. If the proof is real, the
session counts. If the proof is not real, shrink the task.

## The Rule

Use one small loop:

```text
one concept
one artifact
one proof
one pause
```

That is enough for one study session.

The concept is the idea you are learning. The artifact is the row, type, query,
test, runbook, or policy record that makes the idea real. The proof is the
evidence that the artifact works. The pause protects attention before the next
concept starts.

## Now, Next, Done

Use this when attention is scattered or the chapter feels too big. Do not keep
the plan in your head. Write three lines:

```text
now:
next:
done:
```

The lines mean:

```text
now: the one concept or artifact I am looking at
next: the one action I will take
done: the proof that lets me stop
```

Example:

```text
now: leases
next: inspect pick_due_job.sql
done: I can explain why locked_until lets another worker recover the job
```

This is small, but it is still production work. The `done` line must name
evidence: a type, row, query, event, test, receipt, policy record, runbook, or
validation command.

Avoid vague next actions:

```text
next: continue reading
next: understand retries
next: improve the worker
```

Use concrete next actions:

```text
next: inspect retry_or_dead.sql
next: find the IdempotencyKey constructor
next: run the worker retry test
```

If the `done` line cannot point to evidence, shrink the task until it can.

## Small Action, Fast Feedback

Use this when you start practicing. The goal is to get proof before attention
drifts and before the next abstraction arrives.

```text
act:
check:
explain:
repair:
```

The lines mean:

```text
act: do one small action
check: run or inspect one concrete proof
explain: say what the result proves in one sentence
repair: fix the missing proof, shrink the task, or stop
```

Example:

```text
act: inspect ReleaseGateRunReceipt
check: run the release-gate row conversion tests
explain: malformed release rows fail before review code trusts them
repair: add a rejection test for any bad row that still passes
```

Fast feedback helps focus because the result arrives while the task is still
small. It also helps production work because each step leaves evidence.

Good checks:

```text
check: run one focused test
check: inspect one SQL row
check: read one typed error
check: run one runbook query
check: find one audit event
check: run one readiness command
```

Weak checks:

```text
check: read more
check: think harder
check: trust that the framework handles it
```

The `check` line must point to production evidence: a test result, SQL row,
runbook answer, audit event, typed error, receipt, metric, trace field, or
readiness command.

Do not move to a second abstraction until the first check is real.

## The Micro-Lesson

Before a dense Rust, SQL, policy, or operations section, shrink the idea into
five beats:

```text
pain:
rule:
tiny example:
artifact:
proof:
```

Example:

```text
pain: two workers can try to run the same job
rule: only the worker with a valid lease may complete it
tiny example: worker-a owns job-1 until 10:05
artifact: locked_by and locked_until columns
proof: stale workers fail the owner check and expired leases appear in SQL
```

Now read the full mechanism. The micro-lesson gives your attention a handle.
It does not remove the need to inspect the real type, row, query, event, and
test.

## Restart After Distraction

When you lose the thread, do not reread everything. Restart with three
questions:

```text
What state is changing?
What move changes it?
What proof remains after the move?
```

Then write one line:

```text
state:
move:
proof:
```

If you cannot fill those lines, return to the chapter's Focus Cue and Production
Artifact. Do not jump to code yet.

## Seven-Minute Restart

Use this when you come back after a break.

Do one small pass:

```text
1. Read the chapter title.
2. Read the Plain Version.
3. Fill state, move, proof.
4. Pick one artifact.
5. Name the failure it prevents.
6. Name the test, query, or event that proves it.
7. Leave one next-action note.
```

The note should be specific:

```text
next: inspect retry_or_dead.sql
next: read the failed-row conversion test
next: explain why this tool call needs approval
```

This protocol is small on purpose. It reduces restart cost without changing the
production bar.

Before you continue, convert the note into `now`, `next`, and `done`. That
turns a restart into a finishable unit of work.

## Choose One Mode

ADHD and context switching get harder when one session tries to do three jobs
at once. Pick one mode for the next small block:

```text
read: understand one concept
build: change one artifact
operate: prove one behavior
```

Each mode has a different `done` line:

```text
read done: I can explain state, move, and proof
build done: the type, row, query, or test exists
operate done: the validation command, runbook query, or event proves behavior
```

Do not switch modes because the chapter became hard. Switch modes only after
the `done` line is true. This protects attention and also protects production
quality.

## The Chapter Card

For every chapter, make a small card. Keep it short enough to fit on one
screen.

```text
chapter:
concept:
plain rule:
artifact:
failure prevented:
test or query:
operator question:
```

Example:

```text
chapter: leases and heartbeats
concept: temporary worker ownership
plain rule: a worker owns a job only until its lease expires
artifact: locked_by and locked_until columns
failure prevented: a dead worker owning work forever
test or query: expired lease recovery query
operator question: which jobs are running without a valid heartbeat?
```

This card is not a summary for its own sake. It is an external memory aid. It
lets you return after a break without rebuilding the whole chapter in your
head.

## The Two-Pass Method

Use two passes when a chapter is dense.

Pass 1 builds orientation:

```text
What You Will Learn
Plain Version
Focus Cue
Production Artifact
Implementation Map
Operator Question
Summary
```

Pass 2 builds production skill:

```text
Runtime Walkthrough
Tiny Example
Mechanism
Formal Definition
What Can Fail
Production Contract
Progressive Hardening Path
Testing Strategy
Observability Strategy
Security and Safety Considerations
Operational Checklist
Exercises
Self-Check
```

Pass 1 is not skipping. It creates a map. Pass 2 adds the engineering details.

## Faded Practice Loop

Use faded practice when you understand the example but do not yet feel ready to
build the same control alone.

Do three small rounds:

```text
watch one
complete one
prove one
```

In `watch one`, read the worked example or runtime walkthrough without trying
to improve it. Name only the state, move, artifact, and proof.

In `complete one`, cover part of the answer and fill it back in:

```text
state: pending job
move: worker claims it with ______
artifact: pick_due_job.sql
proof: job event plus locked_until
```

In `prove one`, use a different chapter artifact and write the full proof
without hints:

```text
state:
move:
artifact:
failure prevented:
proof:
```

This is support, not a shortcut. The hints fade, but the production standard
stays the same: name the state, name the move, inspect the artifact, and prove
the invariant.

## Simple Language Without Lowering Rigor

Simple language has one job: make the mechanism easier to see.

Good simple language:

```text
A lease is temporary ownership.
The job is running only while the lease is valid.
If the worker dies, the lease expires and another worker can recover the job.
```

Weak simple language:

```text
The framework handles reliability.
```

The first version is simple and precise. The second version hides the
invariant. In this book, plain language must still name the state, the move,
the failure, and the proof.

Use the one-new-term rule in dense sections:

```text
one hard term
one plain sentence
one artifact
one proof
then the next hard term
```

If a sentence needs three formal terms to make sense, split it. Define the
first term, attach it to evidence, and continue. Simple language should reduce
switching cost without hiding the production mechanism.

## Plain-Language Term Ladder

Use this ladder when a formal term slows you down:

```text
plain phrase -> formal term -> production artifact -> proof
```

The plain phrase lowers friction. The formal term gives precision. The
artifact keeps the term honest. The proof tells you whether the system really
has the control.

| Plain phrase | Formal term | Artifact | Proof |
| --- | --- | --- | --- |
| work that must not disappear | durable job | job row | row exists before model execution |
| temporary ownership | lease | owner and expiry columns | non-owners and stale owners cannot finish work |
| work took too long | timeout policy | deadline plus timeout action | a breached deadline becomes retry, cancellation, escalation, or dead-letter evidence |
| stop was requested | cancellation request | cancellation row | requested, applied, ignored, and expired outcomes remain visible |
| repeat without double action | idempotency | key plus receipt | duplicate intent returns existing work or receipt |
| safe correction after a side effect | compensation action | compensation row plus receipt | correction waits for approval, idempotency, lease, retry, and terminal evidence |
| practice failure safely | failure drill | drill plan plus event timeline | hypothesis, blast radius, rollback, evidence, result, and signoff agree |
| model-proposed action | typed tool request | parsed request and tool-call row | validation, policy, sandbox, and approval run first |
| human decision before risk | approval gate | approval row and audit event | risky side effect waits for decision evidence |
| explain the past | event timeline | job events, audit events, operation events | an operator can reconstruct state, actor, transition, evidence, and invariant |

When you meet a hard word, fill the four columns. If you cannot name the
artifact or proof, the term is not yet usable production knowledge. Return to
the chapter's Production Artifact, Implementation Map, or Appendix G evidence
map.

## The No-Compromise Gate

Before you mark a chapter as learned, answer these questions:

```text
Can I name the state?
Can I name the actor allowed to change it?
Can I name the transition?
Can I name the durable evidence?
Can I name the invariant?
Can I name one failure this prevents?
Can I name one test, query, or runbook that would catch a regression?
```

If the answer is no, the next action is small: find the missing row, type,
query, test, or runbook.

## Code Reading Protocol

When reading Rust in this book, use this order:

```text
1. Find the domain type.
2. Find the constructor or conversion boundary.
3. Find the error type.
4. Find the state transition.
5. Find the test that rejects the bad case.
```

This order is easier than reading a module top to bottom. It also matches the
production question: what invalid state is the code preventing?

For example, when you see a database row, ask:

```text
Where does raw database data become a domain type?
What bad status, bad version, missing timestamp, or impossible score is rejected?
Which test proves the rejection?
```

## SQL Reading Protocol

When reading SQL, do not start by memorizing the whole schema. Start with the
promise the table makes.

Use this order:

```text
1. Primary identity.
2. State column.
3. Ownership or version column.
4. Retry, deadline, or idempotency column.
5. Constraint that rejects an impossible row.
6. Index that supports the operational question.
```

For example, a job table is not just storage. It is a promise that work survives
process death. A lease column is not just metadata. It is the rule that lets
another worker recover the job later.

## Runbook Reading Protocol

A runbook is production code for humans. Read it like a control surface.

Use this order:

```text
1. What question does this command answer?
2. What evidence does it read?
3. What action is safe after reading it?
4. What action still needs approval?
5. What event or note records the decision?
```

During an incident, the goal is not to be clever. The goal is to avoid inventing
new behavior under stress.

## Spaced Review

Reliable agents are too large to learn in one pass. Use spaced review.

After each part, wait, then answer:

```text
What broke?
What state protected it?
What evidence proved it?
What test or runbook would catch it next time?
```

Use Appendix L for retrieval practice, Appendix F for chapter checkpoints, and
Appendix V for maintenance cadence. These are not extra homework. They are the
book's way of turning short attention into durable engineering judgment.

Use Appendix Z when the blocker is vocabulary. The plain-language production
cards give one small sentence, one exact term, one artifact, and one proof so a
hard word becomes an inspectable production claim.

## Study Session Template

Use this template for a 30 to 45 minute session:

```text
1. Pick one chapter or one section.
2. Read the Plain Version and Focus Cue.
3. Fill the chapter card.
4. Inspect one artifact.
5. Answer one operator question.
6. Run or name one test, query, or readiness check.
7. Stop with a next-action note.
```

The next-action note should be concrete:

```text
next: read the row conversion test for evaluation_runs
next: run the queue health query
next: explain why retry needs idempotency
```

Avoid vague notes such as:

```text
next: continue chapter
```

Vague notes make restart harder.

## Distraction Parking Lot

When a side question appears, do not carry it in your head. Write it down and
return to the current `now`, `next`, and `done` loop.

Use this shape:

```text
now:
next:
done:
later:
```

Example:

```text
now: release gate evidence
next: inspect release_gate_status.sql
done: I can name why a release is promoted, canaried, or blocked
later: compare this with Temporal after I understand the Postgres proof
```

The `later` line protects attention. It is not a promise to chase the question
now. Finish the current proof first.

## Visible Wins

A visible win is inspectable. It is not a feeling that the chapter "mostly made
sense."

Good visible wins:

```text
I found the row.
I found the Rust type.
I ran the test.
I ran the SQL query.
I answered the operator question.
I can say which failure this prevents.
```

Weak wins:

```text
I read more pages.
I watched the example.
I kind of understand retries.
```

Use visible wins when motivation is low. Production systems are built from
evidence. Learning this book should also leave evidence.

## Tiny Production Path

When the goal is an agent that can run for years, do not try to hold the whole
system in working memory. Build one proof layer at a time.

Use this path:

| Step | Small rule | Artifact | Proof |
| --- | --- | --- | --- |
| 1. Durable intake | Work exists before the model starts. | `agent_jobs` row | The request survives process death. |
| 2. Typed boundary | Raw input becomes a domain type. | constructor or row conversion | Bad input is rejected before worker logic. |
| 3. Worker ownership | One worker owns work only for a while. | lease columns | Another worker can recover after expiry. |
| 4. Idempotent action | Retry must not double the side effect. | idempotency key and receipt | Duplicate intent returns existing evidence. |
| 5. Retry decision | A retry is scheduled, not guessed. | attempts, failure class, `next_run_at` | Transient failure retries; exhausted work stops. |
| 6. Human control | Risk waits for durable approval. | approval row and audit event | The model cannot approve its own risky action. |
| 7. Human escalation | Unsafe waiting needs an owner. | escalation row and open-escalations query | A deadline, repeated failure, or security signal names a human owner and timeline. |
| 8. Observability | Operators can reconstruct what happened. | trace id, operation event, audit event | A run can be followed across API, worker, model, and tool. |
| 9. Cost and capacity guard | Too much work or spend must slow intake. | budget, quota, queue, and latency rows | New work can be delayed or rejected before overload spreads. |
| 10. Evaluation | Behavior changes need evidence. | evaluation run | A prompt or model version passes before release. |
| 11. Release gate | Missing evidence blocks promotion. | `release_gate_runs` and `release_gate_status.sql` | The release is promoted, canaried, or blocked with reasons. |
| 12. Security boundary | Permission is outside the model. | policy and sandbox decision | Tool execution waits for non-model controls. |
| 13. Injection and exfiltration guard | Hostile text stays data. | parser, authorization, sandbox, and approval events | Hostile text cannot choose tenant, egress, filesystem path, secret, or tool authority. |
| 14. Recovery | Backup is not recovery until practiced. | restore drill | RPO, RTO, and replay safety are proven. |
| 15. Maintenance cadence | Years-long reliability needs a calendar. | review cadence and evidence packet | Owners review drift, failures, costs, and stale controls. |

Do not move to the next step until the proof sentence is real. This is the
shortest path through the book that still reaches production reliability.

## Production Builder Template

When you turn a concept into code, use this small build loop:

```text
1. Name the failure.
2. Name the invariant.
3. Add or inspect the type.
4. Add or inspect the row.
5. Add or inspect the transition.
6. Add or inspect the event.
7. Add a negative test.
8. Add or inspect the runbook query.
9. Run the validation gate.
```

This is the book's engineering discipline in small pieces. The loop is simple,
but it still reaches production concerns: type safety, persistence, state
transitions, auditability, operations, and validation.

## Exercises

1. Pick one chapter you already read. Fill the chapter card without reopening
   the chapter. Then reopen the chapter and correct only the missing artifact,
   failure, or proof.

2. Pick one SQL table in the companion implementation. Write the table's promise
   in one sentence. Then name the constraint or index that helps keep the
   promise.

3. Pick one Rust newtype in the companion implementation. Explain what category
   error it prevents. Then find the negative test or add a note for the missing
   negative test.

4. Pick one runbook query. Write the operator question it answers and the action
   that is safe after reading the result.

## Summary

Attention-friendly learning means the book gives you restart points, short
loops, external memory aids, worked examples, retrieval practice, and visible
gates. It does not lower the production requirement.

Invariant: a reader can lose focus, return later, and still find the state,
move, proof, artifact, and next action.

Evidence: the learning path, chapter card, checkpoints, retrieval practice,
role paths, code reading path, runbooks, and maintenance cadence all point back
to deployable artifacts.

## Further Reading and Sources

- [CAST: Universal Design for Learning](./31-credible-resources-further-reading.md#learning-design-and-plain-language) supports giving readers multiple ways to enter the same production concept.
- [CDC: ADHD in the Classroom](./31-credible-resources-further-reading.md#learning-design-and-plain-language) supports predictable routines, organization, clear expectations, and quick feedback for ADHD learners.
- [Digital.gov: Short and Simple](./31-credible-resources-further-reading.md#learning-design-and-plain-language) supports short sentences, clear words, and reader-centered organization.
- [Paas and van Merrienboer: Cognitive-Load Theory](./31-credible-resources-further-reading.md#learning-design-and-plain-language) supports reducing unnecessary load before adding complex Rust and SQL detail.
- [IES: Organizing Instruction and Study to Improve Student Learning](./31-credible-resources-further-reading.md#learning-design-and-plain-language) supports worked examples, concrete representations, spacing, and active recall.
- [Nature Reviews Psychology: Spacing and Retrieval Practice](./31-credible-resources-further-reading.md#learning-design-and-plain-language) supports retrieval practice and spaced review for durable learning.
