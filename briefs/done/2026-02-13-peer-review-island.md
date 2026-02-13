# Brief: PeerReview Island Component

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-13
**Status:** Complete

### What was done

- Created `content/reviews/` directory with 4 review markdown files (claude-opus, gemini3, gpt-52, deepseek-r1, grok) — each with frontmatter for model, company, date, rating, initial, and colour.
- Registered `reviews` collection in `content.config.ts` alongside `designDocs`, with Zod schema validation.
- Created `reviews.json.ts` endpoint following the same pattern as `documents.json.ts`.
- Built `PeerReview.svelte` island: 2-column card grid with star ratings (gold filled / dim empty), coloured avatar circles, left-border accent per review colour, italic quotes in Source Serif 4.
- Added CTA line below cards: "Don't take their word for it — feed the specs to your favourite model and see what it thinks."
- Unified section width constraints in `index.astro` — both PeerReview and DocumentExplorer share `max-width: 980px` wrapper.
- Aligned heading styles (label, title, subtitle) to match DocumentExplorer.
- Updated `CLAUDE.md` with `content/` directory, content architecture convention, and Svelte 5 island list.

### What changed from the brief

- Added a `grok.md` review (5th review, not in original 4).
- Review content was updated by the user to reflect newer model versions (GPT-5.2, Gemini 3, DeepSeek-V3.2).
- Width constraint moved from component internals to section wrappers in `index.astro`.

---

**Created:** 2026-02-13

## Objective

Build a `PeerReview` Svelte 5 island that displays AI model reviews of the Urd specification in a 2-column card grid. Content is authored as markdown files with frontmatter, loaded via Astro content collections and served through a JSON endpoint — the same pattern as DocumentExplorer.

## Content Architecture

### New top-level `content/` directory

Introduce `content/` at the repository root as a scalable home for all structured content that isn't design documentation. Subdirectories map to content types:

```
content/
  reviews/          ← AI peer reviews (this brief)
  (future types)    ← e.g. changelog/, quotes/, features/
```

This keeps `docs/` for design documents and `content/` for site content, with each subdirectory becoming its own Astro content collection.

### Review file format

Each review is a single markdown file. The body is the quote text. Frontmatter carries structured metadata:

```yaml
---
model: Claude Opus
company: Anthropic
date: "2026-02"
rating: 5
initial: C
colour: "#dab860"
---

The separation of schema contract from runtime is the key insight. Everything else follows from that. The seven-symbol vocabulary for Schema Markdown strikes a rare balance between expressiveness and learnability.
```

**Frontmatter fields:**

| Field     | Type   | Description                                    |
|-----------|--------|------------------------------------------------|
| model     | string | Display name of the AI model                   |
| company   | string | Company / lab name                             |
| date      | string | Review date (YYYY-MM or YYYY-MM-DD)            |
| rating    | number | 1–5 star rating                                |
| initial   | string | Single character for the avatar circle          |
| colour    | string | Hex colour for the avatar circle background     |

### Content collection registration

Add a `reviews` collection to `content.config.ts` alongside the existing `designDocs`:

```typescript
const reviews = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../content/reviews' }),
  schema: z.object({
    model: z.string(),
    company: z.string(),
    date: z.string(),
    rating: z.number().min(1).max(5),
    initial: z.string().length(1),
    colour: z.string(),
  }),
});

export const collections = { designDocs, reviews };
```

### JSON endpoint

Create `src/pages/reviews.json.ts` that reads the `reviews` collection and returns an array of review objects with the rendered markdown body as `quote`.

## Component Design

### Visual spec (from screenshot)

- **Section header:** label ("PEER REVIEW"), title ("What the Machines Think"), subtitle
- **Card grid:** 2 columns on desktop, 1 column on mobile (breakpoint: 640px)
- **Each card:**
  - Left border accent using the review's `colour`
  - Italic quote text (Source Serif 4, body font)
  - Star rating row: gold filled stars, dimmed empty stars (text characters only — no icon library)
  - Avatar row: coloured circle with `initial`, bold model name, company + date in mono
- **Card background:** `--raise`, hover to `--surface`
- **Gap:** 10px (matching DocumentExplorer card list)

### Props

```typescript
interface Props {
  label?: string;   // default: "Peer Review"
  title?: string;   // default: "What the Machines Think"
  subtitle?: string; // default: "We asked leading AI systems to review the specification. They had opinions."
}
```

### Data flow

Same pattern as DocumentExplorer:
1. Component mounts, fetches `/reviews.json`
2. Renders cards from the returned array
3. No client-side filtering needed (small dataset)

### Stars

Use the star character `★` for filled and `★` with dim colour for empty. Rating is 1–5. Position stars at the top-right of each card. Stars use `--gold` for filled, `--border-light` for empty.

### Responsive behaviour

- ≤640px: single column
- Avatar circle: 32px diameter, `initial` centred inside, background is review `colour`

## Files to create / modify

| File | Action |
|------|--------|
| `content/reviews/*.md` | **Create** — 4 initial review files |
| `sites/urd.dev/src/content.config.ts` | **Modify** — add `reviews` collection |
| `sites/urd.dev/src/pages/reviews.json.ts` | **Create** — JSON endpoint |
| `sites/urd.dev/src/components/PeerReview.svelte` | **Create** — island component |

## Out of scope

- Filtering or pagination (not needed for ~4–8 reviews)
- Admin interface (content is managed via markdown files in the repo)
- Composing into site layout (will be done separately)

## Execution record

| Step | Status | Notes |
|------|--------|-------|
| Create `content/reviews/` with 4 review files | Done | claude-opus, gemini-2-pro, gpt-4o, deepseek-r1 |
| Register `reviews` collection in content.config.ts | Done | Added alongside designDocs |
| Create `reviews.json.ts` endpoint | Done | Same pattern as documents.json.ts |
| Build `PeerReview.svelte` island | Done | 2-col grid, stars, avatars, left-border accent |
| Verify build passes | Done | 13 pages + 2 JSON endpoints built cleanly |
