# URD — Type Shorthand Aliases and Range Parsing Fix

*A brief for fixing type keyword recognition and range shorthand parsing in frontmatter*

February 2026 | Backlog

`int(0, 100)` → should work, currently broken

> **Document status: BRIEF** — Fixes two bugs in the frontmatter parser and LINK phase that prevent the shorthand type syntax documented in Schema Markdown from compiling correctly. Also formalises the full set of accepted type aliases.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-19
**Status:** Complete

### What was done

- Added `int(min, max)`, `integer(min, max)`, `num(min, max)`, `number(min, max)` range shorthand parsing in `parse_type_signature()` in `src/parse/frontmatter.rs`
- Added bare alias normalisation in the parser: `int`→`integer`, `num`→`number`, `str`→`string`, `bool`→`boolean`
- Added `"int"`, `"num"`, `"str"` aliases to `parse_property_type()` in `src/link/mod.rs` as belt-and-braces safety
- Added `raw_type_string: String` field to `PropertySymbol` in `src/symbol_table.rs`, populated during LINK collection
- Added URD429 warning for unrecognised type strings in `src/validate/types.rs` with `is_recognised_type()` helper
- Fixed missing `raw_type_string` field in synthetic `PropertySymbol` construction in `src/validate/helpers.rs`
- Created test fixture `tests/fixtures/type-aliases.urd.md` with `int(0, 100)`, `bool`, `num(0.0, 10.0)`, `str`, `enum` types
- Added 10 parse tests: alias normalisation (int, bool, num, str), range shorthand (int, integer, num), range with default, malformed range, empty parens
- Added 4 validate tests: unrecognised type warns, recognised aliases no warning, canonical types no warning, uppercase type warns
- Added 6 e2e tests: compiles successfully, zero errors, canonical output, int range, num range, defaults preserved
- All 433 tests pass (20 new + 413 existing, zero regressions)

### What changed from the brief

- Used URD429 instead of URD418 for unrecognised type warning — URD418 was already taken by range validation in `validate/helpers.rs`
- Parser normalises aliases before they reach the symbol table, so `parse_property_type()` aliases in `link/mod.rs` are belt-and-braces (the brief implied LINK would do the normalisation, but doing it in PARSE is cleaner since it means the AST always contains canonical names)
- Added `raw_type_string: String::new()` to the synthetic `PropertySymbol` in `validate/helpers.rs` for list element validation (not mentioned in the brief, but required to compile after adding the field)

---

## The Problem

The Schema Markdown specification and all documentation examples use shorthand type syntax in frontmatter property definitions:

```
trust: int(0, 100) = 30
~knows_secret: bool = true
```

This syntax does not compile. There are two separate bugs:

### Bug 1: `int` not recognised as a type keyword

In `src/link/mod.rs`, the `parse_property_type()` function maps type strings to `PropertyType` variants:

```rust
pub(crate) fn parse_property_type(s: &str) -> PropertyType {
    match s {
        "bool" | "boolean" => PropertyType::Boolean,
        "integer" => PropertyType::Integer,      // ← "int" missing
        "number" => PropertyType::Number,
        "string" => PropertyType::String,
        "enum" => PropertyType::Enum,
        "ref" => PropertyType::Ref,
        "list" => PropertyType::List,
        _ => PropertyType::String,               // ← "int" falls through here
    }
}
```

`"bool"` is accepted alongside `"boolean"`, but `"int"` is not accepted alongside `"integer"`. Writing `trust: int` silently compiles as `PropertyType::String` because the fallback arm catches unrecognised types.

### Bug 2: `int(min, max)` range shorthand not parsed

In `src/parse/frontmatter.rs`, the `parse_type_signature()` function handles parameterised types:

```rust
fn parse_type_signature(type_str: &str) -> (...) {
    if type_str.starts_with("enum(") && type_str.ends_with(')') { ... }
    if type_str.starts_with("ref(") && type_str.ends_with(')') { ... }
    if type_str.starts_with("list(") && type_str.ends_with(')') { ... }

    // Simple types: string, bool, integer, number
    (type_str.to_string(), None, None, None, None, None, None, None)
}
```

There is no handler for `int(0, 100)` or `integer(0, 100)`. The entire string `"int(0, 100)"` is stored as the raw type name, which then fails to match anything in `parse_property_type()` and falls through to `PropertyType::String`. The `min` and `max` values are never extracted.

### Impact

Every example in the Schema Markdown specification that uses `int(0, 100)` or `bool` silently compiles with incorrect types. The playground starter example in the playground brief also uses this syntax. This is a blocking bug for the playground.

## The Fix

### Part 1: Accept `int` as an alias for `integer`

In `src/link/mod.rs`, add `"int"` to the integer match arm:

```rust
pub(crate) fn parse_property_type(s: &str) -> PropertyType {
    match s {
        "bool" | "boolean" => PropertyType::Boolean,
        "int" | "integer" => PropertyType::Integer,
        "num" | "number" => PropertyType::Number,
        "str" | "string" => PropertyType::String,
        "enum" => PropertyType::Enum,
        "ref" => PropertyType::Ref,
        "list" => PropertyType::List,
        _ => PropertyType::String,
    }
}
```

