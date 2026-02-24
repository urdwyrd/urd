/// Tests for the FactSet analysis IR extraction.
///
/// Each test compiles a canonical fixture through the full pipeline and
/// inspects the `fact_set` field on `CompilationResult`.

use urd_compiler::compile;
use urd_compiler::facts::*;

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

// ── Two-room key puzzle ──

#[test]
fn facts_key_puzzle_exit_count() {
    let facts = extract_fixture_facts("two-room-key-puzzle.urd.md");
    // Cell -> north: Corridor (one resolved exit). Corridor has no exits.
    assert_eq!(facts.exits().len(), 1, "exits: {:?}", facts.exits());
}

#[test]
fn facts_key_puzzle_conditional_exit() {
    let facts = extract_fixture_facts("two-room-key-puzzle.urd.md");
    let exit = &facts.exits()[0];
    assert_eq!(exit.from_location, "cell");
    assert_eq!(exit.to_location, "corridor");
    assert_eq!(exit.exit_name, "north");
    assert!(exit.is_conditional, "Exit should be conditional");
    assert_eq!(
        exit.guard_reads.len(),
        1,
        "One guard read: @cell_door.locked == false"
    );

    // Verify the guard read is a PropertyRead for Door.locked.
    let read_idx = exit.guard_reads[0];
    let read = &facts.reads()[read_idx];
    assert_eq!(read.entity_type, "Door");
    assert_eq!(read.property, "locked");
    assert_eq!(read.operator, CompareOp::Eq);
    assert_eq!(read.value_literal, "false");
    assert_eq!(read.value_kind, LiteralKind::Bool);
    assert_eq!(read.site, FactSite::Exit(exit.exit_id()));
}

#[test]
fn facts_key_puzzle_effect_writes() {
    let facts = extract_fixture_facts("two-room-key-puzzle.urd.md");
    // Only @cell_door.locked = false is a Set effect.
    // destroy @rusty_key is a Destroy, not tracked.
    assert_eq!(facts.writes().len(), 1, "writes: {:?}", facts.writes());
    let write = &facts.writes()[0];
    assert_eq!(write.entity_type, "Door");
    assert_eq!(write.property, "locked");
    assert_eq!(write.operator, WriteOp::Set);
    assert_eq!(write.value_expr, "false");
}

#[test]
fn facts_key_puzzle_choice_count() {
    let facts = extract_fixture_facts("two-room-key-puzzle.urd.md");
    // One choice: "Use key" in section "actions".
    assert_eq!(facts.choices().len(), 1, "choices: {:?}", facts.choices());
    let choice = &facts.choices()[0];
    assert_eq!(choice.label, "Use key");
    assert!(!choice.sticky);
    assert!(choice.section.ends_with("/actions"));
}

#[test]
fn facts_key_puzzle_reads_count() {
    let facts = extract_fixture_facts("two-room-key-puzzle.urd.md");
    // 1 PropertyRead: the exit guard condition.
    // The choice condition (@rusty_key in here) is a ContainmentCheck, not tracked.
    assert_eq!(facts.reads().len(), 1, "reads: {:?}", facts.reads());
}

// ── Tavern scene ──

#[test]
fn facts_tavern_exit_count() {
    let facts = extract_fixture_facts("tavern-scene.urd.md");
    // Two exits: Rusty Anchor -> Harbor, Harbor -> Rusty Anchor.
    assert_eq!(facts.exits().len(), 2, "exits: {:?}", facts.exits());
    // Both unconditional.
    for exit in facts.exits() {
        assert!(
            !exit.is_conditional,
            "Exit {} should be unconditional",
            exit.exit_name
        );
        assert!(exit.guard_reads.is_empty());
    }
}

#[test]
fn facts_tavern_choice_count() {
    let facts = extract_fixture_facts("tavern-scene.urd.md");
    // Two choices: "Ask about the ship" (one-shot), "Order a drink" (sticky).
    assert_eq!(facts.choices().len(), 2, "choices: {:?}", facts.choices());
    let sticky_count = facts.choices().iter().filter(|c| c.sticky).count();
    let oneshot_count = facts.choices().iter().filter(|c| !c.sticky).count();
    assert_eq!(sticky_count, 1);
    assert_eq!(oneshot_count, 1);
}

