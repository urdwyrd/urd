# Brief: Document Explorer — Svelte Island Component

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-11
**Status:** Complete

### What was done

- Created `sites/urd.dev/src/components/DocumentExplorer.svelte` — Svelte 5 island component using `$state`, `$derived`, `$props` runes. Fetches `/documents.json` on mount, renders category filter bar and expandable document cards with category-coloured accents.
- Modified `sites/urd.dev/src/pages/index.astro` — imported and rendered DocumentExplorer with `client:visible` directive below the hero section, added a `.documents-section` wrapper with border-top separator.
- Added `--doc-strategy` and `--doc-strategy-dim` CSS variables to `sites/urd.dev/src/styles/global.css` — the strategy category was missing from the taxonomy variables.
- Updated `design/urd-document-taxonomy.md` — expanded from six to seven categories, added Strategy section, updated flowchart and visual reference.
- Updated `design/themes/gloaming/design-brief.md` §7 — added strategy row to the category table.

### Deviations from brief

- Tags in the expanded card section use `color: var(--card-colour)` with `opacity: 0.6` instead of `var(--faint)` — gives them a subtle category tint rather than flat grey.
- The strategy category required adding CSS variables and updating reference docs — not anticipated in the original brief since the taxonomy was documented as six categories.

### Issues encountered

- The `strategy` category had no `--doc-strategy` CSS variable, causing the pill and accent to render with no colour. Fixed by adding the variables to `global.css`.
- `git mv` failed for an untracked file (the brief itself) — used plain file move instead.
- Astro TypeScript types went stale after renaming the content collection from `docs` to `designDocs` — resolved by running `astro sync`.

### Notes for next brief

- The "Read full document →" links point to `/documents/{slug}` but those routes don't exist yet — the homepage and document pages brief will create them.
- The component is 12.5kB / 5.6kB gzipped — acceptable for a Svelte island.
- The DocumentExplorer cards already have the card link wired to `doc.url` — once slug pages exist, navigation will work automatically.

---

## Context

The content pipeline is complete — all 9 design documents have YAML frontmatter, Astro content collections validate the schema at build time, and a static `/documents.json` endpoint serves the manifest with computed fields (wordCount, readingTime, excerpt, colour, url).

This brief builds the interactive component that consumes that JSON and presents an explorable document index. It is the first Svelte island in the project.

### Design References

Before implementing, read:

1. `design/themes/gloaming/design-brief.md` — canonical visual spec (colours, typography, spacing, card anatomy, animation, responsive breakpoints)
2. `design/urd-document-taxonomy.md` — six-category colour system, CSS variable naming (`--doc-{category}`, `--doc-{category}-dim`)

### Prerequisites

- Content pipeline complete: `/documents.json` serves all 9 documents with computed fields
- Svelte 5 integration installed in Astro (`@astrojs/svelte`)
- Global CSS variables defined in `global.css` (surface, text, brand, taxonomy tokens)

---

## Component Design

### Overview

A self-contained Svelte 5 island component (`DocumentExplorer.svelte`) that:

1. Fetches `/documents.json` on mount
2. Renders an explorable list of document cards with category-coloured accents
3. Supports filtering by category
4. Cards expand to reveal key content details
5. Is responsive: works at narrow widths (single column) and wider layouts
6. Uses only global CSS custom properties — no hardcoded colours or fonts
7. Is reusable: can be dropped into any page/layout with configurable header props

### Props

```typescript
interface DocumentExplorerProps {
  label?: string;      // Small category label above heading (e.g. "ARTIFACTS")
  title?: string;      // Main heading (e.g. "The work so far")
  subtitle?: string;   // Description below heading (e.g. "Every document produced. Click to expand.")
}
```

All props are optional with sensible defaults.

### Data Shape

The component fetches `/documents.json` and expects this shape per entry:

```typescript
interface Document {
  title: string;
  slug: string;
  description: string;
  category: string;
  format: string;
  date: string;
  status: string;
  order: number;
  tags: string[];
  details: string[];
  wordCount: number;
  readingTime: number;
  excerpt: string;
  colour: string;
  url: string;
}
```

