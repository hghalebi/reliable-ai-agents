# 28.6 Tenant Isolation And Multi-Tenant Agents

## What You Will Learn

This chapter teaches you to:

- explain why tenant isolation must be a durable production control, not a prompt instruction;
- distinguish actor tenant, requested tenant, tool permission, policy version, and authorization decision;
- inspect cross-tenant attempts, denied requests, approval pressure, and boundary breaches from Postgres;
- verify that cross-tenant requests fail closed before tool execution;
- decode tenant-boundary review rows into typed Rust values before operator code trusts them;
- design tests, runbooks, and alerts that prove one tenant's data cannot quietly influence another tenant's run.

The production evidence is a tenant-boundary review. It connects
authorization events, actor tenant, requested tenant, tool permission, policy
version, decision reason, trace id, and operator runbook evidence.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** security boundaries already keep authority outside the model.
- **Adds:** tenant isolation as an explicit review surface with typed evidence.
- **Prepares:** disaster recovery and scaling, where restored data and new infrastructure must preserve tenant boundaries.

## Production Failure

A support triage agent retrieves a memory record from the wrong customer.

The model did not "hack" the system. The system passed the requested tenant as
a loose string. A tool request asked for `tenant-beta`, while the actor belonged
to `tenant-alpha`. The prompt said to stay in scope, but no deterministic
authorization check owned the tenant boundary.

- **What breaks:** one tenant's context can leak into another tenant's workflow.
- **False fix:** add a stronger system prompt that says "never cross tenants."
- **Design response:** model actor tenant and requested tenant separately, deny cross-tenant requests outside the model, persist authorization evidence, and review tenant-boundary status from Postgres.

## Motivation

In production, agents often serve many users, teams, customers, or companies.
Those groups may share the same worker pool, model provider, tool registry, and
Postgres database. They must not share authority by accident.

Tenant isolation answers a plain question:

```text
Which tenant is this actor allowed to act for?
```

Without tenant isolation, a reliable worker can still do the wrong work
reliably. It can retry the wrong tenant's tool call, cache the wrong tenant's
memory, emit the wrong tenant's audit event, or summarize the wrong tenant's
documents.

This is why tenant isolation belongs in the control system, not in the model.
The model may propose an action. The system must decide whether the actor,
tenant, tool, policy, and requested data scope are allowed to meet.

## Plain Version

Read this as the simple version:

**Simple rule:** The actor's tenant and the requested tenant are different
facts. The actor tenant says who is asking. The requested tenant says whose data
or action is being requested. They may look like the same kind of value in
storage, but they do different jobs in the system.

**Why it matters:** If those facts are confused, the agent can read, write,
remember, or send data for the wrong customer. The model may only be trying to
complete the task, but the control system has failed if it lets tenant scope come
from model text alone.

**What to watch:** Watch cross-tenant attempts, denied requests,
approval-required actions, policy version, and any cross-tenant request that was
not denied. The strongest signal is not "the prompt says stay in scope." The
strongest signal is a durable authorization decision made before the tool runs.

## What You Already Know

Start with these anchors:

- A tool call is a side effect.
- A side effect needs permission.
- Permission must be decided outside the model.
- Audit and operation events prove what happened later.
- Memory, tool calls, and provider usage can all carry tenant-sensitive data.

This chapter adds: tenant scope is not an informal label. It is a production
boundary that must be checked, stored, queried, tested, and operated.

## Focus Cue

Keep three things in view:

- **State:** actor tenant, requested tenant, tool permission, decision, policy version, reason, and decision time.
- **Move:** a proposed action becomes authorized, approval-required, or denied.
- **Proof:** `authorization_events`, `tenant_boundary_review.sql`, typed row conversion, audit events, and operation events explain the boundary.

If you get lost, ask one question: did any cross-tenant request become anything
other than denied?

Return to state, move, and proof: name the tenant facts, name the decision,
then name the evidence row.

## Production Artifact

Build or inspect this artifact before moving on:

- **Artifact:** tenant-boundary review query and typed review row.
- **Why it matters:** operators must see tenant isolation failures before they become data exposure, memory poisoning, or customer-visible incidents.
- **Done when:** cross-tenant attempts, allowed cross-tenant decisions, denied attempts, approval-required actions, and latest decision time are queryable and decoded into domain types.

## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** `authorization_events`, `tenant_boundary_review.sql`, `src/tenant_isolation.rs`, audit events, and operation events.
- **State transition:** model proposal becomes an authorization decision before any tool reads or writes tenant data.
- **Evidence path:** actor tenant, requested tenant, permission, policy version, decision, reason, trace id, and review status travel together.

## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Did any actor attempt to access a different tenant, and was every cross-tenant attempt denied?
- **Evidence to inspect:** `tenant_boundary_review.sql`, `authorization_events`, denied authorization events, policy version, trace id, operation events, and audit events.
- **Escalate if:** a cross-tenant request is authorized, requires approval, lacks a reason, has no trace id, or cannot be tied to an agent run.

## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** The model proposes a tool call that needs tenant data. The proposed
call may come from structured output, a tool plan, a retry, or a resumed run. At
this point, the model has only proposed an action. It has not earned authority.

**Action:** The system builds a typed authorization request with two tenant
facts: the actor tenant and the requested tenant. This step is where raw input
stops being trusted text and becomes data that the policy layer can inspect.

**Decision:** Deterministic policy denies cross-tenant requests before tool
execution. It may require approval for risky same-tenant actions, but
cross-tenant access is a different class of risk. It should not be quietly
converted into a human approval task unless the product has an explicit
delegation model with its own grant, expiry, owner, and audit evidence.

**Persistence:** The authorization event records tenant keys, permission,
decision, reason, policy version, and time. This turns the access decision into
evidence that operators can inspect later.

**Check:** `tenant_boundary_review.sql` groups recent decisions and surfaces
boundary breaches. It answers the operator question that matters most: did any
cross-tenant request avoid denial?

## Acceptance Gate

Do not move on until this minimum evidence exists:

- **Minimum evidence:** one same-tenant authorization and one denied cross-tenant request can be found in durable records.
- **Validation path:** run the tenant isolation Rust tests, inspect the review query, and verify that a cross-tenant request cannot become trusted tool execution.
- **Stop if:** tenant scope is stored only in prompts, tool arguments, frontend state, or private operator notes.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: one tenant's data reaches another tenant's agent run
rule: tenant scope is authorization state, not prompt text
tiny example: tenant-alpha actor asks for tenant-beta memory
artifact: authorization_events plus tenant_boundary_review.sql
proof: every cross-tenant request is denied or escalated as an incident
```

If the next section feels large, keep only these five lines in view.

## Tiny Example

setup: an operator from `tenant-alpha` asks an agent to summarize a case file.
The tool request names `tenant-beta` as the requested tenant.

The naive system treats the tenant as a string inside tool input:

```text
tool: read_case_file
tenant: tenant-beta
case: case-123
```

transition: the production system turns that request into an authorization
decision before the tool can read the case file.

evidence: the authorization row records actor tenant, requested tenant, tool
permission, decision, reason, policy version, and decision time.

invariant: a cross-tenant request is not a risky same-tenant action. It is a
boundary violation unless an explicit product design supports scoped
delegation with separate evidence.

The important point is small: the model can name a tenant, but only the system
can grant tenant authority.

## Mental Model

Think of tenant isolation as a locked door between evidence surfaces.

```text
actor tenant -> authorization policy -> requested tenant
```

The model may knock on the door by proposing a tool call. The authorization
policy decides whether the door opens, requires human approval, or stays
closed.

Do not let tenant identity live only in:

```text
prompt text
tool JSON
frontend state
operator memory
provider metadata
```

Those are boundary inputs. Inside the system, tenant identity is a typed value
that participates in policy, persistence, tracing, and review.

## The Core Problem

Reliable agents mix many data flows:

```text
user request
agent memory
tool input
tool output
audit event
provider usage
evaluation fixture
runbook query
```

In a single-tenant demo, this can look simple. In a real product, the same
worker, model, and database may handle many tenants. A single missing check can
turn a good retry, memory lookup, or tool call into a data leak.

The production question is not:

```text
Did the model intend to access the right tenant?
```

The better question is:

```text
Which deterministic record proves this actor was allowed to access this tenant?
```

## The Naive Solution

The naive implementation passes tenant identity as ordinary text:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/security.rs:typed_authorization}}
```

This excerpt is the production version, but imagine deleting the policy check
and letting the model-proposed tenant flow directly into a tool call. The
signature would still compile. The tool might still work. The incident would
appear only when someone notices the wrong customer's data in the result.

The problem is not that strings exist. The problem is letting raw tenant text
carry authority.

## The Production-Grade Concept

Treat tenant isolation as a typed authorization invariant:

```text
actor tenant + requested tenant + permission + policy version -> decision
```

The actor tenant names who is asking. The requested tenant names whose data,
memory, tool, document, side effect, or evidence is being requested. In storage,
both may be text. In the domain model, they are separate facts that must meet at
an authorization boundary.

