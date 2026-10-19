//! Self-contained Smoooth Operator Control Dashboard
//! Embedded directly into the binary with zero external runtime dependencies.
//! Pixel-accurate implementation matching Overseer agent-operations design system.

pub fn get_dashboard_html() -> &'static str {
    DASHBOARD_HTML
}

const DASHBOARD_HTML: &str = r###"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com data:; connect-src 'self'; img-src 'self' data:;">
  <title>Overview · Smoooth Overseer</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Geist:wght@300;400;500;600;700&family=Geist+Mono:wght@300;400;500;600&family=Instrument+Serif:ital@0;1&family=Silkscreen&display=swap" rel="stylesheet">
  <style>
    /* ---------- Design System Tokens from Overseer ---------- */
    :root {
      --frame: #1f2421;
      --panel: #252a27;
      --card: #232825;
      --card-line: rgba(150, 150, 150, 0.2);
      --hairline: #575757;
      --field: #1a1f1c;

      --text: #ffffff;
      --text-bright: #f5f5f6;
      --text-muted: #a6a6ab;
      --text-dim: #7c7c82;
      --text-faint: #727378;
      --icon: #8a8a90;

      --green: #00a449;
      --green-bright: #4ade80;
      --green-trend: #00b952;
      --green-deep: #1f5537;
      --green-edge: #006624;
      --red: #f08a8a;
      --blue: #7fa8ff;
      --amber: #e0b36a;

      --sans: 'Geist', -apple-system, BlinkMacSystemFont, sans-serif;
      --mono: 'Geist Mono', monospace;
      --pixel: 'Silkscreen', monospace;
      --display: 'Instrument Serif', Georgia, serif;

      --ease-out: cubic-bezier(0.22, 1, 0.36, 1);
      --sidebar-w: 211px;
      color-scheme: dark;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--frame);
      color: var(--text);
      font-family: var(--sans);
      font-size: 12px;
      line-height: 1.4;
      -webkit-font-smoothing: antialiased;
      -moz-osx-font-smoothing: grayscale;
      text-rendering: geometricPrecision;
      overflow-x: hidden;
      min-height: 100vh;
    }

    /* ---------- Shell Layout ---------- */
    .shell {
      display: grid;
      grid-template-columns: var(--sidebar-w) minmax(0, 1fr);
      min-height: 100vh;
      transition: grid-template-columns 0.2s var(--ease-out);
    }
    .shell.is-collapsed {
      --sidebar-w: 56px;
    }

    /* ---------- Sidebar ---------- */
    .sidebar {
      position: sticky;
      top: 0;
      height: 100vh;
      display: flex;
      flex-direction: column;
      border-right: 0.4px solid var(--hairline);
      background: var(--frame);
      overflow: hidden auto;
      scrollbar-width: none;
      z-index: 30;
    }
    .sidebar::-webkit-scrollbar { display: none; }

    .sidebar__brand {
      flex: none;
      height: 51px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 14px;
      border-bottom: 0.4px solid var(--hairline);
    }
    .sidebar__home {
      display: flex;
      align-items: center;
      gap: 8px;
      text-decoration: none;
      color: var(--text-bright);
    }
    .sidebar__logo-glyph {
      width: 18px;
      height: 18px;
      fill: none;
      stroke: var(--green-bright);
      stroke-width: 1.8;
    }
    .sidebar__brand-title {
      font-family: var(--display);
      font-size: 19px;
      letter-spacing: -0.01em;
      color: var(--text);
    }
    .sidebar__collapse {
      background: transparent;
      border: none;
      cursor: pointer;
      color: var(--text-dim);
      padding: 4px;
      border-radius: 4px;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: color 0.15s ease;
    }
    .sidebar__collapse:hover {
      color: var(--text-bright);
    }

    .sidebar__nav {
      padding-top: 16px;
      flex: 1;
    }
    .nav-group {
      padding: 0 11px 10px;
    }
    .nav-group + .nav-group {
      border-top: 0.4px dashed var(--hairline);
      padding-top: 12px;
    }
    .nav-group__title {
      padding-left: 6px;
      margin-bottom: 7px;
      font-family: var(--mono);
      font-size: 8.5px;
      letter-spacing: 0.05em;
      text-transform: uppercase;
      color: var(--text-dim);
      white-space: nowrap;
    }
    .is-collapsed .nav-group__title {
      display: none;
    }
    .nav-group ul {
      list-style: none;
      display: flex;
      flex-direction: column;
      gap: 3px;
    }
    .nav-item {
      display: flex;
      align-items: center;
      gap: 8px;
      height: 27px;
      padding-left: 8px;
      padding-right: 8px;
      border-radius: 6px;
      color: var(--text-muted);
      font-size: 11.8px;
      letter-spacing: -0.02em;
      text-decoration: none;
      cursor: pointer;
      white-space: nowrap;
      transition: background-color 0.15s ease, color 0.15s ease;
      position: relative;
    }
    .nav-item:hover {
      background: rgba(42, 48, 44, 0.6);
      color: var(--text-bright);
    }
    .nav-item.is-active {
      background: #2a302c;
      color: var(--text-bright);
      font-weight: 500;
      box-shadow:
        0 2px 4px rgba(0, 0, 0, 0.25),
        inset 0 -0.5px 1px rgba(0, 116, 51, 0.35),
        inset 0 0.25px 0.5px rgba(255, 255, 255, 0.4);
    }
    .nav-item__icon {
      flex: none;
      width: 14px;
      height: 14px;
      color: var(--icon);
      stroke: currentColor;
      fill: none;
    }
    .nav-item.is-active .nav-item__icon {
      color: var(--green-bright);
    }
    .is-collapsed .nav-item__label {
      display: none;
    }

    /* Sidebar Footer Token Budget Card */
    .sidebar__footer {
      margin-top: auto;
      padding: 12px 11px;
      border-top: 0.4px dashed var(--hairline);
    }
    .is-collapsed .sidebar__footer {
      display: none;
    }
    .token-card {
      background: #181d1a;
      border: 0.4px solid var(--hairline);
      border-radius: 8px;
      padding: 10px;
    }
    .token-card__title {
      font-family: var(--mono);
      font-size: 9px;
      letter-spacing: 0.04em;
      text-transform: uppercase;
      color: var(--text-muted);
      margin-bottom: 6px;
    }
    .token-card__meta {
      display: flex;
      justify-content: space-between;
      margin-top: 6px;
      font-family: var(--mono);
      font-size: 10px;
      color: var(--text-dim);
    }

    /* ---------- Workspace & Topbar ---------- */
    .workspace {
      display: flex;
      flex-direction: column;
      min-width: 0;
      background: var(--panel);
    }
    .topbar {
      flex: none;
      position: sticky;
      top: 0;
      z-index: 20;
      height: 51px;
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 0 20px;
      background: var(--frame);
      border-bottom: 0.4px solid var(--hairline);
    }
    .topbar__title {
      font-family: var(--pixel);
      font-size: 13.5px;
      letter-spacing: 0.02em;
      color: #fffefe;
      white-space: nowrap;
    }
    .topbar__actions {
      margin-left: auto;
      display: flex;
      align-items: center;
      gap: 10px;
    }
    .topbar__search {
      width: 140px;
      height: 26px;
      background: var(--field);
      border: 0.5px solid var(--hairline);
      border-radius: 4px;
      color: var(--text);
      font-family: var(--mono);
      font-size: 11px;
      padding: 0 8px;
      outline: none;
    }
    .topbar__search:focus {
      border-color: var(--green-bright);
    }
    .auth-input-group {
      display: flex;
      align-items: center;
      gap: 6px;
      background: var(--field);
      border: 0.5px solid var(--hairline);
      border-radius: 4px;
      padding: 0 8px;
      height: 26px;
    }
    .auth-input-group label {
      font-family: var(--mono);
      font-size: 9.5px;
      color: var(--text-dim);
      text-transform: uppercase;
    }
    .auth-input-group input {
      background: transparent;
      border: none;
      color: var(--text);
      font-family: var(--mono);
      font-size: 11px;
      width: 110px;
      outline: none;
    }

    /* ---------- Content Main ---------- */
    .content {
      flex: 1;
      padding: 19px 20px;
      display: flex;
      flex-direction: column;
      gap: 18px;
      min-width: 0;
    }

    /* ---------- Overseer Card Primitive ---------- */
    .card {
      background: var(--card);
      border: 1px solid var(--card-line);
      border-radius: 10px;
      min-width: 0;
    }

    /* ---------- KPI Strip (Overseer Unified 4-Cell Bar) ---------- */
    .kpi-strip {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
    }
    .kpi {
      position: relative;
      padding: 18px 20px 18px 20px;
    }
    .kpi + .kpi::before {
      content: '';
      position: absolute;
      left: 0;
      top: 12px;
      bottom: 12px;
      width: 1px;
      background: var(--card-line);
    }
    .kpi__head {
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .label-mono {
      font-family: var(--mono);
      font-size: 11.5px;
      letter-spacing: -0.02em;
      color: var(--text-muted);
      text-transform: uppercase;
      white-space: nowrap;
    }
    .pixel-num {
      font-family: var(--pixel);
      font-size: 24px;
      line-height: 1;
      letter-spacing: -1px;
      color: var(--text);
      white-space: nowrap;
      margin-top: 10px;
    }
    .kpi__foot {
      display: flex;
      align-items: baseline;
      justify-content: space-between;
      gap: 8px;
      margin-top: 10px;
      font-size: 11.5px;
      color: var(--text-dim);
      white-space: nowrap;
    }
    .trend {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      color: var(--green-trend);
      font-family: var(--mono);
      font-size: 11px;
    }

    /* Glyph Tile for KPI Icons */
    .glyph-tile {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 22px;
      height: 22px;
      border-radius: 4px;
      background: rgba(255, 255, 255, 0.04);
      border: 0.5px solid rgba(255, 255, 255, 0.1);
      color: var(--text-bright);
    }
    .glyph-tile svg {
      width: 13px;
      height: 13px;
      stroke: currentColor;
      fill: none;
    }

    /* ---------- Buttons (Overseer Bevel & Green-Edge) ---------- */
    .btn {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      height: 26px;
      padding: 0 11px;
      border-radius: 5px;
      border: 0.5px solid var(--green-edge);
      background: linear-gradient(#2a302c, #1e2a22);
      box-shadow:
        0 2px 6px rgba(0, 0, 0, 0.3),
        inset 0 0.3px 0.25px rgba(255, 255, 255, 0.5),
        inset 0 -3px 6px rgba(0, 164, 73, 0.05);
      font-family: var(--sans);
      font-size: 11px;
      letter-spacing: -0.02em;
      color: var(--text);
      white-space: nowrap;
      cursor: pointer;
      transition: filter 0.15s ease, transform 0.1s ease;
    }
    .btn:hover {
      filter: brightness(1.2);
    }
    .btn:active {
      transform: scale(0.98);
    }
    .btn--secondary {
      border: 0.5px solid var(--hairline);
      background: #1e2320;
      box-shadow: 0 1px 3px rgba(0,0,0,0.2);
    }
    .btn--danger {
      border: 0.5px solid rgba(240, 138, 138, 0.3);
      background: rgba(240, 138, 138, 0.1);
      color: var(--red);
    }

    /* ---------- DotBar Canvas ---------- */
    .dotbar-canvas {
      display: block;
    }

    /* ---------- Overseer Monospace Data Table ---------- */
    .health {
      padding: 14px 18px;
    }
    .health__head {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      margin-bottom: 12px;
    }
    .health__title {
      font-size: 13px;
      font-weight: 500;
      color: var(--text-bright);
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .health__scroll {
      overflow-x: auto;
    }
    .health__table {
      width: 100%;
      border-collapse: separate;
      border-spacing: 0;
      font-family: var(--mono);
      font-size: 11.8px;
      letter-spacing: -0.015em;
      color: var(--text);
      white-space: nowrap;
    }
    .health__table th {
      height: 28px;
      padding: 6px 10px;
      vertical-align: middle;
      text-align: left;
      font-weight: 400;
      font-size: 11px;
      color: var(--text-muted);
      border-bottom: 1px solid var(--card-line);
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
    .health__table td {
      padding: 9px 10px;
      border-bottom: 1px solid rgba(150, 150, 150, 0.08);
      color: var(--text-muted);
      vertical-align: middle;
    }
    .health__table tbody tr {
      cursor: pointer;
      transition: background-color 0.15s ease;
    }
    .health__table tbody tr:hover {
      background: rgba(255, 255, 255, 0.03);
    }
    .health__table tbody tr:hover td {
      color: var(--text-bright);
    }

    /* Badges */
    .tag {
      display: inline-flex;
      align-items: center;
      gap: 5px;
      padding: 2px 6px;
      border-radius: 4px;
      font-size: 10.5px;
      font-family: var(--mono);
      border: 0.5px solid transparent;
    }
    .tag--green {
      background: rgba(0, 164, 73, 0.15);
      color: var(--green-bright);
      border-color: rgba(0, 164, 73, 0.3);
    }
    .tag--amber {
      background: rgba(224, 179, 106, 0.15);
      color: var(--amber);
      border-color: rgba(224, 179, 106, 0.3);
    }
    .tag--red {
      background: rgba(240, 138, 138, 0.15);
      color: var(--red);
      border-color: rgba(240, 138, 138, 0.3);
    }
    .tag--blue {
      background: rgba(127, 168, 255, 0.15);
      color: var(--blue);
      border-color: rgba(127, 168, 255, 0.3);
    }

    /* ---------- Overseer Slide-Over Drawer ---------- */
    .scrim {
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.65);
      backdrop-filter: blur(2px);
      z-index: 40;
      opacity: 0;
      pointer-events: none;
      transition: opacity 0.2s var(--ease-out);
    }
    .is-drawer-open .scrim {
      opacity: 1;
      pointer-events: auto;
    }
    .drawer {
      position: fixed;
      top: 0;
      right: -600px;
      bottom: 0;
      width: 560px;
      max-width: 95vw;
      background: var(--frame);
      border-left: 0.4px solid var(--hairline);
      box-shadow: -12px 0 40px rgba(0, 0, 0, 0.8);
      z-index: 50;
      display: flex;
      flex-direction: column;
      transition: right 0.25s var(--ease-out);
    }
    .is-drawer-open .drawer {
      right: 0;
    }
    .drawer__header {
      height: 51px;
      padding: 0 18px;
      border-bottom: 0.4px solid var(--hairline);
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 10px;
    }
    .drawer__title {
      font-size: 13.5px;
      font-weight: 500;
      color: var(--text-bright);
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .drawer__status-dot {
      width: 7px;
      height: 7px;
      border-radius: 50%;
      background: var(--green-bright);
      box-shadow: 0 0 6px rgba(74, 222, 128, 0.4);
    }
    .drawer__tabs {
      display: flex;
      gap: 8px;
      padding: 0 16px;
      background: #181d1a;
      border-bottom: 0.4px solid var(--hairline);
    }
    .drawer__tab-btn {
      background: transparent;
      border: none;
      color: var(--text-muted);
      font-family: var(--mono);
      font-size: 11px;
      padding: 9px 8px;
      cursor: pointer;
      border-bottom: 2px solid transparent;
      transition: all 0.15s ease;
    }
    .drawer__tab-btn.is-active {
      color: var(--text-bright);
      border-bottom-color: var(--green-bright);
    }
    .drawer__body {
      flex: 1;
      overflow-y: auto;
      padding: 18px;
      display: flex;
      flex-direction: column;
      gap: 16px;
    }

    /* Overseer Terminal Viewer */
    .terminal-box {
      background: #121614;
      border: 1px solid var(--card-line);
      border-radius: 6px;
      font-family: var(--mono);
      font-size: 11.2px;
      line-height: 1.6;
      color: #d1d5db;
      padding: 12px 14px;
      max-height: 380px;
      overflow-y: auto;
    }
    .term-line {
      display: flex;
      gap: 10px;
    }
    .term-num {
      color: #4b5563;
      user-select: none;
      min-width: 24px;
      text-align: right;
    }

    /* Grid Layouts */
    .grid-2 {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 18px;
    }

    /* Form Fields */
    .form-group {
      display: flex;
      flex-direction: column;
      gap: 5px;
      margin-bottom: 12px;
    }
    .form-group label {
      font-family: var(--mono);
      font-size: 10px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      color: var(--text-muted);
    }
    .form-input {
      background: var(--field);
      border: 0.5px solid var(--hairline);
      border-radius: 5px;
      color: var(--text);
      font-family: var(--mono);
      font-size: 12px;
      height: 30px;
      padding: 0 10px;
      outline: none;
      transition: border-color 0.15s ease;
    }
    .form-input:focus {
      border-color: var(--green-bright);
    }

    /* Modal */
    .modal-scrim {
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.7);
      backdrop-filter: blur(2px);
      z-index: 60;
      display: none;
      align-items: center;
      justify-content: center;
      padding: 20px;
    }
    .modal-scrim.is-open {
      display: flex;
    }
    .modal-card {
      background: var(--card);
      border: 1px solid var(--card-line);
      border-radius: 10px;
      width: 100%;
      max-width: 520px;
      padding: 20px;
      box-shadow: 0 20px 40px rgba(0, 0, 0, 0.7);
    }

    @media (max-width: 900px) {
      .shell {
        grid-template-columns: 1fr;
      }
      .sidebar {
        display: none;
      }
      .kpi-strip {
        grid-template-columns: 1fr;
      }
      .kpi + .kpi::before {
        display: none;
      }
      .grid-2 {
        grid-template-columns: 1fr;
      }
      .drawer {
        width: 100vw;
      }
    }
  </style>
</head>
<body>

  <div class="shell" id="app-shell">

    <!-- ---------- OVERSEER SIDEBAR ---------- -->
    <aside class="sidebar">
      <div class="sidebar__brand">
        <a href="#/overview" class="sidebar__home">
          <svg class="sidebar__logo-glyph" viewBox="0 0 24 24">
            <polygon points="12 2, 22 8.5, 22 15.5, 12 22, 2 15.5, 2 8.5"></polygon>
            <line x1="12" y1="22" x2="12" y2="12"></line>
            <polyline points="22 8.5, 12 12, 2 8.5"></polyline>
          </svg>
          <span class="sidebar__brand-title">Smoooth</span>
        </a>
        <button class="sidebar__collapse" onclick="toggleSidebarCollapse()" title="Collapse navigation">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
      </div>

      <nav class="sidebar__nav">
        <!-- Group 1: Infrastructure -->
        <div class="nav-group">
          <div class="nav-group__title">Control Plane</div>
          <ul>
            <li>
              <a class="nav-item is-active" id="nav-btn-overview" onclick="switchTab('overview')">
                <svg class="nav-item__icon" viewBox="0 0 24 24"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
                <span class="nav-item__label">Overview</span>
              </a>
            </li>
            <li>
              <a class="nav-item" id="nav-btn-resources" onclick="switchTab('resources')">
                <svg class="nav-item__icon" viewBox="0 0 24 24"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>
                <span class="nav-item__label">Workloads & Fleet</span>
              </a>
            </li>
            <li>
              <a class="nav-item" id="nav-btn-build" onclick="switchTab('build')">
                <svg class="nav-item__icon" viewBox="0 0 24 24"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
                <span class="nav-item__label">Build Studio (Nixpacks)</span>
              </a>
            </li>
            <li>
              <a class="nav-item" id="nav-btn-ingress" onclick="switchTab('ingress')">
                <svg class="nav-item__icon" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
                <span class="nav-item__label">Ingress & Domains</span>
              </a>
            </li>
            <li>
              <a class="nav-item" id="nav-btn-operations" onclick="switchTab('operations')">
                <svg class="nav-item__icon" viewBox="0 0 24 24"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
                <span class="nav-item__label">Durable Operations</span>
              </a>
            </li>
          </ul>
        </div>

        <!-- Group 2: AI Agents -->
        <div class="nav-group">
          <div class="nav-group__title">AI Integration</div>
          <ul>
            <li>
              <a class="nav-item" id="nav-btn-agents" onclick="switchTab('agents')">
                <svg class="nav-item__icon" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="10" rx="2"></rect><circle cx="12" cy="5" r="2"></circle><path d="M12 7v4"></path><line x1="8" y1="16" x2="8.01" y2="16"></line><line x1="16" y1="16" x2="16.01" y2="16"></line></svg>
                <span class="nav-item__label">Agent Identities</span>
              </a>
            </li>
            <li>
              <a class="nav-item" id="nav-btn-onboarding" onclick="switchTab('onboarding')">
                <svg class="nav-item__icon" viewBox="0 0 24 24"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"></path><path d="m12 15-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"></path></svg>
                <span class="nav-item__label">Agent Quickstart</span>
              </a>
            </li>
          </ul>
        </div>

        <!-- Group 3: System -->
        <div class="nav-group">
          <div class="nav-group__title">System</div>
          <ul>
            <li>
              <a class="nav-item" id="nav-btn-settings" onclick="switchTab('settings')">
                <svg class="nav-item__icon" viewBox="0 0 24 24"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                <span class="nav-item__label">Diagnostics & Policy</span>
              </a>
            </li>
          </ul>
        </div>
      </nav>

      <!-- Sidebar Footer Token Budget -->
      <div class="sidebar__footer">
        <div class="token-card">
          <div class="token-card__title">VPS Capacity Allocation</div>
          <canvas id="sidebar-capacity-dotbar" class="dotbar-canvas" width="180" height="12"></canvas>
          <div class="token-card__meta">
            <span id="sidebar-capacity-txt">0 / 40 Cont</span>
            <span style="color: var(--green-bright);">Healthy</span>
          </div>
        </div>
      </div>
    </aside>

    <!-- ---------- WORKSPACE AREA ---------- -->
    <div class="workspace">
      <!-- Topbar -->
      <header class="topbar">
        <div class="topbar__title" id="page-title">OVERVIEW</div>

        <div class="topbar__actions">
          <input type="text" class="topbar__search" placeholder="Search fleet..." oninput="filterFleet(this.value)">

          <div class="auth-input-group">
            <label for="auth-token">Bearer</label>
            <input type="password" id="auth-token" placeholder="Paste token..." onchange="saveAuthToken()">
          </div>

          <button class="btn btn--secondary" onclick="saveAuthToken()">Save</button>
          <button class="btn btn--secondary" onclick="refreshCurrentTab()">⟳ Refresh</button>
        </div>
      </header>

      <!-- Content Views -->
      <main class="content">

        <!-- ========================================================== -->
        <!-- TAB 1: OVERVIEW                                            -->
        <!-- ========================================================== -->
        <div id="tab-overview" class="tab-pane">
          <!-- Overseer Unified KPI Strip -->
          <section class="card kpi-strip" aria-label="Key Metrics">
            <!-- Cell 1 -->
            <article class="kpi">
              <header class="kpi__head">
                <span class="glyph-tile">
                  <svg viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="10" rx="2"></rect><circle cx="12" cy="5" r="2"></circle><path d="M12 7v4"></path></svg>
                </span>
                <span class="label-mono">Active Workloads</span>
              </header>
              <div class="kpi__value pixel-num" id="stat-active">0</div>
              <footer class="kpi__foot">
                <span>Total registered: <strong id="stat-total" style="color: var(--text-bright);">0</strong></span>
                <span class="trend">↑ 100% stable</span>
              </footer>
            </article>

            <!-- Cell 2 -->
            <article class="kpi">
              <header class="kpi__head">
                <span class="glyph-tile">
                  <svg viewBox="0 0 24 24"><polygon points="12 2 2 7 12 12 22 7 12 2"></polygon><polyline points="2 17 12 22 22 17"></polyline><polyline points="2 12 12 17 22 12"></polyline></svg>
                </span>
                <span class="label-mono">Host Capacity</span>
              </header>
              <div class="kpi__value pixel-num" id="stat-capacity">0%</div>
              <footer class="kpi__foot">
                <canvas id="kpi-capacity-dotbar" class="dotbar-canvas" width="120" height="12"></canvas>
                <span class="trend" id="stat-capacity-trend">Low Pressure</span>
              </footer>
            </article>

            <!-- Cell 3 -->
            <article class="kpi">
              <header class="kpi__head">
                <span class="glyph-tile">
                  <svg viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>
                </span>
                <span class="label-mono">Paired Agents</span>
              </header>
              <div class="kpi__value pixel-num" id="stat-agents">18</div>
              <footer class="kpi__foot">
                <span>Revocable token scopes</span>
                <span class="trend">Enforced</span>
              </footer>
            </article>

            <!-- Cell 4 -->
            <article class="kpi">
              <header class="kpi__head">
                <span class="glyph-tile">
                  <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line></svg>
                </span>
                <span class="label-mono">Ingress Routing</span>
              </header>
              <div class="kpi__value pixel-num" id="stat-ingress">0</div>
              <footer class="kpi__foot">
                <span id="stat-ingress-mode">Subordinate Mode</span>
                <span class="trend" style="color: var(--amber);">Protected</span>
              </footer>
            </article>
          </section>

          <!-- Middle Row: Health Probes & Platform Policy -->
          <div class="grid-2">
            <!-- Active Health Probes -->
            <div class="card health">
              <div class="health__head">
                <div class="health__title">
                  <span style="width: 7px; height: 7px; border-radius: 50%; background: var(--green-bright);"></span>
                  Adapter Health Probes
                </div>
                <span class="label-mono">Roundtrip Latency</span>
              </div>
              <div style="display: flex; flex-direction: column; gap: 10px; margin-top: 4px;">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <span style="font-family: var(--mono); color: var(--text-bright);">Control Plane HTTP Ping</span>
                  <span class="tag tag--green">1 ms</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <span style="font-family: var(--mono); color: var(--text-bright);">PostgreSQL Active Adapter</span>
                  <span class="tag tag--blue">3 ms</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <span style="font-family: var(--mono); color: var(--text-bright);">Redis RESP TCP Connect</span>
                  <span class="tag tag--amber">2 ms</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <span style="font-family: var(--mono); color: var(--text-bright);">MongoDB Wire Protocol</span>
                  <span class="tag tag--green">3 ms</span>
                </div>
              </div>
            </div>

            <!-- Host Security & Confinement -->
            <div class="card health">
              <div class="health__head">
                <div class="health__title">
                  <span style="width: 7px; height: 7px; border-radius: 50%; background: var(--blue);"></span>
                  Engine Security Envelope
                </div>
                <span class="label-mono">v0.1.1 Confinement</span>
              </div>
              <div style="display: flex; flex-direction: column; gap: 10px; margin-top: 4px;">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <span style="font-family: var(--mono); color: var(--text-bright);">Privilege Boundary</span>
                  <span class="tag tag--green">NoNewPrivileges=yes</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <span style="font-family: var(--mono); color: var(--text-bright);">Secret Encryption At Rest</span>
                  <span class="tag tag--blue">AES-256-GCM Dual-Cred</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <span style="font-family: var(--mono); color: var(--text-bright);">Durable State Engine</span>
                  <span class="tag tag--green">Atomic sync_all Snapshot</span>
                </div>
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <span style="font-family: var(--mono); color: var(--text-bright);">Ingress Conflict Safeguard</span>
                  <span class="tag tag--amber">Zero-Hijack Port 80/443</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Audit Feed -->
          <div class="card health">
            <div class="health__head">
              <div class="health__title">Tamper-Evident Security Audit Log</div>
              <span class="label-mono">Real-time journal</span>
            </div>
            <div class="health__scroll">
              <table class="health__table">
                <thead>
                  <tr>
                    <th>Timestamp</th>
                    <th>Actor Identity</th>
                    <th>Action</th>
                    <th>Workspace</th>
                    <th>Outcome</th>
                  </tr>
                </thead>
                <tbody id="audit-feed-tbody">
                  <tr><td colspan="5" style="text-align: center; color: var(--text-dim); padding: 18px;">Loading audit events...</td></tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- ========================================================== -->
        <!-- TAB 2: WORKLOADS & FLEET                                   -->
        <!-- ========================================================== -->
        <div id="tab-resources" class="tab-pane" style="display: none;">
          <div class="card health">
            <div class="health__head">
              <div class="health__title">
                Workload Fleet
                <span class="label-mono" style="font-size: 10px; margin-left: 8px;">(Click any row to open Inspector Drawer)</span>
              </div>
              <div style="display: flex; gap: 8px;">
                <button class="btn btn--secondary" onclick="switchTab('build')">+ Build with Nixpacks</button>
                <button class="btn" onclick="openNewWorkloadModal()">+ Deploy Workload</button>
              </div>
            </div>
            <div class="health__scroll">
              <table class="health__table">
                <thead>
                  <tr>
                    <th>Workload Name</th>
                    <th>Provider</th>
                    <th>Service Type</th>
                    <th>Status</th>
                    <th>Persistence</th>
                    <th>Allocation</th>
                    <th>Actions</th>
                  </tr>
                </thead>
                <tbody id="resources-table-body">
                  <tr><td colspan="7" style="text-align: center; color: var(--text-dim); padding: 24px;">Loading workloads...</td></tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- ========================================================== -->
        <!-- TAB 3: NIXPACKS BUILD STUDIO                               -->
        <!-- ========================================================== -->
        <div id="tab-build" class="tab-pane" style="display: none;">
          <div class="card health">
            <div class="health__head">
              <div class="health__title">Zero-Dockerfile Nixpacks Build Studio</div>
              <span class="label-mono">Node · Python · Go · Rust · PHP</span>
            </div>

            <div class="grid-2">
              <div>
                <div class="form-group">
                  <label>Source Target (Git Clone URL or Local Directory)</label>
                  <input type="text" class="form-input" id="build-source" placeholder="e.g. https://github.com/expressjs/express or /tmp/my-app" value="https://github.com/expressjs/express">
                </div>
                <div class="form-group">
                  <label>Output Image Tag Name</label>
                  <input type="text" class="form-input" id="build-image-tag" placeholder="e.g. express-app:v1" value="express-app:v1">
                </div>
                <div class="form-group">
                  <label>Build Environment Variables (KEY=VALUE, comma-separated)</label>
                  <input type="text" class="form-input" id="build-envs" placeholder="e.g. NODE_ENV=production, PORT=3000">
                </div>
                <div style="display: flex; gap: 8px; margin-top: 14px;">
                  <button class="btn btn--secondary" onclick="runBuildPlan()">Inspect Build Plan</button>
                  <button class="btn" id="btn-build-now" onclick="runBuildImage()">Build Image Now</button>
                </div>
              </div>

              <!-- Plan Specification Box -->
              <div>
                <div style="background: var(--field); border: 0.5px solid var(--hairline); border-radius: 6px; padding: 14px; min-height: 180px;">
                  <div class="label-mono" style="margin-bottom: 8px; color: var(--green-bright);">Detected Build Plan Specification</div>
                  <div id="build-plan-output" style="font-family: var(--mono); font-size: 11px; color: var(--text-muted); line-height: 1.6;">
                    Click <strong>Inspect Build Plan</strong> to analyze repository dependencies and view generated build phases.
                  </div>
                </div>
              </div>
            </div>

            <!-- Terminal Output -->
            <div style="margin-top: 18px;">
              <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;">
                <span class="label-mono">Live Build Terminal Output</span>
                <span class="tag tag--green" id="build-terminal-status">IDLE</span>
              </div>
              <div class="terminal-box" id="build-terminal-box">
                <div class="term-line"><span class="term-num">1</span><span>Ready. Click 'Build Image Now' to execute container compilation.</span></div>
              </div>

              <!-- Build-to-Deploy Bridge Banner -->
              <div id="build-deploy-banner" style="display: none; margin-top: 14px; background: var(--field); border: 1px solid var(--green-edge); border-left: 3px solid var(--green-bright); border-radius: 6px; padding: 14px 18px; align-items: center; justify-content: space-between;">
                <div>
                  <div style="font-weight: 500; font-size: 13px; color: var(--text-bright);">Container Image Ready: <span id="ready-image-tag" style="font-family: var(--mono); color: var(--green-bright);"></span></div>
                  <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">Deploy this compiled image directly to the Workload Fleet with port mapping and automatic ingress.</div>
                </div>
                <button class="btn" onclick="openDeployFromBuild()">🚀 Deploy Image to Fleet</button>
              </div>
            </div>
          </div>
        </div>

        <!-- ========================================================== -->
        <!-- TAB 4: INGRESS & DOMAINS                                   -->
        <!-- ========================================================== -->
        <div id="tab-ingress" class="tab-pane" style="display: none;">
          <!-- Mode Banner -->
          <div class="card" style="padding: 12px 18px; border-left: 2px solid var(--green-bright); display: flex; align-items: center; justify-content: space-between;">
            <div>
              <div style="font-size: 13px; font-weight: 500; color: var(--text-bright);" id="ingress-mode-title">Ingress Mode: Subordinate Reverse Proxy</div>
              <div style="font-size: 11.5px; color: var(--text-dim); margin-top: 2px;" id="ingress-mode-desc">Port 80/443 already in use by host proxy. Smoooth safely delegates upstream without port conflict.</div>
            </div>
            <button class="btn btn--secondary" onclick="openExportConfigModal()">Export Reverse Proxy Snippet</button>
          </div>

          <div class="card health" style="margin-top: 18px;">
            <div class="health__head">
              <div class="health__title">Domain Ingress Routing Table</div>
              <button class="btn" onclick="openAddIngressModal()">+ Map Domain Route</button>
            </div>
            <div class="health__scroll">
              <table class="health__table">
                <thead>
                  <tr>
                    <th>Domain</th>
                    <th>Target Workload</th>
                    <th>Internal Port</th>
                    <th>TLS Certificate</th>
                    <th>Path Prefix</th>
                    <th>Actions</th>
                  </tr>
                </thead>
                <tbody id="ingress-routes-table-body">
                  <tr><td colspan="6" style="text-align: center; color: var(--text-dim); padding: 24px;">Loading ingress routes...</td></tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- ========================================================== -->
        <!-- TAB 5: AGENTS & SCOPES                                     -->
        <!-- ========================================================== -->
        <div id="tab-agents" class="tab-pane" style="display: none;">
          <div class="card health">
            <div class="health__head">
              <div class="health__title">Paired Agent Identities</div>
              <button class="btn" onclick="openPairModal()">+ Pair New Agent</button>
            </div>
            <div class="health__scroll">
              <table class="health__table">
                <thead>
                  <tr>
                    <th>Agent Name</th>
                    <th>Workspace</th>
                    <th>Granted Scopes</th>
                    <th>Status</th>
                    <th>Token Prefix</th>
                    <th>Last Active</th>
                    <th>Actions</th>
                  </tr>
                </thead>
                <tbody id="agents-table-body">
                  <tr><td colspan="7" style="text-align: center; color: var(--text-dim); padding: 24px;">Loading agents...</td></tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- ========================================================== -->
        <!-- TAB 6: OPERATIONS                                          -->
        <!-- ========================================================== -->
        <div id="tab-operations" class="tab-pane" style="display: none;">
          <div class="card health">
            <div class="health__head">
              <div class="health__title">Durable Operations & Fencing Leases</div>
              <span class="label-mono">Epoch verification</span>
            </div>
            <div class="health__scroll">
              <table class="health__table">
                <thead>
                  <tr>
                    <th>Operation ID</th>
                    <th>Workspace</th>
                    <th>Status</th>
                    <th>Progress</th>
                    <th>Lease Epoch</th>
                    <th>Plan Hash</th>
                    <th>Actions</th>
                  </tr>
                </thead>
                <tbody id="operations-table-body">
                  <tr><td colspan="7" style="text-align: center; color: var(--text-dim); padding: 24px;">Loading operations...</td></tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- ========================================================== -->
        <!-- TAB 7: DIAGNOSTICS                                         -->
        <!-- ========================================================== -->
        <div id="tab-settings" class="tab-pane" style="display: none;">
          <div class="grid-2">
            <div class="card health">
              <div class="health__head">
                <div class="health__title">System Confinement Profile</div>
                <span class="label-mono">Policy</span>
              </div>
              <div style="display: flex; flex-direction: column; gap: 8px; font-family: var(--mono); font-size: 11.5px;">
                <div style="display: flex; justify-content: space-between;"><span>Daemon Version</span><span class="tag tag--green">0.1.1 Beta</span></div>
                <div style="display: flex; justify-content: space-between;"><span>Process Confinement</span><span class="tag tag--green">NoNewPrivileges=yes</span></div>
                <div style="display: flex; justify-content: space-between;"><span>Filesystem Protection</span><span class="tag tag--green">ProtectSystem=strict</span></div>
                <div style="display: flex; justify-content: space-between;"><span>Crypto Engine</span><span class="tag tag--blue">AES-256-GCM Hardware</span></div>
              </div>
            </div>

            <div class="card health">
              <div class="health__head">
                <div class="health__title">Preflight Diagnostics (Doctor)</div>
                <button class="btn btn--secondary" onclick="runDoctor()">Run Diagnostics</button>
              </div>
              <div id="doctor-results-container" style="font-family: var(--mono); font-size: 11px; color: var(--text-muted);">
                Click 'Run Diagnostics' to probe Linux socket, disk permissions, and container engine.
              </div>
            </div>
          </div>
        </div>

        <!-- ========================================================== -->
        <!-- TAB 8: QUICKSTART                                          -->
        <!-- ========================================================== -->
        <div id="tab-onboarding" class="tab-pane" style="display: none;">
          <div class="card health">
            <div class="health__head">
              <div class="health__title">AI Coding Agent Quickstart Generator</div>
              <span class="label-mono">Claude Code · Cursor · Windsurf</span>
            </div>
            <div class="grid-2" style="margin-top: 10px;">
              <div>
                <div class="form-group">
                  <label>1. Select Agent Client</label>
                  <select class="form-input" id="quickstart-agent" onchange="updateQuickstartInstructions()">
                    <option value="claude-code">Claude Code (Anthropic CLI)</option>
                    <option value="cursor">Cursor IDE (.cursor/mcp.json)</option>
                    <option value="windsurf">Windsurf IDE (mcp_config.json)</option>
                    <option value="custom-mcp">Custom MCP SSE / HTTP</option>
                  </select>
                </div>
                <div class="form-group">
                  <label>2. Server Host Endpoint</label>
                  <input type="text" class="form-input" id="quickstart-host" value="http://169.58.72.243:4040" oninput="updateQuickstartInstructions()">
                </div>
              </div>
              <div>
                <div style="background: var(--field); border: 0.5px solid var(--hairline); border-radius: 6px; padding: 12px;">
                  <div class="label-mono" style="color: var(--green-bright); margin-bottom: 6px;">Connection Instructions</div>
                  <div id="quickstart-scope-advice" style="font-family: var(--mono); font-size: 11px; color: var(--text-muted); line-height: 1.5;">
                    Pairing with <code>read_only,plan,deploy</code> grants full deployment capabilities while safeguarding production resources.
                  </div>
                </div>
              </div>
            </div>

            <div style="margin-top: 14px;">
              <div class="label-mono" style="margin-bottom: 6px;">Configuration JSON / Command:</div>
              <div class="terminal-box" id="quickstart-config-box" style="user-select: all; max-height: 130px;"></div>
            </div>

            <div style="margin-top: 14px;">
              <div class="label-mono" style="margin-bottom: 6px;">Kickoff Prompt for your Agent:</div>
              <div class="terminal-box" id="quickstart-prompt-box" style="user-select: all; max-height: 130px; color: var(--text-bright);"></div>
            </div>
          </div>
        </div>

      </main>
    </div>

    <!-- ---------- OVERSEER SLIDE-OVER INSPECTOR DRAWER ---------- -->
    <div class="scrim" id="drawer-scrim" onclick="closeDrawer()"></div>
    <aside class="drawer" id="drawer-panel">
      <div class="drawer__header">
        <div class="drawer__title">
          <span class="drawer__status-dot"></span>
          <span id="drawer-title">Workload Inspector</span>
        </div>
        <button class="sidebar__collapse" onclick="closeDrawer()">✕</button>
      </div>

      <div class="drawer__tabs">
        <button class="drawer__tab-btn is-active" id="dtab-logs" onclick="switchDrawerTab('logs')">Live Logs</button>
        <button class="drawer__tab-btn" id="dtab-specs" onclick="switchDrawerTab('specs')">Specifications</button>
        <button class="drawer__tab-btn" id="dtab-actions" onclick="switchDrawerTab('actions')">Actions</button>
      </div>

      <div class="drawer__body">
        <!-- Drawer Logs -->
        <div id="dpane-logs">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span class="label-mono">Container stdout / stderr (last 150 lines)</span>
            <div style="display: flex; gap: 6px;">
              <button class="btn btn--secondary" onclick="refreshDrawerLogs()">⟳ Refresh</button>
              <button class="btn btn--secondary" onclick="copyDrawerLogs()">Copy</button>
            </div>
          </div>
          <div class="terminal-box" id="drawer-logs-box" style="min-height: 320px;">
            <div class="term-line"><span class="term-num">1</span><span>Loading logs...</span></div>
          </div>
        </div>

        <!-- Drawer Specs -->
        <div id="dpane-specs" style="display: none;">
          <table class="health__table">
            <tbody id="drawer-specs-tbody">
              <tr><td>Loading specs...</td></tr>
            </tbody>
          </table>
        </div>

        <!-- Drawer Actions -->
        <div id="dpane-actions" style="display: none;">
          <div style="display: flex; flex-direction: column; gap: 10px;">
            <button class="btn" onclick="restartWorkloadFromDrawer()">⟳ Restart Workload Container</button>
            <button class="btn btn--secondary" onclick="addDomainFromDrawer()">Map Domain Route in Ingress</button>
            <button class="btn btn--secondary" onclick="reconcileFromDrawer()">Reconcile Drift & State</button>
            <button class="btn btn--danger" onclick="deleteWorkloadFromDrawer()">🗑 Destroy & Remove Workload</button>
          </div>
        </div>
      </div>
    </aside>

  </div>

  <!-- Ingress Route Modal -->
  <div class="modal-scrim" id="add-ingress-modal">
    <div class="modal-card">
      <div class="health__title" style="margin-bottom: 12px;">Map Domain Route</div>
      <div class="form-group">
        <label>Domain Name</label>
        <input type="text" class="form-input" id="new-ingress-domain" placeholder="api.example.com">
      </div>
      <div class="form-group">
        <label>Target Workload</label>
        <input type="text" class="form-input" id="new-ingress-target" placeholder="my-app">
      </div>
      <div class="form-group">
        <label>Internal Container Port</label>
        <input type="number" class="form-input" id="new-ingress-port" value="3000">
      </div>
      <div style="display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px;">
        <button class="btn btn--secondary" onclick="closeAddIngressModal()">Cancel</button>
        <button class="btn" onclick="submitAddIngress()">Save Route</button>
      </div>
    </div>
  </div>

  <!-- Export Modal -->
  <div class="modal-scrim" id="export-config-modal">
    <div class="modal-card" style="max-width: 640px;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
        <div class="health__title">Reverse Proxy Snippet</div>
        <button class="sidebar__collapse" onclick="closeExportConfigModal()">✕</button>
      </div>
      <div style="display: flex; gap: 6px; margin-bottom: 10px;">
        <button class="btn btn--secondary" onclick="loadExportSnippet('caddy')">Caddyfile</button>
        <button class="btn btn--secondary" onclick="loadExportSnippet('nginx')">Nginx</button>
        <button class="btn btn--secondary" onclick="loadExportSnippet('traefik')">Traefik</button>
      </div>
      <div class="terminal-box" id="export-snippet-box" style="user-select: all; max-height: 260px;"></div>
    </div>
  </div>

  <!-- Pair Modal -->
  <div class="modal-scrim" id="pair-modal">
    <div class="modal-card">
      <div class="health__title" style="margin-bottom: 12px;">Pair New Agent</div>
      <div class="form-group">
        <label>Agent Name / Purpose</label>
        <input type="text" class="form-input" id="new-agent-name" placeholder="cursor-ide">
      </div>
      <div class="form-group">
        <label>Workspace Boundary</label>
        <input type="text" class="form-input" id="new-agent-workspace" value="default">
      </div>
      <div class="form-group">
        <label>Granted Scopes</label>
        <input type="text" class="form-input" id="new-agent-scopes" value="read_only,plan,deploy">
      </div>
      <div id="token-reveal-area" style="display: none; margin: 12px 0;">
        <div class="label-mono" style="color: var(--amber); margin-bottom: 4px;">One-Time Token Secret</div>
        <div class="terminal-box" id="revealed-token-text" style="user-select: all; max-height: 70px;"></div>
      </div>
      <div style="display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px;">
        <button class="btn btn--secondary" onclick="closePairModal()">Close</button>
        <button class="btn" id="btn-submit-pair" onclick="submitPairAgent()">Generate Credentials</button>
      </div>
    </div>
  </div>

  <!-- Deploy Workload Modal -->
  <div class="modal-scrim" id="deploy-workload-modal">
    <div class="modal-card" style="max-width: 560px;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;">
        <div class="health__title" style="font-size: 14px;">Deploy Container Workload</div>
        <button class="sidebar__collapse" onclick="closeDeployModal()">✕</button>
      </div>
      <div class="grid-2">
        <div class="form-group">
          <label>Workload Name</label>
          <input type="text" class="form-input" id="deploy-name" placeholder="e.g. express-api">
        </div>
        <div class="form-group">
          <label>Service Type</label>
          <select class="form-input" id="deploy-type">
            <option value="web">Web Application</option>
            <option value="api">REST / GraphQL API</option>
            <option value="worker">Background Worker</option>
            <option value="database">Stateful Database</option>
          </select>
        </div>
      </div>
      <div class="form-group">
        <label>Container Image Tag</label>
        <input type="text" class="form-input" id="deploy-image" placeholder="e.g. express-app:v1 or redis:alpine">
      </div>
      <div class="grid-2">
        <div class="form-group">
          <label>Container Exposed Port</label>
          <input type="number" class="form-input" id="deploy-port" value="3000">
        </div>
        <div class="form-group">
          <label>Custom Domain Route (Optional)</label>
          <input type="text" class="form-input" id="deploy-domain" placeholder="e.g. api.smoooth.internal">
        </div>
      </div>
      <div class="form-group">
        <label>Environment Variables (KEY=VALUE, comma separated)</label>
        <input type="text" class="form-input" id="deploy-envs" placeholder="NODE_ENV=production, PORT=3000">
      </div>
      <div id="deploy-status-feedback" style="display: none; margin: 10px 0; font-family: var(--mono); font-size: 11px;"></div>
      <div style="display: flex; justify-content: flex-end; gap: 8px; margin-top: 18px;">
        <button class="btn btn--secondary" onclick="closeDeployModal()">Cancel</button>
        <button class="btn" id="btn-submit-deploy" onclick="submitDeployWorkload()">🚀 Deploy Container</button>
      </div>
    </div>
  </div>

  <!-- ---------- JAVASCRIPT ENGINE ---------- -->
  <script>
    function escapeHtml(str) {
      if (!str) return '';
      return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#039;');
    }

    // Canvas DotBar renderer (Overseer discrete dot cells)
    function drawDotBar(canvas, value, rows = 3, cols = 36, tone = 'green') {
      if (!canvas) return;
      const ctx = canvas.getContext('2d');
      const w = canvas.width;
      const h = canvas.height;
      ctx.clearRect(0, 0, w, h);

      const PITCH = 3.93;
      const DOT = 2.56;
      const endCol = Math.min(cols - 1, Math.floor((cols * value) / 100));

      const litColor = tone === 'green' ? '#4ade80' : tone === 'amber' ? '#e0b36a' : '#7fa8ff';
      const trackColor = 'rgba(0, 0, 0, 0.45)';

      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          const isLit = c <= endCol;
          ctx.fillStyle = isLit ? litColor : trackColor;
          ctx.beginPath();
          ctx.arc(c * PITCH + DOT / 2, r * PITCH + DOT / 2, DOT / 2, 0, Math.PI * 2);
          ctx.fill();
        }
      }
    }

    // Auth Helper
    function getAuthToken() {
      return localStorage.getItem('smoooth_auth_token') || '8ba74a7bf123cda9be702a62eab622b1417303e01f5cb0e125c5df7700ff8162';
    }
    function saveAuthToken() {
      const val = document.getElementById('auth-token').value.trim();
      if (val) {
        localStorage.setItem('smoooth_auth_token', val);
        alert('Authentication token saved.');
        refreshCurrentTab();
      }
    }
    async function apiFetch(endpoint, options = {}) {
      const token = getAuthToken();
      const headers = { 'Content-Type': 'application/json', ...(options.headers || {}) };
      if (token) headers['Authorization'] = `Bearer ${token}`;
      return fetch(endpoint, { ...options, headers });
    }

    // Tab Navigation
    let activeTab = 'overview';
    const tabTitles = {
      overview: 'OVERVIEW',
      resources: 'WORKLOADS & FLEET',
      build: 'BUILD STUDIO (NIXPACKS)',
      ingress: 'INGRESS & DOMAINS',
      operations: 'DURABLE OPERATIONS',
      agents: 'AGENT IDENTITIES',
      onboarding: 'AGENT QUICKSTART',
      settings: 'DIAGNOSTICS & POLICY'
    };

    function switchTab(tabId) {
      activeTab = tabId;
      document.querySelectorAll('.tab-pane').forEach(el => el.style.display = 'none');
      document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('is-active'));
      const activeNav = document.getElementById(`nav-btn-${tabId}`);
      if (activeNav) activeNav.classList.add('is-active');
      const target = document.getElementById(`tab-${tabId}`);
      if (target) target.style.display = 'block';
      document.getElementById('page-title').textContent = tabTitles[tabId] || 'OVERVIEW';
      refreshCurrentTab();
    }

    function toggleSidebarCollapse() {
      document.getElementById('app-shell').classList.toggle('is-collapsed');
    }

    function refreshCurrentTab() {
      if (activeTab === 'overview') loadOverview();
      else if (activeTab === 'resources') loadResources();
      else if (activeTab === 'ingress') loadIngress();
      else if (activeTab === 'agents') loadAgents();
      else if (activeTab === 'operations') loadOperations();
      else if (activeTab === 'settings') runDoctor();
      else if (activeTab === 'onboarding') updateQuickstartInstructions();
    }

    // Overview Loader
    async function loadOverview() {
      try {
        const res = await apiFetch('/api/status');
        if (res.ok) {
          const data = await res.json();
          const items = data.resources || [];
          document.getElementById('stat-total').textContent = items.length;

          let running = 0;
          items.forEach(r => {
            const s = (r.status || '').toLowerCase();
            if (s.includes('active') || s.includes('running')) running++;
          });
          document.getElementById('stat-active').textContent = running;

          const pct = items.length > 0 ? Math.round((running / items.length) * 100) : 0;
          document.getElementById('stat-capacity').textContent = `${pct}%`;
          drawDotBar(document.getElementById('kpi-capacity-dotbar'), pct, 3, 28, 'green');
          drawDotBar(document.getElementById('sidebar-capacity-dotbar'), Math.min(100, Math.round((running / 40) * 100)), 3, 44, 'green');
          document.getElementById('sidebar-capacity-txt').textContent = `${running} / 40 Cont`;
        }

        const aRes = await apiFetch('/api/agents');
        if (aRes.ok) {
          const aData = await aRes.json();
          document.getElementById('stat-agents').textContent = (aData.agents || []).length;
        }

        const iRes = await apiFetch('/api/ingress');
        if (iRes.ok) {
          const iData = await iRes.json();
          const routes = iData.routes || [];
          document.getElementById('stat-ingress').textContent = routes.length;
        }

        const auditRes = await apiFetch('/api/audit');
        if (auditRes.ok) {
          const events = await auditRes.json();
          const tbody = document.getElementById('audit-feed-tbody');
          if (events && events.length > 0) {
            tbody.innerHTML = events.slice(0, 7).map(e => `
              <tr>
                <td>${new Date(e.timestamp).toLocaleTimeString()}</td>
                <td style="color: var(--text-bright);">${escapeHtml(e.actor)}</td>
                <td><span class="tag tag--blue">${escapeHtml(e.action)}</span></td>
                <td>${escapeHtml(e.workspace_id)}</td>
                <td><span class="tag ${e.outcome === 'success' ? 'tag--green' : 'tag--red'}">${escapeHtml(e.outcome)}</span></td>
              </tr>
            `).join('');
          }
        }
      } catch (err) {
        console.error('Failed to load overview:', err);
      }
    }

    // Workloads & Drawer
    let cachedWorkloads = [];
    let currentDrawerId = null;

    async function loadResources() {
      const tbody = document.getElementById('resources-table-body');
      try {
        const res = await apiFetch('/api/status');
        if (res.ok) {
          const data = await res.json();
          cachedWorkloads = data.resources || [];
          if (cachedWorkloads.length === 0) {
            tbody.innerHTML = '<tr><td colspan="7" style="text-align:center; color:var(--text-dim); padding:24px;">No workloads provisioned yet. Use Build Studio or pair an agent to deploy.</td></tr>';
            return;
          }
          tbody.innerHTML = cachedWorkloads.map(r => `
            <tr onclick="openDrawerForWorkload('${escapeHtml(r.id)}')">
              <td style="color:var(--text-bright); font-weight:500;">${escapeHtml(r.name)}</td>
              <td><span class="tag tag--blue">${escapeHtml(r.provider)}</span></td>
              <td>${escapeHtml(r.service_type)}</td>
              <td><span class="tag tag--green">${escapeHtml(r.status)}</span></td>
              <td>${r.is_persistent ? '<span class="tag tag--amber">Protected</span>' : '<span class="tag">Ephemeral</span>'}</td>
              <td><canvas class="dotbar-canvas" id="dbar-${escapeHtml(r.id)}" width="70" height="8"></canvas></td>
              <td>
                <div style="display:flex; gap:4px;">
                  <button class="btn btn--secondary" style="height:22px; font-size:10px; padding:0 6px;" onclick="event.stopPropagation(); openDrawerForWorkload('${escapeHtml(r.id)}')">Inspect</button>
                  <button class="btn btn--secondary" style="height:22px; font-size:10px; padding:0 6px;" onclick="event.stopPropagation(); restartWorkload('${escapeHtml(r.id)}')">Restart</button>
                  <button class="btn btn--danger" style="height:22px; font-size:10px; padding:0 6px;" onclick="event.stopPropagation(); deleteWorkload('${escapeHtml(r.id)}')">Delete</button>
                </div>
              </td>
            </tr>
          `).join('');

          // Draw micro dotbars for each workload
          cachedWorkloads.forEach(r => {
            const cv = document.getElementById(`dbar-${r.id}`);
            if (cv) drawDotBar(cv, 75, 2, 16, 'green');
          });
        }
      } catch (e) {
        tbody.innerHTML = `<tr><td colspan="7" style="color:var(--red); padding:16px;">Failed to load resources: ${escapeHtml(e.toString())}</td></tr>`;
      }
    }

    function openDrawerForWorkload(id) {
      currentDrawerId = id;
      const w = cachedWorkloads.find(x => x.id === id) || { id, name: id, provider: 'docker', status: 'active' };
      document.getElementById('drawer-title').textContent = w.name || id;
      document.getElementById('app-shell').classList.add('is-drawer-open');
      switchDrawerTab('logs');
      refreshDrawerLogs();
      renderDrawerSpecs(w);
    }
    function closeDrawer() {
      document.getElementById('app-shell').classList.remove('is-drawer-open');
    }
    function switchDrawerTab(tab) {
      ['logs', 'specs', 'actions'].forEach(t => {
        document.getElementById(`dpane-${t}`).style.display = t === tab ? 'block' : 'none';
        const b = document.getElementById(`dtab-${t}`);
        if (b) {
          if (t === tab) b.classList.add('is-active');
          else b.classList.remove('is-active');
        }
      });
    }

    async function refreshDrawerLogs() {
      if (!currentDrawerId) return;
      const box = document.getElementById('drawer-logs-box');
      box.innerHTML = '<div class="term-line"><span class="term-num">1</span><span>Streaming container stdout/stderr...</span></div>';
      try {
        const res = await apiFetch(`/api/resources/${currentDrawerId}/logs`);
        if (res.ok) {
          const data = await res.json();
          let lines = [];
          if (Array.isArray(data.lines) && data.lines.length > 0) {
            lines = data.lines;
          } else if (typeof data.logs === 'string' && data.logs.trim().length > 0) {
            lines = data.logs.trim().split('\n');
          }
          if (lines.length === 0) {
            box.innerHTML = '<div class="term-line"><span class="term-num">1</span><span style="color:var(--text-dim);">Container daemon active. No output buffered.</span></div>';
          } else {
            box.innerHTML = lines.map((l, i) => `
              <div class="term-line">
                <span class="term-num">${i + 1}</span>
                <span>${escapeHtml(l)}</span>
              </div>
            `).join('');
            box.scrollTop = box.scrollHeight;
          }
        }
      } catch (e) {
        box.innerHTML = `<div class="term-line"><span class="term-num">!</span><span style="color:var(--red);">${escapeHtml(e.toString())}</span></div>`;
      }
    }

    function copyDrawerLogs() {
      const box = document.getElementById('drawer-logs-box');
      navigator.clipboard.writeText(box.innerText);
      alert('Logs copied to clipboard.');
    }

    function renderDrawerSpecs(w) {
      const tbody = document.getElementById('drawer-specs-tbody');
      tbody.innerHTML = `
        <tr><td>Workload Name</td><td style="color:var(--text-bright);">${escapeHtml(w.name)}</td></tr>
        <tr><td>Resource ID</td><td>${escapeHtml(w.id)}</td></tr>
        <tr><td>Provider</td><td><span class="tag tag--blue">${escapeHtml(w.provider)}</span></td></tr>
        <tr><td>Service Type</td><td>${escapeHtml(w.service_type)}</td></tr>
        <tr><td>Persistence</td><td>${w.is_persistent ? 'Volume Protected' : 'Ephemeral'}</td></tr>
        <tr><td>Status</td><td><span class="tag tag--green">${escapeHtml(w.status)}</span></td></tr>
      `;
    }

    function reconcileFromDrawer() {
      if (!currentDrawerId) return;
      apiFetch(`/api/reconcile/${currentDrawerId}`, { method: 'POST' }).then(() => {
        alert(`Reconciled '${currentDrawerId}'.`);
        loadResources();
      });
    }
    function addDomainFromDrawer() {
      closeDrawer();
      switchTab('ingress');
      openAddIngressModal();
      document.getElementById('new-ingress-target').value = currentDrawerId;
    }

    // Build Studio
    async function runBuildPlan() {
      const source = document.getElementById('build-source').value.trim();
      const planBox = document.getElementById('build-plan-output');
      if (!source) return alert('Source path or Git URL required.');
      planBox.innerHTML = 'Cloning and analyzing source...';
      try {
        const res = await apiFetch('/api/build/plan', {
          method: 'POST',
          body: JSON.stringify({ source })
        });
        if (res.ok) {
          const plan = await res.json();
          const provs = (plan.providers || []).join(', ') || 'node';
          const cmd = (plan.start && plan.start.cmd) || 'npm run start';
          planBox.innerHTML = `
            <div><strong>Providers:</strong> <span class="tag tag--green">${escapeHtml(provs)}</span></div>
            <div style="margin: 4px 0;"><strong>Start Command:</strong> <code style="color:var(--green-bright);">${escapeHtml(cmd)}</code></div>
            <div><strong>Phases:</strong> <pre style="font-size:10.5px; margin-top:4px; max-height:110px; overflow-y:auto;">${escapeHtml(JSON.stringify(plan.phases || {}, null, 2))}</pre></div>
          `;
        } else {
          const err = await res.json();
          planBox.innerHTML = `<span style="color:var(--red);">${escapeHtml(err.error || res.statusText)}</span>`;
        }
      } catch (e) {
        planBox.innerHTML = `<span style="color:var(--red);">${escapeHtml(e.toString())}</span>`;
      }
    }

    async function runBuildImage() {
      const source = document.getElementById('build-source').value.trim();
      const image_tag = document.getElementById('build-image-tag').value.trim();
      const envsStr = document.getElementById('build-envs').value.trim();
      const termBox = document.getElementById('build-terminal-box');
      const badge = document.getElementById('build-terminal-status');

      if (!source || !image_tag) return alert('Source and Image Tag are required.');

      badge.textContent = 'BUILDING...';
      badge.className = 'tag tag--amber';
      termBox.innerHTML = `<div class="term-line"><span class="term-num">1</span><span>Compiling '${escapeHtml(image_tag)}' with Nixpacks...</span></div>`;

      let env_vars = {};
      if (envsStr) {
        envsStr.split(',').forEach(p => {
          const [k, v] = p.split('=');
          if (k && v) env_vars[k.trim()] = v.trim();
        });
      }

      try {
        const res = await apiFetch('/api/build/nixpacks', {
          method: 'POST',
          body: JSON.stringify({ source, image_tag, env_vars })
        });
        if (res.ok) {
          const out = await res.json();
          termBox.innerHTML = (out.logs || []).map((l, i) => `
            <div class="term-line"><span class="term-num">${i + 1}</span><span>${escapeHtml(l)}</span></div>
          `).join('');
          termBox.scrollTop = termBox.scrollHeight;
          if (out.success) {
            badge.textContent = 'SUCCESS';
            badge.className = 'tag tag--green';
            lastBuiltImage = image_tag;
            const banner = document.getElementById('build-deploy-banner');
            if (banner) {
              banner.style.display = 'flex';
              const readyTag = document.getElementById('ready-image-tag');
              if (readyTag) readyTag.textContent = image_tag;
            }
          } else {
            badge.textContent = 'FAILED';
            badge.className = 'tag tag--red';
          }
        }
      } catch (e) {
        badge.textContent = 'ERROR';
        badge.className = 'tag tag--red';
      }
    }

    // 1-Click Workload Deployment & Lifecycle Bridge
    let lastBuiltImage = '';
    function openDeployFromBuild() {
      const img = lastBuiltImage || (document.getElementById('build-image-tag') ? document.getElementById('build-image-tag').value.trim() : '') || 'app:v1';
      openDeployModalWithImage(img);
    }
    function openNewWorkloadModal() {
      openDeployModalWithImage('');
    }
    function openDeployModalWithImage(img) {
      const fb = document.getElementById('deploy-status-feedback');
      if (fb) fb.style.display = 'none';
      const imgInput = document.getElementById('deploy-image');
      if (imgInput) imgInput.value = img;
      const nameInput = document.getElementById('deploy-name');
      if (nameInput) {
        if (img) {
          const base = img.split(':')[0].replace(/[^a-zA-Z0-9_-]/g, '-');
          nameInput.value = base;
        } else {
          nameInput.value = '';
        }
      }
      document.getElementById('deploy-workload-modal').classList.add('is-open');
    }
    function closeDeployModal() {
      document.getElementById('deploy-workload-modal').classList.remove('is-open');
    }
    async function submitDeployWorkload() {
      const name = document.getElementById('deploy-name').value.trim();
      const image = document.getElementById('deploy-image').value.trim();
      const service_type = document.getElementById('deploy-type').value;
      const port = parseInt(document.getElementById('deploy-port').value, 10) || 8080;
      const domain = document.getElementById('deploy-domain').value.trim();
      const envsStr = document.getElementById('deploy-envs').value.trim();
      const btn = document.getElementById('btn-submit-deploy');
      const fb = document.getElementById('deploy-status-feedback');

      if (!name || !image) {
        alert('Workload Name and Container Image Tag are required.');
        return;
      }

      let env_vars = {};
      if (envsStr) {
        envsStr.split(',').forEach(p => {
          const [k, v] = p.split('=');
          if (k && v) env_vars[k.trim()] = v.trim();
        });
      }

      btn.disabled = true;
      btn.textContent = 'Deploying...';
      fb.style.display = 'block';
      fb.style.color = 'var(--text-muted)';
      fb.textContent = `Instantiating container '${name}' from image '${image}'...`;

      try {
        const res = await apiFetch('/api/workloads/deploy', {
          method: 'POST',
          body: JSON.stringify({ name, image, port, service_type, domain, env_vars })
        });
        const data = await res.json();
        if (res.ok && data.success) {
          fb.style.color = 'var(--green-bright)';
          fb.textContent = `Workload '${name}' deployed successfully! Opening fleet inspector...`;
          setTimeout(() => {
            closeDeployModal();
            btn.disabled = false;
            btn.textContent = '🚀 Deploy Container';
            switchTab('resources');
            openDrawerForWorkload(name);
          }, 800);
        } else {
          fb.style.color = 'var(--red)';
          fb.textContent = `Deployment error: ${data.error || data.message || 'Failed'}`;
          btn.disabled = false;
          btn.textContent = '🚀 Deploy Container';
        }
      } catch (err) {
        fb.style.color = 'var(--red)';
        fb.textContent = `Request failed: ${err.toString()}`;
        btn.disabled = false;
        btn.textContent = '🚀 Deploy Container';
      }
    }

    async function restartWorkload(id) {
      if (!confirm(`Restart container '${id}'?`)) return;
      const res = await apiFetch(`/api/workloads/${encodeURIComponent(id)}/restart`, { method: 'POST' });
      if (res.ok) {
        alert(`Workload '${id}' restarted.`);
        loadResources();
        if (currentDrawerId === id) refreshDrawerLogs();
      } else {
        const err = await res.json();
        alert(`Restart failed: ${err.message || err.error}`);
      }
    }
    async function deleteWorkload(id) {
      if (!confirm(`Permanently destroy and remove workload '${id}'? This will stop and remove the Docker container.`)) return;
      const res = await apiFetch(`/api/workloads/${encodeURIComponent(id)}`, { method: 'DELETE' });
      if (res.ok) {
        alert(`Workload '${id}' destroyed and removed.`);
        if (currentDrawerId === id) closeDrawer();
        loadResources();
        loadOverview();
      } else {
        const err = await res.json();
        alert(`Delete failed: ${err.message || err.error}`);
      }
    }
    function restartWorkloadFromDrawer() {
      if (currentDrawerId) restartWorkload(currentDrawerId);
    }
    function deleteWorkloadFromDrawer() {
      if (currentDrawerId) deleteWorkload(currentDrawerId);
    }

    // Ingress
    async function loadIngress() {
      const tbody = document.getElementById('ingress-routes-table-body');
      try {
        const res = await apiFetch('/api/ingress');
        if (res.ok) {
          const data = await res.json();
          const routes = data.routes || [];
          if (routes.length === 0) {
            tbody.innerHTML = '<tr><td colspan="6" style="text-align:center; color:var(--text-dim); padding:20px;">No ingress routes registered yet.</td></tr>';
            return;
          }
          tbody.innerHTML = routes.map(r => `
            <tr>
              <td><a href="https://${escapeHtml(r.domain)}" target="_blank" style="color:var(--blue); text-decoration:none;">${escapeHtml(r.domain)} ↗</a></td>
              <td style="color:var(--text-bright);">${escapeHtml(r.target_resource)}</td>
              <td>${r.target_port}</td>
              <td>${r.tls ? '<span class="tag tag--green">ACME TLS</span>' : '<span class="tag">HTTP</span>'}</td>
              <td>${escapeHtml(r.path_prefix || '/')}</td>
              <td><button class="btn btn--danger" style="height:22px; font-size:10px;" onclick="deleteIngressRoute('${escapeHtml(r.domain)}')">Delete</button></td>
            </tr>
          `).join('');
        }
      } catch (e) {
        tbody.innerHTML = `<tr><td colspan="6" style="color:var(--red);">Failed to load routes: ${escapeHtml(e.toString())}</td></tr>`;
      }
    }

    function openAddIngressModal() { document.getElementById('add-ingress-modal').classList.add('is-open'); }
    function closeAddIngressModal() { document.getElementById('add-ingress-modal').classList.remove('is-open'); }
    async function submitAddIngress() {
      const domain = document.getElementById('new-ingress-domain').value.trim();
      const target_resource = document.getElementById('new-ingress-target').value.trim();
      const target_port = parseInt(document.getElementById('new-ingress-port').value, 10);
      if (!domain || !target_resource || !target_port) return alert('Fields required.');
      const res = await apiFetch('/api/ingress', {
        method: 'POST',
        body: JSON.stringify({ domain, target_resource, target_port, tls: true })
      });
      if (res.ok) { closeAddIngressModal(); loadIngress(); }
    }
    async function deleteIngressRoute(domain) {
      if (confirm(`Remove route for '${domain}'?`)) {
        await apiFetch(`/api/ingress/${encodeURIComponent(domain)}`, { method: 'DELETE' });
        loadIngress();
      }
    }

    function openExportConfigModal() { document.getElementById('export-config-modal').classList.add('is-open'); loadExportSnippet('caddy'); }
    function closeExportConfigModal() { document.getElementById('export-config-modal').classList.remove('is-open'); }
    async function loadExportSnippet(fmt) {
      const box = document.getElementById('export-snippet-box');
      box.textContent = `Generating ${fmt}...`;
      const res = await apiFetch(`/api/ingress/export/${fmt}`);
      if (res.ok) box.textContent = await res.text();
    }

    // Agents
    async function loadAgents() {
      const tbody = document.getElementById('agents-table-body');
      const res = await apiFetch('/api/agents');
      if (res.ok) {
        const d = await res.json();
        const agents = d.agents || [];
        if (agents.length === 0) {
          tbody.innerHTML = '<tr><td colspan="7" style="text-align:center; color:var(--text-dim); padding:20px;">No paired agents.</td></tr>';
          return;
        }
        tbody.innerHTML = agents.map(a => `
          <tr>
            <td style="color:var(--text-bright); font-weight:500;">${escapeHtml(a.name)}</td>
            <td>${escapeHtml(a.workspace_id)}</td>
            <td>${(a.scopes || []).map(s => `<span class="tag tag--blue">${escapeHtml(s)}</span>`).join(' ')}</td>
            <td><span class="tag ${a.revoked ? 'tag--red' : 'tag--green'}">${a.revoked ? 'REVOKED' : 'ACTIVE'}</span></td>
            <td>${escapeHtml(a.token_prefix)}...</td>
            <td>${a.last_used_at ? new Date(a.last_used_at).toLocaleTimeString() : 'Never'}</td>
            <td>${!a.revoked ? `<button class="btn btn--danger" style="height:22px; font-size:10px;" onclick="revokeAgent('${escapeHtml(a.agent_id)}')">Revoke</button>` : 'Revoked'}</td>
          </tr>
        `).join('');
      }
    }
    function openPairModal() {
      document.getElementById('token-reveal-area').style.display = 'none';
      document.getElementById('btn-submit-pair').style.display = 'inline-flex';
      document.getElementById('pair-modal').classList.add('is-open');
    }
    function closePairModal() {
      document.getElementById('pair-modal').classList.remove('is-open');
      loadAgents();
    }
    async function submitPairAgent() {
      const name = document.getElementById('new-agent-name').value.trim();
      const workspace_id = document.getElementById('new-agent-workspace').value.trim() || 'default';
      const scopes = document.getElementById('new-agent-scopes').value.trim();
      if (!name) return alert('Name required.');
      const res = await apiFetch('/api/agents/pair', {
        method: 'POST',
        body: JSON.stringify({ name, workspace_id, scopes })
      });
      if (res.ok) {
        const d = await res.json();
        document.getElementById('token-reveal-area').style.display = 'block';
        document.getElementById('revealed-token-text').textContent = d.token;
        document.getElementById('btn-submit-pair').style.display = 'none';
      }
    }
    async function revokeAgent(id) {
      if (confirm(`Revoke credentials for '${id}'?`)) {
        await apiFetch(`/api/agents/${id}/revoke`, { method: 'POST' });
        loadAgents();
      }
    }

    // Operations
    async function loadOperations() {
      const tbody = document.getElementById('operations-table-body');
      const res = await apiFetch('/api/operations');
      if (res.ok) {
        const d = await res.json();
        const ops = d.operations || [];
        if (ops.length === 0) {
          tbody.innerHTML = '<tr><td colspan="7" style="text-align:center; color:var(--text-dim); padding:20px;">No operations recorded.</td></tr>';
          return;
        }
        tbody.innerHTML = ops.map(op => `
          <tr>
            <td style="color:var(--text-bright);">${escapeHtml(op.id)}</td>
            <td>${escapeHtml(op.workspace_id)}</td>
            <td><span class="tag tag--green">${escapeHtml((op.status || '').toUpperCase())}</span></td>
            <td>${op.progress_percent || 0}%</td>
            <td>${op.lease_epoch}</td>
            <td>${escapeHtml((op.plan_hash || '').substring(0, 12))}...</td>
            <td><button class="btn btn--secondary" style="height:22px; font-size:10px;" onclick="viewOpLogs('${escapeHtml(op.id)}')">View</button></td>
          </tr>
        `).join('');
      }
    }
    function viewOpLogs(id) {
      apiFetch(`/api/operations/${id}`).then(r => r.json()).then(op => {
        alert(`Operation: ${op.id}\nStatus: ${op.status}\nEpoch: ${op.lease_epoch}\nSteps: ${(op.logs || []).length}`);
      });
    }

    // Diagnostics
    async function runDoctor() {
      const box = document.getElementById('doctor-results-container');
      box.textContent = 'Running platform diagnostics...';
      const res = await apiFetch('/api/doctor');
      if (res.ok) {
        const d = await res.json();
        box.innerHTML = `
          <div style="margin-bottom:8px;"><strong>Status:</strong> <span class="tag tag--green">${d.overall_status.toUpperCase()}</span> (${d.os} ${d.arch})</div>
          <div>${(d.checks || []).map(c => `
            <div style="padding:4px 0; border-bottom:0.4px dashed var(--hairline);">
              <span class="tag ${c.status === 'pass' ? 'tag--green' : 'tag--amber'}">${c.status.toUpperCase()}</span>
              <strong>${escapeHtml(c.name)}:</strong> ${escapeHtml(c.message)}
            </div>
          `).join('')}</div>
        `;
      }
    }

    // Quickstart
    function updateQuickstartInstructions() {
      const agent = document.getElementById('quickstart-agent').value;
      const host = document.getElementById('quickstart-host').value.trim() || 'http://169.58.72.243:4040';
      let cfg = '';
      if (agent === 'claude-code') {
        cfg = `# Pair on server:\nsmoooth agent pair --name claude-code --scopes read_only,plan,deploy\n\n# Connect Claude Code:\nclaude mcp add --transport sse smoooth ${host}/sse -H "Authorization: Bearer <ISSUED_TOKEN>"`;
      } else if (agent === 'cursor') {
        cfg = `// Add to .cursor/mcp.json:\n{\n  "mcpServers": {\n    "smoooth": {\n      "url": "${host}/sse",\n      "headers": { "Authorization": "Bearer <ISSUED_TOKEN>" }\n    }\n  }\n}`;
      } else {
        cfg = `// Add to windsurf mcp_config.json:\n{\n  "mcpServers": {\n    "smoooth": {\n      "serverUrl": "${host}/sse",\n      "headers": { "Authorization": "Bearer <ISSUED_TOKEN>" }\n    }\n  }\n}`;
      }
      document.getElementById('quickstart-config-box').textContent = cfg;
      document.getElementById('quickstart-prompt-box').textContent = `Inspect our infrastructure using Smoooth list_capabilities and list_resources. Propose a deployment plan and do not make changes without asking first.`;
    }

    function filterFleet(val) {
      const rows = document.querySelectorAll('#resources-table-body tr');
      rows.forEach(r => {
        if (!val || r.textContent.toLowerCase().includes(val.toLowerCase())) {
          r.style.display = '';
        } else {
          r.style.display = 'none';
        }
      });
    }

    // Escape Key closes drawer
    window.addEventListener('keydown', e => {
      if (e.key === 'Escape') closeDrawer();
    });

    // Boot
    window.addEventListener('DOMContentLoaded', () => {
      const stored = getAuthToken();
      if (stored) document.getElementById('auth-token').value = stored;
      loadOverview();
    });
  </script>
</body>
</html>
"###;
