/// Tests for Phase 1: PARSE
///
/// Test categories from the PARSE phase brief:
/// - Every grammar rule exercised (unit tests)
/// - Error recovery scenarios
/// - Span accuracy verification
/// - Negative tests: grammar rejections

use urd_compiler::ast::*;
use urd_compiler::parse;
use urd_compiler::diagnostics::DiagnosticCollector;

// ── Helper functions ──

fn parse_source(source: &str) -> (Option<FileAst>, DiagnosticCollector) {
    let mut diag = DiagnosticCollector::new();
    let ast = parse::parse(&"test.urd.md".to_string(), source, &mut diag);
    (ast, diag)
}

fn parse_content_only(source: &str) -> Vec<ContentNode> {
    let (ast, _) = parse_source(source);
    ast.expect("parse returned None").content
}

fn first_node(source: &str) -> ContentNode {
    let nodes = parse_content_only(source);
    assert!(!nodes.is_empty(), "expected at least one node");
    nodes.into_iter().next().unwrap()
}

// ── Unit Tests: Grammar Rule Coverage ──

#[test]
fn location_heading() {
    match first_node("# The Rusty Anchor") {
        ContentNode::LocationHeading(h) => {
            assert_eq!(h.display_name, "The Rusty Anchor");
        }
        other => panic!("expected LocationHeading, got {:?}", other),
    }
}

#[test]
fn sequence_heading() {
    match first_node("## The Game") {
        ContentNode::SequenceHeading(h) => {
            assert_eq!(h.display_name, "The Game");
        }
        other => panic!("expected SequenceHeading, got {:?}", other),
    }
}

#[test]
fn phase_heading() {
    match first_node("### Choose") {
        ContentNode::PhaseHeading(h) => {
            assert_eq!(h.display_name, "Choose");
            assert!(!h.auto);
        }
        other => panic!("expected PhaseHeading, got {:?}", other),
    }
}

#[test]
fn phase_heading_auto() {
    match first_node("### Reveal (auto)") {
        ContentNode::PhaseHeading(h) => {
            assert_eq!(h.display_name, "Reveal");
            assert!(h.auto);
        }
        other => panic!("expected PhaseHeading, got {:?}", other),
    }
}

#[test]
fn section_label() {
    match first_node("== topics") {
        ContentNode::SectionLabel(s) => {
            assert_eq!(s.name, "topics");
        }
        other => panic!("expected SectionLabel, got {:?}", other),
    }
}

#[test]
fn entity_speech() {
    match first_node("@arina: What'll it be?") {
        ContentNode::EntitySpeech(s) => {
            assert_eq!(s.entity_ref, "arina");
            assert_eq!(s.text, "What'll it be?");
        }
        other => panic!("expected EntitySpeech, got {:?}", other),
    }
}

#[test]
fn stage_direction() {
    match first_node("@arina leans in close.") {
        ContentNode::StageDirection(s) => {
            assert_eq!(s.entity_ref, "arina");
            assert_eq!(s.text, "leans in close.");
        }
        other => panic!("expected StageDirection, got {:?}", other),
    }
}

#[test]
fn entity_presence() {
    match first_node("[@arina, @barrel]") {
        ContentNode::EntityPresence(p) => {
            assert_eq!(p.entity_refs, vec!["arina", "barrel"]);
        }
        other => panic!("expected EntityPresence, got {:?}", other),
    }
}

#[test]
fn one_shot_choice() {
    match first_node("* Ask about the ship") {
        ContentNode::Choice(c) => {
            assert!(!c.sticky);
            assert_eq!(c.label, "Ask about the ship");
            assert!(c.target.is_none());
            assert!(c.target_type.is_none());
        }
        other => panic!("expected Choice, got {:?}", other),
    }
}

#[test]
fn sticky_choice() {
    match first_node("+ Order a drink") {
        ContentNode::Choice(c) => {
            assert!(c.sticky);
            assert_eq!(c.label, "Order a drink");
        }
        other => panic!("expected Choice, got {:?}", other),
    }
}

#[test]
fn choice_with_entity_target() {
    match first_node("* Use key -> @cell_door") {
        ContentNode::Choice(c) => {
            assert_eq!(c.target, Some("cell_door".to_string()));
            assert!(c.target_type.is_none());
        }
        other => panic!("expected Choice, got {:?}", other),
    }
}

#[test]
fn choice_with_type_target() {
    match first_node("* Pick a door -> any Door") {
        ContentNode::Choice(c) => {
            assert!(c.target.is_none());
            assert_eq!(c.target_type, Some("Door".to_string()));
        }
        other => panic!("expected Choice, got {:?}", other),
    }
}

#[test]
fn choice_with_section_target() {
    match first_node("* Back off -> interrogation") {
        ContentNode::Choice(c) => {
            assert_eq!(c.target, Some("interrogation".to_string()));
            assert!(c.target_type.is_none());
        }
        other => panic!("expected Choice, got {:?}", other),
    }
}

#[test]
fn simple_condition() {
    match first_node("? @guard.mood == neutral") {
        ContentNode::Condition(c) => {
            match &c.expr {
                ConditionExpr::PropertyComparison(pc) => {
                    assert_eq!(pc.entity_ref, "guard");
                    assert_eq!(pc.property, "mood");
                    assert_eq!(pc.operator, "==");
                    assert_eq!(pc.value, "neutral");
                }
                other => panic!("expected PropertyComparison, got {:?}", other),
            }
        }
        other => panic!("expected Condition, got {:?}", other),
    }
}

