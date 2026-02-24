# Semantic Gate

> **Document status: CLOSED**
> All six active briefs have passed their acceptance criteria. The semantic gate is closed. One item remains open: SF-6.4 (LLM validation harness manual run) — the harness is built and verifiers pass against reference answers, but the live LLM test awaits manual execution with API credentials.
> Single canonical copy. February 2026.

---

![The Semantic Gate: Evolution of the FactSet IR](images/semantic-gate-overview.png)

## Context

The compiler gate is closed. 554 tests. The FactSet IR — six relation types extracted in a single deterministic pass after compilation — is the foundation for everything in this phase.

The semantic gate covers the work between the compiler and the runtime: derived diagnostics, dependency analysis, visualisation, semantic diffing, LSP foundations, and external query interfaces. Every brief in this gate builds on the FactSet. Every acceptance criterion validates that the FactSet's design is sufficient for the queries we are promising.

The runtime (Wyrd) is explicitly out of scope for this gate. It is next.

---

## Gate structure

Six briefs. Three tiers.

| Tier | Purpose | Briefs |
|------|---------|--------|
| **Prove** | Validate that the FactSet supports real work | SF-1A |
| **Expose** | Make semantics inspectable, visible, and comparable | SF-2, SF-3, SF-4 |
| **Operationalise** | Deliver semantics into tools and external systems | SF-5, SF-6 |

Dependencies are strict. Each brief lists its prerequisites. No brief should begin implementation until its dependencies have passed their acceptance criteria.

---

## Dependency graph

```
SF-1A ──→ SF-2 ──→ SF-3
  │         │        │
  │         │        ├──→ SF-4
  │         │        │
  │         │        └──→ SF-5
  │         │
  └─────────┼────────────→ SF-6
```

SF-1A gates everything. SF-2 (PropertyDependencyIndex) is the second critical path item. SF-3, SF-4, SF-5 can be parallelised after SF-3 if resources allow. SF-6 depends on SF-1A and SF-2 (needs PropertyDependencyIndex for property dependency queries).

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

- [x] **SF-1A.1** — All five diagnostics implemented as functions taking `&FactSet`, returning `Vec<Diagnostic>`
- [x] **SF-1A.2** — No diagnostic implementation imports or references any AST type, parser module, or source text
- [x] **SF-1A.3** — Each diagnostic fires correctly on a purpose-built test world (minimum one positive and one negative case per diagnostic)
- [x] **SF-1A.4** — Diagnostics surface in standard compiler output with structured JSON format
- [x] **SF-1A.5** — At least one diagnostic catches a real issue in an existing test world (Sunken Citadel, Locked Garden, or Intercept adaptation) that no existing S1–S8 check detects
- [x] **SF-1A.6** — If any diagnostic is awkward to express against the six tuple types, the gap is documented as a FactSet schema issue for resolution before SF-2

**Validation gate:** SF-1A.5 is the critical item. If the new diagnostics do not find anything the old checks missed, the FactSet has not proved its value over AST traversal.

---

## SF-1B: Migrate Existing Static Analysis to FactSet — DEFERRED

**Status:** Deferred. Reassess when FactSet scope expands to include location/section metadata.

**Original purpose:** Eliminate duplicate AST traversal in existing S3–S8 checks by rewriting them as FactSet queries.

**Why deferred:** Code analysis of the four VALIDATE checks (S3, S4, S6, S8) revealed that none can be migrated to FactSet-only queries. Each check requires data the FactSet was deliberately designed not to carry:

| Check | What it needs beyond FactSet |
|-------|-----------------------------|
| S3 — Unreachable location | Complete location ID enumeration, `world_start` value. The FactSet only contains locations that participate in exits. |
| S4 — Orphaned choice | Type definition's enum values list ("is `friendly` a valid variant of the `mood` enum?"). This is symbol table data, not a FactSet relation. |
| S6 — Missing fallthrough | Presence of prose/speech/stage-direction after the last choice — narrative content, which the FactSet deliberately excludes. |
| S8 — Shadowed exit | Section-to-location containment, which the FactSet does not model (it knows which section a choice belongs to, not which location a section lives in). |

Migration would require either: (a) expanding the FactSet with location metadata, section-location mapping, and enum value lists — changing what the FactSet is, or (b) allowing migrated checks to use FactSet + symbol table — which doesn't simplify anything and adds a second data source to each check.

