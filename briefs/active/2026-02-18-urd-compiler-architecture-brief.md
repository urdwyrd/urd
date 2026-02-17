# URD — Compiler Architecture

*The internal blueprint for the five-phase compiler pipeline*

February 2026 | Engineering Phase

`.urd.md → PARSE → IMPORT → LINK → VALIDATE → EMIT → .urd.json`

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-18
**Status:** Done — project scaffolding complete, all types defined, compiles clean

### What was done

- Created `packages/compiler/` as a Rust library crate (`urd-compiler`, edition 2021, `indexmap` dependency)
- Defined all shared types in dedicated modules:
  - `span.rs` — `Span` (file + start/end line/col, 1-indexed, byte-offset columns), `FilePath` type alias
  - `ast.rs` — all 28+ AST node types: `FileAst`, `Frontmatter`, `FrontmatterEntry`, `FrontmatterValue` (8 variants), `Scalar`, `ImportDecl`, `WorldBlock`, `TypeDef`, `PropertyDef`, `EntityDecl`, `ContentNode` (17 variants + `ErrorNode`), `RuleBlock`, `SelectClause`, `ConditionExpr` (3 variants), `EffectType` (4 variants), `Annotation`
  - `symbol_table.rs` — `SymbolTable` (7 `IndexMap`s), `TypeSymbol`, `PropertySymbol`, `EntitySymbol`, `SectionSymbol`, `ChoiceSymbol`, `LocationSymbol`, `ExitSymbol`, `ActionSymbol`, `RuleSymbol`, `SequenceSymbol`, `PhaseSymbol`, `SelectDef`, `AstNodeRef`, `PropertyType`, `Visibility`, `Value`
  - `diagnostics.rs` — `DiagnosticCollector`, `Diagnostic`, `Severity`, `RelatedInfo`
  - `graph.rs` — `DependencyGraph`, `FileNode`, `file_stem()`, all limit constants
  - `slugify.rs` — `slugify()` with 6 passing unit tests
- Created phase modules with stub signatures: `parse::parse()`, `import::resolve_imports()`, `link::collect_declarations()`, `link::resolve_references()`, `link::LinkedWorld`, `validate::validate()`, `emit::emit()`
- Created `lib.rs` with `CompilationResult` and `compile()` orchestrator matching the brief's pseudocode
- Created 6 test files: one per phase + integration, with placeholder test categories from the brief
- All 7 tests pass, zero warnings

### What changed from the brief

- **Annotation fields store `Option<String>` IDs**, not direct symbol references. Rust ownership makes storing `&TypeSymbol` etc. impractical without `Arc`/indices. Phases look up the ID in the symbol table. Same data, different indirection. Every reference-bearing node has its annotation field.
- **`EffectType` uses Rust enum variants** with embedded fields (`Set { target_prop, value_expr }`) rather than the brief's generic `fields` map. More type-safe, same data.
- **`FrontmatterValue` has a `WorldBlock` variant** not listed in the brief's table. The brief defines `WorldBlock` as a frontmatter-specific node; wrapping it in the value enum is necessary since it appears as a value for the `world:` key in frontmatter entries.
- **`ExitSymbol.condition_node` and `blocked_message_node`** use `AstNodeRef { file, node_index }` — a lightweight struct — rather than a direct AST pointer, for the same ownership reasons as annotations.
- **`DiagnosticCollector.sorted()`** currently sorts by file path alphabetically. The brief specifies topological import order. This will be corrected when IMPORT provides that order — the sort function will need the graph as input.
- **Nesting depth constants** live in `graph.rs` alongside the other limit constants rather than in a separate module. Includes `MAX_CHOICE_NESTING_DEPTH` (4), `WARN_CHOICE_NESTING_DEPTH` (3), `MAX_FRONTMATTER_NESTING_DEPTH` (8).
- **`SelectDef.where_clauses`** uses `where_clauses` instead of `where` since `where` is a Rust reserved keyword.

---

> **Document status: BRIEF** — Defines the compiler's internal architecture: the data structures that flow between phases, the shared infrastructure all phases depend on, and the interface contracts that make each phase independently implementable. Individual phase briefs define *what* each phase does. This document defines *how they connect*.

## Purpose

The compiler transforms Schema Markdown source files (`.urd.md`) into a single validated JSON world file (`.urd.json`). It is the most critical component in the Urd ecosystem. Every downstream component — the Wyrd runtime, the testing framework, engine integrations, developer tooling — depends on its correctness.

The compiler has five phases. Each phase has been scoped for a separate implementation brief. This document exists because those briefs cannot be written independently — they share data structures, error infrastructure, ID derivation rules, and assumptions about what the previous phase guarantees. Writing each phase brief without this shared foundation would produce five documents that don't align.

### What This Covers

- **The AST.** The node types that the parser produces and all subsequent phases consume or annotate.
- **The Symbol Table.** The shared registry of types, entities, sections, and actions that phases populate and query.
- **The Diagnostic Collector.** The error and warning infrastructure that all phases write to.
- **The Dependency Graph.** The file import structure that IMPORT builds and later phases use for invalidation.
- **Phase Interface Contracts.** What each phase receives, what it produces, what it guarantees about its output.
- **The Two-Pass Strategy.** How the collection pass and validation pass map onto the five phases.
- **ID Derivation Rules.** How entity IDs, section IDs, and choice IDs are computed — a cross-cutting concern that touches PARSE, LINK, and EMIT.
- **Error Recovery Strategy.** How the compiler continues after errors to report multiple issues in one run.
- **Design Decisions That Affect Multiple Phases.** Forward references, incremental compilation hooks, source map readiness, deterministic output.

### What This Does Not Cover

- Phase-specific logic. The IMPORT brief covers cycle detection. The LINK brief covers reference resolution. The VALIDATE brief covers type checking. The EMIT brief covers JSON generation. This document does not duplicate those responsibilities.
- Runtime semantics. How the runtime evaluates conditions, applies effects, or manages dialogue state is the Wyrd specification's concern, not the compiler's.
- Language choice. The architecture doc leaves this open. This brief is language-agnostic. The data structures are described as conceptual types, not concrete implementations.

### Glossary

| Term | Definition |
|------|-----------|
| **Compilation unit** | The entry file plus every file reachable through its transitive import graph. This is the complete set of files the compiler processes in one `compile()` call. |
| **Entry file** | The `.urd.md` file passed to `compile()`. It is the root of the dependency graph. |
| **File stem** | The filename without its directory path or `.urd.md` extension. `content/tavern.urd.md` has stem `tavern`. Used to derive section IDs and must be unique within the compilation unit. |
| **Visible scope** | The set of files whose declarations a given file can reference: itself plus its direct imports. Non-transitive. |
| **OrderedMap** | A map that preserves insertion order. Provides O(1) key lookup and stable iteration order. |
| **Annotation** | A field on an AST node, initially null, that LINK populates with a reference to the resolved symbol. |
| **Slugify** | Convert a display name to an ID: lowercase, spaces to hyphens, strip non-alphanumeric, collapse hyphens, trim. |

### Phase Contract Summary

| Phase | Input | Output | Key Guarantee |
|-------|-------|--------|--------------|
| PARSE | `.urd.md` source text | `FileAST` with span-tracked nodes | Every syntactically valid construct has a node. Errors produce `ErrorNode` markers. |
| IMPORT | Entry `FileAST` | `DependencyGraph` + all `FileAST`s in topological order | Acyclic. Depth-limited. File stems unique. Paths normalized. |
| LINK | `DependencyGraph` + `FileAST`s | `SymbolTable` + annotated ASTs | Every declared name registered. Every resolvable reference annotated. Duplicates flagged. Visible scope enforced. |
| VALIDATE | Annotated ASTs + `SymbolTable` | Validation diagnostics | All type constraints checked. Unresolved references silently skipped (no cascading). |
| EMIT | Validated ASTs + `SymbolTable` | `.urd.json` string | Conforms to JSON Schema. Deterministic. `urd: "1"` injected. |


