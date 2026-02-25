# URD — Playground Full Syntax Coverage

*Close every tooltip gap in the playground editor — traits, hidden properties, effect commands, condition keywords, rules, sequences, world configuration, and all remaining dark syntax.*

February 2026 | Backlog

> **Document status: BRIEF** — Seventeen new reference kinds in the cursor resolver and matching tooltip content. Extends the existing `cursor-resolver.ts` and `hover-tooltip.ts`. No new files. No new WASM exports. No new npm dependencies.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** —
**Status:** Backlog

### What was done

*(To be filled on completion.)*

### What changed from the brief

*(To be filled on completion.)*

---

## Context

The Playground IDE Features brief delivered four CodeMirror extensions (diagnostics, autocomplete, hover, go-to-definition). The Playground Contextual Hints brief added nine tooltip features covering static documentation, enum completion, default value hints, section previews, exit maps, and presence validation.

After both briefs, a systematic audit of the Sunken Citadel stress-test example against the cursor resolver reveals **seventeen syntax constructs** that produce no tooltip when hovered. These range from core language fundamentals (traits, hidden properties, effect commands, condition keywords) to entire major features (rules, sequences) to configuration syntax (world sub-keys). An author encountering any of these constructs gets a dead hover — no feedback, no documentation, no metadata.

This brief closes every remaining gap. After implementation, hovering *any* meaningful token in a valid Urd document produces a tooltip.

### Current state (post Contextual Hints)

The cursor resolver handles 11 reference kinds: `entity`, `entity-property`, `type-property`, `section-jump`, `section-label`, `location-heading`, `keyword`, `frontmatter-key`, `type-constructor`, `exit-direction`, `exit-destination`.

The hover tooltip covers: entity cards with containment, property metadata with read/write counts, section stats with content preview, location details, exit mini-maps, structural markers (`+`, `*`, `?`, `>`, `->`, `-> END`, `-> RETURN`, `---`), frontmatter top-level keys, type constructor syntax, default value hints on conditions/effects, and presence marker validation.

### What this brief delivers

Seventeen new tooltip targets across four tiers:

**Tier 1 — Language fundamentals (every author hits these immediately):**
1. Trait markers (`[interactable, mobile, container]`)
2. Hidden visibility prefix (`~`)
3. Effect commands (`move`, `destroy`, `reveal`)
4. Condition keywords (`in`, `not in`, `here`, `player`)
5. OR condition combinator (`any:`)

**Tier 2 — Major features with zero coverage:**
6. Rule blocks (`rule name:`, `actor:`, `action`, `selects...from...where`)
7. Sequence headings (`##`)
8. Phase headings (`###`) with modifier detection (`(auto)`)

**Tier 3 — Configuration and special syntax:**
9. World sub-keys (`name:`, `version:`, `start:`, `entry:`, `seed:`, `description:`, `author:`)
10. Exit-jump syntax (`-> exit:direction`)
11. Case-insensitive `-> end` / `-> END` handling
12. Property default values in type definitions (`= 0`, `= "none"`)
13. Implicit `container` property (`@entity.container`)

**Tier 4 — Contextual enrichment (lower priority, polish):**
14. Dialogue attribution lines (`@entity: text`)
15. Narrative action lines (`@entity does something.`)
16. Effect value validation (is this string a valid enum value?)
17. Comments (`//`)


## Dependencies

- **Playground IDE Features complete.** `cursor-resolver.ts`, `hover-tooltip.ts`, `completion-source.ts`, `playground-state.ts` all operational.
- **Playground Contextual Hints complete.** Frontmatter position detection, enclosing location resolution, default value hints all working.
- No new npm dependencies.
- No new Rust/WASM exports.


## Tier 1: Language Fundamentals

These are constructs every author encounters in their first five minutes. Having them be dark is a significant onboarding gap.

### Feature 1: Trait Markers

**What's dark:**

```
Guard [interactable, mobile, container]:
       ^^^^^^^^^^^^  ^^^^^^  ^^^^^^^^^
       no tooltip on any of these
```

**What to show:**

Hovering a trait name inside `[...]` on a type definition line shows what the trait enables:

| Trait | Tooltip |
|-------|---------|
| `interactable` | Trait: entities of this type can be the target of dialogue and actions |
| `mobile` | Trait: entities of this type can move between locations via rules |
| `container` | Trait: entities of this type can hold other entities (items, etc.) |
| `portable` | Trait: entities of this type can be picked up, moved, and carried |

