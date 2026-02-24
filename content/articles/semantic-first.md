---
title: "Semantic-First: The Next Step"
slug: semantic-first
description: Interactive fiction runtimes execute worlds. Urd's FactSet means the world can be queried, inspected, and reasoned about before, during, and after execution. By humans, by tools, and by machines. That is a different kind of system.
date: "2026-02-23"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> An essay on why compile-time semantic extraction changes the category of what an interactive world runtime can be, and why we are building it before the runtime.
> Single canonical copy. February 2026.

## The obvious next step, and why we are not taking it

The [compiler gate is closed](/articles/pre-alpha). 554 tests. Nine compiler requirements, eight static analysis checks, eight FactSet IR requirements, seven specification consistency items. Every one verified. The compiler is spec. v1 complete.

The obvious next step is the runtime. Wyrd. The thing that actually executes worlds. The [system gate](/documents/system-gate) defines 21 runtime requirements, 5 testing requirements, and a boundary gate. The proof-of-concept, Monty Hall running in the browser with seeded random convergence to 2/3, is the single most compelling demo we could ship. Nobody can play an Urd world yet. The tagline says "Define in Urd. Test in Wyrd. Ship Anywhere." Two-thirds of that does not exist.

We are not building the runtime next.

We are building the tooling, diagnostics, and infrastructure that sit on top of the FactSet, the compiler's queryable semantic graph. Derived diagnostics. Property dependency analysis. Location graph visualisation. LSP foundations. The work that deepens the compiler before we move downstream.

The reasoning is strategic. The FactSet already models the same relationships the runtime will need to evaluate. Exits between locations, jumps between dialogue sections, which properties get read by conditions, which properties get written by effects. Every query we build over these tuples is a dry run of logic the runtime will need. If property read/write analysis reveals gaps in the data model (awkward modelling, missing coverage, ambiguous semantics) we find that now, while the cost of changing the schema is low, not after we have committed to a runtime state management design that depends on it.

There is also a quality argument. The more derived diagnostics the compiler ships ("property read but never written", "exit leads to location with no content", "condition tests an enum variant no effect ever produces") the higher quality the worlds arriving at the runtime will be. Wyrd's job gets simpler when the compiler has already rejected or warned about the degenerate cases. Every bug the compiler catches is a bug the runtime never has to handle.

And the infrastructure compounds. The WASM to JSON to Svelte island pipeline we proved for the playground and analysis panel is the same pipeline the runtime's presentation layer will use. Every visualisation we build (location graphs, dialogue flow, property dependencies) exercises that pipeline further and builds fluency with it before the stakes are higher.

The runtime is greenfield. Zero lines exist. Even the Monty Hall proof-of-concept requires ten of the twenty-one runtime requirements. The FactSet tooling builds on 554 passing tests, a working WASM target, a live playground, and an analysis IR that is already extracted, serialised, and displayed in the browser. The marginal cost of each new capability is low because the foundation is already proved.

We are investing in the queryable world before we invest in the executable one. Not because execution does not matter (it is the entire point) but because the queryable world makes the executable one better, and because this is the moment where that investment is cheapest.

This essay explains what that investment produces and why it points toward a different kind of runtime: one we are calling *semantic-first*.

## The runtime everyone builds

Every interactive fiction runtime works roughly the same way. It loads a world definition, whether a script, a graph, or a database, and executes it. The player does something. The runtime evaluates conditions, applies effects, advances state. It returns what happened. The player does something else. The loop continues until the story ends or the player leaves.

This is the execution model. It has been the execution model since Infocom. Inform, ink, Twine, Yarn Spinner, articy:draft, Ren'Py: different syntaxes, different paradigms, but the same fundamental loop. Input, evaluate, mutate, output.

The model works. It produces games people love. But it has a structural limitation that becomes visible the moment you try to do anything other than play the game forward.

Ask ink "which variables does this choice depend on?" and ink cannot tell you. It can evaluate the choice, it can tell you whether the choice is available right now given current state, but it cannot enumerate the dependencies. The knowledge is embedded in the execution path, not extracted as queryable data.

Ask Inform "is there a sequence of actions that reaches this room?" and Inform cannot answer without simulating every possible path. The spatial topology is implicit in the source code, not materialised as a graph you can traverse.

Ask Twine "does this variable ever get set to a value that makes this condition true?" and Twine has no mechanism to even represent the question. It runs passages. It does not reason about them.

These are consequences of architecture, not failures of engineering. When the world definition is a program, when conditions are expressions evaluated at runtime and effects are mutations applied in sequence, the only way to answer structural questions about the world is to execute it. And executing it means simulating, which means combinatorial explosion, which means the questions that matter most are the ones that are hardest to answer.

A semantic-first system inverts this. It extracts the structural relationships at compile time, makes them queryable, and hands both the executable world and the semantic graph to every downstream consumer. Execution is still essential. But it is no longer the only way to understand the world.

## What the FactSet is

