# URD — Compiler VALIDATE Phase: Static Analysis Checks S3, S4, S6, S8

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-21
**Status:** Complete

### What was done

1. **S8 — Shadowed exit (URD434).** `validate_section_exit_shadowing()` walks AST content flat, tracks current location by slugifying `LocationHeading.display_name`, checks each `SectionLabel` name against the location's exit keys. ~45 lines.

2. **S6 — Missing fallthrough (URD433).** `validate_section_fallthrough()` segments files at `SectionLabel` boundaries, looks up `SectionSymbol` via `"{file_stem}/{section_name}"`, checks four conditions: sticky choice present, terminal jump after last choice, prose/speech/stage-direction after last choice, no choices at all. ~80 lines.

3. **S3 — Unreachable location (URD430).** `validate_location_reachability()` is symbol-table only. BFS from `world_start` using exit `resolved_destination`. Reports unvisited locations in insertion order. Skips if no `world_start` declared. ~45 lines.

4. **S4 — Orphaned choice (URD432).** `validate_orphaned_choices()` with helpers `check_choice_orphaned()` and `check_enum_condition()`. Walks choices, finds `PropertyComparison` conditions with `==` on enum properties, checks if value exists in the enum's values list. Recurses into nested choices. ~120 lines.

5. **Fixed three canonical fixtures** that legitimately triggered URD433: `two-room-key-puzzle` (added fallthrough prose to `== actions`), `monty-hall` (added fallthrough prose to `== switch`), `interrogation/main` (added entity speech to `== approach`).

6. **Created four negative test fixtures:** `negative-unreachable-location.urd.md`, `negative-orphaned-choice.urd.md`, `negative-missing-fallthrough.urd.md`, `negative-shadowed-exit.urd.md`.

7. **Added 36 tests total:** 32 unit tests in `validate_tests.rs` (8 for S3, 7 for S4, 11 for S6, 6 for S8) plus 4 e2e tests compiling negative fixtures. Test helpers added: `prose()`, `jump()`, `stage_direction()`, `choice_with_content()`, `effect_node()`.

8. **Playground fix.** `OutputPane.svelte` now shows warnings/info alongside compiled JSON output. Previously, successful compilation with warnings showed only the JSON with no indication that warnings existed. Added warning badge in header and clickable warnings list between header and JSON pane.

9. **Version bumped to 0.1.5.** All 516 tests pass, zero regressions.

### What changed from the brief

- **`on_exhausted` is not a `SectionSymbol` field.** The brief noted this might be the case and provided the fallback: "If it is only represented as content nodes after the choice block in the AST, condition 3 already covers it." The implementation checks for prose/speech/stage-direction after the last choice in the AST, which is exactly what EMIT treats as `on_exhausted`.

- **Playground change was not in scope.** Implementing warning-severity checks exposed that the playground silently dropped warnings on successful compilation. This was fixed as a necessary complement — without it, the four new checks would be invisible to playground users.

- **Implementation order differed from the brief.** The brief suggested S3→S4→S6→S8. Actual order was S8→S6→S3→S4, starting with the simplest check to establish the pattern before tackling the more complex ones.

---

**Created:** 2026-02-21

## Context

The v1 completion gate requires all eight static analysis checks (S1–S8) implemented and tested. Four are done: S1 (undefined entity reference — LINK phase), S2 (type mismatch — VALIDATE phase), S5 (duplicate IDs — LINK phase), S7 (circular imports — IMPORT phase). Four remain:

| # | Check | Description |
|---|-------|-------------|
| S3 | Unreachable location | A location is not reachable from `world.start` via exits. |
| S4 | Orphaned action (choice-scoped in v1) | A choice whose conditions can never be satisfied given the type constraints. The v1 implementation checks at the choice level, not the action level — see Step 10. |
| S6 | Missing fallthrough | A dialogue section with only one-shot choices and no terminal jump or fallthrough. |
| S8 | Shadowed exit (declaration) | A dialogue section name matches an exit name in the same location. |

