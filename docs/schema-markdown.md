---
title: "Schema Markdown Syntax Specification"
slug: "schema-markdown"
description: "The writer-facing authoring format. Feels as clean as ink for pure dialogue but compiles directly to the world schema. Seven symbols: @ for entity references, ? for conditions, > for effects, * and + for choice types, → for section jumps, == for section headers."
category: "authoring"
format: "Syntax Specification"
date: "2026-02-12"
status: "v0.1 complete"
order: 1
tags:
  - syntax
  - authoring
  - writers
  - compiler
  - schema-markdown
details:
  - "Seven-symbol vocabulary for narrative authoring"
  - "Separation of writer, designer, and engineer concerns"
  - "Indentation-scoped nesting with explicit jump targets"
  - "Validated against four test cases including complex interrogation"
---

> **Document status: NORMATIVE**
> Defines the Schema Markdown writer-facing syntax (`.urd.md` files) and its compilation rules to the Urd World Schema JSON. This is the authoritative reference for compiler implementers and writer tooling.
> Single canonical copy. February 2026 draft.

# URD

## Schema Markdown Syntax Specification

*A writer facing syntax that compiles to the Urd World Schema*

*As clean as ink for pure dialogue. Schema native by design.*

urd.world

February 2026

## Introduction

Schema Markdown is the writer facing syntax of the Urd framework. It compiles to the Urd World Schema JSON format, which runtimes consume directly. The syntax is designed to feel like prose with light annotation for writers, while producing typed, validated, engine consumable data for engineers.

The bar, from the product vision:

> *As clean as ink for pure dialogue. Schema native by design.*

This document specifies the syntax. It leads with examples, followed by the complete vocabulary, authorship model, nesting rules, and design rationale. The syntax has been validated against four test cases (a dialogue scene, a sequence driven game show, a freeform spatial puzzle, and a complex interrogation stress test).

## Who Writes What

Schema Markdown serves writers, narrative designers, and engineers, but they don't all touch the same parts of a file.

| Layer | Who Owns It | What It Looks Like | Changes How Often |
|-------|-------------|-------------------|-------------------|
| Type definitions | Engineers | Structured property schemas in dedicated files. | Rarely. |
| Entity declarations | Engineers + Designers | Entity instances with property overrides. | Occasionally. |
| Narrative content | Writers | Prose, dialogue, choices, conditions, effects. | Constantly. |
| Rules (NPC behavior) | Engineers | Constraint based behavioral rules. | Occasionally. |
| Sequences (game flow) | Narrative Designers | Heading based phase structures. | Moderately. |

**Guiding principle:** Writers should never need to edit a type definition or write a rule block. If the syntax forces them to, the tooling has failed.

## File Architecture

In a real project, the frontmatter in a writer's file is minimal: a one line import pointing to the project's shared schema.

### A Writer's File

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

  ? @arina.trust <= 50

  @arina: I don't know what you're talking about.

  -> topics

* Leave -> harbor
```

Three lines of Urd frontmatter. One import. Everything else is narrative.

> **This is not YAML.** The frontmatter block uses YAML like syntax but is parsed by the Urd compiler under strict constraints. Anchors (&name), aliases (*name), and custom tags are rejected. Strings that look like booleans or numbers must be quoted: `"1"` not `1`, `"yes"` not `yes`. If you use an unsupported feature, the compiler tells you why: *"Anchors are not supported in Urd frontmatter (line 3). Write the value explicitly instead."*

### An Engineer's File

```
---
world: tavern-game

types:
  Barkeep [interactable]:
    name: string
    trust: int(0, 100) = 30
    ~knows_secret: bool = true

entities:
  @arina: Barkeep { name: "Arina" }
---
```

*Note: the `urd` version field does not appear in source files. The compiler sets it automatically in the compiled JSON output. All examples in this document show source format (`.urd.md`), where `urd` is absent. For compiled JSON examples showing the required `urd: "1"` field, see the Schema Specification.*

### Project Directory

```
tavern-game/
  world.urd.md      # Types, entities, world config (engineer)
  rules.urd.md      # NPC behavioral rules (engineer)
  tavern.urd.md     # Tavern location + dialogue (writer)
  harbor.urd.md     # Harbor location + events (writer)
  docks.urd.md      # Docks location + puzzle (writer)
```

- **Writers never touch schema definitions.** Their frontmatter is a one line import.
- **Engineers control the schema.** Type changes are code reviewed and validated.
- **Imports are explicit.** The compiler knows exactly what's in scope.

> **Note:** The examples in this document include types and entities inline because they are self contained demos. In a real project, inline frontmatter is the exception, not the pattern.

### Multi File Authoring Pattern

A real project splits content across files by location or narrative scope. Here is the authoring pattern:

- **One shared schema file.** An engineer creates `world.urd.md` containing all type definitions, shared entity declarations, and world metadata. This is the single source of truth for the data model.
- **One rules file per domain.** Engineer authored rule blocks live in dedicated files like `rules.urd.md` or `npc-rules.urd.md`. Writers never edit these.
- **One file per location or scene.** Each writer file starts with `import: ./world.urd.md` and contains narrative content for one location. The file declares which entities are present (`[@arina, @barrel]`) and defines the dialogue and actions for that space.
- **Cross location movement uses exits, not section jumps.** A writer sends the player to the harbor with `* Leave -> harbor` (an exit), not `-> harbor.urd.md/docks` (a cross file section jump, which is not supported in v1). Each file is self contained for dialogue; movement between files is handled by the location/exit system.
- **If two scenes need shared dialogue continuity,** use a sequence phase boundary to bridge them. The sequence system (`## Phase Name`) can advance the world between locations. Alternatively, duplicate a small bridge section in both files. **Note:** duplicated sections track their own exhaustion state independently. Consuming choices in one file does not affect the duplicate in the other. Use entity state (properties set via `>` effects) to communicate across files, not section state.

