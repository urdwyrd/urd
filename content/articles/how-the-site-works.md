---
title: How the Site Works
slug: how-the-site-works
description: The tech stack behind urd.dev — Astro, Svelte islands, Tailwind v4, Cloudflare Pages, and a design system that keeps everything coherent.
date: "2026-02-17"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> A walkthrough of the technologies and architecture behind urd.dev.
> Single canonical copy. February 2026.

## The stack

The site is a static Astro 5 project that deploys to Cloudflare Pages. Content lives in Markdown. Interactivity comes from Svelte 5 islands that hydrate only when needed. Styling is Tailwind CSS v4 driven by a design system that lives in the repository alongside the code.

No SSR. No database. No CMS. The repository is the single source of truth for everything — content, design tokens, components, and deployment.

Here is how the pieces fit together.

## Astro: static first

[Astro](https://astro.build) generates the entire site at build time. Every page is pre-rendered HTML with CSS inlined or linked. JavaScript is only shipped for the interactive islands — and even those are deferred.

The [Astro config](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/astro.config.mjs) is minimal: Svelte integration for islands, a sitemap generator, `rehype-slug` for heading anchors, and the Tailwind v4 Vite plugin. No custom Vite config, no SSR adapter, no middleware.

Content is organised into six [Astro collections](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/content.config.ts) — articles, design documents, updates, reviews, timeline entries, and per-document reviews. Each collection loads Markdown files from `content/` using glob patterns and validates frontmatter against Zod schemas. The schemas enforce required fields, valid categories, and correct types at build time. A document missing its `slug` or an update with a malformed `date` fails the build, not the reader.

The site has three page types:

- **The homepage** (`index.astro`) — static HTML shell with five Svelte islands for the interactive sections
- **Articles** (`articles/[slug].astro`) — rendered Markdown with almost no JavaScript (one "back to top" button handler)
- **Documents** (`documents/[slug].astro`) — rendered Markdown with a per-document review island

Article and document pages are effectively zero-JS. The homepage carries the interactive weight, and even there, every island defers its load.

## Svelte islands: interactive where it matters

Six Svelte 5 components provide the site's interactivity:

| Component | What it does | Hydration |
|-----------|-------------|-----------|
| [Presentation](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/components/Presentation.svelte) | Full-screen walkthrough of the project | `client:idle` |
| [DocumentExplorer](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/components/DocumentExplorer.svelte) | Browse, filter, and search design documents | `client:visible` |
| [ProjectLog](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/components/ProjectLog.svelte) | Chronological project updates | `client:visible` |
| [ProjectTimeline](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/components/ProjectTimeline.svelte) | Phase progress (complete/active/next) | `client:visible` |
| [PeerReview](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/components/PeerReview.svelte) | AI model review cards with star ratings | `client:visible` |
| [DocumentReviews](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/components/DocumentReviews.svelte) | Per-document AI reviews below document content | `client:visible` |

Two client directives keep the JavaScript budget under control. `client:idle` means the Presentation component does not hydrate until the browser's main thread is idle — it is the heaviest island and there is no rush to load it. `client:visible` means the other five islands only hydrate when they scroll into the viewport. If the reader never scrolls to the document explorer, its JavaScript never loads.

Each data-fetching island follows the same pattern: Astro generates a JSON endpoint at build time (e.g., [`/documents.json`](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/pages/documents.json.ts), [`/updates.json`](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/pages/updates.json.ts)), and the Svelte component fetches it on mount. The JSON is static — baked into the build output alongside the HTML. No API server, no runtime data fetching. The island pattern keeps the interactivity isolated: filter a document list, expand a timeline entry, navigate a presentation. The surrounding page remains static HTML.

## Tailwind v4: CSS-first configuration

The site uses [Tailwind CSS v4](https://tailwindcss.com/blog/tailwindcss-v4) via the `@tailwindcss/vite` plugin — not the older `@astrojs/tailwind` integration, which does not support v4.

There is no `tailwind.config.js`. Tailwind v4 moves configuration into CSS itself. All design tokens are CSS custom properties on `:root` in [`global.css`](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/styles/global.css). Colours, fonts, spacing, animation — everything the components need is defined once in CSS and consumed everywhere.

The approach means the design system and the implementation are the same file. Change `--gold` from `#dab860` to something else, and every component that uses it updates. No build step translation. No config-to-CSS mapping. The token *is* the value.

## The design system

Before any visual code was written, the design was specified. The [`design/`](https://github.com/urdwyrd/urd/tree/main/design) directory contains the source of truth for every visual decision.

### The design brief

The [Gloaming design brief](https://github.com/urdwyrd/urd/blob/main/design/themes/gloaming/design-brief.md) is a 600-line specification covering colours, typography, spacing, animation, layout, and component anatomy. It is not a mood board. It is a contract.

Key decisions locked in the brief:

**Colour.** Navy background (`#0e0f16`), not pure black. Warm text (`#f2ece0`), not white. Seven category colours — gold for Urd/schema, purple for Wyrd/runtime, blue for authoring, green for validation, amber for architecture, rose for research, teal for briefs. Each colour has three variants: full, dim, and light.

**Typography.** Four typefaces, each with a single permitted use:

| Typeface | Use | Never |
|----------|-----|-------|
| Georgia | Brand wordmark only (URD · WYRD) | Headings, body, anything else |
| Outfit | Headings, labels, navigation | Body text |
| Source Serif 4 | Body text, long-form reading | Headings |
| JetBrains Mono | Code, technical values | Body text |

All four are self-hosted as WOFF2 files in [`public/fonts/`](https://github.com/urdwyrd/urd/tree/main/sites/urd.dev/public/fonts) with `@font-face` declarations in `global.css`. No Google Fonts links. No external requests. The fonts load with `font-display: swap` so the page renders immediately with system fonts, then swaps in the custom typefaces without layout shift.

**Icons.** No icon libraries. Text characters only: → ◆ ▸ ↗. This keeps the dependency count at zero and the visual language typographic rather than illustrative.

**Animation.** Rune canvas background opacity hard-capped at 0.05. All animations respect `prefers-reduced-motion`. The brief specifies `fadeUp` and `fadeIn` keyframes with exact timing — `global.css` implements them verbatim.

### The document taxonomy

The [document taxonomy](https://github.com/urdwyrd/urd/blob/main/design/urd-document-taxonomy.md) defines a colour system for the eight document categories: research, strategy, contract, authoring, architecture, runtime, validation, and brief. Each category gets a hue mapped to CSS custom properties (`--doc-research`, `--doc-contract`, etc.) that the DocumentExplorer and document pages use for colour-coding.

This is how the site maintains visual coherence across dozens of documents without manual colour assignments. Add a new document with `category: "validation"`, and it automatically inherits the green colour system.

### From brief to CSS

The flow is:

1. The design brief specifies a token (e.g., "gold: `#dab860`, used for Urd, schema, and contract elements")
2. `global.css` declares it: `--gold: #dab860;`
3. Components consume it: `color: var(--gold);`

The brief is the specification. The CSS is the implementation. The components are the consumption. Nothing is invented at the component level — every colour, every font choice, every spacing value traces back to the design brief.

A future light theme (Parchment) is already stubbed in `global.css` behind a `[data-theme="parchment"]` selector. When it ships, it will override the same CSS custom properties with light-theme values. Every component will adapt without changes because they reference tokens, not hard-coded colours.

## Fonts: self-hosted, no external requests

Eight WOFF2 files cover the four typefaces with Latin and Latin Extended character sets:

- **Outfit** — two subsets (latin, latin-ext), variable weight 300–700
- **Source Serif 4** — four subsets (latin, latin-ext, italic latin, italic latin-ext), variable weight 300–600
- **JetBrains Mono** — two subsets (latin, latin-ext), variable weight 300–500

The `@font-face` declarations use `unicode-range` to load only the glyphs needed. For English content, the Latin subset loads first. Latin Extended loads only if the page contains characters outside the basic Latin range.

This matters for a project that will run in clinical settings. External font requests are a privacy concern, a performance concern, and a reliability concern. Self-hosting eliminates all three.

## GitHub Actions: path-filtered deployment

Two workflows run on every push to main:

### Deployment ([`deploy.yml`](https://github.com/urdwyrd/urd/blob/main/.github/workflows/deploy.yml))

Triggers on pushes to main that touch `sites/urd.dev/**`, or on manual dispatch. The workflow installs pnpm, builds the Astro site, and deploys the static output to Cloudflare Pages using Wrangler.

The path filter is important in a monorepo. Changes to `packages/grammar/` or `briefs/` do not trigger a deployment. Only changes to the site itself — layouts, components, styles, content — cause a build and deploy. This keeps the deployment pipeline fast and focused.

### Secret scanning ([`secret-scan.yml`](https://github.com/urdwyrd/urd/blob/main/.github/workflows/secret-scan.yml))

Runs on every push and every pull request. Uses [gitleaks](https://github.com/gitleaks/gitleaks) to scan for accidentally committed secrets. This runs in addition to the local pre-commit hook — defence in depth.

## Cloudflare Pages: static hosting

The deployment target is Cloudflare Pages, configured via a three-line [`wrangler.toml`](https://github.com/urdwyrd/urd/blob/main/wrangler.toml):

```toml
name = "urd-dev"
pages_build_output_dir = "sites/urd.dev/dist"
```

No Workers. No serverless functions. No KV stores. The site is static HTML, CSS, and JavaScript served from Cloudflare's edge network. The simplicity is deliberate — fewer moving parts means fewer failure modes.

Secrets (`CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`) live in GitHub Actions secrets, never in the repository. The gitleaks hook and CI scan enforce this.

## Content architecture: Markdown in, JSON out, Svelte renders

The content pipeline has three stages:

**Stage 1: Markdown files.** Writers and the development journal produce Markdown files with YAML frontmatter. These live in `content/` under collection subdirectories (articles, documents, updates, etc.).

**Stage 2: JSON endpoints.** At build time, Astro reads the collections, validates frontmatter against Zod schemas, and generates static JSON files. The [documents endpoint](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/pages/documents.json.ts) enriches documents with word counts, reading times, excerpts, review ratings, and GitHub URLs. The [updates endpoint](https://github.com/urdwyrd/urd/blob/main/sites/urd.dev/src/pages/updates.json.ts) produces a sorted feed with optional article links.

**Stage 3: Svelte islands.** Each interactive component fetches its JSON endpoint on mount and renders the data. The DocumentExplorer provides filtering, search, and view mode switching. The ProjectLog renders the update timeline with linked entries. The data is static but the interaction is dynamic.

This three-stage pipeline means content authors never touch JavaScript. Add a Markdown file with the right frontmatter, and the build pipeline picks it up, validates it, enriches it, and makes it available to the appropriate island.

Design documents additionally maintain a two-copy convention: `content/documents/` holds the version with Astro frontmatter, and `docs/` holds a clean Markdown version without frontmatter for GitHub viewing and direct download. Both must be kept in sync when documents change.

## What is not here

A few things the site deliberately does not use:

- **No SSR.** Every page is pre-rendered. There is no server to run, no cold starts, no runtime costs.
- **No CMS.** Content is files in a repository. Version control is the editing history.
- **No analytics.** No tracking scripts, no cookie banners, no third-party requests (apart from Cloudflare's edge network).
- **No icon libraries.** Typography-only icons (→ ◆ ▸ ↗) keep the dependency graph clean.
- **No Google Fonts.** All typefaces are self-hosted WOFF2 files.

Each absence is a decision, not an oversight. The site is built for a project that will run in clinical settings. Every external dependency is a risk. Every tracking script is a privacy question. Every server is a failure mode. Static generation, self-hosted assets, and minimal JavaScript are not constraints — they are the architecture.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
