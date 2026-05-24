# 1. The Problem

## What You Will Learn

This chapter teaches you to:

- explain why a model call is not a reliable agent system;
- explain why capturing raw user intent as a durable event is the foundation of Data-Centric AI evaluation;
- inspect the missing durable state, typed boundary, retry rule, audit trail, and owner behind a demo;
- verify that work exists before the model starts thinking.

The production evidence is a durable job row, trace id, idempotency key, and
initial event recorded before Rig or any model provider runs.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** the operating envelope says why the book starts with one database and one worker.
- **Adds:** durable work before model intelligence.
- **Prepares:** separating intelligence, reliability, and product-control layers.

## Production Failure

A demo agent receives a customer request, calls the model, prints a useful
answer, and looks finished.

Then production adds ordinary failure:

```text
same request arrives twice
worker exits after the model call
operator asks what happened
no durable row exists
```

- **What breaks:** the system cannot prove whether the request became work.
- **False fix:** add more logs around `agent.run(...)`.
- **Design response:** create durable work with an idempotency key before any
  model call starts.

## Motivation

In production, the model call is not the workflow. A request can arrive twice, a worker can crash, an operator can need proof, or a customer can ask what happened after the process is gone.

Without durable work before intelligence, the system has no stable answer to the first operational question: did this request become recoverable work? This chapter establishes why the agent must be surrounded by durable state before it can be trusted.

This is the first place where the book separates impressive behavior from
reliable behavior.

An impressive demo can answer a prompt. A reliable agent system can answer a
harder question: what happened to this request after the process, model call,
or network connection failed?

That question changes the architecture. The system can no longer treat the
model response as the only important artifact. It needs an earlier artifact:
the durable record that the work exists at all.

## Plain Version

Read this as the simple version:

- **Simple rule:** A reliable agent is a worker with tools, memory, permissions, and durable evidence around a probabilistic model.
- **Why it matters:** Prompts alone cannot survive crashes, retries, approvals, audits, or real side effects.
- **What to watch:** Watch for hidden state, untyped tool calls, untracked side effects, and model output treated as trusted truth.

## What You Already Know

Start with these anchors:

- A model call can produce a useful answer.
- Useful output is not the same as recoverable work.
- A process can die before anyone sees the result.

This chapter adds: the durable agent job as the unit of reliability. The job,
not the response text, is what can be retried, audited, approved, observed, and
recovered.

## Focus Cue

Keep three things in view:

- **State:** a durable agent job row exists before the model performs work.
- **Move:** raw user intent becomes durable work before any model call, retry, or side effect is allowed.
- **Proof:** A job row exists before execution, and the model call is one transition inside that job.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** a durable intake row with an idempotency key before any model call starts.
- **Why it matters:** the system cannot recover work that only existed in process memory.
- **Done when:** a user request can be found in Postgres even if the worker or provider fails immediately afterward.


## Implementation Map

When you transition from reading to implementation, rely on this map as your guide. The primary surfaces you will interact with are `sql/enqueue_agent_job.sql`, `src/domain.rs`, and the intake path demonstrated in the companion crate. The core state transition here is meticulously persisting the raw user intent before ever calling the model or the worker. The evidence path demands that the incoming request unequivocally possesses an idempotency key, a durable database row, an active trace id, and an initial recorded event.

## Operator Question

Before you ship any architectural idea based on this foundation, you must answer one vital operational question: Did the user request legitimately become durable before any model or worker step ran? To answer this, you must explicitly inspect the idempotency key, the specific intake row, the trace id, the created event, and the initial status. You should immediately escalate the design to leadership if a failed process or an unexpected provider call leaves behind absolutely no database row to explain the failed request.

## Runtime Walkthrough

Follow the concept of durable intake as a single runtime pass. First, a trigger occurs when a user explicitly asks the agent to perform work. Next, the action requires the system to write that request into the database as durable work *before* the model is ever called. For persistence, the system must durably store the idempotency key, the trace context, the raw payload, and the initial status. Finally, the check requires verifying that the request remains completely explainable and recoverable even after an intentional, hard process death.

## Acceptance Gate

Do not move on until you can produce the minimum required evidence. You must be able to empirically prove that a user request effortlessly survives a hard process death injected before any model call starts. To validate this path, an operator must inspect the durable intake row, the explicit idempotency key, the trace id, and the initial event. Stop the design process immediately if the only proof that requested work ever existed is volatile process memory or transient provider logs.

## Micro-Lesson

