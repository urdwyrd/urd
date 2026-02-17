# URD — Compiler Phase 3: LINK

*Declaration collection, reference resolution, ID derivation, and scope enforcement*

February 2026 | Engineering Phase

`CompilationUnit → LINK → LinkedWorld (SymbolTable + annotated ASTs)`

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:**
**Status:**

### What was done

-

### What changed from the brief

-

---

> **Document status: BRIEF** — Defines the LINK phase of the Urd compiler. LINK is the third phase of the five-phase pipeline. It takes the `CompilationUnit` produced by IMPORT (a dependency graph and topologically sorted `FileAST` list), populates a global symbol table with all declarations, derives compiled IDs, resolves every cross-file and intra-file reference, and produces annotated ASTs ready for VALIDATE.

> **Dependencies:** This brief builds on the Compiler Architecture Brief (symbol table structure, annotation model, visible scope, ID derivation rules, namespace rules, duplicate detection, error recovery) and the IMPORT Phase Brief (CompilationUnit structure, topological ordering, path normalisation). Both are required reading.


## Purpose

LINK bridges the gap between syntax (what PARSE and IMPORT produce) and semantics (what VALIDATE and EMIT need). It is the phase where names become symbols and references become resolved pointers.

LINK has exactly four jobs:

1. **Collect declarations.** Walk every `FileAST` in topological order and register types, entities, locations, sections, actions, rules, and sequences in the global symbol table.
2. **Derive compiled IDs.** Compute section IDs (`file_stem/section_name`), location IDs (`slugify(display_name)`), choice IDs (`section_id/slugify(label)`), sequence IDs, and phase IDs from source names. Entity IDs and type names are used as declared.
3. **Resolve references.** Walk every `FileAST` again and resolve entity references (`@name`), type references, property accesses (`@entity.property`), jump targets (`-> name`), exit destinations, and containment lists. Populate annotation slots on AST nodes and resolved fields on symbol table entries.
4. **Enforce scope.** Ensure every resolved reference is visible to the file that contains it, per the non-transitive import rules.

### What LINK Does

- Registers `TypeSymbol`s from `TypeDef` AST nodes, including property schemas, traits, defaults, and visibility modifiers.
- Registers `EntitySymbol`s from `EntityDecl` AST nodes, recording type name and property overrides.
- Registers `LocationSymbol`s from `LocationHeading` AST nodes, deriving IDs by slugification.
- Registers `SectionSymbol`s from `SectionLabel` AST nodes, deriving IDs from file stem and section name.
- Registers `ChoiceSymbol`s nested within sections, deriving IDs by slugification.
- Registers `ActionSymbol`s, `RuleSymbol`s, `SequenceSymbol`s, and `PhaseSymbol`s from their respective frontmatter and heading nodes.
- Detects and reports duplicate declarations within each namespace (URD302 for entities, URD303 for types, URD304 for locations, URD305 for sections).
- Resolves `@entity` references to `EntitySymbol`s and populates annotation slots.
- Resolves `EntityDecl.type_name` to `TypeSymbol` and stores it in `EntitySymbol.type_symbol`.
- Resolves property accesses (`@entity.property`) to `PropertySymbol`s via the entity's resolved type.
- Resolves jump targets (`-> name`) using the normative priority rule: section first, then exit, then error.
- Resolves exit destinations to `LocationSymbol`s by slugifying the destination text and looking up the result.
- Resolves entity presence lists (`[@entity_a, @entity_b]`) to entity symbols and records them in `LocationSymbol.contains`.
- Resolves `world.start` and `world.entry` to their respective symbols for downstream validation.
- Enforces visible scope during resolution: a reference in file F can only resolve to a symbol declared in F or in a file F directly imports.
- Reports unresolved references (URD301) with suggestions when a close match exists.
- Emits a warning when a section name shadows an exit name in the same file (URD310).
- Skips `ErrorNode` entries silently — damaged parse output does not produce LINK diagnostics.
- Passes all non-reference AST content through unchanged.

### What LINK Does Not Do

