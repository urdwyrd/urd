# URD — Playground Contextual Hints

*Rich authoring hints that surface the compiler's knowledge at the point of writing — default value previews, section content peeks, property override completion, exit maps, and presence validation.*

February 2026 | Done

> **Document status: BRIEF** — Nine features extending the playground's existing completion and hover extensions with deeper contextual intelligence. Every feature uses data already returned by the WASM compiler. No new Rust/WASM exports. No new npm dependencies.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-25
**Status:** Done

### What was done

All nine features implemented across three tiers, modifying only the three files specified by the brief:

**Tier 1 — Static documentation tooltips:**
- F1: Keyword and marker tooltips (`+`, `*`, `?`, `>`, `->`, `-> END`, `-> RETURN`, `---`) — static text from a lookup table.
- F2: Frontmatter key tooltips (`world:`, `types:`, `entities:`, `import:`) — shown only inside frontmatter delimiters.
- F3: Property type syntax hints (`int`, `number`, `string`, `bool`, `enum`, `ref`, `list`, `immutable`) — syntax, description, and examples shown when hovering type constructors inside frontmatter type blocks.

**Tier 2 — Compiler-data lookups:**
- F4: Enum value completion after `==`, `!=`, `=` operators when the property is an enum type.
- F5: Property override completion inside `{}` — property name completion (excluding already-present properties), plus value completion for enum (value list), bool (`true`/`false`), and ref (entity IDs of target type) properties.
- F6: Presence marker validation — hovering `[@entity]` appends containment status (`✓` or `⚠`) by resolving the enclosing location and checking its `contains` list.

**Tier 3 — Richer analysis:**
- F7: Default value hints on condition/effect lines — shows default or entity-override value, evaluates the expression at game start, warns on range overflow.
- F8: Section content preview on jump hover — reads the first 5 non-blank lines from the CodeMirror document after the section label, truncates with `…`.
- F9: Exit direction hover — shows all exits from the enclosing location with conditional annotations; exit destination hover shows the destination's exits and contained entities.

**Additional fix:** Corrected a pre-existing bug in `codemirror-urd.ts` where `\b` word boundaries in `stream.match()` could match mid-word (e.g., "in" inside "captain") because CodeMirror applies regexes to the remaining line text as an isolated string. Added previous-character guards to all four keyword/number match sites.

### What changed from the brief

1. **`ReferenceContext` passed via optional parameter.** The brief described frontmatter detection as a helper in `cursor-resolver.ts`. Implementation adds a `ReferenceContext` interface and `getFrontmatterContext()` function, with context passed as an optional third argument to `identifyReference()`. Existing callers (goto-definition) are unaffected — context is only provided by the hover tooltip handler.

2. **Exit detection uses line pattern matching only.** The brief specified resolving the enclosing location in the cursor resolver. The implementation detects the `direction: Destination Name` line pattern in the cursor resolver but defers location resolution to the hover handler, which scans backwards for the nearest `# Heading` and resolves via DefinitionIndex. This keeps the cursor resolver stateless.

3. **`buildTooltipContent` gains `view` and `lineNumber` parameters.** The brief didn't specify internal API changes, but passing the CodeMirror view to the tooltip builder was necessary for section preview (F8, reading document lines) and exit direction hover (F9, finding enclosing location). Parameters are optional to avoid breaking the function signature.

4. **Mid-word keyword highlighting fix.** Not in the brief scope. Fixed as a discovered bug during testing — `stream.match(/\b.../)`  in the StreamLanguage tokenizer doesn't respect word boundaries when the regex is applied to a substring of the line.

5. **Estimated ~400 lines; actual ~480 lines.** The exit direction tooltip and section preview required slightly more code than estimated, plus the keyword highlighting fix added ~8 lines to `codemirror-urd.ts`.

---

## Context

The Playground IDE Features brief delivered four CodeMirror extensions: inline diagnostics, autocomplete (`@entity`, `@entity.property`, `-> section`), hover tooltips (entity cards, property metadata, section stats, location details), and go-to-definition. Those features treat the compiler's output as a lookup table — "what is this identifier?" and "where is it defined?"

This brief goes further: it uses the compiler's output to answer *authoring* questions. "What value does this property have right now?" "What happens in the section I'm jumping to?" "What properties can I override on this entity?" These are the questions an author has in their head while writing, and the compiler already knows the answers.

