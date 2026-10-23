const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const indexHtml = fs.readFileSync(path.join(rootDir, 'index.html'), 'utf8');

// 1. Extract head up to </head>
const headMatch = indexHtml.match(/<head>([\s\S]*?)<\/head>/);
if (!headMatch) {
  throw new Error('Could not find <head> in index.html');
}

let headContent = headMatch[1];
// Update Title & Meta
headContent = headContent.replace(/<title>[\s\S]*?<\/title>/i, '<title>Brand Guidelines & Media Kit — Smoooth</title>');
headContent = headContent.replace(/<meta\s+name="description"\s+content="[^"]*"/i, '<meta name="description" content="Official Brand Guidelines, Design Tokens, Lexend Typography Scale, and Media Kit for Smoooth — the autonomous AI coding agent runtime."');
headContent = headContent.replace(/<meta\s+property="og:title"\s+content="[^"]*"/i, '<meta property="og:title" content="Brand Guidelines & Media Kit — Smoooth"');
headContent = headContent.replace(/<meta\s+property="og:description"\s+content="[^"]*"/i, '<meta property="og:description" content="Official Brand Guidelines, Design Tokens, Lexend Typography Scale, and Media Kit for Smoooth."');

// 2. Extract wrapper before <main>
const bodyStart = indexHtml.indexOf('<body');
const mainStart = indexHtml.indexOf('<main');
const beforeMain = indexHtml.slice(bodyStart, mainStart);

// 3. Extract wrapper after </main>
const mainEnd = indexHtml.indexOf('</main>');
const afterMain = indexHtml.slice(mainEnd + 7);

