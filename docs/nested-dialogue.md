---
title: "Nested Dialogue Architecture"
slug: "nested-dialogue"
description: "Design exploration for nested dialogue in Schema Markdown. Analyses five structural patterns (branch, hub-and-spoke, conditional, loop, fallthrough) and how ink, Yarn Spinner, and Ren'Py solve them."
category: "contract"
format: "Design Document"
date: "2026-02"
status: "v1.0 complete"
order: 2
tags:
  - dialogue
  - nesting
  - syntax
  - design-exploration
details:
  - "Five structural patterns every dialogue system must handle"
  - "Comparison with ink, Yarn Spinner, and Ren'Py approaches"
  - "Indentation-based nesting with explicit hub/spoke via arrows"
  - "Stress-tested against a complex interrogation scene"
---

> **Document status: INFORMATIVE — DESIGN EXPLORATION**
> Documents the design rationale and alternatives considered for the nested dialogue model. The decisions reached here are codified in the Schema Markdown Syntax Specification; this document provides the reasoning behind them.
> Single canonical copy. February 2026 draft.

# URD

## Nested Dialogue Design Exploration

*Solving the highest risk syntax problem in Schema Markdown*

urd.world

February 2026 | Design Exploration

## The Problem

The current Schema Markdown syntax handles single level choices cleanly. But real game dialogue is not a flat menu. Players ask about a topic, the NPC responds, new sub choices appear within that topic, and the player may loop back to earlier topics or drill deeper. Without clean nesting, writers will revert to ink the moment a conversation branches more than one level deep.

This is the highest risk syntax problem because it sits at the intersection of three competing demands:

- **Readability.** Nested structures must remain scannable. If a writer can't visually track where they are in the conversation tree, the syntax is unusable.
- **Expressiveness.** Real dialogue needs branches within branches, hub and spoke topic menus, conditional sub trees, loops back to earlier points, and shared content after branches converge.
- **Compilability.** Whatever syntax we choose must compile deterministically to the schema's JSON structure. Ambiguous nesting or implicit scoping rules create bugs that are invisible in the source.

### The Five Patterns

Every dialogue system in games reduces to combinations of five structural patterns. A syntax that handles all five handles everything.

| Pattern | Description | Example |
|---------|-------------|---------|
| Branch | A choice leads to NPC response with sub choices. | Ask about X → NPC responds → follow up questions |
| Hub and spoke | A menu of topics. After each, return to menu. | "What do you want to know?" → topic → back to menu |
| Conditional branch | Different sub trees based on state. | Trust > 50 → reveals secret; otherwise deflects |
| Loop | A choice returns to an earlier point. | "Ask about something else" → back to topic menu |
| Fallthrough | After any branch, shared content continues. | Regardless of choice, NPC then says farewell |

## How ink Solves This

Before proposing anything, it's worth understanding ink's solution, because it's the benchmark writers compare against.

Ink uses two mechanisms: indentation for local branching, and named sections (knots and stitches) for structural jumps.

```
=== tavern ===

"What do you want to know?" she asks.

+ [The harbor]

  "Quiet today. Too quiet."

  ++ [Why quiet?]

    "Navy pulled out last week."

    +++ [Press for details]

      { trust > 30:

        "They're hunting something out past the reef."

        ~ trust += 10

      - else:

        "That's all I know."

      }

    +++ [Change subject]

      -> tavern

  ++ [Interesting]

    -> tavern

+ [The missing ship]

  { trust > 50:

    "The Selene didn't sink. She was taken."

  - else:

    "Don't know what you're talking about."

  }

  -> tavern

+ [Leave]

  -> harbor
```

### What Works

- **Local nesting is visually clear.** The `+` / `++` / `+++` indentation makes it easy to see two or three levels of branching at a glance.
- **Named sections handle loops.** `-> tavern` jumps back to the topic menu. Simple and powerful.
- **Inline conditions.** `{ trust > 30: ... }` keeps conditions close to the content they gate.

### Where It Breaks Down

- **Deep nesting pushes content off screen.** At three levels of indentation with `+++` markers, the actual dialogue text starts 12+ characters in. Four levels is effectively unusable on a normal editor width.
- **No typed entities.** `trust` is a global variable. There's no way to know it belongs to the barkeep, or that it ranges from 0 to 100, or that it's hidden from the player.
- **Conditions and content are interleaved.** The `{ trust > 30: ... - else: ... }` block mixes control flow with dialogue in a way that gets hard to follow in complex scenes.
- **No semantic distinction between speakers.** All quoted text is attributed to whoever the writer intends, but the format doesn't know who that is.

## Approach A: Pure Indentation

The most ink like approach. Nesting is expressed by indenting content under a choice. Sub choices indent further.

### The Tavern Scene