All four belong in the VALIDATE phase. None require new infrastructure, new AST nodes, or changes to earlier phases. Each is a read-only check against the existing symbol table and AST, emitting Warning-severity diagnostics.

**Scope boundary:** This brief implements four focused validation checks to close the compiler gate. It does NOT build a general-purpose dependency graph, conflict detection infrastructure, or runtime analysis framework. Those are future work, to be designed when Wyrd provides a consumer.


## Dependencies

- **Compiler Architecture Brief** — diagnostic code ranges, severity rules, ordering.
- **VALIDATE Phase Brief** — algorithm structure, skip rules, catalog conventions.
- **LINK Phase Brief** — symbol table population, annotation model, URD310 (existing jump-level shadow warning).
- **Schema Markdown Spec** — section/choice/exit semantics, fallthrough rules.

**Required reading for implementers:** The VALIDATE phase brief (`urd-compiler-validate-brief.md`) and the symbol table definition (`symbol_table.rs`). The four checks slot into the existing VALIDATE pass as Steps 9–12, after the current Step 8 (nesting depth).


## Diagnostic Codes

All four checks use Warning severity. They inform the author of likely structural problems without blocking compilation. The compiler continues and EMIT still produces JSON.

| Code | Check | Severity | Message Template |
|------|-------|----------|-----------------|
| URD430 | S3: Unreachable location | Warning | `"Location '{location_id}' is unreachable. No path from the start location reaches it."` |
| URD432 | S4: Orphaned choice | Warning | `"Choice in section '{section_name}' (file '{file}') may never be available. Condition requires '{property}' == '{value}' but type '{type_name}' only allows: [{values}]."` |
| URD433 | S6: Missing fallthrough | Warning | `"Section '{section_name}' in file '{file}' has only one-shot choices and no terminal jump or fallthrough text. It will exhaust to an empty state."` |
| URD434 | S8: Shadowed exit (decl) | Warning | `"Section '{section_name}' in location '{location_id}' shares a name with exit '{exit_name}'. Jumps to '{section_name}' will target the section, not the exit. Use -> exit:{exit_name} to target the exit explicitly."` |

**Code URD431 is already used** by LINK for built-in terminal shadowing. URD432–URD434 are the next available codes.

**Relationship to URD310:** LINK already emits URD310 when a `-> name` jump resolves to a section that shadows an exit. URD434 is a complementary check: it fires at the *declaration* level, even if no jump targets the section. A world can trigger both — URD434 for the name collision itself, and URD310 for each jump that encounters the ambiguity.


## Step 9: Unreachable Location (S3)

### What it checks

Every location in the world should be reachable from `world.start` via a chain of exits. A location that cannot be reached from the start location — whether because it has no incoming exits at all, or because it sits in a disconnected subgraph — is likely a mistake.

### Algorithm

1. Build a directed graph of locations. Nodes are location IDs from `symbol_table.locations`. For each `LocationSymbol`, iterate its `exits` map. For each `ExitSymbol` where `resolved_destination` is `Some(target_id)`, add a directed edge from the LocationSymbol's ID to `target_id`.

2. Identify the start location. If `symbol_table.world_start` is `Some(start_id)`, use that. If `None` (no `world.start` declared, or it failed to resolve), skip the entire check — unreachability is meaningless without a defined root. Do not emit any diagnostic.

3. Run a breadth-first (or depth-first) traversal from the start location. Mark every visited node as reachable.

4. For each location in `symbol_table.locations` (insertion order) that was NOT visited: emit URD430. Use the location's `declared_in` span.

### Data available

- `symbol_table.locations: IndexMap<String, LocationSymbol>` — all declared locations.
- `LocationSymbol.exits: IndexMap<String, ExitSymbol>` — exits with `resolved_destination: Option<String>`.
- `symbol_table.world_start: Option<String>` — the resolved start location ID (set by LINK, confirmed by VALIDATE Step 1).

No new data structures or phase changes needed. Everything is already in the symbol table.