#[test]
fn facts_tavern_property_writes() {
    let facts = extract_fixture_facts("tavern-scene.urd.md");
    // One write: @arina.trust + 1.
    assert_eq!(facts.writes().len(), 1, "writes: {:?}", facts.writes());
    let write = &facts.writes()[0];
    assert_eq!(write.entity_type, "Character");
    assert_eq!(write.property, "trust");
    assert_eq!(write.operator, WriteOp::Add);
    assert_eq!(write.value_expr, "1");
}

#[test]
fn facts_tavern_no_reads() {
    let facts = extract_fixture_facts("tavern-scene.urd.md");
    // No PropertyComparison conditions on choices or exits.
    assert_eq!(facts.reads().len(), 0, "reads: {:?}", facts.reads());
}

// ── Monty Hall ──

#[test]
fn facts_monty_hall_no_exits() {
    let facts = extract_fixture_facts("monty-hall.urd.md");
    // Single-location game, no exit declarations with resolved destinations.
    assert_eq!(facts.exits().len(), 0, "exits: {:?}", facts.exits());
}

#[test]
fn facts_monty_hall_rule_count() {
    let facts = extract_fixture_facts("monty-hall.urd.md");
    // One rule: monty_reveals.
    assert_eq!(facts.rules().len(), 1, "rules: {:?}", facts.rules());
    assert_eq!(facts.rules()[0].rule_id, "monty_reveals");
}

#[test]
fn facts_monty_hall_choices() {
    let facts = extract_fixture_facts("monty-hall.urd.md");
    // Two choices in section "switch": "Switch doors" and "Stay with your choice".
    // "Pick a door" is under a phase heading, not a section — no ChoiceFact.
    assert_eq!(facts.choices().len(), 2, "choices: {:?}", facts.choices());
}

#[test]
fn facts_monty_hall_choice_condition() {
    let facts = extract_fixture_facts("monty-hall.urd.md");
    // One PropertyRead from choice condition: @door_1.revealed == false.
    let choice_reads: Vec<_> = facts
        .reads()
        .iter()
        .filter(|r| matches!(&r.site, FactSite::Choice(_)))
        .collect();
    assert_eq!(
        choice_reads.len(),
        1,
        "choice reads: {:?}",
        choice_reads
    );
    assert_eq!(choice_reads[0].entity_type, "Door");
    assert_eq!(choice_reads[0].property, "revealed");
    assert_eq!(choice_reads[0].operator, CompareOp::Eq);
    assert_eq!(choice_reads[0].value_literal, "false");
}

// ── Interrogation ──

#[test]
fn facts_interrogation_exit_count() {
    let facts = extract_fixture_facts("interrogation/main.urd.md");
    // Two exits: Interrogation Room -> Lobby, Lobby -> Interrogation Room.
    assert_eq!(facts.exits().len(), 2, "exits: {:?}", facts.exits());
}

#[test]
fn facts_interrogation_choice_count() {
    let facts = extract_fixture_facts("interrogation/main.urd.md");
    // Three choices: "Press harder", "Show evidence", "Push further" (nested).
    assert_eq!(facts.choices().len(), 3, "choices: {:?}", facts.choices());
}

#[test]
fn facts_interrogation_property_reads() {
    let facts = extract_fixture_facts("interrogation/main.urd.md");
    // PropertyReads from choice conditions:
    // 1. @suspect.mood != hostile (Press harder)
    // 2. @suspect.trust >= 2 (Push further, nested)
    // Section-level OrConditionBlock is not tracked (no FactSite owner).
    // @evidence in player is ContainmentCheck, not tracked.
    let choice_reads: Vec<_> = facts
        .reads()
        .iter()
        .filter(|r| matches!(&r.site, FactSite::Choice(_)))
        .collect();
    assert_eq!(
        choice_reads.len(),
        2,
        "choice reads: {:?}",
        choice_reads
    );
}

