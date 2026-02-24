# URD — SF-2: PropertyDependencyIndex

*Ship the property-level query surface and make it visible in the browser.*

February 2026 | Semantic Gate — Tier 2 (Expose)

> **Document status: DONE** — Extends the existing `PropertyDependencyIndex` with set-difference methods, JSON serialisation, a property-centric playground panel, and shared source of truth with SF-1A diagnostics D1/D2.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-24
**Status:** Done

### What was done

| Deliverable | Result |
|------------|--------|
| **SF-2.1** — Six methods on PropertyDependencyIndex | All six implemented: `reads_of`, `writes_of`, `read_properties`, `written_properties`, `read_but_never_written`, `written_but_never_read` |
| **SF-2.2** — Index built from FactSet in single pass | `build()` unchanged. Index now built once in `lib.rs` pipeline, passed to both `analyze()` and WASM serialisation |
| **SF-2.3** — Deterministic `to_json()` | Properties sorted lexicographically by (entity_type, property). Verified by `index_build_deterministic` test |
| **SF-2.4** — JSON serialisation in WASM pipeline | `property_index` field added to `CompilationResult` and WASM JSON output. TypeScript types added to `compiler-bridge.ts` |
| **SF-2.5** — Property-centric playground panel | `PropertyDependencyView.svelte` created. Tab bar [Properties / Facts] added to Analysis panel. Default tab: Properties |
| **SF-2.6** — Shared source of truth | D1/D2 in `analyze.rs` refactored to call `index.read_but_never_written()` / `written_but_never_read()`. Verified by `index_matches_d1_d2` test |
| **SF-2.7** — Test coverage | 11 new tests added to `facts_tests.rs`. All 580 tests pass (569 existing + 11 new) |

### What changed from the brief

- **`analyze()` signature change**: The brief specified changing `analyze(fact_set: &FactSet)` to `analyze(fact_set: &FactSet, index: &PropertyDependencyIndex)`. This was implemented as specified. All callers (lib.rs pipeline, analyze_tests.rs) updated.
- **`CompilationResult` extended**: Added `property_index: Option<facts::PropertyDependencyIndex>` field to avoid building the index twice. Built once after `extract_facts()`, stored on the result, consumed by both analyze and wasm serialisation.
- **Error fallback in compiler-bridge.ts**: Added explicit `facts: null` and `property_index: null` to the catch block — the brief didn't mention this but it was necessary for type correctness.
- **No other deviations.** All JSON structure, method signatures, test matrix, and component design match the brief.

---

## Context

The `PropertyDependencyIndex` was introduced as part of the analysis IR brief (compiler gate, `2026-02-21-urd-compiler-analysis-ir-facts-provenance.md`). It is a secondary index built from the FactSet in a single pass, mapping `PropertyKey` (entity_type + property) to read and write indices.

### What already exists

The following are implemented and tested in `src/facts.rs`:

```rust
pub struct PropertyDependencyIndex {
    readers: IndexMap<PropertyKey, Vec<usize>>,
    writers: IndexMap<PropertyKey, Vec<usize>>,
}

impl PropertyDependencyIndex {
    pub fn build(fact_set: &FactSet) -> Self;
    pub fn reads_of(&self, key: &PropertyKey) -> &[usize];
    pub fn writes_of(&self, key: &PropertyKey) -> &[usize];
    pub fn read_properties(&self) -> impl Iterator<Item = &PropertyKey>;
    pub fn written_properties(&self) -> impl Iterator<Item = &PropertyKey>;
}
```

The playground's Analysis panel (`FactSetView.svelte`) displays raw facts organized by type — exits, choices, reads, writes, jumps, rules. Each row is clickable and scrolls the editor to the corresponding span.

The WASM bridge (`wasm.rs`) serialises the FactSet via `fact_set.to_json()` and returns it in the `CompileResult`. The playground receives `facts: FactSet | null`.

### What this brief delivers

Four things:

1. **Two new methods** on `PropertyDependencyIndex`: `read_but_never_written()` and `written_but_never_read()` — set-difference queries that return property keys where reads exist without writes (or vice versa).

2. **JSON serialisation** of the index. A `to_json()` method on `PropertyDependencyIndex` that produces a property-centric view. Wired into the WASM pipeline alongside the FactSet.

