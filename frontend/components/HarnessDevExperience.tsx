"use client";

import React, { useState } from "react";
import { Terminal, Copy, Check, GitBranch } from "lucide-react";

export function HarnessDevExperience() {
  const [copiedCmd, setCopiedCmd] = useState<string | null>(null);

  const copyToClipboard = (text: string, id: string) => {
    navigator.clipboard.writeText(text);
    setCopiedCmd(id);
    setTimeout(() => setCopiedCmd(null), 2000);
  };

  return (
    <section id="quickstart" className="py-20 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      {/* Section Heading */}
      <div className="text-center max-w-3xl mx-auto mb-12">
        <h2 className="text-3xl sm:text-4xl font-bold tracking-tight text-white font-headline">
          Developer experience
        </h2>
        <p className="mt-3 text-base text-slate-300">
          Get started in seconds with our verified installer or build directly from source.
        </p>
      </div>

      {/* Two Quickstart Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-5xl mx-auto">
        {/* Card 1: Start with one command */}
        <div className="ds-card rounded-2xl p-6 sm:p-7 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 text-blue-400 mb-2">
              <Terminal className="w-4 h-4" />
              <span className="text-xs font-mono uppercase tracking-wider">Quickstart</span>
            </div>
            <h3 className="text-lg font-bold text-white font-headline">
              Start with one command
            </h3>
            <p className="mt-2 text-sm text-slate-300 leading-relaxed">
              Install the Smoooth agent runtime daemon on any Linux VPS or server and connect over loopback.
            </p>
          </div>

          <div className="mt-6 rounded-xl border border-white/[0.08] bg-[#070B14] p-3 flex items-center justify-between font-mono text-xs">
            <span className="text-slate-300 truncate select-all pr-2">
              $ curl -sSL https://smoooth.dev/install.sh | bash
            </span>
            <button
              onClick={() =>
                copyToClipboard("curl -sSL https://smoooth.dev/install.sh | bash", "curl")
              }
              className="p-1.5 rounded-lg bg-white/[0.05] hover:bg-white/[0.1] text-slate-300 hover:text-white transition shrink-0"
              title="Copy command"
            >
              {copiedCmd === "curl" ? (
                <Check className="w-3.5 h-3.5 text-emerald-400" />
              ) : (
                <Copy className="w-3.5 h-3.5" />
              )}
            </button>
          </div>
        </div>

        {/* Card 2: Install from source */}
        <div className="ds-card rounded-2xl p-6 sm:p-7 flex flex-col justify-between">
          <div>
            <div className="flex items-center gap-2 text-emerald-400 mb-2">
              <GitBranch className="w-4 h-4" />
              <span className="text-xs font-mono uppercase tracking-wider">Open Source</span>
            </div>
            <h3 className="text-lg font-bold text-white font-headline">
              Install from source
            </h3>
            <p className="mt-2 text-sm text-slate-300 leading-relaxed">
              Clone the full repository, inspect the Rust daemon core, and launch in developer mode.
            </p>
          </div>

          <div className="mt-6 rounded-xl border border-white/[0.08] bg-[#070B14] p-3 flex items-center justify-between font-mono text-xs">
            <span className="text-slate-300 truncate select-all pr-2">
              $ git clone https://github.com/smoooth-dev/smoooth
            </span>
            <button
              onClick={() =>
                copyToClipboard("git clone https://github.com/smoooth-dev/smoooth", "git")
              }
              className="p-1.5 rounded-lg bg-white/[0.05] hover:bg-white/[0.1] text-slate-300 hover:text-white transition shrink-0"
              title="Copy command"
            >
              {copiedCmd === "git" ? (
                <Check className="w-3.5 h-3.5 text-emerald-400" />
              ) : (
                <Copy className="w-3.5 h-3.5" />
              )}
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}
