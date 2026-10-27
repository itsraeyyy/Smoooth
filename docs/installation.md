# Smoooth Beta — Linux VPS Installation, Security & Operations Guide

This guide describes how to install, verify, operate, upgrade, and secure Smoooth on a developer-owned Linux VPS running Docker.

---

## 1. Verified Baseline & Platform Support

- **Verified Target Baseline**: **Ubuntu 24.04 LTS (x86_64 / AMD64)**
- **Compatible Distributions**:
  - Ubuntu 22.04 LTS, 20.04 LTS
  - Debian 11 (Bullseye), Debian 12 (Bookworm)
  - RHEL / Rocky Linux / AlmaLinux 9
- **Experimental Architectures**: `aarch64` (ARM64) is supported for development; container image tags must match host architecture.
- **Hardware Minimums**: 1 vCPU, 1 GB RAM, 10 GB disk space (>= 1 GB free on `/var`).

---

## 2. Security Model & Privilege Disclosures

### Docker Engine Effective Host-Root Access
Smoooth manages application containers by communicating with the local Docker daemon socket (`/var/run/docker.sock`) through a dedicated system account (`smoooth`).

> [!WARNING]
> In Docker's architecture, write access to `/var/run/docker.sock` grants effective root capabilities on the host. Smoooth confines daemon operations through:
> 1. **Private Loopback Binding**: Binds to `127.0.0.1:4040` by default. Never exposed publicly without explicit opt-in.
> 2. **Cryptographic Token Authorization**: Every HTTP and MCP operation requires a valid Bearer token. Unknown callers receive zero access.
> 3. **Server-Side Scopes**: Agents are bound to specific scopes (`read_only`, `plan`, `deploy`, `destructive`, `admin`) and workspace boundaries.
> 4. **Hashed Token Storage**: Secret tokens are SHA-256 hashed before persistence. Plaintext credentials are never written to disk or logged.
> 5. **Systemd Sandboxing**: Confinement via `NoNewPrivileges=yes`, `ProtectSystem=strict`, `ProtectHome=yes`, and `PrivateTmp=yes`.

---

## 3. Installation

### Quickstart (Automated 1-Line Installer)

Install directly on your Linux VPS with zero compilation and zero CPU spike:

```bash
# 1-Line Zero-Compilation Installer (Downloads verified release binary from GitHub)
curl -fsSL https://raw.githubusercontent.com/itsraeyyy/Smoooth/main/scripts/install.sh | sudo bash -s -- --show-token
```

### Manual Release Bundle Installation

```bash
# 1. Download release bundle and SHA-256 checksum
curl -fsSLO https://github.com/itsraeyyy/Smoooth/releases/download/v0.1.1/smoooth-v0.1.1-linux-x86_64.tar.gz
curl -fsSLO https://github.com/itsraeyyy/Smoooth/releases/download/v0.1.1/SHA256SUMS

# 2. Verify cryptographic integrity
sha256sum -c SHA256SUMS

# 3. Extract and run installer
tar -xzf smoooth-v0.1.1-linux-x86_64.tar.gz
cd smoooth-v0.1.1-linux-x86_64
sudo ./install.sh --show-token
```

### What the Installer Does:
1. **Preflight Checks**: Verifies CPU architecture (`x86_64`), Linux kernel, free disk space (>= 512MB on `/var`), port 4040 availability, and Docker socket.
2. **Dedicated User Account**: Provisions system user `smoooth` (nologin shell) and adds it to the `docker` group.
3. **Directory Confinement**:
   - Binary: `/usr/local/bin/smoooth` (0755)
   - Configuration & Master Token: `/etc/smoooth` (0700, `smoooth:smoooth`)
   - State & Local Storage: `/var/lib/smoooth` (0750, `smoooth:smoooth`)
4. **Token Generation**: Generates 256-bit cryptographically secure master token at `/etc/smoooth/auth.token` (0600).
5. **Systemd Confinement**: Configures, enables, and starts `/etc/systemd/system/smoooth.service`.
6. **Diagnostics**: Automatically executes `smoooth doctor` to verify post-install health.

### Manual Installation Path

If you prefer configuring system components manually:

```bash
# 1. Compile release binary
cargo build --release
sudo install -m 0755 target/release/smoooth /usr/local/bin/smoooth

# 2. Provision dedicated account
sudo groupadd --system smoooth
sudo useradd --system --gid smoooth --home-dir /var/lib/smoooth --shell /usr/sbin/nologin smoooth
sudo usermod -aG docker smoooth

# 3. Create directories with strict permissions
sudo mkdir -p /etc/smoooth /var/lib/smoooth
sudo chmod 0700 /etc/smoooth && sudo chown smoooth:smoooth /etc/smoooth
sudo chmod 0750 /var/lib/smoooth && sudo chown smoooth:smoooth /var/lib/smoooth

# 4. Generate master token
openssl rand -hex 32 | sudo tee /etc/smoooth/auth.token > /dev/null
sudo chmod 0600 /etc/smoooth/auth.token && sudo chown smoooth:smoooth /etc/smoooth/auth.token

# 5. Run daemon
sudo -u smoooth /usr/local/bin/smoooth server --bind 127.0.0.1 --port 4040
```

---

## 4. Diagnostics (`smoooth doctor`)