> **Versioning responsibility.** Authors do not set the urd version field manually. The compiler sets it automatically to `"1"`. If an author specifies a manual value, the compiler warns and overrides.

### v1 Feature Coverage

This table clarifies what is available in v1 at each layer.

| Feature | Writer Syntax | Compiler Emits | v1 Runtime Must Support |
|---------|---------------|----------------|------------------------|
| Entities, types, properties | Allowed | world, types, entities blocks | Yes |
| Locations, exits, containment | Allowed | locations block | Yes |
| Sequences, phases | Allowed | sequences block | Yes |
| Conditions, effects | Allowed | Inline in locations, sequences, actions, and dialogue | Yes |
| Actions | Allowed | actions block (top level) | Yes |
| Rules | Allowed | rules block | Yes |
| Dialogue sections (`== name`) | Allowed | dialogue block | Yes |
| Sticky/one shot choices | Allowed | sticky field in dialogue choices | Yes |
| Exhaustion condition | Allowed | Runtime evaluated; no JSON field | Yes |
| Cross file section jumps | Deferred to future version | N/A | N/A |
| OR conditions (`any:`) | Allowed (`? any:` block) | `any:` condition block in compiled JSON | Yes |
| Lambda functions | Deferred to future version | N/A | N/A |

> **v1 is the complete foundation.** A compliant v1 runtime must support the full schema including dialogue. There is no partial v1 that excludes dialogue. A runtime that has not yet implemented dialogue is an incomplete v1 runtime, not a different version.

## Writer Promises

This page lists what a writer can rely on without learning the schema, understanding JSON, or reading any other specification document. These are **guarantees**, not guidelines.

### What You Write Is What You See

- **Your file is prose.** Below the Urd frontmatter (the `---` block at the top), everything is narrative content. You write dialogue, descriptions, and choices in plain text with light annotations.
- **Three lines of frontmatter.** In a real project, your frontmatter is typically one line: `import: ./world.urd.md`. The types, entities, and rules live in engineer managed files.
- **Seven symbols cover all of writing.** `@` for characters and objects, `?` for conditions, `>` for effects, `*` and `+` for choices, `->` for jumps, `//` for comments. Everything else is plain text.

### What Happens When You Save

- **The compiler checks your file immediately.** If you reference an entity that doesn't exist, set a property to an invalid value, or leave a choice block with no exit, the editor will underline the problem and tell you what's wrong.
- **Errors are specific and actionable.** Not "parse error on line 47" but `"@guard.trust is not a property on type Guard. Did you mean @guard.mood?"`
- **Your file is never modified by the compiler.** Compilation produces a separate `.urd.json` file. Your `.urd.md` source is always your source.

### How Dialogue Works

- **A choice marked `*` disappears after the player picks it.** Use for revelations, one time offers, and questions that shouldn't repeat.
- **A choice marked `+` stays available every time.** Use for recurring topics, shop menus, and small talk.
- **When all choices are gone, the conversation falls through.** The text after the choice block is what the player sees. If there's no text, the conversation ends.
- **Conditions gate what appears.** A `?` line before a choice means the choice only shows when the condition is true. If the condition is false, the player never sees that choice.
- **Effects change the world.** A `>` line after a choice means something changes when the player picks it. The change persists.

### What You Don't Need to Know

