const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const distDir = path.resolve(rootDir, 'dist');

console.log('Building Smoooth product landing page for Cloudflare Pages / Static export...');

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

  // 3. Desktop Preview Analysis accordion
  const thoughtBtn = document.querySelector(".HarnessDesktopPreview_analysisSummary__Qp8FG");
  if (thoughtBtn) {
    thoughtBtn.style.cursor = "pointer";
    let isOpen = false;
    thoughtBtn.addEventListener("click", () => {
      isOpen = !isOpen;
      let detail = document.getElementById("smoooth-thought-details");
      if (!detail) {
        detail = document.createElement("div");
        detail.id = "smoooth-thought-details";
        detail.style.padding = "10px 14px";
        detail.style.margin = "8px 0 12px 0";
        detail.style.borderRadius = "8px";
        detail.style.background = "#0d1117";
        detail.style.border = "1px solid rgba(255,255,255,0.08)";
        detail.style.fontFamily = "monospace";
        detail.style.fontSize = "12px";
        detail.style.color = "#8b949e";
        detail.style.lineHeight = "1.6";
        detail.innerHTML = '<div style="color: #58a6ff; font-weight: 500;">› Resolving 2-phase transactional DAG...</div>' +
          '<div style="color: #c9d1d9;">  • Enforcing 256-bit bearer token scopes: [read_only, plan, deploy]</div>' +
          '<div style="color: #c9d1d9;">  • Mounting persistent volume: /var/lib/smoooth/volumes/pgdata</div>' +
          '<div style="color: #c9d1d9;">  • Allocating loopback networking (127.0.0.1:4040/sse)</div>' +
          '<div style="color: #3fb950; margin-top: 4px;">✓ Health checks verified. Checkpoint committed.</div>';
        thoughtBtn.parentNode?.insertBefore(detail, thoughtBtn.nextSibling);
      }
      detail.style.display = isOpen ? "block" : "none";
    });
  }

  // 4. Hero Dropdown Toggle
  const toggleBtn = document.querySelector(".HarnessDownloadMenu_arrow__60_9L");
  const dropdown = document.getElementById("hero-download-dropdown");
  if (toggleBtn && dropdown) {
    toggleBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      const isVisible = dropdown.style.display !== "none";
      dropdown.style.display = isVisible ? "none" : "block";
      toggleBtn.setAttribute("aria-expanded", isVisible ? "false" : "true");
      setTimeout(() => {
        dropdown.style.opacity = isVisible ? "0" : "1";
        dropdown.style.transform = isVisible ? "translate(-50%, -6px)" : "translate(-50%, 0)";
      }, 10);
    });
    document.addEventListener("click", () => {
      if (dropdown.style.display !== "none") {
        dropdown.style.opacity = "0";
        dropdown.style.transform = "translate(-50%, -6px)";
        toggleBtn.setAttribute("aria-expanded", "false");
        setTimeout(() => { dropdown.style.display = "none"; }, 200);
      }
    });
  }

  // 5. Copy buttons feedback
  document.querySelectorAll("button").forEach(btn => {
    if (btn.textContent?.trim() === "Copy") {
      btn.addEventListener("click", () => {
        const codeEl = btn.parentElement?.querySelector("code");
        if (codeEl) {
          let text = codeEl.textContent || "";
          if (text.startsWith("$ ")) text = text.slice(2);
          navigator.clipboard.writeText(text.trim());
          const original = btn.textContent;
          btn.textContent = "Copied!";
          btn.style.color = "#59ca84";
          setTimeout(() => {
            btn.textContent = original;
            btn.style.color = "";
          }, 2000);
        }
      });
    }
  });

  // 6. Mobile drawer
  const mobileMenuBtn = document.querySelector('button[aria-label="Open menu"]');
  const mobileMenu = document.getElementById("harness-mobile-menu");
  const mobileCloseBtn = mobileMenu?.querySelector('button[aria-label="Close menu"]');
  if (mobileMenuBtn && mobileMenu) {
    mobileMenuBtn.addEventListener("click", () => {
      mobileMenu.classList.add("is-open");
      mobileMenu.style.visibility = "visible";
      mobileMenu.setAttribute("aria-hidden", "false");
      document.body.style.overflow = "hidden";
    });
  }
  if (mobileCloseBtn && mobileMenu) {
    mobileCloseBtn.addEventListener("click", () => {
      mobileMenu.classList.remove("is-open");
      mobileMenu.style.visibility = "hidden";
      mobileMenu.setAttribute("aria-hidden", "true");
      document.body.style.overflow = "";
    });
  }
  mobileMenu?.querySelectorAll("a").forEach(a => {
    a.addEventListener("click", () => {
      mobileMenu.classList.remove("is-open");
      mobileMenu.style.visibility = "hidden";
      mobileMenu.setAttribute("aria-hidden", "true");
      document.body.style.overflow = "";
    });
  });

  // 7. Migration persona tabs switcher
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