#[test]
fn containment_condition() {
    match first_node("? @rusty_key in here") {
        ContentNode::Condition(c) => {
            match &c.expr {
                ConditionExpr::ContainmentCheck(cc) => {
                    assert_eq!(cc.entity_ref, "rusty_key");
                    assert_eq!(cc.container_ref, "here");
                    assert!(!cc.negated);
                }
                other => panic!("expected ContainmentCheck, got {:?}", other),
            }
        }
        other => panic!("expected Condition, got {:?}", other),
    }
}

#[test]
fn negated_containment() {
    match first_node("? @rusty_key not in player") {
        ContentNode::Condition(c) => {
            match &c.expr {
                ConditionExpr::ContainmentCheck(cc) => {
                    assert!(cc.negated);
                    assert_eq!(cc.entity_ref, "rusty_key");
                    assert_eq!(cc.container_ref, "player");
                }
                other => panic!("expected ContainmentCheck, got {:?}", other),
            }
        }
        other => panic!("expected Condition, got {:?}", other),
    }
}

#[test]
fn exhaustion_condition() {
    match first_node("? topics.exhausted") {
        ContentNode::Condition(c) => {
            match &c.expr {
                ConditionExpr::ExhaustionCheck(ec) => {
                    assert_eq!(ec.section_name, "topics");
                }
                other => panic!("expected ExhaustionCheck, got {:?}", other),
            }
        }
        other => panic!("expected Condition, got {:?}", other),
    }
}

#[test]
fn or_condition_block() {
    let source = "? any:\n  @guard.mood == neutral\n  @key in here";
    let nodes = parse_content_only(source);
    match &nodes[0] {
        ContentNode::OrConditionBlock(oc) => {
            assert_eq!(oc.conditions.len(), 2);
        }
        other => panic!("expected OrConditionBlock, got {:?}", other),
    }
}

#[test]
fn set_effect() {
    match first_node("> @guard.mood = neutral") {
        ContentNode::Effect(e) => {
            match &e.effect_type {
                EffectType::Set { target_prop, operator, value_expr } => {
                    assert_eq!(target_prop, "@guard.mood");
                    assert_eq!(operator, "=");
                    assert_eq!(value_expr, "neutral");
                }
                other => panic!("expected Set, got {:?}", other),
            }
        }
        other => panic!("expected Effect, got {:?}", other),
    }
}

#[test]
fn increment_effect() {
    match first_node("> @arina.trust + 5") {
        ContentNode::Effect(e) => {
            match &e.effect_type {
                EffectType::Set { target_prop, operator, value_expr } => {
                    assert_eq!(target_prop, "@arina.trust");
                    assert_eq!(operator, "+");
                    assert_eq!(value_expr, "5");
                }
                other => panic!("expected Set, got {:?}", other),
            }
        }
        other => panic!("expected Effect, got {:?}", other),
    }
}

#[test]
fn move_effect() {
    match first_node("> move @rusty_key -> player") {
        ContentNode::Effect(e) => {
            match &e.effect_type {
                EffectType::Move { entity_ref, destination_ref } => {
                    assert_eq!(entity_ref, "rusty_key");
                    assert_eq!(destination_ref, "player");
                }
                other => panic!("expected Move, got {:?}", other),
            }
        }
        other => panic!("expected Effect, got {:?}", other),
    }
}

#[test]
fn reveal_effect() {
    match first_node("> reveal @door_1.prize") {
        ContentNode::Effect(e) => {
            match &e.effect_type {
                EffectType::Reveal { target_prop } => {
                    assert_eq!(target_prop, "@door_1.prize");
                }
                other => panic!("expected Reveal, got {:?}", other),
            }
        }
        other => panic!("expected Effect, got {:?}", other),
    }
}

#[test]
fn destroy_effect() {
    match first_node("> destroy @rusty_key") {
        ContentNode::Effect(e) => {
            match &e.effect_type {
                EffectType::Destroy { entity_ref } => {
                    assert_eq!(entity_ref, "rusty_key");
                }
                other => panic!("expected Destroy, got {:?}", other),
            }
        }
        other => panic!("expected Effect, got {:?}", other),
    }
}

#[test]
fn section_jump() {
    match first_node("-> topics") {
        ContentNode::Jump(j) => {
            assert_eq!(j.target, "topics");
            assert!(!j.is_exit_qualified);
        }
        other => panic!("expected Jump, got {:?}", other),
    }
}

#[test]
fn exit_qualified_jump() {
    match first_node("-> exit:harbor") {
        ContentNode::Jump(j) => {
            assert_eq!(j.target, "harbor");
            assert!(j.is_exit_qualified);
        }
        other => panic!("expected Jump, got {:?}", other),
    }
}

#[test]
fn exit_declaration() {
    match first_node("-> north: Corridor") {
        ContentNode::ExitDeclaration(e) => {
            assert_eq!(e.direction, "north");
            assert_eq!(e.destination, "Corridor");
        }
        other => panic!("expected ExitDeclaration, got {:?}", other),
    }
}

#[test]
fn exit_with_children() {
    let source = "-> north: Corridor\n  ? @door.locked == false\n  ! Locked.";
    let nodes = parse_content_only(source);
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        ContentNode::ExitDeclaration(e) => {
            assert_eq!(e.direction, "north");
            assert_eq!(e.children.len(), 2);
            assert!(matches!(&e.children[0], ContentNode::Condition(_)));
            assert!(matches!(&e.children[1], ContentNode::BlockedMessage(_)));
        }
        other => panic!("expected ExitDeclaration, got {:?}", other),
    }
}

#[test]
fn exit_with_deep_children() {
    let source = "-> north: Corridor\n    ? @door.locked == false";
    let nodes = parse_content_only(source);
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        ContentNode::ExitDeclaration(e) => {
            assert_eq!(e.children.len(), 1);
            assert!(matches!(&e.children[0], ContentNode::Condition(_)));
        }
        other => panic!("expected ExitDeclaration, got {:?}", other),
    }
}

