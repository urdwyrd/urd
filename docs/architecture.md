---
title: "Architecture & Technical Design"
slug: "architecture"
description: "Four components connected by one contract. The compiler (5-phase pipeline). The Wyrd reference runtime (immutable state, event sourcing, deterministic rules). The testing framework (Monte Carlo, reachability, coverage). Developer tooling (LSP server, VS Code extension)."
category: "architecture"
format: "System Blueprint"
date: "2026-02"
status: "v1.0 complete"
order: 1
tags:
  - architecture
  - compiler
  - runtime
  - testing
  - tooling
details:
  - "Four-component pipeline: compiler, runtime, testing, tooling"
  - "Five-phase compiler: parse → resolve → link → validate → emit"
  - "Immutable state with event sourcing in the runtime"
  - "Build order and dependency graph for implementation"
---

> **Document status: NORMATIVE**
> Defines the system architecture: component boundaries, interfaces, data flow, and build order. This is the authoritative blueprint for engineering implementation.
> Single canonical copy. February 2026 draft.

# URD

## Architecture and Technical Design

*System blueprint: components, boundaries, data flow, and build order*

urd.dev

February 2026

## Purpose

This document describes the high level architecture of the Urd framework: what the components are, what each one consumes and produces, how they connect, and what order they should be built in. It is a blueprint, not an implementation guide. Each component will have its own detailed design document when engineering begins.

The reader should be familiar with two prior documents:

- **Urd World Schema Specification**, which defines the data contract (types, entities, containment, visibility, expressions, effects).
- **Urd Schema Markdown Syntax Specification**, which defines the writer facing syntax that compiles to the schema.

## System Overview

Urd is a pipeline, not a monolith. Content flows through a chain of independent components, each with a defined input, output, and responsibility. No component knows the internals of any other. They communicate through two interchange formats: Schema Markdown source files and compiled JSON.

### The Pipeline

```
┌─────────────────┐
│   Writer's      │
│   .urd.md       │
│   files         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    COMPILER     │
│  parse → link   │
│  → validate     │
│  → emit         │
└────┬──────┬─────┘
     │      │
┌────┘      └────────┐
▼                    ▼
┌───────────────┐  ┌───────────────┐
│  .urd.json    │  │  Diagnostics  │
│  (compiled)   │  │  (errors,     │
│               │  │   warnings)   │
└────┬────┬─────┘  └───────────────┘
     │    │
┌────┘    └───────┐
▼                 ▼
┌─────────────┐  ┌─────────────┐
│   RUNTIME   │  │   TESTING   │
│  executes   │  │  validates  │
│  the world  │  │  the world  │
└─────────────┘  └─────────────┘
```

The four components:

| Component | Input | Output | Who Uses It |
|-----------|-------|--------|-------------|
| Compiler | `.urd.md` source files | `.urd.json` compiled world + diagnostics | Build pipelines, CI, editors |
| Runtime (Wyrd) | `.urd.json` compiled world | Interactive session (state, events, output) | Game engines, text runners, web demos |
| Testing Framework | `.urd.json` compiled world + test definitions | Pass/fail results, coverage reports | CI, developers, QA |
| Developer Tooling | `.urd.md` source files (live) | Diagnostics, completions, navigation | Writers and engineers in editors |

> **The critical boundary:** The `.urd.json` file is the contract between authoring and execution. The compiler produces it. The runtime consumes it. The testing framework validates it. No component on the execution side ever sees `.urd.md` source. No component on the authoring side needs to know how the runtime works.

## Component 1: The Compiler

The compiler transforms Schema Markdown source files into a single, validated, self contained JSON world file. It is the most critical component. Everything downstream depends on its correctness.

### Responsibilities

