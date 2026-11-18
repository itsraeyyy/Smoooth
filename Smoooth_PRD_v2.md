# SMOOOTH — MASTER PRODUCT REQUIREMENTS DOCUMENT
## Vision, Product Strategy, Architecture, Principles & Long-Term Execution

**Document:** Master PRD  
**Product:** Smoooth  
**Category:** Agent-Native Application Infrastructure Platform  
**Strategic Direction:** Unified infrastructure and backend platform  
**Deployment Philosophy:** Hybrid — managed cloud and customer-owned infrastructure  
**Primary Users:** Human developers, AI coding agents, startups and businesses  
**Status:** Foundational product vision and strategic requirements  
**Audience:** Engineering agents, developers, product designers and future collaborators

---

# 1. EXECUTIVE VISION

## 1.1 What we are building

Smoooth is an agent-native application infrastructure platform that provides everything required to build, deploy, operate and scale modern software applications.

A developer should be able to describe an application to an AI coding agent and have that agent use Smoooth to provision the infrastructure, configure the backend, connect integrations, deploy the application and manage its operations.

Smoooth should make the infrastructure behind almost any application accessible through one coherent platform.

This includes:

- Databases and data infrastructure.
- Authentication and authorization.
- File and object storage.
- Application APIs.
- Compute and application hosting.
- Background jobs and queues.
- Realtime communication.
- Payments and billing integrations.
- Email, SMS and notifications.
- Networking and deployment environments.
- Monitoring, logging and observability.
- Secrets and configuration management.
- Agent access, permissions and execution.
- Infrastructure automation and orchestration.

We are building toward a platform where developers do not need to independently assemble, configure and maintain a fragmented collection of infrastructure products just to bring an application to life.

They should have the freedom to choose the underlying technologies while Smoooth makes them work together.

## 1.2 The central promise

**Your agent builds it. Smoooth powers it.**

A developer should be able to ask:

"Build me an ERP system with employee authentication, organizational permissions, PostgreSQL, document storage, email notifications, background processing and a mobile application."

Their coding agent should understand the requirements, use Smoooth to provision and configure the necessary infrastructure, implement the application and verify that everything works.

The developer remains in control.

The agent receives the tools and authority it needs, but only within explicitly granted permissions.

## 1.3 The ambition

Smoooth should become the infrastructure platform that AI coding agents naturally reach for when building software.

We want Smoooth to be useful for:

- A freelancer building a simple SaaS.
- A startup building a multi-tenant application.
- A company building an ERP or CRM.
- A team developing mobile applications.
- A developer building a marketplace.
- A studio creating multiplayer games.
- An AI engineer building agentic applications.
- An enterprise operating internal systems.
- A developer deploying applications on their own VPS.
- A team managing infrastructure across multiple providers.

The product must not be architecturally restricted to one application category, one framework, one database or one hosting provider.

---

# 2. THE PROBLEM

## 2.1 Application development is becoming easier; infrastructure remains fragmented

AI coding agents have dramatically changed how software is created.

A developer can now describe a complex application and have an agent generate substantial portions of the codebase.

However, building the application is only one part of the process.

The application still needs:

- A database.
- Authentication.
- Storage.
- Deployment infrastructure.
- Environment configuration.
- Secrets.
- External services.
- Networking.
- Background processing.
- Monitoring.
- Backups.
- Security controls.
- Production operations.

These capabilities are distributed across many providers, dashboards, APIs and configuration systems.

An AI coding agent may know how to integrate PostgreSQL, Better Auth, Stripe, Resend, Redis and Docker independently, but the developer still has to coordinate these systems.

This creates friction.

## 2.2 The current experience

A typical workflow looks like this:

1. Create a project.
2. Find a database provider.
3. Provision a database.
4. Copy credentials.
5. Configure environment variables.
6. Set up authentication.
7. Configure email delivery.
8. Configure payment processing.
9. Find a storage provider.
10. Set up application hosting.
11. Configure deployments.
12. Configure monitoring.
13. Connect everything.
14. Debug integration problems.
15. Maintain the infrastructure independently.

Every additional service introduces another integration, another set of credentials, another dashboard and another operational responsibility.

Smoooth should reduce this fragmentation.

## 2.3 The opportunity

The future of software development will increasingly involve humans directing AI agents that build and operate applications.

Infrastructure platforms must become understandable and operable by these agents.

Smoooth should provide:

- A unified infrastructure interface.
- Consistent APIs and resource models.
- Agent-accessible documentation.
- Native MCP capabilities.
- CLI and SDK support.
- Infrastructure orchestration.
- Secure delegated permissions.
- Automated configuration and verification.
- Hybrid infrastructure management.

Our opportunity is to make infrastructure feel like a coherent system rather than a collection of unrelated services.

---

# 3. PRODUCT POSITIONING

## 3.1 Category

**Agent-native application infrastructure platform.**

## 3.2 Primary positioning

**SMOOOTH.**

**Everything behind your application. One platform.**

Build any application. Give your agents the infrastructure, backend services and integrations to bring it to life.

Databases, authentication, storage, payments, email, compute and more.

