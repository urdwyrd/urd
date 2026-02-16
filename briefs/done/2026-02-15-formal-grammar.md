# Brief: Urd Schema Markdown Formal Grammar

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-16
**Status:** Done

### What was done

1. **Reconciled spec conflicts** in `docs/urd-formal-grammar-brief.md`: added `NarrativePropRef` and `ReservedPropRef` rules for `player.property` and `target.property` access; added explicit `SP+` between all expression tokens; removed contradictory ChoiceLine EOL note; softened Choice vs Prose context sensitivity language.
2. **Created `packages/grammar/`** — first Rust crate in the monorepo with `Cargo.toml` (pest 2.7, pest_derive 2.7).
3. **Extracted positive test corpus** — 4 locked test cases from syntax spec (tavern, monty-hall, key-puzzle, interrogation) plus 3 edge-case files (frontmatter, comments, structure). 7 files total.
4. **Created negative test corpus** — 5 invalid files (bad-tabs, bad-unclosed-frontmatter, bad-malformed-entity, bad-empty-choice, bad-empty-heading).
5. **Wrote reference PEG grammar** — `urd-schema-markdown.peg` with all rules, ambiguity comments, writer/engineer separation, version header.
6. **Translated to pest** — `src/urd_schema_markdown.pest` with pest-specific adaptations (SOI/EOI, atomic/silent rules, NEWLINE built-in).
7. **Implemented parser** — `src/lib.rs` with `#[derive(Parser)]` and public `parse()` function.
8. **Wrote test runner** — `tests/corpus.rs` with 12 tests (7 positive, 5 negative) with line/column assertions.
9. **Iterated to green** — 3 iterations to pass all 12 tests. Key fixes: BlankLine non-progressing repetition, INDENT* on all block rules, InlineComment on structured rules, SigilPrefix guard on Prose, File rule enforcement for unclosed frontmatter.
10. **Back-ported all fixes** to the `.peg` reference file.
11. **Created validation script** — `scripts/validate.sh`.
12. **Updated root files** — `package.json` (grammar:test script), `CLAUDE.md` (layout, commands, tech stack), `.gitignore` verified.
13. **Synced** `content/documents/urd-formal-grammar-brief.md` with reconciled docs version.

### What changed from the brief

- **`bad-empty-heading.urd.md` added** as a 5th negative test (brief specified 5 but only listed 4 non-deferred tests; empty heading was a natural complement to empty choice).
- **Edge cases split into 3 files** instead of 1 (`edge-cases-frontmatter`, `edge-cases-comments`, `edge-cases-structure`) per the brief's own recommendation for easier debugging.
- **Validation script simplified** — outputs cargo test results directly rather than per-fixture status lines. The 12/12 summary is still printed.
- **File rule changed** from `Frontmatter? Content EOF` to `Frontmatter Content EOF / !('---' EOL) Content EOF` — if a file starts with `---`, it MUST have valid frontmatter. This was necessary for `bad-unclosed-frontmatter` to fail correctly.
- **SigilPrefix guard added to Prose** — prevents malformed sigil lines (e.g., `* \n`, `# \n`) from silently falling through as prose. Not in original brief but essential for negative test correctness.
- **FrontmatterLine rejects tabs** — `FrontmatterLine ← !('---' (EOL / EOF)) (!'\t' !NEWLINE .)* EOL` ensures the tab ban is global, not just narrative content.
- **Cargo.lock not committed** per brief's locked convention (no root workspace Cargo.toml yet).

---

**Created:** 2026-02-15

## Context

The Urd framework has two locked prose specifications: the **Schema Markdown Syntax Specification** (what valid `.urd.md` looks like) and the **World Schema Specification** (what valid `.urd.json` looks like). Prose is readable but ambiguous. The next step is to produce a **formal PEG grammar** that removes the ambiguity and becomes the machine-readable definition of valid `.urd.md` input.

The complete design rationale, every PEG rule, ambiguity resolution, error recovery strategy, and acceptance criteria are documented in `docs/urd-formal-grammar-brief.md` (679 lines). **That document is the specification. This brief is the execution plan.** Read the formal grammar brief in full before starting.

This is a pivotal task: it creates the first code package in the monorepo, establishes the `packages/` directory convention, introduces Rust tooling, and produces the foundational artifact that the compiler will depend on.

