---
title: The Spec Survived
slug: spec-driven-compiler
description: A five-phase compiler built entirely from specifications, by AI, without a single amendment to the design documents. What that proves and what it does not.
date: "2026-02-18"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the completion of the compiler and what the spec-driven approach revealed.
> Single canonical copy. February 2026.

## The claim

On 17 February 2026, the compiler existed only as specifications: an [architecture brief](/documents/urd-compiler-architecture-brief) defining every shared data structure, and five phase briefs ([PARSE](/documents/urd-compiler-parse-brief), [IMPORT](/documents/urd-compiler-import-brief), [LINK](/documents/urd-compiler-link-brief), [VALIDATE](/documents/urd-compiler-validate-brief), [EMIT](/documents/urd-compiler-emit-brief)) defining every transformation, every diagnostic code, every error recovery strategy.

On 18 February, the compiler passes 302 tests across seven groups, compiles four canonical fixtures to valid `.urd.json`, and produces byte-identical output on repeated runs. The [test dashboard](/) on the homepage shows the numbers in real time.

The specifications were not amended. Not once. Not a field name, not a diagnostic code, not a phase boundary. The briefs written before implementation began are the same briefs that the implementation conforms to.

That is the claim. The rest of this article is about what it means, what it cost, and what it does not prove.

## What actually happened

The compiler was built in six sessions over roughly 24 hours. Each session implemented one brief: architecture scaffolding, then PARSE, IMPORT, LINK, VALIDATE, EMIT, in dependency order. Each session followed the same pattern:

1. Read the brief
2. Implement the phase
3. Write the tests specified in the brief
4. Make the tests pass
5. Run the full suite to verify no regressions

The AI — Claude — wrote all the Rust code. A human reviewed diffs, ran tests, and made architectural judgements when the brief was ambiguous about implementation detail (not behaviour). The distinction matters: the briefs specify *what* each phase must do, not *how* the Rust code should be structured internally. Function signatures, module layout, iterator patterns — these are implementation choices that the briefs deliberately do not constrain.

The six briefs together specified over 150 acceptance tests. The implementation passes all of them plus additional tests that emerged during development — edge cases the briefs implied but did not enumerate. The total stands at 302.

## Why the specs survived

This is the interesting question. Specifications rarely survive contact with implementation. Requirements change. Edge cases surface. Assumptions embedded in prose turn out to be wrong when translated to code. The usual outcome is a specification that is amended dozens of times during implementation, accumulating errata and revision marks until it no longer resembles the original.

Three things prevented that here.

**The formalisation phase did the hard work first.** Before the briefs existed, the project built a [PEG grammar](/articles/peg-grammar) that validates compiler input and a [JSON Schema](/articles/json-schema) that validates compiler output. These two machine-checkable artifacts forced every ambiguity in the prose specification to be resolved *before* implementation. By the time the compiler briefs were written, the input format and output format were already proven consistent. The briefs only had to specify the transformation between them.

**The briefs specified contracts, not code.** Each brief defines input types, output types, diagnostic codes, error recovery rules, and acceptance tests. It does not specify function names, module boundaries, or algorithmic approaches. This gives the implementation room to manoeuvre without violating the specification. When a brief says "LINK must resolve entity references using visible scope rules," the implementation can use hash maps, tries, or linear search — the brief does not care, as long as the tests pass.

**The diagnostic code ranges created natural boundaries.** PARSE owns URD100–URD199. IMPORT owns URD200–URD299. LINK owns URD300–URD399. VALIDATE owns URD400–URD499. No collisions are possible. When implementing VALIDATE, there is no temptation to emit a PARSE diagnostic or to check something that is LINK's responsibility. The code ranges are more than identifiers — they are a coordination mechanism that prevents scope creep between phases.

## The AI question

This project is, among other things, an experiment in AI-driven development. The [community feedback](/articles/a-human-entered-the-room) was direct: specifications are not games. Show us something that works.

The compiler is something that works.

It is worth being precise about what the AI did and did not do. Claude wrote all the Rust code — approximately 8,000 lines of implementation and 6,000 lines of tests. A human wrote the specifications, reviewed every diff, and made every architectural decision. The AI did not design the compiler. It implemented a design that was already complete.

This is the model the project is testing: human specification, AI implementation, machine verification. The human decides what the system should do. The AI writes the code. The test suite proves it works. If the tests fail, the AI fixes the code — not the spec.

In this case, the AI never needed to fix the spec. Whether that generalises beyond a compiler — a domain with well-understood patterns and unambiguous correctness criteria — is an open question. Compilers are unusually amenable to specification-driven development because the input and output are both formal. A runtime that executes interactive worlds, with state transitions and emergent behaviour, may be harder. We will find out.

## What this does not prove

**It does not prove the specification is correct.** The compiler faithfully implements the briefs. If the briefs contain a design flaw — a type system that cannot express a necessary constraint, a visibility model that leaks hidden state — the compiler will faithfully implement that flaw. Correctness requires end-to-end testing: compile a world, run it, verify the behaviour. The compiler is the first half of that pipeline. Wyrd, the reference runtime, is the second.

**It does not prove the approach scales.** 302 tests across five phases is significant but not large by compiler standards. The Urd schema is deliberately constrained — seven symbols, no general-purpose programming constructs, no Turing-complete logic. A compiler for a more complex language might surface ambiguities that force specification changes. The briefs survived because the language they compile is small enough to fully specify.

**It does not prove AI can replace engineers.** The human wrote six detailed briefs before any code existed. That is the engineering work. The AI translated those briefs into Rust. That is the implementation work. The experiment shows that the translation step can be automated, not that the design step can be eliminated.

## The timeline

The full path from prose to passing tests:

| Date | Milestone |
|------|-----------|
| Feb 2026 | 12 specification documents published |
| 15 Feb | PEG grammar crate — 75 rules, 12 fixtures |
| 16 Feb | JSON Schema — 9 sub-schemas, 32 fixtures |
| 17 Feb | Six compiler briefs published |
| 17 Feb | PARSE phase — 42 tests |
| 17 Feb | IMPORT phase — 18 tests |
| 18 Feb | LINK phase — 54 tests |
| 18 Feb | VALIDATE phase — 107 tests |
| 18 Feb | EMIT phase — 68 tests |
| 18 Feb | E2E integration — 14 tests |
| 18 Feb | Test dashboard live — 302 total |

From locked specification to working compiler: approximately 48 hours. From first line of Rust to 302 passing tests: approximately 24 hours. The specifications took longer to write than the compiler took to build. That is, arguably, exactly the right ratio.

## What comes next

The compiler answers "does the specification parse and transform correctly?" It does not answer "does the specification *work*?"

That question requires the runtime. Compile the Monty Hall problem. Run it ten thousand times. Verify the switching advantage converges to two-thirds. If it does, the declarative schema can express emergent probability — the first proof that the design is not merely syntactically valid but behaviourally correct.

The compiler is ready. The [test dashboard](/) proves it. Now we build Wyrd.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