Use the technologies you prefer, connect the infrastructure you own and manage everything through Smoooth.

## 3.3 Core tagline

**Your agent builds it. Smoooth powers it.**

## 3.4 Differentiation

Smoooth should compete in the broader application infrastructure category.

Our differentiation rests on five principles:

1. **Agent-native:** AI agents are first-class platform users, not an afterthought.
2. **Infrastructure freedom:** Developers retain meaningful control over technology choices.
3. **Hybrid ownership:** Applications can use managed infrastructure or customer-owned infrastructure.
4. **Unified orchestration:** Multiple technologies and services are coordinated through one platform.
5. **Application independence:** The platform supports many kinds of applications, not just one framework or workload.

We must earn these advantages through actual product quality. Do not make unsupported superiority claims.

---

# 4. PRODUCT PRINCIPLES

These principles govern architecture, engineering and product decisions.

## 4.1 Freedom over lock-in

Developers should be able to choose their preferred technologies.

Smoooth must avoid making unnecessary proprietary replacements for mature, reliable infrastructure.

Where excellent solutions already exist, integrate them.

## 4.2 Integration over reinvention

We should not build a payment processor when we can integrate Stripe or another appropriate provider.

We should not create an authentication framework just to compete with Better Auth.

We should not recreate mature databases, email delivery networks or observability systems without a compelling reason.

Smoooth's value is in making these technologies accessible, configurable, secure and interoperable.

## 4.3 Humans and agents are equal first-class users

Every important capability should be usable through:

- Dashboard.
- CLI.
- REST API.
- MCP.
- SDKs.
- Agent skills and instructions.

An operation available in the dashboard should have a deliberate programmatic equivalent where appropriate.

Agents must receive structured responses and actionable errors.

## 4.4 Hybrid by design

Customers should not be forced into Smoooth Cloud.

They should be able to connect infrastructure they already own and use Smoooth to manage it.

## 4.5 Security is foundational

Security must be part of the architecture from the beginning.

It cannot be postponed until the platform becomes popular.

## 4.6 No unnecessary abstraction

Do not hide the capabilities of underlying technologies behind a universal abstraction that makes them less useful.

Smoooth should unify management and orchestration while preserving access to native capabilities.

## 4.7 Quality over premature breadth

The long-term vision is broad.

The initial implementation must be focused.

A small number of capabilities that work exceptionally well is more valuable than dozens of incomplete integrations.

## 4.8 Preserve the existing foundation

Do not rewrite working components merely because a proposed architecture looks cleaner.

First understand the existing implementation, identify real limitations and propose evidence-based changes.

---

# 5. THE PLATFORM: FOUR MAJOR SYSTEMS

The platform should be organized into four connected product systems.

These are logical product boundaries. They do not necessarily require four separate deployable applications or microservices.

## 5.1 SMOOOTH CORE — Infrastructure Foundation

Smoooth Core provides the underlying infrastructure and control plane.

Responsibilities:

- Organization management.
- Project management.
- Environments.
- Resource provisioning.
- Infrastructure lifecycle management.
- Compute.
- Container management.
- Databases.
- Storage.
- Networking.
- Resource monitoring.
- Secrets.
- Backups.
- Infrastructure policies.
- Usage and billing metering.

Core is responsible for understanding what infrastructure exists, where it runs, who owns it and how it should be managed.

## 5.2 SMOOOTH BACKEND — Application Services

Smoooth Backend provides the services applications commonly require.

Capabilities include:

- Database provisioning and connectivity.
- Authentication integrations.
- Authorization integrations.
- Storage APIs and SDKs.
- Application APIs.
- Realtime services.
- Background jobs.
- Queues.
- Scheduled tasks.
- Webhooks.
- Email integrations.
- SMS integrations.
- Push notifications.
- Payments and billing integrations.
- Search integrations.

The goal is to give an application a complete backend foundation without requiring the developer to independently assemble every service.

Some capabilities will be native Smoooth services. Others will be integrations with external providers.

The distinction must always be explicit.

## 5.3 SMOOOTH CONNECT — Integration and Infrastructure Connectivity

Smoooth Connect provides the integration ecosystem.

It connects:

- External service providers.
- Customer-owned infrastructure.
- Cloud providers.
- Databases.
- Authentication frameworks.
- Payment providers.
- Email providers.
- Storage systems.
- Monitoring systems.
- Developer tools.

It should provide a consistent integration experience without pretending every provider behaves identically.

Each integration should have a documented configuration and lifecycle.

## 5.4 SMOOOTH AGENT — Agent-Native Interface and Authority

Smoooth Agent provides the interface through which coding agents discover, configure and operate the platform.

Capabilities include:

- MCP server.
- CLI.
- Agent skills.
- API access.
- Resource discovery.
- Project inspection.
- Infrastructure provisioning.
- Configuration management.
- Deployment operations.
- Logs and diagnostics.
- Health checks.
- Permission management.
- Approval workflows.
- Operation history.
- Recovery guidance.

The agent layer must be designed around reliable task completion.

It should not simply expose a collection of tools and assume that agents will use them correctly.

---

# 6. THE INFRASTRUCTURE MODEL

