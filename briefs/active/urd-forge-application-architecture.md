# Urd Forge — Application Architecture

**Version**: 0.1.0 (Architectural Foundation)
**Date**: February 2026
**Status**: Design Phase

> **Document status: BRIEF** — Full architecture specification for the Urd Forge desktop IDE. Covers BSP tiling layout engine, 85+ specialised views, command registry, message bus, projection layer, workspace management, theme engine, and eight-phase implementation plan.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-26 (Phase 1)
**Status:** In Progress — Phase 1 complete, Phases 2–8 pending

### What was done

**Phase 1: Framework Shell** — completed 2026-02-26 across 7 commits, followed by 10 bug-fix and enhancement commits.

Scaffolded the Tauri 2 + Svelte 5 + Vite application at `packages/forge/` and built the full framework layer:

- **Message bus** (`MessageBus.ts`) — channel-registered pub/sub with retainLast replay, 4KB dev-mode payload assertion. `ChannelManifest.ts` registers 14 framework channels. `TestBus` test helper records publishes.
- **Command registry** (`CommandRegistry.ts`) — keybinding resolution with normalisation, execution logging to bus, undo action return type. `KeybindingManager.ts` installs keyboard sovereignty suppressing browser shortcuts (Ctrl+P, F5, etc.). `TestCommandRegistry` test helper.
- **Theme engine** — `tokens.css` defines Gloaming (dark) and Parchment (light) theme tokens across surface, text, border, accent, semantic, graph, spreadsheet, and focus categories. Semantic typography and spacing tokens. `ThemeEngine.ts` manages `data-theme` attribute switching. `base.css` handles reset, user-select suppression, overscroll prevention, focus-visible styles.
- **App settings service** (`AppSettingsService.ts`) — corruption-safe persistence with debounced 500ms writes. Loads from OS config dir via Tauri path API with proper `path.join()` for path construction. Backs up corrupt files as `settings.backup.json` and falls back to defaults. Publishes `settings.changed` on bus. `MemorySettingsIO` for tests and browser dev mode (auto-detected via `__TAURI_INTERNALS__`).
- **View registry** (`ViewRegistry.ts`) — register/list/lazy-load with category grouping, singleton tracking, project-aware filtering, state versioning, and migration support. `ViewHostContract.ts` for toolbar and status contributions.
- **BSP layout engine** — `ZoneTree.ts` implements immutable tree reducer with 7 invariant assertions enforced after every mutation. Actions: split, join, swap, resize, changeType, resetDivider. `ZoneStateStore.ts` persists versioned zone state keyed by `zone:${id}::type:${typeId}` with migration on version mismatch.
- **Layout components** — `ZoneRenderer.svelte` (recursive BSP renderer), `SplitContainer.svelte` (with `data-split-id` for DOM lookup), `Divider.svelte` (drag resize against parent `.forge-split` container, right-click context menu, double-click reset, keyboard arrows + Enter), `ZoneShell.svelte` (header + viewport + error boundary), `ZoneHeader.svelte` (type selector dropdown + split buttons), `ZoneErrorBoundary.svelte`, `ZoneLoadingState.svelte`.
- **Context menu system** — `ContextMenu.svelte` positioned at cursor with support for both command dispatch and direct action callbacks. `ContextMenuProvider.ts` registry, `contextMenuSuppressor.ts` for global right-click interception. Framework-level menus for zones and dividers.
- **Global menu bar** — `GlobalMenuBar.svelte` with File/Edit/View/Window/Help. `MenuRegistry.ts` contribution-based population with group separators. `MenuDropdown.svelte` with hover-to-switch-when-open behaviour.
- **Workspace manager** (`WorkspaceManager.svelte.ts`) — multiple workspaces with tabs, dispatch-through-reducer tree mutations, serialisation/deserialisation with corruption-safe loading. `WorkspaceTabs.svelte` with rename-on-double-click, close, and add.
- **Project manager** (`ProjectManager.svelte.ts`) — open/close/recent with Tauri directory picker. `WelcomeScreen.svelte` replaces workspace when no project is open. Recent projects list with remove. Falls back to mock project in browser dev mode.
- **Placeholder views** — `PlaceholderColour` (deterministic hue from zone ID hash), `PlaceholderInfo` (zone metadata with live ResizeObserver dimensions), `PlaceholderBusMonitor` (live bus event log), `PlaceholderCommandLog` (live command execution log). All registered under "Debug" category.
- **Global status bar** (`GlobalStatusBar.svelte`) — compiler status, theme toggle, version display.
- **Bootstrap** (`bootstrap.ts`) — wires all systems: registers channels, loads settings, initialises theme, registers 4 placeholder views, registers 12 commands (fullscreen, theme toggle, open/close project, quit, command palette stub, zone split/join/swap/reset), registers menu contributions, installs keybinding manager, installs global error handlers. Tauri API calls guarded with `__TAURI_INTERNALS__` check.
- **App.svelte** — implements the architecture doc §5.5 hierarchy: `GlobalMenuBar` → `WorkspaceTabs` → `ZoneRenderer`/`WelcomeScreen` → `GlobalStatusBar`. Includes Blender-style split positioning mode (crosshair cursor, divider follows mouse until click confirms, Escape cancels).
- **Tauri backend** — `tauri-plugin-dialog` and `tauri-plugin-fs` registered alongside `tauri-plugin-shell`. Capabilities grant window close/fullscreen, dialog, and scoped app-config filesystem permissions.

**46 files, ~4600 lines of code.** Both `vite build` (frontend) and `cargo check` (Rust backend) pass clean. Tested in both browser dev mode and Tauri native mode.

### Acceptance criteria verification

| Criterion | Status | Evidence |
|-----------|--------|---------|
| **Phase 1** — Framework shell acceptance test passes (split/join/swap/resize/theme/workspace tabs/fullscreen) | Tested | All 15 steps implemented. `vite build` and `cargo check` pass. Manually tested in both browser dev mode and Tauri native — split, join, swap, resize, theme toggle, workspace tabs, fullscreen, quit all working. |
| **Phase 2** — Compiler output flows through chunks → projections → bus signals → placeholder views react | Not started | |
| **Phase 3** — Full editing experience with compiler feedback in Code Editor singleton | Not started | |
| **Phase 4** — Writer and Engineer workspaces fully functional with Tier 1 views | Not started | |
| **Phase 5** — World Builder workspace fully functional with graph views | Not started | |
| **Phase 6** — Debug workspace fully functional with runtime integration | Not started | |
| **Phase 7** — All 85+ views implemented | Not started | |
| **Phase 8** — Architecture litmus test passes (50k entities, 10 keystrokes/s, 60fps scrolling) | Not started | |

### What changed from the brief

- **`<svelte:self>` → self-import**: Svelte 5 deprecates `<svelte:self>` in favour of importing the component by name. `ZoneRenderer.svelte` uses `import ZoneRenderer from './ZoneRenderer.svelte'` for recursion.
- **`<svelte:component>` → direct dynamic component**: Svelte 5 runes mode deprecates `<svelte:component this={...}>`. `ZoneShell.svelte` uses `{@const ViewComponent = loadedComponent}` + `<ViewComponent>` instead.
- **Splash screen deferred**: Step 3 (Tauri splashscreen plugin) was deferred — the main window shows a loading state directly. The splashscreen window config can be added once the visual acceptance test confirms the base shell works.
- **`@tauri-apps/plugin-dialog` and `@tauri-apps/plugin-fs` added as dependencies**: Required for the directory picker (WelcomeScreen, Open Project command) and settings persistence. Both npm packages and Rust crates registered with Tauri capabilities.
- **WorkspaceManager/ProjectManager use `.svelte.ts` extension**: Files using `$state`/`$derived` runes outside `.svelte` components must use the `.svelte.ts` extension per Svelte 5 rules. The architecture doc used plain `.ts`.
- **Tauri API calls guarded with `__TAURI_INTERNALS__`**: Tauri npm packages import successfully in browser dev mode but IPC calls fail silently or break the page. All Tauri-specific calls (window close, fullscreen, settings persistence) check for the Tauri runtime before proceeding, with browser fallbacks where applicable.
- **Directional split/join labels**: "Split Horizontal/Vertical" replaced with "Split Left / Right" and "Split Top / Bottom" for clarity. Join labels use "Keep Left/Right" or "Keep Top/Bottom" based on the actual split direction.
- **Blender-style split positioning**: After splitting a zone, the divider immediately follows the mouse cursor. Click confirms the position, Escape cancels (undoes the split). Not in the original architecture doc — added for UX quality.
- **SplitContainer uses direct child selectors**: Svelte scoped CSS uses the same hash for all instances of the same component. Descendant selectors leaked across nested splits; changed to `>` child selectors to prevent inner dividers inheriting outer dimension constraints.

---

## 1. Vision

Urd Forge is a Tauri-based desktop IDE for authoring Urd interactive fiction worlds. It provides Houdini-grade information density through a Blender-style tiling layout engine, with 85+ specialised views projected over a single compiler output. Every view — spreadsheets, node graphs, matrices, inspectors, visualisations — is a reactive lens into the same underlying data.

The application is designed with architectural separation such that the layout engine, zone system, command registry, message bus, and workspace management form a **generic IDE framework** layer, while all Urd-specific views, compiler integration, and domain logic form an **application** layer. These live in the same codebase but maintain a strict import direction: application imports framework, never the reverse. This separation is annotated throughout, not enforced by package boundaries.

> **Note for future maintainers**: Every file, module, and type in the `framework/` directory tree is domain-agnostic. If you want to extract the framework for a different IDE, start there. The `app/` directory tree contains everything Urd-specific. The seam is the import graph.

---

## 2. Technology Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Shell | **Tauri 2.x** | Native performance, single webview, Rust backend, cross-platform |
| Backend | **Rust** | Urd compiler already in Rust, heavy computation stays here |
| Frontend | **Svelte 5** (runes mode) | Compiler-optimised reactivity, minimal runtime, surgical DOM updates |
| Code Editor | **CodeMirror 6** | Embeddable, extensible, excellent performance in constrained containers |
| Styling | **CSS custom properties** | Theme-switchable, zero-runtime, framework-level token system |
| State | **Svelte stores + message bus** | No external state library; framework-level bus for cross-zone communication |
| Build | **Vite** | Fast HMR for Svelte, Tauri integration via `@tauri-apps/cli` |
| Package Manager | **pnpm** | Workspace support if needed, fast, disk-efficient |

### Deployment Variants

| Variant | Compiler | File System | Notes |
|---------|----------|-------------|-------|
| **Desktop (primary)** | Rust via Tauri IPC | Real filesystem via Tauri FS API | Default target, full capability |
| **Web (future)** | WASM via Web Worker pool | IndexedDB virtual filesystem | Reduced capability, no file watching |

Both variants consume an identical `CompilerService` and `FileSystem` interface. Views never import an implementation directly.

---

## 3. Architectural Principles

1. **The IDE is projections.** Every view renders a transformation of a single `CompilerOutput`. There is one source of truth.
2. **Rust does the heavy lifting.** Compilation, analysis, simulation, graph traversal, and any operation over large datasets runs in the Rust backend. The frontend renders results.
3. **Everything is a command.** Every user-triggerable action is a registered command with a unique ID, typed parameters, and a return type. Keybindings, menus, toolbars, the command palette, and future scripting all dispatch through the command registry.
4. **One bus, typed channels.** Cross-zone communication flows through a single message bus with well-known channel IDs. Views subscribe to the channels they care about.
5. **Editor is king.** The Code Editor zone is architecturally privileged: it is a singleton, hosts the dockable tab universe, and is the default navigation target for all cross-view "go to source" actions.
6. **Framework ← Application.** The generic framework layer has no knowledge of Urd. Domain-specific code flows in one direction only.
7. **Native application, not a website.** Urd Forge must feel indistinguishable from a native desktop application. No browser chrome leaks, no uncontrolled text selection, no default context menus, full keyboard sovereignty.

---

## 4. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Urd Forge (Tauri)                        │
├─────────────────────────────────────────────────────────────────┤
│  RUST BACKEND                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐    │
│  │ Urd Compiler  │  │ Analysis     │  │ File System &      │    │
│  │ (parse/link/  │  │ Engine       │  │ File Watcher       │    │
│  │  analyze/emit)│  │ (monte carlo,│  │ (notify crate)     │    │
│  │              │  │  reachability,│  │                    │    │
│  │              │  │  dead code)  │  │                    │    │
│  └──────┬───────┘  └──────┬───────┘  └────────┬───────────┘    │
│         │                 │                    │                │
│  ───────┴─────────────────┴────────────────────┴────────────── │
│                     Tauri IPC (commands + events)               │
├─────────────────────────────────────────────────────────────────┤
│  SVELTE FRONTEND                                                │
│                                                                 │
│  ┌─ FRAMEWORK LAYER ──────────────────────────────────────────┐ │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌───────────────┐   │ │
│  │  │ Layout   │ │ Command  │ │ Message│ │ View          │   │ │
│  │  │ Engine   │ │ Registry │ │ Bus    │ │ Registry      │   │ │
│  │  │ (BSP)    │ │          │ │        │ │               │   │ │
│  │  └──────────┘ └──────────┘ └────────┘ └───────────────┘   │ │
│  │  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌───────────────┐   │ │
│  │  │ Workspace│ │ Theme    │ │ Nav    │ │ Selection     │   │ │
│  │  │ Manager  │ │ Engine   │ │ Broker │ │ Context       │   │ │
│  │  └──────────┘ └──────────┘ └────────┘ └───────────────┘   │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌─ APPLICATION LAYER (Urd) ──────────────────────────────────┐ │
│  │  ┌──────────────┐ ┌────────────────┐ ┌──────────────────┐  │ │
│  │  │ Compiler     │ │ View           │ │ Urd Commands     │  │ │
│  │  │ Service      │ │ Implementations│ │ & Keybindings    │  │ │
│  │  │ (IPC bridge) │ │ (85+ views)    │ │                  │  │ │
│  │  └──────────────┘ └────────────────┘ └──────────────────┘  │ │
│  │  ┌──────────────┐ ┌────────────────┐ ┌──────────────────┐  │ │
│  │  │ Predefined   │ │ CodeMirror     │ │ Data Providers   │  │ │
│  │  │ Workspaces   │ │ Urd Language   │ │ (FactSet, AST,   │  │ │
│  │  │              │ │ Mode           │ │  SymbolTable)    │  │ │
│  │  └──────────────┘ └────────────────┘ └──────────────────┘  │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 4.1 Data Flow — The Truth Map

This is the single path all data takes from keystroke to pixel. There are no shortcuts.

```
User types in Code Editor
       │
       ▼
   BufferMap            (in-memory file buffers — source of truth for content)
       │
       ▼  (300ms debounce)
   Compiler             (Rust, via Tauri IPC — parse/link/analyze/emit)
       │
       ▼
   Chunked Output       (ast, symbolTable, factSet, etc. — each with content hash)
       │
       ▼
   CompilerOutputCache  (reuses unchanged chunks by hash — avoids redundant parse/GC)
       │
       ├──────────────────────────────┐
       ▼                              ▼
   ProjectionRegistry            Bus signal
   (memoised selectors,          (compiler.completed:
    lazy recompute on            { compileId, hashes })
    hash change)                      │
       │                              ▼
       ▼                         Views wake up,
   Projections                   call registry.get()
   (entityTable,                      │
    locationGraph,                    │
    diagnosticsByFile, ...)           │
       │                              │
       └──────────┬───────────────────┘
                  ▼
              View renders
              (Svelte 5 fine-grained reactivity)
```

**Rules enforced by this flow:**
- Views never access `CompilerOutput` directly — only through projections
- The bus never carries data payloads — only signals that trigger re-evaluation
- Projections are the sole data authority — if two views disagree, the projection is wrong, not the views
- `BufferMap` is the source of truth for file content, not the filesystem

---

## 5. Layout Engine (BSP Zone Tree)

### 5.1 Data Model

The layout is a recursive binary space partition. Each node is either a split (with two children) or a leaf (a zone).

```typescript
// FRAMEWORK

type ZoneTree = SplitNode | LeafNode;

interface SplitNode {
  kind: 'split';
  id: string;
  direction: 'horizontal' | 'vertical';
  ratio: number;           // 0.0–1.0, position of the divider
  children: [ZoneTree, ZoneTree];
}

interface LeafNode {
  kind: 'leaf';
  id: string;              // unique zone ID (e.g., "zone_a3f2")
  zoneTypeId: string;      // registry key (e.g., "urd.codeEditor", "urd.entitySpreadsheet")
  singletonRef?: true;     // if true, this is a reference leaf — mounts shared singleton instance (see §11.4)
}
```

**Tree invariants (enforced after every mutation):**

