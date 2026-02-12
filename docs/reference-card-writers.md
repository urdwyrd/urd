---
title: "Urd Schema Markdown — Writer Reference Card"
slug: "reference-card-writers"
description: "Quick reference for writers authoring .urd.md files. Covers the seven-symbol vocabulary, file structure, dialogue mechanics, nesting rules, and common patterns."
category: "authoring"
format: "Reference Card"
date: "2026-02-12"
status: "v1.0"
order: 1
tags:
  - reference
  - writers
  - schema-markdown
  - quick-start
details:
  - "Seven-symbol vocabulary at a glance"
  - "File structure, dialogue mechanics, and nesting rules"
  - "Common patterns and quick-start examples"
---

> **Document status: INFORMATIVE**
> Quick reference card for writers. All rules are derived from the normative Schema Markdown Syntax Specification and Schema Specification.
> Single canonical copy. February 2026 draft.

# URD

## Writer Reference Card

*Everything a writer needs on one page*

urd.dev · February 2026

## Your File

```
---
import: ./world.urd.md
---

# The Rusty Anchor

A low-ceilinged tavern thick with pipe smoke.

[@arina]

== topics

@arina: What'll it be, stranger?

+ Ask about the harbor
  @arina: Quiet today. Too quiet.
  > @arina.trust + 5
  -> topics

* Ask about the missing ship
  ? @arina.trust > 50
  @arina: The Selene didn't sink. She was taken.
  -> topics

* Leave -> harbor
```

Three lines of frontmatter. One import. Everything else is narrative.

## The Seven Symbols

| Symbol | Name | What It Does | Example |
|--------|------|--------------|---------|
| `@` | Entity reference | References characters, objects, locations | `@guard`, `@door_1` |
| `?` | Condition | Gates content on world state | `? @guard.mood == neutral` |
| `>` | Effect | Changes the world | `> @guard.mood = neutral` |
| `*` | One-shot choice | Disappears after selection | `* Ask about the ship` |
| `+` | Sticky choice | Stays available on revisit | `+ Ask about the harbor` |
| `->` | Jump | Navigates to a section or location | `-> topics`, `-> harbor` |
| `//` | Comment | Ignored by the compiler | `// hub prompt` |

## Dialogue Mechanics

### Choice Types

| Marker | Behaviour | Use For |
|--------|-----------|---------|
| `*` | Consumed after selection. Never shown again. | Revelations, one-time offers, questions that shouldn't repeat. |
| `+` | Available every time the section is entered. | Recurring topics, shop menus, small talk. |

### Exhaustion

When all choices are consumed or gated, the conversation **falls through** to the text after the choice block. If there's no fallthrough text, the conversation ends.

```
== topics
@arina: What'll it be?

* One-time question
  @arina: Here's the answer.
  -> topics

@arina: Suit yourself. I've got glasses to clean.
// ↑ This text appears when all choices are gone.
```

Test for exhaustion from another section: `? topics.exhausted`

### Conditions and Effects

```
? @arina.trust > 50          // Gate: only show if true
? @coin_purse in player       // Containment check
? @coin_purse not in player   // Negated containment
? @key in here                // "here" = player's current location

? any:                         // OR: any one condition being true is enough
  @guard.mood == hostile
  @guard.mood == suspicious

> @arina.trust = 75           // Set to value
> @arina.trust + 5            // Increment
> move @coin_purse -> @arina  // Transfer object
> move @key -> here           // Drop item in current location
> reveal @door.prize          // Unhide a property
> destroy @rusty_key          // Remove from world
```

Multiple `?` lines are AND-ed. All must be true. Use `? any:` for OR logic.

### Entity Speech and Narration

```
@arina: What'll it be?        // Dialogue (colon = speech)
@arina leans in close.         // Stage direction (no colon = narration)
```

Plain text with no marker is prose description.

## Nesting Rules

### Indentation for Shallow Branches (1–2 levels)

```
* Ask about the prisoner
  @halvard: What prisoner?

  * Press him
    @halvard: Cell three.
    > player.knows_cell = true
    -> interrogation

  * Back off -> interrogation
```

Two-space indent per level. Content under a choice is indented one level.

### Sections for Deep Structure and Loops

```
== interrogation
@halvard: You've got questions. Make them quick.

* Ask about the prisoner -> press_details
* Try to bribe him -> bribe
* I'm done here -> farewell

== press_details
? @halvard.mood == neutral
@halvard: Cell three.
> player.knows_cell = true
-> interrogation
```

### Depth Limits

| Depth | Compiler Behaviour |
|-------|--------------------|
| 1–2 levels | Normal. |
| 3 levels | Warning: *"Consider breaking into a labeled section."* |
| 4+ levels | Error. File does not compile. |

## Sections and Jumps

| Syntax | Meaning |
|--------|---------|
| `== name` | Declare a section. Names: lowercase, digits, underscores. Unique within file. |
| `-> name` | Jump to a section in this file. |
| `-> location` | Move player to a location (exit). |
| `-> exit:name` | Explicitly target an exit when a section shadows the name. |
| `-> end` | Exit dialogue mode. |

**Priority:** If a section and an exit share a name, `->` targets the section. Use `-> exit:name` to force the exit.

## File Structure

### Headings

| Heading | Compiles To |
|---------|-------------|
| `# Location Name` | A location in the `locations` block. |
| `## Sequence Name` | A sequence in the `sequences` block. |
| `### Phase Name` | A phase within the enclosing sequence. |
| `== section_name` | A dialogue section. |

### Entity Placement

`[@arina, @barrel]` after a location heading declares what's inside that location. Square brackets list entities present in this location at world start.

### Blocked Messages

`! The door is locked.` — shown when a condition fails on an exit or action.

## Multi-File Projects

```
tavern-game/
  world.urd.md      ← Types, entities, config (engineer)
  rules.urd.md      ← NPC behaviour rules (engineer)
  tavern.urd.md     ← Tavern location + dialogue (writer)
  harbor.urd.md     ← Harbor location + events (writer)
```

- Your frontmatter is one line: `import: ./world.urd.md`
- Cross-location movement uses exits (`-> harbor`), not section jumps.
- Cross-file section jumps are **not supported in v1**.
- Entity state (properties set via `>`) communicates across files. Section exhaustion does not.

## What You Don't Need to Know

- How types are defined (engineers handle this).
- How rules work (engineers encode NPC behaviour; you see the results).
- How JSON works (the compiled output is for runtimes).
- How the runtime evaluates conditions (it's automatic).

> **If you need to learn the schema to do your job, the tooling has failed.** Report it as a bug.

## Quick Pattern Reference

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
| Move to another location | `-> location_name` |
| Branch based on NPC mood | `? @npc.mood == hostile` / `? @npc.mood == neutral` as separate blocks |
| Gate on any of several conditions | `? any:` + indented conditions |
| Drop an item in the current room | `> move @item -> here` |

*Authoritative source: Schema Markdown Syntax Specification. This card is a convenience summary.*

*End of Reference Card*