// 4. Construct rich <main> section for Brand Guidelines & Media Kit
const mainContent = `
<main class="bg-bg text-fg">
  <!-- Brand Header / Hero -->
  <section class="relative overflow-hidden border-b border-border bg-ink-950 py-16 lg:py-24">
    <div class="container-marketing max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex flex-col gap-6">
        <div class="flex items-center gap-2 text-xs font-mono uppercase tracking-wider text-spark">
          <span class="inline-block size-2 rounded-full bg-fern-600"></span>
          <span>Official Identity System • Version 1.0</span>
        </div>

        <h1 class="text-display-xl lg:text-display-2xl font-bold tracking-tight text-bone">
          Brand Guidelines & Media Kit
        </h1>

        <p class="text-xl lg:text-2xl text-on-ink-muted max-w-3xl leading-relaxed font-light">
          The complete visual language, design tokens, Lexend typography scale, and press assets for Smoooth — the autonomous deployment runtime for AI coding agents.
        </p>

        <!-- Jump Navigation Bar -->
        <div class="mt-4 flex flex-wrap gap-2 pt-4 border-t border-border-soft">
          <a href="#narrative" class="focus-ring px-3 py-1.5 rounded text-xs font-medium bg-ink-900 border border-border-soft text-prose hover:text-bone hover:border-spark transition-colors">1. Narrative</a>
          <a href="#logo" class="focus-ring px-3 py-1.5 rounded text-xs font-medium bg-ink-900 border border-border-soft text-prose hover:text-bone hover:border-spark transition-colors">2. Logo & Geometry</a>
          <a href="#typography" class="focus-ring px-3 py-1.5 rounded text-xs font-medium bg-ink-900 border border-border-soft text-prose hover:text-bone hover:border-spark transition-colors">3. Typography (Lexend)</a>
          <a href="#colors" class="focus-ring px-3 py-1.5 rounded text-xs font-medium bg-ink-900 border border-border-soft text-prose hover:text-bone hover:border-spark transition-colors">4. Color System</a>
          <a href="#voice" class="focus-ring px-3 py-1.5 rounded text-xs font-medium bg-ink-900 border border-border-soft text-prose hover:text-bone hover:border-spark transition-colors">5. Voice & Anti-Slop</a>
          <a href="#boilerplate" class="focus-ring px-3 py-1.5 rounded text-xs font-medium bg-ink-900 border border-border-soft text-prose hover:text-bone hover:border-spark transition-colors">6. Press Boilerplate</a>
          <a href="#downloads" class="focus-ring px-3 py-1.5 rounded text-xs font-medium bg-spark text-spark-ink font-semibold hover:bg-bone transition-colors">7. Download Assets</a>
        </div>
      </div>
    </div>
  </section>

  <!-- 1. Brand Narrative -->
  <section id="narrative" class="py-16 lg:py-20 border-b border-border-soft">
    <div class="container-marketing max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="mb-8">
        <h2 class="text-display-md lg:text-display-lg font-bold text-bone tracking-tight">1. Brand Narrative & Mission</h2>
        <p class="mt-2 text-base text-on-ink-muted">The core purpose, engineering philosophy, and architectural promise.</p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div class="p-6 rounded-lg bg-ink-900 border border-border-soft">
          <div class="text-xs font-mono text-spark mb-3">TENET 01</div>
          <h3 class="text-title-lg font-semibold text-bone mb-2">Sovereign Persistence</h3>
          <p class="text-sm text-on-ink-muted leading-relaxed">
            AI agents shouldn't deploy to fragile, ephemeral cloud sandboxes with steep markups. Smoooth targets dedicated Linux servers with real Docker persistence and PostgreSQL storage.
          </p>
        </div>

        <div class="p-6 rounded-lg bg-ink-900 border border-border-soft">
          <div class="text-xs font-mono text-fern-600 mb-3">TENET 02</div>
          <h3 class="text-title-lg font-semibold text-bone mb-2">Protocol Native (MCP)</h3>
          <p class="text-sm text-on-ink-muted leading-relaxed">
            No proprietary CLI gymnastics or web dashboard lock-in. Smoooth implements Anthropic's Model Context Protocol (MCP) natively, allowing Claude Code, Cursor, and Windsurf to orchestrate deployments autonomously.
          </p>
        </div>

        <div class="p-6 rounded-lg bg-ink-900 border border-border-soft">
          <div class="text-xs font-mono text-glacier mb-3">TENET 03</div>
          <h3 class="text-title-lg font-semibold text-bone mb-2">Less Is More</h3>
          <p class="text-sm text-on-ink-muted leading-relaxed">
            Removing is harder than adding. We reject decorative fluff, kicker pills, and hype cycles. We communicate with real terminal telemetry, verified schemas, and atomic rollbacks.
          </p>
        </div>
      </div>
    </div>
  </section>

  <!-- 2. Logo System & Geometry -->
  <section id="logo" class="py-16 lg:py-20 border-b border-border-soft bg-ink-950">
    <div class="container-marketing max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="mb-8">
        <h2 class="text-display-md lg:text-display-lg font-bold text-bone tracking-tight">2. Logo System & Geometry</h2>
        <p class="mt-2 text-base text-on-ink-muted">Clear-space dimensions, sizing constraints, and placeholder alignment grid.</p>
      </div>

      <!-- User Logo Placeholder Notice -->
      <div class="mb-8 p-4 rounded-lg bg-ink-900 border border-spark/30 flex items-start gap-3">
        <span class="text-spark font-mono text-lg shrink-0">ⓘ</span>
        <div class="text-sm">
          <span class="font-semibold text-bone">Official Logo Slot Reserved:</span>
          <span class="text-on-ink-muted"> The vector placeholder below represents the approved proportions and clear-space envelope. Drop your finalized logo vector directly into <code class="px-1.5 py-0.5 rounded bg-ink-800 text-spark font-mono text-xs">/assets/brand/logo.svg</code> to replace it across all documentation and production templates.</span>
        </div>
      </div>

      <!-- Visual Specs Grid -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-10">
        <!-- Dark Canvas Specimen -->
        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between text-xs font-mono text-on-ink-muted">
            <span>DARK CANVAS (PRIMARY)</span>
            <span class="text-spark">1.0X SAFE BOUNDARY</span>
          </div>
          <div class="p-6 rounded-lg bg-ink-900 border border-border-soft flex items-center justify-center min-h-56 overflow-hidden">
            <img src="/assets/brand/logo-spec-dark.svg" alt="Smoooth Logo Dark Specification" class="w-full max-w-md h-auto" />
          </div>
        </div>

        <!-- Light Canvas Specimen -->
        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between text-xs font-mono text-on-ink-muted">
            <span>LIGHT / PRESS CANVAS (SECONDARY)</span>
            <span class="text-fern-600">HIGH CONTRAST</span>
          </div>
          <div class="p-6 rounded-lg bg-white border border-slate-200 flex items-center justify-center min-h-56 overflow-hidden">
            <img src="/assets/brand/logo-spec-light.svg" alt="Smoooth Logo Light Specification" class="w-full max-w-md h-auto" />
          </div>
        </div>
      </div>

      <!-- Clear Space & Scaling Specs -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-6 mb-10">
        <div class="p-5 rounded-lg bg-ink-900 border border-border-soft">
          <div class="text-xs font-mono text-spark mb-1">CLEAR SPACE</div>
          <div class="text-xl font-bold text-bone">1.0X Height</div>
          <p class="mt-2 text-xs text-on-ink-muted leading-relaxed">
            Maintain clear margin equal to the cap-height of the letter 'S' on all four sides. No UI, borders, or text may breach this zone.
          </p>
        </div>

        <div class="p-5 rounded-lg bg-ink-900 border border-border-soft">
          <div class="text-xs font-mono text-spark mb-1">MINIMUM SIZE (MARK)</div>
          <div class="text-xl font-bold text-bone">24px × 24px</div>
          <p class="mt-2 text-xs text-on-ink-muted leading-relaxed">
            For favicons, terminal badges, and micro UI indicators. Never render mark below 24px in digital displays (7mm print).
          </p>
        </div>

        <div class="p-5 rounded-lg bg-ink-900 border border-border-soft">
          <div class="text-xs font-mono text-spark mb-1">MINIMUM SIZE (LOCKUP)</div>
          <div class="text-xl font-bold text-bone">120px Width</div>
          <p class="mt-2 text-xs text-on-ink-muted leading-relaxed">
            Full wordmark lockup requires minimum 120px width (28px height) to ensure optical legibility of the 'ooo' character glyphs.
          </p>
        </div>
      </div>

      <!-- Dos & Don'ts -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
        <div class="p-5 rounded-lg bg-ink-900/60 border border-emerald-900/50">
          <div class="text-xs font-mono text-emerald-400 font-semibold mb-3">✓ APPROVED USAGE</div>
          <ul class="space-y-2 text-xs text-on-ink-muted">
            <li class="flex items-center gap-2"><span class="text-emerald-400">✓</span> Use official SVG vector files without alteration.</li>
            <li class="flex items-center gap-2"><span class="text-emerald-400">✓</span> Maintain 1.0X clear space on all edges.</li>
            <li class="flex items-center gap-2"><span class="text-emerald-400">✓</span> Render on deep obsidian (#0a0b0d) or clean white (#ffffff).</li>
            <li class="flex items-center gap-2"><span class="text-emerald-400">✓</span> Keep aspect ratio locked when resizing.</li>
          </ul>
        </div>

        <div class="p-5 rounded-lg bg-ink-900/60 border border-red-900/50">
          <div class="text-xs font-mono text-red-400 font-semibold mb-3">✗ STRICTLY PROHIBITED</div>
          <ul class="space-y-2 text-xs text-on-ink-muted">
            <li class="flex items-center gap-2"><span class="text-red-400">✗</span> Never stretch, squeeze, or skew the logo geometry.</li>
            <li class="flex items-center gap-2"><span class="text-red-400">✗</span> Never wrap the logo inside decorative pill eyebrow badges.</li>
            <li class="flex items-center gap-2"><span class="text-red-400">✗</span> Never apply drop shadows, 3D extrusions, or outer glows.</li>
            <li class="flex items-center gap-2"><span class="text-red-400">✗</span> Never recolor with unauthorized neon or pastel palettes.</li>
          </ul>
        </div>
      </div>
    </div>
  </section>

  <!-- 3. Typography System: Lexend -->
  <section id="typography" class="py-16 lg:py-20 border-b border-border-soft">
    <div class="container-marketing max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="mb-8">
        <h2 class="text-display-md lg:text-display-lg font-bold text-bone tracking-tight">3. Typography System: Lexend</h2>
        <p class="mt-2 text-base text-on-ink-muted">Lexend is our single unified typographic standard for all headings, body text, and interfaces.</p>
      </div>

      <!-- Live Interactive Specimen Sandbox -->
      <div class="mb-10 p-6 rounded-lg bg-ink-900 border border-border-soft">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6 pb-4 border-b border-border-soft">
          <div>
            <div class="text-xs font-mono text-spark">INTERACTIVE SPECIMEN SANDBOX</div>
            <div class="text-sm text-bone font-medium">Type any headline, quote, or terminal label to preview Lexend in real time</div>
          </div>
          <div class="flex items-center gap-2">
            <button type="button" class="specimen-wt-btn active px-2.5 py-1 rounded text-xs font-mono border border-spark bg-spark text-spark-ink" data-wt="600">600 SemiBold</button>
            <button type="button" class="specimen-wt-btn px-2.5 py-1 rounded text-xs font-mono border border-border-soft text-prose hover:border-spark" data-wt="700">700 Bold</button>
            <button type="button" class="specimen-wt-btn px-2.5 py-1 rounded text-xs font-mono border border-border-soft text-prose hover:border-spark" data-wt="400">400 Regular</button>
            <button type="button" class="specimen-wt-btn px-2.5 py-1 rounded text-xs font-mono border border-border-soft text-prose hover:border-spark" data-wt="300">300 Light</button>
          </div>
        </div>

        <input
          type="text"
          id="specimen-input"
          value="Infrastructure freedom for AI coding agents."
          class="w-full bg-ink-950 border border-border-soft rounded px-4 py-3 text-bone text-base focus:border-spark focus:outline-none mb-6"
          placeholder="Type here to test typography..."
        />

        <div class="space-y-6">
          <div>
            <div class="text-xs font-mono text-on-ink-muted mb-1">Display 2XL (64px)</div>
            <div id="preview-2xl" class="text-4xl sm:text-6xl font-semibold text-bone tracking-tight break-words">
              Infrastructure freedom for AI coding agents.
            </div>
          </div>

          <div>
            <div class="text-xs font-mono text-on-ink-muted mb-1">Display MD (32px)</div>
            <div id="preview-md" class="text-2xl sm:text-3xl font-semibold text-bone tracking-tight break-words">
              Infrastructure freedom for AI coding agents.
            </div>
          </div>

          <div>
            <div class="text-xs font-mono text-on-ink-muted mb-1">Body Large (18px)</div>
            <div id="preview-lg" class="text-lg font-normal text-on-ink-muted leading-relaxed break-words">
              Infrastructure freedom for AI coding agents.
            </div>
          </div>
        </div>
      </div>

      <!-- Type Scale Table -->
      <div class="overflow-x-auto rounded-lg border border-border-soft bg-ink-900">
        <table class="w-full text-left text-xs">
          <thead class="border-b border-border-soft bg-ink-950 text-on-ink-muted font-mono uppercase">
            <tr>
              <th class="py-3 px-4">Level</th>
              <th class="py-3 px-4">Size</th>
              <th class="py-3 px-4">Line Height</th>
              <th class="py-3 px-4">Tracking</th>
              <th class="py-3 px-4">Weight</th>
              <th class="py-3 px-4">Application</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border-soft text-prose">
            <tr>
              <td class="py-3 px-4 font-mono text-spark">display-2xl</td>
              <td class="py-3 px-4 text-bone font-medium">80px</td>
              <td class="py-3 px-4">1.00</td>
              <td class="py-3 px-4">-0.035em</td>
              <td class="py-3 px-4">600 SemiBold</td>
              <td class="py-3 px-4">Hero main title (desktop)</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-mono text-spark">display-xl</td>
              <td class="py-3 px-4 text-bone font-medium">64px</td>
              <td class="py-3 px-4">1.02</td>
              <td class="py-3 px-4">-0.035em</td>
              <td class="py-3 px-4">600 SemiBold</td>
              <td class="py-3 px-4">Page headers, major sections</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-mono text-spark">display-lg</td>
              <td class="py-3 px-4 text-bone font-medium">48px</td>
              <td class="py-3 px-4">1.08</td>
              <td class="py-3 px-4">-0.030em</td>
              <td class="py-3 px-4">600 SemiBold</td>
              <td class="py-3 px-4">Section headings</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-mono text-spark">title-lg</td>
              <td class="py-3 px-4 text-bone font-medium">24px</td>
              <td class="py-3 px-4">1.20</td>
              <td class="py-3 px-4">-0.020em</td>
              <td class="py-3 px-4">600 SemiBold</td>
              <td class="py-3 px-4">Bento card headers, modal titles</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-mono text-spark">body-lg</td>
              <td class="py-3 px-4 text-bone font-medium">18px</td>
              <td class="py-3 px-4">1.45</td>
              <td class="py-3 px-4">0</td>
              <td class="py-3 px-4">400 Regular</td>
              <td class="py-3 px-4">Lead paragraphs, feature descriptions</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-mono text-spark">body-base</td>
              <td class="py-3 px-4 text-bone font-medium">15px</td>
              <td class="py-3 px-4">1.50</td>
              <td class="py-3 px-4">0</td>
              <td class="py-3 px-4">400 Regular</td>
              <td class="py-3 px-4">Standard article prose, FAQ items</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-mono text-spark">mono-code</td>
              <td class="py-3 px-4 text-bone font-medium">13px</td>
              <td class="py-3 px-4">1.45</td>
              <td class="py-3 px-4">0</td>
              <td class="py-3 px-4">400 Regular</td>
              <td class="py-3 px-4">CLI lines, MCP payload JSON, terminal logs</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </section>

  <!-- 4. Color System & Design Tokens -->
  <section id="colors" class="py-16 lg:py-20 border-b border-border-soft bg-ink-950">
    <div class="container-marketing max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="mb-8">
        <h2 class="text-display-md lg:text-display-lg font-bold text-bone tracking-tight">4. Color System & Design Tokens</h2>
        <p class="mt-2 text-base text-on-ink-muted">Dark obsidian surfaces paired with high-voltage spark highlights and bone neutrals. Click any swatch to copy its HEX value.</p>
      </div>

      <!-- Toast Notification Placeholder -->
      <div id="copy-toast" class="fixed bottom-6 right-6 z-50 transform translate-y-12 opacity-0 transition-all duration-200 pointer-events-none px-4 py-2.5 rounded bg-spark text-spark-ink font-mono text-xs font-semibold shadow-lg">
        Hex code copied to clipboard!
      </div>

      <!-- Color Swatch Grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
        <!-- Spark -->
        <div class="color-card p-4 rounded-lg bg-ink-900 border border-border-soft hover:border-spark transition-colors group cursor-pointer" data-copy="#c9f27a">
          <div class="h-20 w-full rounded mb-3 flex items-end p-2.5 transition-transform group-hover:scale-[1.01]" style="background-color: #c9f27a;">
            <span class="text-xs font-mono font-bold text-[#4e7a14]">#c9f27a</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm font-semibold text-bone">Spark (Primary Accent)</span>
            <span class="text-2xs font-mono text-spark uppercase group-hover:underline">Click to Copy</span>
          </div>
          <div class="mt-2 text-xs font-mono text-on-ink-muted">var(--spark) • 14.8:1 AAA</div>
          <p class="mt-2 text-xs text-on-ink-muted">Interactive highlights, CLI prompt symbols, primary buttons.</p>
        </div>

        <!-- Fern 600 -->
        <div class="color-card p-4 rounded-lg bg-ink-900 border border-border-soft hover:border-fern-600 transition-colors group cursor-pointer" data-copy="#00f076">
          <div class="h-20 w-full rounded mb-3 flex items-end p-2.5 transition-transform group-hover:scale-[1.01]" style="background-color: #00f076;">
            <span class="text-xs font-mono font-bold text-[#0a0b0d]">#00f076</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm font-semibold text-bone">Fern 600 (Live Daemon)</span>
            <span class="text-2xs font-mono text-fern-600 uppercase group-hover:underline">Click to Copy</span>
          </div>
          <div class="mt-2 text-xs font-mono text-on-ink-muted">var(--fern-600) • 12.1:1 AAA</div>
          <p class="mt-2 text-xs text-on-ink-muted">Healthy heartbeat, active server status, verified state.</p>
        </div>

        <!-- Ink 950 -->
        <div class="color-card p-4 rounded-lg bg-ink-900 border border-border-soft hover:border-border-strong transition-colors group cursor-pointer" data-copy="#0a0b0d">
          <div class="h-20 w-full rounded mb-3 flex items-end p-2.5 border border-border-soft transition-transform group-hover:scale-[1.01]" style="background-color: #0a0b0d;">
            <span class="text-xs font-mono font-bold text-bone">#0a0b0d</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm font-semibold text-bone">Ink 950 (Canvas)</span>
            <span class="text-2xs font-mono text-on-ink-muted uppercase group-hover:underline">Click to Copy</span>
          </div>
          <div class="mt-2 text-xs font-mono text-on-ink-muted">var(--ink-950) • Root Canvas</div>
          <p class="mt-2 text-xs text-on-ink-muted">Base root background for the entire application and website.</p>
        </div>

        <!-- Ink 900 -->
        <div class="color-card p-4 rounded-lg bg-ink-900 border border-border-soft hover:border-border-strong transition-colors group cursor-pointer" data-copy="#121417">
          <div class="h-20 w-full rounded mb-3 flex items-end p-2.5 border border-border-soft transition-transform group-hover:scale-[1.01]" style="background-color: #121417;">
            <span class="text-xs font-mono font-bold text-bone">#121417</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm font-semibold text-bone">Ink 900 (Surface 1)</span>
            <span class="text-2xs font-mono text-on-ink-muted uppercase group-hover:underline">Click to Copy</span>
          </div>
          <div class="mt-2 text-xs font-mono text-on-ink-muted">var(--ink-900) • Container Layer</div>
          <p class="mt-2 text-xs text-on-ink-muted">Elevated terminal windows, bento card surfaces, code blocks.</p>
        </div>

        <!-- Bone -->
        <div class="color-card p-4 rounded-lg bg-ink-900 border border-border-soft hover:border-bone transition-colors group cursor-pointer" data-copy="#f3f4f6">
          <div class="h-20 w-full rounded mb-3 flex items-end p-2.5 transition-transform group-hover:scale-[1.01]" style="background-color: #f3f4f6;">
            <span class="text-xs font-mono font-bold text-[#0a0b0d]">#f3f4f6</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm font-semibold text-bone">Bone (Primary Neutral)</span>
            <span class="text-2xs font-mono text-bone uppercase group-hover:underline">Click to Copy</span>
          </div>
          <div class="mt-2 text-xs font-mono text-on-ink-muted">var(--bone) • 18.2:1 AAA</div>
          <p class="mt-2 text-xs text-on-ink-muted">Primary display headlines, active navigation text, metric values.</p>
        </div>

        <!-- Terminal Amber -->
        <div class="color-card p-4 rounded-lg bg-ink-900 border border-border-soft hover:border-amber-400 transition-colors group cursor-pointer" data-copy="#f59e0b">
          <div class="h-20 w-full rounded mb-3 flex items-end p-2.5 transition-transform group-hover:scale-[1.01]" style="background-color: #f59e0b;">
            <span class="text-xs font-mono font-bold text-[#0a0b0d]">#f59e0b</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-sm font-semibold text-bone">Terminal Amber (Alert)</span>
            <span class="text-2xs font-mono text-amber-400 uppercase group-hover:underline">Click to Copy</span>
          </div>
          <div class="mt-2 text-xs font-mono text-on-ink-muted">var(--ember) • 8.9:1 AAA</div>
          <p class="mt-2 text-xs text-on-ink-muted">Schema migration warnings, DAG lock states, verification pending.</p>
        </div>
      </div>
    </div>
  </section>

  <!-- 5. Voice & Anti-Slop Manifesto -->
  <section id="voice" class="py-16 lg:py-20 border-b border-border-soft">
    <div class="container-marketing max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="mb-8">
        <h2 class="text-display-md lg:text-display-lg font-bold text-bone tracking-tight">5. Voice & Anti-Slop Directives</h2>
        <p class="mt-2 text-base text-on-ink-muted">Engineering precision over marketing embellishments.</p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
        <div class="p-6 rounded-lg bg-ink-900 border border-red-900/40">
          <div class="text-xs font-mono text-red-400 font-semibold mb-3">✗ MARKETING SLOP (REJECTED)</div>
          <p class="text-sm text-prose italic mb-4">
            "Next-Gen AI Supercharged Hyper-Platform empowering seamless synergies across the cloud ecosystem with magical one-click intelligence."
          </p>
          <div class="text-xs text-on-ink-muted leading-relaxed">
            Why this fails: Meaningless buzzwords, vague adjectives, zero architectural truth, no indication of how it works under the hood.
          </div>
        </div>

        <div class="p-6 rounded-lg bg-ink-900 border border-emerald-900/40">
          <div class="text-xs font-mono text-emerald-400 font-semibold mb-3">✓ SMOOOTH STANDARD (APPROVED)</div>
          <p class="text-sm text-bone font-medium mb-4">
            "Autonomous deployment runtime that allows Claude Code and Cursor to provision stateful Docker containers and PostgreSQL over MCP on your own Linux VPS."
          </p>
          <div class="text-xs text-on-ink-muted leading-relaxed">
            Why this works: Names the specific client agents, declares the protocol (MCP), and states the exact concrete infrastructure outcome.
          </div>
        </div>
      </div>

      <div class="p-6 rounded-lg bg-ink-900 border border-border-soft">
        <h3 class="text-title-lg font-semibold text-bone mb-3">Three Cardinal Rules of Smoooth Communication</h3>
        <ol class="space-y-3 text-sm text-on-ink-muted list-decimal list-inside">
          <li><strong class="text-bone">Proof over decoration:</strong> Accompany claims with actual bash commands, JSON payloads, or latency benchmarks.</li>
          <li><strong class="text-bone">No decorative eyebrow pills:</strong> Start every heading cleanly. Never precede an H1 or H2 with a colored rounded pill tag.</li>
          <li><strong class="text-bone">Zero fake social proof:</strong> If a metric is not verified in production telemetry, it does not appear on the website.</li>
        </ol>
      </div>
    </div>
  </section>

  <!-- 6. Press Boilerplate & Fast Facts -->
  <section id="boilerplate" class="py-16 lg:py-20 border-b border-border-soft bg-ink-950">
    <div class="container-marketing max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="mb-8">
        <h2 class="text-display-md lg:text-display-lg font-bold text-bone tracking-tight">6. Press Boilerplate & Fast Facts</h2>
        <p class="mt-2 text-base text-on-ink-muted">Pre-approved copy for journalists, newsletters, event organizers, and podcasts.</p>
      </div>

      <!-- Boilerplates with 1-click Copy -->
      <div class="space-y-6 mb-12">
        <!-- 1-Sentence -->
        <div class="p-5 rounded-lg bg-ink-900 border border-border-soft">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-mono text-spark">ONE-SENTENCE PITCH (21 words)</span>
            <button type="button" class="copy-bp-btn text-2xs font-mono text-on-ink-muted hover:text-spark uppercase" data-target="bp-1">Copy</button>
          </div>
          <p id="bp-1" class="text-sm text-bone leading-relaxed">
            Smoooth is an autonomous deployment runtime that allows AI coding agents to provision and manage stateful apps on self-hosted Linux VPS.
          </p>
        </div>

        <!-- 50-Word -->
        <div class="p-5 rounded-lg bg-ink-900 border border-border-soft">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-mono text-spark">50-WORD BIO (48 words)</span>
            <button type="button" class="copy-bp-btn text-2xs font-mono text-on-ink-muted hover:text-spark uppercase" data-target="bp-2">Copy</button>
          </div>
          <p id="bp-2" class="text-sm text-bone leading-relaxed">
            Smoooth gives AI coding agents like Claude Code, Cursor, and Windsurf direct infrastructure power. Rather than deploying ephemeral apps to high-markup PaaS sandboxes, Smoooth connects over the Model Context Protocol (MCP) to orchestrate persistent Docker containers, automated PostgreSQL databases, and self-healing rollbacks on the developer's own Linux VPS.
          </p>
        </div>

        <!-- 100-Word -->
        <div class="p-5 rounded-lg bg-ink-900 border border-border-soft">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-mono text-spark">100-WORD COMPREHENSIVE (96 words)</span>
            <button type="button" class="copy-bp-btn text-2xs font-mono text-on-ink-muted hover:text-spark uppercase" data-target="bp-3">Copy</button>
          </div>
          <p id="bp-3" class="text-sm text-bone leading-relaxed">
            As AI coding agents rapidly write production software, deployment remains bottlenecked by manual devops or restricted to proprietary serverless platforms. Smoooth eliminates this divide by acting as a native Model Context Protocol (MCP) deployment daemon for Linux servers. Developers simply run their preferred coding agent. Smoooth analyzes application architecture, generates a verified DAG deployment graph, provisions dedicated Docker containers with persistent storage and PostgreSQL, configures reverse-proxy ingress with automated SSL, and executes atomic zero-downtime health rollbacks. With Smoooth, engineers retain 100% data sovereignty and cloud cost savings without writing Terraform.
          </p>
        </div>
      </div>

      <!-- Fast Facts Table -->
      <div class="overflow-x-auto rounded-lg border border-border-soft bg-ink-900">
        <table class="w-full text-left text-xs">
          <thead class="border-b border-border-soft bg-ink-950 text-on-ink-muted font-mono uppercase">
            <tr>
              <th class="py-3 px-4">Entity</th>
              <th class="py-3 px-4">Canonical Specification</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-border-soft text-prose">
            <tr>
              <td class="py-3 px-4 font-semibold text-bone">Product Name</td>
              <td class="py-3 px-4">Smoooth (spelled with three 'o's)</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-semibold text-bone">Category</td>
              <td class="py-3 px-4">Developer Tools • Cloud Infrastructure • AI Agent Runtimes</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-semibold text-bone">Target OS</td>
              <td class="py-3 px-4">Linux (Ubuntu 22.04 / 24.04 LTS, Debian 12) with Docker 24+</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-semibold text-bone">Orchestration Protocol</td>
              <td class="py-3 px-4">Model Context Protocol (Anthropic MCP open standard)</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-semibold text-bone">Compatible Clients</td>
              <td class="py-3 px-4">Claude Code, Cursor, Windsurf, Roo Code, Goose, Cline</td>
            </tr>
            <tr>
              <td class="py-3 px-4 font-semibold text-bone">Primary Website</td>
              <td class="py-3 px-4 font-mono text-spark">https://smoooth.dev</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </section>

  <!-- 7. Media Kit Downloads & Press Kit Bundle -->
  <section id="downloads" class="py-16 lg:py-24">
    <div class="container-marketing max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="mb-8">
        <h2 class="text-display-md lg:text-display-lg font-bold text-bone tracking-tight">7. Download Media Kit Assets</h2>
        <p class="mt-2 text-base text-on-ink-muted">Vector assets, machine-readable design tokens, and markdown badge collections.</p>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 mb-12">
        <!-- Asset 1: Vector Logo Spec -->
        <a href="/assets/brand/logo-spec-dark.svg" download class="p-6 rounded-lg bg-ink-900 border border-border-soft hover:border-spark transition-colors group flex flex-col justify-between">
          <div>
            <div class="text-xs font-mono text-spark mb-2">VECTOR ASSET • SVG</div>
            <h3 class="text-title font-semibold text-bone mb-2">Logo Construction Spec</h3>
            <p class="text-xs text-on-ink-muted leading-relaxed">
              Full vector drawing with clear space safe boundaries, alignment grid, and Lexend lockup.
            </p>
          </div>
          <div class="mt-6 flex items-center gap-2 text-xs font-mono text-spark group-hover:underline">
            <span>Download SVG</span> →
          </div>
        </a>

        <!-- Asset 2: Design Tokens JSON -->
        <a href="/assets/brand/tokens.json" download class="p-6 rounded-lg bg-ink-900 border border-border-soft hover:border-spark transition-colors group flex flex-col justify-between">
          <div>
            <div class="text-xs font-mono text-fern-600 mb-2">SYSTEM TOKENS • JSON</div>
            <h3 class="text-title font-semibold text-bone mb-2">tokens.json</h3>
            <p class="text-xs text-on-ink-muted leading-relaxed">
              Standard design tokens schema containing all HEX, RGB, Lexend type scales, and radii.
            </p>
          </div>
          <div class="mt-6 flex items-center gap-2 text-xs font-mono text-fern-600 group-hover:underline">
            <span>Download JSON</span> →
          </div>
        </a>

        <!-- Asset 3: Tokens CSS -->
        <a href="/assets/brand/tokens.css" download class="p-6 rounded-lg bg-ink-900 border border-border-soft hover:border-spark transition-colors group flex flex-col justify-between">
          <div>
            <div class="text-xs font-mono text-glacier mb-2">STYLESHEET • CSS</div>
            <h3 class="text-title font-semibold text-bone mb-2">tokens.css</h3>
            <p class="text-xs text-on-ink-muted leading-relaxed">
              Drop-in CSS variables for third-party developer tools, documentation themes, and plugins.
            </p>
          </div>
          <div class="mt-6 flex items-center gap-2 text-xs font-mono text-glacier group-hover:underline">
            <span>Download CSS</span> →
          </div>
        </a>

        <!-- Asset 4: GitHub Badges -->
        <a href="/assets/brand/smoooth-badges.md" download class="p-6 rounded-lg bg-ink-900 border border-border-soft hover:border-spark transition-colors group flex flex-col justify-between">
          <div>
            <div class="text-xs font-mono text-amber-400 mb-2">README BADGES • MD</div>
            <h3 class="text-title font-semibold text-bone mb-2">Project Shields Badges</h3>
            <p class="text-xs text-on-ink-muted leading-relaxed">
              Markdown badges for repositories using Smoooth: Deployed with Smoooth, MCP Native, Linux VPS.
            </p>
          </div>
          <div class="mt-6 flex items-center gap-2 text-xs font-mono text-amber-400 group-hover:underline">
            <span>Download MD</span> →
          </div>
        </a>

        <!-- Asset 5: Full Brand Guideline MD -->
        <a href="/BRAND_GUIDELINES.md" download class="p-6 rounded-lg bg-ink-900 border border-border-soft hover:border-spark transition-colors group flex flex-col justify-between">
          <div>
            <div class="text-xs font-mono text-bone mb-2">DOCUMENTATION • MD</div>
            <h3 class="text-title font-semibold text-bone mb-2">BRAND_GUIDELINES.md</h3>
            <p class="text-xs text-on-ink-muted leading-relaxed">
              Complete markdown manual with typography tables, contrast standards, and editorial policies.
            </p>
          </div>
          <div class="mt-6 flex items-center gap-2 text-xs font-mono text-bone group-hover:underline">
            <span>Download MD</span> →
          </div>
        </a>

        <!-- Press Contact Inquiries -->
        <div class="p-6 rounded-lg bg-ink-950 border border-border-strong flex flex-col justify-between">
          <div>
            <div class="text-xs font-mono text-spark mb-2">PRESS & INQUIRIES</div>
            <h3 class="text-title font-semibold text-bone mb-2">Media Relations</h3>
            <p class="text-xs text-on-ink-muted leading-relaxed">
              For interviews, technical briefings, or custom asset requests, reach out directly.
            </p>
          </div>
          <div class="mt-6">
            <a href="mailto:press@smoooth.dev" class="text-xs font-mono text-spark hover:underline">press@smoooth.dev</a>
          </div>
        </div>
      </div>
    </div>
  </section>
</main>
`;