## The Pipeline

```
  .urd.md source files
       │
  1. PARSE        Source text → per-file ASTs
       │           Each file parsed independently.
       │           Output: FileAST per file.
       │
  2. IMPORT       Resolve import declarations, build dependency graph.
       │           Output: DependencyGraph + ordered list of FileASTs.
       │
  3. LINK         Merge scopes, resolve all cross-file references.
       │           Populate the global SymbolTable.
       │           One phase, two internal passes: collection then resolution.
       │           Output: LinkedWorld (annotated ASTs + SymbolTable).
       │
  4. VALIDATE     Type-check properties, conditions, effects.
       │           Enforce semantic constraints.
       │           Output: ValidatedWorld (LinkedWorld + validation pass).
       │
  5. EMIT         AST → .urd.json
       │           Output: JSON string + diagnostics.
       │
  .urd.json + Diagnostic[]
```

### The Two-Pass Model

The architecture document specifies that the compiler needs at least two passes: a collection pass to gather all declarations (including from imports), and a validation pass to check references and emit. Forward references within a file must work — an action can reference an entity declared later in the same file or in an imported file.

The five phases map onto two logical passes:

**Pass 1 — Collection:** PARSE + IMPORT + LINK (partial). Parse every file, resolve imports, build the dependency graph, merge all type and entity declarations into the global symbol table. After this pass, every declared name is known.

**Pass 2 — Validation and Emission:** LINK (reference resolution) + VALIDATE + EMIT. Resolve every reference against the now-complete symbol table, type-check all values, enforce constraints, and emit JSON.

In practice, LINK straddles both passes. Its declaration-collection work (registering types, entities, sections, actions) happens during Pass 1. Its reference-resolution work (resolving `@entity` references, `->` jumps, property accesses) happens during Pass 2, after the full symbol table is built. This is the natural consequence of supporting forward references.

> **Clarification: the compiler has five phases, not seven.** LINK's two sub-passes are internal to LINK, not separate compiler phases. The phase count is: PARSE, IMPORT, LINK, VALIDATE, EMIT. LINK happens to do its work in two sweeps over the ASTs, but it is one phase with one input contract and one output contract.

The phase briefs will specify which parts of LINK belong to which pass.


## The AST

The Abstract Syntax Tree is the central data structure. PARSE produces it. Every subsequent phase reads it, and LINK and VALIDATE annotate it. EMIT traverses it to produce JSON.

### Design Principles

**File-scoped.** Each parsed file produces its own `FileAST`. Files are never merged at the AST level — merging happens in the symbol table. This keeps the AST a faithful representation of what the author wrote, which matters for diagnostics (error messages reference source positions) and for incremental compilation (a changed file only needs its AST rebuilt).

**Annotatable.** AST nodes carry an optional annotation slot that later phases fill in. PARSE leaves annotations empty. LINK adds resolved references (the actual entity a `@name` points to). VALIDATE adds type information. This avoids rebuilding the tree — phases enrich it in place.

**Span-tracked.** Every AST node records its source position: file path, start line, start column, end line, end column. This is non-negotiable. Diagnostics, source maps, and LSP features all depend on precise source positions.

### Node Types

The AST must represent every syntactic construct in the Schema Markdown grammar. The following is the complete inventory of node types, grouped by the region of a `.urd.md` file they appear in.

#### File-Level Nodes

| Node | Fields | Notes |
|------|--------|-------|
| `FileAST` | `path`, `frontmatter?`, `content`, `span` | Root node per file. |
| `Frontmatter` | `entries[]`, `span` | The `---`-delimited block. |
| `FrontmatterEntry` | `key`, `value`, `span` | A key-value pair. Value is a `FrontmatterValue`. |
| `FrontmatterValue` | (union: `Scalar`, `List`, `Map`, `InlineObject`, `EntityDecl`, `TypeDef`, `ImportDecl`) | Typed value node. |

#### Frontmatter-Specific Nodes

| Node | Fields | Notes |
|------|--------|-------|
| `ImportDecl` | `path`, `span` | `import: ./path.urd.md` |
| `WorldBlock` | `fields: Map<string, Scalar>`, `span` | The `world:` block. |
| `TypeDef` | `name`, `traits[]`, `properties: Map<string, PropertyDef>`, `span` | A type definition. |
| `PropertyDef` | `name`, `type`, `default?`, `visibility?`, `values?`, `min?`, `max?`, `ref_type?`, `description?`, `span` | A property within a type. |
| `EntityDecl` | `id`, `type_name`, `property_overrides: Map<string, Scalar>`, `span` | `@name: Type { ... }` |

#### Content Nodes

| Node | Fields | Notes |
|------|--------|-------|
| `LocationHeading` | `display_name`, `span` | `# Display Name`. The `display_name` is the raw heading text as written by the author. The location ID used in the symbol table and JSON output is derived from it via `slugify(display_name)` during LINK — it is not stored on this node. |
| `SequenceHeading` | `display_name`, `span` | `## Display Name`. ID derived via `slugify(display_name)` during LINK. |
| `PhaseHeading` | `display_name`, `auto`, `span` | `### Name (auto)`. ID derived via `slugify(display_name)` during LINK. |
| `SectionLabel` | `name`, `span` | `== name` |
| `EntityPresence` | `entity_refs[]`, `span` | `[@arina, @barrel]` |
| `EntitySpeech` | `entity_ref`, `text`, `span` | `@arina: What'll it be?` |
| `StageDirection` | `entity_ref`, `text`, `span` | `@arina leans in close.` |
| `Prose` | `text`, `span` | Plain narrative text. |
| `Choice` | `sticky`, `label`, `target?`, `target_type?`, `content[]`, `span` | `*` or `+` with nested content. |
| `Condition` | `expr`, `span` | `? expression` |
| `OrConditionBlock` | `conditions[]`, `span` | `? any:` + indented bare expressions. |
| `Effect` | `effect_type`, `fields`, `span` | `> effect` — see Effect Subtypes below. |
| `Jump` | `target`, `is_exit_qualified`, `span` | `-> name` or `-> exit:name` |
| `ExitDeclaration` | `direction`, `destination`, `span` | `-> north: Corridor` |
| `BlockedMessage` | `text`, `span` | `! The door is locked.` |
| `RuleBlock` | `name`, `actor`, `select?`, `where_clauses[]`, `effects[]`, `span` | `rule name:` block. |
| `Comment` | `text`, `span` | `// text` — retained in AST for potential LSP use, stripped in EMIT. |

#### Rule Block AST Detail

The `RuleBlock` AST node represents a complete rule definition from narrative content. Its full structure:

```
RuleBlock {
  name: string,                     // the rule identifier after "rule "
  actor: string,                    // entity ref (raw string, no @)
  trigger: string,                  // full trigger string, e.g., "phase_is reveal"
  select: SelectClause | null,      // the selects...from...where block, if present
  where_clauses: ConditionExpr[],   // top-level rule conditions (outside select)
  effects: Effect[],                // rule effect nodes
  span: Span,
}

SelectClause {
  variable: string,                 // bound variable name
  entity_refs: string[],            // raw entity ref strings (no @)
  where_clauses: ConditionExpr[],   // select-scoped conditions using the bound variable
  span: Span,
}
```

