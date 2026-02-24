# URD — SF-4: Semantic Diff Engine

*Compare two compiled worlds and report structural changes at the semantic level.*

February 2026 | Semantic Gate — Tier 2 (Expose)

> **Document status: BRIEF** — A pure-function diff engine operating on compiled outputs. Produces a typed change report over entity, location, dialogue, property, rule, and reachability changes. CLI integration for CI pipelines.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-24
**Status:** Done

### What was done

Implemented the semantic diff engine as specified. Created `src/diff.rs` (~700 lines) with:

- **DiffSnapshot** struct with 8 sub-snapshot types (Entity, Location, Exit, Section, Choice, Rule, Property, DiagnosticKey) and `from_compilation()` extraction
- **JSON serialisation** — `to_json()`/`from_json()` for `.urd.snapshot.json` format with version marker `urd_snapshot: "1"`
- **`diff()` function** — keyed set comparison across all 6 change categories with per-category comparators, deterministic output ordering
- **DiffReport** with `to_json()` and `summary()` output methods
- **Diagnostic key extraction** — URD430 (unreachable location) and URD432 (impossible choice) string-based extractors, matching SF-3's TypeScript patterns

Restructured `main.rs` for subcommand dispatch: `urd diff <a> <b>`, `urd snapshot <file>`, default compile.

Created 4 fixture pairs (8 files) in `tests/fixtures/diff/` and 28 tests in `diff_tests.rs` covering: identity, entity, location/exit, dialogue, property dependency, reachability, snapshot roundtrip, and integration.

### What changed from the brief

- **`entity_container_changed`** detection relies on world JSON `contains` arrays, which only populate when compilation succeeds. Partial compilations (with errors) produce snapshots without entity/location data — acceptable for the CI use case.
- **`compiled_at` field** omitted from snapshot format. The brief specified it but the compiler has no chrono dependency and adding one for a metadata-only field was unnecessary. Snapshots are identified by `world_name` only.
- **28 tests vs 27 specified** — added `diff_report_empty_json` and `diff_report_summary_text` as bonus output format tests; combined some entity tests.
- **Property reader_added test** changed to `orphan_status_changed` — the locked-garden fixture changes resulted in a writer being added (changing orphan status) rather than a net reader increase, because the "Force the gate" choice removal offset the new exit guard read.

---

## Context

When an author edits a `.urd.md` file, the compiler emits a new `.urd.json`. Today, comparing two compiled outputs means text diffing JSON — which shows syntactic noise (whitespace, key reordering) and misses semantic meaning. "The exit guard changed" and "a new property reader appeared" are invisible in a text diff.

SF-4 creates a diff engine that understands Urd's semantic model. It compares two compilations and produces a structured change report that answers: what actually changed in the world's meaning?

### What this brief delivers

Three things:

1. **A diff engine.** A Rust module that takes two `DiffSnapshot` values (derived from compiled outputs) and produces a `DiffReport` — a typed, structured list of changes.

2. **A CLI command.** `urd diff <a.urd.md> <b.urd.md>` — compiles both files, diffs the results, prints the change report as JSON to stdout.

3. **A diff snapshot format.** `urd snapshot <file.urd.md>` — compiles and writes a `.urd.snapshot.json` that bundles world JSON + FactSet JSON + PropertyDependencyIndex JSON + diagnostics. Enables `urd diff <a.snapshot.json> <b.snapshot.json>` for CI without recompilation.


## Deviations from Gate Doc

The gate document's SF-4 spec was written before the detailed briefs revealed structural requirements. Three deviations:

1. **SF-4.1 input type.** Gate specifies "two `FactSet` instances (or serialised JSON)." This brief introduces `DiffSnapshot` — a composite of FactSet + compiled world JSON + PropertyDependencyIndex + diagnostics. Reason: three of six change categories (entity container changes, property dependency changes, reachability changes) require data outside the FactSet. Diffing FactSet alone would cover only exit topology, dialogue structure, and rule changes.

