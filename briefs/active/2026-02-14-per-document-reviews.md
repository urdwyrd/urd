# Brief: Per-Document AI Reviews

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** —
**Status:** Backlog

### What was done

_(To be filled on completion.)_

### What changed from the brief

_(To be filled on completion.)_

---

**Created:** 2026-02-14

## Objective

Add per-document AI reviews to the bottom of each document page on urd.dev. Reviews are authored as markdown files grouped by document slug, loaded via a new Astro content collection, served through per-slug JSON endpoints, and rendered by a shared Svelte 5 island. Documents with no reviews show nothing — no empty state, no placeholder.

This extends the homepage `PeerReview` pattern to individual documents, giving each design document its own set of AI peer evaluations.

## Content Architecture

### Directory structure

Reviews live in subdirectories under `content/document-reviews/`, keyed by document slug:

```
content/document-reviews/
  architecture/
    deepseek-r1.md
    gemini3.md
  schema-spec/
    gpt-52.md
  urd-formal-grammar-brief/
    deepseek-r1.md
```

- Subdirectory name **must** match the document's `slug` frontmatter field exactly.
- Review filenames should match existing icon filenames (e.g. `deepseek-r1.md` reuses `public/images/reviews/deepseek-r1.png`).
- Same model can review multiple documents — each lives in its own slug subdirectory, no conflict.

### Review file format

Each review is a single markdown file. The body is the review text. Frontmatter reuses the same schema as homepage reviews:

```yaml
---
model: "DeepSeek-V3.2 (Thinking)"
company: "DeepSeek"
date: "2026-02"
rating: 5
initial: "D"
colour: "#70b080"
---
Review body text here.
```

**Frontmatter fields:**

| Field   | Type   | Description                                 |
|---------|--------|---------------------------------------------|
| model   | string | Display name of the AI model                |
| company | string | Company / lab name                          |
| date    | string | Review date (YYYY-MM or YYYY-MM-DD)         |
| rating  | number | 1–5 star rating                             |
| initial | string | Single character for the avatar circle       |
| colour  | string | Hex colour for the avatar circle background  |

### Content collection registration

Add a `documentReviews` collection to `content.config.ts` with the same Zod schema as the existing `reviews` collection:

```typescript
const documentReviews = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../content/document-reviews' }),
  schema: z.object({
    model: z.string(),
    company: z.string(),
    date: z.string(),
    rating: z.number().min(1).max(5),
    initial: z.string().length(1),
    colour: z.string(),
  }),
});

export const collections = { designDocs, reviews, updates, timeline, documentReviews };
```

Entry IDs will be like `architecture/deepseek-r1` — the document slug is derived from `entry.id.split('/')[0]`.

### JSON endpoint

Create `src/pages/document-reviews/[slug].json.ts` — a parameterised static endpoint:

- Uses `getStaticPaths` iterating the `designDocs` collection to generate one JSON file per document slug.
- Filters `documentReviews` entries where `entry.id.split('/')[0] === params.slug`.
- Returns `[]` for documents with no reviews (avoids 404s).
- Reuses existing review icon path pattern (`/images/reviews/{filename}.png`).

## Component Design

### `DocumentReviews.svelte` — shared island

- Receives `slug` prop from the Astro layout.
- Fetches `/document-reviews/{slug}.json` on mount.
- Renders nothing if no reviews (`{#if loaded && reviews.length > 0}`).
- Single-column card grid (document content column is narrower than homepage).
- Review card styling matches `PeerReview.svelte`: stars, italic quote, avatar + attribution, left-border accent.
- Section header: "Peer Review" label, contextual heading, subtitle.

### Integration into document layout

In `sites/urd.dev/src/layouts/document.astro`:

- Import `DocumentReviews` component.
- Place `<DocumentReviews slug={slug} client:visible />` between the prose slot and the footer.
- `client:visible` defers hydration until scrolled to (reviews sit at the bottom of long documents).

## Files to create / modify

| File | Action |
|------|--------|
| `content/document-reviews/{slug}/*.md` | **Create** — seed at least one review file |
| `sites/urd.dev/src/content.config.ts` | **Modify** — add `documentReviews` collection |
| `sites/urd.dev/src/pages/document-reviews/[slug].json.ts` | **Create** — per-slug JSON endpoint |
| `sites/urd.dev/src/components/DocumentReviews.svelte` | **Create** — shared island component |
| `sites/urd.dev/src/layouts/document.astro` | **Modify** — integrate the island |

## What does NOT change

- `[slug].astro` — already passes `slug` to the layout
- `PeerReview.svelte` — stays as-is for homepage general reviews
- `content/reviews/` — independent collection for homepage reviews

## Edge cases

| Case | Handling |
|------|----------|
| Document with no reviews | Endpoint returns `[]`, component renders nothing |
| Fetch failure | `reviews` stays `[]`, component renders nothing |
| Review icons | Reuse existing PNGs by matching filenames to `content/reviews/` names |
| Same model reviews multiple documents | Each lives in its own slug subdirectory, no conflict |

## Out of scope

- Filtering or pagination (small dataset per document)
- Admin interface (content managed via markdown files)
- Modifying the homepage `PeerReview` component
- Creating review content for all documents (seed one or two for verification)

## Verification

1. Seed at least one review file in a document subdirectory
2. `pnpm build` — confirm per-slug JSON endpoints are generated (e.g. `dist/document-reviews/architecture.json`)
3. Document page with reviews shows the review section below prose
4. Document page without reviews shows no extra section
5. Review icons load correctly (reusing existing PNGs)

## Execution record

| Step | Status | Notes |
|------|--------|-------|
| Create `content/document-reviews/` with seed review files | — | |
| Register `documentReviews` collection in content.config.ts | — | |
| Create `document-reviews/[slug].json.ts` endpoint | — | |
| Build `DocumentReviews.svelte` island | — | |
| Integrate into `document.astro` layout | — | |
| Verify build passes | — | |
