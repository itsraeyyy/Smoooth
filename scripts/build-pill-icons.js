const fs = require('fs');
const path = require('path');

const outDir = path.join(__dirname, '../assets/logos');

// 1. icon-claude.svg
const claudeD = fs.readFileSync('scratch/icons/claude.svg', 'utf8').match(/d="([^"]*)"/)[1];
const claudeSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none">
  <path fill="#CC5500" fill-rule="evenodd" d="${claudeD}"/>
</svg>`;
fs.writeFileSync(path.join(outDir, 'icon-claude.svg'), claudeSvg, 'utf8');

// 2. icon-cursor.svg
const cursorD = fs.readFileSync('scratch/icons/cursor.svg', 'utf8').match(/d="([^"]*)"/)[1];
const cursorSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none">
  <path fill="#161513" fill-rule="evenodd" d="${cursorD}"/>
</svg>`;
fs.writeFileSync(path.join(outDir, 'icon-cursor.svg'), cursorSvg, 'utf8');

// 3. icon-chatgpt.svg
const openaiD = fs.readFileSync('scratch/icons/openai.svg', 'utf8').match(/d="([^"]*)"/)[1];
const chatgptSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none">
  <path fill="#10A37F" fill-rule="evenodd" d="${openaiD}"/>
</svg>`;
fs.writeFileSync(path.join(outDir, 'icon-chatgpt.svg'), chatgptSvg, 'utf8');

// 4. icon-mcp.svg
const mcpPaths = fs.readFileSync('scratch/icons/mcp.svg', 'utf8').match(/d="([^"]*)"/g).map(d => d.replace(/^d="/, '').replace(/"$/, ''));
const mcpSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none">
  ${mcpPaths.map(d => `<path fill="#161513" fill-rule="evenodd" d="${d}"/>`).join('\n  ')}
</svg>`;
fs.writeFileSync(path.join(outDir, 'icon-mcp.svg'), mcpSvg, 'utf8');

// 5. icon-docker.svg
const dockerD = fs.readFileSync('scratch/icons/docker.svg', 'utf8').match(/d="([^"]*)"/)[1];
const dockerSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none">
  <path fill="#2496ED" d="${dockerD}"/>
</svg>`;
fs.writeFileSync(path.join(outDir, 'icon-docker.svg'), dockerSvg, 'utf8');

// 6. icon-postgres.svg
const postgresD = fs.readFileSync('scratch/icons/postgresql.svg', 'utf8').match(/d="([^"]*)"/)[1];
const postgresSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none">
  <path fill="#336791" d="${postgresD}"/>
</svg>`;
fs.writeFileSync(path.join(outDir, 'icon-postgres.svg'), postgresSvg, 'utf8');

console.log('All icon SVGs saved in assets/logos/');
