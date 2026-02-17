# Urd Compiler — EMIT Phase Brief

> Phase 5 of 5. Transforms validated ASTs and the symbol table into a single `.urd.json` string that conforms to the Urd World Schema.

**Status:** Implementation-ready.

## Phase Position

```
.urd.md → PARSE → IMPORT → LINK → VALIDATE → [EMIT] → .urd.json
```

EMIT is the final phase. It runs only when the diagnostic collector contains zero Error-severity diagnostics from all preceding phases. If any error exists, EMIT does not run and the compiler returns `null` for the world output (see architecture brief, Compiler Interface section).

## Input

- **Annotated FileASTs** — produced by PARSE, annotated by LINK and VALIDATE. All reference annotations are populated (or `null` for unresolved references, which already have LINK diagnostics). All spans are intact.
- **SymbolTable** — populated by LINK. Contains all TypeSymbols, EntitySymbols, LocationSymbols, SectionSymbols, ChoiceSymbols, ExitSymbols, ActionSymbols, RuleSymbols, and SequenceSymbols in insertion order (topological file order, then declaration order within file).
- **DependencyGraph** — produced by IMPORT. Provides topological file order and file stem information.

## Output

- **A single JSON string** — the `.urd.json` file content. Conforms to the Urd World Schema (`urd-world-schema.json`). Byte-identical across repeated compilations of the same source for the same compiler version and configuration.
- **Diagnostics** — any EMIT-phase warnings (URD500–URD599 range).

## What EMIT Does

EMIT traverses the annotated ASTs and symbol table to build a JSON object, then serialises it to a string.

- Assembles the `world` block from frontmatter. Injects `urd: "1"`.
- Assembles the `types` block from TypeSymbols.
- Assembles the `entities` block from EntitySymbols. Omits implicit player (no `@player` declared).
- Assembles the `locations` block from LocationSymbols, including `contains` lists, `description` text, and `exits` maps.
- Assembles the `rules` block from RuleSymbols.
- Assembles the `actions` block from ActionSymbols (including choice-derived actions).
- Assembles the `sequences` block from SequenceSymbols and their PhaseSymbols.
- Assembles the `dialogue` block from SectionSymbols and their ChoiceSymbols.
- Lowers AST condition nodes to condition expression strings.
- Lowers AST effect nodes to structured effect objects.
- Expands `KeywordHere` annotations to `player.container` in condition strings and effect destinations.
- Expands `KeywordPlayer` annotations to `player` in condition strings and effect destinations.
- Strips `@` sigils from entity references in all output strings and fields.
- Strips `Comment` nodes. Comments do not appear in JSON.
- Skips `ErrorNode` entries silently.
- Omits top-level blocks that have no entries (`types`, `entities`, `locations`, `rules`, `actions`, `sequences`, `dialogue` are all optional per the JSON Schema; only `world` is required).

## What EMIT Does Not Do

- **No validation.** EMIT assumes all upstream phases have run. It does not re-check types, ranges, references, or constraints.
- **No AST modification.** EMIT reads the AST and symbol table. It does not mutate them.
- **No reference resolution.** All references were resolved by LINK. EMIT reads resolved annotations and symbol fields directly.
- **No ID derivation.** All compiled IDs (section IDs, choice IDs, location IDs) were computed by LINK and stored on symbols. EMIT reads `compiled_id` fields. It does not call `slugify()`.

## Upstream Guarantees EMIT Relies On

| Guarantee | Source Phase |
|-----------|-------------|
| Every reference annotation is populated or `null` (with a LINK diagnostic). | LINK |
| Every symbol has a `compiled_id` that is globally unique within its namespace (including choice IDs within sections). | LINK |
| All type checks, range checks, enum membership, and trait requirements have passed. | VALIDATE |
| The symbol table uses `OrderedMap` with stable insertion order. | LINK |
| Topological file order is deterministic (depth-first, alphabetical tiebreaker). | IMPORT |
| No Error-severity diagnostics exist (otherwise EMIT does not run). | All |

## The Algorithm

EMIT builds a JSON object by walking the symbol table in insertion order and reading AST nodes through their annotations. The result is a tree of JSON-compatible data structures that is serialised to a string in the final step.

### Emit Rule

When this algorithm says "emit field X", the JSON key name is the literal string shown. When it says "emit block X", the result is a JSON object or array as defined by the Urd World Schema. The schema (`urd-world-schema.json`) is the canonical structural contract for the output.

### Top-Level Key Order

The output JSON object has keys in this fixed order:

1. `world` (always present)
2. `types` (omitted if empty)
3. `entities` (omitted if empty)
4. `locations` (omitted if empty)
5. `rules` (omitted if empty)
6. `actions` (omitted if empty)
7. `sequences` (omitted if empty)
8. `dialogue` (omitted if empty)

