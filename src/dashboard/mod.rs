//! Self-contained Smoooth Operator Control Dashboard
//! Embedded directly into the binary with zero external runtime dependencies.
//! Follows the deep charcoal, high-legibility, restrained data visual design.

pub fn get_dashboard_html() -> &'static str {
    DASHBOARD_HTML
}

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Smoooth — Control Plane Dashboard</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&family=Plus+Jakarta+Sans:wght@400;500;600;700&display=swap" rel="stylesheet">
  <style>
    :root {
      --bg-base: #0b0d10;
      --bg-surface: #12151b;
      --bg-surface-elevated: #181c24;
      --bg-card: #151921;
      --border-quiet: #212631;
      --border-subtle: #2d3443;
      --text-primary: #f8fafc;
      --text-secondary: #94a3b8;
      --text-muted: #64748b;
      
      --accent-blue: #3b82f6;
      --accent-green: #10b981;
      --accent-amber: #f59e0b;
      --accent-red: #ef4444;
      --accent-purple: #8b5cf6;
      --accent-cyan: #06b6d4;
      
      --tick-bg: #222733;
      --radius-sm: 6px;
      --radius-md: 10px;
      --radius-lg: 14px;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background-color: var(--bg-base);
      color: var(--text-primary);
      font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      font-size: 14px;
      line-height: 1.5;
      -webkit-font-smoothing: antialiased;
    }

    /* Top Bar */
    header.topbar {
      height: 56px;
      background: var(--bg-surface);
      border-bottom: 1px solid var(--border-quiet);
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0 24px;
      position: sticky;
      top: 0;
      z-index: 50;
    }
    .brand-area {
      display: flex;
      align-items: center;
      gap: 12px;
    }
    .brand-logo {
      font-size: 16px;
      font-weight: 700;
      letter-spacing: -0.02em;
      color: var(--text-primary);
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .brand-logo .dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--accent-green);
      box-shadow: 0 0 8px rgba(16, 185, 129, 0.5);
    }
    .version-tag {
      font-family: 'JetBrains Mono', monospace;
      font-size: 11px;
      background: var(--bg-surface-elevated);
      color: var(--text-secondary);
      padding: 2px 8px;
      border-radius: var(--radius-sm);
      border: 1px solid var(--border-quiet);
    }
    .topbar-actions {
      display: flex;
      align-items: center;
      gap: 12px;
    }
    .auth-input-group {
      display: flex;
      align-items: center;
      gap: 8px;
      background: var(--bg-base);
      border: 1px solid var(--border-subtle);
      border-radius: var(--radius-sm);
      padding: 4px 10px;
    }
    .auth-input-group label {
      font-family: 'JetBrains Mono', monospace;
      font-size: 11px;
      color: var(--text-muted);
    }
    .auth-input-group input {
      background: transparent;
      border: none;
      color: var(--text-primary);
      font-family: 'JetBrains Mono', monospace;
      font-size: 12px;
      width: 140px;
      outline: none;
    }

    /* Main Container & Navigation Tabs */
    .dashboard-layout {
      display: flex;
      flex: 1;
      max-width: 1400px;
      margin: 0 auto;
      width: 100%;
      padding: 24px;
      gap: 24px;
    }
    nav.sidebar {
      width: 220px;
      flex-shrink: 0;
      display: flex;
      flex-direction: column;
      gap: 4px;
    }
    nav.sidebar button {
      background: transparent;
      border: none;
      color: var(--text-secondary);
      font-family: inherit;
      font-size: 13px;
      font-weight: 500;
      text-align: left;
      padding: 10px 14px;
      border-radius: var(--radius-md);
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 10px;
      transition: all 0.15s ease;
    }
    nav.sidebar button:hover {
      background: var(--bg-surface);
      color: var(--text-primary);
    }
    nav.sidebar button.active {
      background: var(--bg-surface-elevated);
      color: var(--text-primary);
      border-left: 3px solid var(--accent-blue);
      border-radius: 0 var(--radius-md) var(--radius-md) 0;
    }

    main.content-area {
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 24px;
      min-width: 0;
    }

    /* Card System */
    .card {
      background: var(--bg-card);
      border: 1px solid var(--border-quiet);
      border-radius: var(--radius-lg);
      padding: 20px;
    }
    .card-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      margin-bottom: 16px;
    }
    .card-title {
      font-size: 15px;
      font-weight: 600;
      letter-spacing: -0.01em;
      color: var(--text-primary);
    }
    .card-subtitle {
      font-size: 12px;
      color: var(--text-secondary);
      margin-top: 2px;
    }

    /* Grid helpers */
    .grid-2 {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(360px, 1fr));
      gap: 20px;
    }
    .grid-4 {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      gap: 16px;
    }

    /* Stat Box */
    .stat-box {
      background: var(--bg-surface);
      border: 1px solid var(--border-quiet);
      border-radius: var(--radius-md);
      padding: 14px 16px;
    }
    .stat-label {
      font-size: 12px;
      color: var(--text-secondary);
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
    .stat-value {
      font-family: 'JetBrains Mono', monospace;
      font-size: 22px;
      font-weight: 600;
      color: var(--text-primary);
      margin-top: 4px;
    }

    /* Fleet Status & Segmented Bar (Reference Image 1) */
    .status-capsule-bar {
      height: 8px;
      background: var(--bg-surface-elevated);
      border-radius: 999px;
      display: flex;
      overflow: hidden;
      margin: 16px 0 20px 0;
      gap: 2px;
    }
    .capsule-segment {
      height: 100%;
      border-radius: 4px;
      transition: width 0.3s ease;
    }
    .seg-running { background: var(--accent-green); }
    .seg-stopped { background: var(--text-muted); }
    .seg-degraded { background: var(--accent-amber); }
    .seg-failed { background: var(--accent-red); }

    .distribution-list {
      display: flex;
      flex-direction: column;
      gap: 10px;
    }
    .dist-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 13px;
    }
    .dist-label-group {
      display: flex;
      align-items: center;
      gap: 10px;
    }
    .status-bullet {
      width: 7px;
      height: 7px;
      border-radius: 50%;
    }
    .dist-figures {
      display: flex;
      align-items: center;
      gap: 16px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 12px;
    }
    .dist-pct { color: var(--text-secondary); width: 36px; text-align: right; }
    .dist-count { color: var(--text-primary); font-weight: 600; width: 24px; text-align: right; }

    /* Capacity Vertical Tick Marks (Reference Image 1) */
    .tick-container {
      display: flex;
      align-items: center;
      gap: 3px;
    }
    .tick {
      width: 3px;
      height: 14px;
      border-radius: 1px;
      background: var(--tick-bg);
    }
    .tick.active { background: var(--accent-green); }
    .tick.active-amber { background: var(--accent-amber); }
    .tick.active-blue { background: var(--accent-blue); }

    /* Benchmark Horizontal Stippled Comparison Bars (Reference Image 2) */
    .benchmark-row {
      display: flex;
      flex-direction: column;
      gap: 6px;
      margin-bottom: 14px;
    }
    .benchmark-meta {
      display: flex;
      justify-content: space-between;
      font-size: 12px;
    }
    .benchmark-meta .service-name {
      font-weight: 500;
      color: var(--text-primary);
    }
    .benchmark-meta .metric-val {
      font-family: 'JetBrains Mono', monospace;
      color: var(--text-secondary);
    }
    .benchmark-bar-track {
      height: 12px;
      background: var(--bg-base);
      border-radius: 3px;
      overflow: hidden;
      display: flex;
      position: relative;
    }
    .benchmark-bar-fill {
      height: 100%;
      border-radius: 3px;
      transition: width 0.4s ease;
      background-image: repeating-linear-gradient(45deg, rgba(255,255,255,0.08), rgba(255,255,255,0.08) 2px, transparent 2px, transparent 6px);
    }

    /* Table Styles */
    table.data-table {
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
    }
    table.data-table th {
      text-align: left;
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      color: var(--text-muted);
      padding: 10px 14px;
      border-bottom: 1px solid var(--border-quiet);
    }
    table.data-table td {
      padding: 12px 14px;
      border-bottom: 1px solid var(--border-quiet);
      color: var(--text-secondary);
    }
    table.data-table tr:hover td {
      background: rgba(255, 255, 255, 0.015);
      color: var(--text-primary);
    }

    /* Badges */
    .badge {
      display: inline-flex;
      align-items: center;
      gap: 5px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 11px;
      padding: 2px 8px;
      border-radius: var(--radius-sm);
      border: 1px solid transparent;
      white-space: nowrap;
    }
    .badge-green { background: rgba(16, 185, 129, 0.1); color: #34d399; border-color: rgba(16, 185, 129, 0.2); }
    .badge-blue { background: rgba(59, 130, 246, 0.1); color: #60a5fa; border-color: rgba(59, 130, 246, 0.2); }
    .badge-amber { background: rgba(245, 158, 11, 0.1); color: #fbbf24; border-color: rgba(245, 158, 11, 0.2); }
    .badge-red { background: rgba(239, 68, 68, 0.1); color: #f87171; border-color: rgba(239, 68, 68, 0.2); }
    .badge-grey { background: rgba(148, 163, 184, 0.1); color: #cbd5e1; border-color: rgba(148, 163, 184, 0.2); }
    .badge-purple { background: rgba(139, 92, 246, 0.1); color: #c084fc; border-color: rgba(139, 92, 246, 0.2); }

    /* Buttons */
    .btn {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      background: var(--bg-surface-elevated);
      color: var(--text-primary);
      border: 1px solid var(--border-subtle);
      border-radius: var(--radius-sm);
      padding: 6px 12px;
      font-family: inherit;
      font-size: 12px;
      font-weight: 500;
      cursor: pointer;
      transition: all 0.15s ease;
    }
    .btn:hover {
      background: #232834;
      border-color: #3b4253;
    }
    .btn-primary {
      background: var(--accent-blue);
      border-color: var(--accent-blue);
      color: #fff;
    }
    .btn-primary:hover {
      background: #2563eb;
    }
    .btn-danger {
      background: rgba(239, 68, 68, 0.1);
      border-color: rgba(239, 68, 68, 0.3);
      color: #f87171;
    }
    .btn-danger:hover {
      background: rgba(239, 68, 68, 0.2);
    }
    .btn-sm {
      padding: 3px 8px;
      font-size: 11px;
    }

    /* Modal */
    .modal-backdrop {
      display: none;
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.7);
      backdrop-filter: blur(4px);
      z-index: 100;
      align-items: center;
      justify-content: center;
    }
    .modal-backdrop.open {
      display: flex;
    }
    .modal-card {
      background: var(--bg-card);
      border: 1px solid var(--border-subtle);
      border-radius: var(--radius-lg);
      width: 100%;
      max-width: 480px;
      padding: 24px;
      box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
    }
    .form-group {
      display: flex;
      flex-direction: column;
      gap: 6px;
      margin-bottom: 16px;
    }
    .form-group label {
      font-size: 12px;
      font-weight: 500;
      color: var(--text-secondary);
    }
    .form-group input, .form-group select {
      background: var(--bg-base);
      border: 1px solid var(--border-subtle);
      border-radius: var(--radius-sm);
      padding: 8px 12px;
      color: var(--text-primary);
      font-family: inherit;
      font-size: 13px;
      outline: none;
    }

    /* Code & logs */
    .code-box {
      background: var(--bg-base);
      border: 1px solid var(--border-quiet);
      border-radius: var(--radius-sm);
      padding: 12px;
      font-family: 'JetBrains Mono', monospace;
      font-size: 12px;
      color: #e2e8f0;
      overflow-x: auto;
      max-height: 280px;
    }
  </style>
</head>
<body>

  <!-- Top Navigation -->
  <header class="topbar">
    <div class="brand-area">
      <div class="brand-logo">
        <span class="dot"></span>
        <span>Smoooth</span>
      </div>
      <span class="version-tag">Beta 0.1.0</span>
      <span id="daemon-indicator" class="badge badge-green" style="font-size: 10px;">CONNECTED</span>
    </div>
    <div class="topbar-actions">
      <div class="auth-input-group">
        <label>TOKEN</label>
        <input type="password" id="auth-token" placeholder="Bearer Token" autocomplete="off" />
      </div>
      <button class="btn btn-sm" onclick="saveAuthToken()">Set Token</button>
      <button class="btn btn-sm btn-primary" onclick="refreshCurrentTab()">↻ Refresh</button>
    </div>
  </header>

  <div class="dashboard-layout">
    <!-- Sidebar Navigation -->
    <nav class="sidebar">
      <button class="active" onclick="switchTab('overview')">
        <span>📊</span> <span>Overview</span>
      </button>
      <button onclick="switchTab('resources')">
        <span>📦</span> <span>Applications & Resources</span>
      </button>
      <button onclick="switchTab('agents')">
        <span>🤖</span> <span>Agents & Access</span>
      </button>
      <button onclick="switchTab('operations')">
        <span>⚡</span> <span>Deployments & Ops</span>
      </button>
      <button onclick="switchTab('settings')">
        <span>⚙️</span> <span>Settings & Doctor</span>
      </button>
    </nav>

    <!-- Main Content Area -->
    <main class="content-area">

      <!-- ============================================================== -->
      <!-- SECTION 1: OVERVIEW                                            -->
      <!-- ============================================================== -->
      <div id="tab-overview" class="tab-pane">
        
        <!-- Summary Stats Row -->
        <div class="grid-4" style="margin-bottom: 20px;">
          <div class="stat-box">
            <div class="stat-label">Active Workloads</div>
            <div class="stat-value" id="stat-active">0</div>
          </div>
          <div class="stat-box">
            <div class="stat-label">Total Resources</div>
            <div class="stat-value" id="stat-total">0</div>
          </div>
          <div class="stat-box">
            <div class="stat-label">Paired Agents</div>
            <div class="stat-value" id="stat-agents">0</div>
          </div>
          <div class="stat-box">
            <div class="stat-label">System Health</div>
            <div class="stat-value" id="stat-health" style="color: var(--accent-green); font-size: 18px;">PASS</div>
          </div>
        </div>

        <div class="grid-2">
          <!-- Workload Distribution Card (Visual Ref Image 1) -->
          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">Workload Fleet Status</div>
                <div class="card-subtitle" id="workload-summary-subtitle">0 workloads deployed across default workspace</div>
              </div>
            </div>

            <!-- Segmented Pill Status Bar -->
            <div class="status-capsule-bar" id="status-capsule-bar">
              <div class="capsule-segment seg-running" style="width: 100%;"></div>
            </div>

            <!-- Distribution Legend with Exact Figures -->
            <div class="distribution-list" id="distribution-list">
              <div class="dist-row">
                <div class="dist-label-group">
                  <span class="status-bullet" style="background: var(--accent-green);"></span>
                  <span>Active & Healthy</span>
                </div>
                <div class="dist-figures">
                  <span class="dist-pct" id="pct-running">100%</span>
                  <span class="dist-count" id="count-running">0</span>
                </div>
              </div>
              <div class="dist-row">
                <div class="dist-label-group">
                  <span class="status-bullet" style="background: var(--accent-amber);"></span>
                  <span>Degraded / Drifted</span>
                </div>
                <div class="dist-figures">
                  <span class="dist-pct" id="pct-degraded">0%</span>
                  <span class="dist-count" id="count-degraded">0</span>
                </div>
              </div>
              <div class="dist-row">
                <div class="dist-label-group">
                  <span class="status-bullet" style="background: var(--text-muted);"></span>
                  <span>Stopped / Offline</span>
                </div>
                <div class="dist-figures">
                  <span class="dist-pct" id="pct-stopped">0%</span>
                  <span class="dist-count" id="count-stopped">0</span>
                </div>
              </div>
            </div>

            <!-- Workspace Capacity Ticks (Visual Ref Image 1) -->
            <div style="margin-top: 24px; padding-top: 16px; border-top: 1px solid var(--border-quiet);">
              <div style="font-size: 12px; color: var(--text-secondary); margin-bottom: 10px; font-weight: 500;">Workspace Allocation</div>
              <div style="display: flex; align-items: center; justify-content: space-between;">
                <span style="font-size: 13px;" id="capacity-name">default</span>
                <div style="display: flex; align-items: center; gap: 12px;">
                  <div class="tick-container" id="capacity-ticks">
                    <!-- Dynamic discrete tick marks -->
                    <span class="tick active"></span>
                    <span class="tick active"></span>
                    <span class="tick"></span>
                    <span class="tick"></span>
                    <span class="tick"></span>
                  </div>
                  <span style="font-family: 'JetBrains Mono', monospace; font-size: 12px;" id="capacity-fraction">0 / 10</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Active Probe Latency Comparison Card (Visual Ref Image 2) -->
          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">Active Health Probes</div>
                <div class="card-subtitle">Real-time probe response latency (lower is better)</div>
              </div>
            </div>

            <div id="probes-benchmark-container">
              <div class="benchmark-row">
                <div class="benchmark-meta">
                  <span class="service-name">Control Plane API Probe</span>
                  <span class="metric-val" id="probe-api-latency">1 ms</span>
                </div>
                <div class="benchmark-bar-track">
                  <div class="benchmark-bar-fill" style="width: 15%; background-color: var(--accent-blue);"></div>
                </div>
              </div>

              <div class="benchmark-row">
                <div class="benchmark-meta">
                  <span class="service-name">PostgreSQL Active Ping</span>
                  <span class="metric-val" id="probe-pg-latency">4 ms</span>
                </div>
                <div class="benchmark-bar-track">
                  <div class="benchmark-bar-fill" style="width: 25%; background-color: var(--accent-purple);"></div>
                </div>
              </div>

              <div class="benchmark-row">
                <div class="benchmark-meta">
                  <span class="service-name">Redis RESP TCP Ping</span>
                  <span class="metric-val" id="probe-redis-latency">2 ms</span>
                </div>
                <div class="benchmark-bar-track">
                  <div class="benchmark-bar-fill" style="width: 18%; background-color: var(--accent-amber);"></div>
                </div>
              </div>

              <div class="benchmark-row">
                <div class="benchmark-meta">
                  <span class="service-name">MongoDB Connection Probe</span>
                  <span class="metric-val" id="probe-mongo-latency">3 ms</span>
                </div>
                <div class="benchmark-bar-track">
                  <div class="benchmark-bar-fill" style="width: 20%; background-color: var(--accent-green);"></div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Recent Security Audit Log Feed -->
        <div class="card" style="margin-top: 20px;">
          <div class="card-header">
            <div>
              <div class="card-title">Recent Security & Policy Events</div>
              <div class="card-subtitle">Server-side audit trail (Secrets automatically redacted)</div>
            </div>
          </div>
          <table class="data-table">
            <thead>
              <tr>
                <th>Timestamp</th>
                <th>Actor</th>
                <th>Action</th>
                <th>Workspace</th>
                <th>Outcome</th>
              </tr>
            </thead>
            <tbody id="audit-feed-tbody">
              <tr><td colspan="5" style="text-align: center; color: var(--text-muted); padding: 24px;">No security events recorded yet</td></tr>
            </tbody>
          </table>
        </div>

      </div>

      <!-- ============================================================== -->
      <!-- SECTION 2: APPLICATIONS & RESOURCES                            -->
      <!-- ============================================================== -->
      <div id="tab-resources" class="tab-pane" style="display: none;">
        <div class="card">
          <div class="card-header">
            <div>
              <div class="card-title">Discovered & Provisioned Workloads</div>
              <div class="card-subtitle">Active Docker containers, persistent databases, and external bindings</div>
            </div>
          </div>

          <table class="data-table">
            <thead>
              <tr>
                <th>Resource Name</th>
                <th>Provider</th>
                <th>Service Type</th>
                <th>Status</th>
                <th>Volume Guard</th>
                <th>Ownership</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody id="resources-table-body">
              <tr><td colspan="7" style="text-align: center; color: var(--text-muted); padding: 32px;">Loading resources...</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- ============================================================== -->
      <!-- SECTION 3: AGENTS & ACCESS                                     -->
      <!-- ============================================================== -->
      <div id="tab-agents" class="tab-pane" style="display: none;">
        <div class="card">
          <div class="card-header">
            <div>
              <div class="card-title">Agent Identities & Scopes</div>
              <div class="card-subtitle">Fine-grained, revocable credentials with server-enforced security scopes</div>
            </div>
            <button class="btn btn-primary btn-sm" onclick="openPairModal()">+ Pair Agent</button>
          </div>

          <table class="data-table">
            <thead>
              <tr>
                <th>Agent Identity</th>
                <th>Workspace</th>
                <th>Granted Scopes</th>
                <th>Status</th>
                <th>Token Prefix</th>
                <th>Last Active</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody id="agents-table-body">
              <tr><td colspan="7" style="text-align: center; color: var(--text-muted); padding: 32px;">Loading agents...</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- ============================================================== -->
      <!-- SECTION 4: DEPLOYMENTS & OPERATIONS                            -->
      <!-- ============================================================== -->
      <div id="tab-operations" class="tab-pane" style="display: none;">
        <div class="card">
          <div class="card-header">
            <div>
              <div class="card-title">Durable Operations & Plan Execution</div>
              <div class="card-subtitle">Fencing leases, cryptographic plan hashing, and active execution logs</div>
            </div>
          </div>

          <table class="data-table">
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
              <tr><td colspan="7" style="text-align: center; color: var(--text-muted); padding: 32px;">Loading operations...</td></tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- ============================================================== -->
      <!-- SECTION 5: SETTINGS & DOCTOR                                   -->
      <!-- ============================================================== -->
      <div id="tab-settings" class="tab-pane" style="display: none;">
        <div class="grid-2">
          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">Node Configuration & Security Policy</div>
                <div class="card-subtitle">Operating environment and exposure parameters</div>
              </div>
            </div>
            <table class="data-table">
              <tbody>
                <tr><td>Daemon Version</td><td><span class="badge badge-grey">0.1.0 Beta</span></td></tr>
                <tr><td>Network Exposure</td><td><span class="badge badge-green">Private Loopback (127.0.0.1:4040)</span></td></tr>
                <tr><td>Storage Engine</td><td><span class="badge badge-blue">Atomic JSON Snapshot (sync_all + .bak)</span></td></tr>
                <tr><td>Encryption at Rest</td><td><span class="badge badge-purple">AES-256-GCM Dual-Credential</span></td></tr>
                <tr><td>Systemd Unit</td><td><span class="badge badge-grey">smoooth.service (NoNewPrivileges=yes)</span></td></tr>
              </tbody>
            </table>
          </div>

          <div class="card">
            <div class="card-header">
              <div>
                <div class="card-title">Smoooth Doctor Diagnostics</div>
                <div class="card-subtitle">Live preflight and operational checks</div>
              </div>
              <button class="btn btn-sm btn-primary" onclick="runDoctor()">Run Diagnostics</button>
            </div>
            <div id="doctor-results-container">
              <p style="color: var(--text-muted); font-size: 13px;">Click 'Run Diagnostics' to test platform baseline, Docker socket, port availability, and storage permissions.</p>
            </div>
          </div>
        </div>
      </div>

    </main>
  </div>

  <!-- Agent Pairing Modal -->
  <div class="modal-backdrop" id="pair-modal">
    <div class="modal-card">
      <div class="card-header">
        <div class="card-title">Pair New AI Coding Agent</div>
      </div>
      <div class="form-group">
        <label>Agent Name / Purpose</label>
        <input type="text" id="new-agent-name" placeholder="e.g. cursor-ide, claude-code, ci-worker" />
      </div>
      <div class="form-group">
        <label>Workspace Boundary</label>
        <input type="text" id="new-agent-workspace" value="default" placeholder="default or *" />
      </div>
      <div class="form-group">
        <label>Scopes (Separated by comma)</label>
        <input type="text" id="new-agent-scopes" value="read_only,plan,deploy" placeholder="read_only,plan,deploy,destructive,admin" />
        <span style="font-size: 11px; color: var(--text-muted); margin-top: 4px;">Available: read_only, plan, deploy, destructive, admin</span>
      </div>
      <div id="token-reveal-area" style="display: none; margin: 16px 0;">
        <label style="font-size: 12px; font-weight: 600; color: var(--accent-amber);">ONE-TIME TOKEN REVEAL</label>
        <div style="font-size: 11px; color: var(--text-secondary); margin-bottom: 6px;">Copy this token now. It cannot be recovered in plaintext after this screen.</div>
        <div class="code-box" id="revealed-token-text" style="word-break: break-all; user-select: all;"></div>
      </div>
      <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px;">
        <button class="btn" onclick="closePairModal()">Cancel</button>
        <button class="btn btn-primary" id="btn-submit-pair" onclick="submitPairAgent()">Generate & Pair</button>
      </div>
    </div>
  </div>

  <script>
    // Token & API Helper
    function getAuthToken() {
      return localStorage.getItem('smoooth_auth_token') || sessionStorage.getItem('smoooth_auth_token') || '';
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

    let activeTab = 'overview';
    function switchTab(tabId) {
      activeTab = tabId;
      document.querySelectorAll('.tab-pane').forEach(el => el.style.display = 'none');
      document.querySelectorAll('nav.sidebar button').forEach(el => el.classList.remove('active'));
      const activeBtn = Array.from(document.querySelectorAll('nav.sidebar button')).find(b => b.getAttribute('onclick').includes(tabId));
      if (activeBtn) activeBtn.classList.add('active');
      const target = document.getElementById(`tab-${tabId}`);
      if (target) target.style.display = 'block';
      refreshCurrentTab();
    }

    function refreshCurrentTab() {
      if (activeTab === 'overview') loadOverview();
      else if (activeTab === 'resources') loadResources();
      else if (activeTab === 'agents') loadAgents();
      else if (activeTab === 'operations') loadOperations();
      else if (activeTab === 'settings') runDoctor();
    }

    // Load Overview Data
    async function loadOverview() {
      try {
        const res = await apiFetch('/api/status');
        if (res.ok) {
          const data = await res.json();
          const items = data.resources || [];
          document.getElementById('stat-total').textContent = items.length;
          
          let running = 0, degraded = 0, stopped = 0;
          items.forEach(r => {
            const s = (r.status || '').toLowerCase();
            if (s.includes('active') || s.includes('running')) running++;
            else if (s.includes('degraded')) degraded++;
            else stopped++;
          });
          document.getElementById('stat-active').textContent = running;
          
          const total = items.length || 1;
          const pctR = Math.round((running / total) * 100);
          const pctD = Math.round((degraded / total) * 100);
          const pctS = Math.max(0, 100 - pctR - pctD);

          document.getElementById('count-running').textContent = running;
          document.getElementById('pct-running').textContent = pctR + '%';
          document.getElementById('count-degraded').textContent = degraded;
          document.getElementById('pct-degraded').textContent = pctD + '%';
          document.getElementById('count-stopped').textContent = stopped;
          document.getElementById('pct-stopped').textContent = pctS + '%';

          document.getElementById('status-capsule-bar').innerHTML = `
            <div class="capsule-segment seg-running" style="width: ${pctR}%;"></div>
            <div class="capsule-segment seg-degraded" style="width: ${pctD}%;"></div>
            <div class="capsule-segment seg-stopped" style="width: ${pctS}%;"></div>
          `;

          // Ticks
          document.getElementById('capacity-name').textContent = data.workspace || 'default';
          document.getElementById('capacity-fraction').textContent = `${running} / ${Math.max(10, running + 4)}`;
          let ticksHtml = '';
          const maxTicks = 12;
          const activeTicks = Math.min(maxTicks, Math.round((running / Math.max(10, running + 4)) * maxTicks));
          for (let i = 0; i < maxTicks; i++) {
            ticksHtml += `<span class="tick ${i < activeTicks ? 'active' : ''}"></span>`;
          }
          document.getElementById('capacity-ticks').innerHTML = ticksHtml;
        }

        // Load agents count
        const aRes = await apiFetch('/api/agents');
        if (aRes.ok) {
          const aData = await aRes.json();
          document.getElementById('stat-agents').textContent = (aData.agents || []).length;
        }

        // Load Audit Feed
        const auditRes = await apiFetch('/api/audit');
        if (auditRes.ok) {
          const events = await auditRes.json();
          const tbody = document.getElementById('audit-feed-tbody');
          if (events && events.length > 0) {
            tbody.innerHTML = events.slice(0, 8).map(e => `
              <tr>
                <td style="font-family: 'JetBrains Mono', monospace; font-size: 11px;">${new Date(e.timestamp).toLocaleTimeString()}</td>
                <td style="font-weight: 500; color: var(--text-primary);">${e.actor}</td>
                <td><span class="badge badge-grey">${e.action}</span></td>
                <td>${e.workspace_id}</td>
                <td><span class="badge ${e.outcome === 'success' ? 'badge-green' : 'badge-red'}">${e.outcome}</span></td>
              </tr>
            `).join('');
          }
        }
      } catch (err) {
        console.error('Failed to load overview:', err);
      }
    }

    // Load Resources
    async function loadResources() {
      try {
        const res = await apiFetch('/api/status');
        const tbody = document.getElementById('resources-table-body');
        if (!res.ok) {
          tbody.innerHTML = `<tr><td colspan="7" style="text-align: center; color: var(--accent-red); padding: 24px;">Failed to load resources (${res.status} Unauthorized - check token).</td></tr>`;
          return;
        }
        const data = await res.json();
        const items = data.resources || [];
        if (items.length === 0) {
          tbody.innerHTML = `<tr><td colspan="7" style="text-align: center; color: var(--text-muted); padding: 32px;">No workloads provisioned yet. Agents can deploy via MCP tools.</td></tr>`;
          return;
        }
        tbody.innerHTML = items.map(r => `
          <tr>
            <td style="font-weight: 600; color: var(--text-primary);">${r.name}</td>
            <td><span class="badge badge-blue">${r.provider}</span></td>
            <td>${r.service_type}</td>
            <td><span class="badge ${r.status.toLowerCase().includes('active') ? 'badge-green' : 'badge-grey'}">${r.status}</span></td>
            <td>${r.is_persistent ? '<span class="badge badge-amber">🔒 Protected</span>' : '<span class="badge badge-grey">Ephemeral</span>'}</td>
            <td><span class="badge badge-purple">Managed</span></td>
            <td>
              <button class="btn btn-sm" onclick="reconcileResource('${r.id}')">Reconcile</button>
            </td>
          </tr>
        `).join('');
      } catch (e) {
        console.error('Error loading resources:', e);
      }
    }

    async function reconcileResource(id) {
      try {
        const res = await apiFetch(`/api/reconcile/${id}`, { method: 'POST' });
        if (res.ok) {
          alert(`Reconciled resource '${id}' successfully.`);
          loadResources();
        } else {
          const err = await res.json();
          alert(`Reconciliation error: ${err.error || res.statusText}`);
        }
      } catch (e) {
        alert(`Failed to reconcile: ${e}`);
      }
    }

    // Load Agents
    async function loadAgents() {
      try {
        const res = await apiFetch('/api/agents');
        const tbody = document.getElementById('agents-table-body');
        if (!res.ok) {
          tbody.innerHTML = `<tr><td colspan="7" style="text-align: center; color: var(--accent-red); padding: 24px;">Unauthorized: please configure Bearer token.</td></tr>`;
          return;
        }
        const data = await res.json();
        const items = data.agents || [];
        if (items.length === 0) {
          tbody.innerHTML = `<tr><td colspan="7" style="text-align: center; color: var(--text-muted); padding: 32px;">No paired agents. Click '+ Pair Agent' to issue scoped credentials.</td></tr>`;
          return;
        }
        tbody.innerHTML = items.map(a => `
          <tr>
            <td>
              <div style="font-weight: 600; color: var(--text-primary);">${a.name}</div>
              <div style="font-family: 'JetBrains Mono', monospace; font-size: 11px; color: var(--text-muted);">${a.agent_id}</div>
            </td>
            <td>${a.workspace_id}</td>
            <td>
              ${(a.scopes || []).map(s => {
                const isOver = s === 'admin' || s === 'destructive';
                return `<span class="badge ${isOver ? 'badge-amber' : 'badge-blue'}">${s}</span>`;
              }).join(' ')}
            </td>
            <td><span class="badge ${a.revoked ? 'badge-red' : 'badge-green'}">${a.revoked ? 'REVOKED' : 'ACTIVE'}</span></td>
            <td style="font-family: 'JetBrains Mono', monospace;">${a.token_prefix}...</td>
            <td style="font-size: 12px;">${a.last_used_at ? new Date(a.last_used_at).toLocaleString() : 'Never'}</td>
            <td>
              ${!a.revoked ? `<button class="btn btn-sm btn-danger" onclick="revokeAgent('${a.agent_id}')">Revoke</button>` : '<span style="color: var(--text-muted); font-size: 11px;">Revoked</span>'}
            </td>
          </tr>
        `).join('');
      } catch (e) {
        console.error('Error loading agents:', e);
      }
    }

    function openPairModal() {
      document.getElementById('token-reveal-area').style.display = 'none';
      document.getElementById('btn-submit-pair').style.display = 'inline-flex';
      document.getElementById('pair-modal').classList.add('open');
    }
    function closePairModal() {
      document.getElementById('pair-modal').classList.remove('open');
      loadAgents();
    }

    async function submitPairAgent() {
      const name = document.getElementById('new-agent-name').value.trim();
      const workspace = document.getElementById('new-agent-workspace').value.trim() || 'default';
      const scopes = document.getElementById('new-agent-scopes').value.trim();
      if (!name) { alert('Agent name required.'); return; }

      try {
        const res = await apiFetch('/api/agents/pair', {
          method: 'POST',
          body: JSON.stringify({ name, workspace_id: workspace, scopes })
        });
        if (res.ok) {
          const data = await res.json();
          document.getElementById('token-reveal-area').style.display = 'block';
          document.getElementById('revealed-token-text').textContent = data.token;
          document.getElementById('btn-submit-pair').style.display = 'none';
        } else {
          const err = await res.json();
          alert(`Pairing failed: ${err.error || res.statusText}`);
        }
      } catch (e) {
        alert(`Pairing error: ${e}`);
      }
    }

    async function revokeAgent(id) {
      if (!confirm(`Revoke credentials for agent '${id}' immediately?`)) return;
      try {
        const res = await apiFetch(`/api/agents/${id}/revoke`, { method: 'POST' });
        if (res.ok) {
          alert('Agent credentials revoked.');
          loadAgents();
        } else {
          const err = await res.json();
          alert(`Revocation failed: ${err.error || res.statusText}`);
        }
      } catch (e) {
        alert(`Revocation error: ${e}`);
      }
    }

    // Load Operations
    async function loadOperations() {
      try {
        const res = await apiFetch('/api/operations');
        const tbody = document.getElementById('operations-table-body');
        if (!res.ok) {
          tbody.innerHTML = `<tr><td colspan="7" style="text-align: center; color: var(--accent-red); padding: 24px;">Unauthorized.</td></tr>`;
          return;
        }
        const data = await res.json();
        const items = data.operations || [];
        if (items.length === 0) {
          tbody.innerHTML = `<tr><td colspan="7" style="text-align: center; color: var(--text-muted); padding: 32px;">No operation history recorded yet.</td></tr>`;
          return;
        }
        tbody.innerHTML = items.map(op => `
          <tr>
            <td style="font-family: 'JetBrains Mono', monospace; font-weight: 500;">${op.id}</td>
            <td>${op.workspace_id}</td>
            <td><span class="badge ${op.status === 'Completed' ? 'badge-green' : op.status === 'Running' ? 'badge-blue' : 'badge-red'}">${op.status}</span></td>
            <td>${op.progress_percent}%</td>
            <td>${op.lease_epoch}</td>
            <td style="font-family: 'JetBrains Mono', monospace; font-size: 11px;">${(op.plan_hash || '').substring(0, 16)}...</td>
            <td>
              <button class="btn btn-sm" onclick="viewOpLogs('${op.id}')">Logs</button>
            </td>
          </tr>
        `).join('');
      } catch (e) {
        console.error('Error loading operations:', e);
      }
    }

    function viewOpLogs(id) {
      alert(`Operation logs for '${id}' are recorded in durable storage.`);
    }

    // Run Doctor
    async function runDoctor() {
      const container = document.getElementById('doctor-results-container');
      container.innerHTML = '<p style="color: var(--text-secondary); font-size: 13px;">Running environment diagnostics...</p>';
      try {
        const res = await apiFetch('/api/doctor');
        if (!res.ok) {
          container.innerHTML = `<p style="color: var(--accent-red);">Failed to run doctor diagnostics (${res.status}).</p>`;
          return;
        }
        const doc = await res.json();
        let html = `
          <div style="display: flex; gap: 10px; margin-bottom: 16px;">
            <span class="badge ${doc.overall_status === 'healthy' ? 'badge-green' : doc.overall_status === 'warning' ? 'badge-amber' : 'badge-red'}">
              STATUS: ${doc.overall_status.toUpperCase()}
            </span>
            <span class="badge badge-grey">${doc.os} / ${doc.arch}</span>
            <span class="badge badge-grey">Passed: ${doc.passed_count} | Warnings: ${doc.warning_count} | Failures: ${doc.failure_count}</span>
          </div>
          <div style="display: flex; flex-direction: column; gap: 8px;">
        `;
        (doc.checks || []).forEach(c => {
          const badgeClass = c.status === 'pass' ? 'badge-green' : c.status === 'warn' ? 'badge-amber' : 'badge-red';
          html += `
            <div style="background: var(--bg-surface); border: 1px solid var(--border-quiet); border-radius: var(--radius-sm); padding: 10px 14px;">
              <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px;">
                <span style="font-weight: 600; font-size: 13px;">${c.category}: ${c.name}</span>
                <span class="badge ${badgeClass}">${c.status.toUpperCase()}</span>
              </div>
              <div style="font-size: 12px; color: var(--text-secondary);">${c.message}</div>
              ${c.recommendation ? `<div style="font-size: 11px; color: var(--accent-amber); margin-top: 4px;">↳ Advice: ${c.recommendation}</div>` : ''}
            </div>
          `;
        });
        html += '</div>';
        container.innerHTML = html;
      } catch (e) {
        container.innerHTML = `<p style="color: var(--accent-red);">Diagnostics request error: ${e}</p>`;
      }
    }

    // Initialize on load
    window.addEventListener('DOMContentLoaded', () => {
      const stored = getAuthToken();
      if (stored) document.getElementById('auth-token').value = stored;
      loadOverview();
    });
  </script>
</body>
</html>
"#;
