---
title: "SF-2: PropertyDependencyIndex — shared query surface"
date: "2026-02-24"
---

The PropertyDependencyIndex gains set-difference queries (`read_but_never_written`, `written_but_never_read`), deterministic JSON serialisation via `to_json()`, and a property-centric playground panel with filter tabs. D1/D2 diagnostics refactored to call the index methods directly — the playground's orphaned-property filters and the compiler diagnostics are now a single source of truth, verified by test. 580 tests, zero regressions.
