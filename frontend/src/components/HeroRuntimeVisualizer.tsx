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
import { Terminal, Database, Server, Cpu, Check, Copy, ArrowRight, Activity, ShieldCheck, Play } from "lucide-react";

type AgentTab = "claude" | "cursor" | "windsurf";
type ContainerId = "postgres" | "redis" | "fastapi";

interface ContainerDetail {
  id: ContainerId;
  name: string;
  image: string;
  port: string;
  scope: string;
  memory: string;
  cpu: string;
  volume: string;
  probe: string;
  status: "ACTIVE" | "HEALTHY";
  protocol: string;
  color: string;
  icon: React.FC<{ className?: string }>;
}

const containerRegistry: Record<ContainerId, ContainerDetail> = {
  postgres: {
    id: "postgres",
    name: "postgres-16",
    image: "postgres:16-alpine",
    port: "127.0.0.1:5432",
    scope: "INTERNAL",
    memory: "14.8 MB",
    cpu: "0.2%",
    volume: "/var/lib/smoooth/volumes/pgdata (NVMe SSD)",
    probe: "pg_isready -U postgres (exit 0, 1.4s)",
    status: "ACTIVE",
    protocol: "TCP / Binary Protocol",
    color: "#336791",
    icon: PostgresIcon,
  },
  redis: {
    id: "redis",
    name: "redis-cache",
    image: "redis:7.2-alpine",
    port: "127.0.0.1:6379",
    scope: "INTERNAL",
    memory: "4.6 MB",
    cpu: "0.1%",
    volume: "In-Memory AOF Persistence",
    probe: "redis-cli ping -> PONG (exit 0)",
    status: "ACTIVE",
    protocol: "RESP / In-Memory",
    color: "#DC382D",
    icon: RedisIcon,
  },
  fastapi: {
    id: "fastapi",
    name: "fastapi-app",
    image: "smoooth-app:v1.2",
    port: "0.0.0.0:8000",
    scope: "HTTP REVERSE PROXY",
    memory: "41.2 MB",
    cpu: "0.8%",
    volume: "/app/src (Read-Only Mount)",
    probe: "GET /healthz -> 200 OK (14ms latency)",
    status: "HEALTHY",
    protocol: "HTTP/1.1 REST API",
    color: "#2496ED",
    icon: DockerIcon,
  },
};

