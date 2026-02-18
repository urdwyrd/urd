# URD — WASM Dual-Target Build

*A brief for compiling the Urd compiler to both native and WebAssembly targets*

February 2026 | Backlog

`cargo build → native CLI` + `wasm-pack build → browser module`

> **Document status: BRIEF** — Defines scope, requirements, and acceptance criteria for producing a WebAssembly build of the Urd compiler alongside the native CLI binary. The WASM artifact powers the browser playground on urd.world and any future in-browser tooling.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:**
**Status:**

### What was done

*(To be filled on completion)*

### What changed from the brief

*(To be filled on completion)*

---

## Purpose

The Urd compiler is a pure transformation: `.urd.md` source in, `.urd.json` + diagnostics out. It has no filesystem dependencies, no network calls, and no threading requirements. This makes it an ideal candidate for WebAssembly compilation.

Shipping a WASM build of the compiler enables three things:

- **Browser playground.** A live editor on urd.world where visitors type Schema Markdown and see the compiled JSON output in real time. No install, no CLI, no barrier to entry.
- **Embedded validation.** The Urd IDE (future) can run the compiler in-browser for instant feedback without a language server round-trip.
- **Credibility through demonstration.** A working playground proves the compiler exists and that the schema compiles. This addresses the community feedback from intfiction.org about needing concrete proof, not just documentation.

### What This Is Not

This is not the runtime (Wyrd). Wyrd executes compiled `.urd.json` files and its implementation language has not been decided. This brief covers only the compiler's WASM target. A future brief may cover embedding both the compiler and Wyrd in a single browser experience, but that is out of scope here.

This is also not the Astro site build. The playground page that consumes this WASM module is a separate deliverable. This brief covers the compiler's build configuration and the JavaScript/TypeScript binding interface that the playground will call.

## Architectural Constraint

**Zero code duplication.** The compiler logic exists once, in a Rust library crate. Two thin consumers wrap that library:

- A native CLI binary (the `main.rs` entrypoint).
- A WASM module with JavaScript bindings (a `wasm.rs` entrypoint).

Both import and call the same library functions. There is no forked codepath, no `#[cfg(target_arch = "wasm32")]` in the core logic, and no separate "browser version" of the compiler. If a bug is fixed in the compiler, both targets get the fix automatically.

### Project Structure

```
urd-compiler/
  src/
    lib.rs            ← All compiler logic. Phases 1–5. The single source of truth.
    main.rs           ← CLI wrapper. Reads files from disk, calls lib, writes output.
    wasm.rs           ← WASM wrapper. Receives strings, calls lib, returns strings.
  Cargo.toml          ← Configured with [lib], [[bin]], and wasm-pack metadata.
  tests/              ← Integration tests run against the library directly.
```

### The Library Boundary

The compiler library exposes a single primary function (and possibly a few convenience variants):

```rust
// src/lib.rs — the contract both wrappers call

pub fn compile(source: &str) -> CompileResult {
    // Phases 1–5: Parse → Import → Link → Validate → Emit
    // Returns compiled JSON + diagnostics
}

pub struct CompileResult {
    pub success: bool,
    pub json: Option<String>,         // The compiled .urd.json as a string
    pub diagnostics: Vec<Diagnostic>, // Errors, warnings, info
}
```

> **Hard rule: no panics.** `compile()`, `parse()`, and every function they call must convert all internal errors into `Diagnostic` entries in the result. They must never panic. This is not just good practice — in the WASM context, a panic kills the module and the user sees a blank screen. All error paths, including malformed input, unexpected tokens, and internal invariant violations, must produce a diagnostic and return gracefully. This rule applies to the library, not just the WASM wrapper.

The CLI wrapper (`main.rs`) reads files from disk, resolves imports by loading files, and calls `compile()`. The WASM wrapper (`wasm.rs`) receives a source string (or a bundle of files as a JSON map), calls `compile()`, and returns the result as a serialized JSON string through `wasm-bindgen`.

### Import Resolution in WASM Context

The compiler's import resolution (Phase 2) reads files from the filesystem in CLI mode. In the browser, there is no filesystem. Two options:

**Option A: Single-file mode.** The WASM entry point accepts a single source string with no imports. The playground demonstrates single-file compilation only. Multi-file projects require the CLI. This is the simplest path for v1.

**Option B: Virtual filesystem.** The WASM entry point accepts a map of `{ filename: source_string }` pairs. The compiler's import resolver is abstracted behind a trait that the CLI implements with `std::fs` and the WASM wrapper implements with an in-memory map. This is more work but enables multi-file demos.

