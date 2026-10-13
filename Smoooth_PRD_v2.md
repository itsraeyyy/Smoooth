# Smoooth v2 MVP PRD — Product Requirements Document & Implementation Plan

**Title:** Smoooth v2 MVP PRD (Official Single Source of Truth)\
**Version:** 2.0 MVP\
**Status:** Active Canonical PRD & Implementation Plan\
**Date:** October 2026\
**Principle:** **Build what you want. Choose your stack. Let Smoooth run it.**\
**Core Thesis:** **Agents choose the stack. Smoooth makes it work.**

## 1. The product

Smoooth helps AI coding agents deploy and operate software on
infrastructure the user chooses. Developers can build different kinds of
applications without being forced into one backend, one hosting
platform, or one way of working.

Users can install Smoooth on their own Linux VPS and keep control of
their infrastructure. Later, they may choose a Smoooth-hosted cloud
option. Smoooth is software that makes infrastructure easier to use; it
does not require customers to host their applications on Smoooth-owned
servers.

**Core principle: Agents choose the stack. Smoooth makes it work.**

## 2. The problem

AI coding tools make it easier to build software, but running that
software still requires infrastructure decisions and work: choosing a
runtime and database, provisioning services, configuring networking and
secrets, deploying releases, checking health, handling failures, and
protecting data.

Because familiar managed backends are convenient, coding agents can
default to the same familiar stack even when it is not the best fit. A
developer might pay for a managed backend when a modest VPS and another
database would suit the workload, or discover later that the original
choice is expensive, limiting, or unsuitable.

This is not a claim that one provider is always cheaper or worse.
Managed services can be the right choice. The problem is that
convenience can narrow the choices, and changing those choices often
takes substantial infrastructure work.

Different applications have different needs: - A small SaaS may need a
web app, database, background jobs, and file storage. - A PHP or Laravel
business app may fit a traditional server and relational database. - A
game server may need long-running processes, specific ports, persistent
world data, and predictable resource limits. - An ERP may need several
connected services, backups, controlled upgrades, and strong data
protection. - A high-traffic app may need a different deployment or
database setup from a personal project.

Developers often bridge gaps with provider dashboards, shell commands,
deployment scripts, configuration files, and manual debugging.

## 3. The opportunity

**Make deployment and operations easy without taking away infrastructure
choice.**

A developer tells an AI coding agent what they are building and what
matters: cost, speed, reliability, language, data location, or existing
infrastructure. The agent can propose a suitable stack. Smoooth checks
compatibility, explains trade-offs, provisions and connects supported
services, deploys the app, verifies that it works, and helps keep it
running.

### Product promise

> Tell your coding agent what you want to build. Choose the
> infrastructure that fits. Smoooth helps get it running and keeps it
> working.

Smoooth aims to bring the convenience people value in managed
application platforms to a wider range of technologies and deployment
models.

## 4. What Smoooth is---and is not

### Smoooth is

-   Software users can install on infrastructure they control, starting
    with a Linux VPS.
-   An agent-accessible way to deploy applications and provision the
    services they need.
-   A common interface for different technologies and infrastructure
    providers.
-   A system that connects application code, databases, storage, queues,
    networking, credentials, deployment, health checks, and recovery.
-   A way for agents to do infrastructure work through structured
    operations instead of unrestricted shell access.
-   Designed to become a self-healing operator that detects problems,
    takes safe corrective action within user-defined rules, and verifies
    the result.

### Smoooth is not

-   A requirement to host applications on Smoooth-owned servers.
-   A Docker-only product, even though Docker is the first execution
    path.
-   A JavaScript-only platform or a requirement to use PostgreSQL.
-   A replacement for every cloud provider or managed service on day
    one.
-   An MCP wrapper that merely forwards calls to provider APIs.
-   An unrestricted root shell for coding agents.
-   A promise that every stack is supported or equally reliable from the
    first release.

## 5. Who we build for

### Primary initial customer: developers who build with AI and ship real applications

These developers use Claude Code, Cursor, Codex, or similar tools. They
can build features quickly but do not want to spend hours wiring
servers, databases, networking, deployment, secrets, and monitoring
together. They may be solo developers, indie hackers, consultants, or
small teams.

**Their job:** "Help my coding agent get my application running on
infrastructure I choose, and deal with routine operational problems."

### Secondary customer: small software teams

Small teams ship multiple applications but do not want to assign a
full-time infrastructure engineer to every project. They need repeatable
deployments, service management, access control, visibility, and safe
recovery.

**Their job:** "Let our coding agents handle routine infrastructure work
while the team controls risk and cost."