1. All leaf node IDs are unique across the entire tree.
2. All split node IDs are unique across the entire tree.
3. A split node always has exactly two children. No split has identical child references.
4. The root is always valid — either a single leaf or a split with two children.
5. No orphan nodes exist after a merge (the purged zone's state is cleaned up).
6. Singleton constraint: for any view type with `navigationStrategy: 'singleton'` or `'singleton-autocreate'`, at most one leaf in the tree has that `zoneTypeId`.
7. After a **join**, if `FocusService.activeZoneId` pointed at the absorbed zone, focus transfers to the surviving zone. If `SelectionContext` references an item scoped to the absorbed zone, selection is cleared.

**Swap semantics:** Swap exchanges the `zoneTypeId` and zone state between two adjacent leaf nodes. **Zone IDs do not move.** This is critical — zone IDs are persistence keys and focus references. Swap is equivalent to: read both leaves' types and states, write each to the other leaf, leave IDs untouched.

**All mutations go through a single reducer function:**

```typescript
// FRAMEWORK

type ZoneTreeAction =
  | { type: 'split'; zoneId: string; direction: 'horizontal' | 'vertical' }
  | { type: 'join'; dividerId: string; keep: 'first' | 'second' }
  | { type: 'swap'; dividerId: string }
  | { type: 'resize'; dividerId: string; ratio: number }
  | { type: 'changeType'; zoneId: string; newTypeId: string }
  | { type: 'resetDivider'; dividerId: string };

function zoneTreeReducer(tree: ZoneTree, action: ZoneTreeAction): ZoneTree {
  const newTree = applyAction(tree, action);
  assertInvariants(newTree);  // throws in dev if any invariant is violated
  return newTree;
}
```

No code outside the reducer may mutate the tree. Zone commands dispatch actions through the reducer. This guarantees that layout bugs are caught immediately at the mutation site, not downstream as ghost zones or broken renders.

### 5.2 Zone State Persistence

Each zone's internal state is stored in a map keyed by `zoneId + zoneTypeId`. When a user changes a zone's type via the header dropdown and later switches back, the state is restored. State is versioned to prevent bricked workspaces as views evolve.

**`ZoneStateStore` is the single authority for all zone state.** The `Workspace` interface holds a reference to it directly. Workspace serialization stores the backing map verbatim — there is no second state model. The key format is explicit and stable: `zone:${zoneId}::type:${zoneTypeId}`.

```typescript
// FRAMEWORK

interface PersistedZoneState {
  stateVersion: number;          // matches ViewRegistration.stateVersion at time of save
  data: unknown;                 // the view's serialized state
}

interface ZoneStateStore {
  // Returns the persisted state, running migration if stateVersion is stale.
  // If migration fails or no state exists, returns the view's defaultState.
  get(zoneId: string, zoneTypeId: string): unknown;

  // Persists zone state with current stateVersion. Called by the zone component on state changes.
  set(zoneId: string, zoneTypeId: string, state: unknown): void;

  // Clears all state for a zone when it is destroyed (via merge).
  purge(zoneId: string): void;

  // Serialize the entire store for workspace persistence.
  serialize(): Record<string, PersistedZoneState>;

  // Restore from serialized data, running migrations as needed.
  deserialize(data: Record<string, PersistedZoneState>): void;
}
```

**On workspace load**, the store compares each entry's `stateVersion` against the view's current `stateVersion`. If stale, it calls `ViewRegistration.migrateState()`. If migration returns `undefined` or the view has no migration function, the entry is replaced with `defaultState`.

Memory concern: with 200+ view types, the theoretical max is `zoneCount × 200` state entries. In practice, a zone will have accumulated state for 2–5 types at most. Purge on zone destruction keeps this bounded.

**Views with per-resource state:** The ZoneStateStore key is `zoneId + zoneTypeId`, which means there is one state blob per zone per view type. Views that manage multiple resources (e.g., Code Editor with multiple open tabs) store a nested map inside their state blob, keyed by resource ID (file path). For example, the Code Editor's state is `{ openTabs: [{ path, cursorPos, scrollTop, selections }], activeTab: string }`. The ZoneStateStore doesn't know about this nesting — it just persists the blob. This keeps the framework generic while allowing views to track per-document state.

### 5.3 Zone Operations

All zone operations are commands:

| Command | Action |
|---------|--------|
| `forge.zone.splitHorizontal` | Split the active zone horizontally |
| `forge.zone.splitVertical` | Split the active zone vertically |
| `forge.zone.joinLeft` | Merge: right sibling absorbed by left |
| `forge.zone.joinRight` | Merge: left sibling absorbed by right |
| `forge.zone.joinUp` | Merge: bottom sibling absorbed by top |
| `forge.zone.joinDown` | Merge: top sibling absorbed by bottom |
| `forge.zone.swap` | Swap two sibling zones |
| `forge.zone.resetDivider` | Reset the focused divider's split ratio to 0.5 |
| `forge.zone.changeType` | Change the active zone's view type |
| `forge.zone.maximize` | Temporarily expand one zone to fill the workspace |
| `forge.zone.restore` | Restore from maximized state |

### 5.4 Global Status Bar

A thin, persistent status bar is always visible at the very bottom of the application window, below the BSP zone tree. It is not part of any zone — it is a fixed, framework-level element like Blender's status bar.

Contents (left to right):
- **Compiler status**: idle / compiling / error count / warning count
- **Active file**: path of the file in the focused editor tab
- **Selection hint**: brief description of the currently selected item (e.g., "Entity: @warden (Guard)")
- **World stats summary**: files, entities, locations, sections (compact)
- **Theme toggle** (icon button)
- **Urd Forge version**

The status bar also provides split operation buttons when no zones exist yet (fresh workspace), serving as the entry point for building a layout from scratch. Once zones exist, splitting is done via zone header controls or keyboard commands.

Individual zones do **not** have their own status bars. Each zone has a header bar (with the type selector and zone-specific toolbar), but the bottom of each zone is purely viewport.

### 5.5 Svelte Component Hierarchy

```
<App>
  <GlobalMenuBar />                     // Top: File, Edit, View, Window, Help menus
  <WorkspaceTabs />                     // Below menu: workspace tab bar (Blender-style)
  <Workspace>                           // Owns the ZoneTree for the active workspace tab
    <ZoneRenderer tree={node}>          // Recursive: renders SplitNode or LeafNode
      <SplitContainer>                  // Renders two children with a draggable divider
        <ZoneRenderer .../>
        <ZoneRenderer .../>
      </SplitContainer>
      — or —
      <ZoneShell zoneId={id}>           // Renders the header bar + view container
        <ZoneHeader>                    // Type selector dropdown, zone-specific toolbar
        <ZoneViewport>                  // The actual view component (dynamically loaded)
          <svelte:component this={viewComponent} {zoneState} />
        </ZoneViewport>
      </ZoneShell>
    </ZoneRenderer>
  </Workspace>
  <GlobalStatusBar />                   // Bottom: always visible, never part of any zone
</App>
```

### 5.6 Divider Interactions

The divider between two sibling zones is an interactive element with multiple behaviours, modelled on Blender's area borders.

**Drag (left-click hold + move):** Resizes the two sibling zones by adjusting the split `ratio`. Clamped so neither zone shrinks below a minimum useful size (e.g., 80px). If dragged past the minimum threshold, triggers a merge — the zone being shrunk is absorbed by its sibling (see merge behaviour below).

**Right-click context menu:** Opens a menu anchored to the divider with the following options:

| Menu Item | Action | Notes |
|-----------|--------|-------|
| **Join Left / Join Up** | Merge: the right (or bottom) zone is absorbed by the left (or top) zone | The absorbing zone keeps its view; the absorbed zone's state is purged |
| **Join Right / Join Down** | Merge: the left (or top) zone is absorbed by the right (or bottom) zone | Opposite direction |
| **Swap** | Swap the two sibling zones (view types, state, and all) without changing the split geometry | Useful when the user built the right layout but put views on the wrong sides |
| **Split Horizontal** | Replace the divider's parent split with a deeper tree, adding a new zone | Convenience shortcut — equivalent to splitting one of the children |
| **Split Vertical** | Same as above, perpendicular direction | |

Menu labels are directional and context-aware: for a vertical divider (left|right split), the options read "Join Left" / "Join Right". For a horizontal divider (top/bottom split), they read "Join Up" / "Join Down". A small arrow icon in the menu item indicates the direction of absorption.

**Merge behaviour details:**

When zone A absorbs zone B:
1. Zone B's state is purged from the `ZoneStateStore`.
2. Zone A expands to fill the space previously occupied by both zones.
3. If zone B was the Code Editor singleton, the singleton lock is released. Zone A does *not* inherit the Code Editor type — it keeps its own type. However, to prevent a state where navigation intents have no target (since the editor is king and the default navigation destination), the navigation broker enters a **queuing mode**: intents are buffered and a subtle, persistent toast appears in the status bar saying "No Code Editor — click to restore." Clicking the toast auto-splits the largest zone and creates a new Code Editor, then replays any queued intents. This avoids the confusion of auto-recreation (which would feel like the UI fighting the user) while keeping the navigation contract intact.
4. The parent `SplitNode` in the BSP tree is replaced by zone A's `LeafNode`.

**Drag-to-merge (Blender-style):** If the user drags the divider far enough that one zone would collapse below minimum size, the shrinking zone is merged into the growing one. A visual overlay (arrow indicator) appears during the drag to preview which zone will be absorbed and in which direction, giving the user a chance to release or continue.

**Double-click:** Resets the split ratio to 0.5 (equal sizing). A small convenience.

**Keyboard:** When a divider is focused (via Tab navigation), arrow keys adjust the ratio in small increments, and Enter opens the context menu.

All divider actions are implemented as commands:

| Command | Action |
|---------|--------|
| `forge.zone.joinLeft` | Merge: right zone absorbed by left |
| `forge.zone.joinRight` | Merge: left zone absorbed by right |
| `forge.zone.joinUp` | Merge: bottom zone absorbed by top |
| `forge.zone.joinDown` | Merge: top zone absorbed by bottom |
| `forge.zone.swap` | Swap two sibling zones |
| `forge.zone.resetDivider` | Reset split ratio to 0.5 |

### 5.7 Global Menu Bar

A horizontal menu bar sits at the very top of the application window, above the workspace tabs. This mirrors Blender's top bar, which hosts the application menus to the left of the workspace tabs and global scene/render selectors to the right.

**Layout (left to right):**

```
[File ▾] [Edit ▾] [View ▾] [Window ▾] [Help ▾]    ──── [WorkspaceTabs] ────    [scene/project selector]
```

The menu bar and workspace tabs share the same horizontal row to maximise vertical space. Menus sit left-aligned; workspace tabs sit centre or right; the project selector (if applicable) sits far right.

**Menu structure:**

| Menu | Contents |
|------|----------|
| **File** | New Project, Open Project, Open Recent ▸, Save, Save As, Import, Export ▸, Preferences, Quit |
| **Edit** | Undo, Redo, Command Palette (Ctrl+Shift+P), Find (delegates to Global Symbol Search or Regex Search depending on context) |
| **View** | Toggle Fullscreen (F11), Toggle Theme, Reset Zoom, Zoom In/Out, Toggle Global Status Bar visibility, Toggle Zone Headers visibility |
| **Window** | New Workspace, Duplicate Workspace, workspace templates submenu (Writer, Engineer, World Builder, QA, Debug), Reset to Default Layout |
| **Help** | Urd Language Reference, Keyboard Shortcuts, About Urd Forge |

Each menu item maps to a registered command. The menu bar is a framework-level component, but its contents are populated from the command registry using category tags — so Urd-specific commands (like "Compile" or "Run Playback") can appear in menus without the framework knowing about them.

**Menu extensibility pattern:**

```typescript
// FRAMEWORK
interface MenuContribution {
  menu: 'file' | 'edit' | 'view' | 'window' | 'help';
  group: string;          // for separator grouping (e.g., "save", "export", "project")
  order: number;          // sort order within group
  commandId: string;      // must be a registered command
  label?: string;         // override command title for menu display
  submenu?: MenuContribution[];
}

// APPLICATION (Urd registers additional menu items)
menuRegistry.contribute({
  menu: 'file',
  group: 'compile',
  order: 50,
  commandId: 'urd.compiler.compile',
  label: 'Recompile World',
});
```

This lets the framework own the menu bar shell and rendering, while the application contributes entries. The same pattern supports future plugins adding their own menu items.

### 5.8 Placeholder Views (Phase 1 Test Harness)

Before any real views are built, the layout engine needs to be validated with placeholder components. The framework ships with a small set of test views that exercise key layout behaviours.

**Included placeholders:**

| Placeholder | Purpose |
|-------------|---------|
| `forge.placeholder.info` | Displays the zone's own metadata: zone ID, zone type, zone dimensions (px), split depth. Updates reactively on resize. Confirms that zones know their own geometry. |
| `forge.placeholder.colour` | Renders a solid background colour assigned by zone ID hash. Makes it immediately obvious which zone is which during split/merge/swap testing. |
| `forge.placeholder.busMonitor` | Displays a live log of all message bus events. Confirms that the bus is working and that views can subscribe to channels. Essential for Phase 2 integration testing. |
| `forge.placeholder.commandLog` | Displays a log of all commands executed, with timestamps and arguments. Confirms the command registry is wired correctly. |
| `forge.placeholder.projectionInspector` | Displays all registered projections with their dependency chunk names, current hashes, recompute counts, and last computation time (ms). Highlights projections that exceed 16ms. Essential for identifying projections that should be promoted from `'sync'` to `'worker'` mode. |

These placeholders are registered in the view registry under the category "Debug" and remain available in production (hidden behind a "Show Debug Views" preference toggle). They are invaluable for diagnosing layout issues, bus connectivity, and command routing at any stage of development.

**Phase 1 acceptance test:**
1. Launch the app. A single zone fills the workspace showing `forge.placeholder.colour`.
2. Right-click the global status bar → split horizontal. Two colour zones appear with distinct colours.
3. Drag the divider. Both zones resize smoothly. The `forge.placeholder.info` view (if active) updates dimensions in real-time.
4. Right-click the divider → Swap. The two colours switch sides.
5. Right-click the divider → Join Left. One zone absorbs the other.
6. Change a zone's type via the header dropdown. The view switches. Change it back — the previous state is restored.
7. Switch workspace tabs. The entire layout swaps. Switch back — everything is preserved.
8. Toggle the theme. All placeholder views, the menu bar, workspace tabs, status bar, and zone headers all change appearance.

If all eight steps pass, the framework shell is proven.

### 5.9 Zone Error Boundary

With 85+ lazily loaded views, a throw inside one view must not crash the entire application. Every zone viewport is wrapped in a `ZoneErrorBoundary` that catches uncaught exceptions and render errors.

```typescript
// FRAMEWORK

/**
 * When a view component throws during render or in an effect:
 * 1. The error boundary catches it.
 * 2. The viewport is replaced with a ZoneErrorPanel showing:
 *    - The view type name
 *    - A truncated error message (not a full stack trace — that goes to console)
 *    - A "Reload View" button that destroys and recreates the component with defaultState
 *    - A "Switch View" dropdown to change to a different zone type
 * 3. The error is logged to a debug bus channel (zone.error) for PlaceholderBusMonitor.
 * 4. All other zones continue operating normally.
 */
```

The error boundary wraps the `<svelte:component>` inside `ZoneShell`. It does NOT wrap the zone header or the zone shell itself — framework chrome must remain functional so the user can switch away from a broken view.

This is Phase 1 infrastructure — it must exist before any real views are loaded.

### 5.10 Global Error Handling

Zone error boundaries catch view-level failures. Three other failure classes need explicit handling:

**Rust backend panic.** If the Rust compiler panics mid-compile (e.g., malformed `.urd.md` crashes the parser), the Tauri IPC call rejects. The compile pipeline catches this, publishes `system.error` on the bus with `{ source: 'compiler', message, compileId }`, and the status bar shows a persistent error banner: "Compiler crashed — check your source files." The last valid `CompilerOutput` remains in the projection cache. Views continue showing stale-but-valid data. The next edit triggers a fresh compile attempt.

**IPC transport failure.** If the Tauri IPC channel itself fails (e.g., backend process died), all pending IPC calls reject. The app shell catches this at the top level and shows a blocking modal: "Backend disconnected — restart the app." There is no recovery path — this is a fatal error.

**Unhandled frontend exceptions.** A global `window.onerror` / `onunhandledrejection` handler logs to the `system.error` bus channel and to the developer console. If the exception originated outside a zone error boundary (e.g., in framework code), the status bar shows a warning. The app does not crash — zones continue operating.

```typescript
// FRAMEWORK — registered bus channels for error handling
bus.registerChannel({ id: 'system.error', domain: 'system', retainLast: false });
```

All errors flow through the bus so the PlaceholderBusMonitor (and future logging infrastructure) can capture them in one place.

---

## 6. The Code Editor Zone (Singleton)

The Code Editor is the only zone type that hosts an internal tab bar and dockable document management. It is architecturally special.

### 6.1 Singleton Enforcement

The view registry marks the Code Editor zone type as `navigationStrategy: 'singleton-autocreate'`. The zone type selector dropdown disables it if it's already instantiated somewhere in the tree. If the user attempts to change a zone to Code Editor when one already exists, the system focuses the existing one instead.

### 6.2 Internal Structure

```
┌─ Code Editor Zone ──────────────────────────────────────────┐
│ [Zone Header: "Code Editor" dropdown | zone toolbar]        │
│ ┌─ Tab Bar ───────────────────────────────────────────────┐ │
│ │ [world.urd.md ×] [gatehouse.urd.md ×] [rules.urd.md ×] │ │
│ ├─────────────────────────────────────────────────────────┤ │
│ │ Breadcrumb: world.urd.md > # The Gatehouse > == greet   │ │
│ ├─────────────────────────────────────────────────────────┤ │
│ │                                                         │ │
│ │  CodeMirror 6 Instance                                  │ │
│ │  (with code lens, gutter icons, inline annotations,     │ │
│ │   scope highlighting, diagnostics, hover tooltips)      │ │
│ │                                                         │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 6.3 Tab Management

Tabs are ordered by user interaction. Tab operations are commands:

| Command | Action |
|---------|--------|
| `forge.editor.openFile` | Open a file in a new tab (or activate existing) |
| `forge.editor.closeTab` | Close the active tab |
| `forge.editor.closeOtherTabs` | Close all tabs except the active one |
| `forge.editor.nextTab` | Switch to the next tab |
| `forge.editor.prevTab` | Switch to the previous tab |
| `forge.editor.reorderTab` | Reorder via drag within the tab bar |

Cross-zone tab dragging is **not supported**. Tabs live within the Code Editor zone exclusively. See Section 7 for how other zones open files in the editor.

### 6.4 Source Viewer Zone (Second-Class Editor)

A separate zone type, `forge.sourceViewer`, provides a lightweight read-only (or light-edit) CodeMirror instance displaying a single buffer. No tabs, no breadcrumbs, no code lens. Syntax highlighting and basic navigation only. Not a singleton — multiple Source Viewer zones can coexist.

Used for: side-by-side reference viewing, pinned file display, split-view comparison.

---

## 7. Navigation Broker

The navigation broker resolves cross-view "go to" requests. Any zone can emit a navigation intent; the broker determines which zone receives it.

### 7.1 Navigation Intent

```typescript
// FRAMEWORK (generic structure)

interface NavigationIntent {
  targetZoneType: string;         // e.g., "urd.codeEditor"
  targetResource: string;         // e.g., a file path or buffer ID
  targetLocation?: {              // optional sub-location
    line?: number;
    column?: number;
    span?: [number, number];
  };
  sourceZoneId: string;           // which zone emitted this
  forceNewTarget?: boolean;       // modifier key held: prefer a different zone
}
```

### 7.2 Resolution Chain (for Code Editor targets)

Since the Code Editor is a singleton, the resolution is simplified:

1. **Is the Code Editor instantiated?**
   - Yes → proceed to step 2.
   - No → auto-split the largest zone and create a Code Editor (per its `'singleton-autocreate'` strategy).

2. **Is the target file already the active tab?**
   - Yes → jump to location. Done.

3. **Is the target file open in a background tab?**
   - Yes → activate that tab, jump to location. Done.

4. **Open a new tab** with the target file, jump to location.

5. **Focus the Code Editor zone** (scroll it into view if necessary, flash the zone border briefly).

The `forceNewTarget` flag (triggered by holding `Ctrl`/`Cmd` during a click) overrides the singleton: it opens the file in a Source Viewer zone instead, creating one via split if needed. This is the explicit escape hatch for "I want to see this side-by-side without losing my current editor context."

### 7.3 Resolution for Non-Editor Targets

For navigation intents targeting other zone types (e.g., "show this entity in the Entity Inspector"), the broker uses the target view's `navigationStrategy`:

1. **`'singleton-autocreate'`** → Is there an instance? Route to it. No instance? Auto-split the largest zone and create one.
2. **`'singleton'`** → Is there an instance? Route to it. No instance? Ignore the intent (log a warning in dev).
3. **`'multi'`** → Is there a matching zone? Route to the first one found. No match? Auto-split and create one.

---

## 8. Message Bus

A single, typed, publish-subscribe message bus. Framework-level infrastructure, with Urd-specific channels defined in the application layer.

### 8.1 Bus Interface

```typescript
// FRAMEWORK

interface ChannelDefinition<T> {
  id: string;
  domain: string;              // e.g., 'compiler', 'selection', 'fs' — for grouping and debugging
  retainLast: boolean;         // if true, new subscribers immediately receive the last published value
  schema?: string;             // optional description for documentation/debugging
  maxPayloadBytes?: number;    // dev-mode payload size limit (default: 4096). 0 = no check. See §8.1.
}

interface MessageBus {
  /** Register a channel. All channels must be registered before use. */
  registerChannel<T>(channel: ChannelDefinition<T>): void;

  /** Publish to a registered channel. Throws if channel not registered. */
  publish<T>(channelId: string, message: T): void;

  /**
   * Subscribe to a registered channel. Returns an unsubscriber.
   * If the channel has retainLast: true, handler fires immediately with the last value.
   */
  subscribe<T>(channelId: string, handler: (message: T) => void): Unsubscriber;

  /** List all registered channels (for PlaceholderBusMonitor and debugging). */
  listChannels(): ChannelDefinition<unknown>[];
}
```

**Guardrails:** Publishing to an unregistered channel throws in development and logs a warning in production. This prevents the bus from becoming a dumping ground of ad-hoc string-keyed events. Every channel must be declared in a manifest with its type and domain.

**Hard rule — bus carries events and state signals, never data payloads.** No exceptions. The `compiler.completed` channel carries a lightweight signal (`{ compileId, header, chunkHashes }`) — not the full `CompilerOutput`. Views pull derived data from the `ProjectionRegistry`, never from bus payloads. This prevents the bus from becoming a secondary data layer that duplicates, retains, and diverges from the projection system.

**Dev-mode enforcement.** In development builds, `bus.publish()` asserts that the serialized payload is under a size threshold. The default is 4KB — generous enough for any legitimate signal but catches anyone stuffing a table or AST onto the bus.

This is a safety net against accidental data dumps, not a security boundary. The threshold can be overridden per channel at registration time for channels that legitimately carry slightly larger payloads:

```typescript
bus.registerChannel({
  id: 'selection.primary',
  domain: 'selection',
  retainLast: true,
  maxPayloadBytes: 8192,          // override: selection items can be larger than 4KB in edge cases
});
```

If `maxPayloadBytes` is not specified, the 4KB default applies. Setting it to `0` disables the check for that channel (use sparingly). In production builds, the assertion is a no-op.

**Philosophy summary:** The bus carries signals by default. Small payloads are permitted for ephemeral UI state (selection changes, hover previews, streaming progress). Persistent or shared data must always live in projections. The 4KB default enforces this distinction mechanically.

### 8.2 Zone Subscription Adapter

Views should not manage raw subscriptions and teardown manually. A framework-provided adapter handles lifecycle automatically:

```typescript
// FRAMEWORK

/**
 * Creates a reactive subscription that auto-unsubscribes when the component is destroyed.
 * Returns a Svelte readable store that holds the latest value from the channel.
 */
function useBusChannel<T>(channelId: string, initialValue: T): Readable<T>;
```

Usage in a view component:

```svelte
<script>
  import { useBusChannel } from '$lib/framework/bus';
  import { projectionRegistry } from '$lib/app/projections';

  // Subscribe to the compile signal (lightweight: just compileId + header + hashes)
  const compileSignal = useBusChannel('compiler.completed', null);
  const filter = useBusChannel('filter.global.set', null);

  // Pull derived data from the projection registry, not from the bus.
  // projectionRegistry.get() is memoised — it only recomputes when dependency hashes change.
  let displayEntities = $derived(
    $compileSignal
      ? filterEntities(projectionRegistry.get('urd.projection.entityTable'), $filter)
      : []
  );
</script>
```

The bus signal triggers re-evaluation of the `$derived` block, which calls `projectionRegistry.get()`. The projection checks its dependency hashes and either returns the cached result by reference (no work) or recomputes. Views never hold a reference to the full `CompilerOutput`.

### 8.3 Channel Manifest

All channels are registered in a single manifest per layer. Framework channels are registered by the framework. Application channels are registered in bootstrap.

```typescript
// FRAMEWORK — built-in channels
bus.registerChannel({ id: 'selection.primary',    domain: 'selection', retainLast: true });
bus.registerChannel({ id: 'selection.cleared',    domain: 'selection', retainLast: false });
bus.registerChannel({ id: 'filter.global.set',    domain: 'filter',   retainLast: true });
bus.registerChannel({ id: 'filter.global.cleared', domain: 'filter',  retainLast: false });
bus.registerChannel({ id: 'fs.file.changed',      domain: 'fs',       retainLast: false });
bus.registerChannel({ id: 'fs.file.created',       domain: 'fs',       retainLast: false });
bus.registerChannel({ id: 'fs.file.deleted',       domain: 'fs',       retainLast: false });
bus.registerChannel({ id: 'focus.zone.changed',    domain: 'focus',    retainLast: true });
bus.registerChannel({ id: 'zone.error',              domain: 'zone',     retainLast: false }); // ZoneErrorBoundary logs here
bus.registerChannel({ id: 'project.opened',         domain: 'project',  retainLast: true });
bus.registerChannel({ id: 'project.closed',          domain: 'project',  retainLast: false });
bus.registerChannel({ id: 'settings.changed',        domain: 'settings', retainLast: true });

// APPLICATION — Urd-specific channels (registered in bootstrap.ts)
bus.registerChannel({ id: 'compiler.started',          domain: 'compiler', retainLast: false });
bus.registerChannel({ id: 'compiler.completed',        domain: 'compiler', retainLast: true });
    // ↑ Carries { compileId, header, chunkHashes } — a signal, NOT the full CompilerOutput.
    //   Views pull data from ProjectionRegistry, not from this channel.
bus.registerChannel({ id: 'compiler.error',            domain: 'compiler', retainLast: false });
bus.registerChannel({ id: 'playback.state.changed',    domain: 'playback', retainLast: true });
bus.registerChannel({ id: 'playback.event',            domain: 'playback', retainLast: false });
bus.registerChannel({ id: 'coverage.overlay.updated',  domain: 'coverage', retainLast: true });
bus.registerChannel({ id: 'coverage.overlay.cleared',  domain: 'coverage', retainLast: false });
```

---

## 9. View Registry

The view registry is the catalogue of all zone types. Framework-level infrastructure; Urd views register at application startup.

### 9.1 View Registration

```typescript
// FRAMEWORK

interface ViewRegistration {
  id: string;                    // unique key, e.g., "urd.entitySpreadsheet"
  name: string;                  // display name in dropdown
  icon: string;                  // icon identifier
  category: string;              // for grouping in the dropdown menu
  component: () => Promise<SvelteComponent>;  // lazy import

  /**
   * Navigation strategy — controls how the navigation broker and zone type dropdown
   * treat this view type. Replaces separate singleton/autoCreate flags with a single
   * declarative policy. Default: 'multi'.
   *
   *   'singleton-autocreate' — One instance across all workspaces. Nav broker routes
   *       to the existing instance. If none exists, auto-splits the largest zone and
   *       creates one. Dropdown disables this type if already active anywhere.
   *       Used by: Code Editor.
   *
   *   'singleton' — One instance across all workspaces. Nav broker routes to the
   *       existing instance if it exists, but does NOT auto-create. Dropdown disables
   *       if already active. Used by: Settings.
   *
   *   'multi' — Multiple instances allowed. Nav broker uses the first matching zone
   *       or creates one if needed. Default for all views.
   *
   * Singleton scope is PER APP, not per workspace. When the user switches workspace
   * tabs, singleton zones persist — the same Code Editor instance appears in every
   * workspace that includes it. This matches Blender's behaviour (3D Viewport is the
   * same instance across workspace tabs).
   */
  navigationStrategy?: 'singleton-autocreate' | 'singleton' | 'multi';

  requiresProject?: boolean;     // default true; if true, hidden from zone type dropdown when no project is open

  // Capabilities — framework uses these to build consistent chrome and enable features
  capabilities?: {
    supportsSelection?: boolean;      // highlights items from the selection bus
    supportsFilter?: boolean;         // responds to global filter context
    supportsNavigationTarget?: boolean; // can be a navigation broker target
    exportFormats?: string[];         // e.g., ['csv', 'svg', 'png'] — enables export menu items
  };

  // State versioning — prevents bricked workspaces when view state evolves
  stateVersion: number;          // increment when state shape changes
  defaultState: unknown;         // initial state for new instances
  migrateState?: (oldState: unknown, fromVersion: number) => unknown;
    // Called when loading a workspace with a stateVersion older than current.
    // Must handle all previous versions. If absent, stale state is replaced with defaultState.
}

interface ViewRegistry {
  register(view: ViewRegistration): void;
  get(id: string): ViewRegistration | undefined;
  list(): ViewRegistration[];
  listByCategory(): Map<string, ViewRegistration[]>;
  isSingletonActive(id: string): boolean;  // true if a singleton/singleton-autocreate view is currently instantiated
}
```

**State migration example:**

```typescript
viewRegistry.register({
  id: 'urd.entitySpreadsheet',
  // ...
  stateVersion: 2,
  defaultState: { sortColumn: 'id', sortDirection: 'asc', columnWidths: {}, filters: [] },
  migrateState: (old: any, fromVersion: number) => {
    if (fromVersion === 1) {
      // v1 had no filters array
      return { ...old, filters: [] };
    }
    return undefined; // unknown version → fall back to defaultState
  },
});
```

### 9.2 Category Structure (Urd Application)

```typescript
// APPLICATION

const VIEW_CATEGORIES = {
  'Editor':        ['urd.codeEditor', 'urd.sourceViewer'],
  'Navigation':    ['urd.view.fileBrowser'],
  'System':        ['forge.view.settings'],
  'Spreadsheets':  ['urd.entitySpreadsheet', 'urd.typeSpreadsheet', 'urd.propertySpreadsheet', ...],
  'Graphs':        ['urd.locationGraph', 'urd.dialogueFlowGraph', 'urd.typeHierarchyGraph', ...],
  'Inspectors':    ['urd.entityInspector', 'urd.typeInspector', 'urd.locationInspector', ...],
  'Analysis':      ['urd.reachabilityMatrix', 'urd.conditionEffectMatrix', 'urd.deadCodePanel', ...],
  'Search':        ['urd.globalSearch', 'urd.findReferences', 'urd.regexSearch', ...],
  'Runtime':       ['urd.playPanel', 'urd.stateInspector', 'urd.eventLog', ...],
  'Specialised':   ['urd.worldMap', 'urd.propertyTimeline', 'urd.schemaBrowser', ...],
};
```

### 9.3 View Host Contract

Views are rendered inside `ZoneShell`, which provides the zone header and viewport container. Two optional hooks let views integrate with framework chrome without tight coupling.

```typescript
// FRAMEWORK

/**
 * A view component may export these optional interfaces.
 * ZoneShell checks for them after mounting the view and wires them up automatically.
 */

/**
 * Toolbar contributions — lets a view add action buttons to its ZoneHeader
 * without the view knowing about ZoneHeader internals.
 */
interface ViewToolbarProvider {
  /** Returns an array of toolbar items to render in the zone header's action area.
   *  Called once on mount and whenever the returned value changes (reactive). */
  getToolbarItems(): ViewToolbarItem[];
}

interface ViewToolbarItem {
  id: string;                     // unique within this view
  icon: string;                   // icon identifier
  label: string;                  // tooltip text
  command: string;                // command ID to dispatch on click
  commandArgs?: Record<string, unknown>;
  active?: boolean;               // visually depressed state (for toggles)
  disabled?: boolean;
}

/**
 * Status hints — lets a view contribute contextual info to the GlobalStatusBar
 * when it has focus, without duplicating display logic.
 */
interface ViewStatusProvider {
  /** Returns a status hint string shown in the status bar when this zone is focused.
   *  Examples: "42 entities · 3 orphans", "Ln 117, Col 24", "Playing: scene_intro" */
  getStatusHint(): string | null;
}
```

ZoneShell detects these interfaces on the mounted view component. If `ViewToolbarProvider` is present, toolbar items are rendered in the zone header's right-side action area (after the type dropdown, before the zone menu). If `ViewStatusProvider` is present, the status bar subscribes to it when the zone gains focus and clears when focus leaves.

This avoids plugin infrastructure — it's just typed interfaces that ZoneShell looks for on the component instance.

---

## 10. Command Registry

Every action is a named command. This is the central nervous system.

### 10.1 Command Interface

```typescript
// FRAMEWORK

interface UndoAction {
  label: string;                           // human-readable description for undo stack
  undo: () => void | Promise<void>;        // reverses the command's effect
}

interface CommandRegistration {
  id: string;                              // e.g., "forge.zone.splitHorizontal"
  title: string;                           // for command palette display
  category: string;                        // for grouping
  keybinding?: string;                     // default keybinding (user-overridable)
  icon?: string;
  when?: string;                           // context condition (e.g., "activeZoneType == 'urd.codeEditor'")
  globalWhenEditorFocused?: boolean;       // if true, dispatches even when CodeMirror has focus (see §21.4)

  /**
   * Execute the command. If the command mutates persistent UI state,
   * it SHOULD return an UndoAction. Commands that are read-only (navigation,
   * focus changes) or that delegate to external systems (compiler) may return void.
   *
   * In dev mode, the framework logs commands that mutate state without returning
   * an undo action as warnings. This validates undo readiness without enforcing
   * it at runtime — global undo is a future feature, but the hooks must exist now.
   */
  execute: (args?: Record<string, unknown>) => void | UndoAction | Promise<void | UndoAction>;
}

interface CommandRegistry {
  register(command: CommandRegistration): void;
  execute(id: string, args?: Record<string, unknown>): void | Promise<void>;
  executeByKeybinding(keybinding: string): void;  // resolve keybinding → command, then execute
  resolveByKeybinding(keybinding: string): CommandRegistration | undefined;  // resolve without executing
  list(): CommandRegistration[];
  search(query: string): CommandRegistration[];  // fuzzy match for command palette
  getKeybinding(id: string): string | undefined;
  setKeybinding(id: string, keybinding: string): void;
}
```

**Undo hook example:**

```typescript
commandRegistry.register({
  id: 'forge.zone.splitHorizontal',
  title: 'Split Horizontal',
  category: 'Zone',
  execute: (args) => {
    const zoneId = args?.zoneId as string;
    const previousTree = cloneTree(workspaceManager.activeTree);
    zoneTreeReducer(workspaceManager.activeTree, { type: 'split', zoneId, direction: 'horizontal' });
    return {
      label: 'Split zone horizontal',
      undo: () => { workspaceManager.setTree(previousTree); }
    };
  }
});
```

Global undo/redo is not a v1 feature. But by requiring commands to *produce* inverse operations now, the architecture is validated and the undo stack can be added later as a consumer of the existing return values.

### 10.2 Command Namespacing

```
forge.*            → Framework commands (zone ops, workspace, theme, general navigation)
forge.zone.*       → Zone management
forge.workspace.*  → Workspace management
forge.project.*    → Project management (open, close, recent)
forge.editor.*     → Code Editor operations (framework-level, tab management)
forge.menu.*       → Menu bar operations (contributed menu items resolve to other commands)
forge.file.*       → File operations (new, open, save, save as, rename, delete)
forge.fileBrowser.* → File browser operations (reveal, focus)
forge.settings.*   → Settings view and keybinding management
forge.theme.*      → Theme switching
forge.window.*     → Fullscreen, window management
forge.contextMenu.* → Context menu operations (framework-level items)
forge.clipboard.*  → Copy, paste (controlled, not browser-native)
urd.*              → Urd-specific commands
urd.compiler.*     → Compile, recompile
urd.navigate.*     → Go-to-definition, find references, etc.
urd.play.*         → Playback controls
urd.analysis.*     → Run analysis commands
```

---

## 11. Workspace Manager

### 11.1 Workspace Tabs

A row of tabs at the very top of the application window (above the BSP zone tree). Each tab is a named workspace containing a complete layout state.

```typescript
// FRAMEWORK

interface Workspace {
  id: string;
  name: string;
  tree: ZoneTree;                    // the BSP layout
  zoneStates: ZoneStateStore;        // single authority — workspace serialization stores this map verbatim
}

interface WorkspaceManager {
  workspaces: Workspace[];
  activeIndex: number;
  activate(index: number): void;
  create(name: string, template?: Workspace): void;
  duplicate(index: number): void;
  remove(index: number): void;
  rename(index: number, name: string): void;
  serialize(): SerializedWorkspaceSet;
  deserialize(data: SerializedWorkspaceSet): void;
}
```

**Corruption-safe loading.** `deserialize()` wraps the entire load in a try/catch. If parsing or migration fails for any reason:
1. The broken file is preserved as `workspaces.backup.json` (overwriting any previous backup).
2. The workspace manager initialises with the default "Writer" template.
3. A non-modal status bar warning appears: *"Workspace file was corrupted — loaded defaults. Backup saved."*

This prevents a single bad workspace file from bricking the application on launch.

### 11.2 Predefined Workspace Templates (Urd Application)

```typescript
// APPLICATION

const WORKSPACE_TEMPLATES = {
  'Writer': {
    // Left 15%: File Browser | Center 50%: Code Editor | Right 35% top: Dialogue Preview, bottom: Play Panel
    tree: split('horizontal', 0.15, [
      leaf('urd.view.fileBrowser'),
      split('horizontal', 0.59, [
        leaf('urd.codeEditor'),
        split('vertical', 0.5, [
          leaf('urd.dialoguePreview'),
          leaf('urd.playPanel'),
        ])
      ])
    ])
  },
  'Engineer': {
    // Left 15%: File Browser | Center: Code Editor | Right top: Property Spreadsheet | Right bottom: Diagnostics
    tree: split('horizontal', 0.15, [
      leaf('urd.view.fileBrowser'),
      split('horizontal', 0.52, [
        leaf('urd.codeEditor'),
        split('vertical', 0.6, [
          leaf('urd.propertySpreadsheet'),
          leaf('urd.diagnosticSpreadsheet'),
        ])
      ])
    ])
  },
  'World Builder': {
    // Top left: Code Editor | Top right: Location Graph | Bottom: Entity Spreadsheet
    tree: split('vertical', 0.65, [
      split('horizontal', 0.5, [
        leaf('urd.codeEditor'),
        leaf('urd.locationGraph'),
      ]),
      leaf('urd.entitySpreadsheet'),
    ])
  },
  'QA': {
    // Code Editor | Coverage Overlay | Monte Carlo | Dead Code
    tree: split('horizontal', 0.4, [
      leaf('urd.codeEditor'),
      split('vertical', 0.33, [
        leaf('urd.coverageOverlay'),
        split('vertical', 0.5, [
          leaf('urd.monteCarloDashboard'),
          leaf('urd.deadCodePanel'),
        ])
      ])
    ])
  },
  'Debug': {
    // Play Panel | State Inspector | Event Log | Code Editor
    tree: split('horizontal', 0.5, [
      split('vertical', 0.6, [
        leaf('urd.playPanel'),
        leaf('urd.eventLog'),
      ]),
      split('vertical', 0.6, [
        leaf('urd.codeEditor'),
        leaf('urd.stateInspector'),
      ])
    ])
  },
};
```

### 11.3 Workspace Serialization Scope

Serialized per-project as a JSON file (e.g., `.forge/workspaces.json`):

- Full BSP tree structure (including singleton reference leaves — see §11.4)
- Zone types for each leaf
- Per-zone state (open tabs, scroll positions, filter configs, graph viewport transforms)
- Active workspace index
- Window dimensions

Predefined templates are **not** saved — they're always available as "new workspace from template" options.

### 11.4 Singleton Zone Lifecycle Across Workspaces

Singleton views (Code Editor, Settings) exist once per app but can appear in multiple workspace trees. This creates a tension: each workspace owns its own BSP tree, but singletons must not duplicate state.

**The rule:** Singleton zones are represented in workspace trees as **reference leaves**. A reference leaf stores only `{ zoneTypeId, ref: true }` — it does not own state. The singleton's actual state is stored once in `ZoneStateStore` under a stable synthetic key: `singleton::${zoneTypeId}` (e.g., `singleton::urd.codeEditor`). This key is independent of any `zoneId`.

```typescript
// FRAMEWORK — leaf node in the BSP tree

type LeafNode =
  | { kind: 'leaf'; id: string; zoneTypeId: string }                   // normal leaf — owns state
  | { kind: 'leaf'; id: string; zoneTypeId: string; singletonRef: true } // reference leaf — mounts shared instance
```

**Mounting:** When the layout renderer encounters a `singletonRef` leaf, it mounts the single shared instance of that view. The instance's state is read from `ZoneStateStore` using the `singleton::` key, not the leaf's `zoneId`. If the singleton is already mounted in the current workspace (shouldn't happen due to invariant #6), the renderer shows a "Already visible" placeholder.

**Serialization:**
- Normal leaves serialize their `zoneId`, `zoneTypeId`, and a reference to their state in `ZoneStateStore`.
- Reference leaves serialize `zoneId`, `zoneTypeId`, and `singletonRef: true`. No state blob — the state lives under the `singleton::` key.
- The `singleton::` state entries are serialized once at the top level of `workspaces.json`, not inside any individual workspace.

```json
{
  "workspaces": [
    { "name": "Writer", "tree": { "...": "includes singletonRef leaves" } },
    { "name": "Engineer", "tree": { "...": "also includes singletonRef leaves" } }
  ],
  "singletonState": {
    "singleton::urd.codeEditor": { "stateVersion": 1, "data": { "openTabs": [...], "activeTab": "..." } },
    "singleton::forge.view.settings": { "stateVersion": 1, "data": { "activeCategory": "appearance" } }
  },
  "zoneStates": { "...": "normal per-zone state" },
  "activeIndex": 0
}
```

**Workspace switching:** Singleton instances are mounted in a **permanent host element** outside the BSP tree (a hidden container in the app shell). When the layout renderer encounters a `singletonRef` leaf, it reparents the singleton's DOM element into that leaf's viewport container. On workspace switch, the DOM element is reparented to the new leaf position — the Svelte component is never destroyed or recreated. This portal-style approach avoids accidental remounts, preserves editor state (cursor, undo history, scroll position, CodeMirror internal state), and keeps the component lifecycle completely stable.

If the new workspace doesn't include a reference leaf for a singleton, its DOM element stays in the hidden host (invisible but alive). This means the Code Editor retains all its state even when temporarily not displayed.

**Workspace without the Code Editor:** If no workspace currently visible includes a Code Editor reference leaf, the navigation broker enters queuing mode (as described in §5.6) — intents are buffered and the "No Code Editor — click to restore" toast appears.

---

## 12. Selection Context

### 12.1 Model

```typescript
// FRAMEWORK (generic selection infrastructure)

interface SelectionContext {
  primary: SelectionItem | null;
  subscribe(handler: (item: SelectionItem | null) => void): Unsubscriber;
  select(item: SelectionItem): void;
  clear(): void;
}

// APPLICATION (Urd-specific selection types)

type SelectionItem =
  | { kind: 'entity';   id: string }
  | { kind: 'type';     id: string }
  | { kind: 'location'; id: string }
  | { kind: 'section';  id: string }
  | { kind: 'property'; typeId: string; propertyId: string }
  | { kind: 'rule';     id: string }
  | { kind: 'action';   id: string }
  | { kind: 'sequence'; id: string }
  | { kind: 'choice';   id: string }
  | { kind: 'exit';     fromLocation: string; direction: string }
  | { kind: 'diagnostic'; filePath: string; span: [number, number]; code: string };
    // ↑ Stable composite key — survives recompilation as long as the diagnostic still exists.
    //   Using an array index would break on every recompile as the diagnostics list reorders.
```

**Selection items are keys, not data.** Every `SelectionItem` is a lightweight stable identifier — an entity ID, a file path + span, a type name. Selection items never carry display data, message text, or resolved details. Views that need to display information about the selected item pull it from projections using the key. This keeps selection payloads tiny (well under the bus 4KB limit), ensures consistency (all views resolve the same key to the same projection data), and prevents selection items from going stale when compiler output changes.

**Future: named selection slots.** The current `SelectionContext` has a single `primary` slot. The interface is designed so that named slots (e.g., `primary`, `comparison`, `pinned`) can be added later without changing the bus contract or view subscription pattern. For v1, `primary` is sufficient.

### 12.2 Selection → View Sync

When `selection.select()` is called, the bus publishes `selection.primary` with the item. Every view that displays the selected item type highlights it. The selection context is a framework service; the selection *types* are Urd-specific.

### 12.3 Global Filter

Separate from selection. The filter narrows what views display.

```typescript
// APPLICATION

interface FilterContext {
  activeFilter: FilterItem | null;
  set(filter: FilterItem): void;
  clear(): void;
}

type FilterItem =
  | { kind: 'location'; id: string }   // show only things in this location
  | { kind: 'type';     id: string }   // show only entities/properties of this type
  | { kind: 'file';     path: string } // show only things declared in this file
  ;
```

When a filter is active, views that support filtering reduce their displayed data accordingly. Views opt-in to filter support; not all views need it.

---

## 13. Compiler Service

### 13.1 Interface

```typescript
// APPLICATION (but designed for swappable implementations)

interface CompilerService {
  compile(buffers: Map<string, string>): Promise<CompilerOutput>;
  getAnalysis(type: AnalysisType, params?: unknown): Promise<AnalysisResult>;
  simulate(config: SimulationConfig): Promise<SimulationResult>;
}

type AnalysisType =
  | 'reachabilityMatrix'
  | 'deadCode'
  | 'monteCarlo'
  | 'enumCoverage'
  | 'thresholdAnalysis'
  | 'circularDependency'
  | 'impactAnalysis'
  ;
```

### 13.2 Chunked Output with Content Hashes

The compiler output is split into named chunks, each with a content hash. On recompile, the backend sends unchanged chunks as `{ status: 'unchanged', hash }` stubs (no data), and the frontend reuses the previous in-memory object. This avoids redundant JSON parsing, garbage collection churn, and Svelte invalidations for data that hasn't changed.

**Completeness rule:** Every `CompilerOutput` must include **all** chunk names in its `chunks` object, every time, even when unchanged. Chunks are never absent — they are either `'updated'` (with data) or `'unchanged'` (hash only). This guarantees that `Object.keys(output.chunks)` is always the full set of `ChunkName` values, and that the `chunkHashes` map in the `CompileSignal` bus event is always complete. The frontend cache, projection registry, and bus signal all depend on this invariant.

```typescript
interface CompilerOutput {
  header: OutputHeader;          // always sent: version, timings, counts, diagnostics summary
  chunks: {
    ast:                    Chunk<AST>;
    symbolTable:            Chunk<SymbolTable>;
    factSet:                Chunk<FactSet>;
    propertyDependencyIndex: Chunk<PropertyDependencyIndex>;
    urdJson:                Chunk<UrdWorld>;
    diagnostics:            Chunk<Diagnostic[]>;
  };
}

interface OutputHeader {
  schemaVersion: number;             // CompilerOutput schema version — increment on breaking changes
  compileId: string;                 // unique ID for this compile run
  timing: PhaseTiming[];
  counts: WorldCounts;           // files, entities, locations, sections, etc.
  diagnosticSummary: { errors: number; warnings: number; info: number };
}

type Chunk<T> =
  | { status: 'updated'; hash: string; data: T }
  | { status: 'unchanged'; hash: string };         // frontend reuses previous

// Type-safe chunk name — prevents typos in projection dependsOn declarations.
type ChunkName = keyof CompilerOutput['chunks'];
// Resolves to: 'ast' | 'symbolTable' | 'factSet' | 'propertyDependencyIndex' | 'urdJson' | 'diagnostics'
```

**Frontend chunk resolution:**

```typescript
// APPLICATION

class CompilerOutputCache {
  private cache = new Map<string, unknown>();  // chunk name → last data

  resolve(output: CompilerOutput): ResolvedCompilerOutput {
    const resolved: any = { header: output.header, chunks: {} };
    for (const [name, chunk] of Object.entries(output.chunks)) {
      if (chunk.status === 'updated') {
        this.cache.set(name, chunk.data);
        resolved.chunks[name] = chunk.data;
      } else {
        resolved.chunks[name] = this.cache.get(name);  // reuse previous
      }
    }
    return resolved;
  }
}
```

For a typical keystroke in one dialogue file: only `ast` and `diagnostics` change. The `symbolTable`, `factSet`, and `urdJson` hashes stay the same, avoiding 80%+ of the parse/render cost.

**Cache memory model:** The cache holds exactly one entry per chunk name (`ast`, `symbolTable`, `factSet`, `propertyDependencyIndex`, `urdJson`, `diagnostics`). Each `resolve()` call replaces updated entries and retains unchanged ones — the cache never grows beyond this fixed set. Total memory is bounded to the size of one complete `CompilerOutput`. There is no eviction strategy because there is nothing to evict — it is a fixed-size sliding window over the last compile.

### 13.3 Buffer Map (In-Memory Compilation)

The compiler reads from an in-memory buffer map, not from disk. This decouples typing from file I/O and avoids filesystem watcher loops.

```
User types in Code Editor
  → CodeMirror dispatches transaction
  → BufferMap.set(filePath, content)        // in-memory, instant
  → 300ms debounce
  → bus.publish('compiler.started')
  → CompilerService.compile(bufferMap)      // IPC sends buffer contents
  → Rust compiles from provided sources
  → CompilerOutputCache.resolve() — reuses unchanged chunks
  → projectionRegistry.updateSource(resolved)
  → bus.publish('compiler.completed', { compileId, header, chunkHashes })  // signal only

Explicit Save (Ctrl+S)
  → FileSystem.writeFile(filePath, bufferMap.get(filePath))
  → Disk is updated

External edit (another editor changes a file on disk)
  → File watcher detects change
  → BufferMap.set(filePath, FileSystem.readFile(filePath))
  → Triggers recompile as above
```

```typescript
// APPLICATION

interface BufferMap {
  get(path: string): string | undefined;
  set(path: string, content: string): void;
  getAll(): Map<string, string>;
  isDirty(path: string): boolean;         // buffer differs from last saved content
  markClean(path: string): void;          // called after successful save
  subscribe(path: string, handler: (content: string) => void): Unsubscriber;
}
```

This means the compiler is always working with the latest keystrokes, disk writes only happen on save, and external file changes are merged cleanly. It also eliminates the Windows file watcher churn problem entirely.

### 13.4 IPC Strategy

Heavy analysis commands (`monteCarlo`, `reachabilityMatrix`) are separate IPC calls that return independently. They do not block the main compilation cycle. Long-running analyses stream progress events via Tauri events.

---

## 14. Projection Layer

Views should not walk raw `CompilerOutput` structures directly. A projection layer sits between the compiler output and the views, providing memoised, view-friendly datasets.

### 14.1 Rationale

Without this layer, each of the 14 spreadsheets independently traverses the SymbolTable, FactSet, and PropertyDependencyIndex to produce its rows. This duplicates logic, creates inconsistencies (different sort/filter implementations), and makes performance unpredictable.

Projections are memoised selectors: they take the resolved `CompilerOutput` and produce stable, sorted, filterable datasets. They recompute only when their input chunk changes (leveraging the content hashes from Section 13.2).

### 14.2 Interface

```typescript
// APPLICATION

/**
 * A projection transforms resolved CompilerOutput into a view-friendly dataset.
 * Projections are memoised by the combined hash of their dependency chunks.
 * They recompute lazily on access when dependency hashes have changed —
 * there is no explicit invalidation.
 */
interface Projection<T> {
  id: string;                              // e.g., 'urd.projection.entityTable'
  dependsOn: ChunkName[];                  // type-safe — compiler error if you typo a chunk name
  mode?: ProjectionMode;                   // default: 'sync'. See tiered projections below.
  compute(output: ResolvedCompilerOutput): T;
}

/**
 * Tiered projection modes. Designed for now, implemented incrementally.
 *
 * 'sync'    — (default, v1) Computed synchronously in the main thread on get().
 *             Suitable for most projections. If a sync projection takes >16ms,
 *             it should be promoted to 'worker'.
 *
 * 'worker'  — (future) Computed in a Web Worker. get() returns the last cached
 *             result immediately and triggers async recomputation. The registry
 *             publishes a 'projection.updated' bus signal when the worker finishes.
 *             Use for expensive joins, aggregations, and graph layout.
 *
 * 'backend' — (future) Precomputed in Rust and shipped as a compiler chunk.
 *             The projection's compute() is a no-op pass-through. Use for
 *             O(n²) analysis like reachability matrices.
 *
 * v1 implements 'sync' only. The mode field exists so projections can be
 * promoted without changing their registration site or consumer code.
 */
type ProjectionMode = 'sync' | 'worker' | 'backend';

interface ProjectionRegistry {
  register<T>(projection: Projection<T>): void;

  /**
   * Returns the projection result. Lazily recomputes if any dependency chunk hash
   * has changed since the last computation. Otherwise returns the cached result
   * by reference. This is the only way views should access derived data.
   */
  get<T>(id: string): T;

  /**
   * Called once after each compile with the new resolved output.
   * Stores the output reference and chunk hashes. Does NOT trigger recomputation —
   * projections recompute lazily on next get() call.
   */
  updateSource(output: ResolvedCompilerOutput): void;
}
```

**Memoisation internals:**

```typescript
// APPLICATION — inside ProjectionRegistry implementation

interface CachedProjection<T> {
  projection: Projection<T>;
  lastDependencyHash: string | null;   // combined hash of dependency chunks at last compute
  lastResult: T | null;
}

function get<T>(id: string): T {
  const cached = this.cache.get(id) as CachedProjection<T>;
  const currentHash = this.computeDependencyHash(cached.projection.dependsOn);

  if (cached.lastDependencyHash === currentHash && cached.lastResult !== null) {
    return cached.lastResult;  // cache hit — same reference as before
  }

  // Dependency hashes changed — recompute
  const result = cached.projection.compute(this.currentOutput);
  cached.lastDependencyHash = currentHash;
  cached.lastResult = __DEV__ ? Object.freeze(result) : result;  // prevent accidental mutation in dev
  return cached.lastResult;
}

function computeDependencyHash(chunkNames: ChunkName[]): string {
  // Include schemaVersion so that all projections invalidate on schema upgrade,
  // even if chunk data happens to hash identically across versions.
  const version = String(this.currentOutput.header.schemaVersion);
  return version + ':' + chunkNames.map(name => this.currentOutput.chunkHashes[name]).join(':');
}
```

This means projections never recompute unnecessarily. If you edit a dialogue file and only `ast` and `diagnostics` chunks change, then `urd.projection.entityTable` (which depends on `symbolTable`, `propertyDependencyIndex`, `urdJson`) returns the exact same cached object by reference. Svelte detects no change and skips the re-render entirely.

### 14.3 Example Projections

```typescript
// APPLICATION

// Entity table — consumed by Entity Spreadsheet, Entity Inspector, Containment Tree
projectionRegistry.register({
  id: 'urd.projection.entityTable',
  dependsOn: ['symbolTable', 'propertyDependencyIndex', 'urdJson'],
  compute: (output) => {
    return output.chunks.symbolTable.entities.map(entity => ({
      id: entity.id,
      typeName: entity.type_name,
      container: resolveContainer(entity, output.chunks.urdJson),
      properties: mergeProperties(entity, output.chunks.symbolTable),
      readCount: sumReads(entity, output.chunks.propertyDependencyIndex),
      writeCount: sumWrites(entity, output.chunks.propertyDependencyIndex),
      hasOrphans: checkOrphans(entity, output.chunks.propertyDependencyIndex),
      declaredIn: entity.declared_in,
    }));
  }
});

// Location graph — consumed by Location Network Graph, World Map, Reachability Matrix
projectionRegistry.register({
  id: 'urd.projection.locationGraph',
  dependsOn: ['symbolTable', 'factSet'],
  compute: (output) => ({
    nodes: buildLocationNodes(output.chunks.symbolTable.locations),
    edges: buildExitEdges(output.chunks.factSet.exits),
    reachability: computeReachability(output.chunks.factSet.exits, output.chunks.urdJson.start),
  })
});
```

**Multiple views share the same projection.** The Entity Spreadsheet, Entity Inspector, and Containment Tree all consume `urd.projection.entityTable`. The projection computes once; all three views get the same reference. This guarantees consistency — if one view shows an entity as orphaned, every view agrees.

**Where computation lives — Rust vs frontend rule of thumb.** If the analysis is O(n²), involves graph traversal over the full world dataset, or benefits from streaming progress, it belongs in Rust and ships as a chunk or a dedicated IPC command. If it is shaping or indexing data for a specific view and can be recomputed quickly from a small number of chunks, it lives as a TypeScript projection. Examples: reachability matrix computation → Rust. Flattening the entity table for a spreadsheet → projection.

**Data authority boundaries — who builds what:**
- **Projections** produce immutable snapshots derived from compiler chunks. They transform, flatten, and shape data for views. They do NOT build indexes or maps for O(1) lookup.
- **QueryService** (Phase 4) provides indexed lookups over projections — `entityById()`, `diagnosticsForFile()`, `typeById()`. It is the only place allowed to build `Map` objects for O(1) access. It caches these maps and invalidates them when the underlying projection recomputes.
- **Views** consume projections (for lists, tables, graphs) and queries (for point lookups). Views never build their own indexes over projection data. If a view needs an index, it belongs in QueryService.

---

## 15. File System Abstraction

```typescript
// FRAMEWORK (generic interface)

interface ForgeFileSystem {
  readFile(path: string): Promise<string>;
  writeFile(path: string, content: string): Promise<void>;
  listDirectory(path: string): Promise<FileEntry[]>;
  stat(path: string): Promise<FileStat>;
  watchFile(path: string, handler: (event: FileChangeEvent) => void): Unsubscriber;
  watchDirectory(path: string, handler: (event: FileChangeEvent) => void): Unsubscriber;
}

// APPLICATION (Tauri implementation, v1 target)

class TauriFileSystem implements ForgeFileSystem {
  // Uses @tauri-apps/plugin-fs for read/write
  // Uses Rust-side `notify` crate for file watching via Tauri events
}

// FUTURE (web deployment)

class IndexedDBFileSystem implements ForgeFileSystem {
  // Uses IndexedDB for storage
  // watchFile becomes a polling mechanism or noop
}
```

---

## 16. Project Management

A project is a directory on disk that contains Urd source files. This follows the VSCode model — "Open Folder" *is* "Open Project." There is no project file format, wizard, or creation ceremony. You point Forge at a folder and it becomes your project.

### 16.1 Interface

```typescript
// FRAMEWORK

interface Project {
  rootPath: string;               // absolute path to project directory
  name: string;                   // derived from directory name (or .forge/project.json override)
}

interface RecentProject {
  rootPath: string;
  name: string;
  lastOpened: string;             // ISO datetime
}

interface ProjectManager {
  /** The currently open project, or null if on Welcome screen. */
  readonly current: Project | null;

  /** Open a project by selecting its root directory. Shows native OS directory picker.
   *  Closes any existing project first (prompts to save dirty buffers). */
  open(): Promise<void>;

  /** Open a specific project by path (used by recent projects list). */
  openPath(rootPath: string): Promise<void>;

  /** Close the current project. Prompts to save dirty buffers.
   *  Returns to the Welcome screen. */
  close(): Promise<void>;

  /** Recently opened projects, most recent first. Persisted in app settings. */
  readonly recentProjects: RecentProject[];
}
```

### 16.2 Project Lifecycle

```
App launch
  → Load app settings (theme, keybindings, recent projects)
  → If openLastProjectOnLaunch && last project path still exists → open it
  → Otherwise → show Welcome screen

Open Project
  → If project currently open → close it first (save prompt if dirty)
  → Native directory picker (Tauri dialog API)
  → Set projectManager.current
  → Load workspace layout from <projectRoot>/.forge/workspaces.json (corruption-safe)
  → Scan directory → populate bufferMap with all .urd.md files
  → Start file watcher on project root
  → Trigger initial compile
  → Update window title: "Urd Forge — <project name>"
  → Add/update in recentProjects
  → bus.publish('project.opened', { rootPath, name })

Close Project
  → Check bufferMap for dirty buffers → prompt: Save All / Discard / Cancel
  → Save workspace state to <projectRoot>/.forge/workspaces.json
  → Stop file watcher
  → bufferMap.clear()
  → projectionRegistry.clear()
  → Set projectManager.current = null
  → bus.publish('project.closed')
  → Show Welcome screen
```

### 16.3 Project-Local Storage

Per-project state lives in a `.forge/` directory inside the project root:

```
my-urd-project/
├── .forge/                      # Forge IDE state (add to .gitignore)
│   ├── workspaces.json          # BSP layouts, zone states, active workspace
│   └── project.json             # Optional: display name, compiler flags overrides
├── world/
│   ├── entities.urd.md
│   ├── locations.urd.md
│   └── ...
└── .gitignore
```

The `.forge/` directory is created on first open if it doesn't exist. No Forge state lives outside this directory. Deleting `.forge/` resets the IDE to defaults for that project — all source files are untouched.

### 16.4 Welcome Screen

When no project is open, the entire viewport is replaced by a Welcome screen. This is **not** a zone type — it replaces the workspace layout entirely (like Blender's splash screen or VSCode's Welcome tab).

Contents:
- **Open Project** button → `forge.project.open`
- **New Project** button → creates a directory with a starter `.urd.md`, then opens it
- **Recent Projects** list → clickable rows, each calls `projectManager.openPath()`. Shows name, path, last opened date. Right-click → Remove from Recent.
- Keyboard: `Ctrl+O` opens directory picker. Recent projects also accessible via command palette (`forge.project.openRecent`).

The Welcome screen disappears the moment a project is opened and reappears when a project is closed. There is no "empty editor" state — either a project is open and you see the workspace, or no project is open and you see Welcome.

### 16.5 Bus Channels

```typescript
bus.registerChannel({ id: 'project.opened',  domain: 'project', retainLast: true });
bus.registerChannel({ id: 'project.closed',  domain: 'project', retainLast: false });
```

### 16.6 Commands

| Command | Keybinding | Action |
|---------|------------|--------|
| `forge.project.open` | `Ctrl+O` | Open project (native directory picker) |
| `forge.project.openRecent` | `Ctrl+Shift+O` | Show recent projects in command palette |
| `forge.project.close` | — | Close current project |
| `forge.project.newProject` | — | Create new project (name prompt → directory picker → scaffold → open) |

---

## 17. File Browser

The File Browser is a **zone type** (Blender model), not a fixed sidebar. It lives in the BSP tree and can be resized, swapped, or closed like any other view. Most workspace templates place it in the left-most zone, but this is convention, not constraint.

### 17.1 View Registration

```typescript
// APPLICATION

viewRegistry.register({
  id: 'urd.view.fileBrowser',
  name: 'File Browser',
  icon: 'folder-tree',
  category: 'Navigation',
  component: () => import('./views/FileBrowser.svelte'),
  navigationStrategy: 'multi',    // multiple file browsers allowed (different expanded states)
  requiresProject: true,          // hidden from zone type dropdown when no project is open
  capabilities: {
    supportsSelection: true,      // selected file(s) participate in SelectionContext
    supportsFilter: false,
    supportsNavigationTarget: true, // "reveal in file browser" navigates here
    exportFormats: [],
  },
  defaultState: {
    expandedPaths: [],
    sortBy: 'name',               // 'name' | 'modified' | 'type'
    showHiddenFiles: false,
  },
  stateVersion: 1,
});
```

### 17.2 Tree Structure

The file browser reads the project directory tree from `ForgeFileSystem.listDirectory()` recursively (lazy — expand on click, not on load). It shows:

```
my-urd-project/
├── 📁 world/                    [expandable]
│   ├── 📄 entities.urd.md      [click → open in Code Editor]
│   ├── 📄 locations.urd.md     [dirty indicator: dot after name]
│   └── 📄 dialogue.urd.md      [error indicator: red text]
├── 📁 assets/                   [expandable]
│   └── 📄 notes.txt            [non-.urd.md files shown but muted]
└── 📄 README.md
```

**Visual indicators per file:**
- Dirty (unsaved changes in BufferMap): dot/circle after filename (VSCode convention)
- Has errors: filename in `--forge-status-error` colour
- Has warnings only: filename in `--forge-status-warning` colour
- Non-Urd files: shown but visually muted (`--forge-text-muted`)

The file browser subscribes to the `compiler.completed` signal and pulls diagnostic counts per file from a projection (`urd.projection.diagnosticsByFile`).

### 17.3 Interactions

**Single click** on a file → opens it in the Code Editor singleton (via navigation broker). If already open in a tab, switches to it.

**Double click** on a file → same as single click. No "preview" vs "permanent" distinction for v1.

**Right-click on a file** → context menu:

| Item | Command | Notes |
|------|---------|-------|
| Open | `forge.editor.openFile` | Same as click |
| Open in Source Viewer | `forge.sourceViewer.open` | Opens in a separate Source Viewer zone |
| Rename | `forge.file.rename` | Inline rename (VSCode-style). Updates BufferMap key. |
| Delete | `forge.file.delete` | Confirmation dialog. Moves to OS trash if available. |
| Reveal in OS File Manager | `forge.file.revealInOS` | `shell.open` via Tauri |
| Copy Path | `forge.file.copyPath` | Copies absolute path to clipboard |
| Copy Relative Path | `forge.file.copyRelativePath` | Relative to project root |

**Right-click on a directory** → context menu:

| Item | Command | Notes |
|------|---------|-------|
| New File | `forge.file.newFile` | Creates `.urd.md` in this directory (name prompt) |
| New Folder | `forge.file.newFolder` | Creates subdirectory (name prompt) |
| Rename | `forge.file.rename` | Inline rename |
| Delete | `forge.file.delete` | Confirmation: "Delete folder and all contents?" |
| Reveal in OS File Manager | `forge.file.revealInOS` | |

**Right-click on empty space** → New File, New Folder (at project root).

### 17.4 File Watcher Integration

The file browser subscribes to `fs.file.created`, `fs.file.deleted`, and `fs.file.changed` bus channels. When external changes happen (git checkout, another editor), the tree updates automatically.

Directory expansion is lazy — only expanded directories are watched. Collapsing a directory stops watching its children. This keeps the watcher count bounded.

### 17.5 Keyboard Navigation

When the File Browser zone has focus:
- `↑`/`↓` — move selection through visible tree
- `→` — expand directory / move into first child
- `←` — collapse directory / move to parent
- `Enter` — open selected file in Code Editor
- `Delete` — delete selected (with confirmation)
- `F2` — rename inline
- `Ctrl+Shift+E` — focus the File Browser zone from anywhere (VSCode convention)

### 17.6 "Reveal in File Browser" Command

```typescript
commandRegistry.register({
  id: 'forge.fileBrowser.reveal',
  title: 'Reveal in File Browser',
  category: 'Navigation',
  keybinding: 'ctrl+shift+r',
  execute: ({ filePath }) => {
    // Find first File Browser zone (via view registry)
    // Expand parent directories as needed
    // Scroll to file, highlight it
    // Focus the File Browser zone
  }
});
```

Available from the editor tab right-click menu, from any "go to source" result, and from the command palette.

### 17.7 New Projection

```typescript
// APPLICATION — diagnostic counts per file for file browser indicators
projectionRegistry.register({
  id: 'urd.projection.diagnosticsByFile',
  dependsOn: ['diagnostics'] as ChunkName[],
  compute: (output) => {
    const map = new Map<string, { errors: number; warnings: number }>();
    for (const diag of output.chunks.diagnostics) {
      const entry = map.get(diag.filePath) ?? { errors: 0, warnings: 0 };
      if (diag.severity === 'error') entry.errors++;
      else if (diag.severity === 'warning') entry.warnings++;
      map.set(diag.filePath, entry);
    }
    return map;
  }
});
```

---

## 18. Application Settings

Application settings are **global** (not per-project) and persist across sessions in the OS config directory. Following Blender's model, the Settings view is a zone type — you can open it in any zone, side-by-side with your editor, not as a blocking modal.

### 18.1 Settings Data

```typescript
// FRAMEWORK

interface AppSettings {
  // Appearance
  theme: 'gloaming' | 'parchment';
  uiFontSize: number;                    // default: 13

  // Editor
  editorFontFamily: string;              // default: 'JetBrains Mono'
  editorFontSize: number;                // default: 14
  editorTabSize: number;                 // default: 2
  editorWordWrap: boolean;               // default: false
  editorLineNumbers: boolean;            // default: true
  editorMinimap: boolean;                // default: false

  // Compiler / Behaviour
  autoSaveOnCompile: boolean;            // default: false
  recompileDebounceMs: number;           // default: 300
  openLastProjectOnLaunch: boolean;      // default: true

  // State (managed by other services, persisted here)
  recentProjects: RecentProject[];
  keybindingOverrides: Record<string, string>;  // commandId → keybinding (sparse, user-modified only)
  lastProjectPath: string | null;
}

interface AppSettingsService {
  readonly current: Readonly<AppSettings>;
  get<K extends keyof AppSettings>(key: K): AppSettings[K];
  set<K extends keyof AppSettings>(key: K, value: AppSettings[K]): void;
  reset(key: keyof AppSettings): void;   // reset single setting to default
  resetAll(): void;                      // reset everything (with confirmation)
}
```

### 18.2 Storage Location

```
// Tauri standard config paths (via @tauri-apps/api/path):
// macOS:   ~/Library/Application Support/com.urd-forge.app/settings.json
// Linux:   ~/.config/com.urd-forge.app/settings.json
// Windows: %APPDATA%/com.urd-forge.app/settings.json
```

Loaded once at startup. Written on change (debounced 500ms). Same corruption-safe pattern as workspaces — if `settings.json` fails to parse, back it up as `settings.backup.json` and load defaults.

### 18.3 Settings View (Zone Type)

```typescript
// FRAMEWORK

viewRegistry.register({
  id: 'forge.view.settings',
  name: 'Settings',
  icon: 'settings',
  category: 'System',
  component: () => import('./views/SettingsView.svelte'),
  navigationStrategy: 'singleton', // one Settings view at a time; not auto-created
  requiresProject: false,         // available even on Welcome screen
  capabilities: {
    supportsSelection: false,
    supportsFilter: true,         // searchable settings list
    supportsNavigationTarget: false,
    exportFormats: [],
  },
  defaultState: {
    activeCategory: 'appearance',
    searchQuery: '',
  },
  stateVersion: 1,
});
```

### 18.4 Settings Layout

The Settings view has a categorised sidebar and a scrollable settings area:

```
┌─ Settings ─────────────────────────────────────────┐
│ 🔍 Search settings...                              │
├──────────────┬──────────────────────────────────────┤
│              │                                      │
│ Appearance   │  Theme         [Gloaming ▾]          │
│ Editor       │  UI Font Size  [13      ]            │
│ Behaviour    │                                      │
│ Keybindings  │  ─── Editor ───                      │
│              │  Font Family   [JetBrains Mono    ]  │
│              │  Font Size     [14      ]            │
│              │  Tab Size      [2       ]            │
│              │  Word Wrap     [○ off   ]            │
│              │  Line Numbers  [● on    ]            │
│              │  Minimap       [○ off   ]            │
│              │                                      │
└──────────────┴──────────────────────────────────────┘
```

**Search**: Typing filters all settings across all categories (VSCode-style instant filter). Category sidebar highlights which categories have matching results.

**Keybindings category**: Full command registry as a filterable table. Each row: command name, category, current keybinding, default binding. Click a keybinding cell → enters capture mode (press desired key chord, Escape cancels). Conflicts shown inline as warnings. Reset-to-default button per binding.

**Changes apply immediately** — no Save or Apply button (Blender convention). The debounced disk write handles persistence. If you change the theme, it switches instantly. If you change the editor font, open editors react via the `settings.changed` bus channel.

### 18.5 Settings Bus Integration

```typescript
bus.registerChannel({ id: 'settings.changed', domain: 'settings', retainLast: true });
// Payload: { key: keyof AppSettings, value: unknown, previousValue: unknown }
```

When a setting changes:
1. `AppSettingsService` updates in-memory state
2. Publishes `settings.changed` signal
3. Queues debounced disk write (500ms)
4. Consumers react — theme engine applies new tokens, Code Editor updates font/size, etc.

### 18.6 Commands

| Command | Keybinding | Action |
|---------|------------|--------|
| `forge.settings.open` | `Ctrl+,` | Open/focus Settings view. If no instance exists, splits active zone and creates one. |
| `forge.settings.openKeybindings` | `Ctrl+K Ctrl+S` | Open Settings directly to Keybindings category |
| `forge.settings.resetAll` | — | Reset all settings to defaults (confirmation prompt) |

### 18.7 Per-Project Overrides (Future)

For v1, all settings are global. The `.forge/project.json` file reserves a `settings` key for future per-project overrides (e.g., `tabSize`, `compilerFlags`). When implemented, per-project settings will merge on top of global settings with a clear "this is overridden by project" indicator in the Settings view.

---

## 19. Focus and Modality Service

A framework-level service that tracks which zone, divider, or UI element has focus. Essential for keybinding resolution, command context, and Blender-style "operator acts on the active area" behaviour.

### 19.1 Interface

```typescript
// FRAMEWORK

type FocusMode = 'normal' | 'commandPalette' | 'contextMenu' | 'modal';

interface FocusService {
  /** The currently focused zone ID. Null if focus is on global chrome (menu, status bar). */
  activeZoneId: string | null;

  /** The zone type of the focused zone. Used for `when` conditions on commands. */
  activeZoneType: string | null;

  /** The most recently focused divider (for divider keybinding resolution). */
  activeDividerId: string | null;

  /**
   * Current focus mode. When mode is not 'normal', keybindings route to the
   * modal system (e.g., Escape closes the command palette) instead of resolving
   * against the active zone. Zone focus is preserved underneath — returning to
   * 'normal' mode restores the previous zone focus without flicker.
   */
  mode: FocusMode;

  /** Set focus to a zone. Called on click/keyboard navigation into a zone. */
  focusZone(zoneId: string): void;

  /** Set focus to a divider. Called when Tab-navigating to a divider. */
  focusDivider(dividerId: string): void;

  /** Enter a modal focus mode. The previous focus state is preserved. */
  pushMode(mode: FocusMode): void;

  /** Exit the current modal mode, restoring the previous focus state. */
  popMode(): void;

  /** Clear all focus (e.g., when clicking global chrome). */
  clearFocus(): void;

  /** Subscribe to focus changes. */
  subscribe(handler: (state: FocusState) => void): Unsubscriber;
}
```

### 19.2 Keybinding Resolution

When a keybinding is pressed, the command registry first checks the focus mode:

1. If `mode !== 'normal'` → route to the modal system. For example, `Escape` during `commandPalette` mode closes the palette and calls `popMode()`. All other keys are either consumed by the modal or ignored.
2. If `mode === 'normal'` → evaluate the command's `when` condition against the active zone/divider context. For example, `Ctrl+W` maps to `forge.editor.closeTab` when `activeZoneType == 'urd.codeEditor'`, but to `forge.zone.joinUp` when focus is on a divider.

Focus changes are published to the bus on channel `focus.zone.changed` so that views can react (e.g., a subtle highlight on the active zone's header bar).

---

## 20. Theme Engine

### 20.1 Token-Based System

All visual styling uses CSS custom properties (tokens). Two built-in themes.

```css
/* FRAMEWORK: token definitions */

:root[data-theme="gloaming"] {
  /* Surface */
  --forge-bg-primary: #1a1a2e;
  --forge-bg-secondary: #16213e;
  --forge-bg-tertiary: #0f3460;
  --forge-bg-zone-header: #1a1a2e;
  --forge-bg-zone-viewport: #0d1117;

  /* Text */
  --forge-text-primary: #e0e0e0;
  --forge-text-secondary: #a0a0b0;
  --forge-text-muted: #606070;

  /* Borders */
  --forge-border-zone: #2a2a4a;
  --forge-border-divider: #3a3a5a;

  /* Accents */
  --forge-accent-primary: #e94560;
  --forge-accent-secondary: #533483;
  --forge-accent-selection: rgba(233, 69, 96, 0.2);

  /* Semantic */
  --forge-status-error: #ff6b6b;
  --forge-status-warning: #ffa62b;
  --forge-status-info: #4ecdc4;
  --forge-status-success: #2ecc71;
  --forge-status-orphan-read: #ff6b6b;
  --forge-status-orphan-write: #ffa62b;
  --forge-status-healthy: #2ecc71;

  /* Graphs */
  --forge-graph-node-default: #2a2a4a;
  --forge-graph-edge-default: #4a4a6a;
  --forge-graph-node-selected: #e94560;

  /* Spreadsheets */
  --forge-table-row-alt: rgba(255, 255, 255, 0.02);
  --forge-table-row-hover: rgba(255, 255, 255, 0.05);
  --forge-table-header-bg: #1a1a3e;

  /* Focus — users must always see which zone is active and which divider has keyboard focus */
  --forge-focus-zone-ring: 1px solid #e94560;            /* active zone border highlight */
  --forge-focus-zone-glow: 0 0 0 1px rgba(233, 69, 96, 0.3); /* subtle outer glow */
  --forge-focus-divider-color: #e94560;                   /* divider highlight when keyboard-focused */
}

:root[data-theme="parchment"] {
  --forge-bg-primary: #faf8f5;
  --forge-bg-secondary: #f0ece4;
  /* ... light theme equivalents ... */
}
```

### 20.2 Semantic Typography and Spacing Tokens

Beyond colours, define semantic tokens for typography, spacing, and border radii. This prevents visual drift across 85+ views:

```css
:root {
  /* Typography */
  --forge-font-family-ui: 'Inter', -apple-system, sans-serif;
  --forge-font-family-mono: 'JetBrains Mono', 'Fira Code', monospace;
  --forge-font-size-xs: 10px;     /* zone headers, status bar */
  --forge-font-size-sm: 11px;     /* spreadsheet cells, graph labels */
  --forge-font-size-md: 13px;     /* default body text */
  --forge-font-size-lg: 15px;     /* inspector headings */
  --forge-font-weight-normal: 400;
  --forge-font-weight-medium: 500;
  --forge-font-weight-bold: 600;

  /* Spacing scale */
  --forge-space-xs: 2px;
  --forge-space-sm: 4px;
  --forge-space-md: 8px;
  --forge-space-lg: 12px;
  --forge-space-xl: 16px;
  --forge-space-2xl: 24px;

  /* Border radii */
  --forge-radius-sm: 2px;         /* buttons, inputs */
  --forge-radius-md: 4px;         /* cards, panels */
  --forge-radius-lg: 6px;         /* modals, dialogs */

  /* Z-indices */
  --forge-z-zone: 0;
  --forge-z-divider: 10;
  --forge-z-dropdown: 100;
  --forge-z-modal: 200;
  --forge-z-tooltip: 300;
}
```

All views reference these tokens. No raw pixel values, no inline font sizes. This is how 85+ views maintain a unified visual language.

### 20.3 Theme Switching

A command: `forge.theme.toggle` or `forge.theme.set`. Sets `data-theme` attribute on the root element. All components use token references, never raw colour values.

The theme system is entirely framework-level. Urd-specific views use the same tokens. The Code Editor (CodeMirror) maps Forge tokens to its own theme API.

---

## 21. Native Application Behaviour

Urd Forge runs in a webview but must feel indistinguishable from a native desktop application. Three behaviours are critical: fullscreen mode, controlled text selection, and custom context menus.

### 21.1 Fullscreen Mode

A command `forge.window.toggleFullscreen` (default: `F11`) toggles OS-level fullscreen via Tauri's window API. In fullscreen, the OS title bar and taskbar disappear entirely — only Forge chrome is visible (menu bar, workspace tabs, zones, status bar).

> **Fullscreen vs. Maximize Zone:** `forge.window.toggleFullscreen` (`F11`) is an OS-level window operation — the entire app goes fullscreen. `forge.zone.maximize` (`Ctrl+Space` or `Shift+Space`, like Blender) temporarily expands a single zone to fill the workspace area while keeping the menu bar, workspace tabs, and status bar visible. Both can be active simultaneously.

```typescript
// FRAMEWORK

commandRegistry.register({
  id: 'forge.window.toggleFullscreen',
  title: 'Toggle Fullscreen',
  category: 'Window',
  keybinding: 'F11',
  execute: async () => {
    const window = getCurrent();           // Tauri window handle
    const isFullscreen = await window.isFullscreen();
    await window.setFullscreen(!isFullscreen);
  },
});
```

**Tauri configuration** (`tauri.conf.json`):

```json
{
  "app": {
    "windows": [{
      "fullscreen": false,
      "resizable": true,
      "decorations": true
    }]
  }
}
```

Fullscreen is also available from the View menu and the command palette. The status bar shows a subtle fullscreen indicator icon when active.

### 21.2 Controlled Text Selection

Users must be able to select and copy text within content areas (code editor, spreadsheet cells, inspector values, log entries) but must **not** be able to drag-select across the entire application and reveal it as a webpage. The selection boundary is always a single zone's viewport.

**Implementation (framework-level CSS):**

```css
/* FRAMEWORK: base.css */

/* Default: suppress all text selection on application chrome */
* {
  -webkit-user-select: none;
  user-select: none;
}

/* Explicitly enable selection within content areas */
.forge-selectable {
  -webkit-user-select: text;
  user-select: text;
}

/* Form elements must always allow selection and input, regardless of global suppression */
input, textarea, [contenteditable="true"] {
  -webkit-user-select: text;
  user-select: text;
}

/* Prevent drag ghost images on all elements */
* {
  -webkit-user-drag: none;
}
```

The `.forge-selectable` class is applied to:
- CodeMirror editor containers (which manage their own selection internally)
- Spreadsheet cell text content
- Inspector value fields
- Log/output panel text
- Command palette input
- Any text the user would reasonably want to copy

It is **not** applied to:
- Zone headers
- Menu bar items
- Workspace tabs
- Toolbar buttons
- Dividers
- Status bar labels (except the status bar's info text area)

**Additional webview hardening** (Tauri config):

```json
{
  "app": {
    "windows": [{
      "dragDropEnabled": false
    }]
  }
}
```

This prevents the browser's native file drag-and-drop behaviour which would also break the application illusion. If file drag-and-drop is needed later (e.g., drag a `.urd.md` file into the editor), it should be implemented via Tauri's native drag-and-drop API, not the browser's.

### 21.3 Custom Context Menus

The browser's default right-click menu is suppressed globally. Every right-click triggers a context-sensitive custom menu that matches the application's visual design.

**Global suppression (framework-level):**

```typescript
// FRAMEWORK — in app initialization

document.addEventListener('contextmenu', (e) => {
  e.preventDefault();
});
```

**Context menu system:**

Each zone type, and certain interactive elements within zones, can register context menu providers. When the user right-clicks, the framework determines the click target, queries the relevant provider, and renders a custom menu.

```typescript
// FRAMEWORK

interface ContextMenuProvider {
  /** Returns menu items for a right-click at the given target, or null for no menu. */
  getItems(target: ContextMenuTarget): ContextMenuItem[] | null;
}

interface ContextMenuItem {
  label: string;
  commandId: string;                     // dispatched on click
  commandArgs?: Record<string, unknown>;
  icon?: string;
  keybinding?: string;                   // displayed as hint, not functional
  disabled?: boolean;
  separator?: boolean;                   // renders a divider line before this item
  children?: ContextMenuItem[];          // submenu
}

interface ContextMenuTarget {
  zoneId: string | null;                 // null if click was on global chrome
  zoneType: string | null;
  element: HTMLElement;                  // the actual DOM element right-clicked
  data?: Record<string, unknown>;        // custom data attributes from the element
}
```

**Registration pattern:**

```typescript
// APPLICATION — example: Entity Spreadsheet registers a context menu

contextMenuRegistry.register('urd.entitySpreadsheet', {
  getItems: (target) => {
    const entityId = target.data?.entityId;
    if (!entityId) return null;
    return [
      { label: 'Go to Definition',      commandId: 'urd.navigate.goToDefinition', commandArgs: { entityId }, keybinding: 'F12' },
      { label: 'Find All References',   commandId: 'urd.navigate.findReferences', commandArgs: { entityId } },
      { separator: true,                label: '', commandId: '' },
      { label: 'Copy Entity ID',        commandId: 'forge.clipboard.copy', commandArgs: { text: `@${entityId}` } },
      { separator: true,                label: '', commandId: '' },
      { label: 'Filter to This Type',   commandId: 'urd.filter.byEntityType', commandArgs: { entityId } },
    ];
  }
});
```

**Framework-level context menus (always available):**

| Click Target | Menu Items |
|-------------|------------|
| Zone header | Change Type ▸, Split Horizontal, Split Vertical, Maximize/Restore |
| Zone viewport (no view-specific menu) | Change Type ▸, Split Horizontal, Split Vertical |
| Divider | Join Left/Right/Up/Down, Swap, Split Horizontal, Split Vertical (as documented in 5.6) |
| Workspace tab | Rename, Duplicate, Delete, New from Template ▸ |
| Status bar | Toggle Fullscreen, Toggle Theme |

**Visual design:** Context menus use framework theme tokens (`--forge-bg-secondary`, `--forge-text-primary`, `--forge-border-zone`) and appear immediately at the cursor position. They dismiss on click-outside, Escape, or item selection. Submenus expand on hover with a small delay. The menu component is `ContextMenu.svelte` in the framework layer.

**View discovery at scale.** With 85+ registered view types, the "Change Type ▸" submenu does not list all views. It shows the 8 most recently used types for this zone, grouped by category, plus a "More…" item that opens the Command Palette pre-filtered to `Change View Type:`. The Command Palette provides fuzzy search across all registered view types. This keeps the common case fast (one click) and the rare case discoverable (two clicks + typing).

### 21.4 Additional Webview Hardening

A few more browser behaviours to suppress for a native feel:

```css
/* FRAMEWORK: base.css */

/* Prevent rubber-band overscroll on macOS */
html, body {
  overscroll-behavior: none;
}

/* Prevent text highlight colour leaking OS theming */
::selection {
  background: var(--forge-accent-selection);
  color: var(--forge-text-primary);
}

/* Prevent focus outlines that look browser-like (replace with custom) */
*:focus {
  outline: none;
}
*:focus-visible {
  outline: 1px solid var(--forge-accent-primary);
  outline-offset: -1px;
}
```

**Keyboard sovereignty — prevent browser shortcuts from leaking through:**

```typescript
// FRAMEWORK — suppress browser shortcuts that conflict

const SUPPRESSED_BROWSER_SHORTCUTS = [
  'ctrl+g',       // "Find next" in browsers — conflicts with Go To Line
  'ctrl+h',       // "History" in browsers — conflicts with Find & Replace
  'ctrl+l',       // "Address bar" in browsers — conflicts with Go To Line
  'ctrl+d',       // "Bookmark" in browsers — conflicts with editor actions
  'ctrl+shift+i', // "DevTools" — suppress in production, allow in dev
  'ctrl+u',       // "View source" — must never be accessible
  'ctrl+p',       // "Print" — override with Command Palette
  'f5',           // "Refresh" — override with Recompile or suppress
  'f7',           // "Caret browsing" in some browsers — suppress
];

document.addEventListener('keydown', (e) => {
  const insideEditor = (e.target as HTMLElement)?.closest('.cm-editor') !== null;
  const key = formatKeybinding(e);

  // Step 1: Always suppress browser shortcuts regardless of context.
  if (SUPPRESSED_BROWSER_SHORTCUTS.includes(key)) {
    e.preventDefault();
  }

  // Step 2: Routing policy depends on whether we're inside CodeMirror.
  if (insideEditor) {
    // Inside editor: ONLY dispatch commands explicitly marked as globalWhenEditorFocused.
    // Examples: Ctrl+Shift+P (command palette), Ctrl+, (settings), F11 (fullscreen).
    // Everything else is left to CodeMirror — including Ctrl+D, Ctrl+Shift+K, etc.
    // This prevents double-routing that would break multi-cursor, selection, and editor chords.
    const command = commandRegistry.resolveByKeybinding(key);
    if (command?.globalWhenEditorFocused) {
      commandRegistry.execute(command.id);
    }
    // If no global command matched, CodeMirror handles it (or it's a no-op).
  } else {
    // Outside editor: full command registry resolution.
    commandRegistry.executeByKeybinding(key);
  }
});
```

Commands like `forge.commandPalette.open`, `forge.settings.open`, `forge.window.fullscreen`, and `forge.project.open` set `globalWhenEditorFocused: true` (see `CommandRegistration` in §10). Editor-specific commands like `forge.editor.closeTab` do NOT — they register with CodeMirror's keymap instead.

This ensures browser shortcuts are always suppressed, CodeMirror retains full keyboard control for editor chords, and a small whitelist of framework-level commands still works from inside the editor.

**Chorded shortcuts (e.g., `Ctrl+K Ctrl+S`).** VSCode-style key chords require a state machine in the command registry: after the first key (`Ctrl+K`), the system enters a "chord pending" state and waits for the second key. During this window, CodeMirror must not consume the second key. Implementation: when a chord prefix is detected, the command registry temporarily captures all keyboard input (including from CodeMirror) until the chord completes or times out (~1s). This is Phase 3 implementation work — for Phase 1 and 2, single-key shortcuts are sufficient.

### 21.5 Application Startup and Loading Experience

The startup sequence has two phases with different solutions.

**Phase A — Webview cold start (0–500ms).** A native OS splash window (not HTML) is shown immediately on launch while the Tauri webview initialises behind it. This uses Tauri's splashscreen plugin to avoid the flash of white/empty that would reveal the app is web-based.

```json
// tauri.conf.json
{
  "app": {
    "windows": [
      {
        "label": "splashscreen",
        "url": "splashscreen.html",
        "width": 480,
        "height": 320,
        "decorations": false,
        "resizable": false,
        "center": true,
        "alwaysOnTop": true
      },
      {
        "label": "main",
        "visible": false,
        "decorations": true,
        "width": 1400,
        "height": 900
      }
    ]
  }
}
```

The splash window shows the Urd Forge logo and a minimal progress indicator. The Svelte app signals readiness once the framework shell has mounted (view registry populated, workspace loaded, theme applied). At that point, the splash closes and the main window becomes visible. This transition should take under 500ms.

```typescript
// FRAMEWORK — in App.svelte onMount

import { getCurrentWindow } from '@tauri-apps/api/window';

onMount(async () => {
  // Framework shell is ready — close splash, show main window
  const splash = await getWindow('splashscreen');
  const main = getCurrentWindow();
  await main.show();
  await splash.close();
});
```

**Phase B — Initial compilation (500ms–3s).** The UI shell renders immediately with the full workspace layout. Zones that depend on compiler data show a view-specific loading state while the first compilation runs in the background.

```
┌─ Code Editor Zone ─────────────────┐ ┌─ Entity Spreadsheet ──────────┐
│ [world.urd.md ×]                   │ │                               │
│                                    │ │   ◌ Compiling world...        │
│ // File content is available       │ │                               │
│ // immediately. Full editing works │ │   (project: 47 files)         │
│ // before compilation finishes.    │ │                               │
│                                    │ │                               │
└────────────────────────────────────┘ └───────────────────────────────┘
┌─ Location Graph ───────────────────────────────────────────────────────┐
│                                                                        │
│                        ◌ Compiling world...                            │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
[═══════════════════════════░░░░░░░░░░░░] Compiling... (PARSE → LINK)
```

Key behaviours during Phase B:
- The **Code Editor** is fully functional immediately. It loads files from the buffer map and provides syntax highlighting (which doesn't require compilation). Code lens, diagnostics, and hover tooltips appear once compilation completes.
- **Spreadsheets, graphs, inspectors** show a themed loading state with a subtle spinner and the compilation phase currently running (if available from progress events).
- The **Global Status Bar** shows compilation progress (phase names and optional progress percentage if the backend streams progress events).
- Once compilation completes, all views populate simultaneously via the bus. The transition from loading state to data should feel like a single coordinated reveal.

This approach is preferable to a Blender-style blocking splash with a progress bar because the user sees their workspace immediately, can begin orienting and even editing, and the data "fills in" within 1–2 seconds. It feels fast even when compilation is slow.

**Zone loading state component:**

```typescript
// FRAMEWORK

// A standard loading state component used by any view that depends on async data.
// Shows a themed spinner and optional status message.
// The framework provides this so all views have a consistent loading appearance.
// Views render this when their required bus channel has not yet received data.
<ZoneLoadingState message="Compiling world..." detail="47 files" />
```

### 21.6 Security

Urd Forge is a local-only desktop app with no network surface, but Tauri apps still need to guard against unintended filesystem access and command injection.

**Tauri allowlist.** The `tauri.conf.json` allowlist restricts IPC commands to the minimum required set: file read/write within the project directory, compiler invocation, and settings persistence. Shell command execution is disabled entirely — the compiler runs as a Rust function inside the Tauri process, not as a spawned subprocess.

**Path validation.** All file paths received from the frontend are validated in the Rust backend before any filesystem operation. Paths must resolve to within the project root or the app config directory. Path traversal attempts (`../`, symlink escapes) are rejected with an error. The frontend never constructs raw filesystem paths — it uses the `FileSystem` abstraction (§15) which delegates to validated Rust commands.

**No remote content.** The webview loads only local assets bundled with the app. No external URLs, no CDN scripts, no remote fonts. CSP headers are set to `default-src 'self'` in the Tauri configuration.

---

## 22. Project Directory Structure

```
urd-forge/
├── src-tauri/                          # Rust backend
│   ├── src/
│   │   ├── main.rs                     # Tauri entry point
│   │   ├── commands/                   # Tauri IPC command handlers
│   │   │   ├── compile.rs              # compile(), recompile()
│   │   │   ├── analysis.rs             # run_analysis(), simulate()
│   │   │   ├── filesystem.rs           # file operations + watch bridge
│   │   │   └── mod.rs
│   │   └── compiler/                   # Urd compiler integration
│   │       ├── bridge.rs               # Wraps urd-compiler crate for IPC serialization
│   │       └── mod.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                                # Svelte frontend
│   ├── lib/
│   │   ├── framework/                  # === GENERIC FRAMEWORK (no Urd imports) ===
│   │   │   ├── layout/
│   │   │   │   ├── ZoneTree.ts         # BSP tree data structure + operations
│   │   │   │   ├── ZoneRenderer.svelte # Recursive layout component
│   │   │   │   ├── SplitContainer.svelte
│   │   │   │   ├── ZoneShell.svelte    # Header + viewport wrapper
│   │   │   │   ├── ZoneHeader.svelte   # Type selector dropdown + toolbar slot
│   │   │   │   ├── Divider.svelte      # Draggable divider with right-click context menu (join, swap, split)
│   │   │   │   ├── DividerContextMenu.svelte  # Right-click menu for divider operations
│   │   │   │   ├── MergeOverlay.svelte # Arrow overlay shown during drag-to-merge gesture
│   │   │   │   ├── GlobalStatusBar.svelte  # Always-visible bottom bar (compiler status, selection hint, stats)
│   │   │   │   ├── ZoneLoadingState.svelte # Standard loading/spinner state for views awaiting async data
│   │   │   │   ├── ZoneErrorBoundary.svelte # Catches view throws, shows crash panel with reload option
│   │   │   │   └── test-helpers.ts     # createTestTree(), assertTreeInvariants()
│   │   │   ├── menu/
│   │   │   │   ├── GlobalMenuBar.svelte    # Top-level menu bar (File, Edit, View, Window, Help)
│   │   │   │   ├── MenuDropdown.svelte     # Individual dropdown menu
│   │   │   │   ├── MenuItem.svelte         # Single menu item (maps to a command)
│   │   │   │   ├── MenuSeparator.svelte    # Group separator within a dropdown
│   │   │   │   └── MenuRegistry.ts         # Contribution-based menu population
│   │   │   ├── context-menu/
│   │   │   │   ├── ContextMenu.svelte          # Rendered menu (positioned at cursor)
│   │   │   │   ├── ContextMenuProvider.ts      # Provider interface + registry
│   │   │   │   └── contextMenuSuppressor.ts    # Global right-click suppression + routing
│   │   │   ├── placeholders/
│   │   │   │   ├── PlaceholderInfo.svelte      # Shows zone metadata (ID, type, dimensions)
│   │   │   │   ├── PlaceholderColour.svelte    # Solid colour per zone for visual layout testing
│   │   │   │   ├── PlaceholderBusMonitor.svelte # Live log of all message bus events
│   │   │   │   └── PlaceholderCommandLog.svelte # Live log of command executions
│   │   │   ├── commands/
│   │   │   │   ├── CommandRegistry.ts
│   │   │   │   ├── CommandPalette.svelte
│   │   │   │   ├── KeybindingManager.ts
│   │   │   │   └── test-helpers.ts     # TestCommandRegistry: records executions and undo actions
│   │   │   ├── bus/
│   │   │   │   ├── MessageBus.ts       # Channel-registered pub/sub bus
│   │   │   │   ├── ChannelManifest.ts  # Framework-level channel definitions
│   │   │   │   ├── useBusChannel.ts    # Svelte adapter: auto-subscribe/unsubscribe with last-value replay
│   │   │   │   └── test-helpers.ts     # TestBus: records publishes, asserts payload size
│   │   │   ├── views/
│   │   │   │   ├── ViewRegistry.ts
│   │   │   │   └── ViewHostContract.ts  # ViewToolbarProvider + ViewStatusProvider interfaces
│   │   │   ├── workspace/
│   │   │   │   ├── WorkspaceManager.ts
│   │   │   │   ├── WorkspaceTabs.svelte
│   │   │   │   └── types.ts
│   │   │   ├── navigation/
│   │   │   │   └── NavigationBroker.ts
│   │   │   ├── selection/
│   │   │   │   └── SelectionContext.ts
│   │   │   ├── focus/
│   │   │   │   └── FocusService.ts     # Tracks active zone, divider, modality
│   │   │   ├── theme/
│   │   │   │   ├── ThemeEngine.ts
│   │   │   │   ├── tokens.css          # Token definitions for all themes
│   │   │   │   └── base.css            # Reset, framework base styles, user-select suppression, overscroll, focus styles
│   │   │   ├── filesystem/
│   │   │   │   └── FileSystem.ts       # Interface definition
│   │   │   ├── project/
│   │   │   │   ├── ProjectManager.ts   # Open, close, recent projects
│   │   │   │   └── WelcomeScreen.svelte # Shown when no project is open
│   │   │   ├── settings/
│   │   │   │   ├── AppSettingsService.ts # Global settings persistence
│   │   │   │   ├── SettingsView.svelte  # Zone type: categorised settings editor
│   │   │   │   └── KeybindingEditor.svelte # Keybinding capture + conflict detection
│   │   │   └── types.ts                # Shared framework types
│   │   │
│   │   └── app/                        # === URD APPLICATION (imports framework) ===
│   │       ├── compiler/
│   │       │   ├── CompilerService.ts   # Interface
│   │       │   ├── TauriCompiler.ts     # Tauri IPC implementation
│   │       │   ├── MockCompilerService.ts # Mock implementation using fixture data (no IPC)
│   │       │   ├── CompilerOutputCache.ts # Chunk hash resolution, reuses unchanged chunks
│   │       │   ├── BufferMap.ts         # In-memory file buffers (editor writes here, compiler reads here)
│   │       │   ├── types.ts             # CompilerOutput, FactSet, SymbolTable, Chunk<T>, ChunkName, etc.
│   │       │   └── channels.ts          # Urd-specific bus channel definitions
│   │       ├── projections/
│   │       │   ├── ProjectionRegistry.ts  # Memoised selector registry
│   │       │   ├── entity-table.ts        # urd.projection.entityTable
│   │       │   ├── type-table.ts          # urd.projection.typeTable
│   │       │   ├── property-table.ts      # urd.projection.propertyTable
│   │       │   ├── location-graph.ts      # urd.projection.locationGraph
│   │       │   ├── dialogue-graph.ts      # urd.projection.dialogueGraph
│   │       │   ├── diagnostic-list.ts     # urd.projection.diagnosticList
│   │       │   ├── diagnostics-by-file.ts # urd.projection.diagnosticsByFile (file browser indicators)
│   │       │   └── index.ts              # Registers all projections
│   │       ├── filesystem/
│   │       │   └── TauriFileSystem.ts   # ForgeFileSystem implementation
│   │       ├── selection/
│   │       │   ├── UrdSelectionTypes.ts # SelectionItem union type
│   │       │   └── UrdFilterTypes.ts    # FilterItem union type
│   │       ├── views/
│   │       │   ├── editor/
│   │       │   │   ├── CodeEditorZone.svelte
│   │       │   │   ├── TabBar.svelte
│   │       │   │   ├── Breadcrumb.svelte
│   │       │   │   ├── EditorPane.svelte       # CodeMirror wrapper
│   │       │   │   ├── urd-language.ts          # CodeMirror language mode
│   │       │   │   ├── urd-code-lens.ts         # Code lens extension
│   │       │   │   ├── urd-gutter.ts            # Gutter annotations
│   │       │   │   └── urd-inline-annotations.ts
│   │       │   ├── source-viewer/
│   │       │   │   └── SourceViewerZone.svelte
│   │       │   ├── file-browser/
│   │       │   │   ├── FileBrowser.svelte          # Zone type: project tree with lazy expansion
│   │       │   │   ├── FileTreeNode.svelte         # Single tree row (file or directory)
│   │       │   │   └── InlineRenameInput.svelte    # Inline rename field (F2)
│   │       │   ├── spreadsheets/
│   │       │   │   ├── _shared/
│   │       │   │   │   ├── VirtualTable.svelte  # Shared virtualized table
│   │       │   │   │   ├── ColumnHeader.svelte
│   │       │   │   │   ├── SortControls.svelte
│   │       │   │   │   └── FilterBar.svelte
│   │       │   │   ├── EntitySpreadsheet.svelte
│   │       │   │   ├── TypeSpreadsheet.svelte
│   │       │   │   ├── PropertySpreadsheet.svelte
│   │       │   │   ├── LocationSpreadsheet.svelte
│   │       │   │   ├── SectionSpreadsheet.svelte
│   │       │   │   ├── ChoiceSpreadsheet.svelte
│   │       │   │   ├── RuleSpreadsheet.svelte
│   │       │   │   ├── ExitSpreadsheet.svelte
│   │       │   │   ├── JumpSpreadsheet.svelte
│   │       │   │   ├── ReadSpreadsheet.svelte
│   │       │   │   ├── WriteSpreadsheet.svelte
│   │       │   │   ├── SequenceSpreadsheet.svelte
│   │       │   │   ├── DiagnosticSpreadsheet.svelte
│   │       │   │   └── FileSpreadsheet.svelte
│   │       │   ├── graphs/
│   │       │   │   ├── _shared/
│   │       │   │   │   ├── GraphCanvas.svelte   # Shared pan/zoom/minimap canvas
│   │       │   │   │   ├── GraphNode.svelte
│   │       │   │   │   ├── GraphEdge.svelte
│   │       │   │   │   └── spatial-index.ts     # Viewport culling
│   │       │   │   ├── LocationGraph.svelte
│   │       │   │   ├── DialogueFlowGraph.svelte
│   │       │   │   ├── TypeHierarchyGraph.svelte
│   │       │   │   ├── PropertyDataFlowGraph.svelte
│   │       │   │   ├── FileDependencyGraph.svelte
│   │       │   │   ├── RuleTriggerNetwork.svelte
│   │       │   │   ├── ContainmentTree.svelte
│   │       │   │   ├── SequenceTimeline.svelte
│   │       │   │   ├── ChoiceTree.svelte
│   │       │   │   └── CrossReferenceGraph.svelte
│   │       │   ├── inspectors/
│   │       │   │   ├── _shared/
│   │       │   │   │   ├── InspectorPanel.svelte
│   │       │   │   │   └── InspectorSection.svelte
│   │       │   │   ├── EntityInspector.svelte
│   │       │   │   ├── TypeInspector.svelte
│   │       │   │   ├── LocationInspector.svelte
│   │       │   │   ├── SectionInspector.svelte
│   │       │   │   ├── RuleInspector.svelte
│   │       │   │   ├── PropertyInspector.svelte
│   │       │   │   └── DiagnosticInspector.svelte
│   │       │   ├── analysis/
│   │       │   │   ├── ReachabilityMatrix.svelte
│   │       │   │   ├── ConditionEffectMatrix.svelte
│   │       │   │   ├── EnumCoverage.svelte
│   │       │   │   ├── ThresholdAnalysis.svelte
│   │       │   │   ├── VisibilityAudit.svelte
│   │       │   │   ├── CircularDependency.svelte
│   │       │   │   ├── DeadCodePanel.svelte
│   │       │   │   ├── WorldStatsDashboard.svelte
│   │       │   │   ├── DiffView.svelte
│   │       │   │   └── NarrativeFlowVisualiser.svelte
│   │       │   ├── search/
│   │       │   │   ├── GlobalSymbolSearch.svelte
│   │       │   │   ├── FindReferences.svelte
│   │       │   │   ├── WhereUsed.svelte
│   │       │   │   ├── WhatIf.svelte
│   │       │   │   ├── ImpactAnalysis.svelte
│   │       │   │   ├── RegexSearch.svelte
│   │       │   │   ├── PropertyValueSearch.svelte
│   │       │   │   └── ConditionPredicateSearch.svelte
│   │       │   ├── runtime/
│   │       │   │   ├── PlayPanel.svelte
│   │       │   │   ├── StateInspector.svelte
│   │       │   │   ├── EventLog.svelte
│   │       │   │   ├── BreadcrumbTrail.svelte
│   │       │   │   ├── CoverageOverlay.svelte
│   │       │   │   ├── MonteCarloDashboard.svelte
│   │       │   │   └── DeterministicReplay.svelte
│   │       │   └── specialised/
│   │       │       ├── WorldMap.svelte
│   │       │       ├── PropertyTimeline.svelte
│   │       │       ├── DialoguePreview.svelte
│   │       │       ├── DependencyHeatmap.svelte
│   │       │       ├── CompilationPipeline.svelte
│   │       │       ├── SchemaBrowser.svelte
│   │       │       ├── TestReportDashboard.svelte
│   │       │       ├── ComparisonView.svelte
│   │       │       ├── AnnotationLayer.svelte
│   │       │       └── AiAssistantPanel.svelte
│   │       ├── workspaces/
│   │       │   └── templates.ts         # Predefined workspace layouts
│   │       ├── commands/
│   │       │   ├── editor-commands.ts   # Urd editor commands
│   │       │   ├── compiler-commands.ts
│   │       │   ├── project-commands.ts  # Open, close, recent projects
│   │       │   ├── file-commands.ts     # New file, rename, delete, reveal in OS
│   │       │   ├── navigation-commands.ts
│   │       │   ├── playback-commands.ts
│   │       │   └── analysis-commands.ts
│   │       └── bootstrap.ts             # Registers all Urd views, commands, workspaces
│   │
│   ├── routes/
│   │   └── +page.svelte                 # Mounts <Workspace /> as root
│   ├── app.css                          # Imports framework tokens + base
│   └── app.html
│
├── static/
│   ├── icons/                           # Zone type icons, toolbar icons
│   └── splashscreen.html               # Native splash window (logo + loading indicator)
│
├── fixtures/                            # Test fixture data (valid CompilerOutput JSON blobs)
│   ├── basic-world.json                # Smoke test: ~20 entities
│   ├── medium-world.json               # Realistic: ~500 entities, 50 locations
│   ├── empty-world.json                # Edge case: fresh project, no content
│   ├── error-world.json                # Edge case: all chunks have errors
│   ├── stress-50k-entities.json        # Performance: 50k entities, 500 types, 200 locations
│   └── stress-huge-diagnostics.json    # Performance: 10k diagnostics, 5k cross-refs
│
├── scripts/
│   └── generate-fixture.ts             # Generates valid CompilerOutput fixtures from parameters
│
├── ARCHITECTURE.md                      # This document
├── CLAUDE.md                            # AI implementation conventions (read by Claude Code every session)
├── STATUS.md                            # Session handoff state (updated at end of each Claude Code session)
├── package.json
├── svelte.config.js
├── eslint.config.js                     # Includes import boundary enforcement
├── vite.config.ts
├── tsconfig.json
└── pnpm-lock.yaml
```

---

## 23. Bootstrap Sequence

Application startup in `bootstrap.ts`:

```typescript
// APPLICATION

import { viewRegistry } from '$lib/framework/views/ViewRegistry';
import { commandRegistry } from '$lib/framework/commands/CommandRegistry';
import { workspaceManager } from '$lib/framework/workspace/WorkspaceManager';
import { projectManager } from '$lib/framework/project/ProjectManager';
import { appSettings } from '$lib/framework/settings/AppSettingsService';
import { bus } from '$lib/framework/bus/MessageBus';

// 1. Load application settings (must be first — theme, font, keybindings depend on it)
await appSettings.load();  // corruption-safe: backs up and loads defaults on failure

// 2. Register Urd-specific bus channels
import { registerUrdChannels } from './compiler/channels';
registerUrdChannels(bus);

// 3. Register all view types (with capabilities and state versioning)

// Framework-level views
viewRegistry.register({
  id: 'forge.view.settings',
  name: 'Settings',
  icon: 'settings',
  category: 'System',
  component: () => import('$lib/framework/settings/SettingsView.svelte'),
  navigationStrategy: 'singleton',
  requiresProject: false,
  capabilities: { supportsFilter: true },
  stateVersion: 1,
  defaultState: { activeCategory: 'appearance', searchQuery: '' },
});

// Application views
viewRegistry.register({
  id: 'urd.codeEditor',
  name: 'Code Editor',
  icon: 'code',
  category: 'Editor',
  component: () => import('./views/editor/CodeEditorZone.svelte'),
  navigationStrategy: 'singleton-autocreate',
  capabilities: { supportsSelection: true, supportsNavigationTarget: true },
  stateVersion: 1,
  defaultState: { openTabs: [], activeTab: null },
});

viewRegistry.register({
  id: 'urd.view.fileBrowser',
  name: 'File Browser',
  icon: 'folder-tree',
  category: 'Navigation',
  component: () => import('./views/file-browser/FileBrowser.svelte'),
  navigationStrategy: 'multi',
  requiresProject: true,
  capabilities: { supportsSelection: true, supportsNavigationTarget: true },
  stateVersion: 1,
  defaultState: { expandedPaths: [], sortBy: 'name', showHiddenFiles: false },
});

viewRegistry.register({
  id: 'urd.entitySpreadsheet',
  name: 'Entity Spreadsheet',
  icon: 'table',
  category: 'Spreadsheets',
  component: () => import('./views/spreadsheets/EntitySpreadsheet.svelte'),
  capabilities: { supportsSelection: true, supportsFilter: true, exportFormats: ['csv'] },
  stateVersion: 1,
  defaultState: { sortColumn: 'id', sortDirection: 'asc', columnWidths: {} },
});

// ... 85+ more registrations (or loop over a manifest)

// 4. Register all commands
import { registerEditorCommands } from './commands/editor-commands';
import { registerCompilerCommands } from './commands/compiler-commands';
import { registerProjectCommands } from './commands/project-commands';
import { registerFileCommands } from './commands/file-commands';
registerEditorCommands(commandRegistry);
registerCompilerCommands(commandRegistry);
registerProjectCommands(commandRegistry, projectManager);
registerFileCommands(commandRegistry);
// ...

// 5. Register workspace templates
import { WORKSPACE_TEMPLATES } from './workspaces/templates';
for (const [name, template] of Object.entries(WORKSPACE_TEMPLATES)) {
  workspaceManager.registerTemplate(name, template);
}

// 6. Register projections
import { registerAllProjections } from './projections';
registerAllProjections(projectionRegistry);

// 7. Initialize compiler service + buffer map + output cache
import { TauriCompiler } from './compiler/TauriCompiler';
import { BufferMap } from './compiler/BufferMap';
import { CompilerOutputCache } from './compiler/CompilerOutputCache';

const compiler = new TauriCompiler();
const bufferMap = new BufferMap();
const outputCache = new CompilerOutputCache();

// CompileSignal — the lightweight notification published on 'compiler.completed'.
// Views use this to trigger re-evaluation. Actual data is pulled from ProjectionRegistry.
interface CompileSignal {
  compileId: string;
  header: OutputHeader;
  chunkHashes: Record<string, string>;
}

// 8. Set up recompile-on-change pipeline (buffer-driven, not filesystem-driven)
//
// Back-pressure policy: "latest wins". If the user types faster than the compiler
// can run, stale results are discarded. The debounce prevents rapid-fire IPC calls,
// and the compileId check ensures only the most recent result is applied.
let latestCompileId = 0;

bufferMap.onAnyChange(debounce(async () => {
  const thisCompileId = ++latestCompileId;
  bus.publish('compiler.started', undefined);
  const rawOutput = await compiler.compile(bufferMap.getAll());

  // Stale result — a newer compile was triggered while this one was running. Discard.
  if (thisCompileId !== latestCompileId) return;

  const resolved = outputCache.resolve(rawOutput);
  projectionRegistry.updateSource(resolved);  // projections recompute lazily on next get()
  bus.publish('compiler.completed', {          // signal only — no data payload
    compileId: resolved.header.compileId,
    header: resolved.header,
    chunkHashes: Object.fromEntries(
      Object.entries(rawOutput.chunks).map(([k, v]) => [k, v.hash])
    ),
  } satisfies CompileSignal);
}, 300));

// 9. File watcher handles EXTERNAL edits only (another editor, git checkout, etc.)
bus.subscribe('fs.file.changed', async ({ path }) => {
  if (path.endsWith('.urd.md') && !bufferMap.isDirty(path)) {
    // External change — update buffer from disk
    const content = await fileSystem.readFile(path);
    bufferMap.set(path, content);  // triggers recompile via step 8
  }
});

// 10. Open last project or show Welcome screen
if (appSettings.get('openLastProjectOnLaunch') && appSettings.get('lastProjectPath')) {
  try {
    await projectManager.openPath(appSettings.get('lastProjectPath')!);
    // projectManager.openPath() handles:
    //   - Loading workspace from <projectRoot>/.forge/workspaces.json
    //   - Scanning directory → populating bufferMap
    //   - Starting file watcher
    //   - Triggering initial compile
  } catch {
    // Last project path no longer exists — show Welcome screen
    projectManager.showWelcome();
  }
} else {
  projectManager.showWelcome();
}
```

---

## 24. Performance Strategy

### 24.1 Frontend Rendering

- **Virtualized tables**: All spreadsheet views render only visible rows. The shared `VirtualTable.svelte` component handles this.
- **Spatial culling for graphs**: Graph views use a spatial index (quadtree or R-tree) to render only nodes/edges within the viewport.
- **Lazy view loading**: View components are loaded via dynamic `import()` only when a zone is set to that type for the first time.
- **Svelte 5 fine-grained reactivity**: `$state` and `$derived` runes ensure only the affected DOM nodes update when compiler output changes.
- **Zone-level render isolation**: Each zone is a separate Svelte component tree. A rerender in one zone cannot cascade to a sibling.
- **Projection memoisation**: Projections only recompute when their input chunk hash changes. Multiple views sharing a projection get the same object by reference.

### 24.2 Rust Backend

- **Incremental compilation**: If the Urd compiler supports it, only reparse changed files and re-link affected symbols. If not, full recompile with the 300ms debounce is the floor.
- **Heavy analysis on demand**: Monte Carlo, reachability matrix, and other O(n²)+ operations are triggered by explicit user action, not on every recompile.
- **Parallel analysis**: Heavy analysis commands can run on separate threads in Rust, streaming progress events via Tauri events.
- **Chunked output with hashes**: Only changed chunks are serialized and parsed, avoiding redundant work on every recompile.

### 24.3 IPC

- **Serialization format**: `serde_json` for CompilerOutput chunks. If profiling shows JSON parse is slow, switch to `postcard` (binary) or `MessagePack`.
- **Streaming for large results**: Monte Carlo streams progress events rather than returning one giant payload.
- **Hash-based skip**: Unchanged chunks send only a hash (< 100 bytes), not the full data.
- **Escape hatch for extreme data density**: If JSON serialization becomes the IPC ceiling (e.g., multi-MB FactSets on stress-50k), the transport can be replaced with shared memory (Tauri supports raw byte passing via `tauri::ipc::Channel`) or a zero-copy binary format like FlatBuffers. This is a transport optimization behind the `CompilerService` interface — no architecture changes required, only the Rust serializer and the TS `CompilerOutputCache.resolve()` method change.

### 24.4 Stress Test Fixtures

Two synthetic world fixtures are maintained from Phase 1 onward to validate performance assumptions at scale:

| Fixture | Shape | Purpose |
|---------|-------|---------|
| **stress-50k-entities** | 50,000 entities across 500 types, 200 locations, 10,000 properties | Validates table virtualisation, projection memoisation, and IPC payload size |
| **stress-huge-diagnostics** | 10,000 diagnostics across 100 files, plus 5,000 cross-references | Validates diagnostic spreadsheet scrolling, graph culling, and CodeMirror annotation performance |

These fixtures are JSON files that mock `CompilerOutput`. They do not require the real Urd compiler to run. Views load them directly via a `forge.debug.loadStressFixture` command for profiling.

**Performance acceptance criteria:**
- Spreadsheet with 50k rows: first render < 100ms, scroll at 60fps
- Graph with 200 nodes / 1000 edges: render < 200ms, pan/zoom at 60fps
- Full CompilerOutput IPC round-trip for stress-50k: < 500ms
- Theme switch: < 50ms (no visible flash)

**Architecture litmus test (must pass before Phase 5):** With the mock compiler loaded with `stress-50k-entities`, type at 10 keystrokes per second for 30 seconds while simultaneously scrolling a 50k-row Entity Spreadsheet and panning the Location Network Graph. The app must remain responsive throughout — no frame drops below 30fps, no input lag exceeding 100ms, no stale data visible in any view. If this test fails, the core architecture has a problem that must be fixed before building more views.

---

## 25. Event Flow Examples

### 25.1 User Edits a File

```
User types in Code Editor
  → CodeMirror dispatches transaction
  → BufferMap.set(filePath, content)           // in-memory, instant
  → 300ms debounce elapses
  → bus.publish('compiler.started')
  → CompilerService.compile(bufferMap.getAll())
  → Rust: parse → import → link → analyze → emit
  → IPC response: CompilerOutput with chunked data + hashes
  → CompilerOutputCache.resolve() — reuses unchanged chunks
  → projectionRegistry.updateSource(resolved) — projections recompute lazily on next access
  → bus.publish('compiler.completed', { compileId, header, chunkHashes })  // signal only
  → Views react to signal, pull data from projectionRegistry.get()
  → Entity Spreadsheet: projectionRegistry.get('urd.projection.entityTable') — cache hit if symbolTable unchanged
  → Location Graph: projectionRegistry.get('urd.projection.locationGraph') — cache hit if factSet unchanged
  → Diagnostic Spreadsheet reads diagnostics chunk (always updated)
  → Code Editor updates inline diagnostics, code lens, gutter icons
```

### 25.2 User Clicks an Entity in a Spreadsheet

```
User clicks row for @warden in Entity Spreadsheet
  → EntitySpreadsheet calls selectionContext.select({ kind: 'entity', id: 'warden' })
  → bus.publish('selection.primary', { kind: 'entity', id: 'warden' })
  → Location Graph highlights the node containing @warden
  → Entity Inspector switches to show @warden details
  → Code Editor scrolls to @warden's declaration and flashes the line
  → Property Spreadsheet filters to show only Guard properties (if @warden is a Guard)
```

### 25.3 User Clicks "Go to Definition" from a Spreadsheet

```
User clicks a source span link in the Property Spreadsheet
  → PropertySpreadsheet dispatches NavigationIntent:
      { targetZoneType: 'urd.codeEditor', targetResource: 'gatehouse.urd.md',
        targetLocation: { line: 47, column: 4 }, sourceZoneId: 'zone_b2' }
  → NavigationBroker receives intent
  → Code Editor singleton exists → check tabs
  → File not open → open new tab for gatehouse.urd.md
  → Scroll to line 47, flash highlight
  → Focus Code Editor zone
```

---

## 26. Non-Goals for v1

These are explicitly out of scope. The framework is designed so they *could* be added later, but building towards them now would be premature.

1. **No plugin system.** Views, commands, and projections are registered internally at startup. No runtime extension loading, no third-party API surface.
2. **No global undo stack.** Commands return `UndoAction` hooks from day one, but a unified undo/redo stack that consumes them is a future feature. CodeMirror handles editor undo independently.
3. **No collaborative editing.** Single-user, single-instance.
4. **No multi-window.** One Tauri window, one webview, one BSP tree per workspace.
5. **No WCAG accessibility.** Keyboard navigation works via the command system and CodeMirror, but screen reader support, ARIA roles, and high-contrast themes are deferred. The theme engine's token system and command-first architecture make these retrofittable without structural changes.

---

## 27. Open Decisions (Parked)

| Decision | Status | Notes |
|----------|--------|-------|
| Scripting language | Parked | Command system designed to be script-consumable. Pick language later. |
| Git integration depth | Parked | Start with file system. Git features (blame, diff, branch) are future work. |
| AI Assistant integration | Parked | Panel registered as a view type. LLM provider choice deferred. |
| IndexedDB VFS | Parked | Interface defined. Implementation deferred to web deployment phase. |
| Undo/redo architecture | Hooks in place | Commands return `UndoAction` from v1. CodeMirror handles editor undo. Global undo stack is a future feature that consumes the existing return values. |
| Collaborative editing | Not planned | Single-user for v1. |
| Export (CSV, SVG, PDF) | Parked | Each view can implement an export command. No framework-level export system yet. |
| Dirty buffer + external edit conflict | Parked | When an external edit arrives for a dirty buffer, current behaviour silently ignores it. Future: raise a non-modal status bar banner offering reload/keep/diff. Log to a debug channel. |
| Query facade over projections | Phase 4 | Boundary rules defined in §14.3. Implementation deferred until Phase 4 reveals actual query patterns. Add `QueryService` with indexed maps when the first view duplication appears. |

---

## 28. Implementation Sequence

### Phase 1: Framework Shell
1. Tauri project scaffolding + Svelte 5 + Vite
2. Native app hardening (user-select suppression, context menu suppression, browser shortcut interception, overscroll prevention)
3. Splash screen (Tauri splashscreen plugin, close-on-ready handshake)
4. App settings service (load/save to OS config dir, corruption-safe, defaults on failure)
5. BSP layout engine (ZoneTree, ZoneRenderer, SplitContainer, Divider, DividerContextMenu, MergeOverlay)
6. Zone shell (ZoneShell, ZoneHeader with type selector dropdown, ZoneLoadingState, ZoneErrorBoundary)
7. Custom context menu system (ContextMenu, ContextMenuProvider, framework-level menus for zones/dividers)
8. View registry (register, list, lazy load, capabilities, state versioning, requiresProject flag)
9. Global menu bar (GlobalMenuBar, MenuRegistry, contribution-based population)
10. Workspace manager (tabs, serialize/deserialize, corruption-safe load)
11. Theme engine (Gloaming + Parchment tokens, semantic typography/spacing, focus ring tokens)
12. Global status bar
13. Welcome screen (shown when no project open — Open Project, New Project, Recent Projects list)
14. Placeholder views: PlaceholderInfo, PlaceholderColour, PlaceholderBusMonitor, PlaceholderCommandLog
15. Fullscreen toggle (`F11`)
16. **Milestone — acceptance test (all must pass):**
    - Launch → splash screen appears, then Welcome screen. No white flash.
    - Welcome screen shows Open Project button and Recent Projects (empty list on first run)
    - Open Project → native directory picker → workspace loads with PlaceholderColour zones
    - Text selection: can select text inside PlaceholderInfo values, cannot drag-select across zone header or toolbar
    - Right-click zone viewport → custom context menu (not browser menu)
    - Right-click divider → custom divider context menu with Join/Swap options
    - Split horizontal → two distinct colour zones appear
    - Drag divider → smooth resize, PlaceholderInfo updates dimensions in real-time
    - Swap → colours switch sides
    - Join → one zone absorbs the other
    - Change zone type via header dropdown → view switches; change back → state restored
    - Switch workspace tabs → layout swaps entirely; switch back → preserved
    - Toggle theme → all chrome (menu bar, tabs, status bar, zone headers, placeholders) re-themes. Theme persists in settings across restart.
    - F11 → fullscreen; F11 again → restore
    - Ctrl+P → does NOT open browser print dialog (intercepted by framework)
    - Close project → returns to Welcome screen. Reopen → workspace layout restored.
    - File > Quit works; Edit > Command Palette opens (even if palette is stub)

### Phase 2: Core Infrastructure
17. Command registry + keybinding manager (with undo hooks)
18. Command palette
19. Message bus (with channel registry, useBusChannel adapter, dev-mode payload assertion)
20. Selection context
21. Filter context
22. Focus service (with modality and focus ring rendering)
23. Navigation broker
24. Project manager (open/close/recent, .forge/ directory, window title, bus signals)
25. File system abstraction + Tauri implementation + file watcher
26. Buffer map
27. Compiler service + Tauri IPC bridge + chunked output cache
28. Projection registry + initial projections (entityTable, locationGraph, diagnosticsByFile)
29. Recompile-on-change pipeline (buffer-driven)
30. Settings view (zone type — Blender-style, with keybinding editor)
31. **Milestone**: Open a project → compiler runs → output flows through chunks to bus → PlaceholderBusMonitor shows compiler events → projections recompute on change. Settings view opens with `Ctrl+,` and changes apply immediately. Close project → Welcome screen.

### Phase 3: The Editor
32. Code Editor zone (singleton, tab bar, CodeMirror 6)
33. Urd language mode (syntax highlighting)
34. Inline diagnostics
35. Go-to-definition
36. Hover tooltips
37. Autocomplete
38. Breadcrumb bar
39. Source Viewer zone
40. **Milestone**: Full editing experience with compiler feedback. Editor loads instantly on startup; data views fill in after compilation.

### Phase 4: Tier 1 Views (Data Already Exists)
41. File Browser zone (project tree, lazy expand, dirty/error indicators, right-click operations, keyboard nav)
42. VirtualTable shared component
43. Entity Spreadsheet (consuming urd.projection.entityTable)
44. Type Spreadsheet
45. Property Spreadsheet (the power view)
46. Location Spreadsheet
47. Section Spreadsheet
48. Diagnostic Spreadsheet
49. Property Inspector
50. World Statistics Dashboard
51. Dead Code Panel
52. Outline Panel
53. Code Lens
54. Gutter Annotations
55. Global Symbol Search
56. **Milestone**: The "Writer" and "Engineer" workspaces are fully functional. File browser shows project tree with diagnostic indicators.

### Phase 5: Graph Views
57. GraphCanvas shared component (pan, zoom, minimap, spatial culling)
58. Location Network Graph
59. Dialogue Flow Graph
60. Type Hierarchy Graph
61. Property Data Flow Graph
62. Containment Tree
63. Cross-Reference Graph
64. **Milestone**: The "World Builder" workspace is fully functional.

### Phase 6: Runtime Integration
65. Play Panel (Wyrd integration)
66. State Inspector (runtime)
67. Event Log
68. Breadcrumb Trail
69. Coverage Overlay
70. **Milestone**: The "Debug" workspace is fully functional.

### Phase 7: Advanced Analysis + Remaining Views
71. Reachability Matrix
72. Condition/Effect Matrix
73. Monte Carlo Dashboard
74. Remaining spreadsheets (Choice, Rule, Exit, Jump, Read, Write, Sequence, File)
75. Remaining inspectors
76. Remaining analysis views
77. Remaining search tools
78. Remaining specialised views
79. **Milestone**: All 85+ views implemented.

### Phase 8: Polish
80. Export commands (CSV, SVG, PDF)
81. Annotation layer
82. Performance profiling + optimisation pass (against stress fixtures)
83. Corner-drag-to-split gesture (Blender-style zone corner drag)
84. Drag-to-merge gesture (deferred from Phase 1)
85. Per-project settings overrides (.forge/project.json → settings merge)
86. **Milestone**: Production-ready v1.

---

## 29. AI Implementation Guide (Claude Code)

This project is built primarily with Claude Code. This section contains conventions, strategies, and artifacts that make AI-assisted implementation reliable and consistent across sessions.

### 29.1 CLAUDE.md

A `CLAUDE.md` file lives at the repo root. Claude Code reads it automatically at the start of every session. It contains:

```markdown
# Urd Forge

Tauri 2 + Svelte 5 + Rust IDE for Urd interactive fiction authoring.

## Architecture

Read ARCHITECTURE.md before making structural changes. It is the source of truth.

## Commands

```
pnpm dev          # launch dev mode (Tauri + Vite)
pnpm test         # run Vitest (frontend unit tests)
pnpm test:watch   # Vitest in watch mode
pnpm check        # svelte-check + tsc --noEmit
pnpm lint         # eslint
cargo test        # Rust tests (run from src-tauri/)
```

## Conventions

- Framework code: `src/lib/framework/` — NEVER imports from `src/lib/app/`
- Application code: `src/lib/app/` — imports from framework freely
- All state mutations through reducers or commands, never direct assignment
- All bus channels registered in manifests, never ad-hoc
- All view types registered in bootstrap.ts
- Every command that mutates state returns an UndoAction
- CSS: use `--forge-*` tokens, never raw colours or pixel values
- Tests: colocate as `*.test.ts` next to the source file

## File Naming

- Components: `PascalCase.svelte`
- Stores/services: `camelCase.ts`
- Types: `types.ts` within each module directory
- Constants: `constants.ts`

## Import Style

- Use `$lib/framework/...` and `$lib/app/...` path aliases
- Barrel exports (`index.ts`) at each module boundary
- No circular imports — if TypeScript complains, the dependency direction is wrong

## Testing

- `pnpm test` must pass before any PR or phase milestone
- Use `vitest` for all frontend tests
- Mock the compiler with `MockCompilerService` from `src/lib/app/compiler/mock.ts`
- Test reducers with direct assertions, not snapshots
- Test bus channels with the `TestBus` helper from `src/lib/framework/bus/test-helpers.ts`

## When stuck

- Re-read the relevant section of ARCHITECTURE.md
- Check the types in `src/lib/framework/types.ts` and `src/lib/app/compiler/types.ts`
- Run `pnpm check` — type errors usually reveal the problem
```

### 29.2 Types-First Implementation Strategy

For each phase step, Claude Code should:

1. **Create the type file first.** Copy the TypeScript interface from ARCHITECTURE.md into a real `.ts` file. This gives immediate type-checking for everything that follows.
2. **Create the test file second.** Write the acceptance test based on the milestone criteria *before* writing the implementation. Tests initially fail.
3. **Implement until tests pass.** The implementation is done when `pnpm test` and `pnpm check` both pass.
4. **Do not move to the next step** until the current step's tests pass and there are zero type errors.

This prevents the most common AI failure mode: writing code that looks right but doesn't actually compile or pass assertions.

### 29.3 Mock Compiler

All frontend development from Phase 1 onward uses a mock compiler. The real Rust compiler is only needed when testing IPC in Phase 2.

```typescript
// APPLICATION — src/lib/app/compiler/mock.ts

import type { CompilerOutput, ChunkName } from './types';

/**
 * Returns a CompilerOutput from a JSON fixture file.
 * Simulates chunk-level changes by accepting an optional set of
 * "dirty" chunk names — those get new hashes, others keep stable hashes.
 *
 * Usage in tests:
 *   const output1 = mockCompile('fixtures/basic-world.json');
 *   const output2 = mockCompile('fixtures/basic-world.json', ['diagnostics', 'ast']);
 *   // output2 has new hashes for diagnostics and ast, same hashes for everything else
 */
export function mockCompile(
  fixturePath: string,
  dirtyChunks?: ChunkName[]
): CompilerOutput;

/**
 * A CompilerService implementation that uses mockCompile internally.
 * Drop-in replacement for TauriCompiler — same interface, no IPC.
 */
export class MockCompilerService implements CompilerService {
  constructor(fixturePath: string);
  compile(buffers: Map<string, string>): Promise<CompilerOutput>;

  // Test helper: simulate a recompile where only specific chunks change.
  simulateChange(dirtyChunks: ChunkName[]): void;
}
```

**Fixture files** live in `fixtures/` and are checked into the repo:

| Fixture | Purpose | Scale |
|---------|---------|-------|
| `basic-world.json` | Smoke test — a handful of entities, locations, sections | ~20 entities |
| `medium-world.json` | Realistic authoring session | ~500 entities, 50 locations |
| `stress-50k-entities.json` | Performance testing | 50k entities, 500 types, 200 locations |
| `stress-huge-diagnostics.json` | Diagnostics rendering | 10k diagnostics, 5k cross-refs |
| `empty-world.json` | Edge case — fresh project, no content | 0 entities |
| `error-world.json` | Edge case — all chunks have errors, diagnostics-heavy | Varies |

Each fixture is a valid `CompilerOutput` JSON blob with stable hashes. Claude Code can generate them by writing a fixture-builder script.

**Golden fixture cross-validation.** The `basic-world.json` fixture serves as a schema contract between Rust and TypeScript. Both sides validate it at build time:
- `cargo test` deserializes `basic-world.json` into the Rust `CompilerOutput` struct and asserts all fields parse correctly.
- `pnpm test` deserializes the same file into the TypeScript `CompilerOutput` type and asserts all chunks are present with valid hashes.

If either side fails, the schema has drifted. This catches Rust/TS type mismatches before they reach IPC at runtime. It's cheaper than codegen and sufficient for a single-team project.

### 29.4 Test Helpers

Framework-level test utilities that every test file can import:

```typescript
// FRAMEWORK — src/lib/framework/bus/test-helpers.ts
export class TestBus implements MessageBus {
  // All published events are recorded and inspectable.
  published: Array<{ channel: string; payload: unknown }>;
  // Assert that a specific channel received exactly N publishes.
  expectPublishCount(channel: string, count: number): void;
  // Assert payload size is under the dev-mode 4KB limit.
  assertPayloadSize(channel: string): void;
}

// FRAMEWORK — src/lib/framework/layout/test-helpers.ts
export function createTestTree(leafCount: number): ZoneTree;
export function assertTreeInvariants(tree: ZoneTree): void;
  // Checks: unique IDs, no orphans, all leaves have zoneTypeId, singleton constraints.

// FRAMEWORK — src/lib/framework/commands/test-helpers.ts
export class TestCommandRegistry implements CommandRegistry {
  // Records all executed commands with args and undo actions.
  executed: Array<{ id: string; args: unknown; undoAction?: UndoAction }>;
}
```

### 29.5 Per-Step Verification Commands

Each implementation step should end with Claude Code running:

```bash
# 1. Type check — catches import direction violations and interface mismatches
pnpm check

# 2. Tests — catches logic errors
pnpm test

# 3. Lint — catches style drift
pnpm lint

# 4. Dev launch — catches runtime errors not caught by types/tests
pnpm dev
# (verify visually via screenshot if running in a headed environment)
```

If any of these fail, fix before moving on. Do not accumulate tech debt across steps.

### 29.6 Import Direction Enforcement

The framework-never-imports-application rule is critical and easy to violate accidentally. Enforce it with a lint rule:

```typescript
// eslint.config.js — import boundary rule
{
  rules: {
    'no-restricted-imports': ['error', {
      patterns: [{
        group: ['$lib/app/*'],
        message: 'Framework code must not import from application code.',
      }],
    }],
  },
  // Apply only to framework files
  files: ['src/lib/framework/**/*.ts', 'src/lib/framework/**/*.svelte'],
}
```

This turns an architectural rule into a CI-enforceable error. Claude Code will see the lint failure immediately if it violates the boundary.

### 29.7 Session Handoff Pattern

When a Claude Code session ends mid-phase, it should leave a `STATUS.md` at the repo root:

```markdown
# Current Status

## Phase: 2 — Core Infrastructure
## Step: 7 — Buffer map
## State: In progress

### Completed this session
- Command registry with undo hooks (step 1) ✅
- Command palette (step 2) ✅
- Message bus with channel registry (step 3) ✅
- Selection context (step 4) ✅
- Filter context (step 5) ✅
- Focus service with modality (step 6) ✅

### Current step notes
- BufferMap interface created in src/lib/app/compiler/buffer-map.ts
- Tests written but onAnyChange debounce not yet implemented
- Blocked on: nothing, just ran out of context

### Next step
- Finish BufferMap.onAnyChange with debounce
- Then: Compiler service (step 8)

### Test status
- `pnpm check`: PASS
- `pnpm test`: 2 failing (buffer-map.test.ts — expected, not implemented yet)
- `pnpm lint`: PASS
```

The next session reads `STATUS.md` and `CLAUDE.md`, knows exactly where to resume.

### 29.8 Fixture Generation Script

Rather than hand-authoring fixture JSON, include a script that generates valid `CompilerOutput` fixtures from parameters:

```typescript
// scripts/generate-fixture.ts
// Usage: npx tsx scripts/generate-fixture.ts --entities 500 --locations 50 --diagnostics 100 --output fixtures/medium-world.json

// Generates a structurally valid CompilerOutput with:
// - Stable, deterministic hashes (based on content, not random)
// - Realistic entity names, type hierarchies, location graphs
// - Cross-references between chunks (entities reference types, locations have exits)
// - Configurable error/warning ratio in diagnostics
```

This lets Claude Code create new test scenarios without understanding the full Urd schema.

---

*This is a living document. Update it as architectural decisions are made or revised.*