- **How types are defined.** Engineers define types in separate files. You reference entities that already exist.
- **How rules work.** Rules govern NPC behaviour (like Monty Hall's constrained door opening). Engineers write them. You see the results: actions appear or disappear based on world state. If you need a rule, describe the behaviour you want and an engineer encodes it.
- **How JSON works.** The compiled output is for runtimes, not for you.
- **How the runtime evaluates things.** You describe the world. The runtime figures out what happens. Conditions are checked automatically. Effects apply automatically. You never write control flow.

> **If you need to learn the schema to do your job, the tooling has failed.** This is the design principle. Report it as a bug.

## The Complete Syntax

The full vocabulary of Schema Markdown. The Author column indicates who uses each syntax element in production.

| Syntax | Meaning | Example | Author |
|--------|---------|---------|--------|
| `---`...`---` | Urd frontmatter. Metadata, imports, types, entities. | `import: ./world.urd.md` | Engineer |
| `import: path` | Import types and entities from another file. | `import: ./world.urd.md` | Writer |
| `# Heading` | Location. A spatial container. | `# The Rusty Anchor` | Writer |
| `## Heading` | Scene or sequence within a location. | `## The Game` | Designer |
| `### Heading` | Phase within a sequence. | `### Choose a Door` | Designer |
| `(auto)` | Auto-advancing phase (no player action required). | `### Reveal (auto)` | Designer |
| `@id` | Entity reference. | `@guard`, `@door_1` | Writer |
| `@id: text` | Entity speech (dialogue). | `@arina: What'll it be?` | Writer |
| `@id text` | Stage direction referencing entity. | `@arina leans in close.` | Writer |
| `* choice` | One shot choice. Disappears after selection. | `* Ask about the ship` | Writer |
| `+ choice` | Sticky choice. Remains available after selection. | `+ Ask about the harbor` | Writer |
| `? expr` | Condition. Must be true for what follows. | `? @guard.mood == neutral` | Writer |
| `? any:` | OR condition block. Any sub-condition being true validates. | `? any:` + indented conditions | Writer |
| `> effect` | State mutation. | `> @guard.mood = neutral` | Writer |
| `~prop` | Hidden property (in type definitions). | `~prize: enum(goat, car)` | Engineer |
| `== name` | Labeled section (dialogue structure). | `== topics` | Writer |
| `-> target` | Jump to section, location, or exit. | `-> topics`, `-> harbor` | Writer |
| `-> exit:name` | Explicitly target an exit (when shadowed by a section). | `-> exit:topics` | Writer |
| `! text` | Blocked message (when a condition fails). | `! The door is locked.` | Writer |
| `// text` | Comment. Stripped during compilation. | `// hub prompt` | Writer |
| `rule name:` | NPC behavioral rule. | `rule monty_reveals:` | Engineer |

Plain text outside any marker is narrative prose: descriptions, stage directions, flavour text. It compiles to description fields in the schema.

## Choices: One Shot and Sticky

Choices come in two forms, borrowing directly from ink's proven `*` / `+` distinction:

### One Shot Choices (`*`)

`*` marks a choice that disappears after the player selects it. If the player returns to this point (via a hub loop), the choice is no longer available.

```
* Ask about the missing ship

  @arina: The Selene didn't sink. She was taken.

  -> topics
```

Use one shot choices for topics that should only be raised once: revelations, one time offers, irreversible decisions.

### Sticky Choices (`+`)

`+` marks a choice that remains available every time the player returns. It is never consumed.

```
+ Ask about the harbor

  @arina: Quiet today. Too quiet.

  -> topics

+ Order a drink

  @arina pours an ale.

  -> topics
```

Use sticky choices for repeatable interactions: small talk, shop menus, recurring actions. In a hub and spoke conversation, sticky choices are the topics the player can always return to.

### Mixing Them

```
== topics

@arina: What do you want to know?

+ Ask about the harbor               # always available

  @arina: Quiet today. Too quiet.

  > @arina.trust + 5

  -> topics

* Ask about the missing ship          # disappears once asked

  ? @arina.trust > 50

  @arina: The Selene didn't sink. She was taken.

  ? @arina.trust <= 50

  @arina: I don't know what you're talking about.

  -> topics

* Confront her about the ledger       # disappears once asked

  ? @ledger in player

  @arina goes pale.

  @arina: Where did you get that?

  -> ledger_confrontation

* Leave -> harbor
```

@arina: Suit yourself. I've got glasses to clean.

### Exhaustion

When all `*` one shot choices in a section have been consumed and only `+` sticky choices remain, the hub continues normally with the remaining sticky options. When **all** choices (both `*` and `+`) have been consumed or are gated by failing conditions, the section is **exhausted**.

On exhaustion, content falls through to the text after the choice block. In the example above, if the player has asked every question and then left, any return would show: "Suit yourself. I've got glasses to clean."

**Linter rule:** If a section contains only one shot choices and has no fallthrough text after them, the linter warns: "All choices can be exhausted with no fallthrough content. Add text after the choices, a sticky choice, or a jump." This prevents accidental dead menus.

The exhaustion state is also available as a condition: `? topics.exhausted` evaluates to true when all choices in the named section have been consumed or gated. Always use the concrete section name (e.g., `? topics.exhausted`), not a generic placeholder. This allows other parts of the world to react to the conversation being "finished." In compiled JSON, the fallthrough content appears in the `on_exhausted` field of the section, not as a boolean.

### Normative Semantics

The following rules are normative. Any runtime that executes Urd dialogue **must** implement them exactly as stated. These are not illustrative examples. They are the contract between authors and runtimes.

**One shot choice (`*`).** A one shot choice is available until selected. Once selected, it is consumed and never presented again in that playthrough. Consumed state persists across revisits to the section. The runtime tracks consumed choices by their stable choice ID, which is world-unique because it includes the file stem (see Stable IDs for Sections and Choices).

**Sticky choice (`+`).** A sticky choice is available every time the section is entered. It is never consumed. It may be selected repeatedly. If the choice has conditional gating, the condition is re evaluated on each visit.

**Visited vs consumed.** A sticky choice is considered "visited" after first selection. The visited state can be used to vary the choice's response text on subsequent selections (e.g., shorter acknowledgment on revisit). This is distinct from consumed: visited choices remain available.

**Exhaustion.** A section is exhausted when every choice is either consumed (one shot, already selected) or gated (conditions evaluate to false). On exhaustion, content falls through to the first text block after the choice block in the source file. If there is no fallthrough text, the dialogue ends. The runtime must never present an empty choice menu. The exhaustion condition uses the canonical form `? <section_name>.exhausted` (e.g., `? topics.exhausted`), where `<section_name>` resolves to a declared section identifier in scope. Section names in exhaustion conditions resolve using the same rules as `->` jumps: they refer to sections declared in the current file only. Cross-file section exhaustion is not supported in v1. The compiled JSON represents the fallthrough content in an `on_exhausted` field; it does not contain an exhausted boolean. Whether a section is exhausted is always a runtime-evaluated predicate.

**Section scope.** Sections are scoped to the file in which they are declared. A section name must be unique within its file. The compiled section ID is namespaced by file stem (e.g., `tavern/topics` for `== topics` in `tavern.urd.md`), making it world-unique in compiled JSON. However, `-> section_name` in writer syntax only targets sections in the current file in v1. Cross file section jumps are not supported in v1 and are listed in Remaining Open Items.

## Nesting: The Hybrid Model

Dialogue nesting uses two complementary mechanisms: indentation for shallow local branching and labeled sections for deep structure and loops. This matches how experienced ink writers already work. Local nesting for immediate sub choices, structural jumps for navigation.

### Indentation for Local Branches

Content under a `*` or `+` choice is indented two spaces. Sub choices indent further. This handles the common case: one to two levels of branching within a single exchange.

```
* Ask about the prisoner

  @halvard: What prisoner?

  * Press him

    ? @halvard.mood == neutral

    @halvard: Cell three. But you didn't hear it from me.

    > player.knows_cell = true

    -> interrogation

    ? @halvard.mood == hostile

    @halvard: I said, what prisoner?

    -> interrogation

  * Back off -> interrogation
```

### Sections for Deep Structure

When branching exceeds two levels, or when a conversation needs hubs and loops, the writer breaks out to a labeled section with `== name`. Sections are flat: no indentation required. Jumps are explicit with `-> name`.

```
== interrogation

@halvard: You've got questions. Make them quick.

* Ask about the prisoner

  @halvard: What prisoner?

  * Press him

    -> press_details

  * Back off -> interrogation

* Try to bribe him -> bribe

* I'm done here -> farewell

== press_details

? @halvard.mood == neutral

@halvard: Cell three. But you didn't hear it from me.

> player.knows_cell = true

-> interrogation

? @halvard.mood == hostile

@halvard: I said, what prisoner?

-> interrogation
```

### Nesting Rules

- **Two space indent per level.** Tabs are not permitted.
- **Content under a choice is indented one level.** This includes dialogue, conditions, effects, sub choices, and jumps.
- **Maximum depth: two levels.** The compiler emits a warning at three levels of indentation and an error at four. At three levels, the message is: *"Nesting depth 3 at line 47. Consider breaking into a labeled section with == for readability."* At four levels, the file does not compile. This is a maintainability constraint, not a style preference. Deeply nested dialogue is unreadable, untestable, and unmergeable in version control. The LSP and editor tooling surface these warnings in real time.
- `== name` **declares a section.** Names follow entity ID rules: lowercase, digits, underscores. Must be unique within the file. **Section resolution is strictly file-local in v1.** A `-> name` jump can only target sections declared in the same file. The `-> exit:` prefix is only needed when a section shadows an exit in the same file.
- `-> name` **jumps to a section.** Can appear at any indentation level. Ends the current branch.
- **Disambiguation: sections take priority over exits.** If a file contains `== topics` and the enclosing location has an exit named `topics`, then `-> topics` inside that file resolves to the section, not the exit. To target the exit explicitly, use `-> exit:topics`. The `exit:` prefix is reserved for this purpose and is only needed when a section shadows an exit. The compiler emits a warning when a section name shadows an exit name: *"Section 'topics' shadows exit 'topics' in this location. Use -> exit:topics to target the exit."*

**Normative resolution rule.** An unqualified `-> name` resolves in the following priority order:

1. **Section.** If a section with that name exists in the current file, the jump targets that section.
2. **Exit.** If no matching section exists, and the enclosing location has an exit with that name, the jump targets that exit.
3. **Compile error.** If neither a section nor an exit matches, the compiler emits an error: *"Unresolved jump target 'name' at line N. No section or exit with this name exists in scope."*

If the same name matches both a section and an exit, resolution always favours the section (rule 1), and the compiler emits the shadowing warning described above. The explicit `-> exit:name` form bypasses this priority and always targets an exit. If two sections in the same file share a name, it is a compile error (section names must be unique within a file).

```
// Example: section 'harbor' shadows the exit 'harbor'

== harbor

@arina: The harbor? Stay clear of it tonight.

* Ask why

  @arina: Trust me on this one.

  -> harbor                 // jumps back to section 'harbor'

* Go there anyway

  -> exit:harbor            // moves player to the harbor location
```

- **Sections without a terminal `->` fall through** to text after the choice block, or to the next section. The linter warns on ambiguous fallthrough.
- `-> end` **ends the conversation.** Control returns to the location's action list or advances the sequence phase.

> **When to use which:** If you can see the whole exchange on one screen without horizontal scrolling, use indentation. If you can't, or if the conversation loops back, use a section.

## Example 1: A Dialogue Scene

A conversation with a barkeep featuring a hub and spoke topic menu, conditional reveals, state mutation, and a mix of sticky and one shot choices.

> **Authorship note:** In a real project, the type and entity declarations would live in a shared `world.urd.md` file. They're inline here for self containment.

```
---
world: tavern-talk
start: tavern

types:
  Barkeep [interactable]:
    name: string
    trust: int(0, 100) = 30
    ~knows_secret: bool = true

entities:
  @arina: Barkeep { name: "Arina" }
---

# The Rusty Anchor

A low-ceilinged tavern thick with pipe smoke and the smell of salt.

[@arina]

== topics

@arina: What'll it be, stranger?

+ Ask about the harbor

  @arina: Quiet today. Too quiet, if you ask me.

  > @arina.trust + 5

  -> topics

* Ask about the missing ship

  ? @arina.trust > 50

  @arina leans in close.

  @arina: The Selene didn't sink. She was taken.

  > @arina.trust + 10

  -> topics

  ? @arina.trust <= 50

  @arina: I don't know what you're talking about.

  She turns away and starts wiping the counter.

  -> topics

+ Buy her a drink

  @arina smiles.

  @arina: Well aren't you a gentleman.

  > @arina.trust + 20

  -> topics

* Leave -> harbor
```

@arina: Suit yourself. I've got glasses to clean.

### What to Notice

**Sticky vs one shot.** "Ask about the harbor" and "Buy her a drink" use `+`, always available. "Ask about the missing ship" uses `*`, once asked, it's gone. "Leave" uses `*` because leaving ends the conversation.

**The hub.** `== topics` names the hub. Every branch that should return to the menu ends with `-> topics`. This is explicit and unambiguous.

**Exhaustion fallthrough.** The final line ("Suit yourself...") is the exhaustion text. If the player has asked every one shot question and doesn't pick a sticky option, this is what they see.

**Entity scoped state.** `@arina.trust` is a typed property on a typed entity. Two NPCs can each have trust without naming conflicts.

**Stage direction vs speech.** `@arina leans in close.` (no colon) is narration. `@arina: The Selene didn't sink.` (with colon) is speech. The compiler distinguishes them.

### The Same Scene in ink

```
VAR trust = 30

=== tavern ===

"What'll it be, stranger?"

+ [Ask about the harbor]

  "Quiet today. Too quiet, if you ask me."

  ~ trust += 5

  -> tavern

* {trust > 50} [Ask about the missing ship]

  She leans in close.

  "The Selene didn't sink. She was taken."

  ~ trust += 10

  -> tavern

* {trust <= 50} [Ask about the missing ship]

  "I don't know what you're talking about."

  She turns away and starts wiping the counter.

  -> tavern

+ [Buy her a drink]

  She smiles.

  "Well aren't you a gentleman."

  ~ trust += 20

  -> tavern

* [Leave]

  -> harbor

"Suit yourself. I've got glasses to clean."
```

**Line count is comparable.** The Schema Markdown version has a frontmatter block that ink doesn't need, but below the frontmatter, the dialogue density is nearly identical. The overhead buys typed entities, attributed speech, and entity scoped state.

**ink duplicates the conditional choice.** The "missing ship" topic requires two separate choice lines in ink (one for trust > 50, one for trust ≤ 50) because ink conditions are choice level, not content level. Schema Markdown puts the conditions inside the choice block, avoiding the duplication.

## Example 2: The Monty Hall Problem

A structured, sequence driven game show demonstrating hidden state, constrained NPC behavior, and phased progression. Sequences use the heading hierarchy rather than dialogue sections.

```
---
world: monty-hall
start: stage
entry: game

types:
  Door [interactable]:
    ~prize: enum(goat, car)
    state: enum(closed, open) = closed
    chosen: bool = false

  Host:
    name: string

entities:
  @door_1: Door { prize: car }
  @door_2: Door { prize: goat }
  @door_3: Door { prize: goat }
  @monty: Host { name: "Monty Hall" }
---

# Stage

A game show stage with three closed doors.

[@door_1, @door_2, @door_3, @monty]

## Game

### Choose

Pick a door.

* Pick a door -> any Door

  ? target.state == closed

  ? target.chosen == false

  > target.chosen = true

### Reveal (auto)

@monty opens a door that hides a goat.

rule monty_reveals:
  @monty selects target from [@door_1, @door_2, @door_3]
  where target.prize != car
  where target.chosen == false
  where target.state == closed
  > target.state = open

### Switch or Stay

Monty opened a door with a goat. Switch or stay?

* Switch to the other closed door -> any Door

  ? target.state == closed

  ? target.chosen == false

  > target.chosen = true

* Stay with your current choice

### Resolve (auto)

> reveal @door_1.prize

> reveal @door_2.prize

> reveal @door_3.prize
```

> **Authorship note:** The rule block is engineer authored. In a real project, it would live in a dedicated rules file. Writers describe the behavior; engineers encode it.

## Example 3: The Two Room Key Puzzle

A freeform spatial puzzle with no sequences or dialogue sections. Progression emerges from action conditions and the containment model.

```
---
world: two-room-key
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

---

## Example 4: A Complex Interrogation

A multi topic interrogation demonstrating the hybrid nesting model at production complexity: hub and spoke with state mutation, a bribe using containment, conditional sub branches, OR conditions (`? any:`), and a state dependent farewell.

```
---
import: ./world.urd.md

types:
  Guard [interactable, mobile, container]:
    name: string
    mood: enum(hostile, suspicious, neutral, nervous) = hostile

entities:
  @halvard: Guard { name: "Halvard" }
  @coin_purse: Item { name: "Coin Purse" }
---

== interrogation

@halvard: You've got questions. Make them quick.

* Ask about the prisoner

  @halvard: What prisoner?

  @halvard stares at you, unblinking.

  * Press him

    ? @halvard.mood == neutral

    @halvard sighs.

    @halvard: Cell three. But you didn't hear it from me.

    > @halvard.mood = nervous

    > player.knows_cell = true

    -> interrogation

    ? @halvard.mood == hostile

    @halvard: I said, what prisoner?

    -> interrogation

  * Back off -> interrogation

* Ask about the warden

  ? player.knows_cell == true

  @halvard: Keep the warden out of this. You got what you wanted.

  > @halvard.mood = hostile

  -> interrogation

  ? player.knows_cell == false

  @halvard: The warden runs a clean operation. End of story.

  -> interrogation

* Try to bribe him -> bribe

* Ask about the escape route

  ? any:
    @halvard.mood == hostile
    @halvard.mood == suspicious

  @halvard: I don't talk to your kind.

  -> interrogation

  ? @halvard.mood == neutral

  @halvard: There's a passage behind the chapel.

  > player.knows_escape = true

  -> interrogation

* I'm done here -> farewell

== bribe

? @coin_purse in player

You slide the coin purse across the table.

  ? @halvard.mood == hostile

  @halvard pushes it back.

  @halvard: Not enough to buy what you're asking.

  -> interrogation

  ? @halvard.mood != hostile

  @halvard pockets it without looking down.

  @halvard: What do you want to know?

  > @halvard.mood = neutral

  > move @coin_purse -> @halvard

  -> interrogation

? @coin_purse not in player

@halvard: Bribe with what? You've got nothing.

-> interrogation

== farewell

? @halvard.mood == nervous

@halvard: Watch yourself out there.

He won't meet your eyes.

? @halvard.mood == hostile

@halvard says nothing. The door slams behind you.

? @halvard.mood == neutral

@halvard: Don't come back.

He says it without conviction.
```

This scene uses one shot choices throughout (each question is asked once). The hub loops via `-> interrogation`. The escape route choice demonstrates OR conditions (`? any:`) where multiple mood states produce the same gated response.

## Design Rationale

The syntax is built on deliberate choices, each with tradeoffs. The decisions below are considered final for v1.

### @ for Entity References

`@` borrows from social media's @mention convention. It's recognisable, unambiguous to a parser, and works in both frontmatter and prose. In frontmatter, `@` declares an entity (`@arina: Barkeep { name: "Arina" }`). In prose, `@` references an existing entity (`@arina: What'll it be?`). The meaning is consistent: `@` always means "this is an entity." The grammatical role (declaration vs. reference) is determined by context, just as variable names work in programming languages. The colon distinguishes speech (`@arina: text`) from stage direction (`@arina does something`).

### ? for Conditions, > for Effects

`?` reads as "is this true?" and `>` reads as "do this." Both live on their own line, making them scannable, diff friendly, lint friendly, and individually addressable by error messages. Multiple `?` lines are AND ed.

### * and + for Choices

Borrowed directly from ink. `*` (one shot) and `+` (sticky) are the two choice types every dialogue system needs. Using the same convention reduces the learning curve for ink writers and avoids inventing new semantics for a solved problem.

### Indentation + Sections (Hybrid Nesting)

Pure indentation breaks at depth. Pure labels fragment the conversation. The hybrid uses indentation for the common case (one to two levels) and sections for structure and loops. This formalises how experienced ink writers already work rather than inventing a new model.

### Heading Hierarchy as Spatial and Temporal Structure

`# Heading` = location. `## Heading` = sequence. `### Heading` = phase. Standard markdown hierarchy, repurposed. Writers already have muscle memory for heading levels. The `(auto)` suffix on a phase heading (e.g., `### Reveal (auto)`) is syntactic sugar that compiles to `auto: true` on the corresponding phase object in JSON, indicating the phase advances without player action.

### The `in` Keyword and `here` Alias

`@key in player` and `@key in here` are sugar over `entity.container == X`. `not in` is the negation. These read naturally in English and unify all spatial checks.

`here` resolves to `player.container` at evaluation time. It is valid in both conditions and effects. In conditions, `? @key in here` checks whether the key is in the player's current location. In effects, `> move @key -> here` drops the key in the player's current location (equivalent to `> move @key -> player.container`).

### Urd Frontmatter with Imports

The structured block between `---` delimiters is called **Urd frontmatter**. It uses YAML like key value syntax but is parsed by the Urd compiler, not a general purpose YAML parser. This distinction is important: Urd frontmatter is its own mini grammar with strict constraints (no anchors, no aliases, no implicit type coercion). The full grammar is specified in the Schema Specification.

In production, frontmatter is a one line import. The split prevents frontmatter growth and keeps writers in prose.

### ~ for Hidden Properties

`~` prefix in type definitions compiles to `visibility: hidden`. Compact and visually distinctive.

### Rules Are Engineer Authored

The rule block syntax exists for completeness and small test worlds. In production, rules live in engineer managed files.

### How Rules Surface to Writers

Writers are told they don't need to know how rules work. But rules affect what writers see, so the surfacing mechanism must be explicit.

**Actions appear and disappear automatically.** When a rule declares that Monty must open a door under certain conditions, the runtime evaluates those conditions and adds or removes the corresponding action from the available actions list. The writer sees the result: a choice appears ("Monty opens door 2") when conditions are met. The writer does not need to declare the action. The rule generates it.

**Rule driven effects are indistinguishable from authored effects.** When a rule fires, it produces the same event types as an authored `>` line: state changes, entity moves, reveals. The runtime and presentation layer treat them identically. A writer reading the event log cannot tell whether an effect came from their script or from a rule.

**The contract between writer and engineer:** Writers describe behaviours they want in prose comments or external briefs. Engineers encode those behaviours as rules. The writer's file references the entities and properties involved (`@guard.mood`, `@door.state`) but never writes the constraint logic. If a writer needs a new behaviour, they request it; they don't learn rule syntax.

**IDE visibility.** The Urd IDE's writer mode will show which rules affect the current scene as a summary panel: "Active rules: monty_reveals (fires when player has chosen a door and Monty hasn't acted yet)." This gives writers awareness without requiring comprehension of the rule syntax.

