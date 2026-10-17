# Smoooth 🚀

> **Infrastructure freedom, built for AI agents.**  
> *Deploy and operate applications on infrastructure you control.*

Smoooth allows AI coding agents (Claude Code, Cursor, Windsurf, OpenDevin, and custom MCP clients) to provision, connect, configure, and monitor stateful applications and databases on developer-owned Linux VPS infrastructure using Docker.

---

## 🏗️ Architecture

```text
Smoooth/
├── src/
│   ├── main.rs                             # CLI entrypoint (server, doctor, agent, mcp, inspect)
│   ├── api.rs                              # HTTP REST API & SSE endpoints (Axum)
│   ├── dashboard/                          # Embedded administrative web UI (CSP, offline fonts)
│   ├── engine/                             # Core orchestration & execution engine
│   │   ├── orchestrator.rs                 # 2-phase transactional execution pipeline
│   │   ├── planner.rs                      # Execution plan generation & validation
│   │   ├── dag.rs                          # Dependency graph resolution & topological sort
│   │   └── health.rs                       # Health checking & verification probes
│   ├── adapters/                           # Resource adapters
│   │   ├── docker.rs                       # Docker container lifecycle & network management
│   │   ├── postgres.rs                     # PostgreSQL database & volume provisioning
│   │   ├── redis.rs                        # Redis caching & persistent storage
│   │   ├── mongodb.rs                      # MongoDB persistent database adapter
│   │   └── sqlite.rs                       # Embedded SQLite with WAL persistence
│   ├── security/                           # Scoped access & credential authority
│   │   ├── agent.rs                        # Agent pairing, rotation, and revocation
│   │   ├── scopes.rs                       # Server-side scope enforcement (read_only, plan, deploy, admin)
│   │   └── audit.rs                        # Append-only audit logger
│   ├── storage/                            # Atomic state persistence
│   │   └── local.rs                        # File-backed local state with sync guarantees
│   └── mcp/                                # Model Context Protocol (MCP) server
│       └── tools.rs                        # Workflow tool definitions for AI agents
├── scripts/
│   └── install.sh                          # Idempotent Linux installer & lifecycle manager
├── docs/                                   # Beta documentation & operational guides
└── tests/                                  # Automated unit, integration, and security tests
```

---

## ⚡ Quickstart (Linux VPS)

### 1. Requirements
- **OS**: Ubuntu 24.04 LTS (x86_64) or Debian 12
- **Runtime**: Docker Engine >= 24.0
- **Privilege**: Root access (via `sudo`)

### 2. Installation
Unpack the verified release archive and run the installer:

```bash
# Extract release bundle
tar -xzf smoooth-v0.1.0-linux-x86_64.tar.gz
cd smoooth-v0.1.0-linux-x86_64

# Verify cryptographic SHA-256 integrity
sha256sum -c SHA256SUMS

# Run idempotent installer
sudo ./install.sh --show-token
```

### 3. Verify Health
```bash
smoooth doctor
systemctl status smoooth.service
```

---

## 🔒 Security Model

1. **Private by Default**: Binds to `127.0.0.1:4040`. Zero unauthenticated network exposure.
2. **Cryptographic Tokens**: Every API and MCP request requires a 256-bit Bearer token.
3. **Dedicated Service Account**: Runs under unprivileged user `smoooth:smoooth`.
4. **Systemd Sandboxing**: Confined via `NoNewPrivileges=yes`, `ProtectSystem=strict`, `ProtectHome=yes`, `PrivateTmp=yes`.
5. **Role-Based Scopes**:
   - `read_only`: System inspection and status monitoring.
   - `plan`: Dry-run planning and specification validation.
   - `deploy`: Deployment execution and service binding.
   - `admin`: Credential management and daemon administration.

---

## 🤖 Agent Setup

### Pair an Agent
```bash
smoooth agent pair --name "my-agent" --workspace "default" --scopes "read_only,plan,deploy"
```

### Configure Claude Code CLI
```bash
claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse \
  --header "Authorization: Bearer <AGENT_TOKEN>"
```

### Configure Cursor (`.cursor/mcp.json`)
```json
{
  "mcpServers": {
    "smoooth": {
      "url": "http://127.0.0.1:4040/sse",
      "headers": {
        "Authorization": "Bearer <AGENT_TOKEN>"
      }
    }
  }
}
```

---

## 📊 Control Dashboard

Access the embedded dashboard via local browser or SSH tunnel:

```bash
# Remote access via SSH tunnel:
ssh -L 4040:127.0.0.1:4040 user@your-vps-ip

# Open in browser:
http://127.0.0.1:4040/dashboard
```

Features:
- Live container and database status overview.
- Full step-by-step operation history and error inspection modal (`/api/operations`).
- Interactive Agent Quickstart configuration generator.
- One-click token clearance and agent credential management.
- Real-time audit log stream.

---

## 📄 Documentation

- [Beta Readiness & Distribution Specification](docs/beta_readiness.md)
- [Agent Onboarding & Deployment Scenarios Guide](docs/onboarding_guide.md)
- [Linux Installation & Operations Guide](docs/installation.md)
- [External Agent Integration Guide](docs/agent_integration.md)

---

## 📜 License

Apache-2.0 / MIT © Smoooth Project Contributors.
