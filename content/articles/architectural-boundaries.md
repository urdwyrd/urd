---
title: "The Document to Read First"
slug: architectural-boundaries
description: A new governance document defines what belongs in the schema, what belongs in the runtime, and what belongs in neither — and why it matters as v1 nears completion.
date: "2026-02-21"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Why the Architectural Boundaries document exists and what it means for the project.
> Single canonical copy. February 2026.

## The problem it solves

Every system accretes features. The pressure is constant, legitimate, and hard to resist: writers want conditional room descriptions, engineers want convenience fields, benchmarks assume parser-IF conventions. Each request is reasonable in isolation. Taken together, they erode the constraint that makes the system worth building.

Urd's value is that it is *deliberately less than a programming language*. That constraint enables machine verification, static analysis, deterministic replay, and AI-native world consumption. But constraints only hold if someone writes them down and refuses to negotiate them away one reasonable feature request at a time.

The [Architectural Boundaries](/documents/architectural-boundaries) document is that written-down refusal. It defines the three-layer architecture — schema, runtime, adapter — and provides a repeatable five-question test for deciding where any proposed feature belongs. It names five categories of concern that are permanently excluded from the schema and runtime, not deferred but excluded by design.

## Why now

The project has a specification suite, a PEG grammar, a JSON Schema, a five-phase compiler passing 480 tests, and a browser playground with live validation. The pieces are converging toward v1. This is exactly the moment when the boundary matters most.

Before this document, the boundaries were implicit. The Architecture document describes the pipeline and component interfaces. The Schema Spec describes what the schema contains. The Wyrd Reference Runtime describes what the runtime does. But none of them answer the question that surfaces with every new feature proposal: *where does this belong?*

"Add conditional room descriptions" — is that a schema feature? "Add verb synonyms" — schema or adapter? "Fire a rule when an action fails" — world semantics or interface feedback? Without a framework, each decision is ad hoc. The same question gets relitigated with every proposal.

The Architectural Boundaries document closes that gap. It provides:

- **Precise definitions** of content, hint, and presentation — three terms that recur throughout the specification suite and now have unambiguous meanings
- **A five-question boundary test** that can be applied mechanically to any proposal
- **Five permanent exclusions** — input model, text rendering, experience feedback, time, and persistence — that are out of scope by design, not by accident
- **A full audit** of every existing v1 field against the framework, confirming that none violate the boundary
- **Worked examples** showing how common proposals (conditional descriptions, parser grammar, fumble actions, always triggers, on_attempt triggers) are evaluated

## Where to start reading

If you are new to the project and want to understand what Urd is and is not, this document is the best starting point. It answers the question that the other documents assume you already know: what is the system's shape, and what will it refuse to become?

The three-layer diagram in §The Three Layers is the clearest single-page summary of the architecture. Layer 1 (Urd) defines what exists. Layer 2 (Wyrd) simulates what happens. Layer 3 (the adapter) handles everything the player sees, types, clicks, or hears. Each layer has a single responsibility. No layer reaches into another.

The boundary test in §The Boundary Test is the operational core. Five questions, applied in order:

1. Does it describe what exists? → Schema
2. Does it describe a rule about what can happen? → Schema (if declarative) or adapter (if imperative)
3. Does it depend on adapter behaviour? → Adapter (even if it passed question 2)
4. Does it describe state evaluation mechanics? → Runtime
5. Does it describe how the player perceives the world? → Adapter

The permanent exclusions in §Permanent Exclusions are the document's strongest claims. These are not features waiting for a later version. They are architectural decisions that constrain every version. The input model exclusion means Urd will never know about verbs. The time exclusion means there is no clock. The persistence exclusion means save/load is always external.

## The governance role

This is the first normative governance document in the project. The specification documents define what the system *is*. This document defines what the system *is allowed to become*. It sits above the other specifications in a precise sense: the Schema Spec's "Design Principles" section is compatible with and subordinate to this boundary framework.

When a proposal fails the boundary test, the document prescribes a process: document the rejection, document the adapter-layer solution, and if the proposer believes the boundary itself is wrong, require them to argue that case explicitly. The bar is high by design. Moving the boundary affects every downstream implementation.

## The full document

The complete Architectural Boundaries document is available in the [artefacts section](/documents/architectural-boundaries) and on [GitHub](https://github.com/urdwyrd/urd/blob/main/docs/urd-architectural-boundaries.md).

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