If you need a concise summary before diving into the heavier mechanisms, remember this sequence: The pain arises because, in production, a simple model call is absolutely not a reliable workflow. The guiding rule is that a reliable agent is fundamentally a worker equipped with tools, memory, permissions, and durable evidence surrounding a probabilistic model. A tiny example of this is ensuring a durable agent job row actively exists in the database *before* the model performs any work. The resulting artifact is a durable intake row explicitly containing an idempotency key created before any model call starts. The ultimate proof of success is that a user request demonstrably survives process death before any model call ever begins.

## The First Production Question

Before asking whether the model is smart, ask whether the work exists.

That sentence may sound too simple, but it prevents a large class of production
failures. A request that only exists in a function call, HTTP handler, queue
callback, prompt string, or provider request is fragile. If the process dies,
the system may have no durable fact to recover from.

The first serious production question is therefore:

```text
Can I find this request after the process that received it is gone?
```

If the answer is no, the agent has not yet entered the reliability system. It
may produce a useful answer, but the business cannot safely retry it, audit it,
approve it, cancel it, or explain it.

The durable job row is the first yes.

## Worked Walkthrough

Start with the smallest agent demo.
The program sends a prompt to a model and prints the answer.
That can be useful for a prototype, but it does not yet describe a production operation.

Now give the same agent one tool: send an email.
The model writes a helpful draft and asks the tool to send it.
The provider accepts the email request.
Then the worker crashes before the database records success.

When the process restarts, the system sees no success record.
A naive retry sends the email again.
The duplicate email is not a model hallucination.
It is a system design failure.

The missing idea is durable operation identity.
The email needed a stable idempotency key before the first send attempt.
The run needed a row that could survive process death.
The side effect needed a receipt that could prove whether the external world already changed.

This is the central problem of reliable agents.
The LLM is uncertain, but the failure above happened even if the model behaved perfectly.
Production reliability comes from the deterministic shell around the uncertain reasoning step.

By the end of the book, the one-line demo becomes a controlled operation.
The system parses model output, validates it, checks policy, records the intended action, executes it once, stores the receipt, emits evidence, and makes the result observable.
That is the transformation this chapter begins.

The lesson is not that demos are bad. Demos are useful because they show that a
model can help with a task.

The lesson is that a demo hides the parts that production depends on. It hides
the identity of the work. It hides the owner of the attempt. It hides whether a
side effect already happened. It hides which version of the prompt and model
produced the output. It hides what an operator should do after a crash.

This book makes those hidden parts visible one by one.

## Intuition

Think of the system as a workshop.

Postgres is the notebook on the table. It records:

```text
what work exists
who picked it up
when the lease expires
what attempt this is
what happened at each step
what result was produced
```

The Rust worker is the person doing the work.

Rig is the expert the worker asks for one reasoning step.

The notebook matters because memory is what makes the system recoverable.

If the worker asks the expert a question and then forgets to write anything
down, the workshop cannot continue safely after an interruption. Another worker
does not know whether the question was asked, whether the answer was useful, or
whether the work already affected the outside world.

Postgres is not only storage in this book. It is the shared memory that lets
workers coordinate, recover, and explain themselves.

## Mechanism

The first architectural move is to change the unit of work. A script usually
has this shape:

```text
receive request -> call model -> return text
```

A reliable agent job has a different shape:

```text
receive request -> create durable job -> lease job -> call model -> record event -> transition state
```

This transition implements a **Single-Consumer Queue** pattern over your relational store. By establishing Postgres as the sole source of truth, we create a clear fault-tolerance boundary. The worker doesn't "know" what to do; it only knows what the notebook tells it.

That extra durable step is not ceremony. It is what lets the system answer
production questions after the original process is gone:

```text
Did the request arrive?
Was it already seen before?
Which worker owned it?
Did the model call start?
Did it fail before or after producing output?
Can retry happen without duplicating a side effect?
```

This is the recurring pattern in the book. We take an action that would
normally happen inside process memory and move the important fact into durable
state before the next risky step.

Once you see that pattern, most reliability mechanisms become easier to place.
Leases are durable ownership. Retry is durable future work. Approval is durable
permission. A receipt is durable proof that an external action already crossed
the boundary.

This is why the book starts with Postgres instead of a larger orchestration
stack. The first problem is not a missing workflow engine. The first problem is
missing durable facts.

Once the facts are explicit, you can decide whether the simple Postgres-backed
system is enough or whether a later chapter's scaling path is justified. Without
those facts, adding infrastructure only moves the mystery to a more expensive
place.

## Tiny Example

A job starts as:

```text
id: incident-123
status: pending
attempt_count: 0
instruction: "Analyze failed deployment logs"
```

A worker picks it:

```text
status: running
attempt_count: 1
locked_by: worker-a
locked_until: now + 5 minutes
```

If the Rig call succeeds:

```text
status: succeeded
result: { summary, next_action, approval }
```

If it fails:

```text
status: pending
run_at: now + backoff
last_error: "provider timeout"
```

The key detail is that both outcomes are represented as state. Success is not a
print statement, and failure is not an exception that disappears with the
process. The row tells the next worker, operator, test, or replay command what
should happen next.

Read the tiny case as:

```text
setup: a request arrives before the model is called
transition: intake creates a durable pending job
evidence: job id, idempotency key, trace id, and job_enqueued event exist
invariant: process death cannot erase accepted work
```

The tiny example is intentionally plain. It does not need multi-agent planning,
vector search, or a complex tool chain to teach the first invariant.

If the job row exists, the system has something to recover. If the idempotency
key exists, duplicate intake has a stable identity. If the trace id exists,
operators can connect the intake event to later worker, model, policy, and tool
events.

> ### 🎓 The Professor's Corner
>
> **Idempotency Keys: The "Don't Do It Twice" Stamp**
>
> In the real world, if you give a letter to two different people to mail, you might end up sending it twice. In computing, "Idempotency" is a fancy word for a simple concept: no matter how many times you receive the same request, you only perform the work once. 
> 
> Think of the **Idempotency Key** (like `local-demo:failed-deployment`) as a unique name for a specific piece of work. If a worker sees a name it already has in its notebook, it knows it can safely ignore the duplicate or point to the existing result. It’s the "name of the work," not just a random string of characters!

These are small pieces of data. Together, they are the difference between a
script that happened to run and a system that can be operated.

## Formal Definition

For this chapter, the precise definition is:

```text
An agent job is a durable unit of model-powered work whose existence and state are recorded before the model is allowed to run.
```

In the book's system model:

- **State:** a durable agent job row exists before the model performs work.
- **Actor:** the API or intake path creates the job, and the worker later executes it under the stored lifecycle.
- **Transition:** raw user intent becomes durable work before any model call, retry, or side effect is allowed.
- **Evidence:** A job row exists before execution, and the model call is one transition inside that job.
- **Invariant:** model-powered work is never the source of truth for its own existence.

## What Can Fail

| Signal | Production meaning |
| --- | --- |
| Design smell | The model call is treated as the workflow. |
| Production symptom | A process crash loses work or makes duplicate intake impossible to explain. |
| Corrective invariant | Durable work exists before intelligence runs. |
| Evidence to inspect | The enqueue path writes a job row and intake event before the model step. |


## Production Contract

The first production contract is simple:

```text
durable job row before model call
explicit state before retry
event evidence before operator action
typed result before side effect
```

If a model-powered task cannot satisfy those four conditions, it is still a
script. It may be useful, but it is not yet a reliable agent job.

This contract is deliberately small. It does not say the agent will always be
right. It does not say the provider will always respond. It does not say every
tool call is safe. It says the system will create a durable, typed place where
the work can be owned, retried, inspected, and completed or refused.

That is enough for the first chapter. Reliability begins when the system has a
stable fact to protect.

## Progressive Hardening Path

Migrating from a naive script to a resilient architecture is a progressive hardening path, usually driven by the increasing volume of pager alerts.

In the naive version, the model call is enthusiastically treated as the entire workflow. In this fragile state, if the model call is the workflow, crash recovery only starts *after* all evidence of the user's intent has already been irrevocably lost to the void of a terminated process.

The safer version improves upon this tragedy by ensuring durable work actually exists before intelligence runs. Suddenly, a durable job row gives the system something concrete to recover, retry, inspect, and mathematically own long before the expensive, unpredictable intelligence layer boots up.

The final, production-grade version hardens this completely. The enqueue path explicitly writes a job row and a formal intake event *before* the model step even begins. At this stage, operators can definitively prove the intake, the execution path, and the terminal outcome, even if the model hallucinates, the worker violently crashes, or the entire process fails. Use the naive approach only when you are recording a neat demo starting with a prompt. Use the safer approach when the work demands actual identity. Use the full production approach the moment aggressive retries, compliance audits, or external side effects enter your system.

## Testing Strategy

