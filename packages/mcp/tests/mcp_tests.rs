/// Assertion-based integration tests for the MCP query surface.
///
/// Compile-once pattern using `OnceLock<WorldData>` for the locked-garden
/// fixture. Each test exercises one query function and checks structural
/// properties of the returned JSON.

use std::sync::OnceLock;

use serde_json::Value;

use urd_mcp::queries;
use urd_mcp::world_data::WorldData;

// ── Helpers ──

fn fixture_path(name: &str) -> String {
    let base = env!("CARGO_MANIFEST_DIR");
    format!("{}/../compiler/tests/fixtures/{}", base, name)
}

fn locked_garden() -> &'static WorldData {
    static DATA: OnceLock<WorldData> = OnceLock::new();
    DATA.get_or_init(|| {
        let path = fixture_path("locked-garden.urd.md");
        let result = urd_compiler::compile(&path);
        WorldData::from_result(result)
    })
}

fn unreachable_fixture() -> &'static WorldData {
    static DATA: OnceLock<WorldData> = OnceLock::new();
    DATA.get_or_init(|| {
        let path = fixture_path("negative-unreachable-location.urd.md");
        let result = urd_compiler::compile(&path);
        WorldData::from_result(result)
    })
}

// ── Tool 1: get_world_metadata ──

#[test]
fn query_world_metadata() {
    let data = locked_garden();
    let result = queries::get_world_metadata(data);

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["world_name"], "the-locked-garden");
    assert_eq!(result["start_location"], "gatehouse");
    assert_eq!(result["entity_count"], 5); // warden, ghost, iron_key, journal, garden_gate
    assert_eq!(result["location_count"], 2); // gatehouse, the-walled-garden
    assert_eq!(result["type_count"], 3); // Character, Item, Lock
    assert_eq!(result["has_errors"], false);
}

// ── Tool 2: get_exit_graph ──

#[test]
fn query_exit_graph_nodes() {
    let data = locked_garden();
    let result = queries::get_exit_graph(data);

    assert_eq!(result["schema_version"], "1");
    let nodes = result["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 2);
    // Sorted alphabetically
    assert!(nodes.iter().any(|n| n == "gatehouse"));
    assert!(nodes.iter().any(|n| n == "the-walled-garden"));
}

#[test]
fn query_exit_graph_edges() {
    let data = locked_garden();
    let result = queries::get_exit_graph(data);

    let edges = result["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 2);

    // One conditional exit (gatehouse → garden)
    let conditional_count = edges
        .iter()
        .filter(|e| e["is_conditional"] == true)
        .count();
    assert!(conditional_count >= 1, "Expected at least one conditional exit");

    // Check that gatehouse→the-walled-garden exists
    let garden_exit = edges
        .iter()
        .find(|e| e["from"] == "gatehouse" && e["to"] == "the-walled-garden");
    assert!(garden_exit.is_some(), "Expected gatehouse→the-walled-garden exit");
}

// ── Tool 3: get_dialogue_graph ──

#[test]
fn query_dialogue_graph_sections() {
    let data = locked_garden();
    let result = queries::get_dialogue_graph(data);

    assert_eq!(result["schema_version"], "1");
    let sections = result["sections"].as_array().unwrap();
    // gatehouse/greet, the-walled-garden/explore, the-walled-garden/revelation
    assert_eq!(sections.len(), 3);
}

#[test]
fn query_dialogue_graph_choices() {
    let data = locked_garden();
    let result = queries::get_dialogue_graph(data);

    let choices = result["choices"].as_array().unwrap();
    assert!(!choices.is_empty(), "Expected choices in dialogue graph");

    // Check sticky flag: "State your purpose" is sticky (+), others are not (*)
    let sticky_count = choices
        .iter()
        .filter(|c| c["sticky"] == true)
        .count();
    assert!(sticky_count >= 1, "Expected at least one sticky choice");
}

// ── Tool 4: get_entity_details ──

#[test]
fn query_entity_details_found() {
    let data = locked_garden();
    let result = queries::get_entity_details(data, "@warden");

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["entity_id"], "@warden");
    assert_eq!(result["type"], "Character");
    assert_eq!(result["container"], "gatehouse");

    let props = result["properties"].as_array().unwrap();
    let prop_names: Vec<&str> = props
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert!(prop_names.contains(&"trust"), "Expected trust property");
    assert!(prop_names.contains(&"mood"), "Expected mood property");
}

#[test]
fn query_entity_details_not_found() {
    let data = locked_garden();
    let result = queries::get_entity_details(data, "@nonexistent");

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["error"], "Entity not found");
}

// ── Tool 5: get_property_dependencies ──

#[test]
fn query_property_deps_trust() {
    let data = locked_garden();
    let result = queries::get_property_dependencies(data, "Character", "trust");

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["property_key"], "Character.trust");

    let read_count = result["read_count"].as_u64().unwrap();
    let write_count = result["write_count"].as_u64().unwrap();
    assert!(read_count > 0, "Expected reads of Character.trust");
    assert!(write_count > 0, "Expected writes of Character.trust");
    assert_eq!(result["read_only"], false);
    assert_eq!(result["write_only"], false);
}

