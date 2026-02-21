---
title: "v1 Completion Gate"
slug: "v1-completion-gate"
description: "Everything that must be true before Urd v1 can be called done. Acceptance criteria across five gates — compiler, runtime, testing, specification, and boundary — with current status, implementation sequence, and permanent exclusions."
category: "architecture"
format: "Planning Document"
date: "2026-02-21"
status: "planning"
order: 3
tags:
  - architecture
  - planning
  - v1
  - acceptance
  - runtime
  - testing
details:
  - "Eight-point definition of what v1 complete means"
  - "Current status audit: compiler, runtime, testing, specifications"
  - "21 runtime requirements (R1–R21) from the specification suite"
  - "5 testing framework requirements (T1–T5)"
  - "8 static analysis checks (S1–S8) with implementation status"
  - "7 specification consistency items (D1–D7)"
  - "Five acceptance gates: compiler, runtime, testing, specification, boundary"
  - "Recommended implementation sequence"
  - "Permanent exclusions and post-v1 deferrals"
---

# Urd — v1 Completion Gate

*Everything that must be true before Urd v1 can be called done*

> **Document status: PLANNING**
> Synthesised from the Schema Specification, Architecture, Architectural Boundaries governance, Test Case Strategy, Formal Grammar Brief, and the Soufflé/graph-model discussion. This document is the acceptance gate for declaring v1 complete.
> February 2026.

---

## What "v1 Complete" Means

v1 complete is not "the compiler runs." The compiler already runs — five phases, 480 tests, 100% pass rate, four canonical fixtures compiling to valid `.urd.json`.

v1 complete means:

1. The specification suite is internally consistent (no contradictions between docs)
2. The compiler implements every specified primitive
3. The JSON Schema validates every compiler output and rejects invalid hand-authored JSON
4. The static analysis checks specified in the Architecture and Test Case Strategy are implemented
5. The boundary is enforced — nothing in the schema or runtime that belongs in the adapter layer
6. The four test cases pass as both compilation targets and runtime playthroughs
7. The Wyrd reference runtime executes all four test cases correctly
8. The testing framework can run scripted playthroughs and statistical validation

This is not Phase 1 of the product roadmap. This is the point at which the system's foundational claims are provable, not just specified.

---

## Current State: What Exists

### Compiler (Urd)

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
| Four canonical fixtures (Tavern, Monty Hall, Key Puzzle, Interrogation) | ✓ Compiling |
| Static analysis: all eight checks (S1–S8) | ✓ Implemented (compiler 0.1.5) |
| 516 tests, 100% pass rate | ✓ Current |

### Runtime (Wyrd)

| Capability | Status |
|-----------|--------|
| Reference runtime | ✗ Not started |

### Testing Framework

| Capability | Status |
|-----------|--------|
| Schema validation against JSON Schema | Partial (JSON Schema exists, tooling not wrapped) |
| Playthrough simulation | ✗ Not started |
| Statistical validation (Monte Carlo) | ✗ Not started |
| Static analysis checks | ✓ Complete — all eight checks (S1–S8) implemented. |

### Specifications

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

## What Must Be Built for v1 Complete

### A. Runtime (Wyrd) — The Critical Path

Everything below is specified. Nothing requires new design work.

