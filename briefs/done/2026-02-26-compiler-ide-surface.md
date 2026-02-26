# URD — Compiler IDE Surface

*Expose the SymbolTable and DependencyGraph on CompilationResult so the IDE package can consume them without reaching into compiler internals.*

February 2026 | Done

> **Document status: BRIEF** — Two additive fields on `CompilationResult`, ownership transfer from the LINK output, and verification tests. No new modules. No WASM changes. No breaking changes.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-26
**Status:** Done

### What was done

Two new `Option` fields added to `CompilationResult` in `src/lib.rs`:

1. **`pub symbol_table: Option<symbol_table::SymbolTable>`** — the full type system with property constraints, entity overrides, rule definitions, sequence phases, and action metadata.
2. **`pub graph: Option<graph::DependencyGraph>`** — the file dependency graph with fully annotated ASTs, import edges, and entry path.

Ownership transfer implemented by destructuring `LinkedWorld` after LINK:
```rust
let linked = link::link(compilation_unit, &mut diagnostics);
let link::LinkedWorld { graph, symbol_table } = linked;
```

All subsequent phases (`extract_facts`, `validate`, `emit`) updated to use `&graph` and `&symbol_table` directly instead of `&linked.graph` and `&linked.symbol_table`. Both fields set to `Some` on the two post-LINK return paths (validate-error and success) and `None` on all pre-LINK return paths (PARSE failure, IMPORT failure, file-read failure).

Eight tests in new `tests/compilation_result_tests.rs`:
- `result_has_symbol_table_on_success` — verifies both fields `Some` on clean compilation
- `result_has_symbol_table_on_validate_error` — verifies both fields `Some` when LINK succeeds but VALIDATE emits errors
- `result_no_symbol_table_on_parse_error` — verifies both fields `None` on unclosed frontmatter (URD101)
- `result_no_symbol_table_on_import_error` — verifies both fields `None` on missing import file
- `result_symbol_table_matches_definition_index` — cross-checks every type and entity in SymbolTable has a corresponding DefinitionIndex entry
- `result_graph_has_entry_path` — verifies `graph.entry_path.is_some()`
- `result_graph_nodes_have_asts` — verifies multi-file fixture has 2+ nodes with valid AST spans
- `result_graph_edges_match_imports` — verifies edge count matches total import count across all files

All 634 compiler tests pass. No changes to `wasm.rs`, `diff.rs`, or any other module.

### Acceptance criteria verification

| Criterion | Status | Evidence |
|-----------|--------|---------|
| **IDE-S.1** — `pub symbol_table: Option<symbol_table::SymbolTable>` | **Pass** | `src/lib.rs:57` |
| **IDE-S.2** — `pub graph: Option<graph::DependencyGraph>` | **Pass** | `src/lib.rs:61` |
| **IDE-S.3** — Both `Some` whenever LINK succeeds | **Pass** | Set on validate-error path (`lib.rs:161-162`) and success path (`lib.rs:175-176`). Tests `result_has_symbol_table_on_success` and `result_has_symbol_table_on_validate_error` confirm. |
| **IDE-S.4** — Both `None` before LINK | **Pass** | Set to `None` on PARSE failure (`lib.rs:105-106`), IMPORT failure (`lib.rs:122-123`), and file-read failure (`lib.rs:204-205`). Tests `result_no_symbol_table_on_parse_error` and `result_no_symbol_table_on_import_error` confirm. |
| **IDE-S.5** — Graph contains fully annotated ASTs | **Pass** | LINK's `resolve.rs` mutates ASTs in place via `&mut graph.nodes`, filling 11 annotation slot types across all node kinds. The graph is destructured from `LinkedWorld` after LINK completes, so all annotations are present. |
| **IDE-S.6** — All existing tests pass without modification | **Pass** | 634 tests pass (626 existing + 8 new). No existing test files modified. |
| **IDE-S.7** — No changes to `wasm.rs` | **Pass** | `wasm.rs` untouched. `pnpm compiler:wasm:check` compiles cleanly. |
| **IDE-S.8** — Tests cover all four compilation outcome paths | **Pass** | 8 tests in `compilation_result_tests.rs`: success, validate-error, parse-error, import-error, plus cross-validation and graph structure checks. |

### What changed from the brief