Note: this also adds `"num"` for `number` and `"str"` for `string`. The principle is: **every type that has a natural abbreviation should accept it**.

**Important:** The `raw_type_string` field on `PropertySymbol` (see Part 3) must be populated from the *pre-normalisation* source string — i.e., the value returned by `parse_type_signature()` in `property_type` position, which is already the canonical name. The raw string must come from the `PropertyDef.property_type` field set during PARSE, before LINK's `parse_property_type()` runs. This is the string that URD418 checks against the recognised set. Writers coming from Python, Rust, C, or TypeScript will instinctively reach for `int`, `str`, `bool`, or `num`. Rejecting these silently (by falling through to `String`) is the worst possible outcome — it compiles but produces wrong types.

The full alias table:

| Canonical (JSON output) | Accepted in Schema Markdown |
|------------------------|-----------------------------|
| `boolean` | `bool`, `boolean` |
| `integer` | `int`, `integer` |
| `number` | `num`, `number` |
| `string` | `str`, `string` |
| `enum` | `enum` |
| `ref` | `ref` |
| `list` | `list` |

The compiled JSON always uses the canonical name. The aliases are a parser convenience — they are normalised during LINK and never appear in output.

### Part 2: Parse `int(min, max)` and `integer(min, max)` range shorthand

In `src/parse/frontmatter.rs`, add handlers for integer and number range shorthand in `parse_type_signature()`:

```rust
// Integer with range: int(0, 100) or integer(0, 100)
if (type_str.starts_with("int(") || type_str.starts_with("integer(")) && type_str.ends_with(')') {
    let paren_start = type_str.find('(').unwrap();
    let inner = &type_str[paren_start + 1..type_str.len() - 1];
    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let min = parts[0].parse::<f64>().ok();
        let max = parts[1].parse::<f64>().ok();
        return ("integer".to_string(), None, None, None, None, None, min, max);
    }
    // Single-arg form: int(100) means max only? No — require both or neither.
    // Fall through to bare type name.
    return ("integer".to_string(), None, None, None, None, None, None, None);
}

// Number with range: num(0.0, 1.0) or number(0.0, 1.0)
if (type_str.starts_with("num(") || type_str.starts_with("number(")) && type_str.ends_with(')') {
    let paren_start = type_str.find('(').unwrap();
    let inner = &type_str[paren_start + 1..type_str.len() - 1];
    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    if parts.len() == 2 {
        let min = parts[0].parse::<f64>().ok();
        let max = parts[1].parse::<f64>().ok();
        return ("number".to_string(), None, None, None, None, None, min, max);
    }
    return ("number".to_string(), None, None, None, None, None, None, None);
}
```

**Normalisation rule:** The parser always stores the canonical type name. `int(0, 100)` produces `property_type: "integer"`, `min: 0`, `max: 100`. The alias never reaches the symbol table or JSON output.

**Range shorthand applies only to `int`/`integer` and `num`/`number`.** It does not apply to `boolean`, `string`, `enum`, `ref`, or `list`. Forms like `str(10, 20)` or `bool(0, 1)` are not valid — these should fall through to the bare type name handler and trigger URD418 if the full string (including parentheses) doesn't match a recognised type.

**Case sensitivity.** Schema Markdown is case-sensitive by design. `Int`, `INT`, `Bool` are not recognised — only lowercase aliases. The grammar enforces lowercase identifiers throughout (`Identifier ← [a-z][a-z0-9_]*`). No case normalisation is needed or desired.

### Part 3: Handle the fallback arm

The current `_ => PropertyType::String` fallback silently accepts any unrecognised type string. This is dangerous — a typo like `integr` compiles as a string property with no warning.

Add a diagnostic for unrecognised type names:

```rust
_ => {
    // Unrecognised type names fall through to String, but this
    // should emit a warning so typos don't compile silently.
    // The diagnostic is emitted during VALIDATE (URD418), not here,
    // because LINK doesn't have access to the diagnostic collector
    // at the point of type parsing.
    PropertyType::String
}
```

A new validation rule (URD418) in `src/validate/types.rs` should check whether any property's raw type string (before normalisation) is not in the recognised set and emit a warning.

To make this work, the raw type string from the source must survive into the symbol table. Currently `parse_property_type()` discards it. The fix: add a `raw_type_string: String` field to `PropertySymbol` in `symbol_table.rs`, populated during LINK collection from the `PropertyDef.property_type` string. VALIDATE then checks this field against the recognised set.

The recognised set is: `bool`, `boolean`, `int`, `integer`, `num`, `number`, `str`, `string`, `enum`, `ref`, `list`. Anything else triggers URD418:

```
warning[URD418]: Unrecognised property type 'integr' on property 'trust' of type 'Barkeep'. 
  Valid types: bool, int, num, str, enum, ref, list (and their long forms).
  Treating as 'string'.
```