- **Parse** Schema Markdown source into an AST (abstract syntax tree). Handle frontmatter, imports, headings, choices, conditions, effects, sections, entity references, and dialogue attribution.
- **Resolve imports.** Follow `import: ./path` declarations, load referenced files, merge their types and entities into the compilation scope. **Import rules:** imports are explicit and non transitive (a file only sees what it directly imports); circular imports are a compile error with a diagnostic showing the cycle; duplicate entity IDs across imported files are a compile error (no silent merging); duplicate type names are a compile error.
- **Link references.** Resolve every `@entity` reference, `->` section jump, and property access against the declared types and entities. Every reference must resolve to something real.
- **Validate types.** Check that entity property values match their declared types, that enum values are in the declared set, that refs point to entities of the correct type, that conditions reference properties that exist.
- **Emit JSON.** Produce a single `.urd.json` file conforming to the Urd World Schema. The urd version field is set automatically by the compiler to `"1"`. Authors do not set this manually; if they do, the compiler warns and overrides. All imports resolved, all references validated, all sections compiled into the dialogue block.
- **Report diagnostics.** Produce structured error and warning messages with file path, line number, and actionable suggestions. Good diagnostics are as important as correct compilation.

### Compiler Phases

```
.urd.md files
     │
  1. PARSE       Source text → per-file ASTs
     │
  2. IMPORT      Resolve imports, build dependency graph
     │
  3. LINK        Merge scopes, resolve cross-file references
     │
  4. VALIDATE    Type-check properties, conditions, effects
     │           - For each action: exactly zero or one of {target, target_type}
     │             must be present. Neither = self-targeted. Both = compile error.
     │           - Validate that condition expressions reference existing properties.
     │           - Validate that effect targets are valid entities or containers.
     │
  5. EMIT        AST → .urd.json
     │
.urd.json + diagnostics
```

### Key Design Decisions

**Single pass vs multi pass.** The compiler needs at least two passes: one to collect all type and entity declarations (including from imports), and one to validate references and emit. Forward references within a file (an action referencing an entity declared later) must work. A two pass architecture with a collection pass and a validation/emit pass is sufficient for v1.

**Error recovery.** The compiler should not stop at the first error. It should collect as many errors as possible in a single run so the writer can fix multiple issues at once. This means the parser must be able to skip malformed blocks and continue, and the validator must handle missing references gracefully (report the error, mark the reference as unresolved, continue checking).

**Incremental compilation strategy.** Full recompile on every change is acceptable for the initial implementation, but the architecture must not preclude incremental compilation because the LSP needs it from Phase 3 onward. The following decisions are locked now to avoid costly retrofits:

- **File dependency graph.** The compiler maintains a directed graph of file imports. When a file changes, only that file and its transitive dependents need recompilation. The graph is built during the collection pass and cached between compiles.
- **Stable entity and section IDs.** Compiled IDs are derived from declared names and file paths, not from declaration order. Specifically: entity IDs are the declared `@name` and must be globally unique; section IDs are `file_stem/section_name` (e.g., `tavern/topics` for `== topics` in `tavern.urd.md`); choice IDs are `section_id/slugified_label` (e.g., `tavern/topics/ask-about-the-harbor`). This means a recompile of one file does not invalidate references from unchanged files, and tooling (LSP, editor, save files) can maintain stable pointers across recompiles. The compiler MUST detect duplicate entity IDs across the entire compilation unit (entry file + all imports) and emit a diagnostic listing all declarations of the conflicting ID with their file paths and line numbers.
- **Cache invalidation.** When a type definition changes, all entities of that type are revalidated. When an entity changes, all references to it are rechecked. The import graph provides the invalidation boundary. The initial compiler may recompile everything, but it must build the graph so that future versions can use it for incremental compilation.

**Source maps.** The compiled JSON should include optional source map data linking each schema element back to its source file and line number. This enables the runtime and testing framework to report errors in terms of the original `.urd.md` source, not the compiled JSON. Not strictly required for v1, but the data model should leave room for it.

### Interface

```
// Conceptual interface, language agnostic

compile(entryFile: path) → {
  success: boolean,
  world: UrdWorldJSON | null,
  diagnostics: Diagnostic[],
}

Diagnostic {
  severity: error | warning | info,
  file: path,
  line: number,
  column: number,
  message: string,
  code: string,             // Machine-readable (e.g. URD001)
  suggestion: string | null, // "Did you mean @guard.mood?"
}
```

## Component 2: The Reference Runtime (Wyrd)