## 6.1 Hybrid infrastructure

Smoooth must support two primary infrastructure modes.

### A. Smoooth Cloud

Smoooth manages the underlying infrastructure.

Customers receive:

- Managed resources.
- Provisioning.
- Configuration.
- Maintenance.
- Monitoring.
- Backups where supported.
- Scaling capabilities.
- Usage visibility.
- Simplified operations.

The customer interacts with Smoooth rather than managing every underlying machine.

### B. Customer-owned infrastructure

Customers connect their own:

- VPS instances.
- Dedicated servers.
- Private cloud environments.
- Docker hosts.
- Existing databases.
- Storage systems.
- Other supported infrastructure.

Smoooth provides a unified management layer while workloads remain on customer-controlled infrastructure.

The customer retains control and portability.

## 6.2 Connected infrastructure architecture

The intended architecture:

1. The customer registers an infrastructure resource.
2. Smoooth establishes a trusted connection.
3. A Smoooth node or agent runs on the customer's infrastructure.
4. The node reports resource health and capabilities.
5. Smoooth authorizes operations according to customer policy.
6. Workloads execute on the customer's infrastructure.
7. Application traffic reaches the workload directly.
8. Smoooth coordinates lifecycle operations and provides management visibility.

Prefer authenticated outbound connections where practical to reduce the need for publicly exposed management ports.

The control plane must not become a mandatory proxy for every application request or database query.

## 6.3 Control plane and data plane

This separation is fundamental.

**Control plane:**

- Identity.
- Organizations.
- Projects.
- Policies.
- Resource metadata.
- Provisioning.
- Configuration.
- Orchestration.
- Billing.
- Audit records.
- Infrastructure management.

**Data plane:**

- Application workloads.
- Database queries.
- File operations.
- Realtime traffic.
- Background jobs.
- Application APIs.
- User requests.

The control plane coordinates the data plane.

The data plane performs application work.

They must be designed to fail independently where possible.

A temporary Smoooth control-plane outage should not automatically take down every running customer application.

## 6.4 Self-hosted control plane

Full self-hosting of the Smoooth control plane is a future capability.

Do not confuse connecting customer infrastructure with deploying the entire Smoooth platform independently.

Prioritize reliable connected infrastructure first.

Design for future portability without letting that requirement delay the initial product.

---

# 7. DATABASE AND DATA INFRASTRUCTURE

## 7.1 Multi-database philosophy

Smoooth should support multiple database technologies.

It must not force every application into one database engine.

Potential supported systems include:

- PostgreSQL.
- SQLite.
- Redis-compatible systems.
- Document databases.
- Other engines based on demand and technical feasibility.

Object storage should support S3-compatible systems.

## 7.2 Database strategy

The initial control-plane database can be PostgreSQL.

This does not mean every customer application must use PostgreSQL.

Application databases should be independently provisioned and managed through provider adapters.

Recommended progression:

1. Managed PostgreSQL.
2. Customer-owned PostgreSQL.
3. Redis-compatible infrastructure.
4. S3-compatible object storage.
5. SQLite and embedded data use cases.
6. Additional engines based on customer demand.

## 7.3 Avoid a universal query abstraction

Different database technologies have different capabilities.

Do not force them into a lowest-common-denominator query interface.

Smoooth should provide:

- Consistent resource lifecycle management.
- Connection and credential management.
- Health information.
- Provisioning.
- Backups where supported.
- Native connection information.
- Provider-specific configuration.
- Migration and operational guidance.

Application developers should retain access to the native capabilities of their chosen database.

---

# 8. BACKEND SERVICES AND INTEGRATION ECOSYSTEM

Smoooth should make it easy to assemble a complete application backend.

## 8.1 Authentication

Better Auth should be a first-class integration candidate.

Better Auth is an authentication framework that runs within the application. It is not simply a hosted authentication service.

Smoooth should make its installation, configuration and infrastructure dependencies straightforward.

Potential capabilities:

- Email/password authentication.
- Social OAuth.
- Session management.
- Organizations.
- Role-based authorization.
- Passkeys.
- Two-factor authentication.
- SSO where supported.
- API keys.
- Application-specific permissions.

Smoooth should help agents:

1. Detect the application framework.
2. Install compatible Better Auth packages.
3. Provision or select a supported database.
4. Configure database access.
5. Generate and securely configure secrets.
6. Configure OAuth providers.
7. Connect email delivery.
8. Apply migrations.
9. Implement application authorization.
10. Test the complete authentication flow.

The application should own its authentication implementation.

Smoooth should provide infrastructure and configuration support rather than requiring proprietary Smoooth authentication.

### Important identity separation

There are two distinct identity systems:

**Smoooth platform identity:** Who can access and manage Smoooth organizations, projects and infrastructure.

**Application identity:** Who can sign into a customer's application.

These systems must remain separate trust boundaries.

They may integrate, but one must not automatically grant access to the other.

## 8.2 Payments

Potential integrations:

- Stripe.
- Polar.
- Regional payment providers.
- Other payment gateways.

Smoooth should provide setup guidance, credentials management, webhook configuration, environment separation and testing support.

