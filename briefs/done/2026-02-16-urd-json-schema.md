# URD — World Schema JSON Schema

*A brief for the `.urd.json` validation artifact*

February 2026 | Formalisation Phase

`.urd.md → compiler → .urd.json → JSON Schema says: is this conformant?`

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-16
**Status:** Done

### What was done

- Created `packages/schema/urd-world-schema.json` — complete JSON Schema (draft 2020-12) with all 8 top-level blocks and 9 reusable `$defs` inline
- Created `packages/schema/package.json` — minimal workspace package (`@urd/schema`)
- Created 7 positive test fixtures in `tests/fixtures/json-schema/positive/` (Monty Hall, Key Puzzle, Tavern Dialogue, minimal world, conditional visibility, OR conditions, spawn effect)
- Created 25 negative test fixtures in `tests/fixtures/json-schema/negative/` (N1–N25 per the brief's test case table)
- Created `scripts/validate-json-schema.sh` — bash validation script using `ajv-cli` with `--spec=draft2020`
- All 32 tests pass: 7 positive fixtures validate, 25 negative fixtures correctly rejected

### What changed from the brief

- `propertySchema` `$def` includes the full set of `if/then` conditional validation blocks (all 8 blocks specified in the brief, covering type-specific `default` constraints, `min`/`max` restriction to numeric types, `values` restriction to enum, `ref_type` restriction to ref)
- Validation script path is `scripts/validate-json-schema.sh` (brief suggested `validate-urd-json.sh` but the plan used the more descriptive name)
- Test fixture directory is `tests/fixtures/json-schema/` (brief's script example used `tests/json-schema/` but the brief's file locations table specified `tests/fixtures/json-schema/`)
- No generated HTML docs were produced (brief marked this as optional)

---

> **Document status: BRIEF** — Defines scope, requirements, and acceptance criteria for the JSON Schema that validates compiled `.urd.json` output. This schema is the machine-readable definition of what constitutes valid compiler output. It lives alongside the compiler as a CI validation artifact and exists as a standalone reference for runtime implementers.

---

## Implementation Instructions

> **For the implementing engineer or AI coder.** Read this section before writing any code. It covers what already exists, where artifacts go, and decisions that are already locked.

### What Already Exists

This project has an established codebase. Before starting:

1. **Read the project knowledge files.** The normative specifications are `schema-spec.md` (the World Schema Specification) and `schema-markdown.md` (the Schema Markdown Syntax Specification). The architecture is in `architecture.md`. These are the upstream sources of truth — if anything in this brief conflicts with them, the specs win.
2. **Read the reference cards.** `reference-card-engine-developers.md` has concise JSON examples for every block. Use these as the basis for positive test fixtures — they are the canonical compiled forms.
3. **Read the test case strategy.** `urd-test-case-strategy.md` defines the four test worlds (Tavern, Monty Hall, Key Puzzle, Interrogation). The Monty Hall and Key Puzzle worlds have complete schema-level examples in `schema-spec.md` — these should be your primary positive test fixtures.
4. **The PEG grammar is complete.** `urd-formal-grammar-brief.md` describes the input-side formalisation. The grammar was implemented with pest (Rust PEG parser). This JSON Schema is the output-side counterpart.

### File Locations

Follow the established repository conventions:

| Artifact | Path | Notes |
|----------|------|-------|
| Schema file | `packages/schema/urd-world-schema.json` | The primary deliverable. Single file. |
| Positive test fixtures | `tests/fixtures/json-schema/positive/` | One `.json` file per test case. Minimum 7 (see Test Strategy section). |
| Negative test fixtures | `tests/fixtures/json-schema/negative/` | One `.json` file per negative case. 25 cases defined in this brief. Name each file after its test ID (e.g., `n01-missing-world.json`). |
| Validation script | `scripts/validate-json-schema.sh` | Bash script that runs all fixtures through the validator. Exit 0 = all pass. |
| Generated docs (optional) | `docs/schema/` | If you generate HTML docs from the schema using `json-schema-for-humans`, put them here. |

If a `packages/schema/` directory doesn't exist yet, create it. The schema is a standalone artifact, not part of the compiler package.

### Technology Decisions (Locked)

- **JSON Schema draft 2020-12.** Not draft-07, not draft 2019-09. Use the `$schema` URI `https://json-schema.org/draft/2020-12/schema`.
- **Validator: ajv v8+** with the `Ajv2020` class (not the default `Ajv` class, which targets draft-07). For CLI validation in the test script, use `ajv-cli`. Install: `npm install -g ajv-cli`.
- **Single file.** The schema must be one self-contained `.json` file with all `$defs` inline. No external `$ref` resolution. This keeps the schema portable — anyone can download one file and validate against it.
- **`additionalProperties: false` everywhere.** Every object with a known, closed field set uses this. The brief calls out the specific exceptions (entity property values, map keys). If in doubt, be strict.

### How to Validate Your Work

After implementation, the following must all pass:

```bash
# 1. Schema file itself is valid JSON Schema
ajv compile -s packages/schema/urd-world-schema.json --spec=draft2020

# 2. All positive fixtures pass
for f in tests/fixtures/json-schema/positive/*.json; do
  ajv validate -s packages/schema/urd-world-schema.json -d "$f" --spec=draft2020
done

# 3. All negative fixtures fail
for f in tests/fixtures/json-schema/negative/*.json; do
  ajv validate -s packages/schema/urd-world-schema.json -d "$f" --spec=draft2020
  # This should exit non-zero for every file
done
```

### Build Order

1. **Start with `$defs`.** Build the reusable definitions first: `effect`, `conditionExpr`, `speech`, `visibility`, `propertySchema`, `exit`, `select`, `phase`, `choice`. Test each in isolation.
2. **Build the `world` block.** It's the simplest required block and validates your schema infrastructure.
3. **Build `types` and `entities`.** These exercise `propertySchema` and `visibility`.
4. **Build `locations`, `actions`, `rules`.** These exercise `exit`, `effect`, `conditionExpr`, `select`.
5. **Build `sequences`.** Exercises `phase`.
6. **Build `dialogue`.** Exercises `choice` (recursive) and `speech`.
7. **Wire the root object.** Connect all blocks, add `additionalProperties: false`, set `required: ["world"]`.
8. **Run all fixtures.** Positive first, then negative. Fix any failures.

### What NOT to Do

- **Don't implement semantic validation.** Cross-reference checks (does `start` point to a real location? does `target_type` reference a real type?) are explicitly out of scope. The brief's Validation Boundaries section is authoritative.
- **Don't invent fields.** If a field isn't in this brief, it doesn't go in the schema. The brief was cross-checked against the normative specifications.
- **Don't add `exhausted` to dialogue sections.** This is called out repeatedly in the brief. Exhaustion is a runtime predicate, never stored.
- **Don't use `$ref` to external files.** Everything goes in one file.
- **Don't use YAML.** The project has a locked decision: test definitions and fixtures are JSON only.

---

## Purpose

The Urd framework has two locked specifications: the **World Schema Specification** (what valid `.urd.json` looks like) and the **Schema Markdown Syntax Specification** (what valid `.urd.md` looks like). The PEG grammar formalises the input side. This JSON Schema formalises the output side.

The schema answers one question: **given a `.urd.json` file, is it structurally conformant?** If yes, a runtime can load it with confidence that required fields are present, types are correct, and the document structure matches the specification. If no, the schema produces a precise validation error: which field failed, what was expected, and where.

Without this artifact, the only way to validate `.urd.json` output is to run it through a runtime and see what breaks. The JSON Schema gives us an earlier, cheaper, and more precise check. It also serves as the authoritative reference for anyone implementing a runtime, whether that's Wyrd, a third-party engine integration, or an AI assistant consuming world files.

### What This Is

A JSON Schema (draft 2020-12) file that can be used with any standard JSON Schema validator (ajv, jsonschema, etc.) to check whether a `.urd.json` file is structurally valid. "Structurally valid" means: required fields present, correct types, enum values within declared sets, object shapes match the specification.

### What This Is Not

This is not the PEG grammar for `.urd.md` input. That is a separate formalisation artifact (completed). The PEG grammar validates the *compiler's input*. The JSON Schema validates the *compiler's output*. Both are needed. Neither replaces the other.

This is also not a *semantic* validator. The JSON Schema can verify that a `conditions` field is an array of strings, but it cannot verify that those condition strings reference entities that exist in the `entities` block. Semantic cross-reference validation remains the compiler's responsibility (phases 3–4) and the runtime's load-time check. The JSON Schema draws the line between "this document has the right shape" and "this document is internally consistent."

### Relationship to the PEG Grammar

The formalisation phase produces **two** artifacts that work together:

| Artifact | Validates | Format | When Used |
|---|---|---|---|
| PEG grammar (completed) | Compiler input (`.urd.md`) | PEG file | Parse phase. Before compilation. |
| JSON Schema (this brief) | Compiler output (`.urd.json`) | JSON Schema file | After compilation. CI checks. Runtime load. |

Together they form the machine-checkable specification. The prose specs define what Urd means. The grammar and JSON Schema define what valid Urd looks like — for input and output respectively. The compiler connects them: valid input (per the grammar) → valid output (per the JSON Schema).

```
  .urd.md ──── PEG grammar says: is this well-formed? ────┐
               (syntactic check)                            │
                                                            ▼
           ┌─── COMPILER (phases 2–5) ────────────────────────┐
           │    import, link, validate, emit                   │
           └───────────────────────────────────────────────────┘
                            │
                            ▼
  .urd.json ─── JSON Schema says: is this conformant? ───┐
                (structural check)                        │
                                                          ▼
                                                     ✓ VALID
```

## Where It Lives

The schema exists in two forms:

- **As a reference artifact.** A standalone file (`urd-world-schema.json`) in the repository. Human-readable. The authoritative definition of the output structure. Versioned alongside the specifications.
- **As a CI gate.** The build pipeline validates every compiled `.urd.json` file against this schema before accepting it. A schema violation is a build failure.

The schema file is the source of truth for structural validity. If a runtime's loader and the schema disagree on what constitutes valid structure, the schema wins.

> **Architectural note.** The JSON Schema covers structural validity only. It can tell you that the `entities` block is an object whose values have `type` and `properties` fields. It cannot tell you that `type: "Door"` references a type that actually exists in the `types` block. That level of cross-reference checking is semantic validation — the compiler's job (phase 4) and the runtime's load-time responsibility.

## Format Choice: JSON Schema Draft 2020-12

The schema should be written in **JSON Schema draft 2020-12**, the current stable specification. This is the right fit for five reasons:

- **Industry standard.** JSON Schema is the de facto standard for describing JSON document structure. Every major language has validators.
- **Expressive enough.** Draft 2020-12 supports `$defs` for reusable sub-schemas, `if/then/else` for conditional validation (needed for the visibility model), `oneOf`/`anyOf` for union types, and `patternProperties` for the flat-map structures in `dialogue` and `entities`.
- **Tooling exists everywhere.** `ajv` (JavaScript/TypeScript), `jsonschema` (Python), `serde_json` + `jsonschema` (Rust), and many others. The schema can be consumed by any runtime implementation without custom tooling.
- **Self-documenting.** JSON Schema supports `description` and `title` annotations on every node, making the schema itself readable documentation.
- **Composable.** Sub-schemas in `$defs` can be referenced and reused, keeping the schema DRY while covering all eight top-level blocks.

## Schema Structure Outline

The generated schema file has the following top-level layout:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://urd.dev/schema/v1/urd-world-schema.json",
  "title": "Urd World Schema v1",
  "description": "Validates compiled .urd.json world files.",
  "type": "object",
  "required": ["world"],
  "additionalProperties": false,
  "properties": {
    "world":     { "...inline or $ref..." },
    "types":     { "$ref": "#/$defs/typesBlock" },
    "entities":  { "$ref": "#/$defs/entitiesBlock" },
    "locations": { "$ref": "#/$defs/locationsBlock" },
    "rules":     { "$ref": "#/$defs/rulesBlock" },
    "actions":   { "$ref": "#/$defs/actionsBlock" },
    "sequences": { "$ref": "#/$defs/sequencesBlock" },
    "dialogue":  { "$ref": "#/$defs/dialogueBlock" }
  },
  "$defs": {
    "typesBlock":     "...",
    "entitiesBlock":  "...",
    "locationsBlock": "...",
    "rulesBlock":     "...",
    "actionsBlock":   "...",
    "sequencesBlock": "...",
    "dialogueBlock":  "...",
    "effect":         "— five effect types (oneOf)",
    "conditionExpr":  "— AND list or any OR block (oneOf)",
    "speech":         "— { speaker?, text } objects",
    "choice":         "— recursive choice structure",
    "exit":           "— exit with to, condition, blocked_message, effects",
    "visibility":     "— simple string or conditional object (oneOf)",
    "propertySchema": "— property definition with conditional type validation",
    "phase":          "— sequence phase with advance modes",
    "select":         "— constrained random choice block"
  }
}
```

The `$id` URI is a namespace identifier. It does not need to resolve to a hosted URL in v1, but should be treated as a stable identifier for tooling that caches schemas. The `world` block may be defined inline (it is small and has no reuse) or via `$ref` — implementer's choice.

## Scope: What the Schema Defines

The JSON Schema must validate the complete structure of a compiled `.urd.json` file. This section enumerates every structural element the schema must accept or reject, organised by top-level block.

### Document Root

A `.urd.json` file is a JSON object with up to eight top-level keys. Only `world` is required. All others are optional. No additional top-level keys are permitted in v1 (use `additionalProperties: false` on the root object).

```json
{
  "world": { },
  "types": { },
  "entities": { },
  "locations": { },
  "rules": { },
  "actions": { },
  "sequences": { },
  "dialogue": { }
}
```

The schema must enforce:
- `world` is required and must be an object.
- All other top-level keys are optional.
- No unknown top-level keys are permitted.
- Every optional block must be typed as `"type": "object"` (either inline or via its `$ref`). This prevents `rules: []` or `dialogue: "none"` from passing validation.

### The `world` Block

Metadata about the world. The only required block.

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `name` | string | Yes | Pattern: `^[a-z][a-z0-9-]*$` (lowercase, hyphens, starts with letter). |
| `urd` | string | Yes | Must be `"1"` for v1. Enum constraint. |
| `version` | string | No | Author-defined. No format constraint. |
| `description` | string | No | |
| `author` | string | No | |
| `start` | string | No | Location reference. (Semantic: must resolve to a key in `locations`. Schema checks type only.) |
| `entry` | string | No | Sequence reference. (Semantic: must resolve to a key in `sequences`. Schema checks type only.) |
| `seed` | integer | No | Deterministic replay seed. |

Additional properties on the `world` block are not permitted.

**Schema fragment:**

```json
{
  "world": {
    "type": "object",
    "required": ["name", "urd"],
    "additionalProperties": false,
    "properties": {
      "name": {
        "type": "string",
        "pattern": "^[a-z][a-z0-9-]*$",
        "description": "Unique world identifier. Lowercase, hyphens allowed, starts with letter."
      },
      "urd": {
        "type": "string",
        "enum": ["1"],
        "description": "Schema version. Always '1' for v1. Set by compiler."
      },
      "version": { "type": "string" },
      "description": { "type": "string" },
      "author": { "type": "string" },
      "start": {
        "type": "string",
        "description": "Starting location reference."
      },
      "entry": {
        "type": "string",
        "description": "Entry sequence reference."
      },
      "seed": {
        "type": "integer",
        "description": "Random seed for deterministic replay."
      }
    }
  }
}
```

### The `types` Block

An object whose keys are type names and whose values are type definitions. Type names are PascalCase identifiers.

#### Type Definition

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `description` | string | No | Human-readable description. |
| `traits` | array of strings | No | Each element must be one of: `container`, `portable`, `mobile`, `interactable`. No duplicates. |
| `properties` | object | No | Keys are property names (lowercase with underscores). Values are property schema objects (`$ref: propertySchema`). |

**Schema fragment:**

```json
{
  "types": {
    "type": "object",
    "additionalProperties": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "description": { "type": "string" },
        "traits": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": ["container", "portable", "mobile", "interactable"]
          },
          "uniqueItems": true
        },
        "properties": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/propertySchema" }
        }
      }
    }
  }
}
```

#### Property Schema

Each property within a type is described by a property schema object. The `type` field is always required and determines which additional fields are valid.

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `type` | string | Yes | One of: `boolean`, `integer`, `number`, `string`, `enum`, `ref`, `list`. |
| `default` | any | No | Must match the declared `type`. |
| `visibility` | string or object | No | See Visibility below. Default: `"visible"`. |
| `description` | string | No | Human-readable explanation. |
| `values` | array of strings | Conditional | Required when `type` is `enum`. The valid value set. Minimum one value. |
| `min` | number | No | Valid when `type` is `integer` or `number`. |
| `max` | number | No | Valid when `type` is `integer` or `number`. |
| `ref_type` | string | No | Valid when `type` is `ref`. The entity type this ref must point to. |

**Conditional validation rules (enforced via `if/then`):**

1. When `type` is `enum`, `values` is required and must be a non-empty array of strings.
2. When `type` is `boolean`, `default` (if present) must be a boolean.
3. When `type` is `integer`, `default` (if present) must be an integer. `min` and `max` (if present) must be integers.
4. When `type` is `number`, `default` (if present) must be a number. `min` and `max` (if present) must be numbers.
5. When `type` is `string`, `default` (if present) must be a string.
6. When `type` is `ref`, `ref_type` (if present) must be a string.
7. When `type` is `list`, `default` (if present) must be an array. The item types within the list are not structurally constrained — a list may contain strings, numbers, entity refs, or mixed types depending on usage. Item type validation is a semantic concern resolved by the compiler against the type definition context.
8. `min` and `max` are only valid when `type` is `integer` or `number`.
9. `values` is only valid when `type` is `enum`.
10. `ref_type` is only valid when `type` is `ref`.
11. No unknown keys on `propertySchema`. Enforced via `additionalProperties: false` on the definition itself, not via conditional rules. Catches typos like `refType` or `min_value`.

#### Visibility

The `visibility` field has two valid forms:

**Simple form (string):** One of `"visible"`, `"hidden"`, `"owner"`.

**Conditional form (object):**

```json
{
  "type": "conditional",
  "condition": "magnifying_glass.container == player"
}
```

The schema validates this using `oneOf`:

```json
{
  "visibility": {
    "oneOf": [
      {
        "type": "string",
        "enum": ["visible", "hidden", "owner"]
      },
      {
        "type": "object",
        "required": ["type", "condition"],
        "additionalProperties": false,
        "properties": {
          "type": { "const": "conditional" },
          "condition": { "type": "string", "minLength": 1 }
        }
      }
    ],
    "default": "visible"
  }
}
```

**Schema fragment (`propertySchema` `$def`):**

The property schema definition is the most complex reusable type in the schema due to the conditional validation rules. Each `if/then` block keys on the `type` field to enforce type-specific constraints.

```json
{
  "$defs": {
    "propertySchema": {
      "type": "object",
      "required": ["type"],
      "additionalProperties": false,
      "properties": {
        "type": {
          "type": "string",
          "enum": ["boolean", "integer", "number", "string", "enum", "ref", "list"]
        },
        "default": {
          "description": "Unconstrained here. Constrained by if/then rules keyed on type."
        },
        "visibility": { "$ref": "#/$defs/visibility" },
        "description": { "type": "string" },
        "values": {
          "type": "array",
          "items": { "type": "string" },
          "minItems": 1
        },
        "min": { "type": "number" },
        "max": { "type": "number" },
        "ref_type": { "type": "string" }
      },
      "allOf": [
        {
          "if": {
            "required": ["type"],
            "properties": { "type": { "const": "enum" } }
          },
          "then": { "required": ["values"] }
        }
      ]
    }
  }
}
```

> **On `default`:** The `default` field schema is intentionally annotation-only (no `type` constraint). In JSON Schema draft 2020-12, an object containing only `description` imposes no type restriction — any JSON value is accepted. The `if/then` conditional rules enforce the correct type per property kind. Do not add a `type` keyword to `default` or it will reject valid defaults for other property types.

> **Implementation note:** The full `propertySchema` needs additional `if/then` blocks for each type (constraining `default` type, restricting `min`/`max` to numeric types, restricting `values` to enum, restricting `ref_type` to ref). The fragment above shows the pattern; the implementation must include all eleven conditional rules listed below. **Every `if` block must include `"required": ["type"]`** alongside the `const` check. Without it, the condition matches vacuously when `type` is absent, causing the `then` clause to fire incorrectly.

### The `entities` Block

An object whose keys are entity IDs and whose values are entity instance objects. Entity IDs are lowercase identifiers with underscores.

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `type` | string | Yes | Type name reference. (Semantic: must resolve to a key in `types`. Schema checks type only.) |
| `properties` | object | No | Keys are property names. Values are the property values. Types are not schema-checkable here — they depend on the referenced type definition, which is a semantic concern. |

Entity property values are limited to primitive JSON types and arrays — no nested objects. This matches the property type system (boolean, integer, number, string, enum, ref, list).

**Schema fragment:**

```json
{
  "entities": {
    "type": "object",
    "additionalProperties": {
      "type": "object",
      "required": ["type"],
      "properties": {
        "type": { "type": "string", "minLength": 1 },
        "properties": {
          "type": "object",
          "additionalProperties": {
            "oneOf": [
              { "type": "string" },
              { "type": "number" },
              { "type": "boolean" },
              { "type": "array" },
              { "type": "null" }
            ]
          }
        }
      },
      "additionalProperties": false
    }
  }
}
```

### The `locations` Block

An object whose keys are location IDs and whose values are location objects.

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `description` | string | No | Human-readable description of the location. |
| `contains` | array of strings | No | Entity IDs present in this location at start. |
| `exits` | object | No | Keys are direction labels. Values are exit objects. |

#### Exit Object

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `to` | string | Yes | Destination location reference. |
| `condition` | string | No | Condition expression that must be true for traversal. |
| `blocked_message` | string | No | Shown when condition is false. |
| `effects` | array | No | Effects applied when the exit is used. Array of effect objects. |

Exits are unidirectional. The schema does not enforce bidirectionality (that's a linting concern, not structural validity).

**Schema fragment:**

```json
{
  "locations": {
    "type": "object",
    "additionalProperties": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "description": { "type": "string" },
        "contains": {
          "type": "array",
          "items": { "type": "string" }
        },
        "exits": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/exit" }
        }
      }
    }
  }
}
```

```json
{
  "$defs": {
    "exit": {
      "type": "object",
      "required": ["to"],
      "additionalProperties": false,
      "properties": {
        "to": { "type": "string", "minLength": 1 },
        "condition": { "type": "string" },
        "blocked_message": { "type": "string" },
        "effects": {
          "type": "array",
          "items": { "$ref": "#/$defs/effect" }
        }
      }
    }
  }
}
```

> **Exit `condition` is singular (a plain string), not plural.** An exit has a single gating expression, not an AND/OR block. This follows the naming convention: `condition` (singular) = single string; `conditions` (plural) = `$ref: conditionExpr`.

### The `rules` Block

An object whose keys are rule IDs and whose values are rule objects.

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `description` | string | No | Human-readable description. |
| `actor` | string | No | Entity reference. Who performs this rule. |
| `trigger` | string | Yes | Trigger expression. One of five forms (see Trigger Types). |
| `conditions` | `$ref: conditionExpr` | No | AND list or `any` OR block. See Condition Expressions. |
| `select` | object | No | The select block for constrained random choice. |
| `effects` | array | Yes | At least one effect. A rule with no effects is invalid. |

#### Trigger Types

The `trigger` field is a string matching one of five patterns:

| Trigger | Pattern | Example |
|---------|---------|---------|
| `phase_is <id>` | Starts with `"phase_is "` | `"phase_is reveal"` |
| `action <id>` | Starts with `"action "` | `"action choose_door"` |
| `enter <location>` | Starts with `"enter "` | `"enter cell"` |
| `state_change <entity.prop>` | Starts with `"state_change "` | `"state_change guard.mood"` |
| `always` | Exact string | `"always"` |

The schema validates this as a string with a pattern that matches any of these forms:

```json
{
  "trigger": {
    "type": "string",
    "pattern": "^(phase_is \\S+|action \\S+|enter \\S+|state_change \\S+|always)$"
  }
}
```

**Schema fragment:**

```json
{
  "rules": {
    "type": "object",
    "additionalProperties": {
      "type": "object",
      "required": ["trigger", "effects"],
      "additionalProperties": false,
      "properties": {
        "description": { "type": "string" },
        "actor": { "type": "string" },
        "trigger": {
          "type": "string",
          "pattern": "^(phase_is \\S+|action \\S+|enter \\S+|state_change \\S+|always)$"
        },
        "conditions": { "$ref": "#/$defs/conditionExpr" },
        "select": { "$ref": "#/$defs/select" },
        "effects": {
          "type": "array",
          "items": { "$ref": "#/$defs/effect" },
          "minItems": 1,
          "description": "At least one effect. A rule with no effects is invalid."
        }
      }
    }
  }
}
```

> **Rules vs actions on effects:** Rules require `minItems: 1` — a rule with no effects is meaningless. Actions allow an empty array (`effects: []`) to support no-op actions like `stay` that exist solely to advance a sequence phase.

#### The `select` Block

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `from` | array of strings | Yes | Candidate entity IDs. Non-empty. |
| `as` | string | Yes | Variable name bound to the selection. |
| `where` | array of strings | No | Conditions each candidate must satisfy. |

```json
{
  "select": {
    "type": "object",
    "required": ["from", "as"],
    "additionalProperties": false,
    "properties": {
      "from": {
        "type": "array",
        "items": { "type": "string" },
        "minItems": 1
      },
      "as": { "type": "string", "minLength": 1 },
      "where": {
        "type": "array",
        "items": { "type": "string" },
        "minItems": 1,
        "description": "Simple condition strings only. Each string is evaluated against the bound variable. Does not use compound conditionExpr (no any: OR blocks) because where clauses filter individual candidates, not world state."
      }
    }
  }
}
```

### The `actions` Block

An object whose keys are action IDs and whose values are action objects.

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `description` | string | No | Shown to the player as the action label. |
| `actor` | string | No | Entity reference. Default: `"player"`. |
| `target` | string | No | Specific entity reference. Mutually exclusive with `target_type`. |
| `target_type` | string | No | Type name reference. Mutually exclusive with `target`. |
| `conditions` | `$ref: conditionExpr` | No | AND list or `any` OR block. See Condition Expressions. |
| `effects` | array | Yes | Required. Array of effect objects. May be empty for no-op actions (e.g., `stay`). |

**Mutual exclusion constraint:** An action may have `target`, `target_type`, or neither (self-targeted). It must not have both. This is expressed via `oneOf` or `not` + `required`:

```json
{
  "not": {
    "required": ["target", "target_type"]
  }
}
```

> **Note on empty effects:** The schema spec shows `stay` with `effects: []`. The JSON Schema allows an empty effects array on actions (no `minItems`). The compiler's validation phase (not the JSON Schema) enforces the semantic rule that actions *do something meaningful*. The `stay` action is the canonical exception — it exists as a player choice that advances a sequence phase without state mutation.

**Schema fragment:**

```json
{
  "actions": {
    "type": "object",
    "additionalProperties": {
      "type": "object",
      "required": ["effects"],
      "additionalProperties": false,
      "properties": {
        "description": { "type": "string" },
        "actor": { "type": "string" },
        "target": { "type": "string" },
        "target_type": { "type": "string" },
        "conditions": { "$ref": "#/$defs/conditionExpr" },
        "effects": {
          "type": "array",
          "items": { "$ref": "#/$defs/effect" }
        }
      },
      "not": { "required": ["target", "target_type"] }
    }
  }
}
```

### The `sequences` Block

An object whose keys are sequence IDs and whose values are sequence objects.

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `description` | string | No | Human-readable description. |
| `phases` | array | Yes | Ordered list of phase objects. Non-empty. |

#### Phase Object

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique phase identifier within the sequence. |
| `prompt` | string | No | Text shown to the player. |
| `auto` | boolean | No | If true, executes without player input. |
| `action` | string | No | Single action reference. |
| `actions` | array of strings | No | Multiple action references. Mutually exclusive with `action`. |
| `rule` | string | No | Rule reference that fires in this phase. |
| `effects` | array | No | Effects applied when the phase begins. |
| `advance` | string | Yes | Advance mode. Pattern: `^(on_action|on_rule|on_condition \S+|end)$`. |
| `condition` | string | No | Phase is skipped if this evaluates to false. |

**Mutual exclusion constraint:** A phase may have `action` (single) or `actions` (list), not both.

```json
{
  "not": {
    "required": ["action", "actions"]
  }
}
```

**Advance mode validation:**

```json
{
  "advance": {
    "type": "string",
    "pattern": "^(on_action|on_rule|on_condition \\S+|end)$",
    "description": "When to advance to the next phase."
  }
}
```

**Schema fragment (full sequences block and phase `$def`):**

```json
{
  "sequences": {
    "type": "object",
    "additionalProperties": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "description": { "type": "string" },
        "phases": {
          "type": "array",
          "items": { "$ref": "#/$defs/phase" },
          "minItems": 1
        }
      },
      "required": ["phases"]
    }
  }
}
```

```json
{
  "$defs": {
    "phase": {
      "type": "object",
      "required": ["id", "advance"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "prompt": { "type": "string" },
        "auto": { "type": "boolean" },
        "action": { "type": "string" },
        "actions": {
          "type": "array",
          "items": { "type": "string" }
        },
        "rule": { "type": "string" },
        "effects": {
          "type": "array",
          "items": { "$ref": "#/$defs/effect" }
        },
        "advance": {
          "type": "string",
          "pattern": "^(on_action|on_rule|on_condition \\S+|end)$"
        },
        "condition": { "type": "string" }
      },
      "not": { "required": ["action", "actions"] }
    }
  }
}
```

### The `dialogue` Block

A flat map of sections keyed by world-unique section IDs. Keys follow the pattern `file_stem/section_name` (e.g., `"tavern/topics"`).

#### Section Object

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `id` | string | Yes | Non-empty string. Convention: matches the map key (`file_stem/section_name`). Key-matching is not structurally enforceable; treated as compiler-guaranteed. |
| `prompt` | object | No | Speaker prompt. `{ "speaker": string, "text": string }`. |
| `description` | string | No | Prose narration before the prompt. |
| `choices` | array | No | Array of choice objects. |
| `conditions` | `$ref: conditionExpr` | No | Reserved for future use. Conditions for section accessibility. |
| `on_exhausted` | object | No | Fallthrough content. Contains `text` (required) and optionally `speaker`. |

**Critical: no `exhausted` boolean field.** The schema must NOT define an `exhausted` field on sections. Exhaustion is a runtime-evaluated predicate, never stored. The `on_exhausted` field contains *content*, not state.

**Schema fragment (full dialogue block):**

```json
{
  "dialogue": {
    "type": "object",
    "additionalProperties": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "prompt": { "$ref": "#/$defs/speech" },
        "description": { "type": "string" },
        "choices": {
          "type": "array",
          "items": { "$ref": "#/$defs/choice" }
        },
        "conditions": { "$ref": "#/$defs/conditionExpr" },
        "on_exhausted": { "$ref": "#/$defs/speech" }
      },
      "required": ["id"]
    }
  }
}
```

#### Prompt / Response Object

Used for section prompts and choice responses:

```json
{
  "$defs": {
    "speech": {
      "type": "object",
      "required": ["text"],
      "additionalProperties": false,
      "properties": {
        "speaker": { "type": "string" },
        "text": { "type": "string", "minLength": 1 }
      }
    }
  }
}
```

`speaker` is optional on `on_exhausted` (narration may not have a speaker). It is also optional on `prompt` (a section might open with player-facing narration rather than NPC speech).

#### Choice Object

| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| `id` | string | Yes | World-unique choice ID. Format: `section_id/slugified-label`. |
| `label` | string | Yes | Text shown to the player. Non-empty. |
| `sticky` | boolean | Yes | `true` = stays available; `false` = consumed after selection. |
| `conditions` | `$ref: conditionExpr` | No | Conditions for this choice to appear. |
| `response` | object | No | Speech object. Dialogue spoken when selected. |
| `effects` | array | No | Effects applied when selected. |
| `goto` | string | No | Section ID to jump to. Omit = stay in current section. |
| `choices` | array | No | Inline sub-choices. Same structure (recursive). |

The `choices` field is **recursive**: a choice can contain nested sub-choices with the same schema. This handles inline branching (indentation-based nesting in the source syntax).

```json
{
  "$defs": {
    "choice": {
      "type": "object",
      "required": ["id", "label", "sticky"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "label": { "type": "string", "minLength": 1 },
        "sticky": { "type": "boolean" },
        "conditions": { "$ref": "#/$defs/conditionExpr" },
        "response": { "$ref": "#/$defs/speech" },
        "effects": {
          "type": "array",
          "items": { "$ref": "#/$defs/effect" }
        },
        "goto": { "type": "string" },
        "choices": {
          "type": "array",
          "items": { "$ref": "#/$defs/choice" }
        }
      }
    }
  }
}
```

### Condition Expressions

Conditions appear in multiple blocks. They take two different forms depending on context:

**Compound form (`$ref: conditionExpr`):** Used by `rules.conditions`, `actions.conditions`, `dialogue.sections[].conditions`, and `dialogue.choices[].conditions`. Supports either an AND list (array of strings) or an `any` OR block (object).

**Single string form:** Used by `exits[].condition`, `phases[].condition`, and visibility `condition`. A single expression string, not wrapped in an array.

> **Wiring rule:** Every `conditions` field (plural) in the schema must reference `$ref: "#/$defs/conditionExpr"`. Every `condition` field (singular) is typed as `{ "type": "string" }`. This is the naming convention: plural = compound, singular = simple.

The compound form has two structural variants:

**Simple form (AND list):** An array of condition strings. All must be true.

```json
["cell_door.locked == true", "rusty_key.container == player"]
```

**OR form (any block):** An object with a single `any` key containing an array of condition strings. Any one being true validates the block.

```json
{
  "any": [
    "player.reputation > 50",
    "bribe_gold.container == player"
  ]
}
```

The schema defines a reusable `conditionExpr` that accepts either form:

```json
{
  "$defs": {
    "conditionExpr": {
      "oneOf": [
        {
          "type": "array",
          "items": { "type": "string" },
          "minItems": 1,
          "description": "AND list. All conditions must be true."
        },
        {
          "type": "object",
          "required": ["any"],
          "additionalProperties": false,
          "properties": {
            "any": {
              "type": "array",
              "items": { "type": "string" },
              "minItems": 1,
              "description": "OR list. Any one condition being true validates the block."
            }
          }
        }
      ]
    }
  }
}
```

### Effect Declarations

Effects appear in actions, rules, exits, dialogue choices, and sequence phases. Each effect is an object matching one of five forms:

| Effect | Required Fields | Optional Fields |
|--------|-----------------|-----------------|
| `set` | `set` (string), `to` (any) | — |
| `move` | `move` (string), `to` (string) | — |
| `reveal` | `reveal` (string) | — |
| `destroy` | `destroy` (string) | — |
| `spawn` | `spawn` (object: `id`, `type`, `in`) | — |

The schema validates effects using `oneOf` across the five forms:

```json
{
  "$defs": {
    "effect": {
      "oneOf": [
        {
          "type": "object",
          "required": ["set", "to"],
          "additionalProperties": false,
          "properties": {
            "set": { "type": "string", "minLength": 1 },
            "to": {
              "description": "New value. May be a literal or an expression (e.g., 'arina.trust + 5')."
            }
          },
          "description": "Set a property to a new value."
        },
        {
          "type": "object",
          "required": ["move", "to"],
          "additionalProperties": false,
          "properties": {
            "move": { "type": "string", "minLength": 1 },
            "to": { "type": "string", "minLength": 1 }
          },
          "description": "Move an entity into a different container."
        },
        {
          "type": "object",
          "required": ["reveal"],
          "additionalProperties": false,
          "properties": {
            "reveal": { "type": "string", "minLength": 1 }
          },
          "description": "Change a hidden property's visibility to visible."
        },
        {
          "type": "object",
          "required": ["destroy"],
          "additionalProperties": false,
          "properties": {
            "destroy": { "type": "string", "minLength": 1 }
          },
          "description": "Remove an entity from the world."
        },
        {
          "type": "object",
          "required": ["spawn"],
          "additionalProperties": false,
          "properties": {
            "spawn": {
              "type": "object",
              "required": ["id", "type", "in"],
              "additionalProperties": false,
              "properties": {
                "id": { "type": "string", "minLength": 1 },
                "type": { "type": "string", "minLength": 1 },
                "in": { "type": "string", "minLength": 1 }
              }
            }
          },
          "description": "Create a new entity at runtime."
        }
      ]
    }
  }
}
```

**Note on `set.to`:** The `to` field in a set effect has no type constraint in the schema. It can be a string (`"open"`), a number (`5`), a boolean (`true`), or an expression string (`"arina.trust + 5"`). The correct type depends on the target property's declared type, which is a semantic cross-reference check beyond the JSON Schema's scope.

### Stable ID Patterns

The compiled JSON uses deterministic, world-unique IDs derived from source structure:

| Element | Format | Example | Pattern |
|---------|--------|---------|---------|
| Entity | Declared `@name` | `rusty_key` | `^[a-z][a-z0-9_]*$` |
| Section | `file_stem/section_name` | `tavern/topics` | Contains `/` |
| Choice | `section_id/slugified-label` | `tavern/topics/ask-about-the-harbor` | Contains at least two `/` segments |

**ID validation policy:** Map keys (entity IDs, location IDs, rule IDs, action IDs, sequence IDs, section IDs) are trusted compiler output. The schema does not enforce key format patterns because the compiler is solely responsible for generating valid keys. However, embedded `id` fields *within* objects (dialogue section `id`, choice `id`) are validated as non-empty strings. The `world.name` field is the one exception where a key-like value gets a strict pattern, because it is author-supplied (not compiler-generated) and is used in file naming.

## Reusable Sub-Schemas (`$defs`)

The following reusable definitions appear in `$defs` and are referenced throughout the schema:

| Definition | Used By | Description |
|------------|---------|-------------|
| `effect` | actions, rules, exits, choices, phases | The five effect types. |
| `conditionExpr` | actions, rules, exits, choices, sections | AND list or `any` OR block. |
| `speech` | section prompts, choice responses, `on_exhausted` | `{ speaker?, text }` objects. |
| `choice` | dialogue sections, recursive sub-choices | The choice object (self-referential). |
| `exit` | locations | Exit structure with `to`, `condition`, `blocked_message`, `effects`. |
| `visibility` | property schemas within types | Simple string or conditional object. |
| `propertySchema` | type definitions | The full property schema with conditional type validation. |
| `phase` | sequences | Phase object with advance modes. |
| `select` | rules | The select block for constrained random choice. |

## Validation Boundaries

The JSON Schema is deliberately conservative about what it validates. This section makes the boundary explicit.

### What the Schema Validates (Structural)

- Required fields are present on every object.
- Field types are correct (string, number, boolean, array, object).
- Enum fields contain valid values (e.g., `urd` is `"1"`, traits are from the valid set).
- Object shapes match the specification (no extra keys via `additionalProperties: false`).
- Arrays contain items of the correct type.
- Mutual exclusion constraints (target/target_type on actions; action/actions on phases).
- Conditional field requirements (e.g., `values` required when `type` is `enum`).
- Pattern matching on structured strings (trigger patterns, advance modes, world name).
- Recursive structures (nested choices in dialogue).

### What the Schema Does NOT Validate (Semantic)

- Cross-reference integrity: `entities.door_1.type` references `types.Door` — the schema cannot check this.
- Property value type matching: `entities.door_1.properties.prize` should be one of `["goat", "car"]` per the Door type — this requires knowing the type definition.
- Reference resolution: `start` location exists in `locations`, `entry` sequence exists in `sequences`.
- Duplicate detection: Two entities with the same ID (already prevented by JSON object key uniqueness).
- Condition expression syntax: The schema validates conditions are strings; it cannot parse the expression language.
- Effect target validity: `set: "guard.mood"` references a real entity and property.
- Dialogue section ID consistency: `id` field matches the map key.
- Choice ID derivation: `id` follows the `section_id/slugified-label` pattern consistently.

These semantic checks are the compiler's responsibility (phases 3–4) and the runtime's load-time validation.

## Test Strategy

### Positive Test Cases

The schema must accept every example from the normative specification documents. The test corpus includes:

**1. Monty Hall (complete world with sequences, rules, select)**

The full Monty Hall `.urd.json` as it would be emitted by a correct compiler. Exercises: `world` (all fields), `types` (enum, boolean, string with hidden visibility), `entities` (property overrides), `locations` (single, with contains), `rules` (trigger, select, effects), `actions` (target_type, conditions, effects, empty effects), `sequences` (all four advance modes).

**2. Two Room Key Puzzle (freeform world with spatial mechanics)**

The full key puzzle `.urd.json`. Exercises: `locations` (two, with exits and conditions and blocked_message), `entities` (ref type, owner visibility), traits (portable, mobile, container, interactable), `actions` (specific target, move effects, destroy effects), containment model, no sequences block (freeform).

**3. Tavern Dialogue (dialogue block with sections, choices, exhaustion)**

A `.urd.json` exercising the dialogue block. Exercises: flat section map with `file_stem/section_name` keys, section prompts with speakers, one-shot and sticky choices, nested sub-choices (recursive), conditions on choices, effects on choices, `goto` jumps between sections, `on_exhausted` fallthrough content, section `description` field.

**4. Minimal Valid World**

The smallest possible valid `.urd.json`:

```json
{
  "world": {
    "name": "empty",
    "urd": "1"
  }
}
```

This validates that only `world` is required and all other blocks are optional.

**5. Conditional Visibility**

A `.urd.json` with a property using the conditional visibility object form:

```json
{
  "world": { "name": "visibility-test", "urd": "1" },
  "types": {
    "Clue": {
      "properties": {
        "secret_message": {
          "type": "string",
          "default": "The key is under the stone",
          "visibility": {
            "type": "conditional",
            "condition": "magnifying_glass.container == player"
          }
        }
      }
    }
  }
}
```

**6. OR Conditions**

A `.urd.json` exercising the `any` condition block in an action:

```json
{
  "world": { "name": "or-test", "urd": "1" },
  "actions": {
    "enter_tavern": {
      "conditions": {
        "any": [
          "player.reputation > 50",
          "bribe_gold.container == player"
        ]
      },
      "effects": [{ "move": "player", "to": "tavern" }]
    }
  }
}
```

**7. Spawn Effect**

A `.urd.json` exercising the spawn effect:

```json
{
  "world": { "name": "spawn-test", "urd": "1" },
  "actions": {
    "summon": {
      "effects": [
        { "spawn": { "id": "familiar", "type": "Creature", "in": "player.container" } }
      ]
    }
  }
}
```

### Negative Test Cases

The schema must reject each of these and produce a meaningful validation error:

| # | Input | Expected Error | What It Tests |
|---|-------|----------------|---------------|
| N1 | Missing `world` block | `required: ["world"]` | Root-level requirement. |
| N2 | `world` missing `name` | `required: ["name", "urd"]` | World field requirements. |
| N3 | `world` missing `urd` | `required: ["name", "urd"]` | World field requirements. |
| N4 | `urd: "2"` | `enum: ["1"]` | Version enforcement. |
| N5 | `urd: 1` (integer, not string) | `type: string` | Version type. |
| N6 | `name: "My World"` (spaces, uppercase) | `pattern` failure | Name format. |
| N7 | Unknown top-level key `"meta": {}` | `additionalProperties` | Root strictness. |
| N8 | Entity missing `type` field | `required: ["type"]` | Entity structure. |
| N9 | Type property missing `type` field | `required: ["type"]` | Property schema structure. |
| N10 | Property `type: "colour"` (invalid) | `enum` failure | Property type enum. |
| N11 | Enum property without `values` | `if/then` conditional | Conditional requirement. |
| N12 | Trait `"flying"` (not in valid set) | `enum` on items | Trait validation. |
| N13 | Action with both `target` and `target_type` | `not: required` | Mutual exclusion. |
| N14 | Phase with both `action` and `actions` | `not: required` | Mutual exclusion. |
| N15 | Rule with empty `effects: []` | `minItems: 1` | Rule must have effects. |
| N16 | Select with empty `from: []` | `minItems: 1` | Select needs candidates. |
| N17 | Exit missing `to` field | `required: ["to"]` | Exit structure. |
| N18 | Choice missing `sticky` field | `required: ["id", "label", "sticky"]` | Choice structure. |
| N19 | Choice with `sticky: "yes"` (string) | `type: boolean` | Sticky type. |
| N20 | Section with `exhausted: true` field | `additionalProperties` | No stored exhaustion. |
| N21 | Advance mode `"on_time 30s"` | `pattern` failure | Advance mode validation. |
| N22 | Sequence with empty `phases: []` | `minItems: 1` | Phases required. |
| N23 | Visibility `"public"` (invalid) | `oneOf` failure | Visibility enum. |
| N24 | Conditional visibility missing `condition` | `required: ["type", "condition"]` | Conditional visibility structure. |
| N25 | Trigger `"on_timer 30"` (invalid) | `pattern` failure | Trigger format. |

### Validation Script

A validation script (`validate-urd-json.sh`) exercises every test case:

```bash
#!/bin/bash
set -euo pipefail