The runtime loads a compiled `.urd.json` world and executes it. It manages world state, evaluates conditions, applies effects, resolves player actions, and exposes the current state to whatever presentation layer is attached (text console, web UI, game engine).

The reference runtime is called **Wyrd**. It runs in the browser (TypeScript compiled to JavaScript) and in Node.js for headless testing. The Wyrd Reference Runtime document specifies its architecture, API, deployment contexts, and build passes in detail.

> **"Reference" means canonical, not minimal.** This is the runtime that defines correct behavior. If a third party engine integration disagrees with the reference runtime, the reference runtime is right. It does not need to be high performance, but it must be correct and complete.

### Responsibilities

- **Load and instantiate.** Parse the `.urd.json` file, create entity instances with default property values, place entities in their declared containers.
- **Manage world state.** Maintain the current state of every entity property, every entity's container, every section's visited/exhausted status, and the current sequence phase (if any).
- **Evaluate conditions.** Given the current world state, evaluate any condition expression to true or false. This includes property comparisons, containment checks (`entity.container == X`), and exhaustion checks (`? <section_name>.exhausted`). **Exhaustion is runtime evaluated:** there is no generated boolean field in the compiled JSON. The compiled JSON contains an `on_exhausted` field with fallthrough content, but whether a section *is* exhausted is a predicate the runtime computes. The runtime checks all choices in the named section and returns true if none are currently available (all one shot choices consumed, all remaining choices gated by false conditions). This is recomputed on every evaluation, not cached.
- **Resolve available actions.** At any moment, compute the set of actions the player can perform: which choices are visible (not consumed, conditions met), which exits are traversable, which sequence actions are available.
- **Apply effects.** When an action is performed, execute its effects in order: set properties, move entities between containers, reveal hidden properties, destroy entities, spawn new ones.
- **Fire rules.** After effects are applied, check if any rules' triggers are met. If so, evaluate their conditions and select blocks, and apply their effects. Rules may cascade (one rule's effects trigger another rule).
- **Manage dialogue state.** Track which section is active, which choices have been consumed, when sections are exhausted, and where jumps lead.
- **Expose state to presentation.** Provide a clean read only view of the current world state to the presentation layer: what the player can see, what choices are available, what text should be displayed. The runtime never renders anything itself.

### The State Model

```
WorldState {
  entities: Map<id, EntityState>,
  locations: Map<id, LocationState>,
  player: {
    container: location_id,
    properties: Map<prop, value>,
  },
  sequence: {
    current_phase: phase_id | null,
    completed_phases: Set<phase_id>,
  },
  dialogue: {
    active_section: section_id | null,
    consumed_choices: Set<choice_id>,
  },
}

EntityState {
  type: type_id,
  container: entity_id | location_id,
  properties: Map<prop, value>,
  destroyed: boolean,
}
```

### Interface

```
// Conceptual interface, language agnostic

load(world: UrdWorldJSON) → WorldState

getAvailableActions(state: WorldState) → Action[]

performAction(state: WorldState, actionId: string) → {
  newState: WorldState,
  events: Event[],
}

Event {
  type: dialogue | narration | move | reveal | destroy | spawn,
  entity: entity_id | null,
  text: string | null,
  details: any,
}
```

### Key Design Decisions

**Immutable state transitions.** Every action produces a new WorldState rather than mutating the existing one. This makes undo/redo trivial, enables state comparison for testing, and prevents a class of bugs where effects partially apply before a condition fails.

**Event sourcing.** Every action returns a list of events describing what happened. The presentation layer uses these events to display text, play animations, or update its UI. The runtime never assumes any particular presentation. A text runner prints dialogue. A Unity integration plays animations. A test harness asserts on events.

**Deterministic rule execution.** When multiple rules trigger simultaneously, they fire in declaration order. When a select block has multiple valid candidates, selection is random but seeded: given the same seed, the same world plays out identically. This is essential for reproducible testing.

**Visibility enforcement.** The runtime maintains the full world state internally but exposes only visible information through the `getAvailableActions` and event interfaces. Hidden properties are never leaked to the presentation layer. The testing framework, however, can access the full internal state for validation.

