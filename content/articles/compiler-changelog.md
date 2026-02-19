---
title: "Compiler 0.1.1: Type Aliases and the Silent Bug"
slug: compiler-changelog
description: The first patch release fixes a class of bug where shorthand types compiled silently to the wrong type, adds range shorthand parsing, and introduces a formal changelog.
date: "2026-02-19"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the first compiler patch and the introduction of a formal changelog.
> Single canonical copy. February 2026.

## The bug that compiled

Version 0.1.0 had a silent type resolution bug. Writing `trust: int` in a property definition did not produce an error — it compiled successfully, but the property was treated as `string`. The word `int` was not in the LINK phase's type lookup table. It fell through to the default arm, which assumes `string`, and carried on without a word.

This is the worst kind of compiler bug. Not a crash. Not an error message. A successful compilation that produces wrong output. The author writes `int` because every language they have used accepts it. The compiler says nothing. The JSON output says `"type": "string"`. Nobody notices until something downstream breaks.

The same was true of `int(0, 100)` — the range shorthand documented in every Schema Markdown example. The parser did not recognise it. The entire string `"int(0, 100)"` was stored as the raw type name, failed to match any known type in LINK, and silently became `string`. The min and max values were never extracted.

## What 0.1.1 fixes

**Type shorthand aliases.** The parser now normalises `int` to `integer`, `num` to `number`, `str` to `string`, and `bool` to `boolean` during the PARSE phase. The canonical names are what reach the symbol table and the JSON output. Writers can use whichever form they prefer — the compiled output is identical.

| Source | Compiled type |
|--------|--------------|
| `int`, `integer` | `"integer"` |
| `num`, `number` | `"number"` |
| `str`, `string` | `"string"` |
| `bool`, `boolean` | `"boolean"` |

**Range shorthand parsing.** `int(0, 100)` and `num(0.0, 1.0)` now correctly extract the min and max values. Both the short and long forms work: `integer(0, 100)` produces the same output as `int(0, 100)`. The range applies only to numeric types — `str(10, 20)` is not valid and will trigger a warning.

**URD429: unrecognised type warning.** A new diagnostic catches type strings that do not match any recognised type. Writing `trust: integr` now produces:

```
warning[URD429]: Unrecognised property type 'integr' — treating as 'string'.
```

The property still compiles as `string` (preserving backwards compatibility), but the author is told something looks wrong. This catches typos, capitalisation errors (`Int`, `Bool`), and invented type names that would previously sail through in silence.

## The test count

Version 0.1.0 shipped with 413 tests. Version 0.1.1 adds 20 new tests: 10 for parse-level alias normalisation and range extraction, 4 for validation-phase type checking, and 6 end-to-end integration tests verifying that aliases produce the correct JSON output. The total stands at 433, with a 100% pass rate.

## The changelog

The compiler now has a formal [changelog on GitHub](https://github.com/urdwyrd/urd/blob/main/packages/compiler/CHANGELOG.md), following the [Keep a Changelog](https://keepachangelog.com/) format. Every version bump records what was added, changed, fixed, and removed. The version number is linked from both the [playground](/playground) footer and the [test dashboard](/) so there is always a path from the live version to what changed.

The version bump itself is automated. Running `pnpm compiler:bump` reads the current version from `Cargo.toml`, computes the next version, runs the full test suite, regenerates the test report, and rebuilds the WASM binary. The version flows from `Cargo.toml` through `env!("CARGO_PKG_VERSION")` into the WASM module, and from there into the playground footer at runtime. One command, one source of truth.

## Why this matters

A compiler for a clinical system cannot have silent type mismatches. If a property is declared as `int(0, 100)` and the compiler stores it as `string`, every downstream system — the runtime, the validation layer, the clinical decision support — receives the wrong type. Range constraints disappear. Numeric comparisons become string comparisons. The error is invisible until it produces a wrong result in a context where wrong results matter.

This was caught during development, not in production. It was caught because the playground made it visible — the starter example used `int(0, 100)` and the output showed `"type": "string"`. That immediate feedback loop, from source to compiled output in the browser, is exactly why the playground exists.

The fix is 20 tests and a warning diagnostic. The lesson is older: compilers must not have silent failure modes. Every unrecognised input should produce a diagnostic. Every type that looks like a type should either be accepted or rejected with a message. Silence is the enemy.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
