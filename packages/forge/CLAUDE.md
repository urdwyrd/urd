# CLAUDE.md — Urd Forge

> This supplements the root `CLAUDE.md`. Both apply. If they conflict, this file wins for work inside `packages/forge/`.

## What this is

Urd Forge is a desktop IDE for Urd Schema Markdown, built with Tauri 2 + Svelte 5 + Rust. The architecture and brief is fully specified in `briefs/backlog/urd-forge-application-architecture.md` (the "architecture doc"). That document is the specification — read it before writing code.

## Monorepo context

Forge lives at `packages/forge/` inside the Urd monorepo. It is a pnpm workspace member via the root `pnpm-workspace.yaml` pattern `packages/*`.

**All paths in the architecture doc (§22) are relative to `packages/forge/`.** When the doc says `src-tauri/`, it means `packages/forge/src-tauri/`. When it says `src/lib/framework/`, it means `packages/forge/src/lib/framework/`.

```
<repo-root>/                     # monorepo root (parent of packages/)
├── packages/
│   ├── compiler/                # urd-compiler Rust crate (existing)
│   ├── grammar/                 # PEG grammar + pest parser (existing)
│   ├── lsp/                     # LSP server — depends on ../compiler (existing)
│   ├── mcp/                     # MCP semantic interface (existing)
│   ├── schema/                  # JSON Schema (existing)
│   └── forge/                   # ← THIS PACKAGE — Tauri desktop IDE
│       ├── src-tauri/           # Rust backend (Tauri + urd-compiler integration)
│       │   ├── Cargo.toml       # depends on urd-compiler = { path = "../../compiler" }
│       │   ├── tauri.conf.json
│       │   └── src/
│       ├── src/                 # Svelte 5 frontend
│       │   ├── lib/
│       │   │   ├── framework/   # Generic IDE framework (no Urd imports)
│       │   │   └── app/         # Urd-specific views, projections, commands
│       │   ├── App.svelte
│       │   └── main.ts
│       ├── fixtures/            # CompilerOutput JSON test fixtures
│       ├── package.json
│       ├── vite.config.ts
│       ├── svelte.config.js
│       ├── tsconfig.json
│       └── CLAUDE.md            # this file
```

## Commands

All commands run from the **monorepo root** (`f:/urd`), using pnpm filter:

```bash
# Development
pnpm --filter forge dev           # Tauri dev mode (hot-reload frontend + Rust backend)
pnpm --filter forge build         # Production build
pnpm --filter forge preview       # Preview production build

# Testing
pnpm --filter forge test          # Vitest — frontend unit tests
pnpm --filter forge test:rust     # cargo test for src-tauri/

# Linting
pnpm --filter forge lint          # ESLint + svelte-check
pnpm --filter forge typecheck     # TypeScript strict mode check
```

These scripts are defined in `packages/forge/package.json`, not the root. The root `package.json` may get convenience aliases later (like `pnpm forge:dev`) but is not required — `--filter forge` is the canonical pattern.

### Rust commands

The Tauri Rust backend has its own `Cargo.toml` at `packages/forge/src-tauri/Cargo.toml`. Direct cargo commands target it:

```bash
cargo test --manifest-path packages/forge/src-tauri/Cargo.toml
cargo build --release --manifest-path packages/forge/src-tauri/Cargo.toml
```

But prefer `pnpm --filter forge test:rust` which wraps these.

### Compiler dependency

The Forge Tauri backend depends on the Urd compiler as a local crate:

```toml
# packages/forge/src-tauri/Cargo.toml
[dependencies]
urd-compiler = { path = "../../compiler" }
```

This means the compiler is compiled directly into the Tauri binary — no subprocess, no IPC to a separate compiler process. The compiler runs as Rust functions inside the Tauri process.

## Architecture doc rules

1. **Read the architecture doc first.** It is at `briefs/backlog/urd-forge-application-architecture.md`. It specifies every interface, data type, bus channel, command, and implementation phase. Do not invent patterns that contradict it.

2. **Follow the implementation phases.** The doc defines 8 phases with 86+ steps. Build in order. Do not skip ahead.

3. **Framework vs application boundary is enforced.** `src/lib/framework/` must never import from `src/lib/app/`. This is the most important structural rule. If you need something from `app/` in `framework/`, the abstraction is wrong — define an interface in `framework/` and implement it in `app/`.

4. **Paths in the doc are relative.** `src/lib/framework/layout/ZoneTree.ts` means `packages/forge/src/lib/framework/layout/ZoneTree.ts`. The doc's §22 directory tree is rooted at `packages/forge/`.

5. **Fixtures are shared with the compiler.** The golden fixture (`fixtures/basic-world.json`) is validated by both `cargo test` (Rust deserialisation) and `pnpm test` (TypeScript deserialisation). It is the schema contract.

## package.json shape

```jsonc
{
  "name": "forge",
  "private": true,
  "version": "0.0.1",
  "type": "module",
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "preview": "vite preview",
    "test": "vitest run",
    "test:rust": "cargo test --manifest-path src-tauri/Cargo.toml",
    "lint": "eslint src/ && svelte-check",
    "typecheck": "svelte-check --tsconfig tsconfig.json"
  }
}
```

## Conventions (inherited from root CLAUDE.md)

- **British English** spelling: behaviour, colour, serialisation, initialise
- **pnpm 10** — never use npm or yarn
- **Commit frequently** — after each logical unit of work, building cleanly
- **Briefs workflow** — significant tasks get a brief in `briefs/`. Forge tasks follow the same `backlog/` → `active/` → `done/` flow with the same format as existing briefs
- **No secrets in code**

## Forge-specific conventions

- **Svelte 5 runes** — use `$state`, `$derived`, `$effect`. No Svelte 4 stores.
- **TypeScript strict mode** — no `any`, no `as` casts except at IPC boundary deserialisation
- **Imports** — use `$lib/framework/...` and `$lib/app/...` aliases. Never relative paths that cross the framework/app boundary.
- **CSS** — Tailwind core utility classes only (no compiler). Forge theme tokens use CSS custom properties (`--forge-*`), defined in the framework theme engine.
- **Tests** — Vitest for frontend. Every new service, reducer, or projection gets a test file. Use the test helpers from the architecture doc (TestBus, TestCommandRegistry, createTestTree).
- **Bus discipline** — signals only, never data. 4KB dev-mode payload assertion. If you need to carry data, it goes in a projection.
- **Commands** — every user-visible action is a command. Commands return `UndoAction` hooks.

## Known integration points

| Forge component | Depends on | How |
|---|---|---|
| `src-tauri/src/compiler/bridge.rs` | `urd-compiler` crate | `use urd_compiler::{compile_source, CompilationResult}` — wraps for IPC serialisation |
| `fixtures/basic-world.json` | Compiler output schema | Generated to match `CompilerOutput` TypeScript type AND Rust `CompilerOutput` struct |
| Future: `src-tauri/src/commands/analysis.rs` | `urd-compiler` FactSet, PropertyDependencyIndex | Direct Rust access to compiler data structures |

## What not to do

- Do not add Forge scripts to the root `package.json` unless asked — use `pnpm --filter forge`
- Do not create a Cargo workspace at the monorepo root — each Rust package is independent with path deps
- Do not use `npm`, `yarn`, or `npx`
- Do not import from `framework/` into `app/` (wrong direction) or from `app/` into `framework/`
- Do not put Urd-specific types in `framework/` — define interfaces there, implement in `app/`
- Do not bypass the bus for view-to-view communication
- Do not build your own indexes in views — that belongs in projections or QueryService
