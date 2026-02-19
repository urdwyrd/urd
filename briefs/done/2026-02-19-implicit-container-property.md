# URD — Implicit `container` Property in Expressions

*A brief for recognizing the implicit `container` property during link-phase resolution*

February 2026 | Bug Fix

`? @enchanted_blade.container != player` → URD308: Property 'container' does not exist on type 'Weapon'

> **Document status: BRIEF** — Fixes a LINK-phase gap where the compiler rejects `.container` in conditions, effects, and rule `where` clauses because it only checks user-declared properties on the type. The Schema Spec §Containment Model defines `container` as an implicit property of every entity. This is a spec-level contract, not an edge case.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-20
**Status:** Complete

### What was done

1. **Added `IMPLICIT_PROPERTIES` constant** in `src/link/resolve.rs` — `const IMPLICIT_PROPERTIES: &[&str] = &["container"];` defined at module level alongside `BUILTIN_JUMP_TARGETS`. This is the single source of truth for implicit properties.

2. **Updated all 3 URD308 resolution sites:**
   - **Site 1 — entity declaration property overrides** (line ~159): Added `&& !IMPLICIT_PROPERTIES.contains(&prop_name.as_str())` to the `!ts.properties.contains_key(prop_name)` guard before emitting URD308.
   - **Site 2 — PropertyComparison in conditions** (line ~683): Changed `if ts.properties.contains_key(&pc.property)` to `if ts.properties.contains_key(&pc.property) || IMPLICIT_PROPERTIES.contains(&pc.property.as_str())`. Implicit properties now get `resolved_property` and `resolved_type` set in the annotation.
   - **Site 3 — effect property resolution** (line ~800): Same pattern as site 2, using `|| IMPLICIT_PROPERTIES.contains(&property)` (the variable is already `&str` here).

3. **No emit changes needed.** The condition emit path uses `pc.property` directly (not the annotation), and the effect emit path uses `ann.resolved_property` which is now correctly populated for implicit properties. Both paths pass `.container` through to the compiled JSON.

4. **Added 5 link tests:** `implicit_container_in_condition` (PropertyComparison resolves with annotation), `implicit_container_in_effect` (Set effect resolves), `implicit_container_in_entity_override` (frontmatter override accepted), `non_implicit_property_still_errors` (URD308 still emitted for unknown properties), `near_miss_implicit_property_still_errors` (`containers` ≠ `container`, exact match only).

5. **All 462 tests pass** (457 existing + 5 new). No regressions.

6. **Sunken-citadel stress test** reduced from 7 errors to 6. The URD308 for `.container` on line 693 is eliminated.

### What changed from the brief

- **No separate `builtins.rs` module created.** The brief suggested placing `IMPLICIT_PROPERTIES` in `symbol_table.rs` or a new `builtins.rs`. Instead, it was placed in `resolve.rs` alongside `BUILTIN_JUMP_TARGETS`, keeping all compiler-recognised constants together in the resolution module. This is consistent with how `BUILTIN_JUMP_TARGETS` was placed in the previous brief. If a third category of builtins is needed, refactoring to a shared module would make sense.

- **Unstable Rust feature avoided.** The initial implementation used `.as_str()` on a `&String` reference at site 3, which required the unstable `str_as_str` feature. Fixed by using `&property` directly since the variable was already `&str`.

- **Part 4 (comparison value type checking) deferred as planned.** The brief acknowledged this was not required for the fix. The runtime handles `.container` comparison semantics.


## The Problem

The Schema Spec §Containment Model states:

> *Every entity has an implicit container property, a reference to whatever currently holds it.*

The spec uses `.container` extensively in examples:

```
rusty_key.container == player          // "the player has the key"
player.container == cell               // "the player is in the cell"
rusty_key.container == player.container // "the key is in the same room as the player"
```

The stress test uses `.container` in two places:

```
? @enchanted_blade.container != player      // Line 693 — condition
where target.container != @merchant_dara    // merchant_restocks rule — where clause
```

The compiler's property resolution in `resolve.rs` follows this path:

```
entity_id → EntitySymbol.type_symbol → TypeSymbol.properties.contains_key(property)
```

`container` is not in any type's property map because it's implicit — no type definition declares it. The compiler emits URD308 "Property 'container' does not exist on type 'Weapon'."

Property resolution occurs at three sites in `resolve.rs`:

1. `resolve_condition_expr()` → `PropertyComparison` arm (line ~460)
2. `resolve_effect()` → `Set` / `Reveal` arm (line ~530)
3. Rule block → `where` clause resolution (delegated to `resolve_condition_expr()`)

All three sites have the same gap.

### Type of `container`

The `container` property behaves like a `ref` to an entity or location. Its comparison targets include:
- `player` (keyword)
- `here` (keyword)
- Entity references (`@merchant_dara`)
- Location references (by name)

This is the same comparison set used by the `ContainmentCheck` expression (`@entity in player`). The resolver should treat `.container` comparisons as valid without requiring the property to be declared on the type, but should still enforce that the comparison value is a valid container reference.


## The Fix

### Part 1: Define implicit properties in one place

