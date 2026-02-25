# URD — Playground IDE Features

*Turn the playground's WASM compiler into a language server with inline diagnostics, autocomplete, hover tooltips, and go-to-definition — entirely client-side.*

February 2026 | Done

> **Document status: BRIEF** — Four self-contained CodeMirror extensions that wire the existing WASM compiler output (diagnostics, DefinitionIndex, FactSet, PropertyDependencyIndex) into the playground editor. No new Rust/WASM work. No Lezer grammar. No web worker. Each phase is independently shippable.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-25
**Status:** Done

### What was done

All four phases implemented and shipped in commit `0afda39`:

1. **Phase 1 — Inline diagnostics:** `lint-source.ts` wraps the `@codemirror/lint` `linter()` extension with 300ms debounce, mapping compiler diagnostics to squiggly underlines via `byteColToCharCol`. Replaces the manual dual-debounce logic (parseTimer/compileTimer/compileSeq) in UrdPlayground.svelte. `playground-state.ts` provides shared compile state with stale retention.

2. **Phase 2 — Autocomplete:** `completion-source.ts` implements three trigger contexts (`@` entities, `@entity.` properties, `->` sections). Reads from shared state — no compilation on each keystroke. Added `@codemirror/autocomplete` dependency.

3. **Phase 3 — Hover tooltips:** `cursor-resolver.ts` ports `cursor.rs` (6 reference types, ~160 lines). `hover-tooltip.ts` renders rich HTML tooltips for entities (type, container, properties), properties (type, default, read/write counts, orphan status), sections (compiled ID, incoming/outgoing jumps, choices), and locations (slug, exits, entities).

4. **Phase 4 — Go-to-definition:** `goto-definition.ts` implements Ctrl/Cmd+click navigation via a `ViewPlugin` click handler and a visual affordance (underline + pointer cursor) via `StateField`/`Decoration.mark` when Ctrl/Cmd is held.

All extensions wired into UrdPlayground.svelte's CodeMirror `EditorState.create()`. Svelte reactivity bridged via `subscribe()` on the shared state store.

### What changed from the brief

1. **`compiler-bridge.ts` was modified** — the brief stated it was unchanged, but `DefinitionEntry`, `DefinitionIndex` types and the `definition_index` field on `CompileResult` were missing from the TypeScript types. The WASM boundary already returned the field — only the TS declarations were absent.

2. **`codemirror-urd.ts` received additional styles** — the brief specified only `.cm-definition-link`. Autocomplete popup styles (`.cm-tooltip-autocomplete`, `.cm-completionLabel`, `.cm-completionDetail`) and hover tooltip styles (`.urd-hover-tooltip`, `.urd-tt-dim`, `.urd-tt-warn`) were added to both Gloaming and shared/Parchment themes.

3. **`PlaygroundState` gained `compileTimeMs`** — needed to bridge compile timing from the linter (which now owns compilation) back to the Svelte UI for the OutputPane's compile time display.

4. **50ms `parseOnly()` fast-feedback loop dropped** — the brief left this as an implementer's choice. The linter's 300ms cycle provides sufficient error feedback.

5. **Entity completion label uses `id` instead of `@id`** — functionally identical since the trigger is after `@` and `from` is set accordingly.

---

## Context

The playground already runs the full Urd compiler as WASM in the browser. It already returns diagnostics, a FactSet, a PropertyDependencyIndex, and a DefinitionIndex from every compilation. The CodeMirror editor already has `@codemirror/lint` installed, a lint gutter registered, custom squiggly SVG styling in both themes, and a diagnostic click-to-scroll handler with byte-to-char offset conversion.

The LSP server (`packages/lsp/`) already implements the same four capabilities against the same data structures — diagnostics, go-to-definition, hover, and autocomplete — using `cursor.rs` for reference resolution and `hover.rs` for tooltip content generation. That logic is a reference implementation for this brief.

### Current state

