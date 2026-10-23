const fs = require('fs');
const path = require('path');

const targetFile = path.resolve(__dirname, '../frontend/app/exact_harness.html');
let html = fs.readFileSync(targetFile, 'utf8');

// 1. Mobile Menu Header & Nav: replace DeepSeek logo and fake desktop downloads
const oldMobileHeaderStart = html.indexOf('<div id="harness-mobile-menu"');
const oldMobileHeaderEnd = html.indexOf('<main class="relative flex flex-col w-full">');
if (oldMobileHeaderStart === -1 || oldMobileHeaderEnd === -1) {
  throw new Error('Could not find mobile menu boundaries');
}

const newMobileMenu = `<div id="harness-mobile-menu" class="ds-mobile-menu" aria-hidden="true" style="visibility:hidden">
  <div class="ds-mobile-menu-header">
    <a class="flex items-center gap-[10px] text-white" href="/">
      <svg width="24" height="24" viewBox="0 0 28 28" fill="none" class="shrink-0">
        <rect width="28" height="28" rx="7" fill="#1c3c66" />
        <path d="M7 14C7 10.134 10.134 7 14 7C17.866 7 21 10.134 21 14C21 17.866 17.866 21 14 21" stroke="#6799fe" stroke-width="2.5" stroke-linecap="round"/>
        <circle cx="14" cy="14" r="3.5" fill="#ffffff" />
      </svg>
      <span class="font-bold text-[18px] tracking-tight text-white font-['Host_Grotesk']">Smoooth</span>
      <span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-mono font-medium bg-white/10 text-[#73a3d2] border border-white/10">v0.1.0</span>
    </a>
    <button class="flex items-center justify-center w-10 h-10 text-ds-primary" aria-label="Close menu">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none"><path d="M6 6l12 12M6 18L18 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"></path></svg>
    </button>
  </div>
  <nav class="ds-mobile-menu-body">
    <a href="#features" class="ds-mobile-menu-item">Features</a>
    <a href="#how-it-works" class="ds-mobile-menu-item">How It Works</a>
    <a href="#migration" class="ds-mobile-menu-item">Migration</a>
    <a href="#faq" class="ds-mobile-menu-item">Guarantees &amp; FAQ</a>
    <a href="#quickstart" class="ds-mobile-menu-item">Install v0.1.0</a>
    <a href="https://github.com/itsraeyyy/Smoooth" target="_blank" rel="noopener noreferrer" class="ds-mobile-menu-item flex items-center justify-between">
      <span>GitHub</span>
      <span class="text-[11px] font-mono text-white/40">&rarr;</span>
    </a>
    <a href="https://github.com/itsraeyyy/Smoooth/tree/main/docs" target="_blank" rel="noopener noreferrer" class="ds-mobile-menu-item flex items-center justify-between">
      <span>Documentation</span>
      <span class="text-[11px] font-mono text-white/40">&rarr;</span>
    </a>
  </nav>
  <div class="p-ds-5 border-t border-white/10 text-[12px] font-mono text-white/40">
    Daemon: Linux x86_64 / ARM64 · MCP v1.0
  </div>
</div>`;

html = html.slice(0, oldMobileHeaderStart) + newMobileMenu + html.slice(oldMobileHeaderEnd);

// 2. Remove Eyebrow badge / Status dot pill above H1 (Strict Anti-Slop Rule)
const heroKickerRegex = /<span data-hero-preview-label="true"[\s\S]*?<\/span><\/span>/;
if (heroKickerRegex.test(html)) {
  html = html.replace(heroKickerRegex, '');
  console.log('✓ Removed eyebrow kicker pill above H1');
}