Urd's compiler does not just validate syntax and emit JSON. After the LINK phase resolves every cross-reference in the world (entity to type, property to definition, jump target to section, exit to location) it extracts a flat, immutable, deterministic set of typed tuples called the FactSet.

Six relation types. Each one captures an atomic relationship the compiler already proved:

**PropertyRead.** A condition somewhere in the world reads an entity property. The FactSet records which property, which comparison operator, which literal value, and where the read occurs: in a choice guard, an exit condition, or a rule's where clause.

**PropertyWrite.** An effect somewhere in the world modifies an entity property. The FactSet records which property, which operator (set, add, subtract), which value expression, and where the write occurs.

**ExitEdge.** One location connects to another through a named exit. The FactSet records whether the exit is conditional and indexes into the guard reads that gate it.

**JumpEdge.** One dialogue section declares a transition to another section, an exit, or the terminal `end`. These are the edges of the conversation graph.

**ChoiceFact.** A player choice exists within a section. The FactSet records its label, whether it is sticky or one-shot, and indexes into both its condition reads and its effect writes.

**RuleFact.** A rule exists with trigger conditions and effects. The FactSet indexes into the reads and writes that constitute it.

The FactSet is a materialised view of what the compiler already resolved, projected into normalised tuples that any consumer can query without touching the AST, without walking the source, without executing anything. It is extracted once, after compilation. It is the same every time for the same source. It is immutable. And it answers, trivially, questions that execution-based runtimes typically cannot answer without simulation.

## The questions that become trivial

"Which conditions depend on `Guard.trust`?" Filter `PropertyRead` by entity type and property name. Every site that reads it, every comparison operator, every threshold value. One pass over a flat array.

"Which effects can change `Guard.trust`?" The same query over `PropertyWrite`. Every site that writes it, every operator, every value expression.

"Is there a path from the tavern to the treasury?" The `ExitEdge` tuples form a directed graph. Breadth-first search. The compiler already does this for the S3 unreachable location check.

"What happens if the player exhausts every choice in this section?" The `ChoiceFact` tuples for the section enumerate every choice. Their `condition_reads` indexes tell you what gates them. Their sticky/one-shot flags tell you whether they persist. The `JumpEdge` tuples from the section tell you where exhaustion leads.

"Is this property ever written before it is read?" The `PropertyDependencyIndex` maps each property key to its read and write site indices. If a key appears in reads but not writes, something is checking a value that nothing sets. If it appears in writes but not reads, something is setting a value that nothing checks.

The first four are already implemented or directly derivable from the existing FactSet code. The fifth is roughly ten lines of Rust. Each one would require a full AST traversal, or runtime simulation, in any system that does not extract semantic relationships at compile time.

## Why this matters for the runtime

Wyrd, the reference runtime, does not exist yet. It is next. When it is built, it will load compiled `.urd.json` files and execute them: condition evaluation, effect application, state transitions, dialogue management, event sourcing.

But Wyrd will also receive the FactSet. That is what makes it semantic-first rather than execution-first.

### Explain mode

An AI agent, or a human tester, or a debugging panel asks: "Why is this choice greyed out?" In a traditional runtime, the answer requires re-evaluating the choice's conditions against current state and reporting which ones failed. That works for simple cases. It fails when the question is "why is this choice *always* greyed out?" or "what would I need to change to make this choice available?"

With the FactSet, the runtime can answer structurally. The choice has condition reads. Those reads reference specific properties with specific comparison operators and threshold values. The runtime evaluates them against current state to determine which failed. Then it consults the FactSet's write index for those properties to determine which effects *could* change them. The answer is not "the condition failed" but "this choice requires `Guard.trust >= 3`, which is currently 1, and the only effects that write `Guard.trust` are in the bribe choice and the defend rule."

That is a semantic answer. It requires both execution state and compile-time structure. Neither alone is sufficient.

### Cache invalidation

Wyrd's architecture specifies immutable state transitions, where each action produces a new world state. Event sourcing records every mutation. Determinism is contractual.

But not every action affects every part of the world. When the player moves from the tavern to the market, which conditions need re-evaluation? In a naive runtime, all of them. With the FactSet, the answer is precise: only conditions that read properties written by the move effect, or that depend on the player's current location. The `PropertyDependencyIndex` makes this a lookup, not a scan.

For small worlds this is irrelevant. For worlds with hundreds of entities, thousands of properties, and complex rule chains, it is the difference between a runtime that scales and one that does not.

### Determinism verification

Urd's determinism contract states that identical JSON, identical seed, identical actions must produce identical event streams. Verifying this requires running the same playthrough twice and comparing results. The FactSet does not change that. But it does tell you *which* properties and rules are involved in any given action resolution, which means a failing determinism test can report not just "the event streams diverged at step 47" but "the divergence is in the evaluation of `Guard.trust >= 3` which was written by rule `bribe_effect` at step 31."

## What this enables beyond the runtime

The FactSet sits between the compiler and every downstream consumer. The runtime is one consumer. It is not the only one.

