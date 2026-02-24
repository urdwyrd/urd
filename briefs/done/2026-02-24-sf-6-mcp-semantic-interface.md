# URD — SF-6: Read-Only Semantic Interface (MCP)

*Expose the FactSet as a structured, read-only query surface for AI agents via the Model Context Protocol.*

February 2026 | Semantic Gate — Tier 3 (Instrument)

> **Document status: BRIEF** — An MCP server binary in a separate crate. Eight tools backed by FactSet, PropertyDependencyIndex, and compiled world JSON. Read-only. No runtime state. Validated by an LLM answering structural questions about a test world.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-24
**Status:** Done

### What was done

Implemented the `urd-mcp` crate in `packages/mcp/` with all eight read-only MCP tools backed by FactSet, PropertyDependencyIndex, and compiled world JSON. The crate compiles a `.urd.md` file on startup and serves tools over stdio using rmcp 0.8.

Created six source files:
- `Cargo.toml` — lib + bin targets, rmcp 0.8.5, schemars 1.x, tokio
- `src/lib.rs` — module declarations
- `src/main.rs` — CLI arg parsing, compile, start MCP stdio server
- `src/world_data.rs` — `WorldData` struct built from `CompilationResult` (~75 lines)
- `src/queries.rs` — eight pure query functions returning JSON with `schema_version: "1"` (~400 lines)
- `src/service.rs` — rmcp service with `#[tool_router]` and `#[tool_handler]`, eight tool definitions (~170 lines)

Test suite: 19 assertion-based tests in `tests/mcp_tests.rs` (all pass), plus 1 verifier sanity test and 1 `#[ignore]` LLM placeholder in `tests/llm_harness.rs`.

Added `mcp:build` and `mcp:test` scripts to root `package.json`.

### What changed from the brief

- **File structure:** The brief proposed `tools.rs` + `handlers.rs` for tool definitions and dispatch. The implementation uses a single `service.rs` with rmcp's `#[tool_router]` macro, which handles both definition and dispatch. This is simpler and more idiomatic for the rmcp SDK.
- **schemars version:** The brief assumed schemars 0.8; rmcp 0.8.5 depends on schemars 1.x. Used schemars 1.x directly.
- **Section IDs:** The brief assumed section IDs use `location/section` (e.g., `gatehouse/greet`). The compiler uses `filestem/section` (e.g., `locked-garden/greet`). Tests corrected accordingly.
- **20 tests not 19:** The LLM harness includes a non-ignored `verifiers_accept_reference_answers` test, bringing the total to 20 non-ignored tests.
- **SF-6.4 deferred:** The LLM validation run requires API credentials and an MCP-capable client. The harness skeleton is complete (5 questions, verifiers, reference answer validation), but the actual LLM run is deferred to manual execution.

---

## Context

AI agents can currently interact with Urd worlds only through compiled JSON — a flat, denormalised document designed for runtimes, not for reasoning. To answer "which properties gate the garden exit?" an agent must parse the full JSON, understand the schema, and reconstruct relationships manually.

SF-6 creates a structured query surface: eight MCP tools that answer structural questions directly. An agent asks `get_property_dependencies("Guard", "trust")` and receives a typed response with read sites, write sites, and orphan status. No JSON parsing. No schema knowledge. No hallucination about world structure.

### Current state

- The FactSet contains all structural relationships (exits, jumps, choices, rules, property reads/writes)
- PropertyDependencyIndex provides property-level dependency analysis
- Compiled world JSON contains entity details, type definitions, location metadata
- Diagnostics contain compiler errors, warnings, and FactSet-derived novel diagnostics (SF-1A)
- No MCP server, no structured query API, no AI-agent-facing interface

### What this brief delivers

Three things:

1. **An MCP server binary.** `urd-mcp`, in a separate crate (`packages/mcp/`). Communicates via stdin/stdout using JSON-RPC 2.0 per the MCP specification. Compiles a `.urd.md` file on startup. Serves queries against the compiled result.

