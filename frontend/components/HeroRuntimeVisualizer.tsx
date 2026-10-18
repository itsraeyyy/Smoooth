"use client";

import React, { useState } from "react";
import {
  AnthropicIcon,
  CursorIcon,
  WindsurfIcon,
  DockerIcon,
  PostgresIcon,
  RedisIcon,
} from "./BrandIcons";

type AgentTab = "claude" | "cursor" | "windsurf";

const agentDemos = {
  claude: {
    name: "Claude Code",
    agentId: "claude-code-cli",
    prompt:
      "smoooth deploy --stack postgres:16,redis:7.2,fastapi:v1.2 --ports 5432:int,6379:int,8000:pub",
    output: [
      { type: "info", text: "→ MCP session connected via stdio (protocol v2024-11-05)" },
      {
        type: "tool",
        tool: "docker_create_network",
        args: '{"name":"smoooth_internal","driver":"bridge"}',
      },
      {
        type: "tool",
        tool: "docker_provision_db",
        args: '{"adapter":"postgres","version":"16-alpine","port":5432,"vol":"pg_data"}',
      },
      {
        type: "tool",
        tool: "docker_provision_cache",
        args: '{"adapter":"redis","version":"7.2","port":6379,"maxmemory":"256mb"}',
      },
      {
        type: "tool",
        tool: "docker_deploy_service",
        args: '{"image":"app:v1.2","port":8000,"health_check":"/healthz"}',
      },
      { type: "success", text: "✓ All 3 services healthy on 127.0.0.1 in 0.68s" },
    ],
  },
  cursor: {
    name: "Cursor",
    agentId: "cursor-composer",
    prompt:
      "@smoooth Inspect active database connection pool and run migration 004_add_vector.sql",
    output: [
      { type: "info", text: "→ Validating bearer authorization (role: dba_operator)..." },
      {
        type: "tool",
        tool: "db_query",
        args: '{"adapter":"postgres","sql":"SELECT pool_size, active_conns FROM stats"}',
      },
      {
        type: "tool",
        tool: "db_migrate",
        args: '{"file":"004_add_vector.sql","target":"postgres:16","checksum":"sha256:8f2a..."}',
      },
      { type: "info", text: "  CREATE EXTENSION IF NOT EXISTS vector;" },
      { type: "info", text: "  CREATE INDEX ON doc_embeddings USING ivfflat (v vector_cosine_ops);" },
      { type: "success", text: "✓ Migration committed in 14ms (0 downtime, 0 connection drops)" },
    ],
  },
  windsurf: {
    name: "Windsurf",
    agentId: "cascade-agent",
    prompt:
      "Scale worker replicas to 2 with Redis queue attached and verify health",
    output: [
      { type: "info", text: "→ Resolving 2-phase transactional DAG dependencies..." },
      {
        type: "tool",
        tool: "docker_scale_service",
        args: '{"service":"worker","replicas":2,"queue":"redis://127.0.0.1:6379"}',
      },
      { type: "info", text: "  [DAG:phase_1] redis-cache: HEALTHY (verified)" },
      { type: "info", text: "  [DAG:phase_2] worker-01: RUNNING (18MB RAM)" },
      { type: "info", text: "  [DAG:phase_2] worker-02: RUNNING (18MB RAM)" },
      { type: "success", text: "✓ 2 worker instances actively consuming redis_queue_default" },
    ],
  },
};