## Component 3: The Testing Framework

The testing framework validates that a compiled world behaves correctly. It sits alongside the runtime, uses the same state model, but adds assertion capabilities and automated playthrough support.

The product vision identifies testing as the early product wedge: the tool that game studios adopt first, before the full editor or engine integrations. This component must be useful independently.

### Responsibilities

- **Schema validation.** Verify that a `.urd.json` file conforms to the schema specification: all required fields present, types correct, references resolvable, no orphaned entities.
- **Reachability analysis.** Static analysis of the world graph: can the player reach every location? Are there dead end sections with no exit? Are there choices that can never be available because their conditions are contradictory?
- **Playthrough simulation.** Execute automated playthroughs: given a sequence of player actions, does the world reach the expected state? Run 10,000 Monty Hall games with random choices and verify the switching advantage converges to 2/3.
- **State assertions.** At any point during a playthrough, assert on world state: "after the player picks door 1 and switches, the prize behind the opened door is goat."
- **Coverage reporting.** Which actions, conditions, effects, and dialogue sections were exercised during a test run? Which were never reached? This is the schema equivalent of code coverage.
- **Visibility auditing.** Verify that hidden state is never exposed to the player through any available action or event sequence. This is a critical correctness property that's hard to verify by hand.

### Test Definition Format

```
test "switching wins 2/3 of the time" {
  world: monty-hall.urd.json
  runs: 10000
  strategy: random
  assert: switch_wins / total > 0.6
  assert: switch_wins / total < 0.7
}

test "key unlocks door" {
  world: two-room-key.urd.json
  steps:
    - action: pick_up_key
    - assert: @rusty_key.container == player
    - action: unlock_door
    - assert: @cell_door.locked == false
    - assert: @rusty_key.destroyed == true
    - move: north
    - assert: player.container == corridor
}
```

### Key Design Decisions

**Tests run against compiled JSON, not source.** The testing framework never parses `.urd.md`. It receives the same `.urd.json` the runtime uses. This means tests validate the actual artifact that engines will consume, not an intermediate representation.

**The runtime is embedded, not mocked.** The testing framework uses the real reference runtime (Wyrd) to execute playthroughs. It does not reimplement world logic. This guarantees that tests exercise the same code path as production.

**Static analysis is complementary, not a replacement.** Reachability analysis can catch structural bugs (orphaned locations, impossible conditions) without running any playthroughs. But it cannot catch semantic bugs (Monty opening the wrong door, the key not working). Both static and dynamic testing are needed.

## Component 4: Developer Tooling

Developer tooling is what makes the syntax usable in practice. Without it, writers revert to ink regardless of how clean the syntax is. Tooling covers everything that helps a writer or engineer work with `.urd.md` files in their editor.

### Capabilities

| Capability | Description | Priority |
|-----------|-------------|----------|
| Syntax highlighting | Colour coded `.urd.md` files in editors. Frontmatter, entities, conditions, effects, sections, dialogue. | Must have |
| Diagnostics (live) | Real time error and warning display as the writer types. Powered by the compiler running in watch mode. | Must have |
| Entity autocomplete | Type `@` and see available entities. Type `@entity.` and see available properties. | Must have |
| Section navigation | Jump to section, list all sections, find all jumps to a section. | Must have |
| Go to definition | Click `@entity` or `->` section and jump to its declaration. | High |
| Depth warning | Visual indicator when indentation exceeds two levels. | High |
| Conversation graph | Visual overview of section structure and jump paths within a file. | Medium |
| Rename symbol | Rename an entity, section, or type and update all references. | Medium |
| World preview | Live preview of the world running in a text mode console alongside the editor. | Medium |
| Schema inspector | View the compiled JSON output for the current file. | Low |

### Architecture: LSP

The standard approach is a **Language Server Protocol (LSP)** implementation. The LSP server embeds the compiler in watch mode, recompiles on every edit, and exposes diagnostics, completions, navigation, and other features through the standard protocol. This gives editor support for VS Code, Vim, Emacs, Sublime, and any other LSP capable editor from a single implementation.