// 5. Build brand page interactive script
const brandScript = `
<script id="smoooth-brand-runtime">
document.addEventListener('DOMContentLoaded', () => {
  // Specimen weight toggles
  const wtBtns = document.querySelectorAll('.specimen-wt-btn');
  const preview2xl = document.getElementById('preview-2xl');
  const previewMd = document.getElementById('preview-md');
  const previewLg = document.getElementById('preview-lg');
  const specimenInput = document.getElementById('specimen-input');

  wtBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      wtBtns.forEach(b => {
        b.classList.remove('active', 'bg-spark', 'text-spark-ink', 'border-spark');
        b.classList.add('border-border-soft', 'text-prose');
      });
      btn.classList.add('active', 'bg-spark', 'text-spark-ink', 'border-spark');
      btn.classList.remove('border-border-soft', 'text-prose');

      const wt = btn.getAttribute('data-wt');
      if (preview2xl) preview2xl.style.fontWeight = wt;
      if (previewMd) previewMd.style.fontWeight = wt;
      if (previewLg) previewLg.style.fontWeight = wt;
    });
  });

  if (specimenInput) {
    specimenInput.addEventListener('input', (e) => {
      const val = e.target.value || 'Infrastructure freedom for AI coding agents.';
      if (preview2xl) preview2xl.textContent = val;
      if (previewMd) previewMd.textContent = val;
      if (previewLg) previewLg.textContent = val;
    });
  }

  // Color Swatch 1-click Hex copy
  const toast = document.getElementById('copy-toast');
  const colorCards = document.querySelectorAll('.color-card');

  function showToast(msg) {
    if (!toast) return;
    toast.textContent = msg;
    toast.classList.remove('translate-y-12', 'opacity-0');
    toast.classList.add('translate-y-0', 'opacity-100');
    setTimeout(() => {
      toast.classList.remove('translate-y-0', 'opacity-100');
      toast.classList.add('translate-y-12', 'opacity-0');
    }, 2000);
  }

  colorCards.forEach(card => {
    card.addEventListener('click', () => {
      const hex = card.getAttribute('data-copy');
      if (hex) {
        navigator.clipboard.writeText(hex).then(() => {
          showToast('Copied ' + hex + ' to clipboard!');
        }).catch(() => {
          showToast('Copied: ' + hex);
        });
      }
    });
  });

  // Boilerplate copy buttons
  const bpBtns = document.querySelectorAll('.copy-bp-btn');
  bpBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const targetId = btn.getAttribute('data-target');
      const el = document.getElementById(targetId);
      if (el) {
        const text = el.textContent.trim();
        navigator.clipboard.writeText(text).then(() => {
          const orig = btn.textContent;
          btn.textContent = 'COPIED!';
          btn.classList.add('text-spark');
          showToast('Boilerplate copied to clipboard!');
          setTimeout(() => {
            btn.textContent = orig;
            btn.classList.remove('text-spark');
          }, 2000);
        });
      }
    });
  });
});
</script>
`;

// 6. Assemble complete HTML
const brandHtml = `<!doctype html>
<html lang="en">
<head>
${headContent}
</head>
${beforeMain}
${mainContent}
${afterMain}
${brandScript}
`;

// 7. Write to brand.html, brand/index.html, mediakit.html, mediakit/index.html
fs.writeFileSync(path.join(rootDir, 'brand.html'), brandHtml, 'utf8');

const brandDir = path.join(rootDir, 'brand');
if (!fs.existsSync(brandDir)) fs.mkdirSync(brandDir, { recursive: true });
fs.writeFileSync(path.join(brandDir, 'index.html'), brandHtml, 'utf8');

fs.writeFileSync(path.join(rootDir, 'mediakit.html'), brandHtml, 'utf8');

const mediaKitDir = path.join(rootDir, 'mediakit');
if (!fs.existsSync(mediaKitDir)) fs.mkdirSync(mediaKitDir, { recursive: true });
fs.writeFileSync(path.join(mediaKitDir, 'index.html'), brandHtml, 'utf8');

console.log('Successfully generated:');
console.log(' - brand.html');
console.log(' - brand/index.html');
console.log(' - mediakit.html');
console.log(' - mediakit/index.html');
