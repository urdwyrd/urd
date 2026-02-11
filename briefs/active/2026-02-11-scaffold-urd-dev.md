# Brief: Scaffold urd.dev in the Urd Monorepo

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:**
**Status:** Complete | Partial | Blocked

### What was done
- (List what was actually built, committed, and configured)

### Deviations from brief
- (Anything that diverged from the plan, and why)

### Issues encountered
- (Problems hit during execution, workarounds applied)

### Notes for next brief
- (Observations, suggestions, things noticed that should inform future work)
- **Reminder:** Set up a pre-commit hook for secret scanning (e.g., `git-secrets` or `secretlint`) before any deployment configuration or API keys enter the project. GitHub's built-in secret scanning is active on public repos, but a local hook catches mistakes before they're pushed.

---

## Context

This is the `urd` monorepo at `https://github.com/urdwyrd/urd`. It currently contains only a README.md and LICENSE file. Urd is a declarative schema system for interactive worlds — a pipeline where writers author in Schema Markdown (`.urd.md`), a compiler produces `.urd.json`, and runtimes execute it. The project is in its early stages: the design phase is complete (nine design documents exist as .docx files), but no code has been written yet.

The first implementation task is building **urd.dev** — a development journal website that transparently documents progress and showcases the technical artifacts produced during the design phase.

---

## Monorepo Structure

Create the following directory structure:

```
urd/
├── .github/
│   └── banner.png                    # Repo banner (added manually later)
├── briefs/                            # ★ AI task briefs — transparent build record
│   ├── active/                        # Brief currently being executed
│   ├── done/                          # Completed briefs (moved here after execution)
│   └── backlog/                       # Future briefs and ideas
├── design/                            # ★ Design system — the AI reference library
│   ├── urd-document-taxonomy.md       # Universal document colour taxonomy
│   └── themes/
│       ├── gloaming/
│       │   ├── design-brief.md        # Gloaming theme canonical reference
│       │   └── design-system.html     # Living visual reference (viewable in browser)
│       └── parchment/
│           ├── design-brief.md        # Parchment theme canonical reference
│           └── design-system.html     # Living visual reference
├── docs/                              # Design documents (markdown, added later)
├── packages/                          # Empty for now — future compiler, runtime, etc.
├── sites/
│   └── urd.dev/                       # Astro project (this task)
├── .gitignore
├── pnpm-workspace.yaml
├── package.json                       # Root workspace package.json
└── README.md                          # Already exists
```

### Why `/design` Lives at the Repo Root

The `/design` folder is the **AI reference library** — the single location any AI tool (Claude Code, Cursor, Copilot, or any future assistant) should read before producing any visual code, component, or design decision for the project. It sits at the repo root (not inside `sites/urd.dev/`) because:

1. **It governs more than one site.** Both urd.dev and urd.world (and any future product surface) share the same taxonomy and theme system.
2. **It's not a build dependency.** The design briefs and HTML references are documentation for humans and AI. They are never imported, bundled, or processed by Astro.
3. **It's findable.** An AI agent exploring the repo encounters `/design` at the top level, alongside `/docs`, `/briefs`, and `/sites`. The convention is: `/docs` = project content, `/design` = visual identity, `/briefs` = AI task history, `/sites` = implementations.
4. **Theme files travel together.** Each theme is a self-contained folder with its brief (the specification) and its design system HTML (the living reference). Adding a third theme means adding a third folder.

### The `/briefs` Folder

Every AI task brief lives here — transparent, versioned, and part of the public record. This is consistent with the project's core value of building in public.

- **`active/`** — the brief currently being executed. Only one at a time.
- **`done/`** — completed briefs, moved here after execution. The AI can read these for context on what's already been built.
- **`backlog/`** — future briefs and ideas, not yet ready for execution.

Filename convention: `YYYY-MM-DD-short-description.md` (e.g., `2026-02-11-scaffold-urd-dev.md`). When complete: `git mv briefs/active/... briefs/done/...`.

### Workspace Setup

- Use **pnpm workspaces** as the monorepo package manager
- Root `pnpm-workspace.yaml` should include `sites/*` and `packages/*`
- Root `package.json` should be private, with workspace-level scripts:
  - `pnpm dev` → `pnpm --filter urd.dev dev`
  - `pnpm build` → `pnpm --filter urd.dev build`
  - `pnpm preview` → `pnpm --filter urd.dev preview`