2. **Eight MCP tools.** Each tool takes typed parameters and returns structured JSON. All read-only. No mutation. No runtime state.

3. **An LLM validation harness.** Five structural questions about the Locked Garden test world, each with a known correct answer. Run against an LLM with MCP access. Scored pass/fail. Minimum 4/5 pass (SF-6.4).


## Deviations from Gate Doc

1. **Implementation as MCP tools, not resources.** The gate doc says "MCP-compatible endpoints." MCP defines two primitive types: *tools* (callable with parameters, return results) and *resources* (named data, read via URI). All eight endpoints accept parameters (entity ID, property key, starting location, section ID). Tools are the correct MCP abstraction for parameterised queries. Resources would be appropriate for static data like "the full world JSON" but not for computed queries.

2. **Compilation model.** Gate doc doesn't specify when compilation happens. This brief compiles once at startup. The MCP server receives a file path as a command-line argument, compiles it, and serves queries against the result. Recompilation requires restarting the server. Watch mode (recompile on file change, push updated data) is a future enhancement — the MCP spec supports notifications for this, but it adds complexity beyond the foundation.


## Dependencies

- **SF-1A passed.** Diagnostics D1–D5 available for `get_diagnostics`.
- **SF-2 passed.** PropertyDependencyIndex available for `get_property_dependencies`.
- **SF-5 DefinitionIndex (optional).** If SF-5 has shipped, the DefinitionIndex provides declaration spans that enrich `get_entity_details` responses. If not, entity details come from compiled world JSON only — still sufficient for the gate.

SF-6 does NOT depend on SF-3 (visualisation), SF-4 (diff), or SF-5 (LSP). It can be implemented in parallel with any of them.


## Architecture

### Crate structure

```
packages/
  compiler/             — existing, unchanged
  mcp/                  — NEW crate
    Cargo.toml
    src/
      main.rs           — entry point, compilation, MCP server startup
      tools.rs          — tool definitions (name, description, parameter schema)
      handlers.rs       — tool call dispatch → query functions
      queries.rs        — pure functions: (WorldData, params) → JSON result
      world_data.rs     — compiled state container
    tests/
      mcp_tests.rs      — assertion-based endpoint tests
      llm_harness.rs    — LLM validation test (SF-6.4)
```

### Dependency boundary

Same principle as SF-5: the MCP crate depends on `urd-compiler` but does not import AST, parser, linker, validator, or emitter modules.

```toml
# packages/mcp/Cargo.toml
[dependencies]
urd-compiler = { path = "../compiler" }
rmcp = { version = "0.8", features = ["server"] }
tokio = { version = "1", features = ["rt", "macros"] }
serde_json = "1"
serde = { version = "1", features = ["derive"] }
```

The MCP crate imports from `urd-compiler`:
- `urd_compiler::compile` — triggers compilation
- `urd_compiler::CompilationResult` — result container
- `urd_compiler::facts::*` — FactSet, PropertyDependencyIndex, ExitEdge, JumpEdge, ChoiceFact, RuleFact, PropertyRead, PropertyWrite, PropertyKey
- `urd_compiler::diagnostics::*` — Diagnostic, Severity
- `urd_compiler::span::Span` — source positions

The MCP crate does NOT import:
- `urd_compiler::ast::*`
- `urd_compiler::parse::*`
- `urd_compiler::link::*`
- `urd_compiler::validate::*`
- `urd_compiler::emit::*`
- `urd_compiler::symbol_table::*`
- `urd_compiler::graph::*`

### Why `rmcp`?

`rmcp` is the official Rust MCP SDK from the Model Context Protocol organisation. It handles JSON-RPC framing, tool registration, parameter validation, and protocol handshake. The alternative — hand-rolling JSON-RPC over stdio — is ~200 lines but risks protocol drift and edge-case bugs. `rmcp` requires tokio, which is acceptable for a standalone binary. The async runtime stays contained in this crate.