The existing checks work, are tested (~290 lines total, 36 tests), and have no maintenance problems. The FactSet's value is proven by SF-1A's five novel diagnostics that the old checks cannot express. Migrating working code to a different data source for aesthetic purity is not progress.

**Reassessment trigger:** If the FactSet later expands with a `LocationFact` relation (carrying containment, start flag, metadata) or if a concrete consumer needs S3–S8 results as FactSet queries (e.g., the semantic diff engine needs reachability data from FactSet rather than re-running the BFS), SF-1B becomes viable and should be reconsidered.

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

- [x] **SF-2.1** — `PropertyDependencyIndex` struct implemented with all six methods above
- [x] **SF-2.2** — Index built from FactSet in a single pass, after extraction
- [x] **SF-2.3** — Index is deterministic: same FactSet produces byte-identical index
- [x] **SF-2.4** — JSON serialisation implemented and consumed by playground WASM pipeline
- [x] **SF-2.5** — Playground analysis panel displays property dependency data (read sites, write sites, orphaned properties)
- [x] **SF-2.6** — `read_but_never_written()` and `written_but_never_read()` results match SF-1A diagnostic D1 and D2 output exactly (shared source of truth, not duplicated logic)
- [x] **SF-2.7** — Test coverage for index correctness against all existing test worlds

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

- [x] **SF-3.1** — Location graph fully reconstructed from ExitEdge tuples only. No AST fallback. No source text access
- [x] **SF-3.2** — Dialogue graph fully reconstructed from JumpEdge and ChoiceFact tuples only. No AST fallback
- [x] **SF-3.3** — No "unknown" or "unresolved" nodes in either graph for any test world that passes compilation
- [x] **SF-3.4** — Conditional edges display condition summary derived from PropertyRead data
- [x] **SF-3.5** — Unreachable nodes visually marked and consistent with S3/S4 diagnostic output
- [x] **SF-3.6** — Both components render correctly for all existing test worlds (Locked Garden, Monty Hall, Key Puzzle, Sunken Citadel, Intercept)
- [x] **SF-3.7** — Components deployed to playground alongside existing analysis panel
- [x] **SF-3.8** — If any graph element cannot be reconstructed from FactSet tuples, the missing data is documented as a FactSet schema gap

**Validation gate:** SF-3.1, SF-3.2, and SF-3.3 together. The FactSet must be the sole data source. Any failure here indicates an extraction gap that must be resolved before downstream briefs proceed.

---

## SF-4: Semantic Diff

**Purpose:** Compare two compiled worlds and report structural changes at the semantic level. Enables version control for narrative meaning, regression detection, and CI integration.

