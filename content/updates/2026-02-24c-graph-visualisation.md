---
title: "SF-3: Location and Dialogue Graph Visualisation"
date: "2026-02-24"
---

Two new tabs in the playground Analysis panel render the FactSet's structural relationships as interactive SVG graphs. The Location tab shows the topology of exits between locations (dagre LR layout); the Dialogue tab shows the flow of jumps and choices between sections (dagre TB layout). Both use a shared renderer with pan/zoom, diagnostic-driven node flags (unreachable locations, impossible choices), and clickable nodes. A prerequisite FactSet extension adds `jump_indices` to `ChoiceFact` for explicit choice-to-jump correlation. 584 tests, zero regressions.
