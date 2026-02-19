# URD — Reserved Property References in Narrative Conditions

*A brief for fixing `target.prop` and `player.prop` parsing in narrative-scope condition expressions*

February 2026 | Bug Fix

`? target.state == closed` → should work, currently URD112

> **Document status: BRIEF** — Fixes a parser gap where the `ReservedPropRef` grammar rule (`target.prop`, `player.prop`) is not implemented in the narrative-scope condition expression parser, causing valid choice guards to emit spurious URD112 errors.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-19
**Status:** Complete

### What was done

1. **Extracted `parse_prop_comparison()` helper** in `src/parse/content.rs` — shared function that takes an entity/binding name and the text after the dot, scans for comparison operators (`==`, `!=`, `>=`, `<=`, `>`, `<`), and returns a `PropertyComparison` node.

2. **Refactored the `@entity.prop` path** in `parse_condition_expr()` to call the new helper instead of inline operator scanning. Pure refactor, identical behaviour.

3. **Added `target.`/`player.` branch** in `parse_condition_expr()` before the final `None` return. Produces `PropertyComparison` nodes with `entity_ref: "target"` or `entity_ref: "player"` (no `@` prefix).

4. **Added LINK phase bypass** in `resolve_condition_expr()` in `src/link/resolve.rs`. When `entity_ref` is `"target"` or `"player"`, the resolver sets the annotation with `resolved_entity` matching the binding name and returns early, skipping entity table lookup. This prevents false URD301 errors. The VALIDATE phase already skips unresolved properties (checks `resolved_property.is_none()`), so reserved bindings pass through correctly.

5. **Added 8 new parse tests:** `target_condition_equality`, `target_condition_boolean`, `player_condition_greater_than`, `player_condition_not_equal`, `non_reserved_bare_identifier_rejected`, `target_condition_in_choice_guard`, `target_condition_in_rule_where_clause` (T3), `target_effect_parses_as_set` (Part 5 confirmation).

6. **All 441 tests pass** (433 existing + 8 new). No regressions.

7. **Version bumped to 0.1.2** with changelog entry.

### What changed from the brief

- **Monty Hall fixture was already simplified.** The brief referenced a version with `target.state == closed` and `target.chosen == false` choice guards on lines 42/43/52/53. The actual fixture uses `@door_1.revealed == false` (EntityProp with `@` prefix) and has no `target.prop` conditions. The fix is correct but the Monty Hall e2e test (T6) exercises entity-targeted choices without `target.prop` guards. The 6 new unit tests cover `target.prop` parsing directly.

- **LINK phase needed modification.** The brief focused on the PARSE phase fix but did not explicitly call out that the LINK resolver's `resolve_condition_expr()` would emit URD301 for `entity_ref: "target"`. Added a reserved binding bypass in LINK to prevent cascading errors. This was necessary for correctness.

- **Part 4 (RulePropRef) confirmed as existing technical debt.** The Monty Hall fixture's `where door.prize == goat` still silently fails to parse (returns `None`). No existing test asserts on this, so no regression. Deferred as documented.

- **Part 5 (effects) confirmed working for PARSE.** `parse_effect_type()` handles `target.prop = value` correctly (operator search without `@` requirement). LINK skips resolution for non-`@` prefixed effect targets. EMIT outputs the raw string. This is correct for now — runtime resolution handles the binding.

---

## The Problem

The PEG grammar defines two kinds of property references in condition expressions:

```peg
NarrativePropRef ← EntityProp / ReservedPropRef
EntityProp       ← '@' Identifier '.' Identifier
ReservedPropRef  ← ('player' / 'target') '.' Identifier
```

`NarrativePropRef` is used in the `ConditionExpr` rule, which governs `? ` condition lines in narrative content (including choice guards). The grammar explicitly allows both `@entity.prop` and `target.prop` / `player.prop` as the left-hand side of comparisons.

The compiler's `parse_condition_expr()` function in `src/parse/content.rs` only implements the `EntityProp` branch. It checks whether the expression starts with `@` and handles everything inside that branch. If the expression does not start with `@`, the function falls through to `None`, which triggers a URD112 "Unrecognised syntax" error.

```rust
pub(crate) fn parse_condition_expr(expr: &str, span: &Span) -> Option<ConditionExpr> {
    // ExhaustionCheck: identifier.exhausted
    if expr.ends_with(".exhausted") { ... }

    // ContainmentCheck / PropertyComparison: @entity...
    if expr.starts_with('@') {
        // handles @entity.prop op value
        // handles @entity in container
        // handles @entity not in container
    }

    None  // ← target.prop and player.prop land here
}
```

### Observed Symptoms

The Monty Hall test fixture (`tests/valid/monty-hall.urd.md`) produces 4 URD112 errors when compiled:

| Line | Expression | Error |
|------|-----------|-------|
| 42 | `? target.state == closed` | URD112: Unrecognised syntax |
| 43 | `? target.chosen == false` | URD112: Unrecognised syntax |
| 52 | `? target.state == closed` | URD112: Unrecognised syntax |
| 53 | `? target.chosen == false` | URD112: Unrecognised syntax |

