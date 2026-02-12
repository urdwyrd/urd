---
title: "Urd Schema — Engine Developer Reference Card"
slug: "reference-card-engine-developers"
description: "Quick reference for developers consuming .urd.json files. Covers the eight schema blocks, property types, containment model, visibility, conditions, effects, dialogue structure, and runtime evaluation order."
category: "runtime"
format: "Reference Card"
date: "2026-02"
status: "v1.0"
order: 2
tags:
  - reference
  - engineers
  - schema
  - runtime
  - integration
  - json
details:
  - "Eight schema blocks and their JSON structure"
  - "Property types, containment model, and visibility rules"
  - "Condition expressions, effects, and runtime evaluation order"
---

> **Document status: INFORMATIVE**
> Quick reference card for engine integration developers. All rules are derived from the normative Schema Specification and Wyrd Reference Runtime specification.
> Single canonical copy. February 2026 draft.

# URD

## Engine Developer Reference Card

*Everything you need to consume `.urd.json` files*

urd.dev · February 2026

## The `.urd.json` Contract

A compiled world file is self-contained, deterministic (same source → byte-identical output), versioned, and human-inspectable. It is the single interchange format between authoring and execution.

```
.urd.md files  →  Compiler  →  .urd.json  →  Your Runtime
```

## Eight Top-Level Blocks

All optional except `world`. Blocks can appear in any order. Cross-references resolved by compiler.

| Block | Purpose | Required |
|-------|---------|----------|
| `world` | Metadata: name, version, start location, entry sequence. | **Yes** |
| `types` | Entity type definitions. Property schemas with types, defaults, constraints, visibility. | No |
| `entities` | Instances of defined types. Unique IDs, type reference, property overrides. | No |
| `locations` | Spatial containers with exits and connections. | No |
| `rules` | NPC behavioural constraints. Trigger → condition → select → effect. | No |
| `actions` | Player-performable interactions. Target, prerequisites, effects. | No |
| `sequences` | Ordered phase flows. Game show rounds, tutorial steps, ritual stages. | No |
| `dialogue` | Flat map of dialogue sections, choices, jumps, and `on_exhausted` content. | No |

## The `world` Block

