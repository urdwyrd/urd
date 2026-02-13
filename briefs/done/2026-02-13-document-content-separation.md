# Brief: Separate Clean Docs from Content Collection

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-13
**Status:** Complete

### What was done

- Created `content/documents/` directory with all 12 design documents, each retaining full YAML frontmatter (title, slug, description, category, format, date, status, order, tags, details).
- Stripped YAML frontmatter from all 12 `docs/` files so they begin directly with markdown content — clean for GitHub rendering and raw downloads.
- Updated `content.config.ts` to point the `designDocs` collection loader at `../../content/documents` instead of `../../docs`.
- Verified `entry.id` values remain unchanged (filename stems), so `githubUrl` and `downloadUrl` in `documents.json.ts` still correctly point to `docs/` (the clean versions).
- Verified all 12 document pages render correctly at `/documents/{slug}` with TOC.
- Updated `CLAUDE.md` repo layout to include `content/documents/` and added sync note to content architecture convention.
- Build passes cleanly: 13 pages + 3 JSON endpoints.

### What changed from the brief

- Nothing. Implementation followed the brief exactly.

---

**Created:** 2026-02-13

## Problem

The 12 design documents in `docs/` currently carry YAML frontmatter (title, slug, description, category, format, date, status, order, tags, details). This frontmatter is required by the Astro content collection that powers the site, but it creates two problems:

1. **GitHub rendering** — anyone viewing the files on GitHub sees raw YAML before the document prose. The "View on GitHub" links in DocumentExplorer take users straight to this cluttered view.
2. **Raw downloads** — the "Download .md" action fetches from `raw.githubusercontent.com`, so the downloaded file includes frontmatter that has no meaning outside the site.

The `docs/` folder should contain clean, portable markdown that reads well on GitHub, in any editor, and when downloaded. Site-specific metadata belongs in the content pipeline, not in the canonical documents.

## Solution

Create a `content/documents/` directory (alongside the existing `content/reviews/` and `content/updates/`) that holds frontmatter-enriched copies of the documents. Strip all frontmatter from `docs/`. Update the Astro collection to read from `content/documents/` instead of `docs/`.

GitHub and download URLs continue to point at `docs/` — the clean versions.

## Detailed changes

### 1. Create `content/documents/`

For each of the 12 files in `docs/`, create a corresponding file in `content/documents/` with the same filename. Each file contains:

- The full YAML frontmatter block (title, slug, description, category, format, date, status, order, tags, details) — exactly as it exists today in `docs/`
- The markdown body — exactly as it exists today in `docs/`

This is effectively a copy of the current `docs/` files, unchanged.

**Files to create:**

| Source | Destination |
|--------|-------------|
| `docs/architecture.md` | `content/documents/architecture.md` |
| `docs/schema-markdown.md` | `content/documents/schema-markdown.md` |
| `docs/schema-spec.md` | `content/documents/schema-spec.md` |
| `docs/nested-dialogue.md` | `content/documents/nested-dialogue.md` |
| `docs/product-vision.md` | `content/documents/product-vision.md` |
| `docs/urd-test-case-strategy.md` | `content/documents/urd-test-case-strategy.md` |
| `docs/world-framework-research.md` | `content/documents/world-framework-research.md` |
| `docs/reference-card-engine-developers.md` | `content/documents/reference-card-engine-developers.md` |
| `docs/reference-card-writers.md` | `content/documents/reference-card-writers.md` |
| `docs/wyrd-reference-runtime.md` | `content/documents/wyrd-reference-runtime.md` |
| `docs/pain-points-report.md` | `content/documents/pain-points-report.md` |
| `docs/future-proposals.md` | `content/documents/future-proposals.md` |

### 2. Strip frontmatter from `docs/`

Remove the `---` delimited YAML block from the top of each file in `docs/`. The files should begin directly with markdown content. No other changes to the document body.

### 3. Update `content.config.ts`

Change the `designDocs` loader base path:

```typescript
// Before
const designDocs = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../docs' }),
  // ...
});

// After
const designDocs = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../content/documents' }),
  // ...
});
```

Schema stays the same — the frontmatter structure is unchanged.

### 4. Verify `documents.json.ts` URLs

The GitHub and download URLs in `documents.json.ts` already point to `docs/`:

```typescript
githubUrl: `https://github.com/urdwyrd/urd/blob/main/docs/${entry.id}.md`,
downloadUrl: `https://raw.githubusercontent.com/urdwyrd/urd/main/docs/${entry.id}.md`,
```

These are correct and should remain unchanged. After this refactor, clicking "View on GitHub" or "Download .md" takes users to the clean, frontmatter-free versions in `docs/`.

**Caveat:** Astro's `entry.id` is derived from the file path relative to the collection base. When the base changes from `../../docs` to `../../content/documents`, the `entry.id` values should remain the same (just the filename stem, e.g. `architecture`). Verify this during implementation — if the IDs change, update the URL template accordingly.

### 5. Verify `[slug].astro` still works

The dynamic document page at `src/pages/documents/[slug].astro` uses `getCollection('designDocs')` and renders `entry.body`. Since the collection name, schema, and body content are unchanged, this should work without modification. Verify during build.

### 6. Update `CLAUDE.md`

Add `content/documents/` to the repo layout in `CLAUDE.md`:

```
content/
  documents/    Frontmatter-enriched document copies (Astro collection source)
  reviews/      AI peer reviews
  updates/      Project log entries
```

Add a note under conventions or the content architecture section explaining the separation:

> `docs/` contains clean markdown for GitHub viewing and download. `content/documents/` contains the same documents with YAML frontmatter for the Astro content collection. When a document is updated, both copies must be kept in sync.

## File summary

| File | Action |
|------|--------|
| `content/documents/*.md` (12 files) | **Create** — copy from `docs/` with frontmatter intact |
| `docs/*.md` (12 files) | **Modify** — strip YAML frontmatter block |
| `sites/urd.dev/src/content.config.ts` | **Modify** — change `designDocs` base path |
| `sites/urd.dev/src/pages/documents.json.ts` | **Verify** — URLs still point to `docs/`, entry.id unchanged |
| `sites/urd.dev/src/pages/documents/[slug].astro` | **Verify** — renders correctly from new base |
| `CLAUDE.md` | **Modify** — update repo layout and add sync note |

## Risks

- **Sync drift:** Two copies of each document means edits to one can diverge from the other. This is an accepted trade-off — the documents are stable (v1.0 locked) and will change infrequently. A future improvement could automate sync or generate frontmatter copies from clean docs.
- **entry.id change:** If Astro's glob loader produces different IDs from the new base path, GitHub/download URLs will break. Check during build.

## Out of scope

- Automating the sync between `docs/` and `content/documents/` (future improvement)
- Changing the document schema or frontmatter fields
- Modifying the DocumentExplorer component
- Adding new documents

## Verification

1. `pnpm build` completes without errors
2. `dist/documents.json` contains all 12 documents with correct metadata
3. GitHub URLs in the JSON point to `docs/` (clean versions)
4. Each `docs/*.md` file starts with markdown content, not `---`
5. Each `content/documents/*.md` file has the full frontmatter block
6. Document pages at `/documents/{slug}` render correctly with TOC

## Execution record

| Step | Status | Notes |
|------|--------|-------|
| Create `content/documents/` with all 12 files | Done | Copied from docs/ with frontmatter intact |
| Strip frontmatter from all 12 `docs/` files | Done | All files now start with markdown content |
| Update `content.config.ts` base path | Done | Changed to `../../content/documents` |
| Verify `entry.id` values and URL correctness | Done | IDs unchanged, GitHub/download URLs point to docs/ |
| Verify document pages render correctly | Done | All 12 pages build and render with TOC |
| Update `CLAUDE.md` | Done | Added content/documents/ to layout, updated content architecture note |
| Build passes cleanly | Done | 13 pages + 3 JSON endpoints built cleanly |