- `compiler-bridge.ts` exposes `compileSource()` returning `CompileResult { diagnostics, facts, property_index, definition_index, world }`
- The `definition_index` field crosses the WASM boundary (serialised in `wasm.rs`) but is **not consumed** in the playground — it's returned and ignored
- `UrdPlayground.svelte` runs dual debounce: 50ms `parseOnly()` for syntax validity, 300ms `compileSource()` for full output
- Diagnostics are rendered in the OutputPane as a clickable list — but not as inline editor decorations
- `@codemirror/lint` is installed and `lintGutter()` is registered — but no `linter()` or `lintSource` is wired up
- `@codemirror/autocomplete` is **not installed**
- The Analysis panel (FactSetView, PropertyDependencyView, LocationGraph, DialogueGraph) already consumes facts and property_index — proving the data is available and correct
- `codemirror-urd.ts` provides StreamLanguage-based highlighting — sufficient for syntax, not needed for semantic features
- The LSP's `cursor.rs` (80 lines of reference-under-cursor resolution) and `completion.rs` (context-aware entity/property/section completion) are portable logic that can be transliterated to TypeScript

### What this brief delivers

Four CodeMirror extensions, each independently shippable:

1. **Inline diagnostics.** Squiggly underlines on error/warning/info spans in the editor, with hover messages.
2. **Autocomplete.** Popup completions for `@` (entities), `@entity.` (properties), and `->` (sections).
3. **Hover tooltips.** Rich entity cards, property metadata with read/write counts, section jump statistics on mouseover.
4. **Go-to-definition.** Ctrl/Cmd+click to jump from a reference to its declaration.


## Dependencies

- **SF-5 (LSP Foundation) passed.** DefinitionIndex exists. `cursor.rs` and `hover.rs` provide reference implementations.
- **SF-2 (PropertyDependencyIndex) passed.** Read/write counts available for hover tooltips.
- **SF-1A (Novel FactSet Diagnostics) passed.** All diagnostic codes (URD100–699) available for inline display.
- **Playground Component passed.** WASM bridge, CodeMirror setup, OutputPane, and Analysis panel all working.


## Architecture

### Shared Compile State

Currently `UrdPlayground.svelte` holds `compileResult` as Svelte 5 `$state` and threads it to child components as props. All four extensions need to read it from within CodeMirror extension functions, which exist outside the Svelte reactivity system.

Introduce `playground-state.ts` — a minimal reactive store that bridges Svelte state and CodeMirror extensions:

```typescript
// playground-state.ts

import type { CompileResult, Diagnostic } from './compiler-bridge';

interface PlaygroundState {
  result: CompileResult | null;
  parsedWorld: Record<string, any> | null;       // JSON.parse(result.world), cached
  definitionIndex: DefinitionEntry[] | null;      // parsed from result.definition_index
}

let current: PlaygroundState = {
  result: null,
  parsedWorld: null,
  definitionIndex: null,
};

type Listener = () => void;
const listeners: Set<Listener> = new Set();

export function getState(): PlaygroundState { return current; }

export function updateState(result: CompileResult): void {
  const parsedWorld = result.world ? JSON.parse(result.world) : current.parsedWorld;
  const definitionIndex = result.definition_index?.definitions ?? current.definitionIndex;
  current = { result, parsedWorld, definitionIndex };
  listeners.forEach(fn => fn());
}

export function subscribe(fn: Listener): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}
```

**Stale state retention:** When a compilation fails (PARSE/IMPORT error → no `world`, no `definition_index`), the store retains the previous `parsedWorld` and `definitionIndex`. Autocomplete, hover, and go-to-definition continue working with stale-but-useful data while the user fixes their syntax error. Only `result` (and therefore diagnostics) is always replaced. This matches the LSP's `WorldState` behaviour.

### Extension wiring in UrdPlayground.svelte

Each phase adds one extension to the CodeMirror `EditorState.create()` extensions array. The existing `EditorView.updateListener` and manual debounce timers are replaced by the `linter()` extension (Phase 1), which handles debouncing and compilation internally.

### File structure

```
sites/urd.dev/src/components/playground/
  playground-state.ts      ← NEW: shared compile state store
  lint-source.ts           ← NEW: Phase 1 — inline diagnostics
  completion-source.ts     ← NEW: Phase 2 — autocomplete
  cursor-resolver.ts       ← NEW: Phase 3 — port of cursor.rs (shared by hover + goto)
  hover-tooltip.ts         ← NEW: Phase 3 — hover tooltips
  goto-definition.ts       ← NEW: Phase 4 — Ctrl/Cmd+click navigation
  compiler-bridge.ts       ← EXISTING: unchanged
  codemirror-urd.ts        ← EXISTING: unchanged
  UrdPlayground.svelte     ← MODIFIED: add extensions, replace manual debounce
  OutputPane.svelte         ← EXISTING: unchanged (diagnostics list remains)
```


