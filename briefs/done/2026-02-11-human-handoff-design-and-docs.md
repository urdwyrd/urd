# Human Handoff: Place Design Files and Convert Documents

> **This is a human task, not an AI brief.** The steps below must be completed by the project owner before the next AI brief can execute. Tick each item off as you go.

---

## Why This Comes First

The scaffold is live — `pnpm dev` runs, the Coming Soon page renders with the Gloaming theme, and the build pipeline works end-to-end. But the next AI brief (wiring up the content pipeline and building the document index) depends on two things only you can provide:

1. **The design system reference files** — the canonical briefs and living HTML references that govern all future visual work.
2. **The design documents as markdown** — the 9 .docx files converted to `.md` with the correct frontmatter schema.

---

## Step 1: Place Design System Files

Copy your authored design files into the directory structure the scaffold created:

| Source file | Destination |
|-------------|-------------|
| Document colour taxonomy | `design/urd-document-taxonomy.md` |
| Gloaming theme brief | `design/themes/gloaming/design-brief.md` |
| Gloaming design system HTML | `design/themes/gloaming/design-system.html` |
| Parchment theme brief | `design/themes/parchment/design-brief.md` |
| Parchment design system HTML | `design/themes/parchment/design-system.html` |

Once placed, you can delete the `.gitkeep` files in those directories — they were only there to keep the empty folders tracked by Git.

- [X] All files placed
- [X] `.gitkeep` files removed from populated directories
- [X] Committed: `git add design/ && git commit -m "Place design system reference files"`

---

## Step 2: Convert Design Documents to Markdown

The ten design documents currently exist as `.docx` files. Each needs to be converted to markdown and placed in `/docs` with YAML frontmatter matching the schema defined in the scaffold brief.

### Frontmatter Template

Every document needs this frontmatter block at the top:

```yaml
---
title: "Full Document Title"
slug: "kebab-case-filename-without-extension"
description: "One-paragraph summary of what the document covers."
category: "contract"
format: "Technical Specification"
date: "2026-02"
status: "v0.1 complete"
order: 1
tags:
  - relevant
  - keywords
details:
  - "Key point 1 shown in expanded card view"
  - "Key point 2"
  - "Key point 3"
---
```

**Category must be one of:** `research` | `contract` | `authoring` | `architecture` | `runtime` | `validation`

### Target Files

| Filename | Title | Category | Format |
|----------|-------|----------|--------|
| `docs/world-framework-research.md` | Landscape Analysis & Gap Assessment | research | Research Report |
| `docs/pain-points-report.md` | Developer Pain Points Report | research | Forum-Sourced Research |
| `docs/schema-spec.md` | Urd World Schema Specification v1 | contract | Technical Specification |
| `docs/nested-dialogue.md` | Nested Dialogue Architecture | contract | Design Document |
| `docs/schema-markdown-v01.md` | Schema Markdown Syntax Specification | authoring | Syntax Specification |
| `docs/architecture.md` | Architecture & Technical Design | architecture | System Blueprint |
| `docs/system-architecture.md` | System Architecture Diagram | architecture | Interactive Visualisation |
| `docs/wyrd-runtime.md` | Wyrd Reference Runtime | runtime | Runtime Specification |
| `docs/test-case-strategy.md` | Test Case Strategy | validation | Validation Plan |
| `docs/product-vision.md` | Urd + Wyrd Product Vision v2.0 | strategy | Product Strategy |

### Conversion Tips

- **Pandoc** can handle the bulk conversion: `pandoc input.docx -t markdown -o output.md`
- You will need to manually add the YAML frontmatter block after conversion
- Review the markdown output — Pandoc sometimes mangles tables, nested lists, or code blocks
- British English spelling throughout (behaviour, colour, visualisation)
- The `slug` should match the filename without the `.md` extension
- Set `order` to control display position within each category (1 = first)
- Write 3–5 `details` bullets that capture the most important points — these appear in the expanded card view on the site

- [X] All nine documents converted to markdown
- [X] YAML frontmatter added to each document (AI-generated from document content)
- [X] Reviewed for formatting issues
- [X] Placed in `/docs`
- [X] `.gitkeep` removed from `docs/`
- [ ] Committed

**Note:** 9 of 10 documents were converted. `system-architecture.md` (Interactive Visualisation) was not converted — it is a diagram, not a prose document from .docx. Some filenames differ from the original spec: `schema-markdown.md` (not `schema-markdown-v01.md`), `schema-spec.md` (not `schema-spec-v01.md`), `urd-test-case-strategy.md` (not `test-case-strategy.md`), `wyrd-reference-runtime.md` (not `wyrd-runtime.md`). Slugs match the actual filenames.

---

## Step 3: Verify

Quick sanity check before handing back to AI:

```bash
# Everything should be clean
git status

# The repo should still build
pnpm build
```

- [X] `pnpm build` succeeds

---

## What Happens Next

Once these files are in place, the next AI brief can:

1. Wire up Astro content collections to read `/docs` markdown files
2. Generate the `documents.json` manifest at build time (with computed fields: word count, reading time, excerpt, colour mapping)
3. Build the interactive document index page with category filtering, search, and expandable artifact cards — all following the Gloaming design brief

That brief will be created in `briefs/backlog/` and is ready to execute the moment this handoff is complete.
