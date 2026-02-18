# URD — Compiler Phase 4: VALIDATE

*Type checking, constraint enforcement, and semantic validation*

February 2026 | Engineering Phase

`LinkedWorld → VALIDATE → ValidatedWorld (diagnostics appended)`

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-18
**Status:** Complete

### What was done

- Prerequisite: added `ContainerKind` and `DestinationKind` discriminator enums to `ast.rs` and extended `Annotation` with `container_kind` and `destination_kind` fields. Fixed LINK `resolve.rs` to use keyword-first resolution for `ContainmentCheck.container_ref` and `Move.destination_ref` (replacing the previous `resolve_entity_ref_value` calls that emitted false URD301 errors for `player`, `here`, and location names). Added 6 LINK tests for the new resolution paths.
- Created `validate/helpers.rs` with shared type-checking logic: `check_value`, `parse_string_to_value`, `has_trait`, `format_value`, `format_property_type`, and a `CheckContext` enum (`Override`, `Default`, `ConditionOrEffect`) to control which diagnostic codes are emitted for enum mismatches.
- Created `validate/types.rs` (Step 2): type definition validation — property defaults (URD413), empty enum values (URD414), ref type existence (URD415), range validity (URD416), range type compatibility (URD417).
- Created `validate/entities.rs` (Step 3): entity property override validation — type check (URD401), enum value (URD402), range (URD418), ref type (URD419).
- Created `validate/conditions.rs` (Step 4): condition validation — PropertyComparison operator compatibility (URD420) and value type (URD401), ContainmentCheck container trait via `container_kind` discriminator (URD422), ExhaustionCheck file-locality (URD423).
- Created `validate/effects.rs` (Step 5): effect validation — Set value type and arithmetic operator (URD401, URD424), Move portable trait and destination kind (URD425, URD422), Reveal visibility (URD426), Destroy (no checks).
- Rewrote `validate/mod.rs` as the entry point calling all 8 steps, with Steps 1 (global config: URD404, URD405, URD411, URD412), 6 (action mutual exclusion: URD406), 7 (sequence/phase validation: URD407, URD408, URD409, URD427, URD428), and 8 (nesting depth: URD410) implemented inline.
- Wrote 70 VALIDATE tests covering all 25 diagnostic codes across property type checking (15), condition validation (10), effect validation (15), structural constraints (17), skip rule (4), and integration (5 including Monty Hall). Four structural tests (`phase_rule_valid`, `phase_rule_invalid`, `advance_mode_invalid`, `auto_phase_with_actions`) use a `link_modify_and_validate` helper that mutates the symbol table between LINK and VALIDATE to exercise code paths that LINK's current phase collection doesn't populate.
- All 279 tests pass (6 slugify + 55 import + 72 link + 76 parse + 70 validate). Zero new warnings.

### What changed from the brief

- **`VALID_ADVANCE_MODES` includes `"auto"` and `"manual"` beyond the brief's four modes.** The brief specifies four valid advance modes: `on_action`, `on_rule`, `on_condition`, `end`. However, LINK's `collect_phase` defaults `advance` to `"auto"` (when `phase.auto == true`) or `"manual"` (otherwise). If only the brief's four modes were accepted, every phase would trigger URD409 on creation. Adding `"auto"` and `"manual"` to the valid set is a necessary deviation.
- **URD423 (exhaustion cross-file) is dead code.** The brief says VALIDATE should emit URD423 when an exhaustion check references a section in another file. In practice, LINK resolves exhaustion checks only against `ctx.local_sections` (the current file). Cross-file section names fail LINK resolution (URD309) and the annotation is left `None`, so VALIDATE skips via the null-annotation rule. The URD423 check is retained as a safety net but can never fire given LINK's current resolution model. The `exhaustion_cross_file` test correctly asserts URD309 from LINK rather than URD423.
- **`CheckContext` enum added to helpers.** The brief implies URD402 is emitted only for entity property overrides (Step 3), while conditions and effects use URD401 for all mismatches including enum. The implementation uses a `CheckContext` enum (`Override`, `Default`, `ConditionOrEffect`) passed to `check_value` to select the correct diagnostic code, which is a structural addition not detailed in the brief.
- **Auto phase detection uses `phase.advance == "auto"` instead of a boolean field.** The brief says "if a phase is marked `auto: true`" but `PhaseSymbol` has no separate `auto` boolean — LINK encodes it as `advance: "auto"`. The implementation checks the advance field directly, which is functionally equivalent.
- **Test count is 70 instead of the planned ~65.** The conformance review identified 8 missing acceptance criteria tests (keyword-location shadowing in moves, world.start with shadowed location, phase rule valid/invalid, advance mode invalid, auto phase with actions, Monty Hall integration) which were added in a follow-up pass.

---

> **Document status: BRIEF** — Defines the VALIDATE phase of the Urd compiler. VALIDATE is the fourth phase of the five-phase pipeline. It takes the `LinkedWorld` produced by LINK (annotated ASTs, a populated symbol table, and a dependency graph), checks every semantic constraint the language defines, and produces diagnostics. VALIDATE does not transform the AST or modify the symbol table — it reads them and reports errors.

