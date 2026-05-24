# How to Read This Book

## Motivation

This book is organized as a sequence of reliability ideas, not as a tour of
tools. Each chapter adds one production capability to the same core system:

```text
Postgres keeps the work durable.
Rust makes the boundaries explicit.
Rig performs one model-powered step.
Events, metrics, policies, and runbooks make the system operable.
```

Read the chapters in order the first time. Later, use the chapter contracts
below to return to the weakest part of your own agent system.

Before Part I, read "System Model And Notation." It defines the small language
the rest of the book uses for state transitions, actors, evidence chains, and
invariants. That chapter is the bridge between intuition-first reading and
serious production reasoning.

Then read "Design Principles." It turns the system model into reusable rules
you can apply outside this companion implementation. Those principles are the
short version of the book's production judgment.

Then read "Production Scope And Trade-Offs." It names the operating envelope
for the Postgres-first design and shows when a workflow engine, queue
framework, distributed platform, or simple script is the better control
surface.

## The Learning Loop

Each serious chapter should move through the same learning loop:

```text
chapter thread -> pain -> plain version -> micro-lesson -> intuition -> tiny example -> mechanism -> what can fail -> invariant -> progressive hardening -> testing -> observability -> safety -> checklist -> exercises -> summary
```

The order matters. If you start from a framework API, the system looks like a
library exercise. If you start from the failure, the API becomes a tool for
preserving a concrete promise.

For example:

```text
Pain:
  A worker can crash after picking a job.

Intuition:
  A job needs a lease, not just a status.

Plain version:
  one worker owns job-1 until the lease expires.

Tiny example:
  worker-a owns job-1 until 10:05.

Mechanism:
  pick the row with FOR UPDATE SKIP LOCKED and store locked_until.

What can fail:
  a worker can keep ownership forever if the lease has no expiry.

Invariant:
  another worker can only recover job-1 after the lease expires.

Operation:
  expired leases appear in a runbook query and can be investigated.
```

This is the teaching style of the book. Every abstraction should earn its place
by protecting a promise that matters in production.

Read each chapter as an engineering transformation, not as a tool lesson. The
pattern is:

```text
naive demo
  -> failure
  -> intuition
  -> typed model
  -> minimal implementation
  -> production hardening
  -> tests and evals
  -> operational judgment
```

This is the reader capability ladder:

```text
recognition:
  I have seen the idea.

understanding:
  I can explain the invariant in simple language.

implementation:
  I can build the type, row, query, test, receipt, or runbook.

judgment:
  I know when to use the pattern, when not to use it, and how it fails.
```

Most agent demos stop at recognition. This book should take you to judgment.
The model may guess. The system must know.

When a chapter feels abstract, ask which transformation it is teaching:

| From | To | Production proof |
| --- | --- | --- |
| raw user text | typed domain request | parser, validator, newtype, and rejection test |
| model output | validated agent intent | schema check, semantic check, policy result, and audit event |
| chat loop | durable agent run | `agent_runs`, steps, trace id, and terminal state |
| tool call | permissioned side effect | authorization event, approval record, idempotency key, and receipt |
| retry | safe repeat | retry state, idempotency key, attempts, and failure history |
| memory | governed state | scope, source, confidence, retention, and retrieval policy |
| logs | operational evidence | traces, metrics, operation events, audit events, and runbook query |

## If Focus Is Hard Today

This book is attention-friendly, not technically lighter. It uses simple
language without lowering the technical bar.

If focus is hard today, read the chapter in this short loop:

```text
1. Read What You Will Learn.
2. Read the Chapter Thread.
3. Read the Plain Version.
4. Read the Focus Cue.
5. Read the Production Artifact.
6. Read the Implementation Map.
7. Read the Operator Question.
8. Read the Runtime Walkthrough.
9. Read the Acceptance Gate.
10. Read the Micro-Lesson.
11. Read the tiny example.
12. Find the production contract.
13. Scan the operational checklist.
14. Answer the self-check.
```

Then stop. That is still real progress.

Use the `Chapter Thread` when the book starts to feel like many separate
controls. It names:

```text
Builds on
Adds
Prepares
```

That is the chapter's local place in the larger production argument.

