# Urd Forge — World Insight Panel

*A unified, information-dense analysis panel that consolidates the compiled world's FactSet, PropertyDependencyIndex, DefinitionIndex, and urd.json into a single interactive surface — the Forge equivalent of the playground's Analysis section, but deeper, richer, and designed for serious authoring.*

February 2026 | Active

> **Document status: BRIEF** — Specifies a new Forge view called **World Insight** that presents the full semantic analysis of a compiled Urd world in a nested, filterable, cross-linked panel. Inspired by the playground's Analysis section (Properties / Location / Dialogue / Facts tabs) but reimagined for the IDE context: denser layout, multi-level nesting, bidirectional navigation to source, entity-aware filtering, and visual status indicators. No new compiler work — all data already exists in projections.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-28
**Status:** Complete

### What was done

Built the complete World Insight Panel as a singleton-autocreate analysis view (`urd.worldInsight`) with all 7 sections specified in the brief:

1. **World Summary** — always-visible metadata grid with world name, start/entry links, count badges (entities, locations, sections, types, rules, facts), diagnostic counts, compile time. Badge clicks scroll to relevant section.
2. **Entity Atlas** — type-grouped entity tree with trait badges. Expandable to show per-entity properties table with read/write counts, orphan indicators, containment info, and affecting rules. Type filter dropdown.
3. **Property Flow** — property dependency list with summary bar and filter modes (All/Read-only/Write-only/Orphaned). Expandable rows show individual read and write sites with kind, operator, value, and clickable source references.
4. **Dialogue Outline** — sections grouped from FactSet choices and jumps, ordered by source line. Expandable to show choice markers (*one-shot/+sticky), labels, condition lines (?), effect lines (>), jump targets, and badges.
5. **Location Map** — location outline with exit count, entity count, and start badge. Expandable to show exits with targets/conditions, entities with inline property snapshots.
6. **Rules Engine** — rules from world.rules cross-referenced with factSet.rules. Expandable to show conditions, effects, selectors, triggers.
7. **Diagnostics** — grouped by file with per-file counts. Severity icons (error/warning/info) with colour coding.

Cross-cutting features:
- Global filter input with 150ms debounce, filtering across all sections simultaneously
- Selection context integration: when an entity is selected elsewhere, the atlas auto-expands to it and properties filter to its type
- Keyboard navigation (arrow keys, Enter to expand, Ctrl+Enter to navigate)
- Bidirectional source navigation via NavigationBroker
- Collapsed/expanded state persistence via zone state
- Analyst workspace template (Code Editor 60% | World Insight 40%)

16 files created, 2 files modified. All data sourced from existing projections (urdJson, factSet, propertyDependencyIndex, diagnosticsByFile, entityTable, symbolTable, worldStats). No compiler changes needed.

### What changed from the brief

- The placeholder `2026-02-28-forge-analysis-panel.md` brief was deleted as it contained no specification.
- Phase 5's cursor-tracking (Source-to-Insight navigation via `editor.cursorChanged`) was not implemented because no such bus channel exists. Selection context provides the equivalent functionality.
- Data flow arrows in Property Flow (mini inline diagram showing write sites -> property -> read sites) were not implemented to keep complexity manageable. The read/write site expansion provides the same information in a more detailed format.
- Keyboard navigation uses a simpler approach than the brief's flat-list model: arrow keys navigate between focusable elements using the browser's existing tabindex system rather than maintaining a separate virtual cursor position.

---

## Context

### The playground's Analysis section

The playground (`urd.dev/playground`) already has a collapsible Analysis panel below the editor/output split. It has four tabs:

1. **Properties** (`PropertyDependencyView.svelte`) — lists every property from the `PropertyDependencyIndex`. Summary bar shows total properties, reads, writes, orphan counts. Filter buttons (All / Read-only / Write-only). Each property row is expandable: click to reveal individual read and write sites with operators, values, and source line references. Every row is clickable → scrolls the editor to the source location.

