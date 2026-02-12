---
title: "Urd + Wyrd Product Vision v2.0"
slug: "product-vision"
description: "The strategic vision for Urd and Wyrd — market analysis, the fragmentation tax in narrative game development, product architecture, revenue model, and development roadmap."
category: "strategy"
format: "Product Strategy"
date: "2026-02"
status: "v2.0 complete"
order: 1
tags:
  - vision
  - strategy
  - product
  - roadmap
  - market
details:
  - "Seven pain points from developer forums and postmortems"
  - "Four-component pipeline: compiler, runtime, testing, tooling"
  - "Open core revenue model with hosted services"
  - "AI-native design philosophy throughout"
---

> **Document status: INFORMATIVE**
> Product strategy, market positioning, and roadmap. Provides context for the normative specifications but does not itself define implementable contracts.
> Single canonical copy. February 2026 draft.

# URD + WYRD

## Product Vision

**A declarative schema system for interactive worlds**

*Define in Urd. Test in Wyrd. Ship anywhere.*

urd.dev · February 2026

## Executive Summary

Urd is an open, structured world definition format. A declarative schema that describes interactive worlds as typed, validated, engine agnostic data. Wyrd is the reference runtime that executes those worlds in the browser. Together, they replace the fragmented pipeline of narrative authoring tools, custom glue code, and engine specific integrations with a single, composable system.

**The thesis:** narrative game development suffers from a fragmentation tax. Every team cobbles together ink or Yarn Spinner for dialogue, articy:draft or spreadsheets for world data, custom C# or GDScript for integration, and ad hoc playtesting for QA. 30 to 50 percent of development effort goes to glue code that is never reusable.

**The solution:** a pipeline, not a monolith. Writers author in Schema Markdown, a prose friendly syntax as clean as ink for pure dialogue but schema native by design. The compiler validates and produces a single `.urd.json` contract file. The runtime executes it. The testing framework validates it. Engine integrations consume it. No custom glue code. No integration tax.

> Here's the Monty Hall problem defined in Urd. Notice we never specified a probability. The 2/3 switching advantage emerges from the world rules alone.

That's the pitch. Declarative world description where emergent behaviour falls out of structure.

The initial design phase is complete but this project is early and in motion. Designs will be revised. Assumptions will be proven wrong. Entire approaches may be scrapped and rebuilt from scratch. Like any good roguelike, death is not a bug — it's a mechanic.

**What comes next:** development journal, compiler, runtime, testing framework, developer tooling, and the Urd IDE, a structured authoring environment with separate writer and engineer modes.


## The Problem We Solve

Seven pain points, documented from developer forums, postmortems, GDC talks, and published interviews. Every claim is sourced. The full evidence base is in the Pain Points Report.

### Three Critical Pain Points

#### The Integration Tax

Every narrative game team builds bespoke glue code between their authoring tool and their engine. This code is fragile, unreusable, and consumes 30 to 50 percent of development effort. There is no standard interface between "narrative data" and "game state."

> *When I start at a studio, I almost always find myself building or rebuilding a pipeline for game dialogue.* — Ian Thomas, senior narrative developer

The Urd world schema is that standard interface. A compiled `.urd.json` file is directly consumable by any engine with a reference runtime.

#### The Narrative World Divide

Current tools treat "narrative" as synonymous with "dialogue." Games need characters with attributes, locations with properties, items with relationships, quests with dependencies. No tool provides a unified model. Urd's schema natively connects narrative content with world objects, spatial data, and simulation state through a single containment model.

#### The Writer Programmer Wall

Writers and programmers operate in different tools, formats, and mental models. The handoff between them is where quality dies.

> *Once, I was contracted to write all the sidequests for a game in about two weeks — 60-some quests — where I had to do it all in Excel. Yes, it was a nightmare.* — Doc Burford, game writer

Schema Markdown lets writers author structured world data without knowing they're doing it. The same file is readable prose to a writer and typed schema to an engineer.

### Four Supporting Pain Points

- **The Scale Wall.** Disco Elysium literally broke articy:draft. Helen Hindpere described how it "froze completely" and the articy team had no solution. The Urd schema is designed for hundreds of thousands of nodes.
- **Collaboration.** Most tools are single user. Schema Markdown is text based, Git friendly, and multi file by design.
- **Testing.** No automated narrative validation exists. Yarn Spinner's Story Solver generated immediate demand at GCAP 2024, validating massive unmet need.
- **Engine Lock In.** Yarn Spinner is Unity first. Pixel Crushers is Unity only. Dialogic is Godot only. The Urd schema and runtime are engine agnostic by architecture.

## The Solution: Urd + Wyrd

Urd is a pipeline, not a monolith. Content flows through independent components that communicate through two interchange formats: Schema Markdown source files and compiled JSON.

