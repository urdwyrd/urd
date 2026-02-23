# Semantic Gate

> **Document status: TRACKING, DEVELOPMENT GATE**
> Defines the briefs, acceptance criteria, and verification requirements for the semantic-first phase. No brief is complete until every acceptance criterion is verified. No downstream brief should begin until its dependencies are met.
> Single canonical copy. February 2026.

---

## Context

The compiler gate is closed. 554 tests. The FactSet IR — six relation types extracted in a single deterministic pass after compilation — is the foundation for everything in this phase.

The semantic gate covers the work between the compiler and the runtime: derived diagnostics, dependency analysis, visualisation, semantic diffing, LSP foundations, and external query interfaces. Every brief in this gate builds on the FactSet. Every acceptance criterion validates that the FactSet's design is sufficient for the queries we are promising.

The runtime (Wyrd) is explicitly out of scope for this gate. It is next.

---

## Gate structure

Seven briefs. Three tiers.

| Tier | Purpose | Briefs |
|------|---------|--------|
| **Prove** | Validate that the FactSet supports real work | SF-1A, SF-1B |
| **Expose** | Make semantics inspectable, visible, and comparable | SF-2, SF-3, SF-4 |
| **Operationalise** | Deliver semantics into tools and external systems | SF-5, SF-6 |

Dependencies are strict. Each brief lists its prerequisites. No brief should begin implementation until its dependencies have passed their acceptance criteria.

---

## Dependency graph

```
SF-1A ──→ SF-1B
  │
  ├──→ SF-2 ──→ SF-3
  │              │
  │              ├──→ SF-4
  │              │
  │              └──→ SF-5
  │
  └──────────────────→ SF-6
```

SF-1A gates everything. SF-2 (PropertyDependencyIndex) is the second critical path item. SF-3, SF-4, SF-5 can be parallelised after SF-2 if resources allow. SF-6 depends only on SF-1A and SF-2.

---

## SF-1A: Novel FactSet Diagnostics

**Purpose:** Prove that the FactSet enables diagnostics that are impossible or impractical without it. This is the validation gate for the entire FactSet design.

**Dependencies:** Compiler gate closed. FactSet extraction working.

**Scope:**

Implement a minimum of five new diagnostic codes. Each must operate solely on the FactSet's six tuple types. No AST traversal. No source text parsing. The diagnostic function signature is `fn check(factset: &FactSet) -> Vec<Diagnostic>`.

| ID | Diagnostic | FactSet query | Severity |
|----|-----------|---------------|----------|
| D1 | Property read but never written | PropertyRead keys minus PropertyWrite keys | Warning |
| D2 | Property written but never read | PropertyWrite keys minus PropertyRead keys | Warning |
| D3 | Effect produces enum variant that no condition ever tests | PropertyWrite values minus PropertyRead comparison values, filtered by enum type | Warning |
| D4 | Choice condition references property threshold unreachable by any effect | PropertyRead thresholds vs PropertyWrite value ranges | Warning |
| D5 | Circular property dependency: property written only behind a condition that reads the same property at a threshold no other effect can reach | PropertyWrite sites cross-referenced with their enclosing ChoiceFact/RuleFact condition_reads, checked against all other PropertyWrite values for the same key | Warning |

**Acceptance criteria:**

- [ ] **SF-1A.1** — All five diagnostics implemented as functions taking `&FactSet`, returning `Vec<Diagnostic>`
- [ ] **SF-1A.2** — No diagnostic implementation imports or references any AST type, parser module, or source text
- [ ] **SF-1A.3** — Each diagnostic fires correctly on a purpose-built test world (minimum one positive and one negative case per diagnostic)
- [ ] **SF-1A.4** — Diagnostics surface in standard compiler output with structured JSON format
- [ ] **SF-1A.5** — At least one diagnostic catches a real issue in an existing test world (Sunken Citadel, Locked Garden, or Intercept adaptation) that no existing S1–S8 check detects
- [ ] **SF-1A.6** — If any diagnostic is awkward to express against the six tuple types, the gap is documented as a FactSet schema issue for resolution before SF-2

**Validation gate:** SF-1A.5 is the critical item. If the new diagnostics do not find anything the old checks missed, the FactSet has not proved its value over AST traversal.

---

## SF-1B: Migrate Existing Static Analysis to FactSet

**Purpose:** Eliminate duplicate AST traversal in existing S3–S8 checks by rewriting them as FactSet queries. Reduces compiler maintenance surface and validates FactSet completeness for known-good queries.

**Dependencies:** SF-1A passed. (If SF-1A reveals FactSet schema gaps, those must be resolved first.)

**Scope:**

Rewrite static analysis checks S3 through S8 to operate on the FactSet. The existing checks and their FactSet equivalents:

| Check | Current implementation | FactSet equivalent |
|-------|----------------------|-------------------|
| S3 — Unreachable location | AST walk over exits | BFS over ExitEdge tuples |
| S4 — Unreachable dialogue section | AST walk over jumps | BFS over JumpEdge tuples |
| S5 — Unused entity | AST walk over references | Entity ID not present in any tuple |
| S6 — Shadowed entity ID | AST walk | (may remain AST-based if shadowing is a parse-time concern) |
| S7 — Choice without effect | AST walk over choices | ChoiceFact with empty effect_writes index |
| S8 — Rule with unsatisfiable trigger | AST walk over rules | RuleFact condition_reads cross-referenced against PropertyWrite |

**Acceptance criteria:**

- [ ] **SF-1B.1** — S3, S4, S5, S7, S8 reimplemented as FactSet queries (S6 exempted if shadowing is parse-time only)
- [ ] **SF-1B.2** — All existing static analysis tests continue to pass with identical diagnostics
- [ ] **SF-1B.3** — Old AST-traversal implementations removed (not just bypassed)
- [ ] **SF-1B.4** — No regression in compiler performance (static analysis phase should be equal or faster)
- [ ] **SF-1B.5** — If S6 cannot migrate, the reason is documented

**Validation gate:** SF-1B.2 is the critical item. Zero diagnostic regression.

---

## SF-2: PropertyDependencyIndex

**Purpose:** Formalise the read/write index over entity properties. This is the foundation for explain mode, cache invalidation, visualisation annotations, and the semantic diff engine.

**Dependencies:** SF-1A passed.

**Scope:**

Build and ship a `PropertyDependencyIndex` that provides direct lookups by property key, with first-class query methods.

**API surface (minimum):**

| Method | Returns |
|--------|---------|
| `readers_of(entity, property)` | All PropertyRead sites for this property |
| `writers_of(entity, property)` | All PropertyWrite sites for this property |
| `read_properties()` | Set of all property keys that appear in any condition |
| `written_properties()` | Set of all property keys that appear in any effect |
| `read_but_never_written()` | Set difference: read minus written |
| `written_but_never_read()` | Set difference: written minus read |

Transitive dependency chains (A writes B which gates C which writes D) are explicitly out of scope. Consumers that need transitive traversal compose using the direct lookups.

**Serialisation:**

The index must serialise to JSON for the WASM pipeline. The playground analysis panel must display property dependency data from the serialised index.

**Acceptance criteria:**

- [ ] **SF-2.1** — `PropertyDependencyIndex` struct implemented with all six methods above
- [ ] **SF-2.2** — Index built from FactSet in a single pass, after extraction
- [ ] **SF-2.3** — Index is deterministic: same FactSet produces byte-identical index
- [ ] **SF-2.4** — JSON serialisation implemented and consumed by playground WASM pipeline
- [ ] **SF-2.5** — Playground analysis panel displays property dependency data (read sites, write sites, orphaned properties)
- [ ] **SF-2.6** — `read_but_never_written()` and `written_but_never_read()` results match SF-1A diagnostic D1 and D2 output exactly (shared source of truth, not duplicated logic)
- [ ] **SF-2.7** — Test coverage for index correctness against all existing test worlds

**Validation gate:** SF-2.4 and SF-2.5 together. The index must be visible in the browser, not just correct in Rust tests.

---

## SF-3: Location and Dialogue Graph Visualisation

**Purpose:** Render the FactSet's structural relationships as interactive graphs. This brief is also a validation gate for FactSet completeness — if the graph cannot be fully reconstructed from FactSet tuples alone, the extraction is incomplete.

**Dependencies:** SF-2 passed. (The PropertyDependencyIndex provides property-level annotations on graph nodes and edges.)

**Scope:**

Two Svelte island components, both consuming FactSet JSON from the WASM pipeline. No AST access. No source text parsing.

**Component 1: Location topology**

- Locations as nodes
- ExitEdge tuples as directed edges
- Conditional exits visually distinguished (dotted, coloured, annotated with condition summary)
- Unreachable locations highlighted (cross-reference with S3 diagnostic)
- Click on node shows contained entities and property summary from PropertyDependencyIndex

**Component 2: Dialogue flow**

- Sections as nodes
- JumpEdge tuples as directed edges
- ChoiceFact tuples rendered as labelled edges from section to target
- Sticky vs one-shot choices visually distinguished
- Unreachable sections highlighted (cross-reference with S4 diagnostic)
- Click on section shows choices and their condition/effect summaries

**Acceptance criteria:**

