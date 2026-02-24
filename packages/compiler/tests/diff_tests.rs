/// Tests for the semantic diff engine.
///
/// Each test compiles fixture pairs through the full pipeline, builds
/// DiffSnapshots, and asserts expected change entries.

use urd_compiler::compile;
use urd_compiler::diff::*;

fn fixture_path(name: &str) -> String {
    let base = env!("CARGO_MANIFEST_DIR");
    format!("{}/tests/fixtures/{}", base, name)
}

fn compile_snapshot(fixture: &str) -> DiffSnapshot {
    let path = fixture_path(fixture);
    let result = compile(&path);
    DiffSnapshot::from_compilation(&result)
}

fn diff_fixtures(a: &str, b: &str) -> DiffReport {
    let snap_a = compile_snapshot(a);
    let snap_b = compile_snapshot(b);
    diff(&snap_a, &snap_b)
}

fn has_change(report: &DiffReport, category: &str, kind: &str, id: &str) -> bool {
    report.changes.iter().any(|c| {
        c.category == category && c.kind == kind && c.id == id
    })
}


// ── Identity tests ──

#[test]
fn diff_identity_empty() {
    let snap = compile_snapshot("locked-garden.urd.md");
    let report = diff(&snap, &snap);
    assert!(
        report.changes.is_empty(),
        "Diffing a snapshot against itself must produce no changes, got: {:?}",
        report.changes.len()
    );
}

#[test]
fn diff_deterministic() {
    let snap_a = compile_snapshot("diff/diff-a-locked-garden.urd.md");
    let snap_b = compile_snapshot("diff/diff-b-locked-garden.urd.md");
    let report1 = diff(&snap_a, &snap_b);
    let report2 = diff(&snap_a, &snap_b);
    let json1 = serde_json::to_string_pretty(&report1.to_json()).unwrap();
    let json2 = serde_json::to_string_pretty(&report2.to_json()).unwrap();
    assert_eq!(json1, json2, "Diff must be deterministic");
}

#[test]
fn diff_all_fixtures_self_identity() {
    let fixtures = [
        "locked-garden.urd.md",
        "two-room-key-puzzle.urd.md",
        "tavern-scene.urd.md",
        "monty-hall.urd.md",
        "sunken-citadel.urd.md",
    ];
    for fixture in &fixtures {
        let snap = compile_snapshot(fixture);
        let report = diff(&snap, &snap);
        assert!(
            report.changes.is_empty(),
            "Self-identity failed for '{}': {} changes",
            fixture,
            report.changes.len()
        );
    }
}

// ── Entity tests ──

#[test]
fn diff_entity_added() {
    let report = diff_fixtures(
        "diff/diff-a-locked-garden.urd.md",
        "diff/diff-b-locked-garden.urd.md",
    );
    assert!(
        has_change(&report, "entity", "added", "@lantern"),
        "Expected entity_added for @lantern"
    );
}

#[test]
fn diff_entity_removed() {
    // Reverse the pair: B→A means @lantern is removed.
    let report = diff_fixtures(
        "diff/diff-b-locked-garden.urd.md",
        "diff/diff-a-locked-garden.urd.md",
    );
    assert!(
        has_change(&report, "entity", "removed", "@lantern"),
        "Expected entity_removed for @lantern"
    );
}

#[test]
fn diff_entity_added_minimal() {
    let report = diff_fixtures(
        "diff/diff-a-minimal.urd.md",
        "diff/diff-b-minimal.urd.md",
    );
    assert!(
        has_change(&report, "entity", "added", "@guide"),
        "Expected entity_added for @guide"
    );
}

// ── Location/exit tests ──

#[test]
fn diff_location_added() {
    let report = diff_fixtures(
        "diff/diff-a-minimal.urd.md",
        "diff/diff-b-minimal.urd.md",
    );
    assert!(
        has_change(&report, "location", "added", "market"),
        "Expected location_added for market"
    );
}

#[test]
fn diff_exit_added() {
    let report = diff_fixtures(
        "diff/diff-a-minimal.urd.md",
        "diff/diff-b-minimal.urd.md",
    );
    assert!(
        has_change(&report, "exit", "added", "plaza/east"),
        "Expected exit_added for plaza/east"
    );
}

#[test]
fn diff_exit_removed() {
    let report = diff_fixtures(
        "diff/diff-a-reachability.urd.md",
        "diff/diff-b-reachability.urd.md",
    );
    assert!(
        has_change(&report, "exit", "removed", "village/east"),
        "Expected exit_removed for village/east"
    );
}

#[test]
fn diff_exit_condition_changed() {
    let report = diff_fixtures(
        "diff/diff-a-locked-garden.urd.md",
        "diff/diff-b-locked-garden.urd.md",
    );
    assert!(
        has_change(&report, "exit", "condition_changed", "gatehouse/garden"),
        "Expected exit_condition_changed for gatehouse/garden"
    );
}