#[test]
fn facts_interrogation_property_writes() {
    let facts = extract_fixture_facts("interrogation/main.urd.md");
    // Two writes: @suspect.trust - 1 (Press harder), @suspect.trust + 2 (Show evidence).
    assert_eq!(facts.writes().len(), 2, "writes: {:?}", facts.writes());
}

#[test]
fn facts_interrogation_jumps() {
    let facts = extract_fixture_facts("interrogation/main.urd.md");
    // One jump: -> confession (resolves to section main/confession).
    assert_eq!(facts.jumps().len(), 1, "jumps: {:?}", facts.jumps());
    let jump = &facts.jumps()[0];
    assert!(matches!(&jump.target, JumpTarget::Section(s) if s.ends_with("/confession")));
}

#[test]
fn facts_interrogation_cross_file_spans() {
    let facts = extract_fixture_facts("interrogation/main.urd.md");
    // All facts should have spans referencing main.urd.md (not world.urd.md).
    // world.urd.md only has types/entities in frontmatter — no content nodes.
    for read in facts.reads() {
        assert!(
            read.span.file.contains("main.urd.md"),
            "Read span should reference main.urd.md, got: {}",
            read.span.file
        );
    }
    for write in facts.writes() {
        assert!(
            write.span.file.contains("main.urd.md"),
            "Write span should reference main.urd.md, got: {}",
            write.span.file
        );
    }
}

// ── Sunken Citadel ──

#[test]
fn facts_citadel_rule_count() {
    let facts = extract_fixture_facts("sunken-citadel.urd.md");
    assert_eq!(facts.rules().len(), 3, "rules: {:?}", facts.rules());
    let rule_ids: Vec<_> = facts.rules().iter().map(|r| r.rule_id.as_str()).collect();
    assert!(rule_ids.contains(&"spirit_manifests"));
    assert!(rule_ids.contains(&"guard_patrols"));
    assert!(rule_ids.contains(&"merchant_restocks"));
}

#[test]
fn facts_citadel_scale_smoke() {
    let facts = extract_fixture_facts("sunken-citadel.urd.md");
    // Bounded range assertions for the large fixture.
    let exit_count = facts.exits().len();
    let choice_count = facts.choices().len();
    let read_count = facts.reads().len();
    let write_count = facts.writes().len();

    assert!(
        exit_count >= 10 && exit_count <= 40,
        "Expected 10-40 exits, got {}",
        exit_count
    );
    assert!(
        choice_count >= 20 && choice_count <= 100,
        "Expected 20-100 choices, got {}",
        choice_count
    );
    assert!(
        read_count >= 5,
        "Expected at least 5 reads, got {}",
        read_count
    );
    assert!(
        write_count >= 5,
        "Expected at least 5 writes, got {}",
        write_count
    );
}

#[test]
fn facts_citadel_exit_density() {
    let facts = extract_fixture_facts("sunken-citadel.urd.md");
    // Mix of conditional and unconditional exits.
    let conditional = facts.exits().iter().filter(|e| e.is_conditional).count();
    let unconditional = facts.exits().iter().filter(|e| !e.is_conditional).count();
    assert!(conditional > 0, "Should have conditional exits");
    assert!(unconditional > 0, "Should have unconditional exits");

    // Conditional exits should have guard_reads populated.
    for exit in facts.exits().iter().filter(|e| e.is_conditional) {
        assert!(
            !exit.guard_reads.is_empty(),
            "Conditional exit {} should have guard_reads",
            exit.exit_name
        );
    }
}

// ── PropertyDependencyIndex ──

#[test]
fn facts_index_writers_key_puzzle() {
    let facts = extract_fixture_facts("two-room-key-puzzle.urd.md");
    let index = PropertyDependencyIndex::build(&facts);

    let door_locked = PropertyKey {
        entity_type: "Door".to_string(),
        property: "locked".to_string(),
    };

    let write_indices = index.writes_of(&door_locked);
    assert_eq!(write_indices.len(), 1, "One write to Door.locked");
    assert_eq!(facts.writes()[write_indices[0]].operator, WriteOp::Set);
}