## Seven-Minute Restart

Use this when you return after a break, lose the thread, or feel the chapter
becoming too large.

Do not reread the whole chapter first. Restart with one small proof:

```text
1. Read the chapter title.
2. Read the Plain Version.
3. Write: state, move, proof.
4. Pick one artifact: type, row, query, test, runbook, or event.
5. Name the failure it prevents.
6. Name the proof that would catch a regression.
7. Write the next action in one line.
```

Example:

```text
state: pending job
move: worker claims the job with a lease
proof: job_picked event and locked_until value
artifact: pick_due_job.sql
failure prevented: two workers owning the same job
regression proof: cooperative claim test
next: inspect the lease-owner predicate
```

This is not a shortcut around production work. It is a small doorway back into
the same standard: durable state, typed transitions, explicit evidence, and
tests that prove the invariant.

If even that loop feels too large, use Appendix W. It gives a smaller restart
protocol:

```text
one concept
one artifact
one proof
one pause
```

Every main chapter also has a `Micro-Lesson` after the acceptance gate. Use it
when the next section looks dense. It compresses the chapter into:

```text
pain
rule
tiny example
artifact
proof
```

That is the smallest accurate bridge into the heavier mechanism.

If you have ADHD, low working memory, or a day with too many context switches,
do not try to read like a scanner. Read like an operator:

```text
one concept
one artifact
one proof
one short pause
```

Appendix W also gives a method map and a fifteen-minute production sprint. Use
those when starting is harder than understanding. The sprint keeps the next
action small while still requiring a real artifact and a real proof.

When your goal is deployment, use Appendix W's Tiny Production Path. It breaks
the full system into twelve proof layers: durable intake, typed boundary,
worker ownership, idempotent action, retry decision, human control,
observability, evaluation, release gate, security boundary, recovery, and
maintenance cadence. Each layer has one artifact and one proof sentence.

Keep the next step outside your head:

```text
now: the one concept or artifact I am looking at
next: the one action I will take
done: the proof that lets me stop
```

Example:

```text
now: retry safety
next: inspect retry_or_dead.sql
done: I can explain why the retry is safe only with an idempotency key
```

The `done` line must point to production evidence: a type, row, query, event,
test, receipt, policy record, runbook, or validation command.

Use fast feedback when you practice. Do not read or code for an hour before
checking whether the idea works. Use this four-step loop:

```text
act: do one small action
check: run or inspect one concrete proof
explain: say what the result proves
repair: fix the missing proof or stop
```

Example:

```text
act: inspect DbReleaseGateRunRow
check: find the row-conversion test
explain: raw database values are revalidated before review code trusts them
repair: add the missing rejection test if the bad row is accepted
```

Fast feedback is not only a study method. It is how production engineers avoid
building on a false assumption.

When even a full lab feels too large, use Appendix AA. It gives production
micro-drills: one action, one check, one proof sentence, and one stop rule.
The drills stay short, but they still touch the real production surfaces:
durable intake, typed boundaries, leases, retries, tool contracts, approvals,
side-effect receipts, traces, evaluation, release gates, security, recovery,
and maintenance.
Its artifact index names exact Rust files, SQL files, test filters, and
readiness commands so the first step is visible.

## Plain-Language Term Ladder

When a term feels heavy, translate it once before reading deeper.

Use this ladder:

```text
plain phrase -> formal term -> production artifact -> proof
```

Do not stop at the formal term. A serious production term must point to
something you can inspect.

| Plain phrase | Formal term | Production artifact | Proof |
| --- | --- | --- | --- |
| work that survives a crash | durable job | `agent_jobs` or `scheduled_jobs` row | the row exists before the model runs |
| temporary ownership | lease | `locked_by` and `locked_until` | a stale worker cannot complete the job |
| doing it again safely | idempotency | idempotency key and receipt | duplicate input maps to one logical action |
| the model asks to act | typed tool request | `ToolInput<T>` and `tool_calls` row | parser, policy, and approval run before execution |
| ask a person before risk | human approval gate | approval request and audit event | risky work waits for durable decision evidence |
| show what happened | event timeline | job events and operation events | an operator can reconstruct the transition |