```
.urd.md files  →  Compiler  →  .urd.json  →  Runtime + Testing + Engine Integrations
```

Five components. One critical boundary. The `.urd.json` file is the contract between authoring and execution.

### Component 1: The Compiler

Transforms Schema Markdown source into a single, validated, self contained JSON world file. Five phases: Parse → Import → Link → Validate → Emit.

- **Error recovery.** Collects as many errors as possible in a single run.
- **Actionable diagnostics.** Not "parse error on line 47" but "`@guard.trust` is not a property on type Guard. Did you mean `@guard.mood`?"
- **Incremental compilation ready.** File dependency graph, stable entity IDs, cache invalidation boundaries.

### Component 2: Wyrd, The Reference Runtime

Wyrd loads a compiled `.urd.json` file and executes it in the browser. The name comes from the Old English cognate of Urðr. Urd defines what the world *is*. Wyrd determines what *happens*.

> Wyrd is to Urd what SQLite is to SQL. Embedded, zero config, good enough for development and demos, not pretending to be production infrastructure.

- **Core engine.** World state, condition evaluation, effect application, action resolution, dialogue management. Immutable state transitions. Event sourcing. Seeded randomness.
- **Presentation layer.** Browser based text and choice UI. Intentionally plain: verifiable, not beautiful.
- **Extension host (future).** Sandboxed lambda functions for imperative logic. Strict contract: read only state in, effect list out.

**Why the browser:** a visitor clicks "Play" on urd.dev and the world runs. No download, no install. That's the conversion moment.

**Canonical behaviour:** any ambiguity in the spec is resolved by what Wyrd does. Engine integrations test against it.

### Component 3: The Testing Framework

The early product wedge, the tool studios adopt first.

- **Schema validation.** Structural correctness of compiled JSON.
- **Reachability analysis.** Dead end sections, contradictory conditions, orphaned locations.
- **Playthrough simulation.** Run 10,000 Monty Hall games. Verify 2/3 switching advantage.
- **Visibility auditing.** Hidden state never leaked through any action or event sequence.
- **Coverage reporting.** Which actions, conditions, dialogue sections were exercised?

### Component 4: Developer Tooling (LSP)

Language Server Protocol wrapping the compiler in watch mode. Live diagnostics, autocomplete, go to definition in VS Code, Vim, Emacs, or any LSP capable editor. Target: 50 files, 500 entities in under 200ms.

### Component 5: The Urd IDE

A structured authoring environment with two modes.

**Writer Mode.** Prose first. The `.urd.md` file is front and centre. Schema invisible unless needed. Inline entity completions on `@`, conversation flow graph, Wyrd play panel. No JSON, no type definitions, no terminal.

**Engineer Mode.** Full schema visibility. Type browser, cross reference graph, compiled JSON inspector, test runner integration.

Both modes edit the same files on disk. Git, CI/CD, and command line tooling all work without the IDE.

## The Schema: What Urd Describes

The Urd World Schema answers: what is the minimum structured representation needed to describe an interactive world that a runtime can execute without custom glue code?

### Design Principles

- **Declarative, not imperative.** Describes what the world is, not what it does. Emergent behaviour from structure.
- **Containment as the universal spatial primitive.** A room holds a sword. A chest holds a sword. A player's pockets hold a sword. One mechanism: `entity.container == other_entity`.
- **Visibility as a first class concept.** Properties are visible, hidden, owner only, or conditionally revealed. Information asymmetry is explicit.
- **AI native by design.** Every element typed and unambiguous. A formal contract, not documentation.
- **Extensible without breakage.** New capabilities are new blocks. Existing files remain valid.

### Schema Structure

Eight top level blocks. All optional except `world`:

| Block | Purpose |
|-------|---------|
| `world` | Required. Metadata: name, version, starting location, entry sequence. |
| `types` | Entity type definitions. Property schemas with types, defaults, constraints, visibility. |
| `entities` | Instances of defined types. Unique IDs, type reference, property overrides. |
| `locations` | Spatial containers with exits and connections. Syntactic sugar over container entities. |
| `rules` | Behavioural constraints. Triggers, conditions, select from set, effects. |
| `actions` | Player performable interactions. Target, prerequisites, effects. |
| `sequences` | Ordered phase flows. Game show rounds, tutorial steps, ritual stages. |
| `dialogue` | Dialogue sections, choices, jumps, and `on_exhausted` fallthrough content. |

### The Containment Model

Every entity exists inside exactly one container. Moving, picking up, dropping, and storing are the same operation. `move: key, to: player` is "pick up." `key.container == player` is "does the player have the key?" One spatial primitive replaces three separate systems.

### The .urd.json Contract

