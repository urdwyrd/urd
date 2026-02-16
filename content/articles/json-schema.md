---
title: The Output Contract
slug: json-schema
description: A JSON Schema that validates what the compiler produces — the output-side counterpart to the PEG grammar.
date: "2026-02-16"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> How and why the project built a machine-readable definition of valid compiler output.
> Single canonical copy. February 2026.

## Two artifacts, one specification

The Urd formalisation phase produces two machine-checkable artifacts that work together:

| Artifact | Validates | When used |
|----------|-----------|-----------|
| PEG grammar (completed) | Compiler input (`.urd.md`) | Parse phase. Before compilation. |
| **JSON Schema (this article)** | Compiler output (`.urd.json`) | After compilation. CI checks. Runtime load. |

The grammar answers: *is this source file well-formed?* The JSON Schema answers: *is this compiled output structurally conformant?* Together they define what valid Urd looks like at both ends of the pipeline. The prose specifications define what Urd means. The grammar and schema define what it looks like — for input and output respectively.

```
  .urd.md ──── PEG grammar: is this well-formed? ────┐
               (syntactic check)                       │
                                                       ▼
           ┌─── COMPILER ─────────────────────────────────┐
           │    parse → resolve → link → validate → emit   │
           └───────────────────────────────────────────────┘
                            │
                            ▼
  .urd.json ── JSON Schema: is this conformant? ──────┐
               (structural check)                      │
                                                       ▼
                                                  ✓ VALID
```

This article is about the second artifact: the JSON Schema.

## What it validates

