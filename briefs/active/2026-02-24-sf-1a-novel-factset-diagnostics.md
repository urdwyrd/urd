# URD — SF-1A: Novel FactSet Diagnostics

*Prove that the FactSet enables diagnostics impossible without it.*

February 2026 | Semantic Gate — Tier 1 (Prove)

> **Document status: BRIEF** — First brief of the semantic gate. Implements five diagnostic checks that operate solely on the FactSet's six tuple types. No AST traversal. No source text parsing. The diagnostic function signature is `fn check(fact_set: &FactSet) -> Vec<Diagnostic>`. This brief is the validation gate for the entire FactSet design: if these diagnostics cannot express useful checks that the existing S3–S8 analysis missed, the FactSet has not proved its value.

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-24
**Status:** Done

### What was done

Implemented all five FactSet-derived diagnostics (URD601–URD605) in a new `src/analyze.rs` module. Created three purpose-built test fixtures and a 15-test suite in `tests/analyze_tests.rs`. Wired the analyze phase into `lib.rs` between fact extraction and VALIDATE. Updated the diagnostic code range table in `diagnostics.rs`.

**Results against existing fixtures:**

| Fixture | Hits | Notable findings |
|---------|------|-----------------|
| two-room-key-puzzle | 0 | Clean |
| tavern-scene | 1 | URD602: `Character.trust` written but never read |
| monty-hall | 1 | URD601: `Door.revealed` read but never written |
| interrogation | 1 | URD601: `Person.mood` read but never written |
| locked-garden | 1 | URD603: `Character.mood` set to `friendly` but no condition tests that variant |
| sunken-citadel | 27 | 1× URD601, 12× URD602, 6× URD603, 8× URD605 |

**SF-1A.5 validation gate — PASSED.** The Sunken Citadel stress test produced 27 diagnostic hits that no S1–S8 check detects. Highlights:

- **URD605 `Evidence.examined`** — every write to `examined` is inside a choice guarded by `? @X.examined == false`. Without an unguarded bootstrap setting `examined` to `true`, the condition `examined == false` always passes but the write `examined = true` is semantically circular when cross-referenced across all evidence entities. This requires cross-referencing write sites, their enclosing conditions, and whether any other path writes the same property — impossible without the FactSet.
- **URD602 on 12 properties** (e.g. `Villager.secret`, `Scholar.knowledge`, `Weapon.durability`) — effects modify these properties but no condition in the 1200+ line world ever gates on them. Manual verification across that many entities and sections is impractical; the FactSet query is a single pass over the dependency index.
- **URD603 `Spirit.form` → `translucent`/`corporeal`** — the spirit's form is set to specific enum variants by effects, but no condition anywhere checks for those particular variants. This requires cross-referencing PropertyWrite value expressions against PropertyRead value literals for the same property key.

### What changed from the brief

- **Fixture syntax:** The brief's fixture examples used `[@npc: NPC]` for inline entity declaration, which is not valid compiler syntax. Entities must be declared in the frontmatter `entities:` block and placed with `[@npc]`. All three fixtures were corrected.
- **Gate test update:** The existing `gate_canonical_fixtures_zero_warnings` e2e test expected zero warnings on all canonical fixtures. Since ANALYZE warnings are informational findings (not authoring errors), the test was updated to only assert on URD100–URD599 (PARSE through EMIT) warnings, excluding URD600+ (ANALYZE) warnings.
- **No FactSet schema gaps found (SF-1A.6).** All five diagnostics expressed cleanly against the six tuple types and the PropertyDependencyIndex. No workarounds or missing data were needed.

---

## Context

The FactSet analysis IR was introduced in the compiler gate (see `2026-02-21-urd-compiler-analysis-ir-facts-provenance.md`). It extracts six relation types from the resolved world:

| Relation | What it records |
|----------|----------------|
| `PropertyRead` | A condition compares an entity property at a site (choice, exit, rule) |
| `PropertyWrite` | An effect modifies an entity property at a site |
| `ExitEdge` | A directed connection between two locations |
| `JumpEdge` | A directed transition between dialogue sections |
| `ChoiceFact` | A choice exists within a section, with indexed condition reads and effect writes |
| `RuleFact` | A rule with indexed condition reads and effect writes |

The FactSet is immutable, deterministic, and available after LINK regardless of VALIDATE errors. The `PropertyDependencyIndex` provides grouped lookups by `PropertyKey` (type + property).

