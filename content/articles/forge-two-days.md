---
title: "Two Days"
slug: forge-two-days
description: A declarative schema system, five-phase Rust compiler, semantic analysis layer, WASM playground, documentation website, and 88-view desktop IDE, built in eighteen days. Almost entirely by AI. At what point does the speed itself become the finding?
date: "2026-02-28"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> On the velocity of AI-assisted software development and what changed in 2026.
> Single canonical copy. February 2026.

## Eighteen days

On February 10th, 2026, the repository was empty.

On February 28th, it contains:

- A declarative schema language for interactive worlds, with a normative specification, a formal PEG grammar, and a JSON Schema for compiled output.
- A six-phase Rust compiler (parse, import, link, validate, analyse, emit) with 634 tests at 100% pass rate, 74 diagnostic codes, sub-millisecond compilation, and a 67 KB WASM binary.
- A queryable semantic intermediate representation (the FactSet) with six tuple types covering every resolved relationship in a compiled world. A semantic diff engine. A property dependency index. A definition index for IDE integration.
- Five novel static analysis diagnostics derived purely from FactSet queries, not AST traversal.
- An LSP foundation with hover, go-to-definition, and autocomplete.
- An MCP server exposing eight read-only query tools for AI agent consumption.
- An interactive playground running the full compiler as WebAssembly in the browser, with a CodeMirror 6 editor, inline diagnostics, 25 semantic reference kinds with hover tooltips, and real-time JSON output.
- A documentation website at urd.dev with a live test dashboard, interactive document explorer, project timeline, and the playground itself.
- 27 design documents, 24 journal articles, and 30 chronological updates.
- An 88-view desktop IDE with a Blender-style BSP layout engine, two themes, five workspace templates, 35 commands with configurable keybindings, and a spec-complete interactive fiction runtime.

One person. Eighteen days. Almost no code written by hand.

## The IDE

The [architecture document](/articles/urd-forge) was published on February 26th. It described eight implementation phases, 85+ view types, a BSP layout engine, a projection-based data architecture, a command registry, workspace management, and a message bus. Four thousand words of specification.

Two days later, all of it exists.

![Urd Forge, Parchment theme, full information density](/images/documents/forge-parchment-overview.png)

Thirteen spreadsheets. Twenty-three graph views in two rendering modes. Eight inspectors. Ten analysis panels. Eight search tools. Seven runtime debugging views. A code editor with syntax highlighting and tabbed file management. A file browser with keyboard navigation. A spec-complete interactive fiction runtime implementing every feature in URD v1: dialogue trees with sticky and one-shot choices, four effect types, container model, actions block, sequence phases, NPC behavioural rules, section exhaustion.

![Gloaming theme, code editor with file browser](/images/documents/forge-dark-code-editor.png)

The layout is a recursive binary space partition. Every leaf is a zone. Every zone can display any of the 88 registered views. Split, join, swap, resize. Five workspace templates. Layout state persists across sessions.

![Gloaming theme, maximum density stress test](/images/documents/forge-gloaming-full-density.png)

Thirty-five commands in the registry, each with a keybinding and a menu location. All configurable through a searchable keybinding editor:

![Keybindings panel](/images/documents/forge-keybindings.png)

38,786 lines of TypeScript and Svelte across 234 source files. Two calendar days.

## The compiler

The compiler took one day.

On February 17th, six engineering briefs were published: architecture plus five phase briefs covering parse, import, link, validate, and emit. Interface contracts, diagnostic catalogues, 150+ acceptance test cases. Every brief written before a line of Rust was committed.

On February 18th, the compiler existed. Five phases. 302 tests. 100% pass rate. The specifications were amended zero times during implementation.

Over the following week, ten patch releases brought it to v0.1.14: 634 tests, 74 diagnostic codes, six phases (an analysis phase was added for FactSet extraction), a semantic diff engine, a definition index, and a property dependency index. 8,189 lines of Rust across 28 source files.

## The semantic layer

The FactSet was built on February 22nd and extended through the 24th. Six tuple types: PropertyRead, PropertyWrite, ExitEdge, JumpEdge, ChoiceFact, RuleFact. A PropertyDependencyIndex for direct lookup. A DefinitionIndex mapping every declaration to its source span.

On top of that: an LSP foundation. An MCP server with eight query tools for AI agents. Five diagnostic codes that exist only as FactSet queries. A semantic diff engine that compares compiled worlds structurally.

The entire semantic gate closed on February 24th. Six briefs across three tiers. Compiler 0.1.13. 626 tests.

## The playground

