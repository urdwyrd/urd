---
title: "Schema Markdown Formal Grammar"
slug: "urd-formal-grammar-brief"
description: "A brief for the PEG grammar artifact. Defines scope, requirements, and acceptance criteria for the machine-readable grammar that validates .urd.md input — the compiler's parse phase as a standalone reference."
category: "brief"
format: "Task Brief"
date: "2026-02-14"
status: "Active"
order: 1
tags:
  - formalisation
  - grammar
  - PEG
  - compiler
  - parser
details:
  - "PEG grammar for .urd.md syntactic validation"
  - "Covers frontmatter, narrative content, and all sigil constructs"
  - "Ambiguity resolution via ordered choice"
  - "Error recovery strategy for batch and editor modes"
  - "Positive and negative validation corpus"
---

# URD — Schema Markdown Formal Grammar

*A brief for the PEG grammar artifact*

February 2026 | Formalisation Phase

`.urd.md → PEG grammar → parser → compiler`

> **Document status: BRIEF** — Defines scope, requirements, and acceptance criteria for the Schema Markdown formal grammar. This grammar is the machine readable definition of what constitutes valid `.urd.md` input. It lives in the compiler as the parse phase and exists as a standalone reference artifact (PEG file) for implementers.

## Purpose

The Urd framework has two locked specifications: the **World Schema Specification** (what valid `.urd.json` looks like) and the **Schema Markdown Syntax Specification** (what valid `.urd.md` looks like). Both are prose documents. Prose is readable but ambiguous. A formal grammar removes the ambiguity.

The grammar answers one question: **given a `.urd.md` file, is it syntactically valid?** If yes, the file can proceed to the compiler's later phases (import resolution, linking, type checking). If no, the grammar produces a precise error: what was expected, where it failed, and ideally what the author should do instead.

Without this artifact, the only way to validate `.urd.md` input is to run the full compiler. The grammar gives us an earlier, cheaper, and more precise check. It also serves as the authoritative reference for anyone implementing a parser, whether that's the Urd compiler, a third party tool, or an AI assistant generating Schema Markdown.

### What This Is Not

This is not the JSON Schema for `.urd.json` output. That is a separate formalisation artifact. The JSON Schema validates the *compiler's output*. The PEG grammar validates the *compiler's input*. Both are needed. Neither replaces the other.

## Where It Lives

The grammar exists in two forms:

- **As a reference artifact.** A standalone PEG file (`urd-schema-markdown.peg`) in the repository. Human readable. The authoritative definition of the syntax. Versioned alongside the specifications.
- **As the compiler's parse phase.** The compiler implements this grammar as its parser. The parse phase is phase 1 of the five phase pipeline (Parse → Import → Link → Validate → Emit). The grammar defines what the parser accepts.

The grammar file is the source of truth. The compiler's parser is an implementation of that source of truth. If the parser and the grammar disagree, the grammar wins.

> **Architectural note.** The grammar covers syntactic validity only. It can tell you that `? @guard.mood == neutral` is a well formed condition line. It cannot tell you that `@guard` exists or that `mood` is a valid property. That's the linker's job (phase 3). The grammar draws the line between "this text is parseable" and "this text is semantically correct."

## Format Choice: PEG

The grammar should be written in **Parsing Expression Grammar (PEG)** notation. PEG is the right fit for Schema Markdown for four reasons.

### Why PEG

- **Unambiguous by construction.** PEG uses ordered choice (the `/` operator). When two rules could match, the first one wins. There is no ambiguity to resolve. This is critical for a syntax where `@arina: text` (speech) and `@arina text` (stage direction) differ by a single colon.
- **Line oriented.** Schema Markdown is fundamentally line oriented. Each line starts with a sigil (`@`, `?`, `>`, `*`, `+`, `==`, `->`, `//`, `!`, `#`) or is plain prose. PEG handles line oriented grammars naturally through explicit newline tokens.
- **Error positions are precise.** When a PEG parse fails, the failure point is the furthest position reached in the input. This directly maps to a line and column, which maps to an actionable error message.
- **Tooling exists.** PEG parsers exist in Rust (`pest`, `pom`), TypeScript (`peggy`, `ohm`), and most other languages. The grammar file can be used directly by parser generators, or implemented by hand as a recursive descent parser following the PEG rules.

### Alternatives Considered

| Format | Verdict | Reason |
|---|---|---|
| EBNF | Workable but weaker | Allows ambiguous grammars. Would need separate disambiguation rules. |
| ANTLR | Over-engineered | Full parser generator framework. Introduces a build dependency. Good for complex languages, overkill for a seven symbol syntax. |
| Regex | Insufficient | Cannot handle nesting (indented sub choices) or context (frontmatter vs prose sections). |
| Handwritten prose | What we have now | The syntax spec is excellent prose. But prose admits interpretation; a grammar does not. |

## Scope: What the Grammar Defines

The grammar must cover the complete surface syntax of `.urd.md` files. This section enumerates every construct the grammar must accept or reject.

### File Structure

A `.urd.md` file has two regions: an optional Urd frontmatter block, followed by narrative content. Frontmatter is optional. If present, the opening `---` must be the first line of the file and must be closed with a matching `---` before narrative content begins. Files without frontmatter begin directly with narrative content.

The grammar must handle line endings robustly. The final line may or may not end in a newline. Blank lines (containing only whitespace) are valid between any constructs.

```
File        ← Frontmatter? Content EOF
Frontmatter ← '---' EOL FrontmatterBody '---' EOL
Content     ← Line*
Line        ← Block / BlankLine
BlankLine   ← SP* EOL?
SP          ← ' '
EOL         ← NEWLINE
NEWLINE     ← '\n' / '\r\n'
```

