# URD — Cascade Verification: Gate Guard Entity Resolution

*A verification task for confirming that comment stripping resolves the @gate_guard URD301*

February 2026 | Verification

`@gate_guard yawns.` → URD301: Unresolved reference '@gate_guard'

> **Document status: BRIEF** — This is a verification task, not a code change. After the comment stripping brief lands, re-run the stress test and confirm that the `@gate_guard` URD301 error disappears. If it persists, this brief becomes an investigation into an independent scoping or collection bug.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-20
**Status:** Done — URD301 resolved, root cause was a Move effect parser bug, not a cascade

**Depends on:** `2026-02-19-frontmatter-comment-stripping.md`

### What was done

1. Applied the comment stripping brief. Re-ran the stress test. URD301 for `@gate_guard` persisted — confirming it was not a type registration cascade.
2. Investigated the root cause. The error was at line 552: `> move @ancient_coin -> @gate_guard`. The Move effect parser in `parse_effect_type()` (`src/parse/content.rs` line 595) did not strip the `@` prefix from the destination reference. It stored `"@gate_guard"` instead of `"gate_guard"`, causing the LINK phase entity lookup to fail.
3. The asymmetry with `@captain_rhys` is explained: `@captain_rhys` is never used as a Move destination. All its references are entity speech and conditions, which use different parsing paths that correctly handle `@`-prefixed identifiers.
4. Fixed `parse_effect_type()` to strip `@` from Move destinations using `strip_prefix('@')`.
5. Added `move_effect_entity_destination` parse test confirming `> move @ancient_coin -> @gate_guard` produces `destination_ref: "gate_guard"`.
6. All 454 tests pass (453 + 1 new). Stress test errors: 19 → 8.

### What changed from the brief

- **Root cause was not a cascade.** The brief hypothesised the URD301 was a cascade from type registration failure. It was actually an independent parsing bug in Move effect destination references. The `@` prefix was stripped from the entity being moved but not from the destination.
- **Code change was required.** The brief described itself as a "verification task, not a code change." A one-line fix in `src/parse/content.rs` was needed.


## The Problem

`@gate_guard` is declared as type `Guard` in the stress test frontmatter. Because the Guard type fails to register (the comment stripping bug), `@gate_guard`'s entity symbol ends up with `type_symbol: None`. At line 552, the compiler emits URD301 "Unresolved reference '@gate_guard'" when it encounters `@gate_guard yawns.` in the Village Gate location.

### Unexplained asymmetry

`@captain_rhys` is also declared as type `Guard` and is used extensively in the Cliff Path location without any URD301 error. Both entities are in the same compilation unit, same frontmatter block, same file scope. If the type registration failure were the sole cause, `@captain_rhys` should also produce URD301 at its usage sites.

Possible explanations:
1. Entity collection is independent of type resolution, but `@gate_guard` failed collection for a different reason
2. The `resolve_in_scope()` function has a path-dependent behavior that differs between the two usage sites
3. The `@gate_guard` entity is collected but its entry in `visible_scope` is somehow different from `@captain_rhys`
4. It's a position-dependent parse issue — `@gate_guard` appears in a section (`== gate_encounter`) while `@captain_rhys` appears in a different section (`== cliff_talk`), and the resolution context differs


## Action

1. Apply the comment stripping brief.
2. Re-run `cargo run -- packages/compiler/tests/fixtures/sunken-citadel.urd.md`.
3. If URD301 for `@gate_guard` is gone: close this brief as "resolved by comment stripping."
4. If URD301 persists, investigate:
   - Add a debug print in `resolve.rs` → `resolve_entity_ref_value()` to dump `symbol_table.entities.keys()` and `visible_scope` when the lookup for `gate_guard` fails
   - Check whether `@gate_guard` is in `symbol_table.entities` after collection
   - Check whether the file context's `visible_scope` includes `@gate_guard`
   - Compare the resolution path for `@captain_rhys` (succeeds) vs `@gate_guard` (fails)
   - Document findings and create a new brief for the independent bug


## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| T1 | After comment stripping brief lands, `@gate_guard yawns.` compiles without URD301. | Stress test compilation. |
| T2 | If T1 fails, the root cause is documented with specifics about which lookup step fails. | Investigation notes in execution record. |
