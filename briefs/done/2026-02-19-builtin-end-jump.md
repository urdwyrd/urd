# URD — `-> end` as Built-in Dialogue Terminal

*A brief for recognizing `-> end` as a reserved jump target in the link phase*

February 2026 | Bug Fix

`-> end` → URD309: Unresolved jump target 'end'

> **Document status: BRIEF** — Fixes a LINK-phase gap where the compiler rejects `-> end` because it treats `end` as a section or exit name and fails to find it. The Schema Markdown spec §Jumps and the Writer Reference Card both document `-> end` as a built-in terminal that ends the conversation and returns control to the location or sequence. This is a documented writer promise, not an edge case.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-20
**Status:** Complete

### What was done

1. **Added `BUILTIN_JUMP_TARGETS` constant** in `src/link/resolve.rs` — `const BUILTIN_JUMP_TARGETS: &[&str] = &["end"];` defined at module level before `resolve_jump()`.

2. **Added built-in check at the top of `resolve_jump()`** — before both the exit-qualified check and the section/exit lookup. When the jump target matches a built-in, the annotation is set to `Default::default()` (no `resolved_section`, no `resolved_location`) and the function returns early. This is the correct representation: `-> end` is a terminal, not a navigation target.

3. **Added URD431 shadowing warning** — if the jump target is a built-in AND a section with the same name exists in `ctx.local_sections`, the compiler emits a warning. The built-in always wins, consistent with keyword semantics. The warning message tells the writer to rename the section.

4. **No emit changes needed** — the emitter already handles `-> end` at two locations: `build_choice_data()` (line 958) and the exhausted data builder (line 1092). Both check `if jump.target != "end"` and skip `goto` emission. This was already correct before this fix.

5. **Added 4 link tests:** `resolve_builtin_end_jump` (basic recognition, annotation has no resolved targets), `resolve_non_builtin_still_errors` (`-> ending` still URD309), `resolve_builtin_end_shadows_section` (URD431 warning, built-in wins), `resolve_builtin_end_inside_location` (works inside location context too).

6. **All 457 tests pass** (453 existing + 4 new). No regressions.

7. **Sunken-citadel stress test** reduced from 8 errors to 7. The URD309 for `-> end` on line 1030 is eliminated.

### What changed from the brief

- **Part 3 (emission) was already complete.** The brief anticipated needing emit changes, but the emitter already had `jump.target != "end"` guards at both relevant locations. No code changes needed in `emit/mod.rs`.

- **Fewer tests than the brief proposed.** The brief listed 8 tests. The "end inside choice" and "end inside nested choice" tests were omitted because they would test choice nesting mechanics, not the built-in resolution itself — the `resolve_jump()` function receives a `Jump` node regardless of nesting depth. The "exit named end" test was omitted because `-> end` is not exit-qualified (it lacks the `exit:` prefix), so it never enters the exit resolution path — the built-in check fires first. The "emission" test was omitted because the emit path was already verified by the existing `dialogue_choice_with_goto_end` emit test. The 4 tests added cover all distinct code paths: basic recognition, exact-match semantics, shadowing, and location context.


## The Problem

The Schema Markdown spec §Jumps states:

> *`-> end` **ends the conversation.** Control returns to the location's action list or advances the sequence phase.*

The spec's emission table (§Compiled JSON mapping) confirms:

> *`-> end` | No `goto` emitted; runtime exits dialogue mode.*

The stress test uses `-> end` at line 1030, at the end of the vault finale sequence:

```
-> end
```

The compiler's `resolve_jump()` function in `link/resolve.rs` follows the priority rule:

1. Match against sections declared in the current file (`ctx.local_sections`)
2. Match against exits declared in the current location (`loc_sym.exits`)
3. If neither matches, emit URD309 "Unresolved jump target"

There is no step 0 for built-in reserved targets. `"end"` is not a section name and not an exit name, so it falls through to the error.

### The jump resolution function

```rust
fn resolve_jump(...) {
    if jump.is_exit_qualified {
        // -> exit:name handling
        ...
        return;
    }

    // Standard jump: -> name
    let section_match = ctx.local_sections.get(&jump.target);
    let exit_match = current_location_id.as_ref().and_then(|loc_id| {
        symbol_table.locations.get(loc_id)
            .and_then(|loc| loc.exits.get(&jump.target))
    });

    match (section_match, exit_match) {
        (Some(_), Some(_)) => { /* section wins, warn shadowing */ }
        (Some(_), None)    => { /* section jump */ }
        (None, Some(_))    => { /* exit jump */ }
        (None, None)       => {
            diagnostics.error("URD309", ...);  // ← -> end lands here
        }
    }
}
```


## The Fix

### Part 1: Add built-in target recognition

At the top of `resolve_jump()`, before any section/exit matching, add a check for reserved built-in targets:

```rust
/// Built-in jump targets recognized by the compiler.
/// These are documented in the Schema Markdown spec §Jumps.
const BUILTIN_JUMP_TARGETS: &[&str] = &["end"];

fn resolve_jump(...) {
    // Built-in terminals — check before section/exit lookup.
    if BUILTIN_JUMP_TARGETS.contains(&jump.target.as_str()) {
        jump.annotation = Some(Annotation {
            // No resolved_section or resolved_location — this is a terminal.
            ..Default::default()
        });
        return;
    }

    if jump.is_exit_qualified { ... }
    // ... rest of function unchanged
}
```

