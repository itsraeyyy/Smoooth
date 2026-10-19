const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const distDir = path.resolve(rootDir, 'dist');

console.log('Building 1:1 DeepSeek Harness landing page for Cloudflare Pages / Static export...');

if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

// 1. Copy deepseek.css to root styles.css and dist/styles.css
const cssSrc = path.join(rootDir, 'frontend/app/deepseek.css');
const cssContent = fs.readFileSync(cssSrc, 'utf8') + '\n\n/* Story visibility override */\n.HarnessWorkflowPreview_story__lcRCk { opacity: 1 !important; }\n';
fs.writeFileSync(path.join(rootDir, 'styles.css'), cssContent, 'utf8');
fs.writeFileSync(path.join(distDir, 'styles.css'), cssContent, 'utf8');

// 2. Generate standalone root index.html
const harnessHtml = fs.readFileSync(path.join(rootDir, 'frontend/app/exact_harness.html'), 'utf8');
const bridgeJs = `
<script>
document.addEventListener("DOMContentLoaded", function () {
  // 1. Hero Canvas Ambient Glow
  const canvas = document.querySelector("canvas");
  if (canvas) {
    const resize = () => {
      canvas.width = canvas.parentElement?.clientWidth || window.innerWidth;
      canvas.height = canvas.parentElement?.clientHeight || 900;
      const ctx = canvas.getContext("2d");
      if (ctx) {
        const grad = ctx.createRadialGradient(
          canvas.width / 2, canvas.height * 0.15, 10,
          canvas.width / 2, canvas.height * 0.25, canvas.width * 0.65
        );
        grad.addColorStop(0, "rgba(35, 85, 175, 0.48)");
        grad.addColorStop(0.35, "rgba(20, 52, 115, 0.32)");
        grad.addColorStop(0.7, "rgba(10, 18, 32, 0.85)");
        grad.addColorStop(1, "rgba(8, 12, 20, 0)");
        ctx.fillStyle = grad;
        ctx.fillRect(0, 0, canvas.width, canvas.height);
      }
    };
    resize();
    window.addEventListener("resize", resize);
  }

  // 2. Activate demo frames
  document.querySelectorAll("[data-running]").forEach(f => {
    f.setAttribute("data-running", "true");
    f.setAttribute("data-paused", "false");
  });

  // 3. Thought for a while accordion
  const thoughtBtn = document.querySelector(".HarnessDesktopPreview_analysisSummary__Qp8FG");
  if (thoughtBtn) {
    thoughtBtn.style.cursor = "pointer";
    let isOpen = false;
    thoughtBtn.addEventListener("click", () => {
      isOpen = !isOpen;
      let detail = document.getElementById("dsh-thought-details");
      if (!detail) {
        detail = document.createElement("div");
        detail.id = "dsh-thought-details";
        detail.style.padding = "10px 14px";
        detail.style.margin = "8px 0 12px 0";
        detail.style.borderRadius = "8px";
        detail.style.background = "#0d1117";
        detail.style.border = "1px solid rgba(255,255,255,0.08)";
        detail.style.fontFamily = "monospace";
        detail.style.fontSize = "12px";
        detail.style.color = "#8b949e";
        detail.style.lineHeight = "1.6";
        detail.innerHTML = '<div style="color: #58a6ff;">› Probing runtime dependencies & plugins...</div><div>  • Initializing Cordis micro-kernel context</div><div>  • Loading dynamic plugin registry (shell, git, fs)</div><div style="color: #3fb950; margin-top: 4px;">✓ Environment verified. Ready for tasks.</div>';
        thoughtBtn.parentNode?.insertBefore(detail, thoughtBtn.nextSibling);
      }
      detail.style.display = isOpen ? "block" : "none";
    });
  }

  // 4. Pomodoro timer
  const timerStart = document.querySelector(".HarnessPluginsPreview_timerStart__34s_M");
  const timerStrips = document.querySelectorAll(".HarnessPluginsPreview_timerStrip__5RKkN");
  if (timerStart && timerStrips.length > 0) {
    timerStart.style.cursor = "pointer";
    let isRunning = false;
    let timerInterval = null;
    let frame = 0;
    timerStart.addEventListener("click", () => {
      isRunning = !isRunning;
      if (isRunning) {
        timerStart.style.opacity = "0.7";
        timerInterval = setInterval(() => {
          frame = (frame + 1) % 25;
          timerStrips.forEach(s => {
            s.style.transform = "translateY(-" + (frame * 36) + "px)";
            s.style.transition = "transform 0.4s ease";
          });
        }, 1000);
      } else {
        clearInterval(timerInterval);
        timerStart.style.opacity = "1";
      }
    });
  }

  // 5. Copy buttons
  document.querySelectorAll("button").forEach(btn => {
    if (btn.textContent?.trim() === "Copy") {
      btn.addEventListener("click", () => {
        const codeEl = btn.parentElement?.querySelector("code");
        if (codeEl) {
          let text = codeEl.textContent || "";
          if (text.startsWith("$ ")) text = text.slice(2);
          navigator.clipboard.writeText(text.trim());
          btn.textContent = "Copied!";
          setTimeout(() => { btn.textContent = "Copy"; }, 2000);
        }
      });
    }
  });

  // 6. Mobile drawer & download accordion
  const mobileMenuBtn = document.getElementById("harness-mobile-menu");
  const mobileMenu = document.querySelector(".ds-mobile-menu");
  const mobileCloseBtn = mobileMenu?.querySelector('button[aria-label="Close menu"]');
  if (mobileMenuBtn && mobileMenu) {
    mobileMenuBtn.addEventListener("click", () => {
      mobileMenu.classList.add("is-open");
      document.body.style.overflow = "hidden";
    });
  }
  if (mobileCloseBtn && mobileMenu) {
    mobileCloseBtn.addEventListener("click", () => {
      mobileMenu.classList.remove("is-open");
      document.body.style.overflow = "";
    });
  }

  const mobileDownloadTrigger = document.querySelector(".harness-mobile-download-trigger");
  const mobileDownload = document.querySelector(".harness-mobile-download");
  if (mobileDownloadTrigger && mobileDownload) {
    mobileDownloadTrigger.addEventListener("click", () => {
      const isOpen = mobileDownload.getAttribute("data-open") === "true";
      mobileDownload.setAttribute("data-open", isOpen ? "false" : "true");
      mobileDownloadTrigger.setAttribute("aria-expanded", isOpen ? "false" : "true");
    });
  }

  // 7. WeChat QR popover
  const qrTriggers = document.querySelectorAll(".ds-qr-trigger");
  qrTriggers.forEach(trig => {
    trig.addEventListener("click", e => {
      e.stopPropagation();
      trig.classList.toggle("ds-qr-active");
    });
  });
  document.addEventListener("click", () => {
    qrTriggers.forEach(t => t.classList.remove("ds-qr-active"));
  });

  // 8. Migration persona tabs switcher
  const personaTabs = document.querySelectorAll(".migration-persona-tab");
  const personaPanels = document.querySelectorAll(".persona-panel");
  if (personaTabs.length > 0 && personaPanels.length > 0) {
    personaTabs.forEach(tab => {
      tab.addEventListener("click", () => {
        const target = tab.getAttribute("data-persona");
        personaTabs.forEach(t => {
          const isActive = t === tab;
          t.setAttribute("aria-selected", isActive ? "true" : "false");
          if (isActive) {
            t.className = "migration-persona-tab px-4 py-2 rounded-lg text-[13px] font-medium font-mono transition-all border border-white/20 bg-white/10 text-white shadow-sm";
          } else {
            t.className = "migration-persona-tab px-4 py-2 rounded-lg text-[13px] font-medium font-mono transition-all border border-transparent text-white/60 hover:text-white hover:bg-white/5";
          }
        });
        personaPanels.forEach(panel => {
          if (panel.getAttribute("data-persona-panel") === target) {
            panel.classList.remove("hidden");
          } else {
            panel.classList.add("hidden");
          }
        });
      });
    });
  }
});
</script>
`;

