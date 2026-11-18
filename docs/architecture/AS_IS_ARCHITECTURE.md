# As-Is Architecture: Smoooth (v0.1.1)

## 1. System Overview

Smoooth is implemented as a single monolithic Rust binary (`smoooth`) that functions simultaneously as:
- A local daemon and background service managed by systemd (`smoooth server`).
- A CLI management and operational tool (`smoooth doctor`, `smoooth agent`, `smoooth approve`, `smoooth migrate`, etc.).
- An Axum-based HTTP REST API and Server-Sent Events (SSE) server on `127.0.0.1:4040`.
- An embedded Model Context Protocol (MCP) server communicating over stdio or HTTP/SSE.
- An embedded administrative single-page dashboard served directly from binary memory (`src/dashboard/mod.rs`).

```
+-------------------------------------------------------------------------------+
|                             Caller Interfaces                                 |
|  +------------------+  +-------------------+  +----------------------------+  |
|  | CLI (Terminal)   |  | AI Coding Agents  |  | Embedded Operator Web UI   |  |
|  | clap Subcommands |  | (Claude, Cursor)  |  | (Axum HTTP Dashboard)      |  |
|  +--------+---------+  +---------+---------+  +--------------+-------------+  |
+-----------|----------------------|---------------------------|----------------+
            |                      | (HTTP Bearer / SSE)       | (HTTP Loopback)
            |                      v                           v
+-----------v-------------------------------------------------------------------+
| Axum HTTP Server & Auth Middleware (src/api.rs)                              |
| - Unauthenticated routes: /, /dashboard, /health, /api/health                 |
| - Authenticated routes: /mcp, /sse, /api/status, /api/workloads/*, etc.       |
| - Auth evaluation: Compares Bearer token to operator secret OR agent hash     |
| - Scopes checked: ReadOnly, Plan, Deploy, Destructive, Admin                  |
+-------------------------------------------------------------------------------+
            |
            v
+-------------------------------------------------------------------------------+
| Orchestrator & State Management (src/engine/orchestrator.rs, src/storage/)     |
| - DAG Topological Sort: Kahn's algorithm in `CanonicalExecutionSpec`          |
| - 2-Phase Execution: Idempotency reservation -> Sequential Adapter dispatch   |
| - Plan Approval: Hash binding (`is_approval_valid`)                          |
| - Storage: In-memory `StateSnapshot` synced atomically to JSON file           |
| - Secret Store: AES-256-GCM envelope encryption with master key               |
+-------------------------------------------------------------------------------+
            |                                           |
            | (Internal Adapter API)                    | (Raw Shell Execution)
            v                                           v
+------------------------------------+   +--------------------------------------+
| Provider Adapters (src/adapters/)  |   | Out-of-band Process Spawns           |
| - Docker: bollard async client     |   | - `docker restart` in api.rs         |
| - Postgres: container + tokio-pg   |   | - `docker rm -f` in api.rs           |
| - Redis: container provisioning    |   | - `nixpacks` & `git clone` in build  |
| - Mongo: container provisioning    |   | - `docker ps/inspect` in scanner     |
+-----------------+------------------+   +------------------+-------------------+
                  |                                         |
                  +--------------------+--------------------+
                                       |
                                       v
+-------------------------------------------------------------------------------+
| Local Linux VPS Host Boundary                                                 |
| - Process: `smoooth:smoooth` system account, member of `docker` group         |
| - Docker Socket: `/var/run/docker.sock`                                       |
| - State Directory: `/var/lib/smoooth/smoooth_state.json` (0750)               |
| - Config Directory: `/etc/smoooth/auth.token` (0700)                          |
+-------------------------------------------------------------------------------+
```

---

## 2. Component Inventory & Verification

### 2.1 Entrypoint and CLI (`src/main.rs`)
- Implements `clap` subcommands: `doctor`, `agent`, `tools`, `capabilities`, `demo`, `status`, `serve`, `mcp`, `approve`, `ops`, `reconcile`, `reconcile-db`, `ingress`, `build`, `migrate`, `capability`, `backup`.
- Instantiates `LocalStorage` file-backed storage (`smoooth_state.json`), resolves or creates master key (`SecretStore`), instantiates `Orchestrator`, and registers adapters (`DockerAdapter`, `PostgresAdapter`, `RedisAdapter`, `MongoAdapter`).

### 2.2 HTTP API & Authentication (`src/api.rs`)
- Framework: `axum` 0.7 with `tower-http` CORS.
- Unauthenticated endpoints: `/`, `/dashboard`, `/health`, `/api/health`.
- Authentication middleware (`auth_middleware`):
  - Extracts Bearer token from `Authorization` header.
  - Matches against `state.auth_token` for operator authority, or computes SHA-256 and matches active agent in `storage`.
  - Checks coarse `AgentScope` (`ReadOnly`, `Plan`, `Deploy`, `Destructive`, `Admin`).
  - Workspace scoping: verifies agent workspace matching target workspace (or wildcard `*`).