```json
{
  "world": {
    "name": "monty-hall",
    "version": "1.0",
    "urd": "1",
    "description": "The classic Monty Hall problem",
    "author": "Urd Examples",
    "start": "stage",
    "entry": "game"
  }
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | Yes | Lowercase, hyphens allowed. |
| `urd` | string | Yes | Schema version. Always `"1"` for v1. Set by compiler, not author. |
| `version` | string | No | Author-defined version. |
| `description` | string | No | |
| `author` | string | No | |
| `start` | location ref | No | Player's starting location. |
| `entry` | sequence ref | No | Sequence that begins on load. |

## Property Types

| Type | Values | Constraints |
|------|--------|-------------|
| `boolean` | `true` / `false` | |
| `integer` | Whole numbers | Optional `min` / `max` |
| `number` | Decimals | Optional `min` / `max` |
| `string` | Text | |
| `enum` | One of a declared set | Requires `values` array |
| `ref` | Entity ID | Optional `ref_type` for type constraint |
| `list` | Ordered values or refs | |

## Visibility Model

| Level | Who Sees It | Use Case |
|-------|-------------|----------|
| `visible` | Everyone (default) | Door state, character name |
| `hidden` | World only. Must be explicitly `reveal`-ed. | Prize behind a door, trap |
| `owner` | The entity itself and the world | NPC private motivations |
| `conditional` | Visible when a condition is met | Clue visible after examining an object |

Conditional visibility structure:

```json
{
  "secret_message": {
    "type": "string",
    "default": "The key is under the stone",
    "visibility": {
      "type": "conditional",
      "condition": "magnifying_glass.container == player"
    }
  }
}
```

## Entity Traits

Traits are boolean flags on type definitions that inform spatial capabilities.

| Trait | Meaning |
|-------|---------|
| `container` | Can hold other entities. Locations have this implicitly. |
| `portable` | Can be moved into another container (picked up, stored, transferred). |
| `mobile` | Can move itself between containers (walk between rooms). |
| `interactable` | Can be the target of player actions. Default: `true`. |

## The Containment Model

**One spatial primitive.** Every entity exists inside exactly one container. Moving, picking up, dropping, and storing are the same operation: changing `entity.container`.

| Operation | Expression |
|-----------|------------|
| Pick up key | `move: rusty_key, to: player` |
| Drop key in room | `move: rusty_key, to: player.container` |
| Give key to NPC | `move: rusty_key, to: guard` |
| Player has key? | `rusty_key.container == player` |
| Key in same room? | `rusty_key.container == player.container` |

The `player` entity is implicitly a mobile container. No special inventory system needed.

## Locations and Exits

```json
{
  "locations": {
    "cell": {
      "description": "A dim stone cell.",
      "contains": ["rusty_key", "guard", "cell_door"],
      "exits": {
        "north": {
          "to": "corridor",
          "condition": "cell_door.locked == false",
          "blocked_message": "The iron door is locked."
        }
      }
    }
  }
}
```

| Exit Field | Type | Required | Notes |
|------------|------|----------|-------|
| `to` | location ref | Yes | Destination. |
| `condition` | expression | No | Must be true for traversal. |
| `blocked_message` | string | No | Shown when condition is false. |
| `effects` | effect list | No | Applied when exit is used. |

Exits are **unidirectional**. Declare both directions explicitly.

## Condition Expressions

String expressions that evaluate to boolean.

```
entity.property == value       // Equality
entity.property != value       // Inequality
entity.property > value        // Comparison (also <, >=, <=)
entity.container == other      // Containment check
entity.property == true        // Boolean check
```

Multiple conditions in a list are **AND-ed**.

OR logic uses the `any` keyword:

```json
{
  "conditions": {
    "any": [
      "player.reputation > 50",
      "bribe_gold.container == player"
    ]
  }
}
```

> **v1 note:** `any:` is part of the v1 JSON schema and writer syntax (`? any:` block in Schema Markdown). Runtimes must evaluate it.

## Effects

| Effect | JSON Structure | Description |
|--------|----------------|-------------|
| `set` | `{ "set": "entity.prop", "to": value }` | Change a property value. |
| `move` | `{ "move": "entity", "to": "container" }` | Move entity into a container. |
| `reveal` | `{ "reveal": "entity.prop" }` | Change hidden → visible. |
| `destroy` | `{ "destroy": "entity" }` | Remove entity from world. |
| `spawn` | `{ "spawn": { "id": "new_id", "type": "TypeName", "in": "container" } }` | Create entity at runtime. |

There are **no separate inventory verbs**. `move` handles pick up, drop, give, loot, and transfer.

## Rules

Rules define constrained NPC behaviour: trigger → condition → select → effect.

```json
{
  "rules": {
    "monty_reveals": {
      "actor": "monty",
      "trigger": "phase_is reveal",
      "select": {
        "from": ["door_1", "door_2", "door_3"],
        "as": "target",
        "where": [
          "target.prize != car",
          "target.chosen == false",
          "target.state == closed"
        ]
      },
      "effects": [
        { "set": "target.state", "to": "open" }
      ]
    }
  }
}
```

### Trigger Types

| Trigger | Fires When |
|---------|------------|
| `phase_is <id>` | Current sequence reaches the named phase. |
| `action <id>` | The named action is performed. |
| `enter <location>` | An entity enters the named location. |
| `state_change <entity.prop>` | The named property changes value. |
| `always` | Every tick (use sparingly). |

### The `select` Block

Constrained random choice from a set. If multiple candidates match, runtime chooses randomly. If none match, rule does not fire.

| Field | Description |
|-------|-------------|
| `from` | Candidate entity list. |
| `as` | Variable name for the selected entity (usable in effects). |
| `where` | Conditions each candidate must satisfy. |

## Actions

```json
{
  "actions": {
    "pick_up_key": {
      "actor": "player",
      "target": "rusty_key",
      "conditions": ["rusty_key.container == player.container"],
      "effects": [{ "move": "rusty_key", "to": "player" }]
    }
  }
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `actor` | entity ref | No | Default: `player`. |
| `target` | entity ref | No* | Specific entity. |
| `target_type` | type ref | No* | Any entity of this type. |
| `conditions` | expression list | No | |
| `effects` | effect list | Yes | |
| `description` | string | No | Shown to player. |

*Use `target` (specific entity) or `target_type` (any matching entity), not both. Neither = self-targeted.

## Sequences

Ordered phase flows. Worlds without sequences are freeform.

```json
{
  "sequences": {
    "game": {
      "phases": [
        { "id": "choose", "prompt": "Pick a door.", "action": "choose_door", "advance": "on_action" },
        { "id": "reveal", "auto": true, "rule": "monty_reveals", "advance": "on_rule" },
        { "id": "switch_or_stay", "prompt": "Switch or stay?", "actions": ["switch_door", "stay"], "advance": "on_action" },
        { "id": "resolve", "auto": true, "effects": [{"reveal": "door_1.prize"}], "advance": "end" }
      ]
    }
  }
}
```

### Advance Modes

| Mode | Behaviour |
|------|-----------|
| `on_action` | Advance after player completes a listed action. |
| `on_rule` | Advance after the phase's rule fires. |
| `on_condition <expr>` | Advance when expression becomes true. |
| `end` | Sequence ends. |

## Dialogue

The `dialogue` block is a **flat map** of sections keyed by world-unique ID.

```json
{
  "dialogue": {
    "tavern/topics": {
      "id": "tavern/topics",
      "prompt": { "speaker": "arina", "text": "What'll it be, stranger?" },
      "choices": [
        {
          "id": "tavern/topics/ask-about-the-harbor",
          "label": "Ask about the harbor",
          "sticky": true,
          "response": { "speaker": "arina", "text": "Quiet today. Too quiet." },
          "effects": [{ "set": "arina.trust", "to": "arina.trust + 5" }],
          "goto": "tavern/topics"
        }
      ],
      "on_exhausted": { "text": "Suit yourself." }
    }
  }
}
```

### Section Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | string | Yes | Format: `file_stem/section_name`. |
| `prompt` | object | No | `{ speaker, text }` — NPC introduction. |
| `description` | string | No | Prose narration before the prompt. |
| `choices` | array | No | |
| `on_exhausted` | object | No | Content when all choices consumed/gated. `{ text, speaker? }` |

### Choice Fields

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | string | Yes | Format: `section_id/slugified-label`. |
| `label` | string | Yes | Text shown to player. |
| `sticky` | boolean | Yes | `true` = stays; `false` = consumed after selection. |
| `conditions` | expression list | No | |
| `response` | object | No | `{ speaker, text }` |
| `effects` | effect list | No | |
| `goto` | string | No | Section ID to jump to. Omit = stay in section. |
| `choices` | array | No | Inline sub-choices (same structure). |

### Runtime Dialogue Rules

- **One-shot** (`sticky: false`): consumed after selection. Never shown again. Persists across revisits.
- **Sticky** (`sticky: true`): available every visit. Re-evaluate conditions each time.
- **Exhaustion**: all choices consumed or gated → fall through to `on_exhausted`. Never present an empty choice menu.
- **Exhaustion is a runtime predicate**, not a stored boolean. No `exhausted` field in JSON.

## Stable ID Derivation

| Element | Format | Example |
|---------|--------|---------|
| Section | `file_stem/section_name` | `tavern/topics` |
| Choice | `section_id/slugified-label` | `tavern/topics/ask-about-the-harbor` |
| Entity | Declared `@name` | `rusty_key` |

All IDs are world-unique in compiled JSON.

## Runtime Evaluation Order

1. **Load and validate.** Parse, resolve types, validate properties.
2. **Instantiate.** Create entities with defaults. Place in declared containers.
3. **Evaluate sequences.** If `entry` is declared, start its first phase. Otherwise freeform.
4. **Process actions.** Evaluate conditions → apply effects → trigger matching rules.

## Wyrd Runtime Characteristics

Wyrd is the reference runtime. Your integration tests against it.

- **Immutable state transitions.** Each state change produces a new world state.
- **Event sourcing.** Every mutation is a logged event.
- **Seeded randomness.** Deterministic replay from a seed.
- **Canonical behaviour.** Any ambiguity in the spec is resolved by what Wyrd does.

## Schema Coverage by Test Case

| Primitive | Tavern | Monty Hall | Key Puzzle | Interrogation |
|-----------|--------|------------|------------|---------------|
| `world` | ✓ | ✓ | ✓ | ✓ |
| `types` | ✓ | ✓ | ✓ | ✓ |
| `visibility: hidden` | — | ✓ | — | — |
| `visibility: owner` | — | — | ✓ | — |
| `traits: portable` | — | — | ✓ | — |
| `containment as inventory` | — | — | ✓ | ✓ |
| `locations` (multi) | — | — | ✓ | — |
| `exits with conditions` | — | — | ✓ | — |
| `rules with select` | — | ✓ | — | — |
| `actions` | — | ✓ | ✓ | — |
| `sequences` | — | ✓ | — | — |
| `dialogue` (sections, choices) | ✓ | — | — | ✓ |
| `sticky / one-shot` | ✓ | — | — | ✓ |
| `on_exhausted` | ✓ | — | — | ✓ |
| `effects: set` | ✓ | ✓ | ✓ | ✓ |
| `effects: reveal` | — | ✓ | — | — |
| `effects: move` | — | — | ✓ | ✓ |
| `effects: destroy` | — | — | ✓ | — |
| `any:` OR conditions | — | — | — | ✓ |

**Not yet exercised:** conditional visibility, spawn effects. Deferred to future test cases.

## v1 Boundaries

| Feature | Status |
|---------|--------|
| All eight blocks | Supported |
| `any:` OR conditions | Fully supported. JSON `any:` block; writer syntax `? any:`. |
| Cross-file section jumps | Not in v1. Use exits for cross-file movement. |
| Lambda functions | Not in v1. Future extension host. |
| Conditional visibility | Specified but not yet test-covered. |
| Spawn effects | Specified but not yet test-covered. |

*Authoritative sources: Schema Specification, Wyrd Reference Runtime specification. This card is a convenience summary.*

*End of Reference Card*