### WorldData

The query surface operates against a pre-compiled world. All data is loaded at startup and immutable for the server's lifetime.

```rust
struct WorldData {
    /// Compiled world JSON (parsed).
    world_json: serde_json::Value,
    /// Full FactSet from compilation.
    fact_set: FactSet,
    /// Property dependency analysis.
    property_index: PropertyDependencyIndex,
    /// All diagnostics (errors, warnings, info).
    diagnostics: Vec<DiagnosticEntry>,
}
```

`DiagnosticEntry` is a serialisable struct flattened from the compiler's `Diagnostic`:

```rust
struct DiagnosticEntry {
    severity: String,
    code: String,
    message: String,
    file: String,
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
}
```


## Startup Flow

```
$ urd-mcp locked-garden.urd.md
```

1. Parse command-line argument: file path.
2. Compile with `urd_compiler::compile(path)`.
3. If compilation produces errors: still start the server. `get_diagnostics` reports errors. Other tools return partial data or error responses depending on what's available (FactSet may exist even with validation errors).
4. Build `WorldData` from `CompilationResult`.
5. Start MCP server on stdin/stdout.
6. Serve tool calls until client disconnects.


## Tool Definitions

### Tool 1: `get_world_metadata`

**Parameters:** None.

**Returns:**
```json
{
  "world_name": "the-locked-garden",
  "start_location": "gatehouse",
  "entity_count": 5,
  "location_count": 2,
  "type_count": 3,
  "section_count": 3,
  "exit_count": 2,
  "rule_count": 0,
  "has_errors": false,
  "diagnostic_count": 0
}
```

**Source:** World JSON (`world.name`, `world.start`), FactSet (counts derived from exit/jump/choice/rule vectors), diagnostics. `world_version` included only if the world defines one.

### Tool 2: `get_exit_graph`

**Parameters:** None.

**Returns:**
```json
{
  "nodes": ["gatehouse", "the-walled-garden"],
  "edges": [
    {
      "from": "gatehouse",
      "to": "the-walled-garden",
      "exit_name": "garden",
      "is_conditional": true,
      "guard_count": 1
    },
    {
      "from": "the-walled-garden",
      "to": "gatehouse",
      "exit_name": "north",
      "is_conditional": false,
      "guard_count": 0
    }
  ]
}
```

**Source:** Nodes from compiled world JSON `locations` keys (guarantees isolated locations appear). Edges from FactSet `exits()`.

### Tool 3: `get_dialogue_graph`

**Parameters:** None.

**Returns:**
```json
{
  "sections": ["gatehouse/greet", "the-walled-garden/explore", "the-walled-garden/revelation"],
  "jumps": [
    {
      "from_section": "gatehouse/greet",
      "to_section": "gatehouse/greet",
      "type": "section"
    },
    {
      "from_section": "the-walled-garden/explore",
      "to_section": "the-walled-garden/revelation",
      "type": "section"
    }
  ],
  "choices": [
    {
      "section": "gatehouse/greet",
      "choice_id": "gatehouse/greet/ask-about-the-garden",
      "label": "Ask about the garden",
      "sticky": true,
      "condition_count": 1,
      "effect_count": 1
    }
  ]
}
```