The playground launched on February 19th, one day after the compiler. Full five-phase Rust compiler running as WebAssembly. CodeMirror 6 editor with a custom Schema Markdown language mode. Type on the left, see compiled JSON on the right. No server. Nothing leaves the page.

Six days later, the editor understood 25 semantic reference kinds. Hover any entity reference and see its type, containment chain, and read/write counts from the FactSet. Inline diagnostics, autocomplete, go-to-definition. The TypeScript cursor resolver mirrors the Rust-side LSP module structurally.

## The website

urd.dev launched on February 11th, day two. Astro 5 static site with Svelte 5 interactive islands. Six components hydrated client-side: document explorer, compiler test dashboard, peer review viewer, project log, project timeline, and the playground. A 600-line design brief driving every visual decision through CSS custom properties. Two themes. Self-hosted fonts. Cloudflare Pages deployment via GitHub Actions.

## The specification suite

Before any code was written:

- A schema language specification defining the source format.
- A JSON Schema for the compiled output. Draft 2020-12. Nine reusable sub-schemas. 32 test fixtures.
- A PEG grammar for the parse rules. Rust/pest. 12 test fixtures.
- An architecture brief plus five phase briefs with interface contracts and acceptance tests.
- A governance document defining what V1 includes, what it permanently excludes, and a five-question boundary test for future proposals.
- A completion gate defining 32 acceptance criteria across five categories.

All specifications were written before implementation began. None were amended during implementation.

## How this happened

I wrote specifications. AI wrote code.

The longer version: I defined what correct means and AI produced things I could check against that definition. The specifications were not vague directives. They were acceptance test suites. The compiler briefs contained 150+ specific test cases. The IDE architecture specified every interface and data flow. The FactSet briefs listed exact tuple shapes and query methods. Every deliverable had a machine-verifiable definition of done.

The AI did not hallucinate an IDE. It translated a 4,000-word specification into 38,786 lines of working code, guided by continuous testing against fixtures and type checking against interfaces. The cycle of specification, implementation, failure report, and correction ran in minutes. Not hours, not days. Minutes.

## Phase shift

Here is what I think actually happened.

Since the release of Opus 4.6 and Sonnet 4.6, the difference to AI coding in late 2025 is not incremental. It is a phase shift.

A six-phase Rust compiler with 634 tests would be a respectable year of solo work. An 88-view desktop IDE with BSP layout and projection architecture would be another year. A WASM playground, a documentation website, a semantic analysis layer, an LSP, an MCP server, 27 design documents. This is a multi-year, multi-person project compressed into eighteen days.

The compression did not come from cutting corners. The compiler has 634 tests. The architecture separates framework from application. The projection layer memoises correctly. The code is typed, tested, and structured. The quality is not proportionally lower. It is just faster.

What changed is not that building software became trivial. What changed is that the translation from specification to working code, which historically consumed the vast majority of calendar time, has become nearly free for well-specified problems. The intellectual work (choosing the right abstractions, defining acceptance criteria, designing data models, making trade-off decisions) still takes the same effort. The mechanical work (writing the Svelte components, the TypeScript services, the CSS, the Rust modules, the import wiring) has been absorbed.

2026 is going to be a wild year. If one person with Opus 4.6 can produce this output in eighteen days, the ceiling on what small teams can build has moved by at least an order of magnitude. Not because AI writes perfect code. It does not. But because the feedback loop of specify, generate, test, correct has collapsed from days to hours, and that loop is where most software development time actually goes.

## What happens next

This project will likely be parked as a historical record. The repository, the specifications, the compiler, the IDE, the website, the full brief archive. Urd.dev captured a specific moment: what happens when you point the current generation of AI at a well-specified problem domain and let it run.

The answer, it turns out, is: everything gets built.

The more interesting experiment has not been run yet. Every brief exists. Every specification is written. The acceptance test suites are defined. The JSON Schema validates. The PEG grammar parses. The compiler output format is locked. The FactSet tuple types are documented. The IDE architecture is specified down to the interface level. The entire project is, by design, a machine-readable description of what "correct" looks like.

Which means someone could take these briefs, point their AI setup at them, and attempt to one-shot the entire project from scratch. Not iteratively over eighteen days, but in a single pass. The specifications are detailed enough. The acceptance criteria are precise enough. The fixtures exist. The question is whether current models can take a brief like "build a six-phase Rust compiler for this schema language, here are the interface contracts and 150 test cases" and produce a working implementation without the human feedback loop.

That would be a genuinely interesting test of where the frontier actually is. Not "can AI write code" (yes, obviously) but "can AI build a complete, tested, specified system from a cold start." The briefs are public. The challenge is open.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
