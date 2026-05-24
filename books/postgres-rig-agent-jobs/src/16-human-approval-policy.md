# 16. Human Approval And Policy Gates

## What You Will Learn

This chapter teaches you to:

- explain why approval is system state, not a button afterthought;
- inspect the policy decision, approval request, reviewer action, audit event, and execution boundary;
- verify that risky tool calls cannot run only because the model suggested them.

The production evidence is an approval gate where the model proposes, policy
classifies risk, a human or rule decides, and workers execute only approved
state.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** operators can observe what the system is doing.
- **Adds:** durable approval as a control surface outside the model.
- **Prepares:** testing strategies that prove each reliability ring.

## Production Failure

The model proposes a CRM update that changes a customer's billing status.

The tool executes because the JSON field says `"approved": true`.

**What breaks:** model output is confused with business authority. This is not
only a bad tool call. It is a broken authority model.

The model produced text. The system treated that text as permission. A billing
status changed because an untrusted component used the word "approved" in the
right shape.

**False fix:** make the prompt stricter: "Only approve billing changes when you
are sure." That may reduce some bad outputs, but it does not change the system
boundary. The model can still be confused by stale memory, prompt injection,
missing context, or a malformed retrieved document.

**Design response:** store the proposed action. Run deterministic policy. If
policy requires human review, create a durable approval request. Only after the
correct actor approves that request may the side-effect worker execute the CRM
update.

## Motivation

In production, confident model text is not permission. Some actions affect customers, money, infrastructure, privacy, or compliance, and those actions need deterministic control.

Without durable approval and policy state, human-in-the-loop becomes a conversation instead of a safety boundary. This chapter separates model proposals from authorization, sandboxing, approval, and execution evidence.

## Plain Version

Read this as the simple version:

**Simple rule:** Human approval is a control surface for risky action. It is not
a decoration after the model answers.

**Why it matters:** Some tool calls should pause. A person with the right
authority must approve, reject, or ask for more evidence. The important part is
not that a human clicked a button. The important part is that the system has
durable evidence that the right decision happened before the side effect.

**What to watch:** When you inspect an approval system, watch five things: the
approval state, the policy reason, the approver identity, the decision evidence,
and the execution path that stays blocked until approval exists.

## What You Already Know

Start with these anchors:

- Observability can explain what happened.
- It does not decide whether a risky action is allowed.
- Model recommendation is not permission.

This chapter adds: approval as state. Risky tool calls move through policy and
review before execution, so a human or rule can approve, reject, or require more
evidence.

## Focus Cue

Keep three things in view:

- **State:** a risky proposal, deterministic policy decision, approval or rejection record, actor, reason, and side-effect receipt.
- **Move:** model output becomes executable work only after policy and approval state authorize it.
- **Proof:** Proposal, policy version, actor, reason, decision time, and receipt exist before side-effect execution.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

**Artifact:** A serious implementation has a durable approval request and a
durable decision record before risky tool execution.

**Why it matters:** "Human in the loop" is often implemented as a chat
interruption. The agent asks, a person replies "yes", and the worker continues.
That is weak. The reply may not name the actor, tenant, policy version, exact
action, or reason. It may not survive process death. It may not be queryable
during an incident.

**Done when:** The artifact is complete when the model can propose a risky
action, but policy and approval state decide whether execution is legal.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `src/approval.rs`, `src/security.rs`, `src/tool_execution_gate.rs`, and approval SQL.
- **State transition:** turn risky model intent into requested, approved, rejected, or escalated state.
- **Evidence path:** execution requires policy evidence and human evidence where risk demands it.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Which human or policy decision made this risky action legal?
- **Evidence to inspect:** approval request, policy evaluation, decider, decision time, reason, and tool execution gate event.
- **Escalate if:** the model can proceed from suggestion to side effect without durable approval evidence.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** The pass begins when the model proposes a risky action. For
example, a support agent may propose refunding a customer, a sales agent may
propose changing CRM ownership, or an incident agent may propose rolling back
production.

**Action:** The system does not execute the proposal immediately. It converts
the proposal into a policy decision and, when needed, an approval request. This
is the moment where fuzzy model output becomes controlled system state.

**Persistence:** The system then persists the request, the policy version, the
eventual decider, the decision time, the reason, and the execution-gate event.

**Check:** The worker checks that state before it performs the side effect. If
the required approval is missing, rejected, expired, or from the wrong actor,
the worker stops.


## Acceptance Gate

