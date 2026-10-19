"use client";

import React, { useState, useEffect } from "react";
import {
  Sparkles,
  Terminal,
  Clock,
  Code2,
  FileText,
  CheckCircle2,
  Play,
  Pause,
  RotateCcw,
  ChevronDown,
  ChevronRight,
  Database,
  HardDrive,
  FileCode
} from "lucide-react";
import { DockerIcon } from "./BrandIcons";

export function HarnessCapabilities() {
  // --- CARD 1 STATE: POMODORO & ADAPTER TOGGLES ---
  const [timerSeconds, setTimerSeconds] = useState(25 * 60);
  const [timerRunning, setTimerRunning] = useState(false);
  const [timerMode, setTimerMode] = useState<"focus" | "break">("focus");

  useEffect(() => {
    let interval: any = null;
    if (timerRunning && timerSeconds > 0) {
      interval = setInterval(() => {
        setTimerSeconds((prev) => prev - 1);
      }, 1000);
    } else if (timerSeconds === 0) {
      setTimerRunning(false);
    }
    return () => clearInterval(interval);
  }, [timerRunning, timerSeconds]);

  const formatTimer = (sec: number) => {
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  };

  const [adapters, setAdapters] = useState([
    { id: "pg", name: "PostgreSQL 16", desc: "Auto-provisioning & volume persist", active: true },
    { id: "redis", name: "Redis 7.2 Cache", desc: "In-memory cache & pub/sub", active: true },
    { id: "docker", name: "Docker Bridge", desc: "Isolated container supervisor", active: true },
    { id: "cron", name: "Scheduled tasks", desc: "Cron automations & health probes", active: true },
    { id: "systemd", name: "Systemd Daemon", desc: "Loopback host service watcher", active: false },
    { id: "pomo", name: "Pomodoro timer", desc: "Stay focused. Take a break.", active: true },
  ]);

  const toggleAdapter = (id: string) => {
    setAdapters((prev) =>
      prev.map((a) => (a.id === id ? { ...a, active: !a.active } : a))
    );
  };

  // --- CARD 2 STATE: FILE SELECTOR & CODE DIFF ---
  const [activeFile, setActiveFile] = useState<"ts" | "sql" | "compose" | "toml">("ts");

  const fileDiffs = {
    ts: {
      name: "src/label.ts",
      lang: "TypeScript code",
      added: 2,
      removed: 1,
      lines: [
        { numOld: 8, numNew: 8, type: "context", text: "function label(name: string) {" },
        { numOld: 9, numNew: null, type: "remove", text: "- return name;" },
        { numOld: null, numNew: 9, type: "add", text: "+ const text = name.trim();" },
        { numOld: null, numNew: 10, type: "add", text: "+ return text || 'Untitled';" },
        { numOld: 10, numNew: 11, type: "context", text: "}" },
      ]
    },
    sql: {
      name: "migrations/001_init.sql",
      lang: "PostgreSQL schema",
      added: 3,
      removed: 0,
      lines: [
        { numOld: 1, numNew: 1, type: "context", text: "CREATE TABLE IF NOT EXISTS sessions (" },
        { numOld: null, numNew: 2, type: "add", text: "+   id UUID PRIMARY KEY DEFAULT gen_random_uuid()," },
        { numOld: null, numNew: 3, type: "add", text: "+   trace_hash TEXT NOT NULL UNIQUE," },
        { numOld: null, numNew: 4, type: "add", text: "+   created_at TIMESTAMPTZ DEFAULT NOW()" },
        { numOld: 2, numNew: 5, type: "context", text: ");" },
      ]
    },
    compose: {
      name: "docker-compose.yml",
      lang: "Docker Compose",
      added: 2,
      removed: 1,
      lines: [
        { numOld: 12, numNew: 12, type: "context", text: "  postgres:" },
        { numOld: 13, numNew: null, type: "remove", text: "-   ports: [\"5432:5432\"]" },
        { numOld: null, numNew: 13, type: "add", text: "+   ports: [\"127.0.0.1:5432:5432\"]" },
        { numOld: null, numNew: 14, type: "add", text: "+   restart: unless-stopped" },
        { numOld: 14, numNew: 15, type: "context", text: "    volumes: [\"pgdata:/var/lib/postgresql/data\"]" },
      ]
    },
    toml: {
      name: "smoooth.toml",
      lang: "Runtime Config",
      added: 2,
      removed: 0,
      lines: [
        { numOld: 4, numNew: 4, type: "context", text: "[security]" },
        { numOld: null, numNew: 5, type: "add", text: "+ bind_host = \"127.0.0.1\"" },
        { numOld: null, numNew: 6, type: "add", text: "+ loopback_only = true" },
        { numOld: 5, numNew: 7, type: "context", text: "log_level = \"info\"" },
      ]
    }
  };

  // --- CARD 3 STATE: SCHEDULED AUTOMATIONS ---
  const [jsonDrawerOpen, setJsonDrawerOpen] = useState(true);

  // --- CARD 4 STATE: DEVELOPER TRACE & TIMELINE ---
  const [traceFilter, setTraceFilter] = useState<"duration" | "turns" | "calls">("duration");
  const [traceTab, setTraceTab] = useState<"summary" | "payload" | "result" | "timing">("summary");

  return (
    <section id="capabilities" className="py-24 max-w-[1240px] mx-auto px-4 sm:px-6 lg:px-8">
      {/* Section Title */}
      <div className="text-center max-w-3xl mx-auto mb-16">
        <h2 className="text-3xl sm:text-4xl font-normal tracking-[-0.02em] text-white font-sans">
          Expanding capabilities. Work on your terms.
        </h2>
      </div>

      {/* 2x2 Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-7">
        {/* ========================================================= */}
        {/* CARD 1: Everything is an adapter */}
        {/* ========================================================= */}
        <div className="ds-cap-card">
          <div className="pb-6">
            <h3 className="text-xl font-medium text-white font-sans">
              Everything is a plugin
            </h3>
            <p className="mt-2 text-sm text-[#9AA0A6] leading-relaxed">
              Install plugins, or create them through chat in “Creator mode”, to extend tools, skills, and the interface.
            </p>
          </div>

          {/* Media flush at bottom */}
          <div className="ds-cap-media p-5 space-y-4">
            {/* Creator Mode Chat Flow */}
            <div className="rounded-xl border border-white/[0.08] bg-[#0A0A0C] p-3.5 text-xs font-mono space-y-2.5">
              <div className="flex items-center justify-between text-[#81858c] pb-2 border-b border-white/[0.06]">
                <span className="text-blue-400 font-semibold uppercase tracking-wider text-[10px]">
                  Creator mode
                </span>
                <span className="text-[10px] text-emerald-400">Workspace write</span>
              </div>

              <div className="text-[#f9fafb]">
                <span className="text-[#81858c]">User: </span>
                <span>Write a Pomodoro timer plugin for me.</span>
              </div>

              <div className="space-y-1 pl-2 border-l border-blue-500/30 text-[#81858c] text-[11px]">
                <div className="text-blue-400">Deep diving...</div>
                <div>Load skill cordis-plugin-development</div>
                <div>Inspect providers · 9 inspect providers</div>
                <div className="text-emerald-400">
                  Write dsh-plugin-pomodoro-float/client.js <span className="text-emerald-400">+638</span> <span className="text-red-400">−0</span>
                </div>
                <div>Tool call plugin_manager · install_bundle</div>
                <div>Bash Verify live plugin state</div>
                <div className="text-[10px] text-slate-500">Took 5m 24s</div>
              </div>

              <div className="text-emerald-400 text-xs font-sans flex items-center gap-1.5 pt-0.5">
                <CheckCircle2 className="w-3.5 h-3.5" />
                <span>The floating Pomodoro timer is built, installed, and verified.</span>
              </div>
            </div>

            {/* Plugin Toggle List & Pomodoro Widget */}
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-1">
              {/* Adapters List */}
              <div className="space-y-1.5">
                <div className="text-[10px] font-mono text-slate-400 uppercase tracking-wider">
                  Installed Plugins
                </div>
                {adapters.slice(0, 4).map((a) => (
                  <div
                    key={a.id}
                    onClick={() => toggleAdapter(a.id)}
                    className="flex items-center justify-between p-2 rounded-lg bg-white/[0.03] border border-white/[0.05] cursor-pointer"
                  >
                    <span className="text-xs text-[#e7e9ec] truncate pr-2">{a.name}</span>
                    <div
                      className={`w-6 h-3.5 rounded-full transition-colors relative flex items-center shrink-0 ${
                        a.active ? "bg-blue-600" : "bg-slate-700"
                      }`}
                    >
                      <div
                        className={`w-2.5 h-2.5 rounded-full bg-white transition-transform ${
                          a.active ? "translate-x-3" : "translate-x-0.5"
                        }`}
                      />
                    </div>
                  </div>
                ))}
              </div>

              {/* Pomodoro Timer */}
              <div className="rounded-xl border border-white/[0.08] bg-[#0A0A0C] p-3 text-center flex flex-col justify-between">
                <div>
                  <div className="flex items-center justify-center gap-2 mb-2">
                    <button
                      onClick={() => {
                        setTimerMode("focus");
                        setTimerSeconds(25 * 60);
                        setTimerRunning(false);
                      }}
                      className={`text-[11px] px-2 py-0.5 rounded transition ${
                        timerMode === "focus" ? "bg-white/[0.12] text-white" : "text-slate-400"
                      }`}
                    >
                      Focus
                    </button>
                    <button
                      onClick={() => {
                        setTimerMode("break");
                        setTimerSeconds(5 * 60);
                        setTimerRunning(false);
                      }}
                      className={`text-[11px] px-2 py-0.5 rounded transition ${
                        timerMode === "break" ? "bg-white/[0.12] text-white" : "text-slate-400"
                      }`}
                    >
                      Short break
                    </button>
                  </div>
                  <div className="text-3xl font-mono font-bold text-white tracking-wider my-2">
                    {formatTimer(timerSeconds)}
                  </div>
                </div>

                <button
                  onClick={() => setTimerRunning(!timerRunning)}
                  className="w-full py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-xs font-medium transition active:scale-95 flex items-center justify-center gap-1.5"
                >
                  {timerRunning ? <Pause className="w-3.5 h-3.5" /> : <Play className="w-3.5 h-3.5" />}
                  <span>{timerRunning ? "Pause" : "Start focus"}</span>
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* ========================================================= */}
        {/* CARD 2: Complete a range of tasks */}
        {/* ========================================================= */}
        <div className="ds-cap-card">
          <div className="pb-6">
            <h3 className="text-xl font-medium text-white font-sans">
              Complete a range of tasks
            </h3>
            <p className="mt-2 text-sm text-[#9AA0A6] leading-relaxed">
              Organize documents, analyze spreadsheets, and write code. Preview the results directly and refine them through conversation.
            </p>
          </div>

          {/* Media flush at bottom */}
          <div className="ds-cap-media p-5 space-y-3">
            {/* File tabs */}
            <div className="flex items-center gap-2 overflow-x-auto pb-1 scrollbar-none">
              <button
                onClick={() => setActiveFile("ts")}
                className={`flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono transition shrink-0 ${
                  activeFile === "ts"
                    ? "bg-blue-500/20 border border-blue-500/40 text-blue-400 font-medium"
                    : "bg-white/[0.04] text-slate-400"
                }`}
              >
                <FileCode className="w-3.5 h-3.5" />
                <span>label.ts</span>
              </button>
              <button
                onClick={() => setActiveFile("sql")}
                className={`flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono transition shrink-0 ${
                  activeFile === "sql"
                    ? "bg-blue-500/20 border border-blue-500/40 text-blue-400 font-medium"
                    : "bg-white/[0.04] text-slate-400"
                }`}
              >
                <Database className="w-3.5 h-3.5" />
                <span>schema.sql</span>
              </button>
              <button
                onClick={() => setActiveFile("compose")}
                className={`flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono transition shrink-0 ${
                  activeFile === "compose"
                    ? "bg-blue-500/20 border border-blue-500/40 text-blue-400 font-medium"
                    : "bg-white/[0.04] text-slate-400"
                }`}
              >
                <DockerIcon className="w-3.5 h-3.5" />
                <span>docker-compose.yml</span>
              </button>
            </div>

            {/* Code Diff Panel */}
            <div className="rounded-xl border border-white/[0.08] bg-[#0A0A0C] overflow-hidden">
              <div className="px-4 py-2 bg-[#101012] border-b border-white/[0.06] flex items-center justify-between text-xs font-mono">
                <div className="flex items-center gap-2 text-slate-300">
                  <span className="text-slate-500">Review · turn 1</span>
                  <span className="text-slate-600">/</span>
                  <span className="text-white">{fileDiffs[activeFile].name}</span>
                </div>
                <div className="flex items-center gap-1.5 text-[11px]">
                  <span className="text-emerald-400">+{fileDiffs[activeFile].added}</span>
                  <span className="text-red-400">−{fileDiffs[activeFile].removed}</span>
                </div>
              </div>

              <div className="p-3 font-mono text-xs space-y-1">
                <div className="text-slate-500 text-[10px] pb-1">@@ -8,3 +8,4 @@</div>
                {fileDiffs[activeFile].lines.map((line, idx) => (
                  <div
                    key={idx}
                    className={`flex items-center px-2 py-0.5 rounded ${
                      line.type === "add"
                        ? "bg-[#1f3124] text-[#41c977]"
                        : line.type === "remove"
                        ? "bg-[#331c1e] text-[#f87171] line-through opacity-80"
                        : "text-[#81858c]"
                    }`}
                  >
                    <div className="w-7 select-none text-[10px] text-slate-500">{line.numOld || " "}</div>
                    <div className="w-7 select-none text-[10px] text-slate-500">{line.numNew || " "}</div>
                    <div className="w-4 select-none font-bold">
                      {line.type === "add" ? "+" : line.type === "remove" ? "−" : " "}
                    </div>
                    <div className="whitespace-pre">{line.text.replace(/^[+-]\s*/, "")}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* ========================================================= */}
        {/* CARD 3: Adapt to your workflow */}
        {/* ========================================================= */}
        <div className="ds-cap-card">
          <div className="pb-6">
            <h3 className="text-xl font-medium text-white font-sans">
              Adapt to your workflow
            </h3>
            <p className="mt-2 text-sm text-[#9AA0A6] leading-relaxed">
              Enable the Scheduled tasks plugin to start tasks anytime and run recurring work on schedule. Stay on top of progress and inspect tool call details when needed.
            </p>
          </div>

          {/* Media flush at bottom */}
          <div className="ds-cap-media p-5 space-y-3 font-mono text-xs">
            <div className="text-slate-200">
              <span className="text-slate-500">User: </span>
              <span>Every Friday at 17:00 Beijing time, summarize this week's project notes in a weekly report.</span>
            </div>

            <div className="flex items-center gap-2 text-[11px] text-slate-400">
              <span className="text-blue-400">Deep diving...</span>
              <span>Worked</span>
            </div>

            <div className="inline-flex items-center gap-2 px-2.5 py-1 rounded bg-white/[0.04] border border-white/[0.08] text-slate-300 text-xs">
              <Clock className="w-3.5 h-3.5 text-blue-400" />
              <span>Create reminder · Weekly project report</span>
            </div>

            {/* JSON Drawer */}
            <div className="rounded-lg border border-white/[0.06] bg-[#0A0A0C] overflow-hidden">
              <button
                onClick={() => setJsonDrawerOpen(!jsonDrawerOpen)}
                className="w-full px-3 py-1.5 bg-[#101012] flex items-center justify-between text-[11px] text-slate-400"
              >
                <div className="flex items-center gap-2">
                  <span className="text-slate-500 uppercase">IN</span>
                  <span className="text-blue-300">schedule_create</span>
                </div>
                {jsonDrawerOpen ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
              </button>

              {jsonDrawerOpen && (
                <pre className="p-3 text-[11px] text-slate-300 overflow-x-auto leading-relaxed">
{`{
  "prompt": "Summarize this week's project notes in a weekly report.",
  "title": "Weekly project report",
  "weekly": {
    "time": "17:00:00",
    "time_zone": "Asia/Shanghai",
    "weekdays": [5]
  }
}`}
                </pre>
              )}
            </div>

            {/* Resulting Task Card */}
            <div className="p-3 rounded-xl border border-white/[0.08] bg-[#101012] flex items-center justify-between font-sans">
              <div>
                <div className="text-xs font-semibold text-white">Weekly project report</div>
                <div className="text-[11px] text-slate-400">Weekly on Fri at 17:00 (Asia/Shanghai)</div>
              </div>
              <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-[10px] font-mono">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                Scheduled
              </span>
            </div>
          </div>
        </div>

        {/* ========================================================= */}
        {/* CARD 4: Developer tools */}
        {/* ========================================================= */}
        <div className="ds-cap-card">
          <div className="pb-6">
            <h3 className="text-xl font-medium text-white font-sans">
              Developer tools
            </h3>
            <p className="mt-2 text-sm text-[#9AA0A6] leading-relaxed">
              Inspect execution traces and detailed runtime information to troubleshoot issues with tool calls and task execution.
            </p>
          </div>

          {/* Media flush at bottom */}
          <div className="ds-cap-media p-5 space-y-3 font-mono text-xs">
            {/* Filter buttons */}
            <div className="flex items-center justify-between text-xs">
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setTraceFilter("duration")}
                  className={`px-2.5 py-1 rounded text-[11px] transition ${
                    traceFilter === "duration" ? "bg-white/[0.12] text-white" : "text-slate-400"
                  }`}
                >
                  Duration
                </button>
                <button
                  onClick={() => setTraceFilter("turns")}
                  className={`px-2.5 py-1 rounded text-[11px] transition ${
                    traceFilter === "turns" ? "bg-white/[0.12] text-white" : "text-slate-400"
                  }`}
                >
                  Turns
                </button>
                <button
                  onClick={() => setTraceFilter("calls")}
                  className={`px-2.5 py-1 rounded text-[11px] transition ${
                    traceFilter === "calls" ? "bg-white/[0.12] text-white" : "text-slate-400"
                  }`}
                >
                  Calls
                </button>
              </div>
              <span className="text-[11px] text-slate-500">Total: 142 ms</span>
            </div>

            {/* Timeline Lanes */}
            <div className="space-y-1.5 py-1">
              <div className="flex items-center gap-3">
                <span className="w-10 text-[10px] text-slate-500 uppercase">Input</span>
                <div className="flex-1 h-3 rounded bg-blue-500/20 border border-blue-500/40 relative">
                  <span className="absolute inset-0 flex items-center px-2 text-[9px] text-blue-300">
                    User Task & Context
                  </span>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <span className="w-10 text-[10px] text-slate-500 uppercase">Model</span>
                <div className="flex-1 h-3 rounded bg-purple-500/20 border border-purple-500/40 relative">
                  <span className="absolute inset-0 flex items-center px-2 text-[9px] text-purple-300">
                    Reasoning & Plan (114ms)
                  </span>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <span className="w-10 text-[10px] text-slate-500 uppercase">Tools</span>
                <div className="flex-1 h-3 rounded bg-emerald-500/20 border border-emerald-500/40 relative">
                  <span className="absolute inset-0 flex items-center px-2 text-[9px] text-emerald-300">
                    bash: navigation verify (12ms)
                  </span>
                </div>
              </div>
            </div>

            {/* Terminal Inspect */}
            <div className="rounded-xl border border-white/[0.08] bg-[#0A0A0C] p-3 text-[11px] space-y-1">
              <div className="text-slate-400">Command: <span className="text-white">bash navigation_verify.sh</span></div>
              <div className="text-slate-400">Exit code: <span className="text-emerald-400">0</span></div>
              <div className="text-emerald-400 pt-1">NAVIGATION_OK</div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