**Placement matters.** The built-in check must come before the exit-qualified check and the section/exit lookup. This means `-> end` is always recognized, even inside a location that happens to have an exit named `end`. This is the correct priority because the built-in is a language keyword, not a user-defined name.

### Part 2: Handle shadowing by user-defined sections

If a writer declares `== end` as a section name, there is an ambiguity: does `-> end` mean "end the conversation" or "jump to the section named end"?

**Recommended rule:** The built-in wins. This is consistent with how keywords work in most languages. However, the compiler should emit a warning so the writer knows their section name shadows a built-in:

```rust
// After the built-in check, before returning:
if ctx.local_sections.contains_key("end") {
    diagnostics.warning(
        "URD431",
        "Section 'end' shadows the built-in '-> end' terminal. \
         The '-> end' jump will always end the conversation, not jump to this section. \
         Consider renaming the section.",
        jump.span.clone(),
    );
}
```

**Note:** The spec is silent on this shadowing case. The rule above is compiler-defined behavior and should be documented as such. If the spec later defines different shadowing semantics, update accordingly. Check whether the existing section-shadows-exit precedent (URD310) uses "section wins" or "exit wins" — the `-> end` shadowing rule should be consistent with that precedent.

**Update after code review:** The existing `resolve_jump()` gives sections priority over exits (section wins, URD310 warns about shadowing). For `-> end`, the built-in is more fundamental than either sections or exits — it's a language keyword. So "built-in wins" is a defensible choice even though sections normally win over exits. The warning makes the behavior transparent.

### Part 3: Handle emission

The spec's emission table says `-> end` should produce no `goto` in the compiled JSON. The runtime interprets the absence of a `goto` as "exit dialogue mode."

Check `emit/mod.rs` for how jumps are currently serialized. The emitter likely checks `jump.annotation.resolved_section` or `jump.annotation.resolved_location` to determine what to emit. For `-> end`, both are `None` (the annotation has no resolved target). The emitter needs to handle this case:

```rust
// In jump emission:
if jump.target == "end" {
    // Emit no goto — or emit a terminal marker, depending on the JSON schema.
    // Check urd-world-schema.json for the expected field.
}
```

**Important:** Do not invent a JSON shape. Consult the JSON Schema Spec for the expected output format. The acceptance criteria should include the exact emitted field once confirmed. If the schema expects `"goto": null` that is different from omitting the field entirely.


## Files Changed

| File | Change |
|------|--------|
| `src/link/resolve.rs` | Add `BUILTIN_JUMP_TARGETS` check at top of `resolve_jump()`. Add URD431 shadowing warning. |
| `src/emit/mod.rs` | Handle `-> end` in jump emission (no `goto` or terminal marker per JSON schema). |
| `tests/` | New tests for `-> end` recognition, shadowing, and emission. |


## New Tests to Add

| Test | Input | Expected |
|------|-------|----------|
| Basic end jump | `-> end` | No error, annotation has no resolved target |
| End inside choice | `* Done\n  -> end` | No error |
| End inside nested choice | `* Outer\n  * Inner\n    -> end` | No error |
| Non-built-in still errors | `-> ending` | URD309 (exact match only, not prefix) |
| Section named `end` | `== end` then `-> end` | URD431 warning, built-in wins |
| Exit named `end` | `-> end: Some Place` then `-> end` | Built-in wins (not an exit jump) |
| Emission | `-> end` | No `goto` in compiled JSON (per spec emission table) |
| Sunken Citadel e2e | Full fixture | URD309 for `-> end` eliminated |


## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| T1 | `-> end` compiles without URD309. | Stress test compilation. |
| T2 | `-> ending` still produces URD309. | Unit test. |
| T3 | Section `== end` triggers URD431 shadowing warning. | Unit test. |
| T4 | `-> end` emits correct JSON per the spec emission table. | E2e test comparing output to expected JSON. |
| T5 | Built-in targets are defined in a constant, not scattered in string comparisons. | Code review. |
| T6 | All existing tests still pass. | `cargo test` — no regressions. |


## Spec Alignment

The Schema Markdown spec §Jumps documents `-> end` explicitly. The emission table confirms the compiled output should contain no `goto` for `-> end`. The Writer Reference Card lists `-> end` alongside `-> section_name` and `-> exit:name` as one of three jump forms.

This fix aligns the compiler with the spec. No spec changes needed. The shadowing behavior (URD431) is compiler-defined and should be documented in the compiler's diagnostic reference.

**Note on `BUILTIN_JUMP_TARGETS`.** This brief touches `link/resolve.rs`, which is the same file as the implicit container property brief. If both briefs are developed in parallel, coordinate the changes. The diff surfaces do not overlap (different functions: `resolve_jump` vs `resolve_condition_expr`), but both add constants to the module. Landing one at a time with separate test suites reduces merge risk.