> **Dependencies:** This brief builds on the Compiler Architecture Brief (property types, effect subtypes, condition expression types, nesting limits, error recovery, diagnostic ordering), the Schema Specification (type constraints, trait rules, action mutual exclusion, advance modes, visibility rules), and the LINK Phase Brief (annotation model, resolution guarantees, visible scope). All are required reading.


## Purpose

VALIDATE is the phase where the compiler enforces every semantic rule that cannot be checked by syntax alone. LINK proved that names exist and references resolve. VALIDATE proves that resolved references are used correctly: types match, values are in range, traits are present, effects target valid properties, and structural constraints hold.

VALIDATE has exactly five jobs:

1. **Type-check property values.** Verify that every property override on every entity matches the declared property type (boolean, integer, number, string, enum, ref, list). Verify that defaults declared on types are also valid.
2. **Enforce range and enum constraints.** Check that integer and number values fall within declared min/max bounds. Check that enum values are members of the declared value set.
3. **Validate references and traits.** Check that ref-typed properties point to entities of the correct type. Check that `move` effects target entities with the `portable` trait and destinations with the `container` trait. Check that `destroy` targets are entities. Check that the explicit `@player` entity (if declared) has `mobile` and `container` traits.
4. **Check structural constraints.** Validate action mutual exclusion (`target` vs `target_type`). Validate nesting depth limits. Validate `world.start` and `world.entry` references. Validate sequence phase advance modes and action/rule references.
5. **Validate conditions and effects.** Check that condition expressions reference valid properties with compatible operators and values. Check that effect targets are valid and effect values match the target property's type.

### What VALIDATE Does

- Checks every `EntitySymbol.property_overrides` value against the property's declared type, range, and enum constraints.
- Checks every `TypeDef` property default against the property's own type and constraints.
- Checks that `ref`-typed property values point to entities whose type matches the declared `ref_type`.
- Checks that `move` effect entities have the `portable` trait on their type.
- Checks that `move` effect destinations have the `container` trait (if an entity) or are locations.
- Checks that an explicit `@player` entity's type has both `mobile` and `container` traits.
- Validates `world.start` resolves to an existing location (URD404).
- Validates `world.entry` resolves to an existing sequence, if present (URD405).
- Checks action mutual exclusion: actions must declare either `target` or `target_type`, not both (URD406).
- Checks that `action` and `actions` references in sequence phases point to declared actions (URD407).
- Checks that `rule` references in sequence phases point to declared rules (URD408).
- Validates sequence phase `advance` modes are one of: `on_action`, `on_rule`, `on_condition`, `end` (URD409).
- Checks choice nesting depth and emits URD410 with mixed severity. Depth 3 = warning. Depth 4+ = error.
- Validates `PropertyComparison` conditions: operator is compatible with the property type, comparison value matches the type.
- Validates `ContainmentCheck` conditions: entity reference is resolved, container `container_kind` discriminator is checked for trait requirements.
- Validates `ExhaustionCheck` conditions: section reference is resolved and file-local.
- Validates `set` effects: target property exists, value matches property type, arithmetic operators (`+`, `-`) only apply to integer and number types.
- Validates `reveal` effects: target property exists and has `visibility: hidden`.
- Skips validation of any construct whose annotation is `null` (unresolved reference from LINK). One root cause, one diagnostic.
- Skips `ErrorNode` entries silently.
- Emits a warning if the author set the `urd` field in the `WorldBlock` (URD411).

### What VALIDATE Does Not Do

- Resolve references. All name resolution is complete before VALIDATE runs. VALIDATE reads annotations, it does not populate them.
- Modify the AST or symbol table. VALIDATE is read-only. It produces diagnostics only.
- Generate JSON. That is EMIT's job.
- Check for duplicate declarations. That is LINK's job.
- Check import structure. That is IMPORT's job.


## Interface Contract

### Input

```
validate(linked_world: LinkedWorld, diagnostics: DiagnosticCollector) → ValidatedWorld
```

- `linked_world`: The `LinkedWorld` from LINK, containing the `SymbolTable`, annotated `FileAST`s, and `DependencyGraph`.
- `diagnostics`: The shared diagnostic collector. VALIDATE appends to it.

### Output

```
ValidatedWorld {
  symbol_table: SymbolTable,      // passed through unchanged from LINK
  annotated_asts: FileAST[],      // passed through unchanged from LINK
  graph: DependencyGraph,         // passed through unchanged from LINK
}
```

VALIDATE always returns a `ValidatedWorld`. The data structures are identical to the input — VALIDATE does not modify them. The sole output is diagnostics appended to the collector. EMIT reads the `ValidatedWorld` and the diagnostic collector to decide whether to produce JSON.

### Guarantees

After VALIDATE completes, the following properties hold:

1. **Every resolved property override has been type-checked.** If `EntitySymbol.type_symbol` is not `null`, every override value has been validated against the property's declared type and constraints.
2. **Every resolved condition expression has been validated.** If the condition's entity and property annotations are not `null`, the operator and value have been checked for type compatibility.
3. **Every resolved effect has been validated.** If the effect's target annotations are not `null`, the effect type, target property, and value have been checked.
4. **Structural constraints are enforced.** Action mutual exclusion, nesting depth, `world.start`, `world.entry`, sequence phase references, and advance modes have all been checked.
5. **No cascading errors from LINK.** If a reference is unresolved (`null` annotation), VALIDATE has silently skipped all checks that depend on that reference. The LINK diagnostic is the sole record of the failure.
6. **Diagnostics are deterministic.** Same input always produces the same diagnostics in the same order (topological file order, then source position within each file).
7. **The AST and symbol table are unchanged.** VALIDATE is read-only.
8. **ErrorNodes are silently skipped.** No diagnostics are emitted for constructs inside `ErrorNode` entries.


## The Algorithm

VALIDATE performs a single pass over all `FileAST`s in topological order. Within each file, it walks the AST top-to-bottom, validating each construct against the symbol table. Additionally, it performs global checks that span the entire compilation unit (world configuration, player entity traits, type definition validation).

### Ordering

VALIDATE processes files in the same topological order as LINK (`ordered_asts` from IMPORT, entry file last). Within each file, validation follows source order (top to bottom). This produces deterministic diagnostic ordering consistent with all other phases.

Global checks that iterate the symbol table (steps 1–2, step 6, step 7) follow the symbol table's insertion order (topological file order, then declaration order within file). Implementers must iterate the `OrderedMap`, not an unordered hash map, to preserve diagnostic determinism.

### Skip Rule

Before validating any construct, VALIDATE checks whether the construct depends on an unresolved reference. The skip rule is:

- If an entity's `type_symbol` is `null`, skip all property validation for that entity.
- If a condition's entity annotation is `null`, skip the entire condition.
- If an effect's target entity annotation is `null`, skip the entire effect.
- If a property annotation is `null`, skip the value check for that property access.
- If a `ContainmentCheck` has `container_kind` = `null`, skip the entire containment check.
- If a move effect has `destination_kind` = `null`, skip the entire move effect.

Exit destinations are fully validated by LINK (resolution to `LocationSymbol`, scope enforcement). VALIDATE does not re-check exit destinations.

The principle: if LINK already reported an error for a missing reference, VALIDATE does not report secondary errors that are consequences of the same root cause.

### Emit Rule

When this algorithm says "emit URDxxx", the diagnostic message must use the catalog template for that code. The catalog is the sole canonical source for message strings. Parenthetical glosses in algorithm steps (e.g., "emit URD404 (start location not found)") are reading aids only.

### Step 1: Global Configuration Checks

Before walking individual files, VALIDATE checks world-level configuration:

a. **`world.start`** — If the `world` block declares a `start` value and LINK did not resolve it to a `LocationSymbol`, emit URD404 (start location not found).

b. **`world.entry`** — If the `world` block declares an `entry` value and LINK did not resolve it to a `SequenceSymbol`, emit URD405 (entry sequence not found). If no `entry` is declared, the world is freeform — no error.

c. **`urd` field override** — If the `WorldBlock` AST node contains an `urd` key, emit URD411 warning (author set urd field). The `urd` field is read from the `WorldBlock` node within the file's `Frontmatter`, not from raw frontmatter text.

d. **Player entity traits** — If an explicit `@player` entity is declared, look up its type. If the type is resolved, check that it has both `mobile` and `container` traits. If either is missing, emit URD412 (player type missing required trait).

### Step 2: Type Definition Validation

Types are validated in symbol table insertion order (topological file order, then declaration order within file).

For each `TypeSymbol` in the symbol table:

a. **Property defaults.** For each property that declares a default value, validate the default against the property's own type and constraints (using the same type-checking rules as entity property overrides in step 3). If the default is invalid, emit URD413 (type default fails type check).

b. **Enum values list.** If a property has type `enum`, verify that `values` is non-empty. If empty, emit URD414 (empty enum values list).

c. **Ref type existence.** If a property has type `ref` and declares a `ref_type`, verify that the `ref_type` resolves to a `TypeSymbol`. If not, emit URD415 (unknown ref type).

d. **Range validity.** If a property declares `min` and `max`, verify that `min ≤ max`. If not, emit URD416 (inverted range).

e. **Range type compatibility.** `min` and `max` are only valid on `integer` and `number` properties. If declared on other types, emit URD417 (range on wrong type).

### Step 3: Entity Property Override Validation

Entities are validated in symbol table insertion order (topological file order, then declaration order within file).

For each `EntitySymbol` in the symbol table, if `type_symbol` is not `null`:

For each property override `(property_name, value)`:

a. **Type check the value.** Apply the rules based on the property's declared type:

| Property Type | Valid Values | Check |
|---------------|-------------|-------|
| `boolean` | `true`, `false` | Value is a boolean literal. |
| `integer` | Whole numbers | Value parses as integer. If `min`/`max` declared, value is within range. |
| `number` | Decimal numbers | Value parses as number (integer is also valid). If `min`/`max` declared, value is within range. |
| `string` | Any text | Value is a string. |
| `enum` | Member of declared `values` | Value is one of the declared enum values. |
| `ref` | Entity ID | Value references an existing entity. If `ref_type` is declared, that entity's type must match exactly. |
| `list` | Array of values | Each element is validated individually — see below. |

**List validation.** List properties carry an `element_type` in the property definition (e.g., `list(string)`, `list(ref(Key))`). VALIDATE validates each element using the same rules as a scalar property of that element type, reusing URD401 and any applicable range, enum, and ref-type diagnostics. If the element type is `ref` with a `ref_type`, each element must reference an entity whose type matches `ref_type` exactly (URD419 per element). Diagnostics are emitted per failing element, not one aggregate diagnostic for the whole list. Elements are validated left to right; diagnostics are emitted in element order.

b. **Type mismatch.** If the value does not match the property type, emit URD401 (type mismatch).

c. **Enum value not in set.** If an enum value is not in the declared values list, emit URD402 (enum value not in declared set).

d. **Range violation.** If an integer or number value is outside the declared min/max range, emit URD418 (value outside declared range).

e. **Ref type mismatch.** If a `ref` property value references an entity whose type does not match the declared `ref_type`, emit URD419 (ref type mismatch). If the referenced entity's `type_symbol` is `null`, skip URD419 — the missing type is already reported by LINK (URD307) and cascading would produce a false positive.

### Reserved Container Keywords

`player` and `here` are reserved keywords in any container position — both containment checks (`? @entity in player`) and move destinations (`> move @entity -> here`). They are matched before location lookup. The collision surface is locations only, because entity references use the `@` sigil and are unambiguous.

**Annotation representation.** LINK represents keyword references distinctly from entity or location references in both container and destination positions. Two discriminators are used:

- **`container_kind`** (on `ContainmentCheck` annotations): `KeywordPlayer`, `KeywordHere`, `EntityRef` (resolved `EntitySymbol`), `LocationRef` (resolved `LocationSymbol`). LINK sets this during resolution. If the container token cannot be resolved to any of these, LINK sets `container_kind` to `null` and emits a LINK-phase diagnostic; VALIDATE will skip the condition via the null-annotation skip rule.
- **`destination_kind`** (on move effect annotations): `KeywordPlayer`, `KeywordHere`, `EntityRef` (resolved `EntitySymbol`), `LocationRef` (resolved `LocationSymbol`). Same discriminator model.

VALIDATE checks these discriminators to determine which rules apply — keywords skip trait checks entirely. This makes VALIDATE purely table-driven for both containment and move validation.

**Runtime player assumption.** The runtime always has a player concept, whether from an explicit `@player` entity declaration or from the implicit player the runtime creates at load time (see architecture brief, Player Entity section). Therefore `player` and `here` are always valid keywords in container positions — VALIDATE does not need to check for player existence before accepting these keywords.

**Keyword scope.** The reserved keywords `player` and `here` only apply in container positions (containment checks and move destinations). They do not apply in general identifier positions such as `world.start`, `world.entry`, exit destinations, or entity declarations. In those contexts, `player` and `here` are treated as ordinary identifiers and resolved normally by LINK.

If an author creates a location with ID `player` or `here`, the keyword takes priority in container positions. The location remains usable in exits, `world.start`, and `world.entry` resolution, but is effectively unreachable as a container reference in conditions and effects. No diagnostic is emitted by VALIDATE for this shadowing — if a warning is desired, it belongs in LINK (URD3xx range) as a location registration check, not in VALIDATE.

Steps 4b and 5b reference this rule.

### Step 4: Condition Validation

For each `Condition` and `OrConditionBlock` node in the AST, validate the condition expressions:

a. **`PropertyComparison`** — If the entity and property annotations are resolved:
   1. Check that the operator is compatible with the property type. `==` and `!=` are valid for all types. `<`, `>`, `<=`, `>=` are valid only for `integer` and `number`. If incompatible, emit URD420 (ordering operator on non-numeric type).
   2. Check that the comparison value matches the property type (same rules as property override validation). If mismatched, emit URD401 (reused code, different context).

b. **`ContainmentCheck`** — If the entity annotation is resolved:
   1. Check the container using the `container_kind` discriminator set by LINK:
      - `KeywordPlayer` or `KeywordHere` → always valid. No trait checks.
      - `LocationRef` → always valid. Locations are containers by definition.
      - `EntityRef` → if the container entity's `type_symbol` is `null`, skip the container trait check (no cascading — no URD422 can be emitted). This is intentional: LINK already emitted URD307 for the unknown type, and emitting URD422 would be a secondary diagnostic for the same root cause. Otherwise, check that the container entity's type has the `container` trait; if not, emit URD422 with `{context}` = `"containment check"`. For URD422, `{entity_id}` refers to the container entity, not the subject entity being checked in the condition.
      - `null` (unresolved container) → skip the entire condition. LINK already emitted a diagnostic for the unresolvable container token. No VALIDATE diagnostic is emitted for unresolved containers — container reference resolution is LINK's responsibility.

