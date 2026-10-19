"use client";

import React from "react";
import Link from "next/link";
import { Github, ExternalLink, ShieldCheck, Terminal, Heart } from "lucide-react";

export function HarnessFooter() {
  return (
    <footer className="border-t border-white/[0.08] bg-[#060910] text-slate-400">
      {/* Ecosystem CTA Section */}
      <div className="py-24 max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <h2 className="text-3xl sm:text-4xl lg:text-5xl font-bold tracking-tight text-white font-headline">
          Join the Smoooth ecosystem
        </h2>
        <p className="mt-4 text-base text-slate-300 max-w-2xl mx-auto leading-relaxed">
          Smoooth is in public preview, with composable MCP adapters and loopback-first isolation. We invite developers worldwide to build deterministic agent infrastructure together.
        </p>

        {/* Action Buttons */}
        <div className="mt-8 flex flex-wrap items-center justify-center gap-4">
          <a
            href="https://github.com/smoooth-dev/smoooth"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 bg-white text-black hover:bg-slate-200 text-sm font-medium px-5 py-2.5 rounded-xl transition-all shadow-md active:scale-95"
          >
            <Github className="w-4 h-4" />
            <span>View on GitHub</span>
          </a>

          <Link
            href="/docs"
            className="flex items-center gap-2 bg-white/[0.06] hover:bg-white/[0.1] border border-white/[0.1] text-white text-sm font-medium px-5 py-2.5 rounded-xl transition-all active:scale-95"
          >
            <span>Developer docs</span>
          </Link>

          <a
            href="#capabilities"
            className="flex items-center gap-2 bg-white/[0.06] hover:bg-white/[0.1] border border-white/[0.1] text-white text-sm font-medium px-5 py-2.5 rounded-xl transition-all active:scale-95"
          >
            <span>Community adapters</span>
          </a>
        </div>
      </div>

      {/* Main Footer Links & Copyright */}
      <div className="border-t border-white/[0.06] py-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col md:flex-row items-center justify-between gap-6 text-xs">
        {/* Brand & Copyright */}
        <div className="flex flex-col sm:flex-row items-center gap-3 text-slate-400">
          <div className="flex items-center gap-2">
            <div className="w-5 h-5 rounded bg-blue-600 flex items-center justify-center text-[10px] text-white font-bold">
              S
            </div>
            <span className="font-semibold text-white tracking-tight">smoooth</span>
          </div>
          <span>© 2026 Smoooth. All rights reserved.</span>
          <span className="hidden sm:inline text-slate-400">•</span>
          <span className="font-mono text-slate-400">Open source · Apache-2.0 & MIT</span>
        </div>

        {/* Policy Links */}
        <div className="flex flex-wrap items-center justify-center gap-5 text-slate-400">
          <Link href="/safe-use" className="hover:text-white transition-colors">
            Safe Use Policy
          </Link>
          <Link href="/privacy" className="hover:text-white transition-colors">
            Privacy Policy
          </Link>
          <Link href="/terms" className="hover:text-white transition-colors">
            Terms of Use
          </Link>
          <a
            href="https://github.com/smoooth-dev/smoooth"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-white transition-colors flex items-center gap-1"
          >
            <span>GitHub</span>
            <ExternalLink className="w-3 h-3 text-slate-400" />
          </a>
        </div>
      </div>
    </footer>
  );
}