**Recommendation:** Start with Option A. The playground's purpose is demonstration and experimentation, not production compilation. Single-file mode covers the tavern scene, the Monty Hall problem, and any self-contained example. Add Option B when a concrete need arises (such as the IDE embedding the compiler).

> **Note for the implementing agent:** If Option A is chosen, the `compile()` function signature should still accept an import resolver trait or callback rather than hardcoding filesystem access. This keeps the door open for Option B without a refactor. The CLI passes a filesystem resolver; the WASM wrapper passes a resolver that always returns "file not found" (or a stub that returns a provided map). The cost of this abstraction is minimal and the retrofit cost of not doing it is significant.

## Build Configuration

### Cargo.toml

The `Cargo.toml` must declare both a library and a binary target:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
# cdylib: required for wasm-pack to produce a .wasm file
# rlib:   required for the native binary and tests to link against the library

[[bin]]
name = "urd"
path = "src/main.rs"
```

### Build Commands

Two build commands, both run on every CI commit:

| Command | Output | Purpose |
|---------|--------|---------|
| `cargo build --release` | `target/release/urd` (native binary) | CLI distribution. |
| `wasm-pack build --target web --release` | `pkg/` directory with `.wasm` + JS/TS bindings | Browser module. Published to npm as `@urd/compiler-wasm` or bundled directly into the Astro site. |

Both commands must succeed for a CI build to pass. If the compiler logic introduces a dependency that is not WASM-compatible, CI fails immediately. This is the enforcement mechanism for the zero-duplication constraint.

### WASM Build Profile

The native CLI and the WASM module have different optimization goals. The CLI should be fast. The WASM module should be small (download size is the bottleneck for first-load experience). Cargo supports this through profile overrides:

```toml
# Cargo.toml

# Native CLI: optimize for speed (default release behavior)
[profile.release]
opt-level = 3
lto = true
strip = true

# WASM: optimize for size
# Applied by wasm-pack via the --release flag using a custom profile
[profile.wasm-release]
inherits = "release"
opt-level = "z"           # Override: optimize for size, not speed
```

Alternatively, if the build tooling does not support custom profiles cleanly, the implementing agent can use `wasm-pack build --release` with the standard release profile (`opt-level = 3`) and rely on `wasm-opt -Oz` as the post-build step to handle size reduction. The compiler is small enough that either approach should meet the 2 MB target. The key constraint is: **the native binary must not be penalized by WASM size optimization.**

The CI pipeline runs `wasm-opt -Oz` after `wasm-pack build` to further reduce the binary. The final gzipped size is logged as a CI metric.

### CI Pipeline

```
  git push
     │
  ┌──┴──────────────────────────────┐
  │  1. cargo test                  │  ← Unit + integration tests
  │  2. cargo build --release       │  ← Native binary
  │  3. wasm-pack build --release   │  ← WASM module
  │  4. wasm-pack test --headless   │  ← WASM-specific tests (optional)
  └──┬──────────────────────────────┘
     │
  Both artifacts produced. Build passes.
```

### Dependency Rules

Any Rust crate used in `src/lib.rs` must compile to `wasm32-unknown-unknown`. In practice this means:

- `pest` (PEG parser): WASM-compatible. No issues.
- `serde` / `serde_json`: WASM-compatible. No issues.
- Standard library string and collection types: WASM-compatible.
- **Forbidden in lib.rs:** `std::fs`, `std::net`, `std::process`, `std::thread`, or any crate that depends on them. These are CLI-only concerns and belong in `main.rs`.

If a future dependency is needed that is not WASM-compatible, it must be isolated behind a trait boundary so that the WASM wrapper can provide an alternative implementation.

### Deterministic Output

The byte-identical requirement (W4) demands that JSON key ordering is deterministic across builds and targets. Rust's `HashMap` uses randomized iteration order by default, which means the same compilation could produce different JSON on different runs.

**Rule:** All maps in the compiler's EMIT phase (Phase 5) must use `BTreeMap`, not `HashMap`. This applies to any intermediate data structure that influences the order of keys in the output JSON. The serializer configuration must also produce stable output (no randomized field ordering in `serde`). This is a compile-time architectural choice, not a runtime toggle.

## WASM Binding Interface

The WASM module exposes a JavaScript API through `wasm-bindgen` designed for real-time interactive use. The primary consumer is an Astro island component where compilation runs on every keystroke (debounced). The binding surface must support fast, repeated calls and return structured results that a UI can render instantly — including partial results on error.

### Design Context (Not In Scope, But Shapes the API)

The intended downstream experience is a split-pane editor on urd.world: Schema Markdown input on the left, compiled output or diagnostics on the right, updating in real time as the user types. This means:

- **Speed is visible.** The compilation time is part of the demonstration. The WASM module should return fast enough that the user perceives the output as instant. The binding should not add overhead (no unnecessary serialization round-trips, no allocations that could be avoided).
- **Errors are the happy path.** While the user is typing, the source is invalid most of the time. The module must return structured diagnostics quickly and gracefully — not panic, not return an empty result, not require the caller to catch exceptions.
- **The caller decides what to display.** The binding returns data. The Astro component decides how to render it (syntax-highlighted JSON, inline error markers, a diagnostics panel). The binding must not format output for a specific UI.

### Core Functions

```rust
// src/wasm.rs

