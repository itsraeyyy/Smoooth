/**
 * Smoooth — Waitlist Page Interactive Logic
 * Features:
 * - Dynamic procedural monochrome stipple/terrain generator on canvas
 * - LocalStorage-backed Waitlist queue ticket system
 * - Interactive MCP tool execution simulations (JSON plan tabs & command actions)
 * - Copy referral link with toast alerts
 * - Stack chip selection
 * - Accessible FAQ accordion
 */

document.addEventListener('DOMContentLoaded', () => {
  initTerrainCanvas();
  initWaitlistEngine();
  initTerminalTabs();
  initTerminalActions();
  initFaqAccordion();
});

/* ==========================================================================
   1. PROCEDURAL MONOCHROME STIPPLE TERRAIN GENERATOR
   Creates the organic black & white stippled mountain terrain from the brief
   ========================================================================== */

function initTerrainCanvas() {
  const canvas = document.getElementById('terrainCanvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  function resize() {
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.parentElement.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    canvas.style.width = `${rect.width}px`;
    canvas.style.height = `${rect.height}px`;
    ctx.scale(dpr, dpr);
    drawTerrain(ctx, rect.width, rect.height);
  }

  function drawTerrain(ctx, width, height) {
    ctx.clearRect(0, 0, width, height);

    // Light canvas background matching reference
    ctx.fillStyle = '#FFFFFF';
    ctx.fillRect(0, 0, width, height);

    // Seeded pseudo-noise function for deterministic mountain ridges
    function pseudoNoise(x, freq, seed = 0) {
      return (
        Math.sin(x * freq * 1.2 + seed) * 0.45 +
        Math.sin(x * freq * 2.6 + 1.1 + seed * 2) * 0.32 +
        Math.cos(x * freq * 0.6 + 2.3 + seed * 3) * 0.23
      );
    }

    // 3 Distinct mountain ridges (Back, Mid, Foreground)
    const ridges = [
      { yBase: height * 0.32, amp: 110, freq: 0.0032, seed: 1.2, dotDensity: 1.0, dotAlpha: 0.35, stroke: 'rgba(17, 17, 17, 0.4)' },
      { yBase: height * 0.48, amp: 140, freq: 0.0042, seed: 3.8, dotDensity: 1.3, dotAlpha: 0.65, stroke: 'rgba(17, 17, 17, 0.6)' },
      { yBase: height * 0.68, amp: 160, freq: 0.0055, seed: 7.4, dotDensity: 1.6, dotAlpha: 0.85, stroke: 'rgba(17, 17, 17, 0.9)' },
    ];

    ridges.forEach((ridge) => {
      const stepX = 2;
      const points = [];

      for (let x = 0; x <= width; x += stepX) {
        const n = pseudoNoise(x, ridge.freq, ridge.seed);
        const y = ridge.yBase - n * ridge.amp;
        points.push({ x, y });
      }

      // Draw mountain silhouette fill
      ctx.beginPath();
      ctx.moveTo(0, height);
      ctx.lineTo(points[0].x, points[0].y);
      for (let i = 1; i < points.length; i++) {
        ctx.lineTo(points[i].x, points[i].y);
      }
      ctx.lineTo(width, height);
      ctx.closePath();
      ctx.fillStyle = 'rgba(250, 250, 252, 0.85)';
      ctx.fill();

      // Sharp ridge silhouette hairline stroke
      ctx.beginPath();
      ctx.moveTo(points[0].x, points[0].y);
      for (let i = 1; i < points.length; i++) {
        ctx.lineTo(points[i].x, points[i].y);
      }
      ctx.strokeStyle = ridge.stroke;
      ctx.lineWidth = 1.2;
      ctx.stroke();

      // Dense stipple grain / halftone texture rendering
      const dotCount = Math.floor(width * 28 * ridge.dotDensity);

      for (let i = 0; i < dotCount; i++) {
        const rx = Math.random() * width;
        const pIndex = Math.min(Math.floor((rx / width) * points.length), points.length - 1);
        const ridgeY = points[pIndex] ? points[pIndex].y : ridge.yBase;

        // Bias dots heavily towards ridge lines & slope crevices
        const power = 1.8 + Math.sin(rx * 0.02) * 0.4;
        const distBelow = Math.pow(Math.random(), power) * (height - ridgeY);
        const ry = ridgeY + distBelow;

        if (ry < height) {
          const alphaFade = (1 - (distBelow / (height - ridgeY))) * ridge.dotAlpha;
          ctx.fillStyle = `rgba(17, 17, 17, ${Math.max(0.08, alphaFade * 0.95)})`;
          const size = Math.random() > 0.82 ? 1.4 : 0.9;
          ctx.fillRect(rx, ry, size, size);
        }
      }
    });

    // Subtle fine geometric hairline grid overlay
    ctx.strokeStyle = 'rgba(0, 0, 0, 0.035)';
    ctx.lineWidth = 1;
    const gridSize = 48;
    for (let x = 0; x < width; x += gridSize) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }
    for (let y = 0; y < height; y += gridSize) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
  }

  window.addEventListener('resize', debounce(resize, 150));
  resize();
}

