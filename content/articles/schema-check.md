---
title: "Schema Check: Closing the Loop"
slug: schema-check
description: The JSON Schema now validates compiler output at three levels — CI test suite, browser playground, and the emit phase itself. Defence in depth for a clinical system.
date: "2026-02-20"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the schema validation pipeline and the compiler fix it surfaced.
> Single canonical copy. February 2026.

## The gap

The Urd project had a JSON Schema since February 16th. It had a test corpus of 32 handwritten fixtures — 7 positive, 25 negative — all passing. But nobody had ever asked the obvious question: does the compiler's actual output pass the schema?

The answer was no. All seven compiler fixtures failed validation.

Three root causes:

1. **World names were display strings.** The schema requires `world.name` to match `^[a-z][a-z0-9-]*$` — a slugified identifier. The compiler emitted whatever the author wrote: `"Monty Hall"`, `"Key Puzzle"`, `"The Locked Garden"`. Every fixture failed on the first field.

2. **Actions omitted empty effects arrays.** The schema declares `effects` as required on actions. The compiler only emitted the field when effects were non-empty. Actions with no effects — like a Monty Hall "pick a door" action — were structurally invalid.

3. **Advance modes were missing from the pattern.** The schema's `advance` field accepted `on_action`, `on_rule`, `on_condition <expr>`, and `end`. The compiler also emits `auto` and `manual`. Two fixtures — monty-hall and sunken-citadel — used these modes.

None of these were spec violations. The compiler was doing reasonable things. The schema was doing reasonable things. They just disagreed on the details.

## Three layers of validation

### Layer 1: CI test suite

The schema validation script (`pnpm schema:test`) now has a third section. After the handwritten positive and negative fixtures, it compiles all seven compiler test fixtures through the `urd` binary and validates each output against the schema. The total test count went from 32 to 39.

This catches regressions at the source. If a future compiler change produces output that violates the schema, CI fails before the code is merged.

### Layer 2: Browser playground

The playground at [urd.dev/playground](/playground) now includes client-side schema validation. AJV — a JSON Schema draft 2020-12 validator — runs in the browser against the compiler's WASM output.

The implementation is lazy-loaded. AJV and the schema are only fetched the first time validation runs. The module is 128KB (39KB gzipped), downloaded once and cached. After first load, validation takes sub-millisecond — it does not affect the compile-and-render path.

The UI is a toggle in the output header: **Schema Check** with **Off** and **On** buttons, styled to match the document explorer's view-mode controls. A traffic light dot sits between the toggle and the Copy JSON button — gold when inactive, green when the schema passes, red when it fails. Off by default; the preference persists in `localStorage`.

This gives writers immediate, zero-setup feedback on whether their compiled output is structurally valid. They do not need to install tooling or run a CLI. They type, the compiler runs, and the light turns green or red.

### Layer 3: Compiler emit phase

The validation failures revealed a design inconsistency. The compiler slugifies location names, sequence names, phase names, and choice labels — every heading and identifier goes through the same `slugify()` function. But `world.name` was passed through verbatim from the frontmatter.

The fix is one function call in the emit phase. When the compiler encounters a `name` field in the world block, it runs the value through `slugify()` before emitting it. Writers can write `name: Monty Hall` and the output contains `"name": "monty-hall"`. This is consistent with every other identifier in the system and guarantees that every compiled world name conforms to the schema pattern.

This is compiler version 0.1.4.

## Defence in depth

For a system that will run in a hospital, each layer serves a different purpose:

- **CI tests** catch regressions before they ship. If a compiler change breaks schema conformance, the build fails.
- **Browser validation** gives writers immediate feedback without needing a local toolchain. A red light means something is wrong — before the JSON ever reaches a runtime.
- **Emit-phase slugification** eliminates an entire class of invalid output at the source. The compiler cannot produce a world name that violates the schema, regardless of what the author writes.

No single layer is sufficient. The CI tests do not run in the browser. The browser validator does not run at build time. The emit fix only addresses world names — there could be other structural mismatches in the future. Together, they form a pipeline where each layer catches what the others miss.

## The test count

The schema test corpus now has 39 fixtures: 7 handwritten positive, 25 handwritten negative, and 7 compiler output validations. The compiler remains at 480 tests with a 100% pass rate. The [changelog on GitHub](https://github.com/urdwyrd/urd/blob/main/packages/compiler/CHANGELOG.md) records every change.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