## Decisions (Locked)

These decisions were made during brief preparation and are not open for re-evaluation:

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Package structure | **Option C** — reference `.peg` file in `packages/grammar/`, compiler in `packages/compiler/` later | Grammar is a standalone reference artifact; multiple implementations can depend on it |
| Parser generator | **pest** (Rust) | The reference `.peg` file uses standard PEG notation; translation to `.pest` is mechanical |
| Test case extraction | **In scope** | Extracting the four test cases from the syntax spec into standalone `.urd.md` files is a prerequisite |
| AST type definitions | **Deferred** | AST types belong in the compiler brief, not the grammar brief |
| `-> end` handling | **Regular Jump** | `end` is parsed as an `Identifier`; semantic handling is Phase 3+ |
| FrontmatterBody | **Opaque rule** | The `.peg` file defines it as a stub; the compiler's frontmatter module is the normative implementation. **Stub definition:** consume any characters until a line consisting of exactly `---` followed by `EOL` (or `EOF`). The stub must treat `---` alone on a line as the closing delimiter and must not accept it as body content. This ensures `bad-unclosed-frontmatter.urd.md` fails correctly even with the stub. |
| `.peg` is normative | **`.peg` file is the reference; `.pest` is the implementation** | If the two diverge due to pest-specific adaptations, the `.peg` file wins. All pest-specific deviations must be documented with comments in the `.pest` file explaining what differs and why. |
| Expression whitespace | **`SP+` between all tokens, including after sigils** | Sub-expression rules (`ConditionExpr`, `SetEffect`, `MoveEffect`, etc.) and sigil rules (`@`, `?`, `>`, `*`, `+`, `==`, `->`, `#`) all accept one or more spaces. The syntax spec examples show exactly one space, but enforcing exactly one would cause hard parse failures on accidental double spaces — unfriendly for writers. **Exceptions where no space is permitted:** `exit:name` (no space after colon), `@identifier` (no space after `@`), `entity.property` (no space around dot). |
| EOL ownership | **Every single-line Block rule consumes its own `EOL`** | The ChoiceLine note saying EOL is excluded is overridden. Multi-line blocks (`OrConditionBlock`, `RuleBlock`) consume multiple EOLs as documented in the grammar brief. |
| Choice context sensitivity | **No context sensitivity** | PEG ordered choice resolves `*` and `+`: `OneShotChoice` and `StickyChoice` are tried before `Prose` in the Block rule, so a line starting with `*` or `+` (after optional indentation) is always parsed as a choice, everywhere in narrative content. There is no context-dependent scope tracking. |
| `player`/`target` property access | **`NarrativePropRef` — see Known Issue #1 resolution** | `player` and `target` are reserved words accepted without `@` in narrative conditions and effects. Generalised bare identifiers are not accepted. |
| Tab rejection | **Tabs are never valid anywhere** | Tab characters (`\t`) are rejected in both narrative content and frontmatter. The grammar defines `Char ← !'\t' !NEWLINE .` and all content tokens (`Text`, `TextRaw`, `String`) are built on `Char`. This is a grammar-level ban, not a diagnostic. A tab anywhere in the file is a parse error. |
| Inline comment boundary | **`SP+ '//'` required** | An inline comment requires at least one space before `//`. The sequence `@arina: hi//x` is **not** a comment — it is parsed as speech text containing `//x`. Only `@arina: hi //x` triggers the inline comment rule. This matches the formal grammar brief's `InlineComment ← SP+ '//' TextRaw` definition. Whole-line comments (`//` at the start of a line, possibly after indentation) do not require a preceding space. |

## Scope

### In scope

1. **Reconcile spec conflicts** — resolve `player`/`target` property access across the formal grammar brief and syntax spec before writing any code (Step 1)
2. **Extract positive test corpus** — four locked test cases from the syntax spec into standalone `.urd.md` files, plus edge-case files
3. **Create negative test corpus** — five invalid `.urd.md` files, one per expected rejection (frontmatter-specific rejections deferred)
4. **Write the reference `.peg` grammar file** — standard PEG notation, complete narrative content grammar, `FrontmatterBody` as an opaque rule
5. **Set up `packages/grammar/`** — Rust crate with Cargo.toml, pest dependency, source structure
6. **Translate `.peg` to `.pest` format** — mechanical translation of the reference grammar into pest syntax
7. **Implement parser** — pest parser that passes the full validation corpus
8. **Validation script** — runs both corpora against the parser, pass/fail output