/* ==========================================================================
   2. WAITLIST ENGINE & TICKET PASS GENERATOR
   ========================================================================== */

function initWaitlistEngine() {
  const form = document.getElementById('waitlistForm');
  const emailInput = document.getElementById('waitlistEmail');
  const formCard = document.getElementById('waitlistFormCard');
  const ticketCard = document.getElementById('waitlistTicketCard');
  const stackChips = document.querySelectorAll('.stack-chip');
  const resetBtn = document.getElementById('resetTicketBtn');
  const copyBtn = document.getElementById('copyRefBtn');
  const refInput = document.getElementById('refLinkInput');

  // Load existing ticket from LocalStorage
  const savedData = localStorage.getItem('smoooth_waitlist_ticket');
  if (savedData) {
    try {
      const ticket = JSON.parse(savedData);
      showTicket(ticket);
    } catch (e) {
      localStorage.removeItem('smoooth_waitlist_ticket');
    }
  }

  // Handle stack chip selection toggles
  stackChips.forEach(chip => {
    chip.addEventListener('click', () => {
      chip.classList.toggle('active');
    });
  });

  // Handle Form Submission
  if (form) {
    form.addEventListener('submit', (e) => {
      e.preventDefault();
      const email = emailInput.value.trim();

      if (!email || !isValidEmail(email)) {
        showToast('Please enter a valid developer or agent email address.');
        emailInput.focus();
        return;
      }

      // Collect chosen stacks
      const selectedStacks = Array.from(document.querySelectorAll('.stack-chip.active'))
        .map(chip => chip.dataset.stack || chip.textContent.trim());

      const chosenStacksStr = selectedStacks.length > 0 ? selectedStacks.join(', ') : 'Standard (Postgres + Redis)';

      // Generate deterministic queue position
      const queueNumber = Math.floor(3480 + Math.random() * 45);
      const refCode = 'SMOOOTH-' + Math.floor(1000 + Math.random() * 9000);
      const referralUrl = `https://smoooth.dev/join?ref=${refCode}`;

      const ticketData = {
        email,
        queueNumber: `#${queueNumber.toLocaleString()}`,
        stack: chosenStacksStr,
        refCode,
        referralUrl,
        joinedAt: new Date().toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
      };

      // Save to localStorage
      localStorage.setItem('smoooth_waitlist_ticket', JSON.stringify(ticketData));

      showTicket(ticketData);
      showToast('🎉 Priority access ticket generated!');
    });
  }

  function showTicket(data) {
    if (!formCard || !ticketCard) return;

    document.getElementById('ticketQueueNum').textContent = data.queueNumber;
    document.getElementById('ticketEmailVal').textContent = data.email;
    document.getElementById('ticketStackVal').textContent = data.stack;
    document.getElementById('ticketDateVal').textContent = data.joinedAt;
    if (refInput) {
      refInput.value = data.referralUrl;
    }

    formCard.style.display = 'none';
    ticketCard.style.display = 'block';
  }

  // Handle Copy Referral Link
  if (copyBtn && refInput) {
    copyBtn.addEventListener('click', () => {
      refInput.select();
      navigator.clipboard.writeText(refInput.value)
        .then(() => {
          showToast('Referral link copied to clipboard!');
          copyBtn.textContent = 'Copied!';
          setTimeout(() => { copyBtn.textContent = 'Copy'; }, 2000);
        })
        .catch(() => {
          showToast('Failed to copy. Please manually select the URL.');
        });
    });
  }

  // Handle Reset / Test again
  if (resetBtn) {
    resetBtn.addEventListener('click', () => {
      localStorage.removeItem('smoooth_waitlist_ticket');
      ticketCard.style.display = 'none';
      formCard.style.display = 'block';
      if (emailInput) {
        emailInput.value = '';
        emailInput.focus();
      }
    });
  }
}

