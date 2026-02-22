# URD — Specification Consistency Audit (E1–E7) and Compiler Gate Closure

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-22
**Status:** Complete

### What was done

1. Read all nine normative documents and three non-authoritative documents referenced in the brief.
2. Performed E1–E7 audit with findings recorded. Results: 4 PASS, 2 FAIL MINOR, 1 INFO.
3. Performed the critical contract check (on_exhausted). Found a CRITICAL divergence: compiler emits `goto` on `on_exhausted` but JSON Schema (`speech` type with `additionalProperties: false`) disallows it and Schema Spec doesn't document it. No canonical fixture triggers this path, explaining why the gate test passed.
4. Performed all 8 cross-document consistency checks. Results: 7 PASS, 1 FAIL MINOR (advance modes).
5. Applied four spec edits to resolve findings:
   - JSON Schema: added `exhaustedContent` definition with optional `goto`; updated `on_exhausted` reference
   - Schema Spec: added `goto` to `on_exhausted` description; added `auto` and `manual` to advance modes table
   - Wyrd Reference Runtime: added failure contract paragraph referencing AB §Layer 2
   - Architecture: added architectural layers paragraph referencing AB §The Three Layers
6. Re-ran all tests: 554 compiler tests pass, 39 schema tests pass (including after JS edit).
7. Updated gate document (both `docs/` and `content/documents/` copies): all Specification Gate items checked off, implementation sequence items 5 and 8 struck through, document status changed from PLANNING to CLOSED, added Gate Closed section with audit record table.

### What changed from the brief

1. **No compiler code changes were needed.** The on_exhausted CRITICAL finding was resolved by aligning the spec and schema to match the compiler's existing behaviour, not by changing the compiler. The `goto` field on `on_exhausted` is a legitimate feature (hub-and-spoke dialogue pattern) that was under-documented.
2. **E7 was resolved as "skip" with a clear rationale** rather than adding a paragraph to SS. The FactSet analysis IR is the canonical formalisation of graph structure.
3. **The five editorial improvements suggested by the user** (rename secondary references, add tie-breaker, simplify status/severity, add re-audit condition, repo-relative paths) were already incorporated into the brief before execution began.

---

**Created:** 2026-02-22

## Context

The compiler gate (urd-v1-completion-gate.md) has all implementation items checked off:

- C1–C9: ✓ All nine compiler requirements implemented and tested.
- S1–S8: ✓ All eight static analysis checks implemented and tested.
- F1–F8: ✓ FactSet analysis IR implemented and verified.
- Canonical fixtures: ✓ Five fixtures compile with zero warnings.
- JSON Schema validation: ✓ All fixtures validate against published schema.
- Negative corpus: ✓ Nine fixtures rejected with correct codes and spans.

Three items remain in the **Specification Gate**:

- [ ] All E1–E7 consistency items verified or resolved.
- [ ] No contradictions between Schema Spec, JSON Schema, Schema Markdown Spec, Wyrd Spec, and Architectural Boundaries.
- [ ] Published JSON Schema matches all compiler outputs.

The third item is already verified by the `gate_json_schema_validates_all_fixtures` test from compiler 0.1.7. This brief covers the first two: the E1–E7 audit and a systematic cross-document consistency check.

This is the final brief before the compiler is declared v1 complete.

### Nature of this brief

This is a **documentation audit**, not a code brief. The deliverable is a completed audit table recording what was checked, what was found, and what (if anything) was fixed. If any contradictions are found that require spec edits, those edits are in scope. If a contradiction requires compiler changes, a separate brief is needed — but this is not expected given the maturity of both specs and implementation.

### Scope boundary

This brief covers:

- Systematic verification of E1–E7.
- Cross-document consistency check across all normative and reference documents.
- Spec edits to resolve any contradictions found.
- Final gate document update: check off all Specification Gate items, change document status from PLANNING to CLOSED.

This brief does NOT cover:

- Compiler code changes (unless a contradiction reveals a bug).
- Runtime or system gate items.
- New features or extensions.


## Audit Rules

These rules govern how the audit is conducted and how disagreements are resolved.

### Rule 1: Pass criteria

A PASS requires both semantic alignment and absence of ambiguity, not just absence of contradiction. Two documents can "not contradict" but still be unclear or differently interpretable. Ambiguity at this stage is treated as a finding that must be resolved or explicitly accepted with a rationale.

### Rule 2: Document authority hierarchy

When documents disagree, the following authority applies:

| Domain | Authoritative document | Abbreviation |
|--------|----------------------|--------------|
| Data model semantics (what `.urd.json` contains) | Schema Specification | SS |
| Structural validation (what JSON Schema enforces) | JSON Schema | JS |
| Authoring syntax (what `.urd.md` can express) | Schema Markdown Specification | SM |
| Layer boundaries (what belongs where) | Architectural Boundaries | AB |
| Runtime behaviour (what Wyrd does) | Wyrd Reference Runtime | WR |
| Pipeline structure (how components connect) | Architecture | AR |
| Diagnostic codes (what the compiler reports) | Diagnostic Code Reference | DC |
| PEG grammar (formal parse rules) | Formal Grammar Brief | FG |

When a contradiction is found, the authoritative document wins. The non-authoritative document is edited to align. If the authoritative document itself is wrong (discovered during audit), that edit is recorded with explicit justification.

**Tie-breaker:** If two authoritative documents conflict within the same domain, AB wins for boundary questions, SS wins for semantic questions, then JS for enforcement questions. This ordering resolves rare edge cases without escalation.

### Rule 3: No silent divergence

Any field present in compiler output must be either explicitly defined in the Schema Specification or explicitly documented as an extension. "It works but isn't documented" is a finding that blocks gate closure.

This rule applies symmetrically: any field defined in the Schema Specification or JSON Schema that the compiler never emits must be documented as "valid for hand-authored JSON but not produced by the compiler" or removed.

### Rule 4: Severity classification

Findings are classified by severity:

| Severity | Definition | Gate impact |
|----------|-----------|-------------|
| **CRITICAL** | Semantic contradiction between documents, or compiler output diverges from spec. | Blocks gate closure. Must be fixed and re-audited. |
| **MINOR** | Terminology inconsistency, missing cross-reference, wording ambiguity. | Can be fixed inline during audit without reopening gate. |
| **INFO** | Observation or recommendation. No contradiction. | Recorded for future reference. Does not affect gate. |

Findings use **Status** (PASS or FAIL only) and **Severity** (CRITICAL, MINOR, or INFO — recorded only when Status is FAIL). The combination PASS + severity should never exist.


## Documents in Scope

All canonical documents are in `docs/` at the repo root. The JSON Schema is in `packages/schema/`.

### Primary audit documents (normative)

| Document | File | Abbreviation | Status |
|----------|------|-------------|--------|
| Schema Specification | `schema-spec.md` | **SS** | Normative |
| Schema Markdown Syntax Specification | `schema-markdown.md` | **SM** | Normative |
| JSON Schema | `packages/schema/urd-world-schema.json` | **JS** | Normative |
| Architecture | `architecture.md` | **AR** | Normative |
| Architectural Boundaries | `urd-architectural-boundaries.md` | **AB** | Normative governance |
| Wyrd Reference Runtime | `wyrd-reference-runtime.md` | **WR** | Normative |
| Test Case Strategy | `urd-test-case-strategy.md` | **TC** | Normative |
| Formal Grammar Brief | `urd-formal-grammar-brief.md` | **FG** | Normative |
| Diagnostic Code Reference | `urd-diagnostic-codes.md` | **DC** | Reference |

### Non-authoritative documents (checked for consistency but do not govern)