// 3. Hero CTA: Dual CTAs (Primary: Install on Linux VPS, Secondary: See MCP Workflow)
const oldHeroDownloadRegex = /<div class="relative inline-flex items-center">[\s\S]*?<div id="hero-download-dropdown"[\s\S]*?<\/div><\/div><\/div><\/div><div class="ds-hero-preview-enter/;
const newHeroCtaHtml = `<div class="flex flex-wrap items-center justify-center gap-3">
  <div class="relative inline-flex items-center">
    <div class="HarnessDownloadMenu_split__148cc ds-btn-primary rounded-full shadow-[0_8px_30px_rgb(0,0,0,0.45)]">
      <a class="ds-btn-primary ds-btn-m HarnessDownloadMenu_action__Qe9nK !rounded-r-none !pr-ds-3 flex items-center gap-ds-2" href="#quickstart">
        <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M8 2V11M8 11L4.5 7.5M8 11L11.5 7.5M2.5 13.5H13.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        <span>Install on Linux VPS</span>
      </a>
      <button type="button" class="HarnessDownloadMenu_arrow__60_9L ds-btn-primary ds-btn-m !rounded-l-none !pl-ds-3 !pr-[14px]" aria-label="Quick setup options" aria-haspopup="menu" aria-expanded="false">
        <span aria-hidden="true" class="HarnessDownloadMenu_divider__Dao4X pointer-events-none absolute left-0 border-l border-[var(--dsw-alias-border-l2)]"></span>
        <span aria-hidden="true" class="HarnessDownloadMenu_arrowIcon__lFxE3">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M3 5.5L7 9.5L11 5.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>
        </span>
      </button>
    </div>
    <div id="hero-download-dropdown" class="ds-dropdown-menu absolute top-[calc(100%+8px)] left-1/2 -translate-x-1/2 min-w-[340px] p-2 bg-[#1b1c20]/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl z-50 transition-all duration-200" style="display:none;opacity:0;transform:translate(-50%, -6px)">
      <div class="px-3 py-2 border-b border-white/10 mb-1 flex items-center justify-between">
        <span class="text-[11px] font-mono uppercase tracking-wider text-[#73a3d2]">Quick Setup Commands</span>
        <span id="quick-copy-status" class="text-[10px] font-mono text-[#59ca84]" style="display:none">Copied!</span>
      </div>
      <button type="button" class="w-full flex flex-col gap-0.5 px-3 py-2 text-left rounded-xl hover:bg-white/10 transition-colors cursor-pointer group" onclick="navigator.clipboard.writeText('curl -fsSL https://smoooth.dev/install.sh | bash'); const s = document.getElementById('quick-copy-status'); if(s){ s.style.display='inline'; setTimeout(()=>{ s.style.display='none'; }, 2000); }">
        <div class="flex items-center justify-between">
          <span class="font-medium text-[13px] text-white">Ubuntu 24.04 / Debian 12</span>
          <span class="text-[10px] font-mono text-white/40 group-hover:text-white/70">Click to Copy</span>
        </div>
        <span class="text-[11px] font-mono text-[#73a3d2]">curl -fsSL https://smoooth.dev/install.sh | bash</span>
      </button>
      <button type="button" class="w-full flex flex-col gap-0.5 px-3 py-2 text-left rounded-xl hover:bg-white/10 transition-colors cursor-pointer group" onclick="navigator.clipboard.writeText('claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse'); const s = document.getElementById('quick-copy-status'); if(s){ s.style.display='inline'; setTimeout(()=>{ s.style.display='none'; }, 2000); }">
        <div class="flex items-center justify-between">
          <span class="font-medium text-[13px] text-white">Claude Code MCP Pairing</span>
          <span class="text-[10px] font-mono text-white/40 group-hover:text-white/70">Click to Copy</span>
        </div>
        <span class="text-[11px] font-mono text-[#73a3d2]">claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse</span>
      </button>
      <a class="flex flex-col gap-0.5 px-3 py-2 text-left rounded-xl hover:bg-white/10 transition-colors" href="https://github.com/itsraeyyy/Smoooth" target="_blank" rel="noopener noreferrer">
        <span class="font-medium text-[13px] text-white">View on GitHub (v0.1.0 Beta)</span>
        <span class="text-[11px] text-white/50">Full open-source daemon, docs & release notes</span>
      </a>
    </div>
  </div>
  <a class="ds-btn-secondary ds-btn-m px-4 py-2.5 rounded-full border border-white/20 text-white hover:bg-white/10 hover:border-white/30 transition-all text-[13px] font-medium flex items-center gap-2" href="#how-it-works">
    <span>See MCP Workflow</span>
    <svg width="13" height="13" viewBox="0 0 16 16" fill="none"><path d="M6 12L10 8L6 4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>
  </a>
</div></div></div></div><div class="ds-hero-preview-enter`;

