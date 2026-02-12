---
title: "Test Case Strategy"
slug: "urd-test-case-strategy"
description: "Validating the schema through executable proof. Four progressive test cases — Tavern Scene (dialogue systems), Monty Hall (emergent probability), Two-Room Key Puzzle (spatial + inventory), and Interrogation (dialogue stress test)."
category: "validation"
format: "Validation Plan"
date: "2026-02"
status: "v1.0 complete"
order: 1
tags:
  - testing
  - validation
  - monte-carlo
  - test-cases
  - proof
details:
  - "Three testing layers: static analysis, playthrough simulation, statistical validation"
  - "Monty Hall: emergent 2/3 switching advantage from structure alone"
  - "Two-Room Key Puzzle: containment as universal spatial primitive"
  - "Tests run against compiled JSON, not source — validates the real artefact"
---

> **Document status: INFORMATIVE**
> Defines the test case strategy for validating the schema and runtime against executable proof. Test definitions reference the normative specifications.
> Single canonical copy. February 2026 draft.

# URD

## Test Case Strategy

*Validating the schema through executable proof*

urd.dev

February 2026

## Purpose

This document defines the test case strategy for the Urd world schema and the Wyrd reference runtime. It specifies what is tested, how tests are structured, what each test case validates, and which schema primitives remain uncovered.

The strategy serves three audiences. Engineers use it to know what to build and in what order. Writers use it (indirectly) because the test cases are also the worked examples in the documentation. The urd.world product face uses it because the test cases are the playable demos that visitors experience.

Testing in Urd has a unique property: **the test cases are also the specification's proof of correctness.** The Monty Hall problem is not just a test. It is the demonstration that declarative structure produces emergent probability. The Two Room Key Puzzle is not just a test. It is the demonstration that a single containment primitive replaces inventory, storage, and spatial navigation. If the tests pass, the schema works. If they don't, either the schema or the runtime has a bug.

## Testing Layers

Urd testing operates at three layers. Each catches a different category of defect.

| Layer | What It Catches | How It Runs |
|-------|----------------|-------------|
| Static analysis | Structural defects in world files: orphaned locations, unreachable actions, impossible conditions, type mismatches, undefined entity references. | Compiler. Runs on `.urd.md` source during compilation. |
| Playthrough simulation | Semantic defects in world behaviour: wrong state after a sequence of actions, incorrect condition evaluation, rules that fire when they shouldn't or fail to fire when they should. | Wyrd runtime. `Wyrd.simulate(actions)` executes a scripted sequence of player actions and returns the resulting state and events. |
| Statistical validation | Probabilistic defects: emergent distributions that don't match expected outcomes, random selection bias, rule constraints that don't produce correct statistical behaviour. | Wyrd runtime. Monte Carlo mode runs thousands of seeded playthroughs and asserts on aggregate distributions. |

**Tests run against compiled JSON, not source.** The testing framework never parses `.urd.md`. It receives the same `.urd.json` the runtime uses. This means tests validate the actual artifact that engines will consume, not an intermediate representation.

**The runtime is embedded, not mocked.** The testing framework uses the real Wyrd engine to execute playthroughs. It does not reimplement world logic. Tests exercise the same code path as production.

**Static analysis is complementary, not a replacement.** Reachability analysis can catch structural bugs (orphaned locations, impossible conditions) without running any playthroughs. But it cannot catch semantic bugs (Monty opening the wrong door, the key not unlocking the cell). Both static and dynamic testing are needed.

## The Four Test Cases

The Urd documentation set contains four worked examples. Together they exercise the full v1 schema. Each test case is a complete, valid world file that can be compiled, loaded into Wyrd, and executed.

The ordering here follows the Schema Markdown Syntax Specification, which leads with dialogue because Schema Markdown is a writer facing syntax and dialogue is the writer's primary domain. The Wyrd build passes follow a different order (core engine first, dialogue last) because the runtime must have fundamentals in place before it can execute conversations. Both orderings are correct for their context; the mapping between test cases and build passes is noted in each section.