| # | Requirement | Source | Notes |
|---|------------|--------|-------|
| R1 | Load and validate `.urd.json` | Schema Spec §Evaluation Order | Reject unknown `urd` version |
| R2 | World state initialisation | Schema Spec §Entity Defaults | Create entities with defaults, place in declared containers |
| R3 | Condition evaluation engine | Schema Spec §Expressions | `==`, `!=`, `>`, `<`, `>=`, `<=`, `in`, `not in`, boolean combinators |
| R4 | `any:` OR condition blocks | Schema Spec §Conditions | Already in JSON Schema; runtime must evaluate |
| R5 | All five effect types: `set`, `move`, `reveal`, `destroy`, `spawn` | Schema Spec §Effect Declarations | `spawn` is specified but not yet test-covered |
| R6 | Containment model | Schema Spec §Containment Model | `move` is sole spatial operation, implicit `player` entity |
| R7 | All four visibility levels: `visible`, `hidden`, `owner`, `conditional` | Schema Spec §Visibility Model | `conditional` is specified but not yet test-covered |
| R8 | Rule execution with trigger matching | Schema Spec §Rules | Five trigger types: `phase_is`, `action`, `enter`, `state_change`, `always` |
| R9 | `select` block with seeded uniform random | Schema Spec §The select Block | Determinism contract: identical seed → identical result |
| R10 | Action resolution (conditions → effects → rule triggers) | Schema Spec §Actions | `getAvailableActions()` API |
| R11 | Sequence management with four advance modes | Schema Spec §Sequences | `on_action`, `on_rule`, `on_condition`, `end` |
| R12 | Dialogue state management | Schema Spec §Dialogue | Active section, consumed choices, `goto` jumps |
| R13 | Sticky vs one-shot choice tracking | Schema Spec §Dialogue | `+` persists, `*` consumed after selection |
| R14 | Exhaustion as runtime predicate | Schema Spec §Dialogue | Computed, never stored. All choices consumed or gated → exhausted |
| R15 | `on_exhausted` fallthrough content | Schema Spec §Dialogue | Displayed when section exhausts |
| R16 | `on_enter` / `on_exit` effects on locations | Schema Spec §Locations + JSON Schema erratum | Fire when entity moves into/out of location |
| R17 | Immutable state transitions | Architecture §Wyrd | Each action → new WorldState, not mutation |
| R18 | Event sourcing | Architecture §Wyrd | Every mutation produces a typed event |
| R19 | Determinism contract | Schema Spec §Determinism Contract | Same JSON + same seed + same actions = same event stream |
| R20 | `world.seed` API | Schema Spec §Determinism Contract | Accept seed from JSON or via API |
| R21 | Structured failure result from `perform()` | Architectural Boundaries §Failure Contract | Two categories: validation failure vs world-state failure. Structured result, not event. No state mutation on failure. Never in event stream. |

### B. Testing Framework

| # | Requirement | Source |
|---|------------|--------|
| T1 | Schema validation — validate `.urd.json` against JSON Schema | Architecture §Testing |
| T2 | Scripted playthrough execution (`Wyrd.simulate(actions)`) | Test Case Strategy §Test Execution Model |
| T3 | State assertions at any point during playthrough | Test Case Strategy §Test Definition Format |
| T4 | Monte Carlo mode — thousands of seeded playthroughs, aggregate assertions | Test Case Strategy §Statistical Validation |
| T5 | Four test cases passing as runtime playthroughs | Test Case Strategy §The Four Test Cases |

### C. Static Analysis (Compiler)

These are specified in the Architecture and Test Case Strategy. All eight checks are now implemented as of compiler 0.1.5.

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

### D. Specification Consistency Audit

Items surfaced during today's architectural review that may not be reflected in all documents.

| # | Issue | Action |
|---|-------|--------|
| D1 | Three-layer model (Urd, Wyrd, Adapter) must be consistent across all docs | Verify Architecture doc names the adapter layer explicitly |
| D2 | Failure contract (structured result, two categories, no mutation, no event) now specified in governance doc | Verify Wyrd Reference Runtime spec includes or references this |
| D3 | `on_enter`/`on_exit` added to JSON Schema as erratum | Verify Schema Spec prose matches JSON Schema |
| D4 | `on_condition` expressions — regex pattern in JSON Schema was overly restrictive | Verify fix is in published JSON Schema |
| D5 | "text composition" terminology — consistent across governance, presentation, essay | Verify Schema Markdown spec doesn't use "conditional text" in a conflicting way |
| D6 | Lambda reframe — runtime-supervised sandboxed logic, not schema-embedded | Verify product vision and architecture doc use consistent framing |
| D7 | Graph model paragraph — consider adding informative section to Schema Spec naming the graph structure explicitly | Optional, informative only |

