---
title: The Docking Procedure
slug: compiler-briefs
description: Six engineering briefs that specify the compiler — every data structure, every diagnostic code, every phase contract — before a single line of implementation is written.
date: "2026-02-17"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Documenting the compiler specification bundle and why it exists.
> Single canonical copy. February 2026.

## Harder than docking

Spacecraft docking is an alignment problem. Two objects, known shapes, known interfaces, known physics. Line up the ports, match velocity, engage the latches. The hard part is precision under constraint — there is no ambiguity about what a successful dock looks like.

Building a compiler from a specification is harder. The specification describes behaviour in prose. The compiler must implement that behaviour in code. Between the two sits a gap where every ambiguity in the prose becomes a bug in the implementation, and every unstated assumption becomes a design decision made under pressure, in the wrong place, by whoever happens to be writing that function.

The formalisation phase closed that gap at the boundaries. The [PEG grammar](/articles/peg-grammar) nailed down what valid input looks like — every sigil, every indent rule, every ambiguity resolved by ordered choice. The [JSON Schema](/articles/json-schema) nailed down what valid output looks like — every block, every field, every constraint machine-checkable. But between those two boundaries is the compiler itself: five phases, hundreds of rules, thousands of interactions.

Docking has two contact surfaces. This has two contact surfaces and a five-stage transformation pipeline between them. The briefs are the alignment guides.

## Why specify before implementing

The compiler is not one thing. It is five things that must agree:

```
  .urd.md ─── PARSE ─── IMPORT ─── LINK ─── VALIDATE ─── EMIT ─── .urd.json
                │           │         │          │           │
             FileAST    DepGraph   SymTable   Diagnostics   JSON
```

Each phase has an input contract (what it receives), an output contract (what it produces), and a set of guarantees (what the next phase can rely on without re-checking). If any phase misunderstands what the previous phase promised, the pipeline breaks in ways that are difficult to diagnose — the symptom appears three phases downstream from the cause.

The architecture brief defines everything that crosses phase boundaries: the AST node types, the symbol table structure, the diagnostic collector, the dependency graph, the ID derivation rules. The five phase briefs each define what happens inside one phase — and crucially, what does *not* happen, because that is another phase's job.

This is the docking procedure: specify every interface, every data structure, every handoff, before any implementation begins. The alternative — discover the interfaces during implementation — is how compilers accumulate the kind of subtle bugs that are expensive to find and dangerous to ship. This code will run in a hospital.

## The bundle

Six documents, published together. One architecture brief and five phase briefs, written in dependency order.

### Compiler Architecture

The shared foundation. Defines every data structure that flows between phases and every contract that makes the phases independently implementable.

[Compiler Architecture Brief](/documents/urd-compiler-architecture-brief)

**The AST.** Twenty-eight node types covering every syntactic construct in the PEG grammar. File-scoped (never merged across files), annotatable (LINK fills in resolved references without rebuilding the tree), and span-tracked (every node records its exact source position for diagnostics). Three structured condition expression variants — `PropertyComparison`, `ContainmentCheck`, `ExhaustionCheck` — parsed by PARSE, consumed by every subsequent phase.

**The symbol table.** Seven ordered maps — types, entities, sections, locations, actions, rules, sequences — each preserving insertion order for deterministic output. The visible scope model enforces non-transitive imports: if A imports B and B imports C, A does not see C. The symbol table stores everything; each file sees only what it has imported.

**The diagnostic collector.** Five non-overlapping code ranges (URD100–URD599), one per phase. Errors prevent JSON emission. Warnings do not. The compiler never stops at the first error — it marks damaged constructs and continues, reporting as many problems as possible in a single run.

**The dependency graph.** Acyclic, depth-limited (64 levels), non-transitive. File paths normalised to forward slashes, relative to the entry file. Case-sensitive comparison even on case-insensitive filesystems.

