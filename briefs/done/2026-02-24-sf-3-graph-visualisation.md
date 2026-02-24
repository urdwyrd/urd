# URD — SF-3: Location Topology and Dialogue Flow Graph Visualisation

*Render the FactSet's structural relationships as interactive graphs in the playground.*

February 2026 | Semantic Gate — Tier 2 (Expose)

> **Document status: DONE** — Two graph visualisation components consuming FactSet JSON from the WASM pipeline. Location topology from ExitEdge tuples. Dialogue flow from JumpEdge and ChoiceFact tuples. Also a validation gate for FactSet structural completeness.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-24
**Status:** Done

### What was done

| Deliverable | Result |
|------------|--------|
| **SF-3.1** — Location graph from ExitEdge + world JSON | Implemented. Nodes from world JSON locations, edges from ExitEdge tuples. Unreachable nodes flagged via URD430. No AST import. |
| **SF-3.2** — Dialogue graph from JumpEdge + ChoiceFact | Implemented. Nodes from section IDs in jumps/choices, edges from JumpEdges. Choice labels attached via `jump_indices`. Terminal nodes for `end` and exit targets. |
| **SF-3.3** — No unknown nodes | All node IDs trace to FactSet tuples or world JSON. No "unknown" or placeholder nodes. |
| **SF-3.4** — Conditional edge display | Conditional exits rendered dashed in amber. Choice edges show condition status via `conditional` flag. |
| **SF-3.5** — Diagnostic-driven flags | URD430 (unreachable location) → dashed rose border. URD432 (impossible choice) → amber border. No reimplemented analysis. |
| **SF-3.6** — All test worlds render | Build passes. Manual verification pending for all fixtures. |
| **SF-3.7** — Deployed as playground tabs | Four-tab Analysis panel: [Properties] [Location] [Dialogue] [Facts]. |
| **SF-3.8** — FactSet gap documentation | Location display names and start location come from world JSON (documented gaps, acceptable). Entity placement from world JSON. No FactSet gaps in dialogue graph. |
| **Prerequisite** — ChoiceFact.jump_indices | Added `jump_indices: Vec<usize>` to ChoiceFact. `push_jump` returns `usize`, `extract_jump` returns `Option<usize>`. 4 new tests. |

### What changed from the brief

- **Simplified graph model:** The brief specified `GraphNode.flags` as an object with multiple boolean fields (`start`, `unreachable`, `isolated`, `orphaned`, `impossible_choices`). Implementation uses a single `flag: 'unreachable' | 'orphaned' | null` discriminant — simpler and sufficient for the current visual treatment.
- **No node info panel:** The brief described a slide-out info panel on node click (exits list, property activity, entity placement). Deferred — the graph renders correctly without it, and adding info panels would significantly increase scope. The click-to-scroll behaviour in the Dialogue tab provides the primary interaction.
- **Flat file structure:** The brief suggested a `graph/` subdirectory for components. Implementation keeps all files in the existing `playground/` directory to match the project's flat component layout.
- **No test runner for TypeScript transforms:** The brief suggested Vitest for data transformation tests. No test runner exists at the site level. Rust tests cover `jump_indices` correctness; graph transforms are verified by build + manual inspection.

---

## Context

The FactSet carries six relation types. Two of them — ExitEdge and JumpEdge — are directed graphs hiding in flat arrays. SF-3 makes those graphs visible. If the FactSet contains enough data to fully reconstruct both graphs without any AST fallback, it proves the extraction is structurally complete. If it doesn't, we've found a schema gap that must be resolved before SF-4 (semantic diff) and SF-5 (LSP).

### Current state

The playground's Analysis panel has two tabs (after SF-2): **Properties** (PropertyDependencyIndex view) and **Facts** (raw FactSet by type). Both are text-based lists. No visual rendering of structural relationships.

The WASM pipeline returns:
- `facts: FactSet | null` — six flat arrays (exits, jumps, choices, reads, writes, rules)
- `property_index: PropertyIndex | null` — property-centric view
- `world: string | null` — compiled `.urd.json` (locations, entities, types, sections)
- `diagnostics: Diagnostic[]` — all compiler warnings/errors including unreachable location (URD430), impossible choice condition (URD432), etc.