- [ ] **SF-3.1** — Location graph fully reconstructed from ExitEdge tuples only. No AST fallback. No source text access
- [ ] **SF-3.2** — Dialogue graph fully reconstructed from JumpEdge and ChoiceFact tuples only. No AST fallback
- [ ] **SF-3.3** — No "unknown" or "unresolved" nodes in either graph for any test world that passes compilation
- [ ] **SF-3.4** — Conditional edges display condition summary derived from PropertyRead data
- [ ] **SF-3.5** — Unreachable nodes visually marked and consistent with S3/S4 diagnostic output
- [ ] **SF-3.6** — Both components render correctly for all existing test worlds (Locked Garden, Monty Hall, Key Puzzle, Sunken Citadel, Intercept)
- [ ] **SF-3.7** — Components deployed to playground alongside existing analysis panel
- [ ] **SF-3.8** — If any graph element cannot be reconstructed from FactSet tuples, the missing data is documented as a FactSet schema gap

**Validation gate:** SF-3.1, SF-3.2, and SF-3.3 together. The FactSet must be the sole data source. Any failure here indicates an extraction gap that must be resolved before downstream briefs proceed.

---

## SF-4: Semantic Diff

**Purpose:** Compare two FactSet snapshots and report structural changes at the semantic level. Enables version control for narrative meaning, regression detection, and CI integration.

**Dependencies:** SF-2 passed. (PropertyDependencyIndex provides the property-level data needed for dependency change detection.)

**Scope:**

A diff engine that takes two serialised FactSet instances and produces a typed change report. Not a text diff. Not a JSON diff. A semantic diff over the six relation types.

**Change types:**

| Category | Detects |
|----------|---------|
| Entity changes | Entity added, removed, type changed, property default changed |
| Location topology | Exit added, removed, condition changed, target changed |
| Dialogue structure | Section added, removed. Jump target changed. Choice added, removed, guard changed |
| Property dependencies | New reader of property, new writer of property, reader removed, writer removed |
| Rule changes | Rule added, removed, trigger changed, effect changed |
| Reachability | Location became unreachable, location became reachable. Section became unreachable, section became reachable |

**Output format:**

Structured JSON. Each change entry includes: change type, affected element identifiers, before value (if applicable), after value (if applicable).

**Acceptance criteria:**

- [ ] **SF-4.1** — Diff engine implemented taking two `FactSet` instances (or serialised JSON) as input
- [ ] **SF-4.2** — All six change categories above detected and reported
- [ ] **SF-4.3** — Output is structured JSON with typed change entries
- [ ] **SF-4.4** — Diffing a FactSet against itself produces an empty change report
- [ ] **SF-4.5** — Adding a location to a test world and recompiling produces exactly the expected change entries (exit added, entity placement, reachability change)
- [ ] **SF-4.6** — Removing a property write and recompiling produces a dependency change (writer removed) and, if applicable, a diagnostic change (property now read-but-never-written)
- [ ] **SF-4.7** — Diff is deterministic: same two inputs produce byte-identical output regardless of invocation order (A→B and A→B again, not A→B vs B→A which is expected to differ)
- [ ] **SF-4.8** — CLI command or API available for CI integration: `urd diff <file1.urd.json> <file2.urd.json>`
- [ ] **SF-4.9** — Identity is defined by element ID (entity ID, location name, section label, rule name). Renamed elements are reported as removed + added, not as renames. This is a deliberate design choice, documented in the brief

**Validation gate:** SF-4.5 and SF-4.6 together. The diff must catch both structural changes (new location) and dependency changes (property write removed).

---

## SF-5: LSP Foundation

**Purpose:** Embed the compiler in watch mode with Language Server Protocol support. Transitions Urd from a batch compiler to a language with real-time feedback.

**Dependencies:** SF-2 passed. SF-3 passed. (The LSP wires diagnostics from SF-1A/1B, and provides go-to-definition and hover data that depend on the PropertyDependencyIndex. SF-3 validates FactSet completeness, which the LSP depends on.)

**Scope:**

Minimal LSP server. Four capabilities. Architectural constraint: wire to FactSet where possible, not directly to AST internals.

| Capability | Data source | Description |
|------------|-------------|-------------|
| Diagnostics | Compiler output + SF-1A/1B diagnostics | Real-time error and warning streaming on file save |
| Go-to-definition | FactSet resolved cross-references | Click `@entity` → jump to entity definition. Click `-> section` → jump to section label |
| Hover | PropertyDependencyIndex + FactSet | Hover `@entity.property` → show type, default, read/write sites. Hover section label → show incoming jumps |
| Autocomplete | FactSet entity/property index | Type `@` → entity list. Type `@entity.` → property list for that entity type |

**Explicitly out of scope:** Syntax highlighting (handled by TextMate grammar, separate deliverable). Rename refactoring. Code actions. Workspace symbol search. VS Code extension packaging (separate brief).

**Acceptance criteria:**

