---
title: Try the Compiler
slug: playground
description: The Urd compiler now runs in the browser. Type Schema Markdown, get compiled JSON in real time, with full diagnostics — no install, no server, no data leaving the page.
date: "2026-02-19"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Announcing the interactive playground and explaining how it works.
> Single canonical copy. February 2026.

## The compiler is in your browser

The [playground](/playground) loads the Urd compiler as a WebAssembly module and runs it entirely client-side. Type Schema Markdown on the left, see compiled `.urd.json` on the right. Nothing is sent to a server. Nothing is stored. The compilation happens in a single synchronous call, typically under a millisecond after the module has initialised.

This is the same compiler that powers the CLI and passes 413 tests. It is not a demo or a subset — it is the full five-phase pipeline: parse, resolve, validate, lower, emit. If it compiles on the playground, it will compile identically on the command line.

## What you can do

The playground ships with a starter example called The Locked Garden — a two-location world with three types, five entities, conditional exits, nested dialogue, OR conditions, entity-targeted choices, section jumps, and a destroy effect. It exercises most of the Schema Markdown grammar in roughly 100 lines.

Edit the source and the output updates in real time. The editor uses a dual debounce: a fast 50ms parse check that lights a green or red indicator, and a slower 300ms full compile that produces either JSON output or a structured diagnostics list.

When compilation fails, diagnostics appear with severity icons, error codes, line numbers, and messages. Click any diagnostic to jump to the relevant line in the editor. The same diagnostic codes documented in the [compiler briefs](/documents/urd-compiler-architecture-brief) appear here — URD100 through URD599, each owned by the phase that emits it.

## How it is built

**The compiler** is Rust, compiled to `wasm32-unknown-unknown` with `wasm-bindgen`. The release build uses LTO and optimisation level 3. The resulting WASM binary is roughly 67 KB gzipped — small enough that the module initialises in under a second on most connections.

**The bridge** is a TypeScript module that lazy-loads the WASM binary on first use, calls `init()` with a `fetch()` of the static `.wasm` file, and exposes typed wrappers for `compile_source()` and `parse_only()`. It also handles byte-offset to character-offset conversion — the Rust compiler reports spans in bytes, but CodeMirror works in characters.

**The editor** is CodeMirror 6 with a custom `StreamLanguage` mode for Schema Markdown. The mode tokenises frontmatter delimiters, YAML keys, headings, section markers, entity references, conditions, effects, choices, jumps, dialogue, strings, and comments. The syntax theme maps tokens to the Gloaming palette — the same colour system used across the rest of the site.

**The output pane** renders compiled JSON with syntax highlighting via a regex tokeniser that classifies keys, strings, numbers, booleans, null values, brackets, and punctuation. When compilation fails, it switches to a diagnostics list view.

**The layout** is a resizable split pane on desktop (drag the divider between 25% and 75%) and a tabbed interface on mobile. The split pane is roughly 60 lines of pointer event handling — no library dependency.

## Why this matters

Most language projects ask you to install a toolchain before you can evaluate whether the language is worth learning. That is a high barrier, especially for a schema system whose target audience includes writers and designers, not only engineers.

The playground removes that barrier. Visit the page, read the starter example, change something, see what happens. The compile time display shows exactly how fast the compiler is. The diagnostics show exactly how helpful the error messages are. The JSON output shows exactly what the compiler produces.

For a project making [clinical claims](/articles/compiler-test-dashboard) about reliability and correctness, letting people verify those claims in their browser — without trusting a CI badge or a README — is more convincing than any documentation.

## What comes next

The playground currently shows compiled JSON. That is the compiler's output format, and it is useful for engineers evaluating the schema. But it is not what writers care about. Writers want to see the world *running* — locations rendering, choices appearing, state changing.

That requires Wyrd, the reference runtime. When Wyrd is ready, the playground will gain a third pane: a live preview that executes the compiled world. Type a world, compile it, play it — all in the browser.

The compiler is ready. The playground proves it. Now we build the runtime.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
