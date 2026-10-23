const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const interactionScript = fs.readFileSync(path.join(rootDir, 'scripts/ferndesk-interaction.js'), 'utf8');

const smooothHeader = `
<a href="/brand" class="group focus-ring bg-ink-900 text-on-ink hover:bg-ink-raised block transition-colors border-b border-border-soft">
  <div class="container-marketing flex min-h-10 items-center justify-center gap-x-3 gap-y-1 py-2 text-sm leading-tight">
    <span class="text-spark hidden shrink-0 text-2xs font-semibold tracking-label uppercase sm:inline">New</span>
    <span class="text-on-ink-muted text-center text-pretty">Deploy stateful apps on your Linux VPS with Claude Code &amp; Cursor over MCP.</span>
    <span class="inline-flex shrink-0 items-center gap-1 font-medium text-spark underline-offset-4 group-hover:underline">Explore Brand Kit <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="transition-transform group-hover:translate-x-0.5"><path d="M5 12h14"></path><path d="m12 5 7 7-7 7"></path></svg></span>
  </div>
</a>
<header class="sticky top-0 z-40 w-full backdrop-blur-md bg-bg/90">
  <div class="border-border bg-bg/80 border-b">
    <div class="container-marketing relative flex h-16 items-center justify-between gap-4">
      <a href="/" class="focus-ring flex items-center gap-2.5 rounded">
        <span class="inline-block shrink-0" style="width: 28px; height: 28px;">
          <svg width="100%" height="100%" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect width="32" height="32" rx="7" fill="#121417"/>
            <rect x="0.5" y="0.5" width="31" height="31" rx="6.5" stroke="#383f4a"/>
            <path d="M10 11C10 9 22 9 22 13C22 17 11 17 11 21C11 24 22 24 22 22" stroke="#c9f27a" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
            <circle cx="22" cy="22" r="1.75" fill="#00f076"/>
          </svg>
        </span>
        <span class="font-display text-fg text-xl leading-none font-semibold tracking-[-0.02em]">Smoooth</span>
      </a>
      <nav class="hidden h-16 items-center gap-1 lg:flex" aria-label="Main">
        <a href="/#capabilities" class="focus-ring text-prose hover:bg-bg-muted hover:text-fg rounded border border-transparent px-3 py-[7px] text-base transition-colors">Capabilities</a>
        <a href="/#runtime" class="focus-ring text-prose hover:bg-bg-muted hover:text-fg rounded border border-transparent px-3 py-[7px] text-base transition-colors">Runtime</a>
        <a href="/#migration" class="focus-ring text-prose hover:bg-bg-muted hover:text-fg rounded border border-transparent px-3 py-[7px] text-base transition-colors">Cost Compare</a>
        <a href="/#faq" class="focus-ring text-prose hover:bg-bg-muted hover:text-fg rounded border border-transparent px-3 py-[7px] text-base transition-colors">Architecture FAQ</a>
        <a href="/brand" class="focus-ring text-prose hover:bg-bg-muted hover:text-fg rounded border border-transparent px-3 py-[7px] text-base transition-colors">Brand &amp; Kit</a>
      </nav>
      <div class="hidden items-center gap-2.5 lg:flex">
        <a href="https://github.com/itsraeyyy/Smoooth" target="_blank" rel="noopener noreferrer" class="focus-ring text-prose hover:bg-bg-muted hover:text-fg rounded border border-transparent px-3 py-[7px] text-base transition-colors">GitHub</a>
        <a href="/#hero" class="btn-face relative whitespace-nowrap btn-primary px-3 py-[7px] text-sm" aria-label="Install Smoooth">
          <span class="inline-flex items-center gap-2 font-semibold">Get Started</span>
        </a>
      </div>
      <button type="button" id="mobile-menu-btn" class="focus-ring border-border-strong bg-bg-raised text-fg shadow-control flex size-9 items-center justify-center rounded border lg:hidden" aria-label="Open menu" aria-expanded="false" aria-controls="mobile-nav-panel">
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" x2="20" y1="12" y2="12"></line><line x1="4" x2="20" y1="6" y2="6"></line><line x1="4" x2="20" y1="18" y2="18"></line></svg>
      </button>
    </div>
  </div>
  <div id="mobile-nav-panel" class="hidden lg:hidden border-b border-border bg-ink-950 px-4 py-4 space-y-2">
    <a href="/#capabilities" class="block py-2 text-base text-bone font-medium hover:text-spark">Capabilities</a>
    <a href="/#runtime" class="block py-2 text-base text-bone font-medium hover:text-spark">Runtime</a>
    <a href="/#migration" class="block py-2 text-base text-bone font-medium hover:text-spark">Cost Compare</a>
    <a href="/#faq" class="block py-2 text-base text-bone font-medium hover:text-spark">Architecture FAQ</a>
    <a href="/brand" class="block py-2 text-base text-bone font-medium hover:text-spark">Brand Guidelines &amp; Media Kit</a>
    <a href="https://github.com/itsraeyyy/Smoooth" target="_blank" rel="noopener noreferrer" class="block py-2 text-base text-spark font-medium">GitHub Repo →</a>
  </div>
</header>
`;