Note: `EOF` appears only on the `File` rule. `SP` matches a single space character. `EOL` is strictly `NEWLINE`. The final `BlankLine` in a file may have no trailing newline, hence `EOL?` on `BlankLine`. Every `Block` rule consumes its own `EOL` at the end of the line. Multi-line blocks consume `EOL` for each of their internal lines as well. This uniform convention means there is one rule: **every line-level rule owns its own line ending.**

**Block rules require a sigil after indentation.** Every `Block` alternative begins with optional `INDENT*` followed by a required sigil (`?`, `>`, `*`, `+`, `->`, `!`, `@`, `#`, `==`, `rule`, `[`, `//`, etc.). A line containing only spaces and no sigil always matches `BlankLine`, never a `Block`. This is enforced by PEG's ordered choice: `Block` alternatives require at least one sigil character after any indentation, and `Prose` (the final fallback) requires at least one non-whitespace character.

### Frontmatter Grammar

The frontmatter block uses YAML like syntax under strict constraints. The grammar must accept:

- `key: value` pairs (strings, integers, floats, booleans, null)
- Nested blocks via two space indentation
- Inline object shorthand: `@arina: Barkeep { name: "Arina" }`
- Flow style lists: `[goat, car]`, `[@door_1, @door_2]`
- Quoted strings when ambiguous: `"1"`, `"true"`
- Comments: `# text`
- Type definitions with traits: `Door [interactable, portable]:`
- Property shorthand: `name: string`, `mood: enum(hostile, neutral) = hostile`
- Hidden property prefix: `~prize: enum(goat, car)`
- Entity declarations: `@door_1: Door { prize: car }`
- `import: ./path` declarations
- `world:` block with metadata fields

The grammar must **reject**:

- Anchors and aliases (`&name`, `*name`)
- Merge keys (`<<:`)
- Custom tags (`!!type`)
- Block style lists (`- item`)

### Narrative Content Grammar

Below the frontmatter, every line is one of the following. The grammar must define each as a distinct rule.

| Rule Name | Sigil | Pattern | Example |
|---|---|---|---|
| LocationHeading | `#` | `# + SPACE + text` | `# The Rusty Anchor` |
| SequenceHeading | `##` | `## + SPACE + text` | `## The Game` |
| PhaseHeading | `###` | `### + SPACE + text + AutoMarker?` | `### Choose (auto)` |
| SectionLabel | `==` | `== + SPACE + name` | `== topics` |
| EntitySpeech | `@id:` | `@id + : + SPACE + text` | `@arina: What'll it be?` |
| StageDirection | `@id` | `@id + SPACE + text (no colon)` | `@arina leans in close.` |
| EntityPresence | `[@...]` | `'[' @id (',' SPACE? @id)* ']'` | `[@arina, @barrel]` |
| OneShotChoice | `*` | `INDENT* + * + SPACE + text` | `* Ask about the ship` |
| StickyChoice | `+` | `INDENT* + + + SPACE + text` | `+ Order a drink` |
| Condition | `?` | `INDENT* + ? + SPACE + expr` | `? @guard.mood == neutral` |
| OrConditionBlock | `? any:` | `INDENT* + ? any: + indented exprs below` | `? any:` followed by indented conditions |
| Effect | `>` | `INDENT* + > + SPACE + effect` | `> @guard.mood = neutral` |
| Jump | `->` | `INDENT* + -> + SPACE + target` | `-> topics` |
| ExitJump | `-> exit:` | `-> SPACE exit:name (no space after colon)` | `-> exit:harbor` |
| BlockedMessage | `!` | `INDENT* + ! + SPACE + text` | `! The door is locked.` |
| LineComment | `//` | `// + text (whole line)` | `// hub prompt` |
| RuleBlock | `rule` | `rule + SPACE + name + :` | `rule monty_reveals:` |
| ExitDeclaration | `->` | `-> SPACE Identifier ':' SPACE text` | `-> north: Corridor` |
| Prose | (none) | `any text not matching above` | `A dim stone cell.` |

**Inline comments.** In addition to whole-line `LineComment` above, any content-bearing line may end with an inline comment: `SP+ // text`. Inline comments are a suffix on other rules, not a separate Block type. See the Comments sub-rule section and the Lexical Tokens section for the full grammar.

#### Block Rule (Ordered)

The `Block` rule is the core dispatch for narrative content. PEG's ordered choice (`/`) means alternatives are tried left to right; the first match wins. The ordering below resolves all ambiguities documented in the Ambiguity Resolution section.

```
Block ← OrConditionBlock
     / RuleBlock
     / Heading
     / SectionLabel
     / EntityLine
     / ArrowLine
     / ConditionLine
     / EffectLine
     / ChoiceLine
     / BlockedMessage
     / EntityPresence
     / LineComment
     / Prose

Heading    ← PhaseHeading / SequenceHeading / LocationHeading
EntityLine ← EntitySpeech / StageDirection
ArrowLine  ← ExitDeclaration / ExitJump / Jump
EffectLine ← INDENT* '>' SP+ Effect EOL
```

Multi-line blocks (`OrConditionBlock`, `RuleBlock`) come first because they consume multiple lines and must not be pre-empted by single-line rules that match their opening sigil. `Prose` is always last — it is the fallback for any line that does not match a sigil. Sub-groups (`Heading`, `EntityLine`, `ArrowLine`) encode the internal ordering documented in the Ambiguity Resolution section.

