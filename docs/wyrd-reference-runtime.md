---
title: "Wyrd Reference Runtime"
slug: "wyrd-reference-runtime"
description: "The browser-native engine for executing Urd worlds. Loads compiled JSON, executes the world model, and presents interactive text and choice interfaces. Zero install, zero config."
category: "runtime"
format: "Runtime Specification"
date: "2026-02"
status: "v0.1 complete"
order: 1
tags:
  - runtime
  - wyrd
  - browser
  - execution
  - api
details:
  - "Browser-native execution — zero install, zero config"
  - "Immutable world state with copy-on-write snapshots"
  - "Event sourcing for undo, replay, and deterministic testing"
  - "Embeddable in the Urd IDE as a web view"
---

> **Document status: NORMATIVE**
> Defines the Wyrd reference runtime architecture, API surface, deployment contexts, and build passes. This is the authoritative specification for runtime implementers.
> Single canonical copy. February 2026 draft.

# WYRD

## Reference Runtime

*A browser native engine for executing Urd worlds*

*The play button for everything the schema describes*

urd.dev

February 2026

---

## What Wyrd Is

**Wyrd** is the reference runtime for the Urd world schema. It loads a compiled `.urd.json` file, executes the world model, and presents it as an interactive text and choice interface in the browser. No install, no game engine, no configuration.

The name comes from the Old English cognate of *Urðr*, the Norse concept of fate and destiny. **Urd** defines what the world *is*. **Wyrd** determines what *happens*. The schema is declarative; the runtime is imperative. Together they are the complete authoring to execution pipeline.

Wyrd is not a game engine. It does not render 3D environments, handle physics, manage audio, or deploy to platforms. It is the smallest thing that can prove an Urd world works.

**The elevator pitch:** Wyrd is to Urd what SQLite is to SQL. Embedded, zero config, good enough for development and demos, not pretending to be production infrastructure. Define your world in Urd. Test it in Wyrd. Ship it in Unity.

## Why It Runs in the Browser

Every other option creates a barrier between someone hearing about Urd and experiencing it. The browser removes that barrier entirely.

### For the urd.world Product Face

The Monty Hall demo, the key puzzle, the tavern scene: all of these need to be playable on urd.world with a single click. A visitor reads the pitch, clicks "Play," and the world runs. No download, no install, no account. That's the conversion moment. If it requires a desktop app or a game engine, the conversion moment never happens.

### For Writers in the IDE

The Urd IDE's writer mode needs a play button. When a writer saves a scene and wants to test it, Wyrd runs the compiled output in an embedded panel. The writer interacts with the world, sees dialogue, makes choices, watches state change. This is the inner loop: write → compile → play → edit. Wyrd running in the browser means the IDE can embed it as a web view with no native dependencies.

### For the Testing Framework

Automated testing needs to execute worlds programmatically. Wyrd's JavaScript API is the execution engine underneath the testing framework: load a world, simulate player actions, assert on world state. Because it's JavaScript, tests run in CI (Node.js), in the IDE (embedded browser), and on urd.world (interactive demos). One runtime, three contexts.

### For Portability

The browser is the only platform that runs everywhere without installation: desktop, tablet, phone, embedded in documentation, embedded in slide decks, embedded in LMS systems. Wyrd being browser native means Urd worlds can appear anywhere a URL works.

**What about performance?** Urd worlds are not computationally expensive. They're state machines with condition evaluation and string rendering. A modern browser can evaluate thousands of conditions per frame. The performance ceiling for Wyrd is not JavaScript speed; it's the rendering of the presentation layer, which for a text and choice interface is trivial.

## Architecture

Wyrd has three layers. Each is independent and replaceable.

| Layer | Responsibility | Replaceability |
|-------|---------------|----------------|
| Core Engine | World state, condition evaluation, effect application, action resolution, dialogue management, event sourcing. | Never replaced. This is the reference implementation of the Urd runtime contract. |
| Presentation | Renders events as visible interface: text, choices, location descriptions, inventory, NPC dialogue. | Easily replaced. The default is a text and choice web UI. A Unity integration replaces this with 3D rendering. |
| Extension Host | Executes lambda functions for custom logic that goes beyond declarative rules. | Optional. Worlds without lambdas don't use it. Sandboxed. |