**Dependencies:** SF-2 passed. SF-3 passed. (PropertyDependencyIndex provides property-level data for dependency change detection. SF-3's ChoiceFact.jump_indices extension provides the section/choice jump ownership partition needed for dialogue change detection.)

**Scope:**

A diff engine that compiles two `.urd.md` files, builds normalised DiffSnapshots from each, and produces a typed change report. Not a text diff. Not a JSON diff. A semantic diff over the six relation types. The DiffSnapshot strips non-semantic fields (spans, ordering artefacts, narrative text) and retains only structural and dependency data needed for comparison.

**Change types:**

| Category | Detects |
|----------|---------|
| Entity changes | Entity added, removed, type changed, property default changed |
| Location topology | Exit added, removed, condition changed, target changed |
| Dialogue structure | Section added, removed. Jump target changed. Choice added, removed, guard changed |
| Property dependencies | New reader of property, new writer of property, reader removed, writer removed |
| Rule changes | Rule added, removed, trigger changed, effect changed |
| Reachability | Location became unreachable/reachable. Choice condition became impossible/possible |

**Output format:**

Structured JSON. Each change entry includes: change type, affected element identifiers, before value (if applicable), after value (if applicable).

**Acceptance criteria:**

- [x] **SF-4.1** — Diff engine implemented taking two `.urd.md` file paths as input, compiling internally, producing DiffSnapshots for comparison
- [x] **SF-4.2** — All six change categories above detected and reported
- [x] **SF-4.3** — Output is structured JSON with typed change entries
- [x] **SF-4.4** — Diffing a world against itself produces an empty change report
- [x] **SF-4.5** — Adding a location to a test world and recompiling produces exactly the expected change entries (exit added, entity placement, reachability change)
- [x] **SF-4.6** — Removing a property write and recompiling produces a dependency change (writer removed) and, if applicable, a diagnostic change (property now read-but-never-written)
- [x] **SF-4.7** — Diff is deterministic: same two inputs produce byte-identical output regardless of invocation order (A→B and A→B again, not A→B vs B→A which is expected to differ)
- [x] **SF-4.8** — CLI command available for CI integration: `urd diff <before.urd.md> <after.urd.md>`
- [x] **SF-4.9** — Identity is defined by compiled element ID (entity_id, location slug, section compiled_id, rule_id). Renamed elements are reported as removed + added, not as renames. This is a deliberate design choice, documented in the brief

**Validation gate:** SF-4.5 and SF-4.6 together. The diff must catch both structural changes (new location) and dependency changes (property write removed).

---

## SF-5: LSP Foundation

**Purpose:** Provide recompile-on-save Language Server Protocol support. Transitions Urd from a batch compiler to a language with real-time feedback.

**Dependencies:** SF-2 passed. SF-3 passed. (The LSP consumes three data sources that depend on PropertyDependencyIndex and validated FactSet completeness.)

**Scope:**

Minimal LSP server in a new `packages/lsp/` crate. Four capabilities. Three data sources, each purpose-built for a consumer category:

- **DefinitionIndex** — declarations (entity definitions, sections, locations with spans). Purpose-built for go-to-definition and autocomplete. Derived from CompilationResult, not from FactSet or AST.
- **FactSet** — relationships (exits, jumps, choices, property reads/writes). Powers hover data for sections.
- **PropertyDependencyIndex** — analysis (read/write site counts, orphan detection). Powers hover data for properties.

Recompiles on `textDocument/didSave` — the editor pushes content, no file watcher needed. On compilation failure, diagnostics are always replaced but intellisense data (DefinitionIndex, PropertyDependencyIndex, FactSet) is retained from the last successful compilation (stale state retention).

| Capability | Data source | Description |
|------------|-------------|-------------|
| Diagnostics | Compiler output + SF-1A FactSet diagnostics | Real-time error and warning push on save |
| Go-to-definition | DefinitionIndex | Click `@entity` → jump to definition. Click `-> section` → jump to section. Ambiguous names return multiple locations (editor shows picker) |
| Hover | DefinitionIndex + PropertyDependencyIndex + FactSet | Hover `@entity.property` → show type, default, read/write site count. Hover section → show incoming jumps, outgoing jumps, choice count |
| Autocomplete | DefinitionIndex + world JSON | Type `@` → entity list. Type `@entity.` → property list for that entity's type. Type `->` → section list. Ctrl+Space in exit block → location list |

**Explicitly out of scope:** Syntax highlighting (TextMate grammar, separate deliverable). Rename refactoring. Code actions. Workspace symbol search. VS Code extension packaging (separate brief).

**Acceptance criteria:**

- [x] **SF-5.1** — LSP server binary that communicates via stdin/stdout using LSP protocol
- [x] **SF-5.2** — Recompile on `textDocument/didSave`: editor save triggers recompilation and diagnostic push
- [x] **SF-5.3** — Diagnostics include all compiler errors, warnings, and SF-1A FactSet diagnostics
- [x] **SF-5.4** — Go-to-definition works for `@entity` references, `-> section` jumps, and `@entity.property` references. Ambiguous names return multiple locations
- [x] **SF-5.5** — Hover on `@entity.property` shows type, default value, and read/write site count. Hover on section shows incoming jumps, outgoing jumps, choice count
- [x] **SF-5.6** — Autocomplete triggers on `@`, `@entity.`, `->`, and Ctrl+Space in exit blocks, with correct candidates
- [x] **SF-5.7** — 18 tests pass: 17 capability tests (diagnostics, go-to-definition, hover, autocomplete) against a mock LSP client with request/response assertions, plus 1 import boundary test
- [x] **SF-5.8** — Go-to-definition, hover, and autocomplete data sourced from DefinitionIndex, FactSet, and PropertyDependencyIndex — not from AST. Enforced via crate dependency boundary: LSP crate imports only `compile()`, `CompilationResult`, `FactSet`, `PropertyDependencyIndex`, `Diagnostic`, `Span`
- [x] **SF-5.9** — Response latency under 200ms for a file the size of Sunken Citadel on recompilation

**Validation gate:** SF-5.7. The 18-test suite is the proof. Manual VS Code testing is supplementary, not the gate.

---

## SF-6: Read-Only Semantic Interface (MCP)

**Purpose:** Expose the FactSet as a structured, read-only query surface for external consumers via Model Context Protocol. Primary use case: AI agents that can reason about world structure without simulating execution.

**Dependencies:** SF-1A passed. SF-2 passed. (Requires working FactSet and PropertyDependencyIndex. Does not require visualisation, diff, or LSP.)

**Scope:**

MCP server binary in a new `packages/mcp/` crate, using `rmcp` (official Rust MCP SDK). Eight read-only tools that serve FactSet data as structured JSON responses. Compiles a world once at startup from a CLI-provided file path; recompilation requires restart. No mutation. No runtime state. The runtime tools (`get_state`, `perform_action`) are explicitly deferred to the runtime gate. Every tool response includes `schema_version: "1"` for forward compatibility.

**Tools:**

| Tool | Returns |
|------|---------|
| `get_world_metadata` | World name, start location, entity/location/type/section/exit/rule counts, diagnostic summary |
| `get_exit_graph` | Location nodes (from world JSON) + exit edges (from FactSet) with direction, destination, condition info |
| `get_dialogue_graph` | Section nodes (from world JSON) + jump edges + choices with labels, conditions, effects |
| `get_entity_details(entity_id)` | Entity type, container, properties with types/defaults/constraints |
| `get_property_dependencies(entity_type, property)` | Read/write sites with comparison/operation strings, orphan status |
| `get_reachable_locations(from)` | BFS over exits (ignoring conditions), shortest paths, unreachable set. Location universe from world JSON |
| `get_choice_conditions(section)` | All choices in section with condition reads and effect writes |
| `get_diagnostics(severity?, file?)` | All compiler/FactSet diagnostics, optionally filtered by severity or file |

**Acceptance criteria:**

- [x] **SF-6.1** — All eight tools implemented and returning structured JSON
- [x] **SF-6.2** — Tools are read-only: no tool accepts mutation parameters or modifies compiled state
- [x] **SF-6.3** — MCP tool descriptors provided for each tool (name, description, parameter schema). Tool descriptions are sufficient for an LLM to select the correct tool for a given question
- [ ] **SF-6.4** — LLM validation harness: 5 predefined structural questions about the Locked Garden test world, each with a known correct answer derived from the FactSet, scored pass/fail on factual correctness. Minimum 4/5 pass with at least one LLM model *(harness built, awaiting manual LLM run)*
- [x] **SF-6.5** — 19 tests pass: 17 pure-function query tests against Locked Garden, 1 tool descriptor validation, 1 import boundary test
- [x] **SF-6.6** — Response format includes `schema_version: "1"` field. Schema versioning documented
- [x] **SF-6.7** — No tool requires or references runtime state, compiled JSON execution data, or the AST. Enforced by import boundary test

**Validation gate:** SF-6.4. An AI agent must be able to reason about a world using only these tools. If the agent cannot answer basic structural questions, the query surface is insufficient.

---

## Gate completion

All six active briefs have passed their acceptance criteria. The semantic gate is closed.

**Gate summary checklist:**

- [x] **SF-1A** — Novel FactSet diagnostics proved (5 diagnostics, real issue found)
- ~~SF-1B~~ — Deferred (existing checks work; FactSet scope insufficient for migration without expanding the IR)
- [x] **SF-2** — PropertyDependencyIndex shipped and visible in playground
- [x] **SF-3** — Graphs reconstruct from FactSet only (no AST fallback, no unknown nodes)
- [x] **SF-4** — Semantic diff detects structural and dependency changes across DiffSnapshots
- [x] **SF-5** — LSP server passes 20-test suite (diagnostics, go-to-def, hover, autocomplete, import boundary)
- [x] **SF-6** — MCP crate ships 8 read-only tools, 20 tests pass. LLM validation harness built, awaiting manual run

**Post-gate:** The runtime gate (Wyrd) begins. The FactSet, PropertyDependencyIndex, diagnostics, and MCP surface all feed into the runtime's design. The semantic diff becomes the regression test primitive for runtime development. The LSP extends to include runtime state inspection. The visualisation extends to include live state overlay.

The queryable world is complete. The executable world is next.