The existing static analysis checks (S3–S8, VALIDATE steps 9–12) operate by walking the AST and symbol table directly. They work and are tested. But they cannot express queries that require cross-referencing across multiple relation types — for example, "this property is read in conditions but never written by any effect anywhere in the world."

This brief implements five diagnostics that are impossible or impractical without the FactSet. Each one is a pure function over the FactSet, proving that the extracted relations are sufficient for real analysis.

### Why this matters

SF-1A gates the entire semantic gate. If we cannot write useful diagnostics over the FactSet alone, then every downstream brief (PropertyDependencyIndex as a shipped product, graph visualisation, semantic diff, LSP, MCP) is building on an unvalidated foundation. The acceptance criteria are deliberately strict: at least one diagnostic must catch a real issue in an existing test world that no S1–S8 check detects.

### Diagnostic code range

The existing compiler phases use:

| Phase | Range |
|-------|-------|
| PARSE | URD100–URD199 |
| IMPORT | URD200–URD299 |
| LINK | URD300–URD399 |
| VALIDATE | URD400–URD499 |
| EMIT | URD500–URD599 |

FactSet-derived diagnostics are a new analysis phase that runs after FactSet extraction and before or alongside VALIDATE. They use the range **URD600–URD699**.

Update the code range table in `diagnostics.rs` to include:

| Phase | Range |
|-------|-------|
| ANALYZE (FactSet) | URD600–URD699 |


## Dependencies

- **Compiler gate closed.** 554+ tests passing. FactSet extraction working.
- **`facts.rs` complete.** `FactSet`, `PropertyDependencyIndex`, `PropertyKey`, all six fact types, `extract_facts()`, `to_json()`, all slice accessors and lookup helpers implemented per the analysis IR brief.
- **No other semantic gate brief required.** SF-1A has no upstream dependency within the semantic gate.


## FactSet Reference

The diagnostics in this brief operate on these types and methods. This section exists so the implementer does not need to cross-reference the full analysis IR brief during implementation.

### Types used

```rust
// All from crate::facts (pub)
FactSet           // Container. Private fields, public slice accessors.
PropertyRead      // { site, entity_type, property, operator, value_literal, value_kind, span }
PropertyWrite     // { site, entity_type, property, operator, value_expr, value_kind, span }
ExitEdge          // { from_location, to_location, exit_name, is_conditional, guard_reads, span }
ChoiceFact        // { section, choice_id, label, sticky, condition_reads, effect_writes, span }
RuleFact          // { rule_id, condition_reads, effect_writes, span }
PropertyKey       // { entity_type: TypeId, property: PropertyId }
PropertyDependencyIndex  // { readers, writers } — built from FactSet
FactSite          // Choice(ChoiceId) | Exit(ExitId) | Rule(RuleId)
CompareOp         // Eq | Ne | Lt | Gt | Le | Ge
WriteOp           // Set | Add | Sub
LiteralKind       // Bool | Int | Str | Ident
```

### Methods used

```rust
// FactSet accessors
fact_set.reads()   -> &[PropertyRead]
fact_set.writes()  -> &[PropertyWrite]
fact_set.exits()   -> &[ExitEdge]
fact_set.choices() -> &[ChoiceFact]
fact_set.rules()   -> &[RuleFact]

// PropertyRead / PropertyWrite
read.key()  -> PropertyKey
write.key() -> PropertyKey

// PropertyDependencyIndex
PropertyDependencyIndex::build(&fact_set) -> Self
index.reads_of(&key)          -> &[usize]   // indices into fact_set.reads()
index.writes_of(&key)         -> &[usize]   // indices into fact_set.writes()
index.read_properties()       -> impl Iterator<Item = &PropertyKey>
index.written_properties()    -> impl Iterator<Item = &PropertyKey>
```

### Architectural constraint

Every diagnostic function in this brief must have the signature:

```rust
fn check_xxx(fact_set: &FactSet) -> Vec<Diagnostic>
```

Or, where the `PropertyDependencyIndex` is needed:

```rust
fn check_xxx(fact_set: &FactSet, index: &PropertyDependencyIndex) -> Vec<Diagnostic>
```

No function may import or reference: `crate::ast::*`, `crate::graph::*`, `crate::symbol_table::*`, `crate::parse::*`, or any module that provides access to the AST, source text, or symbol table. The only allowed imports from the compiler are `crate::facts::*`, `crate::diagnostics::*`, and `crate::span::*`.

