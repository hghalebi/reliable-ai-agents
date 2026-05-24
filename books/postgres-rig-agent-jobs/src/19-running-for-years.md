# 19. Running For Years

## What You Will Learn

This chapter teaches you to:

- explain what changes when an agent must run for years, not days;
- inspect versioning, retention, provider independence, migration safety, ownership, and recovery practice;
- verify that old work remains explainable after code, prompts, models, and teams change.

The production evidence is long-horizon metadata for versions, retention,
replay decisions, provider contracts, restore drills, and owners.

## Chapter Thread

Read this chapter as one link in the production chain:

- **Builds on:** deployments preserve state and ownership.
- **Adds:** versioning, retention, compatibility, and ownership for old work.
- **Prepares:** the final production blueprint that assembles the serious MVP.

## Production Failure

A customer asks why an agent made a decision nine months ago.

The row still exists, but the prompt changed, the model route moved, the owner
left the team, and the old provider response can no longer be interpreted.

**What breaks:** the system kept data but lost meaning.

This is one of the quietest production failures. The database row exists. The
logs exist. The incident timeline exists. But the system can no longer explain
what those records meant when the agent acted. The evidence survived, but the
interpretation did not.

**False fix:** keep every log forever and search harder during audits.

More data does not automatically create more understanding. If the logs do not
name the prompt version, model route, policy version, worker build, provider
contract, owner, and retention rule, then keeping them forever only preserves a
larger mystery.

Design response: store versions, retention policy, compatibility rules,
provider contracts, restore evidence, and ownership review dates. A system that
must run for years needs to preserve meaning, not only bytes.

I’ve seen too many AI projects fail because they couldn't explain a decision from six months ago. In AI, **Lineage is everything**. Your emphasis on storing the `prompt_version` and `model_route` alongside the result is the only way to build a **Verifiable AI**. This lineage is also what allows us to do **Backtesting**—re-running old inputs against new prompts to see if we’ve actually improved.

## Motivation

In production, long-running systems fail by drifting. Schemas evolve, providers change, prompts age, costs grow, staff rotate, and old jobs remain in the database.

Without versioning, retention, compatibility policy, and ownership, a system that worked for months can become impossible to explain. This chapter designs for years, not demos, incidents, or audits.

## Plain Version

Read this as the simple version:

**Simple rule:** A years-long agent system is designed for change, recovery,
ownership, and evidence from the beginning.

The question is not "Can the process stay alive?" A process can stay alive
while the system slowly becomes impossible to understand. The real question is
"Can future engineers explain and safely resume old work after the system has
changed?"

**Why it matters:** Most long-horizon failures come from forgotten assumptions,
stale data, lost context, and unpracticed recovery. These failures rarely look
dramatic at first. They accumulate.

**What to watch:** Watch retention rules, upgrade compatibility, restore
drills, ownership handoffs, credential lifecycle, provider changes, and
operational debt.

## What You Already Know

Start with these anchors:

- Deployment protects one release boundary.
- Long-running systems cross many releases, providers, operators, and business rules.
- Old work must remain explainable after the team has changed.

This chapter adds: long-horizon compatibility. You will preserve versions,
retention rules, provider independence, replay decisions, restore practice, and
ownership over time.

## Focus Cue

Keep three things in view:

- **State:** old jobs, events, versions, retention policies, provider routes, ownership records, and restore evidence.
- **Move:** work crosses time through compatibility checks, retention decisions, provider changes, and ownership reviews.
- **Proof:** Version fields, retention rules, compatibility checks, ownership records, and restore drills are maintained.

If you get lost, return to state, move, and proof. They are the short path from the idea to a production check.


## Production Artifact

Build or inspect this artifact before moving on:

**Artifact:** a long-horizon ledger for versions, retention, ownership,
provider changes, and review dates.

This ledger is the memory that lets the system survive time. It says which
version created the work, which policy governed it, which provider contract
handled it, which retention rule applies, which owner is responsible, and which
restore or review evidence proves the system is still maintainable.

**Why it matters:** systems that run for years fail through drift as often as
through crashes. Drift means the system still runs, but the team slowly loses
the ability to explain, recover, or safely change it.

