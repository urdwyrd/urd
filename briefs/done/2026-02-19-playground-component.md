# URD — Playground Component

*A brief for the interactive compiler playground on urd.dev*

February 2026 | Done

`Schema Markdown → WASM compiler → live JSON output`

> **Document status: BRIEF** — Defines scope, requirements, and acceptance criteria for the browser-based playground component. This is the Astro island that lets visitors type Schema Markdown and see compiled output in real time. It consumes the WASM compiler module already built in `packages/compiler/` (see `briefs/done/2026-02-18-wasm-dual-target.md`).

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-19
**Status:** Complete

### What was done

- Built WASM package via `cargo build --target wasm32-unknown-unknown` + `wasm-bindgen` CLI (537KB binary)
- Created vendor script (`scripts/vendor-wasm.mjs`) copying `.wasm` to `public/wasm/`, JS/TS glue to `src/lib/wasm/`
- Added `compiler:wasm:build` and `compiler:wasm:vendor` pnpm scripts; updated `build:full` pipeline
- Created `compiler-bridge.ts` — lazy WASM loader, typed API wrappers for `compile_source`/`parse_only`/`compiler_version`, byte-to-character column conversion
- Created `codemirror-urd.ts` — `StreamLanguage` mode for Schema Markdown syntax highlighting + custom Gloaming `EditorView.theme()` and `HighlightStyle`
- Created `OutputPane.svelte` — JSON view with copy button, structured diagnostics list with clickable rows, loading/error states, sub-1ms compile time precision
- Created `UrdPlayground.svelte` — CodeMirror 6 editor, custom split pane (pointer events + CSS grid), dual debounce (50ms parse / 300ms compile), mobile tab toggle, diagnostic click-to-scroll, sequence counter for stale result discard
- Created `playground.astro` page with `<UrdPlayground client:visible />`
- Added "Playground" nav link between Documents and Updates
- Installed CodeMirror 6 with individual sub-packages as direct dependencies
- Created "The Locked Garden" starter example — a multi-location game showcasing 3 types (enum, integer, string, bool, immutable), 5 entities, conditional exits with blocked messages, sticky/one-shot choices, OR conditions, nested choices, entity-targeted choices, arithmetic/assignment effects, destroy, section jumps
- Added `locked-garden.urd.md` test fixture with 20 e2e tests (66 e2e tests total, 413 compiler tests total)
- Build produces 29 pages, playground bundle ~67KB gzipped (under 150KB budget)

### What changed from the brief

- **WASM build tool:** Used `cargo build` + `wasm-bindgen` CLI instead of `wasm-pack`. Rust 1.93 renamed `--out-dir` to `--artifact-dir` (nightly-only), breaking wasm-pack 0.13/0.14. Manual build produces identical output.
- **WASM consumption:** Used vendored copy approach (the brief's fallback) rather than pnpm workspace link. Simpler, avoids Vite WASM bundler issues. Added `/* @vite-ignore */` patch in vendor script to suppress Vite build warning.
- **CodeMirror packages:** Added `@codemirror/view`, `@codemirror/state`, `@codemirror/commands`, `@codemirror/language`, `@codemirror/lint`, `@lezer/highlight` as direct dependencies. pnpm strict isolation means transitive dependencies from the `codemirror` meta-package aren't directly importable.
- **JSON output:** Uses plain `<pre><code>` with monospace styling instead of read-only CodeMirror with `@codemirror/lang-json`. Lighter weight, sufficient for v1.
- **Starter example:** Replaced the brief's tavern snippet (which had syntax that didn't compile in WASM single-file mode) with "The Locked Garden" — a richer two-location game that exercises more language features.
- **Test fixture:** Added `locked-garden.urd.md` to the compiler's e2e test suite with 20 tests. Not in the original brief scope but provides regression coverage for the starter example.
- **Deferred:** Editor lint decorations (wavy underlines on diagnostic spans) — the lint gutter is present but inline decorations are not wired up. Collapsible JSON sections deferred to a future enhancement.

---

## Purpose

The playground is the first thing a visitor interacts with on urd.dev. It answers the question "does this actually work?" with a live demonstration rather than documentation. A visitor types Schema Markdown on the left, sees compiled JSON on the right, and watches errors appear and resolve as they edit. No install, no CLI, no account.