This order is fixed regardless of declaration order in source files. It matches the JSON Schema's `properties` declaration order for readability.

### Step 1: Build the `world` Block

Read the `WorldBlock` from the compilation entry file's `Frontmatter` AST node. The entry file is the compilation root and is last in the ordered `FileAST` list produced by IMPORT.

| Source Field | JSON Key | Rule |
|-------------|----------|------|
| `name` | `name` | Copy directly. Required. |
| (none) | `urd` | Always inject `"1"`. Overrides any author value. |
| `version` | `version` | Copy if present. Omit if absent. |
| `description` | `description` | Copy if present. Omit if absent. |
| `author` | `author` | Copy if present. Omit if absent. |
| `start` | `start` | Copy the location ID string. Required by Schema Markdown and enforced by VALIDATE (URD404), but optional in the JSON Schema. |
| `entry` | `entry` | Copy the sequence ID string. Already validated by VALIDATE. Omit if absent. |
| `seed` | `seed` | Copy if present as a JSON integer (not a quoted string). Omit if absent. |

Key order within the `world` object follows the table order above: `name`, `urd`, `version`, `description`, `author`, `start`, `entry`, `seed`. Absent fields are omitted entirely (no `null` values).

### Step 2: Build the `types` Block

Types are emitted in symbol table insertion order.

For each `TypeSymbol` in the symbol table:

1. The JSON key is the type's declared name (e.g., `"Key"`, `"LockedDoor"`).
2. Emit `description` if present on the type definition. Omit if absent.
3. Emit `traits` as an array of trait strings (e.g., `["portable"]`, `["container", "interactable"]`). Omit if the type has no traits.
4. Emit `properties` as an object. For each property on the type:
   a. The JSON key is the property name.
   b. Emit `type` — the property type string (`"boolean"`, `"integer"`, `"number"`, `"string"`, `"enum"`, `"ref"`, `"list"`).
   c. Emit `default` if declared. Omit if absent.
   d. Emit `visibility` if not the default `"visible"`. If the property is marked `hidden` (via `~` prefix in source), emit `"hidden"`. Omit if `"visible"` (the schema default).
   e. Emit `description` if present. Omit if absent.
   f. Emit `values` for enum types — the array of valid value strings.
   g. Emit `min` and `max` for integer/number types if declared. Omit if absent.
   h. Emit `ref_type` for ref types if declared. Omit if absent.

Property key order within each property object: `type`, `default`, `visibility`, `description`, `values`, `min`, `max`, `ref_type`. Absent fields omitted.

If no TypeSymbols exist, omit the entire `types` block.

### Step 3: Build the `entities` Block

Entities are emitted in symbol table insertion order.

For each `EntitySymbol` in the symbol table:

1. **Player omission rule.** If no explicit `@player` entity was declared, the `entities` block does not contain a player entry. The runtime creates one implicitly. If an explicit `@player` was declared, it is emitted like any other entity.
2. The JSON key is the entity ID (the declared `@name` without the `@` sigil — e.g., `@rusty_key` → `"rusty_key"`).
3. Emit `type` — the type name string (e.g., `"Key"`).
4. Emit `properties` — an object containing only the property overrides declared on the entity (not the full type property set). **Omit the `properties` key entirely if the entity has no overrides.** For each override:
   a. The JSON key is the property name.
   b. The value is the override value, serialised according to its type: booleans as JSON booleans, integers as JSON integers, numbers as JSON numbers, strings as JSON strings, enum values as JSON strings, ref values as entity ID strings (without `@`), lists as JSON arrays.

If no EntitySymbols exist (or only an undeclared implicit player), omit the entire `entities` block.

### Step 4: Build the `locations` Block

Locations are emitted in symbol table insertion order.

For each `LocationSymbol` in the symbol table:

1. The JSON key is the location's `compiled_id` (the slugified form — e.g., `"cell"`, `"corridor"`).
2. Emit `description` — the prose text following the `# Heading` in the AST. Concatenate all `Prose` nodes between the `LocationHeading` and the next heading, `EntityPresence`, `Choice`, `ExitDeclaration`, or `SectionLabel`. **Join rule:** normalise line endings to `\n`, trim leading and trailing whitespace per prose block, then join multiple prose blocks with `"\n\n"`. Omit if no prose exists.
3. Emit `contains` — an array of entity ID strings (without `@`) from `EntityPresence` nodes under this location. Omit if empty.
4. Emit `exits` — an object. For each `ExitSymbol` associated with this location:
   a. The JSON key is the exit direction string (e.g., `"north"`, `"south"`).
   b. Emit `to` — the destination location's `compiled_id`, read from `ExitSymbol.resolved_destination`.
   c. Emit `condition` — if the exit has a condition, lower it to a condition string (see Condition Lowering). Omit if unconditional.
   d. Emit `blocked_message` — the text from the `BlockedMessage` node. Omit if absent.
   e. Emit `effects` — an array of effect objects (see Effect Lowering). Omit if no effects.

