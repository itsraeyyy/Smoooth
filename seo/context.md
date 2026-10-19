# Product Context: Smoooth

**URL:** https://smoooth.dev  
**Tagline:** Infrastructure runtime for AI coding agents  
**Version:** v0.1.0 Developer Beta  
**Repository:** https://github.com/itsraeyyy/Smoooth  

---

## 1. What Smoooth Is & What Problem It Solves

Smoooth is an open-source, local-first infrastructure daemon that connects AI coding agents (Claude Code, Cursor, Windsurf, OpenDevin, and MCP clients) directly to Linux servers over the Model Context Protocol (MCP).

### The Problem
AI coding agents write code in seconds, but deploying and running stateful applications still requires tedious, manual DevOps:
- Provisioning and configuring Docker containers
- Managing persistent database volumes and permissions
- Configuring reverse proxies and loopback networking
- Verifying container health probes and rolling back broken deploys

Existing PaaS solutions (Vercel, Railway, Heroku, Render) trap developers in proprietary cloud silos with high usage fees, bandwidth markups, and vendor lock-in. Giving an AI agent raw SSH root access is unsafe and error-prone.

### The Smoooth Solution
Smoooth runs as a sandboxed systemd daemon on developer-owned servers (Ubuntu 24.04, Debian 12, or local home lab) binding exclusively to `127.0.0.1:4040`. It provides:
1. **MCP Tool Suite:** Structured tools (`provision_database`, `deploy_container`, `verify_health`, `read_logs`, `rollback_release`) so agents can perform DevOps safely without raw root shells.
2. **Stateful Database Adapters:** Built-in volume retention, automated health probes, and credentials isolation for PostgreSQL, Redis, MongoDB, and SQLite.
3. **Scoped Security & Least Privilege:** 256-bit cryptographic bearer tokens with granular role-based permissions (`read_only`, `plan`, `deploy`, `admin`).
4. **Transactional 2-Phase DAG:** Validates an execution graph before provisioning containers, with instant automated rollbacks if health probes fail.
5. **Zero Cloud Lock-In:** Runs on any $5 VPS, bare-metal server, or workstation. Your code and databases stay on your own disks.

---

## 2. Target Buyer & Audience

- **Primary Persona:** Software engineers, indie hackers, and full-stack developers using AI coding tools (Claude Code CLI, Cursor IDE, Windsurf, Roo Code).
- **Secondary Persona:** DevOps engineers and technical consultants seeking a secure, auditable runtime for autonomous coding agents without giving them raw server access.
- **Conversion Goal:** 
  1. CLI installation on a Linux VPS (`sudo ./install.sh --show-token`).
  2. MCP client pairing (`claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse`).
  3. Star / Fork on GitHub (`https://github.com/itsraeyyy/Smoooth`).

---

## 3. Pricing & Limits

- **License:** Open-source (Apache-2.0 / MIT). Free forever for self-hosted instances.
- **Hardware Footprint:** Minimal memory footprint (< 45MB RAM idle daemon).
- **Prerequisites:** Linux x86_64 or ARM64 (Ubuntu 22.04/24.04, Debian 12), Docker Engine >= 24.0, systemd.

---

## 4. Competitors & Alternatives

| Competitor | Category | Why Smoooth Wins |
| :--- | :--- | :--- |
| **Coolify** | Self-hosted PaaS | Smoooth is purpose-built for AI agents over MCP with fine-grained agent scopes, not just a human Web UI. |
| **Dokku** | Git-push PaaS | Smoooth supports multi-database stateful orchestration, transactional 2-phase DAG rollbacks, and native MCP protocol. |
| **Railway / Render** | Hosted PaaS | Smoooth has zero markup, runs on your own hardware ($5 VPS vs $50+ PaaS), and has zero vendor lock-in. |
| **Raw SSH / Bash** | Direct agent access | Smoooth prevents server destruction via 256-bit scoped bearer tokens, systemd isolation, and append-only audit logging. |

---

## 5. Canonical Claims & Verified Facts

- Daemon binds to `127.0.0.1:4040` (private by default, access via SSH tunnel).
- Scopes: `read_only`, `plan`, `deploy`, `admin`.
- Supported databases: PostgreSQL, Redis, MongoDB, SQLite with persistent volume mounts.
- Idempotent one-line bash installer with automatic `smoooth doctor` environment check.
