const fs = require('fs');
const path = require('path');
const https = require('https');

const rootDir = path.resolve(__dirname, '..');
const distDir = path.join(rootDir, 'dist');

function fetchPage(url, retries = 3) {
  return new Promise((resolve, reject) => {
    const req = https.get(url, {
      headers: {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
        'Accept-Language': 'en-US,en;q=0.9'
      }
    }, res => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        let redirectUrl = res.headers.location;
        if (redirectUrl.startsWith('/')) redirectUrl = 'https://ferndesk.com' + redirectUrl;
        return fetchPage(redirectUrl, retries - 1).then(resolve).catch(reject);
      }
      if (res.statusCode !== 200) {
        if (retries > 0) {
          setTimeout(() => fetchPage(url, retries - 1).then(resolve).catch(reject), 1000);
        } else {
          resolve({ status: res.statusCode, data: null, url });
        }
        return;
      }
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => resolve({ status: 200, data: Buffer.concat(chunks).toString('utf8'), url }));
    });
    req.on('error', err => {
      if (retries > 0) {
        setTimeout(() => fetchPage(url, retries - 1).then(resolve).catch(reject), 1000);
      } else {
        resolve({ status: 500, error: err.message, url });
      }
    });
    req.setTimeout(15000, () => {
      req.abort();
      if (retries > 0) {
        setTimeout(() => fetchPage(url, retries - 1).then(resolve).catch(reject), 1000);
      } else {
        resolve({ status: 408, error: 'timeout', url });
      }
    });
  });
}

function fetchBinary(url, outPath) {
  return new Promise((resolve) => {
    fs.mkdirSync(path.dirname(outPath), { recursive: true });
    if (fs.existsSync(outPath) && fs.statSync(outPath).size > 0) {
      return resolve(true);
    }
    const req = https.get(url, {
      headers: {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
      }
    }, res => {
      if (res.statusCode !== 200) return resolve(false);
      const chunks = [];
      res.on('data', c => chunks.push(c));
      res.on('end', () => {
        fs.writeFileSync(outPath, Buffer.concat(chunks));
        resolve(true);
      });
    });
    req.on('error', () => resolve(false));
    req.setTimeout(15000, () => { req.abort(); resolve(false); });
  });
}

async function scrapeAll() {
  console.log('Reading sitemap.xml...');
  const sitemapXml = fs.readFileSync(path.join(rootDir, 'sitemap.xml'), 'utf8');
  const urls = [...sitemapXml.matchAll(/<loc>([^<]+)<\/loc>/g)].map(m => m[1].trim());

  // Also include 404 page
  urls.push('https://ferndesk.com/404-not-found-page-test-ferndesk');

  console.log(`Found ${urls.length} URLs to scrape.`);

  const extraAssetsToDownload = new Set();
  const queue = [...urls];
  let successful = 0;
  let failed = 0;

  async function worker(id) {
    while (queue.length > 0) {
      const currentUrl = queue.shift();
      const u = new URL(currentUrl);
      const pathname = u.pathname; // e.g. /pricing or /compare/mintlify or /

      const res = await fetchPage(currentUrl);
      if (!res || !res.data) {
        console.error(`[Worker ${id}] FAILED: ${currentUrl} (status ${res ? res.status : 'unknown'})`);
        failed++;
        continue;
      }

      let html = res.data;

      // Collect all assets referenced in this HTML
      const assetMatches = [...html.matchAll(/(?:href|src)=["'](\.?\/_app\/immutable\/[^"']+)["']/g)].map(m => m[1]);
      for (const a of assetMatches) {
        extraAssetsToDownload.add(a.replace(/^\./, ''));
      }

      // Save HTML to disk
      if (pathname === '/' || pathname === '') {
        fs.writeFileSync(path.join(rootDir, 'index.html'), html, 'utf8');
        fs.writeFileSync(path.join(distDir, 'index.html'), html, 'utf8');
      } else if (currentUrl.includes('404-not-found-page-test-ferndesk')) {
        fs.writeFileSync(path.join(rootDir, '404.html'), html, 'utf8');
        fs.writeFileSync(path.join(distDir, '404.html'), html, 'utf8');
      } else {
        const cleanPath = pathname.replace(/^\//, '').replace(/\/$/, '');
        // e.g. "pricing" -> rootDir/pricing.html AND rootDir/pricing/index.html
        // "compare/mintlify" -> rootDir/compare/mintlify.html AND rootDir/compare/mintlify/index.html
        const rootHtmlPath = path.join(rootDir, cleanPath + '.html');
        const rootDirPath = path.join(rootDir, cleanPath, 'index.html');
        const distHtmlPath = path.join(distDir, cleanPath + '.html');
        const distDirPath = path.join(distDir, cleanPath, 'index.html');

        fs.mkdirSync(path.dirname(rootHtmlPath), { recursive: true });
        fs.mkdirSync(path.dirname(rootDirPath), { recursive: true });
        fs.mkdirSync(path.dirname(distHtmlPath), { recursive: true });
        fs.mkdirSync(path.dirname(distDirPath), { recursive: true });

        fs.writeFileSync(rootHtmlPath, html, 'utf8');
        fs.writeFileSync(rootDirPath, html, 'utf8');
        fs.writeFileSync(distHtmlPath, html, 'utf8');
        fs.writeFileSync(distDirPath, html, 'utf8');
      }

      successful++;
      if (successful % 20 === 0 || queue.length === 0) {
        console.log(`Progress: ${successful}/${urls.length} pages downloaded...`);
      }
    }
  }

  console.log('Starting parallel scraping with 8 workers...');
  const workers = Array(8).fill(0).map((_, i) => worker(i + 1));
  await Promise.all(workers);

  console.log(`Page scraping completed! Success: ${successful}, Failed: ${failed}`);

  // Now check extra assets found in HTMLs
  console.log(`Checking ${extraAssetsToDownload.size} discovered immutable assets...`);
  let downloadedAssets = 0;
  for (const assetPath of extraAssetsToDownload) {
    const localRootPath = path.join(rootDir, assetPath.replace(/^\//, ''));
    const localDistPath = path.join(distDir, assetPath.replace(/^\//, ''));
    if (!fs.existsSync(localRootPath)) {
      const remoteUrl = 'https://ferndesk.com' + (assetPath.startsWith('/') ? assetPath : '/' + assetPath);
      const ok = await fetchBinary(remoteUrl, localRootPath);
      if (ok) {
        fs.mkdirSync(path.dirname(localDistPath), { recursive: true });
        fs.copyFileSync(localRootPath, localDistPath);
        downloadedAssets++;
      }
    } else if (!fs.existsSync(localDistPath)) {
      fs.mkdirSync(path.dirname(localDistPath), { recursive: true });
      fs.copyFileSync(localRootPath, localDistPath);
    }
  }
  console.log(`Downloaded ${downloadedAssets} additional immutable assets.`);
}

scrapeAll().catch(console.error);