/* ==========================================================================
   3. INTERACTIVE MCP TERMINAL TABS
   ========================================================================== */

const TERMINAL_DATA = {
  plan: `// Call: smoooth.plan.create (Idempotent: plan_89f41b2c)
{
  <span class="token-key">"intent"</span>: <span class="token-string">"Provision production cluster with caching and automated mutual TLS"</span>,
  <span class="token-key">"resources"</span>: [
    {
      <span class="token-key">"id"</span>: <span class="token-string">"res_pg_primary"</span>,
      <span class="token-key">"type"</span>: <span class="token-string">"postgres"</span>,
      <span class="token-key">"version"</span>: <span class="token-string">"16-alpine"</span>,
      <span class="token-key">"persistence"</span>: <span class="token-string">"100GiB"</span>,
      <span class="token-key">"port"</span>: <span class="token-number">5432</span>
    },
    {
      <span class="token-key">"id"</span>: <span class="token-string">"res_redis_cache"</span>,
      <span class="token-key">"type"</span>: <span class="token-string">"redis"</span>,
      <span class="token-key">"version"</span>: <span class="token-string">"7.2-alpine"</span>,
      <span class="token-key">"topology"</span>: <span class="token-string">"high-availability-cluster"</span>,
      <span class="token-key">"memory_limit"</span>: <span class="token-string">"4GiB"</span>
    },
    {
      <span class="token-key">"id"</span>: <span class="token-string">"res_app_gateway"</span>,
      <span class="token-key">"type"</span>: <span class="token-string">"container_service"</span>,
      <span class="token-key">"image"</span>: <span class="token-string">"ghcr.io/org/backend-api:v1.4"</span>,
      <span class="token-key">"ingress"</span>: <span class="token-string">"smoooth-mesh.internal"</span>,
      <span class="token-key">"bindings"</span>: [<span class="token-string">"res_pg_primary"</span>, <span class="token-string">"res_redis_cache"</span>]
    }
  ],
  <span class="token-key">"risk_tier"</span>: <span class="token-string">"Tier1_LowImpact"</span>,
  <span class="token-key">"destructive_actions"</span>: <span class="token-number">0</span>
}`,

  tools: `// Agent MCP Active Tools Registered:
[
  { <span class="token-call">"tool"</span>: <span class="token-string">"smoooth.plan.create"</span>, <span class="token-key">"desc"</span>: <span class="token-string">"Compose validated multi-service architecture plan"</span> },
  { <span class="token-call">"tool"</span>: <span class="token-string">"smoooth.plan.apply"</span>, <span class="token-key">"desc"</span>: <span class="token-string">"Execute plan with step-level rollback checkpoints"</span> },
  { <span class="token-call">"tool"</span>: <span class="token-string">"smoooth.resource.inspect"</span>, <span class="token-key">"desc"</span>: <span class="token-string">"Fetch live telemetry, memory, and runtime health"</span> },
  { <span class="token-call">"tool"</span>: <span class="token-string">"smoooth.binding.inject"</span>, <span class="token-key">"desc"</span>: <span class="token-string">"Inject zero-leak credentials to agent runtime"</span> },
  { <span class="token-call">"tool"</span>: <span class="token-string">"smoooth.state.reconcile"</span>, <span class="token-key">"desc"</span>: <span class="token-string">"Drift detection and automated self-healing"</span> }
]`,

  diff: `// Plan Reconciliation Diff:
<span class="token-comment">--- desired_state.json</span>
<span class="token-comment">+++ live_cluster_state.json</span>
<span class="token-success">+ [ADDED] postgresql://agent:****@10.0.4.12:5432/app_db (SSL Active)</span>
<span class="token-success">+ [ADDED] redis-cluster-node-01:6379 (Ping: 0.8ms)</span>
<span class="token-success">+ [WIRED] Container app env injected via IPC vault</span>
<span class="token-string">● 0 errors detected. Convergence achieved in 1.42s.</span>`,

  logs: `[14:10:02.144] <span class="token-call">INFO</span>  smoooth::orchestrator: Received MCP request from agent 'claude-code'
[14:10:02.210] <span class="token-call">INFO</span>  smoooth::models::plan: Verifying DAG dependencies...
[14:10:02.392] <span class="token-call">INFO</span>  smoooth::adapters::docker: Pulling postgres:16-alpine (cached)
[14:10:03.104] <span class="token-call">INFO</span>  smoooth::adapters::postgres: Initializing database cluster 'app_db'
[14:10:03.542] <span class="token-success">SUCCESS</span> Container healthy on internal port 5432 (ping 2.1ms)
[14:10:03.602] <span class="token-success">SUCCESS</span> Operation completed. 3 resources active.`
};

