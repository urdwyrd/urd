// End-to-end integration tests for the Urd compiler.
//
// These tests call `compile()` on real `.urd.md` fixture files and verify
// the JSON output. Unlike unit tests that build ASTs programmatically,
// these exercise the full pipeline: PARSE → IMPORT → LINK → VALIDATE → EMIT.

use urd_compiler::compile;
use urd_compiler::diagnostics::Severity;

// ── Helpers ──

fn fixture_path(name: &str) -> String {
    let base = env!("CARGO_MANIFEST_DIR");
    format!("{}/tests/fixtures/{}", base, name)
}

fn compile_fixture(name: &str) -> urd_compiler::CompilationResult {
    let path = fixture_path(name);
    compile(&path)
}

fn compile_and_parse(name: &str) -> serde_json::Value {
    let result = compile_fixture(name);
    assert!(
        result.success,
        "Expected compilation to succeed. Diagnostics:\n{}",
        format_diagnostics(&result.diagnostics)
    );
    let json_str = result.world.expect("Expected world JSON when success is true");
    serde_json::from_str(&json_str).expect("Expected valid JSON output")
}

fn format_diagnostics(diagnostics: &urd_compiler::diagnostics::DiagnosticCollector) -> String {
    diagnostics
        .sorted()
        .iter()
        .map(|d| format!("  [{:?}] {}: {}", d.severity, d.code, d.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn error_codes(diagnostics: &urd_compiler::diagnostics::DiagnosticCollector) -> Vec<String> {
    diagnostics
        .all()
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// Two Room Key Puzzle — single file, types, entities, exits, actions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_key_puzzle_compiles_successfully() {
    let result = compile_fixture("two-room-key-puzzle.urd.md");
    assert!(
        result.success,
        "Key puzzle should compile. Diagnostics:\n{}",
        format_diagnostics(&result.diagnostics)
    );
    assert!(result.world.is_some());
}

#[test]
fn e2e_key_puzzle_zero_error_diagnostics() {
    let result = compile_fixture("two-room-key-puzzle.urd.md");
    let errors = error_codes(&result.diagnostics);
    assert!(errors.is_empty(), "Expected zero errors, got: {:?}", errors);
}

#[test]
fn e2e_key_puzzle_world_block() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    let world = &json["world"];
    assert_eq!(world["name"], "Key Puzzle");
    assert_eq!(world["urd"], "1");
    assert_eq!(world["start"], "cell");
}

#[test]
fn e2e_key_puzzle_types_block() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    let types = json["types"].as_object().unwrap();
    assert_eq!(types.len(), 2);

    // Key type: portable trait, string name property
    let key = &types["Key"];
    let key_traits = key["traits"].as_array().unwrap();
    assert!(key_traits.contains(&serde_json::json!("portable")));
    let key_props = key["properties"].as_object().unwrap();
    assert_eq!(key_props["name"]["type"], "string");

    // Door type: interactable trait, bool locked property with default
    let door = &types["Door"];
    let door_traits = door["traits"].as_array().unwrap();
    assert!(door_traits.contains(&serde_json::json!("interactable")));
    let door_props = door["properties"].as_object().unwrap();
    assert_eq!(door_props["locked"]["type"], "boolean");
    assert_eq!(door_props["locked"]["default"], true);
}

#[test]
fn e2e_key_puzzle_entities_block() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    let entities = json["entities"].as_object().unwrap();
    assert_eq!(entities.len(), 2);

    let rusty_key = &entities["rusty_key"];
    assert_eq!(rusty_key["type"], "Key");
    assert_eq!(rusty_key["properties"]["name"], "Rusty Key");

    let cell_door = &entities["cell_door"];
    assert_eq!(cell_door["type"], "Door");
}