const fullHtml = `<!DOCTYPE html>
<html lang="en-US">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>Smoooth — Autonomous Infrastructure Runtime for AI Coding Agents</title>
  <meta name="description" content="Deploy and operate stateful applications on infrastructure you control. Smoooth connects Claude Code, Cursor, and Windsurf directly to your Linux VPS over MCP — provisioning Docker containers, persistent databases, and secure networks on demand."/>
  <meta name="keywords" content="Smoooth, infrastructure runtime, AI coding agents, Model Context Protocol, MCP server, Claude Code, Cursor, Windsurf, Docker orchestration, PostgreSQL, Redis, self-hosted PaaS, Linux VPS, autonomous DevOps, open source"/>
  <meta property="og:title" content="Smoooth — Autonomous Infrastructure Runtime for AI Coding Agents"/>
  <meta property="og:description" content="Deploy and operate stateful applications on infrastructure you control. Smoooth connects Claude Code, Cursor, and Windsurf directly to your Linux VPS over MCP."/>
  <meta property="og:type" content="website"/>
  <meta property="og:url" content="https://smoooth.dev"/>
  <meta property="og:image" content="https://smoooth.dev/harness/images/deepseek-harness-social.jpg"/>
  <meta name="twitter:card" content="summary_large_image"/>
  <meta name="twitter:title" content="Smoooth — Autonomous Infrastructure Runtime for AI Coding Agents"/>
  <meta name="twitter:description" content="Deploy and operate stateful applications on infrastructure you control. Smoooth connects Claude Code, Cursor, and Windsurf directly to your Linux VPS over MCP."/>
  <meta name="twitter:image" content="https://smoooth.dev/harness/images/deepseek-harness-social.jpg"/>
  <link rel="canonical" href="https://smoooth.dev"/>
  <link rel="icon" href="/harness/favicon.svg" type="image/svg+xml"/>
  <link rel="icon" href="/favicon.ico" type="image/x-icon"/>
  <link rel="stylesheet" href="styles.css"/>
  <script type="application/ld+json">
  {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "SoftwareApplication",
        "name": "Smoooth",
        "url": "https://smoooth.dev",
        "operatingSystem": "Linux (Ubuntu 22.04/24.04, Debian 12), macOS, Windows",
        "applicationCategory": "DeveloperApplication",
        "applicationSubCategory": "Infrastructure & DevOps Runtime",
        "description": "Deploy and operate stateful applications on infrastructure you control. Smoooth connects Claude Code, Cursor, and Windsurf directly to your Linux VPS over MCP — provisioning Docker containers, persistent databases, and secure networks on demand.",
        "license": "https://opensource.org/licenses/Apache-2.0",
        "offers": {
          "@type": "Offer",
          "price": "0",
          "priceCurrency": "USD"
        }
      },
      {
        "@type": "Organization",
        "name": "Smoooth",
        "url": "https://smoooth.dev",
        "logo": "https://smoooth.dev/harness/favicon.svg",
        "sameAs": [
          "https://github.com/itsraeyyy/Smoooth"
        ]
      }
    ]
  }
  </script>
  <style>
    :root { --ds-color-bg-page: #080C14; }
    body { background-color: #080C14; color: #F9FAFB; margin: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
  </style>
</head>
<body class="antialiased min-h-screen bg-[#080C14] text-[#F9FAFB]">
  ${harnessHtml}
  ${bridgeJs}
</body>
</html>`;

