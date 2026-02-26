---
title: "Reference Manual v0.1.14"
slug: reference-manual-v0114
description: The reference manual catches up. Seven compiler releases, five novel diagnostics, a structural diff engine, and an IDE-ready definition index — all documented in a single consolidated reference.
date: "2026-02-26"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the update of the reference manual from v0.1.7 to v0.1.14.
> Single canonical copy. February 2026.

## What changed in the compiler

The [v0.1.7 reference manual](/documents/urd-v0.1.7-reference) was written the day the [compiler gate closed](/articles/pre-alpha). Seven releases later, the compiler has grown from 554 tests to 634, from five phases to six, from 69 diagnostic codes to 74. The [semantic gate](/articles/semantic-gate-closed) closed along the way. The manual needed to catch up.

Here is what was added between v0.1.8 and v0.1.14:

**ANALYZE phase (v0.1.8).** A sixth compiler phase, running between LINK and VALIDATE, that derives diagnostics entirely from the FactSet. Five novel checks: property read but never written (URD601), property written but never read (URD602), enum variant produced but never tested (URD603), numeric threshold unreachable by any effect (URD604), and circular property dependency (URD605). These are questions that AST-walking checks cannot answer.

**PropertyDependencyIndex enhancements (v0.1.9).** Set-difference queries (`read_but_never_written()`, `written_but_never_read()`), deterministic JSON serialisation, and integration into the WASM pipeline as the `property_index` field. The playground's Properties tab consumes this directly.

**Dialogue graph correlation (v0.1.10).** `ChoiceFact.jump_indices` links choices to their jump edges, enabling the playground's dialogue graph visualisation to draw edges from specific choices to their target sections.

**Semantic diff engine (v0.1.12).** Structural comparison of two compiled worlds across six categories — entities, locations/exits, dialogue, property dependencies, rules, and reachability. CLI subcommands: `urd diff` for comparison, `urd snapshot` for creating `.urd.snapshot.json` baselines. Built for CI regression testing.

**DefinitionIndex (v0.1.13).** A seven-namespace map from identifiers to declaration spans: types, properties, entities, sections, choices, locations, exits, and rules. Built from the SymbolTable after LINK. Provides go-to-definition and hover data for IDE consumers. Serialised in WASM output for the playground editor.

**CompilationResult surface (v0.1.14).** The SymbolTable and DependencyGraph are now exposed on `CompilationResult`, giving downstream consumers full access to the compiler's semantic model. The CLI gained `--help` and `--version` flags with comprehensive usage documentation.

## What was updated in the manual

The [v0.1.14 reference manual](/documents/urd-v0.1.14-reference) is not a rewrite — it is the v0.1.7 manual with targeted updates to reflect the current compiler. The schema language and JSON format are unchanged. What changed is the compiler's tooling surface.

**New sections:**
- Section 7.7: Diff and Snapshots — CLI subcommands, snapshot format, change categories
- Section 7.8: DefinitionIndex — seven namespaces, IDE/LSP integration
- Section 7.9: CLI Reference — full `--help` output
- Section 8, ANALYZE Phase: URD601–URD605 diagnostic table
- Section 9.5: ANALYZE Diagnostics — how the five checks work
- Section 12: Changelog Summary (v0.1.8–v0.1.14)

**Updated sections:**
- Section 1: Version note updated (634 tests, semantic gate closure)
- Section 3: CLI examples expanded with `diff`, `snapshot`, `--help`, `--version`; WASM response shape includes `property_index` and `definition_index`
- Section 7.1: Pipeline diagram shows six phases (PARSE → IMPORT → LINK → ANALYZE → VALIDATE → EMIT)
- Section 7.2: ANALYZE row added to the phase table
- Section 8: Code ranges table includes ANALYZE (URD600–699); total diagnostics updated to 74
- Section 9.1: `ChoiceFact.jump_indices` documented
- Section 9.3: `read_but_never_written()`, `written_but_never_read()`, `to_json()` documented
- Section 9.4: `property_index` and `definition_index` in WASM output
- Section 10.4: Stable diagnostic range extended to URD605; FactSet stability noted

**Unchanged:** All of Section 4 (Schema Markdown Syntax), Section 5 (Worked Examples), Section 6 (Compiled JSON Format), and Section 11 (Quick Reference). Writers will not notice a difference.

## What to read first

**Writers:** Nothing has changed for you. The schema language is identical. The quick reference, worked examples, and syntax sections are unchanged. If you want to see what the compiler now catches, skim the [ANALYZE diagnostics](/documents/urd-v0.1.14-reference#analyze-phase-urd600urd699) — they may flag issues in your existing worlds.

**Developers:** The compiled JSON format is unchanged. The new CLI subcommands (`urd diff`, `urd snapshot`) are useful for CI — see [Diff and Snapshots](/documents/urd-v0.1.14-reference#77-diff-and-snapshots).

**Tool builders:** Start with [DefinitionIndex](/documents/urd-v0.1.14-reference#78-definitionindex) if you are building editor integration. The seven namespaces give you go-to-definition and hover for every construct in the language. The WASM response now includes `property_index` and `definition_index` alongside the existing `facts` — three complementary views of the same compiled world.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
