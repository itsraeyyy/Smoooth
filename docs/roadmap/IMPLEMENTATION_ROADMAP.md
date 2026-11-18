# Implementation Roadmap: Smoooth Infrastructure Platform

## 1. Roadmap Architecture

The evolution of Smoooth from an advanced single-VPS prototype into an agent-native infrastructure platform is structured into six release gates. No gate may be bypassed without passing its automated test and security verification criteria.

```
+-------------------------------------------------------------------------------+
| PHASE 0: Discovery, Architecture Audit & Threat Modeling (CURRENT)            |
| Status: In Progress. Output: AS_IS, THREAT_MODEL, GAP_ANALYSIS, ADRs         |
+-------------------------------------------------------------------------------+
                                       |
                                       v
+-------------------------------------------------------------------------------+
| PHASE 1: Security Foundation & Execution Boundary Hardening                   |
| Focus: Eliminate Docker root escape, typed WorkloadSpec, eliminate bypasses  |
+-------------------------------------------------------------------------------+
                                       |
                                       v
+-------------------------------------------------------------------------------+
| PHASE 2: Core Infrastructure & Unified Runtime Abstraction                    |
| Focus: RuntimeProvider trait, real pg_dump backup/restore, clean lifecycle   |
+-------------------------------------------------------------------------------+
                                       |
                                       v
+-------------------------------------------------------------------------------+
| PHASE 3: Agent-Native Operational & Approval Workflows                        |
| Focus: High-precision MCP tools, human approval gate, actionable error models |
+-------------------------------------------------------------------------------+
                                       |
                                       v
+-------------------------------------------------------------------------------+
| PHASE 4: Hybrid Cloud Control Plane & Outbound Daemon Pairing                 |
| Focus: Hosted control plane, mTLS/WSS daemon pairing, multi-server inventory  |
+-------------------------------------------------------------------------------+
                                       |
                                       v
+-------------------------------------------------------------------------------+
| PHASE 5: Team Governance, Multi-User RBAC & Commercial Entitlements           |
| Focus: Organizations, projects, role-based access, audit export, tiers        |
+-------------------------------------------------------------------------------+
```

---

## 2. Phase-by-Phase Breakdown

### Phase 1: Security Foundation & Execution Boundary Hardening
**Objective:** Seal all known execution boundary bypasses, eliminate host root escalation paths, and enforce strict typed validation on all operations.

- **Milestone 1.1: Eliminate Build Socket Mount (SEC-01)**
  - Refactor `src/build/mod.rs` to terminate `-v /var/run/docker.sock` in Nixpacks builds.
  - Implement unprivileged containerized build execution or isolate builds behind buildkit with non-root privileges.
  - Fix Git option injection in `git clone` by introducing `--` separator.
- **Milestone 1.2: Enforce Strict Typed Workload Specification (SEC-08)**
  - Replace raw `serde_json::Value` configs with a strongly-typed `WorkloadSpec` struct (image, command, ports, environment, limits, mounts, health check).
  - Reject unknown, privileged, or host-namespace fields before they reach the adapter.
- **Milestone 1.3: Eliminate Self-Approval and Production Auto-Approval (SEC-02, SEC-05)**
  - Remove auto-approval from `src/api.rs:deploy_workload_handler`.
  - Enforce author/approver separation in `src/mcp/server.rs`: caller agent cannot approve its own plan.
  - Require explicit human operator confirmation for `production` environment deployments.
- **Milestone 1.4: Wire Verification Permits into Execution Path (SEC-04)**
  - Connect `AuthorizationEngine::synthesize_legacy_permit` into `Orchestrator::execute_plan`.
  - Ensure every adapter operation verifies permit validity, target URN, and monotonic expiration.
- **Quality Gate 1:**
  - `cargo test --test adversarial_security_test` passes.
  - New regression tests verify: (1) no Docker socket exposure during builds, (2) rejection of self-approval, (3) production deployment pause awaiting approval.

---

### Phase 2: Core Infrastructure & Unified Runtime Abstraction
**Objective:** Deliver a production-grade infrastructure lifecycle, real PostgreSQL backups, and clean runtime decoupling.