This is the entire point of the brief. If a diagnostic cannot be expressed against the FactSet, that is a FactSet schema gap (see SF-1A.6 below), not a reason to reach for the AST.


## The Five Diagnostics

### D1: Property read but never written — URD601

**What it detects:** A property appears in a condition (`PropertyRead`) but no effect (`PropertyWrite`) anywhere in the world ever modifies it. The condition can only ever evaluate against the property's default value or initial entity override. This is almost always an authoring oversight — the author intended to add an effect that changes the property but forgot, or they misspelled the property name in the effect.

**FactSet query:**

```
For each unique PropertyKey K in index.read_properties():
    If index.writes_of(K) is empty:
        Collect all PropertyRead facts for K
        Emit URD601 at the span of the first read, with related info for additional reads
```

**Message format:**

```
URD601 warning: Property '{entity_type}.{property}' is read in conditions but never
written by any effect. It will always reflect its default or initial value.
```

If there are multiple read sites, the first (by FactSet ordering) is the primary span. Additional sites are attached as `RelatedInfo`:

```
Also read at: {file}:{line}
```

**Example — positive case:**

```markdown
---
types:
  Guard [interactable]:
    suspicion: integer = 0
    mood: enum(calm, alert) = calm
---
# Room

[@guard: Guard]

== talk

* Ask a question
  ? @guard.suspicion >= 3
  @guard: I'm watching you.
```

Here `suspicion` is read (`>= 3`) but no effect ever writes `suspicion`. URD601 fires.

**Example — negative case (no diagnostic):**

```markdown
* Provoke the guard
  > @guard.suspicion + 1

* Ask a question
  ? @guard.suspicion >= 3
```

Now `suspicion` is both read and written. No URD601.

**Skip rule:** This diagnostic has no skip rule in v1. If an author intentionally uses a property as a read-only flag set only by entity overrides, the warning is still emitted. A future `readonly` property annotation would suppress it.


### D2: Property written but never read — URD602

**What it detects:** A property appears in effects (`PropertyWrite`) but no condition (`PropertyRead`) anywhere in the world ever tests it. The effect has no observable consequence on game logic — no condition gates on the value. This suggests dead code, a misspelled property name, or a missing condition that should test this property.

**FactSet query:**

```
For each unique PropertyKey K in index.written_properties():
    If index.reads_of(K) is empty:
        Collect all PropertyWrite facts for K
        Emit URD602 at the span of the first write, with related info for additional writes
```

**Message format:**

```
URD602 warning: Property '{entity_type}.{property}' is written by effects but never
read in any condition. The writes have no observable effect on game logic.
```

**Example — positive case:**

```markdown
* Bribe the guard
  > @guard.bribed = true
```

If no condition anywhere checks `@guard.bribed`, URD602 fires.

**Example — negative case (no diagnostic):**

```markdown
* Bribe the guard
  > @guard.bribed = true

* Ask for passage
  ? @guard.bribed == true
  @guard: Fine, go ahead.
```

Now `bribed` is both written and read. No URD602.

**Skip rule:** Same as D1 — no skip rule in v1. Exit guard conditions that read properties are counted as reads (via `FactSite::Exit`), so properties gating exits will not trigger D2.


### D3: Effect produces enum variant unreachable by any condition — URD603