const agentDemos = {
  claude: {
    name: "Claude Code",
    agentId: "claude-code-cli",
    promptPill: "Deploy full-stack app with PostgreSQL & Redis on VPS",
    cliCommand: "smoooth deploy --stack postgres:16,redis:7.2,fastapi:v1.2 --ports 5432:int,6379:int,8000:pub",
    files: [
      {
        name: "deploy.ts",
        lines: [
          { num: 1, text: '// Coding agent invokes Smoooth MCP daemon over stdio', type: 'comment' },
          { num: 2, text: 'import { mcp } from "@smoooth/runtime";', type: 'code' },
          { num: 3, text: '', type: 'code' },
          { num: 4, text: 'await mcp.network.create({ name: "smoooth_bridge" });', type: 'code' },
          { num: 5, text: 'await mcp.db.provision({ adapter: "postgres", version: "16", volume: "pgdata" });', type: 'code' },
          { num: 6, text: 'await mcp.cache.provision({ adapter: "redis", version: "7.2", maxmemory: "512mb" });', type: 'code' },
          { num: 7, text: 'await mcp.service.deploy({ image: "app:v1.2", port: 8000, probe: "/healthz" });', type: 'code' },
          { num: 8, text: '', type: 'code' },
          { num: 9, text: '✓ 3/3 services provisioned & healthy on 127.0.0.1 in 0.68s', type: 'success' },
        ],
      },
      {
        name: "docker-compose.yml",
        lines: [
          { num: 1, text: '# Generated transactional DAG plan (validated pre-flight)', type: 'comment' },
          { num: 2, text: 'version: "3.8"', type: 'code' },
          { num: 3, text: 'services:', type: 'code' },
          { num: 4, text: '  postgres: { image: postgres:16, ports: ["127.0.0.1:5432:5432"] }', type: 'code' },
          { num: 5, text: '  redis:    { image: redis:7.2,    ports: ["127.0.0.1:6379:6379"] }', type: 'code' },
          { num: 6, text: '  app:      { image: app:v1.2,     ports: ["0.0.0.0:8000:8000"] }', type: 'code' },
        ],
      },
      {
        name: "schema.sql",
        lines: [
          { num: 1, text: '-- Automated database migration via scoped bearer token', type: 'comment' },
          { num: 2, text: 'CREATE EXTENSION IF NOT EXISTS "uuid-ossp";', type: 'code' },
          { num: 3, text: 'CREATE EXTENSION IF NOT EXISTS vector;', type: 'code' },
          { num: 4, text: 'CREATE TABLE embeddings (id UUID PRIMARY KEY, vec vector(1536));', type: 'code' },
        ],
      },
    ],
  },
  cursor: {
    name: "Cursor",
    agentId: "cursor-composer",
    promptPill: "Inspect active connection pool and run migration 004_add_vector.sql",
    cliCommand: "@smoooth inspect pool && smoooth migrate --file 004_add_vector.sql",
    files: [
      {
        name: "migration.ts",
        lines: [
          { num: 1, text: '// Cursor background agent verifies bearer token & pool health', type: 'comment' },
          { num: 2, text: 'const pool = await mcp.db.inspectPool({ adapter: "postgres" });', type: 'code' },
          { num: 3, text: 'console.log(`Active connections: ${pool.active} / ${pool.max}`);', type: 'code' },
          { num: 4, text: '', type: 'code' },
          { num: 5, text: 'await mcp.db.migrate({', type: 'code' },
          { num: 6, text: '  file: "004_add_vector.sql",', type: 'code' },
          { num: 7, text: '  checksum: "sha256:8f2a4b1c9e3d7a5f6e8d2c1b0a9f8e7d"', type: 'code' },
          { num: 8, text: '});', type: 'code' },
          { num: 9, text: '✓ Migration committed in 14ms (0 downtime, 0 connection drops)', type: 'success' },
        ],
      },
    ],
  },
  windsurf: {
    name: "Windsurf",
    agentId: "cascade-agent",
    promptPill: "Scale worker service to 2 replicas with Redis queue attached",
    cliCommand: "smoooth scale worker --replicas 2 --queue redis://127.0.0.1:6379",
    files: [
      {
        name: "scale.ts",
        lines: [
          { num: 1, text: '// Windsurf Cascade validates 2-phase dependency DAG', type: 'comment' },
          { num: 2, text: 'await mcp.scale({', type: 'code' },
          { num: 3, text: '  service: "worker",', type: 'code' },
          { num: 4, text: '  replicas: 2,', type: 'code' },
          { num: 5, text: '  queue: "redis://127.0.0.1:6379"', type: 'code' },
          { num: 6, text: '});', type: 'code' },
          { num: 7, text: '', type: 'code' },
          { num: 8, text: '✓ 2 worker instances actively consuming redis_queue_default', type: 'success' },
        ],
      },
    ],
  },
};