const siteUrl = "https://smoooth.dev";
const siteTitle = "Smoooth — Infrastructure Runtime for AI Coding Agents";
const siteDescription = "Deploy stateful apps on your own Linux VPS with Claude Code, Cursor, and Windsurf over MCP. Sandboxed Docker, managed databases, zero cloud markups.";

const fullHtml = `<!DOCTYPE html>
<html lang="en-US">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>${siteTitle}</title>
  <meta name="description" content="${siteDescription}"/>
  <meta name="keywords" content="Smoooth, infrastructure runtime, AI coding agents, Model Context Protocol, MCP server, Claude Code, Cursor, Windsurf, Docker deployment, PostgreSQL VPS, Redis, self-hosted PaaS, Linux VPS, autonomous DevOps, open source"/>
  <meta name="robots" content="index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1"/>
  <link rel="canonical" href="${siteUrl}"/>

  <!-- Open Graph -->
  <meta property="og:site_name" content="Smoooth"/>
  <meta property="og:title" content="${siteTitle}"/>
  <meta property="og:description" content="${siteDescription}"/>
  <meta property="og:type" content="website"/>
  <meta property="og:url" content="${siteUrl}"/>
  <meta property="og:image" content="${siteUrl}/og-image.png"/>
  <meta property="og:image:width" content="1200"/>
  <meta property="og:image:height" content="630"/>
  <meta property="og:image:alt" content="Smoooth — Infrastructure Runtime for AI Coding Agents"/>

  <!-- Twitter Card -->
  <meta name="twitter:card" content="summary_large_image"/>
  <meta name="twitter:title" content="${siteTitle}"/>
  <meta name="twitter:description" content="${siteDescription}"/>
  <meta name="twitter:image" content="${siteUrl}/og-image.png"/>

  <!-- Favicon -->
  <link rel="icon" href="/harness/favicon.svg" type="image/svg+xml"/>
  <link rel="icon" href="/favicon.ico" type="image/x-icon"/>
  <link rel="stylesheet" href="styles.css"/>

  <!-- JSON-LD Structured Data -->
  <script type="application/ld+json">
  {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "SoftwareApplication",
        "name": "Smoooth",
        "url": "${siteUrl}",
        "operatingSystem": "Linux (Ubuntu 24.04/22.04 LTS, Debian 12), macOS, Windows",
        "applicationCategory": "DeveloperApplication",
        "applicationSubCategory": "Infrastructure & DevOps Runtime",
        "description": "${siteDescription}",
        "license": "https://opensource.org/licenses/Apache-2.0",
        "offers": {
          "@type": "Offer",
          "price": "0",
          "priceCurrency": "USD"
        },
        "featureList": [
          "Model Context Protocol (MCP) Server Interface",
          "Docker Container Lifecycle Orchestration",
          "Stateful Database Adapters (PostgreSQL, Redis, MongoDB, SQLite)",
          "256-bit Scoped Cryptographic Bearer Tokens",
          "Transactional 2-Phase DAG Deployments with Rollbacks",
          "Zero Cloud Lock-In Self-Hosted Runtime",
          "Private Loopback Binding (127.0.0.1:4040)"
        ]
      },
      {
        "@type": "Organization",
        "name": "Smoooth",
        "url": "${siteUrl}",
        "logo": "${siteUrl}/harness/favicon.svg",
        "sameAs": [
          "https://github.com/itsraeyyy/Smoooth"
        ]
      },
      {
        "@type": "FAQPage",
        "mainEntity": [
          {
            "@type": "Question",
            "name": "How do AI coding agents like Claude Code or Cursor interact with Smoooth?",
            "acceptedAnswer": {
              "@type": "Answer",
              "text": "Smoooth implements the Model Context Protocol (MCP) standard. AI agents pair over stdio or streamable HTTP/SSE using a 256-bit bearer token. Instead of running unsafe shell scripts as root, agents invoke structured, typed tools like create_plan, execute_plan, verify_connection, and configure_ingress."
            }
          },
          {
            "@type": "Question",
            "name": "Is giving an AI coding agent Docker daemon access secure?",
            "acceptedAnswer": {
              "@type": "Answer",
              "text": "Smoooth confines operations via private loopback binding (127.0.0.1:4040), granular role-based scopes (read_only, plan, deploy, admin), systemd sandboxing with NoNewPrivileges, and append-only signed audit logging."
            }
          },
          {
            "@type": "Question",
            "name": "What databases and Linux distributions are supported?",
            "acceptedAnswer": {
              "@type": "Answer",
              "text": "Smoooth is verified on Ubuntu 24.04 LTS and Debian 12 (x86_64 and ARM64). It provides native stateful adapters for PostgreSQL, Redis, MongoDB, and SQLite with automated persistent volume management and health probes."
            }
          },
          {
            "@type": "Question",
            "name": "How does Smoooth compare to Coolify or Dokku?",
            "acceptedAnswer": {
              "@type": "Answer",
              "text": "Coolify requires 2 CPUs and 2 GB RAM minimum, running a full web dashboard, Fastify backend, and Traefik proxy. Smoooth is a single static Rust binary consuming under 28MB RAM idle, purpose-built for autonomous AI agents over MCP with transactional 2-phase DAG deployments and automatic rollbacks."
            }
          },
          {
            "@type": "Question",
            "name": "Can I run Smoooth on a low-cost $5/mo VPS or homelab?",
            "acceptedAnswer": {
              "@type": "Answer",
              "text": "Yes. Minimum requirements are 1 vCPU, 1 GB RAM, and 10 GB disk space. Because Smoooth is compiled in Rust with zero background Node.js/PHP runtime overhead, your VPS resources remain 100% dedicated to your actual application workloads."
            }
          }
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

// 3. Generate static robots.txt and sitemap.xml
const robotsTxt = `User-agent: *
Allow: /
Disallow: /api/
Disallow: /private/