Note: Each `Block` alternative is responsible for consuming its own `EOL`. Single-line blocks append `EOL` at the end of their rule. Multi-line blocks consume `EOL` for each of their internal lines. `Prose` consumes `Text EOL`. `EffectLine` is defined inline here for convenience; the `Effect` sub-rule it references is defined in the Effect Declarations section below. Similarly, `ConditionLine` references rules from the Condition Expressions section.

### Sub Rule Grammars

Several constructs have internal structure that the grammar must parse:

#### Condition Expressions

```
ConditionExpr    ← NarrativePropRef SP+ CompOp SP+ Value
               / EntityRef SP+ 'in' SP+ ContainerRef
               / EntityRef SP+ 'not' SP+ 'in' SP+ ContainerRef
               / SectionName '.exhausted'
CompOp           ← '==' / '!=' / '>' / '>=' / '<' / '<='
NarrativePropRef ← EntityProp / ReservedPropRef
EntityProp       ← '@' Identifier '.' Identifier
ReservedPropRef  ← ('player' / 'target') '.' Identifier
ContainerRef     ← EntityRef / 'here' / 'player'
Value            ← String / Number / Boolean / Identifier
```

Note: `EntityProp` requires the `@` prefix in narrative content conditions. The `@` is what distinguishes an entity property access from prose text. Two reserved words — `player` and `target` — are accepted without `@` via `ReservedPropRef`. This covers `player.knows_cell` in narrative conditions/effects and `target.state`, `target.chosen` in action target contexts (`-> any Type`). Generalised bare identifiers (e.g., `guard.mood`) are not valid — the `@` sigil remains the primary mechanism. Inside frontmatter, property references in type definitions use bare names (e.g., `requires: rusty_key`), but that is the frontmatter grammar, not the condition expression grammar. `NarrativePropRef` replaces `EntityProp` in all narrative-scope sub-rules: `ConditionExpr`, `SetEffect`, and `RevealEffect`. Rule-scoped sub-rules (`RuleCondition`, `RuleEffect`) use their own `RuleLHS` which already handles bare dotted identifiers.

#### Terminal Rules

Several rules above reference `Identifier`, `EntityRef`, and other terminals. Their definitions:

```
Identifier     ← [a-z] [a-z0-9_]*
EntityRef      ← '@' Identifier
SectionName    ← Identifier
TypeName       ← [A-Z] [a-zA-Z0-9]*
```

`Identifier` permits lowercase letters, digits, and underscores. It must start with a lowercase letter. **This is intentional style enforcement at the grammar level, not incidental grammar design.** Section labels (`== topics`), exit names (`-> harbor`), and entity IDs (`@door_1`) are all lowercase by grammar rule. Display names use `TypeName` (uppercase start) or `String` (quoted). If a future spec version wants mixed-case identifiers, this rule must change explicitly. `TypeName` starts with an uppercase letter (e.g., `Door`, `Barkeep`), distinguishing types from identifiers at the grammar level.

#### Lexical Tokens

The grammar references several lexical tokens used across rules. `SP`, `EOL`, and `NEWLINE` are defined in the File Structure section above. The remaining tokens:

```
Char               ← !'\t' !NEWLINE .
InlineCommentStart ← SP+ '//'
Text               ← (!InlineCommentStart Char)+ InlineComment?
TextRaw            ← Char+
InlineComment      ← InlineCommentStart TextRaw
StringChar         ← '\\"' / '\\\\' / !('"' / NEWLINE) .
String             ← '"' StringChar* '"'
Number             ← '-'? [0-9]+ ('.' [0-9]+)?
Boolean            ← 'true' / 'false'
```

`Char` matches any character except tab (`\t`) and newline. **Tab rejection mechanism:** tabs are rejected because `Char` explicitly excludes `\t`, and `Text`, `TextRaw`, and `String` are all built on `Char`. If `Char` is ever refactored, the tab exclusion must be preserved or moved to an equivalent guard. Tabs in indentation are separately rejected because `INDENT` is defined as exactly two space characters.

`Text` requires at least one character and stops before an inline comment boundary (`SP+ //`). This means empty text is invalid — a heading like `# ` with no title is a parse error, as is a choice like `* ` with no label. **No-empty-content policy:** this is a deliberate grammar-wide constraint. Every content-bearing construct (headings, choices, speech, stage directions, blocked messages, prose) requires at least one character of content. Implementers and tool authors must not reintroduce empty nodes. `TextRaw` also requires at least one character and is used only inside comments and in `LineComment`.

`String` forbids raw newlines. A string must open and close on the same line. Escaped quotes (`\\"`) and escaped backslashes (`\\\\`) are supported. Partial forms like `.5` or `5.` are not valid numbers — both sides of the decimal point are required. Scientific notation is not supported in v1. `Boolean` is the literal strings `true` and `false`.

Note: `FrontmatterBody` is part of the grammar artifact but is specified separately. The frontmatter subset grammar — covering the restricted YAML-like syntax for types, entities, properties, and imports — is complex enough to warrant its own sub-grammar section, to be written alongside the compiler's frontmatter parser. Until then, the Frontmatter Grammar section above defines the acceptance and rejection rules in prose, and the compiler treats frontmatter as a dedicated parser module. The PEG file includes a stub nonterminal for `FrontmatterBody` that is implemented by the dedicated frontmatter parser module, so the combined system is the normative validator. Frontmatter parsing is formally delegated: the `.peg` file defines `FrontmatterBody` as an opaque rule, and the compiler's frontmatter module is the normative implementation of that rule. The narrative content grammar defined in this document is complete and self-contained.


#### OR Condition Blocks

The `? any:` opener introduces a block of indented condition expressions. The inner lines are bare `ConditionExpr` without a leading `?` sigil. Any single inner condition being true validates the entire block.