export function HeroRuntimeVisualizer() {
  const [activeTab, setActiveTab] = useState<AgentTab>("claude");
  const [copied, setCopied] = useState(false);
  const [selectedContainer, setSelectedContainer] = useState<string>("postgres");

  const currentDemo = agentDemos[activeTab];

  const handleCopy = () => {
    const text = currentDemo.output.map((o) => o.text || `${o.tool}(${o.args})`).join("\n");
    navigator.clipboard?.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="relative border-b border-[#E5E7EB] bg-[#FAFAFA] overflow-hidden">
      {/* Background Topographic / Mountain Stipple Texture */}
      <div
        className="absolute inset-0 bg-bottom bg-no-repeat bg-contain opacity-25 pointer-events-none"
        style={{
          backgroundImage: "url('/mountain-bg-perfect.png')",
        }}
      />

      {/* Grid Hairline Ambient Lines */}
      <div className="absolute inset-0 bg-[linear-gradient(to_right,#E5E7EB_1px,transparent_1px),linear-gradient(to_bottom,#E5E7EB_1px,transparent_1px)] bg-[size:40px_40px] opacity-40 pointer-events-none" />

      {/* Main Framed Stage Container */}
      <div className="relative z-10 px-4 sm:px-8 py-8 sm:py-12 max-w-[1240px] mx-auto">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
          
          {/* ===================================================================
              LEFT CARD: AI CODING AGENT TERMINAL (MCP CLIENT)
              =================================================================== */}
          <div className="lg:col-span-7 flex flex-col rounded-[10px] border border-[#E5E7EB] bg-white shadow-[0_4px_20px_rgba(0,0,0,0.03)] overflow-hidden transition-all duration-150">
            {/* Terminal Window Header Bar */}
            <div className="flex items-center justify-between border-b border-[#E5E7EB] bg-[#F9FAFB] px-4 py-2.5">
              <div className="flex items-center gap-2">
                <span className="h-2.5 w-2.5 rounded-full bg-[#E5E7EB] border border-[#D1D5DB]" />
                <span className="h-2.5 w-2.5 rounded-full bg-[#E5E7EB] border border-[#D1D5DB]" />
                <span className="h-2.5 w-2.5 rounded-full bg-[#E5E7EB] border border-[#D1D5DB]" />
                <span className="ml-2 font-mono text-[11px] font-medium text-[#6B7280]">
                  agent-mcp-session — 127.0.0.1:4040
                </span>
              </div>

              {/* Agent Selector Tabs */}
              <div className="flex items-center gap-1 bg-[#ECEEF1] p-0.5 rounded-[6px]">
                <button
                  onClick={() => setActiveTab("claude")}
                  className={`flex items-center gap-1.5 px-2.5 py-1 rounded-[4px] font-mono text-[11px] transition-all ${
                    activeTab === "claude"
                      ? "bg-white text-[#111111] font-semibold shadow-sm"
                      : "text-[#6B7280] hover:text-[#111111]"
                  }`}
                >
                  <AnthropicIcon className={`h-3 w-3 ${activeTab === "claude" ? "text-[#D97757]" : "text-[#9CA3AF]"}`} />
                  <span>Claude</span>
                </button>

                <button
                  onClick={() => setActiveTab("cursor")}
                  className={`flex items-center gap-1.5 px-2.5 py-1 rounded-[4px] font-mono text-[11px] transition-all ${
                    activeTab === "cursor"
                      ? "bg-white text-[#111111] font-semibold shadow-sm"
                      : "text-[#6B7280] hover:text-[#111111]"
                  }`}
                >
                  <CursorIcon className={`h-3 w-3 ${activeTab === "cursor" ? "text-[#111111]" : "text-[#9CA3AF]"}`} />
                  <span>Cursor</span>
                </button>

                <button
                  onClick={() => setActiveTab("windsurf")}
                  className={`flex items-center gap-1.5 px-2.5 py-1 rounded-[4px] font-mono text-[11px] transition-all ${
                    activeTab === "windsurf"
                      ? "bg-white text-[#111111] font-semibold shadow-sm"
                      : "text-[#6B7280] hover:text-[#111111]"
                  }`}
                >
                  <WindsurfIcon className={`h-3 w-3 ${activeTab === "windsurf" ? "text-[#09B6A2]" : "text-[#9CA3AF]"}`} />
                  <span>Windsurf</span>
                </button>
              </div>
            </div>

            {/* Prompt Command Pill */}
            <div className="border-b border-[#F0F2F4] bg-[#FCFDFD] px-4 py-3">
              <div className="flex items-center justify-between gap-2">
                <div className="flex items-center gap-2 overflow-x-auto text-[12px] font-mono text-[#111827]">
                  <span className="text-[#10B981] font-bold">›</span>
                  <span className="font-semibold">{currentDemo.prompt}</span>
                </div>
                <button
                  onClick={handleCopy}
                  className="shrink-0 rounded px-2 py-1 font-mono text-[10px] text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#111111] transition-colors"
                >
                  {copied ? "COPIED" : "COPY"}
                </button>
              </div>
            </div>

            {/* Terminal Live Stream Body */}
            <div className="flex-1 p-4 font-mono text-[11.5px] leading-[1.7] text-[#1F2937] overflow-x-auto min-h-[220px]">
              {currentDemo.output.map((line, idx) => (
                <div key={idx} className="flex items-start gap-2">
                  {line.type === "info" && (
                    <span className="text-[#6B7280]">{line.text}</span>
                  )}
                  {line.type === "tool" && (
                    <div className="flex flex-wrap items-baseline gap-1">
                      <span className="text-[#0284C7] font-semibold">[MCP:call]</span>
                      <span className="text-[#111111] font-bold">{line.tool}</span>
                      <span className="text-[#64748B]">{line.args}</span>
                    </div>
                  )}
                  {line.type === "success" && (
                    <span className="text-[#059669] font-medium">{line.text}</span>
                  )}
                </div>
              ))}
              <div className="mt-2 flex items-center gap-1.5 text-[#9CA3AF]">
                <span>smoooth-daemon@vps:~$</span>
                <span className="inline-block h-3.5 w-1.5 bg-[#111111] animate-pulse" />
              </div>
            </div>

            {/* Terminal Footer Indicator */}
            <div className="flex items-center justify-between border-t border-[#E5E7EB] bg-[#F9FAFB] px-4 py-2 font-mono text-[10.5px] text-[#6B7280]">
              <div className="flex items-center gap-2">
                <span className="font-pixel text-[9px] text-[#10B981]">● VERIFIED</span>
                <span>Idempotent 2-Phase DAG</span>
              </div>
              <div>
                <span>Latency: 14ms</span>
              </div>
            </div>
          </div>

          {/* ===================================================================
              RIGHT CARD: LINUX VPS TOPOLOGY & RUNTIME INSPECTOR
              =================================================================== */}
          <div className="lg:col-span-5 flex flex-col rounded-[10px] border border-[#E5E7EB] bg-white shadow-[0_4px_20px_rgba(0,0,0,0.03)] overflow-hidden transition-all duration-150">
            {/* Inspector Header */}
            <div className="flex items-center justify-between border-b border-[#E5E7EB] bg-[#F9FAFB] px-4 py-2.5">
              <div className="flex items-center gap-2">
                <span className="font-headline font-semibold text-[13px] text-[#111111]">
                  VPS Runtime Inspector
                </span>
                <span className="font-mono text-[10px] text-[#9CA3AF]">Ubuntu 24.04</span>
              </div>

              <div className="flex items-center gap-1.5">
                <span className="relative flex h-2 w-2">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#10B981] opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-2 w-2 bg-[#10B981]"></span>
                </span>
                <span className="font-pixel text-[8.5px] tracking-wider text-[#10B981]">
                  HEALTHY
                </span>
              </div>
            </div>

            {/* Container Stack Cards */}
            <div className="p-3.5 flex flex-col gap-2.5 flex-1 justify-center bg-white">
              
              {/* 1. PostgreSQL Container */}
              <div
                onClick={() => setSelectedContainer("postgres")}
                className={`group flex items-center justify-between rounded-[8px] border p-2.5 cursor-pointer transition-all ${
                  selectedContainer === "postgres"
                    ? "border-[#336791] bg-[#F0F5FA] shadow-sm"
                    : "border-[#E5E7EB] bg-[#FAFAFA] hover:border-[#D1D5DB] hover:bg-white"
                }`}
              >
                <div className="flex items-center gap-2.5">
                  <div className="flex h-7 w-7 items-center justify-center rounded-[6px] bg-white border border-[#E5E7EB] text-[#336791] shadow-2xs">
                    <PostgresIcon className="h-4 w-4" />
                  </div>
                  <div>
                    <div className="flex items-center gap-1.5">
                      <span className="font-mono text-[12px] font-semibold text-[#111827]">
                        postgres-16
                      </span>
                      <span className="rounded bg-[#E0E7FF] px-1 py-0.2 font-mono text-[9px] text-[#3730A3]">
                        INTERNAL
                      </span>
                    </div>
                    <div className="font-mono text-[10.5px] text-[#6B7280]">
                      127.0.0.1:5432 • 14.8 MB
                    </div>
                  </div>
                </div>

                <div className="text-right">
                  <span className="font-pixel text-[8.5px] text-[#10B981]">ACTIVE</span>
                </div>
              </div>

              {/* 2. Redis Container */}
              <div
                onClick={() => setSelectedContainer("redis")}
                className={`group flex items-center justify-between rounded-[8px] border p-2.5 cursor-pointer transition-all ${
                  selectedContainer === "redis"
                    ? "border-[#DC382D] bg-[#FEF2F2] shadow-sm"
                    : "border-[#E5E7EB] bg-[#FAFAFA] hover:border-[#D1D5DB] hover:bg-white"
                }`}
              >
                <div className="flex items-center gap-2.5">
                  <div className="flex h-7 w-7 items-center justify-center rounded-[6px] bg-white border border-[#E5E7EB] text-[#DC382D] shadow-2xs">
                    <RedisIcon className="h-4 w-4" />
                  </div>
                  <div>
                    <div className="flex items-center gap-1.5">
                      <span className="font-mono text-[12px] font-semibold text-[#111827]">
                        redis-cache
                      </span>
                      <span className="rounded bg-[#E0E7FF] px-1 py-0.2 font-mono text-[9px] text-[#3730A3]">
                        INTERNAL
                      </span>
                    </div>
                    <div className="font-mono text-[10.5px] text-[#6B7280]">
                      127.0.0.1:6379 • 4.6 MB
                    </div>
                  </div>
                </div>

                <div className="text-right">
                  <span className="font-pixel text-[8.5px] text-[#10B981]">ACTIVE</span>
                </div>
              </div>

              {/* 3. FastApi Web App Container */}
              <div
                onClick={() => setSelectedContainer("fastapi")}
                className={`group flex items-center justify-between rounded-[8px] border p-2.5 cursor-pointer transition-all ${
                  selectedContainer === "fastapi"
                    ? "border-[#2496ED] bg-[#F0F9FF] shadow-sm"
                    : "border-[#E5E7EB] bg-[#FAFAFA] hover:border-[#D1D5DB] hover:bg-white"
                }`}
              >
                <div className="flex items-center gap-2.5">
                  <div className="flex h-7 w-7 items-center justify-center rounded-[6px] bg-white border border-[#E5E7EB] text-[#2496ED] shadow-2xs">
                    <DockerIcon className="h-4 w-4" />
                  </div>
                  <div>
                    <div className="flex items-center gap-1.5">
                      <span className="font-mono text-[12px] font-semibold text-[#111827]">
                        fastapi-app
                      </span>
                      <span className="rounded bg-[#DCFCE7] px-1 py-0.2 font-mono text-[9px] text-[#166534]">
                        HTTP 200
                      </span>
                    </div>
                    <div className="font-mono text-[10.5px] text-[#6B7280]">
                      0.0.0.0:8000 • 41.2 MB
                    </div>
                  </div>
                </div>

                <div className="text-right">
                  <span className="font-pixel text-[8.5px] text-[#10B981]">HEALTHY</span>
                </div>
              </div>
            </div>

            {/* Live Container Detail Inspector Drawer */}
            <div className="border-t border-[#F0F2F4] bg-[#FAFCFD] px-3.5 py-2.5 font-mono text-[10.5px]">
              {selectedContainer === "postgres" && (
                <div className="flex flex-col gap-1 text-[#334155]">
                  <div className="flex items-center justify-between">
                    <span className="text-[#64748B]">Storage Volume:</span>
                    <span className="font-semibold text-[#111827]">/var/lib/smoooth/volumes/pgdata</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-[#64748B]">Health Probe:</span>
                    <span className="text-[#10B981]">pg_isready -U postgres (exit 0)</span>
                  </div>
                </div>
              )}
              {selectedContainer === "redis" && (
                <div className="flex flex-col gap-1 text-[#334155]">
                  <div className="flex items-center justify-between">
                    <span className="text-[#64748B]">Persistence Mode:</span>
                    <span className="font-semibold text-[#111827]">Append-Only File (AOF)</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-[#64748B]">Health Probe:</span>
                    <span className="text-[#10B981]">redis-cli ping -&gt; PONG</span>
                  </div>
                </div>
              )}
              {selectedContainer === "fastapi" && (
                <div className="flex flex-col gap-1 text-[#334155]">
                  <div className="flex items-center justify-between">
                    <span className="text-[#64748B]">Ingress Route:</span>
                    <span className="font-semibold text-[#111827]">0.0.0.0:8000 -&gt; container:8000</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-[#64748B]">Health Endpoint:</span>
                    <span className="text-[#10B981]">GET /healthz (200 OK in 12ms)</span>
                  </div>
                </div>
              )}
            </div>

            {/* Inspector Security Guardrail Footer */}
            <div className="border-t border-[#E5E7EB] bg-[#F9FAFB] p-3 font-mono text-[10px] text-[#4B5563] space-y-1">
              <div className="flex items-center justify-between">
                <span className="text-[#6B7280]">Token Authentication:</span>
                <span className="font-medium text-[#111111]">256-bit Bearer (Verified)</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-[#6B7280]">Database Ingress:</span>
                <span className="font-medium text-[#10B981]">0 Public Ports Exposed</span>
              </div>
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}