#[test]
fn e2e_key_puzzle_locations_block() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    let locations = json["locations"].as_object().unwrap();
    assert_eq!(locations.len(), 2);

    // Cell location
    let cell = &locations["cell"];
    assert_eq!(cell["description"], "A dim stone cell.");
    let contains = cell["contains"].as_array().unwrap();
    assert_eq!(contains.len(), 2);
    assert!(contains.contains(&serde_json::json!("rusty_key")));
    assert!(contains.contains(&serde_json::json!("cell_door")));

    // Cell exit to corridor
    let exits = cell["exits"].as_object().unwrap();
    let north = &exits["north"];
    assert_eq!(north["to"], "corridor");
    assert!(north.get("condition").is_some(), "Exit should have a condition");
    assert_eq!(north["blocked_message"], "The iron door is locked.");

    // Corridor location
    let corridor = &locations["corridor"];
    assert_eq!(corridor["description"], "You made it out.");
}

#[test]
fn e2e_key_puzzle_actions_block() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    let actions = json["actions"].as_object().unwrap();
    assert!(!actions.is_empty(), "Should have at least one action");

    // The use-key action targets cell_door (entity-targeted: has "target" without "target_type")
    let action = actions.values().next().unwrap();
    assert_eq!(action["target"], "cell_door");

    // Should have a containment condition (@rusty_key in here)
    let conditions = action["conditions"].as_array().unwrap();
    assert!(!conditions.is_empty());

    // Should have effects: set locked=false, destroy key
    let effects = action["effects"].as_array().unwrap();
    assert!(effects.len() >= 2, "Expected at least 2 effects (set + destroy)");

    // Verify effect types
    let effect_types: Vec<&str> = effects
        .iter()
        .filter_map(|e| {
            if e.get("set").is_some() {
                Some("set")
            } else if e.get("destroy").is_some() {
                Some("destroy")
            } else {
                None
            }
        })
        .collect();
    assert!(effect_types.contains(&"set"), "Should have a set effect");
    assert!(effect_types.contains(&"destroy"), "Should have a destroy effect");
}

#[test]
fn e2e_key_puzzle_dialogue_block() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    let dialogue = json["dialogue"].as_object().unwrap();
    assert!(!dialogue.is_empty());

    // Section IDs use file_stem/section_name format
    let section_key = "two-room-key-puzzle/actions";
    assert!(
        dialogue.contains_key(section_key),
        "Expected '{}' section. Keys: {:?}",
        section_key,
        dialogue.keys().collect::<Vec<_>>()
    );
    let section = &dialogue[section_key];
    assert_eq!(section["id"], section_key);

    let choices = section["choices"].as_array().unwrap();
    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0]["label"], "Use key");
}

#[test]
fn e2e_key_puzzle_exit_condition_lowered() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    let condition = json["locations"]["cell"]["exits"]["north"]["condition"]
        .as_str()
        .unwrap();
    // Should be a property comparison: cell_door.locked == false
    assert!(
        condition.contains("cell_door.locked"),
        "Exit condition should reference cell_door.locked, got: {}",
        condition
    );
}

#[test]
fn e2e_key_puzzle_action_effects_lowered() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    let actions = json["actions"].as_object().unwrap();
    let action = actions.values().next().unwrap();
    let effects = action["effects"].as_array().unwrap();

    // Find the set effect
    let set_effect = effects.iter().find(|e| e.get("set").is_some()).unwrap();
    assert_eq!(set_effect["set"], "cell_door.locked");
    assert_eq!(set_effect["to"], false);

    // Find the destroy effect
    let destroy_effect = effects.iter().find(|e| e.get("destroy").is_some()).unwrap();
    assert_eq!(destroy_effect["destroy"], "rusty_key");
}

// ═══════════════════════════════════════════════════════════════════════════
// Tavern Scene — entity speech, enum/integer properties, arithmetic effects
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_tavern_compiles_successfully() {
    let result = compile_fixture("tavern-scene.urd.md");
    assert!(
        result.success,
        "Tavern scene should compile. Diagnostics:\n{}",
        format_diagnostics(&result.diagnostics)
    );
}

#[test]
fn e2e_tavern_world_block() {
    let json = compile_and_parse("tavern-scene.urd.md");
    let world = &json["world"];
    assert_eq!(world["name"], "The Rusty Anchor");
    assert_eq!(world["start"], "the-rusty-anchor");
}