These are choice guards under `* Pick a door -> any Door` and `* Switch to the other closed door -> any Door`. The `target` keyword is the implicit binding created by `-> any Door` — it refers to whichever entity the player selects. This is core interactive fiction syntax.

### Impact

Any schema using type-targeted choices (`-> any Type`) with guard conditions is broken. This is not an edge case — `target.prop` guards are the primary mechanism for filtering which entities a player can interact with. The Monty Hall example is the canonical test case for this pattern, and it's one of the grammar test corpus fixtures.

### Note on Effects

The `parse_effect_type()` function in the same file does **not** have this bug. It searches for operators (` = `, ` + `, ` - `) without requiring an `@` prefix, so `> target.chosen = true` parses correctly. Only the condition path is affected.

### Note on Rule Blocks

Rule blocks (`rule name:`) parse `where` clauses using the same `parse_condition_expr()` function. However, rule blocks that use `target` do so via the `selects target from [...]` pattern, and the `where` clauses in rule bodies are parsed separately in `parse_rule_block()` — they call `parse_condition_expr()` with expressions like `target.prize != car`. These **also fail** for the same reason, but the Monty Hall fixture has the rule block in the frontmatter (as a top-level `rule` declaration), so the symptom is currently only visible on the narrative-side choice guards.

**Update after re-reading the code:** Actually, the rule block parser in `parse_rule_block()` also calls `parse_condition_expr()` for its `where` clauses (line: `if let Some(expr) = parse_condition_expr(&wr[6..], &ws)`). This means `where target.prize != car` inside a content-level rule block would also fail. The Monty Hall test fixture places its rule block in the narrative content section (under `### Reveal (auto)`), so this is a second manifestation of the same bug. However, the grammar distinguishes `RuleCondition` (which uses `RuleLHS ← EntityProp / RulePropRef`) from `ConditionExpr` (which uses `NarrativePropRef ← EntityProp / ReservedPropRef`). The difference:

- `ReservedPropRef` only allows `target` and `player` as reserved words.
- `RulePropRef` allows **any** `Identifier.Identifier` (bound variables from `selects`).

The fix should respect this distinction, but for now both paths go through `parse_condition_expr()` and both are broken.


## The Fix

### Part 1: Extract shared property comparison parsing

The existing `@entity.prop` path in `parse_condition_expr()` and the new `target.prop` / `player.prop` path both need to scan for an operator and split into `(property, operator, value)`. Rather than duplicating this logic, extract a shared helper:

```rust
/// Given an entity/binding name and the text after the dot (e.g. "state == closed"),
/// attempt to parse as a PropertyComparison.
fn parse_prop_comparison(entity_ref: String, after_dot: &str, span: &Span) -> Option<ConditionExpr> {
    let ops = ["==", "!=", ">=", "<=", ">", "<"];
    for op in &ops {
        if let Some(op_pos) = after_dot.find(op) {
            let property = after_dot[..op_pos].trim().to_string();
            let value = after_dot[op_pos + op.len()..].trim().to_string();
            if !property.is_empty() {
                return Some(ConditionExpr::PropertyComparison(PropertyComparison {
                    entity_ref,
                    property,
                    operator: op.to_string(),
                    value,
                    annotation: None,
                    span: span.clone(),
                }));
            }
        }
    }
    None
}
```

Both the existing `@entity.prop` branch and the new reserved-word branch call this helper. This eliminates the duplicated operator scanning loop.

### Part 2: Refactor existing `@entity.prop` path to use the shared helper

The existing `@entity.prop` branch inside the `if expr.starts_with('@')` block currently has its own inline operator scanning loop. Refactor it to call `parse_prop_comparison(entity_ref, after_dot, span)` instead. This is a pure refactor — same behavior, no new code paths.

### Part 3: Add `ReservedPropRef` handling to `parse_condition_expr()`

After the `if expr.starts_with('@')` block and before the final `None`, add:

```rust
// ReservedPropRef: target.prop or player.prop (narrative-scope reserved bindings).
// IMPORTANT: "target" and "player" are reserved binding names, NOT entity references.
// They resolve to runtime-bound values (e.g. the entity selected by `-> any Type`).
// Downstream phases (LINK, VALIDATE) must treat entity_ref values of "target" and
// "player" as reserved bindings, not as entity lookups. An entity named @target or
// @player would be rejected by the grammar's Identifier rule (reserved words).
if expr.starts_with("target.") || expr.starts_with("player.") {
    if let Some(dot_pos) = expr.find('.') {
        let entity_ref = expr[..dot_pos].to_string();
        let after_dot = &expr[dot_pos + 1..];
        if let Some(result) = parse_prop_comparison(entity_ref, after_dot, span) {
            return Some(result);
        }
    }
}
```

This produces a `PropertyComparison` node where `entity_ref` is `"target"` or `"player"` (without `@`). The comment above is load-bearing — it prevents LINK from treating these as missing-entity errors.