| Test Case | Source Document | Mode | Primary Validation |
|-----------|----------------|------|-------------------|
| The Tavern Scene | Schema Markdown Syntax Specification | Dialogue | Hub and spoke conversation, sticky/one shot choices, conditional reveals, state mutation, exhaustion |
| The Monty Hall Problem | Schema Specification | Sequence driven | Hidden state, constrained NPC behaviour, emergent probability |
| The Two Room Key Puzzle | Schema Specification | Freeform | Containment as inventory, spatial navigation, conditional exits, entity destruction |
| The Interrogation | Nested Dialogue Design Exploration | Dialogue | Multi topic hub, conditional sub branches, containment in dialogue, state dependent farewell |

## Test Case 1: The Tavern Scene

### What It Validates

This test case validates the dialogue system: labeled sections, hub and spoke navigation, sticky and one shot choices, conditional content gating, state mutation within dialogue, exhaustion detection, and fallthrough content. It is the first test case in the documentation because dialogue is what writers interact with most, and Schema Markdown is a writer facing syntax.

The critical validation: **choice semantics are correct.** One shot choices (`*`) disappear after selection and never reappear. Sticky choices (`+`) remain available on every revisit. Exhaustion occurs when all choices are consumed or gated. These are the normative semantics defined in the syntax spec, and the tests must verify them exactly.

### Schema Primitives Exercised

- Dialogue sections (`== topics`)
- Hub and spoke structure (`-> topics` loops)
- Sticky choices (`+` Ask about the harbor, Buy her a drink)
- One shot choices (`*` Ask about the missing ship, Leave)
- Conditional content gating (`? @arina.trust > 50`)
- State mutation in dialogue (`> @arina.trust + 5`)
- Stage direction vs speech (`@arina leans in close.` vs `@arina: text`)
- Exhaustion and fallthrough text

### Test Definitions

```
test "one-shot choice disappears after selection" {
  world: tavern-talk.urd.json
  steps:
    - choose: "Ask about the missing ship"
    - assert: events contain dialogue from arina
    - jump: topics
    - assert: "Ask about the missing ship" not in available_choices
}

test "sticky choice remains after selection" {
  world: tavern-talk.urd.json
  steps:
    - choose: "Ask about the harbor"
    - jump: topics
    - assert: "Ask about the harbor" in available_choices
}

test "trust gates the secret reveal" {
  world: tavern-talk.urd.json
  steps:
    - assert: arina.trust == 30
    - choose: "Ask about the missing ship"
    - assert: events contain dialogue "I don't know what you're talking about."
    - comment: trust is 30, below the 50 threshold
}

test "trust accumulates across choices" {
  world: tavern-talk.urd.json
  steps:
    - choose: "Ask about the harbor"
    - assert: arina.trust == 35
    - choose: "Buy her a drink"
    - assert: arina.trust == 55
    - choose: "Ask about the missing ship"
    - assert: events contain dialogue "The Selene didn't sink. She was taken."
    - comment: trust is now 55, above the 50 threshold
}

test "exhaustion produces fallthrough text" {
  world: tavern-talk.urd.json
  steps:
    - choose: "Ask about the missing ship"
    - jump: topics
    - choose: "Leave"
    - assert: events contain narration "Suit yourself. I've got glasses to clean."
}
```

### Wyrd Build Pass

**Pass 3.** Although the Tavern Scene is Test Case 1 in the documentation, it is validated in Wyrd's Pass 3 because the runtime needs core engine capabilities (Pass 1) and spatial mechanics (Pass 2) before it can execute dialogue.

## Test Case 2: The Monty Hall Problem

### What It Validates

This test case validates the core engine: entity instantiation, hidden property state, constrained NPC behaviour via the `select` block, sequence phasing, and the `reveal` effect. It exercises the most fundamental schema primitives with the smallest world definition.

The critical validation: **no probability is specified anywhere in the schema.** The 2/3 switching advantage emerges from the structure. If the test produces the correct distribution, the schema's declarative model is proven correct. If it doesn't, the `select` constraint or the condition evaluation is broken.