---

## What the Soufflé Discussion Adds to v1

The Soufflé/Datalog discussion surfaced capabilities that belong in the system. The critical distinction: **derived state computation belongs in Wyrd's API, not in the schema or as author-exposed logic.**

### In Scope for v1 (Wyrd API surface)

These are read-only queries the runtime computes from world state. They pass the boundary test (Question 3: describes how state is evaluated → Wyrd). They don't add schema primitives or author-facing features.

| Capability | What It Does | Why It Matters |
|-----------|-------------|----------------|
| Reachability query | Given current state, which locations can the player reach? | Static analysis (S3), testing, adapter tooling |
| Available actions query | Given current state, which actions have satisfiable conditions? | Already implicit in `getAvailableActions()` API |
| Containment tree query | Transitive containment — what's inside what, recursively | Inventory display, adapter tooling, visibility auditing |
| Visible entities query | Given a location and a viewer, which entities and properties are visible? | Visibility auditing (Architecture §Testing), adapter rendering |
| Exhaustion check | Is a given dialogue section exhausted? | Already specified as runtime predicate |

These are the "Datalog-derived" properties — base facts (world definition) + rules (conditions/effects) = derived state (reachable, visible, available). The runtime computes them. Authors don't write derivation logic.

### Not in Scope for v1 (Post-v1 Wyrd Enhancement)

| Capability | Why Not Now | Where It Goes |
|-----------|-----------|---------------|
| Full graph query API (`getReachableLocations()`, `getContainmentTree()`) as first-class Wyrd methods | Useful but not required for the four test cases to pass. The runtime can compute these internally; exposing them as API is a convenience. | Future Proposals — post-v1 Wyrd API extension |
| Derived state caching / incremental recomputation | Performance optimisation, not correctness | Future Proposals |

### Permanently Excluded (Boundary Violation)

| Capability | Why Excluded | Governance Reference |
|-----------|-------------|---------------------|
| Author-exposed recursive derivation rules | Introduces Turing-completeness risk. Recursive rules can diverge. Fails boundary test Q2a (declarative and verifiable). Urd's static analysis properties depend on the absence of open-ended inference. | Architectural Boundaries §Q2 |
| Implicit derivation exposed to writers (Datalog-style rule authoring) | Crosses from "constrained graph transformations" into "logic programming." The moment authors write recursive derivation, you lose the halting-decidability that makes static analysis work. | Architectural Boundaries §Purpose |
| Fixpoint evaluation / forward-chaining inference | Urd performs controlled, event-driven transitions in response to explicit actions. No implicit propagation. No convergence semantics. | Schema Spec §Evaluation Order |

**The line:** Wyrd can internally compute anything derivable from the world graph. It can expose derived properties through its API. But the schema never contains derivation rules, and authors never write inference logic. The schema is base facts + conditional transitions. The runtime does the rest.

---

## What Is Permanently Excluded from v1 (and All Future Versions)

These are not deferrals. They are architectural exclusions per the governance document.

| Category | What It Means | Governance Section |
|---------|--------------|-------------------|
| Input model | No verb synonyms, parser grammar, disambiguation, input validation in schema or runtime. Wyrd receives action IDs. How they're produced is the adapter's problem. | Permanent Exclusion §1 |
| Text rendering / conditional presentation | No conditional description fields, text templates, multi-description patterns, markup in schema fields. `description` is a static hint. The adapter queries state and renders. | Permanent Exclusion §2 |
| Player experience feedback | No "fumble" actions, "attempt failed" events, adapter-specific side effects on condition failure. Wyrd returns structured failure results. The adapter decides the experience. | Permanent Exclusion §3 |
| Time and pacing | No clock, tick, turn, beat, or real-time duration. `always` fires per-action, not per-tick. Pacing is adapter-layer. | Permanent Exclusion §4 |
| Persistence and save/load | No save-file format in schema. No session management in runtime. `getState()` and state injection via API. Everything else is external. | Permanent Exclusion §5 |
| On-attempt / failure-triggered rules | World rules never fire on failed actions. "Attempting and failing" is an adapter-interaction concept. If a game needs a fumble counter, the adapter detects failure and performs a separate valid action. | Architectural Boundaries §on_attempt evaluation |
| Parser grammar block | Verb synonyms and grammar hints belong in adapter sidecar files, not `.urd.json`. | Architectural Boundaries §parser grammar evaluation |
| Conditional room descriptions | The schema provides state booleans. The adapter queries them and selects text. No `description_lit` / `description_dark` patterns. | Architectural Boundaries §conditional descriptions evaluation |