This is the most important conversion artifact for Urd. If a visitor cannot experience the compiler in under five seconds, the project remains theoretical.

### What This Is

A responsive Svelte 5 component, deployed as an Astro island with `client:visible` hydration. It contains three concerns:

- A code editor pane (Schema Markdown input).
- An output pane (compiled JSON or diagnostics).
- The WASM compiler bridge that connects them.

### What This Is Not

- **Not the Urd IDE.** The IDE is a full authoring environment with LSP integration, file management, and Wyrd playback. The playground is a single-file demonstration tool.
- **Not a Wyrd integration.** The playground compiles but does not execute. A future "compile and play" experience that chains the compiler output into Wyrd is a separate brief.
- **Not the syntax highlighting specification.** The playground needs basic syntax highlighting for Schema Markdown. A minimal v1 token map is included in this brief. A comprehensive syntax highlighting specification (for the playground, the LSP, and third-party editors) is a separate deliverable.

## Where It Lives

```
sites/urd.dev/
  src/
    components/
      playground/
        UrdPlayground.svelte     ← The island component (top-level)
        compiler-bridge.ts       ← WASM lazy-loader and API wrapper
        codemirror-urd.ts        ← CodeMirror language mode for Schema Markdown
        OutputPane.svelte        ← JSON display + diagnostics view
    pages/
      playground.astro           ← The page that hosts the island
```

The compiler WASM module is built from `packages/compiler/` (specifically `src/wasm.rs` behind the `wasm` feature flag). The bridge module imports the WASM package at runtime.

### Design System Compliance

**Before writing any visual code, read `design/themes/gloaming/design-brief.md`.** The playground must follow the Gloaming theme:

- Colours are CSS custom properties on `:root` defined in `global.css`.
- Outfit for headings/labels, Source Serif 4 for body, JetBrains Mono for code (the editor and output pane both use JetBrains Mono).
- No icon libraries — text characters only: → ◆ ▸ ↗
- Respect `prefers-reduced-motion` in all animations.
- The CodeMirror theme must be custom-built to match Gloaming colours (dark navy background `#0e0f16` or recessed `#0a0a12`, warm text `#f2ece0`, amber for compiler-related accents). No stock dark themes.

## The Existing WASM API

The compiler already exposes three WASM functions in `src/wasm.rs` (gated behind `#[cfg(feature = "wasm")]`):

### `compile_source(source: &str) -> String`

Full five-phase pipeline. Returns a JSON string:

```json
{
  "success": true,
  "world": "{ ... compiled .urd.json as string ... }",
  "diagnostics": []
}
```

On failure:

```json
{
  "success": false,
  "world": null,
  "diagnostics": [
    {
      "severity": "error",
      "code": "URD102",
      "message": "Expected closing '---' for frontmatter block",
      "span": {
        "file": "playground.urd.md",
        "start_line": 1,
        "start_col": 1,
        "end_line": 1,
        "end_col": 1
      }
    }
  ]
}
```

The filename is hardcoded to `"playground.urd.md"` in the WASM binding. Import declarations produce URD201 diagnostics (file not found) because WASM mode uses `StubFileReader`.

### `parse_only(source: &str) -> String`

Phase 1 only. Returns:

```json
{
  "success": true,
  "diagnostics": []
}
```

No `world` field. Faster than full compilation — useful for instant syntax feedback.

### `compiler_version() -> String`

Returns the crate version from `Cargo.toml` (currently `"0.1.0"`).

### Diagnostic Shape

Every diagnostic has the same structure:

```typescript
interface Diagnostic {
  severity: 'error' | 'warning' | 'info';
  code: string;           // e.g. "URD102", "URD301"
  message: string;
  span: {
    file: string;         // always "playground.urd.md" in WASM mode
    start_line: number;   // 1-indexed
    start_col: number;    // 1-indexed, byte offset
    end_line: number;
    end_col: number;
  };
}
```

Note: the Rust `Diagnostic` struct also has `suggestion` and `related` fields, but the WASM serialiser in `wasm.rs` does not currently include them. A future enhancement could expose suggestions for richer editor integration.

### Byte Offset Warning

The compiler's `start_col` and `end_col` are **byte offsets**, not character positions. For ASCII content this distinction is invisible. For non-ASCII content (accented characters, Cyrillic, emoji), byte offsets and CodeMirror's character positions will diverge.

