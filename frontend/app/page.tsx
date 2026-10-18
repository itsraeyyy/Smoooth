"use client";

import React, { useState } from "react";
import { Copy, Check, Terminal, ArrowRight, ShieldCheck, Database, Cpu, Activity, Menu, X } from "lucide-react";
import {
  AnthropicIcon,
  CursorIcon,
  WindsurfIcon,
  DockerIcon,
  UbuntuIcon,
  DebianIcon,
  PostgresIcon,
  RedisIcon,
  MongoIcon,
  SqliteIcon,
} from "../components/BrandIcons";
import { HeroRuntimeVisualizer } from "../components/HeroRuntimeVisualizer";

export default function Home() {
  const [activeDbTab, setActiveDbTab] = useState<"PostgreSQL" | "Redis" | "MongoDB" | "SQLite">("PostgreSQL");
  const [activeSecurityTab, setActiveSecurityTab] = useState<"Pairing" | "Scopes" | "Audit">("Pairing");
  const [copiedSecurity, setCopiedSecurity] = useState(false);
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  const dbSnippets = {
    PostgreSQL: `POST /api/adapters/postgres/provision
{
  "service": "primary-db",
  "volume": "/var/lib/smoooth/volumes/pgdata",
  "persistence": true,
  "extensions": ["pgvector", "uuid-ossp"],
  "health_probe": "pg_isready -U postgres"
}
STATUS: HEALTHY (verified in 1.4s, 127.0.0.1:5432 bound)`,

    Redis: `POST /api/adapters/redis/provision
{
  "service": "cache-layer",
  "mode": "AOF_PERSISTENT",
  "volume": "/var/lib/smoooth/volumes/redis",
  "maxmemory": "512mb",
  "health_probe": "redis-cli ping"
}
STATUS: HEALTHY (verified PONG, 127.0.0.1:6379 bound)`,

    MongoDB: `POST /api/adapters/mongodb/provision
{
  "service": "document-store",
  "volume": "/var/lib/smoooth/volumes/mongo",
  "auth": "SCRAM-SHA-256",
  "health_probe": "mongosh --eval 'db.adminCommand({ping:1})'"
}
STATUS: HEALTHY (verified in 2.1s, 127.0.0.1:27017 bound)`,

    SQLite: `POST /api/adapters/sqlite/provision
{
  "database": "production.db",
  "volume": "/var/lib/smoooth/volumes/sqlite",
  "journal_mode": "WAL",
  "synchronous": "NORMAL",
  "busy_timeout_ms": 5000
}
STATUS: READY (WAL enabled, 0 daemon RAM overhead)`
  };

  const securitySnippets = {
    Pairing: {
      line1: "# 1. Generate agent-scoped token",
      line2: "smoooth token create --role agent --scope \"docker:read,docker:write,db:migrate\"",
      token: "smth_live_8f9a2b7c4d1e0f3a6b8c9d2e1f4a7b0c",
      footer: "CRYPTOGRAPHIC SCOPE ENFORCEMENT",
      badge: "ACTIVE",
    },
    Scopes: {
      line1: "# 2. Strict RBAC privilege boundary",
      line2: "POLICY agent_default: ALLOW container:create, ALLOW volume:mount, DENY host:root_shell",
      token: "DENY /etc/shadow, DENY /root/.ssh, DENY iptables:flush",
      footer: "RESTRICTED AGENT SANDBOX",
      badge: "ENFORCED",
    },
    Audit: {
      line1: "# 3. Append-only tamper-evident audit log",
      line2: "2026-10-18T19:42:01Z [MCP] agent:claude tool:docker_run image:postgres:16 [ALLOWED]",
      token: "2026-10-18T19:42:02Z [SYS] health_probe check:5432 result:OK latency:14ms",
      footer: "AUDIT STREAM: 127.0.0.1:4040/audit",
      badge: "VERIFIED",
    },
  };

  const handleCopyText = (text: string, id: string) => {
    navigator.clipboard?.writeText(text);
    setCopiedCommand(id);
    setTimeout(() => setCopiedCommand(null), 2000);
  };

  return (
    <div className="min-h-screen bg-white font-sans text-[#111111] antialiased selection:bg-[#111111] selection:text-white">
      {/* ===================================================================
          GRID CONTAINER (Framed by crisp hairline borders)
          =================================================================== */}
      <div className="mx-auto max-w-[1240px] border-x border-[#E5E7EB] bg-white shadow-[0_0_50px_rgba(0,0,0,0.02)]">
        
        {/* ===================================================================
            1. NAVIGATION BAR
            =================================================================== */}
        <header className="flex h-[64px] items-center justify-between border-b border-[#E5E7EB] px-6 sm:px-8 bg-white">
          {/* Left: Brand Wordmark with Precision Pixel Prompt Glyph */}
          <a href="#home" className="flex items-center gap-2.5 text-[16px] font-semibold tracking-tight text-[#111111] hover:opacity-90 transition-opacity">
            <svg className="h-4 w-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <rect x="2" y="2" width="3" height="3" fill="#111111" />
              <rect x="6" y="5" width="4" height="3" fill="#111111" />
              <rect x="11" y="8" width="3" height="3" fill="#10B981" />
              <rect x="6" y="11" width="4" height="3" fill="#111111" />
              <rect x="2" y="14" width="3" height="3" fill="#111111" />
            </svg>
            <span className="font-headline font-semibold text-[18px] tracking-tight">Smoooth</span>
          </a>

          {/* Center: Navigation Links */}
          <nav className="hidden md:flex items-center gap-8 text-[14px] text-[#4B5563]">
            <a href="#home" className="text-[#111111] font-medium transition-colors">
              Overview
            </a>
            <a href="#features" className="hover:text-[#111111] transition-colors">
              Features
            </a>
            <a href="#how-it-works" className="hover:text-[#111111] transition-colors">
              How It Works
            </a>
            <a href="#security" className="hover:text-[#111111] transition-colors">
              Security
            </a>
            <a href="#quickstart" className="hover:text-[#111111] transition-colors">
              Quickstart
            </a>
            <a href="https://github.com/smoooth-dev/smoooth" target="_blank" rel="noopener noreferrer" className="hover:text-[#111111] transition-colors">
              Docs
            </a>
          </nav>

          {/* Right: Actions */}
          <div className="flex items-center gap-3">
            <a
              href="https://github.com/smoooth-dev/smoooth"
              target="_blank"
              rel="noopener noreferrer"
              className="hidden sm:inline-block px-3 py-1.5 text-[14px] font-medium text-[#111111] hover:text-[#4B5563] transition-colors active:scale-[0.98]"
            >
              GitHub
            </a>
            <a
              href="#quickstart"
              className="rounded-[6px] bg-[#18181B] px-3.5 py-1.5 text-[13px] font-medium text-white hover:bg-black active:scale-[0.97] transition-all shadow-xs flex items-center gap-1.5"
            >
              <span>Install v0.1.0</span>
            </a>
            {/* Mobile Hamburger Button */}
            <button
              type="button"
              onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
              className="md:hidden flex items-center justify-center p-1.5 rounded-[6px] border border-[#E5E7EB] text-[#111111] hover:bg-[#F9FAFB] active:scale-[0.97]"
              aria-label="Toggle navigation menu"
            >
              {isMobileMenuOpen ? <X className="h-4 w-4" /> : <Menu className="h-4 w-4" />}
            </button>
          </div>
        </header>

        {/* Mobile Navigation Drawer */}
        {isMobileMenuOpen && (
          <div className="md:hidden border-b border-[#E5E7EB] bg-white px-6 py-4 flex flex-col gap-3 font-medium text-[14px] text-[#111827]">
            <a
              href="#home"
              onClick={() => setIsMobileMenuOpen(false)}
              className="py-1 hover:text-[#10B981] transition-colors"
            >
              Overview
            </a>
            <a
              href="#features"
              onClick={() => setIsMobileMenuOpen(false)}
              className="py-1 hover:text-[#10B981] transition-colors"
            >
              Features
            </a>
            <a
              href="#how-it-works"
              onClick={() => setIsMobileMenuOpen(false)}
              className="py-1 hover:text-[#10B981] transition-colors"
            >
              How It Works
            </a>
            <a
              href="#security"
              onClick={() => setIsMobileMenuOpen(false)}
              className="py-1 hover:text-[#10B981] transition-colors"
            >
              Security
            </a>
            <a
              href="#quickstart"
              onClick={() => setIsMobileMenuOpen(false)}
              className="py-1 hover:text-[#10B981] transition-colors"
            >
              Quickstart
            </a>
            <a
              href="https://github.com/smoooth-dev/smoooth"
              target="_blank"
              rel="noopener noreferrer"
              className="py-1 text-[#6B7280] hover:text-[#111111] transition-colors"
            >
              Docs &amp; GitHub
            </a>
          </div>
        )}

        {/* ===================================================================
            2. HERO SECTION — ROW 1: EDITORIAL HEADLINE & VALUE PROPOSITION
            =================================================================== */}
        <section className="flex flex-col lg:flex-row border-b border-[#E5E7EB]" id="home">
          
          {/* Left Hero Column (51.67% width) */}
          <div className="w-full lg:w-[51.67%] border-b lg:border-b-0 lg:border-r border-[#E5E7EB] px-8 sm:px-12 pt-14 pb-12 flex flex-col justify-between">
            <div>
              {/* Clean Editorial Headline (Strict Anti-Slop: No Eyebrow Pill) */}
              <h1 className="font-headline text-[44px] sm:text-[54px] lg:text-[60px] font-semibold leading-[1.04] tracking-[-0.035em] text-[#111111]">
                Your agent chooses the stack. Smoooth makes it work.
              </h1>
            </div>

            {/* Action Buttons */}
            <div className="mt-8 flex flex-wrap items-center gap-3.5">
              <a
                href="#quickstart"
                className="rounded-[8px] bg-[#18181B] px-5 py-2.5 text-[14px] font-medium text-white shadow-sm hover:bg-black active:scale-[0.97] transition-all flex items-center gap-2"
              >
                <Terminal className="h-4 w-4 text-[#10B981]" />
                <span>Install on Linux</span>
              </a>
              <a
                href="#features"
                className="rounded-[8px] border border-[#E5E7EB] bg-white px-5 py-2.5 text-[14px] font-medium text-[#111827] shadow-[0_1px_2px_rgba(0,0,0,0.03)] hover:bg-[#F9FAFB] active:scale-[0.97] transition-all flex items-center gap-1.5"
              >
                <span>View Architecture</span>
                <ArrowRight className="h-3.5 w-3.5 text-[#6B7280]" />
              </a>
            </div>
          </div>

          {/* Right Hero Column (48.33% width) */}
          <div className="w-full lg:w-[48.33%] px-8 sm:px-10 pt-12 pb-12 flex flex-col justify-between bg-white">
            {/* Supporting Copy in Lexend */}
            <p className="text-[15px] leading-[1.65] text-[#4B5563] max-w-[440px]">
              Deploy and operate applications on infrastructure you control.
              Smoooth connects Claude Code, Cursor, and Windsurf directly to
              your Linux VPS over MCP—provisioning Docker containers, persistent
              databases, and secure networks on demand.
            </p>

            {/* Ecosystem Compatibility Grid with Real Brand Logos & Hover Color Reveal */}
            <div className="mt-10 lg:mt-0">
              <div className="mb-3 flex items-center justify-between">
                <span className="font-mono text-[10px] tracking-[0.1em] uppercase text-[#6B7280]">
                  COMPATIBLE AGENTS &amp; INFRASTRUCTURE
                </span>
                <span className="font-pixel text-[8.5px] text-[#10B981]">
                  VERIFIED v0.1.0
                </span>
              </div>

              {/* 6 Real Brand Logos with Industrial Resting State & Authentic Color on Hover */}
              <div className="grid grid-cols-3 sm:grid-cols-6 gap-2">
                {/* 1. Claude Code */}
                <div
                  title="Claude Code CLI — Native MCP stdio & SSE integration"
                  className="group relative flex flex-col items-center justify-center gap-1.5 rounded-[8px] border border-[#E5E7EB] bg-[#FAFAFA] py-2.5 px-1.5 transition-all duration-150 hover:border-[#D97757] hover:bg-white hover:shadow-xs active:scale-[0.97] cursor-pointer"
                >
                  <AnthropicIcon className="h-4 w-4 text-[#6B7280] transition-colors duration-150 group-hover:text-[#D97757]" />
                  <span className="font-mono text-[10.5px] font-medium text-[#4B5563] group-hover:text-[#111111] transition-colors">
                    Claude
                  </span>
                </div>

                {/* 2. Cursor */}
                <div
                  title="Cursor IDE — Rules, Background Agent, and MCP tools"
                  className="group relative flex flex-col items-center justify-center gap-1.5 rounded-[8px] border border-[#E5E7EB] bg-[#FAFAFA] py-2.5 px-1.5 transition-all duration-150 hover:border-[#111111] hover:bg-white hover:shadow-xs active:scale-[0.97] cursor-pointer"
                >
                  <CursorIcon className="h-4 w-4 text-[#6B7280] transition-colors duration-150 group-hover:text-[#111111]" />
                  <span className="font-mono text-[10.5px] font-medium text-[#4B5563] group-hover:text-[#111111] transition-colors">
                    Cursor
                  </span>
                </div>

                {/* 3. Windsurf */}
                <div
                  title="Windsurf by Codeium — Cascade Agent integration"
                  className="group relative flex flex-col items-center justify-center gap-1.5 rounded-[8px] border border-[#E5E7EB] bg-[#FAFAFA] py-2.5 px-1.5 transition-all duration-150 hover:border-[#09B6A2] hover:bg-white hover:shadow-xs active:scale-[0.97] cursor-pointer"
                >
                  <WindsurfIcon className="h-4 w-4 text-[#6B7280] transition-colors duration-150 group-hover:text-[#09B6A2]" />
                  <span className="font-mono text-[10.5px] font-medium text-[#4B5563] group-hover:text-[#111111] transition-colors">
                    Windsurf
                  </span>
                </div>

                {/* 4. Docker */}
                <div
                  title="Docker Engine — Local socket & container engine"
                  className="group relative flex flex-col items-center justify-center gap-1.5 rounded-[8px] border border-[#E5E7EB] bg-[#FAFAFA] py-2.5 px-1.5 transition-all duration-150 hover:border-[#2496ED] hover:bg-white hover:shadow-xs active:scale-[0.97] cursor-pointer"
                >
                  <DockerIcon className="h-4 w-4 text-[#6B7280] transition-colors duration-150 group-hover:text-[#2496ED]" />
                  <span className="font-mono text-[10.5px] font-medium text-[#4B5563] group-hover:text-[#111111] transition-colors">
                    Docker
                  </span>
                </div>

                {/* 5. Ubuntu */}
                <div
                  title="Ubuntu 22.04 / 24.04 LTS — First-class Linux host support"
                  className="group relative flex flex-col items-center justify-center gap-1.5 rounded-[8px] border border-[#E5E7EB] bg-[#FAFAFA] py-2.5 px-1.5 transition-all duration-150 hover:border-[#E95420] hover:bg-white hover:shadow-xs active:scale-[0.97] cursor-pointer"
                >
                  <UbuntuIcon className="h-4 w-4 text-[#6B7280] transition-colors duration-150 group-hover:text-[#E95420]" />
                  <span className="font-mono text-[10.5px] font-medium text-[#4B5563] group-hover:text-[#111111] transition-colors">
                    Ubuntu
                  </span>
                </div>

                {/* 6. Debian */}
                <div
                  title="Debian 12 Bookworm — Minimal server footprint"
                  className="group relative flex flex-col items-center justify-center gap-1.5 rounded-[8px] border border-[#E5E7EB] bg-[#FAFAFA] py-2.5 px-1.5 transition-all duration-150 hover:border-[#A80030] hover:bg-white hover:shadow-xs active:scale-[0.97] cursor-pointer"
                >
                  <DebianIcon className="h-4 w-4 text-[#6B7280] transition-colors duration-150 group-hover:text-[#A80030]" />
                  <span className="font-mono text-[10.5px] font-medium text-[#4B5563] group-hover:text-[#111111] transition-colors">
                    Debian
                  </span>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* ===================================================================
            3. HERO SECTION — ROW 2: TECHNICAL RUNTIME STAGE VISUALIZER
            Replacing generic crypto phone mockup with interactive agent runtime
            =================================================================== */}
        <HeroRuntimeVisualizer />

        {/* ===================================================================
            4. SECTION 2: AUTONOMOUS INFRASTRUCTURE RUNTIME (FEATURES)
            =================================================================== */}
        <section className="px-8 sm:px-12 py-16 bg-white" id="features">
          {/* Clean & Direct Editorial Heading Row (Anti-Slop: No Eyebrow Pill) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-end">
            <div className="lg:col-span-7">
              <h2 className="font-headline text-[36px] sm:text-[42px] font-semibold leading-[1.08] tracking-[-0.03em] text-[#111111]">
                Everything Your Agents Need<br />
                to Operate on Your VPS
              </h2>
            </div>
            <div className="lg:col-span-5">
              <p className="text-[14px] leading-[1.65] text-[#6B7280] max-w-[420px] pb-2">
                Coding agents write features in seconds, but configuring
                databases, managing persistent volumes, and checking health still
                wastes hours. Smoooth gives agents safe, structured tools to
                manage the full operational lifecycle.
              </p>
            </div>
          </div>

          {/* Two Feature Cards */}
          <div className="mt-12 grid grid-cols-1 lg:grid-cols-2 gap-8">
            
            {/* Feature Card 1: Multi-Database Adapters */}
            <div className="flex flex-col justify-between rounded-[18px] border border-[#E5E7EB] bg-white p-7 shadow-[0_2px_8px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex h-9 w-9 items-center justify-center rounded-[8px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <Database className="h-4 w-4" />
                </div>
                <h3 className="mt-4 font-headline text-[18px] font-semibold tracking-tight text-[#111111]">
                  Stateful Database &amp; Container Adapters
                </h3>
                <p className="mt-1 text-[13px] leading-[1.5] text-[#6B7280]">
                  Provision PostgreSQL, Redis, MongoDB, and SQLite with automated
                  volume retention, health verification, and clean rollbacks.
                </p>
              </div>

              {/* Code Well with Dotted Stipple Accent */}
              <div className="mt-6 rounded-[12px] border border-[#E5E7EB] bg-[#F9FAFB] stipple-pattern p-4">
                <div className="overflow-hidden rounded-[8px] border border-[#E5E7EB] bg-white shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
                  {/* Code Bar with Tabs and Brand Logos */}
                  <div className="flex h-[42px] items-center justify-between border-b border-[#E5E7EB] bg-[#FAFAFA] px-3">
                    <div className="flex items-center gap-1" role="tablist">
                      {[
                        { id: "PostgreSQL", icon: PostgresIcon, color: "text-[#336791]" },
                        { id: "Redis", icon: RedisIcon, color: "text-[#DC382D]" },
                        { id: "MongoDB", icon: MongoIcon, color: "text-[#47A248]" },
                        { id: "SQLite", icon: SqliteIcon, color: "text-[#003B57]" },
                      ].map((tab) => {
                        const Icon = tab.icon;
                        const isActive = activeDbTab === tab.id;
                        return (
                          <button
                            key={tab.id}
                            type="button"
                            onClick={() => setActiveDbTab(tab.id as any)}
                            className={`flex items-center gap-1.5 rounded px-2.5 py-1 font-mono text-[11px] transition-all ${
                              isActive
                                ? "bg-white text-[#111111] font-semibold shadow-xs border border-[#E5E7EB]"
                                : "text-[#6B7280] hover:text-[#111111] hover:bg-[#F3F4F6]"
                            }`}
                          >
                            <Icon className={`h-3 w-3 ${isActive ? tab.color : "text-[#9CA3AF]"}`} />
                            <span>{tab.id}</span>
                          </button>
                        );
                      })}
                    </div>

                    <div className="hidden sm:flex items-center gap-2">
                      <div className="inline-flex items-center gap-[2.5px]" title="Loopback RPC Active" aria-hidden="true">
                        <span className="h-1 w-1 bg-[#10B981]" />
                        <span className="h-1 w-1 bg-[#10B981]" />
                        <span className="h-1 w-1 bg-[#10B981]" />
                        <span className="h-1 w-1 bg-[#E5E7EB]" />
                      </div>
                      <span className="font-mono text-[10px] text-[#9CA3AF] tracking-tight">127.0.0.1:4040</span>
                    </div>
                  </div>

                  {/* Code Body in Geist Mono */}
                  <div className="p-4 bg-white font-mono text-[11.5px] leading-[1.65] text-[#1F2937] overflow-x-auto min-h-[160px]">
                    <pre>
                      <code>{dbSnippets[activeDbTab]}</code>
                    </pre>
                  </div>
                </div>
              </div>
            </div>

            {/* Feature Card 2: Scoped Security & Least Privilege */}
            <div className="flex flex-col justify-between rounded-[18px] border border-[#E5E7EB] bg-white p-7 shadow-[0_2px_8px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex h-9 w-9 items-center justify-center rounded-[8px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <ShieldCheck className="h-4 w-4" />
                </div>
                <h3 className="mt-4 font-headline text-[18px] font-semibold tracking-tight text-[#111111]">
                  Scoped Security &amp; Least Privilege
                </h3>
                <p className="mt-1 text-[13px] leading-[1.5] text-[#6B7280]">
                  Never grant an AI agent an unrestricted root shell. Enforce 256-bit
                  cryptographic bearer tokens with granular role-based permissions.
                </p>
              </div>

              {/* Security Console Well */}
              <div className="mt-6 rounded-[12px] border border-[#E5E7EB] bg-[#F9FAFB] stipple-pattern p-4">
                <div className="overflow-hidden rounded-[8px] border border-[#E5E7EB] bg-white shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
                  {/* Console Header */}
                  <div className="flex h-[42px] items-center justify-between border-b border-[#E5E7EB] bg-[#FAFAFA] px-3">
                    <div className="flex items-center gap-1">
                      {(["Pairing", "Scopes", "Audit"] as const).map((tab) => (
                        <button
                          key={tab}
                          type="button"
                          onClick={() => setActiveSecurityTab(tab)}
                          className={`rounded px-2.5 py-1 font-mono text-[11px] font-medium transition-all ${
                            activeSecurityTab === tab
                              ? "bg-white text-[#111111] font-semibold shadow-xs border border-[#E5E7EB]"
                              : "text-[#6B7280] hover:text-[#111111] hover:bg-[#F3F4F6]"
                          }`}
                        >
                          {tab}
                        </button>
                      ))}
                    </div>

                    <button
                      type="button"
                      onClick={() => handleCopyText(securitySnippets[activeSecurityTab].token, "security")}
                      className="flex items-center gap-1 text-[11px] font-mono text-[#6B7280] hover:text-[#111111] transition-colors"
                    >
                      {copiedCommand === "security" ? (
                        <>
                          <Check className="h-3 w-3 text-[#10B981]" />
                          <span className="text-[#10B981]">Copied</span>
                        </>
                      ) : (
                        <>
                          <Copy className="h-3 w-3" />
                          <span>Copy</span>
                        </>
                      )}
                    </button>
                  </div>

                  {/* Security Console Body */}
                  <div className="relative min-h-[160px] p-4 bg-white font-mono text-[11.5px] leading-[1.65] text-[#1F2937]">
                    <div className="text-[#9CA3AF] mb-1">
                      <span>{securitySnippets[activeSecurityTab].line1}</span>
                    </div>
                    <div className="text-[#111111] font-semibold mb-2 overflow-x-auto">
                      <code>{securitySnippets[activeSecurityTab].line2}</code>
                    </div>
                    <div className="rounded border border-[#E5E7EB] bg-[#F9FAFB] p-2 text-[#4B5563] text-[11px] break-all">
                      <code>{securitySnippets[activeSecurityTab].token}</code>
                    </div>

                    <div className="mt-3 flex items-center justify-between border-t border-[#F3F4F6] pt-2 text-[10px] text-[#6B7280]">
                      <span className="font-pixel text-[8px] tracking-wider text-[#9CA3AF]">
                        {securitySnippets[activeSecurityTab].footer}
                      </span>
                      <span className="rounded bg-[#ECFDF5] px-1.5 py-0.5 font-mono text-[9px] font-semibold text-[#065F46]">
                        {securitySnippets[activeSecurityTab].badge}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

          </div>
        </section>

        {/* ===================================================================
            5. SECTION 3: QUICKSTART / HOW IT WORKS
            =================================================================== */}
        <section className="px-8 sm:px-12 py-16 bg-[#FAFAFA] border-t border-b border-[#E5E7EB]" id="how-it-works">
          {/* Clean & Direct Heading in Lexend (Anti-Slop: No Eyebrow Pill) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-end">
            <div className="lg:col-span-7">
              <h2 className="font-headline text-[36px] sm:text-[42px] font-semibold leading-[1.08] tracking-[-0.03em] text-[#111111]">
                From Bare VPS to Agent-Operated<br />
                in Under 2 Minutes
              </h2>
            </div>
            <div className="lg:col-span-5">
              <p className="text-[14px] leading-[1.65] text-[#6B7280] max-w-[420px] pb-2">
                Install once on Ubuntu 24.04 or Debian 12. Connect any
                MCP-compatible coding tool over local loopback.
              </p>
            </div>
          </div>

          <div className="mt-12 grid grid-cols-1 md:grid-cols-3 gap-6" id="quickstart">
            {/* Step 1 */}
            <div className="flex flex-col justify-between rounded-[14px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className="font-pixel text-[12px] text-[#111111] font-bold tracking-widest">[01]</span>
                  <span className="font-mono text-[10px] font-semibold uppercase tracking-wider text-[#6B7280]">SYSTEMD DAEMON</span>
                </div>
                <h3 className="font-headline text-[17px] font-semibold tracking-tight text-[#111111] mb-2">
                  Install the Daemon
                </h3>
                <p className="text-[13px] leading-[1.6] text-[#4B5563] mb-4">
                  The idempotent installer configures an unprivileged service account,
                  systemd isolation, and a 256-bit bearer token.
                </p>
              </div>
              <div className="relative group">
                <pre className="overflow-x-auto rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] p-2.5 font-mono text-[11px] text-[#1F2937]">
                  <code>curl -sSL https://smoooth.dev/install.sh | bash</code>
                </pre>
                <button
                  type="button"
                  onClick={() => handleCopyText("curl -sSL https://smoooth.dev/install.sh | bash", "step1")}
                  className="absolute right-2 top-2 rounded bg-white border border-[#E5E7EB] px-1.5 py-0.5 text-[9px] font-mono text-[#6B7280] hover:text-[#111111] opacity-0 group-hover:opacity-100 transition-opacity"
                >
                  {copiedCommand === "step1" ? "COPIED" : "COPY"}
                </button>
              </div>
            </div>

            {/* Step 2 */}
            <div className="flex flex-col justify-between rounded-[14px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className="font-pixel text-[12px] text-[#111111] font-bold tracking-widest">[02]</span>
                  <span className="font-mono text-[10px] font-semibold uppercase tracking-wider text-[#6B7280]">MCP PROTOCOL</span>
                </div>
                <h3 className="font-headline text-[17px] font-semibold tracking-tight text-[#111111] mb-2">
                  Pair Your Coding Agent
                </h3>
                <p className="text-[13px] leading-[1.6] text-[#4B5563] mb-4">
                  Point Claude Code, Cursor, or Windsurf to Smoooth's loopback endpoint.
                  Zero unauthenticated port exposure.
                </p>
              </div>
              <div className="relative group">
                <pre className="overflow-x-auto rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] p-2.5 font-mono text-[10.5px] leading-[1.5] text-[#1F2937] break-all whitespace-pre-wrap">
                  <code>claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse</code>
                </pre>
                <button
                  type="button"
                  onClick={() => handleCopyText("claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse", "step2")}
                  className="absolute right-2 top-2 rounded bg-white border border-[#E5E7EB] px-1.5 py-0.5 text-[9px] font-mono text-[#6B7280] hover:text-[#111111] opacity-0 group-hover:opacity-100 transition-opacity"
                >
                  {copiedCommand === "step2" ? "COPIED" : "COPY"}
                </button>
              </div>
            </div>

            {/* Step 3 */}
            <div className="flex flex-col justify-between rounded-[14px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className="font-pixel text-[12px] text-[#111111] font-bold tracking-widest">[03]</span>
                  <span className="font-mono text-[10px] font-semibold uppercase tracking-wider text-[#6B7280]">AUTONOMOUS RUNTIME</span>
                </div>
                <h3 className="font-headline text-[17px] font-semibold tracking-tight text-[#111111] mb-2">
                  Prompt to Provision &amp; Deploy
                </h3>
                <p className="text-[13px] leading-[1.6] text-[#4B5563] mb-4">
                  Smoooth resolves dependencies into a topological DAG, creates persistent
                  volumes, and verifies container health probes.
                </p>
              </div>
              <div className="relative group">
                <pre className="overflow-x-auto rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] p-2.5 font-mono text-[11px] text-[#1F2937]">
                  <code>"Deploy web app with PostgreSQL and Redis on port 8000."</code>
                </pre>
                <button
                  type="button"
                  onClick={() => handleCopyText("Deploy web app with PostgreSQL and Redis on port 8000.", "step3")}
                  className="absolute right-2 top-2 rounded bg-white border border-[#E5E7EB] px-1.5 py-0.5 text-[9px] font-mono text-[#6B7280] hover:text-[#111111] opacity-0 group-hover:opacity-100 transition-opacity"
                >
                  {copiedCommand === "step3" ? "COPIED" : "COPY"}
                </button>
              </div>
            </div>
          </div>
        </section>

        {/* ===================================================================
            6. SECTION 4: OPERATIONAL GUARANTEES
            =================================================================== */}
        <section className="px-8 sm:px-12 py-16 bg-white border-b border-[#E5E7EB]" id="security">
          {/* Clean & Direct Heading in Lexend (Anti-Slop: No Eyebrow Pill) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-end">
            <div className="lg:col-span-7">
              <h2 className="font-headline text-[36px] sm:text-[42px] font-semibold leading-[1.08] tracking-[-0.03em] text-[#111111]">
                Security, Control, and<br />
                Zero Cloud Lock-In
              </h2>
            </div>
            <div className="lg:col-span-5">
              <p className="text-[14px] leading-[1.65] text-[#6B7280] max-w-[420px] pb-2">
                Smoooth was designed specifically to prevent runaway agents and
                avoid surprise cloud bills.
              </p>
            </div>
          </div>

          <div className="mt-12 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            {/* Card 1 */}
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)] transition-all hover:border-[#111111] hover:shadow-xs">
              <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111] mb-3.5">
                <svg className="h-4 w-4" viewBox="0 0 16 16" fill="currentColor">
                  <rect x="1" y="1" width="4" height="4" />
                  <rect x="11" y="1" width="4" height="4" fill="#9CA3AF" />
                  <rect x="1" y="11" width="4" height="4" fill="#9CA3AF" />
                  <rect x="11" y="11" width="4" height="4" fill="#10B981" />
                  <rect x="7" y="2" width="2" height="2" fill="#D1D5DB" />
                  <rect x="7" y="12" width="2" height="2" fill="#D1D5DB" />
                  <rect x="2" y="7" width="2" height="2" fill="#D1D5DB" />
                  <rect x="12" y="7" width="2" height="2" fill="#D1D5DB" />
                </svg>
              </div>
              <h4 className="font-headline text-[15px] font-semibold text-[#111111] mb-2">
                Zero Cloud Lock-In
              </h4>
              <p className="text-[12.5px] leading-[1.6] text-[#4B5563]">
                Runs on any $5 VPS, bare-metal server, or home lab. Your data stays on your own SSDs without proprietary PaaS pricing tiers.
              </p>
            </div>

            {/* Card 2 */}
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)] transition-all hover:border-[#111111] hover:shadow-xs">
              <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111] mb-3.5">
                <svg className="h-4 w-4" viewBox="0 0 16 16" fill="none">
                  <rect x="3" y="6" width="10" height="8" rx="1" stroke="#111111" strokeWidth="1.5" />
                  <path d="M5 6V4a3 3 0 016 0v2" stroke="#111111" strokeWidth="1.5" />
                  <circle cx="8" cy="10" r="1" fill="#10B981" />
                </svg>
              </div>
              <h4 className="font-headline text-[15px] font-semibold text-[#111111] mb-2">
                Private by Default
              </h4>
              <p className="text-[12.5px] leading-[1.6] text-[#4B5563]">
                The daemon binds exclusively to 127.0.0.1:4040. Access the embedded web dashboard securely via SSH tunnel with zero public attack surface.
              </p>
            </div>

            {/* Card 3 */}
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)] transition-all hover:border-[#111111] hover:shadow-xs">
              <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111] mb-3.5">
                <svg className="h-4 w-4" viewBox="0 0 16 16" fill="none">
                  <path d="M2 8a6 6 0 106-6" stroke="#111111" strokeWidth="1.5" strokeLinecap="round" />
                  <path d="M8 2l2 2-2 2" stroke="#10B981" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </div>
              <h4 className="font-headline text-[15px] font-semibold text-[#111111] mb-2">
                Transactional 2-Phase DAG
              </h4>
              <p className="text-[12.5px] leading-[1.6] text-[#4B5563]">
                Every deployment validates an execution plan before provisioning. If a container fails health checks, Smoooth rolls back automatically.
              </p>
            </div>

            {/* Card 4 */}
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)] transition-all hover:border-[#111111] hover:shadow-xs">
              <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111] mb-3.5">
                <svg className="h-4 w-4" viewBox="0 0 16 16" fill="none">
                  <rect x="2" y="3" width="12" height="10" rx="1" stroke="#111111" strokeWidth="1.5" />
                  <rect x="5" y="6" width="2" height="1" fill="#10B981" />
                  <rect x="9" y="6" width="2" height="1" fill="#111111" />
                  <rect x="5" y="9" width="5" height="1" fill="#9CA3AF" />
                  <rect x="6" y="13" width="4" height="2" fill="#111111" />
                </svg>
              </div>
              <h4 className="font-headline text-[15px] font-semibold text-[#111111] mb-2">
                Local-First Observability
              </h4>
              <p className="text-[12.5px] leading-[1.6] text-[#4B5563]">
                Embedded dashboard with strict Content Security Policy (CSP), offline platform typography, and append-only audit logging. Zero external telemetry.
              </p>
            </div>
          </div>
        </section>

        {/* ===================================================================
            7. SECTION 5: FINAL CALL TO ACTION (CTA)
            =================================================================== */}
        <section className="px-8 sm:px-12 py-20 bg-[#FAFAFA] text-center flex flex-col items-center justify-center">
          <h2 className="font-headline text-[38px] sm:text-[46px] font-semibold leading-[1.06] tracking-[-0.03em] text-[#111111] mb-4">
            Take Control of Your Agent Infrastructure
          </h2>
          <p className="text-[15px] leading-[1.6] text-[#4B5563] max-w-[500px] mb-8">
            Available now for Linux x86_64. Download the v0.1.0 Beta release bundle and start deploying on infrastructure you control.
          </p>
          <div className="flex flex-wrap items-center justify-center gap-4">
            <a
              href="#quickstart"
              className="rounded-[8px] bg-[#18181B] px-5 py-2.5 text-[14px] font-medium text-white shadow-sm hover:bg-black active:scale-[0.97] transition-all"
            >
              Download v0.1.0 Release
            </a>
            <a
              href="https://github.com/smoooth-dev/smoooth"
              target="_blank"
              rel="noopener noreferrer"
              className="rounded-[8px] border border-[#E5E7EB] bg-white px-5 py-2.5 text-[14px] font-medium text-[#111827] shadow-[0_1px_2px_rgba(0,0,0,0.03)] hover:bg-[#F9FAFB] active:scale-[0.97] transition-all"
            >
              Read Documentation
            </a>
          </div>
        </section>

        {/* ===================================================================
            8. FOOTER
            =================================================================== */}
        <footer className="flex flex-col sm:flex-row items-center justify-between border-t border-[#E5E7EB] bg-white px-8 sm:px-12 py-8 text-[13px] text-[#6B7280] gap-4">
          <div className="flex items-center gap-2">
            <svg className="h-3.5 w-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <rect x="2" y="2" width="3" height="3" fill="#111111" />
              <rect x="6" y="5" width="4" height="3" fill="#111111" />
              <rect x="11" y="8" width="3" height="3" fill="#10B981" />
              <rect x="6" y="11" width="4" height="3" fill="#111111" />
              <rect x="2" y="14" width="3" height="3" fill="#111111" />
            </svg>
            <span className="font-headline font-semibold text-[#111111]">Smoooth</span>
            <span className="text-[#D1D5DB]">/</span>
            <span className="text-[#6B7280]">
              Infrastructure freedom, built for AI agents. Open-source under Apache-2.0 / MIT.
            </span>
          </div>

          <div className="flex items-center gap-6">
            <a href="https://github.com/smoooth-dev/smoooth" target="_blank" rel="noopener noreferrer" className="hover:text-[#111111] transition-colors">
              Docs
            </a>
            <a href="https://github.com/smoooth-dev/smoooth" target="_blank" rel="noopener noreferrer" className="hover:text-[#111111] transition-colors">
              Release Notes
            </a>
            <a href="#security" className="hover:text-[#111111] transition-colors">
              Security Model
            </a>
            <a href="https://github.com/smoooth-dev/smoooth" target="_blank" rel="noopener noreferrer" className="hover:text-[#111111] transition-colors">
              GitHub
            </a>
          </div>
        </footer>

      </div>
    </div>
  );
}