- Validate types or values. LINK resolves `@cell_door.locked` to a `PropertySymbol` but does not check whether `true` is a valid value for a boolean property. That is VALIDATE's job.
- Check semantic constraints. LINK does not enforce that an entity with `ref(Key)` actually references a Key, or that `move` targets have the `portable` trait. VALIDATE handles all constraint checking.
- Generate JSON. LINK populates the data structures that EMIT reads, but produces no output artefact.
- Modify AST structure. LINK annotates nodes (filling `null` annotation slots with resolved symbols) but does not add, remove, or reorder nodes.


## Interface Contract

### Input

```
link(compilation_unit: CompilationUnit, diagnostics: DiagnosticCollector) → LinkedWorld
```

- `compilation_unit`: The `CompilationUnit` from IMPORT, containing the `DependencyGraph` and `ordered_asts` (topologically sorted, entry file last).
- `diagnostics`: The shared diagnostic collector. LINK appends to it.

LINK does not touch the filesystem. It operates entirely on in-memory ASTs and the dependency graph.

### Output

```
LinkedWorld {
  symbol_table: SymbolTable,
  annotated_asts: FileAST[],   // same instances as input, with annotation slots populated
  graph: DependencyGraph,      // passed through unchanged from IMPORT
}
```

LINK always returns a `LinkedWorld`. Even if every reference is unresolved, the symbol table and annotated ASTs are still useful for VALIDATE (which will skip unresolved constructs) and for tooling (which can report partial resolution).

### Guarantees

After LINK completes, the following properties hold:

1. **Every declaration is registered.** All types, entities, locations, sections, choices, actions, rules, and sequences from every `FileAST` in the compilation unit are present in the symbol table, in insertion order (topological file order, then declaration order within each file).
2. **Duplicates are detected and reported.** Any symbol name that appears more than once in its namespace has been reported (URD302–URD305). The first declaration is canonical and remains resolvable. The second is recorded for diagnostics but does not replace the first.
3. **Every resolvable reference is annotated.** If a reference can be resolved given the visible scope and the symbol table contents, its annotation slot is populated with the resolved symbol.
4. **Every unresolvable reference is reported.** If a reference cannot be resolved, its annotation slot remains `null` and a diagnostic (URD301 or specific sub-code) has been emitted.
5. **Scope is enforced.** No annotation points to a symbol outside the referencing file's visible scope, even if the symbol exists in the global table.
6. **Compiled IDs are deterministic.** The same source always produces the same IDs. ID derivation follows the architecture brief's rules exactly.
7. **The symbol table insertion order is deterministic.** Same input files always produce the same symbol table ordering, which is the canonical ordering for EMIT's JSON output.
8. **AST structure is unchanged.** LINK does not add, remove, or reorder nodes. It only populates annotation slots and resolved fields on symbol table entries.
9. **ErrorNodes are silently skipped.** No diagnostics are emitted for constructs inside `ErrorNode` entries.
10. **Jump resolution follows the normative priority rule.** `-> name` resolves to section first, exit second, error third. Shadowing is warned (URD310).


## The Algorithm

LINK performs two sequential passes over all `FileAST`s: collection, then resolution. Both passes iterate in topological order (the `ordered_asts` list from IMPORT).

### Why Two Passes

Forward references must work. A section can jump to another section declared later in the same file. An entity's type may be declared in the same file after the entity. The collection pass ensures every name is known before the resolution pass begins.

### Pass 1: Collection

For each `FileAST` in topological order:

1. **Compute file metadata.** Derive the file stem from `file_ast.file_path` (strip directory and `.urd.md` extension). Compute the visible scope: `{ file_ast.file_path } ∪ { direct imports from dependency graph }`. Store both on the file for use during resolution.

2. **Walk the frontmatter.** For each declaration node in `file_ast.frontmatter.entries`:

   a. **`TypeDef`** → Create a `TypeSymbol` with name, traits, and property map. Register in `symbol_table.types`. If the name already exists, emit URD303 and record the new entry in the duplicates list.

   b. **`EntityDecl`** → Create an `EntitySymbol` with id (`@name`), type_name, and property overrides. Register in `symbol_table.entities`. If the id already exists, emit URD302 and record the new entry in the duplicates list.

   c. **`WorldBlock`** → Store `world.start` and `world.entry` values for resolution in pass 2. Not registered as symbols — these are configuration, not declarations.

   d. **`ActionDef`** → Create an `ActionSymbol`. Register in `symbol_table.actions`.

   e. **`RuleDef`** → Create a `RuleSymbol`. Register in `symbol_table.rules`.

   f. **`ImportDecl`** → Skip. Already processed by IMPORT.