export function HeroRuntimeVisualizer() {
  const [activeTab, setActiveTab] = useState<AgentTab>("claude");
  const [activeFileIndex, setActiveFileIndex] = useState<number>(0);
  const [selectedContainer, setSelectedContainer] = useState<ContainerId>("postgres");
  const [copied, setCopied] = useState(false);
  const [isSimulating, setIsSimulating] = useState(false);

  const currentDemo = agentDemos[activeTab];
  const activeFile = currentDemo.files[activeFileIndex] || currentDemo.files[0];
  const selectedDetail = containerRegistry[selectedContainer];

  const handleCopyCode = () => {
    const text = activeFile.lines.map((l) => l.text).join("\n");
    navigator.clipboard?.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleTriggerSim = () => {
    setIsSimulating(true);
    setTimeout(() => setIsSimulating(false), 1200);
  };

  return (
    <div className="relative border-b border-[#E5E7EB] bg-[#FAFAFA] overflow-hidden select-none">
      {/* 
        =======================================================================
        BACKGROUND: Seamless Topographic / Mountain Stipple Backdrop
        Masked so there are ZERO seams, edge artifacts, or abrupt cutoffs
        ======================================================================= 
      */}
      <div
        className="absolute inset-0 bg-bottom bg-no-repeat bg-cover opacity-30 pointer-events-none"
        style={{
          backgroundImage: "url('/mountain_ridge_clean.png')",
          backgroundPosition: "center bottom",
          maskImage: "linear-gradient(to right, transparent, black 8%, black 92%, transparent), linear-gradient(to bottom, transparent, black 15%, black 90%, transparent)",
          WebkitMaskImage: "linear-gradient(to right, transparent, black 8%, black 92%, transparent), linear-gradient(to bottom, transparent, black 15%, black 90%, transparent)",
          maskComposite: "intersect",
          WebkitMaskComposite: "source-in",
        }}
      />

      {/* Ambient Grid Hairline Pattern */}
      <div className="absolute inset-0 bg-[linear-gradient(to_right,#E5E7EB_1px,transparent_1px),linear-gradient(to_bottom,#E5E7EB_1px,transparent_1px)] bg-[size:32px_32px] opacity-40 pointer-events-none" />

      {/* Main Framed Stage Container */}
      <div className="relative z-10 px-4 sm:px-8 py-8 sm:py-12 max-w-[1240px] mx-auto">
        
        {/* ===================================================================
            OVERARCHING RUNTIME CONTROL STRIP (Reserved industrial Overseer style)
            =================================================================== */}
        <div className="mb-6 flex flex-wrap items-center justify-between gap-3 rounded-[8px] border border-[#E5E7EB] bg-white/90 backdrop-blur-sm px-4 py-2 text-[12px] font-mono shadow-[0_1px_3px_rgba(0,0,0,0.02)]">
          <div className="flex items-center gap-4 text-[#4B5563]">
            <div className="flex items-center gap-1.5">
              <span className="h-2 w-2 rounded-full bg-[#10B981] animate-pulse" />
              <span className="font-semibold text-[#111111]">127.0.0.1:4040</span>
              <span className="text-[#9CA3AF]">(Loopback RPC)</span>
            </div>
            <span className="hidden sm:inline text-[#E5E7EB]">|</span>
            <div className="hidden sm:flex items-center gap-1.5 text-[#6B7280]">
              <span>DAEMON:</span>
              <span className="font-medium text-[#111111]">ACTIVE (systemd)</span>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <div className="hidden md:flex items-center gap-1.5 text-[#6B7280]">
              <span>PROTOCOL:</span>
              <span className="font-medium text-[#111111]">MCP SSE v2024-11-05</span>
            </div>
            <span className="hidden md:inline text-[#E5E7EB]">|</span>
            <div className="flex items-center gap-1.5">
              <span className="font-pixel text-[9px] text-[#10B981]">● 3/3 HEALTHY</span>
            </div>
          </div>
        </div>

        {/* ===================================================================
            WORKBENCH GRID: TWO COOPERATING WORKSPACES
            Left: AI Coding Agent Workspace
            Right: Linux VPS Runtime Studio
            =================================================================== */}
        <div className="relative grid grid-cols-1 lg:grid-cols-12 gap-6 items-stretch">
          
          {/* =================================================================
              LEFT: AI CODING AGENT WORKSPACE (MCP Client)
              With Floating Prompt Bubble & Leader Line from reference screenshot
              ================================================================= */}
          <div className="lg:col-span-7 flex flex-col relative">
            
            {/* Floating Prompt Bubble & Delicate Leader Line */}
            <div className="mb-4 flex items-center justify-start pl-4 sm:pl-8">
              <div className="relative inline-flex items-center gap-2 rounded-full border border-[#CBD5E1] bg-[#F1F5F9] px-4 py-1.5 text-[12px] font-medium text-[#0F172A] shadow-xs">
                <span className="h-1.5 w-1.5 rounded-full bg-[#10B981] animate-pulse" />
                <span>{currentDemo.promptPill}</span>
                {/* SVG curved leader line pointing down to editor */}
                <svg
                  className="absolute -bottom-3.5 left-8 h-4 w-5 text-[#94A3B8]"
                  viewBox="0 0 20 16"
                  fill="none"
                  aria-hidden="true"
                >
                  <path d="M2 0v7a4 4 0 004 4h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeDasharray="3 2" />
                  <circle cx="2" cy="1" r="1.5" fill="#10B981" />
                </svg>
              </div>
            </div>

            {/* Editor Window Card */}
            <div className="flex-1 flex flex-col rounded-[14px] border border-[#E5E7EB] bg-white shadow-[0_12px_36px_rgba(0,0,0,0.05)] overflow-hidden transition-all duration-150">
              
              {/* Window Header */}
              <div className="flex items-center justify-between border-b border-[#E5E7EB] bg-[#F9FAFB] px-4 py-2.5">
                {/* macOS Window Controls */}
                <div className="flex items-center gap-2">
                  <span className="h-2.5 w-2.5 rounded-full bg-[#FF5F56] border border-[#E0443E]" />
                  <span className="h-2.5 w-2.5 rounded-full bg-[#FFBD2E] border border-[#DEA123]" />
                  <span className="h-2.5 w-2.5 rounded-full bg-[#27C93F] border border-[#1AAB29]" />
                  <span className="ml-2 font-mono text-[11px] font-medium text-[#6B7280]">
                    smoooth-agent-session
                  </span>
                </div>

                {/* Agent Selector Tabs */}
                <div className="flex items-center gap-1 bg-[#ECEEF1] p-0.5 rounded-[6px]">
                  <button
                    type="button"
                    onClick={() => {
                      setActiveTab("claude");
                      setActiveFileIndex(0);
                    }}
                    className={`flex items-center gap-1.5 px-2.5 py-1 rounded-[4px] font-mono text-[11px] transition-all ${
                      activeTab === "claude"
                        ? "bg-white text-[#111111] font-semibold shadow-xs"
                        : "text-[#6B7280] hover:text-[#111111]"
                    }`}
                  >
                    <AnthropicIcon className={`h-3 w-3 ${activeTab === "claude" ? "text-[#D97757]" : "text-[#9CA3AF]"}`} />
                    <span>Claude</span>
                  </button>

                  <button
                    type="button"
                    onClick={() => {
                      setActiveTab("cursor");
                      setActiveFileIndex(0);
                    }}
                    className={`flex items-center gap-1.5 px-2.5 py-1 rounded-[4px] font-mono text-[11px] transition-all ${
                      activeTab === "cursor"
                        ? "bg-white text-[#111111] font-semibold shadow-xs"
                        : "text-[#6B7280] hover:text-[#111111]"
                    }`}
                  >
                    <CursorIcon className={`h-3 w-3 ${activeTab === "cursor" ? "text-[#111111]" : "text-[#9CA3AF]"}`} />
                    <span>Cursor</span>
                  </button>

                  <button
                    type="button"
                    onClick={() => {
                      setActiveTab("windsurf");
                      setActiveFileIndex(0);
                    }}
                    className={`flex items-center gap-1.5 px-2.5 py-1 rounded-[4px] font-mono text-[11px] transition-all ${
                      activeTab === "windsurf"
                        ? "bg-white text-[#111111] font-semibold shadow-xs"
                        : "text-[#6B7280] hover:text-[#111111]"
                    }`}
                  >
                    <WindsurfIcon className={`h-3 w-3 ${activeTab === "windsurf" ? "text-[#09B6A2]" : "text-[#9CA3AF]"}`} />
                    <span>Windsurf</span>
                  </button>
                </div>
              </div>

              {/* Sub-Tabs: Code Files */}
              <div className="flex items-center justify-between border-b border-[#F0F2F4] bg-[#FCFDFD] px-4 py-1.5">
                <div className="flex items-center gap-1">
                  {currentDemo.files.map((file, idx) => (
                    <button
                      key={file.name}
                      type="button"
                      onClick={() => setActiveFileIndex(idx)}
                      className={`px-2.5 py-1 rounded font-mono text-[11px] transition-colors ${
                        activeFileIndex === idx
                          ? "bg-white text-[#111111] font-semibold border border-[#E5E7EB] shadow-2xs"
                          : "text-[#6B7280] hover:text-[#111111] hover:bg-[#F3F4F6]"
                      }`}
                    >
                      {file.name}
                    </button>
                  ))}
                </div>

                <button
                  type="button"
                  onClick={handleCopyCode}
                  className="flex items-center gap-1 rounded px-2 py-0.5 font-mono text-[10px] text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#111111] transition-colors"
                >
                  {copied ? (
                    <>
                      <Check className="h-3 w-3 text-[#10B981]" />
                      <span className="text-[#10B981]">COPIED</span>
                    </>
                  ) : (
                    <>
                      <Copy className="h-3 w-3" />
                      <span>COPY</span>
                    </>
                  )}
                </button>
              </div>

              {/* Code Body with Real Syntax Highlighting and Line Numbers */}
              <div className="flex-1 p-4 bg-white font-mono text-[11.5px] leading-[1.7] overflow-x-auto min-h-[220px]">
                {activeFile.lines.map((line) => (
                  <div key={line.num} className="flex items-baseline gap-3">
                    <span className="w-5 text-right text-[#9CA3AF] select-none text-[10.5px]">
                      {line.num}
                    </span>
                    <div className="flex-1">
                      {line.type === "comment" && (
                        <span className="text-[#9CA3AF] italic">{line.text}</span>
                      )}
                      {line.type === "code" && (
                        <span className="text-[#1F2937]">{line.text}</span>
                      )}
                      {line.type === "success" && (
                        <span className="text-[#059669] font-medium bg-[#ECFDF5] px-1 rounded inline-block">
                          {line.text}
                        </span>
                      )}
                    </div>
                  </div>
                ))}
              </div>

              {/* Interactive Bottom Prompt Bar */}
              <div className="border-t border-[#E5E7EB] bg-[#F9FAFB] p-2.5 flex items-center justify-between gap-2">
                <div className="flex-1 flex items-center gap-2 rounded-[6px] border border-[#E5E7EB] bg-white px-3 py-1.5 text-[12px] font-mono text-[#111827] min-w-0">
                  <span className="text-[#10B981] font-bold shrink-0">›</span>
                  <input
                    type="text"
                    readOnly
                    value={currentDemo.cliCommand}
                    className="w-full bg-transparent outline-none text-[#1F2937] text-[11px] truncate"
                  />
                </div>
                <button
                  type="button"
                  onClick={handleTriggerSim}
                  className="shrink-0 rounded-[6px] bg-[#18181B] px-3 py-1.5 font-mono text-[11px] font-medium text-white hover:bg-black active:scale-[0.97] transition-all flex items-center gap-1.5 shadow-2xs"
                >
                  <Play className={`h-3 w-3 ${isSimulating ? "text-[#10B981] animate-spin" : "text-white"}`} />
                  <span>Execute</span>
                </button>
              </div>
            </div>
          </div>

          {/* =================================================================
              RIGHT: LINUX VPS RUNTIME STUDIO & CONTAINER TOPOLOGY
              Interactive container pods and live health probe inspector
              ================================================================= */}
          <div className="lg:col-span-5 flex flex-col relative mt-8 lg:mt-0">
            
            {/* Header Tag */}
            <div className="mb-3 flex items-center justify-between px-1">
              <span className="font-mono text-[11px] uppercase tracking-wider text-[#6B7280]">
                HOST TOPOLOGY &amp; CONTAINERS
              </span>
              <span className="font-pixel text-[9px] text-[#10B981]">
                UBUNTU 24.04 LTS
              </span>
            </div>

            {/* Studio Card */}
            <div className="flex-1 flex flex-col rounded-[14px] border border-[#E5E7EB] bg-white shadow-[0_12px_36px_rgba(0,0,0,0.05)] overflow-hidden transition-all duration-150">
              
              {/* Studio Window Bar */}
              <div className="flex items-center justify-between border-b border-[#E5E7EB] bg-[#F9FAFB] px-4 py-2.5">
                <div className="flex items-center gap-2">
                  <Server className="h-3.5 w-3.5 text-[#111111]" />
                  <span className="font-mono text-[12px] font-semibold text-[#111111]">
                    vps-node-01
                  </span>
                  <span className="font-mono text-[10.5px] text-[#9CA3AF]">142.93.18.42</span>
                </div>

                <div className="flex items-center gap-1.5">
                  <span className="h-2 w-2 rounded-full bg-[#10B981]" />
                  <span className="font-pixel text-[8.5px] text-[#10B981]">HEALTHY</span>
                </div>
              </div>

              {/* Interactive Container Pods List */}
              <div className="p-3.5 flex flex-col gap-2 bg-white flex-1">
                {(["postgres", "redis", "fastapi"] as ContainerId[]).map((cId) => {
                  const cont = containerRegistry[cId];
                  const Icon = cont.icon;
                  const isSelected = selectedContainer === cId;

                  return (
                    <div
                      key={cId}
                      onClick={() => setSelectedContainer(cId)}
                      className={`group flex items-center justify-between rounded-[8px] border p-2.5 cursor-pointer transition-all duration-150 ${
                        isSelected
                          ? "border-[#111111] bg-[#F9FAFB] shadow-xs"
                          : "border-[#E5E7EB] bg-white hover:border-[#D1D5DB] hover:bg-[#FAFAFA]"
                      }`}
                    >
                      <div className="flex items-center gap-3">
                        <div
                          className="flex h-8 w-8 items-center justify-center rounded-[6px] border border-[#E5E7EB] bg-white shadow-2xs transition-colors"
                          style={{ color: cont.color }}
                        >
                          <Icon className="h-4 w-4" />
                        </div>
                        <div>
                          <div className="flex items-center gap-1.5">
                            <span className="font-mono text-[12px] font-semibold text-[#111827]">
                              {cont.name}
                            </span>
                            <span
                              className={`rounded px-1.5 py-0.2 font-mono text-[9px] font-medium ${
                                cont.scope === "INTERNAL"
                                  ? "bg-[#E0E7FF] text-[#3730A3]"
                                  : "bg-[#DCFCE7] text-[#166534]"
                              }`}
                            >
                              {cont.scope}
                            </span>
                          </div>
                          <div className="font-mono text-[10.5px] text-[#6B7280]">
                            {cont.port} • {cont.memory}
                          </div>
                        </div>
                      </div>

                      <div className="text-right">
                        <span className="font-pixel text-[8.5px] text-[#10B981]">
                          {cont.status}
                        </span>
                      </div>
                    </div>
                  );
                })}
              </div>

              {/* Inspector Details Panel for Selected Container */}
              <div className="border-t border-[#E5E7EB] bg-[#FAFAFA] p-3.5 font-mono text-[11px]">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-[#6B7280] font-medium">INSPECTOR:</span>
                  <span className="font-semibold text-[#111111]">{selectedDetail.name}</span>
                </div>

                <div className="space-y-2 text-[10.5px]">
                  <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-0.5">
                    <span className="text-[#6B7280] shrink-0">Health Probe:</span>
                    <span className="text-[#059669] font-medium text-left sm:text-right break-all">
                      {selectedDetail.probe}
                    </span>
                  </div>

                  <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-0.5">
                    <span className="text-[#6B7280] shrink-0">Storage Volume:</span>
                    <span className="text-[#1F2937] text-left sm:text-right break-all">
                      {selectedDetail.volume}
                    </span>
                  </div>

                  <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-0.5">
                    <span className="text-[#6B7280] shrink-0">Security Scope:</span>
                    <span className="text-[#1F2937] text-left sm:text-right">
                      Loopback Only (0 Public Ports)
                    </span>
                  </div>
                </div>
              </div>

              {/* Status Footer */}
              <div className="flex items-center justify-between border-t border-[#E5E7EB] bg-[#F9FAFB] px-4 py-2 font-mono text-[10px] text-[#6B7280]">
                <span>TOKEN: 256-bit Bearer (Verified)</span>
                <span className="font-pixel text-[8.5px] text-[#10B981]">● IDEMPOTENT DAG</span>
              </div>
            </div>
          </div>

        </div>

      </div>
    </div>
  );
}
