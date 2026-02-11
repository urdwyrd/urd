# Contributing

Urd is early-stage and built by one person with AI assistance. At this point the project is moving too fast and changing too fundamentally to accept outside contributions — reviewing and merging PRs would slow things down more than it helps. This file exists to document the development setup for the project owner, not to invite pull requests.

If you're interested in Urd, the best thing to do right now is watch the repo and follow progress at [urd.dev](https://urd.dev). Once the core tooling stabilises, contributions will be welcome and this guide will be updated accordingly.

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Node.js | 20+ | [nodejs.org](https://nodejs.org) |
| pnpm | 10+ | `corepack enable` (bundled with Node) |
| gitleaks | 8+ | `winget install gitleaks` · `brew install gitleaks` · [releases](https://github.com/gitleaks/gitleaks/releases) |

**gitleaks is required.** A pre-commit hook runs `gitleaks` on every commit to prevent secrets from entering the repo. If gitleaks is not installed, commits will fail. This is intentional — the project fails closed on missing tooling.

## Setup

```bash
git clone https://github.com/urdwyrd/urd.git
cd urd
pnpm install
```

`pnpm install` automatically sets up Git hooks via Husky. After install, verify the hook is in place:

```bash
cat .husky/pre-commit
# Should output: gitleaks git --pre-commit --staged --verbose
```

## Development

```bash
pnpm dev       # Start urd.dev dev server
pnpm build     # Build urd.dev for production
pnpm preview   # Preview production build locally
```

All three commands filter to the `sites/urd.dev` workspace.

## Repository Structure

```
urd/
├── briefs/            # AI task briefs — the build record
│   ├── backlog/       # Future tasks
│   ├── active/        # Currently executing (one at a time)
│   └── done/          # Completed, with execution records
├── design/            # Design system — AI reference library
│   └── themes/        # Gloaming, Parchment
├── docs/              # Design documents (markdown)
├── packages/          # Shared packages (future)
├── sites/
│   └── urd.dev/       # Astro 5 static site
├── .gitleaks.toml     # Secret scanning config
├── wrangler.toml      # Cloudflare Pages config (no secrets)
└── pnpm-workspace.yaml
```

## Briefs

Every significant task is tracked as a **brief** in the `briefs/` directory. Briefs move through `backlog/` → `active/` → `done/` and include an execution record documenting what actually happened (deviations, issues, lessons). This is the project's permanent build log.

Filename convention: `YYYY-MM-DD-short-description.md`.

## Commits

- Keep commits focused on a single logical change
- The pre-commit hook scans staged changes for secrets — do not bypass it with `--no-verify`
- CI runs a separate gitleaks scan on every push and PR as a safety net

## Secrets

Secrets live **only** in GitHub Actions secrets or local `.dev.vars` files (gitignored). Never commit tokens, keys, or credentials. The gitleaks pre-commit hook and CI workflow exist specifically to catch mistakes.

Files that must never be committed: `.env`, `.env.*`, `.dev.vars`, anything under `.wrangler/`.

## Style

- British English spelling (behaviour, colour, visualisation)
- No icon libraries — text characters only (→, ◆, ▸, ↗)
- TypeScript strict mode
- pnpm only (not npm or yarn)
