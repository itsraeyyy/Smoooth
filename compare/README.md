# Comparisons Architecture: /compare/

Head-to-head architectural comparisons between Smoooth and specific developer tools.

## Canonical Rules
- **Base Path:** `https://smoooth.dev/compare/<tool-a>-vs-<tool-b>`
- **Page Type:** Versus / Head-to-Head Comparison
- **Search Intent:** Decision / Comparison Intent ("smoooth vs coolify", "dokku vs coolify for agents")
- **Standard:** Every benchmark or claim must be empirically reproducible on a standard $5/mo VPS (e.g. 1 vCPU, 2GB RAM, Ubuntu 24.04).
- **Required Sections:**
  1. Executive summary: Who wins on memory footprint, agent ergonomics, protocol support, and disaster recovery.
  2. Verified benchmark table (idle memory, cold start, build crashes, config format).
  3. Feature-by-feature deep dive with architectural reasoning.
  4. Decision matrix ("Choose Tool X if... Choose Smoooth if...").

## Planned Pages in Backlog
1. `/compare/smoooth-vs-coolify` — Head-to-head comparison: Smoooth Runtime vs Coolify v4 for AI agent infrastructure.
2. `/compare/smoooth-vs-dokku` — Head-to-head comparison: Agent-first MCP daemon vs Git-push PaaS.