#[test]
fn facts_index_readers_key_puzzle() {
    let facts = extract_fixture_facts("two-room-key-puzzle.urd.md");
    let index = PropertyDependencyIndex::build(&facts);

    let door_locked = PropertyKey {
        entity_type: "Door".to_string(),
        property: "locked".to_string(),
    };

    let read_indices = index.reads_of(&door_locked);
    assert_eq!(read_indices.len(), 1, "One read of Door.locked");
    assert_eq!(facts.reads()[read_indices[0]].operator, CompareOp::Eq);
}

// ── Referential integrity ──

#[test]
fn facts_roundtrip_consistency() {
    let facts = extract_fixture_facts("sunken-citadel.urd.md");

    // Every index in ChoiceFact.condition_reads points to a valid read
    // with matching FactSite.
    for choice in facts.choices() {
        for &idx in &choice.condition_reads {
            assert!(
                idx < facts.reads().len(),
                "condition_reads index {} out of bounds (reads.len() = {})",
                idx,
                facts.reads().len()
            );
            let read = &facts.reads()[idx];
            assert_eq!(
                read.site,
                FactSite::Choice(choice.choice_id.clone()),
                "Read at index {} has wrong FactSite",
                idx
            );
        }
        for &idx in &choice.effect_writes {
            assert!(
                idx < facts.writes().len(),
                "effect_writes index {} out of bounds (writes.len() = {})",
                idx,
                facts.writes().len()
            );
            let write = &facts.writes()[idx];
            assert_eq!(
                write.site,
                FactSite::Choice(choice.choice_id.clone()),
                "Write at index {} has wrong FactSite",
                idx
            );
        }
    }

    // Every index in ExitEdge.guard_reads points to a valid read.
    for exit in facts.exits() {
        for &idx in &exit.guard_reads {
            assert!(
                idx < facts.reads().len(),
                "guard_reads index {} out of bounds",
                idx
            );
            let read = &facts.reads()[idx];
            assert_eq!(
                read.site,
                FactSite::Exit(exit.exit_id()),
                "Guard read at index {} has wrong FactSite",
                idx
            );
        }
    }

    // Every index in RuleFact.condition_reads and effect_writes is valid.
    for rule in facts.rules() {
        for &idx in &rule.condition_reads {
            assert!(
                idx < facts.reads().len(),
                "rule condition_reads index {} out of bounds",
                idx
            );
        }
        for &idx in &rule.effect_writes {
            assert!(
                idx < facts.writes().len(),
                "rule effect_writes index {} out of bounds",
                idx
            );
        }
    }
}

// ── Determinism ──

#[test]
fn facts_determinism() {
    let path = fixture_path("sunken-citadel.urd.md");
    let result1 = compile(&path);
    let result2 = compile(&path);

    let facts1 = result1.fact_set.expect("FactSet 1");
    let facts2 = result2.fact_set.expect("FactSet 2");

    assert_eq!(facts1.reads().len(), facts2.reads().len(), "reads count mismatch");
    assert_eq!(facts1.writes().len(), facts2.writes().len(), "writes count mismatch");
    assert_eq!(facts1.exits().len(), facts2.exits().len(), "exits count mismatch");
    assert_eq!(facts1.jumps().len(), facts2.jumps().len(), "jumps count mismatch");
    assert_eq!(facts1.choices().len(), facts2.choices().len(), "choices count mismatch");
    assert_eq!(facts1.rules().len(), facts2.rules().len(), "rules count mismatch");

    // Compare reads field by field.
    for (i, (r1, r2)) in facts1.reads().iter().zip(facts2.reads().iter()).enumerate() {
        assert_eq!(r1.entity_type, r2.entity_type, "read {} entity_type", i);
        assert_eq!(r1.property, r2.property, "read {} property", i);
        assert_eq!(r1.operator, r2.operator, "read {} operator", i);
        assert_eq!(r1.value_literal, r2.value_literal, "read {} value", i);
        assert_eq!(r1.site, r2.site, "read {} site", i);
    }

    // Compare exits field by field.
    for (i, (e1, e2)) in facts1.exits().iter().zip(facts2.exits().iter()).enumerate() {
        assert_eq!(e1.from_location, e2.from_location, "exit {} from", i);
        assert_eq!(e1.to_location, e2.to_location, "exit {} to", i);
        assert_eq!(e1.exit_name, e2.exit_name, "exit {} name", i);
        assert_eq!(e1.is_conditional, e2.is_conditional, "exit {} cond", i);
    }

    // Compare choices field by field.
    for (i, (c1, c2)) in facts1.choices().iter().zip(facts2.choices().iter()).enumerate() {
        assert_eq!(c1.choice_id, c2.choice_id, "choice {} id", i);
        assert_eq!(c1.section, c2.section, "choice {} section", i);
        assert_eq!(c1.label, c2.label, "choice {} label", i);
        assert_eq!(c1.sticky, c2.sticky, "choice {} sticky", i);
    }
}