3. **Walk the narrative content.** For each content node in `file_ast.content`:

   a. **`LocationHeading`** → Derive `location_id = slugify(display_name)`. Create a `LocationSymbol` with id, display_name, empty exits map, and empty contains list. Register in `symbol_table.locations`. If the id already exists, emit URD304 and record the new entry in the duplicates list. Set the current location context for subsequent exit and containment processing.

   b. **`SectionLabel`** → Derive `section_id = file_stem + "/" + section_name`. Create a `SectionSymbol` with local_name, compiled_id, file_stem, and empty choices list. Register in `symbol_table.sections` keyed by `compiled_id` (which is globally unique by construction). Additionally check that `local_name` is unique within the current file — if the same `local_name` appears twice in one file, emit URD305. Because the compiled ID is derived from the local name, a duplicate local name also produces a duplicate compiled ID. The second section is recorded in the sections namespace's duplicates list (same as other namespaces). The first section remains canonical in the global map and is the target for `-> name` jumps.

   c. **`SequenceHeading`** → Derive `sequence_id = slugify(display_name)`. Create a `SequenceSymbol`. Register in `symbol_table.sequences`.

   d. **`PhaseHeading`** → Derive `phase_id = slugify(display_name)`. Create a `PhaseSymbol`. Attach to the enclosing `SequenceSymbol.phases`.

   e. **`Choice`** → Derive `choice_id = parent_section_id + "/" + slugify(label)`. Create a `ChoiceSymbol` with label, compiled_id, and sticky flag. Attach to the enclosing `SectionSymbol.choices`. If two choices in the same section produce identical slugified IDs, emit URD306. Additionally, create a corresponding `ActionSymbol` with id equal to the choice's `compiled_id`, `target` set from the choice's entity target (if present), `target_type` set from the choice's type target (if present), and `declared_in` set to the choice's span. Register the `ActionSymbol` in `symbol_table.actions`. Nested sub-choices also generate `ActionSymbol`s — all choices in a section produce actions at the same level, regardless of nesting depth.

   f. **`ExitDeclaration`** → Record the exit direction and raw destination text on the current `LocationSymbol.exits` map. The exit direction (e.g., `north`, `south`) is stored and compared as a raw, case-sensitive string — it is not slugified. This means `-> exit:north` and `-> exit:North` are different lookups. Exit names in the exits map, jump target matching, and shadowing comparison all use the raw direction string. Create an `ExitSymbol` with direction, destination (unresolved), and references to any child `Condition` and `BlockedMessage` nodes. Destination resolution happens in pass 2.

   g. **`EntityPresence`** → Record the raw entity reference strings on the current `LocationSymbol`. Resolution happens in pass 2.

   h. **`ErrorNode`** → Skip silently. Do not attempt to extract declarations.

   i. **All other content nodes** (`Prose`, `EntitySpeech`, `StageDirection`, `Condition`, `Effect`, `Jump`, `BlockedMessage`, `OrConditionBlock`) → Skip during collection. These contain references but not declarations. Processed in pass 2.

### Pass 2: Resolution

After collection, every declared name is in the symbol table. Resolution walks the ASTs again, this time filling annotation slots and resolved fields.

For each `FileAST` in topological order:

1. **Load the file's visible scope** (computed during collection).