3. **A property-centric playground panel** that shows properties grouped by key, with read/write counts, orphaned-property flags, and expandable site lists with clickable spans.

4. **Shared source of truth** with SF-1A. After SF-1A is implemented, D1 (`check_read_never_written`) and D2 (`check_written_never_read`) must be refactored to call the index's set-difference methods rather than reimplementing the logic. The diagnostic functions become thin wrappers around the index queries.


### Why this matters

The PropertyDependencyIndex is the foundation for every downstream brief:

- **SF-3** (Visualisation): Property annotations on graph nodes and edges
- **SF-4** (Semantic Diff): Property dependency changes (reader added, writer removed)
- **SF-5** (LSP): Hover data for `@entity.property` — type, default, read/write site counts
- **SF-6** (MCP): `get_property_dependencies(entity, property)` endpoint

Without SF-2, these briefs would each need to build their own ad-hoc property lookups from raw FactSet data. The index is the shared vocabulary.

**Architectural role:** The PropertyDependencyIndex is the canonical property reference index for all tooling. It powers diagnostics, hover, go-to-usage, find-references, diff, and MCP queries. Any tool that needs to answer "who reads or writes this property?" goes through the index, not through the FactSet directly.


## Dependencies

- **SF-1A passed.** The five FactSet diagnostics must be working. D1 and D2 must exist so that SF-2.6 (shared source of truth) can be verified.
- **Compiler gate closed.** FactSet extraction working. PropertyDependencyIndex `build()` and four lookup methods implemented.


## New Methods

### `read_but_never_written()`

```rust
/// Property keys that appear in conditions but never in effects.
/// Returns keys sorted by (entity_type, property).
pub fn read_but_never_written(&self) -> Vec<&PropertyKey> {
    let mut keys: Vec<_> = self.readers.keys()
        .filter(|k| !self.writers.contains_key(*k))
        .collect();
    keys.sort_by(|a, b| (&a.entity_type, &a.property).cmp(&(&b.entity_type, &b.property)));
    keys
}
```

This is the same set difference that SF-1A diagnostic D1 computes. After SF-2, D1 calls this method instead of reimplementing the logic.

### `written_but_never_read()`

```rust
/// Property keys that appear in effects but never in conditions.
/// Returns keys sorted by (entity_type, property).
pub fn written_but_never_read(&self) -> Vec<&PropertyKey> {
    let mut keys: Vec<_> = self.writers.keys()
        .filter(|k| !self.readers.contains_key(*k))
        .collect();
    keys.sort_by(|a, b| (&a.entity_type, &a.property).cmp(&(&b.entity_type, &b.property)));
    keys
}
```

Same relationship to D2.

### Return type

Both methods return `Vec<&PropertyKey>` rather than `impl Iterator` because downstream consumers (D1/D2 diagnostics, JSON serialisation, playground display) all need to iterate multiple times or collect into structures. Returning a collected `Vec` avoids lifetime complications and makes the API predictable.


## JSON Serialisation

### `PropertyDependencyIndex::to_json()`

The index serialises to a property-centric JSON structure. Each property key appears once with its read indices, write indices, and orphaned flags pre-computed.

```json
{
  "properties": [
    {
      "entity_type": "Guard",
      "property": "trust",
      "read_count": 3,
      "write_count": 2,
      "read_indices": [0, 3, 7],
      "write_indices": [1, 5],
      "orphaned": null
    },
    {
      "entity_type": "Guard",
      "property": "suspicion",
      "read_count": 1,
      "write_count": 0,
      "read_indices": [2],
      "write_indices": [],
      "orphaned": "read_never_written"
    },
    {
      "entity_type": "Guard",
      "property": "loyalty",
      "read_count": 0,
      "write_count": 1,
      "read_indices": [],
      "write_indices": [4],
      "orphaned": "written_never_read"
    }
  ],
  "summary": {
    "total_properties": 3,
    "total_reads": 4,
    "total_writes": 3,
    "read_never_written": 1,
    "written_never_read": 1
  }
}
```

The `orphaned` field is `null` when a property has both readers and writers, `"read_never_written"` when it has readers but no writers, and `"written_never_read"` when it has writers but no readers.