### Use cases to support over time

-   Small SaaS products.
-   PHP and Laravel applications.
-   Python, Node.js, and other web services.
-   ERP and business systems.
-   Game servers and long-running processes.
-   APIs, scheduled jobs, workers, and internal tools.
-   Multi-service apps combining different technologies.

These are the range Smoooth should accommodate, not claims that all are
supported at launch.

## 6. Core user experience

The main experience happens through a coding agent. The CLI handles
installation, administration, debugging, and direct operation. A
dashboard can provide visibility and controls, but core workflows must
not depend on clicking through a web UI.

Example request:

> Deploy this Laravel application on my VPS. Use a database that fits
> it, keep database access private, configure environment variables, set
> up backups, and verify the app works. Keep infrastructure costs low
> and ask me before any action that could delete data.

Smoooth should help the agent:

1.  **Understand** the app's runtime, dependencies, ports, persistent
    data, and required services.
2.  **Discover** available capabilities on the user's Smoooth
    installation and authorized providers.
3.  **Recommend** a compatible plan and trade-offs. Do not invent exact
    costs when pricing data is unavailable.
4.  **Plan** the resources, connections, expected requirements, and
    risks.
5.  **Check permissions** and request approval when policy requires it.
6.  **Provision** the required runtime and services.
7.  **Connect** services using controlled networking and secure
    credential delivery.
8.  **Deploy** the application.
9.  **Verify** service readiness, database connectivity, and application
    health. A completed command alone is not proof of success.
10. **Operate** by checking health, inspecting logs, diagnosing
    failures, and taking permitted recovery actions.
11. **Explain** what changed, what was verified, what failed, and what
    needs human attention.

The agent should receive structured results it can act on, not just logs
or vague success messages.

## 7. Deployment and ownership model

### First: customer-owned infrastructure

The first distribution model is software installed on a Linux VPS or
supported Linux host the customer controls. The customer chooses and
pays their infrastructure provider. Smoooth runs there and manages
applications and services on that infrastructure. It must not require
workloads or data to move to Smoooth-owned hosting.

Initial target: - Linux host or VPS. - Docker as the first execution
adapter. - A working application runtime and PostgreSQL service path. -
MCP and CLI as primary interfaces for agents and operators. - Durable
state and safe recovery; multi-worker coordination only with suitable
durable storage.

Installation must be short, understandable, and safe. Explain required
privileges and what Smoooth can control. Never expose an unauthenticated
control API to the public internet.

### Later: Smoooth Cloud

A hosted option can remove the need to install and maintain Smoooth's
control software. It may offer a hosted control plane and/or managed
execution. This is a later business and delivery model, not a
prerequisite for the first product.

Separate control-plane logic from the user's workloads so a hosted
control plane can be added without forcing everyone to use
Smoooth-hosted compute. Exact cloud scope, pricing, and responsibility
boundaries remain open decisions.

## 8. Infrastructure choice and adapters

Smoooth must not assume every app belongs on the same stack. It should
consider: - Language and framework. - Database features and
compatibility. - Persistent storage, jobs, and queues. - Network
topology and public exposure. - Performance and resource needs. - Budget
and provider prices when reliable data exists. - Data location, backup,
and recovery requirements. - Existing infrastructure and migration
cost. - Operational complexity.

The agent may recommend a stack, but the system must distinguish
recommendations from approved decisions. Users can set provider
allowlists, preferred technologies, budget ceilings, and rules that
block certain changes.

The core describes capabilities and lifecycle operations; adapters
implement them for a specific runtime or provider. For example, the core
may request a persistent relational database with private connectivity
and verified readiness. A Docker PostgreSQL adapter and a managed
database adapter can implement that request differently.

Each adapter must honestly declare support for provisioning, deletion,
persistence, backup/restore, credential rotation, health checks,
upgrades, rollback, networking, cost information, and recovery. Do not
claim two services are interchangeable if their behavior differs. Reject
or explain plans when an adapter cannot meet a requirement.

**Initial proof:** Docker-based application runtime and PostgreSQL with
real connectivity and verification.

**Designed to expand:** PHP/Laravel, more databases, object storage,
caches, queues, game servers, other runtimes, managed providers,
Kubernetes, and other infrastructure where there is demonstrated demand.
This is a sequence, not a promise to implement everything immediately.

## 9. Product requirements

### P0 --- First credible product

1.  **Install and secure Smoooth:** supported Linux host, authenticated
    operator and agent connections, scoped permissions, safe
    configuration and updates.
