---
title: "Urd World Schema Specification v0.1"
slug: "schema-spec"
description: "The formal data contract. Defines entities, typed properties, containment hierarchies, visibility layers, conditional expressions, effects, behavioural rules, dialogue sections, and sequence phases."
category: "contract"
format: "Technical Specification"
date: "2026-02-12"
status: "v0.1 complete"
order: 1
tags:
  - schema
  - specification
  - entities
  - containment
  - visibility
  - dialogue
details:
  - "Entity-property-behaviour model with hidden/revealed state"
  - "Containment as the universal spatial primitive"
  - "Condition expressions and effect grammar"
  - "Rule system: trigger → condition → select → effect"
---

> **Document status: NORMATIVE**
> Defines the Urd World Schema JSON structure. All compiled `.urd.json` output must conform to this specification. This is the authoritative contract between the compiler, runtime, and testing framework.
> Single canonical copy. February 2026 draft.

# URD

## World Schema Specification

*The technical foundation for declarative interactive worlds*

*An open, versioned format for defining entities, locations, behaviors, and narrative structures*

urd.world

February 2026 | Draft

Schema Markdown → JSON (compiled)

## Introduction

This document defines the Urd World Schema, an open format for describing interactive worlds as structured, declarative data. The schema is the foundational layer of the Urd framework. Everything else, the markdown syntax, the compiler, the runtime, the testing tools, the visual editor, consumes or produces instances of this schema.

The schema answers a single question: **what is the minimum structured representation needed to describe an interactive world that a runtime can execute without custom glue code?**

v1 is the complete foundation. It covers entities, types, properties, containment, visibility, locations, exits, sequences, conditions, effects, rules, and dialogue (sections, jumps, sticky/one shot choices, exhaustion). It establishes the extension patterns for everything that follows. Other capabilities not yet specified, faction systems, economic models, multiplayer state, are designed to be addable as schema extensions without breaking backward compatibility.

**Versioning responsibility: the compiler sets the urd field.** Authors do not manually set the schema version. The compiler inspects the compiled output and sets the urd field automatically to `"1"`. If the author manually specifies an urd value in their frontmatter, the compiler emits a warning and overrides it. This removes a category of human error and ensures the version always reflects the compiler's output. Future schema versions will be introduced when new capabilities, such as cross file section jumps or lambda functions, require runtime support that does not exist in v1.

### Format and Notation

The Urd framework has two formats:

- **Schema Markdown** (`.urd.md`): the writer facing syntax. Looks like prose with light annotations. Compiles to JSON. This is what humans write. (Specified in the Schema Markdown Syntax Specification.)
- **JSON** (`.urd.json`): the compiled output. This is what runtimes consume. Machine readable, deterministic, engine agnostic.

The authoring path is: **Schema Markdown → compiler → JSON.** There is no intermediate format. Writers author in `.urd.md` files. The compiler produces `.urd.json` files. Runtimes, testing tools, and engine integrations consume the JSON.

**About the notation in this document.** The examples in this specification use a structured notation that resembles YAML because it is human readable and supports comments, making it well suited for documenting structured data. This is a documentation choice, not a format declaration. The examples here illustrate the JSON structure in a readable form; the actual JSON output follows the same structure without comments.

**JSON is the only normative contract.** The Urd schema is defined in terms of its JSON structure. The notation in this document is non normative. Any ambiguity between the notation used here and the corresponding JSON structure is resolved in favor of the JSON.

### Urd Frontmatter

Schema Markdown files use a structured block between `---` delimiters for metadata, type definitions, entity declarations, and imports. This block is called **Urd frontmatter**. It is parsed by the Urd compiler, not by a general purpose YAML parser.

Urd frontmatter is **YAML like but not YAML.** It uses the same key value and indentation conventions, but operates under strict constraints. Writers and engineers familiar with YAML will find it immediately readable; the constraints exist to prevent the well documented class of YAML bugs (implicit type coercion, anchors, aliases, custom tags) that cause subtle failures in content pipelines.

#### Frontmatter Grammar

- **Delimited by `---` on its own line.** The opening `---` must be the first line of the file. The closing `---` ends the frontmatter block. Everything after it is narrative content.
- **Key value pairs.** `key: value`. Keys are lowercase with underscores. Values are strings, integers, floats, booleans (`true` / `false`), or `null`.
- **Nested blocks via indentation.** Two space indent per level. Used for type properties and entity declarations.
- **Inline object shorthand.** `@arina: Barkeep { name: "Arina" }`. Curly braces for single line entity property overrides.
- **Lists.** Square brackets: `[goat, car]`, `[@door_1, @door_2]`. Flow style only; block style YAML lists (`- item`) are not used in frontmatter.
- **Strings must be quoted when ambiguous.** Version numbers (`"1"`), values that look like booleans (`"yes"`, `"no"`, `"true"`), and values starting with special characters require double quotes.
- **Comments.** `# text` at end of line or on its own line.

#### What Is Not Allowed

