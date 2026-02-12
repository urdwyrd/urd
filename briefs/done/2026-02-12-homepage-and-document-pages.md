# Brief: Homepage and Document Pages

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-12
**Status:** Complete

### What was done

- Installed `rehype-slug` and `rehype-autolink-headings` as dependencies; added `github-slugger` as a direct dependency for TOC ID generation
- Created `sites/urd.dev/src/lib/colours.ts` — shared `categoryColours` map extracted from `documents.json.ts`
- Updated `sites/urd.dev/astro.config.mjs` — registered rehype plugins with `behavior: "wrap"` for autolink headings
- Created `sites/urd.dev/src/components/Nav.astro` — zero-JS nav bar with "URD" wordmark, "Documents" anchor link, and "GitHub" external link
- Added `.prose` typography styles to `sites/urd.dev/src/styles/global.css` — headings (H1–H4), body text, code blocks, tables, blockquotes, lists, links, HR, images
- Created `sites/urd.dev/src/layouts/document.astro` — two-column layout with sticky TOC sidebar (raised panel pattern) and prose content area, collapses to single column at 980px
- Created `sites/urd.dev/src/pages/documents/[slug].astro` — dynamic route using `getStaticPaths()` from `designDocs` collection, `extractToc()` function using `github-slugger` to match `rehype-slug` IDs, renders Content inside DocumentLayout
- Updated `sites/urd.dev/src/pages/index.astro` — added Nav component, removed "Coming soon" text and CSS, added `id="documents"` anchor to the explorer section
- Updated `sites/urd.dev/src/pages/documents.json.ts` — imports colours from shared module
- Completed document explorer brief execution record and moved to `briefs/done/`

### Deviations from brief

- None — implementation followed the brief closely

### Issues encountered

- `github-slugger` was in the pnpm store as a transitive dependency but not hoisted; added as a direct dependency to resolve imports
- Persistent tool parameter issue when removing the "coming-soon" HTML line; resolved by using a different edit context

### Notes for next brief

- TOC links match rendered heading IDs — verified on the architecture document page (41 TOC links, all matching)
- All 9 document pages generate correctly under `dist/documents/`
- The TOC sidebar could benefit from active-section highlighting (vanilla JS scroll listener) in a future brief
- Some documents have very long TOCs (40+ entries for architecture.md) — the sticky panel with overflow-y auto handles this but a collapsible TOC could improve the experience
- Prose styles don't yet handle definition lists or footnotes — add if needed when documents use those patterns

---

## Context

The site has outgrown its "Coming Soon" page. With 9 design documents, a content pipeline, and a working DocumentExplorer component, it needs a real homepage, individual document pages, and basic navigation.

The homepage will evolve into a dashboard — the DocumentExplorer is its centrepiece, not a secondary feature below a hero. Individual document pages at `/documents/{slug}` should render full markdown content with a table of contents. A minimal nav bar ties the pages together.

### Prerequisites

- Content pipeline complete: `designDocs` collection with 9 documents, `/documents.json` endpoint
- DocumentExplorer.svelte component functional (fetches JSON, filters, expands cards)
- DocumentExplorer card links already point to `/documents/{slug}` — those routes just don't exist yet
- Global CSS variables defined in `global.css` (surface, text, brand, taxonomy tokens)

### Design References

Before implementing, read:

1. `design/themes/gloaming/design-brief.md` — canonical visual spec
2. `design/urd-document-taxonomy.md` — seven-category colour system

---

## Implementation Steps

### Step 1: Install dependencies

Add `rehype-slug` and `rehype-autolink-headings` to the urd.dev site. These are rehype plugins that add `id` attributes to rendered headings and make them linkable — required for the table of contents.

```bash
pnpm add rehype-slug rehype-autolink-headings --filter urd.dev
```

Register them in `astro.config.mjs` under `markdown.rehypePlugins`. Use `behavior: "wrap"` for autolink headings so clicking anywhere on the heading text navigates.

