// Ferndesk Interactive Runtime
// Provides seamless interactivity for Ferndesk shell (header dropdowns, mobile menu, modals)
// and Smoooth homepage hero components (installer copy, 4-stage terminal tabs, Ferndesk FAQ accordions).

document.addEventListener('DOMContentLoaded', () => {
  // 1. Copy installer button (Smoooth Hero)
  const copyBtn = document.getElementById('copy-installer-btn');
  if (copyBtn) {
    copyBtn.addEventListener('click', () => {
      const cmd = 'curl -fsSL https://smoooth.dev/install.sh | bash';
      navigator.clipboard.writeText(cmd).then(() => {
        const prev = copyBtn.textContent;
        copyBtn.textContent = 'Copied!';
        copyBtn.style.color = '#3fb950';
        setTimeout(() => {
          copyBtn.textContent = prev;
          copyBtn.style.color = '';
        }, 2000);
      });
    });
  }

  // 2. Hero 4-Stage Showcase Tabs (Smoooth Hero)
  const heroTabPanes = [
    {
      agent: 'Claude Code via stdio',
      status: 'READY FOR PLANNING',
      html: `
        <div class="text-[#8b949e]">$ smoooth status --host 65.109.84.12</div>
        <div class="text-[#3fb950]">[✓] Linux Host: Ubuntu 24.04.1 LTS (Linux 6.8.0-45-generic x86_64)</div>
        <div class="text-[#3fb950]">[✓] Hardware Specs: 8 vCPU · 16.0 GB RAM · 160 GB NVMe (/var/lib/smoooth: 142 GB free)</div>
        <div class="text-[#3fb950]">[✓] Docker Daemon: Docker v26.1.3 (daemon healthy, 0 running containers)</div>
        <div class="text-[#3fb950]">[✓] Active MCP Adapter: stdio &amp; loopback (127.0.0.1:4040/sse)</div>
        <div class="text-[#58a6ff] mt-2">› Discovered Capabilities via MCP:</div>
        <div class="text-[#e6edf3] pl-3">• Runtime: docker-container (scoped bridge, cgroups v2, no-new-privileges)</div>
        <div class="text-[#e6edf3] pl-3">• Database: postgresql@16-alpine (persistent volume /var/lib/smoooth/volumes/pgdata)</div>
        <div class="text-[#e6edf3] pl-3">• Ingress: caddy-proxy (ACME automatic TLS, loopback upstream bridge)</div>
      `
    },
    {
      agent: 'Cursor Agent (MCP Client)',
      status: 'PLAN DETERMINISTIC // VERIFIED HASH',
      html: `
        <div class="text-[#8b949e]">$ smoooth plan --workload api-service --db postgresql</div>
        <div class="text-[#79c0ff]">{</div>
        <div class="text-[#e6edf3] pl-3">"plan_id": "plan_01JA48FC89N4Q",</div>
        <div class="text-[#e6edf3] pl-3">"target_workload": "api-service",</div>
        <div class="text-[#e6edf3] pl-3">"dependencies": [</div>
        <div class="text-[#c9d1d9] pl-6">{ "phase": 1, "step": "volume_mount", "path": "/var/lib/smoooth/volumes/pgdata", "immutable": true },</div>
        <div class="text-[#c9d1d9] pl-6">{ "phase": 1, "step": "provision_database", "adapter": "postgres:16-alpine", "bind": "127.0.0.1:5432" },</div>
        <div class="text-[#c9d1d9] pl-6">{ "phase": 2, "step": "inject_credentials", "scope": ["read_write"], "secret_ref": "sec_pg_auth_live" },</div>
        <div class="text-[#c9d1d9] pl-6">{ "phase": 2, "step": "deploy_container", "image": "ghcr.io/org/api:v1.4.2", "ports": ["8080:80"] }</div>
        <div class="text-[#e6edf3] pl-3">],</div>
        <div class="text-[#3fb950] pl-3">"verification_hash": "sha256:8e4c72a910bf55d781e...",</div>
        <div class="text-[#3fb950] pl-3">"destructive_actions": 0</div>
        <div class="text-[#79c0ff]">}</div>
      `
    },
    {
      agent: 'Windsurf Cascade (Automated Deploy)',
      status: 'DEPLOYMENT COMMITTED // CHECKPOINT #42',
      html: `
        <div class="text-[#8b949e]">$ smoooth execute --hash 8e4c72a910bf</div>
        <div class="text-[#e6edf3]">[00:00.12] › Phase 1: Mount persistent volume /var/lib/smoooth/volumes/pgdata ... <span class="text-[#3fb950]">[OK]</span></div>
        <div class="text-[#e6edf3]">[00:00.45] › Phase 1: Starting PostgreSQL container (postgres:16-alpine) ... <span class="text-[#3fb950]">[OK]</span></div>
        <div class="text-[#e6edf3]">[00:01.02] › Phase 1: Authenticating test query on postgresql://smoooth@127.0.0.1:5432/app ... <span class="text-[#3fb950]">[OK] (3.2ms)</span></div>
        <div class="text-[#e6edf3]">[00:01.35] › Phase 2: Starting app container api-service (ghcr.io/org/api:v1.4.2) ... <span class="text-[#3fb950]">[OK]</span></div>
        <div class="text-[#e6edf3]">[00:01.88] › Phase 2: Probing health endpoint http://127.0.0.1:8080/healthz ... <span class="text-[#3fb950]">[HTTP 200 OK]</span></div>
        <div class="text-[#3fb950] font-semibold mt-2">✓ Deployment verified. Checkpoint #42 committed to state ledger.</div>
      `
    },
    {
      agent: 'Smoooth Daemon Autonomous Reconciler',
      status: 'SELF-HEALED // ZERO DATA LOSS',
      html: `
        <div class="text-[#f85149]">[ALERT] [14:32:01] Container 'api-service' exited unexpectedly (exit code 137: SIGKILL).</div>
        <div class="text-[#e6edf3]">[RECONCILER] Evaluating intended state vs observed state...</div>
        <div class="text-[#3fb950]">[POLICY] Action permitted: Restart stateless application container (budget: 3 restarts/hour).</div>
        <div class="text-[#58a6ff]">[SAFETY] Persistent database volume 'pgdata' is quarantined and protected from deletion.</div>
        <div class="text-[#e6edf3]">[RECONCILER] Spawning replacement container from pinned image ghcr.io/org/api:v1.4.2 ... <span class="text-[#3fb950]">[OK]</span></div>
        <div class="text-[#3fb950]">[VERIFY] Probing http://127.0.0.1:8080/healthz ... [HTTP 200 OK] (recovery completed in 840ms).</div>
        <div class="text-[#8b949e]">[INCIDENT] Logged incident #89 to audit trail. Zero database corruption verified.</div>
      `
    }
  ];

  const heroPanel = document.getElementById('hero-panel');
  const heroDynamicPane = document.getElementById('hero-dynamic-pane');
  const heroAgentStatus = heroPanel?.querySelector('.border-t span:first-child');
  const heroStateStatus = heroPanel?.querySelector('.border-t span:last-child');
  const tabBtns = [0, 1, 2, 3].map(i => document.getElementById(`hero-tab-${i}`)).filter(Boolean);

  function activateTab(index) {
    tabBtns.forEach((btn, idx) => {
      const isSelected = idx === index;
      btn.setAttribute('aria-selected', isSelected ? 'true' : 'false');
      btn.setAttribute('tabindex', isSelected ? '0' : '-1');
      const dot = btn.querySelector('span');
      if (isSelected) {
        btn.className = 'relative flex shrink-0 items-center gap-2 overflow-hidden rounded-t border border-b-0 pt-2.5 pr-4 pb-2.5 pl-3.5 text-sm leading-tight font-medium transition-colors border-border-strong bg-white text-fg';
        if (dot) dot.className = 'size-2 rounded-full bg-fern-600 shrink-0';
      } else {
        btn.className = 'relative flex shrink-0 items-center gap-2 overflow-hidden rounded-t border border-b-0 pt-2.5 pr-4 pb-2.5 pl-3.5 text-sm leading-tight font-medium transition-colors border-transparent text-fg-3 hover:text-fg';
        if (dot) dot.className = 'size-2 rounded-full bg-fg-4 shrink-0';
      }
    });

    const data = heroTabPanes[index];
    if (data && heroDynamicPane) {
      heroDynamicPane.innerHTML = data.html;
      if (heroAgentStatus) heroAgentStatus.textContent = 'Agent: ' + data.agent;
      if (heroStateStatus) heroStateStatus.textContent = 'Status: ' + data.status;
    }
  }

  tabBtns.forEach((btn, idx) => {
    btn.addEventListener('click', (e) => {
      e.preventDefault();
      activateTab(idx);
    });
  });

  // 3. Ferndesk Header Dropdown Menus (Features & Resources)
  const featuresBtn = document.querySelector('button[aria-controls="features-menu"]');
  const resourcesBtn = document.querySelector('button[aria-controls="resources-menu"]');

  let featuresMenu = document.getElementById('features-menu');
  if (!featuresMenu && featuresBtn) {
    featuresMenu = document.createElement('div');
    featuresMenu.id = 'features-menu';
    featuresMenu.className = 'border-border bg-bg-raised shadow-float absolute top-full left-1/2 z-50 mt-2 w-[900px] max-w-[calc(100vw-2.5rem)] -translate-x-1/2 overflow-hidden rounded-lg border hidden';
    featuresMenu.innerHTML = `
      <div class="grid grid-cols-3 gap-x-8 px-7 pt-7 pb-6">
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2.5 text-xs font-semibold uppercase tracking-wider text-fg-3">Help center</span>
          <a href="/ai-help-center" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Hosted help center</span><span class="text-fg-3 text-sm">Your domain or /help</span></span>
          </a>
          <a href="/generative-ai-customer-service" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">AI conversations</span><span class="text-fg-3 text-sm">Answers drawn from your docs</span></span>
          </a>
          <a href="/self-service-help-widget" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">In-app widget</span><span class="text-fg-3 text-sm">Help inside your product</span></span>
          </a>
          <a href="/multilingual-knowledge-base" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Multilingual</span><span class="text-fg-3 text-sm">Translated, kept in sync</span></span>
          </a>
          <a href="/internal-knowledge-base" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Private knowledge base</span><span class="text-fg-3 text-sm">Docs behind a login</span></span>
          </a>
        </div>
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2.5 text-xs font-semibold uppercase tracking-wider text-fg-3">AI & automation</span>
          <a href="/self-updating-help-center" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Verification</span><span class="text-fg-3 text-sm">Every article checked</span></span>
          </a>
          <a href="/code-to-docs" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Code to docs</span><span class="text-fg-3 text-sm">Fixes from every PR</span></span>
          </a>
          <a href="/automated-screenshots-for-docs" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Automated screenshots</span><span class="text-fg-3 text-sm">Re-taken when UI changes</span></span>
          </a>
          <a href="/ai-documentation-agent" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">AI drafting</span><span class="text-fg-3 text-sm">Written from your codebase</span></span>
          </a>
        </div>
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2.5 text-xs font-semibold uppercase tracking-wider text-fg-3">Connect</span>
          <a href="/integrations" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Integrations</span><span class="text-fg-3 text-sm">GitHub, Linear, Slack</span></span>
          </a>
          <a href="https://docs.ferndesk.com/mcp-hjgdc" target="_blank" rel="noopener noreferrer" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">MCP server</span><span class="text-fg-3 text-sm">Let Claude &amp; Cursor read your docs</span></span>
          </a>
          <a href="/chrome-extension-for-documentation" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Chrome extension</span><span class="text-fg-3 text-sm">Capture product changes</span></span>
          </a>
        </div>
      </div>
    `;
    featuresBtn.parentElement?.appendChild(featuresMenu);
  }

  let resourcesMenu = document.getElementById('resources-menu');
  if (!resourcesMenu && resourcesBtn) {
    resourcesMenu = document.createElement('div');
    resourcesMenu.id = 'resources-menu';
    resourcesMenu.className = 'border-border bg-bg-raised shadow-float absolute top-full left-1/2 z-50 mt-2 w-[520px] max-w-[calc(100vw-2.5rem)] -translate-x-1/2 overflow-hidden rounded-lg border hidden';
    resourcesMenu.innerHTML = `
      <div class="grid grid-cols-2 gap-x-6 px-6 pt-6 pb-5">
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2 text-xs font-semibold uppercase tracking-wider text-fg-3">Learn</span>
          <a href="/blog" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Blog</span><span class="text-fg-3 text-sm">Product, AI, and docs advice</span></span>
          </a>
          <a href="/answers" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Answer hub</span><span class="text-fg-3 text-sm">Guides and deep dives</span></span>
          </a>
        </div>
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2 text-xs font-semibold uppercase tracking-wider text-fg-3">Compare</span>
          <a href="/compare" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">All comparisons</span><span class="text-fg-3 text-sm">See how Ferndesk stacks up</span></span>
          </a>
          <a href="/compare/zendesk" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">vs Zendesk</span><span class="text-fg-3 text-sm">Modern, AI-first help center</span></span>
          </a>
          <a href="/compare/intercom" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">vs Intercom</span><span class="text-fg-3 text-sm">Better self-service at lower cost</span></span>
          </a>
        </div>
      </div>
    `;
    resourcesBtn.parentElement?.appendChild(resourcesMenu);
  }

  function toggleDropdown(btn, menu) {
    if (!btn || !menu) return;
    const isExpanded = btn.getAttribute('aria-expanded') === 'true';
    closeAllDropdowns();
    if (!isExpanded) {
      btn.setAttribute('aria-expanded', 'true');
      menu.classList.remove('hidden');
    }
  }

  function closeAllDropdowns() {
    if (featuresBtn) featuresBtn.setAttribute('aria-expanded', 'false');
    if (resourcesBtn) resourcesBtn.setAttribute('aria-expanded', 'false');
    if (featuresMenu) featuresMenu.classList.add('hidden');
    if (resourcesMenu) resourcesMenu.classList.add('hidden');
  }

  if (featuresBtn && featuresMenu) {
    featuresBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleDropdown(featuresBtn, featuresMenu);
    });
  }

  if (resourcesBtn && resourcesMenu) {
    resourcesBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleDropdown(resourcesBtn, resourcesMenu);
    });
  }

  document.addEventListener('click', (e) => {
    if (!e.target.closest('#features-menu') && !e.target.closest('button[aria-controls="features-menu"]') &&
        !e.target.closest('#resources-menu') && !e.target.closest('button[aria-controls="resources-menu"]')) {
      closeAllDropdowns();
    }
  });

  // 4. Mobile Menu Drawer
  const mobileBtn = document.querySelector('button[aria-controls="mobile-menu"]');
  const header = document.querySelector('header');
  let mobileMenu = document.getElementById('mobile-menu');

  if (!mobileMenu && header) {
    mobileMenu = document.createElement('div');
    mobileMenu.id = 'mobile-menu';
    mobileMenu.className = 'border-border bg-bg border-b px-5 pt-4 pb-8 lg:hidden hidden';
    mobileMenu.innerHTML = `
      <div class="flex flex-col gap-4">
        <nav class="flex flex-col divide-y divide-border">
          <a href="/pricing" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Pricing</a>
          <a href="/customers" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Customers</a>
          <a href="/migration" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Migrate</a>
          <a href="/answers" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Answer hub</a>
          <a href="/blog" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Blog</a>
        </nav>
        <div class="flex flex-col gap-2 pt-6">
          <a href="https://ferndesk.com/app" class="focus-ring text-prose hover:bg-bg-muted hover:text-fg rounded border border-gray-300 px-3 py-2 text-center text-base font-medium transition-colors">Sign in</a>
          <a href="https://ferndesk.com/app/signup" class="focus-ring bg-brand text-white hover:bg-brand/90 rounded px-3 py-2 text-center text-base font-medium transition-colors">Start free</a>
        </div>
      </div>
    `;
    header.appendChild(mobileMenu);
  }

  if (mobileBtn && mobileMenu) {
    mobileBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      const isOpen = !mobileMenu.classList.contains('hidden');
      if (isOpen) {
        mobileMenu.classList.add('hidden');
        mobileBtn.setAttribute('aria-expanded', 'false');
      } else {
        mobileMenu.classList.remove('hidden');
        mobileBtn.setAttribute('aria-expanded', 'true');
      }
    });
  }

  // 5. FAQ Accordion Interaction
  const jsonLdScripts = document.querySelectorAll('script[type="application/ld+json"]');
  let faqData = [];
  jsonLdScripts.forEach((s) => {
    try {
      const parsed = JSON.parse(s.textContent);
      if (parsed['@type'] === 'FAQPage' && parsed.mainEntity) {
        faqData = parsed.mainEntity;
      }
    } catch (_) {}
  });

  const faqButtons = document.querySelectorAll('button[aria-expanded]');
  let faqBtnIndex = 0;
  faqButtons.forEach((btn) => {
    const numSpan = btn.querySelector('span.tabular-nums');
    if (!numSpan && !btn.closest('#faq')) return;

    const currentFaqIndex = faqBtnIndex++;
    btn.addEventListener('click', (e) => {
      e.preventDefault();
      const isExpanded = btn.getAttribute('aria-expanded') === 'true';
      const newExpanded = !isExpanded;
      btn.setAttribute('aria-expanded', newExpanded ? 'true' : 'false');

      const icon = btn.querySelector('span[aria-hidden="true"], svg');
      if (icon) {
        if (newExpanded) {
          icon.classList.add('rotate-90');
        } else {
          icon.classList.remove('rotate-90');
        }
      }

      const parentContainer = btn.parentElement;
      let answerDiv = parentContainer?.querySelector(':scope > div:not([class*="grid"])');
      if (!answerDiv && faqData[currentFaqIndex]) {
        answerDiv = document.createElement('div');
        answerDiv.innerHTML = `<p class="pb-6 pr-0 text-lg leading-[1.55] text-pretty text-prose sm:pl-14 sm:pr-10 lg:pb-[26px]">${faqData[currentFaqIndex].acceptedAnswer?.text || ''}</p>`;
        parentContainer.appendChild(answerDiv);
      }
      if (answerDiv) {
        answerDiv.style.display = newExpanded ? 'block' : 'none';
      }
    });
  });

  // 6. Video Demo Modal
  const videoBtn = document.querySelector('button[aria-label="Watch how Fern verifies your help center"]');
  if (videoBtn) {
    videoBtn.addEventListener('click', (e) => {
      e.preventDefault();
      let modal = document.getElementById('ferndesk-video-modal');
      if (!modal) {
        modal = document.createElement('div');
        modal.id = 'ferndesk-video-modal';
        modal.className = 'fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-4';
        modal.innerHTML = `
          <div class="relative w-full max-w-4xl overflow-hidden rounded-xl bg-black border border-white/20 shadow-2xl">
            <button type="button" id="close-video-modal" class="absolute top-4 right-4 z-10 flex size-9 items-center justify-center rounded-full bg-white/10 text-white hover:bg-white/20">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
            </button>
            <div class="aspect-video w-full flex items-center justify-center text-white">
              <video controls autoplay class="w-full h-full" src="https://static.ferndesk.com/assets/marketing/verification-demo.mp4" poster="https://static.ferndesk.com/cdn-cgi/image/format=webp,quality=80,width=1120/assets/marketing/home-hero-verification-2026-09.webp">
                Your browser does not support video.
              </video>
            </div>
          </div>
        `;
        document.body.appendChild(modal);
        modal.querySelector('#close-video-modal')?.addEventListener('click', () => {
          modal.remove();
        });
        modal.addEventListener('click', (ev) => {
          if (ev.target === modal) modal.remove();
        });
      }
    });
  }
});
