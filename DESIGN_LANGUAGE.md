# Smoooth — Brand & Landing Page Design Brief

**Document type:** Design direction and implementation brief  
**Audience:** Coding agent implementing Smoooth's public landing page and foundational brand styling  
**Visual source of truth:** The user-supplied reference image in this conversation  

---

## 1. Purpose

Translate the supplied reference into Smoooth's landing-page design and core visual language. The reference image is the visual source of truth: match its editorial layout, typography contrast, fine grid lines, whitespace, restrained palette, and dramatic technical hero composition.

This is not a request to copy the reference company's identity, content, or crypto-related UI. Keep Smoooth's product thesis and content intact.

**Design target:** Editorial, precise, premium, technical, and calm. White canvas. Hairline grid. Oversized serif headline. Small monospace eyebrow. Restrained controls. A visually rich product illustration.

---

## 2. Product context the design must communicate

Smoooth brings Supabase-level convenience to a much broader set of infrastructure choices. Coding agents should be able to choose the technologies that fit a product—databases, containers, orchestration, storage, and other services—then provision, connect, deploy, and operate them through Smoooth's agent-first interface and MCP.

The landing page should communicate freedom of choice plus operational convenience. It should not present Smoooth as merely a VPS deployment tool or another AI code editor.

---

## 3. What to adopt from the reference

- A compact white navigation bar with a thin bottom border.
- An editorial hero split into two columns by a hairline vertical divider.
- A tiny uppercase/monospace eyebrow, optionally with a small green status dot.
- A large, high-contrast serif headline with tight line-height.
- A concise supporting paragraph in the opposite hero column.
- A dark primary button and a quiet outlined secondary button.
- A large technical/product visual spanning the lower hero area, with subtle monochrome terrain/noise texture behind it.
- A strong horizontal boundary between hero and the next section.
- A second section with an editorial heading and two substantial feature cards, each containing a large UI-style visual.
- Neutral surfaces, modest radii, subtle shadows, and generous whitespace.

### What must remain Smoooth

- The Smoooth name, logo/mark, product thesis, copy, and actual capabilities.
- Visual examples about agents selecting and operating infrastructure across technologies.
- Honest capability claims. Never invent customer logos, funding announcements, testimonials, adoption numbers, integrations, or pricing.
- Do not reuse the reference brand name, its exact copy, or its crypto dashboard visuals.

---

## 4. Visual identity system

### Color palette

The reference is predominantly black, white, and neutral gray. A small green accent adds life without making the page colorful. Use this same restraint.

| Token | Starting value | Usage |
| :--- | :--- | :--- |
| Canvas | `#FFFFFF` | Main page and most sections |
| Primary ink | `#111111` | Headlines, primary button, important text |
| Body ink | `#343A40` | Paragraphs and navigation |
| Muted text | `#747B83` | Descriptions and metadata |
| Hairline | `#E4E6E8` | Grid dividers, card borders, navigation separator |
| Soft surface | `#F5F6F6` | Code panes, illustrations, subdued panels |
| Accent green | `#19B981` | Small status dots and rare positive highlights |

These are starting tokens, not immutable requirements. Tune against the reference image and browser screenshots. Avoid teal gradients, neon blue, purple, and multiple competing accents.

### Typography

- **Headlines:** Use a high-quality editorial serif with strong contrast and elegant forms. Evaluate Instrument Serif, DM Serif Display, or an equivalent already available and legally usable.
- **Body and UI:** Use a neutral sans serif such as Geist Sans, Inter, or the project's existing font. Avoid unnecessary font families.
- **Technical labels:** Use restrained monospace for eyebrows, code labels, timestamps, and small technical metadata.
- **Desktop hero headline:** Approximately 48–64px, tight line-height around 0.95–1.05, scaling fluidly down on mobile.
- **Body copy:** Generally 14–17px; smaller navigation and metadata must remain legible.

Do not make every element serif. The contrast between editorial display type and practical sans-serif UI is essential.

### Borders, radii, and shadows

- Use crisp 1px neutral borders. Grid lines are part of the composition, not decoration.
- Use modest radii, typically 6–12px; do not make everything pill-shaped.
- Keep shadows subtle and local to elevated code/product panels or primary controls.
- Avoid glassmorphism, oversized blurred gradients, neon glows, and generic floating dashboard cards.

### Texture

The reference uses a monochrome stippled/terrain texture to give its hero depth. Recreate a restrained black-and-white technical terrain/noise treatment behind the product composition.