#[test]
fn e2e_tavern_types_block() {
    let json = compile_and_parse("tavern-scene.urd.md");
    let types = json["types"].as_object().unwrap();
    assert_eq!(types.len(), 1);

    let character = &types["Character"];
    let traits = character["traits"].as_array().unwrap();
    assert!(traits.contains(&serde_json::json!("interactable")));

    let props = character["properties"].as_object().unwrap();
    // mood: enum with values
    assert_eq!(props["mood"]["type"], "enum");
    let mood_values = props["mood"]["values"].as_array().unwrap();
    assert!(mood_values.contains(&serde_json::json!("hostile")));
    assert!(mood_values.contains(&serde_json::json!("neutral")));
    assert!(mood_values.contains(&serde_json::json!("friendly")));
    assert_eq!(props["mood"]["default"], "neutral");

    // trust: integer
    assert_eq!(props["trust"]["type"], "integer");
    assert_eq!(props["trust"]["default"], 0);
}

#[test]
fn e2e_tavern_entities_block() {
    let json = compile_and_parse("tavern-scene.urd.md");
    let entities = json["entities"].as_object().unwrap();
    assert_eq!(entities.len(), 1);

    let arina = &entities["arina"];
    assert_eq!(arina["type"], "Character");
    assert_eq!(arina["properties"]["mood"], "friendly");
}

#[test]
fn e2e_tavern_locations_block() {
    let json = compile_and_parse("tavern-scene.urd.md");
    let locations = json["locations"].as_object().unwrap();
    assert_eq!(locations.len(), 2);
    assert!(locations.contains_key("the-rusty-anchor"));
    assert!(locations.contains_key("the-harbor"));

    // Exits
    let anchor_exits = locations["the-rusty-anchor"]["exits"].as_object().unwrap();
    assert!(anchor_exits.contains_key("harbor"));
    assert_eq!(anchor_exits["harbor"]["to"], "the-harbor");

    let harbor_exits = locations["the-harbor"]["exits"].as_object().unwrap();
    assert!(harbor_exits.contains_key("south"));
    assert_eq!(harbor_exits["south"]["to"], "the-rusty-anchor");
}

#[test]
fn e2e_tavern_dialogue_section() {
    let json = compile_and_parse("tavern-scene.urd.md");
    let dialogue = json["dialogue"].as_object().unwrap();
    assert!(!dialogue.is_empty());

    // Should have section "the-rusty-anchor/topics"
    let section_key = dialogue
        .keys()
        .find(|k| k.contains("topics"))
        .expect("Should have a topics section");
    let section = &dialogue[section_key];

    // Should have a prompt (entity speech)
    assert!(
        section.get("prompt").is_some(),
        "Tavern topics should have a prompt from @arina"
    );

    // Should have choices
    let choices = section["choices"].as_array().unwrap();
    assert!(choices.len() >= 2, "Should have at least 2 choices");

    // First choice is one-shot (*), second is sticky (+)
    let ask_choice = &choices[0];
    assert_eq!(ask_choice["label"], "Ask about the ship");
    assert_eq!(ask_choice.get("sticky").and_then(|v| v.as_bool()).unwrap_or(false), false);

    let order_choice = &choices[1];
    assert_eq!(order_choice["label"], "Order a drink");
    assert_eq!(order_choice["sticky"], true);
}

#[test]
fn e2e_tavern_dialogue_has_on_exhausted() {
    let json = compile_and_parse("tavern-scene.urd.md");
    let dialogue = json["dialogue"].as_object().unwrap();
    let section_key = dialogue.keys().find(|k| k.contains("topics")).unwrap();
    let section = &dialogue[section_key];

    // The prose after the last choice becomes on_exhausted
    assert!(
        section.get("on_exhausted").is_some(),
        "Should have on_exhausted from the trailing prose"
    );
}

