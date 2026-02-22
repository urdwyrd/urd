---
title: "The Experiment Becomes a System"
slug: queryable-semantic-graph
description: The FactSet — a flat, queryable intermediate representation of every relationship in a compiled world — marks the point where Urd stops being a working prototype and starts becoming infrastructure.
date: "2026-02-22"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the introduction of the analysis IR (FactSet) and what it means for the project's trajectory.
> Single canonical copy. February 2026.

## What changed

Urd has been approaching its [v1 completion gate](/documents/v1-completion-gate) as a compiler and schema system. Five phases, 516 tests, four canonical fixtures compiling to valid `.urd.json`. Static analysis checks that catch unreachable locations, impossible conditions, missing fallthrough, and naming collisions. A formal grammar, a JSON Schema, and a specification suite that still needs its consistency audit. The compiler gate is getting close.

Within that trajectory, the FactSet is a different kind of milestone. Not another check passing or another fixture compiling — but an architectural investment that changes the shape of what comes after.

## The problem it solves

Every static analysis check in VALIDATE had to rediscover the world from scratch. S3 built a location graph from exit symbols. S6 walked dialogue sections to check for fallthrough. S4 scanned conditions looking for impossible enum comparisons. Each check solved its own problem. None of them shared a model.

They were all looking at the same relationships — locations connected by exits, choices guarded by conditions, effects writing properties that conditions later read — but each one reconstructed those relationships independently from the AST and symbol table.

This is fine when you have four checks. It is not fine when you want twenty, or when you want tooling, or when you want a runtime that can explain its own decisions.

## The FactSet

The analysis IR brief introduces an intermediate representation called the FactSet. It sits between LINK's resolved world and any downstream consumer — static analysis, the Wyrd runtime, an LSP, visualization tools.

The FactSet is a flat set of typed tuples extracted after resolution. Each tuple represents one atomic relationship the compiler already knows about:

| Relation | What it captures |
|----------|-----------------|
| `PropertyRead` | A condition depends on an entity property |
| `PropertyWrite` | An effect modifies an entity property |
| `ExitEdge` | One location connects to another |
| `JumpEdge` | One dialogue section declares a transition to another |
| `ChoiceFact` | A player choice exists, with its conditions and effects indexed |
| `RuleFact` | A rule exists, with its conditions and effects indexed |

This is not inferred data. It is not computed at runtime. It is a materialized view of what the compiler already resolved — normalized, cross-referenced, and queryable without touching the AST again.

The FactSet is immutable, deterministic, and complete. For a given linked compilation unit, the extraction always produces the same output. Once constructed, nothing modifies it. Consumers read it. They do not write to it.

## Why this matters more than it looks

The simplest proof of value is a diagnostic that would be painful to write without the FactSet and trivial to write with it:

> "Property is read in a condition but never written by any effect."

With the FactSet, this is roughly ten lines of code: collect every `PropertyKey` that appears in a `PropertyRead`, check whether any `PropertyWrite` exists for the same key. Done. Without the FactSet, it requires a full AST traversal collecting both conditions and effects, cross-referencing resolved types and properties, and handling all the edge cases of nested choices and rule effects. Essentially reimplementing most of the extraction.

That single diagnostic justifies the entire layer. But the FactSet is not really about one diagnostic. It is about what kind of system Urd is becoming.

## The separation that matters

There is a tempting path here that leads nowhere: using the FactSet as a database and executing narratives from it. Database-driven interactive fiction has been tried before, and it fails because it collapses execution, authoring, and analysis into the same model.

Urd does the opposite. The brief is precise about this:

- `.urd.json` is the runtime model. Wyrd executes it.
- The FactSet is the analysis model. Tools query it.

The FactSet does not evaluate conditions. It does not simulate play. It does not track state. It describes relationships — which conditions read which properties, which effects write them, which exits connect which locations, which sections transition to which other sections.