| Document | File | Why included |
|----------|------|-------------|
| Compiler Architecture Brief | `urd-compiler-architecture-brief.md` | Defines phase pipeline, AST, and symbol table. Should not contradict AR or SS. |
| Product Vision | `product-vision.md` | Referenced in E6 (lambda framing). Informative, not normative. |
| Future Proposals | `future-proposals.md` | AB references it for rejected proposals. E6 may touch it. |

### Explicitly out of scope

| Document | Why excluded |
|----------|-------------|
| System Gate (`urd-system-gate.md`) | Runtime gate, not compiler gate. Separate closure process. |
| Five phase briefs (parse, import, link, validate, emit) | Implementation briefs, derivative of normative specs. |
| Nested Dialogue (`nested-dialogue.md`) | Design exploration. Decisions codified in SM. |
| Reference cards (writers, engine developers) | Informative summaries. Not normative. |
| Pain Points Report, World Framework Research | Market research. Not normative. |


## Part 1: E1–E7 Audit

Each item below requires the auditor to read the referenced sections in the relevant documents, verify the claim, and record the finding.

### E1: Three-layer model consistency

**Claim:** The three-layer model (Urd, Wyrd, Adapter) must be consistent across all docs.

**Action:** Verify the Architecture doc names the adapter layer explicitly.

**Documents to check:**
- **AB** §The Three Layers — defines the three layers with diagrams and responsibilities. This is the canonical source.
- **AR** §System Overview — describes the pipeline. Check that it names Urd (schema + compiler), Wyrd (runtime), and the adapter/presentation layer as three distinct layers with the same responsibilities as AB.
- **WR** §Architecture — describes Wyrd's three internal layers (Core Engine, Presentation, Extension Host). **Critical check:** Verify WR does not present its internal Presentation layer as equivalent to AB's external Adapter layer. WR's Presentation is a default adapter shipped with Wyrd, not the adapter concept itself. If WR conflates these, it creates architectural drift where future implementers think "Wyrd includes the adapter" rather than "Wyrd ships with a default adapter."
- **SS** — check whether it references the three-layer model or assumes a two-layer model.
- **SM** — check whether it references layers at all.

**What to look for:**
- Does AR use the word "adapter"? Or does it only say "presentation layer"?
- Does WR clearly distinguish between its internal presentation layer and the external adapter concept?
- Are there any documents that describe a two-layer model (schema + runtime) without acknowledging the adapter?

**Recording format:**
```
E1 Finding:
- AR: [consistent/inconsistent] — [detail]
- WR: [consistent/inconsistent] — [detail]
- SS: [consistent/inconsistent/not referenced] — [detail]
- SM: [consistent/inconsistent/not referenced] — [detail]
- Severity: [CRITICAL/MINOR/INFO]
- Fix required: [yes/no] — [what to change]
```

### E2: Failure contract

**Claim:** The failure contract (structured result, two categories, no mutation, no event) is now specified in the governance doc. Verify the Wyrd Reference Runtime spec includes or references this.

**Action:** Verify WR specifies or references the failure contract from AB.

**Documents to check:**
- **AB** §Layer 2: Wyrd — defines the failure contract in detail: two categories (request validation failures, world state failures), no state mutation, no events on failure, reason codes, condition references.
- **WR** — search for "failure", "perform", "error", "result". Check whether WR defines its own failure contract, references AB's, or is silent.

**What to look for:**
- Does WR define a `perform()` return type that includes failure cases?
- Does WR distinguish between request validation failures and world state failures?
- Does WR state that failures never appear in the event stream?
- If WR predates AB's failure contract (it may — AB was finalised later), is there a contradiction or just an omission?

**Recording format:**
```
E2 Finding:
- WR failure contract: [present/absent/partial]
- Consistent with AB: [yes/no/N/A]
- Severity: [CRITICAL/MINOR/INFO]
- Fix required: [yes/no] — [what to change]
```

### E3: `on_enter` / `on_exit` in JSON Schema

**Claim:** `on_enter` / `on_exit` were added to the JSON Schema as an erratum. Verify the Schema Spec prose matches.