### 2.3 Model Context Protocol Server (`src/mcp/`)
- Transports: stdio (`smoooth mcp`) and SSE/HTTP (`/sse`, `/sse/message`, `/mcp`).
- Tool catalog: 16 registered tools (`list_capabilities`, `get_capability_schema`, `doctor`, `scan_foreign_workloads`, `smoooth_migrate`, `migrate_stack`, `create_plan`, `get_plan`, `approve_plan`, `inspect_capability`, `revoke_capability`, `execute_plan`, `create_service_binding`, `configure_ingress`, `build_with_nixpacks`, `verify_connection`, `get_operation`, `reconcile_resource`, `list_resources`).
- Tool-level scope gating: checked in `dispatch_tool_with_caller`.

### 2.4 Orchestration Engine (`src/engine/orchestrator.rs`)
- Workflow:
  1. `CanonicalExecutionSpec` topological sort via Kahn's algorithm (`resolve_execution_order`).
  2. Idempotency reservation (`check_or_record_idempotency`).
  3. Plan verification (`is_approval_valid`, requires approval if non-low risk, production, persistent delete, or public network).
  4. Monotonic leasing and fencing epoch generation.
  5. Sequential execution across adapters.
  6. Rollback / partial failure handling.

### 2.5 Storage Subsystem (`src/storage/local.rs`)
- File-backed JSON snapshot (`smoooth_state.json`).
- Stores agents, plans, idempotency records, operations, resources, service bindings, audit events, revocations, and fencing epochs.
- Saves state atomically by writing to `.tmp` file and renaming over target, preserving `.bak` backup.

### 2.6 Cryptographic & Security Modules (`src/security/`)
- `crypto.rs`: AES-256-GCM envelope encryption for secrets and credentials. HKDF-SHA256 domain separation keys (`db_credentials`, `env_secrets`, `agent_tokens`, `backups`).
- `capability.rs`: Ed25519 cryptographic delegation chains with attenuation rules and monotonic expiration.
- `pop.rs`: Proof-of-Possession challenge/response validation over HTTP/RPC payload digests to prevent token relay attacks.
- `authz.rs`: `AuthorizationEngine` producing private, unforgeable `VerifiedExecutionPermit` proofs.
- `backup.rs`: AES-256-GCM encrypted state snapshot backup, isolated in-memory verification, and confirmation-token-gated live restore.

### 2.7 Runtime & Infrastructure Adapters (`src/adapters/`)
- `docker/`: `bollard` client interface. Provisions containers with `no-new-privileges:true`, dropped Linux capabilities (`NET_RAW`, `SYS_ADMIN`, `SYS_PTRACE`, `SYS_MODULE`, `DAC_OVERRIDE`), and pid limits (256). Volume creation with path traversal protection (rejects paths containing `/`, `\`, `.`).
- `postgres/`: Provisions Postgres container on Docker, waits for `SELECT 1;` readiness over `tokio-postgres`, injects credentials into `SecretStore`, supports Supabase role reconciliation.
- `redis/` & `mongo/`: Container provisioning adapters.

### 2.8 Ingress & Migration Subsystems (`src/ingress/`, `src/migration/`)
- Ingress: Caddy reverse proxy integration, Traefik ACME certificate syncing (`acme.json`), proxy network bridging.
- Migration: Translates Docker Compose (`docker-compose.yml`), Coolify export JSON, Render (`render.yaml`), Fly.io (`fly.toml`), Dokku (`Procfile`), and live Docker host scanning into `CanonicalExecutionSpec`.

---

## 3. Discrepancies Between Documentation and Implementation

| Item | Documented Claim | Verified Implementation | Impact |
| :--- | :--- | :--- | :--- |
| **Database Backups** | README and PRD claim database backup & restore | Only Smoooth *state metadata* is backed up via `src/security/backup.rs`. No `pg_dump` or actual database storage dumps exist. | Critical operational gap |
| **Capability Engine Integration** | Documents full cryptographic capability and PoP verification kernel | `AuthorizationEngine::authorize_capability_request` is only executed in unit tests. REST and MCP still use legacy coarse string scopes. | Security decoupling |
| **Direct Shell Calls** | Documented as unified typed adapter abstraction | `src/api.rs`, `src/build/mod.rs`, `src/ingress/bridge.rs`, and `src/migration/docker_scanner.rs` invoke raw host `tokio::process::Command::new("docker")` and `git` directly. | Privilege boundary bypass |
| **Cloud Control Plane** | Architecture documents claim hybrid cloud control plane with daemon agent | Zero cloud connectivity code exists in repository. Daemon only binds locally and has no outbound pairing client. | Feature missing |
| **Self-Approval in REST API** | Explicitly forbidden: agents must not approve own plans | `/api/workloads/deploy` creates a plan and immediately self-approves it with `plan.approve_with_ttl("operator_dashboard", 3600)` without human review. | Critical security bypass |
