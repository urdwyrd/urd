# Brief: Document Explorer v2 — Enhanced Filtering, Metadata, and Links

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-12
**Status:** Complete

### What was done

- Updated all 12 doc frontmatter dates from `YYYY-MM` to `YYYY-MM-DD`. Two January research docs set to `2026-01-28`; all February docs set to `2026-02-12` (first git commit date).
- Added `githubUrl` and `downloadUrl` fields to `documents.json.ts` endpoint, computed from `entry.id`.
- Rewrote `DocumentExplorer.svelte` with four enhancements:
  1. **Format filters** — secondary row of smaller, neutral pills below category filters. Only shown when the current category has more than one format. Resets when category changes.
  2. **Full dates** — `formatDate()` updated to parse `YYYY-MM-DD` and render as "12 Feb 2026" with fallback for `YYYY-MM`.
  3. **Reading time** — meta row now shows "5 min read (1,234 words)" via new `formatReadingTime()`. Handles < 1 min edge case.
  4. **Action links** — expanded section now has three links in a flex row: "Read full document →" (primary), "Download .md" (secondary, with `download` attribute), "View on GitHub ↗" (secondary, opens new tab).
- Cleared stale `.astro/` cache that was causing duplicate ID warnings from a previous `docs` collection definition.

### Deviations from brief

- No deviations. All four features implemented as specified.

### Follow-up tasks

- The `rehype-autolink-headings` package is still installed but unused (removed from config in a previous brief). Can be uninstalled in a future cleanup pass.

---

## Context

The DocumentExplorer Svelte island is the centrepiece of the homepage. It currently shows category filter buttons, expandable cards with title/description/details/tags, word count, and a link to the rendered document page. This brief adds four improvements: secondary format filters, precise dates, reading time display, and external links (download + GitHub view).

### Current state

- **Component:** `sites/urd.dev/src/components/DocumentExplorer.svelte` — Svelte 5 runes, fetches `/documents.json`
- **Data endpoint:** `sites/urd.dev/src/pages/documents.json.ts` — Astro API route, already computes `wordCount` and `readingTime`
- **Schema:** `sites/urd.dev/src/content.config.ts` — `date` field is `z.string()`, currently stores `"2026-02"` (no day)
- **Docs:** 12 markdown files in `docs/`, all with valid frontmatter
- **Categories (7):** research, contract, authoring, architecture, runtime, validation, strategy
- **Formats (10 unique):** System Blueprint, Proposal Collection, Design Document, Forum-Sourced Research, Product Strategy, Reference Card (x2), Syntax Specification, Technical Specification, Validation Plan, Research Report, Runtime Specification
- **GitHub repo:** `urdwyrd/urd`, docs at `docs/` on `main` branch

---

## 1. Secondary format filters

### Goal

Add a second row of smaller filter chips below the primary category filters, allowing users to filter by document format. When both a category and a format are active, the intersection is shown.

### Design

- Render below the category filter row, separated by a subtle gap (12px)
- Same pill style as category filters but at 10px font-size (vs 11px for categories) with no uppercase transform — visually subordinate
- An "All formats" button resets the format filter (active by default)
- Filter logic: `activeCategory AND activeFormat` — both must match when both are set
- No colour theming — format pills use `--faint`/`--dim` colouring with a `--border` border; active state uses `--text` and `--border-light` (neutral, not gold)
- Only show formats that exist in the current category-filtered set (dynamic list)

### Implementation

- Add `activeFormat` state (`$state<string>('all')`)
- Derive `availableFormats` from the category-filtered documents
- Derive `filtered` as the intersection of category + format filters
- Add a `<nav class="doc-explorer-format-filters">` block below the existing category nav

---

## 2. Full dates (day precision)

### Goal

Display dates as "12 Feb 2026" rather than "Feb 2026".

### Frontmatter changes

Update all 12 docs in `docs/` to use `YYYY-MM-DD` format. Use the git log to determine the best date for each file — the date of its first commit to the repository. Where a file has been substantially rewritten, use the most recent major commit date instead.

### Display changes

- Update `formatDate()` in DocumentExplorer.svelte to parse `YYYY-MM-DD` and render as `DD Mon YYYY`
- Gracefully handle the old `YYYY-MM` format as a fallback (render as `Mon YYYY` as today) — defensive, not required long-term

---

## 3. Reading time with word count

### Goal

Replace the current "1,234 words" display with "5 min read (1,234 words)".

### Implementation

- The JSON endpoint already provides both `readingTime` (minutes) and `wordCount`
- In the meta row, change the word count span to show: `{doc.readingTime} min read ({formatWordCount(doc.wordCount)})`
- Keep the `--mono` font and `--faint` colour
- For very short docs (< 1 min), show "< 1 min read"

---

## 4. Download and GitHub view links

### Goal

Add two action links in the expanded card section: a download link for the raw markdown and a "View on GitHub" link that opens in a new tab.

### Design

- Place in the expanded section, on the same row as the existing "Read full document →" link
- Use a flex row with gap, all three links sharing the same pill-button style
- "Download .md" — links to `https://raw.githubusercontent.com/urdwyrd/urd/main/docs/{slug}.md`, sets `download` attribute
- "View on GitHub ↗" — links to `https://github.com/urdwyrd/urd/blob/main/docs/{slug}.md`, opens `target="_blank"` with `rel="noopener noreferrer"`, uses the ↗ character as the external window indicator (per design brief — no icon libraries)
- Existing "Read full document →" stays as-is, remains the primary/leftmost link

### Data changes

- Add `githubUrl` and `downloadUrl` fields to the JSON endpoint, computed from `entry.id` (the filename without extension)
- Pass both URLs through to the component's Document interface

---

## Files to modify

| File | Changes |
|---|---|
| `docs/*.md` (all 12) | Update `date` frontmatter to `YYYY-MM-DD` |
| `sites/urd.dev/src/pages/documents.json.ts` | Add `githubUrl` and `downloadUrl` fields |
| `sites/urd.dev/src/components/DocumentExplorer.svelte` | All four features: format filters, date display, reading time, action links |

## Dependencies

None. No new packages required.

## Verification

- `pnpm build` succeeds with zero warnings
- All 12 document pages render at `/documents/{slug}`
- JSON endpoint includes new fields (`githubUrl`, `downloadUrl`)
- Category + format filter intersection works correctly
- Format filter list updates when category filter changes
- Dates render as "DD Mon YYYY"
- Meta row shows "X min read (N words)"
- Download link triggers file download
- GitHub link opens in new tab
- `prefers-reduced-motion` still respected
- Responsive behaviour at 520px and 980px breakpoints intact