### Out of scope

- Compiler implementation (phases 2–5)
- AST type definitions and AST node structures
- JSON Schema for `.urd.json` output (separate formalisation artifact)
- LSP or editor integration
- FrontmatterBody sub-grammar (delegated to compiler)
- Frontmatter-specific negative tests (`bad-anchor`, `bad-block-list`) — deferred until FrontmatterBody is implemented in the compiler brief
- CI pipeline integration (can be a follow-up)

## Known Issues to Resolve During Implementation

### 1. `player.property` and `target.property` without `@` prefix

**Status: Resolution locked. Requires spec update before code.**

The grammar defines `EntityProp ← '@' Identifier '.' Identifier` and states bare identifiers are not valid in conditions. But the locked syntax spec examples use:

- `player.knows_cell` in conditions and effects (interrogation example)
- `target.state`, `target.chosen` in conditions and effects under `-> any Type` choices (Monty Hall example)

**Resolution:** Introduce `NarrativePropRef` as a new grammar construct:

```
ReservedPropRef  ← ('player' / 'target') '.' Identifier
NarrativePropRef ← EntityProp / ReservedPropRef
```

Apply `NarrativePropRef` everywhere `EntityProp` currently appears in narrative-scope sub-rules: `ConditionExpr`, `SetEffect`, `RevealEffect`, and any other effect that takes `EntityProp`. Do **not** generalise to all bare identifiers — the `@` sigil remains the primary mechanism for distinguishing entity property access from prose text.

This is a **spec-level reconciliation**: update the formal grammar brief to include `NarrativePropRef` and `ReservedPropRef` as part of Step 1 before writing any grammar code. **Once reconciled, the updated formal grammar brief becomes the single source of truth for all subsequent steps. Do not reference the pre-reconciliation version.**

### 2. Inter-token spacing in sub-expressions

**Status: Resolution locked.**

PEG does not handle whitespace implicitly. The grammar brief's sub-expression rules omit explicit spacing tokens between components, but all examples require spaces: `@guard.mood == neutral`, `@arina.trust + 5`, `move @key -> player`.

**Resolution:** Use `SP+` (one or more spaces) between all tokens, including after sigils. This is more forgiving than exactly one space and avoids hard parse failures from accidental double spaces — important for a writer-facing syntax. The `.peg` and `.pest` files must include explicit `SP+` between every token pair in `ConditionExpr`, `SetEffect`, `MoveEffect`, `RevealEffect`, `DestroyEffect`, `RuleCondition`, `RuleEffect`, and after all sigil characters (`?`, `>`, `*`, `+`, `==`, `->`, `#`). **Exceptions where no space is permitted:** `exit:name` (colon binds tightly), `@identifier` (`@` binds tightly to the identifier), `entity.property` (dot binds tightly).

### 3. EOL ownership for single-line Block rules

**Status: Resolution locked.**

The File Structure section says "every Block rule owns its own EOL" but the ChoiceLine section says EOL is not included.

**Resolution:** The File Structure convention is correct. Every single-line Block rule consumes its own `EOL`. Apply this consistently and ignore the contradictory ChoiceLine note. The formal grammar brief will be updated to remove the contradiction as part of Step 1.

## Detailed Steps

### Step 1: Reconcile spec conflicts and update the formal grammar brief

Before writing any code, update `docs/urd-formal-grammar-brief.md` to resolve the three known issues:

1. Add `ReservedPropRef` and `NarrativePropRef` rules. Apply `NarrativePropRef` to all narrative sub-rules that currently use `EntityProp`, including `RevealEffect`.
2. Add explicit `SP+` between expression tokens in all sub-expression rules.
3. Remove the contradictory EOL note from the ChoiceLine section.
4. Soften the Choice vs Prose "context sensitivity" language to: "PEG ordered choice resolves this: `OneShotChoice` and `StickyChoice` are tried before `Prose`, so a line starting with `*` or `+` after indentation is always a choice."

**Output:** Updated formal grammar brief with no internal contradictions. This updated document is the single source of truth for all subsequent steps.

