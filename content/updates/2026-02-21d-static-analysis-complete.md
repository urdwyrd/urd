---
title: "Compiler 0.1.5 — static analysis complete"
date: "2026-02-21"
link: "/articles/static-analysis-complete"
---

Four new VALIDATE-phase checks close the Static Analysis gate: unreachable location (URD430), orphaned choice (URD432), missing fallthrough (URD433), and section-exit shadowing (URD434). All eight static analysis checks (S1–S8) are now implemented. The playground now shows warnings alongside compiled output. 516 tests, zero regressions.