- **No anchors or aliases.** `&anchor` and `*alias` are not supported. Every value must be written explicitly.
- **No merge keys.** `<<:` is not supported.
- **No custom tags.** `!!python/object` or any tag syntax is not supported.
- **No implicit type coercion.** The string `NO` is the string `"NO"`, not boolean false. The compiler uses explicit type information from the schema, not YAML 1.1 inference rules.
- **No block style lists.** Use `[a, b, c]`, not `- a` / `- b` / `- c`.

**Error messages are educational.** When a writer uses a disallowed YAML feature, the compiler says: *"Anchors are not supported in Urd frontmatter (line 5). Write the value explicitly instead."* Not a generic parse error.

### Design Principles

**Declarative, not imperative.** The schema describes what the world *is*, not what it *does*. A door is locked. A guard reveals information under certain conditions. The runtime decides how and when to evaluate these declarations. This is what enables emergent behavior: outcomes arise from structure, not scripted sequences.

**Separation of content and presentation.** The schema contains no rendering instructions. No pixel coordinates, no audio references, no UI layouts. It describes entities, properties, spatial relationships, and behavioral rules. How these get presented, as text, 2D art, 3D environments, or voice driven interaction, is entirely the runtime's concern.

**Containment as the universal spatial primitive.** A room holds entities. A chest holds entities. A player's pockets hold entities. These are the same operation. The schema has one spatial concept, containers that hold things, rather than separate mechanics for locations, inventory, and storage. This is discussed in detail in the Containment Model section.

**Visibility as a first class concept.** Interactive worlds depend on information asymmetry. The player knows some things. The world knows everything. NPCs know subsets. The schema's property visibility model makes this explicit rather than leaving it to runtime hacks.

**AI native by design.** Every element has a typed, unambiguous representation. An AI coding assistant reading a schema file can determine what entities exist, what their valid states are, what conditions gate which behaviors, and what state mutations are possible. This is a formal contract, not documentation.

**Extensible without breakage.** New capabilities are added as new top level blocks or new property types. Existing schema files remain valid as the spec evolves. A v1 file loaded by a later runtime simply lacks the newer features.

## Schema Structure

An Urd world file is a document with the following top level blocks. All blocks are optional except `world`.

```
world:       # Required. Metadata and configuration.
types:       # Entity type definitions with property schemas.
entities:    # Instances of defined types.
locations:   # Spatial containers with exits and connections.
rules:       # Behavioral constraints governing entity actions.
actions:     # Interactions available to the player or entities.
sequences:   # Ordered event flows (game phases, scenes).
dialogue:    # Dialogue sections, choices, jumps, and on_exhausted content.
```

The blocks can appear in any order. Cross references between blocks are resolved by the compiler or runtime, not by declaration order.

### Evaluation Order

The runtime evaluates a world file in the following conceptual order, though implementations may optimize:

1. **Load and validate.** Parse the file, resolve type references, validate property types and constraints.
2. **Instantiate.** Create entity instances with default property values. Place entities in their declared containers (locations or other entities).
3. **Evaluate sequences.** If a sequence is declared, begin executing its first phase. If no sequence exists, the world is freeform (sandbox mode).
4. **Process actions.** When an actor (player or NPC) performs an action, evaluate its conditions, apply its effects, and trigger any matching rules.

## The `world` Block

Metadata about the world. The only required block in a schema file.

```
world:
  name: monty-hall
  version: "1.0"
  urd: "1"
  description: "The classic Monty Hall problem"
  author: "Urd Examples"
  start: stage       # Starting location
  entry: game        # Starting sequence
  seed: 42           # Optional. Deterministic replay seed.
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | Yes | Unique identifier. Lowercase, hyphens allowed. |
| version | string | No | Author defined version string for this world file. |
| urd | string | Yes | Schema version this file targets. `"1"` for this spec. |
| description | string | No | Human readable description of the world. |
| author | string | No | Author or team name. |
| start | location ref | No | The location where the player begins. |
| entry | sequence ref | No | The sequence that begins on world load. |
| seed | integer | No | Random seed for deterministic replay. If omitted, the runtime generates one. |

### Determinism Contract

Any element of the schema that involves non deterministic choice (the `select` block in rules, any future randomness source) is governed by the following normative requirements:

1. **Seed source.** The `world.seed` field in compiled JSON provides the initial seed. If absent, the runtime generates a seed and records it in the first event. The runtime API exposes `world.seed(n)` to set the seed before execution begins.

2. **Deterministic replay guarantee.** Given identical compiled JSON, identical seed, and identical player actions in identical order, the resulting event stream MUST be identical across runs and across conforming runtime implementations. This is the fundamental testability contract.

3. **Selection algorithm.** When a `select` block matches multiple candidates, selection is **uniform random** from the matching set. Declaration order in the `from` list MUST NOT influence selection probability. Declaration order MAY be used only as a tiebreaker when the random algorithm requires one (e.g., when two candidates hash identically under the seeded generator), but this is an implementation detail, not a semantic guarantee.

4. **Seed propagation.** Each random selection consumes one value from the seeded sequence. The runtime MUST consume random values in a deterministic order: rules are evaluated in declaration order within the `rules` block, and each rule's `select` consumes exactly one random value if and only if multiple candidates match. If exactly one candidate matches, no random value is consumed. If zero match, the rule does not fire and no random value is consumed.

## The `types` Block

Type definitions describe categories of entities. Each type declares a set of typed properties with defaults, constraints, and visibility rules. Types are the schema's contract: they tell both the runtime and AI tools exactly what an entity can contain.

```
types:
  Door:
    description: "A door that conceals a prize"
    properties:
      prize:
        type: enum
        values: [goat, car]
        visibility: hidden
      state:
        type: enum
        values: [closed, open]
        default: closed
        visibility: visible
      chosen:
        type: boolean
        default: false
        visibility: visible