- [ ] **SF-5.1** — LSP server binary that communicates via stdin/stdout using LSP protocol
- [ ] **SF-5.2** — Compiler embedded in watch mode: file change triggers recompilation and diagnostic push
- [ ] **SF-5.3** — Diagnostics include all compiler errors, warnings, and SF-1A/1B FactSet diagnostics
- [ ] **SF-5.4** — Go-to-definition works for `@entity` references and `-> section` jumps
- [ ] **SF-5.5** — Hover on `@entity.property` shows type, default value, and read/write site count
- [ ] **SF-5.6** — Autocomplete triggers on `@` and `@entity.` with correct candidates
- [ ] **SF-5.7** — All four capabilities tested against a mock LSP client with request/response assertions
- [ ] **SF-5.8** — Go-to-definition and hover data sourced from FactSet, not from direct AST node references. Enforced via crate dependency boundary: LSP crate must not depend on parser or AST modules
- [ ] **SF-5.9** — Response latency under 200ms for a file the size of Sunken Citadel on recompilation

**Validation gate:** SF-5.7. The mock client test suite is the proof. Manual VS Code testing is supplementary, not the gate.

---

## SF-6: Read-Only Semantic Interface (MCP)

**Purpose:** Expose the FactSet as a structured, read-only query surface for external consumers. Primary use case: AI agents that can reason about world structure without simulating execution.

**Dependencies:** SF-1A passed. SF-2 passed. (Requires working FactSet and PropertyDependencyIndex. Does not require visualisation, diff, or LSP.)

**Scope:**

MCP-compatible endpoints that serve FactSet data as structured JSON responses. Read-only. No mutation. No runtime state. The runtime endpoints (get_state, perform_action) are explicitly deferred to the runtime gate.

**Endpoints:**

| Endpoint | Returns |
|----------|---------|
| `get_world_metadata` | World name, version, start location, entity count, location count |
| `get_exit_graph` | All ExitEdge tuples as a directed graph (nodes + edges with conditions) |
| `get_dialogue_graph` | All JumpEdge and ChoiceFact tuples as a directed graph |
| `get_entity_details(id)` | Entity type, properties with defaults, container, current location |
| `get_property_dependencies(entity, property)` | Read sites, write sites, from PropertyDependencyIndex |
| `get_reachable_locations(from)` | BFS over ExitEdge graph from a starting location |
| `get_choice_conditions(section)` | All ChoiceFact entries for a section with their condition reads |
| `get_diagnostics` | All compiler and FactSet diagnostics for the compiled world |

**Acceptance criteria:**

- [ ] **SF-6.1** — All eight endpoints implemented and returning structured JSON
- [ ] **SF-6.2** — Endpoints are read-only: no endpoint accepts mutation parameters
- [ ] **SF-6.3** — MCP tool/resource descriptors provided for each endpoint (name, description, parameter schema, return schema)
- [ ] **SF-6.4** — A fixed test harness of five predefined structural questions about a test world (e.g., "what is the shortest path from Gatehouse to Walled Garden?", "which properties gate the garden_gate exit?"), each with a known correct answer derived from the FactSet, run against an LLM with MCP endpoints available, scored pass/fail on factual correctness. Minimum 4/5 pass
- [ ] **SF-6.5** — Endpoints tested against all existing test worlds with assertion-based tests
- [ ] **SF-6.6** — Response format is stable and documented: endpoint schemas versioned
- [ ] **SF-6.7** — No endpoint requires or references runtime state, compiled JSON execution data, or the AST

**Validation gate:** SF-6.4. An AI agent must be able to reason about a world using only these endpoints. If the agent cannot answer basic structural questions, the query surface is insufficient.

---

## Gate completion

The semantic gate is closed when all seven briefs have passed their acceptance criteria and all validation gates are verified.

**Gate summary checklist:**

- [ ] **SF-1A** — Novel FactSet diagnostics proved (5 diagnostics, real issue found)
- [ ] **SF-1B** — Existing static analysis migrated to FactSet (zero diagnostic regression)
- [ ] **SF-2** — PropertyDependencyIndex shipped and visible in playground
- [ ] **SF-3** — Graphs reconstruct from FactSet only (no AST fallback, no unknown nodes)
- [ ] **SF-4** — Semantic diff detects structural and dependency changes
- [ ] **SF-5** — LSP server passes mock client test suite (diagnostics, go-to-def, hover, autocomplete)
- [ ] **SF-6** — AI agent answers structural questions using MCP endpoints only

**Post-gate:** The runtime gate (Wyrd) begins. The FactSet, PropertyDependencyIndex, diagnostics, and MCP surface all feed into the runtime's design. The semantic diff becomes the regression test primitive for runtime development. The LSP extends to include runtime state inspection. The visualisation extends to include live state overlay.

The queryable world is complete. The executable world is next.
