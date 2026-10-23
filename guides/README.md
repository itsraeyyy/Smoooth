# Guides Architecture: /guides/

In-depth, operational tutorials demonstrating end-to-end workflows on developer-owned infrastructure with Smoooth.

## Canonical Rules
- **Base Path:** `https://smoooth.dev/guides/<slug>`
- **Page Type:** How-To Guide / Pillar Tutorial
- **Search Intent:** Informational & Transactional ("how to deploy nextjs on vps with ai agent")
- **Schema:** `HowTo` and `TechArticle` JSON-LD
- **Required Sections:**
  1. Outcome in the first two sentences.
  2. Architecture diagram (ASCII / clean SVG / flowchart).
  3. Real bash commands (tested on Ubuntu 24.04 LTS).
  4. Real code / adapter payloads (PostgreSQL, Redis, Caddy).
  5. Troubleshooting & common error recovery.

## Planned Pages in Backlog
1. `/guides/self-hosted-ai-agent-vps` — Running an AI-agent-operated VPS with zero public attack surface.
2. `/guides/postgresql-persistence-mcp` — Managing persistent PostgreSQL volumes and automated backups with coding agents.
3. `/guides/docker-compose-migration` — Migrating multi-container compose stacks to Smoooth with zero downtime.
