# Part III. Operating The System

## Motivation

An agent job system is not finished when it can run. It becomes dependable when
operators can see, control, and improve it under stress.

Part III treats the agent system as a service with reliability objectives:

```text
What does healthy mean?
How much failure is acceptable?
When should traffic slow down?
What should an operator do first?
How does the team learn from incidents?
Who owns the work after launch?
```

## What You Already Know

Part II gave the job system production mechanics. You should already be able to
trace duplicate intake, lease recovery, retry, approval, versioning, and
deployment safety for one job kind.

Part III changes the unit of attention:

```text
before:
  one job is safe

now:
  the service is measurable, controllable, and owned
```

This shift matters because reliability failures often appear first as fleet
signals: queue age, error-budget burn, quota pressure, alert noise, toil, or a
pattern of incidents that no single job explains by itself.

## What This Part Adds

Part III adds the operating model:

```text
SLIs and SLOs
error budgets
capacity and backpressure
provider quota discipline
runbooks
incident response
postmortems
release engineering
toil reduction
ownership
```

These are not administrative extras. They are the feedback loops that keep a
long-running agent system from quietly drifting into an unsafe state.

## Running Example

The incident-triage system is now a service with operators:

```text
traffic grows:
  queue age and provider quota pressure increase

alerts fire:
  one job kind burns its start-latency SLO

operators respond:
  they inspect queue metrics, pause low-priority work, and preserve evidence

team learns:
  the postmortem adds a runbook query, alert change, or automation
```

The part teaches how to move from "the worker works" to "the team can operate
the system under stress without private memory."

Read each chapter by asking what an on-call engineer can do with the evidence.
If a dashboard, alert, or incident note does not lead to a concrete query,
decision, owner, or mitigation, it is not yet an operating control.

The strongest test for this part is handoff. A new operator should be able to
read the runbook, inspect the same durable evidence, and reach the same first
decision without asking the original implementer what the system meant.

## Exit Criteria

You are ready for Part IV when the system can answer operational questions
without guesswork:

```text
Is work flowing?
Which job kind is overloaded?
Which provider or policy version changed?
What should be paused?
What evidence belongs in the incident timeline?
Which recurring manual action should become automation?
```

The key invariant is:

```text
Every alert, release, and incident should point back to state, evidence, owner,
and next action.
```

## Summary

Part III makes the system governable. It gives the team the vocabulary and
control surfaces to operate agent jobs as a service rather than as a collection
of workers and dashboards.

## Further Reading and Sources

- [Anthropic: Building Effective Agents](./31-credible-resources-further-reading.md#agent-architecture) helps distinguish deterministic workflows from agentic behavior before the book adds durable execution machinery.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) gives the data-systems vocabulary behind durable state, logs, transactions, and recoverable workflows.
- [Google SRE books and resources](./31-credible-resources-further-reading.md#reliability-and-operations) grounds the chapter's reliability claims in operational practice rather than agent-specific hype.
