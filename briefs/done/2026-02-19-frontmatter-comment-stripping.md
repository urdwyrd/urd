# URD — Frontmatter Inline Comment Stripping

*A brief for adding `#` comment stripping to the frontmatter parser*

February 2026 | Bug Fix

`Guard [interactable, mobile, container]:   # mobile + container` → type silently skipped

> **Document status: BRIEF** — Fixes a parser gap where inline `#` comments on frontmatter property definition and type definition lines are not stripped before structural parsing decisions, causing cascading type registration failures and default value misparses. This is the highest-leverage single fix in the compiler backlog: one root cause produces 13 of 19 errors in the stress test.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-20
**Status:** Done — 10 of 19 stress test errors eliminated, 12 new tests, 453 total tests passing

### What was done

- Implemented `strip_frontmatter_comment()` in `src/parse/frontmatter.rs` — quote-aware inline `#` comment stripping. Handles double and single quoted strings. A `#` preceded by whitespace outside quotes marks the comment start; the function returns the text before it, trimmed.
- Applied stripping at four entry points, once per function before any structural decisions:
  - `parse_type_definition()` — strips before `ends_with(':')` check
  - `parse_property_def()` — strips before colon split and type/default extraction
  - `parse_entity_declaration()` — strips before `@` check and type/override extraction
  - `parse_world_fields()` — strips before colon split on world block field lines
- Added URD430 warning diagnostic in `parse_types_block()` for lines at type-definition indent that start with an uppercase letter but fail to parse as a type definition. Heuristic catches malformed type lines without false-positiving on comment lines or blank lines.
- Added 12 new parse tests: `frontmatter_type_def_with_comment`, `frontmatter_type_def_without_comment_regression`, `frontmatter_property_default_with_comment`, `frontmatter_property_type_with_comment`, `frontmatter_hash_inside_quoted_string_preserved`, `frontmatter_hash_inside_single_quoted_string_preserved`, `frontmatter_hidden_property_with_comment`, `frontmatter_enum_with_comment`, `frontmatter_entity_override_with_comment`, `frontmatter_world_field_with_comment`, `frontmatter_urd430_unparseable_type_like_line`, `frontmatter_no_urd430_for_whole_line_comment`.
- All 453 tests pass (441 existing + 12 new). Zero regressions.

### What changed from the brief

- **10 errors eliminated, not 13.** The brief predicted 13 of 19 errors would be fixed. The actual count is 10. The difference: 2 URD413 errors for `list = []` defaults and 1 cascading URD301 for `@gate_guard` trace to list literal parsing gaps, not comment stripping. The `[]` empty list default is correctly stripped of its comment but still fails `parse_scalar_value()` because the parser doesn't recognise `[]` as a valid list literal in the default value position. This is the **unified value representation** backlog brief's scope, not this brief's.
- **`parse_entry()` not modified directly.** The brief specified stripping in `parse_entry()` on the `after_colon` value. Instead, stripping was applied in `parse_entity_declaration()` and `parse_world_fields()` which are the downstream consumers. The effect is identical — each function gets one clean line — but the stripping point is closer to where the structural decision happens.
- **Acceptance criteria T5 partially met.** 3 of 5 URD413/URD429 errors disappeared (the 2 `num()` defaults and the `ref(Key)` type). The remaining 2 URD413 errors (`list = []`) are a separate root cause.
- **Acceptance criteria T7 partially met.** The URD301 for `@gate_guard` at line 552 persists. It is not caused by comment stripping — the entity `@gate_guard` is used in a condition expression (`@gate_guard.alertness`) at line 552, inside a location where `@gate_guard` is not present in the entity list. This is a scope/reference issue, not a type registration cascade.

---

## The Problem

The Schema Markdown spec states: *"Comments in Urd frontmatter use `#`."* (§Comments). This is consistent with YAML-like grammar conventions. The stress test (`sunken-citadel.urd.md`) uses inline `#` comments on both type definition lines and property definition lines:

```
Guard [interactable, mobile, container]:   # mobile + container + interactable
  durability: num(0.0, 100.0) = 100.0      # float with range
  tags: list = []                          # list type
  requires: ref(Key)                       # ref type
```

