# Smoooth v0.1.0 — Controlled Developer Beta (Release Candidate 1)

**Release Date:** October 17, 2026  
**Target Platform:** Linux x86_64 (Baseline: Ubuntu 24.04 LTS / Debian 12 / Docker Engine >= 24.0)

---

## Overview

Smoooth empowers AI coding agents (Claude Code, Cursor, Windsurf, OpenDevin, and custom MCP clients) to deploy, configure, and operate polyglot web applications and stateful databases on developer-controlled Linux VPS infrastructure. 

Instead of forcing developers to adapt their architecture to proprietary hosting platforms, Smoooth adapts directly to the developer's chosen stack via Docker and Model Context Protocol (MCP).

---

## Key Features & Capabilities

### 1. Embedded Administrative & Observability Dashboard
- **Local-First & Isolated:** Served on loopback (`http://127.0.0.1:4040/dashboard`) with zero external CDN dependencies.
- **Strict Content-Security-Policy (CSP):** `default-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; font-src 'self' data:;` preventing XSS and external telemetry leaks.
- **Offline System Font Fallbacks:** High-fidelity UI using native platform typography (`-apple-system`, `BlinkMacSystemFont`, `Segoe UI`, `SFMono-Regular`, `Consolas`).
- **Durable Operation Inspector:** Full `/api/operations` step-by-step history modal (`viewOpLogs`) displaying status badges, UTC timestamps, execution levels, error messages, and rollback evidence.
- **Agent Quickstart Generator:** Interactive configurator providing instant CLI commands, MCP JSON configurations, and initial kickoff prompts tailored for Claude Code, Cursor, Windsurf, and custom clients.
- **Explicit Credential Management:** Single-session bearer token input, instant session clearance, and strict one-time display of generated agent pairing credentials.

### 2. Multi-Agent Scoped Security & Least Privilege
- **Cryptographic Token Authority:** 256-bit entropy bearer tokens stored under mode `0600` at `/etc/smoooth/auth.token`.
- **Granular Scopes:** Strict server-side enforcement across four permission tiers:
  - `read_only`: System diagnostics, workspace enumeration, inspect operations.
  - `plan`: Dry-run infrastructure planning and validation without execution.
  - `deploy`: Container provisioning, environment configuration, database deployment, rolling updates.
  - `admin`: Full administrative control, agent pairing, credential revocation.
- **Auditing & Revocation:** Timestamped audit records and instant agent credential revocation across both REST and MCP transports.

### 3. Stateful Database Resource Adapters
- **PostgreSQL:** Automated volume mapping, environment-driven provisioning, connection health checks, and verified persistence across container recreation.
- **Redis:** In-memory key-value caching and persistence with configurable AOF/RDB volume persistence.
- **MongoDB:** Multi-container NoSQL database provisioning with persistent state volumes and authentication support.
- **SQLite:** Dedicated host-mounted data directory retention with verified cross-replacement state integrity.

### 4. Hardened Systemd Service & Packaging
- **Sandboxed Execution:** Dedicated unprivileged `smoooth` service account, `NoNewPrivileges=yes`, `ProtectSystem=strict`, `ProtectHome=yes`, `PrivateTmp=yes`.
- **Default Loopback Binding:** Daemon binds exclusively to `127.0.0.1:4040` by default.
- **Deterministic Lifecycle:** Fully automated, idempotent installer (`install.sh`), safe upgrades with automatic backup rollback on failure, and clean uninstallation with `--purge-data` safeguards.
- **Host Diagnostics:** Built-in `smoooth doctor` command validating Docker engine connectivity, socket permissions, disk space, and daemon status.

---

## Security Advisories

1. **Docker Host-Root Equivalence:**
   Under Docker's architecture, write access to `/var/run/docker.sock` grants effective host root access. Smoooth mitigates this risk by:
   - Requiring root-only file access (`chmod 0700` on `/etc/smoooth`, `0600` on tokens).
   - Binding exclusively to loopback `127.0.0.1:4040` unless explicitly instructed otherwise with `--enable-remote`.
   - Recommending an authenticating reverse proxy (e.g. Caddy/Nginx) with TLS if accessed over public networks.
2. **Distribution Status:**
   The automated web installer (`curl -fsSL https://get.smoooth.dev | sh`) remains in **HELD** status until custom domain DNS and TLS certificates are completed. For this Beta release, distribution is managed via verified release tarballs (`smoooth-v0.1.0-linux-x86_64.tar.gz`) accompanied by cryptographic `SHA256SUMS`.

---

## Installation & Upgrade

### Clean Installation
```bash
# 1. Download and verify release bundle
tar -xzf smoooth-v0.1.0-linux-x86_64.tar.gz
cd smoooth-v0.1.0-linux-x86_64

# 2. Verify SHA256 checksum
sha256sum -c SHA256SUMS

# 3. Run privileged installer
sudo ./install.sh --show-token
```

### Upgrade Existing Installation
```bash
# Unpack new version and run upgrade with automatic rollback protection
sudo ./install.sh --upgrade
```

### Diagnostics
```bash
smoooth doctor
systemctl status smoooth.service
journalctl -u smoooth.service -f
```