### Core Engine

The core engine is the single authoritative implementation of the Urd runtime contract. It is responsible for:

- **Loading and validating.** Parse `.urd.json`, check the urd version field, reject unsupported versions with a clear error.
- **World state management.** Maintain the current value of every entity property, every entity's container, every section's visited/exhausted status, and the current sequence phase.
- **Condition evaluation.** Given the current world state, evaluate any condition expression to true or false. Includes property comparisons, containment checks, and exhaustion checks (`? <section_name>.exhausted`). Exhaustion is runtime evaluated and not cached: the runtime checks all choices in the named section and returns true if none are currently available.
- **Effect application.** Apply effects (set, move, reveal, destroy, spawn) atomically and update world state.
- **Action resolution.** Compute available actions based on entity state, conditions, and rules. Return a list to the presentation layer.
- **Dialogue management.** Track which section is active, which choices have been consumed (one shot) or visited (sticky), when sections are exhausted, and where jumps lead.
- **Event sourcing.** Every action returns a list of events describing what happened. The presentation layer uses these events; the engine never assumes a particular presentation.

**The engine has no opinions about rendering.** It produces events. A text runner prints them. A Unity integration plays animations. A test harness asserts on them. The engine's public surface is: load world, get available actions, perform action, get events.

### Presentation Layer

The default presentation is a browser based text and choice interface. It consumes events from the core engine and renders them as:

- **Location descriptions.** Prose text for the current location, updated when the player moves.
- **NPC dialogue.** Attributed speech with speaker identification. Displayed as a conversation thread.
- **Choice menus.** Available actions presented as clickable options. Gated choices are hidden. Consumed one shot choices are removed.
- **State indicators.** Inventory contents, visited locations, and key entity states shown in a side panel.
- **Narration and stage directions.** Non dialogue text rendered as descriptive prose.

The presentation layer is intentionally plain. It is not trying to be a finished game UI. It is trying to be **readable, immediate, and accurate**, so that writers can verify their scenes and visitors can experience the world model without visual distraction.

### Extension Host: Lambda Functions

**Future schema capability.** Lambda functions are not part of the v1 schema. They are described here because Wyrd's architecture must accommodate them from the start. Retrofitting an extension host into a runtime that wasn't designed for one is expensive. The contracts and sandboxing rules below are design constraints for a future schema version, not current implementation targets.

Some game logic is awkward to express declaratively. Pathfinding, economic calculations, weighted random selection, procedural generation: these are naturally imperative. Lambda functions provide an escape hatch without breaking the declarative model.

#### The Contract

A lambda function is declared in the schema and executed by the extension host. The contract is strict:

- **Input:** A read only snapshot of world state. The lambda receives entity properties, containment, and any parameters declared in the schema. It cannot access the DOM, the network, or anything outside the world model.
- **Output:** A list of effects in the same format as authored `>` lines: set, move, reveal, destroy, spawn. The runtime applies them exactly as it would apply any other effects.
- **No side effects.** A lambda cannot mutate world state directly. It returns effects; the engine applies them. This preserves the event sourcing model and keeps the world inspectable and testable.
- **Deterministic when possible.** Lambdas that use randomness must accept a seed parameter so that tests can reproduce results. The engine provides the seed; the lambda uses it.

#### How They're Declared

```
---
import: ./world.urd.md

lambda calculate_trade_value:
  receives: [@merchant, @player, @offered_item]
  returns: effects
  source: ./lambdas/trade.js
---
```

The lambda block declares the function's name, the entities it receives, and its source file. The writer never sees this. Rules or actions can invoke the lambda: `> apply calculate_trade_value(@merchant, @player, @sword)`. The runtime passes the entity snapshots to the function and applies the returned effects.

#### Sandboxing

Lambdas run in a sandboxed JavaScript environment with no access to the DOM, network, filesystem, or global state. In the browser, this is a Web Worker. In Node.js (CI/testing), this is a vm context. The sandbox enforces a time limit: a lambda that runs for more than 100ms is terminated with an error event.