c. **`ExhaustionCheck`** — Verify the section name resolves to a `SectionSymbol` declared in the current file. If not, emit URD423 (section not file-local). LINK resolves section names; VALIDATE confirms file-locality.

### Step 5: Effect Validation

For each `Effect` node in the AST, validate by effect type:

a. **`set` effect** — If the target entity and property annotations are resolved:
   1. Check that the value matches the property type. If mismatched, emit URD401.
   2. If the operator is `+` or `-` (arithmetic set), check that the property type is `integer` or `number`. Integer and number only — boolean is not numeric. If not, emit URD424 (arithmetic on non-numeric property). Arithmetic set is a variant of the `set` effect where PARSE populates the `operator` field with `"+"` or `"-"` instead of `"="`. It is not a separate effect type.
   3. If the operator is `+` or `-`, check that the value is numeric. If not, emit URD401.

b. **`move` effect** — If the moved entity's annotation is resolved:
   1. If the moved entity's `type_symbol` is `null`, skip the portable trait check. This is intentional: LINK already emitted URD307 for the unknown type, and emitting URD425 would be a secondary diagnostic for the same root cause.
   2. Otherwise, check that the moved entity's type has the `portable` trait. If not, emit URD425 (missing portable trait on move target).
   3. Check the destination using the `destination_kind` discriminator set by LINK:
      - `KeywordPlayer` or `KeywordHere` → always valid. No trait checks.
      - `LocationRef` → always valid. Locations are containers by definition.
      - `EntityRef` → if the destination entity's `type_symbol` is `null`, skip the container trait check (no cascading — no URD422 can be emitted in this case, even though the destination entity is resolved). This is intentional: LINK already emitted URD307 for the unknown type, and emitting URD422 would be a secondary diagnostic for the same root cause. Otherwise, check that the destination entity's type has the `container` trait; if not, emit URD422 with `{context}` = `"move destination"`. For URD422, `{entity_id}` refers to the destination entity, not the moved entity.
      - `null` (unresolved destination) → skip the move effect. No VALIDATE diagnostic is emitted for unresolved destinations — destination resolution is LINK's responsibility.

c. **`reveal` effect** — If the entity and property annotations are resolved:
   1. Check that the property has `visibility: hidden`. If the property is not hidden, emit URD426 warning (reveal on non-hidden property).

d. **`destroy` effect** — If the entity annotation is resolved, no additional type checks are needed. Any entity can be destroyed.

### Step 6: Action Validation

Actions are validated in symbol table insertion order.

For each `ActionSymbol` in the symbol table:

a. **Mutual exclusion.** If both `target` and `target_type` are declared, emit URD406 (mutual exclusion violation).

b. **Target entity resolution.** If `target` is declared, verify it points to a resolved entity. If not, this was already reported by LINK — skip.

c. **Target type resolution.** If `target_type` is declared, verify it points to a resolved type. If not, this was already reported by LINK — skip.

### Step 7: Sequence and Phase Validation

Sequences are validated in symbol table insertion order.

For each `SequenceSymbol` in the symbol table:

**Design note:** Sequence phase references to actions and rules are stored as raw identifier strings on `PhaseSymbol`, not as resolved annotations. LINK intentionally does not resolve phase action and rule identifiers, even though it could, to keep LINK focused on content-level references and to keep phase validation grouped with other configuration checks in VALIDATE. VALIDATE performs the lookup against the symbol table and emits URD407/URD408 if the identifiers do not match declared symbols.

a. **Phase action references.** For each phase that declares `action` or `actions`, verify each reference points to a declared `ActionSymbol`. If not, emit URD407 (unknown action in phase).

b. **Phase rule references.** For each phase that declares `rule`, verify the reference points to a declared `RuleSymbol`. If not, emit URD408 (unknown rule in phase).

c. **Advance mode.** Verify the `advance` value is one of: `on_action`, `on_rule`, `on_condition`, `end`. If not, emit URD409 (invalid advance mode).

d. **Auto phase consistency.** If a phase is marked `auto: true`, it should not declare player-facing actions. If it does, emit URD427 warning (auto phase with player actions).

e. **Empty sequence.** If a sequence has zero phases, emit URD428 (empty sequence).

### Step 8: Nesting Depth Validation

For each `Choice` node and its nested content, check the `indent_level`:

a. **Depth 3 → warning severity.** Emit URD410 at warning severity.

b. **Depth 4+ → error severity.** Emit URD410 at error severity.

The catalog defines the canonical template. Severity is the only difference between depth 3 and depth 4+. Tooling may append a help hint for readability, but the diagnostic code, template, and severity rule in the catalog are the authoritative contract.

