# Part I. The Core System

## Motivation

The first part builds the smallest system that deserves to be called reliable:

```text
durable job state
typed Rust boundaries
one worker transition loop
one model boundary
one local run that proves the lifecycle
```

Do not rush past this part. If the core loop is ambiguous, every production
feature later in the book will amplify that ambiguity.

## What You Already Know

You already know the easy version of the problem: a program can call a language
model and receive text. This part asks you to separate that useful trick from a
production system.

Keep one distinction in mind:

```text
model call:
  a request receives an answer

agent job:
  durable work moves through owned, typed, observable state
```

The chapters in this part deliberately slow down before adding operational
features. The goal is to make the smallest reliable unit visible enough that
later chapters can harden it without changing its identity.

## What This Part Adds

Part I turns a model call into an agent job:

```text
request -> job row -> lease -> agent step -> result -> event trail
```

Each chapter adds one boundary:

```text
problem framing       -> the model is one step, not the whole system
guarantees           -> at-least-once execution with idempotent boundaries
Postgres ledger      -> durable state and worker coordination
Rust domain model    -> named values and explicit states
worker loop          -> state transitions under lease ownership
Rig boundary         -> provider behavior behind a typed adapter
local execution      -> deterministic proof before infrastructure
failure modes        -> visible failure instead of hidden loops
capstone             -> extension without weakening invariants
```

## Running Example

The running example is an incident-triage job:

```text
input:
  "Analyze failed deployment logs"

agent step:
  produce a summary, likely cause, next action, and approval requirement

durable proof:
  job row, lease owner, attempts, result, and event timeline
```

This example is intentionally small. It has enough risk to need state,
evidence, and approval later, but it is simple enough to simulate in your head.
By the end of Part I, you should be able to trace this one job from enqueue to
terminal state without relying on process memory.

Read each chapter by asking where the important fact lives. If the fact lives
only in a stack frame, exception, prompt, or log line, the design is still too
fragile for the later production layers.

## Exit Criteria

You are ready for Part II when you can explain one job at three levels:

```text
row:
  which status is stored, who owns the lease, and when it can be recovered

code:
  which Rust type or trait protects the transition

timeline:
  which events explain the run from enqueue to terminal state
```

The key invariant is:

```text
The job exists durably before the model runs, and every later transition is
owned, typed, and observable.
```

## Summary

Part I is the foundation. It does not make the system operationally complete.
It gives every later production control a stable object to protect: one durable
agent job moving through explicit state.

## Further Reading and Sources



- [Anthropic: Building Effective Agents](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: helps distinguish deterministic workflows from agentic behavior before the book adds durable execution machinery.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: gives the data-systems vocabulary behind durable state, logs, transactions, and recoverable workflows.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: grounds the chapter's reliability claims in operational practice rather than agent-specific hype.