### Step 2: Create `packages/grammar/` package structure

```
packages/grammar/
├── Cargo.toml
├── src/
│   ├── lib.rs                          # Public API: parse function, error types
│   └── urd_schema_markdown.pest        # pest grammar (translated from .peg)
├── tests/
│   ├── valid/                          # Positive corpus
│   │   ├── tavern.urd.md
│   │   ├── monty-hall.urd.md
│   │   ├── key-puzzle.urd.md
│   │   ├── interrogation.urd.md
│   │   ├── edge-cases-frontmatter.urd.md
│   │   ├── edge-cases-comments.urd.md
│   │   └── edge-cases-structure.urd.md
│   ├── invalid/                        # Negative corpus
│   │   ├── bad-tabs.urd.md
│   │   ├── bad-unclosed-frontmatter.urd.md
│   │   ├── bad-malformed-entity.urd.md
│   │   ├── bad-empty-choice.urd.md
│   │   └── bad-empty-heading.urd.md
│   └── corpus.rs                       # Test runner: parse all valid, reject all invalid
├── scripts/
│   └── validate.sh                     # Validation script
└── urd-schema-markdown.peg             # Reference grammar (standard PEG notation)
```

**Cargo.toml** should declare:
- `name = "urd-grammar"`
- `edition = "2021"`
- `pest` and `pest_derive` as dependencies
- No other dependencies

### Step 3: Extract positive test corpus

Extract the four test cases from the syntax spec (`docs/schema-markdown.md`) into standalone `.urd.md` files. Each file must be a complete, valid `.urd.md` source — not a code block excerpt. Cross-reference each against the syntax spec to ensure fidelity:

| File | Source | Key constructs exercised |
|------|--------|--------------------------|
| `tavern.urd.md` | Syntax spec §Example 1: A Dialogue Scene | Hub dialogue, sticky/one-shot choices, conditions, effects, jumps, speech, stage directions, exhaustion fallthrough, entity presence |
| `monty-hall.urd.md` | Syntax spec §Example 2: The Monty Hall Problem | Frontmatter (types with traits, entities with inline objects, flow lists), headings (location/sequence/phase), rule block, `(auto)` phases, action targets (`-> any Type`), `reveal` effects, `target.property` access |
| `key-puzzle.urd.md` | Syntax spec §Example 3: The Two-Room Key Puzzle | Freeform world (no sequences), exit declarations, blocked messages, containment conditions (`in`/`not in`/`here`), `move`/`destroy` effects, action targets (`-> @entity`) |
| `interrogation.urd.md` | Syntax spec §Example 4: The Interrogation (Stress Test) | OR conditions (`? any:`), bribe using containment, multi-section jumps, conditional sub-branches, `import` declaration, `move` effect with entity container, `player.property` access |

**Edge case files** (split by concern for easier debugging):

**`edge-cases-frontmatter.urd.md`** must exercise:
- File with no frontmatter (narrative content only)
- Empty frontmatter (`---` / `---` with nothing between)
- Minimal file (frontmatter only, no narrative)
- Quoted strings in frontmatter (`"1"`, `"true"`)
- Escaped characters in strings (`\"`, `\\`)

**`edge-cases-comments.urd.md`** must exercise:
- Inline comments on content lines (`@arina: text // comment`)
- Bare `//` comment line with no text after the sigil
- Indented comment lines
- Comments after choices, conditions, and effects

**`edge-cases-structure.urd.md`** must exercise:
- All heading levels in one file (`#`, `##`, `###`, `### with (auto)`)
- Heading with multiple spaces after sigil (`#  Title` — valid under `SP+` rule)
- `-> exit:name` explicit exit jump
- Multiple sections with `== name`
- `-> end` as a regular jump
- Entity presence list with multiple entities (`[@arina, @barrel]`)
- Stage direction (`@arina leans in close.`)
- Blocked message (`! The door is locked.`)

### Step 4: Create negative test corpus

Each file tests exactly one rejection. Include a comment at the top of each file explaining what it tests and where the error should occur.