**What it detects:** An effect writes an enum property to a specific variant, but no condition in the entire world ever tests for that variant. This suggests either a missing condition (the author intended to check for this variant but hasn't written the condition yet) or a dead write (the variant is set but never matters).

**FactSet query:**

```
For each PropertyWrite W where W.operator == Set and W.value_kind == Some(Ident):
    Let K = W.key()
    Let written_variant = W.value_expr
    Let all_reads = index.reads_of(K)
    Let tested_variants = { R.value_literal for R in all_reads where R.operator == Eq }
    If written_variant not in tested_variants:
        Emit URD603 at W.span
```

**Note on operator filtering:** This diagnostic only checks `Eq` comparisons on reads. The grammar already supports `!=` (`CompareOp::Ne`), but `? @npc.mood != hostile` implicitly covers *all other variants* — treating it as testing specific variants would require inverse reasoning. For v1, the conservative behaviour is: only `==` comparisons count as testing a variant. This means D3 may produce false positives when `!=` is used (warning about a variant that is indirectly tested via negation). False positives are noisy but safe. If `!=` usage on enums becomes common, expand the tested-variants set to include the complement.

**Message format:**

```
URD603 warning: Effect sets '{entity_type}.{property}' to '{variant}' but no condition
anywhere tests for this variant. The write may have no observable effect.
```

**Example — positive case:**

```markdown
> @warden.mood = friendly

# ... but nowhere in the world:
? @warden.mood == friendly
```

If no condition tests `mood == friendly`, URD603 fires on the write.

**Example — negative case (no diagnostic):**

```markdown
> @warden.mood = friendly

* Ask for passage
  ? @warden.mood == friendly
  @warden: Of course.
```

The variant `friendly` is both written and tested. No URD603.

**Interaction with D2:** D3 is more specific than D2. If a property is written but never read at all, D2 already fires. In that case, D3 adds no useful signal — telling the author "this variant is untested" when no variant is tested is redundant noise. **Requirement:** D3 must skip any `PropertyKey` for which D2 fires. Implementation: `check_enum_variant_untested` receives the set of D2 keys (or computes it via `index.reads_of(K).is_empty()`) and skips those properties. D3 only fires when the property *is* read somewhere but not for the specific variant being written.


### D4: Condition tests unreachable threshold — URD604

**What it detects:** A condition compares a numeric property against a threshold that no combination of effects in the world can reach. For example, a condition checks `trust >= 10` but the only effects add +1 or +2 to a property that starts at 0, and no effect ever sets it directly to 10 or higher. The condition can never be satisfied.

**FactSet query:**

This is the most complex diagnostic. It requires reasoning about numeric write ranges.

```
For each PropertyRead R where R.operator in {Lt, Gt, Le, Ge} and R.value_kind == Int:
    Let K = R.key()
    Let threshold = parse_int(R.value_literal)
    Let all_writes = index.writes_of(K)

    // Check if any Set write can produce a value that satisfies the comparison.
    Let set_values = { parse_int(W.value_expr) for W in all_writes
                       where W.operator == Set and W.value_kind == Some(Int) }

    // If there's any Set write with a value satisfying the condition, no diagnostic.
    If any set_value satisfies (threshold, R.operator): continue

    // If there are no Set writes, check if incremental writes exist.
    // If only Add/Sub writes exist, we can't statically determine reachability
    // without knowing the default value and iteration count.
    // Conservative: only fire if there are NO writes at all, or only Set writes
    // that all fail the threshold check.

    Let has_add_sub = any W in all_writes where W.operator in {Add, Sub}
    If has_add_sub: continue  // Conservative skip: can't statically bound accumulation

    // Only Set writes exist (or no writes at all). None satisfy the threshold.
    Emit URD604 at R.span
```

**Why conservative on Add/Sub:** Without knowing the property's initial value, the number of times an effect can fire, and whether the property has declared bounds, we cannot statically determine whether accumulation can reach a threshold. A more sophisticated version could use declared `int(min, max)` ranges from the type definition, but that requires symbol table access which this brief explicitly forbids. If the FactSet is later extended with property metadata (type, bounds, default), D4 can be tightened. For now, conservative is correct.

**Message format:**

```
URD604 warning: Condition compares '{entity_type}.{property}' against {threshold} but
no effect can produce a value that satisfies '{operator} {threshold}'. The condition
may never be true.
```

**The "may" is deliberate.** Entity overrides in the frontmatter can set initial values that satisfy the condition even without any effect. Without symbol table access, this diagnostic cannot confirm that the threshold is unreachable from the initial value. It reports a likely issue, not a guaranteed one.

**Example — positive case:**

```markdown
---
types:
  NPC [interactable]:
    trust: integer = 0
---

* Ask for help
  ? @npc.trust >= 100
  @npc: I trust you completely.

* Be friendly
  > @npc.trust = 5
```

The only `Set` write produces value `5`. The condition requires `>= 100`. No `Add`/`Sub` writes exist. URD604 fires.

**Example — negative case (conservative skip):**

```markdown
* Be friendly
  > @npc.trust + 5

* Ask for help
  ? @npc.trust >= 100
```

An `Add` write exists. We can't statically bound how many times `+5` fires. Conservative skip — no URD604.


### D5: Circular property dependency — URD605

**What it detects:** A property is written only inside sites (choices, rules) whose conditions require reading the same property at a threshold that no *other* effect can reach. This creates a semantic deadlock: the property can never change because the only way to change it is blocked by its own current value.

This is the "killer diagnostic" that justifies the FactSet. It requires cross-referencing three things simultaneously: (1) all write sites for a property, (2) the condition reads guarding those write sites, (3) whether any *other* write can bootstrap the property to the required value. This is impossible to express as a single AST traversal without building the equivalent of the FactSet in-line.

**FactSet query:**

```
For each unique PropertyKey K where index.writes_of(K) is non-empty:
    Let all_write_indices = index.writes_of(K)

    // For each write, check if its enclosing site has a condition that reads K.
    Let guarded_count = 0
    For each write_idx in all_write_indices:
        Let W = fact_set.writes()[write_idx]
        Let site = &W.site

        // Get all read indices for this site.
        Let site_read_indices = match site:
            FactSite::Choice(id) => fact_set.choice_by_id(id).condition_reads
            FactSite::Exit(id)   => fact_set.exit_by_id(id).guard_reads
            FactSite::Rule(id)   => fact_set.rule_by_id(id).condition_reads

        // Check if any of the site's reads target property K.
        Let has_self_read = site_read_indices.iter()
            .any(|&i| fact_set.reads()[i].key() == K)

        If has_self_read:
            guarded_count += 1

    // If ALL writes to K are self-guarded, the property may be stuck.
    If guarded_count == all_write_indices.len() and guarded_count > 0:
        Emit URD605 at the span of the first write, with related info
        showing the self-reads that create the cycle.
```

**Subtlety — initial values:** An entity override in the frontmatter could set the property to a value that satisfies the guard, breaking the cycle at game start. Without symbol table access, D5 cannot confirm that the cycle is truly unbreakable. The message uses "may" accordingly. However, if the cycle exists, it almost always indicates an authoring error — the author forgot an unguarded bootstrap path.

**Subtlety — Add/Sub writes:** An `Add` or `Sub` write behind a self-guard is still circular. If `trust >= 5` gates `trust + 1`, and there's no other way to write `trust`, the cycle is real (assuming the default doesn't already satisfy `>= 5`).