### Schema Primitives Exercised

- `world` block (metadata, start location, entry sequence)
- `types` block (property schemas with enum, boolean)
- `visibility: hidden` (prize behind doors)
- `traits: interactable`
- `entities` block (instances with property overrides)
- `player` (implicit)
- `locations` (single location)
- `rules` with `select` block (constrained NPC choice)
- `actions` with conditions
- `effects: set`
- `effects: reveal`
- `sequences` with phases and advance modes

### Test Definitions

```
test "Monty never opens the car door" {
  world: monty-hall.urd.json
  runs: 10000
  strategy: random
  assert: every reveal event targets a door where prize == goat
}

test "Monty never opens the player's chosen door" {
  world: monty-hall.urd.json
  runs: 10000
  strategy: random
  assert: every reveal event targets a door where chosen == false
}

test "switching wins approximately 2/3 of the time" {
  world: monty-hall.urd.json
  runs: 10000
  strategy: always_switch
  assert: win_rate > 0.63
  assert: win_rate < 0.70
}

test "staying wins approximately 1/3 of the time" {
  world: monty-hall.urd.json
  runs: 10000
  strategy: always_stay
  assert: win_rate > 0.30
  assert: win_rate < 0.37
}

test "all prizes are revealed in resolve phase" {
  world: monty-hall.urd.json
  steps:
    - action: choose_door (target: door_1)
    - await: phase reveal
    - await: phase resolve
    - assert: door_1.prize.visibility == visible
    - assert: door_2.prize.visibility == visible
    - assert: door_3.prize.visibility == visible
}
```

### Wyrd Build Pass

**Pass 1.** The Monty Hall test case is the acceptance test for the core engine. Pass 1 is complete when these tests pass.

## Test Case 3: The Two Room Key Puzzle

### What It Validates

This test case validates spatial mechanics: movement between locations, containment as inventory (picking up, carrying, and consuming objects), conditional exits, NPC state arcs gated by conditions, and entity destruction. It exercises every schema primitive that the Monty Hall problem does not.

The critical validation: **containment replaces inventory.** "Pick up the key" is `move: rusty_key, to: player`. "Does the player have the key?" is `rusty_key.container == player`. "Drop the key in the room" is `move: rusty_key, to: player.container`. If these operations work correctly, the unified containment model is proven sound.

### Schema Primitives Exercised

