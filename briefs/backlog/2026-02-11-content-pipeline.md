# Brief: Content Pipeline — Frontmatter, Collections, and documents.json

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:**
**Status:** Not started

### What was done
<!-- List every concrete action taken, including files created/modified -->

### Deviations from brief
<!-- Anything that differed from the plan and why -->

### Issues encountered
<!-- Problems hit during execution and how they were resolved -->

### Notes for next brief
<!-- Context, warnings, or recommendations for future work -->

---

## Context

The scaffold brief defined a document data model: each markdown file in `/docs` gets YAML frontmatter, and at build time a `documents.json` manifest is generated with computed fields (`wordCount`, `readingTime`, `excerpt`, `color`). The interactive document index on the site loads this JSON client-side.

The documents are being converted from .docx to markdown and placed in `/docs`. This brief should be executed **after all documents have been placed** — it wires up the pipeline that reads them.

### Prerequisites

Before executing this brief, ensure all markdown documents are in `/docs`:

| Filename | Title | Category |
|----------|-------|----------|
| `product-vision.md` | Urd + Wyrd Product Vision v2.0 | strategy |
| `world-framework-research.md` | Landscape Analysis & Gap Assessment | research |
| `pain-points-report.md` | Developer Pain Points Report | research |
| `schema-spec-v01.md` | Urd World Schema Specification v0.1 | contract |
| `nested-dialogue.md` | Nested Dialogue Architecture | contract |
| `schema-markdown-v01.md` | Schema Markdown Syntax Specification | authoring |
| `architecture.md` | Architecture & Technical Design | architecture |
| `system-architecture.md` | System Architecture Diagram | architecture |
| `wyrd-runtime.md` | Wyrd Reference Runtime | runtime |
| `test-case-strategy.md` | Test Case Strategy | validation |

If not all documents are ready, the pipeline can still be built — it will generate the manifest for whatever documents are present.

---

## Implementation Steps

### Step 1: Rename `product-vision-master.md`

Rename `docs/product-vision-master.md` to `docs/product-vision.md` to match the spec from the scaffold brief.

### Step 2: Add YAML frontmatter to each document

Each markdown file in `/docs` needs frontmatter matching this schema:

```yaml
---
title: "Urd + Wyrd Product Vision v2.0"
slug: "product-vision"
description: "The strategic vision for Urd and Wyrd — market analysis, product architecture, revenue model, and development roadmap."
category: "strategy"
format: "Product Strategy"
date: "2026-02"
status: "v2.0 complete"
order: 1
tags:
  - vision
  - strategy
  - product
  - roadmap
details:
  - "Seven pain points from developer forums and postmortems"
  - "Four-component pipeline: compiler, runtime, testing, tooling"
  - "Revenue model: open core with hosted services"
  - "AI-native design philosophy"
---
```

**Field reference:**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `title` | string | yes | Full document title |
| `slug` | string | yes | URL-safe identifier, used for `/documents/[slug]` |
| `description` | string | yes | 1–2 sentence summary for cards and OG tags |
| `category` | enum | yes | `research`, `contract`, `authoring`, `architecture`, `runtime`, `validation`, or `strategy` |
| `format` | string | yes | Freeform label: "Research Report", "Technical Specification", etc. |
| `date` | string | yes | Year-month (`2026-02`) |
| `status` | string | yes | Freeform: "v0.1 complete", "in progress", "draft" |
| `order` | number | yes | Display order within category |
| `tags` | string[] | yes | Keyword tags for filtering |
| `details` | string[] | yes | Key content bullets shown in expanded card view |

### Step 3: Set up Astro content collections

Create `sites/urd.dev/src/content.config.ts` defining a `docs` collection.

Astro 5 supports content collections that read from directories outside `src/content/` using a glob loader. Point it at the repo-root `docs/` directory so the markdown lives at the monorepo level (not duplicated into the site).

The collection schema uses Zod (via Astro's `defineCollection`) to validate frontmatter at build time. A document with a typo in `category` or a missing required field will fail the build — not silently render wrong.

```typescript
import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const docs = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../docs' }),
  schema: z.object({
    title: z.string(),
    slug: z.string(),
    description: z.string(),
    category: z.enum([
      'research', 'contract', 'authoring',
      'architecture', 'runtime', 'validation', 'strategy'
    ]),
    format: z.string(),
    date: z.string(),
    status: z.string(),
    order: z.number(),
    tags: z.array(z.string()),
    details: z.array(z.string()),
  }),
});

export const collections = { docs };
```

### Step 4: Generate `documents.json` as a static endpoint

Create `sites/urd.dev/src/pages/documents.json.ts` — an Astro static endpoint that at build time:

1. Reads all entries from the `docs` content collection
2. Computes derived fields from each document's body and frontmatter:
   - `wordCount` — count words in the markdown body
   - `readingTime` — `Math.ceil(wordCount / 250)` minutes
   - `excerpt` — first 300 characters of body content (stripped of markdown syntax)
   - `color` — mapped from `category` using the taxonomy:
     ```
     research     → #cc8888
     contract     → #dab860
     authoring    → #6a9acc
     architecture → #cc7a3a
     runtime      → #b090dd
     validation   → #70b080
     strategy     → #dab860
     ```
   - `url` — `/documents/${slug}`
3. Outputs the JSON array to `/documents.json` in `dist/`

**Why a static endpoint:** It's just another static route — cached and served from Cloudflare Pages at `https://urd.dev/documents.json`. No SSR, no separate build step. The interactive component fetches it on page load.

---

## Files to Create/Modify

| File | Action |
|------|--------|
| `docs/product-vision-master.md` | Rename to `docs/product-vision.md` |
| `docs/*.md` (all documents) | Add YAML frontmatter |
| `sites/urd.dev/src/content.config.ts` | Create — content collection definition with Zod schema |
| `sites/urd.dev/src/pages/documents.json.ts` | Create — static endpoint generating the manifest |
| `sites/urd.dev/astro.config.mjs` | May need adjustment for external content directory |

---

## Verification

1. **`pnpm build` succeeds** — content collection validates all frontmatter
2. **`dist/documents.json` exists** — contains the full manifest with computed fields (`wordCount`, `readingTime`, `excerpt`, `url`, `color`)
3. **Schema validation works** — temporarily remove a required field from one document's frontmatter; the build should fail with a clear error
4. **JSON is well-formed** — `cat dist/documents.json | python -m json.tool` or similar

---

## What NOT To Do

- Do not build individual document pages (`/documents/[slug]`) in this brief — that's a separate brief
- Do not build the interactive document index UI — that depends on this pipeline being in place first
- Do not duplicate markdown files into `sites/urd.dev/src/content/` — use the glob loader to read from the repo-root `docs/` directory
- Do not hardcode document metadata — everything comes from frontmatter + computed fields

---

## Future Considerations

- **Individual document pages:** Once the pipeline is in place, add `/documents/[slug]` routes using Astro's `getStaticPaths()` with the content collection
- **Interactive document index:** A Svelte island that fetches `/documents.json` and provides category filtering, search, and expandable cards — this is the next visual brief after the pipeline
- **Incremental adoption:** The pipeline works with any number of documents. Start with one, add more as they're converted