**Implementation:**

New reference kind: `{ kind: 'trait'; name: string }`.

Detection: cursor is inside frontmatter, on a line matching `TypeName [trait1, trait2, ...]:`. Extract the word under the cursor if it falls within the bracket span. Match against known trait names.

Tooltip: static text lookup from a `TRAIT_DOCS` map.

**Acceptance criteria:**

- [ ] **FS-1.1** — Hovering `interactable`, `mobile`, `container`, or `portable` inside `[...]` on a type definition line shows a trait description.
- [ ] **FS-1.2** — Hovering outside the bracket span does not trigger the trait tooltip.


### Feature 2: Hidden Visibility Prefix

**What's dark:**

```
~secret: string = "none"
^
no tooltip on the tilde
```

**What to show:**

```
~ Hidden property
This property is not visible to the player at runtime.
Use `reveal @entity.property` to make it visible.
```

**Implementation:**

New reference kind: `{ kind: 'visibility-prefix'; visibility: 'hidden' }`.

Detection: cursor is inside frontmatter, in a type block, on a property definition line. The line's trimmed content starts with `~`. Cursor is on the `~` character.

Tooltip: static text explaining hidden visibility and how to reveal it.

**Acceptance criteria:**

- [ ] **FS-2.1** — Hovering the `~` prefix on a property definition shows hidden visibility explanation.
- [ ] **FS-2.2** — The tooltip mentions the `reveal` command.


### Feature 3: Effect Commands

**What's dark:**

```
> move @ancient_coin -> player
  ^^^^
> destroy @scholar_voss
  ^^^^^^^
> reveal @magic_mirror.true_reflection
  ^^^^^^
```

**What to show:**

| Command | Tooltip |
|---------|---------|
| `move` | `move @entity -> target` — Transfers an entity to another entity or location. Target can be `player`, `here`, `@entity`, or a location name. |
| `destroy` | `destroy @entity` — Permanently removes the entity from the world. |
| `reveal` | `reveal @entity.property` — Makes a hidden (`~`) property visible to the player. |

**Implementation:**

New reference kind: `{ kind: 'effect-command'; command: string }`.

Detection: cursor is on an effect line (starts with `>`). After the `>` marker, the first word is `move`, `destroy`, or `reveal`. Cursor falls within that word's span.

Tooltip: static text from an `EFFECT_COMMAND_DOCS` map, including syntax examples.

**Acceptance criteria:**

- [ ] **FS-3.1** — Hovering `move`, `destroy`, or `reveal` on an effect line shows command documentation with syntax.
- [ ] **FS-3.2** — The `move` tooltip shows all valid target forms (`player`, `here`, `@entity`).


### Feature 4: Condition Keywords

**What's dark:**

```
? @iron_key in player
             ^^  ^^^^^^
? @cipher_note not in player
               ^^^^^^  ^^^^^^
? @ancient_coin in here
                   ^^^^
```

**What to show:**

| Keyword | Tooltip |
|---------|---------|
| `in` | Containment test — checks if the entity is held by or located in the target |
| `not in` | Negated containment test — checks the entity is NOT in the target |
| `player` | The player entity — the implicit protagonist carrying items |
| `here` | The current location — where the player currently is |

**Implementation:**

New reference kind: `{ kind: 'condition-keyword'; keyword: string }`.

Detection: cursor is on a condition line (starts with `?`). Scan for `in`, `not in`, `player`, `here` tokens. Check if cursor falls within the token span. The `not in` keyword is two words — detect as a unit when `not` is followed by whitespace and `in`.

Note: `player` and `here` also appear in effect lines (`> move @entity -> player`, `> move @entity -> here`). The tooltip should trigger in both contexts — the meaning is the same.

**Acceptance criteria:**

- [ ] **FS-4.1** — Hovering `in` on a condition line shows containment test explanation.
- [ ] **FS-4.2** — Hovering `not in` shows negated containment test.
- [ ] **FS-4.3** — Hovering `player` on a condition or effect line shows player entity explanation.
- [ ] **FS-4.4** — Hovering `here` on a condition or effect line shows current location explanation.


### Feature 5: OR Condition Combinator

**What's dark:**

```
? any:
  ^^^^
    @captain_rhys.alertness == alert
    @captain_rhys.alertness == panicked
```