2. **Resolve frontmatter references:**

   a. **`EntityDecl.type_name`** → Look up the type name in `symbol_table.types`. If found and visible, store the `TypeSymbol` reference in `EntitySymbol.type_symbol`. If not found, emit URD307: *"Unknown type '{type_name}' for entity '@{entity_id}'."* If found but not visible (declared in a non-imported file), emit URD301 with a note that the type exists but is not imported.

   b. **Entity property overrides** → For each override, if `EntitySymbol.type_symbol` is resolved, verify the property name exists on the type. If it does not, emit URD308: *"Property '{property}' does not exist on type '{type_name}'."* Store the resolved `PropertySymbol` reference. (Note: value type checking is VALIDATE's job — LINK only checks that the property name exists.)

   c. **`world.start`** → Look up the value as a location ID in `symbol_table.locations`. Store the resolved `LocationSymbol` reference for VALIDATE to check. If not found, do not emit here — VALIDATE reports it (URD404).

   d. **`world.entry`** → Look up the value as a sequence ID in `symbol_table.sequences`. Store the resolved reference. VALIDATE reports missing entries.

3. **Resolve narrative content references.** Walk each content node recursively:

   a. **Entity references (`@name`)** in any context (speech, stage direction, condition, effect, containment, choice target) → Look up `name` in `symbol_table.entities`. If found and visible, populate the annotation slot with the resolved `EntitySymbol`. If not found, emit URD301: *"Unresolved entity reference '@{name}'."* If a close match exists (edit distance ≤ 2), add a suggestion: *"Did you mean '@{suggestion}'?"*

   b. **Property access (`@entity.property`)** → First resolve the entity (step 3a). If the entity resolved and has a resolved type, look up the property on `TypeSymbol.properties`. If found, populate the property annotation. If the property is not found, emit URD308. If the entity did not resolve, do not emit a property error (no cascading).

   c. **Jump targets (`-> name`)** → Apply the normative priority rule:
      1. Look up `name` in sections declared in the current file (not the global table — section jumps are file-local in v1).
      2. If no section match and there is an active location context, look up `name` in the current location's exits map.
      3. If no match at all, emit URD309: *"Unresolved jump target '{name}'. No section or exit with this name exists in scope."*
      4. If both a section and an exit match, resolve to the section and emit URD310 (warning): *"Section '{name}' shadows exit '{name}' in this location. Use -> exit:{name} to target the exit."*

   **Location context rule.** A location context is established by a `LocationHeading` and remains active until the next `LocationHeading` or end of file. Jump targets and exit references that appear before any `LocationHeading` have no location context — exit lookup is not available and only section resolution applies. Exit declarations (`ExitDeclaration`) and entity presence lists (`EntityPresence`) encountered before any `LocationHeading` emit URD314: *"Exit construct outside of a location context."* The node is skipped.

   d. **Explicit exit jumps (`-> exit:name`)** → If there is no active location context, emit URD314: *"Exit construct outside of a location context."* If there is an active location context, look up `name` in the current location's exits map only. If not found, emit URD311: *"Unresolved exit reference 'exit:{name}'. No exit with this name exists in the current location."*

   e. **Exit destinations** → For each `ExitSymbol` in the current location, slugify the destination text and look up the result in `symbol_table.locations`. If found and visible, store the `LocationSymbol` in `ExitSymbol.resolved_destination`. If not found, emit URD312: *"Exit destination '{destination}' does not resolve to any known location."* **Scope applies to exit destinations.** A location declared in file B is only visible to exits in file A if A imports B. This is consistent with the non-transitive import model — exits are world navigation, but the compiler still requires explicit imports so that files remain self-contained units with declared dependencies.

   f. **Entity presence lists (`[@a, @b]`)** → For each entity reference, resolve per step 3a. Add the resolved entity ID to `LocationSymbol.contains`.

   g. **Condition expressions** → Resolve entity references and property accesses within `ConditionExpr` nodes (PropertyComparison, ContainmentCheck, ExhaustionCheck). For `ExhaustionCheck`, resolve the section name to a `SectionSymbol` in the current file.

   h. **Effect expressions** → Resolve entity references and property accesses within `Effect` nodes. For `move` effects, resolve the destination entity or container. For `destroy` effects, resolve the target entity.

   i. **`ErrorNode`** → Skip silently.

### Resolution Scope Enforcement

Every symbol lookup during pass 2 is scope-checked. The lookup function is conceptually:

```
resolve(name, namespace, current_file, symbol_table, graph) → Symbol | null:
  symbol = symbol_table[namespace].get(name)
  if symbol is null:
    return null                    // not found anywhere
  if symbol.declared_in.file not in visible_scope(current_file):
    return null                    // exists but not visible
  return symbol                    // first (canonical) declaration resolves normally
```

The first declaration is always the canonical one and is stored in the canonical map. Duplicate entries are stored in a separate duplicates list per namespace and are never returned by lookup. The duplicates list exists solely for diagnostic reporting (both declaration sites in the error message).

The `visible_scope(F)` is computed from the dependency graph:

```
visible_scope(F) = { F } ∪ { G : edge(F, G) exists in graph }
```

When a symbol exists in the global table but is not visible, LINK's diagnostic should include a hint: *"'{name}' is declared in {file} but {file} is not imported by {current_file}."* This helps authors fix missing imports.


## ID Derivation

LINK is responsible for computing all compiled IDs. The rules are defined in the architecture brief's ID Derivation Rules section. LINK applies them as follows:

### Entity IDs

Used as declared. `@rusty_key` → `"rusty_key"`. No transformation. Registered during collection.

### Section IDs

```
section_id = file_stem + "/" + section_name
```

Example: `== topics` in `tavern.urd.md` → `"tavern/topics"`. File stem is guaranteed unique by IMPORT.

### Choice IDs

```
choice_id = section_id + "/" + slugify(label)
```

Example: "Ask about the harbor" in `tavern/topics` → `"tavern/topics/ask-about-the-harbor"`.

### Location IDs

```
location_id = slugify(display_name)
```

Example: `# The Rusty Anchor` → `"the-rusty-anchor"`.

### Sequence and Phase IDs

```
sequence_id = slugify(display_name)
phase_id = slugify(display_name)
```

Both derived from `##` and `###` heading text respectively. Phases must be unique within their parent sequence; sequences must be unique within the compilation unit.

### Slugification Rules

All slugification follows the same algorithm:

1. Lowercase the input.
2. Replace spaces with hyphens.
3. Strip characters that are not alphanumeric or hyphens.
4. Collapse consecutive hyphens into a single hyphen.
5. Trim leading and trailing hyphens.

The result must be non-empty. If slugification produces an empty string, emit URD313: *"Heading '{display_name}' produces an empty ID after slugification."* The symbol is not registered and the source node is skipped.


## Duplicate Detection

The symbol table uses two structures per namespace: a **canonical map** (keyed by name, stores the first declaration) and a **duplicates list** (stores subsequent declarations for diagnostic purposes only). When LINK attempts to register a name that already exists in the canonical map, it does not overwrite — it appends the new declaration to the duplicates list and emits the diagnostic. Lookup always queries the canonical map, so the first declaration is the one that resolves.

1. **First declaration wins and remains resolvable.** The first symbol registered (per topological order and declaration order) is the canonical one. References to this name resolve to the first declaration normally.
2. **Second declaration is recorded as a duplicate.** The duplicate entry is stored in the duplicates list, not the canonical map. Both declaration sites are reported in the diagnostic. The second entry does not replace the first.
3. **Only the duplicate entry is excluded from resolution.** During resolution, references resolve to the first (canonical) declaration via the canonical map. The duplicate entry in the duplicates list is never returned by lookup. This ensures one duplicate does not break every reference to that name across the compilation unit.

### Namespace Isolation

Names are unique within each category but allowed to collide across categories:

- A location named `cell` and an entity named `cell` coexist (different namespaces, different sigils).
- A section named `topics` and an exit named `topics` coexist but trigger a shadowing warning (URD310) when a jump is ambiguous.

**Entity IDs and type names are in separate namespaces.** `@guard` (entity) and `Guard` (type) do not conflict.

### Diagnostic Templates

| Code | Message Template | Trigger |
|------|-----------------|---------|
| URD302 | *"Duplicate entity ID '@{id}' declared in {file_a}:{line_a} and {file_b}:{line_b}."* | Same `@name` in two files. |
| URD303 | *"Duplicate type name '{name}' declared in {file_a}:{line_a} and {file_b}:{line_b}."* | Same type name in two files. |
| URD304 | *"Duplicate location ID '{id}' — locations '{display_a}' and '{display_b}' both slugify to '{id}'."* | Two `#` headings produce the same slugified ID. |
| URD305 | *"Duplicate section name '{name}' in {file}. Section names must be unique within a file."* | Same `== name` appears twice in one file. |
| URD306 | *"Duplicate choice ID '{id}' in section '{section_id}'. Choices '{label_a}' and '{label_b}' produce the same slugified ID."* | Two choices in the same section slugify to the same ID. |


## Diagnostic Catalog

All diagnostics emitted by LINK are in the URD300–URD399 range.

### Errors

| Code | Message Template | Trigger | Recovery |
|------|-----------------|---------|----------|
| URD301 | *"Unresolved entity reference '@{name}'."* | Entity not found in visible scope. | Annotation set to `null`. |
| URD302 | *"Duplicate entity ID '@{id}'."* | Same entity ID registered twice. | Second entry recorded in duplicates list. |
| URD303 | *"Duplicate type name '{name}'."* | Same type name registered twice. | Second entry recorded in duplicates list. |
| URD304 | *"Duplicate location ID '{id}'."* | Two headings slugify to the same ID. | Second entry recorded in duplicates list. |
| URD305 | *"Duplicate section name '{name}' in {file}."* | Same section name in one file. | Second entry recorded in duplicates list. |
| URD306 | *"Duplicate choice ID '{id}' in section '{section_id}'."* | Two choice labels slugify to the same ID. | Second entry recorded in duplicates list. |
| URD307 | *"Unknown type '{type_name}' for entity '@{entity_id}'."* | Entity references a type that does not exist or is not visible. | `EntitySymbol.type_symbol` set to `null`. |
| URD308 | *"Property '{property}' does not exist on type '{type_name}'."* | Property access on a type that does not declare the property. | Property annotation set to `null`. |
| URD309 | *"Unresolved jump target '{name}'."* | Neither section nor exit matches in scope. | Jump annotation set to `null`. |
| URD311 | *"Unresolved exit reference 'exit:{name}'."* | Explicit exit reference does not match any exit in the current location. | Jump annotation set to `null`. |
| URD312 | *"Exit destination '{destination}' does not resolve to any known location."* | Slugified destination does not match a registered location. | `ExitSymbol.resolved_destination` set to `null`. |
| URD313 | *"Heading '{display_name}' produces an empty ID after slugification."* | Slugification of a heading yields an empty string. | Symbol not registered. Source node skipped. |
| URD314 | *"Exit construct outside of a location context."* | `ExitDeclaration`, `EntityPresence`, or explicit exit jump (`-> exit:name`) appears before any `LocationHeading` in the file. | Node skipped. |

### Warnings

| Code | Message Template | Trigger |
|------|-----------------|---------|
| URD310 | *"Section '{name}' shadows exit '{name}' in this location. Use -> exit:{name} to target the exit."* | A `-> name` jump resolves to a section when an exit with the same name exists. |

### Informational

| Code | Message Template | Trigger |
|------|-----------------|---------|
| URD350 | *"Entity '@{name}' is declared but never referenced."* | No reference to the entity exists in any file. |
| URD351 | *"Section '{name}' is declared but never targeted by a jump."* | No `-> name` targets this section. |

Informational diagnostics are not emitted by default. They are enabled by a compiler flag and are advisory only. **When enabled**, LINK gathers the required reference-tracking data during the resolution pass (pass 2). When disabled, no reference-tracking bookkeeping is performed — there is no cost to the default path.

### Suggestion Hints

For URD301 and URD307, LINK computes suggestions when a close match exists:

- **Entity references:** If `@gaurd` is not found but `@guard` exists, add *"Did you mean '@guard'?"*
- **Type names:** If `GuardType` is not found but `Guard` exists, add *"Did you mean 'Guard'?"*

Suggestions use edit distance ≤ 2 as the threshold. If multiple candidates are within range, include the one with the smallest edit distance. If tied on distance, use the first in symbol table insertion order (deterministic).


## Error Recovery

LINK follows the architecture brief's "mark and continue" principle:

1. **Unresolved references** → Set annotation to `null`. Continue processing remaining references. VALIDATE will silently skip constructs with `null` annotations (no cascading errors).
2. **Duplicate declarations** → Keep the first in the canonical map, record the second in the duplicates list. Continue collecting remaining declarations.
3. **Duplicate declarations during resolution** → The first declaration remains resolvable. References resolve normally to the canonical (first) symbol. The duplicate diagnostic (URD302–URD305) is the sole record of the conflict.
4. **ErrorNode in AST** → Skip silently. Do not attempt to resolve references inside parse-damaged content.
5. **Missing type on entity** → Set `type_symbol` to `null`. Property resolution on that entity will be skipped (no cascading).
6. **Missing property on type** → Set property annotation to `null`. Value checking will be skipped by VALIDATE.

The goal is: one root cause, one diagnostic. A missing type should not produce N errors for N property accesses on entities of that type.


## Acceptance Criteria

### Unit Tests: Collection

| Test | Setup | Expected |
|------|-------|----------|
| Single type | One `TypeDef` with properties and traits. | `TypeSymbol` registered with all properties. |
| Single entity | One `EntityDecl` with type and overrides. | `EntitySymbol` registered. `type_symbol` null (resolved in pass 2). |
| Location from heading | `# The Tavern` | `LocationSymbol` with id `"the-tavern"`. |
| Section with choices | `== topics` with two choices. | `SectionSymbol` with compiled_id. Two `ChoiceSymbol`s with slugified IDs. |
| Duplicate entity | Two files declare `@guard`. | URD302. First wins. Second recorded in duplicates list. |
| Duplicate type | Two files declare `Guard`. | URD303. First wins. Second recorded in duplicates list. |
| Duplicate location | `# Cell` and `# cell` both slugify to `"cell"`. | URD304. First wins. |
| Duplicate section in same file | Two `== topics` in one file. | URD305. |
| Duplicate choice slugs | "Ask why" and "Ask Why" in same section. | URD306. |
| ErrorNode skipped | AST contains an `ErrorNode` among valid nodes. | ErrorNode ignored. Surrounding valid declarations registered. |
| Topological order | File A imports B. B declares `Guard` type. A declares `@guard: Guard`. | `Guard` registered before `@guard`. |

### Unit Tests: Choice-to-Action Collection

| Test | Setup | Expected |
|------|-------|----------|
| Choice generates action | `== topics` with choice "Ask about the harbor". | `ActionSymbol` registered with id `"tavern/topics/ask-about-the-harbor"`. |
| Choice with entity target | `* Use key -> @cell_door` in section. | `ActionSymbol` with `target: "cell_door"`, `target_type: null`. |
| Choice with type target | `* Pick a door -> any Door` in section. | `ActionSymbol` with `target: null`, `target_type: "Door"`. |
| Choice with no target | `* Ask about the weather` (inline content only). | `ActionSymbol` with `target: null`, `target_type: null`. |
| Nested choice generates action | Sub-choice inside a parent choice. | Both parent and child produce `ActionSymbol`s. Both registered in `symbol_table.actions`. |
| Choice action ID matches choice ID | Choice with label "Ask about the harbor". | `ActionSymbol.id == ChoiceSymbol.compiled_id`. |
| Frontmatter action collides with choice action | Frontmatter declares action `"tavern/topics/ask-about-the-harbor"`, same section has matching choice. | Duplicate detected. Standard duplicate rules apply (first wins). |

### Unit Tests: Resolution

| Test | Setup | Expected |
|------|-------|----------|
| Entity reference | `@guard` in dialogue, `@guard` declared. | Annotation resolves to `EntitySymbol("guard")`. |
| Type resolution | Entity `@guard: Guard`, type `Guard` declared. | `EntitySymbol.type_symbol` populated. |
| Property access | `@guard.mood`, type `Guard` has property `mood`. | Property annotation resolves to `PropertySymbol("mood")`. |
| Unresolved entity | `@missing` not declared anywhere. | URD301. Annotation is `null`. |
| Unresolved with suggestion | `@gaurd` not declared, `@guard` exists. | URD301 with "Did you mean '@guard'?" |
| Scope enforcement | `@guard` declared in file B. File A does not import B. Reference in A. | URD301 with hint "declared in B but B is not imported by A." |
| Jump to section | `-> topics`, section `== topics` in same file. | Resolves to `SectionSymbol`. |
| Jump to exit | `-> north`, no section named `north`, exit named `north` exists. | Resolves to `ExitSymbol`. |
| Jump shadowing | `-> topics`, section `== topics` exists and exit `topics` exists. | Resolves to section. URD310 warning. |
| Explicit exit jump | `-> exit:north`. | Resolves to `ExitSymbol` directly, bypassing section lookup. |
| Unresolved jump | `-> nowhere`, no section or exit matches. | URD309. |
| Exit destination | `-> north: Harbor`, `# Harbor` exists. | `ExitSymbol.resolved_destination` = `LocationSymbol("harbor")`. |
| Unresolved exit dest | `-> north: Nowhere`, no location matches. | URD312. `resolved_destination` = `null`. |
| Entity presence | `[@rusty_key, @cell_door]`. | Both resolved. `LocationSymbol.contains` = `["rusty_key", "cell_door"]`. |
| Forward reference | Entity `@guard` referenced before declaration in same file. | Resolves correctly (collection pass ran first). |
| Cross-file reference | Entity `@guard` in file B, referenced in file A which imports B. | Resolves correctly. Visible scope includes B. |
| Exhaustion check | `? topics.exhausted` in same file as `== topics`. | Resolves to `SectionSymbol("topics")`. |
| Property on unresolved type | `@guard.mood`, type `Guard` not declared. | URD307 for type. No URD308 for property (no cascading). |
| Conflicted symbol | `@guard` declared twice, then referenced. | URD302 during collection. Reference resolves to first declaration normally. |

### Unit Tests: ID Derivation

| Test | Input | Expected ID |
|------|-------|------------|
| Section ID | `== topics` in `tavern.urd.md` | `"tavern/topics"` |
| Choice ID | "Ask about the harbor" in `tavern/topics` | `"tavern/topics/ask-about-the-harbor"` |
| Location ID | `# The Rusty Anchor` | `"the-rusty-anchor"` |
| Slugify with special chars | `# Café & Bar!!!` | `"caf-bar"` |
| Slugify collapse | `# -- hello -- world --` | `"hello-world"` |
| Empty slug | `# !!!` | URD313. |

### Integration Tests

| Test | Setup | Expected |
|------|-------|----------|
| Two Room Key Puzzle | Single file, types + entities + two locations. | Full symbol table. All references resolved. Zero errors. |
| Multi-file project | Entry imports types and npcs files. | Cross-file entity resolution works. Visible scope enforced. |
| Jump disambiguation | File with section and exit sharing a name. | Section takes priority. Warning emitted. Explicit exit syntax works. |

### Error Recovery Tests

| Test | Setup | Expected |
|------|-------|----------|
| One bad entity among good | Five entities, one references missing type. | URD307 for one. Four others fully resolved. |
| Cascading suppressed | Entity type missing. Three property accesses on that entity. | URD307 once. Zero URD308 (no cascading). |
| ErrorNode among valid | AST has ErrorNode between two valid sections. | Sections collected and resolved. ErrorNode skipped. Zero LINK errors from ErrorNode. |
| Duplicate then reference | `@guard` declared twice, then referenced in conditions. | URD302 during collection. References resolve to first declaration. No URD301. |


## Relationship to Other Phases

### From IMPORT

LINK receives the `CompilationUnit` and needs:

1. **`ordered_asts`** to process files in topological order during both passes.
2. **`DependencyGraph`** to compute visible scopes (which files directly import which).
3. **Guarantee that file stems are unique** (IMPORT's guarantee #4), so section IDs are unique by construction.
4. **Guarantee that the graph is acyclic and the ordering is deterministic** (IMPORT's guarantees #1 and #3), so insertion order is stable.

### To VALIDATE

VALIDATE receives the `LinkedWorld` and needs:

1. **The `SymbolTable`** to check types, enum values, ref targets, and constraints.
2. **Annotated ASTs** where annotation slots are either populated (resolved) or `null` (unresolved). VALIDATE silently skips constructs with `null` annotations — no cascading errors from LINK failures.
3. **Confidence that every resolvable reference is resolved**, so VALIDATE can focus on semantic correctness, not name resolution.
4. **Confidence that scope is enforced**, so VALIDATE does not need to re-check visibility.

### To EMIT

EMIT reads the same `LinkedWorld` and needs:

1. **The `SymbolTable` in insertion order** for deterministic JSON output.
2. **Annotated ASTs** with resolved symbols for generating correct references in JSON.
3. **Compiled IDs** on all symbols for JSON keys.

*End of Brief*