#[test]
fn e2e_tavern_arithmetic_effect() {
    let json = compile_and_parse("tavern-scene.urd.md");
    let dialogue = json["dialogue"].as_object().unwrap();
    let section_key = dialogue.keys().find(|k| k.contains("topics")).unwrap();
    let section = &dialogue[section_key];
    let choices = section["choices"].as_array().unwrap();

    // "Ask about the ship" has effect: @arina.trust + 1
    let ask = &choices[0];
    let effects = ask["effects"].as_array().unwrap();
    assert!(!effects.is_empty(), "Ask choice should have effects");
    let effect = &effects[0];
    assert_eq!(effect["set"], "arina.trust");
    // Arithmetic: "arina.trust + 1"
    assert!(
        effect["to"].as_str().unwrap().contains("arina.trust"),
        "Arithmetic effect should reference arina.trust"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Monty Hall — sequences, phases, rules, type-targeted choices
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_monty_hall_compiles_successfully() {
    let result = compile_fixture("monty-hall.urd.md");
    assert!(
        result.success,
        "Monty Hall should compile. Diagnostics:\n{}",
        format_diagnostics(&result.diagnostics)
    );
}

#[test]
fn e2e_monty_hall_world_block() {
    let json = compile_and_parse("monty-hall.urd.md");
    let world = &json["world"];
    assert_eq!(world["name"], "Monty Hall");
    assert_eq!(world["start"], "stage");
}

#[test]
fn e2e_monty_hall_types_and_entities() {
    let json = compile_and_parse("monty-hall.urd.md");

    // Types
    let types = json["types"].as_object().unwrap();
    assert_eq!(types.len(), 1);
    let door = &types["Door"];
    let door_props = door["properties"].as_object().unwrap();
    assert!(door_props.contains_key("prize"));
    assert!(door_props.contains_key("revealed"));
    // prize is hidden
    assert_eq!(door_props["prize"]["visibility"], "hidden");

    // Entities: 4 doors (door_1, door_2, door_3, host)
    let entities = json["entities"].as_object().unwrap();
    assert_eq!(entities.len(), 4);
    assert!(entities.contains_key("door_1"));
    assert!(entities.contains_key("door_2"));
    assert!(entities.contains_key("door_3"));
    assert!(entities.contains_key("host"));
}

#[test]
fn e2e_monty_hall_has_sequence() {
    let json = compile_and_parse("monty-hall.urd.md");
    let sequences = json.get("sequences");
    assert!(
        sequences.is_some(),
        "Monty Hall should have a sequences block. Top-level keys: {:?}",
        json.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    let sequences = sequences.unwrap().as_object().unwrap();
    assert!(!sequences.is_empty(), "Should have at least one sequence");
}

#[test]
fn e2e_monty_hall_has_rules() {
    let json = compile_and_parse("monty-hall.urd.md");
    let rules = json.get("rules");
    assert!(
        rules.is_some(),
        "Monty Hall should have a rules block. Top-level keys: {:?}",
        json.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    let rules = rules.unwrap().as_object().unwrap();
    assert!(
        rules.contains_key("monty_reveals"),
        "Should have monty_reveals rule. Keys: {:?}",
        rules.keys().collect::<Vec<_>>()
    );
}

#[test]
fn e2e_monty_hall_rule_has_actor() {
    let json = compile_and_parse("monty-hall.urd.md");
    let rule = &json["rules"]["monty_reveals"];
    assert_eq!(rule["actor"], "host", "Rule actor should be 'host' (without @)");
    assert_eq!(rule["trigger"], "action reveal");
}

#[test]
fn e2e_monty_hall_actions_with_type_target() {
    let json = compile_and_parse("monty-hall.urd.md");
    let actions = json.get("actions");
    assert!(
        actions.is_some(),
        "Monty Hall should have actions (type-targeted choices). Top-level keys: {:?}",
        json.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    let actions = actions.unwrap().as_object().unwrap();

    // At least one action should target type "Door"
    let has_door_action = actions.values().any(|a| a["target_type"] == "Door");
    assert!(
        has_door_action,
        "Should have at least one type-targeted action"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Interrogation — multi-file import, OR conditions, nested choices
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_interrogation_compiles_successfully() {
    let result = compile_fixture("interrogation/main.urd.md");
    assert!(
        result.success,
        "Interrogation should compile. Diagnostics:\n{}",
        format_diagnostics(&result.diagnostics)
    );
}

#[test]
fn e2e_interrogation_world_block() {
    let json = compile_and_parse("interrogation/main.urd.md");
    let world = &json["world"];
    assert_eq!(world["name"], "Interrogation");
    assert_eq!(world["start"], "interrogation-room");
}

#[test]
fn e2e_interrogation_types_from_import() {
    let json = compile_and_parse("interrogation/main.urd.md");
    let types = json["types"].as_object().unwrap();
    // Types come from the imported world.urd.md
    assert!(types.contains_key("Person"), "Should have Person from imported file");
    assert!(types.contains_key("Evidence"), "Should have Evidence from imported file");
}

#[test]
fn e2e_interrogation_entities_from_import() {
    let json = compile_and_parse("interrogation/main.urd.md");
    let entities = json["entities"].as_object().unwrap();
    assert_eq!(entities.len(), 3);
    assert!(entities.contains_key("suspect"));
    assert!(entities.contains_key("detective"));
    assert!(entities.contains_key("evidence"));
    assert_eq!(entities["evidence"]["properties"]["name"], "The Letter");
}

#[test]
fn e2e_interrogation_locations() {
    let json = compile_and_parse("interrogation/main.urd.md");
    let locations = json["locations"].as_object().unwrap();
    assert_eq!(locations.len(), 2);
    assert!(locations.contains_key("interrogation-room"));
    assert!(locations.contains_key("lobby"));

    // Contains entities in interrogation room
    let contains = locations["interrogation-room"]["contains"]
        .as_array()
        .unwrap();
    assert!(contains.contains(&serde_json::json!("suspect")));
    assert!(contains.contains(&serde_json::json!("detective")));
    assert!(contains.contains(&serde_json::json!("evidence")));
}

#[test]
fn e2e_interrogation_dialogue_sections() {
    let json = compile_and_parse("interrogation/main.urd.md");
    let dialogue = json["dialogue"].as_object().unwrap();

    // Should have both approach and confession sections
    let has_approach = dialogue.keys().any(|k| k.contains("approach"));
    let has_confession = dialogue.keys().any(|k| k.contains("confession"));
    assert!(has_approach, "Should have approach section. Keys: {:?}", dialogue.keys().collect::<Vec<_>>());
    assert!(has_confession, "Should have confession section. Keys: {:?}", dialogue.keys().collect::<Vec<_>>());
}

#[test]
fn e2e_interrogation_or_conditions() {
    let json = compile_and_parse("interrogation/main.urd.md");
    let dialogue = json["dialogue"].as_object().unwrap();
    let approach_key = dialogue.keys().find(|k| k.contains("approach")).unwrap();
    let approach = &dialogue[approach_key];

    // Should have section-level conditions with OR ("any")
    let conditions = approach.get("conditions");
    assert!(
        conditions.is_some(),
        "Approach section should have conditions (OR block)"
    );
}

#[test]
fn e2e_interrogation_nested_choices() {
    let json = compile_and_parse("interrogation/main.urd.md");
    let dialogue = json["dialogue"].as_object().unwrap();
    let approach_key = dialogue.keys().find(|k| k.contains("approach")).unwrap();
    let approach = &dialogue[approach_key];

    let choices = approach["choices"].as_array().unwrap();
    assert!(choices.len() >= 2, "Should have at least 2 choices");

    // "Show evidence" should have nested choices
    let show_evidence = choices
        .iter()
        .find(|c| c["label"] == "Show evidence")
        .expect("Should have 'Show evidence' choice");

    let nested = show_evidence.get("choices");
    assert!(
        nested.is_some(),
        "Show evidence should have nested choices"
    );
    let nested = nested.unwrap().as_array().unwrap();
    assert!(!nested.is_empty(), "Should have at least one nested choice");
}

#[test]
fn e2e_interrogation_goto_between_sections() {
    let json = compile_and_parse("interrogation/main.urd.md");
    let dialogue = json["dialogue"].as_object().unwrap();
    let approach_key = dialogue.keys().find(|k| k.contains("approach")).unwrap();
    let approach = &dialogue[approach_key];
    let choices = approach["choices"].as_array().unwrap();

    // "Show evidence" → "Push further" has goto -> confession
    let show_evidence = choices.iter().find(|c| c["label"] == "Show evidence").unwrap();
    let nested = show_evidence["choices"].as_array().unwrap();
    let push = &nested[0];
    assert_eq!(push["label"], "Push further");

    // Should have goto pointing to the confession section
    let goto = push.get("goto");
    assert!(
        goto.is_some(),
        "Push further should have a goto to confession"
    );
    let goto_str = goto.unwrap().as_str().unwrap();
    assert!(
        goto_str.contains("confession"),
        "Goto should reference confession section, got: {}",
        goto_str
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Cross-cutting: JSON structure
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_key_puzzle_json_has_urd_version() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    assert_eq!(json["world"]["urd"], "1", "urd field must always be \"1\"");
}

#[test]
fn e2e_key_puzzle_top_level_keys() {
    let json = compile_and_parse("two-room-key-puzzle.urd.md");
    let keys: Vec<&String> = json.as_object().unwrap().keys().collect();
    assert!(keys.contains(&&"world".to_string()));
    assert!(keys.contains(&&"types".to_string()));
    assert!(keys.contains(&&"entities".to_string()));
    assert!(keys.contains(&&"locations".to_string()));
}

#[test]
fn e2e_key_puzzle_deterministic_output() {
    // Compile the same file twice — output must be byte-identical
    let result1 = compile_fixture("two-room-key-puzzle.urd.md");
    let result2 = compile_fixture("two-room-key-puzzle.urd.md");
    assert_eq!(
        result1.world.unwrap(),
        result2.world.unwrap(),
        "Two compilations of the same file must produce identical output"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Negative: compilation failures
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn e2e_negative_unresolved_entity() {
    let result = compile_fixture("negative-unresolved-entity.urd.md");
    assert!(
        !result.success,
        "Unresolved entity should cause compilation failure"
    );
    assert!(result.world.is_none());
    assert!(result.diagnostics.has_errors());
}

#[test]
fn e2e_negative_unresolved_entity_has_link_error() {
    let result = compile_fixture("negative-unresolved-entity.urd.md");
    let errors = error_codes(&result.diagnostics);
    // URD308 = unresolved entity reference
    let has_unresolved = errors.iter().any(|c| c.starts_with("URD3"));
    assert!(
        has_unresolved,
        "Should have a LINK-phase error (URD3xx). Got: {:?}",
        errors
    );
}

#[test]
fn e2e_negative_type_mismatch() {
    let result = compile_fixture("negative-type-mismatch.urd.md");
    assert!(
        !result.success,
        "Type mismatch should cause compilation failure"
    );
    assert!(result.world.is_none());
    assert!(result.diagnostics.has_errors());
}

#[test]
fn e2e_negative_type_mismatch_has_validate_error() {
    let result = compile_fixture("negative-type-mismatch.urd.md");
    let errors = error_codes(&result.diagnostics);
    // URD4xx = VALIDATE errors
    let has_validate = errors.iter().any(|c| c.starts_with("URD4"));
    assert!(
        has_validate,
        "Should have a VALIDATE-phase error (URD4xx). Got: {:?}",
        errors
    );
}

#[test]
fn e2e_negative_missing_import() {
    let result = compile_fixture("negative-missing-import.urd.md");
    assert!(
        !result.success,
        "Missing import should cause compilation failure"
    );
    assert!(result.world.is_none());
    assert!(result.diagnostics.has_errors());
}

#[test]
fn e2e_negative_missing_import_has_import_error() {
    let result = compile_fixture("negative-missing-import.urd.md");
    let errors = error_codes(&result.diagnostics);
    // URD2xx = IMPORT errors
    let has_import = errors.iter().any(|c| c.starts_with("URD2"));
    assert!(
        has_import,
        "Should have an IMPORT-phase error (URD2xx). Got: {:?}",
        errors
    );
}