- Use CSS, SVG, canvas, or a local optimized asset if it reproduces the look cleanly.
- Concentrate texture around the hero visual; later sections should be calmer.
- Keep it secondary to the interface and away from text where it could hurt contrast.
- Keep it performant and static unless motion adds clear value. Respect reduced-motion preferences.
- Do not substitute a colorful abstract gradient blob.

---

## 5. Landing page structure

### A. Navigation

- White horizontal bar with a fine bottom border.
- Smoooth mark and wordmark on the left.
- Balanced navigation links, such as Product, How it works, Integrations, and Docs—only if these destinations exist.
- Account actions on the right: a quiet Log in action and one clear primary CTA.
- Desktop height around 64–76px.
- On mobile, collapse navigation cleanly without hiding the main CTA.

### B. Hero

Use a two-column grid with a vertical divider at desktop widths.

- **Left column:** Eyebrow, large headline, primary and secondary CTA.
- **Right column:** Short explanatory paragraph aligned with generous whitespace. A row of actual supported integrations or factual proof may sit below it, but never invent social proof.

#### Suggested copy direction
- **Eyebrow:** `INFRASTRUCTURE FOR AI AGENTS`
- **Headline:** Your agent chooses the stack. Smoooth makes it work.
- **Supporting copy:** Give coding agents a consistent way to provision, connect, deploy, and operate the infrastructure your product needs—from databases and containers to the services around them.
- **Primary CTA:** Get started
- **Secondary CTA:** Explore capabilities

This copy is a starting point, not a mandate. Keep the message about freedom plus convenience. Avoid vague claims such as “revolutionary,” “unlimited,” or “the future of everything.”

### C. Hero product composition

This is the signature visual. It should span the lower hero grid and be partially cropped at the next section boundary, echoing the reference composition.

Build a convincing Smoooth-specific scene:
1. **Foreground agent/MCP panel:** Show a request such as “Provision MongoDB + Redis and connect the app,” followed by believable, structured tool activity.
2. **Service overview panel:** Show service status, runtime, database, cache, endpoint, and deployment health.
3. **Background:** Restrained monochrome terrain/noise texture behind and around the UI panels.
4. **Product message:** The composition must show an agent choosing and operating a stack—not merely AI writing code.
5. **Truthfulness:** Prefer actual implemented product UI. If the product UI is not yet built, make the visual clearly illustrative and do not imply that the shown capabilities are live.

Do not use a generic laptop mockup, unrelated mobile banking screen, or AI sparkle graphic. The visual should explain Smoooth at a glance.

### D. Feature section

Start after a strong horizontal boundary. Use a small eyebrow, an editorial heading/description row, then two broad feature cards side by side on desktop.

**Section heading direction:** Everything your agents need to build with fewer limits

- **Card 1 — Architecture without lock-in**
  - **Message:** Let the agent choose a stack that fits the product, within the user's preferences and policies.
  - **Visual:** A clear architecture plan showing a database, cache, object storage, runtime, and their connections.
- **Card 2 — Infrastructure agents can operate**
  - **Message:** Provisioning is only the start: inspect, deploy, observe, update, and recover through MCP.
  - **Visual:** MCP tool activity paired with a resource lifecycle or healthy deployment view.

Cards should have a subtle border, white surface, compact icon tile, strong title, short description, and a substantial visual area. Match the reference's scale and spacing rather than shrinking the cards into small dashboard widgets.

### E. Supporting sections

Add only sections that help explain the product. Maintain the same visual language throughout.
1. **How it works:** Discover → Plan → Provision → Connect → Deploy → Verify → Operate.
2. **Technology and provider capabilities:** Group by role—databases, containers, storage, queues, orchestration, hosting. Show only support that actually exists.
3. **MCP and agent workflow:** Demonstrate structured tools, explicit plans, and inspectable results. MCP itself is not the moat; the useful, reliable infrastructure workflow is.
4. **Control and safety:** Permissions, cost boundaries, approvals for destructive actions, audit trails, and credential handling.
5. **Pricing:** Keep it simple and transparent once pricing is decided. Do not publish invented numbers.
6. **Final CTA and footer:** One clear next step and valid links to docs/status where available.

---

## 6. Component and layout rules

| Component | Rules |
| :--- | :--- |
| **Page grid** | Use a consistent max-width container and explicit columns. Align hairline dividers across sections where possible. |
| **Buttons** | Primary: near-black fill, white text, restrained radius and slight shadow. Secondary: white fill, thin border, dark text. Include clear hover and focus states. |
| **Feature cards** | Large cards with subtle border, white surface, compact icon tile, strong title, short body, and substantial visual area. |
| **Code/MCP panels** | Monospace details, functional tabs only, restrained syntax color, readable line lengths. |
| **Icons** | Use one consistent icon family. Small monochrome icons in subtle bordered squares. |
| **Spacing** | Generous whitespace around headlines; tighter spacing inside controls and code panels. Align elements to the grid, not by visual guesswork. |
| **Motion** | Optional subtle hover/entrance transitions. Avoid scroll-jacking, perpetual animations, and motion that competes with the product visual. |