2. **Location** (`LocationGraph.svelte`) — Dagre-based directed graph of locations connected by exits. Uses `GraphRenderer.svelte` (shared SVG canvas with pan, zoom, fit-to-view). Edge annotations show exit names and conditional guards.

3. **Dialogue** (`DialogueGraph.svelte`) — section-level flow graph. Nodes are sections, edges are jumps and choice targets. Choice annotations (sticky `+` vs one-shot `*`). Terminal pseudo-nodes for `end` and exit targets. Node click → scroll to source.

4. **Facts** (`FactSetView.svelte`) — flat categorised list of every extracted fact: exits, choices, reads, writes, jumps, rules. Collapsible sections with counts. Each fact row is clickable → scroll to source.

### What already exists in Forge

Forge already has individual views that cover much of this ground separately:

- **Spreadsheets**: Entity, Type, Property, Location, Section, Diagnostic (6 views)
- **Graphs**: Location Network, Dialogue Flow, Type Hierarchy, Property Data Flow, Containment Tree, Cross-Reference (6 views)
- **Analysis**: WorldStatsDashboard, DeadCodePanel, ConditionEffectMatrix, ReachabilityMatrix, ThresholdAnalysis, VisibilityAudit, CircularDependency, EnumCoverage, NarrativeFlowVisualiser, DiffView (10 views)
- **Inspectors**: PropertyInspector (1 view)

The problem is not missing data — it's **fragmentation**. To understand a single entity's full story, a writer currently needs to check the Entity Spreadsheet, the Property Spreadsheet, the Property Data Flow Graph, the Condition/Effect Matrix, and the PropertyInspector. That's five views for one question.

### What the World Insight Panel does differently

It answers the question: *"Show me everything interesting about this world in one place."*

Rather than organising by data type (all entities in one spreadsheet, all properties in another), it organises by **semantic concern** — each section answers a specific authoring question. The panel is not a replacement for the individual views; it's an **entry point** that surfaces the most useful cross-cutting insights and lets you drill into any of them.

---

## Design Principles

1. **Information density over whitespace.** This is a power-user panel. Compact rows, small type for metadata, generous use of inline badges and colour coding. Closer to a terminal or database inspector than a marketing dashboard.

2. **Everything is clickable.** Every entity reference, property name, section label, and source location navigates somewhere — either expanding an inline detail section, scrolling the code editor, or opening the relevant specialised view.

3. **Nesting reveals detail progressively.** Top level shows summaries. First click expands to a detail row. Second click expands to individual read/write sites. Third click navigates to source. The panel never overwhelms on first render.

4. **Colour is semantic, not decorative.** Entity types get consistent hue assignments (same colours as the graph views). Orphaned properties get amber warnings. Unreachable sections get rose indicators. Green for healthy, gold for noteworthy, rose for problems.

5. **Selection context drives focus.** When a user selects an entity in another view (or places their cursor on an `@entity` reference in the editor), the World Insight Panel can filter/highlight to show that entity's full picture.

6. **One panel, not five tabs.** Unlike the playground's tab-based approach, the Forge panel uses **vertically stacked collapsible sections** — all sections visible at once when expanded, with a scrollable viewport. This lets authors see properties and dialogue flow simultaneously.

---

## Panel Structure

The panel is a single scrollable column divided into seven collapsible sections. Each section has a header bar with title, count badge, and optional filter/action buttons. Sections remember their collapsed/expanded state across sessions.

### Section 1: World Summary

A compact header block, always visible (not collapsible).

| Field | Source | Display |
|---|---|---|
| World name | `urd.json → world.meta.id` | Bold title |
| Start location | `urd.json → world.meta.start` | Clickable link |
| Entry section | `urd.json → world.meta.entry` | Clickable link |
| Entity count | `urd.json → world.entities.length` | Badge |
| Location count | `urd.json → world.locations.length` | Badge |
| Section count | factSet.jumps + factSet.choices (unique sections) | Badge |
| Type count | `urd.json → world.types.length` | Badge |
| Rule count | `factSet.rules.length` | Badge |
| Fact count | total across all FactSet arrays | Badge |
| Diagnostic count | by severity (error / warning / info) | Colour-coded badges |
| Compile time | from `OutputHeader.timings` | Mono text |