The `summary` block provides counts for the panel header without requiring the client to iterate.

### Property ordering

Properties in the JSON output are sorted lexicographically by `(entity_type, property)`. This ordering is stable regardless of whether a property is first encountered as a read or a write, which means small edits to the source don't shuffle the JSON output. It also makes the playground's property list predictable for authors and produces clean diffs for SF-4.

The internal `IndexMap` retains insertion order for build performance. The sort happens only at serialisation time (`to_json()`) and in the return values of `read_but_never_written()` / `written_but_never_read()`. Implementation: collect all unique keys from both `readers` and `writers` maps, sort by `(entity_type, property)`, iterate in sorted order.


## WASM Pipeline Integration

### Changes to `wasm.rs`

The `serialise_result()` function currently builds the JSON response with `facts`. Add `property_index` alongside it:

```rust
fn serialise_result(result: crate::CompilationResult) -> String {
    let diags = serialise_diagnostics(&result.diagnostics);
    let facts_json = result.fact_set.as_ref().map(|fs| fs.to_json());
    let index_json = result.fact_set.as_ref().map(|fs| {
        let index = crate::facts::PropertyDependencyIndex::build(fs);
        index.to_json()
    });

    serde_json::json!({
        "success": result.success,
        "world": result.world,
        "diagnostics": diags,
        "facts": facts_json,
        "property_index": index_json,
    })
    .to_string()
}
```

**Index lifecycle:** The index is built once in the compilation pipeline, immediately after `extract_facts()`, and stored on `CompilationResult` alongside `fact_set`. Both `analyze()` and `serialise_result()` receive it — neither builds its own. This prevents the index from being computed twice and ensures all consumers see identical results.

This changes `analyze()`'s signature from `fn analyze(fact_set: &FactSet) -> Vec<Diagnostic>` to `fn analyze(fact_set: &FactSet, index: &PropertyDependencyIndex) -> Vec<Diagnostic>`. The caller controls index construction; the diagnostic module no longer builds its own.

### Changes to `compiler-bridge.ts`

Add TypeScript types for the index:

```typescript
export interface PropertyEntry {
  entity_type: string;
  property: string;
  read_count: number;
  write_count: number;
  read_indices: number[];
  write_indices: number[];
  orphaned: 'read_never_written' | 'written_never_read' | null;
}

export interface PropertyIndexSummary {
  total_properties: number;
  total_reads: number;
  total_writes: number;
  read_never_written: number;
  written_never_read: number;
}

export interface PropertyIndex {
  properties: PropertyEntry[];
  summary: PropertyIndexSummary;
}

export interface CompileResult {
  success: boolean;
  world: string | null;
  diagnostics: Diagnostic[];
  facts: FactSet | null;
  property_index: PropertyIndex | null;  // NEW
}
```


## Playground Component

### Design: `PropertyDependencyView.svelte`

A new Svelte component rendered inside the Analysis panel alongside (or as a tab within) the existing `FactSetView`. It provides a property-centric view of the world's data flow.

**Layout:**

1. **Summary header.** Total property count, total reads, total writes, orphaned counts. Matches the `summary` block from the JSON.

2. **Property list.** Each property key is a collapsible row showing:
   - `EntityType.property` label
   - Read count and write count badges
   - Orphaned flag (amber badge: "read only" or "write only") if applicable
   - When expanded: individual read sites and write sites with spans, clickable to scroll editor

3. **Filter controls.** Three filter buttons at the top:
   - **All** — show all properties (default)
   - **Read-only** — show only `read_never_written` properties
   - **Write-only** — show only `written_never_read` properties

The orphaned filters directly correspond to the SF-1A D1/D2 diagnostics. Clicking a row in "Read-only" mode highlights the same issue that D1 would flag.

### Integration into `UrdPlayground.svelte`

The Analysis panel currently shows only `FactSetView`. Add a tab bar inside the panel:

```
[Properties] [Facts]
```

- **Properties** tab renders `PropertyDependencyView` with `compileResult.property_index`
- **Facts** tab renders `FactSetView` with `compileResult.facts` (existing)
- Default tab: **Properties** (the more useful view for authors)

The tab state is local to the Analysis panel. No URL persistence needed.

### Clickable spans

