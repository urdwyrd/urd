---
title: "Compiler 0.1.2: The Condition That Never Parsed"
slug: reserved-propref-fix
description: A fix for reserved property references in narrative conditions — target.prop and player.prop now parse correctly, unblocking the primary mechanism for filtering entity-targeted choices.
date: "2026-02-19"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the second compiler patch and what it reveals about grammar-compiler alignment.
> Single canonical copy. February 2026.

## The syntax that should have worked

Schema Markdown has a pattern for entity-targeted choices. A player picks from a set of entities:

```
* Pick a door -> any Door
  ? target.state == closed
  ? target.chosen == false
```

The `-> any Door` binding creates a variable called `target` — whichever Door entity the player selects. The `? target.state == closed` line is a guard condition that filters the set: only doors that are closed and unchosen appear as options. This is the primary mechanism for interactive fiction where the player interacts with a world full of typed entities.

The compiler rejected it. Every `target.prop` condition produced URD112: "Unrecognised syntax." The same was true for `player.prop` conditions. Both are defined in the PEG grammar under the `ReservedPropRef` rule:

```
NarrativePropRef ← EntityProp / ReservedPropRef
EntityProp       ← '@' Identifier '.' Identifier
ReservedPropRef  ← ('player' / 'target') '.' Identifier
```

The grammar accepted it. The compiler did not.

## Why it happened

The condition parser in `parse_condition_expr()` had a single branching point: does the expression start with `@`? If yes, handle `@entity.prop` comparisons and `@entity in container` checks. If no, fall through to `None`, which becomes URD112.

```rust
if expr.starts_with('@') {
    // handles @entity.prop op value
    // handles @entity in container
}

None  // ← target.prop lands here
```

The `ReservedPropRef` branch — expressions starting with `target.` or `player.` — was specified in the grammar but never implemented in the parser. It was not a deliberate omission. The PARSE phase brief covered the `@entity.prop` path and the grammar test corpus exercised it. But the reserved binding path, while present in the PEG rules, had no corresponding code path.

The effects parser did not have this bug. `parse_effect_type()` searches for operators (` = `, ` + `, ` - `) without requiring an `@` prefix, so `> target.chosen = true` parsed correctly. Only the condition path was affected.

## The fix

Three changes across two files and two compiler phases.

**PARSE: shared helper and new branch.** The inline operator scanning logic that matched `@entity.prop` comparisons was extracted into a `parse_prop_comparison()` helper. Both the existing `@entity.prop` path and the new `target.`/`player.` path call this helper. The new branch checks for `target.` or `player.` prefixes after the `@` branch falls through, before the final `None` return.

**LINK: reserved binding bypass.** The LINK resolver's `resolve_condition_expr()` tries to look up every `entity_ref` in the symbol table. For `entity_ref: "target"`, this would emit URD301 ("Unresolved entity reference") because `target` is not a declared entity — it is a runtime-bound variable. The fix adds an early return for reserved bindings: when `entity_ref` is `"target"` or `"player"`, the resolver sets a minimal annotation and skips the entity table lookup. The VALIDATE phase already guards against unresolved properties, so reserved bindings pass through without triggering false type-checking errors.

**Tests.** Eight new tests cover the matrix: `target` equality and boolean conditions, `player` greater-than and not-equal, non-reserved bare identifiers still rejected, `target` in choice guards, `target` in rule-block `where` clauses, and `target` effects confirmed working.

## What this unblocks

Entity-targeted choices with guard conditions are the core interaction pattern in Schema Markdown. Without `target.prop` guards, a world with typed entities cannot filter which ones the player can interact with. The Monty Hall problem — the canonical test case for probabilistic interactive fiction — uses this pattern to let the player pick only from closed, unchosen doors.

The fix does not change the AST structure. `PropertyComparison` nodes store `entity_ref: "target"` as a plain string, the same field used for resolved entity names like `"door_1"`. The PEG grammar prevents collisions — `target` and `player` are reserved words that cannot be used as entity identifiers. A typed `EntityRef` enum (distinguishing `Named`, `ReservedTarget`, `ReservedPlayer`) is documented as a future refinement when the runtime needs to resolve bindings.

## Known limitation: rule-block variable scope

The grammar defines a more permissive rule for property references inside rule blocks:

```
RulePropRef ← Identifier '.' Identifier
```

This allows any bound variable name, not just `target` and `player`. A rule that writes `selects victim from [...]` with `where victim.health < 0` should work according to the grammar. The compiler currently only accepts `target` and `player` as valid prefixes, because both narrative conditions and rule-block `where` clauses route through the same `parse_condition_expr()` function.

This is documented technical debt. Every spec example uses `target` as the variable name in `selects` clauses, so all current patterns work. The grammar-complete fix requires adding scope awareness to the condition parser — narrative scope accepts only `ReservedPropRef`, rule scope accepts any `RulePropRef`. That is a separate task.

## The test count

Version 0.1.1 had 433 tests. Version 0.1.2 adds 8 new tests. The total stands at 441, with a 100% pass rate. The [changelog on GitHub](https://github.com/urdwyrd/urd/blob/main/packages/compiler/CHANGELOG.md) records every change.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
