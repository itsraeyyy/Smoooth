"use client";

import React, { useState } from "react";
import Link from "next/link";

export function HarnessHeader() {
  const [lang, setLang] = useState<"EN" | "CN">("EN");

  return (
    <div className="fixed top-0 left-0 right-0 z-50 w-full px-6 pt-3 pointer-events-none">
      <div className="max-w-[1280px] mx-auto flex items-center justify-between pointer-events-auto">
        {/* Brand */}
        <Link href="/" className="flex items-center gap-2.5 group">
          {/* Logo mark */}
          <div className="w-6 h-6 rounded-md bg-white flex items-center justify-center text-black font-bold text-xs tracking-tighter">
            S
          </div>
          <span className="text-white font-semibold text-[17px] tracking-tight font-sans">
            smoooth
          </span>
          <span className="px-1.5 py-0.5 rounded text-[10px] font-mono tracking-wider text-black bg-white uppercase font-bold">
            RUNTIME
          </span>
        </Link>

        {/* Right: Language Switcher Pill */}
        <div className="flex items-center">
          <div className="flex items-center h-8 p-[3px] rounded-full border border-white/[0.12] bg-white/[0.04] text-xs font-sans">
            <button
              onClick={() => setLang("CN")}
              className={`px-2.5 py-0.5 rounded-full transition-all ${
                lang === "CN"
                  ? "bg-white/[0.22] text-white shadow-sm font-medium"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              中文
            </button>
            <button
              onClick={() => setLang("EN")}
              className={`px-2.5 py-0.5 rounded-full transition-all ${
                lang === "EN"
                  ? "bg-white/[0.22] text-white shadow-sm font-medium"
                  : "text-slate-400 hover:text-white"
              }`}
            >
              EN
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
