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
packages/
  compiler/         Rust compiler — 5-phase pipeline (.urd.md → .urd.json)
  grammar/          PEG grammar reference + pest parser + validation corpus
scripts/          Build tooling — test report generator, benchmark harness
sites/urd.dev/    Astro 5 static site — development journal
```

pnpm monorepo. Workspaces: `sites/*`, `packages/*`.

## Commands

```bash
# Site
pnpm dev                    # Astro dev server (urd.dev)
pnpm build                  # Production build → sites/urd.dev/dist/
pnpm preview                # Preview production build
pnpm build:full             # Run compiler tests + copy report + build site

# Compiler
pnpm compiler:test          # Run tests + benchmarks, generate test-report.json
pnpm compiler:test:raw      # Raw cargo test output
pnpm compiler:build         # Release build of the `urd` CLI binary
pnpm compiler:bench         # Release benchmarks + update report
pnpm compiler:wasm:check    # Verify WASM target compiles

# Grammar & Schema
pnpm grammar:test           # PEG validation corpus
pnpm schema:test            # JSON Schema validation
```

## Tech stack

- **Astro 5** with TypeScript strict mode — static output, no SSR
- **Tailwind CSS v4** via `@tailwindcss/vite` (not `@astrojs/tailwind` — it doesn't support v4)
- **Svelte 5** interactive islands (DocumentExplorer, PeerReview, ProjectLog, CompilerStatus, Presentation, ProjectTimeline) hydrated client-side
- **Cloudflare Pages** deployment via GitHub Actions (path-filtered to `sites/urd.dev/**`)
- **Rust** — compiler (`packages/compiler/`): five-phase pipeline with native CLI (`urd`) and WASM target; pest grammar (`packages/grammar/`)
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

## Compiler architecture

The compiler (`packages/compiler/`) has two entry points:
- **Native CLI** (`src/bin/main.rs`) — the `urd` binary, reads files from disk via `OsFileReader`
- **WASM bindings** (`src/wasm.rs`) — gated behind `#[cfg(feature = "wasm")]`, uses `StubFileReader` (no filesystem)

Core API in `lib.rs`:
- `compile_source(filename, source)` — single-file mode (WASM-safe)
- `compile_source_with_reader(filename, source, reader)` — custom import resolution
- `compile(entry_file)` — native-only convenience wrapper, gated with `#[cfg(not(target_arch = "wasm32"))]`

The `FileReader` trait in `import/mod.rs` abstracts filesystem access. All `std::fs` usage is confined to `OsFileReader` and the CLI binary.

Test report data flows: `compiler-test-report.mjs` → `packages/compiler/test-report.json` → copied to `sites/urd.dev/src/data/compiler-test-report.json` → consumed by the CompilerStatus Svelte island.

## Known gotchas

- `@tailwindcss/vite` is the correct Tailwind v4 integration for Astro, not `@astrojs/tailwind`
- Fonts are self-hosted in `sites/urd.dev/public/fonts/` with `@font-face` in `global.css` — do not add Google Fonts `<link>` tags
- The compiler's `Cargo.toml` uses `crate-type = ["cdylib", "rlib"]` for dual-target support — both are needed
- `wasm-bindgen` is an optional dependency behind the `wasm` feature flag — never add it to default dependencies

## Secrets and security

- `.gitignore` covers `.env`, `.env.*`, `.dev.vars`, `.wrangler/`
- Pre-commit hook: gitleaks scans staged changes
- CI: `gitleaks/gitleaks-action@v2` runs on every push to main and all PRs
- Never bypass the hook with `--no-verify`
