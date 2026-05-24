# Appendix N. Production Evidence Packets

## How to Use These Packets

This appendix turns the book's reliability ideas into reviewable production
evidence. Use it when a team needs to decide whether one agent job kind can
launch, receive more autonomy, survive a release, recover from an incident, or
run for years.

An evidence packet is stronger than a checklist because it names:

```text
claim:
  what the team believes is true

evidence:
  which artifact proves it

owner:
  who maintains the proof

expiry:
  when the proof must be refreshed

gap:
  what is still unsafe, partial, or unproven
```

If a claim has no artifact, it is not production evidence. It is intent.

## Packet 1: Job-Kind Launch Packet

Use this before a new job kind handles real traffic.

| Fault-tolerance claim | Evidence artifact | Owner | Expiry or review trigger |
| --- | --- | --- | --- |
| Work is durable before model execution. | Enqueue transaction, job row, idempotency key, enqueue event. | Service owner | Schema or intake change |
| HTTP admission is typed before enqueue. | API request-to-command conversion, `Idempotency-Key` validation, domain validation error tests. | API owner | Intake contract or auth change |
| The state model rejects invalid lifecycle states. | Rust enum, database constraint, transition tests. | Rust owner | State or migration change |
| Only the lease owner can mutate running work. | Pick, heartbeat, success, and retry predicates and tests. | Worker owner | Worker concurrency change |
| Time promises are explicit. | Timeout policy, deadline field, breached-deadline query, retry/cancel/escalate/dead-letter action. | Reliability owner | Job deadline or SLO change |
| Stop intent is durable. | Cancellation request row, typed cancellation lifecycle, pending-cancellation query, applied/ignored/expired outcomes. | On-call owner | Cancellation policy or drain behavior change |
| Retry is typed, bounded, and visible. | RetryDisposition, backoff policy, attempt cap, retry events, dead-letter query. | Reliability owner | Provider or error-class change |
| Provider behavior is isolated. | AgentRunner boundary, adapter conversion, provider compatibility check. | Provider owner | Model or provider route change |
| Risky side effects are gated. | Proposal state, policy version, approval record, side-effect receipt. | Policy owner | New action type |
| Operators can investigate. | `/healthz`, `/readyz`, `/metrics`, queue metrics, event timeline query, runbook commands. | On-call owner | New alert or incident |

Minimum launch rule:

```text
no durable work -> no launch
no typed API admission -> no public or webhook traffic
no lease ownership -> no parallel workers
no idempotency -> no retried intake
no event timeline -> no production incident support
no owner -> no continuous operation
```

## Packet 2: Release Packet

Use this before changing code, schema, prompt, model, policy, or tool behavior.

| Release surface | Required evidence | Question the packet must answer |
| --- | --- | --- |
| Code | Build id, tests, clippy, audit, rollback path. | Can old jobs still be processed? |
| Schema | Migration plan, expand/contract step, rollback risk, affected queries. | Can old and new workers coexist safely? |
| Prompt | Prompt version, evaluation receipt, failure examples, rollback version. | Did behavior improve without breaking protected cases? |
| Model | Model route, provider contract check, latency/cost note, fallback plan. | What changes if provider behavior or cost shifts? |
| Policy | Policy version, approval changes, denied examples, operator note. | Can risky action happen with the wrong authority? |
| Tool | Tool contract, auth scope, input validation, output validation, audit path. | Can untrusted text reach the tool boundary? |

Release packets should include a compatibility statement:

```text
old job rows:
old event rows:
old receipts:
old approvals:
rollback path:
known unsafe replay cases:
```

This makes release engineering concrete. The question is not "did CI pass?"
The question is whether old work, old evidence, and old side-effect receipts
remain understandable after the change.

## Packet 3: Incident Packet

Use this during and after an incident.

```text
incident id:
job kind:
user-visible impact:
failed invariant:
first bad event:
affected versions:
mitigation:
evidence preserved:
unsafe actions blocked:
follow-up owner:
review date:
```

Attach these artifacts:

| Artifact | Why it belongs |
| --- | --- |
| Queue metrics by kind | Shows whether the incident is isolated or systemic. |
| SLI measurement rows | Shows which SLO promise was burned, which window was affected, and whether the evidence came from durable state. |
| Job event timeline | Explains one representative execution path. |
| Running jobs past deadline | Shows whether the incident is caused by breached time promises rather than lost ownership. |
| Pending cancellation requests | Shows whether stop intent is waiting, stale, or aimed at already terminal work. |
| Audit events by run | Shows who or what made important decisions and which subject was affected. |
| Operation events by job | Shows runtime symptoms, severity, and job/run correlation. |
| Dead jobs by reason | Shows dominant failure classes. |
| Expired lease query | Separates crashed workers from provider or policy failures. |
| Release gate status | Connects the incident to code, schema, prompt, model, policy, tool, canary, blocker, signoff, and rollback evidence. |
| Evaluation receipt | Shows whether behavior risk was known before release. |
| Failure drill status | Shows whether the same failure had a rehearsed hypothesis, blast radius, rollback action, and evidence result. |
| Postmortem action items | Turns the incident into system changes. |