**Done when:** old jobs remain interpretable after schema, model, prompt,
policy, provider, and owner changes.


## Implementation Map

Use this map when you move from reading to implementation:

- **Primary surface:** version fields, compatibility checks, retention rules, owner records, and release gates.
- **State transition:** keep old work understandable while prompts, models, schemas, providers, and owners change.
- **Evidence path:** historical rows still decode and route after long-horizon change.


## Operator Question

Before you ship this idea, answer one operational question:

- **Question:** Will this job still be understandable after versions, owners, and providers change?
- **Evidence to inspect:** prompt version, model version, schema version, policy version, worker version, retention rule, and owner record.
- **Escalate if:** old work cannot be decoded, routed, evaluated, or owned after normal long-term change.


## Runtime Walkthrough

Follow the concept as one runtime pass:

**Trigger:** time passes and versions, owners, providers, or policies change.

This is not an exceptional event. It is normal production life. Providers
release new models. Teams rotate. Policies change. Schema migrations land.
Customers ask about old decisions.

**Action:** preserve enough version and ownership metadata to interpret old
work. The job must carry the context future workers and operators will need.

**Persistence:** persist retention, compatibility, and review evidence. Do not
leave long-term meaning in a Slack thread, a release note, or one engineer's
memory.

**Check:** verify historical jobs still decode, route, and audit correctly. If
they do not, the system should migrate, quarantine, or assign ownership with
evidence.


## Acceptance Gate

Do not move on until this minimum evidence exists:

**Minimum evidence:** old work remains interpretable after normal long-term
change.

**Validation path:** inspect version fields, compatibility checks, retention
rules, and ownership evidence. Then test the claim with at least one historical
fixture or representative old row.

**Stop if:** historical jobs cannot be decoded, routed, evaluated, or audited
after provider or schema drift. That is not merely missing documentation. It is
a reliability failure.

## Micro-Lesson

Use this five-line version before the heavier mechanism:

```text
pain: In production, long-running systems fail by drifting
rule: A years-long agent system is designed for change, recovery, ownership, and evidence from the beginning
tiny example: old jobs, events, versions, retention policies, provider routes, ownership records, and restore evidence
artifact: a long-horizon ledger for versions, retention, ownership, provider changes, and review dates
proof: old work remains interpretable after normal long-term change
```

If the next section feels large, keep only these five lines in view. Then read
the mechanism as the detailed proof.

## Mental Model

Long-running systems are mostly compatibility systems. The work you enqueue
today may be retried, audited, or explained after several versions of the code,
prompt, model, policy, and schema have changed.

```text
today's job -> tomorrow's worker -> next month's incident review
```

The durable record must carry enough meaning to survive that path.

Think of every durable row as a message to a future operator. That operator may
not know the original prompt, the old provider quirks, the previous policy, or
the engineer who built the first worker. The row must carry enough context that
they do not need folklore to act safely.

## Long-Horizon Principles

Use these principles as design pressure, not slogans:

```text
durable state over memory
append-only events over overwritten logs
typed boundaries over raw JSON leakage
idempotency over hope
explicit policy over prompt-only safety
small migrations over big rewrites
runbooks over tribal memory
```

Each line turns a future failure into a present design requirement. Durable
state protects against process loss. Append-only events protect historical
truth. Typed boundaries protect old rows from being misread by new code.
Runbooks protect the system when the original builders are no longer available.

## Tiny Example

A job from six months ago produced a bad recommendation. Without version fields,
the incident review asks unanswerable questions:

```text
Which prompt was used?
Which model route was active?
Which policy rule allowed the action?
Which worker build parsed the payload?
```

With versioned rows, the same review becomes an engineering task instead of an
archaeology project.

Read the tiny case as:

```text
setup: an incident reviewer inspects a six-month-old job
transition: versioned metadata reconstructs the old behavior path
evidence: prompt, model, policy, worker build, events, and retention record remain available
invariant: time must not erase the ability to explain production decisions
```

The difference is not that the second system stored more data. The difference
is that it stored the right meaning. The reviewer can reconstruct the decision
path without guessing which current prompt, current model, or current policy
resembles the old one.

