# Smoooth — Brand Guidelines & Media Kit
*Version 1.0.0 • Published October 2026*

---

## 1. Brand Narrative & Positioning

### Mission
**Infrastructure freedom for AI coding agents.**

Smoooth is the headless autonomous deployment runtime built for the AI coding generation (Claude Code, Cursor, Windsurf). We replace overpriced, ephemeral PaaS platforms with dedicated, persistent, self-hosted Linux VPS environments orchestrated entirely through Anthropic's **Model Context Protocol (MCP)**.

### Target Audience
- **Autonomous AI Developers**: Engineers running Claude Code or Cursor who want real deployments, not ephemeral sandbox previews.
- **Solo SaaS Founders**: Builders needing real PostgreSQL, Redis, custom background workers, and SSL without \$500/month cloud markups.
- **AI Infrastructure Consultants**: Agencies delivering sovereign, self-contained client applications directly to client VPS hardware.

---

## 2. Design Craft Standard & Anti-Slop Directives

Every Smoooth asset, page, document, and interface strictly enforces the **Anti-Slop Rule** and **Design Craft Standard**:

### 🚫 The Anti-Slop Rules
1. **No Decorative Eyebrow Badges**: Never place rounded-full pill tags, pastel kicker tags, sparkle icons, or uppercase tracking-wide status pills above headings or sections. Headings must start cleanly and directly.
2. **No Fake Social Proof**: Never fabricate seed round announcements, invented customer logos, fake metrics, or filler buzzwords.
3. **Proof Over Decoration**: Always display real terminal commands, real JSON-RPC adapter payloads (PostgreSQL, Redis, Mongo, SQLite), real MCP tool calls, and verifiable sha256 checksums.
4. **Immediate Interactions**: Interactive elements must respond immediately (hover, active/pressed `scale(0.97)`, visible high-contrast focus rings). No sluggish or distracting decorative animations.

---

## 3. Typography System: Lexend

Smoooth uses **Lexend** as its primary typeface across all headings, body text, and UI controls, paired with a clean monospace font for code and terminal output.

### Primary Typeface: Lexend
- **Designer**: Thomas Jockin (Lexend Project, Google Fonts)
- **Role**: Brand identity, display headings, navigation, long-form prose, and UI labels.
- **Weights**: 
  - **300 Light**: Subdued subtitles, large decorative counters
  - **400 Regular**: Default body text, documentation prose
  - **500 Medium**: Button labels, interactive triggers, table headers
  - **600 SemiBold**: Standard headings (H1–H4), bento titles
  - **700 Bold**: Hero title emphasis, display numerals

### Monospace Companion: ui-monospace / Geist Mono
- **Role**: Terminal prompts, CLI instructions, MCP JSON-RPC schemas, environment configurations.
- **Weights**: 400 Regular, 500 Medium.

### Typographic Scale & Rhythm
| Token | Font Size | Line Height | Letter Spacing | Weight | Usage |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `display-2xl` | 80px (5.0rem) | 1.00 | -0.035em | 600/700 | Hero splash (desktop) |
| `display-xl` | 64px (4.0rem) | 1.02 | -0.035em | 600 | Page hero headings |
| `display-lg` | 48px (3.0rem) | 1.08 | -0.030em | 600 | Section titles |
| `display-md` | 36px (2.25rem) | 1.15 | -0.025em | 600 | Bento grid major titles |
| `title-lg` | 24px (1.5rem) | 1.20 | -0.020em | 600 | Feature card titles, modal headers |
| `title-md` | 20px (1.25rem) | 1.25 | -0.020em | 500/600 | Sub-feature headers |
| `body-lg` | 18px (1.125rem) | 1.45 | 0 | 400 | Hero lede descriptions |
| `body-base` | 15px (0.9375rem) | 1.50 | 0 | 400 | General reading copy |
| `caption` | 13px (0.8125rem) | 1.40 | +0.010em | 400/500 | Metadata, timestamps, footnotes |
| `mono-code` | 13px (0.8125rem) | 1.45 | 0 | 400 | CLI lines, JSON keys, status logs |

---

## 4. Color System & Design Tokens

Smoooth uses an obsidian canvas with high-voltage spark accents and bone-white neutrals, engineered for high contrast and developer clarity.

### Color Swatches & Contrast Table
| Color Name | Hex Code | RGB | CSS Variable | Contrast on `#0a0b0d` | Approved Usage |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Spark** | `#c9f27a` | `201, 242, 122` | `--spark` | 14.8:1 (AAA) | Primary interactive accents, prompt symbols (`$`), focused borders |
| **Fern 600** | `#00f076` | `0, 240, 118` | `--fern-600` | 12.1:1 (AAA) | Live daemon indicator, successful migration status, active badges |
| **Spark Ink** | `#4e7a14` | `78, 122, 20` | `--spark-ink` | - | Text rendered directly on top of `#c9f27a` buttons |
| **Ink 950** | `#0a0b0d` | `10, 11, 13` | `--ink-950` | 1.0:1 | Root dark canvas background |
| **Ink 900** | `#121417` | `18, 20, 23` | `--ink-900` | 1.1:1 | Surface level 1: terminal cards, bento containers |
| **Ink 800** | `#1a1e24` | `26, 30, 36` | `--ink-800` | 1.3:1 | Surface level 2: popovers, active tab headers, elevated cards |
| **Border Soft** | `rgba(255,255,255,0.08)` | - | `--border-soft` | - | Hairline card boundaries, dividers |
| **Border Strong** | `rgba(255,255,255,0.16)` | - | `--border-strong` | - | Hovered card boundaries, active inputs |
| **Bone (Primary)** | `#f3f4f6` | `243, 244, 246` | `--bone` | 18.2:1 (AAA) | Primary headings, active navigation, emphasized values |
| **Stone (Muted)** | `#9ca3af` | `156, 163, 175` | `--on-ink-muted`| 7.2:1 (AAA) | Body copy, secondary descriptions, inactive tabs |
| **Stone (Faint)** | `#4b5563` | `75, 85, 99` | `--on-ink-faint`| 3.1:1 (AA-18pt) | Technical labels, table headers, breadcrumbs |
| **Terminal Amber** | `#f59e0b` | `245, 158, 11` | `--ember` | 8.9:1 (AAA) | Warnings, pending DAG locks, verification checks |
| **Glacier Cyan** | `#06b6d4` | `6, 182, 212` | `--glacier` | 9.4:1 (AAA) | Ingress routing, SSL/TLS handshake, MCP discovery links |