This is a **warning**, not an error — it does not prevent compilation. The property is treated as `string` (the existing fallback behaviour), but the author is told something looks wrong. This catches typos without breaking existing files that might use unconventional type names for some reason.

## Existing Tests to Update

The existing test suite likely has tests that use the shorthand syntax and expect it to work (since the spec examples use it). After the fix, verify:

- `int(0, 100)` parses to `property_type: "integer"`, `min: 0.0`, `max: 100.0`.
- `integer(0, 100)` produces identical output.
- `int` alone (no range) parses to `property_type: "integer"`, `min: None`, `max: None`.
- `bool` parses to `property_type: "boolean"`.
- `num(0.0, 1.0)` parses to `property_type: "number"`, `min: 0.0`, `max: 1.0`.
- `str` parses to `property_type: "string"`.
- The compiled JSON always uses canonical names (`"integer"`, `"boolean"`, `"number"`, `"string"`), never aliases.
- The Tavern Scene integration test compiles without errors (it uses `int(0, 100)` and `bool`).

## New Tests to Add

| Test | Input | Expected |
|------|-------|----------|
| Alias recognition | `trust: int` | `property_type: "integer"` |
| Alias recognition | `flag: bool` | `property_type: "boolean"` |
| Alias recognition | `weight: num` | `property_type: "number"` |
| Alias recognition | `label: str` | `property_type: "string"` |
| Range shorthand | `trust: int(0, 100)` | `property_type: "integer"`, `min: 0`, `max: 100` |
| Range shorthand | `trust: integer(0, 100)` | Same as above |
| Range shorthand | `weight: num(0.0, 10.0)` | `property_type: "number"`, `min: 0.0`, `max: 10.0` |
| Range shorthand | `weight: number(0.0, 10.0)` | Same as above |
| Canonical output | `trust: int(0, 100) = 30` | JSON emits `"type": "integer"` (never `"int"`) |
| Default with range | `trust: int(0, 100) = 30` | `default: 30`, `min: 0`, `max: 100` |
| Bare alias | `trust: int = 50` | `property_type: "integer"`, `default: 50`, no min/max |
| Malformed range | `trust: int(abc, def)` | `property_type: "integer"`, `min: None`, `max: None` (graceful fallback) |
| Empty parens | `trust: int()` | `property_type: "integer"`, no min/max |
| Unrecognised type | `mood: integr` | `property_type: "string"`, URD418 warning emitted |
| Unrecognised type | `mood: floot` | `property_type: "string"`, URD418 warning emitted |
| Recognised alias no warning | `trust: int` | No URD418 warning |
| Range on wrong type | `name: str(10, 20)` | URD418 warning (unrecognised type `str(10, 20)`) |
| Uppercase rejected | `trust: Int` | URD418 warning (case-sensitive, `Int` not recognised) |

## Files Changed

| File | Change |
|------|--------|
| `src/link/mod.rs` | Add `"int"`, `"num"`, `"str"` aliases to `parse_property_type()`. |
| `src/parse/frontmatter.rs` | Add `int(min, max)`, `integer(min, max)`, `num(min, max)`, `number(min, max)` handlers to `parse_type_signature()`. |
| `tests/` | New alias and range shorthand tests. Update any existing tests that expected the broken behaviour. |
| `src/validate/types.rs` | Add URD418 warning for unrecognised raw type strings. |
| `src/symbol_table.rs` | Add `raw_type_string: String` field to `PropertySymbol`. |

## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| T1 | `int` accepted as alias for `integer` in property definitions. | Test: `trust: int` compiles to `PropertyType::Integer`. |
| T2 | `int(0, 100)` extracts min/max range values. | Test: compiled output has `"min": 0, "max": 100`. |
| T3 | `bool`, `num`, `str` accepted as aliases. | Test: each alias compiles to the correct `PropertyType`. |
| T4 | Compiled JSON always uses canonical type names. | Test: `int` in source → `"integer"` in JSON output. |
| T5 | Tavern Scene integration test passes. | `cargo test` — the tavern example uses `int(0, 100)` and `bool`. |
| T6 | All 390+ existing tests still pass. | `cargo test` — no regressions. |
| T7 | `number(min, max)` range shorthand works. | Test: `weight: num(0.0, 10.0)` compiles with correct range. |
| T8 | Unrecognised type string emits URD418 warning. | Test: `mood: integr` compiles as `string` with warning. |
| T9 | Recognised aliases do not emit URD418. | Test: `trust: int` compiles cleanly with no warnings. |

## Spec Alignment

The Schema Markdown specification already uses these shorthands in its examples. The Schema Specification (JSON contract) defines the canonical type names as `"boolean"`, `"integer"`, `"number"`, `"string"`, `"enum"`, `"ref"`, `"list"`. The JSON schema (`urd-world-schema.json`) validates against the canonical names.

This fix aligns the compiler with the spec. No spec changes are needed — the aliases are a compiler convenience that normalise to the canonical forms already defined.

The formal grammar brief should be updated to document the accepted aliases in the frontmatter property type rule. This is a documentation update, not a grammar change — the PEG grammar's `PropertyType` nonterminal should enumerate both canonical names and aliases.