---

## Visual Specification

### Header Section

- **Label**: Outfit 12–13px, 600 weight, uppercase, 1.5px letter-spacing, `var(--gold-dim)` colour
- **Title**: Outfit 26–28px, 600 weight, `var(--text)` colour, -0.01em tracking
- **Subtitle**: Source Serif 4, 17–18px, `var(--faint)` colour
- **Filter buttons**: Aligned right of the subtitle on desktop, below it on mobile

### Filter Bar

- "All" button plus one button per category present in the fetched data
- Button text: the category name in title case (Research, Contract, Authoring, Architecture, Runtime, Validation)
- Active filter: `var(--doc-{category})` text colour with a subtle bottom border or background tint
- "All" active state: `var(--gold)` text
- Inactive: `var(--faint)` text, `var(--border)` border
- Transition: 0.15s ease on colour and border
- On narrow viewports: filter buttons wrap naturally, horizontally scrollable if needed

### Document Cards

Follow the card anatomy from the design brief (§5):

```
┌─ border: 1px solid var(--border) ──────────────────────────┐
│  border-left: 2px solid var(--doc-{category})              │
│  border-radius: 8px                                        │
│  background: var(--raise)                                  │
│  padding: 18–24px                                          │
│                                                            │
│  [FORMAT BADGE]  date  wordCount     [expand indicator ▸]  │
│  TITLE                                                     │
│  Description text (from frontmatter description field)     │
│                                                            │
│  ── expanded content (hidden by default) ──────────────    │
│  KEY CONTENT                                               │
│  ◆ Detail bullet 1                                         │
│  ◆ Detail bullet 2                                         │
│  ◆ Detail bullet 3                                         │
│                                                            │
│  Tags: tag1 · tag2 · tag3                                  │
│                                                            │
│  Read full document →                                      │
└────────────────────────────────────────────────────────────┘
```

#### Card Elements

- **Format badge (pill)**: Outfit 10–11px, 500 weight, uppercase, 1px letter-spacing. Background: `var(--doc-{category}-dim)`. Text: `var(--doc-{category})`. Border: 1px solid with ~0.2 opacity of the category colour. Border-radius: 4px. Padding: 2px 8px.
- **Date**: JetBrains Mono 12–13px, `var(--faint)`, 0.06em tracking. Format: "Jan 2026", "Feb 2026" (parsed from the `date` field).
- **Word count**: JetBrains Mono 12–13px, `var(--faint)`. Display as "4,742 words" or reading time "19 min read" — use whichever feels more natural; prefer word count with comma formatting.
- **Expand indicator**: `▸` character, rotates to `▾` on expand. `var(--faint)` colour, transitions 0.15s.
- **Title**: Outfit 17–18px, 600 weight, `var(--text)`. Use `text-transform: capitalize` or render as-is from the data (titles are already properly cased).
- **Description**: Source Serif 4, 17–18px, 400 weight, `var(--dim)`, line-height 1.6. This is the frontmatter `description` field, not the computed excerpt.
- **Expanded section label** ("KEY CONTENT"): Outfit 11–12px, 600 weight, uppercase, 1.5px tracking, `var(--faint)`.
- **Detail bullets**: Source Serif 4, 16–17px, `var(--dim)`. Use `◆` in category colour as the bullet marker, 7px size.
- **Tags**: JetBrains Mono 11–12px, `var(--faint)`. Separated by ` · `. Displayed inline.
- **Document link**: Outfit 13–14px, 500 weight, `var(--doc-{category})` colour. Text: "Read full document →". Points to the `url` field (`/documents/{slug}`). These routes don't exist yet but will in a future brief — render the link anyway so it works automatically when the pages are built.

#### Card Interactions

- **Click anywhere on card** to expand/collapse (except the document link)
- **Hover**: `background: var(--surface)`, `border-color: var(--border-light)`, transition 0.15s
- **Expand animation**: fadeIn 0.25s ease-out on the expanded content section. Respect `prefers-reduced-motion`.
- **Left accent border**: 2px solid using the category colour. Use the gradient pattern from the design brief where practical (e.g. `linear-gradient(to bottom, var(--doc-{category}), var(--doc-{category}-dim))`).

