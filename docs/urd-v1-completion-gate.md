# Urd — v1 Completion Gate: Compiler

*Everything that must be true before the Urd compiler can be called v1 complete*

> **Document status: CLOSED — 2026-02-22**
> The compiler gate is closed. All acceptance criteria are met: C1–C9 implemented and tested, S1–S8 static analysis complete, F1–F8 FactSet analysis IR verified, E1–E7 specification consistency audit passed, JSON Schema validates all compiler outputs, negative corpus rejected with correct codes. 554 tests, 100% pass rate, compiler 0.1.7.
>
> Runtime, testing framework, and system-level acceptance criteria are in the separate System Gate document.

---

## What "Compiler v1 Complete" Means

Compiler v1 complete is not "the compiler runs." The compiler already runs — five phases, 554 tests, 100% pass rate, five canonical fixtures compiling to valid `.urd.json`.

Compiler v1 complete means:

1. The compiler implements every specified primitive (C1–C9)
2. All eight static analysis checks are implemented and tested (S1–S8)
3. The FactSet analysis IR is implemented and verified (F1–F8)
4. The JSON Schema validates every compiler output and rejects invalid hand-authored JSON
5. The specification suite is internally consistent (no contradictions between docs)
6. All canonical fixtures compile without warnings
7. The negative test corpus is rejected with correct error locations

This is the point at which the compiler's foundational claims are provable, not just specified. The runtime and system-level claims (deterministic replay, multi-interface portability) are gated separately in the System Gate.

---

## Current State

| Capability | Status |
|-----------|--------|
| PEG grammar (pest, Rust) | ✓ Implemented |
| Five-phase pipeline (PARSE → IMPORT → LINK → VALIDATE → EMIT) | ✓ Implemented |
| Frontmatter parsing (constrained YAML subset) | ✓ Implemented |
| Entity references (`@entity.property`) | ✓ Implemented |
| Type validation | ✓ Implemented |
| Import resolution | ✓ Implemented |
| Circular import detection | ✓ Implemented |
| Duplicate entity/type ID detection | ✓ Implemented |
| `.urd.json` emission conforming to schema | ✓ Implemented |
| Dialogue sections, jumps, sticky/one-shot choices | ✓ Implemented |
| `any:` OR conditions | ✓ Implemented |
| Five canonical fixtures (Tavern, Monty Hall, Key Puzzle, Interrogation, Sunken Citadel) | ✓ Compiling |
| Static analysis: all eight checks (S1–S8) | ✓ Implemented (compiler 0.1.5) |
| FactSet analysis IR (F1–F8): six fact types, PropertyDependencyIndex, WASM serialisation | ✓ Implemented (compiler 0.1.6) |
| 554 tests, 100% pass rate | ✓ Current |

---

## Compiler Requirements (C1–C9)

From Architecture §v1 Acceptance Checklist.

| # | Requirement | Source | Status |
|---|------------|--------|--------|
| C1 | Parse Urd frontmatter without a general-purpose YAML parser. Reject anchors, aliases, merge keys, custom tags, block-style lists, and implicit type coercion. | Schema Specification §Frontmatter Grammar | ✓ Implemented |
| C2 | Resolve `import:` declarations. Imports are explicit and non-transitive. | Schema Markdown Specification §Import Resolution Rules | ✓ Implemented |
| C3 | Detect circular imports of any cycle length. Report full cycle path. | Schema Markdown Specification §Import Resolution Rules | ✓ Implemented (URD202) |
| C4 | Reject duplicate entity IDs across the full compilation unit with diagnostics showing all declaration sites. | Schema Markdown Specification §Import Resolution Rules | ✓ Implemented (URD302+) |
| C5 | Reject duplicate type names across the compilation unit. | Schema Markdown Specification §Import Resolution Rules | ✓ Implemented (URD302+) |
| C6 | Validate that every `@entity` reference, `->` jump, and property access resolves to a declared target. | Architecture §Compiler Responsibilities | ✓ Implemented (URD301) |
| C7 | Validate property values against declared types, enum sets, and ref constraints. | Schema Specification §Property Schema | ✓ Implemented (URD410+) |
| C8 | Emit `.urd.json` conforming to the Schema Specification. Set `urd: "1"` automatically. Warn and override if author sets it. | Schema Specification §world block | ✓ Implemented and tested (URD411, compiler 0.1.7) |
| C9 | Enforce nesting depth: warn at 3 levels, error at 4+. | Schema Markdown Specification §Nesting Rules | ✓ Implemented and tested (URD410, compiler 0.1.7) |

