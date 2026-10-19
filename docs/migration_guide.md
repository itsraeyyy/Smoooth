# Migration Guide: Moving to Smoooth

> **Effortless stack migration for developers and AI agents.**  
> *Move your workloads from Coolify, Docker Compose, Portainer, Render, Fly.io, or Dokku into Smoooth with zero downtime and automated persistent volume safeguarding.*

---

## 🌟 Overview & Core Principles

Smoooth is engineered to make platform migration frictionless. Moving from legacy PaaS platforms or standalone container orchestrators should require **little to no manual configuration**.

Smoooth achieves this through three core pillars:
1. **Intelligent Manifest Auto-Detection**: Point Smoooth at a file, a directory, or raw text—it identifies whether it is Docker Compose, Coolify JSON, Render YAML, Fly TOML, or a Dokku Procfile.
2. **Deterministic Primitives**: Containers are automatically categorized into stateful database adapters (PostgreSQL, Redis, MongoDB, MySQL) or ephemeral compute workers. Persistent volumes are safeguarded against accidental deletion.
3. **Automatic Ingress & Domain Adoption**: Host routing rules (from Traefik, Caddy, or Coolify FQDN labels) are translated directly into Smoooth's Caddy-powered reverse proxy routes with automatic TLS.

---

## 🚀 Three Migration Pathways

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                            SOURCE PLATFORMS                                 │
│  [Coolify]   [Docker Compose]   [Portainer]   [Render]   [Fly.io]  [Dokku]  │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
            ┌──────────────────────────┼──────────────────────────┐
            ▼                          ▼                          ▼
     [Terminal CLI]            [Web Dashboard]               [AI Agent MCP]
 `smoooth migrate auto`        "Migration Hub" Tab         `migrate_stack` Tool
 `smoooth migrate scan-host`   (1-Click VPS Scanner)     (Claude Code / Cursor)
            │                          │                          │
            └──────────────────────────┼──────────────────────────┘
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            SMOOOTH PLATFORM                                 │
│  • Verified 2-Phase DAG Plan                                                │
│  • Automated Ingress & ACME TLS                                             │
│  • Stateful Volume Safeguards (Postgres, Redis, Mongo)                      │
│  • Zero-Downtime Live Container Adoption                                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. Migrating from Coolify

### Method A: Live VPS Host Adoption (Recommended for Co-located VPS)
If Smoooth is installed on the same Linux VPS as Coolify:

```bash
# 1. Preview all running Coolify services, databases, and volumes
smoooth migrate coolify --scan-local

# 2. Execute 1-click migration and adopt the stack
smoooth migrate coolify --scan-local --apply
```

Smoooth inspects `/data/coolify/services/` and `/data/coolify/applications/`, resolves internal database links, maps domain routing labels, and brings all services under Smoooth governance.

### Method B: From Coolify JSON Export / Backup
If you exported a project or environment JSON file from the Coolify UI or API:

```bash
# Preview
smoooth migrate coolify --file coolify-export.json

# Apply
smoooth migrate coolify --file coolify-export.json --apply
```

---

## 2. Migrating from Docker Compose & Portainer

Smoooth supports standard Compose files (`docker-compose.yml`, `compose.yaml`):

```bash
# Dry-run inspection
smoooth migrate compose ./docker-compose.yml

# 1-Click Provisioning
smoooth migrate compose ./docker-compose.yml --apply
```

### Automatic Conversions:
- **`depends_on`**: Converted into deterministic DAG dependency edges. Databases are probed for healthy readiness before dependent web apps launch.
- **`volumes`**: Named and host volumes are detected and marked as `is_persistent: true`, protecting databases from data destruction policies.
- **Labels (`traefik.http.routers.*.rule`, `coolify.fqdn`, `caddy`)**: Extracted into native Smoooth Ingress routes with automatic HTTPS.

---

## 3. Zero-Downtime Live Host Docker Scanner

If you have containers already running on your VPS and want Smoooth to adopt management without stopping or restarting them:

```bash
# Scan running containers on host
smoooth migrate scan-host

# Adopt containers into Smoooth state
smoooth migrate scan-host --apply
```

The scanner discovers:
- Running container images and exact port bindings.
- Active volume mounts and mounted data paths.
- Environment variables and network memberships.
- Proxy and domain labels.

---

## 4. Migrating from Render, Fly.io & Dokku

### Render (`render.yaml`)
```bash
smoooth migrate render ./render.yaml --apply
```
- Converts `type: web` services into Nixpacks automated builds.
- Translates `type: psql` managed databases into guarded PostgreSQL adapters.

### Fly.io (`fly.toml`)
```bash
smoooth migrate fly ./fly.toml --apply
```
- Extracts application image, `[env]` variables, and `[http_service]` internal port.
- Maps `<app>.fly.dev` domain ingress automatically.

### Dokku / Heroku (`Procfile`)
```bash
smoooth migrate dokku ./Procfile --env-file ./.env --apply
```
- Maps `web` and `worker` processes into independent container instances.

---

## 5. Web Control Dashboard: Migration Hub

For visual operators, Smoooth includes an interactive **Migration Hub** built into the embedded dashboard:

1. Open `http://127.0.0.1:4040/dashboard` in your browser.
2. Click **Migration Hub** in the sidebar.
3. Choose your input:
   - Click **Scan Host VPS Docker Daemon** to auto-discover running stacks on the server.
   - Or paste your Compose YAML / Coolify JSON into the editor.
   - Or upload a manifest file.
4. Review the real-time **Discovered Workloads & Topology** card:
   - Workload counts and service type categorization.
   - Persistent volume guards.
   - Mapped HTTPS ingress domains.
5. Click **🚀 1-Click Migrate to Smoooth** to execute and monitor real-time terminal output.

---

## 6. AI Coding Agent Integration (MCP)

AI coding agents (Claude Code, Cursor, Windsurf) can perform migrations autonomously using registered MCP tools:

### Tool: `scan_foreign_workloads`
Discovers existing foreign containers or Coolify services on the host VPS:
```json
{
  "name": "scan_foreign_workloads",
  "arguments": {
    "workspace_id": "production"
  }
}
```

### Tool: `migrate_stack`
Imports, validates, and provisions any manifest:
```json
{
  "name": "migrate_stack",
  "arguments": {
    "source": "version: '3.8'\nservices:\n  web:\n    image: node:20\n    ports:\n      - '3000:3000'",
    "workspace_id": "production",
    "apply": true
  }
}
```

---

## 7. Verification & Post-Migration Health

After migrating, verify your workloads:

```bash
# Check status of migrated resources in workspace
smoooth status --workspace default

# Inspect ingress domain routes
smoooth ingress list

# Verify live operational telemetry & audit logs
smoooth doctor
```
