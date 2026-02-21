---
title: "Architectural Boundaries"
slug: "architectural-boundaries"
description: "What belongs in the schema, what belongs in the runtime, and what belongs in neither. The governance framework for evaluating every proposed feature, field, or capability against the three-layer architecture."
category: "architecture"
format: "Governance Document"
date: "2026-02-21"
status: "v1.0 complete"
order: 2
tags:
  - architecture
  - governance
  - boundaries
  - schema
  - runtime
  - adapter
details:
  - "Three-layer architecture: Urd (schema), Wyrd (runtime), Adapter (presentation)"
  - "Five-question boundary test for evaluating proposed changes"
  - "Content vs hint vs presentation — precise definitions"
  - "Five permanent exclusions: input model, text rendering, experience feedback, time, persistence"
  - "Evaluates all existing v1 fields against the framework"
  - "Worked examples for common proposals"
---

# Urd — Architectural Boundaries

*What belongs in the schema, what belongs in the runtime, and what belongs in neither*

> **Document status: NORMATIVE — GOVERNANCE**
> This document defines the architectural boundaries of the Urd system. It is the authority for deciding whether a proposed feature, field, or capability belongs in the schema (Urd), the runtime (Wyrd), or the adapter/presentation layer above both. All specification changes and feature proposals must be evaluated against this document before acceptance.
> Single canonical copy. February 2026.

---

## Purpose

Urd's value proposition is that it is *deliberately less than a programming language*. This constraint is what enables machine verification, static analysis, deterministic replay, and AI-native world consumption. Every feature that enters the schema weakens the constraint. Every presentation-layer concern that leaks into the world model couples Urd to a specific interaction paradigm.

This document exists because the pressure to add features is constant and legitimate. Writers want conditional room descriptions. Designers want parser grammar hints. Engineers want convenience fields. Community benchmarks assume parser-IF conventions. Each request is reasonable in isolation. Taken together, they erode the architectural boundary that makes Urd different from every other interactive fiction engine.

The purpose of this document is to make the boundary explicit, provide a repeatable framework for evaluating proposals, and define the categories of concern that are permanently outside Urd's scope — not deferred, but excluded by design.

---

## Definitions

Three terms recur throughout this document and across the specification suite. They have precise meanings.

**Content** is information that is part of the world's semantic structure. Without it, the world is incomplete or undefined. Dialogue lines are content — they define what an NPC says during a state transition. Choice labels are content — they define what the player is choosing between. Entity properties are content. Rules are content. A world stripped of all content is an empty schema.

**Hint** is an optional field attached to a world-semantic element whose purpose is to provide suggested text for adapters that want it. Hints are never load-bearing: a conforming runtime and adapter must function correctly if every hint field is empty or absent. `description` on a location is a hint — it suggests what the adapter might say about the room, but an adapter could generate its own description from the location's entities, exits, and properties. `blocked_message` on an exit is a hint — it suggests what the adapter might say when the exit's condition is false, but the adapter could produce its own feedback from the condition's semantics.

**Presentation** is any concern related to how the player perceives the world: text rendering, layout, formatting, conditional prose selection and rendering, input parsing, disambiguation, styling, animation, audio. Presentation is the adapter layer's exclusive domain. It never enters the schema or runtime.

The boundary between content and hint is tested by asking: *if this field were removed, would a conforming runtime produce incorrect world state?* If yes, it is content. If no, it is a hint. Hints may be useful, conventional, and universally consumed by adapters — but they remain optional by contract.

---

## The Three Layers

Urd is a three-layer system. Each layer has a single responsibility. The layers communicate through defined contracts. No layer reaches into another.