#### Card Sort Order

Cards are sorted by the `order` field from the JSON (already sorted by the endpoint). Within the same order, group by category visually through the accent colours — no explicit grouping needed.

### Responsive Behaviour

- **> 980px**: Cards in a single column, max-width ~860px. Filter buttons inline with subtitle.
- **520–980px**: Same single column, filter buttons wrap below subtitle.
- **< 520px**: Full width with 18px side padding. Cards maintain all content, nothing hidden.

### Empty / Loading States

- **Loading**: Show a subtle skeleton or simply render nothing until data arrives (the JSON is tiny and served from the same origin, so loading is near-instant for a static site).
- **Empty filter**: If a category filter returns no results, show a quiet message like "No documents in this category yet." in `var(--faint)`.

---

## Implementation Steps

### Step 1: Create the Svelte component

Create `sites/urd.dev/src/components/DocumentExplorer.svelte`.

This is a Svelte 5 component using runes (`$state`, `$derived`, `$effect`). It fetches `/documents.json` on mount using `onMount` + `fetch`, stores the data in reactive state, and derives the filtered list from the active category filter.

Key implementation details:
- Use `onMount` to fetch (not top-level await) since this is a client-side island
- Category colours come from CSS variables, not from the JSON `colour` field — the component reads `--doc-{category}` from the theme. The JSON `colour` field exists for external consumers; the component should use the CSS variables for theme consistency.
- Expand/collapse state is per-card (track by slug or index)
- Format the date field (`"2026-02"` → `"Feb 2026"`) using a simple month lookup
- Format word count with locale comma separators

### Step 2: Add the component to the Coming Soon page

In `sites/urd.dev/src/pages/index.astro`, import and render the Svelte component below the existing hero section. Use Astro's `client:visible` directive so the island hydrates when scrolled into view, keeping the hero zero-JS.

Add a visual separator (a simple `<hr>` or spacing div) between the hero and the document explorer.

### Step 3: Verify

1. `pnpm build` succeeds
2. `pnpm preview` — the component renders below the hero, cards are populated from JSON
3. Category filters work — clicking a category shows only matching documents
4. Card expand/collapse works with animation
5. Responsive: test at 520px, 980px, and full width
6. Colours match the taxonomy — each card's accent and pill use the correct category colour from CSS variables
7. `prefers-reduced-motion` disables expand animation

---

## Files to Create/Modify

| File | Action |
|------|--------|
| `sites/urd.dev/src/components/DocumentExplorer.svelte` | Create — the Svelte 5 island component |
| `sites/urd.dev/src/pages/index.astro` | Modify — add the component below the hero for testing |

---

## What NOT To Do

- Do not render full document content — only metadata, description, and details from the JSON
- Do not build `/documents/[slug]` pages — that's a separate brief
- Do not hardcode document data — everything comes from `/documents.json` fetched at runtime
- Do not use any icon library — text characters only (▸, ▾, ◆, →, ·)
- Do not add a search input — category filtering is sufficient for 9 documents; search can come later
- Do not use the JSON `colour` field for styling — use CSS variables (`--doc-{category}`) for theme consistency
- Do not import framework CSS — the component uses only global CSS custom properties
- Do not add gradient text fills — that's reserved for the brand wordmark only

---

## Future Considerations

- **Search**: Once the document count grows, add a text search input that filters on title, description, and tags
- **Tag filtering**: Click a tag to filter by it (in addition to category)
- **Document pages**: Once `/documents/[slug]` routes exist, the "Read full document →" links will work automatically
- **Multiple instances**: The component is designed to be reusable — different pages can mount it with different header props
- **Strategy category**: The `strategy` category (product vision) doesn't have a dedicated colour in the taxonomy — it shares gold with `contract`. If strategy documents grow, consider adding a distinct colour token
