/// Tests for the ANALYZE phase (FactSet-derived diagnostics, URD600–URD699).
///
/// Each test compiles a fixture, extracts the FactSet, runs `analyze()`,
/// and asserts on the diagnostics produced.

use urd_compiler::analyze;
use urd_compiler::compile;
use urd_compiler::diagnostics::Diagnostic;
use urd_compiler::facts::{FactSet, PropertyDependencyIndex};

fn fixture_path(name: &str) -> String {
    let base = env!("CARGO_MANIFEST_DIR");
    format!("{}/tests/fixtures/{}", base, name)
}

fn extract_fixture_facts(name: &str) -> FactSet {
    let path = fixture_path(name);
    let result = compile(&path);
    assert!(
        result.fact_set.is_some(),
        "Expected FactSet after LINK for fixture '{}'",
        name,
    );
    result.fact_set.unwrap()
}

fn analyze_fixture(name: &str) -> Vec<Diagnostic> {
    let facts = extract_fixture_facts(name);
    let index = PropertyDependencyIndex::build(&facts);
    analyze::analyze(&facts, &index)
}

fn diagnostics_with_code<'a>(diagnostics: &'a [Diagnostic], code: &str) -> Vec<&'a Diagnostic> {
    diagnostics.iter().filter(|d| d.code == code).collect()
}

// ── Clean fixture: no diagnostics ──

#[test]
fn analyze_clean_world_no_diagnostics() {
    let diags = analyze_fixture("negative-factset-diagnostics.urd.md");
    assert!(
        diags.is_empty(),
        "Expected zero analyze diagnostics on clean fixture, got: {:#?}",
        diags
    );
}

// ── D1: Property read but never written — URD601 ──