**What to show:**

```
any:
OR combinator — the following conditions are evaluated as alternatives.
At least one must be true for the block to execute.
```

**Implementation:**

New reference kind: `{ kind: 'condition-combinator'; combinator: string }`.

Detection: cursor is on a condition line. The trimmed text after `?` starts with `any:`. Cursor falls on `any:` or `any`.

Tooltip: static text explaining OR semantics.

**Acceptance criteria:**

- [ ] **FS-5.1** — Hovering `any:` on a condition line shows OR combinator explanation.


## Tier 2: Major Features

Rules and sequences are entire language features with complex syntax. Both are completely dark.

### Feature 6: Rule Block Syntax

**What's dark:**

```
rule spirit_manifests:                    ← rule keyword + name
  actor: @spirit_lament action manifest   ← actor, action keyword
  selects target from [...]               ← selects, from keywords
    where target.form == translucent      ← where keyword
```

**What to show:**

| Token | Context | Tooltip |
|-------|---------|---------|
| `rule` | Line start | Rule declaration — defines an NPC behavioral rule triggered by the runtime |
| Rule name | After `rule` | Show rule metadata: actor, action, select count, effect count (from DefinitionIndex + FactSet) |
| `actor:` | Inside rule | The entity that initiates this rule's action |
| `action` | Inside rule | The action verb that triggers this rule |
| `selects` | Inside rule | Select clause — iterates over candidate entities matching the `from` list |
| `target` | Inside rule | The loop variable — represents each candidate entity being evaluated |
| `from` | Inside rule | The entity pool to select from |
| `where` | Inside rule | Filter condition — candidates must satisfy all `where` clauses |

**Implementation:**

New reference kinds:

- `{ kind: 'rule-keyword'; keyword: string }` — for `rule`, `actor`, `action`, `selects`, `from`, `where`, `target`
- `{ kind: 'rule-name'; name: string }` — for the rule identifier

Detection: track whether the cursor is inside a rule block. A rule block starts with `rule name:` at indent 0 and continues until the next unindented line that isn't a continuation. Within the block, match keywords by position.

The rule name tooltip uses the DefinitionIndex (`rule:name` key) to confirm the rule exists, and the FactSet to show the rule's condition reads and effect writes count.

**Acceptance criteria:**

- [ ] **FS-6.1** — Hovering `rule` at line start shows rule declaration explanation.
- [ ] **FS-6.2** — Hovering a rule name shows rule metadata (actor, effect count) from DefinitionIndex/FactSet.
- [ ] **FS-6.3** — Hovering `actor:`, `action`, `selects`, `from`, `where`, `target` inside a rule block shows keyword documentation.


### Feature 7: Sequence Headings

**What's dark:**

```
## Main Quest
```

Currently only `#` (location) headings are resolved. `##` headings define sequences — a different construct entirely.

**What to show:**

```
Sequence: main-quest
Phases: 5 (3 manual, 2 auto)
```

Source: DefinitionIndex (`sequence:main-quest` if present — note: sequences may not yet be in the DefinitionIndex, in which case static text is sufficient).

**Implementation:**

New reference kind: `{ kind: 'sequence-heading'; name: string }`.

Detection: line starts with `## ` (exactly two `#`). Extract the heading text.

Tooltip: if found in DefinitionIndex, show metadata. If not, show static text: "Sequence — defines a multi-phase quest or progression arc."

**Acceptance criteria:**

- [ ] **FS-7.1** — Hovering a `##` heading shows it's a sequence with available metadata.
- [ ] **FS-7.2** — `##` headings are not confused with `#` location headings.


### Feature 8: Phase Headings with Modifier Detection

**What's dark:**

```
### Investigation
### The Descent (auto)
                ^^^^^
```

The `(auto)` modifier is particularly important — it means the phase auto-advances without player action, which is a critical semantic difference.

**What to show:**

```
Phase: investigation
Advance: manual — requires player to complete an action
```

```
Phase: the-descent
Advance: auto — progresses automatically when conditions are met
```

**Implementation:**

New reference kind: `{ kind: 'phase-heading'; name: string; auto: boolean }`.

Detection: line starts with `### `. Extract heading text. Check for trailing `(auto)` — if present, set `auto: true` and strip from the display name.