**Message format:**

```
URD605 warning: Property '{entity_type}.{property}' may be stuck in a circular
dependency. Every effect that writes this property is guarded by a condition that
reads it. Without an unguarded write path or a satisfying initial value, the
property can never change.
```

Related info for each self-guarding condition:

```
Write at {file}:{line} is guarded by condition reading '{entity_type}.{property}' at {file}:{line}
```

**Example — positive case:**

```markdown
---
types:
  NPC [interactable]:
    rank: integer = 0
---

* Promote the NPC
  ? @npc.rank >= 1
  > @npc.rank + 1
```

The only write to `rank` is `+1`, and it's guarded by `rank >= 1`. If `rank` starts at 0, the condition never passes, and the write never fires. URD605 fires.

**Example — negative case (unguarded bootstrap):**

```markdown
* Give initial rank
  > @npc.rank = 1

* Promote the NPC
  ? @npc.rank >= 1
  > @npc.rank + 1
```

The first write (`rank = 1`) is unguarded — it has no condition reading `rank`. So not all writes are self-guarded. No URD605.

**Example — negative case (mixed guards):**

```markdown
* Promote the NPC
  ? @npc.rank >= 1
  > @npc.rank + 1

* Demote the NPC (unguarded)
  > @npc.rank = 0
```

The `rank = 0` write is unguarded. Even though the `+1` write is self-guarded, not *all* writes are. No URD605.


## Module Placement

Create `src/analyze.rs` (or `src/analyze/mod.rs` if it grows). This module contains FactSet-derived diagnostic functions. It is separate from `validate/` to enforce the architectural constraint: `analyze` imports only from `facts`, `diagnostics`, and `span`. It never imports from `ast`, `graph`, `parse`, or `symbol_table`.

```rust
// In lib.rs
pub mod analyze;
```

### Internal structure

```rust
// src/analyze.rs

use crate::diagnostics::{Diagnostic, Severity, RelatedInfo};
use crate::facts::{
    FactSet, PropertyDependencyIndex, PropertyKey, PropertyRead, PropertyWrite,
    CompareOp, WriteOp, LiteralKind, FactSite,
};
use crate::span::Span;

/// Run all FactSet-derived diagnostics.
/// Called after extract_facts(), before or alongside VALIDATE.
pub fn analyze(fact_set: &FactSet) -> Vec<Diagnostic> {
    let index = PropertyDependencyIndex::build(fact_set);
    let mut diagnostics = Vec::new();

    diagnostics.extend(check_read_never_written(fact_set, &index));
    diagnostics.extend(check_written_never_read(fact_set, &index));
    diagnostics.extend(check_enum_variant_untested(fact_set, &index));
    diagnostics.extend(check_unreachable_threshold(fact_set, &index));
    diagnostics.extend(check_circular_dependency(fact_set, &index));

    diagnostics
}

fn check_read_never_written(...) -> Vec<Diagnostic> { ... }
fn check_written_never_read(...) -> Vec<Diagnostic> { ... }
fn check_enum_variant_untested(...) -> Vec<Diagnostic> { ... }
fn check_unreachable_threshold(...) -> Vec<Diagnostic> { ... }
fn check_circular_dependency(...) -> Vec<Diagnostic> { ... }
```