Exits are collected in AST declaration order within the location (the order `ExitDeclaration` nodes appear under the `LocationHeading` in the AST). This is the same order stored on the `LocationSymbol`'s exit list by LINK during collection. Exits are emitted as object entries in that order when using an order-preserving JSON serialiser. JSON semantics do not guarantee key order, so consumers must not depend on exit ordering — it exists only for deterministic output.

If no LocationSymbols exist, omit the entire `locations` block.

### Step 5: Build the `rules` Block

Rules are emitted in symbol table insertion order.

For each `RuleSymbol` in the symbol table:

1. The JSON key is the rule's declared name.
2. Emit `description` if present. Omit if absent.
3. Emit `actor` — the entity ID string (without `@`). Omit if absent.
4. Emit `trigger` — the trigger string as declared (e.g., `"phase_is reveal"`, `"action unlock"`, `"enter cell"`, `"state_change guard.mood"`, `"always"`). The JSON Schema constrains triggers to `^(phase_is \S+|action \S+|enter \S+|state_change \S+|always)$`. The identifier after the trigger keyword is always a single non-whitespace token. EMIT copies the trigger string from the AST without transformation.
5. Emit `conditions` — using Condition Lowering. Omit if no conditions.
6. Emit `select` — if the rule has a select block:
   a. Emit `from` — array of entity ID strings (without `@`).
   b. Emit `as` — the bound variable name string.
   c. Emit `where` — array of condition strings. Omit if no where clauses.
7. Emit `effects` — array of effect objects (see Effect Lowering). Required by schema (at least one).

If no RuleSymbols exist, omit the entire `rules` block.

### Step 6: Build the `actions` Block

Actions are emitted in symbol table insertion order.

For each `ActionSymbol` in the symbol table:

1. The JSON key is the action's `compiled_id`.
2. Emit `description` if present. Omit if absent.
3. Emit `actor` — the entity ID string (without `@`). Omit if absent.
4. Emit `target` — the entity ID string (without `@`). Omit if absent. Mutually exclusive with `target_type` (enforced by VALIDATE).
5. Emit `target_type` — the type name string. Omit if absent.
6. Emit `conditions` — using Condition Lowering. Omit if no conditions.
7. Emit `effects` — array of effect objects (see Effect Lowering). Required by schema.

If no ActionSymbols exist, omit the entire `actions` block.

### Step 7: Build the `sequences` Block

Sequences are emitted in symbol table insertion order.

For each `SequenceSymbol` in the symbol table:

1. The JSON key is the sequence's `compiled_id`.
2. Emit `description` if present. Omit if absent.
3. Emit `phases` — an array of phase objects, in declaration order. For each `PhaseSymbol`:
   a. Emit `id` — the phase's `compiled_id`.
   b. Emit `prompt` — the prompt text string. Omit if absent.
   c. Emit `auto` — boolean. Omit if `false` (the default).
   d. Emit `action` — the action ID string. Omit if absent. Mutually exclusive with `actions` (enforced by schema).
   e. Emit `actions` — array of action ID strings. Omit if absent.
   f. Emit `rule` — the rule ID string. Omit if absent.
   g. Emit `effects` — array of effect objects. Omit if no effects.
   h. Emit `advance` — the advance mode string. Required. The value is one of four patterns: `"on_action"`, `"on_rule"`, `"on_condition {expression}"`, or `"end"`. When the advance mode is `on_condition`, the condition expression is embedded directly in the advance string after a single space.

   **Advance expression encoding.** The JSON Schema constrains the `advance` field to match `^(on_action|on_rule|on_condition \S+|end)$`. The `\S+` after `on_condition ` must be a single non-whitespace token. Therefore EMIT must lower the embedded condition expression to a **space-free canonical form**: strip all spaces around operators. Examples: `"on_condition guard.mood==helpful"`, `"on_condition key.container==player.container"`, `"on_condition door.locked!=true"`. This space-free encoding applies **only** to the `advance` embedded expression. Normal `conditions` and `condition` strings use the standard spaced form (e.g., `"guard.mood == helpful"`) because the schema imposes no whitespace restriction on those fields.

   The same keyword expansion rules apply: `KeywordHere` annotations produce `player.container`, `KeywordPlayer` annotations produce `player`, `@` sigils are stripped. EMIT does not string-match — it reads LINK annotations and builds the space-free string directly.
   i. Emit `condition` — a separate phase-level condition string (plain string, not an array). This is independent of the `advance` field. The `condition` field gates whether the phase is active; the `advance` field determines how the phase progresses. Both may be present. Omit if absent.