```
@arina: What do you want to know?

* Ask about the harbor

  @arina: Quiet today. Too quiet.

  * Why is it quiet?

    @arina: Navy pulled out last week.

    * Press for details

      ? @arina.trust > 30

      @arina: They're hunting something out past the reef.

      > @arina.trust + 10

      ? @arina.trust <= 30

      @arina: That's all I know.

    * Change the subject

      -> topics

  * Interesting

    -> topics

* Ask about the missing ship

  ? @arina.trust > 50

  @arina: The Selene didn't sink. She was taken.

  ? @arina.trust <= 50

  @arina: Don't know what you're talking about.

  -> topics

* Leave -> harbor
```

### Assessment

**✓ Familiar.** Writers coming from ink will recognise the pattern immediately. The learning curve is near zero for local branching.

**✓ Visual hierarchy.** You can scan the left edge and see the conversation tree structure.

**✗ Deep nesting problem persists.** The third level choice (Press for details) is already indented 6 spaces, and its conditional content is at 6+ spaces. A fourth level would push past comfortable reading width.

**✗ Conditions add vertical weight.** The `?` lines on separate rows mean each conditional branch takes at least two lines (condition + content) where ink's inline `{ condition: }` takes one. At depth, this compounds.

**✗ Ambiguous scoping.** Does the `-> topics` after "Interesting" mean "jump from this sub choice" or "jump from the parent choice?" The indentation answers this, but it's easy to get wrong, especially after editing.

## Approach B: Labeled Sections

Conversation structure is flat. Each topic or branch point is a named section (using `==` markers). Choices jump between sections with `->`. No indentation based nesting at all.

### The Tavern Scene

```
== topics

@arina: What do you want to know?

* Ask about the harbor -> harbor

* Ask about the missing ship -> ship

* Leave -> harbor_location

== harbor

@arina: Quiet today. Too quiet.

* Why is it quiet? -> harbor_why

* Interesting -> topics

== harbor_why

@arina: Navy pulled out last week.

* Press for details -> harbor_details

* Change the subject -> topics

== harbor_details

? @arina.trust > 30

@arina: They're hunting something out past the reef.

> @arina.trust + 10

? @arina.trust <= 30

@arina: That's all I know.

-> topics

== ship

? @arina.trust > 50

@arina: The Selene didn't sink. She was taken.

? @arina.trust <= 50

@arina: Don't know what you're talking about.

-> topics
```

### Assessment

**✓ Zero depth limit.** Every section is at the root indentation level. A conversation can be arbitrarily deep without pushing content off screen.

**✓ Each section is self contained.** A writer can read and edit any section in isolation. There's no need to trace indentation to understand context.

**✓ Jumps are explicit.** Every `->` is a clear, named destination. No ambiguity about where control goes.

**✗ Conversation flow is fragmented.** The dialogue reads like a state machine, not a conversation. To follow the player's path through "harbor → why → details," you jump between three sections. The narrative momentum is lost.

**✗ Naming overhead.** Every branch point needs a unique section name. For a complex conversation with 20+ branches, the writer spends significant effort inventing and managing names.

**✗ Feels like programming.** This is essentially a finite state machine. Engineers will be comfortable. Writers will see it as "the thing ink was supposed to save me from."

## Approach C: Hybrid (Recommended)

**Core idea:** Use indentation for shallow local branching (one to two levels). Use labeled sections for deeper structure and loops. The writer stays in indentation mode for the common case and only breaks into sections when the conversation genuinely needs it.

This is how experienced ink writers actually work. They use indentation for immediate sub choices and knots/stitches for structural navigation. The hybrid approach acknowledges that both mechanisms exist because both are needed, and gives each a clear scope.

### The Tavern Scene

```
== topics

@arina: What do you want to know?

* Ask about the harbor

  @arina: Quiet today. Too quiet.

  * Why is it quiet?

    @arina: Navy pulled out last week.

    -> harbor_details

  * Interesting -> topics

* Ask about the missing ship

  ? @arina.trust > 50

  @arina: The Selene didn't sink. She was taken.

  ? @arina.trust <= 50

  @arina: Don't know what you're talking about.

  -> topics

* Leave -> harbor

== harbor_details

? @arina.trust > 30

@arina: They're hunting something out past the reef.

> @arina.trust + 10

-> topics

? @arina.trust <= 30

@arina: That's all I know.

-> topics
```

### How the Hybrid Works

**Shallow branches stay inline.** The first two levels of the harbor conversation ("Ask about the harbor" → "Why is it quiet?") are indented normally. This is the comfortable range where indentation works.

**The third level breaks out.** Instead of indenting further for "Press for details," the conversation jumps to `== harbor_details`. The deep content lives at root indentation in its own section.

**The hub uses a label.** `== topics` names the hub. `-> topics` returns to it. This is explicit, unambiguous, and handles loops cleanly.