---

## 5. Logo Specifications & Usage Guidelines

> [!NOTE]
> **User Logo Integration**:
> The official logo vector asset will be provided by the user. The specifications below govern clearance, sizing, alignments, and background contrasts.
> To add or update the official logo, replace `assets/brand/logo-spec-dark.svg` and `assets/brand/logo.svg`.

### Clear Space Rule
- **Safe Zone**: Maintain clear space equal to `1.0X` around all edges of the mark and lockup, where `X` equals the cap-height of the initial letter **S** in the wordmark.
- No text, UI borders, images, or navigational elements may intrude into this clearance box.

### Minimum Sizes
- **Mark Only**:
  - Digital Display: `24px × 24px` (e.g. browser favicons, terminal tray icons).
  - Print / Physical: `7mm × 7mm`.
- **Full Lockup (Mark + 'Smoooth')**:
  - Digital Display: `120px` width minimum (height `28px`).
  - Print / Physical: `35mm` width minimum.

### Approved Colorways
1. **Primary Dark Canvas**: Spark (`#c9f27a`) or Fern (`#00f076`) mark, Bone (`#f3f4f6`) wordmark on Ink 950 (`#0a0b0d`).
2. **Light / Monochrome**: Obsidian mark and wordmark (`#0a0b0d`) on Pure White (`#ffffff`) or Bone (`#f3f4f6`).
3. **Monochrome Inverted**: Pure White (`#ffffff`) on dark backgrounds when color reproduction is restricted.

### 🚫 Logo Misuse (Strict Prohibitions)
- ❌ **DO NOT** stretch, squeeze, skew, or distort the aspect ratio.
- ❌ **DO NOT** add drop shadows, outer glows, bevels, or 3D gradient fills.
- ❌ **DO NOT** place inside decorative rounded pill kickers or badges.
- ❌ **DO NOT** alter the spacing between the mark and the wordmark.
- ❌ **DO NOT** recolor the mark in unapproved hues (e.g. magenta, purple, warm yellow).
- ❌ **DO NOT** place over busy photo backgrounds without an opaque dark scrim.

---

## 6. Press & Media Kit Boilerplates

Use these approved summaries in news articles, directory listings, podcast show notes, and conference programs:

### One-Sentence Boilerplate (21 words)
> **Smoooth is an autonomous deployment runtime that allows AI coding agents to provision and manage stateful apps on self-hosted Linux VPS.**

### 50-Word Boilerplate (48 words)
> **Smoooth gives AI coding agents like Claude Code, Cursor, and Windsurf direct infrastructure power. Rather than deploying ephemeral apps to high-markup PaaS sandboxes, Smoooth connects over the Model Context Protocol (MCP) to orchestrate persistent Docker containers, automated PostgreSQL databases, and self-healing rollbacks on the developer's own Linux VPS.**

### 100-Word Boilerplate (96 words)
> **As AI coding agents rapidly write production software, deployment remains bottlenecked by manual devops or restricted to proprietary serverless platforms. Smoooth eliminates this divide by acting as a native Model Context Protocol (MCP) deployment daemon for Linux servers.**
>
> **Developers simply run their preferred coding agent. Smoooth analyzes application architecture, generates a verified DAG deployment graph, provisions dedicated Docker containers with persistent storage and PostgreSQL, configures reverse-proxy ingress with automated SSL, and executes atomic zero-downtime health rollbacks. With Smoooth, engineers retain 100% data sovereignty and cloud cost savings without writing Terraform.**

---

## 7. Fast Facts & Metadata

| Field | Detail |
| :--- | :--- |
| **Product Name** | Smoooth (triple 'o') |
| **Category** | Developer Tools • Cloud Infrastructure • AI Agent Runtimes |
| **Founding Year** | 2026 |
| **License / Protocol** | Model Context Protocol (Anthropic MCP standard) |
| **Host System Requirements** | Linux (Ubuntu 22.04/24.04 LTS, Debian 12) with Docker 24+ |
| **Supported Agent Clients** | Claude Code, Cursor, Windsurf, Roo Code, Goose |
| **Website** | [https://smoooth.dev](https://smoooth.dev) |
| **Press & Inquiries** | `press@smoooth.dev` |

---

## 8. Asset Directory Index

All brand files and downloadable assets reside in the repository at `/assets/brand/`:
- `tokens.json`: Machine-readable tokens (colors, typography, spacing, radii)
- `tokens.css`: Canonical CSS custom properties stylesheet
- `logo-spec-dark.svg`: Dark mode vector logo specification & clear-space geometry
- `logo-spec-light.svg`: Light mode vector logo specification
- `smoooth-badges.md`: Markdown badge templates for GitHub projects
- `BRAND_GUIDELINES.md`: This comprehensive standard document