Tooltip: show phase name and advance mode. If the phase has conditions (lines starting with `?` immediately after), note "Has entry conditions."

**Acceptance criteria:**

- [ ] **FS-8.1** — Hovering a `###` heading shows phase information.
- [ ] **FS-8.2** — The `(auto)` modifier is detected and the tooltip explains auto-advance.
- [ ] **FS-8.3** — Phases without `(auto)` show manual advance.


## Tier 3: Configuration and Special Syntax

### Feature 9: World Sub-Keys

**What's dark:**

```
world:
  name: the-sunken-citadel       ← no tooltip
  version: "1.0"                 ← no tooltip
  start: village-square          ← no tooltip (references a location!)
  entry: main-quest              ← no tooltip (references a sequence!)
  seed: 7919                     ← no tooltip
  description: "..."             ← no tooltip
  author: "..."                  ← no tooltip
```

**What to show:**

| Key | Tooltip |
|-----|---------|
| `name` | World identifier — slug used as the root key in compiled JSON output |
| `version` | Schema version string — included in compiled output for compatibility checking |
| `start` | Starting location — where the player begins. References a location heading. |
| `entry` | Entry sequence — the initial quest/progression arc. References a `##` sequence heading. |
| `seed` | Random seed — used by the runtime for deterministic randomisation |
| `description` | Human-readable description of the world |
| `author` | Author attribution |

For `start` and `entry`, the tooltip should additionally show the resolved target — e.g., "Resolves to: **village-square** (Village Square)" with the location's display name, or "Resolves to: **main-quest** (Main Quest)" with the sequence heading. This reuses the DefinitionIndex for the lookup.

**Implementation:**

New reference kind: `{ kind: 'world-sub-key'; key: string; value?: string }`.

Detection: cursor is inside frontmatter, indented under `world:`. Line matches `key: value` at indent 2+. Extract both key and value.

Tooltip: static description from a `WORLD_KEY_DOCS` map, plus value resolution for `start` and `entry` via DefinitionIndex lookup.

**Acceptance criteria:**

- [ ] **FS-9.1** — Hovering world sub-keys shows documentation.
- [ ] **FS-9.2** — Hovering `start:` additionally shows the resolved location name.
- [ ] **FS-9.3** — Hovering `entry:` additionally shows the resolved sequence name.
- [ ] **FS-9.4** — Sub-keys are only matched inside the `world:` frontmatter block, not elsewhere.


### Feature 10: Exit-Jump Syntax

**What's dark:**

```
-> exit:upstairs
   ^^^^^^^^^^^^
```

This is a special jump form that navigates to a location via its exit direction, rather than jumping to a section. The current section-jump resolver strips the `exit:` prefix and treats it as a section name, which is wrong.

**What to show:**

```
Exit jump — navigates via the exit named "upstairs" from the current location.
Resolves to: Scholar's Room
```

**Implementation:**

Modify the existing section-jump detection in `findSectionJump()`. When the target starts with `exit:`, return a new reference kind: `{ kind: 'exit-jump'; direction: string }`.

Tooltip: explain the exit-jump semantics. Resolve the direction against the enclosing location's exits (reusing `findEnclosingLocation()` from the contextual hints code) to show the destination.

**Acceptance criteria:**

- [ ] **FS-10.1** — Hovering `-> exit:direction` shows exit-jump explanation and resolved destination.
- [ ] **FS-10.2** — The destination is resolved from the enclosing location's exit list.


### Feature 11: Case-Insensitive End/Return Handling

**What's dark:** The resolver checks `-> END` and `-> RETURN` exactly. The Sunken Citadel uses `-> end` (lowercase).

**Fix:** Make the `END`/`RETURN` check case-insensitive in the cursor resolver.

**Implementation:** Change the comparison in `identifyReference()`:

```typescript
if (afterArrow.toUpperCase() === 'END' || afterArrow.toUpperCase() === 'RETURN') {
  return { kind: 'keyword', token: `-> ${afterArrow.toUpperCase()}` };
}
```

**Acceptance criteria:**

- [ ] **FS-11.1** — `-> end`, `-> End`, `-> END` all produce the built-in jump tooltip.
- [ ] **FS-11.2** — `-> return`, `-> Return`, `-> RETURN` all produce the built-in jump tooltip.


### Feature 12: Property Default Values in Type Definitions

**What's dark:**