### Current state (post IDE Features)

- `playground-state.ts` holds shared compile state with stale retention (`parsedWorld`, `definitionIndex`, `result.facts`, `result.property_index`)
- `completion-source.ts` handles three trigger contexts: `@` → entities, `@entity.` → properties, `->` → sections
- `hover-tooltip.ts` returns rich tooltips for entities, properties, sections, and locations
- `cursor-resolver.ts` identifies references under the cursor (entity, entity.property, type.property, section jump, section label, location heading)
- The WASM compiler returns `CompileResult` with `world` (JSON), `definition_index`, `facts` (FactSet), and `property_index` (PropertyDependencyIndex)

### What this brief delivers

Nine contextual hint features across three tiers:

**Tier 1 — Static text (no compiler data needed):**
1. Keyword and marker tooltips (`+`, `*`, `?`, `>`, `->`, `-> END`, `-> RETURN`)
2. Frontmatter key tooltips (`world:`, `types:`, `entities:`, `import:`)
3. Property type syntax hints (`int(min, max)`, `enum(...)`, `ref(TypeName)`)

**Tier 2 — Compiler data, simple lookups:**
4. Enum value completion in conditions and effects
5. Property override completion inside `{}`
6. Presence marker validation hints (`[@entity]`)

**Tier 3 — Compiler data, richer analysis:**
7. Default value hints on conditions and effects
8. Section content preview on jump hover
9. Exit direction hover with location mini-map


## Dependencies

- **Playground IDE Features complete.** `completion-source.ts`, `hover-tooltip.ts`, `cursor-resolver.ts`, and `playground-state.ts` all operational.
- No new npm dependencies.
- No new Rust/WASM exports.


## Tier 1: Static Documentation Tooltips

These features surface language documentation at the point of use. No compiler data needed — all content is hardcoded text.

### Feature 1: Keyword and Marker Tooltips

**What it does:** Hovering structural markers shows what they mean.

| Token | Tooltip |
|-------|---------|
| `+` (at line start) | Sticky choice — remains available after selection |
| `*` (at line start) | One-shot choice — removed after selection |
| `?` (at line start) | Condition — following content executes only if this is true |
| `>` (at line start) | Effect — modifies entity property values |
| `->` | Jump — transfers control to the named section |
| `-> END` | Built-in: terminates the current conversation |
| `-> RETURN` | Built-in: returns to the calling section |
| `---` | Frontmatter delimiter — YAML-like metadata block |

**Implementation:** Extend `identifyReference()` in `cursor-resolver.ts` with a new reference kind: `{ kind: 'keyword', token: string }`. Detection is line-start pattern matching — the same patterns already in `codemirror-urd.ts`'s StreamLanguage mode. The hover tooltip returns static text for each token. No DefinitionIndex or FactSet lookup.

**Acceptance criteria:**

- [ ] **CH-1.1** — Hovering `+` or `*` at line start shows choice type explanation.
- [ ] **CH-1.2** — Hovering `-> END` or `-> RETURN` shows built-in jump explanation.
- [ ] **CH-1.3** — Hovering `?` or `>` at line start shows condition/effect explanation.
- [ ] **CH-1.4** — Hovering `---` shows frontmatter delimiter explanation.


### Feature 2: Frontmatter Key Tooltips

**What it does:** Hovering top-level frontmatter keys shows brief descriptions.

| Key | Tooltip |
|-----|---------|
| `world:` | World identifier — used as the root key in compiled output |
| `types:` | Type definitions — declare entity schemas with typed properties |
| `entities:` | Entity declarations — instantiate types with optional property overrides |
| `import:` | Import — include another .urd.md file in this compilation unit |

**Implementation:** Detect cursor inside frontmatter (between `---` delimiters) on a line matching `key:` at indent level 0. Return static tooltip text. Frontmatter position detection requires tracking whether the cursor is between the first and second `---` lines — add a helper to `cursor-resolver.ts`.

**Acceptance criteria:**

- [ ] **CH-2.1** — Hovering `world:`, `types:`, `entities:`, or `import:` in frontmatter shows a description.
- [ ] **CH-2.2** — Hovering the same keywords outside frontmatter (e.g., in dialogue text) does not show a tooltip.


