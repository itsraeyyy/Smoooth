// 1:1 Replica Interactive Tab & Copy Scripts

document.addEventListener('DOMContentLoaded', () => {
  // Tab snippets
  const bugSnippets = {
    HTML: `<pre><code><span class="line-num">1</span>&lt;!DOCTYPE html&gt;
<span class="line-num">2</span>&lt;html&gt;
<span class="line-num">3</span>&lt;head&gt;
<span class="line-num">4</span>  &lt;title&gt;Hello, World!&lt;/title&gt;
<span class="code-line-error"><span class="line-num-error">5</span>&lt;link rel="stylesheet" href="styles.css" /&gt;</span>
<span class="line-num">6</span>&lt;/head&gt;
<span class="line-num">7</span>&lt;body&gt;
<span class="code-line-error"><span class="line-num-error">8</span>  &lt;h1 class="title"&gt;Hello World!&lt;/h1&gt;</span></code></pre>`,
    CSS: `<pre><code><span class="line-num">1</span>.title {
<span class="line-num">2</span>  font-size: 24px;
<span class="line-num">3</span>  color: #111111;
<span class="code-line-error"><span class="line-num-error">4</span>  display: invlid-flex;</span>
<span class="line-num">5</span>}</code></pre>`,
    JavaScript: `<pre><code><span class="line-num">1</span>function greet(name) {
<span class="line-num">2</span>  console.log("Hello, " + name);
<span class="code-line-error"><span class="line-num-error">3</span>  retun true;</span>
<span class="line-num">4</span>}</code></pre>`
  };

  const suggestSnippets = {
    HTML: `<div>
  <span class="line-num">11</span>
  <span class="token-var">user</span> = <span class="token-fn">get_user</span>(user_id)
</div>
<div style="margin-top: 4px;">
  <span class="line-num">12</span>
  <span class="token-kw">if</span> user.
</div>
<div class="autocomplete-popover">
  <div class="popover-item active">&gt; is_active</div>
  <div class="popover-item">&gt; is_authenticated</div>
  <div class="popover-item">&gt; is_admin</div>
  <div class="popover-item">&gt; get_permission</div>
  <div class="popover-footer">
    <span>AI Suggestion</span>
    <span class="key-pill">Tab</span>
  </div>
</div>`,
    CSS: `<div>
  <span class="line-num">11</span>
  <span class="token-var">.container</span> {
</div>
<div style="margin-top: 4px;">
  <span class="line-num">12</span>
  <span class="token-kw">  display:</span> 
</div>
<div class="autocomplete-popover">
  <div class="popover-item active">&gt; flex</div>
  <div class="popover-item">&gt; grid</div>
  <div class="popover-item">&gt; block</div>
  <div class="popover-footer">
    <span>AI Suggestion</span>
    <span class="key-pill">Tab</span>
  </div>
</div>`,
    JavaScript: `<div>
  <span class="line-num">11</span>
  <span class="token-var">const</span> data = await fetch(url)
</div>
<div style="margin-top: 4px;">
  <span class="line-num">12</span>
  <span class="token-kw">if</span> (!data.
</div>
<div class="autocomplete-popover">
  <div class="popover-item active">&gt; ok</div>
  <div class="popover-item">&gt; status</div>
  <div class="popover-item">&gt; headers</div>
  <div class="popover-footer">
    <span>AI Suggestion</span>
    <span class="key-pill">Tab</span>
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
      if (cardType === 'bug') {
        const el = document.getElementById('bugCodeContent');
        if (el && bugSnippets[tabName]) {
          el.innerHTML = bugSnippets[tabName];
        }
      } else if (cardType === 'suggest') {
        const el = document.getElementById('suggestCodeContent');
        if (el && suggestSnippets[tabName]) {
          el.innerHTML = suggestSnippets[tabName];
        }
      }
    });
  });

  // Copy button
  const copyBtn = document.getElementById('copySuggestBtn');
  const copyText = document.getElementById('copySuggestText');
  if (copyBtn && copyText) {
    copyBtn.addEventListener('click', () => {
      navigator.clipboard?.writeText("user = get_user(user_id)\nif user.");
      copyText.textContent = "Copied";
      setTimeout(() => {
        copyText.textContent = "Copy";
      }, 2000);
    });
  }
});
