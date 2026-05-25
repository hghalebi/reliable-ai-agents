# Part IV. World-Class Reliability

## Motivation

The final part asks a harder question:

```text
Can this agent system keep being correct as behavior, threats, dependencies,
and organizational memory change over years?
```

Availability is not enough for an AI agent. The system must also preserve
behavior quality, trust boundaries, disaster recovery, and a clear improvement
path.

## What You Already Know

Part III made the service observable and governable. You should already be able
to explain the SLO, runbook, incident, release, toil, and ownership story for
one job kind.

Part IV raises the bar:

```text
available:
  the system answers and processes work

reliable:
  the system keeps behavior, authority, recovery, and improvement under control
```

For AI agents, this distinction is not optional. A system can be available
while producing worse behavior, accepting malicious instructions, losing
restore safety, or drifting beyond what the team can understand.

## What This Part Adds

Part IV adds the long-horizon controls:

```text
behavior evaluation
prompt and model release evidence
typed memory, retrieval, and retention
security and abuse boundaries
data protection, retention, and privacy operations
tenant isolation and multi-tenant agent safety
tool and memory trust controls
disaster recovery
provider continuity
extreme fault tolerance through isolation, redundancy, and static stability
reliability maturity planning
optional Temporal adoption when workflow execution becomes the strained invariant
optional Kafka adoption when event distribution and replay become the strained invariant
```

The central distinction is:

```text
service reliability -> the system keeps running
behavior reliability -> the system keeps making acceptable decisions
```

Production agents need both.

## Running Example

The incident-triage system now faces long-horizon risk:

```text
prompt/model change:
  a new route gives better summaries but worse rollback recommendations

trust-boundary attack:
  untrusted incident text tries to authorize a tool call

memory drift:
  an old model-generated note tries to influence a future incident run

restore event:
  backup recovery brings back jobs with some side effects already executed

maturity planning:
  triage, support, and billing jobs need different autonomy levels
```

The part teaches how to preserve behavior and safety after the original launch
excitement is gone. The system must remain testable, restorable, reviewable,
and improvable.

Read each chapter by asking what would still be true one year later. A mature
agent system should not depend on the original builder remembering why a
prompt, policy, backup, or trust boundary was considered safe.

The strongest test for this part is drift. When the model, attacker, provider,
team, and product policy have all changed, the system should still have enough
evidence to decide what is safe to promote, block, restore, or retire.

## Exit Criteria

The system is mature enough to keep improving when it can answer:

```text
Which eval proves this prompt/model change is acceptable?
Which memory records are eligible to influence this run?
Which trust boundary prevents untrusted text from becoming authority?
Which restore drill proves old work can resume safely?
Which job kinds can keep executing from last-known-good state when the control
plane is unavailable?
Which maturity level does each job kind need?
What is the next concrete reliability upgrade?
If Temporal is proposed, what workflow invariant moves and what product
evidence remains in Postgres?
If Kafka is proposed, which typed event is distributed and how is replay made
safe?
```

The key invariant is:

```text
Long-running agent systems need evidence for behavior, security, recovery, and
improvement, not only uptime.
```

## Summary

Part IV completes the production picture. It turns a reliable job service into
a system that can keep its behavior, security posture, and recovery story
inspectable over a long period of change.

## Further Reading and Sources



- [Anthropic: Building Effective Agents](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: helps distinguish deterministic workflows from agentic behavior before the book adds durable execution machinery.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: gives the data-systems vocabulary behind durable state, logs, transactions, and recoverable workflows.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) Read this because: grounds the chapter's reliability claims in operational practice rather than agent-specific hype.