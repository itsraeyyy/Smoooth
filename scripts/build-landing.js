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
console.log('Copying _app assets and brand directory...');
copyDirRecursive(path.join(rootDir, '_app'), path.join(distDir, '_app'));
if (fs.existsSync(path.join(rootDir, 'assets'))) {
  copyDirRecursive(path.join(rootDir, 'assets'), path.join(distDir, 'assets'));
}
if (fs.existsSync(path.join(rootDir, 'fonts'))) {
  copyDirRecursive(path.join(rootDir, 'fonts'), path.join(distDir, 'fonts'));
}

// 2. Read interaction runtime script
const interactionScript = fs.readFileSync(path.join(rootDir, 'scripts/ferndesk-interaction.js'), 'utf8');
const scriptTag = `\n<script>\n${interactionScript}\n</script>\n</body>`;

function injectScript(html) {
  if (html.includes('id="ferndesk-interactive-runtime"')) return html;
  return html.replace('</body>', `<script id="ferndesk-interactive-runtime">\n${interactionScript}\n</script>\n</body>`);
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
  'styles.css',
  'BRAND_GUIDELINES.md'
];

for (const file of rootStaticFiles) {
  const src = path.join(rootDir, file);
  if (fs.existsSync(src)) {
    if (file.endsWith('.html')) {
      const content = injectScript(fs.readFileSync(src, 'utf8'));
      fs.writeFileSync(path.join(distDir, file), content, 'utf8');
      fs.writeFileSync(src, content, 'utf8');
    } else {
      fs.copyFileSync(src, path.join(distDir, file));
    }
  }
}

// 4. Scan and copy all HTML pages from root to dist
console.log('Syncing all pages to dist with clean URL support...');

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

      const content = injectScript(fs.readFileSync(srcFile, 'utf8'));
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