The `indent_level` field is set by PARSE. VALIDATE reads it directly. Depth 1 is a top-level choice within a section. Depth 2 is a sub-choice. Depth 3 is a sub-sub-choice (warn). Depth 4+ is an error.


## Diagnostic Catalog

All diagnostics emitted by VALIDATE are in the URD400–URD499 range.

**Canonical templates.** The catalog message template is the sole canonical string. Algorithm sections may paraphrase or summarise behaviour, but any quoted message text outside the catalog is illustrative and non-normative. If a discrepancy exists between an algorithm section and the catalog, the catalog wins. Every diagnostic also carries structured context fields (entity ID, property name, source span, file path) as machine-readable metadata alongside the human-readable message. These fields are always attached regardless of whether they appear in the template string.

### Errors

| Code | Message Template | Trigger | Recovery |
|------|-----------------|---------|----------|
| URD401 | *"Type mismatch: property '{property}' on entity '@{entity_id}' expects {expected_type} but got '{value}'."* | Value does not match declared property type. The `{entity_id}` field is always available: in entity overrides it is the owning entity; in conditions and effects it is the entity whose property is being checked. Property comparisons, set effects, and reveal effects always name an explicit entity receiver in v1 syntax (`@entity.property`), so `{entity_id}` is always known at the point URD401 is emitted. | Skip value. Continue. |
| URD402 | *"Enum value '{value}' is not valid for property '{property}' on entity '@{entity_id}'. Valid values: {values}."* | Enum value not in declared set. URD402 is emitted only for entity property overrides (Step 3). Enum mismatches in conditions and effects are reported via URD401, which is the general type-mismatch diagnostic. | Skip value. Continue. |
| URD404 | *"world.start references '{value}' but no location with that ID exists."* | Start location not found. | Continue. EMIT will not produce valid output. |
| URD405 | *"world.entry references '{value}' but no sequence with that ID exists."* | Entry sequence not found. | Continue. |
| URD406 | *"Action '{action_id}' declares both 'target' and 'target_type'. Declare one or neither."* | Mutual exclusion violation. | Continue. |
| URD407 | *"Phase '{phase_id}' in sequence '{sequence_id}' references unknown action '{action_name}'."* | Action ref in sequence phase not found. | Continue. |
| URD408 | *"Phase '{phase_id}' in sequence '{sequence_id}' references unknown rule '{rule_name}'."* | Rule ref in sequence phase not found. | Continue. |
| URD409 | *"Invalid advance mode '{mode}' in phase '{phase_id}'. Valid modes: on_action, on_rule, on_condition, end."* | Advance mode not one of the four valid values. | Continue. |
| URD412 | *"Player entity '@player' has type '{type_name}' which is missing required trait '{trait}'. The player type must have both 'mobile' and 'container' traits."* | Player missing mobile or container. | Continue. |
| URD413 | *"Default value '{value}' for property '{property}' on type '{type_name}' does not match the declared type '{prop_type}'."* | Type default fails type check. | Continue. |
| URD414 | *"Enum property '{property}' on type '{type_name}' declares an empty values list."* | Empty enum. | Continue. |
| URD415 | *"Property '{property}' on type '{type_name}' references unknown type '{ref_type}'."* | Ref type not found. | Continue. |
| URD416 | *"Property '{property}' on type '{type_name}' has min ({min}) greater than max ({max})."* | Inverted range. | Continue. |
| URD417 | *"Range constraints (min/max) are only valid on integer and number properties, not '{prop_type}'."* | Range on wrong type. | Continue. |
| URD418 | *"Value {value} for property '{property}' on entity '@{entity_id}' is outside the declared range [{min}, {max}]."* | Value outside declared range. | Continue. |
| URD419 | *"Property '{property}' on entity '@{entity_id}' requires a reference to type '{ref_type}' but '@{ref_entity}' has type '{actual_type}'."* | Ref type mismatch. | Continue. |
| URD420 | *"Operator '{op}' is not valid for property '{property}' of type '{prop_type}'. Use == or != for non-numeric types."* | Ordering operator on non-numeric type. | Continue. |
| URD422 | *"Entity '@{entity_id}' is used as a container in {context} but its type '{type_name}' does not have the 'container' trait."* | Entity without container trait used as container. `{context}` is `"containment check"` or `"move destination"`. | Continue. |
| URD423 | *"Exhaustion check references section '{name}' which is not declared in this file."* | Section not file-local. | Continue. |
| URD424 | *"Arithmetic operator '{op}' is not valid for property '{property}' of type '{prop_type}'. Arithmetic effects require integer or number properties."* | Arithmetic on non-numeric property. | Continue. |
| URD425 | *"Entity '@{entity_id}' cannot be moved because its type '{type_name}' does not have the 'portable' trait."* | Missing portable trait on move target. | Continue. |
| URD428 | *"Sequence '{sequence_id}' declares no phases."* | Empty sequence. | Continue. |