---

## Static Analysis (S1–S8)

All eight checks are implemented as of compiler 0.1.5.

| # | Check | Source | Status |
|---|-------|--------|--------|
| S1 | Undefined entity reference | Test Case Strategy §Static Analysis | ✓ Implemented (LINK phase, URD301) |
| S2 | Type mismatch (property set to invalid value) | Test Case Strategy §Static Analysis | ✓ Implemented (VALIDATE phase, URD410+) |
| S3 | Unreachable location (not reachable from `world.start` via exits) | Test Case Strategy §Static Analysis | ✓ Implemented (VALIDATE phase, URD430) |
| S4 | Orphaned action (choice-scoped in v1: a choice whose conditions can never be satisfied) | Test Case Strategy §Static Analysis | ✓ Implemented (VALIDATE phase, URD432) |
| S5 | Duplicate IDs across compilation unit | Test Case Strategy §Static Analysis | ✓ Implemented (LINK phase, URD302+) |
| S6 | Missing fallthrough (one-shot-only section, no terminal jump) | Test Case Strategy §Static Analysis | ✓ Implemented (VALIDATE phase, URD433) |
| S7 | Circular imports | Test Case Strategy §Static Analysis | ✓ Implemented (IMPORT phase, URD202) |
| S8 | Shadowed exit (section name matches exit name in same location) | Test Case Strategy §Static Analysis | ✓ Implemented (VALIDATE phase, URD434) |

---

## Analysis IR: FactSet (F1–F8)

The FactSet is a normalized, immutable, deterministic intermediate representation extracted after LINK. It projects the resolved world into flat relational tuples — exits, jumps, choices, rules, property reads, and property writes — queryable without AST traversal. Implemented in compiler 0.1.6 with 31 dedicated tests across all five canonical fixtures. WASM output includes serialised facts for playground tooling.

| # | Requirement | Source | Status |
|---|------------|--------|--------|
| F1 | FactSet container type — immutable, deterministic, complete | Analysis IR Brief §Core Invariant | ✓ Implemented |
| F2 | Fact types: ExitEdge, JumpEdge, ChoiceFact, RuleFact, PropertyRead, PropertyWrite | Analysis IR Brief §Fact Types | ✓ Implemented |
| F3 | PropertyKey normalization | Analysis IR Brief §Identity Types | ✓ Implemented |
| F4 | FactSite uniform address space | Analysis IR Brief §FactSite | ✓ Implemented |
| F5 | `extract_facts()` produces facts for all five canonical fixtures | Analysis IR Brief §Extraction Algorithm | ✓ Implemented |
| F6 | PropertyDependencyIndex with query helpers | Analysis IR Brief §Part 2 | ✓ Implemented |
| F7 | FactSetBuilder enforces referential integrity at construction time | Analysis IR Brief §Implementation Notes | ✓ Implemented |
| F8 | Referential integrity, uniqueness constraints, no partial facts | Analysis IR Brief §Invariants | ✓ Implemented |

---

## Specification Consistency Audit (E1–E7)

Items surfaced during architectural review that may not be reflected in all documents.

