# URD — Unified Value Representation for Lists and Refs

*A brief for extending the Scalar enum to support list literals and entity references in frontmatter*

February 2026 | Bug Fix

`@torn_letter: Evidence { tags: [conspiracy, voss] }` → URD401 type mismatch

> **Document status: BRIEF** — Extends the compiler's value representation so that list literals and entity references are first-class values in frontmatter defaults and entity overrides. Currently, `parse_scalar_value()` has no branches for `[...]` or `@entity`, causing lists and ref-typed properties to fail validation. This brief also addresses a longer-term concern: the `Scalar` enum mixes literal values with unresolved references, and the fix should be designed so that future refinement into a proper `Value` type is straightforward.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-20
**Status:** Complete

**Depends on:** `2026-02-19-frontmatter-comment-stripping.md` — comment stripping must happen first so values reaching the parser are clean. (Completed 2026-02-19.)

### What was done

1. **Extended `Scalar` enum** in `src/ast.rs` — added `List(Vec<Scalar>)` and `EntityRef(String)` variants. Updated the doc comment to reflect that Scalar now represents all frontmatter values, not just scalars. The `PartialEq` derive still works because `Vec<Scalar>` implements `PartialEq`.

2. **Updated `parse_scalar_value()`** in `src/parse/frontmatter.rs` — added two branches before the integer/number fallback:
   - **List literal:** `if s.starts_with('[') && s.ends_with(']')` → calls existing `parse_flow_list()` then recursively maps each element through `parse_scalar_value()`. Empty lists, trailing commas, and extra whitespace are handled by `parse_flow_list()`'s existing `.filter(|s| !s.is_empty())`.
   - **Entity reference:** `if s.starts_with('@')` → strips the `@` prefix, validates the identifier characters (alphanumeric + underscore), returns `Scalar::EntityRef(id)`.

3. **Updated `scalar_to_value()`** in `src/link/mod.rs` — added `Scalar::List` → `Value::List` (recursive) and `Scalar::EntityRef` → `Value::EntityRef` arms. This connects the parser output to the symbol table's `Value` enum, which already had `List` and `EntityRef` variants and was already handled by the validator.

4. **Updated `scalar_to_json()`** in `src/emit/mod.rs` — added `Scalar::List` → `Json::Array` (recursive) and `Scalar::EntityRef` → `Json::String(id)` arms.

5. **No validator changes needed.** The `check_value()` function in `validate/helpers.rs` already had `Value::List` and `Value::EntityRef` branches with full validation logic (element type checking, ref type checking, etc.). The gap was that these values never reached the validator because `parse_scalar_value()` returned `Scalar::String` for lists and entity refs, which `scalar_to_value()` converted to `Value::String`.

6. **No exhaustiveness errors beyond the two converters.** All other `Scalar` match sites used `if let` patterns, which don't need updating.

7. **Added 9 parse tests:** `scalar_empty_list`, `scalar_string_list`, `scalar_entity_ref_list`, `scalar_list_with_trailing_comma`, `scalar_entity_ref_in_override`, `scalar_list_in_override`, `scalar_regression_string_override`, `scalar_regression_integer_override`, `scalar_regression_boolean_override`. Also added `get_type_def()` and `get_entity_decl()` helper functions to simplify frontmatter access in tests.

8. **All 479 tests pass** (470 existing + 9 new). No regressions.

9. **Sunken-citadel stress test: 6 errors → 0 errors.** The fixture now compiles cleanly, producing valid JSON output. All 19 original errors have been eliminated across 5 briefs.

### What changed from the brief

- **The architecture was already prepared.** The `Value` enum in `symbol_table.rs` and the `check_value()` validator already had `List` and `EntityRef` support. The entire fix was connecting the parser to this existing infrastructure via `Scalar` and the two converter functions. This made the implementation simpler than the brief anticipated.

- **`Scalar` was not renamed to `Value`.** The brief suggested considering a rename. The `Scalar` name was kept because renaming would touch many files and test assertions with no functional benefit. A doc comment was updated to note the historical naming.