**Flat content stays flat.** The "missing ship" branch has no sub choices, just a conditional response. It stays inline under its choice, no section needed.

### Assessment

**✓ The common case is clean.** Most dialogue branches are one to two levels deep. These stay inline with indentation, which is the most readable form.

**✓ Depth is opt in.** Sections are used only when the conversation genuinely needs them. A simple tavern chat might have zero sections. A complex Disco Elysium style interrogation might have ten.

**✓ Hub and spoke is first class.** The `== topics` pattern directly supports the most common dialogue structure in games.

**✓ Loops are explicit.** `-> topics` makes the navigation graph visible. No implicit fallthrough surprises.

**✓ Fallthrough works naturally.** Content after a section's last choice falls through to the next section or ends. Shared farewell text can go at the end of a section.

**~ Learning curve is modest.** Writers need to understand two things: indentation for local choices, `==` labels for structural jumps. This is the same mental model ink uses, just with different markers.

**~ Section names are still overhead.** But only for deep or looping structures, not for every branch point.

## Stress Test: A Complex Conversation

The tavern scene is deliberately simple. Here's a more demanding test: a multi topic interrogation with conditional branches, state dependent reveals, a persuasion attempt, and a farewell that varies based on how the conversation went.

This is roughly the complexity of a significant conversation in a game like Firewatch or Citizen Sleeper.

### The Interrogation (Hybrid Approach)