## Compiler Mapping

How syntax elements compile to the Urd World Schema JSON.

| Syntax | Compiles To |
|--------|-------------|
| `---`...`---` (Urd frontmatter) | world, types, entities blocks. |
| `import: path` | Resolved types and entities merged into compilation scope. |
| `# Location Name` | Entry in locations block. Description from following prose. |
| `[@entity, ...]` | The contains field of the enclosing location. |
| `## Sequence Name` | Entry in sequences block. |
| `### Phase Name` | A phase within the enclosing sequence. |
| `(auto)` | `auto: true` on the phase. |
| `* choice label` | One shot choice entry. `consumed: true` after selection. |
| `+ choice label` | Sticky choice entry. Never consumed. |
| `* label -> @target` | Action with `target: entity_ref`. |
| `* label -> any Type` | Action with `target_type: TypeName`. |
| `? expression` | Entry in conditions list (AND ed). |
| `? any:` + indented conditions | `any:` block in conditions. Any single sub-condition being true validates the block. |
| `> entity.prop = value` | set effect. |
| `> entity.prop + N` | Arithmetic set: current value plus N. |
| `> move @entity -> container` | move effect. |
| `> reveal @entity.prop` | reveal effect. |
| `> destroy @entity` | destroy effect. |
| `@entity: text` | Dialogue content attributed to entity. |
| `@entity text` (no colon) | Narration/stage direction referencing entity. |
| `== name` | Section in the dialogue block. |
| Plain text at the start of a section (before any `@speaker:` line) | The section's `description` field. |
| `-> name` (section) | goto field targeting the named section. |
| `-> end` | No `goto` emitted; runtime exits dialogue mode. |
| `-> target` (exit) | Exit in the enclosing location's exits map. |
| `-> exit:name` | Explicit exit reference. Compiles identically to `-> target` (exit). Used when a section shadows an exit name. |
| `! text` | blocked_message on the enclosing exit or action. |
| `rule name: ... selects ... where` | Entry in rules block with select sub block. |
| `@entity in other` | Condition: `entity.container == other`. |
| `@entity not in other` | Condition: `entity.container != other`. |
| `@entity in here` | Condition: `entity.container == player.container`. |
| `> move @entity -> here` | Effect: `move: entity, to: player.container`. |
| `? <section_name>.exhausted` | Runtime evaluated condition. No generated boolean in JSON. The runtime checks all choices in the named section and returns true if none are currently available (consumed or gated). `<section_name>` must resolve to a declared section identifier in scope. This is the canonical form; always use the concrete section name (e.g., `? topics.exhausted`), not the generic `? section.exhausted`. |
| `// text` | Stripped during compilation. Does not appear in JSON. |