#[test]
fn query_property_deps_not_found() {
    let data = locked_garden();
    let result = queries::get_property_dependencies(data, "Character", "nonexistent");

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["error"], "Property not found");
}

// ── Tool 6: get_reachable_locations ──

#[test]
fn query_reachable_from_gatehouse() {
    let data = locked_garden();
    let result = queries::get_reachable_locations(data, "gatehouse");

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["from"], "gatehouse");

    let reachable = result["reachable"].as_array().unwrap();
    assert!(
        reachable.iter().any(|l| l == "gatehouse"),
        "Start location should be reachable"
    );
    assert!(
        reachable.iter().any(|l| l == "the-walled-garden"),
        "The Walled Garden should be reachable from gatehouse"
    );
}

#[test]
fn query_reachable_unreachable() {
    // negative-unreachable-location has Room A (start) and Room B (no incoming exits)
    let data = unreachable_fixture();
    let result = queries::get_reachable_locations(data, "room-a");

    assert_eq!(result["schema_version"], "1");

    let unreachable = result["unreachable"].as_array().unwrap();
    assert!(
        unreachable.iter().any(|l| l == "room-b"),
        "Room B should be unreachable from Room A"
    );
}

#[test]
fn query_reachable_not_found() {
    let data = locked_garden();
    let result = queries::get_reachable_locations(data, "nonexistent");

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["error"], "Location not found");
}

// ── Tool 7: get_choice_conditions ──

#[test]
fn query_choice_conditions_greet() {
    let data = locked_garden();
    let result = queries::get_choice_conditions(data, "locked-garden/greet");

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["section"], "locked-garden/greet");

    let choices = result["choices"].as_array().unwrap();
    assert!(!choices.is_empty(), "Expected choices in gatehouse/greet");

    // At least some choices have conditions or effects
    let has_conditions = choices.iter().any(|c| {
        c["conditions"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false)
    });
    let has_effects = choices.iter().any(|c| {
        c["effects"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false)
    });
    assert!(has_conditions, "Expected at least one choice with conditions");
    assert!(has_effects, "Expected at least one choice with effects");
}

#[test]
fn query_choice_conditions_not_found() {
    let data = locked_garden();
    let result = queries::get_choice_conditions(data, "nonexistent/section");

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["error"], "Section not found");
}

// ── Tool 8: get_diagnostics ──

#[test]
fn query_diagnostics_all() {
    let data = locked_garden();
    let result = queries::get_diagnostics(data, None, None);

    assert_eq!(result["schema_version"], "1");
    assert_eq!(result["errors"], 0, "Locked garden should have 0 errors");
}

#[test]
fn query_diagnostics_filter_severity() {
    let data = locked_garden();
    let result = queries::get_diagnostics(data, Some("warning"), None);

    assert_eq!(result["schema_version"], "1");
    // All returned diagnostics should be warnings
    if let Some(diags) = result["diagnostics"].as_array() {
        for d in diags {
            assert_eq!(d["severity"], "warning");
        }
    }
}

#[test]
fn query_diagnostics_filter_file() {
    let data = locked_garden();
    let result =
        queries::get_diagnostics(data, None, Some("locked-garden.urd.md"));

    assert_eq!(result["schema_version"], "1");
    // All returned diagnostics should reference the file
    if let Some(diags) = result["diagnostics"].as_array() {
        for d in diags {
            assert_eq!(d["file"], "locked-garden.urd.md");
        }
    }
}

// ── Cross-cutting: schema_version present ──

#[test]
fn query_schema_version_present() {
    let data = locked_garden();

    // All 8 tool responses must include schema_version
    let results: Vec<Value> = vec![
        queries::get_world_metadata(data),
        queries::get_exit_graph(data),
        queries::get_dialogue_graph(data),
        queries::get_entity_details(data, "@warden"),
        queries::get_property_dependencies(data, "Character", "trust"),
        queries::get_reachable_locations(data, "gatehouse"),
        queries::get_choice_conditions(data, "locked-garden/greet"),
        queries::get_diagnostics(data, None, None),
    ];

    for (i, result) in results.iter().enumerate() {
        assert_eq!(
            result["schema_version"], "1",
            "Tool {} missing schema_version",
            i
        );
    }
}

// ── Import boundary ──

#[test]
fn mcp_import_boundary() {
    // The MCP crate should not import compiler internals.
    // Only urd_compiler::{compile, CompilationResult}, urd_compiler::facts::*,
    // and urd_compiler::diagnostics::Severity are allowed.
    let src_dir = format!("{}/../mcp/src", env!("CARGO_MANIFEST_DIR"));
    let prohibited = [
        "urd_compiler::ast",
        "urd_compiler::parse",
        "urd_compiler::link",
        "urd_compiler::validate",
        "urd_compiler::emit",
        "urd_compiler::symbol_table",
        "urd_compiler::graph",
    ];

    let mut violations = Vec::new();

    for entry in glob::glob(&format!("{}/**/*.rs", src_dir)).unwrap() {
        let path = entry.unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        for module in &prohibited {
            if content.contains(module) {
                violations.push(format!(
                    "{}: imports prohibited module {}",
                    path.display(),
                    module
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Import boundary violations:\n{}",
        violations.join("\n")
    );
}
