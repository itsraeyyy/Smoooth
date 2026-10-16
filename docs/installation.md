# Smoooth v2 MVP — Linux VPS Installation & Operations Guide

This guide describes how to install, configure, verify, and operate Smoooth on a customer-owned Linux VPS running a local Docker Engine.

---

## 1. Supported Distributions & Prerequisites

Smoooth is engineered to run on standard Linux distributions running systemd:

- **Ubuntu**: 20.04, 22.04, 24.04 LTS (Recommended)
- **Debian**: 11 (Bullseye), 12 (Bookworm)
- **RHEL / Rocky Linux / AlmaLinux**: 8, 9

### Prerequisites
- **Docker Engine**: Docker 24.0+ with Docker Compose and local Unix socket at `/var/run/docker.sock`.
- **System Utilities**: `curl`, `openssl`, `jq`, `tar`.
- **Hardware Minimum**: 1 vCPU, 1 GB RAM, 10 GB disk space.

---

## 2. Idempotent Installation

Smoooth provides an automated installer at `scripts/install.sh`:

```bash
# Clone the repository onto your Linux VPS
git clone https://github.com/smoooth/smoooth.git /opt/smoooth
cd /opt/smoooth

# Run the installer with sudo
sudo bash scripts/install.sh
```

### What the Installer Does:
1. **OS & Prerequisite Verification**: Checks Linux distribution, systemd, and Docker Engine.
2. **Dedicated System Account**: Creates a dedicated system user and group `smoooth` (nologin shell, home at `/var/lib/smoooth`) and adds it to the `docker` group so it can manage containers without running as root.
3. **Directory Structure & Permissions**:
   - Binary: `/usr/local/bin/smoooth` (0755, root:root)
   - Configuration & Secrets: `/etc/smoooth` (0700, smoooth:smoooth)
   - State & Local Storage: `/var/lib/smoooth` (0750, smoooth:smoooth)
4. **Cryptographic Token Provisioning**:
   - Generates a 256-bit secure hex authentication token with `openssl rand -hex 32`.
   - Saves it to `/etc/smoooth/auth.token` with restrictive `0600` permissions.
   - Retains existing tokens across updates unless `--rotate-token` is explicitly requested.
5. **Systemd Service Unit**:
   - Installs `/etc/systemd/system/smoooth.service` with strict process isolation (`NoNewPrivileges=yes`, `ProtectSystem=strict`, `ProtectHome=yes`, `PrivateTmp=yes`).
   - Enables and restarts the daemon automatically.

---

## 3. Network Security & Management API Boundary

By default, Smoooth enforces a strict zero-unauthenticated-exposure policy:

| Setting | Default Value | Description |
|---|---|---|
| Bind Address | `127.0.0.1` | Loopback only. No public ports opened. |
| Management Port | `4040` | JSON-RPC 2.0 / MCP endpoint. |
| Authentication | Bearer Token | Verified against `/etc/smoooth/auth.token`. |
| Unauthenticated Access | Blocked | All requests require valid Bearer token. |

### Remote Access over Authenticated Streamable HTTP
To connect external AI coding agents running remotely:
- **Recommended**: Connect via SSH tunnel:
  ```bash
  ssh -L 4040:127.0.0.1:4040 user@your-vps-ip
  ```
- **Reverse Proxy**: If exposing over public internet, place Smoooth behind a TLS reverse proxy (Caddy / Nginx) with HTTPS, domain certificate, and client IP allowlisting. **Never bind Smoooth directly to `0.0.0.0` without TLS termination and authentication.**

---

## 4. Token Rotation & Revocation

To rotate the operator token without disrupting persistent databases or application data:

```bash
# Rotate the authentication token and restart service
sudo bash scripts/install.sh --rotate-token

# View the newly generated token
sudo cat /etc/smoooth/auth.token
```

---

## 5. Verification & Service Operations

Check daemon status:
```bash
sudo systemctl status smoooth
```

Follow live daemon logs:
```bash
sudo journalctl -u smoooth -f
```

Verify local MCP endpoint connectivity:
```bash
TOKEN=$(sudo cat /etc/smoooth/auth.token)
curl -s -X POST http://127.0.0.1:4040/mcp \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_capabilities","arguments":{}}}' | jq .
```

---

## 6. Safe Uninstall & Data Preservation

Smoooth provides safe uninstall behavior that protects your application databases and volumes by default:

```bash
# Safely remove systemd service unit and binaries (preserves /var/lib/smoooth and all databases)
sudo bash scripts/install.sh --uninstall

# Complete purge (removes service unit, binaries, AND deletes state/databases)
sudo bash scripts/install.sh --uninstall --purge-data
```