## Decisions Locked for v1

The following decisions are locked now to prevent ambiguity during compiler implementation.

### Import Resolution Rules

> **Status: NORMATIVE.** These rules are the authoritative specification for import behaviour. Compiler and runtime implementations MUST conform to these rules. The architecture document's description of import handling is derived from and consistent with these rules.

- **Imports are explicit and non transitive.** If `tavern.urd.md` imports `world.urd.md`, and `world.urd.md` imports `types.urd.md`, the tavern file does **not** automatically see the types from `types.urd.md`. It must import them directly or import `world.urd.md` which re exports them.
- **Circular imports are a compile error.** If file A imports file B and file B imports file A, the compiler rejects both with a clear diagnostic: *"Circular import detected: tavern.urd.md → harbor.urd.md → tavern.urd.md."* The compiler MUST detect cycles of any length and report the full cycle path.
- **Duplicate entity IDs are a compile error.** If two imported files both declare `@guard`, the compiler rejects the build: *"Duplicate entity ID '@guard' declared in both world.urd.md (line 12) and npcs.urd.md (line 5)."* No silent merging, no last write wins.
- **Duplicate type names are a compile error.** Same rule. Two files cannot define a type with the same name.
- **Import paths are relative to the importing file.** The path in `import: ./world.urd.md` is resolved relative to the directory containing the file with the import declaration. Absolute paths are not supported.
- **Scope rules.** An imported file's types and entities are merged into the importing file's compilation scope. Sections (dialogue) are file scoped and are NOT made available to the importing file. Cross file section access is not supported in v1.

