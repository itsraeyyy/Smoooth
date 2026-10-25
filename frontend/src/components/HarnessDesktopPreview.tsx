"use client";

import React, { useState } from "react";
import {
  ChevronDown,
  ChevronRight,
  Plus,
  Search,
  SlidersHorizontal,
  FolderPlus,
  Clock,
  Sparkles,
  Paperclip,
  ArrowUp,
  ShieldCheck,
  HardDrive,
  Database,
  Layers,
  Lock,
  Minus,
  Maximize2,
  X,
  Sidebar as SidebarIcon
} from "lucide-react";

export function HarnessDesktopPreview() {
  const [thoughtOpen, setThoughtOpen] = useState(false);
  const [selectedSession, setSelectedSession] = useState("Meet Smoooth");
  const [inputVal, setInputVal] = useState("");

  const sessions = [
    { id: "Meet Smoooth", name: "Meet Smoooth" },
    { id: "Q3 summary slides", name: "Q3 summary slides" },
    { id: "Review prep", name: "Review prep" },
    { id: "Sales report", name: "Sales report" },
  ];

  return (
    <div className="ds-window mx-auto">
      {/* Frosted Sidebar */}
      <aside className="ds-sidebar p-3 justify-between">
        <div>
          {/* Traffic Lights & Sidebar Toggle */}
          <div className="h-9 px-1 flex items-center justify-between">
            <div className="flex items-center gap-[7px]">
              <i className="w-3 h-3 rounded-full bg-[#ff6058] shadow-[inset_0_0_0_0.5px_rgba(0,0,0,0.1)] inline-block" />
              <i className="w-3 h-3 rounded-full bg-[#ffbd2e] shadow-[inset_0_0_0_0.5px_rgba(0,0,0,0.1)] inline-block" />
              <i className="w-3 h-3 rounded-full bg-[#28c840] shadow-[inset_0_0_0_0.5px_rgba(0,0,0,0.1)] inline-block" />
            </div>
            <SidebarIcon className="w-3.5 h-3.5 text-[#adb2b8] hover:text-white cursor-pointer" />
          </div>

          {/* Brand in Sidebar */}
          <div className="h-9 px-1 mb-2 flex items-center gap-2">
            <div className="w-5 h-5 rounded bg-white flex items-center justify-center text-black font-bold text-[10px]">
              S
            </div>
            <span className="text-xs font-semibold text-white tracking-tight">smoooth</span>
            <span className="text-[9px] font-mono font-bold text-black bg-white px-1.5 py-0.2 rounded uppercase ml-auto">
              HARNESS
            </span>
          </div>

          {/* New Session Button */}
          <button
            onClick={() => setSelectedSession("Meet Smoooth")}
            className="w-full h-[34px] rounded-[10px] bg-white/[0.08] hover:bg-white/[0.12] shadow-[inset_0_0_0_0.5px_rgba(255,255,255,0.08)] flex items-center justify-center gap-1.5 text-xs text-[#e7e9ec] font-medium transition mb-2"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>New Session</span>
          </button>

          {/* Navigation Items */}
          <div className="space-y-0.5">
            <div className="h-8 px-2 flex items-center justify-between text-xs text-[#e7e9ec] hover:bg-white/[0.04] rounded-lg cursor-pointer">
              <div className="flex items-center gap-2">
                <Sparkles className="w-3.5 h-3.5 text-blue-400" />
                <span>Adapters</span>
              </div>
              <span className="text-[10px] font-mono text-slate-400">7</span>
            </div>
            <div className="h-8 px-2 flex items-center justify-between text-xs text-[#e7e9ec] hover:bg-white/[0.04] rounded-lg cursor-pointer">
              <div className="flex items-center gap-2">
                <Clock className="w-3.5 h-3.5 text-slate-400" />
                <span>Automation</span>
              </div>
              <span className="text-[10px] font-mono text-slate-400">2</span>
            </div>
          </div>

          {/* Workspace Section */}
          <div className="mt-3 pt-2">
            <div className="h-7 px-2 flex items-center justify-between text-[11px] text-[#adb2b8]">
              <span>Workspace</span>
              <div className="flex items-center gap-2.5">
                <Search className="w-3 h-3 hover:text-white cursor-pointer" />
                <SlidersHorizontal className="w-3 h-3 hover:text-white cursor-pointer" />
                <FolderPlus className="w-3 h-3 hover:text-white cursor-pointer" />
              </div>
            </div>

            {/* Expanded Folder: Default Workspace */}
            <div className="mt-1">
              <div className="h-7 px-2 flex items-center gap-1.5 text-xs text-[#e7e9ec] font-medium cursor-pointer">
                <span className="text-[10px] text-blue-400">▾</span>
                <span className="text-blue-300">Default workspace</span>
              </div>

              {/* Session Rows */}
              <div className="pl-4 space-y-0.5 mt-0.5">
                {sessions.map((s) => (
                  <div
                    key={s.id}
                    onClick={() => setSelectedSession(s.id)}
                    className={`h-[30px] px-2 flex items-center text-xs rounded-[7px] cursor-pointer transition ${
                      selectedSession === s.id
                        ? "bg-white/[0.08] text-white font-medium"
                        : "text-[#adb2b8] hover:text-white hover:bg-white/[0.03]"
                    }`}
                  >
                    <span className="truncate">{s.name}</span>
                  </div>
                ))}
              </div>

              {/* Collapsed Folders */}
              <div className="mt-2 space-y-0.5">
                <div className="h-7 px-2 flex items-center gap-1.5 text-xs text-[#adb2b8] hover:text-white cursor-pointer">
                  <span className="text-[10px] text-slate-500">▸</span>
                  <span>Data analysis</span>
                </div>
                <div className="h-7 px-2 flex items-center gap-1.5 text-xs text-[#adb2b8] hover:text-white cursor-pointer">
                  <span className="text-[10px] text-slate-500">▸</span>
                  <span>DSH adapter dev</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Bottom Account Row */}
        <div className="h-10 px-2 flex items-center gap-2 border-t border-white/[0.06] pt-2">
          <div className="w-6 h-6 rounded-full bg-white/[0.08] flex items-center justify-center text-[11px] text-white font-mono">
            v
          </div>
          <span className="text-xs text-[#e7e9ec] font-medium truncate">vps-root@smoooth</span>
          <div className="w-1.5 h-1.5 rounded-full bg-emerald-400 ml-auto" />
        </div>
      </aside>

      {/* Main Chat Area */}
      <main className="flex-1 bg-[#151517] flex flex-col justify-between overflow-hidden">
        {/* Top Window Bar */}
        <div className="h-11 px-5 flex items-center justify-between text-xs text-[#cfd3d6] shadow-[inset_0_-0.5px_hsla(0,0%,100%,0.1)]">
          <span className="font-medium text-white">{selectedSession}</span>
          <div className="flex items-center gap-3 text-slate-400">
            <Minus className="w-3.5 h-3.5 hover:text-white cursor-pointer" />
            <Maximize2 className="w-3 h-3 hover:text-white cursor-pointer" />
            <X className="w-3.5 h-3.5 hover:text-white cursor-pointer" />
          </div>
        </div>

        {/* Transcript Body */}
        <div className="p-6 overflow-y-auto max-h-[460px] space-y-5">
          {/* User Message Bubble */}
          <div className="flex justify-end">
            <div className="max-w-[70%] bg-[#2c2c2e] text-[#f9fafb] text-[13px] leading-[20px] px-4 py-2.5 rounded-[20px] shadow-sm">
              Could you introduce yourself?
            </div>
          </div>

          {/* Thought for a while Disclosure */}
          <div className="max-w-[760px]">
            <button
              onClick={() => setThoughtOpen(!thoughtOpen)}
              className="w-full flex items-center justify-between text-xs text-[#adb2b8] hover:text-white pb-2 mb-2 border-b border-white/[0.12] transition"
            >
              <div className="flex items-center gap-2">
                <span>Thought for a while</span>
              </div>
              {thoughtOpen ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
            </button>

            {thoughtOpen && (
              <div className="p-3 mb-3 rounded-lg bg-[#0E0E10] font-mono text-[11px] text-slate-400 space-y-1">
                <div>→ Probed loopback 127.0.0.1 security barrier</div>
                <div>→ Verified Docker daemon socket /var/run/docker.sock</div>
                <div className="text-blue-400">→ Loaded 7 active MCP adapters (PostgreSQL, Redis, Mongo, SQLite)</div>
                <div className="text-emerald-400">✓ DAG topological sort complete: zero cycles</div>
              </div>
            )}

            {/* Assistant Reply Body */}
            <div className="text-[13px] leading-[22px] text-[#f9fafb] space-y-3">
              <p>
                I'm <strong className="font-semibold text-white">Smoooth</strong>, an open-source harness built on Model Context Protocol (MCP) and deterministic 2-phase DAG execution. Run me as a desktop app or connect Claude Code directly over loopback.
              </p>

              <p className="font-medium text-white pt-1">I can help with:</p>

              <ol className="list-decimal pl-5 space-y-1.5 text-[#e7e9ec]">
                <li>
                  <strong className="font-semibold text-white">Everyday work</strong> — Organize files, analyze databases, draft schema migrations, and generate reports.
                </li>
                <li>
                  <strong className="font-semibold text-white">Coding</strong> — Explore repos, fix bugs, deploy Docker containers, and run end-to-end tests.
                </li>
                <li>
                  <strong className="font-semibold text-white">Research</strong> — Inspect server logs, audit security configurations, and benchmark queries.
                </li>
                <li>
                  <strong className="font-semibold text-white">Background tasks</strong> — Run cron scripts, batch database backups, and track progress.
                </li>
                <li>
                  <strong className="font-semibold text-white">Adapters</strong> — Add or build custom MCP adapters to fit your infrastructure.
                </li>
              </ol>

              <p className="text-slate-400 text-xs italic pt-1">And so much more...</p>
            </div>
          </div>
        </div>

        {/* Composer Dock at Bottom */}
        <div className="p-4 pt-0">
          <div className="max-w-[760px] mx-auto p-2 rounded-[20px] bg-[#2c2c2e] shadow-[0_0_0_0.5px_hsla(0,0%,100%,0.12),0_4px_16px_rgba(0,0,0,0.06)]">
            <input
              type="text"
              value={inputVal}
              onChange={(e) => setInputVal(e.target.value)}
              placeholder="Describe what you want to build, / commands, @ files or sessions"
              className="w-full bg-transparent px-3 py-1.5 text-[13px] text-white placeholder:text-[#81858c] focus:outline-none"
            />

            {/* Composer Toolbar */}
            <div className="flex items-center justify-between px-2 pt-1.5 text-xs text-[#cfd3d6]">
              {/* Left */}
              <div className="flex items-center gap-2">
                <div className="w-[25px] h-[25px] rounded-full bg-[#353638] flex items-center justify-center cursor-pointer hover:bg-slate-600 transition">
                  <Plus className="w-3.5 h-3.5 text-white" />
                </div>
                <div className="flex items-center gap-1.5 text-[11px] text-[#adb2b8]">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
                  <span>Workspace Write</span>
                </div>
              </div>

              {/* Right */}
              <div className="flex items-center gap-2">
                <div className="flex items-center gap-1 text-[11px] text-slate-300 bg-white/[0.04] px-2 py-0.5 rounded cursor-pointer">
                  <span>Claude 3.7 Sonnet</span>
                  <ChevronDown className="w-3 h-3 text-slate-400" />
                </div>
                <button
                  onClick={() => {
                    if (!inputVal) setInputVal("Deploy PostgreSQL 16 on 127.0.0.1:5432");
                  }}
                  className="w-[30px] h-[30px] rounded-full bg-[rgba(122,170,255,0.4)] hover:bg-blue-500 text-white flex items-center justify-center transition active:scale-95"
                >
                  <ArrowUp className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}