`github-slugger` (transitive dependency of `rehype-slug`) should be imported directly for TOC ID generation — this ensures the TOC link IDs match the IDs that `rehype-slug` generates in the rendered HTML.

### Step 2: Extract shared colour map

Create `sites/urd.dev/src/lib/colours.ts` containing the `categoryColours` map (category → hex). This is currently duplicated inline in `documents.json.ts` and will also be needed by the slug page.

Update `documents.json.ts` to import from this shared module.

### Step 3: Create the Nav component

Create `sites/urd.dev/src/components/Nav.astro` — a zero-JS Astro component.

```
┌──────────────────────────────────────────────────────────┐
│  URD                                  Documents  GitHub ↗ │
└──────────────────────────────────────────────────────────┘
```

- **Left:** "URD" in Georgia (wordmark font), gold colour, links to `/`
- **Right:** "Documents" links to `/#documents`, "GitHub ↗" links to the repo (external, `target="_blank"`)
- Styled with `--display` font for links, `--dim`/`--text` colours, `--border` bottom separator
- No hamburger menu, no dropdowns — links stay visible at all sizes
- Mobile: reduced padding only

### Step 4: Update the homepage

Modify `sites/urd.dev/src/pages/index.astro`:

1. Import and render `<Nav />` before the hero
2. Remove the `<p class="coming-soon">Coming soon</p>` and its `.coming-soon` CSS
3. Add `id="documents"` to the `<section class="documents-section">` so the nav link `/#documents` works
4. Keep the hero as-is (logos, tagline, description, GitHub link) — it stays compact as an intro above the DocumentExplorer

The DocumentExplorer remains the main feature of the page. The hero introduces the project; the explorer is where users spend time.

### Step 5: Add prose styles

Add `.prose` styles to `sites/urd.dev/src/styles/global.css` for rendered markdown content on document pages.

These styles are applied inside a `.prose` wrapper div so they don't affect the rest of the site:

- **H1**: Outfit 14px, uppercase, `--gold-dim` — these are document series markers like "URD" or "WYRD"; style as subtle labels, not page titles
- **H2**: Outfit 22–28px (clamped), 600 weight, `--text`, border-bottom separator, 48px top margin
- **H3**: Outfit 17–20px (clamped), 600 weight, `--text`, 32px top margin
- **H4**: Outfit 16px, 600 weight, `--dim`
- **Body/paragraphs**: Source Serif 4 20px, `--dim`, line-height 1.7
- **Links**: `--gold` colour, underline with `--gold-dark` underline colour
- **Inline code**: JetBrains Mono 0.85em, `--deep` background, 1px `--border`
- **Code blocks (pre)**: `--deep` background, 8px radius, 14px font size, overflow-x auto
- **Tables**: collapsed borders, `--raise` header background, `--border` cell borders
- **Blockquotes**: 2px `--gold-dim` left border, italic, `--faint` text
- **Lists**: 1.5em left padding, 0.5em gap between items
- **HR**: 1px `--border`, 48px vertical margin
- **Strong**: `--text` colour (brighter than surrounding `--dim` body text)

### Step 6: Create the document layout

Create `sites/urd.dev/src/layouts/document.astro` — wraps `base.astro`.

Structure:

```
┌─ Nav ────────────────────────────────────────────────────┐
├──────────────┬───────────────────────────────────────────┤
│  CONTENTS    │  [FORMAT PILL]  category  date  status    │
│              │  Document Title                           │
│  § Section 1 │  Description text                        │
│  § Section 2 │                                           │
│    └ Sub 2a  │  ─── rendered markdown (.prose) ────────  │
│    └ Sub 2b  │                                           │
│  § Section 3 │  Body paragraph text...                   │
│  ...         │                                           │
└──────────────┴───────────────────────────────────────────┘
```