// ── Site resolution ──

#[test]
fn facts_site_choice_resolution() {
    let facts = extract_fixture_facts("two-room-key-puzzle.urd.md");
    let choice = &facts.choices()[0];
    let site = FactSite::Choice(choice.choice_id.clone());

    let resolved = facts.resolve_site(&site);
    assert!(resolved.is_some(), "Choice site should resolve");
    assert!(matches!(resolved.unwrap(), SiteOwner::Choice(_)));
}

#[test]
fn facts_site_exit_resolution() {
    let facts = extract_fixture_facts("two-room-key-puzzle.urd.md");
    let exit = &facts.exits()[0];
    let exit_id = exit.exit_id();

    // exit_by_id and exit_by_location_and_name should return the same exit.
    let by_id = facts.exit_by_id(&exit_id);
    let by_parts = facts.exit_by_location_and_name(&exit.from_location, &exit.exit_name);
    assert!(by_id.is_some());
    assert!(by_parts.is_some());
    assert_eq!(by_id.unwrap().exit_id(), by_parts.unwrap().exit_id());
}

#[test]
fn facts_site_rule_resolution() {
    let facts = extract_fixture_facts("sunken-citadel.urd.md");
    for rule in facts.rules() {
        let site = FactSite::Rule(rule.rule_id.clone());
        let resolved = facts.resolve_site(&site);
        assert!(
            resolved.is_some(),
            "Rule site '{}' should resolve",
            rule.rule_id
        );
        assert!(matches!(resolved.unwrap(), SiteOwner::Rule(_)));
    }
}

// ── Jump target resolution ──

#[test]
fn facts_jump_target_exit_resolution() {
    let facts = extract_fixture_facts("sunken-citadel.urd.md");
    for jump in facts.jumps() {
        if let JumpTarget::Exit(ref exit_id) = jump.target {
            assert!(
                facts.exit_by_id(exit_id).is_some(),
                "JumpTarget::Exit({}) should resolve to an existing ExitEdge",
                exit_id
            );
        }
    }
}

#[test]
fn facts_jump_section_targets() {
    let facts = extract_fixture_facts("interrogation/main.urd.md");
    // The jump -> confession should resolve to a section.
    let section_jumps: Vec<_> = facts
        .jumps()
        .iter()
        .filter(|j| matches!(&j.target, JumpTarget::Section(_)))
        .collect();
    assert_eq!(section_jumps.len(), 1);
}

// ── PropertyDependencyIndex — SF-2 ──

#[test]
fn index_build_deterministic() {
    let facts = extract_fixture_facts("locked-garden.urd.md");
    let index1 = PropertyDependencyIndex::build(&facts);
    let index2 = PropertyDependencyIndex::build(&facts);
    let json1 = index1.to_json().to_string();
    let json2 = index2.to_json().to_string();
    assert_eq!(json1, json2, "Two builds from same FactSet must produce identical JSON");
}