```
OrConditionBlock ← INDENT* '?' SP+ 'any:' EOL OrConditionLine+
OrConditionLine  ← INDENT+ ConditionExpr EOL
```

`OrConditionBlock` is a multi-line construct. The header line consumes its own `EOL` after `any:`, and each `OrConditionLine` consumes its own `EOL`. This makes the block self-contained: each line rule owns its line ending, matching the pattern used by all other multi-line block constructs (see Rule Block below). **Integration rule:** `OrConditionBlock` is a `Block` alternative and is parsed within the same `Content ← Line*` loop. When the block finishes consuming its inner lines, control returns to `Content`, which resumes at the next `Line` normally.

Note: `OrConditionLine` uses `ConditionExpr`, not the full `Condition` rule. The inner lines have no `?` prefix. This matches the syntax spec where `? any:` is followed by indented bare expressions like `@guard.mood == hostile`.

#### Effect Declarations

```
Effect         ← SetEffect / MoveEffect / RevealEffect / DestroyEffect
SetEffect      ← NarrativePropRef SP+ '=' SP+ Value
             / NarrativePropRef SP+ ('+' / '-') SP+ Number
MoveEffect     ← 'move' SP+ EntityRef SP+ '->' SP+ ContainerRef
RevealEffect   ← 'reveal' SP+ NarrativePropRef
DestroyEffect  ← 'destroy' SP+ EntityRef
```

#### Rule Block

Rule blocks define constrained NPC behaviour. They are engineer-authored constructs that appear in narrative content (not frontmatter). A rule block is a multi-line construct: the `rule name:` header line consumes its own `EOL`, and each indented body line consumes its own `EOL`. This makes the block self-contained, matching the pattern used by `OrConditionBlock`.

The rule block body reuses existing sub-rules (entity references, conditions, effects) in a structured sequence: an actor performs a `selects ... from ... where` query, and the matching target receives effects.

```
RuleBlock      ← 'rule' SP+ Identifier ':' EOL RuleBody
RuleBody       ← RuleActorLine RuleWhereLine* RuleEffectLine+
RuleActorLine  ← INDENT EntityRef (SP+ 'selects' SP+ Identifier SP+ 'from' SP+ EntityIdList)? EOL
RuleWhereLine  ← INDENT 'where' SP+ RuleCondition EOL
RuleEffectLine ← INDENT '>' SP+ RuleEffect EOL
EntityIdRef    ← '@' Identifier
EntityIdList   ← '[' EntityIdRef (',' SP? EntityIdRef)* ']'
```

Each body line rule consumes its own trailing `EOL`. This makes `RuleBody` a simple sequence of self-contained line rules, consistent with how a PEG engine will see them and straightforward to port into pest or peggy.

Note: `RuleBody` uses `RuleWhereLine*` (zero or more) rather than `RuleWhereLine+`. A rule without a `selects` clause may still have direct effects with no `where` conditions. `RuleEffectLine+` requires at least one effect — a rule with no effects is meaningless.

Note: `EntityIdRef` is always `@`-prefixed. The keywords `here` and `player` are not valid inside a `selects from` list — the select block operates on declared entities, not runtime-resolved containers. `EntityIdList` uses `EntityIdRef` rather than the broader `EntityRef` to enforce this restriction at the grammar level.

The `selects` clause is optional on `RuleActorLine`. When absent, the rule has no select block (direct conditions and effects only, no bound variable). When present, the `Identifier` after `selects` is the variable name (e.g., `target`) scoped to the rule block.

**Rule-scoped conditions.** Narrative content conditions require the `@` prefix on entity property access (`@guard.mood == neutral`). Inside rule `where` clauses, the bound variable uses a bare dotted identifier (`target.prize != car`). To handle both forms without making the main `ConditionExpr` context-sensitive, rule blocks use a dedicated `RuleCondition` rule:

```
RulePropRef    ← Identifier '.' Identifier
RuleLHS        ← EntityProp / RulePropRef
RuleCondition  ← RuleLHS SP+ CompOp SP+ Value
              / EntityRef SP+ 'in' SP+ ContainerRef
              / EntityRef SP+ 'not' SP+ 'in' SP+ ContainerRef
              / SectionName '.exhausted'
```

`RuleLHS` is scoped to the rule block grammar. It extends the left-hand side of comparisons to accept both `@entity.prop` (via `EntityProp`) and `variable.prop` (via `RulePropRef`). The distinction between a bound variable and a bare identifier that happens to look like one is a semantic check (Phase 3), not a syntactic one. All other condition forms (`in`, `not in`, `.exhausted`) are unchanged. `RuleLHS` is not used in narrative-scope conditions — those continue to use `EntityProp` exclusively.

**Rule-scoped effects.** Effects inside rule blocks may reference the bound variable (`target.state = open`) rather than an `@`-prefixed entity. `RuleEffect` follows the same pattern as `Effect` but accepts `RuleLHS` where property access is needed, and `RuleRef` where an entity or bound variable is needed:

```
RuleRef            ← EntityRef / Identifier
RuleEffect         ← RuleSetEffect / RuleMoveEffect / RuleRevealEffect / RuleDestroyEffect
RuleSetEffect      ← RuleLHS SP+ '=' SP+ Value
                  / RuleLHS SP+ ('+' / '-') SP+ Number
RuleMoveEffect     ← 'move' SP+ RuleRef SP+ '->' SP+ ContainerRef
RuleRevealEffect   ← 'reveal' SP+ RuleLHS
RuleDestroyEffect  ← 'destroy' SP+ RuleRef
```

