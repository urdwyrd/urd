---
title: "The World Is Not the Interface"
slug: world-is-not-the-interface
description: Why interactive world systems keep absorbing their own presentation layer, and what happens when you refuse to let them.
date: "2026-02-21"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> An essay on the structural tendency of interactive world systems to absorb their presentation layer, and the architectural refusal that defines Urd.
> Single canonical copy. February 2026.

Every interactive fiction engine eventually becomes a UI framework.

It starts innocently. The world model needs a room description, so you add a `description` field. Then it needs a *different* description when the lights are off, so you add conditional text. Then you need verb synonyms for the parser, so you add grammar hints. Then you need a "fumble" message for failed actions in the dark, so you add failure-mode text. Then someone wants the NPC to react differently depending on whether the player typed "examine cloak" or "look at cloak," so you add input disambiguation to the rule system.

By now, your world model knows what the player typed. It knows what text to display. It knows how to render failure. It has opinions about verbs, about phrasing, about pacing. It has become the thing it was supposed to describe.

This is not a bug in any particular engine. It is a structural tendency. And it is the reason every major interactive world system hits the same walls at scale: brittle integration, untestable narrative logic, engine lock-in, and AI systems that hallucinate the rules because there are no formal rules to follow.

This essay describes an alternative: a system called Urd that models interactive worlds as pure data contracts, a runtime called Wyrd that executes them deterministically, and an [architectural boundary](/documents/architectural-boundaries) that permanently excludes presentation, input parsing, and experience feedback from both. It is an architecture built around a deliberate refusal — and that refusal is the entire point.

## The pattern that keeps repeating

Inform 7 is a brilliant system. It lets authors write interactive fiction in something close to natural English. It also has a world model, a parser, a rendering layer, and a rule system that can respond to the player's exact phrasing. These are deeply intertwined. The world model knows about verbs. The parser knows about rooms. The rules fire on text input. This integration is what makes Inform powerful — and it is also what makes Inform non-portable, difficult to test automatically, and impossible to consume with a non-parser interface.

Ink solves a narrower problem elegantly. It handles branching dialogue with beautiful syntax. But it has no world model. State lives in global variables. The moment you need spatial reasoning, inventory, or NPC behaviour, you fall out of ink into ad hoc tags and custom glue code. Every team writes the same integration from scratch.

articy:draft provides rich content management with custom entity templates. But it is an authoring tool, not a runtime. Its world model is implicit in a database structure, not a formally specified contract. Export is to XML or JSON that requires engine-specific interpretation. And as the Disco Elysium team discovered, it breaks at scale.

These are excellent tools, each solving part of the problem. The part they share is the failure mode: the world model and the experience layer are entangled in ways that make the system fragile, untestable, and locked to a specific interaction paradigm.

## Separating what the world *is* from how it's *experienced*

Urd is a declarative schema system for interactive worlds. It describes entities, their types and properties, their spatial relationships, and the rules governing state transitions. It compiles to a single JSON file — a typed, validated, self-contained data contract that any runtime can consume without custom integration code.

Here is the Monty Hall problem defined in Urd's authoring syntax:

```
types:
  Door [interactable]:
    ~prize: enum(goat, car)
    state: enum(closed, open) = closed
    chosen: bool = false
```

Three doors. Hidden prizes. A host who must open a door with a goat behind it, one the player didn't choose. No probability is specified anywhere. The two-thirds switching advantage emerges from the structure — from the constraints on the `select` block that governs the host's behaviour.

Run it ten thousand times with seeded randomness. The switching advantage converges to 2/3. The probability was never authored. It fell out of the world rules.

This is what a declarative world model makes possible: emergent behaviour from structure, not scripted sequences. And because the world model is pure data, it can be statically analysed, automatically tested, and consumed by any interface — a parser, a choice-based UI, a graphical renderer, or an AI agent — without modification.

## Three layers, one boundary

