# Smoooth v2 MVP — External AI Coding Agent Integration Guide

Smoooth provides first-class support for external AI coding agents (Claude Code, Cursor, Codex, OpenDevin, Cline, and any Model Context Protocol compliant client).

Agents interact with Smoooth through structured, typed tools rather than raw root shell access, allowing them to provision infrastructure, connect services, verify applications, and reconcile failures safely.

---

## 1. Supported MCP Transports

Smoooth supports two standard MCP transports:

1. **Stdio Transport**: Agent spawns `smoooth mcp` as a subprocess and communicates over standard input/output (`stdin`/`stdout`). Ideal when the agent runs directly on the VPS or over SSH.
2. **Streamable HTTP / SSE Transport**: Agent connects over HTTP to Smoooth's authenticated Server-Sent Events stream (`GET /sse`) and posts messages to `POST /sse/message?sessionId=...` or directly to `POST /mcp` with Bearer token authentication. Ideal when the agent runs locally or in a cloud environment connecting to a remote VPS.

---

## 2. Configuration for Common AI Coding Agents

### A. Claude Code (Anthropic CLI)

#### Stdio Mode (running on the VPS or local machine):
```bash
claude mcp add smoooth -- /usr/local/bin/smoooth mcp
```

#### Streamable HTTP Mode (over SSH tunnel or private network):
```bash
# Obtain your auth token
TOKEN=$(cat /etc/smoooth/auth.token)

# Add Smoooth MCP server to Claude Code
claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse --header "Authorization: Bearer ${TOKEN}"
```

---

### B. Cursor IDE

Configure `.cursor/mcp.json` (or global Cursor Settings → Features → MCP):

#### Option 1: Stdio Transport
```json
{
  "mcpServers": {
    "smoooth": {
      "command": "/usr/local/bin/smoooth",
      "args": ["mcp"]
    }
  }
}
```

#### Option 2: Streamable HTTP (SSE) Transport
```json
{
  "mcpServers": {
    "smoooth": {
      "url": "http://127.0.0.1:4040/sse",
      "headers": {
        "Authorization": "Bearer YOUR_256_BIT_HEX_AUTH_TOKEN"
      }
    }
  }
}
```

---

### C. Claude Desktop

In `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "smoooth": {
      "command": "smoooth",
      "args": ["mcp"]
    }
  }
}
```

---

### D. Direct HTTP JSON-RPC 2.0 (Custom Agent Scripts / curl)

Any script or custom agent runtime can invoke tools directly using standard JSON-RPC 2.0 over HTTP:

```bash
TOKEN=$(cat /etc/smoooth/auth.token)

curl -s -X POST http://127.0.0.1:4040/mcp \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "list_capabilities",
      "arguments": {}
    }
  }' | jq .
```

---

## 3. The 7-Step Golden Workflow for Agents

When an external coding agent deploys an application with Smoooth, it follows this predictable lifecycle:

### Step 01: Discover (`list_capabilities`)
Agent discovers available infrastructure capabilities and support levels:
```json
{
  "name": "list_capabilities",
  "arguments": {}
}
```
*Returns: `cap_docker_container` (compute) and `cap_postgres_db` (database).*

---

### Step 02: Plan (`create_plan`)
Agent drafts a declarative `CanonicalExecutionSpec` specifying desired containers, databases, dependencies, and network isolation:
```json
{
  "name": "create_plan",
  "arguments": {
    "workspace_id": "production_app",
    "environment": "production",
    "actions": [
      {
        "resource_name": "prod_postgres",
        "provider": "postgres",
        "service_type": "database",
        "action_type": "create",
        "config": {
          "image": "postgres:16-alpine",
          "database": "myapp_prod"
        },
        "is_persistent": true,
        "exposes_network_publicly": false
      },
      {
        "resource_name": "prod_web_app",
        "provider": "docker",
        "service_type": "compute",
        "action_type": "create",
        "config": {
          "image": "myorg/app:v1.0.0",
          "port": 8080
        },
        "is_persistent": false,
        "exposes_network_publicly": true
      }
    ]
  }
}
```
*Returns: `plan_id`, `plan_hash` (cryptographic SHA-256), `risk_tier`, `estimated_cost_usd`, and `requires_approval`.*

---

### Step 03: Govern (`approve_plan`)
Approvals are strictly bound to the exact cryptographic plan hash:
```json
{
  "name": "approve_plan",
  "arguments": {
    "plan_id": "plan_abc123",
    "plan_hash": "63f739678c18ff...",
    "approved_by": "user_or_agent"
  }
}
```
*If any detail in the plan is altered, the hash mismatches and execution is blocked.*

---

### Step 04: Provision (`execute_plan`)
Agent starts execution with a client-supplied idempotency key:
```json
{
  "name": "execute_plan",
  "arguments": {
    "plan_id": "plan_abc123",
    "idempotency_key": "idemp_client_key_001"
  }
}
```
*Returns: `operation_id`, status, and fencing lease epoch.*

---

### Step 05: Connect (`create_service_binding`)
Agent injects database connectivity into the application container without leaking raw credentials into stdout, logs, or agent conversation context:
```json
{
  "name": "create_service_binding",
  "arguments": {
    "source_resource_id": "prod_web_app",
    "target_resource_id": "prod_postgres"
  }
}
```
*Returns: `masked_connection_string` (`postgresql://user:***@host:5432/db`), AES-256-GCM encrypted secret reference, and rotation stage.*

---

### Step 06: Verify (`verify_connection`)
Agent validates that application and database are functioning together with concrete evidence (active TCP probe, HTTP readiness status, and query ping):
```json
{
  "name": "verify_connection",
  "arguments": {
    "resource_id": "prod_web_app"
  }
}
```
*Returns: `is_healthy: true`, latency in milliseconds, and verification evidence.*

---

### Step 07: Operate (`reconcile_resource`)
Agent inspects operational status and triggers safe self-healing recovery if an incident occurs:
```json
{
  "name": "reconcile_resource",
  "arguments": {
    "resource_id": "prod_web_app"
  }
}
```
*Smoooth detects container crash or drift, attempts safe restart, and confirms recovery without ever deleting persistent data.*