### Edge cases

- **Conditional exits.** An exit with a condition (e.g., `? @door.locked == false`) is still an edge in the graph. The check is structural, not semantic — it asks "is there a *path*" not "is the path *always available*." If the exit exists as a declaration, the location is reachable in principle. This is an intentional over-approximation. A future semantic version could check path satisfiability using condition and effect analysis, but that is out of scope for v1 and would require the dependency graph infrastructure described in future proposals.
- **Bidirectional connections.** If location A has an exit to B but B has no exit back to A, that's fine — A is reachable from the perspective of this check as long as there's a path from start to A. B is also reachable (via A). The check is about incoming edges from *any* source, not round-trip connectivity.
- **Single-location worlds.** A world with one location that is the start location produces no warnings.
- **No start location.** If `world.start` is unset or unresolved, skip the check entirely. URD404 (Step 1) already covers the missing-start case.
- **Unresolved exit destinations.** Exits where `resolved_destination` is `None` (LINK emitted URD312) contribute no edges. The target location may appear unreachable as a consequence, but that's correct — a broken exit genuinely doesn't connect anything.

### Test cases

| Case | Input | Expected |
|------|-------|----------|
| All connected | Start → A → B, start is Start. | No warnings. |
| One island | Start → A. B exists, no exits point to B. | URD430 for B. |
| No start declared | A → B. No `world.start`. | No check runs. No warnings. |
| Self-loop only | Start → Start. A exists, no exits to A. | URD430 for A. |
| Conditional exit still counts | Start → A (with condition). | No warnings. A is reachable. |
| Chain reachability | Start → A → B → C. | No warnings. All reachable transitively. |
| Multiple islands | Start → A. B and C exist unconnected. | URD430 for B and C. |
| Bidirectional pair, no path from start | Start exists alone. A ↔ B (exits both ways). | URD430 for A and B. |


## Step 10: Orphaned Choice — Pragmatic Version (S4)

### What it checks

A choice whose conditions can never be satisfied is dead code. The full version of this check (constraint satisfiability over all possible effect chains) is a significant undertaking. This brief implements a **pragmatic subset** that catches the most common authoring mistake: a condition compares an enum property against a value that doesn't exist in the enum's declared value set.

The completion gate definition gives this exact example: `? @door.state == locked` where `state` is `enum(closed, open)`. The value `locked` is not in the set. The condition can never be true. The choice is dead.

**Scoping decision: choice, not action.** The check is anchored to the `Choice` node, not to an `ActionSymbol`. In v1, choices are the primary action surface — the player interacts through them. Mapping a choice back to a stable action ID would require infrastructure that doesn't exist (ChoiceSymbol does not carry an action ID). Keeping the diagnostic choice-scoped avoids that coupling. The message includes section name and file for location context.

### Why this is the right scope for v1

VALIDATE already catches type mismatches in conditions (URD401) and invalid enum values in entity overrides (URD402). But those fire at the *individual condition* level. S4 fires at the *choice* level: "this choice can never become available because one of its conditions is statically unsatisfiable." The difference is the diagnostic's framing — it tells the author the *consequence* (dead choice), not just the *cause* (bad enum value).

A broader S4 (e.g., "no effect chain in the world can ever make `@door.state` equal `locked`") requires building a write-set analysis over all effects. That analysis is valuable and aligns with the Soufflé-inspired dependency graph work — but it should be designed when Wyrd exists to consume it. The pragmatic version closes the gate checkbox now.

### Algorithm

After Step 8 (nesting depth), check choices for statically unsatisfiable conditions:

1. For each file in topological order, walk `node.ast.content`. Track the current section name (updated when a `ContentNode::SectionLabel` is encountered; `None` before the first section label).

2. When a `ContentNode::Choice` is encountered, scan its `content` for `Condition` and `OrConditionBlock` nodes.