```
┌─────────────────────┐
│  Editor (VS Code)   │
│                     │
│  .urd.md buffer     │
└──────────┬──────────┘
           │ LSP protocol
           ▼
┌─────────────────────┐
│  Urd Language       │
│  Server             │
│                     │
│  Embedded compiler  │
│  Watch mode         │
│  Diagnostics cache  │
└─────────────────────┘
```

The LSP server is a thin wrapper around the compiler. The compiler does the hard work (parsing, linking, validation). The LSP translates compiler output into the protocol's format for diagnostics, completions, and navigation.

### Key Design Decision

**The compiler must be fast enough for interactive use.** If recompilation takes more than 200ms after an edit, the live diagnostics will feel laggy. This sets a performance target for the compiler: a medium sized world (50 files, 500 entities) should compile in under 200ms. This influences the language choice for the compiler implementation.

## The `.urd.json` Contract

The compiled JSON file is the central artifact of the system. It is the only thing the runtime, testing framework, and engine integrations need to understand. Its structure is defined by the Urd World Schema Specification, but several aspects deserve explicit architectural attention.

> **Normative formats.** The Urd framework has exactly two file formats. `.urd.md` (Schema Markdown) is the authoring format, parsed by the compiler. `.urd.json` (compiled JSON) is the execution format, consumed by runtimes. No tool in the pipeline accepts, produces, or requires YAML. The YAML like notation used in the schema specification is a documentation convention for readability; it is not a supported input or output format. Urd frontmatter in `.urd.md` files resembles YAML but is a distinct constrained grammar parsed by the compiler's own frontmatter parser.

### Properties

- **Self contained.** A `.urd.json` file contains everything needed to execute the world. All imports are resolved, all references are validated, all types are expanded. The runtime never needs to look up external files.
- **Deterministic.** The same set of `.urd.md` source files always produces the same `.urd.json` output. Byte identical. This makes compiled output cacheable, diffable, and CI friendly.
- **Versioned.** The urd field in the world block declares which schema version the file targets. Runtimes check this on load and reject unsupported versions with a clear error.
- **Human inspectable.** Though it's machine consumed, the JSON should be formatted (not minified) by default so engineers can read it for debugging. A `--minify` flag on the compiler produces compact output for production.
- **Extensible with version enforcement.** New schema versions add new top level blocks. A runtime must check the urd version field on load. If the file declares a version the runtime does not fully support, it must report a clear version mismatch error rather than silently ignoring unknown blocks.

### The Dialogue Block

The Schema Markdown syntax introduced dialogue sections (`== name`) and the hybrid nesting model. These compile to a dialogue block in the JSON. Dialogue is a core part of the v1 schema: a compliant runtime must support it.

```json
{
  "dialogue": {
    "tavern/topics": {
      "prompt": { "speaker": "arina", "text": "What'll it be?" },
      "choices": [
        {
          "id": "tavern/topics/ask-about-the-harbor",
          "label": "Ask about the harbor",
          "sticky": true,
          "response": { "..." : "..." },
          "effects": [ "..." ],
          "goto": "tavern/topics"
        },
        {
          "id": "tavern/topics/ask-about-the-missing-ship",
          "label": "Ask about the missing ship",
          "sticky": false,
          "conditions": [ "..." ],
          "response": { "..." : "..." },
          "goto": "tavern/topics"
        }
      ],
      "on_exhausted": { "text": "Suit yourself." }
    }
  }
}
```