The architecture has three layers. Each has a single responsibility. They communicate through defined contracts. No layer reaches into another.

**Urd** (the schema and compiler) answers: *what exists and what are the rules?* Entities, types, properties, containment, visibility, conditions, effects, rules, sequences, dialogue structure. Urd is a data contract. It contains no runtime state, no execution logic, no presentation, no interaction model. A compiled `.urd.json` file is inert — a world that *could* be executed, doing nothing on its own.

**Wyrd** (the runtime) answers: *given this world and these actions, what happens next?* It loads the compiled world, manages state, evaluates conditions, applies effects, fires rules, and returns events. It is deterministic: identical world, identical seed, identical actions produces identical results. Wyrd has no opinions about presentation. It receives structured action identifiers and returns state changes. How the player communicates intent, and how state changes are rendered into experience, is not its concern.

**The adapter layer** answers: *how does the player experience this world?* A parser that converts typed text into structured action calls. A choice UI that renders available actions as buttons. A 3D renderer. A voice interface. An AI agent. A test harness. All adapters. All replaceable. All entirely outside Urd's and Wyrd's concern.

The boundary between these layers is enforced by the [Architectural Boundaries](/documents/architectural-boundaries) governance document, which provides a repeatable decision framework. Any proposed feature is evaluated against a [five-question boundary test](/articles/architectural-boundaries): Does it describe what exists? Does it describe a rule about state transitions? Does it describe how state is evaluated? Does it describe how the player perceives or interacts? The first "yes" determines where it belongs. If it's about perception or interaction, it's adapter-layer work, regardless of how useful it would be inside the world model.

This is not aspirational. It's enforceable. The governance document defines [permanent exclusions](/documents/architectural-boundaries) — categories of concern that are architecturally outside the schema and runtime, not deferred to a future version but excluded by design: input models, conditional text rendering, player experience feedback, time and pacing, persistence.

## What the refusal enables

Keeping the world model clean is not an aesthetic preference. It enables specific capabilities that entangled systems cannot provide.

**Deterministic testing.** Because the world advances only on explicit action calls and state changes are the only events, you can run automated playthroughs and assert on exact state at any point. Run the Monty Hall problem ten thousand times and verify the probability distribution. Run the key puzzle and verify the escape sequence. Run the interrogation scene and verify that mood state gates information correctly. No UI. No human input. No non-determinism beyond seeded randomness.

**Static analysis.** Because the world is a typed graph — entities are nodes, containment and exits are edges, properties are node-local state, rules are conditional edge and property transformations — the compiler can perform reachability analysis, dead-end detection, and orphan checking before the runtime ever loads the file. Can the player reach every location? Are there actions whose conditions are contradictory? Are there dialogue sections that exhaust to empty menus? These are graph properties, and they're computable precisely because the world model doesn't contain imperative logic.

**Multi-interface portability.** The same `.urd.json` file is playable through a parser adapter, a choice-based web UI, a graphical game engine, or an AI agent. No modification needed. The world doesn't know or care how the player communicates. This is not "write once, run anywhere" marketing — it's a structural property of keeping input and presentation outside the world model.

**AI-native consumption.** An AI agent interacting with an Urd world can read the entity graph, query available actions, evaluate conditions, and select actions based on typed properties — all without parsing natural language or guessing at untyped state. The world model is a formal contract. The agent doesn't need to hallucinate the rules because the rules are explicitly declared, typed, and validated.

## The graph underneath

A compiled `.urd.json` file is, at its core, a directed graph. Entities, locations, dialogue sections, actions, and sequences are nodes. Containment, exits, entity references, and dialogue jumps are edges. Properties are node-local state. Rules define conditional transformations of edges and properties.

This isn't a metaphor. It's the actual data structure. And recognising it as such opens up capabilities that are difficult to achieve with entangled systems.