**The two-pass model.** Forward references must work — an action can reference an entity declared later in the same file. The five phases map onto two logical passes: collection (PARSE + IMPORT + LINK declaration sweep) and validation (LINK resolution sweep + VALIDATE + EMIT). LINK straddles both, which is the natural consequence of supporting forward references.

**ID derivation.** Entity IDs are used as declared. Section IDs are `file_stem/section_name`. Choice IDs are `section_id/slugify(label)`. Location IDs are `slugify(display_name)`. The rules are defined once and referenced by three phases.

### Phase 1: PARSE

Source text to per-file ASTs. One job: syntactic validity.

[PARSE Phase Brief](/documents/urd-compiler-parse-brief)

Two-region parsing: an optional frontmatter block (restricted YAML-like syntax between `---` delimiters) and narrative content (PEG grammar block dispatch). The frontmatter sub-parser handles types, entities, imports, and the world block. The narrative parser implements the grammar's ordered choice — thirteen alternatives tried in sequence, with prose as the universal fallback.

Condition expressions are parsed into structured typed variants, not stored as raw strings. Downstream phases operate on fields — they never re-parse condition text.

Error recovery uses synchronisation-point advancement: capture the failed line as an `ErrorNode`, emit a diagnostic, skip to the next recognisable boundary. The parser never aborts — it produces as many valid nodes as it can alongside the damage markers.

### Phase 2: IMPORT

Entry file AST to dependency graph plus ordered file list.

[IMPORT Phase Brief](/documents/urd-compiler-import-brief)

IMPORT is the only phase that touches the filesystem. It follows `ImportDecl` nodes recursively, loading and parsing each discovered file. Cycle detection uses the traversal stack (O(depth) per check, acceptable given the 64-level limit). Casing mismatch detection catches cross-platform portability issues at authoring time.

Failed imports leave no trace in the graph — no broken edges, no stub nodes. The diagnostic is the sole record. LINK discovers unresolved references downstream without needing to know why an import failed.

The output is a topologically sorted file list with deterministic tiebreaking (alphabetical by normalised path). The entry file is always last.

### Phase 3: LINK

Dependency graph plus ASTs to symbol table plus annotated ASTs.

[LINK Phase Brief](/documents/urd-compiler-link-brief)

The most complex phase. Two sequential passes over all files in topological order.

**Pass 1 (collection):** Walk every AST and register every declaration — types, entities, locations, sections, choices (which also generate corresponding action symbols), exits, sequences, rules. Detect duplicates. Derive compiled IDs.

**Pass 2 (resolution):** Walk every AST again. Resolve entity references to symbols. Resolve type names on entities. Resolve property accesses through the resolved type. Resolve jump targets using the normative priority rule (section first, exit second, error third). Resolve exit destinations by slugifying the destination text. Enforce visible scope on every lookup.

The no-cascading principle: if a reference is unresolved, its annotation stays `null` and VALIDATE skips everything that depends on it. One root cause, one diagnostic.

### Phase 4: VALIDATE

Annotated ASTs plus symbol table to diagnostics.

[VALIDATE Phase Brief](/documents/urd-compiler-validate-brief)

VALIDATE is read-only — it checks everything, modifies nothing. Property type checking across seven types (boolean, integer, number, string, enum, ref, list). Range and enum constraint enforcement. Trait validation: `portable` for move targets, `container` for destinations, `mobile` and `container` for the player entity. Structural constraints: action mutual exclusion, nesting depth limits, world configuration references, sequence phase action and rule references.

The skip rule is the phase's most important design decision. If an annotation is `null`, VALIDATE silently skips every check that depends on it. LINK already reported the root cause. VALIDATE's job is maximum diagnostic density without false positives from cascading failures.

### Phase 5: EMIT

Validated ASTs plus symbol table to `.urd.json`.

[EMIT Phase Brief](/documents/urd-compiler-emit-brief)

EMIT runs only when the diagnostic collector contains zero errors. It traverses pre-validated data structures in a fixed, deterministic order.

