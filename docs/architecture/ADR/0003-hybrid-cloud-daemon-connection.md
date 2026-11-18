# ADR 0003: Hybrid Cloud Daemon Architecture & Outbound Connection Protocol

## Status
Accepted

## Context
Smoooth supports both self-hosted single-node installations and a hybrid model where a managed Smoooth Cloud control plane coordinates multiple customer-owned Linux servers.

Allowing a centralized cloud control plane to reach into customer servers by opening inbound management ports (or exposing Docker sockets over TCP) introduces severe security liabilities:
- Publicly exposed ports on customer servers are vulnerable to internet scanning, brute-forcing, and zero-day exploitation.
- Customer firewalls and NATs prevent inbound connections.
- A compromise of the cloud control plane could lead to lateral root compromise of every connected customer server.

## Decision
1. **Outbound-Only Daemon Architecture:** The local Smoooth daemon initiates an outbound, persistent authenticated connection (over secure WebSockets / mutual TLS) to the Smoooth Cloud control plane. Customer servers never expose inbound management ports to the internet.
2. **Local-First Authority & Resilience:**
   - The local daemon retains complete execution authority and local policy enforcement.
   - If cloud connectivity is lost, running customer applications are unaffected, and the local CLI remains fully functional.
   - Destructive operations require cryptographic authorization signatures; the daemon verifies permits locally and rejects expired or replayed commands.
3. **Replay & Stale Operation Protection:**
   - Every operation payload is bound to a monotonic fencing epoch and timestamp.
   - Reconnecting after an outage does not replay stale operations.

## Consequences
### Positive
- Zero inbound network exposure on customer servers.
- Works seamlessly behind NATs, firewalls, and cloud VPC security groups.
- Cloud outages cannot cause downtime for customer workloads.
- Compromising the cloud control plane does not automatically grant host root access to customer servers.

### Negative / Trade-offs
- Requires managing stateful connection pools and heartbeat keep-alives in the cloud control plane.
- Offline queued operations require careful reconciliation logic upon reconnect.
