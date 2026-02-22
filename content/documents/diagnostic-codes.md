---
title: "Diagnostic Code Reference"
slug: "diagnostic-codes"
description: "Complete catalog of every diagnostic code emitted by the Urd compiler, organised by phase with severity, trigger conditions, and cross-references to gate requirements."
category: "architecture"
format: "Reference Document"
date: "2026-02-22"
status: "stable"
order: 5
tags:
  - reference
  - compiler
  - diagnostics
  - error-codes
details:
  - "69 diagnostic codes across five compiler phases"
  - "11 PARSE codes (URD100–URD112)"
  - "14 IMPORT codes (URD201–URD214)"
  - "14 LINK codes (URD301–URD314)"
  - "30 VALIDATE codes (URD401–URD434)"
  - "Cross-reference to compiler gate requirements"
---

# Urd Compiler — Diagnostic Code Reference

*Complete catalog of every diagnostic code emitted by the Urd compiler.*

> **Document status: REFERENCE**
> Extracted from compiler source (v0.1.6). Each code is linked to its emitting phase, severity, and triggering condition.
> February 2026.

---

## Code Ranges

| Phase    | Range         | Source Files |
|----------|---------------|-------------|
| PARSE    | URD100–URD199 | `parse/mod.rs`, `parse/frontmatter.rs`, `parse/content.rs` |
| IMPORT   | URD200–URD299 | `import/mod.rs` |
| LINK     | URD300–URD399 | `link/collect.rs`, `link/resolve.rs` |
| VALIDATE | URD400–URD499 | `validate/mod.rs`, `validate/types.rs`, `validate/entities.rs`, `validate/conditions.rs`, `validate/effects.rs` |
| EMIT     | URD500–URD599 | `emit/mod.rs` (currently no diagnostics emitted) |

---

## Severity Levels

| Severity | Meaning |
|----------|---------|
| Error    | Compilation cannot produce valid output. The EMIT phase will not run if any errors exist. |
| Warning  | Legal but likely unintentional. Output is still produced. |
| Info     | Informational. No impact on compilation. |

---

## PARSE Phase (URD100–URD199)

| Code | Severity | Description | Trigger |
|------|----------|-------------|---------|
| URD101 | Error | Unclosed frontmatter block | Opening `---` found but no closing `---` before end of file. |
| URD102 | Error | Tab character in source | A line contains one or more tab characters. Urd requires spaces for indentation. Emitted per tab found. |
| URD103 | Error | File exceeds size limit | Source file is larger than 1 MB (1,048,576 bytes). Also emitted during IMPORT for imported files. |
| URD104 | Error | Frontmatter nesting too deep | A frontmatter entry exceeds 8 levels of indentation nesting. |
| URD105 | Error | YAML anchor rejected | An `&identifier` anchor pattern was detected in frontmatter. Urd does not support YAML anchors. |
| URD106 | Error | YAML alias rejected | A `*identifier` alias pattern was detected in frontmatter. Urd does not support YAML aliases. |
| URD107 | Error | YAML merge key rejected | A `<<:` merge key was detected in frontmatter. Urd does not support YAML merge keys. |
| URD108 | Error | YAML custom tag rejected | A `!!type` custom tag was detected in frontmatter. Urd does not support YAML custom tags. |
| URD109 | Error | Block-style list rejected | A `- item` block-style list was detected in frontmatter. Use flow-style `[item1, item2]` instead. |
| URD111 | Error | Unrecognised frontmatter syntax | A frontmatter line could not be parsed as any valid entry pattern. |
| URD112 | Error | Unrecognised content syntax | A content line could not be parsed as any valid block type (location, section, choice, condition, effect, entity speech, jump, etc.). Fallback after all grammar rules fail. |

---

## IMPORT Phase (URD200–URD299)