// ── Dialogue tests ──

#[test]
fn diff_choice_removed() {
    let report = diff_fixtures(
        "diff/diff-a-locked-garden.urd.md",
        "diff/diff-b-locked-garden.urd.md",
    );
    // "Force the gate" choice is removed in B.
    let choice_removed = report.changes.iter().any(|c| {
        c.category == "choice" && c.kind == "removed" && c.id.contains("force")
    });
    assert!(
        choice_removed,
        "Expected a choice_removed for the Force the gate choice. Changes: {:?}",
        report.changes.iter().filter(|c| c.category == "choice").map(|c| (&c.kind, &c.id)).collect::<Vec<_>>()
    );
}

#[test]
fn diff_choice_added() {
    // Reverse: B→A, so choices in A but not B are "added" from B's perspective.
    let report = diff_fixtures(
        "diff/diff-b-locked-garden.urd.md",
        "diff/diff-a-locked-garden.urd.md",
    );
    let choice_added = report.changes.iter().any(|c| {
        c.category == "choice" && c.kind == "added" && c.id.contains("force")
    });
    assert!(
        choice_added,
        "Expected a choice_added for the Force the gate choice"
    );
}

#[test]
fn diff_section_added() {
    let report = diff_fixtures(
        "diff/diff-a-minimal.urd.md",
        "diff/diff-b-minimal.urd.md",
    );
    // B adds the "market" location with a "browse" section.
    let section_added = report.changes.iter().any(|c| {
        c.category == "section" && c.kind == "added" && c.id.contains("browse")
    });
    assert!(
        section_added,
        "Expected section_added for browse. Changes: {:?}",
        report.changes.iter().filter(|c| c.category == "section").map(|c| (&c.kind, &c.id)).collect::<Vec<_>>()
    );
}

// ── Property dependency tests ──

#[test]
fn diff_property_writer_added() {
    let report = diff_fixtures(
        "diff/diff-a-locked-garden.urd.md",
        "diff/diff-b-locked-garden.urd.md",
    );
    // B adds a write to @warden.mood = friendly in "Ask about the garden".
    let writer_added = report.changes.iter().any(|c| {
        c.category == "property_dependency"
            && c.kind == "writer_added"
            && c.id == "Character.mood"
    });
    assert!(
        writer_added,
        "Expected writer_added for Character.mood. Property changes: {:?}",
        report.changes.iter().filter(|c| c.category == "property_dependency").map(|c| (&c.kind, &c.id)).collect::<Vec<_>>()
    );
}

#[test]
fn diff_property_orphan_status_changed() {
    let report = diff_fixtures(
        "diff/diff-a-locked-garden.urd.md",
        "diff/diff-b-locked-garden.urd.md",
    );
    // In A, Character.mood is read but never written (orphaned).
    // In B, Character.mood gains a writer (> @warden.mood = friendly),
    // so orphan status changes from read_never_written to balanced (null).
    let orphan_changed = report.changes.iter().any(|c| {
        c.category == "property_dependency"
            && c.kind == "orphan_status_changed"
            && c.id == "Character.mood"
    });
    assert!(
        orphan_changed,
        "Expected orphan_status_changed for Character.mood. Property changes: {:?}",
        report.changes.iter().filter(|c| c.category == "property_dependency").map(|c| (&c.kind, &c.id)).collect::<Vec<_>>()
    );
}

#[test]
fn diff_property_writer_removed() {
    // Reverse: writer removed is B→A where A doesn't have the extra writer.
    let report = diff_fixtures(
        "diff/diff-b-locked-garden.urd.md",
        "diff/diff-a-locked-garden.urd.md",
    );
    let writer_removed = report.changes.iter().any(|c| {
        c.category == "property_dependency" && c.kind == "writer_removed"
    });
    assert!(
        writer_removed,
        "Expected at least one writer_removed. Property changes: {:?}",
        report.changes.iter().filter(|c| c.category == "property_dependency").map(|c| (&c.kind, &c.id)).collect::<Vec<_>>()
    );
}

// ── Reachability tests ──

#[test]
fn diff_reachability_became_unreachable() {
    let report = diff_fixtures(
        "diff/diff-a-reachability.urd.md",
        "diff/diff-b-reachability.urd.md",
    );
    assert!(
        has_change(&report, "reachability", "became_unreachable", "forest"),
        "Expected became_unreachable for forest. Changes: {:?}",
        report.changes.iter().filter(|c| c.category == "reachability").map(|c| (&c.kind, &c.id)).collect::<Vec<_>>()
    );
}

#[test]
fn diff_reachability_became_reachable() {
    // Reverse: forest was unreachable in B, reachable in A.
    let report = diff_fixtures(
        "diff/diff-b-reachability.urd.md",
        "diff/diff-a-reachability.urd.md",
    );
    assert!(
        has_change(&report, "reachability", "became_reachable", "forest"),
        "Expected became_reachable for forest"
    );
}