### Feature 3: Property Type Syntax Hints

**What it does:** When hovering a type constructor in a frontmatter property definition, show syntax documentation.

```
trust: int(|)
         ↑ cursor here
```

Shows:

```
int(min, max)
Integer with optional range constraints.
Examples: int, int(0, 100), int(0)
```

| Constructor | Hint |
|-------------|------|
| `int` / `int(min, max)` | Integer with optional range constraints |
| `number` / `number(min, max)` | Floating-point with optional range constraints |
| `string` | Text value |
| `bool` | Boolean — true or false (default: false) |
| `enum(value1, value2, ...)` | Enumerated string — must be one of the listed values |
| `ref(TypeName)` | Reference to an entity of the named type |
| `list(element_type)` | Ordered list of elements |
| `immutable` | Modifier — property cannot be changed by effects |

**Implementation:** Extend `cursor-resolver.ts` with a new reference kind: `{ kind: 'type-constructor', name: string }`. Detect when the cursor is inside frontmatter, within a type block (indented under a type name), on a line matching `property_name: type_constructor`. Extract the constructor keyword. The hover tooltip returns static text with syntax and examples.

**Acceptance criteria:**

- [ ] **CH-3.1** — Hovering a type constructor keyword in a frontmatter property definition shows syntax documentation with examples.
- [ ] **CH-3.2** — Type keywords outside frontmatter (e.g., in dialogue) do not trigger the hint.


## Tier 2: Compiler-Data Lookups

These features use the parsed world JSON and DefinitionIndex for simple value lookups.

### Feature 4: Enum Value Completion in Conditions and Effects

**What it does:** When typing `? @warden.mood ==` and the property is an enum, the completion popup shows valid enum values (`friendly`, `hostile`, `neutral`). Same for effects: `> @warden.mood =` offers the enum list.

**Trigger detection:** After `==`, `!=`, or `=` operators, look backwards on the same line for a property reference. Resolve the property's type. If it's an enum, return its `values` array as completions.

**Data source:** `parsedWorld.types[typeName].properties[propName].values`

**Implementation:** Extend `completion-source.ts` with a fourth trigger context. The backwards scan reuses the existing entity/property resolution logic — extract `@entity.property` or `Type.property` from before the operator, resolve to a type definition, check if it's an enum.

**Acceptance criteria:**

- [ ] **CH-4.1** — After `==` or `!=` following an enum property reference, completion popup shows valid enum values.
- [ ] **CH-4.2** — After `=` in an effect line following an enum property reference, completion popup shows valid enum values.
- [ ] **CH-4.3** — Non-enum properties (int, string, bool) do not trigger value completion after operators.


### Feature 5: Property Override Completion Inside `{}`

**What it does:** Inside an entity declaration's override block:

```
entities:
  @arina: Barkeep { | }
                    ↑ cursor
```

Offers completions for all properties defined on the `Barkeep` type: `name`, `trust`, `mood`. Each completion shows the property's type and default value. After selecting a property and typing `:` or `=`, if the property is an enum, offers enum value completions. If it's a ref, offers entity IDs of the target type. If it's a boolean, offers `true` / `false`.

**Implementation:** Extend `completion-source.ts` with a fifth trigger context:

1. **Detect cursor inside `{}`:** Scan backwards from cursor for `{` on the same line or preceding lines within frontmatter. Scan forwards for `}`. If both found, cursor is inside an override block.
2. **Resolve entity type:** Scan backwards from `{` for the pattern `@id: TypeName`. Extract `TypeName`.
3. **Property name completion:** If cursor is at a key position (start of token, after `,`), return property names from the type definition. Exclude properties already present in the block.
4. **Value completion:** If cursor is after `property_name:` or `property_name =`, determine the property type:
   - Enum → value list
   - Boolean → `true`, `false`
   - Ref → entity IDs filtered by ref type
   - Integer/Number → no completion, but show range constraints as detail text
   - String → no completion

**Data source:** `parsedWorld.types[typeName].properties` for names and types. `parsedWorld.entities` filtered by type for ref completions.

**Acceptance criteria:**