If no SequenceSymbols exist, omit the entire `sequences` block.

### Step 8: Build the `dialogue` Block

Sections are emitted in symbol table insertion order.

For each `SectionSymbol` in the symbol table:

1. The JSON key is the section's `compiled_id` (e.g., `"tavern/topics"`).
2. Emit `id` — same as the JSON key. Required by schema.

**Section content assembly.** EMIT walks the AST content nodes that belong to this section (all nodes between the `SectionLabel` and the next `SectionLabel`, `LocationHeading`, or end of file). These nodes fall into three ordered regions:

**Region A — Pre-choice content.** All nodes before the first `Choice` node in the section.

- **`Prose` nodes** → concatenated into the section's `description` field. Join rule: normalise line endings to `\n`, trim leading and trailing whitespace per prose block, then join multiple prose blocks with `"\n\n"`. Omit `description` if no prose exists in Region A.
- **`EntitySpeech` node** → the first `EntitySpeech` in Region A becomes the section's `prompt` field, using the speech object shape. If multiple `EntitySpeech` nodes appear before the first choice, only the first becomes the `prompt`; subsequent ones are included in `description` as narrative text.
- **`StageDirection` nodes** → included in `description` as narrative text.
- **`Condition` nodes** → emitted as section-level `conditions` using standard condition lowering.

**Speech object shape.** A `speech` object always has `text` (required, string). `speaker` is optional — present when a speaker entity is named (`@speaker: text`), omitted when the speech has no attributed speaker. Examples: `{ "speaker": "arina", "text": "The Selene didn't sink." }` or `{ "text": "Narration text." }`. The `on_exhausted` field uses the same speech object shape.

3. Emit `prompt` — the first `EntitySpeech` in Region A, as a speech object. Omit if no speech exists before the first choice.
4. Emit `description` — prose and stage directions from Region A. Omit if empty.
5. Emit `conditions` — section-level conditions from Region A. Omit if none.

**Region B — Choices.** The contiguous block of `Choice` nodes (including nested content).

6. Emit `choices` — an array of choice objects, in declaration order. For each `ChoiceSymbol` (and its corresponding `Choice` AST node):

   a. Emit `id` — the choice's `compiled_id` (e.g., `"tavern/topics/ask-about-the-harbor"`).
   b. Emit `label` — the choice label text as written by the author.
   c. Emit `sticky` — `true` for `+` choices, `false` for `*` choices.
   d. Emit `conditions` — lowered from `Condition` nodes in the choice's `content[]`. Uses the `conditionExpr` shape (array for AND, `{ "any": [...] }` for OR). Omit if no conditions.
   e. Emit `response` — the first `EntitySpeech` node in the choice's `content[]`, as a speech object. Omit if absent.
   f. Emit `effects` — lowered from `Effect` nodes in the choice's `content[]`. Omit if no effects.
   g. Emit `goto` — derived from the choice's navigation target:
      - If the choice has a `Jump` node (`-> name`) in its content, and the jump resolves to a `SectionSymbol`, emit the section's `compiled_id`.
      - If the jump target is `end` (literal), omit `goto` — `-> end` signals section termination, not a cross-reference.
      - If the choice has `target` (entity ref via `-> @entity`), `goto` is omitted — entity-targeted choices navigate via the action system, not section jumps.
      - If the choice has `target_type` (via `-> any TypeName`), `goto` is omitted — same reason.
      - If the choice has no jump, no target, and no target_type, `goto` is omitted.
   h. Emit `choices` — nested sub-choices (recursive application of steps 6a–6h). Omit if no nested choices.

   **Key order within each choice object:** `id`, `label`, `sticky`, `conditions`, `response`, `effects`, `goto`, `choices`. Absent fields omitted.

   **Choices with both target and nested content.** A choice may have a `-> @entity` or `-> any Type` target AND inline conditions/effects in its `content[]`. This is valid — the target becomes the action's `target` or `target_type`, while the inline conditions/effects become the action's conditions/effects. The choice object in the `dialogue` block includes the conditions and effects (steps 6d–6f) but omits `goto` (step 6g — target-directed choices navigate via actions).

**Region C — Post-choice content (on_exhausted).** All nodes after the last top-level `Choice` node in the section.

7. Emit `on_exhausted` — if Region C contains content, assemble it as follows:
   - If Region C starts with an `EntitySpeech` node, emit it as a speech object: `{ "speaker": "{entity_id}", "text": "{speech_text}" }`.
   - If Region C starts with a `Prose` node (or `StageDirection`), emit as a speech object with no speaker: `{ "text": "{concatenated prose}" }`.
   - If Region C contains a `Jump` node after the speech/prose, include a `goto` field on the speech object pointing to the target section's `compiled_id`.
   - Omit `on_exhausted` if Region C is empty.

   The `on_exhausted` detection rule is purely positional: any content nodes that appear after the last top-level `Choice` in the section constitute the exhaustion fallthrough. Nested content inside choices does not count — only nodes at the section's top indentation level.