function initTerminalTabs() {
  const tabs = document.querySelectorAll('.terminal-tab-btn');
  const codeContent = document.getElementById('terminalCodeBlock');
  if (!codeContent) return;

  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      tabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');

      const targetKey = tab.dataset.tab;
      if (TERMINAL_DATA[targetKey]) {
        codeContent.innerHTML = TERMINAL_DATA[targetKey];
      }
    });
  });
}

function initTerminalActions() {
  const actionBtns = document.querySelectorAll('.terminal-btn-small');
  const codeContent = document.getElementById('terminalCodeBlock');

  actionBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const action = btn.dataset.action;
      if (action === 'add-mongo') {
        showToast('Agent injected MongoDB resource into deployment plan.');
        if (codeContent) {
          codeContent.innerHTML += `\n<span class="token-success">+ [RESOURCE ADDED] mongodb:7.0-enterprise (Cluster replica set attached)</span>`;
        }
      } else if (action === 'scale-redis') {
        showToast('Scaled Redis cache cluster to 3 HA replicas.');
        if (codeContent) {
          codeContent.innerHTML += `\n<span class="token-call">+ [RECONCILE] Redis scaled from 1 -> 3 nodes (High Availability active)</span>`;
        }
      } else if (action === 'reconcile') {
        showToast('Running drift inspection... All systems converged.');
      }
    });
  });
}

/* ==========================================================================
   4. FAQ ACCORDION LOGIC
   ========================================================================== */

function initFaqAccordion() {
  const faqItems = document.querySelectorAll('.faq-item');
  faqItems.forEach(item => {
    const question = item.querySelector('.faq-question');
    if (question) {
      question.addEventListener('click', () => {
        const isOpen = item.classList.contains('open');
        // Close others
        faqItems.forEach(i => i.classList.remove('open'));
        if (!isOpen) {
          item.classList.add('open');
        }
      });
    }
  });
}

/* ==========================================================================
   5. TOAST NOTIFICATION UTILITY
   ========================================================================== */

function showToast(message) {
  let toast = document.getElementById('toastNotice');
  if (!toast) {
    toast = document.createElement('div');
    toast.id = 'toastNotice';
    toast.className = 'toast-notice';
    document.body.appendChild(toast);
  }

  toast.textContent = message;
  toast.classList.add('visible');

  clearTimeout(toast._timeout);
  toast._timeout = setTimeout(() => {
    toast.classList.remove('visible');
  }, 3200);
}

function isValidEmail(email) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}

function debounce(func, wait) {
  let timeout;
  return function(...args) {
    clearTimeout(timeout);
    timeout = setTimeout(() => func.apply(this, args), wait);
  };
}