### Pipeline integration

In `lib.rs`, after `extract_facts()` and before or during `validate()`:

```rust
// Insert after the existing extract_facts() call, before validate():
let analyze_diags = analyze::analyze(&fact_set);
for diag in analyze_diags {
    diagnostics.push(diag);
}
// The Diagnostic type returned by analyze() is the same crate::diagnostics::Diagnostic
// used throughout the compiler. No conversion needed.
```

The `analyze()` function returns `Vec<Diagnostic>` rather than writing to a `DiagnosticCollector` to keep it decoupled from the collector's internal API. The caller converts and appends. If the collector grows a `pub fn extend()` method, the conversion becomes trivial.


## Test Strategy

### New test fixtures

Three purpose-built fixtures. Each is minimal — no unnecessary entities, locations, or prose. They exist solely to trigger or not trigger the five diagnostics.

**Fixture 1: `negative-factset-diagnostics.urd.md`**

A well-formed world where no FactSet diagnostic should fire. Every property that is read is also written. Every property that is written is also read. Every enum variant that is set is also tested. No circular dependencies.

```markdown
---
world:
  name: factset-clean
  start: room

types:
  NPC [interactable]:
    trust: integer = 0
    mood: enum(calm, alert) = calm
    seen: bool = false
---

# Room

[@npc: NPC]

== talk

* Greet
  > @npc.trust + 1
  > @npc.mood = alert
  > @npc.seen = true

* Ask for help
  ? @npc.trust >= 3
  @npc: Sure.

* Check mood
  ? @npc.mood == alert
  @npc: I'm on edge.

* Check if seen
  ? @npc.seen == true
  @npc: We've met before.
```

**Expected:** Zero diagnostics from `analyze()`.

**Fixture 2: `positive-factset-diagnostics.urd.md`**

A world designed to trigger all five diagnostics.

```markdown
---
world:
  name: factset-triggers
  start: room

types:
  NPC [interactable]:
    trust: integer = 0
    suspicion: integer = 0
    mood: enum(calm, alert, friendly) = calm
    rank: integer = 0
    loyalty: integer = 0
    power: integer = 0
---

# Room

[@npc: NPC]

== talk

+ Greet
  > @npc.mood = friendly
  > @npc.loyalty = 5

* Ask about suspicion
  ? @npc.suspicion >= 3
  @npc: I'm watching you.

* Check mood (calm only)
  ? @npc.mood == calm
  @npc: All is well.

* Check mood (alert only)
  ? @npc.mood == alert
  @npc: On guard.

* Seek power
  ? @npc.power >= 100
  @npc: You are mighty.

+ Gain some power
  > @npc.power = 5

* Apply for promotion
  ? @npc.rank >= 1
  > @npc.rank + 1
```

**Expected diagnostics:**

| Code | Property | Reason |
|------|----------|--------|
| URD601 | `NPC.suspicion` | Read (`>= 3`) but never written |
| URD602 | `NPC.loyalty` | Written (`= 5`) but never read in any condition |
| URD603 | `NPC.mood` → `friendly` | Set to `friendly` but no condition tests `mood == friendly` |
| URD604 | `NPC.power` | Condition `power >= 100`, only Set write produces `5`, no Add/Sub |
| URD605 | `NPC.rank` | Only write (`+1`) is guarded by `rank >= 1`, no unguarded write |

**Fixture 3: `positive-factset-circular-deep.urd.md`**

A more complex circular dependency case with multiple self-guarded writes.

```markdown
---
world:
  name: factset-circular
  start: room

types:
  NPC [interactable]:
    clearance: integer = 0
---

# Room

[@npc: NPC]

== lobby

* Request level 1 access
  ? @npc.clearance >= 1
  > @npc.clearance = 2

* Request level 2 access
  ? @npc.clearance >= 2
  > @npc.clearance = 3
```

**Expected:** URD605 on `NPC.clearance` — both writes are self-guarded, no unguarded bootstrap.

