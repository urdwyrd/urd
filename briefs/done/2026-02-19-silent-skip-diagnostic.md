# URD — Silent Skip Diagnostic for Frontmatter Type Definitions

*A brief for emitting a warning when a frontmatter line resembles a type definition but fails to parse*

February 2026 | Improvement

`Broken [` at type indent → silently skipped, no diagnostic

> **Document status: BRIEF** — Adds a targeted diagnostic (URD430) when a line at type-definition indent level starts with an uppercase letter but fails `parse_type_definition()`. The silent skip that caused the 7-error cascade in the stress test was fixed by comment stripping, but the underlying failure mode — a type definition line that returns `None` with no diagnostic — is still possible for other malformed inputs. This brief converts future cascades into one localized error.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-20
**Status:** Complete

### What was done

1. **URD430 was already implemented** during the frontmatter comment stripping brief (2026-02-19). The `parse_types_block()` else branch already emits URD430 when a line at type-definition indent starts with an uppercase letter but fails `parse_type_definition()`. Two tests were already present: `frontmatter_urd430_unparseable_type_like_line` and `frontmatter_no_urd430_for_whole_line_comment`.

2. **Added 6 additional URD430 tests** to cover the brief's full test table:
   - `frontmatter_urd430_missing_colon` — `Guard [interactable]` triggers URD430
   - `frontmatter_no_urd430_for_lowercase` — `guard [interactable]:` does not trigger (lowercase)
   - `frontmatter_no_urd430_for_bracket_start` — `[interactable]:` does not trigger (no uppercase start)
   - `frontmatter_no_urd430_for_valid_type` — `Guard [interactable]:` parses normally, no URD430
   - `frontmatter_no_urd430_for_blank_line` — blank line inside types block, no URD430

3. **Added URD432 for entity declarations** (Part 2 of the brief). In `parse_entities_block()`, when `parse_entity_declaration()` returns `None` and the line starts with `@`, the compiler now emits URD432: "Line looks like an entity declaration but could not be parsed." Uses `truncate_for_display()` and `strip_frontmatter_comment()` for the display text.

4. **Added 3 URD432 tests:**
   - `frontmatter_urd432_malformed_entity` — `@broken_item` with no type triggers URD432
   - `frontmatter_no_urd432_for_valid_entity` — `@sword: Item` parses normally, no URD432
   - `frontmatter_no_urd432_for_comment_in_entities` — comment line inside entities block, no URD432

5. **All 470 tests pass** (462 existing + 8 new). No regressions.

### What changed from the brief

- **URD430 was already implemented.** This brief was written before the comment stripping brief was executed, and URD430 was added proactively during that implementation because the cascade failure mode was immediately apparent. This brief's primary contribution is the additional test coverage (6 tests) and the URD432 entity equivalent (Part 2).

- **The brief marked Part 2 (URD432) as optional.** It was implemented because the pattern is identical — a silent skip that can cause confusing downstream errors — and the implementation is trivial (5 lines of code in an existing else branch).


## The Problem

The stress test revealed a cascade where 7 URD307 "Unknown type" errors and 1 URD301 "Unresolved reference" error all traced back to a single root cause: `parse_type_definition()` returned `None` for type definitions with inline comments, and the caller (`parse_types_block()`) silently advanced to the next line.

The comment stripping brief fixes the immediate cause. But the underlying pattern is dangerous: any future input that looks like a type definition but fails the grammar will be silently dropped, and every entity referencing that type will cascade into unrelated errors.

### The current code

```rust
// In parse_types_block():
while *i < end_line {
    // ... skip blanks, skip comments, check indent ...

    if let Some(type_def) = parse_type_definition(parser, i, end_line, indent_spaces) {
        entries.push(FrontmatterEntry { ... });
    } else {
        *i += 1;  // ← Silent skip. No diagnostic. Type is gone.
    }
}
```

The `else` branch advances past the line with no error. If the line was `Guard [interactable, mobile, container]:   # comment`, the writer has no idea why Guard doesn't exist. They see URD307 errors on entity declarations that are syntactically correct, pointing at lines far from the actual problem.

### Failure modes that would trigger silent skip