## Phase 1: Inline Diagnostics

### What it does

Red, amber, and blue squiggly underlines appear on the exact character spans where the compiler reports errors, warnings, and info diagnostics. Hovering a squiggly shows the diagnostic message and code. The lint gutter shows severity markers per line.

### Why first

Highest value, lowest effort. The lint gutter is registered, both themes have custom squiggly SVGs, the byte-to-char conversion exists, and `@codemirror/lint` is installed. This is wiring.

### Implementation

Create `playground/lint-source.ts`:

```typescript
import { linter, type Diagnostic as CMLintDiag } from '@codemirror/lint';
import { type Text } from '@codemirror/state';
import { compileSource, byteColToCharCol, type CompileResult } from './compiler-bridge';
import { updateState } from './playground-state';

export function urdLinter() {
  return linter((view) => {
    const source = view.state.doc.toString();
    const result = compileSource(source);
    updateState(result);
    return mapDiagnostics(result, view.state.doc);
  }, { delay: 300 });
}

function mapDiagnostics(result: CompileResult, doc: Text): CMLintDiag[] {
  return result.diagnostics.map(d => {
    const startLine = doc.line(Math.min(d.span.start_line, doc.lines));
    const endLine = doc.line(Math.min(d.span.end_line, doc.lines));
    const startCharCol = byteColToCharCol(startLine.text, d.span.start_col);
    const endCharCol = byteColToCharCol(endLine.text, d.span.end_col);
    const from = startLine.from + Math.max(0, startCharCol - 1);
    const to = endLine.from + Math.max(0, endCharCol - 1);
    return {
      from: Math.max(0, from),
      to: Math.max(from + 1, to),
      severity: d.severity === 'info' ? 'info' : d.severity,
      message: `[${d.code}] ${d.message}`,
    };
  });
}
```

### Integration changes to UrdPlayground.svelte

1. Add `urdLinter()` to the extensions array (after `lintGutter()`).
2. Remove the `EditorView.updateListener` that calls `onSourceChange()`.
3. Remove the manual `parseTimer` and `compileTimer` debounce logic.
4. The `urdLinter()` extension handles debouncing (300ms `delay` parameter) and calls `updateState()` after every compile, which the OutputPane can subscribe to via the shared state.
5. The `parseOnly()` fast-feedback loop (50ms) can remain as a separate `EditorView.updateListener` if the syntax validity indicator is valued. Alternatively, it can be dropped since the linter provides error feedback at 300ms — close enough. Design decision for the implementer.

### What the user sees

- Squiggly underlines appear under error spans as they type (after 300ms pause)
- Hovering a squiggly shows `[URD430] Property 'trust' is not defined on type 'Guard'`
- The lint gutter shows `◆` for errors, `▸` for warnings (already styled in both themes)
- The OutputPane diagnostic list continues to work (from shared state)

### New dependency

None. `@codemirror/lint` is already installed.


## Phase 2: Autocomplete

### What it does

Typing `@` opens a popup listing all entities with their types. Typing `@elder_maren.` shows that entity's properties with their types. Typing `->` shows a list of section names. CodeMirror handles the popup UI, filtering, keyboard navigation, and insertion.

### Why second

This is the feature that makes the playground feel like a real authoring tool. The moment a visitor sees entities are first-class constructs, they expect `@` to offer completions.

### Implementation

Create `playground/completion-source.ts`:

The logic is a direct port of `packages/lsp/src/completion.rs`. The three contexts are:

**1. Entity completion** — triggered after `@`:

```typescript
function completeEntities(state: PlaygroundState): Completion[] {
  const entities = state.parsedWorld?.entities;
  if (!entities) return [];
  return Object.entries(entities).map(([id, entity]: [string, any]) => ({
    label: `@${id}`,
    type: 'variable',
    detail: entity.type ?? '',
    apply: id,  // insert just the id, not the @
  }));
}
```