```

### Property Schema

Each property in a type definition has the following fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| type | string | Yes | Data type. See Property Types below. |
| default | any | No | Default value on instantiation. Must match type. |
| visibility | enum | No | Who can see this. Default: visible. |
| values | array | Conditional | Required for enum type. The valid value set. |
| min / max | number | No | Range constraints for integer and number types. |
| ref_type | string | No | For ref type: the entity type this must reference. |
| description | string | No | Human readable explanation of this property. |

### Property Types

| Type | Values | Example |
|------|--------|---------|
| boolean | true / false | `locked: true` |
| integer | Whole numbers. Optional min/max. | `trust: 50` (min: 0, max: 100) |
| number | Decimal numbers. Optional min/max. | `weight: 3.5` |
| string | Text. | `name: "Rusty Key"` |
| enum | One of a declared set of values. | `prize: goat` (values: [goat, car]) |
| ref | Reference to another entity by ID. | `requires: rusty_key` |
| list | Ordered list of values or refs. | `tags: [metal, small]` |

### Visibility Model

The visibility field controls information asymmetry, the core mechanism that makes interactive worlds interesting. It determines who can observe a property's value at any given moment.

| Visibility | Who Sees It | Use Case |
|------------|-------------|----------|
| visible | Everyone: player, NPCs, all systems. | Door state (open/closed), character name. |
| hidden | The world only. Not revealed unless an action or rule explicitly reveals it. | Prize behind a door, trap status. |
| owner | The entity itself and the world. Others cannot see it unless revealed. | NPC private motivations, secret knowledge. |
| conditional | Visible only when a condition is met. | A clue visible after examining an object. |

Conditional visibility is declared inline with the property:

```
secret_message:
  type: string
  default: "The key is under the stone"
  visibility:
    type: conditional
    condition: "magnifying_glass.container == player"
```

## The Containment Model

**Core insight:** A room holds a sword. A chest holds a sword. A player's pockets hold a sword. There is no reason these should be three different mechanisms.

Urd uses a single spatial primitive: containment. Every entity exists inside exactly one container at any given time. A container is any entity or location with the container trait. This replaces the conventional split between "locations" (where entities are), "inventory" (what the player carries), and "storage" (what objects hold inside them) with one unified model.

### How It Works

Every entity has an implicit container property, a reference to whatever currently holds it. When the world starts, entities are placed in their declared containers (the locations block's entities list, or a parent entity). Moving, picking up, dropping, and storing are all the same operation: changing an entity's container.

```
# A player picks up a key
effects:
  - move: rusty_key
    to: player              # The player IS the container now

# A player puts the key in a chest
effects:
  - move: rusty_key
    to: chest

# A player drops the key in the current room
effects:
  - move: rusty_key
    to: player.container    # The room the player is in

# Checking whether the player has the key
conditions:
  - "rusty_key.container == player"

# Checking whether the key is in the same room as the player
conditions:
  - "rusty_key.container == player.container"
```

### Entity Traits

Traits are boolean flags that inform the runtime about an entity's spatial capabilities:

| Trait | Meaning |
|-------|---------|
| container | This entity can hold other entities inside it. Locations have this implicitly. |
| portable | This entity can be moved into another container (picked up, stored, transferred). |
| mobile | This entity can move itself between containers (walk between rooms). |
| interactable | This entity can be the target of player actions. Default: true. |

```
types:
  Key:
    traits: [portable]
    properties:
      name: { type: string }

  Chest:
    traits: [container, interactable]
    properties:
      locked: { type: boolean, default: true }
      requires: { type: ref, ref_type: Key }

  Guard:
    traits: [mobile, container, interactable]
    properties:
      name: { type: string }
      mood: { type: enum, values: [hostile, neutral, helpful] }
```

Notice that the Guard has both mobile (can move between rooms) and container (can carry things). An NPC's equipment, possessions, or secret items are just entities inside that NPC's container. Searching an NPC, looting a fallen enemy, or giving someone a gift are all container to container transfers.

### The Player as Container

The player entity is implicitly a mobile container. It does not need special inventory mechanics. "The player has the key" is expressed as `rusty_key.container == player`. "The player is in the cell" is expressed as `player.container == cell`. "What's in the player's pockets?" is a query for all entities whose container is the player.

The player entity is always available and doesn't need to be declared (though it can be, to add custom properties):

```
entities:
  player:
    type: Player
    properties:
      health: 100
      reputation: 0
