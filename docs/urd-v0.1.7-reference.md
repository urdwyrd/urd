# Urd v0.1.7 Reference Manual

*Consolidated reference for the Urd declarative schema system*

*Schema Markdown → Compiler → .urd.json + FactSet IR*

February 2026 · Compiler v0.1.7 · 554 tests · Gate closed

> **Pre-Alpha.** Urd is in active early development. The schema language, JSON format, compiler behaviour, and FactSet IR described here are all subject to change. Expect breaking changes. Keep your source worlds under version control. Treat compiled output as disposable.

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [How Urd Differs](#2-how-urd-differs)
3. [Getting Started](#3-getting-started)
4. [Schema Markdown Syntax](#4-schema-markdown-syntax)
5. [Worked Examples](#5-worked-examples)
6. [The Compiled JSON Format](#6-the-compiled-json-format)
7. [The Compiler](#7-the-compiler)
8. [Errors and Warnings](#8-errors-and-warnings)
9. [FactSet Analysis IR](#9-factset-analysis-ir)
10. [Advanced Topics](#10-advanced-topics)
11. [Quick Reference](#11-quick-reference)

---

## 1. Introduction

Urd is an open, structured world definition format — a declarative schema that describes interactive worlds as typed, validated, engine-agnostic data. Writers author in **Schema Markdown** (`.urd.md`), a prose-friendly syntax with a small set of structural symbols. The compiler validates and produces two outputs: a **`.urd.json` contract file** that runtimes consume, and a **FactSet** — a flat, queryable graph of every relationship in the world.

The pipeline:

```
.urd.md files  →  Compiler  →  .urd.json + FactSet IR  →  Runtime / Tooling / Engine Integrations
```

There is no intermediate format. Writers author in `.urd.md`. The compiler produces `.urd.json` and a FactSet. Runtimes, testing tools, and engine integrations consume the JSON. Tooling queries the FactSet.

### Who This Document Is For

This reference covers three audiences:

- **Writers** — start at [Getting Started](#3-getting-started) and [Schema Markdown Syntax](#4-schema-markdown-syntax). You write prose. A handful of symbols handle structure.
- **Developers** — start at [The Compiled JSON Format](#6-the-compiled-json-format). This is what your runtime consumes. Self-contained, deterministic, versioned, human-inspectable.
- **Architects and tool builders** — start at [The Compiler](#7-the-compiler) and [FactSet Analysis IR](#9-factset-analysis-ir). The compiler's five-phase pipeline, diagnostic system, and analysis IR are documented here.

### Version Note

This document describes **compiler v0.1.7** (February 2026), a **pre-alpha** milestone. All compiler-side acceptance criteria are met: 9 compiler requirements (C1–C9), 8 static analysis checks (S1–S8), 8 FactSet verification criteria (F1–F8), 16 test fixtures (7 canonical, 9 negative), and JSON Schema validation on all compiled output. 554 tests, 100% pass rate.

The gate closure means the compiler is *functionally complete for v1 scope* — it does not mean the schema language or JSON contract are frozen. Syntax, semantics, and output structure may all evolve before a stable release.

### How to Read This Manual

- **Required** means the compiler rejects it or the runtime cannot function correctly without it.
- **Recommended** means it usually avoids bugs or confusion.
- **Optional** means you can ignore it safely.

Everything else is explanatory.

### First Ten Minutes

**Writing a world?**
1. Copy the [hello world example](#writing-your-first-world) and compile it.
2. Read [Structural Symbols](#41-structural-symbols) — eight symbols cover everything.
3. Try adding a choice with a condition. See [Conditions and Effects](#45-conditions-and-effects).

**Building a runtime?**
1. Skim [Top-Level Structure](#61-top-level-structure) — eight JSON blocks, only `world` is required.
2. Implement the [containment model](#101-the-containment-model) — it replaces inventory, spatial queries, and object transfer.
3. Implement the [dialogue evaluation loop](#66-dialogue) — sections, choices, exhaustion, jumps.

**Building tooling?**
1. Jump to [FactSet Analysis IR](#9-factset-analysis-ir) — six fact types, one secondary index.
2. Read the [diagnostic shape](#reading-a-diagnostic) — every diagnostic includes code, severity, message, and span.
3. See [Replay and Randomness](#67-replay-and-randomness) for determinism guarantees.

---

## 2. How Urd Differs

Most interactive narrative tooling — Ink, Yarn Spinner, articy:draft (Flow Player) — treats story as a **program to execute**: a cursor moves through text, hits choices, and advances. Urd is fundamentally different. The schema describes what the world **is**, not what it **does**. The runtime evaluates a **system**, not a script.

> Ink is used here for comparison because a surface-level glance at `.urd.md` files — markdown, choices, conditions — might suggest the two are similar. They are not. The resemblance is cosmetic. The underlying models, runtime shapes, and design goals are fundamentally different.

| | Ink | Urd / Wyrd |
|---|---|---|
| **Mental model** | Story = cursor moving through text | World = structured state graph + rules + queries |
| **Runtime shape** | Text stream with pauses (`Continue()`, `ChooseChoiceIndex()`) | State container + rule engine (`Query()`, `Apply()`, `Step()`) |
| **Content model** | Text-first; logic embedded in narrative; loose globals | Schema-first; typed entities, properties, and constraints |
| **Execution** | `Continue → text → choice → Continue → …` | `Query → decide → Apply → Step → recompute` |
| **Validation** | Compiles to JSON; errors can surface at runtime | Five-phase compiler with JSON Schema enforcement and static analysis |
| **Best at** | Branching dialogue, writer-friendly linear narrative | Systems-driven worlds, simulation, correctness guarantees, multi-agent interaction |

The compiler guarantees that every reference resolves, every type checks, and every constraint is satisfied *before* the runtime sees the output. What it does not guarantee is runtime correctness of game logic — that remains the author's responsibility.

Ink is an authoring tool. Wyrd is an execution model for worlds. They occupy different spaces. If a higher-level narrative layer is built on top of Wyrd, *that* would compete with Ink — but the core Urd/Wyrd system is closer to a simulation engine than a dialogue tool.

---

## 3. Getting Started

### Writing Your First World

Create a file called `hello.urd.md`:

```
---
world:
  name: hello-world
  start: room

types:
  NPC [interactable]:
    name: string
    mood: enum(friendly, grumpy) = friendly

entities:
  @greeter: NPC { name: "Ada" }
---

# Room

A quiet room with a single occupant.

[@greeter]

== conversation

@greeter: Hello, traveller. What brings you here?

+ Ask about the room
  @greeter: It's not much, but it's honest.

+ Ask about her name
  @greeter: Ada. And you are...?
  > @greeter.mood = friendly

* Leave
  -> end
```

[Open in Playground](https://urd.dev/playground#code=LS0tCndvcmxkOgogIG5hbWU6IGhlbGxvLXdvcmxkCiAgc3RhcnQ6IHJvb20KCnR5cGVzOgogIE5QQyBbaW50ZXJhY3RhYmxlXToKICAgIG5hbWU6IHN0cmluZwogICAgbW9vZDogZW51bShmcmllbmRseSwgZ3J1bXB5KSA9IGZyaWVuZGx5CgplbnRpdGllczoKICBAZ3JlZXRlcjogTlBDIHsgbmFtZTogIkFkYSIgfQotLS0KCiMgUm9vbQoKQSBxdWlldCByb29tIHdpdGggYSBzaW5nbGUgb2NjdXBhbnQuCgpbQGdyZWV0ZXJdCgo9PSBjb252ZXJzYXRpb24KCkBncmVldGVyOiBIZWxsbywgdHJhdmVsbGVyLiBXaGF0IGJyaW5ncyB5b3UgaGVyZT8KCisgQXNrIGFib3V0IHRoZSByb29tCiAgQGdyZWV0ZXI6IEl0J3Mgbm90IG11Y2gsIGJ1dCBpdCdzIGhvbmVzdC4KCisgQXNrIGFib3V0IGhlciBuYW1lCiAgQGdyZWV0ZXI6IEFkYS4gQW5kIHlvdSBhcmUuLi4/CiAgPiBAZ3JlZXRlci5tb29kID0gZnJpZW5kbHkKCiogTGVhdmUKICAtPiBlbmQK)

This file defines a world with one location, one NPC, and a short conversation. The frontmatter declares the types and entities. The body is narrative.

### Compiling

**Native CLI:**

```bash
urd hello.urd.md > hello.urd.json
```

Diagnostics are printed to stderr. The compiled JSON is written to stdout. Exit code 0 on success, 1 on errors.

**WASM (browser):**

The compiler runs client-side via three WASM entry points:

| Function | Purpose |
|----------|---------|
| `compile_source(source)` | Full five-phase pipeline |
| `parse_only(source)` | Phase 1 only (live syntax checking) |
| `compiler_version()` | Crate version string |

**Response shapes:**

```json
// compile_source(source)
{
  "success": true,
  "world": { ... },         // The compiled .urd.json contract (omitted on failure)
  "diagnostics": [           // Array of { code, severity, message, span }
    { "code": "URD301", "severity": "error", "message": "...", "span": { ... } }
  ],
  "facts": {                  // FactSet IR (see Section 9)
    "reads": [...],
    "writes": [...],
    "exits": [...],
    "jumps": [...],
    "choices": [...],
    "rules": [...]
  }
}

// parse_only(source)
{
  "success": true,
  "diagnostics": []
}

// compiler_version()
"0.1.7"
```

Try it live at [urd.dev/playground](https://urd.dev/playground).

### What the Compiler Produces

Two outputs:

1. **`.urd.json`** — the compiled world. Self-contained, deterministic, versioned. This is the contract between authoring and execution. See [The Compiled JSON Format](#6-the-compiled-json-format).
2. **FactSet IR** — a flat, queryable graph of every relationship in the world. Used by tooling, not runtimes. See [FactSet Analysis IR](#9-factset-analysis-ir).

---

## 4. Schema Markdown Syntax

### 4.1 Structural Symbols

| Symbol | Name | What It Does | Example |
|--------|------|--------------|---------|
| `@` | Entity reference | References characters, objects, locations | `@guard`, `@door_1` |
| `?` | Condition | Gates content on world state | `? @guard.mood == neutral` |
| `>` | Effect | Changes the world | `> @guard.mood = neutral` |
| `*` | One-shot choice | Disappears after selection | `* Ask about the ship` |
| `+` | Sticky choice | Stays available on revisit | `+ Ask about the harbour` |
| `->` | Jump | Navigates to a section or location | `-> topics`, `-> harbour` |
| `!` | Blocked message | Shown when an exit or action is blocked | `! The door is locked.` |
| `//` | Comment | Ignored by the compiler | `// hub prompt` |

Plain text outside any marker is narrative prose.

### 4.2 Frontmatter

The block between `---` delimiters is **Urd frontmatter** — YAML-like but not YAML. Parsed by the Urd compiler under strict constraints.

**What is allowed:**
- Key-value pairs: `key: value`
- Nested blocks via two-space indentation
- Inline objects: `@arina: Barkeep { name: "Arina" }`
- Flow-style lists: `[goat, car]`, `[@door_1, @door_2]`
- Comments: `# text`
- Quoted strings when ambiguous: `"1"`, `"yes"`, `"true"`

**What is not allowed:**
- Anchors (`&name`), aliases (`*name`), merge keys (`<<:`)
- Custom tags (`!!type`)
- Block-style lists (`- item`)
- Implicit type coercion

#### The `world` Block

```
world:
  name: monty-hall
  start: stage
  entry: game
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique identifier. Lowercase, hyphens allowed. |
| `start` | No | Starting location. |
| `entry` | No | Starting sequence. |
| `version` | No | Author-defined version string. |
| `description` | No | Human-readable description. |
| `author` | No | Author or team name. |
| `seed` | No | Random seed for deterministic replay. |

The `urd` version field is set automatically by the compiler to `"1"`. If an author specifies it manually, the compiler warns and overrides.

#### Imports

```
---
import: ./world.urd.md
---
```

Imports are explicit and non-transitive. If file A imports B, and B imports C, A does not see C's types unless it imports C directly. Circular imports are a compile error.

### 4.3 Locations and Headings

| Heading | Compiles To |
|---------|-------------|
| `# Location Name` | A location in the `locations` block |
| `## Sequence Name` | A sequence in the `sequences` block |
| `### Phase Name` | A phase within the enclosing sequence |
| `### Phase Name (auto)` | An auto-advancing phase (`auto: true`) |

Entity placement uses square brackets after a location heading:

```
# The Rusty Anchor
A low-ceilinged tavern thick with pipe smoke.
[@arina, @barrel]
```

Exits connect locations. The name before the colon is the **exit name** (a direction, label, or identifier). The name after the colon is the **destination** (a location heading):

```
-> north: Corridor
  ? @cell_door.locked == false
  ! The iron door is locked.
```

Here, `north` is the exit name and `Corridor` is the destination location. Write the heading title as it appears in the `#` line — the compiler slugifies it internally (e.g., `# The Rusty Anchor` becomes `the-rusty-anchor`). Exits are unidirectional — the destination does not automatically have an exit back.

### 4.4 Types, Entities, and Properties

#### Type Definitions

```
types:
  Guard [interactable, mobile, container]:
    name: string
    mood: enum(hostile, neutral, helpful) = hostile
    ~hint_given: bool = false
```

The `~` prefix marks a property as `visibility: hidden`. Traits (`[interactable, mobile, container]`) inform the runtime about spatial capabilities.

#### Property Types

| Type | Values | Example |
|------|--------|---------|
| `bool` / `boolean` | `true` / `false` | `locked: bool = true` |
| `int` / `integer` | Whole numbers, optional range | `trust: int(0, 100) = 30` |
| `num` / `number` | Decimal numbers, optional range | `weight: num(0.0, 10.0)` |
| `str` / `string` | Text | `name: string` |
| `enum` | One of a declared set | `mood: enum(hostile, neutral)` |
| `ref` | Reference to another entity | `requires: ref(Key)` |
| `list` | Ordered list | `tags: list = []` |

Short forms (`int`, `num`, `str`, `bool`) are normalised to canonical names during compilation.

#### Entity Traits

| Trait | Meaning |
|-------|---------|
| `container` | Can hold other entities. Locations have this implicitly. |
| `portable` | Can be moved into another container. |
| `mobile` | Can move itself between containers. |
| `interactable` | Can be the target of player actions. Default: `true`. |

#### Visibility Model

| Level | Who Sees It | Source Token | JSON Support |
|-------|-------------|-------------|--------------|
| `visible` | Everyone (default) | *(none)* | Yes |
| `hidden` | World only. Must be explicitly revealed. | `~` | Yes |
| `owner` | Entity itself and the world. | `~~` *(reserved — not yet supported in source)* | Yes |
| `conditional` | Visible when a condition is met. | *(no source syntax)* | Yes |

The `owner` and `conditional` levels are part of the v1 JSON format and runtimes are expected to support them. Source syntax for `owner` (`~~`) is reserved but not yet implemented in the compiler. `conditional` visibility is set programmatically in JSON; there is no source shorthand.

#### Entity Declarations

```
entities:
  @rusty_key: Key { name: "Rusty Key" }
  @cell_door: LockedDoor { requires: @rusty_key }
  @guard: Guard { name: "Halvard" }
```

Entity IDs must be globally unique. The `@` prefix is used in source; compiled JSON strips it.

#### Quoting Rules in Frontmatter

Enum values and property overrides in frontmatter follow these rules:

- **Bare tokens** are the default. `prize: goat`, `mood: hostile` — no quotes needed.
- **Quoted strings** are required when the value could be misinterpreted: `"true"`, `"false"`, `"1"`, `"yes"`, `"null"`. Without quotes, these are parsed as booleans, numbers, or null.
- **Consistency.** Examples in this document use bare tokens for enum values. Pick one convention per project and apply it throughout.

### 4.5 Conditions and Effects

#### Conditions (`?` lines)

```
? @arina.trust > 50              // Comparison
? @coin_purse in player          // Containment check
? @coin_purse not in player      // Negated containment
? @key in here                   // "here" = player's current container (see below)
```

Multiple `?` lines are AND-ed. For OR logic, use `? any:`:

```
? any:
  @guard.mood == hostile
  @guard.mood == suspicious
```

A `? any:` block counts as a single condition and can be combined with other `?` lines. An empty `? any:` block (no indented conditions) is accepted by the parser but evaluates to false — avoid it.

#### Effects (`>` lines)

```
> @arina.trust = 75              // Set to value
> @arina.trust + 5               // Increment
> move @coin_purse -> @arina     // Transfer object
> move @key -> here              // Drop in current location
> reveal @door.prize             // Unhide a property
> destroy @rusty_key             // Remove from world
```

#### Reserved Keywords: `player` and `here`

`player` and `here` are reserved runtime bindings, not entity references:

- **`player`** — resolves to the player entity at runtime. Used as a container target: `> move @key -> player`, `? @key in player`.
- **`here`** — resolves to `player.container` at runtime. It always means the player's current location, regardless of whether the condition appears inside a dialogue section, a sequence phase, or a location body.

Both are lowered to their runtime equivalents during EMIT. They are not valid entity IDs.

### 4.6 Dialogue

#### Choice Types

`*` marks a **one-shot** choice — consumed after selection, never shown again. `+` marks a **sticky** choice — available every time the section is entered.

```
== topics
@arina: What'll it be, stranger?

+ Ask about the harbour             // always available
  @arina: Quiet today. Too quiet.
  > @arina.trust + 5
  -> topics

* Ask about the missing ship        // disappears once asked
  ? @arina.trust > 50
  @arina: The Selene didn't sink. She was taken.
  -> topics

* Leave -> harbour

@arina: Suit yourself. I've got glasses to clean.
```

#### Targeting

Choices can target specific entities or entity types:

```
* Pick up the key -> @rusty_key        // target a specific entity
* Pick a door -> any Door              // target any Door the player can reach
```

`-> @entity` compiles to an action with a `target` field. `-> any Type` compiles to an action with a `target_type` field — the runtime presents the player with all reachable entities of that type and lets them choose.

**Reachable** means: inside `here` (the player's current location), plus inside open containers within `here`, recursively. Hidden entities are excluded unless revealed. This rule applies to all `target_type` resolution.

#### Exhaustion

When all choices are consumed or gated, the section is **exhausted**. Content falls through to text after the choice block. Test for exhaustion from another section: `? topics.exhausted`.

**Exhaustion is a runtime-evaluated predicate**, not a stored boolean. The compiled JSON contains an `on_exhausted` field with fallthrough content, not an `exhausted` flag.

Exhaustion predicates only resolve within the same file. Cross-file exhaustion checks (e.g., testing `? topics.exhausted` from a different file) are rejected by the compiler (URD423).

#### Sections and Jumps

| Syntax | Meaning |
|--------|---------|
| `== name` | Declare a section. Names: lowercase, digits, underscores. Unique within file. |
| `-> name` | Jump to a section or exit in this file (see resolution below). |
| `-> exit:name` | Explicitly target an exit when a section shadows the name. |
| `-> end` | Exit dialogue mode. |

**Resolution priority for `-> name`:** The compiler resolves the target as a section first, then as a declared exit. If neither exists, it is a compile error (URD309). `-> name` can only target exits that are explicitly declared on the current location — it does not create an implicit exit or perform a direct move. Use `-> exit:name` to disambiguate when a section and exit share a name.

#### Nesting Rules

Two-space indent per level. Content under a choice is indented one level.

| Depth | Compiler Behaviour |
|-------|--------------------|
| 1–2 levels | Normal. |
| 3 levels | Warning (URD410): *"Consider breaking into a labelled section."* |
| 4+ levels | Error (URD410). File does not compile. |

```
* Level 1 choice                        // depth 1
  Some response text.
  * Level 2 choice                      // depth 2 — OK
    * Level 3 choice                    // depth 3 — WARNING
      * Level 4 choice                  // depth 4 — ERROR
```

#### Entity Speech vs Narration

```
@arina: What'll it be?         // Dialogue (colon = speech)
@arina leans in close.          // Stage direction (no colon = narration)
```

### 4.7 Rules

Rules define constrained NPC behaviour: an actor performs effects when a trigger fires and conditions are met. Rules are typically engineer-authored.

```
rule monty_reveals:
  actor: @host action reveal
  selects door from [@door_1, @door_2, @door_3]
    where door.prize == goat
  > reveal door.prize
```

**Canonical structure:**

| Line | Purpose |
|------|---------|
| `rule name:` | Declares the rule. Names must be unique within the compilation unit. |
| `actor: @entity action trigger` | The entity that performs the rule, and the trigger that activates it. |
| `selects var from [...]` | Constrained selection from an entity set. |
| `where expr` | Filter clauses (indented under `selects`). All must be satisfied. |
| `> effect` | Effects applied when the rule fires. |

The `selects` block enables constrained choice — from a set of entities, choose one matching all `where` conditions. If multiple match, the runtime chooses uniformly at random. If none match, the rule does not fire.

---

## 5. Worked Examples

### 5.1 The Monty Hall Problem

A sequence-driven game show demonstrating hidden state, constrained NPC behaviour, and phased progression. No probability is specified anywhere — the 2/3 switching advantage emerges from the structure.

```
---
world:
  name: monty-hall
  start: stage

types:
  Door [interactable]:
    ~prize: enum(goat, car)
    revealed: bool = false

entities:
  @door_1: Door { prize: "goat" }
  @door_2: Door { prize: "goat" }
  @door_3: Door { prize: "car" }
  @host: Door
---

# Stage

[@door_1, @door_2, @door_3]

## The Game

### Choose

* Pick a door -> any Door

### Reveal (auto)

rule monty_reveals:
  actor: @host action reveal
  selects door from [@door_1, @door_2, @door_3]
    where door.prize == goat
  > reveal door.prize

### Switch

== switch

* Switch doors -> any Door
  ? @door_1.revealed == false
* Stay with your choice

The host opens the final door.
```

[Open in Playground](https://urd.dev/playground#code=LS0tCndvcmxkOgogIG5hbWU6IG1vbnR5LWhhbGwKICBzdGFydDogc3RhZ2UKCnR5cGVzOgogIERvb3IgW2ludGVyYWN0YWJsZV06CiAgICB+cHJpemU6IGVudW0oZ29hdCwgY2FyKQogICAgcmV2ZWFsZWQ6IGJvb2wgPSBmYWxzZQoKZW50aXRpZXM6CiAgQGRvb3JfMTogRG9vciB7IHByaXplOiAiZ29hdCIgfQogIEBkb29yXzI6IERvb3IgeyBwcml6ZTogImdvYXQiIH0KICBAZG9vcl8zOiBEb29yIHsgcHJpemU6ICJjYXIiIH0KICBAaG9zdDogRG9vcgotLS0KCiMgU3RhZ2UKCltAZG9vcl8xLCBAZG9vcl8yLCBAZG9vcl8zXQoKIyMgVGhlIEdhbWUKCiMjIyBDaG9vc2UKCiogUGljayBhIGRvb3IgLT4gYW55IERvb3IKCiMjIyBSZXZlYWwgKGF1dG8pCgpydWxlIG1vbnR5X3JldmVhbHM6CiAgYWN0b3I6IEBob3N0IGFjdGlvbiByZXZlYWwKICBzZWxlY3RzIGRvb3IgZnJvbSBbQGRvb3JfMSwgQGRvb3JfMiwgQGRvb3JfM10KICAgIHdoZXJlIGRvb3IucHJpemUgPT0gZ29hdAogID4gcmV2ZWFsIGRvb3IucHJpemUKCiMjIyBTd2l0Y2gKCj09IHN3aXRjaAoKKiBTd2l0Y2ggZG9vcnMgLT4gYW55IERvb3IKICA/IEBkb29yXzEucmV2ZWFsZWQgPT0gZmFsc2UKKiBTdGF5IHdpdGggeW91ciBjaG9pY2UKClRoZSBob3N0IG9wZW5zIHRoZSBmaW5hbCBkb29yLgo=)

**What to notice:**

- `@host` is typed as `Door` — this is a deliberate shortcut for the example. The host is not spatially important here; it only exists as the `actor:` in the rule. A production world would declare a separate `Host` type (e.g., `Host [interactable]: name: string`).
- The `selects` constraint filters out the car door and the player's chosen door, forcing the host to reveal a goat. The 2/3 switching advantage is an emergent property of a correct world definition, not an authored probability.

### 5.2 The Two Room Key Puzzle

A freeform spatial puzzle with no sequences. Progression emerges from action conditions and the containment model.

```
---
world:
  name: two-room-key
  start: cell

types:
  Key [portable]:
    name: string

  LockedDoor [interactable]:
    locked: bool = true
    requires: ref(Key)

  Guard [interactable, mobile, container]:
    name: string
    mood: enum(hostile, neutral, helpful) = hostile
    ~hint_given: bool = false

entities:
  @rusty_key: Key { name: "Rusty Key" }
  @cell_door: LockedDoor { requires: @rusty_key }
  @guard: Guard { name: "Halvard" }
---

# Cell

A dim stone cell. A guard watches from the corner.

[@rusty_key, @guard, @cell_door]

* Wait quietly and show respect -> @guard
  ? @guard.mood == hostile
  > @guard.mood = neutral

* Talk to the guard -> @guard
  ? @guard.mood == neutral
  ? @guard.hint_given == false
  @guard glances at the loose stone in the wall.
  > @guard.hint_given = true

* Pick up the rusty key -> @rusty_key
  ? @rusty_key in here
  > move @rusty_key -> player

* Use the key on the door -> @cell_door
  ? @rusty_key in player
  ? @cell_door.locked == true
  The lock clicks. The door swings open.
  > @cell_door.locked = false
  > destroy @rusty_key

-> north: Corridor
  ? @cell_door.locked == false
  ! The iron door is locked.

# Corridor

A long corridor. Daylight leaks from the far end.

-> south: Cell
```

[Open in Playground](https://urd.dev/playground#code=LS0tCndvcmxkOgogIG5hbWU6IHR3by1yb29tLWtleQogIHN0YXJ0OiBjZWxsCgp0eXBlczoKICBLZXkgW3BvcnRhYmxlXToKICAgIG5hbWU6IHN0cmluZwoKICBMb2NrZWREb29yIFtpbnRlcmFjdGFibGVdOgogICAgbG9ja2VkOiBib29sID0gdHJ1ZQogICAgcmVxdWlyZXM6IHJlZihLZXkpCgogIEd1YXJkIFtpbnRlcmFjdGFibGUsIG1vYmlsZSwgY29udGFpbmVyXToKICAgIG5hbWU6IHN0cmluZwogICAgbW9vZDogZW51bShob3N0aWxlLCBuZXV0cmFsLCBoZWxwZnVsKSA9IGhvc3RpbGUKICAgIH5oaW50X2dpdmVuOiBib29sID0gZmFsc2UKCmVudGl0aWVzOgogIEBydXN0eV9rZXk6IEtleSB7IG5hbWU6ICJSdXN0eSBLZXkiIH0KICBAY2VsbF9kb29yOiBMb2NrZWREb29yIHsgcmVxdWlyZXM6IEBydXN0eV9rZXkgfQogIEBndWFyZDogR3VhcmQgeyBuYW1lOiAiSGFsdmFyZCIgfQotLS0KCiMgQ2VsbAoKQSBkaW0gc3RvbmUgY2VsbC4gQSBndWFyZCB3YXRjaGVzIGZyb20gdGhlIGNvcm5lci4KCltAcnVzdHlfa2V5LCBAZ3VhcmQsIEBjZWxsX2Rvb3JdCgoqIFdhaXQgcXVpZXRseSBhbmQgc2hvdyByZXNwZWN0IC0+IEBndWFyZAogID8gQGd1YXJkLm1vb2QgPT0gaG9zdGlsZQogID4gQGd1YXJkLm1vb2QgPSBuZXV0cmFsCgoqIFRhbGsgdG8gdGhlIGd1YXJkIC0+IEBndWFyZAogID8gQGd1YXJkLm1vb2QgPT0gbmV1dHJhbAogID8gQGd1YXJkLmhpbnRfZ2l2ZW4gPT0gZmFsc2UKICBAZ3VhcmQgZ2xhbmNlcyBhdCB0aGUgbG9vc2Ugc3RvbmUgaW4gdGhlIHdhbGwuCiAgPiBAZ3VhcmQuaGludF9naXZlbiA9IHRydWUKCiogUGljayB1cCB0aGUgcnVzdHkga2V5IC0+IEBydXN0eV9rZXkKICA/IEBydXN0eV9rZXkgaW4gaGVyZQogID4gbW92ZSBAcnVzdHlfa2V5IC0+IHBsYXllcgoKKiBVc2UgdGhlIGtleSBvbiB0aGUgZG9vciAtPiBAY2VsbF9kb29yCiAgPyBAcnVzdHlfa2V5IGluIHBsYXllcgogID8gQGNlbGxfZG9vci5sb2NrZWQgPT0gdHJ1ZQogIFRoZSBsb2NrIGNsaWNrcy4gVGhlIGRvb3Igc3dpbmdzIG9wZW4uCiAgPiBAY2VsbF9kb29yLmxvY2tlZCA9IGZhbHNlCiAgPiBkZXN0cm95IEBydXN0eV9rZXkKCi0+IG5vcnRoOiBDb3JyaWRvcgogID8gQGNlbGxfZG9vci5sb2NrZWQgPT0gZmFsc2UKICAhIFRoZSBpcm9uIGRvb3IgaXMgbG9ja2VkLgoKIyBDb3JyaWRvcgoKQSBsb25nIGNvcnJpZG9yLiBEYXlsaWdodCBsZWFrcyBmcm9tIHRoZSBmYXIgZW5kLgoKLT4gc291dGg6IENlbGwK)

**What to notice:** Containment replaces inventory throughout. "Pick up" is `> move @rusty_key -> player`. "Does the player have the key?" is `? @rusty_key in player`. The guard's interaction arc (hostile → neutral → hint) emerges from conditions gating what's available. No sequence block needed.

### Schema Coverage Matrix

| Concept | Monty Hall | Key Puzzle |
|---------|------------|------------|
| `world` metadata | ✓ | ✓ |
| Types and properties | ✓ | ✓ |
| `visibility: hidden` (`~`) | ✓ (`~prize`) | ✓ (`~hint_given`) |
| `traits: portable` | — | ✓ |
| `traits: container` | — | ✓ (Guard) |
| Containment as inventory | — | ✓ |
| Locations (multi) | — | ✓ |
| Exits with conditions | — | ✓ |
| Rules with `select` | ✓ | — |
| Sequences (phases) | ✓ | — |
| Freeform (no sequence) | — | ✓ |
| `effects: set` | ✓ | ✓ |
| `effects: reveal` | ✓ | — |
| `effects: move` | — | ✓ |
| `effects: destroy` | — | ✓ |

Together, the two examples cover most v1 primitives. Not covered: `owner` visibility, `conditional` visibility, and `spawn` effects.

---

## 6. The Compiled JSON Format

A compiled `.urd.json` file is self-contained, deterministic (same source → byte-identical output), versioned, and human-inspectable. It is the single interchange format between authoring and execution.

Here is a minimal compiled output for a world with one type, one entity, one location, and one dialogue section:

```json
{
  "world": { "name": "hello-world", "urd": "1", "start": "room" },
  "types": {
    "NPC": {
      "traits": ["interactable"],
      "properties": {
        "name": { "type": "string" },
        "mood": { "type": "enum", "values": ["friendly", "grumpy"], "default": "friendly" }
      }
    }
  },
  "entities": {
    "greeter": { "type": "NPC", "properties": { "name": "Ada" } }
  },
  "locations": {
    "room": {
      "description": "A quiet room with a single occupant.",
      "contains": ["greeter"]
    }
  },
  "dialogue": {
    "hello-world/conversation": {
      "id": "hello-world/conversation",
      "prompt": { "speaker": "greeter", "text": "Hello, traveller. What brings you here?" },
      "choices": [
        { "id": "hello-world/conversation/ask-about-the-room", "label": "Ask about the room", "sticky": true },
        { "id": "hello-world/conversation/ask-about-her-name", "label": "Ask about her name", "sticky": true },
        { "id": "hello-world/conversation/leave", "label": "Leave", "sticky": false }
      ]
    }
  }
}
```

The sections below describe each block in detail.

### 6.1 Top-Level Structure

All blocks are optional except `world`. Blocks can appear in any order.

| Block | Purpose |
|-------|---------|
| `world` | **Required.** Metadata: name, version, start location, entry sequence. |
| `types` | Entity type definitions with property schemas. |
| `entities` | Instances of defined types with property overrides. |
| `locations` | Spatial containers with exits and connections. |
| `rules` | NPC behavioural constraints: trigger → condition → select → effect. |
| `actions` | Player-performable interactions: target, prerequisites, effects. |
| `sequences` | Ordered phase flows: game show rounds, tutorial steps. |
| `dialogue` | Flat map of dialogue sections, choices, jumps, and `on_exhausted` content. |

### 6.2 The `world` Block

```json
{
  "world": {
    "name": "monty-hall",
    "urd": "1",
    "version": "1.0",
    "description": "The classic Monty Hall problem",
    "start": "stage",
    "entry": "game"
  }
}
```

The `urd` field is always `"1"` for v1, set by the compiler. The `name` field is slugified during emit.

### 6.3 Types, Entities, Properties

```json
{
  "types": {
    "Door": {
      "traits": ["interactable"],
      "properties": {
        "prize": { "type": "enum", "values": ["goat", "car"], "visibility": "hidden" },
        "state": { "type": "enum", "values": ["closed", "open"], "default": "closed" }
      }
    }
  },
  "entities": {
    "door_1": { "type": "Door", "properties": { "prize": "car" } }
  }
}
```

Property types in compiled JSON always use canonical long forms (`boolean`, `integer`, `number`, `string`, `enum`, `ref`, `list`).

### 6.4 Locations and Exits

```json
{
  "locations": {
    "cell": {
      "description": "A dim stone cell.",
      "contains": ["rusty_key", "guard"],
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

Exits are unidirectional. The field name is `contains`, consistent with the containment model.

### 6.5 Actions, Rules, Sequences

**Actions** are player-performable interactions:

```json
{
  "pick_up_key": {
    "target": "rusty_key",
    "conditions": ["rusty_key.container == player.container"],
    "effects": [{ "move": "rusty_key", "to": "player" }]
  }
}
```

Declare either `target` (specific entity) or `target_type` (any matching entity), not both.

**Rules** use the `select` block for constrained random choice:

```json
{
  "monty_reveals": {
    "actor": "monty",
    "trigger": "phase_is reveal",
    "select": {
      "from": ["door_1", "door_2", "door_3"],
      "as": "target",
      "where": ["target.prize != car", "target.chosen == false"]
    },
    "effects": [{ "set": "target.state", "to": "open" }]
  }
}
```

**Sequences** define ordered phase flows. Worlds without sequences are freeform.

| Advance Mode | Behaviour |
|-------------|-----------|
| `on_action` | Advance after the player completes a listed action. |
| `on_rule` | Advance after the phase's rule fires. |
| `on_condition <expr>` | Advance when the expression becomes true. |
| `auto` | Advance immediately after phase effects. |
| `manual` | Remains active until explicitly advanced. Default. |
| `end` | The sequence ends. |

### 6.6 Dialogue

The `dialogue` block is a flat map of sections keyed by world-unique ID.

```json
{
  "dialogue": {
    "tavern/topics": {
      "id": "tavern/topics",
      "prompt": { "speaker": "arina", "text": "What'll it be, stranger?" },
      "choices": [{
        "id": "tavern/topics/ask-about-the-harbour",
        "label": "Ask about the harbour",
        "sticky": true,
        "response": { "speaker": "arina", "text": "Quiet today." },
        "effects": [{ "set": "arina.trust", "to": "arina.trust + 5" }],
        "goto": "tavern/topics"
      }],
      "on_exhausted": { "text": "Suit yourself." }
    }
  }
}
```

**Stable IDs:**
- Section: `file_stem/section_name` (e.g., `tavern/topics`)
- Choice: `section_id/slugified-label` (e.g., `tavern/topics/ask-about-the-harbour`)
- Entity: declared `@name` (e.g., `rusty_key`)

**Exhaustion** is never stored. It is recomputed on every evaluation: if every choice is consumed or gated, the section is exhausted. The `on_exhausted` field is content, not a boolean.

### 6.7 Replay and Randomness

**How do I get repeatable runs?** Set `world.seed` in the frontmatter. The runtime uses this seed for every random selection. Same seed + same player choices = same outcome every time.

**What if I don't set a seed?** The runtime generates one. It should record the seed in your save data so the session can be replayed later.

**Where does randomness happen?** Only inside `selects` blocks. When multiple entities match the `where` conditions, one is chosen uniformly at random. Declaration order does not influence probability.

**What breaks replay?** Reordering rules in source changes which random values get consumed where. Runs are still repeatable, but old saves may no longer replay the same way after recompiling. Any change to declarations, conditions, effects, or structure is a semantic change and may produce different output — including different IDs and different evaluation order.

**What stays stable?** The compiler produces byte-identical output for the same source. Adding whitespace, blank lines, or comments does not change the output.

### 6.8 Effects

| Effect | JSON Structure | Description |
|--------|---------------|-------------|
| `set` | `{ "set": "entity.prop", "to": value }` | Change a property value. |
| `move` | `{ "move": "entity", "to": "container" }` | Move entity into a container. |
| `reveal` | `{ "reveal": "entity.prop" }` | Change hidden → visible. |
| `destroy` | `{ "destroy": "entity" }` | Remove entity from world. |
| `spawn` | `{ "spawn": { "id": "new_id", "type": "Type", "in": "container" } }` | Create entity at runtime. |

---

## 7. The Compiler

### 7.1 Five-Phase Pipeline

```
.urd.md files
     │
  1. PARSE       Source text → per-file ASTs
     │
  2. IMPORT      Resolve imports, build dependency graph
     │
  3. LINK        Merge scopes, resolve cross-file references, extract FactSet
     │
  4. VALIDATE    Type-check properties, conditions, effects, static analysis
     │
  5. EMIT        Produce .urd.json output
```

### 7.2 What Each Compiler Phase Does

| Phase | Input | Output | Guarantee |
|-------|-------|--------|-----------|
| PARSE | Source text | Per-file AST | Every token classified. Unrecognised syntax produces URD111/URD112. |
| IMPORT | File ASTs | Dependency graph | Cycles detected (URD202). Topological order established. |
| LINK | Graph + ASTs | Symbol table + FactSet | Every reference resolved. Duplicates rejected. |
| VALIDATE | Symbol table | Diagnostics | Type safety. Constraint satisfaction. Static analysis (S1–S8). |
| EMIT | Validated symbol table | `.urd.json` | Conforms to JSON Schema. `urd: "1"` injected. |

### 7.3 Import Resolution

- Imports are explicit and non-transitive.
- Paths are relative to the importing file. Absolute paths rejected.
- Circular imports produce URD202 with the full cycle path.
- Duplicate entity IDs (URD302) and type names (URD303) across files are compile errors.
- File stem collisions (URD203) are rejected since section IDs would collide.

### 7.4 ID Derivation

| Element | Derivation | Example |
|---------|-----------|---------|
| Entity | Declared `@name` | `@rusty_key` → `rusty_key` |
| Location | Slugified heading | `# The Rusty Anchor` → `the-rusty-anchor` |
| Section | `file_stem/section_name` | `== topics` in `tavern.urd.md` → `tavern/topics` |
| Choice | `section_id/slugified-label` | "Ask about the harbour" → `tavern/topics/ask-about-the-harbour` |

IDs are stable across recompiles. If a writer adds a line, IDs do not change. This is critical for save files, testing assertions, and LSP references.

### 7.5 Input Limits

| Limit | Value |
|-------|-------|
| Maximum file size | 1 MB (1,048,576 bytes) |
| Maximum files per compilation unit | 256 |
| Maximum import depth | 64 levels |
| Maximum frontmatter nesting | 8 levels |
| Choice nesting: warning | 3 levels |
| Choice nesting: error | 4+ levels |

### 7.6 Error Recovery

The compiler does not stop at the first error. Each phase collects diagnostics and continues where possible. EMIT runs only when zero errors exist. Warnings do not block compilation.

Diagnostics include file path, line/column span, actionable message, and error code. Edit-distance suggestions are offered for unresolved references (e.g., *"@guard.trust is not a property on type Guard. Did you mean @guard.mood?"*).

---

## 8. Errors and Warnings

### Reading a Diagnostic

Every diagnostic the compiler emits has four parts:

| Field | Example | Description |
|-------|---------|-------------|
| `code` | `URD301` | Stable numeric code. Look it up in the tables below. |
| `severity` | `error` | One of `error`, `warning`, or `info`. |
| `message` | `Unresolved reference: @gaurd. Did you mean @guard?` | Human-readable explanation, often with a suggestion. |
| `span` | `tavern.urd.md:14:3–14:9` | File path, start line:column, end line:column. |

**Errors** block compilation — the compiler will not produce JSON output. **Warnings** are informational — output is still produced, but the warning usually points to something worth fixing. The compiler collects all diagnostics across all phases before reporting; it does not stop at the first error.

### Code Ranges

| Phase | Range | Errors | Warnings | Total |
|-------|-------|--------|----------|-------|
| PARSE | URD100–URD199 | 11 | 0 | 11 |
| IMPORT | URD200–URD299 | 13 | 1 | 14 |
| LINK | URD300–URD399 | 13 | 1 | 14 |
| VALIDATE | URD400–URD499 | 22 | 8 | 30 |
| EMIT | URD500–URD599 | 0 | 0 | 0 |
| **Total** | | **59** | **10** | **69** |

### PARSE Phase (URD100–URD199)

| Code | Severity | Trigger |
|------|----------|---------|
| URD101 | Error | Unclosed frontmatter block. |
| URD102 | Error | Tab character in source. Urd requires spaces. |
| URD103 | Error | File exceeds 1 MB size limit. |
| URD104 | Error | Frontmatter nesting exceeds 8 levels. |
| URD105 | Error | YAML anchor (`&name`) rejected. |
| URD106 | Error | YAML alias (`*name`) rejected. |
| URD107 | Error | YAML merge key (`<<:`) rejected. |
| URD108 | Error | YAML custom tag (`!!type`) rejected. |
| URD109 | Error | Block-style list (`- item`) rejected. Use `[item1, item2]`. |
| URD111 | Error | Unrecognised frontmatter syntax. |
| URD112 | Error | Unrecognised content syntax. |

### IMPORT Phase (URD200–URD299)

| Code | Severity | Trigger |
|------|----------|---------|
| URD201 | Error | Imported file not found. |
| URD202 | Error | Circular import detected. Full cycle path reported. |
| URD203 | Error | File stem collision across compilation unit. |
| URD204 | Error | Import depth exceeds 64 levels. |
| URD205 | Error | Compilation unit exceeds 256 files. |
| URD206 | Warning | Filename casing mismatch (case-insensitive filesystem). |
| URD207 | Error | Self-import. |
| URD208 | Error | Import path escapes project root. |
| URD209 | Error | Absolute import path. |
| URD210 | Error | Missing `.urd.md` extension. |
| URD211 | Error | Empty import path. |
| URD212 | Error | Invalid UTF-8 in imported file. |
| URD213 | Error | Permission denied reading imported file. |
| URD214 | Error | I/O error reading imported file. |

### LINK Phase (URD300–URD399)

| Code | Severity | Trigger |
|------|----------|---------|
| URD301 | Error | Unresolved reference (`@entity`, type, location, property). Suggestions via edit distance. |
| URD302 | Error | Duplicate entity or rule ID. |
| URD303 | Error | Duplicate type name. |
| URD304 | Error | Duplicate location ID (slugified heading collision). |
| URD305 | Error | Duplicate section name within a file. |
| URD306 | Error | Duplicate choice ID within a section. |
| URD307 | Error | Unknown entity type. Suggestions via edit distance. |
| URD308 | Error | Unknown property on type. |
| URD309 | Error | Unresolved jump target or section. |
| URD310 | Warning | Section shadows exit name. Use `-> exit:name`. |
| URD311 | Error | Unresolved `-> exit:name` jump. |
| URD312 | Error | Unresolved exit destination. |
| URD313 | Error | Empty slugified ID. |
| URD314 | Error | Construct outside location context. |

### VALIDATE Phase (URD400–URD499)

| Code | Severity | Trigger |
|------|----------|---------|
| URD401 | Error | Type mismatch in condition or effect. |
| URD402 | Error | Invalid enum override value. |
| URD404 | Error | Invalid `world.start` location. |
| URD405 | Error | Invalid `world.entry` sequence. |
| URD406 | Error | Both `target` and `target_type` declared. |
| URD407 | Error | Unknown action in sequence phase. |
| URD408 | Error | Unknown rule in sequence phase. |
| URD409 | Error | Invalid advance mode. |
| URD410 | Error/Warn | Choice nesting depth (warn at 3, error at 4+). |
| URD411 | Warning | Author manually set `urd:` field. |
| URD412 | Error | Player entity missing `mobile`/`container` traits. |
| URD413 | Error | Invalid property default value. |
| URD414 | Error | Empty enum values list. |
| URD415 | Error | Unknown ref target type. |
| URD416 | Error | Range min > max. |
| URD417 | Error | Range on non-numeric type. |
| URD418 | Error | Value outside declared range. |
| URD419 | Error | Ref type mismatch. |
| URD420 | Error | Ordering operator on non-numeric type. |
| URD422 | Error | Missing container trait. |
| URD423 | Error | Cross-file exhaustion check. |
| URD424 | Error | Arithmetic on non-numeric property. |
| URD425 | Error | Move without portable trait. |
| URD426 | Warning | Reveal on non-hidden property. |
| URD427 | Warning | Auto phase with player actions. |
| URD428 | Error | Empty sequence. |
| URD429 | Warning | Unrecognised property type string. |
| URD430 | Warning | Unreachable location (no path from `world.start`). |
| URD431 | Warning | Section named `end` shadows built-in terminal. |
| URD432 | Warning | Orphaned choice (condition impossible). |
| URD433 | Warning | Missing fallthrough (all one-shot, no exit). |
| URD434 | Warning | Section-exit shadowing. |

### Gate Cross-Reference

| Gate Check | Primary Codes |
|-----------|---------------|
| C1: Constrained frontmatter | URD104–URD109, URD111 |
| C2: Import resolution | URD201, URD209–URD211 |
| C3: Circular import detection | URD202 |
| C4: Duplicate entity IDs | URD302 |
| C5: Duplicate type names | URD303 |
| C6: Reference resolution | URD301, URD307–URD309, URD311–URD312 |
| C7: Property validation | URD401, URD402, URD413–URD420 |
| C8: `urd: "1"` injection | URD411 |
| C9: Nesting depth | URD410 |
| S1–S2: Entity/type checks | URD301, URD401 |
| S3: Unreachable location | URD430 |
| S4: Orphaned choice | URD432 |
| S5: Duplicate IDs | URD302–URD306 |
| S6: Missing fallthrough | URD433 |
| S7: Circular imports | URD202 |
| S8: Shadowed exit | URD434 |

---

## 9. FactSet Analysis IR

The FactSet is a normalised, immutable, deterministic set of typed tuples extracted after the LINK phase. Each tuple represents one atomic relationship — an exit connecting two locations, a condition reading a property, an effect writing a property. The facts are direct translations of what LINK resolved. No inference, no transitive closure, no runtime simulation.

### 9.1 Fact Types

| Fact Type | What It Represents | Key Fields |
|-----------|-------------------|------------|
| `ExitEdge` | An exit connecting two locations | `from_location`, `to_location`, `exit_name`, `is_conditional` |
| `JumpEdge` | A jump between dialogue sections | `from_section`, `target` (section, exit, or end) |
| `ChoiceFact` | A choice within a section | `section`, `choice_id`, `label`, `sticky`, condition/effect indices |
| `RuleFact` | A behavioural rule | `rule_id`, condition/effect indices |
| `PropertyRead` | A condition reading a property | `site`, `entity_type`, `property`, `operator`, `value_literal` |
| `PropertyWrite` | An effect writing a property | `site`, `entity_type`, `property`, `operator`, `value_expr` |

Every fact carries a `span` (file, line, column range) for source attribution.

### 9.2 FactSite

Every `PropertyRead` and `PropertyWrite` is tagged with a `FactSite` — the construct that owns the read or write:

| Site Kind | Meaning |
|-----------|---------|
| `Choice(id)` | A dialogue choice's condition or effect |
| `Exit(id)` | An exit's guard condition |
| `Rule(id)` | A rule's where-clause or effect |

This enables queries like "which choices read `Guard.mood`?" or "which rules write `Door.state`?"

### 9.3 PropertyDependencyIndex

A derived secondary index built from the FactSet in a single pass. Maps `(entity_type, property)` pairs to their read and write site indices.

| Method | Returns |
|--------|---------|
| `reads_of(key)` | All read indices for a property |
| `writes_of(key)` | All write indices for a property |
| `read_properties()` | All properties read anywhere |
| `written_properties()` | All properties written anywhere |

Use cases: dead property detection (written but never read), impact analysis (what changes if a property is removed), and dependency graphs for incremental revalidation.

### 9.4 WASM Serialisation

The WASM `compile_source()` response includes a `facts` field containing the serialised FactSet (see the full response shape in [Getting Started](#3-getting-started)). The six arrays — `reads`, `writes`, `exits`, `jumps`, `choices`, `rules` — correspond directly to the six fact types above. The browser playground's Analysis panel renders these facts by type.

---

## 10. Advanced Topics

### 10.1 The Containment Model

Urd uses a single spatial primitive: **containment**. Every entity exists inside exactly one container at any given time. A room holds a sword. A chest holds a sword. A player holds a sword. Same mechanism.

- "Pick up" is `> move @key -> player`.
- "Drop" is `> move @key -> here`.
- "Give to NPC" is `> move @key -> @guard`.
- "Does the player have it?" is `? @key in player`.
- "Is it in the same room?" is `? @key in here`.

**Player entity rule.** If a `@player` entity is explicitly declared in frontmatter, its type must include the `mobile` and `container` traits (URD412 if missing). If no `@player` entity is declared, the runtime should synthesise one with both traits, placed at the `world.start` location. In either case, the player is a mobile container — no special inventory system is needed.

### 10.2 Multi-File Projects

```
tavern-game/
  world.urd.md      # Types, entities, config (engineer)
  rules.urd.md      # NPC behaviour rules (engineer)
  tavern.urd.md     # Tavern location + dialogue (writer)
  harbour.urd.md    # Harbour location + events (writer)
```

- Writers' frontmatter is one line: `import: ./world.urd.md`
- Cross-location movement uses exits (`-> harbour`), not section jumps.
- Entity state (properties set via `>`) communicates across files. Section exhaustion does not.
- Each file's sections track their own state independently.

### 10.3 Source Is Not YAML

Urd frontmatter uses `---` delimiters and `key: value` pairs, which *look* like YAML. They are not. The parser is a purpose-built subset with strict constraints. Key differences:

| Feature | YAML | Urd Frontmatter |
|---------|------|----------------|
| Anchors and aliases (`&name`, `*name`) | Supported | Rejected (URD105, URD106) |
| Merge keys (`<<:`) | Supported | Rejected (URD107) |
| Custom tags (`!!type`) | Supported | Rejected (URD108) |
| Block-style lists (`- item`) | Supported | Rejected (URD109). Use `[item1, item2]`. |
| Implicit type coercion (`yes` → `true`) | Supported | Rejected. `true`/`false` only. |
| Tabs | Allowed | Rejected (URD102). Spaces only. |
| Nesting | Unlimited | 8 levels maximum (URD104) |

If you paste YAML from another tool and the compiler rejects it, check the diagnostic code — it will tell you exactly which construct is unsupported.

### 10.4 What You Can Rely On

These things are stable as of v0.1.7. We intend to keep them stable:

- **Diagnostic code numbers.** URD101–URD434 are assigned and will not be reassigned to different meanings.
- **ID derivation rules.** Entity IDs, section IDs, choice IDs, and location IDs are derived deterministically. The algorithm will not change.
- **JSON Schema version field.** `urd: "1"` is injected by the compiler. The value only changes for a new schema version.
- **Compiler output determinism.** Same source produces byte-identical JSON.

These things **will** change as the project matures:

- Source syntax (new symbols, revised grammar)
- JSON output structure (new blocks, renamed fields)
- FactSet field names and fact type shapes
- WASM API function signatures and response shapes
- Diagnostic message text (codes are stable; wording is not)

### 10.5 v1 Boundaries

v1 is the complete foundation. A runtime that fully supports v1 implements: all eight JSON blocks, all property types, all four visibility levels (including `owner` and `conditional` which have JSON support but no source syntax yet), all entity traits, the containment model, all five effect types, all five trigger types, the `select` block with deterministic random selection, all six advance modes, dialogue with sticky/one-shot choices, `any:` OR conditions, and event sourcing.

**Deferred to future versions:**

| Feature | v1 Workaround |
|---------|---------------|
| Cross-file section jumps | Use exits for cross-file movement. Bridge sections for shared context. |
| Lambda functions | Express logic declaratively using rules, conditions, effects. |
| Owner visibility transfer | Use `owner` for static ownership only. |
| Cross-file exhaustion | Accept independent counters in duplicated sections. |

### 10.6 Permanent Exclusions

The following are permanently excluded from the schema (not deferred — they will never be part of Urd):

- Verb synonyms and natural language parsing
- Conditional text rendering (presentation layer concern)
- Failure experience design (UX layer concern)
- Time and pacing mechanics (future extension, not core schema)
- Persistence format specification
- On-attempt rules (rules fire on success, not attempt)

---

## 11. Quick Reference

### Writer Pattern Reference

| I want to... | Pattern |
|--------------|---------|
| Create a hub conversation | `== topics` with choices that `-> topics` |
| Gate a choice on state | `? @entity.prop == value` before the choice |
| Change the world on selection | `> @entity.prop = value` after the choice |
| Make a choice one-time | `* choice text` |
| Make a choice repeatable | `+ choice text` |
| Show fallthrough text | Plain text after the choice block |
| Check if player has an item | `? @item in player` |
| Check if item is in the room | `? @item in here` |
| Transfer an object | `> move @item -> @npc` |
| End the conversation | `-> end` |
| Move to another location | `-> location_name` (requires a declared exit) |
| Branch on NPC mood | `? @npc.mood == hostile` / `? @npc.mood == neutral` as separate blocks |
| Gate on any of several conditions | `? any:` + indented conditions |
| Drop an item | `> move @item -> here` |

### Compiler Mapping Table

| Syntax | Compiles To |
|--------|-------------|
| `---`...`---` | `world`, `types`, `entities` blocks |
| `import: path` | Resolved types/entities merged into scope |
| `# Name` | Entry in `locations` block |
| `[@entity, ...]` | `contains` field of enclosing location |
| `## Name` | Entry in `sequences` block |
| `### Name` | Phase within enclosing sequence |
| `(auto)` | `auto: true` on the phase |
| `* label` | One-shot choice (`sticky: false`) |
| `+ label` | Sticky choice (`sticky: true`) |
| `* label -> @target` | Choice targeting a specific entity (compiles to action with `target`) |
| `* label -> any Type` | Choice targeting any entity of a type the player can reach (compiles to action with `target_type`) |
| `? expr` | Entry in `conditions` list (AND-ed) |
| `? any:` | `any:` block in conditions |
| `> entity.prop = value` | `set` effect |
| `> move @entity -> container` | `move` effect |
| `> reveal @entity.prop` | `reveal` effect |
| `> destroy @entity` | `destroy` effect |
| `== name` | Section in `dialogue` block |
| `-> name` | `goto` field |
| `-> end` | Dialogue exit (no `goto`) |
| `! text` | `blocked_message` on exit or action |
| `rule name:` | Entry in `rules` block |
| `// text` | Stripped. Not in JSON. |

---

*This manual is the practical entry point. If you need formal definitions or design rationale, the individual specifications and briefs contain the deeper detail.*

*End of Reference Manual*
