# Appendix C. Production Design Review

## How to Use This Review

Use this review before shipping an agent job system, after a major redesign, or
after an incident. It is not a compliance checklist. It is a way to test
whether the book's ideas have become real engineering controls.

For each prompt, write three things:

```text
current evidence:
known gap:
next concrete change:
```

If the evidence is only "the code probably does that," the evidence is too
weak. Look for rows, constraints, types, tests, metrics, runbooks, receipts,
approval records, eval results, or restore-drill notes.

## Design Review Questions

| Area | Question | Evidence |
| --- | --- | --- |
| Durable work | Does the job exist before the model runs? | Enqueue transaction, job row, idempotency key. |
| State model | Can every job be in only one valid state? | Rust enum, database constraint, transition tests. |
| Ownership | Can only the lease owner finish, retry, or heartbeat? | SQL predicates and lease-owner tests. |
| Timeout policy | What happens when work is still leased but past its deadline? | Timeout policy type, deadline fields, breached-deadline query. |
| Cancellation | Is stop intent durable before the job is stopped? | Cancellation request row, requester/source/mode/reason, applied or ignored evidence. |
| Duplicate intake | Does the same logical request map to one job? | Unique idempotency key and duplicate-suppression event. |
| Retry | Is every retry based on typed failure classification? | Transient/permanent retry tests and event timeline. |
| Side effects | Are irreversible actions separated from model output? | Proposal, approval, idempotency key, receipt. |
| Provider boundary | Can the model provider change without rewriting the worker? | `AgentRunner` boundary and provider adapter tests. |
| Evaluation | Does behavior release have evidence? | Dataset version, rubric, score, reviewer, prompt/model version. |
| Security | Can untrusted text authorize tools or memory changes? | Tool authorization, scoped permissions, audit events. |
| Observability | Can one job and the fleet be explained separately? | Trace/event timeline plus queue metrics. |
| SLOs | Does alerting map to an objective and an owner? | SLI query, SLO window, burn-rate alert, runbook. |
| Capacity | What happens when provider quota is lower than demand? | Admission control, concurrency limit, backpressure signal. |
| Runbooks | Can an operator inspect, pause, replay, or cancel safely? | Runbook commands and permission model. |
| Release | Can old work survive schema, prompt, model, and policy changes? | Versioned rows, expand/contract migration, canary criteria. |
| Recovery | Can the system restore and resume without duplicate side effects? | Backup drill, RPO/RTO, receipt-aware replay path. |
| Ownership | Who fixes the next failure? | Service owner, escalation path, toil review, postmortem actions. |

## Failure Injection Prompts

Use these prompts to test whether the design holds under stress:

```text
The webhook arrives twice.
The worker crashes after the model call but before result write.
The provider times out for ten minutes.
The worker keeps heartbeating, but the job misses its deadline.
The provider returns malformed structured output.
The model recommends a risky action.
The approval service is unavailable.
The side-effect worker crashes after executing the action but before receipt.
The newest prompt fails an evaluation.
The database is restored from a backup that is fifteen minutes old.
The primary provider changes an API field.
```

For each prompt, answer:

```text
Which state changes?
Which event is recorded?
Which retry or stop decision happens?
Which human or policy gate is involved?
Which evidence would an operator inspect?
```

## Evidence Review

A production design review should reject vague answers.

Weak evidence:

```text
The worker should retry.
The model should not do dangerous things.
The dashboard shows some errors.
The team can check logs.
```

Strong evidence:

```text
retry_or_dead preserves terminal states and records retry_scheduled
rollback proposals require policy_version and approval actor
queue_metrics exposes oldest_pending_age by job kind
job_event_timeline explains one job from enqueue to terminal state
restore drill measured resume behavior with receipt-aware replay
```

The shift is from intention to proof.

## Final Readiness Bar

Do not claim production readiness until the system can satisfy this chain:

```text
durable before execution
typed before business logic
idempotent before retry
deadline before escalation
policy before side effect
receipt before replay
metrics before alerting
runbook before incident
eval before behavior release
restore drill before long-horizon claim
owner before handoff
```

This chain is intentionally strict. A long-running agent system fails at the
weakest missing link, not at the part that looked most impressive in a local
run.

## Summary

The review turns the book into an engineering practice. If each answer points
to concrete evidence, the reader has not only understood the concepts; they
have started turning them into a production system.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