`RuleRef` accepts either an `@`-prefixed entity reference or a bare identifier (the bound variable). This allows `> move target -> here` and `> destroy target` inside rule blocks. `RuleLHS` handles dotted property access (`target.state`, `@entity.prop`) for set and reveal effects. `ContainerRef` is unchanged — move destinations are always concrete (`@entity`, `here`, `player`), never bound variables. `RuleRevealEffect` takes `RuleLHS` because reveal operates on properties (e.g., `> reveal target.prize`), not on entities directly.

**Example parsed by this grammar:**

```
rule monty_reveals:
  @monty selects target from [@door_1, @door_2, @door_3]
  where target.prize != car
  where target.chosen == false
  where target.state == closed
  > target.state = open
```

Parses as: `RuleBlock` with identifier `monty_reveals`, containing a `RuleActorLine` (`@monty` with `selects target from [...]` using `EntityIdList`), three `RuleWhereLine` entries using `RuleCondition` with `RulePropRef` on the left-hand side via `RuleLHS`, and one `RuleEffectLine` using `RuleSetEffect` with `RuleLHS`. Each body line consumes its own `EOL`.

**Integration rule:** `RuleBlock` is a `Block` alternative in the `Content ← Line*` loop. It is listed in the narrative content grammar table as `rule + SP + name + :`. When the block finishes consuming its indented body lines, control returns to `Content`, which resumes at the next `Line` normally. This follows the same pattern as `OrConditionBlock`.

#### Choice Lines

Choice targets appear at the end of the choice label line, not as separate jump lines. The full choice line grammar:

```
ChoiceLine     ← INDENT* ChoiceSigil SP+ ChoiceLabel ChoiceTarget? EOL
ChoiceSigil    ← '*' / '+'
ChoiceLabel    ← Text
ChoiceTarget   ← SP+ '->' SP+ TargetRef
TargetRef      ← '@' Identifier              // specific entity target
             / 'any' SP+ TypeName            // type target
             / Identifier                     // section or exit target
```

A choice with no target (e.g., `* Back off`) has its content nested as indented lines below it. A choice with a target (e.g., `* Pick a door -> any Door`) compiles to an action with `target_type`. Like all single-line `Block` rules, `ChoiceLine` consumes its own `EOL` at the end of the line, following the uniform convention that every line-level rule owns its own line ending.

#### Indentation

Indentation uses exactly two spaces per level. The grammar and the diagnostic layer handle depth differently:

**Grammar acceptance (what the parser accepts):**

```
INDENT         ← '  '                       // exactly two spaces
IndentLevel    ← INDENT{0,4}                // accept up to 4 levels for parsing
```

The grammar accepts up to 4 indent levels so the parser can produce a complete AST and then diagnose. A pure PEG grammar cannot emit a warning while still accepting, so depth enforcement is split into two layers.

**Diagnostic policy (what the compiler reports):**

| Depth | Behaviour |
|---|---|
| 0–2 | Valid. No diagnostic. |
| 3 | Warning: "Nesting depth 3 at line N. Consider breaking into a labeled section with `==` for readability." |
| 4 | Error: file does not compile. "Maximum nesting depth exceeded at line N." |

Tabs are always rejected at the grammar level. The parser does not accept `\t` as indentation.

#### Comments

Comments use `//` and come in two forms. Below the frontmatter, `//` is the comment marker (frontmatter uses `#`).

```
LineComment   ← INDENT* '//' TextRaw? EOL
```

`InlineComment` and `InlineCommentStart` are defined in the Lexical Tokens section above. They are integrated into `Text` itself, so any rule using `Text` automatically handles inline comments.

**`LineComment`** is a `Block` alternative. It allows optional leading indentation (`INDENT*`) so comments can be aligned with the surrounding content. `TextRaw?` is optional to permit bare `//` lines with no comment text. It uses `TextRaw` (not `Text`) because comment content should not be further parsed for nested comment boundaries. Like all `Block` rules, it consumes its own `EOL`.

**`InlineComment`** is **not** a `Block` alternative. It is a suffix consumed by `Text` at the end of any content-bearing line. For example, `@arina: What'll it be? // hub prompt` parses as `EntitySpeech` where the `Text` portion captures `What'll it be?` and the `InlineComment` suffix captures `// hub prompt`. The comment text is stripped during compilation and does not appear in JSON.

#### Auto Marker

The `(auto)` suffix on phase headings compiles to `auto: true`. It requires a space before the opening parenthesis and uses the exact string `auto`.

```
AutoMarker ← SP+ '(auto)'
```

`AutoMarker` is only valid on `PhaseHeading` (`###`). If `(auto)` appears on a `LocationHeading` (`#`) or `SequenceHeading` (`##`), the grammar should accept it as part of the heading text (it will be caught as a semantic warning in a later phase). This keeps the grammar simple and avoids making `(auto)` a globally reserved token.

---

## Quality Properties

A grammar file that exists is not enough. It must have the following properties to serve its purpose.

### Completeness

Every valid `.urd.md` construct must be parseable. Every invalid construct must fail. The danger zone is constructs that the grammar accepts but interprets differently from the author's intent. The grammar must resolve every ambiguity. For example, `@guard mood` must unambiguously parse as a stage direction, not as a malformed speech line missing a colon.

### Single Source of Truth

An engineer should be able to implement a correct parser by reading **only the grammar file**, without cross referencing the prose specs for edge cases. If someone has to ask "but what happens when..." and the answer is not in the grammar, that is a gap.

### Readability

PEG has a natural readability advantage. The rules should read almost like the syntax table on the website. Rule names should be self documenting: `OneShotChoice`, `EntitySpeech`, `ConditionExpr`. Comments in the grammar file explain the *why*, not just the *what*.

