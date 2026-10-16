const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const distDir = path.resolve(rootDir, 'dist');

console.log('Building Smoooth landing page for Cloudflare Pages...');

// Ensure dist directory exists
if (fs.existsSync(distDir)) {
  fs.rmSync(distDir, { recursive: true, force: true });
}
fs.mkdirSync(distDir, { recursive: true });

// Files to copy directly to dist
const filesToCopy = [
  'index.html',
  'styles.css',
  'script.js',
  '_headers',
  'desktop.png',
  'mobile.png',
  'section_view.png',
  'hero_visual_2x.png',
  'code_ref.png',
  'phone_ref.png'
];

for (const file of filesToCopy) {
  const src = path.join(rootDir, file);
  const dest = path.join(distDir, file);
  if (fs.existsSync(src)) {
    fs.copyFileSync(src, dest);
    console.log(`✓ Copied: ${file}`);
  }
}

console.log('Build complete! Output directory: dist/');