Do not move on until this minimum evidence exists:

**Minimum evidence:** Risky execution requires policy and approval evidence.

**Validation path:** You should be able to inspect approval rows, policy
decisions, execution-gate tests, and audit events. Together they should prove
that a model suggestion cannot become a side effect without a durable policy
decision and, where required, a durable human decision.

**Stop if:** the model can move directly from suggestion to side effect. That
path is the bug this chapter is designed to remove.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, confident model text is not permission
rule: Human approval is a control surface for risky action, not a decoration after the model answers
tiny example: a risky proposal, deterministic policy decision, approval or rejection record, actor, reason, and side-effect receipt
artifact: a durable approval request and decision record before risky tool execution
proof: risky execution requires policy and approval evidence
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Approval is a boundary between recommendation and authority:

```text
model output:
  "rollback production"

policy decision:
  risky action, approval required

operator decision:
  approve, reject, or request more evidence

side-effect worker:
  execute only an approved action
```

The model may help choose. It should not grant itself permission.

This split is especially important because model output is persuasive text, not
authority. A good answer can still be wrong, stale, injected, or outside the
tenant's policy. Approval gives the system a place to stop before the world is
changed.

## The Core Split

Separate reasoning from action:

```text
agent worker:
  proposes a next action

policy gate:
  decides whether approval is required

operator:
  approves or rejects risky action

side-effect worker:
  executes approved action idempotently
```

The companion result type already encodes the first gate:

```text
AgentResult {
  summary,
  next_action,
  approval
}
```

## Tiny Example

An incident agent sees elevated payment errors and recommends rollback.

A weak system lets the model call the deployment tool directly. A safe system
stores:

```text
proposal: rollback payment service
risk: production side effect
approval: required
policy_version: approval-policy:v1
operator_decision: pending
```

The worker stops there. A separate side-effect worker acts only after approval
and records a receipt.

Notice the state machine: `proposal_created`, `approval_required`,
`approved` or `rejected`, then a separate execution transition. That structure
lets replay preserve the decision instead of asking the model to decide again.

Read the tiny case as:

```text
setup: the model recommends a production rollback
transition: policy classifies the action and approval state gates execution
evidence: proposal, policy decision, reviewer action, audit event, and receipt exist
invariant: model output can propose risk, but authority must be recorded separately
```

## Typestate Approval Lifecycle

The companion crate models approval as a typed lifecycle. A requested approval
can be approved, rejected, expired, or cancelled. Terminal approval values do
not expose those transition methods.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/approval.rs:approval_typestate}}
```

Database rows stay at the boundary. The row type can contain storage-friendly
strings and nullable fields; the application converts them into an
`ApprovalRecord` only after validating status, actor, reason, and decision
evidence.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/approval.rs:approval_row_boundary}}
```

The useful production invariant is:

```text
only requested approvals can become approved or rejected
terminal approval records are evidence, not mutable authority
```

## Escalation Is Not Approval

Approval and escalation are often confused.

```text
approval answers: may this risky action proceed?
escalation answers: which human owner must take responsibility now?
```

An approval can be pending while the system is still healthy. An escalation
means the system has reached a boundary where autonomous progress is no longer
safe: a deadline breached, failures repeated, an abuse signal appeared, an
approval timed out, or a worker found incompatible durable work.

Treat escalation as durable state, not a Slack message:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/escalation.rs:human_escalation_record}}
```

`HumanEscalationRecord` is the domain object the worker and operator tooling can
trust after the raw database row has been validated.

Database rows stay at the boundary and are validated before becoming domain
records:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/escalation.rs:human_escalation_row_boundary}}
```

The production invariant is:

```text
every escalation names a target, kind, severity, reason, status, owner when
acknowledged, and a resolution timestamp when closed
```

This gives operators a control surface for cases where the agent should stop
being clever and hand responsibility to a person.

## Why This Is Not Just UX

Approval is part of the reliability model.

The user interface may be a review screen, queue, command-line prompt, or
internal dashboard. That surface matters, but it is not the core system. The
core system is the durable state transition that prevents unsafe work from
executing.

Approval protects against model hallucination, prompt injection, wrong
retrieved context, stale tool output, tenant policy mismatch, and operator
intent mismatch. Each of those failures can produce an action that looks
reasonable in text. The approval boundary forces the action to become a typed,
reviewable, policy-checked object before it can change the world.

This is why approval should be implemented near tool execution, not only near
the chat interface. The chat interface can ask for review. The execution gate
must enforce it.