### Modularity

Each construct should be an independent, testable rule. The grammar for a condition expression should be self contained. Changing how effects work should not require touching the condition rules. This also matters for the LSP later: error recovery is easier when the parser can report "the ConditionExpr rule failed at character 14" rather than a generic parse failure.

### Error Friendliness

The structure of the grammar directly determines the quality of error messages. A great grammar is structured so that failure points are diagnostic. PEG's ordered choice operator means the parser knows which alternatives were tried and rejected. Rule grouping should make it possible to generate messages like:

```
Line 12: Expected entity speech (@id: text) or stage direction (@id text),
         but found '@guard' followed by '='. Did you mean '> @guard.mood = ...'?
```

### Writer/Engineer Boundary

The grammar should make the writer/engineer boundary visible in its structure. Writer constructs (dialogue, choices, conditions, effects, jumps) and engineer constructs (type definitions, rule blocks, hidden properties) should be clearly separated rule groups. This mirrors the separation in the syntax spec and makes the grammar navigable by both audiences.

## Validation Corpus

The grammar's quality is measured by how precisely it draws the line between accept and reject. This requires two sets of test files.

### Positive Corpus (Must Parse)

These files must parse without errors. They are derived from the four locked test cases and additional edge case files:

| File | Exercises |
|---|---|
| tavern.urd.md | Hub dialogue, sticky/one shot choices, conditions, effects, jumps, entity speech, stage directions, exhaustion fallthrough |
| monty-hall.urd.md | Frontmatter (types, entities), headings (location/sequence/phase), rule block, (auto) phases, action targets |
| key-puzzle.urd.md | Freeform world (no sequences), exit declarations, blocked messages, containment conditions (in/not in/here), move/destroy effects |
| interrogation.urd.md | OR conditions (? any:), bribe using containment, multi section jumps, conditional sub branches, import declaration |
| edge-cases.urd.md | Empty frontmatter, minimal file (frontmatter only), file with no frontmatter, quoted strings in frontmatter, inline comments, escaped characters |

### Negative Corpus (Must Fail)

These files must fail to parse, and the grammar should produce errors that point to the right location. Each file tests a specific rejection:

| File | Expected Failure |
|---|---|
| bad-anchor.urd.md | YAML anchor (`&name`) used in frontmatter. Error should say anchors are not supported. |
| bad-indent-tabs.urd.md | Tab character used for indentation. Error should say use two spaces. |
| bad-unclosed-frontmatter.urd.md | Opening `---` without closing `---`. Error should identify unclosed frontmatter. |
| bad-block-list.urd.md | Block style YAML list (`- item`) in frontmatter. |
| bad-malformed-entity.urd.md | Entity reference with invalid characters (`@Guard-1` with hyphen). |
| bad-empty-choice.urd.md | Choice line (`*` or `+`) with no label text. |
| bad-unknown-sigil.urd.md | Line starting with an unrecognised sigil in a context where it cannot be prose. |

### Semantic Negative Corpus (Must Fail Compilation, Not Parse)

Some invalid files are syntactically well-formed but semantically incorrect. The grammar should **parse them successfully** into an AST. The error is caught in a later compiler phase, not by the grammar. These files belong in the broader compiler test suite, not the grammar corpus:

| File | Expected Failure | Phase |
|---|---|---|
| bad-duplicate-section.urd.md | Two `==` sections with the same name in one file. | Link (phase 3) — duplicate detection is scope resolution, not syntax. |
| bad-depth-4.urd.md | Four levels of choice nesting. Grammar parses it; compiler emits error. | Validate (phase 4) — depth enforcement is diagnostic policy, not syntax. |
| bad-unresolved-ref.urd.md | `-> nonexistent_section` targets nothing. | Link (phase 3) — reference resolution. |
| bad-wrong-enum.urd.md | Property set to a value not in the declared enum. | Validate (phase 4) — type checking. |

This distinction matters: if the grammar rejects duplicate sections, it conflates parsing with semantic analysis and makes the grammar depend on cross-rule state that PEG cannot cleanly express.

> **Living corpus.** The test corpus grows as the grammar is implemented. Every bug found during compiler development should produce a new negative test file. The corpus is the grammar's regression suite.

## Relationship to the Compiler

The grammar is **phase 1** of the five phase compiler pipeline. Here is how it fits:

```
  .urd.md source
       │
  1. PARSE ←─── THE GRAMMAR DEFINES THIS PHASE
       │        Syntactic validity. Well formed lines.
       │        Produces a per-file AST.
       │
  2. IMPORT      Resolves import: declarations.
       │        Builds the dependency graph.
       │
  3. LINK        Merges scopes. Resolves @entity references,
       │        -> section jumps, property accesses.
       │
  4. VALIDATE    Type checks. Enum values in range.
       │        Ref targets exist and are correct type.
       │
  5. EMIT        AST → .urd.json
       │
  .urd.json + diagnostics
```

The grammar's scope is strictly phase 1. It produces an Abstract Syntax Tree (AST) where each node represents a parsed construct: a heading, a choice, a condition, a jump. The AST does **not** know whether references resolve. It does **not** know whether types exist. It knows the text is well formed.

### What the Grammar Validates vs What It Does Not

