# Part II. Production Engineering

## Motivation

The second part turns the core loop into production machinery.

The question changes from:

```text
Can one job run correctly?
```

to:

```text
Can many jobs run safely while workers crash, providers fail, users retry
requests, and operators need evidence?
```

## What You Already Know

Part I gave you one durable, typed, owned job. You should already be able to
explain:

```text
where the work lives
who owns it while it runs
how the model is isolated
which events explain the path
which invariant survives the transition
```

Part II assumes that foundation. It does not replace the core loop. It adds the
controls required when many jobs, workers, failures, releases, and human
decisions meet the same state machine.

## What This Part Adds

Part II adds the controls that make the core state machine usable under real
traffic:

```text
real Postgres adapter
idempotency and side-effect isolation
leases, heartbeats, and cancellation
retry classification and dead letters
observability and SLO foundations
human approval and policy gates
production test rings
deployment and shutdown discipline
long-horizon versioning and retention
final system blueprint
worked production scenario
```

The central idea is that production reliability is not one feature. It is the
alignment of schema, types, worker behavior, tests, runbooks, and release
practice.

## Running Example

The incident-triage job now receives real production pressure:

```text
duplicate webhook:
  the same deployment failure arrives twice

provider timeout:
  the model route fails after the worker owns the lease

risky recommendation:
  the agent proposes a rollback

deployment:
  a newer worker starts while older jobs are still pending
```

The part teaches the controls that keep those situations from becoming
ambiguous. Idempotency collapses duplicate intent. Retry classification turns
temporary failure into scheduled work. Approval keeps a recommendation from
becoming authority. Versioning lets old work remain explainable after change.

Read each chapter by asking which ambiguity is being removed. Production
engineering is mostly the work of replacing "probably fine" with a durable next
state, a typed decision, or an operator-visible artifact.

## Exit Criteria

You are ready for Part III when a job can survive ordinary production pressure:

```text
duplicate request -> same logical work
worker crash -> lease expiry and recovery
provider timeout -> classified retry
bad input -> terminal evidence
risky action -> approval path
deploy -> old work remains readable
```

The key invariant is:

```text
Failure becomes durable state with a safe next action, not hidden control flow.
```

The worked scenario at the end of this part should feel unsurprising. It does
not add new theory. It shows the production controls cooperating on one risky
request.

## Summary

Part II is where the book stops being a local architecture exercise. The system
now has enough persistence, policy, retry, and deployment discipline to be
operated, inspected, and improved without pretending failures are rare.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