Layout: two-column grid, compact. Diagnostic badges use the standard rose/amber/blue semantic colours. Clicking any count badge scrolls to the relevant section below.

### Section 2: Entity Atlas

**Header:** `ENTITIES` · count badge · filter input · type filter dropdown

A nested tree view of every entity in the world, grouped by type.

**Level 0 — Type group row:**
```
▸ Door (3)                                    [interactable]
```
Shows entity type name, instance count, trait badges. Click to expand.

**Level 1 — Entity row:**
```
  ▸ @door_1        Door    Tavern    prize:car  state:closed  chosen:false
```
Shows entity ID (clickable → editor), type, containing location (clickable → editor), and inline property values as compact `key:value` pairs. Hidden properties (`~` prefix) shown with a 🔮 icon. Click to expand.

**Level 2 — Entity detail:**
When an entity row is expanded, show:
- **Properties table**: each property as a row with name, type, default value, current initial value, read count badge, write count badge, orphan status indicator
- **Containment**: what this entity contains (if anything), and who contains it
- **Referenced by**: which sections, choices, rules, and exits reference this entity (derived from FactSet cross-referencing `entity_type` fields)
- **Rules affecting**: rules whose conditions or effects touch this entity's properties

Each sub-item is clickable → navigates to source.

### Section 3: Property Flow

**Header:** `PROPERTIES` · count badge · filter buttons (All / Read-only / Write-only / Orphaned)

Direct evolution of the playground's PropertyDependencyView, enhanced for Forge.

**Level 0 — Property row:**
```
▸ Door.prize          3R  0W                                    read only ⚠
```
Shows `Type.property`, read count badge (green), write count badge (gold), orphan status badge (amber). Click to expand.

**Level 1 — Read/Write sites:**
```
  READS
    choice: Pick a door       == closed        tavern.urd.md:12
    choice: Switch to other   == closed        tavern.urd.md:18
    exit: gate_passage        != locked        world.urd.md:42
  WRITES
    choice: Order a drink     += 10            tavern.urd.md:14
```
Each site shows: site kind + ID, operator + value, file:line reference. Clicking the file:line navigates to source. Clicking the site ID navigates to that choice/exit/rule in the Dialogue Flow section.

**Enhancements over playground:**
- **Data flow arrows**: when a property has both reads and writes, show a mini inline diagram: `write sites → property → read sites` indicating the causal chain.
- **Cross-entity dependencies**: if `@door_1.state` is written by a rule that reads `@iron_key.exists`, show the dependency chain.

### Section 4: Dialogue Flow

**Header:** `DIALOGUE` · section count badge · choice count badge

A structured outline of the dialogue tree, showing sections, choices, jumps, and conditions — not as a graph, but as an **interactive outline** that reads like a table of contents of the narrative.

**Level 0 — Section row:**
```
▸ ## Choose                    3 choices  1 jump    tavern.urd.md:8
```
Shows section heading (matching the markdown `##` hierarchy), choice count, jump count, source reference. Click to expand.

**Level 1 — Choice/Jump detail:**
```
  * Pick a door → any Door                          [conditional]
      ? target.state == closed
      ? target.chosen == false
      > target.chosen = true
  + Stay with your choice                           [sticky]
  → Reveal                                          [auto]
```
Shows each choice with its marker (`*` one-shot, `+` sticky), label, target, condition lines (prefixed `?`), effect lines (prefixed `>`), and jump arrows (`→`). Badges: `[conditional]`, `[sticky]`, `[auto]`, `[terminal]`.

**Level 2 — Condition/effect detail** (on click):
Inline expansion showing the full property reference, operator, value, and a clickable link to the property in Section 3 above.

### Section 5: Location Map

**Header:** `LOCATIONS` · count badge · exit count badge

A textual outline of the location graph (the graph view is available separately — this is the list-based complement).

