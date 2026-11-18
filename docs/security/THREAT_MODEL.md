# Security Threat Model: Smoooth Infrastructure Platform

## 1. Scope & Objective

This threat model evaluates the Smoooth platform as an agent-native infrastructure manager operating on customer-owned Linux VPS infrastructure. It identifies critical assets, trust boundaries, potential adversaries, concrete attack vectors, and necessary mitigations.

---

## 2. Protected Assets

1. **Host Server Integrity:** The underlying Linux operating system, kernel, systemd runtime, host network interfaces, and host filesystems.
2. **Docker Engine Authority:** Direct access to `/var/run/docker.sock` and the Docker API, which is effectively equivalent to root access on the host.
3. **Database Data & Persistent Volumes:** Customer production and staging data stored in PostgreSQL volumes and persistent storage.
4. **Secret Keys & Credentials:** Master encryption keys (`SecretStore`), agent authentication tokens, database superuser passwords, environment secrets, and TLS private keys.
5. **Infrastructure State & Audit Trail:** Durable execution logs, audit events, agent scopes, fencing epochs, and idempotency records stored in `smoooth_state.json`.
6. **Workload Isolation:** Separation between different workspaces, environments (development, staging, production), and customer applications.

---

## 3. Trust Boundaries

```
+---------------------------------------------------------------------------------+
| Trust Zone 0: Public Network / External Untrusted Environment                    |
| - AI Agents calling over network (Claude Code, Cursor, Windsurf)                |
| - Internet web clients querying custom domains                                  |
+---------------------------------------------------------------------------------+
                                       |
    [Trust Boundary 1: Authentication & Transport Layer (HTTP / Bearer / SSE)]
                                       |
+---------------------------------------------------------------------------------+
| Trust Zone 1: Smoooth Application Layer (Local Loopback 127.0.0.1:4040)          |
| - Axum HTTP Server & Auth Middleware                                            |
| - Model Context Protocol (MCP) Dispatcher                                       |
| - Orchestrator, DAG Resolver, Idempotency Engine                                |
| - Local State Snapshot Storage (`smoooth_state.json`)                           |
+---------------------------------------------------------------------------------+
                                       |
    [Trust Boundary 2: Execution Broker & Authorization Policy Engine]
                                       |
+---------------------------------------------------------------------------------+
| Trust Zone 2: Provider Execution Boundary                                       |
| - Provider Adapters (DockerAdapter, PostgresAdapter, etc.)                      |
| - Typed Workload Specification Validation                                       |
+---------------------------------------------------------------------------------+
                                       |
    [Trust Boundary 3: Container Runtime & System Boundary]
                                       |
+---------------------------------------------------------------------------------+
| Trust Zone 3: Containerized Workloads & Host Operating System                   |
| - Linux Kernel, Namespaces (cgroups, pid, net, mnt)                            |
| - Docker Daemon Socket (`/var/run/docker.sock`)                                 |
| - Dedicated `smoooth:smoooth` System User                                       |
+---------------------------------------------------------------------------------+
```

---

## 4. Adversary Profiles

| Adversary | Motivation | Capabilities |
| :--- | :--- | :--- |
| **Rogue or Compromised AI Agent** | Malicious intent or prompt-injection hallucinations causing destructive actions | Possesses valid scoped API/MCP token (e.g. `Deploy` or `Plan`). Capable of submitting arbitrary JSON payloads, requesting unauthorized ports/volumes, or attempting workspace traversal. |
| **Malicious Workload / Container Escape** | Attacker who gained RCE inside a container provisioned by Smoooth | Executes within container namespace. Attempts container breakout to host via Docker socket, shared volumes, or kernel exploits. |
| **Untrusted Repository / Build Injection** | Attacker submitting malicious code to be built via Nixpacks/Docker | Injects malicious build scripts (`build.sh`, `Makefile`, `Procfile`) or Git options into the build engine. |
| **Network Eavesdropper / MitM** | Intercepts unencrypted traffic if Smoooth is misconfigured to bind `0.0.0.0` | Listens to loopback or LAN traffic, attempts replay attacks or token harvesting. |
| **Local Host Unprivileged User** | Multi-tenant host user attempting privilege escalation | Local user on Linux server attempting to read `/etc/smoooth/auth.token`, `/var/lib/smoooth`, or talk to loopback 4040. |

---