That distinction is the difference between a system that runs stories and a system that can reason about them at compile time.

## What it does not do yet

The brief scopes the FactSet carefully, and the scoping is worth stating here because it is easy to get ahead of the implementation.

The FactSet tracks reads and writes at the type-property level, not per entity instance. It can tell you "something reads `Guard.trust`" but not "the condition checks *this specific guard's* trust." That is the correct granularity for static analysis — the compiler does not know which entity instance is involved. Entity-level tracking is a future additive change, not a v1 requirement.

Jump edges represent declared transitions, not executable control flow. The FactSet can tell you that section A declares a jump to section B. It cannot tell you whether that jump is reachable given current state — that is a runtime question. Reachability analysis is a derived query over facts, not a fact itself.

Explain mode — "why is this choice available?" — requires both the FactSet and a runtime. The compiler side can tell you which conditions exist on a choice and which properties they read. The runtime side evaluates those conditions against current state and tells you which passed. The FactSet enables explain mode. It does not deliver it alone.

## The Datalog connection

The design is influenced by systems like Soufflé and Datalog. In those terms, the FactSet corresponds to the extensional database — the base relations. Any future rule system would operate as an intensional layer over this data.

But Urd does not adopt recursive inference, fixpoint evaluation, or rule authoring. Authors do not write logic programs. There is no inference engine in the compiler. The FactSet makes those approaches possible later without depending on them now. It is a bridge, not a commitment.

## Why this is a landmark

Urd started as an experiment in specification-driven AI development. The specifications came first. Then the grammar. Then the schema. Then the compiler. Each layer was specified before it was built, and each layer survived contact with implementation without amendment.

That sequence proved the method works. But it proved it for a constrained problem — parsing, resolving, and emitting. The compiler is a pipeline: structured input in, structured output out. Compilers are unusually amenable to specification because their correctness criteria are unambiguous.

The FactSet is the first piece that is not a pipeline stage. It is infrastructure. It is an internal representation designed not for one consumer but for an open-ended set of them. Static analysis checks that do not exist yet. A runtime that does not exist yet. An LSP that does not exist yet. Visualization tools. AI-assisted authoring tools. Export to external analysis systems.

This is the point where the experiment stops being "can AI build a compiler from specs?" and starts being "can this approach produce a system with a real architectural future?"

The answer is not proven yet. The runtime is not built. The testing framework is not built. The v1 gate has five acceptance criteria and the compiler gate is approaching completion — but the runtime, testing, specification, and boundary gates are still open. But the FactSet is the kind of decision that separates projects that plateau from projects that compound. It is a deliberate investment in making every future capability cheaper to build, because the semantic relationships are already extracted, normalized, and queryable.

Traditional interactive fiction systems — Inform, Ink, Twine — optimize their internal representations for execution. You cannot ask them "which conditions read this property?" or "is this choice reachable given the current write set?" without instrumenting the runtime or walking the full source. Urd's FactSet makes these questions trivial. That is not a feature. It is a category difference.

## What comes next

The brief is scoped as v1 gate work — section D of the [v1 Completion Gate](/documents/v1-completion-gate), requirements F1 through F8. Implementation is estimated at roughly 860 lines of new code: type definitions, extraction logic, the property dependency index, and tests against all five canonical fixtures.

Once the FactSet exists, the first derived diagnostic — property read but never written — becomes a follow-up brief of its own. Then the existing S3, S4, S6, and S8 checks can optionally be refactored from AST traversals to fact queries. Then Wyrd can consume the FactSet for explain mode and cache invalidation. Then an LSP can use it for go-to-definition and find-all-references on properties.

None of those capabilities require redesigning the FactSet. They are queries over it. That is the whole point.

Urd is no longer just a compiler approaching its v1 gate. It is starting to become the kind of system where each new piece makes every other piece more capable. That is not something you can spec into existence. It emerges from the right architectural decisions, made at the right time, for the right reasons.

This was one of them.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
