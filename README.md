![Urd](.github/assets/banner.png)

# URD & WYRD

**A declarative schema system for interactive worlds**

Define in Urd. Test in Wyrd. Ship anywhere.

![CI](https://github.com/urdwyrd/urd/actions/workflows/ci.yml/badge.svg)
![Deploy](https://github.com/urdwyrd/urd/actions/workflows/deploy.yml/badge.svg)

In Norse mythology, Urð is the Norn of fate — the keeper of what is and what was. The schema that defines the world. Wyrd, its Old English cognate, is destiny unfolding — what happens. The runtime that executes it.

---

Urd is an open, structured world definition format — a declarative schema that describes interactive worlds as typed, validated, engine-agnostic data. Writers author in **Schema Markdown**, a prose-friendly syntax as clean as ink for pure dialogue but schema-native by design. The compiler validates and produces a single `.urd.json` contract file. The runtime executes it. No custom glue code. No integration tax.

## The Problem

Narrative game development suffers from a **fragmentation tax**. Every team cobbles together ink or Yarn Spinner for dialogue, articy:draft or spreadsheets for world data, custom C# or GDScript for integration, and ad-hoc playtesting for QA. 30–50% of development effort goes to glue code that is never reusable.

There is no standard interface between "narrative data" and "game state." Urd is that interface.

## How It Works

Urd is a pipeline, not a monolith:

```
.urd.md files  →  Compiler  →  .urd.json  →  Runtime / Testing / Engine Integrations
```

**Writers** author scenes in Schema Markdown — prose with seven symbols (`@` `?` `>` `*` `+` `->` `==`). Their frontmatter is one line: `import: ./world.urd.md`. Everything else is narrative.

**Engineers** define types, entities, and behavioural rules in separate schema files. Writers never touch them.

**The `.urd.json` contract** is the boundary. The compiler produces it. Everything downstream consumes it. Self-contained, deterministic, versioned, human-inspectable.

## What It Looks Like

A writer's file:

```
---
import: ./world.urd.md
---

# The Rusty Anchor
A low-ceilinged tavern thick with pipe smoke.
[@arina]

== topics
@arina: What'll it be, stranger?

+ Ask about the harbour
  @arina: Quiet today. Too quiet.
  > @arina.trust + 5
  -> topics

* Ask about the missing ship
  ? @arina.trust > 50
  @arina: The Selene didn't sink. She was taken.

  ? @arina.trust <= 50
  @arina: I don't know what you're talking about.
  -> topics

* Leave -> harbour

@arina: Suit yourself. I've got glasses to clean.
```

`*` choices disappear after selection. `+` choices stay. `?` gates on conditions. `>` mutates state. `->` jumps. `@` references typed entities. The compiler validates everything. The runtime executes it.

## Schema Primitives

The Urd World Schema v0.1 covers:

- **Entities & Types** — typed objects with properties, defaults, and constraints
- **Containment** — one universal spatial primitive (a room holds a sword, a chest holds a sword, a player holds a sword — same mechanism)
- **Visibility** — properties are visible, hidden, owner-only, or conditionally revealed
- **Locations & Exits** — rooms with gated connections
- **Rules** — constrained NPC behaviour referencing hidden state
- **Sequences** — phased progression (game rounds, tutorials, rituals)
- **Dialogue** — sections, hubs, sticky/one-shot choices, exhaustion, jumps
- **Conditions & Effects** — a minimal expression language for state checks and mutations

## Components

| Component | Role | Status |
|-----------|------|--------|
| **PEG Grammar** | Formal pest grammar — 75 rules defining Schema Markdown syntax. | Complete |
| **JSON Schema** | 9 sub-schemas validating `.urd.json` contract output. | Complete |
| **Compiler** | `.urd.md` → `.urd.json`. Five phases: parse, import, link, validate, emit. Rust, 390 tests, 100% pass rate. Native CLI (`urd`) and WASM dual-target. | v0.1 complete |
| **Wyrd** | Reference runtime. Loads compiled JSON, executes the world, produces events. Browser-native. | Next milestone |
| **Testing** | Schema validation, reachability analysis, playthrough simulation, coverage reporting. | Planned |
| **LSP** | Language server wrapping the compiler. Live diagnostics, autocomplete, go-to-definition. | Planned |

Wyrd is the canonical runtime — any behavioural ambiguity in the spec is resolved by what Wyrd does.

## Repository Layout

```
briefs/           AI task briefs: backlog/ → active/ → done/
content/          Site content (markdown + frontmatter, Astro collections)
design/           Design system and theme definitions
docs/             Design documents (clean markdown, no frontmatter)
packages/
  compiler/       Rust compiler — 5-phase pipeline (.urd.md → .urd.json)
  grammar/        PEG grammar reference + pest parser + validation corpus
scripts/          Build tooling — test report generator, benchmark harness
sites/urd.dev/    Astro 5 static site — development journal
```

## Development

pnpm 10 monorepo. Rust for the compiler and grammar packages.

```bash
# Site
pnpm dev                    # Astro dev server
pnpm build                  # Production build
pnpm build:full             # Compiler tests + report + site build

# Compiler
pnpm compiler:test          # Run tests + benchmarks, generate report
pnpm compiler:test:raw      # Raw cargo test output
pnpm compiler:build         # Release build of the `urd` CLI binary
pnpm compiler:bench         # Release benchmarks + update report
pnpm compiler:wasm:check    # Verify WASM target compiles

# Grammar & Schema
pnpm grammar:test           # PEG validation corpus
pnpm schema:test            # JSON Schema validation
```

## Validation Strategy

Three progressive test cases, each proving capabilities the previous one couldn't:

1. **Monty Hall Problem** — Hidden state, NPC constraints, emergent probability. Run 10,000 games; the 2/3 switching advantage falls out of the structure.
2. **Two-Room Key Puzzle** — Spatial navigation, inventory via containment, persistent state, conditional NPC dialogue. Everything Monty Hall doesn't test.
3. **Connected Variant** — Cross-system composability. Can independently designed mechanics compose within a single world?

## Design Philosophy

**Declarative, not imperative.** The schema describes what the world *is*, not what it *does*. Emergent behaviour arises from structure.

**Containment as the universal spatial primitive.** `move: key, to: player` is "pick up." `key.container == player` is "does the player have the key?" One mechanism replaces rooms, inventory, and storage.

**Separation of content and presentation.** The schema contains no rendering instructions. How the world gets presented — text, 2D, 3D, voice — is entirely the runtime's concern.

**AI-native by design.** Every element is typed and unambiguous. A formal contract, not documentation. AI coding assistants can reason about what's valid without guessing.

**Writers should never need to edit a type definition or write a rule block.** If the syntax forces them to, the tooling has failed.

## The Approach

This project is an honest experiment in what one engineer can build when AI acts as a collaborator — not a replacement.

The scope of Urd would have been unfathomable for a solo effort a few short years ago. A schema specification, a compiler, a reference runtime, a testing framework, developer tooling, two websites — that's a team-sized roadmap. But the rules have changed. The research was conducted with AI. The architecture was designed with AI. The documents were authored, reviewed, and refined through conversation. The code was written the same way.

Every artefact in this repo is the result of that collaboration. The judgement is human. The throughput is superhuman.

Follow the build at **[urd.dev](https://urd.dev)**.

## Links

- **[urd.dev](https://urd.dev)** — Development journal. Transparent progress, technical artefacts, live test dashboard.
- **[urd.world](https://urd.world)** — Product face. Schema docs, live demos, getting started. Coming soon.

## Licence

Apache 2.0