Self contained (all imports resolved), deterministic (same source → byte identical output), versioned, human inspectable, and extensible. This is the USB C for narrative data.

## Schema Markdown: What Writers Write

As clean as ink for pure dialogue. Schema native by design.

### Seven Symbols

A writer's entire vocabulary:

| Symbol | Name | What It Does |
|--------|------|-------------|
| `@` | Entity reference | References characters, objects, locations |
| `?` | Condition | Gates content on world state |
| `>` | Effect | Changes the world |
| `*` | One shot choice | Disappears after selection |
| `+` | Sticky choice | Stays available on revisit |
| `->` | Jump / exit | Navigates to a section or location |
| `//` | Comment | Ignored by the compiler |

### Writer Promises

Three lines of frontmatter. One import. Everything else is narrative. Writers never touch type definitions, JSON, or rule blocks. If the syntax forces them to, the tooling has failed.

### The Hybrid Dialogue Model

Nested dialogue, the highest risk syntax problem, was solved with a hybrid approach: indentation for shallow branching (one to two levels), labelled sections for deep structure and loops.

- Indentation handles the common case.
- Sections (`== name`) handle hubs, deep branches, and loops.
- Maximum depth: two levels. Compiler warns at three, rejects at four.

Stress tested against a multi topic interrogation with conditional branches, a bribe using containment checks, NPC mood mutations, and a state dependent farewell. Production complexity, clean syntax.

### Multi File Architecture

One shared schema file (engineer owned), one file per location or scene (writer owned). Each writer file starts with `import: ./world.urd.md`. Imports are explicit and non transitive. Cross location movement uses exits. Git friendly, scales indefinitely.

## What Urd Proves: The Test Strategy

Four progressive tests, each proving capabilities the previous couldn't.
### Test 1: Dialogue Systems

**Proves**: sticky and one shot choices, exhaustion, hub and spoke conversation, conditional branches, NPC mood mutations.

The foundational test for the hybrid dialogue model. Validates that indentation handles shallow branching, labelled sections handle deep structure, and the compiler enforces depth limits correctly.

### Test 2: The Monty Hall Problem

**Proves:** hidden state, NPC constraints, decision points, emergent probability.

Switching wins 2/3 of the time, but that probability is never specified. It emerges from the structure. Run 10,000 games; the math falls out. The mic drop introduction to declarative world description.

### Test 3: The Two Room Key Puzzle

**Proves:** spatial navigation, inventory, persistent state, conditional NPC dialogue, object transfer.

Everything Monty Hall doesn't test. Two rooms, a guard, a key, a locked door. Containment transfers, condition checks, persistent state changes.

### Test 4: The Interrogation

**Proves:** dialogue at production scale — multi-topic hub, conditional sub-branches, containment in dialogue, state-dependent farewell.

This is the stress test that validates the hybrid nesting model under real-world complexity.

### Future: Connected Variant

A forthcoming test case will demonstrate **cross-system composability** — proving that independently designed mechanics (e.g., the key puzzle and Monty Hall) can coexist and interact within a single world file.

This progression moves from "the schema can describe a conversation" to "the schema can describe a game" to "the schema can describe a world" to "the schema can handle production-scale dialogue." The future Connected Variant will complete the arc: "the schema can compose independent systems into a coherent world."

## The AI Multiplier Effect

A typed world schema makes AI coding assistants dramatically more capable.

### AI Is Currently Coding Blind

Ink's ad hoc tags have no schema. The AI guesses what they mean. Glue code differs per project. State is untyped. The AI generates syntactically correct code but cannot reason about narrative correctness because there is nothing formal to reason against.

### What Changes With a Typed Schema

The AI knows entity types, valid conditions, cross references, and output format. Generating integration code becomes deterministic. This flips the integration tax: an AI assistant can generate most glue code reliably because the schema is the specification.

### Three Additional Layers

- **AI augmented narrative QA.** Graph traversal + LLM narrative judgment. Detecting quest paths that miss critical NPC introductions.
- **Schema constrained generation.** AI generating into a typed schema cannot produce structurally invalid output. Guardrails make AI content reviewable rather than rewritable.
- **Intelligent authoring assistance.** "This conversation references `barkeep.trust` but never modifies it. Did you mean to add a state mutation?"

**Strategic implication:** the first narrative infrastructure designed to be both human writable and machine readable. AI coding assistants become a distribution channel for schema adoption.

## Key Strategic Decisions