## Versioning

Store enough version information to understand old work:

```text
job kind
payload schema version
prompt version
model route
tool version
policy version
worker build or release id
```

Without this, a job from last year becomes impossible to interpret after the
codebase changes.

The companion schema implements this directly:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/001_agent_jobs.sql:version_columns}}
```

Defaults are only a bootstrap convenience. In production, the enqueue boundary
should set these values from release configuration so replay can choose the
right compatibility path.

Treat missing versions as a production smell. I frame this as **Ownership through Versioning**. If the system cannot name the model, prompt, policy, or worker that handled old work, then no one is responsible for that model's mistakes.

> ### 🎓 The Professor's Corner
>
> **The Time Capsule: Writing to Your Future Self**
>
> Building a system for "Years" is like burying a time capsule! You aren't building it for yourself; you're building it for the person who opens it in 2030. 
> 
> You have to include a **Decoder Ring** (the versioning metadata) so they can understand what you put in there. If you just leave a bag of strings, they'll have no idea what it means. It’s like writing a message to the future!

## Worker Compatibility

Version fields are only useful when a worker checks them before acting.

The naive version says:

```text
if the job is pending, process it
```

The production version says:

```text
if the job is pending and this worker understands its schema, process it;
otherwise quarantine or route it to a compatible worker
```

Your compatibility policy acts as a **Runtime Schema Validator**. This prevents **Silent Data Corruption** over long time horizons where an "Old Worker" might corrupt a "New Schema" (or vice versa).

That distinction matters after years of releases. A new worker may no longer
understand a payload written by an old API. An old worker may accidentally claim
a job created by a newer API. Both cases are compatibility bugs, not model
bugs.

The companion crate names this boundary explicitly:

```rust,ignore
{{#include ../../../examples/postgres-rig-agent-jobs/src/compatibility.rs:compatibility_policy}}
```

The important part is not the amount of code. The important part is the
decision shape:

```text
AgentJob + WorkerCompatibilityPolicy -> Process | Quarantine(reason)
```

There is no boolean named `ok`. A worker either has a typed reason to process
the job or a typed reason to keep it away from the execution path.

Operators also need a database view of the same risk:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/version_compatibility_risks.sql}}
```

This query answers a practical long-term question:

```text
Which pending or running jobs require a different worker version before they
can be handled safely?
```

This is the principle of old-work compatibility in concrete form. The ledger
does not merely store versions for incident review. It lets the worker and the
operator prevent incompatible execution before it happens.

## Data Retention

Decide retention intentionally. Retention is not just a storage cost topic:

```text
job rows: keep as long as replay/audit requires
event rows: keep or archive for incident history
payloads: minimize sensitive data
model outputs: redact where needed
metrics: aggregate for long-term trends
```

Do not store full prompts, secrets, personal data, or retrieved documents
unless the product and legal model require it.

Retention is also a reliability topic. If you delete the only receipt that
proves a side effect happened, replay becomes dangerous.

> ### 🎓 The Professor's Corner
>
> **Tombstones: Forgetting the Data, Remembering the Deletion**
>
> In distributed systems, deleting data is harder than writing it! If you just erase a row, an old worker might "find it again" in its memory and try to recreate it. 
> 
> We use **Tombstones**—a small marker that says: "This was deleted." It's like leaving a sign where the house used to be so people don't try to deliver mail there! It helps us "Forget the data while remembering the deletion."

If you keep raw
personal data forever, privacy and security risk grows. If you aggregate too
early, incident review may lose the evidence it needs. The right retention rule
depends on the evidence surface.

Retention also needs a review surface. A team cannot manage years of agent
history by guessing which tables are growing or which evidence is old enough to
archive.

The first storage-pressure query is read-only:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/storage_pressure_by_table.sql}}
```

It answers:

```text
Which production evidence tables are largest?
Which tables have high estimated dead-row pressure?
When did vacuum or analyze last touch the table?
```

The first retention-review query is also read-only:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/retention_review_by_surface.sql}}
```

It answers:

```text
Which evidence surfaces have rows older than 90 days?
Which evidence surfaces have rows older than 365 days?
Which surfaces need archive, aggregation, redaction, or policy review?
```