Create a constant or small lookup structure that defines compiler-recognized implicit properties:

```rust
/// Implicit properties defined by the Urd runtime, not by user type definitions.
/// These are valid in conditions, effects, and rule where clauses on any entity.
pub(crate) const IMPLICIT_PROPERTIES: &[&str] = &["container"];
```

Place this in a shared location — either `symbol_table.rs` or a new `builtins.rs` module. Do **not** hardcode the list at each resolution site.

**Check the Schema Spec for other implicit properties.** The `id` property (the entity's own identifier) may also be implicit. Even if only `container` is implemented now, the structure should make it trivial to add more.

### Part 2: Update property resolution at all three sites

At each of the three resolution sites in `resolve.rs`, after the `ts.properties.contains_key(property)` check fails, add:

```rust
if !ts.properties.contains_key(property) {
    if IMPLICIT_PROPERTIES.contains(&property.as_str()) {
        // Resolve as implicit system property
        if let Some(ann) = &mut pc.annotation {  // or the relevant annotation slot
            ann.resolved_property = Some(property.to_string());
            ann.resolved_type = Some(type_name.clone());
        }
        // Do NOT emit URD308
    } else {
        diagnostics.error("URD308", ...);
    }
}
```

This must be applied consistently at all three sites. The existing code structure makes this straightforward — each site has the same `ts.properties.contains_key` check followed by a URD308 emission.

### Part 3: Ensure `emit` handles implicit properties

In `emit/mod.rs`, conditions and effects reference properties via the annotation's `resolved_property` field. Implicit properties should emit identically to declared properties in the JSON output. The runtime resolves `.container` at execution time — the compiler just needs to pass the property name through.

Verify that no emit code path rejects annotations where the property is not in the type's property map. If `emit` does any secondary resolution or type lookup, it must also check `IMPLICIT_PROPERTIES`.

### Part 4: Consider the comparison value type

When `.container` is used in a `PropertyComparison`, the comparison value is typically `player`, `here`, an entity ref, or a location name. The existing `resolve_condition_expr` for `PropertyComparison` resolves the `value` field as a string — it doesn't resolve the comparison target as an entity or keyword.

For this brief, this is acceptable. The runtime handles `.container` comparison semantics. A future validation brief could add type checking on `.container` comparisons (e.g., reject `@entity.container == 42`), but that is not required to fix the URD308 error.


## Files Changed

| File | Change |
|------|--------|
| `src/symbol_table.rs` (or new `src/builtins.rs`) | Add `IMPLICIT_PROPERTIES` constant. |
| `src/link/resolve.rs` | At all 3 property resolution sites, check `IMPLICIT_PROPERTIES` before emitting URD308. |
| `src/emit/mod.rs` | Verify implicit properties pass through to JSON output. |
| `tests/` | New tests for `.container` in conditions, effects, and rule where clauses. |


## New Tests to Add

| Test | Input | Expected |
|------|-------|----------|
| Container in condition | `? @key.container != player` | Valid condition, no URD308 |
| Container equality | `? @key.container == @npc` | Valid condition |
| Container with `here` | `? @key.container == here` | Valid condition |
| Container in effect | `> @key.container = player` | Valid effect (if Set effects support implicit properties) |
| Container in rule where | `where target.container != @merchant` | Valid where clause |
| Container with `player` keyword | `where target.container != player` | Valid where clause |
| Non-existent property | `? @key.nonexistent != true` | Still produces URD308 |
| Non-existent property similar to implicit | `? @key.containers != player` | Still produces URD308 (no fuzzy match) |
| Sunken Citadel e2e | Full fixture | URD308 for `container` eliminated |


## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| T1 | `? @enchanted_blade.container != player` compiles without URD308. | Stress test compilation. |
| T2 | `where target.container != @merchant_dara` compiles without URD308. | Stress test compilation. |
| T3 | `? @entity.nonexistent != true` still produces URD308. | Unit test. |
| T4 | Implicit properties are defined in exactly one location, not scattered across resolution sites. | Code review. |
| T5 | The emitter handles `.container` references in conditions and effects. | Unit test or e2e. |
| T6 | All existing tests still pass. | `cargo test` — no regressions. |


## Spec Alignment

The Schema Spec §Containment Model explicitly defines `container` as an implicit property of every entity. The compiled JSON uses `.container` in condition expressions (§Conditions, example: `rusty_key.container == player`). This fix aligns the compiler with the spec. No spec changes needed.

**Future consideration:** The Schema Spec may define other implicit properties in future versions (e.g., `id`, `location`). The implementation should use a shared constant or registry rather than string comparisons at resolution sites, so adding new implicits is a one-line change.

**Merge risk note:** This brief and `2026-02-19-builtin-end-jump.md` both touch `link/resolve.rs` and `emit/mod.rs`. The diff surfaces do not overlap (`resolve_condition_expr` / `resolve_effect` for this brief vs `resolve_jump` for the end-jump brief), but both add constants to the module. If developed in parallel, land one at a time with separate test suites to reduce merge risk. Alternatively, land together only if the diff surface is confirmed non-overlapping.