**Rule-scoped conditions and effects.** Inside rule blocks, conditions and effects may reference the bound variable from a `select` clause. The bound variable appears in the same positions as an entity ref (e.g., `door.prize == goat` where `door` is the bound variable, not a declared entity). PARSE stores these as standard `ConditionExpr` and `Effect` nodes with the variable name in the entity_ref position. LINK resolves the variable contextually during rule body resolution — it recognises the variable name from the enclosing `SelectClause` and does not emit URD301 (unresolved entity) for it.

**Trigger syntax.** The `trigger` field stores the complete trigger string as a single token. Valid trigger patterns are: `phase_is {phase_name}`, `action {action_name}`, `enter {location_name}`, `state_change {entity.property}`, and `always`. PARSE stores the trigger as a raw string. LINK resolves the identifier within the trigger (phase name, action name, location name, or entity.property) against the symbol table during the resolution sub-pass. VALIDATE checks that the referenced symbol exists and is of the correct kind.

#### Effect Subtypes

The `Effect` node has a discriminated `effect_type` field:

| Type | Fields | Source Syntax |
|------|--------|---------------|
| `set` | `target_prop`, `value_expr` | `> @entity.prop = value` or `> @entity.prop + N` |
| `move` | `entity_ref`, `destination_ref` | `> move @entity -> container` |
| `reveal` | `target_prop` | `> reveal @entity.prop` |
| `destroy` | `entity_ref` | `> destroy @entity` |

`spawn` effects are not authored in Schema Markdown v1 — they appear only in hand-authored JSON or are emitted by rules. The parser does not need a spawn AST node.

#### Annotation Slot

Every AST node that contains a reference (entity ref, type name, section name, property access) carries an `annotation` field, initially `null`, that LINK populates:

```
Annotation {
  resolved_entity: EntitySymbol | null,
  resolved_type: TypeSymbol | null,
  resolved_section: SectionSymbol | null,
  resolved_property: PropertySymbol | null,
  resolved_location: LocationSymbol | null,
}
```

**Annotation vs Symbol resolution.** There are two places where resolved references are stored: on AST nodes (via annotations) and on symbol table entries (via resolved fields like `ExitSymbol.resolved_destination` or `EntitySymbol.type_symbol`). The pattern is consistent: AST nodes reference whatever they point to via their annotation slot, and symbols reference whatever they depend on via explicit resolved fields. Both are populated by LINK during the resolution sub-pass. VALIDATE and EMIT may read either, depending on whether they are traversing the AST or querying the symbol table. The two are always in agreement — LINK sets both from the same resolution logic.

VALIDATE reads these annotations to perform type checking. EMIT reads them to generate correct JSON references. If an annotation is still `null` at VALIDATE time, the reference is unresolved — this is already reported as an error by LINK.

### Condition Expression Types

The `ConditionExpr` type is a discriminated union parsed by PARSE and consumed by LINK, VALIDATE, and EMIT. PARSE produces the structured representation from the `? expression` source syntax. Downstream phases operate on typed fields — they never re-parse condition text.

```
ConditionExpr = PropertyComparison | ContainmentCheck | ExhaustionCheck

PropertyComparison {
  kind: "property_comparison",
  entity_ref: string,          // raw identifier, e.g., "guard" (no @ sigil)
  property: string,            // e.g., "mood"
  operator: string,            // "==", "!=", "<", ">", "<=", ">="
  value: string,               // e.g., "neutral", "true", "50"
  span: Span,
}

ContainmentCheck {
  kind: "containment_check",
  entity_ref: string,          // the entity being checked, e.g., "rusty_key"
  container_ref: string,       // raw token as written: "here", "player", entity ID, or display name
  negated: boolean,            // true for "not in", false for "in"
  span: Span,
}

ExhaustionCheck {
  kind: "exhaustion_check",
  section_name: string,        // the local section name, e.g., "topics"
  span: Span,
}
```

**PARSE** produces these variants from source syntax. **LINK** resolves the references inside them (entity_ref → EntitySymbol, property → PropertySymbol, container_ref → container_kind discriminator, section_name → SectionSymbol). **VALIDATE** type-checks operators and values. **EMIT** lowers them to condition strings for JSON output.

The `container_ref` field stores the raw token as written by the author. LINK resolves it into a `container_kind` discriminator (`KeywordPlayer`, `KeywordHere`, `EntityRef`, `LocationRef`, or `null`) stored on the annotation. VALIDATE and EMIT read the discriminator, not the raw string.

### The `Span` Type

```
Span {
  file: FilePath,
  start_line: u32,      // 1-indexed
  start_col: u32,       // 1-indexed, byte offset within the line
  end_line: u32,
  end_col: u32,         // byte offset
}
```

Every AST node has a `span`. This is the compiler's most important debugging primitive. Diagnostics format spans as `file.urd.md:line:col`.

**Column encoding.** Internally, columns are byte offsets within the line. This is the natural output of most parsers and avoids ambiguity with multi-byte UTF-8 characters. The LSP protocol uses UTF-16 code unit offsets (per the LSP specification). Conversion from byte offsets to UTF-16 offsets is the responsibility of the tooling layer (Phase 3), not the compiler core. The compiler never performs this conversion — it stores and reports byte offsets throughout.


## The Symbol Table

The symbol table is the compiler's global registry of declared names. PARSE does not touch it. IMPORT does not touch it. LINK populates it during the collection sub-pass and queries it during the resolution sub-pass. VALIDATE reads it for type information. EMIT reads it for ID generation.

### Structure

```
SymbolTable {
  types: OrderedMap<TypeName, TypeSymbol>,
  entities: OrderedMap<EntityId, EntitySymbol>,
  sections: OrderedMap<SectionId, SectionSymbol>,
  locations: OrderedMap<LocationId, LocationSymbol>,
  actions: OrderedMap<ActionId, ActionSymbol>,
  rules: OrderedMap<RuleId, RuleSymbol>,
  sequences: OrderedMap<SequenceId, SequenceSymbol>,
}
```

**`OrderedMap` is a map that preserves insertion order.** Each category stores symbols in the order they are registered during LINK's collection sub-pass (topological file order, then declaration order within each file). This insertion order is the canonical ordering that EMIT uses for deterministic JSON output. Implementers may use a linked hash map, a map paired with a vector, or any structure that provides both O(1) lookup by key and stable iteration in insertion order.

### Visible Scope

The symbol table is a global registry, but not all symbols are visible to all files. Non-transitive imports mean that each file can only reference declarations from itself and from files it directly imports. LINK enforces this during its resolution sub-pass.

Each `FileAST` has an associated **visible scope**: the set of files whose declarations are accessible from that file. The visible scope for a file F is: F itself, plus every file that F directly imports. Not transitive imports — only direct ones.

```
visible_scope(F) = { F } ∪ { G : F directly imports G }
```

During LINK's resolution sub-pass, when resolving a reference in file F, the resolver checks the global symbol table but only accepts symbols whose `declared_in.file` is in `visible_scope(F)`. If a symbol exists in the table but was declared in a file outside F's visible scope, the reference is unresolved — LINK emits an error as if the symbol did not exist.

This means the global table is used for storage and duplicate detection (a duplicate entity ID is an error regardless of visibility), but visibility is enforced per-file during resolution. The global table knows everything; each file sees only what it has imported.

### Symbol Types

```
TypeSymbol {
  name: string,
  traits: string[],
  properties: Map<PropertyName, PropertySymbol>,
  declared_in: Span,
}

PropertySymbol {
  name: string,
  type: PropertyType,        // boolean | integer | number | string | enum | ref | list
  default: Value | null,
  visibility: Visibility,
  values: string[] | null,   // for enum
  min: number | null,
  max: number | null,
  ref_type: string | null,   // for ref
  element_type: PropertyType | null,  // for list: the scalar type of each element
  element_values: string[] | null,    // for list(enum(...)): the valid enum values
  element_ref_type: string | null,    // for list(ref(Type)): the target type name
  declared_in: Span,
}
```