| # | Issue | Action | Status |
|---|-------|--------|--------|
| E1 | Three-layer model (Urd, Wyrd, Adapter) must be consistent across all docs | Verify Architecture doc names the adapter layer explicitly | ✓ Resolved (AR updated) |
| E2 | Failure contract (structured result, two categories, no mutation, no event) now specified in governance doc | Verify Wyrd Reference Runtime spec includes or references this | ✓ Resolved (WR updated) |
| E3 | `on_enter`/`on_exit` added to JSON Schema as erratum | Verify Schema Spec prose matches JSON Schema | ✓ Consistent |
| E4 | `on_condition` expressions — regex pattern in JSON Schema was overly restrictive | Verify fix is in published JSON Schema | ✓ Verified |
| E5 | "text composition" terminology — consistent across governance, presentation, essay | Verify Schema Markdown spec doesn't use "conditional text" in a conflicting way | ✓ Consistent |
| E6 | Lambda reframe — runtime-supervised sandboxed logic, not schema-embedded | Verify product vision and architecture doc use consistent framing | ✓ Consistent |
| E7 | Graph model paragraph — consider adding informative section to Schema Spec naming the graph structure explicitly | Optional, informative only | ✓ Skipped (FactSet canonical) |

---

## Specifications

| Document | Status |
|----------|--------|
| Schema Specification | ✓ Stable, normative |
| Schema Markdown Syntax Specification | ✓ Stable, normative |
| JSON Schema (`urd-world-schema.json`) | ✓ Published (with known errata fixed) |
| Architecture | ✓ Stable |
| Architectural Boundaries (governance) | ✓ Finalised — three layers, boundary test, permanent exclusions, failure contract |
| Wyrd Reference Runtime Specification | ✓ Stable |
| Test Case Strategy | ✓ Stable |
| Formal Grammar Brief | ✓ Stable |

---

## What Is Permanently Excluded (All Versions)

These are not deferrals. They are architectural exclusions per the governance document.

| Category | What It Means | Governance Section |
|---------|--------------|-------------------|
| Input model | No verb synonyms, parser grammar, disambiguation, input validation in schema or runtime. Wyrd receives action IDs. How they're produced is the adapter's problem. | Permanent Exclusion §1 |
| Text rendering / conditional presentation | No conditional description fields, text templates, multi-description patterns, markup in schema fields. `description` is a static hint. The adapter queries state and renders. | Permanent Exclusion §2 |
| Player experience feedback | No "fumble" actions, "attempt failed" events, adapter-specific side effects on condition failure. Wyrd returns structured failure results. The adapter decides the experience. | Permanent Exclusion §3 |
| Time and pacing | No clock, tick, turn, beat, or real-time duration. `always` fires per-action, not per-tick. Pacing is adapter-layer. | Permanent Exclusion §4 |
| Persistence and save/load | No save-file format in schema. No session management in runtime. `getState()` and state injection via API. Everything else is external. | Permanent Exclusion §5 |
| On-attempt / failure-triggered rules | World rules never fire on failed actions. "Attempting and failing" is an adapter-interaction concept. | Architectural Boundaries §on_attempt evaluation |
| Parser grammar block | Verb synonyms and grammar hints belong in adapter sidecar files, not `.urd.json`. | Architectural Boundaries §parser grammar evaluation |
| Conditional room descriptions | The schema provides state booleans. The adapter queries them and selects text. No `description_lit` / `description_dark` patterns. | Architectural Boundaries §conditional descriptions evaluation |

---

## What Is Deferred to Post-v1 (But Not Excluded)

| Feature | Why Deferred | v1 Workaround |
|---------|-------------|---------------|
| Cross-file section jumps | Design complexity. `->` jumps are file-scoped in v1. | Use location exits for cross-file movement. Entity state communicates across files. |
| Lambda functions | Extension host slot exists. Contract defined (read-only state in, effects out). Implementation is post-v1. | Express logic declaratively using rules, conditions, effects. |
| Owner visibility full semantics (ownership transfer) | Partially specified. `owner` level works. Transfer not specified. | Use `owner` for static ownership. Don't design content requiring ownership transfer. |
| Cross-file exhaustion sharing | Each file tracks exhaustion independently. | Accept independent exhaustion counters. Use entity state for cross-file communication. |
| Relationships (typed connections between entities) | Future schema extension. | Model with entity properties and conditions. |
| Knowledge model (what each entity knows) | Future schema extension. | Use hidden properties and visibility gating. |
| Time system (world clock, schedules) | Would extend runtime, not schema. | No v1 workaround — design content without time dependence. |
| Numeric subsystems (economy, health, reputation) | Future schema extension. | Use integer properties with rules. |

