# Security Gap Analysis: Smoooth Core Subsystems

## 1. Executive Summary

A comprehensive source code audit of the Smoooth repository reveals that while foundational cryptographic components (`SecretStore`, `CapabilityChain`, `ProofOfPossession`, `AuthorizationEngine`) are implemented with high mathematical precision in isolated modules, they are **disconnected from the primary HTTP API, MCP server, and container runtime execution paths**.

The actual execution flow relies on coarse string-based scopes, exposes critical privilege escalation opportunities, allows self-approval of production deployments, and contains out-of-band shell command invocations.

---

## 2. Findings Matrix by Severity

| ID | Severity | Category | Vulnerability / Architectural Gap | Code Reference |
| :--- | :--- | :--- | :--- | :--- |
| **SEC-01** | **CRITICAL** | Host Escalation | Nixpacks build mounts host Docker socket (`/var/run/docker.sock`) into build container | `src/build/mod.rs:182` |
| **SEC-02** | **CRITICAL** | Authorization Bypass | Workload deploy endpoint self-approves production plans with hardcoded operator identity | `src/api.rs:1206-1207` |
| **SEC-03** | **HIGH** | Privilege Boundary | Direct host shell execution via `tokio::process::Command::new("docker")` bypasses adapter boundary | `src/api.rs:1296, 1375` |
| **SEC-04** | **HIGH** | Policy Bypass | Cryptographic `AuthorizationEngine` and `VerifiedExecutionPermit` are only invoked in unit tests | `src/security/authz.rs:78` |
| **SEC-05** | **HIGH** | Authority Abuse | Agents with `Admin` scope can approve their own restricted plans via MCP | `src/mcp/server.rs:270, 522` |
| **SEC-06** | **HIGH** | Operational Gap | PostgreSQL backup is completely missing (`pg_dump` not implemented; only JSON state is backed up) | `src/security/backup.rs` |
| **SEC-07** | **MEDIUM** | Incomplete Model | `Resource` and `Plan` lack organizational, project, and server tenancy dimensions | `src/models/resource.rs:16` |
| **SEC-08** | **MEDIUM** | Untyped Runtime | Workload configuration is passed as unvalidated `serde_json::Value` to Docker adapter | `src/adapters/docker/mod.rs:137` |
| **SEC-09** | **MEDIUM** | Argument Injection | `git clone` in build planner does not terminate option parsing with `--` | `src/build/mod.rs:54` |
| **SEC-10** | **LOW** | Secret Handling | Master key file (`smoooth_master.key`) created in current working directory by default | `src/main.rs:372` |

---

## 3. In-Depth Analysis of Critical & High Findings

### SEC-01: Docker Socket Mount in Nixpacks Build Container
- **Location:** `src/build/mod.rs:181-186`
- **Code:**
  ```rust
  let mut c = Command::new("docker");
  c.args([
      "run", "--rm",
      "-v", "/var/run/docker.sock:/var/run/docker.sock",
      "-v", &format!("{}:/app:ro", working_dir),
      "ghcr.io/railwayapp/nixpacks:latest",
      "build", "/app", "--name", image_tag,
  ]);
  ```
- **Vulnerability:** The container running Nixpacks is given read/write access to the host's `/var/run/docker.sock`. Because Nixpacks parses arbitrary user repository code to build images, an attacker can supply a malicious build plan, hook, or script that directly queries the host Docker socket, escapes the container, and compromises the host operating system with full root privileges.
- **Required Remediation:** 
  1. Terminate Docker socket passthrough into build containers.
  2. Transition image builds to an isolated Dockerfile build pipeline or a rootless buildkit container communicating over an unprivileged network boundary without host socket mounts.

---

### SEC-02: Self-Approval of Production Deployments in REST API
- **Location:** `src/api.rs:1206-1207`
- **Code:**
  ```rust
  let plan_id = format!("plan_{}", uuid::Uuid::new_v4().simple());
  let mut plan = crate::models::Plan::new(plan_id, spec, crate::models::RiskTier::Low, Some(10.0));
  plan.approve_with_ttl("operator_dashboard".to_string(), 3600);
  ```
