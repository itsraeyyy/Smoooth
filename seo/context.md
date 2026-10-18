# Smoooth SEO Context & Ground Truth

**Canonical Domain:** https://smoooth.dev  
**Product:** Self-hosted infrastructure runtime for AI coding agents deploying Docker containers, persistent databases, and secure networks on developer-controlled Linux VPS over Model Context Protocol (MCP).  
**Pricing / Distribution:** Free & Open Source under Apache-2.0 / MIT. Zero PaaS markup, zero usage meter. Runs on any $5/mo Linux VPS (Hetzner, OVH, DigitalOcean, Linode, AWS EC2).  
**Conversion Goal:** Download release binary / run curl installer (`curl -sSL https://smoooth.dev/install.sh | bash`) or clone GitHub repository (`https://github.com/smoooth-dev/smoooth`).  
**Primary ICP (Ideal Customer Profile):** Software engineers, indie hackers, and autonomous agent builders using Claude Code, Cursor, or Windsurf who need backend infrastructure (PostgreSQL with pgvector, Redis, MongoDB, SQLite, and custom Docker containers) without being locked into expensive proprietary cloud platforms.

## Core Capabilities & Technical Truths

| Capability | Source File / Implementation Detail |
| :--- | :--- |
| **Model Context Protocol (MCP)** | `src/mcp/server.rs`: Implements MCP stdio & Server-Sent Events (SSE) transports. Exposes structured tools for container and database management. |
| **Multi-Database Adapters** | `src/storage/local.rs` & `src/api.rs`: Automated provisioning for PostgreSQL (with pgvector), Redis (AOF mode), MongoDB (SCRAM-SHA-256), and SQLite (WAL mode). |
| **Scoped Security & RBAC** | `src/api.rs`: 256-bit cryptographic bearer token validation, loopback RPC (`127.0.0.1:4040`), strict CSP, zero public exposure of database ports. |
| **Transactional 2-Phase DAG** | Internal dependency graph validator ensuring databases and internal bridges are healthy before dependent web services are spawned. |
| **Systemd Host Daemon** | Unprivileged service account isolation on Linux x86_64 (Ubuntu 22.04/24.04, Debian 12). |

## Primary Competitors & Alternatives

1. **E2B (`e2b.dev`):** Cloud-hosted agent sandboxes (proprietary cloud, usage billing, not on your own VPS).
2. **Coolify (`coolify.io`):** Self-hosted PaaS with web UI (heavy footprint, human dashboard-first, not agent/MCP native).
3. **Docker Compose:** Manual YAML infrastructure tool (requires human authoring, lacks agent RBAC safety and MCP server).
4. **Railway / Render / Fly.io:** Hosted proprietary container PaaS (expensive bandwidth/database markup, cloud lock-in).
5. **Portainer:** Container management UI (general purpose, no agent MCP semantics).

## Money Keywords & Search Positioning

- **Core Money Query:** `self-hosted AI agent infrastructure` (Intent: Commercial/Software)
- **High-Intent Technical Queries:**
  - `mcp server docker deploy`
  - `deploy docker containers claude code`
  - `cursor ai agent vps deployment`
  - `autonomous coding agent runtime`
  - `self-hosted mcp database adapter`