## Policy As Code

Do not hide policy inside prompts. Prompts can suggest behavior, but production
policy belongs in deterministic code and data:

```text
job kind
tenant policy
requested action type
risk level
approval requirement
operator decision
```

The model may classify or recommend. The system decides.

Version the policy. When an incident is reviewed later, "the policy required
approval" is not enough. The team needs to know which policy version made the
decision and whether that version was correct for the tenant, job kind, and
risk level.

This also protects product evolution. A summarizer may need no approval, a
customer-visible reply may need review, and a billing adjustment may require
two-person approval. These are different policy states, not different prompt
styles.

Do not make approval depend on the model remembering a rule. Put the rule in a
versioned policy boundary, store the decision, and make execution query that
decision before any irreversible action.

## Formal Definition

For this chapter, the precise definition is:

```text
Approval is durable decision state that authorizes or rejects a proposed risky action after deterministic policy has classified the risk.
```

In the book's system model:

- **State:** a risky proposal, deterministic policy decision, approval or rejection record, actor, reason, and side-effect receipt.
- **Actor:** policy gates classify risk, and authorized humans approve or reject before workers execute risky actions.
- **Transition:** model output becomes executable work only after policy and approval state authorize it.
- **Evidence:** Proposal, policy version, actor, reason, decision time, and receipt exist before side-effect execution.
- **Invariant:** a model can propose risky action, but it cannot grant itself permission to act.

## What Can Fail

**Design smell:** approval lives in chat, tickets, or model text. A reviewer may
have said "looks good", but the worker cannot prove which action was approved,
under which policy, for which tenant, and at what time.

**Production symptom:** an unsafe autonomous path has no owner. The agent
reaches a risky boundary, cannot continue safely, and still keeps retrying or
choosing new actions. That is not autonomy. That is missing escalation state.

The third failure is approval that is too broad. A human approves "update the
customer", but the tool call changes billing status, owner, and risk score. The
approval record must identify the exact action or action class it authorizes.

**Corrective invariant:** approval and escalation are state outside the model.

**Evidence to inspect:** proposal, policy version, actor, reason, timestamp,
side-effect receipt, and escalation owner.


## Production Contract

Approval is reliable only when:

The proposal must be stored before the side effect. If the action is not
recorded first, the later approval has nothing precise to authorize.

Policy must be deterministic and versioned. A prompt instruction is not enough,
because it cannot reliably explain why one tenant required review while another
tenant allowed autonomous execution.

Approval or rejection must be recorded with an actor and reason. A boolean is
too weak. The system needs to know who decided, what they decided, why they
decided it, and when the decision happened.

Side-effect execution must check approval state. It is not enough for the API
handler to check approval when the worker is the component that sends the email,
updates the CRM, rolls back production, or triggers a payment.

Replay must not bypass policy. If a worker restarts, the durable approval state
must still control execution.

Unsafe autonomous progress must create durable escalation. When the system
cannot continue safely, responsibility should move to a named human or team.

This is why approval belongs in the state machine, not in a prompt instruction.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Approval lives in chat, tickets, or model text. | Approval in chat or model text disappears from the state machine that executes the risky action. |
| Safer version | Approval and escalation are state outside the model. | Policy classification, approval request, human decision, and escalation ownership are durable records. |
| Production version | Proposal, policy version, actor, reason, timestamp, side-effect receipt, and escalation owner are recorded. | The side effect waits for named human evidence, policy version, reason, and decision timestamp. |

Use the naive row when a human says yes outside the system. Use the safer row to make approval stateful. Use the production row before risky tools can execute.

## Testing Strategy

Test approval as durable decision state:

**Unit test:** Start with the type model. Prove that Rust approval requests can
move only through legal states: requested, approved, rejected, expired, or
cancelled. The approved and rejected transitions should require actor and reason
evidence.

**Persistence test:** Then test the persistence boundary. A Postgres approval
row may use storage types, but the domain record should not exist unless status,
actor, reason, and decision evidence are valid. Do the same for escalation rows
before owner transfer proceeds.

**Regression test:** Finally, add the regression test that catches the
production failure. Attempt a risky action where model text says it is approved,
but no approval row exists. Execution must remain blocked. This test teaches the
central rule: model text is not authority.

## Observability Strategy

Observe human control as durable state:

Emit structured `tracing` fields for approval id, escalation id, actor id, job
id, run id, policy version, decision, reason, and trace id. These fields connect
the human decision to the agent run and the side-effect attempt.

Record an operation event when approval is requested, approved, rejected,
expired, cancelled, escalated, acknowledged, or resolved. Those events are not
debug noise. They are the operational history of control.

The runbook query should answer a practical question at 2 a.m.: which risky
action is waiting, who owns it, which policy required it, and whether execution
is still blocked?

## Security and Safety Considerations

Human approval is a safety boundary only when it is durable and scoped:

Treat chat messages, model claims, ticket text, and human notes as untrusted
until they become a typed approval or escalation record. A human may have typed
the sentence, but the system still needs to bind it to the exact action.

authorization, sandboxing, and approval must agree on actor, tenant, tool,
permission, reason, and decision before risky execution. If one boundary says
"allowed" and another says "unknown", the safe result is blocked execution.

Redact reviewer comments and sensitive payload context where needed, but
preserve approver identity, policy version, decision time, and approval
evidence. Privacy controls should not erase the proof that a risky action was
authorized.

## Operational Checklist

Use this checklist before relying on human approval and policy gates in production:

- **State:** Risky work moves through proposed, policy-checked, waiting-for-human,
  approved or rejected, executed, and receipt states.
- **Boundary:** Human notes, chat text, and model claims become typed approval or
  escalation records before workers trust them.
- **Failure:** A rejected, expired, or missing approval blocks execution and leaves
  evidence instead of relying on conversation history.
- **Observability:** Approval id, policy version, approver, decision time, reason, trace
  id, and waiting jobs are queryable.
- **Safety:** Authorization, sandboxing, approval, redaction, and side-effect receipts
  must agree before risky execution.

## Exercises

1. Write a negative test where a model proposes a destructive tool and the worker
   refuses execution until approval and idempotency receipt policy exist. Explain which
   idempotency key, receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: human_approval_requests, policy decisions, escalation
   rows, and blocked tool_call rows.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   `ApprovalRequest<Waiting>`, `ApprovalDecision`, `PolicyDecision`, and
   `HumanEscalationRecord` types. Then name the runbook question that proves it
   works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Why is approval state rather than a UI detail?
- Explain: Why does model recommendation not grant permission?
- Apply: Choose rollback or data deletion and trace proposal through policy to execution.
- Evidence: Name the policy decision, approval request, reviewer evidence, audit event, and side-effect receipt.

## Summary

The safe production pattern is proposal-first.

Agents produce auditable recommendations. Deterministic policy and humans
control risky side effects. **Invariant:** A risky action cannot execute until
policy, authorization, sandboxing, approval, idempotency, and receipt
requirements agree.

**Evidence:** approval requests, decisions, escalation records, policy versions,
tool-call state, trace ids, and side-effect receipts. Those records prove the
gate.

Carry this forward: human-in-the-loop is durable state, not a chat message.

## Changed Understanding

- **Before this chapter:** human review looked like a UI step after the agent decides.
- **After this chapter:** human approval is a control surface: policy decides what may execute, and approval creates durable evidence.
- **Keep:** inspect the approval request, policy version, reviewer identity, decision, and resulting audit event.

## Further Reading & Credible References

- **[Amodei et al.: Concrete Problems in AI Safety](https://arxiv.org/abs/1606.06565)** (2016). The foundational academic paper exploring "Scalable Oversight" and the role of human-in-the-loop systems in preventing agents from diverging from human intent.
- **[NIST AI Risk Management Framework (AI RMF 1.0)](https://www.nist.gov/itl/ai-rmf)**. The definitive industry standard for building trustworthy AI systems. It treats human oversight (HITL/HOTL) as a fundamental component of a socio-technical safety system.
- **[The Four-Eyes Principle (2-Person Rule) in Software Authorization](https://en.wikipedia.org/wiki/Two-man_rule)**. A core security protocol that mandates critical decisions be vets by at least two independent actors—the model (proposer) and the human (approver) in this chapter.
- **[OWASP Top 10 for LLM: LLM02 (Insecure Output Handling)](https://genai.ovasp.org/llm-02-insecure-output-handling/)**. Explains the specific security risk of trusting model output as permission, which the "Approval Gate" pattern in this chapter directly remediates.
- **[Designing Data-Intensive Applications](https://dataintensive.net/)** (Martin Kleppmann). Connects human-in-the-loop state to the "Auditability" and "Non-repudiation" requirements of high-stakes automated systems.