These queries do not delete data. They make the review visible. Deletion,
archival, aggregation, and redaction should be separate audited actions because
the database is the evidence backbone.

## Credential Lifecycle Review

Secrets age too. Provider API keys, database URLs, operator tokens, webhooks,
service-account keys, CI secrets, and encryption keys all need owners, rotation
rules, and exposure response.

The system should not store secret values in Postgres. It should store durable
credential lifecycle evidence:

```text
credential kind
secret reference
owner
storage location
status
last rotation time
next rotation due time
last verification time
exposure or revocation evidence
policy version
```

The first credential review query is read-only:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/credential_rotation_review.sql}}
```

It answers:

```text
Which credential kinds are active?
Which credentials are due or overdue for rotation?
Which credentials have open exposure incidents?
Which credentials have stale verification evidence?
Which credentials were revoked recently?
```

This query does not rotate a secret. It tells the operator where the secret
lifecycle is unsafe or stale. Rotation still happens through the external
runtime secret path. Postgres keeps the reference, owner, policy version, and
evidence that the rotation or revocation happened.

This split keeps the design honest. Secret values belong outside the ledger.
Secret lifecycle evidence belongs inside the ledger. The system needs to prove
that credentials are owned, rotated, verified, and revoked without making the
database a place where secrets leak.

## Data Protection Review

Retention asks "how long should this evidence live?" Data protection asks a
sharper question:

```text
Which user, tenant, or policy request requires redaction, erasure, export, or
retention review, and is the request overdue?
```

Do not handle that work in private messages or support notes. A privacy request
is operational state. It needs a row, due date, policy version, evidence object,
and terminal status.

The first data-protection ledger is `data_protection_requests`. It stores the
surface, subject reference, request kind, status, owner reason, policy version,
due date, completion time, and evidence. The subject reference should be an
internal reference or stable hash, not raw personal data.

The first review query is read-only:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/data_protection_review.sql}}
```

It answers:

```text
Which evidence surfaces have open privacy work?
Which surfaces have overdue requests?
Which surfaces have pending redaction or erasure work?
Which surfaces had applied requests in the last 30 days?
```

This query still does not delete data. It tells an operator where privacy work
is waiting. The actual redaction or erasure action must be a separate audited
operation that preserves the minimum evidence needed for security, legal,
replay, and incident review.

## Job Kind Retirement

Running for years also means removing old routes safely. A prompt route, model
route, tool version, or job kind should not disappear because it feels unused.
It should move through evidence:

```text
active -> deprecation candidate -> retirement candidate -> retired outside the hot path
```

The first lifecycle review query is read-only:

```sql
{{#include ../../../examples/postgres-rig-agent-jobs/sql/job_kind_lifecycle_review.sql}}
```

It answers:

```text
Which job kinds still have pending, running, retrying, or human-waiting work?
Which job kinds have recent provider calls?
Which job kinds are paused with no recent usage?
Which latest release decision is tied to the job kind?
```

Retirement is blocked while work is still open, retries are waiting, approvals
are pending, or provider calls are recent. This is not bureaucracy. It is how a
team avoids deleting the path needed to finish old work.

## Provider Independence

Rig helps keep model providers behind a boundary. Use that boundary:

```text
worker -> AgentRunner trait -> Rig provider runner
```

If DeepSeek changes, only the runner boundary should need adjustment. The
Postgres ledger, retry system, events, and policy gates should stay stable.

Provider independence does not mean providers are interchangeable with no work.
It means provider-specific behavior is contained. The durable system should not
depend on one provider's current response shape, error wording, rate-limit
format, or model route naming. Those details belong in an adapter. The ledger
stores the provider route and version evidence needed to explain what happened.

## Formal Definition

For this chapter, the precise definition is:

```text
Long-horizon operation is the ability to explain, parse, recover, and safely resume old work after code, schema, model, policy, and provider changes.
```

In the book's system model:

- **State:** old jobs, events, versions, retention policies, provider routes, ownership records, and restore evidence.
- **Actor:** maintainers, workers, and operators keep old work parseable, explainable, or explicitly quarantined.
- **Transition:** work crosses time through compatibility checks, retention decisions, provider changes, and ownership reviews.
- **Evidence:** Version fields, retention rules, compatibility checks, ownership records, and restore drills are maintained.
- **Invariant:** months-old work remains explainable and safely actionable after system evolution.

## What Can Fail

**Design smell:** old work is expected to remain understandable by memory. The
team assumes someone will remember how the system used to behave.

**Production symptom:** a six-month-old job cannot be replayed or explained
after prompt, model, policy, or code drift. The system has data, but no
trustworthy interpretation.

**Corrective invariant:** long-lived jobs retain enough version evidence to
explain old behavior and prevent incompatible execution. If a worker cannot
understand old work, it must quarantine or route it instead of guessing.

**Evidence to inspect:** job rows record schema, prompt, model, policy, worker,
and evaluation versions; compatibility checks quarantine jobs outside the
worker's supported schema range.


## Production Contract

A system is ready to run for years only if a few promises remain true.

Old jobs remain parseable or explicitly quarantined. Old events remain
meaningful after code changes. Prompt, model, tool, policy, and worker versions
are stored. workers check supported schema ranges before execution. Retention
rules protect privacy without destroying auditability. Credential rotation and
exposure work is visible without storing secret values. Provider changes stay
behind the `AgentRunner` boundary. Ownership survives team rotation.

These promises are less glamorous than model quality, but they are what let a
system keep serving real users for years.

## Progressive Hardening Path

| Stage | Implementation shape | What changes |
| --- | --- | --- |
| Naive version | Old work is expected to remain understandable by memory. | Old jobs are expected to remain understandable by team memory and current code assumptions. |
| Safer version | Long-lived jobs retain enough version evidence to explain old behavior and prevent incompatible execution. | Version fields and compatibility policy keep old work parseable, explainable, and quarantinable. |
| Production version | Job rows record schema, prompt, model, policy, worker, and evaluation versions; compatibility checks quarantine jobs outside the worker's supported schema range. | Years later, schema, prompt, model, policy, worker, and evaluation versions still tell the recovery story. |

Use the naive row when long-term operation depends on memory. Use the safer row to persist version evidence. Use the production row before a system claims it can run for years.

## Testing Strategy

Test long-horizon operation with old work, not only current code:

**Unit or type test:** prove Rust compatibility policies reject too-old and
too-new payload schemas while accepting supported historical versions. This
keeps compatibility as a typed decision, not an ad hoc branch.

**Persistence or boundary test:** prove Postgres rows retain schema, prompt,
model, tool, policy, worker, and evaluation versions needed to explain old
jobs. The row is the long-term evidence packet.

**Regression test:** decode a historical job fixture after changing code. The
system should process, migrate, or quarantine it explicitly.

**Runbook test:** prove storage-pressure and retention-review queries expose
table size, dead-row pressure, last vacuum/analyze evidence, old-row counts,
and the evidence surfaces that need retention review.

**Credential test:** prove credential lifecycle rows expose due rotation,
overdue rotation, open exposure incidents, and stale verification without
storing secret values.

## Observability Strategy

Observe old work with the version evidence needed to explain it:

Emit structured `tracing` fields for job id, payload schema version, prompt
version, model version, policy version, worker build, compatibility decision,
and trace id. These fields let old work remain explainable after the current
release is forgotten.

Record an operation event when historical work is processed, migrated,
quarantined, restored, replayed, or rejected for unsupported schema range. The
event should say what happened and why the system considered it safe.

The runbook query should answer why an old job behaved that way even after
code, models, prompts, and policies changed, and whether the underlying
evidence tables are growing or drifting beyond their retention policy.

Credential lifecycle queries should answer which secret references need
rotation, revocation, or verification without exposing the secret itself.

## Security and Safety Considerations

Long-lived systems must preserve safety meaning across versions:

Treat old payloads, old approvals, historical memory, and restored rows as
untrusted until compatibility policy validates their schema and authority
model. Age does not make data trustworthy.

authorization, sandboxing, and approval rules should be versioned so old work
cannot execute under a silently different policy. A retry from last year should
not accidentally inherit today's broader permission.

