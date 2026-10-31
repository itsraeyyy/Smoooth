const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const distDir = path.join(rootDir, 'dist');

console.log('Building Ferndesk 1:1 replica for static export / Cloudflare Pages...');

// Ensure dist directory exists
if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

// Helper to copy directory recursively
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

// 1. Copy SvelteKit immutable assets (_app) and brand assets
console.log('Copying _app assets and brand assets...');
copyDirRecursive(path.join(rootDir, '_app'), path.join(distDir, '_app'));
copyDirRecursive(path.join(rootDir, 'assets'), path.join(distDir, 'assets'));

// 2. Read interaction runtime script
const interactionScript = fs.readFileSync(path.join(rootDir, 'scripts/ferndesk-interaction.js'), 'utf8');

const lexendHeadInjection = `
<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link href="https://fonts.googleapis.com/css2?family=Lexend:wght@300;400;500;600;700;800&display=swap" rel="stylesheet" />
<style id="lexend-font-override">
  :root {
    --font-display: 'Lexend', ui-sans-serif, system-ui, sans-serif !important;
    --font-sans: 'Lexend', ui-sans-serif, system-ui, sans-serif !important;
    --font-text: 'Lexend', ui-sans-serif, system-ui, sans-serif !important;
    --default-font-family: 'Lexend', ui-sans-serif, system-ui, sans-serif !important;
    --font-display-stretch: 100% !important;
  }
  *, *::before, *::after {
    --font-display-stretch: 100% !important;
  }
  html, body, button, input, select, textarea, h1, h2, h3, h4, h5, h6, .font-display, .font-sans, .kicker, .eyebrow, p, a, li, dt, dd, label, strong, b, em {
    font-family: 'Lexend', ui-sans-serif, system-ui, -apple-system, sans-serif !important;
    font-stretch: 100% !important;
  }
  code, pre, kbd, samp, .font-mono, [class*="font-mono"], code *, pre *, .font-mono * {
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace !important;
    font-stretch: 100% !important;
  }
  
.sound-familiar-grid {
  display: grid !important;
  align-items: flex-start !important;
  gap: 2rem !important;
}
@media (min-width: 1024px) {
  .sound-familiar-grid {
    gap: 3rem !important;
  }
}
@media (min-width: 1280px) {
  .sound-familiar-grid {
    grid-template-columns: minmax(0, 1fr) 400px !important;
  }
}

  .sound-familiar-heading {
    font-size: clamp(36px, 4.5vw, 56px) !important;
    line-height: 1.05 !important;
    font-weight: 600 !important;
    letter-spacing: -0.035em !important;
    color: #ffffff !important;
  }
  .sound-familiar-item {
    font-size: 21px !important;
    line-height: 1.2 !important;
    font-weight: 600 !important;
    letter-spacing: -0.02em !important;
    white-space: nowrap !important;
  }
</style>`;