1. **`result_graph_nodes_have_asts` test adjusted** — the brief assumed all file nodes would have non-empty `ast.content`. The `world.urd.md` fixture in `interrogation/` contains only frontmatter (types and entities) with no body content, so `content` is correctly an empty `Vec`. The test was adjusted to check for a valid AST span instead of non-empty content, with an additional check that the entry file specifically has body content.

No other deviations from the brief. The implementation matched the design exactly.

---

## Context

The Urd IDE will be a new Rust package (`packages/ide`) that depends on `urd-compiler` as a path dependency. It needs typed access to the compiler's internal data structures to build spreadsheets, graphs, inspectors, and analysis views — 85 planned features documented in the IDE feature brainstorm.

### What already exists on CompilationResult

```rust
pub struct CompilationResult {
    pub success: bool,
    pub world: Option<String>,              // compiled urd.json
    pub diagnostics: DiagnosticCollector,
    pub fact_set: Option<facts::FactSet>,
    pub property_index: Option<facts::PropertyDependencyIndex>,
    pub definition_index: Option<definition_index::DefinitionIndex>,
}
```

These six fields cover ~75 of the 85 planned IDE features. The FactSet provides reads, writes, exits, jumps, choices, and rules with spans. The PropertyDependencyIndex provides per-property read/write site lists and orphan detection. The DefinitionIndex provides declaration spans across all seven namespaces.

### What is missing

Two structures are created during compilation but not surfaced on the result:

1. **`SymbolTable`** — the full type system with property constraints (min/max, enum values, ref_type, visibility, defaults), entity property overrides, rule actors/triggers/select clauses, sequence phases with advance modes, and action targets. The DefinitionIndex is a projection of the SymbolTable — it captures declaration spans and kind metadata but drops constraint details and structural data (traits, property schemas, overrides, select definitions, phase lists).

2. **`DependencyGraph`** — the file import structure with topological ordering. Contains `IndexMap<FilePath, FileNode>` where each `FileNode` holds the file's path, its fully annotated `FileAst` (after LINK), and its import list. Also contains `edges: Vec<(FilePath, FilePath)>` and the `entry_path`.

Both are owned by `LinkedWorld` (the output of the LINK phase), used by reference during VALIDATE and EMIT, and then dropped when `compile_source_with_reader()` returns.

### What this brief delivers

Two new `Option` fields on `CompilationResult`. Ownership transfers from `LinkedWorld` at the end of the compilation pipeline. No new serialisation. No WASM changes. No new modules. No breaking changes to any existing consumer.


## Dependencies

None. This brief modifies only `src/lib.rs` and adds tests. All existing structures and modules are unchanged.


## Design

### Ownership transfer

The `compile_source_with_reader()` function currently calls:

```
LINK  → LinkedWorld { graph, symbol_table }
  extract_facts(&linked.graph, &linked.symbol_table)
  validate(&linked.graph, &linked.symbol_table, ...)
  emit(&linked.graph, &linked.symbol_table, ...)
```

All three post-LINK phases take shared references (`&`). After EMIT completes, `linked` goes out of scope and both structures are dropped. The fix: destructure `linked` into its components after LINK, pass references for the remaining phases, and move ownership onto `CompilationResult`.

### Implementation

Replace the current `let linked = link::link(...)` with a destructure:

```rust
let linked = link::link(compilation_unit, &mut diagnostics);
let link::LinkedWorld { graph, symbol_table } = linked;
```

Then update all subsequent references from `linked.graph` / `linked.symbol_table` to `graph` / `symbol_table` (they were shared references before and remain shared references — only the owner binding changes).

The four return paths after LINK all set `symbol_table` and `graph`:

1. **LINK succeeds, VALIDATE finds errors** — `fact_set` is `Some`, `world` is `None`, `symbol_table` is `Some`, `graph` is `Some`.
2. **LINK succeeds, EMIT succeeds** — everything is `Some`.
3. Both of these paths currently exist in `compile_source_with_reader()`. The early-return paths (PARSE failure, IMPORT failure) predate LINK and correctly set both new fields to `None`.

### CompilationResult changes