1. **Schema native syntax, not importer only.** Build Schema Markdown that compiles directly to the schema. The ink importer exists as a migration onramp, not the primary path.
2. **Schema informed syntax co design.** Both specs developed in parallel, each informing the other. The process was co design, producing a tight result.
3. **Multi file, convention based architecture.** Projects are directory trees. Convention, not enforcement.
4. **The IDE is a product.** Two mode IDE (writer + engineer) as a core product surface.
5. **Open core business model.** Open format (Apache 2.0), open runtime (MIT), commercial tooling layer.
6. **Testing as a first class product surface.** Automated narrative validation ships before the visual editor, providing standalone value.
7. **AI native by design.** Schema, syntax, and compiled output designed for machine readability from day one.

## Phased Approach

All phases produce v1 schema output. Phases are delivery milestones, not schema versions.

### Phase 0: Initial Design ✓ Mostly Complete

Delivered: World Framework Research, Pain Points Report, Schema Specification, Schema Markdown Syntax, Nested Dialogue Design Exploration, Architecture & Technical Design, Wyrd Reference Runtime specification, Test Case Strategy, and this Product Vision.

Validation gate passed: all documents cross reference correctly. No contradictions. The design is coherent.

### Phase 1: Developer Journal
Milestone: Automatically generated developer journal (**[urd.dev](https://urd.dev)**)

### Phase 2: Compiler + Core Runtime

Milestone: Run Monty Hall 10,000 times. Switching converges to 2/3. Run Key Puzzle: pick up key, unlock door, reach corridor.

### Phase 3: Dialogue + Testing

Milestone: Tavern scene plays with sticky/one shot choices working correctly. Testing framework verifies Monty Hall probability and Key Puzzle reachability.

### Phase 4: Browser Presentation + Developer Tooling

Milestone: Monty Hall and Key Puzzle playable on urd.dev. Writer types `@` in VS Code and sees available entities.

### Phase 5: IDE + Ink Importer

Milestone: Writer and engineer IDE modes. Ink to Schema Markdown converter. Community release.

### Phase 6: To be defined

## Target Users

### Primary: Narrative Heavy Indie Studios

2 to 20 person teams building narrative driven games. Currently using ink, Yarn Spinner, or articy:draft alongside Unity, Godot, or Unreal. Examples: Disco Elysium, Sable, Firewatch, Hades, Citizen Sleeper, Fallen London.

**Roles Within the Team**

- Writers see Schema Markdown and IDE writer mode.
- Narrative designers see conversation graphs and test coverage.
- Engineers see schema spec, compiled JSON, runtime API.
- Producers see dashboards and CI results.

### Secondary Markets

- Interactive fiction creators. High engagement early adopters.
- Education/training developers. Graduated complexity is a natural fit.
- MUD/virtual world community. Shares architectural DNA.

## Competitive Positioning

No existing tool occupies Urd's layer of the stack.

| Tool | Strength | Gap Urd Fills |
|------|----------|---------------|
| ink | Elegant dialogue syntax, wide adoption | No world model, no types, no spatial awareness, no testing |
| Yarn Spinner | Unity integration, community | Unity coupled, no entity system, limited world modeling |
| articy:draft | Professional content management | Proprietary, no runtime, broke at scale, Windows only |
| Arcweave | Real time collaboration, cloud | No formal schema, limited engine integration |
| Evennia | Sophisticated MUD world model | Python specific, text only, no visual tooling |
| Twine | Accessible, web based | No world model, single board, breaks at scale |

### The Strategic Moat

Once the schema becomes the standard interchange format, switching cost exceeds adoption cost. Same dynamic as HTML, SQL, OpenAPI. Format is free; tooling captures value.

## Business Model

### Open Layer

World schema specification (Apache 2.0) and reference runtime (MIT). Free and open.

### Commercial Layer

- **Urd IDE.** Free for individuals, team licensing for studios.
- **Collaboration.** Real time editing, conflict resolution, review workflows.
- **Hosted testing/CI.** Cloud narrative validation, coverage dashboards.
- **AI features.** Schema constrained generation, authoring assistance, narrative QA.
- **Support & consulting.** Enterprise licensing, dedicated integration support.
- **Marketplace.** Templates, schema extensions, integration packages.

## Strategic Horizon

### Ring 1: Narrative Game Infrastructure (Now)

Eliminate the integration tax for indie and mid tier studios. Every game that ships on the schema proves the format works.

### Ring 2: Simulation, Education & Virtual Worlds (Medium Term)

The entity relationship condition model extends naturally to educational simulations, training environments, and virtual world platforms.

### Ring 3: AI Realised Worlds (Long Horizon)

If AI generates and renders interactive worlds in real time, the bottleneck is a structured description of what the world should contain. The relationship to AI realised worlds is analogous to what HTML was to the early web: declarative description consumed by an increasingly capable runtime.

Design constraint: we don't build for Ring 3 now, but we must not foreclose it. The schema is extensible without breakage, AI native rather than AI adapted, and separates intent from realisation.