Both the property-centric view and the existing fact-type view support clicking a row to scroll the editor to the corresponding span. The new component receives the same `onDiagnosticClick` callback.

**Collapsed row:** Each property row shows, without expanding: the `EntityType.property` label, read/write count badges, orphaned flag if applicable, and the line number of the first site (`L14`). The first-site line makes the list scannable without expanding rows — authors can see where a property first appears and click directly.

**Expanded row:** When expanded, individual read/write sites display:
- The site label (e.g., `choice: gatehouse/greet/ask-about-the-garden`)
- The operator and value (e.g., `>= 3` for reads, `= false` for writes)
- The file and line (e.g., `L14`)

Each site row is clickable via `onDiagnosticClick(span.start_line, span.start_col)`.

**Data resolution:** The `PropertyEntry` contains `read_indices` and `write_indices` which are indices into `facts.reads[]` and `facts.writes[]`. The component must receive both `property_index` and `facts` to resolve indices to display data (site labels, operators, values, spans). This is the same pattern `FactSetView` uses for choice `condition_reads` and `effect_writes`.


## Shared Source of Truth (SF-2.6)

After SF-2 is implemented, the SF-1A diagnostics D1 and D2 in `src/analyze.rs` must be refactored.

### Before (SF-1A implementation)

D1 and D2 compute set differences inline:

```rust
fn check_read_never_written(fact_set: &FactSet, index: &PropertyDependencyIndex) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for key in index.read_properties() {
        if index.writes_of(key).is_empty() {
            // ... emit URD601
        }
    }
    diagnostics
}
```

### After (SF-2 refactor)

D1 and D2 call the index's set-difference methods:

```rust
fn check_read_never_written(fact_set: &FactSet, index: &PropertyDependencyIndex) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for key in index.read_but_never_written() {
        let reads = index.reads_of(key);
        // ... emit URD601 using reads[0] as primary span
    }
    diagnostics
}
```

The logic is identical. The change is that the predicate (`writes_of(key).is_empty()`) moves into the index as a named method. This ensures the playground's "Read-only" filter and the D1 diagnostic always agree on which properties are orphaned.

**This refactor is mandatory.** If D1/D2 and the index disagree on which properties are orphaned, the playground will show one set of orphaned properties and the diagnostics panel will show a different set. That's a trust-destroying inconsistency.

### Verification

After the refactor, add a test that compiles a fixture, runs `analyze()`, and separately calls `index.read_but_never_written()` / `index.written_but_never_read()`. Assert that the diagnostic property keys exactly match the index results. This test is the proof of SF-2.6.


## Test Strategy

### New tests

| Test | Asserts |
|------|---------|
| `index_build_deterministic` | Building the index twice from the same FactSet produces identical results (same keys, same order, same indices) |
| `index_read_but_never_written_empty_on_clean` | On `negative-factset-diagnostics.urd.md`, `read_but_never_written()` returns empty |
| `index_written_but_never_read_empty_on_clean` | On `negative-factset-diagnostics.urd.md`, `written_but_never_read()` returns empty |
| `index_read_but_never_written_positive` | On `positive-factset-diagnostics.urd.md`, returns `NPC.suspicion` |
| `index_written_but_never_read_positive` | On `positive-factset-diagnostics.urd.md`, returns `NPC.loyalty` |
| `index_to_json_structure` | JSON output has `properties` array and `summary` block with correct counts |
| `index_to_json_orphaned_flags` | Orphaned fields set correctly: `null`, `read_never_written`, `written_never_read` |
| `index_all_fixtures_no_panic` | `PropertyDependencyIndex::build()` and `to_json()` run on all existing fixtures without panics |
| `index_matches_d1_d2` | For each fixture, `read_but_never_written()` keys match D1 diagnostic property keys; `written_but_never_read()` keys match D2 diagnostic property keys |
| `index_locked_garden_properties` | Locked Garden has expected property keys with expected read/write counts (regression anchor) |
| `index_sunken_citadel_summary` | Sunken Citadel summary counts are non-zero and reasonable (smoke test for the stress fixture) |

### Existing test reuse

The three SF-1A fixtures (`negative-factset-diagnostics.urd.md`, `positive-factset-diagnostics.urd.md`, `positive-factset-circular-deep.urd.md`) are reused for index tests. No new fixtures needed.