#[test]
fn index_read_but_never_written_empty_on_clean() {
    let facts = extract_fixture_facts("negative-factset-diagnostics.urd.md");
    let index = PropertyDependencyIndex::build(&facts);
    assert!(
        index.read_but_never_written().is_empty(),
        "Clean fixture should have no read-but-never-written properties, got: {:?}",
        index.read_but_never_written()
    );
}

#[test]
fn index_written_but_never_read_empty_on_clean() {
    let facts = extract_fixture_facts("negative-factset-diagnostics.urd.md");
    let index = PropertyDependencyIndex::build(&facts);
    assert!(
        index.written_but_never_read().is_empty(),
        "Clean fixture should have no written-but-never-read properties, got: {:?}",
        index.written_but_never_read()
    );
}

#[test]
fn index_read_but_never_written_positive() {
    let facts = extract_fixture_facts("positive-factset-diagnostics.urd.md");
    let index = PropertyDependencyIndex::build(&facts);
    let keys = index.read_but_never_written();
    let names: Vec<_> = keys
        .iter()
        .map(|k| format!("{}.{}", k.entity_type, k.property))
        .collect();
    assert!(
        names.contains(&"NPC.suspicion".to_string()),
        "Expected NPC.suspicion in read_but_never_written, got: {:?}",
        names
    );
}

#[test]
fn index_written_but_never_read_positive() {
    let facts = extract_fixture_facts("positive-factset-diagnostics.urd.md");
    let index = PropertyDependencyIndex::build(&facts);
    let keys = index.written_but_never_read();
    let names: Vec<_> = keys
        .iter()
        .map(|k| format!("{}.{}", k.entity_type, k.property))
        .collect();
    assert!(
        names.contains(&"NPC.loyalty".to_string()),
        "Expected NPC.loyalty in written_but_never_read, got: {:?}",
        names
    );
}

#[test]
fn index_to_json_structure() {
    let facts = extract_fixture_facts("locked-garden.urd.md");
    let index = PropertyDependencyIndex::build(&facts);
    let json = index.to_json();

    let properties = json["properties"]
        .as_array()
        .expect("JSON must have 'properties' array");
    assert!(!properties.is_empty(), "Properties should not be empty");

    let summary = &json["summary"];
    assert!(summary["total_properties"].as_u64().unwrap() > 0);
    assert!(summary["total_reads"].as_u64().unwrap() > 0);
    assert!(summary["total_writes"].as_u64().unwrap() > 0);

    for prop in properties {
        assert!(prop["entity_type"].is_string());
        assert!(prop["property"].is_string());
        assert!(prop["read_count"].is_number());
        assert!(prop["write_count"].is_number());
        assert!(prop["read_indices"].is_array());
        assert!(prop["write_indices"].is_array());
        assert!(prop["orphaned"].is_null() || prop["orphaned"].is_string());
    }
}

#[test]
fn index_to_json_orphaned_flags() {
    let facts = extract_fixture_facts("positive-factset-diagnostics.urd.md");
    let index = PropertyDependencyIndex::build(&facts);
    let json = index.to_json();
    let properties = json["properties"].as_array().unwrap();

    let suspicion = properties
        .iter()
        .find(|p| p["entity_type"] == "NPC" && p["property"] == "suspicion")
        .expect("Should find NPC.suspicion");
    assert_eq!(suspicion["orphaned"], "read_never_written");

    let loyalty = properties
        .iter()
        .find(|p| p["entity_type"] == "NPC" && p["property"] == "loyalty")
        .expect("Should find NPC.loyalty");
    assert_eq!(loyalty["orphaned"], "written_never_read");

    let has_both = properties.iter().find(|p| {
        p["orphaned"].is_null()
            && p["read_count"].as_u64().unwrap() > 0
            && p["write_count"].as_u64().unwrap() > 0
    });
    assert!(
        has_both.is_some(),
        "Should have at least one non-orphaned property"
    );
}

