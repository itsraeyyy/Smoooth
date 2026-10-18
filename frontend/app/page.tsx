"use client";

import React, { useState } from "react";
import { Copy, Check } from "lucide-react";

export default function Home() {
  const [activeDbTab, setActiveDbTab] = useState<"PostgreSQL" | "Redis" | "MongoDB">("PostgreSQL");
  const [activeSecurityTab, setActiveSecurityTab] = useState<"Pairing" | "Scopes" | "Audit">("Pairing");
  const [copiedSecurity, setCopiedSecurity] = useState(false);

  const dbSnippets = {
    PostgreSQL: `POST /api/adapters/postgres/provision
{
  "service": "primary-db",
  "volume": "/var/lib/smoooth/volumes/pgdata",
  "persistence": true,
  "health_probe": "pg_isready -U postgres"
}
STATUS: HEALTHY (verified in 1.4s, 5432/tcp bound)`,

    Redis: `POST /api/adapters/redis/provision
{
  "service": "cache-layer",
  "mode": "AOF_PERSISTENT",
  "volume": "/var/lib/smoooth/volumes/redis",
  "health_probe": "redis-cli ping"
}
STATUS: HEALTHY (verified PONG, 6379/tcp bound)`,

    MongoDB: `POST /api/adapters/mongodb/provision
{
  "service": "document-store",
  "volume": "/var/lib/smoooth/volumes/mongo",
  "auth": "SCRAM-SHA-256",
  "health_probe": "mongosh --eval 'db.adminCommand({ping:1})'"
}
STATUS: HEALTHY (verified in 2.1s, 27017/tcp bound)`
  };

  const securitySnippets = {
    Pairing: {
      line1: '$ smoooth agent pair --name "claude-code" --scopes "read_only,plan,deploy"',
      line2: 'token: smt_a8f93e... (256-bit entropy, saved to /etc/smoooth/auth.token)',
      items: [
        { name: "> read_only (granted)", active: true },
        { name: "> plan (granted)", active: true },
        { name: "> deploy (granted)", active: true },
        { name: "> admin (blocked)", active: false, danger: true },
      ],
      footer: "LEAST PRIVILEGE",
      badge: "RBAC"
    },
    Scopes: {
      line1: '$ smoooth inspect scopes --agent "cursor-assistant"',
      line2: 'active_scopes: [read_only, plan, deploy]',
      items: [
        { name: "> read_only: inspect system", active: true },
        { name: "> plan: dry-run DAG plans", active: true },
        { name: "> deploy: roll containers", active: true },
        { name: "> admin: key revocation", active: false, danger: true },
      ],
      footer: "PERMISSION TIER",
      badge: "CHECKED"
    },
    Audit: {
      line1: '$ journalctl -u smoooth.service --since "10m ago"',
      line2: 'AUDIT: agent=claude-code action=deploy_postgres status=success',
      items: [
        { name: "> 18:42:01 plan validated", active: true },
        { name: "> 18:42:03 volume retained", active: true },
        { name: "> 18:42:05 healthy 200 OK", active: true },
        { name: "> 18:42:06 commit state", active: false },
      ],
      footer: "APPEND-ONLY LOG",
      badge: "AUDIT"
    }
  };

  const copyCode = (text: string) => {
    navigator.clipboard?.writeText(text);
    setCopiedSecurity(true);
    setTimeout(() => setCopiedSecurity(false), 2000);
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
        <header className="flex h-[64px] items-center justify-between border-b border-[#E5E7EB] px-8 bg-white">
          {/* Left: Brand Wordmark with Precision Pixel Prompt Glyph */}
          <a href="#home" className="flex items-center gap-2 text-[16px] font-semibold tracking-tight text-[#111111] hover:opacity-90 transition-opacity">
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
          <div className="flex items-center gap-3.5">
            <a
              href="https://github.com/smoooth-dev/smoooth"
              target="_blank"
              rel="noopener noreferrer"
              className="px-3 py-1.5 text-[14px] font-medium text-[#111111] hover:text-[#4B5563] transition-colors active:scale-[0.98]"
            >
              GitHub
            </a>
            <a
              href="#quickstart"
              className="rounded-[8px] bg-[#ECEEF1] px-4 py-2 text-[13px] font-medium text-[#111111] hover:bg-[#E2E5E9] active:scale-[0.97] transition-all shadow-[0_1px_2px_rgba(0,0,0,0.04)]"
            >
              Install v0.1.0
            </a>
          </div>
        </header>

        {/* ===================================================================
            2. HERO SECTION — ROW 1: EDITORIAL HEADLINE & ECOSYSTEM
            =================================================================== */}
        <section className="flex flex-col lg:flex-row border-b border-[#E5E7EB]" id="home">
          {/* Left Hero Column (51.67% width) */}
          <div className="w-full lg:w-[51.67%] border-b lg:border-b-0 lg:border-r border-[#E5E7EB] px-8 sm:px-12 pt-12 pb-12 flex flex-col justify-between">
            <div>
              {/* Clean & Direct Headline in Lexend (Anti-Slop: No Eyebrow Pill) */}
              <h1 className="font-headline text-[44px] sm:text-[54px] lg:text-[58px] font-semibold leading-[1.05] tracking-[-0.035em] text-[#111111]">
                Your agent chooses the stack.<br />
                Smoooth makes it work.
              </h1>
            </div>

            {/* Action Buttons */}
            <div className="mt-10 flex flex-wrap items-center gap-3.5">
              <a
                href="#quickstart"
                className="rounded-[8px] bg-[#18181B] px-5 py-2.5 text-[14px] font-medium text-white shadow-sm hover:bg-black active:scale-[0.97] transition-all"
              >
                Install on Linux
              </a>
              <a
                href="#features"
                className="rounded-[8px] border border-[#E5E7EB] bg-white px-5 py-2.5 text-[14px] font-medium text-[#111827] shadow-[0_1px_2px_rgba(0,0,0,0.03)] hover:bg-[#F9FAFB] active:scale-[0.97] transition-all"
              >
                View Architecture
              </a>
            </div>
          </div>

          {/* Right Hero Column (48.33% width) */}
          <div className="w-full lg:w-[48.33%] px-8 sm:px-10 pt-12 pb-12 flex flex-col justify-between">
            {/* Supporting Copy in Lexend */}
            <p className="text-[15px] leading-[1.65] text-[#4B5563] max-w-[440px]">
              Deploy and operate applications on infrastructure you control.
              Smoooth connects Claude Code, Cursor, and Windsurf directly to
              your Linux VPS over MCP—provisioning Docker containers, persistent
              databases, and secure networks on demand.
            </p>

            {/* Ecosystem Compatibility Row with Micro Pixel Markers */}
            <div className="mt-12 lg:mt-0">
              <div className="mb-3 text-right">
                <span className="font-mono text-[10px] tracking-[0.1em] uppercase text-[#9CA3AF]">
                  COMPATIBLE AGENTS &amp; INFRASTRUCTURE:
                </span>
              </div>
              <div className="flex items-center justify-between gap-3 text-[#4B5563]">
                {/* Claude Code */}
                <div className="flex items-center gap-1.5 font-mono text-[12px] font-medium tracking-tight">
                  <span className="h-1 w-1 bg-[#9CA3AF]" />
                  <span>Claude Code</span>
                </div>
                {/* Cursor */}
                <div className="flex items-center gap-1.5 font-mono text-[12px] font-medium tracking-tight">
                  <span className="h-1 w-1 bg-[#9CA3AF]" />
                  <span>Cursor</span>
                </div>
                {/* Windsurf */}
                <div className="flex items-center gap-1.5 font-mono text-[12px] font-medium tracking-tight">
                  <span className="h-1 w-1 bg-[#9CA3AF]" />
                  <span>Windsurf</span>
                </div>
                {/* Docker */}
                <div className="flex items-center gap-1.5 font-mono text-[12px] font-medium tracking-tight">
                  <svg className="h-3.5 w-3.5 fill-current" viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M13.983 11.078h2.119a.186.186 0 00.186-.185V9.006a.186.186 0 00-.186-.186h-2.119a.185.185 0 00-.185.185v1.888c0 .102.083.185.185.185m-2.954-5.43h2.118a.186.186 0 00.186-.186V3.574a.186.186 0 00-.186-.185h-2.118a.185.185 0 00-.185.185v1.888c0 .102.082.186.185.186m0 2.716h2.118a.187.187 0 00.186-.186V6.29a.186.186 0 00-.186-.185h-2.118a.185.185 0 00-.185.185v1.887c0 .102.082.186.185.186m-2.93 0h2.12a.186.186 0 00.184-.186V6.29a.185.185 0 00-.185-.185H8.1a.185.185 0 00-.185.185v1.887c0 .102.083.186.185.186m-2.964 0h2.119a.186.186 0 00.185-.186V6.29a.185.185 0 00-.185-.185H5.136a.186.186 0 00-.186.185v1.887c0 .102.084.186.186.186m5.893 2.715h2.118a.186.186 0 00.186-.186V9.007a.186.186 0 00-.186-.186h-2.118a.185.185 0 00-.185.185v1.888c0 .102.082.185.185.185m-2.93 0h2.12a.185.185 0 00.184-.186V9.007a.185.185 0 00-.184-.186H8.1a.185.185 0 00-.185.185v1.888c0 .102.083.185.185.185m-2.964 0h2.119a.185.185 0 00.185-.186V9.007a.185.185 0 00-.185-.186H5.136a.186.186 0 00-.186.185v1.888c0 .102.084.185.186.185m-2.928 0h2.119a.185.185 0 00.185-.186V9.007a.185.185 0 00-.185-.186H2.208a.185.185 0 00-.185.185v1.888c0 .102.082.185.185.185M23.959 10.9c-.397-.563-1.077-.85-2.02-.85-.122 0-.25.006-.381.018a4.935 4.935 0 00-1.428-.85c-.066-.026-.134-.047-.202-.068-.535-.164-1.127-.123-1.636.113-.087.04-.173.088-.256.143-.243.16-.454.364-.627.606H.619a.62.62 0 00-.619.62c0 2.22.42 4.148 1.25 5.733 1.053 2.012 2.766 3.327 4.957 3.805 1.542.336 3.39.294 5.378-.124 3.01-.634 5.86-2.316 7.82-4.618.307-.36.58-.74.819-1.137.49-.076.99-.214 1.48-.415.545-.224.97-.53 1.264-.91.312-.405.394-.827.247-1.228-.088-.242-.266-.543-.256-.688"/>
                  </svg>
                  <span>Docker</span>
                </div>
                {/* Ubuntu */}
                <div className="flex items-center gap-1.5 font-mono text-[12px] font-medium tracking-tight">
                  <span className="h-1 w-1 bg-[#9CA3AF]" />
                  <span>Ubuntu/Debian</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* ===================================================================
            3. HERO SECTION — ROW 2: TECHNICAL PRODUCT COMPOSITION
            =================================================================== */}
        <div className="relative border-b border-[#E5E7EB] bg-white overflow-hidden">
          <img
            src="/hero_visual_2x.png"
            alt="Smoooth agent orchestration and infrastructure dashboard preview showing Docker container and PostgreSQL database management"
            className="w-full h-auto block select-none pointer-events-none"
          />
        </div>

        {/* ===================================================================
            4. SECTION 2: AUTONOMOUS INFRASTRUCTURE RUNTIME (FEATURES)
            =================================================================== */}
        <section className="px-8 sm:px-12 py-16 bg-white" id="features">
          {/* Clean & Direct Editorial Heading Row (Anti-Slop: No Eyebrow Pill) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-end">
            <div className="lg:col-span-7">
              <h2 className="font-headline text-[38px] sm:text-[44px] font-semibold leading-[1.08] tracking-[-0.03em] text-[#111111]">
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
                  <svg className="h-4 w-4 stroke-[2]" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <ellipse cx="12" cy="5" rx="9" ry="3"></ellipse>
                    <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path>
                    <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path>
                  </svg>
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
                  {/* Code Bar with Tabs and Pixel Dotbar */}
                  <div className="flex h-[38px] items-center justify-between border-b border-[#E5E7EB] bg-[#FAFAFA] px-3.5">
                    <div className="flex items-center gap-1" role="tablist">
                      {(["PostgreSQL", "Redis", "MongoDB"] as const).map((tab) => (
                        <button
                          key={tab}
                          type="button"
                          onClick={() => setActiveDbTab(tab)}
                          className={`rounded px-2 py-0.5 font-mono text-[11px] font-medium transition-all ${
                            activeDbTab === tab
                              ? "bg-[#E5E7EB] text-[#111111] font-semibold"
                              : "text-[#6B7280] hover:text-[#111111]"
                          }`}
                        >
                          {tab}
                        </button>
                      ))}
                    </div>

                    <div className="flex items-center gap-2">
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
                  <div className="p-4 bg-white font-mono text-[11.5px] leading-[1.65] text-[#1F2937] overflow-x-auto">
                    <pre>
                      <code>{dbSnippets[activeDbTab]}</code>
                    </pre>
                  </div>
                </div>
              </div>
            </div>

            {/* Feature Card 2: Scoped Security */}
            <div className="flex flex-col justify-between rounded-[18px] border border-[#E5E7EB] bg-white p-7 shadow-[0_2px_8px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex h-9 w-9 items-center justify-center rounded-[8px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <svg className="h-4 w-4 stroke-[2]" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                  </svg>
                </div>
                <h3 className="mt-4 font-headline text-[18px] font-semibold tracking-tight text-[#111111]">
                  Scoped Security &amp; Least Privilege
                </h3>
                <p className="mt-1 text-[13px] leading-[1.5] text-[#6B7280]">
                  Never grant an AI agent an unrestricted root shell. Enforce
                  256-bit cryptographic bearer tokens with granular role-based permissions.
                </p>
              </div>

              {/* Code Well with Dotted Stipple Accent */}
              <div className="mt-6 rounded-[12px] border border-[#E5E7EB] bg-[#F9FAFB] stipple-pattern p-4">
                <div className="overflow-hidden rounded-[8px] border border-[#E5E7EB] bg-white shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
                  {/* Code Bar with Tabs and Copy Button */}
                  <div className="flex h-[38px] items-center justify-between border-b border-[#E5E7EB] bg-[#FAFAFA] px-3.5">
                    <div className="flex items-center gap-1" role="tablist">
                      {(["Pairing", "Scopes", "Audit"] as const).map((tab) => (
                        <button
                          key={tab}
                          type="button"
                          onClick={() => setActiveSecurityTab(tab)}
                          className={`rounded px-2 py-0.5 font-mono text-[11px] font-medium transition-all ${
                            activeSecurityTab === tab
                              ? "bg-[#E5E7EB] text-[#111111] font-semibold"
                              : "text-[#6B7280] hover:text-[#111111]"
                          }`}
                        >
                          {tab === "Pairing" ? "Agent Pairing" : tab === "Scopes" ? "Scopes" : "Audit Stream"}
                        </button>
                      ))}
                    </div>

                    <button
                      type="button"
                      onClick={() => copyCode(securitySnippets[activeSecurityTab].line1)}
                      className="flex items-center gap-1 text-[11px] font-mono text-[#6B7280] hover:text-[#111111] transition-colors"
                      aria-label="Copy code"
                    >
                      {copiedSecurity ? <Check className="h-3 w-3 text-[#10B981]" /> : <Copy className="h-3 w-3" />}
                      <span>{copiedSecurity ? "Copied" : "Copy"}</span>
                    </button>
                  </div>

                  {/* Code Body with Scoped Autocomplete Popover */}
                  <div className="relative min-h-[195px] p-4 bg-white font-mono text-[11.5px] leading-[1.65] text-[#1F2937]">
                    <div>
                      <span className="text-[#9CA3AF] mr-3 select-none">1</span>
                      <span className="font-medium">{securitySnippets[activeSecurityTab].line1}</span>
                    </div>
                    <div className="mt-1">
                      <span className="text-[#9CA3AF] mr-3 select-none">2</span>
                      <span className="text-[#4B5563]">{securitySnippets[activeSecurityTab].line2}</span>
                    </div>

                    {/* Granular Scope Popover with Silkscreen Footer */}
                    <div className="absolute left-[88px] top-[54px] w-[200px] overflow-hidden rounded-[8px] border border-[#E5E7EB] bg-white shadow-[0_10px_25px_rgba(0,0,0,0.1)] z-20">
                      {securitySnippets[activeSecurityTab].items.map((item, i) => (
                        <div
                          key={i}
                          className={`px-3 py-1 text-[10px] font-mono ${
                            item.danger
                              ? "text-[#DC2626] bg-red-50/50"
                              : item.active
                              ? "bg-[#F0F4F8] text-[#1E293B] font-medium"
                              : "text-[#64748B]"
                          }`}
                        >
                          {item.name}
                        </div>
                      ))}
                      <div className="flex items-center justify-between bg-[#111827] px-3 py-1 text-[9.5px] text-white">
                        <span className="font-pixel text-[8px] tracking-[0.08em] text-[#9CA3AF]">
                          {securitySnippets[activeSecurityTab].footer}
                        </span>
                        <span className="rounded-[2px] bg-white px-1.5 py-0.5 font-mono text-[8.5px] font-bold text-[#111827]">
                          {securitySnippets[activeSecurityTab].badge}
                        </span>
                      </div>
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
              <h2 className="font-headline text-[38px] sm:text-[44px] font-semibold leading-[1.08] tracking-[-0.03em] text-[#111111]">
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
                  <span className="font-pixel text-[11px] text-[#9CA3AF] tracking-widest">01</span>
                  <span className="font-mono text-[10.5px] font-medium text-[#9CA3AF]">SYSTEMD DAEMON</span>
                </div>
                <h3 className="font-headline text-[17px] font-semibold tracking-tight text-[#111111] mb-2">
                  Install the Daemon
                </h3>
                <p className="text-[13px] leading-[1.6] text-[#4B5563] mb-4">
                  The idempotent installer configures an unprivileged service account,
                  systemd isolation, and a 256-bit bearer token.
                </p>
              </div>
              <pre className="overflow-x-auto rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] p-2.5 font-mono text-[11px] text-[#1F2937]">
                <code>sudo ./install.sh --show-token</code>
              </pre>
            </div>

            {/* Step 2 */}
            <div className="flex flex-col justify-between rounded-[14px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className="font-pixel text-[11px] text-[#9CA3AF] tracking-widest">02</span>
                  <span className="font-mono text-[10.5px] font-medium text-[#9CA3AF]">MCP PROTOCOL</span>
                </div>
                <h3 className="font-headline text-[17px] font-semibold tracking-tight text-[#111111] mb-2">
                  Pair Your Coding Agent
                </h3>
                <p className="text-[13px] leading-[1.6] text-[#4B5563] mb-4">
                  Point Claude Code, Cursor, or Windsurf to Smoooth's loopback endpoint.
                  Zero unauthenticated port exposure.
                </p>
              </div>
              <pre className="overflow-x-auto rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] p-2.5 font-mono text-[11px] text-[#1F2937]">
                <code>claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse</code>
              </pre>
            </div>

            {/* Step 3 */}
            <div className="flex flex-col justify-between rounded-[14px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className="font-pixel text-[11px] text-[#9CA3AF] tracking-widest">03</span>
                  <span className="font-mono text-[10.5px] font-medium text-[#9CA3AF]">AUTONOMOUS RUNTIME</span>
                </div>
                <h3 className="font-headline text-[17px] font-semibold tracking-tight text-[#111111] mb-2">
                  Prompt to Provision &amp; Deploy
                </h3>
                <p className="text-[13px] leading-[1.6] text-[#4B5563] mb-4">
                  Smoooth resolves dependencies into a topological DAG, creates persistent
                  volumes, and verifies container health probes.
                </p>
              </div>
              <pre className="overflow-x-auto rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] p-2.5 font-mono text-[11px] text-[#1F2937]">
                <code>"Deploy my web app with PostgreSQL and Redis on port 8000."</code>
              </pre>
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
              <h2 className="font-headline text-[38px] sm:text-[44px] font-semibold leading-[1.08] tracking-[-0.03em] text-[#111111]">
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
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)]">
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
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)]">
              <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111] mb-3.5">
                <svg className="h-4 w-4" viewBox="0 0 16 16" fill="currentColor">
                  <rect x="5" y="2" width="6" height="2" />
                  <rect x="4" y="4" width="2" height="4" />
                  <rect x="10" y="4" width="2" height="4" />
                  <rect x="3" y="7" width="10" height="7" />
                  <rect x="7" y="9" width="2" height="2" fill="#10B981" />
                  <rect x="7.5" y="11" width="1" height="2" fill="#10B981" />
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
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)]">
              <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111] mb-3.5">
                <svg className="h-4 w-4" viewBox="0 0 16 16" fill="currentColor">
                  <rect x="2" y="4" width="2" height="2" />
                  <rect x="4" y="2" width="8" height="2" />
                  <rect x="12" y="4" width="2" height="3" />
                  <rect x="2" y="9" width="2" height="3" />
                  <rect x="4" y="12" width="8" height="2" />
                  <rect x="12" y="10" width="2" height="2" />
                  <rect x="11" y="2" width="3" height="4" fill="#10B981" />
                  <rect x="2" y="10" width="3" height="4" fill="#10B981" />
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
            <div className="rounded-[12px] border border-[#E5E7EB] bg-white p-6 shadow-[0_1px_2px_rgba(0,0,0,0.02)]">
              <div className="flex h-7 w-7 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111] mb-3.5">
                <svg className="h-4 w-4" viewBox="0 0 16 16" fill="currentColor">
                  <rect x="2" y="3" width="12" height="10" fill="#E5E7EB" />
                  <rect x="3" y="4" width="10" height="7" fill="#111111" />
                  <rect x="5" y="6" width="3" height="2" fill="#10B981" />
                  <rect x="9" y="6" width="2" height="1" fill="#FFFFFF" />
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
            7. SECTION 5: FINAL CALL TO ACTION
            =================================================================== */}
        <section className="px-8 sm:px-12 py-20 bg-[#FAFAFA] text-center flex flex-col items-center justify-center">
          <h2 className="font-headline text-[40px] sm:text-[46px] font-semibold leading-[1.06] tracking-[-0.03em] text-[#111111] mb-4">
            Take Control of Your Agent Infrastructure
          </h2>
          <p className="text-[15px] leading-[1.6] text-[#4B5563] max-w-[500px] mb-8">
            Available now for Linux x86_64. Download the v0.1.0 Beta release bundle
            and start deploying on infrastructure you control.
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
            <span className="text-[#6B7280]">Infrastructure freedom, built for AI agents. Open-source under Apache-2.0 / MIT.</span>
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