**Level 0 — Location row:**
```
▸ Tavern              2 exits  4 entities  ▶ start    world.urd.md:3
```
Shows location name, exit count, entity count, start badge if applicable, source reference. Click to expand.

**Level 1 — Location detail:**
```
  EXITS
    → Market           via front_door                [unconditional]
    → Cellar           via trapdoor                  ? @iron_key.found == true
  ENTITIES
    @innkeeper_bessa   NPC     trust:0  mood:neutral
    @iron_key          Item    found:false
  CONTAINS
    (nothing nested)
```
Shows exits with targets and conditions, entities with inline property snapshots, nested sub-locations if any.

### Section 6: Rules Engine

**Header:** `RULES` · count badge

**Level 0 — Rule row:**
```
▸ monty_reveals       2 conditions  1 effect       world.urd.md:15
```

**Level 1 — Rule detail:**
```
  CONDITIONS
    ? target.prize != car
    ? target.chosen == false
  EFFECTS
    > target.state = open
  SELECTOR
    @monty selects target from [@door_1, @door_2, @door_3]
```

### Section 7: Diagnostics

**Header:** `DIAGNOSTICS` · error/warning/info badges

**Level 0 — Diagnostic row:**
```
  ⛌  Property 'trust' is read but never written     URD412  world.urd.md:42
```
Uses severity icons: `⛌` error (rose), `▲` warning (amber), `ℹ` info (blue). Clicking navigates to source.

Grouped by file, with file headers showing per-file counts.

---

## Interaction Behaviours

### Bidirectional navigation

- **Insight → Source**: every source location reference (file:line) is a clickable link that dispatches a `NavigationIntent` to the Code Editor singleton, opening the file and scrolling to the line.
- **Source → Insight**: when the cursor in the Code Editor rests on an `@entity` reference, the World Insight Panel highlights the corresponding entity row (subtle gold left-border indicator) and optionally auto-scrolls to it.

### Selection context integration

When `selection.primary` on the bus carries an entity ID:
- The Entity Atlas section auto-expands that entity.
- The Property Flow section filters to show only that entity's properties.
- The Dialogue Flow section highlights sections that reference the entity.
- A "Clear filter" button appears in the panel header.

### Keyboard navigation

- `↑`/`↓` moves focus between rows
- `Enter` or `→` expands a row
- `←` collapses a row
- `Ctrl+Enter` navigates to source

### Search/filter

A global filter input at the top of the panel filters across all sections simultaneously. Typing `@innkeeper` shows only rows that reference that entity, across all seven sections. Sections with zero matches auto-collapse; sections with matches auto-expand.

---

## Visual Design

### Typography
- Section headers: `var(--display)`, 10px, uppercase, `letter-spacing: 0.03em`, `var(--faint)` — matches Forge convention.
- Row content: `var(--mono)`, 12px — entity IDs, property names, operators, values.
- Badges: `var(--mono)`, 10px, coloured background with matching text — same pill style as playground's `pd-badge`.
- Metadata (file:line, counts): `var(--mono)`, 10px, `var(--faint)`.

### Colour system
| Semantic | Token | Usage |
|---|---|---|
| Entity reference | `var(--amber-light)` | `@entity_id` text |
| Property value | `var(--green-light)` | Value literals in reads/writes |
| Section/jump target | `var(--green-light)` | `→ section_name` |
| Read badge | `green bg 15%, green text` | `3R` pill |
| Write badge | `gold bg 15%, gold text` | `2W` pill |
| Orphan warning | `amber bg 15%, amber text` | `read only` pill |
| Error | `var(--rose)` | Diagnostic icon and text |
| Warning | `var(--amber-light)` | Diagnostic icon and text |
| Rule ID | `var(--purple)` | Rule name text |
| Sticky choice marker | `var(--gold)` | `+` prefix |
| Conditional badge | `amber bg` | `[conditional]` pill |

### Row density
- Row height: 24px (single-line rows), expanding as needed for wrapped content.
- Indent per nesting level: 16px.
- Section header height: 28px with `var(--raise)` background.
- No divider lines between rows — rely on alternating subtle background tint on hover.