Smoooth does not need to become a payment processor.

## 8.3 Email

Potential integrations:

- Resend.
- Postmark.
- Amazon SES.
- SMTP-compatible providers.

Support configuration, credentials, sender domains, verification guidance, templates and testing.

## 8.4 SMS and notifications

Potential integrations:

- Twilio.
- Regional SMS providers.
- Firebase Cloud Messaging.
- Apple Push Notification service.

Support provider configuration and application-level integration guidance.

## 8.5 Observability

Potential integrations:

- OpenTelemetry.
- Sentry.
- PostHog.
- Other compatible monitoring systems.

Developers should be able to understand application health, errors, performance and infrastructure behavior.

## 8.6 Other integrations

Potential future integrations:

- Meilisearch.
- Typesense.
- Temporal.
- Workflow and job systems.
- External AI model providers.
- Webhook providers.
- Cloud infrastructure providers.

These are candidates, not a commitment to support everything at launch.

## 8.7 Integration registry

Create a structured integration system.

Each integration should define:

- Provider identity.
- Supported versions.
- Compatibility requirements.
- Configuration schema.
- Required credentials.
- Environment variable mapping.
- Installation instructions.
- SDK and framework guidance.
- API documentation.
- MCP instructions where useful.
- Webhook setup.
- Health checks.
- Verification procedures.
- Error handling.
- Upgrade information.
- Security requirements.

Integrations should be versioned and testable.

Never advertise an integration as supported simply because an agent can theoretically install its package.

---

# 9. APPLICATION BLUEPRINTS

A major long-term capability should be the Smoooth Application Blueprint.

An application blueprint describes what an application needs.

For example:

**Application:** Business ERP

**Application components:**
- Next.js web application.
- React Native mobile application.
- PostgreSQL database.
- Better Auth.
- S3-compatible document storage.
- Resend email.
- Stripe payments.
- Background workers.
- OpenTelemetry monitoring.

Smoooth should interpret this blueprint and produce an infrastructure plan.

It should identify:

- Required resources.
- Compatible providers.
- Configuration dependencies.
- Credentials that must be supplied.
- Estimated resource requirements where possible.
- Security implications.
- Deployment dependencies.

The agent can then request approval and apply the plan.

## 9.1 Blueprint requirements

Blueprints should eventually be:

- Declarative.
- Version-controlled.
- Reproducible.
- Environment-aware.
- Validatable.
- Inspectable.
- Applicable through APIs and agents.
- Capable of reporting configuration drift.

Smoooth should support reviewing proposed changes before applying them.

Destructive changes must require appropriate authorization.

## 9.2 Infrastructure as code

Infrastructure configuration should be exportable and reproducible.

Customers must not be trapped in a dashboard-only configuration model.

The long-term direction is a versioned application infrastructure manifest that describes the desired state of a project.

---

# 10. APPLICATION USE CASES

Smoooth must be designed to support different application categories.

These examples define the breadth of the vision. They are not all initial launch requirements.

## 10.1 SaaS applications

Requirements:

- Multi-tenant databases.
- Authentication.
- Authorization.
- Storage.
- APIs.
- Billing integrations.
- Email.
- Background processing.
- Monitoring.
- Deployments.

## 10.2 ERP and business systems

Requirements:

- Transactional databases.
- Role-based access.
- Audit logs.
- Document storage.
- Workflow processing.
- Reporting infrastructure.
- Background jobs.
- Third-party integrations.
- Reliable backups.

## 10.3 Mobile applications

Requirements:

- Backend APIs.
- Authentication.
- File uploads.
- Push notifications.
- Data synchronization.
- Offline-friendly architecture.
- Realtime capabilities.
- Mobile application deployment support where appropriate.

## 10.4 Marketplaces

Requirements:

- User and seller identities.
- Listings and transactional data.
- Payment integrations.
- Notifications.
- Search.
- File storage.
- Background processing.
- Auditability.

## 10.5 Multiplayer games

Requirements:

- Authoritative game servers.
- Low-latency networking.
- Matchmaking.
- Player profiles.
- Persistent game state.
- Leaderboards.
- Session management.
- Analytics.

Game infrastructure requires specialized architecture. Do not assume ordinary web hosting and a database are sufficient.

## 10.6 AI applications and agent systems

Requirements:

- Model provider integrations.
- Agent execution environments.
- Persistent data and memory storage.
- Job queues.
- Webhooks.
- Tool credentials.
- Scoped agent permissions.
- Execution logs.
- Resource isolation.
- Monitoring.

The infrastructure should support both applications that use AI and agents that operate infrastructure.

---

# 11. AGENT-NATIVE EXPERIENCE

This is one of Smoooth's most important product requirements.

## 11.1 MCP

Smoooth must provide a reliable, well-designed MCP interface.

Agents should be able to:

- Discover organizations and projects.
- Inspect resources.
- Create supported infrastructure.
- Configure services.
- Deploy applications.
- Read logs.
- Diagnose failures.
- Restart workloads.
- Inspect health.
- Manage supported environments.
- Review permissions.
- Request approvals.

MCP tools must have clear descriptions, strict schemas and predictable behavior.