3. For each `ConditionExpr::PropertyComparison` with a fully resolved annotation (entity, type, property all non-null):

   a. Look up the property in the type definition via `symbol_table.types[resolved_type].properties[resolved_property]`.

   b. If the property type is `Enum` and the comparison operator is `==`:
      - Check if the comparison value is in the property's `values` list.
      - If NOT: flag this condition as statically unsatisfiable.

4. If any condition within a choice is flagged as statically unsatisfiable, emit URD432. Use the choice's span. Include the current section name (or "unnamed" if before any section label) and file path in the message for context.

5. Recurse into nested choices (sub-choices within a choice's content). Apply the same check at every nesting level.

This follows the same traversal pattern as the existing condition validation (Step 4) and effect validation (Step 5). No symbol table changes, no new mappings, no ActionSymbol involvement.

### What this does NOT catch

- A property that no effect in the world ever sets to the required value (requires write-set analysis).
- Contradictory conditions within the same choice (e.g., `? @door.locked == true` and `? @door.locked == false` on the same choice).
- Non-enum impossible conditions (e.g., `? @npc.trust > 200` where trust has `max: 100`). Range-based orphan detection could be added as a follow-up.
- Invalid boolean literals in conditions (already caught by URD401 at the individual condition level).

These are future enhancements, not v1 blockers. The dependency graph infrastructure described in the Soufflé analysis is the right foundation for the broader version.

### Interaction with existing diagnostics

URD401 already fires for the individual type mismatch. URD432 fires *additionally* at the choice level to surface the consequence: this choice will never be available. Both diagnostics appear in the output. URD401 tells you what's wrong, URD432 tells you why it matters.

If the condition's annotation is null (LINK failed to resolve), the skip rule applies — no URD432 is emitted.

### Data available

- AST content nodes: `Choice`, `Condition`, `ConditionExpr::PropertyComparison`.
- Symbol table: `TypeSymbol.properties` → `PropertySymbol { property_type, values, min, max }`.
- Annotations: `PropertyComparison.annotation` with `resolved_entity`, `resolved_type`, `resolved_property`.

### Test cases

| Case | Input | Expected |
|------|-------|----------|
| Impossible enum | `? @door.state == locked` where state is `enum(closed, open)` | URD432 warning. |
| Valid enum | `? @door.state == closed` where state is `enum(closed, open)` | No warning. |
| Non-enum condition | `? @guard.trust > 50` (integer property) | No warning (not checked by pragmatic version). |
| Unresolved annotation | `? @ghost.mood == happy` where `@ghost` doesn't exist | No warning (skip rule; LINK emitted URD301). |
| Multiple conditions, one impossible | Choice has `? @door.state == closed` and `? @door.state == locked` | URD432 for the impossible condition. |
| Choice outside section | A choice in location content (not inside a named section). | Check still runs. Section context in message is "unnamed" or uses file-level context. |


## Step 11: Missing Fallthrough (S6)

### What it checks

A dialogue section where every choice is one-shot (`*`) and there is no terminal jump (`-> end`, `-> section_name`) or prose fallthrough after the choice block will exhaust to an empty state — the player sees the section with no available choices and no narrative continuation. This is almost always an authoring mistake.

### What constitutes a "safe" section

A section is safe (no URD433) if ANY of the following are true:

1. **Has at least one sticky choice** (`+`). Sticky choices never exhaust, so the section always has something available.
2. **Has a terminal jump at the section's top level.** A `-> end` or `-> section_name` jump that is NOT nested inside a choice provides an exit path when all choices are exhausted.
3. **Has prose content after the last choice.** Text after the choice block serves as fallthrough content that the player sees when choices exhaust. (The spec says "sections without a terminal `->` fall through to text after the choice block.")
4. **Has no choices at all.** A section that's pure narrative (speech, prose, effects) with no choices is not subject to this check.
5. **Has `on_exhausted` content.** If the dialogue section declares an `on_exhausted` field (rendered when all choices have been consumed), it provides explicit fallthrough. **Implementer note:** If `on_exhausted` is available as a distinct field on `SectionSymbol`, check it directly — this is cheaper and more robust than AST inspection. If it is only represented as content nodes after the choice block in the AST, condition 3 already covers it. Check the symbol table first, fall back to AST.

### Algorithm

This check requires walking the AST content to identify section boundaries and their contents. The symbol table's `SectionSymbol` has `choices: Vec<ChoiceSymbol>` with `sticky: bool`, which handles condition 1. But conditions 2–3 require inspecting the AST content between section labels.

1. For each file in topological order, walk `node.ast.content` and segment it into sections. A section starts at a `ContentNode::SectionLabel` and extends to the next `SectionLabel` or end of file. Collect the content nodes belonging to each section.

2. For each section:

   a. Look up the `SectionSymbol` by local name. If not found (shouldn't happen after LINK), skip.

   b. **Check condition 4:** If the section has zero choices (`section_sym.choices.is_empty()`), skip — not applicable.

   c. **Check condition 1:** If any choice in `section_sym.choices` has `sticky == true`, skip — section is safe.

   d. **Check conditions 2 and 3:** Walk the section's AST content nodes. Track the position of the last `Choice` node. After the last choice, check if any of the remaining nodes are:
      - A `Jump` node at the section's top level (indent_level 0 relative to the section). This is a terminal jump.
      - A `Prose`, `EntitySpeech`, or `StageDirection` node. These are fallthrough content — narrative material the player sees when choices exhaust.

      **Normative definition of fallthrough content:** The following `ContentNode` variants count as fallthrough when they appear after the last choice at the section's top level: `Prose`, `EntitySpeech`, `StageDirection`. The following do NOT count: `Effect`, `Condition`, `OrConditionBlock`, `Comment`, `ErrorNode`, `EntityPresence`. Effects and conditions are mechanical, not narrative — they don't give the player anything to read.

      "Top level" means the node is a direct child of the section's content list, not nested inside a choice's content.

   e. If none of conditions 1–3 are met, emit URD433. Use the `SectionLabel`'s span.

### What counts as "after the last choice"

The last `Choice` node in the section's content list defines the boundary. Any content nodes that appear after it in source order (excluding `Comment` and `ErrorNode`) are potential fallthrough content. A `Jump` nested inside the last choice does NOT count — it's part of that choice's branch, not a section-level fallthrough.

### Edge cases

- **Choices with inline jumps.** `* Leave -> harbor` is a choice with an inline jump. The jump is part of the choice, not the section. It doesn't prevent exhaustion of the *section* — it just means that particular choice has an exit. If all such choices are one-shot, the section still exhausts.
- **Conditions before/between choices.** `? @guard.trust > 50` before a choice block makes the choices conditional, but doesn't provide fallthrough. Conditions are not fallthrough content.
- **Effects after choices.** An effect (`> @guard.mood = happy`) after the last choice is not prose content and does not constitute fallthrough. Only `Prose`, `EntitySpeech`, `StageDirection`, and `Jump` nodes count.
- **Sections with only `-> end`.** A section with `-> end` and no choices is safe (condition 4 — no choices).
- **Empty sections.** A section label with no content at all has no choices, so condition 4 applies — skip.
- **Mixed sticky and one-shot.** Even one sticky choice makes the section safe (condition 1).

### Data available

- `symbol_table.sections: IndexMap<String, SectionSymbol>` with `choices: Vec<ChoiceSymbol>` and `sticky: bool`.
- AST content nodes segmented by `SectionLabel` boundaries.
- `ContentNode` variants for type-matching (Jump, Prose, EntitySpeech, StageDirection, Choice).

### Test cases

| Case | Input | Expected |
|------|-------|----------|
| All one-shot, no fallthrough | Section with two `*` choices, nothing after. | URD433 warning. |
| All one-shot, terminal jump | Section with two `*` choices, then `-> topics`. | No warning. |
| All one-shot, prose fallthrough | Section with two `*` choices, then prose text. | No warning. |
| Mixed sticky and one-shot | Section with one `+` and one `*`. | No warning. |
| No choices | Section with only speech and prose. | No warning. |
| One-shot with inline jumps only | `* Go north -> north` and `* Go south -> south`, nothing after. | URD433 warning. (Inline jumps are per-choice, not section-level fallthrough.) |
| Jump nested in last choice | Last choice has `-> end` in its content. Section has no top-level jump. | URD433 warning. (Nested jump doesn't protect the section.) |
| Speech after choices | Two `*` choices, then `@arina: Goodbye.` | No warning. (EntitySpeech is fallthrough content.) |
| Only effects after choices | Two `*` choices, then `> @flag.done = true`. | URD433 warning. (Effects are not fallthrough content.) |
| Has on_exhausted content | Two `*` choices, section declares `on_exhausted` prompt. | No warning. (Explicit exhaustion handling.) |


## Step 12: Shadowed Exit at Declaration (S8)

### What it checks

If a dialogue section name matches an exit name in the same location, jumps to that name will always target the section (LINK priority rule). The exit becomes unreachable via bare `-> name` syntax. This is usually an accident — the author named a section the same as an exit without realizing it.

### Relationship to URD310

LINK's URD310 fires when a *jump* encounters the ambiguity: `-> harbor` resolves to section `harbor` instead of exit `harbor`. URD310 is per-jump — it only fires if someone actually writes `-> harbor`.

URD434 fires at *declaration* time: the moment a section name collides with an exit name in the same location, regardless of whether any jump targets it. It's a structural warning about the name collision itself.

Both can fire on the same world. URD434 tells you the collision exists. URD310 tells you a specific jump was affected by it.

### Algorithm

1. For each file in topological order, walk `node.ast.content` and track the current location context. When a `ContentNode::LocationHeading` is encountered, derive the current location ID from the **already-resolved data**, not by re-slugifying the display name. Use the same mechanism the existing VALIDATE steps use to track location context. If the location heading has an annotation with a resolved location ID, use that. If LINK populates `LocationSymbol` entries keyed by slugified ID during collection, then matching the heading's display name to a symbol table entry via `slugify()` is acceptable — but import `slugify` from `crate::slugify`, the same module LINK uses. Do NOT reimplement slug logic. A mismatch between VALIDATE's slug and LINK's slug would produce false positives or false negatives.

2. When a `ContentNode::SectionLabel` is encountered, check if the section's local name matches any exit in the current location:

   **Scoping rule:** This check uses the current location context at the point the section label is declared. The collision is meaningful when both the section name and the exit name are visible to the jump resolution algorithm at the same point in the source. This is intentional and matches LINK's URD310 behavior.

   a. If `current_location_id` is `None` (section before any location heading), skip — no exit scope to collide with. LINK already emitted URD314 for content outside location context if relevant.

   b. Look up the current location in `symbol_table.locations`. Get its `exits` map.

   c. If `exits.contains_key(&section_label.name)`, emit URD434. Use the `SectionLabel`'s span.

### Data available

- AST content nodes walked in source order, tracking `current_location_id`.
- `symbol_table.locations[id].exits: IndexMap<String, ExitSymbol>` — keyed by direction/name.
- `SectionLabel.name` — the section's local name.

### Edge cases

- **Section name matches exit direction, not destination.** Exit declarations are `-> direction: Destination`. The exit is keyed by `direction` in the exits map. So `== north` collides with `-> north: Corridor` (because "north" is the exit's key), not with `-> north: Harbor` where "north" happens to also be the destination. This is the correct behavior — `-> north` as a jump would resolve to the section or the exit.
- **Multiple locations in one file.** Each location has its own exit scope. A section `== harbor` might collide in one location but not another. The check uses the *current* location context at the point the section label appears.
- **Section before any location.** Skip — no collision possible.
- **Case sensitivity.** Section names and exit directions are compared as-is (case-sensitive), consistent with LINK's resolution behavior.

### Test cases

| Case | Input | Expected |
|------|-------|----------|
| Name collision | Location has exit `north`. Section `== north` in same location. | URD434 warning. |
| No collision | Location has exit `north`. Section `== topics` in same location. | No warning. |
| Different location | Location A has exit `harbor`. Section `== harbor` in Location B. | No warning (different scope). |
| Section before location | `== intro` appears before any `# Location`. | No warning (no location context). |
| Exit direction vs destination | Exit `-> north: Harbor`. Section `== harbor`. | No warning (`harbor` is the destination, not the exit key `north`). |
| Exit direction matches | Exit `-> harbor: Harbor Town`. Section `== harbor`. | URD434 warning (`harbor` is the exit key). |


## Implementation Notes

### Where these live in the code

All four checks go in `packages/compiler/src/validate/mod.rs` as new functions called from the main `validate()` function, after the existing Step 8 (nesting depth). They follow the same patterns:

- Called with `(graph, ordered_asts, symbol_table, diagnostics)`.
- Read-only against the symbol table and AST.
- Emit diagnostics via `diagnostics.warning()`.
- Deterministic ordering: files in topological order, content in source order.

For S6 and S8, the implementations need to walk AST content tracking section and location boundaries. This logic can share a single pass if desired (both need `current_location_id` tracking and section segmentation), but separate functions are fine for clarity. The implementer may refactor.

### Suggested function signatures

```rust
// Step 9: S3
fn validate_location_reachability(
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
);

// Step 10: S4 (pragmatic, choice-scoped)
fn validate_orphaned_choices(
    graph: &DependencyGraph,
    ordered_asts: &[String],
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
);

// Step 11: S6
fn validate_section_fallthrough(
    graph: &DependencyGraph,
    ordered_asts: &[String],
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
);

// Step 12: S8
fn validate_section_exit_shadowing(
    graph: &DependencyGraph,
    ordered_asts: &[String],
    symbol_table: &SymbolTable,
    diagnostics: &mut DiagnosticCollector,
);
```

### Test structure

Tests go in `packages/compiler/tests/validate_tests.rs`, extending the existing test file. Follow the established pattern: build ASTs programmatically, run through LINK + VALIDATE, assert on diagnostic codes and counts.

Additionally, create negative test fixtures for integration/e2e testing:

| Fixture | Tests |
|---------|-------|
| `negative-unreachable-location.urd.md` | S3: A location with no incoming exits. |
| `negative-orphaned-choice.urd.md` | S4: A choice with an impossible enum condition. |
| `negative-missing-fallthrough.urd.md` | S6: A section with only one-shot choices and no terminal. |
| `negative-shadowed-exit.urd.md` | S8: A section name matching an exit name. |

### Ordering recommendation

Implement in this order based on complexity (lowest first):

1. **S8** (shadowed exit) — simplest, ~30 lines. Walk content, track location, compare names.
2. **S6** (missing fallthrough) — moderate, ~60 lines. Walk content, segment by section, check sticky/terminal/prose.
3. **S3** (unreachable location) — moderate, ~50 lines. Build small graph from symbol table, BFS from start.
4. **S4** (orphaned choice) — most complex of the four, ~80 lines. Walk choice content, check enum conditions against type definitions.

Total estimated code: ~220 lines of implementation + ~300 lines of tests.


## Acceptance Criteria

This brief is complete when:

1. All four diagnostic codes (URD430, URD432, URD433, URD434) are implemented in the VALIDATE phase.
2. All test cases listed above pass.
3. Four negative test fixtures exist and produce the expected warnings.
4. The four canonical fixtures (`monty-hall`, `two-room-key-puzzle`, `tavern-scene`, `interrogation/`) produce no unexpected warnings from these checks. If a fixture legitimately triggers one of the new warnings (because it contains a latent structural issue these checks are designed to detect), either fix the fixture or explicitly annotate the expected warning in the fixture's test assertion.
5. The existing 480 tests continue to pass with zero regressions.
6. S1–S8 are all confirmed implemented, closing the "Static Analysis" section of the compiler gate.

*End of Brief*