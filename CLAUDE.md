# CLAUDE.md

**Important: The production code will run in a hospital.** Urd and Wyrd will be used as a clinical narrative and decision support layer that sits between raw medical systems and the humans who must make treatment decisions. Code quality, correctness, and safety are not aspirational — they are mandatory.

## Project

Urd is a declarative schema system for interactive worlds. Writers author in Schema Markdown (`.urd.md`), a compiler produces `.urd.json`, and runtimes execute it. The project is early-stage — design is complete, implementation is underway.

## Repository layout

```
briefs/           AI task briefs: backlog/ → active/ → done/
content/          Site content (markdown + frontmatter, Astro collections)
  documents/        Design docs with frontmatter (Astro collection source)
  reviews/          AI peer reviews of the specification
  updates/          Project log / changelog entries
design/           Design system — read before any visual work
  themes/gloaming/  Dark theme (current default)
  themes/parchment/ Light theme (future)
docs/             Design documents (clean markdown, no frontmatter — for GitHub/download)
packages/         Shared packages (future — compiler, runtime, etc.)
sites/urd.dev/    Astro 5 static site — development journal
```

pnpm monorepo. Workspaces: `sites/*`, `packages/*`.

## Commands

```bash
pnpm dev          # Astro dev server (urd.dev)
pnpm build        # Production build → sites/urd.dev/dist/
pnpm preview      # Preview production build
```

## Tech stack

- **Astro 5** with TypeScript strict mode — static output, no SSR
- **Tailwind CSS v4** via `@tailwindcss/vite` (not `@astrojs/tailwind` — it doesn't support v4)
- **Svelte 5** interactive islands (DocumentExplorer, PeerReview, ProjectLog) hydrated client-side
- **Cloudflare Pages** deployment via GitHub Actions (path-filtered to `sites/urd.dev/**`)
- **pnpm 10** — never use npm or yarn

## Design system

**Before writing any visual code, read:**
1. `design/themes/gloaming/design-brief.md` — canonical reference for all colour, typography, spacing, animation, and layout decisions
2. `design/urd-document-taxonomy.md` — six-category colour system for documents

The design brief is the specification. It is the single source of truth.

### Key rules

- Colours are CSS custom properties on `:root` — defined in `sites/urd.dev/src/styles/global.css`
- Georgia is **exclusively** for the brand wordmark — never for headings or body
- Outfit for headings/labels, Source Serif 4 for body, JetBrains Mono for code
- No icon libraries — text characters only: → ◆ ▸ ↗
- Max animation opacity for rune canvas: 0.05 (hard limit)
- Respect `prefers-reduced-motion` in all animations

## Conventions

- **British English** spelling throughout (behaviour, colour, visualisation)
- **Briefs workflow**: every significant task gets a brief in `briefs/`. File naming: `YYYY-MM-DD-short-description.md`. Briefs move `backlog/` → `active/` → `done/` with a filled execution record
- **No secrets in code** — tokens and keys live in GitHub Actions secrets or `.dev.vars` (gitignored). A gitleaks pre-commit hook blocks commits containing secrets
- **Commit frequently** — after each logical unit of work, with the project building cleanly at each point
- **Minimise client JS** — zero-JS pages where possible, vanilla JS preferred over framework code for simple interactions
- **Content architecture**: `content/` holds site content with each subdirectory as its own Astro collection (e.g. `content/documents/` → `designDocs`, `content/reviews/` → `reviews`). Each collection has a JSON endpoint under `src/pages/` and a Svelte island consumer. `docs/` contains the same design documents without frontmatter for clean GitHub viewing and download. When a document is updated, both `docs/` and `content/documents/` must be kept in sync

## Known gotchas

- `@tailwindcss/vite` is the correct Tailwind v4 integration for Astro, not `@astrojs/tailwind`
- Fonts are self-hosted in `sites/urd.dev/public/fonts/` with `@font-face` in `global.css` — do not add Google Fonts `<link>` tags

## Secrets and security

- `.gitignore` covers `.env`, `.env.*`, `.dev.vars`, `.wrangler/`
- Pre-commit hook: gitleaks scans staged changes
- CI: `gitleaks/gitleaks-action@v2` runs on every push to main and all PRs
- Never bypass the hook with `--no-verify`