| File | What it tests | Expected failure point |
|------|---------------|----------------------|
| `bad-tabs.urd.md` | Tab character anywhere in the file (indentation, inline, frontmatter) | Parse failure — tab is excluded by `Char` rule, rejected everywhere. The file should include tabs in at least two positions (e.g., one in frontmatter, one in narrative content) to confirm the ban is global, not context-specific. Only the first tab needs to trigger the failure. |
| `bad-unclosed-frontmatter.urd.md` | Opening `---` without closing `---` | EOF reached while parsing frontmatter. The opaque `FrontmatterBody` stub is defined as "consume any characters until the closing `---` line." Without a closing delimiter, the stub consumes to EOF and the `Frontmatter` rule fails because the closing `---` is never matched. |
| `bad-malformed-entity.urd.md` | Entity reference with uppercase and hyphen (`@Guard-1`) | `Identifier` rule rejects uppercase start and hyphen |
| `bad-empty-choice.urd.md` | Choice line with no label (`* ` followed by newline) | `Text` requires at least one character |
| `bad-empty-heading.urd.md` | Heading with no text (`# ` followed by newline) | `Text` requires at least one character |

**Deferred negative tests** (require FrontmatterBody implementation):
- `bad-anchor.urd.md` — YAML anchor (`&name`) in frontmatter
- `bad-block-list.urd.md` — block style YAML list (`- item`) in frontmatter

These cannot be tested while `FrontmatterBody` is an opaque stub that consumes everything until `---`. They will be added when the compiler brief implements frontmatter parsing.

**Removed from original plan:**
- `bad-unknown-sigil.urd.md` — a line starting with `%` or `&` would match the `Prose` fallback rule and parse successfully, so it cannot be a negative test case.

### Step 5: Write the reference `.peg` file

Write `urd-schema-markdown.peg` in standard PEG notation, following the **reconciled** formal grammar brief (updated in Step 1). The file must:

1. Declare version in a header comment (`v0.1`, February 2026)
2. Define every rule from the grammar brief: File, Frontmatter, Content, Line, Block (ordered), all narrative line forms from the grammar table, all sub-rules
3. Include `FrontmatterBody` as an opaque rule with a comment explaining delegation
4. Document ambiguity resolutions as comments at the point where ordering matters (the five cases from the brief)
5. Separate writer-facing and engineer-facing rule groups with section comments
6. Include the `NarrativePropRef` and `ReservedPropRef` rules from the reconciliation
7. Include explicit `SP+` between expression tokens in all sub-expression rules
8. Be usable with any standard PEG parser generator — no tool-specific syntax

### Step 6: Translate to pest format

Mechanically translate `urd-schema-markdown.peg` into `src/urd_schema_markdown.pest`. pest uses `~` for sequences, `|` for ordered choice, and has its own syntax for character classes. The translation should:

- Preserve rule names
- Preserve comments explaining ambiguity resolution
- Use pest's `WHITESPACE` rule only if appropriate (likely not — Schema Markdown whitespace is significant)
- Mark rules as `silent` (`_`) where appropriate for tokens that should not produce parse tree nodes (e.g., `SP`, `EOL`, `INDENT`)
- **Document any pest-specific adaptations** with a comment explaining how the `.pest` rule differs from the `.peg` rule and why. The `.peg` file remains normative; the `.pest` file is the implementation.

### Step 7: Implement the parser

In `src/lib.rs`:

1. `#[derive(Parser)]` with `#[grammar = "urd_schema_markdown.pest"]`
2. A public `parse(input: &str) -> Result<Pairs<Rule>, Error>` function
3. Error types that include line number, column, and rule name where applicable. The grammar should be structured so failures land on meaningful leaf rules where tests expect them — use silent parent rules (`_` prefix in pest) when a wrapper rule would otherwise obscure the actual failure point.
4. No AST construction — just parse success/failure with error positions

### Step 8: Write the test runner

In `tests/corpus.rs`:

**Positive tests** (parse succeeds):
1. `valid_tavern`
2. `valid_monty_hall`
3. `valid_key_puzzle`
4. `valid_interrogation`
5. `valid_edge_cases_frontmatter`
6. `valid_edge_cases_comments`
7. `valid_edge_cases_structure`