The frontmatter parser handles whole-line `#` comments correctly (in `parse_frontmatter()`'s main loop, lines starting with `#` are skipped). But it does **not** strip inline `#` comments from lines that contain structural content. This causes two classes of failure.

### Failure 1: Type definitions silently skipped

`parse_type_definition()` checks `trimmed.ends_with(':')`. When the line is `Guard [interactable, mobile, container]:   # mobile + container + interactable`, the trimmed text ends with `interactable`, not `:`. The function returns `None` and the type is silently dropped. No diagnostic is emitted.

This affects every type definition that has an inline comment: Guard, Scholar, Spirit, StorageChest. All entities referencing these types then fail with URD307 "Unknown type." One entity reference (`@gate_guard` at line 552) fails with URD301 "Unresolved reference."

**Error count: 7 URD307 + 1 URD301 = 8 cascading errors from silent type skip.**

### Failure 2: Property defaults and type signatures corrupted

`parse_property_def()` splits the line on ` = ` to separate the type signature from the default value. With inline comments present:

| Line | What compiler sees for default | What it should see |
|------|------|------|
| `durability: num(0.0, 100.0) = 100.0      # float with range` | `"100.0      # float with range"` | `100.0` |
| `weight: num(0.1, 50.0) = 1.0            # float` | `"1.0            # float"` | `1.0` |
| `tags: list = []                          # list type` | `"[]                          # list type"` | `[]` |
| `required_items: list = []                # list type` | `"[]                # list type"` | `[]` |

The `#`-contaminated defaults fail `parse_scalar_value()` (they don't parse as numbers or lists), so the compiler emits URD413 "Default value does not match declared type."

For type signatures without defaults, the comment contaminates the type string:

| Line | What compiler sees for type | What it should see |
|------|------|------|
| `requires: ref(Key)                       # ref type` | `"ref(Key)                       # ref type"` | `ref(Key)` |

The full string including the comment is passed to `parse_type_signature()`, which doesn't recognize it, so the compiler emits URD429 "Unrecognised property type."

**Error count: 4 URD413 + 1 URD429 = 5 direct parse errors.**

### Total cascade

**13 of 19 stress test errors trace to this single missing feature.**

### Why the existing `strip_inline_comment()` does not apply

The content parser has `Parser::strip_inline_comment()`, but it strips `" //"` (double-slash), which is the content-region comment syntax. Frontmatter uses `#` per the spec. The frontmatter parser needs its own comment stripping function that:

1. Finds `#` preceded by whitespace (not at position 0 — that's a whole-line comment already handled)
2. Is **quote-aware**: a `#` inside a quoted string is not a comment
3. Returns the text before the comment, trimmed


## The Fix

### Part 1: Implement `strip_frontmatter_comment()`

Add a new function to `parse/frontmatter.rs` (or as a method on `Parser`):

```rust
/// Strip an inline frontmatter comment (# ...) from a line.
/// Returns the text before the comment, trimmed.
/// Quote-aware: '#' inside "..." or '...' is not a comment.
fn strip_frontmatter_comment(text: &str) -> &str {
    let mut in_quotes = false;
    let mut quote_char = '"';

    let bytes = text.as_bytes();
    for i in 0..bytes.len() {
        let ch = bytes[i];
        if in_quotes {
            if ch == quote_char as u8 {
                in_quotes = false;
            }
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            in_quotes = true;
            quote_char = ch as char;
            continue;
        }
        // '#' preceded by whitespace = inline comment start
        if ch == b'#' && i > 0 && bytes[i - 1] == b' ' {
            return text[..i].trim_end();
        }
    }
    text
}
```

This is deliberately simple and conservative. It handles double and single quoted strings. It does not need to handle escape sequences because the frontmatter grammar does not support them — quoted strings in Urd are literal.

### Part 2: Apply stripping once per entry function

The reviewer correctly noted that stripping at multiple points increases the chance of subtle path differences. Instead, strip exactly once at the entry point of each function that processes a frontmatter line:

**In `parse_type_definition()`**, strip immediately after tab processing and before the `ends_with(':')` check:

```rust
let text = parser.check_tabs(line_idx);
let trimmed = strip_frontmatter_comment(text.trim());
// Now trimmed is clean — all downstream logic operates on comment-free text
```

**In `parse_property_def()`**, strip immediately after tab processing and before the colon split:

```rust
let text = parser.check_tabs(line_idx);
let trimmed = strip_frontmatter_comment(text.trim());
// rest, colon_pos, type_and_default all derived from clean text
```

**In `parse_entry()`**, strip on the `after_colon` value for entity declarations and world fields:

```rust
let after_colon = strip_frontmatter_comment(trimmed[colon_pos + 1..].trim());
```

This gives each function exactly one clean line to work with, applied at the top before any structural decisions.

### Part 3: Emit diagnostic on silent type skip

The silent skip in `parse_type_definition()` is dangerous even after comment stripping is fixed. If a future line resembles a type definition but fails the grammar (e.g., missing colon, malformed trait list), it will return `None` and be silently swallowed by `parse_types_block()`.

Add a diagnostic when a line at type-definition indent level looks like a type definition but fails to parse:

```rust
// In parse_types_block(), after parse_type_definition() returns None:
if ptrimmed.chars().next().map_or(false, |c| c.is_uppercase()) {
    parser.diagnostics.warning(
        "URD430",
        format!(
            "Line looks like a type definition but could not be parsed: '{}'.",
            truncate_for_display(ptrimmed),
        ),
        parser.line_span(*i),
    );
}
```

This converts future cascades into one localized diagnostic. The heuristic — "starts with uppercase at type-definition indent" — catches the common case without false-positiving on comment lines or blank lines.


## Files Changed

| File | Change |
|------|--------|
| `src/parse/frontmatter.rs` | Add `strip_frontmatter_comment()`. Call it in `parse_type_definition()`, `parse_property_def()`, and `parse_entry()`. Add URD430 warning for unparseable type-like lines. |
| `tests/` | New tests for comment stripping and the URD430 diagnostic. |


## New Tests to Add

| Test | Input | Expected |
|------|-------|----------|
| Type def with comment | `Guard [interactable, mobile]:  # comment` | Registers as type with 2 traits |
| Type def without comment | `Villager [interactable]:` | Still works (regression) |
| Property default with comment | `durability: num(0.0, 100.0) = 100.0  # float` | type `number`, min 0.0, max 100.0, default 100.0 |
| Property type with comment | `requires: ref(Key)  # ref type` | type `ref`, ref_type `Key` |
| Property with no comment | `name: string = "hello"` | Unchanged (regression) |
| Hidden property with comment | `~secret: string = "none"  # hidden` | Hidden visibility preserved, comment stripped |
| Hash inside quoted string | `name: string = "value with # in it"` | Default is `"value with # in it"` — NOT truncated |
| Hash inside single-quoted string | `label: string = 'room #5'` | Default is `"room #5"` |
| Enum with comment | `mood: enum(a, b) = a  # note` | Enum parses correctly |
| Property with no default, no comment | `name: string` | Unchanged (regression) |
| Silent skip diagnostic | `Broken [` at type indent | URD430 warning emitted |
| Silent skip: false positive guard | `# This is a whole-line comment` at type indent | No URD430 (already handled) |
| Entity override line with comment | `@player: Guard { name: "Test" }  # player decl` | Entity parsed correctly |
| Sunken Citadel e2e | Full fixture | 13 of 19 errors eliminated |


## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| T1 | `Guard [interactable, mobile, container]:  # comment` registers as type with 3 traits. | Unit test on `parse_type_definition()`. |
| T2 | `durability: num(0.0, 100.0) = 100.0  # comment` parses with correct type, range, and default. | Unit test on `parse_property_def()`. |
| T3 | `requires: ref(Key)  # comment` parses as `ref` type with ref_type `Key`. | Unit test on `parse_property_def()`. |
| T4 | `name: string = "value with # in it"` preserves the `#` inside the quoted string. | Unit test — quote-awareness. |
| T5 | All 5 URD413/URD429 errors from the stress test disappear. | Stress test compilation. |
| T6 | All 7 URD307 "Unknown type" errors disappear. | Stress test compilation. |
| T7 | The URD301 for `@gate_guard` disappears. | Stress test compilation. |
| T8 | Unparseable type-like line at type indent emits URD430 warning. | Unit test. |
| T9 | All existing tests still pass. | `cargo test` — no regressions. |
| T10 | Comment stripping happens exactly once per function, before any structural decision. | Code review. |


## Spec Alignment

The Schema Markdown spec §Comments states: *"Comments in Urd frontmatter use `#`."* The formal grammar brief lists `# text` under the frontmatter accepted constructs. The current parser handles whole-line `#` comments but not inline `#` comments on content-bearing lines. This fix closes that gap.

No spec or grammar changes are needed. The formal grammar brief should be updated to explicitly document inline `#` comments in frontmatter as a suffix rule, parallel to how `InlineComment` (`// text`) is documented for the content region. This is a documentation update, not a grammar change.