2. **SF-4.8 file format.** Gate specifies `urd diff <file1.urd.json> <file2.urd.json>`. This brief accepts `.urd.md` (compiled internally) or `.urd.snapshot.json` (pre-built snapshots). Reason: `.urd.json` does not carry PropertyDependencyIndex data or diagnostics. Diffing it would miss property dependency changes and reachability changes entirely.

3. **New dependency on SF-3.** Gate lists SF-2 as the only dependency. This brief adds SF-3 because `ChoiceFact.jump_indices` (added by SF-3's FactSet extension) is required for `choice_target_changed` detection. Without it, the dialogue diff cannot determine which jumps belong to which choices.

All three deviations strengthen the diff engine's coverage. The gate acceptance criteria remain unchanged — the brief satisfies all nine.


## Dependencies

- **SF-2 passed.** PropertyDependencyIndex provides the property-level data for dependency change detection.
- **SF-1A passed.** Diagnostics D1–D5 available. URD430 (unreachable location) and URD432 (impossible choice condition) available for reachability and choice-availability comparison.
- **SF-3's FactSet extension shipped.** `ChoiceFact.jump_indices` must be present for dialogue diff to correlate choices with their jump targets.


## Architecture

### Data flow

```
  .urd.md  ──compile──▶  CompilationResult  ──extract──▶  DiffSnapshot
                                                              │
  .urd.snapshot.json  ──parse──▶  DiffSnapshot                │
                                                              ▼
                                              diff(a, b) → DiffReport
                                                              │
                                                              ▼
                                                    Structured JSON
```

### DiffSnapshot

A `DiffSnapshot` is a normalised, comparable representation of a compiled world. It is deliberately not tied to `CompilationResult` — it can be constructed from either a live compilation or a stored snapshot file.

```rust
pub struct DiffSnapshot {
    /// Entity metadata: ID → (type, properties with values, container).
    pub entities: IndexMap<String, EntitySnapshot>,
    /// Location metadata: ID → (display name, entity list).
    pub locations: IndexMap<String, LocationSnapshot>,
    /// Exit edges: ExitId → (from, to, is_conditional, guard_count).
    pub exits: IndexMap<String, ExitSnapshot>,
    /// Sections: compiled_id → (choice_ids, section_level_jump_targets).
    /// section_level_jump_targets includes ONLY jumps not owned by a choice.
    pub sections: IndexMap<String, SectionSnapshot>,
    /// Choices: ChoiceId → (label, sticky, condition_count, effect_count, jump_targets).
    /// jump_targets includes ONLY jumps owned by this choice (via jump_indices).
    pub choices: IndexMap<String, ChoiceSnapshot>,
    /// Rules: RuleId → (condition_count, effect_count).
    pub rules: IndexMap<String, RuleSnapshot>,
    /// Property dependencies: PropertyKey → (read_count, write_count, orphaned).
    pub properties: IndexMap<String, PropertySnapshot>,
    /// Diagnostic codes present: deduplicated set of (code, target_id) pairs.
    pub diagnostic_keys: BTreeSet<DiagnosticKey>,
}
```

**DiagnosticKey definition:**

```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticKey {
    /// The diagnostic code (e.g., "URD430", "URD432").
    pub code: String,
    /// Canonical element ID extracted from the diagnostic message.
    /// For URD430: location slug (e.g., "walled-garden").
    /// For URD432: section compiled_id (e.g., "gatehouse/greet").
    pub target_id: String,
}
```

**Uniqueness guarantees from the compiler:**

- **URD430** (unreachable location): at most one per location. The compiler iterates `symbol_table.locations` (unique keys) and emits one warning per unreachable location. Guaranteed unique by construction.
- **URD432** (impossible choice condition): may fire multiple times per section — one per choice whose enum condition can never match. The `target_id` is the section compiled_id, not the individual choice. Multiple URD432s for the same section are deduplicated into one `DiagnosticKey` entry because `diagnostic_keys` is a `BTreeSet`. The diff detects "this section gained/lost an impossible-condition warning" — not which specific choice triggered it. This is consistent with the count-not-site philosophy used in property dependency comparison.

**Extraction:** `target_id` is extracted from the diagnostic message by the same string-based parsing as SF-3. For URD430, the location name is extracted from `"Location '{}' is unreachable..."`. For URD432, the section name is extracted from `"Choice in section '{}' (file '{}')..."`. Both extractors are guarded by tests that fail if the message patterns change.

**Note:** SF-3's brief (frozen) refers to URD432 as "orphaned section." The actual compiler emits URD432 for impossible enum conditions on choices, not for orphaned sections. This should be corrected during SF-3 execution. The extractors in both briefs target the same message format, so the code is unaffected — only the brief text is inaccurate.

Each snapshot struct captures the **comparable attributes** of its element — the things that, if changed, constitute a semantic diff. Internal indices, spans, and source positions are excluded because they are compilation artifacts, not semantic properties.

### Building a DiffSnapshot

**From CompilationResult (live compilation):**

```rust
impl DiffSnapshot {
    pub fn from_compilation(result: &CompilationResult) -> Self {
        // entities, locations: parsed from result.world (compiled JSON)
        // exits, sections, choices, rules: extracted from result.fact_set
        // properties: built from PropertyDependencyIndex
        // diagnostic_keys: filtered from result.diagnostics
    }
}
```

**From snapshot file (stored JSON):**

```rust
impl DiffSnapshot {
    pub fn from_json(json: &str) -> Result<Self, DiffError> {
        serde_json::from_str(json)
    }
}
```

The snapshot file is the serialised DiffSnapshot. It is NOT the same as .urd.json (compiled output) — it carries FactSet-derived data that .urd.json does not.

### Why not diff .urd.json directly?

The compiled .urd.json contains structural data (entities, locations, exits, sections, choices, rules) but does NOT contain:
- PropertyDependencyIndex data (read/write sites, orphaned status)
- FactSet fact indices (which conditions guard which exits)
- Diagnostics (reachability flags)

Diffing .urd.json would report 5 of 6 change categories but miss property dependency changes and reachability changes. The snapshot format carries all six categories.

**Future extension (not in SF-4):** A degraded-mode `urd diff <a.urd.json> <b.urd.json>` that reports structural changes only. Not needed for the gate.


## Change Categories

### 1. Entity changes

**Source:** `entities` map comparison.
**Identity:** Entity ID (e.g., `@warden`).

| Change kind | Detected when |
|-------------|---------------|
| `entity_added` | ID exists in B but not A |
| `entity_removed` | ID exists in A but not B |
| `entity_type_changed` | Same ID, different type name |
| `entity_default_changed` | Same ID, same type, property value differs |
| `entity_container_changed` | Same ID, container (location) changed |

**Detail fields:** `entity_id`, `entity_type`, plus category-specific `before`/`after` values.

### 2. Location topology changes

**Source:** `locations` and `exits` map comparison.
**Identity:** Location ID, ExitId.

| Change kind | Detected when |
|-------------|---------------|
| `location_added` | Location ID in B but not A |
| `location_removed` | Location ID in A but not B |
| `exit_added` | ExitId in B but not A |
| `exit_removed` | ExitId in A but not B |
| `exit_target_changed` | Same ExitId, different `to_location` |
| `exit_condition_changed` | Same ExitId, `is_conditional` or `guard_count` changed |

### 3. Dialogue structure changes

**Source:** `sections` and `choices` map comparison.
**Identity:** Section compiled_id, ChoiceId.

| Change kind | Detected when |
|-------------|---------------|
| `section_added` | Section ID in B but not A |
| `section_removed` | Section ID in A but not B |
| `section_jumps_changed` | Same section, different outgoing jump target set |
| `choice_added` | ChoiceId in B but not A |
| `choice_removed` | ChoiceId in A but not B |
| `choice_label_changed` | Same ChoiceId, different label text |
| `choice_sticky_changed` | Same ChoiceId, sticky flag flipped |
| `choice_guard_changed` | Same ChoiceId, condition_count changed |
| `choice_effect_changed` | Same ChoiceId, effect_count changed |
| `choice_target_changed` | Same ChoiceId, jump_targets set changed |

**Jump ownership partition:** Every JumpEdge belongs to exactly one owner — either a choice or the section itself. SF-3's `ChoiceFact.jump_indices` extension makes this partition explicit: a jump whose index appears in any choice's `jump_indices` is owned by that choice; all remaining jumps in the section are section-level.

- `section_jumps_changed` fires when the set of *section-level* jump targets changes (jumps NOT owned by any choice).
- `choice_target_changed` fires when a choice's jump target set changes (jumps owned by that choice via `jump_indices`).

These are mutually exclusive — a jump change cannot trigger both. No double reporting.

### 4. Property dependency changes

**Source:** `properties` map comparison.
**Identity:** Stringified PropertyKey (`EntityType.property`).

| Change kind | Detected when |
|-------------|---------------|
| `property_appeared` | PropertyKey in B but not A (new property usage) |
| `property_disappeared` | PropertyKey in A but not B (all usage removed) |
| `reader_added` | Same key, `read_count` increased |
| `reader_removed` | Same key, `read_count` decreased |
| `writer_added` | Same key, `write_count` increased |
| `writer_removed` | Same key, `write_count` decreased |
| `orphan_status_changed` | Same key, `orphaned` flag changed (e.g., null → read_never_written) |

### 5. Rule changes

**Source:** `rules` map comparison.
**Identity:** RuleId.

| Change kind | Detected when |
|-------------|---------------|
| `rule_added` | RuleId in B but not A |
| `rule_removed` | RuleId in A but not B |
| `rule_trigger_changed` | Same RuleId, condition_count changed |
| `rule_effect_changed` | Same RuleId, effect_count changed |

### 6. Reachability changes

**Source:** `diagnostic_keys` comparison. Filtered to URD430 (unreachable location) and URD432 (impossible choice condition).

| Change kind | Detected when |
|-------------|---------------|
| `became_unreachable` | URD430 for this location in B but not A |
| `became_reachable` | URD430 for this location in A but not B |
| `choice_became_impossible` | URD432 for this section in B but not A |
| `choice_became_possible` | URD432 for this section in A but not B |

**Detail fields:** `element_type` (location or section), `element_id`.

`became_unreachable` / `became_reachable` track location-level reachability (no path from start). `choice_became_impossible` / `choice_became_possible` track section-level choice availability (an enum condition can never match). These are distinct phenomena — a section can have impossible choices while still being reachable.


## Identity Model

**SF-4.9 from the gate doc:** Identity is defined by element ID. Renamed elements are reported as removed + added, not as renames. This is a deliberate design choice.

Rationale: rename detection requires fuzzy matching (similar structure, different name). That's heuristic territory — it can't be made reliable without false positives. An author renaming `@guard` to `@warden` sees `entity_removed: @guard` + `entity_added: @warden`, which is precisely correct: the old entity no longer exists, and a new one was created.

**Key formats:**

| Element | Key format | Example |
|---------|-----------|---------|
| Entity | Entity ID | `@warden` |
| Location | Location slug | `village-square` |
| Exit | `location/exit_name` | `village-square/south` |
| Section | Compiled ID | `gatehouse/greet` |
| Choice | Compiled ID | `gatehouse/greet/ask-about-garden` |
| Rule | Rule name | `spirit_manifests` |
| Property | `EntityType.property` | `Guard.trust` |


## DiffReport Output Format

```json
{
  "changes": [
    {
      "category": "entity",
      "kind": "added",
      "id": "@new_npc",
      "detail": {
        "type": "Villager"
      }
    },
    {
      "category": "exit",
      "kind": "condition_changed",
      "id": "village-square/south",
      "before": { "is_conditional": false, "guard_count": 0 },
      "after": { "is_conditional": true, "guard_count": 1 }
    },
    {
      "category": "property_dependency",
      "kind": "writer_removed",
      "id": "Guard.trust",
      "before": { "write_count": 3 },
      "after": { "write_count": 2 }
    },
    {
      "category": "reachability",
      "kind": "became_unreachable",
      "id": "walled-garden",
      "detail": {
        "element_type": "location"
      }
    }
  ],
  "summary": {
    "total_changes": 4,
    "by_category": {
      "entity": 1,
      "exit": 1,
      "property_dependency": 1,
      "reachability": 1
    }
  }
}
```

### Ordering

Changes are ordered by: category (entity → location → exit → section → choice → rule → property_dependency → reachability), then by kind (added → removed → changed), then by ID (lexicographic). This is deterministic — same two inputs always produce byte-identical output.

### Empty report

Diffing a world against itself produces: `{ "changes": [], "summary": { "total_changes": 0, "by_category": {} } }`.


## Diff Algorithm

The core algorithm is straightforward. For each element type, the diff is a **keyed set comparison**:

```rust
fn diff_maps<V>(
    a: &IndexMap<String, V>,
    b: &IndexMap<String, V>,
    category: &str,
    compare: fn(&V, &V) -> Vec<ChangeEntry>,
) -> Vec<ChangeEntry> {
    let mut changes = Vec::new();

    // Removed: in A but not in B
    for (id, val_a) in a {
        if !b.contains_key(id) {
            changes.push(ChangeEntry::removed(category, id));
        }
    }

    // Added: in B but not in A
    for (id, val_b) in b {
        if !a.contains_key(id) {
            changes.push(ChangeEntry::added(category, id, val_b));
        }
    }

    // Modified: in both, but different
    for (id, val_a) in a {
        if let Some(val_b) = b.get(id) {
            changes.extend(compare(val_a, val_b));
        }
    }

    changes
}
```

Each element type provides a `compare` function that detects attribute-level changes. For example, `compare_exit` checks `to_location`, `is_conditional`, and `guard_count`.

### Complexity

The diff is O(n) in the number of elements (hash map lookups). For Sunken Citadel (~50 entities, ~15 exits, ~20 sections, ~40 choices, ~3 rules, ~30 properties), the diff runs in microseconds. Performance is not a concern.


## CLI Integration

### Subcommand: `urd diff`

```
urd diff <file_a> <file_b> [--format json|summary]
```

- `<file_a>` and `<file_b>` can be `.urd.md` (source files, compiled internally) or `.urd.snapshot.json` (pre-built snapshots).
- `--format json` (default): full structured change report to stdout.
- `--format summary`: human-readable summary to stdout (e.g., "3 entities added, 1 exit condition changed, 2 properties gained readers").
- Exit code 0 if no changes, 1 if changes detected (useful for CI gates).

### Subcommand: `urd snapshot`

```
urd snapshot <file.urd.md> [-o output.snapshot.json]
```

- Compiles the source file.
- Writes `DiffSnapshot` as JSON to the output path (default: `<stem>.urd.snapshot.json`).
- Useful for CI: store snapshots as build artifacts, diff between builds without recompiling.


### CLI architecture

The current `main.rs` is a simple single-command binary. Adding subcommands requires restructuring:

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("diff") => run_diff(&args[2..]),
        Some("snapshot") => run_snapshot(&args[2..]),
        Some(path) if !path.starts_with('-') => run_compile(path),
        _ => print_usage(),
    }
}
```

No argument parsing library needed. Three commands, simple flags. The existing `urd <file>` behavior is preserved as the default command.


## Snapshot File Format

The `.urd.snapshot.json` file is a serialised `DiffSnapshot`:

```json
{
  "urd_snapshot": "1",
  "world_name": "the-locked-garden",
  "compiled_at": "2026-02-24T12:00:00Z",
  "entities": { ... },
  "locations": { ... },
  "exits": { ... },
  "sections": { ... },
  "choices": { ... },
  "rules": { ... },
  "properties": { ... },
  "diagnostic_keys": [ ... ]
}
```

The `urd_snapshot: "1"` field is a format version marker, same pattern as `urd: "1"` in compiled output. This enables future format evolution without breaking existing snapshots.

`world_name` and `compiled_at` are metadata for human identification. They are NOT compared by the diff engine. Two snapshots with different `world_name` values but identical semantic content produce an empty change report.

**Version mismatch:** If `urd_snapshot` does not equal `"1"`, `from_json()` returns `Err(DiffError::UnsupportedSnapshotVersion)`. Comparing snapshots of different format versions is not supported — the user must regenerate snapshots with the current compiler.

The snapshot is **not** the full compiled output. It contains only the fields needed for semantic comparison. Source text, spans, AST data, and full diagnostic messages are excluded.


## Test Strategy

### Fixtures

SF-4 needs **paired fixtures** — two versions of a world with known differences. Three pairs:

**Pair 1: `diff-a-locked-garden.urd.md` / `diff-b-locked-garden.urd.md`**

Minimal pair based on Locked Garden. B differs from A by:
- One entity added (`@lantern: Item { name: "Lantern" }`)
- One exit condition changed (garden exit gains a new guard)
- One choice removed (Force the gate)
- One property dependency changed (`Lock.locked` gains a new writer)

Expected changes: ~6 entries.

**Pair 2: `diff-a-minimal.urd.md` / `diff-b-minimal.urd.md`**

Tiny world (1 location, 1 section, 2 choices). B adds a second location and an exit. Tests the simplest possible structural change.

Expected changes: ~3 entries.

**Pair 3: `diff-a-reachability.urd.md` / `diff-b-reachability.urd.md`**

A has a reachable location. B removes the only exit to it, making it unreachable. Tests location reachability change detection.

Expected changes: exit_removed + became_unreachable.

**Pair 4: `diff-a-impossible-choice.urd.md` / `diff-b-impossible-choice.urd.md`**

A has a choice with a valid enum condition. B changes the condition to reference an impossible enum value. Tests choice availability change detection.

Expected changes: choice_guard_changed + choice_became_impossible.

### Unit tests

| Test | Asserts |
|------|---------|
| `diff_identity_empty` | `diff(snapshot, snapshot)` produces empty change report |
| `diff_entity_added` | Adding an entity produces `entity_added` change |
| `diff_entity_removed` | Removing an entity produces `entity_removed` change |
| `diff_entity_type_changed` | Changing entity type produces `entity_type_changed` |
| `diff_entity_default_changed` | Changing a property default produces `entity_default_changed` |
| `diff_exit_added` | Adding an exit produces `exit_added` |
| `diff_exit_removed` | Removing an exit produces `exit_removed` |
| `diff_exit_condition_changed` | Adding a guard to an exit produces `exit_condition_changed` |
| `diff_exit_target_changed` | Changing exit destination produces `exit_target_changed` |
| `diff_choice_added` | Adding a choice produces `choice_added` |
| `diff_choice_removed` | Removing a choice produces `choice_removed` |
| `diff_choice_target_changed` | Changing a choice's jump target produces `choice_target_changed` |
| `diff_rule_added` | Adding a rule produces `rule_added` |
| `diff_property_reader_added` | Adding a new condition on a property produces `reader_added` |
| `diff_property_writer_removed` | Removing a property write produces `writer_removed` |
| `diff_property_orphan_changed` | Property going from balanced to read-only produces `orphan_status_changed` |
| `diff_reachability_became_unreachable` | Removing the only exit to a location produces `became_unreachable` |
| `diff_reachability_became_reachable` | Adding an exit to a previously unreachable location produces `became_reachable` |
| `diff_choice_became_impossible` | Changing a choice condition to an impossible enum value produces `choice_became_impossible` |
| `diff_choice_became_possible` | Fixing an impossible enum condition produces `choice_became_possible` |
| `diff_locked_garden_pair` | Full pair 1 produces expected change count and categories |
| `diff_deterministic` | `diff(a, b)` and `diff(a, b)` produce byte-identical JSON |
| `diff_snapshot_roundtrip` | `DiffSnapshot::from_json(snapshot.to_json())` equals original |
| `diff_snapshot_version_mismatch` | `DiffSnapshot::from_json()` with `urd_snapshot: "2"` returns `Err(UnsupportedSnapshotVersion)` |
| `diff_diagnostic_extractor_urd430` | Rust diagnostic extractor correctly extracts location ID from URD430 message (parallel to SF-3's TypeScript test) |
| `diff_diagnostic_extractor_urd432` | Rust diagnostic extractor correctly extracts section ID from URD432 message |
| `diff_all_fixtures_self_identity` | For every existing test fixture, `diff(x, x)` produces empty report (integration test: requires compilation pipeline + file I/O) |

### Test file

`tests/diff_tests.rs` (new file). Fixture pairs in `tests/fixtures/diff/`.


## Files Changed

| File | Change |
|------|--------|
| `src/diff.rs` | **New.** DiffSnapshot, DiffReport, diff() function, snapshot extraction, change comparators |
| `src/lib.rs` | Add `pub mod diff;` |
| `src/bin/main.rs` | Restructure to support `diff` and `snapshot` subcommands, preserve default compile behavior |
| `tests/diff_tests.rs` | **New.** ~22 unit tests for diff correctness |
| `tests/fixtures/diff/diff-a-locked-garden.urd.md` | **New.** Fixture pair A |
| `tests/fixtures/diff/diff-b-locked-garden.urd.md` | **New.** Fixture pair B |
| `tests/fixtures/diff/diff-a-minimal.urd.md` | **New.** Fixture pair A |
| `tests/fixtures/diff/diff-b-minimal.urd.md` | **New.** Fixture pair B |
| `tests/fixtures/diff/diff-a-reachability.urd.md` | **New.** Fixture pair A |
| `tests/fixtures/diff/diff-b-reachability.urd.md` | **New.** Fixture pair B |
| `tests/fixtures/diff/diff-a-impossible-choice.urd.md` | **New.** Fixture pair A |
| `tests/fixtures/diff/diff-b-impossible-choice.urd.md` | **New.** Fixture pair B |


## Estimated Size

| Component | Lines |
|-----------|-------|
| `diff.rs` — DiffSnapshot struct + extraction from CompilationResult | ~120 |
| `diff.rs` — DiffSnapshot serialisation/deserialisation | ~60 |
| `diff.rs` — diff() function + per-category comparators | ~250 |
| `diff.rs` — DiffReport struct + JSON output | ~80 |
| `main.rs` — subcommand restructure + diff/snapshot commands | ~80 |
| `diff_tests.rs` — 27 tests | ~440 |
| Fixture pairs (8 files) | ~400 |
| **Total** | **~1430** |

Largest brief by line count, but the bulk is test fixtures (~400 lines) and test functions (~440 lines). The diff engine itself is ~510 lines.


## Acceptance Criteria

- [ ] **SF-4.1** — Diff engine implemented as `diff(a: &DiffSnapshot, b: &DiffSnapshot) -> DiffReport`. Pure function, no side effects, no file I/O.
- [ ] **SF-4.2** — All six change categories detected and reported: entity, location/exit, dialogue (section/choice), property dependency, rule, reachability.
- [ ] **SF-4.3** — Output is structured JSON with typed change entries. Each entry has `category`, `kind`, `id`, and category-specific `detail`/`before`/`after` fields.
- [ ] **SF-4.4** — Diffing a DiffSnapshot against itself produces an empty change report (`total_changes: 0`). Verified for ALL existing test fixtures.
- [ ] **SF-4.5** — Adding a location to a test world and recompiling produces exactly the expected change entries: `location_added`, `exit_added` (if exits were added), and downstream reachability changes.
- [ ] **SF-4.6** — Removing a property write and recompiling produces a `writer_removed` change and, if applicable, an `orphan_status_changed` (property now read-but-never-written).
- [ ] **SF-4.7** — Diff is deterministic: same two inputs produce byte-identical JSON output regardless of how many times the diff is run.
- [ ] **SF-4.8** — CLI commands available: `urd diff <a> <b>` (accepts .urd.md or .snapshot.json), `urd snapshot <file.urd.md>`.
- [ ] **SF-4.9** — Identity is defined by element ID. Renamed elements appear as removed + added. This is documented in the brief and tested.


## Design Decisions

### Why a snapshot format instead of diffing .urd.json?

The compiled `.urd.json` is the runtime's input format — optimised for execution, not comparison. It doesn't carry FactSet-derived data (property dependency counts, read/write analysis) or diagnostics. Diffing it would miss an entire change category (property dependencies) and another (reachability).

The snapshot format is purpose-built for comparison. It carries exactly the data needed for all six change categories and nothing else. It's a diff-specific artifact, not a general-purpose output.

### Why counts, not full fact data in snapshots?

The snapshot captures `read_count: 3, write_count: 2` for a property, not the full list of PropertyRead tuples. This is sufficient for change detection ("trust gained a reader") but not for detailed comparison ("which specific condition was added").

Rationale: detailed fact-level comparison (which specific exit guard changed, which specific choice condition was modified) requires span-level resolution, which means the diff report would need to reference source positions in both A and B. That's a code-review tool, not a semantic diff engine. SF-4 detects *what* changed; a future tool shows *how* it changed.

**Concrete limitation:** site-level property changes are invisible. If an author moves a property read from section A to section B, `read_count` stays the same — no change detected. If an author replaces one condition with a different condition on the same property, the count is unchanged — no change detected. The diff detects *that* usage changed (reader added/removed), not *where* usage changed. This is acceptable for SF-4's CI regression use case.

### Why reachability from diagnostics, not BFS?

The compiler already runs reachability analysis (URD430) and choice availability analysis (URD432). The diff engine compares diagnostic presence between A and B. This reuses tested logic and stays consistent with the rest of the toolchain.

Running independent BFS on the ExitEdge graph would duplicate logic and risk disagreeing with the compiler's reachability analysis. Same argument as SF-3's unreachable node detection.

### Diagnostic key extraction

Reachability and choice-availability comparison uses `DiagnosticKey` (defined in the DiffSnapshot section). Extraction of `target_id` from diagnostic messages uses the same string-based matching as SF-3. URD430 extracts the location slug; URD432 extracts the section compiled_id. Multiple URD432s for the same section are deduplicated by the `BTreeSet`.

**Same known limitation as SF-3:** string-based diagnostic matching. Same forward reference to SF-5's structured diagnostic targets. Same test guard requirement: a test must fail if the extractor doesn't match current URD430/URD432 message patterns.

**Parallel extractors:** SF-3's diagnostic target extraction runs in TypeScript (browser, Svelte components). SF-4's runs in Rust (compiler crate, CLI). These cannot literally share code. Both implementations must match the same URD430/URD432 message patterns and are kept in sync by testing against the same fixtures. When SF-5 adds structured `target_id` fields, both implementations are replaced — two ~10-line functions, not a shared module.


## What This Brief Does NOT Cover

- **Visual diff rendering.** No UI component. The diff report is JSON; rendering it in the playground is a future extension.
- **Three-way diff / merge.** Two-input comparison only. No merge resolution.
- **Inline change highlighting.** No source-level annotation of what changed within a file.
- **Rename detection.** Deliberate omission per SF-4.9.
- **Playground integration.** Comparing playground edits in real-time (live before/after diff) is valuable but out of scope. The diff engine API supports it; wiring it to the UI is a separate brief.
- **.urd.json direct diff.** Future degraded-mode extension. Not needed for the gate.


## Relationship to Downstream Briefs

| Brief | How SF-4 feeds it |
|-------|--------------------|
| **SF-5** (LSP) | When SF-5 adds structured diagnostic targets, the diagnostic extraction shared between SF-3 and SF-4 should be updated. The diff engine's DiffSnapshot could also feed "what changed since last save" indicators in the LSP. |
| **SF-6** (MCP) | A `diff_worlds` endpoint could expose the diff engine to AI agents. Not required for SF-6's gate criteria, but the API is ready. |
| **Runtime gate** | The diff report becomes a regression test primitive: compile world, run tests, edit world, diff, verify only expected changes appear. The snapshot format is the CI artifact for this workflow. |

*End of Brief*