**2. Property completion** — triggered after `@entity.`:

```typescript
function completeEntityProperties(state: PlaygroundState, entityId: string): Completion[] {
  const entity = state.parsedWorld?.entities?.[entityId];
  if (!entity) return [];
  const typeName = entity.type;
  const properties = state.parsedWorld?.types?.[typeName]?.properties;
  if (!properties) return [];
  return Object.entries(properties).map(([name, prop]: [string, any]) => ({
    label: name,
    type: 'property',
    detail: prop.type ?? '',
  }));
}
```

**3. Section completion** — triggered after `->`:

```typescript
function completeSections(state: PlaygroundState): Completion[] {
  if (!state.definitionIndex) return [];
  const seen = new Set<string>();
  return state.definitionIndex
    .filter(d => d.definition.kind === 'section')
    .filter(d => {
      const name = d.definition.local_name;
      if (seen.has(name)) return false;
      seen.add(name);
      return true;
    })
    .map(d => ({
      label: d.definition.local_name,
      type: 'text',
      detail: `in ${d.definition.file_stem}`,
    }));
}
```

**Trigger detection:** The completion source function receives a `CompletionContext` with the current cursor position. Extract the text before the cursor and apply the same heuristics as `completion.rs`:

- Text ends with `@` or contains `@` without `.` → entity completion
- Text matches `@entity_id.` pattern → extract entity ID → property completion
- Text ends with `-> ` or `->` → section completion
- Otherwise → no completions (return `null`)

### Integration

1. `pnpm add @codemirror/autocomplete` in `sites/urd.dev/`
2. Add `autocompletion({ override: [urdCompletionSource] })` to the extensions array
3. The completion source reads from `getState()` — no compilation needed, it uses the latest cached result

### New dependency

`@codemirror/autocomplete` — **not currently installed.** This is the one new npm dependency in this brief.


## Phase 3: Hover Tooltips

### What it does

Hovering over `@elder_maren` shows an entity card: type, container location, properties with current defaults. Hovering over `.trust` shows: type, default value, read/write site count, orphan status. Hovering over `== greet` or `-> greet` shows: section compiled ID, incoming/outgoing jump counts, choice count. Hovering over `# The Sunken Vault` shows: location slug, exit count, contained entities.

### Why third

This is where the FactSet and PropertyDependencyIndex deliver value directly in the editor. The Analysis panel already shows this data in aggregate — hover shows it *in context* at the point of authorship.

### Implementation — Cursor Resolver (shared module)

Create `playground/cursor-resolver.ts` — a direct port of `packages/lsp/src/cursor.rs`:

```typescript
type Reference =
  | { kind: 'entity'; id: string }
  | { kind: 'entity-property'; entityId: string; property: string }
  | { kind: 'type-property'; typeName: string; property: string }
  | { kind: 'section-jump'; name: string }
  | { kind: 'section-label'; name: string }
  | { kind: 'location-heading'; name: string };

function identifyReference(line: string, col: number): Reference | null {
  // Port of cursor.rs — ~80 lines of string scanning
  // Precedence: entity.property > entity > section jump > section label > location heading > type.property
}
```

The `cursor.rs` test cases (8 tests) serve as the specification for this port. The TypeScript implementation should pass the same cases.

### Implementation — Hover Tooltip

Create `playground/hover-tooltip.ts`:

```typescript
import { hoverTooltip, type Tooltip } from '@codemirror/view';
import { identifyReference } from './cursor-resolver';
import { getState } from './playground-state';

export function urdHoverTooltip() {
  return hoverTooltip((view, pos) => {
    const line = view.state.doc.lineAt(pos);
    const col = pos - line.from;
    const ref = identifyReference(line.text, col);
    if (!ref) return null;

    const content = buildTooltipContent(ref);
    if (!content) return null;

    return {
      pos,
      above: true,
      create: () => {
        const dom = document.createElement('div');
        dom.className = 'urd-hover-tooltip';
        dom.innerHTML = content;
        return { dom };
      },
    };
  });
}
```

**Tooltip content by reference type** — port of `packages/lsp/src/hover.rs`:

| Reference | Content | Data source |
|-----------|---------|-------------|
| Entity (`@warden`) | Type, container location, property snapshot | `parsedWorld.entities`, `parsedWorld.locations` |
| Property (`@warden.trust`) | Type, default, read/write counts, orphan status | `definitionIndex` (type/default), `result.property_index` (counts) |
| Section (`== greet`, `-> greet`) | Compiled ID, incoming/outgoing jumps, choice count | `definitionIndex` (compiled_id), `result.facts` (jumps, choices scan) |
| Location (`# The Sunken Vault`) | Slug, exit count, contained entities | `definitionIndex` (slug), `parsedWorld.locations` |

### Tooltip styling

The tooltip DOM element uses existing CSS variables from the Gloaming/Parchment themes. Styled inline or via a class in `codemirror-urd.ts` (which already defines `.cm-tooltip` styling). Content is plain HTML, not Markdown — this is a browser DOM tooltip, not an LSP Markdown response.

### Entity-to-type resolution

When the cursor is on `@warden.trust`, the hover needs to resolve `@warden` → `Guard` → look up `prop:Guard.trust` in the definition index. Resolution path: `parsedWorld.entities[entityId].type` → type name. Same pattern as in `hover.rs`:

```typescript
function resolveEntityType(entityId: string): string | null {
  return getState().parsedWorld?.entities?.[entityId]?.type ?? null;
}
```

### FactSet scans for section hover

Section hover shows incoming/outgoing jump counts and choice counts. These are O(n) scans over `result.facts.jumps` and `result.facts.choices`:

- Incoming jumps: `facts.jumps.filter(j => j.target.kind === 'section' && j.target.id === compiledId).length`
- Outgoing jumps: `facts.jumps.filter(j => j.from_section === compiledId).length`
- Choices: `facts.choices.filter(c => c.section === compiledId).length`

With typical world sizes (~50 jumps, ~30 choices), these scans are sub-millisecond.

### New dependency

None. `hoverTooltip` is in `@codemirror/view`, already installed.


## Phase 4: Go-to-Definition

### What it does

Ctrl+click (Cmd+click on macOS) on `@elder_maren` jumps to the `entity:` declaration line in the frontmatter. On `-> greet` jumps to the `== greet` section label. On `Guard.trust` jumps to the property definition within the type block. A visual affordance (underline) appears when Ctrl/Cmd is held and the cursor hovers over an identifier.

### Why last

Least urgent in a single-file playground — the file is short enough to scroll manually. Becomes critical if multi-file tab support is added later. But it's cheap to implement given the cursor resolver from Phase 3 and the DefinitionIndex that's already available.

### Implementation

Create `playground/goto-definition.ts`:

1. **Reuse `identifyReference()`** from `cursor-resolver.ts`.
2. **Map reference to DefinitionIndex key:**
   - `Entity("warden")` → find entry where key === `"entity:@warden"`
   - `EntityProperty("warden", "trust")` → resolve entity type → find `"prop:Guard.trust"`
   - `SectionJump("greet")` → find entries where `definition.kind === "section"` and `definition.local_name === "greet"`
   - `SectionLabel("greet")` → same as SectionJump
   - `LocationHeading("The Sunken Vault")` → find entry where `definition.kind === "location"` and `definition.display_name === "The Sunken Vault"`
   - `TypeProperty("Guard", "trust")` → find `"prop:Guard.trust"`
3. **Look up span** from the matching DefinitionIndex entry.
4. **Navigate:** Convert span (1-indexed line/col) to CodeMirror position, dispatch `{ selection: { anchor: pos }, scrollIntoView: true }`, call `view.focus()`.

### Visual affordance

Add a `ViewPlugin` that:
- Listens for Ctrl/Cmd keydown/keyup
- On Ctrl/Cmd + mousemove, identifies the reference under the cursor
- If a reference resolves to a definition, adds a `Decoration.mark` with `class: "cm-definition-link"` (underline + pointer cursor)
- Removes the decoration on Ctrl/Cmd keyup or mouse leave

CSS (added to `codemirror-urd.ts` theme):
```css
.cm-definition-link {
  text-decoration: underline;
  text-decoration-color: var(--blue-light);
  cursor: pointer;
}
```

### Integration