| Validates (Grammar / Phase 1) | Does Not Validate (Phases 2–4) |
|---|---|
| Line starts with a recognised sigil or is prose | `@guard` exists as a declared entity |
| Frontmatter uses allowed YAML subset | Imported file exists on disk |
| Indentation uses two spaces per level, parseable up to depth 4 | Depth policy (3 = warning, 4 = error) is diagnostic, not syntax |
| Choice has a label after `*` or `+` | Entity property is defined on the referenced type |
| Condition expression is syntactically valid | Enum value is in the declared value set |
| Effect declaration is syntactically valid | Condition references resolvable properties |
| Section name follows ID rules | Effect target exists and is the right type |
| Jump target is a valid identifier | Section names are unique (duplicate detection) |
| `(auto)` appears only on phase headings | Jump target resolves to a section or exit |
| Comment syntax is well formed | Sequence phases form a valid progression |

### Not a Separate Tool

The grammar is not a standalone validator you run independently of the compiler. It is the compiler's front end. Running the grammar alone (via a PEG parser generator) gives you syntactic validation, which is useful for editor integration and fast feedback. But production validation always runs the full pipeline.

That said, the grammar file should be usable with standard PEG tooling. A developer should be able to load `urd-schema-markdown.peg` into `pest` (Rust), `peggy` (JavaScript), or any PEG parser generator and get a working parser that accepts valid `.urd.md` files. This is the portability argument: the grammar is an interoperability artifact, not just an internal compiler detail.

## Relationship to the JSON Schema

The formalisation phase produces **two** artifacts that work together:

| Artifact | Validates | Format | When Used |
|---|---|---|---|
| PEG grammar (this brief) | Compiler input (`.urd.md`) | PEG file | Parse phase. Before compilation. |
| JSON Schema (separate brief) | Compiler output (`.urd.json`) | JSON Schema file | After compilation. CI checks. |

Together they form the machine checkable specification. The prose specs define what Urd means. The grammar and JSON Schema define what valid Urd looks like, for input and output respectively. The compiler connects them: valid input (per the grammar) → valid output (per the JSON Schema).

```
  .urd.md ──── PEG grammar says: is this well formed? ─────┐
               (syntactic check)                            │
                                                            ▼
           ┌─── COMPILER (phases 2–5) ────────────────┐
           │    import, link, validate, emit            │
           └────────────────┬──────────────────────────┘
                            │
                            ▼
  .urd.json ─── JSON Schema says: is this conformant? ──┐
                (structural check)                       │
                                                         ▼
                                                    ✓ VALID
```

## Error Recovery Expectations

PEG parsers fail at the first mismatch by default. This is acceptable for batch compilation (run the compiler, fix the error, run again) but insufficient for editor integration, where the user is mid-keystroke and the file is always temporarily invalid. The grammar must be designed with recovery in mind from the start, even though the LSP is a later phase.

### Batch Mode (Compiler CLI)

In batch mode, the parser should **collect all recoverable errors** rather than stopping at the first one. The strategy is synchronisation point recovery: when a rule fails, skip forward to the next recognisable line boundary (a newline followed by a known sigil or a blank line) and resume parsing. This produces a partial AST with error nodes marking the damaged spans.

The compiler already specifies this philosophy ("collect all errors, don't stop at the first one" from the architecture doc). The grammar should be structured to make this possible by ensuring every top-level line rule is independently recoverable.

### Editor Mode (LSP, Future)

In editor mode, the parser must produce a **partial AST on every keystroke**, even when the file is syntactically broken. The requirements:

- A malformed line should not prevent parsing the rest of the file. If line 12 is broken, lines 13 onward should still produce valid AST nodes.
- The error node for a broken line should capture: the raw text, the line number, and ideally which rule was being attempted when the failure occurred.
- Frontmatter errors should not prevent parsing the narrative content section, and vice versa.

### Design Implication

Each top-level rule (the 19 rules in the narrative content grammar table) should be wrapped in an error-recovering boundary. In PEG terms, this means the `Line` rule is not just `Block / BlankLine` but `Block / BlankLine / ErrorLine`, where `ErrorLine` consumes everything to the next newline and produces a diagnostic. This is a structural decision that affects the grammar's shape, not just its implementation.

### Synchronisation Rule

When the parser encounters a line that fails all `Block` alternatives, it must resynchronise. The resync procedure is:

1. Consume all characters to the next `EOL` (producing an `ErrorLine` node with the raw text and line number).
2. Consume whitespace (indentation) on the next line.
3. Inspect the first non-whitespace characters. If they begin with any of the sigil set — `@`, `?`, `>`, `*`, `+`, `==`, `->`, `!`, `#`, `//`, `rule `, `[` — treat the line as a new `Block` start and resume normal parsing.
4. If the next line does not start with a sigil, treat it as `Prose` and resume.

This ensures one broken line does not cascade into losing the rest of the file. The sigil set listed here must be kept in sync with the narrative content grammar table.

## Ambiguity Resolution via PEG Ordering

PEG's ordered choice operator (`/`) resolves ambiguity by trying alternatives left to right and committing to the first match. Several Schema Markdown constructs depend on this ordering. The grammar must encode these resolutions explicitly. Here are the critical cases.

### Speech vs Stage Direction

`@arina: What'll it be?` is speech. `@arina leans in close.` is a stage direction. Both start with `@identifier`. The distinguishing token is the colon.

```
EntityLine ← EntitySpeech / StageDirection
EntitySpeech  ← '@' Identifier ':' SP+ Text
StageDirection ← '@' Identifier SP+ Text
```

`EntitySpeech` must come first. If the parser tried `StageDirection` first, it would match `@arina` followed by a space and consume `: What'll it be?` as prose, producing an incorrect AST. The colon check in `EntitySpeech` prevents this.

### Choice vs Prose

A line starting with `*` could be a one-shot choice or prose containing an asterisk. PEG ordered choice resolves this: `OneShotChoice` and `StickyChoice` are tried before `Prose` in the `Block` rule, so a line starting with `*` or `+` after indentation is always a choice. There is no context-dependent scope tracking.