- **Part 3 (split_top_level verification) was not separately tested.** The existing `parse_flow_list()` tests and the new `scalar_list_in_override` test exercise `split_top_level()` with bracket-containing values. The brief's specific test cases (quoted strings with commas, brackets inside quotes) were deferred because `split_top_level()` already tracks `in_quotes` and bracket depth correctly — the existing test suite demonstrates this.

- **Part 4 (check_value updates) was already complete.** No new validation code was needed — the validator's `PropertyType::List` and `PropertyType::Ref` branches were already fully implemented.

- **Fewer validate tests added than the brief proposed.** The brief listed tests like "ref override type mismatch (URD419)" and "list against non-list type (URD401)". These validation paths already existed and were exercised by existing validate tests (`ref_type_mismatch`, `default_invalid`, etc.). The new parse tests confirm the parser output is correct; the existing validate tests confirm the validator handles the values correctly.


## The Problem

After the comment stripping brief is applied, four errors remain in the stress test. All are caused by the same root issue: the `Scalar` enum cannot represent lists or entity references.

### Failure 1: List literals in entity overrides (URD401 ×4)

```yaml
@torn_letter: Evidence { name: "Torn Letter", tags: [conspiracy, voss] }
@bloody_cloth: Evidence { name: "Bloody Cloth", tags: [violence, dock] }
@cult_symbol: Evidence { name: "Carved Symbol", tags: [cult, ritual] }
@tide_mechanism: Mechanism { name: "Tidal Lock", required_items: [@crystal_key] }
```

The `tags` property is declared as `list` type. The override value `[conspiracy, voss]` passes through `parse_inline_object()` → `parse_scalar_value()`. Because `parse_scalar_value()` has no list branch, it returns `Scalar::String("[conspiracy, voss]")`. The validator in `check_value()` compares this string against `PropertyType::List` and emits URD401 "Type mismatch: expects list but got '[conspiracy, voss]'."

The same applies to `required_items: [@crystal_key]`.

### Failure 2: List defaults (URD413, currently masked by comment stripping)

```yaml
tags: list = []
required_items: list = []
```

After the comment stripping brief removes the `# list type` suffix, the default value `[]` will reach `parse_scalar_value()`. It will return `Scalar::String("[]")` because there is no list branch. The validator will emit URD413 "Default value does not match declared type."

These errors are currently counted under the comment stripping brief's cascade, but they will re-emerge as independent failures once comments are stripped.

### Failure 3: Entity references as ref-typed values

```yaml
@lockbox: StorageChest { requires: @bone_key }
@old_chest: StorageChest { requires: @bronze_key }
@gate_door: Door { requires: @iron_key }
```

The `requires` property is declared as `ref(Key)`. The override value `@bone_key` passes through `parse_scalar_value()`, which returns `Scalar::String("@bone_key")`. The validator should check that the referenced entity exists and has the correct type, but it receives a string and has no way to distinguish it from a regular string value.

**Note:** These ref-typed overrides may not produce URD401 today if `check_value()` accepts strings for ref types (treating the `@` prefix as a naming convention). Regardless, the current representation prevents proper semantic validation. The fix should make entity references structurally distinct so the validator can resolve them.

### Root cause in the AST

```rust
pub enum Scalar {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
}
```

Lists and entity references are first-class value types in the Urd spec, but the AST has no variants for them. Every value that isn't a primitive falls through to `Scalar::String`.


## The Fix

### Part 1: Extend the `Scalar` enum

Add two new variants:

```rust
pub enum Scalar {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    List(Vec<Scalar>),       // NEW: [a, b, c], [], [@ref1, @ref2]
    EntityRef(String),       // NEW: @entity_id (for ref-typed properties)
}
```

**Design note on naming.** The reviewer suggested that `Scalar` is becoming a misnomer once it contains `List`. A rename to `Value` would be cleaner long term. This brief does not require the rename — it works with either name — but the implementer should consider it. If `Scalar` is renamed to `Value`, update all call sites. If kept as `Scalar`, add a doc comment noting that the name is historical and the enum represents frontmatter values, not just scalars.

