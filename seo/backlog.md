# SEO Backlog: Smoooth

Ordered by expected signups and installations per hour of work.

| Priority | Query / Opportunity | Target Page / Section | Proposed Change | Status | Expected Impact |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P0** | Deploying Docker with Claude Code | `/guides/deploy-docker-application-claude-code` | End-to-end tutorial: Claude Code + MCP + persistent PostgreSQL + Caddy TLS without raw Docker socket | **Published (Sprint 02)** | Very High (Emerging high-intent query) |
| **P0** | Infrastructure runtime for AI agents | Homepage (`https://smoooth.dev/`) | Remove decorative kickers, align H1 & SERP title (53 chars), add FAQ section with JSON-LD schema, fix mobile drawer & dual CTAs | **Ready to Deploy** | High (Homepage SERP rank & CTR) |
| **P0** | Search crawler sitemap & robots fix | `/robots.txt` & `/sitemap.xml` | Fix Cloudflare SPA HTML fallback: serve valid static plain-text robots.txt and XML sitemap without fragment URLs | **Ready to Deploy** | Critical (Search indexing & hygiene) |
| **P1** | Smoooth vs Coolify for AI agents | `/compare/smoooth-vs-coolify` | Head-to-head empirical comparison: <28MB vs 1.5GB RAM, MCP tooling vs web UI, DAG rollback guarantees | **Next Up (Sprint 03)** | High (Targeting 1.1k/mo comparison) |
| **P1** | Coolify Migration Engineering Story | `/blog/coolify-to-smoooth-migration` | Case study documenting real founder migration: 1.5GB to <28MB RAM, zero downtime cutover | **In Preparation (Sprint 03)** | High (Proof & community credibility) |
| **P1** | Self-hosted PaaS for AI agents | `/guides/self-hosted-ai-agent-vps` | Complete setup guide: $5/mo VPS, loopback sandboxing, Docker supervisor, and agent pairing | Ready for Draft | High (Targeting 950/mo intent) |
| **P2** | Cursor IDE MCP DevOps | `/mcp/cursor-ide-setup` | Tutorial configuring Cursor `.cursor/mcp.json` with scoped bearer tokens and compose imports | Backlog | Medium |
| **P2** | Docker Compose Migration Guide | `/guides/docker-compose-migration` | Playbook for adopting foreign compose stacks without downtime | Backlog | Medium |