```
trust: int(0, 100) = 0
                     ^^  the default value
       ^^^^^^^^^^^       the range arguments
```

The type constructor (`int`) gets a tooltip but the `= 0` default and the `(0, 100)` range are invisible.

**What to show:** Extend the existing `type-constructor` tooltip to include the parsed range and default:

```
int(min, max) = default
Integer with optional range constraints.
Range: 0 to 100
Default: 0
```

**Implementation:** Modify `findTypeConstructor()` to also extract the range arguments and default value from the line. Pass them into the tooltip builder. The existing `TYPE_CONSTRUCTOR_DOCS` becomes a template that includes the actual values when available.

This doesn't need a new reference kind — it enriches the existing `type-constructor` tooltip with contextual data from the same line.

**Acceptance criteria:**

- [ ] **FS-12.1** — Type constructor tooltips show the actual range values when `(min, max)` is present.
- [ ] **FS-12.2** — Type constructor tooltips show the actual default value when `= value` is present.
- [ ] **FS-12.3** — Enum constructor tooltips show the actual declared values.


### Feature 13: Implicit Container Property

**What's dark:**

```
? @enchanted_blade.container != player
                   ^^^^^^^^^
```

`container` is an implicit property — it's not declared in any type definition. It exists on every entity and holds the entity or location that currently contains it.

**What to show:**

```
container (implicit)
The entity or location that currently holds this entity.
Value is an entity ID, location slug, or "player".
Read-only — changed via `move` effects, not direct assignment.
```

**Implementation:** In the existing `propertyTooltip()` / `typePropertyTooltip()` function, when the DefinitionIndex lookup fails for a property named `container`, return the implicit property tooltip instead of `null`.

**Acceptance criteria:**

- [ ] **FS-13.1** — Hovering `.container` on any entity shows the implicit property explanation.
- [ ] **FS-13.2** — The tooltip explains that `container` is modified via `move`, not direct assignment.


## Tier 4: Contextual Enrichment

Lower priority polish. Each is small and independent.

### Feature 14: Dialogue Attribution Lines

**What's dark (contextually):**

```
@elder_maren: Another stranger. We've had enough trouble.
```

The `@elder_maren` part already triggers an entity tooltip. But the colon marks this as a dialogue attribution — a specific construct meaning "this character speaks the following text."

**What to show:** Append a context annotation to the entity tooltip when the entity reference is followed by `:` (dialogue) or is followed by a verb (narrative action):

```
@elder_maren: Guard
Container: village-square
Speaking — dialogue attribution
```

**Implementation:** In `entityTooltip()`, after building the card, check if the character immediately after the entity reference span is `:`. If so, append "Speaking — dialogue attribution" as a dim annotation.

**Acceptance criteria:**

- [ ] **FS-14.1** — Hovering `@entity:` (with colon) shows the entity card plus a "Speaking" annotation.


### Feature 15: Narrative Action Lines

**What's dark (contextually):**

```
@elder_maren sighs heavily.
```

The entity reference triggers a tooltip, but there's no indication this is a narrative action line.

