"use client";

import { useEffect } from "react";

export function HarnessInteractiveBridge() {
  useEffect(() => {
    // 1. Render Atmospheric Oceanic Twilight Sky on Hero Canvas matching live deepseek.com
    const canvas = document.querySelector("canvas") as HTMLCanvasElement | null;
    if (canvas) {
      if (canvas.parentElement) {
        canvas.parentElement.style.opacity = "1";
        canvas.parentElement.style.filter = "none";
        (canvas.parentElement.style as any).webkitMask = "none";
        canvas.parentElement.style.mask = "none";
      }

      const renderAtmosphere = () => {
        const parent = canvas.parentElement;
        const w = parent?.clientWidth || window.innerWidth;
        const h = parent?.clientHeight || 1200;

        canvas.width = w * (window.devicePixelRatio || 1);
        canvas.height = h * (window.devicePixelRatio || 1);
        canvas.style.width = `${w}px`;
        canvas.style.height = `${h}px`;

        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        ctx.scale(window.devicePixelRatio || 1, window.devicePixelRatio || 1);

        // Layer 1: Base Oceanic Sky Gradient calibrated to live deepseek.com pixels:
        // Top Left: (25, 53, 90) | Top Center: (44, 81, 128) | Hero Center: (46, 79, 121)
        const baseGrad = ctx.createLinearGradient(0, 0, 0, h);
        baseGrad.addColorStop(0, "#19355a");
        baseGrad.addColorStop(0.18, "#284b77");
        baseGrad.addColorStop(0.38, "#1d3656");
        baseGrad.addColorStop(0.65, "#14243a");
        baseGrad.addColorStop(0.88, "#0d141e");
        baseGrad.addColorStop(1, "#0a0a0a");
        ctx.fillStyle = baseGrad;
        ctx.fillRect(0, 0, w, h);

        // Layer 2: Volumetric Top-Right Light Shaft (The luminous silver-slate sky glow matching live site at 93, 109, 127)
        const trGlow = ctx.createRadialGradient(
          w * 0.94, h * 0.04, 10,
          w * 0.85, h * 0.12, w * 0.82
        );
        trGlow.addColorStop(0, "rgba(165, 185, 208, 0.48)");
        trGlow.addColorStop(0.28, "rgba(115, 145, 180, 0.30)");
        trGlow.addColorStop(0.60, "rgba(55, 85, 125, 0.14)");
        trGlow.addColorStop(1, "rgba(0, 0, 0, 0)");
        ctx.fillStyle = trGlow;
        ctx.fillRect(0, 0, w, h);

        // Layer 3: Center Luminous Aura behind "DeepSeek Harness Ready to use. Right now."
        const centerAura = ctx.createRadialGradient(
          w * 0.5, h * 0.22, 20,
          w * 0.5, h * 0.22, w * 0.55
        );
        centerAura.addColorStop(0, "rgba(55, 95, 150, 0.42)");
        centerAura.addColorStop(0.35, "rgba(35, 68, 115, 0.26)");
        centerAura.addColorStop(0.70, "rgba(18, 42, 78, 0.10)");
        centerAura.addColorStop(1, "rgba(0, 0, 0, 0)");
        ctx.fillStyle = centerAura;
        ctx.fillRect(0, 0, w, h);

        // Layer 4: Top-Left Navy Depth
        const tlGlow = ctx.createRadialGradient(
          w * 0.08, h * 0.04, 10,
          w * 0.15, h * 0.08, w * 0.58
        );
        tlGlow.addColorStop(0, "rgba(35, 70, 115, 0.30)");
        tlGlow.addColorStop(0.5, "rgba(18, 42, 75, 0.12)");
        tlGlow.addColorStop(1, "rgba(0, 0, 0, 0)");
        ctx.fillStyle = tlGlow;
        ctx.fillRect(0, 0, w, h);

        // Layer 5: Window Ambient Backlight (Soft luminous shelf behind desktop window top)
        const windowGlow = ctx.createRadialGradient(
          w * 0.5, h * 0.52, 40,
          w * 0.5, h * 0.52, w * 0.48
        );
        windowGlow.addColorStop(0, "rgba(45, 78, 125, 0.26)");
        windowGlow.addColorStop(0.5, "rgba(22, 45, 80, 0.10)");
        windowGlow.addColorStop(1, "rgba(0, 0, 0, 0)");
        ctx.fillStyle = windowGlow;
        ctx.fillRect(0, 0, w, h);
      };

      renderAtmosphere();
      window.addEventListener("resize", renderAtmosphere);
    }

    // 2. Wire up Hero Split Download button and dropdown toggle
    const toggleBtn = document.querySelector(".HarnessDownloadMenu_arrow__60_9L") as HTMLButtonElement | null;
    const dropdown = document.getElementById("hero-download-dropdown") as HTMLElement | null;
    if (toggleBtn && dropdown) {
      toggleBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        const isVisible = dropdown.style.display !== "none";
        dropdown.style.display = isVisible ? "none" : "block";
        toggleBtn.setAttribute("aria-expanded", isVisible ? "false" : "true");
        setTimeout(() => {
          dropdown.style.opacity = isVisible ? "0" : "1";
          dropdown.style.transform = isVisible ? "translate(-50%, -6px)" : "translate(-50%, 0)";
        }, 10);
      });
      document.addEventListener("click", () => {
        dropdown.style.opacity = "0";
        dropdown.style.transform = "translate(-50%, -6px)";
        toggleBtn.setAttribute("aria-expanded", "false");
        setTimeout(() => {
          dropdown.style.display = "none";
        }, 200);
      });
    }

    // 3. Reveal entrance-animated elements via IntersectionObserver
    const hiddenElements = document.querySelectorAll<HTMLElement>(
      '[style*="opacity:0"], [style*="opacity: 0"], .HarnessCapabilitiesSection_entrance__EZAWd, #products'
    );

    hiddenElements.forEach((el) => {
      if (el.tagName.toLowerCase() === "canvas" || el.querySelector("canvas")) {
        el.style.opacity = "1";
        el.style.filter = "none";
        return;
      }

      el.style.transition =
        "opacity 0.75s cubic-bezier(0.16, 1, 0.3, 1), transform 0.75s cubic-bezier(0.16, 1, 0.3, 1), filter 0.75s cubic-bezier(0.16, 1, 0.3, 1)";

      const observer = new IntersectionObserver(
        (entries) => {
          entries.forEach((entry) => {
            if (entry.isIntersecting) {
              el.style.opacity = "1";
              el.style.transform = "none";
              el.style.filter = "none";
              observer.unobserve(entry.target);
            }
          });
        },
        { threshold: 0.05, rootMargin: "50px" }
      );
      observer.observe(el);

      const rect = el.getBoundingClientRect();
      if (rect.top < window.innerHeight && rect.bottom > 0) {
        el.style.opacity = "1";
        el.style.transform = "none";
        el.style.filter = "none";
      }
    });

    // 4. Wire up "Thought for a while" accordion
    const thoughtBtn = document.querySelector<HTMLElement>(
      ".HarnessDesktopPreview_analysisSummary__Qp8FG"
    );
    if (thoughtBtn) {
      thoughtBtn.style.cursor = "pointer";
      thoughtBtn.style.userSelect = "none";
      let isOpen = false;
      thoughtBtn.addEventListener("click", () => {
        isOpen = !isOpen;
        let detail = document.getElementById("dsh-thought-details");
        if (!detail) {
          detail = document.createElement("div");
          detail.id = "dsh-thought-details";
          detail.style.padding = "10px 14px";
          detail.style.margin = "8px 0 12px 0";
          detail.style.borderRadius = "8px";
          detail.style.background = "#0d1117";
          detail.style.border = "1px solid rgba(255,255,255,0.08)";
          detail.style.fontFamily = "monospace";
          detail.style.fontSize = "12px";
          detail.style.color = "#8b949e";
          detail.style.lineHeight = "1.6";
          detail.innerHTML = `
            <div style="color: #58a6ff; font-weight: 500;">› Probing runtime dependencies & plugins...</div>
            <div style="color: #c9d1d9;">  • Initializing Cordis micro-kernel context</div>
            <div style="color: #c9d1d9;">  • Loading dynamic plugin registry (shell, git, fs)</div>
            <div style="color: #3fb950; margin-top: 4px;">✓ Environment verified. Ready for tasks.</div>
          `;
          thoughtBtn.parentNode?.insertBefore(detail, thoughtBtn.nextSibling);
        }
        detail.style.display = isOpen ? "block" : "none";
      });
    }

    // 5. Wire up Pomodoro timer Start button
    const timerStart = document.querySelector<HTMLElement>(
      ".HarnessPluginsPreview_timerStart__34s_M"
    );
    const timerStrips = document.querySelectorAll<HTMLElement>(
      ".HarnessPluginsPreview_timerStrip__5RKkN"
    );
    if (timerStart && timerStrips.length > 0) {
      timerStart.style.cursor = "pointer";
      let isRunning = false;
      let timerInterval: any = null;
      let frame = 0;

      timerStart.addEventListener("click", () => {
        isRunning = !isRunning;
        if (isRunning) {
          timerStart.style.opacity = "0.7";
          timerInterval = setInterval(() => {
            frame = (frame + 1) % 25;
            timerStrips.forEach((strip) => {
              strip.style.transform = `translateY(-${frame * 36}px)`;
              strip.style.transition = "transform 0.4s ease";
            });
          }, 1000);
        } else {
          clearInterval(timerInterval);
          timerStart.style.opacity = "1";
        }
      });
    }

    // 6. Wire up Copy buttons in developer experience
    const buttons = document.querySelectorAll<HTMLButtonElement>("button");
    buttons.forEach((btn) => {
      if (btn.textContent?.trim() === "Copy") {
        btn.addEventListener("click", () => {
          const codeEl = btn.parentElement?.querySelector("code");
          if (codeEl) {
            let text = codeEl.textContent || "";
            if (text.startsWith("$ ")) text = text.slice(2);
            navigator.clipboard.writeText(text.trim());
            btn.textContent = "Copied!";
            setTimeout(() => {
              btn.textContent = "Copy";
            }, 2000);
          }
        });
      }
    });

    // 7. Activate live demo frames (Workflow, Trace, Deliverables, Plugins)
    const demoFrames = document.querySelectorAll<HTMLElement>("[data-running]");
    demoFrames.forEach((frame) => {
      frame.setAttribute("data-running", "true");
      frame.setAttribute("data-paused", "false");

      const card = frame.closest(".HarnessCapabilitiesSection_card__Yf6o5");
      const pauseBtn = card?.querySelector<HTMLButtonElement>(
        ".HarnessCapabilitiesSection_pauseControl__SbfPK"
      );
      if (pauseBtn) {
        pauseBtn.addEventListener("click", () => {
          const isPaused = frame.getAttribute("data-paused") === "true";
          frame.setAttribute("data-paused", isPaused ? "false" : "true");
        });
      }
    });

    // 8. Mobile hamburger menu toggle
    const mobileMenuBtn = document.getElementById("harness-mobile-menu");
    const mobileMenu = document.querySelector<HTMLElement>(".ds-mobile-menu");
    const mobileCloseBtn = mobileMenu?.querySelector<HTMLButtonElement>(
      'button[aria-label="Close menu"]'
    );

    if (mobileMenuBtn && mobileMenu) {
      mobileMenuBtn.addEventListener("click", () => {
        mobileMenu.classList.add("is-open");
        document.body.style.overflow = "hidden";
      });
    }
    if (mobileCloseBtn && mobileMenu) {
      mobileCloseBtn.addEventListener("click", () => {
        mobileMenu.classList.remove("is-open");
        document.body.style.overflow = "";
      });
    }

    // 9. Mobile download dropdown inside drawer
    const mobileDownloadTrigger = document.querySelector<HTMLButtonElement>(
      ".harness-mobile-download-trigger"
    );
    const mobileDownload = document.querySelector<HTMLElement>(
      ".harness-mobile-download"
    );
    if (mobileDownloadTrigger && mobileDownload) {
      mobileDownloadTrigger.addEventListener("click", () => {
        const isOpen = mobileDownload.getAttribute("data-open") === "true";
        mobileDownload.setAttribute("data-open", isOpen ? "false" : "true");
        mobileDownloadTrigger.setAttribute(
          "aria-expanded",
          isOpen ? "false" : "true"
        );
      });
    }

    // 10. WeChat QR code popup on click / touch
    const qrTriggers = document.querySelectorAll<HTMLElement>(".ds-qr-trigger");
    qrTriggers.forEach((trig) => {
      trig.addEventListener("click", (e) => {
        e.stopPropagation();
        trig.classList.toggle("ds-qr-active");
      });
    });
    document.addEventListener("click", () => {
      qrTriggers.forEach((trig) => trig.classList.remove("ds-qr-active"));
    });
  }, []);

  return null;
}
