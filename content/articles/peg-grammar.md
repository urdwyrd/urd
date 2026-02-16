---
title: The Input Gate
slug: peg-grammar
description: A PEG grammar that validates what writers produce — the input-side counterpart to the JSON Schema.
date: "2026-02-16"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> How and why the project built a formal grammar for Schema Markdown.
> Single canonical copy. February 2026.

## The first code in the monorepo

Before any compiler, before any runtime, the project needed to answer a basic question: *what does valid Schema Markdown actually look like?* The prose specifications define it in English. English is ambiguous. A PEG grammar is not.

The grammar is the first code package in the repository. It lives in `packages/grammar/` as a Rust crate using [pest](https://pest.rs), a PEG parser generator. It does one thing: given a `.urd.md` file, it either accepts or rejects it, and if it rejects, it tells you the exact line, column, and rule where the problem is.

This is the input-side counterpart to the [JSON Schema](/articles/json-schema). Together they define the machine-checkable boundaries of Urd:

```
  .urd.md ── PEG grammar: is this well-formed? ────┐
              (this article)                         │
                                                     ▼
          ┌─── COMPILER ──────────────────────────────────┐
          │    parse → resolve → link → validate → emit    │
          └────────────────────────────────────────────────┘
                           │
                           ▼
  .urd.json ── JSON Schema: is this conformant? ───┐
               (structural check)                   │
                                                    ▼
                                               ✓ VALID
```

## What it parses

The grammar covers the complete Schema Markdown syntax specification. About 75 rules across six categories:

**File structure.** Optional YAML frontmatter between `---` delimiters, followed by narrative content. Frontmatter is opaque at the grammar level — it validates the delimiters and consumes the block, but does not parse YAML. That is the compiler's job.

**Narrative blocks.** The writer-facing constructs that make up the body of a `.urd.md` file:

- Headings — location (`#`), sequence (`##`), phase (`###`, with optional `(auto)`)
- Section labels (`==`)
- Entity speech (`@entity: text`) and stage directions (`@entity text`)
- Entity presence (`[@entity, @entity]`)
- Choices — one-shot (`*`) and sticky (`+`), with optional targets (`-> @entity`, `-> any Type`, `-> section`)
- Conditions (`?`) with comparisons, containment checks, and OR blocks (`? any:`)
- Effects (`>`) — set, move, reveal, destroy
- Jumps (`-> section`, `-> exit:name`, `-> end`)
- Blocked messages (`!`)
- Comments (`//` inline or full-line)
- Prose — the fallback for unstructured narrative text

**Condition expressions.** Comparisons (`@entity.property == value`), containment (`@entity in player`, `@entity not in here`), reserved properties (`player.property`, `target.property`), and section exhaustion (`section.exhausted`).

**Effects.** Set (`> @entity.property = value`), move (`> move @entity to container`), reveal (`> reveal @entity.property`), destroy (`> destroy @entity`).

**Rule blocks.** The engineer-facing multi-line constructs for NPC behaviour: actor declarations, bound variables, conditions with entity references, and effects that mirror the narrative scope.

## Why PEG

PEG (Parsing Expression Grammar) was the right choice for three reasons.

**No ambiguity.** PEG uses ordered choice — the first matching alternative wins. There is no ambiguity to resolve at parse time. This matters for a writer-facing format where the same character (`*`, `#`, `@`) introduces different constructs depending on context.

**Readable mapping.** The grammar reads as a direct translation of the prose specification. A rule like `ChoiceLine ← INDENT* ('*' / '+') SP+ Text` maps visually to what a writer types. The reference `.peg` file is 268 lines including comments — small enough to read in one sitting.

**Precise errors.** When a parse fails, pest reports the exact line, column, and the rule that could not match. For a system that will run in clinical settings, "line 7, column 3: expected Text after choice sigil" is vastly more useful than "parse error."

## Five ambiguities resolved

The most interesting part of translating prose to formal grammar was discovering where the English was ambiguous. Five orderings in the grammar's `Block` rule are critical:

1. **Phase before sequence before location.** `###` (phase) must be tried before `##` (sequence) before `#` (location). PEG's ordered choice makes this trivial, but getting the order wrong would silently misparse every heading.

2. **Entity speech before stage direction.** `@entity: text` (speech, with colon) must be tried before `@entity text` (stage direction, without colon). Both start with `@entity`.

3. **OR condition block before single condition.** `? any:` (multi-line OR block) must be tried before `? expression` (single condition). Both start with `?`.

4. **Exit declaration before exit jump before plain jump.** `-> north: Corridor` (exit with destination type) before `-> exit:name` (exit reference) before `-> section` (plain jump). All start with `->`.

5. **Sigil lines before prose.** Choice lines (`*`/`+`), conditions (`?`), effects (`>`), and other sigil-prefixed constructs must all be tried before the prose fallback. Without a `SigilPrefix` guard on prose, a malformed `* ` (choice sigil with no label) would silently fall through as prose text instead of failing.

These five orderings are documented in the grammar with comments explaining why each matters.

## The tab ban

Schema Markdown rejects tabs everywhere. Not just in narrative content — in frontmatter too. This is enforced at the lowest level of the grammar: the `Char` rule, which is used in every text-consuming production, includes `!'\t'` as a negative lookahead. A single tab character anywhere in a `.urd.md` file fails the entire parse.

This is deliberate. Schema Markdown uses two-space indentation for choice nesting. Mixing tabs and spaces in a writer-facing format is a source of invisible, maddening bugs. The grammar makes the decision absolute and global.

## The test corpus

The grammar ships with 12 tests: 7 positive and 5 negative.

### Positive fixtures

The four locked test scenarios from the syntax specification, each exercising a different facet of the language:

- **Tavern** — hub dialogue with sticky and one-shot choices, conditions, effects, jumps, entity speech, stage directions, entity presence, exhaustion fallthrough
- **Monty Hall** — sequences with phases, rule blocks, auto phases, action targets, reveal effects, property comparisons
- **Key Puzzle** — exit declarations with blocked messages, containment conditions, move and destroy effects, conditional navigation
- **Interrogation** — OR condition blocks, multi-section jumps, bribery mechanics, import declarations

Plus three edge-case files that isolate specific syntax features: frontmatter with quoted strings and escapes, inline comments in various positions, and structural elements (all heading levels, section labels, exit jumps).

### Negative fixtures

Five files, each testing a specific rejection:

| File | What it rejects | Where |
|------|----------------|-------|
| `bad-tabs.urd.md` | Tab character | Line 1, column 1 |
| `bad-unclosed-frontmatter.urd.md` | Missing closing `---` | EOF |
| `bad-malformed-entity.urd.md` | `@Guard-1` (uppercase, hyphen) | Line 7, column 2 |
| `bad-empty-choice.urd.md` | `* ` with no label | Line 7, column 3 |
| `bad-empty-heading.urd.md` | `# ` with no title | Line 1, column 3 |

Each negative test asserts the exact error position. This is not just "it fails" — it is "it fails at precisely the right place, for precisely the right reason."

## Implementation

The grammar exists in two forms:

- **Reference:** [`urd-schema-markdown.peg`](https://github.com/urdwyrd/urd/blob/main/packages/grammar/urd-schema-markdown.peg) — standard PEG notation, 268 lines, the normative specification
- **Implementation:** `src/urd_schema_markdown.pest` — pest adaptation with `SOI`/`EOI`, atomic rules, silent rules, and documented deviations

The public API is a single function:

```rust
pub fn parse(input: &str) -> Result<Pairs<'_, Rule>, Error<Rule>>
```

No AST construction. No semantic analysis. Just parse success or failure with structured error reporting. AST types belong in the compiler, which will depend on this crate.

The grammar required three iterations to pass all 12 tests. Key fixes included handling non-progressing repetitions in pest's `BlankLine` rule, adding `INDENT*` to all block rules to support indentation under choices, and adding a `SigilPrefix` guard to the prose fallback. Every fix was back-ported from the `.pest` implementation to the `.peg` reference to keep them synchronised.

## Where it lives

- **Grammar package:** [`packages/grammar/`](https://github.com/urdwyrd/urd/tree/main/packages/grammar)
- **Reference PEG:** [`urd-schema-markdown.peg`](https://github.com/urdwyrd/urd/blob/main/packages/grammar/urd-schema-markdown.peg)
- **Test command:** `pnpm grammar:test` (or `cargo test --manifest-path packages/grammar/Cargo.toml`)

All 12 tests pass.

## What comes next

The grammar validates syntax. It cannot validate semantics — it does not know whether `@guard` references an entity that exists, or whether `-> tavern/topics` points to a real section. That is the compiler's job.

The compiler will use this grammar as its Phase 1 (parse). The grammar crate becomes a dependency. The parse tree it produces will be walked to build an AST, resolve imports, link cross-references, validate types, and emit the `.urd.json` that the [JSON Schema](/articles/json-schema) validates.

The input gate and the output contract are both in place. The next step is connecting them.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