An incident packet should preserve evidence before cleanup. Deleting dead jobs,
clearing errors, or patching state without an event may make dashboards look
better while making the system less learnable.

## Packet 3.5: Fault-Tolerance Packet

Use this when a job kind must keep operating through worker, zone, provider,
or control-plane failures.

| Claim | Evidence artifact | Owner | Expiry or review trigger |
| --- | --- | --- | --- |
| Execution is isolated from non-critical control services. | `fault_tolerance_reviews.control_plane_status`, `execution_plane_status`, and static-stability mode. | Reliability owner | Control-plane dependency change |
| Approved state can survive a control-plane outage. | Last-known-good prompt, model, and policy versions. | Release owner | Prompt, model, or policy release |
| Important workers are redundant. | Redundant worker count, minimum worker count, and isolated failure domain. | Worker owner | Worker pool or deployment topology change |
| Failover is practiced. | `failure_drill_runs` and `fault_tolerance_readiness.sql`. | On-call owner | Drill expiry, incident, or topology change |
| Releases do not reach everyone at once. | `release_gate_runs`, canary decision, rollback plan, and production channel evidence. | Release owner | Release process or risk-class change |

Minimum packet rule:

```text
no last-known-good versions -> no static stability claim
no redundant workers -> no production redundancy claim
no passed drill -> no failover confidence
no release gate -> no progressive delivery claim
```

## Packet 4: Behavior Evaluation Packet

Use this before promoting prompt or model behavior.

```text
job kind:
prompt version:
model route:
tool version:
policy version:
dataset version:
rubric version:
grader:
score:
known failures:
promotion decision:
reviewer:
```

Add the release decision fields when the evaluation is part of promotion:

```text
release candidate:
release gate decision:
blockers:
canary percentage:
rollback plan:
operator signoff:
```

A serious evaluation packet includes both pass and fail examples:

| Example type | Evidence to keep |
| --- | --- |
| Expected success | Input, retrieved context snapshot, expected behavior, actual behavior, score. |
| Protected failure | Input that should be refused, gated, escalated, or dead-lettered. |
| Regression case | Previously fixed behavior that must not return. |
| Boundary case | Tool, memory, tenant, policy, or side-effect edge case. |
| Human review sample | Reviewer note explaining why the model output is acceptable or unsafe. |

Evaluation evidence is production evidence only when it is tied to versions.
Without versions, a future incident cannot tell which behavior was actually
approved.

## Packet 4.5: First-User Launch Packet

Use this before a job kind touches real users.

```text
job kind:
target level:
risk class:
launch decision:
owner:
durable intake proof:
worker ownership proof:
provider boundary proof:
side-effect control proof:
policy or approval proof:
observability proof:
evaluation proof:
security proof:
rollback or pause plan:
restore and replay note:
known gaps:
reviewer:
next review date:
```

Store the packet in `job_kind_launch_packets` and inspect it through
`job_kind_launch_packet_status.sql`. The row is not a replacement for judgment.
It is the durable place where judgment leaves evidence.

## Packet 5: Security And Trust Packet

Use this before a job kind gains tools, memory access, tenant data, or external
side effects.

| Trust boundary | Evidence artifact | Failure prevented |
| --- | --- | --- |
| User input | Input validation and prompt-injection test. | Untrusted text controls hidden instructions. |
| Retrieved context | Retrieval snapshot and source policy. | Stale or unauthorized context drives action. |
| Tool call | Tool schema, auth scope, output validation, audit event, operation event. | Model text becomes unchecked authority. |
| Agent handoff | Source run, source agent, target agent, reason, payload, idempotency key, target job, decision evidence, pending-handoff query. | Responsibility disappears between specialist agents. |
| Tool resources | Sandbox policy, egress allowlist, scratch path rule, secret-access mode, sandbox event. | Prompt injection chooses arbitrary network, filesystem, or secret exposure. |
| Credential lifecycle | Secret reference, owner, rotation due date, exposure status, revocation evidence, and `credential_rotation_review.sql`. | Secret values leak into prompts, logs, payloads, or informal notes. |
| Memory write | Memory scope, kind, source, confidence, horizon, retention policy, redacted content policy, and approval where needed. | Unsafe data persists across future jobs. |
| Side effect | Policy decision, approval record, idempotency key, receipt. | Irreversible action bypasses control. |
| Operator access | Role, reason, audit event, break-glass rule. | Humans mutate state without accountability. |

