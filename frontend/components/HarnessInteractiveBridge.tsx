"use client";

import { useEffect } from "react";

export function HarnessInteractiveBridge() {
  useEffect(() => {
    // 1. Draw Royal Navy Ambient Light on Hero Canvas
    const canvas = document.querySelector("canvas") as HTMLCanvasElement | null;
    if (canvas) {
      const resize = () => {
        canvas.width = canvas.parentElement?.clientWidth || window.innerWidth;
        canvas.height = canvas.parentElement?.clientHeight || 900;
        const ctx = canvas.getContext("2d");
        if (ctx) {
          const grad = ctx.createRadialGradient(
            canvas.width / 2,
            canvas.height * 0.15,
            10,
            canvas.width / 2,
            canvas.height * 0.25,
            canvas.width * 0.65
          );
          grad.addColorStop(0, "rgba(35, 85, 175, 0.48)");
          grad.addColorStop(0.35, "rgba(20, 52, 115, 0.32)");
          grad.addColorStop(0.7, "rgba(10, 18, 32, 0.85)");
          grad.addColorStop(1, "rgba(8, 12, 20, 0)");
          ctx.fillStyle = grad;
          ctx.fillRect(0, 0, canvas.width, canvas.height);
        }
      };
      resize();
      window.addEventListener("resize", resize);
      if (canvas.parentElement) {
        canvas.parentElement.style.opacity = "1";
        canvas.parentElement.style.filter = "none";
      }
    }

    // 2. Reveal entrance-animated elements via IntersectionObserver
    const hiddenElements = document.querySelectorAll<HTMLElement>(
      '[style*="opacity:0"], [style*="opacity: 0"], .HarnessCapabilitiesSection_entrance__EZAWd, #products'
    );

    hiddenElements.forEach((el) => {
      // Preserve canvas mask wrapper
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

      // Immediate reveal if already within viewport
      const rect = el.getBoundingClientRect();
      if (rect.top < window.innerHeight && rect.bottom > 0) {
        el.style.opacity = "1";
        el.style.transform = "none";
        el.style.filter = "none";
      }
    });

    // 3. Wire up "Thought for a while" accordion
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

    // 4. Wire up Pomodoro timer Start button
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

    // 5. Wire up Copy buttons in developer experience
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

    // 6. Activate live demo frames (Workflow, Trace, Deliverables, Plugins)
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

    // 7. Mobile hamburger menu toggle
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

    // 8. Mobile download dropdown inside drawer
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

    // 9. WeChat QR code popup on click / touch
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