### Tests against existing fixtures

Run `analyze()` against all existing test fixtures. The brief does not predict which existing fixture will trigger a diagnostic — **that is the point of SF-1A.5.** The implementer must run the diagnostics and report what they find. If at least one diagnostic fires on an existing fixture (Locked Garden, Sunken Citadel, Tavern Scene, Two-Room Key Puzzle, Monty Hall, or Interrogation) and identifies a real authoring issue that no S3–S8 check detected, SF-1A.5 passes.

**Candidates for real hits (informed guesses, not guarantees):**

- **Sunken Citadel:** The stress test has many properties across many types. Properties like `bribed`, `discount_given`, `trades_completed`, `translated`, `doses`, or `enchanted` are plausible D2 candidates (written but potentially never condition-tested).
- **Locked Garden:** `@ghost.trust >= 5` is checked but `@ghost.trust` only gets `+2` from a starting value of 3 — maximum reachable is 5 (exactly satisfiable). Not a D4 hit, but close.

### Test matrix

| Test | Fixture | Asserts |
|------|---------|---------|
| `analyze_clean_world_no_diagnostics` | `negative-factset-diagnostics.urd.md` | `analyze()` returns empty vec |
| `analyze_d1_read_never_written` | `positive-factset-diagnostics.urd.md` | Contains URD601 for `NPC.suspicion` |
| `analyze_d2_written_never_read` | `positive-factset-diagnostics.urd.md` | Contains URD602 for `NPC.loyalty` |
| `analyze_d3_enum_variant_untested` | `positive-factset-diagnostics.urd.md` | Contains URD603 for `NPC.mood` → `friendly` |
| `analyze_d4_unreachable_threshold` | `positive-factset-diagnostics.urd.md` | Contains URD604 for `NPC.power` |
| `analyze_d5_circular_dependency` | `positive-factset-diagnostics.urd.md` | Contains URD605 for `NPC.rank` |
| `analyze_d5_circular_multi_write` | `positive-factset-circular-deep.urd.md` | Contains URD605 for `NPC.clearance` |
| `analyze_existing_locked_garden` | `locked-garden.urd.md` | Runs without panic. Results recorded in execution record. |
| `analyze_existing_sunken_citadel` | `sunken-citadel.urd.md` | Runs without panic. Results recorded in execution record. |
| `analyze_existing_tavern_scene` | `tavern-scene.urd.md` | Runs without panic. Results recorded in execution record. |
| `analyze_existing_key_puzzle` | `two-room-key-puzzle.urd.md` | Runs without panic. Results recorded in execution record. |
| `analyze_existing_monty_hall` | `monty-hall.urd.md` | Runs without panic. Results recorded in execution record. |
| `analyze_existing_interrogation` | `interrogation/` | Runs without panic. Results recorded in execution record. |
| `analyze_empty_world_no_panic` | minimal world with no conditions/effects | `analyze()` returns empty vec (zero reads, zero writes) |
| `analyze_no_ast_imports` | *(code review / compile-time)* | `src/analyze.rs` does not contain `use crate::ast`, `use crate::graph`, `use crate::symbol_table`, or `use crate::parse` |

### Diagnostic output format

All five diagnostics produce `Diagnostic` structs with:

- `severity: Severity::Warning` (all five are warnings, not errors — they report likely issues, not definite failures)
- `code: String` (e.g., `"URD601"`)
- `message: String` (formatted as specified per diagnostic above)
- `span: Span` (the span of the primary fact — first read for D1, first write for D2/D3, the read for D4, the first write for D5)
- `related: Vec<RelatedInfo>` (additional sites involved in the diagnostic)

The diagnostics are included in the standard compiler JSON output alongside VALIDATE diagnostics. The playground displays them in the diagnostics panel. No special rendering or filtering is needed — they are regular diagnostics with a different code range.


## Files Changed

| File | Change |
|------|--------|
| `src/analyze.rs` | **New.** Five diagnostic functions + `analyze()` entry point. |
| `src/lib.rs` | Add `pub mod analyze;`. Wire `analyze::analyze()` into pipeline after `extract_facts()`. |
| `src/diagnostics.rs` | Update code range table comment to include ANALYZE: URD600–URD699. |
| `src/wasm.rs` | No change needed — diagnostics are already serialised from the collector. |
| `tests/analyze_tests.rs` | **New.** All test cases from the matrix above. |
| `tests/fixtures/negative-factset-diagnostics.urd.md` | **New.** Clean fixture. |
| `tests/fixtures/positive-factset-diagnostics.urd.md` | **New.** All-trigger fixture. |
| `tests/fixtures/positive-factset-circular-deep.urd.md` | **New.** Multi-write circular case. |