You must ruthlessly test this first invariant long before any actual model logic is permitted to run. In your unit or type tests, explicitly prove that the Rust intake command physically cannot construct executable work without demanding a job identity, a job kind, a strict idempotency key, and a typed instruction. Your persistence or boundary tests must unequivocally prove the Postgres enqueue path successfully writes the job row and the intake event to disk *before* the worker or Rig boundary can even attempt to call the model. Finally, your regression tests must violently simulate a complete process death immediately after request receipt but before model execution; recovery must then smoothly find durable work, rather than frantically searching for lost prompt text in a recycled container.

## Observability Strategy

You must closely observe the exact, fleeting moment when fragile intent becomes durable work. Emit structured `tracing` fields for the request id, job id, idempotency key, job kind, trace id, and intake status *before* the Rig call begins. You must record a formal operation event the instant the API successfully writes the job row and intake event; these are your cryptographic proof that work existed before the intelligence ran. Ultimately, the runbook query you construct must definitively answer whether a request that tragically crashed before the model call still possesses recoverable, durable work in the database.

## Security and Safety Considerations

Your absolute first safety boundary is the intake phase, placed firmly before the probabilistic model ever receives the work. You must treat the request body, the raw user instruction, the idempotency header, and any model-facing prompt input as wildly untrusted hostile data until it is strictly converted into typed job input. Mandatory authorization, secure sandboxing, and strict human approval decisions must be formally attached to the record before any job is permitted to execute a risky external tool or side effect. Always meticulously redact raw secrets from your intake logs, while strictly keeping the job id, idempotency key, job kind, and trace evidence perfectly visible for the inevitable compliance audit.

## Operational Checklist

Before relying on this pattern in production, operators must perform a strict, skeptical review of the system's boundaries.

First, verify the **State** boundary: ensure a user request incontrovertibly becomes a durable agent job before Rig or the underlying model is ever allowed to run. Second, inspect the **Boundary** transitions themselves: verify that the raw request body, instruction text, idempotency header, and tenant identity are forcefully parsed into strict, typed intake values. 

Third, rehearse your **Failure** modes: verify that a catastrophic crash occurring just before the model call successfully leaves a recoverable job row instead of permanently lost work. Fourth, validate your **Observability** pipeline: confirm that the trace id, job id, idempotency key, intake event, and job status can accurately answer whether a specific request actually survived. Finally, verify **Safety**: ensure that absolutely no external side effect is computationally possible during the intake phase unless the required policy, approval, and idempotency evidence already exist.

## Exercises

To test your operational mastery, write a decidedly negative test where the API receives a request, violently crashes right before Rig runs, and yet miraculously must not lose the idempotency record. You must explicitly explain which idempotency key, receipt, or state transition was supposed to prevent duplicate work from executing. Next, sketch the exact Postgres evidence: the `agent_jobs` and `operation_events` rows that undeniably prove the request actually became durable work. Finally, define or heavily refine the Rust type, enum, constructor, or typestate that forcefully represents `AgentJobId`, `IdempotencyKey`, `JobKind`, and `IntakeAccepted` as completely distinct types. Then, meticulously name the runbook question that proves this enforcement mechanism actually works at 3 AM.

## Self-Check

Before you move on, use this quick retrieval drill to solidify your operational paranoia. First, recall exactly what durable artifact must unconditionally exist before the model call begins. Next, be able to clearly explain why a "useful model output" is fundamentally not the unit of reliability. Then, apply this knowledge to a terrifying scenario: imagine the process crashes exactly after intake but exactly before Rig runs. What, precisely, should your recovery mechanism inspect? Finally, explicitly name the specific job row, idempotency key, trace id, and first event that categorically prove the work actually survived the crash.

## Summary

The model call itself is only one tiny, probabilistic step. The rock-solid, durable system purposefully built around that model is what actually makes the agent useful after inevitable failures, aggressive client retries, server restarts, and compliance audits.

The core invariant to remember is that a raw request must become durable, typed work long before the model or the Rig boundary ever runs. To enforce this, your architecture must rely on ensuring an accepted request permanently possesses an agent job row, a strict idempotency key, an operation event, a trace id, and a recoverable status. 

Moving forward, remember the golden rule: every single later reliability mechanism in this architecture stubbornly starts from this very first promise—work exists safely outside the volatile process.

## Changed Understanding

Before reading this chapter, a simple model call may have looked exactly like the primary unit of agent work. After this chapter, you should understand that the unglamorous, durable job row is the true unit of reliability; the expensive model call is merely one observable transition hidden safely inside it. Moving forward, keep in mind that you must always aggressively inspect the durable intake row, the trace id, the idempotency key, and the very first operation event.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
