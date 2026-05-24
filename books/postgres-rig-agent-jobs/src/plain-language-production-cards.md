# Appendix Z. Plain-Language Production Cards

## Purpose

Use this appendix when a production term feels too large.

The goal is not to make the system simpler than it is. The goal is to give the
reader a small first handle. The handle must still point to a real type, row,
query, event, test, runbook, receipt, policy record, or validation command.

This is the rule:

```text
say it small
make it exact
prove it
```

The small sentence helps attention. The exact term keeps rigor. The proof keeps
the claim production-real.

## How To Use One Card

Read one card at a time. Do not read the table like a dictionary.

Use four moves:

```text
1. Read the small sentence.
2. Name the exact production term.
3. Open or inspect the artifact.
4. Say the proof in one sentence.
```

Stop when the proof is clear. A short stop is better than reading five more
terms without being able to prove one of them.

## Small Sentence Shape

A good small sentence has four parts:

```text
thing -> move -> evidence -> promise
```

Example:

```text
The worker leases the job, records the event, and another worker can recover it after the lease expires.
```

That sentence is simple, but it is not vague. It names the thing, the move, the
evidence, and the promise.

Avoid sentences that sound simple but hide the mechanism:

```text
The system handles reliability.
```

That sentence gives the reader nothing to inspect.

## The Cards

| Small sentence | Exact term | Artifact to inspect | Proof sentence |
| --- | --- | --- | --- |
| Work must exist before the model starts. | Durable agent job | `agent_jobs` row | If the process dies, the row still shows that work was admitted. |
| One worker owns the job only for a while. | Lease | `locked_by` and `locked_until` | A stale worker cannot finish the job after ownership expires. |
| Doing work again must not mean doing the external action twice. | Idempotency | idempotency key and receipt | A duplicate request returns the same job or the same side-effect receipt. |
| A retry is a scheduled second attempt, not a blind loop. | Retry disposition | failure class, attempts, and `next_run_at` | Transient failures run later; permanent or exhausted failures stop. |
| Failed work should become visible state. | Dead letter | terminal failed job and failure history | Operators can query why the job stopped and what evidence led there. |
| Model text is not trusted state. | Raw model output | parser, validation, policy, and approval path | Provider output becomes domain data only after checks pass. |
| A tool call is a side effect boundary. | Typed tool request | `ToolInput<T>`, `tool_calls`, and policy evidence | The tool receives typed input only after validation and permission checks. |
| A person decision is state, not chat. | Human approval gate | approval row and audit event | Risky work waits until a durable approval or rejection exists. |
| Logs help, but rows prove. | Event ledger | job events, audit events, and operation events | An operator can reconstruct the transition without process memory. |
| A trace connects one request across boundaries. | Trace context | trace id on job, run, event, and provider usage rows | The same workflow can be followed through API, worker, model, and runbook evidence. |
| Reliability needs a measured promise. | SLI and SLO | SLI query and SLO target | The team can tell whether the user-facing promise is being met. |
| Behavior changes need evidence before release. | Evaluation receipt | dataset, rubric, prompt version, model version, and score | A prompt or model change promotes only when behavior evidence passes. |
| A release should stop when evidence is missing. | Release gate | `release_gate_runs` row and `release_gate_status.sql` | Promotion is blocked, canaried, or allowed only with evaluation, SLO, compatibility, version, approval, rollback, and signoff evidence. |
| Backup is not recovery until restore is practiced. | Restore drill | restore drill run and replay decision | The system proves RPO, RTO, and receipt-safe replay before disaster. |
| Two strings with different meanings need different types. | Newtype | `AgentId`, `ModelVersion`, or `IdempotencyKey` | The compiler prevents mixing values that look the same in storage. |
| Some moves should be impossible to call too early. | Typestate | builder or lifecycle type | Code exposes the next operation only after required evidence exists. |
| Database rows are raw at the edge. | Row conversion boundary | `Db...Row` to domain conversion | Invalid status, version, timestamp, or payload fails before worker logic. |
| The model cannot grant itself permission. | Trust boundary | authorization event, sandbox event, and policy result | Tool execution waits for controls outside the model. |
| A secret value should not become a database row. | Credential lifecycle | `credential_assets` row and `credential_rotation_review.sql` | Postgres stores secret references, owners, due dates, exposure status, and evidence, not the secret itself. |
| Memory is production data, not a note pile. | Agent memory record | memory scope, source, confidence, retention, and policy | Retrieved memory is allowed to influence a run only when scope and policy permit it. |
| Privacy work is a job, not a promise in chat. | Data-protection request | `data_protection_requests` row and `data_protection_review.sql` | Redaction, erasure, export, and retention-review work is open, overdue, or complete with policy-versioned evidence. |
| Scaling must preserve the proof trail. | Evidence-preserving migration | old/new evidence map and runbook update | Adding infrastructure does not hide state, ownership, idempotency, or audit evidence. |

## When A Term Still Feels Too Hard

Use this repair loop:

```text
term:
small sentence:
artifact:
proof:
next action:
```

Example:

```text
term: SLO
small sentence: the team writes down the reliability promise and measures it
artifact: SLI query plus target window
proof: the query shows whether recent jobs met the promise
next action: inspect the SLI query for terminal jobs
```

The repair is complete only when the `artifact` and `proof` lines are real.

## Small But Not Shallow

Simple language is allowed to remove friction. It is not allowed to remove the
production promise.

Use this test:

```text
Can this sentence tell me what to inspect?
Can this sentence tell me what can fail?
Can this sentence tell me what evidence proves the claim?
```

If not, make the sentence more concrete.

## Exercises

1. Pick one card. Cover the artifact column. Recreate it from the small
   sentence and exact term. Then check the table.

2. Pick one hard paragraph in a chapter. Rewrite it as one small sentence using
   `thing -> move -> evidence -> promise`.

3. Pick one production term from your own agent system. Fill the repair loop.
   Stop only when the artifact and proof are inspectable.

4. Pick one card and find the matching Rust file, SQL file, test, or runbook in
   Appendix O.

## Summary

Plain language is useful when it reduces the cost of starting. It is dangerous
when it hides the invariant.

Invariant: every simple sentence about a reliable agent should still point to
state, movement, evidence, and a production promise.

Evidence: the card names an exact term, an artifact to inspect, and a proof
sentence the reader can verify.

## Further Reading and Sources

- [Appendix A: Credible Resources and Further Reading](./31-credible-resources-further-reading.md) contains the complete list of academic papers and industry standards used to build the reliability model in this chapter.