Sitemap: https://smoooth.dev/sitemap.xml
Host: https://smoooth.dev
`;
fs.writeFileSync(path.join(rootDir, 'robots.txt'), robotsTxt, 'utf8');
fs.writeFileSync(path.join(distDir, 'robots.txt'), robotsTxt, 'utf8');

const sitemapXml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://smoooth.dev/</loc>
    <lastmod>2026-10-23</lastmod>
    <changefreq>daily</changefreq>
    <priority>1.0</priority>
  </url>
  <url>
    <loc>https://smoooth.dev/guides/</loc>
    <lastmod>2026-10-23</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
  </url>
  <url>
    <loc>https://smoooth.dev/guides/deploy-docker-application-claude-code</loc>
    <lastmod>2026-10-23</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.9</priority>
  </url>
  <url>
    <loc>https://smoooth.dev/compare/</loc>
    <lastmod>2026-10-23</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
  </url>
  <url>
    <loc>https://smoooth.dev/compare/smoooth-vs-coolify</loc>
    <lastmod>2026-10-23</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.9</priority>
  </url>
</urlset>
`;
fs.writeFileSync(path.join(rootDir, 'sitemap.xml'), sitemapXml, 'utf8');
fs.writeFileSync(path.join(distDir, 'sitemap.xml'), sitemapXml, 'utf8');

