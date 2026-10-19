# DeepSeek Harness / Smoooth Brand & Design System Guide
**Version:** 1.0.0  
**Theme:** Dark Theme (`[data-theme="dark"]`, Page Base `#0a0a0a`)  
**Standard:** 1:1 Parity Specification with `https://www.deepseek.com/en/harness/`  

---

## 1. Design Ethos

1. **Proof Over Decoration:** High-fidelity UI previews, real terminal inputs, real diff hunks, and real tool call executions. No decorative pastel pill badges or filler marketing buzzwords.
2. **Atmospheric Oceanic Twilight:** A 5-layer volumetric lighting model transitioning from oceanic navy (`#19355a`) and silver sky highlights (`#5d6d7f`) into deep black (`#0a0a0a`).
3. **High-Contrast Precision Typography:** Pairing **Host Grotesk** for clean, modern display headings, **DM Sans** for neutral UI copy, and **Fragment Mono** for monospace terminal commands and parameters.
4. **Immediate Tactile Feedback:** Solid white pill buttons, split platform-detection download controls, subtle glass surfaces with `1px inset` white specular reflections.

---

## 2. Color System & Design Tokens

### 2.1. Canvas Atmospheric Twilight Lighting (Hero Canvas)

| Atmospheric Layer | Coordinates / Position | Color / Gradient Stop | Purpose |
| :--- | :--- | :--- | :--- |
| **Base Sky Wash** | `(0, 0)` to `(0, h)` vertical | `#19355a` (0%) &rarr; `#284b77` (18%) &rarr; `#1d3656` (38%) &rarr; `#14243a` (65%) &rarr; `#0a0a0a` (100%) | Oceanic night sky foundation |
| **Top-Right Silver Shaft** | `(0.94w, 0.04h)` radius `0.82w` | `rgba(165, 185, 208, 0.48)` &rarr; `rgba(115, 145, 180, 0.30)` &rarr; `rgba(0,0,0,0)` | Luminous silver-slate dawn highlight |
| **Center Aura** | `(0.50w, 0.22h)` radius `0.55w` | `rgba(55, 95, 150, 0.42)` &rarr; `rgba(35, 68, 115, 0.26)` &rarr; `rgba(0,0,0,0)` | Soft blue heading halo |
| **Top-Left Navy Depth** | `(0.08w, 0.04h)` radius `0.58w` | `rgba(35, 70, 115, 0.30)` &rarr; `rgba(18, 42, 75, 0.12)` &rarr; `rgba(0,0,0,0)` | Contrast balancing on the left edge |
| **Window Backlight** | `(0.50w, 0.52h)` radius `0.48w` | `rgba(45, 78, 125, 0.26)` &rarr; `rgba(0,0,0,0)` | Ambient horizon glow behind desktop preview window |

```glsl
// DeepSeek Exact WebGL2 Fluid Uniforms
vec3 u_c1 = vec3(0.05, 0.09, 0.16); // Deep oceanic dark base
vec3 u_c2 = vec3(0.10, 0.21, 0.35); // Oceanic twilight navy (#19355a)
vec3 u_c3 = vec3(0.18, 0.32, 0.50); // Sky aura (#2c5180)
vec3 u_c4 = vec3(0.58, 0.69, 0.80); // Silver highlight shaft (#95afcb)
vec3 u_c5 = vec3(0.06, 0.12, 0.20); // Oceanic depth shadow
float u_scale = 0.32;
float u_grain = 0.02;
```

---

### 2.2. Master Surface & Background Palette

