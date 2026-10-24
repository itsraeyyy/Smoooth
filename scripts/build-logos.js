const fs = require('fs');
const path = require('path');

const logosDir = path.join(__dirname, '../assets/logos');
fs.mkdirSync(logosDir, { recursive: true });

// 1. Claude
const claudeSparkD = fs.readFileSync('scratch/icons/claude.svg', 'utf8').match(/d="([^"]*)"/)[1];
const claudeTextD = fs.readFileSync('scratch/icons/claude-text.svg', 'utf8').match(/d="([^"]*)"/)[1];

const claudeSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 130 24" fill="none">
  <g transform="translate(0, 0)">
    <path fill="#FFFFFF" fill-rule="evenodd" d="${claudeSparkD}"/>
  </g>
  <g transform="translate(32, 0)">
    <path fill="#FFFFFF" fill-rule="nonzero" d="${claudeTextD}"/>
  </g>
</svg>`;
fs.writeFileSync(path.join(logosDir, 'claude.svg'), claudeSvg, 'utf8');

// 2. ChatGPT
const chatgptBrand = fs.readFileSync('scratch/icons/chatgpt-brand.svg', 'utf8');
// Clean up chatgptBrand to viewBox 50 405 918 190 and fill #FFFFFF
const chatgptPaths = chatgptBrand.match(/<path[^>]*d="([^"]*)"[^>]*>/g).map(p => p.match(/d="([^"]*)"/)[1]);
const chatgptSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="50 405 918 190" fill="none">
  <g fill="#FFFFFF">
    ${chatgptPaths.map(d => `<path d="${d}" />`).join('\n    ')}
  </g>
</svg>`;
fs.writeFileSync(path.join(logosDir, 'chatgpt.svg'), chatgptSvg, 'utf8');

// 3. Cursor
const cursorCubeD = fs.readFileSync('scratch/icons/cursor.svg', 'utf8').match(/d="([^"]*)"/)[1];
const cursorTextD = fs.readFileSync('scratch/icons/cursor-text.svg', 'utf8').match(/d="([^"]*)"/)[1];

const cursorSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 156 24" fill="none">
  <g transform="translate(0, 0)">
    <path fill="#FFFFFF" fill-rule="evenodd" d="${cursorCubeD}"/>
  </g>
  <g transform="translate(32, 0)">
    <path fill="#FFFFFF" fill-rule="evenodd" d="${cursorTextD}"/>
  </g>
</svg>`;
fs.writeFileSync(path.join(logosDir, 'cursor.svg'), cursorSvg, 'utf8');

// 4. Docker
const dockerPathD = fs.readFileSync('scratch/icons/docker-gilbarbara.svg', 'utf8').match(/d="([^"]*)"/)[1];
const dockerSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 131" fill="none">
  <path fill="#FFFFFF" d="${dockerPathD}"/>
</svg>`;
fs.writeFileSync(path.join(logosDir, 'docker.svg'), dockerSvg, 'utf8');

// 5. MCP
const mcpSymbolPaths = fs.readFileSync('scratch/icons/mcp.svg', 'utf8').match(/d="([^"]*)"/g).map(d => d.replace(/^d="/, '').replace(/"$/, ''));
const mcpTextContent = fs.readFileSync('scratch/icons/mcpso-text.svg', 'utf8');
const mcpSubpaths = mcpTextContent.match(/d="([^"]*)"/)[1].split(/[zZ]/).map(s => s.trim()).filter(Boolean);
const mcpLetters = mcpSubpaths.slice(0, 3).join('z ') + 'z';

const mcpSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 94 24" fill="none">
  <g transform="translate(0, 0)">
    ${mcpSymbolPaths.map(d => `<path fill="#FFFFFF" fill-rule="evenodd" d="${d}"/>`).join('\n    ')}
  </g>
  <g transform="translate(32, 0)">
    <path fill="#FFFFFF" fill-rule="nonzero" d="${mcpLetters}"/>
  </g>
</svg>`;
fs.writeFileSync(path.join(logosDir, 'mcp.svg'), mcpSvg, 'utf8');

console.log('All 5 logo SVGs successfully generated in assets/logos/');