```
┌─────────────────────────────────────────────┐
│          ADAPTER / PRESENTATION             │
│                                             │
│  How the player experiences the world.      │
│  Parser, choice UI, graphical renderer,     │
│  AI agent, voice interface, accessibility   │
│  layer. Replaceable. Multiple can coexist.  │
│                                             │
│  Consumes: Wyrd API (read state, perform    │
│  actions, receive events)                   │
│  Produces: Player-facing experience         │
└──────────────────────┬──────────────────────┘
                       │ API boundary
┌──────────────────────▼──────────────────────┐
│               WYRD (RUNTIME)                │
│                                             │
│  What can happen in the world.              │
│  State management, condition evaluation,    │
│  effect application, rule execution,        │
│  action resolution, dialogue tracking.      │
│  Deterministic. Headless. No opinions       │
│  about presentation.                        │
│                                             │
│  Consumes: .urd.json (compiled world)       │
│  Produces: State snapshots, events          │
└──────────────────────┬──────────────────────┘
                       │ File contract
┌──────────────────────▼──────────────────────┐
│              URD (SCHEMA + COMPILER)        │
│                                             │
│  What the world is.                         │
│  Entities, types, properties, containment,  │
│  visibility, conditions, effects, rules,    │
│  sequences, dialogue structure.             │
│  Declarative. Verifiable. No runtime state. │
│                                             │
│  Consumes: .urd.md (authored source)        │
│  Produces: .urd.json (compiled world)       │
└─────────────────────────────────────────────┘
```

### Layer 1: Urd (Schema + Compiler)

**Urd answers the question: what exists and what are the rules?**

Urd defines the world's ontology — its entities, their types and properties, their spatial relationships, and the declarative rules governing state transitions. Urd is a data contract. It contains no runtime state, no execution logic, no presentation, no interaction model. A compiled `.urd.json` file is inert. It describes a world that *could* be executed; it does nothing on its own.

**Urd knows about:**
- Entity types and their property schemas
- Entity instances and their initial property values
- Containment relationships (what is inside what)
- Locations and exits (spatial topology)
- Property visibility levels (who can see what)
- Conditions (boolean expressions over world state)
- Effects (state mutations: set, move, reveal, destroy, spawn)
- Rules (triggered behaviours with conditions and effects)
- Actions (player-available interactions with conditions and effects)
- Sequences and phases (ordered progressions with advance modes)
- Dialogue structure (sections, choices, jumps, exhaustion)

**Urd does not know about:**
- What the player typed or clicked
- What text to display
- How to render anything
- What language the player speaks
- What interface paradigm is in use (parser, choice, graphical, voice)
- How to disambiguate player intent
- What synonyms or verb aliases exist
- What happens when a condition fails (the blocked *experience* is an adapter concern; the blocked *state* is a world concern)
- How to describe an entity or location to a human (presentation)
- Time, frame rate, or real-world clock
- Network, persistence, or save/load mechanics

### Layer 2: Wyrd (Runtime)

**Wyrd answers the question: given this world and these actions, what happens next?**

Wyrd is a headless simulation kernel. It loads a compiled world, holds its state, and processes actions. It is deterministic: identical world + identical seed + identical actions = identical result. Wyrd has no opinions about presentation, interaction, or user experience.

**Wyrd knows about:**
- Current world state (property values, containment, sequence phase, dialogue state)
- How to evaluate conditions against current state
- How to apply effects and update state
- How to fire rules when triggers are met
- How to resolve which actions are currently available (action applicability and parameter validity, not player intent)
- How to manage dialogue state (active section, consumed choices, exhaustion)
- How to produce typed events for every state mutation
- Deterministic random selection (seeded, reproducible)

**Wyrd resolves action applicability, not intent.** When a `perform(actionId)` call arrives, Wyrd checks whether the action exists, whether its conditions are met, and whether its parameters are valid. It does not determine whether the player meant "take" or "wear," whether "cloak" refers to the velvet cloak or the rain cloak, or whether the player's phrasing was close enough to a valid command. Intent resolution is the adapter's responsibility. Wyrd receives structured, unambiguous action identifiers. How those identifiers are produced from human input is not Wyrd's concern.

**The world advances only when Wyrd receives an action call.** There is no background tick, no frame loop, no time-based evaluation. If no action is performed, no state changes, no rules fire, and no events are produced. This is a normative constraint: a conforming runtime must not advance world state except in response to an explicit action call through the API.

