# Coolify to Smoooth Migration: Empirical Case Study Specification

This document specifies the verified facts, telemetry data, terminal logs, and system metrics required from the founder's live production migration from Coolify to Smoooth on the independent server.

In accordance with Smoooth's "Proof over decoration" standard, no numbers, logs, or metrics will be fabricated. The following data points must be provided directly from the production environment:

---

## 1. Host Infrastructure Baseline

| Field | Source / Command | Required Telemetry / Format |
| :--- | :--- | :--- |
| **Cloud / VPS Provider** | Provider invoice or host metadata | Hetzner / DigitalOcean / Vultr / Linode |
| **Server Hardware Specs** | `lscpu && free -h` | vCPU count, Total RAM (e.g., 2 GB or 4 GB), Architecture (x86_64 / ARM64) |
| **Operating System** | `cat /etc/os-release` | OS name & version (e.g., Ubuntu 24.04 LTS / Debian 12) |
| **Coolify Version** | Coolify dashboard / `docker ps` | Coolify version tag (e.g., v4.0.0-beta.340) |
| **Coolify Idle Footprint** | `docker stats --no-stream` | Table of all Coolify daemon containers (`coolify`, `coolify-db`, `coolify-redis`, `coolify-realtime`, `coolify-proxy`) showing CPU % and MEM USAGE |
| **Host System Idle Memory** | `free -m` | Total used memory with Coolify idling (typically ~1.2 GB – 1.8 GB) |

---

## 2. Workload Profile Under Migration

| Field | Description | Exact Details to Record |
| :--- | :--- | :--- |
| **Application Stack** | The real application migrated | Framework (e.g. Next.js 14, Node.js API, FastAPI, or Go), Dockerfile base |
| **Stateful Databases** | Persistent data stores | Database type and version (e.g. PostgreSQL 16), persistent volume size (GB) |
| **Ingress Configuration** | Public routing | Domains routed, Traefik TLS labels, HTTP -> HTTPS redirect rules |
| **Pain Point Trigger** | Motivation for migration | Document specific failure (e.g. OOM killer invoked during deployment on 2GB VPS; inability to control deployments programmatically via Claude Code MCP) |

---

## 3. Migration Execution Log & Telemetry

Capture the raw terminal output from the migration run:

1. **Host Workload Discovery**:
   - Command: `smoooth migrate scan-host --workspace default` (or MCP tool `scan_foreign_workloads`).
   - Log output: The discovered workloads, mapped volumes, extracted environment variables, and detected database engine.
2. **Canonical Plan Generation**:
   - The generated `CanonicalExecutionSpec` JSON showing actions, dependencies, and deterministic volume mounts.
   - Canonical SHA-256 plan hash.
3. **Cutover & Reverse Proxy Transition**:
   - Port 80/443 handover from Traefik to Caddy.
   - Measured downtime during proxy reload (expected: < 2 seconds or zero-downtime socket handover).
4. **Stateful Data Verification**:
   - Verification that database files under `/var/lib/docker/volumes/...` or migrated Smoooth paths remained intact with zero data loss.
   - Output of `verify_connection` MCP probe.

---

## 4. Post-Migration Verification & Proof

Capture these three specific commands immediately after cutover:

1. **Smoooth Daemon Doctor**:
   ```bash
   smoooth doctor
   ```
   *(Must capture RSS memory usage, Docker socket connectivity, and Caddy proxy state)*
2. **Host System Memory Comparison**:
   ```bash
   free -m
   ```
   *(Compare directly against baseline `free -m` to demonstrate exact RAM recovered)*
3. **Container Count and Status**:
   ```bash
   docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
   ```
   *(Demonstrates clean workload operation without the 5-container Coolify overhead)*

---

## 5. Artifacts to Include in Case Study

- [ ] Uncut terminal recording (ASCIIcast / SVG generated via `termtosvg` or terminal screenshot).
- [ ] Direct `free -m` before/after side-by-side terminal capture.
- [ ] The exact Claude Code session log showing the agent executing the first post-migration deployment via `execute_plan`.