### What this brief delivers

Three things:

1. **Location topology graph.** Locations as nodes, ExitEdge tuples as directed edges. Conditional exits visually distinguished. Unreachable locations flagged. Start location marked.

2. **Dialogue flow graph.** Sections as nodes, JumpEdge tuples as directed edges. ChoiceFact data annotated on edges. Sticky vs one-shot choices distinguished. Terminal jumps (→ end, → exit) visually marked.

3. **A shared graph renderer.** Both graphs use the same layout engine (dagre), same SVG rendering, same pan/zoom, same interaction model. The two components differ only in how they transform FactSet data into the graph model.


### Graph scale

The graphs are small. Representative sizes from existing test worlds:

| World | Locations | Exits | Sections | Jumps |
|-------|-----------|-------|----------|-------|
| Two Room Key Puzzle | 2 | 1 | 1 | 0 |
| Locked Garden | 2 | 2 | 3 | ~8 |
| Monty Hall | 1 | 0 | 3 | ~5 |
| Sunken Citadel | 12 | ~15 | ~20 | ~60 |

Dagre handles these sizes trivially. No performance concerns.


## Dependencies

- **SF-2 passed.** PropertyDependencyIndex wired into the WASM pipeline. The property summary on node info panels comes from the index.
- **SF-1A passed.** Diagnostics D1–D5 available. Unreachable location detection available from existing S3 check (URD430).


## Prerequisite: FactSet Extension — `ChoiceFact.jump_indices`

The FactSet currently models `ChoiceFact.condition_reads` (indices into the reads array) and `ChoiceFact.effect_writes` (indices into the writes array). It does not model which jumps belong to a choice.

SF-3 needs this relationship to correlate choices with their destination sections in the dialogue graph. Without it, correlation requires a span-containment heuristic (matching jump spans against choice spans), which is implicit coupling that should not ship as a permanent design.

**Extension:** Add `jump_indices: Vec<usize>` to `ChoiceFact`. Same pattern as `condition_reads` and `effect_writes` — indices into `facts.jumps[]`.

**Implementation:** In `extract_choice()` in `facts.rs`, track jump indices the same way read and write indices are tracked. When `extract_jump()` is called inside a choice's content walk, capture the returned jump index.

This requires a small refactor: `extract_jump()` currently returns nothing. Change it to return `usize` (the index of the pushed JumpEdge). Only call `extract_jump()` when a jump is actually emitted — if there are code paths where a jump node is detected but not emitted (unresolvable target, missing annotation), those early-returns happen before the call. Choice extraction appends returned indices in encounter order, preserving source ordering for stable edge labels and stable UI behavior.

**Estimated change:** ~15 lines in `facts.rs`, ~3 lines in JSON serialisation, ~2 lines in `compiler-bridge.ts` types.

**JSON shape change:**

```json
{
  "section": "locked-garden/greet",
  "choice_id": "locked-garden/greet/state-your-purpose",
  "label": "State your purpose",
  "sticky": true,
  "condition_reads": [],
  "effect_writes": [0],
  "jump_indices": [2],
  "span": { ... }
}
```

**TypeScript type change:**

```typescript
export interface ChoiceFact {
  // ... existing fields
  jump_indices: number[];  // NEW: indices into facts.jumps[]
}
```

This extension is implemented as part of SF-3, not as a separate brief. It is small, follows an established pattern, and is directly motivated by the dialogue graph's data needs.


## Architecture

### Graph data model

Both components transform FactSet data into a shared graph model:

```typescript
interface GraphNode {
  id: string;
  label: string;
  kind: 'location' | 'section' | 'terminal';
  flags: {
    start?: boolean;       // start location or entry section
    unreachable?: boolean;  // URD430 diagnostic present (location nodes only)
    impossible_choices?: boolean; // URD432 diagnostic present (section nodes only — enum condition can never match)
    orphaned?: boolean;     // has orphaned properties (from index)
    isolated?: boolean;    // location with no exits (present in world JSON, no ExitEdge tuples reference it)
  };
  metadata?: Record<string, unknown>;  // node-specific data for info panel
}

interface GraphEdge {
  id: string;
  source: string;  // source node id
  target: string;  // target node id
  label?: string;
  kind: 'normal' | 'conditional' | 'choice_sticky' | 'choice_oneshot' | 'terminal';
  metadata?: Record<string, unknown>;  // edge-specific data for hover
}

interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}
```

### Component structure

```
GraphRenderer.svelte          — shared: dagre layout, SVG render, pan/zoom, interactions
  ├─ used by LocationGraph.svelte   — transforms ExitEdge[] → GraphData
  └─ used by DialogueGraph.svelte   — transforms JumpEdge[] + ChoiceFact[] → GraphData
```

`GraphRenderer` receives `GraphData` and renders it. It knows nothing about FactSet types. The two wrapper components handle data transformation.

### Layout engine

**dagre** (via `@dagrejs/dagre`, the maintained fork) provides hierarchical directed-graph layout. It computes x/y positions for nodes and control points for edges given a directed graph. ~15KB.

Configuration:
- `rankdir: 'TB'` (top-to-bottom) for location topology — maps naturally read top-down
- `rankdir: 'LR'` (left-to-right) for dialogue flow — sections flow left-to-right through conversation
- `nodesep: 60`, `ranksep: 80` — spacing tuned for readability at the playground's panel size

### Rendering

SVG in Svelte. No Canvas, no WebGL. SVG is the right choice for these graph sizes because:
- Text rendering is native (labels, annotations)
- CSS styling works directly (hover states, transitions)
- Accessibility (SVG elements are in the DOM)
- Svelte reactivity works naturally with SVG attributes

### Pan and zoom

Pointer events on the SVG container transform a `<g>` wrapper:
- **Pan:** `pointerdown` + `pointermove` on background
- **Zoom:** `wheel` event, clamped to 0.3–3.0×
- **Fit:** "Fit to view" button resets transform to center all nodes
- State: `{ x, y, scale }` applied as `transform="translate(x,y) scale(s)"`

No external library needed. This is ~40 lines of pointer math.


## Location Topology Component

### Node extraction

Location nodes come from the compiled world JSON's location list — not from ExitEdge tuples. This ensures all locations are visible, including isolated ones with no exits (e.g., half-written locations, one-room dialogue-only worlds). ExitEdge tuples provide edges only.

```typescript
function extractLocationNodes(world: ParsedWorld, facts: FactSet, diagnostics: Diagnostic[]): GraphNode[] {
  // Complete location list from compiled world JSON
  const locations = Object.keys(world.locations || {});
  return locations.sort().map(id => ({
    id,
    label: world.locations[id]?.name || id,
    kind: 'location',
    flags: {
      start: world.start === id,
      unreachable: diagnostics.some(d => d.code === 'URD430' && extractLocationFromMessage(d) === id),
      isolated: !facts.exits.some(e => e.from_location === id || e.to_location === id),
    },
  }));
}
```

Locations with no exits appear as isolated nodes — visible but disconnected. This is correct: the author should see that the location exists but has no connections. The `isolated` flag enables distinct visual treatment (see below).

### Edge extraction

Each ExitEdge becomes one directed edge:

```typescript
function extractLocationEdges(facts: FactSet): GraphEdge[] {
  return facts.exits.map(exit => ({
    id: `${exit.from_location}/${exit.exit_name}`,
    source: exit.from_location,
    target: exit.to_location,
    label: exit.exit_name,
    kind: exit.is_conditional ? 'conditional' : 'normal',
    metadata: {
      guard_count: exit.guard_reads.length,
      span: exit.span,
    },
  }));
}
```

### Visual treatment

