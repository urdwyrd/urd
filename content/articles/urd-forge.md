---
title: "Urd Forge"
slug: urd-forge
description: A desktop IDE where every view — spreadsheets, node graphs, matrices, inspectors — is a reactive lens into the same compiled world. Blender-style tiling. 85+ specialised views. One source of truth.
date: "2026-02-26"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Announcing Urd Forge — the desktop IDE for authoring Urd interactive fiction worlds.
> Single canonical copy. February 2026.

## The next step

The compiler is at [v0.1.14](/documents/urd-v0.1.14-reference) with 634 tests, six phases, 74 diagnostic codes, a semantic diff engine, and an IDE-ready definition index. The [playground](/playground) has inline diagnostics, autocomplete, hover tooltips, and go-to-definition. The [Code Editor](/articles/playground-editor) understands 25 reference kinds drawn from the compiler's queryable semantic graph.

All of this runs in a browser tab. One file at a time. One editor pane. One analysis panel.

That is not enough.

Urd worlds are complex systems. A writer working on a world with fifty locations, two hundred entities, and a thousand property dependencies needs to see the location graph and the entity spreadsheet and the property data flow and the dialogue flow and the diagnostic list — simultaneously. Not in tabs. Not behind menu clicks. On screen, all at once, each one showing the same truth from a different angle.

That is what Urd Forge is.

## What Urd Forge is

Urd Forge is a Tauri-based desktop IDE for authoring Urd interactive fiction worlds. It provides Houdini-grade information density through a Blender-style tiling layout engine, with 85+ specialised views projected over a single compiler output.

Every view — spreadsheets, node graphs, matrices, inspectors, visualisations — is a reactive lens into the same underlying data. Click an entity in a spreadsheet, and the location graph highlights the node containing it, the inspector switches to show its details, and the code editor scrolls to its declaration. Change a property in the source, and every view that depends on it updates within the same render frame. There is one source of truth.

The application is not a web app in a wrapper. It is a native desktop application with full keyboard sovereignty, custom context menus, controlled text selection, and no browser chrome leaking through. Ctrl+P opens the command palette, not the print dialogue. F11 is fullscreen, not a developer tool. Right-click opens a context-sensitive menu built from the command registry, not the browser default.

## The layout engine

The layout is a recursive binary space partition — the same technique Blender uses for its area system. The screen is a tree of splits. Each leaf is a zone. Each zone can display any of the 85+ registered view types. Split, join, swap, resize — all through right-click context menus on the dividers between zones, or through keyboard commands.

Five predefined workspace templates give writers a starting layout: Writer (file browser, code editor, dialogue preview, play panel), Engineer (file browser, code editor, property spreadsheet, diagnostics), World Builder (code editor, location graph, entity spreadsheet), QA (code editor, coverage overlay, Monte Carlo dashboard, dead code panel), and Debug (play panel, state inspector, event log, code editor). Every template is a starting point — the writer reshapes the layout to fit their workflow, and the workspace persists across sessions.

Workspace tabs sit at the top, Blender-style. Switch between the Writer layout for prose editing and the World Builder layout for spatial design. Each workspace is a complete BSP tree with its own zone configuration. The code editor — the only singleton view — carries its open tabs and cursor positions across workspace switches without remounting.

## The architecture

Three architectural principles define the system.

**The IDE is projections.** Every view renders a transformation of a single `CompilerOutput`. The compiler runs in Rust via Tauri IPC, producing chunked output with content hashes. A projection layer sits between the compiler and the views, providing memoised, view-friendly datasets that recompute only when their input chunks change. If you edit a dialogue file and only the AST and diagnostics chunks change, the entity spreadsheet returns the exact same cached object by reference. Svelte 5's fine-grained reactivity detects no change and skips the re-render entirely.

**Everything is a command.** Every user-triggerable action — splitting a zone, opening a file, compiling the world, toggling the theme — is a registered command with a unique ID, typed parameters, a keybinding, and an undo hook. The command palette, menus, toolbars, keyboard shortcuts, and context menus all dispatch through the same command registry. Global undo is not a v1 feature, but every command returns an `UndoAction` from day one so the architecture is validated before the stack exists.

**Framework and application are separated.** The layout engine, zone system, command registry, message bus, workspace management, theme engine, navigation broker, and settings service form a generic IDE framework layer. All Urd-specific views, compiler integration, and domain logic form an application layer. The framework has no knowledge of Urd. The separation is annotated by import direction — application imports framework, never the reverse. The same framework could power an IDE for a different domain.

## The data flow

The single path from keystroke to pixel:

The user types in the code editor. The buffer map updates in memory. After a 300ms debounce, the Rust compiler runs via Tauri IPC — parse, import, link, analyse, validate, emit. The chunked output comes back with content hashes. The output cache reuses unchanged chunks by hash, avoiding redundant JSON parsing and garbage collection. The projection registry updates its source. A lightweight signal (compile ID and chunk hashes, no data payload) publishes on the message bus. Views wake up, call the projection registry, and get back either a cache hit (same reference, no re-render) or a freshly computed dataset. Svelte renders the diff.

The bus carries signals, never data. Projections are the sole data authority. If two views disagree, the projection is wrong, not the views.