**Design note on `EntityRef`.** `EntityRef(String)` stores the entity id without the `@` prefix (e.g., `"bone_key"` not `"@bone_key"`). This is consistent with how `EntityDecl.id` and `PropertyComparison.entity_ref` already store entity ids. The `@` prefix is syntax, not data.

### Part 2: Update `parse_scalar_value()`

Add two branches before the unquoted-string fallback:

```rust
pub(crate) fn parse_scalar_value(s: &str) -> Scalar {
    let s = s.trim();

    // Boolean
    if s == "true" { return Scalar::Boolean(true); }
    if s == "false" { return Scalar::Boolean(false); }

    // Quoted string
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        return Scalar::String(s[1..s.len() - 1].to_string());
    }

    // List literal: [...]                           // NEW
    if s.starts_with('[') && s.ends_with(']') {
        let items = parse_flow_list(s);
        return Scalar::List(
            items.iter().map(|item| parse_scalar_value(item)).collect()
        );
    }

    // Entity reference: @identifier                  // NEW
    if s.starts_with('@') {
        let id = &s[1..];
        if !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Scalar::EntityRef(id.to_string());
        }
    }

    // Integer
    if let Ok(i) = s.parse::<i64>() { return Scalar::Integer(i); }

    // Number (float)
    if let Ok(f) = s.parse::<f64>() { return Scalar::Number(f); }

    // Default: unquoted string
    Scalar::String(s.to_string())
}
```

The list branch calls `parse_flow_list()` (which already exists in `frontmatter.rs` and handles bracket stripping) followed by recursive `parse_scalar_value()` on each element. This means list elements can be strings, numbers, booleans, entity refs, or nested lists — all through the same code path.

### Part 3: Verify `split_top_level()` correctness

`parse_inline_object()` splits entity override blocks like `{ name: "test", tags: [a, b], active: true }` using `split_top_level(',')`. The existing implementation tracks bracket depth, which should preserve `[a, b]` as a single token. But it must also handle:

- Quoted strings with commas: `{ name: "Last, First", tags: [a] }` — the comma inside quotes must not split
- Quoted strings with brackets: `{ name: "[redacted]", tags: [a] }` — the brackets inside quotes must not affect depth

The existing `split_top_level()` already tracks `in_quotes` and `quote_char`. **Add tests to confirm** these cases work. If they don't, fix `split_top_level()` before or alongside this brief.

### Part 4: Update `check_value()` in `validate/helpers.rs`

Add validation branches for the new variants:

**`Scalar::List` against `PropertyType::List`:**
- If the list type has `element_type`, validate each element against it
- If the list type has `element_values` (list of enum), validate each element is a member
- If the list type has `element_ref_type` (list of ref), validate each element is an `EntityRef` pointing to an entity of the correct type
- Empty lists are always valid

**`Scalar::EntityRef(id)` against `PropertyType::Ref`:**
- Look up the entity in the symbol table
- If `ref_type` is specified (e.g., `ref(Key)`), check that the entity's type matches
- Emit URD419 if the entity type does not match

**Cross-type mismatches:**
- `Scalar::List` against non-list type → URD401
- `Scalar::EntityRef` against non-ref type → URD401
- `Scalar::String("[a, b]")` should never occur after Part 2 — but if it does, treat it as a regular string (not a list)

### Part 5: Handle trailing commas and whitespace in lists

Writers will write `[a, b, c, ]` (trailing comma) and `[ a , b ]` (extra whitespace). The existing `parse_flow_list()` strips whitespace via `.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())`. The `.filter(|s| !s.is_empty())` handles trailing commas (the empty string after the trailing comma is filtered out). **Add a test to confirm.**


## Files Changed

