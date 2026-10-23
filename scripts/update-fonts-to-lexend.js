const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');

const lexendCssBlock = [
  "@font-face{font-family:'Lexend';font-style:normal;font-weight:100 900;font-display:swap;src:url(/_app/immutable/assets/lexend-latin-variable.woff2)format('woff2');unicode-range:U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD}",
  "@font-face{font-family:'Lexend';font-style:normal;font-weight:100 900;font-display:swap;src:url(/_app/immutable/assets/lexend-latin-ext-variable.woff2)format('woff2');unicode-range:U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C4,U+2113,U+2C60-2C7F,U+A720-A7FF}",
  "@font-face{font-family:'Archivo Marketing';font-style:normal;font-weight:100 900;font-display:swap;src:url(/_app/immutable/assets/lexend-latin-variable.woff2)format('woff2')}",
  "@font-face{font-family:'Inter Marketing';font-style:normal;font-weight:100 900;font-display:swap;src:url(/_app/immutable/assets/lexend-latin-variable.woff2)format('woff2')}",
  "@font-face{font-family:'Archivo Variable';font-style:normal;font-weight:100 900;font-display:swap;src:url(/_app/immutable/assets/lexend-latin-variable.woff2)format('woff2')}"
].join('');

function transformCss(css) {
  const firstIdx = css.indexOf('@font-face');
  const lastIdx = css.lastIndexOf('@font-face');
  const nextClose = css.indexOf('}', lastIdx);

  if (firstIdx !== -1 && nextClose !== -1) {
    css = css.slice(0, firstIdx) + lexendCssBlock + css.slice(nextClose + 1);
  }

  css = css.replace(/--font-display:\s*"Archivo Marketing"/g, '--font-display:"Lexend"');
  css = css.replace(/--font-display:\s*'Archivo Marketing'/g, '--font-display:"Lexend"');
  css = css.replace(/--font-text:\s*"Inter Marketing"/g, '--font-text:"Lexend"');
  css = css.replace(/--font-text:\s*'Inter Marketing'/g, '--font-text:"Lexend"');
  css = css.replace(/--font-text:\s*"Inter"/g, '--font-text:"Lexend"');
  css = css.replace(/--font-text:\s*'Inter'/g, '--font-text:"Lexend"');
  css = css.replace(/--font-display-stretch:\s*88%/g, '--font-display-stretch:100%');

  return css;
}

// 1. Update styles.css and _app/immutable/assets/0.C8-5y8i5.css
const cssFiles = [
  path.join(rootDir, 'styles.css'),
  path.join(rootDir, '_app', 'immutable', 'assets', '0.C8-5y8i5.css')
];

for (const f of cssFiles) {
  if (fs.existsSync(f)) {
    const orig = fs.readFileSync(f, 'utf8');
    const updated = transformCss(orig);
    fs.writeFileSync(f, updated, 'utf8');
    console.log(`Updated CSS: ${path.relative(rootDir, f)} (${orig.length} -> ${updated.length} bytes)`);
  }
}

// 2. Update HTML preloads and inline <style> across all HTML files
function transformHtml(html) {
  // Update preloads
  html = html.replace(
    /<link\s+rel="preload"\s+href="[^"]*archivo-latin[^"]*"\s+as="font"[^>]*>/gi,
    '<link rel="preload" href="/_app/immutable/assets/lexend-latin-variable.woff2" as="font" type="font/woff2" crossorigin="anonymous"/>'
  );
  html = html.replace(
    /<link\s+rel="preload"\s+href="[^"]*inter-latin[^"]*"\s+as="font"[^>]*>/gi,
    ''
  );

  // Update inline <style> if present
  html = html.replace(/<style([^>]*)>([\s\S]*?)<\/style>/gi, (match, attrs, content) => {
    return `<style${attrs}>${transformCss(content)}</style>`;
  });

  return html;
}

const ignoredDirs = new Set(['node_modules', '.git', 'dist', 'target', '.cargo', '.cursor', '.agents', 'scripts', 'scratch', 'src', 'tests', 'release-dist', '_app', 'frontend', 'migrations']);

let htmlCount = 0;
function walk(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (!ignoredDirs.has(entry.name)) {
        walk(fullPath);
      }
    } else if (entry.name.endsWith('.html')) {
      const orig = fs.readFileSync(fullPath, 'utf8');
      const updated = transformHtml(orig);
      if (orig !== updated) {
        fs.writeFileSync(fullPath, updated, 'utf8');
        htmlCount++;
      }
    }
  }
}

walk(rootDir);
console.log(`Updated ${htmlCount} HTML files to use Lexend font.`);
