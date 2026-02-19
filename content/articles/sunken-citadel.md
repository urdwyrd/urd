---
title: "Compiler 0.1.3: The Sunken Citadel"
slug: sunken-citadel
description: Four briefs, 38 new tests, and the sunken-citadel fixture drops from 19 errors to zero. Lists, entity references, implicit properties, built-in jumps, and diagnostic coverage all land in a single release.
date: "2026-02-20"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the third compiler patch and the stress-test milestone it clears.
> Single canonical copy. February 2026.

## The stress test

The sunken-citadel fixture is 1,101 lines of Schema Markdown. It defines a world with typed entities, nested containers, entity-targeted choices with guard conditions, list-valued properties, entity reference overrides, and `-> end` jumps. It uses every feature documented in the Schema Markdown spec. If the compiler can handle it cleanly, it can handle real content.

At the start of 0.1.2, the fixture produced 19 errors. By the end of 0.1.2, two fixes — frontmatter comment stripping and the move-effect destination prefix — brought it to 9. The reserved property reference fix in 0.1.2 brought it to 8. Version 0.1.3 eliminates the remaining 8 errors across four targeted briefs.

## What 0.1.3 fixes

**List and entity reference values in frontmatter.** The `Scalar` enum — the parser's representation of frontmatter values — only had four variants: `String`, `Integer`, `Number`, `Boolean`. The symbol table's `Value` enum already had `List` and `EntityRef`, but the parser could not produce them. Writing `tags: [conspiracy, voss]` fell through to a raw string. Writing `requires: @bone_key` became the string `"@bone_key"`.

The fix adds `List(Vec<Scalar>)` and `EntityRef(String)` to `Scalar`, with parse-level recognition of `[...]` list literals and `@identifier` entity references. The existing `parse_flow_list()` infrastructure handles empty lists, trailing commas, and nested entity references inside lists. Three converter functions — `scalar_to_value()` in LINK, `scalar_to_json()` in EMIT, and the type validator — each gained two new match arms. This was the largest single fix: it cleared 6 of the remaining 8 errors.

**Implicit container property.** The Schema Spec defines `container` as an implicit property of every entity — it tracks which entity contains this one at runtime. The compiler's property resolution checked the declared property list and rejected `container` with URD308: "Property not declared on type." The fix adds an `IMPLICIT_PROPERTIES` constant checked at all three resolution sites: entity overrides, condition property access, and effect property access. One error cleared.

**Built-in `-> end` jump target.** Schema Markdown defines `-> end` as a built-in terminal that ends the current conversation. The compiler's jump resolver only checked section names and exit lists, emitting URD309: "Unresolved jump target 'end'." The fix adds a `BUILTIN_JUMP_TARGETS` constant checked before section/exit lookup. If a user-defined section named `end` shadows the built-in, URD431 warns and the built-in wins. One error cleared.

**Silent skip diagnostics.** Two new warnings catch lines that the parser previously skipped in silence. URD430 fires when a line at type-definition indent level starts with an uppercase letter but fails to parse as a type definition — the most common cause of cascade errors from typos. URD432 fires when a line starting with `@` in the entities block fails to parse as an entity declaration. Neither warning existed before; both replace silent skips that left authors wondering why their types or entities were missing.

## The error curve

| Version | Sunken-citadel errors | Total tests |
|---------|----------------------|-------------|
| 0.1.0   | 19                   | 413         |
| 0.1.1   | 19                   | 433         |
| 0.1.2   | 0 (at release)       | 441         |
| 0.1.3   | 0                    | 480         |

The distinction matters: 0.1.2 cleared the comment-stripping and move-effect bugs during development but was released before the remaining 8 errors were addressed. Version 0.1.3 is the first release where the stress test was a target, not a side effect.

## The pattern: briefs as units of work

Each of the four fixes was specified as a brief — a structured task document with a problem statement, proposed changes, test expectations, and an execution record. The briefs lived in `briefs/backlog/`, moved to `active/` during implementation, and landed in `done/` with a filled execution record.

This is not project management overhead. Each brief is a single prompt. The problem statement constrains the scope. The proposed changes describe the implementation at the function level. The test expectations define done. The execution record captures what actually happened — including test counts, error counts, and deviations from the plan.

For 0.1.3, all four briefs executed cleanly. No deviations from the proposed changes. The unified value representation brief was simpler than expected because the `Value` enum already had the needed variants — the gap was only at the parser level. The silent skip brief discovered that URD430 was already implemented during the comment-stripping work in 0.1.2, so its contribution was additional test coverage and the URD432 equivalent for entity declarations.

## The test count

Version 0.1.2 had 441 tests. Version 0.1.3 adds 39 new tests: 9 for list and entity reference parsing, 5 for implicit container resolution, 4 for built-in jump targets, 6 for URD430 silent-skip coverage, 3 for URD432 entity declaration warnings, 8 for the sunken-citadel frontmatter comment fix, and 4 regression tests. The total stands at 480, with a 100% pass rate. The [changelog on GitHub](https://github.com/urdwyrd/urd/blob/main/packages/compiler/CHANGELOG.md) records every change.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