**Failure results.** Not every `perform()` call succeeds. When a call cannot be executed, Wyrd returns a structured failure result — not an event. Events are emitted only on state mutation; failures do not mutate state and are returned as the `perform()` call's response, not added to the event stream. **Neither category of failure may mutate world state under any circumstances.**

Failures fall into two categories:

- **Request validation failures.** The action identifier is unrecognised, parameters are invalid or missing, or the call is malformed. These indicate adapter-side errors — the adapter asked for something the world does not define. Validation failures never include condition references.
- **World state failures.** The action exists but its conditions are not met, an exit is blocked, or a dialogue choice is currently unavailable. These indicate legitimate attempts against current state. World state failures never include parameter schema references.

This distinction gives adapters a clean branching point: validation failures are programming errors to fix; world state failures are gameplay feedback to present. **Failures never appear in the event stream, even as a special failure event type.** The event stream is reserved exclusively for state mutations.

The failure result includes a reason code from a defined vocabulary. For world state failures, the result includes a reference to the failing condition using a stable, compiler-assigned identifier from the compiled `.urd.json` — not free-text. For request validation failures involving parameter constraints, the result references the parameter schema node or compiler-assigned constraint identifier, following the same stability guarantees. If multiple conditions fail, Wyrd returns the first failing condition in deterministic evaluation order, and may optionally include all failures.

The exact identifier format, the condition evaluation order rule, and the exhaustive reason code vocabulary are specified in the Wyrd Reference Runtime document. **This document requires these specifications to exist. The Wyrd Reference Runtime document defines them.**

Generating failure feedback is the adapter's responsibility, not Wyrd's. The runtime returns structured data, never default text. A conforming interactive adapter must handle failure results without corrupting world state, and must provide some player-perceivable feedback unless the adapter is explicitly operating in a non-player-facing mode (e.g., a test harness, analytics runner, or simulation).