const smooothFooter = `
<footer class="bg-ink-900 text-slate-300">
  <div aria-hidden="true" class="relative flex-none h-[var(--seam-size)] w-full" style="--seam-size: 40px; --dither-color: var(--spark);">
    <div class="absolute inset-0 bg-[image:repeating-conic-gradient(var(--dither-color)_0_25%,transparent_0_50%)] bg-[size:8px_8px] bg-[position:0_0]"></div>
    <div class="absolute inset-0 opacity-60 top-1/2 bg-[image:repeating-conic-gradient(var(--dither-color)_0_25%,transparent_0_50%)] bg-[size:16px_16px] bg-[position:4px_0]"></div>
  </div>
  <div class="container-marketing pb-12 pt-14">
    <div class="grid grid-cols-2 gap-x-6 gap-y-10 text-sm sm:grid-cols-3 lg:grid-cols-5 lg:items-start">
      <!-- Col 1: Brand & Tagline -->
      <div class="col-span-2 sm:col-span-3 lg:col-span-1 flex flex-col gap-3.5">
        <a href="/" class="flex items-center gap-2.5 text-on-ink">
          <span class="inline-block shrink-0" style="width: 24px; height: 24px;">
            <svg width="100%" height="100%" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
              <rect width="32" height="32" rx="7" fill="#0a0b0d"/>
              <path d="M10 11C10 9 22 9 22 13C22 17 11 17 11 21C11 24 22 24 22 22" stroke="#c9f27a" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
              <circle cx="22" cy="22" r="1.75" fill="#00f076"/>
            </svg>
          </span>
          <span class="font-display text-lg font-semibold tracking-tight text-bone">Smoooth</span>
        </a>
        <p class="text-xs text-on-ink-muted leading-relaxed">
          Autonomous deployment runtime for AI coding agents. Provision stateful Docker containers, managed PostgreSQL, and automated ingress over MCP on your own Linux VPS.
        </p>
        <div class="flex items-center gap-2 text-2xs font-mono text-fern-600">
          <span class="size-2 rounded-full bg-fern-600 animate-pulse"></span>
          <span>All daemon systems nominal</span>
        </div>
      </div>

      <!-- Col 2: Runtime -->
      <div class="flex flex-col gap-3">
        <span class="font-mono text-2xs uppercase tracking-wider text-spark font-semibold">Runtime</span>
        <a href="/#capabilities" class="text-xs text-on-ink-muted hover:text-bone transition-colors">Bento Capabilities</a>
        <a href="/#runtime" class="text-xs text-on-ink-muted hover:text-bone transition-colors">Autonomous Ingress</a>
        <a href="/#capabilities" class="text-xs text-on-ink-muted hover:text-bone transition-colors">2-Phase DAG Engine</a>
        <a href="/#capabilities" class="text-xs text-on-ink-muted hover:text-bone transition-colors">State Fencing &amp; SQLite</a>
        <a href="/#migration" class="text-xs text-on-ink-muted hover:text-bone transition-colors">Cost Compare</a>
      </div>

      <!-- Col 3: Brand & Media Kit -->
      <div class="flex flex-col gap-3">
        <span class="font-mono text-2xs uppercase tracking-wider text-spark font-semibold">Brand &amp; Press</span>
        <a href="/brand" class="text-xs text-on-ink-muted hover:text-bone transition-colors">Brand Guidelines</a>
        <a href="/mediakit" class="text-xs text-on-ink-muted hover:text-bone transition-colors">Media Kit Downloads</a>
        <a href="/assets/brand/tokens.json" class="text-xs text-on-ink-muted hover:text-bone transition-colors">Design Tokens JSON</a>
        <a href="/assets/brand/tokens.css" class="text-xs text-on-ink-muted hover:text-bone transition-colors">Tokens Stylesheet</a>
        <a href="/assets/brand/smoooth-badges.md" class="text-xs text-on-ink-muted hover:text-bone transition-colors">GitHub Badges</a>
      </div>

      <!-- Col 4: Ecosystem -->
      <div class="flex flex-col gap-3">
        <span class="font-mono text-2xs uppercase tracking-wider text-spark font-semibold">Ecosystem</span>
        <span class="text-xs text-on-ink-muted">Claude Code (Anthropic)</span>
        <span class="text-xs text-on-ink-muted">Cursor Agent</span>
        <span class="text-xs text-on-ink-muted">Windsurf Cascade</span>
        <span class="text-xs text-on-ink-muted">Ubuntu 24.04 / Debian 12</span>
        <span class="text-xs text-on-ink-muted">Docker 24+ &amp; PostgreSQL 16</span>
      </div>

      <!-- Col 5: Developers -->
      <div class="flex flex-col gap-3">
        <span class="font-mono text-2xs uppercase tracking-wider text-spark font-semibold">Developers</span>
        <a href="https://github.com/itsraeyyy/Smoooth" target="_blank" rel="noopener noreferrer" class="text-xs text-on-ink-muted hover:text-bone transition-colors">GitHub Repository</a>
        <a href="/#faq" class="text-xs text-on-ink-muted hover:text-bone transition-colors">Architecture FAQ</a>
        <a href="mailto:press@smoooth.dev" class="text-xs text-on-ink-muted hover:text-bone transition-colors">press@smoooth.dev</a>
        <div class="mt-2 p-2.5 rounded bg-ink-950 border border-border-soft">
          <div class="text-2xs font-mono text-on-ink-muted mb-1">One-Liner Install:</div>
          <code class="text-2xs font-mono text-spark break-all">curl -fsSL https://smoooth.dev/install.sh | bash</code>
        </div>
      </div>
    </div>

    <!-- Bottom subfooter -->
    <div class="mt-12 pt-6 border-t border-border-soft flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-on-ink-muted">
      <div>© 2026 Smoooth Runtime Inc. Built for sovereign AI agent deployments.</div>
      <div class="flex items-center gap-4 text-2xs font-mono">
        <span>MODEL CONTEXT PROTOCOL NATIVE</span>
        <span>•</span>
        <span>ZERO PAAS MARKUP</span>
      </div>
    </div>
  </div>
</footer>
`;