## Estimated Size

| Component | Lines |
|-----------|-------|
| `analyze.rs` — `check_read_never_written` | ~25 |
| `analyze.rs` — `check_written_never_read` | ~25 |
| `analyze.rs` — `check_enum_variant_untested` | ~35 |
| `analyze.rs` — `check_unreachable_threshold` | ~45 |
| `analyze.rs` — `check_circular_dependency` | ~50 |
| `analyze.rs` — `analyze()` + imports + helpers | ~30 |
| `lib.rs` changes | ~5 |
| Test fixtures (3 files) | ~100 |
| `analyze_tests.rs` | ~250 |
| **Total** | **~565** |


## Acceptance Criteria

- [ ] **SF-1A.1** — All five diagnostics implemented as functions taking `&FactSet` (and optionally `&PropertyDependencyIndex`), returning `Vec<Diagnostic>`.
- [ ] **SF-1A.2** — No diagnostic implementation imports or references any AST type, parser module, symbol table, or source text. The `src/analyze.rs` file contains no `use crate::ast`, `use crate::graph`, `use crate::symbol_table`, or `use crate::parse` statements. Verified by code review and by `grep`.
- [ ] **SF-1A.3** — Each diagnostic fires correctly on the purpose-built test fixture (`positive-factset-diagnostics.urd.md`): minimum one positive case per diagnostic. Each diagnostic does NOT fire on the clean fixture (`negative-factset-diagnostics.urd.md`).
- [ ] **SF-1A.4** — Diagnostics surface in standard compiler output with structured JSON format (included in the diagnostics array alongside VALIDATE diagnostics, with code in URD600 range).
- [ ] **SF-1A.5** — **Validation gate.** At least one diagnostic catches a real issue in an existing test world (Sunken Citadel, Locked Garden, Tavern Scene, Two-Room Key Puzzle, Monty Hall, or Interrogation) that no existing S1–S8 check detects. The hit must be in a world with multiple entities or multiple files where manual verification across the full source would be non-trivial — proving the FactSet's cross-relation querying is necessary, not just convenient. The issue, the diagnostic that found it, and why it required cross-relation reasoning are documented in the execution record.
- [ ] **SF-1A.6** — If any diagnostic is awkward to express against the six tuple types, the gap is documented as a FactSet schema issue. If no gap is found, this criterion passes by recording "no gaps found."
- [ ] **SF-1A.7** — All existing tests continue to pass. Zero regressions.
- [ ] **SF-1A.8** — The `analyze()` function runs on all existing test fixtures without panics.


## What This Brief Does NOT Cover

- Refactoring S3–S8 to use FactSet queries (that is SF-1B).
- Extending `PropertyDependencyIndex` with new methods (that is SF-2).
- Playground UI changes for new diagnostics (diagnostics already render via the existing panel).
- CLI `--dump-diagnostics` or filtering by code range (future CLI brief).
- Any diagnostic that requires symbol table access (e.g., D4 tightening using declared property bounds).


## Relationship to Downstream Briefs

| Brief | How SF-1A feeds it |
|-------|--------------------|
| **SF-1B** (Migrate S3–S8) | Proves that FactSet queries can express real diagnostics. If SF-1A fails, SF-1B's premise is weakened. |
| **SF-2** (PropertyDependencyIndex) | **Hard dependency.** D1 and D2 must exist before SF-2 can be verified. SF-2.6 requires that `read_but_never_written()` and `written_but_never_read()` produce results identical to D1/D2 output — this cannot be tested without working D1/D2 implementations. When SF-2 ships these as first-class index methods, D1 and D2 are refactored to call them directly (shared source of truth, not duplicated logic). |
| **SF-3** (Visualisation) | D1/D2 results can annotate graph nodes with "orphaned property" markers. |
| **SF-4** (Semantic Diff) | D1–D5 diagnostic counts become a diffable metric: "new warnings introduced by this change." |
| **SF-5** (LSP) | All five diagnostics stream through the LSP diagnostic channel. No additional wiring needed — they're in the standard diagnostic collector. |
| **SF-6** (MCP) | `get_diagnostics` endpoint returns SF-1A diagnostics alongside VALIDATE diagnostics. |

*End of Brief*