```
Block ← OneShotChoice / StickyChoice / ... / Prose
```

`Prose` is the final fallback. Any line that doesn't match a sigil rule is prose.

### Jump vs Exit Declaration

`-> topics` is a section jump. `-> north: Corridor` is an exit declaration. Both start with `->`. The distinguishing token is the colon after the target name.

```
ArrowLine ← ExitDeclaration / ExitJump / Jump
ExitDeclaration ← '->' SP+ Identifier ':' SP+ Text
ExitJump        ← '->' SP+ 'exit:' Identifier       // no space after colon
Jump            ← '->' SP+ Identifier
```

`ExitDeclaration` (with colon after a freeform identifier) must be tried before `Jump` (without colon), otherwise `-> north: Corridor` would be parsed as a jump to `north:` which is not a valid identifier. All three forms require a single space after `->`, consistent with every other arrow construct. `ExitJump` uses `exit:` as a reserved prefix with no space before the target name. Note: the target in `ExitJump` is an `Identifier`, which requires lowercase. `-> exit:harbor` is valid; `-> exit:Harbor` is a parse error. This is consistent with exit names being identifiers, not display names.

### Or-Condition Block vs Simple Condition

`? any:` is an OR condition block opener followed by indented bare expressions. `? @guard.mood == neutral` is a simple condition. Both start with `?`.

```
ConditionLine    ← OrConditionBlock / Condition
OrConditionBlock ← INDENT* '?' SP+ 'any:' EOL OrConditionLine+
OrConditionLine  ← INDENT+ ConditionExpr EOL
Condition        ← INDENT* '?' SP+ ConditionExpr
```

`OrConditionBlock` must come first because `any:` would otherwise be consumed as the start of a condition expression, failing at the colon. Note that the inner lines of the OR block use bare `ConditionExpr` without a `?` prefix, matching the syntax spec exactly.

### Heading Levels

`###`, `##`, and `#` must be tried longest-match-first:

```
Heading ← PhaseHeading / SequenceHeading / LocationHeading
PhaseHeading    ← '###' SP+ Text AutoMarker? EOL
SequenceHeading ← '##' SP+ Text EOL
LocationHeading ← '#' SP+ Text EOL
```

If `LocationHeading` were tried first, `## The Game` would match `#` and leave `# The Game` as text.

These five resolution cases should be documented as comments in the grammar file at the point where ordering matters. This makes the grammar self-documenting for future maintainers.

## Grammar Evolution and Versioning

The grammar is a normative artifact. Third party tools, AI systems, and future Urd implementations may depend on it. Changes to the grammar must be governed.

### Versioning Rule

The grammar version follows the Schema Markdown Syntax Specification version. When the syntax spec is `v0.1`, the grammar is `v0.1`. There is no independent grammar version. This ensures one-to-one correspondence: a specific grammar version parses exactly the syntax defined by the matching spec version.

The grammar file should declare its version in a header comment:

```
// Urd Schema Markdown Grammar v0.1
// Conforms to: Schema Markdown Syntax Specification v0.1
// Date: February 2026
```

### Compatibility Guarantees

- **Additive changes** (new syntax constructs in a future spec version) produce a new grammar version that is a strict superset of the previous one. A `v0.2` grammar accepts everything `v0.1` accepts, plus new constructs.
- **Breaking changes** (removing or changing the meaning of existing constructs) require a major version bump and are governed by the same policy as the spec: existing world files remain valid, new capabilities are additive.
- **Bug fixes** (the grammar incorrectly accepted or rejected something relative to the prose spec) are not version bumps. They are corrections. The grammar file's git history tracks these.

### Migration Expectations

When a new grammar version is released:

- The positive test corpus is extended with files exercising new constructs.
- The existing positive corpus must still pass (backward compatibility).
- Parser generators using the old grammar will reject new-syntax files with a parse error. This is expected and not a bug. The error should ideally suggest updating the grammar version.
- The compiler should report which grammar version it implements, and when it encounters a file that uses syntax from a newer version, suggest upgrading.

## Acceptance Criteria

The grammar is complete when all of the following are true:

1. **All four test case files parse successfully.** Tavern, Monty Hall, Key Puzzle, and Interrogation `.urd.md` files produce ASTs with no errors.
2. **All negative corpus files fail with correct errors.** Each bad file produces an error pointing to the expected line and construct.
3. **The grammar is self contained.** An engineer can implement a parser from the PEG file alone, without reading the prose specs.
4. **Rules are modular.** Each construct is an independent rule. Changing the condition expression grammar does not affect the choice grammar.
5. **Writer/engineer boundary is visible.** Rule groups are clearly separated and commented.
6. **Error positions are actionable.** When a parse fails, the failure point maps to a specific line and column, and the failing rule name suggests what was expected.
7. **The grammar works with standard PEG tooling.** It can be loaded into at least one PEG parser generator (pest, peggy, or equivalent) and produces a working parser.

## Delivery

| Artifact | Location | Notes |
|---|---|---|
| `urd-schema-markdown.peg` | Repository root or `packages/grammar/` | The grammar file. Versioned. |
| Positive test corpus | `tests/fixtures/valid/` | The four test case `.urd.md` files plus edge cases. |
| Negative test corpus | `tests/fixtures/invalid/` | One file per expected rejection. |
| Validation script | `scripts/` or CI | Runs both corpora against the grammar. Pass/fail. |

The grammar ships as part of the formalisation phase alongside the JSON Schema. Together they are the machine readable bridge between the locked specifications and the compiler implementation.

*End of Brief*