---

## 7. Responsive behavior and accessibility

Desktop is the primary composition, but the page must work well at 1440px, 1280px, 1024px, 768px, 390px, and 360px widths.
- At tablet/mobile widths, stack the hero columns and remove or reposition vertical dividers without creating awkward empty regions.
- Recompose the hero visual for mobile. Stack or crop panels intelligently; do not simply shrink the desktop layout until everything is tiny.
- Prevent horizontal overflow and clipped controls.
- Feature cards become a single column on mobile.
- Maintain strong contrast, visible keyboard focus, semantic heading order, descriptive link labels, and useful alt text.
- Respect reduced-motion preferences and never communicate status through color alone.

---

## 8. Instructions for the implementation agent

Inspect the repository before editing. Do not assume the framework, routes, component library, or existing design system.

- **Step 1 — Audit:** Identify the framework, routing, current landing page, shared layout, fonts, design tokens, assets, dependencies, and responsive conventions. Briefly report relevant constraints before implementation.
- **Step 2 — Plan:** Map the reference into existing components where practical. Decide which visuals should be real UI components and which need a local asset or CSS/SVG treatment. Avoid unnecessary dependencies.
- **Step 3 — Build the design foundation:** Create a small, coherent token set for color, typography, spacing, borders, radii, and shadows. Load fonts efficiently. Preserve existing project conventions where sensible.
- **Step 4 — Implement the page:** Build the navigation, editorial hero grid, product composition, feature section, supporting sections, and footer. Use Smoooth's actual product message and truthful capabilities. Ensure all visible controls and links work.
- **Step 5 — Responsive pass:** Verify desktop, tablet, and mobile layouts. Ensure the hero visual remains composed rather than merely scaled down.
- **Step 6 — Visual verification:** Run the production build and inspect screenshots at target widths. Compare against the supplied reference for:
  - Overall composition and proportions
  - Grid/divider alignment
  - Serif headline scale and line breaks
  - Whitespace and section spacing
  - Borders, neutral surfaces, and button treatment
  - Hero illustration depth and overlap
  - Feature-card size and content hierarchy
  - Mobile stacking, clipping, and overflow  
  *Fix visible mismatches. Code compiling is not sufficient evidence that the design is done.*
- **Step 7 — Report:** Summarize changed files, key design decisions, commands/tests run, screenshots or preview evidence, and incomplete work. Do not claim checks that were not run.

---

## 9. Non-negotiable do / do not

### Do
- Match the reference's editorial grid, serif headline, fine borders, whitespace, and restrained monochrome palette.
- Make the hero visual explain agent-operated, multi-technology infrastructure.
- Use real Smoooth language: agent-selected stack, infrastructure freedom, provisioning, connections, and lifecycle operations.
- Keep accents rare and meaningful.
- Build a real responsive layout and inspect screenshots.
- Show only verified integrations and capabilities.

### Do not
- Replace the design with a generic dark SaaS dashboard or gradient-heavy AI landing page.
- Copy the reference brand, wording, funding claim, logos, or crypto-specific visual content.
- Use unrelated stock-style device mockups or generic AI sparkle graphics.
- Use neon, multiple gradients, saturated feature cards, or competing accents.
- Invent customers, testimonials, adoption numbers, capabilities, or pricing.
- Declare success based only on a successful compile.

---

## 10. Definition of done

- [ ] The landing page closely resembles the supplied reference in visual structure and design language while remaining unmistakably Smoooth in content.
- [ ] Typography hierarchy is clear: editorial serif headline, clean sans-serif body/UI, compact technical labels.
- [ ] Grid lines and section boundaries align consistently; whitespace feels intentional.
- [ ] The hero visual communicates agent-operated, multi-technology infrastructure rather than generic AI coding.
- [ ] The main CTA works; visible navigation links lead somewhere valid or are omitted until destinations exist.
- [ ] No false claims, fake customer logos, fabricated metrics, or placeholder crypto UI remain.
- [ ] Desktop and mobile layouts pass the target viewport checks without overflow or broken cropping.
- [ ] Production build and relevant tests pass; visual verification evidence is included in the implementation report.

---

### Final instruction to the agent

Use the uploaded reference as the visual north star. Preserve its restraint and composition. Translate the design language to Smoooth's actual thesis: agents choosing and operating infrastructure across technologies. Do not redesign the idea; execute it with care.