The permission names the kind of action requested. Reading tenant data, updating
a CRM record, sending a tenant-visible message, and retrieving memory are not
the same risk. The decision records whether the action is authorized, requires
approval, or is denied.

The reason explains non-authorized decisions. A denial without a reason is hard
to debug and hard to review. A policy version lets reviewers explain later why
the system made that decision under the rules active at the time.

The central invariant is:

```text
A tenant boundary can be crossed only by deterministic policy, never by model text.
```

## Postgres Review Query

The companion query turns recent authorization evidence into an operator view:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/tenant_boundary_review.sql}}
```

Read the query as a safety question:

```text
Did any cross-tenant request avoid denial?
```

The result is not an access-control system by itself. It is the review surface
that tells operators whether the access-control system is producing safe
evidence.

## Typed Rust Boundary

The query output is still raw database data until Rust decodes it.

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/tenant_isolation.rs:tenant_isolation_row_boundary}}
```

The row boundary rejects unknown review statuses and negative counts. It also
rejects cross-tenant allowed and denied counts that do not add up to attempts,
missing latest decision time, and review statuses that contradict the evidence
counts.

This keeps the dashboard from treating inconsistent review data as truth.

## Formal Definition

For this chapter, the precise definition is:

```text
Tenant isolation is the durable control that prevents one tenant's data,
memory, tools, side effects, and evidence from being read or changed by an
actor that lacks tenant-scoped authorization.
```

In the book's system model:

**State:** The state is the actor tenant, requested tenant, permission, policy
version, decision, reason, and decision time.

**Actor:** The actor may be a user, operator, worker, agent run, reviewer, or
system process. The actor matters because an agent action can run long after the
original user request, during a retry, resume, or background workflow.

**Transition:** The transition is the controlled move from proposed tool action
to authorized, approval-required, or denied action. The tool should not execute
until this transition has happened.

**Evidence:** The evidence is the authorization event, tenant-boundary review
row, trace id, operation event, audit event, and policy version.

**Invariant:** Model output never grants tenant authority. The model can name a
tenant. The system decides whether that tenant may be accessed.

## What Can Fail

**Design smell:** Tenant id is passed through prompts, tool JSON, provider
metadata, frontend state, or operator notes as an ordinary string with no
authorization evidence.

**Production symptom:** A memory lookup, document read, CRM update, provider
call, or external message uses the wrong tenant scope. The system may still be
fast and retriable, but it is now reliably doing work for the wrong boundary.

**Corrective invariant:** Actor tenant and requested tenant are checked by
deterministic policy before tool execution.

**Evidence to inspect:** Inspect `authorization_events`,
`tenant_boundary_review.sql`, typed row-conversion tests, denied authorization
events, trace ids, operation events, and audit events.

## Production Contract

The contract is:

```text
proposed tool request -> typed tenant request -> deterministic authorization -> durable decision -> reviewed boundary evidence
```

Do not let a tenant boundary depend on:

```text
prompt wording
model self-report
frontend-only checks
provider metadata alone
manual operator memory
```

The system should preserve:

```text
actor tenant
requested tenant
tool permission
decision reason
policy version
trace id
audit evidence
operator review query
```

## Progressive Hardening Path

**Naive version:** Tenant id is a string inside the prompt or tool JSON. The
model can accidentally or maliciously request another tenant, and the tool may
still run because no deterministic boundary owns the decision.

**Safer version:** Actor tenant and requested tenant are separate typed values
checked by authorization policy. Cross-tenant requests are denied before tool
execution.

**Production version:** Authorization events, tenant-boundary review SQL, typed
row conversion, audit events, operation events, and incident drills prove the
boundary. Operators can detect boundary breaches and prove every cross-tenant
attempt was denied.

## Testing Strategy

Unit and Rust type tests should prove cross-tenant requests are denied. They
should also prove that unknown review statuses, negative counts, inconsistent
counts, and missing decision times fail row conversion.

Persistence or Postgres boundary tests should prove
`tenant_boundary_review.sql` surfaces cross-tenant attempts, allowed
cross-tenant decisions, approval-required actions, and latest decision time.

Regression tests should replay a model output that requests another tenant's
memory. Tool execution must stop before the read side effect. This test protects
the exact failure that a stronger prompt cannot prevent.

An operational drill should create one denied cross-tenant authorization event in
staging, run the review query, and record the incident or expected-denial
evidence.

Tests should prove that tenant isolation fails closed. A missing tenant,
unknown tenant, or cross-tenant mismatch should stop the action, not pick a
default tenant.

