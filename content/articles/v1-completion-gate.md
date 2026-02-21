---
title: "The Gate"
slug: v1-completion-gate
description: A new planning document defines what v1 complete actually means — not the compiler running, but the system's foundational claims being provable.
date: "2026-02-21"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Why the v1 Completion Gate exists and what it demands.
> Single canonical copy. February 2026.

## Not "it compiles"

The compiler runs. Five phases, 480 tests, 100% pass rate, four canonical fixtures compiling to valid `.urd.json`. By most project metrics, this is a working system.

It is not v1.

v1 is the point at which the system's foundational claims are provable, not just specified. Deterministic replay, static analysis, multi-interface portability, AI-native consumption — these are not marketing claims. They are architectural properties. And architectural properties are only real when they can be demonstrated under test.

The [v1 Completion Gate](/documents/v1-completion-gate) defines what "done" means across five acceptance gates, with no ambiguity about what passes and what does not.

## Five gates

**Compiler Gate.** All specified primitives implemented. All eight static analysis checks passing — including four that do not yet exist: unreachable location detection, orphaned action detection, missing fallthrough warnings, and shadowed exit detection. The negative test corpus rejected with correct error locations.

**Runtime Gate.** Twenty-one requirements drawn from the Schema Specification, Architecture, and [Architectural Boundaries](/documents/architectural-boundaries) governance document. Everything from condition evaluation and containment modelling to the determinism contract and the structured failure result. The headline test: compile the Monty Hall problem, run it ten thousand times with seeded randomness, and verify the switching advantage converges to two-thirds.

**Testing Gate.** Scripted playthroughs, state assertions at arbitrary points, Monte Carlo mode, and schema validation. All four canonical test cases — Tavern, Monty Hall, Key Puzzle, Interrogation — passing as automated runtime playthroughs.

**Specification Gate.** Seven consistency items surfaced during the architectural review. The three-layer model named consistently across all documents. The failure contract referenced in the runtime spec. The JSON Schema matching every compiler output. No contradictions between the five normative documents.

**Boundary Gate.** No schema field or runtime behaviour that fails the [five-question boundary test](/documents/architectural-boundaries). Hint fields remain optional and non-load-bearing. Failed actions produce structured results, not events. No state mutation on failure.

## What does not exist yet

The document is honest about the gap. The runtime is not started. The testing framework is not started. Four of the eight static analysis checks need implementation. The specification consistency audit has seven open items.

The compiler is the foundation. The runtime is the critical path. The testing framework proves the claims. The specification audit catches contradictions before they become bugs.

## The implementation sequence

The document recommends eight steps, each producing a testable increment:

1. Specification audit — catch inconsistencies before building against them
2. Static analysis gaps — compiler improvements with no runtime dependency
3. Wyrd core engine — state, conditions, effects, rules, actions
4. Wyrd sequences — phase management, advance modes
5. Wyrd dialogue — sections, choices, exhaustion
6. Wyrd contracts — immutable state, events, determinism, failure results
7. Testing framework — schema validation, playthrough simulation, Monte Carlo
8. Acceptance verification — run all gates, fix what fails

Nothing depends on something later in the sequence.

## What is excluded

The document also names what v1 will never include — not as deferrals but as [permanent architectural exclusions](/documents/architectural-boundaries): input models, conditional text rendering, experience feedback, time, persistence. And it names what is deferred to post-v1 but not excluded: cross-file section jumps, lambda functions, relationships, knowledge models, time systems.

The distinction matters. Excluded features constrain every version. Deferred features wait for a later one.

## The full document

The complete v1 Completion Gate is available in the [artefacts section](/documents/v1-completion-gate) and on [GitHub](https://github.com/urdwyrd/urd/blob/main/docs/urd-v1-completion-gate.md).

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
