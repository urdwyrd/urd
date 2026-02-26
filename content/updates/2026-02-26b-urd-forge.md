---
title: "Urd Forge: desktop IDE architecture specified"
date: "2026-02-26"
link: "/articles/urd-forge"
milestone: true
---

Urd Forge is a Tauri-based desktop IDE for authoring Urd worlds — Blender-style tiling layout, 85+ specialised views (spreadsheets, node graphs, matrices, inspectors, visualisations) projected over a single compiler output, Rust backend via IPC, Svelte 5 frontend, CodeMirror 6 editor. Full architecture specification complete: BSP zone tree, command registry, message bus, projection layer, workspace management, theme engine, eight-phase implementation plan. Framework and application layers cleanly separated — the generic IDE framework has no knowledge of Urd.