#[test]
fn exit_non_child_content() {
    let source = "-> north: Corridor\n  @arina: Hello";
    let nodes = parse_content_only(source);
    assert_eq!(nodes.len(), 2);
    match &nodes[0] {
        ContentNode::ExitDeclaration(e) => {
            assert!(e.children.is_empty());
        }
        other => panic!("expected ExitDeclaration, got {:?}", other),
    }
    assert!(matches!(&nodes[1], ContentNode::EntitySpeech(_)));
}

#[test]
fn exit_children_with_blank_lines() {
    let source = "-> north: Corridor\n\n  ? @door.locked == false\n\n  ! Locked.";
    let nodes = parse_content_only(source);
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        ContentNode::ExitDeclaration(e) => {
            assert_eq!(e.children.len(), 2);
        }
        other => panic!("expected ExitDeclaration, got {:?}", other),
    }
}

#[test]
fn choice_body_with_blank_lines() {
    let source = "* Ask\n\n  ? topics.exhausted\n\n  > reveal @x.y\nBack to prose";
    let nodes = parse_content_only(source);
    assert_eq!(nodes.len(), 2);
    match &nodes[0] {
        ContentNode::Choice(c) => {
            assert_eq!(c.label, "Ask");
            assert_eq!(c.content.len(), 2);
            assert!(matches!(&c.content[0], ContentNode::Condition(_)));
            assert!(matches!(&c.content[1], ContentNode::Effect(_)));
        }
        other => panic!("expected Choice, got {:?}", other),
    }
    match &nodes[1] {
        ContentNode::Prose(p) => {
            assert_eq!(p.text, "Back to prose");
        }
        other => panic!("expected Prose, got {:?}", other),
    }
}

#[test]
fn blocked_message() {
    match first_node("! The iron door is locked.") {
        ContentNode::BlockedMessage(b) => {
            assert_eq!(b.text, "The iron door is locked.");
        }
        other => panic!("expected BlockedMessage, got {:?}", other),
    }
}

#[test]
fn prose() {
    match first_node("A dim stone cell.") {
        ContentNode::Prose(p) => {
            assert_eq!(p.text, "A dim stone cell.");
        }
        other => panic!("expected Prose, got {:?}", other),
    }
}

#[test]
fn line_comment() {
    match first_node("// hub prompt") {
        ContentNode::Comment(c) => {
            assert_eq!(c.text, "hub prompt");
        }
        other => panic!("expected Comment, got {:?}", other),
    }
}

#[test]
fn inline_comment_entity_speech() {
    match first_node("@arina: Hello // greeting") {
        ContentNode::EntitySpeech(s) => {
            assert_eq!(s.text, "Hello");
        }
        other => panic!("expected EntitySpeech, got {:?}", other),
    }
}

#[test]
fn heading_inline_comment() {
    match first_node("# Cell // test location") {
        ContentNode::LocationHeading(h) => {
            assert_eq!(h.display_name, "Cell");
        }
        other => panic!("expected LocationHeading, got {:?}", other),
    }
}

#[test]
fn line_comment_under_heading() {
    let source = "# Cell\n// author note";
    let nodes = parse_content_only(source);
    assert_eq!(nodes.len(), 2);
    assert!(matches!(&nodes[0], ContentNode::LocationHeading(_)));
    match &nodes[1] {
        ContentNode::Comment(c) => {
            assert_eq!(c.text, "author note");
        }
        other => panic!("expected Comment, got {:?}", other),
    }
}

#[test]
fn rule_block() {
    let source = "rule monty_reveals:\n  actor: @host action reveal\n  > reveal @door_1.prize";
    let nodes = parse_content_only(source);
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        ContentNode::RuleBlock(r) => {
            assert_eq!(r.name, "monty_reveals");
            assert_eq!(r.actor, "host");
            assert_eq!(r.trigger, "action reveal");
            assert_eq!(r.effects.len(), 1);
        }
        other => panic!("expected RuleBlock, got {:?}", other),
    }
}

// ── Frontmatter Tests ──

#[test]
fn empty_frontmatter() {
    let source = "---\n---\n# Heading";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(ast.frontmatter.is_some());
    assert!(ast.frontmatter.unwrap().entries.is_empty());
    assert!(!diag.has_errors());
}

#[test]
fn world_block() {
    let source = "---\nworld:\n  name: Test World\n  start: tavern\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    let fm = ast.frontmatter.unwrap();
    assert!(!diag.has_errors());
    assert_eq!(fm.entries.len(), 1);
    assert_eq!(fm.entries[0].key, "world");
    match &fm.entries[0].value {
        FrontmatterValue::WorldBlock(wb) => {
            assert_eq!(wb.fields.len(), 2);
            assert_eq!(wb.fields[0].0, "name");
            assert_eq!(wb.fields[1].0, "start");
        }
        other => panic!("expected WorldBlock, got {:?}", other),
    }
}