| Token Name | Value | Hex / RGBA Equivalent | Usage |
| :--- | :--- | :--- | :--- |
| `--ds-color-bg-page` | `#0a0a0a` | `rgb(10, 10, 10)` | Global document background |
| `--layer-1` | `#232324` | `rgb(35, 35, 36)` | Base elevated container surface |
| `--layer-2` | `#2c2c2e` | `rgb(44, 44, 46)` | Secondary card surface / composer dock |
| `--layer-3` | `#353638` | `rgb(53, 54, 56)` | Elevated flyouts, active states |
| `--ds-color-bg-surface-1` | `hsla(0, 0%, 100%, 0.06)` | `rgba(255, 255, 255, 0.06)` | Secondary buttons, feature card background |
| `--ds-color-bg-surface-2` | `hsla(0, 0%, 100%, 0.04)` | `rgba(255, 255, 255, 0.04)` | Sub-card compartments |
| `--ds-color-bg-surface-3` | `hsla(0, 0%, 100%, 0.02)` | `rgba(255, 255, 255, 0.02)` | Lowest subtle surface |
| `--ds-color-bg-surface-raised`| `hsla(0, 0%, 100%, 0.25)` | `rgba(255, 255, 255, 0.25)` | Highlighted pill badges & tags |
| `--ds-color-bg-code` | `rgba(0, 0, 0, 0.35)` | `rgba(0, 0, 0, 0.35)` | Code blocks, bash inputs, diff boxes |
| `--ds-color-bg-input` | `hsla(0, 0%, 100%, 0.08)` | `rgba(255, 255, 255, 0.08)` | Text fields & composer inputs |
| `--ds-color-bg-overlay` | `#1b1c20` / `#262626` | `rgba(27, 28, 32, 0.95)` | Dropdown menus, modal drawers |

---

### 2.3. Brand & Accent Palette

| Role | Token Name | Hex Code | Purpose / Context |
| :--- | :--- | :--- | :--- |
| **Brand Primary (Dark)** | `--ds-color-brand` | `#6799fe` | Links, active tab markers, primary focus rings |
| **Brand Primary (Light)**| `--ds-color-brand-light` | `#4d6bfe` | Light mode fallback & corporate brand mark |
| **Brand Reverse** | `--ds-color-brand-medium-reverse` | `#3157b8` | Logo fills, hero badge accents |
| **Sky Tint** | `--ds-color-brand-light-reverse` | `#73a3d2` | Subtle logo chip text & secondary glyphs |
| **Business / AI** | `--business` | `#7aaaff` | Tool call tags, AI status indicators |
| **Success** | `--success` | `#59ca84` | Green completion checks, diff added lines |
| **Warning** | `--amber` | `#dd8629` | Amber warning badges, attention callouts |
| **Error / Removed** | `--marker` | `#fa423e` / `#f25a5a` | Red error toasts, diff deletion lines |
| **Violet / Plugin** | `--violet` | `#9474bc` | Plugin category badges, ecosystem markers |

---

### 2.4. Typography & Text Hierarchy

| Level | Token Name | Value | Hex / RGBA Equivalent |
| :--- | :--- | :--- | :--- |
| **Primary Text** | `--ds-color-text-primary` | `#ffffff` / `#f9fafb` | Headings, button labels, high-contrast text |
| **Secondary Text** | `--ds-color-text-secondary` | `hsla(0, 0%, 100%, 0.80)` | Subheadings, navigation items, card text |
| **Description Text**| `--ds-color-text-description`| `hsla(0, 0%, 100%, 0.56)` | Supporting paragraph copy, meta labels |
| **Placeholder** | `--ds-color-text-placeholder`| `hsla(0, 0%, 100%, 0.30)` | Input hints, inactive controls |
| **Mono Primary** | `--dsw-alias-markdown-code` | `#e1e4e8` | Code lines, parameter keys |
| **Mono Dimmed** | `--dsw-alias-label-dimmed` | `rgba(255, 255, 255, 0.45)` | Hero preview label (`✦ PREVIEW`) |

---

### 2.5. Border & Specular Stroke Palette

| Token Name | Value | Usage |
| :--- | :--- | :--- |
| `--ds-color-border-subtle` | `hsla(0, 0%, 100%, 0.06)` | Desktop preview chrome inner separators |
| `--ds-color-border-default` | `hsla(0, 0%, 100%, 0.10)` | Card outlines, card grids, section dividers |
| `--ds-color-border-hover` | `hsla(0, 0%, 100%, 0.20)` | Hover state on cards and interactive tiles |
| `--ds-color-border-strong` | `hsla(0, 0%, 100%, 0.24)` | High-emphasis borders |
| `--ds-color-border-divider` | `hsla(0, 0%, 100%, 0.25)` | Content separators, list dividers |
| `--dsw-alias-border-l2` | `hsla(0, 0%, 100%, 0.12)` | Hairline dividers inside split buttons |