#[test]
fn diff_choice_became_impossible() {
    let report = diff_fixtures(
        "diff/diff-a-impossible-choice.urd.md",
        "diff/diff-b-impossible-choice.urd.md",
    );
    // B has URD432 for section 'enter' — impossible choice condition.
    let became_impossible = report.changes.iter().any(|c| {
        c.category == "reachability"
            && c.kind == "choice_became_impossible"
    });
    assert!(
        became_impossible,
        "Expected choice_became_impossible. Changes: {:?}",
        report.changes.iter().map(|c| (&c.category, &c.kind, &c.id)).collect::<Vec<_>>()
    );
}

#[test]
fn diff_choice_became_possible() {
    // Reverse: B→A, impossible choice resolved.
    let report = diff_fixtures(
        "diff/diff-b-impossible-choice.urd.md",
        "diff/diff-a-impossible-choice.urd.md",
    );
    let became_possible = report.changes.iter().any(|c| {
        c.category == "reachability"
            && c.kind == "choice_became_possible"
    });
    assert!(
        became_possible,
        "Expected choice_became_possible. Changes: {:?}",
        report.changes.iter().map(|c| (&c.category, &c.kind, &c.id)).collect::<Vec<_>>()
    );
}

// ── Snapshot serialisation tests ──

#[test]
fn diff_snapshot_roundtrip() {
    let snap = compile_snapshot("locked-garden.urd.md");
    let json = serde_json::to_string_pretty(&snap.to_json("the-locked-garden")).unwrap();
    let restored = DiffSnapshot::from_json(&json).expect("Roundtrip parse should succeed");
    assert_eq!(snap, restored, "Roundtrip snapshot must equal original");
}

#[test]
fn diff_snapshot_version_mismatch() {
    let json = r#"{ "urd_snapshot": "2", "entities": {} }"#;
    match DiffSnapshot::from_json(json) {
        Err(DiffError::UnsupportedSnapshotVersion) => {} // expected
        other => panic!("Expected UnsupportedSnapshotVersion, got {:?}", other),
    }
}

#[test]
fn diff_diagnostic_extractor_urd430() {
    let msg = "Location 'walled-garden' is unreachable. No path from the start location reaches it.";
    assert_eq!(
        extract_urd430_target(msg),
        Some("walled-garden".to_string()),
    );
}

#[test]
fn diff_diagnostic_extractor_urd432() {
    let msg = "Choice in section 'gatehouse/greet' (file 'locked-garden.urd.md') may never be available. Condition requires 'mood' == 'broken' but type 'Character' only allows: [wary, neutral, friendly].";
    assert_eq!(
        extract_urd432_target(msg),
        Some("gatehouse/greet".to_string()),
    );
}

// ── Fixture pair integration tests ──

#[test]
fn diff_locked_garden_pair() {
    let report = diff_fixtures(
        "diff/diff-a-locked-garden.urd.md",
        "diff/diff-b-locked-garden.urd.md",
    );
    assert!(
        !report.changes.is_empty(),
        "Locked garden pair must produce changes"
    );
    // Verify the report JSON is well-formed.
    let json = report.to_json();
    assert!(json.get("changes").unwrap().as_array().is_some());
    assert!(json.get("summary").unwrap().get("total_changes").unwrap().as_u64().unwrap() > 0);
}

#[test]
fn diff_minimal_pair() {
    let report = diff_fixtures(
        "diff/diff-a-minimal.urd.md",
        "diff/diff-b-minimal.urd.md",
    );
    assert!(
        !report.changes.is_empty(),
        "Minimal pair must produce changes"
    );
    // Should detect at least: location added, exit added, entity added, section added.
    assert!(
        report.changes.len() >= 3,
        "Expected at least 3 changes, got {}",
        report.changes.len()
    );
}

// ── DiffReport output tests ──

#[test]
fn diff_report_empty_json() {
    let snap = compile_snapshot("locked-garden.urd.md");
    let report = diff(&snap, &snap);
    let json = report.to_json();
    let changes = json.get("changes").unwrap().as_array().unwrap();
    assert!(changes.is_empty());
    let total = json.get("summary").unwrap().get("total_changes").unwrap().as_u64().unwrap();
    assert_eq!(total, 0);
}

#[test]
fn diff_report_summary_text() {
    let snap = compile_snapshot("locked-garden.urd.md");
    let report = diff(&snap, &snap);
    assert_eq!(report.summary(), "No changes detected.");

    let report2 = diff_fixtures(
        "diff/diff-a-locked-garden.urd.md",
        "diff/diff-b-locked-garden.urd.md",
    );
    let summary = report2.summary();
    assert!(summary.contains("changes:"), "Summary: {}", summary);
}