use wasm_bindgen::prelude::*;
use crate::compile;

/// Compile a single Schema Markdown source string.
/// Returns a JSON-serialized CompileResult.
/// 
/// This is the primary function the playground calls on every edit.
/// It must never panic. Invalid input produces diagnostics, not crashes.
#[wasm_bindgen]
pub fn compile_source(source: &str) -> String {
    let result = compile(source);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(r#"{{"success":false,"diagnostics":[{{"severity":"error","message":"{}"}}]}}"#, e)
    })
}

/// Return the compiler version string.
/// Useful for displaying in the playground UI and for cache-busting.
#[wasm_bindgen]
pub fn compiler_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Validate syntax only (Phase 1: Parse).
/// Faster than full compilation. Returns parse diagnostics.
/// 
/// Useful for providing instant red/green feedback while the user types,
/// with full compilation running on a longer debounce interval.
#[wasm_bindgen]
pub fn parse_only(source: &str) -> String {
    let result = parse(source);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(r#"{{"success":false,"diagnostics":[{{"severity":"error","message":"{}"}}]}}"#, e)
    })
}
```

### Why Three Functions, Not One

A real-time editor has two feedback loops running at different speeds:

| Loop | Trigger | Latency target | Function |
|------|---------|---------------|----------|
| Syntax feedback | Every keystroke (or short debounce, ~50ms) | Under 10ms | `parse_only()` |
| Full compilation | Pause in typing (longer debounce, ~300ms) | Under 200ms | `compile_source()` |

`parse_only()` runs Phase 1 only and tells the UI whether the source is syntactically valid. This is cheap — the PEG parser is fast — and gives instant red/green feedback. `compile_source()` runs all five phases and produces the compiled JSON. The Astro component can call both at different debounce intervals, showing parse errors immediately and compiled output slightly delayed.

> **Scope note:** v1 performs full recompilation on every call. There is no incremental parsing or caching between invocations. For the playground's intended use (small to medium single-file examples), full recompilation is fast enough. Incremental parsing is a future optimization if profiling shows it is needed — do not prematurely optimize here.

`compiler_version()` exists so the playground can display the compiler version and so the Astro build can cache-bust the WASM module when the compiler is updated.

### Return Format

Both `compile_source()` and `parse_only()` return a JSON string. The JavaScript consumer parses it. The return type is deliberately a string (not a complex `JsValue` struct) to keep the binding surface minimal and serialization explicit.

### TypeScript Declaration

The `wasm-pack` build produces a `.d.ts` file. The effective interface from the consuming Astro component's perspective:

```typescript
// Generated by wasm-pack, consumed by the playground component

export function compile_source(source: string): string;
export function parse_only(source: string): string;
export function compiler_version(): string;

// The parsed result shape (same for both compile_source and parse_only):
interface CompileResult {
  success: boolean;
  json?: string;              // Present only on success from compile_source.
                               // Absent from parse_only results.
  diagnostics: Diagnostic[];  // Always present. Empty array on clean compile.
}