```rust
pub struct CompilationResult {
    pub success: bool,
    pub world: Option<String>,
    pub diagnostics: DiagnosticCollector,
    pub fact_set: Option<facts::FactSet>,
    pub property_index: Option<facts::PropertyDependencyIndex>,
    pub definition_index: Option<definition_index::DefinitionIndex>,
    /// The populated symbol table. `Some` whenever LINK succeeds.
    /// Contains full type definitions with property constraints, entity
    /// overrides, rule definitions, sequence phases, and action metadata.
    pub symbol_table: Option<symbol_table::SymbolTable>,
    /// The file dependency graph with annotated ASTs. `Some` whenever
    /// LINK succeeds. Contains per-file ASTs (with annotation slots
    /// filled by LINK), import edges, and topological ordering.
    pub graph: Option<graph::DependencyGraph>,
}
```

### What this unlocks for the IDE package

With these two fields, `packages/ide` has typed access to every data structure the compiler produces:

| IDE feature category | Primary data source |
|---------------------|-------------------|
| Entity Spreadsheet, Type Spreadsheet, Property Spreadsheet | `SymbolTable` — full property schemas, constraints, overrides |
| Location Spreadsheet, Exit Spreadsheet | `SymbolTable.locations` + `FactSet.exits` |
| Section Spreadsheet, Choice Spreadsheet | `SymbolTable.sections` + `FactSet.choices` |
| Rule Spreadsheet, Sequence Spreadsheet | `SymbolTable.rules`, `SymbolTable.sequences` |
| File Spreadsheet, File Dependency Graph | `DependencyGraph.nodes`, `DependencyGraph.edges` |
| Enum Value Coverage, Threshold Analysis | `SymbolTable` (enum values, min/max) + `FactSet` (reads/writes) |
| Containment Tree, Entity Inspector | `SymbolTable.entities` + `urd.json` locations |
| Outline Panel, Breadcrumbs, Code Lens | `DefinitionIndex` + `SymbolTable` for richer metadata |
| All other views | `FactSet`, `PropertyDependencyIndex`, `DefinitionIndex`, `urd.json` (already exposed) |

### Why not expose LinkedWorld directly?

`LinkedWorld` is a LINK-phase internal. Its structure is an implementation detail — it couples consumers to the phase boundary, not to the data. The two fields approach matches the existing pattern: `CompilationResult` already holds `FactSet` (an ANALYZE output), `PropertyDependencyIndex` (a derived index), and `DefinitionIndex` (a LINK projection). Adding `SymbolTable` and `DependencyGraph` follows the same convention — each is an independently useful data structure surfaced at the API boundary.

### Why no WASM changes?

The WASM pipeline (`wasm.rs`) already serialises `FactSet`, `PropertyDependencyIndex`, and `DefinitionIndex` to JSON. These three structures contain sufficient data for the browser playground's IDE features (inline diagnostics, autocomplete, hover, go-to-definition — all shipped in the playground-ide-features brief).

The `SymbolTable` and `DependencyGraph` are needed by the native IDE package, which will be a Rust binary consuming typed structs — not a browser WASM consumer parsing JSON. If a future brief requires SymbolTable or DependencyGraph data in the browser, it would add serialisation to `wasm.rs` at that point. This brief does not speculate.

### Why no Clone derives?

The IDE package takes ownership of `CompilationResult` and destructures it. It does not need to clone the `SymbolTable` or `DependencyGraph`. If a future consumer needs Clone, it can be added as a separate, trivial change — but adding derives speculatively violates the project's convention of minimal surface area.


## Test Strategy

### New tests

All tests go in a new `tests/compilation_result_tests.rs` file.

| Test | Asserts |
|------|---------|
| `result_has_symbol_table_on_success` | Compile a valid fixture → `result.symbol_table.is_some()` and `result.graph.is_some()` |
| `result_has_symbol_table_on_validate_error` | Compile a fixture with validation errors → `result.success == false`, `result.world.is_none()`, but `result.symbol_table.is_some()` and `result.graph.is_some()` |
| `result_no_symbol_table_on_parse_error` | Compile invalid syntax → `result.symbol_table.is_none()` and `result.graph.is_none()` |
| `result_no_symbol_table_on_import_error` | Compile a file with a bad import → `result.symbol_table.is_none()` and `result.graph.is_none()` |
| `result_symbol_table_matches_definition_index` | Compile a fixture → every type in `symbol_table.types` has a corresponding `type:Name` entry in `definition_index`. Every entity in `symbol_table.entities` has a corresponding `entity:@id` entry. |
| `result_graph_has_entry_path` | Compile a fixture → `result.graph.unwrap().entry_path.is_some()` |
| `result_graph_nodes_have_asts` | Compile a multi-file fixture → `result.graph.unwrap().nodes` contains entries for each file, each with a non-empty `ast.content` |
| `result_graph_edges_match_imports` | Compile a multi-file fixture → `graph.edges` length matches the total import count across all files |