// 4. Generate custom 404.html
const notFoundHtml = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>404 — Page Not Found · Smoooth</title>
  <meta name="robots" content="noindex, follow"/>
  <link rel="icon" href="/favicon.ico"/>
  <style>
    body {
      background-color: #080C14;
      color: #F9FAFB;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace, sans-serif;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      margin: 0;
      padding: 24px;
      text-align: center;
    }
    .card {
      max-width: 520px;
      padding: 36px 28px;
      background: #0d121c;
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 16px;
    }
    .code {
      font-family: monospace;
      font-size: 13px;
      color: #73a3d2;
      background: rgba(0,0,0,0.4);
      padding: 4px 8px;
      border-radius: 6px;
      display: inline-block;
      margin-bottom: 16px;
    }
    h1 {
      font-size: 26px;
      margin: 0 0 12px 0;
      letter-spacing: -0.02em;
    }
    p {
      color: #94a3b8;
      font-size: 15px;
      line-height: 1.6;
      margin: 0 0 24px 0;
    }
    .btn {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 10px 20px;
      background: #ffffff;
      color: #000000;
      text-decoration: none;
      font-weight: 500;
      font-size: 14px;
      border-radius: 9999px;
      transition: opacity 0.2s;
    }
    .btn:hover {
      opacity: 0.9;
    }
  </style>
</head>
<body>
  <div class="card">
    <div class="code">HTTP 404 // ROUTE_NOT_FOUND</div>
    <h1>The requested resource does not exist.</h1>
    <p>The page or endpoint you are looking for is not mapped in the Smoooth runtime router.</p>
    <a href="/" class="btn">&larr; Return to Smoooth Home</a>
  </div>
</body>
</html>`;
fs.writeFileSync(path.join(rootDir, '404.html'), notFoundHtml, 'utf8');
fs.writeFileSync(path.join(distDir, '404.html'), notFoundHtml, 'utf8');

// 5. Build and sync Guides
const guidesDir = path.join(rootDir, 'guides');
const distGuidesDir = path.join(distDir, 'guides');
if (fs.existsSync(guidesDir)) {
  fs.mkdirSync(distGuidesDir, { recursive: true });
  for (const file of fs.readdirSync(guidesDir)) {
    if (file.endsWith('.html')) {
      const srcFile = path.join(guidesDir, file);
      const destFile = path.join(distGuidesDir, file);
      fs.copyFileSync(srcFile, destFile);

      // Support clean routing: /guides/foo -> /guides/foo/index.html
      const slug = path.basename(file, '.html');
      if (slug !== 'index') {
        const slugDir = path.join(distGuidesDir, slug);
        fs.mkdirSync(slugDir, { recursive: true });
        fs.copyFileSync(srcFile, path.join(slugDir, 'index.html'));
      }
    }
  }
}

// 6. Build and sync Comparisons
const compareDir = path.join(rootDir, 'compare');
const distCompareDir = path.join(distDir, 'compare');
if (fs.existsSync(compareDir)) {
  fs.mkdirSync(distCompareDir, { recursive: true });
  for (const file of fs.readdirSync(compareDir)) {
    if (file.endsWith('.html')) {
      const srcFile = path.join(compareDir, file);
      const destFile = path.join(distCompareDir, file);
      fs.copyFileSync(srcFile, destFile);

      // Support clean routing: /compare/foo -> /compare/foo/index.html
      const slug = path.basename(file, '.html');
      if (slug !== 'index') {
        const slugDir = path.join(distCompareDir, slug);
        fs.mkdirSync(slugDir, { recursive: true });
        fs.copyFileSync(srcFile, path.join(slugDir, 'index.html'));
      }
    }
  }
}

// 7. Copy public assets (harness, fonts, favicon, og-image, _headers)
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

if (fs.existsSync(path.join(rootDir, 'frontend/public/favicon.ico'))) {
  fs.copyFileSync(path.join(rootDir, 'frontend/public/favicon.ico'), path.join(distDir, 'favicon.ico'));
  fs.copyFileSync(path.join(rootDir, 'frontend/public/favicon.ico'), path.join(rootDir, 'favicon.ico'));
}

if (fs.existsSync(path.join(rootDir, 'frontend/public/og-image.png'))) {
  fs.copyFileSync(path.join(rootDir, 'frontend/public/og-image.png'), path.join(distDir, 'og-image.png'));
  fs.copyFileSync(path.join(rootDir, 'frontend/public/og-image.png'), path.join(rootDir, 'og-image.png'));
}

if (fs.existsSync(path.join(rootDir, '_headers'))) {
  fs.copyFileSync(path.join(rootDir, '_headers'), path.join(distDir, '_headers'));
}

console.log('✓ Successfully synchronized Smoooth landing page, robots.txt, sitemap.xml, 404.html, and headers to root and dist/');
