---
title: Entering the Validation Phase
slug: entering-validation
description: The formalisation phase is complete. Both machine-checkable artifacts are in place. Now the project must prove the specification works.
date: "2026-02-17"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Marking the transition from formalisation to validation.
> Single canonical copy. February 2026.

## What just finished

The formalisation phase had one job: turn prose specifications into machine-checkable artifacts. It produced two.

The [PEG grammar](/articles/peg-grammar) validates the compiler's input. Given a `.urd.md` file, it accepts or rejects — and if it rejects, it reports the exact line, column, and rule where the problem is. Seventy-five rules covering the full Schema Markdown syntax, tested against twelve fixtures.

The [JSON Schema](/articles/json-schema) validates the compiler's output. Given a `.urd.json` file, it checks that required fields are present, types are correct, enum values are within declared sets, and object shapes match the specification. Eight top-level blocks, nine reusable sub-schemas, tested against thirty-two fixtures.

Together they define the machine-checkable boundaries of Urd:

```
  .urd.md ── PEG grammar says: well-formed? ── ✓
                                                 │
                     COMPILER (not yet built)     │
                                                 │
  .urd.json ── JSON Schema says: conformant? ── ✓
```

The input gate and the output contract are both in place. The gap between them is the compiler.

## What is changing

The validation phase is about closing that gap. The question shifts from "what does valid Urd look like?" to "does the specification actually work?"

The deliverables are concrete:

**A compiler.** Rust, five phases: parse, resolve, link, validate, emit. Phase 1 (parse) already has its foundation — the PEG grammar crate becomes a dependency. The compiler reads `.urd.md`, walks the parse tree, builds an AST, resolves imports, links cross-references, validates types, and emits `.urd.json`.

**End-to-end proof.** The Monty Hall problem is the first target. A complete `.urd.md` source file goes in. A `.urd.json` file comes out. The JSON Schema validates the output. If the schema accepts it, the pipeline is connected.

**The probability test.** Compile Monty Hall, run it ten thousand times, verify the switching advantage converges to two-thirds. This is the first test that exercises not just structure but *behaviour*. The declarative schema claims that hidden state, constrained random selection, and conditional effects can produce emergent probability. Ten thousand runs either confirm it or expose a design flaw.

**The spatial test.** The two-room key puzzle exercises the containment model: pick up a key, unlock a door, move between rooms. This tests entity movement, conditional exits, blocked messages, and destroy effects — the physical mechanics that the Monty Hall problem does not touch.

**The dialogue test.** The tavern scenario exercises the conversation system: sections, choices (sticky and one-shot), conditions, effects, goto jumps, and exhaustion. This tests the recursive choice structure and the flat section map that the JSON Schema already validates structurally.

Each test either validates the design or forces it to change. Both outcomes get published.

## What has not changed

The specification itself is untouched. Neither the formalisation exercise nor the translation to machine-checkable artifacts has identified a structural flaw in the schema design.

The experimental thesis is unchanged: can a formal specification drive AI-built implementation? The formalisation phase sharpened the specification. The validation phase tests whether it holds under execution.

## The project timeline

Four phases, three complete:

| Phase | Status | What it produced |
|-------|--------|-----------------|
| Research | Complete | Landscape survey, pain points, product vision |
| Specification | Complete | 12 documents: syntax spec, world schema, architecture, runtime contract |
| Formalisation | Complete | PEG grammar (input), JSON Schema (output), monorepo scaffold, CI pipeline |
| **Validation** | **Active** | Compiler, test fixtures, schema conformance — proving the spec works end-to-end |

## What comes next

The compiler. The Monty Hall problem. Ten thousand runs.

If the switching advantage converges to two-thirds, the declarative thesis holds for at least one non-trivial case. Then the key puzzle. Then dialogue. Each milestone is a test that the specification survives contact with reality.

The formalisation phase gave us the tools to check our work automatically. The validation phase is where we find out whether the work is worth checking.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