**Action:** Verify that SS describes `on_enter` and `on_exit` on locations, and that JS includes them.

**Documents to check:**
- **JS** `locationsBlock` — confirmed present: `on_enter` and `on_exit` as arrays of effects.
- **SS** §Locations — check whether prose describes `on_enter` / `on_exit` effects on locations.
- **SM** — check whether the Schema Markdown syntax supports declaring `on_enter` / `on_exit` in `.urd.md` files.
- **AR** — check whether the compiler phase descriptions mention `on_enter` / `on_exit`.

**What to look for:**
- JS already has `on_enter` and `on_exit` in the `locationsBlock` definition (confirmed).
- Does SS describe these fields? If SS is silent, the JSON Schema has a field that the spec doesn't document — a silent divergence (Rule 3 violation).
- Does the compiler actually emit these fields? If not, the JSON Schema allows them but the compiler never produces them — which is fine for hand-authored JSON but should be documented.
- Does SM define syntax for declaring location enter/exit effects?

**Recording format:**
```
E3 Finding:
- JS: [present/absent]
- SS prose: [present/absent/partial]
- SM syntax: [present/absent]
- Compiler emits: [yes/no]
- Severity: [CRITICAL/MINOR/INFO]
- Fix required: [yes/no] — [what to change]
```

### E4: `on_condition` regex pattern

**Claim:** The `on_condition` regex pattern in JSON Schema was overly restrictive. Verify the fix is in the published schema.

**Action:** Check the `advance` property pattern in JS's `phase` definition.

**Documents to check:**
- **JS** `phase.advance` — read the regex pattern.

**What to look for:**
- The current pattern is: `^(on_action|on_rule|on_condition .+|end|auto|manual)$`
- `on_condition .+` allows any non-empty string after `on_condition `. This is the correct, relaxed pattern.
- If the pattern were more restrictive (e.g., requiring specific comparison operators or entity reference formats), it would reject valid condition expressions. Verify `.+` is present.

**Recording format:**
```
E4 Finding:
- JS advance pattern: [exact pattern]
- Sufficiently permissive: [yes/no]
- Severity: [CRITICAL/MINOR/INFO]
- Fix required: [yes/no]
```

### E5: "Text composition" terminology

**Claim:** The term "text composition" should be consistent across governance, presentation docs, and any essays. Verify the Schema Markdown spec doesn't use "conditional text" in a conflicting way.

**Action:** Search SM for "conditional text", "text composition", "text rendering", "conditional description". Verify terminology is consistent with AB.

**Documents to check:**
- **AB** §Permanent Exclusion 2 — uses "text composition" and "text rendering" as adapter-layer concerns. Defines the boundary clearly.
- **SM** — search for conflicting terminology.
- **SS** — search for conflicting terminology.

**What to look for:**
- Does SM use "conditional text" to describe something that AB calls "text composition"?
- Does any document imply that the schema supports conditional text selection, which AB explicitly excludes?
- Is there any place where "conditional visibility" (a legitimate schema feature) is confused with "conditional text" (an excluded presentation concern)?

**Recording format:**
```
E5 Finding:
- AB terminology: [term used]
- SM terminology: [consistent/conflicting] — [detail]
- SS terminology: [consistent/conflicting] — [detail]
- Severity: [CRITICAL/MINOR/INFO]
- Fix required: [yes/no] — [what to change]
```

### E6: Lambda reframe

**Claim:** Lambdas should be framed as "runtime-supervised sandboxed logic," not "schema-embedded." Verify the product vision and architecture doc use consistent framing.

**Action:** Search for "lambda" in AR, WR, product-vision.md, and future-proposals.md. Verify the framing is consistent with the deferred/excluded status.

**Documents to check:**
- **AR** — search for "lambda", "extension host".
- **WR** §Extension Host — describes lambda architecture.
- **Product Vision** (`product-vision.md`) — search for "lambda".
- **Future Proposals** (`future-proposals.md`) — check if lambdas are documented here as a deferred feature.
- **AB** — check if lambdas are mentioned in permanent exclusions or deferrals.

