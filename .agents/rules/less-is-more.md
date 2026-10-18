---
description: Design Craft & Restraint Philosophy ("Less is more, more or less" — Simplicity in the Age of AI)
globs: "**/*.{html,css,js,ts,tsx,jsx,vue,svelte}"
---

# Design Craft Rule: Less Is More, More or Less

*Inspired by Emil Kowalski's "Simplicity in the age of AI" and Jony Ive's principles on true simplicity.*

When designing, building, or refining interfaces, typography, animations, and components:

## 1. Removing Is Harder Than Adding
- **"Perfection is achieved, not when there is nothing more to add, but when there is nothing left to take away."** — Antoine de Saint-Exupéry.
- AI agents make adding code, decorative elements, extra cards, and animations effortless. Every addition must be justified by genuine product utility.
- When evaluating any component or section, ask: *Does this make the workflow clearer, or is it just filling space?* If it doesn't add clarity, remove it.

## 2. Processing Fluency & Simplicity
- Human cognition seeks clarity. Interfaces with high processing fluency feel familiar, credible, and effortless.
- **"True simplicity is derived from so much more than just the absence of clutter and ornamentation. It's about bringing order to complexity."** — Jony Ive.
- Structure layouts with crisp hairline grids, intentional whitespace, high-contrast type, and scannable technical metadata.

## 3. Restrained Motion (Function Over Spectacle)
- Animating something and animating something *well* are fundamentally different.
- Never animate high-frequency interactions (context menus, primary clicks, daily workflows) with slow, lingering transitions. If an action is repeated throughout the day, make it immediate.
- Avoid perpetual decorative loops, floating bubbles, and spring bounces that distract from actual content.

## 4. Reserved Technical & Pixel Styling (Never Childish)
- When incorporating pixelated/dotted accents (e.g. `Silkscreen`, dotted borders, dotbar matrices, micro-glyph tiles from Overseer):
  - Keep them **reserved, miniature, and monochrome/subdued**.
  - Use them for technical metadata, step counters (`01`), status indicators, and terminal tabs—never for large toy-like headings or playful cartoon themes.
  - Pair pixel accents with industrial typography: **Instrument Serif** for display, **Geist** for body, and **Geist Mono** for code and technical payloads.

## 5. Anti-Slop Strict Enforcement
- Headings start cleanly and directly. No floating pill eyebrows, no status dot kickers, and no AI-template badges.
- Proof over decoration: showcase real CLI commands, verified checksums, realistic adapter JSON payloads, and authentic architecture plans.