---

## Acceptance Criteria: When Is the Compiler Done?

### Compiler Gate

- [x] All nine compiler requirements (C1–C9) pass
  - [x] C1: Constrained frontmatter parsing (no general-purpose YAML)
  - [x] C2: Import resolution (explicit, non-transitive)
  - [x] C3: Circular import detection with full cycle path (URD202)
  - [x] C4: Duplicate entity ID rejection across compilation unit (URD302+)
  - [x] C5: Duplicate type name rejection across compilation unit (URD302+)
  - [x] C6: All @entity references, -> jumps, and property accesses resolve (URD301)
  - [x] C7: Property value validation against types, enums, ref constraints (URD410+)
  - [x] C8: Emit `.urd.json` with `urd: "1"` injected; warn and override if author sets it (URD411, compiler 0.1.7)
  - [x] C9: Nesting depth enforcement (warn at 3, error at 4+) (URD410, compiler 0.1.7)
- [x] All eight static analysis checks (S1–S8) implemented and tested
- [x] FactSet extraction (F1–F8) implemented: produces facts for all five canonical fixtures, determinism verified, referential integrity enforced
- [x] Five canonical fixtures compile without warnings — verified by `gate_canonical_fixtures_zero_warnings` test (compiler 0.1.7)
- [x] Compiled JSON validates against published JSON Schema — verified by `gate_json_schema_validates_all_fixtures` test using `jsonschema` crate (compiler 0.1.7)
- [x] Negative test corpus rejected with correct error locations — nine negative fixtures verified by `gate_negative_corpus_correct_codes` test with span location checks (compiler 0.1.7)

### Specification Gate

- [x] All E1–E7 consistency items verified or resolved — audit completed 2026-02-22
- [x] No contradictions between Schema Spec, JSON Schema, Schema Markdown Spec, Wyrd Spec, and Architectural Boundaries — one CRITICAL finding (on_exhausted goto) resolved, three MINOR findings resolved
- [x] Published JSON Schema matches all compiler outputs — re-verified after schema edit (exhaustedContent definition added)

---

## Implementation Sequence (Remaining Items)

1. ~~**Static analysis gaps** (S3, S4, S6, S8)~~ — ✓ Complete. Compiler 0.1.5.
2. ~~**FactSet extraction** (F1–F8)~~ — ✓ Complete. Compiler 0.1.6.
3. ~~**C8 verification** — `urd:` override warning (URD411)~~ — ✓ Complete. Compiler 0.1.7.
4. ~~**C9 verification** — nesting depth enforcement (URD410)~~ — ✓ Complete. Compiler 0.1.7.
5. ~~**Spec audit** (E1–E7) — verify consistency across all normative documents~~ — ✓ Complete. Audit 2026-02-22.
6. ~~**Schema validation tooling** — JSON Schema validation wrapped as `gate_json_schema_validates_all_fixtures` test~~ — ✓ Complete. Compiler 0.1.7.
7. ~~**Fixture and negative corpus verification** — `gate_canonical_fixtures_zero_warnings` and `gate_negative_corpus_correct_codes` tests~~ — ✓ Complete. Compiler 0.1.7.
8. ~~**Gate closure** — run all compiler acceptance criteria, fix what fails~~ — ✓ Closed. 2026-02-22.

Each step is independent. Nothing depends on the runtime or testing framework.

---

## What Comes After Compiler v1

Once the compiler gate closes, two paths open:

**FactSet-derived tooling.** The FactSet enables an entire class of compiler-side work that does not require Wyrd: derived diagnostics (property read but never written, write-set conflict detection), visualization (location graphs, dialogue flow graphs, dependency maps), LSP foundations (go-to-definition, find-all-references on properties), and structural analysis that tools and AI can consume. This work compounds the FactSet investment and can proceed independently.

