const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const edgePath = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe';
const htmlUrl = 'http://127.0.0.1:3000';
const outputPath = path.resolve(__dirname, '../nextjs_screenshot.png');
const outputCapabilitiesPath = path.resolve(__dirname, '../nextjs_capabilities.png');

async function capture() {
  console.log('Pre-warming Next.js dev server...');
  try {
    const res = await fetch(htmlUrl);
    console.log('Pre-warm response:', res.status);
  } catch (e) {
    console.log('Pre-warm failed, continuing anyway...');
  }

  console.log('Launching Edge for Next.js screenshot...');
  const port = 9339;
  const edge = spawn(edgePath, [
    `--remote-debugging-port=${port}`,
    '--headless=new',
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    '--user-data-dir=' + path.resolve(__dirname, 'edge-profile-next-v2'),
    '--window-size=1440,1100',
    htmlUrl
  ]);

  let wsUrl = null;
  for (let i = 0; i < 30; i++) {
    await new Promise(r => setTimeout(r, 300));
    try {
      const res = await fetch(`http://127.0.0.1:${port}/json/version`);
      if (res.ok) {
        const data = await res.json();
        wsUrl = data.webSocketDebuggerUrl;
        break;
      }
    } catch (e) {}
  }

  if (!wsUrl) {
    console.error('Failed to connect to Edge DevTools');
    edge.kill();
    return;
  }

  const ws = new WebSocket(wsUrl);
  await new Promise((resolve) => ws.onopen = resolve);

  let id = 1;
  function send(method, params = {}) {
    return new Promise((resolve) => {
      const curId = id++;
      const handler = (event) => {
        const msg = JSON.parse(event.data);
        if (msg.id === curId) {
          ws.removeEventListener('message', handler);
          resolve(msg.result);
        }
      };
      ws.addEventListener('message', handler);
      ws.send(JSON.stringify({ id: curId, method, params }));
    });
  }

  const { targetInfos } = await send('Target.getTargets');
  const pageTarget = targetInfos.find(t => t.type === 'page');
  const { sessionId } = await send('Target.attachToTarget', { targetId: pageTarget.targetId, flatten: true });

  function sendPage(method, params = {}) {
    return new Promise((resolve) => {
      const curId = id++;
      const handler = (event) => {
        const msg = JSON.parse(event.data);
        if (msg.id === curId) {
          ws.removeEventListener('message', handler);
          resolve(msg.result);
        }
      };
      ws.addEventListener('message', handler);
      ws.send(JSON.stringify({ id: curId, sessionId, method, params }));
    });
  }

  // Wait 4s for page to settle, fonts to render
  await new Promise(r => setTimeout(r, 4000));

  console.log('Capturing hero screenshot of Next.js app...');
  const { data: heroData } = await sendPage('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(outputPath, Buffer.from(heroData, 'base64'));
  console.log('Hero screenshot saved to', outputPath);

  // Scroll to capabilities section
  await sendPage('Runtime.evaluate', { expression: 'document.getElementById("capabilities")?.scrollIntoView()' });
  await new Promise(r => setTimeout(r, 1000));

  console.log('Capturing capabilities screenshot...');
  const { data: capData } = await sendPage('Page.captureScreenshot', { format: 'png' });
  fs.writeFileSync(outputCapabilitiesPath, Buffer.from(capData, 'base64'));
  console.log('Capabilities screenshot saved to', outputCapabilitiesPath);

  ws.close();
  edge.kill();
  process.exit(0);
}

capture().catch(err => {
  console.error(err);
  process.exit(1);
});