### Mixed Severity

| Code | Message Template | Trigger | Severity Rule | Recovery |
|------|-----------------|---------|---------------|----------|
| URD410 | *"Nesting depth {depth} at line {line}."* | Choice nesting at depth 3+. | Depth 3 = warning. Depth 4+ = error. | Continue. |

### Warnings

| Code | Message Template | Trigger |
|------|-----------------|---------|
| URD411 | *"The 'urd' field is set automatically by the compiler. Author value will be overridden."* | Author set `urd` key in `WorldBlock`. |
| URD426 | *"Property '{property}' on entity '@{entity_id}' is not hidden. Reveal has no effect."* | Reveal on non-hidden property. |
| URD427 | *"Phase '{phase_id}' is auto-advancing but declares player actions. The actions will not be available."* | `auto: true` with player actions. |


## Error Recovery

VALIDATE follows the architecture brief's "skip on null, continue on error" principle:

1. **Null annotation → skip.** If any annotation that a check depends on is `null`, skip that entire check. Do not emit secondary diagnostics.
2. **Type mismatch → continue.** Report the mismatch and move on to the next property/entity/condition/effect.
3. **Structural error → continue.** Report the error and move on to the next construct.
4. **ErrorNode → skip silently.** Do not validate parse-damaged content.
5. **Duplicated symbols → validate the canonical entry only.** LINK's canonical map stores the first declaration; VALIDATE checks it normally. Duplicates in the duplicates list are not re-validated.

The goal is maximum diagnostic density per compilation run without false positives from cascading failures.


## Acceptance Criteria

### Unit Tests: Property Type Checking

| Test | Setup | Expected |
|------|-------|----------|
| Boolean valid | `@door: LockedDoor { locked: true }` | No errors. |
| Boolean invalid | `@door: LockedDoor { locked: "yes" }` | URD401. |
| Integer valid | `@guard: Guard { trust: 50 }` (range 0–100) | No errors. |
| Integer out of range | `@guard: Guard { trust: 150 }` (max 100) | URD418. |
| Number valid | `@item: Item { weight: 3.5 }` | No errors. |
| Enum valid | `@guard: Guard { mood: neutral }` | No errors. |
| Enum invalid | `@guard: Guard { mood: angry }` (not in values) | URD402. |
| Ref valid | `@door: LockedDoor { requires: @rusty_key }`, `@rusty_key: Key` | No errors. |
| Ref type mismatch | `@door: LockedDoor { requires: @guard }`, requires `ref(Key)` | URD419. |
| String valid | `@key: Key { name: "Rusty Key" }` | No errors. |
| Default invalid | Type declares `mood: enum = "angry"`, `angry` not in values. | URD413. |
| Empty enum | Type declares `status: enum` with `values: []`. | URD414. |
| Range inverted | Type declares `trust: integer` with `min: 100, max: 0`. | URD416. |
| Range on string | Type declares `name: string` with `min: 0`. | URD417. |

### Unit Tests: Condition Validation

| Test | Setup | Expected |
|------|-------|----------|
| Property comparison valid | `? @guard.mood == neutral` | No errors. |
| Ordering on enum | `? @guard.mood > neutral` | URD420. |
| Value type mismatch | `? @guard.trust == "high"` (trust is integer) | URD401. |
| Containment valid | `? @key in player` | No errors. |
| Containment with here | `? @key in here` | No errors. |
| Container without trait | `? @key in @door` (Door has no container trait) | URD422. |
| Invalid container ref | `? @key in nowhere` (not player/here/entity/location) | LINK emits diagnostic. `container_kind` is `null`. VALIDATE skips condition. |
| Keyword shadows location | Location with ID `player` exists. `? @key in player`. | Resolves to keyword `player`, not the location. No errors. Location still usable in exits, world.start, and world.entry. |
| Exhaustion valid | `? topics.exhausted`, `== topics` in same file. | No errors. |
| Exhaustion cross-file | `? topics.exhausted`, `== topics` in different file. | URD423. |

### Unit Tests: Effect Validation

| Test | Setup | Expected |
|------|-------|----------|
| Set valid | `> @guard.mood = neutral` | No errors. |
| Set type mismatch | `> @guard.mood = 42` (mood is enum) | URD401. |
| Arithmetic on integer | `> @guard.trust + 10` | No errors. |
| Arithmetic on enum | `> @guard.mood + 1` | URD424. |
| Move valid | `> move @key -> player` (Key is portable) | No errors. |
| Move non-portable | `> move @door -> player` (Door is not portable) | URD425. |
| Move to non-container | `> move @key -> @sword` (Sword has no container trait) | URD422. |
| Reveal hidden | `> reveal @door.prize` (prize is hidden) | No errors. |
| Reveal visible | `> reveal @door.state` (state is not hidden) | URD426 (warning). |
| Destroy entity | `> destroy @key` | No errors. |
| Keyword shadows location in move | Location with ID `player` exists. `> move @key -> player`. | Resolves to keyword `player` destination. No errors. Location still usable in exits, world.start, and world.entry. |
| Destination kind is KeywordPlayer despite location | Location with ID `player` exists. `> move @key -> player`. | LINK sets `destination_kind` = `KeywordPlayer`, not `LocationRef`. VALIDATE skips trait checks. No URD422. |
| Destination kind is KeywordHere despite location | Location with ID `here` exists. `> move @key -> here`. | LINK sets `destination_kind` = `KeywordHere`, not `LocationRef`. VALIDATE skips trait checks. No URD422. |

