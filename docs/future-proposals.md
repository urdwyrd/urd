---
title: "Urd — Future Proposals"
slug: "future-proposals"
description: "Design work for capabilities planned beyond v1. These proposals are not part of the v1 specification and MUST NOT be implemented by v1 runtimes. They are preserved here to inform forward-compatible architectural decisions."
category: "strategy"
format: "Proposal Collection"
date: "2026-02-12"
status: "draft"
order: 99
tags:
  - future
  - proposals
  - lambda
  - extensions
details:
  - "Post-v1 design work — not part of the v1 specification"
  - "Lambda functions, dynamic entity creation, meta-programming"
  - "Forward-compatible architectural guidance for v1 implementation"
---

> **Document status: INFORMATIVE — NOT IN V1**
> Nothing in this document is part of the v1 specification. v1 runtimes MUST NOT implement any feature described here. v1 worlds MUST NOT use any construct described here. This document preserves design work for future schema versions and informs forward-compatible architectural decisions in the v1 implementation.

# URD

## Future Proposals

*Design work for capabilities beyond v1*

urd.dev · February 2026

---

## Lambda Functions (Extension Host)

### Motivation

Some game logic is awkward to express declaratively. Pathfinding, economic calculations, weighted random selection, procedural generation: these are naturally imperative. Lambda functions provide an escape hatch without breaking the declarative model.

### The Contract

A lambda function is declared in the schema and executed by the extension host. The contract is strict:

- **Input:** A read only snapshot of world state. The lambda receives entity properties, containment, and any parameters declared in the schema. It cannot access the DOM, the network, or anything outside the world model.
- **Output:** A list of effects in the same format as authored `>` lines: set, move, reveal, destroy, spawn. The runtime applies them exactly as it would apply any other effects.
- **No side effects.** A lambda cannot mutate world state directly. It returns effects; the engine applies them. This preserves the event sourcing model and keeps the world inspectable and testable.
- **Deterministic when possible.** Lambdas that use randomness must accept a seed parameter so that tests can reproduce results. The engine provides the seed; the lambda uses it.

### How They're Declared

```
---
import: ./world.urd.md

lambda calculate_trade_value:
  receives: [@merchant, @player, @offered_item]
  returns: effects
  source: ./lambdas/trade.js
---
```

The lambda block declares the function's name, the entities it receives, and its source file. The writer never sees this. Rules or actions can invoke the lambda: `> apply calculate_trade_value(@merchant, @player, @sword)`. The runtime passes the entity snapshots to the function and applies the returned effects.

### Sandboxing

Lambdas run in a sandboxed JavaScript environment with no access to the DOM, network, filesystem, or global state. In the browser, this is a Web Worker. In Node.js (CI/testing), this is a vm context. The sandbox enforces a time limit: a lambda that runs for more than 100ms is terminated with an error event.

### Design Principle

Every Urd world must work with purely authored content and declarative rules. Lambdas extend what's expressible; they don't replace the core model. The extension host is designed in the v1 architecture so that when a future schema version introduces lambda support, the runtime is ready without a rewrite.

### Implementation Notes for v1

The Wyrd reference runtime architecture includes an Extension Host component slot. In v1, this slot is empty. The core engine's effect application pipeline is designed to accept effects from any source (authored, rule-generated, or lambda-returned) without modification. This forward-compatibility is the only v1 requirement.

---

## OR Condition Writer Syntax

**RESOLVED — Shipped in v1.** The `? any:` syntax was promoted to v1 during specification review. See the Schema Markdown Syntax Specification, section "Decisions Locked for v1 → OR Conditions" for the definitive syntax.

---

## Cross-File Section Jumps

### Motivation

In v1, dialogue sections are file-scoped. A `-> name` jump can only target sections in the same file. Cross-location movement uses exits. This is sufficient for most content but becomes limiting in large worlds where a conversation in one location needs to reference or continue dialogue defined in another file.

### Proposed Syntax

```
-> harbor.urd.md/dockside_argument
```

Path-qualified section references. The compiler resolves the file path relative to the current file.

### Open Questions

- Should exhaustion state be shared across cross-file section references?
- How does this interact with the import system? Must the target file be imported?
- What are the error diagnostics when a cross-file target is missing or renamed?

---

## Owner Visibility Semantics

The `~~` token is reserved in Schema Markdown for owner visibility. Full semantics depend on the ownership model, which is not yet specified. Key questions include how ownership is declared, whether ownership can transfer, and how owner-visible properties interact with the presentation layer.

---

*This document will grow as new capabilities are designed. Each proposal follows the same pattern: motivation, proposed syntax or structure, open questions, and implementation notes for forward compatibility.*

*End of Document*
