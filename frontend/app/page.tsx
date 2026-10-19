"use client";

import React, { useState } from "react";
import { Copy, Check, Terminal, ArrowRight, ShieldCheck, Database, Cpu, Activity, Menu, X, Lock, CheckCircle2, XCircle } from "lucide-react";
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
  const [activeSecurityTab, setActiveSecurityTab] = useState<"Permissions" | "Pairing" | "Audit">("Permissions");
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null);
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  const dbDetails = {
    PostgreSQL: {
      adapter: "PostgreSQL 16",
      port: "127.0.0.1:5432",
      volume: "/var/lib/smoooth/volumes/pgdata",
      probe: "pg_isready -U postgres (exit 0)",
      latency: "1.4s",
      icon: PostgresIcon,
      color: "text-[#336791]",
      lines: [
        { num: 1, text: 'POST /api/adapters/postgres/provision' },
        { num: 2, text: '{' },
        { num: 3, text: '  "service": "primary-db",' },
        { num: 4, text: '  "volume": "/var/lib/smoooth/volumes/pgdata",' },
        { num: 5, text: '  "persistence": true,' },
        { num: 6, text: '  "extensions": ["pgvector", "uuid-ossp"],' },
        { num: 7, text: '  "health_probe": "pg_isready -U postgres"' },
        { num: 8, text: '}' },
      ],
      status: "HEALTHY (verified in 1.4s, 127.0.0.1:5432 bound)",
    },
    Redis: {
      adapter: "Redis 7.2",
      port: "127.0.0.1:6379",
      volume: "/var/lib/smoooth/volumes/redis",
      probe: "redis-cli ping -> PONG (exit 0)",
      latency: "0.4s",
      icon: RedisIcon,
      color: "text-[#DC382D]",
      lines: [
        { num: 1, text: 'POST /api/adapters/redis/provision' },
        { num: 2, text: '{' },
        { num: 3, text: '  "service": "cache-layer",' },
        { num: 4, text: '  "mode": "AOF_PERSISTENT",' },
        { num: 5, text: '  "volume": "/var/lib/smoooth/volumes/redis",' },
        { num: 6, text: '  "maxmemory": "512mb",' },
        { num: 7, text: '  "health_probe": "redis-cli ping"' },
        { num: 8, text: '}' },
      ],
      status: "HEALTHY (verified PONG, 127.0.0.1:6379 bound)",
    },
    MongoDB: {
      adapter: "MongoDB 7.0",
      port: "127.0.0.1:27017",
      volume: "/var/lib/smoooth/volumes/mongo",
      probe: "mongosh --eval 'db.adminCommand({ping:1})'",
      latency: "2.1s",
      icon: MongoIcon,
      color: "text-[#47A248]",
      lines: [
        { num: 1, text: 'POST /api/adapters/mongodb/provision' },
        { num: 2, text: '{' },
        { num: 3, text: '  "service": "document-store",' },
        { num: 4, text: '  "volume": "/var/lib/smoooth/volumes/mongo",' },
        { num: 5, text: '  "auth": "SCRAM-SHA-256",' },
        { num: 6, text: '  "health_probe": "mongosh ping"' },
        { num: 7, text: '}' },
      ],
      status: "HEALTHY (verified in 2.1s, 127.0.0.1:27017 bound)",
    },
    SQLite: {
      adapter: "SQLite (WAL)",
      port: "Local Socket",
      volume: "/var/lib/smoooth/volumes/sqlite/app.db",
      probe: "sqlite3 app.db 'PRAGMA integrity_check;'",
      latency: "0.1s",
      icon: SqliteIcon,
      color: "text-[#003B57]",
      lines: [
        { num: 1, text: 'POST /api/adapters/sqlite/provision' },
        { num: 2, text: '{' },
        { num: 3, text: '  "database": "production.db",' },
        { num: 4, text: '  "volume": "/var/lib/smoooth/volumes/sqlite",' },
        { num: 5, text: '  "journal_mode": "WAL",' },
        { num: 6, text: '  "synchronous": "NORMAL"' },
        { num: 7, text: '}' },
      ],
      status: "READY (WAL enabled, 0 daemon RAM overhead)",
    },
  };

  const handleCopyText = (text: string, id: string) => {
    navigator.clipboard?.writeText(text);
    setCopiedCommand(id);
    setTimeout(() => setCopiedCommand(null), 2000);
  };

  return (
    <div className="min-h-screen bg-white font-sans text-[#111111] antialiased selection:bg-[#111111] selection:text-white">
      {/* 
        =======================================================================
        GRID CONTAINER (Framed by crisp hairline borders #E5E7EB)
        ======================================================================= 
      */}
      <div className="mx-auto max-w-[1240px] border-x border-[#E5E7EB] bg-white shadow-[0_0_50px_rgba(0,0,0,0.02)]">
        
        {/* ===================================================================
            1. NAVIGATION BAR
            =================================================================== */}
        <header className="flex h-[60px] items-center justify-between border-b border-[#E5E7EB] px-6 sm:px-8 bg-white sticky top-0 z-50">
          {/* Left: Brand Wordmark with Precision Pixel Prompt Glyph */}
          <a href="#home" className="flex items-center gap-2.5 text-[16px] font-semibold tracking-tight text-[#111111] hover:opacity-90 transition-opacity">
            <svg className="h-4 w-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <rect x="2" y="2" width="3" height="3" fill="#111111" />
              <rect x="6" y="5" width="4" height="3" fill="#111111" />
              <rect x="11" y="8" width="3" height="3" fill="#10B981" />
              <rect x="6" y="11" width="4" height="3" fill="#111111" />
              <rect x="2" y="14" width="3" height="3" fill="#111111" />
            </svg>
            <span className="font-headline font-semibold text-[17px] tracking-tight">Smoooth</span>
          </a>

          {/* Center: Navigation Links */}
          <nav className="hidden md:flex items-center gap-8 text-[13.5px] text-[#4B5563]">
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
              className="hidden sm:inline-block px-3 py-1.5 text-[13px] font-medium text-[#4B5563] hover:text-[#111111] transition-colors active:scale-[0.98]"
            >
              GitHub
            </a>
            <a
              href="#quickstart"
              className="rounded-[6px] bg-[#18181B] px-3.5 py-1.5 text-[12.5px] font-medium text-white hover:bg-black active:scale-[0.97] transition-all shadow-xs flex items-center gap-1.5"
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
            <a href="#home" onClick={() => setIsMobileMenuOpen(false)} className="py-1 hover:text-[#10B981] transition-colors">Overview</a>
            <a href="#features" onClick={() => setIsMobileMenuOpen(false)} className="py-1 hover:text-[#10B981] transition-colors">Features</a>
            <a href="#how-it-works" onClick={() => setIsMobileMenuOpen(false)} className="py-1 hover:text-[#10B981] transition-colors">How It Works</a>
            <a href="#security" onClick={() => setIsMobileMenuOpen(false)} className="py-1 hover:text-[#10B981] transition-colors">Security</a>
            <a href="#quickstart" onClick={() => setIsMobileMenuOpen(false)} className="py-1 hover:text-[#10B981] transition-colors">Quickstart</a>
            <a href="https://github.com/smoooth-dev/smoooth" target="_blank" rel="noopener noreferrer" className="py-1 text-[#6B7280] hover:text-[#111111] transition-colors">Docs &amp; GitHub</a>
          </div>
        )}

        {/* ===================================================================
            2. HERO SECTION — ROW 1: EDITORIAL HEADLINE & VALUE PROPOSITION
            Harmonious layout without harsh vertical splits
            =================================================================== */}
        <section className="border-b border-[#E5E7EB] px-8 sm:px-12 pt-14 pb-12 bg-white" id="home">
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
            
            {/* Left Hero Column: Headline & Action Controls */}
            <div className="lg:col-span-7">
              {/* Clean Editorial Headline (Strict Anti-Slop: No Eyebrow Pill) */}
              <h1 className="font-headline text-[42px] sm:text-[52px] lg:text-[56px] font-semibold leading-[1.05] tracking-[-0.035em] text-[#111111]">
                Your agent chooses the stack.<br />
                Smoooth makes it work.
              </h1>

              {/* Action Row: Install on Linux & View Architecture */}
              <div className="mt-8 flex flex-wrap items-center gap-3.5">
                <a
                  href="#quickstart"
                  className="rounded-[8px] bg-[#18181B] px-5 py-2.5 text-[14px] font-medium text-white shadow-xs hover:bg-black active:scale-[0.97] transition-all flex items-center gap-2"
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

            {/* Right Hero Column: Value Proposition & Ecosystem Dock */}
            <div className="lg:col-span-5 flex flex-col justify-between pt-1">
              <p className="text-[15px] leading-[1.65] text-[#4B5563]">
                Deploy and operate applications on infrastructure you control.
                Smoooth connects Claude Code, Cursor, and Windsurf directly to
                your Linux VPS over MCP—provisioning Docker containers, persistent
                databases, and secure networks on demand.
              </p>

              {/* Compatible Agents & Infrastructure Strip */}
              <div className="mt-8 pt-6 border-t border-[#F3F4F6]">
                <div className="mb-3 flex items-center justify-between">
                  <span className="font-mono text-[10px] tracking-[0.1em] uppercase text-[#6B7280]">
                    COMPATIBLE AGENTS &amp; INFRASTRUCTURE
                  </span>
                  <span className="font-pixel text-[8.5px] text-[#10B981]">
                    ● VERIFIED v0.1.0
                  </span>
                </div>

                {/* 6 Real Brand Logos with Slate Resting State & Authentic Color on Hover */}
                <div className="grid grid-cols-6 gap-2">
                  {[
                    { id: "claude", name: "Claude", icon: AnthropicIcon, hoverBorder: "hover:border-[#D97757]", hoverText: "group-hover:text-[#D97757]", tooltip: "Claude Code CLI — Native MCP stdio & SSE" },
                    { id: "cursor", name: "Cursor", icon: CursorIcon, hoverBorder: "hover:border-[#111111]", hoverText: "group-hover:text-[#111111]", tooltip: "Cursor IDE — Rules & Background Agent" },
                    { id: "windsurf", name: "Windsurf", icon: WindsurfIcon, hoverBorder: "hover:border-[#09B6A2]", hoverText: "group-hover:text-[#09B6A2]", tooltip: "Windsurf — Cascade Agent integration" },
                    { id: "docker", name: "Docker", icon: DockerIcon, hoverBorder: "hover:border-[#2496ED]", hoverText: "group-hover:text-[#2496ED]", tooltip: "Docker Engine — Container lifecycle" },
                    { id: "ubuntu", name: "Ubuntu", icon: UbuntuIcon, hoverBorder: "hover:border-[#E95420]", hoverText: "group-hover:text-[#E95420]", tooltip: "Ubuntu 22.04 / 24.04 LTS host support" },
                    { id: "debian", name: "Debian", icon: DebianIcon, hoverBorder: "hover:border-[#A80030]", hoverText: "group-hover:text-[#A80030]", tooltip: "Debian 12 Bookworm — Minimal footprint" },
                  ].map((brand) => {
                    const Icon = brand.icon;
                    return (
                      <div
                        key={brand.id}
                        title={brand.tooltip}
                        className={`group flex flex-col items-center justify-center gap-1.5 rounded-[6px] border border-[#E5E7EB] bg-[#FAFAFA] py-2 px-1 transition-all duration-150 ${brand.hoverBorder} hover:bg-white hover:shadow-2xs active:scale-[0.97] cursor-pointer`}
                      >
                        <Icon className={`h-3.5 w-3.5 text-[#6B7280] transition-colors duration-150 ${brand.hoverText}`} />
                        <span className="font-mono text-[9.5px] font-medium text-[#4B5563] group-hover:text-[#111111] transition-colors">
                          {brand.name}
                        </span>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>

          </div>
        </section>

        {/* ===================================================================
            3. HERO SECTION — ROW 2: TECHNICAL RUNTIME STAGE VISUALIZER
            Seamless mountain backdrop, floating prompt bubble, code editor,
            and live container topology studio
            =================================================================== */}
        <HeroRuntimeVisualizer />

        {/* ===================================================================
            4. SECTION 2: AUTONOMOUS INFRASTRUCTURE RUNTIME (FEATURES)
            Two Master Workbenches: Stateful DB Studio & Scoped Security Matrix
            =================================================================== */}
        <section className="px-8 sm:px-12 py-16 bg-white" id="features">
          {/* Editorial Heading Row (Strict Anti-Slop: No Eyebrow Pill) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-end">
            <div className="lg:col-span-7">
              <h2 className="font-headline text-[34px] sm:text-[40px] font-semibold leading-[1.08] tracking-[-0.03em] text-[#111111]">
                Everything Your Agents Need<br />
                to Operate on Your VPS
              </h2>
            </div>
            <div className="lg:col-span-5">
              <p className="text-[14px] leading-[1.65] text-[#6B7280] max-w-[420px] pb-1">
                Coding agents write features in seconds, but configuring
                databases, managing persistent volumes, and checking health still
                wastes hours. Smoooth gives agents safe, structured tools to
                manage the full operational lifecycle.
              </p>
            </div>
          </div>

          {/* Two Feature Workbenches */}
          <div className="mt-12 grid grid-cols-1 lg:grid-cols-2 gap-8">
            
            {/* Feature Card 1: Stateful Database & Container Adapters */}
            <div className="flex flex-col justify-between rounded-[16px] border border-[#E5E7EB] bg-white p-6 sm:p-7 shadow-[0_2px_12px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex h-9 w-9 items-center justify-center rounded-[8px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <Database className="h-4 w-4" />
                </div>
                <h3 className="mt-4 font-headline text-[18px] font-semibold tracking-tight text-[#111111]">
                  Stateful Database &amp; Container Adapters
                </h3>
                <p className="mt-1 text-[13px] leading-[1.55] text-[#6B7280]">
                  Provision PostgreSQL, Redis, MongoDB, and SQLite with automated
                  volume retention, health verification, and clean rollbacks.
                </p>
              </div>

              {/* Elevated Database Workbench */}
              <div className="mt-6 rounded-[12px] border border-[#E5E7EB] bg-[#F9FAFB] stipple-pattern p-3 sm:p-4">
                <div className="overflow-hidden rounded-[8px] border border-[#E5E7EB] bg-white shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
                  
                  {/* Database Adapter Tabs with Authentic Brand Icons */}
                  <div className="flex h-[42px] items-center justify-between border-b border-[#E5E7EB] bg-[#FAFAFA] px-3">
                    <div className="flex items-center gap-1" role="tablist">
                      {(["PostgreSQL", "Redis", "MongoDB", "SQLite"] as const).map((tab) => {
                        const detail = dbDetails[tab];
                        const Icon = detail.icon;
                        const isActive = activeDbTab === tab;
                        return (
                          <button
                            key={tab}
                            type="button"
                            onClick={() => setActiveDbTab(tab)}
                            className={`flex items-center gap-1.5 rounded px-2.5 py-1 font-mono text-[11px] transition-all ${
                              isActive
                                ? "bg-white text-[#111111] font-semibold shadow-2xs border border-[#E5E7EB]"
                                : "text-[#6B7280] hover:text-[#111111] hover:bg-[#F3F4F6]"
                            }`}
                          >
                            <Icon className={`h-3 w-3 ${isActive ? detail.color : "text-[#9CA3AF]"}`} />
                            <span>{tab}</span>
                          </button>
                        );
                      })}
                    </div>

                    <div className="hidden sm:flex items-center gap-2 font-mono text-[10px] text-[#6B7280]">
                      <span className="h-1.5 w-1.5 rounded-full bg-[#10B981]" />
                      <span>{dbDetails[activeDbTab].port}</span>
                    </div>
                  </div>

                  {/* Provision Spec with Line Numbers */}
                  <div className="p-3.5 bg-white font-mono text-[11px] leading-[1.7] text-[#1F2937] overflow-x-auto min-h-[170px]">
                    {dbDetails[activeDbTab].lines.map((l) => (
                      <div key={l.num} className="flex items-baseline gap-3">
                        <span className="w-4 text-right text-[#9CA3AF] select-none text-[10px]">
                          {l.num}
                        </span>
                        <div className="flex-1">
                          <code>{l.text}</code>
                        </div>
                      </div>
                    ))}
                  </div>

                  {/* Workbench Status Telemetry Footer */}
                  <div className="border-t border-[#E5E7EB] bg-[#F9FAFB] px-3.5 py-2 flex flex-wrap items-center justify-between gap-2 font-mono text-[10.5px]">
                    <div className="flex items-center gap-1.5 text-[#059669]">
                      <span className="h-1.5 w-1.5 rounded-full bg-[#10B981]" />
                      <span className="font-medium">{dbDetails[activeDbTab].status}</span>
                    </div>
                    <div className="text-[#6B7280]">
                      <span>NVMe Volume Retained</span>
                    </div>
                  </div>

                </div>
              </div>
            </div>

            {/* Feature Card 2: Scoped Security & Least Privilege */}
            <div className="flex flex-col justify-between rounded-[16px] border border-[#E5E7EB] bg-white p-6 sm:p-7 shadow-[0_2px_12px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex h-9 w-9 items-center justify-center rounded-[8px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <ShieldCheck className="h-4 w-4" />
                </div>
                <h3 className="mt-4 font-headline text-[18px] font-semibold tracking-tight text-[#111111]">
                  Scoped Security &amp; Least Privilege
                </h3>
                <p className="mt-1 text-[13px] leading-[1.55] text-[#6B7280]">
                  Never grant an AI agent an unrestricted root shell. Enforce 256-bit
                  cryptographic bearer tokens with granular role-based permissions.
                </p>
              </div>

              {/* Elevated Security Workbench */}
              <div className="mt-6 rounded-[12px] border border-[#E5E7EB] bg-[#F9FAFB] stipple-pattern p-3 sm:p-4">
                <div className="overflow-hidden rounded-[8px] border border-[#E5E7EB] bg-white shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
                  
                  {/* Security Console Tabs */}
                  <div className="flex h-[42px] items-center justify-between border-b border-[#E5E7EB] bg-[#FAFAFA] px-3">
                    <div className="flex items-center gap-1">
                      {(["Permissions", "Pairing", "Audit"] as const).map((tab) => (
                        <button
                          key={tab}
                          type="button"
                          onClick={() => setActiveSecurityTab(tab)}
                          className={`rounded px-2.5 py-1 font-mono text-[11px] font-medium transition-all ${
                            activeSecurityTab === tab
                              ? "bg-white text-[#111111] font-semibold shadow-2xs border border-[#E5E7EB]"
                              : "text-[#6B7280] hover:text-[#111111] hover:bg-[#F3F4F6]"
                          }`}
                        >
                          {tab}
                        </button>
                      ))}
                    </div>

                    <span className="font-pixel text-[8.5px] text-[#10B981]">
                      CRYPTOGRAPHIC RBAC
                    </span>
                  </div>

                  {/* Security Body: Permissions Matrix */}
                  {activeSecurityTab === "Permissions" && (
                    <div className="p-3.5 bg-white font-mono text-[11px] min-h-[170px] flex flex-col justify-between">
                      <div className="space-y-2">
                        {[
                          { action: "Docker Container Lifecycle", scope: "container:create,start,stop", status: "PERMITTED", isAllowed: true },
                          { action: "Persistent Volume Mounts", scope: "/var/lib/smoooth/* only", status: "PERMITTED", isAllowed: true },
                          { action: "Host Root Shell Access", scope: "system:bash, sudo -i", status: "BLOCKED", isAllowed: false },
                          { action: "Host SSH & Shadow Keys", scope: "/etc/shadow, /root/.ssh", status: "BLOCKED", isAllowed: false },
                        ].map((perm, idx) => (
                          <div key={idx} className="flex items-center justify-between py-1 border-b border-[#F3F4F6] last:border-none">
                            <div>
                              <div className="font-medium text-[#111111]">{perm.action}</div>
                              <div className="text-[10px] text-[#9CA3AF]">{perm.scope}</div>
                            </div>
                            <span
                              className={`rounded px-1.5 py-0.5 text-[9px] font-bold ${
                                perm.isAllowed
                                  ? "bg-[#ECFDF5] text-[#065F46]"
                                  : "bg-[#FEF2F2] text-[#991B1B]"
                              }`}
                            >
                              {perm.status}
                            </span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* Security Body: Token Pairing */}
                  {activeSecurityTab === "Pairing" && (
                    <div className="p-3.5 bg-white font-mono text-[11px] leading-[1.65] text-[#1F2937] min-h-[170px] flex flex-col justify-between">
                      <div>
                        <div className="text-[#9CA3AF] mb-1"># Generate agent-scoped token</div>
                        <div className="text-[#111111] font-semibold mb-2 overflow-x-auto">
                          <code>smoooth token create --role agent --scope "docker:read,docker:write,db:migrate"</code>
                        </div>
                        <div className="rounded border border-[#E5E7EB] bg-[#F9FAFB] p-2 text-[#4B5563] text-[10.5px] break-all flex items-center justify-between">
                          <code>smth_live_8f9a2b7c4d1e0f3a6b8c9d2e1f4a7b0c</code>
                          <button
                            type="button"
                            onClick={() => handleCopyText("smth_live_8f9a2b7c4d1e0f3a6b8c9d2e1f4a7b0c", "token")}
                            className="ml-2 shrink-0 text-[#6B7280] hover:text-[#111111]"
                          >
                            {copiedCommand === "token" ? "COPIED" : "COPY"}
                          </button>
                        </div>
                      </div>
                      <div className="text-[10px] text-[#9CA3AF] pt-2">
                        Token expiration: 7 days • HMAC-SHA256 signature verified
                      </div>
                    </div>
                  )}

                  {/* Security Body: Audit Stream */}
                  {activeSecurityTab === "Audit" && (
                    <div className="p-3.5 bg-white font-mono text-[10.5px] leading-[1.65] text-[#1F2937] min-h-[170px] overflow-x-auto">
                      <div className="text-[#9CA3AF] mb-1.5"># Append-only tamper-evident audit stream</div>
                      <div className="text-[#0284C7]">[2026-10-18T19:42:01Z] [MCP] agent:claude tool:docker_run image:postgres:16 <span className="text-[#059669]">[ALLOWED]</span></div>
                      <div className="text-[#0284C7]">[2026-10-18T19:42:02Z] [SYS] health_probe check:5432 result:OK latency:14ms</div>
                      <div className="text-[#DC2626]">[2026-10-18T19:43:10Z] [DENY] agent:claude path:/etc/shadow <span className="text-[#DC2626] font-bold">[BLOCKED:RBAC]</span></div>
                    </div>
                  )}

                  {/* Console Footer */}
                  <div className="border-t border-[#E5E7EB] bg-[#F9FAFB] px-3.5 py-2 flex items-center justify-between text-[10.5px] font-mono text-[#6B7280]">
                    <span>RESTRICTED AGENT SANDBOX</span>
                    <span className="font-pixel text-[8.5px] text-[#10B981]">● ACTIVE ENFORCEMENT</span>
                  </div>

                </div>
              </div>
            </div>

          </div>
        </section>

        {/* ===================================================================
            5. SECTION 3: QUICKSTART / HOW IT WORKS
            Connected 3-step pipeline with hairline connector tracks
            =================================================================== */}
        <section className="px-8 sm:px-12 py-16 bg-[#FAFAFA] border-t border-b border-[#E5E7EB]" id="how-it-works">
          {/* Editorial Heading Row (Strict Anti-Slop: No Eyebrow Pill) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-end">
            <div className="lg:col-span-7">
              <h2 className="font-headline text-[34px] sm:text-[40px] font-semibold leading-[1.08] tracking-[-0.03em] text-[#111111]">
                From Bare VPS to Agent-Operated<br />
                in Under 2 Minutes
              </h2>
            </div>
            <div className="lg:col-span-5">
              <p className="text-[14px] leading-[1.65] text-[#6B7280] max-w-[420px] pb-1">
                Install once on Ubuntu 24.04 or Debian 12. Connect any
                MCP-compatible coding tool over local loopback.
              </p>
            </div>
          </div>

          {/* 3 Step Cards */}
          <div className="mt-12 grid grid-cols-1 md:grid-cols-3 gap-6" id="quickstart">
            
            {/* Step 1 */}
            <div className="flex flex-col justify-between rounded-[14px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_3px_rgba(0,0,0,0.02)] transition-all hover:border-[#D1D5DB]">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className="font-pixel text-[12px] text-[#111111] font-bold tracking-widest">[01]</span>
                  <span className="font-mono text-[10px] font-semibold uppercase tracking-wider text-[#6B7280]">SYSTEMD DAEMON</span>
                </div>
                <h3 className="font-headline text-[16.5px] font-semibold tracking-tight text-[#111111] mb-2">
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
            <div className="flex flex-col justify-between rounded-[14px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_3px_rgba(0,0,0,0.02)] transition-all hover:border-[#D1D5DB]">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className="font-pixel text-[12px] text-[#111111] font-bold tracking-widest">[02]</span>
                  <span className="font-mono text-[10px] font-semibold uppercase tracking-wider text-[#6B7280]">MCP PROTOCOL</span>
                </div>
                <h3 className="font-headline text-[16.5px] font-semibold tracking-tight text-[#111111] mb-2">
                  Pair Your Coding Agent
                </h3>
                <p className="text-[13px] leading-[1.6] text-[#4B5563] mb-4">
                  Point Claude Code, Cursor, or Windsurf to Smoooth's loopback endpoint.
                  Zero unauthenticated port exposure.
                </p>
              </div>
              <div className="relative group">
                <pre className="overflow-x-auto rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] p-2.5 font-mono text-[10.5px] leading-[1.5] text-[#1F2937] whitespace-nowrap">
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
            <div className="flex flex-col justify-between rounded-[14px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_3px_rgba(0,0,0,0.02)] transition-all hover:border-[#D1D5DB]">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className="font-pixel text-[12px] text-[#111111] font-bold tracking-widest">[03]</span>
                  <span className="font-mono text-[10px] font-semibold uppercase tracking-wider text-[#6B7280]">AUTONOMOUS RUNTIME</span>
                </div>
                <h3 className="font-headline text-[16.5px] font-semibold tracking-tight text-[#111111] mb-2">
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
            4 sleek cards with Overseer industrial markers
            =================================================================== */}
        <section className="px-8 sm:px-12 py-16 bg-white border-b border-[#E5E7EB]" id="security">
          {/* Editorial Heading Row (Strict Anti-Slop: No Eyebrow Pill) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-end">
            <div className="lg:col-span-7">
              <h2 className="font-headline text-[34px] sm:text-[40px] font-semibold leading-[1.08] tracking-[-0.03em] text-[#111111]">
                Security, Control, and<br />
                Zero Cloud Lock-In
              </h2>
            </div>
            <div className="lg:col-span-5">
              <p className="text-[14px] leading-[1.65] text-[#6B7280] max-w-[420px] pb-1">
                Smoooth was designed specifically to prevent runaway agents and
                avoid surprise cloud bills.
              </p>
            </div>
          </div>

          <div className="mt-12 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
            
            {/* Card 1 */}
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)] transition-all hover:border-[#111111] hover:shadow-2xs">
              <div className="flex items-center justify-between mb-3.5">
                <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <DockerIcon className="h-4 w-4 text-[#2496ED]" />
                </div>
                <span className="font-pixel text-[9px] text-[#9CA3AF]">[01]</span>
              </div>
              <h4 className="font-headline text-[15px] font-semibold text-[#111111] mb-2">
                Zero Cloud Lock-In
              </h4>
              <p className="text-[12.5px] leading-[1.6] text-[#4B5563]">
                Runs on any $5 VPS, bare-metal server, or home lab. Your data stays on your own SSDs without proprietary PaaS pricing tiers.
              </p>
            </div>

            {/* Card 2 */}
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)] transition-all hover:border-[#111111] hover:shadow-2xs">
              <div className="flex items-center justify-between mb-3.5">
                <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <Lock className="h-4 w-4 text-[#10B981]" />
                </div>
                <span className="font-pixel text-[9px] text-[#9CA3AF]">[02]</span>
              </div>
              <h4 className="font-headline text-[15px] font-semibold text-[#111111] mb-2">
                Private by Default
              </h4>
              <p className="text-[12.5px] leading-[1.6] text-[#4B5563]">
                The daemon binds exclusively to 127.0.0.1:4040. Access the embedded web dashboard securely via SSH tunnel with zero public attack surface.
              </p>
            </div>

            {/* Card 3 */}
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)] transition-all hover:border-[#111111] hover:shadow-2xs">
              <div className="flex items-center justify-between mb-3.5">
                <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <Activity className="h-4 w-4 text-[#0284C7]" />
                </div>
                <span className="font-pixel text-[9px] text-[#9CA3AF]">[03]</span>
              </div>
              <h4 className="font-headline text-[15px] font-semibold text-[#111111] mb-2">
                Transactional 2-Phase DAG
              </h4>
              <p className="text-[12.5px] leading-[1.6] text-[#4B5563]">
                Every deployment validates an execution plan before provisioning. If a container fails health checks, Smoooth rolls back automatically.
              </p>
            </div>

            {/* Card 4 */}
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)] transition-all hover:border-[#111111] hover:shadow-2xs">
              <div className="flex items-center justify-between mb-3.5">
                <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <Terminal className="h-4 w-4 text-[#111111]" />
                </div>
                <span className="font-pixel text-[9px] text-[#9CA3AF]">[04]</span>
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
            Authoritative typographic close with 1-click install command
            =================================================================== */}
        <section className="px-8 sm:px-12 py-20 bg-[#FAFAFA] text-center flex flex-col items-center justify-center">
          <h2 className="font-headline text-[36px] sm:text-[44px] font-semibold leading-[1.06] tracking-[-0.035em] text-[#111111] mb-4">
            Take Control of Your Agent Infrastructure
          </h2>
          <p className="text-[15px] leading-[1.6] text-[#4B5563] max-w-[500px] mb-8">
            Available now for Linux x86_64. Download the v0.1.0 Beta release bundle and start deploying on infrastructure you control.
          </p>

          {/* Quickstart Command Box */}
          <div className="mb-8 inline-flex items-center gap-2 rounded-[8px] border border-[#E5E7EB] bg-white px-4 py-2.5 text-[13px] font-mono shadow-xs">
            <span className="text-[#10B981]">$</span>
            <span className="text-[#111111]">curl -sSL https://smoooth.dev/install.sh | bash</span>
            <button
              type="button"
              onClick={() => handleCopyText("curl -sSL https://smoooth.dev/install.sh | bash", "cta-install")}
              className="ml-3 rounded bg-[#F3F4F6] px-2 py-0.5 text-[10.5px] font-mono text-[#4B5563] hover:text-[#111111] hover:bg-[#E5E7EB] transition-colors"
            >
              {copiedCommand === "cta-install" ? "COPIED" : "COPY"}
            </button>
          </div>

          <div className="flex flex-wrap items-center justify-center gap-3.5">
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