**Source:** Sections from compiled world JSON (all sections defined in the world, including those with no jumps or choices — consistent with SF-3's node derivation). Jumps from FactSet `jumps()`. Choices from FactSet `choices()`. Example shows a subset — full response includes all sections, jumps, and choices.

### Tool 4: `get_entity_details`

**Parameters:** `{ "entity_id": "@warden" }`

**Returns:**
```json
{
  "entity_id": "@warden",
  "type": "Character",
  "container": "gatehouse",
  "properties": [
    {
      "name": "mood",
      "type": "Enum",
      "default": "neutral",
      "values": ["wary", "neutral", "friendly"]
    },
    {
      "name": "trust",
      "type": "Integer",
      "default": 0
    },
    {
      "name": "role",
      "type": "String",
      "visibility": "hidden"
    }
  ]
}
```

**Source:** World JSON `entities[entity_id]` and `types[type_name]`. Properties include type-level defaults and any entity-level overrides.

**Error:** If `entity_id` not found, return `{ "error": "Entity not found", "entity_id": "@warden" }`.

### Tool 5: `get_property_dependencies`

**Parameters:** `{ "entity_type": "Character", "property": "trust" }`

**Returns:**
```json
{
  "property_key": "Character.trust",
  "read_count": 1,
  "write_count": 3,
  "read_only": false,
  "write_only": false,
  "reads": [
    { "site": "choice:gatehouse/greet/ask-about-the-garden", "comparison": "trust >= 3" }
  ],
  "writes": [
    { "site": "choice:gatehouse/greet/state-your-purpose", "operation": "trust += 1" },
    { "site": "choice:gatehouse/greet/offer-the-journal", "operation": "trust += 5" },
    { "site": "choice:gatehouse/greet/force-the-gate", "operation": "trust -= 2" }
  ]
}
```

**Source:** PropertyDependencyIndex `reads_of()` and `writes_of()`, dereferenced through FactSet `reads()` and `writes()` for detail. Site descriptions derived from FactSet's `resolve_site()`. `read_only` = has reads, zero writes. `write_only` = has writes, zero reads. Comparison/operation strings reconstructed from PropertyRead's `operator`/`value_literal` and PropertyWrite's `operator`/`value_expr`.

**Error:** If property key not found, return `{ "error": "Property not found", "property_key": "Character.trust" }`.

### Tool 6: `get_reachable_locations`

**Parameters:** `{ "from": "gatehouse" }`

**Returns:**
```json
{
  "from": "gatehouse",
  "reachable": ["gatehouse", "the-walled-garden"],
  "unreachable": [],
  "path_count": 2,
  "paths": {
    "the-walled-garden": ["gatehouse", "the-walled-garden"]
  }
}
```

**Source:** BFS over FactSet `exits()`. From the starting location, follow ExitEdges to reachable locations. Record shortest path (deterministic: alphabetical tie-breaking) for each. Location universe = compiled world JSON `locations` keys. `unreachable` = universe minus reachable set. The starting location is always included in `reachable`.

**Note:** This BFS ignores exit conditions. All exits are traversable regardless of guards. The garden exit from gatehouse is conditional (`@garden_gate.locked == false`), but BFS treats it as traversable. A "conditional reachability" variant that respects guards is a future enhancement.

**Error:** If `from` location not found, return `{ "error": "Location not found", "location": "unknown" }`.

### Tool 7: `get_choice_conditions`

**Parameters:** `{ "section": "gatehouse/greet" }`

**Returns:**
```json
{
  "section": "gatehouse/greet",
  "choices": [
    {
      "choice_id": "gatehouse/greet/state-your-purpose",
      "label": "State your purpose",
      "sticky": false,
      "conditions": [],
      "effects": [
        { "property": "Character.trust", "operation": "+= 1" }
      ]
    },
    {
      "choice_id": "gatehouse/greet/ask-about-the-garden",
      "label": "Ask about the garden",
      "sticky": true,
      "conditions": [
        { "property": "Character.trust", "comparison": ">= 3" }
      ],
      "effects": [
        { "property": "Lock.locked", "operation": "= false" }
      ]
    }
  ]
}
```

**Source:** FactSet `choices()` filtered by section. Condition reads and effect writes dereferenced through FactSet `reads()` and `writes()` by index. Example shows a subset — the real section has 4 choices.

**Error:** If section not found, return `{ "error": "Section not found", "section": "unknown" }`.

### Tool 8: `get_diagnostics`

**Parameters:** None. (Optional filter: `{ "severity": "warning" }` or `{ "file": "gatehouse.urd.md" }`.)

**Returns:**
```json
{
  "total": 0,
  "errors": 0,
  "warnings": 0,
  "info": 0,
  "diagnostics": []
}
```

With a world that has issues:
```json
{
  "total": 2,
  "errors": 0,
  "warnings": 2,
  "info": 0,
  "diagnostics": [
    {
      "severity": "warning",
      "code": "URD430",
      "message": "Location 'crypt' is unreachable. No path from the start location reaches it.",
      "file": "dungeon.urd.md",
      "start_line": 45,
      "start_col": 1,
      "end_line": 45,
      "end_col": 8
    }
  ]
}
```

**Source:** `WorldData.diagnostics`. Optionally filtered by severity or file.


## MCP Tool Descriptors (SF-6.3)

Each tool is registered with a name, description, and JSON Schema for parameters. This is what the AI agent sees when it calls `tools/list`.

```json
{
  "tools": [
    {
      "name": "get_world_metadata",
      "description": "Returns overview information about the compiled Urd world: name, version, start location, and counts of entities, locations, types, sections, exits, and rules.",
      "inputSchema": { "type": "object", "properties": {} }
    },
    {
      "name": "get_exit_graph",
      "description": "Returns the complete location exit graph as nodes (locations) and edges (exits with direction, destination, and condition information). Use this to understand spatial navigation between locations.",
      "inputSchema": { "type": "object", "properties": {} }
    },
    {
      "name": "get_dialogue_graph",
      "description": "Returns the dialogue structure graph: sections (dialogue nodes), jumps between sections, and choices within sections including their labels, conditions, and effects.",
      "inputSchema": { "type": "object", "properties": {} }
    },
    {
      "name": "get_entity_details",
      "description": "Returns detailed information about a specific entity: its type, container location, and all properties with types, defaults, and constraints. Entity IDs start with '@'.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "entity_id": { "type": "string", "description": "Entity ID (e.g., '@warden')" }
        },
        "required": ["entity_id"]
      }
    },
    {
      "name": "get_property_dependencies",
      "description": "Returns where a specific property is read and written across the world. Shows all condition sites (reads) and effect sites (writes) with their expressions. Useful for understanding how a property influences game logic.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "entity_type": { "type": "string", "description": "Type name (e.g., 'Guard')" },
          "property": { "type": "string", "description": "Property name (e.g., 'trust')" }
        },
        "required": ["entity_type", "property"]
      }
    },
    {
      "name": "get_reachable_locations",
      "description": "Returns which locations are reachable from a starting location by following exits (ignoring conditions). Includes shortest paths. Useful for understanding world connectivity and finding isolated areas.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "from": { "type": "string", "description": "Starting location slug (e.g., 'gatehouse')" }
        },
        "required": ["from"]
      }
    },
    {
      "name": "get_choice_conditions",
      "description": "Returns all choices in a dialogue section with their conditions (property reads) and effects (property writes). Useful for understanding what gates player options and what consequences they have.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "section": { "type": "string", "description": "Section compiled ID (e.g., 'gatehouse/greet')" }
        },
        "required": ["section"]
      }
    },
    {
      "name": "get_diagnostics",
      "description": "Returns all compiler diagnostics (errors, warnings, info) for the compiled world. Optionally filter by severity or file.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "severity": { "type": "string", "description": "Filter by severity: 'error', 'warning', or 'info'" },
          "file": { "type": "string", "description": "Filter by source file name" }
        }
      }
    }
  ]
}
```

### Tool description quality

Tool descriptions are critical for SF-6.4. The LLM agent must be able to select the right tool for each question based on the description alone. Descriptions explain *what* the tool returns and *when to use it*. They avoid implementation details. This is the primary lever for improving the LLM's success rate.


## Query Functions

All queries are pure functions: `fn query_*(data: &WorldData, params) -> serde_json::Value`. No side effects. No mutation. No state. This makes them trivially testable.

```rust
// queries.rs — all pure functions

pub fn get_world_metadata(data: &WorldData) -> Value { ... }
pub fn get_exit_graph(data: &WorldData) -> Value { ... }
pub fn get_dialogue_graph(data: &WorldData) -> Value { ... }
pub fn get_entity_details(data: &WorldData, entity_id: &str) -> Value { ... }
pub fn get_property_dependencies(data: &WorldData, entity_type: &str, property: &str) -> Value { ... }
pub fn get_reachable_locations(data: &WorldData, from: &str) -> Value { ... }
pub fn get_choice_conditions(data: &WorldData, section: &str) -> Value { ... }
pub fn get_diagnostics(data: &WorldData, severity: Option<&str>, file: Option<&str>) -> Value { ... }
```

The handler layer (`handlers.rs`) extracts parameters from MCP tool call requests, calls the appropriate query function, and wraps the result in an MCP response. ~60 lines.


## Test Strategy

### Assertion-based endpoint tests (SF-6.5)

Each query function tested against the Locked Garden world and other test fixtures. Pure function tests — no MCP transport involved.

| Test | Asserts |
|------|---------|
| `query_world_metadata` | Returns correct world name, start location, counts |
| `query_exit_graph_nodes` | Node list matches all locations in test world |
| `query_exit_graph_edges` | Edge count and attributes match FactSet exits |
| `query_dialogue_graph_sections` | Section list matches FactSet sections |
| `query_dialogue_graph_choices` | Choice count and attributes match FactSet choices |
| `query_entity_details_found` | `@warden` returns Character type with correct properties |
| `query_entity_details_not_found` | Unknown entity returns error response |
| `query_property_deps_trust` | Character.trust returns expected read/write counts and sites |
| `query_property_deps_not_found` | Unknown property returns error response |
| `query_reachable_from_gatehouse` | BFS from gatehouse returns expected reachable set |
| `query_reachable_unreachable` | Unreachable locations correctly identified |
| `query_reachable_not_found` | Unknown location returns error response |
| `query_choice_conditions_greet` | `gatehouse/greet` returns expected choices with conditions |
| `query_choice_conditions_not_found` | Unknown section returns error response |
| `query_diagnostics_all` | Returns all diagnostics with correct counts |
| `query_diagnostics_filter_severity` | Filtering by severity returns subset |
| `query_diagnostics_filter_file` | Filtering by file returns subset |
| `mcp_tool_descriptors_valid` | All 8 tool descriptors have name, description, and valid inputSchema |
| `mcp_import_boundary` | MCP crate source files do not import prohibited compiler modules |

19 tests. The first 17 are pure-function tests that run without MCP transport. The last two validate MCP integration and crate boundaries.

### LLM validation harness (SF-6.4)

Five structural questions about the Locked Garden world, each with a known correct answer derived from the FactSet.

| # | Question | Expected answer | Verification |
|---|----------|-----------------|--------------|
| 1 | "What condition must be met to travel from the Gatehouse to the Walled Garden?" | Lock.locked must be false (@garden_gate.locked == false) | Lock.locked or garden_gate.locked mentioned with "false" |
| 2 | "Which entities are of type Character?" | @warden, @ghost | Both entity IDs present, no extras |
| 3 | "What are all the ways @warden.trust can change in the gatehouse?" | +1 (State your purpose), +5 (Offer the journal), -2 (Force the gate) | All three operations present with correct values |
| 4 | "What effects does choosing 'Offer the journal' have?" | trust +5, mood = friendly | Both effects present |
| 5 | "How can the garden gate be unlocked?" | Choose "Ask about the garden" when trust >= 3, sets locked = false | Trust threshold (3) and locked = false both present |

**Harness implementation:**

```rust
// llm_harness.rs — not a unit test, runs as a separate binary or integration test

struct Question {
    prompt: String,
    verifier: fn(llm_response: &str) -> bool,
}

fn run_harness(mcp_server_path: &str, world_file: &str, questions: &[Question]) -> (usize, usize) {
    // 1. Start urd-mcp as a subprocess with world_file
    // 2. Connect an LLM client with MCP tool access to the subprocess
    // 3. For each question:
    //    a. Send question to LLM
    //    b. LLM calls MCP tools as needed
    //    c. LLM produces a final answer
    //    d. Run verifier against the answer
    // 4. Return (passed, total)
}
```

**Verifiers** are substring/pattern matchers, not exact string comparisons. Question 1 passes if the response mentions "locked" and "false" or "garden_gate.locked". Question 5 passes if the response mentions "trust" with "3" and "locked" with "false".

**LLM choice:** The harness is LLM-agnostic. It can run against Claude (via API), GPT, or any model with MCP client support. The brief does not mandate a specific model. The gate requires 4/5 pass with at least one model.

**Not a CI test.** The LLM harness requires API credentials and incurs cost. It runs manually as a validation step, not on every commit. Results are recorded in the execution record.


## Response Schema Versioning (SF-6.6)

Every tool response includes a schema version:

```json
{
  "schema_version": "1",
  "world_name": "Locked Garden",
  ...
}
```

If the response format changes (fields added, removed, or renamed), the schema version increments. Consumers can check `schema_version` before parsing. This is a simple, pragmatic approach — no JSON Schema validation at runtime, just a version marker for compatibility detection.


## Files Changed

### MCP crate

| File | Change |
|------|--------|
| `packages/mcp/Cargo.toml` | **New.** Crate manifest |
| `packages/mcp/src/main.rs` | **New.** CLI argument parsing, compilation, MCP server startup |
| `packages/mcp/src/tools.rs` | **New.** 8 tool definitions with descriptions and input schemas |
| `packages/mcp/src/handlers.rs` | **New.** Tool call dispatch: extract params → call query → wrap response |
| `packages/mcp/src/queries.rs` | **New.** 8 pure query functions |
| `packages/mcp/src/world_data.rs` | **New.** WorldData struct, construction from CompilationResult |
| `packages/mcp/tests/mcp_tests.rs` | **New.** 19 assertion-based tests |
| `packages/mcp/tests/llm_harness.rs` | **New.** LLM validation harness (5 questions, verifiers) |

### Compiler crate

No changes. SF-6 consumes existing public API only.


## Estimated Size

| Component | Lines |
|-----------|-------|
| `main.rs` — CLI + startup + MCP server | ~80 |
| `tools.rs` — 8 tool definitions | ~120 |
| `handlers.rs` — dispatch + param extraction | ~80 |
| `queries.rs` — 8 query functions | ~350 |
| `world_data.rs` — WorldData construction | ~60 |
| `mcp_tests.rs` — 19 tests | ~400 |
| `llm_harness.rs` — 5 questions + verifiers | ~120 |
| `Cargo.toml` + crate boilerplate | ~20 |
| **Total** | **~1230** |

The bulk is query functions (~350 lines) and tests (~520 lines). The MCP integration layer is thin (~280 lines).


## Acceptance Criteria

- [ ] **SF-6.1** — All eight tools implemented and returning structured JSON.
- [ ] **SF-6.2** — Tools are read-only: no tool accepts mutation parameters or modifies compiled state.
- [ ] **SF-6.3** — MCP tool descriptors provided for each tool (name, description, parameter schema). Tool descriptions are sufficient for an LLM to select the correct tool for a given question.
- [ ] **SF-6.4** — LLM validation harness: 5 predefined structural questions about Locked Garden, scored pass/fail. Minimum 4/5 pass with at least one LLM model.
- [ ] **SF-6.5** — All 8 query functions tested against Locked Garden and existing test worlds with assertion-based tests. 19 tests pass.
- [ ] **SF-6.6** — Response format includes `schema_version: "1"` field. Versioning documented.
- [ ] **SF-6.7** — No tool requires or references runtime state, compiled JSON execution data, or the AST. Enforced by import boundary test.


## Design Decisions

### Why MCP tools, not resources?

MCP resources are identified by URIs and return static content. MCP tools accept parameters and return computed results. All eight endpoints are parameterised queries (even the no-parameter ones return computed aggregates, not raw data). Tools are the correct abstraction.

If a future consumer wants "give me the raw compiled world JSON," that would be a resource. The query surface is tools.

### Why compile-once, not watch mode?

Watch mode (recompile on file change, notify MCP client) requires:
- File system watcher
- MCP notification protocol for data invalidation
- State management for mid-query recompilation

None of this is needed for the gate. An AI agent reasoning about a world needs a consistent snapshot, not a moving target. Compile once, query many. Restart to recompile.

Watch mode is a natural enhancement when MCP clients support server-initiated notifications.

### Why `rmcp` instead of hand-rolling JSON-RPC?

MCP's JSON-RPC protocol includes content-length framing, capability negotiation, tool listing, and error handling. The official `rmcp` SDK handles all of this correctly. Hand-rolling saves one dependency but risks protocol edge cases. The tokio requirement is acceptable because `urd-mcp` is a standalone binary — the async runtime doesn't leak into the compiler crate.

If `rmcp`'s API changes or tokio is rejected, the fallback is `async-mcp` (lightweight, minimal dependencies) or hand-rolled JSON-RPC (~200 lines).

### Why not embed in the LSP?

SF-5's LSP server could serve MCP tools as well — it already has the compiled world state. But:
- Different consumers. The LSP serves editors. MCP serves AI agents.
- Different lifecycles. The LSP recompiles on save. MCP serves a frozen snapshot.
- Different protocols. LSP ≠ MCP. Combining them in one binary adds protocol multiplexing complexity.
- Different deployment. An author runs the LSP in their editor. An AI agent runs urd-mcp as a subprocess.

Separate binaries, shared data sources.

### BFS ignores exit conditions — why?

`get_reachable_locations` traverses all exits regardless of guards. This is correct for structural reachability ("can a player ever reach this location?") but not for state-dependent reachability ("can a player reach this location given current property values?"). State-dependent reachability requires a runtime — exactly what the next gate (Wyrd) provides. The structural BFS is what the AI agent needs for reasoning about world topology.

### How does the LLM validation harness handle non-determinism?

LLM responses are non-deterministic. The harness uses:
- Temperature 0 (or as low as the API allows) for reproducibility
- Substring/pattern verifiers, not exact match
- 4/5 threshold, not 5/5, to allow for one marginal answer

If the harness fails on first run, the response is to improve tool descriptions (the primary lever), not to tune the LLM. Good tool descriptions make the agent's job trivial.


## What This Brief Does NOT Cover

- **Runtime state queries.** `get_state`, `perform_action`, `simulate_choice` — all deferred to the runtime gate.
- **Write/mutation tools.** No tool modifies the compiled world. MCP tools for editing `.urd.md` files are a future extension.
- **Watch mode.** Recompile on file change + push notifications.
- **Authentication.** The MCP server runs locally as a subprocess. No auth needed.
- **Multi-world support.** One compiled world per server instance.
- **Streaming responses.** All responses are single JSON objects. Streaming large graphs is a future enhancement.
- **VS Code MCP integration.** Wiring urd-mcp into VS Code's MCP client is a future extension.


## Relationship to Downstream Briefs

| Brief | How SF-6 feeds it |
|-------|--------------------|
| **Runtime gate (Wyrd)** | The MCP server extends with runtime tools: `get_state`, `perform_action`, `simulate_choice`. The query functions become the "static world" portion of the runtime's MCP surface. |
| **SF-5** (LSP) | The DefinitionIndex (if available from SF-5) enriches entity detail responses with declaration spans. Not required for the gate. |
| **AI-assisted authoring** | An AI agent with MCP access can reason about world structure, suggest improvements, detect issues, and generate content. SF-6 is the foundation for this. |

*End of Brief*
