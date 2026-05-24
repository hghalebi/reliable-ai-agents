# Appendix T. Running Evidence Thread

## How to Use This Thread

This appendix gives the book one continuous production thread. Use it when the
chapters feel correct one at a time but the whole system still feels hard to
hold in your head.

The thread follows one incident-triage request:

```text
incident id: inc-7841
service: payments-api
symptom: error rate increased after deploy
requested action: recommend one safe next action
risk: a rollback may affect real users
```

The agent is allowed to investigate and recommend. It is not allowed to turn a
model sentence into a rollback by itself. The running question is:

```text
Can we prove every important move from durable evidence?
```

If the answer is no, the system is not reliable yet. It may still be a useful
prototype, but it is not ready for long-running production autonomy.

## Motivation

Production reliability can feel fragmented when each chapter focuses on a
different control. One chapter talks about leases, another about idempotency,
another about model output, another about approval, and another about restore.

The real system does not experience those controls separately. One request
flows through all of them. This appendix exists so the reader can carry one
request through the whole book and see how each control strengthens the same
evidence chain.

## Mental Model

Read this as the simplest accurate version:

```text
one user-visible request
  -> one idempotency key
  -> one durable job
  -> one worker lease
  -> one agent run
  -> one typed model result
  -> one policy decision
  -> one human approval
  -> one side-effect receipt
  -> one reviewable event timeline
```

The model is only one step. The system around it is what makes the step safe to
repeat, pause, approve, audit, and recover.

## Identity Thread

The easiest way to lose a production agent is to lose the identity of the work.
Keep these identifiers connected from the first request to the final review:

| Identity | Meaning | Evidence surface |
| --- | --- | --- |
| `incident_id` | The business event the agent is helping with. | Request payload and audit subject. |
| `idempotency_key` | The duplicate-suppression key for the logical request. | Admission row and duplicate-intake event. |
| `job_id` | The durable unit of scheduled work. | `agent_jobs`, `scheduled_jobs`, or `background_jobs`. |
| `agent_run_id` | The model-backed execution attempt. | `agent_runs` and trace context. |
| `tool_call_id` | The proposed or executed tool operation. | `tool_calls` and tool receipts. |
| `approval_request_id` | The human control record for risky action. | `human_approval_requests`. |
| `receipt_id` | Proof that an external side effect crossed the boundary. | `side_effect_receipts` or outbox records. |
| `trace_id` | The correlation id for runtime reconstruction. | `operation_events`, traces, and logs. |

The names are not cosmetic. They stop different pieces of evidence from
floating away from each other. A production reviewer should be able to start
from any one of these identifiers and find the rest.

## Timeline Thread

The same incident should move through the system as a visible timeline:

| Step | State change | What proves it |
| --- | --- | --- |
| Admission | raw request becomes a typed durable command. | Valid request, idempotency key, accepted admission event. |
| Duplicate delivery | second webhook maps to the existing job. | Existing-job lookup and duplicate-suppressed event. |
| Claim | pending job becomes leased running work. | `locked_by`, `locked_until`, attempt count, pick event. |
| Provider timeout | first model attempt fails transiently. | Failure class, retry event, future `next_run_at`. |
| Retry | job becomes due again after backoff. | Attempt budget and scheduled retry query. |
| Model result | raw provider output becomes typed agent output. | Parser, schema validation, semantic validation, policy input. |
| Policy gate | rollback recommendation becomes risky proposal. | Policy version, risk level, approval requirement. |
| Human approval | reviewer accepts or rejects the proposal. | Actor, timestamp, reason, approval state. |
| Side effect | approved action crosses the external boundary. | Idempotency key, receipt, external correlation id. |
| Evaluation | behavior is judged for future release safety. | Dataset version, expected behavior, evaluation receipt. |
| Incident review | operator reconstructs the run. | Audit events, operation events, runbook query results. |

This table is the book in miniature. Every chapter adds one stronger way to
make a row in this timeline true.

## Chapter Thread

Use this map when a chapter feels isolated:

| Book area | Thread question | Evidence to inspect |
| --- | --- | --- |
| Problem and mental model | Is this an agent workflow or a prompt demo? | Durable job, tool boundary, permission boundary. |
| Postgres ledger | Can the request survive process death? | Job row, status, attempts, event ledger. |
| Rust domain model | Do important values have names? | Newtypes for ids, versions, keys, policies, and attempts. |
| Worker loop | Can only the owner advance the job? | Lease predicates and worker outcome events. |
| Rig boundary | Did provider text become trusted only after validation? | Raw output parser, typed result, tool request validation. |
| Idempotency | Can duplicate delivery or replay avoid duplicate action? | Idempotency key, existing-job lookup, side-effect receipt. |
| Cancellation and timeout | Can slow or unwanted work stop with evidence? | Deadline row, cancellation request, applied or ignored state. |
| Retry and dead letter | Is repeat work bounded and explainable? | Failure class, attempts, backoff, terminal reason. |
| Observability and SLOs | Can one run and the fleet both be explained? | Trace id, operation events, queue metrics, SLI rows. |
| Approval and policy | Does risk pause before action? | Policy decision, approval request, reviewer evidence. |
| Deployment and release | Can old work survive new code? | Prompt/model/policy/schema versions and compatibility report. |
| Evaluation | Can behavior drift block release? | Golden dataset, evaluation receipt, release gate. |
| Memory | Can remembered context be scoped and retained safely? | Memory scope, source, confidence, retention policy. |
| Security | Can untrusted text reach authority or secrets? | Authorization event, sandbox event, credential lifecycle review, denied-action evidence. |
| Data protection | Can privacy work be operated without informal notes? | Data-protection request, review query, policy version, completion evidence, and audit event. |
| Tenant isolation | Did any cross-tenant request avoid denial? | Actor tenant, requested tenant, authorization event, tenant-boundary review query, trace id, and audit event. |
| Recovery | Can restore avoid unsafe replay? | Receipt-aware replay decision and restore drill result. |
| Scaling path | Can a new queue, workflow engine, or event stream preserve evidence? | Invariant-to-component map, Temporal reconciliation, Kafka replay-safety proof, and coexistence test. |

If a chapter does not help answer its thread question, the chapter is probably
drifting away from the book's production promise.

## Failure Thread

Now replay the same request under failure:

```text
duplicate webhook arrives
worker crashes after claim
provider times out
model suggests rollback
reviewer is unavailable
worker crashes after external API call
restore happens from backup
```

The safe system does not need to be perfect. It needs each failure to become a
state with a next action:

| Failure | Safe state | Next action |
| --- | --- | --- |
| Duplicate webhook | existing job returned. | Record duplicate evidence and do not enqueue a second job. |
| Worker crash | lease expires. | Another worker recovers only after `locked_until`. |
| Provider timeout | retry scheduled. | Back off if the failure is transient and idempotent. |
| Risky recommendation | approval waiting. | Stop before action until policy and reviewer evidence exist. |
| Reviewer unavailable | escalation open. | Assign a human owner or keep the job waiting safely. |
| Crash after side effect | receipt uncertain or present. | Reconcile before replay; never assume no side effect happened. |
| Restore from backup | replay candidate classified. | Resume, reconcile, quarantine, or leave terminal work alone. |

This is why reliable agents are mostly ordinary production engineering around
an unreliable reasoning core. The core can be useful and still be bounded by
evidence.

## Reviewer Walkthrough

A reviewer should be able to ask these questions in order:

```text
1. What is the logical request?
2. What idempotency key represents it?
3. Which durable row owns the work?
4. Which worker owned the attempt?
5. Which provider, prompt, model, and policy versions were used?
6. What raw model output was parsed?
7. Which validation and policy checks passed?
8. Which human approved the risky action?
9. Which side-effect receipt proves what happened externally?
10. Which trace, audit event, and operation event reconstruct the timeline?
11. Which evaluation or release gate protects future behavior?
12. Which restore decision prevents unsafe replay?
```

If the team cannot answer one of these questions from durable evidence, that is
the next engineering task. Do not patch the explanation. Fix the evidence.

## Summary

The running evidence thread turns the whole book into one production story. A
reliable agent is not a single loop around a model. It is a typed, durable,
auditable state machine where one step may use model reasoning.

The simplest review rule is:

```text
same request, same identity, visible state, explicit owner, typed boundary,
safe retry, human gate when risky, receipt for side effects, evidence after failure
```

When that rule holds, the system can be operated. When it does not, the system
is still guessing.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