## The views

Eighty-five specialised views across ten categories:

**Spreadsheets** (14 views). Entity, type, property, location, section, choice, rule, exit, jump, read, write, sequence, diagnostic, and file spreadsheets. Each is a virtualised table rendering only visible rows, backed by a shared `VirtualTable` component. Sort, filter, column resize. Click a row to select it across every view in the workspace.

**Graphs** (10 views). Location network, dialogue flow, type hierarchy, property data flow, file dependency, rule trigger network, containment tree, sequence timeline, choice tree, and cross-reference graph. All share a pan/zoom canvas with spatial culling — only nodes and edges within the viewport are rendered.

**Inspectors** (7 views). Entity, type, location, section, rule, property, and diagnostic inspectors. Each shows the full detail of the currently selected item, pulling from projections and the definition index.

**Analysis** (10 views). Reachability matrix, condition/effect matrix, enum coverage, threshold analysis, visibility audit, circular dependency detection, dead code panel, world statistics dashboard, diff view, and narrative flow visualiser. Some run on every recompile; others (Monte Carlo, reachability matrix) are triggered by explicit user action and stream progress from the Rust backend.

**Search** (8 views). Global symbol search, find references, where-used, what-if, impact analysis, regex search, property value search, and condition predicate search.

**Runtime** (7 views). Play panel (Wyrd integration), state inspector, event log, breadcrumb trail, coverage overlay, Monte Carlo dashboard, and deterministic replay.

**Specialised** (10 views). World map, property timeline, dialogue preview, dependency heatmap, compilation pipeline visualiser, schema browser, test report dashboard, comparison view, annotation layer, and AI assistant panel.

Plus the code editor (singleton, with tab bar, breadcrumb, CodeMirror 6, inline diagnostics, code lens, hover tooltips), the source viewer (lightweight read-only editor for side-by-side reference), the file browser (project tree with lazy expansion, dirty/error indicators, keyboard navigation), and the settings view (categorised settings editor with keybinding capture).

## The technology

Tauri 2.x for the native shell. Rust for the backend — the Urd compiler already runs in Rust, heavy computation stays there. Svelte 5 in runes mode for the frontend — compiler-optimised reactivity with minimal runtime and surgical DOM updates. CodeMirror 6 for the code editor — embeddable, extensible, excellent performance in constrained containers. CSS custom properties for theming — two built-in themes (Gloaming dark, Parchment light) expressed entirely as token references, zero runtime cost to switch. pnpm for package management. Vite for builds with Tauri integration.

The desktop variant runs the compiler natively via Tauri IPC with real filesystem access. A future web variant could run the compiler as WASM in a Web Worker pool with an IndexedDB virtual filesystem. Both would consume identical `CompilerService` and `FileSystem` interfaces — views never import an implementation directly.

## The implementation plan

Eight phases. Framework shell first, then core infrastructure, then the editor, then views in order of data availability.

**Phase 1** builds the framework shell: BSP layout engine, zone system, workspace tabs, theme engine, global menu bar, status bar, custom context menus, placeholder views for testing, native app hardening. The milestone is an acceptance test — split, join, swap, resize, change zone type, switch workspace tabs, toggle theme, fullscreen. All framework, no Urd.

**Phase 2** adds the core infrastructure: command registry, message bus, selection and filter contexts, navigation broker, project management, file system abstraction, compiler service with Tauri IPC, chunked output cache, projection registry, and the recompile-on-change pipeline. The milestone: open a project, compiler runs, output flows through chunks to projections to bus signals, placeholder views react.

**Phase 3** builds the code editor: singleton zone with tab management, Urd language mode, inline diagnostics, go-to-definition, hover tooltips, autocomplete, breadcrumb bar, source viewer. Much of this work is a transplant from the playground — the cursor resolver, tooltip builder, and CodeMirror integration already exist.

**Phases 4–7** build the views in tiers: spreadsheets and inspectors first (the data already exists in projections), then graphs (shared pan/zoom canvas), then runtime integration (Wyrd), then advanced analysis and remaining views.

**Phase 8** is polish: export commands, annotation layer, performance profiling against stress fixtures (50,000 entities, 10,000 diagnostics), and the architecture litmus test — typing at 10 keystrokes per second while scrolling a 50k-row spreadsheet and panning a 200-node graph. Responsive throughout, or the architecture has a problem.

## Why this matters

The playground proved that the compiler's semantic model — the FactSet, the PropertyDependencyIndex, the DefinitionIndex — can power real editor features. But a playground is one pane showing one file. The compiler produces a queryable graph of every entity, every property, every transition, every rule in a world. That graph deserves more than one view at a time.

Urd Forge is the interface that matches the compiler's ambition. Eighty-five views projected over a single compiled world, each one a different question answered by the same data. The layout engine lets the writer arrange those questions to fit their thinking. The projection layer ensures they always agree. The command system ensures every action is undoable, searchable, and scriptable.

The specification is complete. The architecture document covers every component, every interface, every data flow, every edge case from singleton zone lifecycle across workspaces to what happens when the Rust backend panics mid-compile. Implementation begins with Phase 1: the framework shell.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