### Theme support
Both Gloaming (dark) and Parchment (light) themes, using existing Forge theme tokens. No hardcoded colours.

---

## Data Sources

All data comes from existing projections in the `ProjectionRegistry`. No new compiler output is needed.

| Section | Projections consumed |
|---|---|
| World Summary | `urdJson`, `factSet`, `diagnosticsByFile` |
| Entity Atlas | `urdJson`, `entityTable`, `factSet`, `propertyDependencyIndex` |
| Property Flow | `propertyDependencyIndex`, `factSet` |
| Dialogue Flow | `factSet` (jumps, choices), `urdJson` (sections) |
| Location Map | `urdJson` (locations), `factSet` (exits), `entityTable` |
| Rules Engine | `factSet` (rules, reads, writes) |
| Diagnostics | `diagnosticsByFile` |

The panel subscribes to `compiler.completed` on the message bus and rebuilds its derived state when projections update.

---

## Implementation

### Registration

```typescript
viewRegistry.register({
  typeId: 'world-insight',
  name: 'World Insight',
  category: 'Analysis',
  singleton: 'singleton-autocreate',
  icon: '◆',
  projectRequired: true,
});
```

Singleton with autocreate — there's only one World Insight panel per workspace, and it creates automatically when a workspace template includes it.

### Component structure

```
views/analysis/world-insight/
  WorldInsightZone.svelte          ← top-level zone view
  InsightSection.svelte            ← shared collapsible section wrapper
  WorldSummary.svelte              ← Section 1
  EntityAtlas.svelte               ← Section 2
  EntityRow.svelte                 ← nested entity row with expand
  PropertyFlow.svelte              ← Section 3
  PropertyFlowRow.svelte           ← nested property row with expand
  DialogueOutline.svelte           ← Section 4
  SectionRow.svelte                ← nested section row with expand
  LocationMap.svelte               ← Section 5
  LocationRow.svelte               ← nested location row with expand
  RulesEngine.svelte               ← Section 6
  RuleRow.svelte                   ← nested rule row with expand
  DiagnosticList.svelte            ← Section 7
  insight-state.ts                 ← derived state from projections
  insight-filter.ts                ← global filter logic
```

### Phases

**Phase 1: Shell + Summary + Diagnostics.** Register the view. Build `InsightSection.svelte` as the shared collapsible wrapper. Implement `WorldSummary.svelte` (straightforward data display from `urdJson`) and `DiagnosticList.svelte` (already exists as a spreadsheet — this is a compact re-rendering). Wire to `compiler.completed`. **This is shippable alone.**

**Phase 2: Entity Atlas.** Build the nested entity tree with type grouping, inline property values, and expand-to-detail. This is the most complex section because it cross-references `urdJson`, `entityTable`, `factSet`, and `propertyDependencyIndex`. **Shippable alone.**

**Phase 3: Property Flow.** Port and enhance the playground's `PropertyDependencyView` into the Forge component system. Add the data flow indicators and cross-entity dependency chains. **Shippable alone.**

**Phase 4: Dialogue Outline + Location Map + Rules Engine.** These three are structurally similar (nested outline with expandable detail) and can be built in one pass. **Shippable alone.**

**Phase 5: Selection context + global filter + keyboard navigation.** The cross-cutting interaction layer. Bus subscription for `selection.primary`, global filter input with multi-section search, arrow-key navigation. **Shippable alone.**

### Workspace integration

Add a new workspace template:

```
Analyst
├── Code Editor (60%)
└── World Insight (40%)
```

Also useful as a secondary panel in the existing Writer and Engineer templates.

---

## Success Criteria

1. A writer can open the World Insight panel and understand the full shape of their world without opening any other view.
2. Every entity, property, section, location, and rule mentioned in the panel is clickable and navigates to source within 200ms.
3. Selecting an entity in the Code Editor or any other view filters the World Insight panel to show that entity's complete picture.
4. The panel renders a 50-entity, 20-location world in under 100ms from projection data.
5. Collapsed/expanded state persists across sessions and workspace switches.