```

**Player entity resolution rules:**

1. If no explicit `@player` entity is declared, the runtime creates one with an implied type `Player` and default spatial properties (mobile container). Its starting container is the `start` location declared in the `world` block.
2. If an explicit `@player` entity is declared, it **replaces** the implicit one entirely. No merging. The declared type must have the mobile and container traits.
3. Duplicate `@player` declarations across imported files are a compile error, following the same rule as any duplicate entity ID.

### Why This Matters

The unified containment model has three practical benefits:

- **Fewer concepts to learn.** Instead of separate systems for rooms, inventory, and storage, there is one mechanism. A writer who understands "move X into Y" understands all spatial operations.
- **Emergent interactions.** Because everything uses the same primitive, combinations that weren't explicitly designed just work. An NPC can carry a locked chest that contains a map. The player can steal the chest. The map is still inside it. No special case needed.
- **Simpler conditions and effects.** The expression language needs one containment check (`X.container == Y`) and one spatial operation (`move: X, to: Y`) instead of separate verbs for pick up, drop, store, equip, give, loot, and transfer.

## The `entities` Block

Entities are instances of types. Each entity has a unique ID, a type reference, and optionally overridden property values. If a property is not specified, it takes its type's default.

```
entities:
  door_1:
    type: Door
    properties:
      prize: car

  door_2:
    type: Door
    properties:
      prize: goat

  door_3:
    type: Door
    properties:
      prize: goat

  monty:
    type: Host
    properties:
      name: "Monty Hall"
```

Entity IDs must be unique within a world file, composed of lowercase letters, digits, and underscores. The ID is the handle used everywhere else in the schema.

## The `locations` Block

Locations are top level containers: the rooms, areas, and zones that make up the world's geography. Under the unified containment model, a location is simply a container that has exits to other containers. The locations block is syntactic sugar that makes spatial structure easy to read; under the hood, locations are entities with the container trait and an exits map.

```
locations:
  cell:
    description: "A dim stone cell with damp walls."
    contains: [rusty_key, guard]
    exits:
      north:
        to: corridor
        condition: "cell_door.locked == false"
        blocked_message: "The door is locked."

  corridor:
    description: "A long corridor stretching into darkness."
    contains: [cell_door]
    exits:
      south:
        to: cell
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| description | string | No | Human readable description shown to the player. |
| contains | list of refs | No | Entities inside this location at world start. |
| exits | map | No | Named exits leading to other locations. |
| on_enter | effect list | No | Effects triggered when an entity enters. |
| on_exit | effect list | No | Effects triggered when an entity leaves. |

Note the field name is `contains`, not `entities`, consistent with the containment model vocabulary.

### Exit Structure

Each exit is a named key (`north`, `south`, `door`, `stairway`, any string) mapping to a destination and optional gate:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| to | location ref | Yes | The destination location ID. |
| condition | expression | No | Must be true for traversal. |
| blocked_message | string | No | Shown when the condition is false. |
| effects | effect list | No | Effects applied when the exit is used. |

Exits are unidirectional by default. If a corridor connects to a cell, the cell must also declare an exit back. This is intentional: one way passages, trapdoors, and asymmetric connections are common in interactive worlds.

## The `rules` Block

Rules define constrained NPC behavior. A rule says: when a trigger occurs, if certain conditions are met, an entity performs a set of effects. Rules give NPCs autonomy within bounds: the schema says what they can and must do, not a script that controls every step.

```
rules:
  monty_reveals:
    description: "Monty opens a door with a goat"
    actor: monty
    trigger: phase_is reveal
    select:
      from: [door_1, door_2, door_3]
      as: target
      where:
        - "target.prize != car"
        - "target.chosen == false"
        - "target.state == closed"
    effects:
      - set: target.state
        to: open
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| actor | entity ref | Yes | The entity performing the action. |
| trigger | string | Yes | What activates this rule. See Trigger Types. |
| description | string | No | Human readable explanation. |
| conditions | expression list | No | Additional conditions beyond select filters. |
| select | select block | No | Dynamically choose a target from a set. |
| effects | effect list | Yes | State changes the rule produces. |

### The `select` Block

The select block enables constrained choice, the mechanism that makes Monty Hall work. It says: from a set of entities, choose one matching all where conditions. If multiple match, the runtime chooses randomly. If none match, the rule does not fire.

| Field | Type | Description |
|-------|------|-------------|
| from | list of refs | Candidate entities to select from. |
| as | string | Variable name for the selected entity, usable in effects. |
| where | expression list | Conditions each candidate must satisfy. |

### Trigger Types

| Trigger | Fires When |
|---------|------------|
| phase_is \<id\> | The current sequence reaches the named phase. |
| action \<id\> | The named action is performed. |
| enter \<location\> | An entity enters the named location. |
| state_change \<entity.prop\> | The named property changes value. |
| always | Evaluated every tick (use sparingly). |

## The `actions` Block

Actions are interactions that players (or entities) can perform. An action has a target, prerequisites, and effects. Actions are the verbs of the world.

```
actions:
  pick_up_key:
    description: "Pick up the rusty key"
    actor: player
    target: rusty_key
    conditions:
      - "rusty_key.container == player.container"
    effects:
      - move: rusty_key
        to: player

  unlock_door:
    description: "Use the key on the door"
    actor: player
    target: cell_door
    conditions:
      - "rusty_key.container == player"
      - "cell_door.locked == true"
    effects:
      - set: cell_door.locked
        to: false
      - destroy: rusty_key