Add to the extensions array:
- A `keymap` entry for Ctrl/Cmd+click that calls the go-to-definition handler
- The `ViewPlugin` for the visual affordance

### Section name ambiguity

If multiple sections share the same `local_name` (e.g., in a multi-file world with imports), multiple definitions may match. In the single-file playground this won't occur. If it does in a future multi-file mode, jump to the first match — disambiguation UI can be added later.

### New dependency

None. Keymap and ViewPlugin are in `@codemirror/view`, already installed.


## Performance Budget

| Feature | Target | Expected |
|---------|--------|----------|
| Diagnostics (full compile + mapping) | < 300ms | ~40ms (34ms compile + ~5ms mapping) |
| Completion popup | < 100ms | < 5ms (JSON property lookup, no compile) |
| Hover tooltip | < 100ms | < 10ms (DefinitionIndex lookup + FactSet scan) |
| Go-to-definition | < 50ms | < 5ms (single map lookup + cursor dispatch) |

The compiler's 34ms full-pipeline time is the bottleneck and it's well under every target. All other operations are map lookups and DOM construction. No web worker is needed.

**Web worker threshold:** Same as the playground brief — move WASM calls to a worker if `compile_source()` consistently exceeds 16ms (one frame budget at 60fps), or if Lighthouse flags main-thread blocking. The `playground-state.ts` subscription pattern is worker-compatible without API changes.


## New Dependency

| Package | Version | Purpose |
|---------|---------|---------|
| `@codemirror/autocomplete` | ^6.x | Completion popup UI, filtering, keyboard navigation, insertion |

All other CodeMirror packages used by this brief are already installed: `@codemirror/lint`, `@codemirror/view`, `@codemirror/state`, `@codemirror/language`.


## Files Changed

### New files

| File | Phase | Purpose |
|------|-------|---------|
| `playground/playground-state.ts` | 1 | Shared compile state store with stale retention |
| `playground/lint-source.ts` | 1 | `linter()` extension mapping compiler diagnostics to CodeMirror |
| `playground/completion-source.ts` | 2 | Completion source for `@entity`, `@entity.property`, `-> section` |
| `playground/cursor-resolver.ts` | 3 | Port of `cursor.rs` — reference-under-cursor identification |
| `playground/hover-tooltip.ts` | 3 | `hoverTooltip()` extension with entity cards and property metadata |
| `playground/goto-definition.ts` | 4 | Ctrl/Cmd+click handler + visual affordance ViewPlugin |

### Modified files

| File | Phase | Change |
|------|-------|--------|
| `UrdPlayground.svelte` | 1–4 | Add extensions to EditorState, replace manual debounce with linter, subscribe to shared state for OutputPane |
| `codemirror-urd.ts` | 4 | Add `.cm-definition-link` style to Gloaming and Parchment themes |
| `sites/urd.dev/package.json` | 2 | Add `@codemirror/autocomplete` dependency |

### Unchanged files

| File | Note |
|------|------|
| `compiler-bridge.ts` | No changes — CompileResult already includes all needed fields |
| `OutputPane.svelte` | Diagnostic list continues working from shared state |
| `FactSetView.svelte`, `PropertyDependencyView.svelte` | Continue working from props (passed from shared state) |
| `packages/compiler/src/wasm.rs` | Already serialises definition_index — no changes needed |
| `packages/lsp/src/*` | Reference implementation only — not modified |


## Estimated Size

| File | Lines |
|------|-------|
| `playground-state.ts` | ~40 |
| `lint-source.ts` | ~50 |
| `completion-source.ts` | ~120 |
| `cursor-resolver.ts` | ~100 |
| `hover-tooltip.ts` | ~150 |
| `goto-definition.ts` | ~80 |
| `UrdPlayground.svelte` changes | ~-30 (remove debounce), ~+20 (add extensions) |
| `codemirror-urd.ts` changes | ~+10 (definition-link style) |
| **Total** | **~540 new lines** |


## Acceptance Criteria

### Phase 1: Inline Diagnostics