Avoid tools that perform many unrelated operations with ambiguous arguments.

## 11.2 CLI

The CLI must provide a powerful and consistent interface.

It should support:

- Authentication.
- Organization and project selection.
- Infrastructure inspection.
- Resource management.
- Deployments.
- Logs.
- Secrets.
- Environment configuration.
- Diagnostics.
- Local development workflows.
- Agent access.

CLI operations should have structured output modes, especially JSON, so agents can reliably interpret results.

## 11.3 SDKs and APIs

Provide stable APIs and SDKs that allow both human developers and agents to interact with Smoooth.

API behavior must be versioned and documented.

## 11.4 Agent skills

Provide official skills for major coding-agent environments.

The instructions should explain:

- How to discover Smoooth resources.
- How to provision infrastructure.
- How to configure integrations.
- How to deploy.
- How to inspect errors.
- How to validate functionality.
- How to request approvals.
- How to recover from failures.

Documentation should be concise, current and executable where possible.

## 11.5 Agent reliability

Agent compatibility is not measured by the number of available MCP tools.

Measure:

- Successful task completion.
- Time to first successful deployment.
- Number of failed tool calls.
- Number of human interventions.
- Incorrect configurations.
- Deployment failures.
- Security policy violations.
- Recovery success.
- Resource provisioning time.

Build repeatable end-to-end agent evaluation tasks.

Smoooth should be tested with real coding agents and real applications.

---

# 12. SECURITY AND TRUST ARCHITECTURE

Security is a product requirement, not a future enhancement.

## 12.1 Identity and authorization

Implement:

- Organization isolation.
- Project isolation.
- Environment isolation.
- Explicit resource permissions.
- Role-based access where appropriate.
- Agent-specific identities.
- Short-lived scoped credentials.
- Revocation.
- Auditable authorization decisions.

Agents must not automatically inherit every permission held by the human who connected them.

## 12.2 Agent authority

Use a clear authority model:

**Owner → Policy Engine → Agent Identity → Authorized Operation → Isolated Execution**

Every meaningful operation should be evaluated against:

- Who initiated it.
- Which project it affects.
- Which resource it affects.
- What permission is required.
- Whether approval is required.
- Whether the operation is reversible.
- Whether the operation is destructive.

## 12.3 Destructive operations

Require explicit authorization or approval for high-impact actions, including:

- Deleting databases.
- Destroying infrastructure.
- Removing persistent volumes.
- Rotating critical credentials.
- Changing network exposure.
- Deleting backups.
- Performing irreversible migrations.

The exact approval rules should be configurable within safe boundaries.

## 12.4 Secrets

Secrets must:

- Be encrypted at rest.
- Be protected in transit.
- Be scoped by project and environment.
- Be excluded from logs.
- Be redacted from agent responses.
- Have controlled access.
- Support rotation.
- Be auditable.

Never return secret values unnecessarily after provisioning.

## 12.5 Workload isolation

Customer workloads must be isolated.

Implement appropriate:

- Container isolation.
- Resource limits.
- Network boundaries.
- Filesystem restrictions.
- Process restrictions.
- Credential boundaries.
- Execution controls.

Never treat an AI agent's request as sufficient authorization to execute arbitrary commands on infrastructure.

## 12.6 Operational security

Include:

- Audit logging.
- Rate limiting.
- Abuse prevention.
- Backup verification.
- Recovery testing.
- Secure updates.
- Dependency management.
- Vulnerability handling.
- Incident response procedures.

Security-critical behavior requires automated tests and documented verification.

---

# 13. PROPOSED TECHNICAL ARCHITECTURE

The following is the target architectural direction. It must be reconciled with the existing Smoooth repository before implementation.

## 13.1 Architecture layers

### Layer 1: Interfaces

- Web dashboard.
- CLI.
- MCP.
- REST API.
- SDKs.
- Agent skills.

### Layer 2: Control plane

- Identity.
- Organizations.
- Projects.
- Environments.
- Resource registry.
- Authorization.
- Policy engine.
- Provisioning orchestration.
- Billing and metering.
- Audit.
- Integration registry.

### Layer 3: Platform services

- Database management.
- Storage management.
- Compute management.
- Container lifecycle.
- Deployment orchestration.
- Secrets.
- Networking.
- Logs and metrics.
- Background operations.
- Service integrations.

### Layer 4: Provider adapters

- PostgreSQL.
- SQLite.
- Redis-compatible systems.
- S3-compatible storage.
- Docker.
- Cloud providers.
- Customer VPS infrastructure.
- External service providers.

### Layer 5: Execution environments

- Smoooth Cloud.
- Customer VPS.
- Dedicated infrastructure.
- Private cloud.
- Future self-hosted Smoooth environments.

## 13.2 Architectural requirements

1. Separate control plane and data plane.
2. Use stable internal interfaces for provider lifecycle operations.
3. Keep provider-specific behavior explicit.
4. Avoid unnecessary microservices.
5. Use asynchronous jobs for long-running provisioning operations.
6. Make operations idempotent where possible.
7. Track operation state and failure details.
8. Support retries safely.
9. Design for partial failure.
10. Maintain auditability.
11. Support resource reconciliation.
12. Preserve portability.