```

Notice how containment unifies the language. "Pick up" is `move: key, to: player`. "Is the key in the room?" is `key.container == player.container`. "Does the player have the key?" is `key.container == player`. One spatial primitive, no special inventory verbs.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| actor | entity ref | No | Who performs the action. Default: player. |
| target | entity ref | No\* | A specific entity this action targets. |
| target_type | type ref | No\* | Any entity of this type can be the target. |
| conditions | expression list | No | Conditions that must be true to perform. |
| effects | effect list | Yes | State changes the action produces. |
| description | string | No | Shown to the player as the action label. |

\* Declare either `target` (specific entity) or `target_type` (any matching entity), not both. Actions with neither are self targeted (e.g., rest, wait).

## The `sequences` Block

Sequences define ordered flows of phases. They give structure to experiences that have a defined progression: a game show with rounds, a tutorial with steps, a ritual with stages. Worlds without sequences are **freeform**: the player can perform any available action at any time, with progression emerging from action conditions and the containment model rather than from an imposed phase order. The Two Room Key Puzzle test case is a freeform world.

```
sequences:
  game:
    description: "The Monty Hall game flow"
    phases:
      - id: choose
        prompt: "Pick a door."
        action: choose_door
        advance: on_action

      - id: reveal
        auto: true
        rule: monty_reveals
        advance: on_rule

      - id: switch_or_stay
        prompt: "Monty opened a door with a goat. Switch or stay?"
        actions: [switch_door, stay]
        advance: on_action

      - id: resolve
        auto: true
        effects:
          - reveal: "door_1.prize"
          - reveal: "door_2.prize"
          - reveal: "door_3.prize"
        advance: end
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Yes | Unique phase identifier within the sequence. |
| prompt | string | No | Text shown to the player at this phase. |
| auto | boolean | No | If true, executes without player input. |
| action / actions | ref / list | No | Action(s) available during this phase. |
| rule | rule ref | No | A rule that fires automatically in this phase. |
| effects | effect list | No | Effects applied when this phase begins. |
| advance | string | No | When to move to the next phase. See below. |
| condition | expression | No | Phase is skipped if this evaluates to false. |

### Advance Modes

| Mode | Behavior |
|------|----------|
| on_action | Advance after the player completes a listed action. |
| on_rule | Advance after the phase's rule fires. |
| on_condition \<expr\> | Advance when the expression becomes true. |
| end | The sequence ends. No further phases. |

## The `dialogue` Block

Dialogue sections compile to a flat map of section objects, keyed by globally unique section ID. The runtime uses this block to drive all conversational interaction. Each section represents a single conversational node: an optional prompt, a set of choices, and fallthrough content for exhaustion.

```
dialogue:
  "tavern/topics":
    id: "tavern/topics"
    prompt:
      speaker: "arina"
      text: "What'll it be, stranger?"
    choices:
      - id: "tavern/topics/ask-about-the-harbor"
        label: "Ask about the harbor"
        sticky: true
        response:
          speaker: "arina"
          text: "Quiet today. Too quiet, if you ask me."
        effects:
          - set: arina.trust
            to: arina.trust + 5
        goto: "tavern/topics"
      - id: "tavern/topics/ask-about-the-missing-ship"
        label: "Ask about the missing ship"
        sticky: false
        conditions:
          - "arina.trust > 50"
        response:
          speaker: "arina"
          text: "The Selene didn't sink. She was taken."
        goto: "tavern/topics"
    on_exhausted:
      text: "Suit yourself. I've got glasses to clean."

  "tavern/bribe":
    id: "tavern/bribe"
    conditions:
      - "coin_purse.container == player"
    choices:
      - id: "tavern/bribe/slide-the-purse"
        label: "Slide the purse across the bar"
        sticky: false
        effects:
          - move: coin_purse
            to: arina
          - set: arina.trust
            to: 100
        response:
          speaker: "arina"
          text: "Well now. That changes things."
        goto: "tavern/topics"
```

The dialogue block is a flat map. All sections from all files in the compilation unit are merged into a single namespace. Section IDs use the `file_stem/section_name` convention (e.g., `tavern/topics` for `== topics` in `tavern.urd.md`). Choice IDs use `section_id/slugified_label` (e.g., `tavern/topics/ask-about-the-harbor`).