2.  **Discover capabilities:** agents can list available capabilities
    and inspect schemas, constraints, and support levels.
3.  **Plan before changes:** structured plan with resources,
    dependencies, risks, and changes. Approvals bind to the exact plan
    version/hash; recheck authorization at execution time.
4.  **Deploy an app:** accept or detect runtime/build configuration, use
    trusted pinned artifacts, start and inspect the app, configure
    ports/environment/networking/health checks.
5.  **Provision and connect PostgreSQL:** create scoped app credentials,
    keep database private by default, verify a real authenticated query,
    preserve data through app redeployments and rollbacks.
6.  **Track operations:** every mutation has an actor, target, status,
    result, and verification evidence. Make retries safe and recover
    from interrupted operations.
7.  **Safe recovery:** detect unhealthy or missing resources, compare
    actual state with intended state, take only policy-approved actions,
    require approval for destructive or high-impact changes, and never
    delete persistent data as routine cleanup.
8.  **MCP and CLI:** working agent interface plus commands to install,
    inspect status, review/approve plans, inspect operations, and view
    audit events. Core workflows work without a dashboard.
9.  **Honest verification:** distinguish "command completed," "resource
    started," and "application verified healthy."

### P1 --- Once the first workflow is reliable

-   Automated backups and tested restoration.
-   Staged credential rotation.
-   Continuous health monitoring and safe reconciliation.
-   Operational history and better incident diagnosis.
-   Multiple apps and environments per host.
-   Resource visibility and budget controls.
-   PHP/Laravel and other runtimes based on customer demand.
-   More database and service adapters.
-   A dashboard for apps, services, operations, incidents, and
    approvals.

### P2 --- Broader platform

-   More VPS and cloud providers.
-   Managed databases and object storage.
-   Kubernetes and multi-host deployments.
-   Team roles, shared policies, and fleet operations.
-   Smoooth Cloud.
-   Better cost/performance comparisons.
-   Cross-provider application topologies.

## 10. Security and user control

Smoooth may be allowed to change real infrastructure. Security is a core
product feature.

-   Never give the coding agent unrestricted root credentials as its
    normal interface.
-   Use typed, validated operations instead of arbitrary shell execution
    for routine infrastructure tasks.
-   Authenticate users, agents, runtimes, and operations.
-   Enforce authorization for every resource lookup and mutation.
-   Encrypt secrets at rest; use secret references where possible and
    reveal plaintext only when needed.
-   Redact secrets from logs, errors, audit events, and tool responses.
-   Keep secrets out of build contexts unless explicitly required.
-   Keep databases and internal services private by default.
-   Bind approvals to exact plans; material changes invalidate approval.
-   Require safeguards for data deletion and public exposure.
-   Record who requested a change, what was authorized, what happened,
    and what verification passed.
-   Use bounded timeouts and retries.
-   Treat repositories, images, packages, and generated configuration as
    untrusted.
-   Recover safely when the control interface or provider is temporarily
    unavailable.

Autonomy levels: - **Read:** inspect authorized status, configuration,
health, and logs. - **Routine changes:** restart or redeploy within
explicit policy. - **Higher-risk changes:** require more authorization
when spending, exposure, downtime, or production risk exceeds configured
limits. - **Destructive changes:** require explicit approval and clear
impact information. Never hide data deletion inside generic cleanup.

## 11. What self-healing means

Self-healing does not mean an AI can change anything it wants. It means
Smoooth can:

1.  Know the intended state of a managed app and its services.
2.  Collect observations such as container state, readiness checks,
    connection tests, and relevant logs.
3.  Detect meaningful failures or differences.
4.  Choose a permitted response based on rules and evidence.
5.  Execute the response safely.
6.  Verify whether the issue is resolved.
7.  Record the incident and result.
8.  Escalate when a fix is uncertain, unsafe, or unsuccessful.

Examples: - Restart a crashed stateless app if policy permits. - Restore
a prior verified app release after a failed deployment. - Detect an
unreachable database and report whether evidence points to service
health, networking, or credentials. - Stop and request approval rather
than deleting a persistent volume to "fix" a database issue.

Introduce self-healing through bounded, testable behaviors---not broad
promises.

## 12. Architecture direction

Keep the core small and technology-neutral. Provider differences belong
in adapters.

Core concepts: - Application and environment. - Resource, service,
capability, and adapter. - Plan and approval. - Operation and lease. -
Service binding. - Credential and secret reference. - Desired and
observed state. - Health check and evidence. - Incident and
reconciliation. - Policy and audit event.