## 13.3 Existing technology foundation

The existing Smoooth project has been built around Rust, Linux, Ubuntu 24.04, Docker, VPS infrastructure, MCP, CLI and a dashboard.

Treat this as existing context, not a guarantee that every implementation detail remains correct.

Before changing the stack, inspect the repository.

Potential architectural roles:

- Rust: control-plane and infrastructure management services.
- PostgreSQL: control-plane persistence.
- Docker: initial workload runtime.
- Redis-compatible infrastructure: caching and job coordination where justified.
- S3-compatible storage: object storage.
- MCP: agent interface.
- CLI: developer and agent interface.
- REST API: stable platform interface.
- OpenTelemetry: observability interoperability.

Do not adopt these choices blindly. Confirm compatibility with the current implementation.

## 13.4 Provider adapters

Provider adapters should expose consistent lifecycle operations where meaningful:

- Discover capabilities.
- Validate configuration.
- Provision.
- Inspect.
- Start.
- Stop.
- Restart.
- Update.
- Delete.
- Retrieve health.
- Retrieve connection information.
- Perform supported backup and recovery operations.

Not every provider supports every operation.

The adapter must accurately communicate capabilities and limitations.

---

# 14. DASHBOARD AND DEVELOPER EXPERIENCE

The dashboard should make infrastructure understandable.

It should not become an overloaded collection of technical settings.

## 14.1 Main product structure

Proposed navigation:

- Overview.
- Projects.
- Infrastructure.
- Backend services.
- Integrations.
- Deployments.
- Logs and monitoring.
- Agent access.
- Settings.

The final navigation should be validated against real workflows.

## 14.2 Project experience

A project should provide a unified view of:

- Application environments.
- Connected infrastructure.
- Databases.
- Storage.
- Deployments.
- Integrations.
- Secrets.
- Agent permissions.
- Activity.
- Health.

## 14.3 Human and agent parity

Where possible, the dashboard should show operations performed by both humans and agents.

Users should understand:

- What changed.
- Who or what changed it.
- When it changed.
- Why it changed.
- Which resources were affected.
- Whether the operation succeeded.
- Whether approval was required.

## 14.4 Design direction

Smoooth should feel like a serious infrastructure product.

Design principles:

- Clear hierarchy.
- High information quality.
- Calm visual language.
- Excellent responsive behavior.
- Fast navigation.
- Consistent interaction patterns.
- Strong empty states.
- Clear error states.
- No decorative complexity.
- No unnecessary dashboard clutter.

The interface should make complex infrastructure feel manageable without hiding important information.

---

# 15. COMMERCIAL MODEL

The intended business model combines platform access and infrastructure consumption.

## 15.1 Revenue components

Potential revenue:

1. Platform subscriptions.
2. Managed infrastructure usage.
3. Compute consumption.
4. Storage consumption.
5. Database services.
6. Managed operational services.
7. Team and enterprise capabilities.

Third-party services may be billed directly by their providers or through Smoooth where commercially and technically appropriate.

## 15.2 Pricing principles

- Avoid charging users for every ordinary agent tool call.
- Make infrastructure consumption understandable.
- Do not advertise unlimited infrastructure without clear cost controls.
- Separate platform fees from provider costs.
- Make managed versus customer-owned costs transparent.
- Provide usage visibility and spending controls.
- Support predictable budgets.

Actual pricing requires cost modeling and customer validation.

Do not invent final pricing or margins.

---

# 16. PRODUCT ROADMAP AND IMPLEMENTATION PHASES

The entire vision must be understood from the beginning, but implementation must be staged.

## PHASE 0 — REPOSITORY AUDIT AND ARCHITECTURE VALIDATION

**Objective:** Understand what already exists and determine the shortest safe path toward the vision.

Requirements:

- Inspect the complete repository.
- Identify the current architecture.
- Map existing backend services.
- Map Docker and VPS functionality.
- Inspect the MCP implementation.
- Inspect the CLI.
- Inspect the dashboard.
- Review database models and migrations.
- Review authentication and authorization.
- Review secret handling.
- Review networking and remote infrastructure access.
- Review tests and CI.
- Identify security vulnerabilities.
- Identify duplicated systems.
- Identify architectural constraints.

Deliverables:

1. Current-state architecture.
2. Existing feature inventory.
3. Working versus incomplete functionality.
4. Security assessment.
5. Technical debt inventory.
6. Architecture gap analysis.
7. Recommended implementation sequence.
8. Explicit list of what should be preserved.
9. Explicit list of what must change.
10. Prioritized engineering backlog.

Do not begin a large rewrite during the audit.

## PHASE 1 — AGENT-NATIVE INFRASTRUCTURE FOUNDATION

**Objective:** Make Smoooth genuinely useful for building and operating real applications.

Priority capabilities:

- Reliable organization and project management.
- Customer VPS connection.
- Secure Smoooth node or agent.
- Docker workload lifecycle.
- Application deployment.
- Environment configuration.
- Secure secrets management.
- PostgreSQL provisioning and management.
- Persistent storage.
- Logs.
- Health checks.
- Restart and recovery.
- MCP operations.
- CLI parity.
- Scoped agent permissions.
- Destructive-operation approval.
- Audit history.