- [ ] **CH-5.1** — Inside `{}` after a typed entity declaration, completion popup shows property names with types and defaults.
- [ ] **CH-5.2** — Properties already present in the override block are excluded from completions.
- [ ] **CH-5.3** — After `property_name:` inside `{}`, enum properties show value completions.
- [ ] **CH-5.4** — After `property_name:` inside `{}`, boolean properties show `true` / `false`.
- [ ] **CH-5.5** — After `property_name:` inside `{}`, ref properties show entity IDs of the target type.


### Feature 6: Presence Marker Validation Hints

**What it does:** Hovering `[@warden]` shows the entity card (from the existing IDE-3.1 hover) plus a containment confirmation:

```
@warden: Guard
✓ Placed in this location (village-square)
Properties: trust = 0, mood = friendly
```

If the entity is not in the enclosing location's `contains` list:

```
@warden: Guard
⚠ Not contained in this location (the-rusty-anchor)
  Contained in: village-square
Properties: trust = 0, mood = friendly
```

**Why it matters:** Presence markers declare which entities are available for dialogue. If the entity isn't in the location's `contains` list, the author may have placed it in the wrong location. This isn't necessarily an error (entities can move at runtime), but surfacing it catches unintentional mismatches.

**Implementation:** Extend `hover-tooltip.ts`. When the reference is an entity inside `[@...]` brackets:

1. Resolve the enclosing location: scan backwards from cursor for the nearest `# Heading`, resolve via DefinitionIndex to a location slug.
2. Check `parsedWorld.locations[slug].contains` for the entity ID.
3. Append containment status to the existing entity card tooltip.

The cursor resolver already identifies `@entity` inside `[@...]`. No change to `cursor-resolver.ts` — the hover handler checks whether the entity reference is wrapped in brackets by inspecting surrounding characters.

**Acceptance criteria:**

- [ ] **CH-6.1** — Hovering `[@entity]` shows the entity card with a containment status line.
- [ ] **CH-6.2** — If the entity is in the enclosing location's `contains` list, `✓` confirmation is shown.
- [ ] **CH-6.3** — If the entity is not in the enclosing location, `⚠` warning is shown with the entity's actual location.


## Tier 3: Richer Analysis

These features combine compiler data with source text or arithmetic to provide deeper insights.

### Feature 7: Default Value Hints on Conditions and Effects

**What it does:** Hovering a property reference on a condition line shows the property's default value and whether the condition passes at game start:

```
? @warden.trust >= 3
```

Tooltip:

```
@warden.trust = 0 (default)
Condition: 0 >= 3 → false at game start
```

On an effect line:

```
> @warden.trust + 5
```

Tooltip:

```
@warden.trust = 0 (default)
After effect: 0 + 5 → 5
```

If the result would exceed the property's range constraints:

```
After effect: 95 + 10 → 105
⚠ Exceeds max (100)
```

**Data source:**

- Default values: `parsedWorld.types[typeName].properties[propName].default` for type-level, or `parsedWorld.entities[entityId].properties[propName]` for entity-level overrides.
- Range constraints: `parsedWorld.types[typeName].properties[propName].min` / `.max`.

**Implementation:** Extend `hover-tooltip.ts`. When the cursor is on a condition (`?`) or effect (`>`) line and the reference is a property:

1. Parse the line to extract operator and value literal. The line format is regular: `? @entity.prop <op> <value>` or `> @entity.prop <op> <value>`.
2. Resolve default value — entity override if present, else type default.
3. For conditions: evaluate `default <operator> literal` → boolean. Display the result.
4. For effects: evaluate `default <operator> literal` → number. Display before/after.
5. Check against `min`/`max` if present.

**Scope limitation:** Static analysis against default values only. Does not simulate runtime state or track cumulative effects. Tooltips are labelled "at game start" or "from default" to make this clear.

**Acceptance criteria:**

- [ ] **CH-7.1** — Hovering a property on a condition line shows default value and condition result at game start.
- [ ] **CH-7.2** — Hovering a property on an effect line shows default value and post-effect result.
- [ ] **CH-7.3** — If the post-effect result exceeds `min`/`max` range, a `⚠` warning is shown.
- [ ] **CH-7.4** — Entity-level overrides take precedence over type defaults.


### Feature 8: Section Content Preview on Jump Hover

**What it does:** Hovering `-> greet` shows the existing section metadata (compiled ID, jump counts, choice count) **plus** a content preview — the first 3–5 non-blank lines of the section's body:

```
Section: gatehouse/greet
Incoming: 2 | Outgoing: 3 | Choices: 2

  @arina: What'll it be, stranger?
  + Ask about the harbour
  * Ask about the missing ship
  …
```

**Why it matters:** This is VS Code's definition peek as a tooltip. Authors frequently jump between sections while writing branching dialogue. Seeing the first few lines confirms "yes, this is the section I meant" without scrolling away.

**Data source:** The editor's own document text. The DefinitionIndex provides the section's span (the `== greet` line). Read forward from that position in the CodeMirror document.

**Implementation:** Extend `hover-tooltip.ts`. When the reference is a `section-jump` or `section-label`:

1. Look up the section's span from the DefinitionIndex.
2. Read the CodeMirror document from `span.start_line + 1` forward.
3. Collect up to 5 non-blank lines (skip the `==` label itself).
4. Render as a `<pre>` block below the existing section metadata.
5. Truncate with `…` if more content exists.

**Acceptance criteria:**

- [ ] **CH-8.1** — Hovering `-> section_name` shows a content preview of the first 3–5 lines.
- [ ] **CH-8.2** — Hovering `== section_name` shows the same preview.
- [ ] **CH-8.3** — Preview is truncated with `…` if the section has more than 5 non-blank lines.
- [ ] **CH-8.4** — If the section is not found (stale state), no preview is shown — existing metadata still appears.


### Feature 9: Exit Direction Hover with Location Mini-Map

**What it does:** Hovering a direction keyword in an exit declaration shows all exits from the current location:

```
Exits from The Rusty Anchor:
  north → village-square
  south → harbour (conditional)
  east  → walled-garden
```

Hovering the destination name shows that location's details:

```
Location: village-square
Exits: north → gatehouse, south → the-rusty-anchor
Contains: @arina, @merchant
```

**Data source:**

- Current location exits: `parsedWorld.locations[slug].exits`
- Destination details: `parsedWorld.locations[destinationSlug]`
- Conditional status: `result.facts.exits` — `is_conditional` field on each `ExitEdge`
- Contained entities: `parsedWorld.locations[slug].contains`

**Implementation:** Extend `cursor-resolver.ts` with two new reference kinds:

- `{ kind: 'exit-direction', locationSlug: string, direction: string }`
- `{ kind: 'exit-destination', destinationSlug: string }`

Detection: cursor is inside a location block (after `# Heading`, before next `# Heading`). Line matches pattern `direction: Destination Name`. Resolve enclosing location from nearest heading via DefinitionIndex.

Extend `hover-tooltip.ts` to render exit summaries for these two reference kinds.

**Acceptance criteria:**

- [ ] **CH-9.1** — Hovering an exit direction shows all exits from the enclosing location.
- [ ] **CH-9.2** — Conditional exits are annotated.
- [ ] **CH-9.3** — Hovering an exit destination shows the destination's exits and contained entities.


## Architecture

All nine features extend existing modules from the Playground IDE Features brief. No new files are created.

### Files modified

| File | Features | Change |
|------|----------|--------|
| `cursor-resolver.ts` | F1, F2, F3, F9 | New reference kinds: `keyword`, `frontmatter-key`, `type-constructor`, `exit-direction`, `exit-destination`. Frontmatter position detection helper. Enclosing location resolution helper. |
| `hover-tooltip.ts` | F1, F2, F3, F6, F7, F8, F9 | New tooltip content builders for static docs, containment validation, default value hints, section previews, exit maps. |
| `completion-source.ts` | F4, F5 | New trigger contexts: enum value completion after operators, property override completion inside `{}`. |


## Performance Budget

All features operate on data already in memory. None trigger a recompile.

| Feature | Operation | Expected |
|---------|-----------|----------|
| Keyword tooltips (F1) | Static text lookup | < 1ms |
| Frontmatter key tooltips (F2) | Static text lookup | < 1ms |
| Type syntax hints (F3) | Static text lookup | < 1ms |
| Enum value completion (F4) | JSON property lookup | < 1ms |
| Property override completion (F5) | JSON enumeration + line scan | < 5ms |
| Presence marker validation (F6) | JSON contains check | < 1ms |
| Default value hints (F7) | JSON lookup + arithmetic | < 1ms |
| Section content preview (F8) | Document line scan (5 lines) | < 1ms |
| Exit direction hover (F9) | JSON location lookup | < 1ms |

