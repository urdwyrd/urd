---
title: "Compiler 0.1.7: The Gate Closes"
slug: pre-alpha
description: All 32 acceptance criteria verified and documented. The v1 completion gate is closed. The Urd compiler enters pre-alpha with 554 tests, eight static analysis checks, a queryable analysis IR, and a cross-document specification audit that found — and fixed — a latent schema divergence.
date: "2026-02-22"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the closure of the v1 completion gate and the compiler's entry into pre-alpha.
> Single canonical copy. February 2026.

## What this means

The [v1 completion gate](/documents/v1-completion-gate) defined 32 acceptance criteria across four categories. Every one of them is now verified and documented. The compiler is not a prototype. It is a v1-complete implementation of the Urd schema specification.

That does not mean the project is finished. It means the foundation is proved. The schema language works. The compiler produces valid, typed, structured output. The static analysis catches real problems. The specification is internally consistent. What remains — the Wyrd reference runtime, the alpha milestone, the Monte Carlo validation — builds on top of this, not beside it.

The compiler version is 0.1.7. Pre-alpha begins now.

## What the compiler does

The pipeline has five phases. Each one transforms the input and passes a richer representation to the next.

**PARSE** reads `.urd.md` files — constrained YAML frontmatter and Schema Markdown narrative. It handles BOM stripping, tab rejection, size limits, and the full narrative syntax: locations, entities, exits, sections, choices, conditions, effects, jumps, dialogue, and presence markers. 124 tests.

**IMPORT** resolves `import:` declarations. Imports are explicit and non-transitive. Circular imports of any cycle length are detected and reported with the full cycle path. File stem collisions, case-sensitivity warnings, depth limits (64 levels), and compilation unit size limits (256 files) are all enforced. 55 tests.

**LINK** builds the symbol table — types, properties, entities, locations, sections — and resolves every cross-reference. Entity-to-type resolution, `@entity.property` access validation, jump target resolution, enum variant checking, duplicate detection across the full compilation unit. When a reference fails, the compiler suggests the closest match by edit distance. 81 tests.

**VALIDATE** runs type checking and structural analysis. Property values matched against declared types, ranges, and enum sets. Nesting depth enforcement (warn at 3, error at 4+). Reachability analysis from `world.start`. Orphaned choice detection. Missing fallthrough detection. Shadowed exit detection. Player entity trait validation. Sequence phase validation. 108 tests.

**EMIT** produces `.urd.json` conforming to the published JSON Schema. Automatic `urd: "1"` injection. Field slugification. Deterministic ordering. 68 tests.

## What it catches

Eight static analysis checks span three phases:

| Check | What it detects | Phase |
|-------|----------------|-------|
| S1 — Undefined reference | `@entity`, `-> jump`, or property access that does not resolve | LINK |
| S2 — Type mismatch | Property set to a value that violates its declared type | VALIDATE |
| S3 — Unreachable location | Location with no path from `world.start` via exits | VALIDATE |
| S4 — Orphaned choice | Choice condition tests an enum value that does not exist | VALIDATE |
| S5 — Duplicate ID | Entity, type, location, section, or choice ID collision | LINK |
| S6 — Missing fallthrough | One-shot-only section with no terminal jump | VALIDATE |
| S7 — Circular import | Import cycle of any length, with full path reported | IMPORT |
| S8 — Shadowed exit | Section name matches an exit name in the same location | VALIDATE |

S1, S2, S5, and S7 are errors — they block compilation. S3, S4, S6, and S8 are warnings — the compiler produces valid output but tells you something looks off. Every check has a dedicated negative fixture that triggers it.

## What it knows

The FactSet is a flat, queryable intermediate representation extracted after LINK. It projects the resolved world into six fact types — `ExitEdge`, `JumpEdge`, `ChoiceFact`, `RuleFact`, `PropertyRead`, `PropertyWrite` — plus a `PropertyDependencyIndex` that maps every `(type, property)` pair to its read and write sites.

The FactSet is immutable, deterministic, and complete. Same input, same facts, same order. No AST traversal required for downstream consumers. The WASM build serialises facts alongside compiled JSON, powering the playground's Analysis panel. 31 dedicated tests verify extraction across all five canonical fixtures.

## The specification audit

The final gate category — E1 through E7 — required a cross-document consistency audit against nine normative documents. The audit procedure defined explicit authority rankings, tie-breaker rules, and a formal record format.

Four findings were resolved:

**E1 (MINOR):** The Architecture document used a four-component model without explicitly naming the Adapter layer. Fixed by adding an architectural layers paragraph referencing the Architectural Boundaries document's three-layer definitions.

**E2 (MINOR):** The Wyrd Reference Runtime omitted the failure contract specified in Architectural Boundaries — structured failure result, two categories, no mutation, no events. Fixed by adding a failure contract paragraph to the Public API section.

**on_exhausted goto (CRITICAL):** The compiler's `ExhaustedData` struct has a `goto: Option<String>` field and emits it to JSON for hub-and-spoke dialogue patterns. But the JSON Schema's `speech` type, referenced by `on_exhausted`, has `additionalProperties: false` — it would reject the field. No canonical fixture triggered this code path, so the gate test passed despite the latent divergence. Fixed by adding a dedicated `exhaustedContent` schema definition with optional `goto`, and documenting the field in the Schema Specification.

**Advance modes (MINOR):** The Schema Specification listed four advance modes (`on_action`, `on_rule`, `on_condition`, `end`) but the JSON Schema regex and compiler both support six — adding `auto` and `manual`. Fixed by updating the Schema Specification's advance modes table.

Three items passed without changes (E3, E4, E5). E6 confirmed consistent lambda framing across all documents. E7 was marked informative — the FactSet brief is canonical for graph structure, so a separate section in the Schema Specification was not needed.

## By the numbers

| Metric | Value |
|--------|-------|
| Compiler version | 0.1.7 |
| Total tests | 554 |
| Pass rate | 100% |
| Canonical fixtures | 5 (plus 10 negative, 2 specialty) |
| Static analysis checks | 8 (S1–S8) |
| FactSet fact types | 6 |
| Gate acceptance criteria | 32 (C1–C9, S1–S8, F1–F8, E1–E7) |
| Diagnostic code ranges | URD100–URD499 across 4 phases |
| Schema validation tests | 39 |
| Specification documents audited | 9 |
| Documents amended during audit | 4 |
| Build targets | Native CLI + WASM |

## What comes next

The next milestone is alpha. The Wyrd reference runtime loads `.urd.json`, executes the canonical test cases, and proves the system works end-to-end. The acid test: compile the Monty Hall problem, run it ten thousand times, verify the switching advantage converges to two-thirds. Not because the probability is coded — because it emerges from the structure.

The compiler is the foundation. The gate is closed. Now we build what runs on top of it.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
