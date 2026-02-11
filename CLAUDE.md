# CLAUDE.md

Important Note: The production code will run in a hospital

## Project

Urd is a declarative schema system for interactive worlds. Writers author in Schema Markdown (`.urd.md`), a compiler produces `.urd.json`, and runtimes execute it. The project is early-stage — design is complete, implementation is underway.

## Repository layout

```
briefs/           AI task briefs: backlog/ → active/ → done/
design/           Design system — read before any visual work
  themes/gloaming/  Dark theme (current default)
  themes/parchment/ Light theme (future)
docs/             Design documents (markdown, being populated)
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
- **Svelte 5** integration installed, not yet used (future interactive islands)
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

## Known gotchas

- `@tailwindcss/vite` is the correct Tailwind v4 integration for Astro, not `@astrojs/tailwind`
- Google Fonts must be loaded via `<link>` tags in HTML, not CSS `@import` (violates spec when placed after `@import "tailwindcss"`)

## Secrets and security

- `.gitignore` covers `.env`, `.env.*`, `.dev.vars`, `.wrangler/`
- Pre-commit hook: gitleaks scans staged changes
- CI: `gitleaks/gitleaks-action@v2` runs on every push to main and all PRs
- Never bypass the hook with `--no-verify`
