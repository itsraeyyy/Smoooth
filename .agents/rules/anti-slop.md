---
description: Anti-Slop Rule and Clean UI/UX Design Standards from Andebet
globs: "**/*.{html,css,js,ts,tsx,jsx,vue,svelte}"
---

# Anti-Slop Rule & Clean Frontend Design Standards

When designing, building, or refactoring UI components, landing pages, documentation, or user flows:

1. **Anti-Slop Rule (Strictly Enforced)**:
   - **NEVER use decorative pill-shaped eyebrow badges or kicker pills** (`rounded-full`, pastel/tinted backgrounds, sparkle icons, uppercase tracking-wide small text, status dot pills) floating above headings, heroes, cards, or sections.
   - **Headings must start cleanly and directly.** Let typography, scale, contrast, and layout carry the visual hierarchy rather than AI-template decoration.

2. **Structure & Craft (from design-engineer skill)**:
   - **Layout logic first**: Use hairline borders, explicit rails, clean grids, and thoughtful gutters.
   - **Proof over decoration**: Showcase real technical payloads, verifiable commands, architecture plans, interactive code windows, and truthful workflow states.
   - **Accessible states**: Every interactive control must have distinct hover, focus-visible, and active/pressed feedback (`transform: scale(0.98)`).
   - **Touch & Mobile**: Maintain minimum 44x44px touch targets on mobile and prevent horizontal overflow.