### List Property Type

The `list` property type represents an ordered collection of values of a single element type. The property definition syntax is `list(element_type)` where `element_type` is any scalar property type: `string`, `bool`, `integer`, `number`, `enum(...)`, or `ref(TypeName)`.

The `element_type`, `element_values`, and `element_ref_type` fields are `null` for non-list properties. For list properties, `type` is `"list"` and `element_type` specifies the scalar type of each element. If the element type is `enum(...)`, `element_values` holds the valid enum values. If the element type is `ref(TypeName)`, `element_ref_type` holds the target type name.

**Frontmatter syntax examples:**

```
types:
  Chest [container]:
    contents: list(ref(Item))
    tags: list(string)
    scores: list(integer)

entities:
  @chest: Chest { contents: [@sword, @shield], tags: ["heavy", "locked"] }
```

Entity override values for list properties use flow-style list syntax (`[a, b, c]`). Each element is validated against the declared element type by VALIDATE.

```

EntitySymbol {
  id: string,                // the declared @name
  type_name: string,
  type_symbol: TypeSymbol | null,  // resolved during LINK
  property_overrides: Map<string, Value>,
  declared_in: Span,
}

SectionSymbol {
  local_name: string,        // the name after ==
  compiled_id: string,       // file_stem/section_name
  file_stem: string,
  choices: ChoiceSymbol[],
  declared_in: Span,
}

ChoiceSymbol {
  label: string,
  compiled_id: string,       // section_id/slugified-label
  sticky: boolean,
  declared_in: Span,
}

LocationSymbol {
  id: string,                // derived from heading name
  display_name: string,      // the original heading text
  exits: Map<string, ExitSymbol>,
  contains: string[],        // entity IDs
  declared_in: Span,
}

ExitSymbol {
  direction: string,
  destination: string,       // location ID as written in source (unresolved until LINK)
  resolved_destination: LocationSymbol | null,  // populated by LINK during resolution sub-pass
  condition_node: ASTNodeRef | null,  // reference to the Condition AST node (not a lowered string)
  blocked_message_node: ASTNodeRef | null,  // reference to the BlockedMessage AST node
  declared_in: Span,
}
```

LINK resolves `destination` to a `LocationSymbol` during its resolution sub-pass and stores it in `resolved_destination`. This follows the same annotation pattern used for entity and section references throughout the AST — EMIT reads the resolved symbol directly rather than re-deriving IDs by slugification.

```
ActionSymbol {
  id: string,
  target: string | null,
  target_type: string | null,
  declared_in: Span,
}

RuleSymbol {
  id: string,
  actor: string,
  trigger: string,
  select: SelectDef | null,
  declared_in: Span,
}

SequenceSymbol {
  id: string,
  phases: PhaseSymbol[],
  declared_in: Span,
}

PhaseSymbol {
  id: string,
  advance: string,
  action: string | null,
  actions: string[] | null,
  rule: string | null,
  declared_in: Span,
}

SelectDef {
  variable: string,            // the bound variable name, e.g., "door"
  from: string[],              // entity IDs to select from, e.g., ["door_1", "door_2", "door_3"]
  where: ConditionExpr[],      // filter conditions using the bound variable
  span: Span,
}
```

The `variable` in `SelectDef` is introduced by the `selects...from...where` syntax in rule blocks. Within the rule's `where` clauses and effects, the variable name can be used in place of a concrete entity ID. LINK registers the variable as a locally scoped alias during rule body resolution. VALIDATE checks that the `from` entities exist and that `where` conditions are valid for the bound variable's inferred type.

### Duplicate Detection

The symbol table rejects duplicate insertions. When LINK attempts to register a name that already exists, it does not overwrite — it records a diagnostic listing both declaration sites (the existing one and the new one). The duplicate entry is marked as `conflicted` in the table so that subsequent phases can skip it without cascading errors.

```
// Pseudo-interface
table.register_entity(id, symbol) → Ok | DuplicateError { existing: Span, new: Span }
```

This satisfies requirements C4 (duplicate entity IDs) and C5 (duplicate type names) from the acceptance checklist.

### Namespace Rules

The symbol table has separate maps for each category (types, entities, sections, locations, actions, rules, sequences). Names are unique *within* each category but are allowed to collide *across* categories. For example, a location named `cell` and an entity named `cell` can coexist — they occupy different namespaces and are disambiguated by context (entity references use `@cell`, location references appear in `world.start`, exit destinations, and containment).