### Part 4: `RulePropRef` support — deferred (technical debt)

The grammar's `RulePropRef` rule (`Identifier '.' Identifier`) is more permissive than `ReservedPropRef`. For rule block `where` clauses, any bound variable name (not just `target` and `player`) should be valid as a property reference LHS. The current code routes rule-block `where` clauses through the same `parse_condition_expr()`.

**This brief only fixes `target` and `player`.** Rule blocks that use `selects` will always bind a variable called `target` by convention (it's the only variable name used in the spec examples). This unblocks all current test cases.

**⚠️ This is known technical debt, not a future nice-to-have.** The grammar explicitly allows `RulePropRef` for any bound variable (`Identifier '.' Identifier`), and the compiler does not implement it. If a rule block uses `selects victim from [...]` with `where victim.health < 0`, the compiler will reject it. The grammar-complete fix requires adding a scope parameter to `parse_condition_expr()` (narrative vs rule) and accepting any `Identifier.Identifier` as a valid LHS in rule scope. Track this as a separate brief when rule blocks need arbitrary variable names.

### Part 5: Effect expressions in rule blocks

Check whether `parse_effect_type()` handles `target.prop = value` correctly in rule block bodies. Based on code review, it does — it searches for ` = ` without requiring `@`. Confirm with a test.


## Files Changed

| File | Change |
|------|--------|
| `src/parse/content.rs` | Extract `parse_prop_comparison()` helper. Refactor `@entity.prop` branch to use it. Add `target.` / `player.` branch in `parse_condition_expr()`. |
| `tests/` | New tests for reserved property references in conditions. Update Monty Hall expected output if it currently expects errors. |


## New Tests to Add

| Test | Input | Expected |
|------|-------|----------|
| Target condition in choice guard | `? target.state == closed` | Parses as `PropertyComparison { entity_ref: "target", property: "state", operator: "==", value: "closed" }` |
| Target boolean condition | `? target.chosen == false` | Parses as `PropertyComparison { entity_ref: "target", property: "chosen", operator: "==", value: "false" }` |
| Player condition | `? player.health > 0` | Parses as `PropertyComparison { entity_ref: "player", property: "health", operator: ">", value: "0" }` |
| Player inequality | `? player.alive != false` | Parses as `PropertyComparison` with `operator: "!="` |
| Target in rule-block where | `where target.prize != car` | Parses as `PropertyComparison { entity_ref: "target", property: "prize", operator: "!=", value: "car" }` |
| Target effect in rule block | `> target.state = open` | Parses as `EffectType::Set` (should already pass — confirm) |
| Monty Hall e2e | Full fixture | Zero URD112 errors on lines 42, 43, 52, 53 |
| Non-reserved bare identifier | `? door.state == closed` | Still returns `None` / URD112 (only `target` and `player` are reserved in narrative scope) |


## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| T1 | `? target.state == closed` parses without error. | Test: Monty Hall fixture produces 0 diagnostics on these lines. |
| T2 | `? player.health > 0` parses as PropertyComparison. | Unit test on `parse_condition_expr()`. |
| T3 | `where target.prize != car` in rule blocks parses correctly. | Unit test confirming rule-block where clauses work. |
| T4 | Non-reserved identifiers (`? door.state == closed`) still rejected in narrative scope. | Test: URD112 emitted for bare non-reserved identifier. |
| T5 | All existing tests still pass. | `cargo test` — no regressions. |
| T6 | Monty Hall integration test passes cleanly. | Zero errors, zero warnings on the full fixture. |
| T7 | No duplicated operator scanning logic. | Code review: both `@entity.prop` and `target.prop` use `parse_prop_comparison()`. |


## Spec Alignment

The PEG grammar (`urd-schema-markdown.peg`) already defines `ReservedPropRef` and `RulePropRef` correctly. The Schema Markdown Syntax Specification documents `target.prop` in choice guard examples. No spec or grammar changes are needed — this is purely a compiler implementation gap.


## Future Consideration: Typed `EntityRef` in the AST

This fix stores reserved bindings (`"target"`, `"player"`) as plain strings in the same `entity_ref: String` field used for entity names like `"door_1"`. That works today because the reserved words cannot collide with entity names (the grammar forbids it), and the load-bearing comment in Part 3 documents the distinction.

However, as the AST matures — particularly once Wyrd needs to resolve bindings at runtime — this string-based approach will become fragile. A typed representation would make the distinction structural:

```rust
enum EntityRef {
    Named(String),       // @door_1, @monty
    ReservedTarget,      // target (bound by -> any Type)
    ReservedPlayer,      // player (implicit player entity)
}
```

This would eliminate the need for downstream phases to string-match against `"target"` and `"player"`, and would make it impossible to accidentally treat a reserved binding as an entity lookup.

Not needed for this fix. Track as a future AST refinement when scope-aware parsing (Part 4) or Wyrd runtime binding resolution is in scope.