Run the diagnostic utility anytime to audit the environment:

```bash
smoooth doctor
```

Output:
```
======================================================================
  Smoooth Doctor Diagnostics (v0.1.0)
======================================================================
  Overall Status: HEALTHY
  Checks: 6 passed, 0 warnings, 0 failures
----------------------------------------------------------------------
[PASS] Platform: CPU Architecture
       Verified architecture: x86_64 (AMD64 / x86_64 verified baseline)
[PASS] Platform: Operating System
       Ubuntu 24.04 LTS confirmed (Target Beta Baseline).
[PASS] Container Runtime: Docker Engine Daemon
       Docker daemon connection active and responsive via local socket.
[PASS] Security Disclosure: Docker Effective Host-Root Access
       Access to Docker daemon grants effective host-root permissions.
[PASS] Networking: Daemon Port Availability
       Port 4040 is free and available on 127.0.0.1
[PASS] Storage: Storage Directory Write Access
       State directory '/var/lib/smoooth' is writable.
[PASS] Security: Token File Permissions
       Auth token file permissions verified as 0600 (owner-only).
======================================================================
```

---

## 5. Agent Access, Pairing & Credential Management

Smoooth issues distinct, revocable tokens for every AI coding agent (e.g., Cursor, Claude Code, CI workers).

### Pair an Agent
```bash
# Pair an agent with specific scopes bound to workspace 'default'
smoooth agent pair --name "cursor-agent" --workspace "default" --scopes "read_only,plan,deploy"
```
Output:
```
======================================================================
  Agent Paired Successfully
======================================================================
  Agent ID:     agent_4d17c76891
  Name:         cursor-agent
  Workspace:    default
  Scopes:       read_only, plan, deploy
  Expires:      Never
----------------------------------------------------------------------
  ONE-TIME TOKEN (Secret):
  smoooth_ag_a9c8f03e4b1d765432...
  
  Copy this token now. It cannot be retrieved again in plaintext.
======================================================================
```

### Available Scopes
- `read_only`: Inspect containers, resources, logs, and drift status.
- `plan`: Generate canonical execution plans and preview changes.
- `deploy`: Execute approved deployment plans.
- `destructive`: Terminate resources or delete persistent volumes.
- `admin`: Full unrestricted management across all workspaces.

### List Active Agents
```bash
smoooth agent list
```

### Rotate Agent Token
```bash
smoooth agent rotate --id agent_4d17c76891
```

### Revoke Agent Access
```bash
smoooth agent revoke --id agent_4d17c76891
```

---

## 6. Stack Freedom & Resource Modes

Smoooth supports three explicit resource modes:

### 1. Managed Resources
Smoooth manages provisioning, lifecycle, readiness gating, and persistent storage:
- **PostgreSQL**: `postgres:16-alpine` with automated DB initialization and `/var/lib/postgresql/data` persistence.
- **Redis**: `redis:7-alpine` with in-memory caching and optional append-only file persistence.
- **MongoDB**: `mongo:7-jammy` with root credentials and persistent volume at `/data/db`.

### 2. Connected External Resources
For databases managed by a cloud provider (e.g. AWS RDS, DigitalOcean Managed DB) or existing infrastructure:
- Ownership is set to `ResourceOwnership::External`.
- **CRITICAL GUARANTEE**: Smoooth will **never** stop, overwrite, or delete external resources during plan execution, cleanup, or teardown.

### 3. Custom Workloads & Embedded SQLite
For containerized applications utilizing embedded databases:
- **SQLite Pattern**: Mount application state to a persistent container volume (e.g. `app_data:/data`) and configure connection string `sqlite:///data/app.db`.
- **Concurrency Guidance**: Use SQLite WAL mode (`PRAGMA journal_mode=WAL;`) and a busy timeout (`PRAGMA busy_timeout=5000;`) to avoid lock contention in containerized deployments.

---

## 7. Control Dashboard

The Smoooth Control Dashboard is embedded directly into the daemon binary with zero Node.js runtime overhead:

- **Local URL**: `http://127.0.0.1:4040/dashboard`
- **Features**:
  - Overview of running, degraded, stopped containers.
  - Live status distribution capsule bar and capacity tick indicators.
  - Applications and resources view with managed vs external distinctions.
  - Agent pairing, scope inspection, and one-click token revocation.
  - Real-time audit log stream.
  - Interactive Smoooth Doctor diagnostic runner.

To access the dashboard from a remote workstation:
```bash
ssh -L 4040:127.0.0.1:4040 user@your-vps-ip
# Open http://localhost:4040/dashboard in your local browser
```

---

## 8. Safe Upgrade, Rollback & Uninstall

### Upgrading Smoooth
Smoooth supports atomic upgrades with automated rollback:

```bash
# Pull latest repository changes or build new binary
git pull
cargo build --release

# Run upgrade (creates binary backup, restarts service, validates health, rolls back on failure)
sudo bash scripts/install.sh --upgrade
```

### Safe Uninstall (Preserves Application Data)
```bash
# Removes systemd service and binaries; preserves /var/lib/smoooth and all databases
sudo bash scripts/install.sh --uninstall
```

### Full Purge (Destructive)
```bash
# Removes binaries, service, AND deletes all application state and volumes
sudo bash scripts/install.sh --uninstall --purge-data
```
