// Ferndesk Interactive Runtime
// Provides seamless interactivity for Ferndesk 1:1 shell & sections:
// - Hero 4-tab product tour switcher & video modal
// - Header dropdown menus (Features, Resources) & mobile menu drawer
// - Section 7 FAQ accordion

document.addEventListener('DOMContentLoaded', () => {
  // 1. Hero 4-Tab Showcase Switcher
  const heroTabs = [
    {
      id: 'hero-tab-0',
      src: 'https://static.ferndesk.com/cdn-cgi/image/format=webp,quality=80,width=1120/assets/marketing/home-hero-verification-2026-09.webp',
      alt: 'Fern verifies every article against the product'
    },
    {
      id: 'hero-tab-1',
      src: 'https://static.ferndesk.com/cdn-cgi/image/format=webp,quality=80,width=1120/assets/marketing/help-center-light.png',
      alt: 'A Ferndesk help center'
    },
    {
      id: 'hero-tab-2',
      src: 'https://static.ferndesk.com/cdn-cgi/image/format=webp,quality=80,width=1120/assets/marketing/home-hero-automatic-updates-2026-09.webp',
      alt: 'Fern drafted an update from PR #530'
    },
    {
      id: 'hero-tab-3',
      src: 'https://static.ferndesk.com/cdn-cgi/image/format=webp,quality=80,width=1120/assets/marketing/ai-chatbot-for-customer-service.png',
      alt: 'Fern answers customer questions from verified docs'
    }
  ];

  const heroPanel = document.getElementById('hero-panel');
  const heroTabBtns = heroTabs.map(t => document.getElementById(t.id)).filter(Boolean);

  if (heroPanel && heroTabBtns.length > 0) {
    const heroImg = heroPanel.querySelector('img');

    function selectTab(index) {
      heroTabBtns.forEach((btn, idx) => {
        const isSelected = idx === index;
        btn.setAttribute('aria-selected', isSelected ? 'true' : 'false');
        btn.setAttribute('tabindex', isSelected ? '0' : '-1');
        const svg = btn.querySelector('svg');
        if (isSelected) {
          btn.className = 'relative flex shrink-0 items-center gap-2 overflow-hidden rounded-t border border-b-0 pt-2.5 pr-4 pb-2.5 pl-3.5 text-sm leading-tight font-medium transition-colors border-border-strong bg-white text-fg';
          if (svg) svg.className = 'shrink-0 text-fern-600';
        } else {
          btn.className = 'relative flex shrink-0 items-center gap-2 overflow-hidden rounded-t border border-b-0 pt-2.5 pr-4 pb-2.5 pl-3.5 text-sm leading-tight font-medium transition-colors border-transparent text-fg-3 hover:text-fg';
          if (svg) svg.className = 'shrink-0 text-fg-4';
        }
      });

      const tabData = heroTabs[index];
      if (heroImg && tabData) {
        heroImg.src = tabData.src;
        heroImg.alt = tabData.alt;
        heroImg.srcset = '';
      }
    }

    heroTabBtns.forEach((btn, idx) => {
      btn.addEventListener('click', (e) => {
        e.preventDefault();
        selectTab(idx);
      });
    });
  }

  // 2. Video Demo Modal
  const videoBtn = document.querySelector('button[aria-label="Watch how Fern verifies your help center"]');
  if (videoBtn) {
    videoBtn.addEventListener('click', (e) => {
      e.preventDefault();
      let modal = document.getElementById('ferndesk-video-modal');
      if (!modal) {
        modal = document.createElement('div');
        modal.id = 'ferndesk-video-modal';
        modal.className = 'fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-4';
        modal.innerHTML = `
          <div class="relative w-full max-w-4xl overflow-hidden rounded-xl bg-black border border-white/20 shadow-2xl">
            <button type="button" id="close-video-modal" class="absolute top-4 right-4 z-10 flex size-9 items-center justify-center rounded-full bg-white/10 text-white hover:bg-white/20">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
            </button>
            <div class="aspect-video w-full flex items-center justify-center text-white">
              <video controls autoplay class="w-full h-full" src="https://static.ferndesk.com/assets/marketing/verification-demo.mp4" poster="https://static.ferndesk.com/cdn-cgi/image/format=webp,quality=80,width=1120/assets/marketing/home-hero-verification-2026-09.webp">
                Your browser does not support video.
              </video>
            </div>
          </div>
        `;
        document.body.appendChild(modal);
        modal.querySelector('#close-video-modal')?.addEventListener('click', () => {
          modal.remove();
        });
        modal.addEventListener('click', (ev) => {
          if (ev.target === modal) modal.remove();
        });
      }
    });
  }

  // 3. Ferndesk Header Dropdown Menus (Features & Resources)
  const featuresBtn = document.querySelector('button[aria-controls="features-menu"]');
  const resourcesBtn = document.querySelector('button[aria-controls="resources-menu"]');

  let featuresMenu = document.getElementById('features-menu');
  if (!featuresMenu && featuresBtn) {
    featuresMenu = document.createElement('div');
    featuresMenu.id = 'features-menu';
    featuresMenu.className = 'border-border bg-bg-raised shadow-float absolute top-full left-1/2 z-50 mt-2 w-[900px] max-w-[calc(100vw-2.5rem)] -translate-x-1/2 overflow-hidden rounded-lg border hidden';
    featuresMenu.innerHTML = `
      <div class="grid grid-cols-3 gap-x-8 px-7 pt-7 pb-6">
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2.5 text-xs font-semibold uppercase tracking-wider text-fg-3">Help center</span>
          <a href="/ai-help-center" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Hosted help center</span><span class="text-fg-3 text-sm">Your domain or /help</span></span>
          </a>
          <a href="/generative-ai-customer-service" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">AI conversations</span><span class="text-fg-3 text-sm">Answers drawn from your docs</span></span>
          </a>
          <a href="/self-service-help-widget" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">In-app widget</span><span class="text-fg-3 text-sm">Help inside your product</span></span>
          </a>
          <a href="/multilingual-knowledge-base" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Multilingual</span><span class="text-fg-3 text-sm">Translated, kept in sync</span></span>
          </a>
          <a href="/internal-knowledge-base" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Private knowledge base</span><span class="text-fg-3 text-sm">Docs behind a login</span></span>
          </a>
        </div>
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2.5 text-xs font-semibold uppercase tracking-wider text-fg-3">AI & automation</span>
          <a href="/self-updating-help-center" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Verification</span><span class="text-fg-3 text-sm">Every article checked</span></span>
          </a>
          <a href="/code-to-docs" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Code to docs</span><span class="text-fg-3 text-sm">Fixes from every PR</span></span>
          </a>
          <a href="/automated-screenshots-for-docs" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Automated screenshots</span><span class="text-fg-3 text-sm">Re-taken when UI changes</span></span>
          </a>
          <a href="/ai-documentation-agent" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">AI drafting</span><span class="text-fg-3 text-sm">Written from your codebase</span></span>
          </a>
        </div>
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2.5 text-xs font-semibold uppercase tracking-wider text-fg-3">Connect</span>
          <a href="/integrations" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Integrations</span><span class="text-fg-3 text-sm">GitHub, Linear, Slack</span></span>
          </a>
          <a href="https://docs.ferndesk.com/mcp-hjgdc" target="_blank" rel="noopener noreferrer" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">MCP server</span><span class="text-fg-3 text-sm">Let Claude &amp; Cursor read your docs</span></span>
          </a>
          <a href="/chrome-extension-for-documentation" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Chrome extension</span><span class="text-fg-3 text-sm">Capture product changes</span></span>
          </a>
        </div>
      </div>
    `;
    featuresBtn.parentElement?.appendChild(featuresMenu);
  }

  let resourcesMenu = document.getElementById('resources-menu');
  if (!resourcesMenu && resourcesBtn) {
    resourcesMenu = document.createElement('div');
    resourcesMenu.id = 'resources-menu';
    resourcesMenu.className = 'border-border bg-bg-raised shadow-float absolute top-full left-1/2 z-50 mt-2 w-[520px] max-w-[calc(100vw-2.5rem)] -translate-x-1/2 overflow-hidden rounded-lg border hidden';
    resourcesMenu.innerHTML = `
      <div class="grid grid-cols-2 gap-x-6 px-6 pt-6 pb-5">
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2 text-xs font-semibold uppercase tracking-wider text-fg-3">Learn</span>
          <a href="/blog" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Blog</span><span class="text-fg-3 text-sm">Product, AI, and docs advice</span></span>
          </a>
          <a href="/answers" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">Answer hub</span><span class="text-fg-3 text-sm">Guides and deep dives</span></span>
          </a>
        </div>
        <div class="flex flex-col gap-1">
          <span class="eyebrow px-2 pb-2 text-xs font-semibold uppercase tracking-wider text-fg-3">Compare</span>
          <a href="/compare" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">All comparisons</span><span class="text-fg-3 text-sm">See how Ferndesk stacks up</span></span>
          </a>
          <a href="/compare/zendesk" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">vs Zendesk</span><span class="text-fg-3 text-sm">Modern, AI-first help center</span></span>
          </a>
          <a href="/compare/intercom" class="hover:bg-bg flex items-center gap-3 rounded px-2 py-2 transition-colors">
            <span class="flex min-w-0 flex-col gap-px"><span class="text-fg text-base font-medium">vs Intercom</span><span class="text-fg-3 text-sm">Better self-service at lower cost</span></span>
          </a>
        </div>
      </div>
    `;
    resourcesBtn.parentElement?.appendChild(resourcesMenu);
  }

  function toggleDropdown(btn, menu) {
    if (!btn || !menu) return;
    const isExpanded = btn.getAttribute('aria-expanded') === 'true';
    closeAllDropdowns();
    if (!isExpanded) {
      btn.setAttribute('aria-expanded', 'true');
      menu.classList.remove('hidden');
    }
  }

  function closeAllDropdowns() {
    if (featuresBtn) featuresBtn.setAttribute('aria-expanded', 'false');
    if (resourcesBtn) resourcesBtn.setAttribute('aria-expanded', 'false');
    if (featuresMenu) featuresMenu.classList.add('hidden');
    if (resourcesMenu) resourcesMenu.classList.add('hidden');
  }

  if (featuresBtn && featuresMenu) {
    featuresBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleDropdown(featuresBtn, featuresMenu);
    });
  }

  if (resourcesBtn && resourcesMenu) {
    resourcesBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleDropdown(resourcesBtn, resourcesMenu);
    });
  }

  document.addEventListener('click', (e) => {
    if (!e.target.closest('#features-menu') && !e.target.closest('button[aria-controls="features-menu"]') &&
        !e.target.closest('#resources-menu') && !e.target.closest('button[aria-controls="resources-menu"]')) {
      closeAllDropdowns();
    }
  });

  // 4. Mobile Menu Drawer
  const mobileBtn = document.querySelector('button[aria-controls="mobile-menu"]');
  const header = document.querySelector('header');
  let mobileMenu = document.getElementById('mobile-menu');

  if (!mobileMenu && header) {
    mobileMenu = document.createElement('div');
    mobileMenu.id = 'mobile-menu';
    mobileMenu.className = 'border-border bg-bg border-b px-5 pt-4 pb-8 lg:hidden hidden';
    mobileMenu.innerHTML = `
      <div class="flex flex-col gap-4">
        <nav class="flex flex-col divide-y divide-border">
          <a href="/pricing" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Pricing</a>
          <a href="/customers" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Customers</a>
          <a href="/migration" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Migrate</a>
          <a href="/answers" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Answer hub</a>
          <a href="/blog" class="focus-ring text-fg hover:bg-bg-muted -mx-2 rounded px-2 py-4 text-lg font-medium transition-colors">Blog</a>
        </nav>
        <div class="flex flex-col gap-2 pt-6">
          <a href="https://ferndesk.com/app" class="focus-ring text-prose hover:bg-bg-muted hover:text-fg rounded border border-gray-300 px-3 py-2 text-center text-base font-medium transition-colors">Sign in</a>
          <a href="https://ferndesk.com/app/signup" class="focus-ring bg-brand text-white hover:bg-brand/90 rounded px-3 py-2 text-center text-base font-medium transition-colors">Start free</a>
        </div>
      </div>
    `;
    header.appendChild(mobileMenu);
  }

  if (mobileBtn && mobileMenu) {
    mobileBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      const isOpen = !mobileMenu.classList.contains('hidden');
      if (isOpen) {
        mobileMenu.classList.add('hidden');
        mobileBtn.setAttribute('aria-expanded', 'false');
      } else {
        mobileMenu.classList.remove('hidden');
        mobileBtn.setAttribute('aria-expanded', 'true');
      }
    });
  }

  // 5. FAQ Accordion Interaction
  const jsonLdScripts = document.querySelectorAll('script[type="application/ld+json"]');
  let faqData = [];
  jsonLdScripts.forEach((s) => {
    try {
      const parsed = JSON.parse(s.textContent);
      if (parsed['@type'] === 'FAQPage' && parsed.mainEntity) {
        faqData = parsed.mainEntity;
      }
    } catch (_) {}
  });

  const faqButtons = document.querySelectorAll('button[aria-expanded]');
  let faqBtnIndex = 0;
  faqButtons.forEach((btn) => {
    const numSpan = btn.querySelector('span.tabular-nums');
    if (!numSpan && !btn.closest('#faq')) return;

    const currentFaqIndex = faqBtnIndex++;
    btn.addEventListener('click', (e) => {
      e.preventDefault();
      const isExpanded = btn.getAttribute('aria-expanded') === 'true';
      const newExpanded = !isExpanded;
      btn.setAttribute('aria-expanded', newExpanded ? 'true' : 'false');

      const icon = btn.querySelector('span[aria-hidden="true"], svg');
      if (icon) {
        if (newExpanded) {
          icon.classList.add('rotate-90');
        } else {
          icon.classList.remove('rotate-90');
        }
      }

      const parentContainer = btn.parentElement;
      let answerDiv = parentContainer?.querySelector(':scope > div:not([class*="grid"])');
      if (!answerDiv && faqData[currentFaqIndex]) {
        answerDiv = document.createElement('div');
        answerDiv.innerHTML = `<p class="pb-6 pr-0 text-lg leading-[1.55] text-pretty text-prose sm:pl-14 sm:pr-10 lg:pb-[26px]">${faqData[currentFaqIndex].acceptedAnswer?.text || ''}</p>`;
        parentContainer.appendChild(answerDiv);
      }
      if (answerDiv) {
        answerDiv.style.display = newExpanded ? 'block' : 'none';
      }
    });
  });

  // 6. One-Command Install Copy Interaction
  const copyBtn = document.getElementById('copy-install-btn');
  const installCmd = document.getElementById('install-cmd');
  const copyText = document.getElementById('copy-text');
  if (copyBtn && installCmd) {
    copyBtn.addEventListener('click', async () => {
      const textToCopy = installCmd.textContent.trim();
      try {
        await navigator.clipboard.writeText(textToCopy);
      } catch (_) {
        const temp = document.createElement('textarea');
        temp.value = textToCopy;
        document.body.appendChild(temp);
        temp.select();
        document.execCommand('copy');
        temp.remove();
      }
      if (copyText) copyText.textContent = 'Copied!';
      copyBtn.classList.add('border-emerald-500/50', 'text-emerald-400');
      setTimeout(() => {
        if (copyText) copyText.textContent = 'Copy';
        copyBtn.classList.remove('border-emerald-500/50', 'text-emerald-400');
      }, 2000);
    });
  }
});