The bridge module must convert byte offsets to character positions when mapping diagnostics to editor locations. CodeMirror operates on character offsets; the Rust compiler operates on byte offsets. The conversion is: for a given line, count characters from the start until the byte count reaches `start_col`. This is a known seam — if it is not handled, clicking a diagnostic on a line containing non-ASCII text will scroll to the wrong column.

## Technology Choices

### Svelte 5 (Not React, Not Vanilla)

The playground is a Svelte 5 component, consistent with the existing islands in the project (`CompilerStatus.svelte`, `DocumentExplorer.svelte`, `ProjectTimeline.svelte`, etc.). The component hydrates independently of the rest of the page via Astro's island architecture.

### CodeMirror 6 for the Editor

CodeMirror 6 is the editor engine. It is modular (import only what you use), extensible (custom syntax modes, themes, keymaps), and has no framework dependency. It runs inside the Svelte component as a plain DOM integration.

**Why not Monaco (VS Code's editor):** Monaco is heavier (~2 MB), assumes a full IDE context, and is harder to embed in a lightweight island. The playground is a demonstration tool, not an IDE. CodeMirror's smaller footprint and modular architecture are a better fit.

**Why not a `<textarea>`:** No syntax highlighting, no line numbers, no bracket matching, no indentation support. The editor must feel good enough that a visitor types more than one line.

## Component Architecture

```
┌─────────────────────────────────────────────────┐
│  Astro Page (static)                            │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │  <UrdPlayground client:visible />         │  │
│  │                                           │  │
│  │  ┌─────────────┐   ┌──────────────────┐   │  │
│  │  │ CodeMirror  │   │ Output Pane      │   │  │
│  │  │ Editor      │   │                  │   │  │
│  │  │             │   │ JSON (success)   │   │  │
│  │  │ .urd.md     │   │ or               │   │  │
│  │  │ input       │──►│ Diagnostics (err)│   │  │
│  │  │             │   │                  │   │  │
│  │  └─────────────┘   └──────────────────┘   │  │
│  │         │                    ▲             │  │
│  │         │ source text        │ result      │  │
│  │         ▼                    │             │  │
│  │  ┌───────────────────────────────────┐    │  │
│  │  │ compiler-bridge.ts                │    │  │
│  │  │                                   │    │  │
│  │  │ parse_only() ← 50ms debounce     │    │  │
│  │  │ compile_source() ← 300ms debounce│    │  │
│  │  └───────────────────────────────────┘    │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

### State Model

The component manages five pieces of state:

| State | Type | Source |
|-------|------|--------|
| `source` | `string` | CodeMirror editor content. Updated on every edit. |
| `compileResult` | `CompileResult \| null` | Return value from `compile_source()`. Null before first compilation. |
| `parseValid` | `boolean` | Return value from `parse_only()`. Drives the instant syntax feedback indicator. |
| `compilerReady` | `boolean` | `false` until the WASM module has loaded and initialised. |
| `compileTimeMs` | `number` | Wall-clock time of the last `compile_source()` call. Displayed in the UI. |

No other state. No undo history (CodeMirror manages its own). No file management. No settings persistence. The playground is stateless between page loads, though a future enhancement could use URL hash encoding to share snippets.

### WASM Loading Strategy

The WASM module is **lazy-loaded** when the component becomes visible (triggered by Astro's `client:visible` directive). The loading sequence:

1. Component mounts. Shows the editor with a starter example pre-filled. Output pane shows a loading state.
2. `import()` fetches the WASM module asynchronously.
3. WASM `init()` runs (the one-time initialisation that `wasm-bindgen` requires).
4. `compilerReady` flips to `true`. The output pane compiles the starter example and displays the result.

**During loading:** The editor is fully interactive. The visitor can type, read the starter example, and explore the syntax. The output pane shows a non-intrusive loading indicator (e.g., "Initialising compiler…" with a subtle animation). The indicator should communicate progress, not block interaction.

**If loading fails:** The output pane shows a clear error message: "Compiler failed to load. Try refreshing the page." No silent failure. No blank pane.

> **Bundle splitting note:** The WASM module (`.wasm` file) and its JS glue code should be separate chunks from the Svelte component. The component JavaScript loads instantly (it's small). The WASM loads in parallel. This means the editor appears immediately and the compiler catches up.

### Compiler Bridge Module

The playground communicates with the WASM compiler through a single bridge module (`compiler-bridge.ts`). This module:

- Handles lazy `import()` of the WASM package and `init()` call.
- Exposes typed async wrapper functions for `compile_source()`, `parse_only()`, and `compiler_version()`.
- Parses the JSON string results into typed TypeScript objects.
- Catches any WASM-level errors and converts them to diagnostics (the compiler should never panic, but the bridge must be defensive).
- Lives in one file so that adding a Web Worker wrapper later is a one-file change, not a refactor.

```typescript
// compiler-bridge.ts — the typed interface the Svelte component uses

export interface CompileResult {
  success: boolean;
  world: string | null;       // The compiled .urd.json as a JSON *string*
  diagnostics: Diagnostic[];
}

export interface ParseResult {
  success: boolean;
  diagnostics: Diagnostic[];
}

export interface Diagnostic {
  severity: 'error' | 'warning' | 'info';
  code: string;
  message: string;
  span: {
    file: string;
    start_line: number;
    start_col: number;    // Byte offset — see Byte Offset Warning above
    end_line: number;
    end_col: number;
  };
}

export async function initCompiler(): Promise<void> { /* ... */ }
export function compileSource(source: string): CompileResult { /* ... */ }
export function parseOnly(source: string): ParseResult { /* ... */ }
export function compilerVersion(): string { /* ... */ }
```

**Double-JSON note.** The WASM binding returns a JSON string containing a `world` field that is itself a JSON string (the compiled `.urd.json`). The bridge module parses the outer JSON. The `world` value remains a string — the output pane is responsible for deciding how to display it. For v1, the output pane should `JSON.parse()` the `world` string and then `JSON.stringify(parsed, null, 2)` for pretty-printed display. This parse-and-reformat step also serves as a validation check: if `world` is not valid JSON, something has gone wrong in the compiler.

> **Note on `wasm-pack` build:** The done WASM brief notes that the full `wasm-pack build` was deferred because there was no consumer yet. This brief is that consumer. The implementing agent will need to run `wasm-pack build --target web --release --features wasm` in `packages/compiler/` to produce the `pkg/` directory with `.wasm` + JS/TS bindings. This should be added as a pnpm script (e.g., `compiler:wasm:build`) and included in the `build:full` pipeline.

## Editor Pane

### CodeMirror Configuration

- **Line numbers.** On.
- **Line wrapping.** On (prose lines in narrative content can be long).
- **Indentation.** Two spaces. Tab key inserts two spaces. Matches the Schema Markdown indentation rules.
- **Bracket matching.** On (for frontmatter).
- **Active line highlight.** On.
- **Theme.** Custom Gloaming dark theme. Background: `var(--deep)` (`#0a0a12`). Text: `var(--text)` (`#f2ece0`). Gutter: `var(--raise)` (`#14151e`). Selection: `var(--surface)` (`#1a1b25`). Cursor: `var(--gold)` (`#dab860`).
- **Placeholder.** If the editor is empty, show ghost-text: "Type Schema Markdown here…"

### Starter Example

The editor loads with a pre-filled example that compiles successfully. The tavern scene, trimmed to essentials:

```
---
world: tavern-demo

types:
  Barkeep [interactable]:
    name: string
    trust: int(0, 100) = 30

entities:
  @arina: Barkeep { name: "Arina" }
---

# The Rusty Anchor

A low-ceilinged tavern thick with pipe smoke.

[@arina]

== topics

@arina: What'll it be, stranger?

+ Ask about the harbour

  @arina: Quiet today. Too quiet.

  > @arina.trust + 5

  -> topics

* Ask about the missing ship

  ? @arina.trust > 50

  @arina: The Selene didn't sink. She was taken.

  -> topics

* Leave -> END
```

Small, readable, touches frontmatter, types, entities, headings, dialogue, conditions, effects, sticky and one-shot choices, and section jumps.

### Minimal Syntax Highlighting (v1)

A basic CodeMirror language mode for Schema Markdown. Line-oriented pattern matching — not a full grammar parse. Sufficient for visual differentiation.

| Token | Pattern | Highlight | Gloaming colour |
|-------|---------|-----------| --------------- |
| Frontmatter delimiters | `---` at line start | `meta` | `var(--faint)` |
| Frontmatter keys | `word:` within frontmatter | `propertyName` | `var(--amber)` |
| Headings | `#`, `##`, `###` at line start | `heading` | `var(--gold)` |
| Section labels | `== name` at line start | `heading` | `var(--gold)` |
| Entity references | `@word` anywhere | `variableName` | `var(--blue-light)` |
| Conditions | `?` at line start (after optional indent) | `keyword` | `var(--green)` |
| Effects | `>` at line start (after optional indent) | `keyword` | `var(--amber-light)` |
| Sticky choices | `+` at line start (after optional indent) | `operator` | `var(--purple)` |
| One-shot choices | `*` at line start (after optional indent) | `operator` | `var(--purple-light)` |
| Jumps | `->` anywhere | `keyword` | `var(--gold-dim)` |
| Dialogue attribution | `@word:` at line start (after optional indent) | `variableName` | `var(--blue-light)` |
| Comments | `//` to end of line | `comment` | `var(--faint)` |
| Strings | `"..."` in frontmatter | `string` | `var(--green-light)` |
| Numbers | Bare integers | `number` | `var(--amber-light)` |
| Keywords | `true`, `false`, `in`, `not in`, `import` | `keyword` | `var(--green)` |

The colour assignments use the Gloaming design system semantically: amber for compiler/schema concerns (frontmatter keys, effects, numbers), gold for structure (headings, sections, jumps), blue for entities (the writer's domain), green for conditions/validation, purple for choices (the player's domain).

> **Future: grammar-driven highlighting.** Once the PEG grammar is stable, highlighting can be derived from the grammar's token categories for perfect accuracy. Out of scope here.

## Output Pane

The output pane displays one of three states:

### 1. Compiled JSON (Success)

When `compileResult.success` is `true`, display the compiled JSON with:

- Syntax-highlighted JSON (CodeMirror in read-only mode, or a lightweight JSON highlighter).
- Collapsible sections for large blocks (types, entities, dialogue).
- A "Copy JSON" button.
- Compilation time displayed subtly (e.g., "Compiled in 3ms"). This is part of the demonstration — visitors should see how fast the compiler is.

### 2. Diagnostics (Error)

When `compileResult.success` is `false`, display diagnostics as a structured list:

- Each diagnostic shows: severity icon (error ◆ / warning ▸ / info →), diagnostic code (e.g., `URD102`), line number, and message.
- Clicking a diagnostic scrolls the editor to the relevant line and highlights it.
- Diagnostics sorted by line number (the compiler's `DiagnosticCollector.sorted()` already does this).
- **Editor decorations:** v1 uses CodeMirror's `Decoration.mark()` to underline diagnostic spans with a wavy line (red for errors, amber for warnings). Multiple diagnostics can be highlighted simultaneously. The decorations update on every `compile_source()` result. Hovering a decorated span shows the diagnostic message in a tooltip. This is standard CodeMirror `lintGutter` / `linter` extension territory — use the built-in lint infrastructure rather than custom decorations where possible.
- The error display should feel helpful, not punitive. The architecture document's style applies: not "parse error on line 47" but "`@guard.trust` is not a property on type Guard. Did you mean `@guard.mood`?"

### 3. Loading / Initialising

Before the WASM module is ready:

- Show a brief loading state: "Initialising compiler…"
- Once ready, automatically compile the starter example and transition to state 1 or 2.

### Output Pane Tabs (Future-Ready)

v1 shows a single output view (JSON or diagnostics). The output pane should be structured to support tabs in the future without a layout refactor. Anticipated future tabs:

- **JSON** (v1, default)
- **Diagnostics** (v1, shown on error — could become a persistent tab)
- **World Graph** (future — Mermaid or custom visualisation of sections, entities, and connections derived from compiled JSON)
- **AST** (future — raw AST explorer for advanced users)

The implementing agent does not need to build a tab system for v1. But the output pane's container should not assume it will only ever hold one view.

## Compilation Flow

### Debounce Strategy

Two compilation loops run concurrently at different frequencies:

```
Editor keystroke
     │
     ├──► parse_only() after 50ms debounce
     │       │
     │       └──► Updates parseValid indicator
     │
     └──► compile_source() after 300ms debounce
             │
             └──► Updates compileResult + compileTimeMs
                  (JSON output or full diagnostics)
```

`parse_only()` is Phase 1 only — fast. It tells the user immediately whether their syntax is valid. `compile_source()` runs all five phases and produces the full compiled output. The user gets instant syntax feedback while the full compilation catches up.

**Debounce values are tunable.** The 50ms and 300ms values are starting points. The implementing agent should expose them as named constants, not hardcoded values.

### Error Handling

When `compile_source()` returns with `success: false`, the output pane switches to the diagnostics view. The `parse_only()` result drives only the syntax validity indicator (a small green/red dot or similar), not the output pane content.

### Cancellation

If the user types while a `compile_source()` call is in progress, the in-progress call's result is discarded when the new call completes. WASM calls are synchronous on the main thread (they block briefly), so in practice the debounce prevents overlap. Always display the result of the most recent call, never an older one.

> **Web Worker threshold.** Move WASM calls to a Web Worker if any of the following are observed during testing: (1) `compile_source()` consistently exceeds 16ms (one frame budget at 60fps), (2) Lighthouse flags main-thread blocking on the playground page, or (3) measured input latency in the editor exceeds 100ms during compilation. For v1, WASM calls run on the main thread. The bridge module is designed for the swap — only `compiler-bridge.ts` changes.

## Responsive Layout

| Breakpoint | Layout | Behaviour |
|------------|--------|-----------|
| ≥ 1024px (desktop) | Side-by-side split pane. Editor left, output right. | Draggable divider. Default split: 50/50. |
| 768px – 1023px (tablet) | Side-by-side, narrower. | Default split: 55/45 favouring editor. Divider still draggable. |
| < 768px (mobile) | Stacked vertically. | Tab toggle to switch between editor and output. |

### Mobile Considerations

Mobile is a secondary context but must not be broken:

- Editor is usable (CodeMirror handles mobile input).
- Output is reachable via tab toggle.
- Loading state is visible.
- No horizontal scroll.

### Split Pane

On desktop, the divider between editor and output is draggable. A lightweight implementation (custom or small library). Not CSS `resize` (insufficient cross-browser). The divider position is not persisted (stateless component).

## Accessibility

- **Keyboard navigation.** Tab moves between editor and output pane. CodeMirror handles editor keyboard support.
- **Screen reader support.** ARIA labels: "Schema Markdown editor", "Compilation output". Diagnostics announced as a list. Compilation status changes via `aria-live` region.
- **Colour contrast.** All syntax highlighting colours must meet WCAG AA contrast ratios against the editor background (`#0a0a12`).
- **Reduced motion.** Loading animations respect `prefers-reduced-motion`.

## Build Integration

### New pnpm Scripts

| Script | Command | Purpose |
|--------|---------|---------|
| `compiler:wasm:build` | `cd packages/compiler && wasm-pack build --target web --release --features wasm` | Produce the WASM package (`pkg/` with `.wasm` + JS/TS bindings). |

The `build:full` script should be updated to include `compiler:wasm:build` before the site build, so the playground has access to the WASM module.

### WASM Package Consumption

The Astro site imports the WASM package from `packages/compiler/pkg/`. The recommended approach is a **pnpm workspace link**: configure the `pkg/` output as a workspace package (add a `package.json` to `pkg/` via `wasm-pack`'s `--out-name` / `--out-dir` options, or a post-build script) so the site can import it as `@urd/compiler-wasm`. This keeps dependency management explicit and consistent with the existing monorepo pattern.

If workspace linking proves awkward with Astro's Vite pipeline (WASM imports can be finicky), the fallback is a **vendored copy** step: a script copies the `pkg/` contents into `sites/urd.dev/src/lib/wasm/` before the site build. Either way, the key constraint is that the `.wasm` binary must not be inlined into the main JavaScript bundle — it must be a separate file fetched on demand by the browser.

## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| P1 | Component renders in an Astro page with `client:visible` hydration. Editor is interactive before WASM loads. | Page loads, editor accepts input, output shows "Initialising compiler…" |
| P2 | WASM module lazy-loads when the component scrolls into view. Starter example compiles automatically once loaded. | DevTools Network tab shows WASM fetch on scroll. JSON output appears. |
| P3 | Typing triggers `parse_only()` within 50ms debounce and `compile_source()` within 300ms debounce. | Output pane updates in real time as user types and pauses. |
| P4 | Successful compilation displays syntax-highlighted JSON with compilation time. | Starter example produces highlighted JSON output with time badge (e.g., "3ms"). |
| P5 | Failed compilation displays structured diagnostics with codes, line numbers, and messages. | Delete a required field. Diagnostics appear with URD codes and actionable messages. |
| P6 | Clicking a diagnostic scrolls the editor to the relevant line. | Click diagnostic → editor scrolls and highlights the line. |
| P7 | Responsive: side-by-side ≥ 1024px, stacked with tab toggle < 768px. | Resize browser. Layout adapts at breakpoints. |
| P8 | WASM load failure shows a clear error message, not a blank pane. | Simulate network failure (DevTools offline). Error message appears. |
| P9 | Syntax highlighting differentiates Schema Markdown tokens using Gloaming colours. | Visual inspection of starter example. Entities blue, conditions green, headings gold. |
| P10 | Compilation time is visible. | Time badge updates on each compile. |
| P11 | Component bundle size (excluding WASM): under 150 KB gzipped. | Build output analysis. |
| P12 | Accessibility: keyboard navigation, ARIA labels, WCAG AA contrast. | Keyboard-only test. Lighthouse audit. |
| P13 | `pnpm compiler:wasm:build` produces a working WASM package. | Script runs, `pkg/` contains `.wasm` + `.js` + `.d.ts`. |
| P14 | Gloaming design compliance. | Visual inspection against `design/themes/gloaming/design-brief.md`. No stock themes. |

## Relationship to Other Briefs

| Document | Relationship |
|----------|-------------|
| `briefs/done/2026-02-18-wasm-dual-target.md` | Produced the WASM bindings this component consumes. The three functions (`compile_source`, `parse_only`, `compiler_version`) and their JSON return shapes are defined there and verified in `src/wasm.rs`. |
| Architecture | Defines the compiler's diagnostic code ranges (URD100–599 by phase) and the five-phase pipeline. |
| Schema Markdown Specification | Defines the syntax the editor highlights and the starter example demonstrates. |
| `design/themes/gloaming/design-brief.md` | Single source of truth for all visual decisions. The CodeMirror theme, syntax colours, and layout must comply. |

## What This Does Not Cover

- **Syntax highlighting specification.** The v1 token map is pragmatic. A full spec aligned with the PEG grammar is separate.
- **Wyrd runtime integration.** A "compile and play" tab is future work. The output pane tab structure accommodates it.
- **World visualisation.** Rendering compiled JSON as a graph (section flow, entity relationships, containment tree) is a future output tab. Mermaid is one candidate — the compiled JSON already contains the structure needed.
- **Shareable URLs.** URL hash encoding for sharing snippets is future.
- **Persistence.** No state saved between page loads.
- **`wasm-opt` size optimisation.** Deferred from the WASM brief. Can be added when the playground ships to measure actual load times.

## Sequencing

The WASM compiler module exists and passes `pnpm compiler:wasm:check`. The full `wasm-pack build` (producing the `.wasm` + JS/TS bindings package) has not been run yet — it was deferred from the WASM brief because there was no consumer. This brief is that consumer.

Build order:

1. **`wasm-pack` build.** Run `wasm-pack build --target web --release --features wasm` in `packages/compiler/`. Verify the `pkg/` output loads in a browser. Add the `compiler:wasm:build` pnpm script.
2. **Compiler bridge module** (`compiler-bridge.ts`). Get WASM loading, `init()`, and `compile_source()` working. Test with a hardcoded source string in the browser console.
3. **Editor pane.** CodeMirror 6 in a Svelte component. Mount, accept input, fire change events.
4. **Output pane.** Render a `CompileResult` as syntax-highlighted JSON and as diagnostics.
5. **Wire them together.** Editor → bridge → output with the debounce loops. This is the "it works" moment.
6. **Syntax highlighting.** CodeMirror language mode. Pattern matching by line prefix.
7. **Responsive layout.** Split pane on desktop, stacked tabs on mobile.
8. **Design system polish.** Custom CodeMirror Gloaming theme. Final colour, typography, spacing pass.

Steps 1–5 are the critical path. Steps 6–8 can be parallelised or fast-followed.