If no `SectionSymbol`s exist, omit the entire `dialogue` block.

**Key order within each section object:** `id`, `prompt`, `description`, `conditions`, `choices`, `on_exhausted`. Absent fields omitted.

## Condition Lowering

AST condition nodes are lowered to JSON strings. These strings are then assembled into the condition structures defined by the JSON Schema.

### Single Condition Strings

Each AST condition node lowers to a single string expression:

| AST Condition Type | Lowered String | Example |
|-------------------|----------------|---------|
| `PropertyComparison` | `"{entity}.{property} {op} {value}"` | `"cell_door.locked == true"` |
| `ContainmentCheck` (entity in entity) | `"{entity}.container == {container}"` | `"rusty_key.container == chest"` |
| `ContainmentCheck` (entity in location) | `"{entity}.container == {location_id}"` | `"rusty_key.container == cell"` |
| `ContainmentCheck` (`container_kind` = `KeywordHere`) | `"{entity}.container == player.container"` | `"rusty_key.container == player.container"` |
| `ContainmentCheck` (`container_kind` = `KeywordPlayer`) | `"{entity}.container == player"` | `"rusty_key.container == player"` |
| `ContainmentCheck` (negated) | `"{entity}.container != {container}"` | `"rusty_key.container != player"` |
| `ExhaustionCheck` | `"{section_compiled_id}.exhausted"` | `"tavern/topics.exhausted"` |

**Entity IDs in condition strings are emitted without the `@` sigil.** `@rusty_key` in source becomes `rusty_key` in the condition string. Entity IDs are always the declared entity ID, never a compiled ID. Entities do not have compiled IDs — their declared `@name` (minus the `@`) is their ID everywhere.

**Keyword expansion is driven by LINK annotations only.** `here` expands to `player.container` only when LINK set `container_kind` or `destination_kind` to `KeywordHere`. `player` emits as the entity ID `player` only when LINK set `container_kind` or `destination_kind` to `KeywordPlayer`. EMIT does not perform string matching on `here` or `player` — it reads the discriminator and emits the appropriate string. There is no `here` or `player` keyword in compiled JSON.

### Condition Field Shapes

The JSON Schema defines two condition field patterns. EMIT must use the correct one depending on the field:

**`conditions` (plural) — uses `conditionExpr`:**
Used on: choices, actions, rules, sections.

- **AND list (no OR block):** Emit a JSON array of condition strings. Always an array, even for a single condition.
  ```json
  ["cell_door.locked == true", "rusty_key.container == player"]
  ```
- **OR block (`? any:` syntax):** Emit a JSON object with an `any` key containing an array of condition strings.
  ```json
  { "any": ["player.reputation > 50", "bribe_gold.container == player"] }
  ```
- **AND and OR cannot be mixed in one `conditions` field.** A `conditions` field is either an array of AND strings or a single `any` object. This is enforced by the `conditionExpr` schema definition (`oneOf` — array or object, not both).

**`condition` (singular) — plain string:**
Used on: exits, phases.

- Emit a single condition string. Not an array, not an object.
  ```json
  "condition": "cell_door.locked == false"
  ```
- **Single condition constraint.** Exits and phases support at most one condition string per the JSON Schema (the `condition` field is `type: string`, not `conditionExpr`). If the source syntax produces multiple conditions on an exit or phase, they must be rejected during VALIDATE or PARSE — EMIT cannot represent an AND list in a single string field. In v1, exits and phases accept exactly zero or one condition. EMIT must never emit an empty string for `condition` — if no condition exists, omit the field entirely.

## Effect Lowering

AST effect nodes are lowered to structured JSON effect objects.

| AST Effect Type | JSON Shape | Example |
|----------------|------------|---------|
| `set` (direct) | `{ "set": "{entity}.{property}", "to": {value} }` | `{ "set": "cell_door.locked", "to": false }` |
| `set` (arithmetic `+`) | `{ "set": "{entity}.{property}", "to": "{entity}.{property} + {N}" }` | `{ "set": "guard.trust", "to": "guard.trust + 10" }` |
| `set` (arithmetic `-`) | `{ "set": "{entity}.{property}", "to": "{entity}.{property} - {N}" }` | `{ "set": "guard.trust", "to": "guard.trust - 5" }` |
| `move` | `{ "move": "{entity}", "to": "{destination}" }` | `{ "move": "rusty_key", "to": "player" }` |
| `reveal` | `{ "reveal": "{entity}.{property}" }` | `{ "reveal": "door.prize" }` |
| `destroy` | `{ "destroy": "{entity}" }` | `{ "destroy": "rusty_key" }` |