The one exception where cross-namespace collision creates ambiguity is the `->` jump syntax, which can target either a section or an exit (see the Schema Markdown Specification's Normative Resolution Rule). The compiler handles this with the priority rule: sections take precedence over exits, and a warning is emitted when shadowing occurs. No other cross-namespace collisions produce ambiguity in v1 syntax.

**Entity IDs and type names are in separate namespaces.** An entity named `guard` and a type named `Guard` do not conflict. Type names are conventionally capitalised; entity IDs are lowercase. The PEG grammar enforces these conventions.


## The Diagnostic Collector

All five phases write to a shared diagnostic collector. The collector accumulates errors, warnings, and informational messages without halting compilation. This satisfies the error recovery requirement: the compiler reports as many errors as possible in a single run.

### Diagnostic Structure

```
Diagnostic {
  severity: Error | Warning | Info,
  code: string,            // Machine-readable code, e.g. "URD001"
  message: string,         // Human-readable message
  span: Span,              // Where in source
  suggestion: string | null, // "Did you mean @guard.mood?"
  related: RelatedInfo[],  // Additional context (e.g. "first declared here")
}

RelatedInfo {
  message: string,
  span: Span,
}
```

### Diagnostic Code Ranges

Each phase owns a code range. This makes it immediately clear which phase produced a diagnostic and avoids code collisions when phase briefs are written independently.

| Phase | Code Range | Examples |
|-------|------------|---------|
| PARSE | URD100–URD199 | `URD101: Unclosed frontmatter`, `URD102: Tab indentation not allowed` |
| IMPORT | URD200–URD299 | `URD201: File not found`, `URD202: Circular import detected`, `URD206: Import path casing mismatch` (Warning — compilation continues using discovered casing) |
| LINK | URD300–URD399 | `URD301: Unresolved entity reference`, `URD302: Duplicate entity ID` |
| VALIDATE | URD400–URD499 | `URD401: Type mismatch`, `URD402: Enum value not in declared set` |
| EMIT | URD500–URD599 | `URD501: Choice ID collision after slugification` |

### Error vs Warning Semantics

**Errors** prevent JSON emission. If the collector contains any Error-severity diagnostics after all phases run, EMIT produces `null` for the world output. Diagnostics are still returned.

**Warnings** do not prevent emission. The compiled JSON is valid but the author should review the flagged issues. Examples: nesting depth 3 (warn), section name shadows exit name (warn), author manually set `urd` field (warn and override).

**Info** messages are advisory. Examples: unused entity, unreachable section. Not emitted by default — enabled by a compiler flag.

### Continuation After Errors

When a phase encounters an error, it must continue processing. The strategies are phase-specific (each phase brief defines its own recovery approach), but the general principle is: record the error, mark the affected construct as damaged, and move on to the next construct. LINK marks unresolved references as `null` annotations. VALIDATE skips type checking on constructs with unresolved references (no cascading errors from LINK failures).

### Diagnostic Ordering

Diagnostics are emitted in a deterministic order for stable CI output. The rules are:

1. **Within a single file:** diagnostics are ordered by source position (line, then column).
2. **Across files:** diagnostics follow topological import order. The entry file's diagnostics appear last (it is the top of the dependency tree). If two files are at the same depth and unrelated, they are ordered alphabetically by normalized path.
3. **Within the same source position:** errors before warnings before info.

This ordering is stable across runs, platforms, and recompilations. CI pipelines can diff diagnostic output between runs to detect regressions.


## The Dependency Graph

IMPORT builds a directed graph of file imports. It is used during compilation for ordering and cycle detection, and cached for future incremental compilation.

### Structure

```
DependencyGraph {
  nodes: Map<FilePath, FileNode>,
  edges: (source: FilePath, target: FilePath)[],
}

FileNode {
  path: FilePath,
  ast: FileAST,
  imports: FilePath[],     // direct imports only (non-transitive)
}
```

### Properties

**Acyclic.** IMPORT rejects cycles with a diagnostic showing the full cycle path. If a cycle is detected, the offending import edge is dropped and compilation continues with whatever scope is available.

**Depth-limited.** The maximum import chain depth is 64. If file A imports B imports C ... and the chain exceeds 64 levels without a cycle, the compiler emits an error (URD204): *"Import depth limit exceeded (64 levels). This usually indicates an architectural problem in the project's file structure."* This guards against pathological dependency graphs that are technically acyclic but impractical.

**Non-transitive.** If A imports B and B imports C, A does NOT see C's declarations. A must import C directly. This is a normative rule from the Schema Markdown specification.

**Stable.** The graph is deterministic — same source files produce the same graph. Import declaration order within a file does not affect the graph structure or compilation output.

### File Path Normalization

All file paths in the dependency graph are normalized to a canonical form before comparison or storage. The normalization rules are:

- **Forward slashes always.** Backslashes are converted to forward slashes on all platforms. The compiled output and all diagnostics use forward slashes regardless of the host operating system.
- **All paths relative to the entry file's directory.** Every path in the dependency graph, including the entry file itself, is stored relative to the directory containing the entry file passed to `compile()`. If `compile("/projects/tavern-game/tavern.urd.md")` is called, then the entry file is stored as `tavern.urd.md`, and an imported file at `/projects/tavern-game/shared/types.urd.md` is stored as `shared/types.urd.md`. No path receives special treatment — the convention is uniform.
- **No symlink resolution.** The compiler operates on logical paths, not physical paths. If two symlinks point to the same file, the compiler treats them as two separate files. This avoids platform-dependent behaviour.
- **No `..` segments in resolved paths.** After resolving `import: ../shared/types.urd.md` from `content/tavern.urd.md`, the resulting path is `shared/types.urd.md`, not `content/../shared/types.urd.md`. Path segments are resolved before storage.
- **Case-sensitive comparison.** File paths are always compared case-sensitively, even on case-insensitive file systems. This prevents "works on my machine" errors where `Types.urd.md` and `types.urd.md` resolve to the same file on macOS but not on Linux.
- **Casing mismatch detection.** When the compiler opens a file, it compares the casing of the path as written in the `import:` declaration against the casing of the path as discovered on the filesystem. If they differ (e.g., `import: ./World.urd.md` but the file on disk is `world.urd.md`), the compiler emits a warning (URD206): *"Import path 'World.urd.md' differs in casing from discovered file 'world.urd.md'. Using discovered casing."* The discovered casing becomes the canonical path stored in the dependency graph. This catches cross-platform portability issues at authoring time.

### Topological Order

IMPORT produces a topologically sorted list of files. LINK processes files in this order so that imported types and entities are registered before the files that reference them. This is the natural consequence of non-transitive imports: a file's dependencies are always processed first.

**Tiebreaking.** Topological sort does not define a unique order when multiple files are at the same depth with no dependency between them. To ensure deterministic output, ties are broken by alphabetical order of normalized file path. This means both diagnostic ordering and EMIT's cross-file declaration ordering are fully determined: dependencies first (by topological depth), then alphabetical within the same depth. The entry file always appears last (it is the root of the dependency tree).


## ID Derivation Rules

ID derivation is a cross-cutting concern that touches PARSE (which sees the raw names), LINK (which computes compiled IDs), and EMIT (which writes them to JSON). The rules are defined here, once, and referenced by all three phase briefs.

### Entity IDs

Entity IDs are the declared `@name`. They are not transformed. `@rusty_key` in source produces `"rusty_key"` in JSON. Entity IDs must be globally unique across the entire compilation unit (entry file + all imports).

**Format:** Lowercase letters, digits, underscores. Must start with a letter. Validated by the PEG grammar's `Identifier` rule.

### Section IDs

Section IDs are derived from the file path and section name.

```
section_id = file_stem(file_path) + "/" + section_name
```

Example: `== topics` in `tavern.urd.md` → `"tavern/topics"`.

`file_stem` strips the directory path and the `.urd.md` extension. If the file is `content/tavern.urd.md`, the stem is `tavern`.

Section names follow the same format as entity IDs: lowercase, digits, underscores. The PEG grammar enforces this.

**File stem uniqueness is enforced by the compiler.** If two files in the compilation unit produce the same stem (e.g., `content/tavern.urd.md` and `scenes/tavern.urd.md` both yield `tavern`), the compiler emits an error (URD203): *"File stem collision: 'tavern' is produced by both content/tavern.urd.md and scenes/tavern.urd.md. Rename one file to avoid section ID conflicts."* This is checked during IMPORT, after all files are discovered but before LINK begins.

Section IDs must be unique within the compiled world. Since section names must be unique within a file and file stems are enforced unique within a compilation unit, this uniqueness is guaranteed by construction. The compiler does not need a separate uniqueness check for section IDs — it follows from the two enforced constraints.

### Choice IDs

Choice IDs are derived from the section ID and the choice label.

```
choice_id = section_id + "/" + slugify(label)
```

Example: "Ask about the harbor" in section `tavern/topics` → `"tavern/topics/ask-about-the-harbor"`.

**Slugification rules:**
- Lowercase the label.
- Replace spaces with hyphens.
- Strip characters that are not alphanumeric or hyphens.
- Collapse consecutive hyphens.
- Trim leading and trailing hyphens.

If two choices in the same section produce identical slugified IDs, the compiler emits an error (URD501). The author must disambiguate by changing one label.

### Location IDs

Location IDs are derived from the `# Heading` text by slugifying it.

```
location_id = slugify(display_name)
```

Example: `# The Rusty Anchor` → `"the-rusty-anchor"`.

The same slugification rules as choice IDs apply. Location IDs must be unique within the compilation unit.

### Action and Rule IDs

Actions and rules declared in frontmatter use their declared key names directly. Actions inferred from choice syntax (e.g., `* Pick a door -> any Door`) are assigned IDs by the compiler using the convention `section_id/slugified-label` — identical to choice IDs. The derivation logic is defined below.

### Choice-to-Action Compilation

Choices in Schema Markdown are an authoring convenience — they represent player-facing actions with conditions and effects. The compiler transforms choices into actions in the `actions` block of the compiled JSON. This transformation is split across LINK and EMIT.

**When a choice generates an action:** Every `Choice` node that belongs to a section generates an `ActionSymbol`. The choice's conditions, effects, and target become the action's conditions, effects, and target. The choice's label becomes the action's display text (stored in the `description` field). This is a 1:1 mapping — every choice produces exactly one action.

**Action ID derivation:** The action ID for a choice-derived action is the same as the choice ID:

```
action_id = section_id + "/" + slugify(label)
```

Example: choice "Pick up the rusty key" in section `tavern/topics` → action ID `"tavern/topics/pick-up-the-rusty-key"`.

**LINK's responsibility:** During the collection sub-pass, when LINK encounters a `Choice` node inside a section, it:

1. Creates the `ChoiceSymbol` (as documented in the LINK brief).
2. Creates a corresponding `ActionSymbol` with:
   - `id`: same as the choice's `compiled_id`.
   - `target`: if the choice has `target` (an entity ref via `-> @entity`), set to the entity ref string.
   - `target_type`: if the choice has `target_type` (via `-> any TypeName`), set to the type name string.
   - `declared_in`: the choice's span.
3. Registers the `ActionSymbol` in `symbol_table.actions`.

If the choice has neither `target` nor `target_type` (its content is inline conditions/effects/jumps), both fields are `null` on the `ActionSymbol`.

Nested choices (sub-choices) also generate actions. Their IDs follow the same derivation but are scoped to the parent section, not the parent choice — all choices in a section produce actions at the same level in the `actions` block.

**EMIT's responsibility:** When building the `actions` block, EMIT iterates `symbol_table.actions` in insertion order. For choice-derived actions:

1. The JSON key is the action's `compiled_id`.
2. `description` is the choice label text.
3. `target` and `target_type` are emitted per the standard action rules (mutually exclusive, omit if absent).
4. `conditions` are lowered from the choice's condition nodes.
5. `effects` are lowered from the choice's effect nodes.

Frontmatter-declared actions and choice-derived actions coexist in the same `actions` block. Both are `ActionSymbol`s in the symbol table — EMIT does not distinguish their origin.

**Duplicate action IDs:** If a frontmatter-declared action has the same ID as a choice-derived action, LINK detects this as a duplicate during `symbol_table.actions` registration and emits a diagnostic. The standard duplicate-detection rules apply (first wins, second recorded in duplicates list).

### Sequence and Phase IDs

Sequence IDs are derived from `## Heading` by slugification. Phase IDs are derived from `### Heading` by slugification. Both must be unique within their parent scope (phases within a sequence, sequences within a file).


## Error Recovery Strategy

The compiler must not stop at the first error. It must collect as many diagnostics as possible in a single run. This section defines the recovery strategy at each phase boundary and the general principles that all phases follow.

### General Principles

**Mark and continue.** When a construct fails, mark it as damaged (with a flag or sentinel value) and move on to the next construct. Do not attempt to interpret the damaged construct in later phases — skip it.

**No cascading errors.** If LINK fails to resolve `@guard`, VALIDATE must not emit "property `mood` not found on unknown entity" for every reference to `@guard.mood`. The rule: if a construct depends on an unresolved reference, silently skip validation of that construct. One root cause, one error.

**Synchronisation points.** When the parser encounters an unrecoverable syntax error, it skips forward to the next recognisable boundary: a blank line followed by a known sigil, a heading, or a section label. The partial AST contains an `ErrorNode` at the damaged span, and parsing continues.

### Phase-Specific Recovery

| Phase | Recovery Strategy |
|-------|-------------------|
| PARSE | Synchronisation-point recovery. Skip to next line boundary on error. Produce partial AST with `ErrorNode` markers. |
| IMPORT | If a file is missing, emit diagnostic and continue without that file's declarations. If a cycle is detected, break the cycle edge and continue. |
| LINK | If a declaration is duplicate, keep the first one and mark the second as conflicted. If a reference is unresolved, set annotation to `null` and continue. |
| VALIDATE | Skip validation of any construct whose annotation is `null` (unresolved reference). Report all other errors. |
| EMIT | If any Error-severity diagnostic exists, do not produce JSON output. Return diagnostics only. |


## Incremental Compilation Readiness

The architecture document mandates that the compiler must not preclude incremental compilation, even though v1 does a full recompile. The following design decisions are locked to avoid costly retrofits later.

### What Is Decided Now

**The dependency graph is cacheable.** IMPORT builds it and the compiler can persist it between runs. When a file changes, only that file and its transitive dependents need recompilation.

**Stable IDs.** Entity, section, and choice IDs are derived from declared names and file paths, not from declaration order. A recompile of one file does not invalidate references from unchanged files.

**The symbol table supports incremental updates.** Symbols are keyed by their declaring file. When a file is recompiled, its symbols are removed from the table and re-added from the new AST. Symbols from unchanged files remain untouched.

**Cache invalidation boundaries.** When a type definition changes, all entities of that type are revalidated. When an entity changes, all references to it are rechecked. The dependency graph provides the file-level boundary; the symbol table provides the declaration-level boundary.

### What Is Deferred

**Partial re-parsing.** v1 re-parses the entire changed file. A future version could re-parse only the changed region using the AST's span information.

**Incremental LINK and VALIDATE.** v1 re-runs LINK and VALIDATE on all files. A future version could limit re-checking to the transitive dependents of changed files.

**AST diffing.** Not needed for v1. The dependency graph and file-level invalidation are sufficient.


## Source Map Readiness

The architecture document specifies that source maps are not required for v1 but the data model must leave room for them. The AST's span information is the foundation — every node knows where it came from. EMIT can optionally produce a companion source map file that maps JSON paths to source spans.

### What Is Decided Now

**Every emitted JSON element can be traced back to an AST node.** EMIT traverses the AST, not a derived data structure. This means the source span of every JSON element is available at emission time.

**The JSON output format has no room for inline source data.** Source maps, if produced, are a separate file (e.g., `world.urd.json.map`) that references the JSON output by path.

### What Is Deferred

The source map format. Whether it uses the JavaScript source map v3 convention or a custom format is a decision for the EMIT phase brief.


## Input Limits

The compiler enforces limits on input size and structural complexity to prevent pathological inputs from exhausting memory or stack space.

| Limit | Value | Diagnostic |
|-------|-------|-----------|
| Maximum file size | 1 MB per `.urd.md` file | URD103: *"File exceeds 1 MB size limit."* |
| Maximum files in compilation unit | 256 files | URD205: *"Compilation unit exceeds 256 files."* |
| Maximum import depth | 64 levels | URD204: *"Import depth limit exceeded."* |
| Maximum choice nesting depth | Error at 4 levels (warn at 3) | URD403: *"Nesting depth 4 exceeds maximum."* |
| Maximum frontmatter nesting depth | 8 levels | URD104: *"Frontmatter nesting exceeds 8 levels."* |

**When limits are checked:** File size (URD103) is checked immediately after reading a file from disk, before parsing begins. File count (URD205) is checked by IMPORT after the dependency graph is fully discovered but before LINK begins. Import depth (URD204) is checked during graph traversal. Nesting depths (URD403, URD104) are checked during PARSE and VALIDATE respectively.

These limits are deliberately generous for any real-world project. They exist to prevent denial-of-service style inputs, not to constrain normal authoring.


## Memory Lifecycle

The v1 compiler is a batch process: it runs, produces output, and exits. All ASTs, the symbol table, and the dependency graph are allocated for the duration of one `compile()` call and freed when it returns.

For long-running tooling (the LSP server in Phase 3), the memory model changes: the compiler must retain ASTs and the symbol table across recompilations to support incremental updates. The following design decisions support this future without constraining v1:

**Symbols are keyed by declaring file.** When a file changes, its symbols can be removed and re-added without touching other files' symbols. This is the incremental invalidation boundary.

**ASTs are owned by the dependency graph.** Each `FileNode` owns its `FileAST`. Replacing a file's AST means replacing the `FileNode`, which automatically drops the old AST.

**The diagnostic collector is cleared per compilation.** In LSP mode, each recompilation produces a fresh diagnostic set. Stale diagnostics from previous runs are never displayed.

The LSP server's memory management strategy (cache eviction, partial invalidation, working set limits) is deferred to the developer tooling brief. The compiler architecture does not preclude any reasonable approach.


## Parallelization Opportunities

The v1 compiler is single-threaded. The architecture does not mandate parallelism but identifies where it would be straightforward to add in a future optimization pass.

**PARSE is embarrassingly parallel.** Each file is parsed independently with no shared mutable state. A thread pool parsing N files simultaneously would produce N independent `FileAST` results. The only coordination point is collecting the results before IMPORT begins.

**VALIDATE is mostly parallel per file.** Type checking within a file depends on the global symbol table (read-only at this point), not on other files' validation results. Files can be validated concurrently with no synchronization beyond read access to the symbol table.

**LINK is inherently sequential.** Declaration collection must process files in topological order so that imported types exist before referencing files are processed. Reference resolution could potentially be parallelized, but the benefit is marginal for v1 project sizes.

**EMIT is sequential.** JSON emission traverses the merged AST in a fixed order to produce deterministic output. Parallelism here would complicate ordering guarantees without meaningful performance benefit.


## Deterministic Output

The architecture document requires that the same `.urd.md` source files always produce byte-identical `.urd.json` output. This constrains how EMIT orders keys and arrays.

### Rules

**Object keys are emitted in declaration order.** Types appear in the order they are declared in source. Entities in declaration order. Rules in declaration order. This is the natural traversal order of the AST.

**Cross-file ordering follows topological import order.** If file A imports file B, B's declarations appear before A's in the output (because B was processed first). Within a file, declaration order is preserved. When multiple files are at the same topological depth, ties are broken by alphabetical order of normalized file path (see Topological Order in the Dependency Graph section).

**The `world` block is always first.** Regardless of declaration order, `world` is the first key in the JSON output. The remaining blocks follow a fixed order: `types`, `entities`, `locations`, `rules`, `actions`, `sequences`, `dialogue`.

**No non-deterministic sources in output.** The compiler does not generate timestamps, random IDs, or platform-dependent values. All IDs are derived from source content.


## The Compiler Interface

The top-level interface that orchestrates the phases:

```
compile(entry_file: FilePath) → CompilationResult

CompilationResult {
  success: boolean,
  world: UrdWorldJSON | null,    // null if any errors
  diagnostics: Diagnostic[],
}
```

### Orchestration Sequence

```
function compile(entry_file):
  diagnostics = new DiagnosticCollector()

  // Phase 1: PARSE
  entry_ast = parse(entry_file, diagnostics)
  if entry_ast is null:
    return result(false, null, diagnostics)

  // Phase 2: IMPORT
  graph = resolve_imports(entry_ast, diagnostics)
  // graph contains all FileASTs in topological order

  // Phase 3: LINK
  symbol_table = new SymbolTable()
  // Pass 1: Collection — register all declarations
  for file_ast in graph.topological_order():
    collect_declarations(file_ast, symbol_table, diagnostics)
  // Pass 2: Resolution — resolve all references
  for file_ast in graph.topological_order():
    resolve_references(file_ast, symbol_table, diagnostics)

  // Phase 4: VALIDATE
  validate(graph, symbol_table, diagnostics)

  // Phase 5: EMIT
  if diagnostics.has_errors():
    return result(false, null, diagnostics)
  json = emit(graph, symbol_table, diagnostics)
  return result(true, json, diagnostics)
```

Note how LINK is split into two sub-passes (`collect_declarations` and `resolve_references`), corresponding to the two-pass model described earlier.


## Player Entity Resolution and World References

### The `world.start` and `world.entry` Fields

`world.start` is a **location ID** (the slugified form, e.g., `"cell"`, not the display name `"Cell"`). VALIDATE checks that it resolves to an existing `LocationSymbol`. If it does not, the compiler emits an error (URD404): *"world.start references 'cell' but no location with that ID exists."*

`world.entry` is a **sequence ID** (also slugified). VALIDATE checks that it resolves to an existing `SequenceSymbol`. If absent, the world is freeform.

In Schema Markdown source, the author writes `start: cell` in frontmatter. The compiler treats this as a location ID directly — it does not slugify the value, because frontmatter values for `start` and `entry` are already expected to be identifiers, not display names.

### Player Entity

The player entity is special-cased in the compiler. The rules (from the Schema Specification) are:

1. If no explicit `@player` entity is declared, the compiler does **not** emit a player entity. The runtime creates one implicitly at load time with type `Player`, traits `[mobile, container]`, and starting container set to `world.start`.
2. If an explicit `@player` entity is declared, it replaces the implicit one entirely. The compiler emits it as a normal entity. VALIDATE checks that its type has `mobile` and `container` traits.
3. Duplicate `@player` declarations across imported files are a compile error, following the standard duplicate entity ID rule.

The distinction between options 1 and 2 matters for EMIT: when no player is declared, the `entities` block in JSON does not contain a player entry. The runtime handles creation.


## What the Phase Briefs Must Define

Each subsequent phase brief must cover:

1. **Input.** Exactly what the phase receives from the previous phase.
2. **Output.** Exactly what the phase produces for the next phase.
3. **Guarantees.** What properties hold about the output that the next phase can rely on.
4. **Error cases.** Every diagnostic the phase can emit, with code, message template, and recovery action.
5. **Acceptance criteria.** Specific test inputs and expected outputs that prove the phase works.

The phase brief sequence:

| Brief | Input | Output | Key Responsibility |
|-------|-------|--------|-------------------|
| PARSE | `.urd.md` source text | `FileAST` | Syntactic validity. Span tracking. Error recovery to line boundaries. |
| IMPORT | Entry `FileAST` | `DependencyGraph` + all `FileAST`s | File resolution, cycle detection, import ordering. |
| LINK | `DependencyGraph` + `FileAST`s | `SymbolTable` + annotated ASTs | Declaration collection, reference resolution, duplicate detection, ID derivation. |
| VALIDATE | Annotated ASTs + `SymbolTable` | Validation diagnostics | Type checking, enum range, ref target, mutual exclusion, nesting depth. |
| EMIT | Validated ASTs + `SymbolTable` | `.urd.json` string | JSON generation, block ordering, `urd` version injection, deterministic output. |

### Dependency Between Briefs

PARSE can be written first — it depends only on the PEG grammar (already complete) and the AST node types (defined in this document).

IMPORT depends on PARSE's output format.

LINK depends on IMPORT's output format and the symbol table design.

VALIDATE depends on LINK's annotation format and the symbol table's type information.

EMIT depends on everything upstream, plus the JSON Schema (already validated and production-ready).

The recommended writing order is: **PARSE → IMPORT → LINK → VALIDATE → EMIT**, each building on the previous brief's output contract.


## Worked Example: The Two Room Key Puzzle Through All Phases

This section traces a concrete source file through every phase of the compiler to anchor the abstract data structures in a real scenario. The example uses a simplified single-file version of the Two Room Key Puzzle.

> **Note:** Some transformations described here (such as `in here` expanding to `player.container`, choices compiling to actions with derived IDs, and exit conditions lowering to string expressions) are illustrative of the compiler's end-to-end behaviour. The precise transformation rules are defined in the individual phase briefs and the Schema Markdown Syntax Specification's Compiler Mapping table. This example shows *what happens*, not the normative *how*. Every transformation mentioned is contingent on the phase briefs — the only properties guaranteed by this example are structural conformance to the JSON Schema and deterministic output.

### Source Input

```
---
world: two-room-key
start: cell

types:
  Key [portable]:
    name: string

  LockedDoor [interactable]:
    locked: bool = true
    requires: ref(Key)

entities:
  @rusty_key: Key { name: "Rusty Key" }
  @cell_door: LockedDoor { requires: @rusty_key }
---

# Cell

A dim stone cell.

[@rusty_key, @cell_door]

* Pick up the rusty key -> @rusty_key

  ? @rusty_key in here

  > move @rusty_key -> player

* Use the key on the door -> @cell_door

  ? @rusty_key in player

  ? @cell_door.locked == true

  > @cell_door.locked = false

  > destroy @rusty_key

-> north: Corridor

  ? @cell_door.locked == false

  ! The iron door is locked.

# Corridor

A long corridor. Daylight leaks from the far end.

-> south: Cell
```

### Phase 1: PARSE

The parser produces a `FileAST` with span-tracked nodes. No references are resolved — the parser only knows syntax.

Key nodes produced (abbreviated):
- `Frontmatter` containing `WorldBlock { name: "two-room-key", start: "cell" }`, two `TypeDef` nodes (Key, LockedDoor), two `EntityDecl` nodes (rusty_key, cell_door).
- `LocationHeading { display_name: "Cell" }` with child `Prose { text: "A dim stone cell." }`.
- `EntityPresence { entity_refs: ["rusty_key", "cell_door"] }`.
- Two `Choice` nodes (pick up key, use key on door), each containing `Condition` and `Effect` children.
- `ExitDeclaration { direction: "north", destination: "Corridor" }` with a `Condition` and `BlockedMessage` child.
- `LocationHeading { display_name: "Corridor" }` with an `ExitDeclaration { direction: "south", destination: "Cell" }`.

Every node carries a `Span` with file path, line, and column. No annotation slots are filled.

### Phase 2: IMPORT

This is a single-file world with no `import:` declarations. IMPORT produces a trivial dependency graph: one node (the entry file), no edges. The topological order is a single-element list.

File stem extraction: `two-room-key.urd.md` → stem `two-room-key`. Registered for uniqueness checking.

### Phase 3: LINK

**Collection sub-pass:** LINK walks the AST and populates the symbol table:

- `TypeSymbol("Key")` with traits `[portable]`, property `name: string`.
- `TypeSymbol("LockedDoor")` with traits `[interactable]`, properties `locked: boolean = true` and `requires: ref(Key)`.
- `EntitySymbol("rusty_key")` with type_name `"Key"`, overrides `{ name: "Rusty Key" }`.
- `EntitySymbol("cell_door")` with type_name `"LockedDoor"`, overrides `{ requires: "rusty_key" }`.
- `LocationSymbol("cell")` derived from `# Cell` by slugification.
- `LocationSymbol("corridor")` derived from `# Corridor` by slugification.
- No sections (no `==` labels in this file), no sequences, no named rules.

**Resolution sub-pass:** LINK walks the AST again and fills annotation slots:

- `@rusty_key` in `EntityPresence` → resolves to `EntitySymbol("rusty_key")`.
- `@cell_door` in `EntityPresence` → resolves to `EntitySymbol("cell_door")`.
- `@rusty_key` in choice target → resolves to `EntitySymbol("rusty_key")`.
- `@rusty_key` in condition `@rusty_key in here` → resolves entity, and `in here` is recognized as `container == player.container`.
- `@cell_door.locked` in condition → resolves entity, resolves `locked` to `PropertySymbol("locked")` on type `LockedDoor`.
- Exit `north: Corridor` → resolves destination `"Corridor"` via slugification to `LocationSymbol("corridor")`.
- Exit `south: Cell` → resolves destination `"Cell"` to `LocationSymbol("cell")`.

No `@player` entity declared → noted for EMIT (runtime creates it implicitly).

### Phase 4: VALIDATE

VALIDATE checks semantic constraints against the resolved annotations:

- `rusty_key` declares type `Key` → Key exists ✓
- `rusty_key` overrides `name` → Key has property `name: string` → override value `"Rusty Key"` is a string ✓
- `cell_door` declares type `LockedDoor` → LockedDoor exists ✓
- `cell_door` overrides `requires` → LockedDoor has property `requires: ref(Key)` → override value `rusty_key` is an entity of type `Key` ✓
- Condition `@rusty_key in here` → `rusty_key` is an entity, `container` is a valid implicit property ✓
- Condition `@cell_door.locked == true` → `locked` is `boolean`, `true` is a valid boolean literal ✓
- Effect `> @cell_door.locked = false` → `locked` is `boolean`, `false` is valid ✓
- Effect `> destroy @rusty_key` → `rusty_key` is an entity ✓
- Effect `> move @rusty_key -> player` → `rusty_key` has trait `portable` ✓, `player` is a container ✓
- No actions have both `target` and `target_type` → mutual exclusion ✓
- Choice nesting depth: maximum 1 level → within limit ✓

Zero diagnostics. Compilation continues to EMIT.

### Phase 5: EMIT

EMIT traverses the annotated AST and symbol table to produce JSON. Key transformations:

- `world` block emitted first. `urd: "1"` injected automatically. `start: "cell"` preserved.
- `types` block: `Key` and `LockedDoor` emitted with full property schemas, traits, and defaults.
- `entities` block: `rusty_key` and `cell_door` emitted with type references and property overrides. No player entity emitted (runtime creates it).
- `locations` block: `cell` and `corridor` emitted with contains lists, exits, conditions, and blocked messages. `in here` expanded to `player.container`. `@rusty_key` expanded to `rusty_key` (stripped `@`).
- `actions` block: the two choices compile to actions. `pick-up-the-rusty-key` and `use-the-key-on-the-door` with target, conditions, and effects.
- No `rules`, `sequences`, or `dialogue` blocks — these are omitted from output (the JSON Schema allows all blocks except `world` to be absent).

The output JSON is validated against the Urd World Schema. Every structural rule passes. The file is byte-identical across repeated compilations.


## Relationship to Existing Artifacts

| Artifact | Relationship to This Brief |
|----------|---------------------------|
| PEG Grammar (formal grammar brief) | Defines the syntactic rules PARSE implements. The AST node types in this brief are the structured output of parsing those rules. |
| JSON Schema (validated) | Defines the structural contract EMIT must produce. Every JSON element in the output must conform to the schema. |
| Architecture Document | The authoritative system blueprint. This brief implements the compiler section of that document as internal engineering specification. |
| Schema Specification | The normative contract for what valid `.urd.json` looks like. VALIDATE enforces its semantic rules. EMIT produces its structural output. |
| Schema Markdown Specification | The normative contract for what valid `.urd.md` looks like. PARSE accepts its syntax. LINK implements its import and scoping rules. |
| Wyrd Reference Runtime | Consumes the compiler's output. The compiler does not need to know how the runtime works, but it must produce output that the runtime can load. |

## Acceptance Criteria

This brief is validated when:

1. **All AST node types are enumerated.** Every syntactic construct in the PEG grammar maps to an AST node type defined here. No construct is left without a node.
2. **The symbol table structure is complete.** Every named construct that can be referenced (types, entities, sections, locations, actions, rules, sequences) has a symbol type with all fields specified.
3. **ID derivation rules are unambiguous.** Given any source construct, the compiled ID can be computed without reading any other document.
4. **Diagnostic code ranges are non-overlapping.** Each phase has exclusive ownership of its code range.
5. **Phase interface contracts are clear.** An engineer can read this document and know exactly what data structure to expect as input and what to produce as output for any phase.
6. **The five phase briefs can be written independently.** After reading this document, a writer of any phase brief has all the cross-cutting information they need. No phase brief needs to define shared data structures — they reference this document.

*End of Brief*
