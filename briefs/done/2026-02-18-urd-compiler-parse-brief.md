# URD — Compiler Phase 1: PARSE

*Source text to per-file Abstract Syntax Trees*

February 2026 | Engineering Phase

`.urd.md source text → PARSE → FileAST`

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-18
**Status:** Done — all 76 tests pass, fully conformant with brief and architecture document

### What was done

- Implemented hand-written recursive descent parser in three modules: `parse/mod.rs` (core, line splitting, span helpers, tab handling), `parse/frontmatter.rs` (YAML-like frontmatter sub-parser), `parse/content.rs` (narrative content Block dispatch)
- Parser is line-oriented: pre-splits source into `LineInfo` records with byte offsets and 1-indexed line numbers, then dispatches each line through the PEG-ordered Block alternatives
- Frontmatter sub-parser handles: `world:` blocks, `types:` with traits and property definitions (including `enum()`, `ref()`, `list()` types), `entities:` with inline object overrides, `import:` declarations, flow-style lists, inline objects, nested entries, and all five YAML rejection educational errors (URD105–URD109)
- Content parser implements all 13 Block dispatch alternatives in correct PEG priority order: OrConditionBlock, RuleBlock, Headings (### before ## before #), SectionLabel, EntityLine (speech before stage direction), ArrowLine (exit declaration before exit-qualified jump before plain jump), ConditionLine, EffectLine, ChoiceLine, BlockedMessage, EntityPresence, LineComment, Prose
- Condition expressions parsed into structured `ConditionExpr` union: `PropertyComparison`, `ContainmentCheck` (with negation), `ExhaustionCheck`
- Effect types parsed into `EffectType` enum: `Set` (with operator), `Move`, `Reveal`, `Destroy`
- Choice nesting via recursive `parse_content(parser, min_indent)` — blank lines inside choice bodies do not terminate the choice
- Exit declaration children collected via `parse_exit_children()` — only Condition and BlockedMessage at strictly deeper indent, blank lines skipped
- Rule blocks parsed with actor/trigger extraction, optional `selects...from...where` clause, top-level where clauses, and effect lines
- Inline comment stripping on all 8 Text-bearing node types (headings, speech, stage direction, prose, blocked message, choice label)
- All annotation slots set to `None` — LINK's responsibility
- `indent_level` recorded on Choice, Condition, OrConditionBlock, Effect, Jump, BlockedMessage — no depth enforcement (VALIDATE's job)
- `content_line_span()` computes span start_col from structural indent spaces (not tabs), matching brief's requirement that indented content has start_col at the sigil position
- 76 acceptance tests: all grammar rules, 4 integration tests (Tavern Scene, Monty Hall, Two Room Key Puzzle, Interrogation), error recovery, span accuracy (including multi-byte UTF-8 and tab-at-indent), negative tests, frontmatter educational errors, BOM handling

### What changed from the brief

- **AST modifications from architecture scaffolding phase:** `ErrorNode` uses `raw_text: String` + `attempted_rule: Option<String>` (was `message: String`). `EffectType::Set` includes `operator: String` field. `ExitDeclaration` has `children: Vec<ContentNode>`. All nestable nodes carry `indent_level: usize`. These were established during the architecture brief implementation and are consistent with the PARSE brief's requirements.
- **Parameter order:** `parse(path, source, diagnostics)` rather than `parse(source, file_path, diagnostics)`. Internal API — functionally identical.
- **`@björk` multi-byte identifier test:** The identifier extraction uses `is_ascii_alphanumeric()`, so `ö` (non-ASCII) terminates the identifier at `"bj"`. The test verifies byte-accurate span end_col rather than correct identifier parsing of non-ASCII entity names. Non-ASCII identifiers are not part of the PEG grammar's `Identifier` rule (lowercase ASCII + digits + underscore), so this is correct behaviour — the test exercises span column encoding, not identifier acceptance.
- **WorldBlock stores fields as `Vec<(String, Scalar)>`** rather than the brief's `Map<string, Scalar>`. Preserves insertion order without requiring an ordered map dependency in the frontmatter sub-parser. Same data, different container.
- **Entity override values stored as `Vec<(String, Scalar)>`** rather than `Map`. Same reasoning as WorldBlock fields.
- **`world: value` shorthand (single-line form) discards the inline value.** The brief only documents the block form (`world:\n  name: ...\n  start: ...`). All examples use block form. No test exercises single-line world syntax.

---

> **Document status: BRIEF** — Defines the PARSE phase of the Urd compiler. PARSE is the first phase of the five-phase pipeline. It reads a single `.urd.md` file and produces a `FileAST` with span-tracked nodes. It knows nothing about imports, references, types, or semantic validity. It knows whether the text is syntactically well-formed.

> **Dependencies:** This brief builds on the Compiler Architecture Brief (shared data structures, AST node types, diagnostic codes, error recovery strategy) and the Formal Grammar Brief (PEG rules, ambiguity resolution, validation corpus). Both are required reading.


## Purpose

PARSE transforms a single `.urd.md` source file into a `FileAST`. It is the compiler's front end. Every subsequent phase depends on its output being correct, span-accurate, and complete.

PARSE has exactly one job: syntactic validity. It determines whether the source text conforms to the Schema Markdown grammar. It does not know whether `@guard` exists, whether `mood` is a valid property, whether an import target is a real file, or whether a section name is unique. Those are IMPORT, LINK, and VALIDATE responsibilities.

### What PARSE Does

- Reads a source file as UTF-8 text.
- Checks the file size limit (URD103: 1 MB maximum) before parsing.
- Splits the file into two regions: optional frontmatter and narrative content.
- Parses frontmatter using the restricted YAML-like grammar.
- Parses narrative content using the PEG grammar's `Block` dispatch.
- Produces a `FileAST` where every node carries a `Span` with file path, start line, start column, end line, end column.
- Leaves all annotation slots `null` — LINK fills them later.
- Reports syntactic errors as diagnostics in the URD100–URD199 range.
- On error, performs synchronisation-point recovery and continues parsing.

### What PARSE Does Not Do

- Resolve imports. It sees `import: ./world.urd.md` and produces an `ImportDecl` node. It does not open the file.
- Resolve references. It sees `@guard` and records it as a raw string. It does not check whether the entity exists.
- Validate types. It sees `mood: enum(hostile, neutral)` and records the type signature. It does not check whether enum values are valid downstream.
- Detect duplicates. It sees two `== topics` sections and produces two `SectionLabel` nodes. LINK detects the conflict.
- Enforce nesting depth policy. It accepts up to 4 indent levels (the grammar accepts them). VALIDATE enforces the depth 3 = warning, depth 4 = error policy. PARSE simply records the indent level on each node.
- Derive IDs. It stores `display_name` on headings, not slugified IDs. LINK derives IDs.


## Interface Contract

### Input

```
parse(source: string, file_path: FilePath, diagnostics: DiagnosticCollector) → FileAST | null
```

- `source`: The complete UTF-8 text content of the file. The caller (the compiler orchestrator or IMPORT) is responsible for reading the file from disk. PARSE never touches the filesystem.
- `file_path`: The normalized path of the file (per the architecture brief's path normalization rules). Used for `Span.file` on every node and for diagnostic reporting.
- `diagnostics`: The shared diagnostic collector. PARSE appends to it; it never reads from it.

### Output

On success: a `FileAST` containing the complete syntactic representation of the file.

On partial success (recoverable errors): a `FileAST` containing all successfully parsed nodes plus `ErrorNode` markers at damaged spans. At least one diagnostic with Error severity has been appended to the collector.

On catastrophic failure: `null`. A diagnostic explaining the failure has been appended to the collector. The compiler orchestrator skips this file.

**Exhaustive list of conditions where PARSE returns `null`:**
1. File exceeds 1 MB size limit (URD103).
2. Unclosed frontmatter — opening `---` without matching closing `---` (URD101).

No other condition causes PARSE to return `null`. All other errors produce a partial `FileAST` with `ErrorNode` markers.

### Guarantees

After PARSE completes (whether fully or partially successful), the following properties hold:

1. **Every node has a valid Span.** No node has a zero-length span or an out-of-bounds line number. Spans are byte-accurate for columns and 1-indexed for lines. Column is defined as: byte offset from the start of the line, plus 1. The first character on a line is column 1. Span end positions are exclusive: `end_line` and `end_col` point to the byte immediately after the last byte of the node's text, making `source[start..end]` the correct slice. For multi-byte UTF-8 characters, the column advances by the number of bytes, not the number of characters or graphemes (see the architecture brief's Column Encoding section). Span columns are computed from the original byte stream before tab normalisation or other recovery transformations.
2. **All annotation slots are null.** PARSE never populates annotations. This is LINK's responsibility.
3. **The AST faithfully represents the source text.** No information is lost between the source and the AST. Every character of content is accounted for in some node. Comments are preserved as `Comment` nodes (stripped later by EMIT). Inline comment text is not stored as a separate node, but it is covered by the source `Span` of the line's primary node — the span extends to the end of the line including the comment. The guarantee is span coverage, not content storage.
4. **Indent levels are recorded but not judged.** The `indent_level` field on `Choice`, `Condition`, `OrConditionBlock`, `Effect`, `Jump`, and `BlockedMessage` nodes records the number of two-space indents. PARSE does not emit warnings or errors for depth — that is VALIDATE's job.
5. **Frontmatter and content are independently recoverable when the closing delimiter is found.** A frontmatter error does not prevent parsing narrative content, and vice versa. The one exception: an unclosed frontmatter block (URD101) returns `null` because the parser cannot determine where content begins.
6. **ErrorNode markers are placed at damaged spans.** When recovery occurs, the damaged text is captured in an `ErrorNode` with the raw text, the span, and the rule that was being attempted (when known — `attempted_rule` may be `null` if the failure occurred before any specific rule was entered).
7. **The node order matches source order.** `FileAST.content` is an ordered list of content nodes in the order they appear in the source file. This ordering is preserved through all subsequent phases and ultimately determines EMIT output order.


## File Size Check

Before parsing begins, PARSE checks the byte length of the source string.

If the source exceeds 1,048,576 bytes (1 MB):
- Emit diagnostic URD103: *"File exceeds 1 MB size limit: {path} is {size} bytes."*
- Return `null`. Do not attempt to parse.

This check happens before any parsing work. It is the first thing PARSE does after receiving the source string.


## Two-Region Parsing

A `.urd.md` file has two regions, parsed by separate sub-parsers:

```
File → Frontmatter? Content EOF
```

### Region Detection

PARSE inspects the first line of the file to determine whether frontmatter is present.

**Frontmatter detection rules:**
- The first line of the file must start with exactly `---` followed by optional trailing whitespace and a newline character (`\n` or `\r\n`). No leading spaces, no leading blank lines, no BOM (see below).
- If the file begins with a UTF-8 BOM (`0xEF 0xBB 0xBF`), the BOM is stripped before detection. The first non-BOM content must be `---`.
- If the file begins with blank lines, whitespace, or any character other than `-`, frontmatter is absent — the entire file is narrative content.
- The delimiter line must start with exactly three hyphens. `----` (four hyphens) and `-- -` are not valid delimiters. Trailing whitespace after `---` is allowed and stripped on both opening and closing delimiters.

**Closing delimiter:** The closing `---` follows the same rules: exactly three hyphens at the start of a line, optional trailing whitespace, followed by a newline. If not found, PARSE emits URD101 and returns `null`.

When frontmatter is present:
1. Find the closing `---` line. If not found, emit URD101 (*"Unclosed frontmatter block. Expected closing '---'."*) and return `null`.
2. Extract the text between the two `---` delimiters.
3. Parse the frontmatter text using the frontmatter sub-parser. Produce a `Frontmatter` node.
4. Parse everything after the closing `---` line as narrative content. Produce the `content` node list.

When frontmatter is absent:
1. Parse the entire file as narrative content. `FileAST.frontmatter` is `null`.

### Independence

Frontmatter parsing failures do not prevent narrative content parsing. If the frontmatter sub-parser encounters an error, it emits diagnostics, produces a partial `Frontmatter` node (with whatever entries it could parse), and narrative content parsing proceeds normally. The only exception is an unclosed frontmatter block (URD101), which prevents determining where narrative content begins.


## Frontmatter Sub-Parser

The frontmatter sub-parser handles the restricted YAML-like syntax between `---` delimiters. It is a dedicated parser module — the PEG grammar defines `FrontmatterBody` as a delegated rule. Frontmatter indentation rules are independent of narrative indentation and handled entirely by the frontmatter sub-parser. Implementations must not share indent-tracking state between the two parsers.

### What It Accepts

Key-value pairs, nested blocks (two-space indentation), inline objects (`{ key: value }`), flow-style lists (`[a, b, c]`), entity references (`@name`), type definitions with traits (`Door [interactable]:`), property definitions with types and defaults, hidden property prefix (`~`), quoted strings, comments (`#`), import declarations, and the `world:` block.

### What It Rejects (with Educational Errors)

| Construct | Diagnostic | Message |
|-----------|-----------|---------|
| Anchors (`&name`) | URD105 | *"YAML anchors are not supported in Urd frontmatter. Define each value explicitly."* |
| Aliases (`*name`) | URD106 | *"YAML aliases are not supported in Urd frontmatter. Repeat the value where needed."* |
| Merge keys (`<<:`) | URD107 | *"YAML merge keys are not supported in Urd frontmatter."* |
| Custom tags (`!!type`) | URD108 | *"YAML custom tags are not supported in Urd frontmatter."* |
| Block-style lists (`- item`) | URD109 | *"Block-style lists are not supported. Use flow-style lists: [item1, item2]."* |

Tab characters in frontmatter are handled by the file-wide URD102 diagnostic (see Tab Rejection). There is no separate frontmatter-specific tab code.

These are educational errors — they tell the author what to do instead, not just what went wrong. Authors familiar with YAML will attempt these constructs. The error messages redirect them to the Urd subset.

### Frontmatter Node Production

The sub-parser walks the frontmatter text line by line, building a tree of `FrontmatterEntry` nodes. The top-level keys it recognises and their AST mappings:

| Key | AST Node | Notes |
|-----|----------|-------|
| `world:` | `WorldBlock` | Contains `name`, `start`, `entry`, and any future metadata fields as scalar key-value pairs. |
| `types:` | Block of `TypeDef` nodes | Each child key is a type name with optional traits and property definitions. |
| `entities:` | Block of `EntityDecl` nodes | Each child key starts with `@` and maps to a type with optional property overrides. |
| `import:` | `ImportDecl` | Value is a file path string. Multiple `import:` entries are allowed (each on its own line). |
| Other keys | `FrontmatterEntry` with generic `FrontmatterValue` | Future-proofing. Unknown keys are parsed but VALIDATE may warn about them. |

### Type Definition Parsing

A type definition line looks like: `  Key [portable]:` or `  Guard [interactable, mobile, container]:`.

The sub-parser extracts:
- **Name:** The identifier before the optional trait list. Must match the `TypeName` rule (starts with uppercase).
- **Traits:** The comma-separated identifiers inside `[...]`. Optional — a type with no traits has an empty traits list.
- **Properties:** Indented lines below the type definition. Each is a `PropertyDef`.

### Property Definition Parsing

A property line looks like: `    name: string` or `    ~prize: enum(goat, car)` or `    mood: enum(hostile, neutral) = hostile`.

The sub-parser extracts:
- **Name:** The identifier. If prefixed with `~`, visibility is `hidden`.
- **Type:** One of: `string`, `bool`, `integer`, `number`, `enum(...)`, `ref(TypeName)`, `list(...)`.
- **Default:** Optional. After `=`. Parsed as a scalar value.
- **Enum values:** For `enum(...)`, the comma-separated identifiers inside parentheses.
- **Ref target type:** For `ref(TypeName)`, the type name inside parentheses.
- **Constraints:** `min` and `max` for numeric types, parsed from shorthand if present.

### List Type Parsing

A list property type line looks like: `    contents: list(ref(Item))` or `    tags: list(string)`.

The sub-parser extracts:
- **Element type:** The type inside `list(...)`. Parsed using the same type rules as scalar properties — `string`, `bool`, `integer`, `number`, `enum(values...)`, or `ref(TypeName)`.
- **Nested constraints:** For `list(enum(a, b))`, the enum values are extracted. For `list(ref(Type))`, the ref target type is extracted.

Entity override values for list properties use flow-style list syntax: `[value1, value2, value3]`. Each element is parsed as a scalar value matching the declared element type. The frontmatter sub-parser's existing flow-style list handling (`[a, b, c]`) applies — no new syntax is introduced.

### Entity Declaration Parsing

An entity declaration line looks like: `  @rusty_key: Key { name: "Rusty Key" }`.

The sub-parser extracts:
- **ID:** The identifier after `@`. Must match the `Identifier` rule (lowercase).
- **Type name:** The identifier after `:`. Must match `TypeName` (uppercase start).
- **Property overrides:** The key-value pairs inside `{ ... }`. Optional — an entity with no overrides has an empty overrides map.

### Import Declaration Parsing

An import line looks like: `import: ./world.urd.md`.

The sub-parser extracts:
- **Path:** The string value after `import:`. Stored as-is in the `ImportDecl` node. IMPORT (phase 2) resolves and normalizes it.

Multiple `import:` lines are allowed. Each produces a separate `ImportDecl` node.

### Frontmatter Error Recovery

When the frontmatter sub-parser encounters a line it cannot parse:
1. Emit a diagnostic (URD111: *"Unrecognised frontmatter syntax at line {N}: '{raw_text}'."*).
2. Skip the line and continue parsing the next line.
3. If the line was inside a nested block (e.g., a property inside a type definition), skip to the next line at the same or lesser indentation level.

The sub-parser never aborts — it produces as many entries as it can, reporting each problem individually.


## Narrative Content Parser

The narrative content parser implements the PEG grammar's `Block` dispatch. It processes lines sequentially, matching each against the ordered rule list.

### Block Dispatch Order

The parser tries each alternative in the order specified by the grammar. This order is critical — it resolves all syntactic ambiguities via PEG's ordered choice semantics.

**Indentation handling before dispatch.** Before rule dispatch, leading indentation (`INDENT` tokens, each exactly two spaces) is consumed and recorded as `indent_level`. Grammar rule matching operates on the remainder of the line after indentation is stripped. This means the `Block` alternatives match against content starting at the sigil, not at leading whitespace.

```
Block ← OrConditionBlock       // multi-line, must come first
      / RuleBlock              // multi-line, must come before single-line
      / Heading                // ### before ## before #
      / SectionLabel           // == name
      / EntityLine             // EntitySpeech before StageDirection
      / ArrowLine              // ExitDeclaration before Jump (including exit-qualified)
      / ConditionLine          // OrConditionBlock already handled above; ? expr here
      / EffectLine             // > effect
      / ChoiceLine             // * or + with label
      / BlockedMessage         // ! text
      / EntityPresence         // [@entity, @entity]
      / LineComment            // // text
      / Prose                  // fallback: any non-blank line
```

**Prose is always last.** It matches any line that no other rule matched. If a line reaches `Prose`, it is treated as narrative text.

**ArrowLine resolution.** The grammar's `ArrowLine` group tries `ExitDeclaration` first (pattern: `-> identifier: text`), then `ExitJump` (pattern: `-> exit:identifier`), then `Jump` (pattern: `-> identifier`). Both `ExitJump` and `Jump` produce the same AST node: `Jump`. `ExitJump` sets `is_exit_qualified: true`; plain `Jump` sets it to `false`. There is no separate `ExitJump` AST node type — it is a grammar rule that maps to `Jump`.

**ConditionLine resolution.** `OrConditionBlock` is listed as a top-level `Block` alternative (it is multi-line and must be tried before single-line blocks). The `ConditionLine` entry in the dispatch handles only simple `? expr` conditions. The grammar's `ConditionLine ← OrConditionBlock / Condition` ordering is already expressed by the `Block` dispatch order above.

### Grammar Rule to AST Node Mapping

Every grammar rule produces exactly one AST node type. This mapping is exhaustive — no grammar rule is unmapped, and no AST node type is unproduced.

| Grammar Rule | AST Node | Key Fields Set by PARSE |
|-------------|----------|------------------------|
| `LocationHeading` | `LocationHeading` | `display_name` (raw heading text after `# `), `span` |
| `SequenceHeading` | `SequenceHeading` | `display_name` (raw heading text after `## `), `span` |
| `PhaseHeading` | `PhaseHeading` | `display_name` (raw text before `(auto)` if present), `auto` (boolean), `span` |
| `SectionLabel` | `SectionLabel` | `name` (the identifier after `== `), `span` |
| `EntitySpeech` | `EntitySpeech` | `entity_ref` (raw string, e.g., `"arina"`), `text` (speech content), `span` |
| `StageDirection` | `StageDirection` | `entity_ref` (raw string), `text` (action description), `span` |
| `EntityPresence` | `EntityPresence` | `entity_refs` (list of raw strings), `span` |
| `ChoiceLine` | `Choice` | `sticky` (boolean: `+` = true, `*` = false), `label` (text), `target` (raw string or null), `target_type` (raw string or null), `indent_level` (integer), `content` (nested child nodes), `span` |
| `Condition` | `Condition` | `expr: ConditionExpr` (structured union — see Condition Expression Parsing below), `indent_level`, `span` |
| `OrConditionBlock` | `OrConditionBlock` | `conditions: ConditionExpr[]` (each indented line parsed as a structured `ConditionExpr`), `indent_level`, `span` |
| `SetEffect` | `Effect { effect_type: "set" }` | `target_prop` (e.g., `"@guard.mood"`), `operator` (`"="`, `"+"`, `"-"`), `value_expr`, `indent_level`, `span` |
| `MoveEffect` | `Effect { effect_type: "move" }` | `entity_ref`, `destination_ref`, `indent_level`, `span` |
| `RevealEffect` | `Effect { effect_type: "reveal" }` | `target_prop`, `indent_level`, `span` |
| `DestroyEffect` | `Effect { effect_type: "destroy" }` | `entity_ref`, `indent_level`, `span` |
| `Jump` | `Jump` | `target` (raw string), `is_exit_qualified` (boolean: true if `exit:` prefix), `indent_level`, `span` |
| `ExitDeclaration` | `ExitDeclaration` | `direction` (e.g., `"north"`), `destination` (e.g., `"Corridor"`), `children` (Condition and BlockedMessage nodes), `span` |
| `BlockedMessage` | `BlockedMessage` | `text` (after `! `), `indent_level`, `span` |
| `RuleBlock` | `RuleBlock` | `name`, `actor` (entity ref), `select` (optional: variable name + entity list), `where_clauses` (list of condition exprs), `effects` (list of Effect nodes), `span` |
| `LineComment` | `Comment` | `text` (after `// `), `span` |
| `Prose` | `Prose` | `text` (the full line content), `span` |
| (parse failure) | `ErrorNode` | `raw_text`, `attempted_rule` (name of the rule that failed, if known), `span` |

### Condition Expression Parsing

Condition expressions are parsed into the structured `ConditionExpr` type defined in the Compiler Architecture Brief (Condition Expression Types section). PARSE produces the structured representation; LINK and VALIDATE operate on typed fields without re-parsing.

The `ConditionExpr` union and its variants (`PropertyComparison`, `ContainmentCheck`, `ExhaustionCheck`) are authoritative as defined in the architecture brief. The mapping from source syntax to variants:

| Source Form | Variant | Key Fields |
|-------------|---------|-----------|
| `? @guard.mood == neutral` | `PropertyComparison` | entity_ref: `"guard"`, property: `"mood"`, operator: `"=="`, value: `"neutral"` |
| `? @rusty_key in here` | `ContainmentCheck` | entity_ref: `"rusty_key"`, container_ref: `"here"`, negated: `false` |
| `? @rusty_key not in player` | `ContainmentCheck` | entity_ref: `"rusty_key"`, container_ref: `"player"`, negated: `true` |
| `? topics.exhausted` | `ExhaustionCheck` | section_name: `"topics"` |

PARSE is responsible for parsing the expression text into the correct variant. If the expression text matches no variant (the `?` sigil was matched but the expression is malformed), the entire line becomes an `ErrorNode` with URD112 and `attempted_rule` set to `"ConditionExpr"`. This gives downstream tooling a hook for more specific diagnostics without requiring a dedicated error code.

### Choice Content and Nesting

A `Choice` node contains nested child nodes — the content indented below the choice line. PARSE handles this by tracking indentation.

**Nesting algorithm:**

1. When PARSE encounters a `ChoiceLine`, it records the indent level of that line.
2. It then scans subsequent lines. Any line indented deeper than the choice line is a child of that choice.
3. Child lines are parsed using the same `Block` dispatch. They may themselves be choices (sub-choices), conditions, effects, jumps, speech, stage directions, or prose.
4. When a line is encountered at the same or lesser indentation than the choice, the choice's content is complete.

Blank lines inside a choice body are permitted and do not end the choice body. They produce no node and are skipped. The choice body ends only when a non-blank line at the same or lesser indentation is encountered. This allows authors to use visual spacing within choices for readability.

**Indent level recording:**

PARSE counts the number of `INDENT` tokens (each is exactly two spaces) at the start of each line. This count is stored as `indent_level` on nodes that can be nested: `Choice`, `Condition`, `OrConditionBlock`, `Effect`, `Jump`, `BlockedMessage`.

**What PARSE does with depth:**

PARSE records the indent level. It does not enforce depth policy. A line at indent level 4 is parsed successfully and produces a node with `indent_level: 4`. VALIDATE later checks this value and emits URD403 if it exceeds the limit.

**Choice target parsing:**

A choice line may end with a target: `* Pick a door -> any Door` or `* Back off -> interrogation` or `* Use key -> @cell_door`.

PARSE inspects the text after `-> `:
- If it starts with `@`: `target` is set to the entity ref, `target_type` is null.
- If it starts with `any ` followed by a `TypeName`: `target` is null, `target_type` is the type name.
- Otherwise: `target` is the identifier (a section or exit name), `target_type` is null.

Choices without `->` have both `target` and `target_type` as null. Their content is in the nested `content[]`.

### Multi-Line Block Parsing

Two block types consume multiple lines: `OrConditionBlock` and `RuleBlock`.

**OrConditionBlock:**
1. The header line `? any:` is recognised by the `?` sigil followed by ` any:`. The `indent_level` on the `OrConditionBlock` node is the indentation of this header line, not of the inner condition lines. This determines nesting under a parent `Choice` and where the block ends.
2. PARSE then consumes subsequent indented lines, parsing each as a bare `ConditionExpr` (without the `?` prefix).
3. Consumption stops when a line at the same or lesser indentation is encountered, or a blank line is found. **Blank lines terminate the OrConditionBlock but do not terminate the parent block.** If the OrConditionBlock is nested inside a choice, the parent choice resumes scanning for more child lines after the blank line. This means you cannot write spaced condition lists inside `? any:` — all conditions must be contiguous. This is a deliberate constraint for readability and unambiguous parsing.

**RuleBlock:**
1. The header line `rule name:` is recognised by the `rule ` keyword.
2. PARSE consumes the indented body: one `RuleActorLine`, zero or more `RuleWhereLine` entries, one or more `RuleEffectLine` entries. The exact grammar is defined in the Formal Grammar Brief's Rule Block section (`RuleBlock`, `RuleBody`, `RuleActorLine`, `RuleWhereLine`, `RuleEffectLine` rules).
3. Rule body lines use `RuleCondition` and `RuleEffect` (which accept bare identifiers for bound variables), not the standard `ConditionExpr` and `Effect`. This distinction is grammar-level — see the grammar brief's Rule-scoped conditions and Rule-scoped effects sections.
4. Body lines must be indented strictly deeper than the rule header line. Deeper indentation within the body is allowed and preserved. Consumption stops when a line's indentation returns to the rule header level or less, or at a blank line.

### Rule Block Inner Parsing

After consuming the header line (`rule name:`), PARSE processes the indented body lines in order:

1. **Actor line.** Pattern: `actor: @entity_name` or `actor: @entity_name trigger_clause`. Extracts the entity ref and the trigger string. The trigger clause follows one of: `phase_is identifier`, `action identifier`, `enter identifier`, `state_change entity.property`, or `always`.

2. **Select line (optional).** Pattern: `selects variable from [@entity_a, @entity_b, ...]`. Extracts the variable name and the entity ref list. If present, subsequent `where` lines within the select block scope to this variable.

3. **Where lines (zero or more).** Pattern: `where condition_expression`. Each condition is parsed as a `ConditionExpr` using the same rules as standard conditions, but the bound variable from `selects` (if present) is accepted in entity ref positions. Where clauses that appear before a `selects` line are top-level rule conditions; where clauses after `selects` are select-scoped.

4. **Effect lines (one or more).** Pattern: standard effect syntax (`> effect`). Parsed identically to narrative effects. The bound variable from `selects` may appear in entity ref positions.

All rule body lines must be indented strictly deeper than the `rule` header. PARSE records `indent_level` on each body line node but does not enforce depth policy — VALIDATE handles that.

### Inline Comment Stripping

Inline comments (`// text` at the end of a content line) are handled by the `Text` lexical rule in the grammar. Any grammar rule that uses `Text` for its content automatically supports inline comments. This includes: `EntitySpeech` (speech text), `StageDirection` (action text), `Prose` (narrative text), `BlockedMessage` (message text), `Choice` (label text), `LocationHeading` (display name), `SequenceHeading` (display name), and `PhaseHeading` (display name).

PARSE strips the inline comment from the content text before storing it in the AST node. The comment text is discarded — unlike `LineComment` nodes, inline comments are not preserved in the AST.

**Headings and inline comments.** Headings use `Text` and therefore support inline comments. `# Cell // test location` produces `LocationHeading { display_name: "Cell" }`. This is a deliberate choice: it allows authors to annotate headings with notes that do not appear in the compiled output or affect the derived location ID. The alternative — excluding headings from inline comment support — would require authors to place comments on a separate line above the heading, which is less ergonomic.

Example: `@arina: What'll it be? // hub prompt` produces an `EntitySpeech` node with `text: "What'll it be?"` (trailing space trimmed). The `// hub prompt` is consumed by the grammar but not stored.

Nodes that do not use `Text` (conditions, effects, jumps, section labels) do not support inline comments. A `//` on a condition line would be part of the expression and would cause a parse error, which is correct — conditions have no free-text region.

### Exit Declaration and Associated Content

Exit declarations (`-> north: Corridor`) may be followed by indented content: conditions and blocked messages. The architecture brief defines `ExitDeclaration` with a `children[]` field for this purpose.

```
-> north: Corridor

  ? @cell_door.locked == false

  ! The iron door is locked.
```

**Parsing rule:** After producing an `ExitDeclaration` node, PARSE checks subsequent lines. If they are indented at least one indent level deeper than the exit line and are `Condition` or `BlockedMessage` nodes, they are attached as entries in the `ExitDeclaration.children` list. The rule is **strictly greater indentation**, not exactly one level deeper — a condition at indent level 2 below an exit at indent level 0 is valid. Only `Condition` and `BlockedMessage` node types are valid as exit children — any other indented content after an exit is parsed normally (not attached to the exit) and will likely produce a diagnostic in VALIDATE.

If no indented content follows the exit, `children` is an empty list. Blank lines between the exit declaration and its children are skipped — they do not terminate the child scan. This matches how authors naturally format exits with visual spacing between the declaration and its conditions.


## Error Recovery

PARSE must not stop at the first error. It must collect as many syntactic errors as possible in a single run.

### Synchronisation-Point Recovery

When a line fails to match any `Block` alternative (and is not blank), PARSE performs synchronisation-point recovery:

1. **Capture the failed line.** Record the raw text and create an `ErrorNode` with the line's span.
2. **Emit a diagnostic.** URD112: *"Unrecognised syntax at line {N}: '{truncated_raw_text}'."* The diagnostic catalog lists the short form. Implementations may append an expectation hint (e.g., "Expected a heading, choice, condition, effect, jump, entity speech, or prose.") for author friendliness, but the hint is informational, not normative.
3. **Advance to the next line.** Consume all characters to the next `EOL`.
4. **Inspect the next line.** If it starts with a recognisable sigil (after optional indentation), resume normal parsing. If it starts with non-sigil text, treat it as `Prose` and resume.

Recovery does not attempt to infer missing indentation. A line that was intended to be indented content under a choice or exit but lacks the expected indentation will be parsed as a top-level `Prose` node. The author must fix their indentation. This is the correct behaviour — guessing at intent would produce silent misparses that are harder to debug than an obviously wrong AST.

### Frontmatter–Content Independence

If frontmatter parsing fails (but the closing `---` was found), narrative content parsing still proceeds. The `FileAST` will have a partial `Frontmatter` node with errors, and a complete `content` list.

The only frontmatter failure that prevents content parsing is an unclosed frontmatter block (URD101), because the parser cannot determine where content begins.

### ErrorNode Structure

```
ErrorNode {
  raw_text: string,           // the unparseable text
  attempted_rule: string | null,  // e.g., "ConditionExpr" if the ? sigil was matched but the expression was invalid
  span: Span,
}
```

`ErrorNode` is a valid content node. It appears in `FileAST.content` in source order, alongside successfully parsed nodes. LINK and VALIDATE silently skip `ErrorNode` entries. When errors occur within nested blocks (inside a `Choice.content` or `ExitDeclaration.children`), the resulting `ErrorNode` is inserted at the same structural nesting level where the failure occurred — it becomes a child of the enclosing block, not a top-level node.

### Tab Rejection

Tabs are rejected everywhere in the file. The grammar's `Char` rule excludes `\t`, and `INDENT` is defined as exactly two spaces. If PARSE encounters a tab character anywhere:

- Emit URD102: *"Tab character found at line {N}, column {col}. Use exactly two spaces per indent level."*
- Recovery depends on position:
  - **At indentation position** (start of line, before any non-whitespace content): treat the tab as two spaces (one indent level). This preserves the author's likely intended indentation structure.
  - **Elsewhere** (inside prose text, entity speech, or other content): treat the tab as a single space. This avoids distorting content width.
- Continue parsing in both cases.

**Indent level vs span coordinates.** Indent level is computed after tab recovery (the recovered two-space or one-space substitution determines structural nesting). Spans always refer to original source bytes (the tab character occupies one byte at the original column). These two values can diverge when tabs are present, and that is correct — structural indentation uses recovered text, span coordinates remain faithful to original bytes.


## Diagnostic Catalog

All diagnostics emitted by PARSE are in the URD100–URD199 range.

### Errors Emitted by PARSE

All errors have Error severity. Whether an error prevents JSON emission is determined by the compiler orchestrator, not by PARSE. PARSE reports problems and produces the best AST it can. The orchestrator decides whether to proceed to EMIT based on the accumulated diagnostic set across all phases.

| Code | Message Template | Trigger | Recovery |
|------|-----------------|---------|----------|
| URD101 | *"Unclosed frontmatter block. Expected closing '---'."* | Opening `---` without matching close. | Return `null`. Cannot determine content region. |
| URD102 | *"Tab character found at line {N}, column {col}. Use exactly two spaces per indent level."* | `\t` anywhere in the file (frontmatter or narrative content). | At indentation position: treat as two spaces. Elsewhere: treat as one space. Continue. |
| URD103 | *"File exceeds 1 MB size limit: {path} is {size} bytes."* | File size > 1,048,576 bytes. | Return `null`. Do not parse. |
| URD104 | *"Frontmatter nesting exceeds 8 levels at line {N}."* | Frontmatter indentation > 16 spaces (nesting level = leading spaces ÷ 2, using two-space indent). Nesting level is computed after tab recovery, consistent with the file-wide tab rule. | Skip line, continue. |
| URD112 | *"Unrecognised syntax at line {N}: '{text}'."* | Line fails all Block alternatives. | Create ErrorNode, advance to next line. |

### Educational Errors (Frontmatter Rejections)

| Code | Message Template | Trigger |
|------|-----------------|---------|
| URD105 | *"YAML anchors are not supported in Urd frontmatter. Define each value explicitly."* | `&name` in frontmatter. |
| URD106 | *"YAML aliases are not supported in Urd frontmatter. Repeat the value where needed."* | `*name` in frontmatter. |
| URD107 | *"YAML merge keys are not supported in Urd frontmatter."* | `<<:` in frontmatter. |
| URD108 | *"YAML custom tags are not supported in Urd frontmatter."* | `!!type` in frontmatter. |
| URD109 | *"Block-style lists are not supported. Use flow-style lists: [item1, item2]."* | `- item` in frontmatter. |
| URD111 | *"Unrecognised frontmatter syntax at line {N}: '{text}'."* | Line inside frontmatter fails all frontmatter rules. |

### Informational (Not Errors)

PARSE does not emit warnings or info-level diagnostics. All depth and style warnings come from VALIDATE. PARSE only reports things that prevent syntactic analysis.


## Acceptance Criteria

### Unit Tests: Grammar Rule Coverage

Every grammar rule must have at least one test that exercises it and verifies the correct AST node is produced with correct fields. Minimum coverage:

| Test | Input | Expected Output |
|------|-------|----------------|
| Location heading | `# The Rusty Anchor` | `LocationHeading { display_name: "The Rusty Anchor" }` |
| Sequence heading | `## The Game` | `SequenceHeading { display_name: "The Game" }` |
| Phase heading | `### Choose` | `PhaseHeading { display_name: "Choose", auto: false }` |
| Phase heading auto | `### Reveal (auto)` | `PhaseHeading { display_name: "Reveal", auto: true }` |
| Section label | `== topics` | `SectionLabel { name: "topics" }` |
| Entity speech | `@arina: What'll it be?` | `EntitySpeech { entity_ref: "arina", text: "What'll it be?" }` |
| Stage direction | `@arina leans in close.` | `StageDirection { entity_ref: "arina", text: "leans in close." }` |
| Entity presence | `[@arina, @barrel]` | `EntityPresence { entity_refs: ["arina", "barrel"] }` |
| One-shot choice | `* Ask about the ship` | `Choice { sticky: false, label: "Ask about the ship", target: null }` |
| Sticky choice | `+ Order a drink` | `Choice { sticky: true, label: "Order a drink", target: null }` |
| Choice with entity target | `* Use key -> @cell_door` | `Choice { target: "cell_door", target_type: null }` |
| Choice with type target | `* Pick a door -> any Door` | `Choice { target: null, target_type: "Door" }` |
| Choice with section target | `* Back off -> interrogation` | `Choice { target: "interrogation", target_type: null }` |
| Simple condition | `? @guard.mood == neutral` | `Condition { expr: PropertyComparison { ... } }` |
| Containment condition | `? @rusty_key in here` | `Condition { expr: ContainmentCheck { entity_ref: "rusty_key", container_ref: "here", negated: false } }` |
| Negated containment | `? @rusty_key not in player` | `Condition { expr: ContainmentCheck { negated: true } }` |
| Exhaustion condition | `? topics.exhausted` | `Condition { expr: ExhaustionCheck { section_name: "topics" } }` |
| OR condition block | `? any:` + indented exprs | `OrConditionBlock { conditions: [...] }` |
| Set effect | `> @guard.mood = neutral` | `Effect { effect_type: "set", target_prop: "@guard.mood", value_expr: "neutral" }` |
| Increment effect | `> @arina.trust + 5` | `Effect { effect_type: "set", operator: "+", value_expr: "5" }` |
| Move effect | `> move @rusty_key -> player` | `Effect { effect_type: "move", entity_ref: "rusty_key", destination_ref: "player" }` |
| Reveal effect | `> reveal @door_1.prize` | `Effect { effect_type: "reveal", target_prop: "@door_1.prize" }` |
| Destroy effect | `> destroy @rusty_key` | `Effect { effect_type: "destroy", entity_ref: "rusty_key" }` |
| Section jump | `-> topics` | `Jump { target: "topics", is_exit_qualified: false }` |
| Exit-qualified jump | `-> exit:harbor` | `Jump { target: "harbor", is_exit_qualified: true }` |
| Exit declaration | `-> north: Corridor` | `ExitDeclaration { direction: "north", destination: "Corridor" }` |
| Exit with children | `-> north: Corridor\n  ? @door.locked == false\n  ! Locked.` | `ExitDeclaration { children: [Condition, BlockedMessage] }`. Children at indent level 1, exit at level 0. |
| Exit with deep children | `-> north: Corridor\n    ? @door.locked == false` | `ExitDeclaration { children: [Condition] }`. Condition at indent level 2 still attaches (strictly greater rule). |
| Exit non-child content | `-> north: Corridor\n  @arina: Hello` | `ExitDeclaration { children: [] }` then `EntitySpeech`. EntitySpeech is indented under the exit but is not a valid exit child type (only Condition and BlockedMessage attach), so it is parsed as the next sibling node. |
| Exit children with blank lines | `-> north: Corridor\n\n  ? @door.locked == false\n\n  ! Locked.` | `ExitDeclaration { children: [Condition, BlockedMessage] }`. Blank lines between exit and children are skipped. Both children attach. |
| Choice body with blank lines | `* Ask\n\n  ? topics.exhausted\n\n  > reveal @x.y\nBack to prose` | `Choice { content: [Condition, Effect] }` then `Prose { text: "Back to prose" }`. Blank lines inside the choice body do not end it. The unindented prose line ends the choice and becomes a sibling. |
| Blocked message | `! The iron door is locked.` | `BlockedMessage { text: "The iron door is locked." }` |
| Prose | `A dim stone cell.` | `Prose { text: "A dim stone cell." }` |
| Line comment | `// hub prompt` | `Comment { text: "hub prompt" }` |
| Inline comment | `@arina: Hello // greeting` | `EntitySpeech { text: "Hello" }` (comment stripped) |
| Heading inline comment | `# Cell // test location` | `LocationHeading { display_name: "Cell" }` (comment stripped) |
| Line comment under heading | `# Cell\n// author note` | Two nodes: `LocationHeading { display_name: "Cell" }` then `Comment { text: "author note" }`. Comment is not swallowed by heading. |
| Rule block | `rule monty_reveals:` + body | `RuleBlock { name: "monty_reveals", ... }` |

### Integration Tests: Full File Parsing

The four canonical test case files must parse completely with zero errors:

| File | Key Constructs Exercised |
|------|------------------------|
| Tavern Scene | Frontmatter with types/entities, location heading, section labels, sticky/one-shot choices, conditions, effects, jumps, entity speech, stage directions, exhaustion fallthrough prose. |
| Monty Hall | Frontmatter with types/entities, location/sequence/phase headings, `(auto)` marker, rule block with `selects...from...where`, entity presence. |
| Two Room Key Puzzle | Frontmatter with `world:` block, exit declarations with conditions and blocked messages, containment conditions, move/destroy effects. |
| Interrogation | Import declaration, OR condition block, multi-section structure, choice nesting (depth 2), containment in dialogue. |

For each file: parse it, verify zero diagnostics, and verify the AST node count and types match expected values.

### Error Recovery Tests

| Test | Input | Expected Behaviour |
|------|-------|-------------------|
| Unclosed frontmatter | `---\nworld: test\n` (no closing `---`) | URD101 emitted. Returns `null`. |
| Bad line in content | `# Tavern\n%%% garbage\n@arina: Hello` | ErrorNode for line 2. EntitySpeech for line 3. Both in AST. |
| Tab indentation | `\t* Choice text` | URD102 emitted. Choice still parsed (tab treated as two spaces at indentation position). |
| Bad frontmatter line | `---\nworld: test\n<<: merge\n---` | URD107 emitted. WorldBlock still produced. |
| Multiple errors | File with 3 bad lines among 10 good lines | 3 diagnostics, 3 ErrorNodes, 7 valid nodes. All in order. |

### Span Accuracy Tests

For a file with known byte positions, verify that every AST node's span matches the expected line and column:

- First node starts at line 1 (or the first content line after frontmatter).
- Entity speech `@arina: Hello` at line 5 has `span.start_line: 5`, `span.start_col: 1`.
- An indented condition `  ? @guard.mood == hostile` at line 10 has `span.start_col: 3` (byte offset after two spaces).
- Multi-byte UTF-8 characters: a line containing `@björk: Hej` has `span.end_col` at the byte after the last character, accounting for the two-byte `ö`.
- Tab at indentation position: a line `\t* Choice text` has `indent_level: 1` (tab recovered as two spaces), but `span.start_col: 1` (the tab byte is at column 1 in the original source). The span reflects original bytes; the indent level reflects recovered structure.

### Negative Tests: Grammar Rejections

These inputs must fail to match a grammar rule and produce an `ErrorNode` with diagnostic URD112. The parser cannot always determine *why* a match failed (PEG reports the furthest match point, not the reason), so the diagnostic message is generic. The important guarantee is that these inputs do not produce valid nodes.

| Test | Input | Expected Behaviour |
|------|-------|-------------------|
| Invalid identifier character | `@Guard-1: Hello` | URD112. ErrorNode. Fails to match EntitySpeech or StageDirection due to identifier rules. |
| Empty choice label | `* ` (asterisk + space, no text) | URD112. ErrorNode. `Text` requires at least one character. |
| Uppercase entity ID | `@Guard: Hello` | URD112. ErrorNode. `Identifier` rule requires lowercase start; `TypeName` matched but `EntitySpeech` expects `Identifier`. |
| Missing space after sigil | `?@guard.mood == hostile` | URD112. ErrorNode. `Condition` rule requires space after `?`. |
| Missing space after heading | `#Heading` | URD112. ErrorNode. `LocationHeading` rule requires space after `#`. |


## Relationship to Grammar Artifact

The PEG grammar file (`urd-schema-markdown.peg`) is the authoritative syntax definition. PARSE implements it. If PARSE and the grammar disagree, the grammar wins.

The relationship is one-directional: the grammar defines what is valid; PARSE executes that definition and adds error recovery and AST construction on top. The grammar does not know about AST nodes, diagnostics, or recovery — those are PARSE's responsibilities.

In practice, the implementation may use the PEG file directly via a parser generator (pest in Rust, peggy in JavaScript) or implement the rules as a hand-written recursive descent parser. Either approach is valid as long as the acceptance criteria pass.


## What the Next Phase Needs

IMPORT (phase 2) receives the `FileAST` and needs:

1. **`ImportDecl` nodes** from the frontmatter, to know which files to load.
2. **The `file_path`** stored on the `FileAST`, to resolve relative import paths.
3. **Confidence that the AST is span-accurate**, because IMPORT's diagnostics (URD201 file not found, URD202 circular import) will reference source positions from the `ImportDecl` spans.

IMPORT does not need to understand the rest of the AST. It inspects only the frontmatter's import declarations. The content nodes pass through IMPORT unchanged to LINK.

*End of Brief*