Section IDs use the `file_stem/section_name` convention (e.g., `tavern/topics`). Choice IDs use `section_id/slugified_label`. The `sticky` field distinguishes `+` (true) from `*` (false) choices. The `on_exhausted` field contains the fallthrough content shown when the section is exhausted; it is a content payload, not a boolean. Whether a section *is* exhausted is a runtime-evaluated predicate (see the Runtime's condition evaluation responsibilities). The `goto` field compiles from `->` jumps and uses the full section ID.

## Build Order

The components have dependencies. The build order is determined by what unblocks the most progress earliest.

> **Build phases are not schema versions.** All phases produce v1 output. The phased build is an engineering strategy for incremental delivery, not a statement about the schema's completeness. A v1 runtime that has only completed Phase 1 is an incomplete v1 runtime, not a different version.

### Phase 1: Compiler + Minimal Runtime

> **Goal:** Compile the Monty Hall and Two Room Key Puzzle source files to valid JSON, and execute them in a text console.

The compiler is the foundation. Nothing else works without it. The minimal runtime is just enough to prove the compiled output is correct: a text mode loop that loads the JSON, displays available actions, accepts player input, and shows results.

**Milestone:** Run the Monty Hall game 10,000 times with random choices. The switching advantage converges to 2/3. Run the Key Puzzle. Pick up the key, unlock the door, reach the corridor.

- **Compiler:** Full parser, import resolution, type validation, JSON emission.
- **Runtime:** State management, condition evaluation, effect application, action resolution. No dialogue sections yet.
- **Not yet:** Dialogue sections, exhaustion, testing framework, LSP.

### Phase 2: Dialogue + Testing

> **Goal:** Compile and execute the tavern dialogue scene and the interrogation scene. Run automated tests against all four test cases.

This phase adds the dialogue block to the compiler and runtime (sections, jumps, sticky/one shot choices, exhaustion), and delivers the testing framework with both static analysis and playthrough simulation.

**Milestone:** The tavern scene compiles and plays correctly, with sticky choices persisting across hub loops and one shot choices disappearing. The testing framework verifies Monty Hall's probability distribution and the Key Puzzle's reachability. The interrogation stress test validates dialogue at production complexity.

- **Compiler:** Dialogue sections, `==` labels, `->` jumps, `*` vs `+` choice types, `on_exhausted` fallthrough content.
- **Runtime:** Dialogue state management, section navigation, choice consumption tracking.
- **Testing:** Schema validation, playthrough simulation, state assertions, reachability analysis.

### Phase 3: Developer Tooling

> **Goal:** VS Code extension with live diagnostics, autocomplete, and section navigation.

The LSP server wraps the compiler in watch mode. This is the phase where the authoring experience becomes usable for real projects, not just demos.

**Milestone:** A writer opens a `.urd.md` file in VS Code, types `@`, and sees a list of available entities. They reference a non existent property and see an underline with a helpful error message. They click on a `->` section jump and navigate to the section.

- **LSP server:** Embedded compiler, diagnostics, completions, go to definition, section navigation.
- **VS Code extension:** Syntax highlighting, LSP client, depth warnings.

### Phase 4: Engine Integration + Visual Editor

Not scoped in this document. These are product phases that depend on the core components being stable. The product vision describes the visual editor and engine integrations in detail.

### v1 Acceptance Checklist

This checklist defines what a v1-compliant implementation must satisfy. Every item references its normative source. Use this to verify completeness at the end of each build phase.

**Compiler**

| # | Requirement | Source |
|---|-------------|--------|
| C1 | Parse Urd frontmatter without a general-purpose YAML parser. Reject anchors, aliases, merge keys, custom tags, block-style lists, and implicit type coercion. | Schema Specification §Frontmatter Grammar |
| C2 | Resolve `import:` declarations. Imports are explicit and non-transitive. | Schema Markdown Specification §Import Resolution Rules |
| C3 | Detect circular imports of any cycle length. Report full cycle path. | Schema Markdown Specification §Import Resolution Rules |
| C4 | Reject duplicate entity IDs across the full compilation unit with diagnostics showing all declaration sites. | Schema Markdown Specification §Import Resolution Rules |
| C5 | Reject duplicate type names across the compilation unit. | Schema Markdown Specification §Import Resolution Rules |
| C6 | Validate that every `@entity` reference, `->` jump, and property access resolves to a declared target. | §Compiler Responsibilities |
| C7 | Validate property values against declared types, enum sets, and ref constraints. | Schema Specification §Property Schema |
| C8 | Emit `.urd.json` conforming to the Schema Specification. Set `urd: "1"` automatically. Warn and override if author sets it. | Schema Specification §world block |
| C9 | Enforce nesting depth: warn at 3 levels, error at 4+. | Schema Markdown Specification §Nesting Rules |

**Runtime**

| # | Requirement | Source |
|---|-------------|--------|
| R1 | Load and validate `.urd.json`. Reject unknown `urd` version. | Schema Specification §Evaluation Order |
| R2 | Implement all five effect types: `set`, `move`, `reveal`, `destroy`, `spawn`. | Schema Specification §Effect Declarations |
| R3 | Implement the containment model. `move` is the sole spatial operation. | Schema Specification §Containment Model |
| R4 | Implement all four visibility levels: `visible`, `hidden`, `owner`, `conditional`. | Schema Specification §Visibility Model |
| R5 | Implement `select` with uniform random selection from matching candidates. | Schema Specification §The select Block |
| R6 | **Determinism contract.** Identical JSON + identical seed + identical actions = identical event stream. | Schema Specification §Determinism Contract |
| R7 | Consume random values in rule declaration order. One value per multi-match select. Zero for single or zero matches. | Schema Specification §Determinism Contract |
| R8 | Support `world.seed` from compiled JSON and `world.seed(n)` API. Generate and record seed if not provided. | Schema Specification §Determinism Contract |
| R9 | Implement sequences with all four advance modes: `on_action`, `on_rule`, `on_condition`, `end`. | Schema Specification §Advance Modes |
| R10 | Implement dialogue: sections, sticky/one-shot choices, `goto`, `on_exhausted`. | Schema Specification §dialogue block |
| R11 | Compute exhaustion as a runtime predicate. Never store it. Never present an empty choice menu. | Schema Specification §dialogue block |
| R12 | Implement `any:` OR conditions. | Schema Specification §Condition Expressions |
| R13 | Implement all five trigger types. | Schema Specification §Trigger Types |
| R14 | Produce typed events for every state mutation. | Wyrd Reference Runtime §Event Types |
| R15 | Implicit `player` entity: create as mobile container at `start` if not declared. If declared, replace entirely (no merging). | Schema Specification §Player Entity Resolution Rules |

**Testing**

| # | Requirement | Source |
|---|-------------|--------|
| T1 | Test definitions are JSON only. YAML is not accepted. | Test Case Strategy §Test Definition Format |
| T2 | Tests run against compiled `.urd.json`, not source. | Test Case Strategy §Testing Layers |
| T3 | Seeded runs produce identical results on every run, every platform. | Schema Specification §Determinism Contract |
| T4 | Monte Carlo: Monty Hall switching advantage between 63%–70% over 10,000 runs. | Test Case Strategy §Test Case 2 |

## Technology Considerations

This document is deliberately language agnostic. The interfaces are defined as conceptual contracts, not concrete APIs. However, the choice of implementation language has real consequences for the compiler and runtime, and the tradeoffs are worth stating.

### The Compiler

The compiler has two competing requirements: correctness (it must never produce invalid JSON from valid source) and speed (it must be fast enough for interactive LSP use, targeting under 200ms for medium sized worlds).

| Option | Strengths | Risks |
|--------|-----------|-------|
| TypeScript | Fast iteration, large ecosystem, easy VS Code integration, team familiarity. | Performance ceiling for large worlds. Single threaded. |
| Rust | Excellent performance, strong type system catches bugs at compile time, WASM target for browser use. | Slower iteration, smaller pool of contributors, steeper learning curve. |
| Hybrid | TypeScript for LSP/tooling shell, Rust/WASM for the compiler core. | Integration complexity, two build systems, debugging across boundary. |

The decision does not need to be made now. The interfaces between components are defined clearly enough that the compiler could be built in either language and the LSP server, testing framework, and runtime would not need to change. The recommendation is to prototype the compiler in the language the team is most productive in, profile it against the 200ms target, and rewrite the hot path if needed.

### The Runtime

The reference runtime has a different profile: correctness matters more than speed, and it needs to run in multiple environments (server side for testing, browser for web demos, embedded in game engines via WASM or native bindings).

A single implementation that compiles to both native and WASM is the ideal target. Rust achieves this naturally. TypeScript achieves it with a WASM bridge for engine integrations. The choice depends on where the team expects the runtime to be consumed most often.

### The Testing Framework

The testing framework embeds the runtime, so it inherits whatever language the runtime uses. The test definition format (how tests are authored) is independent of the implementation language and should be designed for readability by non engineers.

## AI Integration Points

The product vision identifies AI assisted development as a strategic multiplier. The typed schema makes AI coding tools dramatically more capable. This section maps where AI touches the pipeline. Detailed design is deferred, but the architecture must not preclude these integration points.

### Compiler: Schema Constrained Generation

An AI generating content into the Urd schema cannot produce structurally invalid output if the compiler validates it. The compilation pipeline enables a generate then validate loop: an LLM produces `.urd.md` content, the compiler checks it, and errors are fed back to the LLM for correction. This requires the compiler's diagnostic output to be machine readable (structured JSON, not just human readable text), which is already the case in the proposed interface.

### Runtime: Intelligent NPCs

The runtime's `getAvailableActions` and event interfaces are clean enough for an AI agent to operate as a player. An LLM driven NPC could: read the current world state, evaluate which actions are available, select an action based on its character definition, and let the runtime apply the effects. The NPC's personality and goals would be defined in the schema (entity properties), not in the AI prompt. The runtime provides the guardrails; the AI provides the decision making.

### Testing: AI Augmented Narrative QA

The testing framework's playthrough simulation can be combined with LLM judgment. The framework runs a thousand playthroughs; the AI evaluates whether each playthrough produces a narratively coherent experience. This catches semantic bugs (an NPC references an event the player never saw) that purely structural analysis misses. The testing framework's state assertion format already supports this: run playthroughs programmatically, then pass the event logs to an LLM for narrative review.

### Developer Tooling: Intelligent Authoring

The LSP server's compilation data is exactly what an AI coding assistant needs. Entity types, property schemas, valid conditions, available sections: all of this is in the compiler's AST. An AI assistant in the editor could offer completions that are schema valid (suggesting `@guard.mood` not `@guard.trust` when trust doesn't exist), warnings about unused state ("this conversation modifies `@arina.trust` but no condition ever checks it"), and generated content that the compiler can immediately validate.

> **Architectural principle:** AI is an optional enhancement layer, not a dependency. Every Urd world must work with purely authored content. The AI integration points are interfaces the architecture exposes, not components it requires.

## Risk Register

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| Compiler too slow for LSP | High | Medium | Profile early. Prototype against 50 file world. Rewrite hot path if needed. |
| Import resolution creates cycle bugs | Medium | Medium | Detect cycles in Phase 1. Fail with clear error, not stack overflow. |
| Dialogue state model is insufficient | High | Low | The interrogation stress test exercises the hardest cases. Extend if new patterns emerge. |
| Source maps add complexity | Low | High | Defer to Phase 2. Design the data model to accommodate them without requiring them. |
| Third party runtime disagrees with reference | Medium | Medium | Testing framework is the arbiter. Publish conformance test suite. |
| Writers don't adopt VS Code | Medium | Low | LSP is editor agnostic. Support any LSP client. Syntax highlighting is portable. |

## Summary

The Urd framework is four components connected by one contract:

- **The Compiler** transforms `.urd.md` source into `.urd.json`. It is the first thing to build and the most critical to get right.
- **The Reference Runtime (Wyrd)** executes `.urd.json` worlds. It defines correct behavior for all downstream integrations.
- **The Testing Framework** validates worlds against schema and behavioral expectations. It is the early product wedge.
- **Developer Tooling** makes the syntax usable in practice. Without it, correct syntax fails because the experience is poor.

The `.urd.json` file is the contract between authoring and execution. Everything on the authoring side produces it. Everything on the execution side consumes it. The boundary is clean, and the components are independently replaceable.

The build order is: compiler first (unblocks everything), runtime second (proves the compiler's output works), testing third (validates both), tooling fourth (makes it all usable). Each phase has a concrete milestone that proves the component works against the established test cases.

*This document is the system map.* **Each component's detailed design is a separate document, written when engineering begins on that component.**

*End of Architecture*