interface Diagnostic {
  severity: "error" | "warning" | "info";
  file?: string;
  line?: number;
  column?: number;
  message: string;
  code?: string;              // Machine-readable, e.g. "URD001"
  suggestion?: string | null; // "Did you mean @guard.mood?"
}
```

### Future Binding Expansion

The following functions are not needed now but are anticipated. The implementing agent should not build them, but should not make architectural choices that would prevent adding them later:

- `compile_bundle(files_json: &str) -> String` — accepts a JSON map of `{ "filename.urd.md": "source..." }` pairs for multi-file compilation in the browser (Option B from the Import Resolution section).
- `get_ast(source: &str) -> String` — returns the parsed AST as JSON. Useful for a future "AST explorer" panel in the playground.
- `get_completions(source: &str, line: u32, column: u32) -> String` — returns autocomplete suggestions at a cursor position. Useful if the playground grows into a lightweight editor with intellisense.

## What This Enables Downstream

This brief produces the WASM artifact. Downstream deliverables that consume it (each requiring their own brief):

- **urd.world playground page.** An Astro island with a split-pane editor. Left: CodeMirror 6 with Schema Markdown input. Right: compiled JSON output (or diagnostics on error). The WASM module runs on every keystroke (debounced) with `parse_only()` for instant syntax feedback and `compile_source()` for full compilation on pause. No server round-trip. The real-time speed is itself a demonstration of the compiler's quality.
- **Embeddable playground widget.** The same component, exported as a web component or iframe embed, for use in blog posts, documentation, and the intfiction.org forum.
- **Compile-and-play experience.** A future page that chains the compiler WASM module with whatever runtime Wyrd ships as. The compiler produces `.urd.json` in-browser; the runtime executes it. The implementation depends on Wyrd's language choice (if Wyrd is Rust, it could be a second WASM module; if TypeScript, it runs natively in the browser; if something else, the JSON interchange format is the bridge). The compiler's binding surface does not need to change to support this — it outputs a JSON string, and any runtime that accepts JSON can consume it.
- **IDE compiler integration.** The Urd IDE can use the WASM module for instant in-browser compilation alongside the LSP for richer features.

## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| W1 | `cargo build --release` produces a working native CLI binary. | Binary compiles a test `.urd.md` file and produces valid `.urd.json`. |
| W2 | `wasm-pack build --target web --release` produces a working WASM module. | Module loads in a browser, accepts a source string, returns a `CompileResult` JSON string. |
| W3 | Both builds use identical compiler logic from `src/lib.rs`. | No `#[cfg(target_arch)]` in `lib.rs`. No duplicated logic between `main.rs` and `wasm.rs`. |
| W4 | Given identical input, both targets produce identical `CompileResult.json` output. | A test script compiles the same source via CLI and WASM and diffs the JSON output. Byte-identical. |
| W5 | CI runs both builds on every commit. A failure in either fails the pipeline. | CI configuration includes both `cargo build` and `wasm-pack build` steps. |
| W6 | No WASM-incompatible dependencies in `lib.rs`. | `cargo check --target wasm32-unknown-unknown` passes with no errors on `lib.rs` alone. |
| W7 | The WASM module size is under 2 MB (gzipped) for initial release. | `wasm-opt -Oz` applied to stripped build with `opt-level = "z"`. Size logged in CI. |
| W8 | Compiler diagnostics are returned in structured JSON, not printed to stdout/stderr. | The `compile()` function returns diagnostics in the `CompileResult` struct. The CLI formats them for the terminal. The WASM wrapper serializes them as JSON. |
| W9 | No panics reachable from public WASM bindings. `compile()`, `parse()`, and all functions they call must convert internal errors to diagnostics, never panic. | Verified by fuzzing malformed input (random bytes, truncated files, deeply nested structures, empty strings, multi-MB input). A playground that crashes the WASM module once loses trust immediately. |
| W10 | JSON output uses deterministic key ordering. | All maps in the EMIT phase use `BTreeMap` (not `HashMap`). Verified by W4 — if key ordering is non-deterministic, the byte-identical check will catch it. |

## Relationship to Existing Briefs

| Document | Relationship |
|----------|-------------|
| Architecture | Lists Rust and WASM as a strength in the technology considerations. This brief formalizes the dual-target build as a requirement rather than an option. |
| Formal Grammar Brief | Defines Phase 1 (Parse) using `pest`. `pest` is WASM-compatible, so the grammar implementation works in both targets without modification. |
| Wyrd Reference Runtime | Wyrd's implementation language is undecided. It consumes compiled `.urd.json`. Regardless of Wyrd's language choice, the WASM compiler and Wyrd can coexist on the same page: the compiler WASM module produces JSON, and Wyrd executes it. The JSON interchange format is the bridge — no direct coupling between the two. A future brief could combine them into a "compile and play" browser experience. |
| Test Case Strategy | Tests run against compiled `.urd.json`. The WASM module should be able to compile all four test case source files and produce output identical to the CLI. This is a natural extension of the test suite. |

## Sequencing

This brief should be implemented after the compiler's Phase 1 (Parse) is stable and producing ASTs from the grammar. It can be implemented as early as Phase 1 — even a parser-only WASM build that returns parse success/failure and diagnostics has value for the playground. The full compile pipeline (Phases 1–5) extends the same WASM module as each phase lands.

The recommended approach is to set up the dual-target build structure (Cargo.toml, `wasm.rs`, CI) during Phase 1, even if the compiler only parses at that point. This ensures every subsequent phase is tested against both targets from the start, rather than discovering WASM-incompatible dependencies late.
