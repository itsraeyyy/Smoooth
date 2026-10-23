const fs = require('fs');

const html = fs.readFileSync('dist/index.html', 'utf8');

const title = html.match(/<title>(.*?)<\/title>/)?.[1] || '';
const desc = html.match(/<meta name="description" content="(.*?)"/)?.[1] || '';
const h1 = html.match(/<h1[^>]*>([\s\S]*?)<\/h1>/)?.[1]?.replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim() || '';
const hasKicker = html.includes('data-hero-preview-label');
const hasFaq = html.includes('id="faq"');
const hasRobots = fs.existsSync('dist/robots.txt');
const hasSitemap = fs.existsSync('dist/sitemap.xml');
const has404 = fs.existsSync('dist/404.html');

// Guide verification
const guideHtmlPath = 'dist/guides/deploy-docker-application-claude-code.html';
const hasGuide = fs.existsSync(guideHtmlPath);
let guideTitle = '', guideDesc = '', guideH1 = '', guideHasKicker = false, guideHasSchema = false, guideInSitemap = false;

if (hasGuide) {
  const guideHtml = fs.readFileSync(guideHtmlPath, 'utf8');
  guideTitle = guideHtml.match(/<title>(.*?)<\/title>/)?.[1] || '';
  guideDesc = guideHtml.match(/<meta name="description" content="(.*?)"/)?.[1] || '';
  guideH1 = guideHtml.match(/<h1[^>]*>([\s\S]*?)<\/h1>/)?.[1]?.replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim() || '';
  guideHasKicker = guideHtml.includes('rounded-full') && (guideHtml.includes('uppercase') || guideHtml.includes('tracking-'));
  guideHasSchema = guideHtml.includes('"@type": "TechArticle"');
}

const sitemapContent = hasSitemap ? fs.readFileSync('dist/sitemap.xml', 'utf8') : '';
guideInSitemap = sitemapContent.includes('https://smoooth.dev/guides/deploy-docker-application-claude-code');

console.log('--- HOMEPAGE VERIFICATION ---');
console.log(`Title (${title.length} chars): "${title}"`);
console.log(`Description (${desc.length} chars): "${desc}"`);
console.log(`H1: "${h1}"`);
console.log(`Has kicker pill badge (must be false): ${hasKicker}`);
console.log(`Has FAQ section: ${hasFaq}`);
console.log(`dist/robots.txt: ${hasRobots}`);
console.log(`dist/sitemap.xml: ${hasSitemap}`);
console.log(`dist/404.html: ${has404}`);

console.log('--- GUIDE VERIFICATION ---');
console.log(`dist/guides/... exists: ${hasGuide}`);
console.log(`Guide Title (${guideTitle.length} chars): "${guideTitle}"`);
console.log(`Guide Description (${guideDesc.length} chars): "${guideDesc}"`);
console.log(`Guide H1: "${guideH1}"`);
console.log(`Guide has kicker badge: ${guideHasKicker}`);
console.log(`Guide has TechArticle schema: ${guideHasSchema}`);
console.log(`Guide registered in sitemap: ${guideInSitemap}`);

if (title && desc && h1 && !hasKicker && hasFaq && hasRobots && hasSitemap && has404 &&
    hasGuide && guideTitle && guideDesc && guideH1 && !guideHasKicker && guideHasSchema && guideInSitemap) {
  console.log('✓ All verification criteria passed!');
} else {
  console.error('✗ Some checks failed.');
  process.exit(1);
}