| Code | Severity | Description | Trigger |
|------|----------|-------------|---------|
| URD201 | Error | Imported file not found | The file referenced by an `import:` declaration does not exist on disk. |
| URD202 | Error | Circular import detected | A cycle was found in the import graph. The full cycle path is reported (e.g. `a → b → c → a`). |
| URD203 | Error | File stem collision | Two or more files in the compilation unit produce the same stem after stripping the `.urd.md` extension (e.g. `foo/bar.urd.md` and `baz/bar.urd.md`). Section IDs would collide. |
| URD204 | Error | Import depth limit exceeded | The import chain exceeds 64 levels deep. |
| URD205 | Error | Compilation unit too large | More than 256 files discovered in the compilation unit. |
| URD206 | Warning | Filename casing mismatch | The import path differs in casing from the file discovered on disk (case-insensitive filesystem). The compiler uses the discovered casing. |
| URD207 | Error | Self-import | A file's `import:` declaration references itself. |
| URD208 | Error | Import escapes project root | After resolving `..` segments, the import path would resolve outside the project root directory. |
| URD209 | Error | Absolute import path | The import path is absolute (starts with `/` or a drive letter like `C:`). Imports must be relative. |
| URD210 | Error | Missing .urd.md extension | The import path does not end with `.urd.md`. |
| URD211 | Error | Empty import path | The `import:` declaration has an empty path string. |
| URD212 | Error | Invalid UTF-8 | The imported file contains invalid UTF-8 byte sequences. |
| URD213 | Error | Permission denied | The compiler cannot read the imported file due to OS-level permissions. |
| URD214 | Error | I/O error | A generic I/O error occurred while reading the imported file. |

---

## LINK Phase (URD300–URD399)

| Code | Severity | Description | Trigger |
|------|----------|-------------|---------|
| URD301 | Error | Unresolved reference | An `@entity`, type name, location, or property access could not be resolved. Includes scope violations (declared in a file that is not imported). Suggestions offered via edit distance when available. |
| URD302 | Error | Duplicate entity or rule ID | Two entities or two rules share the same ID across the compilation unit. Both declaration sites are reported. |
| URD303 | Error | Duplicate type name | Two type definitions share the same name across the compilation unit. Both declaration sites are reported. |
| URD304 | Error | Duplicate location ID | Two `# Location` headings produce the same slugified ID. Both display names and the colliding slug are reported. |
| URD305 | Error | Duplicate section name | Two `== section` labels share the same name within a single file. Section names must be unique per file. |
| URD306 | Error | Duplicate choice ID | Two choices within the same section produce the same slugified ID. Both labels and the colliding slug are reported. |
| URD307 | Error | Unknown entity type | An `@entity: TypeName` declaration references a type that does not exist in the symbol table. Edit distance suggestions offered. |
| URD308 | Error | Unknown property on type | A property override or property access references a property that does not exist on the entity's declared type. |
| URD309 | Error | Unresolved jump target or section | A `->` jump target or exhaustion check references a section or exit name that does not exist in the current scope. |
| URD310 | Warning | Section shadows exit | A section name matches an exit direction name in the same location. Jumps to that name will target the section, not the exit. Use `-> exit:name` for the exit. |
| URD311 | Error | Unresolved exit-qualified jump | A `-> exit:name` jump references an exit direction that does not exist in the current location. |
| URD312 | Error | Unresolved exit destination | An exit declaration's destination does not resolve to any known location. Includes scope violations. |
| URD313 | Error | Empty slugified ID | A heading or declaration produces an empty string after slugification. |
| URD314 | Error | Construct outside location context | An exit declaration, entity presence list, or exit-qualified jump appears before any `# Location` heading. |

---

## VALIDATE Phase (URD400–URD499)