This is the book's plain-language rule: simple words first, exact term second,
production artifact third, proof last. If a chapter uses a term such as lease,
idempotency, SLO, audit event, or typestate, ask for the row, type, query,
event, test, or runbook that makes the term real.

Use this two-pass method:

```text
pass 1:
  read only the Plain Version, Focus Cue, Production Artifact, and Summary

pass 2:
  read the mechanism, failure modes, tests, and operational checklist
```

The first pass gives your attention a map. The second pass adds the production
details. This is not skipping rigor. It is lowering switching cost before the
chapter asks you to hold Rust types, SQL rows, worker states, and operator
evidence at the same time.

When the example makes sense but independent practice still feels too large,
use a faded practice loop:

```text
watch one
complete one
prove one
```

First, watch one worked example and name only the state, move, artifact, and
proof. Next, complete one partly filled proof. Finally, prove one similar
artifact without hints.

The support fades, but the production bar does not. The final answer still has
to name the invariant, the failure prevented, and the evidence that would catch
a regression.

Use the Focus Cue when the chapter feels large. Ask only three questions:

```text
What state is changing?
What move changes it?
What proof remains after the move?
```

Read the Plain Version when your attention is tired or the chapter feels too
big. It gives the simple rule, why it matters, and what to watch before the
chapter asks you to hold more detail.

Then read the Production Artifact. It tells you what row, type, query, runbook,
test, policy record, or evidence packet this chapter wants you to build or
inspect.

Read the Implementation Map when you are ready to connect the idea to code.
It names the Rust module, SQL file, test, runbook, or operational surface
where the chapter becomes executable.

Read the Operator Question when you want to know why the chapter matters under
real pressure. It tells you what question an engineer or on-call operator
should be able to answer from durable evidence.

Read the Runtime Walkthrough when the idea still feels abstract. It follows
one pass from trigger to action to persistence to check, so the mechanism moves
in your head before you read the heavier detail.

Read the Acceptance Gate before you treat a chapter as done. It tells you the
minimum evidence, validation path, and stop condition for that concept.

Use `What You Will Learn` as a focus map, not as a promise to memorize
everything. Ask four small questions:

```text
What must I explain?
What evidence must I inspect?
What behavior must I verify?
Which production artifact proves the idea?
```

Do not skip the production contract or operational checklist. Those sections
are where the book turns an idea into something you can run for real users.

When you return, read the testing, observability, security, and exercises
sections. The book stays simple in language, but it does not simplify away
Rust, Postgres, Rig, idempotency, leases, evaluation, security, or operations.

## Prerequisite Repair Map

You do not need to master every prerequisite before starting. Use repair, not a
long detour.

If a term blocks you, repair only the missing concept until you can name one
artifact and one proof. Then return to the chapter.

| If this is weak | Read first | Artifact to inspect | Proof to name |
| --- | --- | --- | --- |
| Rust newtypes and enums | Chapter 4 and Appendix O | A domain type and its constructor test | Bad raw values are rejected before workflow logic. |
| Type-state and composition | Chapter 4.5 | A builder or lifecycle type | An illegal transition is hard to express. |
| Postgres row locks and leases | Chapter 3 | `pick_due_job.sql` or `claim_scheduled_jobs.sql` | Two workers cannot own the same job row. |
| Rig model and tool boundary | Chapter 6 | `rig_runner.rs` and `agent_output.rs` | Raw provider output becomes parsed domain output or typed failure. |
| Idempotency and side effects | Chapter 12 | Idempotency key and receipt path | Duplicate intent does not create duplicate external action. |
| SRE vocabulary | Chapters 15, 21, and 23 | SLI query, metric, trace id, and runbook query | An operator can explain one job and one fleet symptom. |
| Agent evaluation | Chapters 17 and 27 | Evaluation receipt and dataset version | Prompt or model change cannot promote without behavior evidence. |
| Agent security | Chapters 16 and 28 | Approval, authorization, sandbox, and audit event | Untrusted text cannot grant authority or execute risky action. |
| Recovery and replay | Chapter 29 and Appendix Y | Restore/replay packet and side-effect receipt | Restored work resumes, reconciles, or quarantines without guessing. |

The repair loop is short:

```text
1. Read the Plain Version.
2. Inspect the artifact.
3. Name the proof.
4. Return to the chapter.
```

Do not use prerequisite repair to avoid hard material. Use it to keep the
learning path moving while still returning to durable state, typed boundaries,
evidence, and operational proof.

## Concept Ladder

The chapters are ordered as a dependency ladder. Later ideas depend on earlier
invariants. If a later chapter feels abstract, return to the rung that supplies
the missing promise.

| Rung | Concept added | Why it must come here | Chapters |
| --- | --- | --- | --- |
| 1 | Durable work before intelligence | A model call cannot be reliable until the work exists outside the process. | 1-3 |
| 2 | Typed boundaries before business logic | Workers cannot protect what the domain model cannot name. | 4-4.5 |
| 3 | Ownership before concurrency | Retries and parallel workers are unsafe until one worker can own one job. | 5, 13 |
| 4 | Provider isolation before failure policy | Provider errors must become typed system decisions before retry logic can be trusted. | 6, 14 |
| 5 | Idempotency before side effects | Repeating uncertain work is safe only when duplicate intent maps to one durable action path. | 8, 12, 16 |
| 6 | Evidence before operations | SLOs and runbooks need persisted facts, not hope or process-local logs. | 15, 21-24 |
| 7 | Versioning before long-horizon operation | Old jobs, prompts, policies, schemas, and specialist-agent handoffs must survive change before the system can run for years. | 18-20.1, 25-26 |
| 8 | Evaluation, memory, security, and recovery before maturity | A mature agent is not only available; it is behavior-tested, memory-bounded, abuse-resistant, restorable, and ready to evolve without losing evidence. | 27-30.5 |

This ladder is the book's main progression:

```text
durable -> typed -> owned -> isolated -> idempotent -> observable -> versioned -> mature
```

Do not skip a rung in your own system. For example, an SLO on job success is
not meaningful if jobs are not durable, and a retry policy is unsafe if side
effects do not have idempotency keys and receipts.

## Reader Role Paths

The first read should be sequential. After that, use the book by role. Each
path below keeps the same concept ladder but changes what you inspect first.

| Reader role | Start with | Then focus on | Evidence to collect |
| --- | --- | --- | --- |
| AI engineer | Chapters 1, 2, 6, 27, and 27.5 | Provider boundary, structured output, evaluation receipts, memory eligibility, behavior release gates. | Prompt/model version, eval dataset, rubric, grader result, memory source/scope evidence, and provider contract test. |
| Rust engineer | Chapters 4, 4.5, 5, 11, and Appendix O | Newtypes, typestate, typed errors, store conversion, worker lifecycle. | Constructors, enums, trait boundaries, conversion tests, clippy, and feature builds. |
| Platform or SRE engineer | Chapters 3, 13-15, 21-26, and 30.5 | Durable ledger, leases, retries, observability, SLOs, runbooks, incidents, releases, ownership, and scaling paths. | Queue queries, event timelines, burn alerts, pause/resume commands, postmortem actions, toil owners, and migration evidence. |
| Security or governance reviewer | Chapters 16, 27-29, and Appendices C, N, P | Approval state, behavior evaluation, memory retention, trust boundaries, abuse paths, restore and replay safety. | Policy decision, approval record, memory scope/retention record, audit event, threat model, evidence packet, and restore drill. |
| Founder or product owner | Production Scope, Chapters 8, 20, 20.1, 20.2, and 30, and Appendix E | Operating envelope, hardening controls, specialist handoffs, worked scenario, maturity target by job kind. | Readiness scorecard, risk classification, owner, gap, next change, and review date. |

This table is not a shortcut around prerequisites. It is a way to decide which
evidence you should inspect first once the shared model is in place.

## Chapter Contracts

Use this table as the map of the book. The "core question" is what the chapter
teaches you to answer. The "evidence" is what would prove that the concept is
implemented in a real system.

