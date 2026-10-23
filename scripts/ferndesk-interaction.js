// Smoooth Interactive Runtime for Homepage
// High-performance, crisp interactive handlers for hero tabs, installer copy, and FAQ accordions

document.addEventListener('DOMContentLoaded', () => {
  // 1. Copy installer button
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

  // 2. Hero 4-Stage Showcase Tabs
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

  // 3. FAQ Accordion Interaction
  const faqButtons = document.querySelectorAll('#faq button[aria-expanded]');
  faqButtons.forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.preventDefault();
      const isExpanded = btn.getAttribute('aria-expanded') === 'true';
      const newExpanded = !isExpanded;
      btn.setAttribute('aria-expanded', newExpanded ? 'true' : 'false');

      const parentContainer = btn.parentElement;
      const answerDiv = parentContainer?.querySelector(':scope > div');
      const icon = btn.querySelector('svg');

      if (answerDiv) {
        answerDiv.style.display = newExpanded ? 'block' : 'none';
      }
      if (icon) {
        icon.style.transform = newExpanded ? 'rotate(180deg)' : 'rotate(0deg)';
      }
    });
  });

  // 4. Mobile Menu Toggle
  const mobileBtn = document.getElementById('mobile-menu-btn');
  const mobileNav = document.getElementById('mobile-nav-panel');
  if (mobileBtn && mobileNav) {
    mobileBtn.addEventListener('click', (e) => {
      e.preventDefault();
      const isExpanded = mobileBtn.getAttribute('aria-expanded') === 'true';
      mobileBtn.setAttribute('aria-expanded', !isExpanded ? 'true' : 'false');
      if (isExpanded) {
        mobileNav.classList.add('hidden');
      } else {
        mobileNav.classList.remove('hidden');
      }
    });

    // Close mobile nav when clicking a link inside it
    const links = mobileNav.querySelectorAll('a');
    links.forEach(l => {
      l.addEventListener('click', () => {
        mobileBtn.setAttribute('aria-expanded', 'false');
        mobileNav.classList.add('hidden');
      });
    });
  }
});