---

## What Is Deferred to Post-v1 (But Not Excluded)

These are legitimate future features that pass the boundary test but are not in scope for v1.

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
| Graph query API as first-class Wyrd methods | Convenience, not correctness. | Runtime computes internally; not exposed as named API methods yet. |

---

## Acceptance Criteria: When Is v1 Done?

### Compiler Gate

- [ ] All nine compiler requirements (C1–C9 from Architecture §v1 Acceptance Checklist) pass
- [x] All eight static analysis checks (S1–S8) implemented and tested
- [ ] Four canonical fixtures compile without warnings
- [ ] Compiled JSON validates against published JSON Schema
- [ ] Negative test corpus (bad-*.urd.md files) rejected with correct error locations

### Runtime Gate

- [ ] All twenty-one runtime requirements (R1–R21) implemented
- [ ] Determinism contract verified: same JSON + same seed + same actions = identical event stream across runs
- [ ] Monty Hall 10,000-run Monte Carlo: switching advantage converges to 2/3 (0.6 < ratio < 0.7)
- [ ] Key Puzzle: pick up key → unlock door → key destroyed → reach corridor
- [ ] Tavern Scene: sticky choices persist, one-shot choices disappear, exhaustion triggers fallthrough
- [ ] Interrogation: multi-topic hub, conditional branches, containment transfer in dialogue, state-dependent farewell

### Testing Gate

- [ ] All four test cases pass as automated playthroughs
- [ ] Monte Carlo mode operational
- [ ] State assertions work at arbitrary playthrough points
- [ ] Schema validation catches malformed `.urd.json`

### Specification Gate

- [ ] All D1–D7 consistency items verified or resolved
- [ ] No contradictions between Schema Spec, JSON Schema, Schema Markdown Spec, Wyrd Spec, and Architectural Boundaries
- [ ] Published JSON Schema matches all compiler outputs

### Boundary Gate

- [ ] No schema field or runtime behaviour that fails the five-question boundary test
- [ ] All hint fields (`description`, `blocked_message`, `prompt`) remain optional and non-load-bearing
- [ ] Runtime `perform()` returns structured failure results per the failure contract
- [ ] No events emitted on failed actions
- [ ] No state mutation on failed actions

---

## Implementation Sequence (Recommended)

This is a suggested order, not a mandate. Dependencies are noted.

1. ~~**Static analysis gaps** (S3, S4, S6, S8)~~ — ✓ Complete. Compiler 0.1.5. URD430, URD432, URD433, URD434.
2. **Spec audit** (D1–D7) — catch inconsistencies before building the runtime
3. **Wyrd core engine** (R1–R10) — state management, conditions, effects, rules, actions
4. **Wyrd sequences** (R11) — phase management, advance modes
5. **Wyrd dialogue** (R12–R16) — sections, choices, exhaustion, on_enter/on_exit
6. **Wyrd contracts** (R17–R21) — immutable state, events, determinism, failure contract
7. **Testing framework** (T1–T5) — schema validation, playthrough simulation, Monte Carlo
8. **Acceptance verification** — run all gates, fix what fails

Each step produces a testable increment. Nothing depends on something later in the sequence.

---

*This document is the single reference for what "v1 complete" means. When every gate passes, v1 ships.*

*End of Document*