### Section Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Yes | World-unique section identifier. Format: `file_stem/section_name`. |
| prompt | object | No | NPC speech that introduces this section. Contains `speaker` (entity ref) and `text` (string). |
| description | string | No | Prose narration before the prompt. Compiled from plain text at the start of a section, before any `@speaker:` line. |
| choices | array | No | List of available choices in this section. |
| conditions | expression list | No | Conditions that must be true for the section to be accessible. Not authored in v1 Schema Markdown; reserved for future use. May appear in hand-authored or tool-generated JSON. |
| on_exhausted | object | No | Content shown when all choices are consumed or gated. Contains `text` (string) and optionally `speaker` (entity ref). This is a content payload, not a boolean. Whether a section *is* exhausted is a runtime-evaluated predicate. |

### Choice Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Yes | World-unique choice ID. Format: `section_id/slugified-label`. |
| label | string | Yes | Text shown to the player for this choice. |
| sticky | boolean | Yes | If `true`, choice remains available after selection. If `false`, consumed after one selection. |
| conditions | expression list | No | Conditions that must be true for this choice to appear. |
| response | object | No | Dialogue spoken when this choice is selected. Same structure as section `prompt`. |
| effects | effect list | No | State changes applied when this choice is selected. |
| goto | string | No | Section ID to jump to after this choice. If omitted, stays in current section. |
| choices | array | No | Inline sub-choices. Same structure as top-level choices. |

The `sticky` field maps directly to the Schema Markdown choice syntax: `+` (sticky) compiles to `true`, `*` (one-shot) compiles to `false`. The `goto` field compiles from `->` jumps in Schema Markdown and always uses the full section ID.

**Normative rule: exhaustion is never stored.** A section's exhausted state is never persisted in world state or compiled JSON. It is recomputed on every evaluation by checking all choices in the named section: if every choice is either consumed (one-shot, already selected) or gated (conditions evaluate to false), the section is exhausted. The compiled JSON contains no `exhausted` field. The `on_exhausted` field contains fallthrough content, not a boolean. Runtimes MUST compute exhaustion as a predicate, not read it from state.

## Expressions and Effects

The schema uses a minimal expression language for conditions and a structured effect format for state mutations. Both are deliberately simple: parseable by runtimes, lintable by tools, and interpretable by AI assistants.

### Condition Expressions

Conditions are string expressions that evaluate to true or false:

```
# Comparison
entity.property == value
entity.property != value
entity.property > value

# Containment (the universal spatial check)
entity.container == other_entity
entity.container == location_id

# Boolean
entity.property == true

# Multiple conditions in a list are AND-ed
conditions:
  - "cell_door.locked == true"
  - "rusty_key.container == player"
```

For OR logic, use the `any` keyword:

```
conditions:
  any:
    - "player.reputation > 50"
    - "bribe_gold.container == player"
```

> **v1 scope.** The `any:` construct is part of the v1 JSON schema and the v1 Schema Markdown syntax. Writers author OR conditions using `? any:` followed by indented conditions. Runtimes must evaluate `any:` blocks correctly. See the Schema Markdown Syntax Specification for the writer-facing syntax.

### Effect Declarations

Effects are structured state mutations:

| Effect | Syntax | Description |
|--------|--------|-------------|
| set | `set: entity.prop, to: value` | Change a property to a new value. |
| move | `move: entity, to: container` | Move an entity into a different container. |
| reveal | `reveal: entity.prop` | Change a hidden property's visibility to visible. |
| destroy | `destroy: entity` | Remove an entity from the world. |
| spawn | `spawn: { id: new_id, type: TypeName, in: container }` | Create a new entity at runtime. |

**Note:** There are no separate add/remove effects for inventory. `move: key, to: player` is "pick up." `move: key, to: cell` is "drop." The containment model eliminates the need for list manipulation verbs.

**The `here` alias.** The keyword `here` resolves to `player.container` at evaluation time. It is valid in both condition expressions and effect declarations. In conditions, `entity.container == player.container` can be written as `entity in here` in Schema Markdown. In effects, `move: entity, to: player.container` can be written as `> move @entity -> here` in Schema Markdown. The compiled JSON always uses the expanded form (`player.container`).

## Complete Example: The Monty Hall Problem

A complete, valid Urd schema file demonstrating hidden state, constrained NPC behavior, sequential phases, and emergent probability from declarative structure.