Security packets should state the smallest authority needed:

```text
read scope:
write scope:
tool scope:
tenant scope:
approval threshold:
audit event:
revocation path:
```

The model should never be the thing that grants authority. The model can
recommend. Deterministic policy, credentials, and audited state decide.

## Packet 6: Restore And Replay Packet

Use this before claiming the system can recover from serious loss.

```text
backup source:
restore target:
RPO:
RTO:
last restore drill:
jobs restored:
events restored:
receipts restored:
approvals restored:
jobs quarantined:
replay rule:
operator signoff:
```

The packet must answer:

| Question | Evidence |
| --- | --- |
| Which data was restored? | State inventory and row counts. |
| Which jobs can safely replay? | Job state, attempt count, idempotency key, and side-effect receipt check. |
| Which jobs must be quarantined? | Missing payload, missing receipt, unknown version, or unsafe action. |
| Which external actions already happened? | Side-effect receipts and external correlation ids. |
| Which versions are needed to interpret old work? | Prompt, model, policy, tool, schema, and worker versions. |
| Who accepted the recovery result? | Operator signoff and incident or drill record. |

Backup is not recovery. Recovery is proven only when restored state can be
interpreted, replayed, quarantined, or stopped without guessing.

## Packet 7: Temporal Adoption Packet

Use this before moving one job kind from the Postgres worker loop into Temporal.

```text
job kind:
strained workflow invariant:
old Postgres owner:
new Temporal owner:
workflow id rule:
activity id rule:
temporal workflow link table:
temporal activity receipt table:
product ledger rows retained:
approval and receipt mapping:
reconciliation query:
rollback path:
operator owner:
review date:
```

The packet must answer:

| Question | Evidence |
| --- | --- |
| Which invariant is Temporal taking over? | A named timer, cancellation, replay, child-workflow, or long-running orchestration problem that is already painful in the Postgres-first worker. |
| Which facts remain product truth? | Postgres job rows, agent runs, tool calls, approval requests, side-effect receipts, audit events, and operation events. |
| How do operators reconcile both histories? | `temporal_workflow_links`, `temporal_activity_receipts`, workflow id mapping, workflow search attributes, trace id, activity receipts, and `temporal_workflow_reconciliation.sql`. |
| Are activities idempotent? | Activity input types, idempotency keys, receipt tables, retry policy, and duplicate-activity tests. |
| Can the system roll back? | Coexistence or rollback plan that names which new work can return to Postgres workers and which in-flight workflows must drain. |

Temporal adoption fails the packet when workflow history becomes the only place
to understand a product decision. Temporal can prove that execution advanced.
It should not be the only proof that a risky business action was allowed.

## Packet 8: Kafka Adoption Packet

Use this before publishing agent events through Kafka.

```text
event family:
product source table:
outbox event kind:
event envelope version:
topic:
partition key:
publish receipt table:
consumer groups:
consumer receipt table:
replay-safety query:
replay rule:
redaction rule:
rollback path:
operator owner:
review date:
```

The packet must answer:

| Question | Evidence |
| --- | --- |
| Which product fact is being distributed? | The Postgres row, outbox event, event id, aggregate id, tenant scope, and schema version. |
| Why is Kafka needed now? | Baseline fanout, replay, consumer independence, polling load, or cross-service distribution evidence from the Postgres-first system. |
| Can consumers process events idempotently? | `kafka_consumer_receipts`, unique event-processing keys, projection tests, and replay drill evidence. |
| Can one event be traced end to end? | Outbox id, `kafka_publish_receipts`, Kafka topic-partition-offset, consumer group, consumer receipt, projection row, operation event, trace id, and `kafka_replay_safety_by_event.sql`. |
| Is the payload safe to distribute? | Schema compatibility policy, authorization boundary, tenant boundary, redaction rule, and data-retention note. |

Kafka adoption fails the packet when Kafka becomes the product source of truth
before the product event is typed, authorized, published from the outbox, and
acknowledged by idempotent consumers.

## Final Packet Review

Before accepting any packet, ask:

```text
Can a new engineer inspect this packet without private memory?
Can an incident reviewer reconstruct the decision later?
Does every high-risk claim point to a durable artifact?
Is the owner still current?
Is the evidence fresh enough for the risk?
Does the packet name gaps instead of hiding them?
```

If the answer is no, the packet is incomplete.

## Summary

Production rigor means claims are reviewable. A reliable AI agent is not ready
because the model sounds good or the demo succeeds. It is ready when the team
can point to durable evidence for the job kind, release, incident, evaluation,
security boundary, and recovery path.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