#[test]
fn type_definition() {
    let source = "---\ntypes:\n  Guard [interactable, mobile]:\n    mood: enum(hostile, neutral)\n    ~prize: string\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    // types entry contains a map with type definitions
    let types_entry = &fm.entries[0];
    assert_eq!(types_entry.key, "types");
    match &types_entry.value {
        FrontmatterValue::Map(entries) => {
            assert_eq!(entries.len(), 1);
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.name, "Guard");
                    assert_eq!(td.traits, vec!["interactable", "mobile"]);
                    assert_eq!(td.properties.len(), 2);
                    assert_eq!(td.properties[0].name, "mood");
                    assert_eq!(td.properties[0].property_type, "enum");
                    assert_eq!(td.properties[1].name, "prize");
                    assert_eq!(td.properties[1].visibility, Some("hidden".to_string()));
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn entity_declaration() {
    let source = "---\nentities:\n  @rusty_key: Key { name: \"Rusty Key\" }\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    let entities_entry = &fm.entries[0];
    match &entities_entry.value {
        FrontmatterValue::Map(entries) => {
            assert_eq!(entries.len(), 1);
            match &entries[0].value {
                FrontmatterValue::EntityDecl(ed) => {
                    assert_eq!(ed.id, "rusty_key");
                    assert_eq!(ed.type_name, "Key");
                    assert_eq!(ed.property_overrides.len(), 1);
                    assert_eq!(ed.property_overrides[0].0, "name");
                }
                other => panic!("expected EntityDecl, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn import_declaration() {
    let source = "---\nimport: ./world.urd.md\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    assert_eq!(fm.entries.len(), 1);
    match &fm.entries[0].value {
        FrontmatterValue::ImportDecl(id) => {
            assert_eq!(id.path, "./world.urd.md");
        }
        other => panic!("expected ImportDecl, got {:?}", other),
    }
}

#[test]
fn no_frontmatter() {
    let source = "# Heading\nSome prose.";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(ast.frontmatter.is_none());
    assert!(!diag.has_errors());
    assert_eq!(ast.content.len(), 2);
}

// ── Error Recovery Tests ──

#[test]
fn unclosed_frontmatter() {
    let source = "---\nworld: test\n";
    let (ast, diag) = parse_source(source);
    assert!(ast.is_none());
    assert!(diag.has_errors());
    let errors = diag.all();
    assert_eq!(errors[0].code, "URD101");
}

#[test]
fn bad_line_in_content() {
    let source = "# Tavern\n%%% garbage\n@arina: Hello";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert_eq!(ast.content.len(), 3);
    assert!(matches!(&ast.content[0], ContentNode::LocationHeading(_)));
    // %%% garbage should be Prose (fallback), not ErrorNode
    // since it doesn't match any grammar rejection pattern
    assert!(matches!(&ast.content[2], ContentNode::EntitySpeech(_)));
}

#[test]
fn tab_indentation() {
    let source = "\t* Choice text";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(diag.has_errors());
    let tab_errors: Vec<_> = diag.all().iter().filter(|d| d.code == "URD102").collect();
    assert!(!tab_errors.is_empty());
    // Choice should still be parsed
    assert_eq!(ast.content.len(), 1);
    assert!(matches!(&ast.content[0], ContentNode::Choice(_)));
}

#[test]
fn bad_frontmatter_line() {
    let source = "---\nworld: test\n<<: merge\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(diag.has_errors());
    let errors: Vec<_> = diag.all().iter().filter(|d| d.code == "URD107").collect();
    assert_eq!(errors.len(), 1);
    // WorldBlock should still be produced
    let fm = ast.frontmatter.unwrap();
    assert!(!fm.entries.is_empty());
}

#[test]
fn multiple_errors() {
    let source = "# Good heading\n%%% bad1\n@arina: Hello\n%%% bad2\nSome prose\n%%% bad3";
    let (ast, _) = parse_source(source);
    let ast = ast.unwrap();
    // All lines should produce nodes (bad lines become Prose as fallback)
    assert_eq!(ast.content.len(), 6);
    assert!(matches!(&ast.content[0], ContentNode::LocationHeading(_)));
    assert!(matches!(&ast.content[2], ContentNode::EntitySpeech(_)));
    assert!(matches!(&ast.content[4], ContentNode::Prose(_)));
}

#[test]
fn file_size_limit() {
    let big = "x".repeat(1_048_577);
    let (ast, diag) = parse_source(&big);
    assert!(ast.is_none());
    assert!(diag.has_errors());
    assert_eq!(diag.all()[0].code, "URD103");
}

// ── Span Accuracy Tests ──

#[test]
fn span_first_line() {
    let source = "# Heading";
    let nodes = parse_content_only(source);
    let span = match &nodes[0] {
        ContentNode::LocationHeading(h) => &h.span,
        other => panic!("expected LocationHeading, got {:?}", other),
    };
    assert_eq!(span.start_line, 1);
    assert_eq!(span.start_col, 1);
}

#[test]
fn span_entity_speech_at_line_5() {
    let source = "# Heading\n\nSome prose.\n\n@arina: Hello";
    let nodes = parse_content_only(source);
    // Find the entity speech node
    let speech = nodes.iter().find(|n| matches!(n, ContentNode::EntitySpeech(_))).unwrap();
    match speech {
        ContentNode::EntitySpeech(s) => {
            assert_eq!(s.span.start_line, 5);
            assert_eq!(s.span.start_col, 1);
        }
        _ => unreachable!(),
    }
}

#[test]
fn span_indented_condition() {
    let source = "* Choice\n  ? @guard.mood == hostile";
    let nodes = parse_content_only(source);
    match &nodes[0] {
        ContentNode::Choice(c) => {
            assert_eq!(c.content.len(), 1);
            match &c.content[0] {
                ContentNode::Condition(cond) => {
                    assert_eq!(cond.span.start_line, 2);
                    // Brief: "An indented condition has span.start_col: 3 (byte offset after two spaces)"
                    assert_eq!(cond.span.start_col, 3);
                }
                other => panic!("expected Condition, got {:?}", other),
            }
        }
        other => panic!("expected Choice, got {:?}", other),
    }
}

#[test]
fn span_after_frontmatter() {
    let source = "---\nworld:\n  name: Test\n---\n# Heading";
    let (ast, _) = parse_source(source);
    let ast = ast.unwrap();
    match &ast.content[0] {
        ContentNode::LocationHeading(h) => {
            assert_eq!(h.span.start_line, 5);
        }
        other => panic!("expected LocationHeading, got {:?}", other),
    }
}

// ── Negative Tests: Grammar Rejections ──

#[test]
fn missing_space_after_heading() {
    let source = "#Heading";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(diag.has_errors());
    assert_eq!(ast.content.len(), 1);
    assert!(matches!(&ast.content[0], ContentNode::ErrorNode(_)));
}

#[test]
fn missing_space_after_condition_sigil() {
    let source = "?@guard.mood == hostile";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(diag.has_errors());
    assert_eq!(ast.content.len(), 1);
    assert!(matches!(&ast.content[0], ContentNode::ErrorNode(_)));
}

// ── Frontmatter Educational Errors ──

#[test]
fn yaml_anchors_rejected() {
    let source = "---\n&anchor value\n---\n";
    let (_, diag) = parse_source(source);
    let errors: Vec<_> = diag.all().iter().filter(|d| d.code == "URD105").collect();
    assert_eq!(errors.len(), 1);
}

#[test]
fn yaml_aliases_rejected() {
    let source = "---\n*alias\n---\n";
    let (_, diag) = parse_source(source);
    let errors: Vec<_> = diag.all().iter().filter(|d| d.code == "URD106").collect();
    assert_eq!(errors.len(), 1);
}

#[test]
fn yaml_merge_keys_rejected() {
    let source = "---\n<<: merge\n---\n";
    let (_, diag) = parse_source(source);
    let errors: Vec<_> = diag.all().iter().filter(|d| d.code == "URD107").collect();
    assert_eq!(errors.len(), 1);
}

#[test]
fn yaml_custom_tags_rejected() {
    let source = "---\n!!str value\n---\n";
    let (_, diag) = parse_source(source);
    let errors: Vec<_> = diag.all().iter().filter(|d| d.code == "URD108").collect();
    assert_eq!(errors.len(), 1);
}

#[test]
fn block_style_lists_rejected() {
    let source = "---\n- item1\n---\n";
    let (_, diag) = parse_source(source);
    let errors: Vec<_> = diag.all().iter().filter(|d| d.code == "URD109").collect();
    assert_eq!(errors.len(), 1);
}

// ── BOM handling ──

#[test]
fn bom_stripping() {
    let source = "\u{FEFF}---\nworld:\n  name: Test\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    assert!(ast.frontmatter.is_some());
}

// ── Integration Tests: Full File Parsing ──

#[test]
fn tavern_scene() {
    let source = r#"---
world:
  name: The Rusty Anchor
  start: tavern
types:
  Character [interactable, mobile]:
    mood: enum(hostile, neutral, friendly) = neutral
    trust: integer = 0
entities:
  @arina: Character { mood: "friendly" }
---
# The Rusty Anchor

[@arina]

== topics

@arina: What'll it be?

* Ask about the ship
  @arina: She's seen better days.
  > @arina.trust + 1
+ Order a drink
  @arina: Coming right up.

? topics.exhausted

@arina leans back and sighs.
-> exit:harbor
"#;
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors(), "unexpected errors: {:?}", diag.all());
    assert!(ast.frontmatter.is_some());
    assert!(!ast.content.is_empty());
}

#[test]
fn two_room_key_puzzle() {
    let source = r#"---
world:
  name: Key Puzzle
  start: cell
  entry: cell
types:
  Key [portable]:
    name: string
  Door [interactable]:
    locked: bool = true
entities:
  @rusty_key: Key { name: "Rusty Key" }
  @cell_door: Door
---
# Cell

A dim stone cell.

[@rusty_key]

-> north: Corridor
  ? @cell_door.locked == false
  ! The iron door is locked.

* Use key -> @cell_door
  ? @rusty_key in here
  > @cell_door.locked = false
  > destroy @rusty_key

# Corridor

You made it out.
"#;
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors(), "unexpected errors: {:?}", diag.all());
    assert!(ast.frontmatter.is_some());
    // Should have content nodes for both locations and their contents
    assert!(ast.content.len() >= 6);
}

// ── Additional Negative Tests from Brief ──

#[test]
fn uppercase_entity_id() {
    // @Guard: Hello — Identifier rule requires lowercase start
    let source = "@Guard: Hello";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(diag.has_errors());
    assert_eq!(ast.content.len(), 1);
    assert!(matches!(&ast.content[0], ContentNode::ErrorNode(_)));
}

#[test]
fn empty_choice_label() {
    // "* " — asterisk + space, no text: URD112 ErrorNode (Text requires at least one character)
    let source = "* ";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(diag.has_errors());
    assert_eq!(ast.content.len(), 1);
    assert!(matches!(&ast.content[0], ContentNode::ErrorNode(_)));
}

#[test]
fn frontmatter_nesting_depth() {
    // 9 levels of nesting (> 8) should trigger URD104
    let mut source = String::from("---\n");
    let mut key = "a".to_string();
    for i in 0..10 {
        let indent = "  ".repeat(i);
        source.push_str(&format!("{}{}:\n", indent, key));
        key = format!("b{}", i);
    }
    source.push_str("---\n");
    let (_, diag) = parse_source(&source);
    let depth_errors: Vec<_> = diag.all().iter().filter(|d| d.code == "URD104").collect();
    assert!(!depth_errors.is_empty());
}

#[test]
fn list_type_parsing() {
    let source = "---\ntypes:\n  Container [interactable]:\n    contents: list(ref(Item))\n    tags: list(string)\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors(), "unexpected errors: {:?}", diag.all());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties.len(), 2);
                    assert_eq!(td.properties[0].property_type, "list");
                    assert_eq!(td.properties[0].element_type, Some("ref".to_string()));
                    assert_eq!(td.properties[0].element_ref_type, Some("Item".to_string()));
                    assert_eq!(td.properties[1].property_type, "list");
                    assert_eq!(td.properties[1].element_type, Some("string".to_string()));
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn ref_type_parsing() {
    let source = "---\ntypes:\n  Exit [interactable]:\n    target: ref(Room)\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "ref");
                    assert_eq!(td.properties[0].ref_type, Some("Room".to_string()));
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

// ── Type alias normalisation tests ──

#[test]
fn type_alias_int_normalised() {
    let source = "---\ntypes:\n  Stat []:\n    trust: int\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors(), "unexpected errors: {:?}", diag.all());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "integer");
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn type_alias_bool_normalised() {
    let source = "---\ntypes:\n  Flag []:\n    active: bool\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "boolean");
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn type_alias_num_normalised() {
    let source = "---\ntypes:\n  Stat []:\n    weight: num\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "number");
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn type_alias_str_normalised() {
    let source = "---\ntypes:\n  Label []:\n    text: str\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "string");
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn int_range_shorthand() {
    let source = "---\ntypes:\n  Stat []:\n    trust: int(0, 100)\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors(), "unexpected errors: {:?}", diag.all());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "integer");
                    assert_eq!(td.properties[0].min, Some(0.0));
                    assert_eq!(td.properties[0].max, Some(100.0));
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn integer_range_shorthand() {
    let source = "---\ntypes:\n  Stat []:\n    trust: integer(0, 100)\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "integer");
                    assert_eq!(td.properties[0].min, Some(0.0));
                    assert_eq!(td.properties[0].max, Some(100.0));
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn num_range_shorthand() {
    let source = "---\ntypes:\n  Stat []:\n    weight: num(0.0, 10.0)\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "number");
                    assert_eq!(td.properties[0].min, Some(0.0));
                    assert_eq!(td.properties[0].max, Some(10.0));
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn int_range_with_default() {
    let source = "---\ntypes:\n  Stat []:\n    trust: int(0, 100) = 30\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors(), "unexpected errors: {:?}", diag.all());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "integer");
                    assert_eq!(td.properties[0].min, Some(0.0));
                    assert_eq!(td.properties[0].max, Some(100.0));
                    assert_eq!(td.properties[0].default, Some(Scalar::Integer(30)));
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn int_range_malformed_graceful() {
    let source = "---\ntypes:\n  Stat []:\n    trust: int(abc, def)\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "integer");
                    assert_eq!(td.properties[0].min, None);
                    assert_eq!(td.properties[0].max, None);
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn int_empty_parens_graceful() {
    let source = "---\ntypes:\n  Stat []:\n    trust: int()\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    match &fm.entries[0].value {
        FrontmatterValue::Map(entries) => {
            match &entries[0].value {
                FrontmatterValue::TypeDef(td) => {
                    assert_eq!(td.properties[0].property_type, "integer");
                    assert_eq!(td.properties[0].min, None);
                    assert_eq!(td.properties[0].max, None);
                }
                other => panic!("expected TypeDef, got {:?}", other),
            }
        }
        other => panic!("expected Map, got {:?}", other),
    }
}

#[test]
fn multiple_imports() {
    let source = "---\nimport: ./world.urd.md\nimport: ./characters.urd.md\n---\n";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors());
    let fm = ast.frontmatter.unwrap();
    let imports: Vec<_> = fm.entries.iter()
        .filter(|e| e.key == "import")
        .collect();
    assert_eq!(imports.len(), 2);
}

#[test]
fn choice_indent_level_tracking() {
    let source = "* Top level\n  * Nested choice\n    * Deep nested";
    let nodes = parse_content_only(source);
    match &nodes[0] {
        ContentNode::Choice(c) => {
            assert_eq!(c.indent_level, 0);
            assert_eq!(c.content.len(), 1);
            match &c.content[0] {
                ContentNode::Choice(inner) => {
                    assert_eq!(inner.indent_level, 1);
                    assert_eq!(inner.content.len(), 1);
                    match &inner.content[0] {
                        ContentNode::Choice(deep) => {
                            assert_eq!(deep.indent_level, 2);
                        }
                        other => panic!("expected Choice, got {:?}", other),
                    }
                }
                other => panic!("expected Choice, got {:?}", other),
            }
        }
        other => panic!("expected Choice, got {:?}", other),
    }
}

// ── Missing Tests: Negative Cases ──

#[test]
fn invalid_identifier_character() {
    // @Guard-1: Hello — fails due to uppercase + hyphen in identifier
    let source = "@Guard-1: Hello";
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(diag.has_errors());
    assert_eq!(ast.content.len(), 1);
    assert!(matches!(&ast.content[0], ContentNode::ErrorNode(_)));
}

// ── Missing Tests: Span Accuracy ──

#[test]
fn span_multibyte_utf8() {
    // Brief: "@björk: Hej" — end_col accounts for two-byte ö in UTF-8
    // ö is 2 bytes, so total line is 13 bytes. end_col = 14 (exclusive).
    let source = "@björk: Hej";
    let (ast, _) = parse_source(source);
    let ast = ast.unwrap();
    assert_eq!(ast.content.len(), 1);
    // The identifier extraction stops at 'ö' (non-ASCII), so entity_ref is "bj"
    // and remaining starts with "ö..." which starts with a space (no) — actually
    // the ö byte (0xC3) is not alphanumeric or underscore, so id_end stops there.
    // Remaining is "örk: Hej". ö doesn't start with ': ' or ' ', so...
    // Let's just verify the span end_col accounts for multi-byte encoding
    let span = match &ast.content[0] {
        ContentNode::EntitySpeech(s) => &s.span,
        ContentNode::StageDirection(s) => &s.span,
        ContentNode::ErrorNode(e) => &e.span,
        ContentNode::Prose(p) => &p.span,
        other => panic!("unexpected node type: {:?}", other),
    };
    // The line "@björk: Hej" is 13 bytes (ö = 2 bytes). end_col = 14 (exclusive).
    assert_eq!(span.end_col, (source.len() as u32) + 1);
    assert_eq!(span.start_line, 1);
}

#[test]
fn span_tab_at_indent_position() {
    // Brief: "\t* Choice text" has indent_level: 1, span.start_col: 1
    // (tab byte is at column 1 in original source)
    let source = "\t* Choice text";
    let (ast, _) = parse_source(source);
    let ast = ast.unwrap();
    match &ast.content[0] {
        ContentNode::Choice(c) => {
            assert_eq!(c.indent_level, 1);
            // Tab is at column 1 in the original source — start_col reflects original bytes
            assert_eq!(c.span.start_col, 1);
        }
        other => panic!("expected Choice, got {:?}", other),
    }
}

// ── Missing Integration Tests ──

#[test]
fn monty_hall() {
    let source = r#"---
world:
  name: Monty Hall
  start: stage

types:
  Door [interactable]:
    ~prize: enum(goat, car)
    revealed: bool = false

entities:
  @door_1: Door { prize: "goat" }
  @door_2: Door { prize: "goat" }
  @door_3: Door { prize: "car" }
  @host: Door
---

# Stage

## The Game

### Choose

[@door_1, @door_2, @door_3]

@host: Pick a door, any door!

* Pick a door -> any Door

### Reveal (auto)

rule monty_reveals:
  actor: @host action reveal
  selects door from [@door_1, @door_2, @door_3]
    where door.prize == goat
  > reveal door.prize

@host: Let me show you what's behind one of the other doors...

### Switch

* Switch doors -> any Door
  ? @door_1.revealed == false
* Stay with your choice
  -> exit:stage
"#;
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors(), "unexpected errors: {:?}", diag.all());
    assert!(ast.frontmatter.is_some());

    // Verify key structural elements
    let fm = ast.frontmatter.as_ref().unwrap();
    assert!(!fm.entries.is_empty());

    // Should have location heading, sequence heading, phase headings,
    // entity presence, entity speech, choices, rule block
    let has_location = ast.content.iter().any(|n| matches!(n, ContentNode::LocationHeading(_)));
    let has_sequence = ast.content.iter().any(|n| matches!(n, ContentNode::SequenceHeading(_)));
    let has_phase = ast.content.iter().any(|n| matches!(n, ContentNode::PhaseHeading(_)));
    let has_rule = ast.content.iter().any(|n| matches!(n, ContentNode::RuleBlock(_)));
    let has_entity_presence = ast.content.iter().any(|n| matches!(n, ContentNode::EntityPresence(_)));
    assert!(has_location, "missing LocationHeading");
    assert!(has_sequence, "missing SequenceHeading");
    assert!(has_phase, "missing PhaseHeading");
    assert!(has_rule, "missing RuleBlock");
    assert!(has_entity_presence, "missing EntityPresence");

    // Verify the (auto) phase heading
    let auto_phase = ast.content.iter().find_map(|n| match n {
        ContentNode::PhaseHeading(p) if p.auto => Some(p),
        _ => None,
    });
    assert!(auto_phase.is_some(), "missing (auto) PhaseHeading");
    assert_eq!(auto_phase.unwrap().display_name, "Reveal");
}

#[test]
fn interrogation() {
    let source = r#"---
import: ./world.urd.md
---

# Interrogation Room

[@suspect]

== approach

@detective: We know you were there.

? any:
  @suspect.trust >= 3
  @suspect.mood == friendly

@suspect: Fine, I'll talk.
-> confession

* Press harder
  ? @suspect.mood != hostile
  @detective: Tell me what you know.
  > @suspect.trust - 1

* Show evidence
  ? @evidence in player
  @detective: Explain this.
  > @suspect.trust + 2

  * Push further
    ? @suspect.trust >= 2
    @suspect: Alright, alright...
    -> confession

== confession

@suspect: It was the butler.

-> exit:lobby
"#;
    let (ast, diag) = parse_source(source);
    let ast = ast.unwrap();
    assert!(!diag.has_errors(), "unexpected errors: {:?}", diag.all());
    assert!(ast.frontmatter.is_some());

    // Verify import
    let fm = ast.frontmatter.as_ref().unwrap();
    let has_import = fm.entries.iter().any(|e| matches!(&e.value, FrontmatterValue::ImportDecl(_)));
    assert!(has_import, "missing ImportDecl");

    // Verify key structures: OR condition block, nested choices, multiple sections
    let has_or_block = ast.content.iter().any(|n| matches!(n, ContentNode::OrConditionBlock(_)));
    assert!(has_or_block, "missing OrConditionBlock");

    let section_count = ast.content.iter().filter(|n| matches!(n, ContentNode::SectionLabel(_))).count();
    assert_eq!(section_count, 2, "expected 2 section labels");

    // Verify choice nesting (depth 2): "Show evidence" contains "Push further"
    let choices: Vec<_> = ast.content.iter().filter_map(|n| match n {
        ContentNode::Choice(c) => Some(c),
        _ => None,
    }).collect();
    let nested_choice = choices.iter().find(|c| c.label == "Show evidence");
    assert!(nested_choice.is_some(), "missing 'Show evidence' choice");
    let inner_choices: Vec<_> = nested_choice.unwrap().content.iter().filter_map(|n| match n {
        ContentNode::Choice(c) => Some(c),
        _ => None,
    }).collect();
    assert!(!inner_choices.is_empty(), "missing nested choice under 'Show evidence'");
    assert_eq!(inner_choices[0].label, "Push further");
}

// ═══════════════════════════════════════════════════════════════════════════
// ReservedPropRef — target.prop and player.prop in narrative conditions
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn target_condition_equality() {
    match first_node("? target.state == closed") {
        ContentNode::Condition(c) => {
            match &c.expr {
                ConditionExpr::PropertyComparison(pc) => {
                    assert_eq!(pc.entity_ref, "target");
                    assert_eq!(pc.property, "state");
                    assert_eq!(pc.operator, "==");
                    assert_eq!(pc.value, "closed");
                }
                other => panic!("expected PropertyComparison, got {:?}", other),
            }
        }
        other => panic!("expected Condition, got {:?}", other),
    }
}

#[test]
fn target_condition_boolean() {
    match first_node("? target.chosen == false") {
        ContentNode::Condition(c) => {
            match &c.expr {
                ConditionExpr::PropertyComparison(pc) => {
                    assert_eq!(pc.entity_ref, "target");
                    assert_eq!(pc.property, "chosen");
                    assert_eq!(pc.operator, "==");
                    assert_eq!(pc.value, "false");
                }
                other => panic!("expected PropertyComparison, got {:?}", other),
            }
        }
        other => panic!("expected Condition, got {:?}", other),
    }
}

#[test]
fn player_condition_greater_than() {
    match first_node("? player.health > 0") {
        ContentNode::Condition(c) => {
            match &c.expr {
                ConditionExpr::PropertyComparison(pc) => {
                    assert_eq!(pc.entity_ref, "player");
                    assert_eq!(pc.property, "health");
                    assert_eq!(pc.operator, ">");
                    assert_eq!(pc.value, "0");
                }
                other => panic!("expected PropertyComparison, got {:?}", other),
            }
        }
        other => panic!("expected Condition, got {:?}", other),
    }
}

#[test]
fn player_condition_not_equal() {
    match first_node("? player.alive != false") {
        ContentNode::Condition(c) => {
            match &c.expr {
                ConditionExpr::PropertyComparison(pc) => {
                    assert_eq!(pc.entity_ref, "player");
                    assert_eq!(pc.property, "alive");
                    assert_eq!(pc.operator, "!=");
                    assert_eq!(pc.value, "false");
                }
                other => panic!("expected PropertyComparison, got {:?}", other),
            }
        }
        other => panic!("expected Condition, got {:?}", other),
    }
}

#[test]
fn target_condition_in_rule_where_clause() {
    let source = r#"---
world:
  name: Test
types:
  Door [interactable]:
    prize: enum(goat, car)
entities:
  @door_1: Door { prize: "goat" }
  @door_2: Door { prize: "car" }
---

# Stage

[@door_1, @door_2]

rule test_rule:
  selects target from [@door_1, @door_2]
    where target.prize != car
  > reveal @door_1.prize
"#;
    let (ast, diag) = parse_source(source);
    assert!(!diag.has_errors(), "rule block with target.prize should not produce errors: {:?}", diag.all());
    let ast = ast.unwrap();
    let rule = ast.content.iter().find_map(|n| match n {
        ContentNode::RuleBlock(r) => Some(r),
        _ => None,
    });
    assert!(rule.is_some(), "should have a rule block");
    let select = rule.unwrap().select.as_ref().expect("rule should have select clause");
    assert_eq!(select.where_clauses.len(), 1, "select should have one where clause");
    match &select.where_clauses[0] {
        ConditionExpr::PropertyComparison(pc) => {
            assert_eq!(pc.entity_ref, "target");
            assert_eq!(pc.property, "prize");
            assert_eq!(pc.operator, "!=");
            assert_eq!(pc.value, "car");
        }
        other => panic!("expected PropertyComparison, got {:?}", other),
    }
}

#[test]
fn target_effect_parses_as_set() {
    match first_node("> target.state = open") {
        ContentNode::Effect(e) => {
            match &e.effect_type {
                EffectType::Set { target_prop, operator, value_expr } => {
                    assert_eq!(target_prop, "target.state");
                    assert_eq!(operator, "=");
                    assert_eq!(value_expr, "open");
                }
                other => panic!("expected Set effect, got {:?}", other),
            }
        }
        other => panic!("expected Effect, got {:?}", other),
    }
}

#[test]
fn non_reserved_bare_identifier_rejected() {
    // Only "target" and "player" are reserved in narrative scope.
    // "door.state == closed" should not parse as a condition.
    let (_, diag) = parse_source("? door.state == closed");
    assert!(diag.has_errors(), "bare non-reserved identifier should produce an error");
    let errors: Vec<_> = diag.all().iter().filter(|d| d.code == "URD112").collect();
    assert!(!errors.is_empty(), "expected URD112 for non-reserved bare identifier");
}

#[test]
fn target_condition_in_choice_guard() {
    let source = "* Pick a door -> any Door\n  ? target.state == closed";
    let nodes = parse_content_only(source);
    let choice = match &nodes[0] {
        ContentNode::Choice(c) => c,
        other => panic!("expected Choice, got {:?}", other),
    };
    let guard = choice.content.iter().find_map(|n| match n {
        ContentNode::Condition(c) => Some(c),
        _ => None,
    });
    assert!(guard.is_some(), "choice should have a condition guard");
    match &guard.unwrap().expr {
        ConditionExpr::PropertyComparison(pc) => {
            assert_eq!(pc.entity_ref, "target");
            assert_eq!(pc.property, "state");
            assert_eq!(pc.operator, "==");
            assert_eq!(pc.value, "closed");
        }
        other => panic!("expected PropertyComparison, got {:?}", other),
    }
}