### OR Conditions

- **OR conditions use `? any:` followed by indented conditions.** Multiple `?` lines are AND-ed by default. To express OR logic, use the `? any:` block: any single indented condition returning true validates the entire block. Indented conditions under `? any:` follow the same two-space indent rule as choice content.

```
? any:
  @guard.mood == hostile
  @guard.mood == suspicious
@guard: Get out of here.
```

This compiles to the `any:` construct in the JSON schema. `? any:` blocks can appear anywhere a `?` condition can appear: before choices, inside choice content, or at the start of a section. A `? any:` block counts as a single condition for AND-ing purposes: it can be combined with other `?` lines, and all must be true.

```
? any:
  @guard.mood == hostile
  @guard.mood == suspicious
? @guard.container == player.container
@guard: Get out of here.
// Both conditions must be true:
// (mood is hostile OR suspicious) AND (guard is in the same room)
```

### Comments

- **The comment syntax is `// text`.** A double slash at the start of a line or after a space makes the rest of the line a comment. The compiler strips comments during parsing; they do not appear in compiled JSON. The `//` marker was chosen over `#` because `#` is the heading syntax, and over `--` because it conflicts with dash usage in prose.
- **Inline comments are allowed.** `@arina: What'll it be? // this is the hub prompt`. The comment begins at `//` preceded by whitespace.
- **Comments in Urd frontmatter use `#`.** This is consistent with the YAML like frontmatter grammar. Below the frontmatter, `//` is the comment marker.

