---
title: "Compiler 0.1.5: Eight Eyes Open"
slug: static-analysis-complete
description: Four new static analysis checks complete the compiler's structural verification layer. Every location, choice, section, and exit is now examined for unreachable nodes, impossible conditions, missing fallthrough, and naming collisions.
date: "2026-02-21"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Recording the completion of all eight static analysis checks and a playground fix they surfaced.
> Single canonical copy. February 2026.

## The gate

The [v1 completion gate](/articles/v1-completion-gate) defines eight static analysis checks, labelled S1 through S8. Four were already implemented across the LINK and IMPORT phases: S1 (undefined entity references), S2 (type mismatches), S5 (duplicate IDs), and S7 (circular imports). These are the hard errors — they block compilation.

The remaining four are softer. They detect structural problems that are not wrong in a strict sense but are almost certainly not what the author intended. A location with no path from the start. A choice whose condition can never be true. A section that will exhaust to silence. A section name that shadows an exit. These are warnings, not errors. The compiler produces valid output. But it tells you something looks off.

Version 0.1.5 implements all four.

## S3: Unreachable location (URD430)

The check builds a directed graph from the symbol table's exit map and runs breadth-first search from `world.start`. Any location not visited is unreachable.

```
warning[URD430]: Location 'island' is unreachable.
  No path from the start location reaches it.
```

This catches a common authoring mistake: adding a location and forgetting to connect it. In a large world with dozens of rooms, it is easy to leave a node floating. The check is conservative — it follows all exits regardless of runtime conditions. A door that requires a key still counts as a path. If the only way to reach a location is through a locked door, the location is reachable. The check only fires when there is literally no exit chain from the start.

When `world.start` is not declared, the check is skipped entirely. Without a root, reachability is undefined.

## S4: Orphaned choice (URD432)

A choice is orphaned when its guard condition tests an enum property against a value that does not exist in the enum's definition.

```
warning[URD432]: Choice in section 'actions' may never be available.
  Condition requires 'state' == 'locked' but type 'Door'
  only allows: [closed, open].
```

This is a pragmatic check, not a full satisfiability solver. It only examines `==` comparisons against enum properties where the LINK phase has fully resolved the type chain. If the property is not an enum, or the operator is not `==`, or the annotation was not resolved (because LINK already reported an error), the check skips it. The goal is to catch the obvious case — a typo in an enum value — without false positives.

The check recurses into nested choices. A deeply nested choice with an impossible condition is just as orphaned as a top-level one.

## S6: Missing fallthrough (URD433)

A section with only one-shot choices (`*`) and no terminal content will eventually exhaust. Once the player has selected every choice, the section has nothing left to display — no prose, no speech, no jump. The runtime would present an empty state.

```
warning[URD433]: Section 'greet' has only one-shot choices and
  no terminal jump or fallthrough text.
  It will exhaust to an empty state.
```

The check segments each file into sections at `SectionLabel` boundaries, looks up the section in the symbol table, and applies four conditions:

1. If any choice is sticky (`+`), the section is safe — sticky choices persist.
2. If there is a `-> jump` after the last choice, the section has an explicit exit.
3. If there is prose, entity speech, or a stage direction after the last choice, it serves as fallthrough content (what EMIT calls `on_exhausted`).
4. If the section has no choices at all, it is purely narrative and does not need fallthrough.

Only sections that fail all four conditions trigger the warning. The check is deliberately conservative about what counts as fallthrough: effects and conditions after the last choice do not count. Content nested inside a choice's body does not count — a jump inside the last choice protects that choice's path, not the section itself.

Three canonical test fixtures — the two-room key puzzle, the Monty Hall problem, and the interrogation scene — had sections that legitimately triggered this warning. Each had one-shot choices with no content after them. These were fixed by adding minimal fallthrough prose or speech, which is what the sections needed anyway.

## S8: Shadowed exit (URD434)

When a section and an exit share the same name within a location, jumps to that name will target the section, not the exit. This is by design — section names shadow exit directions in jump resolution. But it is almost never intentional.

```
warning[URD434]: Section 'north' in location 'tavern' shares a
  name with exit 'north'. Jumps to 'north' will target the
  section, not the exit. Use -> exit:north to target the exit
  explicitly.
```

The check walks the AST flat, tracking the current location context by slugifying each `LocationHeading`. When a `SectionLabel` is encountered, it checks whether the section name appears as an exit key in the current location's symbol table entry. The comparison is against the exit's direction key, not its destination — `-> north: Harbor` has key `north`, and section `== north` would shadow it, but section `== harbor` would not.

The warning message includes the resolution: `-> exit:north` is the explicit exit-qualified jump syntax that bypasses section shadowing.

## The playground fix

Implementing these checks exposed a gap in the playground. The compiler has three diagnostic severity levels — error, warning, and info — and the output pane already had distinct icons and colours for each. But the rendering logic was binary: if `success` is true, show the JSON; if false, show diagnostics. Since warnings do not set `success` to false, a compilation with warnings showed the JSON output and nothing else. The warnings were invisible.

The fix adds a warnings panel between the output header and the JSON pane. When compilation succeeds with warnings, an amber badge in the header shows the count, and the warnings appear as clickable rows above the JSON. Each row navigates to the source location, using the same diagnostic click handler as the error view. When there are no warnings, the output looks exactly as before.

This matters because the four new checks are all warnings. An author writing a world with an unreachable location would see the compiled JSON appear with no indication that anything was wrong. Now the warning is visible in the same view as the output, without blocking it.

## The numbers

Version 0.1.4 had 480 tests. Version 0.1.5 adds 36: 32 unit tests across the four checks and 4 end-to-end tests compiling negative fixtures. The total stands at 516, with a 100% pass rate.

All eight static analysis checks are now implemented. The "Static Analysis" section of the v1 completion gate is closed.

| Check | Code | Phase | Severity |
|-------|------|-------|----------|
| S1: Undefined entity reference | URD301 | LINK | Error |
| S2: Type mismatch | URD410+ | VALIDATE | Error |
| S3: Unreachable location | URD430 | VALIDATE | Warning |
| S4: Orphaned choice | URD432 | VALIDATE | Warning |
| S5: Duplicate IDs | URD302+ | LINK | Error |
| S6: Missing fallthrough | URD433 | VALIDATE | Warning |
| S7: Circular imports | URD202 | IMPORT | Error |
| S8: Shadowed exit | URD434 | VALIDATE | Warning |

The pattern is clean: the four hard errors catch things that would produce invalid output. The four warnings catch things that would produce confusing output. Together they form a structural verification layer that examines every location, choice, section, and exit in the world before the compiler emits a single line of JSON.

*This article is part of the Urd development journal at [urd.dev](https://urd.dev).*