```
world:
  name: monty-hall
  urd: "1"
  description: "The classic Monty Hall problem."
  start: stage
  entry: game

types:
  Door:
    traits: [interactable]
    properties:
      prize:
        type: enum
        values: [goat, car]
        visibility: hidden
      state:
        type: enum
        values: [closed, open]
        default: closed
      chosen:
        type: boolean
        default: false

  Host:
    properties:
      name: { type: string }

entities:
  door_1: { type: Door, properties: { prize: car } }
  door_2: { type: Door, properties: { prize: goat } }
  door_3: { type: Door, properties: { prize: goat } }
  monty: { type: Host, properties: { name: "Monty Hall" } }

locations:
  stage:
    description: "A game show stage with three closed doors."
    contains: [door_1, door_2, door_3, monty]

rules:
  monty_reveals:
    description: "Monty opens a door that hides a goat."
    actor: monty
    trigger: phase_is reveal
    select:
      from: [door_1, door_2, door_3]
      as: target
      where:
        - "target.prize != car"
        - "target.chosen == false"
        - "target.state == closed"
    effects:
      - set: target.state
        to: open

actions:
  choose_door:
    description: "Pick a door"
    target_type: Door
    conditions:
      - "target.state == closed"
      - "target.chosen == false"
    effects:
      - set: target.chosen
        to: true

  switch_door:
    description: "Switch to the other closed door"
    target_type: Door
    conditions:
      - "target.state == closed"
      - "target.chosen == false"
    effects:
      - set: target.chosen
        to: true

  stay:
    description: "Stay with your current choice"
    effects: []

sequences:
  game:
    phases:
      - id: choose
        prompt: "Pick a door."
        action: choose_door
        advance: on_action

      - id: reveal
        auto: true
        rule: monty_reveals
        advance: on_rule

      - id: switch_or_stay
        prompt: "Monty opened a door with a goat. Switch or stay?"
        actions: [switch_door, stay]
        advance: on_action

      - id: resolve
        auto: true
        effects:
          - reveal: door_1.prize
          - reveal: door_2.prize
          - reveal: door_3.prize
        advance: end
```

**What to notice:** No probability is specified anywhere. The 2/3 switching advantage emerges from the structure. Monty's select constraint filters out the car door and the player's door, forcing him to reveal a goat. The math is an emergent property of a correct world definition.

## Complete Example: The Two Room Key Puzzle

This example exercises spatial navigation, containment as inventory, persistent state changes, and conditional NPC interaction: everything the Monty Hall example does not. Note the absence of a sequences block: this world is freeform, with progression emerging from action conditions.

```
world:
  name: two-room-key
  urd: "1"
  description: "Find the key, unlock the door, escape."
  start: cell

types:
  Key:
    traits: [portable]
    properties:
      name: { type: string }

  LockedDoor:
    traits: [interactable]
    properties:
      locked: { type: boolean, default: true }
      requires: { type: ref, ref_type: Key }

  Guard:
    traits: [interactable, mobile, container]
    properties:
      name: { type: string }
      mood:
        type: enum
        values: [hostile, neutral, helpful]
        default: hostile
      hint_given:
        type: boolean
        default: false
        visibility: owner

entities:
  rusty_key:
    type: Key
    properties: { name: "Rusty Key" }

  cell_door:
    type: LockedDoor
    properties: { requires: rusty_key }

  guard:
    type: Guard
    properties: { name: "Halvard" }

locations:
  cell:
    description: "A dim stone cell. A guard watches from the corner."
    contains: [rusty_key, guard, cell_door]
    exits:
      north:
        to: corridor
        condition: "cell_door.locked == false"
        blocked_message: "The iron door is locked."

  corridor:
    description: "A long corridor. Daylight leaks from the far end."
    exits:
      south: { to: cell }

actions:
  offer_patience:
    description: "Wait quietly and show respect"
    target: guard
    conditions:
      - "guard.mood == hostile"
    effects:
      - set: guard.mood
        to: neutral

  talk_to_guard:
    description: "Talk to the guard"
    target: guard
    conditions:
      - "guard.mood == neutral"
      - "guard.hint_given == false"
    effects:
      - set: guard.hint_given
        to: true

  pick_up_key:
    description: "Pick up the rusty key"
    target: rusty_key
    conditions:
      - "rusty_key.container == player.container"
    effects:
      - move: rusty_key
        to: player

  unlock_door:
    description: "Use the key on the door"
    target: cell_door
    conditions:
      - "rusty_key.container == player"
      - "cell_door.locked == true"
    effects:
      - set: cell_door.locked
        to: false
      - destroy: rusty_key
```

**What to notice:** Containment replaces inventory throughout. "Pick up the key" is `move: rusty_key, to: player`. "Does the player have the key?" is `rusty_key.container == player`. The guard's interaction arc (hostile → neutral → hint) emerges from conditions gating what's available. No sequence block needed: the puzzle's structure comes from the dependency chain in the conditions.

## Schema Coverage Matrix

The two validation test cases exercise complementary schema capabilities:

| Schema Concept | Monty Hall | Key Puzzle | Section |
|----------------|------------|------------|---------|
| world (metadata) | ✓ | ✓ | world block |
| types (property schemas) | ✓ | ✓ | types block |
| visibility: hidden | ✓ | — | Visibility Model |
| visibility: owner | — | ✓ | Visibility Model |
| traits: portable | — | ✓ | Containment Model |
| traits: container | — | ✓ (Guard) | Containment Model |
| traits: interactable | ✓ | ✓ | Containment Model |
| entities (instances) | ✓ | ✓ | entities block |
| player (implicit) | ✓ | ✓ | Player as Container |
| containment as inventory | — | ✓ | Containment Model |
| locations | ✓ (single) | ✓ (two) | locations block |
| exits with conditions | — | ✓ | Exit Structure |
| rules with select | ✓ | — | rules block |
| actions with conditions | ✓ | ✓ | actions block |
| effects: set | ✓ | ✓ | Effects |
| effects: reveal | ✓ | — | Effects |
| effects: move (spatial) | — | ✓ | Effects |
| effects: destroy | — | ✓ | Effects |
| sequences (phases) | ✓ | — | sequences block |
| freeform (no sequence) | — | ✓ | sequences block |

