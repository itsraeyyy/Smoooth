# Coolify to Smoooth Migration Playbook
### Scenarios, User Personas, and Step-by-Step Architecture Guide

> **Authoritative engineering reference and user journey playbook for migrating production stacks from Coolify, Docker Compose, Portainer, Render, and Dokku into the Smoooth autonomous infrastructure runtime.**

---

## Executive Summary: Why Developers Leave Coolify

Coolify popularized self-hosted PaaS by offering an open-source alternative to Heroku and Vercel. However, as production workloads grow and AI-assisted workflows dominate modern software engineering, developers encounter structural bottlenecks:

| Dimension | Coolify | Smoooth Runtime |
| :--- | :--- | :--- |
| **Idle Memory Overhead** | **1,200 MB – 1,800 MB RAM** (PHP, Laravel Horizon, Postgres, Redis, Traefik, Docker socket proxy) | **< 28 MB RAM** (Single statically linked Rust binary, zero background runtime tax) |
| **AI Agent Native Access** | None (Web UI and basic REST API; agents must hallucinate web scraping or raw SSH root shells) | **Native Model Context Protocol (MCP)** server over SSE/stdio with cryptographic least-privilege scoping |
| **Ingress & TLS Engine** | Traefik (Complex file/label configuration, frequent certificate renewal race conditions) | **Embedded Caddy Engine** (Memory-safe, automatic Let's Encrypt / ZeroSSL ACME TLS, sub-millisecond route reloading) |
| **State & Volume Safety** | Generic Docker volumes with risk of orphan pruning | **Deterministic Database Adapters** (PostgreSQL, Redis, Mongo, MySQL) with automated volume retention guarantees |
| **Deployment Execution** | Monolithic background job worker queue prone to desyncs | **Transactional 2-Phase Engine with Automatic Rollback** (Health-checked canary before DNS/routing cutover) |
| **Zero-Downtime Host Adoption** | Requires recreating and restarting containers | **Live Host Container Scanner & In-Flight Adoption** (Takes over ingress and health checks without dropping socket connections) |

---

## User Personas & Real-World Migration Scenarios

To ensure the migration experience is effortless, intuitive, and delightful across varying workflows, we model four distinct developer personas:

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                       SMOOOTH USER PERSONA MATRIX                           │
├─────────────────────┬─────────────────────┬─────────────────────────────────┤
│ PERSONA             │ STACK PROFILE       │ PRIMARY PAIN POINT              │
├─────────────────────┼─────────────────────┼─────────────────────────────────┤
│ 1. Indie Hacker     │ Next.js 15, PG,     │ 1.5GB idle RAM kills builds on  │
│    "Alex"           │ Redis, BullMQ       │ $5/mo VPS instances (OOM killer)│
├─────────────────────┼─────────────────────┼─────────────────────────────────┤
│ 2. Digital Agency   │ 15+ Client Stacks   │ Traefik domain clashes, noisy   │
│    "Maya"           │ (WordPress, Strapi) │ neighbors, zero workspace audit │
├─────────────────────┼─────────────────────┼─────────────────────────────────┤
│ 3. AI-First Eng     │ Python FastAPI,     │ Web dashboard context switches; │
│    "Devon"          │ pgvector, Cursor    │ wants AI agent to deploy stacks │
├─────────────────────┼─────────────────────┼─────────────────────────────────┤
│ 4. DevOps Purist    │ Compose files,      │ Monolithic black boxes; desires │
│    "Sam"            │ Vaultwarden, Caddy  │ transparent JSON state & binary │
└─────────────────────┴─────────────────────┴─────────────────────────────────┘
```

---

### Scenario 1: The Indie Hacker / Bootstrapped Founder ("Alex")

#### Profile
- **Infrastructure**: A single $5–$10/month Hetzner or DigitalOcean VPS (2 vCPU, 2GB–4GB RAM).
- **Workload**: Next.js 15 (App Router, Server Actions), PostgreSQL 16 with `pgvector`, Redis for rate limiting and background queues.
- **Goal**: Ship features fast without paying $50+/month for managed database add-ons, while ensuring deployments never run out of memory.

#### Alex's Pain Points with Coolify
1. **The OOM (Out Of Memory) Trap**: Coolify itself consumes 1.4GB of RAM just idling. When Next.js triggers a production webpack/turbopack build, the Linux kernel OOM killer terminates PostgreSQL or the build process midway.
2. **Desynced Webhooks**: GitHub push webhooks periodically get stuck in Coolify's queue, requiring Alex to SSH in and restart the Coolify daemon.

#### The Smoooth Experience
Alex runs one command or uses the web dashboard:
```bash
# Preview what Smoooth detects from the Coolify service
smoooth migrate coolify /data/coolify/services/saas-prod

# 1-Click Zero-Downtime Adoption
smoooth migrate coolify /data/coolify/services/saas-prod --apply
```

#### What Happens Under the Hood:
1. **Footprint Plummets**: Smoooth replaces the 1.4GB Coolify stack with a single 26MB Rust binary. Free RAM jumps by **1.35GB**, giving Next.js builds ample headroom.
2. **Persistent Volume Safeguarding**: Alex's Postgres data volume `/var/lib/postgresql/data` is automatically identified as `is_persistent: true` with a dedicated adapter lock.
3. **Automatic TLS Cutover**: Smoooth extracts the FQDN (`app.indiesaas.com`) and generates an instantaneous Caddy ingress rule with zero certificate renewal lag.

---

### Scenario 2: The Digital Agency Lead & Freelancer ("Maya")

#### Profile
- **Infrastructure**: 2 high-spec VPS nodes running 18 client websites (WordPress + MySQL, Ghost blogs, Strapi CMS, PocketBase, and custom landing pages).
- **Goal**: High client uptime, isolated routing, painless domain SSL issuance, and multi-tenant separation.

#### Maya's Pain Points with Coolify
1. **Traefik SSL Nightmares**: Adding a new custom domain for Client B occasionally corrupts Traefik's `acme.json`, causing SSL certificate downtime for Client A.
2. **Lack of Tenant Scoping**: All services live in a flat namespace; accidental container teardown affects other clients.

#### The Smoooth Experience
Maya uses the **Smoooth Migration Hub** in the Web UI:
1. Clicks **"Live Host Scan"**.
2. Smoooth identifies all 18 active Docker containers across the VPS.
3. Maya partitions them into distinct workspaces (`workspace: client-nike`, `workspace: client-acme`).
4. Caddy routes are generated per workspace with independent ACME certificate isolation.

#### CLI Equivalent:
```bash
# Scan and partition into client workspace
smoooth migrate scan-host --adopt-live --workspace client-nike
```

---

### Scenario 3: The AI-Native Engineer ("Devon")

#### Profile
- **Tools**: Lives inside Claude Code, Cursor, or Windsurf. Writes code and system prompts concurrently.
- **Workload**: FastAPI microservices, Celery workers, PostgreSQL vector store, ChromaDB.
- **Goal**: Never leave the editor to manage infrastructure. Deployment, rollbacks, and log analysis must happen via natural conversation over MCP.

#### Devon's Pain Points with Coolify
1. **Context Switching**: Coolify requires logging into a browser dashboard, manually copying environment variables, and clicking "Redeploy".
2. **Zero Agent Integration**: AI coding assistants have no structured API or tool definitions to interact with Coolify safely.

#### The Smoooth Experience
Devon pairs Claude Code with Smoooth in 5 seconds:
```bash
claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse
```

Inside Claude Code or Cursor, Devon simply types:
> *"Hey Claude, inspect my current docker-compose.yml, migrate it into Smoooth, ensure the vector database volume is safe, and route traffic to api.agentstack.dev."*

#### MCP Execution Flow:
1. Claude calls `scan_foreign_workloads({ target_path: "./docker-compose.yml" })`.
2. Smoooth parses the file, builds a dependency DAG (`postgres` -> `redis` -> `fastapi`), and returns the migration plan.
3. Claude presents the dry-run diff to Devon and calls `migrate_stack({ manifest_content: "...", apply: true })`.
4. Smoooth starts PostgreSQL, validates database readiness via TCP ping on port 5432, mounts the persistent volume, starts FastAPI, and configures the Caddy route.
5. Claude confirms: *"Migration complete. All services healthy. Rollback checkpoint #1 created."*

---

### Scenario 4: The Homelab Enthusiast & DevOps Minimalist ("Sam")

#### Profile
- **Infrastructure**: Self-hosted home server / Proxmox VM running Vaultwarden, Pi-hole, Home Assistant, Gitea, and Plex.
- **Philosophy**: "Small is beautiful." No telemetry, no black-box background services, transparent text files.

#### Sam's Pain Points with Coolify
1. **Bloatware**: Coolify downloads dozens of supporting containers, automated telemetry, and updates that can unexpectedly alter system configurations.
2. **Hidden State**: When things break, finding where configurations are stored requires traversing complex database tables.

#### The Smoooth Experience
- Smoooth is a **single statically compiled executable** (`~25MB`).
- **Human-Readable State**: All deployed workloads, ingress rules, and adapter bindings are stored in plain JSON (`smoooth_state.json`), fully trackable in Git.
- **Air-Gapped & Offline Ready**: Runs seamlessly on a private LAN without external cloud telemetry or mandatory accounts.

```bash
# Sam migrates his entire homelab compose directory
smoooth migrate compose /opt/homelab/compose.yaml --apply
```

---

## Step-by-Step Technical Migration Guide

### Method 1: Automated CLI Migration (`smoooth migrate`)

The `smoooth migrate` CLI command provides an intelligent multi-format ingestion pipeline:

```bash
# 1. Automatic format detection (Compose, Coolify, Render, Fly, Dokku)
smoooth migrate auto ./my-service/

# 2. Inspect the migration plan without applying changes
smoooth migrate coolify /data/coolify/services/my-app

# 3. Apply the migration and spin up workloads
smoooth migrate coolify /data/coolify/services/my-app --apply

# 4. Migrate raw Docker Compose file with custom project name
smoooth migrate compose ./docker-compose.prod.yml --project my-saas --apply
```

#### Command Options
- `--apply`: Executes the plan (provisions containers, network, and proxy). Without this flag, a comprehensive dry-run preview is returned.
- `--workspace <NAME>`: Isolates migrated services within a specific security and network workspace.
- `--scan-local`: Automatically inspects standard host paths (`/data/coolify`, `/var/lib/docker/compose`).

---

### Method 2: Visual Web Dashboard ("Migration Hub")

For developers who prefer visual verification, the Smoooth dashboard provides a dedicated **Migration Hub**:

1. **Navigate to Migration Hub**: Open `http://localhost:4040/dashboard` and select the **Migrate** tab in the sidebar.
2. **Choose Source**:
   - **Host Docker Scan**: Automatically scans all running containers on the VPS and detects their ports, images, labels, and volumes.
   - **Upload / Paste Manifest**: Paste Docker Compose YAML, Coolify JSON, Render YAML, Fly TOML, or Dokku Procfile.
   - **Load Templates**: Test with pre-configured templates (e.g. Coolify Next.js + Postgres stack).
3. **Inspect the DAG Topology**:
   - The visual inspector renders the dependency execution graph.
   - Stateful services are highlighted with a green **Database Adapter** badge.
   - Ingress domains and TLS targets are clearly verified.
4. **Click "Apply Migration"**: Smoooth provisions the stack in a transactional batch with real-time health verification.

---

### Method 3: AI Agent MCP Protocol

If using an AI coding assistant (Claude Code, Cursor, Windsurf, Roo Code), migration requires zero manual syntax:

#### 1. Discover Tool Definition
```json
{
  "name": "scan_foreign_workloads",
  "description": "Inspects and auto-detects foreign container specifications (Coolify, Docker Compose, Render, Fly, Dokku) and returns an actionable Smoooth migration plan.",
  "parameters": {
    "type": "object",
    "properties": {
      "target_path": { "type": "string", "description": "Path to file, directory, or 'host' for live scanner" },
      "raw_manifest": { "type": "string", "description": "Optional inline manifest content" }
    }
  }
}
```

#### 2. Execute Migration Tool Definition
```json
{
  "name": "migrate_stack",
  "description": "Transforms and deploys foreign workloads into Smoooth with persistent volume retention and zero-downtime routing.",
  "parameters": {
    "type": "object",
    "properties": {
      "manifest_content": { "type": "string" },
      "source_format": { "type": "string", "enum": ["auto", "compose", "coolify", "render", "fly", "dokku"] },
      "apply": { "type": "boolean" },
      "workspace": { "type": "string" }
    },
    "required": ["manifest_content"]
  }
}
```

---

## Technical Specifications: How Smoooth Handles Edge Cases

### 1. Ingress & Domain Translation
- **Coolify**: Extracts `coolify.fqdn` or Traefik routing labels (`traefik.http.routers.<name>.rule=Host(\`example.com\`)`).
- **Docker Compose**: Extracts `VIRTUAL_HOST`, `CADDY_DOMAIN`, or Traefik Host SNI rules.
- **Smoooth Action**: Generates an isolated Caddy route block, requests Let's Encrypt / ZeroSSL certificates, and proxies traffic to the internal container port.

### 2. Stateful Database Detection & Volume Retention
- Workloads using database images (`postgres`, `redis`, `mysql`, `mariadb`, `mongo`, `minio`, `clickhouse`) are classified as `WorkloadType::Database`.
- Attached volumes are automatically mounted to `/var/lib/smoooth/volumes/<project>_<volume>`.
- Deleting or updating a container stack **never** destroys persistent volumes unless an explicit administrative purge token is provided.

### 3. Transactional 2-Phase Rollback
- During migration, Smoooth starts the new container alongside any existing workload.
- Health checks probe HTTP endpoints (`/health`, `/api/health`, or TCP socket ping).
- Only after **3 consecutive healthy probes** does Caddy atomically switch traffic routing to the new container.
- If health checks fail within 30 seconds, the migration is aborted and existing routing remains completely uninterrupted.

---

## Summary Checklist for Coolify Users

- [x] **Install Smoooth**: Run `curl -fsSL https://smoooth.dev/install.sh | bash`
- [x] **Dry-run Scan**: Run `smoooth migrate scan-host` to inspect live containers
- [x] **Review Plan**: Confirm database volumes and domains match your production requirements
- [x] **Apply Migration**: Run `smoooth migrate auto <path> --apply` or click **Apply** in Migration Hub
- [x] **Reclaim Memory**: Stop the Coolify background containers (`docker stop coolify-realtime coolify-db coolify-proxy`) and observe **>1.3GB of RAM instantly freed**.
