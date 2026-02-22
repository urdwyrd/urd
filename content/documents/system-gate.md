---
title: "System Gate"
slug: "system-gate"
description: "Everything that must be true before the Urd system — compiler, runtime, and testing framework — can be called complete. Runtime requirements, testing framework, boundary enforcement, and system-level acceptance criteria."
category: "architecture"
format: "Planning Document"
date: "2026-02-22"
status: "planning"
order: 4
tags:
  - architecture
  - planning
  - runtime
  - testing
  - acceptance
details:
  - "21 runtime requirements (R1–R21) from the specification suite"
  - "5 testing framework requirements (T1–T5)"
  - "Boundary gate — five-question boundary test"
  - "Soufflé/Datalog discussion: what belongs in the runtime API"
  - "Monty Hall proof-of-concept as incremental milestone"
  - "System-level acceptance checklist"
---

# Urd — System Gate

*Everything that must be true before the Urd system can be called complete*

> **Document status: PLANNING**
> Covers the runtime (Wyrd), testing framework, and system-level acceptance criteria. The compiler gate is defined separately in the [v1 Completion Gate: Compiler](/documents/v1-completion-gate).
> February 2026.

---

## What "System Complete" Means

The compiler gate validates that the compiler implements every specified primitive, produces correct output, and can analyze the worlds it compiles. The system gate validates that those compiled worlds *work* — that they execute correctly, deterministically, and within the architectural boundaries.

System complete means:

1. The Wyrd reference runtime executes all five canonical fixtures correctly
2. The determinism contract is verified: same JSON + same seed + same actions = identical event stream
3. The testing framework can run scripted playthroughs and statistical validation
4. The boundary is enforced — nothing in the schema or runtime that belongs in the adapter layer
5. Failed actions produce structured results, not events, with no state mutation

---

## Prerequisite: Compiler Gate

The system gate depends on the compiler gate being closed. Specifically:

- All compiler requirements (C1–C9) pass
- All static analysis checks (S1–S8) implemented
- FactSet extraction (F1–F8) implemented
- Specification consistency audit (E1–E7) resolved
- All canonical fixtures compile cleanly

See [v1 Completion Gate: Compiler](/documents/v1-completion-gate) for current status.

---

## Runtime (Wyrd) — Requirements

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

---

## Testing Framework

| # | Requirement | Source |
|---|------------|--------|
| T1 | Schema validation — validate `.urd.json` against JSON Schema | Architecture §Testing |
| T2 | Scripted playthrough execution (`Wyrd.simulate(actions)`) | Test Case Strategy §Test Execution Model |
| T3 | State assertions at any point during playthrough | Test Case Strategy §Test Definition Format |
| T4 | Monte Carlo mode — thousands of seeded playthroughs, aggregate assertions | Test Case Strategy §Statistical Validation |
| T5 | Five test cases passing as runtime playthroughs | Test Case Strategy §The Four Test Cases |

---

## Incremental Milestones

The system gate does not need to be approached as a single block. It can be built incrementally:

### Wyrd Proof-of-Concept: Monty Hall

A minimal runtime scoped to the Monty Hall fixture. This proves the schema-to-execution pipeline works and delivers the headline demo: seeded random convergence to 2/3.

**Required subset:** R1, R2, R3, R5 (set only), R8 (always trigger only), R9, R10, R17, R19, R20.

**Not required:** Containment (R6), visibility (R7), sequences (R11), dialogue (R12–R16), on_enter/on_exit (R16), event sourcing (R18), move/destroy/reveal/spawn effects, any: conditions (R4), structured failure (R21).

### Wyrd Core: Key Puzzle + Tavern

Adds containment, movement, destroy, dialogue, sticky/one-shot choices, exhaustion. Covers R4–R7, R12–R16.

### Wyrd Full: Interrogation + Contracts

Adds multi-file dialogue, event sourcing, determinism verification, failure contracts. Covers R11, R18, R21.

### Testing Framework

Schema validation, scripted playthroughs, Monte Carlo mode. Covers T1–T5.

---

## What the Soufflé Discussion Adds

The Soufflé/Datalog discussion surfaced capabilities that belong in the system. The critical distinction: **derived state computation belongs in Wyrd's API, not in the schema or as author-exposed logic.**