- **Milestone 2.1: RuntimeProvider Trait & Adapter Decoupling (SEC-03)**
  - Define formal `RuntimeProvider` async trait (`create`, `start`, `stop`, `restart`, `inspect`, `logs`, `destroy`, `check_health`).
  - Refactor `DockerAdapter` to implement `RuntimeProvider`.
  - Eliminate all `tokio::process::Command::new("docker")` calls in `src/api.rs`, `src/ingress/`, and `src/migration/`.
- **Milestone 2.2: True PostgreSQL Backup & Verifiable Restore (SEC-06)**
  - Implement streaming `pg_dump` execution inside PostgreSQL container, piping stdout through AES-256-GCM encryption to storage.
  - Implement restore verification in an isolated temporary container before live promotion.
  - Add checksum validation and row count evidence to backup records.
- **Milestone 2.3: Tenancy and Resource Model Expansion (SEC-07)**
  - Extend `Resource`, `Plan`, and `Operation` models with `org_id`, `project_id`, `environment`, and `server_id`.
  - Maintain backward compatibility for existing `smoooth_state.json` files via serde default attributes.
- **Quality Gate 2:**
  - Automated roundtrip PostgreSQL backup, verification, and restore test passes on live container.
  - Zero raw shell `Command::new("docker")` instances remain in API or helper modules.

---

### Phase 3: Agent-Native Operational & Approval Workflows
**Objective:** Provide an intuitive, safe, and actionable interface for AI coding agents and human operators.

- **Milestone 3.1: High-Precision MCP Tools**
  - Standardize MCP tool names: `infrastructure_inspect`, `application_deploy`, `application_restart`, `application_logs`, `database_provision`, `database_backup`, `operation_plan`, `operation_approve`.
  - Enrich error responses with actionable policy remediation messages (e.g., explaining which specific permission or approval is missing and how to request it).
- **Milestone 3.2: Human Approval Workflow & Notification**
  - Implement pending approval queue in Orchestrator.
  - Add dashboard notifications and CLI approval prompts (`smoooth approve <plan_id>`).
  - Implement cryptographic plan hash binding with expiration timers.
- **Milestone 3.3: CLI Ergonomics**
  - Add `smoooth db backup` and `smoooth db restore` CLI commands.
  - Provide structured JSON output options (`--json`) for scriptability.
- **Quality Gate 3:**
  - End-to-end simulation of an AI agent proposing an application + PostgreSQL stack, pausing for human approval, executing upon authorization, and reading logs.

---

### Phase 4: Hybrid Cloud Control Plane & Outbound Daemon Pairing
**Objective:** Support centralized multi-server management from Smoooth Cloud without exposing local Docker sockets to the internet.

- **Milestone 4.1: Outbound Daemon Pairing Agent**
  - Implement outbound mutual TLS / secure WebSocket tunnel from local daemon to control plane.
  - Local daemon initiates outbound connection (no open inbound WAN ports required).
- **Milestone 4.2: Local-First Survival & Replay Protection**
  - Ensure daemon continues servicing local workloads and executing local policies if cloud connection drops.
  - Prevent duplicate execution and stale operation replaying upon reconnection using monotonic fencing epochs.
- **Milestone 4.3: Centralized Server & Resource Inventory**
  - Stream health heartbeats, status updates, and audit summaries from daemon to control plane.
- **Quality Gate 4:**
  - Disconnection resilience test: simulate cloud control plane outage; verify local workloads remain running and local CLI remains functional.

---

### Phase 5: Team Governance, Multi-User RBAC & Commercial Entitlements
**Objective:** Prepare Smoooth for enterprise and team deployments with commercial monetization layers.

- **Milestone 5.1: Multi-User Identity & RBAC**
  - Support Organizations, Workspaces, and User Roles (`Owner`, `Admin`, `Operator`, `Viewer`).
  - Distinct agent identities linked to specific users or teams.
- **Milestone 5.2: Commercial Entitlements Engine**
  - Decouple feature gates (`Community`, `Developer`, `Pro`, `Team`, `Business`) from infrastructure logic.
  - Free core remains 100% functional for self-hosted single-server deployments.
- **Milestone 5.3: Production Diagnostics & Audit Export**
  - Add streaming audit export to external SIEM / logging services.
  - Release hardening, pre-flight doctor checks, and update lifecycle scripts.
- **Quality Gate 5:**
  - Multi-tenant isolation test: verify complete barrier between organizations and workspaces across all interfaces.