```
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

### What This Exercises

- **Hub and spoke with state mutation.** Each topic modifies Halvard's mood, which gates what's available in subsequent topics. The player's path through the conversation changes the conversation.
- **Conditional branches within choices.** "Press him" has two outcomes based on Halvard's current mood. Both are inline under the choice, gated with `?` conditions.
- **A separate section for a complex branch.** The bribe attempt has enough conditional logic (do you have money? what's his mood?) that it breaks out into `== bribe` rather than nesting further under the choice.
- **Containment checks in dialogue.** `? @coin_purse in player` uses the containment model naturally. `> move @coin_purse -> @halvard` transfers the coin purse.
- **State dependent farewell.** The `== farewell` section reads the accumulated state to produce different exits. This is fallthrough with variation.
- **The hub label does the looping.** `-> interrogation` returns to the topic menu after every branch, just like ink's `-> knot_name`.

## Proposed Nesting Rules

The following rules govern how indentation and sections interact in the hybrid model:

### Indentation

- **Two space indent per level.** Consistent with the existing syntax draft. Tabs are not permitted.
- **Content under a `*` choice is indented one level.** This includes dialogue, conditions, effects, and sub choices.
- **Maximum depth: two levels of `*` nesting recommended.** The compiler warns at three levels of nesting with a suggestion to break into a labeled section, and rejects the file at four levels. This is a maintainability constraint, not a style preference.
- **Indentation determines scope.** A line belongs to the most recent `*` choice at its indentation level or above. Blank lines within an indented block do not break the scope.

### Labeled Sections

- `== name` **declares a section.** Names follow entity ID rules: lowercase, digits, underscores.
- **Sections are file scoped.** A section name must be unique within the file, but different files can reuse the same section names.
- `-> name` **jumps to a section.** This can appear at any indentation level. It ends the current branch and transfers control to the named section.
- **A section without a `->` jump at the end falls through to the next section.** This is intentional for farewell/epilogue patterns. The linter warns if a section falls through without an explicit `->` or end marker.
- `-> end` **ends the conversation.** Control returns to the location's action list (freeform mode) or advances the sequence phase.

### How They Interact

Indentation and sections are complementary, not alternatives:

- Indentation handles local branching within a section.
- Sections handle conversation level structure: topic hubs, deep branches, and loops.
- `->` can appear inside indented content (to jump out of a deep branch) or at section level (to loop or end).

### The `not in` Keyword

The stress test introduces `not in` as the negation of `in`: `? @coin_purse not in player` compiles to `entity.container != player`. This reads naturally and completes the containment vocabulary without adding a new symbol.

## Approach Comparison

| Criterion | A: Pure Indent | B: Labels Only | C: Hybrid |
|-----------|---------------|----------------|-----------|
| Simple 2 level branch | ✓ Clean | ~ Fragmented | ✓ Clean |
| Deep branching (4+ levels) | ✗ Unreadable | ✓ Clean | ✓ Clean (breaks out) |
| Hub and spoke (topic menu) | ~ Implicit | ✓ Explicit | ✓ Explicit |
| Loops (return to earlier) | ~ Needs labels anyway | ✓ Built in | ✓ Built in |
| Fallthrough (shared ending) | ✓ Natural | ✓ Natural | ✓ Natural |
| Learning curve for ink writers | ✓ Minimal | ✗ Different model | ~ Small addition |
| Readability at scale | ✗ Degrades | ~ Flat but fragmented | ✓ Maintains |
| Naming burden | ✓ None | ✗ Every branch | ~ Only deep branches |
| Compiler complexity | ✓ Simple | ✓ Simple | ~ Moderate |
| Writer perception | ✓ Familiar | ✗ Programming feel | ✓ Familiar + structured |

**Recommendation:** Approach C. The hybrid model handles the common case (shallow branching) with familiar indentation and provides an explicit escape hatch (labeled sections) for deep structure and loops. It matches how experienced ink writers already think about conversation architecture.

## What This Adds to the Schema

The nested dialogue syntax requires one addition to the Urd World Schema:

### The `dialogue` Block

Labeled sections (`== name`) compile to a dialogue block in the schema. Dialogue is a core part of the v1 schema. The compiler emits the dialogue block alongside all other schema blocks in the compiled output.

> **Note:** The hybrid approach recommended in this exploration was adopted into the Schema Markdown Syntax Specification. The syntax spec is the normative reference for dialogue syntax; the Architecture and Technical Design document is the normative reference for the compiled dialogue block. This document is preserved as a record of design rationale. **If the example below conflicts with the Architecture document, the Architecture document wins for compiled JSON shape.**

> **Historical note on the example below.** This example was written during the design exploration phase and uses a nested structure (`dialogue: interrogation: sections: topics:`) that predates the final compiled format. The normative compiled JSON uses a **flat map keyed by `file_stem/section_name`**, as specified in the Architecture document. The canonical compiled form for the interrogation scene below would look like:

```json
{
  "dialogue": {
    "interrogation/topics": {
      "prompt": { "speaker": "halvard", "text": "You've got questions. Make them quick." },
      "choices": [
        {
          "id": "interrogation/topics/ask-about-the-prisoner",
          "label": "Ask about the prisoner",
          "sticky": false,
          "response": { "speaker": "halvard", "text": "What prisoner?" },
          "choices": [
            {
              "id": "interrogation/topics/press-him",
              "label": "Press him",
              "sticky": false,
              "conditions": ["halvard.mood == neutral"],
              "response": { "..." : "..." },
              "goto": "interrogation/topics"
            },
            {
              "id": "interrogation/topics/back-off",
              "label": "Back off",
              "sticky": false,
              "goto": "interrogation/topics"
            }
          ]
        },
        {
          "id": "interrogation/topics/ask-about-the-warden",
          "label": "Ask about the warden",
          "sticky": true
        }
      ],
      "on_exhausted": { "text": "..." }
    },
    "interrogation/bribe": {
      "conditions": ["coin_purse.container == player"],
      "choices": [ "..." ]
    },
    "interrogation/farewell": {
      "choices": [ "..." ]
    }
  }
}
```

The key structural element is the nested choices within sections model, where each section can contain inline choice trees (handling shallow branching) and jumps to other sections (handling deep structure and loops). This mirrors the hybrid syntax directly.

## Resolved Questions

The following questions were open during the design exploration. All have since been resolved in the Schema Markdown Syntax Specification and the Architecture and Technical Design document.

- **Sticky choices vs. one shot.** Schema Markdown uses `*` for one shot (consumed after selection) and `+` for sticky (always available). The syntax spec defines normative semantics: one shot choices are consumed and never re presented; sticky choices are never consumed and may be selected repeatedly. Visited state is tracked separately from consumed state.
- **Gathering / convergence.** The hybrid approach handles convergence through section jumps (`-> section_name`) and fallthrough after choice blocks. No explicit gather marker was added. Writers reconverge by jumping back to a hub section, which is the natural pattern in hub and spoke conversations.
- **Section scope vs. file scope.** Sections are file scoped in v1. Cross file movement uses location exits, not section jumps. Cross file section jumps are a future capability listed in the syntax spec's Remaining Open Items.
- **NPC initiated topic shifts.** The syntax supports NPC initiated redirection naturally (`@arina: Actually, there's something I need to ask you.` followed by `-> arina_question`). This works syntactically but is listed in the syntax spec's Remaining Open Items for explicit design.
- **Exhaustion.** When all one shot choices in a section are consumed and all remaining choices are gated, the section is exhausted. Content falls through to the first text block after the choice block. The runtime must never present an empty choice menu. Exhaustion state is available as a condition using the canonical form `? <section_name>.exhausted` (e.g., `? topics.exhausted`), where `<section_name>` resolves to a declared section identifier in scope. The compiled JSON represents the fallthrough content in an `on_exhausted` field on the section object; it does not contain an exhausted boolean. Whether a section is exhausted is always a runtime-evaluated predicate: the runtime checks all choices in the named section and returns true if none are available.

*The hybrid approach resolved the highest risk syntax problem while preserving the readability that makes Schema Markdown viable.* **The syntax spec is now the authoritative source for the dialogue vocabulary, nesting rules, and normative semantics.**

*End of Exploration*