Redact historical sensitive data while preserving prompt, model, policy,
worker, schema, and evaluation version evidence. Privacy controls and
auditability must be designed together.

## Operational Checklist

Use this checklist before relying on long-horizon compatibility and maintenance in production:

- **State:** Jobs, prompts, models, policies, schemas, memory, receipts, and runbooks
  carry versions, retention rules, and credential lifecycle evidence.
- **Boundary:** Old rows and provider changes are handled through compatibility policy
  instead of ad hoc worker branches.
- **Failure:** Unsupported old work is migrated, quarantined, or handled by a compatible
  worker with evidence.
- **Observability:** Compatibility risks, retention decisions, credential rotation
  review, provider independence, owner review dates, storage pressure, and restore
  drills are queryable.
- **Safety:** Long-lived data keeps tenant, retention, redaction, approval, replay,
  and credential exposure constraints over time.

## Exercises

1. Write a negative test where a two-year-old pending job is picked by a new worker and
   must respect old idempotency and policy versions. Explain which idempotency key,
   receipt, or state transition prevents duplicate work.
2. Sketch the Postgres evidence: versioned job, prompt, model, policy, receipt,
   retention, and compatibility-risk rows.
3. Define or refine the Rust type, enum, constructor, or typestate that represents
   WorkerCompatibilityPolicy, PayloadSchemaVersion, RetentionPolicy, and
   VersionedRunContext types. Then name the runbook question that proves it works.
## Self-Check

Use this quick retrieval drill before moving on:

- Recall: Which version fields make old work explainable?
- Explain: Why is retention a reliability decision, not only a storage decision?
- Apply: Investigate a six-month-old job after a model route changed.
- Evidence: Name the prompt version, model version, policy version, worker build, event timeline, and restore practice.

## Summary

Running for years means designing for change. The durable parts are the ledger, events, typed contracts, policy decisions, receipts, memory rules, and ownership records.

**Invariant:** old work remains understandable and safe after code, schema,
provider, prompt, model, policy, and staffing changes.

**Evidence:** version fields, compatibility policies, retention records,
restore drills, owner review dates, and provider-continuity plans remain
inspectable.

**Carry forward:** longevity is a compatibility problem before it is a scaling
problem.

## Changed Understanding

- **Before this chapter:** long-running reliability looked like keeping the process alive.
- **After this chapter:** systems run for years when ownership, maintenance cadence, drift detection, data hygiene, and recovery practice keep the invariants alive.
- **Keep:** inspect the maintenance cadence for ownership, drift, recovery practice, and data-retention evidence.

## Further Reading & Credible References

- **[Meir M. Lehman: Laws of Software Evolution](https://en.wikipedia.org/wiki/Lehman%27s_laws_of_software_evolution)**. The foundational academic research (1974-1996) describing the inevitable decline in quality and increase in complexity of long-lived software systems. It provides the "reality check" for the maintenance cadence described in this chapter.
- **[Crunchy Data: Postgres Autovacuum Tuning](https://www.crunchydata.com/blog/postgres-autovacuum-tuning)**. The definitive industry guide to preventing "Database Bloat" in high-traffic task queues, explaining how to adjust scale factors for million-row tables.
- **[EDPB: Guidelines on Data Protection by Design and by Default](https://www.edpb.europa.eu/our-work-tools/our-documents/guidelines/guidelines-42019-article-25-data-protection-design-and_en)**. The regulatory and technical standard for implementing the "Storage Limitation" and "Erasure" workflows discussed in this chapter's privacy review section.
- **[Designing Data-Intensive Applications](https://dataintensive.net/)** (Martin Kleppmann, Chapter 4: Encoding and Evolution). The primary reference for Schema Evolution (backward and forward compatibility) used to ensure old agent rows remain parseable by new worker code.
- **[ICO: Storage Limitation Guidance](https://ico.org.uk/for-organisations/guide-to-data-protection/guide-to-the-general-data-protection-regulation-gdpr/principles/storage-limitation/)**. A high-signal practical guide for developers on setting retention periods and implementing "Crypto-shredding" for data in backups.
