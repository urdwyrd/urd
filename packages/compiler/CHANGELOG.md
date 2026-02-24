# Changelog

All notable changes to the Urd compiler are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/). Versions use [Semantic Versioning](https://semver.org/).

## [0.1.12] — 2026-02-24

### Added

- **Semantic diff engine (`diff` module):** Compares two compiled Urd worlds at the semantic level, producing typed change entries across six categories: entity, location/exit, dialogue (section/choice), property dependency, rule, and reachability. Core types: `DiffSnapshot`, `DiffReport`, `ChangeEntry`.
- **DiffSnapshot extraction:** `DiffSnapshot::from_compilation()` builds a normalised, comparable representation from a `CompilationResult` — composite of FactSet, world JSON, PropertyDependencyIndex, and diagnostics. Captures only comparable attributes (no spans, no indices, no source).
- **Snapshot file format:** `.urd.snapshot.json` with version marker (`urd_snapshot: "1"`), JSON serialisation via `to_json()`/`from_json()` for CI snapshot workflows.
- **Reachability diffing from diagnostics:** URD430 (unreachable location) and URD432 (impossible choice) parsed from diagnostic messages to detect `became_unreachable`, `became_reachable`, `choice_became_impossible`, and `choice_became_possible` changes.
- **CLI subcommands:** `urd diff <a> <b> [--format json|summary]` compares two files or snapshots (exit code 0 = no changes, 1 = changes). `urd snapshot <file> [-o output]` produces a `.urd.snapshot.json`. Default `urd <file>` compile behaviour unchanged.
- 4 new test fixture pairs (8 files) in `tests/fixtures/diff/`: locked-garden, minimal, reachability, impossible-choice.
- 28 new tests covering identity, entity, location/exit, dialogue, property dependency, reachability, snapshot roundtrip, and integration. Total: 612 tests.

## [0.1.11] — 2026-02-24

_Intermediate build — SF-4 diff engine code committed without artefact rebuild. Superseded by 0.1.12._

## [0.1.10] — 2026-02-24

### Added

- **ChoiceFact.jump_indices:** Choices now track which JumpEdge indices they own, enabling dialogue graph edge-to-choice correlation. Follows the same index-reference pattern as `condition_reads` and `effect_writes`.
- **Playground graph visualisation:** Two new Analysis panel tabs — Location (topology from ExitEdge tuples, LR layout) and Dialogue (flow from JumpEdge + ChoiceFact tuples, TB layout). Shared SVG renderer with dagre layout, pan/zoom, diagnostic-driven node flags (URD430 unreachable, URD432 impossible choices).
- 4 new tests for `jump_indices` correctness and referential integrity. Total: 584 tests.

## [0.1.9] — 2026-02-24

### Added

- **PropertyDependencyIndex set-difference queries:** `read_but_never_written()` and `written_but_never_read()` return sorted property keys for orphaned properties. D1/D2 diagnostics refactored to call these methods — shared source of truth between compiler and playground.
- **PropertyDependencyIndex JSON serialisation:** `to_json()` produces a deterministic, property-centric JSON structure with per-property read/write counts, index arrays, orphaned flags, and summary block. Properties sorted lexicographically by (entity_type, property).
- **WASM pipeline:** `property_index` field added to `CompileResult` JSON output alongside `facts`.
- **Playground Properties tab:** New `PropertyDependencyView` component with summary header, All/Read-only/Write-only filter buttons, collapsible property rows with R/W count badges, orphaned badges, and expandable site lists with clickable spans. Tab bar [Properties / Facts] in the Analysis panel.
- `property_index` field on `CompilationResult` — index built once after `extract_facts()`, consumed by both `analyze()` and WASM serialisation.
- 11 new tests covering index determinism, set-difference correctness, JSON structure, orphaned flags, all-fixture no-panic, and D1/D2 correspondence. Total: 580 tests.

## [0.1.8] — 2026-02-24

### Added

- **ANALYZE phase:** Five new FactSet-derived diagnostics (URD601–URD605) that operate solely on the FactSet and PropertyDependencyIndex — no AST, no symbol table, no source text.
  - **URD601** — Property read in conditions but never written by any effect.
  - **URD602** — Property written by effects but never read in any condition.
  - **URD603** — Effect sets enum property to variant that no condition tests.
  - **URD604** — Condition tests numeric threshold unreachable by any effect (conservative: skips if Add/Sub writes exist).
  - **URD605** — Circular property dependency: every write is guarded by a read of the same property, with no unguarded bootstrap path.
- `analyze()` function accepting `&FactSet` and `&PropertyDependencyIndex`, returning `Vec<Diagnostic>`. Called between LINK and VALIDATE in the compilation pipeline.
- 3 new test fixtures: `negative-factset-diagnostics.urd.md` (clean), `positive-factset-diagnostics.urd.md` (triggers all five), `positive-factset-circular-deep.urd.md` (multi-write circular).
- 11 new tests (5 diagnostic-specific, 5 existing-fixture smoke, 1 architectural constraint). Total: 569 tests.

## [0.1.7] — 2026-02-22

### Added

- **C8/C9 gate verification:** Dedicated tests proving URD411 (`urd:` override warning) and URD410 (nesting depth warn at 3, error at 4+) work correctly. Both were already implemented — this release adds gate-level evidence.
- **Gate verification tests:** `gate_canonical_fixtures_zero_warnings` (all five canonical fixtures compile clean), `gate_negative_corpus_correct_codes` (nine negative fixtures rejected with expected diagnostic codes and valid span locations), `gate_json_schema_validates_all_fixtures` (compiled JSON from all fixtures validates against published JSON Schema).
- **JSON Schema validation:** Added `jsonschema` crate as dev-dependency. Schema validation is now part of `cargo test`.
- 2 new negative fixtures: `negative-urd-override.urd.md` (URD411 warning), `negative-nesting-depth.urd.md` (URD410 error at depth 4).
- 7 new tests (unit + e2e + gate). Total: 554 tests (also fixes test report to include 31 previously uncounted FactSet tests).
- **Compiler gate:** All compiler-side acceptance criteria (C1–C9, S1–S8, F1–F8, fixture verification, schema validation, negative corpus) now pass. Specification audit (E1–E7) remains.

## [0.1.6] — 2026-02-22

### Added

- **FactSet analysis IR:** A normalized, immutable, deterministic intermediate representation extracted after LINK. Projects the resolved world into flat relational tuples queryable without AST traversal. Six fact types: `ExitEdge`, `JumpEdge`, `ChoiceFact`, `RuleFact`, `PropertyRead`, `PropertyWrite`.
- **PropertyDependencyIndex:** Derived index mapping `PropertyKey` to read/write site indices for property-centric queries.
- **FactSetBuilder:** Private builder enforcing referential integrity — only `extract_facts()` can construct a FactSet.
- **WASM serialisation:** `compile_source` now returns serialised facts alongside compiled JSON, enabling playground tooling.
- **Playground Analysis panel:** Collapsible panel below the editor/output split displaying extracted facts by type.
- 31 new tests across all five canonical fixtures covering extraction, determinism, cross-reference integrity, and site resolution.
- Total: 547 tests.

## [0.1.5] — 2026-02-21

### Added

- **S3 URD430: unreachable location.** BFS from `world.start` via resolved exits. Locations not reachable emit a warning with the location name and line.
- **S4 URD432: orphaned choice.** Detects choices whose enum condition references a variant that no effect in the file ever sets. The choice can never be selected.
- **S6 URD433: missing fallthrough.** One-shot-only sections (every choice is `one-shot`, no `sticky`) with no terminal jump (`->`) will exhaust and strand the player.
- **S8 URD434: shadowed exit.** Warns when a section label inside a location has the same name as an exit key in that location. Jump target resolution would match the exit, shadowing the section.
- **Playground warning display:** OutputPane now renders warnings alongside compiled JSON instead of silently dropping them.
- 4 negative test fixtures (`negative-unreachable-location`, `negative-orphaned-choice`, `negative-missing-fallthrough`, `negative-shadowed-exit`).
- 36 new tests (validate + e2e). Total: 516 tests.

### Changed

- All eight static analysis checks (S1–S8) now implemented, completing the Static Analysis section of the v1 completion gate.

## [0.1.4] — 2026-02-20

### Fixed

- **World name auto-slugification:** The `world.name` field from frontmatter is now slugified during emit, matching how location, sequence, and phase names are already handled. Writers can use display names like `name: Monty Hall` and the compiler emits `"name": "monty-hall"`, conforming to the JSON Schema pattern `^[a-z][a-z0-9-]*$`.

## [0.1.3] — 2026-02-20

### Added

- **List and entity ref values in frontmatter:** `Scalar` enum now supports `List(Vec<Scalar>)` and `EntityRef(String)` variants. Entity overrides like `tags: [conspiracy, voss]` and `requires: @bone_key` parse as structured data instead of falling through to strings. Empty lists, trailing commas, nested entity refs in lists, and recursive element parsing all work through the existing `parse_flow_list()` infrastructure.
- **Implicit `container` property recognition:** The `container` property — defined in the Schema Spec as an implicit property of every entity — is now accepted in conditions, effects, and entity overrides without requiring a type declaration. Controlled by the `IMPLICIT_PROPERTIES` constant in the resolver.
- **Built-in `-> end` jump target:** `-> end` is now recognised as a built-in dialogue terminal in the LINK phase. The `BUILTIN_JUMP_TARGETS` constant defines reserved targets checked before section/exit lookup. If a section named `end` shadows the built-in, URD431 warns and the built-in wins.
- **URD430 warning** for unparseable type-definition lines at type indent level (uppercase start, failed parse). Replaces the silent skip that previously caused cascade errors.
- **URD431 warning** when a user-defined section shadows the built-in `-> end` terminal.
- **URD432 warning** for unparseable entity-declaration lines starting with `@` in the entities block.
- 39 new tests (9 parse, 5 link for container, 4 link for `-> end`, 8 parse for silent skip, 9 parse for list/ref, 4 regression). Total: 480 tests.

### Fixed

- **Frontmatter inline comment stripping:** Lines like `Guard [interactable]: # NPC type` now parse correctly. A quote-aware `strip_frontmatter_comment()` function strips `# ...` suffixes while preserving `#` inside quoted strings. Applied at four entry points: type definitions, property definitions, entity declarations, and world fields.
- **Move effect destination `@` prefix:** `> move @coin -> @guard` now correctly strips the `@` from the destination reference, producing `destination_ref: "guard"` instead of `"@guard"`. Previously caused cascading URD301 errors.
- **`-> end` no longer emits URD309:** The LINK phase now recognises `end` as a built-in target before attempting section/exit lookup.
- **`.container` no longer emits URD308:** Implicit properties bypass the declared-property check at all three resolution sites (entity overrides, condition property access, effect property access).
- **List defaults and overrides no longer emit URD413/URD401:** `tags: list = []` and `tags: [conspiracy, voss]` are now parsed as `Scalar::List` and validated against `PropertyType::List` correctly.
- **Entity ref overrides no longer misrepresented:** `requires: @bone_key` parses as `Scalar::EntityRef("bone_key")` and validates against `PropertyType::Ref` with proper type checking.
- **Sunken-citadel stress test:** 1101-line fixture now compiles with zero errors (was 19 errors at start of 0.1.2).

## [0.1.2] — 2026-02-19

### Fixed

- **Reserved property references in conditions:** `? target.state == closed` and `? player.health > 0` now parse correctly as `PropertyComparison` nodes. Previously these fell through to `None` in `parse_condition_expr()`, emitting spurious URD112 "Unrecognised syntax" errors. This was the primary mechanism for filtering entity-targeted choices (`-> any Type`) with guard conditions.
- **LINK phase bypass for reserved bindings:** The resolver now skips entity lookup for `target` and `player` property comparisons, preventing false URD301 "Unresolved entity reference" errors. These are runtime-bound values, not static entity names.

### Changed

- Extracted `parse_prop_comparison()` helper from inline operator scanning in `parse_condition_expr()`. Both the `@entity.prop` path and the new `target.`/`player.` path use the shared helper, eliminating duplicated logic.
- 8 new tests covering `target.prop` equality, `target.prop` boolean, `player.prop` greater-than, `player.prop` not-equal, non-reserved bare identifier rejection, `target.prop` in choice guards, `target.prop` in rule-block `where` clauses, and `target.prop` effect parsing confirmation.

## [0.1.1] — 2026-02-19

### Added

- **Type shorthand aliases:** `int`, `num`, `str`, `bool` accepted as aliases for `integer`, `number`, `string`, `boolean` in frontmatter property definitions. Aliases are normalised to canonical names during parsing — compiled JSON always uses the long form.
- **Range shorthand parsing:** `int(0, 100)`, `integer(0, 100)`, `num(0.0, 1.0)`, `number(0.0, 1.0)` now correctly extract min/max range values. Previously the entire string (including parentheses) was treated as an unrecognised type and silently fell through to `string`.
- **URD429 warning** for unrecognised type strings (e.g. `integr`, `floot`). Previously these compiled silently as `string` with no diagnostic. The property is still treated as `string`, but the author is warned.
- `raw_type_string` field on `PropertySymbol` to preserve the source type string for validation diagnostics.
- 20 new tests (10 parse, 4 validate, 6 end-to-end) covering alias normalisation, range extraction, malformed ranges, and unrecognised type warnings.

### Fixed

- `int` no longer silently compiles as `string` (was missing from the `parse_property_type()` match arms in LINK).
- `int(0, 100)` no longer stored as a raw type string — range values are correctly extracted during PARSE.

## [0.1.0] — 2026-02-18

Initial release of the Urd compiler.

### Added

- Five-phase compilation pipeline: PARSE, IMPORT, LINK, VALIDATE, EMIT.
- Full Schema Markdown frontmatter parsing: world metadata, type definitions with traits, entity declarations with inline property overrides.
- Full narrative body parsing: rooms (headings), prose, presence markers, exits with conditions and failure text, sections, dialogue attribution, sticky/one-shot choices with nesting, conditions (`?` lines), effects (`>` lines), jump arrows (`->`), and `any:`/`all:` condition groups.
- IMPORT phase with dependency graph construction and cycle detection.
- LINK phase with symbol table construction, entity-to-type resolution, property collection, and enum variant validation.
- VALIDATE phase with 30+ diagnostic rules (URD400–URD428): undefined references, duplicate definitions, type mismatches, unreachable sections, missing start rooms, and more.
- EMIT phase producing `urd.json` output conforming to the Urd World Schema.
- WASM target via `wasm-bindgen` for browser use (`compile_source`, `parse_only`, `compiler_version`).
- Native CLI binary (`urd`) for command-line compilation.
- 413 tests across all phases.
- Benchmark harness for compilation performance measurement.