Interfaces: - MCP for coding agents. - CLI for installation and operator
control. - Stable API for integrations and a future hosted control
plane. - Dashboard for visibility and human oversight.

Execution: - Rust is the current engine implementation direction. -
Docker is the first runtime adapter. - PostgreSQL is the first database
service path. - Production operation state must not depend on in-memory
storage. - If multiple workers can act on the same resources, use
durable leases and fencing plus provider-aware idempotency and
reconciliation. Database fencing alone cannot guarantee a stale worker
will not make a Docker API call. - Local installation must remain useful
without Smoooth Cloud. - Separate control-plane logic from
runtime/provider operations so new providers do not require rewriting
the core.

Exact module boundaries and dependencies should be chosen after
inspecting the existing repository.

## 13. First complete milestone

Do not implement every backend at once. Prove the architecture with one
complete, real workflow.

### Golden workflow

A developer has an application repository and a Linux VPS with Smoooth
installed.

1.  A coding agent connects through MCP.
2.  It discovers available capabilities.
3.  It requests an application runtime and PostgreSQL.
4.  Smoooth creates a plan describing resources and connections.
5.  Smoooth checks policy and requests approval when required.
6.  It creates the runtime and database with persistent storage.
7.  It configures private networking and a scoped database credential.
8.  The agent deploys the app.
9.  Smoooth verifies a real database query and the application's health
    endpoint.
10. The user can inspect resources and operation history.
11. A controlled test kills the app container. Smoooth detects the
    failure, takes a permitted recovery action, and verifies recovery.
12. A failed deployment does not destroy or corrupt the database volume.
13. If recovery cannot be done safely, Smoooth explains the problem and
    asks for help instead of guessing dangerously.

The milestone is complete only when an external agent executes this
against real infrastructure and tests exercise actual behavior, not just
successful tool responses.

## 14. Success measures

Measure whether Smoooth removes work while preserving choice.

First release: - Time from connecting an agent to a verified
deployment. - Share of test deployments completed without manual
infrastructure intervention. - Share of failures diagnosed with
actionable evidence. - Recovery success rate for supported failure
scenarios. - Manual commands or dashboard steps required. - Data-loss
incidents in tested deployment/recovery flows: target zero. - Secret
leaks in defined test capture surfaces: target zero. - Share of
operations with recorded authorization and verification evidence.

Customer validation: - Can developers deploy real projects using their
own VPS? - Can they use a runtime/backend their default platform would
not naturally support? - Do they retain control over provider choice and
data? - How much time do they save compared with their current
workflow? - Do they use Smoooth on a second application? - Will they pay
for the software, support, team features, or a future hosted option?

Build tests and demos do not prove product-market fit. Validate with
developers' real projects and observe where the workflow breaks.

## 15. Risks and open questions

1.  **Too broad too early:** start with one complete workflow and a
    reusable adapter model.
2.  **Unclear paying customer:** begin with AI-assisted developers
    shipping real applications, then test whether small teams are the
    stronger customer.
3.  **Cost claims:** never claim Smoooth is always cheaper than managed
    services. Show costs only when pricing and resource assumptions are
    reliable.
4.  **Different adapter guarantees:** report real backup, upgrade,
    networking, and rollback behavior.
5.  **Unsafe autonomy:** use policies, plan-bound approvals, audit
    trails, and bounded recovery.
6.  **Self-hosting friction:** installation and updates must not become
    another DevOps burden.
7.  **Cloud distraction:** prove customer-owned deployments before
    building Smoooth Cloud.
8.  **Overbuilding the UI:** prioritize the agent workflow and operator
    CLI; build dashboard views around real user tasks.

## 16. What we are building now

We are building **the first usable version of an agent-operated
infrastructure tool that runs on a customer's own Linux VPS**.

A coding agent will be able to deploy an application with PostgreSQL,
connect the pieces, verify that the app works, inspect failures, and
recover from a small set of known problems under the user's rules.

This is a small first release, not a throwaway demo. The capability
model, provider boundaries, operation tracking, security controls, and
service connections must support the broader vision.

After this path works reliably, choose the next runtime or service based
on actual user demand---perhaps PHP/Laravel, game-server workloads,
another database, or another provider. Do not choose the expansion only
because it is technically interesting.

## Final product definition

**Smoooth makes it easy for AI coding agents to deploy and operate
software on infrastructure their users choose. It removes the manual
work of setting up, connecting, and maintaining the stack without
forcing every application into the same backend or hosting platform.**

The customer owns the choice. The agent can do the work. Smoooth makes
that work reliable and keeps the user in control.