**System gate.** The runtime (Wyrd), testing framework, and system-level acceptance criteria are defined in the System Gate document. That gate covers the 21 runtime requirements, the four test case playthroughs, Monte Carlo validation, and the boundary gate. It can be approached incrementally — a Wyrd proof-of-concept scoped to the Monty Hall fixture, then progressive expansion to the full test case suite.

The compiler gate does not block either path. Both can begin once the compiler is v1 complete.

---

*This document is the single reference for what "compiler v1 complete" means.*

---

## Gate Closed — 2026-02-22

**Compiler version:** 0.1.7
**Test count:** 554 (6 lib + 81 e2e + 68 emit + 31 facts + 55 import + 81 link + 124 parse + 108 validate)
**Pass rate:** 100%
**Schema tests:** 39 (7 positive + 25 negative + 7 compiler output)

### Specification Consistency Audit Record

| Item | Status | Severity | Finding |
|------|--------|----------|---------|
| E1 | FAIL | MINOR | AR uses 4-component model without naming the Adapter layer. Fixed: added architectural layers paragraph to AR referencing AB §The Three Layers. |
| E2 | FAIL | MINOR | WR omits failure contract. Fixed: added failure contract paragraph to WR §Public API referencing AB §Layer 2. |
| E3 | PASS | — | JS and SS agree on `on_enter`/`on_exit`. SM correctly omits (not writer-facing syntax). Compiler does not emit these fields (hand-authored JSON only). |
| E4 | PASS | — | JS advance pattern `on_condition .+` is sufficiently permissive. |
| E5 | PASS | — | SM uses "dialogue"/"prose"/"narrative" — no conflict with AB's "text composition" terminology. |
| E6 | PASS | — | All documents consistently frame lambdas as runtime extensions (Wyrd Extension Host), deferred to post-v1. |
| E7 | INFO | — | SS does not mention graph structure. Recommendation: skip. FactSet analysis IR is canonical for graph structure formalisation. |
| on_exhausted | FAIL | CRITICAL | Compiler emits `goto` on `on_exhausted` but JS (`speech` type) disallows it and SS does not document it. No canonical fixture triggers this path. Fixed: added `exhaustedContent` definition to JS, updated SS to document `goto` on `on_exhausted`. Re-verified: all 39 schema tests pass. |
| Check 1 | PASS | — | All five effect types (set, move, reveal, destroy, spawn) consistent across SS, JS, SM. |
| Check 2 | PASS | — | Five trigger types consistent across SS and JS pattern. |
| Check 3 | PASS | — | Four visibility levels consistent across SS and JS. |
| Check 4 | PASS | — | Condition expression format (AND array, OR `any:` object) consistent across SS and JS. |
| Check 5 | PASS | — | Dialogue section fields consistent. on_exhausted divergence resolved (see above). |
| Check 6 | FAIL | MINOR | SS lists 4 advance modes; JS and compiler support 6 (adds `auto`, `manual`). Fixed: added `auto` and `manual` to SS advance modes table. |
| Check 7 | PASS | — | Four traits (container, portable, mobile, interactable) consistent across SS, JS, and compiler. |
| Check 8 | PASS | — | All gate-referenced diagnostic codes exist in DC with correct severity and phase. No compiler codes absent from DC. |

### Documents edited during audit

| Document | Edit |
|----------|------|
| `packages/schema/urd-world-schema.json` | Added `exhaustedContent` definition; changed `on_exhausted` reference from `speech` to `exhaustedContent` |
| `docs/schema-spec.md` | Added `goto` to `on_exhausted` description; added `auto` and `manual` to advance modes table; updated "four" to "six" advance modes |
| `docs/wyrd-reference-runtime.md` | Added failure contract paragraph to §Public API |
| `docs/architecture.md` | Added architectural layers paragraph referencing AB §The Three Layers |

*End of Document*