---

## 3. Typography Architecture

### 3.1. Font Stacks

1. **Display & Headings (`--ds-font-display`):**
   ```css
   font-family: "Host Grotesk", system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
   font-feature-settings: 'cv02', 'cv03', 'cv04', 'cv11';
   ```

2. **UI & Body Copy (`--ds-font-sans` / `--ds-font-body`):**
   ```css
   font-family: "DM Sans", system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans SC", "PingFang SC", "Microsoft YaHei", sans-serif;
   ```

3. **Monospace & Code (`--ds-font-mono`):**
   ```css
   font-family: "Fragment Mono", ui-monospace, SFMono-Regular, Menlo, Consolas, "Noto Sans SC", "PingFang SC", "Microsoft YaHei", monospace;
   ```

---

### 3.2. Type Scale & Metric Hierarchy

| Scale Token | Font Size | Line Height | Letter Spacing | Weight | Typical Element |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `ds-text-hero` | `40px` (md) / `32px` (sm) | `1.15` (46px) | `-0.025em` | `600` SemiBold | Main Hero Heading ("Ready to use.") |
| `ds-text-subtitle` | `28px` (md) / `24px` (sm) | `1.25` (35px) | `-0.015em` | `600` SemiBold | Section Headings ("Everything is a plugin") |
| `ds-text-lead` | `18px` | `1.50` (27px) | `-0.01em` | `400` Regular | Hero Lead Copy |
| `ds-text-body` | `15px` | `1.55` (23px) | `normal` | `400` Regular | Body paragraphs, capability explanations |
| `ds-text-ui` | `14px` | `1.40` (20px) | `normal` | `500` Medium | Navigation links, buttons, tab items |
| `ds-text-code` | `13px` | `1.50` (19.5px) | `normal` | `400` Mono | Terminal lines, code snippets, tool args |
| `ds-text-xs` | `12px` | `1.00` (12px) | `0.22em` | `500` Mono/Caps | Hero preview tag (`✦ PREVIEW`) |

---

## 4. Components & Interactive Patterns

### 4.1. Hero Split Download Pill Button

- **Shape:** Full pill radius (`rounded-full` / `100px`)
- **Primary Fill:** `#ffffff` (Solid crisp white)
- **Primary Text:** `#0a0a0a` (Deep black)
- **Height:** `42px` (`ds-btn-m`)
- **Divider:** `1px solid rgba(0, 0, 0, 0.15)`, top 25%, bottom 25% (50% button height)
- **Hover State:** Background shifts to `#f0f0f0`
- **Active State:** Instant `transform: scale(0.98)`

### 4.2. Glass Cards (`ds-glass-card`)

```css
.ds-glass-card {
  background-color: hsla(0, 0%, 100%, 0.04);
  border: 1px solid hsla(0, 0%, 100%, 0.08);
  box-shadow: hsla(0, 0%, 100%, 0.12) 0px 1px 0px 0px inset;
  backdrop-filter: blur(12px);
  border-radius: 24px;
}
```

---

## 5. Spacing & Container Metrics

| Token Name | CSS Value | Pixel Equivalent | Typical Application |
| :--- | :--- | :--- | :--- |
| `ds-container` | `max-w-[1280px]` | `1280px` | Main content bounded width |
| `gap-ds-1` | `4px` | `4px` | Tight inline badges, icon pairings |
| `gap-ds-2` | `8px` | `8px` | Button icon + text gap, dropdown row gap |
| `gap-ds-3` | `12px` | `12px` | List item spacing, card internal badges |
| `gap-ds-4` | `16px` | `16px` | Form groups, composer toolbar controls |
| `gap-ds-6` | `24px` | `24px` | Hero heading to subcopy gap |
| `gap-ds-8` | `32px` | `32px` | Section sub-block spacing |
| `gap-ds-10`| `48px` | `48px` | Major card grid column gap |
| `pt-ds-11` | `80px` | `80px` | Hero top breathing room below nav |
| `py-ds-12` | `96px` | `96px` | Section top/bottom gutter |
