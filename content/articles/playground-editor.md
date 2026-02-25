---
title: The Editor Learns the Language
slug: playground-editor
description: The playground code editor is now a FactSet-powered semantic tool — 25 reference kinds with hover tooltips drawn from the compiler's queryable semantic graph, mirroring the Rust-side LSP architecture. Inline diagnostics, autocomplete, and go-to-definition complete the IDE experience.
date: "2026-02-25"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Documenting the code editor improvements across three rounds of work: IDE foundations, contextual hints, and full syntax coverage.
> Single canonical copy. February 2026.

## From text box to language-aware editor

When the [playground](/playground) launched, the editor was a syntax highlighter. It coloured tokens — headings, entity references, conditions, effects — but it did not understand them. Hovering a keyword did nothing. Typing `@` offered no suggestions. Clicking an entity reference had nowhere to go. The compiler ran in the background, but the editor was passive.

Three rounds of work changed that. The editor now understands every construct in Schema Markdown — not as syntax tokens, but as semantic references with definitions, documentation, and cross-references.

## What the editor does now

**Inline diagnostics.** Compiler errors and warnings appear directly in the editor as underlined spans with severity markers, not just in a separate panel. Hover an underline to see the full diagnostic message with its error code. The same URD100–URD599 codes documented in the [compiler briefs](/documents/urd-compiler-architecture-brief) appear inline, tied to the exact span that caused them.

**Autocomplete.** Type `@` and see a list of every entity in the world. Type a property name after a dot and see valid properties for that entity's type. Inside frontmatter, type a top-level key and see `types:`, `entities:`, `world:` offered with descriptions. Completion triggers are context-aware — they know whether you are in frontmatter, inside a type definition, on an effect line, or in narrative content.

**Go-to-definition.** Ctrl+click (or Cmd+click) an entity reference to jump to its declaration in the frontmatter. Click a section jump (`-> section-name`) to jump to the target section heading. Click an exit destination to jump to the target location. The same mechanism works for type references, location headings, and sequence headings.

**Hover tooltips.** This is where the bulk of the work went. The editor now resolves 25 distinct reference kinds — every meaningful token in a valid Urd document produces a tooltip when hovered.

## Twenty-five reference kinds

The cursor resolver — the component that identifies what the cursor is pointing at — started with 4 reference kinds (entity, section jump, location heading, keyword) and grew to 25 across three briefs. Each kind maps to a tooltip builder that assembles contextual information from the compiled world, the definition index, and the fact set.

**Entities and properties.** Hovering `@elder_maren` shows the entity's type, current location, and containment. Hovering `@elder_maren.mood` shows the property's type, default value, valid enum values, and read/write counts from the fact set. On a dialogue line (`@elder_maren: text`), the tooltip annotates "Speaking — dialogue attribution." On a narrative action line (`@elder_maren sighs`), it annotates "Narrative action."

**Type system.** Hovering a type name on its definition line or on an entity declaration shows the type's traits, properties, and instances. Hovering a type constructor like `int(0, 100)` shows the constructor's documentation with the actual range and default value parsed from the line. Hovering a trait name inside `[interactable, mobile]` explains what the trait enables. Hovering the `~` visibility prefix explains hidden properties and how to reveal them. Hovering `.container` on any entity shows that it is an implicit property modified via `move`, not direct assignment.

**Conditions and effects.** Hovering `in`, `not in`, `player`, or `here` on a condition line shows what each keyword tests. Hovering `any:` explains OR combinator semantics. Hovering `move`, `destroy`, or `reveal` on an effect line shows the command's syntax and valid targets. Hovering a value literal after an operator shows whether it is a valid enum value for the property, with a warning if it is not.

**Rules.** Hovering `rule` shows that it declares an NPC behavioural rule. Hovering the rule name shows metadata from the definition index. Hovering `actor:`, `action`, `selects`, `from`, `where`, and `target` inside a rule block explains each keyword's role in the select-filter-act pipeline.

**Sequences and phases.** Hovering a `##` heading shows it defines a sequence (multi-phase quest arc). Hovering a `###` heading shows the phase name and advance mode — manual or auto — with detection of the `(auto)` modifier.

**World configuration.** Hovering sub-keys under `world:` in the frontmatter (`name`, `version`, `start`, `entry`, `seed`, `description`, `author`) shows documentation for each. For `start` and `entry`, the tooltip resolves the reference — showing the location or sequence it points to.

**Navigation constructs.** Hovering `-> section-name` shows where the jump leads with a content preview. Hovering `-> exit:direction` explains exit-jump semantics and resolves the destination. Hovering `-> END` or `-> RETURN` (case-insensitive) shows the built-in jump documentation.