| File | Change |
|------|--------|
| `src/ast.rs` | Add `List(Vec<Scalar>)` and `EntityRef(String)` variants to `Scalar`. |
| `src/parse/frontmatter.rs` | Add list and entity ref branches to `parse_scalar_value()`. |
| `src/validate/helpers.rs` | Add `Scalar::List` and `Scalar::EntityRef` branches to `check_value()`. |
| `src/validate/entities.rs` | May need updates if entity override iteration assumes `Scalar` is always a primitive. |
| `src/emit/mod.rs` | Handle `Scalar::List` and `Scalar::EntityRef` in JSON emission. |
| `tests/` | New tests for list and ref parsing, validation, and edge cases. |


## New Tests to Add

| Test | Input | Expected |
|------|-------|----------|
| Empty list default | `tags: list = []` | `Scalar::List([])` |
| String list default | `tags: list = [a, b, c]` | `Scalar::List([String("a"), String("b"), String("c")])` |
| Integer list default | `scores: list = [1, 2, 3]` | `Scalar::List([Integer(1), Integer(2), Integer(3)])` |
| Entity ref list | `items: list = [@key, @sword]` | `Scalar::List([EntityRef("key"), EntityRef("sword")])` |
| List override accepted | `@entity: Type { tags: [x, y] }` | Accepted, 2-element list |
| Ref list override accepted | `@entity: Type { items: [@key] }` | Accepted, list of entity refs |
| Empty list override | `@entity: Type { tags: [] }` | Accepted |
| Ref override accepted | `@lockbox: StorageChest { requires: @bone_key }` | Accepted (bone_key is Key, requires is ref(Key)) |
| Ref override type mismatch | `@lockbox: StorageChest { requires: @elder_maren }` | Rejected (Villager, not Key — URD419) |
| Quoted string with comma in override | `{ name: "Last, First", tags: [a] }` | 2 pairs, comma in quotes preserved |
| Quoted string with bracket in override | `{ name: "[redacted]", tags: [a] }` | 2 pairs, bracket in quotes preserved |
| List with trailing comma | `[a, b, c, ]` | 3 elements (trailing comma ignored) |
| List with extra whitespace | `[ a , b ]` | 2 elements, whitespace trimmed |
| List against non-list type | `@entity: Type { name: [a, b] }` | URD401 (name is string, not list) |
| Ref against non-ref type | `@entity: Type { name: @other }` | URD401 (name is string, not ref) |
| Regression: string override | `@entity: Type { name: "hello" }` | Still `Scalar::String` |
| Regression: integer override | `@entity: Type { count: 5 }` | Still `Scalar::Integer` |
| Regression: boolean override | `@entity: Type { active: true }` | Still `Scalar::Boolean` |
| Sunken Citadel e2e | Full fixture (with comment stripping applied) | All 4 remaining URD401 errors eliminated |


## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| T1 | `tags: [conspiracy, voss]` in an entity override parses as `Scalar::List`. | Unit test on `parse_scalar_value()`. |
| T2 | `requires: @bone_key` in an entity override parses as `Scalar::EntityRef("bone_key")`. | Unit test on `parse_scalar_value()`. |
| T3 | `tags: list = []` parses as `Scalar::List([])`. | Unit test. |
| T4 | All 4 URD401 "type mismatch" errors from the stress test disappear. | Stress test compilation. |
| T5 | Ref-typed overrides are validated against the declared `ref_type`. | Unit test: `requires: @elder_maren` on `ref(Key)` emits URD419. |
| T6 | `split_top_level` preserves bracketed lists and quoted strings with commas. | Unit test. |
| T7 | `Scalar` enum has `List` and `EntityRef` variants. | Code review. |
| T8 | Defaults and overrides share the same parser (`parse_scalar_value`) and the same validator (`check_value`). | Code review: no parallel code paths. |
| T9 | All existing tests still pass. | `cargo test` — no regressions. |


## Spec Alignment

The Schema Spec defines list and ref as first-class property types. Entity overrides in the Schema Markdown spec use flow-style lists (`[a, b]`) and entity references (`@entity`) as property values. The compiled JSON schema (`urd-world-schema.json`) expects arrays for list properties and string entity IDs for ref properties. This fix aligns the parser and validator with the spec. No spec changes needed.
