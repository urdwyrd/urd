# Changelog

All notable changes to the Urd compiler are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/). Versions use [Semantic Versioning](https://semver.org/).

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