function transformPage(content) {
  // 1. Remove Pirsch script in <head>
  content = content.replace(/<script[^>]*src="https:\/\/psch\.ferndesk\.com[^"]*"[^>]*><\/script>/gi, '');

  // 2. Locate header: from banner start '<a href="https://ferndesk.com/app/signup"' to '</header>'
  const bannerMarker = '<a href="https://ferndesk.com/app/signup" class="group focus-ring';
  const bIdx = content.indexOf(bannerMarker);
  const hEndIdx = content.indexOf('</header>');
  if (bIdx !== -1 && hEndIdx !== -1 && hEndIdx > bIdx) {
    content = content.slice(0, bIdx) + smooothHeader.trim() + content.slice(hEndIdx + 9);
  } else {
    // If banner already replaced
    const b2Idx = content.indexOf('<a href="/brand" class="group focus-ring');
    if (b2Idx !== -1 && hEndIdx !== -1 && hEndIdx > b2Idx) {
      content = content.slice(0, b2Idx) + smooothHeader.trim() + content.slice(hEndIdx + 9);
    }
  }

  // 3. Locate footer: from '<footer' to '</footer>'
  const fIdx = content.indexOf('<footer');
  const fEndIdx = content.indexOf('</footer>');
  if (fIdx !== -1 && fEndIdx !== -1 && fEndIdx > fIdx) {
    content = content.slice(0, fIdx) + smooothFooter.trim() + content.slice(fEndIdx + 9);
  }

  // 4. Locate </main> and replace everything between </main> and </body>
  const mainEndIdx = content.indexOf('</main>');
  const bodyEndIdx = content.indexOf('</body>');
  if (mainEndIdx !== -1 && bodyEndIdx !== -1 && bodyEndIdx > mainEndIdx) {
    const cleanTail = `</main>
</div>
<script id="ferndesk-interactive-runtime">
${interactionScript}
</script>
`;
    content = content.slice(0, mainEndIdx) + cleanTail + content.slice(bodyEndIdx);
  }

  return content;
}

// Transform index.html
const indexHtmlPath = path.join(rootDir, 'index.html');
const indexHtml = fs.readFileSync(indexHtmlPath, 'utf8');
const newIndex = transformPage(indexHtml);
fs.writeFileSync(indexHtmlPath, newIndex, 'utf8');
console.log('Successfully transformed index.html! New size:', newIndex.length);

// Transform brand.html
const brandHtmlPath = path.join(rootDir, 'brand.html');
if (fs.existsSync(brandHtmlPath)) {
  const brandHtml = fs.readFileSync(brandHtmlPath, 'utf8');
  const newBrand = transformPage(brandHtml);
  fs.writeFileSync(brandHtmlPath, newBrand, 'utf8');
  console.log('Successfully transformed brand.html! New size:', newBrand.length);
}

// Transform mediakit.html
const mediakitHtmlPath = path.join(rootDir, 'mediakit.html');
if (fs.existsSync(mediakitHtmlPath)) {
  const mediaHtml = fs.readFileSync(mediakitHtmlPath, 'utf8');
  const newMedia = transformPage(mediaHtml);
  fs.writeFileSync(mediakitHtmlPath, newMedia, 'utf8');
  console.log('Successfully transformed mediakit.html! New size:', newMedia.length);
}