| Code | Severity | Description | Trigger |
|------|----------|-------------|---------|
| URD401 | Error | Type mismatch (condition/effect) | A property comparison value or set-effect value does not match the property's declared type. Used in conditions and effects. |
| URD402 | Error | Invalid enum override | An entity's property override specifies an enum value that is not in the type's declared values list. |
| URD404 | Error | Invalid world.start | The `world.start` value does not match any declared location ID. |
| URD405 | Error | Invalid world.entry | The `world.entry` value does not match any declared sequence ID. |
| URD406 | Error | Mutual exclusion: target + target_type | A choice declares both a `target` (entity/section) and a `target_type` (type selector). Only one is allowed. |
| URD407 | Error | Unknown action in phase | A sequence phase references an action ID that does not exist. |
| URD408 | Error | Unknown rule in phase | A sequence phase references a rule name that does not exist. |
| URD409 | Error | Invalid advance mode | A sequence phase declares an advance mode that is not one of: `on_action`, `on_rule`, `on_condition`, `end`, `auto`, `manual`. |
| URD410 | Error/Warning | Choice nesting depth | A choice is nested too deeply. Warning at depth 3, error at depth 4+. |
| URD411 | Warning | Author set `urd` field | The author explicitly set the `urd:` field in the world block. This field is injected automatically and the author's value will be overridden. |
| URD412 | Error | Player entity missing traits | The `@player` entity's type is missing required `mobile` and/or `container` traits. |
| URD413 | Error | Invalid property default | A type definition's property default value does not match the property's declared type. |
| URD414 | Error | Empty enum values list | An enum property declares `enum()` with no values. |
| URD415 | Error | Unknown ref target type | A `ref(TypeName)` property references a type name that does not exist. |
| URD416 | Error | Invalid range: min > max | A numeric property declares a minimum value greater than its maximum. |
| URD417 | Error | Range on non-numeric type | Range constraints (`min`/`max`) are declared on a property that is not integer or number. |
| URD418 | Error | Value outside declared range | A numeric value (in override, condition, or effect) is outside the property's declared `[min, max]` range. |
| URD419 | Error | Ref type mismatch | A `ref(TypeName)` property references an entity whose type does not match the declared ref constraint. |
| URD420 | Error | Invalid comparison operator | An ordering operator (`<`, `>`, `<=`, `>=`) is used on a non-numeric property. Only `==` and `!=` are valid for non-numeric types. |
| URD422 | Error | Missing container trait | An entity is used as a container (in a containment check or move destination) but its type does not have the `container` trait. |
| URD423 | Error | Cross-file exhaustion check | An exhaustion check references a section that is not declared in the current file. Exhaustion is file-local in v1. |
| URD424 | Error | Arithmetic on non-numeric property | An arithmetic effect operator (`+` or `-`) is used on a property that is not integer or number. |
| URD425 | Error | Move without portable trait | A `move` effect targets an entity whose type does not have the `portable` trait. |
| URD426 | Warning | Reveal on non-hidden property | A `reveal` effect targets a property that is not marked as hidden. The reveal has no effect. |
| URD427 | Warning | Auto phase with player actions | A sequence phase is marked `auto` but also declares player actions. The actions will never be available. |
| URD428 | Error | Empty sequence | A sequence declares no phases. |
| URD429 | Warning | Unrecognised property type | A property's type string is not one of the recognised types (`bool`, `int`, `num`, `str`, `enum`, `ref`, `list` or long forms). Treated as `string`. |
| URD430 | Warning | Unreachable location (S3) | A location has no path from `world.start` via exits. Also used in PARSE for unparseable type definitions. |
| URD431 | Warning | Section shadows built-in jump | A section named `end` shadows the built-in `-> end` terminal. Jumps will always end the conversation, not jump to the section. |
| URD432 | Warning | Orphaned choice (S4) | A choice's condition requires an enum value that is not in the type's declared values list, meaning the choice can never be available. Also used in PARSE for unparseable entity declarations. |
| URD433 | Warning | Missing fallthrough (S6) | A section contains only one-shot choices with no terminal jump or fallthrough text. It will exhaust to an empty state. |
| URD434 | Warning | Section-exit shadowing (S8) | A section label in a location shares a name with an exit direction. Jumps will target the section. Use `-> exit:name` for the exit. |

---

## EMIT Phase (URD500–URD599)

No diagnostic codes are currently emitted during EMIT. The phase runs only when zero errors exist, and operates on pre-validated data structures.

Reserved for future use (e.g. JSON Schema conformance warnings, output size limits).

---

## Summary

| Phase    | Errors | Warnings | Total |
|----------|--------|----------|-------|
| PARSE    | 11     | 0        | 11    |
| IMPORT   | 13     | 1        | 14    |
| LINK     | 13     | 1        | 14    |
| VALIDATE | 22     | 8        | 30    |
| EMIT     | 0      | 0        | 0     |
| **Total** | **59** | **10** | **69** |

---

## Cross-Reference: Gate Requirements → Diagnostic Codes

| Gate Check | Primary Codes |
|-----------|---------------|
| C1: Constrained frontmatter | URD104–URD109, URD111 |
| C2: Import resolution | URD201, URD209–URD211 |
| C3: Circular import detection | URD202 |
| C4: Duplicate entity IDs | URD302 |
| C5: Duplicate type names | URD303 |
| C6: Reference resolution | URD301, URD307–URD309, URD311–URD312 |
| C7: Property validation | URD401, URD402, URD413–URD420 |
| C8: `urd: "1"` injection | URD411 |
| C9: Nesting depth | URD410 |
| S1: Undefined entity ref | URD301 |
| S2: Type mismatch | URD401, URD410+ |
| S3: Unreachable location | URD430 |
| S4: Orphaned choice | URD432 |
| S5: Duplicate IDs | URD302–URD306 |
| S6: Missing fallthrough | URD433 |
| S7: Circular imports | URD202 |
| S8: Shadowed exit | URD434 |

---

*This document is the single reference for all compiler diagnostic codes. When new codes are added, they should be registered here.*

*End of Document*