- **Two-column CSS grid**: ~240px TOC sidebar + 1fr content area
- **TOC sidebar**: sticky positioning, uses the raised panel pattern from design brief §5 (`--raise` background, subtle box-shadow). "Contents" heading in gold. Links in mono font. H3 entries indented. Left-border hover state in gold.
- **Document header**: category pill badge (coloured), format, date, status, title as `<h1>`, description
- **Content area**: `<slot />` renders into a `.prose` div
- **Responsive**: below 980px, collapses to single column — TOC appears inline above the content

Props:

```typescript
interface Props {
  title: string;
  description: string;
  category: string;
  format: string;
  date: string;
  status: string;
  toc: { depth: number; text: string; id: string }[];
  colour: string;
}
```

### Step 7: Create the document slug page

Create `sites/urd.dev/src/pages/documents/[slug].astro` — the dynamic route.

**`getStaticPaths()`**: reads all entries from the `designDocs` collection. Maps each entry to `{ params: { slug: entry.data.slug }, props: { entry } }`.

**`extractToc(markdown)`**: a build-time function that:
1. Strips fenced code blocks (` ```...``` `) and indented code blocks (4-space/tab indented lines)
2. Matches all headings (H1–H6) with a regex to keep `github-slugger` counter in sync with `rehype-slug`
3. Generates IDs using `github-slugger` (same algorithm as `rehype-slug`)
4. Filters to only H2 and H3 for the displayed table of contents

**Why process all headings but only display H2/H3:** `rehype-slug` processes every heading in the document and its internal slugger state advances for each one. If we only processed H2/H3 through the slugger, duplicate heading texts would get different suffixes (e.g. `responsibilities` vs `responsibilities-1`) than what `rehype-slug` generates.

**Rendering**: uses `render(entry)` from Astro to get `<Content />`, passes TOC data and metadata to the document layout.

---

## Files to Create/Modify

| File | Action |
|------|--------|
| `sites/urd.dev/package.json` | Modify — add rehype-slug, rehype-autolink-headings |
| `sites/urd.dev/astro.config.mjs` | Modify — register rehype plugins |
| `sites/urd.dev/src/lib/colours.ts` | **Create** — shared category colour map |
| `sites/urd.dev/src/components/Nav.astro` | **Create** — minimal site navigation |
| `sites/urd.dev/src/layouts/document.astro` | **Create** — document page layout with TOC |
| `sites/urd.dev/src/pages/documents/[slug].astro` | **Create** — dynamic route for document pages |
| `sites/urd.dev/src/styles/global.css` | Modify — add `.prose` typography styles |
| `sites/urd.dev/src/pages/index.astro` | Modify — remove "Coming soon", add Nav, anchor id |
| `sites/urd.dev/src/pages/documents.json.ts` | Modify — import from shared colours |

---

## Verification

1. `pnpm build` succeeds — all 9 documents generate static pages under `dist/documents/`
2. Homepage: "Coming soon" gone, Nav visible, DocumentExplorer loads and filters work
3. Nav: "Documents" scrolls to explorer section, "GitHub" opens repo in new tab
4. Document cards: "Read full document →" navigates to `/documents/{slug}`
5. Document pages: correct title, category pill, prose typography, code blocks render
6. TOC: links scroll to the correct heading (IDs match between TOC and rendered HTML)
7. Responsive: TOC collapses inline above content below 980px
8. No regressions to DocumentExplorer component

---

## What NOT To Do

- Do not add search, tag filtering, or complex features — this is a minimal template
- Do not add breadcrumbs — the design brief explicitly says no breadcrumbs
- Do not use icon libraries — text characters only (→, ◆, ▸, ↗)
- Do not add client JS for the TOC — it's built at build time, plain anchor links
- Do not add a footer — not in scope
- Do not restructure the homepage layout beyond removing "Coming soon" and adding Nav
- Do not hardcode any document data — everything comes from the content collection

---

## Future Considerations

- **Active TOC highlighting**: a small vanilla JS script that highlights the current section as the user scrolls
- **Dashboard evolution**: the homepage will gain more sections as the project matures
- **Document navigation**: prev/next links between documents in the same category
- **Print styles**: `.prose` content should print cleanly