**Structural markers.** Hovering `+` (choice), `*` (one-shot choice), `?` (condition), `>` (effect), `->` (jump), and `---` (frontmatter delimiter) each shows a brief explanation of the construct's purpose. Hovering `!` on a blocked message line explains that it marks content shown when a path is sealed or a gate is locked. Hovering `//` explains comments.

**Frontmatter keys.** Hovering `types:`, `entities:`, or `world:` at the top level of the frontmatter explains what each section contains and how it relates to the compiled output.

## Powered by the FactSet

Most editor tooltips in other tools are static — they show what a symbol's type declaration says. The playground tooltips are different because they draw from the compiler's [FactSet](/articles/queryable-semantic-graph), the queryable semantic graph that the compiler extracts during compilation.

The FactSet is a structured set of six tuple types — containment facts, property read/write facts, transition facts, rule binding facts, mutation facts, and constraint facts — that represent everything the compiler knows about the world's semantics. When you hover `@elder_maren.mood`, the tooltip does not just say "mood is an enum" — it queries the FactSet to report how many conditions read that property and how many effects write to it. When you hover an entity, the tooltip shows its real containment chain from the FactSet, not a static declaration.

This means the tooltips are as accurate as the compiler itself. If the compiler's analysis says a property is read in three conditions and written by two effects, that is exactly what the tooltip shows. If an entity is contained inside another entity which is inside a location, the tooltip shows the full chain. The information updates on every recompilation — change the source and the tooltips reflect the new state.

The same FactSet feeds the [LSP server](/articles/semantic-gate-closed) that powers VS Code and other editors via the Language Server Protocol. The playground's tooltip builder and the LSP's hover handler consume the same data structures. The playground is not a simplified version of the LSP — it is a parallel client of the same semantic infrastructure.

## How it works

The system is three components working together, mirroring the architecture of the Rust-side LSP.

**The cursor resolver** (`cursor-resolver.ts`) takes a cursor position and the document text, and returns a typed `Reference` describing what the cursor points at. It is a deliberate architectural mirror of the Rust-side `cursor.rs` module in the LSP — the same line-oriented approach, the same cascade of pattern matchers, the same context-tracking strategy. It reads the current line, determines context (frontmatter, type block, world block, rule block, effect line, condition line), and runs matchers ordered by specificity: entity property access before bare entity reference, exit-jump before section-jump, condition combinator before condition keyword.

Context tracking is a forward scan from the document start to the cursor line, counting `---` delimiters to detect frontmatter, tracking `types:` and `world:` top-level keys for block detection, and backward-scanning for `rule name:` lines to detect rule blocks. This parallel architecture means that reference kinds added to the playground can be backported to the LSP (and vice versa) with minimal translation — the detection logic is structurally identical, just in different languages.

**The tooltip builder** (`hover-tooltip.ts`) takes a `Reference` and the playground state — which includes the compiled world JSON, the definition index, and the FactSet — and returns an HTML string. Each reference kind has a dedicated builder function. Entity tooltips pull containment from the world JSON and read/write counts from the FactSet. Property tooltips pull type metadata from the definition index and usage statistics from the FactSet. Type name tooltips enumerate instances by scanning the entity declarations. Value literal tooltips validate against enum definitions extracted from type constructors. Keyword tooltips use static documentation maps for language fundamentals where no compiled data is needed.

**The CodeMirror integration** connects the cursor resolver to CodeMirror's `hoverTooltip` facet. On hover, it calls the resolver, passes the result to the tooltip builder, and returns a positioned tooltip widget. The same resolver feeds go-to-definition (Ctrl+click navigates to where the reference is defined) and could feed future features like find-all-references — again mirroring the LSP, where the same cursor module feeds hover, go-to-definition, and references.

## The blocked message marker

One gap discovered during this work: the `!` marker for blocked messages had no syntax highlighting at all. In Urd grammar, `!` at line start marks content shown when a gate is locked or a path is blocked — a distinct construct from choices, conditions, or dialogue. It now highlights as a keyword and shows a tooltip explaining its purpose.

## What this means for authors

An author writing Schema Markdown in the playground can now hover any token they do not recognise and get an explanation. They can hover any entity reference and see its type and location. They can hover a property and see its valid values. They can hover a rule keyword and understand how the select-filter-act pipeline works.

This is not documentation — it is contextual intelligence. The tooltips change based on what the compiler actually found. An entity tooltip shows the entity's real containment, not a generic description. A property tooltip shows real read/write counts from the fact set, not theoretical ones. A value literal tooltip validates against the real enum definition, not a static list.

The difference matters because Schema Markdown is a declarative language for people who may not think of themselves as programmers. Writers and designers should not need to memorise syntax — they should be able to point at something and understand it. The editor now supports that.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