- [ ] **IDE-1.1** — Compiler errors appear as red squiggly underlines on the exact span reported by the diagnostic.
- [ ] **IDE-1.2** — Compiler warnings appear as amber squiggly underlines.
- [ ] **IDE-1.3** — Hovering a squiggly underline shows the diagnostic code and message (e.g., `[URD430] Property 'trust' is not defined on type 'Guard'`).
- [ ] **IDE-1.4** — The lint gutter displays `◆` for error lines and `▸` for warning lines.
- [ ] **IDE-1.5** — Inline diagnostics update within 300ms of the user pausing typing.
- [ ] **IDE-1.6** — The OutputPane diagnostic list continues to function alongside inline diagnostics.
- [ ] **IDE-1.7** — Non-ASCII content (Cyrillic, accented characters) does not cause misaligned squiggly positions.

### Phase 2: Autocomplete

- [ ] **IDE-2.1** — Typing `@` opens a completion popup listing all entities with their type names.
- [ ] **IDE-2.2** — Typing `@elder_maren.` opens a completion popup listing that entity's properties with their types.
- [ ] **IDE-2.3** — Typing `->` opens a completion popup listing all section local names with their file stems.
- [ ] **IDE-2.4** — Completion results are drawn from the latest successful compilation (stale state if current compile has errors).
- [ ] **IDE-2.5** — Completion popup appears within 100ms of the trigger character.

### Phase 3: Hover Tooltips

- [ ] **IDE-3.1** — Hovering `@entity_id` shows: type name, container location, property snapshot.
- [ ] **IDE-3.2** — Hovering a property reference shows: type, default value, read/write site count, orphan status.
- [ ] **IDE-3.3** — Hovering a section label or jump target shows: compiled ID, incoming/outgoing jump counts, choice count.
- [ ] **IDE-3.4** — Hovering a location heading shows: slug, exit count, contained entities.
- [ ] **IDE-3.5** — Tooltip appears within 100ms. Tooltip dismisses on mouse leave.
- [ ] **IDE-3.6** — Tooltip content uses Gloaming/Parchment CSS variables (adapts to theme).

### Phase 4: Go-to-Definition

- [ ] **IDE-4.1** — Ctrl+click (Cmd+click on macOS) on `@entity_id` jumps to the entity declaration in frontmatter.
- [ ] **IDE-4.2** — Ctrl+click on `-> section_name` jumps to the `== section_name` label.
- [ ] **IDE-4.3** — Ctrl+click on a property reference (e.g., `Guard.trust`) jumps to the property definition in the type block.
- [ ] **IDE-4.4** — Holding Ctrl/Cmd while hovering an identifier shows an underline affordance (cursor changes to pointer).
- [ ] **IDE-4.5** — If no definition is found, click is a no-op (no error, no scroll).

### Cross-cutting

- [ ] **IDE-5.1** — All features work in both Gloaming (dark) and Parchment (light) themes.
- [ ] **IDE-5.2** — Stale state retention: autocomplete, hover, and go-to-definition continue to function using previous compilation data while the current source has errors.
- [ ] **IDE-5.3** — No new WASM exports or Rust changes required.
- [ ] **IDE-5.4** — Bundle size increase under 15 KB gzipped (primarily `@codemirror/autocomplete`).


## Design Decisions

### Why not port cursor.rs via WASM?

The cursor resolver is 80 lines of string scanning. Porting to TypeScript is faster than adding a new WASM export, designing a JSON interface, handling the WASM call overhead, and testing the boundary. The logic is simple enough that a direct port is lower-risk than an FFI boundary.

If the cursor resolver grows significantly (e.g., to handle multi-line constructs or semantic disambiguation), moving it to Rust and exposing via WASM would be justified. For the current scope, TypeScript is correct.

### Why shared state instead of re-compiling per extension?

Each extension needs the same `CompileResult`. Without shared state, the linter compiles, the hover handler compiles again, the completion handler compiles again — three redundant 34ms WASM calls. The shared state ensures one compile per edit cycle, with all extensions reading the cached result.

### Why not a Lezer grammar?

A Lezer grammar would give CodeMirror structural parse information (bracket matching, folding, indentation, precise token boundaries). The StreamLanguage mode in `codemirror-urd.ts` handles syntax highlighting. The semantic features in this brief (diagnostics, completion, hover, go-to-def) all use the compiler's output, not the editor's grammar. A Lezer grammar is orthogonal — useful for editor amenities but not required for IDE features. It can be added independently without changing anything in this brief.