### Owner Visibility

- **The token `~` is reserved for hidden visibility.** The token `~~` **is reserved for owner visibility.** The `~~` prefix in a type definition will compile to `visibility: owner`. The syntax is not yet fully specified (the semantics of owner visibility depend on the ownership model, which is a future version feature), but the token is reserved now to prevent future conflicts.

### Stable IDs for Sections and Choices

- **Section IDs** in compiled JSON are derived from the file path and section name: `file_stem + "/" + section_name`. For example, `== topics` in `tavern.urd.md` compiles to ID `tavern/topics`. This makes section IDs stable across recompiles and unique across the world.
- **Choice IDs** are derived from the section ID plus the choice label, slugified: `section_id + "/" + slugify(label)`. For example, "Ask about the harbor" in section `tavern/topics` compiles to `tavern/topics/ask-about-the-harbor`. If two choices in the same section have identical labels after slugification, the compiler emits an error.
- **Entity IDs** are the declared `@name` and must be globally unique across the compiled world. The compiler enforces this.

> **Why stable IDs matter.** The LSP needs to maintain references across recompiles. The testing framework needs stable identifiers for assertions. Save files need to reference sections and choices by ID. If IDs change when a writer adds a line, everything downstream breaks.

## Remaining Open Items

The following items are deferred to future versions:

- **Cross file section jumps.** Can a conversation in the tavern jump to a section in the harbor file? Currently sections are file scoped. Cross file jumps may use a path syntax like `-> harbor.urd.md/dockside_argument` in a future version. **v1 workaround:** use location exits (`-> harbor`) for cross file movement, sequence phase boundaries for narrative continuity across locations, and small duplicated bridge sections if two files need shared dialogue context. These patterns are documented in the Multi File Authoring Pattern section.
- **Cross file state and exhaustion carry over.** When a writer duplicates a bridge section for cross file continuity, exhaustion state from the original section does not automatically transfer to the duplicate. In v1, each file's sections track their own state independently. A future cross file jump mechanism would need to specify whether exhaustion state is shared or file local.
- **NPC initiated topic shifts.** All current examples are player driven. NPC initiated redirection (`@arina: Actually, there's something I need to ask you. -> arina_question`) works syntactically but deserves explicit design.
- **Owner visibility semantics.** The `~~` token is reserved. Full semantics depend on the ownership model.
- **Compiler error quality.** Good error messages are a feature of the syntax. Line level conditions and effects make errors pinpointable; the compiler should produce messages like: `"Line 47: @guard.trust is not a property on type Guard. Did you mean @guard.mood?"`
- **Editor tooling.** Section navigation, graph visualization, rename support, and depth warnings are all critical for production use. Depth warnings in particular must surface the compiler's nesting constraint in real time. The writer should see the warning as they type, not after they save. These are editor features, not syntax decisions, but they determine whether the syntax succeeds in practice.

**This document closes the syntax design phase.** The vocabulary is locked: `@` for entities, `?` / `>` for conditions and effects, `? any:` for OR conditions, `*` / `+` for choices, `==` / `->` for sections and jumps, `#` headings for structure, `~` for hidden, `~~` reserved for owner visibility, `//` for comments, `in` / `not in` for containment. Import merge rules, stable ID derivation, and jump disambiguation are locked for v1. Remaining items are implementation decisions, not language design questions.

*End of Specification*