**What to look for:**
- Does any document describe lambdas as schema-level constructs (e.g., lambda expressions inside `.urd.md` or `.urd.json`)? If so, that conflicts with the intended framing.
- The correct framing: lambdas are a runtime extension (Wyrd's Extension Host), not a schema primitive. The schema may declare a lambda reference; the runtime executes it in a sandbox.
- Is the lambda clearly marked as post-v1 / deferred in all documents?

**Recording format:**
```
E6 Finding:
- AR framing: [consistent/inconsistent] — [detail]
- WR framing: [consistent/inconsistent] — [detail]
- Product Vision framing: [consistent/inconsistent] — [detail]
- Future Proposals: [present/absent] — [detail]
- Severity: [CRITICAL/MINOR/INFO]
- Fix required: [yes/no] — [what to change]
```

### E7: Graph model paragraph

**Claim:** Consider adding an informative section to the Schema Spec naming the graph structure explicitly.

**Action:** Evaluate whether adding a paragraph is worthwhile. This item is marked "optional, informative only."

**Documents to check:**
- **SS** — check whether the schema spec mentions the implicit graph structure (locations connected by exits, sections connected by jumps).
- **The FactSet brief** (in `briefs/done/`) — the analysis IR already names these graph structures explicitly.

**What to look for:**
- Does SS benefit from an informative paragraph explaining that the compiled world implicitly forms a location graph (ExitEdges) and a dialogue graph (JumpEdges)?
- Or is this adequately covered by the FactSet documentation and Architecture doc?

**Decision:** The auditor should recommend one of:
- **Add** — write a short informative paragraph in SS.
- **Skip** — the existing documentation (FactSet brief, Architecture) is sufficient. If skipped, the FactSet brief becomes the canonical place where graph structure is formalised. Document this decision explicitly so it's clear where to look.

**Recording format:**
```
E7 Finding:
- SS currently mentions graph structure: [yes/no]
- Recommendation: [add/skip]
- Rationale: [brief explanation]
- If skipped: FactSet brief is canonical for graph structure formalisation.
- Severity: INFO
```


## Part 2: Critical Contract Check — `on_exhausted` Structure

This check is elevated from the general consistency checks because it tests **schema ↔ compiler contract integrity** — the exact boundary that "v1 complete" depends on.

### The issue

JS defines `on_exhausted` as a `speech` object: `{ speaker?: string, text: string }`. No `goto`.

But the compiler's EMIT produces `on_exhausted` with an optional `goto` field (from `ExhaustedData.goto` in emit/mod.rs). If the compiler emits `goto` but the JSON Schema doesn't allow it, this is a **silent divergence** that violates Rule 3.

The `gate_json_schema_validates_all_fixtures` test passed at compiler 0.1.7. This means either:

1. No canonical fixture produces an `on_exhausted` with `goto`, so the field was never tested.
2. The JSON Schema does allow `goto` somewhere not immediately obvious.
3. The compiler produces `goto` but the `speech` type's `additionalProperties` is not set to `false` (it is — confirmed).

**Priority: CRITICAL.** If the compiler emits a field the schema silently rejects, the "schema validates all compiler outputs" claim is conditional, not universal.

### Resolution options

The auditor must determine which is correct by checking SS's description of `on_exhausted`:

1. **SS specifies `goto` on `on_exhausted`** → Add `goto` to JS. Either extend the `speech` definition or create a separate `exhaustedContent` definition that includes `goto`. Re-run `gate_json_schema_validates_all_fixtures`.
2. **SS does not specify `goto` on `on_exhausted`** → The compiler is emitting an undocumented field. Either:
   - a. Add `goto` to SS and JS (if it's a legitimate feature that was under-documented).
   - b. Remove `goto` from the compiler's `on_exhausted` output (if it shouldn't exist). Run full test suite.
3. **The compiler never actually emits `goto` on `on_exhausted` in practice** → The code path exists but is never triggered. Document this finding. Consider removing the dead code.

### Verification steps

1. Read SS's description of `on_exhausted`.
2. Check whether any canonical fixture has `on_exhausted` with `goto` in its compiled output.
3. Read `emit/mod.rs` `build_exhausted_data` to confirm whether `goto` is conditionally emitted.
4. Resolve per the options above.


## Part 3: Systematic Cross-Document Consistency Checks

Beyond E1–E7, the gate requires: "No contradictions between Schema Spec, JSON Schema, Schema Markdown Spec, Wyrd Spec, and Architectural Boundaries."

This is a targeted check of the most likely contradiction points.

### Check 1: Effect types

All five effect types must be listed identically across SS, JS, SM, AR, and WR.

| Effect | SS | JS | SM | AR | WR |
|--------|----|----|----|----|-----|
| `set` | ? | ? | ? | ? | ? |
| `move` | ? | ? | ? | ? | ? |
| `reveal` | ? | ? | ? | ? | ? |
| `destroy` | ? | ? | ? | ? | ? |
| `spawn` | ? | ? | ? | ? | ? |

Fill in ✓ or ✗ for each cell. Any ✗ is a contradiction to resolve.

### Check 2: Trigger types

The rule trigger vocabulary must match across SS, JS (regex pattern), and WR.

From JS: `^(phase_is \\S+|action \\S+|enter \\S+|state_change \\S+|always)$`

Five triggers: `phase_is`, `action`, `enter`, `state_change`, `always`.

Verify SS and WR list the same five triggers with consistent semantics.

### Check 3: Visibility levels

SS, JS, and AB must agree on visibility levels.

From JS: `visible`, `hidden`, `owner`, `conditional` (object form).

Verify SS and AB describe the same four levels.

### Check 4: Condition expression format

SS and JS must agree on condition expression structure (AND array vs. OR `any:` object).

From JS: `conditionExpr` is either an array of strings (AND) or an object with `any:` array (OR).

Verify SS describes the same two forms.

### Check 5: Dialogue structure

SS and JS must agree on dialogue section properties.

From JS `dialogueBlock`: `id`, `prompt`, `description`, `choices`, `conditions`, `on_exhausted`.

Verify SS lists the same fields with consistent semantics. Verify the `on_exhausted` field in JS matches SS's description (speech object, not a boolean).

### Check 6: Sequence advance modes

SS, JS, and WR must agree on valid advance modes.

From JS: `on_action`, `on_rule`, `on_condition .+`, `end`, `auto`, `manual`.

Verify SS and WR list the same modes. Note: the compiler's `VALID_ADVANCE_MODES` constant should include all six — verify this matches.

### Check 7: Trait vocabulary

SS, JS, and the compiler must agree on valid trait names.

From JS: `container`, `portable`, `mobile`, `interactable`.

Verify SS lists the same four traits. Verify the compiler recognises all four.

### Check 8: Diagnostic code consistency

DC must agree with the compiler source and the gate document.

**Documents to check:** DC, the gate document, compiler source (validate/mod.rs, link/collect.rs, etc.).

**What to check:**
- Every diagnostic code referenced in the gate document (URD202, URD301, URD302, URD410, URD411, URD430, URD432, URD433, URD434) exists in DC with the correct severity and phase.
- DC's code ranges match the compiler source file structure.
- No codes exist in the compiler source that are absent from DC (this would be a silent divergence — a compiler diagnostic that isn't documented).

This is a completeness check, not a line-by-line audit of every code. Focus on the codes the gate document references.


## Part 4: Recording and Resolution

### Audit record format

For each item (E1–E7, the critical contract check, and Checks 1–8), the auditor records:

```
[Item ID]:
  Status: [PASS / FAIL / NOT APPLICABLE]
  Severity: [CRITICAL / MINOR / INFO]
  Finding: [what was found]
  Documents checked: [list]
  Fix applied: [none / description of edit]
```

### Resolution rules

- **PASS**: No action needed. Record the finding.
- **FAIL CRITICAL**: Must be fixed before gate closure. Record what was changed. If the fix requires a JSON Schema edit, re-run `gate_json_schema_validates_all_fixtures`. If the fix requires compiler changes, run the full test suite.
- **FAIL MINOR**: Fix inline during audit. Record the change. Does not reopen the gate.
- **INFO**: Record for future reference. No action required.


## Part 5: Gate Closure

**Gate closure requires that all FAIL items are resolved and re-audited.** A fix that has not been verified does not count as resolved.

After the audit is complete and all CRITICAL items are resolved:

1. **Update `urd-v1-completion-gate.md`:**
   - Check off all three Specification Gate items.
   - Check off step 5 (Spec audit) and step 8 (Gate closure) in the Implementation Sequence.
   - Change document status from `PLANNING` to `CLOSED`.
   - Add a "Gate Closed" section at the bottom with the date, final test count, and compiler version.
   - Update the opening paragraph to reflect the closed state.

2. **Record the audit results** in the Execution Record of this brief. The audit record IS the evidence that the gate conditions were met.

3. **Update DC** if any new diagnostic codes were discovered or if any documented codes were found to be stale. Bump the "extracted from" version note to v0.1.7.


## Implementation Sequence

1. Read all normative and reference documents (or the relevant sections).
2. Perform E1–E7 audit. Record findings.
3. Perform the critical contract check (on_exhausted). Record finding.
4. Perform cross-document consistency checks (Checks 1–8). Record findings.
5. Resolve any contradictions found. Apply spec edits.
6. If any JSON Schema edits were made, re-run `gate_json_schema_validates_all_fixtures`.
7. If any compiler edits were made, run full test suite.
8. Update gate document. Close the gate.
9. Move this brief to `briefs/done/`.


## Estimated Size

| Component | Effort |
|-----------|--------|
| Document reading and cross-referencing | Primary effort — reading, not writing |
| E1–E7 audit findings | ~7 paragraphs of recorded findings |
| Critical contract check | ~1 paragraph + possible schema/compiler edit |
| Cross-document checks (8 items) | ~8 short tables or paragraphs |
| Spec edits (if needed) | 0–5 small edits across documents |
| Gate document update | ~20 lines |
| **Total new text** | **~250 lines** (mostly audit records) |

No compiler changes expected. All effort is reading and documenting.


## Acceptance Criteria

1. All seven E1–E7 items have recorded findings with severity classification.
2. The critical contract check (`on_exhausted` structure) is resolved.
3. All eight cross-document consistency checks have recorded findings.
4. No CRITICAL findings remain unresolved.
5. Any contradictions found are resolved with spec edits recorded in the Execution Record.
6. If JSON Schema was edited, `gate_json_schema_validates_all_fixtures` still passes.
7. If compiler code was edited, the full test suite passes.
8. The v1 completion gate document has all items checked off and status changed to CLOSED.
9. The gate closure date is recorded.


## What Comes After Gate Closure

The compiler gate closing means the compiler's foundational claims are provable. Two paths open:

**Path A: FactSet-derived tooling.** Derived diagnostics ("property read but never written"), visualization (location graphs, dialogue flow), LSP foundations (go-to-definition on properties). All of this is compiler-side work that does not require Wyrd.

**Path B: System gate / Wyrd.** The runtime, testing framework, and system-level acceptance criteria defined in the System Gate document. Start with a Wyrd proof-of-concept scoped to Monty Hall, then expand progressively.

Neither path is blocked by the other. The project can pursue both in parallel or prioritise based on what delivers the most value next.

---

*This is the final brief in the compiler gate sequence. When the audit passes and the gate closes, the Urd compiler is v1 complete.*

*End of Brief*