If you're familiar with Datalog or systems like Soufflé, the pattern will look familiar: base facts (the world definition), rules (conditional transitions), and derived state (reachable locations, visible entities, transitive containment). The critical difference is that Urd does not perform open-ended inference. It performs controlled, event-driven transitions. No recursion. No fixpoint evaluation. No implicit derivation exposed to authors. The graph transforms deterministically in response to explicit actions, and that determinism is what makes it testable, analysable, and portable.

The graph model also makes visualisation trivial. A world's spatial topology is a subgraph you can draw. An action's preconditions and effects are edge queries. A rule's trigger conditions and state mutations are graph transformations. Debugging becomes: show me which edges changed, which conditions blocked, which properties mutated. This is not hypothetical tooling — it falls directly out of the data model.

## What Urd is not

Urd is not a scripting language. There is no control flow, no loops, no function definitions. This is deliberate. The moment you add Turing-complete logic to a world definition, you lose the ability to statically analyse it — you hit the halting problem. Urd is *deliberately less than a programming language* to preserve the properties that matter.

Urd is not a database-driven IF system in the historical sense. Past systems that tried to model worlds as data often failed because they mixed data and execution, had implicit control flow, leaked presentation into the model, and lacked deterministic behaviour. Urd avoids all of these by construction. The [Architectural Boundaries](/documents/architectural-boundaries) document exists specifically to prevent the drift that killed earlier approaches.

Urd is not an engine. It doesn't render anything. It doesn't handle physics, audio, or asset management. Unity, Godot, and Unreal do that. Urd is upstream of engines — it defines the world contract that engines consume.

And Urd is not finished. The specification suite is complete and internally consistent. The compiler is built — five phases (PARSE, IMPORT, LINK, VALIDATE, EMIT) implemented in Rust with 480 tests and a 100% pass rate. Four canonical fixtures compile to valid `.urd.json`. The runtime is next. The architecture is designed; the implementation is underway.

## Who this is for

**If you build narrative games** and spend a third of your development time on glue code between your authoring tool and your engine, Urd replaces that glue with a standard contract. Write your world in Schema Markdown — a syntax as clean as ink for dialogue, but schema-native for everything else. Compile it. Hand the JSON to your engine. No custom integration code.

**If you build interactive fiction** and want your worlds to be testable, analysable, and portable beyond a single platform, Urd gives you a typed world model with automated testing and static analysis. The parser is an adapter that sits on top. Your world logic doesn't change when your interface does.

**If you build AI agents** that interact with environments and need a formally specified, deterministic world model that your agent can consume without hallucinating the rules, Urd provides exactly that: a typed graph with declared conditions, effects, and constraints. Every valid action is enumerable. Every state transition is deterministic. The agent reasons on structure, not on parsed text.

**If you study formal systems** and recognise the value of constrained graph transformation systems with deterministic evaluation — and you're frustrated that interactive world modelling hasn't adopted these properties despite decades of work in related fields — Urd is an attempt to apply them.

## The thesis, stated plainly

Interactive world systems fail at scale because they absorb their presentation layer. The world model starts knowing about verbs, text, rendering, and player input. Once it does, it cannot be tested independently, consumed by multiple interfaces, analysed statically, or reasoned about formally.

The fix is architectural, not incremental. Separate what the world *is* from how it's *experienced*. Make the boundary explicit, enforceable, and permanent. Accept the tradeoffs: less expressive for tightly coupled prose, less suited to parser-specific nuance, requires structured thinking rather than freeform scripting.

What you get in return is a world model that is deterministic, testable, portable, AI-consumable, and statically analysable. A model where probability emerges from structure. Where automated testing catches narrative bugs that no human playthrough would find. Where the same world file runs through a parser, a choice UI, a graphical engine, and an AI agent without modification.

Define in Urd. Test in Wyrd. Ship anywhere.

The [Architectural Boundaries](/documents/architectural-boundaries) document is published at [urd.dev](https://urd.dev). The full specification suite — schema, syntax, runtime contract, test strategy, and governance — is open and available. The compiler is shipping. The runtime is next.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