**Wyrd does not know about:**
- What the player typed or clicked (it receives structured action IDs)
- What text to display (it produces events; the adapter renders them)
- How to parse natural language
- What interface the player is using
- Screen layout, fonts, colours, audio
- Save file formats (it can serialize state; persistence is the adapter's job)
- Network or multiplayer (future extension, not v1)

### Layer 3: Adapter / Presentation

**The adapter answers the question: how does the player experience this world?**

The adapter is the only layer that touches the player. It translates between human intent and structured world actions. It translates between world events and human-perceivable output. Adapters are replaceable, composable, and entirely outside Urd's and Wyrd's concern.

**Examples of adapters:**
- A parser that converts typed text ("take cloak", "hang cloak on hook") into structured action calls
- A choice-based UI that renders available actions as clickable buttons
- A graphical renderer that maps world state to 3D environments
- An AI agent that selects actions based on entity properties and goals
- A voice interface that converts speech to actions and events to speech
- An accessibility layer that adapts output for screen readers
- A test harness that executes scripted action sequences

**The adapter knows about:**
- Wyrd's API (getAvailableActions, perform, getState, events)
- How to present information to the player
- How to interpret player input
- Verb synonyms, grammar, disambiguation
- Text composition (selecting, generating, or combining prose based on world state)
- UI layout, styling, animation
- Save/load, persistence
- Platform-specific concerns

**The adapter does not:**
- Modify `.urd.json` files
- Extend the schema with adapter-specific fields
- Reach into Wyrd's internal state (it uses the public API only)
- Define world rules or conditions (those live in Urd)

---

## The Boundary Test

When evaluating whether a proposed feature, field, or change belongs in Urd, Wyrd, or the adapter layer, apply these questions in order. Stop at the first "yes."

### Question 1: Does it describe what exists?

*Entities, types, properties, relationships, spatial topology.*

> **If yes, it belongs in Urd (the schema).**

Examples: a new entity type, a new property on an existing type, a new location, a new exit, a new containment relationship.

### Question 2a: Does it describe a rule about what can happen?

*Conditions, effects, triggers, constraints on state transitions.*

> **If yes, it belongs in Urd (the schema) if it is declarative and verifiable. It belongs in the adapter if it requires imperative logic, external state, or non-determinism beyond seeded randomness.** Before accepting, also evaluate Question 2b.

Examples that belong in Urd: "the door opens when the key is used" (condition + effect), "Monty reveals a goat door" (rule with select), "the phase advances when the player reads the message" (on_condition advance).

Examples that do not belong in Urd: "the door opens when the player types 'open door'" (input parsing), "the NPC responds differently based on the player's tone" (sentiment analysis), "generate a description of the room" (text generation).

### Question 2b: Does it depend on adapter behaviour rather than world state?

*Would two different adapters (a parser, a choice UI, a test harness) produce different trigger sequences or outcomes for this feature given the same world state?*

> **If yes, it belongs in the adapter layer, even if it passed 2a.** World rules must produce identical results regardless of which adapter is driving the interaction. If a feature's behaviour varies based on how the player communicates intent — how many invalid commands were issued, which disambiguation path was taken, whether the adapter pre-filters actions — it is adapter-coupled and does not belong in the schema or runtime.

### Question 3: Does it describe how state changes are evaluated or applied?

*Condition evaluation, effect application, rule execution ordering, determinism guarantees.*

> **If yes, it belongs in Wyrd (the runtime).**

Examples: the order in which rules fire, how `select` resolves ties, how `always` triggers interact with other triggers, when exhaustion is recalculated.

### Question 4: Does it describe how the player perceives or interacts with the world?

*Text output, input parsing, UI rendering, disambiguation, language, accessibility.*

> **If yes, it belongs in the adapter layer. It does not belong in Urd or Wyrd.**

Examples: room descriptions, verb synonyms, parser grammar, button labels, text composition based on world state, how to render a failed action ("You fumble in the dark" vs. "It's too dark to do that"), inventory display format, how to present choices to the player.

### Question 5: Does it provide a convenience that helps minimal runtimes?

*Hint fields that adapters may use but are not required to.*

> **If yes, it may live in Urd as an optional, non-normative field — but only if it is clearly marked as a presentation hint, not a world-semantic field. Adapters must function correctly without it.**

Examples: `description` on locations (a string hint; the adapter can ignore it and generate its own), `blocked_message` on exits (a hint for what to show when a condition fails; the adapter can produce its own message).

**The critical constraint on hint fields:** they must never become load-bearing. If removing every hint field from a `.urd.json` file causes a conforming runtime to malfunction, the boundary has been violated. Hint fields are convenience, not contract.

---

## Permanent Exclusions

The following categories of concern are not deferred to future versions. They are excluded from Urd and Wyrd by architectural design. Proposing features in these categories requires first arguing that the architectural boundary should move, not just that the feature is useful.

### 1. Input Model

Urd and Wyrd have no concept of how the player communicates intent. Not parser commands, not button clicks, not gestures, not voice. The runtime receives structured action identifiers. How those identifiers are produced from human input is the adapter's problem.

**Consequence:** No verb synonyms, no grammar hints, no disambiguation priorities, no input validation, no "understood" vs "not understood" semantics in the schema. A world file is equally playable through a parser, a choice UI, an AI agent, or an automated test harness.

### 2. Text Rendering and Conditional Presentation

Urd describes world state. Adapters decide what text to show. If a room has different descriptions when lit vs. dark, the world model provides the `lit` boolean property. The adapter queries the property and selects the appropriate text. The world model does not contain both texts or a mechanism for switching between them.

**Consequence:** No conditional description fields, no text template systems, no multi-description-per-entity patterns, no markup or formatting in schema fields. The `description` field is a static hint, not a rendering system. **Core principle: world semantics must not depend on prose.** Prose may exist as content (dialogue lines, choice labels) or hints (location descriptions, blocked messages), but the schema will not include mechanisms for selecting between multiple prose variants based on state. That is text composition, and text composition is adapter work. **The inverse also holds: no rule or condition may depend on reading or parsing hint text.** Hint fields are consumed by adapters for rendering purposes only. They are never inputs to world logic.

**The test:** If an adapter wanted to present the world as a 3D environment with no text at all, could it do so using only the world state? If yes, the boundary is clean. If no — if the adapter needs to read `description_lit` to know what the 3D room looks like when the lights are on — the schema has absorbed a presentation concern.

### 3. Player Experience Feedback

What happens when a player tries something that fails? The world model knows the action's condition was false. The adapter decides the experience: a "fumble" message, a "you can't do that" response, a red flash, silence, a narrator's comment. The world model does not produce failure-mode text or define meta-actions for rejected input.

**Consequence:** No "fumble" actions, no "attempt failed" events, no adapter-specific side effects on condition failure. The world reports state. The adapter decides how failure feels.

### 4. Time and Pacing

Urd has no clock. There is no concept of "a tick," "a turn," "a beat," or real-time duration. The world advances only when Wyrd receives an action call through its API. Between action calls, nothing happens — no state changes, no rule evaluation, no events.

**The `always` trigger is evaluated once per action call, after all other triggered rules for that action have resolved.** It is not a timer, not a loop, and not a background process. A world with an `always` rule and no player actions is inert. Any reference to "tick" in v1 normative documents is defined as "action call" and must be interpreted accordingly. A no-op action does not exist in the Wyrd API unless an adapter chooses to expose one as a valid world action — in which case it is a real action with real effects, not a clock tick.

The pacing of the experience — how fast text appears, how long the player has to decide, whether there are timers or countdowns — is entirely the adapter's concern. Future time systems (NPC schedules, day/night cycles) would extend the runtime contract, not the schema.

**Consequence:** No tick rate, no frame concept, no scheduled events, no time-based triggers in v1. The `always` trigger fires once after each player action. An `always` rule that increments a counter will increment it once per action, not continuously.

### 5. Persistence and Save/Load

How world state is serialized, stored, and restored is an adapter/infrastructure concern. Wyrd can export and import state snapshots through its API, but the mechanism of saving to disk, cloud, or local storage is outside both Urd and Wyrd.

**Consequence:** No save-file format in the schema. No session management in the runtime. The API exposes `getState()` and allows state injection; everything else is external.

---

## Evaluating the Current Schema Against This Framework

The existing v1 schema is largely clean. The following fields are the only ones that sit on the boundary:

| Field | Location | Classification | Status |
|-------|----------|---------------|--------|
| `description` on locations | Schema Spec §Locations | Hint | Acceptable. Optional. Non-load-bearing. A conforming runtime and adapter must function correctly if this field is absent or empty. |
| `description` on types | Schema Spec §Types | Hint | Acceptable. Optional. Non-load-bearing. |
| `description` on actions | Schema Spec §Actions | Hint | Acceptable. Optional. Non-load-bearing. |
| `description` on rules | Schema Spec §Rules | Documentation | Acceptable. Never shown to players. |
| `description` on dialogue sections | Schema Spec §Dialogue | Hint | Acceptable. Prose narration before dialogue. |
| `blocked_message` on exits | Schema Spec §Exits | Hint | Acceptable. Optional. The condition failure is the world concern; the message is the hint. When an exit's condition fails, Wyrd returns a world state failure result (see §Layer 2: Wyrd, Failure results). The `blocked_message` text, if present, is attached to the failure result as an optional field. Adapters may render it, ignore it, or substitute their own text. |
| `prompt` on sequence phases | Schema Spec §Sequences | Hint | Acceptable. Optional. The phase's existence is the world concern; the text is the hint. |
| `prompt` on dialogue sections | Schema Spec §Dialogue | Content | Dialogue text is a semantic transition artifact — it defines what is said during a world state transition, not how it is rendered. The same field serves text UIs, AI agents, and test harnesses. Acceptable for v1. Textual content is adapter-rendered but world-authored; this does not make it presentation. Monitor for scope creep toward multi-variant or conditional prompts. |
| `label` on dialogue choices | Schema Spec §Dialogue | Content | **Acceptable.** The choice's label is its identity. Without it, the choice has no meaning to any adapter. This is world content, not presentation. |
| `response` on dialogue choices | Schema Spec §Dialogue | Content | **Acceptable.** Same reasoning as dialogue prompt. The response is the content of the interaction. |

**Verdict:** No existing v1 field violates the boundary. Several are hints (description, blocked_message, phase prompt), which is acceptable as long as they remain optional and non-load-bearing. The dialogue fields (section prompt, choice label, choice response) are content — they define what is said during world state transitions, not how it is displayed.

---

## Applying the Framework: Common Proposals

To demonstrate how the boundary test works in practice, here are proposals that have already surfaced or are likely to surface, evaluated against the framework.

### "Add conditional room descriptions"

*The room should show different text when lit vs. dark.*

**Boundary test:** Question 4 — this describes how the player perceives the world. The world model provides `room.lit: true/false`. The adapter decides what text to show based on that state. Adding `description_lit` and `description_dark` to the schema absorbs a presentation concern.

**The conditional visibility distinction:** The schema's conditional visibility primitive (§Visibility Model) gates *whether a property's value is accessible* — information availability, not text selection. A property with conditional visibility either reveals its value or doesn't. That is world semantics: the information exists in the player's view or it doesn't. Conditional visibility is not a mechanism for switching between alternative descriptions, formatting text, or selecting which prose to display. It governs the existence of information, not its rendering.

v1 worlds are expected to be playable without rich prose variation. The adapter layer can query any combination of world state (entity properties, containment, visibility) and produce arbitrarily complex text from it. A world that needs "the room feels different when the lights are on" expresses this through the `lit` property on the room; the adapter reads the property and renders accordingly. The world model provides the state. The adapter provides the words.

**Verdict: Adapter layer.** The schema provides the boolean (or enum, or any other state). The adapter queries it and selects, generates, or composes text. No conditional text fields, no multi-description patterns, no text switching in the schema. Room descriptions are hints, not world semantics. Producing multiple description variants based on state is a text composition pattern that lives in the adapter.

### "Add a parser grammar block to the schema"

*Include verb synonyms and grammar hints so parser adapters work well.*

**Boundary test:** Question 1 — no, this doesn't describe what exists. Question 4 — yes, this describes how the player interacts.

**Verdict: Adapter layer.** If parser hints are needed, they go in a sidecar file or an adapter-specific configuration, not in `.urd.json`. The schema should never know about verbs.

### "Add a 'fumble' meta-action for failed attempts in the dark"

*When the player tries an action in a dark room and it fails, a fumble action fires.*

**Boundary test:** Question 4 — yes, "fumble" is experience feedback for a failed action. The world knows the action's condition was false. What happens to the player as a result is the adapter's decision.

**Verdict: Adapter layer.** The world model expresses "this action requires the room to be lit." The adapter decides what "trying and failing" looks like.

### "Add an `always` trigger that fires every tick"

*A rule should fire continuously, not just after player actions.*

**Boundary test:** Question 4 — partially. "Every tick" implies a clock, which is a presentation/infrastructure concept (see Permanent Exclusion #4). "After every player action" is a world-semantic concept — the world reacts to every input.

**Verdict: Wyrd (runtime), defined as per-action.** The trigger fires after every player action, not on a timer. No clock exists in the world model.

### "Add an `on_attempt` trigger for failed actions"

*A rule should fire when a player tries an action that fails its conditions.*

**Boundary test:** Question 2a — this describes a rule about what can happen. Question 2b — yes, "attempting an action that fails" introduces a dependency on the *sequence of invalid calls* the adapter makes, not on world state. Two adapters interacting with the same world may produce different sequences of failed attempts depending on their input handling — a parser adapter might generate many failed attempts as it disambiguates, while a choice UI generates none because it only shows valid actions. Tying world-semantic rules to this sequence makes world state dependent on adapter implementation details.

**The deeper issue:** Firing world rules on failed actions encourages encoding experience feedback into the world model. A "fumble" counter that increments on failed attempts is an adapter-layer concern dressed up as world state. It makes world semantics depend on the interface error path, complicates determinism (the event stream now varies based on which invalid actions were attempted), and incentivises content authors to design around adapter behaviour rather than world structure.

**Verdict: Excluded from the schema and runtime.** Wyrd may return a structured failure result from a `perform()` call — this is runtime API design, not world semantics. But world rules must not trigger on failures. If a game design requires tracking failed attempts for experiential purposes (e.g., a "fumbling in the dark" counter), the adapter maintains that state locally — it is purely experiential, not world-semantic. Alternatively, the adapter may map failures onto existing valid actions that already serve a world purpose. The key constraint: world files should not contain actions or rules whose sole reason for existence is to service adapter-layer UX patterns.

### "Add `on_enter` / `on_exit` effects on locations"

*Effects should fire when an entity enters or leaves a location.*

**Boundary test:** Question 2a — yes, this describes a rule about what can happen when containment changes. It is declarative, verifiable, and operates on world state only. No presentation concern. Question 2b — no, the result is identical regardless of adapter.

**Verdict: Urd (schema).** Already specified in the Schema Spec. Already added to the JSON Schema (erratum fix).

---

## v1 Completion Guidance

This framework provides three guardrails for completing v1 without architectural contamination:

### Guardrail 1: Every proposed change gets the five-question test

Before any change to the Schema Specification, JSON Schema, Schema Markdown syntax, or Wyrd runtime specification, apply the boundary test from §The Boundary Test. If the proposal ends up in the adapter layer, it does not enter the schema or runtime regardless of how useful it would be for the current test cases.

### Guardrail 2: Hint fields stay optional and non-load-bearing

The existing hint fields (`description`, `blocked_message`, `prompt`) are acceptable because adapters can ignore them entirely and still build a complete experience from the structured world state. If any proposal would make a hint field load-bearing (a conforming runtime *must* use this text), it has crossed the boundary.

### Guardrail 3: If a benchmark requires parser-IF assumptions, adapt the benchmark

Urd will encounter benchmarks designed for parser-IF (Cloak of Darkness, Curses, Advent). These benchmarks assume typed commands, implicit actions (examine, inventory), and text output. Adapting these to Urd means modelling their *world semantics* (entities, containment, rules) in the schema and their *interaction model* (parser, feedback, text composition) in the adapter. If a benchmark mechanic cannot be cleanly split, it is the benchmark that needs adaptation, not the schema.

---

## When a Proposal Fails the Boundary Test

Rejection is not deletion. When a proposed feature, field, or capability is evaluated against the boundary test and determined to belong in the adapter layer rather than the schema or runtime, the following process applies:

1. **Document the rejection.** Add an entry to the Future Proposals document with the proposal name, the boundary test evaluation, and the specific question(s) that placed it outside Urd/Wyrd scope.

2. **Document the adapter-layer solution.** Describe how the desired capability can be achieved using the existing schema primitives plus adapter-layer logic. This is the "how to do it without changing the schema" pattern. Most proposals have a clean adapter-side implementation that the schema already supports.

3. **If the adapter-layer solution reveals a gap in the Wyrd API** — for example, the adapter needs a structured failure response from `perform()` that Wyrd doesn't currently provide — that API extension is a legitimate Wyrd enhancement. API surface changes are evaluated separately: they must not leak adapter concerns into the runtime's internal logic, but they may expose additional runtime state or structured responses that adapters need.

4. **If the proposer believes the boundary itself should move,** they must make that case explicitly. The argument is not "this feature is useful" (most are) but "the architectural boundary is drawn in the wrong place for this category of concern." This is a high bar by design. Moving the boundary affects every downstream implementation.

This process ensures that rejected proposals are captured, that adapter-pattern solutions are documented for implementers, and that boundary challenges are elevated rather than ignored.

---

## Relationship to Other Documents

| Document | Relationship |
|----------|-------------|
| Schema Specification | This document governs what may be added to the schema. The Schema Spec's "Design Principles" section is compatible with and subordinate to this boundary framework. |
| Architecture | The Architecture document describes the pipeline and component interfaces. This document defines what each component is *allowed to care about*. |
| Wyrd Reference Runtime | The Wyrd spec defines what the runtime does. This document defines what it must *not* do. |
| Schema Markdown Syntax Specification | The syntax spec defines what writers can express. This document constrains what that expression may describe: world semantics, not presentation. |
| Future Proposals | Every future proposal must pass the boundary test in this document before design work begins. |

*End of Document*