- Root `.gitignore` must cover:

```gitignore
# Node / JS
node_modules/
dist/
.astro/

# Rust (for later)
target/
Cargo.lock

# General
.env
.DS_Store
*.log
```

---

## Design System Files

### Manual Placement (Not Part of This Brief's Execution)

The design system files are authored artifacts that will be **placed manually** by the project owner into the `/design` directory structure. The AI should create the empty directory structure but not populate it.

The files that will be placed:

| File | Destination |
|------|-------------|
| Document colour taxonomy | `design/urd-document-taxonomy.md` |
| Gloaming theme brief | `design/themes/gloaming/design-brief.md` |
| Gloaming design system | `design/themes/gloaming/design-system.html` |
| Parchment theme brief | `design/themes/parchment/design-brief.md` |
| Parchment design system | `design/themes/parchment/design-system.html` |

### How AI Should Use These Files (In Future Briefs)

**Before writing any visual code**, read:
1. `design/themes/gloaming/design-brief.md` — the canonical reference for all colour, typography, spacing, animation, copy voice, and layout decisions
2. `design/urd-document-taxonomy.md` — the six-category colour system for documents

The design brief is the specification. It is the single source of truth for all visual decisions.

---

## Astro Project: `sites/urd.dev`

### Setup

- **Astro** with **TypeScript** (strict mode)
- **Tailwind CSS v4** for utility classes, but with the full colour system defined as CSS custom properties (not Tailwind config) to match the design brief's token system
- **Svelte** integration installed but not yet used (will power interactive islands in future briefs)
- No SSR — fully static output (Astro's default). Will deploy to Cloudflare Pages

### Visual Identity: The Gloaming Theme

The default (and initially only) theme is **Gloaming**. All design decisions are specified in `design/themes/gloaming/design-brief.md`. Key points for implementation:

#### Colour System (CSS Custom Properties)

Define these as CSS custom properties on `:root`. These are the exact tokens from the design brief:

```css
/* Surface */
--bg: #0e0f16;
--raise: #14151e;
--deep: #0a0a12;
--surface: #1a1b25;
--border: #28273a;
--border-light: #353348;

/* Text */
--text: #f2ece0;
--dim: #d4cbb8;
--faint: #9e9585;

/* Brand + Category Colours */
--gold: #dab860;
--gold-light: #f0d78c;
--gold-dim: #a08a58;
--gold-dark: #5a4e35;

--purple: #b090dd;
--purple-light: #ccb0f0;
--purple-dim: #1f1530;

--blue: #6a9acc;
--blue-light: #90bbdd;
--blue-dim: #1a2535;

--green: #70b080;
--green-light: #a0d8aa;
--green-dim: #1a2a1f;

--amber: #cc7a3a;
--amber-light: #e8a060;
--amber-dim: #2a1a10;

--rose: #cc8888;
--rose-dim: #2a1515;

/* Document taxonomy (maps category → colour) */
--doc-research: var(--rose);
--doc-research-dim: var(--rose-dim);
--doc-contract: var(--gold);
--doc-contract-dim: #1e1a0e;
--doc-authoring: var(--blue);
--doc-authoring-dim: var(--blue-dim);
--doc-architecture: var(--amber);
--doc-architecture-dim: var(--amber-dim);
--doc-runtime: var(--purple);
--doc-runtime-dim: var(--purple-dim);
--doc-validation: var(--green);
--doc-validation-dim: var(--green-dim);
```

#### Typography

```css
--wordmark: 'Georgia', 'Times New Roman', serif;
--display: 'Outfit', 'Helvetica Neue', sans-serif;
--body: 'Source Serif 4', 'Georgia', serif;
--mono: 'JetBrains Mono', 'Consolas', monospace;
```

Import from Google Fonts: Outfit (300–700), Source Serif 4 (300–600, italic), JetBrains Mono (300, 400, 500). Georgia ships with every OS — no import needed.

**Critical typography rules:**
- Georgia is **exclusively** for the brand wordmark (`URD · WYRD`, uppercase, 0.08em letter-spacing, gradient text fills)
- Outfit for all headings, labels, badges, component names — **never** for body text or the wordmark
- Source Serif 4 for all body text, descriptions, prose — minimum 16px, preferred 20px
- JetBrains Mono for code, metadata, file paths, dates, technical badges

#### Animations

Only these animations exist. Nothing else:
- `fadeUp` — 0.5–0.6s ease-out, staggered. Page load elements.
- `fadeIn` — 0.25s ease-out. Expanding card content.
- `pulse` / `pulseRing` — 2.5s ease-in-out infinite. Status dots only.
- `wyrdGlow` — 4s infinite. **Wyrd component card only.** Nothing else glows.
- `float` — 3s ease-in-out infinite. Scroll indicator only.
- Rune canvas — continuous, full-page background, opacity ≤ 0.05.

---

## Document Data Model (Reference — Not Built in This Brief)

The design documents currently exist as .docx files and will be converted to markdown in a separate follow-up task. This section documents the target frontmatter schema and manifest format so that future briefs have a clear specification to work from.

### Document Frontmatter Schema

Each markdown document in `/docs` will have YAML frontmatter:

```yaml
---
title: "Urd World Schema Specification v0.1"
slug: "schema-spec-v01"
description: "The formal data contract. Defines entities, typed properties, containment hierarchies, visibility layers, conditional expressions, effects, behavioural rules, dialogue sections, and sequence phases."
category: "contract"            # One of: research | contract | authoring | architecture | runtime | validation
format: "Technical Specification"  # Freeform label: "Research Report", "System Blueprint", "Design Document", etc.
date: "2026-02"                 # Year-month
status: "v0.1 complete"         # Freeform: "v0.1 complete", "in progress", "draft"
order: 1                        # Display order within category
tags:
  - schema
  - v0.1
  - entities
  - containment
  - visibility
details:                        # Key content bullets (shown in expanded card)
  - "Entity-property-behaviour model with hidden/revealed state"
  - "Containment as the universal spatial primitive"
  - "Condition expressions and effect grammar"
  - "Rule system: trigger → condition → select → effect"
---
```

### Generated Manifest: `documents.json`

At build time (once the content pipeline is wired up), a static JSON manifest will be generated from all documents. This file gets served from the CDN and loaded by the client-side interactive index.

```json
[
  {
    "slug": "schema-spec-v01",
    "title": "Urd World Schema Specification v0.1",
    "description": "The formal data contract. Defines entities, typed properties...",
    "category": "contract",
    "format": "Technical Specification",
    "date": "2026-02",
    "status": "v0.1 complete",
    "order": 1,
    "tags": ["schema", "v0.1", "entities", "containment", "visibility"],
    "details": [
      "Entity-property-behaviour model with hidden/revealed state",
      "Containment as the universal spatial primitive",
      "Condition expressions and effect grammar",
      "Rule system: trigger → condition → select → effect"
    ],
    "wordCount": 8500,
    "readingTime": "35 min",
    "excerpt": "First 300 characters of body content...",
    "url": "/documents/schema-spec-v01",
    "color": "#dab860"
  }
]
```

The `color` field is derived from `category` using the taxonomy mapping:
```
research     → #cc8888
contract     → #dab860
authoring    → #6a9acc
architecture → #cc7a3a
runtime      → #b090dd
validation   → #70b080
```

Computed fields (`wordCount`, `readingTime`, `excerpt`, `url`, `color`) are generated at build time. Authors never set these manually.

### The Documents

Ten design documents exist as .docx files. They will be converted to markdown and placed in `/docs` in a follow-up task:

| Filename | Title | Category | Format |
|----------|-------|----------|--------|
| `world-framework-research.md` | Landscape Analysis & Gap Assessment | research | Research Report |
| `pain-points-report.md` | Developer Pain Points Report | research | Forum-Sourced Research |
| `schema-spec-v01.md` | Urd World Schema Specification v0.1 | contract | Technical Specification |
| `nested-dialogue.md` | Nested Dialogue Architecture | contract | Design Document |
| `schema-markdown-v01.md` | Schema Markdown Syntax Specification | authoring | Syntax Specification |
| `architecture.md` | Architecture & Technical Design | architecture | System Blueprint |
| `system-architecture.md` | System Architecture Diagram | architecture | Interactive Visualisation |
| `wyrd-runtime.md` | Wyrd Reference Runtime | runtime | Runtime Specification |
| `test-case-strategy.md` | Test Case Strategy | validation | Validation Plan |
| `product-vision.md` | Urd + Wyrd Product Vision v2.0 | strategy | Product Strategy |

---

## Pages & Routes (Initial Scope)

This brief covers **scaffolding only**. No site components, pages, or interactive features are built yet. Those will be designed and implemented incrementally in follow-up briefs.

### `/` — Coming Soon Page

A single static page that:

- Uses the Gloaming theme (dark background, warm text, correct fonts)
- Shows the `Urd + Wyrd` brand wordmark (Georgia, uppercase, gradient text fills)
- Displays the tagline: "Define in Urd. Test in Wyrd. Ship Anywhere."
- Brief one-line description of the project
- "Coming soon" indication
- Link to the GitHub repo
- Rune canvas background (see below)
- Fully responsive

This page establishes that the theme, fonts, colour tokens, and build pipeline all work end-to-end. It is the foundation that all future components build on.

### Rune Canvas

Include a minimal rune canvas as the page background — a canvas element rendering slowly drifting Elder Futhark runes in gold at very low opacity. Implement as vanilla JS (not Svelte — it doesn't need reactivity):

- Fixed position, full viewport, z-index 0 (behind everything)
- ~25 rune characters, drifting upward at varying speeds
- Maximum opacity: 0.05 (per the design brief — this is a hard limit)
- Opacity oscillates with a sine wave for a breathing effect
- Respects `prefers-reduced-motion`: disable animation, show static runes
- The rune set: `ᚠᚢᚦᚨᚱᚲᚷᚹᚺᚾᛁᛃᛇᛈᛉᛊᛏᛒᛖᛗᛚᛜᛝᛟᛞ`

---

## Content Pipeline (Deferred)

The content collection configuration and manifest generation will be wired up in a follow-up brief once the markdown documents have been placed in `/docs`. For now, the Astro project should build and run without any content dependencies.

---

## Deployment

- Target: **Cloudflare Pages**
- Build command: `pnpm --filter urd.dev build`
- Build output directory: `sites/urd.dev/dist`
- No environment variables or secrets needed for the initial build

---

## What NOT To Build Yet

- No site components beyond the coming soon page (document index, artifact cards, pipeline visualisation, etc. — all future briefs)
- No blog/journal infrastructure
- No urd.world site
- No compiler, runtime, or any Urd tooling code
- No authentication, CMS, or admin interface
- No analytics
- No dark/light theme toggle — Gloaming only
- No Parchment theme implementation (the design files are placed for future use)
- No Svelte islands yet (nothing interactive beyond the rune canvas)

---

## Technical Constraints

- pnpm only, not npm or yarn
- Node.js 20+
- All TypeScript, strict mode
- Pure static site generation — no SSR
- Minimise JavaScript shipped to the client. For this initial scaffold, the only JS should be the rune canvas. Individual pages should be zero-JS where possible
- **British English spelling throughout** (behaviour, colour, visualisation) — per design brief §8
- Accessibility: semantic HTML, proper heading hierarchy, keyboard navigation for filter/search, sufficient colour contrast (WCAG AAA for primary text, AA for dim text — per design brief §12)
- **No icon libraries** (Lucide, Heroicons, etc.) — per design brief §9. Use text characters only: → for arrows, ◆ for list markers, ▸ for expand, ↗ for external links
- **Commit frequently** — after each logical unit of work (workspace setup, Astro scaffold, theme CSS, coming soon page, rune canvas). Each commit should be self-contained and the project should build cleanly at every commit point

---

## Success Criteria

When this task is complete:

1. `pnpm install` at the repo root succeeds
2. `pnpm dev` launches the Astro dev server
3. The coming soon page (`/`) renders with the Gloaming theme — correct colours, fonts, and rune canvas
4. Typography is correctly set up: Georgia for wordmark, Outfit for headings, Source Serif 4 for body, JetBrains Mono for code
5. All CSS custom properties from the design brief are defined and working
6. The rune canvas renders correctly at ≤ 0.05 opacity
7. `pnpm build` produces a static `dist/` folder ready for Cloudflare Pages
8. Empty directory structure exists for `/design`, `/docs`, `/packages`, `/briefs`
9. All filenames in the repo are lowercase