Together, the two examples cover every v1 primitive except conditional visibility and spawn effects, which are specified but deferred to future validation test cases.

## v1 Boundaries and Feature Deferrals

v1 is the complete foundation. This section defines what "v1 compliant" means and what is explicitly deferred.

### What v1 Includes

A v1 runtime MUST implement all of the following:

- All eight top-level blocks: `world`, `types`, `entities`, `locations`, `rules`, `actions`, `sequences`, `dialogue`.
- All property types: `boolean`, `integer`, `number`, `string`, `enum`, `ref`, `list`.
- All four visibility levels: `visible`, `hidden`, `owner`, `conditional`.
- All entity traits: `container`, `portable`, `mobile`, `interactable`.
- The containment model: one spatial primitive, `move` as the sole spatial operation, implicit `player` entity with resolution rules.
- All five effect types: `set`, `move`, `reveal`, `destroy`, `spawn`.
- All five trigger types: `phase_is`, `action`, `enter`, `state_change`, `always`.
- The `select` block with uniform random selection and the determinism contract (§Determinism Contract).
- All four advance modes: `on_action`, `on_rule`, `on_condition`, `end`.
- Dialogue: sections, sticky and one-shot choices, `goto` jumps, `on_exhausted` fallthrough, exhaustion as a runtime predicate.
- `any:` OR conditions.
- Event sourcing: every state mutation produces a typed event.

### What v1 Does NOT Include

The following capabilities are explicitly deferred. v1 runtimes MUST NOT implement them. v1 worlds MUST NOT use them. They are documented in the Future Proposals document for forward-compatible architectural planning only.

| Feature | Status | v1 Workaround |
|---------|--------|---------------|
| Cross-file section jumps | Deferred. `->` jumps are file-scoped. | Use location exits for cross-file movement. Use bridge sections for shared dialogue context. Entity state communicates across files; section exhaustion does not. |
| Lambda functions | Deferred. Extension host slot exists in architecture but is empty. | Express logic declaratively using rules, conditions, and effects. |
| Owner visibility full semantics | Partially specified. `owner` level works. Ownership transfer is not specified. | Use `owner` for static ownership (NPC private knowledge). Do not design content requiring ownership transfer. |
| Cross-file exhaustion sharing | Not specified. Each file tracks exhaustion independently. | Accept independent exhaustion counters in duplicated bridge sections. |

**Content workflow implication:** Do not design content workflows around cross-file dialogue jumps. The v1 content model assumes one file per location or scene, with `import:` for shared types and entities and exits for navigation between locations.

### Test Coverage Notes

Two v1 primitives are specified above but not yet exercised by the current test suite:

- **Conditional visibility.** Specified in §Visibility Model. The condition evaluation engine is tested elsewhere; the integration risk is in the visibility layer.
- **Spawn effects.** Specified in §Effect Declarations. Structurally similar to entity instantiation at load time.

Implementing teams whose first content does not use these may defer their implementation to a later increment. Full v1 compliance requires both to work. The `on_condition` advance mode is also specified (§Advance Modes) but not yet covered by a test case.

## Schema Roadmap: Beyond v1

v1 is the complete foundation: entities, types, properties, containment, visibility, locations, exits, sequences, conditions, effects, rules, and dialogue. Future capabilities are addable as new blocks or extensions without breaking existing world files.

### Future: Relationships, Knowledge, and Extended Logic

- **Relationships.** Typed connections between entities (ally_of, hostile_to, employed_by) that rules and conditions can reference.
- **Knowledge model.** What each entity knows. An NPC who witnessed an event can report it; one who didn't, cannot.
- **Cross file section jumps.** Dialogue sections reachable across file boundaries, with shared state semantics.
- **Lambda functions.** An extension host for imperative logic (pathfinding, economic calculations, procedural generation) sandboxed within the world model.

### Future: Time, Scheduling, and Events

- **Time system.** A world clock with configurable granularity. Time gated events, NPC schedules, day/night cycles.
- **Scheduled rules.** Rules that trigger at specific times or intervals.
- **Event log.** A persistent record of what has happened, enabling NPCs to react to history.

### Future: Simulation Layer

- **Numeric systems.** Economy, health, reputation as typed subsystems with flow rules.
- **Faction system.** Groups with collective state, alignment, and inter faction relationships.
- **Crafting and transformation.** Combining entities to produce new entities (iron + fire → sword).

### Future: Multi File Worlds

- **Scope as directory.** Large worlds decomposed across files. Each directory is a scope. The compiler resolves cross file references.
- **Imports and inheritance.** Type inheritance and shared type libraries importable across worlds.
- **Multi player state.** Per player visibility, shared world state, concurrent action resolution.

*This specification is a living document. The principle remains:* **each version adds capability without breaking worlds authored against earlier versions.**

*End of Specification*