| Chapter | Core question | Production evidence |
| --- | --- | --- |
| System Model And Notation | How should the book name state, actors, transitions, evidence, and invariants? | Shared notation for jobs, leases, events, provider boundaries, policy decisions, evaluation receipts, and side-effect receipts. |
| Design Principles | Which rules should survive a change of framework, provider, or database? | Ten principles map mechanisms to production artifacts and review questions. |
| Production Scope And Trade-Offs | When is the Postgres-first design the right control surface, and when should another architecture own orchestration? | The design names its assumptions, alternatives, operating envelope, and evidence contract. |
| 1. The Problem | Why is a model call not enough? | A durable job exists before the model runs. |
| 2. The Mental Model | Which component owns intelligence, state, execution, and safety? | State, worker, model, events, and policy are separate boundaries. |
| 2.5 Guarantees | What does the system promise, and what does it not promise? | Failure semantics are written down before code depends on them. |
| 3. The Postgres Ledger | How does durable state coordinate workers? | SQL claims work with row locks, leases, idempotency, tracking tables, workflow state, retry state, receipts, and recovery. |
| 4. The Rust Domain Model | Which values deserve real types? | Domain newtypes and enums replace raw strings and booleans at boundaries. |
| 4.5 Typed Composition | How do newtypes and type-state reduce invalid usage? | Illegal lifecycle states are hard to express in code. |
| 5. The Worker Loop | How does one job move through the system? | Each transition records an event and preserves lease ownership. |
| 6. The Rig Boundary | Where should provider behavior enter the system? | Provider errors become typed retry or terminal decisions. |
| 7. Running the System Locally | What can be proved without external infrastructure? | The deterministic local run and tests exercise the same state machine. |
| 8. Production Hardening | Which controls turn a demo into a service? | Idempotency, leases, approval gates, observability, and secrets rules exist. |
| 9. Failure Modes | What mistakes make agent jobs unsafe? | Failures become visible state instead of hidden loops. |
| 10. Capstone | How do you extend the system without weakening it? | New commands, states, tables, invariants, and tests move together. |
| 11. Real Postgres Store | Where does the in-memory model meet the database? | Rows convert through validated domain types. |
| 12. Idempotency | How are duplicate requests made safe? | One logical request maps to one durable side-effect path. |
| 13. Leases and Cancellation | How does long-running work stay owned, timed, and stoppable? | Heartbeats, deadlines, cancellation, and recovery preserve explicit ownership and explicit time promises. |
| 14. Retry and Dead Letters | Which failures should run again? | Retry policy separates transient failures from terminal work. |
| 15. Observability and SLOs | How does an operator know the system is healthy? | State, events, metrics, traces, and SLOs explain the same workflow. |
| 16. Approval and Policy | How are risky actions controlled? | The model proposes; policy and approval decide; workers execute. |
| 17. Testing Agents | What should be tested before production traffic? | Unit, SQL, feature, live, and behavior tests cover the boundaries. |
| 18. Deployment | How does the service change while work is running? | Deploys preserve leases, secrets, credential lifecycle, shutdown, and runbook commands. |
| 19. Running for Years | How does the system survive time? | Versions, retention, credential rotation, data-protection requests, provider independence, and ownership remain inspectable. |
| 20. Blueprint | What is the complete shape of the system? | Every boundary has a responsibility, failure contract, and durable evidence surface. |
| 20.1 Agent Handoffs And Multi-Agent Coordination | How can multiple agents cooperate without losing ownership? | Source run, source agent, target agent, reason, idempotency key, decision, target job, and pending-handoff query. |
| 20.2 Worked Production Scenario | How do the controls cooperate on one risky request? | Duplicate intake, retry, approval, receipt, and operator review form one evidence chain. |
| 21. SLIs and SLOs | Which measurements represent reliability? | SLIs, SLOs, budgets, burn alerts, and query sources are explicit. |
| 22. Capacity and Quotas | How does the system avoid overload? | Backpressure, fairness, and provider quotas shape admission and workers. |
| 23. Runbooks | What should an operator do during stress? | Diagnostic commands answer queue, lease, deadline, dead-letter, active-run, retry, approval, failed-tool, receipt, and pause questions. |
| 24. Incidents | How does the team respond and learn? | Triage, mitigation, postmortems, and action items update the system. |
| 25. Releases | How do code, schema, prompt, and model changes roll out? | Version skew, canaries, migrations, and release receipts are controlled. |
| 26. Toil and Ownership | Who keeps the system healthy? | Toil budgets, automation candidates, and ownership are assigned. |
| 27. Evaluation | How do you know the agent is right, not only available? | Behavior evals are tied to prompt, model, policy, and tool versions. |
| 27.5 Agent Memory, Retrieval, And Retention | How should an agent remember useful context without turning memory into hidden state? | Memory records carry scope, kind, source, confidence, horizon, retention, and inspectable metadata. |
| 28. Security | Where can instructions, tools, memory, and users cross trust boundaries? | Threat models, tool contracts, memory controls, and incident paths exist. |
| 29. Disaster Recovery | How does the system restart after serious loss? | RPO, RTO, state inventory, restore drills, replay safety, and provider continuity are tested. |
| 30. Maturity Model | What should improve next? | Each job kind has a target reliability level and a next concrete upgrade. |
| 30.5 Scaling Paths After Postgres-First | How should the system evolve after the Postgres-first design is no longer enough? | Each new infrastructure component preserves a named invariant, comparable evidence, and rollback or coexistence plan. |
| 30.6 Temporal After Postgres-First | When should a workflow engine own execution mechanics? | Temporal workflow history reconciles with Postgres product rows, approvals, receipts, audit events, and traces. |
| 30.7 Kafka After Postgres-First | When should an event stream own distribution and replay? | Kafka records come from typed outbox events, and consumers prove idempotent processing with receipts. |