- `visibility: owner` (guard's hint_given)
- `traits: portable` (key)
- `traits: container` (guard)
- `traits: mobile` (guard)
- Containment as inventory (move entity to player)
- `locations` (two locations with exits)
- Exits with conditions and blocked messages
- `effects: move` (spatial transfer)
- `effects: destroy` (key consumed on use)
- Freeform mode (no sequence block)

### Test Definitions

```
test "player starts in cell" {
  world: two-room-key.urd.json
  steps:
    - assert: player.container == cell
}

test "exit is blocked when door is locked" {
  world: two-room-key.urd.json
  steps:
    - action: move north
    - assert: player.container == cell
    - assert: events contain blocked_message "The iron door is locked."
}

test "full escape sequence" {
  world: two-room-key.urd.json
  steps:
    - action: offer_patience
    - assert: guard.mood == neutral
    - action: talk_to_guard
    - assert: guard.hint_given == true
    - action: pick_up_key
    - assert: rusty_key.container == player
    - action: unlock_door
    - assert: cell_door.locked == false
    - assert: rusty_key.destroyed == true
    - action: move north
    - assert: player.container == corridor
}

test "guard blocks information when hostile" {
  world: two-room-key.urd.json
  steps:
    - assert: guard.mood == hostile
    - assert: talk_to_guard not in available_actions
}

test "key requires co-location" {
  world: two-room-key.urd.json
  steps:
    - assert: pick_up_key in available_actions
    - comment: key and player are both in cell
}
```

### Wyrd Build Pass

**Pass 2.** The Key Puzzle is the acceptance test for spatial navigation and the containment model. Pass 2 also introduces the browser presentation layer, so the puzzle must be playable interactively.

## Test Case 4: The Interrogation

### What It Validates

This test case is the stress test. It validates the dialogue system at production complexity: multiple hub topics with state mutation across topics, conditional sub branches within choices, a separate section for complex logic (the bribe), containment checks in dialogue (`? @coin_purse in player`), containment transfer as a dialogue effect (`> move @coin_purse -> @halvard`), and a state dependent farewell with three conditional variants.

The critical validation: **the hybrid nesting model works at scale.** Shallow branches stay inline under choices (the "Press him" sub branch). Deep or complex branches break out to labeled sections (the bribe). State accumulated across multiple topics gates later content (knowing about the cell changes the warden topic). If this scene works correctly, the dialogue system handles real game conversations, not just demos.

### Schema Primitives Exercised

- Multi topic hub with state mutation across topics
- Inline conditional sub branches (Press him: mood == neutral vs hostile)
- Section breakout for complex logic (`== bribe`)
- Containment checks in dialogue (`? @coin_purse in player`, `? @coin_purse not in player`)
- Containment transfer as dialogue effect (`> move @coin_purse -> @halvard`)
- State dependent farewell (three conditional variants in `== farewell`)
- Cross topic state dependencies (`player.knows_cell` gates the warden topic)
- OR conditions (`? any:` block gating guard refusal on hostile or suspicious mood)

### Test Definitions

```
test "hostile guard blocks information" {
  world: interrogation.urd.json
  steps:
    - choose: "Ask about the prisoner"
    - choose: "Press him"
    - assert: events contain dialogue "I said, what prisoner?"
    - comment: guard starts hostile, so pressing him fails
}

test "patience unlocks information" {
  world: interrogation.urd.json
  steps:
    - choose: "Ask about the prisoner"
    - choose: "Press him"
    - comment: first attempt fails because mood is hostile
    - jump: interrogation
    - comment: mood changes to neutral through other interactions
    - choose: "Ask about the prisoner"
    - choose: "Press him"
    - assert: halvard.mood == nervous
    - assert: player.knows_cell == true
}

test "bribe requires coin purse" {
  world: interrogation.urd.json
  precondition: coin_purse.container != player
  steps:
    - choose: "Try to bribe him"
    - assert: events contain dialogue "Bribe with what? You've got nothing."
}

test "successful bribe transfers coin purse" {
  world: interrogation.urd.json
  precondition: coin_purse.container == player, halvard.mood == neutral
  steps:
    - choose: "Try to bribe him"
    - assert: coin_purse.container == halvard
    - assert: halvard.mood == neutral
}

test "farewell varies by mood" {
  world: interrogation.urd.json

  variant nervous:
    precondition: halvard.mood == nervous
    steps:
      - choose: "I'm done here"
      - assert: events contain dialogue "Watch yourself out there."

  variant hostile:
    precondition: halvard.mood == hostile
    steps:
      - choose: "I'm done here"
      - assert: events contain narration "The door slams behind you."

  variant neutral:
    precondition: halvard.mood == neutral
    steps:
      - choose: "I'm done here"
      - assert: events contain dialogue "Don't come back."
}

test "OR condition gates guard refusal" {
  comment: validates ? any: OR condition evaluation
  comment: the guard refuses to talk when mood is hostile OR suspicious
  world: interrogation.urd.json

  variant hostile:
    precondition: halvard.mood == hostile
    steps:
      - choose: "Ask about the escape route"
      - assert: events contain dialogue "I don't talk to your kind."

  variant suspicious:
    precondition: halvard.mood == suspicious
    steps:
      - choose: "Ask about the escape route"
      - assert: events contain dialogue "I don't talk to your kind."

  variant neutral_allows:
    precondition: halvard.mood == neutral
    steps:
      - choose: "Ask about the escape route"
      - assert: events contain dialogue "There's a passage behind the chapel."
}
```

### Wyrd Build Pass

**Pass 3.** The Interrogation is the stress test within the same pass as the Tavern Scene. It validates that dialogue scales to production complexity.

## Coverage Analysis

### What the Four Test Cases Cover

The combined test suite exercises every v1 schema primitive. The Schema Specification's coverage matrix maps Monty Hall and Key Puzzle coverage. The table below extends it to include the two dialogue test cases.

| Schema Primitive | Tavern | Monty Hall | Key Puzzle | Interrogation |
|-----------------|--------|------------|------------|---------------|
| world (metadata) | ✓ | ✓ | ✓ | — |
| types (property schemas) | ✓ | ✓ | ✓ | — |
| visibility: hidden | ✓ | ✓ | — | — |
| visibility: owner | — | — | ✓ | — |
| traits: portable | — | — | ✓ | — |
| traits: container | — | — | ✓ | — |
| traits: interactable | ✓ | ✓ | ✓ | — |
| entities (instances) | ✓ | ✓ | ✓ | — |
| player (implicit) | — | ✓ | ✓ | — |
| containment as inventory | — | — | ✓ | ✓ |
| locations | ✓ | ✓ | ✓ | — |
| exits with conditions | — | — | ✓ | — |
| rules with select | — | ✓ | — | — |
| actions with conditions | — | ✓ | ✓ | — |
| effects: set | ✓ | ✓ | ✓ | ✓ |
| effects: reveal | — | ✓ | — | — |
| effects: move | — | — | ✓ | ✓ |
| effects: destroy | — | — | ✓ | — |
| sequences (phases) | — | ✓ | — | — |
| freeform (no sequence) | — | — | ✓ | ✓ |
| dialogue sections | ✓ | — | — | ✓ |
| sticky choices (+) | ✓ | — | — | — |
| one shot choices (*) | ✓ | — | — | ✓ |
| exhaustion + fallthrough | ✓ | — | — | — |
| conditional content gating | ✓ | — | — | ✓ |
| state mutation in dialogue | ✓ | — | — | ✓ |
| hub and spoke navigation | ✓ | — | — | ✓ |
| containment checks in dialogue | — | — | — | ✓ |
| containment transfer in dialogue | — | — | — | ✓ |
| stage direction vs speech | ✓ | — | — | ✓ |
| section breakout (hybrid nesting) | — | — | — | ✓ |
| OR conditions (`any:`) | — | — | — | ✓ |

### What Remains Uncovered

Two v1 primitives are specified in the schema but not exercised by any test case:

- **Conditional visibility.** A property whose visibility changes based on a condition (e.g., a clue visible only when the player holds a magnifying glass). The condition evaluation engine is tested elsewhere; the risk is in the visibility layer's integration with it. A future test case should exercise this.
- **Spawn effects.** Creating a new entity at runtime. Structurally similar to entity instantiation at load time. A crafting or transformation scenario would exercise it naturally.

Additionally, the **`on_condition` advance mode** is specified (Schema Specification §Advance Modes) but not directly exercised. The Monty Hall test uses `on_action` and `on_rule`; `on_condition` should be tested when a sequence uses condition-based phase advancement.

These gaps are acknowledged in the Schema Specification §v1 Boundaries and Feature Deferrals.

**Guidance for implementing teams:** If your first content does not use conditional visibility, spawn effects, or `on_condition` advancement, you may defer their implementation to a later increment without blocking your initial delivery. However, claiming full v1 runtime compliance requires all three to work correctly. Adding test cases for these primitives before that claim is made is required.

## Static Analysis Tests

Beyond playthrough simulation, the compiler performs static analysis on world files during compilation. These checks catch structural defects without executing the world.

| Check | What It Catches | Example |
|-------|----------------|---------|
| Undefined entity reference | An action or rule references an entity that doesn't exist. | `target: @ghost_key` where no entity `ghost_key` is declared. |
| Type mismatch | A property is set to a value outside its declared type. | `> @guard.mood = happy` where mood is `enum(hostile, neutral, helpful)`. |
| Unreachable location | A location has no exit pointing to it and is not the start location. | A room declared but never connected. |
| Orphaned action | An action whose conditions can never be satisfied given the type constraints. | `? @door.state == locked` where state is `enum(closed, open)`. |
| Duplicate IDs | Two entities, sections, or types share the same identifier. | Two files both declare `@guard`. |
| Missing fallthrough | A dialogue section with only one shot choices and no fallthrough text or terminal jump. | A menu that can exhaust to an empty state. |
| Circular imports | File A imports file B which imports file A. | `tavern.urd.md → harbor.urd.md → tavern.urd.md`. |
| Shadowed exit | A dialogue section name matches an exit name in the same location. | `== harbor` in a location with an exit named `harbor`. |

Static analysis is complementary to playthrough testing. It catches the structural errors; playthroughs catch the semantic ones.

## Test Execution Model

### The Wyrd API Surface for Testing

Tests run against the Wyrd reference runtime using its public API. The `@urd/wyrd-test` package provides assertion helpers on top of the core API.

```javascript
const world = await Wyrd.load('monty-hall.urd.json');

// Scripted playthrough
world.perform('choose_door', { target: 'door_1' });
world.seed(42);                          // Reproducible randomness
const events = world.perform('reveal');   // Monty acts
const state = world.getState();           // Full snapshot

// Monte Carlo
const results = world.simulate({
  runs: 10000,
  seed: 42,
  strategy: 'always_switch',
});
assert(results.winRate > 0.63);
assert(results.winRate < 0.70);

// Dialogue
world.choose('ask_about_harbor');
const choices = world.getChoices();
assert(!choices.find(c => c.id === 'ask_about_missing_ship'));
```

### Execution Contexts

| Context | Runner | Use |
|---------|--------|-----|
| CI | Node.js, headless | Automated regression on every commit. No browser, no UI. |
| IDE | Embedded web view | Writer clicks "Test" in the editor. Results appear in a panel. |
| urd.world | Browser | Interactive demos. The playable examples are the test cases running live. |

### Seeded Randomness

Any test involving randomness (Monty's door selection, NPC behaviour with probabilistic elements) uses `world.seed(n)` to produce deterministic results. A test that passes with seed 42 must pass with seed 42 on every run, on every platform. This is what makes Monte Carlo tests reproducible in CI.

## Relationship to Wyrd Build Passes

The test cases align directly with Wyrd's three pass build plan:

| Pass | Engine Capabilities Added | Acceptance Test |
|------|--------------------------|----------------|
| Pass 1: Core | World state, condition evaluation, effect application, action resolution, sequence phasing. | Test Case 2 (Monty Hall): 10,000 runs, 2/3 switching advantage. |
| Pass 2: Spatial + Presentation | Movement between locations, containment transfer, conditional exits, browser UI. | Test Case 3 (Key Puzzle): interactive playthrough in browser, full escape sequence. |
| Pass 3: Dialogue + Testing | Sections, jumps, sticky/one shot, exhaustion, testing framework. | Test Cases 1 and 4 (Tavern Scene + Interrogation): all dialogue tests pass. |

Each pass produces a working, testable increment. No pass depends on features from a later pass. The test cases are both the acceptance criteria and the demo content.

## The Test Definition Format

The test definitions in this document use a conceptual notation. The actual format will be determined during implementation, but the following design constraints are fixed:

- **Tests are data, not code.** A test definition is a structured JSON document, not a JavaScript file. This keeps tests writable by non engineers and introspectable by tools. Consistent with the rest of the Urd toolchain, YAML is not accepted as a test definition format.
- **Steps are player actions.** Each step in a scripted test is an action the player could perform: choose a door, pick up a key, select a dialogue choice. Tests simulate players, not internal engine operations.
- **Assertions are state queries.** Assertions check the current world state or the most recent event list. They use the same expression syntax as conditions in the schema: `entity.property == value`, `entity.container == other`.
- **Preconditions set up state.** For tests that need specific starting conditions (the bribe test needs a coin purse in the player's inventory), preconditions modify the initial state before the test begins.
- **Comments explain intent.** Comments in test definitions explain why a step is there, not what it does. The step itself is self describing.

*End of Document*
