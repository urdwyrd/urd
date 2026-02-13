# Brief: ProjectLog Island Component

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-13
**Status:** Complete

### What was done

- Created `content/updates/` directory with 3 initial update markdown files covering existing project milestones (specs locked, homepage live, Cloudflare deployment).
- Registered `updates` collection in `content.config.ts` alongside `designDocs` and `reviews`, with Zod schema for title and date.
- Created `updates.json.ts` endpoint sorted by date descending.
- Built `ProjectLog.svelte` island: single-column timeline with mono date column (80px, right-aligned), display-font titles, body-font descriptions. Responsive stacking at 640px.
- Added section to homepage below DocumentExplorer with matching wrapper constraints.
- Updated `CLAUDE.md` with `updates/` in repo layout and ProjectLog in island list.

### What changed from the brief

- Tag functionality (tag field, colour mapping, tag pills) was removed at user request — the component is simpler than originally specified.
- Launched with 3 entries instead of 5 (user trimmed the initial set).

---

**Created:** 2026-02-13

## Objective

Build a `ProjectLog` Svelte 5 island that displays a reverse-chronological timeline of project updates. Content is authored as markdown files with frontmatter, loaded via Astro content collections and served through a JSON endpoint — the same pattern as PeerReview.

## Content Architecture

### Content directory

Add an `updates` subdirectory under the existing `content/` root:

```
content/
  reviews/          ← AI peer reviews (existing)
  updates/          ← Project log entries (this brief)
```

### Update file format

Each update is a single markdown file. The body is the description text. Frontmatter carries structured metadata:

```yaml
---
title: Three new documents added
date: "2026-02-12"
tag: specification
---

Writer Reference Card, Engine Developer Reference Card, and Future Proposals published. Total document count now 13. Content pipeline validated with all new frontmatter fields.
```

**Frontmatter fields:**

| Field | Type   | Description                                    |
|-------|--------|------------------------------------------------|
| title | string | Short headline for the update                  |
| date  | string | Entry date (YYYY-MM-DD)                        |
| tag   | string | Category tag (specification, tooling, infrastructure, security) |

### Tag colour mapping

Define a static map inside the component (not a shared module — keep it self-contained until a second consumer appears):

| Tag              | Colour variable |
|------------------|-----------------|
| specification    | `--gold`        |
| tooling          | `--green`       |
| infrastructure   | `--blue`        |
| security         | `--amber`       |

Tags not in the map fall back to `--faint`.

### Content collection registration

Add an `updates` collection to `content.config.ts` alongside the existing collections:

```typescript
const updates = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../content/updates' }),
  schema: z.object({
    title: z.string(),
    date: z.string(),
    tag: z.string(),
  }),
});

export const collections = { designDocs, reviews, updates };
```

### JSON endpoint

Create `src/pages/updates.json.ts` that reads the `updates` collection and returns an array sorted by date (newest first), with the rendered markdown body as `description`.

## Component Design

### Visual spec (from screenshot)

- **Section header:** label ("PROJECT LOG"), title ("Updates"), subtitle ("What's happened, as it happens.")
- **Layout:** single-column list (not a grid) — each entry is a full-width row
- **Each entry:**
  - Separated by a `1px solid var(--border)` top border
  - Date on the left in mono font (`--faint`), formatted as "DD Mon" (e.g. "12 Feb")
  - Bold title in display font
  - Description text in body font, `--dim` colour
  - Tag badge: small pill with coloured text matching the tag map, background at 10% opacity of the tag colour, mono font, lowercase
- **Padding:** 20px vertical per entry
- **Date column:** ~80px fixed width, right-aligned

### Props

```typescript
interface Props {
  label?: string;   // default: "Project Log"
  title?: string;   // default: "Updates"
  subtitle?: string; // default: "What's happened, as it happens."
}
```

### Data flow

Same pattern as PeerReview:
1. Component mounts, fetches `/updates.json`
2. Renders entries from the returned array (already sorted by date desc)
3. No client-side filtering needed

### Date formatting

Parse the `YYYY-MM-DD` date string and format to "DD Mon" (e.g. "12 Feb"). Use a simple lookup array for month abbreviations — no date library needed.

### Responsive behaviour

- ≤ 640px: date moves above the title instead of beside it (stack vertically)
- Tag badge: inline after description text on mobile

## Files to create / modify

| File | Action |
|------|--------|
| `content/updates/*.md` | **Create** — initial update entries |
| `sites/urd.dev/src/content.config.ts` | **Modify** — add `updates` collection |
| `sites/urd.dev/src/pages/updates.json.ts` | **Create** — JSON endpoint |
| `sites/urd.dev/src/components/ProjectLog.svelte` | **Create** — island component |

## Out of scope

- Pagination (not needed for < 20 entries)
- Filtering by tag
- Composing into site layout (will be done separately)

## Execution record

| Step | Status | Notes |
|------|--------|-------|
| Create `content/updates/` with initial entries | Done | 5 entries covering existing milestones |
| Register `updates` collection in content.config.ts | Done | Added alongside designDocs and reviews |
| Create `updates.json.ts` endpoint | Done | Sorted by date descending |
| Build `ProjectLog.svelte` island | Done | Timeline layout, tag colour pills, responsive stacking |
| Verify build passes | Done | 13 pages + 3 JSON endpoints built cleanly |