### Existing test impact

No existing tests are affected. `CompilationResult` is constructed in `lib.rs` and consumed by `wasm.rs`, `diff.rs`, and tests. All existing consumers destructure only the fields they need — the two new `Option` fields are ignored by existing code (Rust's struct pattern matching allows this).

The `diff.rs` module takes `&CompilationResult` and accesses `fact_set`, `property_index`, and `world`. It does not destructure the entire struct, so the new fields have no impact.


## Files Changed

| File | Change |
|------|--------|
| `src/lib.rs` | Add two fields to `CompilationResult`. Destructure `LinkedWorld` after LINK. Set `symbol_table` and `graph` on all return paths. |
| `tests/compilation_result_tests.rs` | **New.** 8 tests verifying field presence on success, validate-error, parse-error, and import-error paths. |

No other files changed.


## Estimated Size

| Component | Lines |
|-----------|-------|
| `lib.rs` — struct fields + doc comments | ~8 |
| `lib.rs` — destructure + variable rename | ~15 (net change, replacing `linked.graph` with `graph` etc.) |
| `lib.rs` — set fields on return paths | ~8 |
| `tests/compilation_result_tests.rs` | ~120 |
| **Total** | **~150** |


## Acceptance Criteria

- [ ] **IDE-S.1** — `CompilationResult` has `pub symbol_table: Option<symbol_table::SymbolTable>` field.
- [ ] **IDE-S.2** — `CompilationResult` has `pub graph: Option<graph::DependencyGraph>` field.
- [ ] **IDE-S.3** — Both fields are `Some` whenever LINK succeeds (including when VALIDATE emits errors and `world` is `None`).
- [ ] **IDE-S.4** — Both fields are `None` when compilation fails before LINK (PARSE failure or fatal IMPORT error).
- [ ] **IDE-S.5** — The `DependencyGraph` on the result contains fully annotated ASTs (annotation slots filled by LINK's resolution pass).
- [ ] **IDE-S.6** — All existing tests pass without modification (569 compiler tests + any site/grammar/schema tests).
- [ ] **IDE-S.7** — No changes to `wasm.rs`. WASM compilation and the browser playground remain unaffected.
- [ ] **IDE-S.8** — New test coverage verifies field presence/absence across all four compilation outcome paths (success, validate-error, parse-error, import-error).


## What This Brief Does NOT Cover

- **WASM serialisation of SymbolTable or DependencyGraph.** The browser playground uses DefinitionIndex, FactSet, and PropertyDependencyIndex. If a future brief needs SymbolTable data in the browser, it adds serialisation at that point.
- **Clone derives on SymbolTable or DependencyGraph.** The IDE takes ownership. Clone can be added trivially if a future consumer needs it.
- **The IDE package itself.** This brief prepares the compiler's API surface. The `packages/ide` package, its Cargo.toml, and its features are a separate brief.
- **Any new compiler diagnostics, analyses, or phases.** This brief is purely additive surface area.
- **Changes to any module other than `lib.rs`.** The SymbolTable, DependencyGraph, and all their nested types are already `pub`. No visibility changes needed anywhere.


## Relationship to Downstream Work

| Work | How this brief feeds it |
|------|------------------------|
| **packages/ide** | The IDE package depends on `urd-compiler` and consumes `CompilationResult`. This brief ensures it has access to every data structure needed for all 85 planned features. |
| **LSP server** (`packages/lsp`) | Already has access to the SymbolTable via its own `WorldState` which calls `compile_source_with_reader()`. Could be simplified to use `CompilationResult` directly. Not required by this brief. |
| **MCP server** (`packages/mcp`) | Same pattern as LSP. Could benefit from direct access but not required. |
| **Future playground enhancements** | If a browser feature needs SymbolTable data (e.g., richer hover tooltips showing enum values or min/max constraints), a follow-up brief adds `to_json()` on the SymbolTable and wires it through `wasm.rs`. This brief does not block that path. |

*End of Brief*
