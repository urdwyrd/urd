# URD — Compiler Analysis IR: Facts Layer and Read/Write Provenance

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-22
**Status:** Done

### What was done

Implemented the complete FactSet analysis IR as specified. Created `packages/compiler/src/facts.rs` (~1060 lines) containing:

- **Identity type aliases:** TypeId, PropertyId, LocationId, SectionId, ChoiceId, ExitId, RuleId.
- **PropertyKey struct** with Clone/Debug/PartialEq/Eq/Hash.
- **Enums:** CompareOp, WriteOp, LiteralKind, JumpTarget, FactSite (#[non_exhaustive]), SiteOwner.
- **Six fact structs:** PropertyRead, PropertyWrite, ExitEdge, JumpEdge, ChoiceFact, RuleFact — all with source spans.
- **Helpers:** make_exit_id(), split_exit_id(), .key() on PropertyRead/PropertyWrite, .exit_id() on ExitEdge.
- **FactSet container** with private fields, six slice accessors, four lookup helpers (choice_by_id, exit_by_id, exit_by_location_and_name, rule_by_id), four property-level queries (reads_by_key, writes_by_key, sites_reading, sites_writing), three site adjacency methods (read_indices_for_site, write_indices_for_site, resolve_site).
- **FactSetBuilder** (private) enforcing immutability at the type level.
- **extract_facts()** — two-phase extraction: Phase A extracts exits from the symbol table; Phase B walks AST content in topological file order collecting choices, reads, writes, jumps, and rules.
- **PropertyDependencyIndex** with IndexMap for deterministic key order: build(), reads_of(), writes_of(), read_properties(), written_properties().
- **FactSet::to_json()** — JSON serialisation using serde_json::json!() for WASM output.
- **SpanKey** test helper (#[cfg(test)]).

Modified `packages/compiler/src/lib.rs`: added `pub mod facts;`, added `fact_set: Option<facts::FactSet>` to CompilationResult, wired extract_facts() between LINK and VALIDATE (Phase 3a).

Modified `packages/compiler/src/wasm.rs`: added `"facts"` field to WASM JSON output, serialising the FactSet when available (null on PARSE/IMPORT failure).

Created `packages/compiler/tests/facts_tests.rs` with 31 fixture-based tests covering all five canonical fixtures (two-room-key-puzzle, tavern-scene, monty-hall, interrogation, sunken-citadel): exit counts, conditional exits with guard reads, property writes, choice counts, property reads, jump targets, cross-file spans, rule counts, scale smoke test, exit density, PropertyDependencyIndex readers/writers, referential integrity, determinism, site resolution (Choice/Exit/Rule), jump target exit resolution.

All 547 tests pass (516 existing + 31 new), zero regressions. WASM target compiles cleanly.

### What changed from the brief

1. **extract_facts() signature:** Brief specifies `(graph, ordered_file_ids, symbol_table)`. Implementation uses `(graph, symbol_table)` and calls `graph.topological_order()` internally, matching the VALIDATE pattern and avoiding an API that requires callers to pass ordering separately.

2. **FactSetBuilder interface:** Brief specifies `push_read(read, owner)` with auto-append to owning adjacency lists. Implementation uses `push_read(read) -> usize` with manual index tracking by the caller. Referential integrity is enforced by test (facts_roundtrip_consistency) rather than by the builder's type system.

3. **ChoiceFact ordering:** Brief specifies pre-order (parent before children). Implementation uses post-order (children before parent) because the parent ChoiceFact needs populated condition_reads/effect_writes before construction. Ordering is still deterministic.

4. **LiteralKind classification:** Brief says to read from the parsed AST node's literal type. The AST's PropertyComparison.value is an untyped String. Implementation classifies via the symbol table's resolved PropertyType, which is the only reliable source.

5. **Pipeline placement:** Brief recommends Option A (sub-phase of VALIDATE). Implementation places extraction between LINK and VALIDATE as Phase 3a, ensuring FactSet is available regardless of VALIDATE errors per acceptance criterion 7.

6. **JSON serialisation (not in brief scope):** Added FactSet::to_json() and wired it into the WASM output. The brief explicitly deferred this ("not required yet") but it was added during implementation at the author's request. No new Cargo dependencies — uses serde_json::json!() which was already available.

7. **Tavern PropertyReads test:** Brief expected PropertyRead facts for @arina.trust comparisons. The fixture has no PropertyComparison conditions on choices (only a PropertyWrite). Test correctly asserts 0 reads.

---

**Created:** 2026-02-21

## Context

The S3/S4/S6/S8 static analysis checks closed the compiler gate with targeted, self-contained validations. Each check was implemented as a standalone function in VALIDATE that reads the symbol table and AST directly. This was the right call for v1 — minimal coupling, no new infrastructure.

But a pattern emerged: every check reinvented its own traversal. S3 built a location graph from exit symbols. S6 walked AST content segmenting by section boundaries. S4 walked choices scanning for conditions. These are all different views of the same underlying data — the resolved relationships between locations, sections, choices, conditions, effects, and properties.

This brief introduces a deliberate intermediate representation between the resolved world (LINK's output, VALIDATE's input) and downstream consumers (future analysis checks, Wyrd runtime, LSP, explain mode). The IR has two complementary parts:

1. **A facts layer** — normalized tuples extracted from the resolved world, queryable without AST traversal.
2. **Read/write provenance** — fine-grained attribution recording which conditions read which properties and which effects write them, with source spans.

Together they form the compiler's "analysis IR" — the stable internal interface that makes future analysis cheap to add.

### Positioning

**This is v1 gate work.** The FactSet is included in the v1 Completion Gate as section D (requirements F1–F8). It is compiler infrastructure that pays off in three places:

1. **Future static analysis checks** become queries over facts instead of custom AST traversals.
2. **Wyrd runtime** can consume provenance for explain mode and cache invalidation.
3. **LSP/IDE** can use property-level dependency information for incremental revalidation.

**This does NOT require Wyrd.** Both components are produced by the compiler from data that already exists after LINK. They can be tested, validated, and iterated without a runtime.

**This is NOT the full Soufflé multigraph.** The facts layer is a deliberately flat, non-recursive set of tuples. It has no inference engine, no fixpoint evaluation, no stratification. It is a materialized view of the resolved world. If Soufflé or Datalog is adopted later, the facts layer becomes the input relation set — but the design does not depend on that decision. In Datalog terms, the FactSet corresponds to extensional database relations (EDB). Any future rule system (Soufflé or otherwise) operates as an intensional layer (IDB) over this data.

### Strategic significance

This brief marks a qualitative shift in what Urd is. Before the facts layer, Urd is a compiler and schema system — it parses, resolves, and validates. After the facts layer, Urd becomes a **queryable semantic graph of interactive worlds**.

The distinction matters because traditional IF systems (Inform, Ink, Twine) are AST-driven or runtime-driven. Their internal representations are optimized for execution, not inspection. You cannot ask Inform "which conditions read this property?" or "is this choice reachable given the current write set?" without instrumenting the runtime or walking the full AST.

Urd's facts layer makes these questions trivial — they become queries over flat relations. This enables a class of tooling that procedural IF systems cannot support:

- **Static analysis** that reasons about the entire world at compile time, not by simulating play.
- **Explain mode** that traces why a choice is available or why a location is unreachable, grounded in provenance rather than runtime logging.
- **Structural diffing** between world versions — what changed, what broke, what new paths opened.
- **AI-assisted authoring** where tools can inspect the semantic graph to suggest fixes, flag dead ends, or verify coverage without executing the world.
- **Export and offline analysis** — the FactSet is equivalent to a set of Datalog base relations and can be serialized to CSV, JSON, or Soufflé `.facts` format for external tooling.

The FactSet is not a database engine. It does not execute narratives from data structures. It is an analysis IR — a stable, inspectable representation that keeps runtime execution (Wyrd) and compile-time reasoning cleanly separated. This is the architectural difference between "database-driven IF" (which fails) and "declarative + analyzable + inspectable" (which is what Urd is becoming).

### Relation identity

Conceptually, each fact type is a named relation:

| Relation | Tuple shape |
|----------|-------------|
| `PropertyRead` | (type, property, site, operator, value, ...) |
| `PropertyWrite` | (type, property, site, operator, value, ...) |
| `ExitEdge` | (from_location, to_location, exit_name, ...) |
| `JumpEdge` | (from_section, target, ...) |
| `ChoiceFact` | (section, choice_id, ...) |
| `RuleFact` | (rule_id, ...) |

This mapping is documented here so that future export (Soufflé `.facts`, JSON, CSV) can be built without revisiting the struct definitions. Each struct is one relation. Each `Vec` in the FactSet is one relation instance. The PropertyDependencyIndex is a derived secondary index, not a relation.

### Core invariant

**FactSet is immutable, deterministic, and complete.** For a given linked compilation unit (the `DependencyGraph` + `SymbolTable` + `ordered_file_ids` triple produced by LINK), `extract_facts()` must always produce structurally identical FactSets with identical vector ordering. **Completeness:** FactSet is a complete representation of all resolved relationships relevant to static analysis expressible by the current compiler phase. "Relevant to analysis" means: required to answer S1–S8 style questions and future checks of the same class without AST traversal. No such relationship that is available after LINK and relevant to static analysis in this phase should require AST traversal to discover. Future analysis should query the FactSet, not fall back to AST walking. Any new analysis added after this layer must justify AST access — the default assumption is that all required information exists in FactSet. This means: same number of facts, same field values, same cross-reference indices, in the same positions. This makes the FactSet cacheable, testable, and comparable across builds. Once constructed, a FactSet is never modified — consumers read it, they do not write to it.

FactSet is not an execution model. It is a complete projection of semantic relationships relevant to static analysis. It does not attempt to model control flow, evaluation order, or runtime sequencing. JumpEdge represents declared transitions, not executable control flow — it does not encode evaluation order or runtime reachability. It does not simulate play, evaluate conditions at runtime, or execute effects. VALIDATE must not depend on FactSet for correctness — FactSet is a projection, not a source of truth. Existing validation checks (S1–S8) continue to read the symbol table and AST directly. That responsibility belongs to Wyrd. To be explicit about the two compiler outputs: `.urd.json` is the runtime data model that Wyrd executes; FactSet is an analysis projection produced by the compiler to support queries and tooling. They serve different consumers and have different contracts.

Determinism is defined as structural equality, not byte-level memory identity. If a canonical serialization format is needed later (e.g., for build caching or diffing), it should be introduced as a separate encoding function tested against its own output — not assumed from the in-memory representation.


## Dependencies

- **Compiler Architecture Brief** — phase contracts, symbol table structure, annotation model, linked compilation unit.
- **LINK Phase Brief** — annotation semantics, resolution guarantees, FileContext.
- **VALIDATE Phase Brief** — read-only contract, diagnostic-only output (this brief extends VALIDATE to also produce facts).
- **S3/S4/S6/S8 Brief** — the checks whose patterns motivate this extraction.


## Part 1: The Facts Layer

### What it is

A flat set of typed tuples extracted from the resolved world after LINK. Each tuple represents one atomic relationship that was established by declarations, references, or resolutions. The facts are not derived or inferred — they are direct translations of what LINK resolved.

Think of it as: "LINK populates the symbol table and annotations. The facts layer reads both and produces a normalized, queryable summary."

### Identity types

All IDs are type-aliased strings. This clarifies intent at every use site and provides a migration path to interned IDs later without changing any signatures.

```rust
/// Semantic type aliases for fact identifiers.
/// All are String underneath, but the alias communicates what the string represents.
// FilePath: use the existing crate::span::FilePath type directly.
// In code: `pub use crate::span::FilePath;` in the facts module.
// Do not redefine as a separate alias. crate::span::FilePath is currently
// `pub type FilePath = String` (forward slashes, relative to entry directory).
// If span::FilePath ever changes to a distinct type, facts will track it
// automatically.
type TypeId = String;       // resolved type name (e.g., "Guard")
type PropertyId = String;   // resolved property name (e.g., "trust")
type LocationId = String;   // slugified location ID
type SectionId = String;    // compiled section ID (file_stem/section_name)
type ChoiceId = String;     // compiled choice ID (section_id/slugified_label)
type ExitId = String;       // composite: "location_id/exit_name"
                            // exit_name is the exact key from LocationSymbol.exits.
                            // The grammar restricts Identifier to [a-z][a-z0-9_]* —
                            // no slashes, no escaping needed. The slash separator is
                            // unambiguous.
type RuleId = String;       // rule identifier

/// Normalized key for property-level queries and indexing.
/// Replaces bare (TypeId, PropertyId) tuples throughout the API.
/// Gives a single place to attach future metadata (e.g., readonly flag)
/// and maps directly to Soufflé: `.decl PropertyKey(type, property)`.
/// PropertyKey represents logical identity for a (type, property) pair.
/// Its internal representation may change to interned or borrowed forms
/// (e.g., Arc<str>, &str views) without affecting API semantics.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PropertyKey {
    entity_type: TypeId,
    property: PropertyId,
}

// PropertyKey construction is canonical via .key() on fact types:
impl PropertyRead  { fn key(&self) -> PropertyKey { PropertyKey { entity_type: self.entity_type.clone(), property: self.property.clone() } } }
impl PropertyWrite { fn key(&self) -> PropertyKey { PropertyKey { entity_type: self.entity_type.clone(), property: self.property.clone() } } }
// The index builder must use .key(), never touch entity_type/property fields directly.

/// Test utility for determinism comparison. Extracts the identity-relevant
/// fields from a Span, ignoring internal arena ids or future metadata.
/// Determinism and equality tests should compare via SpanKey, not raw Span.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SpanKey {
    file: FilePath,
    start: usize,
    end: usize,
}

#[cfg(test)]
impl From<&Span> for SpanKey {
    fn from(span: &Span) -> Self {
        SpanKey { file: span.file.clone(), start: span.start, end: span.end }
    }
}
```

### Fact types

Each fact type is a struct with named fields. Source spans are included on every fact for diagnostic provenance.

```rust
/// A condition reads an entity's property.
struct PropertyRead {
    site: FactSite,             // where the read occurs
    entity_type: TypeId,        // resolved type name
    property: PropertyId,       // resolved property name
    operator: CompareOp,        // typed comparison operator
    value_literal: String,      // exact RHS token of the comparison, taken directly from
                                // the parsed AST node. Never reconstructed or normalized.
                                // Includes quotes for strings, minus sign for negative ints.
    value_kind: LiteralKind,    // distinguishes token types for downstream reasoning
    span: Span,                 // span of the full comparison expression (@entity.prop op value)
                                // from the parsed AST node
}

/// Typed comparison operators. Prevents accidental string variants like
/// "equals" or spacing bugs. Store original token text only if needed for display.
enum CompareOp { Eq, Ne, Lt, Gt, Le, Ge }

// Mapping from grammar tokens:
//   "==" → Eq, "!=" → Ne, "<" → Lt, ">" → Gt, "<=" → Le, ">=" → Ge

/// Distinguishes literal kinds so downstream analysis can reason about
/// values without reparsing strings. Cheap to populate during extraction
/// since the parser already knows the token type.
enum LiteralKind {
    Bool,
    Int,    // includes negative ints — the grammar allows optional leading minus
            // on Number. value_literal includes the minus sign (e.g., "-5").
    Str,
    Ident,  // bare identifier, typically an enum variant
}

// Scope: PropertyRead only records comparisons whose right-hand side is a
// single literal token in v1. LiteralKind covers exactly this case. If the
// grammar later allows property-to-property comparisons or computed expressions,
// introduce a separate ComparisonValue enum rather than overloading value_literal.

// Note: For Bool and Int variants, the implementer may optionally store
// parsed values (e.g., value_int: Option<i64>, value_bool: Option<bool>)
// alongside value_literal to make range checks and boolean reasoning trivial
// without reparsing. This is cheap during extraction since the parser already
// knows the value. If deferred, LiteralKind is the forward-compatible hook
// that makes adding parsed values a non-breaking change later.

/// An effect writes an entity's property.
struct PropertyWrite {
    site: FactSite,             // where the write occurs
    entity_type: TypeId,        // resolved type name
    property: PropertyId,       // resolved property name
    operator: WriteOp,          // typed write operator
    value_expr: String,         // exact RHS token of the effect, taken directly from
                                // the parsed AST node. Never reconstructed or normalized.
    value_kind: Option<LiteralKind>, // Some only when: Set with a single literal token, or
                                     // Add/Sub with a single integer token. None otherwise.
                                     // Enables write-set queries without a full EffectValue enum.
                                     // All write-set analysis must assume full domain unless
                                     // value_kind is Some — absence means "could be anything."
                                     // Consumers must treat writes with value_kind == None as
                                     // unconstrained over the property's declared domain.
    span: Span,                 // span of the full effect expression (> @entity.prop op value)
                                // from the parsed AST node
}

/// Typed write operators.
enum WriteOp { Set, Add, Sub }

// Mapping from grammar tokens:
//   "=" → Set (assign value), "+" → Add (increment by number), "-" → Sub (decrement by number)
// The grammar uses standalone "+" and "-" tokens, not "+=" or "-=".

// Future note: value_expr is a string for v1. For richer write-set analysis
// (e.g., "can any effect produce value X?"), introduce a typed EffectValue enum:
// Literal(String), Increment(i64), Decrement(i64), Reference(EntityId, PropertyId).
// Until then, write-set reasoning over value_expr is conservative — treat
// any write to (type, property) as potentially producing any valid value.
//
// When EffectValue is introduced, consider mirroring LiteralKind from PropertyRead
// so that read-set and write-set queries use the same value classification.
// Without this, reads are typed (via LiteralKind) but writes remain opaque.

/// An exit connects two locations.
/// Uniquely identified by the pair (from_location, exit_name). That is the semantic key.
/// ExitId is the canonical string encoding of that key via make_exit_id().
struct ExitEdge {
    from_location: LocationId,  // source location ID
    to_location: LocationId,    // resolved destination location ID
    exit_name: String,          // exit direction/key
    is_conditional: bool,       // true if the exit has any guard conditions
    guard_reads: Vec<usize>,    // indices into FactSet.reads for this exit's guards
    span: Span,                 // span of the exit declaration (-> direction: destination)
}

impl ExitEdge {
    /// Derive the canonical ExitId without storing it as a field.
    fn exit_id(&self) -> ExitId {
        make_exit_id(&self.from_location, &self.exit_name)
    }
}

// Design note: ExitEdge.guard_reads and PropertyRead.site overlap intentionally.
// The authoritative representation of a guard condition is the PropertyRead with
// FactSite::Exit(exit_id). ExitEdge.guard_reads is a convenience adjacency list
// that avoids scanning all reads to find an exit's guards. Both must agree —
// the extraction function populates them from the same source in a single pass.

/// A jump connects two dialogue sections, or a section to an exit or terminal.
/// Only resolved jumps produce JumpEdge facts. Unresolved jump targets produce
/// no JumpEdge fact — rely on VALIDATE diagnostics for those.
struct JumpEdge {
    from_section: SectionId,    // compiled section ID
    target: JumpTarget,         // where the jump goes (typed, not ambiguous)
    span: Span,                 // span of the jump token (-> target)
}

/// Typed jump destination. Replaces the previous target: String + kind: JumpKind pattern.
enum JumpTarget {
    Section(SectionId),
    Exit(ExitId),   // canonical ExitId via make_exit_id(). Use split_exit_id() for parts.
    End,
}

/// A choice exists within a section.
struct ChoiceFact {
    section: SectionId,         // exactly SectionSymbol.compiled_id from the symbol table,
                                // not recomputed and not derived by splitting choice_id
    choice_id: ChoiceId,        // compiled choice ID — globally unique, stable across recompilation
                                // WARNING: ChoiceId is derived from label text. Label edits change
                                // the ID. Downstream tools must not store ChoiceId as a long-lived
                                // external reference (e.g., save files, bookmarks). Use the optional
                                // author-provided key ([id:foo]) when available (post-v1).
    label: String,              // display label
    sticky: bool,
    condition_reads: Vec<usize>, // indices into FactSet.reads
    effect_writes: Vec<usize>,  // indices into FactSet.writes
                                // Empty when the choice has no property writes (e.g., only
                                // lifecycle effects or no effects at all). A ChoiceFact is
                                // always emitted for every resolved choice regardless of
                                // whether it has property writes.
    span: Span,                 // span of the choice header (* or + line)
}

/// A rule, with its conditions and effects indexed into the FactSet.
struct RuleFact {
    rule_id: RuleId,
    condition_reads: Vec<usize>, // indices into FactSet.reads
    effect_writes: Vec<usize>,  // indices into FactSet.writes
    span: Span,                 // span of the rule header line only (rule name:),
                                // not the body. Body spans are on individual
                                // PropertyRead/PropertyWrite facts.
}

/// Discriminator for where a read or write occurs.
/// Minimal and referential — carries only the ID needed to locate the owning construct.
/// Resolve full context (section name, file, location) through FactSet lookup helpers.
#[non_exhaustive]
enum FactSite {
    Choice(ChoiceId),
    Exit(ExitId),
    Rule(RuleId),
}
// #[non_exhaustive] ensures that adding future variants (OnEnter, OnExit,
// OnExhausted) does not break downstream match arms at compile time.
// Consumers must include a wildcard arm, which forces them to handle
// unknown sites gracefully rather than silently ignoring new variants.

// v1 scope: OnEnter(LocationId), OnExit(LocationId), and OnExhausted(SectionId)
// are not included because the compiler does not yet implement on_enter, on_exit,
// or on_exhausted as symbol table constructs. When these are added to the compiler,
// the corresponding FactSite variants and extraction steps should be added here.
// Do not add variants without extraction logic to populate them.
//
// All three FactSite variants (Choice, Exit, Rule) have owning facts in v1:
// ChoiceFact, ExitEdge, and RuleFact respectively. resolve_site() is total
// over all v1 variants.

/// Convenience lookup helpers on FactSet. These do linear scans — appropriate for
/// v1-scale worlds. If performance becomes an issue, introduce derived index maps.
/// All inputs take &str, not owned String, to avoid accidental clones.
impl FactSet {
    /// All vectors are private. These accessors return read-only slices.
    fn reads(&self) -> &[PropertyRead];
    fn writes(&self) -> &[PropertyWrite];
    fn exits(&self) -> &[ExitEdge];
    fn jumps(&self) -> &[JumpEdge];
    fn choices(&self) -> &[ChoiceFact];
    fn rules(&self) -> &[RuleFact];

    /// Lookup helpers. Linear scans — appropriate for v1-scale worlds.
    fn choice_by_id(&self, id: &str) -> Option<&ChoiceFact>;
    fn exit_by_id(&self, exit_id: &str) -> Option<&ExitEdge>;
    fn exit_by_location_and_name(&self, location: &str, exit_name: &str) -> Option<&ExitEdge>;

    /// Rule lookup.
    fn rule_by_id(&self, id: &str) -> Option<&RuleFact>;

    /// Property-level queries. These decouple consumers from the PropertyDependencyIndex
    /// implementation — internally they may use flat scan or the index.
    fn reads_by_key(&self, key: &PropertyKey) -> impl Iterator<Item = &PropertyRead>;
    fn writes_by_key(&self, key: &PropertyKey) -> impl Iterator<Item = &PropertyWrite>;

    /// Site-level property queries. Returns which sites read/write a given property.
    /// Backbone for explain mode, cache invalidation, and LSP references.
    fn sites_reading(&self, key: &PropertyKey) -> impl Iterator<Item = &FactSite>;
    fn sites_writing(&self, key: &PropertyKey) -> impl Iterator<Item = &FactSite>;

    /// Site adjacency: returns read indices owned by a given site.
    /// Uses ChoiceFact.condition_reads / ExitEdge.guard_reads / RuleFact.condition_reads
    /// when the site resolves to an owning fact. Callers map indices to facts
    /// via self.reads()[idx]. Returns &[usize] to avoid per-call allocation.
    /// If the site does not resolve (extraction bug), returns empty slice.
    fn read_indices_for_site(&self, site: &FactSite) -> &[usize];

    /// Site adjacency: returns write indices owned by a given site.
    /// Uses ChoiceFact.effect_writes / RuleFact.effect_writes when available.
    /// Exits have no writes, so FactSite::Exit always returns empty slice.
    fn write_indices_for_site(&self, site: &FactSite) -> &[usize];

    /// Resolve a FactSite to its owning fact. Total over all v1 FactSite variants.
    /// Returns None only if the referenced ID does not exist in the FactSet
    /// (which would indicate an extraction bug).
    fn resolve_site(&self, site: &FactSite) -> Option<SiteOwner<'_>>;
}

/// The result of resolving a FactSite to its owning construct.
enum SiteOwner<'a> {
    Choice(&'a ChoiceFact),
    Exit(&'a ExitEdge),
    Rule(&'a RuleFact),
}

/// Compose an ExitId from its components.
/// This is the inverse of split_exit_id and the only correct way to build an ExitId.
/// Callers must use this helper — never format strings manually.
/// Debug-asserts that exit_name contains no slashes — if this fires, a grammar
/// change has broken the composition invariant.
fn make_exit_id(location_id: &str, exit_name: &str) -> ExitId {
    debug_assert!(!exit_name.contains('/'), "exit_name must not contain slashes");
    format!("{}/{}", location_id, exit_name)
}

/// Split an ExitId into its (location_id, exit_name) components.
/// ExitId format: "location_id/exit_name". The grammar guarantees
/// no slashes in either component, so splitting on the first slash
/// is unambiguous.
fn split_exit_id(exit_id: &str) -> Option<(&str, &str)>;
```

**Design note: no ConditionFact.** An earlier draft included a separate `ConditionFact` struct with the same fields as `PropertyRead`. This created duplication, divergence risk, and an unclear source of truth. `PropertyRead` is the canonical representation of a condition. Its `FactSite` tells you whether it belongs to a choice, exit, or rule. If convenience queries are needed (e.g., "all conditions on choice X"), filter `FactSet.reads` by `FactSite::Choice(choice_id)` or follow `ChoiceFact.condition_reads` indices. Do not introduce a parallel struct.

**Design note: ChoiceFact.choice_id stability.** Choice IDs are derived from `section_id/slugified_label` per the Architecture Brief's ID derivation rules. They are globally unique within the compilation unit and stable across recompilation as long as the source label and file path are unchanged.

**Stability caveat:** If an author edits a choice's display label, the ChoiceId changes. This is acceptable for v1 — the ID tracks the authored content, not a persistent identity. For future IDE features that need references to survive label edits (e.g., bookmarks, external annotations), an optional author-provided choice key (e.g., `[id:foo]` syntax) could be introduced. That is a post-v1 syntax extension, not part of this brief. For now, consumers should treat ChoiceId as stable relative to label text, not unconditionally stable.

**ID derivation reuse.** The extraction must reuse the existing ID derivation helpers from LINK or the shared `crate::slugify` module — never duplicate slugify or ID composition logic. This includes section IDs: extraction must read `SectionSymbol.compiled_id` from the symbol table, never recompute it from file stem and section name. Recomputing risks subtle differences from IMPORT's file stem normalization. This is the same principle applied in S8 for location slug matching.

### Why these specific facts

Each fact type was chosen because it directly enables a class of analysis:

| Fact | Enables |
|------|---------|
| `PropertyRead` | "Which conditions read this property?" — the foundation for explain mode and incremental invalidation. Also serves as the canonical condition representation (no separate ConditionFact). |
| `PropertyWrite` | "Which effects can change this property?" — the foundation for write-set analysis (richer S4), conflict detection, and cache invalidation. |
| `ExitEdge` | S3 (unreachable location) becomes a trivial graph query. Location graph visualization. |
| `JumpEdge` | S6 (missing fallthrough) becomes a section-graph reachability query. Dialogue flow visualization. |
| `ChoiceFact` | S4 (orphaned choice) becomes a join between ChoiceFact.condition_reads and PropertyWrite. |

**Granularity note: type-level, not entity-level.** Reads and writes are tracked at the `PropertyKey` level (type + property), not per entity instance. If two entities share a type, a write to one and a read on the other will appear as the same `PropertyKey`. This is the correct granularity for static analysis — the compiler does not know which entity instance is involved at compile time.

**Entity dimension forward compatibility.** Entity-level tracking (adding an `entity_id` field to PropertyRead and PropertyWrite) may be needed for precise explain traces in Wyrd and multi-entity worlds. If added, all FactSet APIs must remain backward compatible — existing consumers that query at `PropertyKey` granularity must continue to work. The entity dimension is additive, not a replacement. Do not add it until a concrete consumer requires it.

### The FactSet container

```rust
/// The complete set of facts extracted from a resolved world.
/// Produced by VALIDATE (or a dedicated ANALYZE sub-phase).
/// Immutable after construction. Deterministic for a given linked compilation unit.
///
/// Fields are private. Consumers access data through slice accessors
/// (reads(), writes(), exits(), jumps(), choices()) and lookup helpers.
/// This enforces immutability at the type level, not just by convention.
pub struct FactSet {
    reads: Vec<PropertyRead>,
    writes: Vec<PropertyWrite>,
    exits: Vec<ExitEdge>,
    jumps: Vec<JumpEdge>,
    choices: Vec<ChoiceFact>,
    rules: Vec<RuleFact>,
}
```

Facts are stored in flat `Vec`s. Cross-references use indices (e.g., `ChoiceFact.condition_reads` is a `Vec<usize>` into `FactSet.reads`). This keeps the data cache-friendly and serializable.

**Index safety invariant.** Every `usize` index stored in `ChoiceFact.condition_reads`, `ChoiceFact.effect_writes`, `ExitEdge.guard_reads`, `RuleFact.condition_reads`, and `RuleFact.effect_writes` must point into the corresponding FactSet vector (`reads` or `writes`) and must refer to a fact whose `FactSite` matches the owning construct. Specifically: a `ChoiceFact.condition_reads` entry must point to a `PropertyRead` with `FactSite::Choice` of the same ChoiceId. A `ChoiceFact.effect_writes` entry must point to a `PropertyWrite` with `FactSite::Choice` of the same ChoiceId. A `guard_reads` entry must point to a `PropertyRead` with `FactSite::Exit` matching `owning_exit.exit_id()`. A `RuleFact.condition_reads` entry must point to a `PropertyRead` with `FactSite::Rule` of the same RuleId. A `RuleFact.effect_writes` entry must point to a `PropertyWrite` with `FactSite::Rule` of the same RuleId. Tests should enforce both bounds validity and site consistency.

**Referential integrity.** Every PropertyRead and PropertyWrite must be reachable from exactly one owning site via its FactSite and the corresponding adjacency list. No floating facts (reads/writes not referenced by any owning construct) and no duplicate references (the same read/write index appearing in multiple adjacency lists) are permitted. Every `JumpEdge.from_section` must refer to a section that exists in the symbol table. It does not need to appear elsewhere in the FactSet (a section with no choices and no other jumps is valid). Each PropertyRead and PropertyWrite must have exactly one owning FactSite. No fact may be shared across multiple sites.

**Uniqueness constraints.** ChoiceId, ExitId, and RuleId are each unique within a FactSet. FactSet must not contain duplicate facts with identical semantic identity and span. Double emission during traversal is an extraction bug.

**Span identity contract.** Span equality for determinism and testing is defined by `(file_path, start_offset, end_offset)`. Any additional fields on Span (e.g., internal arena ids) must not affect equality comparisons. This formalizes the SpanKey test helper as a design rule, not just a test convenience.

**Ordering contract.** FactSet vectors preserve deterministic insertion order matching the compiler's traversal: topological file order, then source order within files. Within each file, facts are emitted in pre-order visitation of AST nodes in source order. If the traversal is recursive, this means parent before children, earlier source positions before later ones. Refactors that switch from recursion to an explicit stack must preserve this pre-order guarantee. Consumers may rely on this for deterministic output. Consumers should NOT rely on any semantic ordering (e.g., "all reads for type X are contiguous") — use the PropertyDependencyIndex for grouped access. **Implementation constraint:** all iteration over symbol tables and AST collections during extraction must use `IndexMap` insertion order. Never iterate via `HashMap` or any unordered collection. If a future refactor changes a symbol table field from `IndexMap` to `HashMap`, determinism will silently break. **Per-file symbol ordering:** any per-file symbol collections (locations, sections, rules within a file) must be iterated in source order as recorded by the parser or symbol insertion order, never by span sorting unless the sort key and stability are explicitly specified. This prevents a later refactor from "cleaning up" ordering and accidentally changing traversal order across platforms.

**Indexing strategy.** The FactSet is flat and simple by default. The `PropertyDependencyIndex` (see Part 2) is an optional derived view built on demand. Neither approach is inherently better — flat scans are fast for small worlds, indices pay off for repeated queries in LSP and Wyrd. The implementer should start with flat scans and introduce the index when a consumer needs it.

### Where it lives in the pipeline

The facts layer is produced after LINK and before (or during) VALIDATE. Two options:

**Option A: Sub-phase of VALIDATE.** Add a `facts::extract_facts()` function called at the start of `validate()`. VALIDATE's existing checks can then optionally use the FactSet instead of walking the AST. The FactSet is returned alongside diagnostics.

**Option B: Separate ANALYZE phase between LINK and VALIDATE.** This is a cleaner separation but changes the phase count and pipeline contract. It also means VALIDATE cannot use facts unless it receives them as input.

**Recommendation: Option A.** The facts extraction is a read-only pass over the same data VALIDATE already reads. Adding it as a sub-step keeps the five-phase pipeline intact and lets VALIDATE's checks consume facts immediately. The FactSet becomes part of VALIDATE's output, available to EMIT and downstream consumers.

The pipeline change in `lib.rs` would look like:

```rust
// Phase 4: VALIDATE (now also produces facts)
let (fact_set, _) = validate::validate(&linked.graph, &linked.symbol_table, &mut diagnostics);
```

Or, if the FactSet should be available regardless of validation errors:

```rust
// Phase 4a: Extract facts (always succeeds)
let fact_set = validate::extract_facts(&linked.graph, &linked.symbol_table);

// Phase 4b: VALIDATE (reads facts, produces diagnostics)
validate::validate_with_facts(&linked.graph, &linked.symbol_table, &fact_set, &mut diagnostics);
```

The implementer should choose based on whether VALIDATE checks will actually consume facts in this iteration. If the only consumer is downstream (Wyrd, LSP), Option A with the FactSet as a return value is simplest.

### Extraction algorithm

The extraction does not reparse source files and does not redo semantic resolution — both are already complete after LINK. It traverses the already-parsed ASTs to collect reads, writes, and jumps using the resolved annotations that LINK populated. The traversal pattern is the same as VALIDATE steps 4 and 5 (conditions and effects), plus the symbol table's location and section data. **Extraction visits files in `ordered_file_ids` order and within each file visits AST nodes in source order.** Do not iterate the graph's internal map directly — `ordered_file_ids` is the determinism driver.

**No partial facts rule.** Facts are emitted only for fully resolved constructs. Partially resolved or invalid constructs produce diagnostics, not partial facts. This applies globally to all fact types — exits, jumps, reads, writes, choices, and rules.

1. **Exits.** For each `LocationSymbol`, for each `ExitSymbol` with a resolved destination, emit an `ExitEdge`. Check whether the exit has child condition nodes (via `condition_node` on `ExitSymbol`) to set `is_conditional`. **Unresolved exit references (`resolved_destination` is `None`) produce no ExitEdge fact.** Partial edges with empty destinations are never emitted.

2. **Choices.** For each `SectionSymbol`, iterate its `ChoiceSymbol` children. For each choice, emit a `ChoiceFact`. Sections have no dedicated fact type — they are represented implicitly via SectionId on ChoiceFact and JumpEdge. Track indices for later cross-referencing.

3. **Conditions (as PropertyRead).** Walk AST content nodes (same traversal as VALIDATE step 4). For each resolved `PropertyComparison`, emit a `PropertyRead`. Record the index in the parent choice's `condition_reads` or the parent exit's `guard_reads`. **`LiteralKind` must be read from the parsed AST node's literal type, not reconstructed by inspecting `value_literal` as a string.** The parser already distinguishes token types; extraction preserves that classification.

4. **Effects.** Walk AST content nodes (same traversal as VALIDATE step 5). For each resolved `Set` effect, emit a `PropertyWrite`. Record the index in the parent choice's `effect_writes`. **Entity lifecycle effects (`Move`, `Destroy`, `Reveal`, `Spawn`) are not recorded in the v1 FactSet.** They do not target a (type, property) pair and would require dedicated fact types. These will be added in a follow-up brief if needed. Do not invent partial representations.

5. **Jumps.** Walk AST content nodes for `Jump` nodes with resolved annotations. Emit a `JumpEdge` with a typed `JumpTarget` for each. The annotation on a resolved Jump must contain one of: a resolved `SectionId`, a resolved `ExitId` (or `ExitSymbol` reference), or a resolved `End` marker. For `JumpTarget::Exit`: extraction must use the same resolved annotation object that VALIDATE uses. If the annotation resolved to an `ExitSymbol` directly, use that symbol's owning location and exit name to build ExitId via `make_exit_id()`. If the annotation explicitly indicates it resolved to an exit name only (no owning location), derive the location from the section's containing location. If neither is available (e.g., a global section with no location container, or annotation data is absent), do not emit a `JumpEdge` for this jump — rely on existing VALIDATE diagnostics to flag the unresolvable reference. Similarly, if the jump resolves to an exit symbol whose destination is itself unresolved (no ExitEdge exists for it), do not emit a JumpEdge. Never infer exit resolution independently from VALIDATE.

6. **Rules.** For each `RuleSymbol`, emit a `RuleFact`. Walk its conditions and effects the same way as choices, using `FactSite::Rule(rule_id)` as the site. Record indices in `RuleFact.condition_reads` and `RuleFact.effect_writes`. Rule emission order follows symbol table insertion order: topological file order, then declaration order within each file.

### What is NOT in the facts layer

- **Derived facts.** No inference, no transitive closure, no "reachable from start." Those are queries over facts, not facts themselves.
- **Type definitions.** The symbol table already serves this purpose. No duplication.
- **Entity property defaults.** These are in the symbol table. Facts track reads and writes, not declarations.
- **Narrative content.** Prose, speech, stage directions are not facts. They don't participate in analysis.
- **Graph abstractions.** The FactSet implicitly contains a location graph (ExitEdges) and a section graph (JumpEdges), but does not expose them as graph types. When consumers need graph operations (reachability, cycle detection, topological ordering), they should build derived views like `LocationGraph::from_facts(&FactSet)` or `SectionGraph::from_facts(&FactSet)`. These belong in a follow-up "First Derived Analyses" brief, not in the facts layer itself. **Derived graph structures must be pure functions of FactSet and must not access AST or SymbolTable.** This ensures analysis never reintroduces dual data sources.


## Part 2: Read/Write Provenance

### What it is

Provenance extends the facts layer with the "why" information needed to answer dependency questions. Where the facts layer says "choice X reads property mood on type Guard," provenance says "choice X reads property mood on type Guard because of the condition at line 47 of tavern.urd.md, and that property can be written by the effect at line 23 and the rule effect at line 89."

In v1, provenance covers property comparisons and property mutation only. Entity lifecycle effects (`Move`, `Destroy`, `Reveal`, `Spawn`) are out of scope for the FactSet and will be handled by separate fact relations if needed later.

### What provenance enables

| Question | How it's answered |
|----------|-------------------|
| "Why is this choice available?" | Follow the choice's `condition_reads` to PropertyRead facts. Each read names the (type, property, operator, value) it requires. |
| "Why is this location unreachable?" | Start from the ExitEdge facts. No ExitEdge has this location as `to_location` where the source is reachable from start. (This is S3 as a fact query.) |
| "Which conditions read this property?" | Filter PropertyRead facts by `PropertyKey`. Return the list of sites. |
| "Which effects can change this property?" | Filter PropertyWrite facts by `PropertyKey`. Return the list of sites. |
| "If I change this property, what is affected?" | Union of: all PropertyRead facts for the property (conditions that may change evaluation) + all downstream choices/exits those conditions guard. |
| "What is the write set for this property?" | All PropertyWrite facts for `(type, property)`. The `value_expr` field shows the possible values. For enums, this is the set of values any effect can produce. |

### Killer example: property read but never written

The simplest derived diagnostic that proves the value of the IR:

> **"Property is read but never written anywhere."**

Implementation as a fact query:

```
For each unique PropertyKey in FactSet.reads:
    If no PropertyWrite exists in FactSet.writes for the same PropertyKey:
        Emit warning: "Condition at {span} reads '{entity_type}.{property}' but no effect
        in this world ever writes this property. The condition can only reflect the
        property's default value or initial entity override."
```

This is approximately 10 lines of code over the FactSet. Without the FactSet, it would require a full AST traversal collecting both conditions and effects, then cross-referencing — essentially reimplementing most of the extraction. This single diagnostic justifies the entire facts layer.

**Skip rule:** v1 has no `readonly` or `const` property concept. The warning is always emitted when a property is read but never written. Authors can ignore it for intentionally immutable properties (e.g., a `name` property that is set in the entity declaration and never changed). If a readonly annotation is added to the type system later, this diagnostic should skip properties marked readonly.

### Provenance is already partially captured

LINK's `Annotation` model records resolved references: `resolved_entity`, `resolved_type`, `resolved_property`. The facts layer makes these queryable. True provenance adds one thing LINK doesn't track: **cross-referencing reads to writes.**

The key new data structure is a property dependency index:

```rust
/// Index mapping (type, property) pairs to their read and write sites.
/// Built from the FactSet as a derived secondary index.
struct PropertyDependencyIndex {
    /// Key: property key. Value: indices into FactSet.reads.
    readers: IndexMap<PropertyKey, Vec<usize>>,
    /// Key: property key. Value: indices into FactSet.writes.
    writers: IndexMap<PropertyKey, Vec<usize>>,
}

impl PropertyDependencyIndex {
    /// Convenience: all reads for a given (type, property) pair.
    /// Returns an empty slice if the pair has no readers.
    fn reads_of(&self, key: &PropertyKey) -> &[usize];

    /// Convenience: all writes for a given property key.
    /// Returns an empty slice if the key has no writers.
    fn writes_of(&self, key: &PropertyKey) -> &[usize];

    /// All property keys that are read anywhere.
    fn read_properties(&self) -> impl Iterator<Item = &PropertyKey>;

    /// All property keys that are written anywhere.
    fn written_properties(&self) -> impl Iterator<Item = &PropertyKey>;
}
```

This is a derived view, not stored data. It's built from the FactSet on demand. For v1-scale worlds, building it is a single pass over the reads and writes vectors. Index key insertion order preserves first-seen order based on FactSet traversal and is deterministic for a given input. Do not sort keys. The index must be built by iterating FactSet vectors in their defined order. No additional sorting or reordering is permitted. The index keys are `PropertyKey` instances cloned from FactSet entries (`entity_type` and `property` fields). No new string normalization occurs during index construction.

### Provenance for Wyrd (future)

When Wyrd exists, it can consume the FactSet and PropertyDependencyIndex for:

- **Explain mode.** "Why is this choice available?" → look up the choice's condition_reads, evaluate each, show which passed and which failed with their source spans.
- **Cache invalidation.** When an effect writes `(Guard, mood)`, look up all readers of `(Guard, mood)`. Those choices and exits need re-evaluation. Everything else is unchanged.
- **Incremental recompilation.** When a source file changes, look up which facts came from that file (via spans). Invalidate only the facts from that file and anything that depends on them.

### Provenance for LSP (future)

- **Go to definition** for property reads → jump to the property declaration on the type.
- **Find all references** for a property → return all PropertyRead and PropertyWrite sites.
- **Rename refactoring** → find all facts referencing the old name, compute the set of spans to update.

### What provenance does NOT include

- **Runtime state traces.** Provenance is static — it records what the source code declares, not what happens during execution. "Why did the guard become hostile at turn 7?" is a runtime question, not a compiler question.
- **Transitive dependency chains.** "If I change property A, and that changes property B via a rule, and B gates a choice..." — this requires chaining reads through writes through rules. The facts and provenance give you the building blocks, but the transitive closure is a query, not stored data.


## Integration with Existing Checks

Once the facts layer exists, the S3/S4/S6/S8 checks can optionally be refactored to use it. This is NOT required — the existing implementations work and are tested. But the refactored versions demonstrate the value of the IR:

| Check | Current implementation | With facts layer |
|-------|----------------------|------------------|
| S3 (unreachable location) | Builds a location graph from symbol table, BFS from start. | Filter `FactSet.exits` for `ExitEdge` facts. Build graph. BFS. Same result, no symbol table traversal. |
| S4 (orphaned choice) | Walks AST content, finds choices, checks enum conditions. | Filter `FactSet.reads` by `FactSite::Choice` where value_literal is not in the enum's values. Join with `ChoiceFact`. |
| S6 (missing fallthrough) | Walks AST content, segments by section, checks sticky/terminal/prose. | `ChoiceFact` gives sticky flags. `JumpEdge` gives terminal jumps. Section metadata gives fallthrough status. Most data is pre-extracted. |
| S8 (shadowed exit) | Walks AST content, tracks location, compares section names to exit names. | Join section IDs from `ChoiceFact` with `ExitEdge.exit_name` where locations match. |

The refactored versions are shorter, more declarative, and easier to test — but the originals are correct and tested. Refactoring is an optional follow-up, not part of this brief's scope.


## Data Structures Summary

```
Linked compilation unit (from LINK)
    ├── DependencyGraph (file imports, ASTs)
    ├── SymbolTable (types, entities, locations, sections, actions, rules, sequences)
    │
    └── [NEW] FactSet (produced by VALIDATE or sub-phase)
            ├── reads: Vec<PropertyRead>      // canonical condition representation
            ├── writes: Vec<PropertyWrite>
            ├── exits: Vec<ExitEdge>
            ├── jumps: Vec<JumpEdge>
            ├── choices: Vec<ChoiceFact>
            ├── rules: Vec<RuleFact>
            │
            └── [DERIVED ON DEMAND] PropertyDependencyIndex
                    ├── readers: IndexMap<PropertyKey, Vec<read_index>>
                    └── writers: IndexMap<PropertyKey, Vec<write_index>>
```


## Implementation Notes

### Scope for this brief

This brief covers:

1. Define the identity type aliases and fact types.
2. Define the FactSet container (immutable, deterministic).
3. Implement `extract_facts()` as a function in a new `facts` module under `src/`.
4. Wire it into the pipeline so the FactSet is available after VALIDATE.
5. Write tests that verify fact extraction against the five canonical fixtures.
6. Implement the PropertyDependencyIndex builder.
7. Write tests that verify index correctness (reads and writes for known properties match expectations).

This brief does NOT cover:

- Refactoring S3/S4/S6/S8 to use facts (optional follow-up).
- Derived diagnostics using facts (follow-up brief: "First Derived Analyses Using FactSet").
- Wyrd integration (blocked on Wyrd).
- LSP integration (blocked on LSP).
- JSON serialization of the FactSet (useful for tooling, but not required yet).
- Soufflé or Datalog integration (future decision).

**CLI integration note.** When the CLI exists, add two dump modes: `--dump-facts` outputs the FactSet in traversal order (matching the ordering contract — topological file order, then source order within files), and `--dump-facts-canonical` outputs a canonical sorted form suitable for diffing across builds. The traversal-order dump validates the determinism contract directly. The canonical dump provides a stable regression diff tool even if traversal order changes intentionally. Canonical serialization is a separate function with its own tests — it must not affect the in-memory ordering contract.

### Module placement

Create `src/facts.rs` (or `src/facts/mod.rs` if it grows). The module is public — it's part of the compiler's API surface for downstream consumers.

```rust
// In lib.rs
pub mod facts;
```

The extraction function signature:

```rust
/// Extract normalized analysis facts from the resolved world.
/// Called after LINK, before or during VALIDATE.
/// Read-only — does not modify the graph or symbol table.
/// Deterministic — same input always produces same output.
/// FilePath is re-exported from crate::span, not redefined.
/// extract_facts() must not fail, emit diagnostics, or return partial output.
/// Extraction is total for any successful LINK result — it either produces a
/// complete FactSet or panics in debug builds if LINK invariants are violated.
/// It is a projection of already-resolved structure, not validation.
pub fn extract_facts(
    graph: &DependencyGraph,
    ordered_file_ids: &[crate::span::FilePath],
    symbol_table: &SymbolTable,
) -> FactSet

// Internally, extract_facts() should use a private FactSetBuilder that
// encapsulates index management. The builder exposes methods like:
//
//   builder.push_read(read: PropertyRead, owner: &FactSite) -> usize
//   builder.push_write(write: PropertyWrite, owner: &FactSite) -> usize
//   builder.push_exit(edge: ExitEdge)
//   builder.push_choice(choice: ChoiceFact)
//   builder.push_rule(rule: RuleFact)
//   builder.finish() -> FactSet
//
// push_read/push_write automatically append the returned index to the
// owning construct's adjacency list (condition_reads, effect_writes,
// guard_reads), enforcing the referential integrity invariant at
// construction time rather than relying solely on post-hoc tests.
// FactSetBuilder is private to the facts module — no external code
// can construct a FactSet except through extract_facts().
```

### Test strategy

| Test | What it verifies |
|------|------------------|
| Two-room key puzzle: exit count | `FactSet.exits` has exactly 1 ExitEdge (Cell → Corridor). |
| Two-room key puzzle: conditional exit | The ExitEdge has `is_conditional == true` and `guard_reads` references a PropertyRead for `@cell_door.locked`. |
| Two-room key puzzle: effect writes | `FactSet.writes` includes a PropertyWrite for `locked = false` on the Door type. |
| Tavern scene: choice count | `FactSet.choices` has the expected count of sticky and one-shot choices. |
| Tavern scene: property reads | PropertyRead facts exist for `@arina.trust` comparisons with `FactSite::Choice`. |
| Monty Hall: no exits | `FactSet.exits` is empty (single-location game). |
| Interrogation: cross-file facts | Facts span multiple files, with correct source spans pointing to the originating file. |
| Sunken Citadel: rule facts | `FactSet.rules` contains 3 RuleFacts. Each RuleFact has condition_reads referencing PropertyRead facts with `FactSite::Rule`. |
| Sunken Citadel: scale smoke test | `FactSet` has the expected order-of-magnitude counts: ~23 ExitEdges, ~64 ChoiceFacts, ~3 RuleFacts. Verifies extraction handles the largest fixture without panics or missing facts. |
| Sunken Citadel: exit graph density | Multiple conditional and unconditional exits across 12 locations. Verifies `is_conditional` and `guard_reads` are populated correctly at scale. |
| PropertyDependencyIndex: readers | For `(Guard, trust)`, the readers index returns all condition sites that compare trust. |
| PropertyDependencyIndex: writers | For `(Door, locked)`, the writers index returns the effect that sets it to false. |
| Round-trip consistency | Every index in `ChoiceFact.condition_reads` points to a valid `FactSet.reads` entry. |
| Determinism | Two calls to `extract_facts()` on the same linked compilation unit produce structurally identical FactSets. Define a `SpanKey(file, start, end)` test helper and compare via SpanKey, ignoring any internal Span ids. This keeps determinism tests stable even if Span grows fields later. Do not rely on pointer identity or `Debug` formatting. |
| FactSite Choice resolution | Given a `FactSite::Choice(choice_id)`, `fact_set.choice_by_id(choice_id)` returns a matching `ChoiceFact`. |
| FactSite Exit resolution (by id) | Given a `FactSite::Exit(exit_id)`, `fact_set.exit_by_id(exit_id)` returns a matching `ExitEdge`. |
| FactSite Exit resolution (by parts) | Given a known location and exit name, `fact_set.exit_by_location_and_name(loc, name)` returns the same `ExitEdge` as `exit_by_id(make_exit_id(loc, name))`. |
| FactSite Rule resolution | Given a known rule id, `fact_set.resolve_site(&FactSite::Rule(rule_id))` returns `SiteOwner::Rule`. |
| JumpTarget Exit resolution | For any emitted `JumpEdge` with `JumpTarget::Exit(exit_id)`, `fact_set.exit_by_id(exit_id)` must return `Some`. If an exit has no `ExitEdge` (because its destination is unresolved), then no `JumpEdge` should target it. Every emitted JumpEdge must reference a target that exists in FactSet — no dangling targets are allowed. |

### Estimated size

- Identity type aliases + fact type definitions: ~130 lines
- FactSet lookup helpers: ~20 lines
- Extraction function: ~200 lines (same traversal as VALIDATE steps 4–5, but collecting instead of checking)
- PropertyDependencyIndex + query helpers: ~60 lines
- Tests: ~450 lines (five fixtures including Sunken Citadel scale tests)

Total: ~860 lines of new code.


## Acceptance Criteria

1. Identity type aliases defined and used consistently across all fact types.
2. `FactSet` type is defined, public, and immutable after construction.
3. `extract_facts()` runs on all five canonical fixtures without panics.
4. Fact counts match expectations for each fixture: exact counts for the small fixtures (two-room, tavern, monty-hall, interrogation) and bounded range assertions for Sunken Citadel.
5. `PropertyDependencyIndex` correctly maps `PropertyKey` instances to their read and write sites.
6. No existing tests regress.
7. `CompilationResult` has an `Option<FactSet>` that is `Some` whenever LINK succeeds, even if VALIDATE emits errors. Consumers should not depend on a success-only build to access facts. Facts are constructed after LINK succeeds and before any validation checks that might early-return — extraction must not be gated behind "no errors so far."
8. The extraction does not modify the graph, symbol table, or AST (read-only contract preserved).
9. Two calls to `extract_facts()` on the same input produce structurally identical output (determinism verified in tests).
10. `FactSite::Exit` references can be resolved to `ExitEdge` entries (tested alongside Choice resolution).
11. Every `JumpEdge` target resolves to an existing `SectionId` or `ExitId` in the FactSet. No dangling targets are permitted.


## Relationship to Future Work

| Future capability | How the analysis IR helps |
|-------------------|--------------------------|
| "Read but never written" diagnostic | ~10 lines: cross-reference PropertyRead `PropertyKey` values against PropertyWrite. First proof of FactSet value. |
| Richer S4 (write-set orphan detection) | Join PropertyWrite facts by `PropertyKey` against ChoiceFact.condition_reads. If no write can produce the required value, the choice is dead. |
| Conflict detection | Two effects writing the same `PropertyKey` in the same action. Trivial query over PropertyWrite. |
| Explain mode (Wyrd) | Wyrd receives the FactSet. When asked "why is this choice available?", it walks condition_reads, evaluates each against current state, and returns the explain trace with source spans. |
| Cache invalidation (Wyrd) | On state change to a `PropertyKey`, look up readers in the PropertyDependencyIndex. Only re-evaluate those conditions. |
| Incremental recompilation (LSP) | On file change, invalidate facts whose span is in the changed file. Rerun extraction for that file only. Downstream consumers re-query. |
| Soufflé integration | The FactSet becomes the input relation set. Soufflé rules query over facts. The facts layer is the bridge, regardless of whether Soufflé is adopted. |
| Visualization tooling | ExitEdge and JumpEdge facts directly produce Mermaid or Graphviz diagrams without AST traversal. |

*End of Brief*