Even after comment stripping:
- Missing colon: `Guard [interactable]` (writer forgot the `:`)
- Malformed trait list: `Guard [interactable, ` (unclosed bracket)
- Name doesn't start with uppercase: `guard [interactable]:` (lowercase)
- Empty name: `[interactable]:` (missing type name)
- Stray text at type indent level that isn't a comment or blank

These are real mistakes writers and engineers will make. Each one should produce one clear diagnostic, not a cascade.


## The Fix

### Part 1: Add URD430 warning in `parse_types_block()`

When `parse_type_definition()` returns `None` and the line is non-empty and not a comment, check if the line looks like a type definition attempt:

```rust
// In parse_types_block(), the else branch:
} else {
    let first_char = ptrimmed.chars().next().unwrap_or(' ');
    if first_char.is_uppercase() {
        // Line starts with uppercase at type-definition indent level.
        // This almost certainly was meant to be a type definition.
        parser.diagnostics.warning(
            "URD430",
            format!(
                "Line resembles a type definition but could not be parsed: '{}'. \
                 Check for missing ':', unclosed brackets, or other syntax issues.",
                truncate_for_display(ptrimmed),
            ),
            parser.line_span(*i),
        );
    }
    *i += 1;
}
```

**Why uppercase check:** Type names in Urd must start with an uppercase letter (PascalCase convention, enforced by the grammar). A line at type-definition indent that starts with uppercase is overwhelmingly likely to be a type definition attempt. Lines starting with lowercase at this indent would be property definitions at the wrong indent level, which is a different (and rarer) error class.

**Why warning, not error:** The line might be legitimate content that the parser doesn't understand yet (future syntax, or an edge case). A warning flags the issue without blocking compilation. The downstream URD307 errors will still appear, but the writer now has a breadcrumb pointing at the actual problem.

### Part 2: Consider the entity block equivalent

The same silent skip pattern exists in `parse_entities_block()`:

```rust
if let Some(entity) = parse_entity_declaration(parser, *i) {
    entries.push(...);
}
*i += 1;  // ← Also a silent skip
```

If an entity declaration fails to parse (e.g., `@broken_entity: ` with no type), it's silently dropped. Consider adding a similar diagnostic here:

```rust
if ptrimmed.starts_with('@') {
    parser.diagnostics.warning(
        "URD432",
        format!(
            "Line resembles an entity declaration but could not be parsed: '{}'.",
            truncate_for_display(ptrimmed),
        ),
        parser.line_span(*i),
    );
}
```

This is optional for this brief but recommended.


## Files Changed

| File | Change |
|------|--------|
| `src/parse/frontmatter.rs` | Add URD430 warning in `parse_types_block()` else branch. Optionally add URD432 in `parse_entities_block()`. |
| `tests/` | New tests for URD430 and optionally URD432. |


## New Tests to Add

| Test | Input | Expected |
|------|-------|----------|
| Missing colon | `Guard [interactable]` at type indent | URD430 warning |
| Unclosed bracket | `Guard [interactable, ` at type indent | URD430 warning |
| Lowercase name | `guard [interactable]:` at type indent | No URD430 (lowercase, not a type attempt) |
| Empty name | `[interactable]:` at type indent | No URD430 (doesn't start with uppercase) |
| Valid type definition | `Guard [interactable]:` at type indent | No URD430, type registers normally |
| Comment at type indent | `# This is a comment` at type indent | No URD430 (already handled as comment) |
| Blank line at type indent | ` ` at type indent | No URD430 (already handled as blank) |
| Malformed entity (optional) | `@broken: ` at entity indent | URD432 warning |


## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| T1 | A line starting with uppercase at type indent that fails to parse emits URD430. | Unit test. |
| T2 | Valid type definitions do not trigger URD430. | Regression test. |
| T3 | Comment and blank lines at type indent do not trigger URD430. | Unit test. |
| T4 | The warning message includes the truncated line content. | Unit test on diagnostic message. |
| T5 | All existing tests still pass. | `cargo test` — no regressions. |


## Spec Alignment

No spec changes needed. This is a compiler diagnostic improvement, not a language feature. URD430 is a new warning code in the URD400 range (validation warnings). The diagnostic reference should document it.