### AI agents and the Model Context Protocol

An AI agent interacting with an Urd world through a structured API (the architecture's adapter layer) can query available actions, evaluate conditions, and select actions. Most runtimes support this. But an agent with access to the FactSet can also reason about the world's structure without simulating it.

"What is the shortest path to the treasury?" Graph query over `ExitEdge` tuples. "Which actions affect the guard's mood?" Property write query. "If I set `@guard.trust` to 3, which new choices become available?" Join reads against the threshold, filter by current state. These are planning queries. They require structure, not simulation. An agent with FactSet access can plan before it acts.

The current approach to AI-driven interactive fiction is to wrap a runtime in a LangChain pipeline or MCP tool, give the agent `get_available_actions()` and `perform_action()`, and hope it explores effectively. The agent has no map. It discovers by doing. With the FactSet exposed as a queryable tool, the agent has a map. It can reason about reachability, dependency, and consequence before committing to an action.

MCP, the emerging standard for giving AI models structured access to external tools, is the natural delivery mechanism. An Urd world exposed through MCP would provide the obvious runtime operations (`get_state`, `get_available_actions`, `perform_action`) alongside FactSet resources (`get_exit_graph`, `get_property_dependencies`, `get_choice_conditions`, `get_reachable_locations`). The FactSet is already serialised to JSON for the playground's analysis panel. Wrapping it in MCP resource endpoints is mechanical. The hard part, extracting, normalising, and cross-referencing the semantic relationships, is already done at compile time.

### Multiplayer and shared worlds

Urd's containment model (every entity exists inside exactly one container at any time, `move` is the sole spatial operation) is designed for single-player worlds. But the model is not inherently single-player. Multiple actors can issue actions against the same world state. The runtime's immutable state transitions and event sourcing mean concurrent actions can be serialised deterministically.

The FactSet makes multiplayer more tractable. When two players act simultaneously, which properties are in contention? The `PropertyDependencyIndex` answers this at compile time: if Player A's available actions write `Guard.trust` and Player B's available actions read `Guard.trust`, there is a potential conflict. If their action sets touch entirely disjoint property sets, they can be resolved in parallel. Conflict detection, action serialisation, and state partitioning all depend on knowing which actions touch which state, and that is exactly what the FactSet provides.

### Tooling, inspection, and analysis

The IDE vision for Urd includes separate writer and engineer modes. The engineer mode needs a property dependency graph, a location topology view, a dialogue flow visualisation, and a "why is this happening?" debugging panel. Every one of these is a FactSet query, and they all share the same underlying pattern: project a subset of the six tuple types into a visual or analytical form.

Property dependencies: `PropertyDependencyIndex.read_properties()` and `written_properties()` give you the nodes; cross-referencing reads and writes gives you the edges. Location topology: `ExitEdge` tuples rendered as a directed graph. Dialogue flow: `JumpEdge` plus `ChoiceFact` tuples forming the conversation graph. Debugging: runtime state plus FactSet structure, where the runtime says what happened and the FactSet says why.

The same pattern applies to static analysis. The existing checks (S3 through S8) were built before the FactSet existed and each reimplements its own AST traversal. With the FactSet, new checks become queries. "Property read but never written" is a set difference. "Choice condition tests an enum variant that no effect produces" is a join. "Dialogue section has no incoming jump edges" is a graph query. Each new check is a function that takes a `&FactSet` and returns diagnostics, with no special knowledge of the compiler's internal representation beyond the six tuple types. The extraction is done once. The queries compound.

## What semantic-first means

Traditional interactive fiction systems optimise their internal representations for execution. They are interpreters. They run stories forward. Asking structural questions means running them, or instrumenting them, or building separate analysis tools that duplicate the world model.

Urd compiles the world into an executable format *and* extracts a queryable semantic graph from the same compilation pass. The two representations are derived from the same source, proved consistent by the same compiler, and available to any downstream consumer.

The executable format, `.urd.json`, is what the runtime loads. It is the contract between authoring and execution.

The semantic graph, the FactSet, is what everything else uses. Static analysis. Debugging. Visualisation. AI planning. Conflict detection. Cache invalidation. Explain mode.

We are calling this approach *semantic-first* because the structural understanding of the world is not an afterthought bolted onto an interpreter. It is a primary output of compilation, available before the runtime ever loads the file. Execution and analysis are peers, not parent and child.

The FactSet is six tuple types, roughly 600 lines of Rust, extracted in a single deterministic pass after compilation. It is the smallest investment that makes every future capability cheaper to build. And it is the reason Urd's runtime, when it arrives, will not be another interpreter that runs stories forward and hopes for the best. It will be an engine that knows what it is running, can explain its own decisions, and exposes its entire semantic structure to any tool, agent, or human that asks.

That is what we are building toward. And that is why we are building the queryable world first.

The concrete plan — six briefs across three tiers, from novel diagnostics through LSP foundations to an MCP query surface — is tracked in the [semantic gate](/documents/semantic-gate).

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