- **Vulnerability:** When a caller with `Deploy` scope submits a deployment to `/api/workloads/deploy`, the handler sets `environment: "production"`, marks `risk_tier: RiskTier::Low`, and calls `plan.approve_with_ttl("operator_dashboard", 3600)`. This completely circumvents the two-phase plan/approve/execute security architecture, allowing unreviewed production mutations.
- **Required Remediation:**
  1. Remove automatic plan approval from the REST handler.
  2. If an operation targets `production` or requires approval, the handler must return a pending plan ID (`202 Accepted`) requiring explicit human authorization.

---

### SEC-03: Out-of-Band Process Spawning (`tokio::process::Command`)
- **Location:** `src/api.rs:1296, 1375`, `src/ingress/bridge.rs:20, 56, 95`, `src/migration/docker_scanner.rs:14, 36`
- **Code:**
  ```rust
  let output = tokio::process::Command::new("docker")
      .args(["restart", &id])
      .output()
      .await;
  ```
  ```rust
  let _ = tokio::process::Command::new("docker")
      .args(["rm", "-f", &id])
      .output()
      .await;
  ```
- **Vulnerability:** The API layer bypasses the `ProviderAdapter` and `Orchestrator` execution abstraction, executing raw CLI commands against the host system. This breaks audit trail uniformity, bypasses lifecycle validation, and prevents swapping the runtime to Podman or remote nodes.
- **Required Remediation:**
  1. All container lifecycle actions (start, stop, restart, delete, inspect) must route through the typed `RuntimeProvider` abstraction.
  2. Remove all direct process invocations of the `docker` CLI from `api.rs` and ingress helpers.

---

### SEC-04: Disconnected Cryptographic Capability & Permit Engine
- **Location:** `src/security/authz.rs:78`, `src/engine/orchestrator.rs:230-286`
- **Vulnerability:** The repository contains an advanced cryptographic authorization kernel featuring Ed25519 delegation chains, Proof of Possession, and private sealed execution permits (`VerifiedExecutionPermit`). However, `orchestrator.execute_plan()` only checks legacy `plan.is_approval_valid()` and coarse string scopes (`AgentScope::Deploy`). The function `synthesize_legacy_permit` is never called.
- **Required Remediation:**
  1. Integrate `VerifiedExecutionPermit` generation and validation directly into `Orchestrator::execute_plan` and `ProviderAdapter`.
  2. Validate permit target URNs, action parameters, monotonic deadlines, and fencing epochs at the execution boundary before touching Docker.

---

### SEC-05: Agent Self-Approval in MCP Server
- **Location:** `src/mcp/server.rs:270, 522-555`
- **Code:**
  ```rust
  "approve_plan" | "revoke_capability" => crate::models::AgentScope::Admin,
  ```
- **Vulnerability:** An agent provisioned with the `Admin` scope can call the `approve_plan` MCP tool and approve any plan it created itself.
- **Required Remediation:**
  1. Prohibit agents from approving plans they authored (`plan.spec.author != caller.agent_id`).
  2. High-risk and critical plans (e.g. production database drops, infrastructure destruction) must strictly require human operator authorization.

---

### SEC-06: Database Physical Backups Missing
- **Location:** `src/security/backup.rs:69`
- **Vulnerability:** The PRD and marketing copy promise automated PostgreSQL backups and verifiable restoration. In reality, `src/security/backup.rs` only serializes the internal `StateSnapshot` of Smoooth (the list of agents, plans, and metadata). There is no implementation of `pg_dump`, binary WAL backup, or SQL dump verification. If a customer database is corrupted, Smoooth has no database dump to restore from.
- **Required Remediation:**
  1. Implement a genuine database backup subsystem in `PostgresAdapter` utilizing `pg_dump` piped through stream encryption.
  2. Implement restore verification in an isolated temporary container before promoting to live state.

---

## 4. Verification & Testing Requirements

To close these gaps, the following automated tests must be implemented:
1. **Adversarial Root Escape Test:** Verify that Nixpacks builds cannot access `/var/run/docker.sock` and cannot spawn privileged containers.
2. **Production Gate Test:** Verify that an agent calling `/api/workloads/deploy` targeting production receives `202 Awaiting Approval` and cannot execute without an independent human operator token.
3. **Self-Approval Test:** Verify that an agent cannot approve a plan it created.
4. **Execution Permit Enforcement Test:** Verify that an adapter rejects any operation lacking a valid, unexpired `VerifiedExecutionPermit`.
5. **PostgreSQL Backup & Restore Test:** Verify that a live PostgreSQL instance can be dumped to an encrypted archive, verified in a sandbox container, and restored with verifiable row consistency.