**Entity IDs in effect fields are emitted without the `@` sigil.** Entity IDs are always the declared entity ID, never a compiled ID.

**Keyword expansion is driven by LINK annotations only (same rule as conditions).** When LINK set `destination_kind` to `KeywordHere`, emit `"player.container"` as the `to` value. When LINK set `destination_kind` to `KeywordPlayer`, emit `"player"`. When `destination_kind` is `LocationRef`, emit the location's `compiled_id`. When `destination_kind` is `EntityRef`, emit the entity ID (without `@`). EMIT does not string-match on `here` or `player` — it reads the discriminator.

**Arithmetic set values are emitted as expression strings, not computed values.** The runtime evaluates the expression at execution time.

**Direct set values are emitted as typed JSON values.** Booleans as JSON booleans, integers as JSON integers, numbers as JSON numbers, strings as JSON strings, enum values as JSON strings, ref values as entity ID strings (without `@`). Ref values always use the declared entity ID (e.g., `"rusty_key"`), never a compiled ID — entities do not have compiled IDs.

### Schema Forward Compatibility

The Urd World Schema (`urd-world-schema.json`) defines additional constructs that EMIT does not produce in v1:

- **`spawn` effect.** The schema supports `{ "spawn": { "id": "...", "type": "...", "in": "..." } }`. The v1 AST does not produce spawn nodes (spawn effects appear only in hand-authored JSON or are emitted by rules at runtime — see architecture brief). EMIT does not emit spawn effects.
- **`owner` visibility.** The schema supports `"owner"` as a visibility value. The v1 Schema Markdown syntax does not produce `owner` visibility. EMIT emits `"hidden"` (from `~` prefix) or omits visibility (default `"visible"`).
- **Conditional visibility.** The schema supports `{ "type": "conditional", "condition": "..." }` as a visibility value. The v1 Schema Markdown syntax does not produce conditional visibility. EMIT does not emit conditional visibility objects.

EMIT emits only what the v1 AST can produce. The schema is forward-compatible for future syntax extensions.

## Determinism

The architecture brief requires byte-identical output for identical source. **Byte-identical output is guaranteed for the same compiler version and configuration.** Compiler upgrades may change serialisation details (e.g., numeric formatting, whitespace) and are not required to preserve byte-identity across versions. EMIT guarantees determinism through:

1. **Fixed top-level key order.** `world`, `types`, `entities`, `locations`, `rules`, `actions`, `sequences`, `dialogue`. Always this order.
2. **Symbol table insertion order for block entries.** Types, entities, locations, rules, actions, sequences, and sections are emitted in the order they appear in the symbol table, which follows topological file order then declaration order within each file.
3. **Declaration order for sub-entries.** Properties within a type, overrides on an entity, exits within a location, phases within a sequence, choices within a section — all follow AST declaration order.
4. **Fixed key order within objects.** Each JSON object type has a defined key order (specified in each step above). Absent keys are omitted without affecting the order of present keys.
5. **No non-deterministic sources.** No timestamps, no random IDs, no platform-dependent values. All IDs are derived from source content by LINK.
6. **Consistent JSON serialisation.** The serialiser must be both order-preserving (emitting object keys in the order they are inserted) and deterministic (producing identical output for identical input across runs). Implementation requirement: use a serialiser with these properties (no hash-map-order key emission).

### Serialisation Rules

