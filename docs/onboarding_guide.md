# Smoooth v0.1.0 — Agent Onboarding & Deployment Scenarios Guide

This guide details the first-run experience and end-to-end setup for AI coding agents (Claude Code, Cursor, Windsurf, OpenDevin, and custom scripts) across 6 real-world deployment topologies (Scenarios A through F).

---

## Quick Reference: Scopes & Permissions

| Scope | Allowed Operations | Typical Use Case |
| :--- | :--- | :--- |
| `read_only` | `list_capabilities`, `verify_connection`, list/inspect resources & operations | Auditing, CI status checks, monitoring agents |
| `plan` | `read_only` + `create_plan`, dry-run specification validation | Architecture review, previewing proposed diffs |
| `deploy` | `plan` + `approve_plan`, `execute_plan`, `create_service_binding`, `reconcile_resource` | Primary coding agents (Claude Code, Cursor) |
| `admin` | Full control, token rotation, credential revocation, cross-workspace ops | Host administrator, DevOps lead |

---

## Scenario A: Local Agent on the Linux VPS (Claude Code CLI)

In this scenario, Claude Code CLI runs directly on the Ubuntu Linux VPS where Smoooth is installed.

```
+---------------------------------------------------------+
| Linux VPS (Ubuntu 24.04)                                |
|                                                         |
|  [Claude Code CLI] <---> [Smoooth Daemon (127.0.0.1)]   |
|                             |                           |
|                             +---> [Docker Daemon Socket]|
+---------------------------------------------------------+
```

### 1. Pair Agent & Obtain Token
```bash
# Generate a dedicated agent token for Claude Code
smoooth agent pair --name "claude-vps" --workspace "default" --scopes "read_only,plan,deploy"
```
*Note the returned token, e.g. `smoooth_ag_9f81a7b...`.*

### 2. Configure Claude Code
You can connect Claude Code using either stdio or HTTP SSE.

**Option 1: Stdio Transport (Direct Subprocess)**
```bash
claude mcp add smoooth -- /usr/local/bin/smoooth mcp
```

**Option 2: SSE Transport**
```bash
claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse \
  --header "Authorization: Bearer <YOUR_AGENT_TOKEN>"
```

### 3. Verify Connection
Prompt Claude Code:
> "Run `list_capabilities` via Smoooth and verify what services are available."

---

## Scenario B: Remote Developer Workstation via SSH Tunnel (Cursor IDE)

In this scenario, Cursor runs on your local macOS or Windows laptop. The Smoooth daemon remains private and bound to `127.0.0.1:4040` on the VPS. Zero ports are exposed to the public internet.

```
+-------------------+           SSH Tunnel            +-----------------------------+
| Developer Laptop  | ------------------------------> | Linux VPS                   |
| [Cursor IDE]      |   (localhost:4040 -> 4040)      |  [Smoooth Daemon 127.0.0.1] |
+-------------------+                                 +-----------------------------+
```

### 1. Open SSH Tunnel
Run on your local machine:
```bash
ssh -N -L 4040:127.0.0.1:4040 user@your-vps-ip
```

### 2. Configure Cursor MCP
In your project workspace, create `.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "smoooth": {
      "url": "http://127.0.0.1:4040/sse",
      "headers": {
        "Authorization": "Bearer smoooth_ag_YOUR_TOKEN_HERE"
      }
    }
  }
}
```

### 3. Verify in Cursor
Open Cursor Composer / Chat and type:
> "Check Smoooth status and list all current workloads."

---

## Scenario C: Remote Agent via TLS Reverse Proxy (Windsurf / CI Agent)

In this scenario, an external coding agent connects to Smoooth over HTTPS. An authenticating reverse proxy (Caddy or Nginx) handles TLS termination and forwards traffic to `127.0.0.1:4040`.

```
+--------------------+           HTTPS (TLS)          +-----------------------------+
| External Agent     | -----------------------------> | VPS: Caddy / Nginx (:443)   |
| (Windsurf / CI/CD) |                                |             |               |
|                    |                                |             v (loopback)    |
+--------------------+                                | Smoooth Daemon (127.0.0.1)  |
                                                      +-----------------------------+
```

### 1. Configure Caddyfile (Example)
```caddy
smoooth.yourdomain.com {
    reverse_proxy 127.0.0.1:4040 {
        header_up Host {host}
        header_up X-Real-IP {remote}
    }
}
```

### 2. Configure Windsurf MCP Config
In `~/.codeium/windsurf/mcp_config.json`:
```json
{
  "mcpServers": {
    "smoooth": {
      "url": "https://smoooth.yourdomain.com/sse",
      "headers": {
        "Authorization": "Bearer smoooth_ag_YOUR_TOKEN_HERE"
      }
    }
  }
}
```

### 3. Security Boundary Guarantee
- Public access without the `Authorization: Bearer` header receives instant `401 Unauthorized`.
- Bearer tokens are SHA-256 hashed and verified in constant time.
- All operations are recorded in the server-side audit log.

---

## Scenario D: Multi-Agent Workspace Isolation

In this scenario, two separate agents work on the same VPS in distinct workspaces:
- Agent A: Frontend Agent (`workspace: "frontend"`)
- Agent B: Backend Agent (`workspace: "backend"`)

### 1. Pair Frontend Agent
```bash
smoooth agent pair --name "frontend-dev" --workspace "frontend" --scopes "read_only,plan,deploy"
```

### 2. Pair Backend Agent
```bash
smoooth agent pair --name "backend-dev" --workspace "backend" --scopes "read_only,plan,deploy"
```

### 3. Isolation Guarantees
- The frontend agent can only create, execute, or view plans within the `"frontend"` workspace.
- Any attempt by the frontend agent to access or mutate resources in `"backend"` is rejected with `403 Forbidden: Workspace boundary violation`.
- The dashboard groups operations and resources by workspace for clean auditing.

---

## Scenario E: Read-Only Audit & Monitoring Agent

In this scenario, a compliance or monitoring agent needs to inspect system health, verify running containers, and check for configuration drift without having permission to execute plans or change state.

### 1. Pair Read-Only Agent
```bash
smoooth agent pair --name "compliance-monitor" --workspace "default" --scopes "read_only"
```

### 2. Adversarial Security Verification
If this agent calls `execute_plan` or `approve_plan`:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32003,
    "message": "Forbidden: Token lacks required scope 'deploy' (has: ['read_only'])"
  }
}
```
The unauthorized attempt is immediately written to `/var/lib/smoooth/audit.log` and visible in the dashboard audit stream.

---

## Scenario F: Incident Recovery & Self-Healing

When a containerized service experiences an unexpected failure (e.g. out of memory, crash loop), an agent can safely diagnose and reconcile the resource without deleting persistent data.

### 1. Diagnose via Verification Tool
The agent calls `verify_connection`:
```json
{
  "name": "verify_connection",
  "arguments": {
    "resource_id": "app_backend"
  }
}
```
*Output: `is_healthy: false`, error: `Container exited with code 137 (OOMKilled)`.*

### 2. Reconcile & Recover
The agent calls `reconcile_resource`:
```json
{
  "name": "reconcile_resource",
  "arguments": {
    "resource_id": "app_backend"
  }
}
```
*Smoooth inspects the spec, cleans up the exited container while preserving state volumes, and restarts the workload.*

### 3. Confirm Recovery
The agent verifies health again:
*Output: `is_healthy: true`, response time: `12ms`.*
All database tables and mounted storage volumes remain intact.