html = html.replace(oldHeroDownloadRegex, newHeroCtaHtml);

// 4. Hero Desktop Preview Sidebar cleanup (remove leftover DeepSeek internal folder labels)
html = html.replace('<span>DSH plugin dev</span>', '<span>MCP Server Config</span>');
html = html.replace('<span>Data analysis</span>', '<span>Stack: nextjs-web</span>');

// 5. Replace Capabilities Section with 4 Real Technical Proof Cards
const featuresStart = html.indexOf('<section id="features"');
const featuresEnd = html.indexOf('</section><section id="how-it-works"');
if (featuresStart === -1 || featuresEnd === -1) {
  throw new Error('Could not find features section boundaries');
}

const newFeaturesSection = `<section id="features" class="ds-container pt-ds-11 pb-ds-11 scroll-mt-[80px]" aria-labelledby="harness-capabilities-title">
  <div class="HarnessCapabilitiesSection_entrance__EZAWd max-w-[820px] mx-auto flex flex-col items-center gap-ds-7 text-center">
    <h2 id="harness-capabilities-title" class="harness-heading ds-text-heading1 text-ds-primary">
      <span class="harness-heading-latin">Autonomous</span> <span class="harness-heading-latin">runtime</span>. <br class="hidden md:block"/>
      <span class="harness-heading-latin text-white">Infrastructure</span> <span class="harness-heading-latin text-white">you</span> <span class="harness-heading-latin text-white">control</span>.
    </h2>
    <p class="ds-text-body text-ds-description max-w-[650px]">
      Purpose-built for coding agents. Built-in stateful adapters, scoped security boundaries, transactional DAG rollbacks, and real-time MCP verification.
    </p>
  </div>
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-ds-6 mt-ds-9">
    <!-- Card 1: Stateful Database Adapters -->
    <article class="HarnessCapabilitiesSection_card__Yf6o5 HarnessCapabilitiesSection_entrance__EZAWd">
      <div class="HarnessCapabilitiesSection_copy__l0rC4">
        <h3 class="harness-heading ds-text-subtitle text-ds-primary">
          <span class="harness-heading-latin">Stateful</span> <span class="harness-heading-latin">Database</span> <span class="harness-heading-latin">Adapters</span>
        </h3>
        <p class="ds-text-body text-ds-description">
          Provision PostgreSQL, Redis, MongoDB, and SQLite with automated volume retention, staged credential isolation, and health verification.
        </p>
      </div>
      <div class="p-5 bg-[#0a0f18] rounded-xl border border-white/10 font-mono text-[12px] shadow-inner">
        <div class="flex items-center justify-between pb-2 border-b border-white/10 mb-3 text-white/50 text-[11px]">
          <span class="flex items-center gap-2"><span class="w-2 h-2 rounded-full bg-[#59ca84]"></span><span>adapter // postgres:16</span></span>
          <span class="text-[#73a3d2]">active_lease</span>
        </div>
        <div class="space-y-2 leading-relaxed text-white/80">
          <div><span class="text-[#6799fe]">volume_mount:</span> <span class="text-white">/var/lib/smoooth/volumes/pgdata</span></div>
          <div><span class="text-[#6799fe]">health_probe:</span> <span class="text-[#59ca84]">pg_isready -h 127.0.0.1 -p 5432 -U postgres</span> <span class="text-white/40">(12ms)</span></div>
          <div><span class="text-[#6799fe]">credential_scope:</span> <span class="text-white">staged_rotation</span> <span class="text-white/40">[dual_auth: active]</span></div>
          <div><span class="text-[#6799fe]">retention_policy:</span> <span class="text-white">snapshot_before_destroy (safety_lock: ON)</span></div>
          <div class="pt-2 border-t border-white/5 text-[#59ca84] text-[11px] flex items-center gap-1.5">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M3 8.5L6.5 12L13 4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
            Database container isolated on private bridge network
          </div>
        </div>
      </div>
    </article>

    <!-- Card 2: Scoped Security & Least Privilege -->
    <article class="HarnessCapabilitiesSection_card__Yf6o5 HarnessCapabilitiesSection_entrance__EZAWd">
      <div class="HarnessCapabilitiesSection_copy__l0rC4">
        <h3 class="harness-heading ds-text-subtitle text-ds-primary">
          <span class="harness-heading-latin">Scoped</span> <span class="harness-heading-latin">Security</span> <span class="harness-heading-latin">&amp;</span> <span class="harness-heading-latin">Least</span> <span class="harness-heading-latin">Privilege</span>
        </h3>
        <p class="ds-text-body text-ds-description">
          Never grant an AI agent an unrestricted root shell. Enforce 256-bit cryptographic bearer tokens with granular role permissions (read_only, plan, deploy, admin).
        </p>
      </div>
      <div class="p-5 bg-[#0a0f18] rounded-xl border border-white/10 font-mono text-[12px] shadow-inner">
        <div class="flex items-center justify-between pb-2 border-b border-white/10 mb-3 text-white/50 text-[11px]">
          <span class="flex items-center gap-2"><span class="w-2 h-2 rounded-full bg-[#6799fe]"></span><span>auth // token_guard</span></span>
          <span class="text-[#59ca84]">sha256_hashed</span>
        </div>
        <div class="space-y-2 leading-relaxed text-white/80">
          <div><span class="text-[#6799fe]">token_id:</span> <span class="text-white">smoooth_sec_9f82c4... [256-bit]</span></div>
          <div><span class="text-[#6799fe]">granted_scopes:</span> <span class="text-white">["read_only", "plan", "deploy"]</span></div>
          <div><span class="text-[#6799fe]">prohibited:</span> <span class="text-[#f87171]">["host_root_shell", "arbitrary_write"]</span></div>
          <div><span class="text-[#6799fe]">sandboxing:</span> <span class="text-white">systemd (NoNewPrivileges=yes, ProtectSystem=strict)</span></div>
          <div><span class="text-[#6799fe]">bind_address:</span> <span class="text-white">127.0.0.1:4040</span> <span class="text-white/40">(loopback private)</span></div>
          <div class="pt-2 border-t border-white/5 text-[#59ca84] text-[11px] flex items-center gap-1.5">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M3 8.5L6.5 12L13 4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
            Agent confined to scoped workspace boundaries
          </div>
        </div>
      </div>
    </article>

    <!-- Card 3: Transactional 2-Phase DAG Deployments -->
    <article class="HarnessCapabilitiesSection_card__Yf6o5 HarnessCapabilitiesSection_entrance__EZAWd">
      <div class="HarnessCapabilitiesSection_copy__l0rC4">
        <h3 class="harness-heading ds-text-subtitle text-ds-primary">
          <span class="harness-heading-latin">Transactional</span> <span class="harness-heading-latin">2-Phase</span> <span class="harness-heading-latin">DAG</span> <span class="harness-heading-latin">Deployments</span>
        </h3>
        <p class="ds-text-body text-ds-description">
          Every deployment validates an execution plan before provisioning. If a container fails health checks, Smoooth rolls back automatically to the last healthy state.
        </p>
      </div>
      <div class="p-5 bg-[#0a0f18] rounded-xl border border-white/10 font-mono text-[12px] shadow-inner">
        <div class="flex items-center justify-between pb-2 border-b border-white/10 mb-3 text-white/50 text-[11px]">
          <span class="flex items-center gap-2"><span class="w-2 h-2 rounded-full bg-[#f59e0b]"></span><span>dag // 2-phase_pipeline</span></span>
          <span class="text-white/40">rollback_ready</span>
        </div>
        <div class="space-y-2 leading-relaxed text-white/80">
          <div><span class="text-[#73a3d2]">01 PLAN &rarr;</span> <span class="text-white">create_plan(web_app:v1.2, postgres:16)</span></div>
          <div class="pl-3 text-white/60">&bull; Canonical hash: <span class="text-[#6799fe]">sha256:7f8a92e104cd...</span></div>
          <div><span class="text-[#73a3d2]">02 PROVISION &rarr;</span> <span class="text-white">execute_plan(fencing_lease: acquired)</span></div>
          <div class="pl-3 text-white/60">&bull; Health probes: HTTP /healthz [200 OK, 14ms]</div>
          <div class="pl-3 text-white/60">&bull; Safe promotion: Caddy reverse proxy updated</div>
          <div class="pt-2 border-t border-white/5 text-[#59ca84] text-[11px] flex items-center gap-1.5">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M3 8.5L6.5 12L13 4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
            Checkpoint committed. Zero downtime release active.
          </div>
        </div>
      </div>
    </article>

    <!-- Card 4: Model Context Protocol (MCP) Stream -->
    <article class="HarnessCapabilitiesSection_card__Yf6o5 HarnessCapabilitiesSection_entrance__EZAWd">
      <div class="HarnessCapabilitiesSection_copy__l0rC4">
        <h3 class="harness-heading ds-text-subtitle text-ds-primary">
          <span class="harness-heading-latin">Model</span> <span class="harness-heading-latin">Context</span> <span class="harness-heading-latin">Protocol</span> <span class="harness-heading-latin">(MCP)</span> <span class="harness-heading-latin">Stream</span>
        </h3>
        <p class="ds-text-body text-ds-description">
          Inspect topological DAG steps, container log streams, and environment health probes directly from your agent conversation or terminal.
        </p>
      </div>
      <div class="p-5 bg-[#0a0f18] rounded-xl border border-white/10 font-mono text-[12px] shadow-inner">
        <div class="flex items-center justify-between pb-2 border-b border-white/10 mb-3 text-white/50 text-[11px]">
          <span class="flex items-center gap-2"><span class="w-2 h-2 rounded-full bg-[#6799fe]"></span><span>audit // mcp_stream</span></span>
          <span class="text-white/40">append_only</span>
        </div>
        <div class="space-y-2 leading-relaxed text-white/80">
          <div><span class="text-white/40">[14:32:01]</span> <span class="text-[#73a3d2]">mcp:call</span> &rarr; <span class="text-white">scan_foreign_workloads()</span></div>
          <div class="pl-3 text-white/60">&rarr; 3 running Docker containers adopted into workspace</div>
          <div><span class="text-white/40">[14:32:03]</span> <span class="text-[#73a3d2]">mcp:call</span> &rarr; <span class="text-white">create_service_binding(app, db)</span></div>
          <div class="pl-3 text-white/60">&rarr; Staged credentials injected into container env</div>
          <div><span class="text-white/40">[14:32:04]</span> <span class="text-[#73a3d2]">mcp:call</span> &rarr; <span class="text-white">configure_ingress(domain: "api.site.dev")</span></div>
          <div class="pl-3 text-white/60">&rarr; Automatic ACME TLS certificate issued</div>
          <div class="pt-2 border-t border-white/5 text-[#59ca84] text-[11px] flex items-center gap-1.5">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M3 8.5L6.5 12L13 4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
            Signed audit entry recorded at /var/lib/smoooth/audit.jsonl
          </div>
        </div>
      </div>
    </article>
  </div>
</section>`;