Eight assembly steps produce the JSON object: `world` (always first, `urd: "1"` injected), `types`, `entities`, `locations`, `rules`, `actions`, `sequences`, `dialogue`. Each step reads the symbol table in insertion order and AST nodes through annotations. Empty blocks are omitted.

Condition lowering transforms structured AST nodes into JSON expression strings. Effect lowering transforms AST effect nodes into structured JSON objects. Keyword expansion replaces `here` with `player.container` and `player` with `player`, driven by LINK annotations — EMIT never string-matches on keywords.

The output is byte-identical across repeated compilations. Same source, same compiler version, same JSON. Fixed key order, declaration-order iteration, deterministic tiebreaking, no timestamps, no random IDs.

## The diagnostic map

Every phase owns a code range. No collisions, no ambiguity about which phase produced an error.

| Range | Phase | Examples |
|-------|-------|---------|
| URD100–URD199 | PARSE | Unclosed frontmatter, tab characters, unrecognised syntax |
| URD200–URD299 | IMPORT | File not found, circular import, file stem collision, casing mismatch |
| URD300–URD399 | LINK | Unresolved reference, duplicate entity, unknown type, jump shadowing |
| URD400–URD499 | VALIDATE | Type mismatch, enum violation, missing trait, range violation |
| URD500–URD599 | EMIT | Reserved for future use (v1 has zero EMIT diagnostics) |

PARSE has 10 codes. IMPORT has 14. LINK has 15. VALIDATE has 20. Every code has a message template, a trigger condition, and a recovery action documented in its phase brief.

## How the pieces connect

```
  PEG grammar                      JSON Schema
  (75 rules)                       (548 lines, 9 sub-schemas)
       │                                 │
       │ defines syntax for              │ validates output of
       ▼                                 ▼
  ┌─────────────────────────────────────────────┐
  │               COMPILER                       │
  │                                              │
  │  PARSE ──→ IMPORT ──→ LINK ──→ VALIDATE ──→ EMIT │
  │  FileAST   DepGraph   SymTable  Diagnostics  JSON │
  │                                              │
  │  Architecture Brief: shared data structures  │
  │  Phase Briefs: internal contracts per phase  │
  └─────────────────────────────────────────────┘
```

The PEG grammar crate becomes PARSE's foundation — the grammar rules map directly to AST node types defined in the architecture brief. The JSON Schema is EMIT's structural contract — every JSON element the phase produces must conform. The six briefs specify everything between.

## The test surface

The briefs define 150+ acceptance test cases across all phases:

- **PARSE:** 35 tests — every grammar rule exercised, four canonical integration files parsed with zero errors, five error recovery scenarios, span accuracy verification
- **IMPORT:** 24 tests — path resolution, graph construction, topological sort determinism, cycle detection, error recovery
- **LINK:** 30 tests — collection, choice-to-action compilation, resolution, ID derivation, integration, error recovery with cascading suppression
- **VALIDATE:** 40 tests — property type checking, condition validation, effect validation, structural constraints, skip rule verification, four integration tests
- **EMIT:** 46 tests — world block, type block, entity block, location block, condition lowering, effect lowering, sequence advance modes, dialogue assembly, determinism, four integration tests

Every test has specified input, expected output, and the diagnostic (if any) it should produce. An implementation that passes all tests conforms to the specification.

## What this enables

With the briefs complete, the compiler can be built phase by phase. Each phase has:

- A defined input it can trust without re-checking
- A defined output it must produce
- A defined set of guarantees about that output
- A defined set of test cases that prove it works

Phase 1 (PARSE) can be implemented without reading the VALIDATE brief. Phase 5 (EMIT) can be implemented without understanding cycle detection. The briefs are the alignment guides that let five phases dock into one pipeline.

The formalisation artifacts — PEG grammar and JSON Schema — validate the compiler's boundaries. The six briefs specify its interior. Together, they are the complete engineering specification for the most critical component in the Urd ecosystem.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