fs.writeFileSync(path.join(rootDir, 'index.html'), fullHtml, 'utf8');
fs.writeFileSync(path.join(distDir, 'index.html'), fullHtml, 'utf8');

// 3. Copy public assets (harness, fonts, favicon) to root and dist
function copyDirRecursive(src, dest) {
  if (!fs.existsSync(src)) return;
  fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const sPath = path.join(src, entry.name);
    const dPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDirRecursive(sPath, dPath);
    } else {
      fs.copyFileSync(sPath, dPath);
    }
  }
}

copyDirRecursive(path.join(rootDir, 'frontend/public/harness'), path.join(distDir, 'harness'));
copyDirRecursive(path.join(rootDir, 'frontend/public/harness'), path.join(rootDir, 'harness'));

copyDirRecursive(path.join(rootDir, 'frontend/public/fonts'), path.join(distDir, 'fonts'));
copyDirRecursive(path.join(rootDir, 'frontend/public/fonts'), path.join(rootDir, 'fonts'));

fs.copyFileSync(path.join(rootDir, 'frontend/public/favicon.ico'), path.join(distDir, 'favicon.ico'));
fs.copyFileSync(path.join(rootDir, 'frontend/public/favicon.ico'), path.join(rootDir, 'favicon.ico'));

console.log('✓ Successfully synchronized 1:1 DeepSeek Harness to root index.html, styles.css, and dist/');