- **Indentation:** Two spaces per level.
- **Trailing newline:** The output ends with a single newline character (`\n`).
- **No trailing commas.** Standard JSON.
- **String escaping:** Use minimal JSON escaping. Only escape characters required by the JSON specification (`"`, `\`, control characters). Do not escape `/` or non-ASCII characters.
- **Numeric formatting:** Integers are emitted without a decimal point. Numbers with fractional parts are emitted using the implementation language's standard JSON serialiser with deterministic output. Scientific notation is permitted if the serialiser produces it. The requirement is **determinism**: the same numeric value must always produce the same string for a given compiler version and platform target.
- **Boolean formatting:** `true` and `false` (lowercase).
- **Null:** Never emitted. Absent fields are omitted, not set to `null`. The JSON Schema allows `null` in entity property override values and list items, but v1 Schema Markdown does not permit author-level `null` literals — this is a **language restriction**, not a schema restriction. VALIDATE rejects `null` values in v1, so EMIT never encounters them. Other JSON producers (hand-authored files, future tools) may emit `null` where the schema allows it. If a future language version permits `null`, EMIT must be updated to handle it.

## Diagnostic Catalog

All diagnostics emitted by EMIT are in the URD500–URD599 range.

**EMIT has zero diagnostics in v1.** EMIT is a straightforward traversal of pre-validated data. All uniqueness checks, including choice ID collisions after slugification (URD501), are owned by LINK and detected during LINK's collection sub-pass. LINK guarantees that every `compiled_id` is unique within its namespace before EMIT runs.

Future schema versions may introduce EMIT diagnostics for output-specific concerns (e.g., JSON size limits, source map generation errors).

## Skip Rules

EMIT inherits the upstream guarantee that no Error-severity diagnostics exist. Since unresolved references are LINK errors, and any LINK error prevents EMIT from running, all annotations should be populated when EMIT runs.

**Invariant rule:** If EMIT encounters a `null` annotation where a resolved reference is expected, this is an internal invariant violation — it means a compiler bug allowed an error to slip through. EMIT skips the construct and logs an internal warning. Implementations should treat this as a bug report trigger, not a normal code path.

- `ErrorNode` entries in the AST are skipped silently (they should not exist if EMIT is running, but defensive skipping is harmless).
- `Comment` nodes are skipped (stripped from output).

## Acceptance Criteria

### World Block Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Minimal world | `world: test`, `start: cell`, one location `cell`. | `{ "world": { "name": "test", "urd": "1", "start": "cell" }, "locations": { "cell": {} } }` |
| Full world metadata | All optional fields present (`version`, `description`, `author`, `entry`, `seed`). | All fields emitted in key order. `urd: "1"` injected. |
| Author urd override | Author sets `urd: "2"` in frontmatter. | `urd: "1"` emitted. (VALIDATE already warned via URD411.) |
| Absent optional fields | No `version`, `description`, `author`, `entry`, `seed`. | Only `name`, `urd`, `start` present. No `null` values. |

### Type Block Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Type with all property types | Boolean, integer, number, string, enum, ref, list. | Each property serialised with correct `type` field and applicable constraints. |
| Hidden property | `~mood: enum` | `visibility: "hidden"` on the property. |
| Visible property (default) | `mood: enum` | No `visibility` field emitted. |
| Type with traits | `Key [portable]` | `"traits": ["portable"]` |
| Type with no traits | `Door:` (no trait brackets) | No `traits` field emitted. |
| Omit empty types block | No types declared. | No `types` key in output. |

### Entity Block Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Entity with overrides | `@rusty_key: Key { name: "Rusty Key" }` | `"rusty_key": { "type": "Key", "properties": { "name": "Rusty Key" } }` |
| Entity with no overrides | `@cell_door: LockedDoor` | `"cell_door": { "type": "LockedDoor" }`. No `properties` key. |
| No explicit player | No `@player` declared. | No `player` entry in entities. |
| Explicit player | `@player: Hero { health: 100 }` | `"player": { "type": "Hero", "properties": { "health": 100 } }` |
| Ref override | `@door: LockedDoor { requires: @rusty_key }` | `"requires": "rusty_key"` (no `@`). |

### Location Block Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Location with description | `# Cell` followed by prose. | `"cell": { "description": "A dim stone cell." }` |
| Location with contains | `[@rusty_key, @cell_door]` | `"contains": ["rusty_key", "cell_door"]` |
| Location with exits | `-> north: Corridor` | `"exits": { "north": { "to": "corridor" } }` |
| Exit with condition | `-> north: Corridor` + `? @door.locked == false` | `"condition": "door.locked == false"` |
| Exit with blocked message | `! The door is locked.` | `"blocked_message": "The door is locked."` |
| Omit empty locations block | No locations declared. | No `locations` key in output. |

### Condition Lowering Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Property comparison | `? @guard.mood == neutral` | `"guard.mood == neutral"` |
| Containment (entity in entity) | `? @key in @chest` | `"key.container == chest"` |
| Containment (entity in here) | `? @key in here` | `"key.container == player.container"` |
| Containment (entity in player) | `? @key in player` | `"key.container == player"` |
| Containment (negated) | `? @key not in player` | `"key.container != player"` |
| Exhaustion check | `? topics.exhausted` | `"tavern/topics.exhausted"` (full section ID) |
| AND conditions | Two conditions on a choice. | Array of two strings. |
| OR conditions | `? any:` block. | `{ "any": ["...", "..."] }` |

### Effect Lowering Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Set boolean | `> @door.locked = false` | `{ "set": "door.locked", "to": false }` |
| Set string | `> @guard.mood = helpful` | `{ "set": "guard.mood", "to": "helpful" }` |
| Arithmetic add | `> @guard.trust + 10` | `{ "set": "guard.trust", "to": "guard.trust + 10" }` |
| Move to player | `> move @key -> player` | `{ "move": "key", "to": "player" }` |
| Move to here | `> move @key -> here` | `{ "move": "key", "to": "player.container" }` |
| Move to location | `> move @key -> cell` | `{ "move": "key", "to": "cell" }` |
| Reveal | `> reveal @door.prize` | `{ "reveal": "door.prize" }` |
| Destroy | `> destroy @key` | `{ "destroy": "key" }` |

### Sequence and Advance Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Advance on_action | Phase with `advance: on_action`. | `"advance": "on_action"` |
| Advance on_condition (space-free) | Phase with advance on condition `@guard.mood == helpful`. | `"advance": "on_condition guard.mood==helpful"` (no spaces in expression). |
| Advance on_condition with here | Phase with advance on condition `@key in here`. | `"advance": "on_condition key.container==player.container"` |
| Advance end | Phase with `advance: end`. | `"advance": "end"` |
| Phase with both advance and condition | Phase with `advance: on_action` and `condition: @door.locked == false`. | Both fields present. `"advance": "on_action"`, `"condition": "door.locked == false"`. |

### Dialogue Block Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Section with choices | `== topics` with two `*` choices. | Section with `id`, `choices` array. Each choice has `id`, `label`, `sticky: false`. |
| Sticky choice | `+ Ask again` | `"sticky": true` |
| Choice with goto | `-> farewell` | `"goto": "tavern/farewell"` (full section ID). |
| Choice with goto end | `-> end` | No `goto` field. |
| Nested choices | Choice containing sub-choices. | `choices` array within the parent choice. |
| Section with on_exhausted | Prose after choice block. | `"on_exhausted": { "text": "..." }` |
| Omit empty dialogue block | No sections declared. | No `dialogue` key in output. |

### Additional Dialogue Block Tests

| Test | Input | Expected Output |
|------|-------|-----------------| 
| Section description from prose | `== topics` followed by prose "The tavern is quiet." then choices. | `"description": "The tavern is quiet."` |
| Section prompt from speech | `== topics` then `@arina: What'll it be?` then choices. | `"prompt": { "speaker": "arina", "text": "What'll it be?" }` |
| Section with both prompt and description | `== topics` then prose then `@arina: Hello` then choices. | `"description": "..."`, `"prompt": { "speaker": "arina", "text": "Hello" }` |
| Choice with conditions and effects | Choice with `? condition` and `> effect` in content. | `"conditions": ["..."]`, `"effects": [...]` |
| Choice with entity target and conditions | `* Use key -> @cell_door` with conditions in content. | Choice has `conditions` but no `goto`. Action has `target: "cell_door"` and conditions. |
| Choice with type target | `* Pick a door -> any Door` | Choice has no `goto`. Action has `target_type: "Door"`. |
| Choice with section jump | `* Leave -> farewell` where farewell is a section. | `"goto": "tavern/farewell"` on choice. |
| Choice with -> end | `* Done -> end` | No `goto` field on choice. |
| on_exhausted with speech | Choices followed by `@arina: Come back later.` | `"on_exhausted": { "speaker": "arina", "text": "Come back later." }` |
| on_exhausted with prose | Choices followed by plain prose. | `"on_exhausted": { "text": "..." }` |
| on_exhausted with goto | Choices followed by prose and `-> farewell`. | `"on_exhausted": { "text": "...", "goto": "tavern/farewell" }` |
| No on_exhausted | Section with choices but nothing after them. | No `on_exhausted` field. |
| Section-level conditions | `? guard.mood == neutral` before choices. | `"conditions": ["guard.mood == neutral"]` on section. |
| Nested choice generates action | Sub-choice "Insist" under parent choice. | Action with ID `section_id/insist` in `actions` block. |
| Multiple prose blocks in description | Two prose paragraphs separated by blank line before choices. | `"description": "First paragraph.\n\nSecond paragraph."` |

### Determinism Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Byte-identical output | Compile same source twice. | Outputs are byte-identical. |
| Cross-file ordering | File A imports file B. B declares type X, A declares type Y. | Types block: X before Y (topological order). |
| Tiebreaker ordering | Two files at same depth, no dependency. `b.urd.md` and `a.urd.md`. | `a.urd.md` declarations before `b.urd.md` (alphabetical). |
| Key order within objects | Entity with `type` and `properties`. | `type` before `properties` in JSON. |

### Integration Tests

| Test | Input | Expected Output |
|------|-------|-----------------|
| Two Room Key Puzzle | Full source from architecture brief worked example. | Valid JSON matching schema. All conditions lowered. All effects structured. `here` expanded. `@` stripped. |
| Monty Hall | Full Monty Hall source. | Sequences, phases, rules, select blocks all present. `urd: "1"` injected. |
| Multi-file | Entry file imports two files. | All declarations merged. Topological order respected. |
| Empty world | Minimal `world` block, one location, nothing else. | Only `world` and `locations` blocks present. |

Total: 46 test cases.

*End of Brief*