All well under the 100ms hover/completion budget.


## Estimated Size

| Change | Lines |
|--------|-------|
| `cursor-resolver.ts` additions | ~80 |
| `hover-tooltip.ts` additions | ~200 |
| `completion-source.ts` additions | ~120 |
| **Total** | **~400 new lines** |


## New Dependencies

None.


## Sequencing

Features are independently shippable. Suggested order by effort and value:

```
Tier 1 (static text — quick wins, immediate learning value):
  F1: Keyword tooltips
  F2: Frontmatter key tooltips
  F3: Type syntax hints

Tier 2 (compiler lookups — authoring productivity):
  F4: Enum value completion
  F5: Property override completion
  F6: Presence marker validation

Tier 3 (richer analysis — deeper insight):
  F7: Default value hints
  F8: Section content preview
  F9: Exit direction hover
```

Within each tier, features are independent. Tier 1 can be done in an afternoon. Tier 2 is a day. Tier 3 is a day. No feature blocks another.


## Design Decisions

### Why static default analysis, not symbolic evaluation?

Feature 7 evaluates conditions and effects against default values only. It does not simulate cumulative effects along execution paths or track what happens when multiple branches modify a property. Full symbolic evaluation would require a constraint solver operating over the FactSet's jump/choice graph — a different class of problem that belongs in the compiler, not the tooltip layer. The static approach is simple, correct for the "at game start" case, and clearly labelled.

### Why show section previews from the editor document, not compiled output?

The compiled JSON doesn't preserve source text — it contains structured data but not dialogue lines. The author's source text is in the CodeMirror document. Reading 5 lines is trivial and gives exactly what the author wants: a peek at their own writing. If document text and compiled state are out of sync (stale state), the preview may be slightly stale — consistent with the stale-state-retention pattern.

### Why not promote presence marker warnings to compiler diagnostics?

Feature 6's `⚠ Not contained in this location` is an observation, not an error. An entity not in a location's `contains` list might be intentional (it arrives via a rule at runtime, or the presence marker is for a cutscene). Tooltips convey "here's what the compiler sees" without the authority of a diagnostic squiggly. If this signal proves reliable through usage, it can be promoted to a URD600+ diagnostic in a future brief.

### Why not promote range overflow warnings to compiler diagnostics?

Same reasoning as presence markers. Feature 7's `⚠ Exceeds max (100)` is based on default-state arithmetic. At runtime, the property might have been modified by earlier effects to a value where the arithmetic is safe. Static analysis from defaults can produce false positives. Tooltip annotations are the right weight for uncertain signals.


## What This Brief Does Not Cover

- **Runtime state simulation.** Evaluating against simulated game state. Requires a runtime or constraint solver.
- **Path-sensitive analysis.** "Can this condition ever be true given all possible paths?" Requires graph traversal over the FactSet.
- **Choice consequence preview.** Showing effects and jumps for each choice option. Could use `ChoiceFact.effect_writes` and `jump_indices`. Deferred.
- **Rule hover.** Showing rule trigger, actor, and select clause. Requires extending cursor resolver to identify rule blocks. Deferred.
- **Import line hover.** Showing imported file metadata. Irrelevant in single-file playground.
- **Quick-fix actions.** "Did you mean `@guard`?" on misspelled references. Requires a suggestion engine in the compiler.


## Relationship to Other Briefs

| Brief | Relationship |
|-------|--------------|
| **Playground IDE Features** | Direct prerequisite. This brief extends `completion-source.ts`, `hover-tooltip.ts`, and `cursor-resolver.ts`. |
| **SF-5 (LSP Foundation)** | `cursor.rs` and `hover.rs` remain the reference implementations. New reference kinds added here (keyword, frontmatter-key, type-constructor, exit-direction, exit-destination) may warrant backporting to the LSP for VS Code parity. |
| **Playground Component** | The WASM bridge and CodeMirror setup are unchanged. |
| **SF-2 (PropertyDependencyIndex)** | Read/write counts continue to be used in property hover tooltips. Feature 7 adds default values alongside them. |

*End of Brief*
