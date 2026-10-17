# Smoooth v0.1.0 — Beta Readiness & Distribution Specification

**Status:** Verified Beta Candidate (Release Candidate 1)  
**Author:** Smoooth Engineering  
**Baseline Environment:** Ubuntu 24.04 LTS (x86_64), Linux Kernel >= 6.8, Docker Engine >= 24.0  

---

## 1. Executive Summary & Readiness Gate

Smoooth has completed Milestone 6D and Release Candidate verification. The daemon, embedded dashboard, multi-agent credential security model, Docker orchestration engine, and database persistence layers have been compiled and verified on native Linux x86_64 with live Docker integration tests.

### Readiness Determination
- **Private Developer Beta:** **GO (APPROVED)**
- **Public `curl -fsSL https://get.smoooth.dev | sh` Distribution:** **HELD**
  *Rationale:* Until public DNS records (`get.smoooth.dev`), CDN edge distribution, and detached GPG release signatures are formally provisioned, public self-serve installation must remain gated. Controlled private beta distribution is conducted via verified checksummed tarballs (`smoooth-v0.1.0-linux-x86_64.tar.gz`).

---

## 2. Readiness Criteria & Verification Matrix

| Category | Requirement | Verification Method | Status |
| :--- | :--- | :--- | :--- |
| **Packaging** | Distributable release archive containing runtime-only binaries and scripts | `tar -czvf` packaging without web cache, temporary files, or source files | **VERIFIED** |
| **Integrity** | Cryptographic SHA-256 checksum verification | `sha256sum -c SHA256SUMS` automated self-test | **VERIFIED** |
| **CI / CD** | Automated GitHub Actions release pipeline | `.github/workflows/release.yml` with release asset uploads | **VERIFIED** |
| **Installer** | Idempotent systemd installation on Ubuntu 24.04 | `sudo bash scripts/install.sh` with dedicated `smoooth` service account | **VERIFIED** |
| **Upgrade** | Safe upgrade with automated rollback on failure | `install.sh --upgrade` with binary backup and health probe | **VERIFIED** |
| **Security** | Loopback binding by default (`127.0.0.1:4040`) | Default configuration binds to `127.0.0.1`; `--enable-remote` requires explicit opt-in | **VERIFIED** |
| **Scopes** | Server-side permission enforcement (`read_only`, `plan`, `deploy`, `admin`) | `adversarial_security_test` (12 test vectors covering routes & MCP) | **VERIFIED** |
| **Persistence** | PostgreSQL, Redis, MongoDB, and SQLite data retention across container replacement | `test_live_*` tests on live Docker Engine 29.1.3 | **VERIFIED** |
| **Dashboard** | Embedded local dashboard with CSP, offline fonts, and operation inspection | Zero external assets, strict CSP headers, step-by-step operation viewer | **VERIFIED** |
| **Observability** | Complete `/api/operations` step log history with status and error details | Real storage querying and UI modal rendering (`viewOpLogs`) | **VERIFIED** |
| **Agent Config** | Quickstart generator for Claude Code, Cursor, Windsurf, generic MCP | Interactive dashboard tab + `docs/onboarding_guide.md` | **VERIFIED** |

---

## 3. Security Boundary & Threat Analysis

### 3.1 Docker Daemon Socket (`/var/run/docker.sock`)
- **Risk:** Access to the Docker socket allows creating privileged containers with root mount access to the host filesystem.
- **Mitigation:**
  - Dedicated system account `smoooth:smoooth` with `nologin` shell.
  - The daemon is strictly confined via systemd: `ProtectSystem=strict`, `ProtectHome=yes`, `NoNewPrivileges=yes`, `PrivateTmp=yes`.
  - HTTP and MCP endpoints bind to loopback `127.0.0.1:4040` by default.
  - Unauthenticated access returns `401 Unauthorized`.
  - Master token is stored in `/etc/smoooth/auth.token` with mode `0600`.

### 3.2 Agent Token Lifecycle & Scopes
- **Token Generation:** 256-bit cryptographically secure pseudorandom numbers (OpenSSL / `rand::thread_rng`).
- **Storage:** Plaintext tokens are displayed **once** at generation time. Only salted SHA-256 hashes are persisted in `/var/lib/smoooth/smoooth_state.json`.
- **Scope Verification:**
  - `read_only`: Prohibited from mutating state or creating plans.
  - `plan`: May create and dry-run plans; prohibited from executing.
  - `deploy`: May execute approved plans; prohibited from crossing workspace boundaries.
  - `admin`: May pair new agents and revoke credentials.
- **Immediate Revocation:** Revoking an agent token invalidates both REST HTTP requests and active MCP sessions instantly.

---

## 4. Operational Playbook for Beta Testers

### 4.1 First-Time Host Preparation
```bash
# Verify Ubuntu 24.04 and Docker Engine are installed
cat /etc/os-release
docker --version

# Unpack and verify release package
tar -xzf smoooth-v0.1.0-linux-x86_64.tar.gz
cd smoooth-v0.1.0-linux-x86_64
sha256sum -c SHA256SUMS

# Run installation
sudo ./install.sh --show-token
```

### 4.2 Verifying System Health
```bash
smoooth doctor
systemctl status smoooth.service
```

### 4.3 Accessing the Embedded Dashboard
```bash
# On local workstation:
ssh -L 4040:127.0.0.1:4040 user@vps-ip

# Open in browser:
http://127.0.0.1:4040/dashboard
```
Enter the master operator token or pair a dedicated agent token from the CLI.

---

## 5. Rollback & Uninstallation Procedures

### Automatic Upgrade Rollback
If a daemon upgrade fails post-restart health checks, `install.sh --upgrade` automatically restores the backup binary and restarts the existing service.

### Manual Service Rollback
```bash
sudo cp /usr/local/bin/smoooth.bak.<timestamp> /usr/local/bin/smoooth
sudo systemctl restart smoooth.service
```

### Safe Uninstallation
```bash
# Removes daemon binary and systemd service; preserves all customer database volumes and config
sudo ./install.sh --uninstall
```

### Complete Purge
```bash
# Destructive: Purges all state and configuration
sudo ./install.sh --uninstall --purge-data
```
