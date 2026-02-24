/// Tests for the DefinitionIndex.
///
/// Each test compiles a fixture through the full pipeline and asserts
/// the DefinitionIndex contains expected entries with correct keys,
/// kinds, and non-synthetic spans.

use urd_compiler::compile;
use urd_compiler::definition_index::*;

fn fixture_path(name: &str) -> String {
    let base = env!("CARGO_MANIFEST_DIR");
    format!("{}/tests/fixtures/{}", base, name)
}

fn fixture_index(name: &str) -> DefinitionIndex {
    let path = fixture_path(name);
    let result = compile(&path);
    result
        .definition_index
        .expect("DefinitionIndex should be present after LINK")
}

// ── Presence tests ──

#[test]
fn definition_index_has_types() {
    let index = fixture_index("locked-garden.urd.md");
    assert!(index.get("type:Character").is_some());
    assert!(index.get("type:Item").is_some());
    assert!(index.get("type:Lock").is_some());
}

#[test]
fn definition_index_has_entities() {
    let index = fixture_index("locked-garden.urd.md");
    assert!(index.get("entity:@warden").is_some());
    assert!(index.get("entity:@ghost").is_some());
    assert!(index.get("entity:@iron_key").is_some());
    assert!(index.get("entity:@journal").is_some());
    assert!(index.get("entity:@garden_gate").is_some());
}

#[test]
fn definition_index_has_properties() {
    let index = fixture_index("locked-garden.urd.md");
    assert!(index.get("prop:Character.mood").is_some());
    assert!(index.get("prop:Character.trust").is_some());
    assert!(index.get("prop:Item.name").is_some());
    assert!(index.get("prop:Lock.locked").is_some());
}

#[test]
fn definition_index_has_sections() {
    let index = fixture_index("locked-garden.urd.md");
    assert!(index.get("section:locked-garden/greet").is_some());
    assert!(index.get("section:locked-garden/explore").is_some());
    assert!(index.get("section:locked-garden/revelation").is_some());
}

#[test]
fn definition_index_has_locations() {
    let index = fixture_index("locked-garden.urd.md");
    assert!(index.get("location:gatehouse").is_some());
    assert!(index.get("location:the-walled-garden").is_some());
}

#[test]
fn definition_index_has_exits() {
    let index = fixture_index("locked-garden.urd.md");
    assert!(index.get("exit:gatehouse/garden").is_some());
    assert!(index.get("exit:the-walled-garden/north").is_some());
}

// ── Kind metadata tests ──

#[test]
fn definition_index_entity_kind_has_type_name() {
    let index = fixture_index("locked-garden.urd.md");
    let entry = index.get("entity:@warden").unwrap();
    match &entry.kind {
        DefinitionKind::Entity { type_name } => {
            assert_eq!(type_name, "Character");
        }
        other => panic!("Expected Entity kind, got {:?}", other),
    }
}

#[test]
fn definition_index_property_kind_has_type_info() {
    let index = fixture_index("locked-garden.urd.md");
    let entry = index.get("prop:Character.trust").unwrap();
    match &entry.kind {
        DefinitionKind::Property {
            type_name,
            property_type,
            ..
        } => {
            assert_eq!(type_name, "Character");
            assert_eq!(property_type, "integer");
        }
        other => panic!("Expected Property kind, got {:?}", other),
    }
}

#[test]
fn definition_index_section_kind_has_local_name() {
    let index = fixture_index("locked-garden.urd.md");
    let entry = index.get("section:locked-garden/greet").unwrap();
    match &entry.kind {
        DefinitionKind::Section {
            local_name,
            file_stem,
        } => {
            assert_eq!(local_name, "greet");
            assert_eq!(file_stem, "locked-garden");
        }
        other => panic!("Expected Section kind, got {:?}", other),
    }
}

#[test]
fn definition_index_exit_kind_has_destination() {
    let index = fixture_index("locked-garden.urd.md");
    let entry = index.get("exit:gatehouse/garden").unwrap();
    match &entry.kind {
        DefinitionKind::Exit {
            from_location,
            destination,
        } => {
            assert_eq!(from_location, "gatehouse");
            assert_eq!(destination, "the-walled-garden");
        }
        other => panic!("Expected Exit kind, got {:?}", other),
    }
}

// ── Span validity tests ──

#[test]
fn definition_index_spans_are_valid() {
    let index = fixture_index("locked-garden.urd.md");
    for (key, entry) in index.iter() {
        assert!(
            entry.span.start_line > 0,
            "Span for '{}' has zero start_line",
            key
        );
        assert!(
            entry.span.start_col > 0,
            "Span for '{}' has zero start_col",
            key
        );
    }
}

// ── JSON serialisation tests ──

#[test]
fn definition_index_to_json_has_count() {
    let index = fixture_index("locked-garden.urd.md");
    let json = index.to_json();
    let count = json["count"].as_u64().unwrap();
    assert_eq!(count, index.len() as u64);
    assert!(count > 0, "Index should not be empty");
}

#[test]
fn definition_index_to_json_definitions_array() {
    let index = fixture_index("locked-garden.urd.md");
    let json = index.to_json();
    let defs = json["definitions"].as_array().unwrap();
    assert_eq!(defs.len(), index.len());

    // Spot-check one entry
    let warden = defs
        .iter()
        .find(|d| d["key"].as_str() == Some("entity:@warden"))
        .expect("Should contain entity:@warden");
    assert_eq!(warden["definition"]["kind"].as_str(), Some("entity"));
    assert_eq!(
        warden["definition"]["type_name"].as_str(),
        Some("Character")
    );
    assert!(warden["span"]["start_line"].as_u64().unwrap() > 0);
}

// ── All fixtures identity test ──

#[test]
fn definition_index_all_fixtures_non_empty() {
    let fixtures = [
        "locked-garden.urd.md",
        "two-room-key-puzzle.urd.md",
        "tavern-scene.urd.md",
        "monty-hall.urd.md",
        "sunken-citadel.urd.md",
    ];
    for fixture in &fixtures {
        let index = fixture_index(fixture);
        assert!(
            !index.is_empty(),
            "DefinitionIndex for '{}' should not be empty",
            fixture
        );
    }
}