### Unit Tests: Structural Constraints

| Test | Setup | Expected |
|------|-------|----------|
| World start valid | `start: cell`, location `cell` exists. | No errors. |
| World start with shadowed location | Location with ID `player` exists. `world.start: player`. | Resolves to `LocationRef` (not keyword). No errors. Keywords only apply in container positions. |
| World start invalid | `start: dungeon`, no location `dungeon`. | URD404. |
| World entry valid | `entry: game`, sequence `game` exists. | No errors. |
| World entry invalid | `entry: tutorial`, no sequence `tutorial`. | URD405. |
| No world entry | No `entry` in world block. | No errors (freeform). |
| Action mutual exclusion | Action declares both `target` and `target_type`. | URD406. |
| Phase action valid | Phase references declared action. | No errors. |
| Phase action invalid | Phase references undeclared action. | URD407. |
| Phase rule valid | Phase references declared rule. | No errors. |
| Phase rule invalid | Phase references undeclared rule. | URD408. |
| Advance mode valid | `advance: on_action` | No errors. |
| Advance mode invalid | `advance: immediate` | URD409. |
| Auto phase with actions | `auto: true` with `actions: [choose]`. | URD427 (warning). |
| Empty sequence | Sequence with no phases. | URD428. |
| Nesting depth 2 | Two-level choice nesting. | No errors. |
| Nesting depth 3 | Three-level choice nesting. | URD410 (warning). |
| Nesting depth 4 | Four-level choice nesting. | URD410 (error). |
| Player valid traits | `@player: Hero`, Hero has mobile + container. | No errors. |
| Player missing trait | `@player: Guard`, Guard has mobile but not container. | URD412. |
| Urd override | Author sets `urd: "2"` in `WorldBlock` (frontmatter `world:` block). | URD411 (warning). |
| Ref type unknown | Property declares `ref_type: UnknownType`. | URD415. |

### Unit Tests: Skip Rule (No Cascading)

| Test | Setup | Expected |
|------|-------|----------|
| Unresolved entity in condition | `? @missing.trust == 50`, `@missing` unresolved. | Zero VALIDATE errors. LINK URD301 is the only diagnostic. |
| Unresolved type on entity | `@guard: UnknownType`, type not found. | Zero VALIDATE property errors. LINK URD307 is the only diagnostic. |
| Unresolved property in effect | `> @guard.unknown = 5`, property not found. | Zero VALIDATE errors. LINK URD308 is the only diagnostic. |
| ErrorNode skipped | AST contains ErrorNode with invalid conditions. | Zero VALIDATE errors from the ErrorNode. |

### Integration Tests

| Test | Setup | Expected |
|------|-------|----------|
| Two Room Key Puzzle | Full single-file world from architecture brief walkthrough. | Zero errors. All types, entities, conditions, and effects valid. |
| Monty Hall | Full single-file world with sequences, hidden state, and rules. | Zero errors. Advance modes, action refs, and rule refs all valid. |
| Multi-file with imports | Entry imports types file and location file. Cross-file entity validation. | Zero errors. Ref types resolve cross-file. Traits checked correctly. |
| Maximum errors | World with every error type represented. | One diagnostic per error. No cascading. Diagnostics in deterministic order. |


## Relationship to Other Phases

### From LINK

VALIDATE receives the `LinkedWorld` and needs:

1. **The `SymbolTable`** — to look up types, properties, traits, defaults, ranges, enum values, actions, rules, and sequences.
2. **Annotated ASTs** — where annotation slots are either populated (resolved) or `null` (unresolved). VALIDATE reads annotations to perform type checking and skips `null` annotations.
3. **The `DependencyGraph`** — for deterministic file ordering during validation and for file-locality checks (exhaustion conditions).
4. **LINK's guarantee that scope is enforced** — so VALIDATE does not need to re-check visibility. If an annotation is populated, the reference is visible.
5. **LINK's canonical map model** — VALIDATE checks only the canonical (first) declaration for each name. Duplicates list entries are not re-validated.

### To EMIT

EMIT receives the same `ValidatedWorld` (structurally identical to `LinkedWorld`) and needs:

1. **The diagnostic collector** — to determine whether to produce JSON. If any Error-severity diagnostics exist, EMIT produces `null`.
2. **The symbol table and annotated ASTs** — for JSON generation. VALIDATE has not modified them.
3. **Confidence that all semantic constraints have been checked** — so EMIT can focus on structural output without re-validating values.

*End of Brief*
