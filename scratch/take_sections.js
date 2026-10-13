const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const edgePath = 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe';
const htmlUrl = 'file:///' + path.resolve(__dirname, '../index.html').replace(/\\/g, '/');
const outputPath = path.resolve(__dirname, '../section_view.png');

async function capture() {
  const port = 9335;
  const edge = spawn(edgePath, [
    `--remote-debugging-port=${port}`,
    '--headless=new',
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    '--user-data-dir=' + path.resolve(__dirname, 'edge-profile-sec'),
    '--window-size=1440,1600',
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

  await new Promise(r => setTimeout(r, 1200));

  // Scroll down 700px to see features
  await sendPage('Runtime.evaluate', { expression: 'window.scrollTo(0, 750)' });
  await new Promise(r => setTimeout(r, 600));

  const result = await sendPage('Page.captureScreenshot', { format: 'png' });
  if (result && result.data) {
    fs.writeFileSync(outputPath, Buffer.from(result.data, 'base64'));
    console.log('Saved to', outputPath);
  } else {
    console.error('Failed to capture', result);
  }

  ws.close();
  edge.kill();
  process.exit(0);
}

capture().catch(err => {
  console.error(err);
  process.exit(1);
});