### Test file

Tests go in `tests/facts_tests.rs` alongside the existing FactSet extraction tests, in a new `// ── PropertyDependencyIndex ──` section.


## Files Changed

| File | Change |
|------|--------|
| `src/facts.rs` | Add `read_but_never_written()`, `written_but_never_read()`, `to_json()` on `PropertyDependencyIndex` |
| `src/wasm.rs` | Add `property_index` field to JSON output |
| `src/analyze.rs` | Refactor D1/D2 to call index set-difference methods (after SF-1A) |
| `tests/facts_tests.rs` | Add ~11 new tests for index correctness and JSON serialisation |
| `sites/urd.dev/src/components/playground/compiler-bridge.ts` | Add `PropertyIndex`, `PropertyEntry`, `PropertyIndexSummary` types; update `CompileResult` |
| `sites/urd.dev/src/components/playground/PropertyDependencyView.svelte` | **New.** Property-centric panel component |
| `sites/urd.dev/src/components/playground/UrdPlayground.svelte` | Add tab bar to Analysis panel; import and render `PropertyDependencyView` |


## Estimated Size

| Component | Lines |
|-----------|-------|
| `facts.rs` — two new methods | ~20 |
| `facts.rs` — `to_json()` | ~50 |
| `wasm.rs` — pipeline change | ~5 |
| `analyze.rs` — D1/D2 refactor | ~10 (net change, replacing inline logic with method calls) |
| `compiler-bridge.ts` — types | ~25 |
| `PropertyDependencyView.svelte` | ~200 |
| `UrdPlayground.svelte` — tab changes | ~30 |
| `facts_tests.rs` — new tests | ~150 |
| **Total** | **~490** |


## Acceptance Criteria

- [ ] **SF-2.1** — `PropertyDependencyIndex` has all six methods: `reads_of`, `writes_of`, `read_properties`, `written_properties`, `read_but_never_written`, `written_but_never_read`.
- [ ] **SF-2.2** — Index built from FactSet in a single pass via `PropertyDependencyIndex::build()`. No additional extraction or AST access.
- [ ] **SF-2.3** — Index is deterministic: same FactSet produces byte-identical `to_json()` output.
- [ ] **SF-2.4** — JSON serialisation implemented via `to_json()` and consumed by the playground WASM pipeline. `CompileResult` includes `property_index` field.
- [ ] **SF-2.5** — Playground Analysis panel displays property dependency data: per-property read/write counts, orphaned flags, expandable site lists with clickable spans.
- [ ] **SF-2.6** — `read_but_never_written()` and `written_but_never_read()` results exactly match SF-1A D1 and D2 diagnostic property keys. Verified by test: compile fixture, run `analyze()`, compare diagnostic keys to index method results.
- [ ] **SF-2.7** — Test coverage for index correctness against all existing test worlds (no panics, expected property counts on anchor fixtures).


## What This Brief Does NOT Cover

- Transitive dependency chains (A writes B which gates C which writes D). Consumers compose using direct lookups.
- Graph visualisation of property dependencies (that is SF-3).
- Semantic diff over property changes (that is SF-4).
- LSP hover data (that is SF-5, which consumes the index).
- MCP `get_property_dependencies` endpoint (that is SF-6).
- Extending the index with new query methods beyond the six specified. Future briefs add methods as needed.


## Relationship to Downstream Briefs

| Brief | How SF-2 feeds it |
|-------|--------------------|
| **SF-3** (Visualisation) | Graph node annotations use `reads_of` and `writes_of` to show property activity. Orphaned properties from `read_but_never_written` / `written_but_never_read` are visually flagged on nodes. |
| **SF-4** (Semantic Diff) | Property dependency changes detected by comparing two `PropertyDependencyIndex` instances: new reader, new writer, reader removed, writer removed. The `to_json()` output is the diffable artifact. |
| **SF-5** (LSP) | Hover on `@entity.property` returns read/write site counts and orphaned status from the index. Go-to-definition for property references uses `reads_of` / `writes_of` to find definition sites. |
| **SF-6** (MCP) | `get_property_dependencies(entity, property)` endpoint returns the index entry for a given property key. The JSON structure from `to_json()` is the response format. |

*End of Brief*