**Lambdas are optional and future.** Every Urd world must work with purely authored content and declarative rules. Lambdas extend what's expressible; they don't replace the core model. The extension host is designed now so that when a future schema version introduces lambda support, Wyrd's architecture is ready without a rewrite.

## The Public API

Wyrd exposes a minimal JavaScript API. This is the surface that the presentation layer, the testing framework, the IDE's play panel, and engine integrations all consume.

```javascript
// Load a world
const world = await Wyrd.load(urdJsonUrl);

// Inspect state
world.getState()              // Full world state snapshot
world.getEntity(id)           // Single entity with current properties
world.getLocation()           // Current player location
world.getInventory()          // Entities contained by the player

// Interact
world.getAvailableActions()   // What the player can do right now
world.perform(actionId)       // Do it; returns list of events

// Dialogue
world.getActiveSection()      // Current dialogue section, if any
world.getChoices()            // Available dialogue choices
world.choose(choiceId)        // Select a choice; returns events

// Observe
world.on('event', callback)   // Subscribe to events
world.getHistory()            // Full event log

// Testing
world.reset()                 // Return to initial state
world.seed(n)                 // Set random seed for reproducibility
world.simulate(actions)       // Run a sequence, return final state
```

The API is intentionally small. Everything a consumer needs is: load, inspect, act, observe. The `simulate` method is the testing framework's primary entry point. It runs a sequence of actions and returns the resulting state without rendering anything.

**Event types.** Every `perform` or `choose` call returns an array of events. Event types include: `dialogue` (NPC speech), `narration` (descriptive text), `move` (entity moved), `reveal` (property revealed), `set` (property changed), `destroy` (entity removed), `spawn` (entity created), `section_enter` (dialogue section activated), `exhausted` (section exhausted). The presentation layer maps these to UI updates; the test harness asserts on them.

## What Wyrd Is Not

Drawing the boundaries clearly:

- **Not a game engine.** No rendering pipeline, no physics, no audio, no asset management. Unity and Godot do that. Wyrd is upstream of engines.
- **Not the final player experience.** The text and choice interface is a development and demonstration tool. The final player experience is built by a game team using the compiled schema in their engine of choice.
- **Not a CMS.** Wyrd executes worlds; it doesn't author them. The Urd IDE is the authoring environment. Wyrd is the play button inside that environment.
- **Not a server.** Wyrd runs entirely client side. There is no backend, no database, no authentication. A world file is a static JSON artifact served from a CDN.
- **Not required.** A team can build an Urd runtime in C# for Unity, in GDScript for Godot, or in Rust for a custom engine. Wyrd is the reference implementation, not the only implementation. But it is the *canonical* one: any behavioural ambiguity in the spec is resolved by what Wyrd does.

## Deployment Contexts

Wyrd runs in three contexts. The core engine is identical in all three; only the presentation layer changes.

| Context | How Wyrd Is Used | Presentation |
|---------|-----------------|--------------|
| urd.world | Interactive demos. Visitor clicks Play, the world loads from a URL, and they interact with it in the browser. | Full text and choice web UI with location panel, dialogue thread, choice buttons, and state inspector. |
| Urd IDE | Embedded play panel. Writer saves a file, the compiler runs, and the compiled output is loaded into Wyrd in a side panel. The writer interacts with their scene without leaving the editor. | Compact panel UI. Same core engine. State inspector shows entity properties updating in real time. |
| CI / Testing | Headless execution. The testing framework loads a world into Wyrd via Node.js, runs simulated playthroughs, and asserts on state and events. No browser, no UI. | No presentation layer. `Wyrd.simulate()` runs actions, returns events and final state. Monte Carlo mode runs thousands of playthroughs with seeded randomness. |

## Relationship to the Urd Ecosystem

Where Wyrd fits relative to every other component:

| Component | What It Does | Wyrd's Relationship |
|-----------|-------------|---------------------|
| Urd Schema Spec | Defines the data contract (types, entities, properties, containment, visibility). | Wyrd implements the runtime semantics the schema describes. |
| Schema Markdown | The writer facing syntax that compiles to the schema. | Wyrd never sees `.urd.md` files. It only consumes compiled `.urd.json`. |
| Compiler | Transforms Schema Markdown into compiled JSON. | Wyrd loads the compiler's output. The compiler produces; Wyrd consumes. |
| Testing Framework | Static analysis + playthrough simulation + coverage. | Wyrd is the execution engine under the testing framework. Tests call Wyrd's API. |
| Urd IDE | Writer and engineer authoring environment. | The IDE embeds Wyrd as the play panel. Same API, embedded web view. |
| Unity / Godot Plugins | Native engine integrations for shipping games. | These are alternative runtimes. They implement the same contract Wyrd does. Wyrd is the reference they test against. |
| AI Integration | Schema constrained generation, intelligent NPCs, narrative QA. | AI services can call Wyrd's API to simulate worlds, or generate content that Wyrd validates by execution. |

## Implementation Approach

### Language

**TypeScript**, compiled to JavaScript for browser execution and compatible with Node.js for CI. TypeScript provides type safety during development; the shipped artefact is plain JavaScript with no runtime dependencies.

### Package Structure

- `@urd/wyrd-core`: The engine. World state, condition evaluation, effect application, action resolution, dialogue management. Zero dependencies. Runs in browser and Node.js.
- `@urd/wyrd-web`: The browser presentation layer. Consumes events from core, renders the text and choice UI. Depends on core.
- `@urd/wyrd-test`: Testing utilities. Simulation, assertion helpers, coverage reporting. Depends on core. Used in CI.

### Build Priorities

Wyrd is built in three passes, each producing something usable:

- **Pass 1: Core + Monty Hall.** The engine loads a `.urd.json` file, manages world state, evaluates conditions, applies effects, resolves actions, and returns events. Validated by running the Monty Hall problem 10,000 times and verifying the 2/3 switching advantage. No dialogue, no lambdas.
- **Pass 2: Core + Key Puzzle + Presentation.** Adds spatial navigation (movement between locations), inventory (containment transfer), and the browser presentation layer. Validated by playing the Two Room Key Puzzle interactively in the browser.
- **Pass 3: Dialogue + Testing.** Adds dialogue management (sections, jumps, sticky/one shot, exhaustion), the testing framework integration, and the compact IDE panel mode. Validated by running the tavern scene and the interrogation stress test.

**Lambda functions are Pass 4.** The extension host is built after the core engine is stable and tested. Lambdas extend the engine; they should not be designed under the pressure of getting the fundamentals right.

### The Canonical Behaviour Rule

**Any behavioural ambiguity in the Urd specification is resolved by what Wyrd does.** If the spec says one thing and Wyrd does another, it's a bug in one of them, and the resolution process is: update the spec if Wyrd's behaviour is intentional, or update Wyrd if the spec's intent is clear. The two must always agree. This is what makes Wyrd a *reference* runtime, not just an implementation.

## What This Enables

With Wyrd in place, the Urd ecosystem gains capabilities that don't exist without a reference runtime:

- **Instant demos.** Anyone can play an Urd world by visiting a URL. The Monty Hall problem on urd.world runs live in the browser. No install, no explanation, no trust required.
- **Writer inner loop.** Write → save → play, in under two seconds. The IDE compiles, Wyrd loads the output, the writer interacts with the scene. This is the workflow that makes Schema Markdown usable for non engineers.
- **Automated testing.** Monte Carlo playthroughs, reachability analysis, and coverage reporting all run against the real runtime, not a simulation of it. Tests and production use the same engine.
- **Spec validation.** The specification is only as good as its implementation. Wyrd is the proof that the schema works: not in theory, but in executable code. Every design decision in the schema spec is tested by Wyrd's behaviour.
- **Engine parity testing.** When a team builds a Unity or Godot runtime, they test it against Wyrd. Same world file, same actions, same expected events. Wyrd is the oracle.
- **The commercial pitch.** Define your world in Urd. Test it in Wyrd. Ship it in Unity. That's a three sentence pipeline that a product person can explain to a stakeholder in a hallway.

**Urd** is the language. **Wyrd** is the engine. Lambdas are the bridge when declarative rules aren't enough. The browser is the deployment target that makes all of it accessible without friction.

*End of Document*