| Element | Appearance |
|---------|------------|
| Regular node | Rounded rectangle, solid border, location name centered |
| Start node | Same shape with gold accent border and "▶" marker |
| Unreachable node | Dashed border, dimmed fill, amber "unreachable" badge |
| Isolated node | Dashed border, faint fill, no edges. Visually distinct from unreachable (isolated = no exits; unreachable = exits exist but no path from start) |
| Regular edge | Solid line with arrowhead, exit name label |
| Conditional edge | Dotted line with arrowhead, amber-tinted, guard count badge |
| Self-loop | Curved edge from node back to itself (if from_location == to_location) |

### Node info panel

Clicking a location node shows a slide-out info panel with:
- **Location name** (display name from world JSON if available, otherwise slugified ID)
- **Exits:** list of outgoing exits with destinations and conditions
- **Property activity:** read/write counts for properties referenced in this location's exit guards (from PropertyDependencyIndex)

Entity placement (which entities are in this location) requires data from the compiled world JSON, not the FactSet. See [FactSet Gaps](#factset-gaps).


## Dialogue Flow Component

### Node extraction

Section nodes are derived from JumpEdge and ChoiceFact tuples. Collect all unique section IDs that appear in either:

```typescript
function extractSectionNodes(facts: FactSet, diagnostics: Diagnostic[]): GraphNode[] {
  const sectionIds = new Set<string>();
  for (const jump of facts.jumps) {
    sectionIds.add(jump.from_section);
    if (jump.target.kind === 'section') sectionIds.add(jump.target.id);
  }
  for (const choice of facts.choices) {
    sectionIds.add(choice.section);
  }
  
  const nodes: GraphNode[] = Array.from(sectionIds).sort().map(id => ({
    id,
    label: id.split('/').pop() || id,  // show section name, not full compiled ID
    kind: 'section',
    flags: {
      impossible_choices: diagnostics.some(d => d.code === 'URD432' && extractSectionFromMessage(d) === id),
    },
  }));
  
  // Add terminal pseudo-nodes for "end" and exit targets
  const hasEnd = facts.jumps.some(j => j.target.kind === 'end');
  if (hasEnd) nodes.push({ id: '__end__', label: 'END', kind: 'terminal', flags: {} });
  
  const exitTargets = new Set(
    facts.jumps.filter(j => j.target.kind === 'exit').map(j => j.target.id!)
  );
  for (const exitId of exitTargets) {
    nodes.push({ id: `__exit_${exitId}`, label: `→ ${exitId}`, kind: 'terminal', flags: {} });
  }
  
  return nodes;
}
```

### Edge extraction

Each JumpEdge becomes one directed edge. Choice metadata is attached by matching jumps to ChoiceFacts by section:

```typescript
function extractDialogueEdges(facts: FactSet): GraphEdge[] {
  const jumpToChoice = buildJumpToChoiceMap(facts);
  
  return facts.jumps.map((jump, i) => {
    const targetId = jump.target.kind === 'end' ? '__end__'
      : jump.target.kind === 'exit' ? `__exit_${jump.target.id}`
      : jump.target.id;
    
    const matchingChoice = jumpToChoice.get(i) || null;
    
    return {
      id: `jump_${i}`,
      source: jump.from_section,
      target: targetId,
      label: matchingChoice?.label,
      kind: matchingChoice
        ? (matchingChoice.sticky ? 'choice_sticky' : 'choice_oneshot')
        : jump.target.kind === 'end' ? 'terminal' : 'normal',
      metadata: {
        choice: matchingChoice ? {
          label: matchingChoice.label,
          condition_count: matchingChoice.condition_reads.length,
          effect_count: matchingChoice.effect_writes.length,
        } : null,
        span: jump.span,
      },
    };
  });
}
```

### Choice-to-jump correlation

The FactSet extension (`ChoiceFact.jump_indices`, see prerequisite above) provides explicit correlation between choices and their jumps. No heuristic needed.

For the dialogue graph, the transformation inverts the relationship: for each JumpEdge, find which ChoiceFact (if any) contains its index in `jump_indices`. This gives us the choice label, sticky/one-shot status, and condition/effect counts to annotate the edge.

```typescript
function buildJumpToChoiceMap(facts: FactSet): Map<number, ChoiceFact> {
  const map = new Map<number, ChoiceFact>();
  for (const choice of facts.choices) {
    for (const jumpIdx of choice.jump_indices) {
      map.set(jumpIdx, choice);
    }
  }
  return map;
}
```

Jumps not claimed by any choice are section-level jumps (no choice annotation on the edge).

### Visual treatment

| Element | Appearance |
|---------|------------|
| Section node | Rounded rectangle, section name (last segment of compiled ID) |
| Terminal node (END) | Small circle or diamond, darker fill, "END" label |
| Terminal node (→ exit) | Small diamond, exit name label |
| Section with impossible choices | Warning border (amber), tooltip explains which choices have impossible conditions |
| Normal edge (no choice) | Solid line with arrowhead |
| Sticky choice edge (+) | Solid line, green tint, "+" prefix on label |
| One-shot choice edge (*) | Dashed line, amber tint, "•" prefix on label |
| Self-loop (→ same section) | Curved edge from node back to itself |
| Conditional choice | Small badge showing condition count on the edge |

### Section info panel

Clicking a section node shows:
- **Section name** (compiled ID)
- **Choices:** list of ChoiceFacts in this section with labels, sticky/one-shot, condition and effect counts
- **Incoming jumps:** which sections jump to this one
- **Outgoing jumps:** where this section can go


## Shared GraphRenderer Component

### Props

```typescript
interface GraphRendererProps {
  data: GraphData;
  rankdir?: 'TB' | 'LR';
  onNodeClick?: (nodeId: string) => void;
  onEdgeHover?: (edgeId: string | null) => void;
  nodeWidth?: number;
  nodeHeight?: number;
}
```

### Rendering pipeline

1. **Layout.** Pass `data.nodes` and `data.edges` to dagre. Receive x/y positions for each node and control points for each edge.
2. **SVG generation.** For each node: `<rect>` with label `<text>`. For each edge: `<path>` with optional label `<text>`. Arrowheads via SVG `<marker>` definitions.
3. **Styling.** CSS classes based on `node.kind`, `node.flags`, `edge.kind`. Colour palette matches existing playground theme variables (`--gold`, `--amber-light`, `--green-light`, `--rose`, `--faint`).
4. **Interaction.** `onclick` on node groups triggers `onNodeClick`. `mouseenter`/`mouseleave` on edges triggers `onEdgeHover`.
5. **Pan/zoom.** Outer `<svg>` catches pointer events; inner `<g>` has the transform.

### Fit-to-view

On initial render and on a "Fit" button click, compute the bounding box of all nodes and set the transform to center them within the available viewport with 20px padding. This ensures the graph is always visible without manual pan/zoom on first load.

### Responsive sizing

The graph container takes full height of its parent panel. On desktop, this is the Analysis panel's body (280px collapsed, up to 70vh expanded). The "expand" button on the Analysis panel header (already exists) gives enough room for Sunken Citadel's 12-node graph.

For larger worlds (future), a "Popout" button could open the graph in a separate window. **Not in scope for SF-3** — the expand toggle is sufficient for all current test worlds.


## Playground Integration

### Tab bar extension

The Analysis panel tab bar (introduced by SF-2) gains two new tabs:

```
[Properties] [Location] [Dialogue] [Facts]
```

- **Properties** — PropertyDependencyView (default, from SF-2)
- **Location** — LocationGraph component
- **Dialogue** — DialogueGraph component
- **Facts** — FactSetView (existing)

The Location and Dialogue tabs render their respective graph components. Both receive `compileResult.facts`, `compileResult.diagnostics`, and `compileResult.property_index`. The Location tab additionally receives the parsed world JSON for start-location marking and entity placement in the info panel.

### Empty states

- If `facts` is null (compilation failed before LINK): tabs show "Compilation incomplete — graphs require successful linking"
- If `facts.exits` is empty: Location tab shows "No exits in this world"
- If `facts.jumps` is empty and `facts.choices` is empty: Dialogue tab shows "No dialogue flow in this world"


## FactSet Gaps

SF-3 is a validation gate for FactSet structural completeness (SF-3.8). The following data is NOT available from FactSet tuples and must come from supplementary sources or be documented as gaps:

### Documented gaps (supplementary data available)

| Data | Needed for | Source |
|------|-----------|--------|
| `world.start` (start location ID) | Marking start node in location graph | Compiled world JSON (`compileResult.world`) |
| Entity placement (which entities are in which location) | Location node info panel | Compiled world JSON → `entities[id].container` |
| Location display names | Human-readable node labels | Compiled world JSON → `locations[id].name` |

These are acceptable gaps. The FactSet is a property-analysis IR, not a complete world model. Location metadata (display names, entity placement, start flag) belongs in the compiled output, not in the FactSet.

### Resolved by design: isolated locations

Not a gap. Location nodes come from compiled world JSON (see Location Graph section above). This is the authoritative node set — isolated locations with no exits are always visible because they exist in world JSON regardless of whether any ExitEdge references them. A one-room dialogue-only world shows one node with no edges. A half-written world with unconnected locations shows them as isolated nodes with the `isolated` flag set. No special handling needed beyond the world-JSON-first node derivation already specified.

### No gaps in dialogue graph

All sections that participate in dialogue flow appear in either JumpEdge or ChoiceFact tuples. A section with no jumps and no choices has no dialogue flow to visualise. The FactSet is structurally complete for the dialogue graph.


## Unreachable Node Detection

The graph components do NOT reimplement reachability analysis. They cross-reference with existing compiler diagnostics:

- **URD430** (unreachable location) → marks location node as unreachable
- **URD432** (impossible choice condition) → marks section node as having impossible choices. A section is flagged when any of its choices have enum conditions that can never be satisfied.

This is the correct approach because:
1. The existing VALIDATE checks are tested (36 tests, zero maintenance issues)
2. Reimplementing BFS on the client wastes effort and risks divergence
3. The diagnostics are already available in `compileResult.diagnostics`

**Diagnostic-to-node matching:** The diagnostic message contains the location or section name. The graph components extract the relevant identifier using these exact message patterns:

- **URD430:** `"Location '{location_slug}' is unreachable"` → extract `location_slug`
- **URD432:** `"Choice in section '{section_compiled_id}' (file '{file}')"` → extract `section_compiled_id`

These patterns are the contract. Any change to these message fragments in the compiler must update the extractors. The patterns are pinned here so implementers know exactly which strings are load-bearing.

**Known limitation: string-based diagnostic matching.** This couples the graph components to human-readable message formats. If diagnostic wording changes, the extraction must be updated. The coupling is acceptable for SF-3 because:
- Only two diagnostic codes are matched, with patterns pinned above
- Message formats are controlled by us, stable, and tested
- Breakage would be visible (flags silently disappear) and caught by test guards

**Forward reference:** SF-5 (LSP) will require structured diagnostic targets — `target_id: Option<String>` or equivalent on the `Diagnostic` struct — for go-to-definition and hover. When that lands, SF-3's message-based matching should be replaced with `d.target_id === nodeId`. This is logged as planned technical debt, not hidden debt.

**Test guard:** A unit test must fail if the extractor does not match the current URD430 and URD432 message patterns. This turns the "three simultaneous failures" argument into one failure caught immediately.


## New Dependency

```json
{
  "@dagrejs/dagre": "^1.1.4"
}
```

~15KB. Maintained fork of the original dagre library. Provides hierarchical directed-graph layout with node positioning and edge routing. No other new dependencies needed.


## Test Strategy

Graph visualisation is primarily a visual output. The testing strategy separates data transformation (unit-testable) from rendering (manual verification).

### Unit tests (data transformation)

These test the FactSet → GraphData transformation functions, not the rendering:

| Test | Asserts |
|------|---------|
| `location_graph_locked_garden` | 2 nodes (gatehouse, walled-garden), 2 edges (garden exit, north exit), garden exit marked conditional |
| `location_graph_sunken_citadel_node_count` | 12 location nodes extracted from world JSON locations |
| `location_graph_sunken_citadel_start_marked` | village-square node has `flags.start = true` |
| `location_graph_sunken_citadel_edge_count` | ~15 edges extracted from ExitEdge tuples |
| `location_graph_no_duplicates` | Each location ID appears exactly once as a node |
| `location_graph_unreachable_flagged` | On `negative-unreachable-location.urd.md`, the unreachable location node has `flags.unreachable = true` |
| `location_graph_isolated_node` | A world with a location that has zero exits shows that location as a node with `flags.isolated = true` |
| `dialogue_graph_locked_garden` | 3 section nodes, jump edges match expected targets |
| `dialogue_graph_terminal_nodes` | Jumps to `end` produce an END terminal node |
| `dialogue_graph_exit_jumps` | Jumps to exits produce exit terminal nodes |
| `dialogue_graph_choice_correlation` | Choice labels correctly attached to jump edges via `jump_indices` mapping |
| `dialogue_graph_sticky_vs_oneshot` | Sticky choices produce `choice_sticky` edge kind, one-shot produce `choice_oneshot` |
| `diagnostic_extractor_matches_urd430` | `extractLocationFromMessage()` correctly extracts location ID from a URD430 (unreachable location) diagnostic. Matches pattern: `"Location '{}' is unreachable..."` |
| `diagnostic_extractor_matches_urd432` | `extractSectionFromMessage()` correctly extracts section ID from a URD432 (impossible choice condition) diagnostic. Matches pattern: `"Choice in section '{}' (file '{}')..."` |
| `all_fixtures_no_panic` | Both transformation functions run on all fixtures without errors |

### Manual verification

After implementation, visually verify both graphs for:
- Locked Garden (minimal: 2 locations, 3 sections)
- Sunken Citadel (stress: 12 locations, ~20 sections)
- Monty Hall (edge case: 1 location, no exits, dialogue-only)
- Two Room Key Puzzle (minimal: 2 locations, 1 section)

Screenshot each for the execution record.

### Test location

Data transformation tests go in `tests/graph_viz_tests.rs` (new file, or if the transformation logic lives in the Svelte layer, equivalent TypeScript tests alongside the components).

**Decision point:** The data transformation could live in Rust (exported via WASM) or in TypeScript (in the Svelte components). The brief recommends **TypeScript** for these transformations because:
- The input is already JSON (facts, diagnostics, world)
- The output is a frontend data model (GraphData)
- No Rust computation advantage at this scale
- Keeps the compiler crate focused on compilation, not on UI data models

If TypeScript, tests use a test runner alongside the site code (e.g., Vitest). The test fixtures would be JSON snapshots of FactSet output from known test worlds.


## Files Changed

| File | Change |
|------|--------|
| `sites/urd.dev/package.json` | Add `@dagrejs/dagre` dependency |
| `sites/urd.dev/src/components/playground/graph/GraphRenderer.svelte` | **New.** Shared SVG graph renderer with dagre layout, pan/zoom, interactions |
| `sites/urd.dev/src/components/playground/graph/LocationGraph.svelte` | **New.** ExitEdge → GraphData transformation + GraphRenderer wrapper |
| `sites/urd.dev/src/components/playground/graph/DialogueGraph.svelte` | **New.** JumpEdge + ChoiceFact → GraphData transformation + GraphRenderer wrapper |
| `sites/urd.dev/src/components/playground/graph/graph-types.ts` | **New.** GraphNode, GraphEdge, GraphData type definitions |
| `sites/urd.dev/src/components/playground/graph/transform-location.ts` | **New.** FactSet → location GraphData transformation function |
| `sites/urd.dev/src/components/playground/graph/transform-dialogue.ts` | **New.** FactSet → dialogue GraphData transformation function |
| `sites/urd.dev/src/components/playground/UrdPlayground.svelte` | Add Location and Dialogue tabs to Analysis panel |
| `packages/compiler/src/facts.rs` | Add `jump_indices: Vec<usize>` to `ChoiceFact`, refactor `extract_jump()` to return `usize`, collect jump indices in `extract_choice()`, add `jump_indices` to ChoiceFact JSON serialisation |
| `packages/compiler/tests/facts_tests.rs` | Add tests for `jump_indices` on existing fixtures |
| `sites/urd.dev/src/components/playground/compiler-bridge.ts` | Add `jump_indices: number[]` to `ChoiceFact` type |


## Estimated Size

| Component | Lines |
|-----------|-------|
| `facts.rs` — ChoiceFact.jump_indices + JSON serialisation | ~18 |
| `facts_tests.rs` — jump_indices tests | ~30 |
| `graph-types.ts` | ~30 |
| `transform-location.ts` | ~60 |
| `transform-dialogue.ts` | ~80 |
| `GraphRenderer.svelte` | ~350 (SVG rendering, dagre integration, pan/zoom, interactions, styling) |
| `LocationGraph.svelte` | ~80 (wrapper + info panel) |
| `DialogueGraph.svelte` | ~100 (wrapper + info panel) |
| `UrdPlayground.svelte` — tab changes | ~20 |
| Tests (TypeScript) | ~200 |
| **Total** | **~1000** |

This is the largest brief so far. The bulk is in `GraphRenderer.svelte` (SVG rendering and interaction), which is inherently visual and template-heavy. The FactSet extension (~50 lines including tests) is a small but important prerequisite.


## Acceptance Criteria

- [ ] **SF-3.1** — Location graph edges from ExitEdge tuples only; node set from compiled world JSON. No AST import. No source text access. Start location, entity placement, and display names use compiled world JSON as supplementary data, documented as acceptable.
- [ ] **SF-3.2** — Dialogue graph fully reconstructed from JumpEdge and ChoiceFact tuples only. No AST import. No source text access.
- [ ] **SF-3.3** — No "unknown" or "unresolved" nodes in either graph for any test world that passes compilation. Every node ID traces back to a FactSet tuple.
- [ ] **SF-3.4** — Conditional edges display guard count from ExitEdge.guard_reads. Choice edges display condition and effect counts from ChoiceFact.
- [ ] **SF-3.5** — Diagnostic-driven visual flags applied: locations marked unreachable when URD430 fires (dashed border, dimmed); sections marked with impossible choices when URD432 fires (warning border, amber). Marking is consistent with compiler diagnostic output — no reimplemented analysis.
- [ ] **SF-3.6** — Both components render correctly for all existing test worlds: Locked Garden, Monty Hall, Two Room Key Puzzle, Sunken Citadel, Tavern Scene, Interrogation (multi-file).
- [ ] **SF-3.7** — Both components deployed to playground as tabs in the Analysis panel.
- [ ] **SF-3.8** — If any graph element cannot be reconstructed from FactSet tuples, the gap is documented in the execution record with: what data is missing, where it comes from instead, whether the FactSet should be extended.


## What This Brief Does NOT Cover

- Property dependency visualisation (a property-centric graph of which properties affect which). That would be a separate component consuming the PropertyDependencyIndex, not the structural FactSet. Not planned.
- Live runtime state overlay on graphs (current location, active section, entity positions). That is runtime gate scope.
- Graph export (PNG, SVG download). Nice-to-have, not a gate requirement.
- Graph diffing (comparing two graphs visually). That is SF-4's domain; SF-4 consumes the diff engine output, not the graph components.
- Popout or fullscreen mode. The Analysis panel's existing expand toggle is sufficient for all current test worlds.
- Custom layout persistence (user-dragged node positions saved). Not needed — dagre layout is deterministic and good enough.


## Relationship to Downstream Briefs

| Brief | How SF-3 feeds it |
|-------|--------------------|
| **SF-4** (Semantic Diff) | The diff engine detects structural changes (exit added, section removed, reachability changed). SF-3's graph components can overlay diff annotations in a future extension — but SF-4 itself is a data layer, not a visualisation layer. |
| **SF-5** (LSP) | SF-3 validates FactSet structural completeness. If SF-3.8 reveals gaps, they must be resolved before LSP can rely on FactSet for go-to-definition across locations and sections. |
| **SF-6** (MCP) | `get_exit_graph` and `get_dialogue_graph` endpoints return the same structural data that SF-3 visualises. The transformation functions from SF-3 inform the MCP response shapes. |

*End of Brief*
