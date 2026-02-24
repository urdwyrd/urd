---
title: "SF-1A: FactSet Diagnostics Land"
date: "2026-02-24"
---

Five new diagnostics (URD601–URD605) that operate solely on the FactSet analysis IR — no AST, no source text, no symbol table. The Sunken Citadel stress test yields 27 real findings that no existing check detects: dead writes, untested enum variants, unreachable thresholds, and circular property dependencies. First brief of the semantic gate closed. Compiler bumped to v0.1.8.
