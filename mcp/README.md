# MCP Content Architecture: /mcp/

This section hosts technical guides, schemas, and reference tutorials for Model Context Protocol (MCP) clients connecting to Smoooth daemon.

## Canonical Rules
- **Base Path:** `https://smoooth.dev/mcp/<slug>`
- **Page Type:** Technical Documentation / Integration Guide
- **Target Audience:** Developers using Claude Code CLI, Cursor IDE, Windsurf, Cline, OpenDevin.
- **Required Sections:**
  1. Prerequisites (VPS, Docker version, client setup).
  2. Transport configuration (`stdio` vs `streamable HTTP/SSE`).
  3. Exact configuration snippet (`claude.json` / `cursor/mcp.json`).
  4. Real tool invocation examples with exact JSON schemas.
  5. Security permissions & scope boundaries.
  6. Diagnostic & health verification checks.

## Planned Pages in Backlog
1. `/mcp/docker-deployment-claude-code` — Deploying Docker containers and PostgreSQL via Claude Code CLI.
2. `/mcp/cursor-ide-setup` — Configuring Cursor MCP client with scoped bearer tokens on remote VPS.
3. `/mcp/tool-reference` — Complete reference of all 15 Smoooth MCP tools, parameter schemas, and return types.