html = html.slice(0, featuresStart) + newFeaturesSection + html.slice(featuresEnd);

// 6. Fix How It Works Step 2: replace git clone with actual agent pairing command
html = html.replace(
  '<code class="min-w-0 whitespace-pre-wrap break-all"><span class="select-none text-ds-brand">$ </span>git clone https://github.com/itsraeyyy/Smoooth</code>',
  '<code class="min-w-0 whitespace-pre-wrap break-all"><span class="select-none text-ds-brand">$ </span>claude mcp add --transport sse smoooth http://127.0.0.1:4040/sse</code>'
);

// 7. Remove invented brand names in Persona Panel (Nike -> internal portal)
html = html.replace(/--workspace client-nike/g, '--workspace client-portal');
html = html.replace(/nike-campaign\.com/g, 'portal.internal');

// 8. Add Technical Guarantees & FAQ Section right before #quickstart
const quickstartIndex = html.indexOf('<section id="quickstart"');
if (quickstartIndex === -1) {
  throw new Error('Could not find quickstart section');
}

const faqSectionHtml = `<section id="faq" class="ds-container py-ds-11 scroll-mt-[80px]">
  <div class="max-w-[820px] mx-auto mb-ds-8 text-center">
    <h2 class="harness-heading ds-text-heading1 text-ds-primary tracking-tight">
      <span class="harness-heading-latin">Technical Guarantees</span> <br class="hidden sm:inline"/>
      <span class="harness-heading-latin text-white">&amp; FAQ.</span>
    </h2>
    <p class="ds-text-body text-ds-description text-[16px] leading-relaxed mt-ds-3">
      Clear answers on compatibility, security boundaries, container orchestration, and real-world limitations.
    </p>
  </div>
  <div class="max-w-[820px] mx-auto flex flex-col gap-4">
    <details class="group rounded-xl border border-white/10 bg-white/[0.02] p-5 transition-all open:bg-white/[0.04]">
      <summary class="flex cursor-pointer items-center justify-between text-[16px] font-semibold text-white list-none">
        <span>How do AI coding agents like Claude Code or Cursor interact with Smoooth?</span>
        <svg class="w-4 h-4 text-white/50 transition-transform group-open:rotate-180" viewBox="0 0 16 16" fill="none"><path d="M4 6L8 10L12 6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>
      </summary>
      <div class="mt-3 text-[14px] text-ds-description leading-relaxed border-t border-white/5 pt-3">
        Smoooth implements the Model Context Protocol (MCP) standard. AI agents pair over stdio (spawning <code class="text-[#73a3d2] font-mono">smoooth mcp</code>) or streamable HTTP/SSE (<code class="text-[#73a3d2] font-mono">http://127.0.0.1:4040/sse</code>) using a 256-bit bearer token. Instead of running unsafe shell scripts as root, agents invoke structured, typed tools like <code class="text-[#73a3d2] font-mono">create_plan</code>, <code class="text-[#73a3d2] font-mono">execute_plan</code>, <code class="text-[#73a3d2] font-mono">verify_connection</code>, and <code class="text-[#73a3d2] font-mono">configure_ingress</code>.
      </div>
    </details>
    <details class="group rounded-xl border border-white/10 bg-white/[0.02] p-5 transition-all open:bg-white/[0.04]">
      <summary class="flex cursor-pointer items-center justify-between text-[16px] font-semibold text-white list-none">
        <span>Is giving an AI coding agent Docker daemon access secure?</span>
        <svg class="w-4 h-4 text-white/50 transition-transform group-open:rotate-180" viewBox="0 0 16 16" fill="none"><path d="M4 6L8 10L12 6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>
      </summary>
      <div class="mt-3 text-[14px] text-ds-description leading-relaxed border-t border-white/5 pt-3">
        Smoooth isolates all agent execution through four layers of defense: (1) Loopback binding to <code class="text-[#73a3d2] font-mono">127.0.0.1:4040</code> with zero public port exposure; (2) Role-based bearer token scopes (<code class="text-[#73a3d2] font-mono">read_only</code>, <code class="text-[#73a3d2] font-mono">plan</code>, <code class="text-[#73a3d2] font-mono">deploy</code>, <code class="text-[#73a3d2] font-mono">admin</code>); (3) Systemd sandboxing (<code class="text-[#73a3d2] font-mono">NoNewPrivileges=yes</code>, <code class="text-[#73a3d2] font-mono">ProtectSystem=strict</code>); and (4) Append-only signed audit logging of every container operation.
      </div>
    </details>
    <details class="group rounded-xl border border-white/10 bg-white/[0.02] p-5 transition-all open:bg-white/[0.04]">
      <summary class="flex cursor-pointer items-center justify-between text-[16px] font-semibold text-white list-none">
        <span>What databases and Linux distributions are supported?</span>
        <svg class="w-4 h-4 text-white/50 transition-transform group-open:rotate-180" viewBox="0 0 16 16" fill="none"><path d="M4 6L8 10L12 6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>
      </summary>
      <div class="mt-3 text-[14px] text-ds-description leading-relaxed border-t border-white/5 pt-3">
        Smoooth is verified on Ubuntu 24.04 LTS and Debian 12 (x86_64 and ARM64). It provides native stateful adapters for PostgreSQL, Redis, MongoDB, and SQLite with automated persistent volume management, credential rotation, and container health probes.
      </div>
    </details>
    <details class="group rounded-xl border border-white/10 bg-white/[0.02] p-5 transition-all open:bg-white/[0.04]">
      <summary class="flex cursor-pointer items-center justify-between text-[16px] font-semibold text-white list-none">
        <span>How does Smoooth compare to Coolify or Dokku?</span>
        <svg class="w-4 h-4 text-white/50 transition-transform group-open:rotate-180" viewBox="0 0 16 16" fill="none"><path d="M4 6L8 10L12 6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>
      </summary>
      <div class="mt-3 text-[14px] text-ds-description leading-relaxed border-t border-white/5 pt-3">
        Coolify requires 1.2–1.8 GB RAM idle and is designed around a human web UI. Dokku relies on git-push hooks. Smoooth is a single 25MB static binary consuming under 28MB RAM idle, purpose-built for autonomous AI agents over MCP with transactional 2-phase DAG deployments and automatic rollbacks if health probes fail.
      </div>
    </details>
    <details class="group rounded-xl border border-white/10 bg-white/[0.02] p-5 transition-all open:bg-white/[0.04]">
      <summary class="flex cursor-pointer items-center justify-between text-[16px] font-semibold text-white list-none">
        <span>Can I run Smoooth on a low-cost $5/mo VPS or homelab?</span>
        <svg class="w-4 h-4 text-white/50 transition-transform group-open:rotate-180" viewBox="0 0 16 16" fill="none"><path d="M4 6L8 10L12 6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/></svg>
      </summary>
      <div class="mt-3 text-[14px] text-ds-description leading-relaxed border-t border-white/5 pt-3">
        Yes. Minimum requirements are 1 vCPU, 1 GB RAM, and 10 GB disk space. Because Smoooth is compiled in Rust with zero background Node.js/PHP runtime overhead, your VPS resources remain 100% dedicated to your actual application workloads.
      </div>
    </details>
  </div>
</section>
`;

html = html.slice(0, quickstartIndex) + faqSectionHtml + html.slice(quickstartIndex);

// 9. Update Footer links: include FAQ link
html = html.replace(
  '<a href="#security" class="hover:text-white transition-colors">Security</a>',
  '<a href="#faq" class="hover:text-white transition-colors">FAQ</a>\n      <a href="#security" class="hover:text-white transition-colors">Security</a>'
);

fs.writeFileSync(targetFile, html, 'utf8');
console.log('✓ Successfully enhanced exact_harness.html with all Phase 2 and Anti-Slop improvements.');
