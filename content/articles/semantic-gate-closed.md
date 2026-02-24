---
title: "The Semantic Gate Is Closed"
slug: semantic-gate-closed
description: Six briefs, three tiers, one gate. From novel diagnostics to an MCP query surface for AI agents — the compiler now understands worlds, not just compiles them. 626 tests. The queryable world is complete. The runtime is next.
date: "2026-02-24"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the closure of the semantic gate and the transition from queryable world to executable world.
> Single canonical copy. February 2026.

## What closed

Two days ago, the [compiler gate closed](/articles/pre-alpha) at 0.1.7 with 554 tests. The compiler could parse, link, validate, and emit Urd worlds. It was correct. It was spec-complete. And it had a queryable semantic graph — the FactSet — that nobody was using yet.

The [semantic gate](/documents/semantic-gate) was the plan to change that. Six briefs across three tiers. Each one building on the FactSet to answer a question: *can the compiler's structural understanding of a world actually power real tools?*

Today, all six are done. The compiler is at 0.1.13 with 626 tests. The answer is yes.

## What was built

### Tier 1: Prove (SF-1A)

Five novel diagnostics that operate solely on the FactSet — no AST traversal, no source text parsing. Property read but never written. Property written but never read. Enum variant produced but never tested. Numeric threshold unreachable by any effect. Circular dependency where every write is guarded by a read of the same property.

These are questions that AST-walking checks cannot answer. They require cross-referencing reads against writes across the entire world. The FactSet makes each one a single-pass query.

One of them caught a real issue in an existing test world that no previous check detected. That was the validation gate for the entire FactSet design.

### Tier 2: Expose (SF-2, SF-3, SF-4)

**PropertyDependencyIndex.** A formal read/write index over every entity property. Direct lookups by property key. Set-difference queries for orphaned properties. JSON serialisation into the WASM pipeline. Visible in the playground's analysis panel.

**Graph visualisation.** Location topology and dialogue flow, both reconstructed entirely from FactSet tuples. No AST fallback. Dagre layout, pan/zoom, diagnostic-driven node flags. If a location is unreachable or a choice is impossible, you see it in the graph.

**Semantic diff.** Compare two compiled worlds and get a typed change report across six categories: entities, exits, dialogue, property dependencies, rules, reachability. Not a text diff. Not a JSON diff. A structural diff that knows the difference between "an exit was added" and "a property write was removed." CLI command for CI: `urd diff before.urd.md after.urd.md`.

### Tier 3: Operationalise (SF-5, SF-6)

**LSP server.** Recompile-on-save language server with diagnostics, go-to-definition, hover, and autocomplete. Four capabilities backed by three data sources: DefinitionIndex for declarations, FactSet for relationships, PropertyDependencyIndex for analysis. Hover over `@entity.property` and see its type, default, and how many places read and write it. 20 tests against a mock LSP client.

**MCP server.** Eight read-only tools exposed via the Model Context Protocol for AI agents. `get_world_metadata`, `get_exit_graph`, `get_dialogue_graph`, `get_entity_details`, `get_property_dependencies`, `get_reachable_locations`, `get_choice_conditions`, `get_diagnostics`. Each returns structured JSON with `schema_version: "1"`. An AI agent with MCP access can reason about a world's structure — spatial connectivity, property dependencies, choice gating — without simulating a single turn.

## What this means

The [semantic-first essay](/articles/semantic-first) argued that extracting structural relationships at compile time would make every downstream capability cheaper to build. The semantic gate is the evidence.

Each brief was a different consumer of the same six tuple types. Diagnostics query reads against writes. The diff engine normalises tuples into comparable snapshots. The LSP projects them into hover information. The MCP server wraps them in JSON-RPC tools. The underlying data is the same. The queries compound.

The compiler went from 554 tests to 626. From one crate to three (`compiler`, `lsp`, `mcp`). From a batch tool that emits JSON to a system that powers real-time editor feedback and structured AI queries. The FactSet — six tuple types, extracted in a single deterministic pass — is the reason all of this composes.

## What is next

The runtime. Wyrd. The thing that actually executes worlds.

The [system gate](/documents/system-gate) defines 21 runtime requirements. Immutable state transitions. Event sourcing. Deterministic evaluation with seeded randomness. The proof-of-concept target: Monty Hall running in the browser, converging to 2/3 with identical seeds.

But Wyrd will not be a standalone interpreter. It will receive the FactSet alongside the compiled world. Explain mode — "why is this choice unavailable?" — will use runtime state plus FactSet structure. Cache invalidation will use the PropertyDependencyIndex to skip re-evaluation of unaffected conditions. The MCP server will extend with runtime tools: `get_state`, `get_available_actions`, `perform_action`. The semantic diff will become the regression test primitive.

The queryable world is complete. The executable world begins.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