**What to show:** When the entity reference is followed by a non-colon, non-dot, non-`]` character (i.e., it's not dialogue, not property access, not a presence marker), annotate:

```
@elder_maren: Guard
Narrative action — stage direction or character action
```

**Implementation:** Same pattern as Feature 14 — check the character after the entity span. If it's a space followed by a word (not `:`, `.`, `]`), append the narrative action annotation.

**Acceptance criteria:**

- [ ] **FS-15.1** — Hovering `@entity` followed by prose shows a "Narrative action" annotation.


### Feature 16: Effect Value Validation

**What's dark:**

```
> @elder_maren.mood = terrified
                      ^^^^^^^^^
```

The string literal `terrified` is a valid enum value for `mood`, but the author gets no confirmation. If they typed `scared` (not in the enum), they'd get no tooltip either — they'd only see the error after the compiler runs.

**What to show:** When hovering a string literal on an effect or condition line, if the property is an enum, show whether the value is valid:

```
"terrified" — valid value for Villager.mood
Valid values: hostile, suspicious, neutral, friendly, terrified
```

Or if invalid:

```
"scared" — ⚠ not a valid value for Villager.mood
Valid values: hostile, suspicious, neutral, friendly, terrified
```

**Implementation:**

New reference kind: `{ kind: 'value-literal'; value: string; entityId?: string; property?: string }`.

Detection: on condition or effect lines, after the operator (`==`, `!=`, `=`), detect the literal value. If the left-hand side is a property reference, resolve the property type. If it's an enum, return the value-literal reference.

Tooltip: check the value against the property's `values` array. Show confirmation or warning.

**Acceptance criteria:**

- [ ] **FS-16.1** — Hovering a string literal after an operator on a condition/effect line, when the property is an enum, shows whether the value is valid.
- [ ] **FS-16.2** — Invalid enum values show a `⚠` warning with the valid values list.


### Feature 17: Comments

**What's dark:**

```
// hub prompt
```

**What to show:**

```
Comment — ignored by the compiler.
Use // to annotate your schema for other authors.
```

Very low priority — most authors know what comments are. But it rounds out the "every token has a tooltip" goal.

**Implementation:**

New reference kind: `{ kind: 'comment' }`.

Detection: trimmed line starts with `//` and cursor is on or after the `//`. Alternatively, `//` appears mid-line and cursor is after it.

Tooltip: static text.

**Acceptance criteria:**

- [ ] **FS-17.1** — Hovering `//` shows comment explanation.


## Architecture

All features extend `cursor-resolver.ts` and `hover-tooltip.ts`. No new files.

### New reference kinds in cursor-resolver.ts

| Kind | Tier | Detection context |
|------|------|-------------------|
| `trait` | 1 | Frontmatter, inside `[...]` on type definition line |
| `visibility-prefix` | 1 | Frontmatter, type block, `~` at property start |
| `effect-command` | 1 | Effect line (`>`), first word is `move`/`destroy`/`reveal` |
| `condition-keyword` | 1 | Condition/effect line, `in`/`not in`/`player`/`here` |
| `condition-combinator` | 1 | Condition line, `any:` |
| `rule-keyword` | 2 | Inside rule block, keyword tokens |
| `rule-name` | 2 | `rule name:` line, the name |
| `sequence-heading` | 2 | Line starts with `## ` |
| `phase-heading` | 2 | Line starts with `### `, optional `(auto)` |
| `world-sub-key` | 3 | Frontmatter, indented under `world:` |
| `exit-jump` | 3 | `-> exit:direction` syntax |
| `value-literal` | 4 | Literal value after operator on condition/effect line |
| `comment` | 4 | `//` token |

Features 11, 12, 13, 14, 15 modify existing reference kinds or tooltip builders rather than adding new ones.

### Context tracking additions

The cursor resolver's `ReferenceContext` needs three new fields:

```typescript
export interface ReferenceContext {
  inFrontmatter: boolean;
  inTypeBlock?: boolean;
  inWorldBlock?: boolean;    // NEW: cursor is indented under `world:`
  inRuleBlock?: boolean;     // NEW: cursor is inside a `rule name:` block
  ruleIndent?: number;       // NEW: base indent of the rule block
}
```

`getFrontmatterContext()` already scans backwards for `---` delimiters. Extend it to also track `world:` blocks. For rule blocks, a separate scan or a stateful approach is needed since rules live outside frontmatter.

For rule block detection: scan backwards from the current line for a `rule name:` line at lower or equal indent. If found and no blank line or dedented non-continuation line intervenes, the cursor is inside that rule.


## Estimated Size

| Change | Lines |
|--------|-------|
| `cursor-resolver.ts` — new reference kinds + detection | ~200 |
| `cursor-resolver.ts` — context tracking extensions | ~40 |
| `hover-tooltip.ts` — new tooltip builders | ~250 |
| `hover-tooltip.ts` — enrichments to existing builders | ~50 |
| **Total** | **~540 new lines** |


## Performance Budget

All features are static text lookups, single-pass line scans, or DefinitionIndex lookups. Nothing triggers recompilation.

| Feature category | Expected latency |
|------------------|-----------------|
| Static keyword/documentation tooltips (T1, T2) | < 1ms |
| DefinitionIndex lookups (rules, sequences, world refs) | < 1ms |
| Line scanning for context (rule block, world block) | < 5ms |
| Enum value validation | < 1ms |

All well under the 100ms hover budget.


## New Dependencies

None.


## Sequencing

Tiers are independently shippable. Within each tier, features are independent.

```
Tier 1 (language fundamentals — highest author impact):
  F1:  Trait markers
  F2:  Hidden visibility prefix
  F3:  Effect commands
  F4:  Condition keywords
  F5:  OR combinator

Tier 2 (major feature coverage):
  F6:  Rule blocks
  F7:  Sequence headings
  F8:  Phase headings

Tier 3 (configuration and special syntax):
  F9:  World sub-keys
  F10: Exit-jump syntax
  F11: Case-insensitive end/return
  F12: Property default values in type defs
  F13: Implicit container property

Tier 4 (contextual enrichment — polish):
  F14: Dialogue attribution
  F15: Narrative action lines
  F16: Effect value validation
  F17: Comments
```

Feature 11 (case-insensitive fix) is a one-line change and should be done immediately regardless of tier ordering.

Feature 13 (implicit container) is a small change to an existing function and can be done at any time.


## Design Decisions

### Why static documentation for rule/sequence keywords instead of structural analysis?

Rule and sequence syntax is complex — `selects...from...where` involves iteration, filtering, and variable binding. A full structural analysis (showing which entities match the `from` pool, which `where` clauses filter them, etc.) would require evaluating the select clause against the world state. This is valuable but complex. The brief opts for static keyword documentation with light metadata (rule name lookup in DefinitionIndex, effect/condition counts from FactSet). Structural analysis of rules is deferred.

### Why annotate dialogue attribution and narrative actions on the entity tooltip instead of a separate reference kind?

These aren't new constructs — they're contextual uses of entity references. The entity card is already the correct tooltip. Adding a small annotation ("Speaking" or "Narrative action") gives context without changing the primary information. A separate reference kind would duplicate the entity tooltip logic.

### Why track rule blocks via backward scanning instead of a full document parse?

The cursor resolver is line-oriented by design (port of `cursor.rs`). Adding a full document parse would change the architectural assumption. Backward scanning for `rule name:` with indent tracking is simple, consistent with existing patterns (frontmatter scanning, enclosing location scanning), and correct for the single-file playground case.

### Why not add go-to-definition for world.start and world.entry?

`world.start` references a location and `world.entry` references a sequence. Both could support Ctrl+click → jump to definition. This is a go-to-definition enhancement, not a tooltip enhancement — it belongs in `goto-definition.ts`. The tooltip in this brief shows the resolved target name, which is the prerequisite data. Adding the actual navigation is a small follow-on to the existing go-to-definition extension and can be done alongside or after this brief.


## Acceptance Criteria — Cross-Cutting

- [ ] **FS-CC.1** — After implementation, hovering any non-whitespace, non-prose token in the Sunken Citadel example produces a tooltip.
- [ ] **FS-CC.2** — All tooltips work in both Gloaming (dark) and Parchment (light) themes.
- [ ] **FS-CC.3** — No new reference kind breaks existing tooltip coverage — all previously working tooltips continue to function.
- [ ] **FS-CC.4** — No new WASM exports required.
- [ ] **FS-CC.5** — Bundle size increase under 5 KB gzipped (all changes are in existing files).


## What This Brief Does Not Cover

- **Rule structural analysis.** Showing which entities match `selects...from`, evaluating `where` clauses. Future work.
- **Go-to-definition for world.start / world.entry.** Navigation from the frontmatter to the target location/sequence. Enhancement to `goto-definition.ts`.
- **Sequence/phase metadata from DefinitionIndex.** Sequences and phases may not yet be fully represented in the DefinitionIndex. If not, the tooltips fall back to static text. Extending the DefinitionIndex is a compiler-side change.
- **Autocomplete for traits, effect commands, condition keywords.** Completion popups for these tokens. Separate enhancement to `completion-source.ts`.
- **Runtime state in tooltips.** Showing current property values during a play session. Requires runtime integration.


## Relationship to Other Briefs

| Brief | Relationship |
|-------|--------------|
| **Playground IDE Features** | Foundation. This brief extends `cursor-resolver.ts` and `hover-tooltip.ts` from that brief. |
| **Playground Contextual Hints** | Prerequisite. Frontmatter detection, enclosing location resolution, and default value hints from that brief are reused here. |
| **SF-5 (LSP Foundation)** | The new reference kinds (traits, rule keywords, effect commands, etc.) should eventually be backported to `cursor.rs` in the LSP for VS Code parity. |
| **Compiler Architecture** | Rule and sequence syntax definitions in the spec determine what tooltips should say. |

*End of Brief*