**Negative tests** (parse fails with correct error position — see assertion table below for per-test criteria):
8. `invalid_tabs` — assert line and column
9. `invalid_unclosed_frontmatter` — assert line, column, and rule name from allowed set in table
10. `invalid_malformed_entity` — assert line, column, and rule name from allowed set in table
11. `invalid_empty_choice` — assert line, column, and rule name from allowed set in table
12. `invalid_empty_heading` — assert line, column, and rule name from allowed set in table

Each positive test reads the fixture file at runtime, attempts to parse it, and asserts success. Each negative test reads the fixture file, attempts to parse it, asserts failure, and verifies the error includes **line number, column number, and rule name where applicable**. Since the fixture files are controlled by this project, column assertions should be precise. The per-test assertion table below defines exactly what each test checks.

**Fixture-to-test mapping:** Test function names use `valid_` / `invalid_` prefixes and snake_case descriptions. Fixture files use `bad-` prefixes and kebab-case. The names do not need to match literally — each test function must contain the fixture path explicitly (e.g., `invalid_tabs` loads `tests/invalid/bad-tabs.urd.md`). Do not rely on name conventions to resolve paths.

**Rule name assertions per test:**

| Test | Line/column | Rule name assertion |
|------|-------------|---------------------|
| `invalid_tabs` | Assert precisely | **Drop rule name assertion.** Assert line and column, and that the error message contains one of: a literal tab character, the string `tab`, or the escape sequence `\t`. Accept any of these — pest error formatting varies. |
| `invalid_unclosed_frontmatter` | Assert precisely | Allow any of: `File`, `Frontmatter`, or a dedicated `FrontmatterDelimiter` rule if one is created. |
| `invalid_malformed_entity` | Assert precisely | Allow `Identifier` or its direct parent rule (e.g., `EntityRef`). If strict assertion on `Identifier` is desired, make the parent a silent rule in pest so `Identifier` is the reported failure point. |
| `invalid_empty_choice` | Assert precisely | Allow `Text` or its direct parent rule (e.g., `ChoiceLabel`). Same pattern as the entity test — if the parent is silent, `Text` is the reported failure. |
| `invalid_empty_heading` | Assert precisely | Allow `Text` or its direct parent rule (e.g., `HeadingText`). Same pattern as the entity test. |

This per-test table prevents both over-strict assertions (flaky across refactors) and under-strict assertions (passing when the grammar is wrong).

### Step 9: Iterate until corpus passes

Run `cargo test` and fix grammar rules until all 12 tests pass. This is the core work — expect multiple iterations. When a test fails:

1. Identify which rule is failing and where
2. Fix the `.pest` grammar
3. Back-port the fix to the `.peg` reference file (unless it's a pest-specific adaptation, in which case comment the divergence)
4. Re-run all tests (fixing one rule must not break others)

### Step 10: Validation script

Create `packages/grammar/scripts/validate.sh` (with `set -euo pipefail` at the top) that:

1. Changes to `packages/grammar/` (so the script works when invoked from repo root or from inside the crate)
2. Runs `cargo test`
3. Prints a one-line status per fixture file in the format (paths relative to `packages/grammar/`):
   - `PASS tests/valid/tavern.urd.md`
   - `FAIL tests/invalid/bad-tabs.urd.md at line 3 col 1`
   
   Fixtures are reported in a fixed order: valid corpus alphabetically, then invalid corpus alphabetically. This keeps diffs clean across runs.
   
   Followed by a final summary: `12/12 passed` or `11/12 passed, 1 failed`
4. Exits with non-zero status on any failure

## Root-level changes

### Update root `.gitignore`

Verify that `target/` is in `.gitignore`. **Cargo.lock policy:** do not create or commit `packages/grammar/Cargo.lock` until the repo has a root workspace `Cargo.toml`. This is a locked repo convention. When a workspace is established later, add the workspace-level `Cargo.lock` and remove any crate-level ones. Do not revisit this decision per-crate.

### Update root `package.json`

Add a grammar-related script:

```json
"grammar:test": "cargo test --manifest-path packages/grammar/Cargo.toml"
```

### Update `CLAUDE.md`

Add to the repository layout:

```
packages/
  grammar/          PEG grammar reference + pest parser + validation corpus
```

Add to commands:

```bash
cargo test --manifest-path packages/grammar/Cargo.toml  # Run grammar validation corpus
```

Add to tech stack:

```
- **Rust** with pest for the formal PEG grammar and parser (packages/grammar/)
```

## File Summary

| File | Action |
|------|--------|
| `docs/urd-formal-grammar-brief.md` | **Modify** — Reconcile spec conflicts (Step 1) |
| `packages/grammar/Cargo.toml` | **Create** — Rust crate manifest |
| `packages/grammar/src/lib.rs` | **Create** — Parser implementation |
| `packages/grammar/src/urd_schema_markdown.pest` | **Create** — pest grammar |
| `packages/grammar/urd-schema-markdown.peg` | **Create** — Reference PEG file |
| `packages/grammar/tests/corpus.rs` | **Create** — Test runner |
| `packages/grammar/tests/valid/*.urd.md` (7 files) | **Create** — Positive corpus |
| `packages/grammar/tests/invalid/*.urd.md` (5 files) | **Create** — Negative corpus |
| `packages/grammar/scripts/validate.sh` | **Create** — Validation script |
| `package.json` | **Modify** — Add grammar:test script |
| `CLAUDE.md` | **Modify** — Update layout, commands, tech stack |
| `.gitignore` | **Verify** — Ensure `target/` is present |

## Risks

- **Rust toolchain dependency.** This is the first Rust code in the repo. The developer machine must have `rustc` and `cargo` installed. Rust is not needed for the Astro site — only for grammar work.
- **`player`/`target` property access.** The Step 1 reconciliation may surface additional places where the syntax spec and grammar brief diverge. If so, resolve them before proceeding to code.
- **pest grammar translation.** pest has quirks (implicit whitespace handling, atomic rules, stack-based matching). Some PEG constructs may not translate mechanically. The `.pest` file may need pest-specific adaptations — these must be commented and the `.peg` file remains normative.
- **FrontmatterBody opacity.** Since `FrontmatterBody` is an opaque rule, the positive corpus files with frontmatter will need a permissive stub (e.g., consume everything until `---`) rather than a validated parse. Two frontmatter-specific negative tests are deferred to the compiler brief.
- **Edge case file failures.** The edge case files are split into three files by concern (frontmatter, comments, structure). If one fails, check the specific edge case that failed rather than bisecting a monolithic file.

## Verification

The grammar is complete when all of the following are true (from the formal grammar brief's acceptance criteria):

1. **All four test case files parse successfully.** `cargo test` passes for tavern, Monty Hall, key puzzle, and interrogation.
2. **All three edge case files parse successfully.** `cargo test` passes for frontmatter, comments, and structure edge cases.
3. **All five negative corpus files fail with correct errors.** Each bad file produces a parse error with line and column, plus rule name where applicable (see Step 8 assertion table for per-test criteria). Note: `bad-tabs.urd.md` contains tabs in multiple positions but the parser will fail at the first one encountered — this is expected and correct.
4. **The grammar is self-contained.** The `.peg` file defines every rule needed to parse narrative content without cross-referencing the prose specs.
5. **Rules are modular.** Each construct is an independent rule. Changing condition expressions does not affect choice rules.
6. **Writer/engineer boundary is visible.** Rule groups are clearly separated and commented in the `.peg` file.
7. **Error positions are actionable.** Parse failures include line and column, plus rule name where applicable.
8. **`cargo test` passes with zero failures.**
9. **The `.peg` and `.pest` files are consistent.** Any fix applied to one is back-ported to the other. Pest-specific adaptations are commented in the `.pest` file.
10. **The formal grammar brief is reconciled.** Known issues 1–3 are resolved in the document before code is written.

## References

| Document | Location | Relevance |
|----------|----------|-----------|
| Formal grammar brief | `docs/urd-formal-grammar-brief.md` | **The specification.** Every PEG rule, ambiguity resolution, and quality property. Must be reconciled (Step 1) before use. |
| Schema Markdown syntax spec | `docs/schema-markdown.md` | Source for test corpus extraction. Contains all four locked examples (§Example 1–4). |
| Architecture doc | `docs/architecture.md` | Compiler pipeline context (grammar = phase 1 of 5). |
| Test case strategy | `docs/urd-test-case-strategy.md` | Validation corpus requirements and test philosophy. |
| World Schema spec | `docs/schema-spec.md` | Defines `.urd.json` output (out of scope, but useful for understanding what the grammar does NOT validate). |

*End of Brief*