## What to Do While Reading

Read with a notebook or scratch file. After each chapter, write three lines for
your own system:

```text
The invariant I need:
The evidence that would prove it:
The smallest test or runbook query I can add:
```

This turns reading into engineering work. You are not trying to remember every
term. You are learning to convert agent risk into durable state, typed
boundaries, tests, and operator evidence.

Use Appendix B when a term feels clear but the design still feels fragile. The
misconception index repairs shortcuts such as "the model call is the workflow,"
"retry makes it safe," or "logs are enough evidence."

After finishing the main chapters, use the appendices as a focused toolbox:

- Use Appendix C as a production design review. The review should expose the
  next concrete gap in your own system.
- Use Appendix D when you want practice. Its failure drills ask you to apply the
  same invariants to new incidents before reading the expected reasoning.
- Use Appendix E when you need a readiness decision. Score one job kind at a
  time and require evidence for every required control. The companion code
  turns the same review into `job_kind_readiness_reviews`,
  `job_kind_readiness_review.sql`, and `JobKindReadinessReview`.
- Use Appendix F after each chapter. It asks you to recall the prerequisite,
  simulate the smallest mechanism, and name the production evidence.
- Use Appendix G when you want to connect an idea to the companion
  implementation. It maps controls to Rust modules, SQL files, tests, and
  runbook queries.
- Use Appendix H when the moving parts feel scattered. Its diagrams let you
  redraw the architecture, lifecycle, retry timeline, approval path,
  observability flow, release flow, and recovery flow.
- Use Appendix I when you are ready to change the system itself. Its labs ask
  you to preserve one invariant at a time and leave acceptance evidence.
- Use Appendix J when you want to transfer the design principles to another
  architecture. It maps each principle to a simulation, evidence, transfer
  question, and common review failure.
- Use Appendix K when you want to see the same system model across different
  risk profiles: incident triage, customer support replies, and billing
  adjustments. It separates demo version, prototype version, production
  version, and regulated/high-risk version so examples are not copied above
  their evidence level.
- Use Appendix L after chapters or parts for retrieval practice. It gives each
  chapter a Recall/Explain/Apply/Evidence prompt.
- Use Appendix M when the dependency structure feels fuzzy. It maps every
  chapter from prerequisite, to new concept, to mechanism, to production
  capability unlocked.
- Use Appendix N when you need to prove readiness or safety to another
  engineer. It gives evidence packets for launch, release, incident response,
  behavior evaluation, security, and restore/replay.
- Use Appendix O when you are ready to read the companion Rust and SQL code in
  the same order as the system model: manifest, domain, SQL ledger, store,
  worker, provider boundary, binaries, and validation.
- Use Appendix P when a concept feels clear but you want to recognize the
  broken production version. It maps each chapter from design smell, to
  production symptom, to corrective invariant, to evidence you can inspect.