SCHEMA="urd-world-schema.json"
PASS_DIR="tests/json-schema/positive"
FAIL_DIR="tests/json-schema/negative"
VALIDATOR="ajv"  # or any JSON Schema validator CLI

echo "=== Positive tests ==="
for f in "$PASS_DIR"/*.json; do
  if $VALIDATOR validate -s "$SCHEMA" -d "$f" 2>/dev/null; then
    echo "PASS: $(basename $f)"
  else
    echo "FAIL: $(basename $f) — should have passed"
    exit 1
  fi
done

echo "=== Negative tests ==="
for f in "$FAIL_DIR"/*.json; do
  if $VALIDATOR validate -s "$SCHEMA" -d "$f" 2>/dev/null; then
    echo "FAIL: $(basename $f) — should have been rejected"
    exit 1
  else
    echo "PASS: $(basename $f) — correctly rejected"
  fi
done

echo "All tests passed."
```

## Implementation Notes

### Strictness Policy

The schema uses `additionalProperties: false` on every object where the field set is known and closed. This catches typos and prevents drift between the schema and the specification. If a compiled `.urd.json` contains a field not defined in this schema, it is rejected.

**Exception:** Entity `properties` values are type-constrained to primitives and arrays (string, number, boolean, array, null) but not further refined — the correct *specific* type (e.g., this string must be one of the enum values) depends on the referenced type definition, which is a semantic cross-reference. The schema prevents nested objects in property values; the runtime validates that values match their declared types.

### Schema Size and Organisation

The schema should be a single file (`urd-world-schema.json`). At the expected complexity (8 top-level blocks, ~10 reusable `$defs`, ~25 conditional validation rules), a single file is manageable and avoids the complexity of `$ref` resolution across multiple files.

If the schema exceeds ~1000 lines, it may be split into a main file and a `$defs` file using `$ref` with relative URIs. This is a tooling decision, not a design decision — the schema's logical structure is the same either way.

### Tooling Requirements

The schema must validate correctly with:

- **ajv** (JavaScript/TypeScript) — the reference validator for CI pipelines and the Wyrd runtime. Use ajv v8+ with `--spec=draft2020` (or the `Ajv2020` class in the API). The `ajv-cli` package provides the `ajv validate` command used in the test script.
- **jsonschema** (Python) — for test harness scripts. Use version 4.x+ which supports draft 2020-12 natively.
- Standard draft 2020-12 features only. No vendor-specific extensions. No `$vocabulary` overrides.

### Version Evolution

When the schema specification introduces new capabilities (v2 and beyond), the JSON Schema evolves as follows:

1. New top-level blocks are added as optional properties on the root object.
2. The `urd` enum is extended (e.g., `["1", "2"]`).
3. New effect types are added to the `effect` `oneOf`.
4. New advance modes are added to the `advance` pattern.
5. Backward compatibility: a v1 `.urd.json` must continue to validate against the updated schema.

The `urd` field in the `world` block is the version discriminator. A runtime that only supports v1 checks `urd == "1"` and rejects anything else. The JSON Schema validates structural conformance for the declared version.

**Structural divergence strategy:** If a future version introduces breaking structural changes (fields that are required in v2 but absent in v1, or fields whose type changes), a single flat schema becomes untenable. In that case, the schema should use `if/then/else` keyed on `world.urd` to apply version-specific sub-schemas, or split into separate per-version schema files (`urd-world-schema-v1.json`, `urd-world-schema-v2.json`). For v1, a single schema file is sufficient and correct. This note exists so a future implementer does not assume the single-file approach scales to arbitrary version divergence.

## Acceptance Criteria

The JSON Schema brief is complete when:

1. **Schema file exists.** `urd-world-schema.json` is a valid JSON Schema draft 2020-12 document.
2. **All eight blocks defined.** Every top-level block from the specification has a corresponding schema definition.
3. **All reusable definitions in `$defs`.** Effects, conditions, speech, choices, exits, visibility, property schemas, phases, and select blocks are defined once and referenced.
4. **Positive tests pass.** All positive test fixtures (minimally: Monty Hall, Key Puzzle, Tavern Dialogue, minimal world, conditional visibility, OR conditions, spawn effect) validate successfully.
5. **Negative tests fail.** All 25 negative test cases are correctly rejected with appropriate error messages.
6. **Validation script runs clean.** The `validate-urd-json.sh` script exits 0 with all tests passing.
7. **Cross-checked with specification.** Every `Required` column in the specification tables matches the `required` arrays in the schema. Every type column matches the schema's type constraints. Every enum set matches.
8. **No stored exhaustion.** The schema does NOT define an `exhausted` boolean on dialogue sections. This is the single most important semantic invariant that the structural schema must enforce by omission.

## Relationship to the Compiler and Runtime

The JSON Schema occupies a specific position in the validation chain:

```
Source (.urd.md)
     │
  Compiler (phases 1–5)
     │  ← Structural + semantic validation
     │
  .urd.json
     │
  JSON Schema validation     (this artifact)
     │  ← Structural only
     │
  Runtime load (Wyrd)
     │  ← Structural + semantic validation (redundant for safety)
     │
  Execution
```

The compiler is the primary validator. It performs both structural and semantic checks before emitting `.urd.json`. The JSON Schema is a safety net: it catches compiler bugs that might produce structurally invalid output. It also enables validation *without* the compiler — useful for hand-authored test fixtures, third-party tools generating `.urd.json`, and CI pipelines that validate cached output.

The runtime performs its own load-time validation (checking the `urd` version, resolving references, verifying types). The JSON Schema makes this cheaper by guaranteeing the document's basic structure is sound before the runtime starts its more expensive semantic checks.

## Open Questions

None. All structural decisions are resolved in the normative specifications. The JSON Schema is a mechanical translation of the field tables, type constraints, and structural rules already defined in the World Schema Specification and the Engine Developer Reference Card.

If ambiguity is discovered during implementation (a field whose type or requiredness is unclear from the specification), the resolution is: update the specification first, then update the schema to match. The specification is always upstream of the schema.