### Why replace the manual debounce with the linter extension?

The current `UrdPlayground.svelte` has hand-rolled debounce logic (parse timer, compile timer, sequence counter for stale results). The `@codemirror/lint` `linter()` extension provides built-in debouncing, document-change detection, and result management. Using it eliminates ~30 lines of manual timer management and ensures diagnostics are always in sync with the editor state. The `linter()` extension's `delay` parameter replaces the `COMPILE_DEBOUNCE_MS` constant.

The 50ms `parseOnly()` fast-feedback loop is optional to retain — it provides a faster syntax validity indicator than the full 300ms compile. The implementer can choose to keep it as a separate `EditorView.updateListener` or drop it in favour of the linter's 300ms cycle. Either approach is acceptable.

### Why not a web worker?

At 34ms compile time for the full pipeline, the main thread is not blocked long enough to cause perceptible jank. The 16ms frame budget threshold (from the playground brief) is not exceeded. If compile times grow with larger worlds, the `playground-state.ts` subscription pattern is worker-compatible — the `updateState()` call can be made from a worker's `postMessage` handler without changing any extension code.


## What This Brief Does Not Cover

- **Quick-fix actions on diagnostics.** The lint source can return `actions` (e.g., "Did you mean `@guard`?"). This requires a suggestion engine in the compiler (new WASM export). Future enhancement.
- **Bracket/section folding.** CodeMirror's `foldService` driven by `==` section markers and `---` frontmatter delimiters. Independent of this brief.
- **Multi-file tab support.** Each tab would get its own editor state, sharing a single compile pass. Go-to-definition would switch tabs when the target is in another file. Separate brief.
- **Outline/symbol panel.** The DefinitionIndex provides a flat list of all declarations with spans — rendering an outline is a projection. Separate feature.
- **Semantic tokens.** LSP-driven syntax highlighting (more precise than the StreamLanguage mode). Requires either a Lezer grammar or a new WASM export. Out of scope.
- **Minimap.** Visual overview of the document. Editor amenity, not an IDE feature.


## Relationship to Other Briefs

| Brief | Relationship |
|-------|--------------|
| **SF-5 (LSP Foundation)** | Reference implementation. `cursor.rs`, `hover.rs`, `completion.rs`, and `definition.rs` contain the logic this brief ports to TypeScript. The DefinitionIndex built by SF-5 is the primary data source for Phases 3 and 4. |
| **Playground Component** | Foundation. This brief extends the playground with four CodeMirror extensions. All existing functionality (OutputPane, Analysis panel, theme switching, mobile layout) is preserved. |
| **SF-1A (Novel FactSet Diagnostics)** | All URD600–699 diagnostics appear as inline squigglies (Phase 1) and in hover tooltips showing orphan status (Phase 3). |
| **SF-2 (PropertyDependencyIndex)** | Read/write counts from the PropertyDependencyIndex appear in hover tooltips for property references (Phase 3). |
| **SF-3 (Graph Visualisation)** | The FactSet data consumed by the existing graph views is the same data consumed by hover tooltips. No conflict, no duplication. |
| **Future VS Code extension** | The `urd-lsp` binary (from SF-5) provides the same four capabilities via LSP protocol. This brief provides the same capabilities in the browser without a protocol layer. The cursor resolver logic should stay aligned between the two implementations. |

## Sequencing

Phases are independently shippable but have a logical order:

```
Phase 1: Diagnostics  (standalone — introduces shared state)
    ↓
Phase 2: Autocomplete (standalone — reads shared state)
    ↓
Phase 3: Hover        (introduces cursor-resolver.ts — reused by Phase 4)
    ↓
Phase 4: Go-to-Def    (depends on cursor-resolver.ts from Phase 3)
```

Phase 1 should be done first because it introduces `playground-state.ts`, which all subsequent phases use. Phases 2 and 3 are independent of each other and can be done in either order. Phase 4 depends on Phase 3's cursor resolver.

Each phase can be merged and deployed independently. A visitor benefits from Phase 1 alone, from Phases 1+2, and so on. There is no all-or-nothing gate.

*End of Brief*