- Use Appendix Q after the first sequential read when you want a role-specific
  operating path. It tells each reader role which chapters to revisit, which
  evidence to collect, and which practice artifact to produce.
- Use Appendix R when you want to audit the book against its own production
  requirements. It maps each requirement to the teaching chapter,
  implementation artifact, validation evidence, operator evidence, and reviewer
  question.
- Use Appendix S when an idea feels intuitive but needs a precise production
  definition. It maps each main chapter concept to state, actor, transition,
  evidence, and invariant language.
- Use Appendix T when the book feels like many separate controls. It follows
  one incident-triage request across identity, timeline, failure recovery, and
  reviewer questions so the controls become one production story.
- Use Appendix U when you are ready to build a dashboard, admin page, CLI, or
  internal console. It separates read views from typed, permissioned, audited
  operator actions.
- Use Appendix V when long-running reliability needs a calendar. It turns
  daily, weekly, monthly, quarterly, and incident-triggered reviews into
  evidence, owners, next-review dates, and escalation when reviews are missed.
- Use Appendix W when focus is the main bottleneck. It gives ADHD-friendly
  restart points, fast feedback loops, chapter cards, two-pass reading, code
  reading, SQL reading, runbook reading, and study-session loops without
  lowering the production standard.
- Use Appendix X when you need a fast restart card. It gives each main chapter
  one concept, one artifact, one proof, and one operator question so you can
  re-enter the book without rebuilding the whole context.
- Use Appendix Y before first production exposure. It turns the serious MVP
  into a job-kind launch packet with durable intake, worker ownership, Rig
  boundary proof, side-effect control, evaluation, observability, security,
  rollback, and recovery evidence.
- Use Appendix Z when a production term feels too large. It gives one small
  sentence, the exact term, the artifact to inspect, and the proof sentence for
  concepts such as lease, idempotency, evaluation receipt, release gate,
  restore drill, newtype, typestate, and trust boundary.
- Use Appendix AA when a full lab feels too large. It gives seven-minute
  production micro-drills where each small action ends with an inspectable
  artifact and one proof sentence. Use its artifact index when you need the
  exact file, query, test, or readiness command.
- Use Appendix AB when you want the whole book as a build ladder. It names the
  durable intake, typed boundary, worker ownership, Rig boundary, idempotency,
  approval, observability, evaluation, security, recovery, operations, and
  scaling milestones. Each row says what to build, inspect, run, prove, and
  where to stop if evidence is missing.
- Use Appendix AC when you want the failure-first spine of the book. It maps
  each major chapter from production failure, to false shortcut, to invariant,
  to artifact, to proof sentence so the book stays organized around
  engineering transformations rather than tool features.

## Summary

By the end of the book, a production agent should be explainable from three
distances:

```text
close:
  a Rust type, SQL query, or worker transition preserves one invariant

middle:
  a job timeline explains what happened from enqueue to terminal state

far:
  SLOs, evals, runbooks, and restore drills show whether the service is safe
```

If the system cannot be explained at all three distances, it is not ready to
run for years.

## Further Reading & Credible References

- **[John Sweller: Cognitive Load Theory and Computer Science Education](https://dl.acm.org/doi/10.1145/2839509.2844652)** (2016). A foundational paper explaining why explicit instruction and "Worked Examples" are superior to discovery-based learning for complex technical subjects. It provides the academic basis for the book's short, focused chapters.
- **[CAST: The Universal Design for Learning (UDL) Guidelines](https://udlguidelines.cast.org/)**. The industry standard for creating learning materials that provide multiple means of representation, action, and engagement. This book applies UDL by offering plain-language summaries, code artifacts, and formal proofs for every concept.
- **[Paas & van Merrienboer: Cognitive-Load Theory and Instructional Design](https://www.sciencedirect.com/science/article/pii/S095947520400030X)**. Research into the "Split-Attention Effect," which motivates the book's goal of keeping code and explanation in one visual field to reduce mental tax.
- **[Designing Data-Intensive Applications](https://dataintensive.net/)** (Martin Kleppmann). Grounding the book's reliability claims in established data-systems vocabulary.