#[test]
fn analyze_d1_read_never_written() {
    let diags = analyze_fixture("positive-factset-diagnostics.urd.md");
    let d1 = diagnostics_with_code(&diags, "URD601");
    assert!(
        !d1.is_empty(),
        "Expected URD601 for NPC.suspicion (read but never written)"
    );
    assert!(
        d1.iter().any(|d| d.message.contains("suspicion")),
        "URD601 should mention 'suspicion', got: {:?}",
        d1.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── D2: Property written but never read — URD602 ──

#[test]
fn analyze_d2_written_never_read() {
    let diags = analyze_fixture("positive-factset-diagnostics.urd.md");
    let d2 = diagnostics_with_code(&diags, "URD602");
    assert!(
        !d2.is_empty(),
        "Expected URD602 for NPC.loyalty (written but never read)"
    );
    assert!(
        d2.iter().any(|d| d.message.contains("loyalty")),
        "URD602 should mention 'loyalty', got: {:?}",
        d2.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── D3: Enum variant untested — URD603 ──

#[test]
fn analyze_d3_enum_variant_untested() {
    let diags = analyze_fixture("positive-factset-diagnostics.urd.md");
    let d3 = diagnostics_with_code(&diags, "URD603");
    assert!(
        !d3.is_empty(),
        "Expected URD603 for NPC.mood -> friendly (variant untested)"
    );
    assert!(
        d3.iter().any(|d| d.message.contains("friendly")),
        "URD603 should mention 'friendly', got: {:?}",
        d3.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── D4: Unreachable threshold — URD604 ──

#[test]
fn analyze_d4_unreachable_threshold() {
    let diags = analyze_fixture("positive-factset-diagnostics.urd.md");
    let d4 = diagnostics_with_code(&diags, "URD604");
    assert!(
        !d4.is_empty(),
        "Expected URD604 for NPC.power (threshold 100, only Set to 5)"
    );
    assert!(
        d4.iter().any(|d| d.message.contains("power")),
        "URD604 should mention 'power', got: {:?}",
        d4.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── D5: Circular dependency — URD605 ──

#[test]
fn analyze_d5_circular_dependency() {
    let diags = analyze_fixture("positive-factset-diagnostics.urd.md");
    let d5 = diagnostics_with_code(&diags, "URD605");
    assert!(
        !d5.is_empty(),
        "Expected URD605 for NPC.rank (self-guarded write, no bootstrap)"
    );
    assert!(
        d5.iter().any(|d| d.message.contains("rank")),
        "URD605 should mention 'rank', got: {:?}",
        d5.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

#[test]
fn analyze_d5_circular_multi_write() {
    let diags = analyze_fixture("positive-factset-circular-deep.urd.md");
    let d5 = diagnostics_with_code(&diags, "URD605");
    assert!(
        !d5.is_empty(),
        "Expected URD605 for NPC.clearance (both writes self-guarded)"
    );
    assert!(
        d5.iter().any(|d| d.message.contains("clearance")),
        "URD605 should mention 'clearance', got: {:?}",
        d5.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}

// ── Existing fixtures: no panics ──

#[test]
fn analyze_existing_locked_garden() {
    let diags = analyze_fixture("locked-garden.urd.md");
    // No assertion on content — just verify no panic and record results.
    eprintln!(
        "locked-garden.urd.md: {} analyze diagnostics: {:?}",
        diags.len(),
        diags.iter().map(|d| format!("{}: {}", d.code, d.message)).collect::<Vec<_>>()
    );
}

#[test]
fn analyze_existing_sunken_citadel() {
    let diags = analyze_fixture("sunken-citadel.urd.md");
    eprintln!(
        "sunken-citadel.urd.md: {} analyze diagnostics: {:?}",
        diags.len(),
        diags.iter().map(|d| format!("{}: {}", d.code, d.message)).collect::<Vec<_>>()
    );
}

#[test]
fn analyze_existing_tavern_scene() {
    let diags = analyze_fixture("tavern-scene.urd.md");
    eprintln!(
        "tavern-scene.urd.md: {} analyze diagnostics: {:?}",
        diags.len(),
        diags.iter().map(|d| format!("{}: {}", d.code, d.message)).collect::<Vec<_>>()
    );
}

#[test]
fn analyze_existing_key_puzzle() {
    let diags = analyze_fixture("two-room-key-puzzle.urd.md");
    eprintln!(
        "two-room-key-puzzle.urd.md: {} analyze diagnostics: {:?}",
        diags.len(),
        diags.iter().map(|d| format!("{}: {}", d.code, d.message)).collect::<Vec<_>>()
    );
}

#[test]
fn analyze_existing_monty_hall() {
    let diags = analyze_fixture("monty-hall.urd.md");
    eprintln!(
        "monty-hall.urd.md: {} analyze diagnostics: {:?}",
        diags.len(),
        diags.iter().map(|d| format!("{}: {}", d.code, d.message)).collect::<Vec<_>>()
    );
}

#[test]
fn analyze_existing_interrogation() {
    let path = fixture_path("interrogation/main.urd.md");
    let result = compile(&path);
    assert!(result.fact_set.is_some(), "Expected FactSet for interrogation");
    let fs = result.fact_set.as_ref().unwrap();
    let idx = PropertyDependencyIndex::build(fs);
    let diags = analyze::analyze(fs, &idx);
    eprintln!(
        "interrogation: {} analyze diagnostics: {:?}",
        diags.len(),
        diags.iter().map(|d| format!("{}: {}", d.code, d.message)).collect::<Vec<_>>()
    );
}

// ── Empty world: no panic ──

#[test]
fn analyze_empty_world_no_panic() {
    // A minimal world with no conditions or effects produces an empty FactSet.
    let source = r#"---
world:
  name: empty
  start: room
---

# Room

== talk

@narrator: Hello.
"#;
    let result = urd_compiler::compile_source("empty.urd.md", source);
    assert!(result.fact_set.is_some(), "Expected FactSet for empty world");
    let fs = result.fact_set.as_ref().unwrap();
    let idx = PropertyDependencyIndex::build(fs);
    let diags = analyze::analyze(fs, &idx);
    assert!(
        diags.is_empty(),
        "Expected zero analyze diagnostics on empty world, got: {:#?}",
        diags
    );
}

// ── Architectural constraint: no AST imports ──

#[test]
fn analyze_no_ast_imports() {
    let source = std::fs::read_to_string(
        format!("{}/src/analyze.rs", env!("CARGO_MANIFEST_DIR"))
    ).expect("Could not read analyze.rs");

    let forbidden = [
        "use crate::ast",
        "use crate::graph",
        "use crate::symbol_table",
        "use crate::parse",
    ];

    for pattern in &forbidden {
        assert!(
            !source.contains(pattern),
            "analyze.rs must not contain '{}' — FactSet diagnostics must not depend on the AST",
            pattern
        );
    }
}