#[test]
fn index_all_fixtures_no_panic() {
    let fixtures = [
        "two-room-key-puzzle.urd.md",
        "tavern-scene.urd.md",
        "monty-hall.urd.md",
        "sunken-citadel.urd.md",
        "locked-garden.urd.md",
        "negative-factset-diagnostics.urd.md",
        "positive-factset-diagnostics.urd.md",
        "positive-factset-circular-deep.urd.md",
    ];
    for name in &fixtures {
        let facts = extract_fixture_facts(name);
        let index = PropertyDependencyIndex::build(&facts);
        let _json = index.to_json();
        let _rnw = index.read_but_never_written();
        let _wnr = index.written_but_never_read();
    }
}

#[test]
fn index_matches_d1_d2() {
    let facts = extract_fixture_facts("positive-factset-diagnostics.urd.md");
    let index = PropertyDependencyIndex::build(&facts);

    let diags = urd_compiler::analyze::analyze(&facts, &index);

    // D1 property keys from diagnostics
    let d1_keys_from_diags: std::collections::HashSet<String> = diags
        .iter()
        .filter(|d| d.code == "URD601")
        .filter_map(|d| {
            let start = d.message.find('\'')? + 1;
            let end = d.message[start..].find('\'')?;
            Some(d.message[start..start + end].to_string())
        })
        .collect();

    let d1_keys_from_index: std::collections::HashSet<String> = index
        .read_but_never_written()
        .iter()
        .map(|k| format!("{}.{}", k.entity_type, k.property))
        .collect();

    assert_eq!(
        d1_keys_from_diags, d1_keys_from_index,
        "D1 diagnostic keys must match read_but_never_written() keys"
    );

    // D2 property keys from diagnostics
    let d2_keys_from_diags: std::collections::HashSet<String> = diags
        .iter()
        .filter(|d| d.code == "URD602")
        .filter_map(|d| {
            let start = d.message.find('\'')? + 1;
            let end = d.message[start..].find('\'')?;
            Some(d.message[start..start + end].to_string())
        })
        .collect();

    let d2_keys_from_index: std::collections::HashSet<String> = index
        .written_but_never_read()
        .iter()
        .map(|k| format!("{}.{}", k.entity_type, k.property))
        .collect();

    assert_eq!(
        d2_keys_from_diags, d2_keys_from_index,
        "D2 diagnostic keys must match written_but_never_read() keys"
    );
}

#[test]
fn index_locked_garden_properties() {
    let facts = extract_fixture_facts("locked-garden.urd.md");
    let index = PropertyDependencyIndex::build(&facts);
    let json = index.to_json();
    let properties = json["properties"].as_array().unwrap();

    let prop_names: Vec<String> = properties
        .iter()
        .map(|p| {
            format!(
                "{}.{}",
                p["entity_type"].as_str().unwrap(),
                p["property"].as_str().unwrap()
            )
        })
        .collect();

    // Must be sorted lexicographically
    let mut sorted = prop_names.clone();
    sorted.sort();
    assert_eq!(
        prop_names, sorted,
        "Properties must be sorted lexicographically"
    );

    assert!(
        !prop_names.is_empty(),
        "Locked Garden should have properties"
    );
    eprintln!("Locked Garden properties: {:?}", prop_names);
}

#[test]
fn index_sunken_citadel_summary() {
    let facts = extract_fixture_facts("sunken-citadel.urd.md");
    let index = PropertyDependencyIndex::build(&facts);
    let json = index.to_json();
    let summary = &json["summary"];

    assert!(
        summary["total_properties"].as_u64().unwrap() > 0,
        "Should have properties"
    );
    assert!(
        summary["total_reads"].as_u64().unwrap() > 0,
        "Should have reads"
    );
    assert!(
        summary["total_writes"].as_u64().unwrap() > 0,
        "Should have writes"
    );

    eprintln!(
        "Sunken Citadel: {} properties, {} reads, {} writes, {} read-never-written, {} written-never-read",
        summary["total_properties"],
        summary["total_reads"],
        summary["total_writes"],
        summary["read_never_written"],
        summary["written_never_read"],
    );
}