## Observability Strategy

Emit structured `tracing` fields for actor id, actor tenant, requested tenant,
tool name, permission, policy version, decision, decision reason, and trace id.

Record an operation event when:

```text
tenant_authorization_denied
tenant_authorization_requires_approval
tenant_boundary_breach_detected
tenant_boundary_review_completed
```

The first runbook query is `tenant_boundary_review.sql`. It should answer:
which cross-tenant attempts happened, which were denied, and whether any
cross-tenant request was not denied.

## Security and Safety Considerations

Treat tenant values from HTTP, model output, provider metadata, database rows,
and tool arguments as untrusted until authorization checks them.

Use sandboxing to prevent a tool from turning a tenant mismatch into network,
filesystem, or secret access. A denied tenant decision should stop before the
tool can choose egress, file paths, or credentials.

Require approval only for risky same-tenant actions. A cross-tenant request is
not made safe merely by asking a random reviewer. If cross-tenant delegation is
a real product feature, model it separately with explicit grant, expiry, owner,
and audit evidence.

Redact tenant-sensitive payloads from logs and traces. Keep stable ids,
decision reason, policy version, and trace id so incidents remain
investigable.

## Operational Checklist

Use this checklist as a design review for every multi-tenant agent tool.

First, inspect **State**. Which actor tenant, requested tenant, permission,
decision, reason, and policy version are attached to this tool request?

Next, inspect the **Boundary**. Where do raw tenant strings become typed tenant
keys, and where does authorization happen?

Then inspect **Failure**. What happens if the requested tenant differs from the
actor tenant, is missing, is unknown, or is stale?

Inspect **Observability** next. One trace should connect the proposal,
authorization event, operation event, audit event, and runbook query row.

Finally, inspect **Safety**. The tool must not read memory, documents, secrets,
filesystem paths, network destinations, or side-effect targets before tenant
authorization passes.

## Exercises

1. Design an idempotency key for a same-tenant CRM update. Explain why the tenant key must be part of the key.
2. Sketch the Postgres evidence for a denied cross-tenant memory read using `authorization_events` and `tenant_boundary_review.sql`.
3. Write a negative test plan for the Rust row boundary where `cross_tenant_allowed + denied_cross_tenant_attempts` does not equal `cross_tenant_attempts`.
4. Extend the security drill with one model output that asks for another tenant's case file, then name the runbook query that should expose the denial.

## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Why are actor tenant and requested tenant separate facts?
- Explain: Why a prompt instruction cannot enforce tenant isolation.
- Apply: Trace one cross-tenant tool request from model output to denied authorization event.
- Evidence: Name the typed request, authorization event, tenant-boundary review query, operation event, audit event, and trace id.

## Summary

Tenant isolation is part of reliable agent operations. If an agent can use
tools, memory, and durable state for more than one tenant, the tenant boundary
must be explicit.

Invariant: model output never grants tenant authority.

Evidence: authorization events, `tenant_boundary_review.sql`, typed
row-conversion tests, operation events, audit events, policy versions, and trace
ids prove the boundary.

Carry forward: never let tenant identity be only a raw string in a prompt, tool
argument, or frontend field.

## Changed Understanding

**Before this chapter:** tenant scope looked like an application setting.

**After this chapter:** tenant scope is an authorization boundary that must be
typed, persisted, tested, queried, and audited.

**Keep:** one tenant's data can enter another tenant's workflow only through
explicit, durable, reviewable policy.

## Further Reading and Sources



- [AWS: SaaS Tenant Isolation Strategies](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: The definitive industry guide to the "Silo," "Pool," and "Bridge" isolation models. It explains why the "Pool" model used in this chapter requires the rigorous, deterministic authorization checks implemented here.
- [PostgreSQL row-level security documentation](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: The practical reference for the database-level hardening mentioned in this chapter, which ensures that even a compromised application cannot cross tenant boundaries.
- [Designing Data-Intensive Applications](./31-credible-resources-further-reading.md#durable-execution-and-data-systems) Read this because: (Martin Kleppmann, Chapter 12: The Future of Data Systems). Connects multi-tenancy to the formal "System of Systems" view, where data ownership must be preserved across multiple coordination points (Postgres, workers, models).
- [Non-Interference](./31-credible-resources-further-reading.md#chapter-specific-resources) Read this because: **. The academic "Gold Standard" for isolation. It proves that a system is secure if "high-level" (Tenant A) inputs cannot affect "low-level" (Tenant B) observable behavior.