The schema is a single self-contained file written in [JSON Schema draft 2020-12](https://json-schema.org/draft/2020-12/schema). It validates the complete structure of a compiled `.urd.json` world file. "Structurally valid" means: required fields are present, types are correct, enum values are within declared sets, and object shapes match the specification.

It covers all eight top-level blocks:

- **`world`** — metadata: name, version, starting location, entry sequence, random seed
- **`types`** — type definitions with traits, property schemas, and conditional visibility
- **`entities`** — entity instances with type references and property values
- **`locations`** — rooms with descriptions, containment, and exits (conditional or unconditional)
- **`rules`** — reactive triggers with conditions, select blocks, and effects
- **`actions`** — player-initiated operations with targeting and conditions
- **`sequences`** — phased progressions with advance modes
- **`dialogue`** — flat section maps with recursive choices, speech, and exhaustion content

These are wired through nine reusable sub-schemas in `$defs`: `effect` (five forms), `conditionExpr` (AND/OR), `speech`, `visibility` (simple or conditional), `propertySchema` (with conditional type validation), `exit`, `select`, `phase`, and `choice` (recursive, self-referencing).

Only `world` is required. A valid `.urd.json` can be as small as:

```json
{
  "world": {
    "name": "empty",
    "urd": "1"
  }
}
```

## What it deliberately does not validate

The schema draws a sharp line between structural and semantic validity. It can verify that an entity has a `type` field containing a string. It cannot verify that the string references a type that actually exists in the `types` block. That level of cross-reference checking is the compiler's job (phases 3–4) and the runtime's load-time responsibility.

Specifically out of scope:

- **Cross-reference integrity.** Does `start` point to a real location? Does `target_type` reference a real type?
- **Property value type matching.** Is `"prize": "goat"` actually one of the enum values defined on the Door type?
- **Condition expression syntax.** The schema validates that conditions are non-empty strings. It does not parse the expression language.
- **Effect target validity.** `"set": "guard.mood"` references a real entity and property — the schema cannot check this.
- **Dialogue ID consistency.** The `id` field inside a section should match its map key — that is a compiler guarantee, not a structural constraint.

These semantic checks remain the compiler's responsibility. The JSON Schema is a safety net for structure, not a replacement for the compiler's validation phases.

## Design decisions

### Single file, all `$defs` inline

The schema is one file with no external `$ref` resolution. Anyone can download it and validate against it without dependency resolution. At 548 lines it is comfortably manageable as a single document.

### `additionalProperties: false` everywhere

Every object with a known, closed field set rejects unknown keys. This catches typos (`refType` instead of `ref_type`), prevents drift between the schema and the specification, and ensures that if a compiled `.urd.json` contains a field not defined in the spec, it fails validation.

The two exceptions are intentional: entity property values (whose types depend on the referenced type definition) and map keys (entity IDs, location IDs, and other identifiers generated by the compiler).

### Conditional type validation

The `propertySchema` definition is the most complex piece — it uses `allOf` with `if/then` blocks to enforce type-specific constraints. When a property's `type` is `enum`, `values` is required. When `type` is `boolean`, `default` must be a boolean and `min`/`max`/`values`/`ref_type` are forbidden. Each branch is precise about what is allowed and what is not.

Every `if` block includes `"required": ["type"]` alongside the `const` check. Without this, the condition would match vacuously when `type` is absent, causing the `then` clause to fire incorrectly. This is a subtle but important pattern in JSON Schema conditional validation.

### Mutual exclusion via `not`

Actions may have `target` or `target_type`, but not both. Phases may have `action` or `actions`, but not both. These are expressed as `"not": { "required": ["target", "target_type"] }` — if both keys are present, the schema rejects the document.

### Recursive choices

Dialogue choices can contain nested sub-choices with the same structure, handling inline branching from the source syntax. The `choice` definition references itself via `$ref`, which JSON Schema draft 2020-12 handles natively.

## The test corpus

The schema ships with 32 test fixtures: 7 positive and 25 negative.

### Positive fixtures

Each positive fixture is a complete, valid `.urd.json` that exercises specific schema features:

1. **Monty Hall** — the full game: sequences with all four advance modes, select blocks, enum properties, hidden visibility, empty effects on no-op actions
2. **Key Puzzle** — freeform world (no sequences): conditional exits, blocked messages, ref properties, destroy effects, multiple traits
3. **Tavern Dialogue** — flat section map, sticky and one-shot choices, nested sub-choices, goto jumps, `on_exhausted` content, speech with and without speakers
4. **Minimal** — the smallest valid world: just `name` and `urd`
5. **Conditional Visibility** — the object form of visibility with a condition expression
6. **OR Conditions** — the `any` condition block on an action
7. **Spawn Effect** — the spawn effect structure with `id`, `type`, and `in`

### Negative fixtures

Each negative fixture is a minimal `.urd.json` that violates exactly one constraint. They cover: missing required fields (world, name, urd, type, to, sticky), wrong types (integer urd, string sticky), invalid enums (version "2", trait "flying", property type "colour", visibility "public"), `additionalProperties` violations (unknown top-level key, stored exhaustion boolean), mutual exclusion (both target and target_type, both action and actions), empty arrays where content is required (rule effects, select from, sequence phases), invalid patterns (trigger format, advance mode), and conditional requirements (enum without values, conditional visibility without condition).

All 32 fixtures pass validation via `pnpm schema:test`.

## Where it lives

The schema and its test infrastructure:

- **Schema:** [`packages/schema/urd-world-schema.json`](https://github.com/urdwyrd/urd/blob/main/packages/schema/urd-world-schema.json)
- **Test fixtures:** `tests/fixtures/json-schema/positive/` and `tests/fixtures/json-schema/negative/`
- **Validation script:** `scripts/validate-json-schema.mjs`
- **Test command:** `pnpm schema:test`

The schema uses `ajv` (JavaScript/TypeScript) as its reference validator, but any JSON Schema draft 2020-12 validator will work — `jsonschema` (Python), `serde_json` + `jsonschema` (Rust), or any other compliant implementation.

## What comes next

The formalisation phase is now complete. The input side has a PEG grammar. The output side has a JSON Schema. Both are tested and passing.

The next step is the compiler itself: take a `.urd.md` file that passes the grammar, run it through the five-phase pipeline, and produce a `.urd.json` that passes the schema. The Monty Hall problem remains the first target. When `pnpm schema:test` accepts the compiler's output for Monty Hall, the pipeline is connected end to end.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
