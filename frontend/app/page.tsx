"use client";

import React, { useState, useEffect } from "react";
import {
  Copy,
  Check,
} from "lucide-react";

export default function Home() {
  const [activeBugTab, setActiveBugTab] = useState<"HTML" | "CSS" | "JavaScript">("HTML");
  const [activeSuggestTab, setActiveSuggestTab] = useState<"HTML" | "CSS" | "JavaScript">("HTML");
  const [copiedSuggest, setCopiedSuggest] = useState(false);
  const [backendStatus, setBackendStatus] = useState<"connecting" | "connected" | "offline">("connecting");

  // Check Rust backend connectivity on port 8080
  useEffect(() => {
    fetch("http://127.0.0.1:8080/health")
      .then((res) => res.json())
      .then((data) => {
        if (data.status === "ok") {
          setBackendStatus("connected");
        }
      })
      .catch(() => {
        setBackendStatus("offline");
      });
  }, []);

  const copyCode = (text: string, setFn: (v: boolean) => void) => {
    navigator.clipboard?.writeText(text);
    setFn(true);
    setTimeout(() => setFn(false), 2000);
  };

  return (
    <div className="min-h-screen bg-white font-sans text-[#111111] antialiased selection:bg-[#111111] selection:text-white">
      {/* ===================================================================
          GRID CONTAINER (Framed by crisp hairline borders matching reference)
          =================================================================== */}
      <div className="mx-auto max-w-[1240px] border-x border-[#E5E7EB] bg-white shadow-[0_0_50px_rgba(0,0,0,0.02)]">
        {/* ===================================================================
            1. NAVIGATION BAR
            =================================================================== */}
        <header className="flex h-[64px] items-center justify-between border-b border-[#E5E7EB] px-8 bg-white">
          {/* Left: Brand Wordmark */}
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1.5 text-[16px] font-semibold tracking-tight text-[#111111]">
              <span className="text-[11px] leading-none text-[#111111]">▼</span>
              <span>Vorcel AI</span>
            </div>
            {backendStatus === "connected" && (
              <span className="hidden sm:inline-flex items-center gap-1 rounded border border-[#E5E7EB] bg-[#F9FAFB] px-2 py-0.5 text-[10px] font-mono font-medium text-[#10B981]">
                <span className="h-1.5 w-1.5 rounded-full bg-[#10B981] animate-pulse" />
                RUST BACKEND ON
              </span>
            )}
          </div>

          {/* Center: Navigation Links */}
          <nav className="hidden md:flex items-center gap-8 text-[14px] text-[#4B5563]">
            <a href="#home" className="text-[#111111] font-medium transition-colors">
              Home
            </a>
            <a href="#features" className="hover:text-[#111111] transition-colors">
              Features
            </a>
            <a href="#product" className="hover:text-[#111111] transition-colors">
              Product
            </a>
            <a href="#support" className="hover:text-[#111111] transition-colors">
              Support
            </a>
            <a href="#pricing" className="hover:text-[#111111] transition-colors">
              Pricing
            </a>
          </nav>

          {/* Right: Actions */}
          <div className="flex items-center gap-3.5">
            <button
              type="button"
              className="px-3 py-1.5 text-[14px] font-medium text-[#111111] hover:text-[#4B5563] transition-colors active:scale-[0.98]"
            >
              Login
            </button>
            <button
              type="button"
              className="rounded-[8px] bg-[#ECEEF1] px-4 py-2 text-[13px] font-medium text-[#111111] hover:bg-[#E2E5E9] active:scale-[0.97] transition-all shadow-[0_1px_2px_rgba(0,0,0,0.04)]"
            >
              Create account
            </button>
          </div>
        </header>

        {/* ===================================================================
            2. HERO SECTION — ROW 1: EDITORIAL HEADLINE & TRUSTED BY
            (Subdivided by exact 51.67% / 48.33% hairline vertical divider)
            =================================================================== */}
        <div className="flex flex-col lg:flex-row border-b border-[#E5E7EB]">
          {/* Left Hero Column (Exactly 51.67% width to align with visual divider) */}
          <div className="w-full lg:w-[51.67%] border-b lg:border-b-0 lg:border-r border-[#E5E7EB] px-8 sm:px-12 pt-10 pb-10 flex flex-col justify-between">
            <div>
              {/* Monospace Eyebrow */}
              <div className="flex items-center gap-2 mb-6">
                <span className="h-2 w-2 rounded-full bg-[#10B981]" />
                <span className="font-mono text-[11px] font-semibold tracking-[0.16em] uppercase text-[#111827]">
                  WE JUST RAISED $ 2.1 MILLION PER SEED
                </span>
              </div>

              {/* High-Contrast Editorial Serif Headline */}
              <h1 className="font-serif text-[48px] sm:text-[56px] lg:text-[60px] font-normal leading-[1.04] tracking-[-0.015em] text-[#111111]">
                AI-Powered Coding
                <br />
                Without the Complexity
              </h1>
            </div>

            {/* Action Buttons */}
            <div className="mt-8 flex flex-wrap items-center gap-3.5">
              <button
                type="button"
                className="rounded-[8px] bg-[#18181B] px-5 py-2.5 text-[14px] font-medium text-white shadow-sm hover:bg-black active:scale-[0.97] transition-all"
              >
                Start Free Trial
              </button>
              <button
                type="button"
                className="rounded-[8px] border border-[#E5E7EB] bg-white px-5 py-2.5 text-[14px] font-medium text-[#111827] shadow-[0_1px_2px_rgba(0,0,0,0.03)] hover:bg-[#F9FAFB] active:scale-[0.97] transition-all"
              >
                Try Live Demo
              </button>
            </div>
          </div>

          {/* Right Hero Column (Exactly 48.33% width) */}
          <div className="w-full lg:w-[48.33%] px-8 sm:px-10 pt-10 pb-10 flex flex-col justify-between">
            {/* Supporting Copy */}
            <p className="text-[15px] leading-[1.65] text-[#4B5563] max-w-[440px]">
              Accelerate development workflows with AI-powered code generation,
              debugging, documentation, and intelligent suggestions built
              directly into your coding environment.
            </p>

            {/* Trusted By Section (Muted monochrome styling matching reference) */}
            <div className="mt-12 lg:mt-0">
              <div className="mb-2 text-right">
                <span className="font-mono text-[10px] tracking-[0.15em] uppercase text-[#9CA3AF]">
                  TRUSTED BY:
                </span>
              </div>
              <div className="flex items-center justify-between gap-4 text-[#6B7280]">
                {/* Calendly */}
                <div className="flex items-center gap-1.5 font-bold text-[13px] tracking-tight text-[#4B5563]">
                  <span className="h-2.5 w-2.5 rounded-full bg-[#9CA3AF]" />
                  <span>Calendly</span>
                </div>
                {/* classpass */}
                <div className="flex items-center gap-1.5 font-bold text-[13px] tracking-tight text-[#4B5563]">
                  <span className="text-[14px] leading-none">⊙</span>
                  <span>classpass</span>
                </div>
                {/* GitHub */}
                <div className="flex items-center gap-1.5 font-bold text-[13px] tracking-tight text-[#4B5563]">
                  <svg className="h-4 w-4 fill-[#4B5563]" viewBox="0 0 24 24">
                    <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
                  </svg>
                  <span>GitHub</span>
                </div>
                {/* Basecamp */}
                <div className="flex items-center gap-1.5 font-bold text-[13px] tracking-tight text-[#4B5563]">
                  <span className="text-[13px] leading-none">▲</span>
                  <span>Basecamp</span>
                </div>
                {/* otter */}
                <div className="flex items-center gap-1.5 font-bold text-[13px] tracking-tight text-[#4B5563]">
                  <span className="text-[13px] leading-none">●</span>
                  <span>otter</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* ===================================================================
            3. HERO SECTION — ROW 2: TECHNICAL PRODUCT COMPOSITION (MOUNTAIN)
            (100% Genuine Stipple Mountain Artwork, Code Window & iPhone Mockup)
            =================================================================== */}
        <div className="relative border-b border-[#E5E7EB] bg-white overflow-hidden">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img
            src="/hero_visual_2x.png"
            alt="Vorcel AI Platform Preview"
            className="w-full h-auto block select-none pointer-events-none"
          />
        </div>

        {/* ===================================================================
            4. SECTION 2: EVERYTHING DEVELOPERS NEED TO MOVE FASTER
            =================================================================== */}
        <section className="px-8 sm:px-12 py-16 bg-white" id="features">
          {/* Eyebrow & Editorial Heading Row */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-end">
            <div className="lg:col-span-7">
              <div className="flex items-center gap-2 mb-4">
                <span className="h-2 w-2 rounded-full bg-[#10B981]" />
                <span className="font-mono text-[11px] font-semibold tracking-[0.16em] uppercase text-[#111827]">
                  WHY CHOSE KODA AI
                </span>
              </div>
              <h2 className="font-serif text-[38px] sm:text-[46px] font-normal leading-[1.08] tracking-[-0.015em] text-[#111111]">
                Everything Developers
                <br />
                Need to Move Faster
              </h2>
            </div>
            <div className="lg:col-span-5 pb-2">
              <p className="text-[14px] leading-[1.65] text-[#6B7280] max-w-[420px]">
                Powerful AI tools designed to streamline coding, reduce repetitive
                work, and help developers build with greater speed and confidence.
              </p>
            </div>
          </div>

          {/* Two Substantial Feature Cards */}
          <div className="mt-12 grid grid-cols-1 md:grid-cols-2 gap-8">
            {/* Feature Card 1: Instant Bug Detection */}
            <div className="rounded-[18px] border border-[#E5E7EB] bg-white p-6 sm:p-7 shadow-[0_2px_8px_rgba(0,0,0,0.02)] flex flex-col justify-between">
              <div>
                {/* Bug Icon Tile */}
                <div className="flex h-9 w-9 items-center justify-center rounded-[8px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <svg className="h-4 w-4 fill-current" viewBox="0 0 24 24">
                    <path d="M19 8h-1.81a5.985 5.985 0 00-1.82-1.96l.93-.93a.996.996 0 10-1.41-1.41l-1.47 1.47C12.83 5.06 12.42 5 12 5s-.83.06-1.42.17L9.11 3.7a.996.996 0 10-1.41 1.41l.93.93C7.65 6.64 6.98 7.27 6.81 8H5c-.55 0-1 .45-1 1s.45 1 1 1h1.09c-.05.33-.09.66-.09 1v1H5c-.55 0-1 .45-1 1s.45 1 1 1h1v1c0 .34.04.67.09 1H5c-.55 0-1 .45-1 1s.45 1 1 1h1.81c1.04 1.79 2.97 3 5.19 3s4.15-1.21 5.19-3H19c.55 0 1-.45 1-1s-.45-1-1-1h-1.09c.05-.33.09-.66.09-1v-1h1c.55 0 1-.45 1-1s-.45-1-1-1h-1v-1c0-.34-.04-.67-.09-1H19c.55 0 1-.45 1-1s-.45-1-1-1zm-6 8h-2c-.55 0-1-.45-1-1s.45-1 1-1h2c.55 0 1 .45 1 1s-.45 1-1 1zm0-4h-2c-.55 0-1-.45-1-1s.45-1 1-1h2c.55 0 1 .45 1 1s-.45 1-1 1z" />
                  </svg>
                </div>

                <h3 className="mt-4 text-[17px] font-bold text-[#111111]">
                  Instant Bug Detection
                </h3>
                <p className="mt-1 text-[13px] text-[#6B7280] leading-[1.5]">
                  Identify issues before deployment with intelligent error
                  analysis and fixes.
                </p>
              </div>

              {/* Card 1 Visual: Code Editor Inside Soft Grey Well */}
              <div className="mt-6 rounded-[14px] border border-[#E5E7EB] bg-[#F9FAFB] p-4">
                <div className="rounded-[10px] border border-[#E5E7EB] bg-white shadow-sm overflow-hidden">
                  {/* Tab Bar with Issue Pill & Solved Pill */}
                  <div className="flex h-[38px] items-center justify-between border-b border-[#E5E7EB] bg-[#FAFAFA] px-3.5">
                    <div className="flex items-center gap-1.5">
                      <button
                        type="button"
                        onClick={() => setActiveBugTab("HTML")}
                        className="rounded-[5px] bg-[#E5E7EB] px-2.5 py-0.5 text-[11px] font-semibold text-[#111111]"
                      >
                        HTML
                      </button>
                      <button
                        type="button"
                        onClick={() => setActiveBugTab("CSS")}
                        className="rounded-[5px] px-2.5 py-0.5 text-[11px] font-semibold text-[#9CA3AF] hover:text-[#111111]"
                      >
                        CSS
                      </button>
                      <button
                        type="button"
                        onClick={() => setActiveBugTab("JavaScript")}
                        className="rounded-[5px] px-2.5 py-0.5 text-[11px] font-semibold text-[#9CA3AF] hover:text-[#111111]"
                      >
                        Java Script
                      </button>
                    </div>

                    <div className="flex items-center gap-2">
                      <span className="inline-flex items-center gap-1 rounded-full bg-[#FEE2E2] px-2 py-0.5 text-[9.5px] font-semibold text-[#DC2626]">
                        <span className="h-1.5 w-1.5 rounded-full bg-[#DC2626]" />
                        2 Issue Found
                      </span>
                      <span className="rounded-[5px] bg-[#1F2937] px-2 py-0.5 text-[9.5px] font-semibold text-white">
                        Solved Code
                      </span>
                    </div>
                  </div>

                  {/* Code Body with Red Error Highlight Lines */}
                  <div className="p-4 font-mono text-[11px] leading-[1.65] text-[#1F2937] bg-white">
                    <pre>
                      <code>
                        <span className="text-[#D1D5DB] select-none mr-3">1</span>&lt;!DOCTYPE html&gt;{"\n"}
                        <span className="text-[#D1D5DB] select-none mr-3">2</span>&lt;html&gt;{"\n"}
                        <span className="text-[#D1D5DB] select-none mr-3">3</span>&lt;head&gt;{"\n"}
                        <span className="text-[#D1D5DB] select-none mr-3">4</span>  &lt;title&gt;Hello, World!&lt;/title&gt;{"\n"}
                        <span className="inline-block w-full bg-[#FEE2E2] text-[#DC2626] px-1.5 -mx-1.5 rounded font-medium">
                          <span className="text-[#F87171] select-none mr-3">5</span>&lt;link rel=&quot;stylesheet&quot; href=&quot;styles.css&quot; /&gt;
                        </span>{"\n"}
                        <span className="text-[#D1D5DB] select-none mr-3">6</span>&lt;/head&gt;{"\n"}
                        <span className="text-[#D1D5DB] select-none mr-3">7</span>&lt;body&gt;{"\n"}
                        <span className="inline-block w-full bg-[#FEE2E2] text-[#DC2626] px-1.5 -mx-1.5 rounded font-medium">
                          <span className="text-[#F87171] select-none mr-3">8</span>  &lt;h1 class=&quot;title&quot;&gt;Hello World!&lt;/h1&gt;
                        </span>
                      </code>
                    </pre>
                  </div>
                </div>
              </div>
            </div>

            {/* Feature Card 2: Context-Aware Suggestions */}
            <div className="rounded-[18px] border border-[#E5E7EB] bg-white p-6 sm:p-7 shadow-[0_2px_8px_rgba(0,0,0,0.02)] flex flex-col justify-between">
              <div>
                {/* Lightbulb Icon Tile */}
                <div className="flex h-9 w-9 items-center justify-center rounded-[8px] border border-[#E5E7EB] bg-[#F9FAFB] text-[#111111]">
                  <svg className="h-4 w-4 fill-none stroke-current stroke-2" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" d="M12 2v1m0 18v1m9-10h-1M4 12H3m15.364-6.364l-.707.707M6.343 17.657l-.707.707m12.728 0l-.707-.707M6.343 6.343l-.707-.707M9 21h6m-4-3h2a5 5 0 005-5 5 5 0 00-10 0 5 5 0 003 4.583V18z" />
                  </svg>
                </div>

                <h3 className="mt-4 text-[17px] font-bold text-[#111111]">
                  Context-Aware Suggestions
                </h3>
                <p className="mt-1 text-[13px] text-[#6B7280] leading-[1.5]">
                  Receive coding recommendations tailored to your project
                  structure, patterns, and best pra...
                </p>
              </div>

              {/* Card 2 Visual: Code Editor Inside Soft Grey Well */}
              <div className="mt-6 rounded-[14px] border border-[#E5E7EB] bg-[#F9FAFB] p-4">
                <div className="rounded-[10px] border border-[#E5E7EB] bg-white shadow-sm overflow-hidden">
                  {/* Tab Bar with Copy Button */}
                  <div className="flex h-[38px] items-center justify-between border-b border-[#E5E7EB] bg-[#FAFAFA] px-3.5">
                    <div className="flex items-center gap-1.5">
                      <button
                        type="button"
                        onClick={() => setActiveSuggestTab("HTML")}
                        className="rounded-[5px] bg-[#E5E7EB] px-2.5 py-0.5 text-[11px] font-semibold text-[#111111]"
                      >
                        HTML
                      </button>
                      <button
                        type="button"
                        onClick={() => setActiveSuggestTab("CSS")}
                        className="rounded-[5px] px-2.5 py-0.5 text-[11px] font-semibold text-[#9CA3AF] hover:text-[#111111]"
                      >
                        CSS
                      </button>
                      <button
                        type="button"
                        onClick={() => setActiveSuggestTab("JavaScript")}
                        className="rounded-[5px] px-2.5 py-0.5 text-[11px] font-semibold text-[#9CA3AF] hover:text-[#111111]"
                      >
                        Java Script
                      </button>
                    </div>

                    <button
                      type="button"
                      onClick={() => copyCode("user = get_user(user_id)\nif user.", setCopiedSuggest)}
                      className="flex items-center gap-1 text-[11px] font-medium text-[#6B7280] hover:text-[#111111] transition-colors"
                    >
                      {copiedSuggest ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
                      <span>{copiedSuggest ? "Copied" : "Copy"}</span>
                    </button>
                  </div>

                  {/* Code Body with Fully Visible Autocomplete Menu */}
                  <div className="relative p-4 pb-6 font-mono text-[11px] leading-[1.65] text-[#1F2937] bg-white min-h-[195px]">
                    <div>
                      <span className="text-[#D1D5DB] select-none mr-3">11</span>
                      <span className="text-[#4B5563]">user</span> = <span className="text-[#2563EB]">get_user</span>(user_id)
                    </div>
                    <div className="mt-1">
                      <span className="text-[#D1D5DB] select-none mr-3">12</span>
                      <span className="text-[#7C3AED] font-semibold">if</span> user.
                    </div>

                    {/* Autocomplete Dropdown Popover */}
                    <div className="absolute left-[88px] top-[46px] w-[184px] rounded-[8px] border border-[#E5E7EB] bg-white shadow-xl overflow-hidden z-20">
                      <div className="bg-[#F0F4F8] px-3 py-1.5 text-[10px] font-medium text-[#1E293B]">
                        &gt; is_active
                      </div>
                      <div className="px-3 py-1.5 text-[10px] text-[#64748B]">
                        &gt; is_authenticated
                      </div>
                      <div className="px-3 py-1.5 text-[10px] text-[#64748B]">
                        &gt; is_admin
                      </div>
                      <div className="px-3 py-1.5 text-[10px] text-[#64748B]">
                        &gt; get_permission
                      </div>

                      {/* AI Suggestion Tab Pill at Bottom */}
                      <div className="flex items-center justify-between bg-[#111827] px-3 py-1 text-[9.5px] text-white">
                        <span>AI Suggestion</span>
                        <span className="rounded bg-white px-1.5 py-0.2 text-[8.5px] font-mono font-bold text-[#111827]">
                          Tab
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
            5. SUBTLE FOOTER
            =================================================================== */}
        <footer className="border-t border-[#E5E7EB] bg-white px-8 py-8">
          <div className="flex flex-col sm:flex-row items-center justify-between gap-4 text-[13px] text-[#6B7280]">
            <div className="flex items-center gap-2">
              <span className="text-[11px] leading-none text-[#111111]">▼</span>
              <span className="font-semibold text-[#111111]">Vorcel AI</span>
              <span className="text-[#D1D5DB]">/</span>
              <span>Smoooth 1:1 Pixel Clone with Next.js &amp; Rust</span>
            </div>

            <div className="flex items-center gap-6">
              <a href="#privacy" className="hover:text-[#111111] transition-colors">
                Privacy
              </a>
              <a href="#terms" className="hover:text-[#111111] transition-colors">
                Terms
              </a>
              <a href="#docs" className="hover:text-[#111111] transition-colors">
                Docs
              </a>
              <a href="#github" className="hover:text-[#111111] transition-colors">
                GitHub
              </a>
            </div>
          </div>
        </footer>
      </div>
    </div>
  );
}
