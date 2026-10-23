const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const distHtml = fs.readFileSync(path.join(rootDir, 'dist/brand.html'), 'utf8');

const regex = /(?:src|href)="([^"]+)"/g;
const paths = new Set();
let match;
while ((match = regex.exec(distHtml)) !== null) {
  const url = match[1];
  if (url.startsWith('/') && !url.startsWith('//') && !url.startsWith('/cdn-cgi')) {
    paths.add(url);
  }
}

console.log('Testing all root-relative URLs in dist/brand.html:');
let missing = 0;
for (const p of paths) {
  const full = path.join(rootDir, 'dist', p);
  // Also check if directory index
  const isDirect = fs.existsSync(full);
  const isIndex = fs.existsSync(path.join(full, 'index.html'));
  const isHtml = fs.existsSync(full + '.html');
  const exists = isDirect || isIndex || isHtml;
  console.log(` ${exists ? '✓' : '✗'} ${p} -> ${exists ? 'OK' : 'MISSING'}`);
  if (!exists) missing++;
}

if (missing === 0) {
  console.log('\n✓ All 100% of internal links and assets exist in dist!');
} else {
  console.log(`\nFound ${missing} missing references.`);
}