### Acceptance journey

A developer should be able to:

1. Create a Smoooth project.
2. Connect a server.
3. Give a coding agent scoped access.
4. Ask the agent to provision PostgreSQL.
5. Deploy an application.
6. Connect the application to the database.
7. Verify that application data persists.
8. Inspect logs.
9. Restart the application.
10. Recover from a simulated failure.
11. Confirm that unauthorized destructive operations are blocked and audited.

This must work end to end.

## PHASE 2 — APPLICATION BACKEND FOUNDATION

**Objective:** Expand Smoooth from infrastructure management into a more complete backend platform.

Priorities:

- Better Auth integration.
- Database configuration workflows.
- Application connection management.
- Storage integration.
- Email integration.
- Payment integration.
- Webhook configuration.
- Background jobs.
- Environment management.
- SDKs.
- Framework-specific setup instructions.
- Application templates.

Acceptance journey:

A coding agent creates a working SaaS application with a database, authentication, email verification, file storage and a payment integration using Smoooth.

The agent must verify the integration rather than merely generating configuration files.

## PHASE 3 — UNIFIED BACKEND SERVICES

**Objective:** Build a coherent collection of reusable backend capabilities.

Potential services:

- Realtime.
- Queues.
- Scheduled tasks.
- Functions.
- Notifications.
- Messaging.
- Search.
- API management.
- Event handling.
- Additional identity integrations.

Prioritize based on customer demand, reliability and operational economics.

## PHASE 4 — ADVANCED INFRASTRUCTURE AND HYBRID PLATFORM

**Objective:** Expand infrastructure freedom.

Potential capabilities:

- Managed cloud resources.
- Additional database engines.
- Advanced networking.
- Multi-provider management.
- Scaling.
- Resource placement.
- Advanced backup and recovery.
- Private infrastructure.
- Self-hosted control plane.
- Advanced workload orchestration.

## PHASE 5 — AGENT INFRASTRUCTURE PLATFORM

**Objective:** Make Smoooth a trusted infrastructure operating environment for autonomous agents.

Potential capabilities:

- Agent environments.
- Persistent agent workspaces.
- Delegated authority.
- Policy-driven operations.
- Approval workflows.
- Autonomous diagnostics.
- Controlled recovery.
- Agent resource budgets.
- Agent execution observability.
- Advanced agent evaluations.

These phases are directional. The actual order should be adjusted when repository findings or customer evidence justify it.

---

# 17. QUALITY AND ACCEPTANCE STANDARDS

A feature is not complete because its interface exists.

A feature is complete when it works reliably in a real workflow.

Every significant capability must have:

- Defined behavior.
- Clear ownership.
- Permission requirements.
- Error handling.
- Logging.
- Tests.
- Documentation.
- API behavior.
- Agent accessibility where appropriate.
- Security verification.
- Operational visibility.

## 17.1 Reliability

Long-running infrastructure operations must have:

- Explicit operation states.
- Progress visibility.
- Timeouts.
- Safe retries.
- Failure reasons.
- Recovery instructions.
- Idempotency protections where possible.

## 17.2 Testing

Use multiple testing layers:

- Unit tests.
- Integration tests.
- API tests.
- Provider adapter tests.
- Security tests.
- End-to-end tests.
- MCP tests.
- CLI tests.
- Agent workflow evaluations.

## 17.3 Release standards

Do not release security-sensitive infrastructure functionality without adequate verification.

Do not mark a feature as complete when it only works in a mocked environment.

Test against actual infrastructure whenever practical.

---

# 18. SUCCESS METRICS

Smoooth must measure product usefulness, not just activity.

## Developer activation

- Time from account creation to first working deployment.
- Time from project creation to database connection.
- Time to complete a standard backend setup.
- Percentage of developers who successfully deploy.

## Agent performance

- Successful end-to-end task completion.
- Failed tool calls.
- Human intervention rate.
- Recovery success.
- Unauthorized operation attempts blocked.
- Time to complete infrastructure tasks.

## Platform reliability

- Provisioning success rate.
- Deployment success rate.
- Infrastructure availability.
- Mean time to detect failures.
- Mean time to recover.
- Backup restoration success.

## Business performance

- Activated developers.
- Weekly active projects.
- Paying customers.
- Revenue.
- Retention.
- Infrastructure gross margin.
- Customer acquisition cost.
- Expansion revenue.

Do not optimize vanity metrics at the expense of reliability and retention.

---

# 19. COMPETITIVE LANDSCAPE

Smoooth should understand and benchmark against:

- Supabase.
- Appwrite.
- Coolify.
- Convex.
- Railway.
- Render.
- Fly.io.
- Other infrastructure and backend platforms.

Each competitor has different strengths.

Our goal is not to copy their feature lists.

Our goal is to deliver a coherent platform that combines application infrastructure, provider flexibility, hybrid ownership and genuinely excellent agent workflows.

The competitive advantage we want to establish is:

**One infrastructure control plane that makes multiple technologies feel like one coherent platform to humans and AI agents.**

Every competitive claim must be backed by product evidence.

---

# 20. NON-GOALS AND BOUNDARIES

The long-term vision is broad, but these boundaries prevent unnecessary complexity.

Smoooth does not need to:

- Invent its own database engines.
- Replace every authentication framework.
- Become a payment processor.
- Operate its own email delivery network.
- Build every cloud provider's infrastructure.
- Force customers into one programming language.
- Force customers into one database.
- Proxy every application request through its control plane.
- Build a universal abstraction that removes native technology capabilities.
- Support every possible integration before launching.
- Introduce microservices without an operational reason.
- Rewrite the existing codebase without evidence.

Smoooth should own orchestration, resource management, permissions, integrations and the unified developer experience.

It should use mature underlying technologies wherever appropriate.

---

# 21. INSTRUCTIONS TO THE ENGINEERING AGENT

This document defines the destination.

Your responsibility is to help build it correctly.

## 21.1 First task: understand the current system

Before writing implementation code:

1. Read this entire PRD.
2. Inspect the existing repository.
3. Understand the existing product.
4. Map its current architecture.
5. Identify what already works.
6. Identify what is incomplete.
7. Identify security and reliability gaps.
8. Compare the existing implementation with the target architecture.

Do not assume that a feature is missing simply because it is not described in this document.

Do not assume that a feature is complete simply because a route, component or API exists.

Verify behavior.

## 21.2 Produce an implementation assessment

Return:

**A. Current architecture**
- Existing components.
- Responsibilities.
- Technology choices.
- Dependencies.
- Data flows.

**B. Existing capabilities**
- Fully implemented.
- Partially implemented.
- Planned but absent.
- Broken or unreliable.

**C. Critical risks**
- Security.
- Infrastructure reliability.
- Data integrity.
- Permission boundaries.
- Scalability.
- Operational complexity.

**D. Architecture recommendations**
- What should stay.
- What should change.
- What should be introduced.
- What should be deferred.

**E. Phase 1 execution plan**
- Specific features.
- Dependencies.
- Database changes.
- API changes.
- MCP changes.
- CLI changes.
- Dashboard changes.
- Security requirements.
- Testing requirements.
- Acceptance criteria.

**F. Definition of done**
Provide verifiable criteria for each deliverable.

## 21.3 Engineering behavior

Follow these rules throughout development:

1. Do not implement the entire long-term vision at once.
2. Do not make major architectural changes without explaining the reason.
3. Do not duplicate existing working functionality.
4. Do not introduce unnecessary dependencies.
5. Do not expose secrets to agents.
6. Do not grant agents unrestricted infrastructure access.
7. Do not treat a successful API response as proof of successful infrastructure provisioning.
8. Do not mark untested features as complete.
9. Do not hide limitations behind misleading success states.
10. Do not sacrifice security for convenience.
11. Maintain backward compatibility where practical.
12. Document significant architectural decisions.
13. Build automated tests alongside functionality.
14. Keep the product consistent across dashboard, API, MCP and CLI.
15. Validate real workflows, not just individual components.

## 21.4 Decision-making

When you encounter an architectural or product decision:

- Explain the problem.
- Identify realistic alternatives.
- Compare trade-offs.
- Recommend one approach.
- Explain migration implications.
- Identify security implications.
- Identify operational costs.

Do not silently make irreversible decisions.

When information is missing, investigate before guessing.

If a decision materially changes the product direction or creates substantial risk, present it for review before implementation.

## 21.5 Working relationship

Treat this as a long-term product with a serious engineering standard.

We are not trying to create a large feature list or a convincing demo.

We are trying to build infrastructure that developers can trust with real applications and real business operations.

The implementation should be ambitious, but every released capability must be dependable.

---

# 22. THE FINAL PRODUCT EXPERIENCE

Imagine a developer opens Smoooth and creates a project.

They connect their preferred infrastructure or choose managed resources.

They open their coding agent and ask it to build an application.

The agent discovers Smoooth, understands the project and inspects available resources.

It proposes an infrastructure plan.

The developer approves it.

The agent provisions the database, configures authentication, connects storage, sets up email and payment integrations, deploys the application and verifies the result.

The developer can inspect everything through the dashboard.

The agent can continue managing permitted operations through MCP or CLI.

If something fails, Smoooth provides useful diagnostics and a safe recovery path.

The developer can change providers, connect their own infrastructure or export their configuration without rebuilding the entire application around a proprietary platform.

That is the experience we are building.

---

# 23. THE NORTH STAR

Smoooth should become the place where application infrastructure comes together.

Not another isolated database product.

Not just a deployment dashboard.

Not simply an MCP server.

A unified, agent-native infrastructure platform that gives developers the freedom to build almost anything, using the technologies they prefer, on infrastructure they control or choose to manage.

The platform should make building software easier without taking away developer freedom.

It should make AI agents more capable without giving them uncontrolled authority.

It should make infrastructure simpler without hiding the underlying technology.

**Your agent builds it. Smoooth powers it.**

**Build anything. Own your infrastructure. Stay in control.**