### In Scope (Wyrd API surface)

These are read-only queries the runtime computes from world state. They pass the boundary test (Question 3: describes how state is evaluated → Wyrd). They don't add schema primitives or author-facing features.

| Capability | What It Does | Why It Matters |
|-----------|-------------|----------------|
| Reachability query | Given current state, which locations can the player reach? | Static analysis (S3), testing, adapter tooling |
| Available actions query | Given current state, which actions have satisfiable conditions? | Already implicit in `getAvailableActions()` API |
| Containment tree query | Transitive containment — what's inside what, recursively | Inventory display, adapter tooling, visibility auditing |
| Visible entities query | Given a location and a viewer, which entities and properties are visible? | Visibility auditing (Architecture §Testing), adapter rendering |
| Exhaustion check | Is a given dialogue section exhausted? | Already specified as runtime predicate |

### Post-System-Gate (Wyrd Enhancement)

| Capability | Why Not Now | Where It Goes |
|-----------|-----------|---------------|
| Full graph query API (`getReachableLocations()`, `getContainmentTree()`) as first-class Wyrd methods | Useful but not required for test cases to pass. | Future Proposals — post-system Wyrd API extension |
| Derived state caching / incremental recomputation | Performance optimisation, not correctness | Future Proposals |

### Permanently Excluded (Boundary Violation)

| Capability | Why Excluded | Governance Reference |
|-----------|-------------|---------------------|
| Author-exposed recursive derivation rules | Introduces Turing-completeness risk. Recursive rules can diverge. Fails boundary test Q2a (declarative and verifiable). | Architectural Boundaries §Q2 |
| Implicit derivation exposed to writers (Datalog-style rule authoring) | Crosses from "constrained graph transformations" into "logic programming." | Architectural Boundaries §Purpose |
| Fixpoint evaluation / forward-chaining inference | Urd performs controlled, event-driven transitions in response to explicit actions. No implicit propagation. | Schema Spec §Evaluation Order |

**The line:** Wyrd can internally compute anything derivable from the world graph. It can expose derived properties through its API. But the schema never contains derivation rules, and authors never write inference logic. The schema is base facts + conditional transitions. The runtime does the rest.

---

## Acceptance Criteria: When Is the System Done?

### Runtime Gate

- [ ] All twenty-one runtime requirements (R1–R21) implemented
- [ ] Determinism contract verified: same JSON + same seed + same actions = identical event stream across runs
- [ ] Monty Hall 10,000-run Monte Carlo: switching advantage converges to 2/3 (0.6 < ratio < 0.7)
- [ ] Key Puzzle: pick up key → unlock door → key destroyed → reach corridor
- [ ] Tavern Scene: sticky choices persist, one-shot choices disappear, exhaustion triggers fallthrough
- [ ] Interrogation: multi-topic hub, conditional branches, containment transfer in dialogue, state-dependent farewell

### Testing Gate

- [ ] All five test cases pass as automated playthroughs
- [ ] Monte Carlo mode operational
- [ ] State assertions work at arbitrary playthrough points
- [ ] Schema validation catches malformed `.urd.json`

### Boundary Gate

- [ ] No schema field or runtime behaviour that fails the five-question boundary test
- [ ] All hint fields (`description`, `blocked_message`, `prompt`) remain optional and non-load-bearing
- [ ] Runtime `perform()` returns structured failure results per the failure contract
- [ ] No events emitted on failed actions
- [ ] No state mutation on failed actions

---

## Implementation Sequence (Recommended)

This is a suggested order, not a mandate. Dependencies are noted.

1. **Wyrd proof-of-concept** — Monty Hall only (R1, R2, R3, R5-partial, R8-partial, R9, R10, R17, R19, R20)
2. **Wyrd core** — Key Puzzle + Tavern (R4–R7, R12–R16)
3. **Wyrd contracts** — Interrogation + full contracts (R11, R18, R21)
4. **Testing framework** (T1–T5) — schema validation, playthrough simulation, Monte Carlo
5. **Acceptance verification** — run all gates, fix what fails

Each step produces a testable increment. The proof-of-concept can be built and demonstrated independently.

---

*This document is the reference for system-level completion. The compiler gate is defined separately in the [v1 Completion Gate: Compiler](/documents/v1-completion-gate).*

*End of Document*