## 5. Attack Surfaces & Concrete Abuse Scenarios

### Surface 1: Docker Socket Access & Group Membership
- **Mechanism:** The `smoooth` service account is added to the `docker` group in `scripts/install.sh`.
- **Abuse Scenario:** Any code execution within the Smoooth daemon or any uncontrolled Docker API call can spawn a container with `HostConfig { privileged: true, binds: ["/:/host"] }`, granting instant root takeover of the entire VPS.
- **Severity:** Critical
- **Likelihood:** High if untyped APIs permit arbitrary container parameters.

### Surface 2: Out-of-Band Process Spawning (`tokio::process::Command`)
- **Mechanism:** Direct invocations of `docker restart`, `docker rm -f`, `git clone`, and `nixpacks` in `src/api.rs` and `src/build/mod.rs`.
- **Abuse Scenario:** In `src/build/mod.rs`, containerized Nixpacks is spawned with `-v /var/run/docker.sock:/var/run/docker.sock`. Any repository code built through this engine can talk directly to the host Docker daemon socket and gain root access on the host.
- **Severity:** Critical
- **Likelihood:** High when building untrusted Git repositories.

### Surface 3: Self-Approval of High-Risk / Production Operations
- **Mechanism:** The REST endpoint `/api/workloads/deploy` creates a plan targeting `production` and immediately auto-approves it with `plan.approve_with_ttl("operator_dashboard", 3600)`.
- **Abuse Scenario:** An agent granted only `Deploy` scope can bypass the entire human review and risk tier governance workflow, deploying unauthorized containers directly to production without operator oversight.
- **Severity:** High
- **Likelihood:** High.

### Surface 4: Capability & Permit Engine Bypassing
- **Mechanism:** Although `src/security/capability.rs` and `src/security/authz.rs` implement cryptographic capability chains, Proof of Possession, and `VerifiedExecutionPermit`, neither `api.rs` nor `mcp/server.rs` actually require or verify these permits in practice.
- **Abuse Scenario:** Fine-grained resource-level and operation-level restrictions are ignored; access defaults to coarse string scopes (`ReadOnly`, `Plan`, `Deploy`, `Destructive`, `Admin`). An agent with `Admin` can approve its own plans and revoke arbitrary credentials.
- **Severity:** High
- **Likelihood:** Immediate.

### Surface 5: Path Traversal & Volume Host Mounts
- **Mechanism:** `VolumeMount` creates Docker binds with `volume_name:container_path`. `VolumeManager::ensure_volume` checks for path traversal, but out-of-band endpoints and direct container specs could bypass this if not validated strictly at a single typed boundary.
- **Severity:** High
- **Likelihood:** Medium.

---

## 6. Threat Mitigation Matrix

| Threat ID | Threat Description | Existing Mitigation | Required Mitigation | Residual Risk |
| :--- | :--- | :--- | :--- | :--- |
| **TM-01** | Docker socket root escalation | Systemd sandbox (`NoNewPrivileges=yes`, `ProtectSystem=strict`, `ProtectHome=yes`) | Introduce strict typed WorkloadSpec; prohibit raw parameter passthrough; run builds in rootless/ephemeral buildkit sandboxes without host socket mount. | Low |
| **TM-02** | Rogue agent deploys to production without approval | None (auto-approved in REST endpoint) | Eliminate auto-approval; enforce hard requirement for independent operator approval on all production and high-risk operations. | Very Low |
| **TM-03** | Malicious build repository accesses Docker socket | None (`-v /var/run/docker.sock` in Nixpacks) | Remove Docker socket mount from build container; isolate build execution into unprivileged sandbox. | Low |
| **TM-04** | Agent approves its own plans | Gated by `Admin` scope | Prohibit caller agent from approving a plan where `created_by == approver`; require human/operator approval token. | Very Low |
| **TM-05** | Production database deletion | `is_persistent` check and `allow_data_destruction` flag | Require multi-factor / cryptographic confirmation permit for destructive database deletion; enforce immutable snapshots before deletion. | Low |
| **TM-06** | Cross-workspace data exfiltration | Workspace ID checking in API and MCP | Enforce URN containment in every adapter execution permit; deny wildcard `*` access to non-operator agents. | Low |
| **TM-07** | Replay of intercepted operation requests | Proof of Possession (unit tests only) | Wire PoP and monotonic fencing epoch into all API and MCP execution channels. | Very Low |
