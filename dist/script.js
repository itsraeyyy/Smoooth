// Smoooth Interactive Tabs & Copy Handlers

document.addEventListener('DOMContentLoaded', () => {
  // 1. Database & Container Adapter Snippets
  const dbSnippets = {
    PostgreSQL: `<pre><code><span class="line-num">1</span><span class="token-kw">POST</span> /api/adapters/postgres/provision
<span class="line-num">2</span>{
<span class="line-num">3</span>  <span class="token-var">"service"</span>: <span class="token-fn">"primary-db"</span>,
<span class="line-num">4</span>  <span class="token-var">"volume"</span>: <span class="token-fn">"/var/lib/smoooth/volumes/pgdata"</span>,
<span class="line-num">5</span>  <span class="token-var">"persistence"</span>: <span class="token-kw">true</span>,
<span class="line-num">6</span>  <span class="token-var">"health_probe"</span>: <span class="token-fn">"pg_isready -U postgres"</span>
<span class="line-num">7</span>}
<span class="line-num">8</span><span class="token-var">STATUS:</span> <span class="token-fn">HEALTHY</span> (verified in 1.4s, 5432/tcp bound)</code></pre>`,

    Redis: `<pre><code><span class="line-num">1</span><span class="token-kw">POST</span> /api/adapters/redis/provision
<span class="line-num">2</span>{
<span class="line-num">3</span>  <span class="token-var">"service"</span>: <span class="token-fn">"cache-layer"</span>,
<span class="line-num">4</span>  <span class="token-var">"mode"</span>: <span class="token-fn">"AOF_PERSISTENT"</span>,
<span class="line-num">5</span>  <span class="token-var">"volume"</span>: <span class="token-fn">"/var/lib/smoooth/volumes/redis"</span>,
<span class="line-num">6</span>  <span class="token-var">"health_probe"</span>: <span class="token-fn">"redis-cli ping"</span>
<span class="line-num">7</span>}
<span class="line-num">8</span><span class="token-var">STATUS:</span> <span class="token-fn">HEALTHY</span> (verified PONG, 6379/tcp bound)</code></pre>`,

    MongoDB: `<pre><code><span class="line-num">1</span><span class="token-kw">POST</span> /api/adapters/mongodb/provision
<span class="line-num">2</span>{
<span class="line-num">3</span>  <span class="token-var">"service"</span>: <span class="token-fn">"document-store"</span>,
<span class="line-num">4</span>  <span class="token-var">"volume"</span>: <span class="token-fn">"/var/lib/smoooth/volumes/mongo"</span>,
<span class="line-num">5</span>  <span class="token-var">"auth"</span>: <span class="token-fn">"SCRAM-SHA-256"</span>,
<span class="line-num">6</span>  <span class="token-var">"health_probe"</span>: <span class="token-fn">"mongosh --eval 'db.adminCommand({ping:1})'"</span>
<span class="line-num">7</span>}
<span class="line-num">8</span><span class="token-var">STATUS:</span> <span class="token-fn">HEALTHY</span> (verified in 2.1s, 27017/tcp bound)</code></pre>`
  };

  // 2. Scoped Security Snippets
  const securitySnippets = {
    Pairing: `<div>
  <span class="line-num">1</span>
  <span class="token-kw">$</span> smoooth agent pair --name <span class="token-fn">"claude-code"</span> --scopes <span class="token-var">"read_only,plan,deploy"</span>
</div>
<div style="margin-top: 4px;">
  <span class="line-num">2</span>
  <span class="token-var">token:</span> <span class="token-fn">smt_a8f93e...</span> (256-bit entropy, saved to /etc/smoooth/auth.token)
</div>
<div class="autocomplete-popover">
  <div class="popover-item active">&gt; read_only (granted)</div>
  <div class="popover-item active">&gt; plan (granted)</div>
  <div class="popover-item active">&gt; deploy (granted)</div>
  <div class="popover-item" style="color:#DC2626;">&gt; admin (blocked)</div>
  <div class="popover-footer">
    <span>Scoped Least Privilege</span>
    <span class="key-pill">RBAC</span>
  </div>
</div>`,

    Scopes: `<div>
  <span class="line-num">1</span>
  <span class="token-kw">$</span> smoooth inspect scopes --agent <span class="token-fn">"cursor-assistant"</span>
</div>
<div style="margin-top: 4px;">
  <span class="line-num">2</span>
  <span class="token-var">active_scopes:</span> [read_only, plan, deploy]
</div>
<div class="autocomplete-popover">
  <div class="popover-item active">&gt; read_only: inspect system</div>
  <div class="popover-item active">&gt; plan: dry-run DAG DAGs</div>
  <div class="popover-item active">&gt; deploy: roll containers</div>
  <div class="popover-item" style="color:#DC2626;">&gt; admin: key revocation</div>
  <div class="popover-footer">
    <span>Permission Tier</span>
    <span class="key-pill">Checked</span>
  </div>
</div>`,

    Audit: `<div>
  <span class="line-num">1</span>
  <span class="token-kw">$</span> journalctl -u smoooth.service --since <span class="token-fn">"10m ago"</span>
</div>
<div style="margin-top: 4px;">
  <span class="line-num">2</span>
  <span class="token-var">AUDIT:</span> agent=claude-code action=deploy_postgres status=success
</div>
<div class="autocomplete-popover">
  <div class="popover-item active">&gt; 18:42:01 plan validated</div>
  <div class="popover-item active">&gt; 18:42:03 volume retained</div>
  <div class="popover-item active">&gt; 18:42:05 healthy 200 OK</div>
  <div class="popover-item">&gt; 18:42:06 commit state</div>
  <div class="popover-footer">
    <span>Append-Only Log</span>
    <span class="key-pill">Audit</span>
  </div>
</div>`
  };

  // Tab switching
  const tabButtons = document.querySelectorAll('.tab-btn');
  tabButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      const cardType = btn.getAttribute('data-card');
      const tabName = btn.getAttribute('data-tab');

      // Update active state in this group
      document.querySelectorAll(`.tab-btn[data-card="${cardType}"]`).forEach(b => {
        b.classList.remove('active');
      });
      btn.classList.add('active');

      // Update snippet
      if (cardType === 'db') {
        const el = document.getElementById('dbCodeContent');
        if (el && dbSnippets[tabName]) {
          el.innerHTML = dbSnippets[tabName];
        }
      } else if (cardType === 'security') {
        const el = document.getElementById('securityCodeContent');
        if (el && securitySnippets[tabName]) {
          el.innerHTML = securitySnippets[tabName];
        }
      }
    });
  });

  // Copy button for pairing command
  const copyBtn = document.getElementById('copySecurityBtn');
  const copyText = document.getElementById('copySecurityText');
  if (copyBtn && copyText) {
    copyBtn.addEventListener('click', () => {
      const textToCopy = 'smoooth agent pair --name "claude-code" --scopes "read_only,plan,deploy"';
      navigator.clipboard?.writeText(textToCopy);
      copyText.textContent = 'Copied';
      setTimeout(() => {
        copyText.textContent = 'Copy';
      }, 2000);
    });
  }
});
