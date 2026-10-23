# SEO Changelog: Smoooth

- **2026-10-23 (Sprint 02)**:
  - **Live Production Audit**: Probed `https://smoooth.dev/` with curl across root, `/robots.txt`, `/sitemap.xml`, `/og-image.png`, and nonexistent 404 test path. Documented exact HTTP headers and content types: verified that Cloudflare Pages is currently serving the previous SPA deployment (200 OK `text/html`) because local Sprint 01 files have not yet been pushed to GitHub `origin/main`.
  - **Published Guide #1**: Authoritatively published `https://smoooth.dev/guides/deploy-docker-application-claude-code` addressing the emerging high-growth query "deploying docker applications with claude code".
  - **Guide Features**: Concrete end-to-end walkthrough using verified Smoooth MCP tools (`create_plan`, `execute_plan`, `configure_ingress`, `verify_connection`), persistent PostgreSQL volume specs, Caddy TLS automation, error troubleshooting, and comparison vs. insecure raw Docker socket mounting (`/var/run/docker.sock`).
  - **Structured Data**: Injected `TechArticle` and `BreadcrumbList` JSON-LD schemas into the guide.
  - **Clean Routing**: Configured static generator in `scripts/build-landing.js` to create both `dist/guides/deploy-docker-application-claude-code/index.html` and `.html` versions for clean, canonical URLs on Cloudflare Pages.
  - **Sitemap Expansion**: Updated `sitemap.xml` and `frontend/app/sitemap.ts` with canonical URLs for `/guides/` and `/guides/deploy-docker-application-claude-code`.
  - **GitHub CI Automation**: Updated `.github/workflows/deploy-cloudflare.yml` path triggers to ensure pushes to `guides/**`, `scripts/**`, `robots.txt`, and `sitemap.xml` automatically trigger Cloudflare Pages deployments.
  - **Migration Story Telemetry**: Formulated the empirical case study requirements for the Coolify-to-Smoooth founder migration.

- **2026-10-23 (Sprint 01)**:
  - **Anti-Slop Compliance**: Removed decorative pulsing status-dot kicker pill from hero so the `<h1>` starts cleanly and directly.
  - **SERP Title & Meta Optimization**: Optimized `<title>` to 53 chars (`Smoooth — Infrastructure Runtime for AI Coding Agents`) and `<meta name="description">` to 154 chars for zero SERP truncation and maximum CTR.
  - **Homepage CRO**: Implemented dual CTAs in the hero (Primary: "Install on Linux VPS" with quick clipboard copy; Secondary: "See MCP Workflow").
  - **Crawl & Indexing Fixes**:
    - Resolved Cloudflare SPA fallback issue where `/robots.txt` and `/sitemap.xml` returned full HTML documents. Generated static plain-text `robots.txt` and standard XML `sitemap.xml` with zero hash fragments.
    - Added custom lightweight dark-mode `404.html`.
    - Updated `_headers` with strict MIME types and caching headers for `/robots.txt` and `/sitemap.xml`.
  - **Technical Proof Upgrades**: Replaced legacy placeholder illustrations in Capabilities cards with real technical proof: PostgreSQL adapter with volume persistence and healthcheck probe; Scoped security guard with 256-bit token; 2-Phase transactional DAG deployment; and real Model Context Protocol (MCP) stream trace.
  - **Structured Data**: Added `FAQPage` JSON-LD schema matching 5 newly added, visible technical FAQ questions and answers, alongside `SoftwareApplication` and `Organization` schemas.
  - **Mobile Hygiene**: Replaced leftover DeepSeek wordmark in the mobile menu with Smoooth brand identity, removed fake desktop download accordion, and cleaned mobile navigation.
  - **SEO Content Architecture**: Scaffolded `/mcp/`, `/guides/`, `/alternatives/`, and `/compare/` content directories with clear architectural specs.

- **2026-10-19**: Initialized SEO Autopilot setup. Configured canonical metadata, OpenGraph tags, SoftwareApplication JSON-LD schema, robots.txt, and sitemap.xml.