function processHtml(html) {
  let updated = html;

  // Remove unused Archivo & Inter font preloads to save bandwidth and prevent FOUT
  updated = updated.replace(/<link rel="preload" href="\/_app\/immutable\/assets\/(?:archivo|inter)-latin[^>]+>/gi, '');

  // Normalize font-display-stretch in inline styles from 88% to 100%
  updated = updated.replace(/--font-display-stretch:88%/g, '--font-display-stretch:100%');

  // Replace default font families in inline styles
  updated = updated.replace(/--font-display:"Archivo Marketing",\s*ui-sans-serif,\s*system-ui,\s*sans-serif/g, '--font-display:\'Lexend\', ui-sans-serif, system-ui, sans-serif');
  updated = updated.replace(/--font-text:"Inter Marketing",\s*system-ui,\s*sans-serif/g, '--font-text:\'Lexend\', ui-sans-serif, system-ui, sans-serif');
  updated = updated.replace(/--font-text:"Inter",\s*system-ui,\s*sans-serif/g, '--font-text:\'Lexend\', ui-sans-serif, system-ui, sans-serif');

  // Inject Lexend Google Font links & override CSS before </head>
  if (updated.includes('id="lexend-font-override"')) {
    // update existing override block if present
    updated = updated.replace(/<style id="lexend-font-override">[\s\S]*?<\/style>/i, `<style id="lexend-font-override">\n  :root {\n    --font-display: 'Lexend', ui-sans-serif, system-ui, sans-serif !important;\n    --font-sans: 'Lexend', ui-sans-serif, system-ui, sans-serif !important;\n    --font-text: 'Lexend', ui-sans-serif, system-ui, sans-serif !important;\n    --default-font-family: 'Lexend', ui-sans-serif, system-ui, sans-serif !important;\n    --font-display-stretch: 100% !important;\n  }\n  *, *::before, *::after {\n    --font-display-stretch: 100% !important;\n  }\n  html, body, button, input, select, textarea, h1, h2, h3, h4, h5, h6, .font-display, .font-sans, .kicker, .eyebrow, p, a, li, dt, dd, label, strong, b, em {\n    font-family: 'Lexend', ui-sans-serif, system-ui, -apple-system, sans-serif !important;\n    font-stretch: 100% !important;\n  }\n  code, pre, kbd, samp, .font-mono, [class*="font-mono"], code *, pre *, .font-mono * {\n    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace !important;\n    font-stretch: 100% !important;\n  }\n</style>`);
  } else {
    updated = updated.replace('</head>', `${lexendHeadInjection}\n</head>`);
  }

  // Inject or update interactive runtime
  if (updated.includes('id="ferndesk-interactive-runtime"')) {
    updated = updated.replace(/<script id="ferndesk-interactive-runtime">[\s\S]*?<\/script>/i, `<script id="ferndesk-interactive-runtime">\n${interactionScript}\n</script>`);
  } else {
    updated = updated.replace('</body>', `<script id="ferndesk-interactive-runtime">\n${interactionScript}\n</script>\n</body>`);
  }

  return updated;
}

// 3. Copy root files: index.html, 404.html, robots.txt, sitemap.xml, favicons, headers
const rootStaticFiles = [
  'index.html',
  '404.html',
  'robots.txt',
  'sitemap.xml',
  'favicon.ico',
  'favicon.svg',
  'favicon-32.png',
  '_headers',
  'styles.css'
];

for (const file of rootStaticFiles) {
  const src = path.join(rootDir, file);
  if (fs.existsSync(src)) {
    if (file.endsWith('.html')) {
      const content = processHtml(fs.readFileSync(src, 'utf8'));
      fs.writeFileSync(path.join(distDir, file), content, 'utf8');
      fs.writeFileSync(src, content, 'utf8');
    } else {
      fs.copyFileSync(src, path.join(distDir, file));
    }
  }
}

// 4. Scan and copy all HTML pages from root to dist
console.log('Syncing all pages to dist with clean URL support and Lexend typography...');

const ignoredDirs = new Set(['node_modules', '.git', 'dist', 'target', '.cargo', '.cursor', '.agents', 'scripts', 'scratch', 'src', 'tests', 'release-dist', '_app', 'frontend', 'migrations']);

function syncPages(dir, relativeDir = '') {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isDirectory()) {
      if (ignoredDirs.has(entry.name)) continue;
      const subRel = path.join(relativeDir, entry.name);
      syncPages(path.join(dir, entry.name), subRel);
    } else if (entry.name.endsWith('.html')) {
      if (relativeDir === '' && (entry.name === 'index.html' || entry.name === '404.html')) {
        continue; // already handled
      }
      const srcFile = path.join(dir, entry.name);
      const destFile = path.join(distDir, relativeDir, entry.name);
      fs.mkdirSync(path.dirname(destFile), { recursive: true });

      const content = processHtml(fs.readFileSync(srcFile, 'utf8'));
      fs.writeFileSync(srcFile, content, 'utf8');
      fs.writeFileSync(destFile, content, 'utf8');

      // Support directory index if it's a slug.html
      const slug = path.basename(entry.name, '.html');
      if (slug !== 'index') {
        const slugDirPath = path.join(distDir, relativeDir, slug, 'index.html');
        fs.mkdirSync(path.dirname(slugDirPath), { recursive: true });
        fs.writeFileSync(slugDirPath, content, 'utf8');
      }
    }
  }
}

syncPages(rootDir);

// 5. Update CSS assets in _app and dist/_app
function updateCssFile(filePath) {
  if (!fs.existsSync(filePath)) return;
  let css = fs.readFileSync(filePath, 'utf8');
  css = css.replace(/--font-display-stretch:88%/g, '--font-display-stretch:100%');
  css = css.replace(/--font-display:"Archivo Marketing",\s*ui-sans-serif,\s*system-ui,\s*sans-serif/g, '--font-display:\'Lexend\', ui-sans-serif, system-ui, sans-serif');
  css = css.replace(/--font-text:"Inter Marketing",\s*system-ui,\s*sans-serif/g, '--font-text:\'Lexend\', ui-sans-serif, system-ui, sans-serif');
  css = css.replace(/--font-text:"Inter",\s*system-ui,\s*sans-serif/g, '--font-text:\'Lexend\', ui-sans-serif, system-ui, sans-serif');
  fs.writeFileSync(filePath, css, 'utf8');
}

updateCssFile(path.join(rootDir, 'styles.css'));
updateCssFile(path.join(distDir, 'styles.css'));
updateCssFile(path.join(rootDir, '_app/immutable/assets/0.C8-5y8i5.css'));
updateCssFile(path.join(distDir, '_app/immutable/assets/0.C8-5y8i5.css'));

// Count total pages in dist
function countHtmlPages(dir) {
  let count = 0;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      count += countHtmlPages(full);
    } else if (entry.name.endsWith('.html')) {
      count++;
    }
  }
  return count;
}

const totalDistHtml = countHtmlPages(distDir);
console.log(`✓ Ferndesk 1:1 build complete! Total HTML files in dist: ${totalDistHtml}`);
