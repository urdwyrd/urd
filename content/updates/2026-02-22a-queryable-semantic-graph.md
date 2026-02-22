---
title: "Compiler 0.1.6 — FactSet analysis IR"
date: "2026-02-22"
link: "/articles/queryable-semantic-graph"
milestone: true
---

The compiler now produces a FactSet — a flat, queryable intermediate representation of every resolved relationship in the compiled world. Six fact types (PropertyRead, PropertyWrite, ExitEdge, JumpEdge, ChoiceFact, RuleFact) plus a PropertyDependencyIndex. Extracted after LINK, available on CompilationResult regardless of VALIDATE errors. WASM output now includes serialised facts. 547 tests, zero regressions.
